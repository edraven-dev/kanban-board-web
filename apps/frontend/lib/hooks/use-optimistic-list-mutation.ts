"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

type Options<TVars, TItem> = {
  queryKey: readonly unknown[];
  mutationFn: (vars: TVars) => Promise<unknown>;
  /** Pure transform applied to the cached list for an instant, optimistic update. */
  optimisticUpdate: (items: TItem[], vars: TVars) => TItem[];
  errorMessage: string;
};

/** Shared optimistic list mutation: apply → persist → roll back + toast on error. */
export function useOptimisticListMutation<TVars, TItem>({
  queryKey,
  mutationFn,
  optimisticUpdate,
  errorMessage,
}: Options<TVars, TItem>) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn,
    onMutate: async (vars: TVars) => {
      await queryClient.cancelQueries({ queryKey });
      const previous = queryClient.getQueryData<TItem[]>(queryKey);
      if (previous) {
        queryClient.setQueryData<TItem[]>(
          queryKey,
          optimisticUpdate(previous, vars),
        );
      }
      return { previous };
    },
    onError: (_error, _vars, context) => {
      const previous = (context as { previous?: TItem[] } | undefined)?.previous;
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
