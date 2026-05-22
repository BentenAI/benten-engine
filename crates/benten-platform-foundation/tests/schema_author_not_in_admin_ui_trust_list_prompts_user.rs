//! Phase-4-Foundation R4-FP-1 — T9 pin: schema author not in admin UI
//! trust-list prompts user (Q3 default = EMPTY per Ben's ratification).
//!
//! **G-CORE-7 un-ignore (§3.6e destination-remapped from
//! `phase-4-backlog.md §4.19` Phase-4-Meta carry).** The
//! `ProvenanceOutcome::UserPromptRequired` surface lands at G-CORE-7 in
//! `plugin_manifest::verify_schema_provenance_with_trust_list`; this
//! pin un-ignores against that surface.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-2 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T9
//! defense step 5 ("User-trusted-peer-DID list in admin UI manifest
//! envelope") + r4-triage §7 ratification "Q3 schema-author trust-list
//! default: EMPTY + admin UI prompts at first encounter".
//!
//! ## What this pin establishes
//!
//! Per threat-model §T9 defense step 5: admin UI's
//! `requires.schema_authors` enumerates peer-DIDs the admin UI is
//! allowed to materialize schemas from. **Per Ben's Q3 ratification
//! (r4-triage §7)**: the default trust-list is EMPTY — admin UI v0
//! ships with no pre-trusted peer-DIDs; first encounter with each
//! peer-DID surfaces `ProvenanceOutcome::UserPromptRequired` for the
//! admin UI to consume (NOT auto-reject, NOT auto-accept).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires DID resolution + signature verification but
//! skips the trust-list check (auto-accepts all peer-DIDs). Or ships
//! default trust-list NON-EMPTY (e.g., baked-in Benten authors). Or
//! makes the trust-list check `Err(E_PLUGIN_AUTHOR_NOT_TRUSTED)`
//! (hard-reject — collapses the user-prompt UX surface). The
//! substantive arm pins that the Q3 ratification structurally fires:
//! default-empty → prompt; trusted-list-includes-peer → silent.

#![allow(clippy::unwrap_used)]

mod common;

use benten_platform_foundation::{
    PluginManifest, ProvenanceOutcome, verify_schema_provenance_with_trust_list,
};

#[test]
fn schema_author_not_in_admin_ui_trust_list_returns_user_prompt_outcome() {
    // 1. Confirm Ben Q3 ratification at the structural level: admin UI
    // v0 manifest ships with default-empty `requires_schema_authors`.
    let admin_ui_manifest: PluginManifest = common::manifest_fixtures::admin_ui_v0_manifest();
    assert!(
        admin_ui_manifest
            .requires_schema_authors
            .as_ref()
            .is_none_or(std::vec::Vec::is_empty),
        "Ben Q3 ratification: admin UI v0 default trust-list MUST be \
         EMPTY (None or empty Vec) — got {:?}",
        admin_ui_manifest.requires_schema_authors
    );

    // 2. A schema signed by an unknown peer-DID (alice; not in any
    // trust-list) surfaces `UserPromptRequired` — NOT auto-accept, NOT
    // auto-reject. The admin UI consumes this to surface the prompt.
    let unknown_peer = common::manifest_fixtures::stub_peer_did_alice();
    let outcome = verify_schema_provenance_with_trust_list(&unknown_peer, &admin_ui_manifest);
    match outcome {
        ProvenanceOutcome::UserPromptRequired { peer_did } => {
            assert_eq!(
                peer_did, unknown_peer,
                "T9 trust-list: prompt outcome MUST carry the unknown \
                 peer-DID for user decision"
            );
        }
        ProvenanceOutcome::Trusted => panic!(
            "T9 trust-list: schema from non-trusted peer MUST surface \
             UserPromptRequired (NOT Trusted / auto-accept) — Ben Q3 \
             ratification"
        ),
    }

    // 3. After the user trusts alice (trust-list grows to include
    // alice), a second schema from alice surfaces `Trusted` (silent).
    // This proves the trust-list extension is the round-trip the
    // user-prompt UX is designed to close.
    let admin_ui_manifest_with_alice =
        common::manifest_fixtures::admin_ui_v0_manifest_with_trust_list(vec![unknown_peer.clone()]);
    let outcome2 =
        verify_schema_provenance_with_trust_list(&unknown_peer, &admin_ui_manifest_with_alice);
    assert!(
        matches!(outcome2, ProvenanceOutcome::Trusted),
        "T9 trust-list: schema from peer in trust-list MUST materialize \
         silently (Trusted); got {outcome2:?}"
    );
}
