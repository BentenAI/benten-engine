#!/usr/bin/env -S npx tsx
// @ts-check
//
// **§3.5g item 6 — pub error-variant first-class catalog mirror enforcer.**
//
// Pre-G-CORE-9-FREEZE 2026-05-24 scanner (Ben-ratified). Enforces the
// `feedback_pub_error_variant_first_class_mirror.md` rule (§3.5g item 6
// amendment): every variant of every public Rust error type that is
// constructed in a `pub fn` (or transitively reachable from one) outside
// `#[cfg(test)]` MUST have a first-class `benten_errors::ErrorCode`
// catalog entry, mapped via the type's `code()` (or equivalent) method.
//
// Pre-this-scanner, the routing-correctness obligation lived in human
// reviewer memory. The originating instance:
//
//   `benten_dsl_compiler::CompileError::Io` was a pre-existing `pub`
//   variant that surfaced from `compile_file` (a `pub fn`) but had no
//   `benten_errors::ErrorCode` entry. PR #1339 chunk-3 added `Backend`
//   as a first-class catalog code (E_DSL_BACKEND_REJECTED) but left
//   `Io` mapping through `ErrorCode::Unknown(_)`, so the napi
//   `mapNativeError` boundary collapsed to `E_UNKNOWN`.
//
// The fix-up bundle closes that origin (mints `ErrorCode::DslIoError`).
// THIS scanner catches the next such asymmetry automatically.
//
// # Scanner design
//
// 1. Discover every `pub enum *Error` (configurable suffix) declared in
//    `crates/*/src/**/*.rs` (excluding tests + the definition site for
//    `benten_errors` itself + tests/benches/target).
// 2. For each enum, enumerate variants (top-level `Variant(_)` /
//    `Variant { ... }` / unit `Variant,` shapes).
// 3. For each variant, locate construction sites: `TypeName::Variant(`
//    / `TypeName::Variant {` / `TypeName::Variant ,` across `crates/*/src/`
//    excluding tests / cfg(test) gated functions / the definition file
//    itself. (Definition-file matches are mostly match arms in the
//    type's own `Display` / `error_code` / `code` methods.)
// 4. For each variant with ≥1 production construction site that is
//    reachable from a `pub fn` (heuristic: the construction site lives
//    in a `pub fn` body in the same file, or the enum variant itself
//    is wrapped by a `pub fn` constructor — e.g. `pub fn io(...)` for
//    `CompileError::Io`), ASSERT one of:
//      (a) the variant maps to a `benten_errors::ErrorCode::<Code>`
//          arm via the type's `code()` method (or equivalent — the
//          existing `drift-detect.ts` reachability scanner enumerates
//          the `(TypeName, Variant) => ErrorCode::Code` arms),
//      (b) the variant has an inline `// drift-detect: ...` annotation
//          opting out (named cases below).
//
// # Opt-out annotations
//
// Per-variant, on the line of the variant declaration:
//   `// drift-detect: wrapped-at-napi-by E_OUTER`
//     The variant is wrapped at the napi boundary into a SEPARATE
//     first-class ErrorCode; the WRAPPER is the catalog entry.
//   `// drift-detect: internal-only`
//     The variant is internal (constructed only for in-crate logic,
//     never crosses a public/napi/wire surface). Use sparingly; the
//     §3.5g item 6 rule's whole point is to catch asymmetries.
//   `// drift-detect: ignore` (sibling of the catalog reachability
//     annotation) — generic escape hatch with a comment explaining why.
//
// # Per-enum opt-out
//
// On the `pub enum ...` declaration line, an inline
//   `// drift-detect-mirror: ignore`
// comment skips the ENTIRE enum (e.g. parsing/utility errors that
// genuinely never cross a public surface; doc-cite the reason).
//
// # Exit codes
//
//   0 — every pub error variant either has a first-class catalog mirror
//       or carries a named opt-out annotation.
//   1 — at least one variant violates the §3.5g item 6 rule. stderr
//       explains which variant + which file/line + what catalog entry
//       to mint.
//   2 — structural error (missing source tree, etc.).
//
// # Flags
//
//   --self-test
//     Internal regression: seeds a known-missing-mirror variant + a
//     known-OK variant and confirms the scanner classifies them
//     correctly. Exits 0 on pass, 1 on fail.

import { readFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { resolve, dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(SCRIPT_DIR, "..");
const CRATES_DIR = resolve(REPO_ROOT, "crates");
const ERRORS_DEFINITION_SITE = resolve(REPO_ROOT, "crates/benten-errors/src/lib.rs");
const BASELINE_PATH = resolve(
  SCRIPT_DIR,
  "drift-detect-error-variant-mirror-baseline.txt",
);

// The enum-name pattern: by default we scan every `pub enum *Error`.
// Configurable via the `BENTEN_DRIFT_DETECT_MIRROR_PATTERN` env var if a
// future workspace adds error types with a different suffix convention.
const ENUM_NAME_RX = new RegExp(
  process.env.BENTEN_DRIFT_DETECT_MIRROR_PATTERN ?? "^[A-Z][A-Za-z0-9]*Error$",
);

type EnumDecl = {
  typeName: string;
  file: string;
  line: number;
  variants: VariantDecl[];
  enumIgnored: boolean;
};

type VariantDecl = {
  name: string;
  file: string;
  line: number;
  annotation: VariantAnnotation;
};

type VariantAnnotation =
  | { kind: "none" }
  | { kind: "wrapped-at-napi-by"; wrapperCode: string }
  | { kind: "internal-only" }
  | { kind: "ignore" };

function die(code: number, msg: string): never {
  process.stderr.write(`[drift-detect-mirror] ${msg}\n`);
  process.exit(code);
}

/**
 * Load the grandfathering baseline file. Returns the set of
 * `TypeName::Variant` strings that are pre-existing-known violations
 * the scanner SHOULD NOT fail on (it still reports them as `skipped`
 * with the `baseline` reason — visible in the per-run summary so drift
 * detection ratchets in the right direction).
 *
 * The baseline lives at `scripts/drift-detect-error-variant-mirror-baseline.txt`
 * — one `TypeName::Variant` per line, `#` line comments + blank lines
 * allowed. See the file's header for the remediation contract.
 */
function loadBaseline(path: string): Set<string> {
  if (!existsSync(path)) return new Set();
  const raw = readFileSync(path, "utf8");
  const out = new Set<string>();
  for (const rawLine of raw.split("\n")) {
    const line = rawLine.trim();
    if (line === "" || line.startsWith("#")) continue;
    out.add(line);
  }
  return out;
}

// ---------------------------------------------------------------------------
// File discovery
// ---------------------------------------------------------------------------

type RustFile = { path: string; content: string };

function collectProductionFiles(cratesDir: string): RustFile[] {
  const out: RustFile[] = [];
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
        if (/\b(_test|_tests)\.rs$/.test(name)) continue;
        out.push({
          path: full,
          content: readFileSync(full, "utf8"),
        });
      }
    }
  }
  let crateDirs: string[];
  try {
    crateDirs = readdirSync(cratesDir);
  } catch {
    return out;
  }
  for (const crate of crateDirs) {
    const srcDir = join(cratesDir, crate, "src");
    try {
      if (statSync(srcDir).isDirectory()) walk(srcDir);
    } catch {
      // skip
    }
  }
  return out;
}

// ---------------------------------------------------------------------------
// pub enum *Error discovery + variant enumeration
// ---------------------------------------------------------------------------

/**
 * Find every `pub enum <Name>Error` declaration in the source set.
 * Captures the type name + file + line + per-variant declarations
 * (top-level lines inside the enum body until the matching close brace).
 *
 * Heuristic body-extraction: read characters from after the `{` of the
 * `pub enum` declaration; track depth; on depth-0 close brace, stop.
 * Inside the body, each top-level line whose first non-whitespace token
 * matches `[A-Z][A-Za-z0-9_]*` (variant identifier) is treated as a
 * variant declaration. Doc/comment lines + attribute lines are skipped.
 */
function discoverEnums(files: RustFile[]): EnumDecl[] {
  const out: EnumDecl[] = [];
  // The `pub enum` form (with optional doc-cmt attributes preceding handled
  // by line-by-line scan, but match the declaration token itself directly).
  const declRx = /^(\s*)pub\s+enum\s+([A-Z][A-Za-z0-9]*)\s*(?:<[^>]+>)?\s*\{/gm;
  for (const { path: filePath, content } of files) {
    declRx.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = declRx.exec(content)) !== null) {
      const typeName = m[2];
      if (!ENUM_NAME_RX.test(typeName)) continue;
      // Find the line number of the declaration.
      const declLine = content.slice(0, m.index).split("\n").length;
      // Check for the per-enum ignore annotation on the declaration line
      // OR the immediately preceding line.
      const lines = content.split("\n");
      const declLineText = lines[declLine - 1] ?? "";
      const prevLineText = lines[declLine - 2] ?? "";
      const enumIgnored =
        /\/\/\s*drift-detect-mirror:\s*ignore/i.test(declLineText) ||
        /\/\/\s*drift-detect-mirror:\s*ignore/i.test(prevLineText);

      // Walk the body to enumerate variants.
      const bodyStart = m.index + m[0].length; // just past the `{`
      let depth = 1;
      let i = bodyStart;
      while (i < content.length && depth > 0) {
        const c = content[i];
        if (c === "{") depth++;
        else if (c === "}") {
          depth--;
          if (depth === 0) break;
        }
        i++;
      }
      const body = content.slice(bodyStart, i);
      const variants: VariantDecl[] = [];
      // Tokenize the body by line; for each non-empty / non-comment /
      // non-attribute line, if the first identifier is an UpperCamelCase
      // token followed by `(`, `{`, `,`, or whitespace-then-end, treat
      // it as a variant. Skip lines inside `(` / `{` blocks via a depth
      // counter so payload braces don't trip us.
      const bodyLineOffset = content.slice(0, bodyStart).split("\n").length;
      let payloadDepth = 0;
      let lineIdx = 0;
      const bodyLines = body.split("\n");
      for (const rawLine of bodyLines) {
        const lineNum = bodyLineOffset + lineIdx;
        lineIdx++;
        // Adjust payloadDepth based on net `{` / `(` minus `}` / `)`
        // BEFORE deciding variant-status (so a line opening a struct
        // payload counts itself as the variant line; subsequent lines
        // are inside the payload).
        const startDepth = payloadDepth;
        for (const ch of rawLine) {
          if (ch === "{" || ch === "(") payloadDepth++;
          else if (ch === "}" || ch === ")") payloadDepth--;
        }
        if (startDepth > 0) continue;
        const trimmed = rawLine.trim();
        if (trimmed === "") continue;
        if (trimmed.startsWith("//")) continue;
        if (trimmed.startsWith("/*") || trimmed.startsWith("*")) continue;
        if (trimmed.startsWith("#[")) continue;
        if (trimmed.startsWith("#![")) continue;
        // Variant identifier as first token.
        const varRx = /^([A-Z][A-Za-z0-9_]*)\s*(?:\(|\{|,|=>|$|\s)/;
        const vm = varRx.exec(trimmed);
        if (!vm) continue;
        const variantName = vm[1];
        // Trailing comment on the same line (the conventional annotation
        // location).
        const trailingCommentRx = /\/\/\s*(.*)$/;
        const tc = trailingCommentRx.exec(rawLine);
        const annotation: VariantAnnotation = parseAnnotation(tc ? tc[1] : "");
        variants.push({
          name: variantName,
          file: filePath,
          line: lineNum,
          annotation,
        });
      }

      out.push({
        typeName,
        file: filePath,
        line: declLine,
        variants,
        enumIgnored,
      });
    }
  }
  return out;
}

function parseAnnotation(commentBody: string): VariantAnnotation {
  const wrapRx = /drift-detect:\s*wrapped-at-napi-by\s+(E_[A-Z0-9_]+)/i;
  const wm = wrapRx.exec(commentBody);
  if (wm) return { kind: "wrapped-at-napi-by", wrapperCode: wm[1] };
  if (/drift-detect:\s*internal-only/i.test(commentBody)) {
    return { kind: "internal-only" };
  }
  if (/drift-detect:\s*ignore/i.test(commentBody)) {
    return { kind: "ignore" };
  }
  return { kind: "none" };
}

// ---------------------------------------------------------------------------
// Construction-site discovery — for a given (TypeName, Variant), is it
// constructed in any production (non-#[cfg(test)]) `pub fn`?
// ---------------------------------------------------------------------------

/**
 * Heuristic check: for the given variant, find construction sites of the
 * form `TypeName::Variant(` or `TypeName::Variant {` or `TypeName::Variant,`
 * — INCLUDING a unit variant immediately followed by `)`, the shape of the
 * combinator-argument idioms `.ok_or(TypeName::Variant)` /
 * `.map_err(|_| TypeName::Variant)` / `Err(TypeName::Variant)` (the `\b`
 * word-boundary handles the leading `(` / `| ` / `Err(`). Without `)` in the
 * follow-set a unit variant constructed ONLY via these combinators reads as
 * "no production construction" and the §3.5g mirror requirement silently
 * never fires. Filter out lines that look like match-arm LHS (followed by
 * `=>`) — those are dispatchers, not constructors.
 *
 * A construction-site is "production-reachable" if its line is NOT inside
 * a `#[cfg(test)]`-gated function. We use a coarse line-window check: walk
 * upward from the construction site and ensure we don't cross a function
 * boundary that begins with `#[cfg(test)]` on the preceding non-empty
 * non-doc-comment line.
 */
function isVariantConstructedInProduction(
  typeName: string,
  variant: string,
  files: RustFile[],
): { found: boolean; sites: { file: string; line: number }[] } {
  // Match construction OR match-arm. We distinguish by post-context.
  const ctorRx = new RegExp(
    `\\b${typeName}::${variant}\\s*(?:\\(|\\)|\\{|,|;|\\s|$)`,
    "g",
  );
  const sites: { file: string; line: number }[] = [];
  for (const { path: filePath, content } of files) {
    ctorRx.lastIndex = 0;
    let m: RegExpExecArray | null;
    const lines = content.split("\n");
    while ((m = ctorRx.exec(content)) !== null) {
      const lineNum = content.slice(0, m.index).split("\n").length;
      const lineText = lines[lineNum - 1] ?? "";
      // Skip match-arm LHS (followed by `=>`).
      const tail = lineText.slice(lineText.indexOf(`${typeName}::${variant}`));
      if (/=>/.test(tail) && !/[;=]\s*(?:Self|.+?)::/.test(tail)) {
        // Only skip if `=>` appears AFTER the variant identifier on the
        // same line AND it isn't preceded by an assignment / return /
        // semicolon that would indicate the variant is the value of a
        // closure body. The simple rule: if the line has `=>` AFTER our
        // variant, it's an arm.
        continue;
      }
      // Skip lines inside #[cfg(test)] modules. Walk upward and look for
      // `#[cfg(test)]` immediately preceding a `mod tests {` or `fn` /
      // `impl`. Cheap heuristic: if the file has a `#[cfg(test)]` block
      // marker and our line is within it.
      if (isLineInCfgTestBlock(lines, lineNum)) continue;
      // Skip doc-comment lines (`///` / `//!`).
      const trimmed = lineText.trimStart();
      if (trimmed.startsWith("///") || trimmed.startsWith("//!")) continue;
      // Skip lines that are themselves comments (line comments inside code).
      // Find the first non-string-literal `//` and skip if our match was
      // after it.
      const commentIdx = findLineCommentIndex(lineText);
      if (commentIdx !== -1) {
        const matchColInLine =
          m.index - content.lastIndexOf("\n", m.index - 1) - 1;
        if (matchColInLine >= commentIdx) continue;
      }
      sites.push({ file: filePath, line: lineNum });
    }
  }
  return { found: sites.length > 0, sites };
}

function findLineCommentIndex(line: string): number {
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
    if (c === "/" && line[i + 1] === "/") return i;
  }
  return -1;
}

function isLineInCfgTestBlock(lines: string[], lineNum: number): boolean {
  // Heuristic: scan upward from the target line. Track brace depth
  // (only `{` / `}` outside of string literals — close enough for our
  // crates). If we encounter `#[cfg(test)]` before crossing depth 0 to
  // -1 (the enclosing item header), we're inside a cfg(test) block.
  // Otherwise we're not.
  let depth = 0;
  for (let i = lineNum - 1; i >= 0; i--) {
    const line = lines[i];
    // Count braces on this line, ignoring string contents — coarse but
    // adequate for production source.
    for (const ch of line) {
      if (ch === "}") depth++;
      else if (ch === "{") depth--;
    }
    // If the line begins with `#[cfg(test)]` we found the marker.
    if (/^\s*#\[cfg\(test\)\]/.test(line)) {
      // The marker applies to the next item. If our line is below it AND
      // we're still inside an item it opened, return true.
      // Simplification: assume the marker applies if it's before our line
      // (we won't see it after walking upward beyond the relevant item).
      return true;
    }
    // If we crossed the top-of-file or another `pub fn` / `fn` / `impl` /
    // `mod` header at depth 0, we're out of the cfg(test) scope.
    if (depth <= 0 && /^\s*(pub\s+)?(fn|impl|mod|struct|enum)\b/.test(line)) {
      return false;
    }
  }
  return false;
}

// ---------------------------------------------------------------------------
// Catalog-mapper discovery — does the type have a method that maps the
// variant to a `benten_errors::ErrorCode::<Code>` arm?
// ---------------------------------------------------------------------------

/**
 * For (TypeName, Variant), check whether ANY `(TypeName | Self)::Variant`
 * match-arm RHS in production source contains `benten_errors::ErrorCode::<Name>`
 * or `ErrorCode::<Name>` (where `<Name>` is a real catalog variant — not
 * `Unknown(_)`). Returns the matched ErrorCode variant name on hit.
 *
 * We accept both `TypeName::Variant` and `Self::Variant` arm shapes (Rust
 * `impl` blocks commonly use `Self`). The arm RHS span ends at the first
 * top-level `,` (closing the arm) — coarse but adequate; sequential code
 * inside `=> { ... }` blocks is bounded by the first comma we see at
 * brace-depth 0 outside the arrow body's open brace.
 */
function findCatalogMirrorFor(
  typeName: string,
  variant: string,
  files: RustFile[],
): { matched: boolean; errorCode?: string; site?: string } {
  // Build a regex that matches arm-LHS for either `TypeName::Variant` or
  // `Self::Variant`. Capture the rest of the line (and a small lookahead
  // window) to scan the RHS for `ErrorCode::<Code>`.
  const armRx = new RegExp(
    `(?:${typeName}|Self)::${variant}\\s*(?:\\([^)]*\\)|\\{[^}]*\\})?\\s*=>\\s*([^,\\n]+)`,
    "g",
  );
  // We also accept multi-line block bodies — those are wrapped by `{ ... }`.
  // For the common case of one-line arms in `code()` / `error_code()` the
  // single-line regex suffices.
  const codeRx = /(?:benten_errors::)?ErrorCode::([A-Z][A-Za-z0-9_]*)/;
  for (const { path: filePath, content } of files) {
    armRx.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = armRx.exec(content)) !== null) {
      const rhs = m[1];
      const cm = codeRx.exec(rhs);
      if (!cm) continue;
      const code = cm[1];
      // Skip the `Unknown(_)` wrap — that's the fallback that the §3.5g
      // item 6 rule is specifically trying to catch.
      if (code === "Unknown") continue;
      const lineNum = content.slice(0, m.index).split("\n").length;
      return {
        matched: true,
        errorCode: code,
        site: `${filePath}:${lineNum}`,
      };
    }
  }
  return { matched: false };
}

// ---------------------------------------------------------------------------
// Per-variant verdict
// ---------------------------------------------------------------------------

type Verdict =
  | { kind: "ok"; reason: string }
  | { kind: "skipped"; reason: string }
  | { kind: "violation"; reason: string; sites: { file: string; line: number }[] };

function judgeVariant(
  enumDecl: EnumDecl,
  variant: VariantDecl,
  files: RustFile[],
  baseline: Set<string> = new Set(),
): Verdict {
  // Per-enum opt-out.
  if (enumDecl.enumIgnored) {
    return {
      kind: "skipped",
      reason: `enum ${enumDecl.typeName} carries // drift-detect-mirror: ignore`,
    };
  }
  // Per-variant opt-out.
  switch (variant.annotation.kind) {
    case "wrapped-at-napi-by":
      return {
        kind: "skipped",
        reason: `// drift-detect: wrapped-at-napi-by ${variant.annotation.wrapperCode}`,
      };
    case "internal-only":
      return { kind: "skipped", reason: "// drift-detect: internal-only" };
    case "ignore":
      return { kind: "skipped", reason: "// drift-detect: ignore" };
    case "none":
      break;
  }
  // Check production-reachable construction.
  const ctor = isVariantConstructedInProduction(
    enumDecl.typeName,
    variant.name,
    files,
  );
  if (!ctor.found) {
    // No production construction — variant is internal/test-only; the
    // §3.5g item 6 rule has no obligation. (The separate
    // `drift-detect.ts` reachability scan would flag this if the
    // variant is genuinely unreachable; this scanner's scope is the
    // mirror-or-opt-out question.)
    return {
      kind: "skipped",
      reason: "no production construction site (not in scope for §3.5g item 6)",
    };
  }
  // Construction site exists. Demand a catalog mirror.
  const mirror = findCatalogMirrorFor(enumDecl.typeName, variant.name, files);
  if (mirror.matched) {
    return {
      kind: "ok",
      reason: `mirrored to ErrorCode::${mirror.errorCode} at ${relative(REPO_ROOT, mirror.site!)}`,
    };
  }
  // Check baseline (grandfathered pre-2026-05-24 violations).
  const baselineKey = `${enumDecl.typeName}::${variant.name}`;
  if (baseline.has(baselineKey)) {
    return {
      kind: "skipped",
      reason: `grandfathered by drift-detect-error-variant-mirror-baseline.txt (pre-2026-05-24 §3.5g item 6 ratification; remediation tracked in .addl/phase-4-meta/PRE-FREEZE-BACKLOG.md)`,
    };
  }
  return {
    kind: "violation",
    reason:
      `${enumDecl.typeName}::${variant.name} is constructed in production ` +
      `(see sites below) but has no first-class benten_errors::ErrorCode ` +
      `mirror via a (TypeName | Self)::${variant.name} => ErrorCode::* arm. ` +
      `Add a catalog entry + ErrorCode mapping (§3.5g item 6) OR annotate ` +
      `the variant with // drift-detect: <reason> OR (genuinely-blocking ` +
      `cases only) append "${baselineKey}" to ` +
      `scripts/drift-detect-error-variant-mirror-baseline.txt.`,
    sites: ctor.sites,
  };
}

// ---------------------------------------------------------------------------
// Self-test fixtures
// ---------------------------------------------------------------------------

function runSelfTest(): never {
  // We synthesize 2 in-memory RustFile fixtures + 1 enum and run the
  // judgement pipeline directly (bypassing collectProductionFiles).
  const fixtureEnum: EnumDecl = {
    typeName: "FooError",
    file: "/virtual/lib.rs",
    line: 1,
    enumIgnored: false,
    variants: [
      { name: "Ok", file: "/virtual/lib.rs", line: 2, annotation: { kind: "none" } },
      {
        name: "Missing",
        file: "/virtual/lib.rs",
        line: 3,
        annotation: { kind: "none" },
      },
      {
        name: "OptedOut",
        file: "/virtual/lib.rs",
        line: 4,
        annotation: { kind: "internal-only" },
      },
    ],
  };
  const fixtureFiles: RustFile[] = [
    {
      path: "/virtual/lib.rs",
      content: `
pub enum FooError {
    Ok,
    Missing,
    OptedOut, // drift-detect: internal-only
}

impl FooError {
    pub fn code(&self) -> benten_errors::ErrorCode {
        match self {
            FooError::Ok => benten_errors::ErrorCode::DslIoError,
            _ => benten_errors::ErrorCode::Unknown("E_FOO".to_string()),
        }
    }
}

pub fn make_ok() -> FooError { FooError::Ok }
// Construct ONLY via the combinator-argument shape \`.ok_or(FooError::Missing)\`
// (unit variant immediately followed by \`)\`). This is the F-06 regression
// fixture: under the pre-fix follow-set (no \`)\`) this site read as
// "no production construction" and FooError::Missing wrongly came back
// \`skipped\`; the \`)\`-extended follow-set correctly detects it as a
// production construction → \`violation\` (constructed but not mirrored).
pub fn make_missing(x: Option<()>) -> Result<(), FooError> { x.ok_or(FooError::Missing) }
pub fn make_opted_out() -> FooError { FooError::OptedOut }
`,
    },
  ];
  const results = fixtureEnum.variants.map((v) =>
    [v.name, judgeVariant(fixtureEnum, v, fixtureFiles)] as const,
  );
  let ok = true;
  const log: string[] = [];
  for (const [name, verdict] of results) {
    if (name === "Ok" && verdict.kind !== "ok") {
      ok = false;
      log.push(`[self-test] FAIL: FooError::Ok expected ok, got ${verdict.kind}`);
    } else if (name === "Missing" && verdict.kind !== "violation") {
      ok = false;
      log.push(
        `[self-test] FAIL: FooError::Missing expected violation, got ${verdict.kind}`,
      );
    } else if (name === "OptedOut" && verdict.kind !== "skipped") {
      ok = false;
      log.push(
        `[self-test] FAIL: FooError::OptedOut expected skipped, got ${verdict.kind}`,
      );
    } else {
      log.push(`[self-test] OK: FooError::${name} — ${verdict.kind}`);
    }
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

if (!existsSync(CRATES_DIR)) {
  die(2, `crates dir not found at ${CRATES_DIR}`);
}

const files = collectProductionFiles(CRATES_DIR);
if (files.length === 0) {
  die(2, `zero production files found under ${CRATES_DIR}`);
}

// Exclude the benten-errors definition site from enum DISCOVERY — the
// `ErrorCode` enum lives there and its variants are the codes, not types
// for the §3.5g item 6 check.
const enumDiscoverableFiles = files.filter((f) => f.path !== ERRORS_DEFINITION_SITE);
const enums = discoverEnums(enumDiscoverableFiles);
const baseline = loadBaseline(BASELINE_PATH);

const summary: string[] = [];
summary.push(
  `[drift-detect-mirror] §3.5g item 6 scanner — found ${enums.length} pub *Error enum(s) across ${files.length} production files; baseline grandfathers ${baseline.size} pre-2026-05-24 variant(s)`,
);

let violations = 0;
let ok = 0;
let skipped = 0;
let baselineHits = 0;
const baselineSeen = new Set<string>();

for (const e of enums) {
  for (const v of e.variants) {
    const verdict = judgeVariant(e, v, files, baseline);
    switch (verdict.kind) {
      case "ok":
        ok++;
        break;
      case "skipped":
        skipped++;
        if (verdict.reason.includes("grandfathered by")) {
          baselineHits++;
          baselineSeen.add(`${e.typeName}::${v.name}`);
        }
        break;
      case "violation":
        violations++;
        summary.push(
          `[drift-detect-mirror] VIOLATION: ${e.typeName}::${v.name} (declared at ${relative(REPO_ROOT, e.file)}:${v.line})`,
        );
        summary.push(`  ! ${verdict.reason}`);
        for (const s of verdict.sites.slice(0, 5)) {
          summary.push(`    - constructed at ${relative(REPO_ROOT, s.file)}:${s.line}`);
        }
        if (verdict.sites.length > 5) {
          summary.push(`    - ... and ${verdict.sites.length - 5} more site(s)`);
        }
        break;
    }
  }
}

summary.push(
  `[drift-detect-mirror] ${ok} variant(s) mirrored, ${skipped} skipped (${baselineHits} via baseline), ${violations} new violation(s)`,
);

// Stale baseline entries: report (informational) the variants the
// baseline names that we didn't encounter. These should be removed from
// the baseline (either the variant is gone OR it now has a real mirror).
const staleBaseline = [...baseline].filter((k) => !baselineSeen.has(k));
if (staleBaseline.length > 0) {
  summary.push(
    `[drift-detect-mirror] STALE BASELINE: ${staleBaseline.length} entry/entries in the baseline file are no longer needed (variant either gone OR now mirrored). Remove them from scripts/drift-detect-error-variant-mirror-baseline.txt:`,
  );
  for (const s of staleBaseline.slice(0, 10)) {
    summary.push(`  - ${s}`);
  }
  if (staleBaseline.length > 10) {
    summary.push(`  - ... and ${staleBaseline.length - 10} more`);
  }
}

process.stdout.write(summary.join("\n") + "\n");

if (violations > 0) {
  process.stderr.write(
    `\n[drift-detect-mirror] FAIL: ${violations} pub error-variant first-class-mirror violation(s). ` +
      `Either add a catalog entry + ErrorCode mapping (§3.5g item 6) OR annotate ` +
      `the variant with one of: \`// drift-detect: wrapped-at-napi-by E_WRAPPER\`, ` +
      `\`// drift-detect: internal-only\`, or \`// drift-detect: ignore\` (with a comment ` +
      `explaining why).\n`,
  );
  process.exit(1);
}

process.stdout.write(
  "[drift-detect-mirror] OK — every production-constructed pub error variant has a first-class ErrorCode mirror.\n",
);
process.exit(0);
