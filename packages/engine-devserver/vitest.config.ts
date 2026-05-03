import { defineConfig } from "vitest/config";

// Mirror the workspace `packages/engine/vitest.config.ts` posture for
// process isolation. The engine-devserver tests in scope here exercise
// pure helpers + do not need the native binding loaded; future tests
// that DO load the binding will benefit from forks-pool isolation.
export default defineConfig({
  test: {
    include: ["test/**/*.test.ts"],
    pool: "forks",
  },
});
