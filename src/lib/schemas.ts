import { z } from "zod/v4";

export const SettingsSchema = z.object({
  permissions: z
    .object({
      allow: z.array(z.string()).default([]),
      deny: z.array(z.string()).default([]),
    })
    .optional(),
  model: z.string().optional(),
  enabledPlugins: z.record(z.string(), z.boolean()).optional(),
  alwaysThinkingEnabled: z.boolean().optional(),
  statusLine: z
    .object({
      type: z.string(),
      command: z.string(),
    })
    .optional(),
});

export const PluginInstallationSchema = z.object({
  scope: z.enum(["user", "local"]),
  projectPath: z.string().optional(),
  installPath: z.string().optional(),
  version: z.string(),
  installedAt: z.string(),
  lastUpdated: z.string(),
  gitCommitSha: z.string().optional(),
});

export const InstalledPluginsSchema = z.object({
  version: z.number().optional(),
  plugins: z.record(z.string(), z.array(PluginInstallationSchema)).optional(),
});

export const BlockedPluginSchema = z.object({
  plugin: z.string(),
  added_at: z.string(),
  reason: z.string(),
  text: z.string(),
});

export const BlocklistSchema = z.object({
  fetchedAt: z.string().optional(),
  plugins: z.array(BlockedPluginSchema).optional(),
});

export const McpServerSchema = z.object({
  type: z.string().optional(),
  command: z.string().optional(),
  args: z.array(z.string()).optional(),
  url: z.string().optional(),
  env: z.record(z.string(), z.string()).optional(),
});

export const ProjectSchema = z.object({
  allowedTools: z.array(z.string()).optional(),
  mcpServers: z.record(z.string(), McpServerSchema).optional(),
  hasTrustDialogAccepted: z.boolean().optional(),
  lastCost: z.number().optional(),
  lastDuration: z.number().optional(),
  lastLinesAdded: z.number().optional(),
  lastLinesRemoved: z.number().optional(),
  lastTotalInputTokens: z.number().optional(),
  lastTotalOutputTokens: z.number().optional(),
  lastSessionId: z.string().optional(),
  exampleFiles: z.array(z.string()).optional(),
});

export const OAuthAccountSchema = z.object({
  displayName: z.string().optional(),
  emailAddress: z.string().optional(),
  billingType: z.string().optional(),
  hasExtraUsageEnabled: z.boolean().optional(),
  accountCreatedAt: z.string().optional(),
  subscriptionCreatedAt: z.string().optional(),
});

export const DailyActivitySchema = z.object({
  date: z.string(),
  messageCount: z.number(),
  sessionCount: z.number(),
  toolCallCount: z.number(),
});

export const DailyModelTokensSchema = z.object({
  date: z.string(),
  tokensByModel: z.record(z.string(), z.number()),
});

export const ModelUsageSchema = z.object({
  inputTokens: z.number(),
  outputTokens: z.number(),
  cacheReadInputTokens: z.number().optional(),
  cacheCreationInputTokens: z.number().optional(),
});

export const StatsCacheSchema = z.object({
  dailyActivity: z.array(DailyActivitySchema).optional(),
  dailyModelTokens: z.array(DailyModelTokensSchema).optional(),
  modelUsage: z.record(z.string(), ModelUsageSchema).optional(),
  totalSessions: z.number().optional(),
  totalMessages: z.number().optional(),
  hourCounts: z.record(z.string(), z.number()).optional(),
  firstSessionDate: z.string().optional(),
  lastComputedDate: z.string().optional(),
});

export const ClaudeJsonSchema = z.object({
  numStartups: z.number().optional(),
  installMethod: z.string().optional(),
  firstStartTime: z.string().optional(),
  oauthAccount: OAuthAccountSchema.optional(),
  mcpServers: z.record(z.string(), McpServerSchema).optional(),
  projects: z.record(z.string(), ProjectSchema).optional(),
});

export type Settings = z.infer<typeof SettingsSchema>;
export type PluginInstallation = z.infer<typeof PluginInstallationSchema>;
export type InstalledPlugins = z.infer<typeof InstalledPluginsSchema>;
export type BlockedPlugin = z.infer<typeof BlockedPluginSchema>;
export type Blocklist = z.infer<typeof BlocklistSchema>;
export type McpServer = z.infer<typeof McpServerSchema>;
export type Project = z.infer<typeof ProjectSchema>;
export type OAuthAccount = z.infer<typeof OAuthAccountSchema>;
export type DailyActivity = z.infer<typeof DailyActivitySchema>;
export type ModelUsage = z.infer<typeof ModelUsageSchema>;
export type StatsCache = z.infer<typeof StatsCacheSchema>;
export type ClaudeJson = z.infer<typeof ClaudeJsonSchema>;
