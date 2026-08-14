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
//
// Each is a NAMED const because `registered_envelope_codepoints()` below
// literally SCANS THIS FILE'S SOURCE for `pub const <NAME>: u16 = <literal>;`
// declarations (F4-040 / CP-SCAN / C6). Declaring a codepoint here is the ONLY
// step needed to enroll it in the collision + IANA-disjointness scanner — there
// is no second list to keep in sync, which is the whole point: until R6 round #1
// this comment described a scan that did not exist, and the enumerator was a
// hand-written `vec![...]` that a new const could silently miss.
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

// ---------------------------------------------------------------------------
// The registry ENUMERATOR (F4-040 / CP-SCAN / C6).
//
// This was a hand-written `vec![...]` of the consts above while the comment
// heading the allocation table claimed a source-scan enumerated the real minted
// set. The two disagreed, and the disagreement was SILENT: adding a colliding
// `pub const` and omitting it from the vec left all three tests in this module
// GREEN and shipped a wire collision on a permanently-frozen u16 band.
//
// The enumerator is now DERIVED FROM THE SOURCE of this very file — the same
// `include_str!` scanner idiom `conformance.rs::endianness` uses for the M-19
// endianness gate. A newly-minted codepoint cannot be omitted from the scan,
// because the declaration IS the scan input.
// ---------------------------------------------------------------------------

/// This file's own source, embedded at compile time. Scanning it — rather than
/// re-listing the consts by hand — is what makes a new mint unskippable.
const REGISTRY_SOURCE: &str = include_str!("registry.rs");

/// `(parsed declarations, names whose initializer was not a plain literal)`.
type ScannedDeclarations = (Vec<(&'static str, u16)>, Vec<&'static str>);

/// Whether a source line is a comment / doc line (after trimming). Mirrors
/// `conformance.rs::endianness::is_comment_line`: prose ABOUT a declaration is
/// not a declaration, so the worked mutation examples in the pins below cannot
/// smuggle themselves into the scanned set.
fn is_comment_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with("/*") || t.starts_with('*')
}

/// Parse a `u16` const initializer: `0x6100` / `0xFE00` hex, or a decimal
/// literal; `_` separators tolerated.
///
/// `None` for any NON-literal initializer (a computed one). The caller reports
/// those rather than dropping them — a silently-dropped declaration is exactly
/// the hole this module exists to close, and a scanner that shrinks its own
/// input set without saying so is the same failure class one level down.
fn parse_u16_literal(raw: &str) -> Option<u16> {
    let cleaned = raw.trim().trim_end_matches(';').trim().replace('_', "");
    let hex = cleaned
        .strip_prefix("0x")
        .or_else(|| cleaned.strip_prefix("0X"));
    match hex {
        Some(digits) => u16::from_str_radix(digits, 16).ok(),
        None => cleaned.parse::<u16>().ok(),
    }
}

/// Scan [`REGISTRY_SOURCE`] for every `pub const <NAME>: u16 = <literal>;`
/// declaration, in declaration order.
///
/// [`BENTEN_ENVELOPE_RANGE`] is deliberately NOT matched: the type test is the
/// exact string `u16 = `, and `RangeInclusive<u16> = ` does not start with it.
fn scan_u16_const_declarations() -> ScannedDeclarations {
    let mut parsed: Vec<(&'static str, u16)> = Vec::new();
    let mut unparsed: Vec<&'static str> = Vec::new();

    for line in REGISTRY_SOURCE.lines() {
        if is_comment_line(line) {
            continue;
        }
        let Some(rest) = line.trim().strip_prefix("pub const ") else {
            continue;
        };
        let Some((name, tail)) = rest.split_once(": ") else {
            continue;
        };
        let Some(value_src) = tail.strip_prefix("u16 = ") else {
            continue;
        };
        if let Some(value) = parse_u16_literal(value_src) {
            parsed.push((name, value));
        } else {
            unparsed.push(name);
        }
    }

    (parsed, unparsed)
}

/// Enumerate the REAL minted envelope-codepoint integers from the R0.7 §4.0
/// table (the non-collision / IANA-disjoint scanner input).
///
/// The set is DERIVED by scanning this file's own source for `pub const
/// <NAME>: u16 = <literal>;` declarations and keeping those inside
/// [`BENTEN_ENVELOPE_RANGE`]. It is NOT a hand-list (F4-040 / CP-SCAN / C6):
/// minting a codepoint enrolls it in the collision scanner with no second edit,
/// which is what makes the collision gate real rather than decorative.
///
/// The two deliberately out-of-band consts ([`EXPERIMENTAL_BASE`] `0xFE00` and
/// [`EXTENDED_CODEPOINT_ESCAPE`] `0xFFFF`) are scanned but filtered out here —
/// they are escapes, not envelope allocations. Sig codepoints
/// (`0x0001/0x0002/0x0003`) are the separate `0x00xx` namespace and are not
/// declared in this file at all.
///
/// A `: u16 =` declaration with a computed initializer cannot be enrolled;
/// `scan_u16_const_declarations` surfaces it and the
/// `every_u16_const_declaration_parses` pin below fails the build rather than
/// letting the scanned set shrink in silence.
///
/// Declaration order in the source equals the order of the previous hand-list,
/// so the returned sequence is unchanged for existing consumers.
#[must_use]
pub fn registered_envelope_codepoints() -> Vec<u16> {
    let (declared, _unparsed) = scan_u16_const_declarations();
    declared
        .into_iter()
        .map(|(_name, codepoint)| codepoint)
        .filter(|codepoint| BENTEN_ENVELOPE_RANGE.contains(codepoint))
        .collect()
}

/// The IANA HPKE kem/kdf/aead 16-bit registry ranges a Benten envelope
/// codepoint MUST avoid. These are the real IANA HPKE registry allocations
/// (RFC 9180 §7 + the HPKE-PQ WG-stream additions). The correct current
/// allocations (R13 F-08 correction — the prior comment mislabeled the
/// `0x0010..0x0021` DHKEM block as also covering ML-KEM + X25519MLKEM768):
///   - KDF IDs `0x0001..0x0003` (HKDF-SHA256/384/512); AEAD IDs
///     `0x0001..0x0003` + `0xFFFF` export-only (overlap the low band).
///   - KEM IDs: DHKEM `0x0010..0x0020` (RFC 9180 §7.1);
///     **ML-KEM-512/768/1024 = `0x0040..0x0042`** (NOT the DHKEM block).
///   - `0x11EC` = the IANA **TLS Supported Groups** code point for
///     X25519MLKEM768 (a REFERENCED component-algorithm identifier from the
///     IANA TLS registry — NOT an HPKE KEM ID, and NOT a Benten-minted number).
///     Avoided here alongside the HPKE ranges purely for envelope-codepoint
///     disjointness.
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
        0x11EC..=0x11EC, // IANA TLS Supported Groups code point for X25519MLKEM768 (referenced, NOT an HPKE KEM ID, NOT Benten-minted)
    ]
}

/// Whether `codepoints` contains a duplicate (a silent wire collision). The
/// CI scanner (NQ-W2 / Inv-18) is exactly this check over the minted set.
#[cfg(any(test, feature = "testing"))]
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

    /// The exact `(name, value)` inventory the source-scan must find in this
    /// file — ALL 22 `u16` consts, in declaration order, in-band and out.
    ///
    /// On a permanently-frozen band a mint is a deliberate, reviewed TWO-line
    /// edit: the declaration AND this row. That friction is the feature.
    const EXPECTED_DECLARATIONS: &[(&str, u16)] = &[
        ("VAULT_ENVELOPE", 0x6100),
        ("SYMMETRIC_AEAD_12B", 0x6101),
        ("CIPHER_CLASSICAL_X25519", 0x6400),
        ("CIPHER_HYBRID_X25519_MLKEM768", 0x647a),
        ("CIPHER_HYBRID_MLKEM768_HQC", 0x647b),
        ("CIPHER_PURE_PQ_MLKEM768_ONLY", 0x647c),
        ("DEVICE_LINK_BAND_BASE", 0x6310),
        ("REMOTE_PERMISSION_BAND_BASE", 0x6320),
        ("MLS_APPLICATION_BASE", 0x6380),
        ("MLS_WELCOME_BASE", 0x6390),
        ("CGKA_COMMIT_BASE", 0x63A0),
        ("BIRD_OF_PREY_BASE", 0x63B0),
        ("DRAFT_PRABEL_BASE", 0x63C0),
        ("LAYER_C_DROP", 0x6500),
        ("DROP_TO_RECIPIENT_SEALED_SENDER", 0x6510),
        ("LAYER_C_DROP_MULTI_RECIPIENT", 0x6520),
        ("MEMBERSHIP_SET_ENCRYPTION", 0x6600),
        ("MEMBERSHIP_SET_GROUP_MULTI_STANZA", 0x6610),
        ("MEMBERSHIP_SET_SUBSET_REF", 0x6620),
        ("LIFECYCLE_BAND_BASE", 0x6700),
        ("EXPERIMENTAL_BASE", 0xFE00),
        ("EXTENDED_CODEPOINT_ESCAPE", 0xFFFF),
    ];

    /// FAILS ON THIS ONE-LINE MUTATION: add
    /// `pub const SHADOW_DROP: u16 = 0x6510;` anywhere in this file. `0x6510`
    /// is `DROP_TO_RECIPIENT_SEALED_SENDER`, so the source-derived set now
    /// carries a duplicate and `detects_collision` fires.
    ///
    /// That is the whole of CP-SCAN / C6: under the previous hand-written
    /// `vec![...]` enumerator the identical edit was GREEN on all three tests
    /// in this module, and a silent wire collision shipped on a permanently
    /// frozen u16 band.
    #[test]
    fn registry_is_collision_free() {
        let assigned = registered_envelope_codepoints();
        assert!(
            !detects_collision(&assigned),
            "two §4.0 consts share a codepoint — a silent wire collision on a frozen band"
        );
        let mut sorted = assigned.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), assigned.len());
    }

    /// FAILS ON THIS ONE-LINE MUTATION: delete the `.filter(...)` line in
    /// `registered_envelope_codepoints` — `EXPERIMENTAL_BASE` (`0xFE00`) and
    /// `EXTENDED_CODEPOINT_ESCAPE` (`0xFFFF`) then leak into the envelope set
    /// and the band assertion fires on the first of them.
    #[test]
    fn registry_is_in_band_and_iana_disjoint() {
        for cp in registered_envelope_codepoints() {
            assert!(
                BENTEN_ENVELOPE_RANGE.contains(&cp),
                "{cp:#06x} is not in the Benten envelope band 0x6100..=0x6FFF"
            );
            assert!(!in_iana_hpke_range(cp), "{cp:#06x} collides an IANA range");
        }
    }

    /// Negative control on `detects_collision` itself — proves the collision
    /// predicate can say "yes", so `registry_is_collision_free`'s "no" means
    /// something.
    ///
    /// FAILS ON THIS ONE-LINE MUTATION: make `detects_collision` `return false`.
    #[test]
    fn injection_fires_the_scanner() {
        let mut injected = registered_envelope_codepoints();
        injected.push(DROP_TO_RECIPIENT_SEALED_SENDER);
        assert!(detects_collision(&injected));
    }

    /// The freeze pin over the whole §4.0 table: every declaration, its name,
    /// its value, its order.
    ///
    /// FAILS ON THIS ONE-LINE MUTATION: add a NON-colliding const, e.g.
    /// `pub const SHADOW_LIFECYCLE: u16 = 0x6701;`. `registry_is_collision_free`
    /// stays green (no duplicate) — THIS pin is what makes a quiet mint
    /// impossible. Also fails on any rename, deletion, reorder, or value edit.
    #[test]
    fn registry_source_scan_inventory_is_exact() {
        let (declared, _) = scan_u16_const_declarations();
        assert_eq!(
            declared, EXPECTED_DECLARATIONS,
            "the §4.0 allocation table drifted from its frozen inventory. A codepoint mint \
             is a deliberate TWO-line edit on a permanently-frozen band: the `pub const` \
             AND this EXPECTED_DECLARATIONS row. If you are reading this because you added \
             a const, add the row — do not delete the pin."
        );
    }

    /// Proves the enumerator really scans DECLARATIONS rather than re-spelling
    /// the old hand-list: the scan sees the two deliberately out-of-band consts
    /// that `registered_envelope_codepoints()` filters away.
    ///
    /// FAILS ON THIS ONE-LINE MUTATION: replace `scan_u16_const_declarations`'s
    /// body with a hand-list of the 20 in-band consts — the scan then no longer
    /// sees `EXPERIMENTAL_BASE` and the first assertion fires.
    #[test]
    fn scan_sees_declarations_not_the_returned_envelope_set() {
        let (declared, _) = scan_u16_const_declarations();
        let names: Vec<&str> = declared.iter().map(|(n, _)| *n).collect();
        assert!(
            names.contains(&"EXPERIMENTAL_BASE") && names.contains(&"EXTENDED_CODEPOINT_ESCAPE"),
            "the scan must see the out-of-band escapes; it reads declarations, not the \
             filtered envelope set"
        );
        let envelope = registered_envelope_codepoints();
        assert!(
            !envelope.contains(&EXPERIMENTAL_BASE)
                && !envelope.contains(&EXTENDED_CODEPOINT_ESCAPE),
            "the escapes are scanned but MUST NOT be enrolled as envelope allocations"
        );
        assert_eq!(
            envelope.len() + 2,
            declared.len(),
            "exactly the two escapes are filtered out"
        );
    }

    /// A `: u16 =` declaration the parser cannot read would silently shrink the
    /// collision scanner's input — the same class of hole one level down.
    ///
    /// FAILS ON THIS ONE-LINE MUTATION: change any const to a computed
    /// initializer, e.g.
    /// `pub const LAYER_C_DROP: u16 = LAYER_C_DROP_MULTI_RECIPIENT - 0x20;`.
    /// The name lands in `unparsed` and this fires; under a hand-list the same
    /// refactor was invisible.
    #[test]
    fn every_u16_const_declaration_parses() {
        let (_, unparsed) = scan_u16_const_declarations();
        assert!(
            unparsed.is_empty(),
            "these §4.0 consts have non-literal initializers and are therefore NOT enrolled \
             in the collision scanner: {unparsed:?}. Use a plain integer literal — a frozen \
             wire codepoint must be greppable as a literal."
        );
    }

    /// The `registry::*` consts are the SSOT for two values `vault.rs`
    /// re-declares. Nothing tied the pair together, so they could drift.
    ///
    /// FAILS ON THIS ONE-LINE MUTATION: change either
    /// `vault::VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT` or `VAULT_ENVELOPE`.
    #[test]
    fn vault_codepoint_copies_equal_the_registry_ssot() {
        assert_eq!(
            crate::vault::VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT,
            VAULT_ENVELOPE,
            "vault.rs re-declares 0x6100; it MUST equal the §4.0 registry SSOT"
        );
        assert_eq!(
            crate::vault::SYMMETRIC_AEAD_12B_CODEPOINT,
            SYMMETRIC_AEAD_12B,
            "vault.rs re-declares 0x6101; it MUST equal the §4.0 registry SSOT"
        );
    }
}
