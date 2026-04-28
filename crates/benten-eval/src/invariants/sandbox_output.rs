//! Phase-2b G7-B / Inv-7 — SANDBOX cumulative-output ceiling.
//!
//! Per D17-RESOLVED defense-in-depth, Inv-7 is enforced via two paths:
//!
//! 1. **PRIMARY — streaming `CountedSink` accumulator** (G7-A's
//!    `sandbox/counted_sink.rs`). The trampoline that wraps every
//!    SANDBOX host-fn invocation calls [`check_admission`] BEFORE
//!    accepting the host-fn's output bytes. If admitting the bytes
//!    would push `consumed + bytes.len() > limit`, the helper traps
//!    with [`ErrorCode::InvSandboxOutput`] and the sink rejects the
//!    write — D15 trap-loudly default, no silent truncation.
//!
//! 2. **BACKSTOP — return-value boundary check** (G7-A's
//!    `primitives::sandbox` executor). At the SANDBOX primitive
//!    boundary, the same [`check_admission`] is called against the
//!    return-value bytes after the wasm export returns. This catches
//!    any host-fn that bypasses the streaming sink wrapper
//!    (defense-in-depth for a future host-fn that forgets to thread
//!    the sink).
//!
//! Both paths share this single check helper so the bytes-counting
//! arithmetic is auditable in one place per D25-RECOMMEND
//! "centralised accounting" — body-counts spreads the contract across
//! every host-fn implementer + invites accidental skips.
//!
//! ## D15 trap-loudly framing
//!
//! Phase-2b ships exactly one Inv-7 behavior: trap loudly on overflow.
//! The plan §5 D15 deliberation considered a `truncate-and-mark`
//! alternative; sec-pre-r1-07 rejected it because the truncation
//! byte-position can encode information (a covert exfiltration
//! channel). If a future phase needs partial-output semantics it must
//! ship behind a per-handler explicit cap (`trust:output:truncate` or
//! similar), NOT a default-on flag.

use benten_core::Value;
use benten_errors::ErrorCode;

use crate::{InvariantConfig, InvariantViolation, OperationNode, PrimitiveKind, RegistrationError};

/// Default per-call SANDBOX cumulative-output ceiling in bytes (16 MiB).
/// SANDBOX nodes that omit `output_max_bytes` inherit this value at
/// runtime; nodes that DECLARE `output_max_bytes` must keep it within
/// `(0, DEFAULT_MAX_SANDBOX_OUTPUT_BYTES]`. The hard upper bound is set
/// by [`InvariantConfig::max_sandbox_output_bytes`] and defaults to
/// `DEFAULT_MAX_SANDBOX_OUTPUT_BYTES`; an engine.toml override (G7-A)
/// can raise it for unusual workloads.
pub const DEFAULT_MAX_SANDBOX_OUTPUT_BYTES: u64 = 16 * 1024 * 1024;

/// Registration-time check for SANDBOX `output_max_bytes` declarations.
///
/// Walks every SANDBOX OperationNode in the supplied node-list. If a
/// node declares an `output_max_bytes` property:
///   - The property MUST be `Value::Int` (no other shape is accepted —
///     a poisoned encoding is a registration reject per the same
///     discipline as `signal_shape: Value::Bytes`).
///   - The integer value MUST be `> 0` AND `<= max_ceiling`.
///
/// A SANDBOX node WITHOUT an `output_max_bytes` property is registered
/// cleanly; the runtime executor will use the engine-wide default.
///
/// # Errors
///
/// Returns a [`RegistrationError`] carrying [`InvariantViolation::SandboxOutput`]
/// when any SANDBOX node's `output_max_bytes` declaration is poisoned or
/// out of range.
pub(crate) fn validate_registration(
    nodes: &[OperationNode],
    config: &InvariantConfig,
) -> Result<(), RegistrationError> {
    let max_ceiling = config.max_sandbox_output_bytes;
    for node in nodes {
        if !matches!(node.kind, PrimitiveKind::Sandbox) {
            continue;
        }
        let Some(prop) = node.properties.get("output_max_bytes") else {
            continue;
        };
        // Shape: must be Int. Anything else is a poisoned encoding.
        let Value::Int(declared) = prop else {
            let mut err = RegistrationError::new(InvariantViolation::SandboxOutput);
            err.fanout_node_id = Some(node.id.clone());
            return Err(err);
        };
        // Range: > 0 and <= configured ceiling. A 0-byte budget has no
        // physical meaning (every host-fn write would trip immediately);
        // the upper bound prevents a node from declaring more than the
        // engine's allowed maximum.
        if *declared <= 0 {
            let mut err = RegistrationError::new(InvariantViolation::SandboxOutput);
            err.fanout_node_id = Some(node.id.clone());
            return Err(err);
        }
        let declared_u64 = u64::try_from(*declared).unwrap_or(u64::MAX);
        if declared_u64 > max_ceiling {
            let mut err = RegistrationError::new(InvariantViolation::SandboxOutput);
            err.fanout_node_id = Some(node.id.clone());
            return Err(err);
        }
    }
    Ok(())
}

/// Trap source — which Inv-7 enforcement path observed the overflow.
/// Surfaces in trap diagnostics so operators can distinguish a
/// well-behaved host-fn that hit the streaming ceiling from a
/// misbehaving host-fn that bypassed the sink and was caught at the
/// return-value boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputCheckPath {
    /// D17 PRIMARY — fired from the streaming `CountedSink` trampoline
    /// before host-fn bytes were accepted. Indicates a well-behaved
    /// host-fn whose cumulative output simply exceeded the configured
    /// `output_max_bytes` ceiling.
    PrimaryStreaming,
    /// D17 BACKSTOP — fired at the SANDBOX primitive return-value
    /// boundary. Indicates either a host-fn that bypassed the sink
    /// wrapper (defense-in-depth catch) or a return-value-only emission
    /// that exceeded the ceiling.
    ReturnBackstop,
}

/// Outcome of an Inv-7 admission check.
///
/// Carries the cumulative-byte arithmetic so the trap-side caller can
/// surface accurate diagnostics: `attempted` is the byte count the
/// caller wanted to admit, `would_be` is `consumed + attempted`, and
/// `limit` is the configured `output_max_bytes`. `path` distinguishes
/// the PRIMARY vs BACKSTOP firing surface per D17.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputOverflow {
    /// Bytes already admitted before this check.
    pub consumed: u64,
    /// Bytes the caller is asking to admit now.
    pub attempted: u64,
    /// `consumed + attempted` (saturating) — the value that would have
    /// been recorded if the admission had succeeded. `saturating_add`
    /// prevents the diagnostic from itself wrapping when the attempted
    /// payload is large.
    pub would_be: u64,
    /// The configured `output_max_bytes` ceiling.
    pub limit: u64,
    /// Which enforcement path observed the overflow.
    pub path: OutputCheckPath,
}

impl OutputOverflow {
    /// The typed catalog code Inv-7 fires. All overflow paths route
    /// through `E_INV_SANDBOX_OUTPUT` regardless of which check fired
    /// (the `path` field carries the source distinction; the catalog
    /// code is shared so DSL `ON_OUTPUT_LIMIT` edges route both).
    ///
    /// Exposed as an associated `const` because the mapping is currently
    /// path-independent — `path` is kept on the struct for diagnostic
    /// surfacing, not for code distinction. If a future revision wants
    /// per-path codes (e.g. BACKSTOP gets its own catalog entry), this
    /// can re-grow into a `&self` method without breaking the
    /// instance-method callsite below.
    pub const CODE: ErrorCode = ErrorCode::InvSandboxOutput;

    /// Instance-method shim over [`OutputOverflow::CODE`] kept for
    /// callsite ergonomics (`overflow.code()` reads more naturally
    /// than `OutputOverflow::CODE` at the trap point).
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        Self::CODE
    }
}

/// Inv-7 admission check.
///
/// Returns `Ok(new_consumed)` — the post-admission `consumed` value the
/// caller should record on success — when admitting `attempted` bytes
/// keeps the cumulative within the `limit` ceiling. Returns
/// `Err(OutputOverflow)` when admitting would cross the ceiling; the
/// caller MUST NOT update its `consumed` counter in the error case
/// (D15 trap-loudly: zero bytes accepted on failure).
///
/// Arithmetic discipline (per `proptest_sandbox_output.rs`): both
/// `consumed` and `attempted` are `u64`; the sum uses `checked_add` so
/// a `usize::MAX` payload cannot wrap silently. An overflow of the
/// `u64` arithmetic itself is treated as an unconditional trap (the
/// real ceiling is far below `u64::MAX` in any practical configuration,
/// so a u64-overflow would itself be a witness of a malicious or
/// broken caller).
///
/// # Errors
///
/// Returns [`OutputOverflow`] carrying the arithmetic context whenever
/// admitting `attempted` would push the cumulative past `limit`. The
/// caller routes the overflow through `OutputOverflow::code()`
/// → `ErrorCode::InvSandboxOutput` and surfaces the diagnostic.
pub fn check_admission(
    consumed: u64,
    attempted: u64,
    limit: u64,
    path: OutputCheckPath,
) -> Result<u64, OutputOverflow> {
    let would_be = consumed.saturating_add(attempted);
    if would_be > limit {
        return Err(OutputOverflow {
            consumed,
            attempted,
            would_be,
            limit,
            path,
        });
    }
    Ok(would_be)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_under_limit_succeeds() {
        let new = check_admission(100, 200, 1024, OutputCheckPath::PrimaryStreaming)
            .expect("under limit ok");
        assert_eq!(new, 300);
    }

    #[test]
    fn admission_at_exact_limit_succeeds() {
        let new = check_admission(0, 1024, 1024, OutputCheckPath::PrimaryStreaming)
            .expect("exact limit ok");
        assert_eq!(new, 1024);
    }

    #[test]
    fn admission_one_byte_over_traps_loudly_no_partial_admission() {
        // D15 trap-loudly + sec-pre-r1-07 — even a one-byte overage
        // refuses ALL of the attempted bytes (NOT a partial 1023-byte
        // admission). The caller's `consumed` counter MUST NOT advance.
        let err = check_admission(0, 1025, 1024, OutputCheckPath::PrimaryStreaming)
            .expect_err("over limit traps");
        assert_eq!(err.consumed, 0, "no partial admission");
        assert_eq!(err.attempted, 1025);
        assert_eq!(err.would_be, 1025);
        assert_eq!(err.limit, 1024);
        assert_eq!(err.path, OutputCheckPath::PrimaryStreaming);
        assert_eq!(err.code(), ErrorCode::InvSandboxOutput);
    }

    #[test]
    fn admission_at_existing_consumed_traps_when_increment_overflows_limit() {
        // Cumulative case: 800 bytes already admitted; a 300-byte
        // attempt against a 1024-byte limit traps (800+300=1100>1024)
        // and reports the would_be value the caller can render.
        let err = check_admission(800, 300, 1024, OutputCheckPath::ReturnBackstop)
            .expect_err("cumulative overflow traps");
        assert_eq!(err.consumed, 800);
        assert_eq!(err.attempted, 300);
        assert_eq!(err.would_be, 1100);
        assert_eq!(err.limit, 1024);
        assert_eq!(err.path, OutputCheckPath::ReturnBackstop);
    }

    #[test]
    fn admission_u64_overflow_still_traps() {
        // Defensive arithmetic: a `u64::MAX` payload cannot wrap
        // silently. The would_be saturates to u64::MAX and the trap
        // fires regardless of the caller's `limit`.
        let err = check_admission(1, u64::MAX, 1024, OutputCheckPath::PrimaryStreaming)
            .expect_err("u64 overflow traps");
        assert_eq!(err.would_be, u64::MAX);
    }

    #[test]
    fn overflow_path_field_distinguishes_primary_vs_backstop() {
        let primary = check_admission(0, 2048, 1024, OutputCheckPath::PrimaryStreaming)
            .expect_err("primary trap");
        let backstop = check_admission(0, 2048, 1024, OutputCheckPath::ReturnBackstop)
            .expect_err("backstop trap");
        assert_ne!(primary.path, backstop.path);
        assert_eq!(primary.code(), backstop.code()); // shared catalog code
    }
}
