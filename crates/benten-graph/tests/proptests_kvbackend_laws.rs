//! Proptest: `KVBackend` put/get/delete laws (R4 triage M16).
//!
//! Three algebraic laws every `KVBackend` implementation must obey:
//!   1. get-after-put: `put(k, v); get(k)` returns `Some(v)`.
//!   2. get-after-delete: `put(k, v); delete(k); get(k)` returns `None`.
//!   3. overwrite: `put(k, v1); put(k, v2); get(k)` returns `Some(v2)`.
//!
//! These are the KV-backend "Relational algebra laws" that every storage
//! implementation (redb today, in-memory mock, peer-fetch WASM backend
//! Phase 2+) must satisfy for the graph layer above to be correct.
//!
//! R3 writer: `rust-test-writer-proptest`.

#![allow(clippy::unwrap_used)]

use benten_graph::{KVBackend, RedbBackend};
use proptest::prelude::*;
use tempfile::TempDir;

fn temp() -> (RedbBackend, TempDir) {
    let d = tempfile::tempdir().unwrap();
    let b = RedbBackend::open(d.path().join("t.redb")).unwrap();
    (b, d)
}

proptest! {
    /// R4 triage M16 law 1: get-after-put.
    #[test]
    fn prop_kvbackend_put_get_delete_law_put_then_get(
        key in proptest::collection::vec(any::<u8>(), 1..32),
        value in proptest::collection::vec(any::<u8>(), 0..64),
    ) {
        let (backend, _d) = temp();
        backend.put(&key, &value).unwrap();
        let got = backend.get(&key).unwrap();
        prop_assert_eq!(got.as_deref(), Some(value.as_slice()));
    }

    /// R4 triage M16 law 2: get-after-delete returns None.
    #[test]
    fn prop_kvbackend_put_get_delete_law_delete_removes(
        key in proptest::collection::vec(any::<u8>(), 1..32),
        value in proptest::collection::vec(any::<u8>(), 0..64),
    ) {
        let (backend, _d) = temp();
        backend.put(&key, &value).unwrap();
        backend.delete(&key).unwrap();
        let got = backend.get(&key).unwrap();
        prop_assert!(got.is_none(), "delete(k) then get(k) must be None");
    }

    /// R4 triage M16 law 3: overwrite replaces previous value.
    #[test]
    fn prop_kvbackend_put_get_delete_law_overwrite(
        key in proptest::collection::vec(any::<u8>(), 1..32),
        v1 in proptest::collection::vec(any::<u8>(), 0..32),
        v2 in proptest::collection::vec(any::<u8>(), 0..32),
    ) {
        let (backend, _d) = temp();
        backend.put(&key, &v1).unwrap();
        backend.put(&key, &v2).unwrap();
        let got = backend.get(&key).unwrap();
        prop_assert_eq!(got.as_deref(), Some(v2.as_slice()));
    }
}
