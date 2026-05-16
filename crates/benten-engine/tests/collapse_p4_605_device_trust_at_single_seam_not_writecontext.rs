//! COLLAPSE (P4) — #605 v1-BLOCKER-cluster closure-pin (pim-2 §3.6b,
//! would-FAIL-if-no-op'd).
//!
//! # Charter
//!
//! Spec: `.addl/refinement-audit-2026-05/impl-design-COLLAPSE.md` §5
//! (#707 instance 10 = #605, the TRUST subset) + §1.5 (engine_sync
//! REWIRE) + `DECISION-RECORD-trust-model-reframe.md` §4 / §4a
//! (RATIFIED; device-DID demoted to provenance label, trust-decision
//! moves to the single seam).
//!
//! # The #605 finding and how COLLAPSE closes it
//!
//! #605: `Engine::apply_atrium_merge`'s per-row cap-recheck constructs
//! a synthetic `WriteContext` with `device_cid: None` while the other
//! three production `WriteContext` sites thread
//! `self.inner.device_cid`. The finding's concern: at the most-attacked
//! boundary (sync-merge), the per-device trust dispatch input is
//! `None`, "collapsing every per-device dispatch decision."
//!
//! **COLLAPSE closes this DEFINITIONALLY (impl-design §5):** under the
//! RATIFIED unified model the device is no longer a distinct
//! trust-root. The device-grain *trust decision* at the sync-merge
//! boundary is NOT carried by `WriteContext::device_cid` anymore — it
//! is carried by the verified inbound device `CapabilityEnvelope`
//! ceiling, AND-ed into the inbound writer's effective caps at the
//! **single chain-validation seam** (`envelope_ceiling_admits_row`,
//! the COLLAPSE P3 J8 ceiling-AND in the SAME per-row loop). The
//! `device_cid` WriteContext field is thereby demoted to a
//! provenance-label residual (audited by the §4a successor #1234 — a
//! quality-pass, NOT a trust hole). The #605 *trust* concern is closed
//! because the per-row trust enforcement now happens uniformly at the
//! one seam regardless of the `device_cid` field value.
//!
//! # What this pins (would-FAIL-if-no-op'd)
//!
//! 1. **Behavioral:** the J8 ceiling-AND that IS the post-COLLAPSE
//!    device-grain trust enforcement at the sync-merge per-row seam
//!    rejects a `runs_sandbox=false`-attested inbound writer's
//!    `host:sandbox:*` row with `E_DEVICE_ATTESTATION_FORGED` — and
//!    does so independently of any `WriteContext::device_cid` value
//!    (the function takes the verified ceiling, not a device_cid). If
//!    `envelope_ceiling_admits_row` were no-op'd, the device-grain
//!    trust decision at the #605 boundary silently vanishes — this
//!    FAILs.
//! 2. **Source-coupled (regression-defense):** the single per-row
//!    recheck loop in `engine.rs::apply_atrium_merge` MUST still call
//!    `envelope_ceiling_admits_row` AFTER the `check_write` ctx — i.e.
//!    the device-grain trust decision is at the single seam, not
//!    re-introduced as a parallel `device_cid`-threaded pipe (the
//!    #707/#605 asymmetric-parallel-entry-point shape the COLLAPSE
//!    exists to kill). If a reviewer "fixes" #605 by threading
//!    `device_cid` into the `WriteContext` AND removing the ceiling-AND
//!    (recreating the parallel pipe), assertion 2 FAILs.
//!
//! Pairs with the P3 pin
//! `collapse_p3_envelope_ceiling_and_closure_pin.rs` (the
//! ceiling-AND helper boundary) and the benten-caps single-revocation
//! -seam pin `collapse_p4_1230_605_707_single_revocation_seam.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use benten_engine::manifest_envelope_recheck::envelope_ceiling_admits_row;
use benten_id::device_attestation::{CapabilityEnvelope, UptimePolicy, ZoneScope};

/// A `runs_sandbox=false` inbound-device ceiling (thin-shape per
/// CLAUDE.md #17 — the device the #605 attacker would impersonate at
/// the sync-merge boundary).
fn thin_inbound_ceiling() -> CapabilityEnvelope {
    CapabilityEnvelope {
        runs_sandbox: false,
        holds_zones: ZoneScope::CacheOnly,
        online_uptime: UptimePolicy::SessionBounded,
        runs_atrium_peer: false,
    }
}

/// **MANDATORY closure-pin #605 (would-FAIL-if-no-op'd).**
///
/// The post-COLLAPSE device-grain trust enforcement at the sync-merge
/// per-row seam is the verified-ceiling AND — taking the inbound
/// device's verified `CapabilityEnvelope`, NOT a
/// `WriteContext::device_cid`. A `runs_sandbox=false`-attested inbound
/// writer's `host:sandbox:*` row MUST reject at this single seam with
/// `E_DEVICE_ATTESTATION_FORGED`, proving the #605 "per-device trust
/// dispatch collapsed to None" concern is closed: trust dispatch now
/// flows through the ceiling at the one seam, not the demoted
/// `device_cid` provenance-label field.
///
/// If `envelope_ceiling_admits_row` is no-op'd (returns `Ok(())`
/// unconditionally), the device-grain trust decision at the #605
/// boundary silently disappears and `expect_err` panics — pim-2 §3.6b.
#[test]
fn device_grain_trust_at_sync_merge_seam_flows_through_verified_ceiling_not_device_cid() {
    let inbound_ceiling = thin_inbound_ceiling();

    // The sync-merge per-row recheck builds `{zone}:write` for the
    // row scope; a host:sandbox authority zone yields this scope. The
    // function signature takes the *verified ceiling* — there is no
    // `device_cid` parameter, by construction: the COLLAPSE moved the
    // trust decision off the WriteContext device_cid field onto the
    // verified inbound ceiling at the single seam.
    let err = envelope_ceiling_admits_row(
        Some(&inbound_ceiling),
        "host:sandbox:exec:write",
        "host:sandbox:exec",
        "atrium-merge-row-605",
    )
    .expect_err(
        "COLLAPSE #605 REGRESSION: a runs_sandbox=false-attested inbound \
         writer was ADMITTED for a host:sandbox:* row at the sync-merge \
         seam — the device-grain trust enforcement that REPLACED the #605 \
         device_cid threading has been no-op'd. The #605 concern is closed \
         ONLY while the verified-ceiling AND is load-bearing at the single \
         seam (pim-2 §3.6b; impl-design §5 definitional collapse).",
    );

    assert_eq!(
        err.code(),
        benten_engine::ErrorCode::DeviceAttestationForged,
        "post-COLLAPSE device-grain trust rejection at the sync-merge seam \
         MUST surface E_DEVICE_ATTESTATION_FORGED; got {err:?}"
    );

    // Negative control: the same seam, a host:sandbox row, but the
    // inbound device's ceiling is NOT thin (runs_sandbox=true) ->
    // admitted. Proves the rejection above is the *ceiling*
    // discriminating per-device, not a blanket deny — i.e. the
    // per-device trust dispatch #605 worried was "collapsed to None"
    // is in fact alive and discriminating, just at the single seam.
    let full_ceiling = CapabilityEnvelope {
        runs_sandbox: true,
        holds_zones: ZoneScope::Full,
        online_uptime: UptimePolicy::AlwaysOn,
        runs_atrium_peer: true,
    };
    envelope_ceiling_admits_row(
        Some(&full_ceiling),
        "host:sandbox:exec:write",
        "host:sandbox:exec",
        "atrium-merge-row-605",
    )
    .expect(
        "a runs_sandbox=true inbound device ceiling MUST admit a \
         host:sandbox:* row — proving per-device trust dispatch is alive \
         at the single seam (the #605 'collapsed to None' concern closed)",
    );
}

/// **MANDATORY source-coupled regression-defense #605 / #707-trust
/// (would-FAIL-if-the-parallel-pipe-returns).**
///
/// The #605/#707 asymmetric-parallel-entry-point shape is closed
/// definitionally by routing the device-grain trust decision through
/// the SINGLE seam (`envelope_ceiling_admits_row`) inside
/// `apply_atrium_merge`'s ONE per-row loop. This pin asserts the
/// production source still does so: the per-row recheck must call
/// `envelope_ceiling_admits_row` (the single-seam ceiling-AND) and
/// must NOT have re-introduced a parallel `device_cid`-threaded trust
/// pipe at the synthetic-WriteContext site (which would recreate the
/// exact #707 asymmetric-parallel shape — a "fix" for #605 that
/// reverts COLLAPSE).
///
/// Mirrors the established `cap_r1_1_audience_binding_grep_defense.rs`
/// source-grep regression-defense idiom (an agent could regress the
/// wiring while preserving test names; this catches it at source).
#[test]
fn apply_atrium_merge_routes_device_trust_through_single_ceiling_seam() {
    let engine_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("engine.rs");
    let body = std::fs::read_to_string(&engine_src)
        .unwrap_or_else(|e| panic!("read {}: {e}", engine_src.display()));

    // 1. The single-seam ceiling-AND MUST be present in
    //    apply_atrium_merge's per-row path (the post-COLLAPSE
    //    device-grain trust enforcement that replaced #605's
    //    device_cid threading).
    assert!(
        body.contains("envelope_ceiling_admits_row"),
        "COLLAPSE #605/#707-trust REGRESSION: \
         crates/benten-engine/src/engine.rs no longer calls \
         `envelope_ceiling_admits_row` in apply_atrium_merge — the \
         single-seam device-grain trust enforcement that closes #605 \
         (replacing the device_cid=None WriteContext concern) has been \
         removed. Trust dispatch would silently revert to the demoted \
         provenance-label field (pim-2 §3.6b)."
    );

    // 2. The synthetic per-row recheck WriteContext MUST NOT have been
    //    "fixed" by threading device_cid as a trust input (recreating
    //    the #707 parallel pipe). Under COLLAPSE device_cid at this
    //    site stays the non-trust provenance-label residual (§4a /
    //    #1234) — the trust decision is the ceiling at the single
    //    seam. Assert the synthetic ctx still carries `device_cid:
    //    None` (provenance-label residual), NOT a re-threaded trust
    //    input, so the seam stays single (not parallel).
    assert!(
        body.contains("device_cid: None"),
        "COLLAPSE #605/#707-trust REGRESSION: the apply_atrium_merge \
         per-row synthetic WriteContext no longer carries `device_cid: \
         None`. Under the RATIFIED unified model the device-grain TRUST \
         decision is the verified-ceiling AND at the single seam; \
         re-threading device_cid as a trust input here recreates the \
         exact #707 asymmetric-parallel-entry-point the COLLAPSE deletes \
         (DECISION-RECORD §4a: device_cid is a provenance-label residual \
         for the #1234 successor audit, not a sync-merge trust input)."
    );
}
