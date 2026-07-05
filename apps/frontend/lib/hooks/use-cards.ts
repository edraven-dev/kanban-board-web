"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { arrayMove } from "@dnd-kit/sortable";
import { toast } from "sonner";

import { createCard, deleteCard, moveCard, updateCard } from "@/lib/api/cards";
import type { BoardFull, Card } from "@/lib/api/schemas";
import { queryKeys } from "@/lib/query/keys";

import { useBoardFullMutation } from "./use-board-full-mutation";

export function findCardLocation(
  board: BoardFull,
  cardId: string,
): { columnId: string; index: number } | null {
  for (const column of board.columns) {
    const index = column.cards.findIndex((card) => card.id === cardId);
    if (index !== -1) return { columnId: column.id, index };
  }
  return null;
}

export function findCard(board: BoardFull, cardId: string): Card | undefined {
  const location = findCardLocation(board, cardId);
  if (!location) return undefined;
  return board.columns.find((column) => column.id === location.columnId)?.cards[
    location.index
  ];
}

export type CardDropTarget = { id: string; type?: string; columnId?: string };

/** Resolves a card drag-end into move vars, or null for a no-op / invalid drop. */
export function resolveCardDrop(
  board: BoardFull,
  cardId: string,
  over: CardDropTarget | null,
): { cardId: string; toColumnId: string; toIndex: number } | null {
  if (!over) return null;
  const from = findCardLocation(board, cardId);
  if (!from) return null;

  let toColumnId: string;
  let toIndex: number;
  if (over.type === "card") {
    toColumnId = over.columnId ?? "";
    const column = board.columns.find((c) => c.id === toColumnId);
    if (!column) return null;
    toIndex = column.cards.findIndex((c) => c.id === over.id);
  } else {
    toColumnId = over.id;
    const column = board.columns.find((c) => c.id === toColumnId);
    if (!column) return null;
    toIndex = column.cards.length;
  }

  if (from.columnId === toColumnId && from.index === toIndex) return null;
  return { cardId, toColumnId, toIndex };
}

/** Pure move of a card to `toColumnId` at `toIndex` (same-column = reorder). */
export function moveCardInBoard(
  board: BoardFull,
  cardId: string,
  toColumnId: string,
  toIndex: number,
): BoardFull {
  const from = findCardLocation(board, cardId);
  if (!from) return board;

  const source = board.columns.find((c) => c.id === from.columnId);
  const card = source?.cards[from.index];
  if (!card) return board;

  if (from.columnId === toColumnId) {
    return {
      ...board,
      columns: board.columns.map((column) =>
        column.id === toColumnId
          ? { ...column, cards: arrayMove(column.cards, from.index, toIndex) }
          : column,
      ),
    };
  }

  return {
    ...board,
    columns: board.columns.map((column) => {
      if (column.id === from.columnId) {
        return {
          ...column,
          cards: column.cards.filter((c) => c.id !== cardId),
        };
      }
      if (column.id === toColumnId) {
        const cards = [...column.cards];
        const clamped = Math.max(0, Math.min(toIndex, cards.length));
        cards.splice(clamped, 0, { ...card, columnId: toColumnId });
        return { ...column, cards };
      }
      return column;
    }),
  };
}

function mapCard(
  board: BoardFull,
  cardId: string,
  update: (card: Card) => Card,
): BoardFull {
  return {
    ...board,
    columns: board.columns.map((column) => ({
      ...column,
      cards: column.cards.map((card) =>
        card.id === cardId ? update(card) : card,
      ),
    })),
  };
}

export function useCreateCard(boardId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ columnId, title }: { columnId: string; title: string }) =>
      createCard(columnId, { title }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.boardFull(boardId) });
    },
    onError: () => toast.error("Couldn’t create the card."),
  });
}

export function useUpdateCard(boardId: string) {
  return useBoardFullMutation<{
    id: string;
    title?: string;
    description?: string;
  }>({
    boardId,
    mutationFn: ({ id, title, description }) =>
      updateCard(id, { title, description }),
    optimisticUpdate: (board, { id, title, description }) =>
      mapCard(board, id, (card) => ({
        ...card,
        ...(title !== undefined ? { title } : {}),
        ...(description !== undefined ? { description } : {}),
      })),
    errorMessage: "Couldn’t save the card.",
  });
}

export function useDeleteCard(boardId: string) {
  return useBoardFullMutation<string>({
    boardId,
    mutationFn: (id) => deleteCard(id),
    optimisticUpdate: (board, id) => ({
      ...board,
      columns: board.columns.map((column) => ({
        ...column,
        cards: column.cards.filter((card) => card.id !== id),
      })),
    }),
    errorMessage: "Couldn’t delete the card.",
  });
}

export function useMoveCard(boardId: string) {
  return useBoardFullMutation<{
    cardId: string;
    toColumnId: string;
    toIndex: number;
  }>({
    boardId,
    mutationFn: ({ cardId, toColumnId, toIndex }) =>
      moveCard(cardId, { columnId: toColumnId, position: toIndex }),
    optimisticUpdate: (board, { cardId, toColumnId, toIndex }) =>
      moveCardInBoard(board, cardId, toColumnId, toIndex),
    errorMessage: "Couldn’t move the card. Reverted.",
  });
}
