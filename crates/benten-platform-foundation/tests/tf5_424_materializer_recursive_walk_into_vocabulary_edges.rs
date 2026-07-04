//! Phase-4-Meta-Core G-CORE-4 GREEN landing of the §4.24 materializer
//! recursive walk into vocabulary edges (C9 exit obligation).
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
//!   via `VARIANT` edges".
//!
//! ## SHAPE-FLAG closed
//!
//! G-CORE-4 lands the recursive arm of `materialize_format` in
//! `crates/benten-platform-foundation/src/materializer.rs`. The arm
//! threads `RecursiveVocabWalker` (a per-walk view of the engine seam +
//! walk principal + recursion budget) through both `HtmlJsonMaterializer`
//! and `PlaintextMaterializer` render paths. For each emitted READ
//! primitive whose vocabulary label is FieldRef/FieldList/FieldMap/
//! FieldEnum/FieldUnion the walker emits per-edge markers; for FieldRef
//! it ALSO performs a secondary `read_node_as` against the referenced
//! content-CID, embedding the resolved body inline. The 4 markers are
//! re-exported as `RESOLVED_*_MARKER` consts so tests + downstream
//! consumers can grep for the recursive arm's effect.
//!
//! ## §4.42 wasm32 companion (C9 PRE-FLIGHT)
//!
//! R2 §4-C C9 calls out: "§4.42 wasm32 4-site count is a G-CORE-0
//! verify-pass factual check (site-count re-confirm post-COLLAPSE)
//! before TF-5 sweeps." The wasm32 companion arm is staged-pinned below
//! with that sequencing pre-flight (un-ignored at G-CORE-4 only AFTER
//! G-CORE-0 re-confirms the 4-site count — NOT discharged here).
//!
//! ## §3.6g inherited-discipline pre-flight checklist (literal)
//!
//! - [x] §3.5b HARDENED (pim-1): swept in same wave (INTERNALS.md +
//!   SCHEMA-DRIVEN-RENDERING.md + ARCHITECTURE.md vocab notes).
//! - [x] §3.6b + sub-rule-4 (pim-2): production runtime
//!   (`Materializer::materialize_with_gate`) + observable consequence
//!   (resolved body / per-edge marker in output) + would-FAIL
//!   (pre-§4.24 flat walk omits them).
//! - [x] §3.6e (pim-12): RED-PHASE pins un-ignored at G-CORE-4.
//! - [x] §3.6f (pim-18): SHAPE-not-SUBSTANCE — the recursive arm is
//!   EXERCISED end-to-end (not "a recursion function exists").
//! - [x] §3.5g: no cross-language/cross-doc mirror touched.
//! - [x] §3.5i: file-disjoint from G-CORE-7 (G-CORE-7 owns
//!   plugin_lifecycle / manifest_store / plugin_manifest).
//! - [x] §3.6h: no rule codifying an origin here.
//! - [x] §3.6i/§3.6j: N/A (no JSON authored).
//! - [x] §3.13: no per-test static introduced.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

use benten_core::{Node, Value};
use benten_platform_foundation::{
    HtmlJsonMaterializer, InMemoryMaterializerEngine, Materializer, MaterializerCapRecheck,
    MaterializerWalkInputs, PlaintextMaterializer, RESOLVED_LIST_MARKER, RESOLVED_MAP_MARKER,
    RESOLVED_REF_BODY_MARKER, RESOLVED_VARIANT_MARKER, allow_all_cap_recheck, compile_schema,
};
use std::collections::BTreeMap;

/// Nested schema exercising 4 vocabulary edge kinds in one compile:
///   - `FieldRef`  → `REF_TARGET`
///   - `FieldList` → `ITEM_TYPE`
///   - `FieldMap`  → `KEY_TYPE` + `VALUE_TYPE`
///   - `FieldEnum` → `VARIANT`
const NESTED_SCHEMA_BYTES: &[u8] = br#"{
    "label": "SchemaRoot",
    "name": "Article",
    "fields": [
        { "label": "FieldScalar", "name": "title", "scalar": "text",
          "required": true, "default": null },
        { "label": "FieldRef",    "name": "author", "ref_target_kind": "PluginDid",
          "required": false, "default": null },
        { "label": "FieldList",   "name": "tags", "item_scalar": "text",
          "required": false, "default": null },
        { "label": "FieldMap",    "name": "meta", "key_scalar": "text",
          "value_scalar": "text", "required": false, "default": null },
        { "label": "FieldEnum",   "name": "status",
          "variants": ["draft", "published"], "required": false, "default": null }
    ]
}"#;

fn allow_all() -> MaterializerCapRecheck {
    allow_all_cap_recheck()
}

#[test]
fn tf5_424_materializer_recursively_resolves_field_ref_target_via_read_node_as() {
    let spec = compile_schema(NESTED_SCHEMA_BYTES).expect("nested schema compiles");
    let engine = InMemoryMaterializerEngine::new();

    // (1) Persist the referenced content node (the FieldRef target body).
    //     A real plugin-DID descriptor — its fields will appear inline
    //     in the parent's rendered output once the recursive walk
    //     resolves it via a secondary `read_node_as`.
    let mut author_props = BTreeMap::new();
    author_props.insert("did".into(), Value::Text("did:key:author-handle".into()));
    author_props.insert("display_name".into(), Value::Text("Author A".into()));
    let author_cid = engine.put_node(Node::new(vec!["PluginDid".into()], author_props));

    // (2) Persist the parent content node — the author field holds the
    //     referenced content-CID as text (the canonical schema-author
    //     dialect; the production materializer also accepts Bytes).
    let mut article_props = BTreeMap::new();
    article_props.insert("title".into(), Value::Text("Hello".into()));
    article_props.insert("author".into(), Value::Text(author_cid.to_string()));
    article_props.insert("tags".into(), Value::Text("[\"rust\",\"benten\"]".into()));
    article_props.insert("meta".into(), Value::Text("{}".into()));
    article_props.insert("status".into(), Value::Text("draft".into()));
    let article_cid = engine.put_node(Node::new(vec!["Article".into()], article_props));

    let alice = materializer_fixtures::actor_principal_alice_cid();

    let out = HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs::new(
            &engine,
            &spec,
            article_cid,
            alice,
            allow_all(),
            Vec::new(),
        ))
        .expect("materialize_with_gate over nested schema");

    let html = std::str::from_utf8(out.html_bytes()).expect("html is UTF-8");

    // SUBSTANTIVE assertion: the recursive walk followed REF_TARGET +
    // performed a secondary `read_node_as` for the author field — the
    // RESOLVED_REF_BODY_MARKER container is present in the HTML AND
    // carries the resolved body's properties (the inner `<dl>` of
    // `display_name` / `did`). Pre-§4.24 flat walk emits ONLY the CID
    // text; no marker is present.
    assert!(
        html.contains(RESOLVED_REF_BODY_MARKER),
        "§4.24 substantive arm: HTML output MUST contain the \
         RESOLVED_REF_BODY_MARKER (`{RESOLVED_REF_BODY_MARKER}`) — the \
         recursive walk's FieldRef arm follows REF_TARGET via a \
         secondary `read_node_as` and embeds the resolved body. \
         Pre-§4.24 flat walk omits this marker. HTML: {html}"
    );
    // The resolved body itself must appear inline (the parent's HTML
    // contains the child Node's property names).
    assert!(
        html.contains("display_name"),
        "§4.24 substantive arm: the resolved FieldRef target's properties \
         MUST appear inline in the parent's HTML (display_name key from \
         the author body). HTML: {html}"
    );
}

#[test]
fn tf5_424_materializer_recursive_walk_iterates_list_map_and_dispatches_variant() {
    let spec = compile_schema(NESTED_SCHEMA_BYTES).expect("nested schema compiles");
    let engine = InMemoryMaterializerEngine::new();

    let mut article_props = BTreeMap::new();
    article_props.insert("title".into(), Value::Text("Hello".into()));
    article_props.insert("author".into(), Value::Text("did:key:placeholder".into()));
    article_props.insert("tags".into(), Value::Text("[]".into()));
    article_props.insert("meta".into(), Value::Text("{}".into()));
    article_props.insert("status".into(), Value::Text("draft".into()));
    let article_cid = engine.put_node(Node::new(vec!["Article".into()], article_props));

    let alice = materializer_fixtures::actor_principal_alice_cid();

    let out = HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs::new(
            &engine,
            &spec,
            article_cid,
            alice,
            allow_all(),
            Vec::new(),
        ))
        .expect("materialize_with_gate over nested schema");

    let html = std::str::from_utf8(out.html_bytes()).expect("html is UTF-8");

    // SUBSTANTIVE: each of FieldList / FieldMap / FieldEnum descriptors
    // produces its corresponding RESOLVED_* container in the output.
    // The flat walk pre-§4.24 emits none of these — would-FAIL.
    assert!(
        html.contains(RESOLVED_LIST_MARKER),
        "§4.24: FieldList ITEM_TYPE descriptor MUST emit RESOLVED_LIST_MARKER \
         (`{RESOLVED_LIST_MARKER}`). Pre-§4.24 flat walk omits this. HTML: {html}"
    );
    assert!(
        html.contains(RESOLVED_MAP_MARKER),
        "§4.24: FieldMap KEY_TYPE/VALUE_TYPE descriptors MUST emit \
         RESOLVED_MAP_MARKER (`{RESOLVED_MAP_MARKER}`). Pre-§4.24 flat walk \
         omits this. HTML: {html}"
    );
    assert!(
        html.contains(RESOLVED_VARIANT_MARKER),
        "§4.24: FieldEnum/FieldUnion VARIANT descriptor MUST emit \
         RESOLVED_VARIANT_MARKER (`{RESOLVED_VARIANT_MARKER}`). Pre-§4.24 \
         flat walk omits this. HTML: {html}"
    );

    // Cross-check the plaintext path emits the corresponding edge
    // markers. The plaintext recursive walk uses `[edge:<LABEL>:<field>]`
    // markers (distinct from the HTML container shape) — same SUBSTANCE
    // (each vocabulary edge is observably consumed), different syntax.
    let pt = PlaintextMaterializer
        .materialize_with_gate(MaterializerWalkInputs::new(
            &engine,
            &spec,
            article_cid,
            alice,
            allow_all(),
            Vec::new(),
        ))
        .expect("plaintext materialize_with_gate over nested schema");
    let pt_text = std::str::from_utf8(pt.html_bytes()).expect("plaintext is UTF-8");
    assert!(
        pt_text.contains("[edge:ITEM_TYPE"),
        "§4.24 plaintext: FieldList path MUST emit `[edge:ITEM_TYPE...]`. \
         Output: {pt_text}"
    );
    assert!(
        pt_text.contains("[edge:KEY_TYPE") && pt_text.contains("[edge:VALUE_TYPE"),
        "§4.24 plaintext: FieldMap path MUST emit BOTH `[edge:KEY_TYPE...]` \
         and `[edge:VALUE_TYPE...]`. Output: {pt_text}"
    );
    assert!(
        pt_text.contains("[edge:VARIANT"),
        "§4.24 plaintext: FieldEnum/Union path MUST emit `[edge:VARIANT...]`. \
         Output: {pt_text}"
    );
}

#[test]
#[ignore = "DEFERRED: §4.24 + §4.42 wasm32 companion (Phase-4-Meta carry per \
docs/future/phase-4-backlog.md §4.24/§4.42). G-CORE-0 verify-pass already \
re-confirmed the §4.42 4-site count at PR #1310; G-CORE-4 (PR #1311) shipped \
the recursive-walk seam; the wasm32 companion arm lands at Phase-4-Meta \
§4.42 wave per the named destination. (R4b L1-MIN-4 cite-drift fix 2026-05-24: \
the previous 'un-ignore at G-CORE-4' framing was pim-12 §3.6e text-drift — \
G-CORE-4 merged without un-ignore because the wasm32 companion sequencing \
intentionally rides into the Phase-4-Meta §4.42 carry.)"]
fn tf5_424_materializer_recursive_walk_wasm32_companion_after_g_core_0_site_count_verify() {
    // Body intentionally NOT written: the wasm32 companion arm is
    // authored at G-CORE-4 ONLY after G-CORE-0's verify-pass re-confirms
    // the §4.42 4-site count post-COLLAPSE. Writing it now would presume
    // an unverified site count (the exact thing the callout forbids).
    unimplemented!(
        "Blocked on G-CORE-0 verify-pass §4.42 4-site re-confirm \
         (R2 §4-C C9 sequencing pre-flight); wasm32 recursive-walk \
         companion authored at G-CORE-4 after that factual check"
    );
}
