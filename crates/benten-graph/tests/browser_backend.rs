//! G13-C GREEN-PHASE pins for `BrowserBackend` thin-client cache
//! (Phase-3 R5 wave-3; CLAUDE.md baked-in #17 + br-r1-* + plan §3 G13-C).
//!
//! Originating decision context: `.addl/phase-2b/wave-8j-wasm-browser-bundle-bisect.md`
//! §Phase-3-followup — the Phase-2b retrospective surfaced the
//! "Engine hard-bound to RedbBackend" posture as the load-bearing
//! cause of the 600KB browser-bundle blow-up. PHASE-3-BUNDLE-1 closes
//! that gap by introducing `BrowserBackend` as a non-redb backend
//! routed via the [`benten_graph::GraphBackend`] umbrella substitution
//! on `wasm32-unknown-unknown` per pim-1 §3.5b doc-coupling.
//!
//! Pin sources (per r2-test-landscape §2.1 G13-C + plan §4 seed):
//!
//! - `tests/browser_backend_round_trip` — plan §3 G13-C
//! - `tests/prop_browser_backend_round_trip_arbitrary_node_set` — plan §4 seed
//! - `tests/browser_backend_thin_client_cache_no_transaction_atomicity_required` — baked-in #17
//! - `tests/browser_backend_subscriber_registry_returns_no_op_per_thin_client_scope` — baked-in #17
//! - `tests/browser_backend_no_redb_dep_on_wasm32_unknown_unknown` — `br-r1-1` BLOCKER
//! - `tests/browser_backend_snapshot_returns_owned_btreemap_clone_independent_of_live_writes` — `br-r4-r1-1` / `br-r4-r2-1` MAJOR
//! - `tests/browser_backend_put_node_with_context_thin_client_cache_semantic_pinned` — `br-r4-r1-1` / `br-r4-r2-1` MAJOR
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

#![cfg(feature = "browser-backend")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use benten_core::testing::canonical_test_node;
use benten_graph::{
    BrowserBackend, ChangeEvent, ChangeSubscriber, GraphBackend, KVBackend, NodeStore, WriteContext,
};
use proptest::prelude::*;

/// Test-only no-op subscriber that counts received events.
struct CountingSub(AtomicUsize);

impl ChangeSubscriber for CountingSub {
    fn on_change(&self, _event: &ChangeEvent) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn browser_backend_round_trip() {
    let backend = BrowserBackend::new();
    let bytes = vec![1u8, 2, 3, 4];
    backend.put(b"n:cid_bytes", &bytes).unwrap();
    let got = backend.get(b"n:cid_bytes").unwrap();
    assert_eq!(got, Some(bytes));

    // Also exercise the NodeStore round-trip path so the get/put loop
    // is observable through both byte-level KV and Node-level entry
    // points (defends against silent disagreement between the two).
    let node = canonical_test_node();
    let cid = backend.put_node(&node).unwrap();
    assert_eq!(backend.get_node(&cid).unwrap().as_ref(), Some(&node));
}

#[test]
fn browser_backend_thin_client_cache_no_transaction_atomicity_required() {
    // CLAUDE.md baked-in #17 pin. `BrowserBackend.transaction()`
    // returns a no-op handle that does not commit atomically.
    //
    // The OBSERVABLE consequence is that the runner type is the
    // designated `BrowserTransactionRunner` marker (not a
    // `RedbTransactionRunner` or any closure-based execution surface)
    // — getting one is permitted but does not mint any commit-ordering
    // semantic against a parallel writer. We assert the marker shape
    // directly + observe that interleaved writes through the public
    // KVBackend surface succeed without hitting any transaction-conflict
    // failure path.
    let backend = BrowserBackend::new();
    let _runner: benten_graph::BrowserTransactionRunner = backend.transaction();
    // Two interleaved writes — without any transaction wrapping —
    // both land. There is no commit-step that can fail for a "missing
    // begin" or "double-commit" reason because the marker is unit:
    backend.put(b"n:a", b"1").unwrap();
    backend.put(b"n:b", b"2").unwrap();
    assert_eq!(backend.get(b"n:a").unwrap().as_deref(), Some(&b"1"[..]));
    assert_eq!(backend.get(b"n:b").unwrap().as_deref(), Some(&b"2"[..]));

    // Source-cite assertion: the browser-backend module declares the
    // thin-client commitment in its docstring — protect that prose
    // against silent removal per pim-1 §3.5b doc-coupling.
    let src =
        std::fs::read_to_string("src/browser_backend.rs").expect("read src/browser_backend.rs");
    assert!(
        src.contains("CLAUDE.md baked-in #17") && src.contains("No transactions"),
        "src/browser_backend.rs MUST cite CLAUDE.md baked-in #17 + 'No transactions' per pim-1 §3.5b"
    );
}

#[test]
fn browser_backend_subscriber_registry_returns_no_op_per_thin_client_scope() {
    // CLAUDE.md baked-in #17 pin. Registering a subscriber must NOT
    // produce any local fan-out — browser tabs subscribe via the full
    // peer (G14-D), not via local-cache writes.
    let backend = BrowserBackend::new();
    let sub = Arc::new(CountingSub(AtomicUsize::new(0)));
    backend.register_subscriber(sub.clone());

    // Now write something through every public mutation path — none
    // of them must trigger the subscriber:
    backend.put(b"n:test", b"data").unwrap();
    backend
        .put_batch(&[
            (b"n:a".to_vec(), b"1".to_vec()),
            (b"n:b".to_vec(), b"2".to_vec()),
        ])
        .unwrap();
    backend.delete(b"n:test").unwrap();
    let node = canonical_test_node();
    let _ = backend.put_node(&node).unwrap();
    let ctx = WriteContext::new("post");
    let _ = backend.put_node_with_context(&node, &ctx).unwrap();

    assert_eq!(
        sub.0.load(Ordering::SeqCst),
        0,
        "BrowserBackend MUST NOT fan out local writes to local subscribers per CLAUDE.md baked-in #17"
    );
}

#[test]
fn browser_backend_no_redb_dep_on_wasm32_unknown_unknown() {
    // br-r1-1 BLOCKER pin. The whole point of BrowserBackend is to
    // give the browser bundle a backend that does NOT pull redb
    // (which is native-only because of file-I/O syscalls + mmap).
    //
    // Source-cite assertion against `crates/benten-graph/Cargo.toml`:
    // the `redb` dependency MUST be declared under a target-conditional
    // `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`
    // section (or the wasi-shape equivalent) — NOT under the
    // unconditional `[dependencies]` block.
    let manifest = std::fs::read_to_string("Cargo.toml").expect("read Cargo.toml");

    // Find the unconditional `[dependencies]` block + verify redb is NOT present:
    let unconditional_block = extract_section(&manifest, "[dependencies]")
        .expect("unconditional [dependencies] block present");
    assert!(
        !contains_dep_named(&unconditional_block, "redb"),
        "redb MUST NOT appear in the unconditional [dependencies] block \
         per br-r1-1 BLOCKER (CLAUDE.md baked-in #17 thin-client commitment); \
         saw:\n{unconditional_block}"
    );

    // Verify the target-conditional non-wasm32 block IS present + DOES
    // declare redb (so native targets keep redb):
    let native_block = manifest
        .find("[target.'cfg(not(target_arch = \"wasm32\"))'.dependencies]")
        .expect("target-conditional non-wasm32 dependencies block present per br-r1-1");
    let after_native = &manifest[native_block..];
    let native_section = after_native
        .lines()
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with('['))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        contains_dep_named(&native_section, "redb"),
        "redb MUST be declared under [target.'cfg(not(target_arch = \"wasm32\"))'.dependencies] \
         per br-r1-1 BLOCKER; native section saw:\n{native_section}"
    );

    // Source-cite: lib.rs RedbBackend re-export is gated by
    // `#[cfg(any(not(target_arch = \"wasm32\"), target_os = \"wasi\"))]`
    // so the wasm32-unknown-unknown build does not see the symbol:
    let lib_src = std::fs::read_to_string("src/lib.rs").expect("read src/lib.rs");
    assert!(
        lib_src.contains("#[cfg(any(not(target_arch = \"wasm32\"), target_os = \"wasi\"))]")
            && lib_src.contains("pub use redb_backend::RedbBackend"),
        "src/lib.rs RedbBackend re-export MUST be gated by `cfg(any(not(target_arch = \"wasm32\"), target_os = \"wasi\"))` per br-r1-1 BLOCKER"
    );

    // OBSERVABLE consequence (CI-side): `cargo check --target
    // wasm32-unknown-unknown -p benten-graph --features browser-backend
    // --no-default-features` succeeds without compiling redb. The
    // wasm-checks.yml workflow + the in-Rust source-cite assertion
    // above are the regression-detection surface.
}

#[test]
fn browser_backend_snapshot_returns_owned_btreemap_clone_independent_of_live_writes() {
    // br-r4-r1-1 / br-r4-r2-1 MAJOR pin (Option-i Mutex-based shape
    // per fix-brief). `BrowserBackend::snapshot()` MUST return an owned
    // clone that is INDEPENDENT of subsequent live writes.
    let backend = BrowserBackend::new();
    backend.put(b"n:cid_a", &[1u8, 2, 3]).unwrap();
    backend.put(b"n:cid_b", &[4u8, 5, 6]).unwrap();

    // Take a snapshot (umbrella-trait surface):
    let snap = backend.snapshot();
    assert_eq!(snap.len(), 2);

    // Mutate the live backend AFTER the snapshot:
    backend.put(b"n:cid_c", &[7u8, 8, 9]).unwrap();
    backend.put(b"n:cid_a", &[100u8]).unwrap(); // overwrite

    // Snapshot is unchanged — it's an independent owned clone:
    assert_eq!(
        snap.len(),
        2,
        "snapshot len MUST NOT grow when live backend grows after snapshot"
    );
    assert_eq!(
        snap.get(b"n:cid_a"),
        Some(&[1u8, 2, 3][..]),
        "snapshot value MUST NOT mutate when live backend overwrites after snapshot"
    );
    assert_eq!(
        snap.get(b"n:cid_b"),
        Some(&[4u8, 5, 6][..]),
        "untouched key remains visible in snapshot"
    );
}

#[test]
fn browser_backend_put_node_with_context_thin_client_cache_semantic_pinned() {
    // br-r4-r1-1 / br-r4-r2-1 MAJOR pin (cap-recheck context arm).
    // `BrowserBackend::put_node_with_context(node, ctx)` is the
    // thin-client-cache write path for inbound subscription deliveries.
    //
    // The cap-recheck CONTRACT in this arm is intentionally distinct
    // from full-peer Backend::put_node_with_context:
    //
    //   - Full peer: cap-recheck IS the gate (cap-policy plug fires
    //     pre-write per Phase-2b PrimitiveHost contract).
    //   - Thin-client BrowserBackend: cap-recheck is BYPASSED by
    //     design — the upstream subscription (G14-D) already filters
    //     events per delivered-subscriber's grant; the local cache
    //     simply mirrors the authorized stream.
    //
    // Observable consequence #1: a non-privileged context for a
    // regular-label node round-trips successfully via
    // put_node_with_context — the cache layer accepts the bytes:
    let backend = BrowserBackend::new();
    let node = canonical_test_node();
    let ctx = WriteContext::new("post"); // user authority, non-privileged
    let cid = backend.put_node_with_context(&node, &ctx).unwrap();
    assert_eq!(node.cid().unwrap(), cid);
    assert_eq!(backend.get_node(&cid).unwrap().as_ref(), Some(&node));

    // Observable consequence #2: BrowserBackend's
    // `put_node_with_context` does NOT depend on the `benten-caps`
    // crate — defends against the regression where a future PR wires
    // cap-policy into the cache layer (which would inflate the browser
    // bundle by pulling benten-caps + UCAN backend transitive deps).
    //
    // Source-cite: the browser_backend.rs file MUST NOT contain a
    // `use benten_caps` import (would couple the browser bundle to
    // the cap-policy crate). Defends the `.addl/phase-3/spike-bundle-
    // cap-empirical.md` § 6 per-contributor budget for `benten_caps`
    // (≤ 80 KB raw / ~ 40 KB gz) — the cap glue MUST NOT cross from
    // the engine layer down into the storage layer.
    let src =
        std::fs::read_to_string("src/browser_backend.rs").expect("read src/browser_backend.rs");
    assert!(
        !src.contains("use benten_caps") && !src.contains("benten_caps::"),
        "browser_backend.rs MUST NOT import `benten_caps` per CLAUDE.md baked-in #17 \
         thin-client commitment + spike-bundle-cap-empirical.md §6 per-contributor budget"
    );

    // Observable consequence #3: the system-zone gate is preserved
    // (a non-privileged system-zone label fails loud rather than
    // caching system-zone bytes silently — defends against the
    // failure shape "thin-client subscription delivered a system-zone
    // event without privilege and the cache silently mirrored it"):
    let mut sys_node = canonical_test_node();
    sys_node.labels = vec!["system:Critical".into()];
    let ctx_sys = WriteContext::new("system:Critical"); // non-privileged
    let err = backend
        .put_node_with_context(&sys_node, &ctx_sys)
        .unwrap_err();
    assert!(
        matches!(err, benten_graph::GraphError::SystemZoneWrite { .. }),
        "non-privileged system-zone put MUST fire E_SYSTEM_ZONE_WRITE; saw {err:?}"
    );
}

#[test]
fn browser_backend_get_node_rejects_tampered_cache_bytes_with_content_hash_error() {
    // #1208 / #620 (Safe-3, META #660 W9-T6 slice) closure-pin.
    //
    // `BrowserBackend::get_node` MUST verify the stored bytes re-hash to
    // the requested CID (W9-T6 verify-on-read parity with
    // `RedbBackend::get_node`). The browser thin-client cache is an
    // attacker-adjacent surface — the in-RAM `BTreeMap` is reachable from
    // the JS heap / DevTools console / a malicious extension. Before the
    // #620 fix both `get_node` surfaces did a bare
    // `serde_ipld_dagcbor::from_slice` with NO content-hash check, so a
    // tampered cache entry produced a wrong-but-decodable Node SILENTLY.
    //
    // Threat-model reproduction: seed a Node, then overwrite the value
    // bytes under its `n:<cid>` key with the bytes of a DIFFERENT
    // (decodable) Node via the byte-level `KVBackend` surface — exactly
    // the shape a cache-tampering attacker achieves. The decode still
    // succeeds; only the content-hash recomputation catches the swap.
    //
    // Would-FAIL-if-reverted: pre-#620 `get_node` returned
    // `Ok(Some(node_b))` for `cid_a`; the `expect_err` below would panic
    // and the `ErrorCode::InvContentHash` assertion would never run.
    use benten_graph::ErrorCode;

    let backend = BrowserBackend::new();

    // Two distinct Nodes → distinct canonical bytes + distinct CIDs.
    let node_a = canonical_test_node();
    let cid_a = backend.put_node(&node_a).unwrap();

    let mut node_b = canonical_test_node();
    node_b.labels = vec!["TamperedSubstitute".into()];
    let (cid_b, bytes_b) = node_b.cid_and_canonical_bytes().unwrap();
    assert_ne!(cid_a, cid_b, "fixtures must differ for a real substitution");

    // Cache-tamper: write node_b's bytes UNDER node_a's key. `node_key`
    // is the `n:<cid>` schema both `get_node` surfaces resolve.
    let tampered_key = {
        let mut k = b"n:".to_vec();
        k.extend_from_slice(cid_a.as_bytes());
        k
    };
    backend.put(&tampered_key, &bytes_b).unwrap();

    // (1) NodeStore trait surface (generic-cascade-reachable read path).
    let err = NodeStore::get_node(&backend, &cid_a).expect_err(
        "tampered cache bytes MUST fail NodeStore::get_node — this is the #620 W9-T6 defense",
    );
    assert_eq!(
        err.code(),
        ErrorCode::InvContentHash,
        "trait get_node tamper MUST fire E_INV_CONTENT_HASH (got {:?})",
        err.code()
    );

    // (2) Snapshot surface (`BrowserSnapshot::get_node`) — the same
    // verify-on-read parity must hold for the owned-clone read path.
    let snap = GraphBackend::snapshot(&backend);
    let snap_err = snap.get_node(&cid_a).expect_err(
        "tampered cache bytes MUST fail BrowserSnapshot::get_node — #620 verify-on-read",
    );
    assert_eq!(
        snap_err.code(),
        ErrorCode::InvContentHash,
        "snapshot get_node tamper MUST fire E_INV_CONTENT_HASH (got {:?})",
        snap_err.code()
    );

    // Sanity: an UN-tampered entry still round-trips (the verify is a
    // gate on corruption, not a blanket reject — guards against an
    // over-broad fix that always errors).
    let cid_clean = backend.put_node(&node_b).unwrap();
    assert_eq!(
        NodeStore::get_node(&backend, &cid_clean).unwrap().as_ref(),
        Some(&node_b),
        "a correctly-keyed entry must still verify + decode"
    );
}

#[test]
fn browser_backend_lock_discipline_uses_lock_recover_no_poison_propagation() {
    // #1210 / #627 + #637 (Safe-4, META #707 slice) closure-pin.
    //
    // #627: `BrowserBackend::snapshot` previously returned an EMPTY
    //       BTreeMap on Mutex poison — masking a poisoned cache as an
    //       empty-graph view to every GraphBackend generic-cascade
    //       caller (a correctness landmine).
    // #637: 16+ BrowserBackend Mutex sites previously used
    //       `.lock().map_err(|_| poisoned())` propagating a typed poison
    //       error, inconsistent with the workspace `lock_recover()`
    //       recover-and-proceed discipline (35+ sites).
    //
    // The internal `Mutex` is not poisonable from the public API (no
    // backend method panics while holding the guard — same constraint
    // documented on `subscriber_count_uses_lock_recover`). So this pin
    // combines the established crate pattern: (1) a SOURCE assertion
    // that the regressed idioms are GONE and the recover idiom is
    // present (a revert to `map_err(|_| poisoned())` or an empty-on-
    // poison snapshot re-fires here), plus (2) a healthy-path
    // behavioural assertion that `snapshot()` reflects real state (the
    // #627 empty-snapshot bug would also fail consequence #2 if the
    // poison branch were re-introduced as an unconditional empty).
    //
    // Would-FAIL-if-reverted: re-introducing `map_err(|_| poisoned())`
    // (the #637 idiom) or a `BTreeMap::new()` poison fallback in
    // `snapshot` (the #627 bug) trips the source assertions below.
    let src =
        std::fs::read_to_string("src/browser_backend.rs").expect("read src/browser_backend.rs");
    // Scan only NON-comment code lines — the file's own #637 docstring
    // legitimately quotes the deleted idiom while explaining the fix.
    let code: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    // #637: the typed-poison idiom must be fully eradicated from CODE.
    assert!(
        !code.contains("map_err(|_| poisoned())"),
        "#637: BrowserBackend MUST NOT propagate typed poison — \
         every Mutex site uses the workspace lock_recover() discipline"
    );
    assert!(
        !code.contains("fn poisoned()"),
        "#637: the `poisoned()` GraphError-builder helper must be deleted"
    );
    // The recover idiom must be the one actually in use.
    assert!(
        code.contains(".lock_recover()"),
        "#637: BrowserBackend Mutex sites must use lock_recover()"
    );

    // #627: snapshot() must NOT have an empty-BTreeMap poison fallback;
    // it clones the recovered guard's real contents.
    assert!(
        code.contains("self.inner.lock_recover().clone()"),
        "#627: snapshot() must clone the lock_recover()'d real map, \
         not fabricate an empty BTreeMap on poison"
    );

    // (2) Healthy-path behavioural consequence: a populated cache's
    // snapshot reflects the real entries (a re-introduced empty-on-
    // poison branch that ever fired here would surface as len()==0).
    let backend = BrowserBackend::new();
    let node = canonical_test_node();
    let cid = backend.put_node(&node).unwrap();
    let snap = GraphBackend::snapshot(&backend);
    assert_eq!(
        snap.len(),
        1,
        "snapshot must reflect the one persisted node (not an empty poison view)"
    );
    assert_eq!(
        snap.get_node(&cid).unwrap().as_ref(),
        Some(&node),
        "snapshot must surface the real node, proving lock_recover() returned live state"
    );
}

proptest! {
    // The plan-§4 budget is 2 000 cases × 0-16 keys = up to 32 000
    // put/get operations over arbitrary byte sequences. Every case
    // asserts the deterministic round-trip identity (put followed by
    // get observes the same bytes for every key).
    #![proptest_config(ProptestConfig::with_cases(2_000))]

    #[test]
    fn prop_browser_backend_round_trip_arbitrary_node_set(
        keys in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 1..32), 0..16),
        bytes in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 0..256), 0..16),
    ) {
        let backend = BrowserBackend::new();
        let pairs: Vec<(Vec<u8>, Vec<u8>)> = keys.iter()
            .cloned()
            .zip(bytes.iter().cloned())
            .collect();

        for (k, v) in &pairs {
            backend.put(k, v).unwrap();
        }

        // Build a "last write wins" expectation map (proptest may
        // generate duplicate keys with different values):
        let mut expected: std::collections::BTreeMap<Vec<u8>, Vec<u8>> =
            std::collections::BTreeMap::new();
        for (k, v) in &pairs {
            expected.insert(k.clone(), v.clone());
        }

        for (k, v) in &expected {
            let got = backend.get(k).unwrap();
            prop_assert_eq!(got.as_deref(), Some(v.as_slice()));
        }
    }
}

// -- helpers --------------------------------------------------------------

/// Find a `[<section>]` in a Cargo.toml manifest and return the lines
/// belonging to it (until the next `[…]` header or EOF).
fn extract_section<'a>(manifest: &'a str, header: &str) -> Option<String> {
    let idx = manifest.find(header)?;
    let after = &manifest[idx + header.len()..];
    let body = after
        .lines()
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with('['))
        .collect::<Vec<_>>()
        .join("\n");
    Some(body)
}

/// Whether a Cargo.toml-style section block contains a dependency line
/// for `name`. Matches `name = ...` or `name = { ... }` at the start of
/// a non-comment, non-empty line.
fn contains_dep_named(block: &str, name: &str) -> bool {
    block.lines().any(|l| {
        let trimmed = l.trim_start();
        if trimmed.starts_with('#') {
            return false;
        }
        if let Some(rest) = trimmed.strip_prefix(name) {
            let rest = rest.trim_start();
            return rest.starts_with('=');
        }
        false
    })
}
