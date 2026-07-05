import type { ReactNode } from "react";
import type { DragEndEvent } from "@dnd-kit/core";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { http, HttpResponse } from "msw";
import { toast } from "sonner";
import { afterEach, describe, expect, it, vi } from "vitest";

import { reorderProjects } from "@/lib/api/projects";
import type { Project } from "@/lib/api/schemas";
import { queryKeys } from "@/lib/query/keys";
import { server } from "@/test/msw/server";

import { handleDragEnd, reorder, useReorder } from "./use-reorder";

vi.mock("sonner", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";

function project(id: string, name: string, position: number): Project {
  return { id, name, position, createdAt: ts, updatedAt: ts };
}

const a = project("a-id", "A", 0);
const b = project("b-id", "B", 1);
const c = project("c-id", "C", 2);

describe("reorder()", () => {
  it("moves an item down", () => {
    expect(reorder([a, b, c], "a-id", "c-id").map((p) => p.id)).toEqual([
      "b-id",
      "c-id",
      "a-id",
    ]);
  });

  it("moves an item up", () => {
    expect(reorder([a, b, c], "c-id", "a-id").map((p) => p.id)).toEqual([
      "c-id",
      "a-id",
      "b-id",
    ]);
  });

  it("no-ops when an id is missing", () => {
    expect(reorder([a, b, c], "a-id", "missing")).toEqual([a, b, c]);
  });
});

function dragEvent(activeId: string, overId: string | null): DragEndEvent {
  return {
    active: { id: activeId },
    over: overId === null ? null : { id: overId },
  } as unknown as DragEndEvent;
}

describe("handleDragEnd", () => {
  it("reorders when dropped on a different item", () => {
    const spy = vi.fn();
    handleDragEnd(dragEvent("a-id", "c-id"), spy);
    expect(spy).toHaveBeenCalledWith({ activeId: "a-id", overId: "c-id" });
  });

  it("ignores a drop on the same item", () => {
    const spy = vi.fn();
    handleDragEnd(dragEvent("a-id", "a-id"), spy);
    expect(spy).not.toHaveBeenCalled();
  });

  it("ignores a drop outside any item", () => {
    const spy = vi.fn();
    handleDragEnd(dragEvent("a-id", null), spy);
    expect(spy).not.toHaveBeenCalled();
  });
});

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

describe("useReorder", () => {
  afterEach(() => vi.clearAllMocks());

  it("persists the optimistic order and does not toast on success", async () => {
    const client = newClient();
    client.setQueryData(queryKeys.projects, [a, b, c]);
    let posted: unknown;
    server.use(
      http.put(`${BASE}/projects/reorder`, async ({ request }) => {
        posted = await request.json();
        return new HttpResponse(null, { status: 204 });
      }),
      http.get(`${BASE}/projects`, () => HttpResponse.json([b, c, a])),
    );

    const { result } = renderHook(
      () =>
        useReorder<Project>(queryKeys.projects, (orderedIds) =>
          reorderProjects({ orderedIds }),
        ),
      { wrapper: wrapper(client) },
    );

    act(() => result.current.mutate({ activeId: "a-id", overId: "c-id" }));
    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(posted).toEqual({ orderedIds: ["b-id", "c-id", "a-id"] });
    expect(vi.mocked(toast.error)).not.toHaveBeenCalled();
  });

  it("rolls back and toasts when the request fails", async () => {
    const client = newClient();
    client.setQueryData(queryKeys.projects, [a, b, c]);
    server.use(
      http.put(
        `${BASE}/projects/reorder`,
        () => new HttpResponse(null, { status: 500 }),
      ),
      http.get(`${BASE}/projects`, () => HttpResponse.json([a, b, c])),
    );

    const { result } = renderHook(
      () =>
        useReorder<Project>(queryKeys.projects, (orderedIds) =>
          reorderProjects({ orderedIds }),
        ),
      { wrapper: wrapper(client) },
    );

    act(() => result.current.mutate({ activeId: "a-id", overId: "c-id" }));
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
      "Couldn’t save the new order. Reverted.",
    );
    await waitFor(() =>
      expect(client.getQueryData(queryKeys.projects)).toEqual([a, b, c]),
    );
  });

  it("still toasts when there is no cached list to update", async () => {
    const client = newClient();
    server.use(
      http.put(
        `${BASE}/projects/reorder`,
        () => new HttpResponse(null, { status: 500 }),
      ),
    );

    const { result } = renderHook(
      () =>
        useReorder<Project>(queryKeys.projects, (orderedIds) =>
          reorderProjects({ orderedIds }),
        ),
      { wrapper: wrapper(client) },
    );

    act(() => result.current.mutate({ activeId: "a-id", overId: "c-id" }));
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(vi.mocked(toast.error)).toHaveBeenCalled();
  });
});
