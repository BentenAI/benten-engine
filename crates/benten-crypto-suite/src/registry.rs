//! The codepoint registry — the canonical R0.7 §4.0 envelope-codepoint
//! allocation table + the non-collision / IANA-disjoint scanner (F-CP-2 /
//! NQ-W2 / Inv-18).
//!
//! Benten owns the thin envelope-codepoint band `0x6100..=0x6FFF`; the
//! signature codepoints live in the separate `0x00xx` namespace. Component
//! algorithm IDs reference the IANA HPKE / COSE registries — Benten never
//! mints algorithm numbers, and no Benten envelope codepoint may land in an
//! IANA HPKE registry range.

use std::ops::RangeInclusive;

// ---------------------------------------------------------------------------
// R0.7 §4.0 envelope-codepoint allocation table (the single source of truth).
// Each is a NAMED const so a source-scan / registry iterator enumerates the
// REAL minted set (F4-040: not a hand-list inside the scanner).
// ---------------------------------------------------------------------------

/// Layer-A vault envelope (24-byte `SymmetricAeadXNonce`).
pub const VAULT_ENVELOPE: u16 = 0x6100;
/// Layer-A 12-byte `SymmetricAead` (ChaCha20-Poly1305) sibling.
pub const SYMMETRIC_AEAD_12B: u16 = 0x6101;
/// Cipher classical-only X25519 downgrade.
pub const CIPHER_CLASSICAL_X25519: u16 = 0x6400;
/// Cipher hybrid default (X-Wing X25519⊕ML-KEM-768).
pub const CIPHER_HYBRID_X25519_MLKEM768: u16 = 0x647a;
/// Cipher NF-1 PQ⊕PQ (ML-KEM-768⊕HQC; reserved).
pub const CIPHER_HYBRID_MLKEM768_HQC: u16 = 0x647b;
/// Cipher pure-PQ swap-matrix arm (reserved; audit-gated).
pub const CIPHER_PURE_PQ_MLKEM768_ONLY: u16 = 0x647c;
/// Layer-D DeviceLink band base (`0x6310..0x631F`).
pub const DEVICE_LINK_BAND_BASE: u16 = 0x6310;
/// Layer-D RemotePermission band base (`0x6320..0x632F`).
pub const REMOTE_PERMISSION_BAND_BASE: u16 = 0x6320;
/// MLS-Application bracket (NOT MembershipSet).
pub const MLS_APPLICATION_BASE: u16 = 0x6380;
/// MLS-Welcome bracket (NOT Sealed-Sender).
pub const MLS_WELCOME_BASE: u16 = 0x6390;
/// CGKA-Commit FS-future bracket.
pub const CGKA_COMMIT_BASE: u16 = 0x63A0;
/// Bird-of-Prey AKEM FS-future bracket.
pub const BIRD_OF_PREY_BASE: u16 = 0x63B0;
/// draft-prabel FS-future bracket.
pub const DRAFT_PRABEL_BASE: u16 = 0x63C0;
/// Layer-C plaintext-sender drop (non-default sibling).
pub const LAYER_C_DROP: u16 = 0x6500;
/// Layer-C Sealed-Sender DEFAULT (the ONE canonical value; BR-1 / §0.4).
pub const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = 0x6510;
/// Layer-C group multi-stanza (the blinded 8-field set's codepoint).
pub const LAYER_C_DROP_MULTI_RECIPIENT: u16 = 0x6520;
/// MembershipSet set-keying envelope (RELOCATED from M-CONS-FINAL `0x6380`).
pub const MEMBERSHIP_SET_ENCRYPTION: u16 = 0x6600;
/// MembershipSet group multi-stanza.
pub const MEMBERSHIP_SET_GROUP_MULTI_STANZA: u16 = 0x6610;
/// MembershipSet federation acquisition (reserve-only; refused at v1-beta).
pub const MEMBERSHIP_SET_SUBSET_REF: u16 = 0x6620;
/// Lifecycle / revocation band base (`0x6700..0x67FF`).
pub const LIFECYCLE_BAND_BASE: u16 = 0x6700;
/// Experimental-range base (`0xFE00..0xFFFE`; outside the envelope band).
pub const EXPERIMENTAL_BASE: u16 = 0xFE00;
/// Extended-codepoint escape (`0xFFFF`; outside the envelope band).
pub const EXTENDED_CODEPOINT_ESCAPE: u16 = 0xFFFF;

/// The Benten envelope band (`0x6100..=0x6FFF`).
pub const BENTEN_ENVELOPE_RANGE: RangeInclusive<u16> = 0x6100..=0x6FFF;

/// Enumerate the REAL minted envelope-codepoint integers from the R0.7 §4.0
/// table (the non-collision / IANA-disjoint scanner input).
///
/// This is the registry iterator (F4-040): every in-band envelope const above
/// is listed here, so the scanner operates over the authoritative minted set.
/// Sig codepoints (`0x0001/0x0002/0x0003`) are the separate `0x00xx`
/// namespace and are deliberately excluded from this envelope set.
#[must_use]
pub fn registered_envelope_codepoints() -> Vec<u16> {
    vec![
        VAULT_ENVELOPE,
        SYMMETRIC_AEAD_12B,
        CIPHER_CLASSICAL_X25519,
        CIPHER_HYBRID_X25519_MLKEM768,
        CIPHER_HYBRID_MLKEM768_HQC,
        CIPHER_PURE_PQ_MLKEM768_ONLY,
        DEVICE_LINK_BAND_BASE,
        REMOTE_PERMISSION_BAND_BASE,
        MLS_APPLICATION_BASE,
        MLS_WELCOME_BASE,
        CGKA_COMMIT_BASE,
        BIRD_OF_PREY_BASE,
        DRAFT_PRABEL_BASE,
        LAYER_C_DROP,
        DROP_TO_RECIPIENT_SEALED_SENDER,
        LAYER_C_DROP_MULTI_RECIPIENT,
        MEMBERSHIP_SET_ENCRYPTION,
        MEMBERSHIP_SET_GROUP_MULTI_STANZA,
        MEMBERSHIP_SET_SUBSET_REF,
        LIFECYCLE_BAND_BASE,
    ]
}

/// The IANA HPKE kem/kdf/aead 16-bit registry ranges a Benten envelope
/// codepoint MUST avoid. These are the real IANA HPKE registry allocations
/// (RFC 9180 §7 + the HPKE-PQ WG-stream additions). The correct current
/// allocations (R13 F-08 correction — the prior comment mislabeled the
/// `0x0010..0x0021` DHKEM block as also covering ML-KEM + X25519MLKEM768):
///   - KDF IDs `0x0001..0x0003` (HKDF-SHA256/384/512); AEAD IDs
///     `0x0001..0x0003` + `0xFFFF` export-only (overlap the low band).
///   - KEM IDs: DHKEM `0x0010..0x0020` (RFC 9180 §7.1);
///     **ML-KEM-512/768/1024 = `0x0040..0x0042`** (NOT the DHKEM block);
///     **X25519MLKEM768 = `0x11EC`** (the concrete hybrid KEM ID; NOT in the
///     DHKEM block either).
/// The Benten band `0x6100..` is disjoint from ALL of these by construction
/// (the `0x6100+` band floor sits above every IANA allocation above), so
/// disjointness holds regardless — but the ranges are now accurate.
///
/// Two further IANA HPKE reference points exist for completeness but are
/// intentionally NOT coalesced into the scanner ranges below (disjointness with
/// the `0x6100+` band holds for them anyway): `0x0021` DHKEM(X448, HKDF-SHA512)
/// (RFC 9180 §7.1 — one above the DHKEM `0x0010..0x0020` block scanned here) and
/// `0xFFFF` AEAD Export-only (RFC 9180 §7.3). `0xFFFF` is also Benten's own
/// `EXTENDED_CODEPOINT_ESCAPE` (an out-of-band escape, deliberately outside the
/// suite-selector band), which is why it is not treated as an IANA-avoid range.
/// See `docs/CRYPTO-CODEPOINTS.md` "IANA HPKE referenced ranges" for the full
/// reference table.
#[must_use]
pub fn iana_hpke_reserved_ranges() -> Vec<RangeInclusive<u16>> {
    vec![
        0x0001..=0x0003, // KDF IDs (HKDF-SHA256/384/512) + AEAD IDs (overlap low band)
        0x0010..=0x0020, // KEM IDs — DHKEM (RFC 9180 §7.1)
        0x0040..=0x0042, // KEM IDs — ML-KEM-512/768/1024 (real IANA allocation)
        0x11EC..=0x11EC, // KEM ID — X25519MLKEM768 (the concrete hybrid KEM)
    ]
}

/// Whether `codepoints` contains a duplicate (a silent wire collision). The
/// CI scanner (NQ-W2 / Inv-18) is exactly this check over the minted set.
#[must_use]
pub fn detects_collision(codepoints: &[u16]) -> bool {
    let mut seen = std::collections::HashSet::new();
    for cp in codepoints {
        if !seen.insert(*cp) {
            return true;
        }
    }
    false
}

/// Whether `codepoint` lands in any IANA HPKE reserved range.
#[must_use]
pub fn in_iana_hpke_range(codepoint: u16) -> bool {
    iana_hpke_reserved_ranges()
        .iter()
        .any(|r| r.contains(&codepoint))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_collision_free() {
        let assigned = registered_envelope_codepoints();
        assert!(!detects_collision(&assigned));
        let mut sorted = assigned.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), assigned.len());
    }

    #[test]
    fn registry_is_in_band_and_iana_disjoint() {
        for cp in registered_envelope_codepoints() {
            assert!(BENTEN_ENVELOPE_RANGE.contains(&cp));
            assert!(!in_iana_hpke_range(cp));
        }
    }

    #[test]
    fn injection_fires_the_scanner() {
        let mut injected = registered_envelope_codepoints();
        injected.push(DROP_TO_RECIPIENT_SEALED_SENDER);
        assert!(detects_collision(&injected));
    }
}
