//! Phase-4-Meta-Core — ADDL R3 (TDD red-phase) — R3-W4 / G-CORE-8 / §8-E —
//! `CapabilityPolicy` SEALED-DISCIPLINE compile-test pin (§1.A.FROZEN
//! item 11 — CapabilityPolicy sealed for v1; on-surface per CLAUDE.md
//! baked-in #7 sealed-discipline refinement).
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
//! Pins: G-CORE-8 · §8-E · §1.A.FROZEN item 11 (CapabilityPolicy sealed
//! for v1; PUBLIC surface lock). R2 map: TF-8 F-4 sealed-discipline.

#![allow(unused_imports, dead_code, clippy::unwrap_used)]

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
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§8-E sealed-discipline — \
            external impl of CapabilityPolicy must be rejected by rustc \
            via a private Sealed supertrait; at HEAD c9c11c56 trait is \
            NOT sealed, so this pin asserts the missing-seal failure \
            mode + lands as a trybuild compile_fail pin at G-CORE-8)"]
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
    panic!(
        "§8-E sealed-discipline undelivered: at HEAD c9c11c56 the \
         `CapabilityPolicy` trait has NO Sealed supertrait (orchestrator-\
         ground-truth verified at `crates/benten-caps/src/policy.rs:261` \
         — `pub trait CapabilityPolicy: Send + Sync {{ ... }}`). The \
         shipped internal impl path is exercised above as substrate; \
         G-CORE-8 §8-E lands the private `Sealed` supertrait + the \
         compile_fail trybuild fixture rejecting external impls + \
         preserves the object-safety property exercised in the positive \
         arm above. The pseudo-fixture pattern is documented in this \
         test body for the G-CORE-8 implementer's reference."
    );
}

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
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§8-E three new hooks — \
            install-time consent / per-delegation runtime / audience- \
            aware check_write — all take the sealed-trait shape; this \
            pin holds the dispatch-primitive contract open until §8-E \
            wires the three hooks)"]
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
    // RED-arm: the three new hooks (install-time consent / per-
    // delegation runtime / audience-aware `check_write`) are UNBUILT
    // at HEAD. G-CORE-8 §8-E wires them; the sealed-trait shape
    // preserves the dispatch primitive exercised above.
    // -----------------------------------------------------------------
    panic!(
        "§8-E three new hooks undelivered: at HEAD c9c11c56 the \
         `CapabilityPolicy` trait carries `check_write` + `check_read` \
         only (no install-time-consent hook, no per-delegation-runtime \
         hook, no audience-aware check_write enrichment). G-CORE-8 \
         lands the three hooks under the sealed-discipline (object- \
         safety preserved per the positive arm above)."
    );
}
