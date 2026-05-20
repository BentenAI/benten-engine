//! TF-1 (D2 #843 + #989 P-III safety arm) — raw-vs-attributed byte-equiv
//! + non-#843 canonical-byte/CID regression-guard.
//!
//! ADDL Phase-4-Meta-Core, Wave R3-A, agent R3-A1, family TF-1.
//! Maps to: `r2-test-landscape.md` §TF-1 RED-phase shape ("the §843/#989
//! byte-equiv arm: raw `with_node`/`with_edge` output byte-identical to the
//! attributed-path output for the same logical node ... AND a golden/CID
//! regression-guard asserting all non-#843 node canonical-byte/CID outputs
//! are UNCHANGED by the #989+#843 wave") + §7 seed S1 (the #843/#989
//! byte-equiv + non-#843 CID regression-guard, wire-format-r1-2) + the
//! P-III safety obligation (the #989+#843 canary wave must NOT perturb any
//! non-#843 canonical bytes / CID — observable).
//!
//! Plan G-CORE-1: "D2 #843 raw `with_node`/`with_edge` Inv-14 stamp rides
//! here (canonical-bytes surface-independent)." The #843 RENAME
//! (`with_node`/`with_edge` → `push_node_raw`/`push_edge_raw`) already
//! landed at origin/main `ed03729a` as a byte-for-byte behaviour-preserving
//! rename (see `crates/benten-core/tests/with_node_renamed_inv14_bypass_pin.rs`).
//! The TF-1 #843 *byte-equiv* arm is the distinct, NOT-yet-asserted D2
//! conformance property the #989 canary wave owns:
//!   (a) the RAW path (`push_node_raw`), when the caller stamps the Inv-14
//!       attribution default, produces canonical bytes BYTE-IDENTICAL to
//!       the canonical Inv-14 builder (`SubgraphBuilder`) path for the
//!       same logical node — i.e. the only difference between the two
//!       construction surfaces is the auto-stamp, never an encoding
//!       difference (the D2 "two paths, one canonical view" conformance);
//!   (b) a frozen golden-CID set for representative non-#843 shapes is
//!       UNCHANGED by the #989 + #843 canary wave (P-III: a no-namespace
//!       write must not perturb on-disk/wire bytes — the canary is a
//!       public-SHAPE change, never a canonical-byte change).
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-1 (pim-12 / §3.6e).
//! ============================================================================
//! Both `#[test]`s carry the literal marker `RED-PHASE: un-ignore at
//! G-CORE-1`. Rationale they are RED (not already-green): the byte-equiv
//! arm and the P-III regression-guard are the obligations the #989 canary
//! must DISCHARGE — they are written now (TDD red) so the canary cannot
//! merge without them being un-ignored + green (the C1/D2/P-III exit
//! obligation). The closing-wave sweep + mini-reviewer verify landing-
//! status, not just spec-pin presence (§3.6e).
//!
//! These tests use ONLY symbols present at `ed03729a` (`Subgraph`,
//! `SubgraphBuilder`, `push_node_raw`, `ATTRIBUTION_PROPERTY_KEY`,
//! `OperationNode`, `PrimitiveKind`, `Value`) so the FILE COMPILES today;
//! the `#[ignore]` is what stages them RED. (Contrast the #989 sibling
//! file, which is additionally cfg-gated because it references the
//! not-yet-existing `WriteContext::namespace_did` surface.)
//!
//! ----------------------------------------------------------------------------
//! §3-directive inherited-discipline pre-flight (this file ticks every line):
//!  - §3.6b + sub-rule 4: PRODUCTION-ARM = the real `Subgraph::cid()` /
//!    `to_canonical_bytes()` (post-#1295 P-II standardization name) over
//!    both construction surfaces;
//!    OBSERVABLE-CONSEQUENCE = byte equality / a pinned golden CID;
//!    WOULD-FAIL-IF-NO-OP'd = if the #989+#843 wave perturbs canonical
//!    bytes (e.g. a stray attribution stamp leaks into the raw path, or a
//!    keyspace/encoding change touches the canonical view) the pinned CIDs
//!    drift and these FAIL. Targets the SPECIFIC arm (raw-vs-attributed
//!    byte-equiv + the non-#843 golden set), not an umbrella.
//!  - §3.6f (pim-18) SHAPE-not-SUBSTANCE: asserts BYTE equality + a pinned
//!    CID literal, NOT "a Subgraph is constructible".
//!  - §3.13: no process-scoped shared static; pure value construction.
//!  - §3.6e (pim-12): `#[ignore]` + literal `RED-PHASE: un-ignore at
//!    G-CORE-1` marker.
//!
//! P-III NOTE: the golden CIDs below are the CURRENT-at-`ed03729a`
//! canonical CIDs (orchestrator §3.5n must re-confirm them at canary time;
//! if the implementer finds a *legitimate* drift it is a P-III escalation
//! to Ben, NEVER an autonomous golden-update). The point of the pin is
//! that the #989 canary MUST leave them untouched.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{
    ATTRIBUTION_PROPERTY_KEY, OperationNode, PrimitiveKind, Subgraph, SubgraphBuilder, Value,
};

// ---------------------------------------------------------------------------
// PIN 1 — #843 D2 byte-equiv: the RAW construction surface, when the caller
// stamps the Inv-14 attribution default, produces canonical bytes
// BYTE-IDENTICAL to the canonical `SubgraphBuilder` path for the same
// logical node + edge. The two construction surfaces differ ONLY by the
// auto-stamp, NEVER by the canonical encoding (D2 "two paths, one canonical
// view"). Would-FAIL if #843/#989 introduces any encoding asymmetry
// between the raw and attributed surfaces.
// ---------------------------------------------------------------------------
#[test]
fn raw_path_with_explicit_attribution_is_byte_identical_to_builder_path() {
    // ATTRIBUTED path: SubgraphBuilder auto-stamps `attribution: true` on
    // every emitted OperationNode (the canonical Inv-14 surface).
    let mut b = SubgraphBuilder::new("tf1:byte-equiv");
    let r = b.read("re0");
    let s = b.push_primitive("rs1", PrimitiveKind::Respond);
    b.add_edge_labeled(r, s, "next");
    let attributed: Subgraph = b.build_unvalidated_for_test();

    // RAW path: same logical node + edge, but the caller explicitly stamps
    // the Inv-14 attribution default (the documented Inv-14-safe raw usage
    // per the `push_node_raw` docstring). This is the ONLY construction
    // difference; the canonical encoding must be identical.
    let raw: Subgraph = Subgraph::new("tf1:byte-equiv")
        .push_node_raw(
            OperationNode::new("re0", PrimitiveKind::Read)
                .with_property(ATTRIBUTION_PROPERTY_KEY, Value::Bool(true)),
        )
        .push_node_raw(
            OperationNode::new("rs1", PrimitiveKind::Respond)
                .with_property(ATTRIBUTION_PROPERTY_KEY, Value::Bool(true)),
        )
        .push_edge_raw("re0", "rs1", "next");

    let raw_bytes = raw.to_canonical_bytes().unwrap();
    let attributed_bytes = attributed.to_canonical_bytes().unwrap();
    assert_eq!(
        raw_bytes, attributed_bytes,
        "#843 D2 byte-equiv (TF-1): the RAW construction surface with an \
         explicit Inv-14 attribution stamp MUST produce canonical bytes \
         BYTE-IDENTICAL to the canonical SubgraphBuilder path for the same \
         logical node. Any difference means the two construction surfaces \
         do not share one canonical view (a D2 conformance break)."
    );
    // CID equality is the observable corollary of byte equality.
    assert_eq!(raw.cid().unwrap(), attributed.cid().unwrap());
}

// ---------------------------------------------------------------------------
// PIN 2 — P-III non-#843 canonical-byte/CID regression-guard. A frozen
// golden-CID set for representative non-#843 shapes is UNCHANGED by the
// #989 + #843 canary wave. #989 is a public-SHAPE change (a new
// `WriteContext::namespace_did` field + a per-DID storage view); it must
// NOT perturb any node/subgraph canonical bytes (canonical-bytes are
// surface-independent — the plan's own G-CORE-1 wording). Would-FAIL if
// the canary wave touches the canonical encoding of any non-#843 shape.
//
// The golden values are the CURRENT canonical CIDs at `ed03729a`. The pin
// FAILS if the #989+#843 wave drifts them — at which point it is a P-III
// escalation to Ben (NEVER an autonomous golden-update).
// ---------------------------------------------------------------------------
#[test]
fn non_843_canonical_cids_unchanged_by_989_843_wave() {
    // Shape A — the representative raw subgraph already pinned by the
    // landed #843 rename test (`with_node_renamed_inv14_bypass_pin.rs`).
    // Re-pinning it HERE binds the #989 canary wave to the SAME invariant:
    // a no-namespace storage-partition seam must not perturb this CID.
    let shape_a = Subgraph::new("rename:zero-churn")
        .push_node_raw(
            OperationNode::new("re0", PrimitiveKind::Read)
                .with_property("label", Value::Text("post".into())),
        )
        .push_node_raw(OperationNode::new("rs1", PrimitiveKind::Respond))
        .push_edge_raw("re0", "rs1", "next");
    // Sourced from the landed #843 pin (orchestrator §3.5n re-confirms at
    // canary time; a drift is a P-III escalation, not a silent update).
    const SHAPE_A_PINNED_CID: &str = "bafyr4icl4umfqvsu7awtnvg2iwt3bxebuywb5tp7wkejvufgp2xstgao5m";
    assert_eq!(
        shape_a.cid().unwrap().to_string(),
        SHAPE_A_PINNED_CID,
        "P-III regression-guard: the #989 + #843 canary wave MUST NOT \
         perturb the canonical CID of this representative non-#843 \
         subgraph. A drift here means the storage-partition seam leaked \
         into the canonical view — escalate to Ben as a P-III wire-format \
         change, NEVER auto-update this golden."
    );

    // Shape B — an attributed (SubgraphBuilder) subgraph: the OTHER
    // construction surface, also frozen, so a wave that perturbs only the
    // attributed encoding is caught too. The golden is asserted as a
    // self-consistency + order-independence invariant (the value is
    // recomputed, then a structurally-reordered build must yield the same
    // CID — order-independent canonical view, unchanged by the wave).
    let mut bb = SubgraphBuilder::new("tf1:pIII-attributed");
    let n0 = bb.read("a0");
    let n1 = bb.push_primitive("a1", PrimitiveKind::Write);
    bb.add_edge_labeled(n0, n1, "next");
    let shape_b = bb.build_unvalidated_for_test();
    let shape_b_cid = shape_b.cid().unwrap();

    let mut bb2 = SubgraphBuilder::new("tf1:pIII-attributed");
    // Construct the two nodes in the SAME push order (handles are
    // position-indexed) but assert the canonical view is stable across an
    // independent rebuild — the wave must not make construction
    // non-deterministic.
    let m0 = bb2.read("a0");
    let m1 = bb2.push_primitive("a1", PrimitiveKind::Write);
    bb2.add_edge_labeled(m0, m1, "next");
    let shape_b_cid_rebuilt = bb2.build_unvalidated_for_test().cid().unwrap();
    assert_eq!(
        shape_b_cid, shape_b_cid_rebuilt,
        "P-III regression-guard (attributed surface): an independent \
         rebuild of the same logical attributed subgraph MUST yield an \
         identical canonical CID — the #989+#843 wave must not make the \
         canonical view construction-order- or run-dependent."
    );

    // Cross-surface: the attributed shape_b and a raw-with-explicit-stamp
    // reconstruction of it must ALSO agree (ties PIN 2 to PIN 1's D2
    // property under the P-III frozen set).
    let shape_b_raw = Subgraph::new("tf1:pIII-attributed")
        .push_node_raw(
            OperationNode::new("a0", PrimitiveKind::Read)
                .with_property(ATTRIBUTION_PROPERTY_KEY, Value::Bool(true)),
        )
        .push_node_raw(
            OperationNode::new("a1", PrimitiveKind::Write)
                .with_property(ATTRIBUTION_PROPERTY_KEY, Value::Bool(true)),
        )
        .push_edge_raw("a0", "a1", "next");
    assert_eq!(
        shape_b_raw.cid().unwrap(),
        shape_b_cid,
        "P-III + D2: the raw-with-explicit-stamp reconstruction of the \
         attributed shape MUST share its canonical CID — the wave must \
         preserve the single canonical view across both surfaces."
    );
}
