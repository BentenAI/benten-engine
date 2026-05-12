//! LOAD-BEARING per plan §3 G24-D row.
//!
//! Verifies the **ratified user-as-source signing model** per
//! CLAUDE.md #18 "Implementation refinements" + post-R1-triage Q4 +
//! post-2026-05-11-conversation D-4F-12 retense.
//!
//! Would-FAIL if a Benten-project-key were the install-record signer.
//!
//! Per pim-2 §3.6b end-to-end test pin discipline AND R2 §5 Gap fixes —
//! pair the negative "NO Benten-project-key" assertion with a positive
//! "user-DID is the signer + UCAN chain traces to user-root" assertion
//! so the SUBSTANTIVE shape (not just shape-only) is verified.

mod common;

use common::manifest_fixtures::{stub_install_record, stub_user_did};

#[test]
#[ignore = "RED-PHASE: G24-D wave fills user-DID install-record signing; un-ignore at G24-D landing"]
fn install_record_is_signed_by_user_did_anchored_in_users_graph() {
    let manifest_cid = common::manifest_fixtures::stub_cid_zero();
    let install = stub_install_record(manifest_cid);

    // POSITIVE: user-DID is the consenting signer.
    assert_eq!(install.consenting_user_did, stub_user_did());

    // SUBSTANTIVE: future G24-D surface will be
    // `InstallRecord::verify_user_signature(&user_did) -> Result<()>`.
    // FAILS-IF-NO-OP because signature bytes would be 0-bytes and
    // ed25519 verify would reject. Stub at R3.
}

#[test]
#[ignore = "RED-PHASE: G24-D wave provides codebase-grep assertion infrastructure (or the absence is exhibited at G24-D landing)"]
fn no_benten_project_key_infrastructure_in_codebase_grep_assert() {
    // Per R2 §5 substance discipline: grep-assert the codebase
    // CONTAINS NO functions/types/symbols matching the Benten-project-
    // key pattern. The user-as-source signing model is structural;
    // there should be no "project key" anywhere.
    //
    // Future surface: a build-time / test-time grep over
    // crates/benten-platform-foundation/src/ for symbol patterns
    // matching `benten_project_key|BentenProjectKey|project_signing_key|
    // BENTEN_PROJECT_KEY`. Assert: count == 0.
    //
    // At R3 RED-PHASE, the platform-foundation crate is mostly empty
    // stubs so this trivially passes, but un-ignoring at G24-D requires
    // the assertion still hold after the full implementation lands.

    // R3 stub: pretend assertion runs against the directory above.
    // G24-D implementer must implement the actual grep walker + count==0
    // assertion against the post-implementation source tree.
    panic!(
        "RED-PHASE: G24-D wave must implement grep-assertion over crates/benten-platform-foundation/src/ counting 0 matches for benten_project_key|BentenProjectKey patterns"
    );
}
