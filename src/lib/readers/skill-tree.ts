import { join } from "path";
import { lstat, readdir } from "fs/promises";
import matter from "gray-matter";
import { SKILL_TREE_HOME, SKILLS_YAML_PATH } from "../claude-home";
import { safeReadText } from "../safe-read";
import { parseSkillsYaml, type SkillTagMap } from "../yaml-lite";

export interface Skill {
  dirName: string;
  name: string;
  description: string;
  content: string;
  tags: string[];
  charCount: number;
}

export interface ProjectSkillLink {
  projectPath: string;
  projectName: string;
  linkedSkills: string[];
}

export interface SkillTreeData {
  initialized: boolean;
  skills: Skill[];
  allTags: string[];
  tagMap: SkillTagMap;
  projectLinks: ProjectSkillLink[];
}

const TOOL_SKILLS_DIRS = [".claude/skills", ".codex/skills"] as const;

export async function readSkillTree(
  projectPaths: string[],
): Promise<SkillTreeData> {
  const yamlRaw = await safeReadText(SKILLS_YAML_PATH);
  if (yamlRaw === null) {
    return {
      initialized: false,
      skills: [],
      allTags: [],
      tagMap: {},
      projectLinks: [],
    };
  }

  const tagMap = parseSkillsYaml(yamlRaw);

  // Scan skill directories
  let entries: string[] = [];
  try {
    entries = await readdir(SKILL_TREE_HOME);
  } catch {
    // directory doesn't exist
  }

  const skills = (
    await Promise.all(
      entries.map(async (dirName): Promise<Skill | null> => {
        if (dirName.startsWith(".")) return null;
        const raw = await safeReadText(
          join(SKILL_TREE_HOME, dirName, "SKILL.md"),
        );
        if (!raw) return null;

        const { data, content } = matter(raw);
        return {
          dirName,
          name: String(data.name ?? "") || dirName,
          description: String(data.description ?? ""),
          content,
          tags: tagMap[dirName] ?? [],
          charCount: content.length,
        };
      }),
    )
  ).filter((s): s is Skill => s !== null);

  skills.sort((a, b) => a.dirName.localeCompare(b.dirName));

  // Collect unique tags
  const tagSet = new Set<string>();
  for (const tags of Object.values(tagMap)) {
    for (const t of tags) tagSet.add(t);
  }
  const allTags = Array.from(tagSet).sort();

  // Scan project skill links across all tool directories
  const projectLinks = await Promise.all(
    projectPaths.map(async (projectPath): Promise<ProjectSkillLink> => {
      const linkedSkills: string[] = [];
      for (const subdir of TOOL_SKILLS_DIRS) {
        const skillsDir = join(projectPath, ...subdir.split("/"));
        try {
          const projectEntries = await readdir(skillsDir);
          for (const name of projectEntries) {
            if (name.startsWith(".")) continue;
            try {
              const stat = await lstat(join(skillsDir, name));
              if (stat.isSymbolicLink()) {
                linkedSkills.push(name);
              }
            } catch {
              // skip unreadable entries
            }
          }
        } catch {
          // directory doesn't exist
        }
      }
      // Deduplicate and sort
      const uniqueSkills = [...new Set(linkedSkills)].sort();
      return {
        projectPath,
        projectName: projectPath.split("/").pop() ?? projectPath,
        linkedSkills: uniqueSkills,
      };
    }),
  );

  return {
    initialized: true,
    skills,
    allTags,
    tagMap,
    projectLinks,
  };
}
