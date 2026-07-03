//! Layer-D multi-device key-wrap-on-device-link (e2r §7; R0.7 §3.4;
//! Signal-Provisioning precedent).
//!
//! Device B generates a fresh device-encryption keypair (X25519+ML-KEM-768
//! hybrid, reusing the Layer-C HPKE primitive — Inv-16) + a fresh local DAK;
//! displays a [`ProvisioningOffer`] QR. Device A scans, confirms the
//! device-fingerprint, HPKE-encrypts a [`ProvisioningInnerPayload`]
//! `{ k_principal, user_did_signing_key, user_did_pubkey, atrium_memberships,
//! provisioning_session_id, granted_at }` to B's pubkey, signs with the
//! user-DID-signing-key, transmits a [`ProvisioningPayload`] over iroh.
//! Device B HPKE-decrypts, verifies the user-DID signature, stores under its
//! own DAK.
//!
//! # Properties (e2r §7)
//!
//! - HPKE IND-CCA2 **recipient-confidentiality** (the real X-Wing KEM-DEM):
//!   `K_principal` sealed to device B's real pubkey does NOT decrypt under any
//!   OTHER recipient secret — a party holding a different secret (an attacker's,
//!   or a wrong device's) fails the unwrap closed (§10.2 HIGH). This is what the
//!   §10.2-HIGH test actually exercises;
//! - user-DID-signature authentication (a forged/unsigned offer rejects);
//! - **NO forward-secrecy for `K_principal`** (identity-equivalent by design);
//! - session-layer FS + replay defense via `provisioning_session_id` binding.
//!
//! # MITM-substitution defense — HUMAN-FINGERPRINT-dependent, ENFORCEMENT DEFERRED (Row D-30, R10 F-05)
//!
//! The out-of-band device fingerprint ([`ProvisioningOffer::device_b_fingerprint`]
//! + [`fingerprint_recipient`]) is the intended defense against an ACTIVE
//! MITM that swaps B's pubkey in the offer **before A seals** (so A would seal
//! `K_principal` to the attacker's pubkey). At v1-beta this defense is
//! **HUMAN-FINGERPRINT-dependent and its enforcement is NOT wired into the
//! production seal/open path**: `device_b_fingerprint` / `fingerprint_recipient`
//! have **zero production consumers** (they are exercised only by the F-LD-4 test
//! corpus), and nothing in [`seal_provisioning_payload`] binds device-B's
//! human-confirmed identity to the pubkey A actually seals to. The
//! recipient-confidentiality property above is a REAL but WEAKER guarantee — it
//! rejects a substituted pubkey at OPEN time (wrong secret → fail-closed) but
//! does NOT prevent A from sealing to a swapped-BEFORE-seal pubkey. The
//! fingerprint-binding MITM-substitution enforcement is **DEFERRED to G-COMP-1
//! (Phase-4-Meta-Composing device-link UX)** — see Row D-30 in
//! `docs/V1-FROZEN-INTERFACE-DEFERRED.md`. `device_b_fingerprint` /
//! `fingerprint_recipient` are a **RESERVED SEAM** (register-then-enforce, like
//! Row D-64 / D-52) — the genuine planned surface for that deferred defense;
//! they are intentionally retained (do NOT delete) but are inert at v1-beta.
//!
//! # HPKE reuse (Inv-16 / C-2 — and the layer_c-reuse FLAG)
//!
//! The wrap routes through the unified Layer-C/Layer-D HPKE primitive
//! [`benten_crypto_suite::hpke::wrap_key_to_recipient`] (the real
//! X25519⊕ML-KEM-768 X-Wing KEM-DEM at codepoint `0x647a`). The
//! `benten_drop::layer_c` module is a SIBLING wave not yet merged into this
//! base; this module wraps directly against the canary HPKE primitive. **FLAG
//! (eventual layer_c reuse):** once `benten_drop::layer_c` lands, the
//! Layer-C drop assembler and this Layer-D wrap should share the same
//! envelope-assembly helper rather than both calling the crypto-suite
//! primitive independently — a no-wire-change consolidation.

use benten_crypto_suite::cipher_suite::{RecipientPublic, RecipientSecret, WrappedKey};
use benten_crypto_suite::domain_registry::PROVISIONING_DOMAIN;
use benten_crypto_suite::hpke::{unwrap_key_from_recipient, wrap_key_to_recipient};
use benten_id::keypair::{Keypair, PublicKey, Signature};

/// Provisioning wire-format version. M-20: V2 from the first commit.
pub const PROVISIONING_WIRE_VERSION: u8 = 2;

/// DeviceLink band base (R0.7 §4.1 `0x6310..0x631F` FREEZE).
pub const DEVICE_LINK_BAND_BASE: u16 = 0x6310;

/// DeviceLink band end (inclusive).
pub const DEVICE_LINK_BAND_END: u16 = 0x631F;

/// e2r §7.2 — what device B publishes in its QR. B generates a fresh
/// device-encryption pubkey-fingerprint (over the real hybrid recipient
/// pubkey material) + a fresh DAK.
#[derive(Clone, Debug)]
pub struct ProvisioningOffer {
    /// Provisioning wire version (V2).
    pub version: u8,
    /// Device-fingerprint A confirms out-of-band (QR scan) — BLAKE3 over the
    /// recipient public material. The recipient public material itself
    /// travels in the [`ProvisioningPayload`] / is reconstructed by A.
    ///
    /// **RESERVED SEAM (register-then-enforce; Row D-30, R10 F-05).** Zero
    /// production consumers at v1-beta — nothing binds this human-confirmed value
    /// to the pubkey A actually seals to. The active swap-before-seal MITM-
    /// substitution enforcement is DEFERRED to G-COMP-1; this field is the seam
    /// for that wave (inert at v1-beta; do NOT delete).
    pub device_b_fingerprint: [u8; 32],
    /// The provisioning session id (replay + session-FS binding).
    pub provisioning_session_id: [u8; 32],
}

/// e2r §7.2 — the secret payload A HPKE-encrypts to B's pubkey. Carries the
/// identity-equivalent `K_principal` (NO forward-secrecy by design).
///
/// `Debug` is implemented MANUALLY (NOT derived) so the secret key material —
/// `k_principal` (identity-equivalent root key) + `user_did_signing_key` —
/// is REDACTED out of any Debug/log surface (a derived `Debug` would print the
/// raw 32-byte secret). The non-secret fields (pubkey, membership CIDs,
/// session id, time bucket) print normally.
#[derive(Clone, PartialEq, Eq)]
pub struct ProvisioningInnerPayload {
    /// The principal root key (identity-equivalent; no FS).
    pub k_principal: [u8; 32],
    /// The user-DID signing key.
    pub user_did_signing_key: [u8; 32],
    /// The user-DID public key.
    pub user_did_pubkey: [u8; 32],
    /// The Atrium membership-set CIDs B is being granted.
    pub atrium_memberships: Vec<[u8; 32]>,
    /// The provisioning session id (bound into the inner payload).
    pub provisioning_session_id: [u8; 32],
    /// Coarse 1-hour bucket of grant time (Layer-D metadata; M-14).
    pub granted_at_bucket: u64,
}

impl core::fmt::Debug for ProvisioningInnerPayload {
    /// Redacts the secret key material (`k_principal` +
    /// `user_did_signing_key`) so it can never leak through a Debug/log
    /// surface. The non-secret fields print normally.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ProvisioningInnerPayload")
            .field("k_principal", &"[REDACTED]")
            .field("user_did_signing_key", &"[REDACTED]")
            .field("user_did_pubkey", &self.user_did_pubkey)
            .field("atrium_memberships", &self.atrium_memberships)
            .field("provisioning_session_id", &self.provisioning_session_id)
            .field("granted_at_bucket", &self.granted_at_bucket)
            .finish()
    }
}

impl ProvisioningInnerPayload {
    /// Canonical BE bytes for HPKE sealing (M-20; length-injective).
    #[must_use]
    pub fn to_canonical_be(&self) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&self.k_principal);
        b.extend_from_slice(&self.user_did_signing_key);
        b.extend_from_slice(&self.user_did_pubkey);
        #[allow(clippy::cast_possible_truncation)]
        b.extend_from_slice(&(self.atrium_memberships.len() as u32).to_be_bytes());
        for m in &self.atrium_memberships {
            b.extend_from_slice(m);
        }
        b.extend_from_slice(&self.provisioning_session_id);
        b.extend_from_slice(&self.granted_at_bucket.to_be_bytes());
        b
    }
}

/// The signed wire wrapper A transmits (e2r §7.2 `ProvisioningPayload`): the
/// HPKE-wrapped inner + A's user-DID signature over `(session_id ‖
/// wrapped_bytes)`. The signature authenticates the offer — an unsigned /
/// forged offer fails verification.
#[derive(Clone)]
pub struct ProvisioningPayload {
    /// Provisioning wire version (V2).
    pub version: u8,
    /// The provisioning session id.
    pub provisioning_session_id: [u8; 32],
    /// The HPKE-wrapped inner payload (real X-Wing KEM-DEM, Inv-16).
    pub wrapped: WrappedKey,
    /// A's user-DID detached signature over the canonical signing bytes.
    pub user_did_signature: Vec<u8>,
}

/// The canonical signing bytes over which A's user-DID signs the offer:
/// `PROVISIONING_DOMAIN ‖ session_id ‖ wrapped_inner_bytes`. The `wrapped`
/// bytes flattened so a tampered ciphertext breaks the signature.
///
/// The leading [`PROVISIONING_DOMAIN`] domain-separation prefix (C-01) joins
/// this offer signature to the same-key (user-DID Ed25519) domain-separation
/// family enumerated in
/// [`benten_crypto_suite::domain_registry::registered_domain_tags`], so a
/// provisioning-offer signature can NEVER be reinterpreted as any other Benten
/// signature surface (and vice-versa). This prefix is part of the FROZEN
/// provisioning signed-bytes wire: a domain-less (old-format) signature over
/// the same `(session_id ‖ wrapped)` no longer verifies.
#[must_use]
pub fn provisioning_signing_bytes(session_id: &[u8; 32], wrapped: &WrappedKey) -> Vec<u8> {
    let mut b = Vec::new();
    // C-01 domain-separation prefix — the same-key signature family head.
    b.extend_from_slice(PROVISIONING_DOMAIN);
    b.extend_from_slice(session_id);
    // Flatten the wrapped key's wire material into the signed bytes.
    b.extend_from_slice(&wrapped.codepoint.raw().to_be_bytes());
    #[allow(clippy::cast_possible_truncation)]
    b.extend_from_slice(&(wrapped.ek_x.len() as u32).to_be_bytes());
    b.extend_from_slice(&wrapped.ek_x);
    #[allow(clippy::cast_possible_truncation)]
    b.extend_from_slice(&(wrapped.ek_mlkem.len() as u32).to_be_bytes());
    b.extend_from_slice(&wrapped.ek_mlkem);
    b.extend_from_slice(&wrapped.aead_envelope.to_wire_bytes());
    b
}

/// Errors raised on the device-link path.
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future device-link failure
/// mode lands ADDITIVELY without a breaking SemVer bump on the frozen v1 API.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeviceLinkError {
    /// The user-DID signature on the provisioning offer did not verify
    /// (forged / unsigned offer — `E_DEVICE_ATTESTATION_FORGED`-class).
    OfferSignatureForged,
    /// The HPKE unwrap failed (wrong recipient secret — `K_principal` sealed to
    /// B's pubkey does not decrypt under any other secret; recipient-
    /// confidentiality, §10.2 HIGH). NOTE: this fires at OPEN time; it is NOT the
    /// active swap-before-seal MITM defense (that is fingerprint-dependent +
    /// DEFERRED — see the module MITM-substitution doc-block + Row D-30).
    HpkeUnwrapFailed,
    /// The recovered inner payload's session id did not match the offer's
    /// session id.
    SessionIdMismatch,
    /// The provisioning session id was already consumed (replay defense).
    SessionIdReplayed,
}

/// Build a fingerprint over a recipient's public material (BLAKE3). A confirms
/// this out-of-band (QR scan) before sealing.
///
/// **RESERVED SEAM (register-then-enforce; Row D-30, R10 F-05).** This is the
/// planned surface for the active swap-before-seal MITM-substitution defense, but
/// it has **zero production consumers** at v1-beta — no seal/open path binds this
/// fingerprint to the sealed pubkey. Retained as the seam for the G-COMP-1
/// fingerprint-enforcement wave; inert (do NOT delete) at v1-beta.
#[must_use]
pub fn fingerprint_recipient(recipient_pub_bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(recipient_pub_bytes).as_bytes()
}

/// Device A: HPKE-seal the inner payload to device B's recipient pubkey (the
/// real X-Wing KEM-DEM, Inv-16) + sign with the user-DID key.
///
/// # Errors
///
/// Returns [`DeviceLinkError::HpkeUnwrapFailed`] only if the underlying wrap
/// itself fails (a malformed recipient pubkey).
pub fn seal_provisioning_payload(
    user_did: &Keypair,
    recipient_pub: &RecipientPublic,
    session_id: &[u8; 32],
    inner: &ProvisioningInnerPayload,
) -> Result<ProvisioningPayload, DeviceLinkError> {
    let inner_bytes = inner.to_canonical_be();
    let wrapped = wrap_key_to_recipient(recipient_pub, &inner_bytes)
        .map_err(|_| DeviceLinkError::HpkeUnwrapFailed)?;
    let sig = user_did.sign(&provisioning_signing_bytes(session_id, &wrapped));
    Ok(ProvisioningPayload {
        version: PROVISIONING_WIRE_VERSION,
        provisioning_session_id: *session_id,
        wrapped,
        user_did_signature: sig.to_bytes().to_vec(),
    })
}

/// Device B: verify A's user-DID signature, HPKE-unwrap the inner payload with
/// B's recipient secret, and confirm the bound session id. A payload sealed to a
/// DIFFERENT recipient pubkey yields a wrong recipient secret on B's side and the
/// unwrap fails closed (recipient-confidentiality; §10.2 HIGH); a forged offer
/// signature rejects before any unwrap.
///
/// This does NOT enforce the active swap-before-seal MITM-substitution defense:
/// nothing here binds the human-confirmed `device_b_fingerprint` to the pubkey A
/// sealed to, so a MITM that swapped B's pubkey BEFORE A sealed would be opened
/// successfully by the attacker (who holds the matching secret). That
/// fingerprint-binding enforcement is DEFERRED to G-COMP-1 (Row D-30); see the
/// module MITM-substitution doc-block.
///
/// # Errors
///
/// Returns the typed [`DeviceLinkError`] for a forged signature, a failed
/// HPKE unwrap (wrong recipient secret), or a session-id mismatch.
pub fn open_provisioning_payload(
    user_did_pubkey: &PublicKey,
    recipient_sec: &RecipientSecret,
    payload: &ProvisioningPayload,
) -> Result<ProvisioningInnerPayload, DeviceLinkError> {
    // Authenticate FIRST: a forged/unsigned offer rejects before any unwrap.
    let sig_bytes: [u8; 64] = payload
        .user_did_signature
        .as_slice()
        .try_into()
        .map_err(|_| DeviceLinkError::OfferSignatureForged)?;
    let sig = Signature::from_bytes(&sig_bytes);
    let signing_bytes =
        provisioning_signing_bytes(&payload.provisioning_session_id, &payload.wrapped);
    user_did_pubkey
        .verify(&signing_bytes, &sig)
        .map_err(|_| DeviceLinkError::OfferSignatureForged)?;

    // HPKE-unwrap with B's recipient secret. A payload sealed to a DIFFERENT
    // pubkey → wrong recipient secret → fail closed (recipient-confidentiality;
    // NOT the swap-before-seal MITM defense — that is fingerprint-dependent +
    // deferred, Row D-30).
    let recovered = unwrap_key_from_recipient(recipient_sec, &payload.wrapped)
        .map_err(|_| DeviceLinkError::HpkeUnwrapFailed)?;

    let inner = parse_inner_be(&recovered).ok_or(DeviceLinkError::HpkeUnwrapFailed)?;
    if inner.provisioning_session_id != payload.provisioning_session_id {
        return Err(DeviceLinkError::SessionIdMismatch);
    }
    Ok(inner)
}

/// Parse the canonical BE inner-payload bytes (inverse of
/// [`ProvisioningInnerPayload::to_canonical_be`]). Bounded-decode: a
/// `membership_count` that would over-run the buffer returns `None`.
fn parse_inner_be(bytes: &[u8]) -> Option<ProvisioningInnerPayload> {
    let mut off = 0usize;
    let take = |off: &mut usize, n: usize| -> Option<&[u8]> {
        let end = off.checked_add(n)?;
        let s = bytes.get(*off..end)?;
        *off = end;
        Some(s)
    };
    let k_principal: [u8; 32] = take(&mut off, 32)?.try_into().ok()?;
    let user_did_signing_key: [u8; 32] = take(&mut off, 32)?.try_into().ok()?;
    let user_did_pubkey: [u8; 32] = take(&mut off, 32)?.try_into().ok()?;
    let count = u32::from_be_bytes(take(&mut off, 4)?.try_into().ok()?) as usize;
    // Bounded-decode guard: the declared count must fit the remaining buffer.
    let remaining = bytes.len().checked_sub(off)?;
    if count.checked_mul(32)? > remaining {
        return None;
    }
    let mut atrium_memberships = Vec::with_capacity(count);
    for _ in 0..count {
        atrium_memberships.push(take(&mut off, 32)?.try_into().ok()?);
    }
    let provisioning_session_id: [u8; 32] = take(&mut off, 32)?.try_into().ok()?;
    let granted_at_bucket = u64::from_be_bytes(take(&mut off, 8)?.try_into().ok()?);
    Some(ProvisioningInnerPayload {
        k_principal,
        user_did_signing_key,
        user_did_pubkey,
        atrium_memberships,
        provisioning_session_id,
        granted_at_bucket,
    })
}

/// Dispatch a raw DeviceLink codepoint. Integers outside the
/// `0x6310..=0x631F` band typed-reject (fail-closed; CLAUDE.md #5).
///
/// # Errors
///
/// Returns the out-of-band codepoint for any integer outside the band.
pub fn dispatch_device_link_codepoint(cp: u16) -> Result<(), u16> {
    if (DEVICE_LINK_BAND_BASE..=DEVICE_LINK_BAND_END).contains(&cp) {
        Ok(())
    } else {
        Err(cp)
    }
}
