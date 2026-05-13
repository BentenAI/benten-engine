//! Schema-compile error type. Each variant carries a structured payload
//! suitable for the runtime ErrorCode mapping (`code()` accessor below).
//!
//! Per §3.5g (cross-language rule-mirror) the typed Rust variants are
//! mirrored as `E_SCHEMA_*` strings on the TS side at
//! `packages/engine/src/errors.generated.ts`. The Rust → TS direction is
//! enforced by `crates/benten-errors/tests/error_codes_g23_a_subset_closure.rs`.

use benten_errors::ErrorCode;
use thiserror::Error;

/// Error type returned by [`crate::schema_compiler::compile`].
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum SchemaCompileError {
    /// Top-level validation failure (malformed JSON / missing field /
    /// unconstrained EMIT/RESPOND target with no scope).
    #[error("schema_compiler: validation failed ({reason}){}", location.as_ref().map(|l| format!(" at {l}")).unwrap_or_default())]
    ValidationFailed {
        /// Human-readable reason.
        reason: String,
        /// Best-effort location pointer (`SchemaRoot.fields[2].name`-style).
        location: Option<String>,
    },

    /// Defensive guard — the emitter produced a kind outside the 12.
    #[error("schema_compiler: emitter would mint new PrimitiveKind `{requested_kind}` outside canonical 12 (CLAUDE.md baked-in #1 violation)")]
    EmitNewPrimitiveRejected {
        /// Requested kind name.
        requested_kind: String,
    },

    /// SANDBOX ref requested storage-mutating host fn.
    #[error("schema_compiler: SANDBOX module reference requests storage-mutating host fn `{host_fn}` — forbidden per CLAUDE.md baked-in #16")]
    SandboxHostFnRejected {
        /// The forbidden host fn name (`kv:write` etc).
        host_fn: String,
        /// Best-effort module CID.
        module_cid: Option<String>,
    },

    /// Label outside the 8-set.
    #[error("schema_compiler: vocabulary label `{label}` is not in the 8-label set")]
    VocabInvalidLabel {
        /// Offending label.
        label: String,
        /// Field name for diagnostics.
        field_name: Option<String>,
    },

    /// Edge outside the 6-set.
    #[error("schema_compiler: edge `{edge}` is not in the 6-edge set (source={source_label}, target={target_label})")]
    VocabEdgeMismatch {
        /// Source label.
        source_label: String,
        /// Target label.
        target_label: String,
        /// Offending edge name.
        edge: String,
    },

    /// Scalar outside the 8-set.
    #[error("schema_compiler: scalar `{scalar}` is not in the 8-scalar vocabulary")]
    VocabScalarUnknown {
        /// Offending scalar.
        scalar: String,
        /// Field for diagnostics.
        field_name: Option<String>,
    },

    /// FieldRef missing or unresolvable target.
    #[error("schema_compiler: FieldRef `{field_name}` has missing/unresolvable ref_target_kind={ref_target_kind:?}")]
    VocabRefTargetMissing {
        /// FieldRef field name.
        field_name: String,
        /// Offending ref-target kind (or None).
        ref_target_kind: Option<String>,
    },

    /// FieldRef cycle.
    #[error("schema_compiler: FieldRef cycle through {cycle_through:?}")]
    VocabCycleRejected {
        /// Cycle-trace.
        cycle_through: Vec<String>,
    },

    /// Field missing a mandatory property.
    #[error("schema_compiler: field `{field_name}` is missing mandatory property `{missing_property}`")]
    VocabRequiredPropertyMissing {
        /// Field name.
        field_name: String,
        /// Missing property (one of `name` / `required` / `default`; not
        /// `scope` — `scope` is derived).
        missing_property: String,
    },
}

impl SchemaCompileError {
    /// Map the typed variant to the stable [`ErrorCode`] from
    /// `benten-errors`. The catalog of 9 ErrorCode strings is the single
    /// source of truth that the §3.5g cross-language rule-mirror anchors
    /// on (Rust enum → as_static_str → string-form → TS mirror).
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            SchemaCompileError::ValidationFailed { .. } => ErrorCode::SchemaValidationFailed,
            SchemaCompileError::EmitNewPrimitiveRejected { .. } => {
                ErrorCode::SchemaEmitNewPrimitiveRejected
            }
            SchemaCompileError::SandboxHostFnRejected { .. } => {
                ErrorCode::SchemaSandboxHostFnRejected
            }
            SchemaCompileError::VocabInvalidLabel { .. } => ErrorCode::SchemaVocabInvalidLabel,
            SchemaCompileError::VocabEdgeMismatch { .. } => ErrorCode::SchemaVocabEdgeMismatch,
            SchemaCompileError::VocabScalarUnknown { .. } => ErrorCode::SchemaVocabScalarUnknown,
            SchemaCompileError::VocabRefTargetMissing { .. } => {
                ErrorCode::SchemaVocabRefTargetMissing
            }
            SchemaCompileError::VocabCycleRejected { .. } => ErrorCode::SchemaVocabCycleRejected,
            SchemaCompileError::VocabRequiredPropertyMissing { .. } => {
                ErrorCode::SchemaVocabRequiredPropertyMissing
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_variant_maps_to_distinct_error_code() {
        // Sample one of each variant to confirm the catalog mapping is
        // exhaustive + injective at G23-A canary.
        let variants = [
            SchemaCompileError::ValidationFailed {
                reason: "x".into(),
                location: None,
            },
            SchemaCompileError::EmitNewPrimitiveRejected {
                requested_kind: "Foo".into(),
            },
            SchemaCompileError::SandboxHostFnRejected {
                host_fn: "kv:write".into(),
                module_cid: None,
            },
            SchemaCompileError::VocabInvalidLabel {
                label: "X".into(),
                field_name: None,
            },
            SchemaCompileError::VocabEdgeMismatch {
                source_label: "A".into(),
                target_label: "B".into(),
                edge: "X".into(),
            },
            SchemaCompileError::VocabScalarUnknown {
                scalar: "x".into(),
                field_name: None,
            },
            SchemaCompileError::VocabRefTargetMissing {
                field_name: "x".into(),
                ref_target_kind: None,
            },
            SchemaCompileError::VocabCycleRejected {
                cycle_through: vec!["a".into(), "b".into()],
            },
            SchemaCompileError::VocabRequiredPropertyMissing {
                field_name: "x".into(),
                missing_property: "name".into(),
            },
        ];
        let mut codes = std::collections::HashSet::new();
        for v in &variants {
            assert!(
                codes.insert(v.code().as_str().to_string()),
                "duplicate code for variant {v:?}"
            );
        }
        assert_eq!(codes.len(), 9, "G23-A mints exactly 9 ErrorCodes");
    }
}
