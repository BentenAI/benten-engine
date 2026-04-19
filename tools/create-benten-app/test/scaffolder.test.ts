// Phase 1 R3 Vitest: `npx create-benten-app <name>` headline exit criterion.
// Verifies the scaffolder produces a project where `npm install && npm test`
// exits 0 — the 10-minute DX path from docs/QUICKSTART.md.
// Status: FAILING until T1 (scaffolder) + the rest of the stack land.

import { describe, expect, it } from "vitest";
import { execSync } from "node:child_process";
import { mkdtempSync, rmSync, existsSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

describe("create-benten-app scaffolder (headline exit criterion)", () => {
  it("scaffolder_produces_working_project", () => {
    const tmp = mkdtempSync(join(tmpdir(), "benten-scaffold-"));
    try {
      // Execute the scaffolder in CI-friendly mode (no prompts, offline-ok).
      execSync("node " + join(__dirname, "..", "bin.mjs") + " my-app --skip-install", {
        cwd: tmp, stdio: "inherit",
      });

      const appDir = join(tmp, "my-app");
      expect(existsSync(appDir)).toBe(true);
      expect(existsSync(join(appDir, "package.json"))).toBe(true);
      expect(existsSync(join(appDir, "src", "handlers.ts"))).toBe(true);
      expect(existsSync(join(appDir, "test", "smoke.test.ts"))).toBe(true);

      // The smoke.test.ts file contains the six exit-criterion assertions from §1.
      const smoke = readFileSync(join(appDir, "test", "smoke.test.ts"), "utf8");
      expect(smoke).toContain("register_succeeds");
      expect(smoke).toContain("three_creates_list_returns_them");
      expect(smoke).toContain("typed_error_surface_unregistered_handler");
      expect(smoke).toContain("trace_non_zero_timing");
      expect(smoke).toContain("mermaid_output_parses");
      expect(smoke).toContain("ts_rust_cid_roundtrip");

      // Install + test roundtrip.
      execSync("npm install --silent --no-audit --no-fund", { cwd: appDir, stdio: "inherit" });
      execSync("npm test", { cwd: appDir, stdio: "inherit" });
      // `npm run build` (tsc) must also pass — the README advertises it
      // alongside `npm test` and `npm run dev`, and a prior regression
      // (missing `@types/node`) silently broke it because this harness
      // only exercised the test script. Enforce the full trio.
      execSync("npm run build", { cwd: appDir, stdio: "inherit" });
      // `npm run dev` runs main() and exits cleanly. It must not crash
      // on a freshly-generated project — the dbPath's parent directory
      // is created by Engine.open().
      execSync("npm run dev", { cwd: appDir, stdio: "inherit" });
    } finally {
      rmSync(tmp, { recursive: true, force: true });
    }
  }, 180_000); // 3-minute timeout covers npm install + test + build + dev.

  it("scaffolder_smoke_test_asserts_all_six_exit_criteria", () => {
    // Meta-test: the generated smoke.test.ts must contain exactly six top-level
    // it() blocks mapping to the six exit-criterion assertions. A regression
    // that drops one of them is caught here.
    //
    // We count only `it(` call-site occurrences (line-start whitespace +
    // `it(`) so stray mentions of `it()` inside comments don't skew the
    // count.
    const templatePath = join(__dirname, "..", "template", "test", "smoke.test.ts");
    const template = readFileSync(templatePath, "utf8");
    const itCount = (template.match(/^\s*it\s*\(/gm) ?? []).length;
    expect(itCount).toBe(6);
  });
});
