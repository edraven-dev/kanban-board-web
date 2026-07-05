# backend

Rust + axum API for the Kanban board, built with a hexagonal architecture. The
entire HTTP API is served under the `/api` prefix (e.g. `GET /api/health`).
Interactive docs (Swagger UI) are at `/api/docs`, and the OpenAPI spec at
`/api/openapi.json`.

## Architecture

Hexagonal (ports & adapters) with a DDD core; dependencies point **inward** — the
domain knows nothing about axum, sqlx, or HTTP.

```
src/
  domain/          # pure: entities, value objects, ids, DomainError, repository ports
  application/     # use-case services orchestrating domain + ports; ApplicationError
  adapters/
    inbound/http/          # axum handlers, camelCase DTOs, error->status mapping, router
    outbound/persistence/  # Postgres repositories (sqlx), row records -> domain
  infrastructure/  # config, db pool, telemetry
  app_state.rs     # composition root: wires repositories into services
```

**Dependency rule**: `domain` depends on nothing; `application` depends only on
`domain` via port traits; `adapters` and `infrastructure` depend inward. The `Pg*Repo`
adapters implement the domain's repository ports, so the core is testable with
in-memory fakes and Postgres stays a swappable detail.

### Domain model

**Two aggregate roots**: **Project** and **Board**. The `Board` root **owns** its ordered
`Column`s, each owning its ordered `Card`s — one consistency boundary, one ownership
tree. All structural changes to columns and cards go through methods on the `Board` root
(`src/domain/board.rs`), which enforce the invariants below.

| Entity | Aggregate | Key rules |
|---|---|---|
| **Project** | Project *(root)* | name 1–120, trimmed, non-empty |
| **Board** | Board *(root)* | name 1–120; **≤ 99 boards per project** |
| **Column** | Board | name 1–120; **≤ 99 columns per board** |
| **Card** | Board | title 1–200; description ≤ 10 000 (may be empty) |

Value-object constructors return `Result<_, DomainError>`; the `Board` root returns
`BoardError` for structural failures (missing column/card, column limit, bad reorder
set). The **≤ 99 columns** limit and same-/cross-column card moves are enforced by the
`Board` root; **≤ 99 boards per project** is a Board-side concern checked in
`BoardService`. Ordering is a contiguous integer `position` (0..n-1) scoped to the
parent, re-sequenced by the root and persisted with the aggregate.

**One repository per aggregate root.** `PgProjectRepo` owns the `projects` table;
`PgBoardRepo` owns `boards` + `columns` + `cards`. Column/card writes **load the whole
board aggregate, mutate it through the `Board` root, and save the whole tree in one
transaction** (`save`) — generalizing the board-full read model to the write side.

**Aggregate boundaries are non-negotiable** (see the root `CLAUDE.md`): a repository's
SQL may touch **only its own aggregate's tables**. `PgBoardRepo` never reads `projects`;
when the Board aggregate needs to know a project exists (creating a board), it asks the
`ProjectDirectory` **system service** — the Project aggregate's public interface — rather
than querying the `projects` table. In this monolith the directory is backed by the
Project repository; across a service split it would be an API call. The
`boards.project_id → projects.id` foreign key is kept only as a database safety net, not
as a licence to read across the boundary.

## Run locally

```bash
# 1. Start PostgreSQL 18 (from the repo root)
docker compose up -d

# 2. Configure the app
cp .env.example .env        # adjust if needed

# 3. Run the API (listens on :5000)
pnpm --filter backend dev   # or: cargo run
```

- `APP_ENV=development` adds a **permissive** CORS layer, since the dev frontend
  runs on a different origin (`:3000`).
- `APP_ENV=production` adds **no** CORS layer — production is served same-origin
  under `host.com/api`.

## Migrations

SQL migrations live in `migrations/` (timestamped `<YYYYMMDDHHMMSS>_<name>.sql`) and
are applied automatically on app startup and in tests. Managed with `sqlx-cli`
(`cargo install sqlx-cli --no-default-features --features rustls,postgres`):

```bash
pnpm --filter backend migrate:add <name>   # scaffold a new (empty) migration file
pnpm --filter backend migrate:run          # apply pending migrations (uses DATABASE_URL)
pnpm --filter backend migrate:info         # show applied / pending status
pnpm --filter backend migrate:revert       # roll back the last migration
```

`migrate:add` only creates the file — you write the SQL (sqlx is not an ORM; there's
no schema diffing). Compile-time-checked queries (`query!`, from later tasks) use an
offline cache in `.sqlx/` via `cargo sqlx prepare`; set `SQLX_OFFLINE=true` to build
without a database.

## Test

```bash
pnpm --filter backend test           # unit + HTTP + DB (testcontainers) integration
pnpm --filter backend test:coverage  # cargo llvm-cov, fails under 90% lines
```

Integration tests provision their own PostgreSQL via **testcontainers**: the
`db_test!` macro (in `tests/common`) boots **one container per test binary**,
migrates it into a template database, and clones a fresh database from that template
for each test — so per-test setup is a fast copy, not a container boot. No
pre-running Postgres or `DATABASE_URL` is needed, but a **Docker- or
Podman-compatible daemon must be available**. Test scripts run through
`scripts/with-cleanup.sh`, which removes the container after the run (covering
Podman, where testcontainers' Ryuk reaper is unreliable).

Coverage tooling: `cargo install cargo-llvm-cov` and
`rustup component add llvm-tools-preview`.
