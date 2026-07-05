"use client";

import Link from "next/link";
import { FolderKanbanIcon } from "lucide-react";

import { EmptyState } from "@/components/atoms/empty-state";
import { InlineCreate } from "@/components/molecules/inline-create";
import { Skeleton } from "@/components/ui/skeleton";
import { useCreateProject, useProjects } from "@/lib/hooks/use-projects";

export default function Home() {
  const { data: projects, isPending, isError } = useProjects();
  const createProject = useCreateProject();

  return (
    <main className="mx-auto flex w-full max-w-2xl flex-1 flex-col gap-6 px-6 py-16">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold tracking-tight">Projects</h1>
        <p className="text-sm text-muted-foreground">
          Choose a project to open its boards, or create a new one.
        </p>
      </header>

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
        <ul className="flex flex-col gap-2">
          {projects.map((project) => (
            <li key={project.id}>
              <Link
                href={`/projects/${project.id}`}
                className="block rounded-lg border border-border bg-card px-4 py-3 text-sm font-medium transition-colors hover:bg-muted"
              >
                {project.name}
              </Link>
            </li>
          ))}
        </ul>
      )}

      <InlineCreate
        label="project"
        onCreate={(name) => createProject.mutate(name)}
      />
    </main>
  );
}

function ProjectListSkeleton() {
  return (
    <div className="flex flex-col gap-2" data-testid="projects-skeleton">
      {Array.from({ length: 3 }).map((_, index) => (
        <Skeleton key={index} className="h-12 w-full rounded-lg" />
      ))}
    </div>
  );
}
