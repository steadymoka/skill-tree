use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::fs_util::{self, Tool};

/// List skill directory names inside a given parent directory.
/// Only includes entries that are directories (real or symlink targets).
pub fn scan_skill_dirs(dir: &Path) -> Result<Vec<String>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();

        if name.starts_with('.') {
            continue;
        }

        let path = entry.path();
        if path.is_dir() {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

/// List skill directory names that are real directories (not symlinks).
pub fn scan_real_dirs(dir: &Path) -> Result<Vec<String>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();

        if name.starts_with('.') {
            continue;
        }

        let meta = fs::symlink_metadata(entry.path())?;
        if meta.is_dir() {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

/// List skill directory names that are symlinks in a project's tool-specific skills dir.
pub fn scan_linked_skills(project_path: &Path, tool: Tool) -> Vec<String> {
    let skills_dir = fs_util::project_skills_dir(project_path, tool);
    if !skills_dir.exists() {
        return Vec::new();
    }

    let mut names = Vec::new();
    let entries = match fs::read_dir(&skills_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        if let Ok(meta) = fs::symlink_metadata(entry.path()) {
            if meta.file_type().is_symlink() {
                names.push(name);
            }
        }
    }
    names.sort();
    names
}

/// Find entries in each tool's skills path that are NOT managed by skilltree.
/// An entry is "managed" only if it is a symlink pointing to ~/.skilltree/<name>.
pub fn scan_unmanaged_skills(home: &Path) -> Result<Vec<(Tool, Vec<String>)>> {
    let central = home.join(".skilltree");
    let canonical_central = central.canonicalize().unwrap_or(central.clone());
    let mut result = Vec::new();
    for tool in fs_util::ALL_TOOLS {
        let dir = home.join(tool.skills_subdir());
        if !dir.exists() {
            continue;
        }
        let mut unmanaged = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            if is_skilltree_managed(&entry.path(), &canonical_central.join(&name)) {
                continue;
            }
            unmanaged.push(name);
        }
        unmanaged.sort();
        if !unmanaged.is_empty() {
            result.push((tool, unmanaged));
        }
    }
    Ok(result)
}

/// Check if a path is a symlink whose target resolves to the expected central path.
fn is_skilltree_managed(path: &Path, expected_central: &Path) -> bool {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if !meta.file_type().is_symlink() {
        return false;
    }
    match path.canonicalize() {
        Ok(actual) => actual == *expected_central,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    #[test]
    fn scan_skill_dirs_returns_sorted_dirs() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("charlie")).unwrap();
        fs::create_dir(tmp.path().join("alpha")).unwrap();
        fs::create_dir(tmp.path().join("bravo")).unwrap();
        // Files should be excluded
        fs::write(tmp.path().join("readme.txt"), "hello").unwrap();

        let dirs = scan_skill_dirs(tmp.path()).unwrap();
        assert_eq!(dirs, vec!["alpha", "bravo", "charlie"]);
    }

    #[test]
    fn scan_skill_dirs_skips_hidden() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".hidden")).unwrap();
        fs::create_dir(tmp.path().join("visible")).unwrap();

        let dirs = scan_skill_dirs(tmp.path()).unwrap();
        assert_eq!(dirs, vec!["visible"]);
    }

    #[test]
    fn scan_skill_dirs_returns_empty_for_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let dirs = scan_skill_dirs(&tmp.path().join("nope")).unwrap();
        assert!(dirs.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn scan_real_dirs_excludes_symlinks() {
        let tmp = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("real")).unwrap();
        symlink(target.path(), tmp.path().join("linked")).unwrap();

        let dirs = scan_real_dirs(tmp.path()).unwrap();
        assert_eq!(dirs, vec!["real"]);
    }

    #[test]
    #[cfg(unix)]
    fn scan_linked_skills_finds_symlinks() {
        let tmp = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let skills_dir = tmp.path().join(".claude").join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        symlink(target.path(), skills_dir.join("my-skill")).unwrap();
        // Real dir should be excluded
        fs::create_dir(skills_dir.join("real-dir")).unwrap();

        let linked = scan_linked_skills(tmp.path(), Tool::Claude);
        assert_eq!(linked, vec!["my-skill"]);
    }

    #[test]
    fn scan_linked_skills_returns_empty_for_no_skills_dir() {
        let tmp = TempDir::new().unwrap();
        let linked = scan_linked_skills(tmp.path(), Tool::Claude);
        assert!(linked.is_empty());
    }

    #[test]
    fn unmanaged_detects_real_dirs_in_claude() {
        let home = TempDir::new().unwrap();
        let claude_skills = home.path().join(".claude").join("skills");
        fs::create_dir_all(&claude_skills).unwrap();
        fs::create_dir(claude_skills.join("my-skill")).unwrap();

        let result = scan_unmanaged_skills(home.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, Tool::Claude);
        assert_eq!(result[0].1, vec!["my-skill"]);
    }

    #[test]
    #[cfg(unix)]
    fn unmanaged_detects_non_skilltree_symlinks() {
        let home = TempDir::new().unwrap();
        let external_target = TempDir::new().unwrap();
        let claude_skills = home.path().join(".claude").join("skills");
        fs::create_dir_all(&claude_skills).unwrap();
        // Symlink pointing outside ~/.skilltree/ → unmanaged
        symlink(external_target.path(), claude_skills.join("external-skill")).unwrap();

        let result = scan_unmanaged_skills(home.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, vec!["external-skill"]);
    }

    #[test]
    #[cfg(unix)]
    fn unmanaged_ignores_skilltree_symlinks() {
        let home = TempDir::new().unwrap();
        let central = home.path().join(".skilltree");
        let central_skill = central.join("my-skill");
        fs::create_dir_all(&central_skill).unwrap();
        let claude_skills = home.path().join(".claude").join("skills");
        fs::create_dir_all(&claude_skills).unwrap();
        // Symlink pointing to ~/.skilltree/my-skill → managed
        symlink(&central_skill, claude_skills.join("my-skill")).unwrap();

        let result = scan_unmanaged_skills(home.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn unmanaged_detects_in_both_tools() {
        let home = TempDir::new().unwrap();
        let claude_skills = home.path().join(".claude").join("skills");
        let codex_skills = home.path().join(".codex").join("skills");
        fs::create_dir_all(&claude_skills).unwrap();
        fs::create_dir_all(&codex_skills).unwrap();
        fs::create_dir(claude_skills.join("skill-a")).unwrap();
        fs::create_dir(codex_skills.join("skill-b")).unwrap();

        let result = scan_unmanaged_skills(home.path()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, Tool::Claude);
        assert_eq!(result[0].1, vec!["skill-a"]);
        assert_eq!(result[1].0, Tool::Codex);
        assert_eq!(result[1].1, vec!["skill-b"]);
    }

    #[test]
    fn unmanaged_returns_empty_when_clean() {
        let home = TempDir::new().unwrap();
        let result = scan_unmanaged_skills(home.path()).unwrap();
        assert!(result.is_empty());
    }
}
