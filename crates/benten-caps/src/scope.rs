//! `Scope` — the structured cap-grant scope enum for G-CORE-3b.
//!
//! ## EXACTLY two arms — no-opaque-arm decision
//!
//! Per `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
//! §R1 (RATIFIED 2026-05-21):
//!
//! - **`Hashes(Vec<Cid>)`** — explicit content-CID set. The recipient
//!   may serve / decrypt the exact named CIDs.
//! - **`RestrictedSelector(RestrictedScope)`** — structural restriction
//!   over a sub-graph language (Path (a)). See [`RestrictedScope`] for
//!   the 6-dimensional language.
//!
//! **THERE IS NO `OpaqueSelector` ARM.** Path (b) (refinement-witness
//! over opaque specs) is structurally unsound per Spike H+1.1 §b.SEC #4
//! — the witness binds the attestation, not the spec body; the eval-set
//! witness is DOA at scale (12.2× wire-size, exponential attestation
//! explosion). Adding a third arm here would re-open the §1.A.FROZEN
//! item 15(c) freeze surface and requires HALT-AND-SURFACE-TO-BEN
//! escalation per HARD RULE 12.
//!
//! Sentinel marker for the cite-drift CI lane + future agents reading
//! this surface: `no-opaque-arm`.

use benten_core::Cid;
use serde::{Deserialize, Serialize};

use crate::restricted_spec::RestrictedScope;

/// Structured cap-grant scope. EXACTLY two arms by the
/// `no-opaque-arm` decision (RATIFIED-S&C 2026-05-21 §R1).
///
/// **NOT `#[non_exhaustive]`** — the §1.A.FROZEN item 15(c) explicit
/// "EXACTLY two arms; NO `OpaqueSelector`" freeze is STRONGER than the
/// §1.A.FROZEN item 11 META #907 default of `#[non_exhaustive]` on
/// public enums. The freeze MUST be enforced by the type system so a
/// future agent who proposes a third arm cannot land it as a "minor
/// version bump"; instead they get exhaustive-match compile failures
/// at every downstream call site + the cite-drift CI lane fires on the
/// `no-opaque-arm` sentinel below. The two-arm count IS the structural
/// pin (per the wave-3b `tf3b_no_opaque_selector_arm_structural` test
/// arm A-4.1: a third arm trips a missing-pattern compile error rather
/// than an `#[non_exhaustive]`-permitted wildcard). Adding a third arm
/// is a HALT-AND-SURFACE-TO-BEN escalation per HARD RULE 12.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    /// Explicit content-CID set — recipient may serve / decrypt the
    /// named CIDs (and only those).
    Hashes(Vec<Cid>),
    /// Structural sub-graph restriction (Path (a) of RATIFIED §R1).
    RestrictedSelector(RestrictedScope),
}
