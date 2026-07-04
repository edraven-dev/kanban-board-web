# backend

Rust + axum API for the Kanban board, built with a hexagonal architecture. The
entire HTTP API is served under the `/api` prefix (e.g. `GET /api/health`).
Interactive docs (Swagger UI) are at `/api/docs`, and the OpenAPI spec at
`/api/openapi.json`.

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
