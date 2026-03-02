use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use skill_tree::config::Paths;
use skill_tree::fs_util::Tool;
use skill_tree::{init, linker, serve, tagger, tree, tui};

#[derive(Parser)]
#[command(name = "skill-tree", about = "Skill management tool for Claude Code & Codex")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize skill-tree (idempotent)
    Init,

    /// Link skills to the current project by tags
    Link {
        /// Tags to match (union)
        #[arg(required = true)]
        tags: Vec<String>,

        /// Project path (defaults to CWD)
        #[arg(long, short)]
        path: Option<PathBuf>,

        /// Target tool (claude or codex)
        #[arg(long, short, default_value = "claude")]
        tool: String,
    },

    /// Link a single skill by name
    LinkSkill {
        /// Skill directory name
        skill: String,

        /// Project path (defaults to CWD)
        #[arg(long, short)]
        path: Option<PathBuf>,

        /// Target tool (claude or codex)
        #[arg(long, short, default_value = "claude")]
        tool: String,
    },

    /// Unlink a skill from the current project
    Unlink {
        /// Skill directory name to unlink
        skill: Option<String>,

        /// Unlink all skills
        #[arg(long)]
        all: bool,

        /// Project path (defaults to CWD)
        #[arg(long, short)]
        path: Option<PathBuf>,

        /// Target tool (claude or codex)
        #[arg(long, short, default_value = "claude")]
        tool: String,
    },

    /// Set tags for a skill (used internally by web UI)
    #[command(hide = true)]
    Tag {
        skill: String,
        tags: Vec<String>,
    },

    /// Print skill tree grouped by tags
    Tree,

    /// Start the web UI (Next.js standalone server)
    Serve {
        /// Project root containing .next/standalone/
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let Some(command) = cli.command else {
        let paths = Paths::default_paths()?;
        let project_paths = load_project_paths();
        return tui::run(paths, project_paths);
    };

    match command {
        Command::Serve { root } => serve::start_web(root),

        _ => {
            let paths = Paths::default_paths()?;
            dispatch(command, &paths)
        }
    }
}

fn dispatch(command: Command, paths: &Paths) -> Result<()> {
    match command {
        Command::Init => init::initialize(paths),

        Command::Link { tags, path, tool } => {
            let project = resolve_project(path)?;
            let tool: Tool = tool.parse()?;
            linker::link_by_tags(paths, &project, &tags, tool)
        }

        Command::LinkSkill { skill, path, tool } => {
            let project = resolve_project(path)?;
            let tool: Tool = tool.parse()?;
            linker::link_skill(paths, &project, &skill, tool)
        }

        Command::Unlink { skill, all, path, tool } => {
            let project = resolve_project(path)?;
            let tool: Tool = tool.parse()?;
            if all {
                linker::unlink_all(&project, tool)
            } else if let Some(name) = skill {
                linker::unlink_skill(&project, &name, tool)
            } else {
                anyhow::bail!("specify a skill name or --all");
            }
        }

        Command::Tag { skill, tags } => tagger::set_tags(paths, &skill, &tags),

        Command::Tree => {
            let project_paths = load_project_paths();
            tree::print_tree(paths, &project_paths)
        }

        Command::Serve { .. } => unreachable!(),
    }
}

fn resolve_project(path: Option<PathBuf>) -> Result<PathBuf> {
    match path {
        Some(p) => Ok(p),
        None => env::current_dir().context("cannot determine current directory"),
    }
}

/// Load project paths from ~/.claude.json for TUI project list.
fn load_project_paths() -> Vec<String> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };
    let claude_json = home.join(".claude.json");
    let content = match std::fs::read_to_string(&claude_json) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let Some(projects) = value.get("projects").and_then(|p| p.as_object()) else {
        return Vec::new();
    };
    let mut paths: Vec<String> = projects
        .keys()
        .map(|k| k.to_string())
        .collect();
    paths.sort();
    paths
}
