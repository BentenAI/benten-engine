//! Phase-4-Foundation R4-FP-1 — T4 pin: admin-UI-DID cannot mint a
//! root grant.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-1 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T4
//! defense step 3 ("No admin-UI-DID self-elevation") + CLAUDE.md
//! baked-in #18 Layer 1.
//!
//! ## What this pin establishes
//!
//! Admin UI v0's plugin-DID is a UCAN audience handle (NOT an attested
//! sub-identity per CLAUDE.md #18 four-identity-concepts ratification).
//! Plugin-DID can HOLD caps (delegated from user-root) but CANNOT mint
//! root grants. The cap-policy backend's grant-mint surface MUST verify
//! the grant-issuer is the user DID; admin-UI-plugin-DID as issuer of a
//! root grant MUST return typed `E_NOT_USER_ROOT`.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer omits the issuer-is-user-DID check at grant-mint. Admin
//! UI plugin code (or any non-user-DID principal) issues a root grant
//! to itself; cap-store admits it; subsequent writes trace back to a
//! "root" that's actually the plugin-DID — silent trust-anchor
//! short-circuit.
//!
//! Per threat-model §T4 defense step 3: "Cap-policy backend's grant-
//! mint surface checks the issuer is the user DID; admin-UI-plugin-DID
//! as issuer of a root grant returns `E_NOT_USER_ROOT`. Plugin-DID is
//! just an audience identifier — it has no inherent authority."

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "RED-PHASE: G24-B-FP wires grant-mint issuer-is-user-DID check; un-ignore at G24-B-FP landing. Pin source: r4-triage §1 r4-tc-1 + threat-model §T4 step 3."]
fn admin_ui_did_attempt_to_mint_root_grant_surfaces_e_not_user_root() {
    // G24-B-FP wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //
    //   // Attempt: admin-UI-DID issues a root grant for itself for an
    //   // arbitrary scope. This is the self-elevation attack — plugin
    //   // forging the user-as-root anchor.
    //   let mint_attempt = harness.mint_root_grant(
    //       /* issuer */ admin_ui_did.clone(),
    //       /* audience */ admin_ui_did.clone(),
    //       /* scope */ "store:notes:write",
    //   );
    //
    //   // Cap-policy backend's grant-mint surface MUST refuse because
    //   // issuer is NOT the user DID. Plugin-DID is a UCAN audience
    //   // handle, NOT a mint authority — per CLAUDE.md #18 four-
    //   // identity-concepts.
    //   let err = mint_attempt.expect_err(
    //       "T4 step 3: admin-UI-DID MUST NOT mint root grants — \
    //        plugin-DID is an audience handle, not an issuer of \
    //        root authority"
    //   );
    //
    //   assert!(
    //       matches!(err.code(), ErrorCode::E_NOT_USER_ROOT),
    //       "T4 step 3: must surface typed E_NOT_USER_ROOT; got {:?}",
    //       err.code()
    //   );
    //
    //   // Defense-in-depth: confirm the cap-store has NOT been written.
    //   // A no-op implementer might surface E_NOT_USER_ROOT but still
    //   // commit the grant — observable check that no admin-UI-DID-
    //   // issued root grant exists after the attempt:
    //   let admin_ui_root_grants = harness
    //       .cap_store()
    //       .grants_issued_by(&admin_ui_did)
    //       .into_iter()
    //       .filter(|g| g.is_root())
    //       .count();
    //   assert_eq!(
    //       admin_ui_root_grants, 0,
    //       "T4 step 3: NO admin-UI-DID-issued root grants must exist \
    //        in cap-store after rejected mint attempt"
    //   );
    //
    // OBSERVABLE consequence: structural prevention of plugin-DID-as-
    // root-issuer; cap-store integrity preserved.
    unimplemented!(
        "G24-B-FP wires admin-UI-DID-cannot-mint-root-grant (T4 step 3). \
         Substantive end-to-end: real cap-store + grant-mint attempt + \
         typed E_NOT_USER_ROOT + cap-store integrity check."
    );
}
