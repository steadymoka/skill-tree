use std::path::Path;
use std::str::FromStr;

use anyhow::Result;

pub const ALL_TOOLS: [Tool; 3] = [Tool::Claude, Tool::Codex, Tool::Agents];

/// Tools whose project-level skills directories are managed (link/unlink targets).
pub const LINKABLE_TOOLS: [Tool; 2] = [Tool::Claude, Tool::Codex];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tool {
    Claude,
    Codex,
    Agents,
}

impl Tool {
    pub fn skills_subdir(&self) -> &'static str {
        match self {
            Tool::Claude => ".claude/skills",
            Tool::Codex => ".codex/skills",
            Tool::Agents => ".agents/skills",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Tool::Claude => "claude",
            Tool::Codex => "codex",
            Tool::Agents => "agents",
        }
    }
}

impl FromStr for Tool {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(Tool::Claude),
            "codex" => Ok(Tool::Codex),
            "agents" => Ok(Tool::Agents),
            _ => anyhow::bail!("unknown tool: {} (expected: claude, codex, agents)", s),
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

/// Remove a filesystem entry regardless of type (file, symlink, or directory).
pub fn remove_entry(path: &Path) -> Result<()> {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };
    if meta.is_dir() && !meta.file_type().is_symlink() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Recursively copy a directory, skipping .git.
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        let src_path = entry.path();
        let dst_path = dst.join(&name);
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
