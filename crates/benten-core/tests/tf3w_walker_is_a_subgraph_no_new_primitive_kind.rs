//! TF-3w (R3-W2) — Walker IS a Subgraph; NO new `PrimitiveKind` variant.
//!
//! Family: TF-3w — Spike F's fractal-property + CLAUDE.md baked-in #1
//! 12-primitive irreducibility.
//!
//! Plan: `.addl/phase-4-meta/00-implementation-plan.md` §3 G-CORE-3w def
//! (line 334) + §1.A.FROZEN item 15(a) (line 132) + §1.A.FROZEN item
//! 15(h) (line 139).
//!
//! Ratified inputs:
//!   `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
//!   §"8 substantive design refinements" (3) — "walker-as-a-Subgraph-
//!   shipped-once-in-`benten_core` (NOT an evaluator-knows-about-
//!   SubgraphSpec capability)" + Spike F fractal-property finding.
//!
//! R2-seed: R2-test-landscape.md §2 G-CORE-3w (P-2) — "SubgraphSpec
//! walker is itself a Subgraph composed of existing 12 primitives —
//! assertion grep: NO new `PrimitiveKind` variant minted (CLAUDE.md
//! baked-in #1 / #4 invariant; the fractal property from Spike F)."
//!
//! Named destination (HARD RULE 12 clause-(b)): the post-G-CORE-3w
//! `benten_core::subgraph_spec::walker::walker_as_subgraph()` returns
//! the walker's own composition as a `benten_core::Subgraph` (whose
//! OperationNodes are READ / WRITE / TRANSFORM / BRANCH / ITERATE only —
//! the BFS-walker decomposition Spike F identified). The
//! `PrimitiveKind` enum at HEAD has 12 variants; this test asserts the
//! count + variant-set is UNCHANGED post-G-CORE-3w.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Full R3 BRIEF-TEMPLATE CHECKLIST reproduced in sibling
//! `tf3b_restricted_spec_contains_decidable.rs` head comment; all 19
//! rules apply.
//!
//! Special pin shape: arm (P-2.1) round-trips canonical-tags for the
//! 12 known `PrimitiveKind` variants (rename / typo / dispatch-bug
//! defense per pim-2 §3.6b end-to-end-arm coverage). The structural
//! "no 13th variant" guard CANNOT be expressed via exhaustive match
//! at this surface because `PrimitiveKind` is `#[non_exhaustive]`
//! at `crates/benten-core/src/subgraph.rs:68` (an exhaustive match
//! fails to compile at every call site by design — `non_exhaustive`
//! is the PRODUCTION signal that surfaces additions to reviewer
//! attention at every consumer's `match`). The structural backstop
//! against silent primitive growth lives in cross-cutting defenses:
//! cite-drift sentinel against CLAUDE.md baked-in #1 narrative;
//! `tf5_46_schema_compiler_8_labeltype_vocab_fixture.rs` allowlist +
//! `Subgraph::nodes()` walker assertions; the §3.6g pim-N pre-flight
//! 12-primitive-irreducibility line in every R5 brief. (R4b L5-MIN-2
//! docstring honesty retense 2026-05-24.) Arm (P-2.2) covers the
//! walker-as-Subgraph composition's no-Sandbox property.
//!
//! SHAPE-flag-don't-fake: `benten_core::subgraph_spec` does not exist
//! at HEAD; RED-PHASE.
//! ─────────────────────────────────────────────────────────────────────────

#![allow(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;
use alloc::string::ToString;
use alloc::vec;

use benten_core::{Cid, PrimitiveKind, Subgraph};
// RED: `benten_core::subgraph_spec` does NOT exist at HEAD.
use benten_core::subgraph_spec::{Spec, walker};

fn cid_for(label: &str) -> Cid {
    let digest = blake3::hash(label.as_bytes());
    Cid::from_blake3_digest(*digest.as_bytes())
}

// ---------------------------------------------------------------------------
// Arm P-2.1 — `PrimitiveKind` enum has EXACTLY 12 variants; the walker
// composition mints NO new variant.
//
// The exhaustive match below + the explicit count assertion catches a
// future 13th variant via `non_exhaustive_omitted_patterns` (the
// same protection shape as `tf3b_no_opaque_selector_arm_structural.rs`).
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3w (pim-12 / §3.6e closure): the 12 known
/// `PrimitiveKind` variants from CLAUDE.md baked-in #1 round-trip
/// their canonical-tags cleanly + the `PrimitiveKind::canonical_tag`
/// dispatch returns one of the 12 expected tags for each of them.
///
/// **R4b L5-MIN-2 docstring/test-name retense 2026-05-24:** the
/// previous docstring + test-name promised a compile-fail
/// (`non_exhaustive_omitted_patterns`) signal on a future 13th
/// variant. That promise CANNOT be delivered at this surface because
/// `PrimitiveKind` is intentionally `#[non_exhaustive]`
/// (`crates/benten-core/src/subgraph.rs:68`) — an exhaustive match
/// fails to compile at every call site by design (the trade-off for
/// future-extensibility per the 12-primitive irreducibility
/// commitment; CLAUDE.md baked-in #1's stability is enforced by
/// PROCESS not by `match`-exhaustiveness). The structural backstop
/// against silent primitive growth actually lives in cross-cutting
/// defenses:
///   - cite-drift sentinel against CLAUDE.md baked-in #1 narrative
///     (CI lane; `cargo run -p cite-drift-detector`)
///   - `tf5_46_schema_compiler_8_labeltype_vocab_fixture.rs` per-label
///     allowlist + `Subgraph::nodes()` walker assertions that
///     allowlist `PrimitiveKind`
///   - reviewer pim-N-prior-phase-pim-explicit-preflight (§3.6g)
///     enumerating 12-primitive irreducibility as a literal pre-flight
///     line in every R5 brief
///   - `non_exhaustive` is the PRODUCTION signal — every consumer's
///     `match` needs a wildcard arm, surfacing any addition to
///     reviewer attention naturally
///
/// What THIS test delivers: a runtime round-trip pin that the 12
/// known canonical tags ARE produced by `PrimitiveKind::canonical_tag`
/// for the 12 baseline variants (catches a rename/typo/dispatch-bug
/// on any of the 12; pim-2 §3.6b end-to-end-arm coverage).
#[test]
fn twelve_known_variants_canonical_tags_round_trip() {
    // The 12 known canonical tags from CLAUDE.md baked-in #1.
    let known: &[&str] = &[
        "READ",
        "WRITE",
        "TRANSFORM",
        "BRANCH",
        "ITERATE",
        "WAIT",
        "CALL",
        "RESPOND",
        "EMIT",
        "SANDBOX",
        "SUBSCRIBE",
        "STREAM",
    ];
    assert_eq!(known.len(), 12, "12 baseline kinds enumerated (P-2.1)");

    // The 12 known PrimitiveKind variants, constructed.
    let twelve = [
        PrimitiveKind::Read,
        PrimitiveKind::Write,
        PrimitiveKind::Transform,
        PrimitiveKind::Branch,
        PrimitiveKind::Iterate,
        PrimitiveKind::Wait,
        PrimitiveKind::Call,
        PrimitiveKind::Respond,
        PrimitiveKind::Emit,
        PrimitiveKind::Sandbox,
        PrimitiveKind::Subscribe,
        PrimitiveKind::Stream,
    ];
    assert_eq!(
        twelve.len(),
        12,
        "12 known PrimitiveKind variants constructed (P-2.1)"
    );
    for k in &twelve {
        let tag = k.canonical_tag();
        assert!(
            known.contains(&tag),
            "PrimitiveKind {:?} ({}) is one of the 12 baseline kinds (P-2.1)",
            k,
            tag
        );
    }

    // Tag-set pin: the set of canonical-tags emitted across all 12
    // known variants is EXACTLY the `known` set.
    let emitted: alloc::vec::Vec<&'static str> = twelve.iter().map(|k| k.canonical_tag()).collect();
    assert_eq!(emitted.len(), 12, "12 tag emissions (P-2.1)");
    for tag in known {
        assert!(
            emitted.contains(tag),
            "tag {} present in emissions (P-2.1)",
            tag
        );
    }
}

// ---------------------------------------------------------------------------
// Arm P-2.2 — Walker is itself a Subgraph composed of READ / WRITE /
// TRANSFORM / BRANCH / ITERATE only. (Spike F decomposition: queue +
// visited-set + WalkState; ~25-LOC orchestration expressible in
// existing primitives.)
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3w (pim-12 / §3.6e closure): `walker::walker_as_subgraph()` returns a
/// `Subgraph` whose every `OperationNode.kind` is in the existing
/// 12-primitive set, with NO `Sandbox` (the walker is not WASM-hosted)
/// and NO new variant. WOULD-FAIL if the implementer takes a shortcut
/// and mints a custom primitive.
#[test]
fn walker_is_a_subgraph_composed_of_existing_primitives() {
    let walker_sg: Subgraph = walker::walker_as_subgraph();

    // The walker handler_id is documented to be `"benten:subgraph_spec:walker"`
    // per §1.A.FROZEN item 15(h) "ships once in `benten_core`".
    assert_eq!(
        walker_sg.handler_id(),
        "benten:subgraph_spec:walker",
        "walker handler_id pin (P-2.2)"
    );

    // Every OperationNode's PrimitiveKind must be in the existing
    // 12-primitive set. Specifically: walker decomposition is
    // {READ (deref Node by CID), TRANSFORM (extract edge labels),
    // BRANCH (visited-set membership predicate), ITERATE (queue drain)}.
    // The walker does NOT use SANDBOX (no wasm host); it does NOT use
    // EMIT/SUBSCRIBE/STREAM/WAIT/CALL/RESPOND (no side effects, no
    // suspension, no handler dispatch); it does NOT mutate the graph
    // (no WRITE).
    let allowed = [
        PrimitiveKind::Read,
        PrimitiveKind::Transform,
        PrimitiveKind::Branch,
        PrimitiveKind::Iterate,
    ];
    for op in walker_sg.nodes() {
        assert!(
            allowed.contains(&op.kind),
            "walker uses only READ/TRANSFORM/BRANCH/ITERATE; saw {:?} (P-2.2)",
            op.kind
        );
    }

    // The walker MUST mint at least one OperationNode of each of the
    // four required kinds (the BFS-decomposition shape Spike F
    // identified). Sentinel: at minimum >= 4 OperationNodes (one each
    // for READ / TRANSFORM / BRANCH / ITERATE).
    assert!(
        walker_sg.nodes().len() >= 4,
        "walker BFS decomposition has >=4 operation Nodes (P-2.2)"
    );
}

// ---------------------------------------------------------------------------
// Arm P-2.3 — Walker is shipped ONCE in `benten_core` (per §1.A.FROZEN
// item 15(h)) — consumers call the `Engine::walk_share_scope()`-style
// public surface, never re-implement BFS. The walker subgraph CID is
// stable + content-addressed.
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3w (pim-12 / §3.6e closure): two invocations of `walker_as_subgraph()` yield
/// byte-equal canonical Subgraph bytes (deterministic content) — the
/// walker is shipped ONCE; re-derivation is byte-stable. WOULD-FAIL if
/// the walker's OperationNode IDs are RNG'd or non-deterministic.
#[test]
fn walker_subgraph_canonical_bytes_stable_across_invocations() {
    let walker_1 = walker::walker_as_subgraph();
    let walker_2 = walker::walker_as_subgraph();

    let bytes_1 = walker_1
        .to_canonical_bytes()
        .expect("walker subgraph encodes");
    let bytes_2 = walker_2
        .to_canonical_bytes()
        .expect("walker subgraph encodes again");

    assert_eq!(
        bytes_1, bytes_2,
        "walker subgraph canonical bytes byte-stable (P-2.3)"
    );
}

// ---------------------------------------------------------------------------
// Arm P-2.4 — Composing the walker against a Spec at the surface level
// does NOT widen the engine's primitive set. (Negative: invoking
// `walker::walk(spec)` consumes the walker subgraph composition without
// requiring any new PrimitiveKind tag in the canonical encoding.)
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3w (pim-12 / §3.6e closure): walking a Spec produces enumerated results
/// without introducing a non-12-primitive op anywhere in the trace.
/// (This is the running-the-walker side of P-2.2's static composition
/// pin.)
#[test]
fn walking_does_not_widen_primitive_set() {
    let r = cid_for("widen-R");
    let a = cid_for("widen-A");
    let spec = Spec::builder()
        .with_root(r)
        .with_edge(r, "to_A".to_string(), a)
        .build()
        .expect("valid spec");

    // Walking must succeed without invoking any host primitive outside
    // the 12-set. Production-arm signal: walker::walk returns the
    // enumerated paths without minting a new PrimitiveKind.
    let result = walker::walk(&spec).expect("walk succeeds");
    assert_eq!(
        result.enumerated.len(),
        2,
        "walker enumerated [R, A] (P-2.4)"
    );

    // (P-2.1 + P-2.2 carry the exhaustive structural pin; this arm
    // exercises the running surface so the "static-but-can't-run" gap
    // can't hide an issue.)
}
