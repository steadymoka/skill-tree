import { homedir } from "os";
import { join } from "path";

export const CLAUDE_HOME = join(homedir(), ".claude");
export const CLAUDE_JSON_PATH = join(homedir(), ".claude.json");

export const SETTINGS_PATH = join(CLAUDE_HOME, "settings.json");
export const PLUGINS_PATH = join(CLAUDE_HOME, "plugins", "installed_plugins.json");
export const BLOCKLIST_PATH = join(CLAUDE_HOME, "plugins", "blocklist.json");
export const SKILLS_DIR = join(CLAUDE_HOME, "skills");
export const SKILL_TREE_HOME = join(homedir(), ".skill-tree");
export const SKILLS_YAML_PATH = join(SKILL_TREE_HOME, "skills.yaml");
export const COMMANDS_DIR = join(CLAUDE_HOME, "commands");
export const STATS_PATH = join(CLAUDE_HOME, "stats-cache.json");
export const GLOBAL_CLAUDE_MD_PATH = join(CLAUDE_HOME, "CLAUDE.md");
export const PROJECTS_DIR = join(CLAUDE_HOME, "projects");
