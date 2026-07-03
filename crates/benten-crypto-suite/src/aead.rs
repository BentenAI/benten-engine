//! AEAD substrate for the cipher-suite (G-CORE-3a CANARY).
//!
//! # Surface
//!
//! - [`AeadKeyMaterial`] — the symmetric key newtype carried into / out of
//!   the X-Wing-hybrid wrap (the `K_root` / `K(N)` material). Codepoint-
//!   tagged so downstream consumers know which suite produced it.
//! - [`AeadEnvelope`] — the serializable AEAD ciphertext envelope
//!   carrying a `format_version: u8` discriminator + `cipher_codepoint:
//!   u16` + nonce + ciphertext-with-tag. Per §1.A.FROZEN item 15(g):
//!   AAD-binds-plaintext-CID + (for per-chunk AEAD) chunk-index for
//!   rebinding-attack defense.
//! - [`wrap`] / [`unwrap`] — production seal/open API over
//!   ChaCha20-Poly1305 with a key dispatched through [`AeadKeyMaterial`].
//! - [`AeadError`] — typed error envelope: `AeadAuthFailed` +
//!   `MalformedEnvelope` + `Unsupported` (codepoint-mismatch).
//!
//! # Wire format (G-CORE-9 freezes; canonical at G-CORE-3a)
//!
//! ```text
//! byte 0   : magic 0xae        (envelope identifier; Varsig sibling)
//! byte 1   : format_version    (v1-beta = 0x01)
//! bytes 2-3: cipher codepoint  (BE u16; e.g. 0x647a for X-Wing-hybrid; M-19)
//! byte 4   : nonce_len         (12 for ChaCha20-Poly1305)
//! bytes 5..: nonce || ciphertext_with_tag
//! ```
//!
//! Per Gate-6 (R4-FP-1 m-7 closure): the explicit format-version byte
//! at position 1 makes the envelope distinguishable from a future v2
//! envelope WITHOUT pre-coordination (the multiformats-permanent-
//! commitment property per CLAUDE.md baked-in #5).
//!
//! # Chunking (per §1.A.FROZEN item 15(g))
//!
//! Per-chunk-AEAD chunk-size = `IROH_BLOCK_SIZE` (16 KiB) for Nodes ≥
//! 64 KiB. Whole-content-AEAD for Nodes < 64 KiB. AAD binds (plaintext-
//! CID, chunk-index) for rebinding-attack defense. This wave ships the
//! single-shot whole-content surface; per-chunk AEAD support lands at
//! G-CORE-3d (graph-AEAD layer). The chunk-index field lives in the
//! AAD-builder helpers here so G-CORE-3d can call into them.

use chacha20poly1305::aead::{Aead as _, AeadCore as _, KeyInit as _, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce};
use thiserror::Error;
use zeroize::Zeroize;

use crate::codepoint::CipherSuiteCodepoint;

/// Per `00-implementation-plan.md` §1.A.FROZEN item 15(g): per-chunk
/// AEAD chunk-size MUST equal `iroh-blobs::IROH_BLOCK_SIZE` (16 KiB).
/// Exposed here so G-CORE-3d can pin chunk-size against it.
pub const IROH_BLOCK_SIZE: usize = 16 * 1024;

/// Per §1.A.FROZEN item 15(g): whole-content AEAD for Nodes < 64 KiB;
/// per-chunk AEAD ≥ 64 KiB.
pub const WHOLE_CONTENT_AEAD_THRESHOLD: usize = 64 * 1024;

/// The AEAD envelope's format-version discriminator (G-CORE-9 may
/// re-numerate at the wire-freeze pass; until then v1-beta = 0x01).
pub const ENVELOPE_FORMAT_VERSION_V1: u8 = 0x01;

/// Magic byte identifying a Benten AEAD envelope (Varsig-style
/// multiformats sibling at `0xae`; Varsig uses `0xb5`).
pub const ENVELOPE_MAGIC: u8 = 0xae;

/// ChaCha20-Poly1305 nonce size — 12 bytes (96 bits) per RFC 8439.
/// **Not** a hardcoded crypto-primitive size in the CLAUDE.md #5 sense;
/// this is the algorithm-parameter-fixed nonce size for the codepoint
/// dispatched here. Future codepoints with different AEADs would dispatch
/// their own nonce-size at construction time.
const CHACHA20POLY1305_NONCE_LEN: usize = 12;

/// Symmetric key material — the K_root / K(N) byte vector carried into
/// / out of the X-Wing-hybrid wrap.
///
/// The byte length is dispatched by `cipher_codepoint` — at G-CORE-3a's
/// `HYBRID_X25519_MLKEM768` codepoint the wrapped key is the
/// SHA3-256-combiner-derived 32-B ChaCha20-Poly1305 key (the X-Wing
/// combiner output per `draft-connolly-cfrg-xwing-kem-10` §6;
/// [`crate::cipher_suite::combine_x_wing`] — NOT HKDF). **NOT a hardcoded
/// size in the CLAUDE.md #5 sense** — the codepoint surface enforces
/// the dispatch (a future codepoint at a different AEAD would carry
/// a different key length).
///
/// Zeroizes on drop.
#[derive(Clone)]
pub struct AeadKeyMaterial {
    /// The cipher-suite codepoint that produced this key material.
    pub codepoint: CipherSuiteCodepoint,
    /// The raw key bytes (32 B for the X-Wing-hybrid + ChaCha20-Poly1305
    /// dispatch this wave ships).
    bytes: Vec<u8>,
}

impl AeadKeyMaterial {
    /// Construct from raw bytes + codepoint. Crate-public for cipher-
    /// suite internals + the public `from_raw_bytes` constructor below.
    pub(crate) fn from_bytes(codepoint: CipherSuiteCodepoint, bytes: Vec<u8>) -> Self {
        Self { codepoint, bytes }
    }

    /// Construct from raw bytes + codepoint. No validation of byte length
    /// against the codepoint's expected AEAD key length (the codepoint's
    /// AEAD wrap path performs that check at use-time). Production-safe
    /// constructor for swap-matrix `sign_and_seal` / `open_and_verify`
    /// hot paths (G-CORE-3c) where `&[u8]` from caller-owned buffers is
    /// the natural input shape, AND for per-test K_ROOT vector
    /// injection.
    ///
    /// Renamed from `from_bytes_for_test` at G-CORE-3c fix-pass (mr-major-2)
    /// after the test-named API was identified as called from production
    /// hot paths — see `.addl/phase-4-meta/r5-g-core-3c-mini-review.json`.
    #[must_use]
    pub fn from_raw_bytes(codepoint: CipherSuiteCodepoint, bytes: &[u8]) -> Self {
        Self {
            codepoint,
            bytes: bytes.to_vec(),
        }
    }

    /// Raw key bytes (test-only inspection + AEAD-key feed).
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The codepoint discriminator carried alongside the key.
    #[must_use]
    pub const fn codepoint(&self) -> CipherSuiteCodepoint {
        self.codepoint
    }
}

impl Drop for AeadKeyMaterial {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

/// AEAD envelope — the serializable wire-format for a sealed payload.
///
/// Carries a format-version discriminator + cipher codepoint + nonce +
/// ciphertext-with-tag. The codepoint disciplines bidirectional
/// dispatch: peers that see an unknown codepoint surface typed
/// [`AeadError::Unsupported`] (never silent-fallback).
#[derive(Clone, Debug)]
pub struct AeadEnvelope {
    /// Format-version discriminator (Gate-6 R4-FP-1 m-7 contract).
    pub format_version: u8,
    /// Cipher-suite codepoint (which combiner / AEAD produced this).
    pub cipher_codepoint: CipherSuiteCodepoint,
    /// AEAD nonce (12 B for ChaCha20-Poly1305).
    pub nonce: Vec<u8>,
    /// Ciphertext bytes including the 16-B Poly1305 authentication tag.
    pub ciphertext: Vec<u8>,
}

impl AeadEnvelope {
    /// Serialize to wire bytes per the canonical format documented in
    /// this module's header. Gate-6 conformance pin asserts byte-1 is
    /// the format-version discriminator.
    #[must_use]
    pub fn to_wire_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(5 + self.nonce.len() + self.ciphertext.len());
        out.push(ENVELOPE_MAGIC);
        out.push(self.format_version);
        // M-19: codepoint serialized BIG-ENDIAN (network byte order per
        // RFC 9180 / FIPS 203 / MLS). Migrated from LE at F-full Wave-0.
        out.extend_from_slice(&self.cipher_codepoint.raw().to_be_bytes());
        // nonce-length-byte at position 4 — a single byte (≤ 255) is
        // sufficient for all AEADs we dispatch (ChaCha20-Poly1305 = 12).
        out.push(
            u8::try_from(self.nonce.len())
                .expect("AEAD nonce length fits in u8 (ChaCha20-Poly1305 = 12)"),
        );
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.ciphertext);
        out
    }

    /// Parse from wire bytes. Returns typed
    /// [`AeadError::MalformedEnvelope`] on shape violations + typed
    /// [`AeadError::Unsupported`] on unknown codepoints.
    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self, AeadError> {
        if bytes.len() < 5 {
            return Err(AeadError::MalformedEnvelope(
                "envelope too short for header",
            ));
        }
        if bytes[0] != ENVELOPE_MAGIC {
            return Err(AeadError::MalformedEnvelope("bad envelope magic byte"));
        }
        let format_version = bytes[1];
        let cipher_codepoint =
            CipherSuiteCodepoint::from_raw(u16::from_be_bytes([bytes[2], bytes[3]]));
        let nonce_len = usize::from(bytes[4]);
        if bytes.len() < 5 + nonce_len {
            return Err(AeadError::MalformedEnvelope("envelope truncated mid-nonce"));
        }
        let nonce = bytes[5..5 + nonce_len].to_vec();
        let ciphertext = bytes[5 + nonce_len..].to_vec();
        Ok(Self {
            format_version,
            cipher_codepoint,
            nonce,
            ciphertext,
        })
    }
}

/// Whole-content AEAD AAD domain-separation info string. A registered
/// cross-surface domain-separation tag mirrored in the central
/// [`crate::domain_registry::AEAD_WHOLE_CONTEXT`] corpus table over which the
/// prefix-free invariant runs (intra-crate `domain_registry_mirror` test pins
/// byte-equality). Canonical home is HERE.
pub const AEAD_WHOLE_CONTEXT: &[u8] = b"benten-aead:whole:";

/// Per-chunk AEAD AAD domain-separation info string (registered tag; mirrored
/// in [`crate::domain_registry::AEAD_CHUNK_CONTEXT`]). Canonical home is HERE.
pub const AEAD_CHUNK_CONTEXT: &[u8] = b"benten-aead:chunk:";

/// Per-Recipe AEAD AAD domain-separation info string (registered tag; mirrored
/// in [`crate::domain_registry::AEAD_RECIPE_CONTEXT`]). Canonical home is HERE.
pub const AEAD_RECIPE_CONTEXT: &[u8] = b"benten-aead:recipe:";

/// Build the canonical AAD for whole-content AEAD: binds plaintext-CID
/// (item 15(g) AAD-binds-plaintext-CID contract).
#[must_use]
pub fn aad_whole_content(plaintext_cid: &[u8]) -> Vec<u8> {
    let mut aad = Vec::with_capacity(AEAD_WHOLE_CONTEXT.len() + plaintext_cid.len());
    aad.extend_from_slice(AEAD_WHOLE_CONTEXT);
    aad.extend_from_slice(plaintext_cid);
    aad
}

/// Build the canonical AAD for per-chunk AEAD: binds
/// `(plaintext-CID, chunk_index, total_chunks)` per item 15(g)
/// cross-chunk-rebinding-attack defense + cross-chunk-truncation defense
/// (Bundle F3, R6 R1 fix-pass — supersedes the 2-tuple as-shipped layout
/// per the retraction of G-CORE-9 R1 triage Fork 1).
///
/// The `total_chunks` binding closes the truncation attack: an attacker
/// who slices the chunk list (e.g. mounts the first 5 chunks of a
/// 10-chunk ciphertext as a freshly-encoded 5-chunk ciphertext) cannot
/// fabricate per-chunk AAD-matching tags because every chunk's AAD
/// committed to the seal-time `total_chunks=10` value; the producer at
/// the smaller list builds AAD over `total_chunks=5` and per-chunk AEAD
/// authentication fails at every chunk boundary.
///
/// **Wire-format note:** this is the v1-beta canonical layout per the
/// R6 R1 retraction of Fork 1. The 2-tuple layout
/// `(plaintext_cid, chunk_index)` shipped earlier is REPLACED. The
/// `chunk_index: u64` + `total_chunks: u32` fields are appended
/// big-endian (M-19; migrated from LE at F-full Wave-0). Encoders MUST
/// fail-CLOSED if `total_chunks` exceeds `u32::MAX`.
///
/// G-CORE-3d's graph-AEAD layer threads this through every per-chunk
/// seal + unwrap call site.
#[must_use]
pub fn aad_per_chunk(plaintext_cid: &[u8], chunk_index: u64, total_chunks: u32) -> Vec<u8> {
    let mut aad = Vec::with_capacity(AEAD_CHUNK_CONTEXT.len() + plaintext_cid.len() + 8 + 4);
    aad.extend_from_slice(AEAD_CHUNK_CONTEXT);
    aad.extend_from_slice(plaintext_cid);
    // M-19: chunk_index + total_chunks BIG-ENDIAN (migrated from LE at
    // F-full Wave-0). The integer bytes are the discriminating suffix.
    aad.extend_from_slice(&chunk_index.to_be_bytes());
    aad.extend_from_slice(&total_chunks.to_be_bytes());
    aad
}

/// Build the canonical AAD for per-Recipe AEAD within a multi-Recipe
/// container (e.g. a `DropBundle`'s `content: Vec<EncryptedContent>`):
/// binds `(plaintext-CID, recipe_index, total_recipes)`.
///
/// **R6 R2 fix-pass (Bundle R6-R2-FP-A L4 sibling):** closes the
/// inter-Recipe truncation attack. Pre-fix, each Recipe was sealed
/// with `aad_whole_content(plaintext_cid)` only — an attacker could
/// drop one Recipe from `bundle.content` and the remaining Recipes'
/// AEAD tags still verified individually because the per-Recipe seal
/// committed neither to its position in the list nor to the list
/// length. With `recipe_index` + `total_recipes` bound via this AAD,
/// per-Recipe authentication fails if an attacker presents a sliced
/// list (the per-Recipe AAD committed at seal time names the
/// original `total_recipes` count + each Recipe's original position).
///
/// Shape mirrors [`aad_per_chunk`] exactly: distinct domain-separator
/// prefix (`benten-aead:recipe:`) so a per-Recipe seal can NEVER be
/// reinterpreted as a per-chunk seal or a whole-content seal. The
/// position + total are encoded big-endian (M-19; migrated from LE at
/// F-full Wave-0).
///
/// **Wire-format note:** this is the v1-beta canonical layout for the
/// DropBundle per-Recipe AAD. Encoders MUST fail-CLOSED if
/// `total_recipes` exceeds `u32::MAX`.
#[must_use]
pub fn aad_per_recipe(plaintext_cid: &[u8], recipe_index: u32, total_recipes: u32) -> Vec<u8> {
    let mut aad = Vec::with_capacity(AEAD_RECIPE_CONTEXT.len() + plaintext_cid.len() + 4 + 4);
    aad.extend_from_slice(AEAD_RECIPE_CONTEXT);
    aad.extend_from_slice(plaintext_cid);
    // M-19: recipe_index + total_recipes BIG-ENDIAN (migrated from LE at
    // F-full Wave-0).
    aad.extend_from_slice(&recipe_index.to_be_bytes());
    aad.extend_from_slice(&total_recipes.to_be_bytes());
    aad
}

/// Wrap (seal) `plaintext` under `key` + bind `aad` via
/// ChaCha20-Poly1305. The returned [`AeadEnvelope`] carries the
/// format-version discriminator + cipher codepoint so decrypt-side can
/// dispatch.
///
/// Per CLAUDE.md baked-in #5: typed-unsupported (NEVER silent-fallback)
/// on a key whose codepoint doesn't match an AEAD dispatch arm.
pub fn wrap(
    plaintext: &[u8],
    key: &AeadKeyMaterial,
    aad: &[u8],
) -> Result<AeadEnvelope, AeadError> {
    // Dispatch on the key's codepoint. At G-CORE-3a only the X25519⊕
    // ML-KEM-768 hybrid codepoint is live (its derived K_root feeds
    // ChaCha20-Poly1305); the classical-only X25519 downgrade arm also
    // routes here (the AEAD is the same; the difference is the wrap
    // path that produced the key).
    match key.codepoint.raw() {
        0x647a | 0x6400 => {
            if key.bytes.len() != 32 {
                return Err(AeadError::MalformedEnvelope(
                    "ChaCha20-Poly1305 key MUST be 32 B",
                ));
            }
            let chacha_key = ChaChaKey::from_slice(&key.bytes);
            let cipher = ChaCha20Poly1305::new(chacha_key);
            let nonce_bytes = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            let nonce = Nonce::from_slice(&nonce_bytes);
            let ciphertext = cipher
                .encrypt(
                    nonce,
                    chacha20poly1305::aead::Payload {
                        msg: plaintext,
                        aad,
                    },
                )
                .map_err(|_| AeadError::AeadAuthFailed)?;
            Ok(AeadEnvelope {
                format_version: ENVELOPE_FORMAT_VERSION_V1,
                cipher_codepoint: key.codepoint,
                nonce: nonce_bytes.to_vec(),
                ciphertext,
            })
        }
        other => Err(AeadError::Unsupported(
            crate::error::UnsupportedAlgorithm::CipherSuite { codepoint: other },
        )),
    }
}

/// Unwrap (open) an envelope under `key` + `aad`. Returns typed
/// [`AeadError::AeadAuthFailed`] on tag-mismatch (including rebinding
/// attacks where `aad` doesn't match the seal-time AAD).
pub fn unwrap(
    envelope: &AeadEnvelope,
    key: &AeadKeyMaterial,
    aad: &[u8],
) -> Result<Vec<u8>, AeadError> {
    // Codepoint-mismatch is a silent-downgrade vector — typed-reject.
    if envelope.cipher_codepoint != key.codepoint {
        return Err(AeadError::Unsupported(
            crate::error::UnsupportedAlgorithm::CipherSuite {
                codepoint: envelope.cipher_codepoint.raw(),
            },
        ));
    }
    if envelope.format_version != ENVELOPE_FORMAT_VERSION_V1 {
        return Err(AeadError::MalformedEnvelope(
            "unsupported envelope format-version",
        ));
    }
    match key.codepoint.raw() {
        0x647a | 0x6400 => {
            if key.bytes.len() != 32 {
                return Err(AeadError::MalformedEnvelope(
                    "ChaCha20-Poly1305 key MUST be 32 B",
                ));
            }
            if envelope.nonce.len() != CHACHA20POLY1305_NONCE_LEN {
                return Err(AeadError::MalformedEnvelope(
                    "ChaCha20-Poly1305 nonce MUST be 12 B",
                ));
            }
            let chacha_key = ChaChaKey::from_slice(&key.bytes);
            let cipher = ChaCha20Poly1305::new(chacha_key);
            let nonce = Nonce::from_slice(&envelope.nonce);
            cipher
                .decrypt(
                    nonce,
                    chacha20poly1305::aead::Payload {
                        msg: &envelope.ciphertext,
                        aad,
                    },
                )
                .map_err(|_| AeadError::AeadAuthFailed)
        }
        other => Err(AeadError::Unsupported(
            crate::error::UnsupportedAlgorithm::CipherSuite { codepoint: other },
        )),
    }
}

/// AEAD-side typed error envelope.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AeadError {
    /// AEAD authentication failed — tag-mismatch or rebinding (the AAD
    /// doesn't match what the seal-time AAD bound). The load-bearing
    /// rebinding-attack defense per §1.A.FROZEN item 15(g).
    #[error("AEAD authentication failed (tag mismatch or AAD rebinding)")]
    AeadAuthFailed,

    /// The envelope's wire shape is malformed (truncated, bad magic,
    /// wrong nonce length, etc).
    #[error("malformed envelope: {0}")]
    MalformedEnvelope(&'static str),

    /// The recipient lacks one of the required key halves for the
    /// hybrid suite (e.g. classical-only recipient handed a hybrid
    /// codepoint). The named typed-arm per the F-3 W1 spec-gap. Surfaced
    /// from the wrap/unwrap path when the codepoint requires keys the
    /// recipient doesn't have.
    #[error("recipient lacks one of the required key halves for the dispatched cipher-suite")]
    RecipientLacksKeysForSuite,

    /// A serialized [`crate::cipher_suite::RecipientPublic`] byte blob was
    /// malformed for its codepoint (wrong total length / truncated ML-KEM
    /// encapsulation key). Fail-closed typed-reject on
    /// `RecipientPublic::from_bytes` — never a silent default (CLAUDE.md
    /// baked-in #5 typed-reject-on-malformed).
    #[error("malformed serialized recipient public material: {0}")]
    MalformedRecipientPublic(&'static str),

    /// A serialized [`crate::cipher_suite::RecipientSecret`] byte blob was
    /// malformed for its codepoint (wrong total length / truncated ML-KEM
    /// decapsulation key). Fail-closed typed-reject on
    /// `RecipientSecret::from_bytes` — never a silent default (CLAUDE.md
    /// baked-in #5 typed-reject-on-malformed).
    #[error("malformed serialized recipient secret material: {0}")]
    MalformedRecipientSecret(&'static str),

    /// Codepoint dispatch surfaced typed-unsupported (NEVER silent
    /// fallback per CLAUDE.md baked-in #5).
    #[error(transparent)]
    Unsupported(#[from] crate::error::UnsupportedAlgorithm),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drift defense: the three chunked-AEAD info strings are registered
    /// cross-surface domain-separation tags in the central
    /// [`crate::domain_registry`] table over which the prefix-free invariant
    /// runs. Pin byte-equality so a mirror can never silently diverge from the
    /// home definitions here.
    #[test]
    fn aead_contexts_match_central_registry() {
        use crate::domain_registry as reg;
        assert_eq!(
            AEAD_WHOLE_CONTEXT,
            reg::AEAD_WHOLE_CONTEXT,
            "AEAD_WHOLE_CONTEXT drifted from the central domain_registry mirror"
        );
        assert_eq!(
            AEAD_CHUNK_CONTEXT,
            reg::AEAD_CHUNK_CONTEXT,
            "AEAD_CHUNK_CONTEXT drifted from the central domain_registry mirror"
        );
        assert_eq!(
            AEAD_RECIPE_CONTEXT,
            reg::AEAD_RECIPE_CONTEXT,
            "AEAD_RECIPE_CONTEXT drifted from the central domain_registry mirror"
        );
    }

    #[test]
    fn wrap_unwrap_round_trips_at_hybrid_codepoint() {
        let key = AeadKeyMaterial::from_raw_bytes(
            CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
            &[0x42; 32],
        );
        let pt = b"hello, hybrid";
        let cid = b"plaintext-cid-A";
        let aad = aad_whole_content(cid);
        let env = wrap(pt, &key, &aad).expect("wrap MUST succeed");
        let recovered = unwrap(&env, &key, &aad).expect("unwrap MUST succeed");
        assert_eq!(recovered, pt);
    }

    #[test]
    fn wrap_unwrap_round_trips_at_classical_codepoint() {
        let key =
            AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::CLASSICAL_X25519, &[0x77; 32]);
        let pt = b"hello, classical";
        let aad = aad_whole_content(b"plaintext-cid-B");
        let env = wrap(pt, &key, &aad).expect("wrap MUST succeed");
        let recovered = unwrap(&env, &key, &aad).expect("unwrap MUST succeed");
        assert_eq!(recovered, pt);
    }

    #[test]
    fn aad_rebinding_fails_closed() {
        let key = AeadKeyMaterial::from_raw_bytes(
            CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
            &[0x99; 32],
        );
        let pt = b"payload";
        let aad_a = aad_whole_content(b"cid-A");
        let aad_b = aad_whole_content(b"cid-B");
        let env = wrap(pt, &key, &aad_a).expect("wrap MUST succeed");
        let outcome = unwrap(&env, &key, &aad_b);
        assert!(
            matches!(outcome, Err(AeadError::AeadAuthFailed)),
            "AAD-rebinding MUST fail with AeadAuthFailed (got {outcome:?})"
        );
    }

    #[test]
    fn wire_format_carries_explicit_format_version_byte() {
        let key = AeadKeyMaterial::from_raw_bytes(
            CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
            &[0x10; 32],
        );
        let env = wrap(b"pt", &key, &aad_whole_content(b"cid")).expect("wrap MUST succeed");
        let bytes = env.to_wire_bytes();
        assert_eq!(bytes[0], ENVELOPE_MAGIC, "byte 0 = envelope magic");
        assert_eq!(
            bytes[1], ENVELOPE_FORMAT_VERSION_V1,
            "byte 1 = format-version discriminator (Gate-6)"
        );
        let cp = u16::from_be_bytes([bytes[2], bytes[3]]);
        assert_eq!(
            cp,
            CipherSuiteCodepoint::HYBRID_X25519_MLKEM768.raw(),
            "bytes 2-3 = cipher codepoint BE (M-19)"
        );
    }

    #[test]
    fn round_trip_through_wire_bytes() {
        let key = AeadKeyMaterial::from_raw_bytes(
            CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
            &[0x33; 32],
        );
        let pt = b"wire round-trip payload";
        let aad = aad_whole_content(b"cid-roundtrip");
        let env = wrap(pt, &key, &aad).expect("wrap MUST succeed");
        let wire = env.to_wire_bytes();
        let parsed = AeadEnvelope::from_wire_bytes(&wire).expect("parse MUST succeed");
        let recovered = unwrap(&parsed, &key, &aad).expect("unwrap MUST succeed");
        assert_eq!(recovered, pt);
    }

    #[test]
    fn iroh_block_size_is_16_kib() {
        assert_eq!(IROH_BLOCK_SIZE, 16 * 1024);
    }

    #[test]
    fn whole_content_threshold_is_64_kib() {
        assert_eq!(WHOLE_CONTENT_AEAD_THRESHOLD, 64 * 1024);
    }
}
