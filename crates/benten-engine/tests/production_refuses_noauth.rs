//! Phase 1 R3 security test — `Engine::builder().production()` refuses NoAuth (R1 SC2).
//!
//! Attack class: the silent-default misuse bug. `NoAuthBackend` is the
//! Phase 1 builder default (required for the 10-minute DX path). That's
//! fine for embedded/local-only use — dangerous for networked/multi-tenant
//! deployments. The R1 security-auditor named this as a critical concern:
//! Elasticsearch / MongoDB style default-no-auth is a headline breach
//! generator.
//!
//! R1 triage SC2 disposition:
//!   (a) Info-log on startup when NoAuth is in use (noisy, but one-shot).
//!   (b) `Engine::builder().production()` constructor that REFUSES to build
//!       without an explicit capability policy — opt-in safer path for
//!       production users.
//!   (c) New doc `docs/SECURITY-POSTURE.md` naming the semantics.
//!
//! This file tests (b): the production builder must return an error with
//! the stable `EngineError::NoCapabilityPolicyConfigured` variant if a
//! user calls `.production()` without providing a policy. It must accept
//! any EXPLICIT policy — including NoAuth chosen deliberately — so
//! operators aren't forced to hand-roll one to use the strict builder.
//!
//! TDD contract: FAIL at R3. R5 lands the production builder method +
//! the `NoCapabilityPolicyConfigured` variant.
//!
//! Cross-refs:
//! - `.addl/phase-1/r1-security-auditor.json` finding #2 (critical)
//! - `.addl/phase-1/r1-triage.md` SC2 disposition
//! - `.addl/phase-1/r2-test-landscape.md` §2.6 `Engine::builder().production()`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{NoAuthBackend, UCANBackend};
use benten_engine::{Engine, EngineError};

/// The guard test: calling `.production()` on the builder without a policy
/// must fail at BUILD time, not at first-write time. Fail-early is the
/// whole point of the constructor.
#[test]
fn engine_builder_production_refuses_noauth() {
    let dir = tempfile::tempdir().unwrap();
    let result = Engine::builder()
        .production()
        .open(dir.path().join("benten.redb"));

    let err = result.expect_err(
        "Engine::builder().production() without an explicit capability \
         policy must refuse to build",
    );

    assert!(
        matches!(err, EngineError::NoCapabilityPolicyConfigured),
        "production-builder rejection must surface the stable \
         `NoCapabilityPolicyConfigured` variant so operators can match \
         on it in onboarding scripts. got: {err:?}"
    );
}

/// The inverse: `.production()` WITH an explicit policy must succeed even
/// if that policy is `NoAuthBackend`. Operators who knowingly choose
/// NoAuth must be able to opt in without fighting the guardrail — the
/// guardrail exists to stop ACCIDENTAL NoAuth, not deliberate NoAuth.
#[test]
fn engine_builder_production_accepts_explicit_noauth() {
    let dir = tempfile::tempdir().unwrap();
    let result = Engine::builder()
        .capability_policy(Box::new(NoAuthBackend::new()))
        .production()
        .open(dir.path().join("benten.redb"));
    assert!(
        result.is_ok(),
        "explicit NoAuth is a valid deliberate choice; production() must \
         accept it (the guardrail is against *silent* NoAuth, not \
         *chosen* NoAuth). got: {:?}",
        result.err()
    );
}

/// Third: an explicit non-NoAuth policy (e.g. `UCANBackend`, even though
/// it's Phase 1 stub) also satisfies the guard. This proves the guard is
/// "any policy set" not "specifically not-NoAuth-by-type-name".
#[test]
fn engine_builder_production_accepts_ucan_stub() {
    let dir = tempfile::tempdir().unwrap();
    let result = Engine::builder()
        .capability_policy(Box::new(UCANBackend::new()))
        .production()
        .open(dir.path().join("benten.redb"));
    // The engine opens fine; the UCANBackend will fail at first write with
    // E_CAP_NOT_IMPLEMENTED (see `ucan_stub_messages.rs`) — but build
    // succeeds, which is what this test locks in.
    assert!(result.is_ok(), "UCANBackend is a valid explicit policy");
}

/// NON-production path retains the permissive default — don't regress the
/// 10-minute DX. If THIS test fails, someone moved the guard from
/// `.production()` onto the common builder path and broke the embedded
/// user experience.
#[test]
fn engine_builder_default_path_still_permits_silent_noauth() {
    let dir = tempfile::tempdir().unwrap();
    let result = Engine::builder().open(dir.path().join("benten.redb"));
    assert!(
        result.is_ok(),
        "the default builder path must still work with implicit NoAuth — \
         breaking this breaks the Phase 1 QUICKSTART. The guard is on \
         `.production()`, NOT on the default builder."
    );
}
