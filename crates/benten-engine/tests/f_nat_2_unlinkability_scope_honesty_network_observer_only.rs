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
//! # LANDED (RED-PHASE discharged; F-16 2026-07-03)
//!
//! The W6 per-recipient stanza unlinkability surface
//! (`benten_membership_set::privacy`) AND `docs/THREAT-MODEL.md` (minted at the
//! F-DISC-2 doc-wave) BOTH exist at HEAD. This file wires the real surface (see
//! the `use` below) and ALL THREE arms — including the doc-coupling arm — are
//! LIVE (no `#[ignore]`, no stub-shim). The prior RED-PHASE stub-shim +
//! un-ignore checklist is fully discharged.
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
// The doc-coupling arm `threat_model_doc_states_network_observer_only_scope`
// is LIVE (un-ignored; F-16 2026-07-03) — `docs/THREAT-MODEL.md` is present at
// HEAD and states the network-observer-only scope, so the arm reads it for real.
// =====================================================================
use benten_membership_set::privacy::{
    admin_can_correlate_members, network_observer_can_link_stanzas,
};

/// F-NAT-2 (a): a network observer CANNOT link two BLINDED per-recipient
/// stanzas to the same recipient — the unlinkability property that IS
/// delivered. The predicate is INPUT-SENSITIVE (F-06): the positive arm proves
/// blinded stanzas are unlinkable AND the negative-control arm proves the
/// predicate is NOT a constant-false (a stanza that LEAKED a recipient id in
/// the clear WOULD be linkable — the regression the blinding forbids).
#[test]
fn network_observer_cannot_link_per_recipient_stanzas() {
    // Two distinct correctly-BLINDED on-wire stanzas addressed to the same
    // recipient — opaque bytes carrying NO cleartext recipient marker.
    let stanza_a = b"wire:stanza:a:opaque-bytes".as_slice();
    let stanza_b = b"wire:stanza:b:opaque-bytes".as_slice();
    assert!(
        !network_observer_can_link_stanzas(stanza_a, stanza_b),
        "F-NAT-2: a NETWORK OBSERVER (no K_Set, no admin view) MUST NOT be \
         able to link two correctly-blinded per-recipient stanzas to the same \
         recipient from the wire bytes — this is the unlinkability property \
         the design delivers"
    );

    // NEGATIVE CONTROL (F-06 input-sensitivity proof): if a recipient
    // identifier ever LEAKED onto the wire in the clear, two stanzas exposing
    // the SAME identifier WOULD be linkable. This arm exercises the predicate's
    // dependence on the wire bytes — it is NOT a constant-false. It also
    // demonstrates the would-FAIL: were the production wire form to leak a
    // recipient marker, arm (a) above would flip.
    let leaked_a = b"wire|stanza|recipient=did:key:zAlice|cipher-1".as_slice();
    let leaked_b = b"wire|stanza|recipient=did:key:zAlice|cipher-2".as_slice();
    assert!(
        network_observer_can_link_stanzas(leaked_a, leaked_b),
        "F-NAT-2 negative control: two stanzas that LEAK the SAME recipient \
         identifier in the clear ARE linkable — proving the unlinkability \
         predicate genuinely inspects the wire bytes (not a hard-coded false)"
    );
    // And two LEAKED stanzas exposing DIFFERENT recipient ids are NOT linkable
    // (the observer sees distinct markers) — the predicate keys on identity,
    // not merely on the presence of a marker.
    let leaked_c = b"wire|stanza|recipient=did:key:zBob|cipher-3".as_slice();
    assert!(
        !network_observer_can_link_stanzas(leaked_a, leaked_c),
        "F-NAT-2 negative control: leaked stanzas to DIFFERENT recipients are \
         not linked (the predicate keys on the recipient identity, not the \
         mere presence of a marker)"
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
/// `tf3f` doc-coupling shape. LIVE (F-16): THREAT-MODEL.md is present at HEAD,
/// so this arm reads it for real (no `#[ignore]`).
#[test]
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
            || (threat_model.contains("network observer") && threat_model.contains("unlinkab")),
        "F-NAT-2: THREAT-MODEL.md MUST scope per-recipient unlinkability as \
         NETWORK-OBSERVER-ONLY (not admin-proof) — the honest boundary"
    );

    let security_posture = std::fs::read_to_string(repo_docs.join("SECURITY-POSTURE.md"))
        .expect("SECURITY-POSTURE.md must be present");
    assert!(
        security_posture.contains("#58")
            && (security_posture.contains("insider") || security_posture.contains("correlat")),
        "F-NAT-2: SECURITY-POSTURE.md MUST carry the Compromise #58 \
         insider-correlation honest disclosure (the admin-sees-members_table \
         boundary)"
    );
}
