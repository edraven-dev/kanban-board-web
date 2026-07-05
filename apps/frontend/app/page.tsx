"use client";

import { FolderKanbanIcon } from "lucide-react";

import { EmptyState } from "@/components/atoms/empty-state";
import { InlineCreate } from "@/components/molecules/inline-create";
import { TileGrid } from "@/components/molecules/tile-grid";
import { ProjectList } from "@/components/organisms/project-list";
import { Skeleton } from "@/components/ui/skeleton";
import { useCreateProject, useProjects } from "@/lib/hooks/use-projects";

export default function Home() {
  const { data: projects, isPending, isError } = useProjects();
  const createProject = useCreateProject();

  return (
    <main className="mx-auto flex w-full max-w-6xl flex-1 flex-col gap-4 px-4 py-6 sm:px-6">
      <div className="flex items-center justify-between gap-4">
        <h1 className="text-lg font-semibold tracking-tight">Projects</h1>
        <InlineCreate
          label="project"
          onCreate={(name) => createProject.mutate(name)}
        />
      </div>

      {isPending ? (
        <ProjectListSkeleton />
      ) : isError ? (
        <p role="alert" className="text-sm text-destructive">
          Couldn’t load projects. Please try again.
        </p>
      ) : projects.length === 0 ? (
        <EmptyState
          icon={<FolderKanbanIcon />}
          title="No projects yet"
          description="Create your first project to start building boards."
        />
      ) : (
        <ProjectList projects={projects} />
      )}
    </main>
  );
}

function ProjectListSkeleton() {
  return (
    <div data-testid="projects-skeleton">
      <TileGrid>
        {Array.from({ length: 6 }).map((_, index) => (
          <Skeleton key={index} className="aspect-[16/9] w-full rounded-xl" />
        ))}
      </TileGrid>
    </div>
  );
}
