import type { ReactNode } from "react";
import { DndContext } from "@dnd-kit/core";
import { SortableContext } from "@dnd-kit/sortable";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { SortableItem } from "./sortable-item";

function Wrapper({ children }: { children: ReactNode }) {
  return (
    <DndContext>
      <SortableContext items={["a"]}>{children}</SortableContext>
    </DndContext>
  );
}

describe("SortableItem", () => {
  it("renders a labelled drag handle alongside its children", () => {
    render(
      <Wrapper>
        <SortableItem id="a">
          <span>Card body</span>
        </SortableItem>
      </Wrapper>,
    );

    expect(
      screen.getByRole("button", { name: "Drag to reorder" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Card body")).toBeInTheDocument();
  });

  it("supports a custom handle label", () => {
    render(
      <Wrapper>
        <SortableItem id="a" handleLabel="Reorder card">
          <span>Body</span>
        </SortableItem>
      </Wrapper>,
    );

    expect(
      screen.getByRole("button", { name: "Reorder card" }),
    ).toBeInTheDocument();
  });
});
