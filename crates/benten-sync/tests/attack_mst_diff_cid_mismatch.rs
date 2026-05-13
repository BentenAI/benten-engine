//! R6-FP Wave-C1 (ds-r6-1 closure) — sec-r4r2-1 attack-vector pin
//! un-ignored against the live production rehash check at
//! [`benten_sync::mst::Mst::apply_entries`].
//!
//! Pre-Wave-C1 this test was RED-PHASE under
//! `#[ignore = "RED-PHASE: G16-C wave-6b lands MST diff
//! application-layer CID-byte verification"]` with an `unimplemented!()`
//! body. The application-layer rehash check has been live in
//! `benten_sync::mst::Mst::apply_entries` since G16-B canary; this fix
//! pass un-ignores the integration-test pin so it drives the same
//! production path the engine receive-boundary will plumb when
//! `consume_sync_replica_mst_diff` lands.
//!
//! ## What this defends against
//!
//! An adversarial peer crafts an MST-diff frame whose entries declare
//! one CID but whose payload bytes hash to a different CID. Naive
//! MST-diff application (trust-by-declaration) would commit the
//! adversarial bytes under the declared CID — and Phase-1's
//! content-addressing invariant (CIDs are computed from bytes, not
//! declared) would silently break.
//!
//! Defense: at the **application layer** ([`Mst::apply_entries`]),
//! every entry's payload bytes are re-hashed locally and compared
//! byte-for-byte against the declared CID. On mismatch: reject with
//! [`benten_sync::mst::MstError::EntryCidByteMismatch`] mapping to
//! [`benten_errors::ErrorCode::SyncHashMismatch`] at the engine
//! boundary.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! - drives the production receive path
//!   ([`Mst::apply_entries`]);
//! - asserts an OBSERVABLE behavioral consequence (typed-error variant
//!   `MstError::EntryCidByteMismatch` + write-NOT-applied);
//! - would FAIL if the application-layer rehash check were silently
//!   no-op'd (i.e., if entries were trusted by declaration).
//!
//! ## R4-FP-4 substance audit cross-reference
//!
//! This file is audited at
//! `.addl/phase-4-foundation/notes-wave-c1-attack-audit.md` §2.3 per
//! sec-3.5-r1-8 + pim-18 §3.6f SHAPE-not-SUBSTANCE pre-flight. Audit
//! verdict: SUBSTANTIVE — production-surface (`Mst::apply_entries`) +
//! 3 observable-consequence assertions (typed variant + CID field
//! values + write-NOT-applied) + companion-positive pin at lines
//! 108-122 confirms asymmetric rehash check. No substance gaps
//! detected at HEAD.

#![allow(clippy::unwrap_used)]

use benten_sync::mst::{Mst, MstCid, MstEntry, MstError};

#[test]
fn mst_diff_entry_with_cid_byte_mismatch_rejected_at_application_layer() {
    // sec-r4r2-1 attack-vector pin (R6-FP Wave-C1 closure).
    //
    // Attacker crafts an MST-diff entry whose declared CID is the
    // hash of legitimate content but whose payload bytes are
    // attacker-substitute content. The application-layer rehash
    // check at `Mst::apply_entries` MUST reject without commit.

    // Legitimate content at CID X.
    let real_payload = b"legitimate-content".to_vec();
    let real_cid = MstCid::from_bytes(&real_payload);

    // Attacker substitutes payload bytes (hash to Y, Y != X) but
    // declares CID X. `MstEntry::new_with_explicit_cid_for_testing`
    // is the audit-trail-named construction surface for adversarial
    // entries — production callers always go through `from_payload`
    // which computes the CID locally from bytes.
    let attacker_payload = b"attacker-substitute".to_vec();
    let adversarial_entry = MstEntry::new_with_explicit_cid_for_testing(
        /* declared = */ real_cid,
        /* payload  = */ attacker_payload.clone(),
    );

    let mut mst = Mst::new();

    // FIRST line of defense — the application-layer rehash check
    // fires BEFORE the entry is inserted into local storage.
    let result = mst.apply_entries(vec![adversarial_entry]);

    let computed_actual = MstCid::from_bytes(&attacker_payload);
    match result {
        Err(MstError::EntryCidByteMismatch { declared, computed }) => {
            assert_eq!(
                declared, real_cid,
                "declared CID should match the legitimate content's CID (X)"
            );
            assert_eq!(
                computed, computed_actual,
                "computed CID should match BLAKE3 of the attacker's substitute bytes"
            );
            assert_ne!(
                declared, computed,
                "the attack succeeded if declared == computed; the rehash check is silently no-op'd"
            );
        }
        Err(other) => panic!(
            "expected EntryCidByteMismatch; got {other:?} — \
             application-layer rehash check was silently no-op'd or fired the wrong typed error"
        ),
        Ok(_) => panic!(
            "attack succeeded — MST entry with CID-byte mismatch was committed to local storage; \
             content-addressing invariant is broken at the sync boundary"
        ),
    }

    // OBSERVABLE consequence: write-NOT-applied. The MST is empty
    // post-rejection — the adversarial entry was rejected without
    // commit, so the legitimate content's CID space is uncontaminated.
    assert!(
        mst.is_empty(),
        "MST should be empty after rejected adversarial entry — content-addressing broken if not"
    );
}

#[test]
fn mst_apply_entries_legitimate_entry_inserts_cleanly() {
    // Companion-positive pin: legitimate entries (declared CID
    // matches BLAKE3 of payload) insert cleanly without firing the
    // rehash-mismatch defense. Pairs with the adversarial pin above
    // so the rehash check is asymmetric — it rejects mismatches but
    // does not over-reject legitimate entries.
    let payload = b"legitimate-content".to_vec();
    let entry = MstEntry::from_payload("/zone/posts/p1", payload);

    let mut mst = Mst::new();
    let applied = mst.apply_entries(vec![entry]).expect("legitimate entry");
    assert_eq!(applied, 1);
    assert_eq!(mst.len(), 1);
}
