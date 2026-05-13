//! Phase-4-Foundation R4-FP-1 — T10-upgrade (a) pin: plugin upgrade
//! requires same author DID.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-3 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T10-upgrade step 4(a) + plan §3 G24-D-FP-1.
//!
//! ## What this pin establishes
//!
//! Per threat-model §T10-upgrade (a): "Upgrade from peer-DID alice to
//! peer-DID attacker (transitive substitution) MUST be REJECTED — the
//! silent-upgrade path requires same author DID + accepting a different
//! peer-DID is a re-install requiring new consent."
//!
//! Defense: `verify_upgrade_author_continuity(old, new) -> Result` at
//! `crates/benten-platform-foundation/src/module_ecosystem.rs` rejects
//! with `E_PLUGIN_AUTHOR_NOT_TRUSTED` if peer-DIDs differ.
//!
//! Per pim-2-amendment §3.6b sub-rule 4: T10-upgrade has multiple
//! sub-arms (a) same-author, (b) reject-downgrade. This pin is the
//! (a) arm only.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires upgrade flow but skips peer-DID author identity
//! check. Attacker delivers an "upgrade" CID with different peer-DID
//! signature; upgrade flow accepts; admin UI is now signed by a
//! different peer-DID without re-consent.

#![allow(clippy::unwrap_used)]

mod common;

use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::module_ecosystem::verify_upgrade_author_continuity;
use common::manifest_fixtures::minimal_manifest;

#[test]
fn plugin_upgrade_with_different_peer_did_author_rejected_with_typed_error() {
    // SUBSTANTIVE per pim-2 §3.6b + pim-2-amendment sub-rule 4:
    // build two manifests with DIFFERENT peer_did values; exercise
    // verify_upgrade_author_continuity; expect typed
    // PluginAuthorNotTrusted. Would-FAIL if continuity check skipped
    // (silent upgrade would admit attacker's peer-DID).
    let alice = Keypair::generate();
    let attacker = Keypair::generate();
    assert_ne!(
        alice.public_key().to_did(),
        attacker.public_key().to_did(),
        "test setup: distinct keypairs"
    );

    let mut old = minimal_manifest();
    old.peer_did = alice.public_key().to_did();

    let mut new_attacker = minimal_manifest();
    new_attacker.peer_did = attacker.public_key().to_did();

    let err = verify_upgrade_author_continuity(&old, &new_attacker)
        .expect_err("T10-upgrade (a) MUST reject different peer-DID");
    assert_eq!(
        err,
        ErrorCode::PluginAuthorNotTrusted,
        "T10-upgrade (a): different peer-DID MUST surface typed \
         PluginAuthorNotTrusted; would-FAIL if continuity check skipped"
    );
}

#[test]
fn plugin_upgrade_with_same_peer_did_author_admits_continuity_check() {
    // SUBSTANTIVE boundary per pim-2 §3.6b: complementary positive arm
    // — same peer-DID admits. Would-FAIL if continuity check
    // over-rejected.
    let alice = Keypair::generate();

    let mut old = minimal_manifest();
    old.peer_did = alice.public_key().to_did();

    let mut new_same_author = minimal_manifest();
    new_same_author.peer_did = alice.public_key().to_did();

    verify_upgrade_author_continuity(&old, &new_same_author).expect("same peer-DID MUST admit");
}
