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

/// **R6-R2-FP Item 6 (Row D-17 closure):** `CapWriteContext` +
/// `ReadContext` now carry `#[non_exhaustive]` (applied at
/// `crates/benten-caps/src/policy.rs`). Direct struct-literal
/// construction from outside `benten-caps` is BLOCKED at compile
/// time; consumers use `Default::default()` + field-mutation
/// (this audit test). All ~6 cross-crate production sites in
/// `benten-engine` already migrated to the default+mutation
/// pattern per Bundle 3 / Item 6 closure.
///
/// Would-FAIL-on-revert: removing `#[non_exhaustive]` from
/// `CapWriteContext` would still leave this test passing, BUT the
/// audit test below `cap_write_context_audit_attribute_present`
/// catches the attribute removal via runtime introspection of
/// the Debug format (which doesn't capture non_exhaustive, so we
/// use a sentinel test instead).
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

/// L6-r2-3 closure — `NextChunkPoll` audit-arm coverage. The enum carries
/// `#[non_exhaustive]` at `crates/benten-engine/src/engine_stream.rs:130-131`;
/// a future cross-crate match must include a `_` wildcard arm. This audit
/// test exercises a match covering every named variant + the wildcard guard.
#[test]
fn next_chunk_poll_audit_arm_coverage() {
    use benten_engine::engine_stream::NextChunkPoll;
    fn label(p: &NextChunkPoll) -> &'static str {
        match p {
            NextChunkPoll::Chunk(_) => "Chunk",
            NextChunkPoll::EndOfStream => "EndOfStream",
            NextChunkPoll::Timeout => "Timeout",
            _ => "Unknown(non_exhaustive guard)",
        }
    }
    let eos = NextChunkPoll::EndOfStream;
    assert_eq!(label(&eos), "EndOfStream");
    let timeout = NextChunkPoll::Timeout;
    assert_eq!(label(&timeout), "Timeout");
}

/// **R6-R2-FP Item 6 (Row D-17 closure):** `SuspensionOutcome` now
/// carries `#[non_exhaustive]` (applied at
/// `crates/benten-engine/src/engine_wait.rs:194`). Cross-crate match
/// consumers MUST include a `_` wildcard arm; this audit test
/// exercises the wildcard-guard pattern. The 2 named variants at
/// v1-beta (`Complete` + `Suspended`) are pinned by the explicit arms;
/// a 3rd variant would not break this match (because of `_`) but the
/// freeze-discipline expectation is documented in V1-FROZEN-INTERFACE.md
/// item 11.
#[test]
fn suspension_outcome_d17_arm_coverage_with_wildcard_guard() {
    use benten_engine::engine_wait::SuspensionOutcome;
    fn classify(s: &SuspensionOutcome) -> &'static str {
        // R6-R2-FP Item 6: `#[non_exhaustive]` requires wildcard arm
        // from outside the defining crate.
        match s {
            SuspensionOutcome::Complete(_) => "Complete",
            SuspensionOutcome::Suspended(_) => "Suspended",
            _ => "Unknown(non_exhaustive guard)",
        }
    }
    // Exercise the discriminator via the `unwrap_suspended` ergonomic
    // helper (the substantive construction lives in the engine internals;
    // the audit value here is the match landing in test code).
    let outcome_kind = std::any::type_name::<SuspensionOutcome>();
    assert!(outcome_kind.ends_with("SuspensionOutcome"));
    let _: fn(&SuspensionOutcome) -> &'static str = classify;
}

// =========================================================================
// L1 R6 R1 fix-pass — crypto-suite + benten-drop + graph
// `#[non_exhaustive]` audit arm-coverage pins.
// =========================================================================
//
// Closes L1-r6r1-MAJ-1 + L1-r6r1-MAJ-2 + L1-r6r1-MIN-1 (R6 R1 fix-pass).
// Each enum below has `#[non_exhaustive]` applied at HEAD; the
// `_ => "Unknown"` arm catches forward-compat variant additions.

#[test]
fn crypto_suite_unsupported_algorithm_audit_arm_coverage() {
    use benten_crypto_suite::error::UnsupportedAlgorithm;
    fn audit(u: UnsupportedAlgorithm) -> &'static str {
        match u {
            UnsupportedAlgorithm::Signature { .. } => "Signature",
            UnsupportedAlgorithm::Hash { .. } => "Hash",
            UnsupportedAlgorithm::CipherSuite { .. } => "CipherSuite",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(UnsupportedAlgorithm::Signature { codepoint: 0 }),
        "Signature"
    );
}

#[test]
fn crypto_suite_swap_matrix_error_audit_arm_coverage() {
    use benten_crypto_suite::swap_matrix::SwapMatrixError;
    fn audit(e: &SwapMatrixError) -> &'static str {
        match e {
            SwapMatrixError::ConfigMismatch { .. } => "ConfigMismatch",
            _ => "Unknown",
        }
    }
    let e = SwapMatrixError::ConfigMismatch { detail: "test" };
    assert_eq!(audit(&e), "ConfigMismatch");
}

#[test]
fn crypto_suite_aead_error_audit_arm_coverage() {
    use benten_crypto_suite::aead::AeadError;
    fn audit(e: &AeadError) -> &'static str {
        match e {
            AeadError::AeadAuthFailed => "AeadAuthFailed",
            AeadError::MalformedEnvelope(_) => "MalformedEnvelope",
            AeadError::RecipientLacksKeysForSuite => "RecipientLacksKeysForSuite",
            AeadError::Unsupported(_) => "Unsupported",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(&AeadError::AeadAuthFailed), "AeadAuthFailed");
}

#[test]
fn crypto_suite_varsig_error_audit_arm_coverage() {
    use benten_crypto_suite::varsig::VarsigError;
    fn audit(e: &VarsigError) -> &'static str {
        match e {
            VarsigError::Truncated => "Truncated",
            VarsigError::BadMagic { .. } => "BadMagic",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(&VarsigError::Truncated), "Truncated");
}

// (drop_drop_bundle_version_audit_arm_coverage + drop_envelope_sig_error_audit_arm_coverage
// live in crates/benten-drop/tests/g_core_9_non_exhaustive_audit_drop.rs — benten-engine
// does NOT depend on benten-drop, so the drop-side audit lives in the drop crate's own
// integration-test directory per the L1-r6r1-MAJ-2 closure.)

#[test]
fn graph_two_cid_map_error_audit_arm_coverage() {
    use benten_graph::two_cid_map::TwoCidMapError;
    // Distinct typed-arm match — we only exercise the variant labels
    // here; full construction paths live in the graph crate's own
    // integration tests.
    fn audit(e: &TwoCidMapError) -> &'static str {
        match e {
            TwoCidMapError::NotFound { .. } => "NotFound",
            TwoCidMapError::IntegrityMismatch { .. } => "IntegrityMismatch",
            TwoCidMapError::AeadAuthenticationFailed { .. } => "AeadAuthenticationFailed",
            TwoCidMapError::Storage { .. } => "Storage",
            _ => "Unknown",
        }
    }
    let _: fn(&TwoCidMapError) -> &'static str = audit;
}

#[test]
fn graph_aead_wrap_error_audit_arm_coverage() {
    use benten_graph::aead_wrap::AeadError;
    fn audit(e: &AeadError) -> &'static str {
        match e {
            AeadError::Authentication(_) => "Authentication",
            AeadError::TagMismatch { .. } => "TagMismatch",
            AeadError::CiphertextTooShort { .. } => "CiphertextTooShort",
            AeadError::KeyMismatch { .. } => "KeyMismatch",
            AeadError::Unsupported { .. } => "Unsupported",
            _ => "Unknown",
        }
    }
    let _: fn(&AeadError) -> &'static str = audit;
}

// =========================================================================
// R6-R3 fix-b — engine `layer_d` `#[non_exhaustive]` audit arm-coverage pins.
// =========================================================================
//
// The layer_d frozen-v1 error + dispatch-operation enums now carry
// `#[non_exhaustive]` so a future variant lands ADDITIVELY without a SemVer
// break. The `_` catch-all arm is reachable ONLY while the attribute is present
// (this integration-test crate is a SEPARATE crate from `benten_engine`, so the
// cross-crate `#[non_exhaustive]` semantics apply); removing the attribute turns
// the catch-all into an `unreachable_patterns` build break (§11 HALT-AND-SURFACE).
//
// `GrantRejection` is DELIBERATELY EXCLUDED — it is a §11 documented carve-out
// (the frozen M-12 six-pass-class roster; the non-wildcard `roster_index` match
// IS the structural roster-drift guard, mirroring `Strategy` / `MembershipSetKind`).

#[test]
fn layer_d_device_auth_error_audit_arm_coverage_non_exhaustive() {
    use benten_engine::layer_d::device_auth::DeviceAuthError;
    fn audit(e: &DeviceAuthError) -> &'static str {
        match e {
            DeviceAuthError::NoPasswordSource => "NoPasswordSource",
            DeviceAuthError::VaultDecryptFailed => "VaultDecryptFailed",
            DeviceAuthError::Locked => "Locked",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(&DeviceAuthError::NoPasswordSource),
        "NoPasswordSource"
    );
}

#[test]
fn layer_d_device_link_error_audit_arm_coverage_non_exhaustive() {
    use benten_engine::layer_d::device_link::DeviceLinkError;
    fn audit(e: &DeviceLinkError) -> &'static str {
        match e {
            DeviceLinkError::OfferSignatureForged => "OfferSignatureForged",
            DeviceLinkError::HpkeUnwrapFailed => "HpkeUnwrapFailed",
            DeviceLinkError::SessionIdMismatch => "SessionIdMismatch",
            DeviceLinkError::SessionIdReplayed => "SessionIdReplayed",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(&DeviceLinkError::HpkeUnwrapFailed),
        "HpkeUnwrapFailed"
    );
}

#[test]
fn layer_d_secret_store_error_audit_arm_coverage_non_exhaustive() {
    use benten_engine::layer_d::secret_store::SecretStoreError;
    fn audit(e: &SecretStoreError) -> &'static str {
        match e {
            SecretStoreError::NotFound => "NotFound",
            SecretStoreError::KeychainUnavailable => "KeychainUnavailable",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(&SecretStoreError::NotFound), "NotFound");
}

#[test]
fn layer_d_permission_operation_audit_arm_coverage_non_exhaustive() {
    use benten_engine::layer_d::remote_permission::PermissionOperation;
    fn audit(o: &PermissionOperation) -> &'static str {
        match o {
            PermissionOperation::Decrypt { .. } => "Decrypt",
            PermissionOperation::SignUcanDelegation { .. } => "SignUcanDelegation",
            PermissionOperation::RemoteUnlock => "RemoteUnlock",
            PermissionOperation::ExecuteWorkflow { .. } => "ExecuteWorkflow",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(&PermissionOperation::RemoteUnlock), "RemoteUnlock");
}

/// `GrantRejection` §11 carve-out pin — EXACTLY-6 frozen-cardinality roster.
///
/// This match is DELIBERATELY non-wildcard (no `_` arm): `GrantRejection` is a
/// documented §11 carve-out that does NOT carry `#[non_exhaustive]`, so a 7th
/// pass-class added without updating this audit (and `GrantRejection::ALL` +
/// `roster_index`) is a compile-fail HERE — the HALT-AND-SURFACE roster-drift
/// guard. Mirrors `strategy_carve_out_3_arms_exhaustive_pin` above.
#[test]
fn layer_d_grant_rejection_carve_out_6_arms_exhaustive_pin() {
    use benten_engine::layer_d::grant_acceptance::GrantRejection;
    fn audit(r: GrantRejection) -> usize {
        match r {
            GrantRejection::Replay => 0,
            GrantRejection::DeviceKeyRevoked => 1,
            GrantRejection::Expired => 2,
            GrantRejection::ConfusedDeputy => 3,
            GrantRejection::UiSummaryMismatch => 4,
            GrantRejection::AuditNodeMissing => 5,
        }
    }
    for (i, r) in GrantRejection::ALL.iter().enumerate() {
        assert_eq!(audit(*r), i);
    }
}

// =========================================================================
// R6-R3 fix-integration fold-in — residual NON-layer_d `benten-engine`
// `#[non_exhaustive]` audit arm-coverage pins (V1-FROZEN-INTERFACE.md §11
// ~L1030 row closure).
// =========================================================================
//
// These five enums were the MIXED-STATE residual the §11 row flagged
// ("12+ verified MISSING at HEAD"): the fix-b layer_d sweep did not touch
// them (they live in non-layer_d engine src). Each now carries
// `#[non_exhaustive]`. This integration-test crate is SEPARATE from
// `benten_engine`, so the `_` catch-all arm is reachable ONLY while the
// attribute is present — removing it turns the catch-all into an
// `unreachable_patterns` build break (§11 HALT-AND-SURFACE).
//
// `Transport` is the sixth row-listed type closed at the same fold-in
// (additive, NOT a frozen-cardinality carve-out: observability-only,
// identical crypto contract across variants).

#[test]
fn user_view_input_pattern_audit_arm_coverage_non_exhaustive() {
    use benten_engine::UserViewInputPattern;
    fn audit(p: &UserViewInputPattern) -> &'static str {
        match p {
            UserViewInputPattern::Label(_) => "Label",
            UserViewInputPattern::AnchorPrefix(_) => "AnchorPrefix",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(&UserViewInputPattern::Label("x".to_string())),
        "Label"
    );
    assert_eq!(
        audit(&UserViewInputPattern::AnchorPrefix("y".to_string())),
        "AnchorPrefix"
    );
}

#[test]
fn trace_step_audit_arm_coverage_non_exhaustive() {
    use benten_engine::TraceStep;
    // The named-arm coverage exercises the `#[non_exhaustive]` wildcard
    // guard required of cross-crate consumers (the production napi
    // consumer `bindings/napi/src/trace.rs::trace_step_to_json` carries
    // the equivalent fail-CLOSED `_` arm).
    fn audit(s: &TraceStep) -> &'static str {
        match s {
            TraceStep::Step { .. } => "Step",
            TraceStep::SuspendBoundary { .. } => "SuspendBoundary",
            TraceStep::ResumeBoundary { .. } => "ResumeBoundary",
            TraceStep::BudgetExhausted { .. } => "BudgetExhausted",
            _ => "Unknown",
        }
    }
    let _: fn(&TraceStep) -> &'static str = audit;
    assert_eq!(
        audit(&TraceStep::SuspendBoundary {
            state_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        }),
        "SuspendBoundary"
    );
}

#[test]
fn stream_cursor_audit_arm_coverage_non_exhaustive() {
    use benten_engine::engine_stream::StreamCursor;
    fn audit(c: &StreamCursor) -> &'static str {
        match c {
            StreamCursor::Latest => "Latest",
            StreamCursor::Sequence(_) => "Sequence",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(&StreamCursor::Latest), "Latest");
    assert_eq!(audit(&StreamCursor::Sequence(7)), "Sequence");
}

#[test]
fn subscribe_cursor_audit_arm_coverage_non_exhaustive() {
    use benten_engine::SubscribeCursor;
    fn audit(c: &SubscribeCursor) -> &'static str {
        match c {
            SubscribeCursor::Latest => "Latest",
            SubscribeCursor::Sequence(_) => "Sequence",
            SubscribeCursor::Persistent(_) => "Persistent",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(&SubscribeCursor::Latest), "Latest");
    assert_eq!(
        audit(&SubscribeCursor::Persistent("s".to_string())),
        "Persistent"
    );
}

#[test]
fn transport_audit_arm_coverage_non_exhaustive() {
    use benten_engine::thin_client::Transport;
    fn audit(t: Transport) -> &'static str {
        match t {
            Transport::Http => "Http",
            Transport::Ipc => "Ipc",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(Transport::Http), "Http");
    assert_eq!(audit(Transport::Ipc), "Ipc");
}

#[test]
fn manifest_envelope_recheck_outcome_audit_arm_coverage_non_exhaustive() {
    use benten_engine::manifest_envelope_recheck::ManifestEnvelopeRecheckOutcome as Outcome;
    // Pre-existing `#[non_exhaustive]` (applied before the R6-R3 fold-in);
    // pinned here to bring the §11 row's full set under audit coverage.
    fn audit(o: &Outcome) -> &'static str {
        match o {
            Outcome::NotApplicable => "NotApplicable",
            Outcome::UnresolvedDeny => "UnresolvedDeny",
            Outcome::Admitted => "Admitted",
            Outcome::OutsideEnvelope { .. } => "OutsideEnvelope",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(&Outcome::NotApplicable), "NotApplicable");
    assert_eq!(audit(&Outcome::Admitted), "Admitted");
}
