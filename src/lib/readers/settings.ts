import { SETTINGS_PATH } from "../claude-home";
import { safeReadJson } from "../safe-read";
import { SettingsSchema, type Settings } from "../schemas";

export async function readSettings(): Promise<Settings | null> {
  const raw = await safeReadJson(SETTINGS_PATH, null);
  if (!raw) return null;

  const parsed = SettingsSchema.safeParse(raw);
  return parsed.success ? parsed.data : null;
}
