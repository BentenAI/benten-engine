//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for dogfood path (d):
//! revoke cap mid-session — subscription terminates; toast + re-render
//! to redacted view.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 14 (LOAD-BEARING §3.6f substantive); closes ux-r1-1 + ratification
//! #7 (revoke-mid-session UX) + T12 (subscribe-time cap-recheck).
//!
//! ## Per pim-18 §3.6f LOAD-BEARING substantive shape
//!
//! Production-runtime arms:
//! 1. **Live subscribe via `on_change_as_with_cursor`** with a real
//!    UCAN cap-grant from user-DID to admin-UI-DID.
//! 2. **Cap revoked mid-session** by user-DID via real revocation flow
//!    (`Engine::revoke_capability_by_grant_cid` shipped at PR #199).
//! 3. **Subscription receives `CapRecheckOutcome::Cancel`** per
//!    G22-FP-1 PR #210 LIVE seam; admin UI surfaces toast notification.
//! 4. **Re-render shows redacted shape** (not stale-cached unauthorized
//!    data) — verified by reading the rendered DOM after cap revocation.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A wave-6 wires this. Pin source: r2-test-landscape.md §2.6 row 14 + ratification #7 + T12. LOAD-BEARING per pim-18 §3.6f: real revoke flow (PR #199 surface) + CapRecheckOutcome::Cancel observation + toast surfaced + redacted re-render. Would FAIL if cap-recheck no-op'd on event delivery."]
fn dogfood_path_d_revoke_cap_mid_session_ux_acceptance() {
    // G24-A wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //
    //   // Setup: admin-UI-DID holds a cap-grant on 'notes' label.
    //   // Live-subscribe via on_change_as_with_cursor:
    //   let user_did = harness.user_did();
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //   let grant_cid = harness.user_did_issues_grant(
    //       &user_did, &admin_ui_did, "store:notes:read+subscribe"
    //   );
    //
    //   let notes_view = harness.dispatch_admin_ui_route("notes_live_view");
    //   let initial_dom = notes_view.dom_snapshot();
    //   assert!(
    //       initial_dom.contains("note_1_title"),
    //       "Initial render MUST show authorised notes content"
    //   );
    //
    //   // (1) (2) Subscribe is live; revoke mid-session:
    //   let trace = harness.trace_capture(|h| {
    //       // User revokes the grant — real surface, PR #199:
    //       h.engine_revoke_capability_by_grant_cid(&grant_cid).unwrap();
    //       notes_view.await_re_render_or_termination(
    //           std::time::Duration::from_secs(2)
    //       )
    //   });
    //
    //   // (3) CapRecheckOutcome::Cancel observed at evaluator
    //   // per G22-FP-1 PR #210 seam:
    //   assert!(
    //       trace.cap_recheck_outcomes.iter().any(|o| {
    //           matches!(o, benten_eval::primitives::subscribe::CapRecheckOutcome::Cancel)
    //       }),
    //       "Dogfood path (d): mid-session cap revoke MUST surface \
    //        CapRecheckOutcome::Cancel per PR #210 seam; no Cancel \
    //        outcome seen — cap-recheck is no-op'd on event delivery"
    //   );
    //
    //   // (3 cont.) Toast surfaced to user per ratification #7:
    //   let toasts = notes_view.toast_history();
    //   assert!(
    //       toasts.iter().any(|t| {
    //           t.kind == "cap_revoked" && t.text.contains("permission removed")
    //       }),
    //       "Dogfood path (d): UX MUST surface cap-revoked toast per \
    //        ratification #7; toasts seen: {:?}",
    //       toasts,
    //   );
    //
    //   // (4) Re-render shows redacted shape (NOT stale data per T12):
    //   let post_revoke_dom = notes_view.dom_snapshot();
    //   assert!(
    //       !post_revoke_dom.contains("note_1_title"),
    //       "Dogfood path (d): post-revoke DOM MUST redact the previously \
    //        visible content per T12; saw '{}'",
    //       post_revoke_dom,
    //   );
    //   assert!(
    //       post_revoke_dom.contains("access revoked") ||
    //       post_revoke_dom.contains("not authorized"),
    //       "Dogfood path (d): post-revoke DOM MUST show redacted-state \
    //        affordance per ratification #7 UX; saw '{}'",
    //       post_revoke_dom,
    //   );
    //
    // OBSERVABLE consequence: cap revocation propagates through admin
    // UI within session. Defends against the most-cited failure shape
    // for live-subscribe UX (data leak on stale-cached client).
    unimplemented!(
        "G24-A wires dogfood path (d): revoke-mid-session with 4-arm \
         production-runtime check (live subscribe + real revoke flow + \
         CapRecheckOutcome::Cancel observation + redacted re-render) \
         per pim-18 §3.6f"
    );
}
