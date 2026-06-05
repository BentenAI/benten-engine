//! **F-LB-2 — per-chunk / per-Recipe AAD truncation defense, BE-migrated.**
//! (merges CE-F2 + WF-C3)
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-6 F-LB-2 ("extends
//!     `tf3d_inter_recipe_truncation_rejected.rs`; `aad_per_chunk(plaintext_cid,
//!     idx, total)` + `aad_per_recipe(...)` bind position+count; sliced list
//!     fails AEAD; domain-prefixes `benten-aead:chunk:` vs `:recipe:`; all
//!     index/count **BE** (F-W0-3 dep)").
//!   - R0 §3.2 / §4.1 / §9.1 ; U3 (canonical-TLV length-injective) / U17.
//!   - R0 §4.1 BE row + §4.1.MIGRATION (M-19/M-20): "ALL multiformats-framed
//!     integer wire/AAD fields → BE" — the in-tree `aad_per_chunk`/
//!     `aad_per_recipe` (`aead.rs:244,277`) currently emit **LE**; this family
//!     pins the **BE** post-migration shape from the first commit (M-20).
//!
//! # What this pins (FREEZE-GATING — the AAD truncation/substitution contract)
//!
//!   1. `aad_per_chunk` binds `(plaintext_cid, chunk_index, total_chunks)` with
//!      ALL integers **big-endian** — a re-serialize of the same tuple is
//!      byte-identical to the pinned BE reference (the LE in-tree shape is the
//!      red-phase failer);
//!   2. truncation: a list sealed at `total=10` cannot be presented as `total=5`
//!      — the count is AAD-bound, so AEAD-open over the re-counted AAD FAILS;
//!   3. cross-prefix reinterpretation: a per-CHUNK seal can NEVER be opened as a
//!      per-RECIPE seal — the domain prefixes (`benten-aead:chunk:` vs
//!      `benten-aead:recipe:`) are load-bearing (a swapped prefix → AAD mismatch
//!      → open fails).
//!
//! # M-20 / Wave-0 DAG edge — authored against BE from the first commit
//!
//! The in-tree `aad_per_chunk`/`aad_per_recipe` emit LE (`aead.rs:244,277`).
//! This family pins the **BE** layout the W0 migration installs. The red-phase
//! failure for an un-migrated tree is exactly "still LE". This file does NOT
//! depend on W0's module (wave-independence); it pins the BE bytes via a LOCAL
//! BE-emitting stub so R5 can swap in the migrated `aead::aad_per_chunk`.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! The stub `f_lb_2_stub::aad_per_chunk_be` emits the CORRECT BE layout (so the
//! BE-reference pin is a real, meaningful assertion) but the
//! `aead_open_with_aad` stub deliberately IGNORES the AAD (the truncation/
//! substitution bug) so the negative pins FAIL until R5 wires the real
//! AAD-bound AEAD open. R5 DELETEs the stub + wires the LIVE migrated
//! `benten_crypto_suite::aead::{aad_per_chunk, aad_per_recipe, open}`,
//! un-ignores, verifies green.

#![allow(dead_code)]


// R5: wired to the LIVE migrated `aead::{aad_per_chunk, aad_per_recipe}` (now
// BE per M-19) + the real AAD-bound ChaCha20-Poly1305 seal/open.
use benten_crypto_suite::aead::{
    AeadKeyMaterial, aad_per_chunk, aad_per_recipe, unwrap as aead_unwrap, wrap as aead_wrap,
};
use benten_crypto_suite::codepoint::CipherSuiteCodepoint;

/// BE per-chunk AAD via the LIVE migrated `aead::aad_per_chunk`.
fn aad_per_chunk_be(plaintext_cid: &[u8], chunk_index: u64, total_chunks: u32) -> Vec<u8> {
    aad_per_chunk(plaintext_cid, chunk_index, total_chunks)
}

/// BE per-Recipe AAD via the LIVE migrated `aead::aad_per_recipe`.
fn aad_per_recipe_be(plaintext_cid: &[u8], recipe_index: u32, total_recipes: u32) -> Vec<u8> {
    aad_per_recipe(plaintext_cid, recipe_index, total_recipes)
}

/// A real AEAD seal carrying the ChaCha20-Poly1305 envelope under a fixed key.
struct SealedChunk {
    envelope: benten_crypto_suite::aead::AeadEnvelope,
}

/// Fixed 32-byte key for the hermetic AAD pins.
fn fixture_key() -> AeadKeyMaterial {
    AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, &[0x42u8; 32])
}

fn aead_seal_with_aad(plaintext: &[u8], aad: &[u8]) -> SealedChunk {
    let envelope = aead_wrap(plaintext, &fixture_key(), aad).expect("seal");
    SealedChunk { envelope }
}

/// Real AEAD open under a presented AAD — the ChaCha20-Poly1305 tag binds the
/// seal-time AAD, so a presented AAD ≠ the seal AAD fails closed. The
/// `_enforce` knob is ignored (R5: the real open always enforces).
fn aead_open_with_aad(
    sealed: &SealedChunk,
    presented_aad: &[u8],
    _enforce: bool,
) -> Result<Vec<u8>, AeadOpenError> {
    aead_unwrap(&sealed.envelope, &fixture_key(), presented_aad).map_err(|_| AeadOpenError::AadMismatch)
}

/// R5: the real AEAD open always binds the AAD (no knob).
const ENFORCE_AAD: bool = true;

#[derive(Debug, Clone, PartialEq, Eq)]
enum AeadOpenError {
    AadMismatch,
}

fn fixed_cid(byte: u8) -> [u8; 32] {
    [byte; 32]
}

/// F-LB-2 (a) — `aad_per_chunk` emits the canonical **BE** layout (M-19/M-20).
///
/// Pins the exact BE byte layout: prefix ‖ cid ‖ BE(chunk_index) ‖
/// BE(total_chunks). would-FAIL-if-no-op'd against an LE encoder: the last 12
/// integer bytes differ. The reference is hand-built BE so a silent LE
/// regression is caught.
#[test]
fn aad_per_chunk_is_big_endian() {
    let cid = fixed_cid(0xAA);
    let chunk_index: u64 = 0x0102_0304_0506_0708;
    let total_chunks: u32 = 0x0A0B_0C0D;

    let aad = aad_per_chunk_be(&cid, chunk_index, total_chunks);

    // Reconstruct the expected canonical BE layout independently.
    let mut expected = Vec::new();
    expected.extend_from_slice(b"benten-aead:chunk:");
    expected.extend_from_slice(&cid);
    expected.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]); // BE(chunk_index)
    expected.extend_from_slice(&[0x0A, 0x0B, 0x0C, 0x0D]); // BE(total_chunks)

    assert_eq!(
        aad, expected,
        "aad_per_chunk MUST encode chunk_index + total_chunks BIG-ENDIAN \
         (network-byte-order; M-19). would-FAIL while the in-tree encoder emits \
         LE (aead.rs:244). The integer bytes are the discriminating suffix."
    );
    // Explicit anti-LE guard: the LE encoding of chunk_index would put 0x08
    // FIRST among the index bytes. Pin that it does NOT.
    let index_bytes = &aad[b"benten-aead:chunk:".len() + cid.len()..][..8];
    assert_eq!(
        index_bytes[0], 0x01,
        "the FIRST chunk_index byte MUST be the most-significant (BE) — LE would \
         place 0x08 first"
    );
}

/// F-LB-2 (b) — truncation defense: a list sealed at `total=10` cannot be opened
/// as `total=5` (the count is AAD-bound).
///
/// would-FAIL-if-no-op'd: the stub ignores the AAD (ENFORCE_AAD=false) so the
/// re-counted open succeeds (the truncation bug); R5's AAD-bound open returns
/// `AadMismatch`.
#[test]
fn truncation_recount_total_chunks_fails_open() {
    let cid = fixed_cid(0xBB);
    let seal_aad = aad_per_chunk_be(&cid, /* chunk_index */ 0, /* total */ 10);
    let sealed = aead_seal_with_aad(b"chunk-0 body", &seal_aad);

    // Adversary re-presents the SAME chunk but claims total=5 (truncated list).
    let truncated_aad = aad_per_chunk_be(&cid, 0, 5);
    let outcome = aead_open_with_aad(&sealed, &truncated_aad, ENFORCE_AAD);

    assert!(
        matches!(outcome, Err(AeadOpenError::AadMismatch)),
        "a chunk sealed under total_chunks=10 MUST NOT open under a re-counted \
         total_chunks=5 — the count is AAD-bound, closing the truncation attack \
         (U17). would-FAIL while the stub ignores the AAD; got {outcome:?}"
    );
}

/// F-LB-2 (c) — cross-prefix reinterpretation: a per-CHUNK seal cannot be opened
/// as a per-RECIPE seal (the domain prefixes are load-bearing).
///
/// would-FAIL-if-no-op'd: the stub ignores the AAD; R5's open binds the prefix,
/// so a chunk-sealed ciphertext presented under a recipe-AAD FAILS.
#[test]
fn cross_prefix_chunk_vs_recipe_fails_open() {
    let cid = fixed_cid(0xCC);
    let chunk_aad = aad_per_chunk_be(&cid, /* idx */ 2, /* total */ 4);
    let sealed = aead_seal_with_aad(b"position-2 body", &chunk_aad);

    // Reinterpret the same position/count under the RECIPE domain prefix.
    let recipe_aad = aad_per_recipe_be(&cid, /* idx */ 2, /* total */ 4);
    // The prefixes differ, so the AADs differ even at identical position/count.
    assert_ne!(
        chunk_aad, recipe_aad,
        "the chunk and recipe AADs MUST differ even at identical \
         (cid, index, count) — the domain prefix is load-bearing"
    );

    let outcome = aead_open_with_aad(&sealed, &recipe_aad, ENFORCE_AAD);
    assert!(
        matches!(outcome, Err(AeadOpenError::AadMismatch)),
        "a per-CHUNK seal MUST NOT open under a per-RECIPE AAD — the \
         `benten-aead:chunk:` vs `benten-aead:recipe:` domain prefixes prevent \
         cross-context reinterpretation. would-FAIL while the stub ignores the \
         AAD; got {outcome:?}"
    );
}
