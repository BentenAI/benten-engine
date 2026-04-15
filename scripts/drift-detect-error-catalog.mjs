#!/usr/bin/env node
// @ts-check
//
// Error-catalog drift detector (T7).
//
// Parses `docs/ERROR-CATALOG.md` and compares the set of declared error
// codes against:
//
//   1. `crates/benten-core/src/error_code.rs` — hand-authored source of
//      truth for Rust. Every `### E_XXX` header in the catalog must have
//      a matching variant in the `ErrorCode` enum. Extra Rust variants
//      with no catalog entry also fail (codegen drift in the other
//      direction).
//
//   2. `packages/engine/src/errors.ts` — codegenned from the catalog
//      (per implementation plan T7, "codegen; one place to add a new
//      error, two consumers stay in sync"). Every catalog code must
//      appear as an exported symbol here.
//
// Exit codes:
//   0 = in sync
//   1 = drift detected; diff printed to stderr with explicit missing/
//       extra listings per consumer.
//   2 = structural error (catalog or source file unparseable, file
//       missing, etc.) — surfaces as a CI red flag distinct from drift.
//
// The detector is deliberately language-agnostic at the script layer so
// the same harness can validate future consumers (Python bindings,
// documentation tables, etc.) by teaching it one more parser per
// consumer.
//
// Invoked by `.github/workflows/drift-detect.yml` (owned by
// `rust-test-writer-security` per R2 file ownership) and locally via
// `node scripts/drift-detect-error-catalog.mjs`.

import { readFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(SCRIPT_DIR, "..");

const CATALOG_PATH = resolve(REPO_ROOT, "docs/ERROR-CATALOG.md");
const RUST_PATH = resolve(REPO_ROOT, "crates/benten-core/src/error_code.rs");
const TS_PATH = resolve(REPO_ROOT, "packages/engine/src/errors.ts");

// ---------------------------------------------------------------------------
// Parsers
// ---------------------------------------------------------------------------

/**
 * Extract all `### E_XXX` headers from the catalog. Matches the exact
 * heading shape documented in ERROR-CATALOG.md §Format:
 *   ### E_CAP_DENIED
 * Codes outside `E_[A-Z0-9_]+` are rejected (typo guard).
 *
 * @param {string} markdown
 * @returns {Set<string>}
 */
function parseCatalogCodes(markdown) {
  const codes = new Set();
  const rx = /^###\s+(E_[A-Z0-9_]+)\s*$/gm;
  let match;
  while ((match = rx.exec(markdown)) !== null) {
    codes.add(match[1]);
  }
  return codes;
}

/**
 * Extract enum variant identifiers from the Rust `ErrorCode` enum.
 * Matches simple `E_XXX,` or `E_XXX =` lines within the enum body. The
 * scan is conservative: only recognizes the `ErrorCode` enum block,
 * skipping doc comments and attributes.
 *
 * @param {string} rust
 * @returns {Set<string>}
 */
function parseRustCodes(rust) {
  const codes = new Set();
  // Locate the enum body: `pub enum ErrorCode { ... }`. Non-greedy match
  // on the brace body keeps this working even if additional enums are
  // declared later in the file.
  const enumMatch = rust.match(/pub\s+enum\s+ErrorCode\s*\{([\s\S]*?)\}/);
  if (!enumMatch) return codes;
  const body = enumMatch[1];
  const rx = /\b(E_[A-Z0-9_]+)\b/g;
  let m;
  while ((m = rx.exec(body)) !== null) {
    codes.add(m[1]);
  }
  return codes;
}

/**
 * Extract exported error code symbols from the TypeScript `errors.ts`
 * file. The file is codegenned as:
 *
 *   export const ErrorCode = { E_CAP_DENIED: "E_CAP_DENIED", ... } as const;
 *
 * We tolerate either the `const` object shape or a union-type shape
 * (`export type ErrorCode = "E_CAP_DENIED" | ...`). Whichever ships, the
 * parser extracts the code names.
 *
 * @param {string} ts
 * @returns {Set<string>}
 */
function parseTsCodes(ts) {
  const codes = new Set();
  const rx = /\b(E_[A-Z0-9_]+)\b/g;
  let m;
  while ((m = rx.exec(ts)) !== null) {
    codes.add(m[1]);
  }
  return codes;
}

// ---------------------------------------------------------------------------
// Diff helpers
// ---------------------------------------------------------------------------

/**
 * @param {Set<string>} a
 * @param {Set<string>} b
 * @returns {string[]}
 */
function missingFrom(a, b) {
  return [...a].filter((x) => !b.has(x)).sort();
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

function die(code, msg) {
  process.stderr.write(msg + "\n");
  process.exit(code);
}

function readOrDie(path, label) {
  if (!existsSync(path)) {
    die(
      2,
      `[drift-detect] STRUCTURAL: ${label} not found at ${path}. ` +
        `This detector runs post-R5; if you are seeing this pre-R5, the ` +
        `file has not been created yet and the detector cannot compare.`,
    );
  }
  return readFileSync(path, "utf8");
}

const catalog = readOrDie(CATALOG_PATH, "error catalog");
const catalogCodes = parseCatalogCodes(catalog);

if (catalogCodes.size === 0) {
  die(
    2,
    `[drift-detect] STRUCTURAL: zero error codes parsed from ${CATALOG_PATH}. ` +
      `Expected '### E_XXX' headers per the ERROR-CATALOG.md format.`,
  );
}

// The Rust + TS consumers may not exist yet at R3 time (they are R5
// deliverables). Soft-skip those legs with a warning rather than failing,
// so the detector CI job turns green once R5 lands without needing to be
// re-enabled. The catalog-parse itself still hard-fails.
let driftDetected = false;
const summary = [];

summary.push(`[drift-detect] catalog codes: ${catalogCodes.size}`);

if (existsSync(RUST_PATH)) {
  const rust = readFileSync(RUST_PATH, "utf8");
  const rustCodes = parseRustCodes(rust);
  const missingInRust = missingFrom(catalogCodes, rustCodes);
  const extraInRust = missingFrom(rustCodes, catalogCodes);
  summary.push(`[drift-detect] rust codes: ${rustCodes.size}`);
  if (missingInRust.length > 0) {
    driftDetected = true;
    summary.push(
      `[drift-detect] DRIFT: codes in catalog but NOT in rust:\n  - ${missingInRust.join("\n  - ")}`,
    );
  }
  if (extraInRust.length > 0) {
    driftDetected = true;
    summary.push(
      `[drift-detect] DRIFT: codes in rust but NOT in catalog:\n  - ${extraInRust.join("\n  - ")}`,
    );
  }
} else {
  summary.push(
    `[drift-detect] SKIP rust leg: ${RUST_PATH} does not exist yet (R5 deliverable).`,
  );
}

if (existsSync(TS_PATH)) {
  const ts = readFileSync(TS_PATH, "utf8");
  const tsCodes = parseTsCodes(ts);
  const missingInTs = missingFrom(catalogCodes, tsCodes);
  const extraInTs = missingFrom(tsCodes, catalogCodes);
  summary.push(`[drift-detect] ts codes: ${tsCodes.size}`);
  if (missingInTs.length > 0) {
    driftDetected = true;
    summary.push(
      `[drift-detect] DRIFT: codes in catalog but NOT in ts:\n  - ${missingInTs.join("\n  - ")}`,
    );
  }
  if (extraInTs.length > 0) {
    driftDetected = true;
    summary.push(
      `[drift-detect] DRIFT: codes in ts but NOT in catalog:\n  - ${extraInTs.join("\n  - ")}`,
    );
  }
} else {
  summary.push(
    `[drift-detect] SKIP ts leg: ${TS_PATH} does not exist yet (R5 deliverable).`,
  );
}

process.stdout.write(summary.join("\n") + "\n");

if (driftDetected) {
  process.stderr.write(
    "\n[drift-detect] FAIL: error-catalog drift detected. Update the lagging " +
      "consumer(s) to match the catalog, or update the catalog if the code " +
      "change is intentional. Both must land in the same PR.\n",
  );
  process.exit(1);
}

process.stdout.write("[drift-detect] OK — error catalog and consumers are in sync.\n");
process.exit(0);
