"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import type { BoardFull } from "@/lib/api/schemas";
import { queryKeys } from "@/lib/query/keys";

type Options<TVars> = {
  boardId: string;
  mutationFn: (vars: TVars) => Promise<unknown>;
  /** Pure transform of the cached board-full view for an instant optimistic update. */
  optimisticUpdate: (board: BoardFull, vars: TVars) => BoardFull;
  errorMessage: string;
};

/**
 * Optimistic mutation over the nested board-full cache (board → columns → cards):
 * snapshot → apply → persist → roll back and toast on error → refetch on settle.
 * Shared by column mutations and, later, card mutations.
 */
export function useBoardFullMutation<TVars>({
  boardId,
  mutationFn,
  optimisticUpdate,
  errorMessage,
}: Options<TVars>) {
  const queryClient = useQueryClient();
  const queryKey = queryKeys.boardFull(boardId);

  return useMutation({
    mutationFn,
    onMutate: async (vars: TVars) => {
      await queryClient.cancelQueries({ queryKey });
      const previous = queryClient.getQueryData<BoardFull>(queryKey);
      if (previous) {
        queryClient.setQueryData<BoardFull>(
          queryKey,
          optimisticUpdate(previous, vars),
        );
      }
      return { previous };
    },
    onError: (_error, _vars, context) => {
      const previous = (context as { previous?: BoardFull } | undefined)
        ?.previous;
      if (previous) {
        queryClient.setQueryData(queryKey, previous);
      }
      toast.error(errorMessage);
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey });
    },
  });
}
