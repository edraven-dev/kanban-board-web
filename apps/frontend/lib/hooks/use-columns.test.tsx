import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { http, HttpResponse } from "msw";
import { toast } from "sonner";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { BoardFull, ColumnFull } from "@/lib/api/schemas";
import { queryKeys } from "@/lib/query/keys";
import { server } from "@/test/msw/server";

import {
  useCreateColumn,
  useDeleteColumn,
  useRenameColumn,
  useReorderColumns,
} from "./use-columns";

vi.mock("sonner", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));

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
    name: "Board",
    position: 0,
    createdAt: ts,
    updatedAt: ts,
    columns,
  };
}

function newClient() {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
}

function wrapper(client: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    );
  };
}

const key = queryKeys.boardFull(boardId);

afterEach(() => vi.clearAllMocks());

describe("useCreateColumn", () => {
  it("toasts the 99-limit message on a 409", async () => {
    const client = newClient();
    server.use(
      http.post(`${BASE}/boards/${boardId}/columns`, () =>
        HttpResponse.json(
          { error: { code: "limit_exceeded", message: "too many" } },
          { status: 409 },
        ),
      ),
    );

    const { result } = renderHook(() => useCreateColumn(boardId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate("X"));
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
      "You’ve reached the limit of 99 columns for this board.",
    );
  });

  it("toasts a generic message on other errors", async () => {
    const client = newClient();
    server.use(
      http.post(
        `${BASE}/boards/${boardId}/columns`,
        () => new HttpResponse(null, { status: 500 }),
      ),
    );

    const { result } = renderHook(() => useCreateColumn(boardId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate("X"));
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
      "Couldn’t create the column.",
    );
  });
});

describe("useRenameColumn", () => {
  it("optimistically renames within the board-full cache", async () => {
    const client = newClient();
    client.setQueryData(key, boardFull([column(c1, "To Do", 0)]));
    server.use(
      http.patch(`${BASE}/columns/${c1}`, () =>
        HttpResponse.json({
          id: c1,
          boardId,
          name: "Doing",
          position: 0,
          createdAt: ts,
          updatedAt: ts,
        }),
      ),
    );

    const { result } = renderHook(() => useRenameColumn(boardId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate({ id: c1, name: "Doing" }));

    await waitFor(() =>
      expect(client.getQueryData<BoardFull>(key)?.columns[0].name).toBe(
        "Doing",
      ),
    );
  });
});

describe("useDeleteColumn", () => {
  it("optimistically removes from the board-full cache", async () => {
    const client = newClient();
    client.setQueryData(
      key,
      boardFull([column(c1, "To Do", 0), column(c2, "Done", 1)]),
    );
    server.use(
      http.delete(
        `${BASE}/columns/${c1}`,
        () => new HttpResponse(null, { status: 204 }),
      ),
    );

    const { result } = renderHook(() => useDeleteColumn(boardId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate(c1));

    await waitFor(() =>
      expect(
        client.getQueryData<BoardFull>(key)?.columns.map((c) => c.id),
      ).toEqual([c2]),
    );
  });
});

describe("useReorderColumns", () => {
  it("rolls back and toasts on API error", async () => {
    const client = newClient();
    const original = boardFull([column(c1, "To Do", 0), column(c2, "Done", 1)]);
    client.setQueryData(key, original);
    server.use(
      http.put(
        `${BASE}/boards/${boardId}/columns/reorder`,
        () => new HttpResponse(null, { status: 500 }),
      ),
    );

    const { result } = renderHook(() => useReorderColumns(boardId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate({ activeId: c1, overId: c2 }));
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
      "Couldn’t save the new order. Reverted.",
    );
    await waitFor(() =>
      expect(
        client.getQueryData<BoardFull>(key)?.columns.map((c) => c.id),
      ).toEqual([c1, c2]),
    );
  });
});
