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
// IMPLEMENTATION STATE (post mini-review fix F-G7C-MR-CR-2):
//
// The TS-side `engine.describeSandboxNode(handlerId, nodeId)` is currently
// SYNTHESIZED CLIENT-SIDE in `packages/engine/src/sandbox.ts` — it does
// NOT call into a native napi method. The TS function returns a typed
// `SandboxNodeDescription` shape constructed from the spec / handler-id
// inputs alone, sufficient for the ts-r4-3 type-shape pin
// (`describe_sandbox_node_returns_diagnostic_shape`) but NOT bridging
// real native runtime introspection state (fuel-consumed-high-water,
// last-invocation-ms, etc.).
//
// The native bridge (a `#[napi] fn describe_sandbox_node` on
// `napi_surface::Engine` reaching `benten_engine::Engine::describe_sandbox_node`)
// is intentionally NOT yet wired here. It WILL land alongside G7-A's
// executor body — the runtime introspection state requires the executor's
// per-call `Store` lifecycle (D3-RESOLVED) to be live. Until then, the
// client-side synthesis is the contract.
//
// When the native bridge lands: it will be cfg-gated behind the engine
// crate's `test-helpers` feature per sec-r6r2-02 discipline (the napi
// cdylib opts into the narrower `envelope-cache-test-grade` feature, so
// the accessor is NOT exposed to TS by default — devtools that need real
// runtime state require an explicit feature opt-in). The visible TS
// contract (the `sandbox_napi_bridge.test.ts` symbol-presence pin) is the
// `sandbox_target_supported` probe above + the client-synthesized
// `describeSandboxNode` for shape-pinning.
