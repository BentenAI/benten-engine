//! **F-LB-1 — `K(N)` structural-KDF chain on a REAL `K_principal` (CE-F1).**
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-6 F-LB-1 ("extend
//!     `tf3a_structural_kdf_step_root_derivation.rs` with a real-`K_principal`
//!     arm; 5-Node parity; info-tag-elision negative; runs on real
//!     `K_principal` not stub").
//!   - R0.5 §3.2 Layer-B: `K(root) = HKDF-SHA256(K_principal, "root"||root_cid)`;
//!     `K(N) = HKDF-SHA256(K(predecessor), "step"||edge_label||N.cid)`
//!     (Cryptree-aligned structural-KDF chain). "Residual after Layer-A (~50
//!     LOC): once `K_principal` is a real type, the existing combiner plugs into
//!     it."
//!   - **R0.5 §4.2 FREEZE-table row "Per-member K(N) walk-scope" + Inv-20
//!     clause-h** ("per-member `K(N)` walk-scope"; the 8th of the 12 a–l
//!     MembershipSet clauses, R0.5 §10 Inv-20). This is the Cryptree
//!     selective-share confinement property — a member who holds `K(X)` for
//!     one walk-scope MUST NOT be able to derive `K(Y)` for a sibling `Y`
//!     outside that scope. **R2 §2.1 mis-maps clause-h to F-AUDIT-3**, which
//!     only pins `AuditAccessGradation` UCAN read-scope and never touches
//!     `K(N)`; F-LB-1 (this file) is clause-h's correct behavioral home (the
//!     R2 §2.1 + §"Inv-20 12-clause coverage" rows are corrected this round to
//!     `clause-h → F-LB-1`).
//!   - In-tree LIVE surface `benten_crypto_suite::structural_kdf`
//!     (`derive_root`/`derive_step`/`StructuralKdfKey`) — the existing G-CORE-3a
//!     canary module (already on `main`).
//!   - CLAUDE.md baked-in #5 (HKDF wraps the vetted upstream `hkdf` crate).
//!
//! # What this pins (FN/FG)
//!
//! F-LB-1 is the bridge pin: the structural-KDF chain (LIVE) must derive from a
//! REAL `K_principal` (the Layer-A vault output), not a test stub, AND preserve
//! the load-bearing properties end-to-end on that real key:
//!   1. 5-Node walk parity (same canonical path → byte-identical key sequence);
//!   2. path-divergence (different predecessor → different key at the same Node);
//!   3. the `"step"` info-tag is load-bearing (eliding it changes the key) — the
//!      multitenant-r1.4-2 / crypto-agility-r1.4-2 root-cause negative control;
//!   4. the chain is seeded from the REAL `K_principal` handle (a different
//!      `K_principal` yields a different root key — the keying-root binding);
//!   5. **sibling-confinement (Inv-20 clause-h)** — a member who is given ONLY
//!      `K(X)` for one walk-scope CANNOT compute `K(Y)` for a non-descendant
//!      sibling `Y`, because `derive_step` is a one-way HKDF over the
//!      predecessor (the Cryptree selective-share confinement guarantee).
//!
//! # Hybrid live/stub strategy (wave-independence)
//!
//! `structural_kdf::{derive_root, derive_step, StructuralKdfKey}` is ALREADY
//! LIVE on `main` (the G-CORE-3a canary), so this file USES the real derivation
//! API directly — that part is not stubbed. What is NOT live is the **real
//! `K_principal` source** (the Layer-A vault `UnlockedKeyMaterial.k_principal`,
//! which R0.5 §3.1 names a STUB). Per wave-independence (NO dependency on W1's
//! own vault file `f_va_*` or on W0's modules), this file commits a LOCAL
//! `f_lb_1_k_principal_stub` modelling the vault-derived `K_principal` source.
//! The R5 closing wave:
//!   1. DELETEs the `f_lb_1_k_principal_stub` module,
//!   2. INSERTs `use benten_crypto_suite::vault::UnlockedKeyMaterial;` and seeds
//!      the chain from the REAL `k_principal`,
//!   3. UN-IGNOREs each test,
//!   4. Verifies green (the structural-KDF properties already hold on `main`;
//!      R5 only re-roots the seed).
//!
//! # Would-FAIL-if-no-op'd (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! The derivation pins drive the LIVE `derive_root`/`derive_step` production
//! call sites over a fixed 5-Node walk + assert byte-level consequences. The
//! info-tag-elision negative uses a LOCAL no-`"step"` foil. The
//! real-`K_principal` binding pin would FAIL on a chain that ignores the seed.
//! The clause-h sibling-confinement pin would FAIL on any derivation that let a
//! held step-key re-derive a non-descendant sibling's key (an over-derivable
//! chain).

#![allow(clippy::unwrap_used)]
#![allow(dead_code)]

use benten_crypto_suite::structural_kdf::{StructuralKdfKey, derive_root, derive_step};

/// SELF-CONTAINED stub for the vault-derived `K_principal` SOURCE only (the
/// derivation API itself is LIVE and used directly above). R5 deletes this.
mod f_lb_1_k_principal_stub {
    use benten_crypto_suite::structural_kdf::StructuralKdfKey;
    use hkdf::Hkdf;
    use sha2::Sha256;

    /// SELF-CONTAINED no-`"step"`-info-tag foil for the elision negative control.
    ///
    /// The production `derive_step` builds HKDF info as `"step" || edge_label ||
    /// node_cid`. This foil omits the `"step"` prefix (info = `edge_label ||
    /// node_cid`) — the multitenant-r1.4-2 / crypto-agility-r1.4-2 root-cause
    /// shape. Implemented locally (wraps the vetted upstream `hkdf` crate, no
    /// Benten reimplementation per CLAUDE.md #5) so the negative control is
    /// parallel-safe + does NOT depend on the `testing`-feature-gated
    /// `StructuralKdfKey::derive_step_without_info_tag_for_test`. R5 may swap to
    /// that feature-gated helper once the wave runs under `--features testing`.
    pub fn derive_step_without_step_info_tag(
        predecessor: &StructuralKdfKey,
        edge_label: &[u8],
        node_cid: &[u8],
    ) -> [u8; 32] {
        let mut info = Vec::with_capacity(edge_label.len() + node_cid.len());
        // NOTE: NO `"step"` prefix — the elided cross-role domain separator.
        info.extend_from_slice(edge_label);
        info.extend_from_slice(node_cid);
        let hk = Hkdf::<Sha256>::new(None, &predecessor.as_bytes());
        let mut okm = [0u8; 32];
        hk.expand(&info, &mut okm)
            .expect("HKDF-SHA256 expand to 32 B is infallible");
        okm
    }
}

// R5: wired to the LIVE vault `UnlockedKeyMaterial` as the real `K_principal`
// SOURCE (the structural-KDF derivation API was already LIVE on the base). The
// no-`"step"`-info-tag foil remains a local negative control.
use benten_crypto_suite::vault::UnlockedKeyMaterial as UnlockedKeyMaterialStub;
use f_lb_1_k_principal_stub::derive_step_without_step_info_tag;

/// The cipher-suite codepoint the structural-KDF root binds (X-Wing default).
const CODEPOINT_HYBRID: u16 = 0x647a;

fn fixed_cid(byte: u8) -> [u8; 32] {
    [byte; 32]
}

/// F-LB-1 (a) — 5-Node walk parity ON A REAL `K_principal` source.
///
/// Owner + recipient, each given the same vault-derived `K_principal`, walk the
/// same canonical 5-Node path and arrive at byte-identical keys at each step.
/// would-FAIL-if-no-op'd: a non-deterministic derivation, or one that ignores
/// the seed, breaks parity.
#[test]
fn structural_kdf_5_node_walk_parity_on_real_k_principal() {
    let vault = UnlockedKeyMaterialStub::from_vault_bytes_for_test([0x6Au8; 32]);
    let k_principal = vault.structural_kdf_root_key();

    let root_cid = fixed_cid(0xE0);
    let k_root_owner = derive_root(&k_principal, &root_cid, CODEPOINT_HYBRID);
    let k_root_recip = derive_root(
        &vault.structural_kdf_root_key(),
        &root_cid,
        CODEPOINT_HYBRID,
    );
    assert_eq!(
        k_root_owner.as_bytes(),
        k_root_recip.as_bytes(),
        "derive_root seeded from the same REAL K_principal MUST be \
         deterministic (owner == recipient); would-FAIL on a randomized seed."
    );

    let edges: [&[u8]; 4] = [
        b"edge:VERSION_OF",
        b"edge:CURRENT",
        b"edge:ITEM_TYPE",
        b"edge:VALUE",
    ];
    let cids = [
        fixed_cid(0xE1),
        fixed_cid(0xE2),
        fixed_cid(0xE3),
        fixed_cid(0xE4),
    ];

    let mut k_owner = k_root_owner.clone();
    let mut k_recip = k_root_recip.clone();
    for (edge, cid) in edges.iter().zip(cids.iter()) {
        k_owner = derive_step(&k_owner, edge, cid);
        k_recip = derive_step(&k_recip, edge, cid);
        assert_eq!(
            k_owner.as_bytes(),
            k_recip.as_bytes(),
            "each canonical BFS step MUST be deterministic on the real \
             K_principal chain (Cryptree recipient-derivable property)."
        );
    }
}

/// F-LB-1 (b) — path-divergence on the real chain: different predecessor →
/// different key at the SAME Node CID (selective-share feature; Spike-E
/// Interpretation-B).
///
/// would-FAIL-if-no-op'd: a structure-independent formula yields the same key
/// regardless of predecessor (Spike-E proved that does NOT converge).
#[test]
fn structural_kdf_path_divergence_on_real_k_principal() {
    let vault = UnlockedKeyMaterialStub::from_vault_bytes_for_test([0x7Bu8; 32]);
    let k_root = derive_root(
        &vault.structural_kdf_root_key(),
        &fixed_cid(0xF0),
        CODEPOINT_HYBRID,
    );

    let target_cid = fixed_cid(0xFC);
    let edge_label = b"edge:ITEM_TYPE";

    let k_a = derive_step(
        &derive_step(&k_root, b"edge:VERSION_OF", &fixed_cid(0xF1)),
        edge_label,
        &target_cid,
    );
    let k_b = derive_step(
        &derive_step(&k_root, b"edge:CURRENT", &fixed_cid(0xF2)),
        edge_label,
        &target_cid,
    );
    assert_ne!(
        k_a.as_bytes(),
        k_b.as_bytes(),
        "two paths reaching the SAME Node CID via the SAME edge_label MUST \
         yield DIFFERENT keys (path-tagged selective-share property; would-FAIL \
         on the structure-independent formula Spike-E disproved)."
    );
}

/// F-LB-1 (c) — the `"step"` HKDF info-tag is load-bearing on the real chain.
///
/// Negative control via the LOCAL `derive_step_without_step_info_tag` foil.
/// would-FAIL-if-no-op'd: a `derive_step` that omits the `"step"` prefix matches
/// the foil (the crypto-agility-r1.4-2 root-cause).
#[test]
fn structural_kdf_step_info_tag_load_bearing_on_real_k_principal() {
    let vault = UnlockedKeyMaterialStub::from_vault_bytes_for_test([0x2Du8; 32]);
    let k_root = derive_root(
        &vault.structural_kdf_root_key(),
        &fixed_cid(0x10),
        CODEPOINT_HYBRID,
    );
    let edge_label = b"edge:ITEM_TYPE";
    let target = fixed_cid(0x11);

    let with_step = derive_step(&k_root, edge_label, &target);
    let without_step = derive_step_without_step_info_tag(&k_root, edge_label, &target);
    assert_ne!(
        with_step.as_bytes(),
        without_step,
        "the `step` HKDF info-tag MUST be load-bearing — eliding it MUST change \
         the derived key (crypto-agility-r1.4-2 corrective). would-FAIL on an \
         `edge_label || N.cid`-only HKDF info."
    );
}

/// F-LB-1 (d) — the chain is BOUND to the real `K_principal` seed: a different
/// vault `K_principal` yields a different root key.
///
/// This is the bridge property the family exists for — the structural-KDF root
/// is genuinely keyed off the Layer-A vault output, not a fixed constant.
/// would-FAIL-if-no-op'd: a derivation that ignores the seed yields the same
/// root for both vaults.
#[test]
fn structural_kdf_root_bound_to_real_k_principal_seed() {
    let vault_a = UnlockedKeyMaterialStub::from_vault_bytes_for_test([0x01u8; 32]);
    let vault_b = UnlockedKeyMaterialStub::from_vault_bytes_for_test([0x02u8; 32]);
    let root_cid = fixed_cid(0x20);

    let k_root_a = derive_root(
        &vault_a.structural_kdf_root_key(),
        &root_cid,
        CODEPOINT_HYBRID,
    );
    let k_root_b = derive_root(
        &vault_b.structural_kdf_root_key(),
        &root_cid,
        CODEPOINT_HYBRID,
    );
    assert_ne!(
        k_root_a.as_bytes(),
        k_root_b.as_bytes(),
        "two distinct vault `K_principal` seeds MUST yield distinct root keys \
         for the SAME root_cid — the structural-KDF chain is genuinely keyed \
         off the Layer-A vault output (the keying-root binding; O-2). would-FAIL \
         on a seed-independent derivation."
    );
}

/// F-LB-1 (e) — **Inv-20 clause-h: per-member `K(N)` walk-scope confinement
/// (Cryptree sibling-confinement).**
///
/// This is the C-MAJOR-2 closure: clause-h ("per-member `K(N)` walk-scope",
/// R0.5 §4.2 FREEZE-table + §10 Inv-20) had ZERO behavioral pin in the corpus
/// and was mis-mapped by R2 §2.1 to F-AUDIT-3 (which only covers audit
/// read-gradation). F-LB-1 is its correct home.
///
/// # The confinement property
///
/// A member granted ONLY the step-key `K(X)` for one walk-scope (subtree X)
/// receives a `StructuralKdfKey` opaque handle — they hold derived key MATERIAL,
/// not `K_principal` and not `K(root)`. Because `derive_step` is a one-way
/// HKDF-expand over the predecessor key, from `K(X)` a member can ONLY walk
/// FORWARD into X's own descendants. They CANNOT:
///   - reach a non-descendant sibling `Y` (a different subtree off the same
///     root), because reaching `Y` requires `K(root)`→…→`K(Y)`, and the
///     root/sibling-prefix predecessors are NOT recoverable from `K(X)` (HKDF
///     pre-image resistance);
///   - reconstruct `K(root)` from any `K(N)` they hold.
///
/// We pin this STRUCTURALLY (the held material is insufficient) rather than
/// attempting to break HKDF: the test models the member as holding ONLY `K(X)`
/// (subtree-X step-key) and asserts that the sibling key `K(Y)` — which the
/// owner computes from `K(root)` along a DISJOINT walk — is unequal to anything
/// the X-holder can derive by walking forward from `K(X)` with `Y`'s own
/// edge/cid. The forward-walk-from-X result is path-tagged by X's predecessor
/// chain, so it can never collide with the owner's root-anchored `K(Y)`.
///
/// would-FAIL-if-no-op'd: a structure-INDEPENDENT derivation (one that ignored
/// the predecessor — exactly the formula Spike-E disproved) would let the
/// X-holder land on `K(Y)` by supplying `Y`'s edge_label+cid, collapsing the
/// confinement. The `assert_ne!` fires only on a one-way path-tagged chain.
#[test]
fn structural_kdf_clause_h_sibling_walk_scope_confinement_on_real_k_principal() {
    let vault = UnlockedKeyMaterialStub::from_vault_bytes_for_test([0x9Cu8; 32]);

    // The OWNER holds K_principal and derives the shared root.
    let k_root = derive_root(
        &vault.structural_kdf_root_key(),
        &fixed_cid(0xA0),
        CODEPOINT_HYBRID,
    );

    // Two DISJOINT sibling subtrees hang off the root via different edges/cids:
    //   X  reached by edge:ITEM_TYPE  → cid 0xA1
    //   Y  reached by edge:VERSION_OF → cid 0xA2  (a NON-descendant of X)
    let k_x = derive_step(&k_root, b"edge:ITEM_TYPE", &fixed_cid(0xA1));
    let k_y = derive_step(&k_root, b"edge:VERSION_OF", &fixed_cid(0xA2));

    // Sanity: the two sibling scope-keys are themselves distinct (different
    // predecessors-of-root edges; the selective-share precondition).
    assert_ne!(
        k_x.as_bytes(),
        k_y.as_bytes(),
        "sibling subtrees X and Y MUST have distinct scope-keys (selective-share \
         precondition for clause-h confinement)."
    );

    // The member is granted ONLY K(X). Everything they can compute is a FORWARD
    // walk from K(X). The most adversarial attempt: replay Y's OWN edge_label +
    // cid against the held K(X), trying to land on Y's key.
    let x_holder_attempt_at_y =
        derive_step(&k_x, b"edge:VERSION_OF", &fixed_cid(0xA2));

    // CONFINEMENT: the X-holder's forward walk (predecessor = K(X)) can NEVER
    // equal the owner's root-anchored K(Y) (predecessor = K(root)). The chain is
    // path-tagged — Y's key is bound to its root predecessor, unreachable from X.
    assert_ne!(
        x_holder_attempt_at_y.as_bytes(),
        k_y.as_bytes(),
        "Inv-20 clause-h VIOLATED: a member holding ONLY K(X) reconstructed the \
         non-descendant sibling key K(Y). The structural-KDF chain is \
         over-derivable — derive_step ignored its predecessor (the \
         structure-INDEPENDENT formula Spike-E disproved). would-FAIL on a \
         confined one-way path-tagged chain."
    );

    // And the held K(X) is not itself K(root): the X-holder cannot present their
    // grant as the principal/root scope.
    assert_ne!(
        k_x.as_bytes(),
        k_root.as_bytes(),
        "a per-member scope-key K(X) MUST NOT equal K(root) — the member holds \
         a confined sub-scope, not the principal root (clause-h walk-scope)."
    );
}
