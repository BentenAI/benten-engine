//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for admin UI v0
//! subscribe paths flowing ONLY via `Engine::on_change_as_with_cursor`,
//! never `Engine::subscribe_change_events`.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 4 + §5 SHAPE-not-SUBSTANCE smell-test pairing; closes sec-3.5-r1-9
//! + T12 (admin UI subscribe redaction + cap-recheck per-row on event
//! delivery, G16-B-F precedent + ratification #7 revoke-mid-session).
//!
//! ## What this pin establishes
//!
//! Per G22-FP-1 PR #210 closure (`CapRecheckOutcome` shipped LIVE) + T12:
//! `on_change_as_with_cursor` is the cap-recheck-enabled subscribe seam.
//! Each event delivery invokes a per-row `CapabilityPolicy::check_read`;
//! revoked grants cause `Drop` (skip event) or `Cancel` (terminate sub).
//!
//! The older `subscribe_change_events` surface does NOT recheck caps on
//! event delivery — it's reserved for engine-internal flows (audit log,
//! IVM materialization). Admin UI plugin code reaching for it is the
//! exact failure shape this pin defends against.
//!
//! ## SHAPE+SUBSTANCE pairing (per pim-18 §3.6f, R2 §5 table row 1)
//!
//! This file pairs the grep-assert (compile-time source-text presence)
//! with a **runtime trace assertion** (actually subscribe and assert
//! the trace-event hits `on_change_as_with_cursor` not
//! `subscribe_change_events`). R2 §5 explicitly flagged this pin as
//! shape-smell-prone; both halves land HERE in one file for cohesion.

#![allow(clippy::unwrap_used)]

mod common;

/// SHAPE half — grep-assert that admin UI source contains zero
/// references to `subscribe_change_events` and at least one reference
/// to `on_change_as_with_cursor`.
#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A wave-6 wires this. Pin source: r2-test-landscape.md §2.6 row 4 + §5 row 1. SHAPE half of pim-18 SHAPE+SUBSTANCE pair."]
fn admin_ui_v0_source_uses_only_on_change_as_with_cursor_for_subscribe() {
    // G24-A wave wires this. Substantive shape:
    //
    //   // Walk admin UI v0 plugin source (Rust + TS). G24-A defines
    //   // the canonical roots — likely
    //   // `crates/benten-platform-foundation/src/admin_ui_v0/` (Rust
    //   // handlers) + `packages/admin-ui-v0/src/` (TS shell). Both
    //   // surfaces must be swept.
    //   let roots = [
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..")
    //           .join("benten-platform-foundation")
    //           .join("src")
    //           .join("admin_ui_v0"),
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..")
    //           .join("..")
    //           .join("packages")
    //           .join("admin-ui-v0")
    //           .join("src"),
    //   ];
    //
    //   let mut found_correct = 0_usize;
    //   for root in &roots {
    //       for entry in walkdir::WalkDir::new(root) {
    //           let entry = entry.unwrap();
    //           if !entry.file_type().is_file() { continue; }
    //           let src = std::fs::read_to_string(entry.path()).unwrap();
    //           assert!(
    //               !src.contains("subscribe_change_events"),
    //               "Admin UI v0 source MUST NEVER call \
    //                Engine::subscribe_change_events (no cap-recheck \
    //                seam); found in {}",
    //               entry.path().display(),
    //           );
    //           if src.contains("on_change_as_with_cursor") {
    //               found_correct += 1;
    //           }
    //       }
    //   }
    //
    //   assert!(
    //       found_correct > 0,
    //       "Admin UI v0 source MUST call on_change_as_with_cursor at \
    //        least once (cap-recheck-enabled subscribe seam); ZERO \
    //        references found — admin UI either has no live-update or \
    //        is bypassing cap-recheck"
    //   );
    //
    // OBSERVABLE consequence: shape-level grep defense. Required
    // companion to the SUBSTANCE pin below.
    unimplemented!(
        "G24-A wires admin UI source grep-assert (SHAPE half of \
         pim-18 §3.6f pair)"
    );
}

/// SUBSTANCE half — runtime trace assertion that the admin UI actually
/// uses `on_change_as_with_cursor` when it subscribes at runtime.
#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A wave-6 wires this. Pin source: r2-test-landscape.md §5 row 1 substance-check (pim-18 §3.6f). Runtime trace: actually subscribe + assert trace hits on_change_as_with_cursor not subscribe_change_events."]
fn admin_ui_v0_subscribe_runtime_traces_to_on_change_as_with_cursor_not_subscribe_change_events() {
    // G24-A wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //
    //   // Open admin UI Workflows view — it sets up a live subscribe
    //   // to render new workflows as users create them:
    //   let trace = harness.trace_capture(|h| {
    //       let view = h.dispatch_admin_ui_route("workflows");
    //       // Cause a write to the workflows label so the subscribe
    //       // delivers an event:
    //       let _new = h.create_test_workflow();
    //       view.await_first_event();
    //   });
    //
    //   // Per pim-18 §3.6f SUBSTANCE check:
    //   assert!(
    //       trace.calls_to("Engine::on_change_as_with_cursor").len() >= 1,
    //       "Admin UI subscribe MUST hit on_change_as_with_cursor at \
    //        runtime (cap-recheck seam); trace shows ZERO invocations"
    //   );
    //   assert_eq!(
    //       trace.calls_to("Engine::subscribe_change_events").len(),
    //       0,
    //       "Admin UI subscribe MUST NEVER hit subscribe_change_events \
    //        at runtime; trace shows {:?}",
    //       trace.calls_to("Engine::subscribe_change_events"),
    //   );
    //
    // OBSERVABLE consequence: cap-recheck fires on every subscribed
    // event delivery. Defends against the failure shape where grep
    // passes (no source-text reference) but a transitive dep reaches
    // the bypass surface.
    unimplemented!(
        "G24-A wires admin UI subscribe runtime-trace pin (SUBSTANCE \
         half of pim-18 §3.6f pair)"
    );
}
