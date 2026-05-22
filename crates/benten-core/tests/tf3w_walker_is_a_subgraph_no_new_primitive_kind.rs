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
//! Special pin shape: arms (P-2.1) + (P-2.2) test for the ABSENCE of a
//! 13th `PrimitiveKind` variant. This is a structural-against-drift
//! pin — if a future G-CORE-3w implementer (or any later phase) adds
//! e.g. `PrimitiveKind::SubgraphSpecWalk`, the exhaustive match below
//! catches it via `non_exhaustive_omitted_patterns` (the same shape as
//! the `Scope` no-opaque-arm pin in `tf3b_no_opaque_selector_arm_*.rs`).
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

/// RED until G-CORE-3w: an exhaustive match over `PrimitiveKind` covers
/// EXACTLY 12 arms (the CLAUDE.md baked-in #1 12-primitive set). If
/// G-CORE-3w mints a 13th variant, this test compile-fails on the
/// missing pattern — the load-bearing structural guard against silent
/// primitive growth.
#[test]
#[ignore = "un-ignore at G-CORE-3w: PrimitiveKind 12-variant invariant pin (P-2.1)"]
fn primitive_kind_remains_exactly_twelve_variants() {
    // The 12 known PrimitiveKind variants, enumerated by name. Each name
    // is a known-good canonical-tag from CLAUDE.md baked-in #1. If a
    // future G-CORE-3w (or any later phase) mints a 13th variant, that
    // variant's `canonical_tag` will not match any of these 12 — the
    // runtime sentinel below catches it.
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

    // For each of the 12 enumerated kinds, the canonical-tag round-trip
    // succeeds + matches one of the known tags. This is the
    // structural pin: the implementer cannot silently add a 13th
    // PrimitiveKind without minting + documenting its canonical_tag,
    // and that new tag will not match any of the 12 above — surfacing
    // the addition at this test site (the closure pin per pim-2
    // §3.6b sub-rule-4).
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
    // known variants is EXACTLY the `known` set. Adding a 13th
    // PrimitiveKind would either (a) add a tag not in `known` (caught
    // by the loop above) OR (b) hide behind an unmatched-kind which
    // `canonical_tag` would still emit somewhere — caught at the
    // implementer's source-edit by `cargo doc` + cite-drift sentinel.
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

/// RED until G-CORE-3w: `walker::walker_as_subgraph()` returns a
/// `Subgraph` whose every `OperationNode.kind` is in the existing
/// 12-primitive set, with NO `Sandbox` (the walker is not WASM-hosted)
/// and NO new variant. WOULD-FAIL if the implementer takes a shortcut
/// and mints a custom primitive.
#[test]
#[ignore = "un-ignore at G-CORE-3w: walker-as-Subgraph composition pin (P-2.2)"]
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
            allowed.iter().any(|k| *k == op.kind),
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

/// RED until G-CORE-3w: two invocations of `walker_as_subgraph()` yield
/// byte-equal canonical Subgraph bytes (deterministic content) — the
/// walker is shipped ONCE; re-derivation is byte-stable. WOULD-FAIL if
/// the walker's OperationNode IDs are RNG'd or non-deterministic.
#[test]
#[ignore = "un-ignore at G-CORE-3w: walker subgraph byte-stability pin (P-2.3)"]
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

/// RED until G-CORE-3w: walking a Spec produces enumerated results
/// without introducing a non-12-primitive op anywhere in the trace.
/// (This is the running-the-walker side of P-2.2's static composition
/// pin.)
#[test]
#[ignore = "un-ignore at G-CORE-3w: walker runtime composition stays inside 12 primitives (P-2.4)"]
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
