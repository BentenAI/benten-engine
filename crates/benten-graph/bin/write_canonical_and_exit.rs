//! Test-fixture binary for `d2_cross_process_graph.rs`.
//!
//! Opens a `RedbBackend` at `argv\[1\]`, writes the canonical test node, exits 0.
//! The integration test invokes this binary with `CARGO_BIN_EXE_write-canonical-and-exit`
//! to prove PID separation in the cross-process round-trip (R4 triage M3).

#![allow(
    clippy::print_stdout,
    reason = "test-fixture binary communicates CID to parent process via stdout"
)]

use std::io::Write;

use benten_core::testing::canonical_test_node;
use benten_graph::RedbBackend;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: write-canonical-and-exit <db-path>");
    let backend = RedbBackend::open(&path).expect("open redb");
    let cid = backend.put_node(&canonical_test_node()).expect("put_node");
    // Print CID on stdout so the parent can assert without opening the db
    // itself (belt + suspenders check for PID separation).
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    writeln!(lock, "{}", cid.to_base32()).expect("write stdout");
}
