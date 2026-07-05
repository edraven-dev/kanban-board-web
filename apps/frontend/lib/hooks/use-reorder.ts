"use client";

import type { DragEndEvent } from "@dnd-kit/core";
import { arrayMove } from "@dnd-kit/sortable";
import { useQueryClient } from "@tanstack/react-query";

import { useOptimisticListMutation } from "./use-optimistic-list-mutation";

export type ReorderVars = { activeId: string; overId: string };

type Identifiable = { id: string };

/** Translates a dnd-kit drag end into a reorder, skipping no-op drops. */
export function handleDragEnd(
  event: DragEndEvent,
  reorder: (vars: ReorderVars) => void,
) {
  const { active, over } = event;
  if (over && active.id !== over.id) {
    reorder({ activeId: String(active.id), overId: String(over.id) });
  }
}

/** Pure list reorder: move `activeId` to `overId`'s slot. No-op if either is absent. */
export function reorder<T extends Identifiable>(
  items: T[],
  activeId: string,
  overId: string,
): T[] {
  const from = items.findIndex((item) => item.id === activeId);
  const to = items.findIndex((item) => item.id === overId);
  if (from === -1 || to === -1) {
    return items;
  }
  return arrayMove(items, from, to);
}

/**
 * Generic optimistic drag-reorder for any cached list. The cache is reordered
 * immediately; `persist` receives the resulting id order to save server-side.
 */
export function useReorder<T extends Identifiable>(
  queryKey: readonly unknown[],
  persist: (orderedIds: string[]) => Promise<void>,
) {
  const queryClient = useQueryClient();

  return useOptimisticListMutation<ReorderVars, T>({
    queryKey,
    optimisticUpdate: (items, { activeId, overId }) =>
      reorder(items, activeId, overId),
    // Runs after onMutate, so the cache already holds the new order.
    mutationFn: () =>
      persist((queryClient.getQueryData<T[]>(queryKey) ?? []).map((i) => i.id)),
    errorMessage: "Couldn’t save the new order. Reverted.",
  });
}
