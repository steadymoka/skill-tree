import { readFile, readdir, access } from "fs/promises";

export async function safeReadJson<T>(filePath: string, fallback: T): Promise<T> {
  try {
    const content = await readFile(filePath, "utf-8");
    if (!content.trim()) return fallback;
    return JSON.parse(content) as T;
  } catch (err) {
    if (err instanceof SyntaxError) return fallback;
    if (isFileError(err, "ENOENT")) return fallback;
    if (isFileError(err, "EACCES")) return fallback;
    throw err;
  }
}

export async function safeReadText(filePath: string): Promise<string | null> {
  try {
    return await readFile(filePath, "utf-8");
  } catch {
    return null;
  }
}

export async function safeReadDir(dirPath: string): Promise<string[]> {
  try {
    return await readdir(dirPath);
  } catch {
    return [];
  }
}

export async function pathExists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

function isFileError(err: unknown, code: string): boolean {
  return (
    typeof err === "object" &&
    err !== null &&
    "code" in err &&
    (err as { code: string }).code === code
  );
}
