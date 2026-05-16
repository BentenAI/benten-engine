//! [`KVBackend`] trait + supporting types ([`ScanResult`],
//! [`DurabilityMode`]).
//!
//! The trait is the narrow storage waist the rest of the Benten graph layer
//! consumes. Two R1-triage deliverables shape this module:
//!
//! - **`P1.graph.error-polymorphism`** — `KVBackend` carries an associated
//!   `Error` type bounded by `std::error::Error + Send + Sync + 'static`, so
//!   non-redb implementations (in-memory mock, WASM peer-fetch, iroh-fetch)
//!   surface their errors through their own enums rather than lying through
//!   the spike-era `GraphError::Redb(String)` variant.
//!
//! - **`P1.graph.scan-iterator`** — `scan` returns a [`ScanResult`] newtype
//!   (rather than a raw `Vec<(Vec<u8>, Vec<u8>)>`), giving the trait a stable
//!   return shape we can evolve toward true lazy iteration in Phase 2 without
//!   re-breaking the call sites. [`ScanResult`] is deliberately shape-opaque:
//!   callers see `.len()` / `.is_empty()` / `.iter()` / `.as_slice()` /
//!   `IntoIterator` through inherent methods, and the backing storage (today
//!   a `Vec`) is not part of the public contract. Phase 2 can swap in a boxed
//!   streaming iterator without a semver break.
//!
//! ## Phase 2 evolution points (read before adding call sites downstream)
//!
//! - `ScanResult` is shape-opaque. Do **not** name its `IntoIter` type; do
//!   **not** rely on slice semantics beyond the explicit `.as_slice()`
//!   accessor. Phase 2 may swap the backing storage to a boxed iterator.
//! - `ChangeEvent.labels: Vec<String>` carries the full label set on Node
//!   events and a single-element vector on Edge events. Callers that need
//!   only the primary label should prefer the `primary_label()` accessor
//!   rather than indexing into the vec directly — the accessor is stable
//!   across any future changes to how the vec is populated (e.g., ordering).
//! - `KVBackend::put_batch` is Put-only. Heterogeneous write sets (node put
//!   + edge delete + index remove in a single commit) belong on the G3
//!     transaction primitive, not on `put_batch`.

use core::slice::Iter;

/// Durability knob for a backend commit.
///
/// The variants are ordered from safest to loosest. The trait surface picks
/// the default; backends are responsible for mapping each variant to their
/// own durability primitive (redb v4 only exposes `Durability::Immediate`
/// and `Durability::None`, so [`DurabilityMode::Group`] collapses to
/// `Durability::Immediate` at the redb backend — see
/// `crates/benten-graph/src/redb_backend.rs::to_redb_durability`). The
/// enum lives here in the trait-surface module so heterogeneous backends
/// can all honor the same vocabulary.
///
/// Semantics finalized in G2-B alongside the `RedbBackend` wiring; this enum
/// is declared here so the trait-level reshape lands without a circular dep
/// between `backend.rs` and a redb-specific module.
///
/// **Default flipped at Phase-3 G13-E (2026-05-05):** `default()` returns
/// [`DurabilityMode::Group`] (was [`DurabilityMode::Immediate`] through
/// Phase-2b). The flip closes `docs/SECURITY-POSTURE.md` Compromise #12
/// (macOS APFS fsync floor; CRUD fast-path 150–300 µs §14.6 target) by
/// declaring grouped-fsync as the engine-level default posture for
/// CRUD writes. Backend mapping is a separate concern: redb v4 still
/// collapses Group → Immediate (the
/// `crates/benten-graph/src/redb_backend.rs::warn_if_group_durability_collapsed`
/// one-shot warning still fires for explicit Group requests on benches).
/// When redb grows native batched-commit support — or a peer-sync /
/// in-RAM backend lands a true Group implementation — the engine
/// surface already defaults to the correct posture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurabilityMode {
    /// fsync before commit returns. Strongest guarantee, slowest throughput.
    /// Used by capability-grant writes regardless of the engine-configured
    /// default — see
    /// `crates/benten-graph/tests/capability_grant_writes_immediate.rs`.
    Immediate,
    /// Group commits into a batched fsync window. Higher throughput, bounded
    /// tail latency on the fsync flush.
    ///
    /// **Default since Phase-3 G13-E.** `DurabilityMode::default()` returns
    /// [`DurabilityMode::Group`] for the CRUD fast-path; callers can opt
    /// back into [`DurabilityMode::Immediate`] explicitly via the
    /// `_with_durability` constructors (see
    /// `crates/benten-graph/src/redb_backend.rs::RedbBackend::open_or_create_with_durability`).
    ///
    /// **redb v4 backend caveat:** the underlying redb API only exposes
    /// `Durability::Immediate` and `Durability::None`; `Group` therefore
    /// collapses to `Durability::Immediate` at the redb mapping today
    /// (see `crates/benten-graph/src/redb_backend.rs::to_redb_durability`).
    /// The construction path emits a one-shot warning so operators are
    /// not misled by benchmark numbers. The default flip is still
    /// load-bearing because: (a) the engine-level posture is the right
    /// surface to declare for Compromise #12 closure; (b) non-redb
    /// backends (in-RAM thin-client, future peer-sync) can implement
    /// a true grouped fsync without changing call sites; (c) when
    /// redb grows the capability, the default is already correct.
    Group,
    /// Commit returns before the durable fsync; durability is best-effort and
    /// a crash may lose the last few commits. Test-only / in-memory-mock
    /// friendly. Never the default for disk-backed stores.
    Async,
}

impl Default for DurabilityMode {
    fn default() -> Self {
        // Phase-3 G13-E posture: CRUD fast-path defaults to grouped fsync
        // (closes `docs/SECURITY-POSTURE.md` Compromise #12). Backends that
        // cannot honor Group natively (redb v4) collapse to Immediate at
        // the mapping — see
        // `crates/benten-graph/src/redb_backend.rs::to_redb_durability`.
        // Capability-grant writes always use `Immediate` regardless of
        // the engine-configured default; see
        // `crates/benten-graph/tests/capability_grant_writes_immediate.rs`.
        DurabilityMode::Group
    }
}

/// Return type of [`KVBackend::scan`] — an opaque collection of (key, value)
/// byte pairs matching the supplied prefix.
///
/// `ScanResult` is deliberately a *shape-opaque* newtype. Callers consume it
/// through inherent methods ([`ScanResult::len`], [`ScanResult::is_empty`],
/// [`ScanResult::iter`], [`ScanResult::as_slice`]) or `IntoIterator` (the
/// associated iterator type is the opaque [`ScanIter`]); the backing
/// storage (today a `Vec`) is not part of the public contract.
///
/// This is load-bearing for Phase 2: when a true lazy-streaming backend
/// lands (WASM peer-fetch, iroh-fetch), `ScanResult` can wrap a boxed
/// iterator or a streaming handle without changing the public surface.
/// Removing the former `Deref<Target=[...]>` impl keeps slice-semantics
/// from leaking into consumer code.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ScanResult(Vec<(Vec<u8>, Vec<u8>)>);

impl ScanResult {
    /// Construct an empty scan result (e.g., for a zero-hit prefix).
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Construct a scan result from an owned vector of pairs. Crate-private
    /// so the Vec-backed shape is not part of the public contract. Backends
    /// outside this crate should `.collect()` through [`FromIterator`].
    #[must_use]
    pub(crate) fn from_vec(pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Self(pairs)
    }

    /// Number of (key, value) pairs in the result. O(1) for the Phase-1
    /// backing, but part of the stable public contract — Phase-2 streaming
    /// backends materialize the count on construction.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `true` if the scan matched zero keys.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Borrow the results as `&[(Vec<u8>, Vec<u8>)]`. Explicit accessor —
    /// prefer `.iter()` when you just want to walk the pairs; use this only
    /// when you genuinely need slice semantics (indexing, windowing, etc.).
    #[must_use]
    pub fn as_slice(&self) -> &[(Vec<u8>, Vec<u8>)] {
        &self.0
    }

    /// Iterator over `&(key, value)` pairs. Stable replacement for the
    /// deprecated `Deref`-based `.iter()` call path.
    pub fn iter(&self) -> Iter<'_, (Vec<u8>, Vec<u8>)> {
        self.0.iter()
    }
}

/// Opaque owning iterator returned by `ScanResult::into_iter`. Internals are
/// intentionally not part of the public contract — a Phase-2 streaming
/// backend may replace the backing representation without a semver break.
//
// Implementation note: we wrap `std::vec::IntoIter` today rather than a
// `Box<dyn Iterator>` so iteration is allocation-free for Phase 1. The
// newtype is the forward-compat shim.
pub struct ScanIter(std::vec::IntoIter<(Vec<u8>, Vec<u8>)>);

impl Iterator for ScanIter {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for ScanIter {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FromIterator<(Vec<u8>, Vec<u8>)> for ScanResult {
    fn from_iter<I: IntoIterator<Item = (Vec<u8>, Vec<u8>)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for ScanResult {
    type Item = (Vec<u8>, Vec<u8>);
    type IntoIter = ScanIter;

    fn into_iter(self) -> Self::IntoIter {
        ScanIter(self.0.into_iter())
    }
}

impl<'a> IntoIterator for &'a ScanResult {
    type Item = &'a (Vec<u8>, Vec<u8>);
    type IntoIter = Iter<'a, (Vec<u8>, Vec<u8>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Minimal key/value backend trait for the Benten graph.
///
/// Values are opaque byte blobs from the trait's perspective; the graph layer
/// above ([`NodeStore`](crate::store::NodeStore), [`EdgeStore`](crate::store::EdgeStore))
/// is responsible for (de)serializing domain types. Keys are also opaque
/// bytes so the graph layer can choose its own key schema.
///
/// # Error polymorphism
///
/// Each backend picks its own `Error` type (bounded by `std::error::Error +
/// Send + Sync + 'static`). This closes the R1 finding `P1.graph.error-
/// polymorphism`: non-redb backends (in-memory mock, WASM peer-fetch,
/// iroh-fetch) no longer have to stringify their errors into a
/// `GraphError::Redb(String)` variant that lies about where the error came
/// from.
///
/// # Atomic batches
///
/// [`put_batch`](KVBackend::put_batch) must be atomic: either every pair in
/// the batch commits or none do. This is the primitive the G3 transaction
/// primitive (`begin`/`commit`/`rollback`) builds on.
///
/// # Phase 2 async note
///
/// `KVBackend` is synchronous today — every method returns `Result<_, _>`
/// directly. Phase-2 network-backed implementations (WASM peer-fetch,
/// iroh-fetch) will need either async mirror methods or a parallel
/// `AsyncKVBackend` trait; the `Self::Error` bound is already permissive
/// enough for the error shapes those backends will surface.
///
/// # Example — a trivial in-memory backend
///
/// ```rust
/// use std::collections::BTreeMap;
/// use std::sync::Mutex;
/// use benten_graph::{KVBackend, ScanResult};
///
/// #[derive(Default)]
/// struct MemBackend {
///     inner: Mutex<BTreeMap<Vec<u8>, Vec<u8>>>,
/// }
///
/// #[derive(Debug, thiserror::Error)]
/// enum MemError {
///     #[error("mem: poisoned")]
///     Poisoned,
/// }
///
/// impl KVBackend for MemBackend {
///     type Error = MemError;
///
///     fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
///         Ok(self.inner.lock().map_err(|_| MemError::Poisoned)?.get(key).cloned())
///     }
///     fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
///         self.inner.lock().map_err(|_| MemError::Poisoned)?
///             .insert(key.to_vec(), value.to_vec());
///         Ok(())
///     }
///     fn delete(&self, key: &[u8]) -> Result<(), Self::Error> {
///         self.inner.lock().map_err(|_| MemError::Poisoned)?.remove(key);
///         Ok(())
///     }
///     fn scan(&self, prefix: &[u8]) -> Result<ScanResult, Self::Error> {
///         let g = self.inner.lock().map_err(|_| MemError::Poisoned)?;
///         Ok(g.range(prefix.to_vec()..)
///             .take_while(|(k, _)| k.starts_with(prefix))
///             .map(|(k, v)| (k.clone(), v.clone()))
///             .collect())
///     }
///     fn put_batch(&self, pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), Self::Error> {
///         let mut g = self.inner.lock().map_err(|_| MemError::Poisoned)?;
///         for (k, v) in pairs { g.insert(k.clone(), v.clone()); }
///         Ok(())
///     }
/// }
///
/// let b = MemBackend::default();
/// b.put(b"k", b"v").unwrap();
/// assert_eq!(b.get(b"k").unwrap().as_deref(), Some(&b"v"[..]));
/// ```
pub trait KVBackend: Send + Sync {
    /// Backend-specific error type. Constrained to the standard error-object
    /// shape so consumers can `.source()`-chain across heterogeneous backends
    /// without the spike's String-coercion footgun.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Fetch the value stored under `key`. Returns `Ok(None)` on a clean miss
    /// — a missing key is information, not a failure.
    ///
    /// # Errors
    /// Implementation-defined. redb surfaces transactional or I/O failures;
    /// an in-memory mock surfaces lock poisoning; a peer-fetch WASM backend
    /// surfaces network errors — each through its own `Self::Error` enum.
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;

    /// Insert or overwrite the value at `key`.
    ///
    /// # Errors
    /// Implementation-defined, per [`Self::Error`].
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;

    /// Delete the value at `key`. Idempotent: returns `Ok(())` even if the
    /// key was absent.
    ///
    /// # Errors
    /// Implementation-defined, per [`Self::Error`].
    fn delete(&self, key: &[u8]) -> Result<(), Self::Error>;

    /// Return every (key, value) pair whose key starts with `prefix` as a
    /// [`ScanResult`]. A zero-hit scan returns an empty [`ScanResult`], never
    /// an error.
    ///
    /// Consumers walk the result through [`ScanResult::iter`] /
    /// [`ScanResult::len`] / [`ScanResult::is_empty`] /
    /// [`ScanResult::as_slice`], or iterate directly via `for (k, v) in hits`.
    /// The result is shape-opaque: Phase 2 may swap the backing storage
    /// without breaking call sites.
    ///
    /// # Errors
    /// Implementation-defined. A zero-hit prefix is *not* an error.
    fn scan(&self, prefix: &[u8]) -> Result<ScanResult, Self::Error>;

    /// Commit multiple puts atomically. Either every pair lands or none do.
    ///
    /// Signature is a slice of `(key, value)` pairs because every call site
    /// is pure-Put. Heterogeneous write sets (node put + edge delete + index
    /// remove in a single commit) live on the G3 transaction primitive
    /// (`crate::transaction`), not here; this method stays put-only.
    ///
    /// # Errors
    /// Implementation-defined.
    fn put_batch(&self, pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), Self::Error>;

    /// Whether this backend honors [`DurabilityMode`] preferences.
    ///
    /// `DurabilityMode` lives in this trait-surface module as the
    /// cross-backend durability vocabulary, but the trait surface does
    /// not thread a per-call durability tier through `put` / `put_batch`
    /// (durability is configured per-backend at construction time — e.g.
    /// `RedbBackend::open_or_create_with_durability` — and derived
    /// per-call from `WriteAuthority` in the privileged path). This
    /// accessor gives a generic `<B: GraphBackend>` consumer a runtime
    /// signal of whether a configured durability preference will be
    /// honored at all:
    ///
    /// - Disk-backed backends that map `DurabilityMode` to an fsync
    ///   primitive (`RedbBackend`) return `true` (the default).
    /// - In-RAM / read-only backends with no fsync semantic to honor
    ///   (`BrowserBackend` thin-client cache, `SnapshotBlobBackend`
    ///   read-only memory map, `NetworkFetchStubBackend`) return `false`
    ///   — a configured `DurabilityMode::Immediate` is silently a no-op
    ///   on these and callers that need a durability guarantee should
    ///   refuse to wire such a backend.
    ///
    /// Default returns `true` (the disk-backed posture). Surf-1 #860.
    fn supports_durability(&self) -> bool {
        true
    }
}
