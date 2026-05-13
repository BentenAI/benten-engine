//! cap-r1-13 pin — RotationLog rehydrated at engine-open.

#[test]
#[ignore = "DESTINATION-REMAPPED at R6-FP-BF per HARD RULE rule-12 clause-(b) BELONGS-NAMED-NOW. Engine-open rehydration of persistent RotationLog state was NOT yet wired at G24-D (G24-D-FP-2 shipped HLC-monotonic-strict integration into RotationLog but the engine-open rehydration seam couples to §4.20 engine-builder seam). Named destination: docs/future/phase-4-backlog.md §4.26 (Phase-4-Meta RotationLog rehydration at engine open + resolve_did_for_cid round-trip)."]
fn rotation_log_rehydrates_persisted_rotation_events_at_engine_open() {
    // Substantive surface lands at §4.26. Body deferred.
}
