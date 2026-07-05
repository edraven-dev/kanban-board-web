import { useState } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { EditableTitle } from "./editable-title";

function Harness({
  onSave,
  maxLength,
  initial = "Todo",
}: {
  onSave: (value: string) => void;
  maxLength?: number;
  initial?: string;
}) {
  const [value, setValue] = useState(initial);
  return (
    <EditableTitle
      value={value}
      label="Name"
      maxLength={maxLength}
      onSave={(next) => {
        onSave(next);
        setValue(next);
      }}
    />
  );
}

describe("EditableTitle", () => {
  it("enters edit mode and saves on Enter", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    render(<Harness onSave={onSave} />);

    await user.click(screen.getByRole("button", { name: "Todo" }));
    const input = screen.getByRole("textbox", { name: "Name" });
    await user.clear(input);
    await user.type(input, "Done{Enter}");

    expect(onSave).toHaveBeenCalledExactlyOnceWith("Done");
    expect(screen.getByRole("button", { name: "Done" })).toBeInTheDocument();
  });

  it("saves on blur", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    render(<Harness onSave={onSave} />);

    await user.click(screen.getByRole("button", { name: "Todo" }));
    const input = screen.getByRole("textbox", { name: "Name" });
    await user.clear(input);
    await user.type(input, "Blurred");
    fireEvent.blur(input);

    expect(onSave).toHaveBeenCalledExactlyOnceWith("Blurred");
  });

  it("cancels on Escape without saving", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    render(<Harness onSave={onSave} />);

    await user.click(screen.getByRole("button", { name: "Todo" }));
    const input = screen.getByRole("textbox", { name: "Name" });
    await user.clear(input);
    await user.type(input, "Discarded{Escape}");

    expect(onSave).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Todo" })).toBeInTheDocument();
  });

  it("does not save when the value is unchanged", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    render(<Harness onSave={onSave} />);

    await user.click(screen.getByRole("button", { name: "Todo" }));
    await user.type(screen.getByRole("textbox", { name: "Name" }), "{Enter}");

    expect(onSave).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Todo" })).toBeInTheDocument();
  });

  it("rejects an empty value with a message", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    render(<Harness onSave={onSave} />);

    await user.click(screen.getByRole("button", { name: "Todo" }));
    const input = screen.getByRole("textbox", { name: "Name" });
    await user.clear(input);
    await user.type(input, "{Enter}");

    expect(onSave).not.toHaveBeenCalled();
    expect(screen.getByRole("alert")).toHaveTextContent("Name is required");
  });

  it("rejects a value over the max length", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    render(<Harness onSave={onSave} maxLength={5} />);

    await user.click(screen.getByRole("button", { name: "Todo" }));
    const input = screen.getByRole("textbox", { name: "Name" });
    await user.clear(input);
    await user.type(input, "abcdef{Enter}");

    expect(onSave).not.toHaveBeenCalled();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Name must be 5 characters or fewer",
    );
  });

  it("starts in edit mode with autoEdit and calls onDone after saving", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    const onDone = vi.fn();
    render(
      <EditableTitle
        value="Todo"
        label="Name"
        autoEdit
        onSave={onSave}
        onDone={onDone}
      />,
    );

    const input = screen.getByRole("textbox", { name: "Name" });
    await user.clear(input);
    await user.type(input, "Done{Enter}");

    expect(onSave).toHaveBeenCalledExactlyOnceWith("Done");
    expect(onDone).toHaveBeenCalledTimes(1);
  });

  it("calls onDone when editing is cancelled", async () => {
    const user = userEvent.setup();
    const onDone = vi.fn();
    render(
      <EditableTitle
        value="Todo"
        label="Name"
        autoEdit
        onSave={vi.fn()}
        onDone={onDone}
      />,
    );

    await user.type(
      screen.getByRole("textbox", { name: "Name" }),
      "{Escape}",
    );

    expect(onDone).toHaveBeenCalledTimes(1);
  });
});
