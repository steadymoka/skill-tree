use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::Paths;
use crate::fs_util::{Tool, LINKABLE_TOOLS};
use crate::scanner;
use crate::yaml;

/// Entry: "project_name (claude, codex)" or "project_name (claude)" etc.
struct ProjectLink {
    name: String,
    tools: BTreeSet<Tool>,
}

/// Build a map: skill_name → list of ProjectLink.
fn build_link_map(project_paths: &[String]) -> BTreeMap<String, Vec<ProjectLink>> {
    // skill → project_name → tools
    let mut raw: BTreeMap<String, BTreeMap<String, BTreeSet<Tool>>> = BTreeMap::new();

    for project in project_paths {
        let project_path = Path::new(project);
        let name = crate::fs_util::basename(project).to_string();

        for tool in &LINKABLE_TOOLS {
            for skill in scanner::scan_linked_skills(project_path, *tool) {
                raw.entry(skill)
                    .or_default()
                    .entry(name.clone())
                    .or_default()
                    .insert(*tool);
            }
        }
    }

    raw.into_iter()
        .map(|(skill, projects)| {
            let links = projects
                .into_iter()
                .map(|(name, tools)| ProjectLink { name, tools })
                .collect();
            (skill, links)
        })
        .collect()
}

mod ansi {
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const CYAN_BOLD: &str = "\x1b[36;1m";
    pub const GREEN: &str = "\x1b[32m";
    pub const RESET: &str = "\x1b[0m";
}

fn print_link(link: &ProjectLink) {
    let tools: Vec<&str> = link.tools.iter().map(|t| t.short_label()).collect();
    println!(
        "        {}→{}  {}{}{}  {}({}){}",
        ansi::DIM,
        ansi::RESET,
        ansi::GREEN,
        link.name,
        ansi::RESET,
        ansi::DIM,
        tools.join(","),
        ansi::RESET,
    );
}

pub fn print_tree(paths: &Paths, project_paths: &[String]) -> Result<()> {
    let map = yaml::read_skills_yaml(&paths.skills_yaml)
        .context("skills.yaml not found — run `skilltree init` first")?;
    let central_dirs: HashSet<String> = scanner::scan_skill_dirs(&paths.skill_tree_dir)?
        .into_iter()
        .collect();

    let link_map = build_link_map(project_paths);

    let mut by_tag: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut untagged: Vec<String> = Vec::new();

    for (skill, tags) in &map {
        if !central_dirs.contains(skill) {
            continue;
        }
        if tags.is_empty() {
            untagged.push(skill.clone());
        } else {
            for tag in tags {
                by_tag.entry(tag.clone()).or_default().push(skill.clone());
            }
        }
    }

    let total_skills = central_dirs.len();
    let total_tags = by_tag.len();
    println!(
        "{}Skill Tree  ·  {} skills  ·  {} tags{}\n",
        ansi::BOLD,
        total_skills,
        total_tags,
        ansi::RESET,
    );

    let print_skill = |skill: &str| match link_map.get(skill) {
        Some(links) if !links.is_empty() => {
            println!("    {}", skill);
            for link in links {
                print_link(link);
            }
        }
        _ => println!("    {}", skill),
    };

    let print_section = |tag: &str, skills: &[String]| {
        println!("  {}[{}]{}", ansi::CYAN_BOLD, tag, ansi::RESET);
        for skill in skills {
            print_skill(skill);
        }
        println!();
    };

    for (tag, skills) in &by_tag {
        print_section(tag, skills);
    }

    if !untagged.is_empty() {
        print_section("untagged", &untagged);
    }

    Ok(())
}
