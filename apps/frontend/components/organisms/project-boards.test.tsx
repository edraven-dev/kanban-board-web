import { delay, http, HttpResponse } from "msw";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import type { Board } from "@/lib/api/schemas";
import { server } from "@/test/msw/server";
import { renderWithClient } from "@/test/render";

import { ProjectBoards } from "./project-boards";

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";

/** Deterministic valid UUIDs (schemas enforce `z.uuid()`). */
function uid(n: number) {
  return `018f1e2d-3c4b-7a6d-8e9f-${String(n).padStart(12, "0")}`;
}

const projectId = uid(900);
const b1 = uid(1);
const b2 = uid(2);

function board(id: string, name: string, position: number): Board {
  return { id, projectId, name, position, createdAt: ts, updatedAt: ts };
}

// The header's ProjectSwitcher lists all projects.
function stubProjects() {
  return http.get(`${BASE}/projects`, () =>
    HttpResponse.json([
      { id: projectId, name: "Alpha", position: 0, createdAt: ts, updatedAt: ts },
    ]),
  );
}

describe("ProjectBoards", () => {
  it("renders the project's boards with open links", async () => {
    server.use(
      stubProjects(),
      http.get(`${BASE}/projects/${projectId}/boards`, () =>
        HttpResponse.json([
          board(b1, "Sprint 1", 0),
          board(b2, "Sprint 2", 1),
        ]),
      ),
    );

    renderWithClient(<ProjectBoards projectId={projectId} />);

    expect(
      await screen.findByRole("link", { name: "Open Sprint 1" }),
    ).toHaveAttribute("href", `/boards/${b1}`);
    expect(screen.getByRole("link", { name: "Open Sprint 2" })).toHaveAttribute(
      "href",
      `/boards/${b2}`,
    );
  });

  it("shows the empty state", async () => {
    server.use(
      stubProjects(),
      http.get(`${BASE}/projects/${projectId}/boards`, () =>
        HttpResponse.json([]),
      ),
    );

    renderWithClient(<ProjectBoards projectId={projectId} />);

    expect(await screen.findByText("No boards yet")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Add board" }),
    ).toBeInTheDocument();
  });

  it("shows a skeleton while loading", () => {
    server.use(
      stubProjects(),
      http.get(`${BASE}/projects/${projectId}/boards`, async () => {
        await delay("infinite");
        return HttpResponse.json([]);
      }),
    );

    renderWithClient(<ProjectBoards projectId={projectId} />);

    expect(screen.getByTestId("boards-skeleton")).toBeInTheDocument();
  });

  it("shows an error state when the request fails", async () => {
    server.use(
      stubProjects(),
      http.get(
        `${BASE}/projects/${projectId}/boards`,
        () => new HttpResponse(null, { status: 500 }),
      ),
    );

    renderWithClient(<ProjectBoards projectId={projectId} />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Couldn’t load boards",
    );
  });

  it("creates a board via InlineCreate and refreshes the list", async () => {
    const user = userEvent.setup();
    const state = [board(b1, "Sprint 1", 0)];
    let posted: unknown;
    server.use(
      stubProjects(),
      http.get(`${BASE}/projects/${projectId}/boards`, () =>
        HttpResponse.json(state),
      ),
      http.post(`${BASE}/projects/${projectId}/boards`, async ({ request }) => {
        posted = await request.json();
        const created = board(b2, "Sprint 2", 1);
        state.push(created);
        return HttpResponse.json(created, { status: 201 });
      }),
    );

    renderWithClient(<ProjectBoards projectId={projectId} />);
    await screen.findByRole("link", { name: "Open Sprint 1" });

    await user.click(screen.getByRole("button", { name: "Add board" }));
    await user.type(
      screen.getByRole("textbox", { name: "New board name" }),
      "Sprint 2{Enter}",
    );

    expect(
      await screen.findByRole("link", { name: "Open Sprint 2" }),
    ).toBeInTheDocument();
    expect(posted).toEqual({ name: "Sprint 2" });
  });

  it("renames a board from the tile menu", async () => {
    const user = userEvent.setup();
    const state = [board(b1, "Sprint 1", 0)];
    let patched: unknown;
    server.use(
      stubProjects(),
      http.get(`${BASE}/projects/${projectId}/boards`, () =>
        HttpResponse.json(state),
      ),
      http.patch(`${BASE}/boards/${b1}`, async ({ request }) => {
        patched = await request.json();
        state[0] = { ...state[0], name: "Renamed" };
        return HttpResponse.json(state[0]);
      }),
    );

    renderWithClient(<ProjectBoards projectId={projectId} />);

    await user.click(
      await screen.findByRole("button", { name: "Sprint 1 actions" }),
    );
    await user.click(await screen.findByRole("menuitem", { name: "Rename" }));
    const input = screen.getByRole("textbox", { name: "Board name" });
    await user.clear(input);
    await user.type(input, "Renamed{Enter}");

    expect(
      await screen.findByRole("link", { name: "Open Renamed" }),
    ).toBeInTheDocument();
    expect(patched).toEqual({ name: "Renamed" });
  });

  it("deletes a board after confirmation", async () => {
    const user = userEvent.setup();
    const state = [board(b1, "Sprint 1", 0), board(b2, "Sprint 2", 1)];
    let deletedId: string | undefined;
    server.use(
      stubProjects(),
      http.get(`${BASE}/projects/${projectId}/boards`, () =>
        HttpResponse.json(state),
      ),
      http.delete(`${BASE}/boards/:id`, ({ params }) => {
        deletedId = params.id as string;
        const index = state.findIndex((x) => x.id === deletedId);
        if (index !== -1) state.splice(index, 1);
        return new HttpResponse(null, { status: 204 });
      }),
    );

    renderWithClient(<ProjectBoards projectId={projectId} />);

    await user.click(
      await screen.findByRole("button", { name: "Sprint 1 actions" }),
    );
    await user.click(await screen.findByRole("menuitem", { name: "Delete" }));
    await user.click(await screen.findByRole("button", { name: "Delete" }));

    await waitFor(() =>
      expect(
        screen.queryByRole("link", { name: "Open Sprint 1" }),
      ).not.toBeInTheDocument(),
    );
    expect(deletedId).toBe(b1);
    expect(
      screen.getByRole("link", { name: "Open Sprint 2" }),
    ).toBeInTheDocument();
  });

  it("hides the create control at the 99-board limit", async () => {
    const many = Array.from({ length: 99 }, (_, index) =>
      board(uid(index), `Board ${index}`, index),
    );
    server.use(
      stubProjects(),
      http.get(`${BASE}/projects/${projectId}/boards`, () =>
        HttpResponse.json(many),
      ),
    );

    renderWithClient(<ProjectBoards projectId={projectId} />);
    await screen.findByRole("link", { name: "Open Board 0" });

    expect(
      screen.queryByRole("button", { name: "Add board" }),
    ).not.toBeInTheDocument();
  });
});
