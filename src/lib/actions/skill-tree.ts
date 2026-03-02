"use server";

import { execFile as execFileCb } from "child_process";
import { promisify } from "util";
import { revalidatePath } from "next/cache";

const execFile = promisify(execFileCb);
const BIN = "skill-tree";

export async function initializeSkillTree() {
  await execFile(BIN, ["init"]);
  revalidatePath("/");
}

export async function updateSkillTags(dirName: string, tags: string[]) {
  await execFile(BIN, ["tag", dirName, ...tags]);
  revalidatePath("/");
}

export async function linkSkillToProject(
  projectPath: string,
  skillName: string,
  tool: string = "claude",
) {
  await execFile(BIN, ["link-skill", skillName, "--path", projectPath, "--tool", tool]);
  revalidatePath("/");
}

export async function linkTagsToProject(
  projectPath: string,
  tags: string[],
  tool: string = "claude",
) {
  await execFile(BIN, ["link", ...tags, "--path", projectPath, "--tool", tool]);
  revalidatePath("/");
}

export async function unlinkSkillFromProject(
  projectPath: string,
  skillName: string,
  tool: string = "claude",
) {
  await execFile(BIN, ["unlink", skillName, "--path", projectPath, "--tool", tool]);
  revalidatePath("/");
}
