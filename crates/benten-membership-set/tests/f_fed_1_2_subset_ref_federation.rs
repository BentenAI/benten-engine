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
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! Self-contained in-file stub-shim; compiles green behind `#[ignore]`. R5
//! swaps in `benten_membership_set::federation::{KSetAcquisitionPath,
//! verify_acquisition_offline, MEMBERSHIP_RECURSION_MAX_DEPTH}` +
//! `…::member::MemberRef::SubsetRef` and un-ignores. Would-FAIL-if-no-op'd: a
//! depth-5 path that verifies, a cycle that slips through receiver-local-only
//! detection, a SubsetRef accepted at v1-beta, a selectable Model-A, or any
//! little-endian / V1 byte all break a pin.

#![allow(dead_code)]

// ── self-contained in-file stub-shim ────────────────────────────────────────

/// Stand-in for `benten_membership_set::MembershipSetId` (a content-addressed
/// id; modeled as a fixed 4-byte tag for the offline-decidability pins).
type MembershipSetId = [u8; 4];

/// Stand-in for `benten_core::Cid` (the acquisition proof).
type Cid = [u8; 4];

/// `MEMBERSHIP_RECURSION_MAX_DEPTH = 4` (Inv-20 clause-k). Accept hop_path
/// length ≤ 4; reject 5.
const MEMBERSHIP_RECURSION_MAX_DEPTH: usize = 4;

/// The federation codepoint (`0x6620`) — reserve-and-refused at v1-beta.
const MEMBERSHIP_SET_SUBSET_REF: u16 = 0x6620;

/// The FROZEN `KSetAcquisitionPath` field-set (M-9). `hop_path` carries the
/// visited-set so cycle-detect is PATH-CARRIED (offline-decidable), not
/// receiver-local.
#[derive(Clone, PartialEq, Eq, Debug)]
struct KSetAcquisitionPath {
    target_set_id: MembershipSetId,
    hop_path: Vec<MembershipSetId>, // bounded len ≤ 4; carries visited-set
    acquisition_proof_cid: Cid,
}

#[derive(Debug, PartialEq, Eq)]
enum AcquisitionError {
    RecursionDepthExceeded,
    CycleDetected,
}

impl KSetAcquisitionPath {
    /// Production-shaped OFFLINE verifier: decides depth + cycle using ONLY the
    /// carried wire fields (NO external DB / graph lookup). At R5 this is
    /// `verify_acquisition_offline(&path)`.
    fn verify_offline(&self) -> Result<(), AcquisitionError> {
        // Depth bound (Inv-20 clause-k).
        if self.hop_path.len() > MEMBERSHIP_RECURSION_MAX_DEPTH {
            return Err(AcquisitionError::RecursionDepthExceeded);
        }
        // Cycle detection using ONLY the path-carried hops + the target. A
        // repeated id anywhere in (hop_path ++ target) is a cycle.
        let mut seen = std::collections::BTreeSet::new();
        for hop in &self.hop_path {
            if !seen.insert(*hop) {
                return Err(AcquisitionError::CycleDetected);
            }
        }
        if seen.contains(&self.target_set_id) {
            return Err(AcquisitionError::CycleDetected);
        }
        Ok(())
    }

    /// Canonical V2 + big-endian serialization (M-20 — no LE/V1 vector). The
    /// layout is: format_version(u8=2) ‖ hop_count(u8) ‖ target_set_id(4) ‖
    /// each hop(4) ‖ acquisition_proof_cid(4). Multi-byte counts BE.
    fn to_wire_v2_be(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(ENVELOPE_FORMAT_VERSION_V2); // V2 from the first commit
        out.push(self.hop_path.len() as u8);
        out.extend_from_slice(&self.target_set_id);
        for hop in &self.hop_path {
            out.extend_from_slice(hop);
        }
        out.extend_from_slice(&self.acquisition_proof_cid);
        out
    }
}

/// V2 format-version byte (M-20 — the single V1→V2 bump; NO V1 survives).
const ENVELOPE_FORMAT_VERSION_V2: u8 = 0x02;

/// Stand-in for `MemberRef::SubsetRef` dispatch + the federation model toggle.
#[derive(Debug, PartialEq, Eq)]
enum FederationError {
    /// SubsetRef / `0x6620` selected at v1-beta.
    FederationReserved,
    /// Model-A selected at v1-beta.
    ModelAUnavailable,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FederationModel {
    /// DEFAULT — independent K_Set per set (Inv-20 clause-l).
    ModelB,
    /// Opt-in post-v1-beta additive; NOT selectable at v1-beta.
    ModelA,
}

/// Production-shaped admit of a `SubsetRef` member at v1-beta → typed-reject.
fn admit_subset_ref_at_v1_beta() -> Result<(), FederationError> {
    Err(FederationError::FederationReserved)
}

/// Production-shaped dispatch of the `0x6620` codepoint at v1-beta.
fn dispatch_codepoint_at_v1_beta(cp: u16) -> Result<(), FederationError> {
    if cp == MEMBERSHIP_SET_SUBSET_REF {
        return Err(FederationError::FederationReserved);
    }
    Ok(())
}

/// Production-shaped model selection at v1-beta — only Model-B is selectable.
fn select_model_at_v1_beta(m: FederationModel) -> Result<FederationModel, FederationError> {
    match m {
        FederationModel::ModelB => Ok(FederationModel::ModelB),
        FederationModel::ModelA => Err(FederationError::ModelAUnavailable),
    }
}

// ── F-FED-1 pins ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "RED-PHASE: F-FED-1 — offline verifier decides depth+cycle from KSetAcquisitionPath wire bytes ALONE (no DB); un-ignore at R5"]
fn fed1_offline_depth_and_cycle_decidable() {
    // A valid depth-3 acyclic path verifies offline (no external state).
    let ok = KSetAcquisitionPath {
        target_set_id: [0xDD, 0, 0, 0],
        hop_path: vec![[0xAA, 0, 0, 0], [0xBB, 0, 0, 0], [0xCC, 0, 0, 0]],
        acquisition_proof_cid: [0x01, 0x02, 0x03, 0x04],
    };
    assert!(
        ok.verify_offline().is_ok(),
        "a depth-3 acyclic acquisition path verifies offline from wire bytes alone"
    );
}

#[test]
#[ignore = "RED-PHASE: F-FED-1 — depth-5 path → RecursionDepthExceeded (MAX_DEPTH=4); un-ignore at R5"]
fn fed1_depth_5_rejected() {
    assert_eq!(MEMBERSHIP_RECURSION_MAX_DEPTH, 4);
    // 4 hops accepts (at the ceiling).
    let at_ceiling = KSetAcquisitionPath {
        target_set_id: [0xEE, 0, 0, 0],
        hop_path: vec![
            [0x01, 0, 0, 0],
            [0x02, 0, 0, 0],
            [0x03, 0, 0, 0],
            [0x04, 0, 0, 0],
        ],
        acquisition_proof_cid: [0; 4],
    };
    assert!(
        at_ceiling.verify_offline().is_ok(),
        "depth-4 is at the ceiling and accepts"
    );
    // 5 hops rejects.
    let too_deep = KSetAcquisitionPath {
        target_set_id: [0xEE, 0, 0, 0],
        hop_path: vec![
            [0x01, 0, 0, 0],
            [0x02, 0, 0, 0],
            [0x03, 0, 0, 0],
            [0x04, 0, 0, 0],
            [0x05, 0, 0, 0],
        ],
        acquisition_proof_cid: [0; 4],
    };
    assert_eq!(
        too_deep.verify_offline(),
        Err(AcquisitionError::RecursionDepthExceeded),
        "depth-5 exceeds MEMBERSHIP_RECURSION_MAX_DEPTH=4"
    );
}

#[test]
#[ignore = "RED-PHASE: F-FED-1 — cycle [A,B,C,A] rejected using ONLY the path-carried hops; un-ignore at R5"]
fn fed1_cycle_detected_from_carried_path() {
    // The target re-appears in the hop_path → cycle, detected with NO external
    // graph state (path-carried, not receiver-local).
    let cyclic = KSetAcquisitionPath {
        target_set_id: [0xAA, 0, 0, 0], // == hop_path[0] → cycle A..A
        hop_path: vec![[0xAA, 0, 0, 0], [0xBB, 0, 0, 0], [0xCC, 0, 0, 0]],
        acquisition_proof_cid: [0; 4],
    };
    assert_eq!(
        cyclic.verify_offline(),
        Err(AcquisitionError::CycleDetected),
        "a cycle [A,B,C,A] is rejected using ONLY the carried path (offline-decidable)"
    );
}

#[test]
#[ignore = "RED-PHASE: F-FED-1 — intra-path cycle [A,B,A] rejected by the recursion-bound (repeat WITHIN hop_path, distinct target); un-ignore at R5"]
fn fed1_intra_path_cycle_detected() {
    // F4-044: a cycle that lives ENTIRELY inside the hop_path (a hop repeats
    // before reaching the target) — distinct from the target-re-appears case
    // in `fed1_cycle_detected_from_carried_path`. This exercises the
    // `seen.insert` dedup branch (NOT the `seen.contains(&target)` branch).
    // The target [0xDD..] is NOT in the path, so a verifier that ONLY checked
    // "target re-appears" would MISS this and accept the cycle.
    let intra = KSetAcquisitionPath {
        target_set_id: [0xDD, 0, 0, 0], // distinct from every hop
        hop_path: vec![[0xAA, 0, 0, 0], [0xBB, 0, 0, 0], [0xAA, 0, 0, 0]], // A,B,A
        acquisition_proof_cid: [0; 4],
    };
    assert_eq!(
        intra.verify_offline(),
        Err(AcquisitionError::CycleDetected),
        "an intra-path cycle [A,B,A] is rejected from the carried path alone (a repeated hop, target absent)"
    );
    // Paired positive control: the SAME shape with a non-repeating middle hop
    // (A,B,C) and the same distinct target verifies — so the rejection is the
    // repeat, not a blanket fail on 3-hop paths.
    let acyclic = KSetAcquisitionPath {
        target_set_id: [0xDD, 0, 0, 0],
        hop_path: vec![[0xAA, 0, 0, 0], [0xBB, 0, 0, 0], [0xCC, 0, 0, 0]],
        acquisition_proof_cid: [0; 4],
    };
    assert!(
        acyclic.verify_offline().is_ok(),
        "an acyclic 3-hop path (A,B,C) with a distinct target verifies — the intra-path rejection is targeted"
    );
}

#[test]
#[ignore = "RED-PHASE: F-FED-1 — KSetAcquisitionPath wire is V2 + big-endian from the first commit (M-20); un-ignore at R5"]
fn fed1_wire_is_v2_big_endian() {
    // Hex-pin the frozen field-set serialization. V2 from the first commit;
    // no LE / V1 vector survives. Would-FAIL the moment any field went LE or
    // the version byte regressed to V1.
    let path = KSetAcquisitionPath {
        target_set_id: [0xDD, 0xCC, 0xBB, 0xAA],
        hop_path: vec![[0x11, 0x22, 0x33, 0x44], [0x55, 0x66, 0x77, 0x88]],
        acquisition_proof_cid: [0xDE, 0xAD, 0xBE, 0xEF],
    };
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
#[ignore = "RED-PHASE: F-FED-2 — MemberRef::SubsetRef refused at v1-beta (typed-reject at 0x6620); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-FED-2 — Model-B is the default; Model-A NOT selectable at v1-beta; un-ignore at R5"]
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
