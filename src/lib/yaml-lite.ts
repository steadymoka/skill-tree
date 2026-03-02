/**
 * Read-only parser for skills.yaml.
 * Handles both formats written by serde_yaml:
 *
 *   Inline:  skill-name: [tag1, tag2]
 *   List:    skill-name:\n- tag1\n- tag2
 */

export type SkillTagMap = Record<string, string[]>;

const INLINE_RE = /^([^#\s][^:]+):\s*\[([^\]]*)\]\s*$/;
const KEY_RE = /^([^#\s][^:]+):\s*$/;
const ITEM_RE = /^-\s+(.+)$/;

export function parseSkillsYaml(raw: string): SkillTagMap {
  const map: SkillTagMap = {};
  const lines = raw.split("\n");

  let i = 0;
  while (i < lines.length) {
    const trimmed = lines[i].trim();
    if (!trimmed || trimmed.startsWith("#")) {
      i++;
      continue;
    }

    const inline = trimmed.match(INLINE_RE);
    if (inline) {
      const skill = inline[1].trim();
      const tagsRaw = inline[2].trim();
      map[skill] = tagsRaw
        ? tagsRaw.split(",").map((t) => t.trim()).filter(Boolean)
        : [];
      i++;
      continue;
    }

    const key = trimmed.match(KEY_RE);
    if (key) {
      const skill = key[1].trim();
      const tags: string[] = [];
      i++;
      while (i < lines.length) {
        const item = lines[i].trim().match(ITEM_RE);
        if (!item) break;
        tags.push(item[1].trim());
        i++;
      }
      map[skill] = tags;
      continue;
    }

    i++;
  }

  return map;
}
