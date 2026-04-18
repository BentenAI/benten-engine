//! Placeholder for the M16-deferred `prop_hlc_monotonic` proptest.
//!
//! R4 triage M16 deferred three proptests to R5. Two (capability-check
//! determinism, transform-grammar-accepted determinism) landed during R5 /
//! R4b close. The third — HLC monotonicity — depends on HLC-state-machine
//! infrastructure that is not present in Phase 1 (HLC is a Phase 3 CRDT
//! prerequisite per CLAUDE.md §Tech Stack). The placeholder below pins the
//! deferral visibly in the test tree so a future Phase-3 agent lands a real
//! proptest in the same spot rather than hunting through triage documents.
//!
//! Cross-refs:
//! - `.addl/phase-1/r4-triage.md` M16
//! - `.addl/phase-1/r4b-rust-test-coverage.json` r4b-rtc-1 (b)

#[test]
#[ignore = "TODO(phase-3-hlc): prop_hlc_monotonic requires HLC state machine \
  (uhlc crate) which lands with Phase-3 CRDT plumbing. When populated, \
  assert monotonicity across threads and processes."]
fn prop_hlc_monotonic_placeholder() {
    // Deliberately empty — Phase 3 replaces this with the real proptest.
}
