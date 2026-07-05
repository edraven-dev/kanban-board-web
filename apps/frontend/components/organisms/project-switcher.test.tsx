import { http, HttpResponse } from "msw";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { server } from "@/test/msw/server";
import { renderWithClient } from "@/test/render";

import { ProjectSwitcher } from "./project-switcher";

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";
const alphaId = "018f1e2d-3c4b-7a6d-8e9f-000000000001";
const betaId = "018f1e2d-3c4b-7a6d-8e9f-000000000002";

function project(id: string, name: string, position: number) {
  return { id, name, position, createdAt: ts, updatedAt: ts };
}

describe("ProjectSwitcher", () => {
  it("shows the current project and links to the others", async () => {
    server.use(
      http.get(`${BASE}/projects`, () =>
        HttpResponse.json([
          project(alphaId, "Alpha", 0),
          project(betaId, "Beta", 1),
        ]),
      ),
    );

    const user = userEvent.setup();
    renderWithClient(<ProjectSwitcher currentProjectId={alphaId} />);

    await user.click(await screen.findByRole("button", { name: "Alpha" }));

    const beta = await screen.findByRole("menuitem", { name: "Beta" });
    expect(beta).toHaveAttribute("href", `/projects/${betaId}`);
    expect(
      screen.getByRole("menuitem", { name: "All projects" }),
    ).toHaveAttribute("href", "/");
  });
});
