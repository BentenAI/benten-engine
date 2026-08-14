//! Phase-4-Meta-Core G-CORE-8 §4.23 — structural-always-on user-DID
//! root write-boundary chain validator.
//!
//! # What this seam does
//!
//! CLAUDE.md baked-in #18 Layer-1 user-as-root invariant
//! (runtime-enforced): EVERY WRITE's UCAN delegation chain MUST trace
//! back to a registered user-DID root grant. A plugin-DID-minted root
//! chain is structurally rejected (a plugin cannot mint its own root
//! authority).
//!
//! This module hosts the [`WriteBoundaryChainValidator`] port that the
//! engine's WRITE admission path consults BEFORE the write lands. The
//! seam is intended to be **structural-always-on** at the WRITE
//! admission boundary (mirroring Phase-3 G16-B-F structural-always-on
//! per-row cap-recheck — fail-CLOSED, NOT an opt-in).
//!
//! # Layering / dep-direction discipline
//!
//! The port lives in `benten-engine` (the trust-boundary owner). The
//! concrete implementation that consults the engine's install-record-
//! backed `UserDidRegistry` + composes
//! `benten_caps::validate_chain_with_manifest_envelope` lives in the
//! engine-adapter crate that owns the plugin library (typically
//! `benten-platform-foundation`'s glue type). Engine code path here
//! NEVER imports `benten-platform-foundation`.
//!
//! # G-CORE-8 deliverable scope
//!
//! G-CORE-8 lands:
//! - The [`WriteBoundaryChainValidator`] trait + outcome shape.
//! - The [`NoopWriteBoundaryChainValidator`] default (returns
//!   `NotApplicable` for every chain → admit; the "no validator
//!   installed" case).
//! - The typed [`benten_errors::ErrorCode::WriteBoundaryChainNotUserRooted`]
//!   reject code + the [`outcome_to_admission_reject`] mapping that the
//!   WRITE admission path calls.
//! - The [`crate::Engine::set_write_boundary_chain_validator`] setter
//!   so platform-foundation can swap in the substantive validator at
//!   engine construction time.
//!
//! G-CORE-8.2 consumption — **LANDED (Row D-1 CLOSED at R6 R1 FP-F4 §S1,
//! 2026-05-24; sharpened at R6 R2 FP-B, 2026-05-25).** The actual call into
//! the WRITE admission path is now wired at every WRITE call site: the
//! `Engine::commit` / `Engine::put_node_with_context` cluster consults the
//! configured validator via
//! `write_boundary_chain_validator::WriteAdmissionFrame::engine_internal()` /
//! `::with_chain(...)` (see `engine_crud.rs`, `engine_caps.rs`,
//! `engine_wait.rs`, `handler_versions.rs`, `engine_views.rs`; the Atrium
//! merge path consults its own `manifest_envelope_recheck` seam in
//! `Engine::apply_atrium_merge`). The full WRITE-call-site audit (Phase-3
//! G16-B-F precedent) + integration-tests are complete. The one residual
//! Ben-decision — the engine-installed-DEFAULT posture (admit vs reject when
//! no validator is installed) — follows the §8-E SEALED-discipline framework
//! (which ratified `CapabilityPolicy` sealing; the parallel
//! `WriteBoundaryChainValidator` default-builder posture rides the same
//! framework); the shipped default here is [`NoopWriteBoundaryChainValidator`]
//! (returns `NotApplicable` → admit) until platform-foundation swaps in the
//! substantive validator via [`crate::Engine::set_write_boundary_chain_validator`].

use benten_errors::ErrorCode;

use crate::EngineError;

/// **R6 R1 FP-F4 (Δv3-2) — sealed write-admission frame** carrying the
/// optional chain anchor + actor DID the WRITE entry points present to
/// `Engine::admit_write_chain`. Sealed (private fields) per Class B β +
/// §1.A.FROZEN item 8 strict-additivity discipline; adding a field is
/// a non-breaking internal change because no external constructor
/// exists (only the two `pub` builder fns below).
///
/// The frame is the input shape for the structural-always-on WRITE
/// admission consultation. Most engine-internal WRITE entry points have
/// no UCAN delegation chain in scope (engine-internal writes are
/// authoritative-by-construction; the user-DID acts as the principal);
/// these pass [`WriteAdmissionFrame::engine_internal`]. User-facing CRUD
/// entry points / delegation / sync-merge can present a chain anchor
/// when one is in scope.
///
/// **Why a sealed frame instead of extending `CapWriteContext` with a
/// new field:** the CRITIC-2 disposition for D-17 + strict-additivity
/// prefers ONE local sealed-shape addition over a cascade through every
/// test fixture that names CapWriteContext.
#[derive(Debug, Clone, Default)]
pub struct WriteAdmissionFrame<'a> {
    chain_anchor_cid: Option<&'a benten_core::Cid>,
    actor_did: Option<&'a str>,
}

impl<'a> WriteAdmissionFrame<'a> {
    /// Engine-internal write: no UCAN chain anchor in scope. The
    /// validator returns
    /// [`WriteBoundaryChainOutcome::NotApplicable`] and the write
    /// proceeds (Layer-1 user-root enforcement at
    /// `CapabilityPolicy::check_write` remains in force).
    #[must_use]
    pub fn engine_internal() -> Self {
        Self {
            chain_anchor_cid: None,
            actor_did: None,
        }
    }

    /// A write that carries a UCAN chain anchor + actor DID — the
    /// validator walks the chain backward from the anchor through the
    /// engine's `UserDidRegistry` (per the configured
    /// [`WriteBoundaryChainValidator`] impl).
    #[must_use]
    pub fn with_chain(chain_anchor_cid: &'a benten_core::Cid, actor_did: &'a str) -> Self {
        Self {
            chain_anchor_cid: Some(chain_anchor_cid),
            actor_did: Some(actor_did),
        }
    }

    /// The chain anchor CID, if any.
    #[must_use]
    pub fn chain_anchor_cid(&self) -> Option<&benten_core::Cid> {
        self.chain_anchor_cid
    }

    /// The actor DID, if any.
    #[must_use]
    pub fn actor_did(&self) -> Option<&str> {
        self.actor_did
    }
}

/// Outcome of a write-boundary chain validation call.
///
/// `#[non_exhaustive]` per V1-FROZEN-INTERFACE.md item 11 + L6-r1-2
/// (G-CORE-9 R1 fix-pass): adding a new variant post-v1 is breaking;
/// the attribute makes the variant-set additively extensible.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WriteBoundaryChainOutcome {
    /// No UCAN delegation chain is in scope for this write (e.g. a
    /// direct user-DID-signed write with no plugin-delegation chain),
    /// or no validator is installed (the Noop case). Layer-1
    /// user-root enforcement at the policy boundary
    /// (`CapabilityPolicy::check_write`) is the relevant defense for
    /// this path; the chain validator does not over-fire on direct
    /// user writes.
    NotApplicable,
    /// The chain was found AND its root anchors at a registered
    /// user-DID. Admit the write.
    Admitted,
    /// The chain was found BUT does NOT terminate at a registered
    /// user-DID — the chain root is a plugin-DID or other non-user
    /// DID. CLAUDE.md baked-in #18 Layer-1 elevation defense; reject
    /// with typed
    /// [`ErrorCode::WriteBoundaryChainNotUserRooted`].
    ChainNotUserRooted {
        /// The chain root DID that is not a registered user-DID.
        chain_root_did: String,
    },
}

/// Port the engine adapter implements to drive write-boundary chain
/// validation at the WRITE admission seam.
///
/// Implementations are typically a thin glue over the engine's
/// install-record-backed `UserDidRegistry` (the set of user-DIDs
/// derived from the user-signed `InstallRecord`s the engine has
/// admitted) + `benten_caps::validate_chain_with_manifest_envelope`
/// (the composing chain validator from G24-D-FP-2 that returns
/// `ChainValidationOutcome::RootNotUserDid` for the failure case).
///
/// The default implementation [`NoopWriteBoundaryChainValidator`]
/// returns `NotApplicable` for every call — equivalent to no
/// validator installed.
pub trait WriteBoundaryChainValidator: Send + Sync {
    /// Validate the UCAN delegation chain that authorizes a pending
    /// write. The chain is represented as an opaque CID-keyed
    /// reference into the engine's grant store; implementations
    /// resolve the chain through the configured `GrantReader` and
    /// walk it through `validate_chain_with_manifest_envelope`.
    ///
    /// `chain_anchor_cid` is the CID of the grant the write directly
    /// presents; the validator walks it backward to the root.
    /// `actor_did` is the principal performing the write (for
    /// diagnostic correlation).
    fn validate_chain(
        &self,
        chain_anchor_cid: &benten_core::Cid,
        actor_did: &str,
    ) -> WriteBoundaryChainOutcome;
}

/// Default validator — returns `NotApplicable` for every call.
///
/// Used by the engine builder when no platform-foundation glue is
/// installed; deployments with a PluginLibrary install a substantive
/// `ProductionWriteBoundaryChainValidator` via
/// [`crate::Engine::set_write_boundary_chain_validator`].
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopWriteBoundaryChainValidator;

impl WriteBoundaryChainValidator for NoopWriteBoundaryChainValidator {
    fn validate_chain(
        &self,
        _chain_anchor_cid: &benten_core::Cid,
        _actor_did: &str,
    ) -> WriteBoundaryChainOutcome {
        // The Noop has no UserDidRegistry to consult; Layer-1
        // enforcement at the policy boundary remains the defense for
        // this path.
        WriteBoundaryChainOutcome::NotApplicable
    }
}

/// Map a [`WriteBoundaryChainOutcome`] to a [`Result`] suitable for
/// the WRITE admission path. `Admitted` / `NotApplicable` → proceed;
/// `ChainNotUserRooted` → typed reject with
/// [`ErrorCode::WriteBoundaryChainNotUserRooted`].
///
/// Exposed `pub` so test pins can exercise the outcome-to-reject
/// mapping without spinning a full Engine harness. The WRITE
/// admission wire-up (in `Engine::commit`) calls this same helper.
pub fn outcome_to_admission_reject(outcome: WriteBoundaryChainOutcome) -> Result<(), EngineError> {
    match outcome {
        WriteBoundaryChainOutcome::Admitted | WriteBoundaryChainOutcome::NotApplicable => Ok(()),
        WriteBoundaryChainOutcome::ChainNotUserRooted { chain_root_did } => {
            Err(EngineError::Other {
                code: ErrorCode::WriteBoundaryChainNotUserRooted,
                message: format!(
                    "write admission: capability chain does not terminate at a registered \
                     user-DID root (chain_root_did='{chain_root_did}'): CLAUDE.md baked-in #18 \
                     Layer-1 user-as-root invariant — a plugin-DID-minted root chain is \
                     structurally rejected (G-CORE-8 §4.23)"
                ),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use benten_core::Cid;

    #[test]
    fn noop_returns_not_applicable() {
        let noop = NoopWriteBoundaryChainValidator;
        let cid = Cid::from_blake3_digest([1u8; 32]);
        assert_eq!(
            noop.validate_chain(&cid, "did:key:zUser"),
            WriteBoundaryChainOutcome::NotApplicable
        );
    }

    #[test]
    fn admitted_proceeds() {
        assert!(outcome_to_admission_reject(WriteBoundaryChainOutcome::Admitted).is_ok());
    }

    #[test]
    fn not_applicable_proceeds() {
        assert!(outcome_to_admission_reject(WriteBoundaryChainOutcome::NotApplicable).is_ok());
    }

    #[test]
    fn chain_not_user_rooted_rejects_with_typed_code() {
        let outcome = WriteBoundaryChainOutcome::ChainNotUserRooted {
            chain_root_did: "did:key:zPluginRoot".to_string(),
        };
        let err = outcome_to_admission_reject(outcome).expect_err("must reject");
        assert_eq!(err.code(), ErrorCode::WriteBoundaryChainNotUserRooted);
    }
}
