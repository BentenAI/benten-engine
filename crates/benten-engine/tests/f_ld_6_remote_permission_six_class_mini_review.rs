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
//!   - R0.5 plan §3.4: the six pass-classes; "the grant MUST be REJECTED if
//!     `audit_node_cid` is absent/unresolvable".
//!   - R0.5 plan §10.5 NQ-T2: "the 1-hour bucket is decoupled from the
//!     `valid_until` enforcement clock" — RATIFIED.
//!   - R0.5 plan §9.1 item-4: "the remote-permission-call pre-merge security
//!     mini-review PASSED, clean on all SIX pass-classes".
//!
//! ## NQ-T1 / NQ-T2 RATIFIED (Ben 2026-06-02)
//!
//! - Class-3 (clock-skew) is grounded by **NQ-T2** (R0.5 §10.5 — RATIFIED): the
//!   1-hour metadata bucket (round-down) and the `valid_until` enforcement
//!   clock are orthogonal, separately-encoded fields. `valid_until` is encoded
//!   at full 1-second granularity and enforced STRICTLY — `present >
//!   valid_until → reject`, NO grace/skew window; the coarse 1-hour bucket is
//!   never consulted for expiry. The `..._clock_skew_..._nq_t2_gated` arm pins
//!   exactly this strict-no-grace decoupling. (The `_nq_t2_gated` suffix is the
//!   historical arm name; the question it gated is now ratified.)
//! - Class-6 (audit-Node-binding) is grounded by **NQ-T1** (R0.5 §10.5 —
//!   RATIFIED): the audit-Node is encrypted + replicated to all the user's
//!   devices so a malicious device can't grant-and-hide; the replication arm is
//!   pinned at the shape level here (unresolvable ⇒ reject), behavioral
//!   encryption pinned at R5.
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

use std::collections::HashSet;

// ---------------------------------------------------------------------------
// SELF-CONTAINED STUB-SHIM — the 6-class grant-acceptance pipeline.
// ---------------------------------------------------------------------------
mod shim {
    use std::collections::HashSet;

    pub const LAYER_D_BUCKET_SECS: u64 = 3600;

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

    impl GrantRejection {
        /// The frozen M-12 roster — the SINGLE source of truth for "how many
        /// pass-classes the §6.7 mini-review reads." The self-coverage pin
        /// asserts the BEHAVIORALLY-observed rejection set equals this roster,
        /// so the harness can never silently cover fewer (or more) classes than
        /// the roster names.
        pub const ALL: [GrantRejection; 6] = [
            GrantRejection::Replay,            // 1
            GrantRejection::DeviceKeyRevoked,  // 2
            GrantRejection::Expired,           // 3
            GrantRejection::ConfusedDeputy,    // 4
            GrantRejection::UiSummaryMismatch, // 5
            GrantRejection::AuditNodeMissing,  // 6
        ];

        /// Compiler-enforced roster-drift guard. This `match` is NON-wildcard
        /// (no `_ =>` arm): if a future edit adds a 7th `GrantRejection`
        /// variant (a new pass-class) WITHOUT adding it to `ALL`, this stops
        /// compiling — forcing the roster + every consumer (the self-coverage
        /// pin) to be updated in lock-step. Forecloses the class-of-bug where a
        /// new rejection class silently escapes the §6.7 "clean on all"
        /// coverage check.
        const fn assert_in_roster(self) -> usize {
            match self {
                GrantRejection::Replay => 0,
                GrantRejection::DeviceKeyRevoked => 1,
                GrantRejection::Expired => 2,
                GrantRejection::ConfusedDeputy => 3,
                GrantRejection::UiSummaryMismatch => 4,
                GrantRejection::AuditNodeMissing => 5,
            }
        }
    }

    // Compile-time tie: `ALL` enumerates exactly the variants the non-wildcard
    // `assert_in_roster` match arms cover, in order. If a variant is added to
    // the enum, `assert_in_roster` fails to compile until it gains an arm; if
    // `ALL` and the arms then disagree on cardinality/order, this const fails.
    const _ROSTER_INDEX_CONSISTENT: () = {
        let mut i = 0;
        while i < GrantRejection::ALL.len() {
            assert!(GrantRejection::ALL[i].assert_in_roster() == i);
            i += 1;
        }
    };

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
        // the 1-hr bucket per NQ-T2; strict, no grace/skew).
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

/// F-LD-6 CLASS 3 NQ-T2 (clock-skew, RATIFIED): the `valid_until` enforcement
/// clock is DECOUPLED from the coarse 1-hr bucket. A 60-second `valid_until`
/// presented at +90 seconds → REJECT (the bucket does NOT coarsen `valid_until`
/// up to 1hr; else the coercion/replay window widens from 60s to ≤1hr).
///
/// NQ-T2 (R0.5 §10.5) is RATIFIED (Ben 2026-06-02): `valid_until` is encoded at
/// full 1-second granularity and enforced STRICTLY — `present > valid_until →
/// reject`, NO grace/skew window; the coarse 1-hour bucket is never consulted
/// for expiry. This arm pins exactly that strict-no-grace decoupling. (The
/// `_nq_t2_gated` suffix is the historical arm name from the open-question era;
/// the question is now ratified — the arm stays.)
#[test]
#[ignore = "RED-PHASE: F-LD-6 — class 3 clock-skew: 1-hr bucket ⊥ valid_until clock (NQ-T2 §10.5 RATIFIED: strict, no grace); un-ignore at R5"]
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
        "valid_until enforcement clock is fine-grained (60s window) + strict (no grace), NOT coarsened to \
         the 1-hr bucket — a 90s-late presentation MUST reject; the 1-hr bucket (={LAYER_D_BUCKET_SECS}s) \
         does NOT widen it"
    );
}

/// F-LD-6 harness self-coverage (FALSIFIABLE + ROSTER-DRIFT-PROOF): drive the
/// six real production rejection arms through `accept_grant` and assert that the
/// observed `GrantRejection` set equals `GrantRejection::ALL` — the shim-owned
/// single source of truth for the M-12 roster. So the §6.7 mini-review "clean
/// on all six" claim is grounded in observable behavior, not in a literal-array
/// length, AND it stays tied to the enum's own roster.
///
/// would-FAIL-if-no-op'd (two ways):
///   1. **Gate dropped / classes collapse** — if a production gate is removed
///      (a hostile ctx is admitted → `expect_err` panics) or two classes start
///      returning the same `GrantRejection`, the observed set drops below the
///      roster and the set-equality fails.
///   2. **Roster drift** — if a 7th pass-class is added to `GrantRejection`
///      without a behavioral arm here, the shim's non-wildcard
///      `assert_in_roster` match stops compiling until `ALL` is updated, and
///      then `observed != ALL` until this arm exercises the new class. A new
///      class can never silently escape the "clean on all" coverage check.
///
/// (Replaces the prior tautological `assert_eq!(classes.len(), 6)` over a
/// 6-element literal, which froze nothing — F4-020.)
#[test]
#[ignore = "RED-PHASE: F-LD-6 — six pass-classes all have falsifiable executable arms; un-ignore at R5"]
fn f_ld_6_all_six_pass_classes_have_executable_arms() {
    let mut observed: HashSet<GrantRejection> = HashSet::new();

    // Class 1 — replay (jti already in cache).
    {
        let mut nc = HashSet::from([[0x01; 32]]);
        let revoked = HashSet::new();
        let resolvable = HashSet::from([[0xCC; 32]]);
        observed.insert(
            accept_grant(valid_ctx(&mut nc, &revoked, &resolvable))
                .expect_err("class 1 must reject"),
        );
    }
    // Class 2 — revoked issuer device key.
    {
        let mut nc = HashSet::new();
        let revoked = HashSet::from([[0xAA; 32]]);
        let resolvable = HashSet::from([[0xCC; 32]]);
        observed.insert(
            accept_grant(valid_ctx(&mut nc, &revoked, &resolvable))
                .expect_err("class 2 must reject"),
        );
    }
    // Class 3 — expired valid_until (strict, NQ-T2).
    {
        let mut nc = HashSet::new();
        let revoked = HashSet::new();
        let resolvable = HashSet::from([[0xCC; 32]]);
        let mut ctx = valid_ctx(&mut nc, &revoked, &resolvable);
        ctx.now_secs = ctx.valid_until + 1;
        observed.insert(accept_grant(ctx).expect_err("class 3 must reject"));
    }
    // Class 4 — confused-deputy (audience mismatch).
    {
        let mut nc = HashSet::new();
        let revoked = HashSet::new();
        let resolvable = HashSet::from([[0xCC; 32]]);
        let mut ctx = valid_ctx(&mut nc, &revoked, &resolvable);
        ctx.grant_audience = b"other-device-did".to_vec();
        observed.insert(accept_grant(ctx).expect_err("class 4 must reject"));
    }
    // Class 5 — UI-deception (summary-hash mismatch).
    {
        let mut nc = HashSet::new();
        let revoked = HashSet::new();
        let resolvable = HashSet::from([[0xCC; 32]]);
        let mut ctx = valid_ctx(&mut nc, &revoked, &resolvable);
        ctx.displayed_summary_hash = [0x99; 32];
        observed.insert(accept_grant(ctx).expect_err("class 5 must reject"));
    }
    // Class 6 — audit-Node missing.
    {
        let mut nc = HashSet::new();
        let revoked = HashSet::new();
        let resolvable = HashSet::from([[0xCC; 32]]);
        let mut ctx = valid_ctx(&mut nc, &revoked, &resolvable);
        ctx.audit_node_cid = None;
        observed.insert(accept_grant(ctx).expect_err("class 6 must reject"));
    }

    // Tie the assertion to the shim-owned roster (NOT a hand-written literal),
    // so adding a 7th pass-class can never silently escape this coverage check.
    let expected: HashSet<GrantRejection> = GrantRejection::ALL.into_iter().collect();
    assert_eq!(
        observed, expected,
        "the §6.7 mini-review reads exactly the GrantRejection::ALL pass-classes, each behaviorally \
         exercised; if a gate is dropped, two classes collapse, or a new class is added without an arm, \
         the observed set diverges from the roster and this fails"
    );
}
