import { join } from "path";
import matter from "gray-matter";
import { SKILLS_DIR } from "../claude-home";
import { safeReadDir, safeReadText } from "../safe-read";

export interface Skill {
  dirName: string;
  name: string;
  description: string;
  content: string;
}

export async function readSkills(): Promise<Skill[]> {
  const entries = await safeReadDir(SKILLS_DIR);

  const results = await Promise.all(
    entries.map(async (dirName): Promise<Skill | null> => {
      const raw = await safeReadText(join(SKILLS_DIR, dirName, "SKILL.md"));
      if (!raw) return null;

      const { data, content } = matter(raw);
      return {
        dirName,
        name: String(data.name ?? "") || dirName,
        description: String(data.description ?? ""),
        content,
      };
    })
  );

  return results.filter((s): s is Skill => s !== null);
}
