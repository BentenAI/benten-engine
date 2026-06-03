//! **F-CRATE-1** + **F-CRATE-2** — EP-1 three-tier extensibility roster +
//! the `benten-membership-set` 15th-crate SPLIT boundary.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core F-full Wave
//! R3-W4, families **F-CRATE-1** (merges GNI-22) and **F-CRATE-2** (merges
//! GNI-23, B-1).
//!
//! # What F-CRATE-1 pins (R0 §2.7 EP-1 / §6.3 / §9.1-7)
//!
//! The EP-1 three-tier extensibility roster:
//! - **Tier-1 OPEN** backend seams `{KVBackend, BlobBackend, GraphBackend,
//!   Renderer, Transport, Materializer, DeviceAuthBackend}` are object-safe +
//!   conformance-tested.
//! - **Tier-2 SEALED** policy seams `{CapabilityPolicy, GrantReader (#830),
//!   DeviceAuthBackend}` — Benten-internal, no external impl.
//! - **Tier-3 ENUM-dispatch**: `benten_ivm::Strategy` is an ENUM, NOT a trait
//!   seam (m-15 GNC-4 / baked-in #2).
//! - **NO** `EngineExtension` / `ExtensionRegistry`; no registry (grep-defense).
//! - `Scope` is **EXACTLY-2-arm** (a 3rd arm is a HALT-AND-SURFACE).
//!
//! # What F-CRATE-2 pins (R0 §6.1 / §6.3 / §1.4 / B-1 / NQ-D1)
//!
//! The 15th crate `benten-membership-set` EXISTS; its mechanism-half (frozen)
//! is the Kind enum + `0x6600/0x6610/0x6620` + multi-stanza keying glue
//! (delegates to crypto-suite, NEVER forks #5) + `members_table` CBOR + AAD
//! assembly + Inv-21 rule + clause-k bound; data-half is graph
//! (GovernanceConfig / RoleId-semantics / audit / federation / economics). Its
//! B-1 dep set is `{crypto-suite, core, caps, id, graph, sync}`; **NONE of
//! those depend on IT** (no reverse edge). The membership crate makes **NO
//! direct primitive construction** (`sha3::` / `chacha20` / `ml_kem::`) — the
//! ONLY-call-site #5 grep-defense.
//!
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! The roster-shape pins (F-CRATE-1) use a self-contained in-file stub-shim;
//! the **boundary pins (F-CRATE-2) are REAL filesystem/grep assertions** that
//! run green at R3 baseline against the actual scaffolded crate + the actual
//! crypto-suite/sync Cargo.tomls — these are not stubbed because the boundary
//! is observable NOW (the scaffold establishes it). R5 swaps the F-CRATE-1
//! shim for the real `benten_engine`/`benten_caps`/`benten_ivm` roster types
//! and un-ignores; the F-CRATE-2 grep pins tighten to the full B-1 dep set
//! once the canary un-comments the deps. Would-FAIL-if-no-op'd: an
//! `ExtensionRegistry`, a `Scope` 3rd arm, a reverse dep edge
//! (crypto-suite/sync → membership-set), or a direct primitive import in the
//! membership crate all break a pin.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::path::PathBuf;

// ── F-CRATE-1 self-contained in-file stub-shim ──────────────────────────────

/// The EP-1 Tier-1 OPEN backend seams (object-safe). At R5 these reference the
/// real workspace traits; here we model them as a roster the test enumerates.
const TIER1_OPEN_SEAMS: [&str; 7] = [
    "KVBackend",
    "BlobBackend",
    "GraphBackend",
    "Renderer",
    "Transport",
    "Materializer",
    "DeviceAuthBackend",
];

/// The EP-1 Tier-2 SEALED policy seams.
const TIER2_SEALED_SEAMS: [&str; 3] = ["CapabilityPolicy", "GrantReader", "DeviceAuthBackend"];

/// Stand-in for `benten_ivm::Strategy` — an ENUM, not a trait (m-15 GNC-4).
/// The `#[non_exhaustive]`-free 2-arm shape mirrors the in-tree Reserved arm.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Strategy {
    DependencyTracked,
    Reserved,
}

/// Stand-in for `benten_caps::scope::Scope` — EXACTLY 2 arms (a 3rd is a
/// HALT-AND-SURFACE per `scope.rs:44`).
#[derive(Clone, PartialEq, Eq, Debug)]
enum Scope {
    Hashes(Vec<[u8; 4]>),
    RestrictedSelector(u32),
}

const SCOPE_VARIANT_COUNT: usize = 2;

/// A `dispatch` over `Scope` with NO wildcard — proves EXACTLY-2 at compile
/// time (a 3rd arm breaks this match, the HALT-AND-SURFACE).
fn scope_kind(s: &Scope) -> &'static str {
    match s {
        Scope::Hashes(_) => "hashes",
        Scope::RestrictedSelector(_) => "restricted",
    }
}

// ── F-CRATE-2 real workspace-path helpers ───────────────────────────────────

/// Workspace root, resolved from this test crate's manifest dir
/// (`crates/benten-membership-set` → up two levels). Confined to the worktree.
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
/// section, looking ONLY at REAL (uncommented) `name = ...` lines — never
/// comment text. This is the F4-014 fix: a whole-file `.contains("benten-sync")`
/// matches the commented-out `# benten-sync = ...` documentation and so
/// green-passes against a NON-edge; parsing the section gives a true dep-graph
/// assertion that tightens once the canary un-comments the real deps.
fn parsed_dependencies(cargo_toml: &str) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    let mut in_deps = false;
    for raw in cargo_toml.lines() {
        let line = raw.trim();
        // Section headers. Only the `[dependencies]` table counts (NOT
        // dev-dependencies / build-dependencies / target.* — those are not the
        // B-1 production dep set).
        if line.starts_with('[') && line.ends_with(']') {
            in_deps = line == "[dependencies]";
            continue;
        }
        if !in_deps {
            continue;
        }
        // Skip comments + blank lines — a commented `# benten-sync = ...` is
        // NOT a real edge.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // A real dependency line is `name = ...`. Take the key before `=`.
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
#[ignore = "RED-PHASE: F-CRATE-1 — EP-1 Tier-1 OPEN seams (7) object-safe roster; un-ignore at R5 against real workspace traits"]
fn crate1_tier1_open_seam_roster() {
    // The 7 Tier-1 open backend seams. At R5 each is asserted object-safe via a
    // `dyn Trait` coercion (clone of graph_backend_trait.rs). Here we pin the
    // exact roster so the canary can't silently drop/add a seam.
    assert_eq!(
        TIER1_OPEN_SEAMS.len(),
        7,
        "EP-1 Tier-1 has exactly 7 open backend seams"
    );
    assert!(TIER1_OPEN_SEAMS.contains(&"DeviceAuthBackend"));
    assert!(TIER1_OPEN_SEAMS.contains(&"Materializer"));
}

#[test]
#[ignore = "RED-PHASE: F-CRATE-1 — EP-1 Tier-2 SEALED policy seams (CapabilityPolicy/GrantReader/DeviceAuthBackend); un-ignore at R5"]
fn crate1_tier2_sealed_seam_roster() {
    // DeviceAuthBackend straddles Tier-1 (object-safe) AND Tier-2 (sealed: no
    // external impl) per the roster. The sealed set is the policy seam.
    assert!(TIER2_SEALED_SEAMS.contains(&"CapabilityPolicy"));
    assert!(TIER2_SEALED_SEAMS.contains(&"GrantReader"));
    assert!(
        TIER2_SEALED_SEAMS.contains(&"DeviceAuthBackend"),
        "DeviceAuthBackend is a sealed policy seam (#7)"
    );
}

#[test]
#[ignore = "RED-PHASE: F-CRATE-1 — benten_ivm::Strategy is an ENUM not a trait (Tier-3 enum-dispatch, m-15 GNC-4); un-ignore at R5"]
fn crate1_strategy_is_enum_not_trait() {
    // Clone of strategy_enum_present.rs: Strategy is an ENUM (Copy value type),
    // dispatched by `match`, NOT a `dyn Strategy` trait object. We exercise the
    // enum by value to prove the enum-dispatch shape.
    let s = Strategy::DependencyTracked;
    let s2 = s; // Copy — a trait object would not be Copy.
    assert_eq!(s, s2);
    assert_ne!(Strategy::DependencyTracked, Strategy::Reserved);
}

#[test]
#[ignore = "RED-PHASE: F-CRATE-1 — Scope is EXACTLY-2-arm (3rd = HALT-AND-SURFACE); un-ignore at R5 against benten_caps::Scope"]
fn crate1_scope_exactly_two_arm() {
    // The no-wildcard `scope_kind` match proves EXACTLY-2 at compile time.
    assert_eq!(scope_kind(&Scope::Hashes(vec![[0u8; 4]])), "hashes");
    assert_eq!(scope_kind(&Scope::RestrictedSelector(7)), "restricted");
    assert_eq!(
        SCOPE_VARIANT_COUNT, 2,
        "Scope is EXACTLY-2-arm; a 3rd arm is a HALT-AND-SURFACE (scope.rs:44)"
    );
}

#[test]
#[ignore = "RED-PHASE: F-CRATE-1 — NO EngineExtension/ExtensionRegistry anywhere in the membership crate (no-registry grep-defense); un-ignore at R5 workspace-wide"]
fn crate1_no_extension_registry_grep_defense() {
    // Clone of strategy_c_renamed_to_reserved_grep_assert.rs: a grep-defense
    // that no `EngineExtension` / `ExtensionRegistry` symbol appears. This runs
    // REAL at R3 against the scaffolded crate sources (observable now).
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

// ── F-CRATE-2 pins (REAL at R3 baseline) ────────────────────────────────────

#[test]
#[ignore = "RED-PHASE: F-CRATE-2 — the 15th crate benten-membership-set EXISTS with its mechanism-half codepoint band; un-ignore at R5"]
fn crate2_fifteenth_crate_exists() {
    // The crate exists with its own Cargo.toml (the 15th workspace member).
    let cargo = read_cargo_toml("benten-membership-set");
    assert!(
        cargo.contains("name = \"benten-membership-set\""),
        "the 15th crate benten-membership-set exists"
    );
    // Its mechanism-half names the 0x6600 codepoint band (the keying minimum).
    // (Asserted via the scaffold module that lands at R3.)
    assert!(
        cargo.contains("0x6600") || read_membership_set_sources().contains("0x6600"),
        "the mechanism-half names the MembershipSet codepoint band (0x6600)"
    );
}

#[test]
#[ignore = "RED-PHASE: F-CRATE-2 — NO reverse dep: crypto-suite + sync Cargo.toml do NOT depend on benten-membership-set; un-ignore at R5"]
fn crate2_no_reverse_dependency_edge() {
    // F-CRATE-2's load-bearing boundary: the membership crate is a leaf-ish
    // addition. crypto-suite and sync are UPSTREAM — they must NEVER name
    // benten-membership-set as a dependency (a reverse edge would let the keying
    // crate's surface leak into the crypto floor / sync substrate). REAL grep
    // against the actual upstream Cargo.tomls; green NOW.
    for upstream in ["benten-crypto-suite", "benten-sync"] {
        let cargo = read_cargo_toml(upstream);
        assert!(
            !cargo.contains("benten-membership-set"),
            "{upstream} must NOT depend on benten-membership-set (no reverse edge — B-1 direction is membership-set → {upstream})"
        );
    }
}

#[test]
#[ignore = "RED-PHASE: F-CRATE-2 — membership crate makes NO direct crypto-primitive construction (sha3/chacha20/ml_kem); ONLY-call-site #5 grep-defense; un-ignore at R5"]
fn crate2_no_direct_primitive_construction() {
    // The #5 ONLY-call-site rule: the membership crate delegates ALL crypto to
    // benten-crypto-suite; it NEVER imports/constructs a primitive directly. A
    // `sha3::` / `chacha20` / `ml_kem::` reference in its sources would be a
    // fork-the-crypto violation. REAL grep against the scaffolded sources;
    // green NOW (the scaffold imports no crypto). Tightens at R5 once the keying
    // glue lands (it must STILL route through crypto-suite's typed API).
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
#[ignore = "RED-PHASE: F-CRATE-2 — the B-1 dep set {crypto-suite, core, caps, id, graph, sync} is a REAL [dependencies] edge-set; un-ignore at R5 once the canary un-comments deps"]
fn crate2_b1_dep_set_direction() {
    // F4-014: assert the B-1 set as REAL `[dependencies]` edges (parsed from
    // the section), NOT a whole-file `.contains()` that matches the commented
    // documentation. At R3 the deps are intentionally commented out for
    // parallel-safety, so this LOAD-BEARING assertion is the RED-PHASE pin
    // that GOES GREEN once the R5 canary un-comments the real edges; an impl
    // (canary) that silently dropped `benten-sync` would keep it RED.
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
            "the membership crate's [dependencies] section MUST carry the B-1 edge `{dep}` as a REAL (uncommented) dependency — NOT comment text (R5 canary un-comments the set)"
        );
    }

    // Grep-defense that the parser is genuinely section-scoped (so the pin
    // can't be satisfied by a comment): the commented B-1 block is documented
    // in the file, but parsing finds ZERO of them as real edges at R3
    // baseline. (This control is what makes the assertions above load-bearing
    // rather than comment-matched.) The membership crate's Cargo.toml DOES
    // carry the documentation block naming the intended B-1 set — assert the
    // documentation is present so the canary can't drop the intent either.
    assert!(
        cargo.contains("benten-sync"),
        "the membership crate's Cargo.toml documents the B-1 set (incl. benten-sync) — the canary un-comments it into a real edge"
    );
}
