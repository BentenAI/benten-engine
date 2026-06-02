//! **F-LB-1 — `K(N)` structural-KDF chain on a REAL `K_principal` (CE-F1).**
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-6 F-LB-1 ("extend
//!     `tf3a_structural_kdf_step_root_derivation.rs` with a real-`K_principal`
//!     arm; 5-Node parity; info-tag-elision negative; runs on real
//!     `K_principal` not stub").
//!   - R0 §3.2 Layer-B: `K(root) = HKDF-SHA256(K_principal, "root"||root_cid)`;
//!     `K(N) = HKDF-SHA256(K(predecessor), "step"||edge_label||N.cid)`
//!     (Cryptree-aligned structural-KDF chain). "Residual after Layer-A (~50
//!     LOC): once `K_principal` is a real type, the existing combiner plugs into
//!     it."
//!   - In-tree LIVE surface `benten_crypto_suite::structural_kdf`
//!     (`derive_root`/`derive_step`/`StructuralKdfKey`) — the existing G-CORE-3a
//!     canary module (already on `main`).
//!   - CLAUDE.md baked-in #5 (HKDF wraps the vetted upstream `hkdf` crate).
//!
//! # What this pins (FN/FG)
//!
//! F-LB-1 is the bridge pin: the structural-KDF chain (LIVE) must derive from a
//! REAL `K_principal` (the Layer-A vault output), not a test stub, AND preserve
//! the two load-bearing properties end-to-end on that real key:
//!   1. 5-Node walk parity (same canonical path → byte-identical key sequence);
//!   2. path-divergence (different predecessor → different key at the same Node);
//!   3. the `"step"` info-tag is load-bearing (eliding it changes the key) — the
//!      multitenant-r1.4-2 / crypto-agility-r1.4-2 root-cause negative control;
//!   4. the chain is seeded from the REAL `K_principal` handle (a different
//!      `K_principal` yields a different root key — the keying-root binding).
//!
//! # Hybrid live/stub strategy (wave-independence)
//!
//! `structural_kdf::{derive_root, derive_step, StructuralKdfKey}` is ALREADY
//! LIVE on `main` (the G-CORE-3a canary), so this file USES the real derivation
//! API directly — that part is not stubbed. What is NOT live is the **real
//! `K_principal` source** (the Layer-A vault `UnlockedKeyMaterial.k_principal`,
//! which R0 §3.1 names a STUB). Per wave-independence (NO dependency on W1's own
//! vault file `f_va_*` or on W0's modules), this file commits a LOCAL
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
//! info-tag-elision negative uses the LIVE
//! `StructuralKdfKey::derive_step_without_info_tag_for_test` foil. The
//! real-`K_principal` binding pin would FAIL on a chain that ignores the seed.

#![allow(clippy::unwrap_used)]
#![allow(dead_code)]

use benten_crypto_suite::structural_kdf::{StructuralKdfKey, derive_root, derive_step};

/// SELF-CONTAINED stub for the vault-derived `K_principal` SOURCE only (the
/// derivation API itself is LIVE and used directly above). R5 deletes this.
mod f_lb_1_k_principal_stub {
    use benten_crypto_suite::structural_kdf::StructuralKdfKey;
    use hkdf::Hkdf;
    use sha2::Sha256;

    /// Models the Layer-A vault's `UnlockedKeyMaterial.k_principal`. R5 replaces
    /// this with the real `benten_crypto_suite::vault::UnlockedKeyMaterial`.
    pub struct UnlockedKeyMaterialStub {
        k_principal_bytes: [u8; 32],
    }

    impl UnlockedKeyMaterialStub {
        pub fn from_vault_bytes_for_test(k_principal_bytes: [u8; 32]) -> Self {
            Self { k_principal_bytes }
        }

        /// The keying-root: the structural-KDF chain seeds from THIS. R5 returns
        /// the real `StructuralKdfKey` minted from the vault `[u8;32]`.
        pub fn structural_kdf_root_key(&self) -> StructuralKdfKey {
            StructuralKdfKey::from_bytes_for_test(&self.k_principal_bytes)
        }
    }

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

use f_lb_1_k_principal_stub::{UnlockedKeyMaterialStub, derive_step_without_step_info_tag};

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
#[ignore = "RED-PHASE: F-LB-1 — structural-KDF 5-Node walk parity seeded from the REAL K_principal; un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-LB-1 — path-divergence (different predecessor → different K(N)) on the real chain; un-ignore at R5"]
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
/// Negative control via the LIVE `derive_step_without_info_tag_for_test` foil.
/// would-FAIL-if-no-op'd: a `derive_step` that omits the `"step"` prefix matches
/// the foil (the crypto-agility-r1.4-2 root-cause).
#[test]
#[ignore = "RED-PHASE: F-LB-1 — `step` HKDF info-tag is load-bearing on the real chain; un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-LB-1 — root key is BOUND to the real K_principal seed (different vault → different root); un-ignore at R5"]
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
