import { http, HttpResponse } from "msw";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type { Card } from "@/lib/api/schemas";
import { server } from "@/test/msw/server";
import { renderWithClient } from "@/test/render";

import { CardModal } from "./card-modal";

const BASE = "http://localhost:5000/api";
const ts = "2026-07-05T00:00:00Z";
const boardId = "018f1e2d-3c4b-7a6d-8e9f-000000000500";
const cardId = "018f1e2d-3c4b-7a6d-8e9f-000000000001";
const columnId = "018f1e2d-3c4b-7a6d-8e9f-000000000011";

const card: Card = {
  id: cardId,
  columnId,
  title: "Task",
  description: "old description",
  position: 0,
  createdAt: ts,
  updatedAt: ts,
};

function renderModal() {
  return renderWithClient(
    <CardModal
      boardId={boardId}
      card={card}
      columnName="To Do"
      open
      onOpenChange={vi.fn()}
    />,
  );
}

describe("CardModal", () => {
  it("shows the card details", () => {
    renderModal();
    expect(screen.getByRole("button", { name: "Task" })).toBeInTheDocument();
    expect(screen.getByText("To Do")).toBeInTheDocument();
    expect(screen.getByText(cardId)).toBeInTheDocument();
  });

  it("edits the title", async () => {
    const user = userEvent.setup();
    let patched: unknown;
    server.use(
      http.patch(`${BASE}/cards/${cardId}`, async ({ request }) => {
        patched = await request.json();
        return HttpResponse.json({ ...card, title: "Renamed" });
      }),
    );

    renderModal();

    await user.click(screen.getByRole("button", { name: "Task" }));
    const input = screen.getByRole("textbox", { name: "Card title" });
    await user.clear(input);
    await user.type(input, "Renamed{Enter}");

    await waitFor(() => expect(patched).toEqual({ title: "Renamed" }));
  });

  it("edits the description without touching created_at", async () => {
    const user = userEvent.setup();
    let patched: unknown;
    server.use(
      http.patch(`${BASE}/cards/${cardId}`, async ({ request }) => {
        patched = await request.json();
        return HttpResponse.json({ ...card, description: "New details" });
      }),
    );

    renderModal();

    const textarea = screen.getByLabelText("Description");
    await user.clear(textarea);
    await user.type(textarea, "New details");
    await user.click(screen.getByRole("button", { name: "Save description" }));

    await waitFor(() =>
      expect(patched).toEqual({ description: "New details" }),
    );
  });

  it("deletes the card after confirmation", async () => {
    const user = userEvent.setup();
    const onOpenChange = vi.fn();
    let deleted = false;
    server.use(
      http.delete(`${BASE}/cards/${cardId}`, () => {
        deleted = true;
        return new HttpResponse(null, { status: 204 });
      }),
    );

    renderWithClient(
      <CardModal
        boardId={boardId}
        card={card}
        columnName="To Do"
        open
        onOpenChange={onOpenChange}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Delete card" }));
    await user.click(await screen.findByRole("button", { name: "Delete" }));

    await waitFor(() => expect(deleted).toBe(true));
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });
});
