import { http, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";

import { server } from "@/test/msw/server";

import { ApiError, apiFetch, apiUrl } from "./client";

const BASE = "http://localhost:5000/api";

describe("apiUrl", () => {
  it("uses the dev default base and prefixes every path with /api", () => {
    expect(apiUrl("/projects")).toBe("http://localhost:5000/api/projects");
  });

  it("uses an empty base in prod so requests are same-origin /api paths", () => {
    vi.stubEnv("NEXT_PUBLIC_API_URL", "");
    expect(apiUrl("/projects")).toBe("/api/projects");
  });

  it("honors an explicitly configured base URL", () => {
    vi.stubEnv("NEXT_PUBLIC_API_URL", "https://api.example.com");
    expect(apiUrl("/boards/1/full")).toBe(
      "https://api.example.com/api/boards/1/full",
    );
  });
});

describe("apiFetch", () => {
  it("sends JSON headers and a serialized body, returning parsed JSON", async () => {
    let contentType: string | null = null;
    let received: unknown;
    server.use(
      http.post(`${BASE}/projects`, async ({ request }) => {
        contentType = request.headers.get("content-type");
        received = await request.json();
        return HttpResponse.json({ id: "x" }, { status: 201 });
      }),
    );

    const result = await apiFetch("/projects", {
      method: "POST",
      body: { name: "Alpha" },
    });

    expect(contentType).toBe("application/json");
    expect(received).toEqual({ name: "Alpha" });
    expect(result).toEqual({ id: "x" });
  });

  it("returns null for a 204 No Content response", async () => {
    server.use(
      http.delete(
        `${BASE}/projects/1`,
        () => new HttpResponse(null, { status: 204 }),
      ),
    );
    await expect(
      apiFetch("/projects/1", { method: "DELETE" }),
    ).resolves.toBeNull();
  });

  it("throws a typed ApiError parsed from a 4xx error body", async () => {
    server.use(
      http.get(`${BASE}/projects`, () =>
        HttpResponse.json(
          { error: { code: "limit_exceeded", message: "too many" } },
          { status: 409 },
        ),
      ),
    );

    const err = await apiFetch("/projects").catch((e: unknown) => e);
    expect(err).toBeInstanceOf(ApiError);
    expect(err).toEqual(
      expect.objectContaining({ status: 409, code: "limit_exceeded" }),
    );
    expect((err as ApiError).message).toBe("too many");
  });

  it("falls back to a generic ApiError when a 5xx body is not the expected shape", async () => {
    server.use(
      http.get(
        `${BASE}/projects`,
        () => new HttpResponse("boom", { status: 500 }),
      ),
    );

    const err = await apiFetch("/projects").catch((e: unknown) => e);
    expect(err).toBeInstanceOf(ApiError);
    expect(err).toEqual(
      expect.objectContaining({ status: 500, code: "unknown" }),
    );
  });
});
