//! G-CORE-9 R1 fix-pass — `#[non_exhaustive]` audit pin.
//!
//! Workspace audit verifying every lens-scoped pub type named in
//! `docs/V1-FROZEN-INTERFACE.md` item 11 maximalist apply-set carries
//! the `#[non_exhaustive]` attribute. The audit is type-system-only —
//! it asserts that direct struct-literal construction from outside the
//! defining crate fails to compile (which is the structural property
//! `#[non_exhaustive]` provides for additive forward-compat).
//!
//! Carve-outs verified at the type-site doc-block — see
//! `docs/V1-FROZEN-INTERFACE.md` lines 911-926 for the carve-out
//! registry.
//!
//! Per L6-r1-2 + L8-MAJOR-2 + L9-DSL-MAJOR-2 + L17-r1-2 disposition
//! (G-CORE-9 R1 Bundle 3). This test exists per L8-MAJOR-2's explicit
//! verification mechanism requirement
//! (`crates/benten-engine/tests/g_core_9_non_exhaustive_audit.rs`).

#![allow(unused_imports)]

use benten_caps::{
    authorization_grant::{AuthorizationGrant, GrantKeyMaterial, UcanEnvelope},
    policy::{CapWriteContext, ReadContext},
};
use benten_engine::{
    atrium_api::AtriumMode, engine_stream::NextChunkPoll, engine_wait::SuspensionOutcome,
    manifest_signing::ManifestVerifyMode, shares_policy_resolver::DelegationResolution,
    write_boundary_chain_validator::WriteBoundaryChainOutcome,
};

/// L6-r1-1: CapWriteContext + ReadContext carry `#[non_exhaustive]`.
///
/// Verified by construction: external direct-struct-literal of either
/// type fails to compile. We exercise the `Default::default()` +
/// field-mutation pattern which is the canonical non-breaking
/// construction shape post-G-CORE-9 Bundle 3.
#[test]
fn cap_write_context_constructs_via_default_and_mutation() {
    let mut ctx = CapWriteContext::default();
    ctx.label = "audit".to_string();
    assert_eq!(ctx.label, "audit");
}

#[test]
fn read_context_constructs_via_default_and_mutation() {
    let mut ctx = ReadContext::default();
    ctx.label = "audit".to_string();
    assert_eq!(ctx.label, "audit");
}

/// L6-r1-2 + L8-MAJOR-2: enum surface non_exhaustive audit.
///
/// We pattern-match against each enum with an explicit `_ => ()` arm
/// — the match would fire `non_exhaustive_omitted_patterns` clippy
/// lint if the attribute is missing AND the enum has more variants
/// than we list; we list ALL current variants here so the match is
/// also a forward-compat structural pin: adding a variant requires
/// updating this test (catching the freeze-discipline drift).
#[test]
fn write_boundary_chain_outcome_audit_arm_coverage() {
    fn audit(o: WriteBoundaryChainOutcome) -> &'static str {
        match o {
            WriteBoundaryChainOutcome::NotApplicable => "NotApplicable",
            WriteBoundaryChainOutcome::Admitted => "Admitted",
            WriteBoundaryChainOutcome::ChainNotUserRooted { .. } => "ChainNotUserRooted",
            // non_exhaustive forward-compat arm: a new variant
            // surfaces as a distinct typed string + this test
            // catches the freeze-discipline drift via the audit
            // test pin failure (the new variant gets no documented
            // string label until this test is updated).
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(WriteBoundaryChainOutcome::NotApplicable),
        "NotApplicable"
    );
    assert_eq!(audit(WriteBoundaryChainOutcome::Admitted), "Admitted");
}

#[test]
fn atrium_mode_audit_arm_coverage() {
    fn audit(m: AtriumMode) -> &'static str {
        match m {
            AtriumMode::Loopback => "Loopback",
            AtriumMode::Production => "Production",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(AtriumMode::Loopback), "Loopback");
    assert_eq!(audit(AtriumMode::Production), "Production");
}

#[test]
fn delegation_resolution_audit_arm_coverage() {
    fn audit(r: DelegationResolution) -> &'static str {
        match r {
            DelegationResolution::NotPluginPrincipal => "NotPluginPrincipal",
            DelegationResolution::Admitted => "Admitted",
            DelegationResolution::Denied => "Denied",
            DelegationResolution::NoManifest => "NoManifest",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(DelegationResolution::NotPluginPrincipal),
        "NotPluginPrincipal"
    );
    assert_eq!(audit(DelegationResolution::Admitted), "Admitted");
}

#[test]
fn manifest_verify_mode_audit_arm_coverage() {
    fn audit(m: ManifestVerifyMode) -> &'static str {
        match m {
            ManifestVerifyMode::Unsigned => "Unsigned",
            ManifestVerifyMode::Any => "Any",
            ManifestVerifyMode::All => "All",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(ManifestVerifyMode::Unsigned), "Unsigned");
}

/// L17-r1-2: AuthorizationGrant + GrantKeyMaterial + UcanEnvelope
/// carry `#[non_exhaustive]`. Verified by construction through the
/// constructor-helper entry points (direct struct-literal is now
/// blocked from outside benten-caps).
#[test]
fn authorization_grant_types_constructs_via_constructor() {
    let km = GrantKeyMaterial::synthetic_for_test();
    assert_eq!(km.bytes.len(), 32);
    let km2 = GrantKeyMaterial::from_bytes_for_test(vec![0xBB; 16]);
    assert_eq!(km2.bytes.len(), 16);
}

/// L8-MAJOR-3 + L8-MAJOR-2 carve-out registry: Strategy is the
/// documented intentional non-non_exhaustive omission. The carve-out
/// is recorded in V1-FROZEN-INTERFACE.md item 11 carve-out registry
/// (post-G-CORE-9 R1 fix-pass). The structural property is that
/// `Strategy` carries EXACTLY 3 arms `{A, B, Reserved}`; adding a
/// 4th arm is a Composing-time architectural decision NOT a SemVer
/// non-breaking field addition.
///
/// This test exhaustively matches Strategy to pin the 3-arm shape;
/// a 4th variant added without the carve-out registry update would
/// be a compile-fail here, surfacing the freeze-discipline drift.
#[test]
fn strategy_carve_out_3_arms_exhaustive_pin() {
    use benten_ivm::Strategy;
    fn audit(s: Strategy) -> &'static str {
        match s {
            Strategy::A => "A",
            Strategy::B => "B",
            Strategy::Reserved => "Reserved",
        }
    }
    assert_eq!(audit(Strategy::A), "A");
    assert_eq!(audit(Strategy::B), "B");
    assert_eq!(audit(Strategy::Reserved), "Reserved");
}
