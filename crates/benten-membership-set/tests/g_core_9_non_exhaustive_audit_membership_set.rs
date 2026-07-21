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
//! Every membership-set PUBLIC **semantic** enum that V1-FROZEN-INTERFACE.md §11
//! requires to carry `#[non_exhaustive]` (so a future variant lands ADDITIVELY
//! without a breaking SemVer bump) is audited here via an explicit-arms `match`
//! + a `_` catch-all. Because this is an INTEGRATION test crate — a DIFFERENT
//! crate from the `benten-membership-set` library — these enums are cross-crate
//! non_exhaustive enums here: the `_` arm is reachable ONLY while the
//! `#[non_exhaustive]` attribute is present; REMOVING the attribute makes the
//! explicit arms exhaustive, turning the catch-all into an `unreachable_patterns`
//! warning that the workspace `-D warnings` lint promotes to a build break (§11
//! HALT-AND-SURFACE). This mirrors the drop crate's mechanism EXACTLY.
//!
//! ## The completeness guard (R9 F-07 — "catch a FUTURE missed enum")
//!
//! [`semantic_pub_enum_registry_is_complete`] is the census pin: it enumerates
//! EVERY semantic `pub enum` in the crate into exactly one of two buckets —
//! `#[non_exhaustive]`-REQUIRED or an EXPLICIT exhaustive-by-design CARVE-OUT —
//! and asserts the census total against the crate's live `pub enum` count. A
//! future `pub enum` added to the crate without a registry entry trips this
//! census (the total no longer matches), forcing the author to classify it —
//! closing the R9 F-07 gap where `benten-membership-set` was added AFTER the
//! G-CORE-9 §11 sweep and its six semantic enums were silently missed.
//!
//! ## Enums covered — `#[non_exhaustive]`-REQUIRED (12)
//!
//! Error enums (6, landed at earlier waves):
//! - `error::MembershipSetError`
//! - `audit::AuditChainError`
//! - `keying_kv::KvError`
//! - `federation::AcquisitionError`
//! - `federation::FederationError`
//! - `kind::KindDispatchError` (R6-R3 fix-b — was MISSING `#[non_exhaustive]`
//!   while its 5 sibling error enums carried it)
//!
//! Semantic decision enums (6, landed at **R9 F-07** — the crate was added
//! after the G-CORE-9 §11 sweep so these were missed):
//! - `audit::AdminOp`
//! - `audit::AuditAccessGradation`
//! - `audit::AuditReadDecision`
//! - `audit::RequesterRole`
//! - `federation::FederationModel`
//! - `governance::GovernanceTier`
//!
//! ## Carve-outs — EXHAUSTIVE-BY-DESIGN (5; deliberately NO `#[non_exhaustive]`)
//!
//! - `kind::MembershipSetKind` — the EXACTLY-3 frozen-cardinality Kind
//!   (`#[repr(u8)]`, wire-keying-load-bearing ordinals); §15.c HALT-AND-SURFACE
//!   wants every `match` over it exhaustive (a 4th arm is a compile error).
//! - `kind::RequestedReserveKind` — the reserve selector; V1-FROZEN §16
//!   carve-out (the two reserves typed-reject at v1-beta; a new reserve is a
//!   deliberate architectural decision, not a silent additive variant).
//! - `role::RoleId` — the 5-value RBAC ordinal (`#[repr(u8)]`,
//!   keying-AAD-bound, golden-vector-pinned); the ordinal byte is the canonical
//!   wire form, so exhaustiveness is the structural pin.
//! - `member::MemberRef` — the KEYING/FEDERATION int-tagged member reference
//!   (`#[repr(u8)]`, `#[serde(into = "u8")]`); the discriminant is the
//!   AAD-keying-bound wire form.
//! - `keying_kv::CidTarget` — the `K(V)` KDF-input dispatch enum; its one
//!   `derive_kv` match is an EXHAUSTIVE fail-closed dispatch (the
//!   `MutableAnchor` arm is the Inv-19 rejection), so exhaustiveness IS the
//!   structural pin (a new target Kind must be classified permitted/rejected at
//!   that site, never fall through additively).

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

// ---------------------------------------------------------------------------
// R9 F-07 — the SIX semantic decision enums (missed by the G-CORE-9 §11 sweep
// because this crate was added afterward). Each is now `#[non_exhaustive]`; the
// arm-coverage `match` + `_` catch-all pins the attribute (removing it makes the
// `_` unreachable → `-D warnings` build break, §11 HALT-AND-SURFACE).
// ---------------------------------------------------------------------------

#[test]
fn admin_op_audit_arm_coverage_non_exhaustive() {
    // `AdminOp` is `Copy` → take by value (clippy `trivially_copy_pass_by_ref`).
    use benten_membership_set::audit::AdminOp;
    fn audit(op: AdminOp) -> &'static str {
        match op {
            AdminOp::AdmitMember => "AdmitMember",
            AdminOp::KickMember => "KickMember",
            AdminOp::RotateKey => "RotateKey",
            AdminOp::PromoteRole => "PromoteRole",
            AdminOp::GovernanceChange => "GovernanceChange",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(AdminOp::AdmitMember), "AdmitMember");
    assert_eq!(audit(AdminOp::GovernanceChange), "GovernanceChange");
}

#[test]
fn audit_access_gradation_audit_arm_coverage_non_exhaustive() {
    // `AuditAccessGradation` is `Copy` → take by value.
    use benten_membership_set::audit::AuditAccessGradation;
    fn audit(g: AuditAccessGradation) -> &'static str {
        match g {
            AuditAccessGradation::AdminOnly => "AdminOnly",
            AuditAccessGradation::PublicAllMembers => "PublicAllMembers",
            AuditAccessGradation::MemberOnly => "MemberOnly",
            AuditAccessGradation::Threshold => "Threshold",
            AuditAccessGradation::TimeLocked => "TimeLocked",
            AuditAccessGradation::Anonymized => "Anonymized",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(AuditAccessGradation::AdminOnly), "AdminOnly");
    assert_eq!(audit(AuditAccessGradation::Anonymized), "Anonymized");
}

#[test]
fn audit_read_decision_audit_arm_coverage_non_exhaustive() {
    // `AuditReadDecision` is `Copy` → take by value.
    use benten_membership_set::audit::AuditReadDecision;
    fn audit(d: AuditReadDecision) -> &'static str {
        match d {
            AuditReadDecision::Admit => "Admit",
            AuditReadDecision::Deny => "Deny",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(AuditReadDecision::Admit), "Admit");
    assert_eq!(audit(AuditReadDecision::Deny), "Deny");
}

#[test]
fn requester_role_audit_arm_coverage_non_exhaustive() {
    // `RequesterRole` is `Copy` → take by value.
    use benten_membership_set::audit::RequesterRole;
    fn audit(r: RequesterRole) -> &'static str {
        match r {
            RequesterRole::Admin => "Admin",
            RequesterRole::Member => "Member",
            RequesterRole::NonMember => "NonMember",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(RequesterRole::Admin), "Admin");
    assert_eq!(audit(RequesterRole::NonMember), "NonMember");
}

#[test]
fn federation_model_audit_arm_coverage_non_exhaustive() {
    // `FederationModel` is `Copy` → take by value.
    use benten_membership_set::federation::FederationModel;
    fn audit(m: FederationModel) -> &'static str {
        match m {
            FederationModel::ModelB => "ModelB",
            FederationModel::ModelA => "ModelA",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(FederationModel::ModelB), "ModelB");
    assert_eq!(audit(FederationModel::ModelA), "ModelA");
}

#[test]
fn governance_tier_audit_arm_coverage_non_exhaustive() {
    // `GovernanceTier` is `Copy` → take by value.
    use benten_membership_set::governance::GovernanceTier;
    fn audit(t: GovernanceTier) -> &'static str {
        match t {
            GovernanceTier::Flat => "Flat",
            GovernanceTier::Moderated => "Moderated",
            GovernanceTier::Polycentric => "Polycentric",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(GovernanceTier::Flat), "Flat");
    assert_eq!(audit(GovernanceTier::Polycentric), "Polycentric");
}

// ---------------------------------------------------------------------------
// R9 F-07 — completeness census. This is the guard that catches a FUTURE missed
// semantic pub enum (the exact failure mode that let this crate's six enums slip
// past the G-CORE-9 §11 sweep). Every semantic `pub enum` in the crate is
// classified into exactly one of two buckets, and the census total is asserted
// against the crate's live `pub enum` count. Adding a new `pub enum` without a
// registry entry breaks this census — forcing an explicit classify-or-carve-out
// decision rather than a silent miss.
// ---------------------------------------------------------------------------

#[test]
fn semantic_pub_enum_registry_is_complete() {
    // Bucket 1: `#[non_exhaustive]`-REQUIRED semantic enums. Each has a
    // dedicated arm-coverage test above pinning the attribute presence.
    const NON_EXHAUSTIVE_REQUIRED: &[&str] = &[
        // error enums
        "error::MembershipSetError",
        "audit::AuditChainError",
        "keying_kv::KvError",
        "federation::AcquisitionError",
        "federation::FederationError",
        "kind::KindDispatchError",
        // semantic decision enums (R9 F-07)
        "audit::AdminOp",
        "audit::AuditAccessGradation",
        "audit::AuditReadDecision",
        "audit::RequesterRole",
        "federation::FederationModel",
        "governance::GovernanceTier",
    ];

    // Bucket 2: EXHAUSTIVE-BY-DESIGN carve-outs. Each deliberately carries NO
    // `#[non_exhaustive]` — a documented structural reason (frozen cardinality,
    // wire-load-bearing ordinal, reserve selector, or fail-closed KDF-dispatch)
    // makes exhaustiveness the pin. A carve-out cannot be added silently: it
    // must appear HERE (part of the freeze per V1-FROZEN §16 discipline).
    const EXHAUSTIVE_BY_DESIGN_CARVE_OUTS: &[&str] = &[
        "kind::MembershipSetKind",    // EXACTLY-3 frozen-cardinality (#[repr(u8)])
        "kind::RequestedReserveKind", // reserve selector — V1-FROZEN §16 carve-out
        "role::RoleId",               // 5-value keying-AAD-bound ordinal (#[repr(u8)])
        "member::MemberRef",          // int-tagged member ref (#[repr(u8)], serde into u8)
        "keying_kv::CidTarget",       // K(V) KDF-input — exhaustive fail-closed dispatch
    ];

    // The census total is the count of `pub enum` declarations LIVE in the
    // crate's `src/` (derived structurally below — NOT a hardcoded literal), so
    // adding a new `pub enum` WITHOUT a registry entry makes `registry_total`
    // diverge from `live_count` and BREAKS this test. (F-03 R12: previously the
    // assertion compared the two registry buckets against a hardcoded `17`, which
    // is `17 == 17` — vacuous with respect to the crate's live enum count; a new
    // unregistered `pub enum` would NOT have broken it. The doc-comment above
    // claimed a live-count comparison the code did not perform; this derives it.)
    let live_count = live_pub_enum_count();
    let registry_total = NON_EXHAUSTIVE_REQUIRED.len() + EXHAUSTIVE_BY_DESIGN_CARVE_OUTS.len();

    assert_eq!(
        registry_total, live_count,
        "R9 F-07 census drift: the two registry buckets ({registry_total}) must sum to \
         the crate's LIVE `pub enum` count ({live_count}). If you added a `pub enum` to \
         benten-membership-set, classify it into NON_EXHAUSTIVE_REQUIRED (add \
         #[non_exhaustive] + an arm-coverage test) or EXHAUSTIVE_BY_DESIGN_CARVE_OUTS \
         (document the structural reason)."
    );

    // No enum appears in both buckets (a classification must be unambiguous).
    for name in NON_EXHAUSTIVE_REQUIRED {
        assert!(
            !EXHAUSTIVE_BY_DESIGN_CARVE_OUTS.contains(name),
            "R9 F-07: `{name}` is classified in BOTH buckets — a semantic enum is \
             either #[non_exhaustive] OR an exhaustive-by-design carve-out, never both"
        );
    }
}

/// Count the `pub enum` declarations LIVE in `benten-membership-set/src/`.
/// Derived structurally so the F-07 completeness census (above) breaks when a
/// new `pub enum` is added without a registry entry — matching the census
/// doc-comment's stated guarantee. Counts leading `pub enum ` at the start of a
/// (trimmed) line, ignoring `pub(crate)`/`pub(super)` and in-comment matches.
fn live_pub_enum_count() -> usize {
    let src_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut count = 0usize;
    for path in walk_rs(&src_dir) {
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        for line in text.lines() {
            let t = line.trim_start();
            if t.starts_with("pub enum ") {
                count += 1;
            }
        }
    }
    count
}

/// Recursively collect `.rs` files under `dir`.
fn walk_rs(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walk_rs(&p));
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    out
}
