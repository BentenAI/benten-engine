//! R3-A RED-PHASE pins for `BrowserBackend` thin-client cache
//! (G13-C wave-3; CLAUDE.md baked-in #17 + br-r1-* + plan §3 G13-C).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-C + plan §4 seed):
//!
//! - `tests/browser_backend_round_trip` — plan §3 G13-C
//! - `tests/prop_browser_backend_round_trip_arbitrary_node_set` — plan §4 seed
//! - `tests/browser_backend_thin_client_cache_no_transaction_atomicity_required` — baked-in #17
//! - `tests/browser_backend_subscriber_registry_returns_no_op_per_thin_client_scope` — baked-in #17
//! - `tests/browser_backend_no_redb_dep_on_wasm32_unknown_unknown` — `br-r1-1` BLOCKER
//!
//! ## Thin-client cache scope (CLAUDE.md baked-in #17)
//!
//! `BrowserBackend` is an in-RAM `BTreeMap<Cid, NodeBytes>` for thin-
//! client cache use ONLY:
//!
//! - **No transactions:** returns a no-op transaction handle (no atomic
//!   commit/rollback). Browser tabs are thin-client views of full peers;
//!   the full peer is the source of truth for atomicity.
//! - **No subscribers:** registering a subscriber returns a fan-out
//!   that produces no events (browser tabs subscribe to the full peer
//!   over SSE/WebSocket per G14-D thin-client subscription, not to
//!   their own local cache).
//! - **No sync state:** sync is full-peer-only per baked-in #17;
//!   browser-side BrowserBackend never participates in iroh/Loro/MST.

#![allow(clippy::unwrap_used, unreachable_code)]

use proptest::prelude::*;

#[test]
#[ignore = "RED-PHASE: G13-C wave-3 introduces benten_graph::BrowserBackend"]
fn browser_backend_round_trip() {
    // G13-C implementer wires this:
    //   let backend = benten_graph::BrowserBackend::new();
    //   let cid = benten_core::testing::canonical_test_node().cid().unwrap();
    //   let bytes = vec![1u8, 2, 3, 4];
    //   backend.put(b"n:cid_bytes", &bytes).unwrap();
    //   let got = backend.get(b"n:cid_bytes").unwrap();
    //   assert_eq!(got, Some(bytes));
    //
    // OBSERVABLE consequence: in-RAM BTreeMap round-trip works for
    // node body get/put. Defends against G13-C accidentally shipping
    // a backend that drops writes on the floor.
    unimplemented!("G13-C wires BrowserBackend put/get round-trip assertion");
}

#[test]
#[ignore = "RED-PHASE: G13-C — baked-in #17 — no transaction atomicity required"]
fn browser_backend_thin_client_cache_no_transaction_atomicity_required() {
    // CLAUDE.md baked-in #17 pin. `BrowserBackend.transaction()`
    // returns a no-op handle that does not commit atomically.
    // Concurrent writes to the same key from different tasks may race
    // — the thin-client semantic permits that (the full peer is the
    // source of truth for atomicity).
    //
    // G13-C implementer wires this:
    //   let backend = benten_graph::BrowserBackend::new();
    //   let txn = backend.transaction();
    //   // No-op: txn.commit() is permitted but does not enforce atomicity.
    //   // The contract here is observable BEHAVIORAL absence — the
    //   // backend documents "no atomicity guarantees on browser thin-
    //   // client cache" and the test pins that documentation.
    //
    // OBSERVABLE consequence: a future PR that adds atomic commit/
    // rollback to BrowserBackend would either (a) inflate the
    // browser bundle by wiring the full transaction module (defying
    // bundle-cap pin), or (b) silently break the thin-client
    // semantic. Both fail this test's accompanying source-cite
    // assertion.
    unimplemented!("G13-C wires BrowserBackend no-atomicity behavioral pin");
}

#[test]
#[ignore = "RED-PHASE: G13-C — baked-in #17 — subscriber registry no-ops"]
fn browser_backend_subscriber_registry_returns_no_op_per_thin_client_scope() {
    // CLAUDE.md baked-in #17 pin. Browser-side BrowserBackend is a
    // thin-client cache; subscribers go to the full peer over the
    // wire (G14-D thin-client subscription), NOT to local cache
    // mutations. Local subscribers register but receive no events.
    //
    // G13-C implementer wires this:
    //   let backend = benten_graph::BrowserBackend::new();
    //   let (tx, rx) = std::sync::mpsc::channel();
    //   struct Sub(std::sync::mpsc::Sender<benten_graph::ChangeEvent>);
    //   impl benten_graph::ChangeSubscriber for Sub {
    //       fn on_change(&self, e: &benten_graph::ChangeEvent) {
    //           let _ = self.0.send(e.clone());
    //       }
    //   }
    //   backend.register_subscriber(std::sync::Arc::new(Sub(tx)));
    //   // Now write something:
    //   backend.put(b"n:test", b"data").unwrap();
    //   // The subscriber registry IS a no-op fan-out:
    //   assert!(rx.try_recv().is_err(),
    //       "BrowserBackend MUST NOT fan out local writes to local subscribers \
    //        per CLAUDE.md baked-in #17 thin-client commitment");
    //
    // OBSERVABLE consequence: a future regression that wires local
    // fan-out (e.g. for "responsive UI") would either fight with the
    // full-peer subscription (double-fire) or silently change the
    // browser-tab UX. Both fail this test.
    unimplemented!("G13-C wires BrowserBackend subscriber-registry no-op pin");
}

#[test]
#[ignore = "RED-PHASE: G13-C — br-r1-1 BLOCKER — no redb dep on wasm32"]
fn browser_backend_no_redb_dep_on_wasm32_unknown_unknown() {
    // br-r1-1 BLOCKER pin. The whole point of BrowserBackend is to
    // give the browser bundle a backend that does NOT pull redb
    // (which is native-only because of file-I/O syscalls + mmap).
    //
    // G13-C implementer wires this as a Cargo.toml + cfg-attr
    // grep assertion:
    //
    //   let manifest = std::fs::read_to_string("crates/benten-graph/Cargo.toml").unwrap();
    //   // The browser-backend feature MUST NOT depend on redb:
    //   // grep that the [features] block routes browser-backend to
    //   // a deps subset that excludes redb.
    //
    //   // Plus a target-conditional check:
    //   let lib_src = std::fs::read_to_string("crates/benten-graph/src/lib.rs").unwrap();
    //   // RedbBackend re-export is gated by `#[cfg(not(target_arch = "wasm32"))]`.
    //   assert!(lib_src.contains("#[cfg(not(target_arch = \"wasm32\"))]")
    //         || lib_src.contains("#[cfg(any(not(target_arch = \"wasm32\"), feature = ..."));
    //
    // OBSERVABLE consequence: `cargo check --target wasm32-unknown-unknown
    // -p benten-graph --features browser-backend --no-default-features`
    // succeeds without compiling redb. The CI extension to wasm-checks.yml
    // is the authoritative verifier; this in-Rust test is the
    // source-side cite-assertion regression guard.
    unimplemented!("G13-C wires Cargo.toml + cfg-attr assertion that redb is excluded on wasm32");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2_000))]

    #[test]
    #[ignore = "RED-PHASE: G13-C — plan §4 seed — proptest round-trip"]
    fn prop_browser_backend_round_trip_arbitrary_node_set(
        keys in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 1..32), 0..16),
        bytes in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 0..256), 0..16),
    ) {
        // G13-C implementer wires this:
        //   let backend = benten_graph::BrowserBackend::new();
        //   for (k, v) in keys.iter().zip(bytes.iter()) {
        //       backend.put(k, v).unwrap();
        //   }
        //   for (k, v) in keys.iter().zip(bytes.iter()) {
        //       prop_assert_eq!(backend.get(k).unwrap().as_deref(), Some(v.as_slice()));
        //   }
        //
        // OBSERVABLE consequence: 2 000 cases × 0-16 keys = up to
        // 32 000 put/get operations over arbitrary byte sequences;
        // every put is recoverable via get.
        let _ = (keys, bytes);
        unimplemented!("G13-C wires BrowserBackend put/get proptest round-trip");
    }
}
