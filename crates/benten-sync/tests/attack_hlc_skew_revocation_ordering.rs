//! Compromise #25 closure pin (HLC-monotonic enforcement at sync layer).
//!
//! R6-FP Wave-C1 (ds-r6-1 / hlc-r6-r1-1 closure) — sec-r4r2-1
//! attack-vector pin un-ignored against the live production
//! `Hlc::update` skew classifier.
//!
//! Pre-Wave-C1 this test was RED-PHASE under
//! `#[ignore = "RED-PHASE: G14-pre-D + G16-D wire inbound-sync-frame
//! HLC skew classifier"]` with an `unimplemented!()` body. Wave-C1
//! wires the per-row `Hlc::update` call inside
//! `crates/benten-engine/src/engine.rs::apply_atrium_merge`'s row-loop;
//! this sync-crate pin asserts the underlying production-grade
//! `benten_core::hlc::Hlc::update` skew classifier rejects with
//! `CoreError::HlcSkewExceeded` against an adversarial
//! `BentenHlc { physical_ms = u64::MAX/2 }` stamp. The engine-side
//! end-to-end pin (production sync-frame ingress → typed engine error
//! mapping to `E_SYNC_HLC_DRIFT`) lives at
//! `crates/benten-engine/tests/sync_inbound_hlc_skew_rejected.rs`.
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
//!    revoked party.
//!
//! An adversarial peer can manipulate its local HLC to inject sync
//! frames with future-HLC values, biasing both decisions. The
//! `Hlc::update` skew classifier caps inbound drift relative to the
//! local clock; future-skew exceeding the tolerance window rejects
//! WITHOUT mutating local clock state.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! - drives the production receive path (`Hlc::update`) — same
//!   classifier the `apply_atrium_merge` row loop calls;
//! - asserts an OBSERVABLE behavioral consequence (typed-error variant
//!   `CoreError::HlcSkewExceeded` + local clock NOT mutated +
//!   `Hlc::now()` post-rejection still less than the future-skew stamp);
//! - would FAIL if the inbound skew-cap check were silently no-op'd.
//!
//! ## R4-FP-4 substance audit cross-reference
//!
//! This file is audited at
//! `.addl/phase-4-foundation/notes-wave-c1-attack-audit.md` §2.1 per
//! sec-3.5-r1-8 + pim-18 §3.6f SHAPE-not-SUBSTANCE pre-flight. Audit
//! verdict: SUBSTANTIVE — production-surface (`Hlc::update`) + 4
//! observable-consequence assertions + would-FAIL-if-no-op'd documented
//! inline. No substance gaps detected at HEAD.

#![allow(clippy::unwrap_used)]

use benten_core::CoreError;
use benten_core::hlc::{BentenHlc, Hlc};

#[test]
fn hlc_skew_exceeded_in_inbound_sync_frame_rejected_with_e_hlc_skew_exceeded() {
    // sec-r4r2-1 attack-vector pin (R6-FP Wave-C1 closure).
    //
    // Construct a deterministic local HLC at "now" via a mock clock
    // returning 1_000 ms-since-epoch. The mock clock signature is
    // `fn() -> u64` so the mock state lives in a `static` per the
    // `benten_core::hlc::Hlc::new` PhysicalClockFn contract.
    fn mock_now_ms() -> u64 {
        1_000
    }
    let local = Hlc::new(/* node_id = */ 0xAAAA, mock_now_ms);

    // Adversarial peer crafts a sync frame with a future-HLC stamp:
    // u64::MAX/2 ms is well beyond the default 5-minute (300_000 ms)
    // skew tolerance window. Defends against the LWW-bias forgery.
    let adversarial_hlc = BentenHlc::new(
        /* physical_ms = */ u64::MAX / 2,
        /* logical    = */ 0,
        /* node_id    = */ 0xBBBB, // attacker peer node-id
    );

    // OBSERVABLE consequence #1: local clock state captured BEFORE
    // the rejection so consequence #4 (state-not-mutated) can verify
    // the rejection arm did not advance local state.
    let pre_attack_now = local.now();

    // FIRST line of defense — the skew classifier rejects BEFORE
    // local state mutation.
    let result = local.update(&adversarial_hlc);

    match result {
        Err(CoreError::HlcSkewExceeded {
            local_physical_ms,
            remote_physical_ms,
            tolerance_ms,
        }) => {
            assert_eq!(
                local_physical_ms, 1_000,
                "local_physical_ms should match the mock clock reading at rejection time"
            );
            assert_eq!(
                remote_physical_ms,
                u64::MAX / 2,
                "remote_physical_ms should match the adversarial HLC stamp"
            );
            assert_eq!(
                tolerance_ms,
                Hlc::DEFAULT_SKEW_TOLERANCE_MS,
                "tolerance_ms should be the default 5-minute window"
            );
            // OBSERVABLE consequence #2: typed catalog code maps to
            // the stable `E_HLC_SKEW_EXCEEDED` (the sync-boundary
            // surface code `E_SYNC_HLC_DRIFT` fires at the engine
            // wireup; THIS classifier surfaces the underlying
            // benten-core code).
            assert_eq!(
                CoreError::HlcSkewExceeded {
                    local_physical_ms,
                    remote_physical_ms,
                    tolerance_ms,
                }
                .code(),
                benten_errors::ErrorCode::HlcSkewExceeded
            );
        }
        Err(other) => panic!(
            "expected HlcSkewExceeded; got {other:?} — \
             inbound HLC skew-cap defense was silently no-op'd or fired the wrong typed error"
        ),
        Ok(stamp) => panic!(
            "attack succeeded — future-HLC stamp was applied; LWW resolution + revocation-vs-data \
             ordering are open to HLC-skew injection. Returned stamp: {stamp:?}"
        ),
    }

    // OBSERVABLE consequence #3: local clock state was NOT mutated by
    // the rejection. Per the `Hlc::update` doc-string contract: 'The
    // local state is **not** mutated in [the skew-rejection] case'.
    // A second `now()` call must return a stamp at the same physical
    // tick the pre-attack stamp observed (mock clock pinned at 1_000),
    // not at the adversarial future-skew physical_ms.
    let post_attack_now = local.now();
    assert!(
        post_attack_now.physical_ms() <= pre_attack_now.physical_ms() + 1,
        "post-attack local clock physical_ms ({post}) advanced past pre-attack ({pre}) — \
         HlcSkewExceeded mutated local state, violating the no-mutate-on-reject contract",
        post = post_attack_now.physical_ms(),
        pre = pre_attack_now.physical_ms(),
    );
    assert!(
        post_attack_now.physical_ms() < adversarial_hlc.physical_ms(),
        "post-attack local clock advanced into the adversarial future-skew range — \
         the skew-rejection arm leaked state mutation"
    );
}
