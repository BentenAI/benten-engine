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
}
