import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { AppHeader } from "./app-header";

describe("AppHeader", () => {
  it("shows the app name linking to home", () => {
    render(<AppHeader />);

    expect(screen.getByRole("link", { name: "Kanban" })).toHaveAttribute(
      "href",
      "/",
    );
  });
});
