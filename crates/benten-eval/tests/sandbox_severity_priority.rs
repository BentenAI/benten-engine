//! Phase 2b R3-B — SANDBOX D21 severity-priority unit tests (G7-A).
//!
//! D21-RESOLVED priority: MEMORY > WALLCLOCK > FUEL > OUTPUT
//!
//! When multiple axes are eligible at a single trap-callback frame,
//! the highest-priority axis is selected (matches OS-level OOM > deadline
//! > CPU > IO ordering).
//!
//! **cr-g7a-mr-1 fix-pass:** 1 of 5 tests FLIPPED to live assertion
//! (`sandbox_priority_order_documented_in_catalog` — markdown parse
//! works against G7-A's catalog narratives). Other 4 require G7-C's
//! `testing_force_simultaneous_traps` helper which lands with the
//! Store+Instance dispatch (PR #33). G7-A's `resolve_priority()` is
//! covered by unit tests in `src/primitives/sandbox.rs`.
//!
//! Pin sources: D21-RESOLVED, wsa-4 suggested fix.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_eval::sandbox::counted_sink::{OverflowPath, SinkOverflow};
use benten_eval::sandbox::{SandboxError, resolve_priority};

#[test]
fn sandbox_severity_priority_memory_wins_over_wallclock() {
    // D21 priority resolver — memory > wallclock at the same trap frame.
    let pick = resolve_priority(vec![
        SandboxError::WallclockExceeded { limit_ms: 1000 },
        SandboxError::MemoryExhausted { limit: 100 },
    ]);
    assert!(matches!(pick, Some(SandboxError::MemoryExhausted { .. })));
}

#[test]
fn sandbox_simultaneous_wallclock_and_fuel_picks_wallclock() {
    let pick = resolve_priority(vec![
        SandboxError::FuelExhausted {
            consumed: 0,
            limit: 100,
        },
        SandboxError::WallclockExceeded { limit_ms: 1000 },
    ]);
    assert!(matches!(pick, Some(SandboxError::WallclockExceeded { .. })));
}

#[test]
fn sandbox_simultaneous_fuel_and_output_picks_fuel() {
    let overflow = SinkOverflow {
        consumed: 5,
        limit: 5,
        emitter_kind: "host_fn:compute:log".to_string(),
        path: OverflowPath::PrimaryStreaming,
    };
    let pick = resolve_priority(vec![
        SandboxError::OutputOverflow(overflow),
        SandboxError::FuelExhausted {
            consumed: 0,
            limit: 100,
        },
    ]);
    assert!(matches!(pick, Some(SandboxError::FuelExhausted { .. })));
}

#[test]
fn sandbox_priority_order_documented_in_catalog() {
    // D21 doc-drift — `docs/ERROR-CATALOG.md` MUST list each
    // SANDBOX-axis entry with text describing the D21 priority
    // ordering. Drift detector: parses the catalog markdown + asserts
    // the priority-text presence per code.
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let catalog =
        std::fs::read_to_string(root.join("docs/ERROR-CATALOG.md")).expect("catalog readable");
    // Each per-axis entry must mention the priority ordering string.
    for code in [
        "E_SANDBOX_MEMORY_EXHAUSTED",
        "E_SANDBOX_WALLCLOCK_EXCEEDED",
        "E_SANDBOX_FUEL_EXHAUSTED",
        "E_INV_SANDBOX_OUTPUT",
    ] {
        // Find the section for this code and assert the priority text
        // appears in the immediate vicinity (within 30 lines).
        let lines: Vec<&str> = catalog.lines().collect();
        let header_idx = lines
            .iter()
            .position(|l| l.starts_with('#') && l.contains(code))
            .unwrap_or_else(|| panic!("catalog missing section for {code}"));
        let scan_end = (header_idx + 30).min(lines.len());
        let priority_text = "MEMORY > WALLCLOCK > FUEL > OUTPUT";
        let has_pri = lines[header_idx..scan_end]
            .iter()
            .any(|l| l.contains(priority_text));
        assert!(
            has_pri,
            "catalog section for {code} must reference D21 priority \
             order ('{priority_text}') within 30 lines of header"
        );
    }
}

#[test]
fn sandbox_axis_in_isolation_still_fires_correctly() {
    // Each axis in isolation MUST be picked when alone.
    assert!(matches!(
        resolve_priority(vec![SandboxError::MemoryExhausted { limit: 0 }]),
        Some(SandboxError::MemoryExhausted { .. })
    ));
    assert!(matches!(
        resolve_priority(vec![SandboxError::WallclockExceeded { limit_ms: 0 }]),
        Some(SandboxError::WallclockExceeded { .. })
    ));
    assert!(matches!(
        resolve_priority(vec![SandboxError::FuelExhausted {
            consumed: 0,
            limit: 0
        }]),
        Some(SandboxError::FuelExhausted { .. })
    ));
    assert!(matches!(
        resolve_priority(vec![SandboxError::OutputOverflow(SinkOverflow {
            consumed: 0,
            limit: 0,
            emitter_kind: "x".to_string(),
            path: OverflowPath::PrimaryStreaming,
        })]),
        Some(SandboxError::OutputOverflow(_))
    ));
}
