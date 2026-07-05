import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { EntityMenu } from "./entity-menu";

describe("EntityMenu", () => {
  it("invokes onRename from the menu", async () => {
    const user = userEvent.setup();
    const onRename = vi.fn();
    const onDelete = vi.fn();
    render(
      <EntityMenu
        label="Board actions"
        onRename={onRename}
        onDelete={onDelete}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Board actions" }));
    await user.click(await screen.findByRole("menuitem", { name: "Rename" }));

    expect(onRename).toHaveBeenCalledTimes(1);
    expect(onDelete).not.toHaveBeenCalled();
  });

  it("invokes onDelete from the menu", async () => {
    const user = userEvent.setup();
    const onRename = vi.fn();
    const onDelete = vi.fn();
    render(
      <EntityMenu
        label="Board actions"
        onRename={onRename}
        onDelete={onDelete}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Board actions" }));
    await user.click(await screen.findByRole("menuitem", { name: "Delete" }));

    expect(onDelete).toHaveBeenCalledTimes(1);
    expect(onRename).not.toHaveBeenCalled();
  });
});
