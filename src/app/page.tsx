import { readClaudeJson } from "@/lib/readers/claude-json";
import { readSkillTree } from "@/lib/readers/skill-tree";
import { pathExists } from "@/lib/safe-read";
import { SKILL_TREE_HOME } from "@/lib/claude-home";
import { SkillsView } from "./skills-view";

export const dynamic = "force-dynamic";

export default async function SkillsPage() {
  const initialized = await pathExists(SKILL_TREE_HOME);

  if (!initialized) {
    return (
      <div className="flex flex-col items-center justify-center py-24 text-center">
        <h1 className="text-3xl font-bold mb-4">Skill Tree</h1>
        <p className="text-muted-foreground max-w-md">
          Skill Tree가 초기화되지 않았습니다. `skill-tree init`을 실행하세요.
        </p>
      </div>
    );
  }

  const claudeJson = await readClaudeJson();
  const projectPaths = Object.keys(claudeJson.projects);
  const data = await readSkillTree(projectPaths);

  return (
    <SkillsView
      initialized={data.initialized}
      skills={data.skills}
      allTags={data.allTags}
      projectLinks={data.projectLinks}
      projectPaths={projectPaths}
    />
  );
}
