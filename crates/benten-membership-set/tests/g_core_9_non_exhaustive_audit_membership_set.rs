//! phase-4-meta-core R6-R2 fix wave (F-05 MAJOR-residual closure) —
//! `benten-membership-set` `#[non_exhaustive]` audit arm-coverage pins.
//!
//! Companion to `crates/benten-drop/tests/g_core_9_non_exhaustive_audit_drop.rs`
//! and `crates/benten-engine/tests/g_core_9_non_exhaustive_audit.rs`. The
//! membership-set audit lives here because benten-engine does NOT depend on
//! benten-membership-set (deliberate dependency-graph stratification — the
//! membership-set keying primitive is a leaf-ish addition; crypto-suite / sync
//! NEVER depend on it, per the F-CRATE-2 no-reverse-edge fence).
//!
//! ## What this pins
//!
//! Every membership-set PUBLIC error enum that V1-FROZEN-INTERFACE.md §11
//! requires to carry `#[non_exhaustive]` (so a future failure mode lands
//! ADDITIVELY without a breaking SemVer bump) is audited here via an
//! explicit-arms `match` + a `_` catch-all. The `_` arm is reachable ONLY
//! while the `#[non_exhaustive]` attribute is present; REMOVING the attribute
//! turns the catch-all into a `unreachable_patterns` warning that the
//! workspace `-D warnings` lint promotes to a build break (§11
//! HALT-AND-SURFACE). This mirrors the drop crate's mechanism EXACTLY.
//!
//! ## Enums covered (all `pub`, all `#[non_exhaustive]`)
//!
//! - `error::MembershipSetError`
//! - `audit::AuditChainError`
//! - `keying_kv::KvError`
//! - `federation::AcquisitionError`
//! - `federation::FederationError`
//! - `kind::KindDispatchError` (R6-R3 fix-b — was MISSING `#[non_exhaustive]`
//!   while its 5 sibling error enums carried it)
//!
//! Carve-out preserved: `kind::MembershipSetKind` is INTENTIONALLY
//! exhaustive-by-design (the EXACTLY-3 frozen-cardinality enum — §15.c
//! HALT-AND-SURFACE wants every `match` over it to be exhaustive, so it
//! deliberately has NO `#[non_exhaustive]`). NOT covered here.

#[test]
fn membership_set_error_audit_arm_coverage_non_exhaustive() {
    use benten_membership_set::error::MembershipSetError;
    fn audit(e: &MembershipSetError) -> &'static str {
        match e {
            MembershipSetError::AdminCardinality => "AdminCardinality",
            MembershipSetError::MemberCardinality => "MemberCardinality",
            MembershipSetError::MemberRefKindMismatch => "MemberRefKindMismatch",
            MembershipSetError::WireCostCeilingExceeded => "WireCostCeilingExceeded",
            MembershipSetError::AuthorityMissingPubkey => "AuthorityMissingPubkey",
            MembershipSetError::ReserveTypedReject => "ReserveTypedReject",
            MembershipSetError::RoleStaleAtVerify => "RoleStaleAtVerify",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(&MembershipSetError::AdminCardinality),
        "AdminCardinality"
    );
}

#[test]
fn audit_chain_error_audit_arm_coverage_non_exhaustive() {
    // `AuditChainError` is `Copy`, so take by value (clippy
    // `trivially_copy_pass_by_ref`); the `_` arm + `#[non_exhaustive]`
    // mechanism is identical.
    use benten_membership_set::audit::AuditChainError;
    fn audit(e: AuditChainError) -> &'static str {
        match e {
            AuditChainError::TamperDetectedLinkageBroken { .. } => "TamperDetectedLinkageBroken",
            AuditChainError::NonMonotonicAppend => "NonMonotonicAppend",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(AuditChainError::NonMonotonicAppend),
        "NonMonotonicAppend"
    );
}

#[test]
fn kv_error_audit_arm_coverage_non_exhaustive() {
    // `KvError` is `Copy`, so take by value (clippy
    // `trivially_copy_pass_by_ref`).
    use benten_membership_set::keying_kv::KvError;
    fn audit(e: KvError) -> &'static str {
        match e {
            KvError::TargetNotImmutable => "TargetNotImmutable",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(KvError::TargetNotImmutable), "TargetNotImmutable");
}

#[test]
fn acquisition_error_audit_arm_coverage_non_exhaustive() {
    use benten_membership_set::federation::AcquisitionError;
    fn audit(e: &AcquisitionError) -> &'static str {
        match e {
            AcquisitionError::RecursionDepthExceeded => "RecursionDepthExceeded",
            AcquisitionError::CycleDetected => "CycleDetected",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(&AcquisitionError::CycleDetected), "CycleDetected");
}

#[test]
fn federation_error_audit_arm_coverage_non_exhaustive() {
    use benten_membership_set::federation::FederationError;
    fn audit(e: &FederationError) -> &'static str {
        match e {
            FederationError::FederationReserved => "FederationReserved",
            FederationError::ModelAUnavailable => "ModelAUnavailable",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(&FederationError::FederationReserved),
        "FederationReserved"
    );
}

#[test]
fn kind_dispatch_error_audit_arm_coverage_non_exhaustive() {
    // R6-R3 fix-b (F-01): `KindDispatchError` now carries `#[non_exhaustive]`,
    // mirroring its 5 sibling error enums. `KindDispatchError` is `Copy`, so
    // take by value (clippy `trivially_copy_pass_by_ref`). The `_` catch-all
    // arm is reachable ONLY while the attribute is present; removing it turns
    // the catch-all into an `unreachable_patterns` build break (§11
    // HALT-AND-SURFACE).
    use benten_membership_set::kind::KindDispatchError;
    fn audit(e: KindDispatchError) -> &'static str {
        match e {
            KindDispatchError::ReserveTypedReject => "ReserveTypedReject",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(KindDispatchError::ReserveTypedReject),
        "ReserveTypedReject"
    );
}
