//! [`SchemaSubgraphSpec`] — the output of the schema compiler.
//!
//! Wraps a [`benten_core::Subgraph`] (the canonical-bytes-shape engine
//! input) plus enough metadata for round-trip + diagnostics. The engine
//! handoff path uses [`SchemaSubgraphSpec::into_subgraph`]; the existing
//! `benten-engine::IntoSubgraphSpec for benten_eval::Subgraph` impl makes
//! `engine.register_subgraph(spec.into_subgraph())` a one-line registration
//! that DOES NOT widen the engine API (arch-r1-15).

use benten_core::Subgraph;

use super::emit::PrimitiveDescriptor;
use super::parse::ParsedSchema;

/// Output of [`crate::schema_compiler::compile`] — a content-addressed
/// schema-subgraph ready for engine registration.
#[derive(Debug, Clone)]
pub struct SchemaSubgraphSpec {
    parsed: ParsedSchema,
    subgraph: Subgraph,
    /// Per-primitive metadata in emit order — mirrors `subgraph.nodes()`
    /// and exposes the cap-scope annotation each primitive carries (so
    /// tests + the G23-B materializer can inspect without re-deriving).
    descriptors: Vec<PrimitiveDescriptor>,
}

impl SchemaSubgraphSpec {
    pub(crate) fn new(parsed: ParsedSchema, subgraph: Subgraph) -> Self {
        let descriptors = PrimitiveDescriptor::derive_for(&subgraph);
        Self {
            parsed,
            subgraph,
            descriptors,
        }
    }

    /// Read-only access to the emitted Subgraph — round-trip via
    /// `benten_core::canonical_subgraph_bytes(spec.as_subgraph())`.
    #[must_use]
    pub fn as_subgraph(&self) -> &Subgraph {
        &self.subgraph
    }

    /// Consume + return the underlying Subgraph for engine handoff.
    /// `engine.register_subgraph(spec.into_subgraph())` uses this path.
    #[must_use]
    pub fn into_subgraph(self) -> Subgraph {
        self.subgraph
    }

    /// Schema name (top-level `name` field).
    #[must_use]
    pub fn schema_name(&self) -> &str {
        &self.parsed.name
    }

    /// Per-primitive descriptors in emit order. Equal in length to
    /// `self.as_subgraph().nodes()`.
    #[must_use]
    pub fn primitives(&self) -> &[PrimitiveDescriptor] {
        &self.descriptors
    }

    /// Handler-id the Subgraph will register under.
    #[must_use]
    pub fn handler_id(&self) -> &str {
        self.subgraph.handler_id()
    }

    /// G24-A wave-completion sweep closes phase-4-backlog §4.13 mr-4
    /// MAJOR: defense-in-depth integration-test surface.
    ///
    /// The materializer entry-point at `materializer.rs:905-921` has a
    /// re-check for `SchemaSubgraphSpec` inputs whose SANDBOX node
    /// references a banned storage-mutating host-fn. The PRIMARY
    /// defense is at `schema_compiler::compile` (G23-A), which refuses
    /// such schemas during parse. This `for_test_*` constructor lets
    /// integration tests construct a hand-authored spec that bypasses
    /// the parse-time defense, so the materializer-entry-arm can be
    /// exercised in isolation (closes mr-4 G24-A acceptance).
    ///
    /// **Not part of the stable public API.** Marked `#[doc(hidden)]`.
    /// Production callers use [`crate::schema_compiler::compile`].
    #[doc(hidden)]
    #[must_use]
    pub fn for_test_from_handcoded_subgraph(
        schema_name: impl Into<String>,
        subgraph: Subgraph,
    ) -> Self {
        let descriptors = PrimitiveDescriptor::derive_for(&subgraph);
        Self {
            parsed: ParsedSchema::for_test_empty(schema_name.into()),
            subgraph,
            descriptors,
        }
    }
}
