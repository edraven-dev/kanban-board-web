import type { DragEndEvent, DragStartEvent } from "@dnd-kit/core";

import type { BoardFull, Card } from "@/lib/api/schemas";
import { findCard, resolveCardDrop } from "@/lib/hooks/use-cards";
import { handleDragEnd, type ReorderVars } from "@/lib/hooks/use-reorder";

/** The lifted card for a drag overlay, or null when a column (not a card) is dragged. */
export function activeCardFromDragStart(
  event: DragStartEvent,
  board: BoardFull,
): Card | null {
  if (event.active.data.current?.type !== "card") return null;
  return findCard(board, String(event.active.id)) ?? null;
}

type BoardDragHandlers = {
  reorderColumns: (vars: ReorderVars) => void;
  moveCard: (vars: {
    cardId: string;
    toColumnId: string;
    toIndex: number;
  }) => void;
};

/** Routes a board drag-end to either a column reorder or a card move. */
export function onBoardDragEnd(
  event: DragEndEvent,
  board: BoardFull,
  handlers: BoardDragHandlers,
) {
  const { active, over } = event;
  if (!over) return;

  if (active.data.current?.type === "column") {
    handleDragEnd(event, handlers.reorderColumns);
    return;
  }

  const overData = over.data.current;
  const move = resolveCardDrop(board, String(active.id), {
    id: String(over.id),
    type: overData?.type as string | undefined,
    columnId: overData?.columnId as string | undefined,
  });
  if (move) handlers.moveCard(move);
}
