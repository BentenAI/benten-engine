//! Phase 2a R3 security — Inv-11 system-zone prefix-table drift CI guard.
//!
//! **Attack class (atk-4 / sec-r1-3).** Every new `system:*` label literal
//! added in any crate has TWO places to update: the writer code site AND the
//! `SYSTEM_ZONE_PREFIXES` const in `benten-engine::system_zones`. Drift ⇒
//! silent capability bypass: a user-authored subgraph can reach a freshly-
//! minted system-zone label that the phf probe doesn't know about because
//! the writer added the label but forgot to register it.
//!
//! **Prerequisite (attacker capability).** Fork Benten, add a new
//! `system:<NewLabel>` writer in some crate, forget to update the prefix
//! table. On upstream merge, Inv-11 lets user subgraphs read/write that
//! new label.
//!
//! **Attack sequence.** CI runs. Drift test should detect that
//! `"system:<NewLabel>"` appears in `crates/**/*.rs` but is absent from
//! `SYSTEM_ZONE_PREFIXES` → fail the build. Without this guard, CI green +
//! vulnerability ships.
//!
//! **Impact.** Silent capability bypass on every `system:*` label added
//! after Phase 1.
//!
//! **Recommended mitigation.** Grep-based workspace test enumerates every
//! `"system:<literal>"` string in `crates/**/*.rs`, asserts each one is a
//! prefix of (or matches) at least one entry in `SYSTEM_ZONE_PREFIXES`.
//! Paired CI workflow `.github/workflows/inv-11-system-zone-drift.yml`
//! re-runs this on every PR.
//!
//! **Red-phase correct:** `SYSTEM_ZONE_PREFIXES` const does not yet exist
//! (G1-A creates the skeleton; G5-B-i populates). The test is gated with
//! `#[ignore]` until that skeleton lands; the walker body asserts the
//! walker works so regressions in the walker itself are still caught.
//!
//! Test name: `all_system_zone_writers_registered_in_prefix_table`
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};

/// Walk `crates/*/src/**/*.rs` under the workspace root, collecting every
/// distinct `"system:<Label>"` literal. Returns sorted+deduped labels.
fn collect_system_zone_literals(workspace_root: &Path) -> Vec<String> {
    let crates_dir = workspace_root.join("crates");
    let mut labels: Vec<String> = Vec::new();
    walk_rs(&crates_dir, &mut |src: &str| {
        let mut cursor = 0usize;
        while let Some(idx) = src[cursor..].find("\"system:") {
            let start = cursor + idx + 1;
            let tail = &src[start..];
            if let Some(end_rel) = tail.find('"') {
                let literal = &tail[..end_rel];
                if literal.starts_with("system:") && literal.len() > "system:".len() {
                    labels.push(literal.to_string());
                }
                cursor = start + end_rel + 1;
            } else {
                break;
            }
        }
    });
    labels.sort();
    labels.dedup();
    labels
}

fn walk_rs(dir: &Path, visit: &mut dyn FnMut(&str)) {
    if !dir.is_dir() {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == "tests" || name.starts_with('.') {
                continue;
            }
            walk_rs(&path, visit);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs")
            && let Ok(src) = fs::read_to_string(&path)
        {
            visit(&src);
        }
    }
}

fn workspace_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let p = PathBuf::from(manifest_dir);
    match p.parent().and_then(|q| q.parent()) {
        Some(root) => root.to_path_buf(),
        None => p,
    }
}

/// Sanity check on the walker — even in red-phase, the walker must find the
/// Phase-1 PascalCase labels so the drift-guard's implementation is known to
/// work when the const lands. Guards against a silently-broken walker passing
/// vacuously.
#[test]
fn drift_walker_finds_phase_1_pascalcase_labels() {
    let root = workspace_root();
    let literals = collect_system_zone_literals(&root);
    let required = [
        "system:CapabilityGrant",
        "system:CapabilityRevocation",
        "system:IVMView",
    ];
    for req in &required {
        assert!(
            literals.iter().any(|l| l == req),
            "walker did not surface `\"{req}\"` — walker regression or the \
             label was renamed (which would break Inv-11 silently). \
             Collected labels: {literals:?}"
        );
    }
}

/// CI drift guard: every `"system:*"` literal that appears in any `.rs` file
/// under `crates/**/src/` MUST be covered by an entry in
/// `benten_engine::system_zones::SYSTEM_ZONE_PREFIXES` (match-or-prefix-of).
///
/// G1-A landed the skeleton file + const; G5-B-i will populate additional
/// prefixes as they surface. The `#[ignore]` was dropped at G1-A per the
/// in-file instructions because `SYSTEM_ZONE_PREFIXES` now exists. Any
/// drift between workspace literals and the const fires a typed assert
/// with the missing labels.
#[test]
fn all_system_zone_writers_registered_in_prefix_table() {
    let root = workspace_root();
    let literals = collect_system_zone_literals(&root);
    assert!(
        !literals.is_empty(),
        "drift guard found zero `\"system:*\"` literals — walker is broken \
         or workspace layout changed. Root: {}",
        root.display()
    );

    let missing: Vec<&String> = literals
        .iter()
        .filter(|lit| {
            !benten_engine::system_zones::SYSTEM_ZONE_PREFIXES
                .iter()
                .any(|p| lit.starts_with(p) || *p == lit.as_str())
        })
        .collect();

    assert!(
        missing.is_empty(),
        "drift: literals present in workspace but absent from \
         SYSTEM_ZONE_PREFIXES — add them in \
         crates/benten-engine/src/system_zones.rs. Missing: {missing:?}"
    );
}
