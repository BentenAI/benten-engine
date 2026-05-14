//! Phase 4-Foundation Family D canary helper — schema fixtures consumed by
//! G23-A schema_compiler R3 pins + G23-B materializer R3 pins (Family E) +
//! G24-B workflow-editor R3 pins (Family F2) + G24-D plugin-manifest R3 pins
//! that reference schema content (Family F3).
//!
//! **Canary-shape** at R3 RED-PHASE (2026-05-11):
//!
//! - Helper signatures + fixture identities are committed FIRST so downstream
//!   families (E/F1/F2) can dispatch against this branch without waiting for
//!   the full Family D test set to land.
//! - Fixture BODIES intentionally use placeholder/string-only shapes — the
//!   ratified 8-label vocabulary is named by string constant, but the actual
//!   `Schema` / `SchemaSpec` type does not exist yet (lands at R5 G23-A).
//! - Downstream R3 agents import the named functions; their tests fail at
//!   the `use` line (RED-PHASE) per pim-12 §3.6e.
//!
//! ## Fixture inventory
//!
//! - [`canonical_note_type_schema_bytes`] — the canonical "Note" content type
//!   schema (text body + timestamp + optional FieldRef to author plugin-DID).
//!   Round-trip target for `schema_compiler_emits_valid_subgraph_spec_for_canonical_note_type`.
//!
//! - [`minimal_schema_bytes`] — single FieldScalar (text) under SchemaRoot.
//!   Smallest valid schema; round-trip + idempotency target.
//!
//! - [`hostile_schema_unknown_label_bytes`] — schema referencing an
//!   undefined vocabulary label. Negative pin / T1 defense fixture.
//!
//! - [`hostile_schema_with_sandbox_kv_write_bytes`] — schema with a SANDBOX
//!   reference whose manifest requests `kv:write`. Rejected per
//!   sec-3.5-r1-14 + CLAUDE.md baked-in #16.
//!
//! - [`hostile_schema_with_cycle_bytes`] — FieldRef cycle through 3 schemas;
//!   `E_SCHEMA_VOCAB_CYCLE_REJECTED` target.
//!
//! - [`benign_schema_round_trip_bytes`] — T1 regression-guard fixture: a
//!   structurally-valid schema that MUST continue to compile after hostile
//!   schemas land in the rejection set.
//!
//! ## Vocabulary string constants (D-4F-NEW-TYPED-FIELD-NODE-VOCAB)
//!
//! Exposed as `pub const` so RED-PHASE pins can grep-assert vocabulary
//! identity without depending on a runtime symbol that doesn't exist yet.

#![allow(dead_code)] // RED-PHASE: helpers referenced by downstream families'
// R3 pins; some unused in Family D itself.

/// 8-label vocabulary (D-4F-NEW-TYPED-FIELD-NODE-VOCAB).
pub const VOCAB_LABELS: &[&str] = &[
    "SchemaRoot",
    "FieldScalar",
    "FieldObject",
    "FieldList",
    "FieldMap",
    "FieldRef",
    "FieldEnum",
    "FieldUnion",
];

/// 5 labeled edges (parent→child is implicit-via-recursion; no FIELD label).
pub const VOCAB_EDGES: &[&str] = &[
    "ITEM_TYPE",
    "KEY_TYPE",
    "VALUE_TYPE",
    "REF_TARGET",
    "VARIANT",
];

/// 8-scalar vocabulary (matches `benten_core::Value` discriminants).
pub const VOCAB_SCALARS: &[&str] = &[
    "text",
    "int",
    "float",
    "bool",
    "bytes",
    "bytes-cid",
    "timestamp-hlc",
    "null",
];

/// 4 mandatory field properties (`scope` is schema-derived per
/// sec-3.5-r1-4).
pub const VOCAB_FIELD_PROPS: &[&str] = &["name", "required", "default", "scope"];

/// 9 ErrorCode string forms minted at G23-A canary (post-R5 surface).
pub const G23_A_ERROR_CODES: &[&str] = &[
    "E_SCHEMA_VALIDATION_FAILED",
    "E_SCHEMA_EMIT_NEW_PRIMITIVE_REJECTED",
    "E_SCHEMA_SANDBOX_HOST_FN_REJECTED",
    "E_SCHEMA_VOCAB_INVALID_LABEL",
    "E_SCHEMA_VOCAB_EDGE_MISMATCH",
    "E_SCHEMA_VOCAB_SCALAR_UNKNOWN",
    "E_SCHEMA_VOCAB_REF_TARGET_MISSING",
    "E_SCHEMA_VOCAB_CYCLE_REJECTED",
    "E_SCHEMA_VOCAB_REQUIRED_PROPERTY_MISSING",
];

/// Canonical "Note" content type schema source (engineer-facing JSON-Schema
/// dialect). Round-trip target for G23-A primary pin.
///
/// RED-PHASE: bytes-only; the actual parser doesn't exist yet.
pub fn canonical_note_type_schema_bytes() -> &'static [u8] {
    br#"{
  "label": "SchemaRoot",
  "name": "Note",
  "fields": [
    { "label": "FieldScalar", "name": "body", "scalar": "text", "required": true,  "scope": ["read:note", "write:note"] },
    { "label": "FieldScalar", "name": "created_at", "scalar": "timestamp-hlc", "required": true, "scope": ["read:note"] },
    { "label": "FieldRef",    "name": "author", "ref_target_kind": "PluginDid", "required": false, "scope": ["read:note"] }
  ]
}"#
}

/// Minimal valid schema: SchemaRoot with one FieldScalar/text child.
pub fn minimal_schema_bytes() -> &'static [u8] {
    br#"{
  "label": "SchemaRoot",
  "name": "Min",
  "fields": [
    { "label": "FieldScalar", "name": "v", "scalar": "text", "required": true, "scope": ["read:min"] }
  ]
}"#
}

/// Hostile fixture — references a vocabulary label outside the 8-label set.
/// `E_SCHEMA_VOCAB_INVALID_LABEL` target.
pub fn hostile_schema_unknown_label_bytes() -> &'static [u8] {
    br#"{
  "label": "SchemaRoot",
  "name": "Bad",
  "fields": [
    { "label": "FieldQuaternion", "name": "x", "scalar": "text", "required": true, "scope": ["read:bad"] }
  ]
}"#
}

/// Hostile fixture — schema with SANDBOX reference requesting storage-mutating
/// host fn `kv:write`. Rejected per sec-3.5-r1-14 + CLAUDE.md #16.
pub fn hostile_schema_with_sandbox_kv_write_bytes() -> &'static [u8] {
    br#"{
  "label": "SchemaRoot",
  "name": "BadSandbox",
  "fields": [
    { "label": "FieldScalar", "name": "v", "scalar": "text", "required": true, "scope": ["read:bad-sandbox"] }
  ],
  "sandbox_refs": [
    { "module_cid": "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda", "host_fns": ["kv:write"] }
  ]
}"#
}

/// Hostile fixture — FieldRef cycle. `E_SCHEMA_VOCAB_CYCLE_REJECTED` target.
pub fn hostile_schema_with_cycle_bytes() -> &'static [u8] {
    br#"{
  "label": "SchemaRoot",
  "name": "Cyclic",
  "fields": [
    { "label": "FieldRef", "name": "self_ref", "ref_target_kind": "Cyclic", "required": true, "scope": ["read:cyclic"] }
  ]
}"#
}

/// T1 regression-guard fixture — structurally valid schema that must continue
/// to round-trip after hostile schemas land in the rejection set.
pub fn benign_schema_round_trip_bytes() -> &'static [u8] {
    br#"{
  "label": "SchemaRoot",
  "name": "Benign",
  "fields": [
    { "label": "FieldScalar", "name": "title", "scalar": "text", "required": true, "scope": ["read:benign", "write:benign"] },
    { "label": "FieldList",   "name": "tags",  "item_scalar": "text", "required": false, "scope": ["read:benign"] }
  ]
}"#
}
