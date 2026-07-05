import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { Card } from "@/lib/api/schemas";

import { CardTile } from "./card-tile";

function card(overrides: Partial<Card> = {}): Card {
  return {
    id: "018f1e2d-3c4b-7a6d-8e9f-000000000001",
    columnId: "018f1e2d-3c4b-7a6d-8e9f-000000000011",
    title: "Write the report",
    description: "",
    position: 0,
    createdAt: "2026-07-05T11:55:00Z",
    updatedAt: "2026-07-05T11:55:00Z",
    ...overrides,
  };
}

describe("CardTile", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-05T12:00:00Z"));
  });

  afterEach(() => vi.useRealTimers());

  it("shows only the title and a humanized created time", () => {
    render(<CardTile card={card()} />);

    expect(screen.getByText("Write the report")).toBeInTheDocument();
    expect(screen.getByText("created 5 minutes ago")).toBeInTheDocument();
  });
});
