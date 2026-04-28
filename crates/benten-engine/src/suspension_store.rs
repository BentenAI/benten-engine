//! Phase-2b G12-E — engine-side [`SuspensionStore`] adapter.
//!
//! The trait + in-memory reference impl live in
//! [`benten_eval::suspension_store`] (benten-eval is by-design dep-broken
//! from benten-graph per arch-1 / phil-r1-2). This module wires the
//! engine's existing `Arc<RedbBackend>` into a [`RedbSuspensionStore`]
//! that durably persists the three suspension namespaces (WAIT
//! metadata, envelope bytes, SUBSCRIBE persistent cursors) under
//! reserved key prefixes inside the engine's redb file.
//!
//! # Key schema
//!
//! All entries land in the engine's existing `NODES_TABLE` (the redb
//! `KVBackend` impl exposes a single `(key, value)` table). Three
//! reserved prefixes namespace the three logical surfaces:
//!
//! - `"sw:" ++ cid_bytes` — WAIT metadata side-table value
//!   (DAG-CBOR-encoded `SerializableWaitMetadata` — module-private).
//! - `"se:" ++ cid_bytes` — `ExecutionStateEnvelope` bytes
//!   (canonical DAG-CBOR — same encoding `to_dagcbor` produces).
//! - `"sc:" ++ subscriber_cid_bytes` — SUBSCRIBE persistent cursor
//!   value (`u64` `max_delivered_seq` little-endian).
//!
//! The prefixes are disjoint from the existing `n:`, `e:`, `es:`, `et:`,
//! `s:` Node / Edge / Subgraph prefixes (see
//! `benten_graph::store::{NODE_PREFIX,...}`). Collision-freedom is pinned
//! by `suspension_store_handles_both_wait_and_cursor_keys_without_collision`.
//!
//! # Cross-process resume
//!
//! When the operator opens a fresh `Engine` against the same on-disk
//! redb path the suspending engine wrote, the `RedbSuspensionStore`
//! `get_wait` / `get_envelope` / `get_cursor` paths surface the
//! suspended entries unchanged. This closes the Phase-2a Compromise
//! #10 cross-process resume gap (`docs/SECURITY-POSTURE.md`).

use std::sync::Arc;

use benten_core::{Cid, SubscriberId, Value};
use benten_eval::ExecutionStateEnvelope;
use benten_eval::suspension_store::{
    SuspensionKey, SuspensionStore, SuspensionStoreError, WaitMetadata,
};
use benten_graph::{KVBackend, RedbBackend};
use serde::{Deserialize, Serialize};

const WAIT_PREFIX: &[u8] = b"sw:";
const ENVELOPE_PREFIX: &[u8] = b"se:";
const CURSOR_PREFIX: &[u8] = b"sc:";

fn wait_key(cid: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(WAIT_PREFIX.len() + cid.as_bytes().len());
    k.extend_from_slice(WAIT_PREFIX);
    k.extend_from_slice(cid.as_bytes());
    k
}

fn envelope_key(cid: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(ENVELOPE_PREFIX.len() + cid.as_bytes().len());
    k.extend_from_slice(ENVELOPE_PREFIX);
    k.extend_from_slice(cid.as_bytes());
    k
}

fn cursor_key(sub: &SubscriberId) -> Vec<u8> {
    let cid = sub.as_cid();
    let mut k = Vec::with_capacity(CURSOR_PREFIX.len() + cid.as_bytes().len());
    k.extend_from_slice(CURSOR_PREFIX);
    k.extend_from_slice(cid.as_bytes());
    k
}

// ---------------------------------------------------------------------------
// On-disk WAIT metadata serialization
// ---------------------------------------------------------------------------

/// DAG-CBOR-serialisable mirror of [`WaitMetadata`]. Lives here rather
/// than in benten-eval so the trait stays serde-free at its public
/// surface (the in-memory variant doesn't need encoding); the engine
/// adapter pays the encoding cost on the redb boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableWaitMetadata {
    suspend_elapsed_ms: Option<u64>,
    timeout_ms: Option<u64>,
    /// Optional signal shape. Encoded as DAG-CBOR-able `Value`. Phase-2a
    /// shipped untyped `Value`-shape; the encoder serialises whatever
    /// the WAIT primitive captured at suspend time.
    signal_shape: Option<Value>,
    is_duration: bool,
}

impl From<WaitMetadata> for SerializableWaitMetadata {
    fn from(m: WaitMetadata) -> Self {
        Self {
            suspend_elapsed_ms: m.suspend_elapsed_ms,
            timeout_ms: m.timeout_ms,
            signal_shape: m.signal_shape,
            is_duration: m.is_duration,
        }
    }
}

impl From<SerializableWaitMetadata> for WaitMetadata {
    fn from(m: SerializableWaitMetadata) -> Self {
        Self {
            suspend_elapsed_ms: m.suspend_elapsed_ms,
            timeout_ms: m.timeout_ms,
            signal_shape: m.signal_shape,
            is_duration: m.is_duration,
        }
    }
}

// ---------------------------------------------------------------------------
// RedbSuspensionStore
// ---------------------------------------------------------------------------

/// redb-backed [`SuspensionStore`] adapter over the engine's existing
/// `Arc<RedbBackend>`.
///
/// Reuses the engine's redb file rather than opening a sibling DB — one
/// `Engine::drop` releases all suspension state alongside graph state,
/// and the operator does not have to manage two paths.
pub struct RedbSuspensionStore {
    backend: Arc<RedbBackend>,
}

impl RedbSuspensionStore {
    /// Construct a redb-backed store over an existing
    /// `Arc<RedbBackend>`. Borrowed via `Arc::clone` so the engine's
    /// own backend handle can stay live.
    #[must_use]
    pub fn new(backend: Arc<RedbBackend>) -> Self {
        Self { backend }
    }
}

fn backend_err<E: std::fmt::Display>(e: E) -> SuspensionStoreError {
    SuspensionStoreError::Backend(e.to_string())
}

impl SuspensionStore for RedbSuspensionStore {
    fn put_wait(&self, cid: Cid, meta: WaitMetadata) -> Result<(), SuspensionStoreError> {
        let payload: SerializableWaitMetadata = meta.into();
        let bytes = serde_ipld_dagcbor::to_vec(&payload).map_err(backend_err)?;
        self.backend
            .put(&wait_key(&cid), &bytes)
            .map_err(backend_err)
    }

    fn get_wait(&self, cid: &Cid) -> Result<Option<WaitMetadata>, SuspensionStoreError> {
        let Some(bytes) = self.backend.get(&wait_key(cid)).map_err(backend_err)? else {
            return Ok(None);
        };
        let parsed: SerializableWaitMetadata =
            serde_ipld_dagcbor::from_slice(&bytes).map_err(backend_err)?;
        Ok(Some(parsed.into()))
    }

    fn put_envelope(&self, envelope: ExecutionStateEnvelope) -> Result<(), SuspensionStoreError> {
        let cid = envelope.payload_cid;
        let bytes = envelope.to_dagcbor().map_err(backend_err)?;
        self.backend
            .put(&envelope_key(&cid), &bytes)
            .map_err(backend_err)
    }

    fn get_envelope(
        &self,
        cid: &Cid,
    ) -> Result<Option<ExecutionStateEnvelope>, SuspensionStoreError> {
        let Some(bytes) = self.backend.get(&envelope_key(cid)).map_err(backend_err)? else {
            return Ok(None);
        };
        let envelope = ExecutionStateEnvelope::from_dagcbor(&bytes).map_err(backend_err)?;
        Ok(Some(envelope))
    }

    fn put_cursor(
        &self,
        sub: &SubscriberId,
        max_delivered_seq: u64,
    ) -> Result<(), SuspensionStoreError> {
        let bytes = max_delivered_seq.to_le_bytes();
        self.backend
            .put(&cursor_key(sub), &bytes)
            .map_err(backend_err)
    }

    fn get_cursor(&self, sub: &SubscriberId) -> Result<Option<u64>, SuspensionStoreError> {
        let Some(bytes) = self.backend.get(&cursor_key(sub)).map_err(backend_err)? else {
            return Ok(None);
        };
        if bytes.len() != 8 {
            return Err(SuspensionStoreError::Backend(format!(
                "cursor entry has wrong length {} (expected 8)",
                bytes.len()
            )));
        }
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes);
        Ok(Some(u64::from_le_bytes(buf)))
    }

    fn delete(&self, key: SuspensionKey) -> Result<(), SuspensionStoreError> {
        let raw = match key {
            SuspensionKey::WaitMetadata(cid) => wait_key(&cid),
            SuspensionKey::Envelope(cid) => envelope_key(&cid),
            SuspensionKey::Cursor(sub) => cursor_key(&sub),
        };
        self.backend.delete(&raw).map_err(backend_err)
    }
}
