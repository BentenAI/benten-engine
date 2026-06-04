// Standalone Playwright config for the top-of-pyramid E2E tests.
//
// Scope: cross-origin / CSRF / session-token threat-model scenarios
// (T2 in `admin-ui-v0-threat-model.md`) + Tauri 2.x shape-(c)
// acceptance via `tauri-plugin-webdriver` + WebDriver BiDi.
//
// Vitest Browser Mode handles component + integration tests; Playwright
// here handles the multi-origin / cross-process scenarios that need
// real browser contexts + real network stacks. Per orchestrator's 2026
// framework-research lock (F2 brief).

import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/e2e",
  fullyParallel: true,
  // Force CI-mode reporter when running in GitHub Actions; HTML report
  // is the local-dev default.
  reporter: process.env.CI ? "github" : "html",
  use: {
    // Each test names its own baseURL; T2 scenarios use multiple
    // origins so this stays unset at the top-level.
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    // WebKit + Firefox projects intentionally omitted at R3; G24-B
    // wave-6b adds them once the production thin-client serving
    // boundary exists. The single chromium project is enough to pin
    // the cross-origin / CSRF defenses at R3 RED-PHASE shape.
  ],
});
