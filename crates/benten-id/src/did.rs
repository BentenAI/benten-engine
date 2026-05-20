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

/// `did:key` URI prefix (literal string the W3C spec mandates).
pub const DID_KEY_PREFIX: &str = "did:key:z";

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

    /// Resolve a `did:key` string back to its underlying public key.
    ///
    /// Round-trip property (per
    /// `crates/benten-id/tests/prop_did_key.rs::prop_did_key_round_trip_byte_identity`):
    /// for any 32-byte sequence, encoding then resolving recovers the
    /// exact bytes — NO bit can be silently dropped or rewritten by
    /// the encode → decode path.
    pub fn resolve(&self) -> Result<PublicKey, DidError> {
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
    /// **NEVER call from production code.** Production callers MUST
    /// use [`Did::parse_validated`].
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
