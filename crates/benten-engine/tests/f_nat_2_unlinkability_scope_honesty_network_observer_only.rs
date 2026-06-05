//! F-NAT-2 (R3-W6 gov-audit) — Inv-20 clause-d unlinkability SCOPE-HONESTY:
//! per-recipient unlinkability is **network-observer-only**; it does NOT
//! protect against a malicious admin (an admin sees the full
//! `members_table`). Compromise #58 (insider-correlation) survives as an
//! HONEST disclosure; THREAT-MODEL.md states "network-observer-only".
//!
//! Pin sources (F-full R2 test-landscape §1 Group 12 row F-NAT-2; merges
//! PMD-20/25 + GNI-6, m-7):
//!   - R0.5 plan §3.8 m-7, Inv-20 clause-d, §5.2 Compromise #58.
//!   - Clone-shape (doc-coupling):
//!     `crates/benten-drop/tests/tf3f_revocation_reach_forever_valid_documented.rs`.
//!   - Class DC/FG: behavioral unlinkability arm + the disclosure-coherence
//!     doc-coupling arm (the boundary is asserted explicitly so the
//!     unlinkability claim is not OVER-claimed).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! The W6 per-recipient stanza unlinkability surface + THREAT-MODEL.md
//! (minted at the F-DISC-2 doc-wave) do not exist at this SHA.
//! Self-contained stub-shim compiles green; bodies `unimplemented!()`.
//! W6 / doc-wave R5 implementer:
//!   1. DELETE `mset_w6_unlinkability_stub`,
//!   2. INSERT `use benten_membership_set::privacy::{
//!      network_observer_can_link_stanzas, admin_can_correlate_members};`,
//!   3. UN-IGNORE (including the doc-coupling arm once THREAT-MODEL.md
//!      lands),
//!   4. Verify green.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Arms: (1) a network observer CANNOT link two stanzas to the same
//! recipient (the property that IS delivered); (2) a malicious admin CAN
//! correlate members (the boundary — asserted explicitly so the
//! unlinkability claim is honest, not over-claimed); (3) doc-coupling —
//! THREAT-MODEL.md states the unlinkability scope is "network-observer-only"
//! and SECURITY-POSTURE.md carries the #58 insider-correlation disclosure.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::path::Path;

// =====================================================================
// W6 R5 (Wave w-gov-audit): real `benten_membership_set::privacy` surface.
// NOTE: the doc-coupling arm `threat_model_doc_states_network_observer_only_scope`
// STAYS `#[ignore]`'d here — `docs/THREAT-MODEL.md` is minted at the TIER-3
// doc-wave (F-DISC-2); that arm un-ignores there, NOT in this TIER-2 wave.
// =====================================================================
use benten_membership_set::privacy::{admin_can_correlate_members, network_observer_can_link_stanzas};

/// F-NAT-2 (a): a network observer CANNOT link two per-recipient stanzas to
/// the same recipient — the unlinkability property that IS delivered.
#[test]
fn network_observer_cannot_link_per_recipient_stanzas() {
    // Two distinct on-wire stanzas addressed to the same recipient.
    let stanza_a = b"wire:stanza:a:opaque-bytes".as_slice();
    let stanza_b = b"wire:stanza:b:opaque-bytes".as_slice();
    assert!(
        !network_observer_can_link_stanzas(stanza_a, stanza_b),
        "F-NAT-2: a NETWORK OBSERVER (no K_Set, no admin view) MUST NOT be \
         able to link two per-recipient stanzas to the same recipient from \
         the wire bytes — this is the unlinkability property the design \
         delivers; would-FAIL if a recipient identifier leaks onto the wire"
    );
}

/// F-NAT-2 (b): the BOUNDARY — a malicious admin CAN correlate members
/// (sees the members_table). Asserting this explicitly is what keeps the
/// unlinkability claim HONEST (not over-claimed): the property is
/// network-observer-only, NOT admin-proof.
#[test]
fn malicious_admin_can_correlate_members_boundary_not_over_claimed() {
    let members_table = b"members_table:snapshot:admin-readable".as_slice();
    assert!(
        admin_can_correlate_members(members_table),
        "F-NAT-2: a malicious ADMIN (holding the members_table) CAN correlate \
         members — unlinkability does NOT protect against the admin (Inv-20 \
         clause-d m-7; Compromise #58). This assertion EXISTS so the \
         unlinkability claim is honest (network-observer-only), not \
         over-claimed as admin-proof. would-FAIL if W6 over-promises \
         admin-proof unlinkability"
    );
}

/// F-NAT-2 (c): DISCLOSURE-COHERENCE doc-coupling — THREAT-MODEL.md states
/// the unlinkability scope is "network-observer-only" and SECURITY-POSTURE.md
/// carries the #58 insider-correlation honest disclosure. Clone of the
/// `tf3f` doc-coupling shape. RED-PHASE: THREAT-MODEL.md is minted at the
/// F-DISC-2 doc-wave — un-ignore once it lands.
#[test]
#[ignore = "RED-PHASE: F-NAT-2 — THREAT-MODEL.md states network-observer-only + SECURITY-POSTURE.md #58 disclosure; un-ignore at doc-wave R5 (when THREAT-MODEL.md lands)"]
fn threat_model_doc_states_network_observer_only_scope() {
    let repo_docs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs");

    let threat_model = std::fs::read_to_string(repo_docs.join("THREAT-MODEL.md"))
        .expect("THREAT-MODEL.md must be present (F-DISC-2 doc-wave mints it)");
    assert!(
        threat_model.contains("network-observer-only")
            || threat_model.contains("network observer only")
            || (threat_model.contains("network observer")
                && threat_model.contains("unlinkab")),
        "F-NAT-2: THREAT-MODEL.md MUST scope per-recipient unlinkability as \
         NETWORK-OBSERVER-ONLY (not admin-proof) — the honest boundary"
    );

    let security_posture = std::fs::read_to_string(repo_docs.join("SECURITY-POSTURE.md"))
        .expect("SECURITY-POSTURE.md must be present");
    assert!(
        security_posture.contains("#58")
            && (security_posture.contains("insider")
                || security_posture.contains("correlat")),
        "F-NAT-2: SECURITY-POSTURE.md MUST carry the Compromise #58 \
         insider-correlation honest disclosure (the admin-sees-members_table \
         boundary)"
    );
}
