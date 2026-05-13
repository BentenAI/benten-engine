//! Phase-4-Foundation R4-FP-1 — T4 pin: cap-policy chain validation at
//! the synchronous write boundary (defense-in-depth with
//! apply_atrium_merge).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-1 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T4
//! defense step 2 ("Walk-time principal = chain-rooted-at-user").
//!
//! ## What this pin establishes
//!
//! Per threat-model §T4 step 2: "Chain validation at `apply_atrium_merge`
//! (structural-always-on per-row recheck, PR #161) AND at the
//! synchronous write boundary."
//!
//! The Phase-3 G16-B-F per-row recheck inside `apply_atrium_merge`
//! handles the Atrium-merge surface; this pin establishes that the
//! parallel chain-validator firing at `Engine::call_as` synchronous
//! write entry is structurally always-on (not predicate-gated), forming
//! the defense-in-depth pair.
//!
//! Without this companion check, a write that bypasses the Atrium-merge
//! path (purely local write originated on the same peer) would skip
//! chain validation entirely.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires chain validation only at the Atrium-merge boundary
//! (since Phase-3 already shipped that). A direct local write via
//! `Engine::call_as` with a chain that fails user-root validation
//! succeeds because the synchronous write boundary skipped the check.
//! Test asserts the synchronous boundary fires the same validator —
//! catches the gap.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "RED-PHASE: G24-B-FP wires synchronous write-boundary chain validation; un-ignore at G24-B-FP landing. Pin source: r4-triage §1 r4-tc-1 + threat-model §T4 step 2 defense-in-depth."]
fn cap_policy_chain_validation_fires_at_synchronous_write_boundary() {
    // G24-B-FP wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //
    //   // Construct a chain that signature-verifies cleanly but DOES
    //   // NOT trace to user-root (e.g., chain from a forged peer-DID).
    //   let non_user_rooted_chain = harness
    //       .construct_signature_valid_but_not_user_rooted_chain();
    //
    //   // Capture chain-validator invocations.
    //   let validator_trace = harness.capture_chain_validator_calls(|h| {
    //       h.dispatch_local_write_via_call_as(
    //           admin_ui_did.clone(),
    //           "store:notes:write",
    //           vec![0u8; 32],
    //           non_user_rooted_chain.clone(),
    //       )
    //   });
    //
    //   // Defense-in-depth: chain validator MUST fire at the
    //   // synchronous write boundary (i.e., inside Engine::call_as).
    //   // Without this, only Atrium-merge boundary validates — local
    //   // writes bypass the check.
    //   let synchronous_fires = validator_trace.calls_at_site(
    //       "Engine::call_as::pre_write_chain_validation"
    //   );
    //   assert!(
    //       !synchronous_fires.is_empty(),
    //       "T4 step 2: chain validator MUST fire at synchronous write \
    //        boundary; trace shows ZERO invocations — defense-in-depth \
    //        gap (only apply_atrium_merge fires, leaving local-write \
    //        path unguarded)"
    //   );
    //
    //   // And the validator MUST have rejected the non-user-rooted
    //   // chain at the synchronous boundary specifically:
    //   assert!(
    //       synchronous_fires
    //           .iter()
    //           .any(|call| matches!(call.outcome, ValidationOutcome::Rejected { .. })),
    //       "T4 step 2: synchronous validator MUST reject non-user-rooted \
    //        chain; outcomes: {:?}",
    //       synchronous_fires.iter().map(|c| &c.outcome).collect::<Vec<_>>()
    //   );
    //
    //   // Structural-always-on (NOT predicate-gated): test that the
    //   // validator fires REGARDLESS of write type — couples to the
    //   // G16-B-F structural-always-on per-row-recheck precedent.
    //   let validator_calls_unconditional = harness
    //       .capture_chain_validator_calls(|h| {
    //           h.dispatch_local_write_via_call_as(
    //               admin_ui_did.clone(),
    //               "private:admin-ui-private:auto_save",
    //               vec![0u8; 4],
    //               harness.user_rooted_chain(), // even valid chain triggers validator
    //           )
    //       });
    //   assert!(
    //       !validator_calls_unconditional
    //           .calls_at_site("Engine::call_as::pre_write_chain_validation")
    //           .is_empty(),
    //       "T4 step 2: validator MUST fire structurally — every call_as \
    //        invocation triggers chain check (not predicate-gated)"
    //   );
    //
    // OBSERVABLE consequence: defense-in-depth pair complete; both
    // synchronous + Atrium-merge boundaries fire the validator.
    unimplemented!(
        "G24-B-FP wires cap-policy chain validation at synchronous \
         write boundary (T4 step 2 defense-in-depth). Substantive \
         end-to-end: chain-validator trace at Engine::call_as boundary \
         + structural-always-on assertion + reject-on-non-user-rooted."
    );
}
