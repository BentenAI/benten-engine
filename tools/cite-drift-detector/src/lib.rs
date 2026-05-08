//! Phase-3 G13-pre-A: cite-drift detector.
//!
//! Walks every documentation file under a given root (`docs/`, `README.md`,
//! and `.addl/` when present locally) and extracts two kinds of cross-tree
//! references:
//!
//!   1. **`file:line` line cites** — e.g. `crates/benten-engine/src/engine.rs:472`.
//!      The detector validates that the file exists at HEAD and contains the
//!      cited line. It additionally flags any **bare line cite against the
//!      high-churn surface list** per §3.5b HARDENED point 3 of the Phase-2b
//!      dispatch-conventions: those surfaces MUST use `path::symbol` form,
//!      because line numbers drift on every refactor.
//!
//!   2. **`path::symbol` symbol cites** — e.g.
//!      `crates/benten-engine/src/primitive_host.rs::execute_sandbox`.
//!      The detector validates that the symbol exists in the cited file by
//!      grepping for its definition (`fn`/`struct`/`enum`/`type`/`pub`/
//!      `const`/`static`/`trait`/`impl`/`macro_rules!`/`mod`/JS-`function`/
//!      JS-`class`/JS-`const`/JS-`let`/JS-`export` shapes).
//!
//! A separate `numeric_claim` pass scans the same documentation files for
//! cross-doc numeric claims (crate counts, primitive counts, invariant
//! counts, test counts) and verifies each against a single source-of-truth
//! map. This closes `docs/future/phase-2-backlog.md` §8.2 (the historic
//! "ENGINE-SPEC §14.6 numeric claims drift" lint) by reusing this tool's
//! parser/validator infrastructure.
//!
//! The detector is intentionally **non-blocking on first deployment** — the
//! CI workflow runs in PR-comment mode. Promotion-to-required is tracked
//! as `D-PHASE-3-10` in the Phase-3 implementation plan.
//!
//! ## CLI
//!
//! ```text
//! cite-drift-detector <root-dir> [--numeric-claims] [--json]
//! ```
//!
//! Exits non-zero when any finding is emitted.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// One finding emitted by the detector. The `kind` field discriminates
/// between cite-drift and numeric-claim-drift findings; `path`+`line`
/// locate the offending source-of-cite (NOT the cited target).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub kind: FindingKind,
    /// File the cite appears IN (the doc/source emitting the cite).
    pub path: PathBuf,
    /// Line in `path` where the cite appears (1-indexed).
    pub line: usize,
    /// Human-readable explanation, in the shape `<offending-cite> :: <reason>`.
    pub message: String,
}

/// Discriminator for the kind of drift the detector caught.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FindingKind {
    /// `path/to/file.rs:NN` — file does not exist at HEAD.
    LineCiteFileMissing,
    /// `path/to/file.rs:NN` — file exists but does not have line NN.
    LineCiteLineOutOfRange,
    /// `path/to/file.rs:NN` — file is on the high-churn surface list and
    /// MUST use `path::symbol` form per §3.5b HARDENED point 3.
    LineCiteOnHighChurnSurface,
    /// `path/to/file.rs::symbol` — file does not exist at HEAD.
    SymbolCiteFileMissing,
    /// `path/to/file.rs::symbol` — file exists but the symbol is not
    /// defined in it.
    SymbolCiteSymbolMissing,
    /// A cross-doc numeric claim does not match the source-of-truth value.
    NumericClaimDrift,
}

impl fmt::Display for FindingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FindingKind::LineCiteFileMissing => "line-cite-file-missing",
            FindingKind::LineCiteLineOutOfRange => "line-cite-line-out-of-range",
            FindingKind::LineCiteOnHighChurnSurface => "line-cite-on-high-churn-surface",
            FindingKind::SymbolCiteFileMissing => "symbol-cite-file-missing",
            FindingKind::SymbolCiteSymbolMissing => "symbol-cite-symbol-missing",
            FindingKind::NumericClaimDrift => "numeric-claim-drift",
        };
        f.write_str(s)
    }
}

/// Phase-2b §3.5b HARDENED point 3: these surfaces churn fast enough that
/// line cites go stale within rounds. Cites against them MUST use the
/// `path::symbol` form. The detector emits
/// `LineCiteOnHighChurnSurface` for any bare `file.rs:NN` cite that
/// targets one of these surfaces.
///
/// Stored as basename suffixes (case-sensitive) — a cite to
/// `crates/benten-engine/src/primitive_host.rs:899` matches because the
/// path's last component matches the entry `primitive_host.rs`.
pub const HIGH_CHURN_SURFACES: &[&str] = &[
    "primitive_host.rs",
    "engine_views.rs",
    "evaluator.rs",
    "lib.rs",
    "builder.rs",
    "wait.rs",
    "subscribe.rs",
    "mermaid.ts",
    "dsl.ts",
];

// ---------------------------------------------------------------------------
// Walking + parsing
// ---------------------------------------------------------------------------

/// Returns the list of doc/source files this detector inspects, rooted at
/// `root`. The walker includes:
///
///   - every `.md` under `docs/`
///   - `README.md` at the root
///   - every `.md` under `.addl/` (only when the directory exists; it is
///     gitignored, so CI runs against `docs/` + `README.md` only).
///   - every `.rs` under `crates/` (recursive — covers `crates/*/src/`,
///     `crates/*/tests/`, and `crates/*/build.rs` automatically since
///     `walk_ext_recursive` matches by extension and walks the full
///     subtree).
///   - every `.rs` under `tools/` (orchestrator-side tooling; §7.12b
///     file-tree-gap closure — `walk_ext_recursive` skips per-tool
///     `target/` build outputs).
///   - every `.toml` under `.cargo/` — typically `.cargo/config.toml`
///     alias-comment blocks; §7.12b file-tree-gap closure.
///     `extract_line_cites` already accepts the `.toml` extension per
///     its allow-list.
///   - typescript source files under `packages/engine/src/` (extracts
///     doc-comment cites)
pub fn walk_doc_inputs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();

    // README.md
    let readme = root.join("README.md");
    if readme.is_file() {
        out.push(readme);
    }

    // docs/**/*.md
    walk_md_recursive(&root.join("docs"), &mut out);

    // .addl/**/*.md (when present locally)
    let addl = root.join(".addl");
    if addl.is_dir() {
        walk_md_recursive(&addl, &mut out);
    }

    // crates/*/src/**/*.rs (doc-comment cites)
    walk_ext_recursive(&root.join("crates"), "rs", &mut out);

    // tools/*/src/**/*.rs (orchestrator-side tooling; §7.12b extension).
    // Walks the entire `tools/` subtree — `walk_ext_recursive` already
    // skips `target/` + `node_modules/` so per-tool `target/` build
    // outputs are excluded automatically. Picks up build.rs at any depth
    // by virtue of `.rs` extension (the walker matches by extension, not
    // path-component name).
    walk_ext_recursive(&root.join("tools"), "rs", &mut out);

    // .cargo/config.toml — alias-comment blocks frequently cite source
    // files. Use the `walk_ext_recursive` walker for consistency with
    // the rest of the envelope; `.cargo/` typically only holds
    // `config.toml`, so the walker terminates after one entry.
    walk_ext_recursive(&root.join(".cargo"), "toml", &mut out);

    // packages/engine/src/**/*.ts (doc-comment cites)
    walk_ext_recursive(
        &root.join("packages").join("engine").join("src"),
        "ts",
        &mut out,
    );

    // De-duplicate before sort: `walk_ext_recursive` is invoked
    // multiple times and the `crates/` walk already covers `crates/*/build.rs`
    // + `crates/*/tests/**/*.rs`, but a defensive de-dup guards against
    // future walker rearrangement double-listing.
    out.sort();
    out.dedup();
    out
}

fn walk_md_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    walk_ext_recursive(dir, "md", out);
}

fn walk_ext_recursive(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip build/cache subtrees + the detector's own subtree.
            // The detector's `src/` + `tests/` files contain intentional
            // cite-shaped fixture strings (extract_line_cites unit-test
            // inputs, fixture-doc-narrative test bodies); scanning them
            // as input would double-count those fixtures as real cites.
            // Self-exclusion is the simplest discipline that doesn't
            // require generic cfg-test awareness.
            let basename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if basename == "target"
                || basename == "node_modules"
                || basename == "cite-drift-detector"
            {
                continue;
            }
            walk_ext_recursive(&path, ext, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

// ---------------------------------------------------------------------------
// Cite parsing
// ---------------------------------------------------------------------------

/// One raw `path:line` cite extracted from a doc.
#[derive(Clone, Debug, PartialEq, Eq)]
struct LineCite {
    target_path: String,
    target_line: usize,
}

/// One raw `path::symbol` cite extracted from a doc.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SymbolCite {
    target_path: String,
    target_symbol: String,
}

/// Extract every `path/to/file.rs:NN` style cite from a single line of
/// text. Recognises `.rs`, `.ts`, `.tsx`, `.toml`, `.wat`, `.wasm`, `.md`,
/// `.json`, `.yml`, `.yaml` extensions. Returns the list of cites found
/// on this line.
///
/// **Bare-basename filter:** cites whose path contains no `/` are
/// shorthand-context (e.g. `sandbox_escape_attempts_denied.rs:76` inside
/// a SECURITY-POSTURE table). These are not validated because the
/// resolved-from-root form would be ambiguous; the **high-churn surface**
/// check above DOES still operate on basename, so the surface-coverage
/// commitment of §3.5b HARDENED point 3 is unaffected.
fn extract_line_cites(s: &str) -> Vec<LineCite> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Find next `:digit` window — the line-number tail anchors the
        // search. Skip forward when we don't see a colon.
        if bytes[i] != b':' {
            i += 1;
            continue;
        }
        // Need at least one digit after the colon.
        let line_start = i + 1;
        let mut line_end = line_start;
        while line_end < bytes.len() && bytes[line_end].is_ascii_digit() {
            line_end += 1;
        }
        if line_end == line_start {
            i += 1;
            continue;
        }
        // Walk LEFT from `i-1` to find the path. Stop at any character
        // that can't be part of a path segment for our purposes.
        let path_end = i;
        let mut path_start = i;
        while path_start > 0 {
            let c = bytes[path_start - 1];
            if is_path_char(c) {
                path_start -= 1;
            } else {
                break;
            }
        }
        if path_start == path_end {
            i = line_end;
            continue;
        }
        let raw_path = &s[path_start..path_end];
        // Strip leading punctuation / quoting.
        let raw_path = raw_path.trim_start_matches(['(', '[', '`', '\'', '"', '<']);
        // Strip leading `./` artifacts.
        let raw_path = raw_path.trim_start_matches("./");
        if raw_path.is_empty() {
            i = line_end;
            continue;
        }
        // Must contain a recognised extension before the colon — the path
        // must end with `.<ext>` where ext is in our allow-list.
        if !looks_like_source_file(raw_path) {
            i = line_end;
            continue;
        }
        // Reject if the matched path is a bare hostname segment like
        // `https://example.com` — we look for at least one `/` (interior
        // path separator) AND the path must not start with a `:` or a
        // protocol indicator.
        if raw_path.starts_with("//") {
            i = line_end;
            continue;
        }
        // Reject github.com URLs and friends — these contain `.com:` or
        // similar with a digit after, but our extension check rejects them.
        let line_num: usize = s[line_start..line_end].parse().unwrap_or(0);
        if line_num == 0 {
            i = line_end;
            continue;
        }
        out.push(LineCite {
            target_path: raw_path.to_string(),
            target_line: line_num,
        });
        i = line_end;
    }
    out
}

/// Extract every `path/to/file.rs::symbol_name` style cite from a single
/// line. Symbols accept `[A-Za-z0-9_]+` with optional trailing `(...)` or
/// `::` qualifications stripped to the head identifier.
fn extract_symbol_cites(s: &str) -> Vec<SymbolCite> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        // Look for `::` AFTER an extension.
        if bytes[i] == b':' && bytes[i + 1] == b':' {
            // Walk LEFT to find the file path ending in a known extension.
            let path_end = i;
            let mut path_start = i;
            while path_start > 0 {
                let c = bytes[path_start - 1];
                if is_path_char(c) {
                    path_start -= 1;
                } else {
                    break;
                }
            }
            if path_start == path_end {
                i += 2;
                continue;
            }
            let raw_path = &s[path_start..path_end];
            let raw_path = raw_path.trim_start_matches(['(', '[', '`', '\'', '"', '<']);
            let raw_path = raw_path.trim_start_matches("./");
            if raw_path.is_empty() || !looks_like_source_file(raw_path) {
                i += 2;
                continue;
            }
            // Walk RIGHT past `::` to find the head identifier.
            let sym_start = i + 2;
            let mut sym_end = sym_start;
            while sym_end < bytes.len() && is_ident_char(bytes[sym_end]) {
                sym_end += 1;
            }
            if sym_end == sym_start {
                i += 2;
                continue;
            }
            let symbol = &s[sym_start..sym_end];
            out.push(SymbolCite {
                target_path: raw_path.to_string(),
                target_symbol: symbol.to_string(),
            });
            i = sym_end;
            continue;
        }
        i += 1;
    }
    out
}

#[inline]
fn is_path_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'/' | b'.')
}

#[inline]
fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

fn looks_like_source_file(p: &str) -> bool {
    const EXTS: &[&str] = &[
        ".rs", ".ts", ".tsx", ".toml", ".wat", ".wasm", ".md", ".json", ".yml", ".yaml",
    ];
    EXTS.iter().any(|e| p.ends_with(e))
}

fn is_high_churn(target_path: &str) -> bool {
    // Match by basename to keep the surface tight; line cites against any
    // file whose basename matches a high-churn entry are flagged.
    let basename = target_path.rsplit('/').next().unwrap_or(target_path);
    HIGH_CHURN_SURFACES.contains(&basename)
}

/// Workspace-relative cites must start with one of these top-level
/// segments. A cite that contains a `/` but starts with anything else
/// (e.g. `benten-eval/Cargo.toml:66`, `primitives/mod.rs:100`) is
/// crate-relative shorthand inside a doc-comment and is not validated
/// (the resolved-from-root form would be ambiguous).
const WORKSPACE_TOP_LEVEL_SEGMENTS: &[&str] = &[
    "crates/",
    "bindings/",
    "tools/",
    "tests/",
    "docs/",
    "packages/",
    "scripts/",
    ".github/",
    ".addl/",
];

fn is_workspace_relative(target_path: &str) -> bool {
    WORKSPACE_TOP_LEVEL_SEGMENTS
        .iter()
        .any(|seg| target_path.starts_with(seg))
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate every cite in every doc-input under `root`. Returns the list
/// of findings (empty on a clean tree).
pub fn run_cite_drift_check(root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    let inputs = walk_doc_inputs(root);

    for input in &inputs {
        let Ok(text) = fs::read_to_string(input) else {
            continue;
        };
        for (line_idx, line) in text.lines().enumerate() {
            let line_no = line_idx + 1;

            // Symbol cites first — `file.rs::symbol` would otherwise also
            // match as `file.rs:` with a malformed line number, but the
            // line-cite parser requires digits after the colon, so order
            // is safe; we still process symbols first for clarity.
            for sc in extract_symbol_cites(line) {
                check_symbol_cite(input, line_no, &sc, root, &mut findings);
            }

            for lc in extract_line_cites(line) {
                check_line_cite(input, line_no, &lc, root, &mut findings);
            }
        }
    }

    findings.sort();
    findings
}

fn check_line_cite(
    source_path: &Path,
    source_line: usize,
    lc: &LineCite,
    root: &Path,
    findings: &mut Vec<Finding>,
) {
    let is_bare_basename = !lc.target_path.contains('/');

    // High-churn enforcement applies regardless of bare-basename vs
    // full-path: §3.5b HARDENED point 3 promotes symbol cites to MUST
    // for these surfaces, and the convention catches both forms (a
    // `primitive_host.rs:899` shorthand inside a doc table is just as
    // stale as a fully-qualified cite).
    if is_high_churn(&lc.target_path) {
        findings.push(Finding {
            kind: FindingKind::LineCiteOnHighChurnSurface,
            path: source_path.to_path_buf(),
            line: source_line,
            message: format!(
                "{}:{} :: high-churn surface; use `path::symbol` form per §3.5b HARDENED point 3",
                lc.target_path, lc.target_line
            ),
        });
        return;
    }

    // Bare-basename cites against non-high-churn surfaces are shorthand
    // context (e.g. SECURITY-POSTURE tables citing test-file basenames);
    // the basename-only form is ambiguous to resolve from root, so the
    // detector skips them. The high-churn check above STILL operates on
    // basename, so the surface-coverage commitment is unaffected.
    if is_bare_basename {
        return;
    }

    // Cites with a `/` but no recognised top-level segment are
    // crate-relative or in-doc shorthand (e.g. `benten-eval/Cargo.toml:66`
    // inside a `crates/benten-engine` doc-comment); same rationale as
    // bare-basename — ambiguous to resolve from workspace root.
    if !is_workspace_relative(&lc.target_path) {
        return;
    }

    let target = root.join(&lc.target_path);
    if !target.is_file() {
        findings.push(Finding {
            kind: FindingKind::LineCiteFileMissing,
            path: source_path.to_path_buf(),
            line: source_line,
            message: format!(
                "{}:{} :: target file does not exist at HEAD",
                lc.target_path, lc.target_line
            ),
        });
        return;
    }

    let Ok(target_text) = fs::read_to_string(&target) else {
        return;
    };
    let line_count = target_text.lines().count();
    if lc.target_line == 0 || lc.target_line > line_count {
        findings.push(Finding {
            kind: FindingKind::LineCiteLineOutOfRange,
            path: source_path.to_path_buf(),
            line: source_line,
            message: format!(
                "{}:{} :: file has {} lines",
                lc.target_path, lc.target_line, line_count
            ),
        });
    }
}

fn check_symbol_cite(
    source_path: &Path,
    source_line: usize,
    sc: &SymbolCite,
    root: &Path,
    findings: &mut Vec<Finding>,
) {
    // Bare-basename cites are shorthand context (same rationale as
    // `check_line_cite`); skip them to avoid flagging conventional
    // doc-prose forms like `primitive_host.rs::execute_sandbox` that
    // appear inside narrative sentences. The symbol form for
    // high-churn surfaces is still validated when the cite is
    // fully-qualified, which is the form §3.5b HARDENED point 3
    // expects callers to migrate TO.
    if !sc.target_path.contains('/') {
        return;
    }

    // Same workspace-prefix filter as `check_line_cite` — crate-relative
    // shorthand (e.g. `benten-graph/src/redb_backend.rs::guard_x` inside
    // a `crates/benten-engine` doc-comment) is ambiguous to resolve from
    // workspace root and is not validated.
    if !is_workspace_relative(&sc.target_path) {
        return;
    }

    // Skip `path::symbol` cites where the path is a markdown file —
    // the markdown-header-as-symbol convention (e.g.
    // `docs/ERROR-CATALOG.md::E_FOO_BAR` referencing a section
    // header) is documentation shorthand, not a code-shape symbol
    // cite. We still validate the file exists; we do NOT validate
    // the header is present (would require markdown parsing scope
    // expansion the lint doesn't carry).
    let target = root.join(&sc.target_path);
    if !target.is_file() {
        findings.push(Finding {
            kind: FindingKind::SymbolCiteFileMissing,
            path: source_path.to_path_buf(),
            line: source_line,
            message: format!(
                "{}::{} :: target file does not exist at HEAD",
                sc.target_path, sc.target_symbol
            ),
        });
        return;
    }
    if Path::new(&sc.target_path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        return;
    }
    let Ok(target_text) = fs::read_to_string(&target) else {
        return;
    };
    if !target_text_defines_symbol(&target_text, &sc.target_symbol) {
        findings.push(Finding {
            kind: FindingKind::SymbolCiteSymbolMissing,
            path: source_path.to_path_buf(),
            line: source_line,
            message: format!(
                "{}::{} :: symbol not defined in target file",
                sc.target_path, sc.target_symbol
            ),
        });
    }
}

/// Return `true` when `text` contains a `pub use` re-export naming
/// `symbol`. Handles three shapes:
///
///   1. Single-line: `pub use foo::Symbol;` (no brace list)
///   2. Single-line brace list: `pub use foo::{A, Symbol, B};`
///   3. Multi-line brace list:
///      ```text
///      pub use foo::{
///          A,
///          Symbol,
///          B,
///      };
///      ```
///
/// The scanner looks for `pub use` then walks until the next `;`,
/// extracting every identifier appearing as a comma- / brace- /
/// whitespace-bounded token inside that span. Aliases (`A as Symbol`)
/// match on the alias side since the cite refers to the re-exported
/// name. Glob re-exports (`pub use foo::*;`) are NOT inferred — we
/// can't tell from this side what names they cover.
fn text_reexports_symbol(text: &str, symbol: &str) -> bool {
    let mut rest = text;
    while let Some(idx) = rest.find("pub use") {
        // Validate that this match is at start-of-line / preceded by
        // whitespace (so we don't trip on `// pub use` doc-comment
        // narrative — the leading-`//` case is implicitly excluded
        // because doc-comments still produce real `pub use` re-exports
        // in their explanations only via prose. Acceptable noise.)
        let after = &rest[idx + "pub use".len()..];
        // Find terminating `;` or end-of-text.
        let end = after.find(';').unwrap_or(after.len());
        let span = &after[..end];
        // Tokenise on whitespace + `{}` + `,` + `:` + `as`.
        // Walk the span byte-by-byte collecting identifier tokens.
        let bytes = span.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if is_ident_char(bytes[i]) {
                let start = i;
                while i < bytes.len() && is_ident_char(bytes[i]) {
                    i += 1;
                }
                let token = &span[start..i];
                if token == symbol {
                    return true;
                }
            } else {
                i += 1;
            }
        }
        rest = &after[end..];
    }
    false
}

/// Return `true` when `text` contains a syntactic definition of `symbol`.
/// Matches keyword-prefixed shapes from Rust + TypeScript — we do NOT
/// parse the language, but we look for the canonical definition forms
/// that mainstream code uses. Also recognises `pub use ... { Symbol, ... }`
/// re-exports, since many crate roots expose their public API surface
/// via re-export from sibling modules; a cite to the re-export site is
/// legitimate even though the in-place definition lives elsewhere.
fn target_text_defines_symbol(text: &str, symbol: &str) -> bool {
    // First, the cheap re-export scan (handles both single-line
    // `pub use foo::Symbol;` and multi-line `pub use foo::{ Symbol, ... };`
    // shapes). Returns early on hit.
    if text_reexports_symbol(text, symbol) {
        return true;
    }
    // Canonical definition prefixes — we look for `<keyword> <space> <symbol>`
    // followed by a non-identifier character (so `fn foo` matches but
    // `fn foo_bar` does not match a search for `foo`).
    const KEYWORDS: &[&str] = &[
        "fn ",
        "struct ",
        "enum ",
        "trait ",
        "type ",
        "const ",
        "static ",
        "mod ",
        "macro_rules! ",
        "impl ",
        "function ",
        "class ",
        "interface ",
        // namespace + lexical-binding shapes from TS.
        "namespace ",
        "let ",
        "var ",
    ];
    for line in text.lines() {
        // Trim attribute-only / pub-only prefixes so `pub fn foo` matches
        // a search for keyword `fn `.
        let trimmed = line.trim_start();
        // Strip leading visibility / async / unsafe / export modifiers.
        let trimmed = strip_modifiers(trimmed);
        for kw in KEYWORDS {
            if let Some(rest) = trimmed.strip_prefix(kw) {
                let rest = rest.trim_start();
                if let Some(after) = rest.strip_prefix(symbol) {
                    if after.is_empty() {
                        return true;
                    }
                    let next = after.as_bytes()[0];
                    if !is_ident_char(next) {
                        return true;
                    }
                }
            }
        }
        // Bonus: TS `export const NAME =` / `export class NAME` etc.
        // already handled by strip_modifiers + KEYWORDS loop. Also accept
        // bare `NAME(...)` JS-arrow style: `const NAME = (...) =>`.
        // (Handled because `const ` is in KEYWORDS.)

        // Bonus: macro_rules!-like cases where the `!` glues to the
        // identifier: `macro_rules! foo` is matched above; some docs
        // cite a macro by its `!` form: handle `foo!` by stripping the
        // trailing `!` from the search, which the symbol parser already
        // does (we accept `[A-Za-z0-9_]+` only).
    }
    false
}

fn strip_modifiers(line: &str) -> &str {
    let mods = [
        "pub(crate) ",
        "pub(super) ",
        "pub ",
        "async ",
        "unsafe ",
        "const ", // tricky — see note below
        "default ",
        "export ",
        "declare ",
        "abstract ",
        "static ", // TS class member modifier; same caveat as `const`
    ];
    // We strip greedily but never strip `const ` or `static ` because
    // those ARE keywords we want to preserve at the front of the line —
    // a top-level `const FOO = ...` should match the `const ` keyword in
    // the search loop. So we filter modifiers down to the non-overloaded
    // set:
    let safe_mods: &[&str] = &[
        "pub(crate) ",
        "pub(super) ",
        "pub ",
        "async ",
        "unsafe ",
        "default ",
        "export ",
        "declare ",
        "abstract ",
    ];
    let _ = mods; // intentional — kept as commentary aid.
    let mut s = line;
    loop {
        let mut stripped = false;
        for m in safe_mods {
            if let Some(rest) = s.strip_prefix(m) {
                s = rest;
                stripped = true;
                break;
            }
        }
        if !stripped {
            break;
        }
    }
    s
}

// ---------------------------------------------------------------------------
// Numeric-claim drift (closes phase-2-backlog §8.2)
// ---------------------------------------------------------------------------

/// One numeric claim to enforce across docs. Source-of-truth is hard-pinned
/// in `numeric_claims_source_of_truth()`; the detector emits a finding for
/// any other phrasing of the same surface that disagrees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericClaim {
    /// What the number is counting (e.g. `"primitives"`, `"invariants"`).
    pub label: &'static str,
    /// Authoritative value.
    pub value: u32,
    /// Phrasings to scan for. Each phrasing is a substring containing
    /// `{N}` where the number lives. The detector finds every occurrence,
    /// extracts the number, and compares to `value`.
    pub phrasings: &'static [&'static str],
}

/// Authoritative count map. As of Phase-3 R3 corpus merge (post `82f1c7e`):
///
///   - 12 operation primitives (CLAUDE.md baked-in #1) — hardcoded
///   - 14 invariants (CLAUDE.md status table; INVARIANT-COVERAGE.md) —
///     hardcoded
///   - **`crates` count is now derived dynamically** from the workspace
///     `Cargo.toml` `members =` table at the supplied root. See
///     `derive_crate_count_from_workspace`. Pim-12 NEW shape (iii)
///     tools-as-meta-spec closure: the detector self-derives this row so
///     the count never drifts when a crate is added or removed.
///     Fallback when `Cargo.toml` is unparseable: returns the
///     historical static value 10 (last manually-confirmed count) so
///     the lint still operates against a stable baseline rather than
///     silently disabling.
///
/// **Phrasing scope is deliberately tight.** We only flag the
/// authoritative-total shapes (e.g. "all 12 primitives", "all 14
/// invariants"); bare-count phrasings like `8 primitives` are NOT flagged
/// because they routinely refer to phase-bounded subsets in narrative
/// prose ("the Phase-1 8 primitives", "the 4 IO primitives"). Catching
/// those in narrative context would generate noise. The lint targets
/// **claims of authority**, not all numeric mentions.
///
/// `primitives` + `invariants` remain hardcoded because their
/// authoritative source is documentation (CLAUDE.md baked-in
/// commitments), not workspace structure. When either of those
/// counts changes, the touching group's brief MUST update this table
/// in the SAME commit per pim-1 §3.5b post-fix doc-coupling (the
/// detector ITSELF is a cited surface).
///
/// **Backwards-compatible API:** the `value: u32` field shape is
/// unchanged; the dynamic derivation runs once at this function's
/// call site and embeds the result.
pub fn numeric_claims_source_of_truth() -> Vec<NumericClaim> {
    numeric_claims_source_of_truth_at(Path::new("."))
}

/// Variant of `numeric_claims_source_of_truth` that takes the workspace
/// root explicitly. Used by tests + by callers (e.g. the CLI) that
/// invoke the detector against an arbitrary path.
pub fn numeric_claims_source_of_truth_at(root: &Path) -> Vec<NumericClaim> {
    let crate_count = derive_crate_count_from_workspace(root).unwrap_or(10);
    vec![
        NumericClaim {
            label: "primitives",
            value: 12,
            phrasings: &[
                "all {N} operation primitives",
                "all {N} primitives",
                "{N} operation primitives",
                "the {N} primitives",
            ],
        },
        NumericClaim {
            label: "invariants",
            value: 14,
            phrasings: &[
                "all {N} invariants",
                "{N} of {N} invariants",
                "the {N} invariants",
            ],
        },
        NumericClaim {
            label: "crates",
            value: crate_count,
            phrasings: &["all {N} crates", "the {N} crates", "{N}-crate"],
        },
    ]
}

/// Parse `<root>/Cargo.toml` and count the number of workspace members
/// whose path begins with `crates/`. The `crates` numeric-claim
/// authoritative value is derived from this count rather than
/// hardcoded — closes pim-12 NEW shape (iii) per
/// `dispatch-conventions.md::§3.5c` 2026-05-05 amendment.
///
/// Returns `Some(count)` on success; `None` when the file is missing,
/// unparseable, or has no `[workspace] members = [...]` table. Callers
/// fall back to the historical static value (10) when this returns
/// `None` — the lint stays operational even if a tree-state change
/// invalidates the parse temporarily.
///
/// **Counting rule:** every entry whose path string starts with the
/// literal segment `crates/` counts. `tools/`, `bindings/`, and
/// `tests/` entries are excluded. This matches the original rustdoc
/// rule for the static count.
pub fn derive_crate_count_from_workspace(root: &Path) -> Option<u32> {
    let cargo_toml = root.join("Cargo.toml");
    let text = fs::read_to_string(&cargo_toml).ok()?;
    let parsed: toml::Value = toml::from_str(&text).ok()?;
    let workspace = parsed.get("workspace")?.as_table()?;
    let members = workspace.get("members")?.as_array()?;
    let mut count: u32 = 0;
    for entry in members {
        let Some(s) = entry.as_str() else { continue };
        if s.starts_with("crates/") {
            count = count.saturating_add(1);
        }
    }
    Some(count)
}

/// Run the numeric-claim drift pass. Returns one finding per disagreement.
///
/// The `crates` row is derived from `<root>/Cargo.toml`'s `members =`
/// table per `derive_crate_count_from_workspace`; `primitives` +
/// `invariants` rows remain hardcoded.
pub fn run_numeric_claim_check(root: &Path) -> Vec<Finding> {
    run_numeric_claim_check_with_truth(root, &numeric_claims_source_of_truth_at(root))
}

/// Same as `run_numeric_claim_check` but with an explicit truth set —
/// used by the test fixture to plant a controlled drift.
pub fn run_numeric_claim_check_with_truth(root: &Path, truth: &[NumericClaim]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let inputs = walk_doc_inputs(root);
    for input in &inputs {
        let Ok(text) = fs::read_to_string(input) else {
            continue;
        };
        for (line_idx, line) in text.lines().enumerate() {
            for claim in truth {
                for phrasing in claim.phrasings {
                    for actual in scan_phrasing(line, phrasing) {
                        if actual != claim.value {
                            findings.push(Finding {
                                kind: FindingKind::NumericClaimDrift,
                                path: input.clone(),
                                line: line_idx + 1,
                                message: format!(
                                    "claim `{}` expected {} (source-of-truth) but doc says {} (phrasing: `{}`)",
                                    claim.label, claim.value, actual, phrasing
                                ),
                            });
                        }
                    }
                }
            }
        }
    }
    findings.sort();
    findings.dedup();
    findings
}

/// Extract every numeric value matching a phrasing template like
/// `"{N} primitives"`. Returns the list of distinct values found on
/// the line.
///
/// Operates on byte slices (`line.as_bytes()`) to avoid the UTF-8
/// char-boundary trap when advancing past a non-match. The returned
/// values come from ASCII-digit substrings (which are always at char
/// boundaries by construction); the only string-slice we still take is
/// the digit substring at the very end, which is always ASCII.
fn scan_phrasing(line: &str, phrasing: &str) -> Vec<u32> {
    let mut out = Vec::new();
    let n_count = phrasing.matches("{N}").count();
    let bytes = line.as_bytes();

    if n_count == 1 {
        let Some((prefix, suffix)) = phrasing.split_once("{N}") else {
            return out;
        };
        let prefix_b = prefix.as_bytes();
        let suffix_b = suffix.as_bytes();
        let mut start = 0;
        while start <= bytes.len() {
            let Some(off) = find_subslice(&bytes[start..], prefix_b) else {
                break;
            };
            let prefix_start = start + off;
            let after_prefix = prefix_start + prefix_b.len();
            let mut digit_end = after_prefix;
            while digit_end < bytes.len() && bytes[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            if digit_end == after_prefix {
                // No digits — advance past this prefix occurrence by 1 byte.
                start = prefix_start + 1;
                continue;
            }
            if !bytes[digit_end..].starts_with(suffix_b) {
                start = digit_end;
                continue;
            }
            // BOUNDARY: prefix must not be embedded in a longer ascii word.
            if prefix_start > 0 {
                let prev = bytes[prefix_start - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' {
                    start = digit_end;
                    continue;
                }
            }
            // Digits are pure ASCII so this slice is always at a char
            // boundary.
            if let Ok(n) = std::str::from_utf8(&bytes[after_prefix..digit_end])
                .unwrap_or("0")
                .parse::<u32>()
            {
                out.push(n);
            }
            start = digit_end;
        }
    } else if n_count == 2 {
        let parts: Vec<&str> = phrasing.split("{N}").collect();
        if parts.len() != 3 {
            return out;
        }
        let p0_b = parts[0].as_bytes();
        let p1_b = parts[1].as_bytes();
        let p2_b = parts[2].as_bytes();
        let mut start = 0;
        while start <= bytes.len() {
            let Some(off) = find_subslice(&bytes[start..], p0_b) else {
                break;
            };
            let p0_start = start + off;
            let after_p0 = p0_start + p0_b.len();
            let mut d1_end = after_p0;
            while d1_end < bytes.len() && bytes[d1_end].is_ascii_digit() {
                d1_end += 1;
            }
            if d1_end == after_p0 || !bytes[d1_end..].starts_with(p1_b) {
                start = p0_start + 1;
                continue;
            }
            let after_p1 = d1_end + p1_b.len();
            let mut d2_end = after_p1;
            while d2_end < bytes.len() && bytes[d2_end].is_ascii_digit() {
                d2_end += 1;
            }
            if d2_end == after_p1 || !bytes[d2_end..].starts_with(p2_b) {
                start = d2_end.max(p0_start + 1);
                continue;
            }
            if p0_start > 0 {
                let prev = bytes[p0_start - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' {
                    start = d2_end;
                    continue;
                }
            }
            if let (Ok(n1), Ok(n2)) = (
                std::str::from_utf8(&bytes[after_p0..d1_end])
                    .unwrap_or("0")
                    .parse::<u32>(),
                std::str::from_utf8(&bytes[after_p1..d2_end])
                    .unwrap_or("0")
                    .parse::<u32>(),
            ) {
                out.push(n1);
                if n1 != n2 {
                    out.push(n2);
                }
            }
            start = d2_end;
        }
    }
    out
}

/// Tiny `&[u8]::find` polyfill — std doesn't ship `slice::find` for
/// arbitrary subslices, and pulling in `memchr` for one byte-pattern
/// search is overkill for a Phase-3-G13-pre-A tool.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

// ---------------------------------------------------------------------------
// Reporting
// ---------------------------------------------------------------------------

/// Render a list of findings into a markdown-friendly report. Used by both
/// the CLI and the CI workflow (the workflow takes this output verbatim
/// and posts it as a PR comment in non-blocking mode per D-PHASE-3-10).
pub fn render_markdown_report(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "No cite drift detected.\n".to_string();
    }
    let mut by_kind: BTreeSet<FindingKind> = BTreeSet::new();
    for f in findings {
        by_kind.insert(f.kind);
    }
    let mut out = String::new();
    let _ = writeln!(out, "## Cite-drift findings ({} total)\n", findings.len());
    for kind in &by_kind {
        let kind_findings: Vec<&Finding> = findings.iter().filter(|f| f.kind == *kind).collect();
        let _ = writeln!(
            out,
            "### `{}` ({} finding{})\n",
            kind,
            kind_findings.len(),
            if kind_findings.len() == 1 { "" } else { "s" },
        );
        for f in kind_findings {
            let _ = writeln!(out, "- `{}:{}` — {}", f.path.display(), f.line, f.message);
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod parse_unit_tests {
    use super::*;

    #[test]
    fn extract_line_cite_basic() {
        let cites = extract_line_cites("see `crates/foo/src/lib.rs:42` for context");
        assert_eq!(cites.len(), 1);
        assert_eq!(cites[0].target_path, "crates/foo/src/lib.rs");
        assert_eq!(cites[0].target_line, 42);
    }

    #[test]
    fn extract_symbol_cite_basic() {
        let cites = extract_symbol_cites(
            "see `crates/benten-engine/src/primitive_host.rs::execute_sandbox`",
        );
        assert_eq!(cites.len(), 1);
        assert_eq!(
            cites[0].target_path,
            "crates/benten-engine/src/primitive_host.rs"
        );
        assert_eq!(cites[0].target_symbol, "execute_sandbox");
    }

    #[test]
    fn high_churn_match_basename() {
        assert!(is_high_churn("crates/benten-engine/src/primitive_host.rs"));
        assert!(is_high_churn("packages/engine/src/dsl.ts"));
        assert!(!is_high_churn(
            "crates/benten-engine/src/engine_snapshot.rs"
        ));
    }

    #[test]
    fn scan_phrasing_single_n() {
        let v = scan_phrasing("All 12 primitives are listed", "{N} primitives");
        assert_eq!(v, vec![12]);
    }

    #[test]
    fn scan_phrasing_word_boundary() {
        // "small 12 primitives" must NOT match phrasing "all {N} primitives".
        let v = scan_phrasing("small 12 primitives", "all {N} primitives");
        assert_eq!(v, Vec::<u32>::new());
    }

    #[test]
    fn target_text_defines_pub_fn() {
        let txt = "pub fn execute_sandbox() { todo!() }\n";
        assert!(target_text_defines_symbol(txt, "execute_sandbox"));
        assert!(!target_text_defines_symbol(txt, "execute_sandbo"));
    }

    #[test]
    fn target_text_defines_struct() {
        let txt = "pub(crate) struct PrimitiveHost { backend: B }\n";
        assert!(target_text_defines_symbol(txt, "PrimitiveHost"));
    }

    // -----------------------------------------------------------------
    // §7.12: workspace-aware crate-count derivation tests
    // -----------------------------------------------------------------

    #[test]
    fn derive_crate_count_synthetic_workspace_with_n_crates() {
        // Plant a synthetic workspace with 7 `crates/` rows + 3 non-crate
        // rows; assert the derivation returns exactly 7.
        let tmp = tempfile::tempdir().expect("tempdir");
        let cargo = tmp.path().join("Cargo.toml");
        std::fs::write(
            &cargo,
            r#"
[workspace]
resolver = "3"
members = [
    "crates/foo-1",
    "crates/foo-2",
    "crates/foo-3",
    "crates/foo-4",
    "crates/foo-5",
    "crates/foo-6",
    "crates/foo-7",
    "tools/some-tool",
    "bindings/napi",
    "tests/integration",
]
"#,
        )
        .unwrap();
        let count = derive_crate_count_from_workspace(tmp.path()).expect("derivation");
        assert_eq!(count, 7);
    }

    #[test]
    fn derive_crate_count_excludes_non_crate_paths() {
        // A workspace with only non-`crates/` entries should derive 0,
        // not None.
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            r#"
[workspace]
resolver = "3"
members = [
    "tools/x",
    "tests/y",
    "bindings/z",
]
"#,
        )
        .unwrap();
        let count = derive_crate_count_from_workspace(tmp.path()).expect("derivation");
        assert_eq!(count, 0);
    }

    #[test]
    fn derive_crate_count_returns_none_on_missing_cargo_toml() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // No Cargo.toml at all.
        assert!(derive_crate_count_from_workspace(tmp.path()).is_none());
    }

    #[test]
    fn derive_crate_count_returns_none_on_unparseable_cargo_toml() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "this is not [valid TOML} content =",
        )
        .unwrap();
        assert!(derive_crate_count_from_workspace(tmp.path()).is_none());
    }

    #[test]
    fn derive_crate_count_returns_none_when_workspace_table_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            r#"
[package]
name = "single-crate"
version = "0.1.0"
"#,
        )
        .unwrap();
        assert!(derive_crate_count_from_workspace(tmp.path()).is_none());
    }

    #[test]
    fn numeric_claims_at_root_uses_derived_count() {
        // The crate-count row must reflect the derivation result.
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            r#"
[workspace]
members = ["crates/a", "crates/b", "crates/c"]
"#,
        )
        .unwrap();
        let claims = numeric_claims_source_of_truth_at(tmp.path());
        let crates_claim = claims
            .iter()
            .find(|c| c.label == "crates")
            .expect("crates claim present");
        assert_eq!(crates_claim.value, 3);
    }

    #[test]
    fn numeric_claims_at_root_falls_back_to_static_when_cargo_unparseable() {
        // When the workspace can't be parsed, fall back to the historical
        // static value (10) so the lint stays operational.
        let tmp = tempfile::tempdir().expect("tempdir");
        // No Cargo.toml at all → derivation returns None → fallback to 10.
        let claims = numeric_claims_source_of_truth_at(tmp.path());
        let crates_claim = claims
            .iter()
            .find(|c| c.label == "crates")
            .expect("crates claim present");
        assert_eq!(crates_claim.value, 10);
    }

    // -----------------------------------------------------------------
    // §7.12b: walker file-tree gap closure tests
    // -----------------------------------------------------------------

    #[test]
    fn walker_includes_tools_subtree() {
        // Plant a synthetic root with `tools/foo/src/lib.rs` and assert
        // the walker enumerates it. The original walker only covered
        // `crates/*/src/**/*.rs` + `packages/engine/src/**/*.ts` +
        // `docs/**/*.md` + `README.md` — `tools/` was a §7.12b gap.
        let tmp = tempfile::tempdir().expect("tempdir");
        let tools_src = tmp.path().join("tools").join("foo").join("src");
        std::fs::create_dir_all(&tools_src).unwrap();
        let lib_rs = tools_src.join("lib.rs");
        std::fs::write(&lib_rs, "//! tools-side rust file.\n").unwrap();

        let inputs = walk_doc_inputs(tmp.path());
        assert!(
            inputs.iter().any(|p| p == &lib_rs),
            "walker did not enumerate {}",
            lib_rs.display()
        );
    }

    #[test]
    fn walker_includes_dot_cargo_config_toml() {
        // Plant `.cargo/config.toml` and assert the walker enumerates
        // it. `.cargo/` was a §7.12b gap.
        let tmp = tempfile::tempdir().expect("tempdir");
        let cargo_dir = tmp.path().join(".cargo");
        std::fs::create_dir_all(&cargo_dir).unwrap();
        let config_toml = cargo_dir.join("config.toml");
        std::fs::write(&config_toml, "# alias-comment block.\n").unwrap();

        let inputs = walk_doc_inputs(tmp.path());
        assert!(
            inputs.iter().any(|p| p == &config_toml),
            "walker did not enumerate {}",
            config_toml.display()
        );
    }

    #[test]
    fn walker_includes_crates_build_rs_and_tests() {
        // Confirm `crates/*/build.rs` and `crates/*/tests/*.rs` are
        // covered by the recursive `crates/` walk. The §7.12b origin
        // doc cited these as gaps; the walker has actually been
        // recursive since G13-pre-A so they were already covered —
        // this test pins the coverage so a future walker rewrite
        // doesn't accidentally narrow the envelope.
        let tmp = tempfile::tempdir().expect("tempdir");
        let crate_root = tmp.path().join("crates").join("benten-foo");
        std::fs::create_dir_all(crate_root.join("tests")).unwrap();
        let build_rs = crate_root.join("build.rs");
        std::fs::write(&build_rs, "// header doc-comment.\n").unwrap();
        let tests_rs = crate_root.join("tests").join("integration.rs");
        std::fs::write(&tests_rs, "//! integration test.\n").unwrap();

        let inputs = walk_doc_inputs(tmp.path());
        assert!(
            inputs.iter().any(|p| p == &build_rs),
            "walker did not enumerate {}",
            build_rs.display()
        );
        assert!(
            inputs.iter().any(|p| p == &tests_rs),
            "walker did not enumerate {}",
            tests_rs.display()
        );
    }

    #[test]
    fn walker_skips_target_subtree_under_tools() {
        // The `target/` skip should apply uniformly under `tools/` too —
        // a synthetic `tools/foo/target/release/build/lib.rs` should NOT
        // be enumerated.
        let tmp = tempfile::tempdir().expect("tempdir");
        let target_dir = tmp
            .path()
            .join("tools")
            .join("foo")
            .join("target")
            .join("release")
            .join("build");
        std::fs::create_dir_all(&target_dir).unwrap();
        let target_rs = target_dir.join("lib.rs");
        std::fs::write(&target_rs, "//! build-output.\n").unwrap();

        let inputs = walk_doc_inputs(tmp.path());
        assert!(
            !inputs.iter().any(|p| p == &target_rs),
            "walker leaked target/ subtree: {}",
            target_rs.display()
        );
    }
}
