//! ESC-9 (cap-revoke mid-call) closure pins (G17-A1 wave-5b).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-A1 row +
//! r1-revision-triage):
//!
//! - `tests/sandbox_capability_check_per_call_after_revoke_un_ignored_and_passes`
//!   — ESC-9 closure (asserts the existing
//!   `sandbox_capability_check_per_call_after_revoke.rs` is un-ignored
//!   at G17-A1 wave-5b).
//! - `tests/esc_9_live_cap_check_fires_at_every_host_fn_boundary_no_caching_window`
//!   — r1-wsa-3 MAJOR; live_cap_check fires at EVERY host-fn boundary.
//! - `tests/esc_9_live_cap_check_within_kv_read_loop_consults_once_per_call_not_per_iteration`
//!   — r4-r1-wsa-4 MAJOR; cadence (a) once-per-host-fn-entry pin.
//!
//! ## ESC-9 closure shape
//!
//! Phase-2b shipped the `sandbox_capability_check_per_call_after_revoke`
//! body but left it `#[ignore]`'d pointing to phase-3-backlog §7.3.A.7
//! (the testing-helper SURFACE). G17-A1 wave-5b ships:
//!
//! 1. The §7.3.A.7 helper SURFACE
//!    ([`benten_eval::testing::testing_revoke_cap_mid_call`], cfg-gated
//!    behind `cfg(any(test, feature = "test-helpers", feature = "testing"))`
//!    per r1-wsa-6 enumerated narrative).
//! 2. The architectural-shape assertions that the un-ignore landed
//!    (sibling test file
//!    `crates/benten-eval/tests/sandbox_capability_check_per_call_after_revoke.rs`
//!    has the canonical un-ignored test names).
//! 3. The narrative pin that the callback fires AT EVERY host-fn
//!    boundary (not cached at frame start), per r1-wsa-3.
//!
//! G17-A2 wave-5b owns wiring the production trampoline path
//! (`SandboxStoreData.live_caps` → engine-backed callable); G17-A1
//! ships the helper + cadence pins. The full fixture-driven
//! integration test un-ignores at G20-A1 wave-8a.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
fn sandbox_capability_check_per_call_after_revoke_un_ignored_and_passes() {
    // ESC-9 closure pin — verifies the sibling un-ignore landed.
    //
    // Per pim-2 §3.6b end-to-end: the closing-claim PR must
    // surface an observable consequence. Here the observable
    // consequence is the SIBLING test file
    // `sandbox_capability_check_per_call_after_revoke.rs` having
    // the named un-ignored test (which is run + passes by the same
    // test runner; this pin asserts the un-ignore happened at HEAD).
    let body = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("sandbox_capability_check_per_call_after_revoke.rs"),
    )
    .expect("sibling test file must exist");

    let fn_name = "fn sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent";
    let fn_idx = body
        .find(fn_name)
        .expect("canonical sibling test fn name must be present at HEAD");
    let preceding = &body[..fn_idx];
    let tail_start = preceding.len().saturating_sub(400);
    let tail = &preceding[tail_start..];
    assert!(
        !tail.contains("#[ignore"),
        "G17-A1 wave-5b MUST un-ignore sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent \
         per ESC-9 closure (phase-3-backlog §6.3 + r1-wsa-3)"
    );
}

#[test]
fn esc_9_live_cap_check_fires_at_every_host_fn_boundary_no_caching_window() {
    // r1-wsa-3 MAJOR architectural-shape pin. Defends the narrative:
    // the live_cap_check callback fires at EVERY host-fn boundary;
    // there is NO caching window where the cap is consulted once at
    // frame entry then trusted for the remainder of the frame.
    //
    // The host_fns.rs source carries the narrative; this pin asserts
    // the callback field exists + the doc-comment names the cadence
    // as "live cap-recheck" (not "cached snapshot").
    let host_fns_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("sandbox")
            .join("host_fns.rs"),
    )
    .expect("host_fns.rs must exist");

    assert!(
        host_fns_src.contains("live_cap_check"),
        "host_fns.rs MUST declare the live_cap_check callback field per r1-wsa-3"
    );

    // Narrative-shape pin: the doc-comment names "every" host-fn
    // invocation as the cadence. A regression that introduces a
    // per-frame snapshot would silently degrade ESC-9 coverage; the
    // narrative-name pin makes such a regression explicit (the
    // code change would have to update the narrative too).
    let cadence_named = host_fns_src.contains("BEFORE every")
        || host_fns_src.contains("on each invocation")
        || host_fns_src.contains("every host-fn")
        || host_fns_src.contains("BEFORE EVERY");
    assert!(
        cadence_named,
        "host_fns.rs MUST narrate live_cap_check cadence as fires-on-every-call \
         per r1-wsa-3 MAJOR (no caching window)"
    );
}

#[test]
fn esc_9_live_cap_check_within_kv_read_loop_consults_once_per_call_not_per_iteration() {
    // r4-r1-wsa-4 MAJOR cadence pin — defends the once-per-host-fn-
    // entry cadence (option (a) per r1-wsa-3 disposition). The
    // host_fns.rs narrative locks the cadence; this pin asserts the
    // narrative names the unit of policy-recheck as the host-fn
    // invocation.
    let host_fns_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("sandbox")
            .join("host_fns.rs"),
    )
    .expect("host_fns.rs must exist");

    // Narrative names the cadence as PerCall (host-fn-invocation),
    // not per-loop-iteration:
    assert!(
        host_fns_src.contains("PerCall"),
        "host_fns.rs MUST declare the PerCall cap-recheck cadence variant per r1-wsa-3 disposition (a)"
    );
    assert!(
        host_fns_src.contains("per_call_read_cap") || host_fns_src.contains("per-call read"),
        "kv:read internal loop runs against a PER-CALL read budget; the cap-recheck cadence is \
         PER-CALL (not per-iteration) per r4-r1-wsa-4"
    );
}
