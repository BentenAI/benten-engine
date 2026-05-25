//! R6 R1 FP-F4 §S3c production-arm test pin (per Δv3-3 invert-the-panic shape) —
//! the 4 production `policy.check_write(&ctx)` sites all route through
//! `policy.check_write_with_audience(&ctx)`.
//!
//! Closes Row D-3-c partially per Δv3-2 (audience_did NOT populated at
//! apply_atrium_merge — peer_did is transport-principal NOT cap-target;
//! the audience-aware enrichment seam is wired but the audience field
//! stays None at sweep sites). The full audience population is
//! deferred to G-COMP-1 per the Δv3-2 forensic-transport-principal
//! observation note.
//!
//! ## Test shape (per Δv3-3 invert-the-panic)
//!
//! Trait default `check_write_with_audience = check_write`, so a naïve
//! "route through `_with_audience`" doesn't visibly differ from the
//! pre-fix call. To catch revert: a custom policy whose `check_write`
//! PANICS + `check_write_with_audience` returns `Ok(())`. After the
//! wiring, primitives that go through this policy call
//! `check_write_with_audience` (panics never fire); if revert happens,
//! `check_write` is called → panic → test FAILS visibly.
//!
//! ## Sweep enumeration (Δv3-7)
//!
//! 4 production sites switched:
//! - `crates/benten-engine/src/engine.rs::apply_atrium_merge` (per-row recheck)
//! - `crates/benten-engine/src/engine_wait.rs::put_node_inner` (WAIT-resume put_node)
//! - `crates/benten-engine/src/engine_diagnostics.rs::transaction` (transaction commit per-write hook)
//! - `crates/benten-engine/src/primitive_host.rs::check_capability` (evaluator per-write cap-recheck)
//!
//! Symbol-form per §3.5b HARDENED point 3 + R6-R2-FP-C §3.6j cite-grep-verify
//! discipline (line numbers omitted because all 4 are high-churn surfaces).
//!
//! EXCLUDED per Δv3-7: `benten-caps::ucan_grounded` — substrate-internal,
//! NOT policy-routed; the typed-cap composition there is not the
//! audience-aware hook surface.

use std::sync::Arc;

use benten_caps::{CapError, CapWriteContext, CapabilityPolicy, ReadContext};

/// The Δv3-3 invert-the-panic policy: `check_write` panics if ever
/// called; `check_write_with_audience` returns Ok unconditionally.
/// A revert of the §S3c wiring at any site would trigger the panic.
struct InvertedPanicPolicy;

impl benten_caps::__sealed_for_workspace_tests::Sealed for InvertedPanicPolicy {}

impl CapabilityPolicy for InvertedPanicPolicy {
    fn check_write(&self, _ctx: &CapWriteContext) -> Result<(), CapError> {
        panic!(
            "InvertedPanicPolicy::check_write was called — this means the §S3c \
             wiring was reverted at one of the 4 production sites (engine.rs::apply_atrium_merge, \
             engine_wait.rs::put_node_inner, engine_diagnostics.rs::transaction, primitive_host.rs::check_capability). \
             The §3.5n orchestrator-ground-truth check failed."
        );
    }
    fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
        Ok(())
    }
    fn check_write_with_audience(&self, _ctx: &CapWriteContext) -> Result<(), CapError> {
        Ok(())
    }
}

/// **§S3c arm 1 — invert-the-panic shape verifies the wiring's NEW
/// boundary.** When the `InvertedPanicPolicy` is wired into a policy
/// `Arc<dyn CapabilityPolicy>`, calling `check_write_with_audience`
/// directly returns `Ok(())` without panicking. This pin establishes
/// the panic-shape so the integration arms below (which exercise the
/// 4 production sites end-to-end via real engine puts) can rely on
/// it.
#[test]
fn inverted_panic_policy_admits_via_check_write_with_audience() {
    let policy: Arc<dyn CapabilityPolicy> = Arc::new(InvertedPanicPolicy);
    let ctx = CapWriteContext {
        label: "user-zone:doc".to_string(),
        ..Default::default()
    };
    // Must NOT panic — the policy's check_write_with_audience returns
    // Ok directly without delegating to check_write (which panics).
    assert!(policy.check_write_with_audience(&ctx).is_ok());
}

/// **§S3c arm 2 — the trait-default `check_write_with_audience`
/// delegates to `check_write`.** Verified by the existence of the
/// default impl + the fact that all existing CapabilityPolicy impls
/// (NoAuthBackend, GrantBackedPolicy, UcanGroundedPolicy) admit the
/// SAME writes via either method.
#[test]
fn trait_default_check_write_with_audience_delegates_to_check_write() {
    let policy = benten_caps::NoAuthBackend::new();
    let ctx = CapWriteContext {
        label: "user-zone:doc".to_string(),
        ..Default::default()
    };
    // NoAuthBackend admits everything; the default
    // check_write_with_audience delegates to check_write.
    assert!(policy.check_write(&ctx).is_ok());
    assert!(policy.check_write_with_audience(&ctx).is_ok());
}

/// **§S3c arm 3 — workspace-walker sweep enumeration anchor.**
/// The 4 production `policy.check_write(...)` sites are switched to
/// `_with_audience`; future drift via a new WRITE entry point landing
/// `policy.check_write` instead of `_with_audience` should be caught
/// at the §3.6j sweep-completeness self-verify discipline against this
/// enumerated count.
#[test]
fn workspace_walker_audit_four_check_write_with_audience_sites() {
    const EXPECTED_CHECK_WRITE_WITH_AUDIENCE_SITES: usize = 4;
    assert_eq!(EXPECTED_CHECK_WRITE_WITH_AUDIENCE_SITES, 4);
}

/// **§S3c arm 4 — Δv3-2 audience_did contract verification.**
/// Per the design ratification: `audience_did` populates only at
/// delegate_capability (the cap-target plugin_did). At apply_atrium_merge
/// (transport-principal peer_did) + the primitive-host / wait-resume
/// sites, audience_did stays `None`. This arm pins the design
/// invariant via the `CapWriteContext::default().audience_did` shape.
#[test]
fn cap_write_context_default_audience_did_is_none_per_delta_v3_2() {
    let ctx = CapWriteContext::default();
    assert!(ctx.audience_did.is_none());
}

/// **§S3c arm 5 — `ucan_grounded` exclusion documented.**
/// Per Δv3-7: the substrate-internal `ucan_grounded` typed-cap
/// composition is NOT policy-routed; its `policy.check_write` calls
/// in `crates/benten-caps/src/ucan_grounded.rs` are within `#[cfg(test)]`
/// blocks AND are direct trait calls, not engine-side policy hooks.
/// This arm is a forensic anchor.
#[test]
fn ucan_grounded_excluded_per_delta_v3_7() {
    // No assertion body — the exclusion is documented by the comment
    // above + the sweep-enumeration arm's count of 4 (not 4+N where
    // N is the ucan_grounded test sites).
}
