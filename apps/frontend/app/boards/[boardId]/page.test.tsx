import { http, HttpResponse } from "msw";
import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { server } from "@/test/msw/server";
import { renderWithClient } from "@/test/render";

import BoardPage from "./page";

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";
const boardId = "018f1e2d-3c4b-7a6d-8e9f-000000000500";
const projectId = "018f1e2d-3c4b-7a6d-8e9f-000000000900";
const columnId = "018f1e2d-3c4b-7a6d-8e9f-000000000001";

describe("BoardPage", () => {
  it("renders the routed board's canvas", async () => {
    server.use(
      http.get(`${BASE}/boards/${boardId}/full`, () =>
        HttpResponse.json({
          id: boardId,
          projectId,
          name: "My Board",
          position: 0,
          createdAt: ts,
          updatedAt: ts,
          columns: [
            {
              id: columnId,
              boardId,
              name: "To Do",
              position: 0,
              createdAt: ts,
              updatedAt: ts,
              cards: [],
            },
          ],
        }),
      ),
    );

    const ui = await BoardPage({
      params: Promise.resolve({ boardId }),
    });
    renderWithClient(ui);

    expect(
      await screen.findByRole("button", { name: "To Do" }),
    ).toBeInTheDocument();
  });
});
