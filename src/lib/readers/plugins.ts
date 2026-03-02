import { PLUGINS_PATH, BLOCKLIST_PATH } from "../claude-home";
import { safeReadJson } from "../safe-read";
import {
  InstalledPluginsSchema,
  BlocklistSchema,
  type PluginInstallation,
  type BlockedPlugin,
} from "../schemas";

export interface PluginInfo {
  id: string;
  name: string;
  marketplace: string;
  installations: PluginInstallation[];
  isEnabled: boolean | undefined;
}

export interface PluginsData {
  plugins: PluginInfo[];
  blocklist: BlockedPlugin[];
}

interface RawPluginsData {
  plugins: Omit<PluginInfo, "isEnabled">[];
  blocklist: BlockedPlugin[];
}

export async function readPluginsRaw(): Promise<RawPluginsData> {
  const [rawInstalled, rawBlocklist] = await Promise.all([
    safeReadJson(PLUGINS_PATH, {}),
    safeReadJson(BLOCKLIST_PATH, {}),
  ]);

  const installed = InstalledPluginsSchema.safeParse(rawInstalled);
  const blocklist = BlocklistSchema.safeParse(rawBlocklist);

  const plugins: Omit<PluginInfo, "isEnabled">[] = [];

  if (installed.success && installed.data.plugins) {
    for (const [id, installations] of Object.entries(installed.data.plugins)) {
      const atIndex = id.lastIndexOf("@");
      const name = atIndex > 0 ? id.slice(0, atIndex) : id;
      const marketplace = atIndex > 0 ? id.slice(atIndex + 1) : "unknown";

      plugins.push({ id, name, marketplace, installations });
    }
  }

  return {
    plugins,
    blocklist: blocklist.success ? (blocklist.data.plugins ?? []) : [],
  };
}

export function mergeEnabledPlugins(
  raw: RawPluginsData,
  enabledPlugins: Record<string, boolean> | undefined
): PluginsData {
  return {
    plugins: raw.plugins.map((p) => ({
      ...p,
      isEnabled: enabledPlugins?.[p.id],
    })),
    blocklist: raw.blocklist,
  };
}
