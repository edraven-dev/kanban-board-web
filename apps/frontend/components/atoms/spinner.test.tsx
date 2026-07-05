import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { Spinner } from "./spinner";

describe("Spinner", () => {
  it("exposes a status role with the default label", () => {
    render(<Spinner />);
    const status = screen.getByRole("status");
    expect(status).toBeInTheDocument();
    expect(status).toHaveTextContent("Loading…");
  });

  it("uses a custom label", () => {
    render(<Spinner label="Fetching boards" />);
    expect(screen.getByRole("status")).toHaveTextContent("Fetching boards");
  });
});
