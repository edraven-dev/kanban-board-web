"use client";

import { useState } from "react";
import Link from "next/link";
import {
  closestCenter,
  closestCorners,
  type CollisionDetection,
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  horizontalListSortingStrategy,
  SortableContext,
  sortableKeyboardCoordinates,
} from "@dnd-kit/sortable";
import { ArrowLeftIcon } from "lucide-react";

import { CardTile } from "@/components/molecules/card-tile";
import { InlineCreate } from "@/components/molecules/inline-create";
import { KanbanColumn } from "@/components/organisms/kanban-column";
import { CardModal } from "@/components/organisms/card-modal";
import { buttonVariants } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import type { Card } from "@/lib/api/schemas";
import {
  COLUMN_LIMIT,
  useBoardFull,
  useCreateColumn,
  useReorderColumns,
} from "@/lib/hooks/use-columns";
import { activeCardFromDragStart, onBoardDragEnd } from "@/lib/dnd/board-dnd";
import { useMoveCard } from "@/lib/hooks/use-cards";

// While dragging a column, only collide with columns; cards collide with both.
const collisionDetection: CollisionDetection = (args) => {
  if (args.active.data.current?.type === "column") {
    return closestCenter({
      ...args,
      droppableContainers: args.droppableContainers.filter(
        (container) => container.data.current?.type === "column",
      ),
    });
  }
  return closestCorners(args);
};

export function BoardCanvas({ boardId }: { boardId: string }) {
  const { data: board, isPending, isError } = useBoardFull(boardId);
  const createColumn = useCreateColumn(boardId);
  const reorderColumns = useReorderColumns(boardId);
  const moveCard = useMoveCard(boardId);
  const [activeCard, setActiveCard] = useState<Card | null>(null);
  const [selectedCardId, setSelectedCardId] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  if (isPending) {
    return <BoardCanvasSkeleton />;
  }

  if (isError) {
    return (
      <main className="flex flex-1 flex-col px-4 py-6">
        <p role="alert" className="text-sm text-destructive">
          Couldn’t load the board. Please try again.
        </p>
      </main>
    );
  }

  const atLimit = board.columns.length >= COLUMN_LIMIT;
  const selectedColumn = board.columns.find((column) =>
    column.cards.some((card) => card.id === selectedCardId),
  );
  const selectedCard = selectedColumn?.cards.find(
    (card) => card.id === selectedCardId,
  );

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <header className="flex items-center gap-3 border-b border-border px-4 py-3">
        <Link
          href={`/projects/${board.projectId}`}
          aria-label="Back to boards"
          className={buttonVariants({ variant: "ghost", size: "icon-sm" })}
        >
          <ArrowLeftIcon />
        </Link>
        <h1 className="truncate text-lg font-semibold tracking-tight">
          {board.name}
        </h1>
      </header>

      <DndContext
        sensors={sensors}
        collisionDetection={collisionDetection}
        onDragStart={(event) => setActiveCard(activeCardFromDragStart(event, board))}
        onDragEnd={(event) => {
          setActiveCard(null);
          onBoardDragEnd(event, board, {
            reorderColumns: reorderColumns.mutate,
            moveCard: moveCard.mutate,
          });
        }}
        onDragCancel={() => setActiveCard(null)}
      >
        <SortableContext
          items={board.columns.map((column) => column.id)}
          strategy={horizontalListSortingStrategy}
        >
          <div
            data-testid="board-scroll"
            className="flex min-h-0 flex-1 items-start gap-3 overflow-x-auto overflow-y-hidden p-4"
            onWheel={(event) => {
              if (event.deltaY !== 0) {
                event.currentTarget.scrollLeft += event.deltaY;
              }
            }}
          >
            {board.columns.map((column) => (
              <KanbanColumn
                key={column.id}
                boardId={boardId}
                column={column}
                onCardClick={setSelectedCardId}
              />
            ))}
            {!atLimit && (
              <div className="w-72 shrink-0">
                <InlineCreate
                  label="column"
                  onCreate={(name) => createColumn.mutate(name)}
                />
              </div>
            )}
          </div>
        </SortableContext>
        <DragOverlay dropAnimation={null}>
          {activeCard ? <CardTile card={activeCard} /> : null}
        </DragOverlay>
      </DndContext>

      <CardModal
        boardId={boardId}
        card={selectedCard}
        columnName={selectedColumn?.name}
        open={selectedCard !== undefined}
        onOpenChange={(open) => {
          if (!open) setSelectedCardId(null);
        }}
      />
    </div>
  );
}

function BoardCanvasSkeleton() {
  return (
    <div className="flex flex-1 flex-col" data-testid="board-skeleton">
      <div className="flex items-center gap-3 border-b border-border px-4 py-3">
        <Skeleton className="h-7 w-40" />
      </div>
      <div className="flex items-start gap-3 p-4">
        {Array.from({ length: 3 }).map((_, index) => (
          <Skeleton key={index} className="h-64 w-72 shrink-0 rounded-lg" />
        ))}
      </div>
    </div>
  );
}
