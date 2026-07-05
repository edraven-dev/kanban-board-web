import { z } from "zod";

const isoDatetime = z.iso.datetime();

export const projectSchema = z.object({
  id: z.uuid(),
  name: z.string(),
  position: z.number().int(),
  createdAt: isoDatetime,
  updatedAt: isoDatetime,
});

export const boardSchema = z.object({
  id: z.uuid(),
  projectId: z.uuid(),
  name: z.string(),
  position: z.number().int(),
  createdAt: isoDatetime,
  updatedAt: isoDatetime,
});

export const columnSchema = z.object({
  id: z.uuid(),
  boardId: z.uuid(),
  name: z.string(),
  position: z.number().int(),
  createdAt: isoDatetime,
  updatedAt: isoDatetime,
});

export const cardSchema = z.object({
  id: z.uuid(),
  columnId: z.uuid(),
  title: z.string(),
  description: z.string(),
  position: z.number().int(),
  createdAt: isoDatetime,
  updatedAt: isoDatetime,
});

export const columnFullSchema = columnSchema.extend({
  cards: z.array(cardSchema),
});

export const boardFullSchema = boardSchema.extend({
  columns: z.array(columnFullSchema),
});

export const projectListSchema = z.array(projectSchema);
export const boardListSchema = z.array(boardSchema);
export const columnListSchema = z.array(columnSchema);
export const cardListSchema = z.array(cardSchema);

export type Project = z.infer<typeof projectSchema>;
export type Board = z.infer<typeof boardSchema>;
export type Column = z.infer<typeof columnSchema>;
export type Card = z.infer<typeof cardSchema>;
export type ColumnFull = z.infer<typeof columnFullSchema>;
export type BoardFull = z.infer<typeof boardFullSchema>;

export type NameInput = { name: string };
export type ReorderInput = { orderedIds: string[] };
export type CreateCardInput = { title: string; description?: string };
export type UpdateCardInput = { title?: string; description?: string };
export type MoveCardInput = { columnId: string; position: number };
