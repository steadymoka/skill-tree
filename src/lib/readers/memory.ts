import { join } from "path";
import { PROJECTS_DIR } from "../claude-home";
import { safeReadDir, safeReadText } from "../safe-read";

export interface ProjectMemory {
  encodedPath: string;
  memoryContent: string;
  conventionsContent: string | null;
}

export async function readProjectMemories(): Promise<ProjectMemory[]> {
  const entries = await safeReadDir(PROJECTS_DIR);

  const results = await Promise.all(
    entries.map(async (encodedPath): Promise<ProjectMemory | null> => {
      const memoryDir = join(PROJECTS_DIR, encodedPath, "memory");
      const [memoryContent, conventionsContent] = await Promise.all([
        safeReadText(join(memoryDir, "MEMORY.md")),
        safeReadText(join(memoryDir, "conventions.md")),
      ]);

      if (!memoryContent) return null;

      return {
        encodedPath,
        memoryContent,
        conventionsContent,
      };
    })
  );

  return results.filter((m): m is ProjectMemory => m !== null);
}

export function decodeProjectName(encodedPath: string): string {
  const parts = encodedPath.split("-").filter(Boolean);
  const lastPart = parts[parts.length - 1];
  return lastPart ?? encodedPath;
}
