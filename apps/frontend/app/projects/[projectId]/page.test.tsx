import { http, HttpResponse } from "msw";
import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { server } from "@/test/msw/server";
import { renderWithClient } from "@/test/render";

import ProjectPage from "./page";

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";
const alphaId = "018f1e2d-3c4b-7a6d-8e9f-000000000001";

describe("ProjectPage", () => {
  it("renders the project switcher for the routed project", async () => {
    server.use(
      http.get(`${BASE}/projects`, () =>
        HttpResponse.json([
          { id: alphaId, name: "Alpha", position: 0, createdAt: ts, updatedAt: ts },
        ]),
      ),
    );

    const ui = await ProjectPage({ params: Promise.resolve({ projectId: alphaId }) });
    renderWithClient(ui);

    expect(await screen.findByRole("button", { name: "Alpha" })).toBeInTheDocument();
    expect(
      screen.getByText("Boards for this project will appear here."),
    ).toBeInTheDocument();
  });
});
