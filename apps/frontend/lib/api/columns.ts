import { apiFetch } from "./client";
import {
  columnListSchema,
  columnSchema,
  type Column,
  type NameInput,
  type ReorderInput,
} from "./schemas";

export async function listColumns(boardId: string): Promise<Column[]> {
  return columnListSchema.parse(await apiFetch(`/boards/${boardId}/columns`));
}

export async function createColumn(
  boardId: string,
  input: NameInput,
): Promise<Column> {
  return columnSchema.parse(
    await apiFetch(`/boards/${boardId}/columns`, {
      method: "POST",
      body: input,
    }),
  );
}

export async function updateColumn(
  columnId: string,
  input: NameInput,
): Promise<Column> {
  return columnSchema.parse(
    await apiFetch(`/columns/${columnId}`, { method: "PATCH", body: input }),
  );
}

export async function deleteColumn(columnId: string): Promise<void> {
  await apiFetch(`/columns/${columnId}`, { method: "DELETE" });
}

export async function reorderColumns(
  boardId: string,
  input: ReorderInput,
): Promise<void> {
  await apiFetch(`/boards/${boardId}/columns/reorder`, {
    method: "PUT",
    body: input,
  });
}
