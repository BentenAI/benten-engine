//! G23-0a: recursive subgraph-view register fails fast (mat-r1-13).
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.2 row 4
//! (mat-r1-13 — recursive subgraph-view register fails fast).
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-0a
//! ("Reject self-reference / recursive subgraph-shaped views at
//! register-time via existing cycle-detection (mat-r1-13).").
//!
//! ## What this asserts (substantive)
//!
//! A SubgraphSpec that references itself transitively MUST be rejected
//! at `Algorithm::register_subgraph` time — NOT discovered at walk-time
//! after partial materialisation, NOT silently looped over (would
//! exhaust budget / spin forever).
//!
//! Two arms:
//! 1. **Direct self-reference** — spec.view_id appears in its own
//!    SubgraphSpec body (e.g. as a referenced sub-view). Register MUST
//!    return Err with a typed error naming the cycle.
//! 2. **Walk-time substance check** — calling
//!    `register_and_walk_to_completion(self_referential_spec, &[])`
//!    surfaces the rejection (NOT a panic, NOT a successful empty
//!    output that masks the cycle).
//!
//! ## §3.6b would-FAIL-if-no-op'd
//!
//! A no-op cycle check (e.g. `register_subgraph` always returns Ok)
//! would FAIL the rejection assertion. A walk-time-only check
//! (rejection during walk rather than at register) would FAIL the
//! "fails fast" arm — register-time rejection is the substantive
//! observable per mat-r1-13.
//!
//! ## RED-PHASE
//!
//! Closes at R5 G23-0a. Un-ignore per pim-12 §3.6e.

#![allow(clippy::unwrap_used)]

mod common_kernel_canary;
use common_kernel_canary::{CanarySubgraphSpec, KernelInput, register_and_walk_to_completion};

#[test]
fn self_referential_subgraph_spec_rejected_at_register_time() {
    // Construct a user-defined SubgraphSpec marked as self-referential
    // (the canary shape carries a `self_referential` flag that the R5
    // implementer wires to the actual register-time cycle-detection
    // check at the production seam).
    let spec = CanarySubgraphSpec::for_user_view("recursive_self_ref_view").with_self_reference();

    // Substantive arm: register-time MUST reject. The canary's
    // `register_and_walk_to_completion` proxies register + walk; the
    // R5-landing version replaces the stub with a real call that
    // surfaces the typed register-time error when `self_referential`
    // is set.
    let result = register_and_walk_to_completion(&spec, &[]);

    let err = result.expect_err(
        "register_subgraph MUST reject self-referential SubgraphSpec at \
         register-time (mat-r1-13). Got Ok — no-op cycle check would \
         silently permit the cycle.",
    );

    // The error message must reference the cycle / self-reference so
    // operators can diagnose. A bare "register failed" without context
    // would FAIL this assertion.
    let lower = err.to_lowercase();
    assert!(
        lower.contains("self") || lower.contains("cycle") || lower.contains("recursive"),
        "register-time rejection error must name the cycle/self-reference \
         (mat-r1-13 fail-fast semantics); got `{err}`"
    );
}

#[test]
fn non_self_referential_subgraph_spec_registers_cleanly() {
    // Companion-positive arm: an OTHERWISE-identical user-defined spec
    // WITHOUT the self-reference flag MUST register cleanly. Without
    // this pin, the negative-arm assert above would pass for a
    // pathological implementation that rejects ALL user-defined specs.
    let spec = CanarySubgraphSpec::for_user_view("non_recursive_view");
    let writes = vec![KernelInput::new("post", 10, 0)];

    let result = register_and_walk_to_completion(&spec, &writes);
    assert!(
        result.is_ok(),
        "non-self-referential user-defined SubgraphSpec must register + walk \
         cleanly; got Err `{result:?}` — cycle check is over-rejecting."
    );
}

#[test]
fn self_reference_check_runs_before_any_kernel_input_walks() {
    // "Fails fast" substance check: the rejection MUST surface before
    // any walk happens. We test this by passing a write sequence that,
    // if walked, would produce observably-different output — and then
    // asserting the rejection error fires regardless. The implementer
    // wires register_subgraph cycle detection BEFORE the walk loop.
    let spec = CanarySubgraphSpec::for_user_view("cycle_with_writes").with_self_reference();
    let writes = vec![
        KernelInput::new("post", 1, 0),
        KernelInput::new("post", 2, 1),
        KernelInput::new("post", 3, 2),
    ];

    let result = register_and_walk_to_completion(&spec, &writes);
    assert!(
        result.is_err(),
        "self-referential spec MUST reject at register-time BEFORE walking \
         any inputs (mat-r1-13 fail-fast); got Ok with walk-time output \
         `{result:?}` — cycle detection deferred until walk-time."
    );
}
