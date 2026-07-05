"use client";

import { LayoutDashboardIcon } from "lucide-react";

import { EmptyState } from "@/components/atoms/empty-state";
import { InlineCreate } from "@/components/molecules/inline-create";
import { TileGrid } from "@/components/molecules/tile-grid";
import { BoardList } from "@/components/organisms/board-list";
import { ProjectSwitcher } from "@/components/organisms/project-switcher";
import { Skeleton } from "@/components/ui/skeleton";
import { BOARD_LIMIT, useBoards, useCreateBoard } from "@/lib/hooks/use-boards";

export function ProjectBoards({ projectId }: { projectId: string }) {
  const { data: boards, isPending, isError } = useBoards(projectId);
  const createBoard = useCreateBoard(projectId);

  const atLimit = (boards?.length ?? 0) >= BOARD_LIMIT;

  return (
    <main className="mx-auto flex w-full max-w-6xl flex-1 flex-col gap-4 px-4 py-6 sm:px-6">
      <div className="flex items-center justify-between gap-4">
        <ProjectSwitcher currentProjectId={projectId} />
        {!isPending && !isError && !atLimit && (
          <InlineCreate
            label="board"
            onCreate={(name) => createBoard.mutate(name)}
          />
        )}
      </div>

      {isPending ? (
        <BoardsSkeleton />
      ) : isError ? (
        <p role="alert" className="text-sm text-destructive">
          Couldn’t load boards. Please try again.
        </p>
      ) : boards.length === 0 ? (
        <EmptyState
          icon={<LayoutDashboardIcon />}
          title="No boards yet"
          description="Create your first board to start organizing cards."
        />
      ) : (
        <BoardList projectId={projectId} boards={boards} />
      )}
    </main>
  );
}

function BoardsSkeleton() {
  return (
    <div data-testid="boards-skeleton">
      <TileGrid>
        {Array.from({ length: 6 }).map((_, index) => (
          <Skeleton key={index} className="aspect-[16/9] w-full rounded-xl" />
        ))}
      </TileGrid>
    </div>
  );
}
