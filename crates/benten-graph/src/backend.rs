//! [`KVBackend`] trait + supporting types ([`ScanResult`], [`BatchOp`],
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
//!   re-breaking the call sites. [`ScanResult`] exposes slice-like ergonomics
//!   (via `Deref`) and iterator-like consumption (via `IntoIterator`) in the
//!   same type.

use core::ops::Deref;
use core::slice::Iter;

/// A batch operation enqueued into [`KVBackend::put_batch`] or the transaction
/// primitive.
///
/// Phase 1 only uses `Put` in the batch path; `Delete` is exposed so that
/// future hetereogeneous write sets (index update + node put + edge delete in
/// a single commit) can all round-trip through the trait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchOp {
    /// Insert or overwrite `key` with `value`.
    Put {
        /// Storage key (opaque bytes).
        key: Vec<u8>,
        /// Stored value (opaque bytes).
        value: Vec<u8>,
    },
    /// Remove `key`. Idempotent — a `Delete` of an absent key commits cleanly.
    Delete {
        /// Storage key to remove.
        key: Vec<u8>,
    },
}

/// Durability knob for a backend commit.
///
/// The variants are ordered from safest to loosest. Backends pick a default
/// (redb defaults to [`DurabilityMode::Immediate`]); the enum lives here in
/// the trait-surface module so that heterogeneous backends can all honor the
/// same vocabulary.
///
/// Semantics finalized in G2-B alongside the `RedbBackend` wiring; this enum
/// is declared here so the trait-level reshape lands without a circular dep
/// between `backend.rs` and a redb-specific module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurabilityMode {
    /// fsync before commit returns. Strongest guarantee, slowest throughput.
    Immediate,
    /// Group commits into a batched fsync window. Higher throughput, bounded
    /// tail latency on the fsync flush.
    Group,
    /// Commit returns before the durable fsync; durability is best-effort and
    /// a crash may lose the last few commits. Test-only / in-memory-mock
    /// friendly. Never the default for disk-backed stores.
    Async,
}

impl Default for DurabilityMode {
    fn default() -> Self {
        // Safety posture — disk-backed stores fsync on commit unless the
        // caller explicitly opts out.
        DurabilityMode::Immediate
    }
}

/// Return type of [`KVBackend::scan`] — an owned list of (key, value) byte
/// pairs matching the supplied prefix.
///
/// `ScanResult` is deliberately a *newtype* rather than a raw `Vec<...>` or
/// `Box<dyn Iterator<...>>` because two consumer shapes must both compile:
///
/// - slice-like — `hits.len()`, `hits.is_empty()`, `hits.iter()`, `hits[0]`
///   (via [`Deref`] to `[(Vec<u8>, Vec<u8>)]`);
/// - iterator-like — `for (k, v) in hits { ... }` (via [`IntoIterator`]).
///
/// The newtype also gives Phase 2 a migration path: when a true-lazy
/// streaming scan lands, `ScanResult` can wrap a boxed iterator internally
/// without changing the public return type on [`KVBackend::scan`].
///
/// `ScanResult` implements [`FromIterator`] so backends can `.collect()` the
/// results of an internal iteration directly into the trait-visible return
/// type.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ScanResult(Vec<(Vec<u8>, Vec<u8>)>);

impl ScanResult {
    /// Construct an empty scan result (e.g., for a zero-hit prefix).
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Construct a scan result from an owned vector of pairs.
    #[must_use]
    pub fn from_vec(pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Self(pairs)
    }

    /// Consume the scan result and return the underlying vector of pairs.
    #[must_use]
    pub fn into_inner(self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.0
    }

    /// Borrow the underlying pairs as a slice. Equivalent to `&*result`.
    #[must_use]
    pub fn as_slice(&self) -> &[(Vec<u8>, Vec<u8>)] {
        &self.0
    }
}

impl Deref for ScanResult {
    type Target = [(Vec<u8>, Vec<u8>)];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[(Vec<u8>, Vec<u8>)]> for ScanResult {
    fn as_ref(&self) -> &[(Vec<u8>, Vec<u8>)] {
        &self.0
    }
}

impl FromIterator<(Vec<u8>, Vec<u8>)> for ScanResult {
    fn from_iter<I: IntoIterator<Item = (Vec<u8>, Vec<u8>)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for ScanResult {
    type Item = (Vec<u8>, Vec<u8>);
    type IntoIter = std::vec::IntoIter<(Vec<u8>, Vec<u8>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ScanResult {
    type Item = &'a (Vec<u8>, Vec<u8>);
    type IntoIter = Iter<'a, (Vec<u8>, Vec<u8>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<Vec<(Vec<u8>, Vec<u8>)>> for ScanResult {
    fn from(v: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Self(v)
    }
}

impl From<ScanResult> for Vec<(Vec<u8>, Vec<u8>)> {
    fn from(r: ScanResult) -> Self {
        r.0
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
    /// Consumers may treat the result as a slice (via [`Deref`] —
    /// `.len()`, `.iter()`, indexing) or as an iterator (via `for (k, v) in
    /// hits`). See [`ScanResult`] for the full ergonomic surface.
    ///
    /// # Errors
    /// Implementation-defined. A zero-hit prefix is *not* an error.
    fn scan(&self, prefix: &[u8]) -> Result<ScanResult, Self::Error>;

    /// Commit multiple puts atomically. Either every pair lands or none do.
    ///
    /// Phase 1 signature is a slice of `(key, value)` pairs rather than a
    /// slice of [`BatchOp`] because every current call site is pure-Put.
    /// When hetereogeneous write sets land in G3 the transaction primitive
    /// will consume [`BatchOp`] directly; this method stays put-only.
    ///
    /// # Errors
    /// Implementation-defined.
    fn put_batch(&self, pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), Self::Error>;
}
