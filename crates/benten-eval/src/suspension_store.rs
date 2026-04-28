//! Phase-2b G12-E — generalized [`SuspensionStore`] covering BOTH WAIT
//! suspend metadata AND SUBSCRIBE persistent cursors.
//!
//! # Why one trait, two surfaces
//!
//! Phase-2a + early-2b shipped two parallel ad-hoc surfaces:
//!
//! - WAIT metadata (deadline + signal-shape) lived in a process-local
//!   `OnceLock<Mutex<HashMap<Cid, WaitMetadata>>>` inside
//!   `primitives::wait`. Cross-process resume silently dropped the
//!   deadline / shape check (Phase-2a Compromise #10 cross-process gap).
//! - SUBSCRIBE persistent cursors went through a `SubscribeError`-typed
//!   trait in `primitives::subscribe` with an in-memory placeholder
//!   (`InMemorySuspensionStore`).
//! - Engine-side WAIT envelope persistence used a third surface,
//!   `engine_wait::ENVELOPE_CACHE`, gated behind the
//!   `envelope-cache-test-grade` feature.
//!
//! G12-E collapses these into a single `SuspensionStore` port living in
//! benten-eval (so primitive-layer code can consume it without crossing
//! the arch-1 dep-break to benten-graph). The trait carries five
//! operations:
//!
//! - `put_wait` / `get_wait` — WAIT metadata side table keyed by envelope
//!   CID. Restoring deadlines + signal shapes across a cross-process
//!   resume closes the Phase-2a Compromise #10 cross-process gap.
//! - `put_envelope` / `get_envelope` — `ExecutionStateEnvelope` bytes
//!   keyed by envelope CID. Replaces the test-grade `ENVELOPE_CACHE`
//!   in `engine_wait`.
//! - `put_cursor` / `get_cursor` — SUBSCRIBE `max_delivered_seq` keyed
//!   by [`SubscriberId`]. Replaces the early-2b
//!   `subscribe::SuspensionStore` placeholder.
//!
//! ## Key namespacing
//!
//! The three logical key spaces (wait-metadata-by-CID,
//! envelope-bytes-by-CID, cursor-by-SubscriberId) collide if reduced to
//! raw bytes — both wait metadata and envelope bytes are keyed by
//! [`Cid`]. Implementations MUST namespace their key prefixes so a
//! `put_wait(cid, …)` followed by `get_envelope(&cid)` returns `None`
//! (and vice-versa). The default test-suite asserts this in
//! `suspension_store_handles_both_wait_and_cursor_keys_without_collision`.
//!
//! ## Production redb backing
//!
//! The redb-backed concrete impl lives in `benten-engine` (benten-eval
//! is by-design dep-broken from benten-graph per arch-1 / phil-r1-2).
//! benten-eval ships [`InMemorySuspensionStore`] as the default + the
//! reference impl tests run against; `benten-engine::engine_wait` wires
//! the engine's `Arc<RedbBackend>` into a redb-backed `SuspensionStore`
//! at construction.

use benten_core::Cid;
use benten_core::SubscriberId;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::exec_state::ExecutionStateEnvelope;

// ---------------------------------------------------------------------------
// Error envelope
// ---------------------------------------------------------------------------

/// Error envelope for [`SuspensionStore`] operations.
///
/// Crate-rooted so neither WAIT- nor SUBSCRIBE-specific error types leak
/// across the trait boundary. The SUBSCRIBE wrapper layer in
/// `primitives::subscribe` lifts this into `SubscribeError` for its
/// public API; the WAIT layer surfaces it as
/// [`crate::EvalError::Host`] with [`crate::ErrorCode::HostBackendUnavailable`].
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum SuspensionStoreError {
    /// The underlying backing store rejected the operation. Carries an
    /// opaque diagnostic string; callers should NOT pattern-match on
    /// the message text.
    #[error("suspension store backend failure: {0}")]
    Backend(String),
}

// ---------------------------------------------------------------------------
// WAIT metadata side table value type (mirrors primitives::wait::WaitMetadata
// at the trait boundary).
// ---------------------------------------------------------------------------

/// Side-table metadata persisted for a suspended WAIT primitive. Mirrors
/// the `primitives::wait::WaitMetadata` shape but is `pub` so a
/// `SuspensionStore` consumer outside benten-eval (e.g. the
/// engine-side redb-backed impl) can construct + decode entries.
///
/// The metadata is INTENTIONALLY kept out of the
/// `ExecutionStatePayload` shape (which is frozen by the Inv-14 fixture
/// CID); cross-process resume hydrates this side-table from durable
/// storage on engine open.
#[derive(Debug, Clone, PartialEq)]
pub struct WaitMetadata {
    /// Millisecond value of `ctx.elapsed_ms()` at suspend time. `None`
    /// if no clock was injected; resume treats absence as "no deadline".
    pub suspend_elapsed_ms: Option<u64>,
    /// Timeout in ms, relative to `suspend_elapsed_ms`. `None` for the
    /// signal variant without an explicit timeout.
    pub timeout_ms: Option<u64>,
    /// Expected signal shape, if the WAIT node declared one. Absent
    /// means "untyped — any Value is admitted".
    pub signal_shape: Option<benten_core::Value>,
    /// Whether this WAIT is the `duration` variant (i.e. has
    /// `duration_ms` instead of `signal`).
    pub is_duration: bool,
}

// ---------------------------------------------------------------------------
// SuspensionStore trait
// ---------------------------------------------------------------------------

/// Generalized persistence port for WAIT suspensions AND SUBSCRIBE
/// persistent cursors.
///
/// G12-E unifies what were three separate ad-hoc surfaces
/// (`wait::registry`, `subscribe::SuspensionStore`,
/// `engine_wait::ENVELOPE_CACHE`) behind a single trait so the engine
/// can wire ONE backing store at construction time and have all three
/// suspension shapes survive a process restart.
///
/// # Implementations
///
/// - [`InMemorySuspensionStore`] — process-local, Send + Sync via
///   interior mutability. Default for `Engine::open` and the only
///   impl reachable from benten-eval directly.
/// - `benten_engine::RedbSuspensionStore` — redb-backed, persistent.
///   Wired by `EngineBuilder` against the engine's existing
///   `Arc<RedbBackend>`. Production default once the engine surfaces
///   it (G12-E lands the trait + in-memory impl + the engine wire-up).
pub trait SuspensionStore: Send + Sync {
    // -- WAIT metadata side table (keyed by envelope CID) --

    /// Persist the WAIT metadata side-table entry for envelope `cid`.
    ///
    /// # Errors
    /// Surfaces [`SuspensionStoreError::Backend`] on persistence failure.
    fn put_wait(&self, cid: Cid, meta: WaitMetadata) -> Result<(), SuspensionStoreError>;

    /// Look up the WAIT metadata side-table entry for envelope `cid`.
    /// Returns `Ok(None)` on a clean miss (entry never registered or
    /// already evicted).
    ///
    /// # Errors
    /// Surfaces [`SuspensionStoreError::Backend`] on persistence failure.
    fn get_wait(&self, cid: &Cid) -> Result<Option<WaitMetadata>, SuspensionStoreError>;

    // -- WAIT envelope persistence (keyed by envelope CID) --

    /// Persist a suspended [`ExecutionStateEnvelope`] under its
    /// `payload_cid`. Used by the engine-side `suspend_to_bytes`
    /// implementation so a `SuspendedHandle` can round-trip across a
    /// process boundary without the caller having to thread the bytes
    /// through their own storage layer.
    ///
    /// # Errors
    /// Surfaces [`SuspensionStoreError::Backend`] on persistence failure.
    fn put_envelope(&self, envelope: ExecutionStateEnvelope) -> Result<(), SuspensionStoreError>;

    /// Look up a persisted [`ExecutionStateEnvelope`] by its
    /// `payload_cid`. Returns `Ok(None)` on a clean miss.
    ///
    /// # Errors
    /// Surfaces [`SuspensionStoreError::Backend`] on persistence failure.
    fn get_envelope(
        &self,
        cid: &Cid,
    ) -> Result<Option<ExecutionStateEnvelope>, SuspensionStoreError>;

    // -- SUBSCRIBE persistent cursors (keyed by SubscriberId) --

    /// Persist `max_delivered_seq` for SUBSCRIBE persistent cursor `sub`.
    ///
    /// # Errors
    /// Surfaces [`SuspensionStoreError::Backend`] on persistence failure.
    fn put_cursor(
        &self,
        sub: &SubscriberId,
        max_delivered_seq: u64,
    ) -> Result<(), SuspensionStoreError>;

    /// Look up the persisted `max_delivered_seq` for SUBSCRIBE
    /// persistent cursor `sub`. Returns `Ok(None)` on a clean miss.
    ///
    /// # Errors
    /// Surfaces [`SuspensionStoreError::Backend`] on persistence failure.
    fn get_cursor(&self, sub: &SubscriberId) -> Result<Option<u64>, SuspensionStoreError>;

    /// True iff the SUBSCRIBE persistent cursor for `sub` has drifted
    /// past the retention window.
    ///
    /// Default impl returns `false`; only the in-memory test impl wires
    /// the override (the production redb impl computes drift from the
    /// stored `max_delivered_seq` + `registered_at`).
    fn is_retention_exhausted(&self, sub: &SubscriberId) -> bool {
        let _ = sub;
        false
    }

    /// Test-only: force the SUBSCRIBE retention window for `sub` to
    /// "exhausted". Default impl is a no-op so production backends never
    /// expose the hook.
    #[cfg(any(test, feature = "testing"))]
    fn testing_force_retention_exhausted(&self, sub: &SubscriberId) {
        let _ = sub;
    }

    /// Delete every key (wait metadata, envelope, cursor) under `key`.
    /// Variant erasure lets a single trait method serve all three key
    /// shapes without forcing the caller to know which surface owns the
    /// key. Implementations apply the operation to whichever namespace
    /// the [`SuspensionKey`] selects.
    ///
    /// # Errors
    /// Surfaces [`SuspensionStoreError::Backend`] on persistence failure.
    fn delete(&self, key: SuspensionKey) -> Result<(), SuspensionStoreError>;
}

/// Key shape for [`SuspensionStore::delete`]. The variant identifies
/// the namespace; implementations route to the correct internal table /
/// key prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuspensionKey {
    /// Drop the WAIT metadata entry for envelope `cid`.
    WaitMetadata(Cid),
    /// Drop the persisted envelope bytes for envelope `cid`.
    Envelope(Cid),
    /// Drop the SUBSCRIBE persistent cursor entry for `sub`.
    Cursor(SubscriberId),
}

// ---------------------------------------------------------------------------
// In-memory reference implementation
// ---------------------------------------------------------------------------

/// Process-local in-memory [`SuspensionStore`] implementation. Default
/// for `Engine::open` until the operator wires the redb-backed impl.
///
/// Send + Sync via interior mutability over a single `Mutex`; the
/// expected workload (a handful of pending suspensions per engine, low
/// kHz cursor ack rate) does not justify finer-grained locking.
///
/// Cross-process state is NOT preserved by this impl (it lives in
/// process memory). The redb-backed impl is the cross-process answer;
/// the in-memory variant exists for tests and for in-process Phase-2b
/// dev-server usage where survival across an explicit process restart
/// is not required.
#[derive(Default)]
pub struct InMemorySuspensionStore {
    inner: Mutex<InMemoryStoreInner>,
}

#[derive(Default)]
struct InMemoryStoreInner {
    wait_meta: HashMap<Cid, WaitMetadata>,
    envelopes: HashMap<Cid, ExecutionStateEnvelope>,
    cursors: HashMap<SubscriberId, u64>,
    retention_exhausted: HashMap<SubscriberId, bool>,
}

impl InMemorySuspensionStore {
    /// Construct an empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Test helper: force the retention window for `sub` to "exhausted".
    /// Used by the SUBSCRIBE persistent-cursor red-phase tests.
    pub fn force_retention_exhausted(&self, sub: &SubscriberId) {
        let mut g = self.inner.lock().expect("suspension store poisoned");
        g.retention_exhausted.insert(*sub, true);
    }
}

impl SuspensionStore for InMemorySuspensionStore {
    fn put_wait(&self, cid: Cid, meta: WaitMetadata) -> Result<(), SuspensionStoreError> {
        let mut g = self.inner.lock().expect("suspension store poisoned");
        g.wait_meta.insert(cid, meta);
        Ok(())
    }

    fn get_wait(&self, cid: &Cid) -> Result<Option<WaitMetadata>, SuspensionStoreError> {
        let g = self.inner.lock().expect("suspension store poisoned");
        Ok(g.wait_meta.get(cid).cloned())
    }

    fn put_envelope(&self, envelope: ExecutionStateEnvelope) -> Result<(), SuspensionStoreError> {
        let mut g = self.inner.lock().expect("suspension store poisoned");
        g.envelopes.insert(envelope.payload_cid, envelope);
        Ok(())
    }

    fn get_envelope(
        &self,
        cid: &Cid,
    ) -> Result<Option<ExecutionStateEnvelope>, SuspensionStoreError> {
        let g = self.inner.lock().expect("suspension store poisoned");
        Ok(g.envelopes.get(cid).cloned())
    }

    fn put_cursor(
        &self,
        sub: &SubscriberId,
        max_delivered_seq: u64,
    ) -> Result<(), SuspensionStoreError> {
        let mut g = self.inner.lock().expect("suspension store poisoned");
        g.cursors.insert(*sub, max_delivered_seq);
        Ok(())
    }

    fn get_cursor(&self, sub: &SubscriberId) -> Result<Option<u64>, SuspensionStoreError> {
        let g = self.inner.lock().expect("suspension store poisoned");
        Ok(g.cursors.get(sub).copied())
    }

    fn is_retention_exhausted(&self, sub: &SubscriberId) -> bool {
        let g = self.inner.lock().expect("suspension store poisoned");
        *g.retention_exhausted.get(sub).unwrap_or(&false)
    }

    #[cfg(any(test, feature = "testing"))]
    fn testing_force_retention_exhausted(&self, sub: &SubscriberId) {
        self.force_retention_exhausted(sub);
    }

    fn delete(&self, key: SuspensionKey) -> Result<(), SuspensionStoreError> {
        let mut g = self.inner.lock().expect("suspension store poisoned");
        match key {
            SuspensionKey::WaitMetadata(cid) => {
                g.wait_meta.remove(&cid);
            }
            SuspensionKey::Envelope(cid) => {
                g.envelopes.remove(&cid);
            }
            SuspensionKey::Cursor(sub) => {
                g.cursors.remove(&sub);
                g.retention_exhausted.remove(&sub);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Process-default singleton (for the WAIT primitive's metadata side
// table). The engine wires its own per-Engine store via Arc when one is
// configured; the WAIT primitive falls back to this singleton when run
// outside the engine (the unit tests in `primitives::wait`).
// ---------------------------------------------------------------------------

use std::sync::{Arc, OnceLock};

static DEFAULT_PROCESS_STORE: OnceLock<Arc<dyn SuspensionStore>> = OnceLock::new();

/// Return the process-default [`SuspensionStore`] handle.
///
/// Call sites that have an explicit `Arc<dyn SuspensionStore>` (engine
/// suspend / resume paths, configured SUBSCRIBE registrations) should
/// thread their own store through and ignore this singleton. The
/// fallback exists for unit tests + primitive-layer code that runs
/// outside the engine.
#[must_use]
pub fn default_process_store() -> Arc<dyn SuspensionStore> {
    DEFAULT_PROCESS_STORE
        .get_or_init(|| Arc::new(InMemorySuspensionStore::new()) as Arc<dyn SuspensionStore>)
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec_state::{ExecutionStatePayload, Frame};

    fn cid_for(seed: &[u8]) -> Cid {
        Cid::from_blake3_digest(*blake3::hash(seed).as_bytes())
    }

    fn sample_envelope(seed: &[u8]) -> ExecutionStateEnvelope {
        let payload = ExecutionStatePayload {
            attribution_chain: Vec::new(),
            pinned_subgraph_cids: Vec::new(),
            context_binding_snapshots: Vec::new(),
            resumption_principal_cid: cid_for(seed),
            frame_stack: vec![Frame::root()],
            frame_index: 0,
        };
        ExecutionStateEnvelope::new(payload).expect("envelope encode")
    }

    #[test]
    fn wait_meta_round_trip() {
        let store = InMemorySuspensionStore::new();
        let cid = cid_for(b"wait-meta-rt");
        let meta = WaitMetadata {
            suspend_elapsed_ms: Some(10),
            timeout_ms: Some(50),
            signal_shape: None,
            is_duration: false,
        };
        store.put_wait(cid, meta.clone()).unwrap();
        assert_eq!(store.get_wait(&cid).unwrap(), Some(meta));
    }

    #[test]
    fn envelope_round_trip() {
        let store = InMemorySuspensionStore::new();
        let env = sample_envelope(b"env-rt");
        let cid = env.payload_cid;
        store.put_envelope(env.clone()).unwrap();
        let got = store.get_envelope(&cid).unwrap().unwrap();
        assert_eq!(got.payload_cid, env.payload_cid);
    }

    #[test]
    fn cursor_round_trip() {
        let store = InMemorySuspensionStore::new();
        let sub = SubscriberId::from_cid(cid_for(b"cursor-rt"));
        store.put_cursor(&sub, 42).unwrap();
        assert_eq!(store.get_cursor(&sub).unwrap(), Some(42));
    }

    #[test]
    fn delete_routes_per_namespace() {
        let store = InMemorySuspensionStore::new();
        let cid = cid_for(b"del-shared");
        let env = sample_envelope(b"del-shared-env");
        let env_cid = env.payload_cid;
        let sub = SubscriberId::from_cid(cid);

        store
            .put_wait(
                cid,
                WaitMetadata {
                    suspend_elapsed_ms: None,
                    timeout_ms: None,
                    signal_shape: None,
                    is_duration: false,
                },
            )
            .unwrap();
        store.put_envelope(env).unwrap();
        store.put_cursor(&sub, 7).unwrap();

        store.delete(SuspensionKey::WaitMetadata(cid)).unwrap();
        assert!(store.get_wait(&cid).unwrap().is_none());
        // Envelope + cursor untouched.
        assert!(store.get_envelope(&env_cid).unwrap().is_some());
        assert_eq!(store.get_cursor(&sub).unwrap(), Some(7));
    }
}
