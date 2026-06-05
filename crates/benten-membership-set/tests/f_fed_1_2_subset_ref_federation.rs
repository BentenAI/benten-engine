//! **F-FED-1** + **F-FED-2** — Federation: `KSetAcquisitionPath` frozen
//! field-set + offline-decidability, and `SubsetRef` refused-at-v1-beta +
//! Model-B default.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core F-full Wave
//! R3-W4, families **F-FED-1** (merges J1/J2/J3 + GNI-4, NQ-D3) and
//! **F-FED-2** (merges J4 + GNI-4).
//!
//! # What F-FED-1 pins (R0 §3.6.C / M-9 / NQ-D3 / Inv-20 clause-k / §4.2
//! `0x6620`)
//!
//! The FROZEN `KSetAcquisitionPath { target_set_id, hop_path:
//! Vec<MembershipSetId> (≤4), acquisition_proof_cid }` makes depth-4 +
//! cycle-detect decidable **offline from the wire bytes ALONE** (the visited
//! path is PATH-CARRIED, NOT receiver-local). `MEMBERSHIP_RECURSION_MAX_DEPTH
//! = 4` (accept 4, reject 5). A cycle `[A, B, C, A]` is rejected using ONLY
//! the carried path. The byte layout is **V2 + big-endian** from the first
//! commit (M-20 — no LE/V1 golden vector survives).
//!
//! # What F-FED-2 pins (R0 §3.6.C / Inv-20 clause-k/l / §4.2)
//!
//! `MemberRef::SubsetRef` is **reserved-and-REFUSED** at v1-beta (typed-reject
//! at codepoint `0x6620`). The default federation model is **Model-B**
//! (independent-`K_Set`-per-set); Model-A is opt-in post-v1-beta additive (NOT
//! selectable at v1-beta).
//!
//! # R5 (w-ms-sync): wired to the REAL `benten_membership_set::federation`
//!
//! The self-contained stub-shim is DELETED. The production surface is
//! `benten_membership_set::federation::{KSetAcquisitionPath,
//! MEMBERSHIP_RECURSION_MAX_DEPTH, AcquisitionError, FederationError,
//! FederationModel, admit_subset_ref_at_v1_beta, dispatch_codepoint_at_v1_beta,
//! select_model_at_v1_beta}` + the `0x6620` codepoint
//! `benten_membership_set::codepoints::MEMBERSHIP_SET_RESERVED_0X6620`. The
//! production `KSetAcquisitionPath` carries `Vec<u8>` ids (content-addressed
//! bytes); the `[u8; 4]` fixtures below are lifted to `Vec<u8>` by the `mk`
//! helper. The wire layout (V2 + big-endian) is byte-identical to the frozen
//! golden vector (the `to_wire_v2_be` serialization is unchanged).

#![allow(dead_code)]

use benten_membership_set::codepoints::MEMBERSHIP_SET_RESERVED_0X6620;
use benten_membership_set::federation::{
    AcquisitionError, FederationError, FederationModel, KSetAcquisitionPath,
    MEMBERSHIP_RECURSION_MAX_DEPTH, admit_subset_ref_at_v1_beta, dispatch_codepoint_at_v1_beta,
    select_model_at_v1_beta,
};

/// The federation codepoint (`0x6620`) — reserve-and-refused at v1-beta.
const MEMBERSHIP_SET_SUBSET_REF: u16 = MEMBERSHIP_SET_RESERVED_0X6620;

/// Lift a `[u8; 4]` fixture id into the production `Vec<u8>` id type.
fn id(tag: [u8; 4]) -> Vec<u8> {
    tag.to_vec()
}

/// Build a production `KSetAcquisitionPath` from `[u8; 4]` fixtures.
fn mk(target: [u8; 4], hops: &[[u8; 4]], cid: [u8; 4]) -> KSetAcquisitionPath {
    KSetAcquisitionPath {
        target_set_id: id(target),
        hop_path: hops.iter().map(|h| id(*h)).collect(),
        acquisition_proof_cid: id(cid),
    }
}

// ── F-FED-1 pins ──────────────────────────────────────────────────────────────

#[test]
fn fed1_offline_depth_and_cycle_decidable() {
    // A valid depth-3 acyclic path verifies offline (no external state).
    let ok = mk(
        [0xDD, 0, 0, 0],
        &[[0xAA, 0, 0, 0], [0xBB, 0, 0, 0], [0xCC, 0, 0, 0]],
        [0x01, 0x02, 0x03, 0x04],
    );
    assert!(
        ok.verify_offline().is_ok(),
        "a depth-3 acyclic acquisition path verifies offline from wire bytes alone"
    );
}

#[test]
fn fed1_depth_5_rejected() {
    assert_eq!(MEMBERSHIP_RECURSION_MAX_DEPTH, 4);
    // 4 hops accepts (at the ceiling).
    let at_ceiling = mk(
        [0xEE, 0, 0, 0],
        &[
            [0x01, 0, 0, 0],
            [0x02, 0, 0, 0],
            [0x03, 0, 0, 0],
            [0x04, 0, 0, 0],
        ],
        [0; 4],
    );
    assert!(
        at_ceiling.verify_offline().is_ok(),
        "depth-4 is at the ceiling and accepts"
    );
    // 5 hops rejects.
    let too_deep = mk(
        [0xEE, 0, 0, 0],
        &[
            [0x01, 0, 0, 0],
            [0x02, 0, 0, 0],
            [0x03, 0, 0, 0],
            [0x04, 0, 0, 0],
            [0x05, 0, 0, 0],
        ],
        [0; 4],
    );
    assert_eq!(
        too_deep.verify_offline(),
        Err(AcquisitionError::RecursionDepthExceeded),
        "depth-5 exceeds MEMBERSHIP_RECURSION_MAX_DEPTH=4"
    );
}

#[test]
fn fed1_cycle_detected_from_carried_path() {
    // The target re-appears in the hop_path → cycle, detected with NO external
    // graph state (path-carried, not receiver-local).
    let cyclic = mk(
        [0xAA, 0, 0, 0], // == hop_path[0] → cycle A..A
        &[[0xAA, 0, 0, 0], [0xBB, 0, 0, 0], [0xCC, 0, 0, 0]],
        [0; 4],
    );
    assert_eq!(
        cyclic.verify_offline(),
        Err(AcquisitionError::CycleDetected),
        "a cycle [A,B,C,A] is rejected using ONLY the carried path (offline-decidable)"
    );
}

#[test]
fn fed1_intra_path_cycle_detected() {
    // F4-044: a cycle that lives ENTIRELY inside the hop_path (a hop repeats
    // before reaching the target) — distinct from the target-re-appears case.
    // The target [0xDD..] is NOT in the path, so a verifier that ONLY checked
    // "target re-appears" would MISS this and accept the cycle.
    let intra = mk(
        [0xDD, 0, 0, 0],                                      // distinct from every hop
        &[[0xAA, 0, 0, 0], [0xBB, 0, 0, 0], [0xAA, 0, 0, 0]], // A,B,A
        [0; 4],
    );
    assert_eq!(
        intra.verify_offline(),
        Err(AcquisitionError::CycleDetected),
        "an intra-path cycle [A,B,A] is rejected from the carried path alone (a repeated hop, target absent)"
    );
    // Paired positive control: the SAME shape with a non-repeating middle hop
    // (A,B,C) and the same distinct target verifies — so the rejection is the
    // repeat, not a blanket fail on 3-hop paths.
    let acyclic = mk(
        [0xDD, 0, 0, 0],
        &[[0xAA, 0, 0, 0], [0xBB, 0, 0, 0], [0xCC, 0, 0, 0]],
        [0; 4],
    );
    assert!(
        acyclic.verify_offline().is_ok(),
        "an acyclic 3-hop path (A,B,C) with a distinct target verifies — the intra-path rejection is targeted"
    );
}

#[test]
fn fed1_wire_is_v2_big_endian() {
    // Hex-pin the frozen field-set serialization. V2 from the first commit;
    // no LE / V1 vector survives. Would-FAIL the moment any field went LE or
    // the version byte regressed to V1.
    let path = mk(
        [0xDD, 0xCC, 0xBB, 0xAA],
        &[[0x11, 0x22, 0x33, 0x44], [0x55, 0x66, 0x77, 0x88]],
        [0xDE, 0xAD, 0xBE, 0xEF],
    );
    let wire = path.to_wire_v2_be();
    let expected: Vec<u8> = vec![
        0x02, // ENVELOPE_FORMAT_VERSION_V2 (M-20; NOT 0x01)
        0x02, // hop_count = 2
        0xDD, 0xCC, 0xBB, 0xAA, // target_set_id (as-authored byte order)
        0x11, 0x22, 0x33, 0x44, // hop[0]
        0x55, 0x66, 0x77, 0x88, // hop[1]
        0xDE, 0xAD, 0xBE, 0xEF, // acquisition_proof_cid
    ];
    assert_eq!(
        wire, expected,
        "KSetAcquisitionPath serializes V2 + big-endian from the first commit (M-20 — no LE/V1 golden vector)"
    );
    // Explicit version-byte assertion (the M-20 freeze-gating guarantee).
    assert_eq!(wire[0], 0x02, "format_version is V2");
}

// ── F-FED-2 pins ──────────────────────────────────────────────────────────────

#[test]
fn fed2_subset_ref_refused_at_v1_beta() {
    // Admitting a SubsetRef member at v1-beta typed-rejects.
    assert_eq!(
        admit_subset_ref_at_v1_beta(),
        Err(FederationError::FederationReserved),
        "MemberRef::SubsetRef is reserved-and-REFUSED at v1-beta"
    );
    // The 0x6620 codepoint typed-rejects at dispatch.
    assert_eq!(
        dispatch_codepoint_at_v1_beta(MEMBERSHIP_SET_SUBSET_REF),
        Err(FederationError::FederationReserved),
        "codepoint 0x6620 typed-rejects at v1-beta"
    );
    assert_eq!(MEMBERSHIP_SET_SUBSET_REF, 0x6620);
    // Paired positive control: a non-federation codepoint does NOT reject — so
    // the 0x6620 rejection is targeted, not a blanket fail.
    assert!(dispatch_codepoint_at_v1_beta(0x6600).is_ok());
}

#[test]
fn fed2_model_b_default_model_a_unavailable() {
    // Model-B (independent K_Set per set) is the selectable default.
    assert_eq!(
        select_model_at_v1_beta(FederationModel::ModelB),
        Ok(FederationModel::ModelB),
        "Model-B (independent-K_Set-per-set) is the v1-beta default"
    );
    // Model-A is opt-in post-v1-beta additive — NOT selectable now.
    assert_eq!(
        select_model_at_v1_beta(FederationModel::ModelA),
        Err(FederationError::ModelAUnavailable),
        "Model-A is post-v1-beta additive; NOT selectable at v1-beta (Inv-20 clause-l)"
    );
}
