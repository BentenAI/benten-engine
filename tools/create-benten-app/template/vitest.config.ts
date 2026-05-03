import { defineConfig } from "vitest/config";

// Mirror the workspace `packages/engine/vitest.config.ts` posture: each
// test file opens its own `Engine.open(...)` against a unique redb file;
// redb's lock model treats each Database as exclusive per process. With
// vitest 4's default thread pool, multiple files share a worker and the
// second `Engine.open(...)` call against the same file path can hit
// `redb: Database already open`. `pool: "forks"` gives each file a fresh
// process, mirroring the vitest 2.x default isolation that the smoke
// test was written against.
export default defineConfig({
  test: {
    include: ["test/**/*.test.ts"],
    testTimeout: 30_000,
    pool: "forks",
  },
});
