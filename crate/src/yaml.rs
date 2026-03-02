use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

pub type SkillTagMap = BTreeMap<String, Vec<String>>;

pub fn read_skills_yaml(path: &Path) -> Result<SkillTagMap> {
    let content = fs::read_to_string(path).context("failed to read skills.yaml")?;
    let map: SkillTagMap = serde_yaml::from_str(&content).context("failed to parse skills.yaml")?;
    Ok(map)
}

pub fn write_skills_yaml(path: &Path, map: &SkillTagMap) -> Result<()> {
    let content = serde_yaml::to_string(map).context("failed to serialize skills.yaml")?;
    fs::write(path, content).context("failed to write skills.yaml")?;
    Ok(())
}

pub fn read_skills_yaml_or_empty(path: &Path) -> Result<SkillTagMap> {
    match fs::read_to_string(path) {
        Ok(content) => {
            let map: SkillTagMap =
                serde_yaml::from_str(&content).context("failed to parse skills.yaml")?;
            Ok(map)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(e) => Err(anyhow::Error::from(e).context("failed to read skills.yaml")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn roundtrip_yaml() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("skills.yaml");

        let mut map = SkillTagMap::new();
        map.insert("skill-a".into(), vec!["kmp".into(), "compose".into()]);
        map.insert("skill-b".into(), vec![]);

        write_skills_yaml(&path, &map).unwrap();
        let loaded = read_skills_yaml(&path).unwrap();

        assert_eq!(loaded, map);
    }

    #[test]
    fn read_missing_file_returns_error() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.yaml");
        assert!(read_skills_yaml(&path).is_err());
    }

    #[test]
    fn read_or_empty_returns_empty_for_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.yaml");
        let map = read_skills_yaml_or_empty(&path).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn read_or_empty_returns_data_for_existing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("skills.yaml");

        let mut map = SkillTagMap::new();
        map.insert("test".into(), vec!["tag".into()]);
        write_skills_yaml(&path, &map).unwrap();

        let loaded = read_skills_yaml_or_empty(&path).unwrap();
        assert_eq!(loaded.len(), 1);
    }

    #[test]
    fn read_or_empty_propagates_parse_error() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("skills.yaml");
        fs::write(&path, "not: [valid: yaml: {{").unwrap();
        assert!(read_skills_yaml_or_empty(&path).is_err());
    }
}
