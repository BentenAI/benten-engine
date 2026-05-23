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
//! G-CORE-8.2 follow-up (HARD-RULE-12 BELONGS-NAMED-NOW disposition;
//! named destination: this module's `Engine::commit`-path wire-up):
//! the actual call into the WRITE admission path inside
//! `Engine::commit` / `Engine::put_node_with_context` — pulled to a
//! follow-up wave because (a) it requires audit of every WRITE call
//! site to ensure no admission path is missed (Phase-3 G16-B-F
//! precedent), (b) integration-tests + sync interaction need a full
//! validator wired (Atrium merge → Engine::apply_atrium_merge already
//! has its own seam at `manifest_envelope_recheck`), and (c) the
//! engine-installed-default behavior choice (admit vs reject when no
//! validator is installed) is a Ben-decision deferral per §8-E
//! sealed-discipline (the §8-E ratification covers `CapabilityPolicy`
//! sealing; the parallel question for `WriteBoundaryChainValidator`'s
//! default-builder posture follows the same SEALED-discipline
//! framework and waits for the corresponding ratification).

use benten_errors::ErrorCode;

use crate::EngineError;

/// Outcome of a write-boundary chain validation call.
#[derive(Debug, Clone, PartialEq, Eq)]
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
