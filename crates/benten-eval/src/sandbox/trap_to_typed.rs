//! Trap → SandboxError typed-error mapping (Phase 2b Wave-8b).
//!
//! When wasmtime's `Instance::call` (or `TypedFunc::call`) returns an
//! error, the error's underlying [`wasmtime::Trap`] (or wrapped
//! `Error`) determines which axis fired. This module owns the mapping
//! per the brief table:
//!
//! | Trap signal | Typed error | Priority |
//! |---|---|---|
//! | `wasmtime::Trap::OutOfFuel` | `E_SANDBOX_FUEL_EXHAUSTED` | 3 |
//! | OOM via ResourceLimiter | `E_SANDBOX_MEMORY_EXHAUSTED` | 1 (highest) |
//! | Epoch deadline | `E_SANDBOX_WALLCLOCK_EXCEEDED` | 2 |
//! | `CountedSink` overflow | `E_INV_SANDBOX_OUTPUT` | 4 |
//! | Other trap codes | `SandboxError::ModuleInvalid` | 5 |
//!
//! D21 priority resolver (`MEMORY > WALLCLOCK > FUEL > OUTPUT`) lives in
//! [`crate::primitives::sandbox::resolve_priority`] (the
//! `primitives/sandbox.rs::resolve_priority` function); this module just
//! does the per-trap classification. wasmtime delivers a single trap per
//! call so the priority resolver is a no-op for the single-trap case;
//! the multi-axis vector input shape is reserved for future
//! multi-axis-trip composition (verified by the three unit tests at
//! `crates/benten-eval/src/primitives/sandbox.rs::tests` —
//! `d21_priority_memory_over_wallclock`,
//! `d21_priority_wallclock_over_fuel`, `d21_priority_fuel_over_output`).
//!
//! Side-channel signal: the host trampoline can raise a
//! [`HostFnDenialMarker`] anyhow-error which this module unwraps as a
//! [`SandboxError::HostFnDenied`] / [`SandboxError::HostFnNotFound`] /
//! [`SandboxError::NestedDispatchDenied`] without a wasmtime trap in
//! flight (sec-r1 D7 — host-fn cap denial routes typed error, NOT
//! wasmtime trap that would corrupt module state).
//!
//! This module is `#[cfg(not(target_arch = "wasm32"))]`-gated per
//! sec-pre-r1-05; the wasm32 build cuts SANDBOX entirely.

#![cfg(not(target_arch = "wasm32"))]

use crate::primitives::sandbox::SandboxError;
use crate::sandbox::counted_sink::SinkOverflow;

/// Marker error the host-fn trampoline raises to signal a typed,
/// non-trap denial (cap denial, host-fn-not-found, nested-dispatch).
/// The executor catches anyhow-errors out of `Instance::call` + checks
/// `downcast_ref::<HostFnDenialMarker>()` — when present, the typed
/// error is delivered directly instead of treating the failure as a
/// wasmtime trap. sec-r1 D7 closure: cap-denial does NOT corrupt the
/// store; the module receives a typed error rather than an unwinding
/// trap.
#[derive(Debug, thiserror::Error)]
#[error("sandbox host-fn typed-error marker: {kind:?}")]
pub struct HostFnDenialMarker {
    /// The typed-error variant the trampoline wants to deliver.
    pub kind: HostFnDenialKind,
}

/// Sub-variants of [`HostFnDenialMarker`] — what kind of typed error
/// the trampoline saw at host-side.
#[derive(Debug, Clone)]
pub enum HostFnDenialKind {
    /// D7/D18 cap-recheck denial.
    CapDenied {
        /// Cap-string the call required.
        cap: String,
    },
    /// Module imported a host-fn name not in the active manifest.
    NotFound {
        /// Imported name.
        name: String,
    },
    /// D19 nested-dispatch attempt by host-fn callback.
    NestedDispatchDenied,
    /// CountedSink PRIMARY overflow (host-fn write exceeded budget).
    OutputOverflow(SinkOverflow),
}

impl HostFnDenialKind {
    /// Convert to the corresponding [`SandboxError`] variant.
    #[must_use]
    pub fn into_sandbox_error(self) -> SandboxError {
        match self {
            HostFnDenialKind::CapDenied { cap } => SandboxError::HostFnDenied { cap },
            HostFnDenialKind::NotFound { name } => SandboxError::HostFnNotFound { name },
            HostFnDenialKind::NestedDispatchDenied => SandboxError::NestedDispatchDenied,
            HostFnDenialKind::OutputOverflow(o) => SandboxError::OutputOverflow(o),
        }
    }
}

/// Map a wasmtime `anyhow::Error` from `Instance::call` / `TypedFunc::call`
/// into a typed [`SandboxError`].
///
/// The `consumed_fuel` + `wallclock_limit_ms` + `memory_limit_bytes`
/// parameters supply context for the variant payloads; the executor
/// captures these per-call and threads them through.
///
/// Order of recognition (NOT priority — that's `resolve_priority`):
/// 1. Host-fn typed-error marker (sec-r1 D7) — bypasses trap path.
/// 2. wasmtime `Trap` — classified by code (`OutOfFuel` /
///    `MemoryOutOfBounds` etc.).
/// 3. Epoch interruption — wasmtime surfaces as `Trap::Interrupt` in
///    43.x (the epoch-deadline mechanism produces an Interrupt trap;
///    we map to wallclock).
/// 4. Anything else → `ModuleInvalid` with the original Display string.
#[must_use]
pub fn map_call_error(
    err: wasmtime::Error,
    consumed_fuel: u64,
    wallclock_limit_ms: u64,
    memory_limit_bytes: u64,
    fuel_limit: u64,
) -> SandboxError {
    // 1. Host-fn typed-error marker first (sec-r1 D7).
    if let Some(marker) = err.downcast_ref::<HostFnDenialMarker>() {
        return marker.kind.clone().into_sandbox_error();
    }

    // 1b. ResourceLimiter MemoryCapExceededMarker — deterministic D21
    //     priority-1 memory-cap routing.
    if let Some(marker) =
        err.downcast_ref::<crate::sandbox::resource_limiter::MemoryCapExceededMarker>()
    {
        return SandboxError::MemoryExhausted {
            limit: marker.limit_bytes,
        };
    }

    // 1c. Walk the cause chain — wasmtime sometimes wraps the limiter
    //     error in a context layer; downcast_ref alone may miss the
    //     marker in nested-cause cases.
    for cause in err.chain() {
        if let Some(m) = cause.downcast_ref::<HostFnDenialMarker>() {
            return m.kind.clone().into_sandbox_error();
        }
        if let Some(m) =
            cause.downcast_ref::<crate::sandbox::resource_limiter::MemoryCapExceededMarker>()
        {
            return SandboxError::MemoryExhausted {
                limit: m.limit_bytes,
            };
        }
    }

    // 2. wasmtime Trap classification.
    if let Some(trap) = err.downcast_ref::<wasmtime::Trap>() {
        return match trap {
            wasmtime::Trap::OutOfFuel => SandboxError::FuelExhausted {
                consumed: consumed_fuel,
                limit: fuel_limit,
            },
            wasmtime::Trap::MemoryOutOfBounds
            | wasmtime::Trap::HeapMisaligned
            | wasmtime::Trap::TableOutOfBounds
            | wasmtime::Trap::IndirectCallToNull => SandboxError::ModuleInvalid {
                reason: format!("wasmtime trap: {trap:?}"),
            },
            wasmtime::Trap::Interrupt => SandboxError::WallclockExceeded {
                limit_ms: wallclock_limit_ms,
            },
            wasmtime::Trap::StackOverflow => SandboxError::ModuleInvalid {
                reason: "wasmtime stack overflow (max_wasm_stack exceeded)".to_string(),
            },
            wasmtime::Trap::UnreachableCodeReached => SandboxError::ModuleInvalid {
                reason: "wasmtime unreachable instruction".to_string(),
            },
            other => SandboxError::ModuleInvalid {
                reason: format!("wasmtime trap: {other:?}"),
            },
        };
    }

    // 3. ResourceLimiter rejection of memory growth surfaces as a wasmtime
    //    error whose Display contains "memory minimum size of N pages exceeds
    //    memory limits" or similar — these aren't classified as Trap by
    //    wasmtime 43, so we fall through here. Detect by error string.
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    if lower.contains("memory") && (lower.contains("limit") || lower.contains("exceeds")) {
        return SandboxError::MemoryExhausted {
            limit: memory_limit_bytes,
        };
    }

    // 4. Default: ModuleInvalid with the Display string preserved.
    SandboxError::ModuleInvalid {
        reason: format!("wasmtime error: {msg}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_fn_denial_marker_round_trips_cap_denied() {
        let marker = HostFnDenialMarker {
            kind: HostFnDenialKind::CapDenied {
                cap: "host:compute:kv:read".to_string(),
            },
        };
        let err = wasmtime::Error::from(marker);
        let mapped = map_call_error(err, 0, 30_000, 64 * 1024 * 1024, 1_000_000);
        assert!(matches!(mapped, SandboxError::HostFnDenied { .. }));
    }

    #[test]
    fn host_fn_denial_marker_round_trips_not_found() {
        let marker = HostFnDenialMarker {
            kind: HostFnDenialKind::NotFound {
                name: "kv_read".to_string(),
            },
        };
        let err = wasmtime::Error::from(marker);
        let mapped = map_call_error(err, 0, 30_000, 64 * 1024 * 1024, 1_000_000);
        assert!(matches!(mapped, SandboxError::HostFnNotFound { .. }));
    }

    #[test]
    fn host_fn_denial_marker_round_trips_nested_dispatch() {
        let marker = HostFnDenialMarker {
            kind: HostFnDenialKind::NestedDispatchDenied,
        };
        let err = wasmtime::Error::from(marker);
        let mapped = map_call_error(err, 0, 30_000, 64 * 1024 * 1024, 1_000_000);
        assert!(matches!(mapped, SandboxError::NestedDispatchDenied));
    }

    #[test]
    fn out_of_fuel_trap_routes_fuel_exhausted() {
        let err = wasmtime::Error::from(wasmtime::Trap::OutOfFuel);
        let mapped = map_call_error(err, 1_000_000, 30_000, 64 * 1024 * 1024, 1_000_000);
        assert!(matches!(
            mapped,
            SandboxError::FuelExhausted {
                consumed: 1_000_000,
                limit: 1_000_000
            }
        ));
    }

    #[test]
    fn interrupt_trap_routes_wallclock() {
        let err = wasmtime::Error::from(wasmtime::Trap::Interrupt);
        let mapped = map_call_error(err, 0, 1000, 64 * 1024 * 1024, 1_000_000);
        assert!(matches!(
            mapped,
            SandboxError::WallclockExceeded { limit_ms: 1000 }
        ));
    }

    #[test]
    fn memory_oob_trap_routes_module_invalid() {
        let err = wasmtime::Error::from(wasmtime::Trap::MemoryOutOfBounds);
        let mapped = map_call_error(err, 0, 30_000, 64 * 1024 * 1024, 1_000_000);
        assert!(matches!(mapped, SandboxError::ModuleInvalid { .. }));
    }

    #[test]
    fn output_overflow_marker_routes_through() {
        use crate::sandbox::counted_sink::OverflowPath;
        let marker = HostFnDenialMarker {
            kind: HostFnDenialKind::OutputOverflow(SinkOverflow {
                consumed: 100,
                limit: 100,
                emitter_kind: "host_fn:compute:log".to_string(),
                path: OverflowPath::PrimaryStreaming,
            }),
        };
        let err = wasmtime::Error::from(marker);
        let mapped = map_call_error(err, 0, 30_000, 64 * 1024 * 1024, 1_000_000);
        assert!(matches!(mapped, SandboxError::OutputOverflow(_)));
    }
}
