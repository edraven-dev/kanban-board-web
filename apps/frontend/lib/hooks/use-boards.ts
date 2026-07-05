"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import {
  createBoard,
  deleteBoard,
  listBoards,
  updateBoard,
} from "@/lib/api/boards";
import { ApiError } from "@/lib/api/client";
import type { Board } from "@/lib/api/schemas";
import { queryKeys } from "@/lib/query/keys";

import { useOptimisticListMutation } from "./use-optimistic-list-mutation";

/** Max boards per project, enforced server-side (409) and mirrored in the UI. */
export const BOARD_LIMIT = 99;

export function useBoards(projectId: string) {
  return useQuery({
    queryKey: queryKeys.boardsByProject(projectId),
    queryFn: () => listBoards(projectId),
  });
}

export function useCreateBoard(projectId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => createBoard(projectId, { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.boardsByProject(projectId),
      });
    },
    onError: (error) => {
      toast.error(
        error instanceof ApiError && error.status === 409
          ? "You’ve reached the limit of 99 boards for this project."
          : "Couldn’t create the board.",
      );
    },
  });
}

export function useRenameBoard(projectId: string) {
  return useOptimisticListMutation<{ id: string; name: string }, Board>({
    queryKey: queryKeys.boardsByProject(projectId),
    mutationFn: ({ id, name }) => updateBoard(id, { name }),
    optimisticUpdate: (boards, { id, name }) =>
      boards.map((board) => (board.id === id ? { ...board, name } : board)),
    errorMessage: "Couldn’t rename the board.",
  });
}

export function useDeleteBoard(projectId: string) {
  return useOptimisticListMutation<string, Board>({
    queryKey: queryKeys.boardsByProject(projectId),
    mutationFn: (id) => deleteBoard(id),
    optimisticUpdate: (boards, id) => boards.filter((board) => board.id !== id),
    errorMessage: "Couldn’t delete the board.",
  });
}
