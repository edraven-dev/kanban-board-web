import { delay, http, HttpResponse } from "msw";
import { fireEvent, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import type { BoardFull, Card, ColumnFull } from "@/lib/api/schemas";
import { server } from "@/test/msw/server";
import { renderWithClient } from "@/test/render";

import { BoardCanvas } from "./board-canvas";

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";

function uid(n: number) {
  return `018f1e2d-3c4b-7a6d-8e9f-${String(n).padStart(12, "0")}`;
}

const boardId = uid(500);
const projectId = uid(900);
const c1 = uid(1);
const c2 = uid(2);

function column(id: string, name: string, position: number): ColumnFull {
  return { id, boardId, name, position, createdAt: ts, updatedAt: ts, cards: [] };
}

function boardFull(columns: ColumnFull[]): BoardFull {
  return {
    id: boardId,
    projectId,
    name: "My Board",
    position: 0,
    createdAt: ts,
    updatedAt: ts,
    columns,
  };
}

function plainColumn(id: string, name: string, position: number) {
  return { id, boardId, name, position, createdAt: ts, updatedAt: ts };
}

function card(id: string, columnId: string, title: string, position: number): Card {
  return { id, columnId, title, description: "", position, createdAt: ts, updatedAt: ts };
}

describe("BoardCanvas", () => {
  it("renders the board's columns", async () => {
    server.use(
      http.get(`${BASE}/boards/${boardId}/full`, () =>
        HttpResponse.json(
          boardFull([column(c1, "To Do", 0), column(c2, "Done", 1)]),
        ),
      ),
    );

    renderWithClient(<BoardCanvas boardId={boardId} />);

    expect(await screen.findByRole("button", { name: "To Do" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Done" })).toBeInTheDocument();
  });

  it("shows a skeleton while loading", () => {
    server.use(
      http.get(`${BASE}/boards/${boardId}/full`, async () => {
        await delay("infinite");
        return HttpResponse.json(boardFull([]));
      }),
    );

    renderWithClient(<BoardCanvas boardId={boardId} />);

    expect(screen.getByTestId("board-skeleton")).toBeInTheDocument();
  });

  it("shows an error state when the request fails", async () => {
    server.use(
      http.get(
        `${BASE}/boards/${boardId}/full`,
        () => new HttpResponse(null, { status: 500 }),
      ),
    );

    renderWithClient(<BoardCanvas boardId={boardId} />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Couldn’t load the board",
    );
  });

  it("creates a column via InlineCreate and refreshes", async () => {
    const user = userEvent.setup();
    const cols = [column(c1, "To Do", 0)];
    let posted: unknown;
    server.use(
      http.get(`${BASE}/boards/${boardId}/full`, () =>
        HttpResponse.json(boardFull(cols)),
      ),
      http.post(`${BASE}/boards/${boardId}/columns`, async ({ request }) => {
        posted = await request.json();
        cols.push(column(c2, "Doing", 1));
        return HttpResponse.json(plainColumn(c2, "Doing", 1), { status: 201 });
      }),
    );

    renderWithClient(<BoardCanvas boardId={boardId} />);
    await screen.findByRole("button", { name: "To Do" });

    await user.click(screen.getByRole("button", { name: "Add column" }));
    await user.type(
      screen.getByRole("textbox", { name: "New column name" }),
      "Doing{Enter}",
    );

    expect(
      await screen.findByRole("button", { name: "Doing" }),
    ).toBeInTheDocument();
    expect(posted).toEqual({ name: "Doing" });
  });

  it("renames a column inline", async () => {
    const user = userEvent.setup();
    const cols = [column(c1, "To Do", 0)];
    let patched: unknown;
    server.use(
      http.get(`${BASE}/boards/${boardId}/full`, () =>
        HttpResponse.json(boardFull(cols)),
      ),
      http.patch(`${BASE}/columns/${c1}`, async ({ request }) => {
        patched = await request.json();
        cols[0] = { ...cols[0], name: "Doing" };
        return HttpResponse.json(plainColumn(c1, "Doing", 0));
      }),
    );

    renderWithClient(<BoardCanvas boardId={boardId} />);

    await user.click(await screen.findByRole("button", { name: "To Do" }));
    const input = screen.getByRole("textbox", { name: "Column name" });
    await user.clear(input);
    await user.type(input, "Doing{Enter}");

    expect(
      await screen.findByRole("button", { name: "Doing" }),
    ).toBeInTheDocument();
    expect(patched).toEqual({ name: "Doing" });
  });

  it("deletes a column after confirmation", async () => {
    const user = userEvent.setup();
    const cols = [column(c1, "To Do", 0), column(c2, "Done", 1)];
    let deletedId: string | undefined;
    server.use(
      http.get(`${BASE}/boards/${boardId}/full`, () =>
        HttpResponse.json(boardFull(cols)),
      ),
      http.delete(`${BASE}/columns/:id`, ({ params }) => {
        deletedId = params.id as string;
        const index = cols.findIndex((c) => c.id === deletedId);
        if (index !== -1) cols.splice(index, 1);
        return new HttpResponse(null, { status: 204 });
      }),
    );

    renderWithClient(<BoardCanvas boardId={boardId} />);

    await user.click(await screen.findByRole("button", { name: "Delete To Do" }));
    await user.click(await screen.findByRole("button", { name: "Delete" }));

    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: "To Do" }),
      ).not.toBeInTheDocument(),
    );
    expect(deletedId).toBe(c1);
    expect(screen.getByRole("button", { name: "Done" })).toBeInTheDocument();
  });

  it("scrolls the board horizontally on a vertical wheel", async () => {
    server.use(
      http.get(`${BASE}/boards/${boardId}/full`, () =>
        HttpResponse.json(boardFull([column(c1, "To Do", 0)])),
      ),
    );

    renderWithClient(<BoardCanvas boardId={boardId} />);
    await screen.findByRole("button", { name: "To Do" });

    const scroll = screen.getByTestId("board-scroll");
    fireEvent.wheel(scroll, { deltaY: 0 });
    expect(scroll.scrollLeft).toBe(0);
    fireEvent.wheel(scroll, { deltaY: 120 });
    expect(scroll.scrollLeft).toBe(120);
  });

  it("hides the create control at the 99-column limit", async () => {
    const many = Array.from({ length: 99 }, (_, index) =>
      column(uid(index), `Col ${index}`, index),
    );
    server.use(
      http.get(`${BASE}/boards/${boardId}/full`, () =>
        HttpResponse.json(boardFull(many)),
      ),
    );

    renderWithClient(<BoardCanvas boardId={boardId} />);
    await screen.findByRole("button", { name: "Col 0" });

    expect(
      screen.queryByRole("button", { name: "Add column" }),
    ).not.toBeInTheDocument();
  });

  it("creates a card in a column", async () => {
    const user = userEvent.setup();
    const cards: Card[] = [];
    let posted: unknown;
    server.use(
      http.get(`${BASE}/boards/${boardId}/full`, () =>
        HttpResponse.json(boardFull([{ ...column(c1, "To Do", 0), cards }])),
      ),
      http.post(`${BASE}/columns/${c1}/cards`, async ({ request }) => {
        posted = await request.json();
        const created = card(uid(31), c1, "New Task", cards.length);
        cards.push(created);
        return HttpResponse.json(created, { status: 201 });
      }),
    );

    renderWithClient(<BoardCanvas boardId={boardId} />);
    await screen.findByRole("button", { name: "To Do" });

    await user.click(screen.getByRole("button", { name: "Add card" }));
    await user.type(
      screen.getByRole("textbox", { name: "New card name" }),
      "New Task{Enter}",
    );

    expect(
      await screen.findByRole("button", { name: "New Task" }),
    ).toBeInTheDocument();
    expect(posted).toEqual({ title: "New Task" });
  });

  it("opens the card modal when a card is clicked", async () => {
    const user = userEvent.setup();
    server.use(
      http.get(`${BASE}/boards/${boardId}/full`, () =>
        HttpResponse.json(
          boardFull([
            { ...column(c1, "To Do", 0), cards: [card(uid(31), c1, "Task", 0)] },
          ]),
        ),
      ),
    );

    renderWithClient(<BoardCanvas boardId={boardId} />);
    await user.click(await screen.findByRole("button", { name: "Task" }));

    expect(
      await screen.findByRole("button", { name: "Delete card" }),
    ).toBeInTheDocument();
  });

  it("opens the card modal via keyboard", async () => {
    server.use(
      http.get(`${BASE}/boards/${boardId}/full`, () =>
        HttpResponse.json(
          boardFull([
            { ...column(c1, "To Do", 0), cards: [card(uid(31), c1, "Task", 0)] },
          ]),
        ),
      ),
    );

    renderWithClient(<BoardCanvas boardId={boardId} />);
    const cardEl = await screen.findByRole("button", { name: "Task" });

    fireEvent.keyDown(cardEl, { key: "a" });
    expect(
      screen.queryByRole("button", { name: "Delete card" }),
    ).not.toBeInTheDocument();

    fireEvent.keyDown(cardEl, { key: " " });
    expect(
      await screen.findByRole("button", { name: "Delete card" }),
    ).toBeInTheDocument();
  });
});
