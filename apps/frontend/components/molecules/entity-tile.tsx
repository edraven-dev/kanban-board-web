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
  href: string;
  openLabel: string;
  menuLabel: string;
  renameLabel: string;
  onRename: (name: string) => void;
  onDelete: () => void;
};

export function EntityTile({
  name,
  seed,
  href,
  openLabel,
  menuLabel,
  renameLabel,
  onRename,
  onDelete,
}: EntityTileProps) {
  const [editing, setEditing] = useState(false);
  const dragging = useIsDragging();
  const background = { backgroundImage: pastelGradient(seed) };

  if (editing) {
    return (
      <div className={cn(SURFACE, "items-start")} style={background}>
        <EditableTitle
          value={name}
          label={renameLabel}
          autoEdit
          onDone={() => setEditing(false)}
          onSave={onRename}
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
              href={href}
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
      <div
        className="absolute right-1.5 top-1.5 text-neutral-900 opacity-0 transition-opacity focus-within:opacity-100 group-hover:opacity-100"
        onPointerDown={(event) => event.stopPropagation()}
      >
        <EntityMenu
          label={menuLabel}
          onRename={() => setEditing(true)}
          onDelete={onDelete}
        />
      </div>
    </div>
  );
}
