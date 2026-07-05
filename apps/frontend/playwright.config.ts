import { defineConfig, devices } from "@playwright/test";

const BACKEND_PORT = 5000;
const FRONTEND_PORT = 3000;

// Isolated e2e database on the local Postgres — never the dev `kanban` DB.
const E2E_DATABASE_URL =
  process.env.E2E_DATABASE_URL ??
  "postgres://kanban:kanban@localhost:5432/kanban_e2e";

export default defineConfig({
  testDir: "./e2e",
  testMatch: /.*\.e2e\.ts/,
  fullyParallel: false,
  workers: 1,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: "list",
  timeout: 120_000,
  expect: { timeout: 15_000 },
  use: {
    baseURL: `http://localhost:${FRONTEND_PORT}`,
    trace: "on-first-retry",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: [
    {
      // Drop/create/migrate the isolated e2e DB, then boot the API against it.
      // `--no-dotenv --database-url` pins the reset to the e2e DB so the dev
      // `kanban` database can never be dropped.
      command: `cargo sqlx database reset -y --no-dotenv --database-url '${E2E_DATABASE_URL}' && cargo run`,
      cwd: "../backend",
      url: `http://localhost:${BACKEND_PORT}/api/health`,
      timeout: 180_000,
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
      stderr: "pipe",
      env: {
        APP_ENV: "development",
        PORT: String(BACKEND_PORT),
        DATABASE_URL: E2E_DATABASE_URL,
        RUST_LOG: "warn",
      },
    },
    {
      command: "pnpm dev",
      url: `http://localhost:${FRONTEND_PORT}`,
      timeout: 180_000,
      reuseExistingServer: !process.env.CI,
      env: { NEXT_PUBLIC_API_URL: `http://localhost:${BACKEND_PORT}` },
    },
  ],
});
