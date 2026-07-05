import { http, HttpResponse } from "msw";
import { describe, expect, it } from "vitest";

import { server } from "@/test/msw/server";

import {
  createColumn,
  deleteColumn,
  listColumns,
  reorderColumns,
  updateColumn,
} from "./columns";

const BASE = "http://localhost:5000/api";
const uuid = "018f1e2d-3c4b-7a6d-8e9f-0123456789ab";
const ts = "2026-07-05T00:00:00Z";
const column = { id: uuid, boardId: uuid, name: "To Do", position: 0, createdAt: ts, updatedAt: ts };

describe("columns api", () => {
  it("lists columns for a board", async () => {
    server.use(
      http.get(`${BASE}/boards/${uuid}/columns`, () =>
        HttpResponse.json([column]),
      ),
    );
    expect(await listColumns(uuid)).toEqual([column]);
  });

  it("creates a column under a board", async () => {
    server.use(
      http.post(`${BASE}/boards/${uuid}/columns`, () =>
        HttpResponse.json({ ...column, name: "Doing" }, { status: 201 }),
      ),
    );
    expect((await createColumn(uuid, { name: "Doing" })).name).toBe("Doing");
  });

  it("updates a column", async () => {
    server.use(
      http.patch(`${BASE}/columns/${uuid}`, () =>
        HttpResponse.json({ ...column, name: "Done" }),
      ),
    );
    expect((await updateColumn(uuid, { name: "Done" })).name).toBe("Done");
  });

  it("deletes a column", async () => {
    server.use(
      http.delete(
        `${BASE}/columns/${uuid}`,
        () => new HttpResponse(null, { status: 204 }),
      ),
    );
    await expect(deleteColumn(uuid)).resolves.toBeUndefined();
  });

  it("reorders columns within a board", async () => {
    let received: unknown;
    server.use(
      http.put(`${BASE}/boards/${uuid}/columns/reorder`, async ({ request }) => {
        received = await request.json();
        return new HttpResponse(null, { status: 204 });
      }),
    );
    await reorderColumns(uuid, { orderedIds: [uuid] });
    expect(received).toEqual({ orderedIds: [uuid] });
  });
});
