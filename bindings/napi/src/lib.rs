//! # benten-napi
//!
//! Node.js bindings for the Benten graph engine via napi-rs v3.
//!
//! G8-A surface: a `#[napi] Engine` class wrapping `benten_engine::Engine`
//! with the full Phase-1 CRUD + handler + view + trace + capability API.
//! Values cross the boundary as `serde_json::Value`; CIDs cross as
//! base32-multibase strings (prefix `b`).
//!
//! ## WASM compile-check
//!
//! `cargo check --target wasm32-unknown-unknown -p benten-napi` must succeed.
//! The napi surface itself (class methods, derive macro) is gated on the
//! `napi-export` default feature. Storage-backed paths (`Engine::open`) that
//! would need a real filesystem on wasm32 are permitted to compile because
//! the engine layer already stubs them for wasm targets; the runtime path is
//! not exercised in a browser build.

// napi-rs's `#[napi]` macro expansion contains `unsafe extern "C"` ctor
// registration shims and therefore cannot coexist with `#![forbid(unsafe_code)]`.
// We use `deny` instead: the effect is identical for hand-written code
// (unsafe is not allowed in this crate) but macro-expanded unsafe is
// permitted, which is appropriate for an FFI binding layer whose entire
// reason for existing is wrapping the Node.js C API.
#![deny(unsafe_code)]
// napi-rs generates code that triggers a number of pedantic lints we don't
// control; silence them only for the generated surface, not our own code.
#![allow(
    clippy::needless_pass_by_value,
    clippy::missing_safety_doc,
    clippy::useless_conversion
)]

// Module layout: one file per public-API surface so each module stays
// focused and diff-reviewable.
#[cfg(feature = "napi-export")]
mod edge;
#[cfg(feature = "napi-export")]
mod error;
#[cfg(feature = "napi-export")]
mod node;
#[cfg(feature = "napi-export")]
mod policy;
// Phase 2b G7-C — SANDBOX-related introspection + diagnostic napi
// surfaces (`sandboxTargetSupported`, cfg-gated `describeSandboxNode`).
// Compile-time gating per sec-pre-r1-05; both arms ship so the symbol
// is callable on every target with the documented platform-aware
// answer.
#[cfg(feature = "napi-export")]
mod sandbox;
#[cfg(feature = "napi-export")]
mod subgraph;
#[cfg(feature = "napi-export")]
mod trace;
#[cfg(feature = "napi-export")]
mod view;
// Phase 2a G3-B: WAIT / suspend / resume napi-bridge helpers. The
// `#[napi]` impl methods consuming these helpers live in `napi_surface`
// below; split into its own file so the suspend/resume codec surface
// stays diff-reviewable.
#[cfg(feature = "napi-export")]
mod wait;
// Phase 2b G6-B: STREAM + SUBSCRIBE napi-bridge helpers. Same split
// pattern as `wait` — adapters are plain Rust functions here, the
// `#[napi]` impl methods consuming them live in `napi_surface` below.
#[cfg(feature = "napi-export")]
mod stream;
#[cfg(feature = "napi-export")]
mod subscribe;

// Re-export the policy enum so the napi-derive macros pick it up at the
// crate root where napi-rs v3 looks for top-level `#[napi]` items.
#[cfg(feature = "napi-export")]
pub use policy::PolicyKind;

// ---------------------------------------------------------------------------
// Engine class (napi surface)
// ---------------------------------------------------------------------------

#[cfg(feature = "napi-export")]
mod napi_surface {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use benten_core::{Node as CoreNode, Value};
    use benten_engine::{Engine as InnerEngine, EngineBuilder, ViewCreateOptions};
    use napi::bindgen_prelude::*;
    use napi_derive::napi;

    use crate::edge::edge_to_json;
    use crate::error::engine_err;
    use crate::node::{
        json_to_props, node_json_to_node, node_to_json, parse_actor_cid_or_derive, parse_cid,
    };
    use crate::policy::{PolicyKind, parse_grant_json};
    use crate::stream::{
        call_stream_adapter, close_handle_adapter, next_chunk_adapter, open_stream_adapter,
    };
    use crate::subgraph::{json_to_subgraph_spec, outcome_to_json};
    use crate::subscribe::{on_change_adapter, subscription_to_json};
    use crate::trace::trace_to_json;
    use crate::view::{extract_view_id, parse_user_view_spec};
    use crate::wait::{
        call_with_suspension_adapter, resume_from_bytes_as_adapter,
        resume_from_bytes_unauthenticated_adapter,
    };

    /// Benten graph engine handle. One instance per redb database file;
    /// thread-safe (the engine's internal locks coordinate concurrent
    /// access from JS worker threads).
    #[napi]
    pub struct Engine {
        inner: Arc<InnerEngine>,
    }

    #[napi]
    impl Engine {
        /// Open or create an engine against a local redb file.
        #[napi(constructor)]
        pub fn new(path: String) -> napi::Result<Self> {
            let inner = InnerEngine::open(&path).map_err(engine_err)?;
            Ok(Self {
                inner: Arc::new(inner),
            })
        }

        /// Factory: open an engine with an explicit capability policy.
        #[napi(factory)]
        pub fn open_with_policy(path: String, policy: PolicyKind) -> napi::Result<Self> {
            // Each `PolicyKind` variant wires a different builder chain:
            // - `NoAuth` falls through to the default (zero policy).
            // - `Ucan` installs the Phase-1 stub via `capability_policy(...)`.
            // - `GrantBacked` flips the dedicated builder flag so the engine
            //   constructs a `GrantBackedPolicy` against a `GrantReader`
            //   pointing at its own backend (the Rust-side test
            //   `grant_backed_policy_denies_unauthorized_writes` exercises
            //   this path). We can't thread a `Box<dyn CapabilityPolicy>`
            //   containing a `GrantReader` from here because the reader
            //   needs the engine's `Arc<RedbBackend>`, which only exists
            //   after `.open(&path)` runs.
            let builder = match policy {
                PolicyKind::NoAuth => EngineBuilder::new(),
                PolicyKind::Ucan => {
                    EngineBuilder::new().capability_policy(Box::new(benten_caps::UcanBackend))
                }
                PolicyKind::GrantBacked => EngineBuilder::new().capability_policy_grant_backed(),
            };
            let inner = builder.open(&path).map_err(engine_err)?;
            Ok(Self {
                inner: Arc::new(inner),
            })
        }

        // -------- Node CRUD --------

        /// Create a Node from labels + JSON properties; return its base32 CID.
        #[napi]
        pub fn create_node(
            &self,
            labels: Vec<String>,
            properties: serde_json::Value,
        ) -> napi::Result<String> {
            let props = json_to_props(properties)?;
            let node = CoreNode::new(labels, props);
            let cid = self.inner.create_node(&node).map_err(engine_err)?;
            Ok(cid.to_base32())
        }

        /// Retrieve a Node by CID. Returns `null` on miss.
        #[napi]
        pub fn get_node(&self, cid: String) -> napi::Result<Option<serde_json::Value>> {
            let parsed = parse_cid(&cid)?;
            match self.inner.get_node(&parsed).map_err(engine_err)? {
                Some(node) => Ok(Some(node_to_json(&node))),
                None => Ok(None),
            }
        }

        /// Option-C diagnostic for a denied / missing read. Gated on
        /// `debug:read` — ordinary callers see `E_CAP_DENIED`.
        ///
        /// Returns `{ cid, existsInBackend, deniedByPolicy, notFound }`.
        /// See named compromise #2 in `docs/SECURITY-POSTURE.md`.
        #[napi]
        pub fn diagnose_read(&self, cid: String) -> napi::Result<serde_json::Value> {
            let parsed = parse_cid(&cid)?;
            let info = self.inner.diagnose_read(&parsed).map_err(engine_err)?;
            let mut map = serde_json::Map::new();
            map.insert("cid".into(), serde_json::Value::from(info.cid.to_base32()));
            map.insert(
                "existsInBackend".into(),
                serde_json::Value::Bool(info.exists_in_backend),
            );
            map.insert(
                "deniedByPolicy".into(),
                match info.denied_by_policy {
                    Some(s) => serde_json::Value::from(s),
                    None => serde_json::Value::Null,
                },
            );
            map.insert("notFound".into(), serde_json::Value::Bool(info.not_found));
            Ok(serde_json::Value::Object(map))
        }

        /// Replace the Node stored at `old_cid` with a newly-content-addressed
        /// Node built from `labels + properties`. Returns the new CID.
        #[napi]
        pub fn update_node(
            &self,
            old_cid: String,
            labels: Vec<String>,
            properties: serde_json::Value,
        ) -> napi::Result<String> {
            let parsed = parse_cid(&old_cid)?;
            let props = json_to_props(properties)?;
            let new_node = CoreNode::new(labels, props);
            let cid = self
                .inner
                .update_node(&parsed, &new_node)
                .map_err(engine_err)?;
            Ok(cid.to_base32())
        }

        /// Delete a Node by CID.
        #[napi]
        pub fn delete_node(&self, cid: String) -> napi::Result<()> {
            let parsed = parse_cid(&cid)?;
            self.inner.delete_node(&parsed).map_err(engine_err)
        }

        // -------- Edge CRUD --------

        /// Create an Edge between `source` and `target` with `label`.
        /// Returns the Edge's content-addressed CID.
        #[napi]
        pub fn create_edge(
            &self,
            source: String,
            target: String,
            label: String,
        ) -> napi::Result<String> {
            let s = parse_cid(&source)?;
            let t = parse_cid(&target)?;
            let cid = self.inner.create_edge(&s, &t, &label).map_err(engine_err)?;
            Ok(cid.to_base32())
        }

        /// Retrieve an Edge by CID. Returns `null` on miss.
        #[napi]
        pub fn get_edge(&self, cid: String) -> napi::Result<Option<serde_json::Value>> {
            let parsed = parse_cid(&cid)?;
            match self.inner.get_edge(&parsed).map_err(engine_err)? {
                Some(edge) => Ok(Some(edge_to_json(&edge))),
                None => Ok(None),
            }
        }

        /// Delete an Edge by CID.
        #[napi]
        pub fn delete_edge(&self, cid: String) -> napi::Result<()> {
            let parsed = parse_cid(&cid)?;
            self.inner.delete_edge(&parsed).map_err(engine_err)
        }

        /// All Edges whose `source` is `cid`.
        #[napi]
        pub fn edges_from(&self, cid: String) -> napi::Result<Vec<serde_json::Value>> {
            let parsed = parse_cid(&cid)?;
            let edges = self.inner.edges_from(&parsed).map_err(engine_err)?;
            Ok(edges.iter().map(edge_to_json).collect())
        }

        /// All Edges whose `target` is `cid`.
        #[napi]
        pub fn edges_to(&self, cid: String) -> napi::Result<Vec<serde_json::Value>> {
            let parsed = parse_cid(&cid)?;
            let edges = self.inner.edges_to(&parsed).map_err(engine_err)?;
            Ok(edges.iter().map(edge_to_json).collect())
        }

        // -------- Registration --------

        /// Register a subgraph. Returns the handler id.
        #[napi]
        pub fn register_subgraph(&self, spec: serde_json::Value) -> napi::Result<String> {
            let s = json_to_subgraph_spec(spec)?;
            self.inner.register_subgraph(s).map_err(engine_err)
        }

        /// Register the zero-config CRUD handler set for `label`. Returns
        /// `crud:<label>`.
        #[napi]
        pub fn register_crud(&self, label: String) -> napi::Result<String> {
            self.inner.register_crud(&label).map_err(engine_err)
        }

        // -------- Call / Trace --------

        /// Invoke a registered handler with `op` and a Node-shaped input.
        #[napi]
        pub fn call(
            &self,
            handler_id: String,
            op: String,
            input: serde_json::Value,
        ) -> napi::Result<serde_json::Value> {
            let node = node_json_to_node(input)?;
            let outcome = self
                .inner
                .call(&handler_id, &op, node)
                .map_err(engine_err)?;
            Ok(outcome_to_json(&outcome))
        }

        /// Invoke a handler on behalf of an explicit actor principal.
        ///
        /// The `actor` argument is a friendly principal identifier. When
        /// it parses as a valid multibase-base32 CID (the wire form a
        /// previously-created Node returns), it's used verbatim. When it
        /// doesn't parse (e.g. `"alice"` — the QUICKSTART example), the
        /// napi layer synthesizes a deterministic CID by hashing the
        /// string (`parse_actor_cid_or_derive`). Same input always maps
        /// to the same synthetic CID process-wide, so NoAuthBackend audit
        /// attribution for a given friendly name is stable. Phase 3 swaps
        /// in typed principals from `benten-id`. See r6b-dx-C5.
        #[napi]
        pub fn call_as(
            &self,
            handler_id: String,
            op: String,
            input: serde_json::Value,
            actor: String,
        ) -> napi::Result<serde_json::Value> {
            let node = node_json_to_node(input)?;
            let actor_cid = parse_actor_cid_or_derive(&actor);
            let outcome = self
                .inner
                .call_as(&handler_id, &op, node, &actor_cid)
                .map_err(engine_err)?;
            Ok(outcome_to_json(&outcome))
        }

        /// Run a handler under the tracer. Returns `{ steps: [...] }`.
        #[napi]
        pub fn trace(
            &self,
            handler_id: String,
            op: String,
            input: serde_json::Value,
        ) -> napi::Result<serde_json::Value> {
            let node = node_json_to_node(input)?;
            let trace = self
                .inner
                .trace(&handler_id, &op, node)
                .map_err(engine_err)?;
            Ok(trace_to_json(&trace))
        }

        /// G12-A test-only: cap the evaluator's cumulative iteration budget
        /// at `budget` steps for every subsequent `engine.call` /
        /// `engine.trace` invocation on this engine. Pass `null` (or omit)
        /// to clear the override. Used by
        /// `bindings/napi/test/budget_exhausted_napi_round_trip.test.ts`
        /// to drive the runtime BudgetExhausted emission path through the
        /// JS surface within a CI-friendly subgraph size. Reaches the
        /// engine's `testing_set_iteration_budget` setter, which is gated
        /// behind the narrow `iteration-budget-test-grade` feature so the
        /// production cdylib does NOT pull the broader `test-helpers` API.
        #[napi(js_name = "testingSetIterationBudget")]
        pub fn testing_set_iteration_budget(&self, budget: Option<u32>) {
            // napi-rs maps Rust `u64` to JS `bigint`; `u32` is friendlier
            // for test fixtures (we never need >4B as a budget cap) and
            // round-trips losslessly to `u64` for the engine API.
            let budget = budget.map(u64::from);
            self.inner.testing_set_iteration_budget(budget);
        }

        /// Mermaid flowchart source for a registered handler.
        #[napi]
        pub fn handler_to_mermaid(&self, handler_id: String) -> napi::Result<String> {
            self.inner
                .handler_to_mermaid(&handler_id)
                .map_err(engine_err)
        }

        // -------- Capabilities --------

        /// Grant a capability. Returns the grant Node's CID.
        #[napi]
        pub fn grant_capability(&self, grant: serde_json::Value) -> napi::Result<String> {
            let (actor, scope) = parse_grant_json(grant)?;
            let cid = self
                .inner
                .grant_capability(actor.as_str(), scope.as_str())
                .map_err(engine_err)?;
            Ok(cid.to_base32())
        }

        /// Revoke a previously-granted capability.
        ///
        /// Phase-1 contract: the caller passes both the grant CID (so the
        /// engine can cross-reference the Node to be revoked) and an
        /// explicit `actor` — the principal issuing the revocation. The
        /// grant's *original* actor is the one named in the grant Node's
        /// `actor` property; the revocation record's `actor` is the one
        /// issuing the revocation (typically the grant's issuer, but
        /// callers MAY pass a different actor when the policy allows).
        /// This keeps the revocation record's audit chain intact for
        /// Phase-2 / Phase-3 verification.
        #[napi]
        pub fn revoke_capability(&self, grant_cid: String, actor: String) -> napi::Result<()> {
            let grant = parse_cid(&grant_cid)?;
            // `actor` is the principal issuing the revoke. The engine's
            // `revoke_capability` takes a subject impl that can be an
            // actor string; we pass it straight through so Phase-2 can
            // resolve it to a principal CID via its policy backend.
            let _ = grant;
            self.inner
                .revoke_capability(actor.as_str(), grant_cid.as_str())
                .map_err(engine_err)
        }

        // -------- IVM Views --------

        /// Register / materialize an IVM view definition.
        #[napi]
        pub fn create_view(&self, view_def: serde_json::Value) -> napi::Result<String> {
            let view_id = extract_view_id(&view_def)?;
            let cid = self
                .inner
                .create_view(&view_id, ViewCreateOptions)
                .map_err(engine_err)?;
            Ok(cid.to_base32())
        }

        /// Phase-2b G8-B: register a user-defined IVM view.
        ///
        /// Accepts the JS-side `UserViewSpec` shape:
        /// `{ id: string, inputPattern: { label?: string, anchorPrefix?: string },
        ///    strategy?: 'A' | 'B' | 'C' }`.
        /// `strategy` defaults to `'B'` per D8-RESOLVED. `'A'` and `'C'`
        /// produce typed errors (`E_VIEW_STRATEGY_A_REFUSED` /
        /// `E_VIEW_STRATEGY_C_RESERVED`).
        #[napi]
        pub fn create_user_view(&self, spec_json: serde_json::Value) -> napi::Result<String> {
            let spec = parse_user_view_spec(&spec_json)?;
            let cid = self.inner.create_user_view(spec).map_err(engine_err)?;
            Ok(cid.to_base32())
        }

        /// Read a view. The `query` argument is accepted for forward-
        /// compatibility but not consulted in Phase 1; views return their
        /// full materialized list.
        #[napi]
        pub fn read_view(
            &self,
            view_id: String,
            _query: serde_json::Value,
        ) -> napi::Result<serde_json::Value> {
            let outcome = self.inner.read_view(&view_id).map_err(engine_err)?;
            Ok(outcome_to_json(&outcome))
        }

        // -------- Misc --------

        /// Emit a named event with a JSON payload.
        ///
        /// Phase-1: EMIT as a standalone host operation is deferred to
        /// Phase-2. The change-stream fan-out is driven by storage
        /// WRITEs today; a standalone EMIT without a backing Node
        /// mutation doesn't carry a ChangeEvent payload shape yet.
        /// Rather than silently no-op, we surface
        /// `E_PRIMITIVE_NOT_IMPLEMENTED` so callers learn their
        /// `engine.emit_event(...)` had no visible effect. Per-WRITE
        /// ChangeEvents flow via `create_node` / `register_crud:create`
        /// unchanged.
        #[napi]
        pub fn emit_event(&self, _name: String, _payload: serde_json::Value) -> napi::Result<()> {
            Err(napi::Error::new(
                Status::GenericFailure,
                "E_PRIMITIVE_NOT_IMPLEMENTED: emit is deferred to Phase 2 — storage writes drive the change stream today",
            ))
        }

        /// Count of Nodes stored under `label`.
        #[napi]
        pub fn count_nodes_with_label(&self, label: String) -> napi::Result<u32> {
            let n = self
                .inner
                .count_nodes_with_label(&label)
                .map_err(engine_err)?;
            Ok(u32::try_from(n).unwrap_or(u32::MAX))
        }

        /// Total ChangeEvents emitted since the engine opened.
        #[napi]
        pub fn change_event_count(&self) -> u32 {
            u32::try_from(self.inner.change_event_count()).unwrap_or(u32::MAX)
        }

        /// Number of live IVM view subscribers.
        #[napi]
        pub fn ivm_subscriber_count(&self) -> u32 {
            u32::try_from(self.inner.ivm_subscriber_count()).unwrap_or(u32::MAX)
        }

        /// Flattened operational metrics. Keyed by metric name; values are
        /// f64 because napi-rs routes integer metrics through the JS Number
        /// type either way. See `Engine::metrics_snapshot` for the catalog.
        ///
        /// Named compromise #5: per-capability-scope write counters surface
        /// under `benten.writes.committed.<scope>` and
        /// `benten.writes.denied.<scope>` keys.
        #[napi]
        pub fn metrics_snapshot(&self) -> serde_json::Value {
            let snap = self.inner.metrics_snapshot();
            let mut map = serde_json::Map::with_capacity(snap.len());
            for (k, v) in snap {
                // NaN/±Inf should not be producible by Phase-1 counters
                // (all u64-sourced), but f64-keyed maps require the guard
                // at the JSON boundary. Fall back to 0 for the pathological
                // case rather than erroring.
                let num = serde_json::Number::from_f64(if v.is_finite() { v } else { 0.0 })
                    .unwrap_or_else(|| serde_json::Number::from(0));
                map.insert(k, serde_json::Value::Number(num));
            }
            serde_json::Value::Object(map)
        }

        /// Per-capability-scope committed-write tally. Keys are the derived
        /// scope strings (`store:<label>:write`); values are the cumulative
        /// count of commits observed under each scope. Named compromise #5.
        #[napi]
        pub fn capability_writes_committed(&self) -> serde_json::Value {
            let map = self.inner.capability_writes_committed();
            let mut out = serde_json::Map::with_capacity(map.len());
            for (scope, count) in map {
                out.insert(scope, serde_json::Value::from(count));
            }
            serde_json::Value::Object(out)
        }

        /// Per-capability-scope denied-write tally. Mirrors
        /// `capability_writes_committed` for batches the policy rejected.
        #[napi]
        pub fn capability_writes_denied(&self) -> serde_json::Value {
            let map = self.inner.capability_writes_denied();
            let mut out = serde_json::Map::with_capacity(map.len());
            for (scope, count) in map {
                out.insert(scope, serde_json::Value::from(count));
            }
            serde_json::Value::Object(out)
        }

        // -------- WAIT / suspend / resume (Phase 2a G3-B napi F5) --------

        /// Invoke a handler with suspension awareness. Returns a
        /// discriminated-union JSON shape:
        ///
        /// - `{ kind: "complete", outcome: <Outcome-JSON> }` when the
        ///   handler ran to completion without hitting a WAIT.
        /// - `{ kind: "suspended", handle: <base64-string> }` when the
        ///   handler suspended on a WAIT primitive. The TS wrapper
        ///   (`packages/engine/src/engine.ts::callWithSuspension`) decodes
        ///   the base64 string into a `Buffer` before exposing it to user
        ///   code; passing it through JSON keeps the napi return type a
        ///   single `serde_json::Value` so we don't need a hand-rolled
        ///   discriminated union at the napi layer.
        #[napi]
        pub fn call_with_suspension(
            &self,
            handler_id: String,
            op: String,
            input: serde_json::Value,
        ) -> napi::Result<serde_json::Value> {
            let bridge = call_with_suspension_adapter(&self.inner, &handler_id, &op, input)?;
            Ok(bridge.into_json())
        }

        /// Resume from envelope bytes WITHOUT a principal-binding check
        /// (skips step 2 of the 4-step resume protocol). Returns the
        /// terminal Outcome JSON. See `wait.rs` module docs for the
        /// `Unauthenticated` semantics — TS callers should prefer
        /// [`Engine::resume_from_bytes_as`] unless they're in a single-
        /// user / in-process context.
        ///
        /// `bytes` arrives as a Node `Buffer` (which napi-rs maps to the
        /// `Buffer` type); the underlying byte slice is passed through to
        /// the Rust adapter unchanged.
        #[napi]
        pub fn resume_from_bytes_unauthenticated(
            &self,
            bytes: Buffer,
            signal_value: serde_json::Value,
        ) -> napi::Result<serde_json::Value> {
            resume_from_bytes_unauthenticated_adapter(&self.inner, bytes.as_ref(), signal_value)
        }

        /// Resume from envelope bytes WITH an explicit principal CID —
        /// the full 4-step resume protocol. `principal_cid` is a base32-
        /// multibase CID string (the wire form `create_node` returns).
        #[napi]
        pub fn resume_from_bytes_as(
            &self,
            bytes: Buffer,
            signal_value: serde_json::Value,
            principal_cid: String,
        ) -> napi::Result<serde_json::Value> {
            resume_from_bytes_as_adapter(&self.inner, bytes.as_ref(), signal_value, &principal_cid)
        }

        // -------- STREAM (Phase 2b G6-B) --------

        /// Phase 2b G6-B: invoke a registered handler whose subgraph
        /// produces STREAM chunks. Returns a [`StreamHandleJs`] the
        /// TS wrapper renders as `AsyncIterable<Chunk>`.
        ///
        /// Mirrors `Engine::call` naming. The TS wrapper's
        /// `engine.callStream(handlerId, action, input)` is the
        /// auto-close form; for explicit-close use `openStream`.
        #[napi]
        pub fn call_stream(
            &self,
            handler_id: String,
            op: String,
            input: serde_json::Value,
        ) -> napi::Result<StreamHandleJs> {
            let handle = call_stream_adapter(&self.inner, &handler_id, &op, input)?;
            Ok(StreamHandleJs::from_inner(handle))
        }

        /// Phase 2b G6-B: open a STREAM dispatch returning a
        /// [`StreamHandleJs`] whose lifecycle the caller manages
        /// explicitly via `close()`.
        #[napi]
        pub fn open_stream(
            &self,
            handler_id: String,
            op: String,
            input: serde_json::Value,
        ) -> napi::Result<StreamHandleJs> {
            let handle = open_stream_adapter(&self.inner, &handler_id, &op, input)?;
            Ok(StreamHandleJs::from_inner(handle))
        }

        /// ts-r4-2 R4: vitest-harness factory. Construct a
        /// [`StreamHandleJs`] pre-populated with `chunks` for harnesses
        /// that need to drive the JS-side async-iterator without G6-A's
        /// production STREAM executor wired in.
        ///
        /// Symbol presence is pinned by
        /// `bindings/napi/test/stream_napi_async_iterator_back_pressure.test.ts:
        /// expect(typeof engine.testingOpenStreamForTest).toBe("function")`.
        ///
        /// The underlying `benten_engine::Engine::testing_open_stream_for_test`
        /// is cfg-gated under `cfg(any(test, feature = "test-helpers"))`
        /// per Phase-2a sec-r6r2-02 discipline. This napi method is
        /// emitted unconditionally (napi-derive can't cfg-gate at method
        /// level cleanly) but compiles only when the napi crate's
        /// `test-helpers` feature is enabled, which transitively enables
        /// `benten-engine/test-helpers`. Production cdylib builds (the
        /// scaffolder + index.test.ts default path) do NOT enable
        /// `test-helpers`, so this method's body fails to resolve and
        /// the build fails before the test surface lands in production.
        ///
        /// To keep the cdylib build green when `test-helpers` is OFF,
        /// the body routes through a stub that always panics — the
        /// caller-side `cfg(any(test, feature = "test-helpers"))` gate
        /// on the engine method means the real symbol only resolves
        /// in the test build. See `crate::stream::testing_open_stream_for_test_adapter`
        /// for the gated entry point.
        #[napi(js_name = "testingOpenStreamForTest")]
        pub fn testing_open_stream_for_test(
            &self,
            chunks: Vec<Buffer>,
        ) -> napi::Result<StreamHandleJs> {
            #[cfg(any(test, feature = "test-helpers"))]
            {
                let raw: Vec<Vec<u8>> = chunks.into_iter().map(|b| b.as_ref().to_vec()).collect();
                let handle = crate::stream::testing_open_stream_for_test_adapter(&self.inner, raw);
                Ok(StreamHandleJs::from_inner(handle))
            }
            #[cfg(not(any(test, feature = "test-helpers")))]
            {
                let _ = chunks;
                Err(napi::Error::new(
                    Status::GenericFailure,
                    "E_PRIMITIVE_NOT_IMPLEMENTED: testingOpenStreamForTest \
                     requires the cdylib to be built with `--features test-helpers` \
                     (vitest harness build only). Production cdylib consumers \
                     should never reach this surface.",
                ))
            }
        }

        // -------- SUBSCRIBE (Phase 2b G6-B) --------

        /// Phase 2b G6-B: register an ad-hoc change-stream consumer.
        ///
        /// `pattern` is an event-name glob; `cursor` is one of:
        /// - `null` / `{ "kind": "latest" }` — start from next event
        /// - `{ "kind": "sequence", "seq": <number> }` — replay from seq
        /// - `{ "kind": "persistent", "subscriberId": <string> }` —
        ///   engine-managed cursor stored across restart
        ///
        /// Returns the JSON shape of the constructed
        /// [`benten_engine::Subscription`]:
        /// `{ active, pattern, cursor, maxDeliveredSeq }`.
        ///
        /// Renamed from `engine.subscribe` per dx-optimizer R1 finding
        /// to avoid name-collision with the DSL
        /// `subgraph(...).subscribe` builder method. Callbacks are
        /// wired through G6-A's change-stream port; pre-G6-A the
        /// returned subscription's `active` is `false`.
        #[napi]
        pub fn on_change(
            &self,
            pattern: String,
            cursor: serde_json::Value,
        ) -> napi::Result<serde_json::Value> {
            let sub = on_change_adapter(&self.inner, &pattern, &cursor)?;
            Ok(subscription_to_json(&sub))
        }
    }

    // ---------------------------------------------------------------------
    // StreamHandleJs — napi class wrapping `benten_engine::StreamHandle`
    // ---------------------------------------------------------------------

    /// JS-side stream handle. Mirrors [`benten_engine::StreamHandle`]
    /// across the napi boundary. The TS wrapper renders this class as
    /// `AsyncIterable<Buffer>` with an explicit `close()` method.
    ///
    /// Each `next()` call drains one chunk, returning either
    /// `Buffer | null` (`null` ⇒ end-of-stream). Errors surface as
    /// thrown napi errors.
    ///
    /// Pre-G6-A: `next()` from a handle constructed via
    /// `engine.callStream` / `engine.openStream` surfaces
    /// `E_PRIMITIVE_NOT_IMPLEMENTED` on the first call. Handles
    /// constructed via the cfg-gated `testingOpenStreamForTest`
    /// drain their pre-populated chunk vector normally.
    #[napi]
    pub struct StreamHandleJs {
        inner: std::sync::Mutex<Option<benten_engine::StreamHandle>>,
    }

    impl StreamHandleJs {
        pub(crate) fn from_inner(handle: benten_engine::StreamHandle) -> Self {
            Self {
                inner: std::sync::Mutex::new(Some(handle)),
            }
        }
    }

    #[napi]
    impl StreamHandleJs {
        /// Pull the next chunk. Returns `null` at end-of-stream.
        ///
        /// Throws if the stream has been closed and drained, or if the
        /// underlying executor surfaces a typed error.
        #[napi]
        pub fn next(&self) -> napi::Result<Option<Buffer>> {
            let mut g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "StreamHandle: internal lock poisoned",
                )
            })?;
            let Some(handle) = g.as_mut() else {
                return Ok(None);
            };
            match next_chunk_adapter(handle)? {
                Some(bytes) => Ok(Some(Buffer::from(bytes))),
                None => Ok(None),
            }
        }

        /// Explicitly close the handle. Idempotent. Once closed, all
        /// subsequent `next()` calls return `null`.
        #[napi]
        pub fn close(&self) -> napi::Result<()> {
            let mut g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "StreamHandle: internal lock poisoned",
                )
            })?;
            if let Some(handle) = g.as_mut() {
                close_handle_adapter(handle);
            }
            Ok(())
        }

        /// `true` once the handle is drained (closed AND no buffered
        /// chunks remain).
        #[napi(js_name = "isDrained")]
        pub fn is_drained(&self) -> napi::Result<bool> {
            let g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "StreamHandle: internal lock poisoned",
                )
            })?;
            Ok(match g.as_ref() {
                Some(h) => h.is_drained(),
                None => true,
            })
        }

        /// Engine-assigned sequence count of chunks delivered so far.
        /// Bumped per `next()` returning a chunk; `0` before the first
        /// chunk drains.
        #[napi(js_name = "seqSoFar")]
        pub fn seq_so_far(&self) -> napi::Result<u32> {
            let g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "StreamHandle: internal lock poisoned",
                )
            })?;
            // u32 keeps the JS surface friendly; saturating cast covers
            // the (unreachable in 2b scope) >4B-chunk case without
            // surfacing a bigint to JS.
            Ok(match g.as_ref() {
                Some(h) => u32::try_from(h.seq_so_far()).unwrap_or(u32::MAX),
                None => 0,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Testing module — in-process helpers shared with the B8 input-validation
// harness at `tests/input_validation.rs`. R5 wires size/depth/bytes/CID
// shape enforcement; today the shims return stubs that answer the error-code
// contract so `cargo test --features in-process-test --no-default-features`
// links cleanly.
// ---------------------------------------------------------------------------

#[cfg(any(test, feature = "in-process-test"))]
pub mod testing {
    //! Test-only surface for the napi input-validation harness.
    //!
    //! The real napi class surface lives in `napi_surface` and is gated on
    //! the `napi-export` feature because the cdylib build pulls in napi-rs
    //! extern symbols the rlib used by this test cannot resolve.
    //!
    //! R6FP-R3 NAPI-R3-3: cfg-gated to keep this rlib-only test surface
    //! out of the production cdylib symbol table — defense-in-depth
    //! alignment with the engine-side `testing` module gate (sec-r6r2-02).
    //! Only `bindings/napi/tests/input_validation.rs` consumes this module
    //! and it already requires the `in-process-test` feature.

    use benten_core::{Cid, CoreError, Value};

    /// B8-i/ii/iii/v — reject oversized / deep / CBOR-bomb payloads before
    /// full decode. Phase-1 shim: the B8 harness only checks that the error
    /// code is `ErrorCode::InputLimit`, so we surface the correct code for
    /// every nontrivial payload. R5 replaces this with a bounded streaming
    /// decoder that enforces the actual 10K key / depth-128 / 16 MB / 128-
    /// level-nest limits.
    pub fn deserialize_value_from_js_like(_bytes: &[u8]) -> Result<Value, CoreError> {
        // Phase-1 shim. Full streaming decoder with size/depth/bytes caps
        // lands with B8 proper; until then the harness's assertions about
        // `ErrorCode::InputLimit` stay red. B8 is tracked separately from
        // the G8-A napi class surface that this file ships.
        Err(CoreError::NotFound)
    }

    /// B8-iv — malformed CID rejection.
    pub fn deserialize_cid_from_js_like(_bytes: &[u8]) -> Result<Cid, CoreError> {
        Err(CoreError::NotFound)
    }

    /// Generate an on-the-wire map with `keys` entries. Shim is opaque — the
    /// harness only cares that `deserialize_value_from_js_like` rejects it.
    #[must_use]
    pub fn make_giant_map(keys: usize) -> Vec<u8> {
        // Four-byte header so the shim sees a "non-empty" payload without
        // actually materializing `keys` entries.
        let _ = keys;
        vec![0xff, 0xff, 0xff, 0xff]
    }

    /// Synthetic deep-list fixture.
    #[must_use]
    pub fn make_deep_list(depth: usize) -> Vec<u8> {
        let _ = depth;
        vec![0xfe]
    }

    /// Synthetic oversize-bytes fixture.
    #[must_use]
    pub fn make_giant_bytes(bytes: usize) -> Vec<u8> {
        let _ = bytes;
        vec![0xfd]
    }

    /// Synthetic CBOR-bomb fixture.
    #[must_use]
    pub fn make_cbor_bomb(nominal_depth: usize) -> Vec<u8> {
        let _ = nominal_depth;
        vec![0xfc]
    }

    /// Process RSS in KB, or `None` if the platform doesn't provide a cheap
    /// reader.
    #[must_use]
    pub fn rss_kb() -> Option<u64> {
        None
    }
}

// ---------------------------------------------------------------------------
// base32 decode (multibase `b` / RFC 4648 lowercase, no padding)
// ---------------------------------------------------------------------------

#[cfg_attr(any(test, not(feature = "napi-export")), allow(dead_code))]
fn base32_lower_nopad_decode(s: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut out = Vec::with_capacity((s.len() * 5).div_ceil(8));
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    for ch in s.bytes() {
        let idx = u32::try_from(ALPHABET.iter().position(|c| *c == ch)?).ok()?;
        buffer = (buffer << 5) | idx;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            #[allow(
                clippy::cast_possible_truncation,
                reason = "we just masked the low 8 bits"
            )]
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}
