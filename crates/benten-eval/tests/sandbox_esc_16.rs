//! R3-D RED-PHASE pin for ESC-16 (fingerprint-collapse defense)
//! (G17-A1 wave 5b).
//!
//! Pin source: r2-test-landscape §2.5 G17-A1 row
//! `sandbox_esc_16_fingerprint_collapse_fires_via_committed_wat_fixture`;
//! r1-wsa-4 MAJOR + phase-3-backlog §6.1.
//!
//! ## ESC-16 closure shape
//!
//! Phase-2b deferred ESC-16 (fingerprint-collapse — guest reads a
//! wallclock-influenced internal state to fingerprint host nondeterminism)
//! pending the engine-side memory-read helper architecture. r1-wsa-4
//! pinned the architecture: the helper lives at
//! `crates/benten-eval/src/sandbox/fingerprint.rs` (NEW per G17-A1) and
//! reads guest-controlled wallclock-influenced state. The defense
//! fires at the next host-fn boundary per phase-3-backlog §6.1
//! (BEFORE the wallclock divergence becomes guest-observable).
//!
//! ## Committed `.wat` fixture
//!
//! The driver fixture is a committed `.wat` (and per G17-B, paired
//! committed `.wasm`) at
//! `crates/benten-eval/tests/fixtures/sandbox/esc_16_fingerprint_collapse.wat`.
//! The fixture builds a guest module that:
//!
//! 1. Calls `host:wallclock` twice in succession (legitimately).
//! 2. Reads the linear-memory page that the host wrote the wallclock
//!    diff into (the fingerprint vector).
//! 3. Branches on the fingerprint to leak a side-channel.
//!
//! ESC-16 fires when the engine-side `fingerprint::read_collapse_state`
//! observes the guest's read of a wallclock-correlated cell.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-A1 wave 5b authors fingerprint.rs helper + commits ESC-16 .wat fixture"]
fn sandbox_esc_16_fingerprint_collapse_fires_via_committed_wat_fixture() {
    // r1-wsa-4 pin. G17-A1 implementer wires this:
    //
    // PRECONDITION — fixture committed:
    //   crates/benten-eval/tests/fixtures/sandbox/esc_16_fingerprint_collapse.wat
    //   (paired .wasm via G17-B build.rs)
    //
    // SHAPE:
    //   use benten_eval::sandbox::{Sandbox, SandboxConfig, ManifestRef};
    //
    //   let module = load_fixture_wat_or_wasm("esc_16_fingerprint_collapse");
    //   let sandbox = Sandbox::new(/* config granting host:wallclock */);
    //   let result = sandbox.execute(module);
    //
    //   // ESC-16 fires before the guest's branch leaks the side-channel:
    //   assert!(matches!(
    //       result.unwrap_err(),
    //       benten_eval::SandboxError::EscapeAttempt {
    //           vector: benten_eval::EscVector::Esc16FingerprintCollapse,
    //           ..
    //       }
    //   ));
    //
    //   // And the engine-side memory-read helper lives where r1-wsa-4 pinned it:
    //   let helper_src = std::fs::read_to_string(
    //       "crates/benten-eval/src/sandbox/fingerprint.rs"
    //   ).unwrap();
    //   assert!(helper_src.contains("read_collapse_state")
    //         || helper_src.contains("fingerprint"),
    //       "ESC-16 engine-side memory-read helper must live at fingerprint.rs per r1-wsa-4");
    //
    // OBSERVABLE consequence: a guest module that performs the ESC-16
    // attack pattern is DENIED with a typed `Esc16FingerprintCollapse`
    // variant before the side-channel becomes observable. Defends
    // against "ESC-16 marked closed but the defense path was never
    // wired" failure shape (pim-2 LOAD-BEARING regression class).
    unimplemented!(
        "G17-A1 wires ESC-16 fingerprint-collapse defense + committed-fixture driver test"
    );
}
