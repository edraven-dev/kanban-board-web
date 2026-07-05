"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { createProject, listProjects } from "@/lib/api/projects";
import { queryKeys } from "@/lib/query/keys";

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
