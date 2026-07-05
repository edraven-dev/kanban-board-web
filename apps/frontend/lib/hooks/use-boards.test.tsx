import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { http, HttpResponse } from "msw";
import { toast } from "sonner";
import { afterEach, describe, expect, it, vi } from "vitest";

import { reorderBoards } from "@/lib/api/boards";
import type { Board } from "@/lib/api/schemas";
import { queryKeys } from "@/lib/query/keys";
import { server } from "@/test/msw/server";

import { useCreateBoard, useDeleteBoard, useRenameBoard } from "./use-boards";
import { useReorder } from "./use-reorder";

vi.mock("sonner", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";

/** Deterministic valid UUIDs (schemas enforce `z.uuid()`). */
function uid(n: number) {
  return `018f1e2d-3c4b-7a6d-8e9f-${String(n).padStart(12, "0")}`;
}

const projectId = uid(900);
const aId = uid(1);
const bId = uid(2);

function board(id: string, name: string, position: number): Board {
  return { id, projectId, name, position, createdAt: ts, updatedAt: ts };
}

const a = board(aId, "A", 0);
const b = board(bId, "B", 1);

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

afterEach(() => vi.clearAllMocks());

describe("useCreateBoard", () => {
  it("creates a board without toasting on success", async () => {
    const client = newClient();
    let posted: unknown;
    server.use(
      http.post(`${BASE}/projects/${projectId}/boards`, async ({ request }) => {
        posted = await request.json();
        return HttpResponse.json(board(uid(10), "New", 0), { status: 201 });
      }),
    );

    const { result } = renderHook(() => useCreateBoard(projectId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate("New"));
    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(posted).toEqual({ name: "New" });
    expect(vi.mocked(toast.error)).not.toHaveBeenCalled();
  });

  it("toasts the 99-limit message on a 409", async () => {
    const client = newClient();
    server.use(
      http.post(`${BASE}/projects/${projectId}/boards`, () =>
        HttpResponse.json(
          { error: { code: "limit_exceeded", message: "too many" } },
          { status: 409 },
        ),
      ),
    );

    const { result } = renderHook(() => useCreateBoard(projectId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate("X"));
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
      "You’ve reached the limit of 99 boards for this project.",
    );
  });

  it("toasts a generic message on other errors", async () => {
    const client = newClient();
    server.use(
      http.post(
        `${BASE}/projects/${projectId}/boards`,
        () => new HttpResponse(null, { status: 500 }),
      ),
    );

    const { result } = renderHook(() => useCreateBoard(projectId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate("X"));
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
      "Couldn’t create the board.",
    );
  });
});

describe("useRenameBoard", () => {
  it("optimistically renames in the cache", async () => {
    const client = newClient();
    client.setQueryData(queryKeys.boardsByProject(projectId), [a, b]);
    server.use(
      http.patch(`${BASE}/boards/${aId}`, () =>
        HttpResponse.json({ ...a, name: "Renamed" }),
      ),
    );

    const { result } = renderHook(() => useRenameBoard(projectId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate({ id: aId, name: "Renamed" }));

    await waitFor(() =>
      expect(
        client.getQueryData<Board[]>(queryKeys.boardsByProject(projectId))?.[0]
          .name,
      ).toBe("Renamed"),
    );
  });
});

describe("useDeleteBoard", () => {
  it("optimistically removes from the cache", async () => {
    const client = newClient();
    client.setQueryData(queryKeys.boardsByProject(projectId), [a, b]);
    server.use(
      http.delete(
        `${BASE}/boards/${aId}`,
        () => new HttpResponse(null, { status: 204 }),
      ),
    );

    const { result } = renderHook(() => useDeleteBoard(projectId), {
      wrapper: wrapper(client),
    });
    act(() => result.current.mutate(aId));

    await waitFor(() =>
      expect(
        client
          .getQueryData<Board[]>(queryKeys.boardsByProject(projectId))
          ?.map((x) => x.id),
      ).toEqual([bId]),
    );
  });
});

describe("board reorder", () => {
  it("rolls back and toasts on API error", async () => {
    const client = newClient();
    client.setQueryData(queryKeys.boardsByProject(projectId), [a, b]);
    server.use(
      http.put(
        `${BASE}/projects/${projectId}/boards/reorder`,
        () => new HttpResponse(null, { status: 500 }),
      ),
    );

    const { result } = renderHook(
      () =>
        useReorder<Board>(queryKeys.boardsByProject(projectId), (orderedIds) =>
          reorderBoards(projectId, { orderedIds }),
        ),
      { wrapper: wrapper(client) },
    );
    act(() => result.current.mutate({ activeId: aId, overId: bId }));
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
      "Couldn’t save the new order. Reverted.",
    );
    await waitFor(() =>
      expect(
        client.getQueryData(queryKeys.boardsByProject(projectId)),
      ).toEqual([a, b]),
    );
  });
});
