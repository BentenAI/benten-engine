//! Schema → SubgraphSpec emitter.
//!
//! Walks a [`super::parse::ParsedSchema`] and emits a
//! [`benten_core::Subgraph`] composed entirely over the canonical 12
//! primitives (CLAUDE.md baked-in #1). For each top-level field the
//! emitter wires the standard READ → TRANSFORM → RESPOND access path,
//! plus a parallel WRITE path for mutable fields, plus a SUBSCRIBE entry
//! point so the materializer pipeline (G23-B) can register reactive
//! consumers against the same schema-derived primitives.
//!
//! Every emitted primitive Node carries a derived cap-scope annotation
//! in its `properties` bag under
//! [`crate::schema_compiler::CAP_SCOPE_PROPERTY_KEY`]. The scope is
//! synthesized via [`crate::schema_compiler::derive_scope`] from the
//! field path — NOT supplied by the user (sec-3.5-r1-4).

use benten_core::{NodeHandle, OperationNode, PrimitiveKind, Subgraph, SubgraphBuilder, Value};

use super::error::SchemaCompileError;
use super::parse::{ParsedField, ParsedSchema};
use super::vocab::{VocabLabel, canonical_handler_id};
use super::{
    CAP_SCOPE_PROPERTY_KEY, FIELD_PATH_PROPERTY_KEY, VOCAB_LABEL_PROPERTY_KEY, derive_scope,
};

/// Per-primitive descriptor exposed by
/// [`crate::schema_compiler::SchemaSubgraphSpec::primitives`]. Mirrors the
/// emitted Subgraph node order; tests inspect this to verify each
/// primitive carries its derived cap-scope (sec-3.5-r1-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimitiveDescriptor {
    /// Stable per-primitive id within the subgraph (e.g. `"r_body"`).
    pub id: String,
    /// Which of the 12 canonical primitive kinds.
    pub kind: PrimitiveKind,
    /// The derived cap-scope (`Some("read:Note.body")` etc). All
    /// schema-emitted primitives carry one per sec-3.5-r1-4.
    pub cap_scope: Option<String>,
    /// Field path that produced this primitive (e.g. `"Note.body"`).
    pub field_path: Option<String>,
    /// Schema vocab label that produced this primitive.
    pub vocab_label: Option<VocabLabel>,
}

impl PrimitiveDescriptor {
    /// Convenience: returns `Some(&str)` for tests asserting cap-scope
    /// presence.
    #[must_use]
    pub fn cap_scope(&self) -> Option<&str> {
        self.cap_scope.as_deref()
    }

    /// Convenience: returns the canonical primitive kind.
    #[must_use]
    pub fn kind(&self) -> PrimitiveKind {
        self.kind
    }

    /// Derive the per-primitive descriptors from an emitted Subgraph.
    /// Called once by [`crate::schema_compiler::SchemaSubgraphSpec::new`].
    pub(crate) fn derive_for(subgraph: &Subgraph) -> Vec<PrimitiveDescriptor> {
        subgraph
            .nodes()
            .iter()
            .map(|op| PrimitiveDescriptor {
                id: op.id.clone(),
                kind: op.primitive_kind(),
                cap_scope: op.property(CAP_SCOPE_PROPERTY_KEY).and_then(|v| match v {
                    Value::Text(s) => Some(s.clone()),
                    _ => None,
                }),
                field_path: op.property(FIELD_PATH_PROPERTY_KEY).and_then(|v| match v {
                    Value::Text(s) => Some(s.clone()),
                    _ => None,
                }),
                vocab_label: op.property(VOCAB_LABEL_PROPERTY_KEY).and_then(|v| match v {
                    Value::Text(s) => VocabLabel::from_str(s).ok(),
                    _ => None,
                }),
            })
            .collect()
    }
}

/// Emit a `benten_core::Subgraph` from a validated [`ParsedSchema`].
///
/// G23-A canary emit shape per top-level field:
///   - READ  primitive (id `r_<field>`)    — read access path
///   - TRANSFORM primitive (id `t_<field>`) — projection/transform stage
///   - WRITE primitive (id `w_<field>`)    — mutation path (mutable
///     fields only — required + non-default; FieldRef are read-only)
///   - SUBSCRIBE primitive (id `s_<field>`) — reactive entry (materializer)
///   - RESPOND primitive (id `resp_<field>`) — closes the dispatch path
///
/// All emitted primitives carry the derived `cap_scope` property; the
/// edges form a DAG (CLAUDE.md baked-in #4):
///
/// ```text
///   READ → TRANSFORM → WRITE → RESPOND
///          ↘ SUBSCRIBE (parallel branch; reactive arm)
/// ```
///
/// The G23-B materializer walks SUBSCRIBE entry points to wire its
/// reactive arm; the engine evaluator walks the READ → TRANSFORM →
/// WRITE → RESPOND spine. RESPOND is terminal (closes the walk).
pub(crate) fn emit_subgraph(parsed: &ParsedSchema) -> Result<Subgraph, SchemaCompileError> {
    let mut sb = SubgraphBuilder::new(canonical_handler_id(&parsed.name));
    let schema_name = &parsed.name;

    // Track every emitted handle so we can wire SUBSCRIBE arms + RESPOND
    // tail per field.
    for field in &parsed.fields {
        emit_field(&mut sb, schema_name, field, "")?;
    }

    // The builder emits each field's chain independently. If the schema
    // has zero fields, we still need at least one terminal RESPOND so
    // the registered subgraph isn't empty (which the engine's invariant
    // validator would reject at `register_subgraph` time).
    if parsed.fields.is_empty() {
        emit_empty_root_chain(&mut sb, schema_name);
    }

    let sg = sb.build_unvalidated_for_test();
    // Post-build sanity: every node must carry a populated
    // `cap_scope` property (sec-3.5-r1-4). If somewhere a future emitter
    // mutation forgets the stamp, this catches it BEFORE the spec leaves
    // the compiler.
    for node in sg.nodes() {
        if node.property(CAP_SCOPE_PROPERTY_KEY).is_none() {
            return Err(SchemaCompileError::ValidationFailed {
                reason: format!(
                    "emitter bug: primitive `{}` ({}) missing cap_scope annotation; \
                     sec-3.5-r1-4 requires every schema-emitted primitive Node to \
                     carry a derived cap-scope",
                    node.id,
                    node.primitive_kind().canonical_tag()
                ),
                location: None,
            });
        }
    }
    Ok(sg)
}

/// Emit the per-field primitive chain. Wires READ → TRANSFORM → WRITE? →
/// SUBSCRIBE (parallel) → RESPOND.
fn emit_field(
    sb: &mut SubgraphBuilder,
    schema_name: &str,
    field: &ParsedField,
    path_prefix: &str,
) -> Result<(), SchemaCompileError> {
    let field_path = if path_prefix.is_empty() {
        field.name.clone()
    } else {
        format!("{path_prefix}.{}", field.name)
    };

    let read_scope = derive_scope("read", schema_name, &field_path);
    let write_scope = derive_scope("write", schema_name, &field_path);
    let subscribe_scope = derive_scope("subscribe", schema_name, &field_path);
    let transform_scope = derive_scope("transform", schema_name, &field_path);
    let respond_scope = derive_scope("respond", schema_name, &field_path);

    // READ primitive.
    let read = push_with_scope(
        sb,
        format!("r_{}", field.name),
        PrimitiveKind::Read,
        &read_scope,
        &field_path,
        field.label,
    );

    // TRANSFORM primitive.
    let xform = push_with_scope(
        sb,
        format!("t_{}", field.name),
        PrimitiveKind::Transform,
        &transform_scope,
        &field_path,
        field.label,
    );
    sb.add_edge(read, xform);

    // WRITE primitive (mutable fields only; FieldRef + non-required
    // FieldList items are emit-time read-only).
    let mutable = is_field_mutable(field);
    let tail_handle = if mutable {
        let write = push_with_scope(
            sb,
            format!("w_{}", field.name),
            PrimitiveKind::Write,
            &write_scope,
            &field_path,
            field.label,
        );
        sb.add_edge(xform, write);
        write
    } else {
        xform
    };

    // SUBSCRIBE primitive — parallel branch from TRANSFORM (the
    // materializer pipeline reads transformed values reactively).
    let subscribe = push_with_scope(
        sb,
        format!("s_{}", field.name),
        PrimitiveKind::Subscribe,
        &subscribe_scope,
        &field_path,
        field.label,
    );
    sb.add_edge(xform, subscribe);

    // RESPOND primitive — terminal.
    let respond = push_with_scope(
        sb,
        format!("resp_{}", field.name),
        PrimitiveKind::Respond,
        &respond_scope,
        &field_path,
        field.label,
    );
    sb.add_edge(tail_handle, respond);

    // Recurse into FieldObject sub-fields.
    if field.label == VocabLabel::FieldObject {
        for sub in &field.sub_fields {
            emit_field(sb, schema_name, sub, &field_path)?;
        }
    }

    Ok(())
}

/// Emit a degenerate "empty-schema" chain so an empty SchemaRoot still
/// produces a registrable Subgraph. The chain is a single READ → RESPOND
/// pair scoped to the schema name.
fn emit_empty_root_chain(sb: &mut SubgraphBuilder, schema_name: &str) {
    let read = push_with_scope(
        sb,
        "r_root".to_string(),
        PrimitiveKind::Read,
        &derive_scope("read", schema_name, ""),
        "",
        VocabLabel::SchemaRoot,
    );
    let respond = push_with_scope(
        sb,
        "resp_root".to_string(),
        PrimitiveKind::Respond,
        &derive_scope("respond", schema_name, ""),
        "",
        VocabLabel::SchemaRoot,
    );
    sb.add_edge(read, respond);
}

/// Push an `OperationNode` of the given kind with cap-scope +
/// field-path + vocab-label properties bound on its `properties` bag.
///
/// Uses [`SubgraphBuilder::push_primitive`] (the lowest-level
/// constructor for any of the 12 kinds) to mint the node, then layers
/// the 3 schema-compiler properties via
/// [`SubgraphBuilder::set_property_for_test`]. The "_for_test" name on
/// that setter is historical — the method is fully `pub` and is the
/// canonical post-mint property-setter on the builder.
fn push_with_scope(
    sb: &mut SubgraphBuilder,
    id: String,
    kind: PrimitiveKind,
    cap_scope: &str,
    field_path: &str,
    label: VocabLabel,
) -> NodeHandle {
    let h = sb.push_primitive(id, kind);
    sb.set_property_for_test(
        h,
        CAP_SCOPE_PROPERTY_KEY,
        Value::Text(cap_scope.to_string()),
    );
    sb.set_property_for_test(
        h,
        FIELD_PATH_PROPERTY_KEY,
        Value::Text(field_path.to_string()),
    );
    sb.set_property_for_test(
        h,
        VOCAB_LABEL_PROPERTY_KEY,
        Value::Text(label.as_str().to_string()),
    );
    h
}

/// Heuristic: is this field shape mutable? FieldScalar / FieldList /
/// FieldMap / FieldObject are mutable. FieldRef is read-only (the
/// reference content is content-addressed; mutation is via FieldRef
/// re-binding, which is a different graph op). FieldEnum / FieldUnion
/// are mutable.
fn is_field_mutable(field: &ParsedField) -> bool {
    !matches!(field.label, VocabLabel::FieldRef)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema_compiler::compile;

    const NOTE: &[u8] = br#"{
        "label": "SchemaRoot",
        "name": "Note",
        "fields": [
            { "label": "FieldScalar", "name": "body", "scalar": "text", "required": true, "default": null }
        ]
    }"#;

    #[test]
    fn every_primitive_carries_cap_scope() {
        let spec = compile(NOTE).unwrap();
        for p in spec.primitives() {
            assert!(
                p.cap_scope().is_some(),
                "primitive {:?} missing cap_scope; sec-3.5-r1-4 violation",
                p.id
            );
        }
    }

    #[test]
    fn emits_only_canonical_primitive_kinds() {
        let spec = compile(NOTE).unwrap();
        for p in spec.primitives() {
            // PrimitiveKind is #[non_exhaustive] — a future-variant
            // wildcard catches an emitter widening; on the canary the
            // negative arm fails the assertion.
            #[allow(unreachable_patterns)]
            let ok = matches!(
                p.kind(),
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
                    | PrimitiveKind::Stream
            );
            assert!(ok, "non-canonical PrimitiveKind emitted: {:?}", p.kind());
        }
    }

    #[test]
    fn emits_read_transform_subscribe_respond_at_minimum() {
        let spec = compile(NOTE).unwrap();
        let kinds: std::collections::HashSet<_> =
            spec.primitives().iter().map(|p| p.kind).collect();
        assert!(kinds.contains(&PrimitiveKind::Read));
        assert!(kinds.contains(&PrimitiveKind::Transform));
        assert!(kinds.contains(&PrimitiveKind::Subscribe));
        assert!(kinds.contains(&PrimitiveKind::Respond));
    }
}
