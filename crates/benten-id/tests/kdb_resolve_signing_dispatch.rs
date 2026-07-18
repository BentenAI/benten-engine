//! GAP-KDB Shape-B — `resolve_signing` method/multicodec-aware dispatch
//! (families DID-4, RS-1, RS-2). W0-canary RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §2 Tier-1 + C6 (resolve_signing
//! is a NEW fn that PEEKS the leading multicodec — 0xed01→classical,
//! 0x1211→composite — rather than dispatching on the method string alone,
//! and EXPECTS the trailing 36-B CID on a did:benten). `R2-LANDSCAPE`
//! DID-4/RS-1/RS-2.
//!
//! # would_fail_on_revert
//! - DID-4: a resolver that dispatches on method-string alone mis-handles a
//!   hybrid did:key (method `did:key`, multicodec 0x1211) → recovers the
//!   wrong key / errors → the equality arm flips.
//! - RS-2: a resolve_signing that does NOT strip the trailing keyset-CID (or
//!   that fetches a doc) fails to recover a did:benten's signing key
//!   zero-I/O → the equality arm flips.
//!
//! # R5 un-ignore
//! Mint `Did::resolve_signing` (composite arm + trailing-CID strip, C6);
//! repoint the stub; drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_id::did::Did;
use benten_id::kdb_testing as kdb;
use benten_id::keypair::Keypair;

// ── RS-1 — classical arm (0xed01) round-trip ──────────────────────────────

#[test]
fn rs1_resolve_signing_classical_didkey_arm() {
    let kp = Keypair::generate();
    let did = kp.public_key().to_did();
    let recovered =
        kdb::resolve_signing(&did).expect("resolve_signing recovers a classical did:key");
    // A classical did:key yields a classical-only signing key (pq=None) via
    // the CS-1 classical-from-bytes constructor (W1). The exact 32-byte
    // identity round-trip is pinned by the live DID-5 regression
    // (`did5_didkey_classical_encoding_unchanged_regression`).
    assert!(
        !recovered.is_hybrid(),
        "resolve_signing (classical 0xed01 arm) MUST recover a classical-only key (pq=None)"
    );
}

// ── RS-2 — did:benten composite arm + trailing-CID strip, ZERO-I/O ────────

#[test]
fn rs2_resolve_signing_did_benten_composite_arm_strips_trailing_cid() {
    // The killer-neutralizer: recover a did:benten's composite signing key
    // WITHOUT any key-set doc (zero-I/O) by stripping the trailing 36-B CID
    // component before the composite decode (design §2 Tier-1).
    let (did, _honest, _attacker) = kdb::substituted_recipient_scenario();
    // Rebuild the same DID from a known keypair so we can compare.
    let kp = kdb::hybrid_keypair();
    let sig_mk = kdb::signing_multikey_of(&kp.public());
    let kem_mk = kdb::kem_multikey_hybrid(
        &kdb::det_x25519_pub("rs2/x"),
        &kdb::det_mlkem768_ek("rs2/ek"),
    );
    let doc = kdb::KeySetDocument::v1_hybrid(sig_mk.clone(), kem_mk);
    let bent = kdb::self_committed_did(&doc);

    let recovered = kdb::resolve_signing(&bent)
        .expect("resolve_signing recovers a did:benten signing key with NO doc (zero-I/O)");
    assert_eq!(
        recovered.to_lamps_composite_bytes().unwrap(),
        kp.public().to_lamps_composite_bytes().unwrap(),
        "resolve_signing (did:benten composite arm) MUST recover the exact embedded \
         composite signing key after stripping the trailing keyset-CID — no doc, no I/O"
    );
    // The wider scenario DID resolves too (its signing key is present).
    assert!(
        kdb::resolve_signing(&did).is_ok(),
        "the substitution-scenario did:benten also resolves its signing key zero-I/O"
    );
}

// ── DID-4 — C6 multicodec-peek dispatch matrix ────────────────────────────

#[test]
fn did4_dispatch_peeks_multicodec_not_method_string() {
    let kp = kdb::hybrid_keypair();

    // (a) HYBRID did:key — method is `did:key` but leading multicodec is
    //     0x1211 (composite). resolve_signing MUST use the COMPOSITE arm
    //     (peek the multicodec, not the method string) and agree with the
    //     existing resolve_hybrid.
    let hybrid_didkey = Did::from_hybrid_public_key(&kp.public());
    assert!(
        hybrid_didkey.as_str().starts_with("did:key:z"),
        "precondition: a hybrid did:key still uses the did:key method"
    );
    let via_signing = kdb::resolve_signing(&hybrid_didkey)
        .expect("resolve_signing handles a hybrid did:key via the composite arm");
    let via_hybrid = hybrid_didkey.resolve_hybrid().expect("resolve_hybrid");
    assert_eq!(
        via_signing.to_lamps_composite_bytes().unwrap(),
        via_hybrid.to_lamps_composite_bytes().unwrap(),
        "DID-4: a hybrid did:key MUST dispatch on multicodec 0x1211 (composite), \
         NOT on the `did:key` method string (C6)"
    );

    // (b) did:benten — composite leading multicodec + a trailing CID tail.
    //     resolve_signing MUST recover the same composite key.
    let sig_mk = kdb::signing_multikey_of(&kp.public());
    let doc = kdb::KeySetDocument::v1_hybrid(
        sig_mk,
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub("did4/x"),
            &kdb::det_mlkem768_ek("did4/ek"),
        ),
    );
    let bent = kdb::self_committed_did(&doc);
    let via_benten = kdb::resolve_signing(&bent).expect("resolve_signing handles did:benten");
    assert_eq!(
        via_benten.to_lamps_composite_bytes().unwrap(),
        kp.public().to_lamps_composite_bytes().unwrap(),
        "DID-4: a did:benten MUST recover the same composite signing key (composite arm + CID strip)"
    );

    // (c) classical did:key — leading multicodec 0xed01 → classical arm
    //     (classical-only key; pq=None). Identity byte-pin: live DID-5.
    let classic = Keypair::generate();
    let cdid = classic.public_key().to_did();
    let via_classic =
        kdb::resolve_signing(&cdid).expect("resolve_signing handles classical did:key");
    assert!(
        !via_classic.is_hybrid(),
        "DID-4: a classical did:key MUST dispatch on multicodec 0xed01 (classical arm, pq=None)"
    );
}
