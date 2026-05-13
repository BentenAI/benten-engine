//! Phase-4-Foundation R4-FP-1 — T4 LOAD-BEARING pin: admin UI v0
//! background write MUST trace to user-root.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-1 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T4
//! ("Admin-UI-internal-as-principal forgery") + CLAUDE.md baked-in #18
//! Layer 1 (user-as-root anchor).
//!
//! ## What this pin establishes
//!
//! Admin UI v0 runs under its own per-plugin DID (`admin-UI-DID`). A
//! background write (timer-triggered refresh / auto-save / background
//! sync) MUST thread a UCAN chain whose root grant traces to a
//! user-mint root. If the background scheduler forgets to thread the
//! user identity, the write MUST be DENIED with typed
//! `E_CHAIN_NOT_USER_ROOTED` (NOT silently admitted under the
//! admin-UI-DID's own authority).
//!
//! Per CLAUDE.md baked-in #18 Layer 1 ("user-as-root"): every cap chain
//! traces to a user-mint root. Background writes triggered by admin UI
//! v0's own scheduler are the classic short-circuit attack vector —
//! admin UI plugin code authoring a write WITHOUT the user-as-root
//! chain must NOT succeed under admin-UI-DID's own grant.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer omits the chain-validator check at the synchronous write
//! boundary. Background write proceeds against admin-UI-DID's own grant
//! (admin-UI-DID has caps for `private:admin-ui-private:*` — Layer 3
//! delegation envelope intact in isolation). Test sees `Ok(_)` instead
//! of `Err(E_CHAIN_NOT_USER_ROOTED)` — silent admin-UI-as-root.
//!
//! LOAD-BEARING per r4-triage §1 r4-tc-1 + threat-model §T4 §3 list
//! item 4: this is the substantive end-to-end pin per pim-2 §3.6b.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "RED-PHASE: G24-B-FP wires background-write chain validator; un-ignore at G24-B-FP wave landing. Pin source: r4-triage §1 r4-tc-1 + threat-model §T4 LOAD-BEARING. Substantive end-to-end per pim-2 §3.6b: real evaluator walk + chain-trace check + typed E_CHAIN_NOT_USER_ROOTED."]
fn admin_ui_v0_background_write_must_trace_to_user_root() {
    // G24-B-FP wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //
    //   // Construct an admin-UI background-scheduler dispatch WITHOUT
    //   // a user-as-root chain — only admin-UI-DID's own grant
    //   // (private:admin-ui-private:*). Background scheduler "forgot"
    //   // to thread the user identity.
    //   let background_dispatch = harness.dispatch_background_write(
    //       // active principal = admin-UI-DID
    //       admin_ui_did.clone(),
    //       // payload: write to private namespace
    //       "private:admin-ui-private:auto_save",
    //       vec![0u8; 32],
    //       // chain provenance: only admin-UI-DID's own delegation;
    //       // NO user-DID root grant in the chain
    //       harness.admin_ui_did_only_chain(),
    //   );
    //
    //   // LOAD-BEARING: chain-validator at the synchronous write
    //   // boundary MUST reject the chain because it lacks user-root.
    //   // Admin-UI-DID cannot self-authorize; user-DID is the trust
    //   // anchor per CLAUDE.md #18 Layer 1.
    //   let err = background_dispatch.expect_err(
    //       "T4 LOAD-BEARING: admin UI v0 background write WITHOUT \
    //        user-root chain MUST be DENIED — chain validator at \
    //        synchronous write boundary missed the Layer 1 trust anchor"
    //   );
    //
    //   // Defense-in-depth: the typed error must be the chain-not-rooted
    //   // variant (NOT generic E_CAP_DENIED — implementer must surface
    //   // the trust-anchor short-circuit specifically per threat-model
    //   // §T4 step 4):
    //   assert!(
    //       matches!(err.code(), ErrorCode::E_CHAIN_NOT_USER_ROOTED),
    //       "T4: must surface typed E_CHAIN_NOT_USER_ROOTED; got {:?}",
    //       err.code()
    //   );
    //
    //   // Audit-log invariant: the rejection MUST emit an Inv-13-style
    //   // record naming admin-UI-DID + the attempted scope. Confirms
    //   // observable consequence beyond return-value.
    //   let audit = harness.audit_log_since_last_dispatch();
    //   assert!(
    //       audit.iter().any(|r| r.principal == admin_ui_did
    //           && r.outcome.is_denied()),
    //       "T4: audit log MUST record admin-UI-DID denial for \
    //        observability"
    //   );
    //
    // OBSERVABLE consequence: trust-anchor enforcement at the write
    // boundary; admin UI v0 cannot escalate to root.
    unimplemented!(
        "G24-B-FP wires admin UI v0 background-write-must-trace-to-user-root \
         (T4 LOAD-BEARING). Substantive end-to-end per pim-2 §3.6b: real \
         evaluator dispatch + chain-validator check + typed \
         E_CHAIN_NOT_USER_ROOTED + audit-log invariant."
    );
}
