//! GAP-KDB Shape-B — CS-1 crypto-suite kem-field ↔ `RecipientPublic`
//! wiring (X25519-first per C2). W1 RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §5 (0x120c/0xec wiring +
//! `RecipientPublic::from_bytes` reorder-DELETED-by-C2) + ratified
//! correction **C2** ("make the doc `kem` field X25519-first =
//! `0xec‖x25519(32) ‖ 0x120c‖mlkem768_ek(1184)`, matching the
//! already-frozen `RecipientPublic::to_bytes()` order — DELETES the §5
//! REORDER step"). `GAP-KDB-B-R2-LANDSCAPE.md` §3 W1 slice (CS-1
//! crypto-suite kem-wiring) + §4 TIER-C.
//!
//! # Why this file is SELF-CONTAINED (no `benten_id::kdb_testing` import)
//!
//! `benten-id` depends on `benten-crypto-suite`, so a crypto-suite test
//! cannot import the benten-id W0 shared fixtures (that would be a
//! circular dependency). CS-1 is pinned entirely against crypto-suite's
//! OWN frozen surface: `RecipientPublic::{to_bytes,from_bytes}` +
//! `CipherSuiteCodepoint` + `wrap_key_material`/`unwrap_key_material` +
//! the named size constants (`X25519_PUBLIC_LEN`, `ML_KEM_768_EK_LEN`) —
//! never hardcoded (CLAUDE.md baked-in #5).
//!
//! # The frozen shape under pin (design §5 + C2)
//!
//! A [`KeySetDocument`]'s `kem` field is the X25519-first hybrid multikey
//! `[0xec,0x01 ‖ x25519(32) ‖ 0x8c,0x24 ‖ mlkem768_ek(1184)]` (`0x8c,0x24`
//! = unsigned-varint of the registered `mlkem-768-pub = 0x120c`; `0xec,0x01`
//! = `x25519-pub = 0xec`). The CS-1 crypto-suite entry decodes that
//! multikey and wires the two component payloads into a
//! [`RecipientPublic`] at `kem_cp = 0x647a` via the **already-frozen**
//! X25519-first `RecipientPublic::from_bytes` (payload
//! `x25519(32) ‖ mlkem768_ek(1184)`) — so C2 means the multikey payload,
//! after stripping the two component varints, is byte-identical to what
//! `from_bytes` consumes. NO reorder (the §5 REORDER foot-gun is deleted).
//!
//! # RED-PHASE stub-shim discipline (pim-12 §3.6e)
//!
//! The `cs1_shim::recipient_public_from_kem_multikey` entry (parse the
//! multikey + cross-check component codecs against `kem_cp` + build the
//! `RecipientPublic`) does NOT exist at the W0 canary base. It is a local
//! `todo!()` stub so this file COMPILES green while `#[ignore]`d. R5
//! implementer:
//!   1. replace the `cs1_shim` body with a delegation to the minted real
//!      entry (e.g. `RecipientPublic::from_kem_multikey(kem_cp, multikey)`
//!      or the `benten-id` `resolve_kem` component-decode helper it calls),
//!   2. drop `#[ignore]`,
//!   3. verify green.
//! Because `todo!()` panics, no pin can pass against the stub — the only
//! way to green is a real, non-no-op implementation (substance by
//! construction).
//!
//! # would_fail_on_revert
//!
//! `cs1_kem_wiring_round_trips_x25519_first`: a wiring that drops the
//! ML-KEM half, keeps only x25519, or mis-orders the two halves produces
//! `to_bytes() != kp.public().to_bytes()` → the `assert_eq` flips; a no-op
//! fails equally.
//! `cs1_kem_wiring_wrap_unwrap_end_to_end`: a wiring that recovers the
//! wrong bytes yields a `RecipientPublic` the matching secret cannot
//! unwrap → the round-trip `assert_eq` flips (the gold-standard
//! no-op-detector: only a functionally-correct encapsulation key
//! wrap/unwraps against its own secret).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_variables)]

use benten_crypto_suite::CipherSuiteCodepoint;
use benten_crypto_suite::cipher_suite::{
    CipherSuite, ML_KEM_768_EK_LEN, RecipientPublic, X25519_PUBLIC_LEN,
};

/// `x25519-pub = 0xec`, unsigned-varint `[0xec, 0x01]` (design §5 /
/// C2). Frozen wire — un-confusable with the `[0x8c, 0x24]` ML-KEM tag.
const X25519_PUB_MULTICODEC: [u8; 2] = [0xec, 0x01];
/// `mlkem-768-pub = 0x120c`, unsigned-varint `[0x8c, 0x24]` (design §5 —
/// retires the #5-risky private `HYBRID_KEM_MULTICODEC = 0xf0`).
const MLKEM768_PUB_MULTICODEC: [u8; 2] = [0x8c, 0x24];

/// Assemble the frozen X25519-first hybrid `kem` multikey (design C2):
/// `0xec,0x01 ‖ x25519 ‖ 0x8c,0x24 ‖ mlkem768_ek`. Real frozen assembly.
fn kem_multikey_x25519_first(x25519: &[u8], mlkem768_ek: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + x25519.len() + 2 + mlkem768_ek.len());
    out.extend_from_slice(&X25519_PUB_MULTICODEC);
    out.extend_from_slice(x25519);
    out.extend_from_slice(&MLKEM768_PUB_MULTICODEC);
    out.extend_from_slice(mlkem768_ek);
    out
}

/// The CS-1 logic-under-test — delegates to the minted real crypto-suite
/// entry [`RecipientPublic::from_kem_multikey`] (R5 GAP-KDB-B W1).
mod cs1_shim {
    use super::{CipherSuiteCodepoint, RecipientPublic};
    use benten_crypto_suite::AeadError;

    /// CS-1 kem-field → `RecipientPublic` wiring (design §5 + C2). Delegates
    /// to the real [`RecipientPublic::from_kem_multikey`], which:
    /// 1. parses the X25519-first component multikeys
    ///    (`0xec‖x25519(32)`, then `0x120c‖mlkem768_ek(1184)`),
    /// 2. **cross-checks the component multicodec set against `kem_cp`**
    ///    (`0x647a ⟺ {0xec, 0x120c}`), fail-closed on disagreement
    ///    (algorithm-confusion defense, C2),
    /// 3. hands the concatenated `x25519(32) ‖ mlkem768_ek(1184)` payload
    ///    to the already-frozen X25519-first
    ///    [`RecipientPublic::from_bytes`] (NO reorder — the §5 REORDER
    ///    step is deleted by C2).
    ///
    /// # Errors
    /// [`AeadError`] on any malformed multikey / component-vs-`kem_cp`
    /// disagreement / unsupported codepoint — never a silent default.
    pub fn recipient_public_from_kem_multikey(
        kem_cp: u16,
        kem_multikey: &[u8],
    ) -> Result<RecipientPublic, AeadError> {
        RecipientPublic::from_kem_multikey(CipherSuiteCodepoint::from_raw(kem_cp), kem_multikey)
    }
}

// ── CS-1 — X25519-first kem-multikey wiring recovers both halves ──────────

/// Wire the frozen X25519-first `kem` multikey into a `RecipientPublic`
/// and prove BOTH halves are recovered in the frozen order.
///
/// The x25519 + ML-KEM-768 halves come from a REAL generated hybrid
/// recipient keypair (so the bytes are valid keys), get re-framed as the
/// `kem` multikey, decoded back through the CS-1 wiring, and the result's
/// `to_bytes()` (X25519-first `x25519(32) ‖ mlkem768_ek(1184)`) MUST equal
/// the original public's bytes.
#[test]
fn cs1_kem_wiring_round_trips_x25519_first() {
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("0x647a LIVE at G-CORE-3a");
    let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);

    // The frozen X25519-first RecipientPublic layout: x25519(32) ‖ ek(1184).
    let pub_bytes = kp.public().to_bytes();
    assert_eq!(
        pub_bytes.len(),
        X25519_PUBLIC_LEN + ML_KEM_768_EK_LEN,
        "sanity: hybrid RecipientPublic is x25519(32)‖mlkem768_ek(1184)"
    );
    let (x25519, mlkem768_ek) = pub_bytes.split_at(X25519_PUBLIC_LEN);

    // Re-frame as the KeySetDocument `kem` multikey (design C2).
    let kem_multikey = kem_multikey_x25519_first(x25519, mlkem768_ek);

    let decoded = cs1_shim::recipient_public_from_kem_multikey(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768.raw(),
        &kem_multikey,
    )
    .expect("CS-1 MUST wire a well-formed X25519-first hybrid kem multikey");

    assert_eq!(
        decoded.codepoint(),
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        "CS-1 MUST bind the recovered RecipientPublic to kem_cp=0x647a"
    );
    assert_eq!(
        decoded.to_bytes(),
        pub_bytes,
        "CS-1 wiring MUST recover x25519 AND mlkem768_ek in the frozen \
         X25519-first order (C2); would-FAIL if the ML-KEM half is \
         dropped, only x25519 is kept, or the two halves are swapped"
    );
}

/// The gold-standard no-op detector: the wired `RecipientPublic` must
/// actually encapsulate to the matching secret.
///
/// Decode the `kem` multikey → `RecipientPublic`, wrap a `K_root` to it,
/// then unwrap with the ORIGINAL keypair's secret. A wiring that recovers
/// the wrong bytes (mis-ordered, half-dropped, no-op) produces a
/// `RecipientPublic` the secret cannot unwrap → the recovery `assert_eq`
/// flips.
#[test]
fn cs1_kem_wiring_wrap_unwrap_end_to_end() {
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("0x647a LIVE at G-CORE-3a");
    let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);

    let pub_bytes = kp.public().to_bytes();
    let (x25519, mlkem768_ek) = pub_bytes.split_at(X25519_PUBLIC_LEN);
    let kem_multikey = kem_multikey_x25519_first(x25519, mlkem768_ek);

    let wired: RecipientPublic = cs1_shim::recipient_public_from_kem_multikey(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768.raw(),
        &kem_multikey,
    )
    .expect("CS-1 MUST wire the well-formed kem multikey");

    let k_root = [0x5au8; 32];
    let wrapped = suite
        .wrap_key_material(&wired, &k_root)
        .expect("wrap to the CS-1-wired recipient public MUST succeed");
    let recovered = suite
        .unwrap_key_material(kp.secret(), &wrapped)
        .expect("the ORIGINAL secret MUST unwrap a wrap to its own wired public");

    assert_eq!(
        recovered.as_bytes(),
        &k_root,
        "CS-1 wiring MUST yield a RecipientPublic that encapsulates to the \
         matching secret; would-FAIL if the wiring recovered the wrong \
         bytes (no-op / half-dropped / mis-ordered — the X25519 and \
         ML-KEM-768 halves both feed the X-Wing combiner)"
    );
}
