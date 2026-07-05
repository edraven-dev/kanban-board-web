import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { InlineCreate } from "./inline-create";

describe("InlineCreate", () => {
  it("expands, submits, and resets for the next entry", async () => {
    const user = userEvent.setup();
    const onCreate = vi.fn();
    render(<InlineCreate label="board" onCreate={onCreate} />);

    await user.click(screen.getByRole("button", { name: "Add board" }));
    const input = screen.getByRole("textbox", { name: "New board name" });
    await user.type(input, "Backlog{Enter}");

    expect(onCreate).toHaveBeenCalledExactlyOnceWith("Backlog");
    expect(screen.getByRole("textbox", { name: "New board name" })).toHaveValue(
      "",
    );
  });

  it("rejects an empty submission", async () => {
    const user = userEvent.setup();
    const onCreate = vi.fn();
    render(<InlineCreate label="column" onCreate={onCreate} />);

    await user.click(screen.getByRole("button", { name: "Add column" }));
    await user.click(screen.getByRole("button", { name: "Add" }));

    expect(onCreate).not.toHaveBeenCalled();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Column name is required",
    );
  });

  it("rejects a value over the max length", async () => {
    const user = userEvent.setup();
    const onCreate = vi.fn();
    render(<InlineCreate label="card" onCreate={onCreate} maxLength={3} />);

    await user.click(screen.getByRole("button", { name: "Add card" }));
    await user.type(
      screen.getByRole("textbox", { name: "New card name" }),
      "abcd{Enter}",
    );

    expect(onCreate).not.toHaveBeenCalled();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Name must be 3 characters or fewer",
    );
  });

  it("collapses on Cancel", async () => {
    const user = userEvent.setup();
    render(<InlineCreate label="board" onCreate={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: "Add board" }));
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    expect(
      screen.getByRole("button", { name: "Add board" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  });

  it("collapses on Escape", async () => {
    const user = userEvent.setup();
    render(<InlineCreate label="board" onCreate={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: "Add board" }));
    await user.type(
      screen.getByRole("textbox", { name: "New board name" }),
      "{Escape}",
    );

    expect(
      screen.getByRole("button", { name: "Add board" }),
    ).toBeInTheDocument();
  });
});
