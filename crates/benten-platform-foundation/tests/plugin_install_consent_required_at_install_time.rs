//! G24-D row + CLAUDE.md #18 Layer 2 — install-time consent required.
//!
//! User reviews `requires` + `shares` at install; consents to
//! envelope. Without an install record carrying user-DID signature,
//! install fails with `E_PLUGIN_INSTALL_CONSENT_REQUIRED`.

mod common;

use common::manifest_fixtures::minimal_manifest;

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn install_without_user_consent_record_surfaces_e_plugin_install_consent_required() {
    let _manifest = minimal_manifest();

    // Future surface:
    //   plugin_lifecycle::install_plugin(manifest, install_record_opt:
    //     Option<&InstallRecord>) -> Result
    // returns ErrorCode::PluginInstallConsentRequired when
    // install_record_opt is None.
    //
    // FAILS-IF-NO-OP because the consent gate must explicitly check
    // the install record's presence and user-DID signature validity.
    panic!("RED-PHASE: G24-D wave must wire install consent gate");
}
