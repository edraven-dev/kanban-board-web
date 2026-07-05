import { apiFetch } from "./client";
import {
  boardFullSchema,
  boardListSchema,
  boardSchema,
  type Board,
  type BoardFull,
  type NameInput,
  type ReorderInput,
} from "./schemas";

export async function listBoards(projectId: string): Promise<Board[]> {
  return boardListSchema.parse(
    await apiFetch(`/projects/${projectId}/boards`),
  );
}

export async function createBoard(
  projectId: string,
  input: NameInput,
): Promise<Board> {
  return boardSchema.parse(
    await apiFetch(`/projects/${projectId}/boards`, {
      method: "POST",
      body: input,
    }),
  );
}

export async function getBoardFull(boardId: string): Promise<BoardFull> {
  return boardFullSchema.parse(await apiFetch(`/boards/${boardId}/full`));
}

export async function updateBoard(
  boardId: string,
  input: NameInput,
): Promise<Board> {
  return boardSchema.parse(
    await apiFetch(`/boards/${boardId}`, { method: "PATCH", body: input }),
  );
}

export async function deleteBoard(boardId: string): Promise<void> {
  await apiFetch(`/boards/${boardId}`, { method: "DELETE" });
}

export async function reorderBoards(
  projectId: string,
  input: ReorderInput,
): Promise<void> {
  await apiFetch(`/projects/${projectId}/boards/reorder`, {
    method: "PUT",
    body: input,
  });
}
