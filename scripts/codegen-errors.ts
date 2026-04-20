#!/usr/bin/env -S npx tsx
// @ts-check
//
// Codegen: ERROR-CATALOG.md -> packages/engine/src/errors.generated.ts
//
// The catalog is the single source of truth. This script parses every
// `### E_XXX` heading + its surrounding metadata (Message template,
// Context fields, Fix hint, Thrown at) and emits one exported TypeScript
// class per code, each extending a shared `BentenError` base class and
// carrying a static `code` + `fixHint` property. The drift-detect script
// asserts the generated file is in sync with both the catalog and the
// Rust enum.
//
// The Rust enum at `crates/benten-errors/src/lib.rs` is ALREADY
// hand-authored (and every workspace crate depends on its exact shape).
// We deliberately DO NOT regenerate it — the drift detector's job is to surface
// divergence so a human reconciles it. Mechanical regeneration of both
// sides defeats the purpose of the detector. (The G8-C brief lists the
// Rust output as "optionally" generated; we take "no" here.)
//
// Usage:
//   npx tsx scripts/codegen-errors.ts
//
// Exit codes:
//   0 = generated file is up to date or was regenerated cleanly
//   1 = parse error in ERROR-CATALOG.md (e.g. missing fix hint for a code)
//   2 = structural error (catalog file missing, unwritable output, etc.)

import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(SCRIPT_DIR, "..");

const CATALOG_PATH = resolve(REPO_ROOT, "docs/ERROR-CATALOG.md");
const TS_OUT_PATH = resolve(REPO_ROOT, "packages/engine/src/errors.generated.ts");

interface CatalogEntry {
  code: string;
  message: string;
  fixHint: string;
  thrownAt: string;
}

function die(exit: number, msg: string): never {
  process.stderr.write(`[codegen-errors] ${msg}\n`);
  process.exit(exit);
}

function parseCatalog(markdown: string): CatalogEntry[] {
  // Split by top-level section headers (`### E_XXX`). Each slice contains
  // the section body until the next `###` (or EOF). We then extract
  // message / fix hint / thrown-at from the bullet list.
  const entries: CatalogEntry[] = [];
  const sectionRx = /^###\s+(E_[A-Z0-9_]+)\s*$/gm;
  const matches = [...markdown.matchAll(sectionRx)];

  for (let i = 0; i < matches.length; i++) {
    const m = matches[i];
    const code = m[1];
    const start = (m.index ?? 0) + m[0].length;
    const end = i + 1 < matches.length ? (matches[i + 1].index ?? markdown.length) : markdown.length;
    const body = markdown.slice(start, end);

    const msg = extractBullet(body, "Message");
    const fix = extractBullet(body, "Fix");
    const thrown = extractBullet(body, "Thrown at");

    if (!msg || !fix) {
      die(1, `catalog entry ${code} missing Message or Fix bullet. Body was:\n${body}`);
    }

    entries.push({ code, message: msg, fixHint: fix, thrownAt: thrown ?? "unspecified" });
  }

  return entries;
}

function extractBullet(body: string, label: string): string | null {
  // Matches lines like `- **Message:** "..."` or `- **Fix:** ...`. The
  // value extends until the next `- **` bullet or blank line.
  const rx = new RegExp(`\\-\\s+\\*\\*${label}:?\\*\\*\\s+(.+?)(?=\\n\\-\\s+\\*\\*|\\n\\n|$)`, "s");
  const m = body.match(rx);
  if (!m) return null;
  return m[1].trim().replace(/\s+/g, " ");
}

/**
 * Convert `E_CAP_DENIED` -> `ECapDenied` (PascalCase class name).
 */
function toClassName(code: string): string {
  // E_CAP_DENIED -> [E, CAP, DENIED] -> ECapDenied
  return code
    .split("_")
    .map((part, idx) => {
      if (idx === 0) return part; // keep `E` as-is
      return part.charAt(0).toUpperCase() + part.slice(1).toLowerCase();
    })
    .join("");
}

function jsStringLiteral(s: string): string {
  return JSON.stringify(s);
}

function generateTs(entries: CatalogEntry[]): string {
  const header = `// AUTO-GENERATED from docs/ERROR-CATALOG.md by scripts/codegen-errors.ts.
// DO NOT EDIT BY HAND. Run \`npx tsx scripts/codegen-errors.ts\` to regenerate.
//
// Each error class below corresponds to one \`### E_XXX\` entry in the
// catalog. The class carries a static \`code\`, a static \`fixHint\`, and
// exposes them as instance properties so \`err.code\` / \`err.fixHint\`
// work on any thrown instance. The drift-detect script asserts this
// file stays in sync with the catalog and the Rust \`ErrorCode\` enum
// at \`crates/benten-errors/src/lib.rs\`.

/* eslint-disable @typescript-eslint/no-unused-vars */

export class BentenError extends Error {
  /** Stable catalog code (e.g. "E_CAP_DENIED"). */
  readonly code: string;
  /** Human-readable fix hint from the catalog. */
  readonly fixHint: string;
  /** Optional structured context attached at throw site. */
  readonly context?: Record<string, unknown>;

  constructor(code: string, fixHint: string, message: string, context?: Record<string, unknown>) {
    super(message);
    this.name = "BentenError";
    this.code = code;
    this.fixHint = fixHint;
    this.context = context;
  }

  override toString(): string {
    return \`\${this.name} [\${this.code}]: \${this.message}\\n  fix: \${this.fixHint}\`;
  }
}

/** Exhaustive list of catalog codes, for parity checks and narrowing. */
export const CATALOG_CODES = [
${entries.map((e) => `  ${JSON.stringify(e.code)},`).join("\n")}
] as const;

export type CatalogCode = (typeof CATALOG_CODES)[number];

`;

  const classes = entries
    .map((e) => {
      const cls = toClassName(e.code);
      const code = jsStringLiteral(e.code);
      const fix = jsStringLiteral(e.fixHint);
      return `/**
 * ${e.code}
 *
 * Thrown at: ${e.thrownAt}
 * Message template: ${e.message.replace(/\*\//g, "* /")}
 */
export class ${cls} extends BentenError {
  static readonly code = ${code};
  static readonly fixHint = ${fix};
  constructor(message: string, context?: Record<string, unknown>) {
    super(${code}, ${fix}, message, context);
    this.name = ${JSON.stringify(cls)};
  }
}
`;
    })
    .join("\n");

  return header + classes;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

if (!existsSync(CATALOG_PATH)) {
  die(2, `catalog not found at ${CATALOG_PATH}`);
}

const catalog = readFileSync(CATALOG_PATH, "utf8");
const entries = parseCatalog(catalog);

if (entries.length === 0) {
  die(1, `no error codes parsed from ${CATALOG_PATH}. Expected '### E_XXX' headers.`);
}

const ts = generateTs(entries);

mkdirSync(dirname(TS_OUT_PATH), { recursive: true });
writeFileSync(TS_OUT_PATH, ts);

process.stdout.write(`[codegen-errors] wrote ${entries.length} error classes -> ${TS_OUT_PATH}\n`);
process.exit(0);
