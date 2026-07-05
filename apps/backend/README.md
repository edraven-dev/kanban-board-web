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

**Two aggregate roots** (for now): **Project** and **Board**. `Column` and `Card` are
entities *within* the Board aggregate — the board is their consistency boundary and
owns their ordering and count limits. (Either may become its own root later if it grows
independent invariants — hence "for now".)

| Entity | Aggregate | Key rules |
|---|---|---|
| **Project** | Project *(root)* | name 1–120, trimmed, non-empty |
| **Board** | Board *(root)* | name 1–120; **≤ 99 boards per project** |
| **Column** | Board | name 1–120; **≤ 99 columns per board** |
| **Card** | Board | title 1–200; description ≤ 10 000 (may be empty) |

Invariants live in the domain (value-object constructors return `Result<_,
DomainError>`); the ≤ 99 limits are enforced in the application layer within the same
transaction as the insert. Ordering is a contiguous integer `position` (0..n-1) scoped
to the parent, rewritten transactionally on reorder/move. Persistence keeps a
repository per entity as an implementation detail — the aggregate boundary is a
domain/consistency concept, not a 1:1 repository mapping.

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
