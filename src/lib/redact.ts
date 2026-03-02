import type { McpServer, OAuthAccount } from "./schemas";

export function redactMcpServers(
  servers: Record<string, McpServer>
): Record<string, McpServer> {
  return Object.fromEntries(
    Object.entries(servers).map(([name, config]) => [
      name,
      {
        ...config,
        env: config.env
          ? Object.fromEntries(
              Object.keys(config.env).map((k) => [k, "***"])
            )
          : undefined,
      },
    ])
  );
}

export interface SafeAccount {
  displayName?: string;
  emailAddress?: string;
  billingType?: string;
}

export function redactAccount(
  account: OAuthAccount | undefined
): SafeAccount | null {
  if (!account) return null;
  return {
    displayName: account.displayName,
    emailAddress: account.emailAddress,
    billingType: account.billingType,
  };
}
