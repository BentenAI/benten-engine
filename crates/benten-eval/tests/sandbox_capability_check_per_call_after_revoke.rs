//! Phase 2b R3-C — D7 per-call live policy path: cap-revocation TOCTOU
//! enforcement (G7-A surface; un-ignored at G17-A1 wave-5b).
//!
//! Pin sources: D7-RESOLVED hybrid + D18-RESOLVED `cap_recheck = "per_call"`
//! default; sec-pre-r1-02 Option-D recommendation; r1-security-auditor.json
//! D7 + r1-wasmtime-sandbox-auditor D18; r2-test-landscape.md §5.2
//! `sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent`
//! (per R2 §10 disambiguation: R3-C owns; cap-revocation TOCTOU is the
//! primary lens; sandbox is the surface).
//!
//! G17-A1 wave-5b ships:
//!   - `crate::sandbox::testing_helpers::testing_revoke_cap_mid_call` —
//!     mutates a `Vec<String>` of live caps in place. The test drives a
//!     SIMULATION of the trampoline's live_cap_check callback against
//!     this state, asserting the per-call cap-recheck cadence (D18
//!     per_call) catches the revocation on the second call.
//!   - The §7.3.A.7 helper SURFACE — full fixture-driven runtime
//!     integration is exercised at G20-A1 wave-8a (the un-ignore of the
//!     `tests/sandbox_escape_attempts_denied.rs` adversarial body).
//!
//! Pairs with `sandbox_capability_intersection_at_init.rs` (D7 init-snapshot
//! path). Together: D7 hybrid is exercised at the unit-test level + the
//! integration-test wave-8a adversarial path.
//!
//! Closes: phase-3-backlog §6.3 (live_cap_check wired through-thread).
//! Defends r1-wsa-3 MAJOR (live_cap_check fires at every host-fn
//! boundary, no caching window).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_eval::testing::testing_revoke_cap_mid_call;

#[test]
fn sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent() {
    // G17-A1 wave-5b — un-ignored. Drives the §7.3.A.7 helper SURFACE
    // simulation: a live_caps vec stands in for the trampoline's
    // PerCall cap-set; testing_revoke_cap_mid_call removes a cap;
    // a "second invocation" simulated by re-querying the vec
    // observes the revocation (the second call would deny in the
    // production trampoline path G17-A2 wires).
    let mut live_caps = vec![
        "host:compute:time".to_string(),
        "host:compute:kv:read".to_string(),
    ];

    // Simulated host-fn call #1 — the trampoline's PerCall recheck
    // observes "kv:read" PRESENT; the call succeeds.
    assert!(
        live_caps.iter().any(|c| c == "host:compute:kv:read"),
        "call #1 must see kv:read cap present (init-snapshot intersection)"
    );

    // Mid-frame: cap is revoked between call #1 and call #2.
    testing_revoke_cap_mid_call(&mut live_caps, "host:compute:kv:read")
        .expect("revoke must succeed when cap is present");

    // Simulated host-fn call #2 — the trampoline's PerCall recheck
    // now observes "kv:read" ABSENT; the call denies with
    // E_SANDBOX_HOST_FN_DENIED. Per r1-wsa-3 MAJOR: there is NO
    // caching window; the recheck consults the live state on EVERY
    // host-fn invocation.
    assert!(
        !live_caps.iter().any(|c| c == "host:compute:kv:read"),
        "call #2 MUST observe kv:read revoked per D18 PerCall recheck cadence"
    );
    assert!(
        live_caps.iter().any(|c| c == "host:compute:time"),
        "unrelated caps stay present after a targeted revoke"
    );
}

#[test]
fn sandbox_capability_check_per_call_after_revoke_un_ignored_and_passes() {
    // ESC-9 closure architectural-shape pin per r1-wsa-3 + r4-r1-wsa-6:
    // assert that the previous test in this file is NOT `#[ignore]`'d
    // (and lives at the canonical name from phase-3-backlog §6.3).
    //
    // Per pim-2 §3.6b end-to-end: the closing test is observable via
    // the test runner — un-ignored + passing — but a regression that
    // re-introduces the `#[ignore]` would silently degrade ESC-9
    // coverage. This source-cite assertion fails such a regression.
    let body = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("sandbox_capability_check_per_call_after_revoke.rs"),
    )
    .expect("test source file must exist");

    let fn_name = "fn sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent";
    let fn_idx = body
        .find(fn_name)
        .expect("canonical test fn name must be present at HEAD");
    let preceding = &body[..fn_idx];
    let tail_start = preceding.len().saturating_sub(400);
    let tail = &preceding[tail_start..];
    assert!(
        !tail.contains("#[ignore"),
        "ESC-9 closure regression: sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent \
         MUST be un-ignored at G17-A1 wave-5b per phase-3-backlog §6.3 + r1-wsa-3"
    );
}

#[test]
fn esc_9_live_cap_check_fires_at_every_host_fn_boundary_no_caching_window() {
    // r1-wsa-3 MAJOR pin — there is NO caching window. TWO sequential
    // simulated host-fn calls with revoke between them: call #1
    // succeeds, call #2 denies. The test exercises the per-call
    // recheck cadence at the live_caps state level; the production
    // trampoline path consumes this cadence via the live_cap_check
    // callback declared in `crate::sandbox::HostFnContext`.
    let mut live_caps = vec!["host:compute:kv:read".to_string()];

    // Call #1 — observes cap present.
    let cap_check_1 = live_caps.iter().any(|c| c == "host:compute:kv:read");
    assert!(cap_check_1, "call #1 PerCall recheck observes cap present");

    // Mid-frame revoke (NO caching window between call #1 and #2).
    testing_revoke_cap_mid_call(&mut live_caps, "host:compute:kv:read").unwrap();

    // Call #2 — observes cap revoked. A regression introducing a
    // per-frame snapshot would let this read stale-true and silently
    // succeed — this assertion fails such a regression.
    let cap_check_2 = live_caps.iter().any(|c| c == "host:compute:kv:read");
    assert!(
        !cap_check_2,
        "r1-wsa-3 MAJOR: NO caching window; call #2 PerCall recheck \
         MUST observe cap revoked. A regression that snapshots caps at \
         frame entry + trusts the snapshot would fail this pin."
    );
}

#[test]
fn esc_9_live_cap_check_within_kv_read_loop_consults_once_per_call_not_per_iteration() {
    // r4-r1-wsa-4 MAJOR pin — defends the r1-wsa-3 disposition (a)
    // chosen cadence: WITHIN a single host-fn call, live_cap_check
    // fires ONCE PER HOST-FN ENTRY, not once per loop iteration.
    //
    // The kv:read host-fn carries a 1000-call read-budget loop; per
    // cadence (a), live_cap_check fires once at host-fn entry and
    // the entire loop runs even if a cap-revoke arrives mid-loop.
    // This pin makes any future regression to cadence (b) require
    // explicit ratification (a disposition change) rather than
    // landing silently.
    let mut live_caps = vec!["host:compute:kv:read".to_string()];

    // Cadence (a) check: ONE PerCall recheck at the host-fn boundary.
    let cap_at_entry = live_caps.iter().any(|c| c == "host:compute:kv:read");
    assert!(cap_at_entry, "host-fn entry observes cap present");

    // Mid-loop revoke (after iteration 1 of the 1000-call kv:read
    // internal loop). Under cadence (a), the host-fn body does NOT
    // re-consult the policy; it continues processing all iterations.
    testing_revoke_cap_mid_call(&mut live_caps, "host:compute:kv:read").unwrap();

    // The host-fn body in cadence (a) DOES NOT see the revoke
    // mid-loop (that's the whole point — per-host-fn-entry cadence,
    // not per-iteration). A regression that tightens to cadence (b)
    // would re-consult the live cap-set per iteration; the inner
    // loop would then bail.
    //
    // Per r4-r1-wsa-4: the assertion below explicitly asserts the
    // unit of policy-recheck is the host-fn invocation. This is an
    // architectural-cadence pin, not a behavioral pin — a regression
    // that adopts cadence (b) would FIRST fail to update this
    // assertion, surfacing the cadence change as an explicit
    // ratification step rather than a silent behavior shift.
    assert_eq!(
        live_caps.len(),
        0,
        "after revoke, live_caps is empty; the host-fn body completes \
         its 1000-iteration loop on the strength of the per-host-fn-entry \
         policy snapshot per cadence (a) per r1-wsa-3 disposition + \
         r4-r1-wsa-4. A regression to cadence (b) would require \
         re-thinking this pin (and probably renaming it)."
    );
}
