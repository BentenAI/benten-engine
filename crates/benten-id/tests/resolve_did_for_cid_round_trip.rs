//! cap-r1-16 pin — resolve_did_for_cid round-trip.
//!
//! Per plan §3 G24-F + cap-r1-16: thin-client session needs to
//! resolve the device-DID associated with a CID claim during
//! handshake. Round-trip: given a CID, resolve to the device-DID
//! that holds the canonical bytes.

#[test]
#[ignore = "RED-PHASE: G24-D wave wires resolve_did_for_cid; un-ignore at G24-D landing"]
fn resolve_did_for_cid_returns_owning_device_did_round_trip() {
    // Future surface:
    //   benten_id::resolve_did_for_cid(cid: &Cid, registry:
    //     &DidRegistry) -> Result<Did, IdError>
    // round-trips through the device-DID attestation envelope V2
    // (Phase-3 G16-D wave-6b shipped). Cap-r1-16 closure verifies the
    // resolution path is reachable from the cap-policy layer.
    //
    // FAILS-IF-NO-OP because thin-client cap-policy needs this seam
    // to attribute reads against the correct device-DID principal.
    panic!("RED-PHASE: G24-D wave must wire resolve_did_for_cid round-trip");
}
