import { STATS_PATH } from "../claude-home";
import { safeReadJson } from "../safe-read";
import { StatsCacheSchema, type StatsCache } from "../schemas";

export async function readStats(): Promise<StatsCache | null> {
  const raw = await safeReadJson(STATS_PATH, null);
  if (!raw) return null;

  const parsed = StatsCacheSchema.safeParse(raw);
  return parsed.success ? parsed.data : null;
}
