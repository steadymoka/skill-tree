import { GLOBAL_CLAUDE_MD_PATH } from "../claude-home";
import { safeReadText, pathExists } from "../safe-read";
import { join } from "path";

export async function readGlobalClaudeMd(): Promise<string | null> {
  return safeReadText(GLOBAL_CLAUDE_MD_PATH);
}

export interface ProjectClaudeMd {
  projectPath: string;
  projectName: string;
  hasClaudeMd: boolean;
  hasLocalClaudeMd: boolean;
  hasRulesDir: boolean;
}

export async function scanProjectClaudeMds(
  projectPaths: string[]
): Promise<ProjectClaudeMd[]> {
  const results = await Promise.all(
    projectPaths.map(async (projectPath): Promise<ProjectClaudeMd | null> => {
      const projectName = projectPath.split("/").pop() ?? projectPath;

      const [hasClaudeMd, hasLocalClaudeMd, hasRulesDir] = await Promise.all([
        pathExists(join(projectPath, "CLAUDE.md")),
        pathExists(join(projectPath, "CLAUDE.local.md")),
        pathExists(join(projectPath, ".claude", "rules")),
      ]);

      if (!hasClaudeMd && !hasLocalClaudeMd && !hasRulesDir) return null;

      return {
        projectPath,
        projectName,
        hasClaudeMd,
        hasLocalClaudeMd,
        hasRulesDir,
      };
    })
  );

  return results.filter((r): r is ProjectClaudeMd => r !== null);
}
