//! TF-3d pins — G-CORE-3d per-chunk AEAD with chunk size = `IROH_BLOCK_SIZE`.
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition. Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3d (P-2)
//!     "Per-chunk AEAD for Nodes ≥64 KiB (item 15(g) freeze surface):
//!     synthetic ~256 KiB Node split into 16 KiB chunks; each chunk
//!     decrypts independently (range-fetch preservation per Spike H+1.2);
//!     AAD = `(plaintext_cid, chunk_index)`. Would-FAIL on double-chunking
//!     overhead (chunk-size diverges from `IROH_BLOCK_SIZE`)."
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3d (P-3)
//!     "Whole-content AEAD for Nodes <64 KiB; single AEAD ciphertext +
//!     AAD-binds-plaintext-CID."
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3d (A-1)
//!     cross-chunk rebinding attack: swap chunk N with chunk M (same
//!     Node) → AEAD AAD-binds-chunk-index detects → fails.
//!   - `00-implementation-plan.md` §1.A.FROZEN item 15(g): "Two-CID
//!     mapping contract + per-chunk-AEAD chunk-size constant — For Nodes
//!     ≥64 KiB: per-chunk AEAD with chunk size = `IROH_BLOCK_SIZE`
//!     (16 KiB) + AAD-binds-chunk-index. Whole-content AEAD for smaller
//!     Nodes." LOAD-BEARING: chunk-size constant alignment with
//!     `IROH_BLOCK_SIZE` (different chunk size = double-chunking overhead).
//!   - `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R2: per-Spike
//!     H+1.2 chunk size = `IROH_BLOCK_SIZE`.
//!   - §6 CI gate (13) "Per-chunk-AEAD chunk-size = `IROH_BLOCK_SIZE`
//!     (16 KiB) for Nodes ≥64 KiB; whole-AEAD for <64 KiB."
//!   - §6 CI gate (14) "AAD-binds-chunk-index for cross-chunk rebinding
//!     prevention."
//!
//! ============================================================================
//! LANDED at G-CORE-3d (pim-12 / §3.6e closure).
//! ============================================================================
//! The `aead_wrap::{IROH_BLOCK_SIZE, WHOLE_AEAD_THRESHOLD, EncryptedNode,
//! ChunkedCiphertext}` surfaces do not exist at origin/main `c9c11c56` →
//! compile-but-fail at the `use` line.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::unnested_or_patterns)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_graph::{RedbBackend, WriteContext};
// Production failure point (LANDED at G-CORE-3d).
use benten_graph::aead_wrap::{
    AeadError, ChunkedCiphertext, EncryptedNode, IROH_BLOCK_SIZE, WHOLE_AEAD_THRESHOLD,
    decrypt_chunk, encrypt_chunk,
};
use tempfile::tempdir;

fn namespace_cid(seed: &str) -> Cid {
    let mut props = BTreeMap::new();
    props.insert("did-seed".to_string(), Value::text(seed));
    Node::new(vec!["system:Principal".to_string()], props)
        .cid()
        .expect("namespace seed node must hash")
}

/// Build a Node whose canonical-bytes size exceeds `target_size_kib` KiB.
fn large_node(payload_kib: usize) -> Node {
    let mut props = BTreeMap::new();
    // 1 KiB of base64-shaped text per repetition.
    let unit = "A".repeat(1024);
    let blob = unit.repeat(payload_kib);
    props.insert("blob".to_string(), Value::text(blob));
    Node::new(vec!["LargeBlob".to_string()], props)
}

fn fresh_backend() -> (tempfile::TempDir, RedbBackend) {
    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("tf3d-chunks.redb")).expect("open redb");
    (dir, backend)
}

// ---------------------------------------------------------------------------
// PIN 1 — `IROH_BLOCK_SIZE` constant is exactly 16 KiB.
// ---------------------------------------------------------------------------
// The chunk-size constant is load-bearing per §1.A.FROZEN item 15(g):
// "different chunk size = double-chunking overhead." Spike H+1.2
// validated that `IROH_BLOCK_SIZE == 16384`. Would-FAIL if a future
// drift sets it to 32 KiB / 64 KiB / etc.
#[test]

fn tf3d_iroh_block_size_is_16_kib_exactly() {
    assert_eq!(
        IROH_BLOCK_SIZE,
        16 * 1024,
        "`IROH_BLOCK_SIZE` MUST equal 16 KiB (16384 bytes) per Spike H+1.2 \
         and §1.A.FROZEN item 15(g). A drift here means double-chunking \
         overhead when serving via iroh-blobs (different chunking strategy \
         from iroh's wire layer)."
    );
}

// ---------------------------------------------------------------------------
// PIN 2 — `WHOLE_AEAD_THRESHOLD` = 64 KiB.
// ---------------------------------------------------------------------------
// The Node-size threshold above which per-chunk AEAD is used (vs
// whole-content AEAD for smaller Nodes). Frozen at 64 KiB per item 15(g).
#[test]

fn tf3d_whole_aead_threshold_is_64_kib() {
    assert_eq!(
        WHOLE_AEAD_THRESHOLD,
        64 * 1024,
        "`WHOLE_AEAD_THRESHOLD` MUST equal 64 KiB (65536 bytes) — the \
         boundary at which the AEAD layer switches from whole-content to \
         per-chunk strategy. Frozen at §1.A.FROZEN item 15(g)."
    );
}

// ---------------------------------------------------------------------------
// PIN 3 — Per-chunk AEAD for ~256 KiB Node → 16 chunks of 16 KiB each.
// ---------------------------------------------------------------------------
// Production-arm P-2: a Node whose canonical bytes are ≥64 KiB is split
// into chunks of size `IROH_BLOCK_SIZE` (16 KiB); each chunk decrypts
// INDEPENDENTLY (range-fetch preservation). AAD = (plaintext_cid,
// chunk_index).
//
// Would-FAIL-IF-NO-OP'd: a stub that treats the Node as whole-content
// AEAD produces a single ciphertext, not N chunks. Range-fetch breaks.
#[test]

fn tf3d_large_node_chunked_to_16_kib_each_independent_decrypt() {
    let node = large_node(256); // ~256 KiB
    let plaintext_cid = node.cid().unwrap();
    let bytes = node.to_canonical_bytes().unwrap();
    assert!(
        bytes.len() >= 256 * 1024,
        "fixture Node must be ≥256 KiB so the per-chunk arm fires"
    );

    let chunked: ChunkedCiphertext =
        ChunkedCiphertext::encrypt(&bytes, &plaintext_cid, &node.derive_key_for_test())
            .expect("per-chunk encrypt OK");

    // PIN: chunk count == ceil(bytes.len() / IROH_BLOCK_SIZE)
    let expected_chunks = bytes.len().div_ceil(IROH_BLOCK_SIZE);
    assert_eq!(
        chunked.chunks().len(),
        expected_chunks,
        "256 KiB Node MUST split into exactly {} chunks of {} bytes each \
         (chunk-size = IROH_BLOCK_SIZE). A divergent chunk size yields a \
         different chunk count + double-chunking overhead at iroh wire.",
        expected_chunks,
        IROH_BLOCK_SIZE
    );

    // Each chunk decrypts INDEPENDENTLY — the range-fetch preservation
    // property. Pick chunk index 7 (middle of the file) and decrypt only
    // that chunk; assert it round-trips to the expected bytes at offset
    // 7 * IROH_BLOCK_SIZE.
    let middle = 7usize;
    let decrypted_chunk = decrypt_chunk(
        &chunked.chunks()[middle],
        middle,
        &plaintext_cid,
        &node.derive_key_for_test(),
    )
    .expect("independent chunk decrypt OK");
    let start = middle * IROH_BLOCK_SIZE;
    let end = (start + IROH_BLOCK_SIZE).min(bytes.len());
    assert_eq!(
        decrypted_chunk,
        &bytes[start..end],
        "Chunk {} MUST decrypt independently to the corresponding byte \
         slice — this is the range-fetch preservation per Spike H+1.2.",
        middle
    );
}

// ---------------------------------------------------------------------------
// PIN 4 — Whole-content AEAD for Node <64 KiB.
// ---------------------------------------------------------------------------
// Production-arm P-3: small Node uses single AEAD ciphertext +
// AAD-binds-plaintext-CID, NOT chunked.
#[test]

fn tf3d_small_node_uses_whole_aead_not_chunked() {
    let node = large_node(8); // ~8 KiB; well below 64 KiB threshold
    let plaintext_cid = node.cid().unwrap();
    let bytes = node.to_canonical_bytes().unwrap();
    assert!(
        bytes.len() < WHOLE_AEAD_THRESHOLD,
        "fixture Node must be < threshold so the whole-AEAD arm fires"
    );

    let encrypted: EncryptedNode =
        EncryptedNode::encrypt(&bytes, &plaintext_cid, &node.derive_key_for_test())
            .expect("whole-AEAD encrypt OK");

    assert!(
        matches!(encrypted, EncryptedNode::Whole { .. }),
        "Node < {} bytes MUST use the Whole AEAD arm (single ciphertext, \
         AAD-binds-plaintext-CID), NOT chunked.",
        WHOLE_AEAD_THRESHOLD
    );
}

// ---------------------------------------------------------------------------
// PIN 5 — A-1 adversarial: cross-chunk rebinding attack defeated by
// AAD-binds-chunk-index.
// ---------------------------------------------------------------------------
// Take a valid per-chunk ciphertext at chunk index N; present it at
// chunk index M (where M ≠ N). The AEAD AAD binds the chunk index, so
// the authentication fails. Would-FAIL-IF-NO-OP'd: a stub that omits
// `chunk_index` from the AAD allows free chunk-shuffling (an attacker
// could reorder a file's content silently).
#[test]

fn tf3d_cross_chunk_rebinding_fails_aad_binds_chunk_index() {
    let node = large_node(256);
    let plaintext_cid = node.cid().unwrap();
    let bytes = node.to_canonical_bytes().unwrap();
    let chunked: ChunkedCiphertext =
        ChunkedCiphertext::encrypt(&bytes, &plaintext_cid, &node.derive_key_for_test()).unwrap();

    // Attacker shuffles: present chunk-N's ciphertext at index M.
    let n = 3usize;
    let m = 11usize;
    let attacker_ciphertext = &chunked.chunks()[n];

    let result = decrypt_chunk(
        attacker_ciphertext,
        m, // wrong index → AAD mismatch
        &plaintext_cid,
        &node.derive_key_for_test(),
    );
    assert!(
        matches!(result, Err(AeadError::Authentication { .. })),
        "Cross-chunk rebinding (chunk-N at index M) MUST fail AEAD \
         authentication (AAD binds chunk_index). NEVER silently accept \
         the shuffled chunk. got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}
