use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::config::Paths;
use crate::fs_util::{self, Tool, ALL_TOOLS};
use crate::lock::{self, SkillLockEntry};
use crate::yaml;

#[derive(Debug)]
pub struct SkillInfo {
    pub name: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub lock_entry: Option<SkillLockEntry>,
    pub linked_projects: Vec<LinkedProject>,
}

#[derive(Debug)]
pub struct LinkedProject {
    pub name: String,
    pub tools: Vec<Tool>,
}

/// Gather info about a skill.
pub fn get_skill_info(paths: &Paths, name: &str, project_paths: &[String]) -> Result<SkillInfo> {
    let skill_dir = paths.skill_tree_dir.join(name);
    let map = yaml::read_skills_yaml_or_empty(&paths.skills_yaml)?;

    if !skill_dir.exists() && !map.contains_key(name) {
        anyhow::bail!("skill '{}' not found", name);
    }

    let tags = map.get(name).cloned().unwrap_or_default();

    let skill_lock = lock::read_lock(&paths.skill_lock_json)?;
    let lock_entry = skill_lock.get(name).cloned();

    let mut linked_projects = Vec::new();
    for project in project_paths {
        let project_path = Path::new(project);
        let mut tools = Vec::new();
        for tool in &ALL_TOOLS {
            let link_path = fs_util::project_skills_dir(project_path, *tool).join(name);
            if let Ok(meta) = std::fs::symlink_metadata(&link_path) {
                if meta.file_type().is_symlink() {
                    tools.push(*tool);
                }
            }
        }
        if !tools.is_empty() {
            let project_name = fs_util::basename(project).to_string();
            linked_projects.push(LinkedProject {
                name: project_name,
                tools,
            });
        }
    }

    Ok(SkillInfo {
        name: name.to_string(),
        path: skill_dir,
        tags,
        lock_entry,
        linked_projects,
    })
}

/// Print skill info to stdout.
pub fn print_info(paths: &Paths, name: &str, project_paths: &[String]) -> Result<()> {
    let info = get_skill_info(paths, name, project_paths)?;

    println!("Skill: {}", info.name);
    println!("Path:  {}", info.path.display());

    if let Some(entry) = &info.lock_entry {
        println!("Source:     {}", entry.source);
        println!("SHA:        {}", entry.installed_sha);
        println!("Git ref:    {}", entry.git_ref);
        println!("Skill path: {}", entry.skill_path);
        println!("Installed:  {}", entry.installed_at);
    } else {
        println!("Source: local");
    }

    if info.tags.is_empty() {
        println!("Tags:  (none)");
    } else {
        println!("Tags:  [{}]", info.tags.join(", "));
    }

    if info.linked_projects.is_empty() {
        println!("Linked: (none)");
    } else {
        println!("Linked projects:");
        for lp in &info.linked_projects {
            let tools: Vec<&str> = lp.tools.iter().map(|t| t.short_label()).collect();
            println!("  {} ({})", lp.name, tools.join(", "));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Paths;
    use crate::linker;
    use crate::lock::{SkillLock, SkillLockEntry};
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Paths) {
        let home = TempDir::new().unwrap();
        let paths = Paths::from_home(home.path());

        fs::create_dir_all(paths.skill_tree_dir.join("skill-a")).unwrap();

        let mut map = yaml::SkillTagMap::new();
        map.insert("skill-a".into(), vec!["kmp".into(), "kotlin".into()]);
        yaml::write_skills_yaml(&paths.skills_yaml, &map).unwrap();

        let mut skill_lock = SkillLock::new();
        skill_lock.insert(
            "skill-a".into(),
            SkillLockEntry::new("user/repo", "skills/a", "main", "abc123"),
        );
        lock::write_lock(&paths.skill_lock_json, &skill_lock).unwrap();

        (home, paths)
    }

    #[test]
    fn info_returns_full_metadata() {
        let (_home, paths) = setup();

        let info = get_skill_info(&paths, "skill-a", &[]).unwrap();

        assert_eq!(info.name, "skill-a");
        assert_eq!(info.tags, vec!["kmp", "kotlin"]);
        let entry = info.lock_entry.as_ref().unwrap();
        assert_eq!(entry.source, "user/repo");
        assert_eq!(entry.installed_sha, "abc123");
        assert_eq!(entry.git_ref, "main");
        assert_eq!(entry.skill_path, "skills/a");
    }

    #[test]
    fn info_with_missing_lock_entry() {
        let (_home, paths) = setup();
        // Add a skill without lock entry
        fs::create_dir_all(paths.skill_tree_dir.join("local-skill")).unwrap();
        let mut map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        map.insert("local-skill".into(), vec!["dev".into()]);
        yaml::write_skills_yaml(&paths.skills_yaml, &map).unwrap();

        let info = get_skill_info(&paths, "local-skill", &[]).unwrap();

        assert_eq!(info.name, "local-skill");
        assert!(info.lock_entry.is_none());
        assert_eq!(info.tags, vec!["dev"]);
    }

    #[test]
    fn info_nonexistent_skill_returns_error() {
        let (_home, paths) = setup();

        let result = get_skill_info(&paths, "nonexistent", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn info_shows_linked_projects() {
        let (_home, paths) = setup();
        let project = TempDir::new().unwrap();
        linker::link_skill(&paths, project.path(), "skill-a", Tool::Claude).unwrap();

        let project_paths = vec![project.path().to_string_lossy().to_string()];
        let info = get_skill_info(&paths, "skill-a", &project_paths).unwrap();

        assert_eq!(info.linked_projects.len(), 1);
        assert_eq!(info.linked_projects[0].tools, vec![Tool::Claude]);
    }

    #[test]
    fn info_no_linked_projects() {
        let (_home, paths) = setup();

        let info = get_skill_info(&paths, "skill-a", &[]).unwrap();

        assert!(info.linked_projects.is_empty());
    }

    #[test]
    fn info_skill_in_yaml_but_no_dir() {
        let (_home, paths) = setup();
        // Add to yaml without creating directory
        let mut map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        map.insert("ghost".into(), vec![]);
        yaml::write_skills_yaml(&paths.skills_yaml, &map).unwrap();

        let info = get_skill_info(&paths, "ghost", &[]).unwrap();

        assert_eq!(info.name, "ghost");
        let dir_exists = info.path.exists();
        assert!(!dir_exists);
    }
}
