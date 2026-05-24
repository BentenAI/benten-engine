#!/usr/bin/env -S npx tsx
// @ts-check
//
// Integration smoke-test for the §3.5g item 6 enforcer
// (`scripts/drift-detect-error-variant-mirror.ts`).
//
// This file delegates to the scanner's `--self-test` flag — the scanner
// ships an internal regression that seeds a synthetic missing-mirror
// variant + a known-OK variant + an opted-out variant and confirms the
// pipeline classifies them correctly. We invoke that pathway here so
// the test fixture is discoverable as a sibling `*.test.ts` file (the
// repo convention) + can be wired into a future `vitest` reachability
// suite without rewriting the fixture inside this file.
//
// Run via:
//   node --import tsx scripts/drift-detect-error-variant-mirror.test.ts
// Exit code 0 = self-test passed; 1 = self-test failed.

import { spawnSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const SCANNER = resolve(SCRIPT_DIR, "drift-detect-error-variant-mirror.ts");

const result = spawnSync("node", ["--import", "tsx", SCANNER, "--self-test"], {
  stdio: "inherit",
});

if (result.status === 0) {
  process.stdout.write(
    "[drift-detect-error-variant-mirror.test] OK — self-test passed (scanner correctly classifies " +
      "known-OK / known-violation / opted-out fixtures).\n",
  );
  process.exit(0);
} else {
  process.stderr.write(
    "[drift-detect-error-variant-mirror.test] FAIL — self-test exited non-zero. " +
      "The scanner's classification of synthetic fixtures regressed — investigate before " +
      "trusting the scanner's verdict on the real codebase.\n",
  );
  process.exit(1);
}
