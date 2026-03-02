use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::Paths;
use crate::fs_util::{self, Tool};
use crate::yaml;

/// Link all skills matching any of the given tags to a project directory.
pub fn link_by_tags(paths: &Paths, project_path: &Path, tags: &[String], tool: Tool) -> Result<()> {
    let map = yaml::read_skills_yaml(&paths.skills_yaml)?;
    let tag_set: HashSet<&str> = tags.iter().map(|s| s.as_str()).collect();

    let target_dir = fs_util::project_skills_dir(project_path, tool);
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("failed to create {}", tool.skills_subdir()))?;

    let mut linked = 0;
    for (skill, skill_tags) in &map {
        let matches = skill_tags.iter().any(|t| tag_set.contains(t.as_str()));
        if !matches {
            continue;
        }

        let link_path = target_dir.join(skill);
        if link_path.exists() {
            continue;
        }

        let original = paths.skill_tree_dir.join(skill);
        if !original.exists() {
            eprintln!("Warning: skill directory not found: {}", skill);
            continue;
        }

        fs_util::create_symlink(&original, &link_path)
            .with_context(|| format!("failed to link {}", skill))?;
        println!("  Linked: {}", skill);
        linked += 1;
    }

    println!("Done. {} skill(s) linked.", linked);
    Ok(())
}

/// Link a single skill by name to a project directory.
pub fn link_skill(paths: &Paths, project_path: &Path, skill_name: &str, tool: Tool) -> Result<()> {
    let target_dir = fs_util::project_skills_dir(project_path, tool);
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("failed to create {}", tool.skills_subdir()))?;

    let link_path = target_dir.join(skill_name);
    if link_path.exists() {
        return Ok(());
    }

    let original = paths.skill_tree_dir.join(skill_name);
    if !original.exists() {
        anyhow::bail!("skill directory not found: {}", skill_name);
    }

    fs_util::create_symlink(&original, &link_path)
        .with_context(|| format!("failed to link {}", skill_name))?;
    Ok(())
}

/// Unlink a single skill from a project.
pub fn unlink_skill(project_path: &Path, skill_name: &str, tool: Tool) -> Result<()> {
    let link_path = fs_util::project_skills_dir(project_path, tool).join(skill_name);

    let meta = match fs::symlink_metadata(&link_path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("{} is not linked in this project.", skill_name);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    if !meta.file_type().is_symlink() {
        anyhow::bail!(
            "{} is a real directory, not a symlink. Refusing to remove.",
            skill_name
        );
    }

    fs::remove_file(&link_path).with_context(|| format!("failed to unlink {}", skill_name))?;
    println!("Unlinked: {}", skill_name);
    Ok(())
}

/// Unlink all skill symlinks from a project.
pub fn unlink_all(project_path: &Path, tool: Tool) -> Result<()> {
    let skills_dir = fs_util::project_skills_dir(project_path, tool);
    if !skills_dir.exists() {
        println!("No {} directory in this project.", tool.skills_subdir());
        return Ok(());
    }

    let mut removed = 0;
    for entry in fs::read_dir(&skills_dir)? {
        let entry = entry?;
        let meta = fs::symlink_metadata(entry.path())?;
        if meta.file_type().is_symlink() {
            fs::remove_file(entry.path())?;
            let name = entry.file_name().to_string_lossy().into_owned();
            println!("  Unlinked: {}", name);
            removed += 1;
        }
    }

    println!("Done. {} skill(s) unlinked.", removed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Paths;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Paths, TempDir) {
        let home = TempDir::new().unwrap();
        let paths = Paths::from_home(home.path());

        fs::create_dir_all(paths.skill_tree_dir.join("skill-a")).unwrap();
        fs::create_dir_all(paths.skill_tree_dir.join("skill-b")).unwrap();
        fs::create_dir_all(paths.skill_tree_dir.join("skill-c")).unwrap();

        let mut map = yaml::SkillTagMap::new();
        map.insert("skill-a".into(), vec!["kmp".into(), "kotlin".into()]);
        map.insert("skill-b".into(), vec!["kmp".into()]);
        map.insert("skill-c".into(), vec!["design".into()]);
        yaml::write_skills_yaml(&paths.skills_yaml, &map).unwrap();

        let project = TempDir::new().unwrap();
        (home, paths, project)
    }

    const T: Tool = Tool::Claude;

    #[test]
    fn link_by_single_tag() {
        let (_home, paths, project) = setup();

        link_by_tags(&paths, project.path(), &["design".into()], T).unwrap();

        let skills_dir = fs_util::project_skills_dir(project.path(), T);
        assert!(fs::symlink_metadata(skills_dir.join("skill-c"))
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(!skills_dir.join("skill-a").exists());
    }

    #[test]
    fn link_by_multiple_tags_union() {
        let (_home, paths, project) = setup();

        link_by_tags(&paths, project.path(), &["kmp".into(), "design".into()], T).unwrap();

        let skills_dir = fs_util::project_skills_dir(project.path(), T);
        assert!(skills_dir.join("skill-a").exists());
        assert!(skills_dir.join("skill-b").exists());
        assert!(skills_dir.join("skill-c").exists());
    }

    #[test]
    fn link_skips_already_linked() {
        let (_home, paths, project) = setup();

        link_by_tags(&paths, project.path(), &["kmp".into()], T).unwrap();
        link_by_tags(&paths, project.path(), &["kmp".into()], T).unwrap();

        let skills_dir = fs_util::project_skills_dir(project.path(), T);
        assert!(skills_dir.join("skill-a").exists());
    }

    #[test]
    fn link_with_no_matching_tag() {
        let (_home, paths, project) = setup();

        link_by_tags(&paths, project.path(), &["nonexistent".into()], T).unwrap();

        let skills_dir = fs_util::project_skills_dir(project.path(), T);
        assert!(skills_dir.exists());
        assert_eq!(fs::read_dir(&skills_dir).unwrap().count(), 0);
    }

    #[test]
    fn unlink_single_skill() {
        let (_home, paths, project) = setup();

        link_by_tags(&paths, project.path(), &["kmp".into()], T).unwrap();
        unlink_skill(project.path(), "skill-a", T).unwrap();

        let skills_dir = fs_util::project_skills_dir(project.path(), T);
        assert!(!skills_dir.join("skill-a").exists());
        assert!(skills_dir.join("skill-b").exists());
    }

    #[test]
    fn unlink_nonexistent_skill_is_noop() {
        let (_home, _paths, project) = setup();
        unlink_skill(project.path(), "nonexistent", T).unwrap();
    }

    #[test]
    fn unlink_refuses_real_directory() {
        let (_home, _paths, project) = setup();
        let skills_dir = fs_util::project_skills_dir(project.path(), T);
        fs::create_dir_all(skills_dir.join("real-dir")).unwrap();

        let result = unlink_skill(project.path(), "real-dir", T);
        assert!(result.is_err());
    }

    #[test]
    fn unlink_all_removes_only_symlinks() {
        let (_home, paths, project) = setup();

        link_by_tags(&paths, project.path(), &["kmp".into()], T).unwrap();
        let skills_dir = fs_util::project_skills_dir(project.path(), T);
        fs::create_dir_all(skills_dir.join("real-dir")).unwrap();

        unlink_all(project.path(), T).unwrap();

        assert!(!skills_dir.join("skill-a").exists());
        assert!(!skills_dir.join("skill-b").exists());
        assert!(skills_dir.join("real-dir").exists());
    }
}
