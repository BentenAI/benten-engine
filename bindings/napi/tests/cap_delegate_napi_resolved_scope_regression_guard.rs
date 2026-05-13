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
//! ## RED-PHASE pin shape
//!
//! At HEAD the napi binding + the underlying engine seam DO NOT EXIST.
//! This pin compiles as a compile-time witness for the FUTURE seam
//! shape — the test body is the canary that fires when the implementer
//! at G24-D un-ignores + wires the production arm without honoring the
//! resolving-seam discipline.
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

/// RED-PHASE: G27-A class-of-bug audit for future delegate surface.
///
/// G24-D introduces `crates/benten-caps/src/plugin_delegation.rs`
/// + (per G24-D scope) a napi `delegateCapability` binding. The pin
/// here is the canary the implementer un-ignores AT G24-D wave time
/// to confirm the resolving-seam discipline is inherited.
#[test]
#[ignore = "RED-PHASE: G27-A — un-ignore when G24-D ships the napi delegate surface AND the resolving-seam discipline is verified end-to-end"]
fn future_delegate_napi_binding_resolves_source_scope_not_cid() {
    // RED-PHASE: this body fires at G24-D wave-time to:
    // 1. Issue a source grant (`store:notes:write`) to source-plugin DID.
    // 2. Invoke the future `Engine::issue_delegation(source_grant_cid, audience_did, ...)` seam.
    // 3. Verify the delegation Node persisted carries
    //    `source_scope = "store:notes:write"` (the resolved source scope),
    //    NOT `source_scope = "<source_grant_cid base32>"`.
    // 4. Drive an audience-side write at `store:notes:write` and
    //    assert OK edge (delegation observably grants the audience).
    //
    // At HEAD: the seam doesn't exist; the body MUST surface this
    // explicitly so the un-ignore happens at the correct wave with
    // the correct production-arm wire-up.
    panic!(
        "RED-PHASE: G27-A — future delegate seam awaits G24-D; un-ignore at that wave with resolving-seam end-to-end pin"
    );
}

/// Compile-time witness: the Engine surface is reachable from the napi
/// test crate. When G24-D lands `Engine::issue_delegation` (or whatever
/// the seam name is), this witness extends to accept that seam.
#[test]
fn future_delegate_seam_engine_reachable_compile_witness() {
    fn _accepts_engine(_engine: &Engine) {}
    let _: fn(&Engine) = _accepts_engine;
}
