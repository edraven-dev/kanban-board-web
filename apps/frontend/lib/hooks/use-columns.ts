"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import { getBoardFull } from "@/lib/api/boards";
import { ApiError } from "@/lib/api/client";
import {
  createColumn,
  deleteColumn,
  reorderColumns,
  updateColumn,
} from "@/lib/api/columns";
import type { BoardFull } from "@/lib/api/schemas";
import { queryKeys } from "@/lib/query/keys";

import { useBoardFullMutation } from "./use-board-full-mutation";
import { reorder, type ReorderVars } from "./use-reorder";

/** Max columns per board, enforced server-side (409) and mirrored in the UI. */
export const COLUMN_LIMIT = 99;

export function useBoardFull(boardId: string) {
  return useQuery({
    queryKey: queryKeys.boardFull(boardId),
    queryFn: () => getBoardFull(boardId),
  });
}

export function useCreateColumn(boardId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => createColumn(boardId, { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.boardFull(boardId) });
    },
    onError: (error) => {
      toast.error(
        error instanceof ApiError && error.status === 409
          ? "You’ve reached the limit of 99 columns for this board."
          : "Couldn’t create the column.",
      );
    },
  });
}

export function useRenameColumn(boardId: string) {
  return useBoardFullMutation<{ id: string; name: string }>({
    boardId,
    mutationFn: ({ id, name }) => updateColumn(id, { name }),
    optimisticUpdate: (board, { id, name }) => ({
      ...board,
      columns: board.columns.map((column) =>
        column.id === id ? { ...column, name } : column,
      ),
    }),
    errorMessage: "Couldn’t rename the column.",
  });
}

export function useDeleteColumn(boardId: string) {
  return useBoardFullMutation<string>({
    boardId,
    mutationFn: (id) => deleteColumn(id),
    optimisticUpdate: (board, id) => ({
      ...board,
      columns: board.columns.filter((column) => column.id !== id),
    }),
    errorMessage: "Couldn’t delete the column.",
  });
}

export function useReorderColumns(boardId: string) {
  const queryClient = useQueryClient();
  const queryKey = queryKeys.boardFull(boardId);

  return useBoardFullMutation<ReorderVars>({
    boardId,
    optimisticUpdate: (board, { activeId, overId }) => ({
      ...board,
      columns: reorder(board.columns, activeId, overId),
    }),
    // Runs after onMutate, so the cache already holds the new column order.
    mutationFn: () =>
      reorderColumns(boardId, {
        orderedIds: (
          queryClient.getQueryData<BoardFull>(queryKey)?.columns ?? []
        ).map((column) => column.id),
      }),
    errorMessage: "Couldn’t save the new order. Reverted.",
  });
}
