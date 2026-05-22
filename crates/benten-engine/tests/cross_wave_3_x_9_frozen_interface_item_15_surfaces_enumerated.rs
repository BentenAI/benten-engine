//! Phase-4-Meta-Core — R3-W5 — Cross-wave integration test §4-F:
//! G-CORE-3 × G-CORE-9 freeze — §1.A.FROZEN item 15 surfaces all
//! enumerated. The 10 sub-clauses (a-j) of the S&C surface are each
//! present in the frozen-interface document AND validated by a
//! structural pin. `cargo-public-api` baseline includes the new
//! public types (`SubgraphSpec`, `RestrictedSpec`, `Scope`,
//! `AuthorizationGrant`, `EncryptionClass`, `DropBundle`,
//! `walk_share_scope`). Production-arm. Owned-by: G-CORE-9.
//!
//! ============================================================================
//! RED-PHASE STATUS
//! ============================================================================
//!
//! All pins `#[ignore = "RED-PHASE: un-ignore at G-CORE-9 FREEZE wave
//! (the §1.A.FROZEN item 15 surface inventory landing)"]`.
//!
//! ============================================================================
//! GROUND-TRUTH (synced HEAD c9c11c56)
//! ============================================================================
//!
//!   * `docs/V1-FROZEN-INTERFACE.md` does NOT exist at HEAD (the
//!     §1.A.FROZEN proposed-name). The frozen-interface contract is
//!     a G-CORE-9 deliverable; pre-freeze it is captured ONLY in the
//!     plan-doc `.addl/phase-4-meta/00-implementation-plan.md`
//!     §1.A.FROZEN section.
//!   * `SubgraphSpec` / `RestrictedSpec` / `Scope` / `AuthorizationGrant`
//!     / `EncryptionClass` / `DropBundle` / `walk_share_scope` —
//!     NONE EXIST at HEAD (G-CORE-3 not yet built).
//!
//! ============================================================================
//! WHY A STRUCTURAL BACKSTOP PIN IS CORRECT HERE (pim-18 waiver — R2 §4-F)
//! ============================================================================
//!
//! r2-test-landscape.md §4-F is necessarily structural: "a structural
//! pin enumerates the 10 sub-clauses (a-j) of the S&C surface;
//! missing item → CI red". A behavioral test would be WRONG here:
//! the property is a SURFACE-PRESENCE invariant (is the type
//! enumerated in the FROZEN-INTERFACE document AND in the cargo-
//! public-api baseline), not a runtime behavior. Same reasoning class
//! as `tf12_benten_engine_freeze_conformance_cargo_public_api.rs` and
//! `tf12_benten_engine_freeze_conformance_non_exhaustive_907.rs`.
//!
//! ============================================================================
//! §3.6g LITERAL discipline checklist
//! ============================================================================
//!
//!  1. §3.5b HARDENED — implementer sweeps adjacent docs at freeze.
//!  2. §3.6b sub-rule 4 — SPECIFIC arm = each of the 10 sub-clauses
//!     (a-j) is independently checked; OBSERVABLE = a structural
//!     pin fires on missing surface; WOULD-FAIL if any sub-clause
//!     is silently dropped from the freeze (the load-bearing
//!     §1.A.FROZEN escape-valve property: a frozen-surface mutation
//!     fails CI structurally, not just policy).
//!  3. §3.6e — RED-PHASE; un-ignore at G-CORE-9.
//!  4. §3.6f (pim-18) — structural backstop pin per §4-F SHAPE-not-
//!     SUBSTANCE waiver above.
//!  5. §3.13 — per-test locals.
//!  6. §3.5g — no new ErrorCode here; the implementer mints
//!     surface-named errors which §3.5g rule-mirrors.
//!  7. §3.5n — R3-W5 verified the 10 sub-clauses live in plan-doc
//!     §1.A.FROZEN item 15 (a-j) at the cited path.
//!
//! Pin source: R2-test-landscape.md §4-F + plan §1.A.FROZEN item 15
//! sub-clauses (a) SubgraphSpec / (b) RestrictedSpec / (c) Scope /
//! (d) AuthorizationGrant / (e) Encryption-class / (f) Two-path KDF /
//! (g) Two-CID + chunk-AEAD / (h) walker location / (i) revocation-
//! reach doc / (j) resolver eval model.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// The 10 sub-clauses of §1.A.FROZEN item 15 (S&C public surface).
/// Each sub-clause must be present in the frozen-interface document
/// after G-CORE-9 ratifies it. R3-W5 author verified this is the
/// canonical list from
/// `.addl/phase-4-meta/00-implementation-plan.md` §1.A.FROZEN item 15.
const ITEM_15_SUBCLAUSES: &[(&str, &str)] = &[
    ("a", "SubgraphSpec primitive"),
    ("b", "RestrictedSpec"),
    ("c", "Scope enum"),
    ("d", "AuthorizationGrant"),
    ("e", "Encryption-class enum"),
    ("f", "Two-path key-derivation"),
    ("g", "Two-CID mapping + per-chunk-AEAD"),
    ("h", "SubgraphSpec walker"),
    ("i", "Revocation reach"),
    ("j", "Resolver evaluation"),
];

/// §4-F ARM 1 — All 10 sub-clauses (a-j) of §1.A.FROZEN item 15 are
/// present in the frozen-interface document. Missing any → CI red.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9 FREEZE wave (when docs/V1-FROZEN-INTERFACE.md is committed). §3.6e."]
fn all_10_item_15_subclauses_a_through_j_present_in_frozen_interface_doc() {
    let frozen_doc = workspace_root().join("docs/V1-FROZEN-INTERFACE.md");
    let body = std::fs::read_to_string(&frozen_doc).unwrap_or_else(|e| {
        panic!(
            "§4-F: docs/V1-FROZEN-INTERFACE.md MUST exist at the \
             G-CORE-9 FREEZE wave (the §1.A.FROZEN terminal \
             deliverable per plan C13). Missing at {}: {e}",
            frozen_doc.display()
        )
    });

    // Each sub-clause MUST appear in the body. Tolerant matcher:
    // either by `(a)` letter-marker, or by named surface, or both.
    let mut missing: Vec<&str> = Vec::new();
    for (letter, surface) in ITEM_15_SUBCLAUSES {
        let letter_marker = format!("({letter})");
        let has_letter = body.contains(&letter_marker);
        let has_surface = body.contains(surface);
        if !has_letter && !has_surface {
            missing.push(*surface);
        }
    }
    assert!(
        missing.is_empty(),
        "§4-F: ALL 10 sub-clauses (a-j) of §1.A.FROZEN item 15 MUST be \
         enumerated in the frozen-interface document. Missing: {missing:?}. \
         The load-bearing escape-valve property is: a frozen-surface \
         mutation fails CI STRUCTURALLY (the cargo-public-api gate + \
         this enumeration pin), not just by policy."
    );
}

/// §4-F ARM 2 — The cargo-public-api baseline INCLUDES the new
/// public types: SubgraphSpec, RestrictedSpec, Scope,
/// AuthorizationGrant, EncryptionClass, DropBundle, walk_share_scope.
/// Sibling structural to
/// `tf12_benten_engine_freeze_conformance_cargo_public_api.rs` but
/// at a more specific level (the §4-F type-set inventory).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9 (when the baselines are regenerated + committed)."]
fn cargo_public_api_baseline_includes_item_15_named_public_types() {
    // The cargo-public-api workflow's per-crate baseline files live
    // under docs/public-api/<crate>/<crate>.txt (per the workflow at
    // .github/workflows/cargo-public-api.yml). At freeze time those
    // baselines must enumerate the new S&C public types.
    let baselines_root = workspace_root().join("docs/public-api");
    if !baselines_root.exists() {
        panic!(
            "§4-F + §1.A.FROZEN item 9: docs/public-api/ baseline \
             directory MUST exist post-FREEZE (the cargo-public-api \
             baseline regenerated + committed at G-CORE-9). Missing \
             at {}.",
            baselines_root.display()
        );
    }

    // The named public types from §1.A.FROZEN item 15 + item 6
    // signature/encryption seam types. Each MUST appear in at least
    // one baseline file (the implementer crate; benten-core for
    // SubgraphSpec/walker; benten-caps for Scope/RestrictedSpec/
    // AuthorizationGrant; benten-crypto-suite for EncryptionClass;
    // benten-drop for DropBundle).
    let required_types = [
        "SubgraphSpec",
        "RestrictedSpec",
        "Scope",
        "AuthorizationGrant",
        "EncryptionClass",
        "DropBundle",
        "walk_share_scope",
    ];

    // Walk all baseline files; concatenate; the set must intersect each.
    let mut all_baselines = String::new();
    for entry in std::fs::read_dir(&baselines_root).expect("read public-api dir") {
        let entry = entry.expect("read entry");
        let path = entry.path();
        if path.is_dir() {
            for inner in std::fs::read_dir(&path).expect("read crate dir") {
                let inner = inner.expect("read inner");
                if let Ok(s) = std::fs::read_to_string(inner.path()) {
                    all_baselines.push_str(&s);
                    all_baselines.push('\n');
                }
            }
        }
    }

    let mut missing: Vec<&str> = Vec::new();
    for t in required_types {
        if !all_baselines.contains(t) {
            missing.push(t);
        }
    }
    assert!(
        missing.is_empty(),
        "§4-F: cargo-public-api baselines MUST enumerate the new \
         §1.A.FROZEN item 15 + item 6 public types. Missing from any \
         baseline: {missing:?}. The post-freeze public-API drift gate \
         is the structural backstop on the §1.A.FROZEN escape-valve."
    );
}

/// §4-F ARM 3 — `#[non_exhaustive]` is applied to the new enums
/// per §1.A.FROZEN item 11 ONE-coherent-sweep. The Scope enum
/// (item 15 (c)) and EncryptionClass (item 15 (e)) MUST carry
/// `#[non_exhaustive]` so post-v1 variant additions are NOT a
/// breaking change. Sibling to
/// `tf12_benten_engine_freeze_conformance_non_exhaustive_907.rs`.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9 (when Scope + EncryptionClass land + the #[non_exhaustive] sweep applies)."]
fn item_15_enums_carry_non_exhaustive_attribute_per_item_11_sweep() {
    // Body intent at un-ignore: scan the source of the future Scope
    // enum (benten-caps) + EncryptionClass enum (benten-crypto-suite),
    // assert each `pub enum` declaration carries `#[non_exhaustive]`
    // on the preceding line (or in the attribute list). This is the
    // structural-source-scan pattern from
    // tf12_benten_engine_freeze_conformance_non_exhaustive_907.rs.
    //
    //   let scope_src = std::fs::read_to_string(
    //       workspace_root().join("crates/benten-caps/src/scope.rs")
    //   ).expect("Scope source post-G-CORE-3b");
    //   assert!(
    //       scope_src.contains("#[non_exhaustive]")
    //         && scope_src.contains("pub enum Scope"),
    //       "§4-F + item 11: Scope MUST carry #[non_exhaustive] \
    //        (post-v1 variant additions stay non-breaking)"
    //   );
    //   let enc_src = std::fs::read_to_string(
    //       workspace_root().join("crates/benten-crypto-suite/src/cipher_suite.rs")
    //   ).expect("EncryptionClass source post-G-CORE-3a");
    //   assert!(
    //       enc_src.contains("#[non_exhaustive]")
    //         && enc_src.contains("pub enum EncryptionClass"),
    //       "§4-F + item 11: EncryptionClass MUST carry #[non_exhaustive]"
    //   );
    //
    panic!(
        "RED-PHASE: §4-F + §1.A.FROZEN item 11 — Scope + \
         EncryptionClass MUST carry #[non_exhaustive] per the \
         ONE-coherent-sweep. Un-ignore at G-CORE-9 (when these types \
         land); scan source for the attribute. Would-FAIL if a \
         future agent ships Scope/EncryptionClass without \
         #[non_exhaustive] (forecloses post-v1 additivity)."
    );
}

/// §4-F ARM 4 — §1.A.FROZEN item 15 sub-clause (i) doc-coupling:
/// `docs/SECURITY-POSTURE.md` carries an explicit "Revocation reach"
/// section documenting the asymmetry (UCAN revocation cuts future
/// serves; already-derived keys remain decryptable; Drop bundles
/// forever-valid). The pim-2 sub-rule-4 doc-coupling pin: a wave
/// claiming to close (i) MUST land the doc section, not just code.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3 doc-retense wave + G-CORE-9 freeze."]
fn security_posture_carries_revocation_reach_section_item_15_i_doc_coupling() {
    let sp = workspace_root().join("docs/SECURITY-POSTURE.md");
    let body = std::fs::read_to_string(&sp).unwrap_or_else(|e| {
        panic!(
            "§4-F item 15 (i): docs/SECURITY-POSTURE.md MUST exist at \
             {}: {e}",
            sp.display()
        )
    });
    // The pim-2 sub-rule-4 doc-coupling check: a specific named
    // section, not just a passing mention.
    assert!(
        body.contains("Revocation reach")
            || body.contains("revocation reach")
            || body.contains("Revocation-reach"),
        "§4-F + §1.A.FROZEN item 15 (i): docs/SECURITY-POSTURE.md \
         MUST carry a 'Revocation reach' section documenting: UCAN \
         revocation cuts future serves; already-derived keys remain \
         decryptable; Drop bundles forever-valid once distributed; \
         mitigation = tight nbf/exp + key rotation. Would-FAIL on \
         silent omission of this doc surface (pim-2 sub-rule-4 \
         doc-coupling failure)."
    );
}
