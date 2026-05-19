//! ADDL Phase-4-Meta-Core R3-B5 / TF-8 (benten-caps lane) — §4.28
//! private-namespace cross-plugin delegation policy SUBSTANTIVE arm
//! + §3.6e stranded-pin retarget.
//!
//! ## RED-PHASE — un-ignore at G-CORE-8
//!
//! §4.28: the stranded pins
//! `private_namespace_cross_plugin_delegation_denied.rs` +
//! `private_namespace_scope_prefix_canonicalization.rs` cite a
//! `private_namespace_policy::reject_cross_plugin` symbol that does
//! NOT exist at HEAD — the surface shipped under different names
//! (`benten_caps::validate_chain_with_manifest_envelope` +
//! `benten_caps::is_private_namespace_cap` +
//! `benten_caps::private_namespace_scope_admits_actor`). §4.28 / §3.6e
//! require these stranded pins be RE-TARGETED to the actually-shipped
//! surface (reviewer verifies landing-status, not just spec-pin
//! presence) at G-CORE-8 — NOT left citing a phantom symbol.
//!
//! This file is the retargeted SUBSTANTIVE arm written against the
//! shipped public surface. It is RED-phase because the §4.28
//! disposition (retarget-or-fold) lands at G-CORE-8; on un-ignore the
//! G-CORE-8 implementer either keeps this retargeted arm + DELETES the
//! two phantom-symbol stranded pins, or folds their case into the
//! `manifest_envelope_chain_validation` family per §4.28.
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL):
//!   - pim-2-amendment (§3.6b sub-rule-4): exercises the SPECIFIC
//!     cross-plugin private-namespace refusal step (production
//!     `validate_chain_with_manifest_envelope` call-site, observable
//!     `ChainValidationOutcome` reject, would-FAIL if private-namespace
//!     scopes were delegable cross-plugin).
//!   - pim-12 (§3.6e): RED-PHASE staged-pin + the explicit
//!     stranded-phantom-symbol retarget obligation named above.
//!   - pim-18 (§3.6f): substantive arm against the shipped surface,
//!     NOT the phantom `private_namespace_policy::reject_cross_plugin`.
//!   - §3.13: no shared process-scoped static (discharged structurally).
//!
//! Pins: G-CORE-8 · C8 · couples §5.5 manifest-envelope-chain-validation.
//! R2 map: TF-8 §4.28 private-namespace cross-plugin substantive arm.

use benten_caps::is_private_namespace_cap;

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.28 private-namespace \
            cross-plugin delegation substantive arm — retargets the \
            stranded private_namespace_policy::reject_cross_plugin \
            phantom-symbol pins per §3.6e)"]
fn private_namespace_cap_is_recognized_by_shipped_surface() {
    // The SHIPPED classifier (`benten_caps::is_private_namespace_cap`)
    // is the canonical recognizer the §4.28 stranded pins must
    // retarget to (they currently cite the phantom
    // `private_namespace_policy::reject_cross_plugin`).
    //
    // A `private:<did>:*` scope IS a private-namespace cap.
    assert!(
        is_private_namespace_cap("private:did:key:z6MkPluginA:*"),
        "shipped surface: a private:<did>:* scope must be classified \
         a private-namespace cap (retarget anchor for §4.28)"
    );
    // A non-private store scope is NOT.
    assert!(
        !is_private_namespace_cap("store:notes:read"),
        "shipped surface: a non-private scope must NOT be classified \
         private-namespace"
    );

    // The RED contract: G-CORE-8 must (per §4.28) wire the FULL
    // substantive cross-plugin-delegation refusal arm against
    // `validate_chain_with_manifest_envelope` (user → plugin-A →
    // plugin-B delegating `private:plugin-A:*`) AND delete/fold the
    // two phantom-symbol stranded pins. Until that disposition lands,
    // this staged-pin holds the retarget obligation open.
    panic!(
        "§4.28 substantive arm undelivered: the cross-plugin \
         private-namespace refusal arm against the SHIPPED \
         `validate_chain_with_manifest_envelope` surface (+ the \
         stranded-phantom-symbol retarget/fold disposition) lands at \
         G-CORE-8. The shipped classifier is exercised above as the \
         retarget anchor; the full chain-validation arm + the \
         stranded-pin deletion are the G-CORE-8 wave obligation \
         (§3.6e — reviewer verifies landing-status)."
    );
}
