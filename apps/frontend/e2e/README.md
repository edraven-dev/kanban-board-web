# End-to-end tests (Playwright)

A single full-stack happy path (`happy-path.e2e.ts`) that drives the real UI
against a live frontend, backend, and Postgres, re-asserting persistence after a
reload at each step: create/rename a project → create a board → add/reorder
columns → add cards → move a card across columns → edit a card's description →
delete a card, board, and project.

Kept separate from the unit/integration suite (`pnpm test`) — these need the full
stack running and are not part of `turbo run test`.

## Prerequisites

- Local Postgres running (`docker compose up -d` from the repo root).
- `sqlx-cli` on `PATH` (`cargo install sqlx-cli`) — used to provision the DB.
- Playwright's browser, once: `pnpm --filter frontend exec playwright install chromium`.

## Run

```bash
pnpm --filter frontend test:e2e
```

Playwright's `webServer` boots everything for you:

1. Resets an **isolated** `kanban_e2e` database (drop/create/migrate) — the dev
   `kanban` database is never touched — and starts the API (`cargo run`) on `:5000`.
2. Starts the frontend (`next dev`) on `:3000`, pointed at that API.

Override the database with `E2E_DATABASE_URL` if your Postgres differs. Run
`pnpm --filter frontend exec playwright test --ui` to debug interactively.
