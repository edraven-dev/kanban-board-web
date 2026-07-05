"use client";

import Link from "next/link";
import {
  closestCenter,
  DndContext,
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

import { InlineCreate } from "@/components/molecules/inline-create";
import { KanbanColumn } from "@/components/organisms/kanban-column";
import { buttonVariants } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
  COLUMN_LIMIT,
  useBoardFull,
  useCreateColumn,
  useReorderColumns,
} from "@/lib/hooks/use-columns";
import { handleDragEnd } from "@/lib/hooks/use-reorder";

export function BoardCanvas({ boardId }: { boardId: string }) {
  const { data: board, isPending, isError } = useBoardFull(boardId);
  const createColumn = useCreateColumn(boardId);
  const reorderColumns = useReorderColumns(boardId);

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
        collisionDetection={closestCenter}
        onDragEnd={(event) => handleDragEnd(event, reorderColumns.mutate)}
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
              <KanbanColumn key={column.id} boardId={boardId} column={column} />
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
      </DndContext>
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
