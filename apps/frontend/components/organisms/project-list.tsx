"use client";

import { reorderProjects } from "@/lib/api/projects";
import type { Project } from "@/lib/api/schemas";
import { useDeleteProject, useRenameProject } from "@/lib/hooks/use-projects";
import { queryKeys } from "@/lib/query/keys";

import { SortableTileGrid } from "./sortable-tile-grid";

export function ProjectList({ projects }: { projects: Project[] }) {
  const rename = useRenameProject();
  const remove = useDeleteProject();

  return (
    <SortableTileGrid
      items={projects}
      queryKey={queryKeys.projects}
      reorderPersist={(orderedIds) => reorderProjects({ orderedIds })}
      hrefFor={(project) => `/projects/${project.id}`}
      openLabelFor={(project) => `Open ${project.name}`}
      menuLabelFor={(project) => `${project.name} actions`}
      renameLabel="Project name"
      deleteTitleFor={(project) => `Delete “${project.name}”?`}
      deleteDescription="This permanently deletes the project and all of its boards."
      onRename={(id, name) => rename.mutate({ id, name })}
      onDelete={(id) => remove.mutate(id)}
    />
  );
}
