//! Phase 4-Foundation G23-A schema_compiler — schemas-as-subgraphs-of-
//! primitive-typed-field-Nodes compiler.
//!
//! ## Surface (G23-A canary)
//!
//! The public surface is [`compile`] — `&[u8] -> Result<SchemaSubgraphSpec,
//! SchemaCompileError>`. The input is JSON-Schema-dialect bytes (other ingest
//! dialects land at later G23-A sub-waves via [`ingest_dialect`]). The output
//! [`SchemaSubgraphSpec`] is a content-addressed wrapper around a
//! [`benten_core::Subgraph`] (the canonical-bytes-shaped engine-internal
//! type); it carries enough metadata for round-trip + cap-scope inspection.
//!
//! ## Round-trip + registration path (arch-r1-15)
//!
//! `SchemaSubgraphSpec` exposes [`SchemaSubgraphSpec::as_subgraph`] for
//! canonical-bytes derivation and [`SchemaSubgraphSpec::into_subgraph`] for
//! engine handoff. The `benten-engine::IntoSubgraphSpec for
//! benten_eval::Subgraph` impl (where `benten_eval::Subgraph` is the same
//! type as `benten_core::Subgraph`) means `engine.register_subgraph(spec)`
//! works WITHOUT widening the engine API or introducing a parallel
//! registration surface. Pinned by
//! `tests/schema_compiler_routes_through_existing_register_subgraph_surface_no_new_engine_method.rs`
//! + `tests/schema_compiler_does_not_widen_register_subgraph_signature.rs`.
//!
//! ## Vocabulary (D-4F-NEW-TYPED-FIELD-NODE-VOCAB)
//!
//! - 8 labels: [`VocabLabel::SchemaRoot`] / [`VocabLabel::FieldScalar`] /
//!   [`VocabLabel::FieldObject`] / [`VocabLabel::FieldList`] /
//!   [`VocabLabel::FieldMap`] / [`VocabLabel::FieldRef`] /
//!   [`VocabLabel::FieldEnum`] / [`VocabLabel::FieldUnion`].
//! - 6 edges: [`VocabEdge::Field`] / [`VocabEdge::ItemType`] /
//!   [`VocabEdge::KeyType`] / [`VocabEdge::ValueType`] /
//!   [`VocabEdge::RefTarget`] / [`VocabEdge::Variant`].
//! - 8 scalars: [`Scalar::Text`] / [`Scalar::Int`] / [`Scalar::Float`] /
//!   [`Scalar::Bool`] / [`Scalar::Bytes`] / [`Scalar::BytesCid`] /
//!   [`Scalar::TimestampHlc`] / [`Scalar::Null`] (derived from
//!   `benten_core::Value` per `docs/SCHEMA-DRIVEN-RENDERING.md §2.3`).
//! - 4 mandatory field properties: `name` / `required` / `default` / `scope`
//!   (with `scope` schema-DERIVED per sec-3.5-r1-4 — see [`derive_scope`]).
//!
//! ## Composition over 12 primitives (CLAUDE.md baked-in #1)
//!
//! The emitter wires READ / WRITE / TRANSFORM / RESPOND / SUBSCRIBE
//! primitive Nodes for each field-access path the schema admits. NO new
//! `PrimitiveKind` variants are minted. Pinned by
//! `tests/schema_compiler_emits_subgraph_with_no_new_primitive_kind_variants.rs`
//! + `tests/schema_compiler_typed_field_vocab_composes_over_12_primitives_no_extension.rs`.
//!
//! ## Cap-scope annotations (sec-3.5-r1-4)
//!
//! Every emitted primitive Node carries a derived cap-scope annotation in
//! its `properties` bag under the [`CAP_SCOPE_PROPERTY_KEY`]. The scope is
//! synthesized from the field's path (e.g. `read:Note.body`) — NOT
//! user-supplied. Pinned by
//! `tests/schema_compiler_emits_subgraph_with_cap_scope_annotations_per_primitive_node.rs`
//! + `tests/schema_compiler_emitted_subgraph_walk_fires_cap_policy_at_each_primitive_boundary.rs`.
//!
//! ## SANDBOX storage-mutating host-fn rejection (CLAUDE.md #16)
//!
//! Schemas that embed SANDBOX references requesting `kv:write` / `kv:delete`
//! / `edges:add` / `edges:remove` host-fns are rejected with
//! [`SchemaCompileError::SandboxHostFnRejected`]. Pinned by
//! `tests/schema_compiler_rejects_schema_referencing_sandbox_with_storage_mutating_host_fn_request.rs`.

#![allow(clippy::module_name_repetitions, missing_docs)]

use benten_core::PrimitiveKind;

pub mod emit;
pub mod error;
pub mod ingest_dialect;
pub mod parse;
pub mod spec;
pub mod vocab;

pub use emit::PrimitiveDescriptor;
pub use error::SchemaCompileError;
pub use spec::SchemaSubgraphSpec;
pub use vocab::{
    SCALAR_NAMES, Scalar, VOCAB_EDGE_NAMES, VOCAB_LABEL_NAMES, VOCAB_REQUIRED_FIELD_PROPS,
    VocabEdge, VocabLabel,
};

/// Property-bag key used by emitted primitive Nodes to carry the derived
/// cap-scope. Production cap-policy backends read this property at
/// walk-time and dispatch through `check_capability` per sec-3.5-r1-4.
pub const CAP_SCOPE_PROPERTY_KEY: &str = "cap_scope";

/// Property-bag key carrying the schema field path that produced the
/// primitive Node (e.g. `"Note.body"`). Used for diagnostics + cap-scope
/// derivation symmetry across the 3 emitted primitives per field.
pub const FIELD_PATH_PROPERTY_KEY: &str = "schema_field_path";

/// Property-bag key carrying the schema vocab label that produced the
/// primitive Node (e.g. `"FieldScalar"`). Diagnostic.
pub const VOCAB_LABEL_PROPERTY_KEY: &str = "schema_vocab_label";

/// Top-level entry point — parse + validate + emit. The input bytes are
/// JSON-Schema dialect (the canary G23-A wave-4 ingest dialect). Returns
/// a [`SchemaSubgraphSpec`] whose [`SchemaSubgraphSpec::as_subgraph`]
/// round-trips through `benten_core::canonical_subgraph_bytes`.
///
/// ## Errors
///
/// - [`SchemaCompileError::ValidationFailed`] — malformed JSON / missing
///   required field at SchemaRoot / unconstrained EMIT or RESPOND target.
/// - [`SchemaCompileError::VocabInvalidLabel`] — label outside 8-set.
/// - [`SchemaCompileError::VocabScalarUnknown`] — scalar outside 8-set.
/// - [`SchemaCompileError::VocabRefTargetMissing`] — FieldRef missing
///   `ref_target_kind`.
/// - [`SchemaCompileError::VocabRequiredPropertyMissing`] — field missing
///   one of `name` / `required` / `default` (note: `scope` is
///   schema-derived; user-supplied `scope` is discarded silently).
/// - [`SchemaCompileError::VocabCycleRejected`] — FieldRef cycle.
/// - [`SchemaCompileError::SandboxHostFnRejected`] — SANDBOX reference
///   requesting storage-mutating host-fn.
/// - [`SchemaCompileError::EmitNewPrimitiveRejected`] — defensive
///   regression-guard against 12-primitive commitment drift.
/// - [`SchemaCompileError::VocabEdgeMismatch`] — edge-label pairing
///   outside 6-edge vocabulary.
pub fn compile(bytes: &[u8]) -> Result<SchemaSubgraphSpec, SchemaCompileError> {
    let parsed = parse::parse_schema_json(bytes)?;
    parse::validate_vocab(&parsed)?;
    parse::validate_no_sandbox_storage_mutation(&parsed)?;
    parse::validate_no_unconstrained_emit_respond(&parsed)?;
    parse::detect_field_ref_cycle(&parsed)?;
    let subgraph = emit::emit_subgraph(&parsed)?;
    // Defensive regression-guard: the emitter MUST only produce
    // PrimitiveKind values within the canonical 12. The check is cheap +
    // single-point (here, at the entry) rather than per-emit-site. If any
    // future emitter mutation accidentally widens the variant set, this
    // halts compile before the spec leaves the foundation crate.
    for op in subgraph.nodes() {
        assert_canonical_primitive_kind(op.primitive_kind())?;
    }
    Ok(SchemaSubgraphSpec::new(parsed, subgraph))
}

/// Defensive check — confirms the kind is one of the 12 canonical
/// variants. Future enum-widening would compile but trip this guard,
/// surfacing the 12-primitive-irreducibility violation at G23-A's exit
/// rather than at downstream cap-policy or evaluator-walk time.
fn assert_canonical_primitive_kind(kind: PrimitiveKind) -> Result<(), SchemaCompileError> {
    // `PrimitiveKind` is marked `#[non_exhaustive]` — a wildcard arm is
    // mandatory at the type-system level. The wildcard's existence is
    // exactly why this defensive check is load-bearing: if a future
    // commit lands a 13th variant, this guard surfaces it as a typed
    // `E_SCHEMA_EMIT_NEW_PRIMITIVE_REJECTED` rather than letting the
    // emitter silently emit it.
    match kind {
        PrimitiveKind::Read
        | PrimitiveKind::Write
        | PrimitiveKind::Transform
        | PrimitiveKind::Branch
        | PrimitiveKind::Iterate
        | PrimitiveKind::Wait
        | PrimitiveKind::Call
        | PrimitiveKind::Respond
        | PrimitiveKind::Emit
        | PrimitiveKind::Sandbox
        | PrimitiveKind::Subscribe
        | PrimitiveKind::Stream => Ok(()),
        _ => Err(SchemaCompileError::EmitNewPrimitiveRejected {
            requested_kind: kind.canonical_tag().to_string(),
        }),
    }
}

/// Schema-derived cap-scope per sec-3.5-r1-4. Returns a string like
/// `"read:Note.body"` — the property is bound in the emitted primitive
/// Node's `properties` bag under [`CAP_SCOPE_PROPERTY_KEY`].
///
/// `scope` is NEVER taken from user input — even if the schema JSON
/// supplied a `scope` key, the compiler discards it and synthesizes its
/// own here. This is load-bearing for sec-3.5-r1-4: a user-supplied scope
/// would be an authorization-side-channel (the schema author would
/// effectively decide their own cap-policy scope-key namespace).
#[must_use]
pub fn derive_scope(action: &str, schema_name: &str, field_path: &str) -> String {
    if field_path.is_empty() {
        format!("{action}:{schema_name}")
    } else {
        format!("{action}:{schema_name}.{field_path}")
    }
}

/// Re-export of the property-bag keys to a const slice for grep-able
/// downstream code that wants to enumerate the schema-compiler-stamped
/// keys (e.g. the materializer at G23-B walking emitted SubgraphSpec
/// primitives).
pub const SCHEMA_COMPILER_PROPERTY_KEYS: &[&str] = &[
    CAP_SCOPE_PROPERTY_KEY,
    FIELD_PATH_PROPERTY_KEY,
    VOCAB_LABEL_PROPERTY_KEY,
];

// Re-export for tests that need typed access without going through the
// crate root.
pub(crate) use parse::ParsedSchema;
pub(crate) use vocab::canonical_handler_id;

#[cfg(test)]
mod canary_smoke {
    //! Inline-canary smoke tests proving the module compiles + the public
    //! surface round-trips on the canonical Note fixture. The real test
    //! pins live at `crates/benten-platform-foundation/tests/`.

    use super::*;

    const CANONICAL_NOTE: &[u8] = br#"{
        "label": "SchemaRoot",
        "name": "Note",
        "fields": [
            { "label": "FieldScalar", "name": "body", "scalar": "text", "required": true, "default": null },
            { "label": "FieldScalar", "name": "created_at", "scalar": "timestamp-hlc", "required": true, "default": null },
            { "label": "FieldRef", "name": "author", "ref_target_kind": "PluginDid", "required": false, "default": null }
        ]
    }"#;

    #[test]
    fn compile_canonical_note_succeeds() {
        let spec = compile(CANONICAL_NOTE).expect("canonical Note must compile");
        assert!(
            spec.as_subgraph().primitive_count() > 0,
            "schema-emitted Subgraph must have at least one primitive"
        );
    }

    #[test]
    fn compile_canonical_note_canonical_bytes_round_trip() {
        let s1 = compile(CANONICAL_NOTE).unwrap();
        let s2 = compile(CANONICAL_NOTE).unwrap();
        let b1 = benten_core::canonical_subgraph_bytes(s1.as_subgraph()).unwrap();
        let b2 = benten_core::canonical_subgraph_bytes(s2.as_subgraph()).unwrap();
        assert_eq!(b1, b2, "canonical-bytes must be stable across compiles");
    }
}
