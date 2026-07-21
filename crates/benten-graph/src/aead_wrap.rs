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
//!   `aad = aad_per_chunk(plaintext_cid, chunk_index, total_chunks)`. Chunk size =
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
//! - per-chunk: `aad_per_chunk(plaintext_cid, chunk_index, total_chunks)` — shuffling
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
    aad_per_chunk as suite_aad_per_chunk, aad_per_recipe as suite_aad_per_recipe,
    aad_whole_content as suite_aad_whole_content, unwrap as suite_unwrap, wrap as suite_wrap,
};
use benten_crypto_suite::{
    AeadEnvelope, AeadError as SuiteAeadError, AeadKeyMaterial, CipherSuiteCodepoint,
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
        /// `aad_per_chunk(plaintext_cid, chunk_index, total_chunks)`.
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

    /// Encrypt `plaintext` bytes under `key` with AAD bound to
    /// `(plaintext_cid, recipe_index, total_recipes)` — for use within
    /// a multi-Recipe container (e.g. `DropBundle.content`). Closes
    /// the inter-Recipe truncation attack per the F3 chunk precedent.
    ///
    /// Always returns an [`EncryptedNode::Whole`] (multi-Recipe
    /// containers seal each Recipe as a whole; chunked Recipes within
    /// a bundle are NOT a wave-3f shape). The decrypt counterpart is
    /// [`decrypt_recipe_encrypted_node`].
    ///
    /// **R6 R2 fix-pass (Bundle R6-R2-FP-A L4 sibling).**
    ///
    /// # Errors
    ///
    /// Same arms as [`Self::encrypt`].
    pub fn encrypt_recipe(
        plaintext: &[u8],
        plaintext_cid: &Cid,
        key: &[u8],
        recipe_index: u32,
        total_recipes: u32,
    ) -> Result<Self, AeadError> {
        let envelope = encrypt_recipe(plaintext, recipe_index, total_recipes, plaintext_cid, key)?;
        Ok(Self::Whole {
            plaintext_cid: *plaintext_cid,
            envelope,
        })
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
/// independently under `aad_per_chunk(plaintext_cid, chunk_index, total_chunks)`.
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
        let total_chunks_usize = plaintext.len().div_ceil(IROH_BLOCK_SIZE).max(1);
        let total_chunks =
            u32::try_from(total_chunks_usize).map_err(|_| AeadError::KeyMismatch {
                reason: "chunk count exceeds u32".to_string(),
            })?;
        let mut chunks = Vec::with_capacity(total_chunks_usize);
        for (chunk_index, slice) in plaintext.chunks(IROH_BLOCK_SIZE).enumerate() {
            let chunk_index_u64 = u64::try_from(chunk_index)
                .map_err(|_| AeadError::Authentication("chunk index exceeds u64".to_string()))?;
            // F3 (R6 R1 fix-pass): AAD binds total_chunks for
            // cross-chunk-truncation defense. Per-chunk authentication
            // fails if an attacker presents a shorter slice of chunks
            // (the per-chunk AAD committed at seal time names the
            // original total_chunks count).
            let aad = suite_aad_per_chunk(plaintext_cid.as_bytes(), chunk_index_u64, total_chunks);
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
            //
            // F3 (R6 R1 fix-pass): AAD binds total_chunks (= chunks.len()
            // observed at decrypt time). If an attacker truncates the
            // chunk list to N' < N, the per-chunk AAD reconstructed here
            // commits to total_chunks=N' but the seal-time AAD committed
            // to total_chunks=N — per-chunk authentication fails at
            // every boundary, surfacing the truncation cryptographically.
            let total_chunks =
                u32::try_from(chunked.chunks.len()).map_err(|_| AeadError::KeyMismatch {
                    reason: "chunk count exceeds u32".to_string(),
                })?;
            let mut out = Vec::new();
            for (chunk_index, chunk) in chunked.chunks.iter().enumerate() {
                let chunk_index_u64 = u64::try_from(chunk_index).map_err(|_| {
                    AeadError::Authentication("chunk index exceeds u64".to_string())
                })?;
                let aad =
                    suite_aad_per_chunk(plaintext_cid.as_bytes(), chunk_index_u64, total_chunks);
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

/// Decrypt a single chunk INDEPENDENTLY at `chunk_index` of a chunked
/// ciphertext whose total chunk count is `total_chunks` — the
/// range-fetch preservation property per Spike H+1.2.
///
/// F3 (R6 R1 fix-pass): `total_chunks` is now an AAD-bound input. The
/// caller (range-fetch consumer) MUST present the same `total_chunks`
/// the producer sealed under; presenting a different value
/// (the cross-chunk-truncation attack — claim a 10-chunk ciphertext is
/// really 5-chunk) surfaces `AeadError::Authentication`.
///
/// # Errors
///
/// - [`AeadError::Authentication`] on tag mismatch — including the
///   cross-chunk-rebinding attack (chunk-N's ciphertext presented at
///   index M ≠ N fails because the AAD-bound `chunk_index` doesn't
///   match) AND the cross-chunk-truncation attack (presenting a chunk
///   under a different `total_chunks` value fails because the AAD-bound
///   `total_chunks` doesn't match).
/// - [`AeadError::Unsupported`] on codepoint mismatch.
pub fn decrypt_chunk(
    chunk: &AeadEnvelope,
    chunk_index: usize,
    total_chunks: u32,
    plaintext_cid: &Cid,
    key: &[u8],
) -> Result<Vec<u8>, AeadError> {
    let key_material = AeadKeyMaterial::from_raw_bytes(chunk.cipher_codepoint, key);
    let chunk_index_u64 = u64::try_from(chunk_index)
        .map_err(|_| AeadError::Authentication("chunk index exceeds u64".to_string()))?;
    let aad = suite_aad_per_chunk(plaintext_cid.as_bytes(), chunk_index_u64, total_chunks);
    suite_unwrap(chunk, &key_material, &aad).map_err(AeadError::from)
}

/// Seal one chunk's bytes at `chunk_index` of a chunked ciphertext whose
/// total chunk count is `total_chunks` — the producer-side counterpart
/// to [`decrypt_chunk`]. Used by sync-side bytes-as-they-arrive ingest
/// paths that need to seal incrementally rather than materializing the
/// whole Node first.
///
/// F3 (R6 R1 fix-pass): the caller MUST know `total_chunks` at seal
/// time so the AAD commits the cross-chunk-truncation defense field.
/// Sync-side incremental sealers compute this from the plaintext-length
/// hint they received with the ingest stream.
///
/// # Errors
///
/// Same arms as [`EncryptedNode::encrypt`].
pub fn encrypt_chunk(
    chunk_plaintext: &[u8],
    chunk_index: usize,
    total_chunks: u32,
    plaintext_cid: &Cid,
    key: &[u8],
) -> Result<AeadEnvelope, AeadError> {
    let key_material = make_key_material(key)?;
    let chunk_index_u64 = u64::try_from(chunk_index)
        .map_err(|_| AeadError::Authentication("chunk index exceeds u64".to_string()))?;
    let aad = suite_aad_per_chunk(plaintext_cid.as_bytes(), chunk_index_u64, total_chunks);
    suite_wrap(chunk_plaintext, &key_material, &aad).map_err(AeadError::from)
}

/// Encrypt one Recipe's bytes at `recipe_index` of a multi-Recipe
/// container (e.g. `DropBundle.content: Vec<EncryptedContent>`) whose
/// total Recipe count is `total_recipes`. Position-aware AAD binding
/// closes the inter-Recipe truncation attack at the container layer.
///
/// **R6 R2 fix-pass (Bundle R6-R2-FP-A L4 sibling):** prior to this
/// helper, multi-Recipe containers sealed each Recipe with
/// `EncryptedNode::encrypt` (whole-content AAD only). An attacker
/// could drop one Recipe from the `Vec` and the remaining Recipes'
/// AEAD tags still verified individually because the per-Recipe seal
/// committed neither to its position nor to the list length. With
/// `recipe_index` + `total_recipes` AAD-bound (matching the F3
/// precedent for per-chunk), per-Recipe authentication fails when
/// the consumer reconstructs AAD with a shorter `total_recipes`.
///
/// Mirrors [`encrypt_chunk`] exactly in shape — the only differences
/// are the AAD seam ([`suite_aad_per_recipe`] vs `aad_per_chunk`) and
/// that the position type is `u32` (Recipe lists are bounded
/// container-wise; chunk indices are `u64` because individual content
/// can be much larger than the container Recipe count).
///
/// The returned [`AeadEnvelope`] is a self-contained whole-content
/// envelope that the consumer pairs with `recipe_index` +
/// `total_recipes` at decrypt time via [`decrypt_recipe`].
///
/// # Errors
///
/// Same arms as [`EncryptedNode::encrypt`].
pub fn encrypt_recipe(
    plaintext: &[u8],
    recipe_index: u32,
    total_recipes: u32,
    plaintext_cid: &Cid,
    key: &[u8],
) -> Result<AeadEnvelope, AeadError> {
    let key_material = make_key_material(key)?;
    let aad = suite_aad_per_recipe(plaintext_cid.as_bytes(), recipe_index, total_recipes);
    suite_wrap(plaintext, &key_material, &aad).map_err(AeadError::from)
}

/// Decrypt one Recipe at `recipe_index` of a multi-Recipe container
/// whose total Recipe count is `total_recipes` — the consumer-side
/// counterpart to [`encrypt_recipe`].
///
/// **R6 R2 fix-pass (Bundle R6-R2-FP-A L4 sibling):** the consumer
/// (e.g. `DropBundle::consume_offline`) MUST present the same
/// `total_recipes` the producer sealed under; presenting a different
/// value (the inter-Recipe truncation attack — claim a 5-Recipe
/// bundle is really 4-Recipe) surfaces `AeadError::Authentication`.
///
/// # Errors
///
/// - [`AeadError::Authentication`] on tag mismatch — including the
///   inter-Recipe rebinding attack (Recipe-N's ciphertext presented
///   at index M ≠ N) AND the inter-Recipe truncation attack
///   (Recipe presented under a different `total_recipes` value).
/// - [`AeadError::CiphertextTooShort`] on structural failures.
/// - [`AeadError::Unsupported`] on codepoint mismatch.
pub fn decrypt_recipe(
    envelope: &AeadEnvelope,
    recipe_index: u32,
    total_recipes: u32,
    plaintext_cid: &Cid,
    key: &[u8],
) -> Result<Vec<u8>, AeadError> {
    let key_material = AeadKeyMaterial::from_raw_bytes(envelope.cipher_codepoint, key);
    if envelope.ciphertext.len() < 16 {
        return Err(AeadError::CiphertextTooShort {
            got: envelope.ciphertext.len(),
        });
    }
    let aad = suite_aad_per_recipe(plaintext_cid.as_bytes(), recipe_index, total_recipes);
    suite_unwrap(envelope, &key_material, &aad).map_err(AeadError::from)
}

/// Decrypt an `EncryptedNode` that was sealed via
/// [`EncryptedNode::encrypt_recipe`] (i.e. with per-Recipe AAD binding).
/// The consumer MUST present the same `recipe_index` + `total_recipes`
/// the producer sealed under.
///
/// **R6 R2 fix-pass (Bundle R6-R2-FP-A L4 sibling):** the inter-Recipe
/// truncation defense fires here — if the bundle's `Vec<Recipe>` is
/// truncated by a relay, the consumer reconstructs AAD with a smaller
/// `total_recipes` than was sealed and AEAD authentication fails
/// cryptographically (no silent admission).
///
/// Only [`EncryptedNode::Whole`] is supported (per the wave-3f shape:
/// each Recipe is a whole-content seal at the bundle layer). A
/// `EncryptedNode::Chunked` input returns
/// [`AeadError::Unsupported`].
///
/// # Errors
///
/// - [`AeadError::Authentication`] on any tamper — including
///   inter-Recipe truncation/shuffle, plaintext-CID rebinding, and
///   ciphertext bit-flip.
/// - [`AeadError::CiphertextTooShort`] on structural failures.
/// - [`AeadError::Unsupported`] on codepoint mismatch OR on a
///   `EncryptedNode::Chunked` input.
pub fn decrypt_recipe_encrypted_node(
    node: &EncryptedNode,
    key: &[u8],
    recipe_index: u32,
    total_recipes: u32,
) -> Result<Vec<u8>, AeadError> {
    match node {
        EncryptedNode::Whole {
            plaintext_cid,
            envelope,
        } => decrypt_recipe(envelope, recipe_index, total_recipes, plaintext_cid, key),
        EncryptedNode::Chunked { .. } => Err(AeadError::Unsupported {
            note: "EncryptedNode::Chunked is not supported by the per-Recipe \
                   bundle-level decrypt path; Recipes must be sealed via \
                   EncryptedNode::encrypt_recipe which always returns Whole"
                .to_string(),
        }),
    }
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
/// The format is internal to G-CORE-3d's storage layer; it was FROZEN
/// at the G-CORE-9 v1-beta wire-freeze (it is not a `pub`-API surface,
/// but its bytes are now part of the frozen v1 wire contract). Any
/// drift here is caught by the round-trip pins +
/// `tf3d_two_cid_mapping_durable_across_reopen`.
//
// See also crates/benten-drop/src/bundle.rs DropBundle docstring for the
// parallel wire-format coupling callout — the Drop bundle's
// `EncryptedContent.bytes` wraps this storage encoding, so both layers
// were atomically frozen at G-CORE-9. Any post-freeze change here is a
// wire-break and MUST route through the crypto-agility framework
// (additive codepoint, never an in-place mutation) — mirrored in
// benten-drop's DropBundle wire shape + the tf3f offline-consume pins.
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
            // M-19: chunk count + per-chunk length prefixes BIG-ENDIAN
            // (migrated from LE at F-full Wave-0).
            out.extend_from_slice(&count.to_be_bytes());
            for chunk in &chunked.chunks {
                let bytes = chunk.to_wire_bytes();
                let len = u32::try_from(bytes.len()).map_err(|_| AeadError::KeyMismatch {
                    reason: "chunk length exceeds u32".to_string(),
                })?;
                out.extend_from_slice(&len.to_be_bytes());
                out.extend_from_slice(&bytes);
            }
        }
    }
    Ok(out)
}

/// Compute `off + len` and verify the end lies within `total`, using
/// `checked_add` so a wire-read `len` (attacker-controlled `u32 as usize`)
/// cannot OVERFLOW-WRAP `off + len` below `total` and spuriously pass a
/// naive `off + len > total` bounds check on a 32-bit `usize` target
/// (wasm32 thin-client). Returns `None` — fail-closed → typed reject — on
/// EITHER integer overflow OR out-of-bounds. Mirrors `benten-drop`'s
/// `layer_c::lp_range_end` discipline (F-04). No behavioral change on
/// 64-bit where these lengths cannot overflow.
#[inline]
fn checked_range_end(off: usize, len: usize, total: usize) -> Option<usize> {
    match off.checked_add(len) {
        Some(end) if end <= total => Some(end),
        _ => None,
    }
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
            let count = u32::from_be_bytes([bytes[38], bytes[39], bytes[40], bytes[41]]) as usize;
            // F-01 (R12 bounded-decode / unbounded-allocation DoS fix):
            // `count` is attacker-controlled (up to u32::MAX) and reachable
            // PRE-AUTH on the untrusted-host tier via
            // `RedbBackend::get_encrypted_node`, which feeds raw stored bytes
            // here before any AEAD decrypt / plaintext_cid integrity check.
            // A 42-byte crafted blob with `count = 0xFFFFFFFF` would force a
            // multi-GB `Vec::with_capacity` -> allocation-abort / OOM. Bound
            // `count` by what the input length can possibly encode BEFORE the
            // pre-allocation. Each chunk entry consumes AT LEAST a 4-byte
            // length prefix + a 5-byte minimal `AeadEnvelope` header (the
            // cipher-suite `from_wire_bytes` rejects `< 5` bytes) = 9 bytes.
            // So a well-formed input of length `bytes.len()` encodes at most
            // `(bytes.len() - 42) / 9` chunks; an over-count input is
            // malformed and rejected honestly (typed reject, not a silent
            // allocation abort). This restores THREAT-MODEL §6's positively
            // claimed bounded-decode ceiling on this wire-decode path.
            const MIN_CHUNK_ENTRY_LEN: usize = 4 + 5; // 4-byte len prefix + minimal AeadEnvelope header
            let max_chunks = bytes.len().saturating_sub(42) / MIN_CHUNK_ENTRY_LEN;
            if count > max_chunks {
                return Err(AeadError::ChunkCountExceedsInput {
                    count,
                    max: max_chunks,
                });
            }
            let mut cursor = 42usize;
            let mut chunks = Vec::with_capacity(count);
            for _ in 0..count {
                // F-04 (R13 per-chunk integer-overflow bounded-decode DoS):
                // `cursor + N` where N derives from wire bytes (the 4-byte
                // length prefix, then the attacker-controlled `len` itself)
                // OVERFLOW-WRAPS on 32-bit `usize` (wasm32 thin-client, shape
                // b/c). A chunk `len = 0xFFFFFFFF` wraps `cursor + len` BELOW
                // `bytes.len()`, bypassing a raw `> bytes.len()` guard, so the
                // subsequent `&bytes[cursor..cursor + len]` slice panics
                // (pre-auth DoS via `RedbBackend::get_encrypted_node`, decoded
                // before any AEAD/integrity check). Route EVERY wire-derived
                // add through `checked_range_end` (mirrors `benten-drop`'s
                // `lp_range_end` discipline): `None` on overflow OR
                // out-of-bounds → the SAME typed `CiphertextTooShort` reject.
                // No behavioral change on 64-bit (the lengths cannot overflow
                // there); the `checked_add` is what makes the guard
                // target-agnostic.
                let len_end = checked_range_end(cursor, 4, bytes.len())
                    .ok_or(AeadError::CiphertextTooShort { got: bytes.len() })?;
                let len = u32::from_be_bytes([
                    bytes[cursor],
                    bytes[cursor + 1],
                    bytes[cursor + 2],
                    bytes[cursor + 3],
                ]) as usize;
                cursor = len_end;
                let chunk_end = checked_range_end(cursor, len, bytes.len())
                    .ok_or(AeadError::CiphertextTooShort { got: bytes.len() })?;
                let envelope = AeadEnvelope::from_wire_bytes(&bytes[cursor..chunk_end])
                    .map_err(AeadError::from)?;
                cursor = chunk_end;
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

/// Build a `AeadKeyMaterial` from raw bytes for the wave-live default
/// codepoint (X-Wing-hybrid `0x647a`). Production key-derivation paths
/// already produce typed `AeadKeyMaterial` via the cipher-suite; this
/// helper is for the storage-layer surface that accepts a `&[u8]` key
/// (the `decrypt(&envelope, &key: &[u8])` shape the R3 pins use).
fn make_key_material(key: &[u8]) -> Result<AeadKeyMaterial, AeadError> {
    if key.len() != 32 {
        return Err(AeadError::KeyMismatch {
            reason: "AEAD key MUST be 32 B for ChaCha20-Poly1305 dispatch".to_string(),
        });
    }
    Ok(AeadKeyMaterial::from_raw_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        key,
    ))
}

/// Build a `AeadKeyMaterial` whose codepoint matches the envelope's. The
/// cipher-suite `unwrap` typed-rejects on codepoint mismatch; this
/// helper threads the envelope's codepoint into the key newtype so a
/// caller that presents the *correct* key for an envelope sealed under
/// a non-default codepoint (e.g. `0x6400` classical-only X25519
/// downgrade arm) gets through, while a caller with a wrong-codepoint
/// presumption surfaces `AeadError::Unsupported`.
fn make_key_material_matching(
    envelope: &EncryptedNode,
    key: &[u8],
) -> Result<AeadKeyMaterial, AeadError> {
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
    Ok(AeadKeyMaterial::from_raw_bytes(codepoint, key))
}

/// Graph-side AEAD error envelope. Maps the cipher-suite's typed errors
/// onto the storage-side surface + adds typed arms for the
/// storage-specific pre-flight cases (key length mismatch +
/// structurally-too-short ciphertext).
#[derive(Debug, Error)]
#[non_exhaustive]
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

    /// A chunked-envelope decode declared a chunk `count` larger than the
    /// input length can possibly encode (bounded-decode / unbounded-
    /// allocation DoS tripwire, F-01). The chunk count is read from
    /// attacker-controlled bytes and reached PRE-AUTH on the untrusted-host
    /// tier; a well-formed input encodes at most
    /// `(bytes.len() - 42) / 9` chunks (4-byte length prefix + 5-byte
    /// minimal `AeadEnvelope` header per chunk). An over-count input is
    /// malformed and rejected here BEFORE any pre-allocation, per
    /// THREAT-MODEL §6's bounded-decode ceiling.
    #[error("chunked-envelope chunk count {count} exceeds max encodable for input ({max})")]
    ChunkCountExceedsInput {
        /// Attacker-declared chunk count.
        count: usize,
        /// Maximum chunk count the input length can encode.
        max: usize,
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
            // L1-r6r1-MIN-1 (R6 R1): SuiteAeadError is #[non_exhaustive];
            // future-variant additions surface here as opaque
            // Authentication-class failures (fail-CLOSED default; specific
            // arm-mapping ratchets up at the wave that mints the variant).
            other => Self::Authentication(format!(
                "unmapped cipher-suite AeadError variant (post-non_exhaustive forward-compat): {other:?}"
            )),
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
        // Independent decrypt of chunk 2 (F3: total_chunks=4 must match)
        let chunk_2 = decrypt_chunk(&chunked.chunks()[2], 2, 4, &cid, &key).unwrap();
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
        let result = decrypt_chunk(&chunked.chunks()[1], 3, 4, &cid, &key);
        assert!(matches!(result, Err(AeadError::Authentication { .. })));
    }

    /// F3 (R6 R1 fix-pass): cross-chunk-truncation defense pin —
    /// presenting a chunk under a different `total_chunks` value than
    /// the seal-time value MUST fail AEAD authentication. This is the
    /// load-bearing per-chunk truncation-binding test per the
    /// retraction of G-CORE-9 R1 triage Fork 1.
    #[test]
    fn cross_chunk_truncation_fails() {
        let cid = fixed_cid(0xee);
        let key = [0x42u8; 32];
        // Encrypt under 10 chunks (total_chunks = 10 sealed into every chunk's AAD)
        let plaintext = vec![0x5Au8; 10 * IROH_BLOCK_SIZE];
        let chunked = ChunkedCiphertext::encrypt(&plaintext, &cid, &key).unwrap();
        assert_eq!(chunked.chunks().len(), 10);
        // Attacker truncates the chunk list to 5 chunks and re-presents.
        // Per-chunk authentication MUST fail at every boundary because
        // every chunk's AAD committed to total_chunks=10, not 5.
        // Try decrypting chunk 0 with the truncated total_chunks=5:
        let result = decrypt_chunk(&chunked.chunks()[0], 0, 5, &cid, &key);
        assert!(
            matches!(result, Err(AeadError::Authentication { .. })),
            "truncated total_chunks=5 (real=10) MUST fail AEAD auth, got {result:?}"
        );
        // Try decrypting chunk 4 (the truncation boundary) with the
        // truncated total_chunks=5: still fails because seal-time was 10.
        let result = decrypt_chunk(&chunked.chunks()[4], 4, 5, &cid, &key);
        assert!(
            matches!(result, Err(AeadError::Authentication { .. })),
            "truncation-boundary chunk MUST fail AEAD auth, got {result:?}"
        );
        // Positive control: presenting with the correct total_chunks=10
        // succeeds for the same chunk.
        let plaintext_chunk_0 = decrypt_chunk(&chunked.chunks()[0], 0, 10, &cid, &key).unwrap();
        assert_eq!(plaintext_chunk_0, &plaintext[0..IROH_BLOCK_SIZE]);
    }

    /// F3 negative-test pin: tampering total_chunks in the AAD-reconstruction
    /// path flips the bound bytes — decryption MUST fail. (Adjacent to
    /// the cross_chunk_truncation_fails pin; this tests the symmetric
    /// inflation case — claiming total_chunks=20 when seal-time was 10.)
    #[test]
    fn cross_chunk_inflation_fails() {
        let cid = fixed_cid(0x77);
        let key = [0x99u8; 32];
        let plaintext = vec![0xA5u8; 10 * IROH_BLOCK_SIZE];
        let chunked = ChunkedCiphertext::encrypt(&plaintext, &cid, &key).unwrap();
        let result = decrypt_chunk(&chunked.chunks()[0], 0, 20, &cid, &key);
        assert!(
            matches!(result, Err(AeadError::Authentication { .. })),
            "inflated total_chunks=20 (real=10) MUST fail AEAD auth, got {result:?}"
        );
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

    /// F-01 (R12 bounded-decode / unbounded-allocation DoS): a 42-byte
    /// crafted chunked-envelope blob declaring `count = 0xFFFFFFFF` MUST
    /// be rejected with the typed [`AeadError::ChunkCountExceedsInput`]
    /// reject BEFORE the pre-allocation, NOT attempt a
    /// `Vec::with_capacity(u32::MAX)` (which would abort the process /
    /// OOM). This blob is reachable PRE-AUTH via
    /// `RedbBackend::get_encrypted_node`. Without the ceiling guard this
    /// call would attempt `Vec::with_capacity(0xFFFF_FFFF)`.
    #[test]
    fn chunk_count_overflow_is_typed_reject_not_alloc_abort() {
        let cid = fixed_cid(0x01);
        // STORAGE_MAGIC ‖ 0x01 (chunked) ‖ <36-byte CID> ‖ count=0xFFFFFFFF
        let mut blob = Vec::with_capacity(42);
        blob.push(0x3d); // STORAGE_MAGIC
        blob.push(0x01); // variant tag: Chunked
        blob.extend_from_slice(cid.as_bytes()); // 36 bytes
        blob.extend_from_slice(&u32::MAX.to_be_bytes()); // count = 0xFFFFFFFF
        assert_eq!(blob.len(), 42, "crafted blob is the minimal 42-byte header");
        let result = decode_encrypted_node(&blob);
        assert!(
            matches!(
                result,
                Err(AeadError::ChunkCountExceedsInput {
                    count: 0xFFFF_FFFF,
                    max: 0
                })
            ),
            "42-byte blob with count=u32::MAX MUST typed-reject, got {result:?}"
        );
    }

    /// F-01 boundary: `count == max_chunks` (well-formed) decodes OK;
    /// `count == max_chunks + 1` rejects. Uses a real chunked encode to
    /// get valid chunk bytes, then rewrites only the 4-byte count field.
    #[test]
    fn chunk_count_boundary_at_max_encodable() {
        let cid = fixed_cid(0x03);
        let key = [0x55u8; 32];
        // 3 × IROH_BLOCK_SIZE = 48 KiB... below the 64 KiB whole-threshold,
        // so build the chunked container directly for a multi-chunk case.
        let plaintext = vec![0x11u8; 3 * IROH_BLOCK_SIZE];
        let chunked = ChunkedCiphertext::encrypt(&plaintext, &cid, &key).unwrap();
        let node = EncryptedNode::Chunked {
            plaintext_cid: cid,
            chunked,
        };
        let encoded = encode_encrypted_node(&node).unwrap();
        // Positive control: the honest encode round-trips.
        assert!(matches!(
            decode_encrypted_node(&encoded),
            Ok(EncryptedNode::Chunked { .. })
        ));

        // max_chunks = (len - 42) / 9 for this input.
        let max_chunks = encoded.len().saturating_sub(42) / (4 + 5);
        assert!(
            max_chunks >= 3,
            "input can encode at least its 3 real chunks"
        );

        // count == max_chunks: passes the ceiling guard (the per-chunk
        // bounds loop then rejects mid-stream once the real bytes run out,
        // but the point here is the ceiling guard does NOT fire).
        let mut at_max = encoded.clone();
        at_max[38..42].copy_from_slice(&(max_chunks as u32).to_be_bytes());
        assert!(
            !matches!(
                decode_encrypted_node(&at_max),
                Err(AeadError::ChunkCountExceedsInput { .. })
            ),
            "count == max_chunks MUST pass the ceiling guard"
        );

        // count == max_chunks + 1: the ceiling guard fires.
        let mut over = encoded;
        let over_count = (max_chunks + 1) as u32;
        over[38..42].copy_from_slice(&over_count.to_be_bytes());
        assert!(
            matches!(
                decode_encrypted_node(&over),
                Err(AeadError::ChunkCountExceedsInput { .. })
            ),
            "count == max_chunks + 1 MUST typed-reject via the ceiling guard"
        );
    }

    /// F-04 (R13 per-chunk integer-overflow bounded-decode DoS): a chunked
    /// blob with `count = 1` and a single chunk whose 4-byte length prefix
    /// declares `len = 0xFFFFFFFF` MUST typed-reject (`CiphertextTooShort`),
    /// NOT panic. On a 32-bit `usize` target (wasm32 thin-client, shape
    /// b/c) `cursor + len` would OVERFLOW-WRAP below `bytes.len()`,
    /// bypassing a raw `> bytes.len()` guard and reaching a panicking
    /// `&bytes[cursor..cursor + len]` slice (pre-auth DoS via
    /// `RedbBackend::get_encrypted_node`).
    ///
    /// HONEST NOTE: on the 64-bit `usize` this test runs under, `cursor +
    /// 0xFFFFFFFF` does NOT overflow, so the pre-existing `end <=
    /// bytes.len()` check already rejects — this pin asserts the typed-
    /// reject BEHAVIOR is preserved. The 32-bit overflow safety is provided
    /// by `checked_range_end`'s `checked_add` (target-agnostic), pinned
    /// directly by `checked_range_end_returns_none_on_overflow` below.
    /// Would-FAIL-on-revert: reverting to `cursor + len > bytes.len()` keeps
    /// this 64-bit assertion GREEN but re-introduces the 32-bit wrap; the
    /// checked-helper unit test is the target-agnostic regression guard.
    #[test]
    fn per_chunk_len_overflow_is_typed_reject_not_panic() {
        let cid = fixed_cid(0x07);
        // STORAGE_MAGIC ‖ 0x01 ‖ <36-byte CID> ‖ count=1 ‖ chunk_len=0xFFFFFFFF
        let mut blob = Vec::with_capacity(46);
        blob.push(0x3d); // STORAGE_MAGIC
        blob.push(0x01); // variant: Chunked
        blob.extend_from_slice(cid.as_bytes()); // 36 bytes
        blob.extend_from_slice(&1u32.to_be_bytes()); // count = 1 (passes ceiling: (46-42)/9 == 0? -> ceiling first)
        blob.extend_from_slice(&u32::MAX.to_be_bytes()); // chunk len = 0xFFFFFFFF
        // NOTE: with these 46 bytes, max_chunks = (46-42)/9 = 0, so the
        // COUNT ceiling actually fires first for count=1. Pad the blob so
        // the ceiling admits 1 chunk, forcing the loop to read the poisoned
        // per-chunk length and exercise the per-chunk overflow guard.
        blob.extend_from_slice(&[0u8; 9]); // +9 bytes -> len 55 -> max_chunks=(55-42)/9=1
        assert!(
            blob.len() >= 46,
            "blob carries the count + poisoned per-chunk length prefix"
        );
        let result = decode_encrypted_node(&blob);
        assert!(
            matches!(result, Err(AeadError::CiphertextTooShort { .. })),
            "per-chunk len=0xFFFFFFFF MUST typed-reject (not panic), got {result:?}"
        );
    }

    /// F-04 target-agnostic guard: `checked_range_end` returns `None` on
    /// integer overflow (the 32-bit wrap the per-chunk pin cannot exercise
    /// on a 64-bit host) AND on out-of-bounds, and `Some(end)` only for a
    /// valid in-bounds range. This is the load-bearing regression guard —
    /// it fails on any host if the `checked_add` is reverted to a raw `+`.
    #[test]
    fn checked_range_end_returns_none_on_overflow() {
        // Overflow: off + len wraps past usize::MAX.
        assert_eq!(checked_range_end(usize::MAX - 2, 5, usize::MAX), None);
        assert_eq!(checked_range_end(10, usize::MAX, 100), None);
        // Out-of-bounds (no overflow): end > total.
        assert_eq!(checked_range_end(90, 20, 100), None);
        // Valid in-bounds range.
        assert_eq!(checked_range_end(10, 20, 100), Some(30));
        // Exact end == total is in-bounds.
        assert_eq!(checked_range_end(80, 20, 100), Some(100));
    }
}
