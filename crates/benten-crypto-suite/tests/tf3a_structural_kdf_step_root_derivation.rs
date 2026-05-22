//! TF-3a pins (G-CORE-3a P-1 + P-2) — structural-KDF Spike-E
//! Interpretation-B path-tagged key derivation: `derive_step` + `derive_root`
//! with explicit HKDF info-tag domain separation (`"step"` / `"root"`).
//!
//! ADDL R3-W1 (TDD RED-phase) test-writer — Phase-4-Meta-Core G-CORE-3a
//! CANARY (`benten_crypto_suite::structural_kdf`). Pin sources:
//!   - `R2-test-landscape.md` §2 G-CORE-3a row P-1 + P-2 (Spike-E formula
//!     `K(N) = HKDF-SHA256(K(predecessor), info = "step" || edge_label || N.cid)`
//!     across a 5-Node walk; same path → same key; different predecessor →
//!     different key; would-FAIL if `"step"` info-tag elided — that is the
//!     multitenant-r1.4-2 / crypto-agility-r1.4-2 root-cause).
//!   - `00-implementation-plan.md` R0.8.1 §3 G-CORE-3a wave def
//!     (`structural_kdf` is one of the named source files of the CANARY)
//!     + §1.A.FROZEN item 15(f) (the formula incl. `"step"` info-tag —
//!     R0.8 crypto-agility-r1.4-2 corrective).
//!   - `RATIFIED-sharing-and-confidentiality-2026-05-21.md` refinement (2)
//!     "path-tagged keys with one canonical-owner-path" + Spike-E
//!     Interpretation B (NOT the structure-independent literal formula
//!     which Spike E proved doesn't converge).
//!   - CLAUDE.md baked-in #5 ("Never fork / never reimplement crypto
//!     primitives" — the integration crate is HKDF-glue + codepoint
//!     dispatch; HKDF impl wraps vetted upstream `hkdf` crate).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! At HEAD `c9c11c56`, `benten-crypto-suite` is a G-CORE-2 stub-plus
//! (signature surface LIVE since G-CORE-2-FP-1; the `structural_kdf`
//! module + `derive_step`/`derive_root` API described by Spike-E DO NOT
//! YET EXIST). Per the `faa5475d` precedent (R3 tests that compile-fail
//! on workspace check were removed; "each canary brings its R3 slice in
//! its own PR"), this file commits a **local stub module** matching the
//! intended G-CORE-3a public surface so the file COMPILES green at
//! baseline + `#[ignore]` keeps the runtime gate per pim-12. The
//! G-CORE-3a closing-wave R5 implementer MUST:
//!   1. DELETE the local `pq_to_be_dispatched_to_g_core_3a` stub module,
//!   2. INSERT the real `use benten_crypto_suite::structural_kdf::*;` line,
//!   3. UN-IGNORE each test (`#[ignore = "RED-PHASE..."]` → `#[test]`),
//!   4. Verify all 4 pins PASS green.
//! Reviewer verifies landing-status (un-ignored + green), not just spec-pin
//! presence (pim-12 §3.6e).
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! These pins exercise the **production** key-derivation path. Load-bearing
//! safety property = **path-tagged domain separation**: the same Node
//! reached by different predecessor paths MUST yield different keys
//! (feature for selective-share per Spike E Interpretation B); the `"step"`
//! HKDF info-tag is what provides cross-role domain separation (Spike E
//! corrected the literal DESIGN-doc formula which lacked it and proved
//! structure-independent convergence DOES NOT HOLD). The stub module
//! intentionally returns `unimplemented!()` so the runtime asserts never
//! pass with a stub still-in-place — if R5 forgets to delete the stub
//! the tests panic with `unimplemented`, the OPPOSITE of a silent-green
//! pim-18 SHAPE-trap.
//!
//! # §3.6f SHAPE-not-SUBSTANCE production call-site enumeration
//!
//! Named-destination: `crates/benten-crypto-suite/src/structural_kdf.rs`
//! (the G-CORE-3a CANARY-named NEW module per plan §3 G-CORE-3a Files
//! line: `{lib.rs, structural_kdf.rs, cipher_suite.rs, signature.rs,
//! codepoint.rs, errors.rs}`). The `pub fn derive_step` + `pub fn
//! derive_root` + `pub struct StructuralKdfKey` are the production
//! surface; HKDF impl wraps the upstream `hkdf` crate per the
//! never-fork-primitives discipline (CLAUDE.md #5).
//!
//! # §3.13 per-test-static decomposition
//!
//! These tests are pure (no process-scoped shared state). Each test
//! constructs its own `key_principal` via per-test fixed byte vectors —
//! no shared static, safe under parallel-runner.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// =====================================================================
// RED-PHASE stub-shim — DELETE at G-CORE-3a implementation; replace with:
//     use benten_crypto_suite::structural_kdf::{derive_root, derive_step, StructuralKdfKey};
// =====================================================================
mod pq_to_be_dispatched_to_g_core_3a {
    //! Local stub matching the intended G-CORE-3a `structural_kdf`
    //! public surface. Every method `unimplemented!()`s so any `#[ignore]`
    //! forgotten / stub left-in-place fails LOUD (not silent-green).

    /// G-CORE-3a stub: `pub struct StructuralKdfKey` (intended). Real
    /// G-CORE-3a impl wraps a `[u8; 32]` HKDF-SHA256 PRK behind a
    /// zeroize-on-drop newtype.
    #[derive(Clone)]
    pub struct StructuralKdfKey(#[allow(dead_code)] [u8; 32]);

    impl StructuralKdfKey {
        pub fn from_bytes_for_test(_bytes: &[u8]) -> Self {
            unimplemented!(
                "G-CORE-3a stub — R5 implementer replaces this file's stub module with real `use benten_crypto_suite::structural_kdf::*;`"
            )
        }
        pub fn as_bytes(&self) -> [u8; 32] {
            unimplemented!("G-CORE-3a stub")
        }
        pub fn derive_step_without_info_tag_for_test(
            _predecessor: &StructuralKdfKey,
            _edge_label: &[u8],
            _node_cid: &[u8],
        ) -> Self {
            unimplemented!(
                "G-CORE-3a stub — adversarial-test helper for the `\"step\"` info-tag elision pin"
            )
        }
    }

    /// G-CORE-3a stub: `pub fn derive_root(K_principal, root_cid) -> StructuralKdfKey`
    /// HKDF-SHA256 with `info = "root" || root_cid`.
    pub fn derive_root(_k_principal: &StructuralKdfKey, _root_cid: &[u8]) -> StructuralKdfKey {
        unimplemented!("G-CORE-3a stub — Spike E formula `info = \"root\" || root_cid`")
    }

    /// G-CORE-3a stub: `pub fn derive_step(predecessor, edge_label, node_cid) -> StructuralKdfKey`
    /// HKDF-SHA256 with `info = "step" || edge_label || N.cid`.
    pub fn derive_step(
        _predecessor: &StructuralKdfKey,
        _edge_label: &[u8],
        _node_cid: &[u8],
    ) -> StructuralKdfKey {
        unimplemented!(
            "G-CORE-3a stub — Spike E Interpretation-B formula `info = \"step\" || edge_label || N.cid`"
        )
    }
}

use pq_to_be_dispatched_to_g_core_3a::{StructuralKdfKey, derive_root, derive_step};

/// The canonical Node CID for a synthetic 5-Node walk. We pin a fixed
/// byte string (NOT a real BLAKE3 digest) so the test is hermetic and
/// the R5 implementer can swap to a real CID type without changing the
/// pin semantics.
fn fixed_cid(byte: u8) -> [u8; 32] {
    [byte; 32]
}

/// Pin (P-1) ROUND-TRIP across a 5-Node walk — same producer path yields
/// the same recipient-derivable key sequence.
///
/// Spike-E Interpretation-B: owner walks canonical BFS path producing
/// `K(root), K(N1), K(N2), …, K(N4)`; recipient given `K_principal`
/// walks the SAME edge-labels and arrives at byte-identical keys at
/// each step. would-FAIL if `derive_step` is not deterministic, or if
/// it diverges from `info = "step" || edge_label || N.cid` formula.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (delete local stub module; insert real `use benten_crypto_suite::structural_kdf::*;`)"]
fn tf3a_p1_5_node_walk_same_path_yields_same_key_sequence() {
    let k_principal = StructuralKdfKey::from_bytes_for_test(&[0u8; 32]);
    let root_cid = fixed_cid(0xA0);
    let k_root_owner = derive_root(&k_principal, &root_cid);
    let k_root_recip = derive_root(&k_principal, &root_cid);
    assert_eq!(
        k_root_owner.as_bytes(),
        k_root_recip.as_bytes(),
        "derive_root MUST be deterministic — owner and recipient given the \
         same (K_principal, root_cid) MUST derive byte-identical K(root); \
         would-FAIL if HKDF info-tag elided or randomized"
    );

    let edges: [&[u8]; 4] = [
        b"edge:VERSION_OF",
        b"edge:CURRENT",
        b"edge:ITEM_TYPE",
        b"edge:VALUE",
    ];
    let cids: [[u8; 32]; 4] = [
        fixed_cid(0xA1),
        fixed_cid(0xA2),
        fixed_cid(0xA3),
        fixed_cid(0xA4),
    ];

    let mut k_owner = k_root_owner.clone();
    let mut k_recip = k_root_recip.clone();
    for (edge, cid) in edges.iter().zip(cids.iter()) {
        k_owner = derive_step(&k_owner, edge, cid);
        k_recip = derive_step(&k_recip, edge, cid);
        assert_eq!(
            k_owner.as_bytes(),
            k_recip.as_bytes(),
            "step-key MUST be deterministic at each BFS-canonical step \
             (Spike E recipient-derivable property; would-FAIL on \
             randomized salts or wrong info-tag)"
        );
    }
}

/// Pin (P-1) PATH-DIVERGENCE — different predecessor → different key
/// at the SAME Node CID.
///
/// Spike-E Interpretation-B "path-tagged keys": a Node reachable by
/// multiple paths gets multiple distinct keys (feature for
/// selective-share). The "structure-independent canonical key"
/// (literal-DESIGN-doc formula) would yield the same key regardless of
/// predecessor — Spike E proved this DOES NOT converge. would-FAIL if
/// an implementer reverts to the literal formula.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (Spike-E Interpretation-B path-tagging contract)"]
fn tf3a_p1_path_divergence_different_predecessor_different_key_same_node() {
    let k_principal = StructuralKdfKey::from_bytes_for_test(&[1u8; 32]);
    let root_cid = fixed_cid(0xB0);
    let k_root = derive_root(&k_principal, &root_cid);

    let target_cid = fixed_cid(0xBC);
    let edge_label = b"edge:ITEM_TYPE";

    let k_a_mid = derive_step(&k_root, b"edge:VERSION_OF", &fixed_cid(0xB1));
    let k_a_target = derive_step(&k_a_mid, edge_label, &target_cid);

    let k_b_mid = derive_step(&k_root, b"edge:CURRENT", &fixed_cid(0xB2));
    let k_b_target = derive_step(&k_b_mid, edge_label, &target_cid);

    assert_ne!(
        k_a_target.as_bytes(),
        k_b_target.as_bytes(),
        "Spike-E Interpretation-B contract — two paths reaching the SAME \
         Node CID via the SAME edge_label MUST yield DIFFERENT keys (the \
         path-tagged property; selective-share feature). would-FAIL if an \
         implementer reverts to the structure-independent literal-DESIGN \
         formula which Spike E proved DOES NOT converge."
    );
}

/// Pin (P-1) `"step"` HKDF info-tag domain separation — eliding `"step"`
/// MUST change the derived key.
///
/// R0.8 crypto-agility-r1.4-2 corrective: the `"step"` HKDF info-tag is
/// the cross-role domain separation slot. Negative-control pin —
/// production `derive_step` must produce a specific output that the
/// no-info-tag variant does NOT match.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (R0.8 crypto-agility-r1.4-2 corrective)"]
fn tf3a_p1_step_info_tag_elision_changes_derived_key() {
    let k_principal = StructuralKdfKey::from_bytes_for_test(&[2u8; 32]);
    let root_cid = fixed_cid(0xC0);
    let k_root = derive_root(&k_principal, &root_cid);

    let edge_label = b"edge:ITEM_TYPE";
    let target_cid = fixed_cid(0xC1);
    let with_step = derive_step(&k_root, edge_label, &target_cid);

    let without_step =
        StructuralKdfKey::derive_step_without_info_tag_for_test(&k_root, edge_label, &target_cid);
    assert_ne!(
        with_step.as_bytes(),
        without_step.as_bytes(),
        "the `\"step\"` HKDF info-tag MUST be load-bearing in derive_step — \
         eliding it MUST change the derived key (Spike E + R0.8 \
         crypto-agility-r1.4-2 corrective). would-FAIL if an implementer \
         constructs HKDF info as `edge_label || N.cid` only (missing the \
         `\"step\"` cross-role domain-separation prefix)."
    );
}

/// Pin (P-2) `derive_root` info-tag + root-vs-step domain separation.
///
/// Spike-E formula: `K(root) = HKDF-SHA256(K_principal, info = "root"
/// || root_cid)`. The root key is INDEPENDENT of step keys — feeding
/// the same byte string into `derive_step` with edge_label=""
/// MUST NOT yield the root key (the `"root"` / `"step"` info-tags are
/// load-bearing domain separation; HKDF's textbook role for the info
/// parameter).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (P-2 root-vs-step domain separation)"]
fn tf3a_p2_root_key_independent_of_step_keys_via_info_tag() {
    let k_principal = StructuralKdfKey::from_bytes_for_test(&[3u8; 32]);
    let root_cid = fixed_cid(0xD0);
    let k_root_via_derive_root = derive_root(&k_principal, &root_cid);
    let k_root_via_derive_step_no_edge = derive_step(
        &StructuralKdfKey::from_bytes_for_test(&k_principal.as_bytes()),
        b"",
        &root_cid,
    );

    assert_ne!(
        k_root_via_derive_root.as_bytes(),
        k_root_via_derive_step_no_edge.as_bytes(),
        "derive_root MUST use info = `\"root\" || root_cid` and derive_step \
         MUST use info = `\"step\" || edge_label || N.cid` — they MUST NOT \
         collide on any input (the role-separation property; would-FAIL on \
         a no-info-tag HKDF construction)."
    );
}
