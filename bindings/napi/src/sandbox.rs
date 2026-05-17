//! Phase 2b G7-C — napi bridge for SANDBOX-related introspection +
//! diagnostic surfaces.
//!
//! ## Surface scope (per Phase-2b plan §3 G7-C row)
//!
//! G7-C OWNS:
//! - `sandbox_target_supported()` — boolean introspection probe so a TS
//!   caller can decide whether to drive a SANDBOX call locally vs route
//!   to a Node-resident peer (Phase-3 P2P sync).
//! - `describe_sandbox_node(node_cid)` — read-only diagnostic accessor
//!   returning the resolved `SandboxNodeDescription` for a registered
//!   SANDBOX node. Cfg-gated (sec-r6r2-02 discipline) — present only
//!   when the underlying engine crate compiles with the `test-helpers`
//!   feature on.
//!
//! G10-B owns (NOT G7-C — wsa-r1-5 plan-internal conflict resolution):
//! - `sandbox_install_manifest(manifest, expected_cid)` — manifest
//!   install lifecycle.
//! - `sandbox_uninstall_manifest(cid)` — manifest uninstall lifecycle.
//! - `sandbox_compute_manifest_cid(manifest)` — manifest canonical-CID
//!   computation helper.
//!
//! G10-B may add the install/uninstall/compute napi entries to this
//! same file later, or carve them into a sibling `manifest.rs`. The
//! file structure is open; the OWNERSHIP boundary is closed.
//!
//! ## Compile-time gating discipline (sec-pre-r1-05)
//!
//! Per Phase-2b plan §3 G7-C row + sec-pre-r1-05: the production
//! SANDBOX surface (`sandbox_target_supported` returning `true`) is
//! `#[cfg(not(target_arch = "wasm32"))]`-gated. On wasm32 the
//! complementary `#[cfg(target_arch = "wasm32")]` arm answers `false`
//! so a TS caller doing `if (engine.targetSupportsSandbox()) { ... }`
//! sees the correct platform answer without the engine ever loading
//! the (compile-time absent) wasmtime executor.
//!
//! The `describe_sandbox_node` surface is similarly cfg-split: on wasm32
//! it surfaces the typed `E_SANDBOX_UNAVAILABLE_ON_WASM` error with the
//! wsa-14 actionable text from `docs/SANDBOX-LIMITS.md` §5.
//!
//! ## Why both halves of the cfg-gate ship
//!
//! Defence-in-depth against the symbol-presence-vs-symbol-behaviour
//! confusion. A bare `#[cfg(not(target_arch = "wasm32"))]` on the
//! single function would make `sandboxTargetSupported` literally absent
//! on a wasm32-built napi cdylib, which means a TS caller doing
//! `typeof native.sandboxTargetSupported` sees `"undefined"` rather
//! than the documented `"function"`. Cross-platform code that probes
//! for the symbol then has to special-case the absence. By providing a
//! complementary wasm32 stub that returns `false`, we keep the symbol
//! present (callers always see a function) while answering the
//! platform-availability question correctly.

#![cfg(feature = "napi-export")]

use napi::bindgen_prelude::*;
use napi_derive::napi;

// ---------------------------------------------------------------------------
// `sandbox_target_supported` — boolean introspection probe
// ---------------------------------------------------------------------------

/// Returns `true` when this napi build supports SANDBOX execution
/// locally (i.e. the wasmtime executor is compiled in), `false` when
/// the build is `wasm32-unknown-unknown` and SANDBOX execution must
/// route to a Node-resident peer (Phase-3 P2P sync).
///
/// Mirrors the Rust `cfg(not(target_arch = "wasm32"))` gate on the
/// engine's `execute_sandbox_*` plumbing. Pinned by
/// `bindings/napi/test/sandbox_napi_bridge.test.ts::"sandboxTargetSupported() returns true on Node target builds"`.
#[cfg(not(target_arch = "wasm32"))]
#[napi(js_name = "sandboxTargetSupported")]
pub fn sandbox_target_supported() -> bool {
    // Native target — wasmtime is compiled in via the engine's
    // `benten-eval` dependency. Returning `true` here is the canonical
    // signal that `engine.call(...)` against a SANDBOX-bearing handler
    // executes the wasm guest locally.
    true
}

/// wasm32 target — SANDBOX executor is compile-time absent. Returning
/// `false` lets TS callers decide whether to route the SANDBOX-bearing
/// call to a Node peer (Phase-3 P2P sync) instead of attempting local
/// execution that would surface `E_SANDBOX_UNAVAILABLE_ON_WASM`.
///
/// Pinned by
/// `bindings/napi/test/sandbox_napi_bridge.test.ts::"sandbox-disabled wasm32 builds surface E_SANDBOX_UNAVAILABLE_ON_WASM"`.
#[cfg(target_arch = "wasm32")]
#[napi(js_name = "sandboxTargetSupported")]
pub fn sandbox_target_supported() -> bool {
    false
}

// ---------------------------------------------------------------------------
// `describe_sandbox_node` — diagnostic accessor (ts-r4-3)
// ---------------------------------------------------------------------------
//
// METRIC-PROPAGATION STATE (Phase-3 §7.1 closure — fully wired
// end-to-end; R5 wave-7 G19-C2 + R6 fp Wave C2 obs-r6r1-1 + final
// "unknown" → `null` sentinel-drop at this wave):
//
// The native napi bridge for `describeSandboxNode` lives in `lib.rs`
// alongside the `EngineNapi` impl block (search:
// `describe_sandbox_node_napi`). It is cfg-gated under the engine
// crate's `test-helpers` feature (sec-r6r2-02 discipline) and:
//
// 1. Calls `Engine::describe_sandbox_node_for_handler(&handler_id)`
//    which reads from `EngineInner::sandbox_metrics`.
// 2. The metrics map is populated by `primitive_host::execute_sandbox`
//    on each successful invocation — `SandboxResult.fuel_consumed` +
//    `SandboxResult.output_consumed` + wall-clock duration are
//    captured into a `SandboxNodeMetrics` observation + merged
//    monotonically via `EngineInner::record_sandbox_metric`.
// 3. Serializes the resulting `SandboxNodeDescription` to a JSON
//    template the TS wrapper parses back into the typed shape.
// 4. Returns `Ok(None)` (-> JS `null`) when no SANDBOX invocation
//    has been recorded yet for the handler; the TS wrapper surfaces
//    `null` for each metric field in that case (was `"unknown"`
//    string sentinel pre-§7.1-closure; replaced by structural
//    `number | null` per the cross-language rule-mirror discipline
//    §3.5g).
//
// Cross-ref: `docs/SECURITY-POSTURE.md` Compromise #17 (separate
// concern — durable module-bytes registry, CLOSED at G14-C wave-4b).
// The metric-propagation gap was a SIBLING of Compromise #17 (same
// reinforcement narrative pre-closure); both are closed.
//
// Cross-ref: the metric-propagation gap is tracked at
// `docs/future/phase-3-backlog.md` (SnapshotBlobBackend
// metric-propagation entry, named two paragraphs above) — NOT a
// `docs/SECURITY-POSTURE.md` named compromise. (Compromise #17 is
// "in-memory module-bytes registry", an unrelated surface — the
// prior cite here was a cross-ref error flagged by the X10
// compromise-registry reviewer.)
//
// Visible TS contract today: the `sandbox_target_supported` probe
// above + the typed `describeSandboxNode` accessor. Each metric
// field is `number | null` — `null` means "node not yet invoked"
// (no SANDBOX observation recorded for the handler). The legacy
// `"unknown"` string sentinel was dropped at this wave per the
// cross-language rule-mirror discipline §3.5g (§7.1 closure).
//
// Load-bearing pins:
// - Rust side (engine production-runtime path):
//   `crates/benten-engine/tests/sandbox_metrics.rs::sandbox_node_metrics_high_water_tracker_round_trip`
// - napi side (source-cite diagnostic):
//   `bindings/napi/tests/describe_sandbox.rs::describe_sandbox_node_napi_returns_real_metric_values_not_unknown_placeholder`
// - TS side (vitest end-to-end closure-pin per pim-2 §3.6b):
//   `packages/engine/test/sandbox.test.ts::"describeSandboxNode returns real numeric metrics after invocation (§7.1 closure)"`
