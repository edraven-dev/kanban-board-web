# frontend

Next.js 16 (App Router, React 19) UI for the Kanban board. Dark theme by default,
served on port **3000**.

## Stack

- **Next.js 16** (App Router, Turbopack, React Strict Mode), **TypeScript** (strict)
- **Tailwind v4** + **shadcn/ui** on **base-ui** (`@base-ui/react`)
- **TanStack Query** — server-state cache + optimistic updates
- **dnd-kit** — accessible drag-and-drop (pointer + keyboard sensors)
- **react-hook-form** + **zod** — forms and validation
- **date-fns** — humanized "created N ago" times

## Architecture

**Atomic Design.** Components compose upward and take data via props:

```
app/                     # routes: / , /projects/[projectId] , /boards/[boardId]
components/
  ui/                    # shadcn primitives (generated)
  atoms/                 # RelativeTime, EmptyState, Spinner
  molecules/             # EditableTitle, ConfirmDeleteDialog, InlineCreate,
                         #   EntityMenu, CardTile, SortableItem, TileGrid
  organisms/             # ProjectList, BoardCanvas, KanbanColumn, CardModal, ...
lib/
  api/                   # typed client (client.ts), zod schemas, endpoint fns
  query/                 # query-key factory + QueryClient provider
  hooks/                 # useProjects, useBoards, useColumns, useCards, useReorder
  dnd/                   # board drag routing + shared drag helpers
```

**Data flow.** TanStack Query owns **all** server state — components never fetch ad
hoc. The typed API client (`lib/api/client.ts`) is the single I/O boundary and
validates every response with zod. Mutations do **optimistic updates** that roll back
and toast on error, which keeps drag reorders and cross-column moves smooth. Reorder
for projects, boards, columns, and cards shares one generic `useReorder` hook.

**API base URL.** `NEXT_PUBLIC_API_URL` (default `http://localhost:5000`), and every
path is prefixed with `/api`. In production the value is empty, so requests are
same-origin (`/api/...`).

## Run

Needs the backend and Postgres running (see the repo root README). Then:

```bash
pnpm --filter frontend dev   # http://localhost:3000
```

## Test

```bash
pnpm --filter frontend test           # Vitest + React Testing Library + MSW
pnpm --filter frontend test:coverage  # coverage, fails under 90% lines/branches
pnpm --filter frontend test:e2e       # Playwright full-stack happy path
```

- **Unit/integration** — pure logic (API client, zod schemas, hooks) plus feature
  flows rendered with a real `QueryClient` and the API mocked by **MSW**, covering
  optimistic update + rollback and the 99-limit (409) paths.
- **e2e** — a Playwright happy path that drives the real UI against a live
  frontend + backend + isolated Postgres. Kept separate from `test`. Setup and how to
  run: [e2e/README.md](e2e/README.md).
