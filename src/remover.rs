use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::config::Paths;
use crate::fs_util::LINKABLE_TOOLS;
use crate::linker;
use crate::lock;
use crate::yaml;

/// Remove a skill completely: directory, yaml entry, lock entry, and project symlinks.
pub fn remove_skill(paths: &Paths, name: &str, project_paths: &[String]) -> Result<()> {
    let skill_dir = paths.skill_tree_dir.join(name);
    let mut map = yaml::read_skills_yaml_or_empty(&paths.skills_yaml)?;

    if !skill_dir.exists() && !map.contains_key(name) {
        anyhow::bail!("skill '{}' not found", name);
    }

    // Clean up symlinks from all projects (both claude and codex)
    for project in project_paths {
        let project_path = Path::new(project);
        for tool in &LINKABLE_TOOLS {
            let _ = linker::unlink_skill(project_path, name, *tool);
        }
    }

    // Remove from skills.yaml
    map.remove(name);
    yaml::write_skills_yaml(&paths.skills_yaml, &map)?;

    // Remove from .skill-lock.json
    let mut skill_lock = lock::read_lock(&paths.skill_lock_json)?;
    skill_lock.remove(name);
    lock::write_lock(&paths.skill_lock_json, &skill_lock)?;

    // Delete the skill directory
    match fs::remove_dir_all(&skill_dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            return Err(anyhow::Error::from(e)
                .context(format!("failed to delete skill directory: {}", name)))
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Paths;
    use crate::fs_util::{self, Tool};
    use crate::lock::{SkillLock, SkillLockEntry};
    use tempfile::TempDir;

    fn setup() -> (TempDir, Paths) {
        let home = TempDir::new().unwrap();
        let paths = Paths::from_home(home.path());

        fs::create_dir_all(paths.skill_tree_dir.join("skill-a")).unwrap();
        fs::create_dir_all(paths.skill_tree_dir.join("skill-b")).unwrap();

        let mut map = yaml::SkillTagMap::new();
        map.insert("skill-a".into(), vec!["kmp".into()]);
        map.insert("skill-b".into(), vec!["design".into()]);
        yaml::write_skills_yaml(&paths.skills_yaml, &map).unwrap();

        let mut skill_lock = SkillLock::new();
        skill_lock.insert(
            "skill-a".into(),
            SkillLockEntry::new("user/repo-a", ".", "main", "sha-a"),
        );
        skill_lock.insert(
            "skill-b".into(),
            SkillLockEntry::new("user/repo-b", ".", "main", "sha-b"),
        );
        lock::write_lock(&paths.skill_lock_json, &skill_lock).unwrap();

        (home, paths)
    }

    #[test]
    fn remove_deletes_dir_and_yaml_and_lock() {
        let (_home, paths) = setup();

        remove_skill(&paths, "skill-a", &[]).unwrap();

        assert!(!paths.skill_tree_dir.join("skill-a").exists());

        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert!(!map.contains_key("skill-a"));
        assert!(map.contains_key("skill-b"));

        let skill_lock = lock::read_lock(&paths.skill_lock_json).unwrap();
        assert!(skill_lock.get("skill-a").is_none());
        assert!(skill_lock.get("skill-b").is_some());
    }

    #[test]
    fn remove_cleans_up_project_symlinks() {
        let (_home, paths) = setup();
        let project = TempDir::new().unwrap();

        linker::link_skill(&paths, project.path(), "skill-a", Tool::Claude).unwrap();
        let link_path = fs_util::project_skills_dir(project.path(), Tool::Claude).join("skill-a");
        assert!(link_path.exists());

        let project_paths = vec![project.path().to_string_lossy().to_string()];
        remove_skill(&paths, "skill-a", &project_paths).unwrap();

        assert!(!link_path.exists());
    }

    #[test]
    fn remove_nonexistent_skill_returns_error() {
        let (_home, paths) = setup();

        let result = remove_skill(&paths, "nonexistent", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn remove_preserves_other_skills() {
        let (_home, paths) = setup();

        remove_skill(&paths, "skill-a", &[]).unwrap();

        assert!(paths.skill_tree_dir.join("skill-b").exists());
        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("skill-b"));
    }

    #[test]
    fn remove_skill_in_yaml_but_no_dir() {
        let (_home, paths) = setup();
        // Remove directory but keep yaml entry
        fs::remove_dir_all(paths.skill_tree_dir.join("skill-a")).unwrap();

        remove_skill(&paths, "skill-a", &[]).unwrap();

        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert!(!map.contains_key("skill-a"));
    }

    #[test]
    fn remove_skill_dir_but_not_in_yaml() {
        let (_home, paths) = setup();
        // Create a dir not in yaml
        fs::create_dir_all(paths.skill_tree_dir.join("orphan")).unwrap();

        remove_skill(&paths, "orphan", &[]).unwrap();

        assert!(!paths.skill_tree_dir.join("orphan").exists());
    }

    #[test]
    fn remove_with_no_lock_entry() {
        let (_home, paths) = setup();
        // Create a skill only in yaml (no lock entry)
        fs::create_dir_all(paths.skill_tree_dir.join("local-skill")).unwrap();
        let mut map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        map.insert("local-skill".into(), vec![]);
        yaml::write_skills_yaml(&paths.skills_yaml, &map).unwrap();

        remove_skill(&paths, "local-skill", &[]).unwrap();

        assert!(!paths.skill_tree_dir.join("local-skill").exists());
        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert!(!map.contains_key("local-skill"));
    }
}
