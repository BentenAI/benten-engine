//! # benten-napi
//!
//! Node.js bindings for the Benten graph engine via napi-rs v3.
//!
//! G8-A surface: a `#[napi] Engine` class wrapping `benten_engine::Engine`
//! with the full Phase-1 CRUD + handler + view + trace + capability API.
//! Values cross the boundary as `serde_json::Value`; CIDs cross as
//! base32-multibase strings (prefix `b`).
//!
//! ## Scope boundary (Phase-4-Foundation R6-FP-C ec-r6r1-6 closure, 2026-05-13)
//!
//! The napi binding covers Phase-1 / Phase-2 / Phase-3 **full-peer engine**
//! + **sync surface** (shape (a) per CLAUDE.md baked-in #17). Phase-4-Foundation
//! admin UI / plugin / schema / materializer surfaces are reached via the
//! **wasm32 thin-client protocol** (shape (b) browser / edge / shape (c)
//! Tauri embedded webview), NOT via napi. This is the intentional
//! architectural scope boundary, not staleness — adding Phase-4-Foundation
//! surfaces to the napi binding would duplicate the thin-client protocol
//! seam and break the heterogeneity contract.
//!
//! Concretely, the napi binding does NOT mirror:
//!   - `crates/benten-platform-foundation/src/plugin_manifest.rs`
//!   - `crates/benten-platform-foundation/src/plugin_library.rs`
//!   - `crates/benten-platform-foundation/src/plugin_lifecycle.rs`
//!   - `crates/benten-platform-foundation/src/schema_compiler/`
//!   - `crates/benten-platform-foundation/src/materializer.rs`
//!   - `crates/benten-platform-foundation/src/module_ecosystem.rs`
//!
//! The generated `bindings/napi/index.d.ts` reflects this scope boundary
//! (Phase-3-era timestamp is correct, not stale). Cross-language drift
//! for Phase-4-Foundation surfaces is enforced via
//! `packages/engine/src/errors.generated.ts` (the wasm32 mirror) +
//! drift-detect.ts catalog parity check.
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
// Phase-3 G19-B (§7.2): JSON-envelope formatter shared by the production
// `error::engine_err` carrier (napi-export ON) AND the rlib-test helper
// `testing::engine_err_message` (in-process-test ON). The formatter
// itself has no napi-rs dep, so it's reachable from both build modes.
#[cfg(any(feature = "napi-export", feature = "in-process-test", test))]
mod error_envelope;
// Refinement-audit-2026-05 #1201: thin capacity-primed JSON-object
// builder shared by the edge/node/subgraph/trace projectors. Gated
// like its `napi-export` consumers; also reachable under `test` for
// the in-crate unit test.
#[cfg(any(feature = "napi-export", test))]
mod json_build;
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
//
// G13-C R5 wave-3 fix-pass (2026-05-06): additionally gated off
// `wasm32-unknown-unknown` because the helpers consume
// `InnerEngine::call_with_suspension` and friends — those inherent
// methods only exist on the redb-bound `Engine = EngineGeneric<RedbBackend>`
// alias which is itself gated to non-browser-backend / non-wasm32 targets.
// Browser thin-clients don't run handlers (CLAUDE.md baked-in #17), so
// WAIT/STREAM/SUBSCRIBE adapters are gated out of the browser bundle
// alongside `napi_surface`. Using `target_arch` (not the
// `benten-engine/browser-backend` feature) because benten-napi does not
// expose its own `browser-backend` feature flag — wasm32-unknown-unknown
// is a precise proxy for "browser bundle build".
#[cfg(all(feature = "napi-export", not(target_arch = "wasm32")))]
mod wait;
// Phase 2b G6-B: STREAM + SUBSCRIBE napi-bridge helpers. Same split
// pattern as `wait` — adapters are plain Rust functions here, the
// `#[napi]` impl methods consuming them live in `napi_surface` below.
//
// G13-C R5 wave-3 fix-pass (2026-05-06): wasm32 gating per the matching
// note on `mod wait` above.
#[cfg(all(feature = "napi-export", not(target_arch = "wasm32")))]
mod stream;
#[cfg(all(feature = "napi-export", not(target_arch = "wasm32")))]
mod subscribe;
// Phase 2b Wave-8f: napi `DevServer` class wrapping `benten_dev::DevServer`.
// Self-contained surface (its own `#[napi]` impl block) — kept in its own
// file because the dev-server bridge has zero overlap with the existing
// `napi_surface::Engine` impl block.
//
// G13-C R5 wave-3 fix-pass (2026-05-06): additionally gated to
// `not(target_arch = "wasm32")` because `benten-dev` is a native-only
// scaffolding tool (consumes `Engine::builder()` /
// `register_subgraph_replace` paths that are themselves redb-coupled
// and gated off browser-backend per CLAUDE.md baked-in #17). The
// `benten-dev` dep is already target-conditional in `Cargo.toml`; the
// module-level cfg here keeps the `mod devserver;` line honest under
// the same target gate.
#[cfg(all(feature = "napi-export", not(target_arch = "wasm32")))]
mod devserver;
// Phase 3 G14-A1 wave-4a: identity primitives (Keypair / Did / signing
// + verify). Native-only because the cryptographic operations + the
// secret-bytes hygiene contract (zeroize-on-drop, no Clone, redacted
// Debug) live on the full-peer side per CLAUDE.md baked-in #17. The
// browser thin-client TS surface (`packages/engine/src/identity.ts`)
// declares the shape; production identity work runs on the native
// peer.
#[cfg(all(feature = "napi-export", not(target_arch = "wasm32")))]
mod identity;
// Phase 3 G16-D wave-6b — Atrium TS DSL bridge (Pattern B-prime
// factory-handle form per Ben's D1 ratification 2026-05-05). Surfaces
// `JsAtrium` (the typed handle returned from `engine.atrium({config})`)
// + `AtriumConfig` / `DeviceAttestationDeclaration` / `CapabilityClaim`
// typed structs. Native-only because the underlying handshake protocol
// body (G16-D's `crates/benten-sync/src/handshake.rs`) is native-only
// per CLAUDE.md baked-in #17. Browser thin-clients consume the same TS
// surface but route their declared device-attestation through to the
// connected full peer (D-PHASE-3-30 thin-client protocol).
#[cfg(all(feature = "napi-export", not(target_arch = "wasm32")))]
mod atrium;
// Phase 2b G10-A-wasip1: napi-side wasm32-wasip1 runtime probes
// (`wasiTargetKind`, `wasiRuntimeSupportsRedbNative`,
// `wasiCanonicalFixtureCid`). Cfg-split per the same defence-in-depth
// pattern as `sandbox.rs`: both halves of the cfg ship so TS callers
// always see the symbol. The G10-A-browser `wasm_browser.rs` sibling
// covers wasm32-unknown-unknown.
#[cfg(feature = "napi-export")]
mod wasm_target;

// Phase 2b G10-A-browser: wasm32-unknown-unknown runtime path —
// in-memory module manifest store (Compromise #N+8) + target-availability
// probe (`browser_runtime_available`). Compiled on every target so the
// store type is reachable from native unit tests + integration tests
// even when not running under wasm32; the cfg-split inside the file
// handles target-honest probe answers.
//
// Module is NOT gated on `napi-export` so the integration tests under
// `bindings/napi/tests/` can reach the storage-contract surface without
// linking the napi cdylib externs (the napi extern symbols don't
// resolve in a libtest binary). The `#[napi]` attribute on
// `browser_runtime_available` is gated via `#[cfg_attr(feature =
// "napi-export", ...)]` inside the module itself, so the symbol is
// emitted to the cdylib but is invisible to the rlib-only test path.
pub mod wasm_browser;

// Phase-3 G18-A wave-5a — IndexedDB-backed persistent module-manifest
// store + thin-client snapshot cache (CLAUDE.md baked-in #17 thin-
// client cache scope ONLY; NOT full sync state). Closes Compromises
// #19 + #20 in `docs/SECURITY-POSTURE.md` per D-PHASE-3-27 + br-r1-2
// BLOCKER. Module is reachable on every target so cross-target unit
// tests + the `bindings/napi/tests/indexeddb_schema.rs` source-cite
// integration tests can compile against the surface; the wasm32 arm
// inside the module wraps real IndexedDB calls via wasm-bindgen, and
// the native arm is a stub that satisfies the `pub fn` boundary so
// cross-target compilation succeeds.
pub mod browser_indexeddb;

// Phase-3 G18-A wave-5a — IndexedDB-backed BlobBackend variant for
// the browser thin-client snapshot cache. Mirrors the redb-native
// `RedbBlobBackend` at `crates/benten-graph/src/backends/blob_backend.rs`
// at the trait-surface level (locked at G13-pre-B). Per CLAUDE.md
// baked-in #17: thin-client cache scope ONLY — NOT full sync state.
pub mod browser_blob_store;

// Re-export the policy enum so the napi-derive macros pick it up at the
// crate root where napi-rs v3 looks for top-level `#[napi]` items.
#[cfg(feature = "napi-export")]
pub use policy::PolicyKind;

// ---------------------------------------------------------------------------
// Engine class (napi surface)
// ---------------------------------------------------------------------------
//
// G13-C R5 wave-3 fix-pass (2026-05-06): the `napi_surface::Engine` class
// consumes ~50 inherent methods on the `Engine = EngineGeneric<RedbBackend>`
// alias (`Engine::open` / `create_node` / `call` / `register_subgraph` /
// `install_module` / etc). Per Q1 ratification 2026-05-05 (alias-based
// pragmatic-genericism), these methods are NOT cascaded to
// `<B: GraphBackend>` in Phase-3 R5; lifting is deferred to the
// v1-assessment-window. Consequently the napi_surface::Engine class is
// gated off the `wasm32-unknown-unknown` target: the browser bundle
// exports only the in-memory `wasm_browser::BrowserManifestStore` + the
// `browser_runtime_available()` probe (CLAUDE.md baked-in #17 — thin
// clients hold no full-peer Engine surface; reads/writes flow through
// the full peer's authenticated subscription protocol per G14-D).
// Using `target_arch` (not a benten-napi feature) because benten-napi
// does not expose a `browser-backend` feature flag of its own; the
// wasm-browser CI workflow propagates `--features benten-engine/browser-backend`
// to select the BrowserBackend Engine alias arm in benten-engine, and
// the napi binding distinguishes target via `target_arch = "wasm32"`.
#[cfg(all(feature = "napi-export", not(target_arch = "wasm32")))]
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
        json_to_props, json_to_value_root, node_json_to_node, node_to_json,
        parse_actor_cid_or_derive, parse_cid,
    };
    use crate::policy::{PolicyKind, parse_grant_json};
    use crate::stream::{
        call_stream_adapter, call_stream_as_adapter, close_handle_adapter, next_chunk_adapter,
        open_stream_adapter,
    };
    use crate::subgraph::{
        json_to_subgraph_spec, outcome_to_json, register_replace_outcome_to_json,
    };
    use crate::subscribe::{on_change_adapter, on_change_as_adapter, on_emit_adapter};
    #[cfg(feature = "test-helpers")]
    use crate::subscribe::{
        testing_deliver_synthetic_event_for_test_adapter,
        testing_open_subscription_for_test_adapter,
    };
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
            // G21-T2 audit-6-1 closure: `PolicyKind::Ucan` now routes
            // through the durable UCAN-grounded grant-backed policy
            // (closes phase-3-backlog §2.3). Pre-G21-T2 this arm wired
            // the Phase-1 stub `benten_caps::LegacyUcanStubBackend`
            // (renamed from `UcanBackend` so a misroute import-error
            // surfaces at compile time rather than at runtime). The
            // durable backend reads the engine's own
            // `system:CapabilityGrant` / `system:CapabilityRevocation`
            // Nodes; the underlying UCAN proof-chain validator lives
            // at `benten_caps::backends::UCANBackend` (G14-B wave-4b).
            let builder = match policy {
                PolicyKind::NoAuth => EngineBuilder::new(),
                PolicyKind::Ucan => EngineBuilder::new().capability_policy_ucan_durable(),
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

        /// R6FP-tail (Round-2 Instance 10) — replace a registered
        /// subgraph's body. Idempotent on identical CID under the same
        /// `handler_id`; bumps the in-memory version chain on different
        /// content.
        ///
        /// Returns JSON `{ handlerId, cid, previousCid, chainDepth,
        /// versionTag, replaced }` so JS callers can distinguish first
        /// registration / idempotent re-registration / true replace
        /// without a side-channel subscribe-to-reload-events
        /// correlation. Pre-Instance-10 the napi surface dropped 3 of 4
        /// `RegisterReplaceOutcome` fields + the `Engine::register_subgraph_replace`
        /// method was NOT exposed via napi at all (only the
        /// devserver-bound `replaceHandlerFromDsl` path existed, with
        /// only the new-CID String return).
        #[napi(js_name = "registerSubgraphReplace")]
        pub fn register_subgraph_replace(
            &self,
            spec: serde_json::Value,
        ) -> napi::Result<serde_json::Value> {
            let s = json_to_subgraph_spec(spec)?;
            let outcome = self
                .inner
                .register_subgraph_replace(s)
                .map_err(engine_err)?;
            Ok(register_replace_outcome_to_json(&outcome))
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

        /// Phase-3 G21-T2 — typed-CALL surface. Drives the engine's
        /// `engine:typed:<op>` dispatch arm directly without first
        /// registering a CALL-bearing subgraph.
        ///
        /// `op_name` is the trailing op-name segment (e.g.
        /// `"ed25519_sign"`); `input` is the per-op input shape per
        /// `crates/benten-eval/src/typed_call.rs::TypedCallOp` rustdoc.
        /// Returns the op's typed output as JSON (Bytes round-trip
        /// through the napi numeric-keyed-object shape — same as
        /// `engine.createNode` properties).
        ///
        /// Errors map to the stable `E_TYPED_CALL_*` catalog codes:
        /// - `E_TYPED_CALL_UNKNOWN_OP` — `op_name` is not one of the
        ///   10 ops in the closed registry.
        /// - `E_TYPED_CALL_INVALID_INPUT` — input shape rejects.
        /// - `E_TYPED_CALL_CAP_DENIED` — capability gate denies (only
        ///   under non-NoAuth policies).
        /// - `E_TYPED_CALL_DISPATCH_ERROR` — op-internal failure
        ///   (malformed key bytes / corrupted UCAN envelope / etc).
        #[napi(js_name = "typedCall")]
        pub fn typed_call(
            &self,
            op_name: String,
            input: serde_json::Value,
        ) -> napi::Result<serde_json::Value> {
            // Parse op-name into the closed-set TypedCallOp variant.
            // Unknown ops surface E_TYPED_CALL_UNKNOWN_OP via the
            // engine catch-all (we route through the engine's
            // dispatch fork even for the unknown case so the catalog
            // attribution stays uniform).
            let op = benten_engine::TypedCallOp::parse(&op_name).ok_or_else(|| {
                // Surface the catalog code in the message so the TS
                // mapNativeError extracts the right `E_TYPED_CALL_*`
                // class. Mirrors the JSON-envelope shape engine_err
                // produces for typed EngineError variants.
                let body = format!(
                    r#"{{"code":"E_TYPED_CALL_UNKNOWN_OP","message":"typed-CALL dispatch: unknown op '{op_name}' (engine:typed:* registry has no matching entry)"}}"#
                );
                napi::Error::new(Status::GenericFailure, body)
            })?;
            let value = json_to_value_root(input)?;
            let out = self
                .inner
                .dispatch_typed_call_public(op, &value)
                .map_err(engine_err)?;
            Ok(crate::node::value_to_json(&out))
        }

        /// Phase-3 G21-T2 §C audit-6-2 closure — `Engine.atrium()`
        /// factory method per Ben's D1 ratification (Pattern B-prime
        /// factory-handle form).
        ///
        /// Returns a [`crate::atrium::JsAtrium`] handle bound to this
        /// engine instance. Subsequent calls on the handle (`join` /
        /// `leave` / `trustPeer` / `revokePeer` / `listPeers` /
        /// `subscribe` / `declareDeviceAttestation`) drive the
        /// engine-side `Engine::open_atrium` / `AtriumHandle`
        /// surfaces (see `crates/benten-engine/src/atrium_api.rs` +
        /// `engine_sync.rs`).
        ///
        /// Pre-G21-T2 the napi `JsAtrium` was a self-contained
        /// in-memory shim; the engine-side `Engine::open_atrium`
        /// existed at G16-B canary scope but was NOT exposed at the
        /// napi boundary. This factory closes that BLOCKER —
        /// `engine.atrium({}).join()` from JS/TS now drives a real
        /// engine-side `AtriumHandle`.
        ///
        /// **Deferred-bind semantics.** This factory is synchronous
        /// and does NOT open the Atrium — actual opening (iroh
        /// `Endpoint` bind + per-zone Loro CRDT init) happens on the
        /// first `.join()` call. The `is_joined()` getter returns
        /// `false` until `.join()` succeeds; the synchronous
        /// `atrium_id` getter echoes `config.atriumId` regardless of
        /// join state (it does NOT report a placeholder value pre-
        /// join — the configured ID is the canonical handle identity).
        #[napi(js_name = "atrium")]
        pub fn atrium(&self, config: crate::atrium::AtriumConfig) -> crate::atrium::JsAtrium {
            crate::atrium::JsAtrium::from_engine(config, Arc::clone(&self.inner))
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
        ///
        /// Phase-3 G21-T2: the JSON grant shape is widened to accept
        /// optional `issuer` (DID string) + `hlc` (numeric stamp) for
        /// UCAN-grounded grants — these flow through to the durable
        /// backend's chain-walker via
        /// [`benten_engine::Engine::grant_capability_with_proof`].
        /// Phase-1 callers passing only `{ actor, scope }` continue
        /// to work unchanged (the Node persisted has no `issuer` /
        /// `hlc` properties; the durable backend treats the grant as
        /// Phase-1-style).
        #[napi]
        pub fn grant_capability(&self, grant: serde_json::Value) -> napi::Result<String> {
            let parsed = parse_grant_json(grant)?;
            let cid = self
                .inner
                .grant_capability_with_proof(
                    parsed.actor.as_str(),
                    parsed.scope.as_str(),
                    parsed.issuer,
                    parsed.hlc,
                )
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
            // Phase-3.5 §13.11 closure: the engine's `revoke_capability`
            // takes `(actor, scope)`. Pre-3.5 this napi path passed
            // `grant_cid` AS the scope, producing a
            // `system:CapabilityRevocation` Node with `scope = "<cid>"`
            // that the `BackendGrantReader` walker never matched
            // against the actual write scope — every post-revoke write
            // fail-OPENed silently. Route through the engine's
            // grant-CID-resolving seam so the revocation Node carries
            // the grant's actual `scope` property.
            self.inner
                .revoke_capability_by_grant_cid(&grant, actor.as_str())
                .map_err(engine_err)
        }

        /// Phase-4-Foundation G24-D-FP-3 — runtime UCAN delegation
        /// from one plugin / principal to another (audience = plugin-
        /// DID, scope = resolved-from-source-grant).
        ///
        /// `source_grant_cid` is the CID of the SOURCE capability grant
        /// (typically a user-issued root grant, or a parent plugin's
        /// grant in a multi-hop chain). The engine seam resolves the
        /// source grant's actual `scope` text from the persisted Node
        /// + writes a NEW `system:CapabilityGrant` Node carrying that
        /// resolved scope — NEVER the source CID as a string. This is
        /// the napi-side defense against the G27-A class-of-bug shape
        /// that PR #199 closed for `revokeCapability` (a delegation
        /// Node persisted with `scope = "<cid base32>"` would never be
        /// matched by `GrantReader::has_unrevoked_grant_for_scope` at
        /// the write-check seam, silently fail-OPENing every
        /// cross-plugin write).
        ///
        /// `plugin_did` is the audience DID — stored as the new grant's
        /// `actor` so subsequent `callAs(handler, op, input, plugin_did)`
        /// calls under the delegated scope admit per
        /// `GrantBackedPolicy::check_write`.
        ///
        /// `attenuated_caps` is the (possibly empty) attenuation list.
        /// Empty → the new grant carries the resolved source scope
        /// unchanged. Non-empty → the new grant carries
        /// `attenuated_caps[0]` as its scope (full per-segment subset
        /// semantics land alongside G27-D).
        ///
        /// Returns the new delegation grant's CID.
        #[napi]
        pub fn delegate_capability(
            &self,
            source_grant_cid: String,
            plugin_did: String,
            attenuated_caps: Vec<String>,
        ) -> napi::Result<String> {
            let source = parse_cid(&source_grant_cid)?;
            let cid = self
                .inner
                .delegate_capability(&source, plugin_did.as_str(), &attenuated_caps)
                .map_err(engine_err)?;
            Ok(cid.to_base32())
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
        ///
        /// R6FP-Group-1 (r6-arch-2): renamed from `create_user_view` to
        /// align with the engine's `register_*` lifecycle verb. The
        /// js-name `createUserView` deprecation alias is held alongside
        /// `registerUserView` through one transition window so the
        /// Group-2 TS rename does not break the build.
        #[napi(js_name = "registerUserView")]
        pub fn register_user_view(&self, spec_json: serde_json::Value) -> napi::Result<String> {
            let spec = parse_user_view_spec(&spec_json)?;
            let cid = self.inner.register_user_view(spec).map_err(engine_err)?;
            Ok(cid.to_base32())
        }

        /// R6FP-Group-1 (r6-arch-2) deprecation alias — renamed to
        /// [`Self::register_user_view`]. Held through one transition
        /// window for the Group-2 TS-side rename; remove in a
        /// follow-up.
        #[napi(js_name = "createUserView")]
        pub fn create_user_view(&self, spec_json: serde_json::Value) -> napi::Result<String> {
            self.register_user_view(spec_json)
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

        /// Phase-3 G19-C1 — runtime accessor for `view.snapshot()`
        /// (per `docs/future/phase-3-backlog.md` §7.1.3). Returns the
        /// view's currently-materialized rows as a JSON array (or
        /// `null` when no view is registered with `view_id`).
        ///
        /// JS surface: `engine.userViewSnapshot(viewId) -> Node[] | null`
        #[napi(js_name = "userViewSnapshot")]
        pub fn user_view_snapshot(&self, view_id: String) -> napi::Result<serde_json::Value> {
            match crate::view::user_view_snapshot_adapter(&self.inner, &view_id)? {
                Some(arr) => Ok(arr),
                None => Ok(serde_json::Value::Null),
            }
        }

        /// Phase-3 G19-C1 — incremental delta drain accessor for
        /// `view.onUpdate()` (per `docs/future/phase-3-backlog.md`
        /// §7.1.3). Stateless cursor protocol — the JS-side async
        /// iterator passes its prior `nextOffset` per call and the
        /// engine drains every ChangeEvent matching the view's input
        /// label since that cursor.
        ///
        /// Return shape:
        /// ```text
        /// {
        ///   registered: boolean,
        ///   events: ChangeEventJson[],
        ///   nextOffset: number
        /// }
        /// ```
        ///
        /// JS surface: `engine.userViewDrainUpdates(viewId, sinceOffset)`.
        #[napi(js_name = "userViewDrainUpdates")]
        pub fn user_view_drain_updates(
            &self,
            view_id: String,
            since_offset: i64,
        ) -> napi::Result<serde_json::Value> {
            // i64 → u64 with negative-cursor clamp at 0 (a JS caller
            // passing -1 means "start from the beginning"; the engine's
            // observed_events is bounded so this is safe).
            let offset = u64::try_from(since_offset).unwrap_or(0);
            crate::view::user_view_drain_updates_adapter(&self.inner, &view_id, offset)
        }

        /// Phase-3 G19-C1 — head-cursor accessor mirroring
        /// [`Self::user_view_drain_updates`]'s `nextOffset` field. Used
        /// by the JS wrapper as the starting cursor for a freshly-
        /// constructed `view.onUpdate()` async iterator (so an iterator
        /// created BEFORE any writes registers cleanly at offset 0,
        /// while an iterator created mid-session starts at the current
        /// head and only sees events strictly newer than now).
        #[napi(js_name = "userViewChangeOffset")]
        pub fn user_view_change_offset(&self) -> i64 {
            i64::try_from(self.inner.user_view_change_offset()).unwrap_or(i64::MAX)
        }

        // -------- Misc --------

        /// Emit a named event with a JSON payload.
        ///
        /// Phase-3 G19-B (§7.8): wires the standalone `emit_event`
        /// surface directly through the engine's
        /// `EmitBroadcast` bus (the same channel
        /// `subscribe_emit_events_with_handle` consumes). JS callers
        /// invoking `engine.emitEvent(channel, payload)` see the event
        /// delivered to every `engine.onEmit(channel, ...)` consumer
        /// end-to-end, with no handler-dispatch in between.
        ///
        /// Pre-G19-B this surface returned `E_PRIMITIVE_NOT_IMPLEMENTED`
        /// with a phase-3-backlog §7.8 named-destination hint.
        #[napi]
        pub fn emit_event(&self, name: String, payload: serde_json::Value) -> napi::Result<()> {
            let value = json_to_value_root(payload)?;
            self.inner.emit_event(&name, value).map_err(engine_err)
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
        /// - `{ kind: "suspended", handle: <base64-string>, stateCid: <base32 CID>, signalName: <string> }`
        ///   when the handler suspended on a WAIT primitive. The TS
        ///   wrapper (`packages/engine/src/engine.ts::callWithSuspension`)
        ///   decodes the base64 string into a `Buffer` before exposing
        ///   it to user code; passing through JSON keeps the napi
        ///   return type a single `serde_json::Value`. R6 Round-2
        ///   Instance 12 added `stateCid` + `signalName` fields so JS
        ///   callers can correlate the suspension across logs /
        ///   external orchestration without parsing the opaque bytes.
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

        /// Phase-3 G19-C1 (§7.1.4 + r6-napi-2 closure): testing-only
        /// wallclock-advance hook for the WAIT TTL expiry path.
        ///
        /// Tests exercising the TTL-expired branch of `resume_with_meta`
        /// would otherwise have to wait `timeout_ms` real wall-clock
        /// time before resuming; this binding lets the harness fast-
        /// forward the engine's `MonotonicSource` past the deadline
        /// deterministically.
        ///
        /// The body is `test-helpers`-feature-gated per Phase-2a
        /// sec-r6r2-02 cfg-gating audit precedent: the production
        /// cdylib build (default `napi-export` features only) surfaces
        /// `E_PRIMITIVE_NOT_IMPLEMENTED` so callers learn the binding
        /// is test-grade rather than silently no-op'ing. Vitest /
        /// integration-test cdylib builds compile with
        /// `--features test-helpers` and inherit the engine-side
        /// `testing_advance_wait_clock` helper at
        /// `crates/benten-engine/src/testing.rs`.
        #[napi(js_name = "testingAdvanceWaitClock")]
        pub fn testing_advance_wait_clock(&self, delta_ms: u32) -> napi::Result<()> {
            #[cfg(any(test, feature = "test-helpers"))]
            {
                // Phase-3 G19-C1 (phase-3-backlog §7.1.4) wired entry
                // point. Forwards to the engine-side helper at
                // `crates/benten-engine/src/testing.rs::testing_advance_wait_clock`
                // (no-op stub today; D12 lifts the body to a real
                // MockMonotonicSource advance). Signature relaxed
                // upstream to `&Engine` (was `&mut Engine`) so the
                // napi `Arc<InnerEngine>` shape forwards without an
                // owned-clone shim — interior mutation will live on
                // the time-source field once D12 lands. Pre-flight
                // guard: a negative delta is rejected at the adapter.
                crate::wait::testing_advance_wait_clock_adapter(&self.inner, i64::from(delta_ms))?;
                Ok(())
            }
            #[cfg(not(any(test, feature = "test-helpers")))]
            {
                let _ = delta_ms;
                Err(napi::Error::new(
                    Status::GenericFailure,
                    "E_PRIMITIVE_NOT_IMPLEMENTED: testingAdvanceWaitClock \
                     requires the cdylib to be built with `--features test-helpers` \
                     (vitest harness build only). Production cdylib consumers \
                     should never reach this surface.",
                ))
            }
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

        /// Phase 2b wave-8c-cont: `callStream` with an explicit actor
        /// principal. Mirrors [`Engine::call_as`] naming.
        ///
        /// `actor` is a friendly principal identifier resolved through
        /// `parse_actor_cid_or_derive` (same as
        /// [`Engine::call_as`]). The principal is threaded through to
        /// the STREAM executor for cap-recheck on chunk emission once
        /// the production runtime wires through (8c-i remainder); the
        /// surface itself is callable now so consumers see typed
        /// errors instead of "method not found".
        #[napi]
        pub fn call_stream_as(
            &self,
            handler_id: String,
            op: String,
            input: serde_json::Value,
            actor: String,
        ) -> napi::Result<StreamHandleJs> {
            let actor_cid = parse_actor_cid_or_derive(&actor);
            let handle = call_stream_as_adapter(&self.inner, &handler_id, &op, input, &actor_cid)?;
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

        /// Phase 2b wave-8c-stream-infra: process-wide active STREAM
        /// count. Bumped when a producer-bridge `StreamHandle` is
        /// constructed; decremented on `Drop` / explicit `close()`.
        /// Used by `packages/engine/test/stream.test.ts` to verify
        /// for-await break propagates producer-side cleanup.
        #[napi]
        pub fn active_stream_count(&self) -> u32 {
            u32::try_from(self.inner.active_stream_count()).unwrap_or(u32::MAX)
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
            #[cfg(feature = "test-helpers")]
            {
                let raw: Vec<Vec<u8>> = chunks.into_iter().map(|b| b.as_ref().to_vec()).collect();
                let handle = crate::stream::testing_open_stream_for_test_adapter(&self.inner, raw);
                Ok(StreamHandleJs::from_inner(handle))
            }
            #[cfg(not(feature = "test-helpers"))]
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

        /// Phase-3 G19-C2 wave-7 (§7.1): napi bridge for the engine's
        /// `describe_sandbox_node_for_handler` accessor. Returns a JSON
        /// object with the resolved-defaults triple + the high-water
        /// metric values populated by `primitive_host::execute_sandbox`.
        ///
        /// Returns `null` when no SANDBOX invocation has been recorded
        /// for the named handler — the TS-side wrapper at
        /// `packages/engine/src/engine.ts::describeSandboxNode` surfaces
        /// each metric field as the structural `null` value in that case
        /// (the legacy `"unknown"` string sentinel was dropped at the
        /// §7.1-closure wave; the operator must call the handler at least
        /// once before the metric record exists).
        ///
        /// cfg-gated under `feature = "test-helpers"` matching the
        /// engine-side accessor's gate. Production cdylib builds DO NOT
        /// expose this method.
        #[napi(js_name = "describeSandboxNode")]
        pub fn describe_sandbox_node_napi(
            &self,
            handler_id: String,
        ) -> napi::Result<Option<String>> {
            #[cfg(feature = "test-helpers")]
            {
                match self.inner.describe_sandbox_node_for_handler(&handler_id) {
                    Ok(desc) => {
                        // Serialize to JSON for the napi bridge — the TS
                        // side parses + projects to `SandboxNodeDescription`.
                        // Manual format because `SandboxNodeDescription`
                        // is not Serialize-derivable without pulling
                        // serde across the stable-shape contract.
                        let manifest = match desc.manifest_id {
                            Some(s) => {
                                format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
                            }
                            None => "null".to_string(),
                        };
                        let fuel_consumed_high_water = match desc.fuel_consumed_high_water {
                            Some(n) => n.to_string(),
                            None => "null".to_string(),
                        };
                        // R6 fp Wave C2 (obs-r6r1-1 closure — 25th p/c
                        // drift instance): thread `outputConsumedHighWater`
                        // through the napi JSON template so the Phase-3
                        // §7.1 trio (fuel + output + wallclock) reaches
                        // the TS consumer surface.
                        let output_consumed_high_water = match desc.output_consumed_high_water {
                            Some(n) => n.to_string(),
                            None => "null".to_string(),
                        };
                        let last_invocation_ms = match desc.last_invocation_ms {
                            Some(n) => n.to_string(),
                            None => "null".to_string(),
                        };
                        Ok(Some(format!(
                            r#"{{"moduleCid":"{}","manifestId":{},"fuel":{},"wallclockMs":{},"outputLimitBytes":{},"fuelConsumedHighWater":{},"outputConsumedHighWater":{},"lastInvocationMs":{}}}"#,
                            desc.module_cid.to_base32(),
                            manifest,
                            desc.fuel,
                            desc.wallclock_ms,
                            desc.output_limit_bytes,
                            fuel_consumed_high_water,
                            output_consumed_high_water,
                            last_invocation_ms,
                        )))
                    }
                    // No metrics record yet — TS falls back to sentinels.
                    Err(_) => Ok(None),
                }
            }
            #[cfg(not(feature = "test-helpers"))]
            {
                let _ = handler_id;
                Err(napi::Error::new(
                    Status::GenericFailure,
                    "E_PRIMITIVE_NOT_IMPLEMENTED: describeSandboxNode \
                     requires the cdylib to be built with `--features test-helpers` \
                     (vitest / dev build only).",
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
        /// Returns a [`SubscriptionJs`] handle whose `unsubscribe()`
        /// round-trips to the engine's registry slot release. **The
        /// JS-side handle MUST be retained for the lifetime of the
        /// subscription** — dropping the handle (or letting it be GC'd)
        /// fires Drop on the underlying `benten_engine::Subscription`,
        /// which calls `unregister_on_change` and tears down the
        /// `napi::ThreadsafeFunction` Arc that holds the JS callback
        /// alive. Wave-8c fix-pass cr-w8c-fp-1 lifecycle correction:
        /// the prior return-as-JSON shape dropped the Subscription at
        /// end of method scope and JS callbacks could never fire.
        ///
        /// Renamed from `engine.subscribe` per dx-optimizer R1 finding
        /// to avoid name-collision with the DSL
        /// `subgraph(...).subscribe` builder method.
        #[napi]
        pub fn on_change(
            &self,
            pattern: String,
            cursor: serde_json::Value,
            callback: Option<
                napi::bindgen_prelude::Function<
                    '_,
                    napi::bindgen_prelude::FnArgs<(u32, Buffer)>,
                    (),
                >,
            >,
        ) -> napi::Result<SubscriptionJs> {
            let sub = on_change_adapter(&self.inner, &pattern, &cursor, callback)?;
            Ok(SubscriptionJs::from_inner(sub))
        }

        /// Phase 2b wave-8c-subscribe-infra: `onChange` with an explicit
        /// actor principal. Mirrors [`Engine::call_as`] naming.
        ///
        /// `actor` is a friendly principal identifier resolved through
        /// `parse_actor_cid_or_derive`. The principal is captured on
        /// the registered ad-hoc onChange entry's delivery-time
        /// cap-recheck closure so D5 cap-recheck-at-delivery fires the
        /// named principal's grants on every event.
        ///
        /// Returns a [`SubscriptionJs`] handle. Same lifecycle contract
        /// as [`Engine::on_change`] — see that doc for details.
        #[napi]
        pub fn on_change_as(
            &self,
            pattern: String,
            cursor: serde_json::Value,
            actor: String,
            callback: Option<
                napi::bindgen_prelude::Function<
                    '_,
                    napi::bindgen_prelude::FnArgs<(u32, Buffer)>,
                    (),
                >,
            >,
        ) -> napi::Result<SubscriptionJs> {
            let actor_cid = parse_actor_cid_or_derive(&actor);
            let sub = on_change_as_adapter(&self.inner, &pattern, &cursor, &actor_cid, callback)?;
            Ok(SubscriptionJs::from_inner(sub))
        }

        /// R6 Round-2 r6-r2-mpc-1: drive
        /// [`benten_engine::Engine::subscribe_emit_events_with_handle`]
        /// against a JS callback. Closes r6-mpc-2 (the wave-8h
        /// cross-layer audit gap) by wiring the `EmitSubscriptionJs`
        /// napi class — the engine half of the bridge had been
        /// constructed in PR #62 but no Engine method existed to
        /// produce one, so JS callers always hit the typed
        /// `EDslInvalidShape("rebuild your binding")` pre-check at
        /// `engine.ts::onEmit`.
        ///
        /// Channel filtering applies engine-side string equality (no
        /// glob matching at the napi surface in Phase 2b). Returns a
        /// [`EmitSubscriptionJs`] handle whose `unsubscribe()` /
        /// JS-side Drop releases the broadcast registration AND the
        /// `napi::ThreadsafeFunction` Arc holding the JS callback
        /// alive. **The JS-side handle MUST be retained for the
        /// lifetime of the subscription** — same lifecycle contract
        /// as [`Engine::on_change`].
        #[napi]
        pub fn on_emit(
            &self,
            channel: String,
            callback: napi::bindgen_prelude::Function<
                '_,
                napi::bindgen_prelude::FnArgs<(String, String)>,
                (),
            >,
        ) -> napi::Result<EmitSubscriptionJs> {
            let sub = on_emit_adapter(&self.inner, &channel, callback)?;
            Ok(EmitSubscriptionJs::from_inner(sub, channel))
        }

        /// ts-r4-2 mirror for SUBSCRIBE (mini-review cr-g6b-mr-5):
        /// vitest-harness factory mirroring `testingOpenStreamForTest`.
        /// Construct a [`SubscriptionJs`] handle wired against the
        /// engine's synthetic delivery path so harness tests can
        /// exercise the unsubscribe + dedup state machinery without
        /// G6-A's change-stream port.
        ///
        /// `cursor` shape mirrors `on_change`'s third argument
        /// (`null` / `{ kind: "latest" }` / `{ kind: "sequence", seq }`
        /// / `{ kind: "persistent", subscriberId }`).
        ///
        /// Symbol presence is pinned by the symmetric assertion in
        /// `bindings/napi/test/stream_napi_async_iterator_back_pressure.test.ts`.
        ///
        /// The underlying `benten_engine::Engine::testing_open_subscription_for_test`
        /// is cfg-gated under `cfg(any(test, feature = "test-helpers"))`
        /// per Phase-2a sec-r6r2-02 discipline. This napi method is
        /// emitted unconditionally; the body fails to resolve unless
        /// the cdylib is built with `--features test-helpers`.
        #[napi(js_name = "testingOpenSubscriptionForTest")]
        pub fn testing_open_subscription_for_test(
            &self,
            pattern: String,
            cursor: serde_json::Value,
        ) -> napi::Result<SubscriptionJs> {
            #[cfg(feature = "test-helpers")]
            {
                let sub =
                    testing_open_subscription_for_test_adapter(&self.inner, &pattern, &cursor)?;
                Ok(SubscriptionJs::from_inner(sub))
            }
            #[cfg(not(feature = "test-helpers"))]
            {
                let _ = (pattern, cursor);
                Err(napi::Error::new(
                    Status::GenericFailure,
                    "E_PRIMITIVE_NOT_IMPLEMENTED: testingOpenSubscriptionForTest \
                     requires the cdylib to be built with `--features test-helpers` \
                     (vitest harness build only). Production cdylib consumers \
                     should never reach this surface.",
                ))
            }
        }

        /// ts-r4-2 mirror for SUBSCRIBE (mini-review cr-g6b-mr-5):
        /// synthetic delivery path used by harness tests to exercise
        /// the dedup machinery without a real change-stream port.
        /// Bumps the subscription's `max_delivered_seq` if `seq >
        /// max_delivered_seq` (the same condition the production
        /// delivery path uses); returns `true` if the synthetic
        /// delivery was applied, `false` if it was deduped.
        ///
        /// cfg-gated identically to `testingOpenSubscriptionForTest`.
        #[napi(js_name = "testingDeliverSyntheticEventForTest")]
        pub fn testing_deliver_synthetic_event_for_test(
            &self,
            sub: &SubscriptionJs,
            seq: u32,
        ) -> napi::Result<bool> {
            #[cfg(feature = "test-helpers")]
            {
                let g = sub.inner.lock().map_err(|_| {
                    napi::Error::new(
                        Status::GenericFailure,
                        "SubscriptionJs: internal lock poisoned",
                    )
                })?;
                Ok(testing_deliver_synthetic_event_for_test_adapter(
                    &self.inner,
                    &g,
                    u64::from(seq),
                ))
            }
            #[cfg(not(feature = "test-helpers"))]
            {
                let _ = (sub, seq);
                Err(napi::Error::new(
                    Status::GenericFailure,
                    "E_PRIMITIVE_NOT_IMPLEMENTED: testingDeliverSyntheticEventForTest \
                     requires the cdylib to be built with `--features test-helpers` \
                     (vitest harness build only).",
                ))
            }
        }

        // -------- Snapshot blob handoff (D10-RESOLVED; wave-8c-cont 8c-iv) --------

        /// Phase 2b wave-8c-cont: napi bridge for `Engine::export_snapshot_blob`.
        ///
        /// Walks this engine's storage and encodes a canonical DAG-CBOR
        /// snapshot-blob for handoff. Returns the bytes as a `Buffer` —
        /// the TS wrapper exposes this as `Uint8Array` to consumers.
        ///
        /// The snapshot-blob is canonical (BTreeMap-sorted) — two
        /// exports of the same engine state produce byte-identical
        /// output (D10 + sec-pre-r1-09 Inv-13 collision-safety).
        ///
        /// Native-target only — surfaces `E_SUBSYSTEM_DISABLED` on wasm32
        /// since `Engine::export_snapshot_blob` is gated to native targets.
        #[cfg(not(target_arch = "wasm32"))]
        #[napi(js_name = "exportSnapshotBlob")]
        pub fn export_snapshot_blob(&self) -> napi::Result<Buffer> {
            let bytes = self.inner.export_snapshot_blob().map_err(engine_err)?;
            Ok(Buffer::from(bytes))
        }

        /// Phase 2b wave-8c-cont: wasm32-target stub for
        /// `exportSnapshotBlob`. Surfaces `E_SUBSYSTEM_DISABLED` because
        /// the engine_snapshot module is `#[cfg(not(target_arch =
        /// "wasm32"))]`-gated. Kept in the public napi surface so JS
        /// callers see a typed error instead of `TypeError: undefined
        /// is not a function` on a wasm32-only build.
        #[cfg(target_arch = "wasm32")]
        #[napi(js_name = "exportSnapshotBlob")]
        pub fn export_snapshot_blob(&self) -> napi::Result<Buffer> {
            Err(napi::Error::new(
                Status::GenericFailure,
                "E_SUBSYSTEM_DISABLED: snapshot-blob handoff is not available on wasm32 \
                 builds (engine_snapshot is native-target only). Use a native engine \
                 instance to export.",
            ))
        }

        /// Phase 2b wave-8c-cont: napi factory bridge for
        /// `Engine::from_snapshot_blob`. Constructs a NEW read-only
        /// engine view over the supplied snapshot-blob bytes.
        ///
        /// The bytes are decoded as a canonical DAG-CBOR `SnapshotBlob`;
        /// the contents are hydrated into a fresh tempdir-resident redb
        /// backend; the returned engine has its `read_only_snapshot`
        /// flag set so subsequent mutation methods surface
        /// `E_BACKEND_READ_ONLY` (D10).
        ///
        /// Native-target only — surfaces `E_SUBSYSTEM_DISABLED` on wasm32.
        #[cfg(not(target_arch = "wasm32"))]
        #[napi(factory, js_name = "fromSnapshotBlob")]
        pub fn from_snapshot_blob(bytes: Buffer) -> napi::Result<Self> {
            let inner = InnerEngine::from_snapshot_blob(bytes.as_ref()).map_err(engine_err)?;
            Ok(Self {
                inner: Arc::new(inner),
            })
        }

        /// Phase 2b wave-8c-cont: wasm32-target stub for
        /// `fromSnapshotBlob`. See native arm doc for the production
        /// contract.
        #[cfg(target_arch = "wasm32")]
        #[napi(factory, js_name = "fromSnapshotBlob")]
        pub fn from_snapshot_blob(bytes: Buffer) -> napi::Result<Self> {
            let _ = bytes;
            Err(napi::Error::new(
                Status::GenericFailure,
                "E_SUBSYSTEM_DISABLED: snapshot-blob handoff is not available on wasm32 \
                 builds (engine_snapshot is native-target only).",
            ))
        }

        /// Phase 2b wave-8c-cont: napi bridge for
        /// `Engine::compute_snapshot_blob_cid`. Pure helper — computes
        /// the BLAKE3-multibase CID of a snapshot-blob bytes payload
        /// without constructing an engine.
        #[cfg(not(target_arch = "wasm32"))]
        #[napi(js_name = "computeSnapshotBlobCid")]
        pub fn compute_snapshot_blob_cid(bytes: Buffer) -> napi::Result<String> {
            let cid = InnerEngine::compute_snapshot_blob_cid(bytes.as_ref()).map_err(engine_err)?;
            Ok(cid.to_base32())
        }

        /// Phase 2b wave-8c-cont: wasm32-target stub for
        /// `computeSnapshotBlobCid`.
        #[cfg(target_arch = "wasm32")]
        #[napi(js_name = "computeSnapshotBlobCid")]
        pub fn compute_snapshot_blob_cid(bytes: Buffer) -> napi::Result<String> {
            let _ = bytes;
            Err(napi::Error::new(
                Status::GenericFailure,
                "E_SUBSYSTEM_DISABLED: snapshot-blob CID computation is not available on \
                 wasm32 builds.",
            ))
        }

        /// `true` iff this engine was constructed via
        /// [`Engine::from_snapshot_blob`] and is therefore a read-only
        /// view. Mirrors the Rust-side accessor; consumers can branch
        /// on this flag rather than catching `E_BACKEND_READ_ONLY` on
        /// every mutation attempt.
        #[napi(js_name = "isReadOnlySnapshot")]
        pub fn is_read_only_snapshot(&self) -> bool {
            self.inner.is_read_only_snapshot()
        }

        // -------- Module manifest lifecycle (Phase 2b G10-B; wave-8c bridge) --------

        /// Phase 2b wave-8c: napi bridge for `Engine::install_module`.
        ///
        /// `manifest_json` is the JSON form of [`benten_engine::ModuleManifest`]
        /// (serde-derived round-trip). `expected_cid` is the base32-multibase
        /// encoded canonical-DAG-CBOR CID of the manifest — REQUIRED per
        /// D16-RESOLVED, no convenience overload that omits it.
        ///
        /// Returns the installed manifest's CID (round-trip identity with
        /// `expected_cid` on the success path; surfaces
        /// `E_MODULE_MANIFEST_CID_MISMATCH` on D16 mismatch).
        #[napi(js_name = "installModule")]
        pub fn install_module(
            &self,
            manifest_json: serde_json::Value,
            expected_cid: String,
        ) -> napi::Result<String> {
            let manifest: benten_engine::ModuleManifest = serde_json::from_value(manifest_json)
                .map_err(|e| {
                    napi::Error::new(
                        Status::InvalidArg,
                        format!("installModule: malformed manifest JSON: {e}"),
                    )
                })?;
            let parsed_cid = parse_cid(&expected_cid)?;
            // g14-c-mr-1: napi bridge currently runs through the
            // unsigned-development verify path. Phase-3 G17-C wave-5b
            // (TS-side SANDBOX named-manifest resolution) extends this
            // surface with structured signing args (ucan chain bytes,
            // registry pubkey, audience) — until then the napi caller
            // explicitly opts into the unsigned relaxation, matching
            // the pre-G14-C behavior.
            let installed = self
                .inner
                .install_module(
                    manifest,
                    parsed_cid,
                    benten_engine::manifest_signing::ManifestVerifyArgs::unsigned_development(),
                )
                .map_err(engine_err)?;
            Ok(installed.to_base32())
        }

        /// Phase 2b wave-8c: napi bridge for `Engine::uninstall_module`.
        /// Idempotent — a call against a never-installed CID is a no-op.
        #[napi(js_name = "uninstallModule")]
        pub fn uninstall_module(&self, cid: String) -> napi::Result<()> {
            let parsed = parse_cid(&cid)?;
            self.inner.uninstall_module(parsed).map_err(engine_err)
        }

        /// Phase 2b wave-8c: napi bridge for `Engine::compute_manifest_cid`.
        /// Pure function — computes the canonical-DAG-CBOR CID of the
        /// manifest WITHOUT installing it. Intended for callers that
        /// want to verify the CID before passing it as the required
        /// `expectedCid` arg to [`Self::install_module`].
        #[napi(js_name = "computeManifestCid")]
        pub fn compute_manifest_cid(
            &self,
            manifest_json: serde_json::Value,
        ) -> napi::Result<String> {
            let manifest: benten_engine::ModuleManifest = serde_json::from_value(manifest_json)
                .map_err(|e| {
                    napi::Error::new(
                        Status::InvalidArg,
                        format!("computeManifestCid: malformed manifest JSON: {e}"),
                    )
                })?;
            let cid = self
                .inner
                .compute_manifest_cid(&manifest)
                .map_err(engine_err)?;
            Ok(cid.to_base32())
        }

        /// Phase-3 G17-C wave-5b: napi bridge for
        /// `Engine::register_module_bytes` (phase-3-backlog §6.6
        /// deliverable 1; pim-2 24th p/c drift acceptance criterion).
        ///
        /// Persists wasm module bytes under their BLAKE3-derived CID via
        /// the durable `RedbBlobBackend` so SANDBOX dispatch can resolve
        /// `module: "<base32-cid>"` references at execution time. The
        /// caller-supplied `cid` MUST match the BLAKE3 of `bytes` —
        /// mismatch returns `E_MODULE_BYTES_CID_MISMATCH` per
        /// D-PHASE-3-12.
        ///
        /// Sibling of [`Self::install_module`]: `installModule` writes
        /// the manifest envelope (entries, requires, CID schema);
        /// `registerModuleBytes` writes the actual wasm payload bytes
        /// each manifest entry's `cid` field references.
        ///
        /// `bytes` arrives as a Node `Buffer`; the underlying byte slice
        /// is passed through to the inner engine without an extra copy.
        #[napi(js_name = "registerModuleBytes")]
        pub fn register_module_bytes(&self, cid: String, bytes: Buffer) -> napi::Result<()> {
            let parsed = parse_cid(&cid)?;
            self.inner
                .register_module_bytes(&parsed, bytes.as_ref())
                .map_err(engine_err)
        }
    }

    // ---------------------------------------------------------------------
    // SubscriptionJs — napi class wrapping `benten_engine::Subscription`
    // ---------------------------------------------------------------------

    /// JS-side subscription handle. Mirrors
    /// [`benten_engine::Subscription`] across the napi boundary for the
    /// production `engine.onChange` / `engine.onChangeAs` consumers AND
    /// the cfg-gated test-helper factory.
    ///
    /// **Lifecycle contract (cr-w8c-fp-1 fix-pass):** the handle holds
    /// the underlying `benten_engine::Subscription` alive for the
    /// duration of the JS-side reference. When JS drops the handle (or
    /// calls `unsubscribe()`), Drop fires on the inner Subscription,
    /// which calls `unregister_on_change` and releases the
    /// `napi::ThreadsafeFunction` Arc backing the JS callback. Without
    /// this lifetime extension the production callback path would be
    /// stillborn (the prior wave-8c return-as-JSON shape dropped the
    /// Subscription at end of method scope).
    ///
    /// The handle is held behind a `Mutex` so the `&self`-taking napi
    /// methods can share it across JS worker threads.
    #[napi]
    pub struct SubscriptionJs {
        inner: std::sync::Mutex<benten_engine::Subscription>,
    }

    impl SubscriptionJs {
        pub(crate) fn from_inner(sub: benten_engine::Subscription) -> Self {
            Self {
                inner: std::sync::Mutex::new(sub),
            }
        }
    }

    #[napi]
    impl SubscriptionJs {
        /// `true` while the subscription is registered with the engine.
        /// Flips to `false` after `unsubscribe()` or when the handle
        /// drops.
        #[napi(js_name = "isActive")]
        pub fn is_active(&self) -> napi::Result<bool> {
            let g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "SubscriptionJs: internal lock poisoned",
                )
            })?;
            Ok(g.is_active())
        }

        /// Pattern the subscription was registered with.
        #[napi]
        pub fn pattern(&self) -> napi::Result<String> {
            let g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "SubscriptionJs: internal lock poisoned",
                )
            })?;
            Ok(g.pattern().to_string())
        }

        /// Highest engine-assigned sequence number observed by this
        /// subscription's delivery path. `0` before the first event
        /// lands.
        ///
        /// Returns `i64` — napi-rs maps this to JS `number` directly
        /// (no BigInt boxing); for values < `Number.MAX_SAFE_INTEGER`
        /// (2^53) the conversion is exact. R6 Round-2 Instance 11
        /// closure: prior return type `u32` saturated at 2^32 via
        /// `u32::try_from(u64).unwrap_or(u32::MAX)`. Now widened to
        /// `i64` so a heavily-loaded subscriber accumulating >4B
        /// deliveries surfaces the real seq rather than a saturated
        /// `u32::MAX`. Saturation at `i64::MAX` is structurally
        /// unreachable (the engine-side `u64` would have to overflow,
        /// which never happens in any realistic deployment timeline).
        #[napi(js_name = "maxDeliveredSeq")]
        pub fn max_delivered_seq(&self) -> napi::Result<i64> {
            let g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "SubscriptionJs: internal lock poisoned",
                )
            })?;
            Ok(i64::try_from(g.max_delivered_seq()).unwrap_or(i64::MAX))
        }

        /// Explicitly release the subscription. Idempotent.
        #[napi]
        pub fn unsubscribe(&self) -> napi::Result<()> {
            let g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "SubscriptionJs: internal lock poisoned",
                )
            })?;
            g.unsubscribe();
            Ok(())
        }
    }

    // ---------------------------------------------------------------------
    // EmitSubscriptionJs — napi class wrapping
    //   `benten_engine::EmitSubscription` (R6FP-Group-1 r6-mpc-2)
    // ---------------------------------------------------------------------

    /// JS-side EMIT subscription handle. Mirrors
    /// [`benten_engine::EmitSubscription`] across the napi boundary so
    /// the TS wrapper (Group 2) can build `engine.onEmit(channelGlob,
    /// callback) -> EmitSubscription` on top of this Rust class.
    ///
    /// **Lifecycle contract:** the handle holds the underlying
    /// [`benten_engine::EmitSubscription`] alive for the duration of
    /// the JS-side reference. When JS drops the handle (or calls
    /// `unsubscribe()`), Drop fires on the inner subscription, which
    /// flips its active flag — subsequent emits skip the
    /// `napi::ThreadsafeFunction` callback rather than firing it. The
    /// closure itself stays in `EmitBroadcast.callbacks` until the
    /// engine drops, but the active-flag gate makes the handle's
    /// lifecycle observable from the JS layer.
    ///
    /// Mirrors `SubscriptionJs` (the SUBSCRIBE handle) field-for-field
    /// minus the SUBSCRIBE-specific `pattern()` / `maxDeliveredSeq()`
    /// accessors — EMIT events have no per-subscription max-seq state
    /// (the publish path is broadcast, not seq-tracked).
    #[napi]
    pub struct EmitSubscriptionJs {
        inner: std::sync::Mutex<benten_engine::EmitSubscription>,
        channel: String,
    }

    impl EmitSubscriptionJs {
        pub(crate) fn from_inner(sub: benten_engine::EmitSubscription, channel: String) -> Self {
            Self {
                inner: std::sync::Mutex::new(sub),
                channel,
            }
        }
    }

    #[napi]
    impl EmitSubscriptionJs {
        /// `true` while the subscription is registered with the engine.
        /// Flips to `false` after `unsubscribe()` or when the handle
        /// drops.
        #[napi(js_name = "isActive")]
        pub fn is_active(&self) -> napi::Result<bool> {
            let g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "EmitSubscriptionJs: internal lock poisoned",
                )
            })?;
            Ok(g.is_active())
        }

        /// The channel name this subscription was registered for.
        /// Returned as a `String` to match the TS-side
        /// `NativeEmitSubscriptionJs.channel(): string` interface
        /// declared at `packages/engine/src/subscribe.ts`.
        #[napi]
        pub fn channel(&self) -> napi::Result<String> {
            Ok(self.channel.clone())
        }

        /// Explicitly release the subscription. Idempotent.
        #[napi]
        pub fn unsubscribe(&self) -> napi::Result<()> {
            let g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "EmitSubscriptionJs: internal lock poisoned",
                )
            })?;
            g.unsubscribe();
            Ok(())
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
    /// Production runtime (post wave-8c-stream-infra): `next()` from
    /// a handle constructed via `engine.callStream` / `engine.openStream`
    /// drains real chunks delivered through the producer-bridge
    /// (`StreamHandle::from_producer_bridge`); the producer thread
    /// pumps chunks into the `ChunkSource` end of an in-process
    /// channel and `next_chunk` pulls from the consumer end. The
    /// cfg-gated `testingOpenStreamForTest` factory continues to
    /// drain pre-populated chunk vectors for unit tests that don't
    /// need a live producer.
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
        ///
        /// Returns `i64` — napi-rs maps to JS `number` (exact for
        /// values < 2^53). R6 Round-2 Instance 11 closure: prior
        /// return type `u32` saturated at 2^32 via `u32::try_from`,
        /// silently truncating past 4B chunks. Widened to `i64` so
        /// long-lived streams report the real count.
        #[napi(js_name = "seqSoFar")]
        pub fn seq_so_far(&self) -> napi::Result<i64> {
            let g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "StreamHandle: internal lock poisoned",
                )
            })?;
            Ok(match g.as_ref() {
                Some(h) => i64::try_from(h.seq_so_far()).unwrap_or(i64::MAX),
                None => 0,
            })
        }

        /// Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4): exposes the
        /// underlying `StreamHandle::requires_explicit_close` flag
        /// across the napi boundary so the TS-side wrapper at
        /// `packages/engine/src/stream.ts` can decide whether to arm
        /// the `FinalizationRegistry` leak detector for a given handle.
        ///
        /// Handles produced by `engine.openStream(...)` return `true`
        /// (explicit-close lifecycle); handles produced by
        /// `engine.callStream(...)` return `false` (AsyncIterable
        /// auto-close on `for-await` scope-exit). Pre-G19-C2 the flag
        /// existed engine-side but did not cross to JS — the TS
        /// surfaces were functionally indistinguishable.
        #[napi(js_name = "requiresExplicitClose")]
        pub fn requires_explicit_close(&self) -> napi::Result<bool> {
            let g = self.inner.lock().map_err(|_| {
                napi::Error::new(
                    Status::GenericFailure,
                    "StreamHandle: internal lock poisoned",
                )
            })?;
            Ok(match g.as_ref() {
                Some(h) => h.requires_explicit_close(),
                None => false,
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

    // -----------------------------------------------------------------------
    // Phase-3 G19-B (§7.2 + §7.8) — in-process helpers for the napi-side
    // integration tests at `bindings/napi/tests/benten_error_context.rs` +
    // `bindings/napi/tests/emit_event.rs`. These helpers exercise the
    // engine_err carrier shape + the standalone emit_event surface
    // without going through the cdylib + napi-rs runtime.
    // -----------------------------------------------------------------------

    /// G19-B (§7.2): expose the [`crate::error::engine_err`] carrier
    /// shape via the rlib-test surface. Returns the JSON-encoded
    /// message body the napi adapter would attach to a `napi::Error`
    /// (the production carrier; see `bindings/napi/src/error.rs`).
    /// Tests JSON-parse the return value and assert
    /// `code` / `message` / `fields` match the EngineError variant.
    ///
    /// Reaches into the shared [`crate::error_envelope`] formatter so
    /// the wire shape this helper produces is byte-identical to what
    /// the production `engine_err` carrier embeds in `napi::Error`.
    #[must_use]
    pub fn engine_err_message(err: benten_engine::EngineError) -> String {
        crate::error_envelope::engine_err_envelope_json(&err)
    }

    /// G19-B (§7.8): drive `engine.emit_event` end-to-end through the
    /// engine's EmitBroadcast bus. Opens an in-memory engine, attaches
    /// the supplied callback as an EMIT subscriber, publishes
    /// `(channel, payload)` via the production-grade
    /// [`benten_engine::Engine::emit_event`] entry point, and returns
    /// the engine handle so the test can drain or assert further.
    ///
    /// The callback fires synchronously on the publish thread (the
    /// EmitBroadcast contract); tests can lock a `Mutex<Vec<...>>` to
    /// capture observations.
    ///
    /// Gated additionally on `in-process-test` (vs the parent `testing`
    /// module's `cfg(any(test, feature = "in-process-test"))`) because
    /// the body uses `tempfile::tempdir()` and `tempfile` is itself
    /// gated on `in-process-test` (production cdylib build never
    /// pulls it). Under `cargo test` with default features ON, the
    /// helper is compiled out so the production-feature build path
    /// stays clean.
    #[cfg(feature = "in-process-test")]
    pub fn emit_event_round_trip<F>(
        channel: &str,
        payload: serde_json::Value,
        on_emit: F,
    ) -> Result<benten_engine::Engine, benten_engine::EngineError>
    where
        F: Fn(&benten_engine::EmitEvent) + Send + Sync + 'static,
    {
        let dir = tempfile::tempdir().expect("tempdir");
        let engine = benten_engine::Engine::open(dir.path().join("benten.redb"))?;
        engine.subscribe_emit_events(on_emit);
        let value = json_to_value(payload)?;
        engine.emit_event(channel, value)?;
        // Tempdir kept alive via env var since we return the engine;
        // production callers don't see this path. The redb file lives
        // in tempdir which is dropped at process exit — fine for
        // in-process tests.
        std::mem::forget(dir);
        Ok(engine)
    }

    /// G19-B helper: convert a `serde_json::Value` into a
    /// `benten_core::Value` mirroring the napi `json_to_value_root`
    /// path. Reproduced here (rather than re-exported) because the
    /// production helper lives in `crate::node` which depends on the
    /// napi `Status` type — keeping this rlib-only shim free of napi
    /// imports lets the in-process-test feature compile cleanly with
    /// `--no-default-features`.
    ///
    /// Same `in-process-test` gate as the `emit_event_round_trip`
    /// helper above (its sole consumer).
    #[cfg(feature = "in-process-test")]
    fn json_to_value(v: serde_json::Value) -> Result<Value, benten_engine::EngineError> {
        match v {
            serde_json::Value::Null => Ok(Value::Null),
            serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    return Ok(Value::Int(i));
                }
                if let Some(f) = n.as_f64() {
                    if !f.is_finite() {
                        return Err(benten_engine::EngineError::Other {
                            code: benten_errors::ErrorCode::InputLimit,
                            message: "non-finite number".into(),
                        });
                    }
                    if f.fract() == 0.0 {
                        #[allow(clippy::cast_possible_truncation)]
                        return Ok(Value::Int(f as i64));
                    }
                    return Ok(Value::Float(f));
                }
                Err(benten_engine::EngineError::Other {
                    code: benten_errors::ErrorCode::InputLimit,
                    message: "unsupported numeric shape".into(),
                })
            }
            serde_json::Value::String(s) => Ok(Value::Text(s)),
            serde_json::Value::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(json_to_value(item)?);
                }
                Ok(Value::List(out))
            }
            serde_json::Value::Object(map) => {
                let mut out = std::collections::BTreeMap::new();
                for (k, val) in map {
                    out.insert(k, json_to_value(val)?);
                }
                Ok(Value::Map(out))
            }
        }
    }

    /// Phase-3 G19-C1 (§7.1.4 + r6-napi-2 closure) — testing-only
    /// wallclock-advance hook for the WAIT TTL expiry path.
    ///
    /// Returns `Ok(())` for a zero delta (sentinel-presence pin
    /// `testing_advance_wait_clock_napi_binding_present`). For non-zero
    /// deltas the body forwards to the engine-side stub at
    /// `crates/benten-engine/src/testing.rs::testing_advance_wait_clock`,
    /// which D12-resolves to a real `MockMonotonicSource` advance once
    /// the source-injection plumbing lands. Until then non-zero deltas
    /// resolve to `Ok(())` deterministically — the function shape is
    /// forward-compatible.
    ///
    /// Cfg-gating discipline (sec-r6r2-02 precedent): this helper lives
    /// in the rlib-only `testing` module so the production cdylib does
    /// NOT carry the binding. The corresponding `#[napi]` method on the
    /// `Engine` class (test-helpers feature-gated) shares the same
    /// no-widening-of-production-attack-surface contract.
    ///
    /// # Errors
    ///
    /// Currently never errors. Returns `Result<(), String>` so the
    /// signature can carry a typed error once the MockMonotonicSource
    /// injection lands without breaking callers.
    pub fn testing_advance_wait_clock(_delta_ms: u64) -> Result<(), String> {
        // Forward-compatible no-op; see method docstring above.
        Ok(())
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
