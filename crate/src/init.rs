use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::Paths;
use crate::fs_util;
use crate::scanner;
use crate::yaml;

const LINK_SKILLS_SH: &str = r#"#!/usr/bin/env bash
set -euo pipefail

CENTRAL="$HOME/.skill-tree"
YAML="$CENTRAL/skills.yaml"
TOOL="${TOOL:-claude}"

case "$TOOL" in
  claude) TARGET=".claude/skills" ;;
  codex)  TARGET=".codex/skills" ;;
  *)      echo "Unknown tool: $TOOL"; exit 1 ;;
esac

if [ $# -eq 0 ]; then
  echo "Usage: [TOOL=claude|codex] link-skills <tag1> [tag2] ..."
  exit 1
fi

mkdir -p "$TARGET"
linked=0
for tag in "$@"; do
  while IFS=: read -r skill tags; do
    skill=$(echo "$skill" | tr -d ' ')
    [ -z "$skill" ] && continue
    [[ "$skill" == \#* ]] && continue
    if echo "$tags" | grep -qw "$tag"; then
      if [ -d "$CENTRAL/$skill" ] && [ ! -e "$TARGET/$skill" ]; then
        ln -s "$CENTRAL/$skill" "$TARGET/$skill"
        echo "  Linked: $skill"
        linked=$((linked + 1))
      fi
    fi
  done < "$YAML"
done
echo "Done. $linked skill(s) linked."
"#;

pub fn initialize(paths: &Paths) -> Result<()> {
    let central = &paths.skill_tree_dir;
    let skills = &paths.skills_dir;
    let yaml_path = &paths.skills_yaml;
    let sh_path = &paths.link_skills_sh;

    // 0. Migrate legacy ~/.claude/skills-central/ → ~/.skill-tree/
    if let Some(parent) = skills.parent() {
        let legacy = parent.join("skills-central");
        if legacy.exists() && !central.exists() {
            fs::rename(&legacy, central)
                .context("failed to migrate ~/.claude/skills-central/ to ~/.skill-tree/")?;
            println!("Migrated: ~/.claude/skills-central/ → ~/.skill-tree/");
        }
    }

    // 1. Create ~/.skill-tree/ if needed
    if !central.exists() {
        fs::create_dir_all(central).context("failed to create ~/.skill-tree/")?;
        println!("Created {}", central.display());
    }

    // 2. Move real directories from ~/.claude/skills/ to ~/.skill-tree/
    //    and leave symlinks in their place
    if skills.exists() {
        let real_dirs = scanner::scan_real_dirs(skills)?;
        for name in &real_dirs {
            let src = skills.join(name);
            let dst = central.join(name);
            if dst.exists() {
                continue;
            }
            fs::rename(&src, &dst)
                .with_context(|| format!("failed to move {} to skill-tree", name))?;
            fs_util::create_symlink(&dst, &src)
                .with_context(|| format!("failed to create global symlink for {}", name))?;
            println!("Moved: {} → skill-tree (symlink left in skills/)", name);
        }
    }

    // 3. Generate/merge skills.yaml
    let mut map = yaml::read_skills_yaml_or_empty(yaml_path)?;
    let central_dirs = scanner::scan_skill_dirs(central)?;
    let mut new_count = 0;
    for name in &central_dirs {
        if !map.contains_key(name) {
            map.insert(name.clone(), Vec::new());
            new_count += 1;
        }
    }
    yaml::write_skills_yaml(yaml_path, &map)?;
    if new_count > 0 {
        println!(
            "Updated skills.yaml: {} new skill(s) added ({} total)",
            new_count,
            map.len()
        );
    } else {
        println!("skills.yaml is up to date ({} skills)", map.len());
    }

    // 4. Generate link-skills.sh if missing
    if !sh_path.exists() {
        fs::write(sh_path, LINK_SKILLS_SH).context("failed to write link-skills.sh")?;
        #[cfg(unix)]
        set_executable(sh_path)?;
        println!("Created link-skills.sh");
    }

    println!("Initialization complete.");
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_paths(tmp: &TempDir) -> Paths {
        Paths::from_home(tmp.path())
    }

    #[test]
    fn init_creates_central_and_yaml_from_empty() {
        let tmp = TempDir::new().unwrap();
        let paths = test_paths(&tmp);

        initialize(&paths).unwrap();

        assert!(paths.skill_tree_dir.exists());
        assert!(paths.skills_yaml.exists());
        assert!(paths.link_skills_sh.exists());

        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn init_moves_skills_from_skills_dir() {
        let tmp = TempDir::new().unwrap();
        let paths = test_paths(&tmp);

        let skill_dir = paths.skills_dir.join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\n---\ncontent").unwrap();

        initialize(&paths).unwrap();

        assert!(paths.skill_tree_dir.join("my-skill").join("SKILL.md").exists());

        let link_meta = fs::symlink_metadata(paths.skills_dir.join("my-skill")).unwrap();
        assert!(link_meta.file_type().is_symlink());

        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert!(map.contains_key("my-skill"));
        assert!(map["my-skill"].is_empty());
    }

    #[test]
    fn init_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let paths = test_paths(&tmp);

        fs::create_dir_all(paths.skill_tree_dir.join("skill-a")).unwrap();

        initialize(&paths).unwrap();
        let map1 = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(map1.len(), 1);

        initialize(&paths).unwrap();
        let map2 = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(map2.len(), 1);
    }

    #[test]
    fn init_merges_new_skills_into_existing_yaml() {
        let tmp = TempDir::new().unwrap();
        let paths = test_paths(&tmp);

        fs::create_dir_all(paths.skill_tree_dir.join("skill-a")).unwrap();
        let mut map = yaml::SkillTagMap::new();
        map.insert("skill-a".into(), vec!["tag1".into()]);
        fs::create_dir_all(paths.skills_yaml.parent().unwrap()).unwrap();
        yaml::write_skills_yaml(&paths.skills_yaml, &map).unwrap();

        fs::create_dir_all(paths.skill_tree_dir.join("skill-b")).unwrap();

        initialize(&paths).unwrap();

        let updated = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(updated.len(), 2);
        assert_eq!(updated["skill-a"], vec!["tag1".to_string()]);
        assert!(updated["skill-b"].is_empty());
    }

    #[test]
    fn init_skips_move_if_already_in_central() {
        let tmp = TempDir::new().unwrap();
        let paths = test_paths(&tmp);

        fs::create_dir_all(paths.skills_dir.join("skill-a")).unwrap();
        fs::create_dir_all(paths.skill_tree_dir.join("skill-a")).unwrap();

        initialize(&paths).unwrap();

        let meta = fs::symlink_metadata(paths.skills_dir.join("skill-a")).unwrap();
        assert!(!meta.file_type().is_symlink());
    }
}
