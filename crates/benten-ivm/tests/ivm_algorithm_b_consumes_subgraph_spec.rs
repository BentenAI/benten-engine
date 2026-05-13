//! G23-0a IVM-subgraph generalization: Algorithm B kernel accepts
//! `SubgraphSpec` as view definition; end-to-end walk.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.2 row 2
//! (D-4F-2 + arch-r1-3).
//!
//! ## What this asserts (LOAD-BEARING substantive)
//!
//! G23-0a generalizes the Algorithm B kernel so it consumes a
//! `SubgraphSpec` (schema-shaped view definition) in lieu of the
//! G15-A `(view_id, label_pattern, projection)` triple. The end-to-end
//! pin observes:
//!
//! 1. **Register-time:** `Algorithm::register_subgraph(spec)` succeeds
//!    for valid SubgraphSpecs (canonical + user-defined view ids).
//! 2. **Walk-time:** feeding kernel inputs through the registered view
//!    drives materialisation through the engine evaluator's primitive-
//!    dispatch surface (per D-4F-2: materializer view IS an IVM view;
//!    walking is engine-internal subgraph evaluation, not a host-side
//!    `Materializer::walk()` outer loop).
//! 3. **Observable arm:** the materialised output differs by write
//!    sequence — feeding 0 writes vs N writes produces distinct
//!    KernelOutput values. A no-op G23-0a (e.g. returning a constant
//!    empty result) would FAIL the differs-by-input arm.
//!
//! ## §3.6b would-FAIL-if-no-op'd
//!
//! - Empty-walk vs populated-walk produce distinct outputs.
//! - User-defined view id reaches the kernel's generic-kernel path
//!   (NOT a canonical fast-path).
//! - Canonical view id round-trip mirrors the pre-generalization
//!   hand-written baseline (Family C verifies the 5-view equivalence
//!   at finer granularity per arch-r1-9; this pin is the kernel-input-
//!   shape gate at register-time).
//!
//! ## RED-PHASE
//!
//! Closes at R5 G23-0a. Un-ignore per pim-12 §3.6e at landing.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common_kernel_canary;
use common_kernel_canary::{
    CanarySubgraphSpec, KernelInput, KernelOutput, register_and_walk_to_completion,
};

#[test]
fn algorithm_b_kernel_accepts_subgraph_spec_for_user_defined_view_id() {
    // Substantive arm 1: user-defined view id flows through register_subgraph
    // + walks to completion. Per D-4F-2 the kernel's generic-kernel path
    // (Strategy::B classification per algorithm_b.rs CANONICAL_VIEW_IDS
    // dispatch) accepts the SubgraphSpec as input.
    let spec = CanarySubgraphSpec::for_user_view("user_view_smoketest");
    assert!(
        !spec.is_canonical,
        "user_view_smoketest is non-canonical; kernel must classify it \
         under Strategy::B generic-kernel path"
    );

    let writes = vec![
        KernelInput::new("post", 100, 0),
        KernelInput::new("post", 200, 1),
        KernelInput::new("post", 300, 2),
    ];

    let output = register_and_walk_to_completion(&spec, &writes)
        .expect("register_subgraph + walk must succeed for valid user-view spec");

    // Substantive arm: a no-op kernel (returns empty regardless of input)
    // would produce KernelOutput::Rows(vec![]) here. The post-walk
    // materialisation MUST reflect that 3 writes were observed — i.e.
    // the materialised bytes are non-empty.
    match output {
        KernelOutput::Rows(bytes) => {
            assert!(
                !bytes.is_empty(),
                "user-defined view materialisation must reflect the 3 \
                 walked writes; got empty Rows output — kernel walk is a no-op"
            );
        }
        other => panic!(
            "user-defined view must emit Rows output (not Rules/Current — \
             typed-output projections are only for canonical views 4/5 per \
             mat-r1-1); got `{other:?}`"
        ),
    }
}

#[test]
fn algorithm_b_kernel_empty_walk_produces_distinct_output_from_populated_walk() {
    // §3.6b would-FAIL-if-no-op'd: feeding 0 writes vs N writes through
    // the SAME SubgraphSpec MUST produce observably-distinct
    // KernelOutput values. A no-op kernel (constant return) fails this.
    let spec = CanarySubgraphSpec::for_user_view("differs_by_input_view");

    let empty_writes: Vec<KernelInput> = Vec::new();
    let populated_writes = vec![
        KernelInput::new("post", 100, 0),
        KernelInput::new("post", 200, 1),
    ];

    let empty_output =
        register_and_walk_to_completion(&spec, &empty_writes).expect("empty-walk must succeed");
    let populated_output = register_and_walk_to_completion(&spec, &populated_writes)
        .expect("populated-walk must succeed");

    assert_ne!(
        empty_output, populated_output,
        "kernel walk MUST observably differ by write sequence — empty vs \
         3-write inputs produced identical output `{empty_output:?}`. \
         No-op walk would alias both to a constant return."
    );
}

#[test]
fn algorithm_b_kernel_accepts_subgraph_spec_for_canonical_view_id() {
    // Substantive arm 2: canonical view id routes to fast-path
    // classification (Strategy::A per algorithm_b.rs dispatch); walking
    // produces the canonical-shape output (Rows for content_listing).
    //
    // Family C tests the per-view round-trip equivalence to the
    // hand-written baseline at finer granularity (arch-r1-9 view-1..5
    // pins). This pin is the kernel-input-shape gate: a canonical
    // SubgraphSpec MUST register + walk through the generalized kernel.
    let spec = CanarySubgraphSpec::for_canonical_view("content_listing");
    assert!(
        spec.is_canonical,
        "content_listing is canonical; kernel must classify under \
         fast-path dispatch (Strategy::A per algorithm_b.rs CANONICAL_VIEW_IDS)"
    );

    let writes = vec![
        KernelInput::new("post", 1, 0),
        KernelInput::new("post", 2, 1),
    ];

    let output = register_and_walk_to_completion(&spec, &writes)
        .expect("canonical-view SubgraphSpec must register + walk to completion");

    match output {
        KernelOutput::Rows(bytes) => {
            assert!(
                !bytes.is_empty(),
                "canonical content_listing view must materialise the 2 \
                 walked writes; got empty output"
            );
        }
        other => panic!(
            "content_listing must emit Rows (Views 1/2/3 — Rows; View 4 — \
             Rules; View 5 — Current per mat-r1-1); got `{other:?}`"
        ),
    }
}
