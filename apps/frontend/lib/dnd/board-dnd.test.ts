import type { DragEndEvent, DragStartEvent } from "@dnd-kit/core";
import { describe, expect, it, vi } from "vitest";

import type { BoardFull, Card, ColumnFull } from "@/lib/api/schemas";

import { activeCardFromDragStart, onBoardDragEnd } from "./board-dnd";

const ts = "2026-07-05T00:00:00Z";
const boardId = "b";
const col1 = "col-1";
const col2 = "col-2";
const cardA = "card-a";
const cardC = "card-c";

function card(id: string, columnId: string, position: number): Card {
  return { id, columnId, title: id, description: "", position, createdAt: ts, updatedAt: ts };
}

function column(id: string, position: number, cards: Card[]): ColumnFull {
  return { id, boardId, name: id, position, createdAt: ts, updatedAt: ts, cards };
}

const board: BoardFull = {
  id: boardId,
  projectId: "p",
  name: "B",
  position: 0,
  createdAt: ts,
  updatedAt: ts,
  columns: [column(col1, 0, [card(cardA, col1, 0)]), column(col2, 1, [card(cardC, col2, 0)])],
};

function dragStart(type: string, id: string): DragStartEvent {
  return { active: { id, data: { current: { type } } } } as unknown as DragStartEvent;
}

function dragEnd(
  active: { id: string; type: string },
  over: { id: string; type?: string; columnId?: string } | null,
): DragEndEvent {
  return {
    active: { id: active.id, data: { current: { type: active.type } } },
    over: over
      ? { id: over.id, data: { current: { type: over.type, columnId: over.columnId } } }
      : null,
  } as unknown as DragEndEvent;
}

describe("activeCardFromDragStart", () => {
  it("returns the lifted card for a card drag", () => {
    expect(activeCardFromDragStart(dragStart("card", cardA), board)?.id).toBe(cardA);
  });

  it("returns null for a column drag", () => {
    expect(activeCardFromDragStart(dragStart("column", col1), board)).toBeNull();
  });
});

describe("onBoardDragEnd", () => {
  it("reorders columns when a column is dropped on another", () => {
    const reorderColumns = vi.fn();
    const moveCard = vi.fn();
    onBoardDragEnd(
      dragEnd({ id: col1, type: "column" }, { id: col2, type: "column" }),
      board,
      { reorderColumns, moveCard },
    );
    expect(reorderColumns).toHaveBeenCalledWith({ activeId: col1, overId: col2 });
    expect(moveCard).not.toHaveBeenCalled();
  });

  it("moves a card across columns", () => {
    const reorderColumns = vi.fn();
    const moveCard = vi.fn();
    onBoardDragEnd(
      dragEnd({ id: cardA, type: "card" }, { id: cardC, type: "card", columnId: col2 }),
      board,
      { reorderColumns, moveCard },
    );
    expect(moveCard).toHaveBeenCalledWith({
      cardId: cardA,
      toColumnId: col2,
      toIndex: 0,
    });
  });

  it("does nothing without a drop target", () => {
    const reorderColumns = vi.fn();
    const moveCard = vi.fn();
    onBoardDragEnd(dragEnd({ id: cardA, type: "card" }, null), board, {
      reorderColumns,
      moveCard,
    });
    expect(reorderColumns).not.toHaveBeenCalled();
    expect(moveCard).not.toHaveBeenCalled();
  });

  it("does nothing for a no-op card drop", () => {
    const moveCard = vi.fn();
    onBoardDragEnd(
      dragEnd({ id: cardA, type: "card" }, { id: cardA, type: "card", columnId: col1 }),
      board,
      { reorderColumns: vi.fn(), moveCard },
    );
    expect(moveCard).not.toHaveBeenCalled();
  });
});
