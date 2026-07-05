import { describe, expect, it } from "vitest";

import { queryKeys } from "./keys";

describe("queryKeys", () => {
  it("builds a stable, scoped key for every entity", () => {
    expect(queryKeys.projects).toEqual(["projects"]);
    expect(queryKeys.boardsByProject("p1")).toEqual([
      "projects",
      "p1",
      "boards",
    ]);
    expect(queryKeys.boardFull("b1")).toEqual(["boards", "b1", "full"]);
    expect(queryKeys.columnsByBoard("b1")).toEqual([
      "boards",
      "b1",
      "columns",
    ]);
    expect(queryKeys.cardsByColumn("c1")).toEqual(["columns", "c1", "cards"]);
    expect(queryKeys.card("card1")).toEqual(["cards", "card1"]);
  });
});
