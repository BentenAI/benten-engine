//! R3-E RED-PHASE pins for G20-A2 §7.3.A.6 WAIT TTL runtime expiry path
//! GC machinery (wave-8a; MEDIUM-risk per scope-real-03).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.8 G20-A2 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G20-A2 must-pass column):
//!
//! - `tests/wait_ttl_runtime_expiry_path_gc_machinery_correct` — §7.3.A.6
//! - `tests/stream_subscribe_end_to_end_no_residual_ignore` — §7.3.A.2
//!
//! ## What G20-A2 establishes (§7.3.A.6 + §7.3.A.2)
//!
//! Per scope-real-03: the original §7.3.A.6 was MISCATEGORIZED as test
//! source — actually carries ~200-400 LOC of GC machinery PRODUCTION
//! code (cross-process correctness MEDIUM-risk; ~400-600 LOC test source).
//! G20-A2 wave-8a un-ignores the test bodies AND lands the missing GC
//! machinery production code.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G20-A2 wave-8a — WAIT TTL runtime expiry GC machinery correct (§7.3.A.6)"]
fn wait_ttl_runtime_expiry_path_gc_machinery_correct() {
    // §7.3.A.6 LOAD-BEARING pin. G20-A2 implementer wires this:
    //
    //   let engine = benten_engine::Engine::open_in_memory().unwrap();
    //   let sg = engine.register_subgraph_with_ttl_wait("ttl-test", 100).unwrap();
    //
    //   // Spawn N parallel suspended invocations; let TTLs expire:
    //   let envelopes: Vec<_> = (0..50).map(|i| {
    //       let s = engine.call_with_suspension(sg, "main", json!({"i": i})).unwrap();
    //       s.envelope
    //   }).collect();
    //
    //   // Advance past TTL boundary:
    //   engine.testing_advance_wait_clock(150).unwrap();
    //
    //   // Trigger GC pass — expired suspensions are reaped:
    //   engine.testing_run_wait_ttl_gc_pass().unwrap();
    //
    //   // OBSERVABLE consequence: suspension-store has been pruned of
    //   // expired entries; resume attempts fire E_WAIT_TIMEOUT (not
    //   // E_INVALID_RESUME because the entry was reaped):
    //   for env in envelopes {
    //       let result = engine.resume_with_meta(env, "go");
    //       assert!(result.is_err());
    //       assert_eq!(result.err().unwrap().error_code(),
    //           benten_errors::ErrorCode::WaitTimeout,
    //           "post-TTL+GC resume must fire E_WAIT_TIMEOUT");
    //   }
    //
    //   // GC stats are observable:
    //   let stats = engine.testing_wait_ttl_gc_stats();
    //   assert!(stats.reaped_count >= 50,
    //       "GC must have reaped all 50 expired suspensions");
    //
    // OBSERVABLE consequence: GC machinery reaps expired suspensions
    // correctly. Defends against the silent-leak failure mode where
    // expired suspensions accumulate in the suspension store unbounded.
    unimplemented!("G20-A2 wires WAIT TTL GC machinery correctness pin");
}

#[test]
#[ignore = "RED-PHASE: G20-A2 wave-8a — §7.3.A.2 STREAM/SUBSCRIBE no residual #[ignore]"]
fn stream_subscribe_end_to_end_no_residual_ignore() {
    // §7.3.A.2 closure pin. G20-A2 implementer un-ignores all
    // STREAM/SUBSCRIBE end-to-end test bodies; this regression gate
    // verifies no Phase-3-destination `#[ignore]` rationales remain.
    //
    //   // Walk crates/benten-eval/tests/stream_*.rs + subscribe_*.rs:
    //   let test_dirs = [
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("benten-eval").join("tests"),
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests"),
    //   ];
    //   let mut residuals = Vec::new();
    //   for dir in &test_dirs {
    //       for entry in std::fs::read_dir(dir).unwrap() {
    //           let path = entry.unwrap().path();
    //           if !path.extension().map_or(false, |e| e == "rs") { continue; }
    //           let name = path.file_name().unwrap().to_string_lossy().to_string();
    //           if !(name.starts_with("stream_") || name.starts_with("subscribe_")) {
    //               continue;
    //           }
    //           let src = std::fs::read_to_string(&path).unwrap();
    //           for line in src.lines() {
    //               if line.contains("#[ignore") && line.contains("Phase 3") {
    //                   residuals.push(format!("{}: {}", name, line.trim()));
    //               }
    //           }
    //       }
    //   }
    //   assert!(residuals.is_empty(),
    //       "G20-A2 incomplete: residual Phase-3-destination ignores in \
    //        stream/subscribe tests:\n{}", residuals.join("\n"));
    //
    // OBSERVABLE consequence: the §7.3.A.2 residual cluster fully
    // clears. End-to-end pin per §3.6b.
    unimplemented!("G20-A2 wires no-Phase-3-residual-ignore audit for stream/subscribe tests");
}
