import { http, HttpResponse } from "msw";
import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { server } from "@/test/msw/server";
import { renderWithClient } from "@/test/render";

import ProjectPage from "./page";

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";
const alphaId = "018f1e2d-3c4b-7a6d-8e9f-000000000001";
const boardId = "018f1e2d-3c4b-7a6d-8e9f-0000000000b1";

describe("ProjectPage", () => {
  it("renders the routed project's boards", async () => {
    server.use(
      http.get(`${BASE}/projects`, () =>
        HttpResponse.json([
          { id: alphaId, name: "Alpha", position: 0, createdAt: ts, updatedAt: ts },
        ]),
      ),
      http.get(`${BASE}/projects/${alphaId}/boards`, () =>
        HttpResponse.json([
          {
            id: boardId,
            projectId: alphaId,
            name: "Sprint 1",
            position: 0,
            createdAt: ts,
            updatedAt: ts,
          },
        ]),
      ),
    );

    const ui = await ProjectPage({
      params: Promise.resolve({ projectId: alphaId }),
    });
    renderWithClient(ui);

    expect(
      await screen.findByRole("link", { name: "Open Sprint 1" }),
    ).toHaveAttribute("href", `/boards/${boardId}`);
  });
});
