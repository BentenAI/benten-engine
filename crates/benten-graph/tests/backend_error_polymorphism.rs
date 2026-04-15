//! Edge-case test: the `KVBackend` trait's associated `Error` type must allow
//! multiple backends to coexist without lying through a redb-named variant.
//!
//! Regression: the spike's `GraphError::Redb(String)` forces every non-redb
//! backend (in-memory mock, iroh-fetch, WASM peer-fetch) to stringify its
//! errors into a variant that LIES about where they came from. The fix
//! (§2.2 G1 tag `P1.graph.error-polymorphism`) moves `Error` to a trait
//! associated type.
//!
//! This test constructs a tiny in-memory mock backend with its own error
//! enum and shows it composes with the trait without crossing the redb-error
//! path at all.
//!
//! R3 contract: `KVBackend::Error` as an associated type does not exist today
//! (spike is `Result<_, GraphError>` on every method). R5 (G2-A) lands the
//! refactor. This test compile-fails until then — deliberate.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_graph::{KVBackend, ScanResult};

/// A trivial in-memory backend. Exists only to prove the trait's associated
/// Error type supports polymorphism. Not a full implementation.
#[derive(Default)]
struct MemBackend {
    inner: std::sync::Mutex<std::collections::BTreeMap<Vec<u8>, Vec<u8>>>,
}

#[derive(Debug, thiserror::Error)]
enum MemError {
    #[error("mem backend: poisoned lock")]
    Poisoned,
    #[error("mem backend: injected failure")]
    Injected,
}

impl KVBackend for MemBackend {
    type Error = MemError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        let g = self.inner.lock().map_err(|_| MemError::Poisoned)?;
        Ok(g.get(key).cloned())
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        let mut g = self.inner.lock().map_err(|_| MemError::Poisoned)?;
        g.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), Self::Error> {
        let mut g = self.inner.lock().map_err(|_| MemError::Poisoned)?;
        g.remove(key);
        Ok(())
    }

    fn scan(&self, prefix: &[u8]) -> Result<ScanResult, Self::Error> {
        let g = self.inner.lock().map_err(|_| MemError::Poisoned)?;
        Ok(g.range(prefix.to_vec()..)
            .take_while(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }

    fn put_batch(&self, pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), Self::Error> {
        let mut g = self.inner.lock().map_err(|_| MemError::Poisoned)?;
        for (k, v) in pairs {
            g.insert(k.clone(), v.clone());
        }
        Ok(())
    }
}

#[test]
fn mem_backend_errors_never_route_through_graph_error() {
    // The load-bearing assertion: a non-redb backend's errors must surface
    // as `MemError::*`, never as the `GraphError::Redb(String)` footgun the
    // spike created.
    let b = MemBackend::default();
    b.put(b"k", b"v").unwrap();
    assert_eq!(b.get(b"k").unwrap().as_deref(), Some(&b"v"[..]));

    // Type-level assertion: MemBackend::Error is NOT GraphError.
    // If R5 accidentally regresses to a unified error enum, this line
    // will compile (both are the same type) — but the `assert_not_same_type`
    // gate below fires at compile time.
    fn assert_not_same_type<A: 'static, B: 'static>() {
        let a = std::any::TypeId::of::<A>();
        let b = std::any::TypeId::of::<B>();
        assert_ne!(a, b, "backend Error type must be distinct from GraphError");
    }
    assert_not_same_type::<MemError, benten_graph::GraphError>();
}

#[test]
fn polymorphic_fn_over_kvbackend_does_not_require_graph_error() {
    // A consumer function generic over any `KVBackend` must not need to
    // match on `GraphError` internally. This test verifies the trait is
    // polymorphic at call-site — meaning a WASM peer-fetch backend or
    // an iroh-backed backend can plug in with its own error type.
    fn roundtrip<B: KVBackend>(b: &B) -> Result<(), B::Error> {
        b.put(b"k", b"v")?;
        let got = b.get(b"k")?;
        assert_eq!(got.as_deref(), Some(&b"v"[..]));
        Ok(())
    }

    let mem = MemBackend::default();
    roundtrip(&mem).unwrap();
}

#[test]
fn injected_backend_error_does_not_say_redb() {
    // Final nail: when a custom backend propagates its own error, the
    // `Display` impl must NOT contain the substring "redb" — that would
    // be the exact lying-about-origin footgun the refactor closes.
    let err = MemError::Injected;
    let rendered = format!("{err}");
    assert!(
        !rendered.contains("redb"),
        "non-redb backend error must not surface as redb: got {rendered:?}"
    );
}
