"use client";

import { reorderBoards } from "@/lib/api/boards";
import type { Board } from "@/lib/api/schemas";
import { useDeleteBoard, useRenameBoard } from "@/lib/hooks/use-boards";
import { queryKeys } from "@/lib/query/keys";

import { SortableTileGrid } from "./sortable-tile-grid";

export function BoardList({
  projectId,
  boards,
}: {
  projectId: string;
  boards: Board[];
}) {
  const rename = useRenameBoard(projectId);
  const remove = useDeleteBoard(projectId);

  return (
    <SortableTileGrid
      items={boards}
      queryKey={queryKeys.boardsByProject(projectId)}
      reorderPersist={(orderedIds) => reorderBoards(projectId, { orderedIds })}
      hrefFor={(board) => `/boards/${board.id}`}
      openLabelFor={(board) => `Open ${board.name}`}
      menuLabelFor={(board) => `${board.name} actions`}
      renameLabel="Board name"
      deleteTitleFor={(board) => `Delete “${board.name}”?`}
      deleteDescription="This permanently deletes the board and all of its columns and cards."
      onRename={(id, name) => rename.mutate({ id, name })}
      onDelete={(id) => remove.mutate(id)}
    />
  );
}
