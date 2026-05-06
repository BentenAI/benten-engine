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
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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
    /// verified the string parses via [`Did::resolve`]. (Used at
    /// deserialization boundaries; the round-trip pin still holds.)
    pub fn from_string_unchecked(s: String) -> Self {
        Self(s)
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
