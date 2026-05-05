//! R3-C/R4-FP RED-PHASE pin: G14-B → G16-B coordination handoff —
//! Loro merge throttle consumes the rate-limit policy from
//! `benten-caps`.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.2 G14-B + §7 rate-limits row
//!   `rate_limit_policy_consumed_by_g16_b_loro_merge_throttle`.
//! - `crates/benten-caps/tests/rate_limit_policy.rs` lines 27-30
//!   coordination handoff comment naming this file as the destination.
//! - `tcc-r1-2` (R4 large-council Round 1 test-coverage-completeness
//!   lens — caught the 25th-p/c-drift-instance-precursor handoff
//!   miss).
//! - `D-F` (per-actor rate-limit) + `D-PHASE-3-26` (per-peer
//!   bandwidth budget at Atrium boundary).
//!
//! ## Coordination handoff
//!
//! At R3-B landing time, `benten-sync` did not exist. R3-B authored
//! 5 rate-limit pins on the producer (caps) side + documented in the
//! file header that the consumer-side pin
//! `rate_limit_policy_consumed_by_g16_b_loro_merge_throttle` lives at
//! `crates/benten-sync/tests/rate_limit_consumption.rs` (R3-C
//! territory). R3-C landed `benten-sync` but did NOT create this
//! file — caught by R4 large-council R1 test-coverage-completeness
//! lens (`tcc-r1-2`). R4-FP/R3-C lands the missing file.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-B wave-6b — D-F + D-PHASE-3-26 — Loro merge throttle consumes rate-limit policy"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-F + D-PHASE-3-26 + tcc-r1-2 — Loro merge throttle consumes rate_limit_policy from benten-caps"]
fn rate_limit_policy_consumed_by_g16_b_loro_merge_throttle() {
    // tcc-r1-2 + D-F + D-PHASE-3-26 pin. G16-B implementer wires this
    // against the Loro-merge throttle path that consumes the policy
    // landed at G14-B in `benten-caps`.
    //
    //   use benten_caps::rate_limit_policy::{
    //       RateLimitPolicy, PerActorBudget, PerPeerBandwidthBudget,
    //   };
    //   use benten_sync::loro_merge::{LoroMergeThrottle, ThrottleResult};
    //
    //   // Producer (caps) side: construct a policy with low per-peer
    //   // bandwidth budget so the throttle observably engages.
    //   let policy = RateLimitPolicy::builder()
    //       .per_actor_budget(PerActorBudget::default())
    //       .per_peer_bandwidth_budget(PerPeerBandwidthBudget::bytes_per_sec(1024))
    //       .build();
    //
    //   // Consumer (sync) side: construct the throttle pulling from
    //   // policy. The throttle is the production entry-point; it is
    //   // wired through engine→atrium→loro-merge.
    //   let throttle = LoroMergeThrottle::from_policy(&policy, peer_b_did);
    //
    //   // Drive a merge that exceeds the per-peer bandwidth budget:
    //   let large_merge = synthesize_loro_merge_with_byte_size(8 * 1024);
    //   let result = throttle.try_apply(&large_merge);
    //
    //   // The throttle returns `Throttled` (back-pressure surfaced):
    //   match result {
    //       ThrottleResult::Throttled { wait_until_hlc, reason } => {
    //           assert!(wait_until_hlc > current_hlc());
    //           assert!(reason.contains("per_peer_bandwidth_budget"));
    //       }
    //       ThrottleResult::Applied(_) => panic!("expected Throttled, got Applied"),
    //   }
    //
    //   // Smaller merges below the budget pass through:
    //   let small_merge = synthesize_loro_merge_with_byte_size(512);
    //   assert!(matches!(
    //       throttle.try_apply(&small_merge),
    //       ThrottleResult::Applied(_)
    //   ));
    //
    // OBSERVABLE consequence: the rate-limit policy from benten-caps
    // is observably consumed by the benten-sync Loro-merge throttle;
    // back-pressure surfaces as a typed `ThrottleResult::Throttled`
    // variant carrying the wait-until HLC + reason. Defends against
    // the producer/consumer drift where the producer (caps) ships the
    // policy but no consumer integrates it. Closes the R3-B → R3-C
    // coordination handoff caught by tcc-r1-2.
    unimplemented!(
        "G16-B wires LoroMergeThrottle::from_policy + ThrottleResult::Throttled at the production loro-merge path"
    );
}
