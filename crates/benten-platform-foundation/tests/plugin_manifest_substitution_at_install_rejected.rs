//! Phase-4-Foundation R4-FP-1 — T5b LOAD-BEARING pin: plugin content
//! substitution at install rejected (peer-DID signature check).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-4 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T5
//! (T5b sub-threat) + defense step 3 (DID resolution for peer-DID at
//! install).
//!
//! ## What this pin establishes
//!
//! Per threat-model §T5b: "Attacker intercepts plugin content delivery
//! and substitutes bytes signed by a DIFFERENT peer-DID. If install
//! flow doesn't validate the peer-DID signature against the user's
//! known-trusted-author list, user installs hostile content under the
//! same human-readable name."
//!
//! Defense: admin UI v0 install flow refuses install if the peer-DID
//! who signed the original content is NOT in user's known-trusted-
//! author list (first install = explicit confirmation; subsequent
//! installs from same peer-DID = silent).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires content-CID match check but skips peer-DID
//! signature verification. Attacker constructs hostile content;
//! signs with attacker's key; manifest claims peer-DID = alice
//! (a trusted author); install flow accepts because CID matches own
//! bytes. Substitution attack succeeds.
//!
//! Sibling to `plugin_content_carries_peer_did_signature_for_provenance.rs`
//! (existing R3 pin) — that pins the POSITIVE arm; this pin is the
//! NEGATIVE rejection arm per pim-2 §3.6b sub-rule 4.

#![allow(clippy::unwrap_used)]

mod common;

use common::manifest_fixtures::{
    stub_peer_did_alice, stub_peer_did_attacker, stub_plugin_did, stub_user_did,
};

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn plugin_install_with_content_substituted_by_attacker_peer_did_rejected() {
    let _plugin = stub_plugin_did();
    let _user = stub_user_did();

    // G24-D wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_lifecycle::install_plugin;
    //
    //   let mut engine = common::manifest_fixtures::test_engine_with_user_did();
    //   let user_did = stub_user_did();
    //   let alice = stub_peer_did_alice();
    //   let attacker = stub_peer_did_attacker();
    //
    //   // User's trust-list: alice is a known author.
    //   common::manifest_fixtures::set_user_trusted_authors(
    //       &mut engine, &user_did, vec![alice.clone()]
    //   );
    //
    //   // Attack: manifest CLAIMS peer-DID = alice, but signature is
    //   // actually from attacker's key. Content CID hashes its own
    //   // bytes (passes content-addressing).
    //   let hostile_manifest = common::manifest_fixtures::
    //       hostile_manifest_with_forged_peer_did_claim(
    //           /* claimed_peer_did */ alice.clone(),
    //           /* actual_signing_key */ attacker.clone(),
    //           /* content */ b"hostile bytes",
    //       );
    //
    //   let install_attempt = install_plugin(
    //       &mut engine,
    //       /* manifest */ hostile_manifest,
    //       /* user_consent_record */ common::manifest_fixtures::
    //           install_record_signed_by_user(&user_did, vec![]),
    //   );
    //
    //   // T5b LOAD-BEARING: install MUST be REJECTED because the
    //   // claimed peer-DID (alice) does NOT match the actual signing
    //   // key (attacker). Without this check, substitution succeeds
    //   // silently.
    //   let err = install_attempt.expect_err(
    //       "T5b LOAD-BEARING: install with content signed by \
    //        different peer-DID than CLAIMED MUST be REJECTED — \
    //        substitution attack must not succeed"
    //   );
    //   assert!(
    //       matches!(err.code(),
    //           ErrorCode::E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID),
    //       "T5b: must surface typed peer-signature-invalid per arch-r1-3 \
    //        ErrorCode split; got {:?}", err.code()
    //   );
    //
    //   // Defense-in-depth: cap-store + manifest-store UNCHANGED:
    //   let installed = engine.manifest_store().installed_plugins();
    //   assert!(
    //       installed.is_empty(),
    //       "T5b: rejected install MUST NOT commit any state — \
    //        manifest-store contains {} entries", installed.len()
    //   );
    //
    //   // Boundary: SAME attacker-signed content, but with manifest
    //   // CLAIMING attacker (not alice). User's trust-list does NOT
    //   // include attacker → prompts user (T9 trust-list path) per
    //   // threat-model §T5 defense step 3. Distinct attack: forged-
    //   // claim vs unknown-author:
    //   let unknown_author_install = install_plugin(
    //       &mut engine,
    //       common::manifest_fixtures::manifest_signed_by(&attacker, b"different bytes"),
    //       common::manifest_fixtures::install_record_signed_by_user(&user_did, vec![]),
    //   );
    //   match unknown_author_install {
    //       Err(e) if matches!(e.code(),
    //           ErrorCode::E_PLUGIN_AUTHOR_NOT_TRUSTED) => {},
    //       _ => panic!("T5b: unknown-author install MUST surface \
    //                    E_PLUGIN_AUTHOR_NOT_TRUSTED for user prompt"),
    //   }
    //
    // OBSERVABLE consequence: forged-claim vs unknown-author are
    // distinct typed errors; substitution attack defeated.
    panic!(
        "RED-PHASE: G24-D must wire install-time peer-DID signature \
         verification (T5b LOAD-BEARING). Substantive: forged-claim \
         attack + typed E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID + \
         no-state-commit + unknown-author boundary."
    );
}
