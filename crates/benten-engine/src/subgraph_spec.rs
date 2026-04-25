//! DSL-friendly `SubgraphSpec` / `WriteSpec` builders + the conversion traits
//! that sugar `Engine::register_subgraph` and `Engine::call` inputs.
//!
//! Extracted from `lib.rs` by R6 Wave 2 (R-major-01). The types moved
//! verbatim; their public surface is unchanged.

use std::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_eval::RegistrationError;

use crate::error::EngineError;

// ---------------------------------------------------------------------------
// SubgraphSpec + SubgraphSpecBuilder + WriteSpec
// ---------------------------------------------------------------------------

/// DSL-friendly specification passed to `Engine::register_subgraph`.
///
/// Records the handler id, the ordered list of primitive kinds (so the
/// invariant validator can see the subgraph's shape) and the per-WRITE
/// payload (label, properties, requires scope, failure-injection flag) so
/// `Engine::call` can actually dispatch. Fix for philosophy finding
/// `g7-ep-1` — the v1 builder dropped every WriteSpec field on the floor.
#[derive(Debug, Clone)]
pub struct SubgraphSpec {
    pub(crate) handler_id: String,
    pub(crate) primitives: Vec<(String, benten_eval::PrimitiveKind)>,
    /// Per-WRITE payload, indexed in registration order. `primitives` refers
    /// to this list via its `Write` entries; non-Write primitives don't
    /// appear here.
    pub(crate) write_specs: Vec<WriteSpec>,
}

impl SubgraphSpec {
    #[must_use]
    pub fn builder() -> SubgraphSpecBuilder {
        SubgraphSpecBuilder::new()
    }

    /// Read-only access to the handler id.
    #[must_use]
    pub fn handler_id(&self) -> &str {
        &self.handler_id
    }

    /// Read-only access to the recorded WriteSpecs (for tests + diagnostics).
    #[must_use]
    pub fn write_specs(&self) -> &[WriteSpec] {
        &self.write_specs
    }

    /// Convenience: build an empty SubgraphSpec (no primitives) with just a
    /// handler id. Used by the testing fixtures for shape-only tests that
    /// don't exercise the primitive dispatch path.
    pub(crate) fn empty(handler_id: impl Into<String>) -> Self {
        Self {
            handler_id: handler_id.into(),
            primitives: Vec::new(),
            write_specs: Vec::new(),
        }
    }
}

/// DSL builder that produces a [`SubgraphSpec`]. Calling `write(|w| w.label
/// (...).property(...))` stores the configured `WriteSpec` so downstream
/// dispatch can see exactly what the caller requested.
pub struct SubgraphSpecBuilder {
    handler_id: String,
    primitives: Vec<(String, benten_eval::PrimitiveKind)>,
    write_specs: Vec<WriteSpec>,
}

impl SubgraphSpecBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handler_id: String::new(),
            primitives: Vec::new(),
            write_specs: Vec::new(),
        }
    }

    #[must_use]
    pub fn handler_id(mut self, id: &str) -> Self {
        self.handler_id = id.to_string();
        self
    }

    #[must_use]
    pub fn iterate<F>(self, _max: u32, _body: F) -> Self
    where
        F: FnOnce(IterateBody) -> IterateBody,
    {
        // Phase-1: ITERATE bodies aren't executed by `call` yet; the structural
        // shape is what gets registered. Leave the builder's primitive list
        // untouched so invariant-1 (DAG-ness) stays trivially satisfied.
        self
    }

    #[must_use]
    pub fn write<F>(mut self, f: F) -> Self
    where
        F: FnOnce(WriteSpec) -> WriteSpec,
    {
        let spec = f(WriteSpec::new());
        self.primitives.push((
            format!("w{}", self.primitives.len()),
            benten_eval::PrimitiveKind::Write,
        ));
        self.write_specs.push(spec);
        self
    }

    #[must_use]
    pub fn respond(mut self) -> Self {
        self.primitives.push((
            format!("r{}", self.primitives.len()),
            benten_eval::PrimitiveKind::Respond,
        ));
        self
    }

    /// Register an arbitrary primitive kind by label. Used by the napi
    /// JSON-shape decoder so hand-built DSL subgraphs that use any of
    /// the 12 primitive types (not just `write` / `respond`) can
    /// structurally register. The evaluator returns
    /// `E_PRIMITIVE_NOT_IMPLEMENTED` for Phase-2-only kinds at call
    /// time; registration merely preserves the shape.
    #[must_use]
    pub fn primitive(mut self, id: &str, kind: benten_eval::PrimitiveKind) -> Self {
        self.primitives.push((id.to_string(), kind));
        self
    }

    #[must_use]
    pub fn build(self) -> SubgraphSpec {
        SubgraphSpec {
            handler_id: self.handler_id,
            primitives: self.primitives,
            write_specs: self.write_specs,
        }
    }
}

impl Default for SubgraphSpecBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// DSL body object handed to `iterate(|body| ...)`.
pub struct IterateBody;

impl IterateBody {
    #[must_use]
    pub fn write<F>(self, _f: F) -> Self
    where
        F: FnOnce(WriteSpec) -> WriteSpec,
    {
        self
    }
}

/// DSL object handed to `write(|w| ...)`.
///
/// Records the label, property set, capability-scope requirements, and
/// failure-injection flag so `Engine::call` can dispatch the write with the
/// caller's intent rather than a stripped facade.
#[derive(Debug, Clone, Default)]
pub struct WriteSpec {
    pub(crate) label: String,
    pub(crate) properties: BTreeMap<String, Value>,
    pub(crate) requires: Vec<String>,
    pub(crate) inject_failure: bool,
}

impl WriteSpec {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    #[must_use]
    pub fn property(mut self, k: &str, v: Value) -> Self {
        self.properties.insert(k.to_string(), v);
        self
    }

    #[must_use]
    pub fn requires(mut self, scope: &str) -> Self {
        self.requires.push(scope.to_string());
        self
    }

    /// Test-only failure-injection toggle.
    ///
    /// Exposed on the public builder so integration tests in sibling crates
    /// (`benten-engine/tests/integration/*`) can trip the `ON_ERROR` edge
    /// with `E_TX_ABORTED` without reaching into private internals.
    /// Production code paths never set this; Phase-2 gates behind
    /// `#[cfg(any(test, feature = "testing"))]` once the integration
    /// layout stabilises (R-minor-04).
    #[must_use]
    pub fn test_inject_failure(mut self, inject: bool) -> Self {
        self.inject_failure = inject;
        self
    }

    /// Read-only accessor for the configured label.
    #[must_use]
    pub fn label_ref(&self) -> &str {
        &self.label
    }

    /// Read-only accessor for the configured property set.
    #[must_use]
    pub fn properties_ref(&self) -> &BTreeMap<String, Value> {
        &self.properties
    }
}

// ---------------------------------------------------------------------------
// Helper trait adapters for overloaded register / grant / call arguments.
// ---------------------------------------------------------------------------

/// Accepts `SubgraphSpec`, `&SubgraphSpec`, and `benten_eval::Subgraph`.
/// The `into_eval_subgraph` method converts any of them into the lower-level
/// `Subgraph` shape the G6 invariant validator consumes.
pub trait IntoSubgraphSpec {
    fn into_eval_subgraph(self) -> Result<benten_eval::Subgraph, EngineError>;

    /// Return a clone of the underlying `SubgraphSpec` when the input is one;
    /// otherwise `None`. Used so `register_subgraph` can cache the spec for
    /// later `call()` dispatch.
    fn as_subgraph_spec(&self) -> Option<SubgraphSpec> {
        None
    }
}

impl IntoSubgraphSpec for SubgraphSpec {
    fn as_subgraph_spec(&self) -> Option<SubgraphSpec> {
        Some(self.clone())
    }
    fn into_eval_subgraph(self) -> Result<benten_eval::Subgraph, EngineError> {
        // Construct a minimal Subgraph from the collected primitives so the
        // invariant validator can run.
        //
        // R6 cag-r6-2: dispatch on every PrimitiveKind so each kind
        // produces a structurally-distinct OperationNode shape — the prior
        // wildcard `_ => sb.read(id)` silently degraded 9 of 12 primitives
        // to READ, collapsing distinct registered subgraphs onto identical
        // CIDs (Inv-13 immutability collision surface). Read/Write/Respond
        // retain their kind-specific helpers so existing tests keep their
        // node-shape expectations; every other kind routes through the
        // raw `push_primitive` builder method which materializes the
        // declared kind verbatim.
        let mut sb = benten_eval::SubgraphBuilder::new(self.handler_id);
        let mut last: Option<benten_eval::NodeHandle> = None;
        for (id, kind) in self.primitives {
            let h = match kind {
                benten_eval::PrimitiveKind::Write => sb.write(id),
                benten_eval::PrimitiveKind::Read => sb.read(id),
                benten_eval::PrimitiveKind::Respond => {
                    // `respond` is terminal and MUST have a predecessor so the
                    // registered subgraph's CID matches user intent (no
                    // silently-fabricated synthetic READ). Fix for
                    // code-reviewer finding g7-cr-13.
                    let Some(prev) = last else {
                        return Err(EngineError::Invariant(Box::new(RegistrationError::new(
                            benten_eval::InvariantViolation::Registration,
                        ))));
                    };
                    sb.respond(prev)
                }
                other => sb.push_primitive(id, other),
            };
            if let Some(p) = last {
                sb.add_edge(p, h);
            }
            last = Some(h);
        }
        Ok(sb.build_unvalidated_for_test())
    }
}

impl IntoSubgraphSpec for &SubgraphSpec {
    fn as_subgraph_spec(&self) -> Option<SubgraphSpec> {
        Some((*self).clone())
    }
    fn into_eval_subgraph(self) -> Result<benten_eval::Subgraph, EngineError> {
        self.clone().into_eval_subgraph()
    }
}

impl IntoSubgraphSpec for benten_eval::Subgraph {
    fn into_eval_subgraph(self) -> Result<benten_eval::Subgraph, EngineError> {
        Ok(self)
    }
}

impl IntoSubgraphSpec for &benten_eval::Subgraph {
    fn into_eval_subgraph(self) -> Result<benten_eval::Subgraph, EngineError> {
        Ok(self.clone())
    }
}

/// Subject arg for `grant_capability`.
pub trait GrantSubject {
    fn as_value(&self) -> Value;
}

impl GrantSubject for &Cid {
    fn as_value(&self) -> Value {
        Value::Bytes(self.as_bytes().to_vec())
    }
}

impl GrantSubject for Cid {
    fn as_value(&self) -> Value {
        Value::Bytes(self.as_bytes().to_vec())
    }
}

impl GrantSubject for &str {
    fn as_value(&self) -> Value {
        Value::Text((*self).to_string())
    }
}

impl GrantSubject for &String {
    fn as_value(&self) -> Value {
        Value::Text((*self).clone())
    }
}

pub trait RevokeSubject {
    fn as_value(&self) -> Value;
}

impl RevokeSubject for &Cid {
    fn as_value(&self) -> Value {
        Value::Bytes(self.as_bytes().to_vec())
    }
}

impl RevokeSubject for Cid {
    fn as_value(&self) -> Value {
        Value::Bytes(self.as_bytes().to_vec())
    }
}

impl RevokeSubject for &str {
    fn as_value(&self) -> Value {
        Value::Text((*self).to_string())
    }
}

pub trait RevokeScope {
    fn as_scope_string(&self) -> String;
}

impl RevokeScope for &str {
    fn as_scope_string(&self) -> String {
        (*self).to_string()
    }
}

impl RevokeScope for &String {
    fn as_scope_string(&self) -> String {
        (*self).clone()
    }
}

impl RevokeScope for String {
    fn as_scope_string(&self) -> String {
        self.clone()
    }
}

/// Call-input overload — accept `Node`, default `()`, and the
/// `BTreeMap<String, benten_core::Value>` path some R3 tests build inline.
pub trait IntoCallInput {
    /// Convert into a Node for uniform downstream handling.
    fn into_node(self) -> Node;
}
impl IntoCallInput for Node {
    fn into_node(self) -> Node {
        self
    }
}
impl IntoCallInput for () {
    fn into_node(self) -> Node {
        Node::empty()
    }
}
impl IntoCallInput for BTreeMap<String, Value> {
    fn into_node(self) -> Node {
        Node::new(Vec::new(), self)
    }
}
