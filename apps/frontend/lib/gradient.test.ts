import { describe, expect, it } from "vitest";

import { pastelGradient } from "./gradient";

describe("pastelGradient", () => {
  it("is deterministic for the same seed", () => {
    expect(pastelGradient("project-1")).toBe(pastelGradient("project-1"));
  });

  it("differs for different seeds", () => {
    expect(pastelGradient("project-1")).not.toBe(pastelGradient("project-2"));
  });

  it("produces a two-stop hsl linear-gradient", () => {
    expect(pastelGradient("abc")).toMatch(
      /^linear-gradient\(\d+deg, hsl\(\d+ 70% 85%\), hsl\(\d+ 65% 78%\)\)$/,
    );
  });
});
