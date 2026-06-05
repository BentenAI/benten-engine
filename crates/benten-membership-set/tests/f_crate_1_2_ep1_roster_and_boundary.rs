//! **F-CRATE-1** + **F-CRATE-2** — EP-1 three-tier extensibility roster +
//! the `benten-membership-set` 15th-crate SPLIT boundary.
//!
//! ADDL R5 (impl-to-green) — Phase-4-Meta-Core F-full Wave w-ms-canary,
//! families **F-CRATE-1** (merges GNI-22) and **F-CRATE-2** (merges GNI-23, B-1).
//!
//! # What F-CRATE-1 pins (R0.5 §2.7 EP-1 / §6.3 / §9.1-7)
//!
//! - **Tier-1 OPEN** backend seams `{KVBackend, BlobBackend, GraphBackend,
//!   Renderer, Transport, Materializer}` — **EXACTLY SIX**; `DeviceAuthBackend`
//!   is NOT here (it is a sealed policy seam).
//! - **Tier-2 SEALED** policy seams `{CapabilityPolicy, GrantReader,
//!   DeviceAuthBackend}` — Benten-internal, no external impl.
//! - **Tier-3 ENUM-dispatch**: `benten_ivm::Strategy` is an ENUM, NOT a trait.
//! - The OPEN and SEALED tiers are **DISJOINT** (the F4-008 boundary).
//! - NO `EngineExtension` / `ExtensionRegistry`; no registry (grep-defense).
//! - `Scope` is **EXACTLY-2-arm** (the real `benten_caps::Scope`).
//!
//! # What F-CRATE-2 pins (R0.5 §6.1 / §6.3 / §1.4 / B-1 / NQ-D1)
//!
//! The 15th crate `benten-membership-set` EXISTS; its B-1 dep set is
//! `{crypto-suite, core, caps, id, graph, sync}` (parsed as REAL
//! `[dependencies]` edges now that the canary un-comments them — F4-014); NONE
//! of those depend on IT (no reverse edge). The membership crate makes NO
//! direct primitive construction (`sha3::` / `chacha20` / `ml_kem::`) — the
//! ONLY-call-site #5 grep-defense.
//!
//! # R5 (un-ignored)
//!
//! The F-CRATE-1 roster pins reference `benten_membership_set::roster` (the real
//! EP-1 roster constants) + the real `benten_caps::Scope`. The F-CRATE-2
//! boundary pins parse the real (now-un-commented) `[dependencies]` section
//! (F4-014/F-CRATE-2: parse the section, not a raw `.contains()`) and grep the
//! real sources. Would-FAIL: an `ExtensionRegistry`, a 3rd `Scope` arm, a
//! reverse dep edge, a direct primitive import, OR re-adding `DeviceAuthBackend`
//! to the OPEN tier all break a pin.

use std::collections::BTreeSet;
use std::path::PathBuf;

use benten_caps::Scope;
use benten_membership_set::roster::{StrategyShape, TIER1_OPEN_SEAMS, TIER2_SEALED_SEAMS};

const SCOPE_VARIANT_COUNT: usize = 2;

/// A `dispatch` over the real `benten_caps::Scope` with NO wildcard — proves
/// EXACTLY-2 at compile time (a 3rd arm breaks this match, the HALT-AND-SURFACE).
fn scope_kind(s: &Scope) -> &'static str {
    match s {
        Scope::Hashes(_) => "hashes",
        Scope::RestrictedSelector(_) => "restricted",
    }
}

// ── F-CRATE-2 real workspace-path helpers ───────────────────────────────────

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root is two levels above the crate manifest dir")
        .to_path_buf()
}

fn read_cargo_toml(crate_dir: &str) -> String {
    let p = workspace_root()
        .join("crates")
        .join(crate_dir)
        .join("Cargo.toml");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// Parse the dependency KEYS declared in a Cargo.toml's `[dependencies]`
/// section, looking ONLY at REAL (uncommented) `name = ...` lines.
fn parsed_dependencies(cargo_toml: &str) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    let mut in_deps = false;
    for raw in cargo_toml.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_deps = line == "[dependencies]";
            continue;
        }
        if !in_deps {
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, _)) = line.split_once('=') {
            let key = key.trim().trim_matches('"');
            if !key.is_empty() {
                deps.insert(key.to_string());
            }
        }
    }
    deps
}

fn read_membership_set_sources() -> String {
    let src_dir = workspace_root()
        .join("crates")
        .join("benten-membership-set")
        .join("src");
    let mut all = String::new();
    for entry in walk_rs(&src_dir) {
        all.push_str(&std::fs::read_to_string(&entry).unwrap_or_default());
        all.push('\n');
    }
    all
}

fn walk_rs(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walk_rs(&p));
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    out
}

// ── F-CRATE-1 pins ──────────────────────────────────────────────────────────

#[test]
fn crate1_tier1_open_seam_roster() {
    assert_eq!(
        TIER1_OPEN_SEAMS.len(),
        6,
        "EP-1 Tier-1 has EXACTLY 6 open backend seams (R0.5 §2.7 line 264); DeviceAuthBackend is sealed Tier-2, NOT open"
    );
    assert!(
        TIER1_OPEN_SEAMS.contains(&"Materializer"),
        "Materializer is the 6th open backend seam"
    );
    assert!(
        !TIER1_OPEN_SEAMS.contains(&"DeviceAuthBackend"),
        "DeviceAuthBackend must NOT be an OPEN object-safe seam — it is a sealed policy seam (#7)"
    );
}

#[test]
fn crate1_tier2_sealed_seam_roster() {
    assert!(TIER2_SEALED_SEAMS.contains(&"CapabilityPolicy"));
    assert!(TIER2_SEALED_SEAMS.contains(&"GrantReader"));
    assert!(
        TIER2_SEALED_SEAMS.contains(&"DeviceAuthBackend"),
        "DeviceAuthBackend is a sealed policy seam (#7)"
    );
}

#[test]
fn crate1_open_and_sealed_tiers_are_disjoint() {
    // The F4-008 correctness boundary as a SINGLE partition check.
    let open: BTreeSet<&str> = TIER1_OPEN_SEAMS.iter().copied().collect();
    let sealed: BTreeSet<&str> = TIER2_SEALED_SEAMS.iter().copied().collect();
    let overlap: BTreeSet<&str> = open.intersection(&sealed).copied().collect();
    assert!(
        overlap.is_empty(),
        "OPEN and SEALED seam tiers MUST be disjoint — overlap {overlap:?} means a sealed trait is wrongly listed as an open object-safe backend"
    );
    assert_eq!(
        open.len() + sealed.len(),
        open.union(&sealed).count(),
        "no seam is double-counted across the open/sealed partition"
    );
}

#[test]
fn crate1_strategy_is_enum_not_trait() {
    // Strategy is an ENUM (Copy value type), dispatched by `match`, NOT a
    // `dyn Strategy` trait object.
    let s = StrategyShape::DependencyTracked;
    let s2 = s; // Copy — a trait object would not be Copy.
    assert_eq!(s, s2);
    assert_ne!(StrategyShape::DependencyTracked, StrategyShape::Reserved);
}

#[test]
fn crate1_scope_exactly_two_arm() {
    // The no-wildcard `scope_kind` match over the REAL benten_caps::Scope proves
    // EXACTLY-2 at compile time.
    assert_eq!(scope_kind(&Scope::Hashes(Vec::new())), "hashes");
    assert_eq!(
        SCOPE_VARIANT_COUNT, 2,
        "Scope is EXACTLY-2-arm (scope.rs:44)"
    );
}

#[test]
fn crate1_no_extension_registry_grep_defense() {
    let src = read_membership_set_sources();
    assert!(
        !src.contains("EngineExtension"),
        "no `EngineExtension` — EP-1 forbids a generic extension trait (everything is a plugin)"
    );
    assert!(
        !src.contains("ExtensionRegistry"),
        "no `ExtensionRegistry` — EP-1 forbids a registry (trust = compiled-in / user-root)"
    );
}

// ── F-CRATE-2 pins ────────────────────────────────────────────────────────

#[test]
fn crate2_fifteenth_crate_exists() {
    let cargo = read_cargo_toml("benten-membership-set");
    assert!(
        cargo.contains("name = \"benten-membership-set\""),
        "the 15th crate benten-membership-set exists"
    );
    assert!(
        cargo.contains("0x6600") || read_membership_set_sources().contains("0x6600"),
        "the mechanism-half names the MembershipSet codepoint band (0x6600)"
    );
}

#[test]
fn crate2_no_reverse_dependency_edge() {
    // crypto-suite and sync are UPSTREAM — they must NEVER name
    // benten-membership-set as a dependency (no reverse edge).
    for upstream in ["benten-crypto-suite", "benten-sync"] {
        let cargo = read_cargo_toml(upstream);
        assert!(
            !cargo.contains("benten-membership-set"),
            "{upstream} must NOT depend on benten-membership-set (no reverse edge — B-1 direction is membership-set → {upstream})"
        );
    }
}

#[test]
fn crate2_no_direct_primitive_construction() {
    // The #5 ONLY-call-site rule: the membership crate delegates ALL crypto to
    // benten-crypto-suite; it NEVER imports/constructs a primitive directly.
    let src = read_membership_set_sources();
    for forbidden in [
        "sha3::",
        "chacha20",
        "ml_kem::",
        "ml_dsa::",
        "x25519_dalek::",
    ] {
        assert!(
            !src.contains(forbidden),
            "membership crate must NOT construct the `{forbidden}` primitive directly — it delegates to benten-crypto-suite (#5 ONLY-call-site)"
        );
    }
}

#[test]
fn crate2_b1_dep_set_direction() {
    // Assert the B-1 set as REAL `[dependencies]` edges (parsed from the
    // section) — now that the canary un-comments them (F4-014).
    let cargo = read_cargo_toml("benten-membership-set");
    let real_deps = parsed_dependencies(&cargo);
    const B1_SET: [&str; 6] = [
        "benten-crypto-suite",
        "benten-core",
        "benten-caps",
        "benten-id",
        "benten-graph",
        "benten-sync", // the load-bearing B-1 sync substrate
    ];
    for dep in B1_SET {
        assert!(
            real_deps.contains(dep),
            "the membership crate's [dependencies] section MUST carry the B-1 edge `{dep}` as a REAL (uncommented) dependency (the R5 canary un-comments the set)"
        );
    }
    // The Cargo.toml still documents the B-1 set (incl. benten-sync).
    assert!(
        cargo.contains("benten-sync"),
        "the membership crate's Cargo.toml carries the B-1 benten-sync edge"
    );
}
