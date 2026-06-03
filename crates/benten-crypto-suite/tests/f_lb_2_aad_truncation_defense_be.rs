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

/// SELF-CONTAINED stub-shim. The AAD builders emit the correct **BE** layout
/// (these are the reference the freeze pins lock); the AEAD-open stub IGNORES
/// the AAD so the negative pins fail until R5 wires the real AAD-bound open.
mod f_lb_2_stub {
    /// Big-endian per-chunk AAD: `"benten-aead:chunk:" || plaintext_cid ||
    /// chunk_index.to_be_bytes() || total_chunks.to_be_bytes()`.
    ///
    /// (The in-tree `aead::aad_per_chunk` emits LE at `aead.rs:244` — the M-19
    /// migration flips it to this BE shape; F-W0-3 dep.)
    pub fn aad_per_chunk_be(plaintext_cid: &[u8], chunk_index: u64, total_chunks: u32) -> Vec<u8> {
        let mut aad = Vec::new();
        aad.extend_from_slice(b"benten-aead:chunk:");
        aad.extend_from_slice(plaintext_cid);
        aad.extend_from_slice(&chunk_index.to_be_bytes()); // BE (M-19)
        aad.extend_from_slice(&total_chunks.to_be_bytes()); // BE (M-19)
        aad
    }

    /// Big-endian per-Recipe AAD: distinct domain prefix `benten-aead:recipe:`.
    pub fn aad_per_recipe_be(
        plaintext_cid: &[u8],
        recipe_index: u32,
        total_recipes: u32,
    ) -> Vec<u8> {
        let mut aad = Vec::new();
        aad.extend_from_slice(b"benten-aead:recipe:");
        aad.extend_from_slice(plaintext_cid);
        aad.extend_from_slice(&recipe_index.to_be_bytes()); // BE (M-19)
        aad.extend_from_slice(&total_recipes.to_be_bytes()); // BE (M-19)
        aad
    }

    /// AEAD seal carrying the seal-time AAD. STUB stores the AAD alongside the
    /// ciphertext (modelling the real AEAD tag binding).
    #[derive(Debug, Clone)]
    pub struct SealedChunk {
        pub ciphertext: Vec<u8>,
        seal_aad: Vec<u8>,
    }

    pub fn aead_seal_with_aad(plaintext: &[u8], aad: &[u8]) -> SealedChunk {
        SealedChunk {
            ciphertext: plaintext.to_vec(),
            seal_aad: aad.to_vec(),
        }
    }

    /// AEAD open under a presented AAD. STUB IGNORES the presented AAD (the
    /// truncation/substitution bug) and always returns the plaintext — so the
    /// negative pins FAIL until R5. R5's real open returns `Err` when the
    /// presented AAD ≠ the seal AAD.
    pub fn aead_open_with_aad(
        sealed: &SealedChunk,
        presented_aad: &[u8],
        enforce: bool,
    ) -> Result<Vec<u8>, AeadOpenError> {
        if enforce && presented_aad != sealed.seal_aad {
            Err(AeadOpenError::AadMismatch)
        } else {
            // RED-PHASE: enforce == false → AAD ignored.
            Ok(sealed.ciphertext.clone())
        }
    }

    /// RED-PHASE knob: STUB = false (AAD ignored). R5 = true (AAD-bound open).
    pub const ENFORCE_AAD: bool = false;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum AeadOpenError {
        AadMismatch,
    }
}

use f_lb_2_stub::{
    AeadOpenError, ENFORCE_AAD, aad_per_chunk_be, aad_per_recipe_be, aead_open_with_aad,
    aead_seal_with_aad,
};

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
#[ignore = "RED-PHASE: F-LB-2 — aad_per_chunk MUST emit big-endian index/count (M-19; in-tree is LE); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-LB-2 — re-counting total_chunks (10→5) MUST fail AEAD-open (truncation defense, U17); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-LB-2 — per-chunk seal MUST NOT open under a per-recipe AAD (domain-prefix separation); un-ignore at R5"]
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
