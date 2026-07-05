import { http, HttpResponse } from "msw";
import { describe, expect, it } from "vitest";

import { server } from "@/test/msw/server";

import {
  createProject,
  deleteProject,
  listProjects,
  reorderProjects,
  updateProject,
} from "./projects";

const BASE = "http://localhost:5000/api";
const uuid = "018f1e2d-3c4b-7a6d-8e9f-0123456789ab";
const ts = "2026-07-05T00:00:00Z";
const project = { id: uuid, name: "Alpha", position: 0, createdAt: ts, updatedAt: ts };

describe("projects api", () => {
  it("lists projects", async () => {
    server.use(http.get(`${BASE}/projects`, () => HttpResponse.json([project])));
    expect(await listProjects()).toEqual([project]);
  });

  it("creates a project and sends the name body", async () => {
    let received: unknown;
    server.use(
      http.post(`${BASE}/projects`, async ({ request }) => {
        received = await request.json();
        return HttpResponse.json({ ...project, name: "New" }, { status: 201 });
      }),
    );
    const result = await createProject({ name: "New" });
    expect(received).toEqual({ name: "New" });
    expect(result.name).toBe("New");
  });

  it("updates a project", async () => {
    server.use(
      http.patch(`${BASE}/projects/${uuid}`, () =>
        HttpResponse.json({ ...project, name: "Renamed" }),
      ),
    );
    expect((await updateProject(uuid, { name: "Renamed" })).name).toBe("Renamed");
  });

  it("deletes a project", async () => {
    server.use(
      http.delete(
        `${BASE}/projects/${uuid}`,
        () => new HttpResponse(null, { status: 204 }),
      ),
    );
    await expect(deleteProject(uuid)).resolves.toBeUndefined();
  });

  it("reorders projects with the full ordered id list", async () => {
    let received: unknown;
    server.use(
      http.put(`${BASE}/projects/reorder`, async ({ request }) => {
        received = await request.json();
        return new HttpResponse(null, { status: 204 });
      }),
    );
    await reorderProjects({ orderedIds: [uuid] });
    expect(received).toEqual({ orderedIds: [uuid] });
  });
});
