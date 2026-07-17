//! `did:key` DID generation + resolution per W3C did-method-key spec.
//!
//! ## Crypto-minor-3 contract
//!
//! - **Multibase prefix `z`** = base58btc.
//! - **Multicodec `0xed01`** = Ed25519 public key (varint of `0xed`).
//! - **Form**: `did:key:z<base58btc(0xed01 || <32 pubkey bytes>)>`.
//!
//! Per the W3C spec at <https://w3c-ccg.github.io/did-method-key/>,
//! this encoding is byte-stable across spec-conformant implementations
//! (didkit / ssi / our crate all produce byte-identical strings for
//! the same pubkey).
//!
//! ## PQ-hybrid did:key (NQ-C4 / U15)
//!
//! The v1-beta default signature key is the LAMPS Composite
//! Ed25519⊕ML-DSA-65 hybrid. Its `did:key` uses the **two-registered-
//! component-multikey** form — `varint(0x1211) ‖ mldsaPK(1952) ‖
//! varint(0xed) ‖ tradPK(32)`, ML-DSA-FIRST — so every component algorithm
//! ID references a REGISTERED multiformats multicodec
//! ([`MLDSA65_PUB_MULTICODEC`] = `0x1211` + [`ED25519_MULTICODEC`] = `0xed`)
//! rather than a Benten-private number (CLAUDE.md baked-in #5). See
//! [`Did::from_hybrid_public_key`] / [`Did::resolve_hybrid`]. The single-byte
//! [`HYBRID_SIG_MULTICODEC`] / [`HYBRID_KEM_MULTICODEC`] private values
//! (the G-CORE-9 NQ-C4 interim, reserved before registered codes existed) are
//! retained as documented fallback-only and are #5-risky.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::errors::DidError;
use crate::keypair::PublicKey;

/// Multicodec varint prefix for Ed25519 public keys.
///
/// Per W3C did-method-key spec + multicodec table:
/// <https://github.com/multiformats/multicodec/blob/master/table.csv>
/// — `ed25519-pub` = `0xed`, varint-encoded as `0xed 0x01`.
pub const ED25519_MULTICODEC: [u8; 2] = [0xed, 0x01];

/// Multicodec varint prefix for the **ML-DSA-65** public-key COMPONENT —
/// the registered `mldsa-65-pub = 0x1211`, unsigned-varint-encoded as
/// `[0x91, 0x24]`. Per the multicodec table:
/// <https://github.com/multiformats/multicodec/blob/master/table.csv>
///
/// This is a REGISTERED multiformats value (NOT a Benten-private number),
/// so it satisfies CLAUDE.md baked-in #5 ("component algorithm IDs reference
/// the multiformats/IANA registry — never a Benten-private number"). It is
/// the FIRST of the two component multikeys the v1 hybrid `did:key`
/// concatenates (ML-DSA FIRST, consistent with the LAMPS composite pubkey
/// serialization `mldsaPK(1952) ‖ tradPK(32)`).
pub const MLDSA65_PUB_MULTICODEC: [u8; 2] = [0x91, 0x24];

/// Length (bytes) of the ML-DSA-65 public-key component the hybrid `did:key`
/// carries — sourced from the upstream `benten_crypto_suite` size witness
/// (NOT a hardcoded redefinition; CLAUDE.md baked-in #5 "never hardcode
/// key/sig/ciphertext sizes"). FIPS-204 Category-3 reference dimension is
/// 1952 B.
fn mldsa65_pubkey_len() -> usize {
    benten_crypto_suite::sizes::ml_dsa_65_pubkey_len()
}

/// Length (bytes) of the Ed25519 public-key component (the W3C `did:key`
/// Ed25519 body is always 32 B; sourced from the upstream
/// `ed25519_dalek::PUBLIC_KEY_LENGTH` constant via the crypto-suite re-export
/// — NOT a Benten redefinition).
const ED25519_PUBKEY_LEN: usize = 32;

/// Multicodec varint prefix for the **PQ-hybrid signature** public key
/// (Ed25519⊕ML-DSA-65 — the v1-beta LAMPS Composite default).
///
/// **FALLBACK-ONLY interim (NQ-C4 / §5.D-9 — RESOLVED).** This single-byte
/// private value (`0xef`, varint `[0xef, 0x01]`) was reserved at G-CORE-9
/// (the NQ-C4 reserved-private interim) when **no registered multiformats
/// code existed** for the hybrid shape. Registered COMPONENT codes now DO
/// exist ([`MLDSA65_PUB_MULTICODEC`] = `0x1211` + [`ED25519_MULTICODEC`] =
/// `0xed`), so the v1 hybrid `did:key` encoding ([`Did::from_hybrid_public_key`]
/// / [`Did::resolve_hybrid`]) uses the **two-component-multikey form**
/// (`varint(0x1211) ‖ mldsaPK ‖ varint(0xed) ‖ tradPK`) — #5-clean, no
/// invented number. This `0xef` const is **RETAINED as documented
/// fallback-only**; it is #5-RISKY (a single-byte squat over the registered
/// single-byte multicodec range) and is NOT the v1 wire encoding. See
/// `docs/CRYPTO-CODEPOINTS.md` (NQ-C4 section).
pub const HYBRID_SIG_MULTICODEC: [u8; 2] = [0xef, 0x01];

/// Multicodec varint prefix for the **PQ-hybrid KEM** public key
/// (X25519⊕ML-KEM-768).
///
/// **FALLBACK-ONLY interim (NQ-C4 / §5.D-9 — RESOLVED).** Same status as
/// [`HYBRID_SIG_MULTICODEC`]: the single-byte private value (`0xf0`, varint
/// `[0xf0, 0x01]`) was the G-CORE-9 reserved-private interim. The KEM hybrid
/// `did:key` (when wired) uses the two-registered-component-multikey form
/// (the registered multiformats `mlkem-768-pub` = `0x120c` component code +
/// `x25519-pub`), so this const is RETAINED as documented fallback-only and is
/// #5-RISKY (single-byte squat). See `docs/CRYPTO-CODEPOINTS.md` NQ-C4.
pub const HYBRID_KEM_MULTICODEC: [u8; 2] = [0xf0, 0x01];

/// `did:key` URI prefix (literal string the W3C spec mandates).
pub const DID_KEY_PREFIX: &str = "did:key:z";

/// Hard upper bound on the length (bytes) of a `did:key` STRING accepted
/// by [`Did::resolve`] / [`Did::resolve_hybrid`] before any base58btc
/// decode runs.
///
/// **Pre-auth DoS defense (F2).** `bs58::decode` is O(N²) in the body
/// length, and the live UCAN chain validators resolve the
/// attacker-controlled `iss` DID of every chain link BEFORE verifying
/// its signature (`crate::ucan::validate_chain_inner`). Without a cap, a
/// multi-MB junk `iss` × up to [`crate::ucan::MAX_UCAN_PROOF_DEPTH`]
/// links is quadratic-CPU exhaustion reachable pre-signature-check.
///
/// A valid PQ-hybrid `did:key` (the largest legitimate shape — the
/// two-component ML-DSA-65 ‖ Ed25519 multikey) base58btc-encodes a
/// ~2 KB body to ≈2.7 KB of string; 4096 gives ample headroom, so NO
/// well-formed DID's resolution outcome changes. The cap only rejects
/// input that is already far larger than any structurally-valid DID.
pub const MAX_DID_KEY_STRING_LEN: usize = 4096;

/// `did:agent:` — an OPTIONAL allowlist alias method (NQ-C4 + Inv-22). The
/// principal's NATURE is DERIVED via method-parse; the `did:agent:` alias is
/// an OPTIONAL allowlist hint, **never a stored authoritative discriminator**
/// and never authority-bearing. It does NOT grant authority — it is a naming
/// convenience that an allowlist may recognize, nothing more.
pub const DID_AGENT_PREFIX: &str = "did:agent:";

/// `did:key` DID — wrapper around the resolved string.
///
/// Construct via [`Did::from_public_key`] (forward path) or
/// [`Did::resolve`] (reverse-direction round-trip; consumers that
/// receive a DID string from the wire validate-then-resolve).
///
/// `Serialize` + `Deserialize` impls round-trip the resolved string
/// form (canonical-bytes-symmetric with the rest of the engine when
/// flowed through DAG-CBOR). Phase-3 G16-A wave-6 wired these for the
/// `benten-sync` handshake wire-format struct
/// (`crates/benten-sync/src/handshake_wire.rs::HandshakeFrame`) which
/// requires both peer-DID + device-DID at the wire-format level per
/// `net-blocker-4` BLOCKER. Deserialization does NOT validate the
/// `did:key:z` prefix or pubkey bytes — callers that need
/// validate-on-deserialize call [`Did::resolve`] explicitly to
/// surface a typed [`DidError`].
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Did(String);

impl Did {
    /// Encode an Ed25519 public key as a `did:key` DID.
    ///
    /// Per `crypto-minor-3`, the encoding is:
    ///
    /// ```text
    /// "did:key:z" + base58btc(0xed 0x01 || <32 pubkey bytes>)
    /// ```
    pub fn from_public_key(pk: &PublicKey) -> Self {
        let pk_bytes = pk.to_bytes();
        let mut payload = Vec::with_capacity(2 + 32);
        payload.extend_from_slice(&ED25519_MULTICODEC);
        payload.extend_from_slice(&pk_bytes);
        let body = bs58::encode(&payload).into_string();
        Self(format!("{DID_KEY_PREFIX}{body}"))
    }

    /// Borrow the resolved string (e.g. for serialization or display).
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Pre-decode length gate shared by [`Did::resolve`] +
    /// [`Did::resolve_hybrid`] (F2 pre-auth DoS defense).
    ///
    /// Rejects an over-long DID string with a typed
    /// [`DidError::BodyTooLong`] BEFORE the O(N²) `bs58::decode` runs, so
    /// an attacker-controlled `iss` cannot inflict quadratic-CPU cost on
    /// the pre-signature-check DID resolve that the UCAN chain walker
    /// performs per link. Nothing structurally valid exceeds
    /// [`MAX_DID_KEY_STRING_LEN`], so no valid resolution changes.
    fn length_pre_check(&self) -> Result<(), DidError> {
        if self.0.len() > MAX_DID_KEY_STRING_LEN {
            return Err(DidError::BodyTooLong {
                got: self.0.len(),
                max: MAX_DID_KEY_STRING_LEN,
            });
        }
        Ok(())
    }

    /// Resolve a `did:key` string back to its underlying public key.
    ///
    /// Round-trip property (per
    /// `crates/benten-id/tests/prop_did_key.rs::prop_did_key_round_trip_byte_identity`):
    /// for any 32-byte sequence, encoding then resolving recovers the
    /// exact bytes — NO bit can be silently dropped or rewritten by
    /// the encode → decode path.
    pub fn resolve(&self) -> Result<PublicKey, DidError> {
        // F2: bound the input length BEFORE the O(N²) base58btc decode
        // so an attacker-controlled `iss` cannot inflict quadratic CPU
        // pre-signature-check.
        self.length_pre_check()?;

        let body = self
            .0
            .strip_prefix(DID_KEY_PREFIX)
            .ok_or_else(|| DidError::InvalidPrefix(self.0.clone()))?;

        let decoded = bs58::decode(body)
            .into_vec()
            .map_err(|_| DidError::Base58Decode)?;

        if decoded.len() < 2 + 32 {
            return Err(DidError::BodyTooShort {
                got: decoded.len(),
                min: 2 + 32,
            });
        }

        if decoded[0] != ED25519_MULTICODEC[0] || decoded[1] != ED25519_MULTICODEC[1] {
            return Err(DidError::UnknownMulticodec(decoded[0], decoded[1]));
        }

        let mut pk_bytes = [0u8; 32];
        pk_bytes.copy_from_slice(&decoded[2..2 + 32]);

        PublicKey::from_bytes(&pk_bytes).ok_or(DidError::InvalidPublicKey)
    }

    /// Encode a **PQ-hybrid** verifying key (Ed25519⊕ML-DSA-65 — the v1-beta
    /// LAMPS Composite default signature key) as a `did:key` DID using the
    /// **two-registered-component-multikey** form.
    ///
    /// The multibase body is:
    ///
    /// ```text
    /// "did:key:z" + base58btc(
    ///     varint(0x1211) || mldsaPK(1952) || varint(0xed) || tradPK(32)
    /// )
    /// ```
    ///
    /// i.e. ML-DSA FIRST (consistent with
    /// [`benten_crypto_suite::sig::PublicKey::from_lamps_composite_bytes`],
    /// which expects `mldsaPK ‖ tradPK`), each component carried as its
    /// REGISTERED multiformats multicodec ([`MLDSA65_PUB_MULTICODEC`] =
    /// `0x1211`, [`ED25519_MULTICODEC`] = `0xed`) — NOT a Benten-private
    /// number (CLAUDE.md baked-in #5). The single-byte private
    /// [`HYBRID_SIG_MULTICODEC`] is fallback-only and is NOT used here.
    ///
    /// The reverse direction is [`Did::resolve_hybrid`]; round-trip:
    /// `resolve_hybrid(from_hybrid_public_key(pk))` recovers a byte-identical
    /// composite public key.
    ///
    /// # Panics
    ///
    /// Panics if `pk` carries no PQ half (a classical-only key handed to the
    /// hybrid encoder is a caller contract violation — encode a classical key
    /// via [`Did::from_public_key`] instead). The hybrid keys produced by the
    /// v1-beta default suite always carry the PQ half.
    #[must_use]
    pub fn from_hybrid_public_key(pk: &benten_crypto_suite::sig::PublicKey) -> Self {
        // Extract the two component pubkeys via the crypto-suite's symmetric
        // serializer (`mldsaPK(1952) ‖ tradPK(32)`, ML-DSA FIRST). This is the
        // ONLY place the composite byte layout is sourced — never re-derived
        // here.
        let composite = pk.to_lamps_composite_bytes().expect(
            "from_hybrid_public_key requires a hybrid (PQ-carrying) key; \
             classical-only keys encode via Did::from_public_key",
        );
        let mldsa_len = mldsa65_pubkey_len();
        let (mldsa_pk, trad_pk) = composite.split_at(mldsa_len);

        let mut payload = Vec::with_capacity(
            MLDSA65_PUB_MULTICODEC.len()
                + mldsa_pk.len()
                + ED25519_MULTICODEC.len()
                + trad_pk.len(),
        );
        // ML-DSA-65 component multikey (FIRST).
        payload.extend_from_slice(&MLDSA65_PUB_MULTICODEC);
        payload.extend_from_slice(mldsa_pk);
        // Ed25519 component multikey (SECOND).
        payload.extend_from_slice(&ED25519_MULTICODEC);
        payload.extend_from_slice(trad_pk);

        let body = bs58::encode(&payload).into_string();
        Self(format!("{DID_KEY_PREFIX}{body}"))
    }

    /// Resolve a **PQ-hybrid** `did:key` string back to its underlying
    /// composite verifying key (Ed25519⊕ML-DSA-65).
    ///
    /// Decodes the multibase body and varint-dispatches the two component
    /// multikeys IN ORDER: leading varint MUST be [`MLDSA65_PUB_MULTICODEC`]
    /// (`0x1211`) → consume the ML-DSA-65 pubkey; next varint MUST be
    /// [`ED25519_MULTICODEC`] (`0xed`) → consume the Ed25519 pubkey; then
    /// reconstruct via
    /// [`benten_crypto_suite::sig::PublicKey::from_lamps_composite_bytes`]
    /// (`mldsaPK ‖ tradPK`).
    ///
    /// Typed-rejects (NEVER panics, NEVER a silent fallback) on: a non-`z`
    /// prefix ([`DidError::InvalidPrefix`]); base58 decode failure
    /// ([`DidError::Base58Decode`]); a wrong/unknown component codec
    /// ([`DidError::UnknownMulticodec`]); a body too short for either
    /// component ([`DidError::HybridBodyTooShort`]); trailing bytes after
    /// both components ([`DidError::HybridTrailingBytes`]); or a reconstructed
    /// composite key one of whose halves is not a valid key encoding
    /// ([`DidError::InvalidHybridPublicKey`]).
    ///
    /// This is additive to [`Did::resolve`] (the Ed25519-only path) — both
    /// remain available; a hybrid `did:key` resolved via [`Did::resolve`]
    /// surfaces [`DidError::UnknownMulticodec`] (the leading `0x1211` varint
    /// is not the Ed25519 `0xed01`), so callers select the resolver matching
    /// the key shape they expect.
    ///
    /// # Errors
    ///
    /// See the typed-reject list above.
    pub fn resolve_hybrid(&self) -> Result<benten_crypto_suite::sig::PublicKey, DidError> {
        // F2: bound the input length BEFORE the O(N²) base58btc decode
        // (same pre-auth DoS defense as [`Did::resolve`]).
        self.length_pre_check()?;

        let body = self
            .0
            .strip_prefix(DID_KEY_PREFIX)
            .ok_or_else(|| DidError::InvalidPrefix(self.0.clone()))?;

        let decoded = bs58::decode(body)
            .into_vec()
            .map_err(|_| DidError::Base58Decode)?;

        let mldsa_len = mldsa65_pubkey_len();
        let mut cursor = 0usize;

        // --- Component 1: ML-DSA-65 (FIRST). ---
        // varint prefix (2 bytes) + mldsaPK(mldsa_len).
        let mldsa_min = MLDSA65_PUB_MULTICODEC.len() + mldsa_len;
        if decoded.len() < cursor + mldsa_min {
            return Err(DidError::HybridBodyTooShort {
                component: "ML-DSA-65",
                got: decoded.len().saturating_sub(cursor),
                min: mldsa_min,
            });
        }
        if decoded[cursor] != MLDSA65_PUB_MULTICODEC[0]
            || decoded[cursor + 1] != MLDSA65_PUB_MULTICODEC[1]
        {
            return Err(DidError::UnknownMulticodec(
                decoded[cursor],
                decoded[cursor + 1],
            ));
        }
        cursor += MLDSA65_PUB_MULTICODEC.len();
        let mldsa_pk = &decoded[cursor..cursor + mldsa_len];
        cursor += mldsa_len;

        // --- Component 2: Ed25519 (SECOND). ---
        let ed_min = ED25519_MULTICODEC.len() + ED25519_PUBKEY_LEN;
        if decoded.len() < cursor + ed_min {
            return Err(DidError::HybridBodyTooShort {
                component: "Ed25519",
                got: decoded.len().saturating_sub(cursor),
                min: ed_min,
            });
        }
        if decoded[cursor] != ED25519_MULTICODEC[0] || decoded[cursor + 1] != ED25519_MULTICODEC[1]
        {
            return Err(DidError::UnknownMulticodec(
                decoded[cursor],
                decoded[cursor + 1],
            ));
        }
        cursor += ED25519_MULTICODEC.len();
        let trad_pk = &decoded[cursor..cursor + ED25519_PUBKEY_LEN];
        cursor += ED25519_PUBKEY_LEN;

        // --- Reject trailing bytes (a well-formed body is EXACT). ---
        if cursor != decoded.len() {
            return Err(DidError::HybridTrailingBytes {
                extra: decoded.len() - cursor,
            });
        }

        // --- Reconstruct the composite key (mldsaPK ‖ tradPK, ML-DSA-first). ---
        let mut composite = Vec::with_capacity(mldsa_pk.len() + trad_pk.len());
        composite.extend_from_slice(mldsa_pk);
        composite.extend_from_slice(trad_pk);
        benten_crypto_suite::sig::PublicKey::from_lamps_composite_bytes(&composite)
            .map_err(|_| DidError::InvalidHybridPublicKey("LAMPS composite half not a valid key"))
    }

    /// Construct from a pre-resolved string. Caller must have already
    /// verified the string parses via [`Did::resolve`]. Used at
    /// deserialization boundaries inside `benten-id` (the round-trip
    /// pin in `prop_did_key` covers it).
    ///
    /// # #835 discharge — verify-and-execute (G-CORE-2 / 2026-05-19)
    ///
    /// Per `RATIFIED-crypto-agility-2026-05-18.md` §"Discharges": #835
    /// → "collapse to ONE unvalidated boundary (Deserialize stays
    /// structurally-trusting; the signature/codepoint gate at chain-walk
    /// is the load-bearing assertion); `from_string_unchecked` → delete
    /// or `pub(crate)`." This wave executes the `pub(crate)` half:
    /// the function is no longer reachable from outside `benten-id`
    /// (the Rust type system enforces this at compile time). External
    /// callers route through [`Did::parse_validated`] (validates the
    /// `did:key` round-trip on construction) or — for tests with
    /// hardcoded placeholder DID strings that intentionally bypass
    /// the W3C validator — [`Did::from_string_for_test_fixture`]
    /// (the explicitly test-named openly-unsafe constructor, which the
    /// `#835` audit treats as a distinct (named) surface and does NOT
    /// flag).
    pub(crate) fn from_string_unchecked(s: String) -> Self {
        Self(s)
    }

    /// Test-fixture constructor — open-unsafe, named to signal intent.
    ///
    /// Used by integration tests + Rust crate tests that need to
    /// construct a `Did` from a hardcoded placeholder string (e.g.
    /// `"did:key:z6MkAlice"`) that doesn't validate against the W3C
    /// `did:key` spec. The function name is the load-bearing safety
    /// signal — code-review for any production-path call here is
    /// always a regression flag (a callsite with `for_test_fixture`
    /// in production code reads as a self-evident smell, where the
    /// previous `from_string_unchecked` blended into surrounding
    /// production code).
    ///
    /// **Not for production code** — enforced by naming convention +
    /// code review, NOT by the type system: this constructor is `pub`
    /// and physically callable, so a production-path call is a
    /// review-flagged regression rather than a compile error. Production
    /// callers construct DIDs through the validating constructors —
    /// [`Did::parse_validated`] for a classical `did:key`, or
    /// [`Did::parse_validated_hybrid`] for the PQ-hybrid `did:key` form —
    /// so a malformed DID string surfaces a typed [`DidError`] instead of
    /// silently constructing.
    #[must_use]
    pub fn from_string_for_test_fixture(s: String) -> Self {
        Self(s)
    }

    /// Validate-on-construct typed constructor — the **post-#835-discharge
    /// safe-by-default** path for external callers (napi bindings,
    /// fixtures, other crates).
    ///
    /// Validates the input string round-trips through [`Did::resolve`]
    /// (`did:key:z` prefix + multibase-decodable + Ed25519-multicodec
    /// prefix + 32-byte pubkey) before construction. Surfaces a typed
    /// [`DidError`] on failure rather than swallowing the bad input.
    pub fn parse_validated(s: impl Into<String>) -> Result<Self, DidError> {
        let s = s.into();
        let candidate = Self(s);
        // Round-trip-validate: if `resolve()` succeeds the string is
        // well-formed; we return the same candidate.
        candidate.resolve()?;
        Ok(candidate)
    }

    /// Validate-on-construct typed constructor for a **PQ-hybrid** `did:key`
    /// (the LAMPS Composite Ed25519⊕ML-DSA-65 two-registered-component
    /// multikey form). The hybrid sibling of [`Did::parse_validated`].
    ///
    /// Validates the input string round-trips through [`Did::resolve_hybrid`]
    /// (`did:key:z` prefix + multibase-decodable + the ML-DSA-first
    /// two-component multikey layout + valid component keys) before
    /// construction. The **production-safe** path for callers (e.g.
    /// `benten-drop`'s Sealed-Sender ORIGIN-AUTH verify) that receive a
    /// hybrid sender-DID string from the wire and need to resolve its hybrid
    /// verifying key — distinct from the Ed25519-only [`Did::parse_validated`]
    /// (which would reject a hybrid DID at the leading `0x1211` multicodec)
    /// and from the test-only [`Did::from_string_for_test_fixture`].
    ///
    /// # Errors
    ///
    /// Surfaces the [`Did::resolve_hybrid`] typed-reject set (wrong prefix /
    /// base58 / unknown component multicodec / body-too-short / trailing
    /// bytes / invalid composite half) rather than swallowing bad input.
    pub fn parse_validated_hybrid(s: impl Into<String>) -> Result<Self, DidError> {
        let s = s.into();
        let candidate = Self(s);
        candidate.resolve_hybrid()?;
        Ok(candidate)
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Did {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
