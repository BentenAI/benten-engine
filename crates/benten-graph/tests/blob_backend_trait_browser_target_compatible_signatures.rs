//! Phase-3 G13-pre-B — `BlobBackend` browser-target signature compat pin.
//!
//! ## What this pins
//!
//! The trait MUST compile + be implementable on `wasm32-unknown-unknown`
//! so the G18-A IndexedDB-browser variant can land in a later wave
//! without re-litigating the API shape. Two specific risks the test
//! defends against:
//!
//! 1. **Native-only async-runtime deps creeping into the trait surface.**
//!    The trait's RPITIT `impl Future + Send` shape lets the IndexedDB
//!    impl wrap a `wasm-bindgen-futures::JsFuture` (which is inherently
//!    async on the JS event-loop wire) without dragging tokio /
//!    `std::thread` / `std::sync::Mutex` into the trait itself. This
//!    test confirms the trait CAN be implemented with futures backed by
//!    a manual poll-state-machine (the [`InstantReadyFuture`] /
//!    [`InstantWriteFuture`] types here mirror what
//!    `wasm-bindgen-futures` does internally — instantly-resolving
//!    futures that don't need a multi-threaded executor).
//!
//! 2. **Type-system primitives the trait references being native-only.**
//!    `benten_core::Cid` + `Vec<u8>` + `core::future::Future` + thiserror
//!    typed-error enums are all wasm32-compatible (verified via existing
//!    `wasm-checks.yml` coverage of `benten-core` + `benten-errors`).
//!    This test compiles a backend impl using only those primitives,
//!    confirming the trait's surface stays inside the wasm32-compatible
//!    waist.
//!
//! ## Browser-target compile verification
//!
//! The actual `wasm32-unknown-unknown` build verification happens at
//! orchestrator pre-flight + CI extension (per dispatch-conventions §3.5
//! dimension 3 + the existing `wasm-checks.yml` workflow). This test
//! file's role is to PIN the SIGNATURES — proving the trait surface can
//! be implemented with poll-state-machine-based futures (the canonical
//! shape `wasm-bindgen-futures::JsFuture` exposes) without leaning on
//! native-only async machinery. The wasm32 target compile catches any
//! transitive dep regression; the signature pin here catches a refactor
//! that adds a non-wasm32-compatible bound to the trait surface.

#![allow(dead_code, clippy::unwrap_used)]

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use benten_core::Cid;
use benten_graph::BlobBackend;

/// Manual poll-state-machine future that resolves immediately on the
/// first poll. Mirrors the shape `wasm-bindgen-futures::JsFuture` exposes
/// (a future backed by an `IDBRequest` callback that flips a state
/// machine on the JS event loop) without depending on any wasm-specific
/// crate. If this compiles, the trait's `impl Future` shape accepts the
/// browser-target implementation strategy.
struct InstantReadyFuture<T>(Option<T>);

impl<T: Unpin> Future for InstantReadyFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.get_mut().0.take().expect("polled after Ready"))
    }
}

/// Browser-shaped backend impl using only manual-poll futures (no tokio,
/// no `async fn`). Compilation here confirms the trait's surface is
/// implementable on `wasm32-unknown-unknown` where async runtimes are
/// constrained.
struct ManualPollBlobBackend;

impl BlobBackend for ManualPollBlobBackend {
    type Error = ManualPollError;

    fn get(&self, _cid: &Cid) -> impl Future<Output = Result<Option<Vec<u8>>, Self::Error>> + Send {
        InstantReadyFuture(Some(Ok(None)))
    }

    fn put(
        &self,
        _cid: &Cid,
        _bytes: &[u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        InstantReadyFuture(Some(Ok(())))
    }

    fn is_persistent(&self) -> bool {
        // Browser thin-client cache per CLAUDE.md baked-in #17 — false.
        false
    }
}

/// Typed-error enum demonstrating the `thiserror`-derived shape G18-A's
/// IndexedDB impl will use. `thiserror` is wasm32-compatible (used by
/// `benten-errors` which compiles to wasm32 today).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
enum ManualPollError {
    /// Reserved variant — IndexedDB call failed at the JS wire. The real
    /// G18-A variant carries the underlying `JsValue` reason; the stub
    /// shape here is enough to confirm the typed-error pattern compiles.
    #[error("manual-poll backend: indexeddb wire error")]
    Wire,
}

/// Drives the manual-poll future to completion via direct `poll`. No
/// executor needed since [`InstantReadyFuture`] resolves on the first
/// poll — this is exactly the shape the IndexedDB impl uses (the
/// `IDBRequest` `success` callback flips the state machine + `wake`s
/// the task; the next poll resolves).
fn drive_to_ready<F: Future>(fut: F) -> F::Output {
    let mut fut = Box::pin(fut);
    let waker = std::task::Waker::noop();
    let mut cx = Context::from_waker(waker);
    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(out) => out,
        Poll::Pending => panic!("InstantReadyFuture must resolve on first poll"),
    }
}

#[test]
fn manual_poll_browser_backend_implements_blob_backend() {
    let backend = ManualPollBlobBackend;
    assert!(
        !backend.is_persistent(),
        "browser thin-client cache reports is_persistent=false per CLAUDE.md baked-in #17"
    );

    let cid = benten_core::testing::canonical_test_node().cid().unwrap();
    let got = drive_to_ready(backend.get(&cid)).expect("get must not fail in stub");
    assert!(got.is_none(), "stub backend reports clean miss");

    let bytes = b"phase-3 g13-pre-b smoke";
    drive_to_ready(backend.put(&cid, bytes)).expect("put must not fail in stub");
}

/// Compile-time assertion that the trait's `get` / `put` futures are
/// `Send` + heap-pin-able — the IndexedDB-browser variant runs on the
/// JS event loop (single-threaded conceptually but `wasm-bindgen`
/// requires `Send` on cross-task moves), and this assertion catches a
/// refactor that strips the `+ Send` bound from the RPITIT signature.
/// Note: the future borrows the backend / cid for its lifetime
/// (consistent with `&self` / `&Cid` / `&[u8]` parameter shape) so the
/// assertion uses a lifetime parameter rather than `'static`.
fn _compile_assert_signatures_are_browser_friendly<'a, B: BlobBackend>(
    backend: &'a B,
    cid: &'a Cid,
    bytes: &'a [u8],
) {
    fn assert_send<T: Send>(_: &T) {}
    // Both futures are `Send` so they survive the wasm-bindgen-futures
    // cross-task move (and on native, tokio's worker-thread scheduling).
    let g = backend.get(cid);
    assert_send(&g);
    let p = backend.put(cid, bytes);
    assert_send(&p);
    // Heap-pinning into a Send-bound dyn future works — that's the
    // shape `wasm-bindgen-futures::JsFuture` uses internally.
    let _gp: Pin<Box<dyn Future<Output = _> + Send + 'a>> = Box::pin(g);
    let _pp: Pin<Box<dyn Future<Output = _> + Send + 'a>> = Box::pin(p);
}
