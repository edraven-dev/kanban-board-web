# Kanban Board

A web Kanban board — **Project → Board → Column → Card** — with drag-and-drop
reordering, optimistic UI, and a self-documenting API. Turborepo monorepo with a
Next.js frontend and a Rust/axum backend over PostgreSQL.

## Stack

- **Frontend** (`apps/frontend`) — Next.js 16 (App Router, React 19), TypeScript,
  Tailwind v4, shadcn/ui (base-ui), TanStack Query, dnd-kit. Dev port **3000**.
- **Backend** (`apps/backend`) — Rust, axum, sqlx, PostgreSQL 18; hexagonal / DDD.
  The API is served under **`/api`** (Swagger UI at `/api/docs`). Dev port **5000**.
- **Monorepo** — Turborepo + pnpm; shared configs in `packages/`
  (`@repo/eslint-config`, `@repo/typescript-config`).

## Prerequisites

- Node **22+** and **pnpm 11** (`corepack enable`)
- **Rust** (stable) and a **Docker- or Podman-compatible daemon** (for Postgres and
  the backend's integration tests)

## Quick start

```bash
pnpm install
docker compose up -d                              # PostgreSQL 18 on :5432
cp apps/backend/.env.example apps/backend/.env
pnpm dev                                          # frontend :3000 + backend :5000
```

Open http://localhost:3000. To populate demo data: `pnpm --filter backend seed`.

## Environment

Backend (`apps/backend/.env`):

| Variable       | Default       | Purpose                                                                                        |
| -------------- | ------------- | ---------------------------------------------------------------------------------------------- |
| `APP_ENV`      | `development` | `development` adds permissive CORS (dev frontend is a different origin); `production` omits it |
| `DATABASE_URL` | —             | Postgres connection (matches `docker-compose.yml`)                                             |
| `PORT`         | `5000`        | API port                                                                                       |

Frontend: `NEXT_PUBLIC_API_URL` (optional, default `http://localhost:5000`) is the
API base in dev. Leave it empty in production, where the app and API are served
same-origin under `/api` (no CORS needed).

## Architecture

- **Backend** — hexagonal (ports & adapters) with a DDD core and **two aggregate
  roots**, Project and Board (Board owns its Columns → Cards). Details in
  [apps/backend/README.md](apps/backend/README.md).
- **Frontend** — Atomic Design; TanStack Query owns all server state behind a typed,
  zod-validated API client; every reorder/move is optimistic with rollback. Details
  in [apps/frontend/README.md](apps/frontend/README.md).

**Ordering** is a contiguous integer `position` (0..n-1) scoped to the parent;
reorder/move rewrites the affected rows in a single transaction.

## API

JSON under the `/api` prefix. Interactive docs at http://localhost:5000/api/docs and
the OpenAPI spec at `/api/openapi.json`. Full endpoint reference:
[apps/backend/README.md](apps/backend/README.md#api).

## Testing

```bash
pnpm test                        # unit + integration across all packages
pnpm test:coverage               # the above with coverage gates
pnpm --filter frontend test:e2e  # Playwright full-stack happy path
```

- **Backend** integration tests self-provision Postgres via **testcontainers** — a
  Docker/Podman daemon must be available; no test `DATABASE_URL` is needed.
- **Frontend** unit/integration tests use Vitest + React Testing Library + MSW.
- **e2e** boots the whole stack against an isolated database — see
  [apps/frontend/e2e/README.md](apps/frontend/e2e/README.md).

## Other tasks

```bash
pnpm lint          # ESLint (frontend) + clippy (backend)
pnpm check-types   # tsc (frontend) + cargo check (backend)
pnpm build         # next build + cargo build --release
```
