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
//! `inv_8_iteration` pattern at `evaluator.rs:185-192`. Same for
//! `sandbox_memory`, `sandbox_wallclock`, `sandbox_output` budget types.
//!
//! This module is `#[cfg(not(target_arch = "wasm32"))]`-gated per
//! sec-pre-r1-05; the wasm32 build cuts SANDBOX entirely.

#![cfg(not(target_arch = "wasm32"))]

use crate::sandbox::counted_sink::{CountedSink, OverflowPath, SinkOverflow};
use crate::sandbox::host_fns::{CapAllowlist, default_host_fns};
use crate::sandbox::manifest::{ManifestRef, ManifestRegistry};
use crate::{AttributionFrame, TraceStep};
use benten_errors::ErrorCode;

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
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            fuel: 1_000_000,
            memory_bytes: 64 * 1024 * 1024,
            wallclock_ms: WALLCLOCK_DEFAULT_MS,
            output_bytes: 1024 * 1024,
            max_nest_depth: 4,
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
    /// `inv_8_iteration` pattern at `evaluator.rs:185-192`). Returns
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
pub fn execute(
    module_bytes: &[u8],
    manifest_ref: ManifestRef,
    registry: &ManifestRegistry,
    config: SandboxConfig,
    grant_caps: &[String],
    attribution: &AttributionFrame,
) -> Result<SandboxResult, SandboxError> {
    // sec-g7a-mr-1 — pin: the attribution frame parameter exists on
    // execute() so G7-C can thread it into HostFnContext. G7-A scaffold
    // does not yet emit the audit-row; the load-bearing surface is the
    // signature.
    let _ = attribution;

    // 1. Resolve the manifest. ESC-15 closure: `Named` lookup either
    //    returns a bundle or fires `SandboxError::ManifestUnknown`.
    //    cr-g7a-mr-6 fix-pass: ManifestError::Encode now routes through
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
            // resolve() never produces this variant against an
            // existing registry (register_runtime is the only
            // producer); map defensively to ManifestEncodeFailed
            // so the type stays honest.
            SandboxError::ManifestEncodeFailed {
                reason: "RuntimeRegistrationDeferred surfaced from resolve() — \
                                 should not happen against existing registry"
                    .to_string(),
            }
        }
    })?;

    // sec-g7a-mr-5 — defensive D1 random-host-fn deferral guard. Until
    // the full module-link-time host-fn enumeration lands in G7-C, fire
    // SandboxHostFnNotFound at validate-time if the manifest claims a
    // `random` cap. Operator-actionable hint encoded in the message
    // (mirrors the host-functions.toml comment).
    for required in &bundle.caps {
        if required.starts_with(DEFERRED_HOST_FN_RANDOM_CAP_PREFIX) {
            return Err(SandboxError::HostFnNotFound {
                name: format!(
                    "random (cap='{}'): deferred to Phase 2c per D1 + sec-pre-r1-06 §2.3 \
                     (workspace CSPRNG framework choice not yet made)",
                    required
                ),
            });
        }
    }

    // 2. D7 init-snapshot intersection — fail loud if the manifest
    //    claims caps the dispatching grant lacks. **sec-g7a-mr-4 +
    //    perf-g7a-mr-8 fix-pass:** delegate to CapAllowlist::intersect
    //    + satisfies_all so the executor's intersection behavior is
    //    pinned by the same contract HostFnContext.allowlist consumes.
    //    Locks O(M+G) sorted-set intersection vs the previous O(M*G)
    //    inline loop.
    let allowlist = CapAllowlist::intersect(&bundle.caps, grant_caps);
    let required_refs: Vec<&str> = bundle.caps.iter().map(String::as_str).collect();
    if !allowlist.satisfies_all(&required_refs) {
        // Pick the first missing required cap for the error payload.
        for required in &bundle.caps {
            if !allowlist.contains(required) {
                return Err(SandboxError::HostFnDenied {
                    cap: required.clone(),
                });
            }
        }
    }

    // 3. Resolve the host-fn table. The default codegen surface
    //    contributes `time`/`log`/`kv:read` (D1). `random` is
    //    intentionally absent — defensive prefix-check above fires
    //    SandboxHostFnNotFound before reaching here for `random`.
    //    perf-g7a-mr-2 fix-pass: returns OnceLock-cached Arc; no
    //    per-call BTreeMap rebuild.
    let _host_fns = default_host_fns();

    // 4. Compile (or fetch from cache) the module.
    let _module = crate::sandbox::instance::module_for_bytes(module_bytes).map_err(|e| {
        SandboxError::ModuleInvalid {
            reason: e.to_string(),
        }
    })?;

    // 5. Per-call state: CountedSink (D17 PRIMARY) + read budget +
    //    log byte-volume budget. The trampoline (engine integration)
    //    consumes these.
    let _sink = CountedSink::new(config.output_bytes);

    // 6. Per-call wasmtime Store + Instance lifecycle. The engine
    //    integration (G7-C) constructs Store + Instance fresh, runs
    //    the module, drops both at completion. This stub returns
    //    success with empty output bytes; the integration replaces
    //    the body with the real wasmtime invocation.
    //
    //    **wsa-g7a-mr-8 fix-pass tripwire:** in debug builds we
    //    `debug_assert!(cfg!(test), ...)` to abort if production code
    //    reaches the stub at runtime — a stalled G7-C that left the
    //    scaffold in place would silently report empty SANDBOX
    //    successes. The test gate carves out a path so the existing
    //    scaffold-flow tests still pass.
    //
    //    NOTE: This scaffold does NOT yet execute the wasm module.
    //    G7-C wires the full Store+Instance+invoke path. The
    //    return-value backstop (D17 BACKSTOP) runs at the engine
    //    integration boundary against the actual return-value bytes.
    #[allow(clippy::assertions_on_constants)]
    {
        debug_assert!(
            cfg!(test),
            "G7-A scaffold reached at runtime — production code path must replace this stub \
             with the wasmtime Store+Instance dispatch (G7-C). If this fires you have a \
             stalled G7-C wire-through; flip the implementation in this function before ship."
        );
    }

    Ok(SandboxResult {
        output: Vec::new(),
        fuel_consumed: 0,
        output_consumed: 0,
    })
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
        let module_bytes = wat::parse_str("(module)").expect("empty module compiles");
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
            "inline manifest with matching grant must succeed"
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
            assert!(
                name.contains("Phase 2c") || name.contains("deferred"),
                "operator hint MUST mention Phase 2c deferral; got: {name}"
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
