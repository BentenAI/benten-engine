//! Phase 4-Foundation R3 (Family A — R1-FP wave-1 G22-FP-2 regression-
//! defense). Grep-assert paired with the
//! `ucan_grounded_policy_rejects_proof_for_wrong_audience_before_time_check.rs`
//! production-runtime acceptance test (SHIPPED at PR #208 closing
//! cap-r1-1 + cap-r1-9 BLOCKERs).
//!
//! # Charter
//!
//! Per R3 dispatch brief (Family A R1-FP regression-defense section).
//! The G22-FP-2 closure landed the audience-binding ordering at
//! `crates/benten-caps/src/ucan_grounded.rs::typed_cap_permitted_by_proof`:
//! when an audience is threaded the chain-walker MUST use
//! `validate_chain_for_audience_at` (composes audience-binding BEFORE
//! time-window check); only the `audience=None` fallback path may use
//! `validate_chain_at` (audience-less; legacy + engine-internal typed-
//! CALL paths that haven't yet threaded actor).
//!
//! # What this pin asserts (would-FAIL-if-no-op'd per §3.6b)
//!
//! Source-level invariants on `crates/benten-caps/src/ucan_grounded.rs`:
//!
//! 1. The source MUST mention `validate_chain_for_audience_at` (the
//!    audience-binding-first walker) — proves the post-fix call site
//!    has not been reverted to audience-less `validate_chain_at` alone.
//! 2. The source MUST contain the audience-aware match arm shape
//!    `Some(aud) =>` paired with `validate_chain_for_audience_at`, AND
//!    the audience-less fallback arm `None =>` paired with
//!    `validate_chain_at`. This pins the cap-r1-9 ORDERING contract:
//!    when audience is threaded, audience-binding fires; without
//!    audience, fallback to the legacy walker (preserves Phase-1/2
//!    fixtures).
//!
//! Removing the audience-aware branch (collapsing to
//! `self.ucan.validate_chain_at(chain, self.now_secs)` unconditionally)
//! trips both assertions — the regression mode the post-fix
//! production test in `ucan_grounded_policy_rejects_proof_for_wrong_audience_before_time_check.rs`
//! catches at runtime, this test catches at source-level for defense-
//! in-depth (an agent could in theory regress the impl while
//! preserving the test name).
//!
//! # Pairs with
//!
//! - `crates/benten-caps/tests/ucan_grounded_policy_rejects_proof_for_wrong_audience_before_time_check.rs`
//!   (the §3.6b production-runtime arm; SHIPPED at PR #208).
//!
//! # Status
//!
//! NOT RED-PHASE — this is a regression-defense pin guarding a SHIPPED
//! closure. Runs unconditionally in CI; fails if anyone reverts the
//! audience-binding wiring.
//!
//! # Owned by
//!
//! Phase 4-Foundation R3 Family A test-writer (regression-defense set).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn ucan_grounded_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ucan_grounded.rs");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

#[test]
fn ucan_grounded_typed_cap_permitted_by_proof_calls_validate_chain_for_audience_at() {
    let body = ucan_grounded_source();
    assert!(
        body.contains("validate_chain_for_audience_at"),
        "expected `validate_chain_for_audience_at` reference in \
         crates/benten-caps/src/ucan_grounded.rs (cap-r1-1 + cap-r1-9 \
         BLOCKER closure regression-defense): the audience-binding-first \
         chain walker MUST be the call path when an audience is threaded \
         (G22-FP-2 SHIPPED at PR #208). If this assertion fires, the \
         audience-binding wiring has been reverted to the audience-less \
         `validate_chain_at` — re-run the companion \
         `ucan_grounded_policy_rejects_proof_for_wrong_audience_before_time_check` \
         test for the production-runtime arm proof + re-land the fix.",
    );
}

#[test]
fn ucan_grounded_audience_some_arm_routes_through_audience_walker() {
    let body = ucan_grounded_source();

    // Substantive match shape: the audience-binding wiring lives in a
    // match arm that dispatches by `audience: Option<&Did>`. The
    // canonical post-fix form pins the SHAPE:
    //
    //   let chain_check = match audience {
    //       Some(aud) => ... validate_chain_for_audience_at(chain, aud, ...) ...
    //       None => ... validate_chain_at(chain, ...) ...
    //   };
    //
    // We assert both branch shapes are present in the source so a
    // regression that collapses to a single branch (audience-less or
    // audience-only) trips this gate.
    let has_some_arm =
        body.contains("Some(aud)") && body.contains("validate_chain_for_audience_at");
    let has_none_arm =
        body.contains("None =>") && body.contains("validate_chain_at(chain, self.now_secs)");

    assert!(
        has_some_arm,
        "expected `Some(aud) => ... validate_chain_for_audience_at` match \
         arm in ucan_grounded.rs (cap-r1-9 ordering pin): audience-aware \
         walker MUST be selected when caller threads a principal; \
         collapsing to the audience-less walker would re-introduce \
         the cap-r1-1 BLOCKER.",
    );
    assert!(
        has_none_arm,
        "expected `None => ... validate_chain_at(chain, self.now_secs)` \
         match arm in ucan_grounded.rs: the audience-less fallback \
         preserves Phase-1/2 fixtures + engine-internal typed-CALL paths \
         that don't yet thread actor (per CapWriteContext {{ actor_hint: \
         None, .. }} at engine_wait.rs:881-891).",
    );
}
