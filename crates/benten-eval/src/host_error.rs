//! Phase 2a G1-B: `HostError` — Option A shape per plan §9.2 + R1 triage.
//!
//! `HostError` is the host-boundary error surface that lets `PrimitiveHost`
//! implementations report storage / backend failures without forcing
//! `benten-eval` to depend on `benten-graph` (arch-1 dep-break). The struct
//! is FROZEN at Phase 2a close (plan §8 frozen-interfaces item 3).
//!
//! Wire format (sec-r1-6 / atk-6):
//! - `code` + `context` serialise onto the wire.
//! - `source` is `Box<dyn std::error::Error + Send + Sync>` and MUST NOT
//!   appear on the wire.
//!
//! TODO(phase-2a-G1-B): finish the wire encode/decode to use DAG-CBOR; the
//! stub below returns placeholder bytes.

use benten_errors::ErrorCode;

/// Host-boundary error: stable `code` discriminant + opaque `source` +
/// optional human `context`. See module docs.
pub struct HostError {
    /// Stable catalog code (on-wire).
    pub code: ErrorCode,
    /// Opaque error cause. Never serialised onto the wire.
    pub source: Box<dyn std::error::Error + Send + Sync>,
    /// Optional human-readable context (on-wire).
    pub context: Option<String>,
}

impl core::fmt::Debug for HostError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HostError")
            .field("code", &self.code)
            .field("context", &self.context)
            .field("source", &format_args!("<opaque>"))
            .finish()
    }
}

impl core::fmt::Display for HostError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.context {
            Some(c) => write!(f, "host error ({}): {}", self.code.as_str(), c),
            None => write!(f, "host error ({})", self.code.as_str()),
        }
    }
}

impl std::error::Error for HostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl HostError {
    /// Serialise to the wire bytes (code + optional context only, never
    /// `source`). Phase-2a stub.
    ///
    /// # Errors
    /// Returns `Err(HostError)` on encode failure.
    pub fn to_wire_bytes(&self) -> Result<Vec<u8>, HostError> {
        // Minimal wire format: `code_str\0context_str_or_empty`. The bytes
        // encode only stable public surface; `source` is intentionally
        // absent per sec-r1-6 / atk-6 wire-leak contract.
        //
        // TODO(phase-2a-G1-B): switch to DAG-CBOR with a versioned envelope
        // once the full host-error catalog is live.
        let mut out = Vec::new();
        out.extend_from_slice(self.code.as_str().as_bytes());
        out.push(0);
        if let Some(ctx) = &self.context {
            out.extend_from_slice(ctx.as_bytes());
        }
        Ok(out)
    }

    /// Decode from wire bytes. Phase-2a stub.
    ///
    /// # Errors
    /// Returns `Err(HostError)` on decode failure.
    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self, HostError> {
        let mut parts = bytes.splitn(2, |b| *b == 0);
        let code_bytes = parts.next().unwrap_or(&[]);
        let ctx_bytes = parts.next().unwrap_or(&[]);
        let code_str = core::str::from_utf8(code_bytes).unwrap_or("E_UNKNOWN");
        let code = ErrorCode::from_str(code_str);
        let context = if ctx_bytes.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(ctx_bytes).into_owned())
        };
        Ok(Self {
            code,
            source: Box::new(std::io::Error::other("decoded from wire; opaque source")),
            context,
        })
    }
}
