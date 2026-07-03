//! Layer-D remote-permission grant-acceptance pipeline — the §6.7 / §9.1-4
//! six-class freeze-gate harness (R0.7 §3.4; M-12).
//!
//! The pre-merge security mini-review (owner = the threat-model lens,
//! Pattern 6) reads this harness. §9.1 item-4 reads "mini-review passed, clean
//! on all SIX pass-classes." The six classes (e2r's 5 + audit-Node-binding):
//!   1. **replay** — a `jti` already in the nonce-cache is rejected (the
//!      `jti`-keyed nonce-cache is the net-new v1-beta replay-defense, M-1);
//!   2. **device-key-revocation** — a grant from a RotationLog-revoked device
//!      key is rejected (revocation cuts FUTURE grants);
//!   3. **clock-skew** — the `valid_until` enforcement clock is decoupled from
//!      the coarse 1-hour bucket (NQ-T2) and enforced STRICTLY (`present >
//!      valid_until → reject`; NO grace/skew window);
//!   4. **confused-deputy** — the audience/operation is bound and checked
//!      BEFORE the time-window (the `validate_chain_for_audience_at`-before-
//!      `validate_chain_at` ordering precedent);
//!   5. **UI-deception** — the signed grant binds the displayed
//!      operation-summary hash; a displayed-vs-bound mismatch is rejected;
//!   6. **audit-Node-binding** — the grant is REJECTED if `audit_node_cid` is
//!      absent OR unresolvable (un-replicated). So a "grant without audit
//!      trail" is non-constructible; a malicious device can't grant-and-hide
//!      (NQ-T1 replication).

use std::collections::HashSet;

/// The six grant-rejection pass-classes (the frozen M-12 roster). The
/// non-wildcard `match` in [`GrantRejection::roster_index`] is the
/// compiler-enforced roster-drift guard: a future 7th variant fails to compile
/// until [`GrantRejection::ALL`] + every consumer is updated in lock-step.
///
/// **§11 `#[non_exhaustive]` CARVE-OUT (documented; registered in
/// `docs/V1-FROZEN-INTERFACE.md` item 11 carve-out registry).** This enum is
/// DELIBERATELY exhaustive-by-design — applying `#[non_exhaustive]` would
/// defeat the [`GrantRejection::roster_index`] non-wildcard roster-drift guard
/// (the whole point of the M-12 roster: a 7th pass-class must HALT-AND-SURFACE
/// at every consumer match site, not slip in additively). Structurally
/// identical to the `benten-ivm::Strategy` / `MembershipSetKind` frozen-
/// cardinality carve-outs: the EXACTLY-6-arms-by-the-type-system property IS
/// the structural pin. Adding a 7th class is a Composing-time architectural
/// decision, NOT a SemVer non-breaking variant addition.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum GrantRejection {
    /// Class 1 — replayed `jti` (nonce-cache hit).
    Replay,
    /// Class 2 — issuer device key was revoked (RotationLog).
    DeviceKeyRevoked,
    /// Class 3 — `valid_until` enforcement-clock expired (strict; NQ-T2).
    Expired,
    /// Class 4 — audience/operation mismatch (checked BEFORE the time-window).
    ConfusedDeputy,
    /// Class 5 — displayed operation-summary hash ≠ bound summary
    /// (UI-deception).
    UiSummaryMismatch,
    /// Class 6 — `audit_node_cid` absent or unresolvable.
    AuditNodeMissing,
}

impl GrantRejection {
    /// The frozen M-12 roster — the SINGLE source of truth for "how many
    /// pass-classes the §6.7 mini-review reads."
    pub const ALL: [GrantRejection; 6] = [
        GrantRejection::Replay,
        GrantRejection::DeviceKeyRevoked,
        GrantRejection::Expired,
        GrantRejection::ConfusedDeputy,
        GrantRejection::UiSummaryMismatch,
        GrantRejection::AuditNodeMissing,
    ];

    /// Compiler-enforced roster-drift guard. NON-wildcard `match` (no `_ =>`
    /// arm): a future 7th variant stops this compiling until [`Self::ALL`] +
    /// every consumer are updated in lock-step. Forecloses the class-of-bug
    /// where a new rejection class silently escapes the §6.7 coverage check.
    #[must_use]
    pub const fn roster_index(self) -> usize {
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

/// Compile-time tie: [`GrantRejection::ALL`] enumerates exactly the variants
/// the non-wildcard [`GrantRejection::roster_index`] match arms cover, in
/// order.
const _ROSTER_INDEX_CONSISTENT: () = {
    let mut i = 0;
    while i < GrantRejection::ALL.len() {
        assert!(GrantRejection::ALL[i].roster_index() == i);
        i += 1;
    }
};

/// The grant + the acceptance context. The acceptance pipeline runs the six
/// checks in the §6.7-mandated order: confused-deputy (audience/operation) is
/// checked BEFORE the time-window (class 4 before class 3).
pub struct GrantAcceptanceContext<'a> {
    /// The grant's `jti` (replay key).
    pub jti: [u8; 32],
    /// The per-device `jti` nonce-cache (mutated on admit). This is a
    /// **caller-supplied** in-RAM set: per-device durability is a **caller
    /// contract** (the caller persists this set to disk + re-hydrates it on
    /// restart, mirroring the `benten_sync::handshake::JtiNonceCache`
    /// durable-CAS-marker + `from_durable` hydration seam). The full
    /// disk-persistence wiring on this (currently zero-production-caller)
    /// accept-grant path is DEFERRED with the remote-permission wiring
    /// (`docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-64-adjacent; Compromise
    /// #64 / NQ-T4). NOT an intrinsically-durable store at v1-beta-core.
    pub nonce_cache: &'a mut HashSet<[u8; 32]>,
    /// The RotationLog-revoked device-key set.
    pub revoked_device_keys: &'a HashSet<[u8; 32]>,
    /// The issuer device key (checked against the revocation set).
    pub issuer_device_key: [u8; 32],
    /// The audience the operation was requested for.
    pub requested_audience: Vec<u8>,
    /// The audience the grant was issued to.
    pub grant_audience: Vec<u8>,
    /// The full-granularity `valid_until` enforcement clock (NQ-T2).
    pub valid_until: u64,
    /// The current time (seconds).
    pub now_secs: u64,
    /// The operation-summary hash the operator was shown.
    pub displayed_summary_hash: [u8; 32],
    /// The operation-summary hash bound into the signed grant.
    pub bound_summary_hash: [u8; 32],
    /// The grant's audit-Node CID (absent ⇒ non-constructible).
    pub audit_node_cid: Option<[u8; 32]>,
    /// Which audit-Node CIDs are resolvable (replicated; NQ-T1).
    pub resolvable_audit_cids: &'a HashSet<[u8; 32]>,
}

/// Run the six-class grant-acceptance pipeline. Returns `Ok(())` and records
/// the `jti` on admit; otherwise the typed [`GrantRejection`] of the FIRST
/// failing class in the §6.7-mandated order.
///
/// # Errors
///
/// Returns the [`GrantRejection`] of the first failing pass-class.
pub fn accept_grant(ctx: GrantAcceptanceContext<'_>) -> Result<(), GrantRejection> {
    // Class 6 FIRST (audit-binding): a grant without a resolvable audit trail
    // is non-constructible.
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
    // Class 3 time-window (valid_until enforcement clock — distinct from the
    // 1-hr bucket per NQ-T2; STRICT, no grace/skew).
    if ctx.now_secs > ctx.valid_until {
        return Err(GrantRejection::Expired);
    }
    // Admit + record the nonce.
    ctx.nonce_cache.insert(ctx.jti);
    Ok(())
}
