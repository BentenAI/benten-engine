//! cap-r1-13 pin — RotationLog rehydrated at engine-open.
//!
//! Per R1 cap-r1-13: on engine restart, the RotationLog persistent
//! state must be loaded so that previously-rotated keys are recognized.
//! Engine-open path consults the persistent store and rebuilds the
//! in-memory RotationLog map.

#[test]
#[ignore = "RED-PHASE: G24-D wave wires RotationLog rehydration at engine-open; un-ignore at G24-D landing"]
fn rotation_log_rehydrates_persisted_rotation_events_at_engine_open() {
    // Future surface: Engine::open(path) consults the on-disk
    // RotationLog (redb backend); rebuilds in-memory state; subsequent
    // lookups against rotated peer-DIDs return the rotation event.
    //
    // FAILS-IF-NO-OP because the rehydration step must explicitly walk
    // the persistent rotation events. Without it, restarts lose the
    // rotation knowledge and would treat rotated keys as never-rotated
    // (T9b race-defense regression).
    panic!("RED-PHASE: G24-D wave must wire RotationLog rehydration at engine-open");
}
