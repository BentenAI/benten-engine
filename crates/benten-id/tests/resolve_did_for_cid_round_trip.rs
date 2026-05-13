//! cap-r1-16 pin — resolve_did_for_cid round-trip.

#[test]
#[ignore = "DESTINATION-REMAPPED at R6-FP-BF per HARD RULE rule-12 clause-(b) BELONGS-NAMED-NOW. `benten_id::resolve_did_for_cid` round-trip surface was never minted at G24-D (G24-D shipped plugin_did::mint + cap-r1-16 was triaged into G24-F's DidKeyedSession::resolve; the standalone seam `resolve_did_for_cid` is a separate Phase-4-Meta concern coupled to RotationLog rehydration). Named destination: docs/future/phase-4-backlog.md §4.26 (Phase-4-Meta RotationLog rehydration at engine open + resolve_did_for_cid round-trip)."]
fn resolve_did_for_cid_returns_owning_device_did_round_trip() {
    // Substantive surface lands at §4.26. Body deferred.
}
