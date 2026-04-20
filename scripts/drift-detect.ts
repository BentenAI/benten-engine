#!/usr/bin/env -S npx tsx
// @ts-check
//
// Error-catalog drift detector (TypeScript variant, T7).
//
// Compares three sources of truth for the error catalog:
//   1. `docs/ERROR-CATALOG.md` — `### E_XXX` headings.
//   2. `crates/benten-errors/src/lib.rs` — `ErrorCode` enum + its
//      `as_str` match arms.
//   3. `packages/engine/src/errors.generated.ts` (or `errors.ts` if the
//      hand-authored variant exists) — exported BentenError subclasses.
//
// Additionally enforces an ErrorCode-variant **reachability** check
// (5d-A, high-leverage preventive tooling):
//   For every `ErrorCode::V` variant defined in `benten-errors`, there
//   must be at least one *construction site* in production Rust source
//   (`crates/*/src/**/*.rs`, excluding the definition file itself and
//   excluding test files / test-only `tests/` subtrees). A construction
//   site is either a direct `ErrorCode::V` usage that is not a bare
//   match-arm LHS in the mapper, or an upstream variant
//   (`<TypeName>::<UpstreamVariant>`) that is declared to map to V via
//   a `=> ErrorCode::V` arm AND is actually constructed somewhere.
//
//   This catches the "aspirational prose" pattern: a catalog variant
//   defined + mapped + round-tripped in tests, but never returned by any
//   production code path. A variant reachable *only* from tests is still
//   flagged as unreachable — test-firing is not production-firing.
//
//   Opt-out annotations on catalog entries:
//     `<!-- reachability: test-only -->` — allows test-only firing.
//     `<!-- reachability: ignore -->`    — skips the check entirely
//                                          (for genuine unreachable
//                                          forward-compat fallbacks).
//
// This is the canonical TS-authored detector called by
// `.github/workflows/ci.yml` at the drift-detector job. A sibling
// script at `scripts/drift-detect-error-catalog.mjs` exists for legacy
// invocation paths; both enforce the same contract and either one
// failing is a merge blocker.
//
// Exit codes:
//   0 = all three sources in sync + all variants reachable
//   1 = drift detected OR unreachable variant(s); stderr explains
//   2 = structural error (catalog unparseable, required file missing)
//
// Flags:
//   --self-test           Run an internal regression that seeds a known-
//                         unreachable variant + a known-reachable variant
//                         and confirms the reachability pass classifies
//                         them correctly. Exits 0 on pass, 1 on fail.
//   --skip-reachability   Skip the reachability pass. Escape hatch for
//                         local debugging ONLY; CI never sets this.

import { readFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(SCRIPT_DIR, "..");

const CATALOG_PATH = resolve(REPO_ROOT, "docs/ERROR-CATALOG.md");
const RUST_PATH = resolve(REPO_ROOT, "crates/benten-errors/src/lib.rs");
const TS_GEN_PATH = resolve(REPO_ROOT, "packages/engine/src/errors.generated.ts");
const TS_HAND_PATH = resolve(REPO_ROOT, "packages/engine/src/errors.ts");
const CRATES_DIR = resolve(REPO_ROOT, "crates");
// Files to exclude from production-source scanning — the enum definition
// site would otherwise self-satisfy every variant via its own as_str / from_str
// match arms.
const DEFINITION_SITE = resolve(REPO_ROOT, "crates/benten-errors/src/lib.rs");

function die(code: number, msg: string): never {
  process.stderr.write(`[drift-detect] ${msg}\n`);
  process.exit(code);
}

// ---------------------------------------------------------------------------
// Catalog + Rust + TS parsers (existing coherence check)
// ---------------------------------------------------------------------------

type CatalogParse = {
  all: Set<string>;
  phase1Required: Set<string>;
  reachabilityAnnotations: Map<string, "test-only" | "ignore">;
};

/**
 * Parse catalog codes grouped by Phase marker.
 *
 * A catalog entry is a Phase-N-scoped code when its body section (the
 * block until the next `### E_` heading) contains a line of the form
 * `- **Phase:** N` (or `Phase: N` inline). Codes without an explicit
 * Phase marker are considered Phase-1 (the default — they must already
 * be wired in the Rust enum).
 *
 * Also extracts the optional reachability annotation:
 *   `<!-- reachability: test-only -->` — allow test-only firing
 *   `<!-- reachability: ignore -->`    — skip the reachability check
 */
function parseCatalog(md: string): CatalogParse {
  const all = new Set<string>();
  const phase1Required = new Set<string>();
  const reachabilityAnnotations = new Map<string, "test-only" | "ignore">();
  const headingRx = /^###\s+(E_[A-Z0-9_]+)\s*$/gm;
  const matches: { code: string; start: number }[] = [];
  let m: RegExpExecArray | null;
  while ((m = headingRx.exec(md)) !== null) {
    matches.push({ code: m[1], start: m.index });
  }
  for (let i = 0; i < matches.length; i++) {
    const entry = matches[i];
    const next = matches[i + 1];
    const body = md.slice(entry.start, next ? next.start : md.length);
    all.add(entry.code);
    // Look for an explicit Phase tag. `- **Phase:** 2` / `Phase: 3` / etc.
    const phaseRx = /\*\*Phase:\*\*\s*(\d+)|Phase:\s*(\d+)/i;
    const pm = phaseRx.exec(body);
    const phase = pm ? Number(pm[1] ?? pm[2]) : 1;
    // TS-only codes (the DSL wrapper layer) live in the TS errors.ts
    // and never surface from Rust; they're catalog-documented for
    // consumer reference only.
    const tsOnly = /TS-only|ts-only/i.test(body);
    if (phase <= 1 && !tsOnly) {
      phase1Required.add(entry.code);
    }
    // Optional reachability annotation.
    const reachRx = /<!--\s*reachability:\s*(test-only|ignore)\s*-->/i;
    const rm = reachRx.exec(body);
    if (rm) {
      reachabilityAnnotations.set(entry.code, rm[1].toLowerCase() as
        | "test-only"
        | "ignore");
    }
    // TS-only entries are not produced by Rust; skip reachability.
    if (tsOnly) {
      reachabilityAnnotations.set(entry.code, "ignore");
    }
  }
  return { all, phase1Required, reachabilityAnnotations };
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
// Reachability pass
// ---------------------------------------------------------------------------

/**
 * Enumerate every `ErrorCode::<Variant>` arm in the enum-definition file's
 * `as_str` body. The list of variants here is the authoritative set the
 * reachability pass must check. Skips the `Unknown(_)` variant: it is the
 * forward-compat fallback and by design is never deliberately constructed
 * by engine code (see `benten-errors` crate docs).
 */
function parseErrorCodeVariants(rust: string): string[] {
  const variants = new Set<string>();
  // Grab any ErrorCode::<Variant> token that is followed by ` => "`, which
  // uniquely identifies a defining arm (not a use-site).
  const rx = /ErrorCode::([A-Z][A-Za-z0-9]+)\s*=>\s*"E_/g;
  let m: RegExpExecArray | null;
  while ((m = rx.exec(rust)) !== null) {
    variants.add(m[1]);
  }
  return [...variants].sort();
}

/**
 * For each `ErrorCode::<V>` variant, collect the list of upstream
 * `(TypeName, UpstreamVariant)` pairs that map to it via a
 * `... => ErrorCode::V` arm somewhere in production source.
 *
 * This is the "transitive construction set": if any upstream variant is
 * constructed in production code, the mapped ErrorCode variant is
 * reachable by delegation.
 *
 * The `mapperFileLines` return value records which file + line every
 * `=> ErrorCode::V` mapping arm lives on, so the construction-site
 * scanner can ignore those arm lines when checking whether an upstream
 * variant is actually *constructed* versus merely *matched*.
 */
type Alias = { typeName: string; variant: string; file: string; line: number };
function collectAliases(productionFiles: { path: string; content: string }[]): {
  aliasByCode: Map<string, Alias[]>;
  directErrorCodeArms: Set<string>; // `file:line` of bare `ErrorCode::V =>` arms
} {
  const aliasByCode = new Map<string, Alias[]>();
  const directErrorCodeArms = new Set<string>();
  // Matches:  TypeName::Variant  [optional pattern tail e.g. { .. } or (_)]  =>  ErrorCode::V
  // We intentionally allow whitespace / simple pattern bindings between the
  // variant and the fat arrow. This is a heuristic but covers every arm in
  // the current tree.
  const aliasRx =
    /(\b[A-Z][A-Za-z0-9]*(?:Error|Violation))::([A-Z][A-Za-z0-9_]*)\s*(?:\([^)]*\)|\{[^}]*\})?\s*=>\s*(?:[a-z_][\w]*::)*ErrorCode::([A-Z][A-Za-z0-9_]*)/g;
  // Matches bare `ErrorCode::V =>` arms, which appear in chained
  // match expressions (e.g., a `code()` delegator on ErrorCode itself).
  // These are not construction sites; record them so the direct-usage
  // scan can skip them.
  const directRx = /ErrorCode::([A-Z][A-Za-z0-9_]*)\s*(?:=>|,)/g;
  for (const { path: p, content } of productionFiles) {
    let m: RegExpExecArray | null;
    // Reset regex state per file.
    aliasRx.lastIndex = 0;
    while ((m = aliasRx.exec(content)) !== null) {
      const typeName = m[1];
      const variant = m[2];
      const code = m[3];
      // Ignore self-referential ErrorCode::X arms — handled below.
      if (typeName === "ErrorCode") continue;
      const line = content.slice(0, m.index).split("\n").length;
      if (!aliasByCode.has(code)) aliasByCode.set(code, []);
      aliasByCode.get(code)!.push({ typeName, variant, file: p, line });
    }
    // Mark bare `ErrorCode::X =>` lines as match-arm-only (not a
    // construction site) so the direct usage scanner ignores them.
    directRx.lastIndex = 0;
    while ((m = directRx.exec(content)) !== null) {
      // Confirm the match is actually on an arm-LHS line. A trailing `=>`
      // token confirms it (directRx also matches bare `ErrorCode::X,` in
      // static lists, which we do NOT mark — those can only appear inside
      // the definition site which is already excluded).
      const matchedText = content.slice(m.index, m.index + m[0].length);
      if (!/=>\s*$/.test(matchedText)) continue;
      const line = content.slice(0, m.index).split("\n").length;
      directErrorCodeArms.add(`${p}:${line}:${m[1]}`);
    }
  }
  return { aliasByCode, directErrorCodeArms };
}

/**
 * Strip Rust line comments (`//`, `///`, `//!`) from a source file before
 * scanning. Prevents doc-comment references to a variant
 * (e.g. `/// [`ViewError::PatternMismatch`]`) from being mistaken for a
 * construction site.
 *
 * We deliberately do NOT strip Rust block comments (slash-star to
 * star-slash) — multi-line stripping is error-prone and block comments
 * are rare in this codebase.
 * If a variant's only "construction" lives in a block comment, that's a
 * niche false-positive we can address if it ever happens.
 *
 * Line-aware: replaces comment bodies with spaces so line numbers in
 * reported evidence still match the original file.
 */
function stripRustLineComments(src: string): string {
  const lines = src.split("\n");
  return lines
    .map((line) => {
      // Find the first `//` that is not inside a string literal. For our
      // purposes the heuristic "`//` not inside `"..."`" is sufficient — we
      // never inspect string-literal contents for variant names.
      let inStr = false;
      let strCh = "";
      for (let i = 0; i < line.length - 1; i++) {
        const c = line[i];
        if (inStr) {
          if (c === "\\") {
            i++;
            continue;
          }
          if (c === strCh) inStr = false;
          continue;
        }
        if (c === '"' || c === "'") {
          inStr = true;
          strCh = c;
          continue;
        }
        if (c === "/" && line[i + 1] === "/") {
          return line.slice(0, i) + " ".repeat(line.length - i);
        }
      }
      return line;
    })
    .join("\n");
}

/**
 * Walk every Rust source file under `crates/<crate>/src/`, excluding:
 *   - the `benten-errors` `src` definition site (self-references don't count)
 *   - anything under a `tests/` or `benches/` directory
 *   - files whose basename contains `_test.rs` / `_tests.rs`
 *   - anything under `target/`
 *
 * Returns { path, content } pairs.
 */
function collectProductionFiles(cratesDir: string): {
  path: string;
  content: string;
}[] {
  const out: { path: string; content: string }[] = [];
  function walk(dir: string): void {
    let entries: string[];
    try {
      entries = readdirSync(dir);
    } catch {
      return;
    }
    for (const name of entries) {
      const full = join(dir, name);
      let st;
      try {
        st = statSync(full);
      } catch {
        continue;
      }
      if (st.isDirectory()) {
        if (name === "target" || name === "tests" || name === "benches") continue;
        walk(full);
      } else if (st.isFile() && name.endsWith(".rs")) {
        if (full === DEFINITION_SITE) continue;
        if (/\b(_test|_tests)\.rs$/.test(name)) continue;
        out.push({
          path: full,
          content: stripRustLineComments(readFileSync(full, "utf8")),
        });
      }
    }
  }
  // Only walk each crate's `src/` directory.
  let crateDirs: string[];
  try {
    crateDirs = readdirSync(cratesDir);
  } catch {
    return out;
  }
  for (const crate of crateDirs) {
    const srcDir = join(cratesDir, crate, "src");
    try {
      if (statSync(srcDir).isDirectory()) {
        walk(srcDir);
      }
    } catch {
      // crate without a src/ (README.md etc.) — skip
    }
  }
  return out;
}

/**
 * A single reachability decision.
 */
type ReachabilityResult =
  | { variant: string; status: "reachable"; evidence: string }
  | { variant: string; status: "unreachable" }
  | { variant: string; status: "skipped"; reason: string };

/**
 * Decide whether every ErrorCode variant has at least one production
 * construction site. For each variant V:
 *
 *   1. If the catalog annotates the corresponding E_XXX entry with
 *      `reachability: ignore`, skip.
 *   2. If the catalog annotates it `reachability: test-only`, also allow
 *      evidence from test files — but we don't currently expand the
 *      file-set; a test-only annotation is effectively treated as
 *      `ignore` for Phase-1 strictness. (Reserved for Phase-2 policy
 *      loosening.)
 *   3. Else search production files for:
 *        a. Any `ErrorCode::V` token that is NOT on a bare match-arm LHS
 *           line (`directErrorCodeArms` skiplist).
 *        b. Any `TypeName::UpstreamVariant` that maps to V, where the
 *           occurrence is NOT itself the mapper's LHS arm line.
 *   4. If either (a) or (b) yields ≥1 hit → reachable.
 */
function checkReachability(
  variants: string[],
  productionFiles: { path: string; content: string }[],
  catalogAnnotations: Map<string, "test-only" | "ignore">,
  variantToCode: (v: string) => string,
): ReachabilityResult[] {
  const { aliasByCode, directErrorCodeArms } = collectAliases(productionFiles);
  const results: ReachabilityResult[] = [];

  for (const variant of variants) {
    const catalogCode = variantToCode(variant);
    const annotation = catalogAnnotations.get(catalogCode);
    if (annotation === "ignore" || annotation === "test-only") {
      results.push({
        variant,
        status: "skipped",
        reason:
          annotation === "ignore"
            ? "catalog annotation: reachability: ignore"
            : "catalog annotation: reachability: test-only (Phase-1 policy treats as skip)",
      });
      continue;
    }
    // --- (a) direct `ErrorCode::V` usage outside mapper arms ------------
    const directRx = new RegExp(`\\bErrorCode::${variant}\\b`, "g");
    let directHit: { file: string; line: number } | null = null;
    for (const { path: p, content } of productionFiles) {
      directRx.lastIndex = 0;
      let m: RegExpExecArray | null;
      while ((m = directRx.exec(content)) !== null) {
        const line = content.slice(0, m.index).split("\n").length;
        if (directErrorCodeArms.has(`${p}:${line}:${variant}`)) continue;
        // Also skip when the match is part of `=> ErrorCode::V` — that's
        // an alias-map RHS, not a construction.
        const upto = content.lastIndexOf("\n", m.index);
        const lineStart = upto === -1 ? 0 : upto + 1;
        const lineEnd = content.indexOf("\n", m.index);
        const ln = content.slice(lineStart, lineEnd === -1 ? undefined : lineEnd);
        if (
          /=>\s*(?:[a-z_][\w]*::)*ErrorCode::/.test(ln) &&
          ln.indexOf("=>") < m.index - lineStart
        ) {
          // This occurrence is the RHS of an alias mapper arm, not a
          // standalone construction. (Allows a fully-qualified RHS such
          // as `=> benten_errors::ErrorCode::V`.)
          continue;
        }
        directHit = { file: p, line };
        break;
      }
      if (directHit) break;
    }
    if (directHit) {
      results.push({
        variant,
        status: "reachable",
        evidence: `direct ErrorCode::${variant} at ${rel(directHit.file)}:${directHit.line}`,
      });
      continue;
    }
    // --- (b) upstream alias constructed somewhere ------------------------
    const aliases = aliasByCode.get(variant) ?? [];
    let aliasHit: {
      typeName: string;
      variant: string;
      file: string;
      line: number;
    } | null = null;
    for (const { typeName, variant: upstream, file: mapperFile, line: mapperLine } of aliases) {
      const rx = new RegExp(`\\b${typeName}::${upstream}\\b`, "g");
      for (const { path: p, content } of productionFiles) {
        rx.lastIndex = 0;
        let m: RegExpExecArray | null;
        while ((m = rx.exec(content)) !== null) {
          const line = content.slice(0, m.index).split("\n").length;
          // Skip the mapper arm line itself — that's match-arm LHS, not
          // construction.
          if (p === mapperFile && line === mapperLine) continue;
          // Also skip any occurrence that is the LHS of a `=>` on its
          // line (i.e., appears in another match against this upstream
          // variant). We only count *construction* (Err(X), return X,
          // let foo = X, struct initialization, etc.).
          const lineStart = content.lastIndexOf("\n", m.index) + 1;
          const lineEnd = content.indexOf("\n", m.index);
          const ln = content.slice(
            lineStart,
            lineEnd === -1 ? undefined : lineEnd,
          );
          // Heuristic: if the match token is followed by ` =>` or a
          // destructuring tail `(` / `{` *then* `=>`, it's an arm LHS.
          const afterIdx = m.index - lineStart + m[0].length;
          const rest = ln.slice(afterIdx);
          if (/^\s*(?:\([^)]*\)|\{[^}]*\})?\s*=>/.test(rest)) continue;
          aliasHit = { typeName, variant: upstream, file: p, line };
          break;
        }
        if (aliasHit) break;
      }
      if (aliasHit) break;
    }
    if (aliasHit) {
      results.push({
        variant,
        status: "reachable",
        evidence: `upstream ${aliasHit.typeName}::${aliasHit.variant} constructed at ${rel(aliasHit.file)}:${aliasHit.line}`,
      });
      continue;
    }
    results.push({ variant, status: "unreachable" });
  }
  return results;
}

function rel(p: string): string {
  return p.startsWith(REPO_ROOT + "/") ? p.slice(REPO_ROOT.length + 1) : p;
}

/**
 * Build a `VariantName` → `"E_CATALOG_CODE"` lookup by re-parsing the
 * enum file. We key reachability annotations in the catalog by the
 * E_XXX form; the reachability pass iterates over Rust variant names
 * (`ValueFloatNan`), so we need the bridge.
 */
function buildVariantCodeMap(rust: string): Map<string, string> {
  const map = new Map<string, string>();
  const rx = /ErrorCode::([A-Z][A-Za-z0-9]+)\s*=>\s*"(E_[A-Z0-9_]+)"/g;
  let m: RegExpExecArray | null;
  while ((m = rx.exec(rust)) !== null) {
    map.set(m[1], m[2]);
  }
  return map;
}

// ---------------------------------------------------------------------------
// Self-test
// ---------------------------------------------------------------------------

/**
 * Seed two in-memory fixture "files" and confirm the reachability pass
 * classifies them correctly:
 *   - `ErrorCode::FooReachable` has a direct construction site.
 *   - `ErrorCode::FooUnreachable` is defined + mapped but never
 *     constructed outside a mapper arm.
 * Exits 0 on pass, 1 on fail.
 */
function runSelfTest(): never {
  const fixtureDefiner = `
match self {
    ErrorCode::FooReachable => "E_FOO_REACHABLE",
    ErrorCode::FooUnreachable => "E_FOO_UNREACHABLE",
}
`;
  const fixtureProducer: { path: string; content: string }[] = [
    {
      path: "/fixture/reachable_producer.rs",
      content: `
fn kaboom() -> EngineError {
    // Direct construction — this line should count.
    return EngineError::from(ErrorCode::FooReachable);
}

// A mapper that maps an upstream variant to FooReachable. This arm
// alone should NOT satisfy reachability (the upstream variant is not
// constructed in this fixture).
fn map(e: FakeError) -> ErrorCode {
    match e {
        FakeError::Boom => ErrorCode::FooReachable,
        FakeError::Fizz => ErrorCode::FooUnreachable,
    }
}
`,
    },
    {
      path: "/fixture/no_construction.rs",
      content: `
// This file only references the variants in match-arm LHS — not
// construction.
fn inspect(c: ErrorCode) -> &'static str {
    match c {
        ErrorCode::FooReachable => "reachable",
        ErrorCode::FooUnreachable => "unreachable",
        _ => "other",
    }
}
`,
    },
  ];
  const variantToCode = (v: string): string => {
    if (v === "FooReachable") return "E_FOO_REACHABLE";
    if (v === "FooUnreachable") return "E_FOO_UNREACHABLE";
    return `E_${v.toUpperCase()}`;
  };
  const results = checkReachability(
    parseErrorCodeVariants(fixtureDefiner),
    fixtureProducer,
    new Map(),
    variantToCode,
  );
  const byVariant = new Map(results.map((r) => [r.variant, r]));
  const reachable = byVariant.get("FooReachable");
  const unreachable = byVariant.get("FooUnreachable");
  let ok = true;
  const log: string[] = [];
  if (!reachable || reachable.status !== "reachable") {
    ok = false;
    log.push(
      `[self-test] FAIL: FooReachable expected reachable, got ${reachable?.status ?? "missing"}`,
    );
  } else {
    log.push(`[self-test] OK: FooReachable — ${reachable.evidence}`);
  }
  if (!unreachable || unreachable.status !== "unreachable") {
    ok = false;
    log.push(
      `[self-test] FAIL: FooUnreachable expected unreachable, got ${unreachable?.status ?? "missing"}${
        unreachable && unreachable.status === "reachable"
          ? ` (evidence: ${unreachable.evidence})`
          : ""
      }`,
    );
  } else {
    log.push(`[self-test] OK: FooUnreachable correctly flagged`);
  }
  process.stdout.write(log.join("\n") + "\n");
  if (!ok) {
    process.stderr.write("[self-test] FAIL\n");
    process.exit(1);
  }
  process.stdout.write("[self-test] PASS\n");
  process.exit(0);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const args = new Set(process.argv.slice(2));
if (args.has("--self-test")) {
  runSelfTest();
}
const skipReachability = args.has("--skip-reachability");

if (!existsSync(CATALOG_PATH)) {
  die(2, `catalog not found at ${CATALOG_PATH}`);
}
const { all: catalogCodes, phase1Required: catalogPhase1, reachabilityAnnotations } =
  parseCatalog(readFileSync(CATALOG_PATH, "utf8"));
if (catalogCodes.size === 0) {
  die(2, `zero codes parsed from ${CATALOG_PATH}`);
}

const summary: string[] = [];
summary.push(
  `[drift-detect] catalog codes: ${catalogCodes.size} (phase-1 required: ${catalogPhase1.size})`,
);

let drifted = false;

// ---- Rust leg -------------------------------------------------------------
let rustContent: string | null = null;
if (existsSync(RUST_PATH)) {
  rustContent = readFileSync(RUST_PATH, "utf8");
  const rustCodes = parseRust(rustContent);
  summary.push(`[drift-detect] rust codes:    ${rustCodes.size}`);
  // Only the Phase-1-required slice of the catalog must be wired in Rust
  // today; Phase-2 / Phase-3 codes are reserved in the catalog but not
  // yet implemented (`Phase: 2`/`Phase: 3` body marker).
  const missingInRust = diff(catalogPhase1, rustCodes);
  const extraInRust = diff(rustCodes, catalogCodes);
  if (missingInRust.length > 0) {
    drifted = true;
    summary.push(
      `[drift-detect] DRIFT: in catalog but not in rust:\n  - ${missingInRust.join("\n  - ")}`,
    );
  }
  if (extraInRust.length > 0) {
    drifted = true;
    summary.push(
      `[drift-detect] DRIFT: in rust but not in catalog:\n  - ${extraInRust.join("\n  - ")}`,
    );
  }
} else {
  summary.push(`[drift-detect] SKIP rust leg: ${RUST_PATH} does not exist`);
}

// ---- TS leg --------------------------------------------------------------
// Prefer errors.generated.ts (codegen output); fall back to errors.ts.
const tsPath = existsSync(TS_GEN_PATH)
  ? TS_GEN_PATH
  : existsSync(TS_HAND_PATH)
    ? TS_HAND_PATH
    : null;
if (tsPath) {
  const tsCodes = parseTs(readFileSync(tsPath, "utf8"));
  summary.push(
    `[drift-detect] ts codes:      ${tsCodes.size} (from ${tsPath.replace(REPO_ROOT + "/", "")})`,
  );
  const missingInTs = diff(catalogCodes, tsCodes);
  const extraInTs = diff(tsCodes, catalogCodes);
  if (missingInTs.length > 0) {
    drifted = true;
    summary.push(
      `[drift-detect] DRIFT: in catalog but not in ts:\n  - ${missingInTs.join("\n  - ")}`,
    );
  }
  if (extraInTs.length > 0) {
    drifted = true;
    summary.push(
      `[drift-detect] DRIFT: in ts but not in catalog:\n  - ${extraInTs.join("\n  - ")}`,
    );
  }
} else {
  summary.push(
    `[drift-detect] SKIP ts leg: neither errors.generated.ts nor errors.ts present`,
  );
}

// ---- Reachability leg ----------------------------------------------------
let unreachable: string[] = [];
if (!skipReachability && rustContent !== null) {
  const variants = parseErrorCodeVariants(rustContent);
  const variantCodeMap = buildVariantCodeMap(rustContent);
  const prodFiles = collectProductionFiles(CRATES_DIR);
  summary.push(
    `[drift-detect] reachability: scanning ${variants.length} variants across ${prodFiles.length} production files`,
  );
  const results = checkReachability(
    variants,
    prodFiles,
    reachabilityAnnotations,
    (v) => variantCodeMap.get(v) ?? `E_${v}`,
  );
  const skipped = results.filter((r) => r.status === "skipped");
  unreachable = results
    .filter((r) => r.status === "unreachable")
    .map((r) => r.variant);
  if (skipped.length > 0) {
    summary.push(
      `[drift-detect] reachability: ${skipped.length} variant(s) skipped via catalog annotation`,
    );
    for (const s of skipped) {
      if (s.status === "skipped") {
        summary.push(`  ~ ErrorCode::${s.variant} — ${s.reason}`);
      }
    }
  }
  if (unreachable.length === 0) {
    summary.push(
      `[drift-detect] reachability: ${results.filter((r) => r.status === "reachable").length} variant(s) reachable, 0 unreachable`,
    );
  } else {
    summary.push(
      `[drift-detect] reachability: ${unreachable.length} unreachable variant(s)`,
    );
    for (const v of unreachable) {
      summary.push(
        `  ! ErrorCode::${v} — no construction site in crates/*/src/ (excluding tests)`,
      );
    }
  }
} else if (skipReachability) {
  summary.push(`[drift-detect] SKIP reachability leg: --skip-reachability`);
} else {
  summary.push(
    `[drift-detect] SKIP reachability leg: rust enum file not found`,
  );
}

process.stdout.write(summary.join("\n") + "\n");

if (drifted) {
  process.stderr.write(
    "\n[drift-detect] FAIL: error-catalog drift. Update the lagging consumer(s) or update the catalog; both must land in the same commit.\n",
  );
  process.exit(1);
}
if (unreachable.length > 0) {
  process.stderr.write(
    `\n[drift-detect] FAIL: ${unreachable.length} ErrorCode variant(s) have no production construction site. ` +
      `Either add a construction path, remove the variant, or annotate the catalog entry with ` +
      `\`<!-- reachability: ignore -->\` (for genuine forward-compat fallbacks).\n`,
  );
  process.exit(1);
}

process.stdout.write(
  "[drift-detect] OK — catalog, Rust enum, and TS classes agree; every ErrorCode variant has a production construction site.\n",
);
process.exit(0);
