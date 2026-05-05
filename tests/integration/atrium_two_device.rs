//! R3-C RED-PHASE end-to-end pin: two-device same-identity selective
//! zone sync (G14-D + G16-B + G16-D wave-6b; per r2-test-landscape
//! §3.F + plan §1 deliverable 16).
//!
//! ## Pin source
//!
//! - r2-test-landscape §3.F multi-device sync row
//!   `integration/atrium_two_device_same_identity_selective_zone_sync`.
//! - plan §1 deliverable 16 (multi-device support per FULL-ROADMAP.md
//!   amendment 2026-05-04).
//! - exit-criterion 16 (multi-device support; full peers under
//!   shared identity + heterogeneous capability envelopes).
//! - `.addl/phase-3/exploration-device-mesh.md` (D-PHASE-3 multi-device
//!   resolution).
//!
//! ## What this pins
//!
//! Two FULL PEER instances under the SAME identity (e.g., user's
//! laptop + user's phone-OS-app) sync a SHARED ZONE SUBSET
//! bidirectionally with HETEROGENEOUS capability envelopes — e.g.,
//! desktop runs SANDBOX workflows + holds full data; phone-OS-app
//! receives notifications only + holds a subset.
//!
//! Originally placed in `tests/phase_3_workspace/`; relocated to
//! `tests/integration/` at R4-FP/R3-C per R3-CPC-1.
//!
//! ## Atrium DSL shape (B-prime per Ben's D1 decision 2026-05-04)
//!
//! Both devices use `engine.atrium({config, deviceAttestation}).join()`
//! factory pattern; the handle carries device-grain state.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G14-D + G16-B + G16-D wave-6b — exit-criterion 16 multi-device"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D + G16-B + G16-D wave-6b — exit-criterion 16 — two-device same-identity selective zone sync"]
fn atrium_two_device_same_identity_selective_zone_sync() {
    // exit-criterion 16 pin. G16-B + G16-D + G14-D implementers wire
    // this end-to-end:
    //
    //   1. Spin up two engines under the SAME peer-DID (user's
    //      account identity) but DIFFERENT device-DIDs (laptop +
    //      phone).
    //   2. Both join the same Atrium under per-device capability
    //      envelopes:
    //      - laptop: full envelope (read+write all zones)
    //      - phone: notifications-only envelope (read /zone/notifications/*)
    //   3. Laptop writes to /zone/notes (NOT in phone's envelope);
    //      phone does NOT receive the write (per G14-D F6 filtering).
    //   4. Laptop writes to /zone/notifications/n1; phone DOES
    //      receive the write (in envelope).
    //   5. Phone writes to /zone/notifications/n2; laptop DOES
    //      receive the write.
    //   6. Phone CANNOT write to /zone/notes (capability denied).
    //   7. Both attribution frames carry BOTH peer-DID AND
    //      device-DID per Inv-14.
    //
    // OBSERVABLE consequence: heterogeneous envelopes correctly
    // gate per-device sync; attribution preserves device-grain;
    // defends against the failure shape where multi-device sync
    // would silently widen any device's effective cap-set.
    unimplemented!(
        "G14-D + G16-B + G16-D wire two-device same-identity selective-zone-sync end-to-end"
    );
}
