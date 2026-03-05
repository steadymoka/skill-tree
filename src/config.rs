use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Paths {
    pub home_dir: PathBuf,
    /// ~/.skilltree/
    pub skill_tree_dir: PathBuf,
    pub skills_yaml: PathBuf,
    pub link_skills_sh: PathBuf,
    pub skill_lock_json: PathBuf,
}

impl Paths {
    pub fn from_home(home: &std::path::Path) -> Self {
        let skill_tree = home.join(".skilltree");
        let skills_yaml = skill_tree.join("skills.yaml");
        let link_skills_sh = skill_tree.join("link-skills.sh");
        let skill_lock_json = skill_tree.join(".skill-lock.json");
        Self {
            home_dir: home.to_path_buf(),
            skill_tree_dir: skill_tree,
            skills_yaml,
            link_skills_sh,
            skill_lock_json,
        }
    }

    pub fn default_paths() -> Result<Self> {
        let home = dirs::home_dir().context("cannot determine home directory")?;
        Ok(Self::from_home(&home))
    }
}

/// Load project paths from ~/.claude.json and ~/.codex/ state DB, merged and deduplicated.
pub fn load_project_paths() -> Vec<String> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };

    let mut all: BTreeSet<String> = BTreeSet::new();
    all.extend(load_claude_project_paths(&home).unwrap_or_default());
    all.extend(load_codex_project_paths(&home).unwrap_or_default());
    all.into_iter().collect()
}

fn load_claude_project_paths(home: &std::path::Path) -> Option<Vec<String>> {
    let content = std::fs::read_to_string(home.join(".claude.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let projects = value.get("projects")?.as_object()?;
    Some(projects.keys().map(|k| k.to_string()).collect())
}

/// Best-effort: read project paths from Codex's SQLite state DB.
/// Schema is undocumented, so failures are silently ignored.
fn load_codex_project_paths(home: &std::path::Path) -> Option<Vec<String>> {
    let db_path = find_latest_state_db(&home.join(".codex"))?;
    let flags =
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = rusqlite::Connection::open_with_flags(&db_path, flags).ok()?;
    let mut stmt = conn.prepare("SELECT DISTINCT cwd FROM threads").ok()?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .ok()?
        .filter_map(|r| r.ok())
        .collect();
    Some(rows)
}

/// Find the highest-versioned state_N.sqlite in the given directory.
fn find_latest_state_db(dir: &std::path::Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let version = name
                .strip_prefix("state_")?
                .strip_suffix(".sqlite")?
                .parse::<u32>()
                .ok()?;
            Some((version, e.path()))
        })
        .max_by_key(|(v, _)| *v)
        .map(|(_, path)| path)
}
