//! GAP-KDB Shape-B — CS-1 / **FS-2** classical-from-bytes `sig::PublicKey`
//! constructor (`pq = None`). W1 RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-R2-LANDSCAPE.md` §1 benten-crypto-suite row
//! (`CS-1 — FS-2 classical-from-bytes constructor (sig::PublicKey pq=None)
//! so ONE helper handles both issuer shapes`) + §3 dependency graph
//! ("W3 … gate only on W0 resolve_signing + **W1 FS-2**"). Design R1
//! Fork-A correction **C6** (resolve_signing must peek the leading
//! multicodec — `0xed01`→classical, `0x1211`→composite — so ONE helper
//! returns the right `sig::PublicKey` shape for both issuer kinds) +
//! §3.1/identity-resolve FS-2.
//!
//! # What FS-2 is (and why it belongs to the crypto-suite W1 slice)
//!
//! The Fork-A authority path (W3) needs ONE codepoint-dispatched
//! `resolve_signing` helper that returns a `sig::PublicKey` for BOTH
//! issuer shapes: a classical `did:key` (`0xed01 ‖ ed25519(32)`) → a
//! `pq = None` handle, and a hybrid / `did:benten` (`0x1211 ‖ mldsa …`)
//! → a `pq = Some` composite handle. Today `sig::PublicKey` has only
//! `from_lamps_composite_bytes` (which ALWAYS sets `pq = Some`); there is
//! NO classical-only-from-bytes constructor. FS-2 mints it so the ONE
//! helper does not need a second, Ed25519-only code path (which would be
//! the silent-PQ-strip surface AUTH-3/AUTH-7 pin against). This is the
//! crypto-suite primitive W3's AUTH-* pins consume — hence its home in W1.
//!
//! (Distinct from the `kem`-field wiring facet of CS-1 pinned in
//! `kdb_cs1_kem_recipient_wiring.rs` / `kdb_cs1_kem_cp_component_crosscheck.rs`;
//! both facets are the single benten-crypto-suite CS-1 family.)
//!
//! # SELF-CONTAINED + stub-shim
//!
//! `benten-id` depends on `benten-crypto-suite` → no cross-crate fixture
//! import. The classical input bytes come from a REAL v1-default hybrid
//! keypair's composite (its trailing Ed25519 half) so the point is valid;
//! sizes flow from `sizes::ml_dsa_65_pubkey_len()` — never hardcoded (#5).
//! `cs1_fs2_shim::classical_public_from_ed25519_bytes` is a local
//! `todo!()` stub → real `sig::PublicKey` classical constructor at R5;
//! then drop `#[ignore]`.
//!
//! # would_fail_on_revert
//!
//! An FS-2 impl that fabricates a PQ half (`pq = Some`) flips
//! `is_hybrid()` from `false` to `true` and makes `to_lamps_composite_bytes()`
//! succeed instead of erroring → both asserts flip. That fabrication is a
//! silent-PQ-UPGRADE on the authority path (the dual of the AUTH-3
//! silent-PQ-strip): a classical issuer would be mis-typed as hybrid.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_variables)]

use benten_crypto_suite::VerifyError;
use benten_crypto_suite::sig::{PublicKey, SignatureSuite};

/// The trailing Ed25519 (32-byte) half of a REAL v1-default hybrid public
/// key's LAMPS composite `mldsaPK(1952) || ed25519(32)`, plus the full
/// composite. The Ed25519 half is a valid curve point; the composite feeds
/// the hybrid-contrast arm.
fn real_ed25519_half_and_composite() -> ([u8; 32], Vec<u8>) {
    let hybrid_pub: PublicKey = SignatureSuite::v1_default().generate_keypair().public();
    let composite = hybrid_pub
        .to_lamps_composite_bytes()
        .expect("v1-default keypair carries a PQ half → composite serializes");
    let mldsa_len = benten_crypto_suite::sizes::ml_dsa_65_pubkey_len();
    let ed25519_half: [u8; 32] = composite[mldsa_len..]
        .try_into()
        .expect("LAMPS composite tail is the 32-byte Ed25519 half");
    (ed25519_half, composite)
}

/// The FS-2 logic-under-test (RED-PHASE stub → real classical constructor
/// at R5).
mod cs1_fs2_shim {
    use super::{PublicKey, VerifyError};

    /// FS-2 — build a CLASSICAL-only `sig::PublicKey` (`pq = None`) from a
    /// raw 32-byte Ed25519 verifying key (design R2 CS-1 / identity-resolve
    /// FS-2). The `pq = None` shape is load-bearing: it lets ONE
    /// `resolve_signing` helper serve both the classical `did:key` arm and
    /// the composite `did:benten` arm without a second Ed25519-only verify
    /// path.
    ///
    /// STUB `todo!()` at R3. R5: delegate to the minted real entry
    /// (e.g. `sig::PublicKey::from_classical_ed25519_bytes`).
    ///
    /// # Errors
    /// [`VerifyError::MalformedKey`] if the 32 bytes are not a valid
    /// Ed25519 curve point — fail-closed (never a silent default).
    pub fn classical_public_from_ed25519_bytes(
        _ed25519: &[u8; 32],
    ) -> Result<PublicKey, VerifyError> {
        todo!(
            "RED-PHASE (CS-1/FS-2): classical-from-bytes sig::PublicKey \
             (pq=None) constructor lands at R5 (GAP-KDB-B W1). un-ignore then."
        )
    }
}

// ── FS-2 — classical constructor yields a pq=None handle ──────────────────

/// A `sig::PublicKey` built from a raw Ed25519 verifying key via FS-2 is
/// CLASSICAL-only: `is_hybrid()` is `false` and it carries no LAMPS
/// composite to serialize.
#[test]
#[ignore = "RED-PHASE: CS-1/FS-2 classical-from-bytes sig::PublicKey is pq=None — un-ignore at R5"]
fn cs1_fs2_classical_constructor_is_not_hybrid() {
    let (ed25519_half, _composite) = real_ed25519_half_and_composite();

    let classical: PublicKey = cs1_fs2_shim::classical_public_from_ed25519_bytes(&ed25519_half)
        .expect("FS-2 MUST accept a valid Ed25519 verifying key");

    assert!(
        !classical.is_hybrid(),
        "FS-2 classical-from-bytes MUST yield pq=None (is_hybrid()==false); \
         would-FAIL if the constructor fabricates a PQ half — a silent \
         PQ-UPGRADE mis-typing a classical issuer as hybrid"
    );
    assert!(
        classical.to_lamps_composite_bytes().is_err(),
        "a classical-only (pq=None) handle has NO LAMPS composite to \
         serialize → to_lamps_composite_bytes() MUST fail closed; \
         would-FAIL if FS-2 wrongly attached a PQ half"
    );
}

// ── FS-2 completeness — the SAME shape distinction on the hybrid arm ───────

/// The contrast half of "ONE helper handles both issuer shapes": the
/// composite constructor (`from_lamps_composite_bytes`, already real) MUST
/// yield `is_hybrid()==true` with a serializable composite — so FS-2's
/// classical arm and the composite arm are distinguishable by exactly the
/// `pq` shape the ONE `resolve_signing` helper dispatches on.
#[test]
#[ignore = "RED-PHASE: CS-1/FS-2 composite arm stays pq=Some (both-shapes distinction) — un-ignore at R5"]
fn cs1_fs2_composite_arm_is_hybrid_distinct_from_classical() {
    let (ed25519_half, composite) = real_ed25519_half_and_composite();

    let hybrid = PublicKey::from_lamps_composite_bytes(&composite)
        .expect("a valid LAMPS composite MUST reconstruct a hybrid handle");
    let classical = cs1_fs2_shim::classical_public_from_ed25519_bytes(&ed25519_half)
        .expect("FS-2 MUST accept the valid Ed25519 half");

    assert!(
        hybrid.is_hybrid(),
        "the composite arm MUST be pq=Some (is_hybrid()==true)"
    );
    assert!(
        !classical.is_hybrid(),
        "the FS-2 classical arm MUST be pq=None (is_hybrid()==false)"
    );
    assert!(
        hybrid.is_hybrid() != classical.is_hybrid(),
        "the ONE resolve_signing helper distinguishes the two issuer shapes \
         by the pq shape alone; would-FAIL if FS-2 collapsed the classical \
         arm into a hybrid handle"
    );
}
