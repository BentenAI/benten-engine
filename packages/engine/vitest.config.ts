import { defineConfig } from "vitest/config";

// Each test file opens its own `Engine.open(":memory:")` — redb's lock
// model treats each Database as exclusive per process. With vitest's
// default thread pool, multiple files share a worker and the second
// `Engine.open(":memory:")` call hits `redb: Database already open`.
// `pool: "forks"` gives each file a fresh process, mirroring the
// vitest 2.x default isolation that the test suite was written against.
export default defineConfig({
  test: {
    pool: "forks",
  },
});
