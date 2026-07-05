import { http, HttpResponse, delay } from "msw";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { server } from "@/test/msw/server";
import { renderWithClient } from "@/test/render";

import Home from "./page";

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";

function project(id: string, name: string, position: number) {
  return { id, name, position, createdAt: ts, updatedAt: ts };
}

const alphaId = "018f1e2d-3c4b-7a6d-8e9f-000000000001";
const betaId = "018f1e2d-3c4b-7a6d-8e9f-000000000002";

describe("Home (project chooser)", () => {
  it("renders the project list from the API", async () => {
    server.use(
      http.get(`${BASE}/projects`, () =>
        HttpResponse.json([
          project(alphaId, "Alpha", 0),
          project(betaId, "Beta", 1),
        ]),
      ),
    );

    renderWithClient(<Home />);

    const alpha = await screen.findByRole("link", { name: "Alpha" });
    expect(alpha).toHaveAttribute("href", `/projects/${alphaId}`);
    expect(screen.getByRole("link", { name: "Beta" })).toHaveAttribute(
      "href",
      `/projects/${betaId}`,
    );
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
    await screen.findByRole("link", { name: "Alpha" });

    await user.click(screen.getByRole("button", { name: "Add project" }));
    await user.type(
      screen.getByRole("textbox", { name: "New project name" }),
      "Beta{Enter}",
    );

    expect(await screen.findByRole("link", { name: "Beta" })).toBeInTheDocument();
    expect(posted).toEqual({ name: "Beta" });
  });
});
