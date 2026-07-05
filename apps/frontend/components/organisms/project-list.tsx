"use client";

import { useState } from "react";
import {
  closestCenter,
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  rectSortingStrategy,
  SortableContext,
  useSortable,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

import { ConfirmDeleteDialog } from "@/components/molecules/confirm-delete-dialog";
import { EntityTile } from "@/components/molecules/entity-tile";
import { TileGrid } from "@/components/molecules/tile-grid";
import { reorderProjects } from "@/lib/api/projects";
import type { Project } from "@/lib/api/schemas";
import { useDeleteProject, useRenameProject } from "@/lib/hooks/use-projects";
import { handleDragEnd, useReorder } from "@/lib/hooks/use-reorder";
import { queryKeys } from "@/lib/query/keys";

export function ProjectList({ projects }: { projects: Project[] }) {
  const [activeId, setActiveId] = useState<string | null>(null);
  const reorder = useReorder<Project>(queryKeys.projects, (orderedIds) =>
    reorderProjects({ orderedIds }),
  );

  // Distance constraint so a tap opens the tile and only a drag reorders it.
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
  );

  const activeProject = projects.find((project) => project.id === activeId);

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={({ active }) => setActiveId(String(active.id))}
      onDragEnd={(event) => {
        handleDragEnd(event, reorder.mutate);
        setActiveId(null);
      }}
      onDragCancel={() => setActiveId(null)}
    >
      <SortableContext
        items={projects.map((project) => project.id)}
        strategy={rectSortingStrategy}
      >
        <TileGrid>
          {projects.map((project) => (
            <ProjectTile key={project.id} project={project} />
          ))}
        </TileGrid>
      </SortableContext>
      <DragOverlay>
        {activeProject ? (
          <EntityTile
            name={activeProject.name}
            seed={activeProject.id}
            preview
          />
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}

function ProjectTile({ project }: { project: Project }) {
  const rename = useRenameProject();
  const remove = useDeleteProject();
  const [confirmOpen, setConfirmOpen] = useState(false);

  const { setNodeRef, listeners, transform, transition, isDragging } =
    useSortable({ id: project.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.3 : undefined,
  };

  return (
    <>
      <div ref={setNodeRef} style={style} {...listeners} className="touch-none">
        <EntityTile
          name={project.name}
          seed={project.id}
          href={`/projects/${project.id}`}
          openLabel={`Open ${project.name}`}
          menuLabel={`${project.name} actions`}
          renameLabel="Project name"
          onRename={(name) => rename.mutate({ id: project.id, name })}
          onDelete={() => setConfirmOpen(true)}
        />
      </div>
      <ConfirmDeleteDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        title={`Delete “${project.name}”?`}
        description="This permanently deletes the project and all of its boards."
        onConfirm={() => remove.mutate(project.id)}
      />
    </>
  );
}
