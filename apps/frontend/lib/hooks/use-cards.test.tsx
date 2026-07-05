import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { http, HttpResponse } from "msw";
import { toast } from "sonner";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { BoardFull, Card, ColumnFull } from "@/lib/api/schemas";
import { queryKeys } from "@/lib/query/keys";
import { server } from "@/test/msw/server";

import {
  findCard,
  moveCardInBoard,
  resolveCardDrop,
  useCreateCard,
  useDeleteCard,
  useMoveCard,
  useUpdateCard,
} from "./use-cards";

vi.mock("sonner", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";

function uid(n: number) {
  return `018f1e2d-3c4b-7a6d-8e9f-${String(n).padStart(12, "0")}`;
}

const boardId = uid(500);
const projectId = uid(900);
const col1 = uid(11);
const col2 = uid(12);
const cardA = uid(1);
const cardB = uid(2);
const cardC = uid(3);

function card(id: string, columnId: string, title: string, position: number): Card {
  return { id, columnId, title, description: "", position, createdAt: ts, updatedAt: ts };
}

function column(id: string, position: number, cards: Card[]): ColumnFull {
  return { id, boardId, name: `Col ${position}`, position, createdAt: ts, updatedAt: ts, cards };
}

function boardFull(): BoardFull {
  return {
    id: boardId,
    projectId,
    name: "Board",
    position: 0,
    createdAt: ts,
    updatedAt: ts,
    columns: [
      column(col1, 0, [card(cardA, col1, "A", 0), card(cardB, col1, "B", 1)]),
      column(col2, 1, [card(cardC, col2, "C", 0)]),
    ],
  };
}

const key = queryKeys.boardFull(boardId);

function newClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
}

function wrapper(client: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    );
  };
}

afterEach(() => vi.clearAllMocks());

describe("moveCardInBoard", () => {
  it("reorders within a column", () => {
    const next = moveCardInBoard(boardFull(), cardA, col1, 1);
    expect(next.columns[0].cards.map((c) => c.id)).toEqual([cardB, cardA]);
  });

  it("moves a card across columns and updates its columnId", () => {
    const next = moveCardInBoard(boardFull(), cardA, col2, 0);
    expect(next.columns[0].cards.map((c) => c.id)).toEqual([cardB]);
    expect(next.columns[1].cards.map((c) => c.id)).toEqual([cardA, cardC]);
    expect(next.columns[1].cards[0].columnId).toBe(col2);
  });

  it("no-ops for an unknown card", () => {
    const board = boardFull();
    expect(moveCardInBoard(board, "missing", col2, 0)).toBe(board);
  });
});

describe("findCard", () => {
  it("returns the card by id", () => {
    expect(findCard(boardFull(), cardA)?.title).toBe("A");
  });

  it("returns undefined for an unknown card", () => {
    expect(findCard(boardFull(), "missing")).toBeUndefined();
  });
});

describe("resolveCardDrop", () => {
  const board = boardFull();

  it("targets the over-card's slot across columns", () => {
    expect(
      resolveCardDrop(board, cardA, { id: cardC, type: "card", columnId: col2 }),
    ).toEqual({ cardId: cardA, toColumnId: col2, toIndex: 0 });
  });

  it("reorders within the same column", () => {
    expect(
      resolveCardDrop(board, cardA, { id: cardB, type: "card", columnId: col1 }),
    ).toEqual({ cardId: cardA, toColumnId: col1, toIndex: 1 });
  });

  it("appends when dropped on a column", () => {
    expect(resolveCardDrop(board, cardA, { id: col2, type: "column" })).toEqual({
      cardId: cardA,
      toColumnId: col2,
      toIndex: 1,
    });
  });

  it("is null for no drop target", () => {
    expect(resolveCardDrop(board, cardA, null)).toBeNull();
  });

  it("is null for an unknown column", () => {
    expect(resolveCardDrop(board, cardA, { id: "nope", type: "column" })).toBeNull();
    expect(
      resolveCardDrop(board, cardA, { id: cardC, type: "card", columnId: "nope" }),
    ).toBeNull();
  });

  it("is null when dropped on itself (no move)", () => {
    expect(
      resolveCardDrop(board, cardA, { id: cardA, type: "card", columnId: col1 }),
    ).toBeNull();
  });

  it("is null for an unknown active card", () => {
    expect(
      resolveCardDrop(board, "missing", { id: cardC, type: "card", columnId: col2 }),
    ).toBeNull();
  });
});

describe("useMoveCard", () => {
  it("optimistically moves across columns and persists", async () => {
    const client = newClient();
    client.setQueryData(key, boardFull());
    let posted: unknown;
    server.use(
      http.put(`${BASE}/cards/${cardA}/move`, async ({ request }) => {
        posted = await request.json();
        return new HttpResponse(null, { status: 204 });
      }),
    );

    const { result } = renderHook(() => useMoveCard(boardId), {
      wrapper: wrapper(client),
    });
    act(() =>
      result.current.mutate({ cardId: cardA, toColumnId: col2, toIndex: 0 }),
    );
    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const board = client.getQueryData<BoardFull>(key)!;
    expect(board.columns[0].cards.map((c) => c.id)).toEqual([cardB]);
    expect(board.columns[1].cards.map((c) => c.id)).toEqual([cardA, cardC]);
    expect(posted).toEqual({ columnId: col2, position: 0 });
    expect(vi.mocked(toast.error)).not.toHaveBeenCalled();
  });

  it("rolls back and toasts on API error", async () => {
    const client = newClient();
    client.setQueryData(key, boardFull());
    server.use(
      http.put(
        `${BASE}/cards/${cardA}/move`,
        () => new HttpResponse(null, { status: 500 }),
      ),
    );

    const { result } = renderHook(() => useMoveCard(boardId), {
      wrapper: wrapper(client),
    });
    act(() =>
      result.current.mutate({ cardId: cardA, toColumnId: col2, toIndex: 0 }),
    );
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
      "Couldn’t move the card. Reverted.",
    );
    await waitFor(() =>
      expect(
        client.getQueryData<BoardFull>(key)?.columns[0].cards.map((c) => c.id),
      ).toEqual([cardA, cardB]),
    );
  });
});

describe("useUpdateCard", () => {
  it("optimistically edits a card without touching createdAt", async () => {
    const client = newClient();
    client.setQueryData(key, boardFull());
    let patched: unknown;
    server.use(
      http.patch(`${BASE}/cards/${cardA}`, async ({ request }) => {
        patched = await request.json();
        return HttpResponse.json(card(cardA, col1, "Renamed", 0));
      }),
    );

    const { result } = renderHook(() => useUpdateCard(boardId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate({ id: cardA, title: "Renamed" }));

    await waitFor(() =>
      expect(
        client.getQueryData<BoardFull>(key)?.columns[0].cards[0].title,
      ).toBe("Renamed"),
    );
    expect(client.getQueryData<BoardFull>(key)?.columns[0].cards[0].createdAt).toBe(ts);
    expect(patched).toEqual({ title: "Renamed" });
  });

  it("optimistically edits the description", async () => {
    const client = newClient();
    client.setQueryData(key, boardFull());
    let patched: unknown;
    server.use(
      http.patch(`${BASE}/cards/${cardA}`, async ({ request }) => {
        patched = await request.json();
        return HttpResponse.json(card(cardA, col1, "A", 0));
      }),
    );

    const { result } = renderHook(() => useUpdateCard(boardId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate({ id: cardA, description: "Details" }));

    await waitFor(() =>
      expect(
        client.getQueryData<BoardFull>(key)?.columns[0].cards[0].description,
      ).toBe("Details"),
    );
    expect(patched).toEqual({ description: "Details" });
  });
});

describe("useDeleteCard", () => {
  it("optimistically removes a card", async () => {
    const client = newClient();
    client.setQueryData(key, boardFull());
    server.use(
      http.delete(
        `${BASE}/cards/${cardA}`,
        () => new HttpResponse(null, { status: 204 }),
      ),
    );

    const { result } = renderHook(() => useDeleteCard(boardId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate(cardA));

    await waitFor(() =>
      expect(
        client.getQueryData<BoardFull>(key)?.columns[0].cards.map((c) => c.id),
      ).toEqual([cardB]),
    );
  });
});

describe("useCreateCard", () => {
  it("creates a card without toasting on success", async () => {
    const client = newClient();
    let posted: unknown;
    server.use(
      http.post(`${BASE}/columns/${col1}/cards`, async ({ request }) => {
        posted = await request.json();
        return HttpResponse.json(card(uid(9), col1, "New", 2), { status: 201 });
      }),
    );

    const { result } = renderHook(() => useCreateCard(boardId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate({ columnId: col1, title: "New" }));
    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(posted).toEqual({ title: "New" });
    expect(vi.mocked(toast.error)).not.toHaveBeenCalled();
  });
});
