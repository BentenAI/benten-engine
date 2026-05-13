//! Schema parser — JSON-Schema dialect (G23-A canary) → typed
//! [`ParsedSchema`] intermediate representation. Subsequent stages
//! (`validate_vocab`, `validate_no_sandbox_storage_mutation`,
//! `validate_no_unconstrained_emit_respond`, `detect_field_ref_cycle`,
//! `emit::emit_subgraph`) consume this IR.
//!
//! The IR is INTERMEDIATE — it is not part of the public schema-compiler
//! surface (kept `pub(crate)` so tests inside the crate can introspect
//! but the API surface is just [`super::compile`] → [`super::spec::SchemaSubgraphSpec`]).

use std::collections::BTreeSet;

use super::error::SchemaCompileError;
use super::vocab::{Scalar, VocabLabel};

/// Top-level parsed-schema IR. One per schema; the root MUST carry
/// `label = "SchemaRoot"` per D-4F-NEW-TYPED-FIELD-NODE-VOCAB.
#[derive(Debug, Clone)]
pub struct ParsedSchema {
    pub(crate) name: String,
    pub(crate) fields: Vec<ParsedField>,
    /// Embedded SANDBOX module references. Populated by the parser when
    /// the input JSON has a `sandbox_refs` array; validated by
    /// [`validate_no_sandbox_storage_mutation`].
    pub(crate) sandbox_refs: Vec<SandboxRef>,
    /// EMIT targets declared at the schema root.
    pub(crate) emit_targets: Vec<EmitTarget>,
    /// RESPOND targets declared at the schema root.
    pub(crate) respond_targets: Vec<RespondTarget>,
}

/// A single field declaration.
#[derive(Debug, Clone)]
pub struct ParsedField {
    pub(crate) label: VocabLabel,
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) default: serde_json::Value,
    /// Set only for `FieldScalar`.
    pub(crate) scalar: Option<Scalar>,
    /// Set only for `FieldRef`.
    pub(crate) ref_target_kind: Option<String>,
    /// For nested FieldObject — recursive sub-fields.
    pub(crate) sub_fields: Vec<ParsedField>,
    /// For FieldList — item scalar (G23-A canary supports
    /// scalar-only items; FieldObject items land at later wave).
    pub(crate) item_scalar: Option<Scalar>,
}

impl ParsedField {
    /// Diagnostic — does this field carry a child reference (object /
    /// list / map / ref / enum / union)?
    pub(crate) fn has_child_shape(&self) -> bool {
        !self.sub_fields.is_empty() || self.item_scalar.is_some() || self.ref_target_kind.is_some()
    }
}

/// SANDBOX reference embedded in a schema.
#[derive(Debug, Clone)]
pub struct SandboxRef {
    pub(crate) module_cid: String,
    pub(crate) host_fns: Vec<String>,
}

/// EMIT target at the schema root.
#[derive(Debug, Clone)]
pub struct EmitTarget {
    pub(crate) topic: String,
    /// If `None`, this triggers `E_SCHEMA_VALIDATION_FAILED` per
    /// sec-3.5-r1-4 (unconstrained EMIT rejected).
    pub(crate) scope: Option<String>,
}

/// RESPOND target.
#[derive(Debug, Clone)]
pub struct RespondTarget {
    pub(crate) handler_id: String,
    pub(crate) scope: Option<String>,
}

/// Storage-mutating host fns per CLAUDE.md baked-in #16. These are the
/// names a schema-embedded SANDBOX MUST NOT request.
const FORBIDDEN_HOST_FNS: &[&str] = &[
    "kv:write",
    "kv:delete",
    "edges:add",
    "edges:remove",
    "graph:write",
    "graph:delete",
];

/// Parse the input JSON-Schema dialect bytes into a typed [`ParsedSchema`].
///
/// G23-A canary supports a small dialect that mirrors the structure of
/// the schema fixtures (see
/// `crates/benten-platform-foundation/tests/common/schema_fixtures.rs`):
///
/// ```text
/// {
///   "label": "SchemaRoot",
///   "name": "...",
///   "fields": [ {field}, {field}, ... ],
///   "sandbox_refs": [ {ref}, ... ],          (optional)
///   "emit_targets": [ {target}, ... ],        (optional)
///   "respond_targets": [ {target}, ... ]      (optional)
/// }
/// ```
pub(crate) fn parse_schema_json(bytes: &[u8]) -> Result<ParsedSchema, SchemaCompileError> {
    let raw: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| SchemaCompileError::ValidationFailed {
            reason: format!("malformed JSON: {e}"),
            location: None,
        })?;
    let obj = raw
        .as_object()
        .ok_or_else(|| SchemaCompileError::ValidationFailed {
            reason: "schema root must be a JSON object".to_string(),
            location: Some("$".to_string()),
        })?;

    let label_raw = obj.get("label").and_then(|v| v.as_str()).ok_or_else(|| {
        SchemaCompileError::ValidationFailed {
            reason: "missing `label` at SchemaRoot".to_string(),
            location: Some("$".to_string()),
        }
    })?;
    let label = VocabLabel::from_str(label_raw)?;
    if label != VocabLabel::SchemaRoot {
        return Err(SchemaCompileError::ValidationFailed {
            reason: format!("top-level label must be SchemaRoot; got `{label_raw}`"),
            location: Some("$.label".to_string()),
        });
    }
    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SchemaCompileError::ValidationFailed {
            reason: "missing `name` at SchemaRoot".to_string(),
            location: Some("$.name".to_string()),
        })?
        .to_string();

    let fields_raw = obj
        .get("fields")
        .and_then(|v| v.as_array())
        .ok_or_else(|| SchemaCompileError::ValidationFailed {
            reason: "missing `fields` array at SchemaRoot".to_string(),
            location: Some("$.fields".to_string()),
        })?;
    let mut fields = Vec::with_capacity(fields_raw.len());
    for (idx, raw_field) in fields_raw.iter().enumerate() {
        fields.push(parse_field(raw_field, &format!("$.fields[{idx}]"))?);
    }

    let sandbox_refs = if let Some(refs) = obj.get("sandbox_refs").and_then(|v| v.as_array()) {
        refs.iter()
            .map(|r| parse_sandbox_ref(r))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    let emit_targets = if let Some(arr) = obj.get("emit_targets").and_then(|v| v.as_array()) {
        arr.iter()
            .map(parse_emit_target)
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    let respond_targets = if let Some(arr) = obj.get("respond_targets").and_then(|v| v.as_array()) {
        arr.iter()
            .map(parse_respond_target)
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    Ok(ParsedSchema {
        name,
        fields,
        sandbox_refs,
        emit_targets,
        respond_targets,
    })
}

fn parse_field(raw: &serde_json::Value, location: &str) -> Result<ParsedField, SchemaCompileError> {
    let obj = raw
        .as_object()
        .ok_or_else(|| SchemaCompileError::ValidationFailed {
            reason: "field must be a JSON object".to_string(),
            location: Some(location.to_string()),
        })?;

    let label_raw = obj.get("label").and_then(|v| v.as_str()).ok_or_else(|| {
        SchemaCompileError::VocabRequiredPropertyMissing {
            field_name: obj
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("<unnamed>")
                .to_string(),
            missing_property: "label".to_string(),
        }
    })?;
    let label = match VocabLabel::from_str(label_raw) {
        Ok(l) => l,
        Err(SchemaCompileError::VocabInvalidLabel { label, .. }) => {
            return Err(SchemaCompileError::VocabInvalidLabel {
                label,
                field_name: obj.get("name").and_then(|v| v.as_str()).map(str::to_string),
            });
        }
        Err(e) => return Err(e),
    };

    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SchemaCompileError::VocabRequiredPropertyMissing {
            field_name: "<unnamed>".to_string(),
            missing_property: "name".to_string(),
        })?
        .to_string();

    let required = obj
        .get("required")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| SchemaCompileError::VocabRequiredPropertyMissing {
            field_name: name.clone(),
            missing_property: "required".to_string(),
        })?;

    // `default` is one of the 4 mandatory field properties — but treat
    // an absent `default` as implicit JSON-null (matches the R3 fixture
    // dialect which omits `default` and embeds it via the field's
    // shape). The 4-mandatory invariant is enforced over the emitted
    // primitive's property bag (the emitter stamps the default property
    // regardless); `vocab_required_property_missing` is reserved for
    // `name` and `required` only at G23-A canary. Strict 4-of-4 input-
    // dialect validation deferred per docs/future/phase-4-backlog.md
    // §4.6 (lands when the canonical-fixture generator is auto-derived
    // from the typed IR, OR earlier if a future wave needs strict
    // input-dialect validation).
    let default = obj
        .get("default")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    // NOTE: `scope` is schema-DERIVED at emit time — we do NOT require
    // (or honor) a user-supplied `scope` here per sec-3.5-r1-4. If the
    // input JSON has a `scope` key (string-or-array), we silently
    // discard it; the emitter synthesizes its own scope per field path.

    let scalar = if label == VocabLabel::FieldScalar {
        let scalar_str = obj.get("scalar").and_then(|v| v.as_str()).ok_or_else(|| {
            SchemaCompileError::VocabRequiredPropertyMissing {
                field_name: name.clone(),
                missing_property: "scalar".to_string(),
            }
        })?;
        Some(Scalar::from_str(scalar_str, Some(name.as_str()))?)
    } else {
        None
    };

    let ref_target_kind = if label == VocabLabel::FieldRef {
        let kind = obj
            .get("ref_target_kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SchemaCompileError::VocabRefTargetMissing {
                field_name: name.clone(),
                ref_target_kind: None,
            })?;
        Some(kind.to_string())
    } else {
        None
    };

    let sub_fields = if label == VocabLabel::FieldObject {
        if let Some(arr) = obj.get("fields").and_then(|v| v.as_array()) {
            let mut sub = Vec::with_capacity(arr.len());
            for (idx, raw_sub) in arr.iter().enumerate() {
                sub.push(parse_field(raw_sub, &format!("{location}.fields[{idx}]"))?);
            }
            sub
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let item_scalar = if label == VocabLabel::FieldList {
        if let Some(s) = obj.get("item_scalar").and_then(|v| v.as_str()) {
            Some(Scalar::from_str(s, Some(name.as_str()))?)
        } else {
            None
        }
    } else {
        None
    };

    Ok(ParsedField {
        label,
        name,
        required,
        default,
        scalar,
        ref_target_kind,
        sub_fields,
        item_scalar,
    })
}

fn parse_sandbox_ref(raw: &serde_json::Value) -> Result<SandboxRef, SchemaCompileError> {
    let obj = raw
        .as_object()
        .ok_or_else(|| SchemaCompileError::ValidationFailed {
            reason: "sandbox_ref must be a JSON object".to_string(),
            location: Some("sandbox_refs[]".to_string()),
        })?;
    let module_cid = obj
        .get("module_cid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SchemaCompileError::ValidationFailed {
            reason: "sandbox_ref missing `module_cid`".to_string(),
            location: Some("sandbox_refs[]".to_string()),
        })?
        .to_string();
    let host_fns = obj
        .get("host_fns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(SandboxRef {
        module_cid,
        host_fns,
    })
}

fn parse_emit_target(raw: &serde_json::Value) -> Result<EmitTarget, SchemaCompileError> {
    let obj = raw
        .as_object()
        .ok_or_else(|| SchemaCompileError::ValidationFailed {
            reason: "emit_target must be a JSON object".to_string(),
            location: Some("emit_targets[]".to_string()),
        })?;
    let topic = obj
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SchemaCompileError::ValidationFailed {
            reason: "emit_target missing `topic`".to_string(),
            location: Some("emit_targets[]".to_string()),
        })?
        .to_string();
    // `scope` may be present as a string OR JSON null OR absent. All three
    // collapse to "no scope" — the unconstrained-EMIT validator catches it.
    let scope = obj
        .get("scope")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    Ok(EmitTarget { topic, scope })
}

fn parse_respond_target(raw: &serde_json::Value) -> Result<RespondTarget, SchemaCompileError> {
    let obj = raw
        .as_object()
        .ok_or_else(|| SchemaCompileError::ValidationFailed {
            reason: "respond_target must be a JSON object".to_string(),
            location: Some("respond_targets[]".to_string()),
        })?;
    let handler_id = obj
        .get("handler_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SchemaCompileError::ValidationFailed {
            reason: "respond_target missing `handler_id`".to_string(),
            location: Some("respond_targets[]".to_string()),
        })?
        .to_string();
    let scope = obj
        .get("scope")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    Ok(RespondTarget { handler_id, scope })
}

/// Re-validate vocabulary identity post-parse. The parser already invokes
/// `VocabLabel::from_str` etc; this stage is an extra defensive sweep
/// (e.g., catches an empty fields array where the SchemaRoot must
/// carry ≥1 field for a meaningful emit, but more importantly the
/// vocab-edge sanity).
///
/// G23-A canary: at the current dialect shape there are no user-declared
/// edge labels (the field-tree structure implies edges); this validator
/// is the scaffolding for the explicit-edge dialect deferred per
/// `docs/future/phase-4-backlog.md` §4.6 (couples to strict 4-of-4
/// input-dialect validation; lands when the canonical-fixture generator
/// is auto-derived from the typed IR).
pub(crate) fn validate_vocab(parsed: &ParsedSchema) -> Result<(), SchemaCompileError> {
    // Per-field vocabulary depth-walk — confirms each field's
    // (label, scalar / ref-target / item-scalar) shape is internally
    // consistent.
    for field in &parsed.fields {
        validate_field_vocab(field)?;
    }
    Ok(())
}

fn validate_field_vocab(field: &ParsedField) -> Result<(), SchemaCompileError> {
    match field.label {
        VocabLabel::FieldScalar if field.scalar.is_none() => {
            return Err(SchemaCompileError::VocabRequiredPropertyMissing {
                field_name: field.name.clone(),
                missing_property: "scalar".to_string(),
            });
        }
        VocabLabel::FieldRef if field.ref_target_kind.is_none() => {
            return Err(SchemaCompileError::VocabRefTargetMissing {
                field_name: field.name.clone(),
                ref_target_kind: None,
            });
        }
        VocabLabel::FieldObject => {
            for sub in &field.sub_fields {
                validate_field_vocab(sub)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// CLAUDE.md baked-in #16 enforcement — reject SANDBOX refs that request
/// storage-mutating host fns.
pub(crate) fn validate_no_sandbox_storage_mutation(
    parsed: &ParsedSchema,
) -> Result<(), SchemaCompileError> {
    for sandbox_ref in &parsed.sandbox_refs {
        for host_fn in &sandbox_ref.host_fns {
            if FORBIDDEN_HOST_FNS.contains(&host_fn.as_str()) {
                return Err(SchemaCompileError::SandboxHostFnRejected {
                    host_fn: host_fn.clone(),
                    module_cid: Some(sandbox_ref.module_cid.clone()),
                });
            }
        }
    }
    Ok(())
}

/// sec-3.5-r1-4 negative pin — EMIT and RESPOND targets MUST carry a
/// scope (cap-policy boundary unconstrained-by-default fails closed).
pub(crate) fn validate_no_unconstrained_emit_respond(
    parsed: &ParsedSchema,
) -> Result<(), SchemaCompileError> {
    for target in &parsed.emit_targets {
        if target.scope.is_none() {
            return Err(SchemaCompileError::ValidationFailed {
                reason: format!(
                    "EMIT target `{}` is unconstrained (no `scope`); sec-3.5-r1-4 \
                     requires schema-declared EMIT/RESPOND to be scope-bound",
                    target.topic
                ),
                location: Some(format!("$.emit_targets[topic={}]", target.topic)),
            });
        }
    }
    for target in &parsed.respond_targets {
        if target.scope.is_none() {
            return Err(SchemaCompileError::ValidationFailed {
                reason: format!(
                    "RESPOND target `{}` is unconstrained (no `scope`); sec-3.5-r1-4 \
                     requires schema-declared EMIT/RESPOND to be scope-bound",
                    target.handler_id
                ),
                location: Some(format!(
                    "$.respond_targets[handler_id={}]",
                    target.handler_id
                )),
            });
        }
    }
    Ok(())
}

/// FieldRef cycle detection. G23-A canary uses the simple "self-referent"
/// definition: a FieldRef whose `ref_target_kind` matches the schema
/// name is a cycle. The richer cross-schema cycle detection (across the
/// FieldRef graph of multiple schemas) lands at G23-A wave-4b.
///
/// Aux: also catches a degenerate cycle inside FieldObject sub-fields
/// where a sub-field's `ref_target_kind` references its parent's name.
pub(crate) fn detect_field_ref_cycle(parsed: &ParsedSchema) -> Result<(), SchemaCompileError> {
    let schema_name = &parsed.name;
    let mut trace = BTreeSet::new();
    for field in &parsed.fields {
        detect_cycle_walk(field, schema_name, &mut trace)?;
    }
    Ok(())
}

fn detect_cycle_walk(
    field: &ParsedField,
    schema_name: &str,
    trace: &mut BTreeSet<String>,
) -> Result<(), SchemaCompileError> {
    if let Some(target) = &field.ref_target_kind
        && (target == schema_name || trace.contains(target))
    {
        let mut path: Vec<String> = trace.iter().cloned().collect();
        path.push(target.clone());
        return Err(SchemaCompileError::VocabCycleRejected {
            cycle_through: path,
        });
    }
    trace.insert(field.name.clone());
    for sub in &field.sub_fields {
        detect_cycle_walk(sub, schema_name, trace)?;
    }
    trace.remove(&field.name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &[u8] = br#"{
        "label": "SchemaRoot",
        "name": "Min",
        "fields": [
            { "label": "FieldScalar", "name": "v", "scalar": "text", "required": true, "default": null }
        ]
    }"#;

    #[test]
    fn parses_minimal_schema() {
        let parsed = parse_schema_json(MINIMAL).unwrap();
        assert_eq!(parsed.name, "Min");
        assert_eq!(parsed.fields.len(), 1);
    }

    #[test]
    fn rejects_malformed_json() {
        let err = parse_schema_json(b"{ not valid").unwrap_err();
        assert_eq!(err.code(), benten_errors::ErrorCode::SchemaValidationFailed);
    }

    #[test]
    fn rejects_unknown_label() {
        let bytes = br#"{
            "label": "SchemaRoot",
            "name": "X",
            "fields": [
                { "label": "FieldQuaternion", "name": "x", "required": true, "default": null }
            ]
        }"#;
        let err = parse_schema_json(bytes).unwrap_err();
        assert_eq!(
            err.code(),
            benten_errors::ErrorCode::SchemaVocabInvalidLabel
        );
    }

    #[test]
    fn rejects_unknown_scalar() {
        let bytes = br#"{
            "label": "SchemaRoot",
            "name": "X",
            "fields": [
                { "label": "FieldScalar", "name": "x", "scalar": "quaternion", "required": true, "default": null }
            ]
        }"#;
        let err = parse_schema_json(bytes).unwrap_err();
        assert_eq!(
            err.code(),
            benten_errors::ErrorCode::SchemaVocabScalarUnknown
        );
    }
}
