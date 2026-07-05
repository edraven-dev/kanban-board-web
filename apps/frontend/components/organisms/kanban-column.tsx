"use client";

import { useState } from "react";
import { type AnimateLayoutChanges, useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVerticalIcon, Trash2Icon } from "lucide-react";

import { ConfirmDeleteDialog } from "@/components/molecules/confirm-delete-dialog";
import { EditableTitle } from "@/components/molecules/editable-title";
import { Button } from "@/components/ui/button";
import type { ColumnFull } from "@/lib/api/schemas";
import { useDeleteColumn, useRenameColumn } from "@/lib/hooks/use-columns";

const noLayoutAnimation: AnimateLayoutChanges = () => false;

export function KanbanColumn({
  boardId,
  column,
}: {
  boardId: string;
  column: ColumnFull;
}) {
  const rename = useRenameColumn(boardId);
  const remove = useDeleteColumn(boardId);
  const [confirmOpen, setConfirmOpen] = useState(false);

  const {
    setNodeRef,
    attributes,
    listeners,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: column.id, animateLayoutChanges: noLayoutAnimation });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : undefined,
  };

  return (
    <section
      ref={setNodeRef}
      style={style}
      className="flex w-72 shrink-0 flex-col rounded-lg border border-border bg-card"
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

      {/* Card list is added in F7. */}
      <div className="flex min-h-24 flex-1 flex-col gap-2 p-2" />

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
