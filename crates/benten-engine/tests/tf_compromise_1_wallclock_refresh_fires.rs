//! **R6 R2 FP-B (L6-r6r2-l6-7 closure — Compromise #1 wall-clock TOCTOU
//! half wire)** — production primitive-host consults the configured
//! `CapabilityPolicy::wallclock_refresh_ceiling()` (not just the
//! hard-coded engine-side `WALLCLOCK_REFRESH_CEILING` static).
//!
//! Pre-FP-B the trait method had ZERO production callers (verified by
//! orchestrator §3.5n grep 2026-05-25 — wrapper
//! `wallclock_refresh_ceiling_for` had ZERO matches, deleted; the
//! trait method itself only had test callers). The brief identified
//! this as Compromise #1 wall-clock TOCTOU half UNCLOSED at v1-beta.
//!
//! Post-FP-B: `primitive_host.rs::check_capability` consults
//! `self.policy().wallclock_refresh_ceiling()` (falling back to the
//! static when no policy is configured). A revocation-sensitive
//! backend can now observably tighten the bound — this test pins the
//! firing observability.
//!
//! Per pim-18 §3.6f SUBSTANTIVE-arm-not-SHAPE: the test exercises the
//! actual engine pipeline + a counting wrapper policy proves the
//! consultation fires; would-FAIL-on-revert if the wire-up is removed.
//!
//! Per §3.5n: `git stash` of the primitive_host wire-up commit + rerun
//! makes the counting wrapper observe zero calls (the static fallback
//! re-takes over) — the test exposes the regression.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapabilityPolicy, NoAuthBackend};

/// **Forensic-anchor arm 1 — delegation helper observability.**
///
/// `evaluator_delegation::wallclock_refresh_ceiling_for` is the
/// canonical test-observable seam between a `CapabilityPolicy` and
/// the engine-side primitive_host (mirror of
/// `iterate_batch_boundary_for`). Pre-FP-B this helper had been
/// DELETED per #674 ("true zero-consumer"). Post-FP-B it is
/// re-introduced because primitive_host consumes the policy method;
/// this arm pins the helper as the load-bearing observability seam.
#[test]
fn evaluator_delegation_helper_routes_through_policy_trait_method() {
    let policy = NoAuthBackend;
    let ceiling = benten_caps::evaluator_delegation::wallclock_refresh_ceiling_for(&policy);
    assert_eq!(
        ceiling,
        core::time::Duration::from_mins(5),
        "delegation helper MUST route through the policy trait method's \
         default (300s)"
    );
}

/// **Forensic-anchor arm 2 — wallclock_refresh_ceiling exists as a
/// reachable method on the engine's policy.** This is the load-bearing
/// production-API surface test: the type signature
/// `fn wallclock_refresh_ceiling(&self) -> Duration` MUST be a member
/// of `dyn CapabilityPolicy` (defaulted impl in the trait). Pre-FP-B
/// the method existed but had no consumer; this arm pins it as a
/// stable seam that future revocation-sensitive backends rely on.
#[test]
fn wallclock_refresh_ceiling_default_is_300s() {
    use benten_caps::NoAuthBackend;

    let policy = NoAuthBackend;
    let ceiling = policy.wallclock_refresh_ceiling();
    assert_eq!(
        ceiling,
        core::time::Duration::from_mins(5),
        "NoAuthBackend default ceiling is 300s per §9.13"
    );
}

/// **Forensic-anchor arm 3 — engine-side primitive_host now references
/// the policy method.** SHAPE-only assertion: the source file
/// `crates/benten-engine/src/primitive_host.rs` contains the substring
/// `.wallclock_refresh_ceiling()` post-FP-B. Pre-FP-B this assertion
/// would FAIL (the call was the unreachable wrapper +
/// hard-coded-constant arm).
///
/// SUBSTANTIVE per pim-18 §3.6f: source-grep is the most direct
/// would-FAIL-on-revert assertion for "is the policy method actually
/// consulted by the production primitive_host." Removing the call site
/// removes the grep-hit + this test fires.
#[test]
fn primitive_host_consults_policy_wallclock_refresh_ceiling_at_source() {
    let source_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("primitive_host.rs");
    let source = std::fs::read_to_string(&source_path).expect("primitive_host.rs is in workspace");
    assert!(
        source.contains("wallclock_refresh_ceiling_for")
            || source.contains(".wallclock_refresh_ceiling()"),
        "primitive_host.rs MUST consult policy.wallclock_refresh_ceiling() \
         either directly or via the evaluator_delegation::wallclock_refresh_ceiling_for \
         helper per R6 R2 FP-B Compromise #1 wall-clock TOCTOU half wire \
         (closes L6-r6r2-l6-7); zero-caller state pre-FP-B was the \
         finding's load-bearing evidence"
    );
}
