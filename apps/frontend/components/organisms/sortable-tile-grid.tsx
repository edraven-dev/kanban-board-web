"use client";

import { useEffect, useRef, useState } from "react";
import {
  closestCenter,
  DndContext,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  type AnimateLayoutChanges,
  rectSortingStrategy,
  SortableContext,
  useSortable,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

import { ConfirmDeleteDialog } from "@/components/molecules/confirm-delete-dialog";
import { EntityTile } from "@/components/molecules/entity-tile";
import { TileGrid } from "@/components/molecules/tile-grid";
import { DraggingProvider } from "@/lib/dnd/dragging";
import { handleDragEnd, useReorder } from "@/lib/hooks/use-reorder";

const noLayoutAnimation: AnimateLayoutChanges = () => false;

type TileItem = { id: string; name: string };

type SortableTileGridProps<T extends TileItem> = {
  items: T[];
  queryKey: readonly unknown[];
  reorderPersist: (orderedIds: string[]) => Promise<void>;
  hrefFor: (item: T) => string;
  openLabelFor: (item: T) => string;
  menuLabelFor: (item: T) => string;
  renameLabel: string;
  deleteTitleFor: (item: T) => string;
  deleteDescription: string;
  onRename: (id: string, name: string) => void;
  onDelete: (id: string) => void;
};

export function SortableTileGrid<T extends TileItem>({
  items,
  queryKey,
  reorderPersist,
  hrefFor,
  openLabelFor,
  menuLabelFor,
  renameLabel,
  deleteTitleFor,
  deleteDescription,
  onRename,
  onDelete,
}: SortableTileGridProps<T>) {
  const [dragging, setDragging] = useState(false);
  const reorder = useReorder<T>(queryKey, reorderPersist);

  // Distance constraint so a tap opens the tile and only a drag reorders it.
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
  );

  return (
    <DraggingProvider value={dragging}>
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragStart={() => setDragging(true)}
        onDragEnd={(event) => {
          handleDragEnd(event, reorder.mutate);
          setDragging(false);
        }}
        onDragCancel={() => setDragging(false)}
      >
        <SortableContext
          items={items.map((item) => item.id)}
          strategy={rectSortingStrategy}
        >
          <TileGrid>
            {items.map((item) => (
              <SortableTile
                key={item.id}
                id={item.id}
                name={item.name}
                href={hrefFor(item)}
                openLabel={openLabelFor(item)}
                menuLabel={menuLabelFor(item)}
                renameLabel={renameLabel}
                deleteTitle={deleteTitleFor(item)}
                deleteDescription={deleteDescription}
                onRename={(name) => onRename(item.id, name)}
                onDelete={() => onDelete(item.id)}
              />
            ))}
          </TileGrid>
        </SortableContext>
      </DndContext>
    </DraggingProvider>
  );
}

type SortableTileProps = {
  id: string;
  name: string;
  href: string;
  openLabel: string;
  menuLabel: string;
  renameLabel: string;
  deleteTitle: string;
  deleteDescription: string;
  onRename: (name: string) => void;
  onDelete: () => void;
};

function SortableTile({
  id,
  name,
  href,
  openLabel,
  menuLabel,
  renameLabel,
  deleteTitle,
  deleteDescription,
  onRename,
  onDelete,
}: SortableTileProps) {
  const [confirmOpen, setConfirmOpen] = useState(false);
  const { setNodeRef, listeners, transform, transition, isDragging } =
    useSortable({ id, animateLayoutChanges: noLayoutAnimation });

  const draggedRef = useRef(false);
  useEffect(() => {
    if (isDragging) draggedRef.current = true;
  }, [isDragging]);

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : undefined,
  };

  return (
    <>
      <div
        ref={setNodeRef}
        style={style}
        {...listeners}
        className="touch-none"
        onPointerDownCapture={() => {
          draggedRef.current = false;
        }}
        onClickCapture={(event) => {
          if (draggedRef.current) {
            event.preventDefault();
            event.stopPropagation();
            draggedRef.current = false;
          }
        }}
      >
        <EntityTile
          name={name}
          seed={id}
          href={href}
          openLabel={openLabel}
          menuLabel={menuLabel}
          renameLabel={renameLabel}
          onRename={onRename}
          onDelete={() => setConfirmOpen(true)}
        />
      </div>
      <ConfirmDeleteDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        title={deleteTitle}
        description={deleteDescription}
        onConfirm={onDelete}
      />
    </>
  );
}
