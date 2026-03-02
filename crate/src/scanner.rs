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
}
