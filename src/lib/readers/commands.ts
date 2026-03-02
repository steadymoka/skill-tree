import { join } from "path";
import matter from "gray-matter";
import { COMMANDS_DIR } from "../claude-home";
import { safeReadDir, safeReadText } from "../safe-read";

export interface Command {
  fileName: string;
  name: string;
  description: string;
  allowedTools: string[];
  argumentHint?: string;
  content: string;
}

export async function readCommands(): Promise<Command[]> {
  const entries = await safeReadDir(COMMANDS_DIR);
  const mdFiles = entries.filter((f) => f.endsWith(".md"));

  const results = await Promise.all(
    mdFiles.map(async (fileName): Promise<Command | null> => {
      const raw = await safeReadText(join(COMMANDS_DIR, fileName));
      if (!raw) return null;

      const { data, content } = matter(raw);
      const allowedToolsRaw = data["allowed-tools"] as string | string[] | undefined;
      const allowedTools =
        typeof allowedToolsRaw === "string"
          ? allowedToolsRaw.split(",").map((t) => t.trim())
          : Array.isArray(allowedToolsRaw)
            ? allowedToolsRaw
            : [];

      const baseName = fileName.replace(".md", "");
      return {
        fileName: baseName,
        name: String(data.name ?? "") || baseName,
        description: String(data.description ?? ""),
        allowedTools,
        argumentHint: data["argument-hint"] as string | undefined,
        content,
      };
    })
  );

  return results.filter((c): c is Command => c !== null);
}
