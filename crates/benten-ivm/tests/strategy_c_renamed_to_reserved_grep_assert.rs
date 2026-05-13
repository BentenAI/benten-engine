//! G23-0a `Strategy::C` → `Strategy::Reserved` rename — full-source
//! grep-assert.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.2 row 3
//! (arch-r1-14).
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-0a +
//! §7 backlog row "`benten-ivm::Strategy::C` Z-set/DBSP semantics".
//!
//! ## What this asserts
//!
//! arch-r1-14 names a one-line rename to close ambiguity per
//! CRATES-DEEP-DIVE §4 — the variant remains reserved-not-implemented;
//! only the spelling changes. Post-G23-0a there MUST be ZERO
//! `Strategy::C` references in the production source tree.
//!
//! Source-wide scope (NOT just `strategy.rs`):
//! - All `crates/benten-ivm/src/*.rs` — dispatch, kernel, testing,
//!   per-view files.
//! - `crates/benten-ivm/INTERNALS.md` is INFORMATIONAL (rustdoc /
//!   architecture sketch); the rename sweep MUST touch it per
//!   pim-1 §3.5b HARDENED post-fix doc-coupling pre-flight (the
//!   reviewer-pre-flight pin in this file's companion ensures the
//!   sweep is complete).
//! - Test files (`crates/benten-ivm/tests/*.rs`) are EXCLUDED from
//!   the assert — pre-rename tests (e.g. `strategy_c_reserved.rs`)
//!   that pin the OLD spelling stay until they themselves get
//!   renamed; arch-r1-14 names the SOURCE rename, not the test
//!   sweep.
//!
//! ## §3.6b would-FAIL-if-no-op'd
//!
//! A no-op rename (e.g. adding `Reserved` as an alias while keeping
//! `C` in source) would have BOTH `Strategy::C` AND `Strategy::Reserved`
//! references; this pin FAILS. The complementary positive pin in
//! `ivm_generalized_kernel_no_new_strategy_variant.rs` asserts
//! `Reserved` is present. Together they pin the rename is complete.
//!
//! ## RED-PHASE
//!
//! Closes at R5 G23-0a. Un-ignore per pim-12 §3.6e.

#![allow(clippy::unwrap_used)]

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

/// Collect all `.rs` files under `crates/benten-ivm/src/`.
fn collect_source_rs_files() -> Vec<PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is always set under cargo test");
    let src = PathBuf::from(&manifest_dir).join("src");

    let mut out = Vec::new();
    walk_collect_rs(&src, &mut out);
    assert!(
        !out.is_empty(),
        "no .rs files found under {} — manifest layout drifted?",
        src.display()
    );
    out
}

fn walk_collect_rs(dir: &PathBuf, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_collect_rs(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn strategy_c_rename_complete_in_source_tree() {
    let files = collect_source_rs_files();

    let mut offending: Vec<(PathBuf, Vec<(usize, String)>)> = Vec::new();

    for path in &files {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut hits: Vec<(usize, String)> = Vec::new();
        for (i, line) in source.lines().enumerate() {
            // Match `Strategy::C` as a code reference. Use a substring
            // match — false positives possible in doc-comments naming
            // the OLD spelling for historical context, so we allow
            // those by skipping lines that begin with `//!` or `///`
            // ONLY when they end with a clear "historical" marker.
            // Cleanest pin: zero hits anywhere (including rustdoc), so
            // the rename sweep covers documentation references too.
            if !line.contains("Strategy::C") {
                continue;
            }
            // The only acceptable mention is `Strategy::Cancellation`
            // or `Strategy::Cached` etc. — guard against false positive
            // by requiring the match-suffix to be word-boundary `C` not
            // followed by an identifier character.
            let positions: Vec<usize> = line
                .match_indices("Strategy::C")
                .map(|(idx, _)| idx)
                .collect();
            for pos in positions {
                let suffix_idx = pos + "Strategy::C".len();
                let next_char = line.as_bytes().get(suffix_idx).copied();
                let is_word_boundary = match next_char {
                    None => true,
                    Some(b) => !(b as char).is_alphanumeric() && b != b'_',
                };
                if is_word_boundary {
                    hits.push((i + 1, line.to_string()));
                    break;
                }
            }
        }
        if !hits.is_empty() {
            offending.push((path.clone(), hits));
        }
    }

    if !offending.is_empty() {
        let mut msg = String::from(
            "Strategy::C references found in production source post-G23-0a \
             rename (arch-r1-14). Sweep MUST replace with Strategy::Reserved:\n",
        );
        for (path, hits) in &offending {
            let _ = writeln!(msg, "  {}:", path.display());
            for (ln, line) in hits {
                let _ = writeln!(msg, "    {ln}: {line}");
            }
        }
        panic!("{msg}");
    }
}

#[test]
fn strategy_reserved_referenced_in_source_tree() {
    // Companion-positive: the rename target MUST appear somewhere in
    // the source tree (strategy.rs minimum; testing.rs typically too
    // since `try_construct_view_with_strategy` matches on it).
    let files = collect_source_rs_files();
    let mut found_count = 0;
    for path in &files {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for line in source.lines() {
            if line.contains("Strategy::Reserved") || line.contains("Reserved") {
                // Word-boundary check — `Reserved` alone may match
                // `ReservedSomething`; the stricter pin is `Strategy::
                // Reserved` substring + the strategy.rs body identifier
                // `Reserved`.
                if line.contains("Strategy::Reserved") {
                    found_count += 1;
                    break;
                }
            }
        }
    }
    assert!(
        found_count >= 1,
        "Strategy::Reserved must appear in production source post-G23-0a; \
         found 0 references across {} files. Rename arch-r1-14 incomplete.",
        files.len()
    );
}
