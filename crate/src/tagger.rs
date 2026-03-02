use anyhow::Result;

use crate::config::Paths;
use crate::yaml;

/// Read yaml, get mutable tags for a skill, apply mutation, write back if changed.
fn modify_skill_tags<F>(paths: &Paths, skill_name: &str, mutate: F) -> Result<bool>
where
    F: FnOnce(&mut Vec<String>) -> bool,
{
    let mut map = yaml::read_skills_yaml(&paths.skills_yaml)?;
    let tags = map
        .get_mut(skill_name)
        .ok_or_else(|| anyhow::anyhow!("skill '{}' not found in skills.yaml", skill_name))?;
    let changed = mutate(tags);
    if changed {
        yaml::write_skills_yaml(&paths.skills_yaml, &map)?;
    }
    Ok(changed)
}

/// Set tags for a skill (overwrites existing tags).
pub fn set_tags(paths: &Paths, skill_name: &str, tags: &[String]) -> Result<()> {
    let new_tags = tags.to_vec();
    modify_skill_tags(paths, skill_name, |current| {
        *current = new_tags;
        true
    })?;
    println!("{}: [{}]", skill_name, tags.join(", "));
    Ok(())
}

/// Add a single tag to a skill.
pub fn add_tag(paths: &Paths, skill_name: &str, tag: &str) -> Result<()> {
    let tag_owned = tag.to_string();
    let changed = modify_skill_tags(paths, skill_name, |tags| {
        if tags.contains(&tag_owned) {
            return false;
        }
        tags.push(tag_owned);
        tags.sort();
        true
    })?;
    if changed {
        println!("{}: added '{}'", skill_name, tag);
    } else {
        println!("{} already has tag '{}'", skill_name, tag);
    }
    Ok(())
}

/// Remove a single tag from a skill.
pub fn remove_tag(paths: &Paths, skill_name: &str, tag: &str) -> Result<()> {
    let changed = modify_skill_tags(paths, skill_name, |tags| {
        let before = tags.len();
        tags.retain(|t| t != tag);
        tags.len() != before
    })?;
    if changed {
        println!("{}: removed '{}'", skill_name, tag);
    } else {
        println!("{} does not have tag '{}'", skill_name, tag);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Paths) {
        let home = TempDir::new().unwrap();
        let paths = Paths::from_home(home.path());

        fs::create_dir_all(&paths.skill_tree_dir).unwrap();

        let mut map = yaml::SkillTagMap::new();
        map.insert("skill-a".into(), vec!["kmp".into(), "kotlin".into()]);
        map.insert("skill-b".into(), vec![]);
        yaml::write_skills_yaml(&paths.skills_yaml, &map).unwrap();

        (home, paths)
    }

    #[test]
    fn set_tags_overwrites() {
        let (_home, paths) = setup();
        set_tags(&paths, "skill-a", &["new-tag".into()]).unwrap();
        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(map["skill-a"], vec!["new-tag".to_string()]);
    }

    #[test]
    fn set_tags_fails_for_unknown_skill() {
        let (_home, paths) = setup();
        assert!(set_tags(&paths, "nonexistent", &["tag".into()]).is_err());
    }

    #[test]
    fn add_tag_appends() {
        let (_home, paths) = setup();
        add_tag(&paths, "skill-b", "design").unwrap();
        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(map["skill-b"], vec!["design".to_string()]);
    }

    #[test]
    fn add_tag_is_idempotent() {
        let (_home, paths) = setup();
        add_tag(&paths, "skill-a", "kmp").unwrap();
        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(map["skill-a"].iter().filter(|t| *t == "kmp").count(), 1);
    }

    #[test]
    fn add_tag_sorts() {
        let (_home, paths) = setup();
        add_tag(&paths, "skill-a", "aaa").unwrap();
        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(map["skill-a"][0], "aaa");
    }

    #[test]
    fn remove_tag_removes() {
        let (_home, paths) = setup();
        remove_tag(&paths, "skill-a", "kotlin").unwrap();
        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(map["skill-a"], vec!["kmp".to_string()]);
    }

    #[test]
    fn remove_nonexistent_tag_is_noop() {
        let (_home, paths) = setup();
        remove_tag(&paths, "skill-a", "nonexistent").unwrap();
        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(map["skill-a"].len(), 2);
    }

    #[test]
    fn remove_tag_fails_for_unknown_skill() {
        let (_home, paths) = setup();
        assert!(remove_tag(&paths, "nonexistent", "tag").is_err());
    }

    #[test]
    fn modify_unknown_skill_errors() {
        let (_home, paths) = setup();
        let result = modify_skill_tags(&paths, "ghost", |_| true);
        assert!(result.is_err());
    }
}
