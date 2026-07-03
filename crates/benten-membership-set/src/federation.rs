//! MembershipSet federation: the `KSetAcquisitionPath` frozen field-set +
//! offline depth/cycle decidability, and the `SubsetRef`-refused-at-v1-beta +
//! Model-B-default dispatch (F-FED-1 / F-FED-2).
//!
//! # `KSetAcquisitionPath` (M-9 / NQ-D3 / Inv-20 clause-k / §4.2 `0x6620`)
//!
//! The FROZEN [`KSetAcquisitionPath`] makes depth-4 + cycle-detect decidable
//! **offline from the wire bytes ALONE** — the visited path is PATH-CARRIED, NOT
//! receiver-local. [`MEMBERSHIP_RECURSION_MAX_DEPTH`] = 4 (accept 4, reject 5).
//! The wire layout is **V2 + big-endian from the first commit** (M-20 — no
//! LE/V1 golden vector survives; [`KSetAcquisitionPath::to_wire_v2_be`]).
//!
//! # `SubsetRef` refused at v1-beta (Inv-20 clause-k/l / §4.2)
//!
//! [`MemberRef::SubsetRef`](crate::member::MemberRef) is reserved-and-REFUSED at
//! v1-beta — a typed-reject at codepoint `0x6620`
//! ([`crate::codepoints::MEMBERSHIP_SET_RESERVED_0X6620`]). The default
//! federation model is **Model-B** (independent-`K_Set`-per-set); Model-A is
//! opt-in post-v1-beta additive (NOT selectable at v1-beta).

use crate::codepoints::MEMBERSHIP_SET_RESERVED_0X6620;

/// `MEMBERSHIP_RECURSION_MAX_DEPTH = 4` (Inv-20 clause-k). Accept a `hop_path`
/// of length ≤ 4; reject 5.
pub const MEMBERSHIP_RECURSION_MAX_DEPTH: usize = 4;

/// The V2 envelope format-version byte (M-20 — the single V1→V2 bump; NO V1
/// vector survives).
pub const ENVELOPE_FORMAT_VERSION_V2: u8 = 0x02;

/// A content-addressed MembershipSet id (modeled here as opaque bytes; the
/// offline-decidability pins treat it as an equality-comparable tag).
pub type FederationSetId = Vec<u8>;

/// Errors from the offline acquisition-path verifier.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum AcquisitionError {
    /// The `hop_path` exceeds [`MEMBERSHIP_RECURSION_MAX_DEPTH`] (depth-bound;
    /// Inv-20 clause-k).
    #[error("acquisition recursion depth exceeded (MEMBERSHIP_RECURSION_MAX_DEPTH = 4)")]
    RecursionDepthExceeded,
    /// A cycle was detected from the PATH-CARRIED hops alone (a repeated id in
    /// `hop_path`, or the `target_set_id` reappearing among the hops).
    #[error("acquisition cycle detected from the path-carried hops")]
    CycleDetected,
}

impl AcquisitionError {
    /// The stable [`benten_errors::ErrorCode`] for this offline-verifier
    /// rejection. Following the [`crate::error::MembershipSetError`] precedent
    /// (only `RoleStaleAtVerify` carries a first-class §3.5g catalog mint; the
    /// construction/decode-time rejections route through existing catalog
    /// codes), the depth-bound maps to `ValueOutOfRange` and the cycle to
    /// `CapDenied` — NO new catalog variant is minted (these are offline-decode
    /// rejections of a RESERVED-and-REFUSED federation path, not a positive wire
    /// path at v1-beta).
    #[must_use]
    pub fn error_code(&self) -> benten_errors::ErrorCode {
        match self {
            AcquisitionError::RecursionDepthExceeded => benten_errors::ErrorCode::ValueOutOfRange,
            AcquisitionError::CycleDetected => benten_errors::ErrorCode::CapDenied,
        }
    }
}

/// The FROZEN `KSetAcquisitionPath` field-set (M-9). `hop_path` carries the
/// visited-set so cycle-detect is PATH-CARRIED (offline-decidable), not
/// receiver-local.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KSetAcquisitionPath {
    /// The set being acquired.
    pub target_set_id: FederationSetId,
    /// The bounded (≤ 4) visited-hop path. Carries the visited-set so a verifier
    /// decides depth + cycles from the wire bytes ALONE.
    pub hop_path: Vec<FederationSetId>,
    /// The acquisition proof CID (content-addressed).
    pub acquisition_proof_cid: Vec<u8>,
}

impl KSetAcquisitionPath {
    /// OFFLINE verifier: decides depth + cycle using ONLY the carried wire
    /// fields (NO external DB / graph lookup).
    ///
    /// # Errors
    ///
    /// - [`AcquisitionError::RecursionDepthExceeded`] if `hop_path.len() > 4`.
    /// - [`AcquisitionError::CycleDetected`] if any hop repeats, or the target
    ///   reappears among the hops.
    pub fn verify_offline(&self) -> Result<(), AcquisitionError> {
        // Depth bound (Inv-20 clause-k).
        if self.hop_path.len() > MEMBERSHIP_RECURSION_MAX_DEPTH {
            return Err(AcquisitionError::RecursionDepthExceeded);
        }
        // Cycle detection using ONLY the path-carried hops + the target. A
        // repeated id anywhere in (hop_path ++ target) is a cycle.
        let mut seen = std::collections::BTreeSet::new();
        for hop in &self.hop_path {
            if !seen.insert(hop.clone()) {
                return Err(AcquisitionError::CycleDetected);
            }
        }
        if seen.contains(&self.target_set_id) {
            return Err(AcquisitionError::CycleDetected);
        }
        Ok(())
    }

    /// Canonical V2 + big-endian serialization (M-20 — no LE/V1 vector). Layout:
    /// `format_version(u8=2) || hop_count(u8) || target_set_id || each hop ||
    /// acquisition_proof_cid`. The set-id / CID byte sequences are emitted
    /// as-authored (content-addressed bytes); the counts are V2 + BE.
    #[must_use]
    pub fn to_wire_v2_be(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(ENVELOPE_FORMAT_VERSION_V2); // V2 from the first commit
        out.push(u8::try_from(self.hop_path.len()).expect("hop_path ≤ 4 fits u8"));
        out.extend_from_slice(&self.target_set_id);
        for hop in &self.hop_path {
            out.extend_from_slice(hop);
        }
        out.extend_from_slice(&self.acquisition_proof_cid);
        out
    }
}

/// Federation-dispatch errors (the `SubsetRef`/`0x6620` typed-reject + the
/// Model-A-unavailable rejection).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum FederationError {
    /// `SubsetRef` / codepoint `0x6620` selected at v1-beta — reserved-and-REFUSED
    /// (additive over the crypto-agility framework; never a silent accept).
    #[error("federation SubsetRef / 0x6620 is reserved-and-refused at v1-beta")]
    FederationReserved,
    /// Model-A selected at v1-beta — opt-in post-v1-beta additive, NOT selectable.
    #[error("federation Model-A is post-v1-beta additive; not selectable at v1-beta")]
    ModelAUnavailable,
}

impl FederationError {
    /// The stable [`benten_errors::ErrorCode`] for this federation-dispatch
    /// rejection. Both variants are reserved-codepoint / not-yet-selectable
    /// typed-rejects; following the [`crate::error::MembershipSetError`]
    /// precedent they route through the generic `CapDenied` disposition — NO new
    /// catalog variant is minted (the federation path is RESERVED-and-REFUSED at
    /// v1-beta; there is no positive wire path to mirror).
    #[must_use]
    pub fn error_code(&self) -> benten_errors::ErrorCode {
        match self {
            FederationError::FederationReserved | FederationError::ModelAUnavailable => {
                benten_errors::ErrorCode::CapDenied
            }
        }
    }
}

/// The federation model toggle (Inv-20 clause-l).
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future federation-model variant
/// lands additively, never a downstream `match` break.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum FederationModel {
    /// DEFAULT — independent `K_Set` per set.
    ModelB,
    /// Opt-in post-v1-beta additive; NOT selectable at v1-beta.
    ModelA,
}

/// Admit a `SubsetRef` member at v1-beta → typed-reject (reserved-and-refused).
///
/// # Errors
///
/// Always returns [`FederationError::FederationReserved`] at v1-beta.
pub fn admit_subset_ref_at_v1_beta() -> Result<(), FederationError> {
    Err(FederationError::FederationReserved)
}

/// Federation-reserve gate at v1-beta: typed-reject the reserved federation
/// `0x6620` (`SubsetRef`) codepoint; every OTHER codepoint passes.
///
/// **Fail-OPEN shape (R10-council F-16):** this is a targeted BLOCKLIST of one
/// reserved codepoint, NOT a general codepoint dispatcher — an unknown/future
/// codepoint returns `Ok(())` (passes). At v1-beta the sole test-only callers
/// only ever feed it the frozen MembershipSet band, so the fail-open shape is
/// contained, but the inverted → **ALLOWLIST** follow-up (return `Ok(())` ONLY
/// for the explicitly-enumerated frozen codepoints, typed-reject everything
/// else) is the correct fail-CLOSED end-state — tracked in
/// `docs/CRYPTO-CODEPOINTS.md` (federation-reserve-gate inversion note).
///
/// # Errors
///
/// Returns [`FederationError::FederationReserved`] iff `cp ==
/// MEMBERSHIP_SET_RESERVED_0X6620` (`0x6620`).
pub fn federation_reserve_gate_at_v1_beta(cp: u16) -> Result<(), FederationError> {
    if cp == MEMBERSHIP_SET_RESERVED_0X6620 {
        return Err(FederationError::FederationReserved);
    }
    Ok(())
}

/// Select a federation model at v1-beta — only Model-B is selectable.
///
/// # Errors
///
/// Returns [`FederationError::ModelAUnavailable`] if Model-A is requested.
pub fn select_model_at_v1_beta(m: FederationModel) -> Result<FederationModel, FederationError> {
    match m {
        FederationModel::ModelB => Ok(FederationModel::ModelB),
        FederationModel::ModelA => Err(FederationError::ModelAUnavailable),
        // NOTE: no `_` arm. `FederationModel` is `#[non_exhaustive]` for
        // downstream SemVer-readiness, but within the defining crate this match
        // stays exhaustive — a future model variant HALT-AND-SURFACEs here,
        // forcing an explicit selectable/typed-reject decision.
    }
}
