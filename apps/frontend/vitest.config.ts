import { defineConfig } from "vitest/config";

export default defineConfig({
  resolve: {
    alias: {
      "@": import.meta.dirname,
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./test/setup.ts"],
    include: ["**/*.{test,spec}.{ts,tsx}"],
    coverage: {
      provider: "v8",
      exclude: ["components/ui/**", "test/**", "**/*.config.*", "**/*.d.ts"],
      thresholds: {
        lines: 90,
        branches: 90,
      },
    },
  },
});
