"use client";

import { useState } from "react";
import Link from "next/link";

import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useIsDragging } from "@/lib/dnd/dragging";
import { pastelGradient } from "@/lib/gradient";
import { cn } from "@/lib/utils";

import { EditableTitle } from "./editable-title";
import { EntityMenu } from "./entity-menu";

const SURFACE =
  "flex aspect-[16/9] w-full items-end rounded-xl p-3 text-sm font-medium text-neutral-900 shadow-sm";

type EntityTileProps = {
  name: string;
  seed: string;
  href?: string;
  openLabel?: string;
  menuLabel?: string;
  renameLabel?: string;
  onRename?: (name: string) => void;
  onDelete?: () => void;
  preview?: boolean;
};

export function EntityTile({
  name,
  seed,
  href,
  openLabel,
  menuLabel,
  renameLabel = "Name",
  onRename,
  onDelete,
  preview,
}: EntityTileProps) {
  const [editing, setEditing] = useState(false);
  const dragging = useIsDragging();
  const background = { backgroundImage: pastelGradient(seed) };

  if (preview) {
    return (
      <div className={cn(SURFACE, "shadow-2xl")} style={background}>
        <span className="w-full truncate">{name}</span>
      </div>
    );
  }

  if (editing) {
    return (
      <div className={cn(SURFACE, "items-start")} style={background}>
        <EditableTitle
          value={name}
          label={renameLabel}
          autoEdit
          onDone={() => setEditing(false)}
          onSave={(next) => onRename?.(next)}
          className="w-full"
        />
      </div>
    );
  }

  return (
    <div className="group relative">
      <Tooltip disabled={dragging}>
        <TooltipTrigger
          render={
            <Link
              href={href ?? "#"}
              aria-label={openLabel}
              draggable={false}
              className={SURFACE}
              style={background}
            />
          }
        >
          <span className="w-full truncate">{name}</span>
        </TooltipTrigger>
        <TooltipContent>{name}</TooltipContent>
      </Tooltip>
      {(onRename || onDelete) && (
        <div
          className="absolute right-1.5 top-1.5 text-neutral-900 opacity-0 transition-opacity focus-within:opacity-100 group-hover:opacity-100"
          onPointerDown={(event) => event.stopPropagation()}
        >
          <EntityMenu
            label={menuLabel ?? `${name} actions`}
            onRename={() => setEditing(true)}
            onDelete={() => onDelete?.()}
          />
        </div>
      )}
    </div>
  );
}
