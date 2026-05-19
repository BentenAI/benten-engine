//! ADDL R3 (TDD red-phase) — Phase-4-Meta-Core, Wave R3-B, agent R3-B2,
//! family **TF-5**. RED-phase pin for the **§4.24 materializer recursive
//! walk into vocabulary edges** (C9 exit obligation; G-CORE-4 substrate).
//!
//! ## Pin provenance
//!
//! - R2 `.addl/phase-4-meta/r2-test-landscape.md` TF-5 ("materializer
//!   recursive walk into vocabulary edges (§4.24)") + C9 row.
//! - Plan G-CORE-4 group def + §1.A **C9** ("Materializer recursive
//!   walk + IVM byte-equivalence shipped — §4.24 + §4.31 + §4.42 wasm32
//!   companion").
//! - Named destination `docs/future/phase-4-backlog.md §4.24`:
//!   "the materializer's recursive walk that consumes those [5
//!   vocabulary] edges at materialize time — resolving
//!   `FieldRef::REF_TARGET` content via a secondary `read_node_as`
//!   against the referenced content-CID; iterating `FieldList` /
//!   `FieldMap` elements via `ITEM_TYPE` / `VALUE_TYPE` descriptor
//!   lookup; dispatching `FieldEnum` / `FieldUnion` variant selection
//!   via `VARIANT` edges". Integration-pin name specified by §4.24:
//!   `materializer_resolves_field_ref_target_via_engine_read_node_as.rs`
//!   — that name's behaviour is folded into this consolidated TF-5
//!   file (per `feedback_subtrack_sizing_heuristic`; one TF-5 lane
//!   owner) with the §4.24 cite carried.
//!
//! ## §4.42 wasm32 companion — C9 PRE-FLIGHT (R2 §4-C C9 callout)
//!
//! R2 §4-C flags: "§4.42 wasm32 4-site count is a G-CORE-0 verify-pass
//! factual check (site-count re-confirm post-COLLAPSE) before TF-5
//! sweeps — R3-B2's brief carries the 'verify the 4-site claim before
//! writing the wasm32 companion arm' pre-flight (not a test gap, a
//! sequencing pre-flight)." Accordingly: this file does NOT write the
//! wasm32-companion arm. The 4-site claim
//! (`crates/benten-graph/src/backends/blob_backend.rs:63/135/164/248`,
//! per §4.42) is a G-CORE-0 verify-pass deliverable; the wasm32
//! recursive-walk companion arm is staged-pinned below with that
//! sequencing pre-flight recorded in its ignore message (un-ignored at
//! G-CORE-4 only AFTER G-CORE-0 re-confirms the 4-site count).
//!
//! ## §3.6b sub-rule-4 production-arm shape
//!
//! - PRODUCTION RUNTIME ARM: a schema with a `FieldRef`
//!   (`REF_TARGET`-edged) + a `FieldList` (`ITEM_TYPE`-edged) + a
//!   `FieldEnum`/`FieldUnion` (`VARIANT`-edged) + a `FieldMap`
//!   (`KEY_TYPE`/`VALUE_TYPE`-edged) is `compile`d, then materialized
//!   via the production `Materializer::materialize_with_gate` over a
//!   real `MaterializerEngine` (the `EngineMaterializerAdapter` that
//!   routes through `Engine::read_node_as` — Class B β).
//! - OBSERVABLE CONSEQUENCE: the materialized output **recursively
//!   resolves** the referenced content (`FieldRef` target body appears
//!   in the output, not just the bare CID), iterates list/map
//!   descriptors, and dispatches the enum/union variant — i.e. the
//!   nested-form rendering the §4.24 driver requires.
//! - WOULD-FAIL-IF-NO-OP: the HEAD `ed03729a` materializer does an
//!   opcode-list-shaped flat walk (G23-B canary) — it does NOT do the
//!   secondary `read_node_as` against `REF_TARGET`. Pre-G-CORE-4 the
//!   FieldRef target body is ABSENT from the output → the
//!   "referenced-content-resolved" assertion FAILS. Post-§4.24 it is
//!   present → PASS. (Removing the recursion arm regresses to absent.)
//!
//! ## SHAPE-FLAG (not faked)
//!
//! The recursive-walk arm of `materialize_format`
//! (`crates/benten-platform-foundation/src/materializer.rs`) is the
//! G-CORE-4 / §4.24 deliverable (~200-300 LOC per the backlog scope
//! target). It does NOT exist at HEAD. These tests are written against
//! the intended production surface and are `#[ignore]`d (§3.6e staged
//! pin; reviewer verifies landing). The body encodes the substantive
//! "referenced content is recursively resolved into the output"
//! assertion — NOT a sentinel that a recursion function exists.
//!
//! ## §3.6g inherited-discipline pre-flight checklist (literal)
//!
//! - [x] §3.5b HARDENED (pim-1): tests only; G-CORE-4 sweeps docs.
//! - [x] §3.6b + sub-rule-4: recursive-resolution SPECIFIC arm (above).
//! - [x] §3.6e: RED-PHASE staged-pin; ignore msg names §4.24/G-CORE-4.
//! - [x] §3.6f: SHAPE-not-SUBSTANCE — asserts the referenced content is resolved into output, NOT "a recursion fn exists".
//! - [x] §3.5g: no cross-language/cross-doc mirror touched.
//! - [x] §3.5i: benten-platform-foundation/tests MATERIALIZER lane (TF-5). R3-B5 owns benten-engine/benten-caps; R3-B4 owns plugin_lifecycle/manifest_store. The materializer + schema_compiler-vocab surface is disjoint from both — §3.5i mini-review asserts file-disjointness (no shared test file).
//! - [x] §3.6h: no rule-naming-origin codified here.
//! - [x] §3.6i/§3.6j: N/A. §3.13: no shared static introduced.
//! - [x] §3.6g: this checklist IS the literal reproduction.

#![allow(clippy::unwrap_used, clippy::expect_used)]

/// A schema exercising all 5 vocabulary edges in ONE compile so the
/// recursive walk has every edge kind to consume:
/// - `FieldRef` → `REF_TARGET`
/// - `FieldList` → `ITEM_TYPE`
/// - `FieldMap` → `KEY_TYPE` + `VALUE_TYPE`
/// - `FieldEnum` → `VARIANT`
/// (FieldUnion VARIANT is covered by the §4.6 sibling fixture file.)
const NESTED_SCHEMA_BYTES: &[u8] = br#"{
    "label": "SchemaRoot",
    "name": "Article",
    "fields": [
        { "label": "FieldRef",  "name": "author", "ref_target_kind": "PluginDid",
          "required": false, "default": null },
        { "label": "FieldList", "name": "tags", "item_scalar": "text",
          "required": false, "default": null },
        { "label": "FieldMap",  "name": "meta", "key_scalar": "text",
          "value_scalar": "text", "required": false, "default": null },
        { "label": "FieldEnum", "name": "status",
          "variants": ["draft", "published"], "required": false, "default": null }
    ]
}"#;

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-4 — §4.24 materializer recursive walk \
into vocabulary edges. The HEAD materializer does an opcode-list-shaped flat walk \
(G23-B canary); the recursive arm of `materialize_format` (~200-300 LOC, backlog \
§4.24 scope target) is the G-CORE-4 deliverable. C9 exit obligation. Named \
destination: docs/future/phase-4-backlog.md §4.24. Reviewer verifies landing-status \
per §3.6e (not just spec-pin presence)."]
fn tf5_424_materializer_recursively_resolves_field_ref_target_via_read_node_as() {
    // PRODUCTION-ARM (un-ignore + wire at G-CORE-4):
    //
    //   use benten_platform_foundation::schema_compiler::compile;
    //   use benten_platform_foundation::{HtmlJsonMaterializer, Materializer,
    //       MaterializerWalkInputs};
    //   #[path = "common/admin_ui_v0_engine_adapter.rs"]
    //   mod engine_adapter;
    //
    //   // (1) Persist a referenced content node (the FieldRef target).
    //   let target_cid = engine.put_node_as(principal, referenced_body_node);
    //
    //   // (2) Compile the nested schema + emit the Subgraph (carries
    //   //     the 5 vocabulary edges per the G23-B emit side).
    //   let spec = compile(NESTED_SCHEMA_BYTES).unwrap();
    //
    //   // (3) Materialize via the PRODUCTION trait over a real engine
    //   //     adapter (routes reads through Engine::read_node_as).
    //   let adapter = engine_adapter::EngineMaterializerAdapter::new(&engine);
    //   let out = HtmlJsonMaterializer
    //       .materialize_with_gate(MaterializerWalkInputs { /* spec, principal, ... */ })
    //       .unwrap();
    //
    //   // SUBSTANTIVE assertion (NOT a sentinel): the recursive walk
    //   // followed REF_TARGET and resolved the referenced content via
    //   // a secondary read_node_as — the target BODY is in the output,
    //   // not merely the bare CID. Pre-§4.24 the flat walk emits only
    //   // the CID → this FAILS; post-§4.24 the body is present → PASS.
    //   let html = std::str::from_utf8(out.html_bytes()).unwrap();
    //   assert!(html.contains(REFERENCED_BODY_MARKER),
    //       "recursive walk must resolve FieldRef::REF_TARGET content via \
    //        a secondary read_node_as (§4.24); flat walk emits only the \
    //        CID — got: {html}");
    let _ = NESTED_SCHEMA_BYTES;
    unimplemented!(
        "G-CORE-4 / §4.24: wire the recursive arm of `materialize_format` \
         (secondary read_node_as against REF_TARGET) then un-ignore"
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-4 — §4.24 recursive walk iterates \
FieldList ITEM_TYPE + FieldMap KEY_TYPE/VALUE_TYPE descriptors + dispatches \
FieldEnum/FieldUnion VARIANT selection. C9 exit obligation. §4.24 destination. \
Reviewer verifies landing-status per §3.6e."]
fn tf5_424_materializer_recursive_walk_iterates_list_map_and_dispatches_variant() {
    // PRODUCTION-ARM (un-ignore + wire at G-CORE-4): same fixture as
    // above; assert the materialized output reflects:
    //   - FieldList: each element rendered via ITEM_TYPE descriptor;
    //   - FieldMap: entries rendered via KEY_TYPE + VALUE_TYPE;
    //   - FieldEnum/FieldUnion: the selected variant dispatched via
    //     the VARIANT edge (not a flat opcode dump).
    // WOULD-FAIL pre-§4.24: the flat walk does not consume these edges.
    unimplemented!(
        "G-CORE-4 / §4.24: wire ITEM_TYPE/KEY_TYPE/VALUE_TYPE/VARIANT \
         consumption in the recursive materializer walk then un-ignore"
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-4 — §4.24 + §4.42 wasm32 companion. \
SEQUENCING PRE-FLIGHT (R2 §4-C C9 callout): G-CORE-0 verify-pass MUST first \
re-confirm the §4.42 4-site count \
(crates/benten-graph/src/backends/blob_backend.rs:63/135/164/248) post-COLLAPSE \
BEFORE this wasm32 recursive-walk companion arm is written/un-ignored. Not a \
test gap — a sequencing pre-flight. C9. Destination: phase-4-backlog §4.24/§4.42."]
fn tf5_424_materializer_recursive_walk_wasm32_companion_after_g_core_0_site_count_verify() {
    // Body intentionally NOT written at R3 per the R2 §4-C C9
    // sequencing pre-flight: the wasm32 companion arm is authored at
    // G-CORE-4 ONLY after G-CORE-0's verify-pass re-confirms the
    // §4.42 4-site count post-COLLAPSE. Writing it now would presume
    // an unverified site count (the exact thing the callout forbids).
    unimplemented!(
        "Blocked on G-CORE-0 verify-pass §4.42 4-site re-confirm \
         (R2 §4-C C9 sequencing pre-flight); wasm32 recursive-walk \
         companion authored at G-CORE-4 after that factual check"
    );
}
