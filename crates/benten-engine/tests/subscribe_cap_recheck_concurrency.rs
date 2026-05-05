//! R4-FP-R3-B RED-PHASE pin: cap_recheck closure non-blocking under
//! grant-store contention (G14-D wave-5a; cap-r4-5 MINOR closure of
//! cap-minor-1 fix-now-action).
//!
//! Pin source (per R4 R1 capability-system-reviewer lens, finding
//! r4-r1-cap-5):
//!
//! - `tests/cap_recheck_closure_does_not_block_change_broadcast_fan_out_under_grant_store_contention`
//!
//! ## Architectural intent (cap-r4-5 MINOR closure)
//!
//! D-PHASE-3-5 specifies "sync-per-event closure consulted at
//! delivery boundary". A naive implementation could stall the entire
//! ChangeBroadcast fan-out across all subscribers if the closure's
//! grant-store consultation blocks (e.g., redb txn lock contention).
//!
//! The fix specifies the closure consults an **in-memory snapshot**,
//! not a blocking redb read; invalidation flows through the existing
//! ChangeSubscriber path.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores AND replaces stub bodies. Per §3.6b pim-2
//! the test must drive parallel subscribers + a contended grant-store
//! write + assert observable bounded fan-out latency.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D — cap-r4-5 — cap_recheck closure non-blocking under grant-store contention"]
fn cap_recheck_closure_does_not_block_change_broadcast_fan_out_under_grant_store_contention() {
    // cap-r4-5 pin. The cap_recheck closure consults an in-memory
    // snapshot, not a blocking redb read; ChangeBroadcast fan-out
    // latency stays bounded even under contended grant-store writes.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let engine = std::sync::Arc::new(
    //       benten_engine::Engine::open(store_dir.path()).unwrap());
    //
    //   // N parallel subscribers on the same zone:
    //   const N: usize = 16;
    //   let subscribers: Vec<_> = (0..N).map(|_| {
    //       let kp = benten_id::keypair::Keypair::generate();
    //       let grant = ... .audience(kp.public_key().to_did()) ... ;
    //       engine.caps().install_proof(&grant).unwrap();
    //       engine.subscribe("/zone/posts", kp.public_key().to_did(),
    //           |_| {}).unwrap()
    //   }).collect();
    //
    //   // Spawn a contended grant-store writer that holds a redb
    //   // write txn for ~100ms:
    //   let engine_writer = engine.clone();
    //   let writer_thread = std::thread::spawn(move || {
    //       let _txn = engine_writer.caps().begin_long_running_write_txn();
    //       std::thread::sleep(std::time::Duration::from_millis(100));
    //       drop(_txn);
    //   });
    //
    //   // While writer holds txn, drive a write that fans out via SUBSCRIBE:
    //   let start = std::time::Instant::now();
    //   engine.write_node(&node_in_zone_posts).unwrap();
    //   let fan_out_latency = start.elapsed();
    //
    //   writer_thread.join().unwrap();
    //
    //   // Fan-out latency stays bounded — well under the 100ms txn hold:
    //   assert!(fan_out_latency.as_millis() < 50,
    //       "ChangeBroadcast fan-out latency must stay bounded (<50ms) under \
    //        grant-store contention per cap-r4-5; observed {}ms",
    //       fan_out_latency.as_millis());
    //
    //   // Source-cite that the closure reads from in-memory snapshot:
    //   let src = std::fs::read_to_string("crates/benten-engine/src/cap_recheck.rs").unwrap();
    //   assert!(src.contains("snapshot") || src.contains("Snapshot"),
    //       "cap_recheck.rs must read from in-memory snapshot per cap-r4-5 contract");
    //
    // OBSERVABLE consequence: even under sustained grant-store write
    // contention, ChangeBroadcast fan-out completes promptly. Defends
    // against the "one slow grant-store reader stalls all subscribers"
    // failure shape.
    unimplemented!(
        "G14-D wires cap_recheck in-memory-snapshot semantics + invalidation via ChangeSubscriber per cap-r4-5"
    );
}
