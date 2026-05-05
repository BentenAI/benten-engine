//! R3-D RED-PHASE pins for ESC-9 (cap-revoke mid-call) closure
//! (G17-A1 wave-5b).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-A1 row + r1-revision-triage):
//!
//! - `tests/sandbox_capability_check_per_call_after_revoke_un_ignored_and_passes`
//!   — ESC-9 closure (un-ignores the existing
//!   `sandbox_capability_check_per_call_after_revoke.rs` body shipped
//!   in Phase 2b but `#[ignore]`'d pending the helper SURFACE).
//! - `tests/esc_9_live_cap_check_fires_at_every_host_fn_boundary_no_caching_window`
//!   — r1-wsa-3 MAJOR; the live_cap_check callback fires at EVERY
//!   host-fn boundary (not cached / not at frame start), so a
//!   cap-revoke between two host-fn calls in the same SANDBOX frame
//!   is observable on the next call.
//!
//! ## ESC-9 closure shape
//!
//! Phase-2b shipped the `sandbox_capability_check_per_call_after_revoke`
//! body but left it `#[ignore]`'d pointing to phase-3-backlog §7.3.A.7
//! (the testing-helper SURFACE). G17-A1 wave-5b ships:
//!
//! 1. The §7.3.A.7 helper SURFACE (`testing_revoke_cap_mid_call` etc.,
//!    cfg-gated behind `cfg(any(test, feature = "test-helpers"))` per
//!    r1-wsa-6).
//! 2. The `live_cap_check` callback wired through-thread per
//!    phase-3-backlog §6.3, consuming the G13-pre-C `cap_recheck`
//!    helper.
//! 3. The narrative pin that the callback fires AT EVERY host-fn
//!    boundary (not cached at frame start), per r1-wsa-3.
//!
//! ## Why two distinct pin functions
//!
//! Per pim-2 §3.6b end-to-end test pin requirement: a "fix landed but
//! the runtime arm is silently no-op" failure shape is defended by
//! distinct pin functions covering distinct observable consequences.
//!
//! - `..._after_revoke_un_ignored_and_passes` asserts the existing
//!   Phase-2b test body now drives end-to-end (the `#[ignore]` is
//!   removed at G17-A1; the body executes).
//! - `..._fires_at_every_host_fn_boundary_no_caching_window` asserts
//!   the absence of a caching window: TWO host-fn calls in the same
//!   SANDBOX frame, with cap revocation between them, both observe
//!   the revocation on the second call.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-A1 wave-5b ships the §7.3.A.7 helper SURFACE + live_cap_check wire-through; un-ignores `sandbox_capability_check_per_call_after_revoke`"]
fn sandbox_capability_check_per_call_after_revoke_un_ignored_and_passes() {
    // ESC-9 closure pin. G17-A1 implementer wires this.
    //
    // ## R4-FP recommendation per r4-r1-wsa-6 (MINOR — narrative tightening)
    //
    // The current source-cite-scan shape (substring grep of
    // `#[ignore`) is brittle: it depends on the function name spelling
    // matching exactly + the 400-char preceding window covering the
    // full attribute block. R4-FP recommends the implementer choose
    // ONE of the two sharper shapes at un-ignore time:
    //
    // - **Option (a) — preferred:** RETIRE this source-cite assertion
    //   when wave-5b lands. The Phase-2b `#[ignore]`'d test runs
    //   directly as part of the suite; orchestrator's
    //   `cargo nextest run -p benten-eval` is the end-to-end signal.
    //   No secondary source-cite pin needed; this fn becomes redundant
    //   once the un-ignore lands.
    // - **Option (b) — fallback:** REPLACE the substring-grep with a
    //   `syn`-crate AST scan (~20 LOC change). Walk the file's `#[test]`
    //   items + verify the named fn has zero `#[ignore]` siblings.
    //   Robust against attribute-block layout drift.
    //
    // STEP 1 — flip the existing `#[ignore]` in
    //   `crates/benten-eval/tests/sandbox_capability_check_per_call_after_revoke.rs`
    //   from `#[ignore = "Phase 3 — D7 hybrid + D18 per_call ..."]`
    //   to no `#[ignore]`. The body already exists.
    //
    // STEP 2 — wire the §7.3.A.7 helper SURFACE
    //   `benten_eval::testing::testing_revoke_cap_mid_call` so the
    //   existing body's `R5 surfaces consumed` block compiles + runs.
    //
    // STEP 3 — pick option (a) or (b):
    //   - Option (a): retire this fn (let the live test be the signal).
    //   - Option (b): replace the substring-grep below with a `syn`-AST scan.
    //
    //   let body = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("tests").join("sandbox_capability_check_per_call_after_revoke.rs")
    //   ).unwrap();
    //   // Find the `fn sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent`
    //   // attribute block immediately preceding it.
    //   let fn_idx = body.find(
    //       "fn sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent"
    //   ).unwrap();
    //   let preceding = &body[..fn_idx];
    //   // The trailing 200 chars before the fn must NOT contain `#[ignore`:
    //   let tail = &preceding[preceding.len().saturating_sub(400)..];
    //   assert!(!tail.contains("#[ignore"),
    //       "G17-A1 must un-ignore sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent \
    //        per ESC-9 closure (phase-3-backlog §6.3 + r1-wsa-3)");
    //
    // OBSERVABLE consequence: the Phase-2b ESC-9 test runs end-to-end
    // and asserts the typed `SandboxHostFnDenied` error fires after a
    // mid-execution revoke. Defends against the "fix landed but the
    // existing test stayed silently `#[ignore]`'d" failure shape that
    // pim-2 was named for.
    unimplemented!(
        "G17-A1 wires the un-ignore source-cite assertion + ships testing_revoke_cap_mid_call helper"
    );
}

#[test]
#[ignore = "RED-PHASE: G17-A1 wave-5b wires live_cap_check to fire at every host-fn boundary (not cached)"]
fn esc_9_live_cap_check_fires_at_every_host_fn_boundary_no_caching_window() {
    // r1-wsa-3 MAJOR pin. G17-A1 implementer wires this:
    //
    //   use benten_eval::sandbox::{Sandbox, SandboxConfig, ManifestRef};
    //   use benten_eval::testing::testing_revoke_cap_mid_call;
    //
    //   // Build a SANDBOX frame whose guest calls kv:read TWICE in
    //   // sequence with no intervening trap.
    //   let sandbox = Sandbox::new(/* config with kv:read cap granted */);
    //   let revoke_handle = sandbox.testing_install_revoke_handle();
    //   let result = sandbox.execute_with_callback(|frame| {
    //       // Mid-frame: between call #1 and call #2, revoke kv:read.
    //       frame.after_host_fn_n(1, |f| testing_revoke_cap_mid_call(f, "kv:read"));
    //   });
    //
    //   // Call #1 succeeded; call #2 hit a typed denial:
    //   assert_eq!(result.host_fn_calls_succeeded(), 1);
    //   assert!(matches!(
    //       result.unwrap_err(),
    //       benten_eval::SandboxError::HostFnDenied { code, .. }
    //         if code == benten_errors::ErrorCode::SandboxHostFnDenied
    //   ));
    //
    // This pin is DISTINCT from the previous one because it asserts a
    // sequence-of-two property (not just "one revoke surfaces") —
    // specifically that there is no caching window of ANY size where
    // the cap is consulted once at frame entry then trusted for the
    // remainder of the frame. The cap is re-checked on EACH host-fn
    // call.
    //
    // OBSERVABLE consequence: a regression that introduces a per-frame
    // cap snapshot (e.g. for "performance") would let call #2 succeed
    // even after revoke. This pin fails. Defends r1-wsa-3 specifically.
    unimplemented!(
        "G17-A1 wires per-host-fn-boundary live_cap_check assertion exercising sequential calls + mid-frame revoke"
    );
}

#[test]
#[ignore = "RED-PHASE: G17-A1 wave-5b — within-host-fn-call cadence pin (r4-r1-wsa-4; locks r1-wsa-3 disposition (a) once-per-host-fn-entry, NOT once-per-backend-touch)"]
fn esc_9_live_cap_check_within_kv_read_loop_consults_once_per_call_not_per_iteration() {
    // r4-r1-wsa-4 MAJOR pin. Defends the r1-wsa-3 disposition (a)
    // chosen cadence — within a SINGLE host-fn call, live_cap_check
    // fires ONCE PER HOST-FN ENTRY, not once per backend touch / per
    // loop iteration inside the host-fn implementation.
    //
    // The previous two pins establish:
    // - That live_cap_check fires (vs. caching).
    // - That two distinct host-fn calls each consult fresh.
    //
    // This pin is DISTINCT because it pins the cadence WITHIN a single
    // host-fn call. The kv:read host-fn carries a 1000-call read-budget
    // loop (per host-functions.toml `per_call_read_cap = 1000`); a
    // cadence-(b) implementation would re-check the cap on every
    // iteration of that loop, giving stricter security but changing
    // the runtime contract. r1-revision-triage chose cadence-(a) per
    // HOST-FUNCTIONS.md §kv:read 'every invocation re-asks the policy'
    // — the unit of policy-recheck is the host-fn invocation, not the
    // sub-loop iteration.
    //
    // Without this pin, an R5 implementer who picks (b) for security
    // would still pass the existing ..._fires_at_every_host_fn_boundary
    // pin (because (b) is strictly stronger), but the chosen
    // disposition would be silently overturned. A future regression
    // back to (a) would also pass the existing pin without surfacing
    // the cadence change.
    //
    // G17-A1 implementer wires this:
    //
    //   use benten_eval::sandbox::{Sandbox, SandboxConfig, ManifestRef};
    //   use benten_eval::testing::testing_revoke_cap_mid_call;
    //
    //   // Build a SANDBOX whose guest issues ONE kv:read call that
    //   // reads multiple blobs (driving the 1000-call internal
    //   // read-budget loop):
    //   let sandbox = Sandbox::new(/* config with kv:read cap granted */);
    //   let revoke_handle = sandbox.testing_install_revoke_handle();
    //
    //   // Cap revoke happens AFTER the first iteration of the kv:read
    //   // internal loop (i.e. after blob 1 is read, before blob 2 is
    //   // read), but BEFORE the kv:read host-fn call returns.
    //   let result = sandbox.execute_with_callback(|frame| {
    //       frame.after_kv_read_iteration_n(1, |f| {
    //           testing_revoke_cap_mid_call(f, "kv:read");
    //       });
    //   });
    //
    //   // Per cadence (a) — live_cap_check fires ONCE at host-fn entry,
    //   // succeeds (cap was held at entry), and the single kv:read host-
    //   // fn call SUCCEEDS WHOLLY for all blobs:
    //   assert!(result.is_ok(),
    //       "kv:read host-fn call MUST succeed wholly under cadence (a) \
    //        once-per-host-fn-entry per r1-wsa-3 disposition + r4-r1-wsa-4; \
    //        a regression to cadence (b) once-per-iteration would fail mid-loop");
    //   assert_eq!(result.unwrap().blobs_read(), expected_total_blobs,
    //       "all blobs read because the cap-recheck cadence is per-host-fn-entry, \
    //        not per-iteration; this distinguishes cadence (a) from (b)");
    //
    // OBSERVABLE consequence: a regression that tightens kv:read's
    // internal cap-recheck to per-iteration (cadence (b), well-meaning
    // for security) fails this pin. The pin DOES NOT prevent the
    // future ratification of cadence (b) — it asserts that any change
    // away from (a) requires explicit ratification (a disposition
    // change in r4-r1 or successor brief), not a silent tightening.
    //
    // Pairs with the previous pin (..._fires_at_every_host_fn_boundary)
    // — that one verifies cadence at the host-fn boundary; this one
    // verifies cadence WITHIN a host-fn. Two distinct dimensions of
    // the live_cap_check contract per pim-2 §3.6b.
    unimplemented!(
        "G17-A1 wires within-host-fn-call cadence assertion (r4-r1-wsa-4; locks disposition (a) once-per-host-fn-entry per r1-wsa-3)"
    );
}
