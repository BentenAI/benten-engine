//! Per-Node AEAD wrap layer for the storage path (G-CORE-3d).
//!
//! # Surface
//!
//! - [`EncryptedNode`] — the storage-layer envelope for a single Node's
//!   AEAD ciphertext. Two arms: [`EncryptedNode::Whole`] (Node canonical
//!   bytes < `WHOLE_AEAD_THRESHOLD`) carrying a single
//!   `benten_crypto_suite::AeadEnvelope`, and [`EncryptedNode::Chunked`]
//!   (Node canonical bytes ≥ threshold) carrying a [`ChunkedCiphertext`].
//! - [`ChunkedCiphertext`] — N-chunk per-chunk-AEAD container; each
//!   chunk is a `benten_crypto_suite::AeadEnvelope` with
//!   `aad = aad_per_chunk(plaintext_cid, chunk_index)`. Chunk size =
//!   [`IROH_BLOCK_SIZE`] (16 KiB) — alignment with iroh's wire layer is
//!   load-bearing per §1.A.FROZEN item 15(g) ("different chunk size =
//!   double-chunking overhead").
//! - [`encrypt`] / [`decrypt`] — whole-content surface (single AEAD).
//! - [`encrypt_chunk`] / [`decrypt_chunk`] — per-chunk independent
//!   surface (range-fetch preservation per Spike H+1.2).
//! - [`AeadError`] — typed error envelope: `Authentication`,
//!   `TagMismatch`, `CiphertextTooShort`, `Unsupported`, `KeyMismatch`.
//!
//! # AAD-binds-plaintext-CID rebinding-attack prevention
//!
//! Per §1.A.FROZEN item 15(g) + Spike G/H + R3 ratification, the AAD
//! cryptographically binds:
//! - whole-content: `aad_whole_content(plaintext_cid)` — relocating a
//!   ciphertext under a different plaintext CID fails AEAD authentication.
//! - per-chunk: `aad_per_chunk(plaintext_cid, chunk_index)` — shuffling
//!   chunks within a Node fails AEAD authentication (cross-chunk
//!   rebinding defeated).
//!
//! # No-fork-primitives discipline (CLAUDE.md baked-in #5)
//!
//! Every AEAD operation routes through `benten_crypto_suite::aead::wrap`
//! / `::unwrap` / `::aad_whole_content` / `::aad_per_chunk`. This module
//! is glue: threshold dispatch + chunk slicing + envelope packaging.
//!
//! See also `crates/benten-graph/src/two_cid_map.rs` for the
//! `plaintext_cid → ciphertext_cid` mapping that stores these envelopes.

use benten_core::Cid;
use benten_crypto_suite::aead::{
    aad_per_chunk as suite_aad_per_chunk, aad_whole_content as suite_aad_whole_content,
    unwrap as suite_unwrap, wrap as suite_wrap,
};
use benten_crypto_suite::{
    AeadEnvelope, AeadError as SuiteAeadError, CipherSuiteCodepoint, KeyMaterial,
};
use thiserror::Error;

/// Per §1.A.FROZEN item 15(g): per-chunk AEAD chunk-size MUST equal
/// `iroh-blobs::IROH_BLOCK_SIZE` (16 KiB). Re-exported from
/// `benten_crypto_suite::IROH_BLOCK_SIZE` so the storage layer pins
/// against the same constant the cipher-suite layer holds. A divergent
/// chunk size yields double-chunking overhead at iroh wire transport.
pub const IROH_BLOCK_SIZE: usize = benten_crypto_suite::aead::IROH_BLOCK_SIZE;

/// Per §1.A.FROZEN item 15(g): whole-content AEAD for Nodes < 64 KiB;
/// per-chunk AEAD ≥ 64 KiB. Re-exported from
/// `benten_crypto_suite::WHOLE_CONTENT_AEAD_THRESHOLD` under the
/// historical storage-side spelling so R3 RED-PHASE pins keep their
/// import shape.
pub const WHOLE_AEAD_THRESHOLD: usize = benten_crypto_suite::aead::WHOLE_CONTENT_AEAD_THRESHOLD;

/// The graph-side per-Node AEAD envelope. Stored at the ciphertext CID
/// in the `RedbBackend` partition keyed on `WriteContext::namespace_did`.
///
/// The arm is dispatched by canonical-bytes size at encrypt time
/// ([`WHOLE_AEAD_THRESHOLD`] = 64 KiB):
/// - bytes.len() <  64 KiB → [`EncryptedNode::Whole`]
/// - bytes.len() >= 64 KiB → [`EncryptedNode::Chunked`]
///
/// At decrypt time the arm is observable structurally (the consumer can
/// `match` on it); the cipher-codepoint + format-version travel inside
/// the `AeadEnvelope` for downstream dispatch.
#[derive(Clone, Debug)]
pub enum EncryptedNode {
    /// Single-AEAD whole-content variant for Nodes below the chunking
    /// threshold. Carries the seal-time `plaintext_cid` so the
    /// decrypt-time AAD reconstruction is unambiguous + so the
    /// rebinding-attack-prevention pin can independently surface the
    /// AAD-bound CID.
    Whole {
        /// Plaintext CID that the AAD was bound to at encrypt time.
        /// AEAD authentication MUST fail at decrypt time if this field
        /// is mutated post-encrypt (the rebinding-attack defense).
        plaintext_cid: Cid,
        /// The cipher-suite envelope holding magic + format-version +
        /// codepoint + nonce + ciphertext-with-tag.
        envelope: AeadEnvelope,
    },
    /// Per-chunk variant for Nodes at-or-above the threshold. Each
    /// chunk is independently decryptable (range-fetch preservation
    /// per Spike H+1.2).
    Chunked {
        /// Plaintext CID — bound into every chunk's AAD via
        /// `aad_per_chunk(plaintext_cid, chunk_index)`.
        plaintext_cid: Cid,
        /// The chunked-AEAD container.
        chunked: ChunkedCiphertext,
    },
}

impl EncryptedNode {
    /// Encrypt `plaintext` bytes under `key` with AAD bound to
    /// `plaintext_cid`. Dispatches on `plaintext.len()` against
    /// [`WHOLE_AEAD_THRESHOLD`].
    ///
    /// # Errors
    ///
    /// - [`AeadError::Authentication`] if the cipher-suite seal layer
    ///   refuses the key material (degraded-cryptographic-state pin).
    /// - [`AeadError::Unsupported`] if `key`'s codepoint is not one of
    ///   the wave-live cipher suites (`0x647a` X-Wing-hybrid + `0x6400`
    ///   classical-only X25519 downgrade — both feed ChaCha20-Poly1305).
    pub fn encrypt(plaintext: &[u8], plaintext_cid: &Cid, key: &[u8]) -> Result<Self, AeadError> {
        let key_material = make_key_material(key)?;
        if plaintext.len() < WHOLE_AEAD_THRESHOLD {
            let aad = suite_aad_whole_content(plaintext_cid.as_bytes());
            let envelope = suite_wrap(plaintext, &key_material, &aad).map_err(AeadError::from)?;
            Ok(Self::Whole {
                plaintext_cid: *plaintext_cid,
                envelope,
            })
        } else {
            let chunked = ChunkedCiphertext::encrypt(plaintext, plaintext_cid, key)?;
            Ok(Self::Chunked {
                plaintext_cid: *plaintext_cid,
                chunked,
            })
        }
    }

    /// The plaintext CID this envelope was sealed against — the AAD
    /// binding surface.
    #[must_use]
    pub fn plaintext_cid(&self) -> &Cid {
        match self {
            Self::Whole { plaintext_cid, .. } | Self::Chunked { plaintext_cid, .. } => {
                plaintext_cid
            }
        }
    }

    /// Test-only seam: clone `self` with a re-written AAD-binding
    /// plaintext CID. Used by the rebinding-attack pin
    /// (tf3d_aad_binds_plaintext_cid_rebinding_attack.rs PIN 1) to
    /// mount valid ciphertext bytes under a different plaintext CID;
    /// decrypt MUST fail AEAD authentication because the AAD that the
    /// authenticator reconstructs at decrypt time
    /// (`aad_whole_content(plaintext_cid)` with the *mutated* CID) no
    /// longer matches the AAD bound at seal time.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn with_aad_plaintext_cid_for_test(&self, new_plaintext_cid: Cid) -> Self {
        match self {
            Self::Whole { envelope, .. } => Self::Whole {
                plaintext_cid: new_plaintext_cid,
                envelope: envelope.clone(),
            },
            Self::Chunked { chunked, .. } => Self::Chunked {
                plaintext_cid: new_plaintext_cid,
                chunked: chunked.clone(),
            },
        }
    }

    /// Test-only seam: strip the last 16 bytes (the Poly1305
    /// authentication tag) from the ciphertext for truncation pins.
    /// Decrypt MUST fail with a typed `Authentication` / `TagMismatch`
    /// / `CiphertextTooShort` error.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn truncate_tag_for_test(&self) -> Self {
        match self {
            Self::Whole {
                plaintext_cid,
                envelope,
            } => {
                let mut clone = envelope.clone();
                let new_len = clone.ciphertext.len().saturating_sub(16);
                clone.ciphertext.truncate(new_len);
                Self::Whole {
                    plaintext_cid: *plaintext_cid,
                    envelope: clone,
                }
            }
            Self::Chunked {
                plaintext_cid,
                chunked,
            } => Self::Chunked {
                plaintext_cid: *plaintext_cid,
                chunked: chunked.clone(),
            },
        }
    }
}

/// Per-chunk-AEAD container for Nodes ≥ [`WHOLE_AEAD_THRESHOLD`].
///
/// Each chunk is exactly [`IROH_BLOCK_SIZE`] bytes of plaintext (the
/// final chunk may be smaller). The plaintext is sliced
/// `bytes.chunks(IROH_BLOCK_SIZE)` and each slice is AEAD-sealed
/// independently under `aad_per_chunk(plaintext_cid, chunk_index)`.
///
/// Two load-bearing invariants:
/// 1. Chunk-size = [`IROH_BLOCK_SIZE`] (alignment with iroh's wire
///    chunking — different size = double-chunking overhead per Spike
///    H+1.2).
/// 2. AAD binds `chunk_index` — shuffling a chunk to a different index
///    fails AEAD authentication (cross-chunk-rebinding defense).
#[derive(Clone, Debug)]
pub struct ChunkedCiphertext {
    pub(crate) chunks: Vec<AeadEnvelope>,
}

impl ChunkedCiphertext {
    /// Encrypt `plaintext` bytes into N independently-decryptable
    /// AEAD chunks of [`IROH_BLOCK_SIZE`] bytes each (final chunk may
    /// be shorter). Each chunk's AAD binds
    /// `(plaintext_cid, chunk_index)`.
    ///
    /// # Errors
    ///
    /// Same arms as [`EncryptedNode::encrypt`].
    pub fn encrypt(plaintext: &[u8], plaintext_cid: &Cid, key: &[u8]) -> Result<Self, AeadError> {
        let key_material = make_key_material(key)?;
        let mut chunks = Vec::with_capacity(plaintext.len().div_ceil(IROH_BLOCK_SIZE));
        for (chunk_index, slice) in plaintext.chunks(IROH_BLOCK_SIZE).enumerate() {
            let chunk_index_u64 = u64::try_from(chunk_index)
                .map_err(|_| AeadError::Authentication("chunk index exceeds u64".to_string()))?;
            let aad = suite_aad_per_chunk(plaintext_cid.as_bytes(), chunk_index_u64);
            let envelope = suite_wrap(slice, &key_material, &aad).map_err(AeadError::from)?;
            chunks.push(envelope);
        }
        Ok(Self { chunks })
    }

    /// The N chunk envelopes in order.
    #[must_use]
    pub fn chunks(&self) -> &[AeadEnvelope] {
        &self.chunks
    }
}

/// Whole-content AEAD decrypt for an [`EncryptedNode`]. Reconstructs the
/// AAD from the envelope's plaintext-CID field + dispatches to the
/// cipher-suite `unwrap`.
///
/// # Errors
///
/// - [`AeadError::Authentication`] on tag mismatch — including
///   rebinding attacks where `EncryptedNode::with_aad_plaintext_cid_for_test`
///   was used to mutate the AAD-bound CID post-seal.
/// - [`AeadError::TagMismatch`] / [`AeadError::CiphertextTooShort`] on
///   structural failures.
/// - [`AeadError::Unsupported`] on codepoint mismatch (silent-downgrade
///   defense per CLAUDE.md baked-in #5).
pub fn decrypt(envelope: &EncryptedNode, key: &[u8]) -> Result<Vec<u8>, AeadError> {
    let key_material = make_key_material_matching(envelope, key)?;
    match envelope {
        EncryptedNode::Whole {
            plaintext_cid,
            envelope: aead_env,
        } => {
            // Pre-flight: an envelope whose ciphertext is shorter than
            // the AEAD tag (16 B for Poly1305) is structurally
            // impossible to decrypt — surface as a typed
            // `CiphertextTooShort` rather than letting the underlying
            // primitive return a generic auth-fail.
            if aead_env.ciphertext.len() < 16 {
                return Err(AeadError::CiphertextTooShort {
                    got: aead_env.ciphertext.len(),
                });
            }
            let aad = suite_aad_whole_content(plaintext_cid.as_bytes());
            suite_unwrap(aead_env, &key_material, &aad).map_err(AeadError::from)
        }
        EncryptedNode::Chunked {
            plaintext_cid,
            chunked,
        } => {
            // Decrypt every chunk in order + reassemble. The
            // independent-decrypt property is exercised by
            // `decrypt_chunk` directly; this is the whole-Node read path.
            let mut out = Vec::new();
            for (chunk_index, chunk) in chunked.chunks.iter().enumerate() {
                let chunk_index_u64 = u64::try_from(chunk_index).map_err(|_| {
                    AeadError::Authentication("chunk index exceeds u64".to_string())
                })?;
                let aad = suite_aad_per_chunk(plaintext_cid.as_bytes(), chunk_index_u64);
                let plaintext =
                    suite_unwrap(chunk, &key_material, &aad).map_err(AeadError::from)?;
                out.extend_from_slice(&plaintext);
            }
            Ok(out)
        }
    }
}

/// Free-function alias for [`EncryptedNode::encrypt`]. Provided so
/// `use benten_graph::aead_wrap::{encrypt, decrypt}` callers (mirroring
/// the cipher-suite-side `wrap`/`unwrap` shape) get a symmetric API
/// without unwrapping the enum constructor at every call site.
///
/// # Errors
///
/// See [`EncryptedNode::encrypt`].
pub fn encrypt(
    plaintext: &[u8],
    plaintext_cid: &Cid,
    key: &[u8],
) -> Result<EncryptedNode, AeadError> {
    EncryptedNode::encrypt(plaintext, plaintext_cid, key)
}

/// Decrypt a single chunk INDEPENDENTLY at `chunk_index` — the
/// range-fetch preservation property per Spike H+1.2.
///
/// # Errors
///
/// - [`AeadError::Authentication`] on tag mismatch — including the
///   cross-chunk-rebinding attack (chunk-N's ciphertext presented at
///   index M ≠ N fails because the AAD-bound `chunk_index` doesn't
///   match).
/// - [`AeadError::Unsupported`] on codepoint mismatch.
pub fn decrypt_chunk(
    chunk: &AeadEnvelope,
    chunk_index: usize,
    plaintext_cid: &Cid,
    key: &[u8],
) -> Result<Vec<u8>, AeadError> {
    let key_material = KeyMaterial::from_raw_bytes(chunk.cipher_codepoint, key);
    let chunk_index_u64 = u64::try_from(chunk_index)
        .map_err(|_| AeadError::Authentication("chunk index exceeds u64".to_string()))?;
    let aad = suite_aad_per_chunk(plaintext_cid.as_bytes(), chunk_index_u64);
    suite_unwrap(chunk, &key_material, &aad).map_err(AeadError::from)
}

/// Seal one chunk's bytes at `chunk_index` — the producer-side
/// counterpart to [`decrypt_chunk`]. Used by sync-side bytes-as-they-
/// arrive ingest paths that need to seal incrementally rather than
/// materializing the whole Node first.
///
/// # Errors
///
/// Same arms as [`EncryptedNode::encrypt`].
pub fn encrypt_chunk(
    chunk_plaintext: &[u8],
    chunk_index: usize,
    plaintext_cid: &Cid,
    key: &[u8],
) -> Result<AeadEnvelope, AeadError> {
    let key_material = make_key_material(key)?;
    let chunk_index_u64 = u64::try_from(chunk_index)
        .map_err(|_| AeadError::Authentication("chunk index exceeds u64".to_string()))?;
    let aad = suite_aad_per_chunk(plaintext_cid.as_bytes(), chunk_index_u64);
    suite_wrap(chunk_plaintext, &key_material, &aad).map_err(AeadError::from)
}

/// Serialize an [`EncryptedNode`] into a wire byte vector that can be
/// stored in redb at `ENCRYPTED_NODES_TABLE`. The format is:
///
/// ```text
/// byte 0    : magic 0x3d (G-CORE-3d storage envelope identifier)
/// byte 1    : variant tag — 0x00 = Whole, 0x01 = Chunked
/// bytes 2-37: plaintext CID (36-B raw layout)
/// (Whole)
///   bytes 38..: AeadEnvelope::to_wire_bytes()
/// (Chunked)
///   bytes 38-41 : chunk count (LE u32)
///   for each chunk: u32 LE length || AeadEnvelope::to_wire_bytes()
/// ```
///
/// The format is internal to G-CORE-3d's storage layer and is NOT a
/// frozen public surface (the v1 wire-freeze happens at G-CORE-9).
/// Any drift here is caught by the round-trip pins +
/// `tf3d_two_cid_mapping_durable_across_reopen`.
//
// See also crates/benten-drop/src/bundle.rs DropBundle docstring for the
// parallel wire-format coupling callout — the Drop bundle's
// `EncryptedContent.bytes` wraps this storage encoding, so both layers
// are atomically frozen at G-CORE-9. Any pre-freeze mutation here MUST
// also retense benten-drop's DropBundle wire shape + the tf3f
// offline-consume pins.
pub fn encode_encrypted_node(encrypted: &EncryptedNode) -> Result<Vec<u8>, AeadError> {
    const STORAGE_MAGIC: u8 = 0x3d;
    let mut out = Vec::new();
    out.push(STORAGE_MAGIC);
    match encrypted {
        EncryptedNode::Whole {
            plaintext_cid,
            envelope,
        } => {
            out.push(0x00);
            out.extend_from_slice(plaintext_cid.as_bytes());
            out.extend_from_slice(&envelope.to_wire_bytes());
        }
        EncryptedNode::Chunked {
            plaintext_cid,
            chunked,
        } => {
            out.push(0x01);
            out.extend_from_slice(plaintext_cid.as_bytes());
            let count =
                u32::try_from(chunked.chunks.len()).map_err(|_| AeadError::KeyMismatch {
                    reason: "chunk count exceeds u32".to_string(),
                })?;
            out.extend_from_slice(&count.to_le_bytes());
            for chunk in &chunked.chunks {
                let bytes = chunk.to_wire_bytes();
                let len = u32::try_from(bytes.len()).map_err(|_| AeadError::KeyMismatch {
                    reason: "chunk length exceeds u32".to_string(),
                })?;
                out.extend_from_slice(&len.to_le_bytes());
                out.extend_from_slice(&bytes);
            }
        }
    }
    Ok(out)
}

/// Inverse of [`encode_encrypted_node`].
pub fn decode_encrypted_node(bytes: &[u8]) -> Result<EncryptedNode, AeadError> {
    const STORAGE_MAGIC: u8 = 0x3d;
    if bytes.len() < 2 + 36 {
        return Err(AeadError::CiphertextTooShort { got: bytes.len() });
    }
    if bytes[0] != STORAGE_MAGIC {
        return Err(AeadError::Authentication(
            "storage-envelope bad magic byte".to_string(),
        ));
    }
    let variant = bytes[1];
    let plaintext_cid = Cid::from_bytes(&bytes[2..38])
        .map_err(|e| AeadError::Authentication(format!("plaintext_cid parse: {e:?}")))?;
    match variant {
        0x00 => {
            let env_bytes = &bytes[38..];
            let envelope = AeadEnvelope::from_wire_bytes(env_bytes).map_err(AeadError::from)?;
            Ok(EncryptedNode::Whole {
                plaintext_cid,
                envelope,
            })
        }
        0x01 => {
            if bytes.len() < 38 + 4 {
                return Err(AeadError::CiphertextTooShort { got: bytes.len() });
            }
            let count = u32::from_le_bytes([bytes[38], bytes[39], bytes[40], bytes[41]]) as usize;
            let mut cursor = 42usize;
            let mut chunks = Vec::with_capacity(count);
            for _ in 0..count {
                if cursor + 4 > bytes.len() {
                    return Err(AeadError::CiphertextTooShort { got: bytes.len() });
                }
                let len = u32::from_le_bytes([
                    bytes[cursor],
                    bytes[cursor + 1],
                    bytes[cursor + 2],
                    bytes[cursor + 3],
                ]) as usize;
                cursor += 4;
                if cursor + len > bytes.len() {
                    return Err(AeadError::CiphertextTooShort { got: bytes.len() });
                }
                let envelope = AeadEnvelope::from_wire_bytes(&bytes[cursor..cursor + len])
                    .map_err(AeadError::from)?;
                cursor += len;
                chunks.push(envelope);
            }
            Ok(EncryptedNode::Chunked {
                plaintext_cid,
                chunked: ChunkedCiphertext { chunks },
            })
        }
        other => Err(AeadError::Authentication(format!(
            "storage-envelope unknown variant tag {other:#x}"
        ))),
    }
}

/// Build a `KeyMaterial` from raw bytes for the wave-live default
/// codepoint (X-Wing-hybrid `0x647a`). Production key-derivation paths
/// already produce typed `KeyMaterial` via the cipher-suite; this
/// helper is for the storage-layer surface that accepts a `&[u8]` key
/// (the `decrypt(&envelope, &key: &[u8])` shape the R3 pins use).
fn make_key_material(key: &[u8]) -> Result<KeyMaterial, AeadError> {
    if key.len() != 32 {
        return Err(AeadError::KeyMismatch {
            reason: "AEAD key MUST be 32 B for ChaCha20-Poly1305 dispatch".to_string(),
        });
    }
    Ok(KeyMaterial::from_raw_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        key,
    ))
}

/// Build a `KeyMaterial` whose codepoint matches the envelope's. The
/// cipher-suite `unwrap` typed-rejects on codepoint mismatch; this
/// helper threads the envelope's codepoint into the key newtype so a
/// caller that presents the *correct* key for an envelope sealed under
/// a non-default codepoint (e.g. `0x6400` classical-only X25519
/// downgrade arm) gets through, while a caller with a wrong-codepoint
/// presumption surfaces `AeadError::Unsupported`.
fn make_key_material_matching(
    envelope: &EncryptedNode,
    key: &[u8],
) -> Result<KeyMaterial, AeadError> {
    if key.len() != 32 {
        return Err(AeadError::KeyMismatch {
            reason: "AEAD key MUST be 32 B for ChaCha20-Poly1305 dispatch".to_string(),
        });
    }
    let codepoint = match envelope {
        EncryptedNode::Whole { envelope, .. } => envelope.cipher_codepoint,
        EncryptedNode::Chunked { chunked, .. } => chunked
            .chunks
            .first()
            .map_or(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, |e| {
                e.cipher_codepoint
            }),
    };
    Ok(KeyMaterial::from_raw_bytes(codepoint, key))
}

/// Graph-side AEAD error envelope. Maps the cipher-suite's typed errors
/// onto the storage-side surface + adds typed arms for the
/// storage-specific pre-flight cases (key length mismatch +
/// structurally-too-short ciphertext).
#[derive(Debug, Error)]
pub enum AeadError {
    /// AEAD authentication failed — tag mismatch or AAD rebinding.
    /// The load-bearing rebinding-attack defense per §1.A.FROZEN item
    /// 15(g). Surfaced when the cipher-suite layer returns
    /// `AeadAuthFailed`.
    #[error("AEAD authentication failed: {0}")]
    Authentication(String),

    /// Poly1305 tag mismatch on a structurally-valid envelope. Distinct
    /// from generic authentication failure for tests that pin the
    /// specific failure shape.
    #[error("Poly1305 tag mismatch")]
    TagMismatch {
        /// Optional diagnostic note.
        note: &'static str,
    },

    /// Ciphertext bytes are too short to contain a Poly1305 tag (16 B).
    /// Pre-flight tripwire: an envelope whose ciphertext is < 16 B
    /// cannot have been produced by ChaCha20-Poly1305 against any
    /// payload + tag.
    #[error("ciphertext too short for AEAD tag: got {got} bytes (need >= 16)")]
    CiphertextTooShort {
        /// Observed ciphertext length.
        got: usize,
    },

    /// Codepoint dispatch surfaced typed-unsupported (NEVER silent
    /// fallback per CLAUDE.md baked-in #5).
    #[error("unsupported cipher suite codepoint: {note}")]
    Unsupported {
        /// Diagnostic note describing the codepoint.
        note: String,
    },

    /// Key length / codepoint dispatch failure at the storage-side
    /// pre-flight (before reaching the cipher-suite layer).
    #[error("AEAD key mismatch: {reason}")]
    KeyMismatch {
        /// Human-readable reason — bad length, codepoint mismatch, etc.
        reason: String,
    },
}

impl From<SuiteAeadError> for AeadError {
    fn from(value: SuiteAeadError) -> Self {
        match value {
            SuiteAeadError::AeadAuthFailed => Self::Authentication(
                "cipher-suite AEAD auth fail (tag mismatch or AAD rebinding)".to_string(),
            ),
            SuiteAeadError::MalformedEnvelope(reason) => {
                Self::Authentication(format!("malformed envelope: {reason}"))
            }
            SuiteAeadError::RecipientLacksKeysForSuite => Self::KeyMismatch {
                reason: "recipient lacks one of the required key halves for the cipher suite"
                    .to_string(),
            },
            SuiteAeadError::Unsupported(unsupported) => Self::Unsupported {
                note: format!("{unsupported:?}"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_cid(seed: u8) -> Cid {
        Cid::from_blake3_digest([seed; 32])
    }

    #[test]
    fn whole_round_trip_at_hybrid_codepoint() {
        let cid = fixed_cid(0xaa);
        let key = [0x42u8; 32];
        let plaintext = b"hello, whole-content AEAD";
        let env = EncryptedNode::encrypt(plaintext, &cid, &key).unwrap();
        let decrypted = decrypt(&env, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn rebinding_attack_fails() {
        let cid_a = fixed_cid(0xaa);
        let cid_b = fixed_cid(0xbb);
        let key = [0x42u8; 32];
        let env = EncryptedNode::encrypt(b"payload", &cid_a, &key).unwrap();
        let attacker = env.with_aad_plaintext_cid_for_test(cid_b);
        let result = decrypt(&attacker, &key);
        assert!(matches!(result, Err(AeadError::Authentication { .. })));
    }

    #[test]
    fn chunked_round_trip_independent_decrypt() {
        let cid = fixed_cid(0xcc);
        let key = [0x77u8; 32];
        // 4 × IROH_BLOCK_SIZE = 64 KiB → exactly threshold (uses chunked arm)
        let plaintext = vec![0xA5u8; 4 * IROH_BLOCK_SIZE];
        let chunked = ChunkedCiphertext::encrypt(&plaintext, &cid, &key).unwrap();
        assert_eq!(chunked.chunks().len(), 4);
        // Independent decrypt of chunk 2
        let chunk_2 = decrypt_chunk(&chunked.chunks()[2], 2, &cid, &key).unwrap();
        let start = 2 * IROH_BLOCK_SIZE;
        let end = start + IROH_BLOCK_SIZE;
        assert_eq!(chunk_2, &plaintext[start..end]);
    }

    #[test]
    fn cross_chunk_rebinding_fails() {
        let cid = fixed_cid(0xdd);
        let key = [0x99u8; 32];
        let plaintext = vec![0x5Au8; 4 * IROH_BLOCK_SIZE];
        let chunked = ChunkedCiphertext::encrypt(&plaintext, &cid, &key).unwrap();
        // Present chunk-1 at index-3 (wrong index → AAD mismatch)
        let result = decrypt_chunk(&chunked.chunks()[1], 3, &cid, &key);
        assert!(matches!(result, Err(AeadError::Authentication { .. })));
    }

    #[test]
    fn wrong_key_fails() {
        let cid = fixed_cid(0xee);
        let key = [0x11u8; 32];
        let wrong = [0x22u8; 32];
        let env = EncryptedNode::encrypt(b"payload", &cid, &key).unwrap();
        let result = decrypt(&env, &wrong);
        assert!(matches!(result, Err(AeadError::Authentication { .. })));
    }

    #[test]
    fn iroh_block_size_constant_is_16_kib() {
        assert_eq!(IROH_BLOCK_SIZE, 16 * 1024);
    }

    #[test]
    fn whole_aead_threshold_is_64_kib() {
        assert_eq!(WHOLE_AEAD_THRESHOLD, 64 * 1024);
    }

    #[test]
    fn key_length_mismatch_typed_reject() {
        let cid = fixed_cid(0x01);
        let short = [0u8; 16];
        let result = EncryptedNode::encrypt(b"x", &cid, &short);
        assert!(matches!(result, Err(AeadError::KeyMismatch { .. })));
    }

    #[test]
    fn truncated_ciphertext_yields_typed_error() {
        let cid = fixed_cid(0x02);
        let key = [0x33u8; 32];
        let env = EncryptedNode::encrypt(b"payload-to-truncate", &cid, &key).unwrap();
        let truncated = env.truncate_tag_for_test();
        let result = decrypt(&truncated, &key);
        assert!(matches!(
            result,
            Err(AeadError::Authentication { .. }
                | AeadError::TagMismatch { .. }
                | AeadError::CiphertextTooShort { .. })
        ));
    }
}
