//! Typed codepoint enums + dispatch table.
//!
//! Benten owns **only the thin suite-selector codepoint table** — the
//! one-codepoint-per-suite selector (MLS RFC 9420 model, NOT HPKE
//! algorithm-triple) — per `RATIFIED-pq-default-reframe-2026-05-19.md` §3.
//! Component algorithm IDs reference the IANA HPKE/COSE registries
//! (we never mint Benten algorithm numbers).
//!
//! # Wire format note (P-III deferred)
//!
//! The codepoint VALUES below are the operative constants the canary
//! brief carries and that the integration crate treats as authoritative.
//! The **FREEZE ACT** (making these wire-canonical) is the scheduled
//! G-CORE-9 P-III Ben decision-point — this wave produces the codepoint
//! dispatch shape; G-CORE-9 locks the values. They are stable for the
//! span of this crate's life today; G-CORE-9 may re-numerate them in a
//! single deliberate freeze pass (no consumers outside the integration
//! crate depend on the literal value — they call the typed enum).

// Re-export so test files that `use benten_crypto_suite::codepoint::UnsupportedAlgorithm`
// (per TF-2 spec) find it under codepoint where the dispatch happens.
pub use crate::error::UnsupportedAlgorithm;

/// Typed signature codepoint enum.
///
/// The v1-beta SHIPPED arms are [`SigCodepoint::HYBRID_ED25519_MLDSA65`]
/// (the DEFAULT — NF-4 concatenated/committing/strip-resistant Ed25519⊕
/// ML-DSA-65) and [`SigCodepoint::CLASSICAL_ED25519`] (the non-default
/// downgrade). The NF-1 end-state arm
/// [`SigCodepoint::HYBRID_MLDSA65_SLHDSA`] is a reserved-but-unimplemented
/// codepoint (live impl lands at G-CORE-3c). Any other codepoint surfaces
/// as [`UnsupportedAlgorithm::Signature`] — NEVER a silent fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SigCodepoint(pub(crate) u16);

impl SigCodepoint {
    /// v1-beta DEFAULT: hybrid Ed25519⊕ML-DSA-65 (NF-4
    /// concatenated/committing/strip-resistant). Aligned with
    /// `draft-ietf-lamps-pq-composite-sigs-18`.
    pub const HYBRID_ED25519_MLDSA65: Self = Self(0x0001);

    /// Non-default downgrade: classical-only Ed25519 (built +
    /// conformance-testable; NOT the default).
    pub const CLASSICAL_ED25519: Self = Self(0x0002);

    /// NF-1 PQ⊕PQ end-state: ML-DSA-65 ⊕ SLH-DSA. **Reserved-but-
    /// unimplemented** at this wave; live impl + conformance-test at
    /// G-CORE-3c (the full swap matrix). Dispatching this codepoint at
    /// G-CORE-2 surfaces [`UnsupportedAlgorithm::Signature`].
    pub const HYBRID_MLDSA65_SLHDSA: Self = Self(0x0003);

    /// Raw 16-bit codepoint value (P-III deferred; G-CORE-9 freezes the
    /// canonical wire form).
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Construct from a raw 16-bit codepoint. Used by deserializers /
    /// adversarial tests. **NOT a guarantee the codepoint is supported** —
    /// the dispatch is what surfaces typed-unsupported.
    #[must_use]
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// Test-only alias for [`Self::from_raw`] — used by TF-2 adversarial
    /// pins to drive arbitrary codepoints into the dispatch and assert
    /// the typed-unsupported arm fires.
    #[must_use]
    pub const fn from_raw_for_test(raw: u16) -> Self {
        Self::from_raw(raw)
    }

    /// Returns a reserved-but-unimplemented codepoint specifically for
    /// testing the additive-codepoint discipline (NF-1 end-state /
    /// future signature codepoints).
    #[must_use]
    pub const fn reserved_unimplemented_for_test() -> Self {
        // 0x00FE is in the reserved-but-unimplemented range.
        Self(0x00FE)
    }

    /// True iff this codepoint is the v1-beta hybrid default.
    #[must_use]
    pub const fn is_hybrid_default(self) -> bool {
        self.0 == Self::HYBRID_ED25519_MLDSA65.0
    }

    /// True iff this codepoint is the classical-only downgrade.
    #[must_use]
    pub const fn is_classical_only(self) -> bool {
        self.0 == Self::CLASSICAL_ED25519.0
    }

    /// Resolve a codepoint into a typed dispatch outcome — returning
    /// `Ok(())` for supported arms and `Err(UnsupportedAlgorithm)` for
    /// unknown / reserved-unimplemented. **Never a silent fallback.**
    pub fn resolve(self) -> Result<(), UnsupportedAlgorithm> {
        match self.0 {
            // Supported live arms.
            0x0001 | 0x0002 => Ok(()),
            // NF-1 reserved-but-unimplemented (lights at G-CORE-3c).
            0x0003 => Err(UnsupportedAlgorithm::Signature { codepoint: self.0 }),
            // Anything else — typed reject.
            other => Err(UnsupportedAlgorithm::Signature { codepoint: other }),
        }
    }
}

/// Typed hash codepoint enum (multihash codepoints; CLAUDE.md baked-in #5).
///
/// v1 default = BLAKE3 (`0x1e`). Pre-blessed agile fallbacks =
/// SHA-512/256 (`0x1015`) + SHA3-256 (`0x16`). Hash is PQ-UNAFFECTED.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HashCodepoint(pub(crate) u64);

impl HashCodepoint {
    /// v1 default — BLAKE3-256, multihash code `0x1e`.
    pub const BLAKE3: Self = Self(0x1e);

    /// Pre-blessed agile fallback — SHA-512/256 (FIPS), multihash code `0x1015`.
    pub const SHA2_512_256: Self = Self(0x1015);

    /// Pre-blessed agile fallback — SHA3-256, multihash code `0x16`.
    pub const SHA3_256: Self = Self(0x16);

    /// Raw multihash codepoint.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Construct from raw multihash codepoint (adversarial-test driver).
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Test-only alias for [`Self::from_raw`].
    #[must_use]
    pub const fn from_raw_for_test(raw: u64) -> Self {
        Self::from_raw(raw)
    }

    /// Resolve a hash codepoint into a typed dispatch outcome.
    pub fn resolve(self) -> Result<(), UnsupportedAlgorithm> {
        match self.0 {
            0x1e | 0x1015 | 0x16 => Ok(()),
            other => Err(UnsupportedAlgorithm::Hash { codepoint: other }),
        }
    }
}

/// Cipher-suite codepoint (G-CORE-3 / #1301 surface).
///
/// `HYBRID_X25519_MLKEM768` at `0x647a` is the IETF HPKE-PQ WG-stream
/// `MLKEM768-X25519` hybrid-KEM codepoint (IANA-requested; X-Wing-style
/// vendored combiner over `ml-kem` + `x25519-dalek` + `sha3`). The live
/// impl lands at G-CORE-3 #1301; at this wave the codepoint dispatch
/// typed-rejects with the same [`UnsupportedAlgorithm`] envelope so the
/// codepoint reservation is structurally honored (additive-codepoint
/// discipline; old-codepoints-supported-forever invariant).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CipherSuiteCodepoint(pub(crate) u16);

impl CipherSuiteCodepoint {
    /// v1-beta DEFAULT for #1301: X25519⊕ML-KEM-768 hybrid KEM at
    /// codepoint `0x647a` + ChaCha20-Poly1305 bulk. **Reserved-but-
    /// unimplemented at G-CORE-2** (live impl + conformance test at
    /// G-CORE-3 #1301 — see plan §3 G-CORE-3 + RATIFIED §1 + §6).
    pub const HYBRID_X25519_MLKEM768: Self = Self(0x647a);

    /// Non-PQ downgrade: X25519-only KEM + ChaCha20-Poly1305 bulk.
    /// Reserved at G-CORE-2 / live at G-CORE-3c (full swap matrix).
    pub const CLASSICAL_X25519: Self = Self(0x6400);

    /// No-encryption (plaintext partition) downgrade. Reserved at
    /// G-CORE-2 / live at G-CORE-3c.
    pub const NONE_PLAINTEXT: Self = Self(0x0000);

    /// NF-1 KEM PQ⊕PQ end-state: ML-KEM-768 ⊕ HQC. **Reserved-but-
    /// unimplemented** — build-trigger = FIPS 207 (HQC-KEM) final
    /// (NIST-projected 2027); draft (~early-2026) is the early-warning.
    pub const HYBRID_MLKEM768_HQC: Self = Self(0x647b);

    /// Raw 16-bit codepoint.
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Construct from raw codepoint.
    #[must_use]
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// Test-only alias for [`Self::from_raw`].
    #[must_use]
    pub const fn from_raw_for_test(raw: u16) -> Self {
        Self::from_raw(raw)
    }

    /// Resolve cipher-suite codepoint into a typed dispatch outcome.
    ///
    /// **In this wave (G-CORE-2) every cipher-suite codepoint is
    /// reserved-but-unimplemented and surfaces typed-unsupported.**
    /// G-CORE-3 will flip 0x647a from typed-unsupported to live.
    pub fn resolve(self) -> Result<(), UnsupportedAlgorithm> {
        match self.0 {
            0x647a => Err(UnsupportedAlgorithm::CipherSuite { codepoint: self.0 }),
            0x647b => Err(UnsupportedAlgorithm::CipherSuite { codepoint: self.0 }),
            0x6400 | 0x0000 => Err(UnsupportedAlgorithm::CipherSuite { codepoint: self.0 }),
            other => Err(UnsupportedAlgorithm::CipherSuite { codepoint: other }),
        }
    }
}
