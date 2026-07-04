import { render, screen } from "@testing-library/react";
import { http, HttpResponse } from "msw";
import { describe, expect, it } from "vitest";

import { cn } from "@/lib/utils";

import { server } from "./msw/server";

describe("frontend test stack", () => {
  it("renders into jsdom and applies jest-dom matchers", () => {
    render(<p>hello kanban</p>);
    expect(screen.getByText("hello kanban")).toBeInTheDocument();
  });

  it("resolves the @ alias and merges class names", () => {
    expect(cn("a", false && "b", "c")).toBe("a c");
  });

  it("intercepts network requests with MSW", async () => {
    server.use(
      http.get("https://example.test/ping", () =>
        HttpResponse.json({ ok: true }),
      ),
    );

    const res = await fetch("https://example.test/ping");
    await expect(res.json()).resolves.toEqual({ ok: true });
  });
});
