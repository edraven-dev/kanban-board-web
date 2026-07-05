"use client";

import { useState } from "react";
import {
  type AnimateLayoutChanges,
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVerticalIcon, Trash2Icon } from "lucide-react";

import { CardTile } from "@/components/molecules/card-tile";
import { ConfirmDeleteDialog } from "@/components/molecules/confirm-delete-dialog";
import { EditableTitle } from "@/components/molecules/editable-title";
import { InlineCreate } from "@/components/molecules/inline-create";
import { Button } from "@/components/ui/button";
import type { Card, ColumnFull } from "@/lib/api/schemas";
import { useClickAfterDragGuard } from "@/lib/dnd/use-click-after-drag-guard";
import { useCreateCard } from "@/lib/hooks/use-cards";
import { useDeleteColumn, useRenameColumn } from "@/lib/hooks/use-columns";

const noLayoutAnimation: AnimateLayoutChanges = () => false;

export function KanbanColumn({
  boardId,
  column,
  onCardClick,
}: {
  boardId: string;
  column: ColumnFull;
  onCardClick: (cardId: string) => void;
}) {
  const rename = useRenameColumn(boardId);
  const remove = useDeleteColumn(boardId);
  const createCard = useCreateCard(boardId);
  const [confirmOpen, setConfirmOpen] = useState(false);

  const {
    setNodeRef,
    attributes,
    listeners,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: column.id,
    data: { type: "column" },
    animateLayoutChanges: noLayoutAnimation,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : undefined,
  };

  return (
    <section
      ref={setNodeRef}
      style={style}
      className="flex max-h-full w-72 shrink-0 flex-col rounded-lg border border-border bg-card"
      aria-label={column.name}
    >
      <header className="flex items-center gap-1 border-b border-border p-2">
        <button
          type="button"
          aria-label={`Reorder ${column.name}`}
          className="cursor-grab touch-none text-muted-foreground"
          {...attributes}
          {...listeners}
        >
          <GripVerticalIcon className="size-4" />
        </button>
        <div className="flex-1 text-sm font-medium">
          <EditableTitle
            value={column.name}
            label="Column name"
            onSave={(name) => rename.mutate({ id: column.id, name })}
          />
        </div>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label={`Delete ${column.name}`}
          onClick={() => setConfirmOpen(true)}
        >
          <Trash2Icon />
        </Button>
      </header>

      <div className="flex min-h-16 flex-1 flex-col gap-2 overflow-y-auto p-2">
        <SortableContext
          items={column.cards.map((card) => card.id)}
          strategy={verticalListSortingStrategy}
        >
          {column.cards.map((card) => (
            <SortableCard
              key={card.id}
              card={card}
              onOpen={() => onCardClick(card.id)}
            />
          ))}
        </SortableContext>
      </div>

      <div className="p-2 pt-0">
        <InlineCreate
          label="card"
          maxLength={200}
          onCreate={(title) => createCard.mutate({ columnId: column.id, title })}
        />
      </div>

      <ConfirmDeleteDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        title={`Delete “${column.name}”?`}
        description="This permanently deletes the column and all of its cards."
        onConfirm={() => remove.mutate(column.id)}
      />
    </section>
  );
}

function SortableCard({ card, onOpen }: { card: Card; onOpen: () => void }) {
  const { setNodeRef, listeners, transform, transition, isDragging } =
    useSortable({
      id: card.id,
      data: { type: "card", columnId: card.columnId },
      animateLayoutChanges: noLayoutAnimation,
    });
  const clickGuard = useClickAfterDragGuard(isDragging);

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : undefined,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...listeners}
      {...clickGuard}
      role="button"
      tabIndex={0}
      aria-label={card.title}
      className="touch-none rounded-xl outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onClick={onOpen}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onOpen();
        }
      }}
    >
      <CardTile card={card} />
    </div>
  );
}
