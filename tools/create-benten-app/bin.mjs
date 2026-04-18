#!/usr/bin/env node
// @ts-check
//
// create-benten-app — Phase 1 scaffolder for the 10-minute DX path.
//
// Usage:
//   npx create-benten-app <name> [--skip-install]
//
// Behavior:
//   1. Create `<name>/` under cwd (fail if it already exists).
//   2. Copy template/**  into it verbatim, substituting `{{name}}`
//      in file contents + file/directory names (simple whole-token
//      replace — no templating engine dep).
//   3. Unless `--skip-install` is set, run `npm install` inside the
//      new project.
//
// The generated project points at `@benten/engine` via the workspace
// link configured at the repo root package.json. Inside a checked-out
// Benten repo this lets the scaffolder test harness (see `test/
// scaffolder.test.ts`) install-and-test without pushing to a registry.
//
// Phase 1 exit criterion #7: this scaffolder's generated project runs
// `npm install && npm test` to green against the six exit-criterion
// assertions embedded in `template/test/smoke.test.ts`.

import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execSync } from "node:child_process";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const TEMPLATE_DIR = resolve(SCRIPT_DIR, "template");

/**
 * Resolve the `@benten/engine` dependency spec to inject into the
 * generated project's package.json. When the scaffolder is invoked
 * from INSIDE the benten-engine monorepo (detected via the presence
 * of `packages/engine/package.json` three levels up), we emit a
 * `file:` link to the local workspace copy so `npm install` in the
 * generated project resolves offline. Otherwise we fall back to a
 * pinned npm version; this is the future path for external users
 * once `@benten/engine` is published.
 */
function resolveBentenEngineSpec(targetDir) {
  const candidate = resolve(SCRIPT_DIR, "..", "..", "packages", "engine", "package.json");
  if (existsSync(candidate)) {
    const packagesEngineDir = resolve(SCRIPT_DIR, "..", "..", "packages", "engine");
    // Emit an absolute file: URL so `npm install` works regardless of
    // where the scaffolder placed the generated project (the test
    // generates into `os.tmpdir()`, users generate into arbitrary cwd).
    return `file:${packagesEngineDir}`;
  }
  // TODO(phase-2-npm-publish): pin to the published version once
  // `@benten/engine` is on the npm registry.
  return "^0.1.0";
}

function die(msg) {
  process.stderr.write(`create-benten-app: ${msg}\n`);
  process.exit(1);
}

const args = process.argv.slice(2);
if (args.length === 0 || args[0].startsWith("--")) {
  die("missing project name. Usage: npx create-benten-app <name> [--skip-install]");
}

const name = args[0];
const skipInstall = args.includes("--skip-install");

if (!/^[a-z0-9][a-z0-9-_]*$/i.test(name)) {
  die(`invalid project name '${name}' — must match /^[a-z0-9][a-z0-9-_]*$/i`);
}

const targetDir = resolve(process.cwd(), name);
if (existsSync(targetDir)) {
  die(`target directory already exists: ${targetDir}`);
}

if (!existsSync(TEMPLATE_DIR)) {
  die(`template directory missing at ${TEMPLATE_DIR} — scaffolder package is broken`);
}

const bentenEngineSpec = resolveBentenEngineSpec(targetDir);

/**
 * Recursively copy src -> dst, substituting template tokens in both
 * file contents and file/directory basenames.
 *
 * Tokens:
 *   * `{{name}}`              — the generated project's name.
 *   * `{{bentenEngineSpec}}`  — the resolved `@benten/engine` package
 *                               spec (a `file:` URL inside the monorepo,
 *                               a semver range for external users).
 */
function copyTemplate(src, dst) {
  const stat = statSync(src);
  if (stat.isDirectory()) {
    mkdirSync(dst, { recursive: true });
    for (const entry of readdirSync(src)) {
      const sub = entry.replaceAll("{{name}}", name);
      copyTemplate(join(src, entry), join(dst, sub));
    }
    return;
  }
  // Files: decide text vs binary by extension. Template is TS/MD/JSON-only
  // so this is safe; guard anyway so a future binary dropped in the template
  // doesn't get mangled.
  const textExt = /\.(ts|tsx|js|mjs|json|md|yml|yaml|txt|toml|html|css)$/i;
  if (textExt.test(src)) {
    const body = readFileSync(src, "utf8")
      .replaceAll("{{name}}", name)
      .replaceAll("{{bentenEngineSpec}}", bentenEngineSpec);
    writeFileSync(dst, body);
  } else {
    cpSync(src, dst);
  }
}

process.stdout.write(`Scaffolding ${name} into ${targetDir}\n`);
copyTemplate(TEMPLATE_DIR, targetDir);

if (!skipInstall) {
  process.stdout.write("Running npm install (use --skip-install to skip)\n");
  try {
    execSync("npm install --no-audit --no-fund", {
      cwd: targetDir,
      stdio: "inherit",
    });
  } catch (err) {
    die(`npm install failed inside ${targetDir}: ${err instanceof Error ? err.message : String(err)}`);
  }
}

process.stdout.write(`
Done. Next steps:

  cd ${name}
  npm run test      # smoke test (6 exit-criterion assertions)
  npm run dev       # start dev handler
  npm run build     # tsc

See README.md inside the project for the 10-minute quickstart.
`);
