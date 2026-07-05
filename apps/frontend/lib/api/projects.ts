import { apiFetch } from "./client";
import {
  projectListSchema,
  projectSchema,
  type NameInput,
  type Project,
  type ReorderInput,
} from "./schemas";

export async function listProjects(): Promise<Project[]> {
  return projectListSchema.parse(await apiFetch("/projects"));
}

export async function createProject(input: NameInput): Promise<Project> {
  return projectSchema.parse(
    await apiFetch("/projects", { method: "POST", body: input }),
  );
}

export async function updateProject(
  id: string,
  input: NameInput,
): Promise<Project> {
  return projectSchema.parse(
    await apiFetch(`/projects/${id}`, { method: "PATCH", body: input }),
  );
}

export async function deleteProject(id: string): Promise<void> {
  await apiFetch(`/projects/${id}`, { method: "DELETE" });
}

export async function reorderProjects(input: ReorderInput): Promise<void> {
  await apiFetch("/projects/reorder", { method: "PUT", body: input });
}
