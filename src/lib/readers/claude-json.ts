import { CLAUDE_JSON_PATH } from "../claude-home";
import { safeReadJson } from "../safe-read";
import { ClaudeJsonSchema, type McpServer, type Project } from "../schemas";
import { redactMcpServers, redactAccount, type SafeAccount } from "../redact";

export interface ClaudeJsonData {
  projects: Record<string, Project>;
  mcpServers: Record<string, McpServer>;
  account: SafeAccount | null;
  numStartups: number;
}

export async function readClaudeJson(): Promise<ClaudeJsonData> {
  const raw = await safeReadJson(CLAUDE_JSON_PATH, {});
  const parsed = ClaudeJsonSchema.safeParse(raw);

  if (!parsed.success) {
    return { projects: {}, mcpServers: {}, account: null, numStartups: 0 };
  }

  const data = parsed.data;
  return {
    projects: data.projects ?? {},
    mcpServers: redactMcpServers(data.mcpServers ?? {}),
    account: redactAccount(data.oauthAccount),
    numStartups: data.numStartups ?? 0,
  };
}
