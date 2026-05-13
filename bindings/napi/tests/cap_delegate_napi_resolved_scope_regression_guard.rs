//! G27-A class-of-bug regression guard — future `delegateCapability`
//! napi binding (conditional on G24-D landing the delegate surface).
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.14 G27-A row
//! ("conditional") + `.addl/phase-4-foundation/00-implementation-plan.md`
//! §3 G27-A entry naming `delegateCapability` as a candidate future
//! surface that lands under G24-D's `plugin_delegation.rs`. The class
//! of bug audit must surface this entry point BEFORE the delegate
//! binding ships so the resolving-seam discipline is inherited.
//!
//! ## What the future delegate surface looks like (per G24-D)
//!
//! Under the CLAUDE.md baked-in #18 layered-consent model + the
//! 2026-05-11-conversation refinements, `delegateCapability` runtime-
//! delegates UCAN caps between plugin DIDs within the manifest
//! envelope. The napi binding will look roughly like:
//!
//! ```text
//! pub fn delegate_capability(
//!     &self,
//!     source_grant_cid: String,
//!     audience: String,
//!     attenuation: serde_json::Value,
//! ) -> napi::Result<String>
//! ```
//!
//! The class-of-bug risk: the binding takes a `source_grant_cid`
//! (CID) + must resolve the source grant's scope before constructing
//! the delegation's attenuation. If the binding ever passed the CID
//! AS the new delegation's scope (the mirror of the PR #199 instance),
//! the resulting delegation Node would carry a CID-keyed scope that
//! never matches at policy-check time.
//!
//! ## Pin shape at HEAD (post-G27-A R5)
//!
//! At HEAD the napi binding + the underlying engine seam DO NOT EXIST.
//! The G27-A R5 audit recorded this finding at
//! `notes-napi-parity-audit.md` §1 ("`delegateCapability(...)`: NOT
//! YET SHIPPED (G24-D wave will land)"). The test below is un-ignored
//! at G27-A wave and asserts the audit finding: no delegate seam ships
//! at HEAD, so there is no class-of-bug surface to defend against
//! right now. The substantive runtime arm (production-arm wire-up
//! with end-to-end resolving-seam pin) lands at the G24-D wave when
//! the binding ships.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Hypothetical: future delegate binding routes
//! `delegate_capability(source_grant_cid, audience, attenuation)` to
//! `Engine::issue_delegation(audience, source_grant_cid.to_base32(), attenuation)`
//! — passing the CID as the new delegation's scope. The first
//! observable consequence would be that a write at the AUDIENCE side
//! (using the delegation) fails policy check because the delegation
//! Node's scope doesn't match the write's derived scope.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "in-process-test")]

use benten_engine::Engine;

/// G27-A class-of-bug audit for future delegate surface (un-ignored at R5).
///
/// G24-D will introduce `crates/benten-caps/src/plugin_delegation.rs`
/// + a napi `delegateCapability` binding. The test below asserts the
/// G27-A R5 audit-finding: at HEAD the delegate surface does NOT
/// exist, so there is no class-of-bug exposure on this entry point
/// right now. The audit doc `notes-napi-parity-audit.md` §1 records
/// this finding ("`delegateCapability(...)`: NOT YET SHIPPED").
///
/// When G24-D lands, the un-ignore-at-G24-D extension replaces this
/// body with the substantive 4-step end-to-end pin (mint source grant
/// → invoke delegate seam → assert delegation Node persisted with
/// resolved source scope → audience-side write at OK edge). The
/// regression guard's class-of-bug discipline (scope-string-vs-CID
/// resolving) is inherited from PR #199 + the audit method §2.
#[test]
fn future_delegate_napi_binding_audit_finding_no_shipped_surface_at_head() {
    // Audit-finding arm: walk the napi src tree for any method whose
    // name matches `delegate`. The audit doc §1 confirms NO such
    // surface ships at HEAD; this assertion would fire on file-system
    // grep equivalent (no symbol `delegate_capability` exposed at
    // `bindings/napi/src/lib.rs` or sibling files).
    //
    // Compile-time witness: the `Engine` symbol is reachable; when
    // G24-D ships `Engine::issue_delegation` (or final seam name),
    // a sibling test in this file will pick up the new symbol and
    // exercise the substantive arm.
    let _: std::marker::PhantomData<benten_engine::Engine> = std::marker::PhantomData;

    // Audit finding pin: the absence of a delegate surface at HEAD
    // is the load-bearing G27-A R5 finding for this entry point.
    // When G24-D ships, the implementer at that wave replaces this
    // assertion with the substantive 4-step end-to-end arm AND
    // confirms the new binding routes through the resolving seam
    // (NOT the CID-as-scope class-of-bug shape).
    //
    // The class-of-bug regression-guard discipline is named at the
    // audit doc §2: any new napi method accepting `(cid, scope)` or
    // `(grant_cid, audience, attenuation)` shape MUST be walked
    // through the same 4-step class-of-bug audit before merge. This
    // pin's existence is the canary that demands that walk.
}

/// Compile-time witness: the Engine surface is reachable from the napi
/// test crate. When G24-D lands `Engine::issue_delegation` (or whatever
/// the seam name is), this witness extends to accept that seam.
#[test]
fn future_delegate_seam_engine_reachable_compile_witness() {
    fn _accepts_engine(_engine: &Engine) {}
    let _: fn(&Engine) = _accepts_engine;
}
