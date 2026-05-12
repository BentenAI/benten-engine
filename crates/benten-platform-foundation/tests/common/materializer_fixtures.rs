//! Phase 4-Foundation Family E helper — materializer fixtures consumed by
//! G23-B materializer canary R3 pins.
//!
//! **Canary-shape** at R3 RED-PHASE (2026-05-11):
//!
//! - Built on top of Family D's `schema_fixtures.rs` (vocabulary string
//!   constants + canonical Note schema bytes) — `materializer_*.rs` pins
//!   `#[path]` BOTH this module AND `schema_fixtures.rs`.
//! - Fixture BODIES intentionally use placeholder shapes — the actual
//!   `Materializer` / `Renderer` trait + concrete impls do not exist yet
//!   (lands at R5 G23-B wave-5).
//! - RED-PHASE pins describe production shape in comments + `unimplemented!()`
//!   body, mirroring Family D's compile-but-ignore canary pattern.
//!
//! ## Fixture inventory
//!
//! - [`G23_B_ERROR_CODES`] — 3 NEW ErrorCode string-form mints
//!   (`E_MATERIALIZER_CAP_DENIED`, `E_MATERIALIZER_SCHEMA_MISMATCH`,
//!   `E_MATERIALIZER_SUBSCRIBE_SEAM_FAILURE`) added atomically to
//!   `benten-errors` + TS mirror per §3.5g at G23-B canary.
//!
//! - [`note_content_html_expected_skeleton`] — placeholder for the canonical
//!   HTML+JSON output skeleton produced by `HtmlJsonMaterializer` over the
//!   canonical Note schema. RED-PHASE: bytes-only string; G23-B implementer
//!   replaces with deterministic canonical-bytes target.
//!
//! - [`note_content_plaintext_expected_skeleton`] — placeholder for the
//!   `PlaintextMaterializer` 2nd impl's output over the same Note schema.
//!   Pluggability validation per arch-r1-10 (different format from HTML+JSON
//!   verifies the trait is not accidentally HtmlJson-specific).
//!
//! - [`sample_note_content_bytes`] — canonical Note content payload feeding
//!   the materializer walk; pair with `schema_fixtures::canonical_note_type_schema_bytes()`.
//!
//! - [`hostile_subgraph_with_unregistered_sandbox_host_fn_bytes`] — fixture
//!   for the sandbox-host-fn-rejection pin (sec-3.5-r1-14 + CLAUDE.md #16).
//!   Materializer walk MUST reject before any READ fanout.
//!
//! - [`actor_principal_alice_cid`] + [`actor_principal_bob_cid`] +
//!   [`actor_principal_unauthorized_cid`] — fixture actor principal CIDs for
//!   dual-gate composition + cap-denial pins.
//!
//! ## RED-PHASE compile contract
//!
//! Like `schema_fixtures.rs`: helpers compile, are `#[ignore]`d at the test
//! site, and the actual `Materializer` / `Renderer` symbol references live
//! in pin comments + `unimplemented!()` bodies — they do not appear at
//! `use` lines (would block compile of the test crate otherwise).

#![allow(dead_code)] // RED-PHASE: helpers referenced by G23-B R3 pins; some
// unused per pin until G23-B wave-5 fleshes the materializer surface.

use benten_core::{Cid, Node, Value};
use std::collections::BTreeMap;

/// 3 ErrorCode string forms minted at G23-B canary (post-R5 surface) per
/// §3.5g cross-language rule-mirror.
///
/// At HEAD these do NOT exist in `benten-errors`; the `error_catalog_mints_3_g23_b_error_codes`
/// pin proves their post-G23-B presence by round-tripping through `from_str`.
pub const G23_B_ERROR_CODES: &[&str] = &[
    "E_MATERIALIZER_CAP_DENIED",
    "E_MATERIALIZER_SCHEMA_MISMATCH",
    "E_MATERIALIZER_SUBSCRIBE_SEAM_FAILURE",
];

/// Canonical Note content bytes — fed to the materializer alongside the
/// canonical Note schema bytes from `schema_fixtures::canonical_note_type_schema_bytes()`.
///
/// RED-PHASE: opaque bytes; G23-B implementer replaces with the canonical
/// `benten-core::Node` DAG-CBOR-serialized form once the materializer
/// integrates with the engine's WRITE path.
pub fn sample_note_content_bytes() -> &'static [u8] {
    br#"{
  "label": "Note",
  "body": "the quick brown fox",
  "created_at": "2026-05-11T20:00:00Z",
  "author": null
}"#
}

/// Expected HTML+JSON output skeleton from `HtmlJsonMaterializer` over the
/// canonical Note schema + sample content. RED-PHASE: target string; G23-B
/// implementer pins canonical-bytes deterministic output per mat-r1-3.
pub fn note_content_html_expected_skeleton() -> &'static str {
    "<article class=\"benten-note\"><div class=\"benten-field-body\">the quick brown fox</div></article>"
}

/// Expected plaintext output from `PlaintextMaterializer` 2nd impl. Differs
/// structurally from the HTML output to empirically validate output-format
/// pluggability per arch-r1-10.
pub fn note_content_plaintext_expected_skeleton() -> &'static str {
    "body: the quick brown fox\ncreated_at: 2026-05-11T20:00:00Z\n"
}

/// Hostile subgraph fixture — references a SANDBOX module whose host-fn is
/// NOT in the registered manifest. Materializer walk MUST reject before any
/// READ fanout per sec-3.5-r1-14 + CLAUDE.md baked-in #16.
///
/// RED-PHASE: opaque bytes; G23-B implementer pins the SubgraphSpec rejection
/// arm at the materializer walk boundary.
pub fn hostile_subgraph_with_unregistered_sandbox_host_fn_bytes() -> &'static [u8] {
    br#"{
  "label": "SubgraphSpec",
  "primitives": [
    { "kind": "READ", "label": "post" },
    { "kind": "SANDBOX", "module_cid": "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda", "host_fn": "fs:write" }
  ]
}"#
}

/// Mint a stable actor principal CID by labeling a Node with `principal_name`.
///
/// Mirrors the `ivm_read_gate.rs` `principal_for(label)` pattern. The 3
/// named principals below have stable CIDs across runs (BLAKE3 of canonical
/// DAG-CBOR over the labeled+name-property Node).
fn principal_cid_for(name: &str) -> Cid {
    let mut props = BTreeMap::new();
    props.insert(String::from("name"), Value::text(name));
    let node = Node::new(vec!["actor".to_string()], props);
    node.cid().unwrap()
}

/// Fixture actor principal — Alice (authorized to READ canonical Note).
pub fn actor_principal_alice_cid() -> Cid {
    principal_cid_for("alice")
}

/// Fixture actor principal — Bob (authorized to READ public Note only).
pub fn actor_principal_bob_cid() -> Cid {
    principal_cid_for("bob")
}

/// Fixture actor principal — unauthorized (cap-policy denies all).
pub fn actor_principal_unauthorized_cid() -> Cid {
    principal_cid_for("unauthorized")
}

/// Construct a canonical `Note`-labeled content Node from `body`. Used by
/// pins that need to round-trip content through `Engine::put_node` +
/// `Materializer::materialize_with_gate`.
pub fn make_note_node(body: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert(String::from("body"), Value::text(body));
    props.insert(
        String::from("created_at"),
        Value::text("2026-05-11T20:00:00Z"),
    );
    Node::new(vec!["Note".to_string()], props)
}

/// Construct a `post`-labeled row Node for dual-gate per-row pins (mirrors
/// the `ivm_read_gate.rs::cid_for_label` fixture shape).
pub fn make_post_row_node(visibility: &str, idx: u64) -> Node {
    let mut props = BTreeMap::new();
    props.insert(String::from("visibility"), Value::text(visibility));
    props.insert(String::from("seq"), Value::Int(idx as i64));
    Node::new(vec![format!("post:{visibility}")], props)
}

/// Helper: materializer "would-FAIL-if-no-op'd" probe shape — a known
/// 2-row fixture where ONE row is admitted and ONE is denied. Mirrors the
/// `ivm_read_gate.rs::materialize_view_with_gate_filters_rows_per_actor_cap_set_at_engine_entry_point_e2e`
/// shape but for the materializer-side gate.
///
/// Returns `(admitted_node, denied_node)`.
pub fn dual_gate_fixture_pair() -> (Node, Node) {
    let admitted = {
        let mut props = BTreeMap::new();
        props.insert(String::from("kind"), Value::text("admitted"));
        Node::new(vec!["post".to_string()], props)
    };
    let denied = {
        let mut props = BTreeMap::new();
        props.insert(String::from("kind"), Value::text("denied"));
        Node::new(vec!["post".to_string()], props)
    };
    (admitted, denied)
}
