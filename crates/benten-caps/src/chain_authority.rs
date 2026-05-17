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

use benten_id::device_attestation::{CapabilityEnvelope, DeviceAttestation};
use benten_id::did::Did;
use benten_id::did_rotation::RotationLog;
use benten_id::errors::UcanError;
use benten_id::ucan::{Ucan, validate_chain_no_time_check};
use benten_platform_foundation::PluginManifest;
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

// =====================================================================
// COLLAPSE P5 — #669 plugin-manifest ceiling: the SECOND caller of the
// ONE `envelope_ceiling_rejects_cap` predicate (build-constraint iii,
// DECISION-RECORD §4 — ONE code path, two callers, NOT a parallel pipe).
// =====================================================================

/// Derive a [`CapabilityEnvelope`]-shaped ceiling from a plugin
/// manifest's `requires` half.
///
/// CLAUDE.md #18 trust model: a plugin's `requires` is the *only* set
/// of caps the user consented to at install. The plugin-manifest
/// envelope is **structurally identical to the device-attestation
/// envelope** (DECISION-RECORD §4 / impl-design-COLLAPSE §2 — "the
/// manifest envelope and the device envelope are the same
/// abstraction"). The single load-bearing ceiling dimension this seam
/// enforces (per the P2 scope note on
/// [`validate_chain_with_envelope_ceiling`]) is `runs_sandbox`: a
/// plugin that did NOT declare a `host:sandbox:*` capability in its
/// signed, user-consented `requires` half is — for the purpose of the
/// CLAUDE.md #17 thin-shape ceiling — a `runs_sandbox=false`
/// principal, exactly like a thin device. Any other manifest dimension
/// (`holds_zones` / `online_uptime` / `runs_atrium_peer`) is NOT a
/// manifest concept and stays `CapabilityEnvelope::default()` so the
/// SAME [`envelope_ceiling_rejects_cap`] predicate (which only reads
/// `runs_sandbox`) governs both shapes without a parallel rule.
///
/// This is the adapter that makes the manifest path a *caller of the
/// existing predicate*, not a new pipe.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn manifest_to_envelope_ceiling(manifest: &PluginManifest) -> CapabilityEnvelope {
    // The plugin runs sandbox iff it declared a host:sandbox:*
    // requirement in the signed manifest the user consented to.
    let declares_sandbox = manifest
        .requires
        .iter()
        .any(|req| req.scope.starts_with("host:sandbox:"));
    CapabilityEnvelope {
        runs_sandbox: declares_sandbox,
        ..CapabilityEnvelope::default()
    }
}

/// Validate a chain against a plugin manifest's install-consented
/// ceiling — the #669 Layer-2/Layer-3 envelope decision, enforced
/// through the **same** [`envelope_ceiling_rejects_cap`] predicate the
/// device-envelope caller uses.
///
/// This is COLLAPSE P5's deliverable: the #669 plugin-manifest ceiling
/// is wired as a *second caller of the one factored predicate*, NOT a
/// parallel pipe. Compare its body to
/// [`validate_chain_with_envelope_ceiling`] — the ONLY difference is
/// the envelope source (a `DeviceAttestation`'s signed `envelope` vs a
/// `PluginManifest`'s install-consented `requires`-derived ceiling).
/// The per-capability rule (`envelope_ceiling_rejects_cap`), the error
/// variant, and the chain-walk structure are identical. The #707 /
/// META-#1140 asymmetric-parallel-pipe class the COLLAPSE exists to
/// kill is therefore NOT recreated: there is one ceiling rule, in one
/// function, called from two adapter sites.
///
/// `plugin_did` is the DID the manifest was installed under (the UCAN
/// `audience` the user delegated to). Only chain links *issued by* that
/// plugin-DID are ceiling-checked — a manifest constrains what *its*
/// plugin may exercise, exactly as a device attestation constrains
/// what *that device* may exercise.
///
/// Rejects with [`UcanError::DeviceEnvelopeViolated`] (the shared
/// envelope-ceiling error; the `issuer`/`cap` payload identifies the
/// offending plugin-DID + cap) when a token issued by `plugin_did`
/// exercises a cap outside the manifest-derived ceiling — e.g.
/// `host:sandbox:exec` from a manifest whose `requires` never declared
/// a `host:sandbox:*` capability (the user never consented to this
/// plugin running sandbox).
#[cfg(not(target_arch = "wasm32"))]
pub fn validate_chain_with_manifest_ceiling(
    chain: &[Ucan],
    manifest: &PluginManifest,
    plugin_did: &Did,
) -> Result<(), UcanError> {
    // Pure structural+crypto validation stays a `benten-id` primitive
    // call (the CONSOLIDATE line); only the manifest-ceiling
    // *authority consultation* is local policy.
    validate_chain_no_time_check(chain)?;
    let ceiling = manifest_to_envelope_ceiling(manifest);
    for token in chain {
        // ct-eq — same identity-compare discipline as the device
        // caller (g14-a2-mr-2 UNIFORMITY); the manifest constrains
        // only links the plugin itself issued.
        if ct_bytes_eq(plugin_did.as_str().as_bytes(), token.claims.iss.as_bytes()) {
            for cap in &token.claims.att {
                // THE ONE PREDICATE — identical call the device
                // caller makes. build-constraint iii.
                if envelope_ceiling_rejects_cap(&ceiling, &cap.resource) {
                    return Err(UcanError::DeviceEnvelopeViolated {
                        issuer: token.claims.iss.clone(),
                        cap: format!("{}:{}", cap.resource, cap.ability),
                    });
                }
            }
        }
    }
    Ok(())
}

// =====================================================================
// COLLAPSE P5 — #1241 / F2 cap-predicate completion.
//
// DECISION-RECORD §4b RATIFIED: the ceiling MUST be able to
// discriminate on the writer's *actual UCAN `cap.resource`*, not just
// a synthetic `{zone}:write` scope (the shape-not-substance gap F2
// named — `F2-exploitability-investigation.md`). The engine-side
// `apply_atrium_merge` recheck passed `row_scope = "{zone}:write"`,
// which can never `starts_with("host:sandbox:")`, so a
// `runs_sandbox=false` writer self-delegating `host:sandbox:exec` was
// NOT rejected. #1241 completes the predicate so the REAL production
// arm (the writer's own delegated cap resources) is checked through
// the SAME `envelope_ceiling_rejects_cap` predicate.
// =====================================================================

/// The #1241 cap-predicate completion: AND the verified inbound device
/// envelope-ceiling against the inbound writer's **actual delegated
/// `cap.resource` set**, not a synthetic zone-write scope.
///
/// This closes the F2 shape-not-substance gap
/// (`F2-exploitability-investigation.md`, DECISION-RECORD §4b): the
/// pre-P5 engine seam discriminated on `format!("{zone}:write")` — a
/// string that structurally can never trip the `host:sandbox:`
/// prefix, so the CLAUDE.md #17 thin-shape ceiling was never actually
/// enforced against a sandbox-cap-bearing writer on the inbound path.
///
/// `cap_resources` is the inbound writer's actual delegated cap
/// `resource` strings (from the verified UCAN chain the merge
/// admitted). Every one is run through the SAME
/// [`envelope_ceiling_rejects_cap`] predicate the device + manifest
/// chain-walkers use — ONE code path, build-constraint iii. Returns
/// the offending cap-resource on the first violation so the engine
/// can map it to its typed reject; `Ok(())` when every cap-resource
/// is within the ceiling (or no ceiling is present).
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn envelope_ceiling_first_rejected_resource<'a>(
    ceiling: Option<&CapabilityEnvelope>,
    cap_resources: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    let env = ceiling?;
    for resource in cap_resources {
        // THE ONE PREDICATE — same call the device + manifest
        // chain-walkers make. The F2 completion is *what we feed it*
        // (the writer's real cap.resource, not "{zone}:write").
        if envelope_ceiling_rejects_cap(env, resource) {
            return Some(resource.to_string());
        }
    }
    None
}

// =====================================================================
// COLLAPSE P5 — F3 durable replay-marker re-home.
//
// DECISION-RECORD §4b RATIFIED: the nonce/freshness durable marker P3
// deferred ("durable replay-marker re-home: tracked P2/P5") is homed
// HERE, in the single chain-validation crate, consuming the existing
// durable-KV grammar (`benten-graph::GraphBackend` put/get keyed by a
// stable prefix — the same pattern `UCANBackend::revoke`/`is_revoked`
// uses). The COLLAPSE Compromise #23 rewrite promises "anti-replay …
// re-homed into the unified chain-validation seam … not dropped"; the
// P3 `DeviceAttestationEnvelope::verify` step-(3) freshness gate is
// ephemeral-only (it cannot catch a *replay within the freshness
// window*). This durable marker closes that: a frame whose
// attestation nonce was already observed is rejected even if it is
// still inside its freshness window.
// =====================================================================

const KV_FRAME_REPLAY_PREFIX: &[u8] = b"collapse:framenonce:";

/// Durable replay-marker seam for inbound sync frames (F3 re-home).
///
/// Re-homes the anti-replay defense the COLLAPSE Compromise #23
/// rewrite promised "not dropped" into the single chain-validation
/// crate. `mark_and_check_frame` records the inbound attestation's
/// nonce in the durable KV and reports whether it was *already*
/// present — a replayed frame (same nonce re-presented inside the
/// freshness window, which the ephemeral `verify` step-(3) cannot
/// catch) is observably rejectable.
///
/// Generic over [`benten_graph::GraphBackend`] so it reuses the exact
/// durable substrate `UCANBackend::revoke`/`is_revoked` use — one
/// durable grammar, no parallel store.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct FrameReplayMarker<B: benten_graph::GraphBackend> {
    backend: std::sync::Arc<B>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<B: benten_graph::GraphBackend> FrameReplayMarker<B> {
    /// Construct over the engine's durable backend.
    #[must_use]
    pub fn new(backend: std::sync::Arc<B>) -> Self {
        Self { backend }
    }

    fn nonce_key(nonce: &[u8]) -> Vec<u8> {
        let mut key = Vec::with_capacity(KV_FRAME_REPLAY_PREFIX.len() + nonce.len());
        key.extend_from_slice(KV_FRAME_REPLAY_PREFIX);
        key.extend_from_slice(nonce);
        key
    }

    /// Record `nonce` as observed and report whether it had ALREADY
    /// been observed (i.e. this is a replay).
    ///
    /// Returns `Ok(true)` when the nonce was already durably present
    /// (REPLAY — the caller MUST reject the frame); `Ok(false)` on
    /// first observation (the marker is now persisted).
    ///
    /// # Errors
    ///
    /// Returns [`crate::CapError::BackendStorage`] on KV read/write failure.
    pub fn mark_and_check_frame(&self, nonce: &[u8]) -> Result<bool, crate::CapError> {
        let key = Self::nonce_key(nonce);
        let already = self
            .backend
            .get(&key)
            .map_err(|e| crate::CapError::BackendStorage {
                reason: format!("KV get frame-replay marker: {e}"),
            })?
            .is_some();
        if !already {
            self.backend
                .put(&key, &[])
                .map_err(|e| crate::CapError::BackendStorage {
                    reason: format!("KV put frame-replay marker: {e}"),
                })?;
        }
        Ok(already)
    }
}
