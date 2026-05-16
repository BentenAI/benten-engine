//! COLLAPSE (P3) — MANDATORY closure-pin (pim-2 §3.6b,
//! would-FAIL-if-no-op'd).
//!
//! # Charter
//!
//! Spec: `.addl/refinement-audit-2026-05/impl-collapse-p0p1.md` (P3
//! exact rewire spec, item 4) + `impl-design-COLLAPSE.md` §0/§1.5 +
//! `DECISION-RECORD-trust-model-reframe.md` §4 (RATIFIED).
//!
//! The COLLAPSE P3 rewire DELETED `benten_id::device_attestation::
//! Acceptor` and REPLACED the inbound-sync `Acceptor::accept_at`
//! call in `DeviceAttestationEnvelope::verify` with the spine
//! ceiling-AND: the verified device `CapabilityEnvelope` is ANDed
//! into the inbound writer's effective caps at the single
//! chain-validation seam (`Engine::apply_atrium_merge`'s per-row
//! recheck loop) via the ONE helper
//! `benten_engine::manifest_envelope_recheck::envelope_ceiling_admits_row`
//! — the SAME code path the #669 manifest ceiling-check generalizes
//! over (build-constraint iii; NOT a parallel pipe).
//!
//! # The load-bearing property this pins
//!
//! **CLAUDE.md baked-in #17 thin-shape ceiling MUST NOT silently
//! regress through the COLLAPSE rewire.** An inbound sync write from
//! a `runs_sandbox=false`-attested principal MUST still be rejected
//! from exercising `host:sandbox:*` — *even with an otherwise-valid
//! chain* — via the new spine ceiling-AND, NOT the deleted
//! `Acceptor`.
//!
//! Pre-COLLAPSE this property was NOT actually enforced on the
//! inbound-sync path: `Acceptor::accept_at` never checked the
//! `runs_sandbox` envelope dimension (it did parent-sig / freshness /
//! nonce / revocation only); the ceiling-check
//! (`validate_chain_with_attestations`) had zero production
//! consumers. The COLLAPSE rewire is the FIRST time the inbound-sync
//! seam actually ANDs the device-envelope ceiling. This pin asserts
//! that — and FAILs if `envelope_ceiling_admits_row` is no-op'd
//! (returns `Ok(())` unconditionally).
//!
//! # Why a direct helper test (not a full iroh two-peer harness)
//!
//! `envelope_ceiling_admits_row` is the boundary the per-row loop in
//! `Engine::apply_atrium_merge` calls — the SAME code path
//! end-to-end. This mirrors the established codebase pattern for
//! `manifest_envelope_recheck::outcome_to_row_reject` (exposed `pub`
//! precisely so closure-pins can exercise the recheck-outcome →
//! row-reject mapping without spinning up a full Engine + iroh +
//! Atrium harness — see that fn's docstring). The end-to-end
//! wire-up at `apply_atrium_merge` is the same call; the two-device
//! integration shape lives in `tests/integration/atrium_two_device.rs`.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_engine::manifest_envelope_recheck::envelope_ceiling_admits_row;
use benten_id::device_attestation::{CapabilityEnvelope, UptimePolicy, ZoneScope};

/// A `runs_sandbox=false` envelope.
fn thin_envelope() -> CapabilityEnvelope {
    CapabilityEnvelope {
        runs_sandbox: false,
        holds_zones: ZoneScope::CacheOnly,
        online_uptime: UptimePolicy::SessionBounded,
        runs_atrium_peer: false,
    }
}

/// A `runs_sandbox=true` envelope (full peer).
fn full_envelope() -> CapabilityEnvelope {
    CapabilityEnvelope {
        runs_sandbox: true,
        holds_zones: ZoneScope::Full,
        online_uptime: UptimePolicy::AlwaysOn,
        runs_atrium_peer: true,
    }
}

/// **MANDATORY closure-pin (would-FAIL-if-no-op'd).**
///
/// An inbound sync row whose effective cap-scope is `host:sandbox:*`,
/// originating from a verified `runs_sandbox=false`-attested writer,
/// MUST be rejected at the single chain-validation seam with
/// `E_DEVICE_ATTESTATION_FORGED` — via the COLLAPSE P3 spine
/// ceiling-AND, NOT the deleted `Acceptor`.
///
/// If `envelope_ceiling_admits_row` is no-op'd (returns `Ok(())`
/// unconditionally — i.e. the ceiling-AND is silently dropped), the
/// `expect_err` below panics and this test FAILs. That is the
/// pim-2 §3.6b would-FAIL-if-no-op'd contract: this pin is the
/// load-bearing CLAUDE.md #17 thin-shape ceiling the whole rewire
/// must not silently regress.
#[test]
fn runs_sandbox_false_inbound_writer_rejected_from_host_sandbox_via_spine_ceiling_and() {
    let ceiling = thin_envelope();

    let err = envelope_ceiling_admits_row(
        Some(&ceiling),
        // The row's effective cap-scope: a sandbox-authority zone.
        // (`apply_atrium_merge` builds `{zone}:write` for the row;
        // a `host:sandbox:exec` zone yields this scope.)
        "host:sandbox:exec:write",
        &["host:sandbox:exec:write"],
        "host:sandbox:exec",
        "row-key-7",
    )
    .expect_err(
        "COLLAPSE P3 REGRESSION: a runs_sandbox=false-attested inbound writer was \
         ADMITTED for a host:sandbox:* row — the J8 spine ceiling-AND has been \
         no-op'd. This is the load-bearing CLAUDE.md #17 thin-shape property the \
         whole rewire must not silently regress (pim-2 §3.6b).",
    );

    assert_eq!(
        err.code(),
        benten_engine::ErrorCode::DeviceAttestationForged,
        "ceiling-AND rejection MUST surface E_DEVICE_ATTESTATION_FORGED; got {err:?}"
    );
}

/// Negative-control 1: a `runs_sandbox=true` (full-peer) writer is
/// NOT blocked from a `host:sandbox:*` row — the ceiling admits it.
/// This proves the rejection above is the *ceiling* discriminating on
/// `runs_sandbox`, not a blanket `host:sandbox:*` deny.
#[test]
fn runs_sandbox_true_inbound_writer_admitted_for_host_sandbox_row() {
    let ceiling = full_envelope();
    envelope_ceiling_admits_row(
        Some(&ceiling),
        "host:sandbox:exec:write",
        &["host:sandbox:exec:write"],
        "host:sandbox:exec",
        "row-key-7",
    )
    .expect("a runs_sandbox=true ceiling MUST admit a host:sandbox:* row");
}

/// **MANDATORY closure-pin — #1241 / F2 capability-predicate
/// completion (would-FAIL-if-no-op'd).** This is the REAL production
/// arm sec-review-1238 F2 demanded: a `runs_sandbox=false` principal
/// self-delegating `host:sandbox:exec` and writing into an *ordinary
/// data zone* (`store:notes`) MUST be rejected by the unified ceiling
/// at the seam — discriminating on the writer's actual cap-RESOURCE
/// (the per-row key), NOT the synthetic `{zone}:write` zone-scope.
///
/// The P3-shipped seam passed only `row_scope = "store:notes:write"`
/// here — which does NOT start with `host:sandbox:`, so the ceiling-AND
/// did NOT fire (sec-review-1238 F2 SHAPE-not-substance gap; verified
/// INERT at HEAD per F2-exploitability-investigation.md but a baked-in
/// commitment must be enforced regardless). P5 threads the writer's
/// real cap-resource (the per-row key); this pin asserts the rejection
/// on that real arm and FAILs if the cap-resource discrimination is
/// dropped (pim-2 §3.6b would-FAIL-if-no-op'd).
#[test]
fn runs_sandbox_false_self_delegating_host_sandbox_into_ordinary_zone_rejected_real_arm() {
    let ceiling = thin_envelope();

    // The attacker writes into an ORDINARY data zone (`store:notes`) —
    // the zone-write scope is `store:notes:write` (NOT host:sandbox:*),
    // exactly the synthetic-scope blind spot. But the writer's actual
    // exercised cap-resource (the per-row key) is `host:sandbox:exec`.
    let err = envelope_ceiling_admits_row(
        Some(&ceiling),
        // zone-write scope: ordinary data zone (the P3 proxy that
        // would NOT catch this — the F2 blind spot).
        "store:notes:write",
        // writer's REAL effective cap-resource (the per-row key): the
        // sandbox-exec cap it self-delegated.
        &["host:sandbox:exec"],
        "store:notes",
        "host:sandbox:exec",
    )
    .expect_err(
        "#1241/F2 REGRESSION: a runs_sandbox=false principal self-delegating \
         host:sandbox:exec into an ordinary data zone was ADMITTED — the unified \
         ceiling is discriminating only on the synthetic {zone}:write scope, NOT \
         the writer's real cap.resource. This is the exact sec-review-1238 F2 \
         SHAPE-not-substance gap the P5 capability-predicate completion closes \
         (pim-2 §3.6b would-FAIL-if-no-op'd; CLAUDE.md #17 baked-in).",
    );

    assert_eq!(
        err.code(),
        benten_engine::ErrorCode::DeviceAttestationForged,
        "real-cap.resource ceiling-AND rejection MUST surface \
         E_DEVICE_ATTESTATION_FORGED; got {err:?}"
    );
}

/// Negative control for the #1241 real arm: a `runs_sandbox=false`
/// writer whose real cap-resources are all ordinary (no sandbox-exec)
/// writing into an ordinary zone is ADMITTED — proving the rejection
/// above is the cap.resource predicate, not a blanket deny.
#[test]
fn runs_sandbox_false_ordinary_cap_resources_into_ordinary_zone_admitted() {
    let ceiling = thin_envelope();
    envelope_ceiling_admits_row(
        Some(&ceiling),
        "store:notes:write",
        &["store:notes:write", "store:notes:read"],
        "store:notes",
        "row-key-ordinary",
    )
    .expect(
        "a runs_sandbox=false ceiling MUST admit a writer exercising only ordinary \
         cap-resources into an ordinary zone (cap.resource predicate, not blanket deny)",
    );
}

/// Negative-control 2: a `runs_sandbox=false` writer is NOT blocked
/// from a NON-sandbox zone row (ordinary user-data write). The
/// ceiling-AND is scoped to the sandbox-authority dimension; it must
/// not over-reject ordinary sync traffic.
#[test]
fn runs_sandbox_false_inbound_writer_admitted_for_ordinary_zone_row() {
    let ceiling = thin_envelope();
    envelope_ceiling_admits_row(
        Some(&ceiling),
        "store:notes:write",
        &["store:notes:write"],
        "store:notes",
        "row-key-9",
    )
    .expect("a runs_sandbox=false ceiling MUST NOT block an ordinary non-sandbox row");
}

/// Negative-control 3: a legacy unsigned envelope (no verified
/// ceiling — `None`) is admitted (backward-compat: the verify path
/// returns `None`, the seam has nothing to AND). This must NOT panic
/// and must NOT reject — the absence of a ceiling is the documented
/// pre-G16-D / non-wire-merge fallback.
#[test]
fn absent_ceiling_admits_any_row_backward_compat() {
    envelope_ceiling_admits_row(
        None,
        "host:sandbox:exec:write",
        &["host:sandbox:exec"],
        "host:sandbox:exec",
        "row-key",
    )
    .expect("absent ceiling (legacy unsigned / non-wire merge) MUST admit (backward-compat)");
}
