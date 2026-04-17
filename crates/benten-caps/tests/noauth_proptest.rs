//! Phase 1 R3 security proptest — NoAuthBackend permits unconditionally.
//!
//! Attack class (adversarial coverage for the Phase 1 default): a mis-
//! configured `NoAuthBackend` that *accidentally* rejects valid writes (due
//! to unnamed edge-case handling — NaN in a property, absurdly long labels,
//! zero-length bytes, etc.) would break the Phase 1 DX promise AND create
//! a deceptive security posture: operators would see intermittent denials
//! and assume a policy is wired in when none is. The semantic of
//! NoAuthBackend is "zero authorization, zero friction" — ANY deviation is
//! a silent bug.
//!
//! This proptest exhaustively fuzzes `check_write` with structurally-valid
//! `WriteContext`s and asserts it returns `Ok(())` unconditionally.
//!
//! G4 mini-review (g4-cr-8): the second proptest (a "zero-alloc" assertion
//! that compared `alloc_count() before` vs `after`) was removed. The helper
//! returned a constant `0`, which made the assertion trivially `0 == 0` for
//! every generated context — false coverage. A real allocation counter
//! lands behind a `testing-alloc` feature flag in Phase 2 alongside the
//! global allocator decision (mimalloc vs snmalloc-rs).
//!
//! G4 mini-review (g4-cr-10): the redundant `target_label` field on
//! `WriteContext` was dropped; this generator now populates `label` only.
//!
//! Cross-refs:
//! - `.addl/phase-1/r1-security-auditor.json` finding #2 (critical — the
//!   broader concern is operator awareness; this proptest locks in the
//!   runtime contract)
//! - `.addl/phase-1/r2-test-landscape.md` §3 `prop_noauth_returns_ok_unconditionally`

#![allow(clippy::unwrap_used)]

use benten_caps::{CapabilityPolicy, NoAuthBackend, WriteContext};
use proptest::prelude::*;

/// Strategy generating arbitrary-but-structurally-valid WriteContexts. Any
/// context NoAuth rejects would be a bug.
fn any_write_context() -> impl Strategy<Value = WriteContext> {
    (
        proptest::string::string_regex("[a-z0-9:_-]{1,64}").unwrap(),
        any::<bool>(),
        proptest::option::of(proptest::string::string_regex("[a-z0-9]{4,32}").unwrap()),
    )
        .prop_map(|(label, is_priv, actor)| {
            let mut ctx = WriteContext::default();
            ctx.label = label;
            ctx.is_privileged = is_priv;
            ctx.actor_hint = actor;
            ctx
        })
}

proptest! {
    /// For every structurally-valid context, NoAuth must return Ok.
    /// If this ever fails to reduce to a concrete failing context, we have
    /// accidentally introduced a code path that deviates from the "allow
    /// everything" contract — that is a Phase 1 P0 bug.
    #[test]
    fn prop_noauth_returns_ok_unconditionally(ctx in any_write_context()) {
        let backend = NoAuthBackend::new();
        prop_assert!(
            backend.check_write(&ctx).is_ok(),
            "NoAuthBackend rejected a write — contract violation. ctx: {:?}",
            ctx
        );
    }
}
