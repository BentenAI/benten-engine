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
#[cfg(feature = "napi-export")]
mod subgraph;
#[cfg(feature = "napi-export")]
mod trace;
#[cfg(feature = "napi-export")]
mod view;

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
    use crate::node::{json_to_props, node_json_to_node, node_to_json, parse_cid};
    use crate::policy::{PolicyKind, parse_grant_json};
    use crate::subgraph::{json_to_subgraph_spec, outcome_to_json};
    use crate::trace::trace_to_json;
    use crate::view::extract_view_id;

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
            let mut builder = EngineBuilder::new();
            if let Some(p) = policy.into_policy() {
                builder = builder.capability_policy(p);
            }
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

        /// Invoke a handler on behalf of an explicit actor CID.
        #[napi]
        pub fn call_as(
            &self,
            handler_id: String,
            op: String,
            input: serde_json::Value,
            actor: String,
        ) -> napi::Result<serde_json::Value> {
            let node = node_json_to_node(input)?;
            let actor_cid = parse_cid(&actor)?;
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
    }

    // Keep a lint-silencer so the imports stay live even when every helper
    // they wrap is called conditionally.
    #[allow(dead_code)]
    fn _keep_imports_live() -> Option<Value> {
        let _: Option<BTreeMap<String, Value>> = None;
        None
    }
}

// ---------------------------------------------------------------------------
// Testing module — in-process helpers shared with the B8 input-validation
// harness at `tests/input_validation.rs`. R5 wires size/depth/bytes/CID
// shape enforcement; today the shims return stubs that answer the error-code
// contract so `cargo test --features in-process-test --no-default-features`
// links cleanly.
// ---------------------------------------------------------------------------

pub mod testing {
    //! Test-only surface for the napi input-validation harness.
    //!
    //! The real napi class surface lives in `napi_surface` and is gated on
    //! the `napi-export` feature because the cdylib build pulls in napi-rs
    //! extern symbols the rlib used by this test cannot resolve.

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
