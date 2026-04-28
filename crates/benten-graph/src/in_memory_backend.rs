//! In-memory [`KVBackend`] implementation backed by a `BTreeMap`.
//!
//! HANDOFF §3.F wave-5: 9 of 10 `packages/engine/test/*.test.ts` files use
//! `Engine.open(":memory:")` and need a transient backend. `InMemoryBackend`
//! satisfies the [`KVBackend`] trait surface — get / put / delete / scan /
//! put_batch — with byte-for-byte parity to [`crate::RedbBackend`]'s
//! `KVBackend` impl on the same op sequences (see
//! `tests/in_memory_backend_equiv_to_redb.rs`).
//!
//! ## Scope vs. [`crate::RedbBackend`]
//!
//! `InMemoryBackend` implements ONLY the [`KVBackend`] trait. It is **not**
//! a [`crate::store::NodeStore`] or [`crate::store::EdgeStore`] — those
//! traits carry per-CID change-event subscription, index maintenance, and
//! the `WriteContext` capability-policy hook, which are
//! [`crate::RedbBackend`]-specific concerns wired into the engine's
//! `Arc<RedbBackend>` field. Wiring `Engine::open(":memory:")` to a
//! transient store therefore goes through redb's own `InMemoryBackend`
//! (a memory-backed redb page store) so the engine retains full
//! `RedbBackend` semantics — the higher-level invariants (Inv-11 system-
//! zone gating, Inv-13 immutability cache, change-event publishing,
//! transaction primitive) are unchanged.
//!
//! This file provides the **pure-trait** in-memory backend so that future
//! non-redb consumers documented in the [`KVBackend`] trait docs (Phase 2
//! WASM peer-fetch, iroh-fetch, test mocks) have a reference impl with a
//! known-correct equivalence proof against redb.
//!
//! ## Concurrency
//!
//! A `Mutex<BTreeMap<Vec<u8>, Vec<u8>>>` is used rather than `papaya` for
//! three reasons:
//!
//! 1. Ordered iteration is required for [`KVBackend::scan`] (prefix
//!    semantics) — `BTreeMap::range` makes this O(log n + hits) without
//!    extra sorting.
//! 2. `put_batch` is required to be atomic; a single coarse `Mutex` makes
//!    that trivially correct.
//! 3. The intended workload is single-process tests / transient runs; the
//!    finer-grained concurrency `papaya` provides is not needed and would
//!    complicate the atomic-batch contract.
//!
//! Lock poisoning surfaces as [`crate::GraphError::Redb`] with a
//! `"in-memory: lock poisoned"` payload; this matches the
//! `Redb(String)`-payload precedent used elsewhere for non-redb-origin
//! storage-layer failures.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::GraphError;
use crate::backend::{KVBackend, ScanResult};
use crate::redb_backend::next_prefix;

/// In-memory [`KVBackend`]. See module docs.
///
/// # Examples
///
/// ```rust
/// use benten_graph::{InMemoryBackend, KVBackend};
///
/// let b = InMemoryBackend::new();
/// b.put(b"k", b"v").unwrap();
/// assert_eq!(b.get(b"k").unwrap().as_deref(), Some(&b"v"[..]));
/// b.delete(b"k").unwrap();
/// assert_eq!(b.get(b"k").unwrap(), None);
/// ```
#[derive(Debug, Default)]
pub struct InMemoryBackend {
    inner: Mutex<BTreeMap<Vec<u8>, Vec<u8>>>,
}

impl InMemoryBackend {
    /// Construct an empty in-memory backend.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(BTreeMap::new()),
        }
    }
}

/// Convert a poisoned-lock error to a [`GraphError`] payload that names
/// the in-memory backend as the source. Pulled out of the hot path so each
/// trait method is a single line of lock + body.
#[inline]
fn poisoned() -> GraphError {
    GraphError::Redb("in-memory: lock poisoned".into())
}

impl KVBackend for InMemoryBackend {
    type Error = GraphError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, GraphError> {
        let g = self.inner.lock().map_err(|_| poisoned())?;
        Ok(g.get(key).cloned())
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), GraphError> {
        let mut g = self.inner.lock().map_err(|_| poisoned())?;
        g.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), GraphError> {
        let mut g = self.inner.lock().map_err(|_| poisoned())?;
        // Idempotent — deleting an absent key is `Ok(())` per the
        // `KVBackend::delete` contract.
        g.remove(key);
        Ok(())
    }

    fn scan(&self, prefix: &[u8]) -> Result<ScanResult, GraphError> {
        let g = self.inner.lock().map_err(|_| poisoned())?;

        // Mirror RedbBackend::scan exactly:
        //  - empty prefix → full table iter
        //  - non-empty prefix → bounded range [prefix, next_prefix)
        //  - all-0xff prefix → unbounded prefix.. (next_prefix returns None)
        // Ordering follows BTreeMap (lex on Vec<u8>) which matches redb's
        // byte-key ordering, so the equivalence test asserts identical
        // (k, v) sequences.
        let pairs: Vec<(Vec<u8>, Vec<u8>)> = if prefix.is_empty() {
            g.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            let next = next_prefix(prefix);
            match next {
                Some(upper) => g
                    .range(prefix.to_vec()..upper)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
                None => g
                    .range(prefix.to_vec()..)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            }
        };

        Ok(ScanResult::from_vec(pairs))
    }

    fn put_batch(&self, pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), GraphError> {
        // Atomic by virtue of holding the single coarse `Mutex` for the
        // duration of the batch. Either every pair lands or none do (the
        // only failure path is lock poisoning, which fires before any
        // mutation).
        let mut g = self.inner.lock().map_err(|_| poisoned())?;
        for (k, v) in pairs {
            g.insert(k.clone(), v.clone());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_delete_round_trip() {
        let b = InMemoryBackend::new();
        assert_eq!(b.get(b"missing").unwrap(), None);
        b.put(b"k1", b"v1").unwrap();
        b.put(b"k2", b"v2").unwrap();
        assert_eq!(b.get(b"k1").unwrap().as_deref(), Some(&b"v1"[..]));
        b.delete(b"k1").unwrap();
        assert_eq!(b.get(b"k1").unwrap(), None);
        // Idempotent delete: no error on absent key.
        b.delete(b"k1").unwrap();
    }

    #[test]
    fn scan_empty_prefix_returns_all_keys_in_order() {
        let b = InMemoryBackend::new();
        b.put(b"b", b"2").unwrap();
        b.put(b"a", b"1").unwrap();
        b.put(b"c", b"3").unwrap();
        let r = b.scan(b"").unwrap();
        let pairs: Vec<_> = r.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        assert_eq!(
            pairs,
            vec![
                (b"a".to_vec(), b"1".to_vec()),
                (b"b".to_vec(), b"2".to_vec()),
                (b"c".to_vec(), b"3".to_vec()),
            ]
        );
    }

    #[test]
    fn scan_with_prefix_bounds_correctly() {
        let b = InMemoryBackend::new();
        b.put(b"foo:1", b"a").unwrap();
        b.put(b"foo:2", b"b").unwrap();
        b.put(b"food", b"c").unwrap();
        b.put(b"bar", b"d").unwrap();
        let r = b.scan(b"foo:").unwrap();
        let keys: Vec<_> = r.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(keys, vec![b"foo:1".to_vec(), b"foo:2".to_vec()]);
    }

    #[test]
    fn scan_all_0xff_prefix_unbounded_upper() {
        // `next_prefix(&[0xff])` returns None — exercise the open-upper
        // arm of scan.
        let b = InMemoryBackend::new();
        b.put(&[0xff, 0x00], b"a").unwrap();
        b.put(&[0xff, 0x01], b"b").unwrap();
        b.put(&[0xfe, 0x99], b"c").unwrap();
        let r = b.scan(&[0xff]).unwrap();
        let keys: Vec<_> = r.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(keys, vec![vec![0xff, 0x00], vec![0xff, 0x01]]);
    }

    #[test]
    fn put_batch_atomic_visibility() {
        let b = InMemoryBackend::new();
        b.put_batch(&[
            (b"a".to_vec(), b"1".to_vec()),
            (b"b".to_vec(), b"2".to_vec()),
        ])
        .unwrap();
        assert_eq!(b.get(b"a").unwrap().as_deref(), Some(&b"1"[..]));
        assert_eq!(b.get(b"b").unwrap().as_deref(), Some(&b"2"[..]));
    }
}
