import { http, HttpResponse } from "msw";
import { describe, expect, it } from "vitest";

import { server } from "@/test/msw/server";

import {
  createCard,
  deleteCard,
  getCard,
  listCards,
  moveCard,
  updateCard,
} from "./cards";

const BASE = "http://localhost:5000/api";
const uuid = "018f1e2d-3c4b-7a6d-8e9f-0123456789ab";
const ts = "2026-07-05T00:00:00Z";
const card = {
  id: uuid,
  columnId: uuid,
  title: "T",
  description: "",
  position: 0,
  createdAt: ts,
  updatedAt: ts,
};

describe("cards api", () => {
  it("lists cards for a column", async () => {
    server.use(
      http.get(`${BASE}/columns/${uuid}/cards`, () => HttpResponse.json([card])),
    );
    expect(await listCards(uuid)).toEqual([card]);
  });

  it("gets a single card", async () => {
    server.use(http.get(`${BASE}/cards/${uuid}`, () => HttpResponse.json(card)));
    expect((await getCard(uuid)).title).toBe("T");
  });

  it("creates a card with an optional description", async () => {
    let received: unknown;
    server.use(
      http.post(`${BASE}/columns/${uuid}/cards`, async ({ request }) => {
        received = await request.json();
        return HttpResponse.json(
          { ...card, title: "New", description: "body" },
          { status: 201 },
        );
      }),
    );
    const result = await createCard(uuid, { title: "New", description: "body" });
    expect(received).toEqual({ title: "New", description: "body" });
    expect(result.description).toBe("body");
  });

  it("updates a card", async () => {
    server.use(
      http.patch(`${BASE}/cards/${uuid}`, () =>
        HttpResponse.json({ ...card, title: "Edited" }),
      ),
    );
    expect((await updateCard(uuid, { title: "Edited" })).title).toBe("Edited");
  });

  it("deletes a card", async () => {
    server.use(
      http.delete(
        `${BASE}/cards/${uuid}`,
        () => new HttpResponse(null, { status: 204 }),
      ),
    );
    await expect(deleteCard(uuid)).resolves.toBeUndefined();
  });

  it("moves a card with a columnId and position body", async () => {
    let received: unknown;
    server.use(
      http.put(`${BASE}/cards/${uuid}/move`, async ({ request }) => {
        received = await request.json();
        return new HttpResponse(null, { status: 204 });
      }),
    );
    await moveCard(uuid, { columnId: uuid, position: 2 });
    expect(received).toEqual({ columnId: uuid, position: 2 });
  });
});
