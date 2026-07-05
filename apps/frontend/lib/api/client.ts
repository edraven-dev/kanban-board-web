import { z } from "zod";

const apiErrorBodySchema = z.object({
  error: z.object({
    code: z.string(),
    message: z.string(),
  }),
});

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
  }
}

function baseUrl(): string {
  return process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:5000";
}

/** Builds an absolute API URL, always prefixing the path with `/api`. */
export function apiUrl(path: string): string {
  return `${baseUrl()}/api${path}`;
}

type RequestOptions = {
  method?: string;
  body?: unknown;
};

export async function apiFetch(
  path: string,
  options: RequestOptions = {},
): Promise<unknown> {
  const { method = "GET", body } = options;
  const res = await fetch(apiUrl(path), {
    method,
    headers: { "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });

  if (!res.ok) {
    throw await toApiError(res);
  }
  if (res.status === 204) {
    return null;
  }
  return res.json();
}

async function toApiError(res: Response): Promise<ApiError> {
  const body: unknown = await res.json().catch(() => null);
  const parsed = apiErrorBodySchema.safeParse(body);
  if (parsed.success) {
    return new ApiError(
      res.status,
      parsed.data.error.code,
      parsed.data.error.message,
    );
  }
  return new ApiError(res.status, "unknown", res.statusText || "Request failed");
}
