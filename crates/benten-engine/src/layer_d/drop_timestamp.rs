//! Layer-D timestamp posture (R0.7 §3.10 / §4.1; M-14; Compromise #62).
//!
//! Three Layer-D timestamp postures:
//!   - [`DropToRecipient`] (Sealed-Sender drop) carries **NO** `sealed_at` /
//!     `valid_until` — drops are forever-valid (#62); freshness rides
//!     `recipient_key_generation` + the nonce-cache, NOT a time field (L6
//!     HIGH-leak finding closed-by-exclusion). The `0x6510`/`0x6610` AAD also
//!     carry NO coarse_epoch (R0.7 §4.1).
//!   - [`DeviceLinkEpoch`] / RemotePermission DO carry a coarse 1-hour bucket
//!     ([`LAYER_D_BUCKET_SECS`]) — the bucket is **Layer-D-ONLY** (M-14).
//!   - The bucket is [`round_down_to_bucket`]: round-DOWN, NO jitter
//!     (NQ-C5 RATIFIED) — deterministic, a clean multiple of 3600.

/// 1-hour bucket width (M-14: DeviceLink + RemotePermission ONLY; NEVER on
/// drops).
pub const LAYER_D_BUCKET_SECS: u64 = 3600;

/// `DropToRecipient` (Sealed-Sender drop) — carries NO timestamp. Drops are
/// forever-valid (#62); freshness rides `recipient_key_generation` + the
/// nonce-cache, NOT a timestamp. The struct has NO `sealed_at`/`valid_until`
/// field BY CONSTRUCTION (L6 closed-by-exclusion). V2 + BE.
#[derive(Clone, Debug)]
pub struct DropToRecipient {
    /// Wire version (V2 from the first commit; M-20).
    pub version: u8,
    /// The recipient key generation freshness rides on (NOT a timestamp).
    pub recipient_key_generation: u32,
    /// The HPKE-wrapped ciphertext (real X-Wing KEM-DEM at Layer-C).
    pub hpke_ciphertext: Vec<u8>,
    // INTENTIONALLY NO sealed_at / valid_until fields. The drop wire-format
    // freezes the field set {version, recipient_key_generation, ciphertext}.
}

impl DropToRecipient {
    /// Canonical wire bytes (V2, BE). The field set is exactly `{version,
    /// recipient_key_generation, hpke_ciphertext}` — NO timestamp. Two drops
    /// built in two distinct wall-clock contexts but with identical generation
    /// + ciphertext serialize to IDENTICAL bytes (the seal-time wall-clock is
    /// never on the wire — M-14 / L6 closed-by-exclusion).
    #[must_use]
    pub fn to_wire_be(&self) -> Vec<u8> {
        let mut b = Vec::new();
        b.push(self.version);
        b.extend_from_slice(&self.recipient_key_generation.to_be_bytes());
        #[allow(clippy::cast_possible_truncation)]
        b.extend_from_slice(&(self.hpke_ciphertext.len() as u32).to_be_bytes());
        b.extend_from_slice(&self.hpke_ciphertext);
        b
    }

    /// The wire field-name set the struct serializes (for the structural
    /// timestamp-exclusion pin). `sealed_at`/`valid_until` MUST NOT appear.
    #[must_use]
    pub fn wire_field_names() -> &'static [&'static str] {
        &["version", "recipient_key_generation", "hpke_ciphertext"]
    }
}

/// DeviceLink + RemotePermission DO carry a coarse 1-hr epoch bucket (U5;
/// Layer-D-ONLY per M-14).
#[derive(Clone, Debug)]
pub struct DeviceLinkEpoch {
    /// Wire version (V2).
    pub version: u8,
    /// The coarse 1-hour bucket of grant time (a clean multiple of 3600).
    pub granted_at_bucket: u64,
}

/// Round a raw unix-seconds time DOWN to the 1-hr bucket — NO jitter (NQ-C5
/// RATIFIED). The result is always a multiple of 3600. Deterministic (same
/// input → same bucket; no randomized jitter — avoids the ≤2×jitter+skew
/// window-widening).
#[must_use]
pub const fn round_down_to_bucket(now_secs: u64) -> u64 {
    (now_secs / LAYER_D_BUCKET_SECS) * LAYER_D_BUCKET_SECS
}
