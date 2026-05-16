//! Typed-error-only network-fetch stub backend (origin: Phase-2b
//! G10-A-wasip1 / Phase-2a §9.8).
//!
//! ## What this is
//!
//! A loud-fail anchor on the [`KVBackend`] storage waist for the wasm32
//! runtime path that has no redb file to lean on. Every method surfaces a
//! typed error (`Phase3DeferredFetch` for reads / scans, `BackendReadOnly`
//! for writes) so any caller that wires the stub fails loud rather than
//! silently serving zeros (or — worse — a default-constructed empty result
//! that looks like a clean miss).
//!
//! ## Status (post `phase-3-close` / `phase-4-foundation-close`)
//!
//! This stub was originally minted in Phase-2b to reserve the
//! `KVBackend` shape for an anticipated Phase-3 iroh-fetch body-swap. That
//! swap **never happened**: P2P sync shipped in its own crate
//! (`benten-sync`, iroh + Loro directly) rather than by hot-swapping a
//! `KVBackend` implementation. The stub is retained as a typed-error
//! loud-fail anchor — any future code that tries to wire a network-backed
//! `KVBackend` into the engine before that path is genuinely built fails
//! loudly with a typed error instead of degrading silently. Whether to
//! retire the stub entirely (it has zero production callers) is tracked as
//! a v1-API-stabilization decision in
//! `docs/future/phase-4-backlog.md` (couples to the mode-(b)/(c)
//! light-client trait surface).

use benten_errors::ErrorCode;

use crate::backend::{KVBackend, ScanResult};

/// Phase-2b stub backend reserving the wire for the Phase-3 iroh-fetch
/// `KVBackend`.
///
/// All operations surface [`NetworkFetchStubError`] — the stub never
/// returns a successful read/write. See module-level docs for the
/// rationale.
#[derive(Debug, Default, Clone)]
pub struct NetworkFetchStubBackend {
    /// Optional human-readable name surfaced in the typed error so a
    /// caller wiring multiple stubs can tell them apart.
    label: Option<String>,
}

impl NetworkFetchStubBackend {
    /// Construct a stub backend with no label.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a stub backend with a human-readable label, surfaced in
    /// the typed error for diagnostic clarity.
    #[must_use]
    pub fn with_label(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
        }
    }

    /// The label this stub was constructed with, if any.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

/// Errors surfaced by [`NetworkFetchStubBackend`].
///
/// Two variants:
///
/// - `Phase3DeferredFetch` for reads + scans — a network-backed
///   `KVBackend` was never built (P2P sync shipped in `benten-sync`
///   instead); the stub fails loud rather than serving fabricated zeros.
/// - `ReadOnly` for writes — even when the Phase-3 implementation lands,
///   network-fetch is a read-only consume side; the upstream peer is the
///   write authority.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NetworkFetchStubError {
    /// Read or scan was attempted against the network-fetch stub. A
    /// network-backed `KVBackend` was never built — P2P sync shipped in
    /// the `benten-sync` crate (iroh + Loro) instead of via a
    /// `KVBackend` body-swap. Retire-vs-keep is tracked in
    /// `docs/future/phase-4-backlog.md` (v1-API-stabilization). This
    /// stays a typed failure rather than a silent zero.
    #[error("network-fetch KVBackend not implemented (stub{label}): {operation}")]
    Phase3DeferredFetch {
        /// Which method was called (`"get"`, `"scan"`).
        operation: &'static str,
        /// Optional label suffix `" (label=...)"` for diagnostic clarity.
        /// Constructed once on `from` so the message renders correctly
        /// without a custom Display.
        label: String,
    },
    /// Write was attempted against the network-fetch stub. A
    /// network-fetch path is a read-only consume side by design; the
    /// upstream peer is the write authority.
    #[error("backend is read-only: {operation} rejected (network-fetch-stub{label})")]
    ReadOnly {
        /// Which mutation method was called (`"put"`, `"delete"`,
        /// `"put_batch"`).
        operation: &'static str,
        /// Optional label suffix `" (label=...)"` for diagnostic clarity.
        label: String,
    },
}

impl NetworkFetchStubError {
    /// Stable [`ErrorCode`] for the variant. Reads route through
    /// `NotImplemented` because no network-backed `KVBackend` was built;
    /// writes route through `BackendReadOnly` because the network-fetch
    /// consume side is read-only by design.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            NetworkFetchStubError::Phase3DeferredFetch { .. } => ErrorCode::NotImplemented,
            NetworkFetchStubError::ReadOnly { .. } => ErrorCode::BackendReadOnly,
        }
    }
}

fn label_suffix(label: Option<&str>) -> String {
    match label {
        Some(l) => format!(" label={l}"),
        None => String::new(),
    }
}

impl KVBackend for NetworkFetchStubBackend {
    type Error = NetworkFetchStubError;

    fn get(&self, _key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        Err(NetworkFetchStubError::Phase3DeferredFetch {
            operation: "get",
            label: label_suffix(self.label.as_deref()),
        })
    }

    fn put(&self, _key: &[u8], _value: &[u8]) -> Result<(), Self::Error> {
        Err(NetworkFetchStubError::ReadOnly {
            operation: "put",
            label: label_suffix(self.label.as_deref()),
        })
    }

    fn delete(&self, _key: &[u8]) -> Result<(), Self::Error> {
        Err(NetworkFetchStubError::ReadOnly {
            operation: "delete",
            label: label_suffix(self.label.as_deref()),
        })
    }

    fn scan(&self, _prefix: &[u8]) -> Result<ScanResult, Self::Error> {
        Err(NetworkFetchStubError::Phase3DeferredFetch {
            operation: "scan",
            label: label_suffix(self.label.as_deref()),
        })
    }

    fn put_batch(&self, _pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), Self::Error> {
        Err(NetworkFetchStubError::ReadOnly {
            operation: "put_batch",
            label: label_suffix(self.label.as_deref()),
        })
    }

    /// Network-fetch stub — no local durable write path, no fsync
    /// semantic to honor. Surf-1 #860.
    fn supports_durability(&self) -> bool {
        false
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests + benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;

    /// `network_fetch_stub_returns_typed_error` — brief must-pass test #2.
    /// Every operation surfaces a typed `NetworkFetchStubError` (read /
    /// scan -> `Phase3DeferredFetch` -> `ErrorCode::NotImplemented`;
    /// writes -> `ReadOnly` -> `ErrorCode::BackendReadOnly`).
    #[test]
    fn network_fetch_stub_returns_typed_error() {
        let backend = NetworkFetchStubBackend::new();

        let get_err = backend.get(b"n:k").unwrap_err();
        assert_eq!(get_err.code(), ErrorCode::NotImplemented);
        assert!(matches!(
            get_err,
            NetworkFetchStubError::Phase3DeferredFetch { .. }
        ));

        let scan_err = backend.scan(b"").unwrap_err();
        assert_eq!(scan_err.code(), ErrorCode::NotImplemented);

        let put_err = backend.put(b"n:k", b"v").unwrap_err();
        assert_eq!(put_err.code(), ErrorCode::BackendReadOnly);

        let del_err = backend.delete(b"n:k").unwrap_err();
        assert_eq!(del_err.code(), ErrorCode::BackendReadOnly);

        let batch_err = backend
            .put_batch(&[(b"n:k".to_vec(), b"v".to_vec())])
            .unwrap_err();
        assert_eq!(batch_err.code(), ErrorCode::BackendReadOnly);
    }

    #[test]
    fn label_surfaces_in_error_message() {
        let backend = NetworkFetchStubBackend::with_label("peer-A");
        let err = backend.get(b"n:k").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("peer-A"), "label must surface in error: {msg}");
    }

    #[test]
    fn unlabelled_stub_does_not_inject_label_text() {
        let backend = NetworkFetchStubBackend::new();
        let err = backend.get(b"n:k").unwrap_err();
        let msg = format!("{err}");
        assert!(
            !msg.contains("label="),
            "unlabelled stub must not render `label=`: {msg}"
        );
    }
}
