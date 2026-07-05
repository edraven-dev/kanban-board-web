"use client";

import type { ReactNode } from "react";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVerticalIcon } from "lucide-react";

import { cn } from "@/lib/utils";

type SortableItemProps = {
  id: string;
  children: ReactNode;
  handleLabel?: string;
  className?: string;
};

/**
 * dnd-kit `useSortable` wrapper with a keyboard-accessible drag handle. One
 * implementation shared by project, board, column, and card reordering.
 */
export function SortableItem({
  id,
  children,
  handleLabel = "Drag to reorder",
  className,
}: SortableItemProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : undefined,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      data-dragging={isDragging || undefined}
      className={cn("flex items-start gap-1", className)}
    >
      <button
        type="button"
        aria-label={handleLabel}
        className="cursor-grab touch-none text-muted-foreground"
        {...attributes}
        {...listeners}
      >
        <GripVerticalIcon className="size-4" />
      </button>
      <div className="flex-1">{children}</div>
    </div>
  );
}
