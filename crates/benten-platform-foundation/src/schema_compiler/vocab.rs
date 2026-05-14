//! Schema vocabulary types per D-4F-NEW-TYPED-FIELD-NODE-VOCAB.
//!
//! These are typed Rust mirrors of the 8 labels / 6 edges / 8 scalars / 4
//! mandatory field properties. Construction sites for these enums are the
//! parser (`parse.rs`) + emitter (`emit.rs`); they're public so downstream
//! consumers (G23-B materializer + tests) can pattern-match against them
//! without going through string-form indirection.

use super::SchemaCompileError;

/// The 8 vocabulary labels (D-4F-NEW-TYPED-FIELD-NODE-VOCAB ratified
/// 2026-05-11 post-R1-triage).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VocabLabel {
    /// Root container of a schema-subgraph; one per schema.
    SchemaRoot,
    /// A primitive value field. Carries a [`Scalar`] discriminant.
    FieldScalar,
    /// A nested object field (composes recursively over schemas).
    FieldObject,
    /// An ordered collection.
    FieldList,
    /// A keyed collection.
    FieldMap,
    /// A cross-content reference, content-CID-keyed.
    FieldRef,
    /// An enumerated choice between named variants.
    FieldEnum,
    /// A tagged union of variant types.
    FieldUnion,
}

impl VocabLabel {
    /// The canonical string form used in JSON-Schema dialect input + in
    /// emitted Node properties (the materializer at G23-B reads this).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            VocabLabel::SchemaRoot => "SchemaRoot",
            VocabLabel::FieldScalar => "FieldScalar",
            VocabLabel::FieldObject => "FieldObject",
            VocabLabel::FieldList => "FieldList",
            VocabLabel::FieldMap => "FieldMap",
            VocabLabel::FieldRef => "FieldRef",
            VocabLabel::FieldEnum => "FieldEnum",
            VocabLabel::FieldUnion => "FieldUnion",
        }
    }

    /// Parse a label string from JSON-Schema input. Returns
    /// [`SchemaCompileError::VocabInvalidLabel`] for anything outside the
    /// 8-set.
    pub fn from_str(s: &str) -> Result<Self, SchemaCompileError> {
        match s {
            "SchemaRoot" => Ok(VocabLabel::SchemaRoot),
            "FieldScalar" => Ok(VocabLabel::FieldScalar),
            "FieldObject" => Ok(VocabLabel::FieldObject),
            "FieldList" => Ok(VocabLabel::FieldList),
            "FieldMap" => Ok(VocabLabel::FieldMap),
            "FieldRef" => Ok(VocabLabel::FieldRef),
            "FieldEnum" => Ok(VocabLabel::FieldEnum),
            "FieldUnion" => Ok(VocabLabel::FieldUnion),
            other => Err(SchemaCompileError::VocabInvalidLabel {
                label: other.to_string(),
                field_name: None,
            }),
        }
    }
}

/// Static catalog of label string-forms — load-bearing for tests that
/// grep-assert the 8-set without instantiating the enum.
pub const VOCAB_LABEL_NAMES: &[&str] = &[
    "SchemaRoot",
    "FieldScalar",
    "FieldObject",
    "FieldList",
    "FieldMap",
    "FieldRef",
    "FieldEnum",
    "FieldUnion",
];

/// The 5 labeled vocabulary edges.
///
/// (Object-to-field relationships are implicit-via-recursion in
/// `schema_compiler::emit` — each `SchemaRoot` / `FieldObject` walks its
/// child `Field*` nodes during emit, so no `FIELD` edge label is minted.
/// See `docs/SCHEMA-DRIVEN-RENDERING.md §2.2` for the parent-child
/// recursion shape.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VocabEdge {
    /// Element type of a FieldList / FieldMap.
    ItemType,
    /// Key type of a FieldMap; → FieldScalar.
    KeyType,
    /// Value type of a FieldMap; → Field*.
    ValueType,
    /// FieldRef → content-CID resolution.
    RefTarget,
    /// FieldEnum / FieldUnion → variant type.
    Variant,
}

impl VocabEdge {
    /// Canonical string form (e.g. `"ITEM_TYPE"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            VocabEdge::ItemType => "ITEM_TYPE",
            VocabEdge::KeyType => "KEY_TYPE",
            VocabEdge::ValueType => "VALUE_TYPE",
            VocabEdge::RefTarget => "REF_TARGET",
            VocabEdge::Variant => "VARIANT",
        }
    }

    /// Parse a string into a `VocabEdge`. Used by the parser when reading
    /// raw schema-JSON.
    pub fn from_str(s: &str) -> Result<Self, SchemaCompileError> {
        match s {
            "ITEM_TYPE" => Ok(VocabEdge::ItemType),
            "KEY_TYPE" => Ok(VocabEdge::KeyType),
            "VALUE_TYPE" => Ok(VocabEdge::ValueType),
            "REF_TARGET" => Ok(VocabEdge::RefTarget),
            "VARIANT" => Ok(VocabEdge::Variant),
            other => Err(SchemaCompileError::VocabEdgeMismatch {
                source_label: "<unknown>".to_string(),
                target_label: "<unknown>".to_string(),
                edge: other.to_string(),
            }),
        }
    }
}

/// Static catalog of edge string-forms.
pub const VOCAB_EDGE_NAMES: &[&str] = &[
    "ITEM_TYPE",
    "KEY_TYPE",
    "VALUE_TYPE",
    "REF_TARGET",
    "VARIANT",
];

/// The 8 scalar types per `docs/SCHEMA-DRIVEN-RENDERING.md §2.3`. Each
/// maps to a `benten_core::Value` variant (or to an int/bytes shape with
/// a flag for the typed-as-X scalars `bytes-cid` / `timestamp-hlc`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scalar {
    /// UTF-8 string. Maps to `Value::Text`.
    Text,
    /// 64-bit signed integer. Maps to `Value::Int`.
    Int,
    /// 64-bit float. Maps to `Value::Float`.
    Float,
    /// Boolean. Maps to `Value::Bool`.
    Bool,
    /// Raw byte array. Maps to `Value::Bytes`.
    Bytes,
    /// Content identifier — bytes carry the CID multibase encoding. Maps
    /// to `Value::Bytes` with a documented CID interpretation.
    BytesCid,
    /// HLC timestamp — int carries the HLC ticks. Maps to `Value::Int`
    /// with HLC interpretation.
    TimestampHlc,
    /// Null. Maps to `Value::Null`.
    Null,
}

impl Scalar {
    /// Canonical string form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Scalar::Text => "text",
            Scalar::Int => "int",
            Scalar::Float => "float",
            Scalar::Bool => "bool",
            Scalar::Bytes => "bytes",
            Scalar::BytesCid => "bytes-cid",
            Scalar::TimestampHlc => "timestamp-hlc",
            Scalar::Null => "null",
        }
    }

    /// Parse a scalar name. Returns
    /// [`SchemaCompileError::VocabScalarUnknown`] for anything outside
    /// the 8-set.
    pub fn from_str(s: &str, field_name: Option<&str>) -> Result<Self, SchemaCompileError> {
        match s {
            "text" => Ok(Scalar::Text),
            "int" => Ok(Scalar::Int),
            "float" => Ok(Scalar::Float),
            "bool" => Ok(Scalar::Bool),
            "bytes" => Ok(Scalar::Bytes),
            "bytes-cid" => Ok(Scalar::BytesCid),
            "timestamp-hlc" => Ok(Scalar::TimestampHlc),
            "null" => Ok(Scalar::Null),
            other => Err(SchemaCompileError::VocabScalarUnknown {
                scalar: other.to_string(),
                field_name: field_name.map(str::to_string),
            }),
        }
    }
}

/// Static catalog of scalar string-forms.
pub const SCALAR_NAMES: &[&str] = &[
    "text",
    "int",
    "float",
    "bool",
    "bytes",
    "bytes-cid",
    "timestamp-hlc",
    "null",
];

/// The 4 mandatory field properties (`name` / `required` / `default` /
/// `scope`). `scope` is schema-DERIVED; the parser does not require the
/// JSON to supply it (and if it does, the value is silently discarded —
/// see `schema_compiler::derive_scope`).
pub const VOCAB_REQUIRED_FIELD_PROPS: &[&str] = &["name", "required", "default", "scope"];

/// Canonical handler-id for a compiled schema-subgraph. Format:
/// `schema:<SchemaName>` — collision-resistant with user handler-ids
/// (which historically use unprefixed names) AND with the reserved
/// `engine:typed:` namespace (CLAUDE.md baked-in #16). The `schema:`
/// prefix is foundation-owned + Phase-4 stable.
pub(crate) fn canonical_handler_id(schema_name: &str) -> String {
    format!("schema:{schema_name}")
}
