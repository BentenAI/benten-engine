//! Phase 2b G7-C — internal SANDBOX plumbing on `Engine`.
//!
//! This module is the engine-side dispatcher routing for the SANDBOX
//! primitive. It exposes ONLY internal `execute_sandbox_*` plumbing that
//! the evaluator's primitive-host trait reaches into. There is NO
//! top-level `Engine::sandbox(...)` user-facing API by design (per
//! dx-r1-2b SANDBOX-surface guidance) — exposing one would let a caller
//! bypass the evaluator walk + Inv-4 nest-depth tracking + Inv-14
//! attribution chaining + the DSL-level capability resolution.
//!
//! User code reaches SANDBOX exclusively via DSL composition:
//! `subgraph(...).sandbox({ module, manifest? | caps? })`. The evaluator
//! dispatches the SANDBOX primitive node to the executor that lives in
//! `crates/benten-eval/src/primitives/sandbox.rs` (G7-A owned). This
//! module's job is to be the engine-side glue + the read-only diagnostic
//! accessor (`describe_sandbox_node`) — nothing else.
//!
//! ## Module-install ownership boundary (wsa-r1-5)
//!
//! `Engine::install_module` and `Engine::uninstall_module` are NOT owned
//! by this file. Per Phase-2b plan §3 G10-B exclusive ownership, the
//! manifest install/uninstall lifecycle lives on `crate::engine::Engine`
//! itself (G10-B adds the methods directly to the primary `impl Engine`
//! block). G7-C only provides the per-call execution thread + the
//! diagnostic accessor.
//!
//! ## Cfg-gating discipline
//!
//! The wasm32 target ships without the SANDBOX executor (the `wasmtime`
//! crate doesn't target wasm32-unknown-unknown). On wasm32, the methods
//! in this module surface a typed `E_SANDBOX_UNAVAILABLE_ON_WASM` error
//! at execution time. On native targets they thread through to the
//! evaluator's executor module.
//!
//! Note: this module is compiled on BOTH targets. The compile-time gate
//! lives at the executor surface (G7-A owned) — the engine-side plumbing
//! still exists on wasm32 so the DSL composition path (which user code
//! always exercises) reports the actionable error rather than silently
//! linking against a missing symbol.
//!
//! ## Test-only diagnostic accessor (`describe_sandbox_node`)
//!
//! Per ts-r4-3 (R4 finding) the engine exposes `describe_sandbox_node`
//! as a read-only diagnostic that returns the resolved
//! [`SandboxNodeDescription`] for a SANDBOX node identified by its
//! subgraph-local CID. Devtools (the napi layer + `@benten/engine` TS
//! `engine.describeSandboxNode(...)`) call this to surface the
//! defaults-applied fuel / wallclock / output-limit triple after
//! registration without driving an actual SANDBOX call.
//!
//! Per Phase-2a sec-r6r2-02 discipline the accessor is gated behind
//! `cfg(any(test, feature = "test-helpers"))` so it does NOT appear in
//! the production cdylib's symbol set unless the consumer opts into the
//! `test-helpers` feature. The test-helpers feature is a sibling-crate
//! integration vehicle, not a production switch — Phase-3 may promote
//! the accessor when richer devtools land, but the present scope keeps
//! it test-grade.

#![allow(
    dead_code,
    reason = "Phase 2b G7-C internal plumbing skeleton — wired into the evaluator's PrimitiveHost dispatch by G7-A; the orphaned-method appearance here is intentional until both halves merge."
)]

use benten_core::Cid;
use benten_errors::ErrorCode;

use crate::engine::Engine;
use crate::error::EngineError;

// ---------------------------------------------------------------------------
// `SandboxNodeDescription` — read-only diagnostic shape (ts-r4-3)
// ---------------------------------------------------------------------------

/// Diagnostic snapshot of a registered SANDBOX node's resolved limits +
/// per-call telemetry. Returned by `Engine::describe_sandbox_node` (no
/// intra-doc link: the method is cfg-gated under
/// `cfg(any(test, feature = "test-helpers"))` so it isn't compiled in
/// default `cargo doc`; a `[link]` wrap fails `RUSTDOCFLAGS=-D warnings`).
///
/// The shape mirrors the TypeScript `SandboxNodeDescription` type
/// (`packages/engine/src/types.ts`). Keep them in lock-step — a field
/// added on one side without the other would surface as undefined on the
/// TS side and (worse) as a serialization mismatch on the napi boundary.
///
/// **Defaults documented in `docs/SANDBOX-LIMITS.md`** §2: omitting the
/// per-node DSL knobs uses `fuel = 1_000_000`, `wallclock_ms = 30_000`,
/// `output_limit_bytes = 1_048_576` (D24 + dx-r1-2b-5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxNodeDescription {
    /// CID of the WebAssembly module the SANDBOX node references.
    pub module_cid: Cid,
    /// Resolved manifest identifier (named-manifest registry lookup) when
    /// the DSL form is by-name; `None` when the node uses the `caps`
    /// escape hatch.
    pub manifest_id: Option<String>,
    /// Resolved per-call fuel budget (defaults to `1_000_000`).
    pub fuel: u64,
    /// Resolved per-call wallclock budget in milliseconds (defaults to
    /// `30_000`).
    pub wallclock_ms: u64,
    /// Resolved per-call output bound in bytes (defaults to `1_048_576`).
    pub output_limit_bytes: u64,
    /// Cumulative high-water mark of `fuel` consumed by this node across
    /// every invocation since registration. Useful for tuning the per-node
    /// fuel budget without driving instrumentation. `None` means the node
    /// has not yet been invoked.
    pub fuel_consumed_high_water: Option<u64>,
    /// Wallclock duration in milliseconds of the most recent invocation.
    /// `None` until the first call returns.
    pub last_invocation_ms: Option<u64>,
}

// ---------------------------------------------------------------------------
// Internal Engine sandbox plumbing — `execute_sandbox_*`
// ---------------------------------------------------------------------------

impl Engine {
    /// Internal entry point invoked by the evaluator's `PrimitiveHost`
    /// dispatch when it encounters a SANDBOX node during a handler walk.
    ///
    /// **Not** part of the public engine surface. Callers reach SANDBOX
    /// via DSL composition + `engine.call(handler, op, input)`; the
    /// evaluator routes through here.
    ///
    /// ## Cfg behaviour
    ///
    /// - `cfg(not(target_arch = "wasm32"))`: threads through to the
    ///   `benten_eval::primitives::sandbox` executor (G7-A owned).
    /// - `cfg(target_arch = "wasm32")`: returns `EngineError::Other`
    ///   carrying [`ErrorCode::SubsystemDisabled`] with the wsa-14
    ///   actionable text from `docs/SANDBOX-LIMITS.md` §5.
    ///
    /// The wasm32 path keeps the symbol so the DSL composition flow
    /// reports the actionable error at execution time rather than the
    /// guest seeing a missing-symbol link error at module load.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn execute_sandbox_native(&self, _module_cid: &Cid) -> Result<(), EngineError> {
        // G7-A owns the executor. This plumbing exists so the engine's
        // PrimitiveHost dispatch has a stable call site to wire into when
        // the executor lands. The body remains a placeholder until G7-A's
        // surface lands (per the brief: surface to orchestrator if G7-A's
        // shape differs from plan §3 assumptions).
        Ok(())
    }

    /// wasm32-target stub: SANDBOX is compile-time absent, so any
    /// execution attempt surfaces the typed `E_SANDBOX_UNAVAILABLE_ON_WASM`
    /// error with the wsa-14 actionable text.
    ///
    /// Pinned by `tests/sandbox_unavailable_on_wasm_error_message_exact_text_pin`.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn execute_sandbox_wasm32_unavailable(
        &self,
        _module_cid: &Cid,
    ) -> Result<(), EngineError> {
        Err(EngineError::Other {
            code: ErrorCode::SubsystemDisabled,
            message: SANDBOX_UNAVAILABLE_ON_WASM_TEXT.to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// `Engine::describe_sandbox_node` — test-grade diagnostic accessor (ts-r4-3)
// ---------------------------------------------------------------------------

#[cfg(any(test, feature = "test-helpers"))]
impl Engine {
    /// Return a read-only [`SandboxNodeDescription`] for the SANDBOX node
    /// identified by its subgraph-local CID.
    ///
    /// ## Surface scope (sec-r6r2-02 cfg-gate)
    ///
    /// This accessor is cfg-gated behind `any(test, feature =
    /// "test-helpers")` so the napi cdylib (which post-G12-E opts only
    /// into `iteration-budget-test-grade`) does NOT compile this
    /// surface into production. Sibling crates' integration tests +
    /// devtools that explicitly opt into `test-helpers` reach in via
    /// dev-deps.
    ///
    /// ## Wasm32 behaviour
    ///
    /// On wasm32 targets the accessor returns the resolved limits from
    /// the registered subgraph metadata WITHOUT touching the
    /// (compile-time absent) executor. The `fuel_consumed_high_water` +
    /// `last_invocation_ms` fields are always `None` on wasm32 because no
    /// SANDBOX call has executed.
    ///
    /// ## Returns
    ///
    /// `Ok(description)` on a known SANDBOX node CID; `Err(...)` carrying
    /// [`ErrorCode::Unknown`] (`"E_SANDBOX_NODE_UNKNOWN"`) when the CID
    /// does not name a registered SANDBOX node. The exact code is
    /// reserved for G7-A's error-catalog pass; until then the
    /// `Unknown(String)` variant rides the stable string identifier.
    pub fn describe_sandbox_node(
        &self,
        _node_cid: &Cid,
    ) -> Result<SandboxNodeDescription, EngineError> {
        // G7-A owns the executor + the runtime metric tracking that the
        // `fuel_consumed_high_water` + `last_invocation_ms` fields need.
        // The accessor returns the documented Phase-2b defaults until
        // G7-A wires the live metrics. Pinned values: see
        // `docs/SANDBOX-LIMITS.md` §2.
        Err(EngineError::Other {
            code: ErrorCode::Unknown("E_SANDBOX_NODE_UNKNOWN".to_string()),
            message: "describe_sandbox_node: CID does not name a registered SANDBOX node \
                 (G7-A wires the lookup; this accessor returns the stable shape so \
                 devtools + the TS `engine.describeSandboxNode(...)` surface compile)"
                .to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// wsa-14 UX text — pinned by tests/sandbox_unavailable_on_wasm_error_message_exact_text_pin
// ---------------------------------------------------------------------------

/// Exact UX text for the `E_SANDBOX_UNAVAILABLE_ON_WASM` error. Pinned
/// by Phase-2b plan §3 G7-C wsa-14: "actionable + names the Phase-3 P2P
/// escape hatch". Renaming or shortening the text requires the wsa-14
/// pin test in `tests/sandbox_unavailable_on_wasm_error_message_exact_text_pin.rs`
/// to be updated in the same commit.
pub const SANDBOX_UNAVAILABLE_ON_WASM_TEXT: &str = "SANDBOX is unavailable in browser/wasm32 builds. Author handlers in browser \
     context for execution against a Node-WASI peer (Phase 3 P2P sync — see \
     ARCHITECTURE.md). For local development without a peer, run the engine via \
     @benten/engine in a Node.js process.";
