import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { RelativeTime } from "./relative-time";

describe("RelativeTime", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-05T12:00:00Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders a humanized 'created …' label for a past date string", () => {
    render(<RelativeTime date="2026-07-05T11:55:00Z" />);
    expect(screen.getByText("created 5 minutes ago")).toBeInTheDocument();
  });

  it("accepts a Date and refreshes on the 60s interval", () => {
    render(<RelativeTime date={new Date("2026-07-05T11:59:00Z")} />);
    expect(screen.getByText("created 1 minute ago")).toBeInTheDocument();

    act(() => {
      vi.advanceTimersByTime(60_000);
    });

    expect(screen.getByText("created 2 minutes ago")).toBeInTheDocument();
  });

  it("supports a custom prefix", () => {
    render(<RelativeTime date="2026-07-05T11:00:00Z" prefix="" />);
    expect(screen.getByText("about 1 hour ago")).toBeInTheDocument();
  });
});
