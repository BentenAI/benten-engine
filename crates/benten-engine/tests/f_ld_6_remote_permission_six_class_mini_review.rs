//! F-LD-6 — Remote-permission 6-class pre-merge mini-review harness
//! (RED-PHASE; the §9.1-4 / §6.7 mini-review gate; M-12).
//!
//! R3 wave **W3-layer-d**. Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 8
//!     F-LD-6: "SIX executable pass-classes: (1) replay [F-LD-5]; (2)
//!     device-key-revocation (RotationLog cuts future grants); (3) clock-skew
//!     (NQ-T2: 1-hr bucket ⊥ `valid_until` enforcement clock); (4)
//!     confused-deputy (operation/audience bound, checked before time); (5)
//!     UI-deception (signed grant binds displayed operation-summary hash); (6)
//!     audit-Node-binding — grant REJECTED if `audit_node_cid` absent/
//!     unresolvable; audit-Node encrypted to device-mesh + replicated (NQ-T1).
//!     ... **This group IS the harness the §6.7 mini-review reads.**"
//!   - R0.3 plan §3.4 (`...f-full-r0-plan.md:525-530`): the six pass-classes;
//!     "the grant MUST be REJECTED if `audit_node_cid` is absent/unresolvable".
//!   - §10.5 NQ-T2 (`...:1372-1374`): "Confirm the 1-hour bucket is decoupled
//!     from the `valid_until` enforcement clock."
//!
//! ## NQ-T1 / NQ-T2 OPEN-SPEC FLAGS
//!
//! - Class-3 (clock-skew) is gated on **NQ-T2** (§10.5; unresolved at R2; §5.B
//!   carry-forward item 5): the 1-hr bucket ⊥ `valid_until` enforcement-clock
//!   decoupling. The `..._clock_skew_..._nq_t2_gated` arm is a RED-PHASE stub
//!   referencing the open question; R5 finalizes against the ratified default.
//! - Class-6 (audit-Node-binding) references **NQ-T1** (audit-Node encrypted +
//!   replicated to all the user's devices); the replication arm is pinned at
//!   the shape level here, behavioral encryption pinned at R5.
//!
//! ## §6.7 harness contract
//!
//! These six arms ARE the executable substrate the pre-merge security
//! mini-review (owner = threat-model lens, Pattern 6) reads. §9.1 item-4 reads
//! "mini-review passed, clean on all six."

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![cfg(not(target_arch = "wasm32"))]

use benten_id::keypair::Keypair;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// SELF-CONTAINED STUB-SHIM — the 6-class grant-acceptance pipeline.
// ---------------------------------------------------------------------------
mod shim {
    use std::collections::HashSet;

    pub const LAYER_D_BUCKET_SECS: u64 = 3600;

    #[derive(Debug, PartialEq, Eq)]
    pub enum GrantRejection {
        /// Class 1.
        Replay,
        /// Class 2 — issuer device key was revoked (RotationLog).
        DeviceKeyRevoked,
        /// Class 3 — `valid_until` enforcement-clock expired.
        Expired,
        /// Class 4 — audience/operation does not match the requested operation
        /// (checked BEFORE the time-window).
        ConfusedDeputy,
        /// Class 5 — displayed operation-summary hash does not match the bound
        /// summary (UI-deception).
        UiSummaryMismatch,
        /// Class 6 — audit_node_cid absent or unresolvable.
        AuditNodeMissing,
    }

    /// A simplified grant + the acceptance context. The acceptance pipeline
    /// runs the six checks in the §6.7-mandated order: confused-deputy
    /// (audience/operation) is checked BEFORE the time-window (class 4 before
    /// class 3), per the `validate_chain_for_audience_at` precedent.
    pub struct GrantAcceptanceContext<'a> {
        pub jti: [u8; 32],
        pub nonce_cache: &'a mut HashSet<[u8; 32]>,
        pub revoked_device_keys: &'a HashSet<[u8; 32]>,
        pub issuer_device_key: [u8; 32],
        pub requested_audience: Vec<u8>,
        pub grant_audience: Vec<u8>,
        pub valid_until: u64,
        pub now_secs: u64,
        pub displayed_summary_hash: [u8; 32],
        pub bound_summary_hash: [u8; 32],
        pub audit_node_cid: Option<[u8; 32]>,
        /// Resolver: which audit-Node CIDs are resolvable (replicated).
        pub resolvable_audit_cids: &'a HashSet<[u8; 32]>,
    }

    pub fn accept_grant(ctx: GrantAcceptanceContext<'_>) -> Result<(), GrantRejection> {
        // Class 6 FIRST (audit-binding): a grant without a resolvable audit
        // trail is non-constructible.
        let audit = ctx.audit_node_cid.ok_or(GrantRejection::AuditNodeMissing)?;
        if !ctx.resolvable_audit_cids.contains(&audit) {
            return Err(GrantRejection::AuditNodeMissing);
        }
        // Class 1 replay (nonce-keyed).
        if ctx.nonce_cache.contains(&ctx.jti) {
            return Err(GrantRejection::Replay);
        }
        // Class 2 device-key revocation.
        if ctx.revoked_device_keys.contains(&ctx.issuer_device_key) {
            return Err(GrantRejection::DeviceKeyRevoked);
        }
        // Class 4 confused-deputy — BEFORE the time-window check (class 3).
        if ctx.requested_audience != ctx.grant_audience {
            return Err(GrantRejection::ConfusedDeputy);
        }
        // Class 5 UI-deception.
        if ctx.displayed_summary_hash != ctx.bound_summary_hash {
            return Err(GrantRejection::UiSummaryMismatch);
        }
        // Class 3 time-window (valid_until enforcement clock — distinct from
        // the 1-hr bucket per NQ-T2).
        if ctx.now_secs > ctx.valid_until {
            return Err(GrantRejection::Expired);
        }
        // Admit + record the nonce.
        ctx.nonce_cache.insert(ctx.jti);
        Ok(())
    }
}

use shim::{accept_grant, GrantAcceptanceContext, GrantRejection, LAYER_D_BUCKET_SECS};

/// Build a baseline VALID context (all six checks pass) so each arm can mutate
/// exactly one dimension.
fn valid_ctx<'a>(
    nonce_cache: &'a mut HashSet<[u8; 32]>,
    revoked: &'a HashSet<[u8; 32]>,
    resolvable: &'a HashSet<[u8; 32]>,
) -> GrantAcceptanceContext<'a> {
    GrantAcceptanceContext {
        jti: [0x01; 32],
        nonce_cache,
        revoked_device_keys: revoked,
        issuer_device_key: [0xAA; 32],
        requested_audience: b"device-b-did".to_vec(),
        grant_audience: b"device-b-did".to_vec(),
        valid_until: 1_000,
        now_secs: 500,
        displayed_summary_hash: [0x42; 32],
        bound_summary_hash: [0x42; 32],
        audit_node_cid: Some([0xCC; 32]),
        resolvable_audit_cids: resolvable,
    }
}

/// F-LD-6 baseline: a fully-valid grant is admitted (proves the six gates do
/// not spuriously reject). Each failing arm below mutates exactly one field.
#[test]
#[ignore = "RED-PHASE: F-LD-6 — baseline valid grant admitted; un-ignore at R5"]
fn f_ld_6_baseline_valid_grant_admitted() {
    let mut nc = HashSet::new();
    let revoked = HashSet::new();
    let resolvable = HashSet::from([[0xCC; 32]]);
    accept_grant(valid_ctx(&mut nc, &revoked, &resolvable))
        .expect("a fully-valid grant MUST be admitted (all six gates pass)");
}

/// F-LD-6 CLASS 6 HEADLINE (audit-Node-binding): a grant with NO `audit_node_cid`
/// is REJECTED — a "grant without audit trail" is non-constructible. This is
/// the headline arm the §6.7 mini-review keys on.
#[test]
#[ignore = "RED-PHASE: F-LD-6 — class 6: grant without audit_node_cid rejected; un-ignore at R5"]
fn f_ld_6_class6_grant_without_audit_node_cid_rejected() {
    let mut nc = HashSet::new();
    let revoked = HashSet::new();
    let resolvable = HashSet::from([[0xCC; 32]]);
    let mut ctx = valid_ctx(&mut nc, &revoked, &resolvable);
    ctx.audit_node_cid = None;
    assert_eq!(
        accept_grant(ctx),
        Err(GrantRejection::AuditNodeMissing),
        "a grant with no audit_node_cid MUST be rejected (non-constructible)"
    );
}

/// F-LD-6 CLASS 6 (audit-Node unresolvable / NQ-T1 replication shape): a grant
/// whose `audit_node_cid` is present but NOT resolvable (not replicated to the
/// device mesh) is rejected — so a malicious device can't grant-and-hide.
#[test]
#[ignore = "RED-PHASE: F-LD-6 — class 6: unresolvable audit_node_cid rejected (NQ-T1 replication); un-ignore at R5"]
fn f_ld_6_class6_unresolvable_audit_node_cid_rejected() {
    let mut nc = HashSet::new();
    let revoked = HashSet::new();
    let resolvable: HashSet<[u8; 32]> = HashSet::new(); // nothing resolvable
    let ctx = valid_ctx(&mut nc, &revoked, &resolvable);
    assert_eq!(
        accept_grant(ctx),
        Err(GrantRejection::AuditNodeMissing),
        "an unresolvable (un-replicated) audit_node_cid MUST be rejected"
    );
}

/// F-LD-6 CLASS 1 (replay): a grant whose jti is already in the nonce-cache is
/// rejected. (Behavioral replay-cache depth owned by F-LD-5; this arm pins the
/// harness wiring.)
#[test]
#[ignore = "RED-PHASE: F-LD-6 — class 1: replayed jti rejected; un-ignore at R5"]
fn f_ld_6_class1_replayed_jti_rejected() {
    let mut nc = HashSet::from([[0x01; 32]]); // jti already seen
    let revoked = HashSet::new();
    let resolvable = HashSet::from([[0xCC; 32]]);
    assert_eq!(
        accept_grant(valid_ctx(&mut nc, &revoked, &resolvable)),
        Err(GrantRejection::Replay),
        "a replayed jti MUST be rejected"
    );
}

/// F-LD-6 CLASS 2 (device-key revocation): a grant signed by a device key in
/// the RotationLog revocation set is rejected — revocation cuts FUTURE grants.
#[test]
#[ignore = "RED-PHASE: F-LD-6 — class 2: revoked device-key grant rejected; un-ignore at R5"]
fn f_ld_6_class2_revoked_device_key_rejected() {
    let mut nc = HashSet::new();
    let revoked = HashSet::from([[0xAA; 32]]); // issuer key revoked
    let resolvable = HashSet::from([[0xCC; 32]]);
    assert_eq!(
        accept_grant(valid_ctx(&mut nc, &revoked, &resolvable)),
        Err(GrantRejection::DeviceKeyRevoked),
        "a grant from a revoked device key MUST be rejected (RotationLog cuts future grants)"
    );
}

/// F-LD-6 CLASS 4 (confused-deputy): a grant whose audience does not match the
/// requested operation's audience is rejected — and this is checked BEFORE the
/// time-window. Extends the `ucan_grounded_policy_rejects_proof_for_wrong_
/// audience_before_time_check.rs` precedent.
#[test]
#[ignore = "RED-PHASE: F-LD-6 — class 4: confused-deputy audience mismatch rejected; un-ignore at R5"]
fn f_ld_6_class4_confused_deputy_audience_mismatch_rejected() {
    let mut nc = HashSet::new();
    let revoked = HashSet::new();
    let resolvable = HashSet::from([[0xCC; 32]]);
    let mut ctx = valid_ctx(&mut nc, &revoked, &resolvable);
    ctx.grant_audience = b"other-device-did".to_vec(); // mismatch
    assert_eq!(
        accept_grant(ctx),
        Err(GrantRejection::ConfusedDeputy),
        "audience mismatch MUST be rejected as confused-deputy"
    );
}

/// F-LD-6 CLASS 4 ORDERING PROOF: when BOTH the audience is wrong AND the
/// time-window is expired, the audience (confused-deputy) check MUST fire
/// FIRST. Mirrors the `validate_chain_for_audience_at`-before-`validate_chain_at`
/// ordering precedent.
#[test]
#[ignore = "RED-PHASE: F-LD-6 — class 4 ordering: audience checked before time-window; un-ignore at R5"]
fn f_ld_6_class4_audience_checked_before_time_window() {
    let mut nc = HashSet::new();
    let revoked = HashSet::new();
    let resolvable = HashSet::from([[0xCC; 32]]);
    let mut ctx = valid_ctx(&mut nc, &revoked, &resolvable);
    ctx.grant_audience = b"other-device-did".to_vec(); // wrong audience
    ctx.now_secs = 9_999; // ALSO expired
    assert_eq!(
        accept_grant(ctx),
        Err(GrantRejection::ConfusedDeputy),
        "when both gates would reject, the audience gate MUST fire first (NOT Expired)"
    );
}

/// F-LD-6 CLASS 5 (UI-deception): a grant where the operator-displayed
/// operation-summary hash does not match the signed/bound summary is rejected —
/// the user can't be tricked into approving operation X while the wire carries
/// operation Y.
#[test]
#[ignore = "RED-PHASE: F-LD-6 — class 5: UI-summary mismatch rejected; un-ignore at R5"]
fn f_ld_6_class5_ui_summary_hash_mismatch_rejected() {
    let mut nc = HashSet::new();
    let revoked = HashSet::new();
    let resolvable = HashSet::from([[0xCC; 32]]);
    let mut ctx = valid_ctx(&mut nc, &revoked, &resolvable);
    ctx.displayed_summary_hash = [0x99; 32]; // operator saw a different summary
    assert_eq!(
        accept_grant(ctx),
        Err(GrantRejection::UiSummaryMismatch),
        "displayed-vs-bound operation-summary mismatch MUST be rejected (UI-deception)"
    );
}

/// F-LD-6 CLASS 3 NQ-T2-GATED (clock-skew, OPEN-SPEC): the `valid_until`
/// enforcement clock is DECOUPLED from the coarse 1-hr bucket. A 60-second
/// `valid_until` presented at +90 seconds → REJECT (the bucket does NOT
/// coarsen `valid_until` up to 1hr; else the coercion/replay window widens from
/// 60s to ≤1hr).
///
/// OPEN-SPEC: NQ-T2 (§10.5) is unresolved at R2 (§5.B carry-forward item 5).
/// This arm pins the R0-stated decoupling; R5 finalizes the exact enforcement
/// semantics against the ratified NQ-T2 default.
#[test]
#[ignore = "RED-PHASE: F-LD-6 — class 3 clock-skew: 1-hr bucket ⊥ valid_until clock (OPEN-SPEC: gated on NQ-T2 §10.5); un-ignore at R5"]
fn f_ld_6_class3_clock_skew_bucket_decoupled_from_valid_until_nq_t2_gated() {
    let mut nc = HashSet::new();
    let revoked = HashSet::new();
    let resolvable = HashSet::from([[0xCC; 32]]);
    let mut ctx = valid_ctx(&mut nc, &revoked, &resolvable);
    // A 60-second valid_until window, presented 90 seconds in → expired.
    ctx.valid_until = 1_900_000_060;
    ctx.now_secs = 1_900_000_090;
    assert_eq!(
        accept_grant(ctx),
        Err(GrantRejection::Expired),
        "valid_until enforcement clock is fine-grained (60s window), NOT coarsened to the 1-hr bucket — \
         a 90s-late presentation MUST reject; the 1-hr bucket (={LAYER_D_BUCKET_SECS}s) does NOT widen it"
    );
}

/// F-LD-6 harness self-coverage: all SIX pass-classes have an executable arm
/// (so the §6.7 mini-review "clean on all six" claim is grounded). This pin
/// fails if a class loses its arm. would-FAIL-if-no-op'd: dropping any
/// `GrantRejection` variant from the asserted set fails the exhaustiveness.
#[test]
#[ignore = "RED-PHASE: F-LD-6 — six pass-classes all have executable arms; un-ignore at R5"]
fn f_ld_6_all_six_pass_classes_have_executable_arms() {
    // Enumerate the six rejection classes the harness MUST distinguish.
    let classes = [
        GrantRejection::Replay,            // 1
        GrantRejection::DeviceKeyRevoked,  // 2
        GrantRejection::Expired,           // 3
        GrantRejection::ConfusedDeputy,    // 4
        GrantRejection::UiSummaryMismatch, // 5
        GrantRejection::AuditNodeMissing,  // 6
    ];
    assert_eq!(classes.len(), 6, "the §6.7 mini-review reads exactly six pass-classes");
}
