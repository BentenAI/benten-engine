//! Encryption class taxonomy — §8-CC public surface.
//!
//! Distinguishes between public-readable content (no confidentiality
//! envelope) and confidential content (encrypted per the #1301 substrate;
//! reader must hold the appropriate `AuthorizationGrant.key_material`
//! defined in `benten-caps`).
//!
//! **Crate placement (`benten-core`).** Vocabulary-of-encryption-state
//! type, NOT a capability-policy type. Same crate as [`crate::WriteAuthority`]
//! + [`crate::ChangeEvent`] — both `benten-graph` and `benten-caps`
//! re-export from the same source-of-truth, avoiding cross-crate newtype
//! proliferation.
//!
//! **Frozen at G-CORE-9 V1-FROZEN-INTERFACE row 3 / §1.A.FROZEN item
//! 15(e)** — two-variant baseline (`Public` + `Confidential`) per
//! RATIFIED-S&C §R6. Reserved future arms (`AnonymousGroup`,
//! `PrivateLocal`) are NOT built at v1-beta; explicit-add via additive
//! enum variants post-v1.
//!
//! Wire codepoint table (Public = 0x00, Confidential = 0x01) is part
//! of the freeze per item 4 P-III decision; unknown codepoints surface
//! a typed [`EncryptionClassError::UnsupportedClass`] via [`EncryptionClass::from_codepoint`]
//! (fail-closed; the same typed-reject discipline as crypto-agility
//! per CLAUDE.md baked-in #5).

use serde::{Deserialize, Serialize};

/// Encryption class for content stored under the per-DID partition.
///
/// **`#[non_exhaustive]`** per V1-FROZEN-INTERFACE item 15(e): reserved
/// future variants (`AnonymousGroup`, `PrivateLocal`) are documented
/// but NOT built at v1-beta; explicit-add via additive enum variants
/// post-v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EncryptionClass {
    /// Public — content visible to any reader with the CID; no
    /// confidentiality envelope.
    Public,
    /// Confidential — content encrypted per #1301 substrate; reader
    /// must hold the appropriate `benten_caps::AuthorizationGrant`'s
    /// `key_material`.
    Confidential,
    // Reserved future arms (NOT built at v1-beta; documented for
    // forward planning):
    //   AnonymousGroup,   // group-keyed without revealing principal identity
    //   PrivateLocal,     // device-local-only; never sync-eligible
}

/// Wire codepoints for [`EncryptionClass`]. Each variant maps to a
/// stable single-byte wire codepoint per V1-FROZEN-INTERFACE item 4
/// (D2 v1-canonical-bytes contract).
///
/// Permanent: a future agent MUST NOT reuse a codepoint for a
/// different class (the #1341 0x647b incident established this
/// discipline). New classes land additively at unused values.
pub mod codepoint {
    /// Public class wire codepoint.
    pub const PUBLIC: u8 = 0x00;
    /// Confidential class wire codepoint.
    pub const CONFIDENTIAL: u8 = 0x01;
    // Reserved unused: 0x02, 0x03, ... — future additive arms only.
}

impl EncryptionClass {
    /// Return the stable wire codepoint for this class.
    #[must_use]
    pub const fn codepoint(self) -> u8 {
        match self {
            Self::Public => codepoint::PUBLIC,
            Self::Confidential => codepoint::CONFIDENTIAL,
        }
    }

    /// Construct from a wire codepoint. Unknown codepoints return
    /// [`EncryptionClassError::UnsupportedClass`] — fail-closed
    /// typed-reject (per CLAUDE.md baked-in #5 crypto-agility framing
    /// extended to the encryption-class dispatch).
    ///
    /// # Errors
    ///
    /// Returns [`EncryptionClassError::UnsupportedClass`] for any
    /// codepoint not in the LIVE table. Future codepoints for reserved
    /// arms (`AnonymousGroup`, `PrivateLocal`) MUST land additively
    /// before this method admits them.
    pub fn from_codepoint(codepoint: u8) -> Result<Self, EncryptionClassError> {
        match codepoint {
            self::codepoint::PUBLIC => Ok(Self::Public),
            self::codepoint::CONFIDENTIAL => Ok(Self::Confidential),
            raw => Err(EncryptionClassError::UnsupportedClass { raw }),
        }
    }
}

/// Error variants for [`EncryptionClass`] codepoint dispatch.
///
/// `#[non_exhaustive]` per V1-FROZEN-INTERFACE item 11.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum EncryptionClassError {
    /// Codepoint does not map to any LIVE encryption class. The wire
    /// envelope MUST fail-closed (typed reject; never silent fallback)
    /// per CLAUDE.md baked-in #5 + V1-FROZEN-INTERFACE item 14
    /// typed-reject discipline.
    #[error("unsupported encryption class codepoint: 0x{raw:02x}")]
    UnsupportedClass {
        /// The raw codepoint byte that did not resolve.
        raw: u8,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_round_trips_through_codepoint() {
        let class = EncryptionClass::Public;
        assert_eq!(class.codepoint(), codepoint::PUBLIC);
        assert_eq!(
            EncryptionClass::from_codepoint(class.codepoint()).unwrap(),
            class
        );
    }

    #[test]
    fn confidential_round_trips_through_codepoint() {
        let class = EncryptionClass::Confidential;
        assert_eq!(class.codepoint(), codepoint::CONFIDENTIAL);
        assert_eq!(
            EncryptionClass::from_codepoint(class.codepoint()).unwrap(),
            class
        );
    }

    #[test]
    fn unknown_codepoint_fails_closed_typed_reject() {
        // Every codepoint that doesn't map to a LIVE class MUST surface
        // a typed reject — NEVER silent fallback. V1-FROZEN-INTERFACE
        // item 14 typed-reject discipline.
        for raw in [0x02_u8, 0x03, 0x10, 0xFF] {
            match EncryptionClass::from_codepoint(raw) {
                Err(EncryptionClassError::UnsupportedClass { raw: r }) => {
                    assert_eq!(r, raw, "unknown codepoint preserved in error");
                }
                other => panic!(
                    "expected UnsupportedClass typed-reject for codepoint 0x{raw:02x}, \
                     got {other:?} — would-FAIL if dispatch silently fell back"
                ),
            }
        }
    }

    #[test]
    fn codepoint_table_no_collisions_with_reserved_arms() {
        // The two LIVE codepoints occupy 0x00 + 0x01. Reserved future
        // arms MUST land at unused values; this test pins the freeze
        // boundary.
        assert_ne!(codepoint::PUBLIC, codepoint::CONFIDENTIAL);
        assert_eq!(codepoint::PUBLIC, 0x00);
        assert_eq!(codepoint::CONFIDENTIAL, 0x01);
    }
}
