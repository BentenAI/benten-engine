//! R4-R2-FP/B RED-PHASE pin for sec-r4r2-1 / sec-r4r1-5 attack-vector
//! cluster (R4 R1 security-auditor MAJOR; carry through R4-R1 + R4-FP
//! merge cycle).
//!
//! ## Pin source
//!
//! - sec-r4r1-5 MAJOR pin (c): `hlc_skew_exceeded_in_inbound_sync_frame_rejected_with_e_hlc_skew_exceeded`.
//! - sec-r4r2-1 MAJOR (carry; r4-r2-security.json:25-32).
//! - cross-corroborates with distributed-systems-reviewer lens
//!   (sync-trust-boundary attack-vector cluster) per
//!   r4-r2-security.json:96 process_notes.
//! - composes with `benten-core::HlcSkewExceeded` (existing in
//!   `crates/benten-core/tests/hlc_clock_skew_exceeded_fires_e_hlc_skew_exceeded.rs`)
//!   — that test pins single-clock skew detection; THIS test pins
//!   the sync-frame inbound-skew defense + revocation-vs-data
//!   ordering protection.
//!
//! ## What this defends against
//!
//! HLC ordering is load-bearing for two Phase-3 trust-boundary
//! decisions:
//!
//! 1. **LWW resolution** in user-data zones (Inv-13 row-4a) —
//!    higher-HLC writes win.
//! 2. **Revocation-vs-data ordering** — a UCAN revocation issued at
//!    HLC=T MUST be applied before any data write at HLC<T from the
//!    revoked party (per the Phase-3 D-PHASE-3-N revocation
//!    propagation contract).
//!
//! An adversarial peer can manipulate its local HLC to inject sync
//! frames with **future-HLC values** (e.g., now + 24 hours), thereby:
//!
//! - **Biasing LWW resolution**: any honest write the attacker wants
//!   to overwrite can be retroactively-defeated by an
//!   attacker-injected future-HLC write under the same property.
//! - **Forging revocation-vs-data ordering**: the attacker writes
//!   data at HLC=T+24h, then an honest peer revokes the attacker's
//!   cap at HLC=T+1m; the data write WINS the LWW race even though
//!   the revocation came chronologically later, because the
//!   adversarial HLC was 24 hours in the future. Revocation
//!   propagation is silently defeated.
//!
//! Defense: at the **inbound sync-frame entry point**, the receiving
//! peer caps inbound HLC drift relative to its local HLC (per the
//! existing `Hlc::tolerance_window_secs` policy). Inbound frames
//! with HLC values exceeding the tolerance window MUST reject with
//! `EngineError::HlcSkewExceededInInboundSyncFrame` (typed variant,
//! mapping to stable code `E_HLC_SKEW_EXCEEDED`).
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G14-pre-D + G16-D wire
//! inbound-sync-frame HLC skew-cap defense"`. Body documents the
//! production wiring against `engine.consume_sync_replica_frame` per
//! sec-r4r1-5 enumeration.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! - drives the production receive path (`Engine::consume_sync_replica_frame`
//!   → `HlcSkewClassifier::check_inbound_frame` → frame-acceptance);
//! - asserts an OBSERVABLE behavioral consequence (typed-error variant
//!   `EngineError::HlcSkewExceededInInboundSyncFrame` mapping to
//!   stable code `E_HLC_SKEW_EXCEEDED` + LWW-not-defeated end-to-end);
//! - would FAIL if the inbound skew-cap check were silently no-op'd
//!   (i.e., if inbound HLC values were trusted unconditionally).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-pre-D + G16-D wave-6b — sec-r4r2-1/sec-r4r1-5 — HLC skew exceeded in inbound sync frame rejected with E_HLC_SKEW_EXCEEDED"]
fn hlc_skew_exceeded_in_inbound_sync_frame_rejected_with_e_hlc_skew_exceeded() {
    // sec-r4r2-1 attack-vector pin (HLC skew injection biasing
    // LWW + revocation-vs-data ordering).
    //
    // G14-pre-D + G16-D implementer wires this against the production
    // receive path:
    //
    //   use benten_sync::handshake::Session;
    //   use benten_core::hlc::Hlc;
    //   use benten_engine::Engine;
    //   use benten_engine::errors::EngineError;
    //   use benten_errors::ErrorCode;
    //
    //   // Two peers under sync-replica trust handshake cleanly:
    //   let mut engine_legitimate = test_engine_with_peer_did(peer_legitimate);
    //   let mut engine_attacker = test_engine_with_peer_did(peer_attacker);
    //   let session = run_clean_handshake(&engine_legitimate, &engine_attacker);
    //
    //   // Receiving peer's local HLC at "now":
    //   let now_hlc = engine_legitimate.local_hlc().now();
    //
    //   // Attacker manipulates its local HLC to inject a future-HLC
    //   // write — 24 hours in the future:
    //   let future_24h_hlc = Hlc::synthetic_at_offset_secs(now_hlc, 24 * 3600);
    //
    //   // Attacker writes a data update at the future HLC:
    //   let adversarial_write = engine_attacker
    //       .craft_loro_op_log_at_hlc(
    //           "/zone/posts/p1.title",
    //           "attacker-substitute-title",
    //           future_24h_hlc,
    //       )
    //       .unwrap();
    //   let frame = session.encrypt_loro_op_log_frame(adversarial_write).unwrap();
    //
    //   // Receiving peer's consume path:
    //   let result = engine_legitimate.consume_sync_replica_frame(frame);
    //
    //   // Inbound-skew classifier rejects:
    //   let err = match result {
    //       Err(EngineError::HlcSkewExceededInInboundSyncFrame {
    //           inbound_hlc, local_hlc, tolerance_secs, attacker_peer_did, ..
    //       }) => {
    //           assert_eq!(inbound_hlc, future_24h_hlc);
    //           assert!(local_hlc <= now_hlc.add_tolerance(tolerance_secs));
    //           assert!(future_24h_hlc.skew_secs_relative_to(local_hlc) > tolerance_secs as i64);
    //           assert_eq!(attacker_peer_did, peer_attacker.did());
    //           EngineError::HlcSkewExceededInInboundSyncFrame { /* ... */ }
    //       }
    //       Err(other) => panic!(
    //           "expected HlcSkewExceededInInboundSyncFrame; got {other:?} — \
    //            inbound HLC skew-cap defense at sync-frame entry point \
    //            was silently no-op'd"),
    //       Ok(_) => panic!("attack succeeded — future-HLC write was applied; \
    //                        LWW resolution + revocation-vs-data ordering \
    //                        are open to HLC-skew injection"),
    //   };
    //
    //   // OBSERVABLE consequence #1: typed error maps to stable
    //   // catalog code `E_HLC_SKEW_EXCEEDED` (composes with
    //   // benten-core HlcSkewExceeded code-path):
    //   assert_eq!(err.code(), ErrorCode::HlcSkewExceeded);
    //   assert_eq!(err.code().as_str(), "E_HLC_SKEW_EXCEEDED");
    //
    //   // OBSERVABLE consequence #2: LWW resolution NOT defeated.
    //   // Honest write at honest HLC remains the CURRENT.
    //   engine_legitimate
    //       .write_node_in_zone("/zone/posts", make_post_with_title("p1", "honest-title"))
    //       .unwrap();
    //   let p1 = engine_legitimate
    //       .read_current_for_anchor_in_zone("/zone/posts", "p1")
    //       .unwrap();
    //   assert_eq!(p1.title(), "honest-title",
    //       "LWW resolution was defeated by HLC-skew injection — \
    //        attacker's future-HLC write won the merge race despite \
    //        chronologically-later honest write");
    //
    //   // OBSERVABLE consequence #3: revocation-vs-data ordering
    //   // protection. Issue an honest revocation of attacker's cap at
    //   // local HLC; verify the attacker's pre-revocation data writes
    //   // (which would have been applied if the skew check no-op'd)
    //   // are NOT in the merged state.
    //   let revocation_hlc = engine_legitimate.local_hlc().now();
    //   engine_legitimate.revoke_cap_for_peer(peer_attacker.did(), "/zone/posts:write").unwrap();
    //   assert!(revocation_hlc < future_24h_hlc,
    //       "test setup invariant: revocation HLC < attacker future HLC");
    //   // Attacker's adversarial write was REJECTED at frame-ingress;
    //   // had skew-check no-op'd, the future-HLC write would have won
    //   // LWW over the revocation-mediated post-revoke writes.
    //   let p1_post_revoke = engine_legitimate
    //       .read_current_for_anchor_in_zone("/zone/posts", "p1")
    //       .unwrap();
    //   assert_eq!(p1_post_revoke.title(), "honest-title",
    //       "revocation-vs-data ordering was forged by HLC-skew injection");
    //
    //   // OBSERVABLE consequence #4 (defends against silent no-op):
    //   // inbound-skew classifier counter increments per inbound frame.
    //   assert!(engine_legitimate.metrics()
    //       .hlc_inbound_skew_classifier_calls() > 0,
    //       "inbound-skew classifier was never invoked — \
    //        sync frames are being applied without HLC skew check");
    //
    // OBSERVABLE consequence: inbound sync frames with HLC values
    // exceeding the local skew tolerance reject with
    // `EngineError::HlcSkewExceededInInboundSyncFrame` mapping to
    // `E_HLC_SKEW_EXCEEDED`. Defends against the LWW-bias and
    // revocation-vs-data-ordering forgery failure shapes.
    unimplemented!(
        "G14-pre-D + G16-D wire inbound-sync-frame HLC skew classifier + \
         EngineError::HlcSkewExceededInInboundSyncFrame typed variant \
         (composes with benten-core HlcSkewExceeded → ErrorCode::HlcSkewExceeded)"
    );
}
