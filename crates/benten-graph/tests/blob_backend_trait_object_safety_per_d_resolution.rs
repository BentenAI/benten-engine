//! Phase-3 G13-pre-B — `BlobBackend` trait NOT-object-safe pin
//! (D-PHASE-3-1 RESOLVED at R1-revision 2026-05-04).
//!
//! ## What this pins
//!
//! The trait surface is consumed via **generic-cascade** —
//! `fn install<B: BlobBackend>(...)` — never via `Arc<dyn BlobBackend>` /
//! `Box<dyn BlobBackend>`. D-PHASE-3-1 RESOLVED (R5-brief-time RECOMMEND
//! ratified at R1) names the generic-cascade direction: the trait carries
//! an associated `type Error` (so each backend surfaces its own typed-
//! error enum without `Box<dyn Error>` erasure inside the generic impl
//! per D-PHASE-3-1a) AND its methods return `impl Future + Send` (RPITIT,
//! stable since 1.75 — also incompatible with `dyn` erasure). Both
//! independently preclude object-safety.
//!
//! This integration test pins the **positive direction** of that design:
//! the trait WORKS as a generic bound. The non-object-safety property
//! itself is guaranteed *by construction* by the type system (the
//! compiler rejects `dyn BlobBackend` because of the associated type +
//! RPITIT) and does not need a separate runtime assertion. The risk this
//! test defends against is "someone refactors the trait to drop the
//! associated type or the RPITIT methods, accidentally enabling
//! `dyn BlobBackend`, and the engine starts compiling under
//! `Arc<dyn BlobBackend>` instead of `Engine<B>` cascade" — that
//! refactor would still pass this positive smoke, but the engine-side
//! generic-cascade tests at G13-B (`engine_generic_compiles_with_redb_
//! default` per plan §3 G13-B row) would catch it because they assert
//! the inverse: the engine compiles ONLY under generic-cascade. Together
//! the pins lock the design from both directions.
//!
//! Async tests are driven by a tiny zero-dep poll driver
//! ([`block_on_polling`]) so this test file does not pull tokio into
//! `benten-graph`'s dev-dep graph (consistent with `chunk_sink.rs`'s
//! tokio-free posture per its module docstring decision: "avoids dragging
//! tokio into `benten-eval`"; same rationale here).

#![allow(dead_code, clippy::unwrap_used)]

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use benten_core::Cid;
use benten_graph::BlobBackend;

/// Tokio-free poll-driver. Polls `fut` to completion using
/// [`Waker::noop`] (stable since 1.85; benten-engine MSRV is 1.94+).
/// Spins on `Pending`; the trait's stub impls below return immediately
/// from the inner future so the spin never actually loops more than
/// once. Suitable only for tests that exercise futures with no real
/// async wait.
fn block_on_polling<F: Future>(fut: F) -> F::Output {
    let mut fut = Box::pin(fut);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(out) => out,
        // Stub-impl futures resolve on the first poll; if a real future
        // returned Pending we'd want a real executor — panic so the
        // regression is loud rather than spinning.
        Poll::Pending => panic!("test stub future returned Pending; use a real executor"),
    }
}

/// Stub `BlobBackend` impl whose only job is to prove the trait CAN be
/// implemented. Returns hard-coded values; persistence claim is
/// configurable so the test exercises both branches of the
/// thin-client-vs-full-peer split (CLAUDE.md baked-in #17).
struct StubBlobBackend {
    persistent: bool,
}

impl BlobBackend for StubBlobBackend {
    type Error = Infallible;

    async fn get(&self, _cid: &Cid) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }

    async fn put(&self, _cid: &Cid, _bytes: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_persistent(&self) -> bool {
        self.persistent
    }
}

/// Generic-cascade smoke: `fn<B: BlobBackend>(...)` compiles + the bound
/// is exercised through `get` / `put` / `is_persistent`. This is the
/// **positive direction** of D-PHASE-3-1 RESOLVED — generic-cascade
/// works.
async fn install_via_generic_cascade<B: BlobBackend>(backend: &B, cid: &Cid, bytes: &[u8]) -> bool {
    let _ = backend.put(cid, bytes).await;
    let _ = backend.get(cid).await;
    backend.is_persistent()
}

#[test]
fn blob_backend_works_under_generic_cascade_per_d_phase_3_1() {
    let backend = StubBlobBackend { persistent: true };
    let cid = benten_core::testing::canonical_test_node().cid().unwrap();
    let bytes = vec![0u8; 8];
    let persistent = block_on_polling(install_via_generic_cascade(&backend, &cid, &bytes));
    assert!(
        persistent,
        "generic-cascade direction must surface is_persistent() through the bound"
    );
}

#[test]
fn blob_backend_thin_client_branch_returns_false_for_is_persistent() {
    // Pins the CLAUDE.md baked-in #17 thin-client commitment: a non-
    // persistent backend (browser cache shape) reports false.
    let backend = StubBlobBackend { persistent: false };
    assert!(!backend.is_persistent());
    let cid = benten_core::testing::canonical_test_node().cid().unwrap();
    let _miss = block_on_polling(backend.get(&cid));
}

/// Compile-time assertion that the trait carries `Send + Sync + 'static`
/// — these auto-trait bounds are part of the public contract (the
/// engine holds the backend across `.await` points + worker threads) and
/// a refactor that drops them must fail this file's compilation.
fn _compile_assertassert_send_sync_static<B: BlobBackend>() {
    fn assert_send_sync_static<T: Send + Sync + 'static + ?Sized>() {}
    assert_send_sync_static::<B>();
}

/// Compile-time assertion that the futures returned by `get` / `put` are
/// `Send` — required so the engine's tokio worker threads can move them
/// across thread boundaries. RPITIT (`impl Future + Send`) bakes this
/// in; the assertion catches a refactor that strips the `+ Send` bound.
fn _compile_assert_get_put_futures_are_send<B: BlobBackend>(backend: &B, cid: &Cid, bytes: &[u8]) {
    fn assert_send<T: Send>(_: &T) {}
    let g = backend.get(cid);
    assert_send(&g);
    let p = backend.put(cid, bytes);
    assert_send(&p);
    // Pin both to suppress unused warnings under strict clippy.
    let _gp: Pin<Box<dyn Future<Output = _> + Send>> = Box::pin(g);
    let _pp: Pin<Box<dyn Future<Output = _> + Send>> = Box::pin(p);
}
