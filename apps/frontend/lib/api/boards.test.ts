import { http, HttpResponse } from "msw";
import { describe, expect, it } from "vitest";

import { server } from "@/test/msw/server";

import {
  createBoard,
  deleteBoard,
  getBoardFull,
  listBoards,
  reorderBoards,
  updateBoard,
} from "./boards";

const BASE = "http://localhost:5000/api";
const uuid = "018f1e2d-3c4b-7a6d-8e9f-0123456789ab";
const ts = "2026-07-05T00:00:00Z";
const board = { id: uuid, projectId: uuid, name: "B", position: 0, createdAt: ts, updatedAt: ts };

describe("boards api", () => {
  it("lists boards for a project", async () => {
    server.use(
      http.get(`${BASE}/projects/${uuid}/boards`, () =>
        HttpResponse.json([board]),
      ),
    );
    expect(await listBoards(uuid)).toEqual([board]);
  });

  it("creates a board under a project", async () => {
    server.use(
      http.post(`${BASE}/projects/${uuid}/boards`, () =>
        HttpResponse.json({ ...board, name: "New" }, { status: 201 }),
      ),
    );
    expect((await createBoard(uuid, { name: "New" })).name).toBe("New");
  });

  it("fetches the full nested board read model", async () => {
    server.use(
      http.get(`${BASE}/boards/${uuid}/full`, () =>
        HttpResponse.json({
          ...board,
          columns: [
            {
              id: uuid,
              boardId: uuid,
              name: "To Do",
              position: 0,
              createdAt: ts,
              updatedAt: ts,
              cards: [
                {
                  id: uuid,
                  columnId: uuid,
                  title: "T",
                  description: "",
                  position: 0,
                  createdAt: ts,
                  updatedAt: ts,
                },
              ],
            },
          ],
        }),
      ),
    );
    const full = await getBoardFull(uuid);
    expect(full.columns.map((c) => c.cards.length)).toEqual([1]);
  });

  it("updates a board", async () => {
    server.use(
      http.patch(`${BASE}/boards/${uuid}`, () =>
        HttpResponse.json({ ...board, name: "R" }),
      ),
    );
    expect((await updateBoard(uuid, { name: "R" })).name).toBe("R");
  });

  it("deletes a board", async () => {
    server.use(
      http.delete(
        `${BASE}/boards/${uuid}`,
        () => new HttpResponse(null, { status: 204 }),
      ),
    );
    await expect(deleteBoard(uuid)).resolves.toBeUndefined();
  });

  it("reorders boards within a project", async () => {
    let received: unknown;
    server.use(
      http.put(`${BASE}/projects/${uuid}/boards/reorder`, async ({ request }) => {
        received = await request.json();
        return new HttpResponse(null, { status: 204 });
      }),
    );
    await reorderBoards(uuid, { orderedIds: [uuid] });
    expect(received).toEqual({ orderedIds: [uuid] });
  });
});
