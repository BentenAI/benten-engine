//! R6 R1 FP-A Bundle F1.d — no-regression pin asserting every
//! `pub fn .*_for_test*` (and `pub const .*_for_test*`) in any
//! `crates/<X>/src/` source file carries a `#[cfg(any(test, feature =
//! "testing"))]` (or `feature = "test-helpers"` for benten-engine)
//! attribute, OR is named in the `EXEMPT_PUB_ITEMS` table below.
//!
//! ## Context
//!
//! Per V1-FROZEN-INTERFACE.md §1 (§8-A; the "`_for_test` `pub fn` is a
//! red-flag" rule), V1-FROZEN-INTERFACE-DEFERRED.md Row D-22 sub-task 5,
//! and the R4b L6-MAJOR-1 finding (path-(a) full cascade per Ben
//! 2026-05-24 PM ratification): the production `cdylib` build must NOT
//! ship `_for_test*`-suffixed public symbols. The ~70 surfaces present
//! at HEAD time (Row D-22 inventory) are CLOSED at this fix-pass; this
//! pin holds the line forward.
//!
//! ## What this pin enforces
//!
//! Walks every `crates/*/src/**/*.rs` and `tools/*/src/**/*.rs` file
//! (regex grep — no `syn` parse) and flags any `pub fn <name>_for_test`
//! / `pub fn <name>_for_testing` / `pub const fn <name>_for_test` /
//! `pub const <NAME>_for_test` declaration whose preceding 5 lines do
//! NOT contain a `#[cfg(...)]` attribute mentioning `feature = "testing"`
//! or `feature = "test-helpers"`, AND whose qualified name is not in
//! the [`EXEMPT_PUB_ITEMS`] allow-list.
//!
//! ## Exempt-list policy
//!
//! Items in [`EXEMPT_PUB_ITEMS`] are PRODUCTION-SHAPED despite the
//! `_for_test*` suffix — they're consumed by non-test code paths and
//! cannot be cfg-gated without breaking production builds. New entries
//! to this list require explicit justification + V1-FROZEN-INTERFACE-DEFERRED
//! Row D-22 EXEMPT_PUB_ITEMS table update.
//!
//! The `_for_test*` name pattern itself is a design smell on those
//! items — a future rename (without the misleading suffix) is the
//! v1-GM-target cleanup; this pin does not block the rename, only the
//! re-introduction of newly-public ungated `_for_test*` items.
//!
//! ## Build invocation
//!
//! Workspace test runner mode (default): inspects source files; no
//! features are needed at this test's compile time. Run via:
//!
//! ```text
//! cargo nextest run -p phase-3-workspace-tests --test for_test_symbols_are_feature_gated
//! ```

use std::fs;
use std::path::{Path, PathBuf};

/// PRODUCTION-SHAPED exempt items: `pub fn .*_for_test*` declarations
/// that are consumed by non-test production code paths. Per Row D-22
/// EXEMPT_PUB_ITEMS table; updates here MUST mirror that table.
///
/// Format: `("crates/<crate>/src/<path>", "fn_or_const_name")`.
const EXEMPT_PUB_ITEMS: &[(&str, &str)] = &[
    // benten-core — SubgraphBuilder construction helpers consumed by
    // platform-foundation's schema_compiler emit pipeline production code
    // (build_unvalidated_for_test + set_property_for_test).
    (
        "crates/benten-core/src/subgraph.rs",
        "set_property_for_test",
    ),
    (
        "crates/benten-core/src/subgraph.rs",
        "build_unvalidated_for_test",
    ),
    // benten-id — Keypair byte-extraction used by benten-sync's
    // production peer_discovery + transport bridge to iroh SecretKey
    // (secret_bytes_for_test) + napi-side public-key projection
    // (bytes_for_test).
    ("crates/benten-id/src/keypair.rs", "bytes_for_test"),
    ("crates/benten-id/src/keypair.rs", "secret_bytes_for_test"),
    // benten-id — Did string-construction consumed by benten-id's own
    // device_attestation production code + platform-foundation's
    // manifest_store / plugin_lifecycle production paths.
    (
        "crates/benten-id/src/did.rs",
        "from_string_for_test_fixture",
    ),
    // benten-caps — AuthorizationGrant CID accessor consumed by
    // benten-sync's ucan_blobs_protocol production revocation
    // observance ARM 4.
    (
        "crates/benten-caps/src/authorization_grant.rs",
        "grant_cid_for_test",
    ),
    // benten-crypto-suite — production canonical-bytes pathway:
    // SizeTouchingSurfaces::dag_cbor_encode + load_signature_fixture
    // call these three.
    (
        "crates/benten-crypto-suite/src/sizes.rs",
        "ml_dsa65_for_test",
    ),
    (
        "crates/benten-crypto-suite/src/sig.rs",
        "classical_half_for_test",
    ),
    ("crates/benten-crypto-suite/src/sig.rs", "pq_half_for_test"),
    // benten-crypto-suite — StructuralKdfKey byte-construction used
    // by benten-drop's bundle-build pipeline + future benten-graph
    // structural-encryption seam.
    (
        "crates/benten-crypto-suite/src/structural_kdf.rs",
        "from_bytes_for_test",
    ),
    // benten-ivm — view constructors used by production tests +
    // (forward-protected) production cadence-tuning paths.
    (
        "crates/benten-ivm/src/views/capability_grants.rs",
        "with_budget_for_testing",
    ),
    (
        "crates/benten-ivm/src/views/content_listing.rs",
        "with_budget_for_testing",
    ),
    (
        "crates/benten-ivm/src/views/event_handler_dispatch.rs",
        "with_budget_for_testing",
    ),
    (
        "crates/benten-ivm/src/views/governance_inheritance.rs",
        "with_budget_for_testing",
    ),
    (
        "crates/benten-ivm/src/views/version_current.rs",
        "with_budget_for_testing",
    ),
    // benten-engine thin_client — active_session_count_for_test consumed
    // by tools/benten-admin-shell/src/main.rs:62 (boot banner active
    // sessions count). Added at R6 R1 FP integration PR #1351 fix-up #6
    // after A's F1.a sweep cfg-gated the method, breaking the
    // admin-shell production build (admin-shell doesn't enable
    // benten-engine/test-helpers feature in its dep declaration).
    // v1-GM rename target per the same row: drop the misleading
    // `_for_test` suffix.
    (
        "crates/benten-engine/src/thin_client.rs",
        "active_session_count_for_test",
    ),
];

/// Discover every `crates/*/src/**/*.rs` + `tools/*/src/**/*.rs` source
/// file at the workspace root, relative to `CARGO_MANIFEST_DIR`.
fn collect_source_files(workspace_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for parent in ["crates", "tools"] {
        let parent_dir = workspace_root.join(parent);
        if !parent_dir.exists() {
            continue;
        }
        for crate_entry in fs::read_dir(&parent_dir).unwrap() {
            let crate_entry = crate_entry.unwrap();
            let src_dir = crate_entry.path().join("src");
            if !src_dir.exists() {
                continue;
            }
            walk_rs(&src_dir, &mut out);
        }
    }
    out
}

fn walk_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            walk_rs(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Returns the relative path from `workspace_root` to `p`, in POSIX-style
/// separators (so EXEMPT_PUB_ITEMS comparisons are portable across host
/// platforms).
fn rel_posix(workspace_root: &Path, p: &Path) -> String {
    let stripped = p.strip_prefix(workspace_root).unwrap_or(p);
    stripped.to_string_lossy().replace('\\', "/")
}

/// Extract the `pub fn` / `pub const` identifier that immediately follows
/// the keyword on `line` (skipping `fn`, `const`, `unsafe`, etc.).
fn extract_pub_fn_name(line: &str) -> Option<&str> {
    let l = line.trim_start();
    // `pub fn FOO`, `pub const fn FOO`, `pub unsafe fn FOO`, `pub const FOO:`
    let rest = l.strip_prefix("pub ")?;
    let rest = rest.strip_prefix("unsafe ").unwrap_or(rest);
    let rest = rest
        .strip_prefix("const fn ")
        .or_else(|| rest.strip_prefix("const "))
        .or_else(|| rest.strip_prefix("fn "))?;
    let ident: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if ident.is_empty() {
        None
    } else {
        // Cheat: leak via static-storage? No — return the slice from the
        // original line so the caller can use it without lifetime drama.
        // Walk the original line to find the substring boundaries.
        line.find(&ident)
            .map(|start| &line[start..start + ident.len()])
    }
}

/// Extract a *trait-method* identifier from a bare `fn NAME(...)` line
/// (no `pub` prefix — trait methods inherit the trait's visibility).
///
/// R9 F-06: a `_for_test`-suffixed method declared on a `pub trait`
/// leaks into the frozen public surface exactly like a `pub fn`, but
/// [`extract_pub_fn_name`] is blind to it (it requires the `pub `
/// prefix). This extractor matches the bare-`fn` form so the caller can
/// gate it on the enclosing-block being a `pub trait`.
///
/// Only matches a `fn` declaration (optionally `unsafe`/`const`) — NOT
/// a `pub fn` (already handled by [`extract_pub_fn_name`]) and NOT a
/// mid-line `fn` inside a larger expression.
fn extract_trait_method_name(line: &str) -> Option<&str> {
    let l = line.trim_start();
    // Reject `pub fn` — that path is `extract_pub_fn_name`'s job.
    if l.starts_with("pub ") {
        return None;
    }
    let rest = l.strip_prefix("unsafe ").unwrap_or(l);
    let rest = rest
        .strip_prefix("const fn ")
        .or_else(|| rest.strip_prefix("fn "))?;
    let ident: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if ident.is_empty() {
        None
    } else {
        line.find(&ident)
            .map(|start| &line[start..start + ident.len()])
    }
}

/// Walk backward from `idx` to determine whether the current `fn` lives
/// directly inside a `pub trait <Name> { ... }` block. Mirrors the
/// brace-depth tracking of [`in_test_cfg_impl_block`] (no string-literal
/// awareness, but adequate for the trait-declaration shapes here).
///
/// R9 F-06 companion to [`extract_trait_method_name`]: only trait
/// methods whose enclosing block is `pub trait` are on the frozen public
/// surface; a bare `fn` inside a private helper, an inherent `impl`, or
/// a non-`pub` trait is not.
fn in_pub_trait_block(lines: &[&str], idx: usize) -> bool {
    let mut depth: i32 = 0;
    for i in (0..idx).rev() {
        let ln = lines[i];
        for ch in ln.chars().rev() {
            if ch == '}' {
                depth += 1;
            } else if ch == '{' {
                depth -= 1;
                if depth < 0 {
                    // Found the enclosing open-brace; the block-header is line i.
                    let header = ln.split_once('{').map_or(ln, |(a, _)| a);
                    let h = header.trim_start();
                    return h.starts_with("pub trait ") || h.contains(" pub trait ");
                }
            }
        }
    }
    false
}

/// Does the identifier `name` end in `_for_test` / `_for_testing` /
/// `_for_test_<suffix>` (allowing trailing words like `_distinct`,
/// `_signal`, etc.)?
fn is_for_test_pattern(name: &str) -> bool {
    name.contains("_for_test") || name.contains("_for_testing")
}

/// Inspect up to 5 preceding lines of `lines[idx]` for a cfg attribute
/// that gates the item to test-only compilation, accepting either:
///   - `#[cfg(test)]` (strictly test-build only),
///   - `#[cfg(any(test, feature = "testing"))]`,
///   - `#[cfg(any(test, feature = "test-helpers"))]`,
///   - or any `feature = "testing"` / `feature = "test-helpers"` form.
/// Returns true iff a matching cfg is present.
fn has_test_cfg_attr(lines: &[&str], idx: usize) -> bool {
    let mut i = idx;
    while i > 0 {
        i -= 1;
        let ln = lines[i].trim();
        if ln.starts_with("#[cfg(")
            && (ln == "#[cfg(test)]"
                || ln.contains("feature = \"testing\"")
                || ln.contains("feature = \"test-helpers\"")
                || ln.contains("test, feature"))
        {
            return true;
        }
        // attribute-doc continuation OR another attribute — keep walking
        if ln.starts_with('#') || ln.starts_with("///") || ln.starts_with("//") || ln.is_empty() {
            continue;
        }
        // Hit a non-attribute, non-comment, non-empty line — stop.
        return false;
    }
    false
}

/// Walk backward to detect whether the function lives inside an `impl`
/// block whose attribute is `#[cfg(any(test, feature = "testing"))]`.
/// Brace-depth tracking is rough (no string-literal awareness) but
/// adequate for the cases we care about.
fn in_test_cfg_impl_block(lines: &[&str], idx: usize) -> bool {
    let mut depth: i32 = 0;
    for i in (0..idx).rev() {
        let ln = lines[i];
        for ch in ln.chars().rev() {
            if ch == '}' {
                depth += 1;
            } else if ch == '{' {
                depth -= 1;
                if depth < 0 {
                    // found enclosing open-brace; the `impl` line is i
                    let stripped = ln.split_once('{').map_or(ln, |(a, _)| a);
                    if stripped.contains("impl ") {
                        // Walk attributes above
                        for j in (0..i).rev() {
                            let aln = lines[j].trim();
                            if aln.starts_with("#[cfg(")
                                && (aln.contains("feature = \"testing\"")
                                    || aln.contains("feature = \"test-helpers\"")
                                    || aln.contains("test, feature"))
                            {
                                return true;
                            }
                            if aln.starts_with('#')
                                || aln.starts_with("///")
                                || aln.starts_with("//")
                            {
                                continue;
                            }
                            return false;
                        }
                    }
                    return false;
                }
            }
        }
    }
    false
}

#[test]
fn no_ungated_pub_for_test_symbols_in_production_source() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let files = collect_source_files(&workspace_root);
    let mut violations: Vec<String> = Vec::new();

    for f in &files {
        let body = fs::read_to_string(f).expect("read source");
        let lines: Vec<&str> = body.lines().collect();
        for (i, ln) in lines.iter().enumerate() {
            // Two public-surface forms carry `_for_test*` leakage risk:
            //   (1) `pub fn NAME` (inherent / free fn)                    → extract_pub_fn_name
            //   (2) bare `fn NAME` inside a `pub trait` block (trait method) → extract_trait_method_name
            //       + in_pub_trait_block  (R9 F-06 — the guard was blind to form (2)).
            let (name, kind) = if let Some(name) = extract_pub_fn_name(ln) {
                (name, "pub fn")
            } else if let Some(name) = extract_trait_method_name(ln) {
                if !in_pub_trait_block(&lines, i) {
                    continue; // bare `fn` not on a public trait — not frozen surface
                }
                (name, "pub trait fn")
            } else {
                continue;
            };
            if !is_for_test_pattern(name) {
                continue;
            }
            let rel = rel_posix(&workspace_root, f);
            // Exempt-list check
            if EXEMPT_PUB_ITEMS
                .iter()
                .any(|(file, fn_name)| *file == rel && *fn_name == name)
            {
                continue;
            }
            // Direct attribute
            if has_test_cfg_attr(&lines, i) {
                continue;
            }
            // Enclosing impl-block attribute
            if in_test_cfg_impl_block(&lines, i) {
                continue;
            }
            violations.push(format!(
                "{}:{} : {} {} carries `_for_test*` suffix but \
                 no `#[cfg(any(test, feature = \"testing\"))]` (or \
                 `feature = \"test-helpers\"`) attribute, and is not \
                 in EXEMPT_PUB_ITEMS",
                rel,
                i + 1,
                kind,
                name
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "found {} ungated `pub fn .*_for_test*` declarations:\n{}",
        violations.len(),
        violations.join("\n")
    );
}

#[test]
fn exempt_list_entries_all_exist() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    for (file, fn_name) in EXEMPT_PUB_ITEMS {
        let path = workspace_root.join(file);
        let body = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("EXEMPT_PUB_ITEMS entry refers to missing file: {file}"));
        let needle = format!("pub fn {fn_name}");
        assert!(
            body.contains(&needle),
            "EXEMPT_PUB_ITEMS entry `{file}::{fn_name}` not found in source"
        );
    }
}

/// R9 F-06 focused pin: the three `_for_test` methods on the public
/// `benten_eval::SubgraphExt` trait MUST NOT appear in the frozen
/// default public surface (the cargo-public-api baseline is generated
/// with default features — no `testing`), and MUST carry a
/// `#[cfg(any(test, feature = "testing"))]` gate in source.
///
/// Round-8 finding: these three were declared as bare trait `fn`s (not
/// `pub fn`), so the pre-R9 `extract_pub_fn_name` scan was blind to them
/// and they leaked into the frozen surface. This test locks the fix
/// directly (independent of the generic scan's trait-method extension)
/// so a future re-introduction is caught even if the generic walker is
/// refactored.
#[test]
fn subgraph_ext_for_test_methods_absent_from_frozen_surface() {
    const GATED_TRAIT_METHODS: &[&str] = &[
        "cumulative_budget_for_root_for_test",
        "cumulative_budget_for_handle_for_test",
        "has_multiplicative_budget_tracked_for_test",
    ];

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    // (a) The committed cargo-public-api baseline (default features) must
    //     NOT list any of the three on the SubgraphExt trait surface.
    let baseline_path = workspace_root.join("docs/public-api/benten-eval.txt");
    let baseline = fs::read_to_string(&baseline_path)
        .expect("docs/public-api/benten-eval.txt baseline must exist");
    for m in GATED_TRAIT_METHODS {
        let needle = format!("SubgraphExt::{m}");
        assert!(
            !baseline.contains(&needle),
            "test-only trait method `{m}` leaked into the frozen benten-eval \
             public-api baseline (docs/public-api/benten-eval.txt). It must be \
             gated behind #[cfg(any(test, feature = \"testing\"))] and dropped \
             from the default-features baseline."
        );
    }

    // (b) The source declarations must each carry the cfg gate: assert the
    //     `#[cfg(any(test, feature = "testing"))]` line precedes each method.
    let src_path = workspace_root.join("crates/benten-eval/src/subgraph_ext.rs");
    let src = fs::read_to_string(&src_path).expect("subgraph_ext.rs must exist");
    let lines: Vec<&str> = src.lines().collect();
    for m in GATED_TRAIT_METHODS {
        // Two source occurrences per method (trait decl + impl body); every
        // occurrence of the `fn NAME` form must be cfg-gated on its preceding
        // lines.
        let mut seen_decl = false;
        for (i, ln) in lines.iter().enumerate() {
            let t = ln.trim_start();
            if t.starts_with(&format!("fn {m}")) {
                seen_decl = true;
                assert!(
                    has_test_cfg_attr(&lines, i),
                    "`fn {m}` at subgraph_ext.rs:{} is not preceded by a \
                     #[cfg(any(test, feature = \"testing\"))] gate",
                    i + 1
                );
            }
        }
        assert!(
            seen_decl,
            "expected a `fn {m}` declaration in subgraph_ext.rs (finding drifted?)"
        );
    }
}
