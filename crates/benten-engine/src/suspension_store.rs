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
//!   (canonical DAG-CBOR — same encoding `to_canonical_bytes` produces).
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
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use benten_core::{Cid, SubscriberId, Value};
use benten_eval::ExecutionStateEnvelope;
use benten_eval::suspension_store::{
    CapSnapshot, SuspensionKey, SuspensionStore, SuspensionStoreError, WaitMetadata,
};
use benten_graph::{KVBackend, RedbBackend};
use serde::{Deserialize, Serialize};

const WAIT_PREFIX: &[u8] = b"sw:";
const ENVELOPE_PREFIX: &[u8] = b"se:";
const CURSOR_PREFIX: &[u8] = b"sc:";
/// Phase-3 G17-A2 (phase-3-backlog §6.5 + r1-wsa-10) — SUBSCRIBE
/// persistent-cursor metadata key prefix. Stores per-subscriber
/// `delivered_count` + `registered_at_unix_secs` so the
/// `is_retention_exhausted` check works across engine re-opens.
/// Disjoint from the existing eight prefixes (`n:`, `e:`, `es:`, `et:`,
/// `s:`, `sw:`, `se:`, `sc:`, `sx:`) per the collision-freedom contract.
const CURSOR_META_PREFIX: &[u8] = b"sm:";
/// Phase-3 G17-A2 — singleton key for the durable retention-window
/// override (per r1-wsa-10 persistence pin). The override is global
/// to the store; reading at `is_retention_exhausted` time + writing at
/// `set_retention_window`. Encoded as DAG-CBOR
/// [`PersistedRetentionWindow`].
const RETENTION_WINDOW_KEY: &[u8] = b"sr:retention_window";
/// G14-D wave-5a: cap-snapshot key prefix. Disjoint from the other
/// eight prefixes (`n:`, `e:`, `es:`, `et:`, `s:`, `sw:`, `se:`, `sc:`)
/// per the collision-freedom contract pinned at
/// `suspension_store_handles_both_wait_and_cursor_keys_without_collision`
/// and extended at `suspension_store_handles_cap_snapshot_key_without_collision_with_wait_or_cursor`.
const CAP_SNAPSHOT_PREFIX: &[u8] = b"sx:";

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

fn cap_snapshot_key(envelope_cid: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(CAP_SNAPSHOT_PREFIX.len() + envelope_cid.as_bytes().len());
    k.extend_from_slice(CAP_SNAPSHOT_PREFIX);
    k.extend_from_slice(envelope_cid.as_bytes());
    k
}

/// Phase-3 G17-A2 (§6.5) — per-subscriber cursor-metadata key.
fn cursor_meta_key(sub: &SubscriberId) -> Vec<u8> {
    let cid = sub.as_cid();
    let mut k = Vec::with_capacity(CURSOR_META_PREFIX.len() + cid.as_bytes().len());
    k.extend_from_slice(CURSOR_META_PREFIX);
    k.extend_from_slice(cid.as_bytes());
    k
}

/// Phase-3 G17-A2 (§6.5) — per-subscriber retention metadata: tracks
/// when the cursor was first registered (so age-based windows can
/// fire) + how many events have been delivered (so count-based windows
/// can fire). Encoded DAG-CBOR; persisted under `sm:<sub_cid>`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PersistedCursorMeta {
    /// First-registration UNIX-seconds wall-clock; used by age-based
    /// retention windows.
    registered_at_unix_secs: u64,
    /// Cumulative delivered-event count; used by count-based retention
    /// windows (Phase-2b documented 1000-event ceiling).
    delivered_count: u64,
}

/// Phase-3 G17-A2 (§6.5 + r1-wsa-10) — durable retention-window
/// override. `Some(d)` means a custom override is in effect; `None`
/// (the absence of the singleton key) means default semantics
/// (`is_retention_exhausted` returns `false`, the trait-default).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedRetentionWindow {
    /// Window duration in milliseconds. A cursor whose
    /// `registered_at_unix_secs` is older than NOW − window_ms is
    /// retention-exhausted.
    window_ms: u64,
}

/// G14-D wave-5a: DAG-CBOR-serialisable mirror of [`CapSnapshot`].
/// Stored in the redb side-table under `sx:<envelope_cid>` so a
/// cross-process resume can re-validate the bound UCAN-proof-chain
/// hash + historical-policy metadata against the live cap store.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableCapSnapshot {
    cap_snapshot_hash: [u8; 32],
    historical_policy_metadata: Vec<u8>,
}

impl From<CapSnapshot> for SerializableCapSnapshot {
    fn from(s: CapSnapshot) -> Self {
        Self {
            cap_snapshot_hash: s.cap_snapshot_hash,
            historical_policy_metadata: s.historical_policy_metadata,
        }
    }
}

impl From<SerializableCapSnapshot> for CapSnapshot {
    fn from(s: SerializableCapSnapshot) -> Self {
        Self {
            cap_snapshot_hash: s.cap_snapshot_hash,
            historical_policy_metadata: s.historical_policy_metadata,
        }
    }
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
    /// Phase-3 G20-A2 (D12 wave-8a): TTL hours from the WAIT spec. Optional
    /// for forward / backward compatibility — DAG-CBOR de-encoder treats
    /// missing keys as `None` so a Phase-2b-shape on-disk entry parses
    /// cleanly under the Phase-3 schema.
    #[serde(default)]
    ttl_hours: Option<u32>,
    /// Phase-3 G20-A2 (D12 wave-8a): wall-clock anchor for the TTL.
    #[serde(default)]
    suspend_wallclock_ms: Option<u64>,
}

impl From<WaitMetadata> for SerializableWaitMetadata {
    fn from(m: WaitMetadata) -> Self {
        Self {
            suspend_elapsed_ms: m.suspend_elapsed_ms,
            timeout_ms: m.timeout_ms,
            signal_shape: m.signal_shape,
            is_duration: m.is_duration,
            ttl_hours: m.ttl_hours,
            suspend_wallclock_ms: m.suspend_wallclock_ms,
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
            ttl_hours: m.ttl_hours,
            suspend_wallclock_ms: m.suspend_wallclock_ms,
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

    /// Phase-3 G17-A2 (phase-3-backlog §6.5) — convenience constructor
    /// that opens (or creates) a redb file at `path` and wraps it in a
    /// `RedbSuspensionStore`. Each call hands back an independent
    /// `Arc<RedbBackend>`; for the production engine path, prefer
    /// [`Self::new`] over an existing `Arc<RedbBackend>` so suspension
    /// state lives alongside graph state in the same redb file.
    ///
    /// # Errors
    /// Surfaces [`SuspensionStoreError::Backend`] on file-open failure.
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, SuspensionStoreError> {
        let backend = RedbBackend::create(path).map_err(backend_err)?;
        Ok(Self {
            backend: Arc::new(backend),
        })
    }

    /// Phase-3 G17-A2 (§6.5 + r1-wsa-10 persistent-state pin) — set the
    /// SUBSCRIBE persistent-cursor retention window. Persisted in the
    /// redb side-table; survives engine close + re-open.
    ///
    /// Setting the window to `Duration::ZERO` is treated as "every
    /// cursor is immediately retention-exhausted" (operator force-
    /// exhaust escape hatch).
    ///
    /// # Errors
    /// Surfaces [`SuspensionStoreError::Backend`] on persistence failure.
    pub fn set_retention_window(&self, window: Duration) -> Result<(), SuspensionStoreError> {
        let payload = PersistedRetentionWindow {
            window_ms: u64::try_from(window.as_millis()).unwrap_or(u64::MAX),
        };
        let bytes = serde_ipld_dagcbor::to_vec(&payload).map_err(backend_err)?;
        self.backend
            .put(RETENTION_WINDOW_KEY, &bytes)
            .map_err(backend_err)
    }

    /// Phase-3 G17-A2 (§6.5 + r1-wsa-10) — read the persisted SUBSCRIBE
    /// retention window. Returns `Ok(None)` when no override is set
    /// (trait default semantics apply).
    ///
    /// # Errors
    /// Surfaces [`SuspensionStoreError::Backend`] on persistence failure.
    pub fn retention_window(&self) -> Result<Option<Duration>, SuspensionStoreError> {
        let Some(bytes) = self
            .backend
            .get(RETENTION_WINDOW_KEY)
            .map_err(backend_err)?
        else {
            return Ok(None);
        };
        let parsed: PersistedRetentionWindow =
            serde_ipld_dagcbor::from_slice(&bytes).map_err(backend_err)?;
        Ok(Some(Duration::from_millis(parsed.window_ms)))
    }

    /// Internal — load the per-subscriber cursor metadata if present.
    fn cursor_meta(
        &self,
        sub: &SubscriberId,
    ) -> Result<Option<PersistedCursorMeta>, SuspensionStoreError> {
        let Some(bytes) = self
            .backend
            .get(&cursor_meta_key(sub))
            .map_err(backend_err)?
        else {
            return Ok(None);
        };
        let parsed: PersistedCursorMeta =
            serde_ipld_dagcbor::from_slice(&bytes).map_err(backend_err)?;
        Ok(Some(parsed))
    }

    /// Internal — write per-subscriber cursor metadata.
    fn put_cursor_meta(
        &self,
        sub: &SubscriberId,
        meta: &PersistedCursorMeta,
    ) -> Result<(), SuspensionStoreError> {
        let bytes = serde_ipld_dagcbor::to_vec(meta).map_err(backend_err)?;
        self.backend
            .put(&cursor_meta_key(sub), &bytes)
            .map_err(backend_err)
    }
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
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
        let bytes = envelope.to_canonical_bytes().map_err(backend_err)?;
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
        let envelope = ExecutionStateEnvelope::from_canonical_bytes(&bytes).map_err(backend_err)?;
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
            .map_err(backend_err)?;
        // Phase-3 G17-A2 (§6.5) — lazy-initialise per-subscriber
        // cursor metadata. `registered_at` is stamped on first put;
        // `delivered_count` increments per put. Both fields feed the
        // retention-window override at `is_retention_exhausted`.
        let mut meta = self.cursor_meta(sub)?.unwrap_or_default();
        if meta.registered_at_unix_secs == 0 {
            meta.registered_at_unix_secs = now_unix_secs();
        }
        meta.delivered_count = meta.delivered_count.saturating_add(1);
        self.put_cursor_meta(sub, &meta)
    }

    /// Phase-3 G17-A2 (§6.5 + r1-wsa-10) override: consults the durable
    /// retention-window setting + per-subscriber metadata to determine
    /// whether the cursor has drifted past the window. Without an
    /// explicit override (no `set_retention_window` call) the
    /// trait-default `false` semantics apply.
    fn is_retention_exhausted(&self, sub: &SubscriberId) -> bool {
        // Read errors are conservatively dispositioned as
        // "not-exhausted" — a redb-side glitch should not silently
        // tear down active subscriptions. Operator-visible failures
        // already surface via the put-side
        // `SuspensionStoreError::Backend`.
        let Ok(Some(window)) = self.retention_window() else {
            return false;
        };
        let Ok(Some(meta)) = self.cursor_meta(sub) else {
            return false;
        };
        let now = now_unix_secs();
        let window_secs = window.as_secs();
        // Age-based: cursor is exhausted when (now - registered_at) > window.
        if window_secs == 0 {
            // Force-exhaust escape hatch — set_retention_window(ZERO)
            // marks every cursor exhausted.
            return true;
        }
        let registered_at = meta.registered_at_unix_secs;
        if registered_at == 0 {
            return false;
        }
        now.saturating_sub(registered_at) > window_secs
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
        // Phase-3 G17-A2 (§6.5) — cursor delete also wipes the
        // companion cursor-metadata side-table entry so a fresh
        // re-subscribe re-stamps `registered_at_unix_secs` cleanly.
        if let SuspensionKey::Cursor(sub) = &key {
            self.backend
                .delete(&cursor_meta_key(sub))
                .map_err(backend_err)?;
        }
        let raw = match key {
            SuspensionKey::WaitMetadata(cid) => wait_key(&cid),
            SuspensionKey::Envelope(cid) => envelope_key(&cid),
            SuspensionKey::Cursor(sub) => cursor_key(&sub),
            SuspensionKey::CapSnapshot(cid) => cap_snapshot_key(&cid),
        };
        self.backend.delete(&raw).map_err(backend_err)
    }

    fn put_cap_snapshot(
        &self,
        envelope_cid: Cid,
        snapshot: CapSnapshot,
    ) -> Result<(), SuspensionStoreError> {
        let payload: SerializableCapSnapshot = snapshot.into();
        let bytes = serde_ipld_dagcbor::to_vec(&payload).map_err(backend_err)?;
        self.backend
            .put(&cap_snapshot_key(&envelope_cid), &bytes)
            .map_err(backend_err)
    }

    fn get_cap_snapshot(
        &self,
        envelope_cid: &Cid,
    ) -> Result<Option<CapSnapshot>, SuspensionStoreError> {
        let Some(bytes) = self
            .backend
            .get(&cap_snapshot_key(envelope_cid))
            .map_err(backend_err)?
        else {
            return Ok(None);
        };
        let parsed: SerializableCapSnapshot =
            serde_ipld_dagcbor::from_slice(&bytes).map_err(backend_err)?;
        Ok(Some(parsed.into()))
    }
}

#[cfg(test)]
mod backward_compat_tests {
    //! Phase-3 G20-A2 wave-8a mr-5: backward-compat pin for the
    //! Phase-2b on-disk shape of `SerializableWaitMetadata`. Old
    //! persisted entries (from before the `ttl_hours` +
    //! `suspend_wallclock_ms` fields were added) MUST deserialize
    //! cleanly via `#[serde(default)]` and MUST NOT be classified as
    //! TTL-expired by the resume-time deadline check (since the entry
    //! was never stamped with a TTL deadline; the in-process
    //! `timeout_ms` is the only authoritative deadline a Phase-2b-shape
    //! entry carries).
    //!
    //! Without this pin, an old persisted state could regress silently
    //! if a future change drops `#[serde(default)]` or changes the
    //! deadline-check semantics on a `None` field.

    use super::SerializableWaitMetadata;
    use crate::wait_ttl_gc;
    use benten_eval::suspension_store::WaitMetadata;
    use serde::{Deserialize, Serialize};

    /// Phase-2b-shape `SerializableWaitMetadata` mirror — exactly the
    /// fields that existed pre-Phase-3-G20-A2. Serializing this
    /// produces the canonical Phase-2b on-disk byte shape.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct LegacyPhase2bWaitMetadata {
        suspend_elapsed_ms: Option<u64>,
        timeout_ms: Option<u64>,
        signal_shape: Option<benten_core::Value>,
        is_duration: bool,
    }

    #[test]
    fn phase_2b_shape_deserializes_with_serde_default_no_ttl_fields() {
        let legacy = LegacyPhase2bWaitMetadata {
            suspend_elapsed_ms: Some(1_000),
            timeout_ms: Some(60_000),
            signal_shape: None,
            is_duration: false,
        };
        let bytes = serde_ipld_dagcbor::to_vec(&legacy)
            .expect("legacy phase-2b shape encodes via DAG-CBOR cleanly");

        // Decode under the Phase-3 schema: #[serde(default)] must fill
        // the new fields with `None` rather than failing the decode.
        let parsed: SerializableWaitMetadata = serde_ipld_dagcbor::from_slice(&bytes)
            .expect("phase-2b on-disk bytes MUST round-trip into the Phase-3 SerializableWaitMetadata via #[serde(default)]");
        let meta: WaitMetadata = parsed.into();

        assert_eq!(meta.suspend_elapsed_ms, Some(1_000));
        assert_eq!(meta.timeout_ms, Some(60_000));
        assert!(!meta.is_duration);
        assert_eq!(
            meta.ttl_hours, None,
            "phase-2b shape MUST decode with ttl_hours=None (entry was never stamped with a TTL)"
        );
        assert_eq!(
            meta.suspend_wallclock_ms, None,
            "phase-2b shape MUST decode with suspend_wallclock_ms=None"
        );
    }

    #[test]
    fn phase_2b_shape_resume_does_not_fire_ttl_expired() {
        // Build a phase-2b-shape entry as it would have looked on
        // disk; verify the deadline check is a no-op (returns false /
        // None) regardless of how far into the future `now_ms` is.
        let legacy = LegacyPhase2bWaitMetadata {
            suspend_elapsed_ms: Some(0),
            timeout_ms: Some(60_000),
            signal_shape: None,
            is_duration: false,
        };
        let bytes = serde_ipld_dagcbor::to_vec(&legacy).unwrap();
        let parsed: SerializableWaitMetadata = serde_ipld_dagcbor::from_slice(&bytes).unwrap();
        let meta: WaitMetadata = parsed.into();

        // `deadline_ms` returns `None` when either `ttl_hours` or
        // `suspend_wallclock_ms` is `None`. A phase-2b-shape entry has
        // both fields as `None` so the deadline is not computed.
        assert_eq!(
            wait_ttl_gc::deadline_ms(&meta),
            None,
            "phase-2b shape MUST NOT compute a deadline (entry was never stamped with TTL)"
        );

        // `is_expired` consults `deadline_ms` and is `false` when the
        // deadline is `None`. Exercise across a wide range of `now_ms`
        // values to confirm: a phase-2b entry is NEVER classified as
        // TTL-expired by the resume-time check, so its in-process
        // `timeout_ms` remains the only authoritative deadline.
        for now_ms in [0u64, 1_000, 60_000, 3_600_000, u64::MAX / 2, u64::MAX] {
            assert!(
                !wait_ttl_gc::is_expired(&meta, now_ms),
                "phase-2b shape MUST NOT fire TTL-expired at now_ms={now_ms} \
                 (regression — the deadline-check semantics on missing fields drifted)"
            );
        }
    }
}
