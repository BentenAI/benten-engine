//! R3-A RED-PHASE pin: `BlobBackend` (G13-pre-B trait scaffold) consumed
//! under generic-cascade by G14-C engine integration (forward-looking
//! cross-wave seam pin).
//!
//! Pin source: R3-A scope per dispatch brief — the `BlobBackend` trait
//! ships at G13-pre-B (already merged on origin/main); G14-C
//! `register_module_bytes` consumes it under generic-cascade. This test
//! pins the seam pre-G14-C so the consumer-side surface is locked.
//!
//! ## Why this exists
//!
//! Mirrors the G13-pre-B `blob_backend_trait_object_safety_per_d_resolution.rs`
//! pattern (positive-direction generic-cascade smoke), but specifically
//! pins a CONSUMER-shape function signature — the kind of function
//! G14-C will ship at `Engine::register_module_bytes`'s storage-side
//! call site. The test compiles AGAINST THE CURRENT scaffold (no G14-C
//! code yet) so the seam is verified before the consumer wave starts.

#![allow(dead_code, clippy::unwrap_used)]

use std::convert::Infallible;

use benten_core::Cid;
use benten_graph::BlobBackend;

/// Stub `BlobBackend` impl whose only job is to satisfy the trait
/// bound for this test's compile-time consumer check.
struct StubBackend;

impl BlobBackend for StubBackend {
    type Error = Infallible;

    async fn get(&self, _cid: &Cid) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }

    async fn put(&self, _cid: &Cid, _bytes: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_persistent(&self) -> bool {
        true
    }
}

/// G14-C consumer-shape function: takes a generic backend bound and
/// composes BlobBackend under the cascade. This is what
/// `Engine::register_module_bytes` will look like at G14-C landing
/// — the trait-object alternative (`Arc<dyn BlobBackend>`) does NOT
/// compile because of the associated type per D-PHASE-3-1 RESOLVED.
async fn g14c_consumer_shape<B: BlobBackend>(
    backend: &B,
    cid: &Cid,
    bytes: &[u8],
) -> Result<bool, B::Error> {
    backend.put(cid, bytes).await?;
    let _ = backend.get(cid).await?;
    Ok(backend.is_persistent())
}

#[test]
fn blob_backend_consumed_under_generic_cascade_at_g14_c() {
    // Currently RUNNABLE (not #[ignore]'d) because G13-pre-B's
    // BlobBackend trait scaffold is already on main. This test pins
    // that the trait is GENERIC-USABLE in the consumer-shape that
    // G14-C will ship. If a future refactor breaks generic-cascade
    // (e.g. adds an unbounded associated type), this test fails to
    // compile loudly.
    //
    // Invokes the consumer function via a tiny zero-dep poll driver
    // (matching the G13-pre-B test pattern — no tokio dependency).
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll, Waker};

    fn block_on<F: Future>(fut: F) -> F::Output {
        let mut fut = Box::pin(fut);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(out) => out,
            Poll::Pending => panic!("stub future returned Pending"),
        }
    }

    let backend = StubBackend;
    let cid = benten_core::testing::canonical_test_node().cid().unwrap();
    let bytes = vec![0u8; 16];
    let persistent = block_on(g14c_consumer_shape(&backend, &cid, &bytes)).unwrap();
    assert!(persistent);
}

/// Compile-time pin: a future PR cannot construct `Box<dyn BlobBackend>`
/// because the associated `type Error` precludes object-safety.
///
/// This function MUST NOT compile. We don't instantiate it — we just
/// rely on the type system rejecting the `dyn` form. The function is
/// commented out so the test file compiles; future contributors can
/// uncomment to verify the negative direction.
///
/// ```compile_fail
/// fn _negative_pin() -> Box<dyn benten_graph::BlobBackend> { todo!() }
/// ```
fn _negative_direction_documentation() {}
