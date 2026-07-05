import { describe, expect, it } from "vitest";

import {
  boardFullSchema,
  cardSchema,
  projectListSchema,
  projectSchema,
} from "./schemas";

const uuid = "018f1e2d-3c4b-7a6d-8e9f-0123456789ab";
const ts = "2026-07-05T00:00:00Z";

const project = { id: uuid, name: "Alpha", position: 0, createdAt: ts, updatedAt: ts };
const card = {
  id: uuid,
  columnId: uuid,
  title: "T",
  description: "",
  position: 3,
  createdAt: ts,
  updatedAt: ts,
};

describe("entity schemas", () => {
  it("accepts a valid project including a v7 uuid", () => {
    expect(projectSchema.parse(project)).toEqual(project);
  });

  it("accepts ISO-8601 UTC timestamps at the precisions the backend emits", () => {
    for (const stamp of [
      "2026-07-05T00:49:37Z",
      "2026-07-05T00:49:37.123456Z",
      "2026-07-05T00:49:37.123456789Z",
    ]) {
      expect(
        projectSchema.safeParse({ ...project, createdAt: stamp, updatedAt: stamp })
          .success,
      ).toBe(true);
    }
  });

  it("rejects a timestamp that is not an ISO datetime", () => {
    expect(
      projectSchema.safeParse({ ...project, createdAt: "2026-07-05 00:00:00" })
        .success,
    ).toBe(false);
    expect(
      projectSchema.safeParse({ ...project, updatedAt: "yesterday" }).success,
    ).toBe(false);
  });

  it("rejects a non-uuid id", () => {
    expect(projectSchema.safeParse({ ...project, id: "nope" }).success).toBe(
      false,
    );
  });

  it("rejects a non-integer position", () => {
    expect(projectSchema.safeParse({ ...project, position: 1.5 }).success).toBe(
      false,
    );
  });

  it("rejects a payload missing a field", () => {
    expect(
      projectSchema.safeParse({ id: uuid, position: 0, createdAt: ts, updatedAt: ts })
        .success,
    ).toBe(false);
  });

  it("accepts an empty card description but requires a string", () => {
    expect(cardSchema.parse(card).description).toBe("");
    expect(cardSchema.safeParse({ ...card, description: null }).success).toBe(
      false,
    );
  });

  it("parses the nested board-full read model with empty arrays intact", () => {
    const parsed = boardFullSchema.parse({
      id: uuid,
      projectId: uuid,
      name: "B",
      position: 0,
      createdAt: ts,
      updatedAt: ts,
      columns: [
        {
          id: uuid,
          boardId: uuid,
          name: "To Do",
          position: 0,
          createdAt: ts,
          updatedAt: ts,
          cards: [card],
        },
        {
          id: uuid,
          boardId: uuid,
          name: "Done",
          position: 1,
          createdAt: ts,
          updatedAt: ts,
          cards: [],
        },
      ],
    });
    expect(parsed.columns.map((c) => c.cards.length)).toEqual([1, 0]);
  });

  it("rejects a board-full whose nested card is malformed", () => {
    const result = boardFullSchema.safeParse({
      id: uuid,
      projectId: uuid,
      name: "B",
      position: 0,
      createdAt: ts,
      updatedAt: ts,
      columns: [
        {
          id: uuid,
          boardId: uuid,
          name: "To Do",
          position: 0,
          createdAt: ts,
          updatedAt: ts,
          cards: [{ ...card, position: "nope" }],
        },
      ],
    });
    expect(result.success).toBe(false);
  });

  it("parses a list of projects", () => {
    expect(projectListSchema.parse([project])).toHaveLength(1);
  });
});
