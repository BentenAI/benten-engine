//! Phase 2a R4b Wave-3c COVERAGE M2 — `benten-dev` CLI argument-validation
//! coverage.
//!
//! `tools/benten-dev/src/main.rs` ships the binary that dispatches the
//! `inspect-state <path>` subcommand (a Phase-2a deliverable per plan
//! §G11-A DEVSERVER) plus the bare-no-args / unknown-subcommand error
//! paths. The Rust integration tests at `tests/devserver_*.rs` exercise
//! the LIBRARY surface (`DevServer`, `ReloadCoordinator`,
//! `pretty_print_envelope_bytes`); none of them drive the BINARY. This
//! test closes that gap by spawning the compiled `benten-dev` binary and
//! asserting the four CLI exit-code contracts the source documents:
//!
//! - bare invocation (no subcommand) → exit 2 + usage to stderr
//! - unknown subcommand → exit 2 + diagnostic to stderr
//! - `inspect-state` without a path arg → exit 2 + sub-usage to stderr
//! - `inspect-state <missing-file>` → exit 1 (read failure, not 2)
//! - `inspect-state <valid-file>` → exit 0 with rendered output to stdout
//!
//! Why exit 2 vs. exit 1: `main.rs` matches Unix CLI convention — exit 2
//! means "usage error" (operator passed garbage); exit 1 means "the
//! command ran but the operation failed" (file present but unreadable
//! or undecodable). Mixing these confuses CI tooling that distinguishes
//! "we mis-spelled the flag" from "the flag was right but the operation
//! failed".
//!
//! `assert_cmd` is intentionally NOT pulled in — `Command::new(env!(
//! "CARGO_BIN_EXE_benten-dev"))` is the cargo-native way to drive a
//! workspace binary from a sibling integration test. Adding `assert_cmd`
//! would buy a small ergonomic sugar at the cost of a new dev-dependency
//! (and a transitive supply-chain expansion under `supply-chain.yml`).
//!
//! Wave-3c R4b fix-pass writer.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::process::Command;

fn benten_dev() -> Command {
    Command::new(env!("CARGO_BIN_EXE_benten-dev"))
}

#[test]
fn bare_invocation_without_subcommand_exits_2_with_usage() {
    let output = benten_dev().output().expect("spawn benten-dev");
    let code = output.status.code().expect("exit code");
    assert_eq!(
        code,
        2,
        "no-subcommand invocation must exit 2 (usage error). Got {code}.\n\
         stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("usage:") && stderr.contains("benten-dev"),
        "bare invocation must print a usage message to stderr; got: {stderr}"
    );
    assert!(
        stderr.contains("inspect-state"),
        "usage must enumerate the inspect-state subcommand; got: {stderr}"
    );
}

#[test]
fn unknown_subcommand_exits_2_with_diagnostic() {
    let output = benten_dev()
        .arg("not-a-real-subcommand")
        .output()
        .expect("spawn benten-dev");
    let code = output.status.code().expect("exit code");
    assert_eq!(
        code,
        2,
        "unknown-subcommand invocation must exit 2 (usage error). Got {code}.\n\
         stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown subcommand"),
        "diagnostic must name the unknown subcommand; got: {stderr}"
    );
}

#[test]
fn inspect_state_without_path_exits_2_with_subusage() {
    let output = benten_dev()
        .arg("inspect-state")
        .output()
        .expect("spawn benten-dev");
    let code = output.status.code().expect("exit code");
    assert_eq!(
        code,
        2,
        "inspect-state without a path must exit 2. Got {code}.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("inspect-state") && stderr.contains("usage"),
        "missing-path diagnostic must include sub-usage; got: {stderr}"
    );
}

#[test]
fn inspect_state_with_unreadable_path_exits_1_not_2() {
    // A path that does NOT exist — the CLI should return exit 1 (the
    // operation failed) NOT exit 2 (the operator's syntax was bad).
    // /nonexistent/<random> avoids any race with a real file.
    let output = benten_dev()
        .args(["inspect-state", "/nonexistent/benten-dev-test-missing"])
        .output()
        .expect("spawn benten-dev");
    let code = output.status.code().expect("exit code");
    assert_eq!(
        code,
        1,
        "inspect-state on a missing file must exit 1 (operation failed), \
         distinct from exit 2 (usage error). Got {code}.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("inspect-state"),
        "diagnostic must identify the failing subcommand; got: {stderr}"
    );
}

#[test]
fn inspect_state_with_valid_envelope_bytes_exits_0() {
    // Happy path: produce a valid `ExecutionStateEnvelope` via the same
    // pretty-printer surface the binary consumes, write it to a temp
    // file, run `benten-dev inspect-state <file>`, assert exit 0 +
    // non-empty stdout.
    use benten_eval::{AttributionFrame, ExecutionStateEnvelope, ExecutionStatePayload, Frame};
    let dir = tempfile::tempdir().expect("tempdir");
    let file_path = dir.path().join("envelope.dagcbor");

    // Synthesise the simplest valid payload — a single attribution frame
    // with all-zero CIDs is sufficient to exercise the pretty-printer
    // without requiring a live engine.
    let zero = benten_core::Cid::from_blake3_digest([0u8; 32]);
    let payload = ExecutionStatePayload {
        attribution_chain: vec![AttributionFrame {
            actor_cid: zero,
            handler_cid: zero,
            capability_grant_cid: zero,
        }],
        pinned_subgraph_cids: Vec::new(),
        context_binding_snapshots: Vec::new(),
        resumption_principal_cid: zero,
        frame_stack: vec![Frame::root()],
        frame_index: 0,
    };
    let envelope = ExecutionStateEnvelope::new(payload).expect("envelope construct");
    let bytes = envelope.to_dagcbor().expect("envelope encode");
    std::fs::write(&file_path, &bytes).expect("write envelope file");

    let output = benten_dev()
        .args(["inspect-state", file_path.to_str().expect("utf8 path")])
        .output()
        .expect("spawn benten-dev");
    let code = output.status.code().expect("exit code");
    assert_eq!(
        code,
        0,
        "inspect-state on a valid envelope file must exit 0. Got {code}.\n\
         stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "happy-path inspect-state must render non-empty output to stdout; \
         got: {stdout:?}"
    );
}
