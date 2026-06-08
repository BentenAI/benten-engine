//! Typed codepoint enums + dispatch table.
//!
//! Benten owns **only the thin suite-selector codepoint table** — the
//! one-codepoint-per-suite selector (MLS RFC 9420 model, NOT HPKE
//! algorithm-triple) — per `RATIFIED-pq-default-reframe-2026-05-19.md` §3.
//! Component algorithm IDs reference the IANA HPKE/COSE registries
//! (we never mint Benten algorithm numbers).
//!
//! # Wire format note (FROZEN at G-CORE-9 V1-FROZEN-INTERFACE row 6)
//!
//! The codepoint VALUES below are FROZEN at v1-beta per
//! `docs/V1-FROZEN-INTERFACE.md` item 6 (commit `8cc4eddd` + R1 fix-pass
//! commit `235ad861` ratifying the typed-rejection framing for 0x0003 +
//! 0x647c). Codepoint table integer values are PERMANENT per the freeze
//! contract — algorithms behind each codepoint are SWAPPABLE within the
//! framing; the codepoint values themselves are wire-canonical. Future
//! additions land additively at unused codepoint values per the
//! additive-codepoint discipline; existing values are never repurposed.
//! Pinned by
//! `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs::codepoint_table_integer_values_pinned`
//! (G-CORE-9 R1 Bundle 5).

// Re-export so test files that `use benten_crypto_suite::codepoint::UnsupportedAlgorithm`
// (per TF-2 spec) find it under codepoint where the dispatch happens.
pub use crate::error::UnsupportedAlgorithm;

/// Typed signature codepoint enum.
///
/// The v1-beta SHIPPED arms are [`SigCodepoint::HYBRID_ED25519_MLDSA65`]
/// (the DEFAULT — byte-faithful IETF LAMPS composite
/// `id-MLDSA65-Ed25519-SHA512` Ed25519⊕ML-DSA-65; both-must-verify;
/// `mldsaSig||tradSig`, NO commitment trailer) and
/// [`SigCodepoint::CLASSICAL_ED25519`] (the non-default
/// downgrade). The NF-1 end-state arm
/// [`SigCodepoint::HYBRID_MLDSA65_SLHDSA`] is a **reserved swap-matrix arm
/// — typed-rejected by default** at [`SigCodepoint::resolve`] and at
/// `SignatureSuite::resolve_codepoint`; reachable only via
/// `SwapMatrix::try_pure_pq_sole_trust_path` which is gated by the C11b
/// `AUDIT_LANDED_PURE_PQ_FLAG` (compile-time `false` at v1-beta per
/// `SwapMatrixError::AuditNotLandedPurePqRejected`). The full swap
/// matrix shipped at G-CORE-3c retains 0x0003 as typed-rejected (the
/// 0x647c cipher-side mirror is the canonical mate). Any other codepoint
/// surfaces as [`UnsupportedAlgorithm::Signature`] — NEVER a silent fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SigCodepoint(pub(crate) u16);

impl SigCodepoint {
    /// v1-beta DEFAULT: hybrid Ed25519⊕ML-DSA-65, byte-faithful IETF
    /// LAMPS composite `id-MLDSA65-Ed25519-SHA512` (both-must-verify;
    /// `mldsaSig||tradSig`, NO commitment trailer). Byte-faithful to
    /// `draft-ietf-lamps-pq-composite-sigs-19`.
    pub const HYBRID_ED25519_MLDSA65: Self = Self(0x0001);

    /// Non-default downgrade: classical-only Ed25519 (built +
    /// conformance-testable; NOT the default).
    pub const CLASSICAL_ED25519: Self = Self(0x0002);

    /// NF-1 PQ⊕PQ end-state: ML-DSA-65 ⊕ SLH-DSA. **Reserved swap-matrix
    /// arm — typed-rejected by default** at [`Self::resolve`] +
    /// `SignatureSuite::resolve_codepoint` + `varsig.rs::decode_payload`.
    /// G-CORE-3c shipped the full swap matrix; 0x0003 stays typed-rejected
    /// at the dispatcher per the C11b safety gate (reachable only via the
    /// audit-gated `SwapMatrix::try_pure_pq_sole_trust_path` constructor
    /// which itself returns `SwapMatrixError::AuditNotLandedPurePqRejected`
    /// at v1-beta). Dispatching this codepoint at the v1-beta default
    /// surfaces [`UnsupportedAlgorithm::Signature`].
    pub const HYBRID_MLDSA65_SLHDSA: Self = Self(0x0003);

    /// Raw 16-bit codepoint value. **FROZEN at G-CORE-9
    /// V1-FROZEN-INTERFACE row 6** (the integer values are PERMANENT
    /// per V1-FROZEN-INTERFACE.md item 6.2 codepoint table; never reuse
    /// a value for a different algorithm).
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
    #[cfg(any(test, feature = "testing"))]
    pub const fn from_raw_for_test(raw: u16) -> Self {
        Self::from_raw(raw)
    }

    /// Returns a reserved-but-unimplemented codepoint specifically for
    /// testing the additive-codepoint discipline (NF-1 end-state /
    /// future signature codepoints).
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
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

/// The lifecycle state of a codepoint (R0.5 §4.1 U16; the
/// `0x6700..0x67FF` lifecycle band carries the per-codepoint state).
///
/// `Live`/`Deprecated` codepoints dispatch (a deprecated codepoint still
/// decodes existing content — old-codepoints-supported-forever); a
/// `Quarantined` or `Burned` codepoint MUST be typed-rejected — a burned
/// codepoint is permanently un-dispatchable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodepointLifecycle {
    /// Actively dispatched.
    Live,
    /// Deprecated but still decodable (no new content; existing content still
    /// opens).
    Deprecated,
    /// Quarantined — suspended pending a security review; rejected.
    Quarantined,
    /// Burned — permanently un-dispatchable; rejected forever.
    Burned,
}

impl CodepointLifecycle {
    /// Dispatch by lifecycle state. `Live`/`Deprecated` are `Ok`;
    /// `Quarantined`/`Burned` are typed-rejected.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedAlgorithm::CipherSuite`] (codepoint `0`) for a
    /// `Quarantined`/`Burned` state — the caller threads the real codepoint
    /// at the call site; the lifecycle gate's contract is the state-reject.
    pub fn dispatch(self) -> Result<(), UnsupportedAlgorithm> {
        match self {
            Self::Live | Self::Deprecated => Ok(()),
            Self::Quarantined | Self::Burned => {
                Err(UnsupportedAlgorithm::CipherSuite { codepoint: 0 })
            }
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
    #[cfg(any(test, feature = "testing"))]
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
/// vendored combiner over `ml-kem` + `x25519-dalek` + `sha3`).
/// **G-CORE-3a CANARY flips `0x647a` + `0x6400` (classical-X25519
/// downgrade arm) to LIVE.** `0x647b` (NF-1 ML-KEM-768⊕HQC end-state)
/// + `0x647c` (pure-PQ ML-KEM-768-only swap-matrix arm; reserved-named
/// at G-CORE-3c but typed-reject at the v1-beta default `CipherSuite`
/// dispatcher level — the pure-PQ arm is only constructible via
/// [`crate::swap_matrix::SwapMatrix::try_pure_pq_sole_trust_path`]
/// which gates on `AUDIT_LANDED_PURE_PQ_FLAG`)
/// + `0x0000` (no-encryption) remain reserved-typed-reject via
/// [`UnsupportedAlgorithm`] until G-CORE-3c's full swap-matrix wave —
/// the additive-codepoint discipline + old-codepoints-supported-forever
/// invariant hold across the partial-light step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CipherSuiteCodepoint(pub(crate) u16);

impl CipherSuiteCodepoint {
    /// v1-beta DEFAULT for #1301: X25519⊕ML-KEM-768 hybrid KEM at
    /// codepoint `0x647a` + ChaCha20-Poly1305 bulk. **LIVE at G-CORE-3a
    /// CANARY** (X-Wing-style combiner over `ml-kem` + `x25519-dalek`
    /// + `sha3`; conformance test at G-CORE-3 #1301 — see plan §3
    /// G-CORE-3 + RATIFIED §1 + §6).
    pub const HYBRID_X25519_MLKEM768: Self = Self(0x647a);

    /// Non-PQ downgrade: X25519-only KEM + ChaCha20-Poly1305 bulk.
    /// **LIVE at G-CORE-3a CANARY** — classical-only X25519 downgrade
    /// arm of the hybrid swap matrix; full swap-matrix conformance
    /// (incl. `0x647b` + `0x0000`) lands at G-CORE-3c.
    pub const CLASSICAL_X25519: Self = Self(0x6400);

    /// No-encryption (plaintext partition) downgrade. Reserved at
    /// G-CORE-2 / live at G-CORE-3c.
    pub const NONE_PLAINTEXT: Self = Self(0x0000);

    /// NF-1 KEM PQ⊕PQ end-state: ML-KEM-768 ⊕ HQC. **Reserved-but-
    /// unimplemented** — build-trigger = FIPS 207 (HQC-KEM) final
    /// (NIST-projected 2027); draft (~early-2026) is the early-warning.
    pub const HYBRID_MLKEM768_HQC: Self = Self(0x647b);

    /// Pure-PQ ML-KEM-768-only swap-matrix arm (the
    /// `EncryptionArm::PurePqMlKem768Only` destination — module-private
    /// internal variant; not an intra-doc link —
    /// classical X25519 dropped). **Reserved-named** at the cipher-suite
    /// dispatcher level — the v1-beta default [`crate::cipher_suite::CipherSuite::resolve`]
    /// arm typed-rejects `0x647c` per the C11b safety gate (pure-PQ is
    /// only constructible via [`crate::swap_matrix::SwapMatrix::try_pure_pq_sole_trust_path`]
    /// which gates on `AUDIT_LANDED_PURE_PQ_FLAG`). Distinct from
    /// [`Self::HYBRID_MLKEM768_HQC`] (`0x647b`) which is strictly reserved
    /// for the future ML-KEM⊕HQC PQ⊕PQ end-state arm. **Pre-G-CORE-9-FREEZE
    /// 2026-05-24 ratification** — minted as a NEW codepoint to prevent
    /// wire-format collision when the AUDIT_LANDED flag flips at v1-GM.
    pub const PURE_PQ_MLKEM768_ONLY: Self = Self(0x647c);

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
    #[cfg(any(test, feature = "testing"))]
    pub const fn from_raw_for_test(raw: u16) -> Self {
        Self::from_raw(raw)
    }

    /// Resolve cipher-suite codepoint into a typed dispatch outcome.
    ///
    /// **G-CORE-3a flips `0x647a` (X25519⊕ML-KEM-768 hybrid) + `0x6400`
    /// (classical-only X25519 downgrade) to LIVE.** The full swap matrix
    /// (incl. `0x0000` no-encryption + `0x647b` NF-1 PQ⊕PQ end-state + the
    /// pure-PQ-ML-KEM-only arm at `0x647c`) is G-CORE-3c's deliverable;
    /// here `0x0000` + `0x647b` + `0x647c` remain typed-rejected at the
    /// [`crate::cipher_suite::CipherSuite`] dispatcher level per the
    /// additive-codepoint discipline + C11b safety gate (pure-PQ is only
    /// reachable via the named `SwapMatrix::try_pure_pq_sole_trust_path`
    /// constructor which gates on `AUDIT_LANDED_PURE_PQ_FLAG`).
    pub fn resolve(self) -> Result<(), UnsupportedAlgorithm> {
        match self.0 {
            // G-CORE-3a LIVE arms.
            0x647a | 0x6400 => Ok(()),
            // Reserved-but-unimplemented at the cipher-suite dispatcher
            // level (G-CORE-3c lights the full swap matrix). `0x647c`
            // pure-PQ is reachable ONLY via SwapMatrix; `0x647b` is the
            // future ML-KEM⊕HQC end-state.
            0x647b => Err(UnsupportedAlgorithm::CipherSuite { codepoint: self.0 }),
            0x647c => Err(UnsupportedAlgorithm::CipherSuite { codepoint: self.0 }),
            0x0000 => Err(UnsupportedAlgorithm::CipherSuite { codepoint: self.0 }),
            other => Err(UnsupportedAlgorithm::CipherSuite { codepoint: other }),
        }
    }
}

// ---------------------------------------------------------------------------
// F-full CODEPOINT-RESERVE set (NQ-A1 / NQ-W5 / GAP-6b).
// ---------------------------------------------------------------------------

/// The F-full reserved-codepoint set (NQ-A1 conservative-fallback: post-freeze
/// high/critical findings → reserve-codepoints + new tag, NEVER a silent
/// wire-break). Each slot is RESERVED at v1-beta and typed-rejected by
/// [`ReservedCodepoint::resolve`]; it becomes LIVE additively at its
/// §4.0-named band, never via a wire-format break.
///
/// - [`Self::ExecuteWorkflow`] — RemotePermission `0x6320..0x632F` band.
/// - [`Self::SubsetRef`] — `MEMBERSHIP_SET_SUBSET_REF = 0x6620`.
/// - [`Self::RecoveryArtifact`] — codepoint RESERVED at Core (NQ-W5/m-14);
///   the `RecoveryHook` trait is NOT frozen at Core — it lands in
///   Phase-4-Meta-Composing alongside the allocated codepoint.
/// - [`Self::RotatingGroupKeyChainedMode`] — CGKA-Commit FS-future bracket
///   (`CGKA_COMMIT_BASE == 0x63A0`; §4.0).
/// - [`Self::ChainedStateTlv`] — the per-stanza `Option<ChainedStateTlv>`
///   codepoint-reserve sub-slot (GAP-6b), AAD-bound when present (see
///   [`chained_state_tlv_aad_binding`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ReservedCodepoint {
    /// `ExecuteWorkflow` reserve (RemotePermission `0x6320..0x632F` band).
    ExecuteWorkflow,
    /// `SubsetRef` federation reserve (`0x6620`).
    SubsetRef,
    /// `RecoveryArtifact` reserve (codepoint at Core; trait in Composing).
    RecoveryArtifact,
    /// `RotatingGroupKeyChainedMode` reserve (CGKA-Commit FS-future bracket,
    /// `CGKA_COMMIT_BASE == 0x63A0`).
    RotatingGroupKeyChainedMode,
    /// `ChainedStateTlv` per-stanza sub-slot reserve (GAP-6b; AAD-bound).
    ChainedStateTlv,
}

impl ReservedCodepoint {
    /// The §4.0 band base integer this reserve dispatches from (the
    /// `RecoveryArtifact` conceptual reserve has no Core integer — it returns
    /// `None`).
    #[must_use]
    pub const fn band_base(self) -> Option<u16> {
        match self {
            Self::ExecuteWorkflow => Some(0x6320),
            Self::SubsetRef => Some(0x6620),
            // `RotatingGroupKeyChainedMode` + `ChainedStateTlv` dispatch from
            // the §4.0 CGKA-Commit FS-future bracket (`CGKA_COMMIT_BASE ==
            // 0x63A0`), NOT the MLS-Application bracket `0x6380`.
            Self::RotatingGroupKeyChainedMode | Self::ChainedStateTlv => Some(0x63A0),
            Self::RecoveryArtifact => None,
        }
    }

    /// Resolve a reserved codepoint — ALWAYS typed-rejected at v1-beta (NEVER
    /// a silent accept). The reserve becomes LIVE additively at v1-GM+.
    ///
    /// # Errors
    ///
    /// Always returns [`UnsupportedAlgorithm::CipherSuite`] with the band base
    /// codepoint (or `0` for the conceptual `RecoveryArtifact` reserve).
    pub fn resolve(self) -> Result<(), UnsupportedAlgorithm> {
        Err(UnsupportedAlgorithm::CipherSuite {
            codepoint: self.band_base().unwrap_or(0),
        })
    }
}

/// GAP-6b — the per-stanza `Option<ChainedStateTlv>` sub-slot is AAD-BOUND
/// when present, so a present-vs-absent flip is detectable at decrypt (not
/// advisory). This helper appends the optional `ChainedStateTlv` codepoint
/// reserve into the AAD byte string (big-endian): a present sub-slot pushes
/// `0x01 ‖ band_base_be`; an absent sub-slot pushes `0x00`. Binding it into
/// the AAD means a relay that strips it fails AEAD-open (it cannot be silently
/// removed).
#[must_use]
pub fn chained_state_tlv_aad_binding(present: bool) -> Vec<u8> {
    let mut aad = Vec::new();
    if present {
        aad.push(0x01u8);
        // The ChainedStateTlv reserve dispatches from the CGKA-Commit FS-future
        // band base (`CGKA_COMMIT_BASE == 0x63A0`; §4.0).
        aad.extend_from_slice(
            &ReservedCodepoint::ChainedStateTlv
                .band_base()
                .unwrap_or(0)
                .to_be_bytes(),
        );
    } else {
        aad.push(0x00u8);
    }
    aad
}
