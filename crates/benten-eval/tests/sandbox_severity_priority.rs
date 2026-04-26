//! Phase 2b R3-B — SANDBOX D21 severity-priority unit tests (G7-A).
//!
//! D21-RESOLVED priority: MEMORY > WALLCLOCK > FUEL > OUTPUT
//!
//! When multiple axes are eligible at a single trap-callback frame,
//! the highest-priority axis is selected (matches OS-level OOM > deadline
//! > CPU > IO ordering).
//!
//! Pin sources: D21-RESOLVED, wsa-4 suggested fix.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-A pending — D21 priority MEMORY > WALLCLOCK"]
fn sandbox_severity_priority_memory_wins_over_wallclock() {
    // wsa-4 suggested fix — fixture trips memory + wallclock in the same
    // trap frame. Asserts: `E_SANDBOX_MEMORY_EXHAUSTED` fires; NOT
    // `E_SANDBOX_WALLCLOCK_EXCEEDED`.
    //
    // Use `testing_force_simultaneous_traps(engine, &[Memory, Wallclock])`
    // helper (R2 §9 anticipated helper, G7-A scope).
    todo!("R5 G7-A — testing helper + dual-trip assertion");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D21 next-priority pair"]
fn sandbox_simultaneous_wallclock_and_fuel_picks_wallclock() {
    // D21 — next-priority pair. Fuel exhaustion + wallclock deadline in
    // same frame: WALLCLOCK fires; NOT FUEL.
    todo!("R5 G7-A — testing_force_simultaneous_traps(&[Wallclock, Fuel])");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D21 lowest-priority pair"]
fn sandbox_simultaneous_fuel_and_output_picks_fuel() {
    // D21 — lowest-priority pair. Fuel exhaustion + output overflow in
    // same frame: FUEL fires; NOT OUTPUT.
    todo!("R5 G7-A — testing_force_simultaneous_traps(&[Fuel, Output])");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D21 doc-drift detector"]
fn sandbox_priority_order_documented_in_catalog() {
    // D21 doc-drift — `docs/ERROR-CATALOG.md` MUST list each
    // E_SANDBOX_*_EXHAUSTED / E_SANDBOX_WALLCLOCK_EXCEEDED /
    // E_INV_SANDBOX_OUTPUT entry with text:
    //   "fires before [other axes] when multiple are simultaneously
    //    eligible (D21 priority MEMORY > WALLCLOCK > FUEL > OUTPUT)"
    //
    // Drift detector: parses the catalog markdown + asserts the
    // priority-text presence per code.
    //
    // Owner overlap: `docs/SANDBOX-LIMITS.md` (G7-C) is the canonical
    // source; ERROR-CATALOG cross-references.
    todo!("R5 G7-A — markdown parse + per-code priority-text assertion");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D21 single-axis regression guard"]
fn sandbox_axis_in_isolation_still_fires_correctly() {
    // D21 regression guard — priority ordering MUST NOT mask single-axis
    // trips. 4 sub-cases (one per axis) using `testing_force_simultaneous_traps`
    // with a SINGLETON axis input.
    //
    // - Memory-only → E_SANDBOX_MEMORY_EXHAUSTED.
    // - Wallclock-only → E_SANDBOX_WALLCLOCK_EXCEEDED.
    // - Fuel-only → E_SANDBOX_FUEL_EXHAUSTED.
    // - Output-only → E_INV_SANDBOX_OUTPUT.
    todo!("R5 G7-A — 4 sub-cases via testing_force_simultaneous_traps singleton");
}
