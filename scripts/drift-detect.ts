#!/usr/bin/env -S npx tsx
// @ts-check
//
// Error-catalog drift detector (TypeScript variant, T7).
//
// Compares three sources of truth for the error catalog:
//   1. `docs/ERROR-CATALOG.md` — `### E_XXX` headings.
//   2. `crates/benten-core/src/error_code.rs` — `ErrorCode` enum + its
//      `as_str` match arms.
//   3. `packages/engine/src/errors.generated.ts` (or `errors.ts` if the
//      hand-authored variant exists) — exported BentenError subclasses.
//
// This is the canonical TS-authored detector called by
// `.github/workflows/ci.yml` at the drift-detector job. A sibling
// script at `scripts/drift-detect-error-catalog.mjs` exists for legacy
// invocation paths; both enforce the same contract and either one
// failing is a merge blocker.
//
// Exit codes:
//   0 = all three sources in sync
//   1 = drift detected; stderr lists missing/extra codes per source
//   2 = structural error (catalog unparseable, required file missing)

import { readFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(SCRIPT_DIR, "..");

const CATALOG_PATH = resolve(REPO_ROOT, "docs/ERROR-CATALOG.md");
const RUST_PATH = resolve(REPO_ROOT, "crates/benten-core/src/error_code.rs");
const TS_GEN_PATH = resolve(REPO_ROOT, "packages/engine/src/errors.generated.ts");
const TS_HAND_PATH = resolve(REPO_ROOT, "packages/engine/src/errors.ts");

function die(code: number, msg: string): never {
  process.stderr.write(`[drift-detect] ${msg}\n`);
  process.exit(code);
}

function parseCatalog(md: string): Set<string> {
  const codes = new Set<string>();
  const rx = /^###\s+(E_[A-Z0-9_]+)\s*$/gm;
  let m: RegExpExecArray | null;
  while ((m = rx.exec(md)) !== null) {
    codes.add(m[1]);
  }
  return codes;
}

/**
 * Parse the `as_str` match arms in error_code.rs. Each arm has the shape
 * `ErrorCode::<Variant> => "E_XXX",` — the RHS string is authoritative for
 * what the Rust side emits on the wire. Relying on the RHS (not the
 * variant identifier) makes this parser robust to Rust-side renames that
 * preserve the catalog code.
 */
function parseRust(rust: string): Set<string> {
  const codes = new Set<string>();
  const rx = /=>\s*"(E_[A-Z0-9_]+)"/g;
  let m: RegExpExecArray | null;
  while ((m = rx.exec(rust)) !== null) {
    codes.add(m[1]);
  }
  return codes;
}

/**
 * Extract every E_XXX literal from the TS file. Works equally well on the
 * codegenned errors.generated.ts (where codes appear as string literals
 * inside static readonly properties) and on a hand-authored errors.ts.
 */
function parseTs(ts: string): Set<string> {
  const codes = new Set<string>();
  const rx = /"(E_[A-Z0-9_]+)"/g;
  let m: RegExpExecArray | null;
  while ((m = rx.exec(ts)) !== null) {
    codes.add(m[1]);
  }
  return codes;
}

function diff(a: Set<string>, b: Set<string>): string[] {
  return [...a].filter((x) => !b.has(x)).sort();
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

if (!existsSync(CATALOG_PATH)) {
  die(2, `catalog not found at ${CATALOG_PATH}`);
}
const catalogCodes = parseCatalog(readFileSync(CATALOG_PATH, "utf8"));
if (catalogCodes.size === 0) {
  die(2, `zero codes parsed from ${CATALOG_PATH}`);
}

const summary: string[] = [];
summary.push(`[drift-detect] catalog codes: ${catalogCodes.size}`);

let drifted = false;

// ---- Rust leg -------------------------------------------------------------
if (existsSync(RUST_PATH)) {
  const rustCodes = parseRust(readFileSync(RUST_PATH, "utf8"));
  summary.push(`[drift-detect] rust codes:    ${rustCodes.size}`);
  const missingInRust = diff(catalogCodes, rustCodes);
  const extraInRust = diff(rustCodes, catalogCodes);
  if (missingInRust.length > 0) {
    drifted = true;
    summary.push(`[drift-detect] DRIFT: in catalog but not in rust:\n  - ${missingInRust.join("\n  - ")}`);
  }
  if (extraInRust.length > 0) {
    drifted = true;
    summary.push(`[drift-detect] DRIFT: in rust but not in catalog:\n  - ${extraInRust.join("\n  - ")}`);
  }
} else {
  summary.push(`[drift-detect] SKIP rust leg: ${RUST_PATH} does not exist`);
}

// ---- TS leg --------------------------------------------------------------
// Prefer errors.generated.ts (codegen output); fall back to errors.ts.
const tsPath = existsSync(TS_GEN_PATH) ? TS_GEN_PATH : existsSync(TS_HAND_PATH) ? TS_HAND_PATH : null;
if (tsPath) {
  const tsCodes = parseTs(readFileSync(tsPath, "utf8"));
  summary.push(`[drift-detect] ts codes:      ${tsCodes.size} (from ${tsPath.replace(REPO_ROOT + "/", "")})`);
  const missingInTs = diff(catalogCodes, tsCodes);
  const extraInTs = diff(tsCodes, catalogCodes);
  if (missingInTs.length > 0) {
    drifted = true;
    summary.push(`[drift-detect] DRIFT: in catalog but not in ts:\n  - ${missingInTs.join("\n  - ")}`);
  }
  if (extraInTs.length > 0) {
    drifted = true;
    summary.push(`[drift-detect] DRIFT: in ts but not in catalog:\n  - ${extraInTs.join("\n  - ")}`);
  }
} else {
  summary.push(`[drift-detect] SKIP ts leg: neither errors.generated.ts nor errors.ts present`);
}

process.stdout.write(summary.join("\n") + "\n");

if (drifted) {
  process.stderr.write(
    "\n[drift-detect] FAIL: error-catalog drift. Update the lagging consumer(s) or update the catalog; both must land in the same commit.\n",
  );
  process.exit(1);
}

process.stdout.write("[drift-detect] OK — catalog, Rust enum, and TS classes agree.\n");
process.exit(0);
