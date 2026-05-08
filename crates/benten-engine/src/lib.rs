//! # benten-engine
//!
//! Orchestrator crate composing the Benten graph engine public API.
//!
//! [`EngineBuilder`] selects capability policy, IVM subscriber, production
//! mode guard, and durability. [`Engine`] exposes CRUD (Node + Edge),
//! `register_subgraph` (runs G6 invariants), `transaction` (closure over
//! [`benten_graph::Transaction`]), `snapshot` (MVCC handle), and the three
//! privileged system-zone entry points
//! `grant_capability` / `create_view` / `revoke_capability`.
//! [`change::ChangeBroadcast`] fans committed events to every registered
//! subscriber.
//!
//! Call-time primitive dispatch (register_crud → evaluator → primitive
//! execution) is threaded through `impl PrimitiveHost for Engine`
//! (see [`primitive_host`]) — host-side writes are buffered inside the
//! active-call frame and replayed atomically at commit.
//!
//! ## Module layout (post R6 Wave 2 split)
//!
//! - [`error`] — [`EngineError`] + conversions.
//! - [`builder`] — [`EngineBuilder`] fluent surface + `BackendGrantReader`.
//! - [`engine`] — [`Engine`] struct + CRUD / register / dispatch /
//!   view-read / transaction / snapshot / diagnostics.
//! - [`primitive_host`] — `impl PrimitiveHost for Engine` + the buffered
//!   replay state (`ActiveCall` / `PendingHostOp`).
//! - [`subgraph_spec`] — DSL builders (`SubgraphSpec`, `PrimitiveSpec`,
//!   `WriteSpec`, …).
//! - [`outcome`] — response shapes (`Outcome`, `Trace`, …).
//! - [`engine_transaction`] — `EngineTransaction` passed into the
//!   `.transaction(|tx| …)` closure.
//! - [`change`] — [`change::ChangeBroadcast`] subscriber fan-out.
//! - [`change_probe`] — [`ChangeProbe`] observation handle.
//! - `testing` — integration-test helpers consumed by sibling crates
//!   (cfg-gated behind `any(test, feature = "test-helpers")`; not part
//!   of the production cdylib surface).
//!
//! `lib.rs` is deliberately thin after the R6 Wave 2 split (R-major-01): it
//! declares the modules and re-exports the full public surface so call-site
//! paths like `benten_engine::Engine` stay stable.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(
    clippy::todo,
    reason = "Phase-1 scope: primitive-dispatch deliverables remain as typed todos until benten-eval's evaluator gains a PrimitiveHost trait (Phase 2)."
)]

// G13-C BLOCKER-2 fix-pass (browser-backend cfg-gating): the
// `EngineBuilder` is redb-bound (carries `Option<RedbBackend>` field +
// `assemble(backend: RedbBackend)` body). Gated to NOT-`browser-backend`
// so the wasm32-unknown-unknown thin-client target compiles. The browser
// bundle constructs `EngineGeneric<BrowserBackend>` directly via the
// generic `from_parts*` constructors on the `impl<B: GraphBackend>
// EngineGeneric<B>` block.
#[cfg(not(feature = "browser-backend"))]
pub mod builder;
// Phase-3 G19-E (wave-7b): per-handler TRANSFORM AST cache. Closes
// `docs/future/phase-2-backlog.md` §9.2 by storing parsed `Expr` ASTs
// keyed by `(handler_cid, node_id)` so per-call dispatch consults the
// cache via `PrimitiveHost::cached_transform_ast` instead of re-parsing
// the `expr` source string. See module rustdoc for the population +
// invalidation contract.
pub mod ast_cache;
// Phase-3 G13-pre-C scaffold (wave-1pre) — shared per-event
// read-cap-coverage helper. Both G14-D F6 SUBSCRIBE filtering (wave-5a)
// and G17-A1 ESC-9 `live_cap_check` (wave-5b) consume this module from
// day one per `seq-minor-6` (extract first; no inline-then-refactor).
// See `cap_recheck.rs` rustdoc for the design pins.
pub mod cap_recheck;
pub mod cap_snapshot_hash;
pub mod change;
pub mod change_probe;
pub mod handler_router;
// Phase-3 G15-A wave-5a — materialization-time per-row READ gate for
// IVM-materialized views. Closes Compromise #11 in coordination with
// G14-D delivery-time gate per `ivm-major-2` + `ds-r4r2-7` shared-trait
// callout (composes [`cap_recheck::CapRecheckFn`]).
pub mod ivm_view_read_gate;
pub mod thin_client_subscribe;
// Wave-8h audit-gap fix — EMIT-only broadcast channel so a handler with
// a standalone EMIT primitive (no backing WRITE) produces an observable
// event. Mirrors `change::ChangeBroadcast` but for emit-only events.
pub mod emit_broadcast;
pub mod engine;
// G13-C BLOCKER-2 fix-pass: redb-coupled CRUD/caps/diagnostics modules
// gated to NOT-`browser-backend`. They consume `RedbBackend` inherent
// methods (`get_node`, `get_by_label`, `transaction(|tx| ...)`, etc.)
// which the umbrella `GraphBackend` trait does not surface (deferred
// to phase-3-backlog §1.2-followup).
#[cfg(not(feature = "browser-backend"))]
pub(crate) mod engine_caps;
// Phase 2b G7-A — workspace-level `engine.toml` loader (Ben's brief
// addition). Compile-time wasm32-disabled per the same sec-pre-r1-05
// cut as SANDBOX itself; on wasm32 the engine config is always
// built-in defaults.
#[cfg(not(target_arch = "wasm32"))]
pub mod engine_config;
#[cfg(not(feature = "browser-backend"))]
pub(crate) mod engine_crud;
#[cfg(not(feature = "browser-backend"))]
pub(crate) mod engine_diagnostics;
#[cfg(not(feature = "browser-backend"))]
pub mod engine_transaction;
// G13-C BLOCKER-2 fix-pass: engine_views uses `RedbBackend::put_node_with_context`
// + `get_node` inherent paths in the privileged view-creation flow. Gated out
// of the browser thin-client bundle (views are read-only projections of the
// full peer's state in the wasm32 target).
#[cfg(not(feature = "browser-backend"))]
pub(crate) mod engine_views;
pub mod error;
pub mod outcome;
// G13-C BLOCKER-2 fix-pass: primitive_host hosts the
// `impl PrimitiveHost for Engine` boundary that drives evaluator-side
// writes through the engine's redb-bound CRUD path. Browser thin clients
// don't run handlers (stateless reads only per CLAUDE.md baked-in #17),
// so this module is gated out of the wasm32 bundle.
#[cfg(not(feature = "browser-backend"))]
pub mod primitive_host;
pub mod subgraph_spec;
// Phase-3 G21-T1 — engine-side typed-CALL dispatch implementations.
// Wires the 10 typed-CALL ops (Ed25519 sign/verify, BLAKE3, multibase,
// DID resolve, UCAN chain validation, VC verify) to their underlying
// APIs in `benten-id` + `benten-core`. Per CLAUDE.md baked-in #16 +
// #17: full-peer-only (thin clients consume already-verified results
// from full peers); module is `cfg(not(target_arch = "wasm32"))`-
// gated at its single call site in `primitive_host.rs`.
#[cfg(not(target_arch = "wasm32"))]
pub mod typed_call_dispatch;
// R6 round-2 sec-r6r2-02: gate the test-helper surface (`principal_cid`,
// `minimal_wait_handler`, `policy_with_grants`, `counting_capability_policy`,
// `subgraph_bytes_for_handler`, etc.) behind `cfg(any(test, feature =
// "test-helpers"))` so the napi cdylib (which post-G12-E opts only
// into `iteration-budget-test-grade`) does NOT compile this surface
// into production. Sibling crates' integration tests reach in via dev-deps
// that already declare `benten-engine = { features = ["test-helpers"] }`
// (see `benten-eval/Cargo.toml:66`, `benten-graph/Cargo.toml:86`,
// `benten-caps/Cargo.toml:40`), so the test path is unaffected.
#[cfg(any(test, feature = "test-helpers"))]
pub mod testing;

// ---------------------------------------------------------------------------
// Public re-exports — preserve every call-site path that existed before the
// R6 Wave 2 split.
// ---------------------------------------------------------------------------

pub use benten_errors::ErrorCode;
pub use benten_eval::PrimitiveKind;

#[cfg(not(target_arch = "wasm32"))]
pub use engine_config::{EngineConfig, EngineConfigError, SandboxSection};

pub use ast_cache::AstCacheStats;
#[cfg(not(feature = "browser-backend"))]
pub use builder::{EngineBuilder, NOAUTH_STARTUP_LOG};
pub use change_probe::ChangeProbe;
pub use emit_broadcast::{EmitBroadcast, EmitEvent, EmitSubscription};
pub use engine::{CHANGE_STREAM_MAX_BUFFERED, Engine, EngineGeneric};
#[cfg(not(feature = "browser-backend"))]
pub use engine_transaction::EngineTransaction;
pub use error::EngineError;
pub use outcome::{
    AnchorHandle, BudgetExhaustedView, DiagnosticInfo, HandlerPredecessors, NestedTx, Outcome,
    OutcomeExt, ReadViewOptions, RegisterReplaceOutcome, TerminalError, Trace, TraceStep,
    UserViewInputPattern, UserViewSpec, UserViewSpecBuilder, ViewCreateOptions,
};
pub use subgraph_spec::{
    GrantSubject, IntoCallInput, IntoSubgraphSpec, IterateBody, PrimitiveSpec, RevokeScope,
    RevokeSubject, SubgraphSpec, SubgraphSpecBuilder, WriteSpec,
};

// G13-C BLOCKER-2 fix-pass: engine_modules consumes `RedbBackend::transaction(|tx| ...)`
// inherent on the closure-based execution path. Gated to NOT-`browser-backend`.
#[cfg(not(feature = "browser-backend"))]
pub mod engine_modules;
// Phase-3 G14-C wave-4b — durable handler-version chain (Compromise #18
// closure). Persists `system:HandlerVersion` zone Nodes per registration;
// rebuilds the in-memory `BTreeMap<HandlerId, Vec<Cid>>` chain at engine
// open via `Engine::rehydrate_handler_version_chains_from_zone`.
#[cfg(not(feature = "browser-backend"))]
pub mod handler_versions;
// Phase-3 G14-C wave-4b — manifest-signing wire-through (Compromise #21
// closure). Ed25519 sign/verify with UCAN-proof-chain primary +
// publisher-key-registry fallback per D-PHASE-3-20 / crypto-minor-5.
#[cfg(not(feature = "browser-backend"))]
pub mod manifest_signing;
// Phase-3 G14-C wave-4b — anchor-store consolidation (cov-f3 residual
// from `docs/future/phase-2-backlog.md` §6.3).
#[cfg(not(feature = "browser-backend"))]
pub mod anchor_store;
pub mod engine_sandbox;
// Phase-2b G10-A-wasip1 (D10-RESOLVED): snapshot-blob handoff API on
// `Engine` (`export_snapshot_blob` / `from_snapshot_blob` /
// `compute_snapshot_blob_cid`). Native-target only — wasm32 builds
// don't yet ship the redb tempdir backend the snapshot-blob construct
// path materializes into; revisited when G10-A-browser lands a
// snapshot-blob-backed engine for wasm32-unknown-unknown.
// G13-C BLOCKER-2 fix-pass: engine_snapshot uses RedbBackend tempdir +
// EngineBuilder. Gated to NOT-`browser-backend` (additive to the
// pre-existing not-wasm32 gate from Phase-2b G10-A-wasip1).
#[cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
pub mod engine_snapshot;
// G13-C BLOCKER-2 fix-pass: engine_stream consumes `engine_wait::HandlerRef`
// (handler-driven STREAM primitive). Browser thin clients receive STREAM
// chunks over the thin-client subscription protocol from the full peer;
// they don't initiate handler-driven streams locally.
#[cfg(not(feature = "browser-backend"))]
pub mod engine_stream;
pub mod engine_subscribe;
// G13-C BLOCKER-2 fix-pass: engine_wait dispatches calls through the
// `impl Engine`-block `dispatch_call` (redb-coupled) and reads
// `get_node_label_only` off the inherent RedbBackend path. Gated out of
// the browser bundle (WAIT primitive runs on full peers per CLAUDE.md
// baked-in #17).
#[cfg(not(feature = "browser-backend"))]
pub mod engine_wait;
// Phase-2b G12-E — engine-side `RedbSuspensionStore` adapter wiring the
// engine's existing `Arc<RedbBackend>` into `benten_eval::SuspensionStore`.
// Closes the Phase-2a Compromise #10 cross-process WAIT-resume gap +
// retires the test-grade `engine_wait::ENVELOPE_CACHE` surface.
//
// G13-C BLOCKER-2 fix-pass: gated to NOT-`browser-backend` since the
// adapter holds `Arc<RedbBackend>` directly. Browser thin-client tabs
// don't need durable WAIT/SUBSCRIBE persistence — those run on the full
// peer per CLAUDE.md baked-in #17.
#[cfg(not(feature = "browser-backend"))]
pub mod suspension_store;
// Phase-3 G20-A2 (D12 wave-8a) — WAIT TTL GC machinery. Production
// code (NOT test source — backlog miscategorization corrected per
// scope-real-03). Three sweep paths: event-driven (suspend / resume),
// 1h interval backstop, Engine::drop final.
pub mod wait_ttl_gc;
pub use wait_ttl_gc::WaitTtlGcStats;
// Phase 2b G10-B — module manifest format (D9-RESOLVED canonical
// DAG-CBOR; D16-RESOLVED-FURTHER REQUIRED expected_cid arg on
// `Engine::install_module`). See `module_manifest.rs` for the format
// spec and `engine_modules.rs` for the install/uninstall lifecycle.
pub mod module_manifest;
pub mod system_zones;
// Phase-3 G16-B wave-6b — Atrium API surface (engine-side wrapper for
// the iroh transport + Loro CRDT in `benten-sync`). Native-only per
// CLAUDE.md baked-in #17 — gated to non-wasm32 + non-browser-backend
// (browser tabs participate as authenticated thin-client views via
// `thin_client_subscribe.rs`, NOT as full Atrium peers running iroh).
#[cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
pub mod atrium_api;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
pub mod engine_sync;

pub use benten_eval::chunk_sink::{Chunk, ChunkSink};
pub use engine_sandbox::{SANDBOX_UNAVAILABLE_ON_WASM_TEXT, SandboxNodeDescription};
#[cfg(not(feature = "browser-backend"))]
pub use engine_stream::{
    STREAM_GRANT_CEILING_CHUNK_COUNT, STREAM_GRANT_CEILING_WALLCLOCK_MS, StreamCursor, StreamHandle,
};
pub use engine_subscribe::{OnChangeCallback, SubscribeCursor, Subscription};
#[cfg(not(feature = "browser-backend"))]
pub use engine_wait::{ResumePayload, SuspensionOutcome};
#[cfg(not(feature = "browser-backend"))]
pub use suspension_store::RedbSuspensionStore;
// Phase-2b G12-E re-exports of the eval-layer trait + value types so
// downstream consumers (napi bindings, integration tests) can name the
// types without depending on benten-eval directly.
pub use benten_eval::suspension_store::{
    CapSnapshot, InMemorySuspensionStore, SuspensionKey, SuspensionStore, SuspensionStoreError,
    WaitMetadata,
};
pub use module_manifest::{
    ManifestError, ManifestSignature, ManifestSummary, MigrationStep, ModuleManifest,
    ModuleManifestEntry,
};
pub use system_zones::SYSTEM_ZONE_PREFIXES;

// ---------------------------------------------------------------------------
// Tests — lightweight smoke coverage; the heavy lifting lives in
// `tests/` + `tests/integration/`.
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use benten_core::testing::canonical_test_node;

    #[test]
    fn create_then_get_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
        let node = canonical_test_node();
        let cid = engine.create_node(&node).unwrap();
        let fetched = engine.get_node(&cid).unwrap().expect("node exists");
        assert_eq!(fetched, node);
        assert_eq!(fetched.cid().unwrap(), cid);
    }

    #[test]
    fn missing_cid_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
        let cid = canonical_test_node().cid().unwrap();
        assert!(engine.get_node(&cid).unwrap().is_none());
    }
}
