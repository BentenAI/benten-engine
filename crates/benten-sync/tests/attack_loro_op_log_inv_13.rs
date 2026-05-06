//! R4-R2-FP/B RED-PHASE pin for sec-r4r2-1 / sec-r4r1-5 (R4 R1
//! security-auditor MAJOR; carry through R4-R1 + R4-FP merge cycle).
//!
//! ## Pin source
//!
//! - sec-r4r1-5 MAJOR (3 concrete sync-trust-boundary adversarial
//!   attack vectors named with symbol-form pins; FIX-NOW disposition_hint
//!   independent of sec-r4r1-4 matrix-timing decision).
//! - sec-r4r2-1 MAJOR (carry; r4-r2-security.json:25-32).
//! - cross-corroborates with distributed-systems-reviewer lens
//!   (sync-trust-boundary attack-vector cluster) per
//!   r4-r2-security.json:96 process_notes.
//!
//! ## What this defends against
//!
//! An adversarial peer crafts a Loro CRDT op-log whose **byte-level
//! merge semantics** legitimately converge — Loro's CRDT property
//! ensures the bytes themselves can be applied without corruption —
//! but whose **target Node lives in a Phase-1 Inv-13 immutability
//! domain** (system-zone Anchor, governance rule, capability-delegation
//! Node, or zone-definition Node).
//!
//! The naive defense is row-4b CID-divergence rejection at the
//! sync-replica layer. That defense fires **after** the Loro merge
//! produces a divergent root CID. The adversarial path here is
//! subtler: the attacker constructs an op-log that mutates an
//! Anchor-immutable Node's **internal property** without changing the
//! root MST CID at the system-zone layer — the merge bytes **converge
//! locally** but the underlying Node violates Inv-13's immutability
//! guarantee.
//!
//! Defense: the **dispatch layer** (Inv-13 row-4b enforcement during
//! op-log application, BEFORE Loro merge produces canonical bytes)
//! must walk every Loro op against the immutability classifier and
//! reject system-zone / Anchor-immutable mutations with
//! `E_INV_13_VIOLATION_VIA_LORO_OP_LOG`. The CID-divergence check at
//! row-4b is a SECOND line of defense; the dispatch-layer rejection
//! is the FIRST.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-B wave-6b lands
//! Inv-13 dispatch-layer Loro op-log rejection"`. Body documents the
//! production wiring against `engine.consume_sync_replica_frame` /
//! `LoroDoc::apply_op_log` per sec-r4r1-5 enumeration.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! - drives the production receive path (`Engine::consume_sync_replica_frame`
//!   → `Inv13Dispatch::classify_loro_op` → `LoroDoc::apply_op_log`);
//! - asserts an OBSERVABLE behavioral consequence (typed-error variant
//!   `EngineError::Inv13ViolationViaLoroOpLog` + write-NOT-applied at
//!   receiving peer);
//! - would FAIL if the dispatch-layer immutability classifier were
//!   silently no-op'd (i.e., if rejection only fired at row-4b
//!   CID-divergence after merge).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — sec-r4r2-1/sec-r4r1-5 — Loro op-log Inv-13 violation rejected at dispatch, not just at CID divergence"]
fn loro_merge_op_log_violating_inv_13_immutability_rejected_at_dispatch_not_just_at_cid_divergence()
{
    // sec-r4r2-1 attack-vector pin (Loro op-log Inv-13 violation).
    //
    // G16-B + G16-D implementer wires this against the production
    // receive path:
    //
    //   use benten_sync::crdt::LoroDoc;
    //   use benten_sync::handshake::{Handshake, Session};
    //   use benten_engine::Engine;
    //   use benten_engine::errors::EngineError;
    //
    //   // Two peers under sync-replica trust handshake cleanly:
    //   let mut engine_legitimate = test_engine_with_peer_did(peer_legitimate);
    //   let mut engine_attacker = test_engine_with_peer_did(peer_attacker);
    //   let session = run_clean_handshake(&engine_legitimate, &engine_attacker);
    //
    //   // System-zone governance rule (Anchor-immutable per Inv-13):
    //   let governance_anchor_cid = engine_legitimate
    //       .write_node_in_zone("/system/zone/governance",
    //           make_governance_rule("rule-1", policy_v1()))
    //       .unwrap();
    //
    //   // Attacker crafts Loro op-log targeting the governance-rule
    //   // Anchor's internal property. The op-log's BYTES are merge-valid
    //   // (Loro's CRDT property holds locally), but the target Node is
    //   // in an Inv-13 immutability domain — the dispatch layer must
    //   // reject regardless of byte-merge validity.
    //   let adversarial_op_log = engine_attacker
    //       .craft_loro_op_log_for_anchor(governance_anchor_cid,
    //           "/system/zone/governance/rule-1.policy",
    //           policy_attacker_substitute())
    //       .unwrap();
    //
    //   // Adversarial frame is signed under the attacker's session
    //   // (so signature-tampering defense at handshake.rs is bypassed
    //   // — this attack vector targets the dispatch-layer defense
    //   // SPECIFICALLY, not the signature surface):
    //   let frame = session.encrypt_loro_op_log_frame(adversarial_op_log).unwrap();
    //
    //   // Receiving peer's consume path:
    //   let result = engine_legitimate.consume_sync_replica_frame(frame);
    //
    //   // FIRST line of defense — Inv-13 dispatch-layer classifier
    //   // rejects BEFORE Loro merge produces canonical bytes:
    //   match result {
    //       Err(EngineError::Inv13ViolationViaLoroOpLog {
    //           anchor_cid, zone, attacker_peer_did, ..
    //       }) => {
    //           assert_eq!(anchor_cid, governance_anchor_cid);
    //           assert_eq!(zone, "/system/zone/governance");
    //           assert_eq!(attacker_peer_did, peer_attacker.did());
    //       }
    //       Err(other) => panic!(
    //           "expected Inv13ViolationViaLoroOpLog (FIRST-LINE dispatch defense); \
    //            got {other:?} — if this is E_SYNC_DIVERGENT_CID_REJECTED, \
    //            the test FAILS because that is the SECOND-LINE row-4b \
    //            defense firing AFTER Loro merge — the FIRST-LINE \
    //            dispatch classifier was silently no-op'd"),
    //       Ok(_) => panic!("attack succeeded — system-zone Anchor was mutated via Loro op-log; \
    //                        Inv-13 dispatch-layer defense is missing"),
    //   }
    //
    //   // OBSERVABLE consequence #1: write-NOT-applied at receiving peer.
    //   let post_attack_rule = engine_legitimate
    //       .read_current_for_anchor_in_zone("/system/zone/governance", "rule-1")
    //       .unwrap();
    //   assert_eq!(post_attack_rule.policy(), policy_v1(),
    //       "attack altered system-zone Anchor via Loro op-log — Inv-13 dispatch layer failed");
    //
    //   // OBSERVABLE consequence #2: rejection observable in
    //   // engine.atrium_status().last_rejected_frame:
    //   let rejection_record = engine_legitimate.atrium_status()
    //       .last_rejected_frame_for_classifier("Inv13ViolationViaLoroOpLog")
    //       .unwrap();
    //   assert_eq!(rejection_record.attacker_peer_did, peer_attacker.did());
    //   assert_eq!(rejection_record.target_anchor_cid, governance_anchor_cid);
    //
    //   // OBSERVABLE consequence #3 (defends against silent no-op):
    //   // dispatch-layer classifier-call counter increments.
    //   assert!(engine_legitimate.metrics()
    //       .inv_13_dispatch_classifier_calls() > 0,
    //       "Inv-13 dispatch-layer classifier was never invoked — \
    //        the dispatch defense is silently no-op'd; row-4b CID-divergence \
    //        is firing as a fallback, but the FIRST-LINE defense is missing");
    //
    // OBSERVABLE consequence: the FIRST-LINE Inv-13 dispatch-layer
    // classifier rejects adversarial Loro op-logs targeting
    // Anchor-immutable Nodes BEFORE Loro merge produces bytes; the
    // SECOND-LINE row-4b CID-divergence check is a fallback, not the
    // primary defense. Defends against the failure shape where an
    // attacker crafts a byte-merge-valid op-log that violates Inv-13's
    // structural immutability invariant.
    unimplemented!(
        "G16-B/G16-D wire Inv-13 dispatch-layer Loro op-log classifier + \
         EngineError::Inv13ViolationViaLoroOpLog typed variant"
    );
}
