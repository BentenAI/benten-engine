//! UCAN claim envelope + chain-walk validation.
//!
//! ## Scope at G14-A1
//!
//! - In-memory [`Ucan`] envelope: issuer / audience / capabilities /
//!   `nbf` / `exp` / proof chain.
//! - [`Ucan::builder`] / [`UcanBuilder::sign`] for issuance.
//! - [`validate_chain_no_time_check`] / [`validate_chain_at`] /
//!   [`validate_chain_for_audience`] for chain-walk verification.
//! - [`Ucan::validate_at`] single-token entry point (composes with
//!   chain-walk; §11 CLR-2 redundant-distinct shape pins).
//!
//! ## Crypto-blocker-2 BLOCKER + CLR-2 contract
//!
//! `nbf` and `exp` enforcement happens at chain-walk site — EVERY
//! link in the chain is checked, not just the leaf. A child token
//! whose parent has expired rejects even if the child's own `exp` is
//! in the future. Per `crates/benten-id/tests/ucan.rs`, this defends against the
//! "renew the leaf forever" delegation attack.
//!
//! ## Crypto-major-4 contract
//!
//! Signature comparisons go through `ct_signature_eq` (private) which
//! calls `subtle::ConstantTimeEq`. Source-grep audit at
//! `crates/benten-id/tests/ucan.rs::ucan_chain_walk_constant_time_comparison_audit`
//! pins that no naive `==` on signature/audience/proof-CID bytes
//! exists in this file. (Look for the `// const-time-eq` markers in
//! the source.)
//!
//! ## Out of scope at G14-A1 (G14-A2 / G14-B / wave-4b)
//!
//! - `validate_chain_with_revocations` (G14-B durable backend).
//! - `validate_chain_with_attestations` (G14-A2 device-attestation).
//! - `validate_chain_with_rotation_log` (G14-A2 DID rotation).
//! - `DurableBackend` / `RevocationSet` / `AuthoritySet` /
//!   `DelegationError::AudienceEnvelopeIncompatibleWithCapability`.
//!
//! These tests stay `#[ignore]`'d at G14-A1 with rationale strings
//! pointing at the next wave. See `crates/benten-id/tests/ucan.rs` for the full pin
//! catalogue.

use ed25519_dalek::{Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

use crate::did::Did;
use crate::errors::UcanError;
use crate::keypair::{Keypair, PublicKey};

/// Capability grant pair: `(resource, ability)`.
///
/// Example: `Capability::new("/zone/posts", "read")`. The
/// `resource:ability` pair is the unit of attenuation — a child UCAN
/// MUST NOT widen either field beyond what its parent grants.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capability {
    /// Resource path (e.g., `/zone/posts`, `host:sandbox:exec`).
    pub resource: String,
    /// Action / verb (e.g., `read`, `write`, `*`).
    pub ability: String,
}

impl Capability {
    /// Construct a capability from `resource` + `ability` strings.
    pub fn new(resource: impl Into<String>, ability: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            ability: ability.into(),
        }
    }
}

/// Signed UCAN token. The signature is over the canonical-bytes
/// encoding of the [`UcanClaims`] payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ucan {
    /// Inner claims (issuer / audience / capability / nbf / exp /
    /// proof chain).
    pub claims: UcanClaims,
    /// Ed25519 signature of `canonical_bytes(claims)`.
    pub signature: Vec<u8>,
}

/// UCAN claim payload. The `canonical_bytes` encoding is what the
/// signature signs (per `crypto-major-1` shape: signature field
/// excluded from signed bytes).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UcanClaims {
    /// Issuer DID — the keypair that signed this claim.
    pub iss: String,
    /// Audience DID — the recipient this claim is delegated to.
    pub aud: String,
    /// Capabilities granted by this token.
    pub att: Vec<Capability>,
    /// Not-before epoch seconds (per `crypto-blocker-2`).
    pub nbf: Option<u64>,
    /// Expiration epoch seconds (per `crypto-blocker-2`).
    pub exp: Option<u64>,
    /// Proof chain — the parent UCAN (or chain of parents) whose
    /// authority this claim derives from. Empty for the root token.
    /// Walked at `validate_chain_no_time_check` / `validate_chain_at` site.
    pub prf: Vec<Ucan>,
}

impl Ucan {
    /// Begin a new UCAN claim builder.
    pub fn builder() -> UcanBuilder {
        UcanBuilder::default()
    }

    /// Validate this single token at `now` (epoch seconds).
    ///
    /// Composes with [`validate_chain_at`] but at the single-token
    /// entry point — per `crates/benten-id/tests/ucan.rs::ucan_chain_nbf_enforcement` /
    /// `ucan_chain_exp_enforcement`, both entry points converge on
    /// the same `nbf` / `exp` rejection. Does NOT verify signature
    /// (signature verification is a chain-walk-level concern; for a
    /// 1-token "chain", call `validate_chain_at(&[ucan], now)`).
    pub fn validate_at(&self, now: u64) -> Result<(), UcanError> {
        check_time_window(&self.claims, now)
    }

    /// Compute canonical-bytes for the claims payload (signature
    /// input).
    fn canonical_bytes(&self) -> Vec<u8> {
        canonical_bytes(&self.claims)
    }
}

fn canonical_bytes(claims: &UcanClaims) -> Vec<u8> {
    serde_ipld_dagcbor::to_vec(claims)
        .expect("DAG-CBOR encoding of UcanClaims fixed shape cannot fail")
}

fn check_time_window(claims: &UcanClaims, now: u64) -> Result<(), UcanError> {
    if let Some(nbf) = claims.nbf
        && now < nbf
    {
        return Err(UcanError::NotYetValid { nbf, now });
    }
    if let Some(exp) = claims.exp
        && now >= exp
    {
        return Err(UcanError::Expired { exp, now });
    }
    Ok(())
}

/// UCAN claim builder. Construct via [`Ucan::builder`].
#[derive(Default)]
pub struct UcanBuilder {
    iss: Option<String>,
    aud: Option<String>,
    att: Vec<Capability>,
    nbf: Option<u64>,
    exp: Option<u64>,
    prf: Vec<Ucan>,
}

impl UcanBuilder {
    /// Set the issuer DID. Typically `keypair.public_key().to_did()`.
    #[must_use]
    pub fn issuer(mut self, iss: impl Into<String>) -> Self {
        self.iss = Some(iss.into());
        self
    }

    /// Convenience: set issuer from a `Did`.
    #[must_use]
    pub fn issuer_did(self, did: &Did) -> Self {
        self.issuer(did.as_str().to_string())
    }

    /// Set the audience DID.
    #[must_use]
    pub fn audience(mut self, aud: impl Into<String>) -> Self {
        self.aud = Some(aud.into());
        self
    }

    /// Convenience: set audience from a `Did`.
    #[must_use]
    pub fn audience_did(self, did: &Did) -> Self {
        self.audience(did.as_str().to_string())
    }

    /// Add a capability grant.
    #[must_use]
    pub fn capability(mut self, resource: impl Into<String>, ability: impl Into<String>) -> Self {
        self.att.push(Capability::new(resource, ability));
        self
    }

    /// Set the not-before epoch seconds (per `crypto-blocker-2`).
    #[must_use]
    pub fn not_before(mut self, nbf: u64) -> Self {
        self.nbf = Some(nbf);
        self
    }

    /// Set the expiration epoch seconds (per `crypto-blocker-2`).
    #[must_use]
    pub fn expiry(mut self, exp: u64) -> Self {
        self.exp = Some(exp);
        self
    }

    /// Attach a proof token (the parent UCAN that delegated authority
    /// to this issuer).
    #[must_use]
    pub fn proof(mut self, parent: Ucan) -> Self {
        self.prf.push(parent);
        self
    }

    /// Sign the assembled claims with `keypair`. The keypair's public
    /// DID MUST equal the configured `issuer` DID (the chain-walk
    /// validator checks this; we don't enforce here so callers can
    /// build adversarial fixtures for tests).
    pub fn sign(self, keypair: &Keypair) -> Ucan {
        let claims = UcanClaims {
            iss: self.iss.unwrap_or_default(),
            aud: self.aud.unwrap_or_default(),
            att: self.att,
            nbf: self.nbf,
            exp: self.exp,
            prf: self.prf,
        };
        let bytes = canonical_bytes(&claims);
        let sig = keypair.sign(&bytes);
        Ucan {
            claims,
            signature: sig.to_bytes().to_vec(),
        }
    }
}

/// Constant-time signature byte comparison.
///
/// Per `crypto-major-4`, signature / DID / proof-CID byte
/// comparisons go through `subtle::ConstantTimeEq` to defend against
/// timing-side-channel leak of "how many leading bytes match." The
/// source-grep audit at
/// `crates/benten-id/tests/ucan.rs::ucan_chain_walk_constant_time_comparison_audit`
/// pins this site as the only byte-equality entry point.
// const-time-eq: load-bearing — DO NOT replace with naive `==` per crypto-major-4
// Made `pub(crate)` per g14-a2-mr-2 fix-pass so DID-rotation +
// device-attestation security-decision sites use the same helper.
pub(crate) fn ct_signature_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Validate a UCAN delegation chain (no time check).
///
/// The chain ordering is **leaf-first**: `chain[0]` is the leaf
/// (most-recently-issued) token; `chain[1..]` are progressively
/// older parents. Equivalent to [`validate_chain_at`] with `now =
/// u64::MAX` (which never trips `exp`). Use the timed variant for
/// production paths.
pub fn validate_chain_no_time_check(chain: &[Ucan]) -> Result<(), UcanError> {
    // For "no time check", we still want `nbf` / `exp` consistency
    // checks to be skipped — pass `now = 0` to skip nbf only if all
    // tokens have nbf = 0. Better: split the check. We deliberately
    // accept the ambiguity at this entry point and direct callers to
    // `validate_chain_at` for production. The `crates/benten-id/tests/ucan.rs::ucan_chain_validation_basic`
    // pin uses well-formed nbf=now-1, exp=now+3600 tokens, so the
    // ambiguity does not surface; attenuation + signature + chain-link
    // structure are the load-bearing checks here.
    validate_chain_inner(chain, None, None)
}

/// Validate a UCAN delegation chain at a given epoch second.
///
/// Per `crypto-blocker-2` BLOCKER, EVERY link in the chain has its
/// `nbf` and `exp` checked at this `now`. A token presented before
/// any link's `nbf` rejects with [`UcanError::NotYetValid`]; a token
/// presented after any link's `exp` rejects with [`UcanError::Expired`].
pub fn validate_chain_at(chain: &[Ucan], now: u64) -> Result<(), UcanError> {
    validate_chain_inner(chain, Some(now), None)
}

/// Validate a UCAN delegation chain bound to a specific audience DID.
///
/// Per `crates/benten-id/tests/ucan.rs::ucan_audience_binding_prevents_cross_atrium_replay`,
/// a UCAN issued to atrium A replayed at atrium B rejects with
/// [`UcanError::AudienceMismatch`]. Skips `nbf` / `exp` checks
/// (compose with `validate_chain_at` if both gates are needed).
pub fn validate_chain_for_audience(
    chain: &[Ucan],
    expected_audience: &Did,
) -> Result<(), UcanError> {
    validate_chain_inner(chain, None, Some(expected_audience.as_str()))
}

/// Validate a UCAN delegation chain at a given audience for a specific
/// required capability.
///
/// Composes [`validate_chain_for_audience`] + [`validate_chain_at`]
/// + a leaf-claim check that the leaf token's `att` array actually
/// grants the requested `(resource, ability)` capability. The leaf-
/// claim check uses the same subsume relation as
/// [`validate_chain_at`]'s attenuation walk
/// ([`capability_satisfies_requirement`]) so the engine's own
/// internal subsume rule is the SAME relation external callers
/// query.
///
/// Per the typed-CALL `ucan_validate_chain` op: a chain that's
/// structurally sound (audience-bound + signed + in-window +
/// well-attenuated) but whose leaf does NOT name the requested
/// capability MUST reject. Without this gate, a handler asking "does
/// this chain grant `zone:write` to `audience`?" gets `valid: true`
/// regardless of the leaf `att` — a defense-in-depth hole at the
/// heart of the Phase-3 Atrium / UCAN authorization story.
///
/// Required capability format: `"<resource>:<ability>"` where
/// `<ability>` is the LAST `:`-separated segment. Example:
/// `"zone:user:write"` parses to
/// `Capability { resource: "zone:user", ability: "write" }`. The
/// caller MUST pass a string with at least one `:`; an
/// ability-only string (no `:`) returns
/// [`UcanError::CapabilityNotGranted`].
///
/// # Errors
///
/// Returns [`UcanError::AudienceMismatch`] if the leaf is not bound
/// to `expected_audience`; [`UcanError::Expired`] /
/// [`UcanError::NotYetValid`] if any link is out of `now`'s window;
/// [`UcanError::BadSignature`] / [`UcanError::ChainLinkBroken`] /
/// [`UcanError::AttenuationViolated`] from the chain walk;
/// [`UcanError::CapabilityNotGranted`] if the leaf does not name
/// the requested capability.
pub fn validate_chain_for_capability(
    chain: &[Ucan],
    expected_audience: &Did,
    required: &Capability,
    now: u64,
) -> Result<(), UcanError> {
    // Audience binding + chain walk (signature + nbf/exp + chain-link
    // integrity + attenuation) all happen in `validate_chain_inner`.
    validate_chain_inner(chain, Some(now), Some(expected_audience.as_str()))?;

    // Leaf-claim check: the leaf's `att` MUST contain a capability
    // that subsumes the requested one (exact / wildcard-ability /
    // path-prefix-resource per `caps_match_or_subsume`).
    let leaf = chain.first().ok_or(UcanError::EmptyChain)?;
    let granted = leaf
        .claims
        .att
        .iter()
        .any(|granted_cap| caps_match_or_subsume(granted_cap, required));
    if !granted {
        let leaf_caps = leaf
            .claims
            .att
            .iter()
            .map(|c| format!("{}:{}", c.resource, c.ability))
            .collect();
        return Err(UcanError::CapabilityNotGranted {
            required: format!("{}:{}", required.resource, required.ability),
            leaf_caps,
        });
    }
    Ok(())
}

/// Test whether `granted` subsumes `required` per the same subsume
/// relation `validate_chain_at`'s internal attenuation walk uses.
///
/// Public so engine-side code (typed-CALL dispatch in
/// `benten-engine`) can answer "does this chain grant `required`?"
/// using the SAME relation the chain-walk uses internally — single
/// source of truth.
///
/// Subsume relation:
/// - exact match (`resource` AND `ability` equal), OR
/// - `granted.ability` is `*` AND `resource` matches, OR
/// - `granted.resource` is a path-prefix of `required.resource` AND
///   ability matches per the wildcard rule above.
#[must_use]
pub fn capability_satisfies_requirement(granted: &Capability, required: &Capability) -> bool {
    caps_match_or_subsume(granted, required)
}

fn validate_chain_inner(
    chain: &[Ucan],
    now: Option<u64>,
    expected_audience: Option<&str>,
) -> Result<(), UcanError> {
    if chain.is_empty() {
        return Err(UcanError::EmptyChain);
    }

    // Audience-binding check on the leaf (chain[0]) — defends against
    // cross-atrium replay.
    if let Some(expected) = expected_audience {
        let token_aud = &chain[0].claims.aud;
        // const-time-eq: audience-DID comparison goes through ct_eq
        // per crypto-major-4. (Audience strings are not secret but
        // the policy applies uniformly to security-decision compares.)
        if !ct_signature_eq(token_aud.as_bytes(), expected.as_bytes()) {
            return Err(UcanError::AudienceMismatch {
                token_aud: token_aud.clone(),
                expected: expected.to_string(),
            });
        }
    }

    for (idx, token) in chain.iter().enumerate() {
        // 1. Time-window check at every link (crypto-blocker-2 BLOCKER).
        if let Some(now) = now {
            check_time_window(&token.claims, now)?;
        }

        // 2. Signature check at every link (crypto-major-4: comparison
        // is constant-time via subtle).
        let sig_bytes: [u8; 64] = token
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| UcanError::BadSignature { link_index: idx })?;
        let sig = Signature::from_bytes(&sig_bytes);

        // Resolve issuer DID to its public key.
        let iss_did = Did::from_string_unchecked(token.claims.iss.clone());
        let pk: PublicKey = iss_did
            .resolve()
            .map_err(|_| UcanError::BadSignature { link_index: idx })?;
        let bytes = canonical_bytes(&token.claims);
        // ed25519-dalek's verify is itself constant-time on the
        // signature bytes (ed25519 verification has no early-exit on
        // signature mismatch); we still flow the result through a
        // typed error for the chain-walk audit trail.
        if pk.as_verifying_key().verify(&bytes, &sig).is_err() {
            return Err(UcanError::BadSignature { link_index: idx });
        }

        // 3. Chain-link integrity: token's `aud` must equal next
        // token's `iss` (audience-binding within the chain).
        if let Some(parent) = chain.get(idx + 1) {
            // const-time-eq: chain-link aud↔iss comparison
            // per crypto-major-4.
            let aud_bytes = token.claims.aud.as_bytes();
            // The chain ordering is leaf-first, so chain[idx]'s
            // PARENT is chain[idx+1]. The parent's audience MUST
            // equal the child's issuer (the parent delegated TO
            // the child).
            let parent_aud = parent.claims.aud.as_bytes();
            let child_iss = token.claims.iss.as_bytes();
            // Two binding checks compose the chain integrity. First:
            // parent.aud == this.iss (parent delegated TO this issuer).
            if !ct_signature_eq(parent_aud, child_iss) {
                return Err(UcanError::ChainLinkBroken {
                    link_index: idx + 1,
                    aud: parent.claims.aud.clone(),
                    next_iss: token.claims.iss.clone(),
                });
            }
            // Second (defense-in-depth): suppress the unused
            // `aud_bytes` variable warning. (Variable bound for
            // potential future tightening: a four-way check that
            // also pins `child.aud` is downstream, but the parent
            // chain-link check above is the load-bearing assertion
            // here.)
            let _ = aud_bytes;

            // 4. Attenuation check: every capability in the child
            // MUST be a subset of (resource:ability == match OR
            // ability is `*` and parent's covers more) parent's
            // capabilities. Per crypto-blocker-2 + UCAN spec.
            for child_cap in &token.claims.att {
                let widens = !parent
                    .claims
                    .att
                    .iter()
                    .any(|p| caps_match_or_subsume(p, child_cap));
                if widens {
                    return Err(UcanError::AttenuationViolated {
                        link_index: idx + 1,
                        child_cap: format!("{}:{}", child_cap.resource, child_cap.ability),
                        parent_caps: parent
                            .claims
                            .att
                            .iter()
                            .map(|c| format!("{}:{}", c.resource, c.ability))
                            .collect(),
                    });
                }
            }

            // 5. Time-window narrowing (G16-B-B-rest sub-item B/A
            // closure of cap-r4-2 (a)/(b) MAJOR + tcc-r1-5 R3-A): the
            // child's `[nbf, exp]` window MUST be a SUBSET of every
            // ancestor's window. Widening rejects with
            // `AttenuationViolated` (joining the same cap-attenuation
            // family — the time-window axis is a sister attenuation
            // dimension to authority).
            //
            // - `child.nbf < parent.nbf` = child claims earlier
            //   activation than parent allows (backdating attack).
            // - `child.exp > parent.exp` = child claims later expiry
            //   than parent allows (forward-dating attack).
            //
            // Implementation: an absent child bound is treated as
            // unbounded on that axis (nbf=0 / exp=u64::MAX) which the
            // narrowing check then compares against the parent's
            // explicit bound. An absent parent bound is treated as
            // unbounded — the parent did not constrain that axis, so
            // the child cannot widen what was never narrowed.
            let child_nbf = token.claims.nbf.unwrap_or(0);
            let child_exp = token.claims.exp.unwrap_or(u64::MAX);
            let parent_nbf = parent.claims.nbf.unwrap_or(0);
            let parent_exp = parent.claims.exp.unwrap_or(u64::MAX);
            if child_nbf < parent_nbf {
                return Err(UcanError::AttenuationViolated {
                    link_index: idx + 1,
                    child_cap: format!(
                        "(time-window: child.nbf={child_nbf} < parent.nbf={parent_nbf})"
                    ),
                    parent_caps: vec![format!("(time-window: parent allows nbf >= {parent_nbf})")],
                });
            }
            if child_exp > parent_exp {
                return Err(UcanError::AttenuationViolated {
                    link_index: idx + 1,
                    child_cap: format!(
                        "(time-window: child.exp={child_exp} > parent.exp={parent_exp})"
                    ),
                    parent_caps: vec![format!("(time-window: parent allows exp <= {parent_exp})")],
                });
            }
        }
    }

    Ok(())
}

/// Validate a chain against a [`crate::did_rotation::RotationLog`].
///
/// Per `crates/benten-id/tests/did_rotation.rs::superseded_did_cannot_sign_new_ucan_delegations`,
/// any UCAN whose issuer DID has been rotated rejects with
/// [`UcanError::IssuerKeypairSuperseded`] — the chain-walker
/// consults the rotation log so post-rotation UCANs from the OLD
/// keypair are observably rejected even when their signature is
/// structurally valid.
pub fn validate_chain_with_rotation_log(
    chain: &[Ucan],
    log: &crate::did_rotation::RotationLog,
) -> Result<(), UcanError> {
    validate_chain_inner(chain, None, None)?;
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

/// Validate a chain against a list of
/// [`crate::device_attestation::DeviceAttestation`]s. Per
/// `crates/benten-id/tests/device_attestation.rs::device_attestation_consumed_at_ucan_delegation_chain_walk`,
/// rejects with [`UcanError::DeviceEnvelopeViolated`] when a token's
/// issuer has an attestation declaring it cannot exercise the
/// claimed capability (e.g. `host:sandbox:exec` from a
/// `runs_sandbox=false` device).
///
/// **G14-A2 scope (per g14-a2-mr-4 docstring sharpen):** currently
/// enforces the `runs_sandbox` envelope dimension only. Broader
/// envelope-dimension enforcement (`runs_atrium_peer`, `holds_zones`,
/// `online_uptime`) lands at G14-B when atrium-peer + zone caps fully
/// exist. Backlog: `docs/future/phase-3-backlog.md §2.1-followup` for
/// the multi-dimension extension.
pub fn validate_chain_with_attestations(
    chain: &[Ucan],
    attestations: &[crate::device_attestation::DeviceAttestation],
) -> Result<(), UcanError> {
    validate_chain_inner(chain, None, None)?;
    for token in chain {
        for att in attestations {
            // ct-eq per crypto-major-4 UNIFORMITY (g14-a2-mr-2 fix-pass).
            if ct_signature_eq(att.device_did.as_bytes(), token.claims.iss.as_bytes()) {
                for cap in &token.claims.att {
                    if cap.resource.starts_with("host:sandbox:") && !att.envelope.runs_sandbox {
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

/// Subsume rule: parent grants child's capability iff:
/// - exact match (resource AND ability equal), OR
/// - parent's `ability` is `*` and resource matches, OR
/// - parent's `resource` is a prefix of child's resource (path
///   semantics; `/zone/posts` covers `/zone/posts/foo`) AND ability
///   matches per the wildcard rule above.
///
/// The basic-attenuation pin
/// (`crates/benten-id/tests/ucan.rs::ucan_chain_attenuation_rejects_overgrant`)
/// uses only the exact-match path; the prefix + wildcard widening of the
/// match relation is a defensive default that does NOT widen child
/// authority beyond parent's literal grants.
///
/// **Constant-time discipline:** capability `resource` / `ability` strings
/// are not secret per se (they are the cap-system's public schema), but
/// the rule-7 brief commits to ct-eq UNIFORMITY at security-decision sites.
/// All `==` comparisons here go through `ct_signature_eq` so the
/// `ucan_chain_walk_constant_time_comparison_audit` grep test pins this
/// surface (resource / ability are the most-likely-future-drift sites for
/// a contributor who adds a new authority comparison).
fn caps_match_or_subsume(parent: &Capability, child: &Capability) -> bool {
    let parent_res = parent.resource.as_bytes();
    let child_res = child.resource.as_bytes();
    let parent_ab = parent.ability.as_bytes();
    let child_ab = child.ability.as_bytes();
    let star = b"*";
    // Exact match.
    if ct_signature_eq(parent_res, child_res) && ct_signature_eq(parent_ab, child_ab) {
        return true;
    }
    // Wildcard ability.
    if ct_signature_eq(parent_ab, star) && ct_signature_eq(parent_res, child_res) {
        return true;
    }
    // Path-prefix resource + matching/wildcard ability.
    if child.resource.starts_with(&parent.resource)
        && (ct_signature_eq(parent_ab, child_ab) || ct_signature_eq(parent_ab, star))
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_kp() -> Keypair {
        Keypair::generate()
    }

    #[test]
    fn empty_chain_rejects() {
        assert_eq!(
            validate_chain_no_time_check(&[]),
            Err(UcanError::EmptyChain)
        );
    }

    #[test]
    fn single_token_signature_round_trip() {
        let kp = fresh_kp();
        let aud_kp = fresh_kp();
        let now = 1_000_000_000;
        let token = Ucan::builder()
            .issuer(kp.public_key().to_did().as_str())
            .audience(aud_kp.public_key().to_did().as_str())
            .capability("/zone/posts", "read")
            .not_before(now - 1)
            .expiry(now + 3600)
            .sign(&kp);
        validate_chain_at(&[token], now).unwrap();
    }

    #[test]
    fn ct_eq_zero_length() {
        // const-time-eq smoke
        assert!(ct_signature_eq(b"", b""));
    }
}
