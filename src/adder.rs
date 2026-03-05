use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::config::Paths;
use crate::git::GitClient;
use crate::lock::{self, SkillLockEntry};
use crate::yaml;

pub struct AddOpts {
    pub source: String,
    pub skill: Option<String>,
    pub name: Option<String>,
    pub tags: Vec<String>,
    pub force: bool,
    pub git_ref: String,
}

#[derive(Debug)]
pub struct AddResult {
    pub skill_name: String,
}

pub fn add_skill(paths: &Paths, opts: &AddOpts, git: &dyn GitClient) -> Result<AddResult> {
    let (owner, repo) = parse_github_source(&opts.source)?;
    let skill_name = resolve_name(opts, &repo);
    validate_name(&skill_name)?;

    let dest = paths.skill_tree_dir.join(&skill_name);
    if dest.exists() {
        if !opts.force {
            bail!(
                "skill '{}' already exists. Use --force to overwrite.",
                skill_name
            );
        }
        fs::remove_dir_all(&dest)
            .with_context(|| format!("failed to remove existing skill '{}'", skill_name))?;
    }

    let tmp = make_temp_dir()?;
    let clone_result = (|| -> Result<(PathBuf, String)> {
        let url = github_url(&owner, &repo);
        git.shallow_clone(&url, &opts.git_ref, &tmp)?;

        let sha = git
            .ls_remote(&url, &opts.git_ref)
            .unwrap_or_else(|_| "unknown".to_string());

        let skill_path = if let Some(ref skill_filter) = opts.skill {
            git.sparse_checkout(&tmp, skill_filter)?;
            skill_filter.clone()
        } else {
            ".".to_string()
        };

        let source_dir = discover_skill_dir(&tmp, opts.skill.as_deref())?;
        copy_dir_recursive(&source_dir, &dest)?;

        Ok((PathBuf::from(skill_path), sha))
    })();

    // Clean up temp regardless of success/failure
    let _ = fs::remove_dir_all(&tmp);

    let (skill_path, sha) = clone_result?;

    // Register in skills.yaml
    let mut map = yaml::read_skills_yaml_or_empty(&paths.skills_yaml)?;
    map.insert(skill_name.clone(), opts.tags.clone());
    yaml::write_skills_yaml(&paths.skills_yaml, &map)?;

    // Record in lock file
    let mut lock = lock::read_lock(&paths.skill_lock_json)?;
    lock.insert(
        skill_name.clone(),
        SkillLockEntry::new(
            &opts.source,
            &skill_path.to_string_lossy(),
            &opts.git_ref,
            &sha,
        ),
    );
    lock::write_lock(&paths.skill_lock_json, &lock)?;

    Ok(AddResult { skill_name })
}

pub fn parse_github_source(source: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = source.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        bail!(
            "invalid source '{}'. Expected format: owner/repo (e.g. vercel-labs/agent-skills)",
            source
        );
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn resolve_name(opts: &AddOpts, repo: &str) -> String {
    if let Some(ref name) = opts.name {
        return name.clone();
    }
    if let Some(ref skill) = opts.skill {
        // Take the last path component as name
        return crate::fs_util::basename(skill).to_string();
    }
    repo.to_string()
}

pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("skill name cannot be empty");
    }
    if name.starts_with('.') {
        bail!("skill name cannot start with '.'");
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        bail!(
            "invalid skill name '{}'. Path traversal characters are not allowed.",
            name
        );
    }
    let valid = name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
    if !valid {
        bail!(
            "invalid skill name '{}'. Only alphanumeric, hyphens, and underscores are allowed.",
            name
        );
    }
    Ok(())
}

fn discover_skill_dir(clone_dir: &Path, skill_filter: Option<&str>) -> Result<PathBuf> {
    if let Some(skill) = skill_filter {
        let target = clone_dir.join(skill);
        if !target.exists() {
            // List available skill directories to help the user
            let available = list_skill_dirs(clone_dir);
            if available.is_empty() {
                bail!(
                    "skill '{}' not found in repository and no other skills found.",
                    skill
                );
            }
            bail!(
                "skill '{}' not found. Available skills: [{}]",
                skill,
                available.join(", ")
            );
        }
        return Ok(target);
    }

    // No filter — check root first
    if clone_dir.join("SKILL.md").exists() {
        return Ok(clone_dir.to_path_buf());
    }

    // Look for subdirectories with SKILL.md
    let skill_dirs = list_skill_dirs(clone_dir);
    match skill_dirs.len() {
        0 => bail!("no SKILL.md found in repository. Is this an agent skills repo?"),
        1 => Ok(clone_dir.join(&skill_dirs[0])),
        _ => bail!(
            "repository contains multiple skills: [{}]. Use --skill <name> to pick one.",
            skill_dirs.join(", ")
        ),
    }
}

fn list_skill_dirs(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return names,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        if path.join("SKILL.md").exists() {
            names.push(name.into_owned());
        }
    }
    names.sort();
    names
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    crate::fs_util::copy_dir_recursive(src, dst)
}

pub fn github_url(owner: &str, repo: &str) -> String {
    format!("https://github.com/{}/{}.git", owner, repo)
}

fn make_temp_dir() -> Result<PathBuf> {
    let id = format!(
        "skilltree-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(id);
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::mock::MockGitClient;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Paths) {
        let home = TempDir::new().unwrap();
        let paths = Paths::from_home(home.path());
        fs::create_dir_all(&paths.skill_tree_dir).unwrap();
        (home, paths)
    }

    fn default_opts(source: &str) -> AddOpts {
        AddOpts {
            source: source.into(),
            skill: None,
            name: None,
            tags: vec![],
            force: false,
            git_ref: "main".into(),
        }
    }

    // --- parse_github_source ---

    #[test]
    fn parse_valid_source() {
        let (owner, repo) = parse_github_source("vercel-labs/agent-skills").unwrap();
        assert_eq!(owner, "vercel-labs");
        assert_eq!(repo, "agent-skills");
    }

    #[test]
    fn parse_invalid_no_slash() {
        assert!(parse_github_source("noslash").is_err());
    }

    #[test]
    fn parse_invalid_too_many_slashes() {
        assert!(parse_github_source("a/b/c").is_err());
    }

    #[test]
    fn parse_invalid_empty_parts() {
        assert!(parse_github_source("/repo").is_err());
        assert!(parse_github_source("owner/").is_err());
    }

    // --- validate_name ---

    #[test]
    fn validate_valid_names() {
        assert!(validate_name("my-skill").is_ok());
        assert!(validate_name("skill_v2").is_ok());
        assert!(validate_name("foo123").is_ok());
        assert!(validate_name("ABC").is_ok());
    }

    #[test]
    fn validate_invalid_names() {
        assert!(validate_name("").is_err());
        assert!(validate_name(".hidden").is_err());
        assert!(validate_name("../escape").is_err());
        assert!(validate_name("has space").is_err());
        assert!(validate_name("path/sep").is_err());
        assert!(validate_name("back\\slash").is_err());
    }

    // --- resolve_name ---

    #[test]
    fn resolve_name_from_name_flag() {
        let opts = AddOpts {
            skill: Some("skills/auth".into()),
            name: Some("custom-name".into()),
            ..default_opts("u/r")
        };
        assert_eq!(resolve_name(&opts, "r"), "custom-name");
    }

    #[test]
    fn resolve_name_from_skill_flag() {
        let opts = AddOpts {
            skill: Some("skills/auth-middleware".into()),
            ..default_opts("u/r")
        };
        assert_eq!(resolve_name(&opts, "r"), "auth-middleware");
    }

    #[test]
    fn resolve_name_from_repo() {
        let opts = default_opts("u/my-repo");
        assert_eq!(resolve_name(&opts, "my-repo"), "my-repo");
    }

    // --- github_url ---

    #[test]
    fn github_url_format() {
        assert_eq!(
            github_url("owner", "repo"),
            "https://github.com/owner/repo.git"
        );
    }

    // --- copy_dir_recursive ---

    #[test]
    fn copy_excludes_git() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let dst_path = dst.path().join("output");

        fs::create_dir_all(src.path().join(".git/objects")).unwrap();
        fs::write(src.path().join(".git/HEAD"), "ref").unwrap();
        fs::write(src.path().join("SKILL.md"), "content").unwrap();
        fs::create_dir_all(src.path().join("sub")).unwrap();
        fs::write(src.path().join("sub/file.txt"), "nested").unwrap();

        copy_dir_recursive(src.path(), &dst_path).unwrap();

        assert!(dst_path.join("SKILL.md").exists());
        assert!(dst_path.join("sub/file.txt").exists());
        assert!(!dst_path.join(".git").exists());
    }

    // --- discover_skill_dir ---

    #[test]
    fn discover_root_skill() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("SKILL.md"), "---\nname: x\n---").unwrap();

        let result = discover_skill_dir(tmp.path(), None).unwrap();
        assert_eq!(result, tmp.path());
    }

    #[test]
    fn discover_single_subdir() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("my-skill");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("SKILL.md"), "---\nname: my-skill\n---").unwrap();

        let result = discover_skill_dir(tmp.path(), None).unwrap();
        assert_eq!(result, sub);
    }

    #[test]
    fn discover_multiple_without_filter_errors() {
        let tmp = TempDir::new().unwrap();
        for name in ["skill-a", "skill-b"] {
            let sub = tmp.path().join(name);
            fs::create_dir_all(&sub).unwrap();
            fs::write(sub.join("SKILL.md"), "---").unwrap();
        }

        let err = discover_skill_dir(tmp.path(), None).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("multiple skills"));
        assert!(msg.contains("skill-a"));
        assert!(msg.contains("skill-b"));
    }

    #[test]
    fn discover_with_filter() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("target-skill");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("SKILL.md"), "---").unwrap();

        let result = discover_skill_dir(tmp.path(), Some("target-skill")).unwrap();
        assert_eq!(result, sub);
    }

    #[test]
    fn discover_filter_not_found() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("other-skill");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("SKILL.md"), "---").unwrap();

        let err = discover_skill_dir(tmp.path(), Some("nonexistent")).unwrap_err();
        assert!(err.to_string().contains("not found"));
        assert!(err.to_string().contains("other-skill"));
    }

    #[test]
    fn discover_no_skills_at_all() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("empty-dir")).unwrap();

        let err = discover_skill_dir(tmp.path(), None).unwrap_err();
        assert!(err.to_string().contains("no SKILL.md"));
    }

    // --- add_skill integration ---

    #[test]
    fn add_creates_dir_and_registers() {
        let (_home, paths) = setup();
        let git = MockGitClient::new("abc123", vec![]);
        let opts = AddOpts {
            tags: vec!["api".into()],
            ..default_opts("user/my-repo")
        };

        let result = add_skill(&paths, &opts, &git).unwrap();
        assert_eq!(result.skill_name, "my-repo");

        assert!(paths.skill_tree_dir.join("my-repo/SKILL.md").exists());
        assert!(!paths.skill_tree_dir.join("my-repo/.git").exists());

        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(map["my-repo"], vec!["api".to_string()]);
    }

    #[test]
    fn add_writes_lock_entry() {
        let (_home, paths) = setup();
        let git = MockGitClient::new("sha456", vec![]);
        let opts = default_opts("owner/repo");

        add_skill(&paths, &opts, &git).unwrap();

        let lock = lock::read_lock(&paths.skill_lock_json).unwrap();
        let entry = lock.get("repo").unwrap();
        assert_eq!(entry.source, "owner/repo");
        assert_eq!(entry.installed_sha, "sha456");
        assert_eq!(entry.git_ref, "main");
    }

    #[test]
    fn add_rejects_existing_without_force() {
        let (_home, paths) = setup();
        fs::create_dir_all(paths.skill_tree_dir.join("existing")).unwrap();

        let git = MockGitClient::new("abc", vec![]);
        let opts = default_opts("u/existing");

        let err = add_skill(&paths, &opts, &git).unwrap_err();
        assert!(err.to_string().contains("already exists"));
        assert!(err.to_string().contains("--force"));
    }

    #[test]
    fn add_overwrites_with_force() {
        let (_home, paths) = setup();
        let existing = paths.skill_tree_dir.join("my-repo");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("old.txt"), "old content").unwrap();

        let git = MockGitClient::new("new-sha", vec![]);
        let opts = AddOpts {
            force: true,
            ..default_opts("u/my-repo")
        };

        let result = add_skill(&paths, &opts, &git).unwrap();
        assert_eq!(result.skill_name, "my-repo");
        assert!(!existing.join("old.txt").exists());
        assert!(existing.join("SKILL.md").exists());
    }

    #[test]
    fn add_with_skill_flag() {
        let (_home, paths) = setup();
        let git = MockGitClient::new("abc", vec!["skills/auth", "skills/db"]);
        let opts = AddOpts {
            skill: Some("skills/auth".into()),
            ..default_opts("u/r")
        };

        let result = add_skill(&paths, &opts, &git).unwrap();
        assert_eq!(result.skill_name, "auth");
        assert!(paths.skill_tree_dir.join("auth/SKILL.md").exists());
    }

    #[test]
    fn add_with_name_override() {
        let (_home, paths) = setup();
        let git = MockGitClient::new("abc", vec![]);
        let opts = AddOpts {
            name: Some("my-custom-name".into()),
            ..default_opts("u/r")
        };

        let result = add_skill(&paths, &opts, &git).unwrap();
        assert_eq!(result.skill_name, "my-custom-name");
        assert!(paths.skill_tree_dir.join("my-custom-name").exists());
    }

    #[test]
    fn add_with_tags() {
        let (_home, paths) = setup();
        let git = MockGitClient::new("abc", vec![]);
        let opts = AddOpts {
            tags: vec!["api".into(), "backend".into()],
            ..default_opts("u/r")
        };

        add_skill(&paths, &opts, &git).unwrap();

        let map = yaml::read_skills_yaml(&paths.skills_yaml).unwrap();
        assert_eq!(map["r"], vec!["api".to_string(), "backend".to_string()]);
    }

    #[test]
    fn add_cleans_temp_on_clone_failure() {
        let (_home, paths) = setup();
        let git = MockGitClient::failing_clone();
        let opts = default_opts("u/r");

        let err = add_skill(&paths, &opts, &git);
        assert!(err.is_err());

        // Skill dir should not exist
        assert!(!paths.skill_tree_dir.join("r").exists());
        // Yaml should not be modified
        assert!(!paths.skills_yaml.exists());
    }
}
