//! Policy-bearing UCAN chain-authority consultation (the CONSOLIDATE
//! residence per `impl-design-COLLAPSE.md` §2, COLLAPSE P2).
//!
//! ## Why this lives in `benten-caps`, not `benten-id`
//!
//! The CONSOLIDATE line (DECISION-RECORD-trust-model-reframe.md §4,
//! RATIFIED 2026-05-15) is:
//!
//! - **Pure crypto/structural validation = `benten-id`.** The
//!   signature + per-link attenuation + time-window primitive
//!   ([`benten_id::ucan::validate_chain_at`] /
//!   [`benten_id::ucan::validate_chain_no_time_check`]) has no policy
//!   and stays in `benten-id`.
//! - **Policy-bearing authority consultation = `benten-caps`.** The
//!   chain-walks that consult an *authority surface* — the rotation
//!   log (is this issuer's keypair superseded?) and the
//!   envelope-ceiling (does this issuer's signed deployment-shape
//!   ceiling forbid the claimed capability?) — are policy. They move
//!   here, *with* the crate dependency arrow
//!   (`benten-caps` → `benten-id`; never the reverse), so there is
//!   exactly **one authority seam, in one crate**.
//!
//! Pre-COLLAPSE these two walkers lived as
//! `benten_id::ucan::validate_chain_with_rotation_log` and
//! `benten_id::ucan::validate_chain_with_attestations`. COLLAPSE P1
//! (#1238) already deleted the redundant device-revocation parallel
//! pipe (`validate_chain_with_device_revocations` / `DeviceRevocation`
//! / `Acceptor`); P2 (this module) moves the *surviving* two
//! authority walkers out of `benten-id` so `benten-id` is left with
//! pure key/DID/sig/rotation/envelope-*type* primitives only.
//!
//! ## The single generalized ceiling seam
//!
//! [`validate_chain_with_envelope_ceiling`] is the **one** code path
//! that AND-s an `envelope_widens`-style narrowing into chain
//! validation (per impl-design-COLLAPSE.md §2 item 1 + DECISION-RECORD
//! §4 build-constraint iii: "J8-caveat + #669-ceiling-check are ONE
//! code path"). COLLAPSE P5 (`feat(#669)`) extends THIS function with
//! the plugin-manifest `shares`/`requires` ceiling as a *second
//! caller* of the same predicate ([`envelope_ceiling_rejects_cap`]) —
//! NOT a parallel pipe. The predicate is factored out so the device
//! shape and the future manifest shape enforce the identical
//! `runs_sandbox=false → reject host:sandbox:*` rule through one
//! function, eliminating the #707 / META-#1140 asymmetric-parallel-pipe
//! class the COLLAPSE exists to kill.

#![cfg(not(target_arch = "wasm32"))]

use benten_id::device_attestation::DeviceAttestation;
use benten_id::did::Did;
use benten_id::did_rotation::RotationLog;
use benten_id::errors::UcanError;
use benten_id::ucan::{Ucan, validate_chain_no_time_check};
use subtle::ConstantTimeEq;

/// Constant-time byte-string equality, mirroring the crypto-decision
/// `subtle::ConstantTimeEq` discipline already used at the
/// `benten-caps` chain-walk layer (see
/// `crates/benten-caps/src/backends/ucan.rs` audience-binding) and the
/// pre-move `ct_signature_eq` UNIFORMITY commitment
/// (g14-a2-mr-2 fix-pass). The capability `resource`/`ability` and DID
/// strings are the cap-system's public schema rather than secrets, but
/// every authority-decision comparison goes through constant-time eq
/// so the cross-crate move preserves the exact security posture of the
/// pre-COLLAPSE `benten-id` walker.
fn ct_bytes_eq(a: &[u8], b: &[u8]) -> bool {
    // const-time-eq
    a.ct_eq(b).into()
}

/// Validate a chain against a [`RotationLog`].
///
/// Moved verbatim (behavior-preserving) from
/// `benten_id::ucan::validate_chain_with_rotation_log` per CONSOLIDATE
/// P2. The `RotationLog` *type* + `accept_rotation_event` stay in
/// `benten_id::did_rotation` (J4 pure key primitive, unchanged); only
/// the *chain-walk that consults it as an authority* moves here.
///
/// Per `crates/benten-caps/tests/collapse_p2_consolidate_chain_authority.rs`,
/// any UCAN whose issuer DID has been rotated rejects with
/// [`UcanError::IssuerKeypairSuperseded`] — the chain-walker consults
/// the rotation log so post-rotation UCANs from the OLD keypair are
/// observably rejected even when their signature is structurally
/// valid. (Behavioral parity with the pre-COLLAPSE-P2 benten-id
/// rotation-supersession pin, which moved here with this function.)
pub fn validate_chain_with_rotation_log(
    chain: &[Ucan],
    log: &RotationLog,
) -> Result<(), UcanError> {
    // Pure structural+crypto validation stays a `benten-id` primitive
    // call (the CONSOLIDATE line); only the rotation-log *authority
    // consultation* is local policy.
    validate_chain_no_time_check(chain)?;
    for token in chain {
        let did = Did::from_string_unchecked(token.claims.iss.clone());
        if log.is_superseded(&did) {
            return Err(UcanError::IssuerKeypairSuperseded {
                issuer: token.claims.iss.clone(),
            });
        }
    }
    Ok(())
}

/// The single envelope-ceiling predicate: does `cap` violate the
/// `envelope`'s deployment-shape ceiling for an issuer that the
/// `envelope` attests?
///
/// **This is the unified seam P5 (#669) extends.** The device-shape
/// caller ([`validate_chain_with_envelope_ceiling`]) and the future
/// plugin-manifest `shares`/`requires` caller (COLLAPSE P5) both call
/// THIS predicate — ONE code path, build-constraint iii
/// (DECISION-RECORD §4). The rule is the load-bearing CLAUDE.md #17
/// thin-shape property: a `runs_sandbox=false` principal cannot
/// exercise `host:sandbox:*` even with an otherwise-valid chain.
///
/// Returns `true` when the capability is REJECTED by the ceiling.
pub fn envelope_ceiling_rejects_cap(
    envelope: &benten_id::device_attestation::CapabilityEnvelope,
    resource: &str,
) -> bool {
    resource.starts_with("host:sandbox:") && !envelope.runs_sandbox
}

/// Validate a chain against a list of signed envelope-ceiling
/// attestations.
///
/// Moved + generalized from
/// `benten_id::ucan::validate_chain_with_attestations` per CONSOLIDATE
/// P2 (impl-design-COLLAPSE.md §2 item 1). Renamed to
/// `validate_chain_with_envelope_ceiling` to make the
/// device↔plugin-manifest unification lexically visible: a
/// [`DeviceAttestation`] is, post-COLLAPSE-P1-demotion, "a signed
/// envelope-ceiling attestation on a principal key", not a
/// device-trust-root. The per-capability ceiling rule is factored
/// into [`envelope_ceiling_rejects_cap`] so COLLAPSE P5 wires the
/// #669 plugin-manifest ceiling as a *second caller of the same
/// predicate*, not a parallel pipe.
///
/// Per `crates/benten-caps/tests/collapse_p2_consolidate_chain_authority.rs`,
/// rejects with [`UcanError::DeviceEnvelopeViolated`] when a token's
/// issuer has an attestation declaring it cannot exercise the claimed
/// capability (e.g. `host:sandbox:exec` from a `runs_sandbox=false`
/// device). Behavioral parity with the deleted
/// `benten-id::tests/device_attestation.rs` consume-time pins.
///
/// **Scope (post-COLLAPSE):** this validate-side seam enforces the
/// `runs_sandbox` envelope dimension only. Broader multi-dimension
/// envelope enforcement (`runs_atrium_peer`, `holds_zones`,
/// `online_uptime`) is NOT enforced here — it collapses to the
/// engine's single inbound-sync envelope-ceiling recheck plus
/// user-root UCAN revocation (the existing
/// [`crate::backends::UCANBackend::revoke`] self-anchored content-CID
/// seam). This function is the pure `runs_sandbox` ceiling-AND; the
/// engine-side recheck consumes the same predicate.
pub fn validate_chain_with_envelope_ceiling(
    chain: &[Ucan],
    attestations: &[DeviceAttestation],
) -> Result<(), UcanError> {
    // Pure structural+crypto validation stays a `benten-id` primitive
    // call (the CONSOLIDATE line); only the envelope-ceiling
    // *authority consultation* is local policy.
    validate_chain_no_time_check(chain)?;
    for token in chain {
        for att in attestations {
            // const-time-eq — ct discipline preserved across the move
            // (was `ct_signature_eq` in the pre-COLLAPSE benten-id
            // walker; g14-a2-mr-2 UNIFORMITY).
            if ct_bytes_eq(att.device_did.as_bytes(), token.claims.iss.as_bytes()) {
                for cap in &token.claims.att {
                    if envelope_ceiling_rejects_cap(&att.envelope, &cap.resource) {
                        return Err(UcanError::DeviceEnvelopeViolated {
                            issuer: token.claims.iss.clone(),
                            cap: format!("{}:{}", cap.resource, cap.ability),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}
