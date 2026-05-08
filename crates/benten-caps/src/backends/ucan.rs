//! Durable UCAN backend (G14-B wave-4b).
//!
//! Replaces the Phase-2b stub `crate::ucan_stub::LegacyUcanStubBackend` — which
//! returned [`CapError::NotImplemented`] from every entry point —
//! with a durable grant store backed by the
//! [`benten_graph::GraphBackend`] umbrella trait (G13-A) plus full
//! UCAN chain-walk validation (delegation, attenuation, revocation,
//! `nbf` / `exp` time-window enforcement).
//!
//! ## Architectural shape (per plan §3 G14-B)
//!
//! - `UCANBackend<B: GraphBackend>` carries an `Arc<B>` for durable
//!   grant + revocation persistence (Q1 alias-based pragmatic-
//!   genericism: the backend trait surface IS generic-cascaded per
//!   `arch-r1-1` / `D-PHASE-3-1`; the `Engine` boundary stays alias-
//!   based on the `benten-engine` side).
//! - Grants persist as DAG-CBOR-encoded UCAN bytes under the
//!   `g14b:grant:<ucan_cid>` KV key. The CID is a BLAKE3 hash over
//!   the canonical-DAG-CBOR encoding of the full `Ucan` envelope
//!   (claims + signature) — matching Benten's content-address scheme
//!   and giving the durable store a single identity for each token.
//! - Revocations persist as empty markers under `g14b:revoked:<ucan_cid>`
//!   (presence == revoked). Re-opening the backend at the same path
//!   re-observes the marker, so revocation persists across engine
//!   restarts.
//! - Device-revocation entries persist under
//!   `g14b:dev_revoke:<device_did>` and the validate path consults
//!   them before chain-walk acceptance.
//! - Rate-limit-policy plug per D-F + D-PHASE-3-26: an
//!   `Arc<dyn RateLimitPolicy>` on the backend lets `pre_write_with_actor`
//!   route through the configured plug. The default plug is
//!   [`crate::rate_limit::NullRateLimitPolicy`]; concrete impls land
//!   at G14-D / G16.
//!
//! ## Time-window enforcement (`crypto-blocker-2` + CLR-2)
//!
//! The chain-walk path delegates to
//! [`benten_id::ucan::validate_chain_at`] which checks `nbf` / `exp`
//! at EVERY link in the chain — not just the leaf. The durable
//! backend then layers on top a revocation-store lookup so a UCAN
//! that has been explicitly revoked rejects even when otherwise
//! within its `[nbf, exp]` window.
//!
//! ## D-PHASE-3-21 D2 (UCAN-gated `host:atrium:publish_view_result`)
//!
//! Per Ben's 2026-05-05 ratification, view-result replication in
//! Atriums is gated by UCAN delegation of the
//! `host:atrium:publish_view_result` capability — no new
//! trust-policy primitive. The durable backend recognizes the
//! capability string in the chain-walk by virtue of the standard
//! UCAN resource:ability shape (no special-case bypass).
//! Attenuation applies uniformly: a child cannot widen the
//! capability beyond its parent's grant (per UCAN spec).
//!
//! ## ssi re-evaluation (per phase-3-backlog §2.1)
//!
//! G14-B is the named destination for ssi-for-UCAN-external-interop.
//! At impl-time the durable backend continues to use the hand-rolled
//! internal UCAN format defined at
//! [`benten_id::ucan::Ucan`] — Phase-3's must-pass contract is
//! covered without ssi (no external Benten producers exist yet; the
//! Atrium handshake at G16 names ssi as its forward-compat axis if
//! external producer-interop becomes required). Adding ssi as a dep
//! is deferred to the G16 Atrium-handshake landing per
//! `feedback_no_defer_HARD_RULE` clause (b) BELONGS-ELSEWHERE
//! (named destination: `docs/future/phase-3-backlog.md §2.1` "Durable
//! UCAN backend in `benten-id`" — updated NOW with the G16 ssi-
//! re-evaluation pointer).
//!
//! ## Dual durable-grant-store seam (intentional)
//!
//! Two parallel durable-grant-store seams coexist in the engine by
//! design. Each consumer reads its own seam; they are NOT a write-
//! mirror of one another (per mini-review g14b-mr-3 disposition):
//!
//! - **`UCANBackend` raw KV `g14b:grant:<cid>`** (this file). Optimized
//!   for chain-walk lookup: CID-keyed direct-fetch with no Node-decode
//!   cycle. The CID is the content-address of the full UCAN envelope
//!   (claims + signature) so the durable identity for each token is
//!   deterministic across restarts. Used by signed UCAN-typed proofs
//!   whose identity IS their content-CID.
//!
//! - **`GrantReader` Node-encoded `system:CapabilityGrant` zone**
//!   (`crates/benten-caps/src/grant_backed.rs`). Optimized for cap-
//!   recheck delivery-time scans: ChangeSubscriber-driven, used by the
//!   Phase-2a `NoAuthBackend` / direct-policy seam for unsigned
//!   capability-grant Nodes (Phase-3 SUBSCRIBE delivery-time recheck
//!   per §6 G14-D path).
//!
//! Both seams will coexist; the G14-D wave (next, per §6 of the
//! plan) reconciles cross-seam read-during-write semantics where the
//! SUBSCRIBE delivery-time cap-recheck closure consumes both. A
//! future "unify the stores" PR is rejected by reference to this
//! module-doc paragraph: the dual shape is intentional — the two
//! seams have different read-shapes (CID-keyed direct-fetch vs zone-
//! prefix scan) and different write-paths (UCAN envelope persist vs
//! Node-encoded grant write).

use std::sync::Arc;

use benten_core::Cid;
use benten_graph::GraphBackend;
use benten_graph::backend::KVBackend;
use benten_id::device_attestation::DeviceRevocation;
use benten_id::did::Did;
use benten_id::errors::UcanError;
use benten_id::ucan::{
    Ucan, validate_chain_at, validate_chain_for_audience, validate_chain_with_device_revocations,
};
use serde_ipld_dagcbor as cbor;

use crate::error::CapError;
use crate::rate_limit::{NullRateLimitPolicy, RateLimitPolicy};

const KV_GRANT_PREFIX: &[u8] = b"g14b:grant:";
const KV_REVOKED_PREFIX: &[u8] = b"g14b:revoked:";
const KV_DEV_REVOKE_PREFIX: &[u8] = b"g14b:dev_revoke:";

/// Compute the content CID of a UCAN envelope (claims + signature).
///
/// Hashes the DAG-CBOR-encoded full `Ucan` value via BLAKE3, then
/// wraps in the standard Benten CIDv1 layout (multicodec `0x71`
/// dag-cbor, multihash `0x1e` BLAKE3). Matches the content-address
/// scheme used elsewhere in the engine
/// ([`benten_core::Node::cid`]) so a UCAN's identity is consistent
/// with the Node-encoded mirror at `system:Grant`.
fn ucan_cid(ucan: &Ucan) -> Result<Cid, CapError> {
    let bytes = cbor::to_vec(ucan).map_err(|e| CapError::BackendStorage {
        reason: format!("encode UCAN for CID: {e}"),
    })?;
    let digest = blake3::hash(&bytes);
    Ok(Cid::from_blake3_digest(*digest.as_bytes()))
}

fn kv_key(prefix: &[u8], cid: &Cid) -> Vec<u8> {
    let cid_bytes = cid.as_bytes();
    let mut key = Vec::with_capacity(prefix.len() + cid_bytes.len());
    key.extend_from_slice(prefix);
    key.extend_from_slice(cid_bytes);
    key
}

fn dev_revoke_key(device_did: &str) -> Vec<u8> {
    let mut key = Vec::with_capacity(KV_DEV_REVOKE_PREFIX.len() + device_did.len());
    key.extend_from_slice(KV_DEV_REVOKE_PREFIX);
    key.extend_from_slice(device_did.as_bytes());
    key
}

/// Map [`UcanError`] to [`CapError`] for the durable-backend chain-
/// walk surface. Each in-memory chain-walk failure shape lifts to a
/// catalog-typed cap error so the durable-layer caller sees the
/// G14-B `E_CAP_UCAN_*` codes per `docs/ERROR-CATALOG.md`.
fn cap_err_from_ucan(err: UcanError) -> CapError {
    match err {
        UcanError::EmptyChain => CapError::Denied {
            required: String::new(),
            entity: "empty UCAN chain".to_string(),
        },
        UcanError::Expired { exp, now } => CapError::UcanExpired { exp, now },
        UcanError::NotYetValid { nbf, now } => CapError::UcanNotYetValid { nbf, now },
        UcanError::BadSignature { link_index } => CapError::UcanBadSignature { link_index },
        UcanError::ChainLinkBroken { .. } => CapError::Denied {
            required: String::new(),
            entity: "UCAN chain-link integrity violated".to_string(),
        },
        UcanError::AttenuationViolated {
            link_index,
            child_cap,
            ..
        } => CapError::UcanAttenuationViolated {
            link_index,
            child_cap,
        },
        UcanError::AudienceMismatch {
            token_aud,
            expected,
        } => CapError::UcanAudienceMismatch {
            expected,
            actual: token_aud,
        },
        UcanError::IssuerKeypairSuperseded { issuer } => CapError::Denied {
            required: String::new(),
            entity: format!("UCAN issuer keypair superseded: {issuer}"),
        },
        UcanError::IssuerDeviceRevoked { issuer } => CapError::Denied {
            required: String::new(),
            entity: format!("UCAN issuer device revoked: {issuer}"),
        },
        UcanError::DeviceEnvelopeViolated { issuer, cap } => CapError::Denied {
            required: cap,
            entity: format!("UCAN device envelope violated: {issuer}"),
        },
        UcanError::DecodeFailed => CapError::BackendStorage {
            reason: "UCAN decode failure".to_string(),
        },
        UcanError::CapabilityNotGranted {
            required,
            leaf_caps,
        } => CapError::Denied {
            required,
            entity: format!("UCAN leaf does not grant required cap; leaf_caps={leaf_caps:?}"),
        },
    }
}

/// Durable UCAN capability backend (G14-B).
///
/// See module docs for the architectural shape + storage layout.
///
/// `UCANBackend` is intentionally generic over `B: GraphBackend` per
/// `arch-r1-1` / `D-PHASE-3-1` (the backend trait surface is
/// generic-cascaded). The `Engine` consumer alias-binds
/// `Engine = EngineGeneric<RedbBackend>` per Q1 standing rule, so the
/// generic cascade STOPS at the engine boundary.
pub struct UCANBackend<B: GraphBackend> {
    backend: Arc<B>,
    rate_limit: Arc<dyn RateLimitPolicy>,
}

impl<B: GraphBackend> std::fmt::Debug for UCANBackend<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UCANBackend")
            .field("backend", &"<Arc<B: GraphBackend>>")
            .field("rate_limit", &"<Arc<dyn RateLimitPolicy>>")
            .finish()
    }
}

impl<B: GraphBackend> UCANBackend<B> {
    /// Construct a durable UCAN backend over `backend`. The rate-
    /// limit plug defaults to [`NullRateLimitPolicy`] (every check
    /// passes); call [`UCANBackend::with_rate_limit_policy`] to swap
    /// in a configured plug.
    #[must_use]
    pub fn new(backend: Arc<B>) -> Self {
        Self {
            backend,
            rate_limit: Arc::new(NullRateLimitPolicy::new()),
        }
    }

    /// Replace the configured rate-limit policy plug. Builder-style.
    #[must_use]
    pub fn with_rate_limit_policy(mut self, policy: Arc<dyn RateLimitPolicy>) -> Self {
        self.rate_limit = policy;
        self
    }

    /// Borrow the configured rate-limit policy plug. Engine wiring
    /// at the Atrium boundary (G16-A) calls into this directly to
    /// account peer-inbound bandwidth.
    #[must_use]
    pub fn rate_limit_policy(&self) -> &dyn RateLimitPolicy {
        self.rate_limit.as_ref()
    }

    /// Borrow the underlying graph backend (engine wiring + tests).
    #[must_use]
    pub fn graph_backend(&self) -> &B {
        self.backend.as_ref()
    }

    /// Compute and return a UCAN's content CID without persisting.
    ///
    /// # Errors
    ///
    /// Returns [`CapError::BackendStorage`] if the UCAN cannot be
    /// DAG-CBOR encoded (the only failure path on a fixed-shape
    /// envelope is encoder integer-overflow on adversarial input).
    pub fn cid_of(&self, ucan: &Ucan) -> Result<Cid, CapError> {
        ucan_cid(ucan)
    }

    /// Persist a UCAN as a grant in the durable store. Returns the
    /// content CID of the persisted token.
    ///
    /// Storage layout: the UCAN body is written under the
    /// `g14b:grant:<cid>` KV key (encoded via DAG-CBOR). Re-`install_proof`-
    /// ing the same UCAN is idempotent — the KV layer overwrites
    /// with the byte-identical body.
    ///
    /// # Errors
    ///
    /// Returns [`CapError::BackendStorage`] on encode or KV write
    /// failure.
    pub fn install_proof(&self, ucan: &Ucan) -> Result<Cid, CapError> {
        let cid = ucan_cid(ucan)?;
        let bytes = cbor::to_vec(ucan).map_err(|e| CapError::BackendStorage {
            reason: format!("encode UCAN for grant store: {e}"),
        })?;
        let key = kv_key(KV_GRANT_PREFIX, &cid);
        self.backend
            .put(&key, &bytes)
            .map_err(|e| CapError::BackendStorage {
                reason: format!("KV put grant {cid}: {e}"),
            })?;
        Ok(cid)
    }

    /// Persist a UCAN as a grant with explicit `WriteContext`. The
    /// rate-limit policy plug fires for non-privileged contexts (the
    /// engine-privileged path skips the check, consistent with the
    /// [`crate::policy::WriteAuthority::EnginePrivileged`] Inv-13
    /// dispatch precedent).
    ///
    /// # Errors
    ///
    /// Returns [`CapError::RateLimitExceeded`] when the rate-limit
    /// plug rejects, or [`CapError::BackendStorage`] on KV write
    /// failure.
    pub fn record_grant(
        &self,
        ucan: &Ucan,
        ctx: &crate::policy::WriteContext,
    ) -> Result<Cid, CapError> {
        if !ctx.is_privileged
            && let Some(actor) = &ctx.actor_hint
        {
            self.rate_limit.check_writes_per_sec(actor, &ctx.label)?;
        }
        self.install_proof(ucan)
    }

    /// Persist a [`DeviceRevocation`] in the durable store keyed by
    /// the revoked device-DID. Subsequent chain-walks consult this
    /// entry via [`UCANBackend::validate_chain_with_durable_revocations`].
    ///
    /// # Errors
    ///
    /// Returns [`CapError::BackendStorage`] on encode or KV write
    /// failure.
    pub fn record_revocation(
        &self,
        revocation: &DeviceRevocation,
        _ctx: &crate::policy::WriteContext,
    ) -> Result<(), CapError> {
        let bytes = cbor::to_vec(revocation).map_err(|e| CapError::BackendStorage {
            reason: format!("encode device revocation: {e}"),
        })?;
        let key = dev_revoke_key(&revocation.device_did);
        self.backend
            .put(&key, &bytes)
            .map_err(|e| CapError::BackendStorage {
                reason: format!("KV put device revocation {}: {e}", revocation.device_did),
            })?;
        Ok(())
    }

    /// Mark a previously-installed UCAN as revoked. The chain-walk
    /// path consults the marker on every validate call so a revoked
    /// UCAN observably rejects even when otherwise within its
    /// `[nbf, exp]` window. Revocation persists across engine
    /// restarts via the underlying KV.
    ///
    /// # Errors
    ///
    /// Returns [`CapError::BackendStorage`] on KV write failure.
    pub fn revoke(&self, ucan_cid: &Cid) -> Result<(), CapError> {
        let key = kv_key(KV_REVOKED_PREFIX, ucan_cid);
        self.backend
            .put(&key, &[])
            .map_err(|e| CapError::BackendStorage {
                reason: format!("KV put revocation marker {ucan_cid}: {e}"),
            })
    }

    /// Probe whether `ucan_cid` is recorded as revoked.
    ///
    /// # Errors
    ///
    /// Returns [`CapError::BackendStorage`] on KV read failure.
    pub fn is_revoked(&self, ucan_cid: &Cid) -> Result<bool, CapError> {
        let key = kv_key(KV_REVOKED_PREFIX, ucan_cid);
        let opt = self
            .backend
            .get(&key)
            .map_err(|e| CapError::BackendStorage {
                reason: format!("KV get revocation marker {ucan_cid}: {e}"),
            })?;
        Ok(opt.is_some())
    }

    /// Validate a UCAN delegation chain at `now`.
    ///
    /// Composes [`benten_id::ucan::validate_chain_at`] (in-memory
    /// chain integrity + signature + time-window + attenuation) with
    /// a durable revocation lookup. A chain whose leaf or any link
    /// is durable-revoked rejects with [`CapError::Revoked`].
    ///
    /// # Errors
    ///
    /// Returns the typed cap error for the failure mode (see
    /// `docs/ERROR-CATALOG.md` `E_CAP_UCAN_*`).
    pub fn validate_chain_at(&self, chain: &[Ucan], now: u64) -> Result<(), CapError> {
        validate_chain_at(chain, now).map_err(cap_err_from_ucan)?;
        for token in chain {
            let cid = ucan_cid(token)?;
            if self.is_revoked(&cid)? {
                return Err(CapError::Revoked);
            }
        }
        Ok(())
    }

    /// Convenience alias for [`UCANBackend::validate_chain_at`].
    /// Matches the test-pin call shape and reads naturally on the
    /// invocation site (`backend.validate_chain(&[token], now)`).
    ///
    /// # Errors
    ///
    /// Same as [`UCANBackend::validate_chain_at`].
    pub fn validate_chain(&self, chain: &[Ucan], now: u64) -> Result<(), CapError> {
        self.validate_chain_at(chain, now)
    }

    /// Validate a UCAN delegation chain at `now` AND bind the leaf's
    /// audience to `audience`. Pins CLR-2 (audience-binding) at the
    /// durable chain-walk seam: a UCAN issued to atrium-A persisted
    /// in atrium-B's durable store and replayed against atrium-B's
    /// audience rejects with [`CapError::UcanAudienceMismatch`] —
    /// distinct from the generic [`CapError::Denied`] family so
    /// audit pipelines can route on cross-atrium replay independently.
    ///
    /// Composes:
    /// 1. [`benten_id::ucan::validate_chain_for_audience`] — leaf-
    ///    audience binding (constant-time via `subtle::ConstantTimeEq`
    ///    per `crypto-major-4`).
    /// 2. [`benten_id::ucan::validate_chain_at`] — chain integrity +
    ///    signature + `nbf` / `exp` time-window at every link
    ///    (per `crypto-blocker-2` + CLR-2).
    /// 3. Durable per-token revocation lookup (per
    ///    [`UCANBackend::is_revoked`]).
    ///
    /// # Errors
    ///
    /// - [`CapError::UcanAudienceMismatch`] when the leaf's `aud` does
    ///   not match `audience` (cross-atrium replay).
    /// - Other typed cap errors per the chain-walk failure modes
    ///   (see `docs/ERROR-CATALOG.md` `E_CAP_UCAN_*`).
    pub fn validate_chain_for_audience_at(
        &self,
        chain: &[Ucan],
        audience: &Did,
        now: u64,
    ) -> Result<(), CapError> {
        // 1. Audience binding at the leaf — fires
        //    `CapError::UcanAudienceMismatch` BEFORE the time-window
        //    walk so a cross-atrium replay rejects with the typed
        //    audience error rather than (e.g.) an `exp` error if the
        //    chain happens to be expired.
        validate_chain_for_audience(chain, audience).map_err(cap_err_from_ucan)?;
        // 2. Standard chain-walk (signature + time-window at every
        //    link + chain integrity).
        validate_chain_at(chain, now).map_err(cap_err_from_ucan)?;
        // 3. Durable revocation lookup.
        for token in chain {
            let cid = ucan_cid(token)?;
            if self.is_revoked(&cid)? {
                return Err(CapError::Revoked);
            }
        }
        Ok(())
    }

    /// Validate a UCAN chain composing the durable revocation store
    /// with the in-memory device-revocation list. The typed list
    /// (`extra_revocations`) is composed on top of any device-DID
    /// revocations recorded durably under `g14b:dev_revoke:<did>`.
    ///
    /// # Errors
    ///
    /// Returns the typed cap error for the failure mode.
    pub fn validate_chain_with_durable_revocations(
        &self,
        chain: &[Ucan],
        now: u64,
        extra_revocations: &[DeviceRevocation],
    ) -> Result<(), CapError> {
        // 1. Standard chain-walk + time-window + signature.
        validate_chain_at(chain, now).map_err(cap_err_from_ucan)?;
        // 2. Durable per-token revocation.
        for token in chain {
            let cid = ucan_cid(token)?;
            if self.is_revoked(&cid)? {
                return Err(CapError::Revoked);
            }
        }
        // 3. Durable device-revocation lookup composed with the
        //    in-memory list. Reads each token's issuer DID and
        //    probes the durable `g14b:dev_revoke:<did>` entry.
        let mut all_revs: Vec<DeviceRevocation> = extra_revocations.to_vec();
        for token in chain {
            let key = dev_revoke_key(&token.claims.iss);
            if let Some(bytes) = self
                .backend
                .get(&key)
                .map_err(|e| CapError::BackendStorage {
                    reason: format!("KV get device revocation {}: {e}", token.claims.iss),
                })?
            {
                let r: DeviceRevocation =
                    cbor::from_slice(&bytes).map_err(|e| CapError::BackendStorage {
                        reason: format!("decode device revocation {}: {e}", token.claims.iss),
                    })?;
                all_revs.push(r);
            }
        }
        validate_chain_with_device_revocations(chain, &all_revs).map_err(cap_err_from_ucan)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // The `UCANBackend` integration tests live at
    // `crates/benten-caps/tests/ucan_backend.rs` (durable-store
    // chain-walk + revocation-across-restart) and at
    // `crates/benten-caps/tests/prop_ucan_window.rs` (10k proptest at
    // the durable layer). Unit-level smoke for the CID helper is
    // exercised by those integration tests transitively.
}
