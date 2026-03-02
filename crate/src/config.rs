use std::path::PathBuf;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Paths {
    /// Legacy ~/.claude/skills (migration source)
    pub skills_dir: PathBuf,
    /// ~/.skill-tree/
    pub skill_tree_dir: PathBuf,
    pub skills_yaml: PathBuf,
    pub link_skills_sh: PathBuf,
}

impl Paths {
    pub fn from_home(home: &std::path::Path) -> Self {
        let skill_tree = home.join(".skill-tree");
        let skills_yaml = skill_tree.join("skills.yaml");
        let link_skills_sh = skill_tree.join("link-skills.sh");
        Self {
            skills_dir: home.join(".claude").join("skills"),
            skill_tree_dir: skill_tree,
            skills_yaml,
            link_skills_sh,
        }
    }

    pub fn default_paths() -> Result<Self> {
        let home = dirs::home_dir().context("cannot determine home directory")?;
        Ok(Self::from_home(&home))
    }
}
