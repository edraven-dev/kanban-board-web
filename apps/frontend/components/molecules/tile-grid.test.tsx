import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { TileGrid } from "./tile-grid";

describe("TileGrid", () => {
  it("renders its children", () => {
    render(
      <TileGrid>
        <span>one</span>
        <span>two</span>
      </TileGrid>,
    );

    expect(screen.getByText("one")).toBeInTheDocument();
    expect(screen.getByText("two")).toBeInTheDocument();
  });
});
