import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { renderWithClient } from "@/test/render";

import { EntityTile } from "./entity-tile";

describe("EntityTile", () => {
  it("renders the name and a navigation link", () => {
    renderWithClient(
      <EntityTile
        name="Alpha"
        seed="a"
        href="/projects/a"
        openLabel="Open Alpha"
      />,
    );

    expect(screen.getByText("Alpha")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Open Alpha" })).toHaveAttribute(
      "href",
      "/projects/a",
    );
  });

  it("renames inline from the overflow menu", async () => {
    const user = userEvent.setup();
    const onRename = vi.fn();
    renderWithClient(
      <EntityTile
        name="Alpha"
        seed="a"
        href="/projects/a"
        openLabel="Open Alpha"
        menuLabel="Alpha actions"
        renameLabel="Project name"
        onRename={onRename}
        onDelete={vi.fn()}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Alpha actions" }));
    await user.click(await screen.findByRole("menuitem", { name: "Rename" }));
    const input = screen.getByRole("textbox", { name: "Project name" });
    await user.clear(input);
    await user.type(input, "Renamed{Enter}");

    expect(onRename).toHaveBeenCalledExactlyOnceWith("Renamed");
    expect(
      await screen.findByRole("link", { name: "Open Alpha" }),
    ).toBeInTheDocument();
  });

  it("triggers delete from the overflow menu", async () => {
    const user = userEvent.setup();
    const onDelete = vi.fn();
    renderWithClient(
      <EntityTile
        name="Alpha"
        seed="a"
        href="/projects/a"
        openLabel="Open Alpha"
        menuLabel="Alpha actions"
        onRename={vi.fn()}
        onDelete={onDelete}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Alpha actions" }));
    await user.click(await screen.findByRole("menuitem", { name: "Delete" }));

    expect(onDelete).toHaveBeenCalledTimes(1);
  });

  it("renders a non-interactive preview", () => {
    renderWithClient(<EntityTile name="Alpha" seed="a" preview />);

    expect(screen.getByText("Alpha")).toBeInTheDocument();
    expect(screen.queryByRole("link")).not.toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
