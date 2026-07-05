import { http, HttpResponse, delay } from "msw";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import type { Project } from "@/lib/api/schemas";
import { server } from "@/test/msw/server";
import { renderWithClient } from "@/test/render";

import Home from "./page";

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";

function project(id: string, name: string, position: number): Project {
  return { id, name, position, createdAt: ts, updatedAt: ts };
}

const alphaId = "018f1e2d-3c4b-7a6d-8e9f-000000000001";
const betaId = "018f1e2d-3c4b-7a6d-8e9f-000000000002";

describe("Home (projects feature)", () => {
  it("renders the project tiles from the API", async () => {
    server.use(
      http.get(`${BASE}/projects`, () =>
        HttpResponse.json([
          project(alphaId, "Alpha", 0),
          project(betaId, "Beta", 1),
        ]),
      ),
    );

    renderWithClient(<Home />);

    expect(
      await screen.findByRole("link", { name: "Open Alpha" }),
    ).toHaveAttribute("href", `/projects/${alphaId}`);
    expect(screen.getByRole("link", { name: "Open Beta" })).toHaveAttribute(
      "href",
      `/projects/${betaId}`,
    );
    // Each tile exposes a keyboard-accessible reorder handle.
    expect(
      screen.getByRole("button", { name: "Reorder Alpha" }),
    ).toBeInTheDocument();
  });

  it("shows the empty state when there are no projects", async () => {
    server.use(http.get(`${BASE}/projects`, () => HttpResponse.json([])));

    renderWithClient(<Home />);

    expect(await screen.findByText("No projects yet")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Add project" }),
    ).toBeInTheDocument();
  });

  it("shows a skeleton while loading", () => {
    server.use(
      http.get(`${BASE}/projects`, async () => {
        await delay("infinite");
        return HttpResponse.json([]);
      }),
    );

    renderWithClient(<Home />);

    expect(screen.getByTestId("projects-skeleton")).toBeInTheDocument();
  });

  it("shows an error state when the request fails", async () => {
    server.use(
      http.get(
        `${BASE}/projects`,
        () => new HttpResponse(null, { status: 500 }),
      ),
    );

    renderWithClient(<Home />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Couldn’t load projects",
    );
  });

  it("creates a project via InlineCreate and refreshes the list", async () => {
    const user = userEvent.setup();
    const state = [project(alphaId, "Alpha", 0)];
    let posted: unknown;
    server.use(
      http.get(`${BASE}/projects`, () => HttpResponse.json(state)),
      http.post(`${BASE}/projects`, async ({ request }) => {
        posted = await request.json();
        const created = project(betaId, "Beta", state.length);
        state.push(created);
        return HttpResponse.json(created, { status: 201 });
      }),
    );

    renderWithClient(<Home />);
    await screen.findByRole("link", { name: "Open Alpha" });

    await user.click(screen.getByRole("button", { name: "Add project" }));
    await user.type(
      screen.getByRole("textbox", { name: "New project name" }),
      "Beta{Enter}",
    );

    expect(
      await screen.findByRole("link", { name: "Open Beta" }),
    ).toBeInTheDocument();
    expect(posted).toEqual({ name: "Beta" });
  });

  it("renames a project (optimistic + persisted)", async () => {
    const user = userEvent.setup();
    const state = [project(alphaId, "Alpha", 0), project(betaId, "Beta", 1)];
    let patched: unknown;
    server.use(
      http.get(`${BASE}/projects`, () => HttpResponse.json(state)),
      http.patch(`${BASE}/projects/${alphaId}`, async ({ request }) => {
        patched = await request.json();
        state[0] = { ...state[0], name: "Renamed" };
        return HttpResponse.json(state[0]);
      }),
    );

    renderWithClient(<Home />);

    await user.click(await screen.findByRole("button", { name: "Alpha actions" }));
    await user.click(await screen.findByRole("menuitem", { name: "Rename" }));
    const input = screen.getByRole("textbox", { name: "Project name" });
    await user.clear(input);
    await user.type(input, "Renamed{Enter}");

    expect(
      await screen.findByRole("link", { name: "Open Renamed" }),
    ).toBeInTheDocument();
    expect(patched).toEqual({ name: "Renamed" });
  });

  it("deletes a project after confirmation", async () => {
    const user = userEvent.setup();
    const state = [project(alphaId, "Alpha", 0), project(betaId, "Beta", 1)];
    let deletedId: string | undefined;
    server.use(
      http.get(`${BASE}/projects`, () => HttpResponse.json(state)),
      http.delete(`${BASE}/projects/:id`, ({ params }) => {
        deletedId = params.id as string;
        const index = state.findIndex((p) => p.id === deletedId);
        if (index !== -1) state.splice(index, 1);
        return new HttpResponse(null, { status: 204 });
      }),
    );

    renderWithClient(<Home />);
    await screen.findByRole("link", { name: "Open Alpha" });

    await user.click(screen.getByRole("button", { name: "Alpha actions" }));
    await user.click(await screen.findByRole("menuitem", { name: "Delete" }));
    await user.click(await screen.findByRole("button", { name: "Delete" }));

    await waitFor(() =>
      expect(
        screen.queryByRole("link", { name: "Open Alpha" }),
      ).not.toBeInTheDocument(),
    );
    expect(deletedId).toBe(alphaId);
    expect(
      screen.getByRole("link", { name: "Open Beta" }),
    ).toBeInTheDocument();
  });
});
