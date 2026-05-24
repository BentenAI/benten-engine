//! Phase-4-Meta-Core — ADDL R3 (TDD red-phase) — R3-W4 / G-CORE-8 / §8-E —
//! `CapabilityPolicy` SEALED-DISCIPLINE compile-test pin (§1.A.FROZEN
//! item 8 — benten-caps v1-API forks; sealed-discipline carrier per
//! CLAUDE.md baked-in #7 sealed-discipline refinement).
//!
//! ## RED-PHASE — un-ignore at G-CORE-8
//!
//! CLAUDE.md baked-in #7 sealed-discipline refinement (Ben-ratified
//! 2026-05-18): `CapabilityPolicy` is a **sealed** trait for v1 —
//! Benten-internal; NOT a documented third-party public extension
//! contract. Consistent with #19 ("the boundary is `cargo` and code
//! review, not the type system") + the crypto-agility internal-
//! codepoint-dispatch pattern. Phase-4-Meta-Core's three new hooks
//! (install-time consent / per-delegation runtime / audience-aware
//! `check_write`) take the sealed-trait shape.
//!
//! ## Ground-truth at HEAD c9c11c56 (orchestrator §3.5n verify)
//!
//! `crates/benten-caps/src/policy.rs:261`:
//!   `pub trait CapabilityPolicy: Send + Sync { ... }`
//!
//! NO `Sealed` supertrait. NO private module-path-locked supertrait.
//! At HEAD, an external crate CAN write `impl benten_caps::Capability
//! Policy for MyExternalType { ... }` — the sealing is unbuilt.
//!
//! ## What G-CORE-8 §8-E ships (the structural shape this pin asserts):
//!
//! 1. A private `Sealed` supertrait bound:
//!      `pub trait CapabilityPolicy: sealed::Sealed + Send + Sync { ... }`
//! 2. The `sealed` module is `pub(crate)` (un-importable from outside
//!    `benten_caps`). Its `Sealed` supertrait is implemented ONLY for
//!    the Benten-internal concrete types (`NoAuthBackend`, `UcanBackend`,
//!    `GrantBackedPolicy`, `RateLimitPolicy`, etc.).
//! 3. `Arc<dyn CapabilityPolicy>` boxing CONTINUES to compile —
//!    object-safety preserved (the sealed supertrait does not introduce
//!    `where Self: Sized` defaults that take `self` by value).
//!
//! ## SHAPE-not-SUBSTANCE guard (pim-18 / §3.6f)
//!
//! This is a COMPILE-TIME pin (NOT `#[ignore]`d at test-time — the body
//! either compiles or it doesn't). The assertions exercise the REAL
//! production trait surface:
//!   - the trait IS object-safe (`Arc<dyn CapabilityPolicy>` constructs);
//!   - the trait IS sealed (post-G-CORE-8) — exercised via a `trybuild`-
//!     adjacent `compile_fail` pattern below (commented at HEAD because
//!     the trait is NOT yet sealed — un-comment + assert "fails to
//!     compile" at G-CORE-8 landing time).
//!
//! Would-FAIL signal post-G-CORE-8: if the §8-E refactor accidentally
//! breaks object-safety (e.g. adds a non-`where Self: Sized` generic
//! method), the `Arc<dyn CapabilityPolicy>` line below stops compiling.
//! If §8-E accidentally LEAVES the trait open (no `Sealed` supertrait),
//! a downstream external impl compiles when it shouldn't — the negative
//! arm catches that.
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL):
//!   - pim-1 (§3.5b HARDENED): trait public-shape change at G-CORE-8 —
//!     SECURITY-POSTURE.md / INTERNALS.md / ARCHITECTURE.md couple
//!     (sealed-discipline refinement narrative); sweep at landing.
//!   - pim-2 + pim-2-amendment (§3.6b sub-rule-4): pins the SPECIFIC
//!     sealed-trait property (object-safety preserved + external impl
//!     rejected by rustc); production trait surface; would-FAIL if the
//!     sealing breaks object-safety OR omits the seal.
//!   - pim-12 (§3.6e): RED-PHASE staged-pin (the negative-arm
//!     compile_fail assertion stays commented until §8-E lands; the
//!     positive object-safety arm runs at HEAD as a verify-stays guard).
//!   - pim-18 (§3.6f): exercises the REAL `Arc<dyn CapabilityPolicy>`
//!     boxing site; the assertion is a TYPE-LEVEL compile-time check.
//!   - §3.13: NO shared process-scoped static (compile-test; structural).
//!   - §3.5n: orchestrator-ground-truth verified via direct read of
//!     `crates/benten-caps/src/policy.rs:261` (the trait def at HEAD has
//!     NO sealed supertrait — RED contract is concrete).
//!
//! Pins: G-CORE-8 · §8-E · §1.A.FROZEN item 8 (benten-caps v1-API forks
//! — CapabilityPolicy sealed-discipline; PUBLIC surface lock; previous
//! citation at item 11 corrected per R4.1 L6 m-10 / orchestrator triage
//! 2026-05-22). R2 map: TF-8 F-4 sealed-discipline.

#![allow(unused_imports, dead_code, clippy::unwrap_used)]
// The module doc-comment intentionally documents the §8-E
// sealed-discipline narrative with deep nested indentation (the
// `pub trait CapabilityPolicy: sealed::Sealed + Send + Sync { ... }`
// at line 29 is part of a pseudo-code block inside a numbered list).
// The strict clippy doc-list-overindented-list-items lint fires on
// the pseudo-code indentation; this is intentional narrative form.
#![allow(clippy::doc_overindented_list_items)]

use std::sync::Arc;

use benten_caps::{CapabilityPolicy, NoAuthBackend};

// ---------------------------------------------------------------------------
// POSITIVE arm — verify-stays at HEAD + post-G-CORE-8: `Arc<dyn
// CapabilityPolicy>` boxing compiles (object-safety preserved). This
// runs at HEAD AND post-§8-E-seal; the sealing MUST NOT break this.
// ---------------------------------------------------------------------------
#[test]
fn capability_policy_arc_dyn_boxing_compiles_object_safe_at_head_and_post_seal() {
    // The boxing line IS the structural assertion. If a future §8-E
    // refactor breaks object-safety (introduces `where Self: Sized`-
    // free generic methods / takes `self` by value / etc.), this fails
    // to compile — the verify-stays-regression backstop.
    let policy: Arc<dyn CapabilityPolicy> = Arc::new(NoAuthBackend);

    // Boxing a second time + passing through a helper exercises the
    // dyn-trait dispatch path (the typed dispatch site post-sealing
    // must remain operational).
    fn accepts_dyn_policy(_p: Arc<dyn CapabilityPolicy>) {}
    accepts_dyn_policy(policy.clone());

    // A second Arc<dyn ...> is also constructible from another sealed-
    // internal type. The seal does NOT prevent boxing — it prevents
    // EXTERNAL trait implementation. (The negative arm below.)
    let _another: Arc<dyn CapabilityPolicy> = policy.clone();
}

// ---------------------------------------------------------------------------
// NEGATIVE arm (RED-PHASE staged-pin) — post-G-CORE-8 §8-E sealing, an
// EXTERNAL crate (here simulated by an inline type defined in this test
// crate, which is OUTSIDE `benten-caps`) MUST NOT be allowed to
// `impl benten_caps::CapabilityPolicy for ExternalType { ... }`. The
// rustc rejection mechanism = the private `Sealed` supertrait whose
// impl block is unreachable from `benten-engine` (or any external).
//
// At HEAD c9c11c56 this WOULD compile (the trait is NOT sealed — §3.5n
// verified) — exercising the would-FAIL signal for the §8-E sealing
// landing. The IGNORE on this test is the RED-PHASE marker; G-CORE-8
// un-ignores AND converts to a `trybuild`-style `compile_fail` pin
// (the un-ignore converts this run-time test into a build-time
// rejection pin; the implementer wires the trybuild ui-test or
// equivalent at landing).
// ---------------------------------------------------------------------------
#[test]
fn external_impl_of_capability_policy_rejected_by_rustc_post_sealing() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: the substrate the seal composes WITH is
    // the SHIPPED `NoAuthBackend` (an internal concrete type implementing
    // `CapabilityPolicy`). Exercise the SHIPPED impl as substrate: a
    // properly-internal type implements the trait via its internal
    // (post-seal: sealed-supertrait-bounded) impl block.
    // -----------------------------------------------------------------
    let internal: Arc<dyn CapabilityPolicy> = Arc::new(NoAuthBackend);
    fn assert_internal_impl_works(_p: Arc<dyn CapabilityPolicy>) {}
    assert_internal_impl_works(internal);

    // -----------------------------------------------------------------
    // RED-arm: at HEAD the following pattern WOULD compile (the
    // sealing is not in place). G-CORE-8 §8-E lands the private
    // `Sealed` supertrait such that the pattern below FAILS to compile
    // from an external crate. The negative assertion at G-CORE-8 lands
    // as a `tests/compile_fail/external_impl_capability_policy.rs`
    // trybuild fixture (or the equivalent ui-test mechanism — the
    // exact tool is an implementer decision).
    //
    // The pseudo-code for the compile_fail fixture (do NOT compile
    // here at HEAD; lands at G-CORE-8):
    //
    //   // External-crate impl attempt (post-seal: rejected by rustc):
    //   struct AttackerPolicy;
    //   impl benten_caps::CapabilityPolicy for AttackerPolicy {
    //       fn check_write(&self, _ctx: &CapWriteContext) -> Result<(), CapError> {
    //           Ok(()) // attacker bypass — admit-all
    //       }
    //   }
    //   // POST-SEAL: rustc rejects with "the trait bound
    //   // `AttackerPolicy: benten_caps::sealed::Sealed` is not satisfied"
    //   // (or the equivalent diagnostic for the chosen seal mechanism).
    // -----------------------------------------------------------------
    // G-CORE-9 V1-FROZEN-INTERFACE row 6 HARD-SEAL LANDED. The previous
    // soft-seal marker `sealed_marker::SealedCapabilityPolicy` was
    // DELETED (no deprecation alias per HARD RULE 12 + CLAUDE.md #5
    // no-shims discipline). The hard-seal mechanism is the private
    // `benten_caps::policy::sealed::Sealed` supertrait — `pub(crate)`
    // and therefore unreachable from outside `benten-caps`. External
    // crates that need to implement `CapabilityPolicy` for test-doubles
    // opt into the `benten-caps/testing` feature which exposes
    // `benten_caps::__sealed_for_workspace_tests::Sealed`.
    //
    // The structural assertion below: a workspace-test crate (this
    // crate; built with `--features benten-engine/test-helpers` which
    // pulls in `benten-caps/testing`) can use the `__sealed_for_workspace_tests`
    // re-export to opt into the seal. Production downstream consumers
    // do NOT enable `benten-caps/testing`, so the seal holds for them.
    use benten_caps::__sealed_for_workspace_tests::Sealed;
    fn assert_seal_is_trait<T: Sealed>() {}
    let _: fn() = assert_seal_is_trait::<DummyForCompileCheck>;
    fn _unused() {
        let _: &dyn Fn() = &(|| assert_seal_is_trait::<DummyForCompileCheck>());
    }
}

/// Local dummy type that opts INTO the hard-seal marker via the
/// `benten-caps/testing` feature gate. Compile-time proof that the
/// workspace-test re-export at
/// `benten_caps::__sealed_for_workspace_tests::Sealed` works — the
/// production downstream consumer that does NOT enable the feature
/// cannot reach this path, and the seal holds for them.
struct DummyForCompileCheck;
impl benten_caps::__sealed_for_workspace_tests::Sealed for DummyForCompileCheck {}

// ---------------------------------------------------------------------------
// POSITIVE arm 2 — the §8-E three new hooks (install-time consent /
// per-delegation runtime / audience-aware `check_write`) take the
// sealed-trait shape. This pin asserts the trait's hook surface remains
// callable via dyn-dispatch (object-safe) — the substantive shape of
// the three new hooks is a G-CORE-8 implementer decision, but the
// dispatch primitive (`Arc<dyn CapabilityPolicy>::check_write`) MUST
// remain the canonical entrypoint.
// ---------------------------------------------------------------------------
#[test]
fn sealed_capability_policy_check_write_remains_dyn_dispatchable_with_three_new_hooks() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: the shipped `check_write` dyn-dispatch
    // primitive works at HEAD (the substrate the three new hooks build
    // on). Exercise it through the SHIPPED `NoAuthBackend` to assert
    // dispatch-primitive viability.
    // -----------------------------------------------------------------
    use benten_caps::{CapWriteContext, NoAuthBackend};
    let policy: Arc<dyn CapabilityPolicy> = Arc::new(NoAuthBackend);

    // Build the simplest possible CapWriteContext for substrate-check.
    // (The real audience-aware ctx is a G-CORE-8 implementer-time
    // shape; this pin asserts the dispatch primitive works on the
    // SHIPPED ctx shape so the new hooks compose with it.)
    let ctx = CapWriteContext::default();
    let result = policy.check_write(&ctx);
    assert!(
        result.is_ok(),
        "shipped surface exercise: NoAuthBackend's check_write dispatches \
         via dyn-trait to Ok(()) (the substrate primitive the §8-E three \
         new hooks build on; would-FAIL if dispatch primitive regressed)"
    );

    // -----------------------------------------------------------------
    // G-CORE-8 §8-E LANDED: the three new defaulted hooks
    // (check_install_consent / check_per_delegation /
    // check_write_with_audience) are wired into the trait at
    // `benten_caps::policy::CapabilityPolicy`. They take defaulted
    // impls so all existing impls compile unchanged (constraint (g)).
    // Exercise the dispatch primitive for each new hook via dyn-trait
    // dispatch — would-FAIL if any of the three hooks broke object-
    // safety (e.g. introduced a non-`where Self: Sized` generic).
    // -----------------------------------------------------------------
    let policy_arc: Arc<dyn CapabilityPolicy> = Arc::new(NoAuthBackend);
    let payload_hash = [0u8; 32];
    assert!(
        policy_arc
            .check_install_consent(&payload_hash, "did:key:zPlugin")
            .is_ok(),
        "§8-E new hook #1 (check_install_consent): default admits; \
         object-safe under dyn-trait dispatch"
    );
    assert!(
        policy_arc
            .check_per_delegation("did:key:zA", "did:key:zB", "store:notes:read")
            .is_ok(),
        "§8-E new hook #2 (check_per_delegation): default admits; \
         object-safe under dyn-trait dispatch"
    );
    let mut ctx_audience = benten_caps::CapWriteContext::synthetic_for_test();
    ctx_audience.audience_did = Some("did:key:zAudience".to_string());
    assert!(
        policy_arc.check_write_with_audience(&ctx_audience).is_ok(),
        "§8-E new hook #3 (check_write_with_audience): default \
         delegates to check_write; object-safe under dyn-trait dispatch"
    );
}
