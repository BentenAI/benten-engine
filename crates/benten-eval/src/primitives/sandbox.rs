//! SANDBOX primitive executor (Phase 2b G7-A).
//!
//! Wires the four enforcement axes against a wasmtime instance constructed
//! per [`crate::sandbox::instance`]. Per D21-RESOLVED severity priority,
//! when multiple axes trip simultaneously the highest-priority axis wins:
//!
//! ```text
//! MEMORY > WALLCLOCK > FUEL > OUTPUT
//! ```
//!
//! Documented in `docs/SANDBOX-LIMITS.md` (G7-C-owned). Catalog rows for
//! each error code mirror the priority text per `sandbox_priority_order_documented_in_catalog`.
//!
//! D24-RESOLVED defaults: 30s wallclock default / 5min ceiling.
//! Per-handler `wallclock_ms` opt-in via `SubgraphSpec.primitives` (G12-D
//! widening; the SANDBOX primitive reads the per-call value through
//! [`SandboxConfig::wallclock_ms`]).
//!
//! `BudgetExhausted` runtime emission for `sandbox_fuel` budget_type
//! (wsa-17 R3 carry): when fuel exhaustion fires, the executor emits
//! `TraceStep::BudgetExhausted { budget_type: "sandbox_fuel", consumed,
//! limit, path }` BEFORE propagating the typed error, mirroring G12-A's
//! `inv_8_iteration` budget-exhaustion arm inside `evaluator.rs::run_inner`. Same for
//! `sandbox_memory`, `sandbox_wallclock`, `sandbox_output` budget types.
//!
//! This module is `#[cfg(not(target_arch = "wasm32"))]`-gated per
//! sec-pre-r1-05; the wasm32 build cuts SANDBOX entirely.

#![cfg(not(target_arch = "wasm32"))]

use crate::sandbox::counted_sink::{CountedSink, OverflowPath, SinkOverflow};
use crate::sandbox::epoch_ticker::{epoch_ticks_for_ms, spawn_epoch_ticker};
use crate::sandbox::escape_defenses::{EscDefenseState, EscVector, run_all_checks};
use crate::sandbox::fingerprint::{
    WallclockTaintedAddress, read_collapse_state, record_wallclock_write,
};
use crate::sandbox::host_fns::{CapAllowlist, HostFnBehavior, HostFnSpec, default_host_fns};
use crate::sandbox::manifest::{ManifestRef, ManifestRegistry};
use crate::sandbox::resource_limiter::SandboxResourceLimiter;
use crate::sandbox::trap_to_typed::{
    EscapeAttemptMarker, HostFnDenialKind, HostFnDenialMarker, map_call_error,
};
use crate::{AttributionFrame, TraceStep};
use benten_errors::ErrorCode;
use std::collections::BTreeMap;
use std::sync::Arc;
use wasmtime::{Caller, Engine, Linker, Store};

/// Per-call SANDBOX configuration. Caller-overrides go on top of
/// [`SandboxConfig::default`]; per-handler overrides come through
/// `SubgraphSpec.primitives` (G12-D widening).
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Per-call fuel budget (wasmtime units). dx-r1-2b-5 default 1_000_000.
    pub fuel: u64,
    /// Per-call memory limit in bytes. Default 64 MiB candidate (the
    /// memory-axis test pins 64 MiB; future revisions may tune).
    pub memory_bytes: u64,
    /// Per-call wallclock deadline in milliseconds. D24-RESOLVED default
    /// 30s; ceiling 5min (clamped via [`Self::with_wallclock_ms`]).
    pub wallclock_ms: u64,
    /// Per-call cumulative output-byte budget (D17 PRIMARY + BACKSTOP).
    /// Default 1 MiB.
    pub output_bytes: u64,
    /// D20 — max sandbox_depth for THIS call. Inherited from the dispatcher
    /// via [`crate::AttributionFrame`] (when wired). Default 4 per the D20
    /// "safety + audibility" tradeoff.
    pub max_nest_depth: u8,
    /// Phase-3 G17-A1 wave-5b — max wasmtime guest stack size in bytes
    /// per phase-3-backlog §6.4 + r1-wsa-7 BLOCKER closure. Default
    /// 512 KiB (matches wasmtime default). Surfaced through the typed
    /// [`SandboxError::StackOverflow`] variant when exceeded; carried
    /// in the variant payload so operator dashboards distinguish a
    /// recursive-runaway guest from a generic invalid module.
    pub max_wasm_stack: u64,
    /// **Test-only attack-pattern injection seam (cfg-gated; G20-A1
    /// wave-8a unwidens further per `tests/sandbox_helpers_no_widening.rs`).**
    /// Phase-3 wave-5c §6.1-followup tasks #3 + #4: end-to-end test
    /// pins drive ESC-7 (`Esc7FuelRefillViaReEntry`) + ESC-13
    /// (`Esc13StorePoison`) detection from inside the `time` host-fn
    /// trampoline. Production callsites pass `None` (the field
    /// defaults to `TestEscAttackInjection::None`); test callsites
    /// pass an explicit attack-pattern enum. The host-fn trampoline
    /// observes the requested attack-pattern, mutates the per-call
    /// `EscDefenseState` accordingly, then [`run_all_checks`] fires
    /// the typed `EscapeAttempt` variant — exercising the SAME runtime
    /// arm a hypothetical real attack pattern would trigger.
    ///
    /// **Why a test-only seam not a real attack:** ESC-7's canonical
    /// attack pattern requires a host-fn that calls
    /// `Store::add_fuel` mid-execution — no such host-fn exists in
    /// the production surface (and CLAUDE.md baked-in #16 forbids
    /// adding one). ESC-13's canonical attack requires a wasmtime
    /// fuel-meter callback that panics — wasmtime 43.x offers no
    /// stable user-pluggable callback to insert one. The test seam
    /// drives the per-call state shape that BOTH attacks would
    /// produce, exercising the typed-error routing end-to-end so the
    /// runtime arm is verified-active rather than verified-inert.
    /// The seam is feature-gated; the production cdylib never
    /// compiles the field.
    #[cfg(any(test, feature = "test-helpers", feature = "testing"))]
    pub testing_inject_attack: TestEscAttackInjection,
}

/// Phase-3 wave-5c §6.1-followup tasks #3 + #4 — test-only attack
/// injection at the host-fn trampoline boundary. See
/// [`SandboxConfig::testing_inject_attack`].
#[cfg(any(test, feature = "test-helpers", feature = "testing"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TestEscAttackInjection {
    /// No attack pattern; production-equivalent path.
    #[default]
    None,
    /// Drive ESC-7 detection: the `time` host-fn trampoline marks
    /// `re_entry_count = 1` while `guest_active = true`; the boundary
    /// `run_all_checks` invocation fires
    /// `EscapeAttempt(Esc7FuelRefillViaReEntry)`.
    Esc7ReEntryAttempt,
    /// Drive ESC-13 detection: the `time` host-fn trampoline panics
    /// (simulating a fuel-meter callback panic); the
    /// `std::panic::catch_unwind` wrapper around `func.call` in
    /// [`execute_with_live_cap_check`] catches it, sets
    /// `fuel_meter_callback_trapped = true`, and surfaces typed
    /// `EscapeAttempt(Esc13StorePoison)`.
    Esc13FuelMeterCallbackTrap,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            fuel: 1_000_000,
            memory_bytes: 64 * 1024 * 1024,
            wallclock_ms: WALLCLOCK_DEFAULT_MS,
            output_bytes: 1024 * 1024,
            max_nest_depth: 4,
            max_wasm_stack: MAX_WASM_STACK_DEFAULT,
            #[cfg(any(test, feature = "test-helpers", feature = "testing"))]
            testing_inject_attack: TestEscAttackInjection::None,
        }
    }
}

impl SandboxConfig {
    // cr-g7a-mr-8 fix-pass: dropped the duplicate `SandboxConfig::WALLCLOCK_DEFAULT_MS`
    // and `SandboxConfig::WALLCLOCK_MAX_MS` associated constants. The
    // module-level `WALLCLOCK_DEFAULT_MS` / `WALLCLOCK_MAX_MS` consts
    // (re-exported via `crate::sandbox`) are the canonical surface;
    // `engine_config.rs::WALLCLOCK_*` re-exports them. Two-name discoverability
    // is a footgun per CLAUDE.md non-negotiable rule 5 ("no deprecated aliases").

    /// Apply a per-handler `wallclock_ms` override. **wsa-g7a-mr-3 +
    /// sec-g7a-mr-6 fix-pass:** semantics are REJECT (fail-loud), NOT
    /// CLAMP. The brief language "ceiling clamps per-handler if exceeded"
    /// was casual-shorthand; the more secure default is fail-loud so a
    /// mis-configured handler learns at validate-time rather than
    /// running silently with a tighter-than-intended budget. Returns
    /// [`ErrorCode::SandboxWallclockInvalid`] when `ms == 0` or
    /// `ms > WALLCLOCK_MAX_MS`.
    ///
    /// # Errors
    /// Returns [`ErrorCode::SandboxWallclockInvalid`] when `ms == 0` or
    /// `ms > 5min`.
    pub fn with_wallclock_ms(mut self, ms: u64) -> Result<Self, ErrorCode> {
        if ms == 0 {
            return Err(ErrorCode::SandboxWallclockInvalid);
        }
        if ms > WALLCLOCK_MAX_MS {
            return Err(ErrorCode::SandboxWallclockInvalid);
        }
        self.wallclock_ms = ms;
        Ok(self)
    }
}

/// D24 default wallclock (30s). Public so the `EngineConfig`
/// precedence layer in `benten-engine` (which re-exports this constant)
/// can name the same constant. cr-g7a-mr-8 fix-pass dropped the prior
/// `SandboxConfig::WALLCLOCK_DEFAULT_MS` associated-const re-export.
pub const WALLCLOCK_DEFAULT_MS: u64 = 30_000;
/// D24 ceiling (5min).
pub const WALLCLOCK_MAX_MS: u64 = 5 * 60_000;
/// Phase-3 G17-A1 wave-5b — default wasmtime guest stack size (512 KiB).
/// Matches wasmtime's `Config::max_wasm_stack` default. Per
/// phase-3-backlog §6.4 + r1-wsa-7 BLOCKER closure: stack-overflow
/// traps route to a dedicated [`SandboxError::StackOverflow`] typed
/// variant (catalog code `E_SANDBOX_STACK_OVERFLOW`).
pub const MAX_WASM_STACK_DEFAULT: u64 = 512 * 1024;

/// Result of a single SANDBOX primitive execution.
#[derive(Debug, Clone)]
pub struct SandboxResult {
    /// Output bytes the module returned (the primitive's `output` value).
    pub output: Vec<u8>,
    /// Fuel consumed (diagnostic for cold-start budget tests + bench
    /// reports).
    pub fuel_consumed: u64,
    /// Cumulative output bytes accounted via [`CountedSink`] (PRIMARY
    /// + BACKSTOP combined).
    pub output_consumed: u64,
}

/// Failure modes for SANDBOX execution.
///
/// Maps to [`ErrorCode`] via [`Self::code`]. Per D21 priority, when
/// multiple axes are eligible at a single trap-callback frame the
/// highest-priority axis is selected.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum SandboxError {
    /// D21 priority 1 — memory exhaustion (most catastrophic; matches
    /// OS-level OOM trump).
    #[error("SANDBOX memory limit exhausted: {limit} bytes")]
    MemoryExhausted {
        /// Configured per-call cap.
        limit: u64,
    },
    /// D21 priority 2 — wallclock deadline exceeded.
    #[error("SANDBOX wallclock deadline exceeded: {limit_ms} ms")]
    WallclockExceeded {
        /// Configured per-call deadline (ms).
        limit_ms: u64,
    },
    /// D21 priority 3 — fuel exhaustion (CPU-bound runaway).
    #[error("SANDBOX fuel exhausted: limit={limit} consumed={consumed}")]
    FuelExhausted {
        /// Fuel consumed before firing.
        consumed: u64,
        /// Configured per-call fuel budget.
        limit: u64,
    },
    /// D21 priority 4 — output overflow.
    #[error(transparent)]
    OutputOverflow(#[from] SinkOverflow),
    /// D19 — host-fn cap denial routes typed error here (NOT a wasmtime
    /// trap per sec-r1 D7).
    #[error("SANDBOX host-fn capability denied: {cap}")]
    HostFnDenied {
        /// Cap-string the call required.
        cap: String,
    },
    /// Module references a host-fn name not in the active manifest.
    #[error("SANDBOX host-fn not found: {name}")]
    HostFnNotFound {
        /// The unrecognized host-fn name.
        name: String,
    },
    /// Manifest lookup failure (ESC-15).
    #[error("SANDBOX manifest unknown: {name}")]
    ManifestUnknown {
        /// The unrecognized manifest name.
        name: String,
    },
    /// Module bytes failed wasmtime structural validation.
    #[error("SANDBOX module invalid: {reason}")]
    ModuleInvalid {
        /// Human-readable reason.
        reason: String,
    },
    /// D19 — nested-dispatch attempt by host-fn callback denied.
    #[error("SANDBOX nested dispatch denied")]
    NestedDispatchDenied,
    /// D20 — `AttributionFrame.sandbox_depth` saturating-counter overflow.
    #[error("SANDBOX nested dispatch depth exceeded: max={max}")]
    NestedDispatchDepthExceeded {
        /// Configured max-nest depth (default 4).
        max: u8,
    },
    /// **cr-g7a-mr-6 fix-pass:** manifest canonical-bytes encode
    /// failure surfaces with type fidelity through this variant rather
    /// than collapsing into [`SandboxError::ModuleInvalid`] (which is
    /// reserved for wasmtime-side structural validation failures, NOT
    /// manifest-encode failures). Routes to [`ErrorCode::Serialize`].
    #[error("SANDBOX manifest canonical-bytes encode failure: {reason}")]
    ManifestEncodeFailed {
        /// Human-readable reason from the encoder.
        reason: String,
    },
    /// Wave-8d-types: a SANDBOX dispatch named a module CID that has
    /// no bytes registered through `Engine::register_module_bytes`.
    /// Distinct from [`SandboxError::ModuleInvalid`] (bytes present
    /// but failed wasmtime structural validation): this fires BEFORE
    /// the executor sees any bytes, at the engine's lookup step.
    /// Routes to [`ErrorCode::SandboxModuleNotInstalled`].
    #[error("SANDBOX module bytes not registered for CID {0}")]
    ModuleNotInstalled(benten_core::Cid),
    /// Phase-3 G17-A1 wave-5b — SANDBOX guest module's call stack
    /// exceeded the configured `max_wasm_stack` ceiling (wasmtime
    /// default 512 KiB). Distinct from [`SandboxError::FuelExhausted`]
    /// (CPU-bound runaway) and [`SandboxError::ModuleInvalid`]
    /// (structural validation failure). Routes to
    /// [`ErrorCode::SandboxStackOverflow`] per phase-3-backlog §6.4 +
    /// r1-wsa-7 BLOCKER closure. The trap routing lives at
    /// [`crate::sandbox::trap_to_typed::map_call_error`] (the
    /// `wasmtime::Trap::StackOverflow` arm).
    #[error("SANDBOX stack overflow: guest exceeded max_wasm_stack ({max_wasm_stack} bytes)")]
    StackOverflow {
        /// Configured `max_wasm_stack` budget (default 512 KiB).
        max_wasm_stack: u64,
    },
    /// Phase-3 G17-A1 wave-5b — SANDBOX guest attempted one of the
    /// enumerated escape vectors (ESC-7 / ESC-13 / ESC-16 currently;
    /// extensible per [`crate::sandbox::escape_defenses::EscVector`]).
    /// The defense at
    /// [`crate::sandbox::escape_defenses`] fires this typed variant
    /// rather than collapsing into [`SandboxError::ModuleInvalid`] so
    /// audit pipelines can route per-vector. Routes to
    /// [`ErrorCode::SandboxEscapeAttempt`].
    ///
    /// Defends r1-wsa-1 BLOCKER (ESC-7 + ESC-13) + r1-wsa-4 (ESC-16)
    /// per phase-3-backlog §6.1 + D-E (R1 revision triage).
    #[error("SANDBOX escape attempt detected: {vector:?} — {reason}")]
    EscapeAttempt {
        /// Discriminating ESC vector.
        vector: crate::sandbox::escape_defenses::EscVector,
        /// Operator-actionable reason string.
        reason: String,
    },
}

impl SandboxError {
    /// Stable catalog code for routing.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            SandboxError::MemoryExhausted { .. } => ErrorCode::SandboxMemoryExhausted,
            SandboxError::WallclockExceeded { .. } => ErrorCode::SandboxWallclockExceeded,
            SandboxError::FuelExhausted { .. } => ErrorCode::SandboxFuelExhausted,
            SandboxError::OutputOverflow(o) => o.code(),
            SandboxError::HostFnDenied { .. } => ErrorCode::SandboxHostFnDenied,
            SandboxError::HostFnNotFound { .. } => ErrorCode::SandboxHostFnNotFound,
            SandboxError::ManifestUnknown { .. } => ErrorCode::SandboxManifestUnknown,
            SandboxError::ModuleInvalid { .. } => ErrorCode::SandboxModuleInvalid,
            SandboxError::NestedDispatchDenied => ErrorCode::SandboxNestedDispatchDenied,
            SandboxError::NestedDispatchDepthExceeded { .. } => {
                ErrorCode::SandboxNestedDispatchDepthExceeded
            }
            SandboxError::ManifestEncodeFailed { .. } => ErrorCode::Serialize,
            SandboxError::ModuleNotInstalled(_) => ErrorCode::SandboxModuleNotInstalled,
            SandboxError::StackOverflow { .. } => ErrorCode::SandboxStackOverflow,
            SandboxError::EscapeAttempt { .. } => ErrorCode::SandboxEscapeAttempt,
        }
    }

    /// `BudgetExhausted` budget_type tag for the budget-axis variants
    /// (wsa-17). `None` for non-budget axes (host-fn denial,
    /// nested-dispatch, manifest, module-invalid).
    #[must_use]
    pub fn budget_type(&self) -> Option<&'static str> {
        match self {
            SandboxError::FuelExhausted { .. } => Some("sandbox_fuel"),
            SandboxError::MemoryExhausted { .. } => Some("sandbox_memory"),
            SandboxError::WallclockExceeded { .. } => Some("sandbox_wallclock"),
            SandboxError::OutputOverflow(_) => Some("sandbox_output"),
            _ => None,
        }
    }

    /// Construct the matching [`TraceStep::BudgetExhausted`] row to
    /// emit BEFORE propagating the typed error (wsa-17, mirrors G12-A's
    /// `inv_8_iteration` budget-exhaustion arm inside `evaluator.rs::run_inner`). Returns
    /// `None` for non-budget axes.
    #[must_use]
    pub fn to_budget_exhausted_trace(&self, path: Vec<String>) -> Option<TraceStep> {
        let budget_type = self.budget_type()?;
        let (consumed, limit) = match self {
            SandboxError::FuelExhausted { consumed, limit } => (*consumed, *limit),
            SandboxError::MemoryExhausted { limit } => (*limit, *limit),
            SandboxError::WallclockExceeded { limit_ms } => (*limit_ms, *limit_ms),
            SandboxError::OutputOverflow(o) => (o.consumed, o.limit),
            _ => return None,
        };
        Some(TraceStep::BudgetExhausted {
            budget_type,
            consumed,
            limit,
            path,
        })
    }
}

/// D21 priority resolver — when multiple axes trip in the same trap
/// frame, return the highest-priority one. The trap-callback path
/// constructs all eligible axis errors and calls this to pick.
///
/// **perf-g7a-mr-6 fix-pass:** takes `Vec<SandboxError>` by value and
/// drains via `into_iter` so non-trivial String-bearing variants do not
/// allocate a clone per resolution (the trap-bounce path already has
/// allocation pressure; saving the per-trap String alloc is a measurable
/// trace-step improvement).
#[must_use]
pub fn resolve_priority(eligible: Vec<SandboxError>) -> Option<SandboxError> {
    // Higher priority value = wins. MEMORY > WALLCLOCK > FUEL > OUTPUT.
    eligible.into_iter().max_by_key(|e| match e {
        SandboxError::MemoryExhausted { .. } => 4,
        SandboxError::WallclockExceeded { .. } => 3,
        SandboxError::FuelExhausted { .. } => 2,
        SandboxError::OutputOverflow(_) => 1,
        _ => 0,
    })
}

/// Phase-3 wave-5c — live cap-recheck callback type. Consulted by the
/// host-fn trampoline at the BEFORE-EVERY-host-fn-invocation cadence
/// (cadence (a) per r1-wsa-3 disposition + r4-r1-wsa-4) for `PerCall`
/// cap-recheck host-fns. Returns `true` iff the dispatching grant
/// CURRENTLY holds the cap-string (a live read, not a snapshot). The
/// engine override at
/// `benten_engine::primitive_host::execute_sandbox` threads an
/// engine-backed callable that consults the engine's revoked-actors
/// set + future grant-store; tests use a closure capturing a mutable
/// state vec for synthetic revoke-mid-call drives.
///
/// Closes phase-3-backlog §6.3 + §6.1-followup task #5 (ESC-9
/// cap-revoke mid-call defense, r1-wsa-3 MAJOR closure).
pub type LiveCapCheck = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// Cap-string prefix that identifies the deferred `random` host-fn
/// (sec-g7a-mr-5 + D1-RESOLVED + sec-pre-r1-06 §2.3). The TOML at
/// workspace-root `host-functions.toml` declares the deferral; this
/// constant lets the executor fail-loud at validate time IF a manifest
/// claims any cap matching this prefix (defensive belt-and-braces while
/// the full module-link-time host-fn-name enumeration lands in G7-C).
pub const DEFERRED_HOST_FN_RANDOM_CAP_PREFIX: &str = "host:compute:random";

/// Execute a SANDBOX primitive call.
///
/// **Phase 2b G7-A scaffold.** The full per-call wasmtime `Store` +
/// `Instance` lifecycle (per D3-RESOLVED + wsa-20) lives in the
/// integration with the engine's `PrimitiveHost`; G7-C wires
/// `Engine::execute_sandbox_*` calling through to this surface.
///
/// This entry point validates the manifest reference, reserves the
/// per-call [`CountedSink`] + read-budget + log-budget state, and
/// returns either:
///   - Ok([`SandboxResult`]) on successful completion.
///   - Err([`SandboxError`]) on any axis trip / cap-denial / module
///     invalidity.
///
/// **sec-g7a-mr-1 fix-pass:** takes the dispatching `attribution`
/// frame as the surface that satisfies sec-pre-r1-03 (host-fn invocation
/// MUST be audit-recorded against the dispatching frame). The G7-C
/// trampoline reads `attribution` off [`crate::sandbox::HostFnContext`]
/// when emitting the host-fn audit row; G7-A stamps the field through
/// `execute()` so the surface compiles + the schema is locked.
///
/// `BudgetExhausted` trace-row emission is the caller's responsibility
/// (the SANDBOX call site that owns the trace buffer). The error's
/// [`SandboxError::to_budget_exhausted_trace`] method constructs the
/// row given the active walk-path.
///
/// # Errors
/// Returns [`SandboxError`] on any axis trip / cap-denial / manifest
/// lookup failure / module-invalidity.
#[allow(clippy::needless_pass_by_value)] // Manifest+config are conceptually owned by the call.
#[allow(clippy::too_many_lines)] // Step-by-step plan is intentionally readable top-to-bottom.
pub fn execute(
    module_bytes: &[u8],
    manifest_ref: ManifestRef,
    registry: &ManifestRegistry,
    config: SandboxConfig,
    grant_caps: &[String],
    attribution: &AttributionFrame,
) -> Result<SandboxResult, SandboxError> {
    execute_with_live_cap_check(
        module_bytes,
        manifest_ref,
        registry,
        config,
        grant_caps,
        attribution,
        None,
    )
}

/// Phase-3 wave-5c — `execute` variant that accepts an optional
/// [`LiveCapCheck`] callback. Threads the callback into
/// `SandboxStoreData` (private; see crate root) so the host-fn
/// trampolines consult it for `PerCall` cap-recheck cadence
/// (D18 — fires BEFORE every host-fn invocation per r1-wsa-3 MAJOR;
/// cadence (a) per r4-r1-wsa-4).
///
/// The legacy [`execute`] entry point delegates here with `None` —
/// preserving the Phase-2b grant-caps-snapshot fallback for callers
/// that have not been migrated to the live-callback shape. The engine
/// override at
/// `benten_engine::primitive_host::execute_sandbox` passes
/// `Some(callback)` so production runs observe live revocation.
///
/// Closes phase-3-backlog §6.1-followup tasks #1, #2, #3, #4, #5
/// (the SandboxStoreData field add + the ESC-7/9/13/16 wiring at
/// the host-fn trampoline boundary).
///
/// # Errors
/// See [`execute`].
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
pub fn execute_with_live_cap_check(
    module_bytes: &[u8],
    manifest_ref: ManifestRef,
    registry: &ManifestRegistry,
    config: SandboxConfig,
    grant_caps: &[String],
    attribution: &AttributionFrame,
    live_cap_check: Option<LiveCapCheck>,
) -> Result<SandboxResult, SandboxError> {
    // 0. R6FP-Group-1 (r6-cr-1 / r6-mpc-4 / r6-wsa-1) — D20 runtime arm
    //    enforcement. The dispatching `AttributionFrame.sandbox_depth`
    //    reflects the cumulative SANDBOX nest count along the active
    //    call chain (depth=1 = top-level SANDBOX entry, depth=2 = the
    //    inner SANDBOX of a SANDBOX→handler→SANDBOX chain, etc.). Pre-
    //    R6FP-G1 the engine-side production override hardcoded
    //    `sandbox_depth: 1` literally at every entry, so the runtime
    //    arm could never fire (depth never grew past 1, and 1 is below
    //    every plausible `max_nest_depth`). Wave-8 "Inv-4 runtime arm
    //    dormant" was the 3-lens convergent finding (code-reviewer +
    //    metadata-producer-vs-consumer + wasmtime-sandbox-auditor); the
    //    fix threads `parent.sandbox_depth + 1` through the engine's
    //    `ActiveCall` so each nested SANDBOX entry observes the correct
    //    depth here.
    //
    //    Semantics: fire when depth > max_nest_depth (a depth equal to
    //    the configured max is the FINAL allowed level; the next call
    //    that would push depth+1 trips). Default max_nest_depth=4 →
    //    depths 1..=4 admitted, depth 5 fires.
    if attribution.sandbox_depth > config.max_nest_depth {
        return Err(SandboxError::NestedDispatchDepthExceeded {
            max: config.max_nest_depth,
        });
    }

    // 1. Resolve the manifest. ESC-15 closure: `Named` lookup either
    //    returns a bundle or fires `SandboxError::ManifestUnknown`.
    //    cr-g7a-mr-6 fix-pass: ManifestError::Encode routes through
    //    SandboxError::ManifestEncodeFailed (Serialize code) rather than
    //    being collapsed into ModuleInvalid (which is wasmtime-side
    //    structural validation only).
    let bundle = manifest_ref.resolve(registry).map_err(|e| match e {
        crate::sandbox::manifest::ManifestError::Unknown { name } => {
            SandboxError::ManifestUnknown { name }
        }
        crate::sandbox::manifest::ManifestError::Encode { reason } => {
            SandboxError::ManifestEncodeFailed { reason }
        }
        crate::sandbox::manifest::ManifestError::RuntimeRegistrationDeferred => {
            SandboxError::ManifestEncodeFailed {
                reason: "RuntimeRegistrationDeferred surfaced from resolve() — \
                                 should not happen against existing registry"
                    .to_string(),
            }
        }
    })?;
    // Take an owned clone so we don't hold a borrow on the registry across
    // the wasmtime invocation; the bundle is small (Vec<String> caps).
    let bundle = bundle.clone();

    // sec-g7a-mr-5 — defensive D1 random-host-fn deferral guard. Until
    // the full module-link-time host-fn enumeration lands in a future
    // wave, fire SandboxHostFnNotFound at validate-time if the manifest
    // claims a `random` cap. Operator-actionable hint encoded in the
    // message (mirrors the host-functions.toml comment).
    for required in &bundle.caps {
        if required.starts_with(DEFERRED_HOST_FN_RANDOM_CAP_PREFIX) {
            return Err(SandboxError::HostFnNotFound {
                name: format!(
                    "random (cap='{required}'): not yet implemented (Phase 3 — see \
                     docs/future/phase-3-backlog.md §6.10 for the workspace CSPRNG \
                     framework choice; original deferral rationale: D1 + sec-pre-r1-06 §2.3)"
                ),
            });
        }
    }

    // 2. D7 init-snapshot intersection — fail loud if the manifest
    //    claims caps the dispatching grant lacks. sec-g7a-mr-4 +
    //    perf-g7a-mr-8 fix-pass: delegate to CapAllowlist::intersect.
    let allowlist = CapAllowlist::intersect(&bundle.caps, grant_caps);
    let required_refs: Vec<&str> = bundle.caps.iter().map(String::as_str).collect();
    if !allowlist.satisfies_all(&required_refs) {
        for required in &bundle.caps {
            if !allowlist.contains(required) {
                return Err(SandboxError::HostFnDenied {
                    cap: required.clone(),
                });
            }
        }
    }

    // 3. Resolve the host-fn table. perf-g7a-mr-2 fix-pass: returns
    //    OnceLock-cached Arc; no per-call BTreeMap rebuild.
    let host_fns = default_host_fns();

    // 4. Compile (or fetch from cache) the module.
    let module = crate::sandbox::instance::module_for_bytes(module_bytes).map_err(|e| {
        SandboxError::ModuleInvalid {
            reason: e.to_string(),
        }
    })?;

    // 5. Wave-8b: ensure the epoch ticker is running (D24 wallclock
    //    enforcement). Idempotent — first call spawns; subsequent are
    //    no-ops.
    spawn_epoch_ticker();

    // 6. Per-call wasmtime lifecycle. D3-RESOLVED no-pool: fresh
    //    Store + Instance per call.
    let engine: &Engine = crate::sandbox::instance::shared_engine();

    // Build the per-call store data the host-fn trampolines borrow
    // through. Phase-3 wave-5c: thread the optional live_cap_check
    // callback so PerCall recheck cadence consults the engine-side
    // revoked-actors set; absent → fall back to Phase-2b
    // grant-caps-snapshot semantics (live_caps frozen at entry).
    let store_data = SandboxStoreData::new(
        config.clone(),
        allowlist.clone(),
        attribution.clone(),
        Arc::clone(&host_fns),
        bundle.caps.clone(),
        live_cap_check,
    );

    let mut store: Store<SandboxStoreData> = Store::new(engine, store_data);

    // D21 fuel — set at store construction.
    store
        .set_fuel(config.fuel)
        .map_err(|e| SandboxError::ModuleInvalid {
            reason: format!("set_fuel failed: {e}"),
        })?;

    // D24 epoch deadline (wallclock).
    let ticks = epoch_ticks_for_ms(config.wallclock_ms);
    store.set_epoch_deadline(ticks);

    // D21 priority-1 memory cap via ResourceLimiter.
    store.limiter(|sd: &mut SandboxStoreData| &mut sd.limiter);

    // 7. Build the Linker. Walk default_host_fns() + register each
    //    THAT IS IN THE MANIFEST ALLOWLIST. ESC-8 closure: a manifest
    //    that doesn't authorise kv:read causes the linker to NOT
    //    register kv_read; wasmtime raises "unknown import" at
    //    instantiate-time which the executor maps to
    //    SandboxHostFnNotFound. Host-fns within the manifest's
    //    allowlist are registered; their trampolines further enforce
    //    D7/D18 cap-recheck on every invocation.
    let mut linker: Linker<SandboxStoreData> = Linker::new(engine);
    register_default_host_fns(&mut linker, &host_fns, &allowlist).map_err(|e| {
        SandboxError::ModuleInvalid {
            reason: format!("linker host-fn registration failed: {e}"),
        }
    })?;

    // 8. Instantiate. ESC-8 (host-fn not on manifest) defense fires here
    //    if the module imports a host-fn name the linker doesn't have —
    //    wasmtime returns an `unknown import` error which we map to
    //    HostFnNotFound.
    let instance = match linker.instantiate(&mut store, &module) {
        Ok(inst) => inst,
        Err(e) => {
            // First check if the marker is in the error chain (limiter
            // raised it during instantiate-time memory init).
            for cause in e.chain() {
                if let Some(m) = cause
                    .downcast_ref::<crate::sandbox::resource_limiter::MemoryCapExceededMarker>()
                {
                    return Err(SandboxError::MemoryExhausted {
                        limit: m.limit_bytes,
                    });
                }
            }
            // Try to recognise "unknown import" / "incompatible import"
            // shapes from wasmtime's error display.
            let msg = e.to_string();
            let lower = msg.to_lowercase();
            if lower.contains("unknown import") || lower.contains("incompatible import") {
                let name = extract_unknown_import_name(&msg).unwrap_or_else(|| msg.clone());
                return Err(SandboxError::HostFnNotFound { name });
            }
            // Map memory-related instantiation failures to MemoryExhausted.
            if lower.contains("memory") && (lower.contains("limit") || lower.contains("exceeds")) {
                return Err(SandboxError::MemoryExhausted {
                    limit: config.memory_bytes,
                });
            }
            return Err(SandboxError::ModuleInvalid {
                reason: format!("instantiation failed: {msg}"),
            });
        }
    };

    // 9. Resolve the entry function. Phase-2b convention: the exported
    //    "run" function is the entry point (the WAT corpus exports
    //    "run" universally). If "run" is missing AND a `_start` exists
    //    (wasi-style), call _start instead — but for the current corpus
    //    "run" is always present.
    let entry_name = "run";
    let entry: Option<wasmtime::Func> = instance.get_func(&mut store, entry_name);
    let Some(func) = entry else {
        // No "run" export — module shape isn't compatible.
        return Err(SandboxError::ModuleInvalid {
            reason: format!("module has no exported `{entry_name}` function"),
        });
    };

    // 10. Invoke. Use a dynamically-typed call so the corpus's varied
    //     return signatures (i32, i64, no-result) all work without a
    //     per-fixture trampoline. The return value bytes are derived
    //     from the typed return value (encoded as little-endian) for
    //     the D17 BACKSTOP check.
    //
    //     Phase-3 wave-5c §6.1-followup task #4 (ESC-13 panic-catcher
    //     around the fuel-meter callback): wrap `func.call` in
    //     `std::panic::catch_unwind` so a panicking host-side callback
    //     (currently: any of the `time` / `log` / `kv:read` trampolines
    //     that wrap user-provided panic-prone code) does NOT poison the
    //     wasmtime `Store` by unwinding through the host frames. On
    //     panic we set `esc_defense_state.fuel_meter_callback_trapped =
    //     true` and surface the typed `EscapeAttempt(Esc13StorePoison)`
    //     via the wasmtime `Error` cause chain. The per-call `Store`
    //     lifecycle (D3-RESOLVED no-pool) drops the (potentially-
    //     poisoned) Store on return so the next SANDBOX call gets a
    //     fresh one.
    let func_ty = func.ty(&store);
    let n_results = func_ty.results().len();
    // Phase-3 wave-5c — mark guest active before the call so host-fn
    // trampolines that re-enter the Store while guest_active=true bump
    // the ESC-7 re-entry counter.
    store.data_mut().esc_defense_state.enter_guest();
    let panic_caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut results: Vec<wasmtime::Val> =
            (0..n_results).map(|_| wasmtime::Val::I32(0)).collect();
        let r = func.call(&mut store, &[], &mut results);
        (r, results)
    }));
    // Always exit guest mode regardless of success / panic / trap so a
    // subsequent legitimate cleanup re-entry does not falsely fire ESC-7.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        store.data_mut().esc_defense_state.exit_guest();
    }));
    let (call_result, results): (Result<(), wasmtime::Error>, Vec<wasmtime::Val>) =
        match panic_caught {
            Ok((r, vs)) => (r, vs),
            Err(_panic_payload) => {
                // ESC-13: a host-side callback panicked; wasmtime Store is
                // poisoned. Per phase-3-backlog §6.1-followup task #4 +
                // r1-wsa-1 BLOCKER half-b: route the typed
                // `EscapeAttempt(Esc13StorePoison)` directly (the per-call
                // Store is dropped on return per D3-RESOLVED).
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    store
                        .data_mut()
                        .esc_defense_state
                        .fuel_meter_callback_trapped = true;
                }));
                return Err(SandboxError::EscapeAttempt {
                    vector: EscVector::Esc13StorePoison,
                    reason: "host-side callback panicked during guest execution; \
                         wasmtime Store is poisoned and being dropped per \
                         D3-RESOLVED per-call Store lifecycle. The next \
                         SANDBOX call gets a fresh Store. Phase-3 wave-5c \
                         §6.1-followup task #4 + r1-wsa-1 BLOCKER closure"
                        .to_string(),
                });
            }
        };

    // 11. Read fuel-consumed (regardless of success/failure).
    let fuel_remaining = store.get_fuel().unwrap_or(config.fuel);
    let fuel_consumed = config.fuel.saturating_sub(fuel_remaining);

    // Snapshot output_consumed BEFORE potentially appending the return-
    // value bytes; the BACKSTOP needs the consumed-so-far state.
    let output_consumed_before = store.data().sink.consumed();

    // 12. Map call result.
    if let Err(e) = call_result {
        let mapped = map_call_error(
            e,
            fuel_consumed,
            config.wallclock_ms,
            config.memory_bytes,
            config.fuel,
            config.max_wasm_stack,
        );
        return Err(mapped);
    }

    // 13. Encode return values into a Vec<u8> for D17 BACKSTOP +
    //     SandboxResult.output. Phase-2b convention: serialise the
    //     wasmtime::Val results little-endian. A future revision may
    //     adopt a richer ABI; for now the bytes are the raw scalar
    //     return values which is what the existing test corpus (echo
    //     handlers, ESC-fixtures) expect.
    let return_bytes = encode_return_values(&results);

    // 14. D17 BACKSTOP — check the return-value bytes against the
    //     CountedSink budget. Catches a host-fn that bypassed the
    //     PRIMARY streaming path (test-only `testing_register_uncounted_host_fn`
    //     fixture).
    let n_return = u64::try_from(return_bytes.len()).unwrap_or(u64::MAX);
    store
        .data()
        .sink
        .backstop_check(n_return, "return_value")
        .map_err(SandboxError::OutputOverflow)?;

    let _ = output_consumed_before; // currently unused; kept for symmetry

    let output_consumed = store.data().sink.consumed().saturating_add(n_return);

    Ok(SandboxResult {
        output: return_bytes,
        fuel_consumed,
        output_consumed,
    })
}

// ---------------------------------------------------------------------
// Per-call wasmtime Store data + host-fn registration (Wave-8b)
// ---------------------------------------------------------------------

/// Per-call state held in `Store::data()`. Holds the CountedSink (D17
/// PRIMARY accumulator), the init-snapshot CapAllowlist, the ResourceLimiter
/// (memory cap), per-call read/log budgets, the dispatching attribution
/// frame, the host-fn table reference, and the live cap-set (D18 PerCall
/// recheck consults this).
///
/// In wave-8b the live-cap-set is just the init-snapshot caps — the
/// engine-side wire-through that flips `kv:read` to live-policy lookup
/// lands when the engine layer's `impl PrimitiveHost::execute_sandbox`
/// override threads a live cap callback into
/// this executor (8c-paired work).
pub(crate) struct SandboxStoreData {
    /// D17 PRIMARY CountedSink the trampoline writes through.
    pub(crate) sink: CountedSink,
    /// D7 init-snapshot allowlist (consumed by PerBoundary host-fns).
    allowlist: CapAllowlist,
    /// D21 ResourceLimiter for memory cap.
    limiter: SandboxResourceLimiter,
    /// Per-call kv:read budget remaining (D1: 1000 default).
    kv_reads_remaining: u64,
    /// Per-call log byte-volume budget remaining (D1: 64 KiB default).
    log_bytes_remaining: u64,
    /// sec-pre-r1-03 — dispatching attribution frame; every host-fn
    /// invocation must carry this as the audit-frame.
    #[allow(dead_code)]
    attribution: AttributionFrame,
    /// Codegen host-fn table reference.
    #[allow(dead_code)]
    host_fns: Arc<BTreeMap<String, HostFnSpec>>,
    /// Live cap-set for D18 PerCall recheck — Phase-2b grant-caps
    /// snapshot fallback. Phase-3 wave-5c: when [`Self::live_cap_check`]
    /// is `Some`, the trampoline consults the callback first; this
    /// snapshot is the last-resort fallback for callsites that have
    /// not been migrated to the live-callback shape (engine override at
    /// `benten_engine::primitive_host::execute_sandbox` always passes
    /// `Some(callback)`).
    live_caps: Vec<String>,
    /// Phase-3 wave-5c §6.1-followup task #1 — per-call ESC defense
    /// state. Carries the ESC-7 re-entry counter, ESC-7 `guest_active`
    /// flag, ESC-13 `fuel_meter_callback_trapped` flag, ESC-16
    /// `fingerprint_correlated_reads` counter. Mutated from host-fn
    /// trampolines via `caller.data_mut()`; consulted by
    /// [`crate::sandbox::escape_defenses::run_all_checks`] at every
    /// host-fn boundary BEFORE returning to guest wasm.
    pub(crate) esc_defense_state: EscDefenseState,
    /// Phase-3 wave-5c §6.1-followup task #2 — per-call tainted-address
    /// side-table. Populated by the `time` host-fn trampoline (which
    /// writes wallclock-correlated values into guest memory) via
    /// [`record_wallclock_write`]; consulted by subsequent guest
    /// memory-read paths via
    /// [`crate::sandbox::fingerprint::read_collapse_state`]. The
    /// engine-side memory-read helper at
    /// [`crate::sandbox::fingerprint`] increments
    /// `esc_defense_state.fingerprint_correlated_reads` when a guest
    /// read hits an address in this table.
    pub(crate) tainted_addresses: Vec<WallclockTaintedAddress>,
    /// Phase-3 wave-5c §6.1-followup task #5 — live cap-recheck
    /// callback. When `Some`, the trampoline `cap_check` helper
    /// consults the callback for `PerCall` recheck (live read of the
    /// dispatching grant's cap-state); when `None`, falls back to the
    /// [`Self::live_caps`] snapshot. Engine production override always
    /// passes `Some`; Phase-2b legacy callsites pass `None`.
    pub(crate) live_cap_check: Option<LiveCapCheck>,
    /// Phase-3 wave-5c §6.1-followup tasks #3 + #4 — test-only
    /// attack-pattern injection at the trampoline boundary; mirrors
    /// [`SandboxConfig::testing_inject_attack`] into the per-call
    /// store so the host-fn trampoline can observe it.
    #[cfg(any(test, feature = "test-helpers", feature = "testing"))]
    pub(crate) testing_inject_attack: TestEscAttackInjection,
}

impl SandboxStoreData {
    fn new(
        config: SandboxConfig,
        allowlist: CapAllowlist,
        attribution: AttributionFrame,
        host_fns: Arc<BTreeMap<String, HostFnSpec>>,
        live_caps: Vec<String>,
        live_cap_check: Option<LiveCapCheck>,
    ) -> Self {
        let kv_reads_remaining = host_fns
            .get("kv:read")
            .and_then(|s| match &s.behavior {
                HostFnBehavior::KvRead { per_call_read_cap } => Some(*per_call_read_cap),
                _ => None,
            })
            .unwrap_or(1000);
        let log_bytes_remaining = host_fns
            .get("log")
            .and_then(|s| match &s.behavior {
                HostFnBehavior::LogSink { per_call_byte_cap } => Some(*per_call_byte_cap),
                _ => None,
            })
            .unwrap_or(65_536);
        #[cfg(any(test, feature = "test-helpers", feature = "testing"))]
        let testing_inject_attack = config.testing_inject_attack;
        Self {
            sink: CountedSink::new(config.output_bytes),
            allowlist,
            limiter: SandboxResourceLimiter::new(config.memory_bytes),
            kv_reads_remaining,
            log_bytes_remaining,
            attribution,
            host_fns,
            live_caps,
            esc_defense_state: EscDefenseState::new(),
            tainted_addresses: Vec::new(),
            live_cap_check,
            #[cfg(any(test, feature = "test-helpers", feature = "testing"))]
            testing_inject_attack,
        }
    }
}

/// Walk `host_fns` and register a wasmtime `Linker` import for each.
/// The closures are the trampolines: they consult the cap allowlist
/// (PerBoundary) or the live cap-set (PerCall), apply per-fn budgets
/// (kv:read read-count, log byte-volume), count output bytes via
/// CountedSink (D17 PRIMARY + D25 trampoline-counts), and return a
/// typed [`HostFnDenialMarker`] for cap-denial (sec-r1 D7) instead of a
/// wasmtime trap.
fn register_default_host_fns(
    linker: &mut Linker<SandboxStoreData>,
    host_fns: &Arc<BTreeMap<String, HostFnSpec>>,
    allowlist: &CapAllowlist,
) -> wasmtime::Result<()> {
    use crate::sandbox::host_fns::CapRecheckPolicy;

    for (name, spec) in host_fns.as_ref() {
        // ESC-8 closure: only register host-fns whose required cap is
        // in the init-snapshot allowlist. Host-fns outside the
        // allowlist are LINK-TIME absent — wasmtime raises
        // "unknown import" if the module tries to call them.
        if !allowlist.contains(&spec.requires) {
            continue;
        }
        let cap_required = spec.requires.clone();
        let recheck = spec.cap_recheck;
        let behavior = spec.behavior.clone();
        let host_name = name.clone();

        // Match the WASM import signatures used by the test corpus.
        // The corpus shape:
        //   (import "host" "time"     (func               (result i64)))
        //   (import "host" "log"      (func (param i32 i32)))
        //   (import "host" "kv_read"  (func (param i32 i32 i32 i32) (result i32)))
        match host_name.as_str() {
            "time" => {
                let cap = cap_required.clone();
                let beh = behavior.clone();
                let policy = recheck;
                linker.func_wrap(
                    "host",
                    "time",
                    move |mut caller: Caller<'_, SandboxStoreData>| -> Result<i64, wasmtime::Error> {
                        cap_check(&mut caller, policy, &cap)?;
                        // Phase-3 wave-5c §6.1-followup tasks #3 + #4 —
                        // test-only attack-pattern injection at the
                        // trampoline boundary. The cfg-gate ensures
                        // production cdylib never compiles the branch;
                        // tests drive ESC-7 / ESC-13 detection by
                        // requesting the attack-pattern via
                        // [`SandboxConfig::testing_inject_attack`].
                        #[cfg(any(test, feature = "test-helpers", feature = "testing"))]
                        {
                            let attack = caller.data().testing_inject_attack;
                            match attack {
                                TestEscAttackInjection::None => {}
                                TestEscAttackInjection::Esc7ReEntryAttempt => {
                                    let data = caller.data_mut();
                                    // Bump re-entry counter while
                                    // guest_active = true (the attack-
                                    // pattern state shape per
                                    // `testing_register_uncounted_host_fn`).
                                    data.esc_defense_state.re_entry_count = data
                                        .esc_defense_state
                                        .re_entry_count
                                        .saturating_add(1);
                                }
                                TestEscAttackInjection::Esc13FuelMeterCallbackTrap => {
                                    // Simulate a fuel-meter callback
                                    // panic. The catch_unwind wrapper in
                                    // `execute_with_live_cap_check`
                                    // around `func.call` catches the
                                    // panic + sets
                                    // `fuel_meter_callback_trapped` +
                                    // returns the typed
                                    // `EscapeAttempt(Esc13StorePoison)`.
                                    panic!(
                                        "ESC-13 test injection: fuel-meter \
                                         callback panicked during host-fn \
                                         dispatch (Phase-3 wave-5c §6.1-\
                                         followup task #4)"
                                    );
                                }
                            }
                        }
                        // Phase-3 wave-5c §6.1-followup task #2 — fingerprint
                        // side-table population. Each `time` call:
                        //   (a) records a fresh tainted-address marker (the
                        //       conceptual destination cell the wallclock-
                        //       derived value flows into; a synthetic address
                        //       increment per call so repeated calls
                        //       accumulate the side-table),
                        //   (b) reads the collapse-state against the same
                        //       address — semantically: each `time` call is
                        //       both a write (host writes wallclock value)
                        //       AND a read (guest receives + uses the value).
                        // The 3rd `time` call within a single SANDBOX
                        // dispatch trips
                        // [`crate::sandbox::fingerprint::FINGERPRINT_COLLAPSE_THRESHOLD`]
                        // and the next host-fn boundary
                        // [`run_all_checks`] fires `EscapeAttempt(Esc16FingerprintCollapse)`.
                        {
                            let data = caller.data_mut();
                            let next_idx = u32::try_from(data.tainted_addresses.len())
                                .unwrap_or(u32::MAX);
                            // 8-byte stride so each i64 wallclock value
                            // gets its own conceptual slot.
                            let addr = 0x0000_1000_u32.saturating_add(next_idx.saturating_mul(8));
                            let tainted = record_wallclock_write(addr);
                            data.tainted_addresses.push(tainted);
                            // Synthetic guest-read of the just-written cell
                            // (the host returns the value to the guest,
                            // which by definition reads it).
                            // Clone the addresses to avoid borrowing
                            // `data.tainted_addresses` while also borrowing
                            // `data.esc_defense_state` mutably.
                            let tainted_set = data.tainted_addresses.clone();
                            read_collapse_state(
                                &mut data.esc_defense_state,
                                tainted,
                                &tainted_set,
                            );
                        }

                        // ESC defense boundary check BEFORE the host-fn
                        // returns to guest. If ESC-16 (or any other vector)
                        // tripped, surface as `EscapeAttemptMarker` so
                        // [`map_call_error`] unwraps to typed
                        // `SandboxError::EscapeAttempt`.
                        if let Err(esc_err) =
                            run_all_checks(&caller.data().esc_defense_state)
                            && let SandboxError::EscapeAttempt { vector, reason } = esc_err
                        {
                            return Err(wasmtime::Error::from(EscapeAttemptMarker {
                                vector,
                                reason,
                            }));
                        }

                        // D1 monotonic-coarsened-100ms. Phase-2b: derive a
                        // module-start-relative monotonic value coarsened
                        // to the configured granularity (default 100ms);
                        // returning the same value across a sub-window
                        // closes ESC-16 fingerprinting.
                        let coarsening_ms = match &beh {
                            HostFnBehavior::TimeMonotonicCoarsened { coarsening_ms } => *coarsening_ms,
                            _ => 100,
                        };
                        // Use a process-static start instant; modules see a
                        // monotonic offset coarsened to the configured ms
                        // granularity.
                        let elapsed = sandbox_module_relative_time_ms();
                        let coarsened = elapsed
                            .checked_div(coarsening_ms)
                            .map_or(elapsed, |q| q * coarsening_ms);
                        Ok(i64::try_from(coarsened).unwrap_or(i64::MAX))
                    },
                )?;
            }
            "log" => {
                let cap = cap_required.clone();
                let beh = behavior.clone();
                let policy = recheck;
                linker.func_wrap(
                    "host",
                    "log",
                    move |mut caller: Caller<'_, SandboxStoreData>,
                          ptr: i32,
                          len: i32|
                          -> Result<(), wasmtime::Error> {
                        cap_check(&mut caller, policy, &cap)?;
                        let per_call_byte_cap = match &beh {
                            HostFnBehavior::LogSink { per_call_byte_cap } => *per_call_byte_cap,
                            _ => 65_536,
                        };
                        let len_u64 = u64::try_from(len.max(0)).unwrap_or(0);
                        // D1 + sec-pre-r1-06 §2.2: `per_call_byte_cap`
                        // semantics — the cap is PER `log` HOST-FN
                        // INVOCATION (not aggregated across calls). The
                        // aggregate budget IS CountedSink (the
                        // primitive-call output_bytes budget), which
                        // every host-fn shares.
                        if len_u64 > per_call_byte_cap {
                            return Err(wasmtime::Error::from(HostFnDenialMarker {
                                kind: HostFnDenialKind::CapDenied {
                                    cap: format!("log:per_call_byte_cap={per_call_byte_cap}"),
                                },
                            }));
                        }
                        let data = caller.data_mut();
                        // D17 PRIMARY: count log bytes against the
                        // per-call output budget through CountedSink.
                        // CountedSink is the cumulative cross-host-fn
                        // budget — when it overflows the typed error
                        // routes to E_INV_SANDBOX_OUTPUT (NOT
                        // SandboxHostFnDenied).
                        if let Err(o) = data.sink.write_n_bytes(len_u64, "host_fn:compute:log") {
                            return Err(wasmtime::Error::from(HostFnDenialMarker {
                                kind: HostFnDenialKind::OutputOverflow(o),
                            }));
                        }
                        // Diagnostic: track total log bytes per primitive
                        // call (kept for SandboxResult.fuel_consumed
                        // future telemetry).
                        data.log_bytes_remaining = data.log_bytes_remaining.saturating_sub(len_u64);
                        // We don't actually read the bytes from guest
                        // memory in 2b — `log` is fire-and-forget; the
                        // budget enforcement is what's load-bearing.
                        let _ = ptr;
                        // Phase-3 wave-5c §6.1-followup task #3 — ESC
                        // defense boundary check BEFORE returning to
                        // guest. If any vector tripped, surface as
                        // `EscapeAttemptMarker` so [`map_call_error`]
                        // unwraps to typed `SandboxError::EscapeAttempt`.
                        if let Err(esc_err) = run_all_checks(&caller.data().esc_defense_state)
                            && let SandboxError::EscapeAttempt { vector, reason } = esc_err
                        {
                            return Err(wasmtime::Error::from(EscapeAttemptMarker {
                                vector,
                                reason,
                            }));
                        }
                        Ok(())
                    },
                )?;
            }
            "kv:read" => {
                // The wasm-import name is `kv_read` (underscore) by
                // convention; the host-fn registry uses `kv:read`
                // (colon) for the cap-string namespacing. The Linker
                // registration uses the wasm-side name.
                let cap = cap_required.clone();
                let beh = behavior.clone();
                let policy = recheck;
                linker.func_wrap(
                    "host",
                    "kv_read",
                    move |mut caller: Caller<'_, SandboxStoreData>,
                          key_ptr: i32,
                          key_len: i32,
                          out_ptr: i32,
                          out_len: i32|
                          -> Result<i32, wasmtime::Error> {
                        cap_check(&mut caller, policy, &cap)?;
                        let per_call_read_cap = match &beh {
                            HostFnBehavior::KvRead { per_call_read_cap } => *per_call_read_cap,
                            _ => 1000,
                        };
                        let _ = per_call_read_cap;
                        let data = caller.data_mut();
                        if data.kv_reads_remaining == 0 {
                            return Err(wasmtime::Error::from(HostFnDenialMarker {
                                kind: HostFnDenialKind::CapDenied {
                                    cap: "kv:read:per_call_read_cap_exhausted".to_string(),
                                },
                            }));
                        }
                        // ESC-3 host-buffer overrun defense: validate
                        // (key_ptr, key_len, out_ptr, out_len) shape
                        // against the module's declared memory size.
                        // Negative values (interpreted as i32 ints
                        // representing huge u32s in unsigned semantics)
                        // also fire — the WASM ABI passes these as i32
                        // but a negative value here is always pathological.
                        if out_len < 0 || out_ptr < 0 || key_len < 0 || key_ptr < 0 {
                            return Err(wasmtime::Error::from(wasmtime::Trap::MemoryOutOfBounds));
                        }
                        let out_len_u64 = u64::try_from(out_len).unwrap_or(0);
                        let out_ptr_u64 = u64::try_from(out_ptr).unwrap_or(0);
                        let key_ptr_u64 = u64::try_from(key_ptr).unwrap_or(0);
                        let key_len_u64 = u64::try_from(key_len).unwrap_or(0);
                        let mem = caller.get_export("memory").and_then(|e| e.into_memory());
                        if let Some(mem) = mem {
                            let mem_size =
                                u64::try_from(mem.data_size(&caller)).unwrap_or(u64::MAX);
                            if out_len_u64 > mem_size
                                || out_ptr_u64.saturating_add(out_len_u64) > mem_size
                                || key_ptr_u64.saturating_add(key_len_u64) > mem_size
                            {
                                return Err(wasmtime::Error::from(
                                    wasmtime::Trap::MemoryOutOfBounds,
                                ));
                            }
                        }
                        let data = caller.data_mut();
                        data.kv_reads_remaining = data.kv_reads_remaining.saturating_sub(1);
                        // D17 PRIMARY: count "would-be-written" bytes
                        // against the output budget. Wave-8b stub
                        // doesn't actually write to guest memory; the
                        // engine wire-through (8c) will.
                        if let Err(o) = data.sink.write_n_bytes(0, "host_fn:compute:kv:read") {
                            return Err(wasmtime::Error::from(HostFnDenialMarker {
                                kind: HostFnDenialKind::OutputOverflow(o),
                            }));
                        }
                        // Phase-3 wave-5c §6.1-followup task #3 — ESC
                        // defense boundary check BEFORE returning to
                        // guest.
                        if let Err(esc_err) = run_all_checks(&caller.data().esc_defense_state)
                            && let SandboxError::EscapeAttempt { vector, reason } = esc_err
                        {
                            return Err(wasmtime::Error::from(EscapeAttemptMarker {
                                vector,
                                reason,
                            }));
                        }
                        Ok(0)
                    },
                )?;
            }
            _ => {
                // Unknown host-fn name in the table — declined at
                // registration time. Wave-8b only ships the D1 surface.
            }
        }
    }

    Ok(())
}

/// Cap-check helper called at the top of every host-fn trampoline.
/// PerBoundary consults the init-snapshot allowlist; PerCall consults
/// the live cap-set (Phase-3 wave-5c §6.1-followup task #5: when
/// `live_cap_check` callback is present, that callback is the
/// authoritative source — fires BEFORE every host-fn invocation per
/// r1-wsa-3 MAJOR cadence (a) per r4-r1-wsa-4. When the callback is
/// absent, falls back to the Phase-2b `live_caps` snapshot for
/// backward compatibility with legacy callsites).
fn cap_check(
    caller: &mut Caller<'_, SandboxStoreData>,
    policy: crate::sandbox::host_fns::CapRecheckPolicy,
    cap: &str,
) -> Result<(), wasmtime::Error> {
    use crate::sandbox::host_fns::CapRecheckPolicy;
    let data = caller.data();
    let ok = match policy {
        CapRecheckPolicy::PerBoundary => data.allowlist.contains(cap),
        CapRecheckPolicy::PerCall => match &data.live_cap_check {
            // Phase-3 wave-5c — consult the live callback (engine-backed
            // production path observes mid-call cap-revoke per ESC-9
            // closure). Cadence (a): once per host-fn entry, not per
            // loop iteration inside the host-fn body (the body completes
            // its budget on the per-host-fn-entry policy snapshot).
            Some(callback) => callback(cap),
            // Phase-2b legacy fallback — snapshot frozen at SANDBOX
            // entry. Effectively PerBoundary semantics; the engine
            // override always passes Some(callback) so production runs
            // never hit this branch.
            None => data.live_caps.iter().any(|c| c == cap),
        },
    };
    if !ok {
        return Err(wasmtime::Error::from(HostFnDenialMarker {
            kind: HostFnDenialKind::CapDenied {
                cap: cap.to_string(),
            },
        }));
    }
    Ok(())
}

/// Module-relative monotonic time helper for the `time` host-fn.
/// Returns elapsed milliseconds since process start, NOT since epoch
/// (closes ESC-16 timezone leak + the no-correlation-with-system-clock
/// pin in `sandbox_host_fn_time_returns_monotonic_coarsened_100ms`).
fn sandbox_module_relative_time_ms() -> u64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static PROCESS_START: OnceLock<Instant> = OnceLock::new();
    let start = *PROCESS_START.get_or_init(Instant::now);
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Encode a wasmtime `Val` results vector to little-endian bytes for
/// D17 BACKSTOP + SandboxResult.output.
fn encode_return_values(results: &[wasmtime::Val]) -> Vec<u8> {
    let mut out = Vec::with_capacity(results.len() * 8);
    for v in results {
        match v {
            wasmtime::Val::I32(n) => out.extend_from_slice(&n.to_le_bytes()),
            wasmtime::Val::I64(n) => out.extend_from_slice(&n.to_le_bytes()),
            wasmtime::Val::F32(bits) => out.extend_from_slice(&bits.to_le_bytes()),
            wasmtime::Val::F64(bits) => out.extend_from_slice(&bits.to_le_bytes()),
            // V128, FuncRef, ExternRef — current corpus doesn't use them; encode as zero placeholder.
            _ => out.extend_from_slice(&[0u8; 16]),
        }
    }
    out
}

/// Best-effort extraction of an "unknown import" name from wasmtime's
/// error-display string.
fn extract_unknown_import_name(msg: &str) -> Option<String> {
    // wasmtime 43 displays: `unknown import: \`module::name\` has not been defined`
    // or similar. Look for backticks first; fall back to colon-split.
    if let (Some(start), Some(end)) = (msg.find('`'), msg.rfind('`'))
        && start < end
    {
        return Some(msg[start + 1..end].to_string());
    }
    None
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::sandbox::manifest::CapBundle;
    use benten_core::Cid;

    /// Test helper: dummy AttributionFrame for the executor surface.
    fn test_attribution() -> AttributionFrame {
        let zero = Cid::from_blake3_digest([0u8; 32]);
        AttributionFrame {
            actor_cid: zero,
            handler_cid: zero,
            capability_grant_cid: zero,
            sandbox_depth: 0,
        }
    }

    #[test]
    fn d21_priority_memory_over_wallclock() {
        let pick = resolve_priority(vec![
            SandboxError::WallclockExceeded { limit_ms: 1000 },
            SandboxError::MemoryExhausted { limit: 100 },
        ]);
        assert!(matches!(pick, Some(SandboxError::MemoryExhausted { .. })));
    }

    #[test]
    fn d21_priority_wallclock_over_fuel() {
        let pick = resolve_priority(vec![
            SandboxError::FuelExhausted {
                consumed: 0,
                limit: 100,
            },
            SandboxError::WallclockExceeded { limit_ms: 1000 },
        ]);
        assert!(matches!(pick, Some(SandboxError::WallclockExceeded { .. })));
    }

    #[test]
    fn d21_priority_fuel_over_output() {
        let overflow = SinkOverflow {
            consumed: 5,
            limit: 5,
            emitter_kind: "host_fn:compute:log".to_string(),
            path: OverflowPath::PrimaryStreaming,
        };
        let pick = resolve_priority(vec![
            SandboxError::OutputOverflow(overflow),
            SandboxError::FuelExhausted {
                consumed: 0,
                limit: 100,
            },
        ]);
        assert!(matches!(pick, Some(SandboxError::FuelExhausted { .. })));
    }

    #[test]
    fn d24_default_wallclock_30s() {
        let cfg = SandboxConfig::default();
        assert_eq!(cfg.wallclock_ms, WALLCLOCK_DEFAULT_MS);
        assert_eq!(WALLCLOCK_DEFAULT_MS, 30_000);
    }

    #[test]
    fn d24_with_wallclock_within_ceiling_accepts() {
        let cfg = SandboxConfig::default().with_wallclock_ms(60_000).unwrap();
        assert_eq!(cfg.wallclock_ms, 60_000);
    }

    #[test]
    fn d24_with_wallclock_above_ceiling_rejects() {
        let err = SandboxConfig::default()
            .with_wallclock_ms(WALLCLOCK_MAX_MS + 1)
            .unwrap_err();
        assert_eq!(err, ErrorCode::SandboxWallclockInvalid);
    }

    #[test]
    fn d24_with_wallclock_zero_rejects() {
        let err = SandboxConfig::default().with_wallclock_ms(0).unwrap_err();
        assert_eq!(err, ErrorCode::SandboxWallclockInvalid);
    }

    #[test]
    fn budget_type_for_each_axis() {
        assert_eq!(
            SandboxError::FuelExhausted {
                consumed: 0,
                limit: 0
            }
            .budget_type(),
            Some("sandbox_fuel")
        );
        assert_eq!(
            SandboxError::MemoryExhausted { limit: 0 }.budget_type(),
            Some("sandbox_memory")
        );
        assert_eq!(
            SandboxError::WallclockExceeded { limit_ms: 0 }.budget_type(),
            Some("sandbox_wallclock")
        );
        assert_eq!(
            SandboxError::HostFnDenied {
                cap: "x".to_string()
            }
            .budget_type(),
            None,
            "host-fn denial is not a budget axis"
        );
    }

    #[test]
    fn manifest_unknown_routes_typed_error() {
        let registry = ManifestRegistry::new();
        let module_bytes = wat::parse_str("(module)").expect("empty module compiles");
        let attribution = test_attribution();
        let err = execute(
            &module_bytes,
            ManifestRef::named("compute-power"),
            &registry,
            SandboxConfig::default(),
            &[],
            &attribution,
        )
        .unwrap_err();
        assert_eq!(err.code(), ErrorCode::SandboxManifestUnknown);
    }

    #[test]
    fn init_snapshot_denies_when_grant_lacks_manifest_cap() {
        // Plan §3 G7-A — D7 init-time intersection. Manifest claims
        // {time, log}; grant holds {time} only — log denied at init.
        let registry = ManifestRegistry::new();
        let module_bytes = wat::parse_str("(module)").expect("empty module compiles");
        let attribution = test_attribution();
        let err = execute(
            &module_bytes,
            ManifestRef::named("compute-basic"),
            &registry,
            SandboxConfig::default(),
            &["host:compute:time".to_string()],
            &attribution,
        )
        .unwrap_err();
        assert_eq!(err.code(), ErrorCode::SandboxHostFnDenied);
    }

    #[test]
    fn inline_manifest_resolves_without_registry_entry() {
        let registry = ManifestRegistry::new();
        let inline = CapBundle::new(vec!["host:compute:time".to_string()], None);
        // Wave-8b: the executor now actually invokes wasmtime — module
        // MUST export a `run` function. Use a trivial echo-shape that
        // returns 0.
        let module_bytes =
            wat::parse_str("(module (func (export \"run\") (result i32) i32.const 0))")
                .expect("trivial run module compiles");
        let attribution = test_attribution();
        let res = execute(
            &module_bytes,
            ManifestRef::Inline(inline),
            &registry,
            SandboxConfig::default(),
            &["host:compute:time".to_string()],
            &attribution,
        );
        assert!(
            res.is_ok(),
            "inline manifest with matching grant must succeed; got {:?}",
            res
        );
    }

    /// **sec-g7a-mr-8 fix-pass:** inline-manifest denial-case mirror of
    /// `init_snapshot_denies_when_grant_lacks_manifest_cap`. Defends
    /// against a future change that special-cases inline-bundles in the
    /// resolution path and accidentally skips the cap-intersection
    /// check.
    #[test]
    fn inline_manifest_denied_when_grant_lacks_inline_cap() {
        let registry = ManifestRegistry::new();
        let inline = CapBundle::new(
            vec![
                "host:compute:log".to_string(),
                "host:compute:time".to_string(),
            ],
            None,
        );
        let module_bytes = wat::parse_str("(module)").expect("empty module compiles");
        let attribution = test_attribution();
        let err = execute(
            &module_bytes,
            ManifestRef::Inline(inline),
            &registry,
            SandboxConfig::default(),
            &["host:compute:time".to_string()],
            &attribution,
        )
        .unwrap_err();
        assert_eq!(
            err.code(),
            ErrorCode::SandboxHostFnDenied,
            "inline manifest claiming `log` against grant w/o `log` MUST be denied"
        );
    }

    /// **sec-g7a-mr-5 fix-pass:** D1 `random` deferred host-fn — manifest
    /// claiming a `host:compute:random*` cap fires
    /// SandboxHostFnNotFound at validate time with the deferred-to-2c
    /// hint. Defensive belt-and-braces while G7-C wires module-link-time
    /// host-fn enumeration.
    #[test]
    fn random_host_fn_cap_in_manifest_fires_not_found_with_phase_2c_hint() {
        let registry = ManifestRegistry::new();
        let inline = CapBundle::new(vec!["host:compute:random".to_string()], None);
        let module_bytes = wat::parse_str("(module)").expect("empty module compiles");
        let attribution = test_attribution();
        let err = execute(
            &module_bytes,
            ManifestRef::Inline(inline),
            &registry,
            SandboxConfig::default(),
            &["host:compute:random".to_string()],
            &attribution,
        )
        .unwrap_err();
        assert_eq!(err.code(), ErrorCode::SandboxHostFnNotFound);
        if let SandboxError::HostFnNotFound { name } = err {
            // Operator-facing hint MUST signal (a) the host-fn isn't
            // available yet AND (b) where to find the canonical
            // destination doc — see phase-3-backlog.md §6.10 for the
            // workspace CSPRNG framework choice that gates re-enabling.
            assert!(
                name.contains("not yet implemented") && name.contains("§6.10"),
                "operator hint MUST signal random-host-fn not-yet-implemented + \
                 cite phase-3-backlog §6.10; got: {name}"
            );
        }
    }

    /// **cr-g7a-mr-6 fix-pass:** ManifestEncodeFailed routes through
    /// E_SERIALIZE (not E_SANDBOX_MODULE_INVALID).
    #[test]
    fn manifest_encode_failure_routes_to_serialize_not_module_invalid() {
        let err = SandboxError::ManifestEncodeFailed {
            reason: "test".to_string(),
        };
        assert_eq!(err.code(), ErrorCode::Serialize);
    }
}
