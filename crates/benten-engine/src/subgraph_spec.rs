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
// SubgraphSpec + SubgraphSpecBuilder + PrimitiveSpec + WriteSpec
// ---------------------------------------------------------------------------

/// Per-primitive declaration carried by [`SubgraphSpec::primitives`]. Each
/// entry pairs the canonical [`benten_eval::PrimitiveKind`] tag with a
/// per-primitive **properties bag** (`BTreeMap<String, Value>` —
/// sorted-by-key for canonical-bytes deterministic ordering per
/// sec-pre-r1-09; typed `Value` rather than generic CBOR passthrough for
/// the same reason).
///
/// The bag carries the configuration each primitive declares at registration
/// time — e.g. WRITE's `label` / `requires` / `inject_failure` / user
/// properties; WAIT's `ttl_hours` / `wallclock_ms`; SANDBOX's `module` /
/// `wallclock_ms` / `output_limit`; SUBSCRIBE's `pattern`. New primitives
/// (G6 STREAM/SUBSCRIBE, G7 SANDBOX, G10 module manifest) plug in by
/// declaring their config here, alongside the canonical `kind` tag.
///
/// Phase-2b D6-RESOLVED: BTreeMap+Value, NOT generic CBOR passthrough. See
/// plan §5 D6 + sec-pre-r1-09.
#[derive(Debug, Clone)]
pub struct PrimitiveSpec {
    /// Stable per-primitive id within the subgraph (e.g. `"w0"`, `"r1"`).
    /// Not the CID; the CID derives from the constructed [`Subgraph`]'s
    /// canonical bytes.
    ///
    /// [`Subgraph`]: benten_core::Subgraph
    pub id: String,
    /// Which of the 12 operation primitives this entry represents.
    pub kind: benten_eval::PrimitiveKind,
    /// Per-primitive configuration bag. Empty for primitives without
    /// declared config (e.g. plain READ, plain RESPOND). Sorted by key
    /// at canonical-bytes encode time via `BTreeMap`'s ordered iteration.
    pub properties: BTreeMap<String, Value>,
}

impl PrimitiveSpec {
    /// Build a `PrimitiveSpec` with an empty properties bag — the most
    /// common shape for primitives without declared config.
    #[must_use]
    pub fn new(id: impl Into<String>, kind: benten_eval::PrimitiveKind) -> Self {
        Self {
            id: id.into(),
            kind,
            properties: BTreeMap::new(),
        }
    }

    /// Set a property in the bag (replacing any existing value at that key).
    /// Builder-style for ergonomic chaining.
    #[must_use]
    pub fn with_property(mut self, key: impl Into<String>, value: Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }
}

// Property-bag keys used by the D6-RESOLVED widening to fold WriteSpec config
// into a `PrimitiveSpec` of kind=Write. Centralised so both the producer
// (`SubgraphSpecBuilder::write`) and the consumers (`Engine::subgraph_for_spec`,
// `SubgraphSpec::write_specs()`) use the same string keys.
//
// These keys are CANONICAL — they are folded into `PrimitiveSpec.properties`
// at registration and read back by the engine consumer. Renaming any of them
// silently breaks the WRITE dispatch path. Phase-3 sync will need to preserve
// these key names across protocol versions.
pub(crate) const WRITE_PROP_LABEL: &str = "_label";
pub(crate) const WRITE_PROP_REQUIRES: &str = "_requires";
pub(crate) const WRITE_PROP_INJECT_FAILURE: &str = "_inject_failure";
pub(crate) const WRITE_PROP_USER_PROPERTIES: &str = "_user_properties";

/// DSL-friendly specification passed to `Engine::register_subgraph`.
///
/// Records the handler id and the ordered list of [`PrimitiveSpec`]s — each
/// primitive carries both the canonical `kind` tag (so the invariant
/// validator can see the subgraph's shape) and a per-primitive **properties
/// bag** (so `Engine::call` can dispatch with the caller's intent). Fix for
/// philosophy finding `g7-ep-1` — the v1 builder dropped every WriteSpec
/// field on the floor.
///
/// Phase-2b G12-D widening: `primitives` carries `Vec<PrimitiveSpec>`
/// (D6-RESOLVED). The legacy parallel `write_specs: Vec<WriteSpec>` field is
/// retired — WRITE config now lives inside the `primitives` entries
/// themselves under the `WRITE_PROP_*` keys. The
/// [`SubgraphSpec::write_specs`] accessor synthesises a `Vec<WriteSpec>`
/// from those entries for back-compat reads.
#[derive(Debug, Clone)]
pub struct SubgraphSpec {
    pub(crate) handler_id: String,
    /// Ordered per-primitive declarations. Walked by
    /// `Engine::subgraph_for_spec` to materialise the runnable Subgraph.
    pub(crate) primitives: Vec<PrimitiveSpec>,
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

    /// Read-only access to the recorded primitives (for tests + diagnostics).
    #[must_use]
    pub fn primitives(&self) -> &[PrimitiveSpec] {
        &self.primitives
    }

    /// Synthesise the legacy `Vec<WriteSpec>` view from the widened
    /// `primitives` storage. Each `PrimitiveSpec { kind: Write, properties }`
    /// entry produces one `WriteSpec` with `label` / `requires` /
    /// `inject_failure` / user `properties` lifted out of the bag under the
    /// `WRITE_PROP_*` keys.
    ///
    /// Provided as a back-compat read-only accessor — pre-G12-D code paths
    /// that inspected `spec.write_specs()` keep working unchanged. Mutation
    /// has moved into the `primitives` bag (the source of truth), so this
    /// returns an owned `Vec` rather than a borrowed slice.
    #[must_use]
    pub fn write_specs(&self) -> Vec<WriteSpec> {
        self.primitives
            .iter()
            .filter(|p| matches!(p.kind, benten_eval::PrimitiveKind::Write))
            .map(WriteSpec::from_primitive_spec)
            .collect()
    }

    /// Convenience: build an empty SubgraphSpec (no primitives) with just a
    /// handler id. Used by the testing fixtures for shape-only tests that
    /// don't exercise the primitive dispatch path.
    pub(crate) fn empty(handler_id: impl Into<String>) -> Self {
        Self {
            handler_id: handler_id.into(),
            primitives: Vec::new(),
        }
    }
}

/// DSL builder that produces a [`SubgraphSpec`]. Calling `write(|w| w.label
/// (...).property(...))` records a [`PrimitiveSpec`] of kind=Write whose
/// properties bag carries the configured WriteSpec fields under the
/// `WRITE_PROP_*` canonical keys.
pub struct SubgraphSpecBuilder {
    handler_id: String,
    primitives: Vec<PrimitiveSpec>,
}

impl SubgraphSpecBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handler_id: String::new(),
            primitives: Vec::new(),
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
        let id = format!("w{}", self.primitives.len());
        self.primitives.push(spec.into_primitive_spec(id));
        self
    }

    #[must_use]
    pub fn respond(mut self) -> Self {
        let id = format!("r{}", self.primitives.len());
        self.primitives
            .push(PrimitiveSpec::new(id, benten_eval::PrimitiveKind::Respond));
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
        self.primitives.push(PrimitiveSpec::new(id, kind));
        self
    }

    /// Register a primitive declaration with a pre-built per-primitive
    /// properties bag — the widened-shape entry point for callers that need
    /// to declare per-primitive config (G6/G7/G10 future primitives). The
    /// bag keys are primitive-specific and the canonical-bytes encoding
    /// preserves their `BTreeMap` ordering.
    #[must_use]
    pub fn primitive_with_props(mut self, spec: PrimitiveSpec) -> Self {
        self.primitives.push(spec);
        self
    }

    #[must_use]
    pub fn build(self) -> SubgraphSpec {
        SubgraphSpec {
            handler_id: self.handler_id,
            primitives: self.primitives,
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

    /// Fold this `WriteSpec` into a `PrimitiveSpec` of kind=Write whose
    /// properties bag carries the WriteSpec fields under the `WRITE_PROP_*`
    /// canonical keys. Used by `SubgraphSpecBuilder::write` to fold legacy
    /// WriteSpec construction into the widened `primitives` storage.
    pub(crate) fn into_primitive_spec(self, id: String) -> PrimitiveSpec {
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        if !self.label.is_empty() {
            props.insert(WRITE_PROP_LABEL.into(), Value::Text(self.label));
        }
        if !self.requires.is_empty() {
            props.insert(
                WRITE_PROP_REQUIRES.into(),
                Value::List(self.requires.into_iter().map(Value::Text).collect()),
            );
        }
        if self.inject_failure {
            props.insert(WRITE_PROP_INJECT_FAILURE.into(), Value::Bool(true));
        }
        if !self.properties.is_empty() {
            props.insert(
                WRITE_PROP_USER_PROPERTIES.into(),
                Value::Map(self.properties),
            );
        }
        PrimitiveSpec {
            id,
            kind: benten_eval::PrimitiveKind::Write,
            properties: props,
        }
    }

    /// Reconstruct a `WriteSpec` from a `PrimitiveSpec` of kind=Write —
    /// the inverse of `into_primitive_spec`. Used by
    /// `SubgraphSpec::write_specs()` to synthesise the back-compat
    /// `Vec<WriteSpec>` view from the widened `primitives` storage.
    ///
    /// Tolerant of partial / missing keys (an empty bag yields a default
    /// WriteSpec); unknown bag keys are silently ignored so a Phase-2c
    /// addition of a new WriteSpec field can safely extend the bag without
    /// breaking older readers.
    pub(crate) fn from_primitive_spec(spec: &PrimitiveSpec) -> Self {
        let mut out = WriteSpec::default();
        if let Some(Value::Text(label)) = spec.properties.get(WRITE_PROP_LABEL) {
            out.label.clone_from(label);
        }
        if let Some(Value::List(reqs)) = spec.properties.get(WRITE_PROP_REQUIRES) {
            out.requires = reqs
                .iter()
                .filter_map(|v| match v {
                    Value::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();
        }
        if matches!(
            spec.properties.get(WRITE_PROP_INJECT_FAILURE),
            Some(Value::Bool(true))
        ) {
            out.inject_failure = true;
        }
        if let Some(Value::Map(user)) = spec.properties.get(WRITE_PROP_USER_PROPERTIES) {
            out.properties = user.clone();
        }
        out
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
        for ps in self.primitives {
            let PrimitiveSpec { id, kind, .. } = ps;
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
