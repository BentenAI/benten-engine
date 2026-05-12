// Vitest 4.x Browser Mode config with Playwright provider.
//
// Per orchestrator's 2026 framework-research lock (F2 brief): Browser
// Mode graduated stable late 2025 and is the 2026 default for
// component+integration testing of the admin UI v0 surface. We use the
// Playwright provider (NOT WebDriverIO) for real-headless-Chromium
// fidelity + the ~40ms HMR re-run latency Playwright provides.
//
// Vitest 4.x API: `browser.provider` is now a factory imported from
// `@vitest/browser-playwright` (the v4 split-out provider package),
// NOT the string `"playwright"` of earlier versions.
//
// Cross-origin / CSRF / Tauri-WebDriver-BiDi tests live in
// `tests/e2e/*.spec.ts` and run under standalone Playwright via
// `playwright.config.ts` — those tests need real network stacks +
// multi-origin contexts that Vitest Browser Mode does not expose.
//
// The wasm plugins (`vite-plugin-wasm` + `vite-plugin-top-level-await`)
// are listed in `package.json` devDependencies so G24-A/B/C wave-6b
// can wire them when wasm-loading tests un-ignore. At R3 RED-PHASE the
// tests are all `test.skip`'d and don't touch the wasm bundle, so the
// vitest.config.ts does NOT register the plugins yet.

import { defineConfig } from "vitest/config";
import { playwright } from "@vitest/browser-playwright";

export default defineConfig({
  test: {
    include: ["tests/**/*.test.ts"],
    exclude: ["tests/e2e/**", "node_modules/**", "dist/**"],
    browser: {
      enabled: false, // toggled on via `--browser` flag in `test:browser`
      provider: playwright(),
      // Browser Mode 4.x: `instances` replaces the older `name` field.
      instances: [
        { browser: "chromium" },
      ],
      headless: true,
    },
  },
});
