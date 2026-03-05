use std::path::Path;
use std::str::FromStr;

use anyhow::Result;

pub const ALL_TOOLS: [Tool; 2] = [Tool::Claude, Tool::Codex];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tool {
    Claude,
    Codex,
}

impl Tool {
    pub fn skills_subdir(&self) -> &'static str {
        match self {
            Tool::Claude => ".claude/skills",
            Tool::Codex => ".codex/skills",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Tool::Claude => "claude",
            Tool::Codex => "codex",
        }
    }
}

impl FromStr for Tool {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(Tool::Claude),
            "codex" => Ok(Tool::Codex),
            _ => anyhow::bail!("unknown tool: {} (expected: claude, codex)", s),
        }
    }
}

impl std::fmt::Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.short_label())
    }
}

#[cfg(unix)]
pub fn create_symlink(original: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(original, link)?;
    Ok(())
}

#[cfg(not(unix))]
pub fn create_symlink(_original: &Path, _link: &Path) -> Result<()> {
    anyhow::bail!("symlinks are only supported on Unix systems");
}

pub fn basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

pub fn project_skills_dir(project_path: &Path, tool: Tool) -> std::path::PathBuf {
    project_path.join(tool.skills_subdir())
}

const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "go.mod",
    ".claude",
    ".codex",
];

pub fn is_project_dir(path: &Path) -> bool {
    PROJECT_MARKERS.iter().any(|m| path.join(m).exists())
}
