//! SANDBOX subsystem (Phase 2b G7-A).
//!
//! Public surface for SANDBOX primitive evaluation:
//!   - [`manifest`] — D2-RESOLVED hybrid named-manifest registry +
//!     D9-RESOLVED canonical DAG-CBOR encoding.
//!   - [`host_fns`] — D1/D7/D18/D19/D25 host-fn declarations + cap
//!     enforcement layers + trampoline-count contract.
//!   - [`counted_sink`] — D17-RESOLVED defense-in-depth Inv-7 enforcement
//!     (PRIMARY streaming + BACKSTOP return-value paths).
//!   - [`instance`] — D3-RESOLVED + wsa-20 per-call wasmtime lifecycle
//!     (shared `Engine` + content-CID-cached `Module` + per-call `Store`
//!     + `Instance`).
//!
//! The primitive executor lives in `crate::primitives::sandbox` and is
//! re-exported here for convenience. The complete dispatch surface
//! (`sandbox_call` etc.) is wired through `benten-engine` in G7-C; this
//! crate exposes the building blocks.
//!
//! `BudgetExhausted` runtime emission for `sandbox_fuel` /
//! `sandbox_memory` / `sandbox_wallclock` / `sandbox_output` budget_type
//! mirrors G12-A's `inv_8_iteration` pattern at `evaluator.rs:185-192`
//! (per wsa-17 R3 carry). The SANDBOX call site (G7-C engine integration)
//! emits the [`crate::TraceStep::BudgetExhausted`] row BEFORE
//! propagating the typed error; see [`crate::primitives::sandbox::SandboxError::to_budget_exhausted_trace`].
//!
//! ## Compile-time wasm32 disable (sec-pre-r1-05)
//!
//! The whole sandbox subsystem (this module + the executor) is gated
//! `#[cfg(not(target_arch = "wasm32"))]` — on the wasm32 build the
//! symbol is literally absent so the napi sandbox surface (G7-C) cannot
//! be called. Compromise #4 closure narrative.

#![cfg(not(target_arch = "wasm32"))]

pub mod counted_sink;
pub mod host_fns;
pub mod instance;
pub mod manifest;

pub use counted_sink::{CountedSink, OverflowPath, SinkOverflow};
pub use host_fns::{
    CapAllowlist, CapRecheckPolicy, HostFnBehavior, HostFnContext, HostFnReturn, HostFnSpec,
    RESERVED_HOST_ASYNC_CAP, default_host_fns, host_fn_names,
};
pub use manifest::{
    CapBundle, ManifestError, ManifestRef, ManifestRegistry, ManifestSignature,
    default_manifest_names, default_manifests,
};

// Convenience re-export — the executor surface lives in
// `crate::primitives::sandbox` to mirror the existing per-primitive
// module organization, but downstream consumers (engine integration,
// tests) will mostly want `crate::sandbox::Sandbox` shorthand.
pub use crate::primitives::sandbox as primitives_sandbox;
pub use crate::primitives::sandbox::{
    SandboxConfig, SandboxError, SandboxResult, WALLCLOCK_DEFAULT_MS, WALLCLOCK_MAX_MS, execute,
    resolve_priority,
};
