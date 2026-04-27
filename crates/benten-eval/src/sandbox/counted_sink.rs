//! Streaming output-byte accumulator (Phase 2b G7-A).
//!
//! D17-RESOLVED defense-in-depth — Inv-7 (SANDBOX output limit) enforcement
//! has TWO live paths:
//!   - **PRIMARY**: streaming [`CountedSink`] wraps every host-fn write.
//!     Per the D25 trampoline pattern, the trampoline calls
//!     [`CountedSink::write`] AFTER the host-fn body returns its output
//!     bytes; the sink checks `consumed + bytes.len() > limit` BEFORE
//!     accepting bytes and traps with [`SinkOverflow`].
//!   - **BACKSTOP**: return-value path runs the SAME check at primitive
//!     boundary, catching any host-fn that bypasses the streaming sink
//!     (test-only `testing_register_uncounted_host_fn` helper exercises
//!     this).
//!
//! The `path` field on [`SinkOverflow`] (`"primary_streaming"` /
//! `"return_backstop"`) lets tests + audit logs distinguish which path
//! caught the violation.
//!
//! References [`crate::chunk_sink::ChunkSink`] (G6-A scaffold trait) for
//! the future composition with STREAM primitive output (per arch-pre-r1-9
//! G7-A scope addition). Today the [`CountedSink`] is sink-shaped but
//! does not yet route bytes to a [`crate::chunk_sink::ChunkSink`]
//! implementation — that wiring lands when STREAM-into-SANDBOX
//! composition is exercised in G11-2b.

use crate::chunk_sink::ChunkSink;
use benten_errors::ErrorCode;

/// Defense-in-depth detection path. Recorded on every [`SinkOverflow`]
/// so tests + audit logs can distinguish which D17 path caught the
/// violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowPath {
    /// Caught by the PRIMARY streaming check (D17 PRIMARY).
    PrimaryStreaming,
    /// Caught by the BACKSTOP return-value check at primitive boundary
    /// (D17 BACKSTOP — defense-in-depth for misbehaving host-fns).
    ReturnBackstop,
}

impl OverflowPath {
    /// Static string form for trace-step + test-side equality assertions.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            OverflowPath::PrimaryStreaming => "primary_streaming",
            OverflowPath::ReturnBackstop => "return_backstop",
        }
    }
}

/// Trapped-overflow payload. Routes to [`ErrorCode::InvSandboxOutput`]
/// (`E_INV_SANDBOX_OUTPUT`) at the primitive boundary.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "SANDBOX output budget exceeded: consumed={consumed} limit={limit} \
     emitter={emitter_kind} path={}",
    path.as_str()
)]
pub struct SinkOverflow {
    /// Bytes accumulated before the overflowing write.
    pub consumed: u64,
    /// Configured per-call cumulative limit.
    pub limit: u64,
    /// Tag for the overflowing emitter (e.g. `"host_fn:compute:log"`,
    /// `"return_value"`).
    pub emitter_kind: String,
    /// Which D17 path caught the overflow.
    pub path: OverflowPath,
}

impl SinkOverflow {
    /// Stable catalog code for routing.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        ErrorCode::InvSandboxOutput
    }
}

/// Per-call cumulative-byte counter for SANDBOX output (D17 PRIMARY).
///
/// Constructed once per primitive call by the SANDBOX executor, threaded
/// through the host-fn trampoline via [`super::host_fns::HostFnContext`].
/// At primitive boundary, the executor calls [`Self::backstop_check`]
/// with the return-value bytes to exercise the BACKSTOP path.
#[derive(Debug)]
pub struct CountedSink {
    consumed: u64,
    limit: u64,
}

impl CountedSink {
    /// Construct a sink with the given per-call cumulative-byte limit.
    #[must_use]
    pub fn new(limit: u64) -> Self {
        Self { consumed: 0, limit }
    }

    /// Bytes accumulated so far.
    #[must_use]
    pub fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Configured limit.
    #[must_use]
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// PRIMARY path — accept `bytes` if doing so would not exceed the
    /// configured limit. Otherwise return `Err(SinkOverflow { path:
    /// PrimaryStreaming, .. })`.
    ///
    /// Boundary semantics (wsa D17 boundary): `consumed + bytes.len()
    /// == limit` succeeds; `consumed + bytes.len() > limit` traps.
    ///
    /// # Errors
    /// Returns `Err(SinkOverflow)` when the cumulative byte count
    /// would exceed the limit.
    pub fn write(&mut self, bytes: &[u8], emitter_kind: &str) -> Result<(), SinkOverflow> {
        let n = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        let next = self.consumed.saturating_add(n);
        if next > self.limit {
            return Err(SinkOverflow {
                consumed: self.consumed,
                limit: self.limit,
                emitter_kind: emitter_kind.to_string(),
                path: OverflowPath::PrimaryStreaming,
            });
        }
        self.consumed = next;
        Ok(())
    }

    /// BACKSTOP path — at primitive boundary, check the return-value
    /// bytes against the limit. Catches host-fns that bypassed the
    /// streaming sink (test-only fixture).
    ///
    /// # Errors
    /// Returns `Err(SinkOverflow { path: ReturnBackstop, .. })` when the
    /// cumulative byte count after the return-value would exceed the
    /// limit.
    pub fn backstop_check(
        &self,
        return_value_bytes: u64,
        emitter_kind: &str,
    ) -> Result<(), SinkOverflow> {
        let next = self.consumed.saturating_add(return_value_bytes);
        if next > self.limit {
            return Err(SinkOverflow {
                consumed: self.consumed,
                limit: self.limit,
                emitter_kind: emitter_kind.to_string(),
                path: OverflowPath::ReturnBackstop,
            });
        }
        Ok(())
    }
}

/// CountedSink also implements the G6-A scaffold trait so STREAM-into-
/// SANDBOX composition can route chunks through the per-call output
/// budget. The marker impl currently has no methods (G6-A scaffold trait
/// body is empty); G6-A's `tokio::sync::mpsc` impl will name a `send`
/// method that this `impl` extends. Until then this `impl` is the shape
/// pin — moving the trait surface in G6-A surfaces here as a
/// compile-error.
impl ChunkSink for CountedSink {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_within_limit_accepts() {
        let mut sink = CountedSink::new(100);
        assert!(sink.write(b"hello", "host_fn:compute:log").is_ok());
        assert_eq!(sink.consumed(), 5);
    }

    #[test]
    fn write_at_exact_limit_succeeds() {
        // wsa D17 boundary — consumed == limit succeeds.
        let mut sink = CountedSink::new(5);
        assert!(sink.write(b"hello", "host_fn:compute:log").is_ok());
        assert_eq!(sink.consumed(), 5);
    }

    #[test]
    fn write_exceeds_limit_traps_primary_path() {
        // wsa D17 boundary — consumed > limit traps; path == primary.
        let mut sink = CountedSink::new(5);
        let err = sink
            .write(b"helloworld", "host_fn:compute:log")
            .unwrap_err();
        assert_eq!(err.path, OverflowPath::PrimaryStreaming);
        assert_eq!(err.code(), ErrorCode::InvSandboxOutput);
        assert_eq!(
            sink.consumed(),
            0,
            "rejected write must not advance counter"
        );
    }

    #[test]
    fn aggregate_across_writes_traps_when_exceeded() {
        // wsa-1 — N successive writes accumulate against the same budget.
        let mut sink = CountedSink::new(10);
        sink.write(b"abcde", "host_fn:compute:log").unwrap();
        let err = sink.write(b"fghijk", "host_fn:compute:log").unwrap_err();
        assert_eq!(err.consumed, 5);
        assert_eq!(err.limit, 10);
    }

    #[test]
    fn backstop_check_traps_with_distinct_path() {
        let sink = CountedSink::new(5);
        let err = sink.backstop_check(10, "return_value").unwrap_err();
        assert_eq!(err.path, OverflowPath::ReturnBackstop);
    }

    #[test]
    fn overflow_path_str_pinned() {
        assert_eq!(OverflowPath::PrimaryStreaming.as_str(), "primary_streaming");
        assert_eq!(OverflowPath::ReturnBackstop.as_str(), "return_backstop");
    }
}
