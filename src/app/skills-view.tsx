"use client";

import { useState, useTransition } from "react";
import { ChevronRight } from "lucide-react";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { PillToggle } from "@/components/pill-toggle";
import { Section } from "@/components/section";
import { EmptyState } from "@/components/empty-state";
import { MarkdownViewer } from "@/components/markdown-viewer";
import {
  initializeSkillTree,
  updateSkillTags,
  linkSkillToProject,
  unlinkSkillFromProject,
  linkTagsToProject,
} from "@/lib/actions/skill-tree";
import type { Skill, ProjectSkillLink } from "@/lib/readers/skill-tree";

interface Props {
  initialized: boolean;
  skills: Skill[];
  allTags: string[];
  projectLinks: ProjectSkillLink[];
  projectPaths: string[];
}

export function SkillsView({
  initialized,
  skills,
  allTags,
  projectLinks,
  projectPaths,
}: Props) {
  const t = useT();
  const [isPending, startTransition] = useTransition();
  const [selectedTag, setSelectedTag] = useState<string | null>(null);
  const [expandedSkill, setExpandedSkill] = useState<string | null>(null);
  const [editingSkill, setEditingSkill] = useState<string | null>(null);
  const [editTagsValue, setEditTagsValue] = useState("");
  const [selectedProject, setSelectedProject] = useState<string | null>(
    projectPaths[0] ?? null,
  );
  const [selectedTool, setSelectedTool] = useState<"claude" | "codex">("claude");

  if (!initialized) {
    return (
      <div className="max-w-3xl mx-auto space-y-6">
        <EmptyState message={t.skills.initializeDesc} />
        <div className="flex justify-center">
          <Button
            disabled={isPending}
            onClick={() => startTransition(() => initializeSkillTree())}
          >
            {t.skills.initialize}
          </Button>
        </div>
      </div>
    );
  }

  const filteredSkills = selectedTag
    ? skills.filter((s) => s.tags.includes(selectedTag))
    : skills;

  const activeProject = projectLinks.find(
    (p) => p.projectPath === selectedProject,
  );
  const linkedSet = new Set(activeProject?.linkedSkills ?? []);

  const startEditTags = (skill: Skill) => {
    setEditingSkill(skill.dirName);
    setEditTagsValue(skill.tags.join(", "));
  };

  const saveTags = (dirName: string) => {
    const tags = editTagsValue
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);
    setEditingSkill(null);
    startTransition(() => updateSkillTags(dirName, tags));
  };

  return (
    <div className="max-w-3xl mx-auto space-y-6">
      {/* Summary */}
      <div className="flex items-center gap-3 text-sm text-muted-foreground">
        <span>
          {skills.length} {t.skills.skills}
        </span>
        <span>·</span>
        <span>
          {allTags.length} {t.skills.tags}
        </span>
      </div>

      {/* Tag filter */}
      <div className="flex flex-wrap gap-2">
        <PillToggle
          isActive={selectedTag === null}
          onClick={() => setSelectedTag(null)}
        >
          {t.skills.allTags}
        </PillToggle>
        {allTags.map((tag) => (
          <PillToggle
            key={tag}
            isActive={selectedTag === tag}
            onClick={() => setSelectedTag(tag)}
          >
            {tag}
          </PillToggle>
        ))}
      </div>

      {/* Skills list */}
      <Section title={t.skills.title} count={filteredSkills.length}>
        {filteredSkills.length === 0 ? (
          <EmptyState message={t.skills.noSkills} />
        ) : (
          <div className="space-y-2">
            {filteredSkills.map((skill) => {
              const isExpanded = expandedSkill === skill.dirName;
              const isEditing = editingSkill === skill.dirName;
              const isLinked = linkedSet.has(skill.dirName);

              return (
                <Card
                  key={skill.dirName}
                  className={cn(
                    "transition-colors",
                    isLinked && "border-l-4 border-l-green-500",
                  )}
                >
                  <CardContent className="py-4 px-5 space-y-2">
                    {/* Header row */}
                    <div className="flex items-start justify-between gap-2">
                      <div className="min-w-0">
                        <p className="text-sm font-semibold truncate">
                          {skill.name}
                        </p>
                        {skill.description && (
                          <p className="text-xs text-muted-foreground mt-0.5 line-clamp-1">
                            {skill.description}
                          </p>
                        )}
                      </div>
                      <span className="text-xs text-muted-foreground font-mono shrink-0">
                        {skill.charCount.toLocaleString()} {t.skills.chars}
                      </span>
                    </div>

                    {/* Tags row */}
                    {isEditing ? (
                      <div className="flex items-center gap-2">
                        <input
                          type="text"
                          value={editTagsValue}
                          onChange={(e) => setEditTagsValue(e.target.value)}
                          onKeyDown={(e) => {
                            if (e.key === "Enter") saveTags(skill.dirName);
                            if (e.key === "Escape") setEditingSkill(null);
                          }}
                          className="flex-1 text-xs font-mono bg-transparent border border-border rounded px-2 py-1 outline-none focus:border-primary"
                          placeholder="comma-separated tags"
                          autoFocus
                        />
                        <Button
                          variant="ghost"
                          size="sm"
                          className="text-xs h-7"
                          onClick={() => saveTags(skill.dirName)}
                          disabled={isPending}
                        >
                          {t.skills.save}
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="text-xs h-7"
                          onClick={() => setEditingSkill(null)}
                        >
                          {t.skills.cancel}
                        </Button>
                      </div>
                    ) : (
                      <div className="flex items-center gap-2">
                        <div className="flex flex-wrap gap-1">
                          {skill.tags.map((tag) => (
                            <Badge
                              key={tag}
                              variant="secondary"
                              className="text-xs cursor-pointer"
                              onClick={() => setSelectedTag(tag)}
                            >
                              {tag}
                            </Badge>
                          ))}
                        </div>
                        <button
                          className="text-xs text-muted-foreground hover:text-foreground transition-colors ml-auto shrink-0"
                          onClick={() => startEditTags(skill)}
                        >
                          {t.skills.editTags}
                        </button>
                      </div>
                    )}

                    {/* Link/unlink for selected project */}
                    {activeProject && (
                      <div className="flex items-center gap-2 text-xs">
                        {isLinked ? (
                          <>
                            <span className="text-green-500">✓ {t.skills.linked}</span>
                            <button
                              className="text-muted-foreground hover:text-destructive transition-colors"
                              onClick={() =>
                                startTransition(() =>
                                  unlinkSkillFromProject(
                                    activeProject.projectPath,
                                    skill.dirName,
                                    selectedTool,
                                  ),
                                )
                              }
                              disabled={isPending}
                            >
                              {t.skills.unlink}
                            </button>
                          </>
                        ) : (
                          <button
                            className="text-muted-foreground hover:text-foreground transition-colors"
                            onClick={() =>
                              startTransition(() =>
                                linkSkillToProject(
                                  activeProject.projectPath,
                                  skill.dirName,
                                  selectedTool,
                                ),
                              )
                            }
                            disabled={isPending}
                          >
                            {t.skills.link}
                          </button>
                        )}
                      </div>
                    )}

                    {/* Expand/collapse */}
                    <button
                      className="text-xs text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1"
                      onClick={() =>
                        setExpandedSkill(isExpanded ? null : skill.dirName)
                      }
                    >
                      <ChevronRight
                        className={cn(
                          "h-3 w-3 transition-transform",
                          isExpanded && "rotate-90",
                        )}
                      />
                      {isExpanded ? t.skills.collapse : t.skills.viewContent}
                    </button>

                    {isExpanded && (
                      <div className="border-t border-border pt-3 max-h-[400px] overflow-auto">
                        <MarkdownViewer content={skill.content} />
                      </div>
                    )}
                  </CardContent>
                </Card>
              );
            })}
          </div>
        )}
      </Section>

      {/* Projects section */}
      <Section title={t.skills.projects} count={projectPaths.length}>
        {projectPaths.length === 0 ? (
          <EmptyState message={t.skills.noProjects} />
        ) : (
          <div className="space-y-4">
            <div className="flex flex-wrap gap-2">
              {projectLinks.map((p) => (
                <PillToggle
                  key={p.projectPath}
                  isActive={selectedProject === p.projectPath}
                  onClick={() => setSelectedProject(p.projectPath)}
                >
                  {p.projectName}
                  {p.linkedSkills.length > 0 && (
                    <Badge variant="secondary" className="ml-1.5 text-[10px]">
                      {p.linkedSkills.length}
                    </Badge>
                  )}
                </PillToggle>
              ))}
            </div>

            {activeProject && (
              <Card>
                <CardContent className="py-4 px-5 space-y-2">
                  <p className="text-xs text-muted-foreground font-mono truncate">
                    {activeProject.projectPath}
                  </p>
                  <div className="text-xs text-muted-foreground">
                    {activeProject.linkedSkills.length} {t.skills.linked}
                  </div>

                  {/* Tool selector */}
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-muted-foreground">Tool:</span>
                    <div className="flex gap-1">
                      {(["claude", "codex"] as const).map((tool) => (
                        <PillToggle
                          key={tool}
                          isActive={selectedTool === tool}
                          onClick={() => setSelectedTool(tool)}
                        >
                          {tool}
                        </PillToggle>
                      ))}
                    </div>
                  </div>

                  {/* Bulk link by selected tag */}
                  {selectedTag && (
                    <Button
                      variant="outline"
                      size="sm"
                      className="text-xs"
                      disabled={isPending}
                      onClick={() =>
                        startTransition(() =>
                          linkTagsToProject(
                            activeProject.projectPath,
                            [selectedTag],
                            selectedTool,
                          ),
                        )
                      }
                    >
                      {t.skills.bulkLink}: [{selectedTag}]
                    </Button>
                  )}
                </CardContent>
              </Card>
            )}
          </div>
        )}
      </Section>
    </div>
  );
}
