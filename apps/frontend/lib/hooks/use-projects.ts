"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import {
  createProject,
  deleteProject,
  listProjects,
  updateProject,
} from "@/lib/api/projects";
import type { Project } from "@/lib/api/schemas";
import { queryKeys } from "@/lib/query/keys";
import { useOptimisticListMutation } from "./use-optimistic-list-mutation";

export function useProjects() {
  return useQuery({
    queryKey: queryKeys.projects,
    queryFn: listProjects,
  });
}

export function useCreateProject() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => createProject({ name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.projects });
    },
  });
}

export function useRenameProject() {
  return useOptimisticListMutation<{ id: string; name: string }, Project>({
    queryKey: queryKeys.projects,
    mutationFn: ({ id, name }) => updateProject(id, { name }),
    optimisticUpdate: (projects, { id, name }) =>
      projects.map((project) =>
        project.id === id ? { ...project, name } : project,
      ),
    errorMessage: "Couldn’t rename the project.",
  });
}

export function useDeleteProject() {
  return useOptimisticListMutation<string, Project>({
    queryKey: queryKeys.projects,
    mutationFn: (id) => deleteProject(id),
    optimisticUpdate: (projects, id) =>
      projects.filter((project) => project.id !== id),
    errorMessage: "Couldn’t delete the project.",
  });
}
