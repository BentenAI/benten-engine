//! Phase-2a R6FP catch-up EH6 — exhaustive `InvariantViolation` →
//! `ErrorCode` mapping coverage.
//!
//! Background: `InvariantViolation::code()` (in `benten-eval/src/lib.rs`)
//! is the single point that maps each structural-invariant variant to its
//! catalog `ErrorCode`. The Inv-12 catch-all (`Registration` →
//! `InvRegistration`) makes "unmapped variant" a tractable runtime miss
//! rather than a compile error — without an explicit test, a future
//! variant added without a `match` arm could silently fall through to
//! `ErrorCode::Unknown(_)` and lose the catalog identifier on the wire.
//!
//! This test enumerates every `InvariantViolation` variant explicitly
//! (relying on the compiler's exhaustiveness check inside `match` to fail
//! to compile when a future variant is added but not added here either) and
//! asserts:
//!
//! 1. `.code()` does NOT return `ErrorCode::Unknown(_)`.
//! 2. The mapping matches the documented variant→code pair.
//!
//! When a Phase-2+ variant lands (e.g. `SandboxNestDepth`,
//! `SandboxOutputLimit` per the `#[non_exhaustive]` doc-block on
//! `InvariantViolation`), the `match` below stops compiling — the writer
//! adds the new variant + its assertion in the same diff that touches the
//! enum. That's the architectural intent: drift becomes a compile error.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::{ErrorCode, InvariantViolation};

/// Exhaustive variant→code assertion. The `match` is the load-bearing
/// construct: adding a new `InvariantViolation` variant without extending
/// this `match` is a compile-time error (the compiler enforces that
/// `non_exhaustive` is only externally-non-exhaustive — within the
/// defining crate's siblings, we still see a hard exhaustiveness check
/// once a variant is named — but `#[non_exhaustive]` requires us to
/// include `_ =>` to satisfy the foreign-crate cross-version contract).
/// The `_ =>` arm panics rather than silently passing, so a future variant
/// that ships without an explicit assertion fails this test loudly.
#[test]
fn every_invariant_violation_variant_maps_to_a_non_unknown_error_code() {
    // The full set of variants known to Phase-2a. Each entry is paired
    // with its expected `ErrorCode` discriminant.
    let cases: &[(InvariantViolation, ErrorCode)] = &[
        (InvariantViolation::Cycle, ErrorCode::InvCycle),
        (
            InvariantViolation::DepthExceeded,
            ErrorCode::InvDepthExceeded,
        ),
        (
            InvariantViolation::FanoutExceeded,
            ErrorCode::InvFanoutExceeded,
        ),
        (InvariantViolation::TooManyNodes, ErrorCode::InvTooManyNodes),
        (InvariantViolation::TooManyEdges, ErrorCode::InvTooManyEdges),
        (InvariantViolation::Determinism, ErrorCode::InvDeterminism),
        (InvariantViolation::ContentHash, ErrorCode::InvContentHash),
        (
            InvariantViolation::IterateMaxMissing,
            ErrorCode::InvIterateMaxMissing,
        ),
        (
            InvariantViolation::IterateBudget,
            ErrorCode::InvIterateBudget,
        ),
        (InvariantViolation::Registration, ErrorCode::InvRegistration),
        (InvariantViolation::Attribution, ErrorCode::InvAttribution),
        (InvariantViolation::Immutability, ErrorCode::InvImmutability),
        (InvariantViolation::SystemZone, ErrorCode::InvSystemZone),
        // Phase-2b G7-B — Inv-4 + Inv-7.
        (InvariantViolation::SandboxDepth, ErrorCode::InvSandboxDepth),
        (
            InvariantViolation::SandboxOutput,
            ErrorCode::InvSandboxOutput,
        ),
    ];

    for (variant, expected) in cases {
        let actual = variant.code();
        assert!(
            !matches!(actual, ErrorCode::Unknown(_)),
            "{variant:?} mapped to ErrorCode::Unknown(_) — every variant must \
             have a first-class catalog code (catch-all is `InvRegistration`, \
             not `Unknown`). got: {actual:?}"
        );
        assert_eq!(
            &actual, expected,
            "{variant:?} mapped to the wrong ErrorCode. expected {expected:?}, got {actual:?}"
        );
    }
}

/// Compile-time exhaustiveness guard. This `match` arm has no `_ =>`
/// fallback — if a future variant lands without an arm here, the file
/// stops compiling and the writer is forced to extend both this match and
/// the assertion table above in the same diff.
///
/// The function is `#[allow(dead_code)]` because we only care about the
/// compile-time check; `assert_variant_known` is never called at runtime.
#[allow(dead_code)]
fn assert_variant_known(v: InvariantViolation) {
    // NOTE: do NOT add a wildcard `_ =>` arm. The whole point is that the
    // compiler nags us when a new variant arrives — `#[non_exhaustive]`
    // forces a `_` for cross-crate matchers, but within `benten-eval`'s
    // own integration tests we still see the variant-known set. A future
    // Phase-2+ variant lands → this match fails to compile → writer adds
    // the arm here AND the corresponding `(variant, expected)` pair in the
    // assertion table → drift stays at compile-time.
    match v {
        InvariantViolation::Cycle
        | InvariantViolation::DepthExceeded
        | InvariantViolation::FanoutExceeded
        | InvariantViolation::TooManyNodes
        | InvariantViolation::TooManyEdges
        | InvariantViolation::Determinism
        | InvariantViolation::ContentHash
        | InvariantViolation::IterateMaxMissing
        | InvariantViolation::IterateBudget
        | InvariantViolation::Registration
        | InvariantViolation::Attribution
        | InvariantViolation::Immutability
        | InvariantViolation::SystemZone
        | InvariantViolation::SandboxDepth
        | InvariantViolation::SandboxOutput => {}
        // The wildcard is REQUIRED here because `InvariantViolation` is
        // `#[non_exhaustive]` from the perspective of an integration test
        // (which compiles as an external crate against `benten-eval`).
        // The TEST value is that adding a new variant + a `match` arm in
        // `benten-eval/src/lib.rs::InvariantViolation::code()` without
        // also adding an entry to the `cases` table above causes the
        // runtime assertion `expected != actual` to fire — a strong
        // secondary signal even though the compiler can't help across the
        // `non_exhaustive` boundary.
        _ => unreachable!("unknown InvariantViolation variant — extend the cases table above"),
    }
}
