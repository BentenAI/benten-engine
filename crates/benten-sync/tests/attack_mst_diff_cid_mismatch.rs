//! R4-R2-FP/B RED-PHASE pin for sec-r4r2-1 / sec-r4r1-5 attack-vector
//! cluster (R4 R1 security-auditor MAJOR; carry through R4-R1 + R4-FP
//! merge cycle).
//!
//! ## Pin source
//!
//! - sec-r4r1-5 MAJOR pin (b): `mst_diff_entry_with_cid_byte_mismatch_rejected_at_application_layer`.
//! - sec-r4r2-1 MAJOR (carry; r4-r2-security.json:25-32).
//! - cross-corroborates with distributed-systems-reviewer lens
//!   (sync-trust-boundary attack-vector cluster) per
//!   r4-r2-security.json:96 process_notes.
//!
//! ## What this defends against
//!
//! An adversarial peer crafts an MST-diff frame whose entries declare
//! one CID but whose payload bytes hash to a **different** CID. Naive
//! MST-diff application (trust-by-declaration) would commit the
//! adversarial bytes under the declared CID — and Phase-1's
//! content-addressing invariant (CIDs are computed from bytes, not
//! declared) would silently break: a peer could store a Node under a
//! CID that is not the actual hash of its bytes.
//!
//! Defense: at the **application layer** (the step that ingests the
//! diff entries into local storage / Loro doc / IVM router), every
//! entry's payload bytes MUST be re-hashed locally and compared
//! byte-for-byte against the entry's declared CID. On mismatch:
//! reject with `E_SYNC_MST_ENTRY_CID_BYTE_MISMATCH` (typed variant
//! distinct from row-4b's `E_SYNC_DIVERGENT_CID_REJECTED` — that
//! defense is for divergent-but-internally-consistent CIDs across
//! peers; this defense is for entries whose declared CID does not
//! match its own bytes).
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-C wave-6b lands MST
//! diff application-layer CID-byte verification"`. Body documents the
//! production wiring against `MstDiff::apply_entries` /
//! `engine.consume_sync_replica_mst_diff` per sec-r4r1-5 enumeration.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! - drives the production receive path (`Engine::consume_sync_replica_mst_diff`
//!   → `MstDiff::apply_entries` → byte-rehash check);
//! - asserts an OBSERVABLE behavioral consequence (typed-error variant
//!   `EngineError::MstEntryCidByteMismatch` + write-NOT-applied at
//!   receiving peer);
//! - would FAIL if the application-layer rehash check were silently
//!   no-op'd (i.e., if entries were trusted by declaration).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-C wave-6b — sec-r4r2-1/sec-r4r1-5 — MST-diff entry with CID-byte mismatch rejected at application layer"]
fn mst_diff_entry_with_cid_byte_mismatch_rejected_at_application_layer() {
    // sec-r4r2-1 attack-vector pin (MST-diff CID-byte mismatch).
    //
    // G16-C implementer wires this against the production receive
    // path:
    //
    //   use benten_sync::mst::{Mst, MstDiff, MstEntry};
    //   use benten_sync::handshake::{Handshake, Session};
    //   use benten_engine::Engine;
    //   use benten_engine::errors::EngineError;
    //   use benten_core::cid::Cid;
    //   use benten_core::canonical::canonical_bytes;
    //
    //   // Two peers under sync-replica trust handshake cleanly:
    //   let mut engine_legitimate = test_engine_with_peer_did(peer_legitimate);
    //   let mut engine_attacker = test_engine_with_peer_did(peer_attacker);
    //   let session = run_clean_handshake(&engine_legitimate, &engine_attacker);
    //
    //   // Attacker crafts an MST-diff frame whose entries declare CID
    //   // X but whose payload bytes hash to CID Y (X != Y).
    //   let real_post_bytes = canonical_bytes(&make_post("user-content"));
    //   let real_cid = Cid::from_bytes(&real_post_bytes); // = X
    //
    //   let attacker_substitute_bytes = canonical_bytes(&make_post("attacker-substitute"));
    //   // hash(attacker_substitute_bytes) = Y; Y != X
    //
    //   let adversarial_entry = MstEntry::new_with_explicit_cid_for_testing(
    //       /* declared_cid = */ real_cid,            // X (legitimate)
    //       /* payload_bytes = */ attacker_substitute_bytes, // hashes to Y (substituted)
    //   );
    //   let adversarial_diff = MstDiff::new()
    //       .add_entry(adversarial_entry)
    //       .build();
    //   let frame = session.encrypt_mst_diff_frame(adversarial_diff).unwrap();
    //
    //   // Receiving peer's consume path:
    //   let result = engine_legitimate.consume_sync_replica_mst_diff(frame);
    //
    //   // Application-layer rehash check fires:
    //   match result {
    //       Err(EngineError::MstEntryCidByteMismatch {
    //           declared_cid, computed_cid, attacker_peer_did, ..
    //       }) => {
    //           assert_eq!(declared_cid, real_cid);  // X
    //           assert_ne!(computed_cid, real_cid);  // Y != X
    //           assert_eq!(attacker_peer_did, peer_attacker.did());
    //       }
    //       Err(other) => panic!(
    //           "expected MstEntryCidByteMismatch; got {other:?} — \
    //            if this is E_SYNC_DIVERGENT_CID_REJECTED, the test \
    //            FAILS because that defense is for divergent CIDs \
    //            across peers, NOT for entries whose declared CID \
    //            doesn't match its own bytes — the application-layer \
    //            rehash check was silently no-op'd"),
    //       Ok(_) => panic!("attack succeeded — MST entry with CID-byte mismatch \
    //                        was committed to local storage; content-addressing \
    //                        invariant is broken"),
    //   }
    //
    //   // OBSERVABLE consequence #1: write-NOT-applied at receiving peer.
    //   // The legitimate post (under CID X) is NOT in storage — the
    //   // adversarial bytes (Y) were rejected without commit.
    //   assert!(engine_legitimate.read_node_by_cid(real_cid).is_err(),
    //       "attacker-substitute bytes were committed under legitimate CID — \
    //        content-addressing broken");
    //
    //   // OBSERVABLE consequence #2: rejection observable in
    //   // engine.atrium_status().last_rejected_frame:
    //   let rejection_record = engine_legitimate.atrium_status()
    //       .last_rejected_frame_for_classifier("MstEntryCidByteMismatch")
    //       .unwrap();
    //   assert_eq!(rejection_record.attacker_peer_did, peer_attacker.did());
    //   assert_eq!(rejection_record.declared_cid, real_cid);
    //
    //   // OBSERVABLE consequence #3 (defends against silent no-op):
    //   // application-layer rehash counter increments per entry.
    //   assert_eq!(engine_legitimate.metrics()
    //       .mst_diff_entry_rehash_check_calls(), 1,
    //       "MST-diff application-layer rehash check was never invoked — \
    //        entries are being trusted by declaration; \
    //        content-addressing invariant unenforced at sync boundary");
    //
    // OBSERVABLE consequence: every MST-diff entry's payload bytes
    // are re-hashed locally and compared byte-for-byte against the
    // declared CID at application time; mismatches reject loudly.
    // Defends against the failure shape where an attacker substitutes
    // bytes under a legitimate CID.
    unimplemented!(
        "G16-C wires MstDiff::apply_entries application-layer rehash check + \
         EngineError::MstEntryCidByteMismatch typed variant"
    );
}
