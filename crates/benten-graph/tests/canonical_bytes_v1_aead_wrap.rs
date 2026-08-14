//! G-COMP-1 pull-forward #1 — ABSOLUTE golden-hex byte-pins for the
//! per-chunk / per-Node AEAD storage-envelope wire formats (Rows D-9 /
//! D-79).
//!
//! Per Row D-9, the per-chunk-AAD-*layout* pin already landed
//! (`benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs`).
//! This file adds the MISSING ABSOLUTE hex byte-pins for the two frozen
//! encoders on the storage path:
//!
//! - `AeadEnvelope::to_wire_bytes()` — the cipher-suite envelope framing
//!   (`magic 0xae | format_version | cipher_codepoint u16-BE | nonce_len
//!   | nonce | ciphertext`).
//! - `encode_encrypted_node()` — the G-CORE-3d storage envelope framing
//!   (`magic 0x3d | variant tag | plaintext_cid(36) | AeadEnvelope wire`)
//!   for the `Whole` arm.
//! - `encode_encrypted_node()` / `decode_encrypted_node()` for the
//!   **`Chunked`** arm — every stored object ≥ 64 KiB (E-03 / E-04, added at
//!   the R6 round-#1 falsification sweep). Prior to that sweep this arm had
//!   NO golden anywhere in the workspace: flipping the `Chunked` variant tag
//!   (`aead_wrap.rs:614`) and the u32 chunk count from big- to little-endian
//!   (`:622` encode / `:679` decode) left all nine benten-graph corpus
//!   targets at 27/27 PASS. The only tests that fired were two adversarial
//!   bounded-decode pins, and they fired for the wrong reason (they craft
//!   hostile blobs; they do not assert layout).
//!
//! # Determinism (golden-hex-via-throwaway-compute, memory M-20)
//!
//! A real `wrap()` call is NON-deterministic (random ChaCha20-Poly1305
//! nonce). So the fixtures do NOT call `wrap()`; they construct an
//! `AeadEnvelope` DIRECTLY from FIXED fields (fixed nonce + fixed
//! ciphertext bytes) via the public struct, then pin the deterministic
//! wire encoding. That is exactly the frozen envelope framing surface.
//! The plaintext CID is a fixed BLAKE3-digest CID. Hex captured via
//! throwaway compute + pasted below.
//!
//! would-FAIL-on-drift: magic byte, format-version, codepoint endianness
//! (M-19 BE; LE would swap `647a`→`7a64`), nonce-length byte, field
//! order, or the storage variant tag all change these bytes.

#![allow(clippy::unwrap_used)]

use benten_core::Cid;
use benten_crypto_suite::aead::AeadEnvelope;
use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
use benten_graph::aead_wrap::{
    ChunkedCiphertext, EncryptedNode, IROH_BLOCK_SIZE, decode_encrypted_node, decrypt_chunk,
    encode_encrypted_node, encrypt_chunk,
};

/// Lowercase-hex encoder (no `hex` crate dep in this workspace).
fn to_hex(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Fixed CID from a constant BLAKE3 digest (0xAA x32) — deterministic.
fn fixed_cid() -> Cid {
    Cid::from_blake3_digest([0xAA; 32])
}

/// A fixed `AeadEnvelope` at the default 0x647a codepoint with a fixed
/// 12-byte nonce + fixed ciphertext. All fields literal so the wire
/// bytes are deterministic.
fn fixed_envelope() -> AeadEnvelope {
    AeadEnvelope {
        format_version: 0x01,
        cipher_codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, // 0x647a
        nonce: vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
        ],
        ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
    }
}

#[test]
fn aead_envelope_to_wire_bytes_golden_hex_0x647a() {
    let got = to_hex(&fixed_envelope().to_wire_bytes());
    // ae(magic) 01(fmt) 647a(codepoint BE) 0c(nonce_len=12)
    //   || 010203..0c (nonce) || deadbeef (ciphertext)
    let expected = "ae01647a0c0102030405060708090a0b0cdeadbeef";
    assert_eq!(
        got, expected,
        "AeadEnvelope wire framing drifted from the frozen v1-beta bytes"
    );
}

#[test]
fn encode_encrypted_node_whole_golden_hex() {
    let node = EncryptedNode::Whole {
        plaintext_cid: fixed_cid(),
        envelope: fixed_envelope(),
    };
    let bytes = encode_encrypted_node(&node).unwrap();
    let got = to_hex(&bytes);
    // 3d(storage magic) 00(Whole variant) || plaintext_cid(36) ||
    //   AeadEnvelope::to_wire_bytes()
    let expected = "3d0001711e20aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaae01647a0c0102030405060708090a0b0cdeadbeef";
    assert_eq!(
        got, expected,
        "encode_encrypted_node(Whole) storage framing drifted from the frozen v1-beta bytes"
    );
}

// ---------------------------------------------------------------------------
// E-03 / E-04 — the `Chunked` storage-envelope arm.
//
// WHAT BREAKS: every stored object ≥ WHOLE_AEAD_THRESHOLD (64 KiB) is encoded
// through this arm. Its layout is
//   magic 0x3d | tag 0x01 | plaintext_cid(36) | count u32-BE
//     | { len u32-BE | AeadEnvelope wire } * count
// and it was FROZEN at G-CORE-9 with no golden covering the variant
// discriminant, the count width/endianness, or the per-chunk length prefix.
// ---------------------------------------------------------------------------

/// Wire width of one `AeadEnvelope` sealing a FULL 16 KiB chunk:
/// 5 header (magic | format_version | codepoint u16-BE | nonce_len)
/// + 12 nonce + 16384 plaintext + 16 Poly1305 tag.
const FULL_CHUNK_ENVELOPE_LEN: u32 = 16_417;

/// Wire width of the envelope sealing the 100-byte TAIL chunk: 5 + 12 + 100 + 16.
/// Deliberately DIFFERENT from `FULL_CHUNK_ENVELOPE_LEN` so a mutation that
/// emits a constant length prefix, or re-emits chunk 0's length for every
/// chunk, is caught.
const FINAL_CHUNK_ENVELOPE_LEN: u32 = 133;

/// Offset of the chunk-count field: magic(1) + tag(1) + plaintext_cid(36).
const CHUNK_COUNT_OFFSET: usize = 38;

/// E-03 + E-04 closure — the full `Chunked` layout: variant discriminant,
/// chunk-count width + BIG-endianness, and every per-chunk u32-BE length
/// prefix, walked to an exact end-of-buffer.
///
/// The expected lengths below are LITERALS derived from the frozen envelope
/// framing, never read back out of the encoder — that is what keeps this a
/// pin rather than a restatement.
///
/// MUTATIONS THAT MUST MAKE THIS FAIL (each is exactly one line in
/// `crates/benten-graph/src/aead_wrap.rs`):
///   :614  `out.push(0x01);`                → `out.push(0x00);`
///         (Chunked collapses onto the Whole discriminant — E-03)
///   :622  `&count.to_be_bytes()`           → `&count.to_le_bytes()`
///         (chunk count LE — E-04; caught twice: by the explicit anti-LE
///          `assert_ne!` and by the offset walk desynchronising)
///   :628  `&len.to_be_bytes()`             → `&len.to_le_bytes()`
///         (per-chunk length prefix LE)
///   :622  `u32` → `u16`/`u64` count width  (offset walk fails)
///   any reordering of magic / tag / cid / count / per-chunk framing
#[test]
fn encode_encrypted_node_chunked_layout_frozen() {
    let cid = fixed_cid();
    let key = [0x77_u8; 32];
    // 4 full chunks + a 100-byte tail => 5 chunks, non-uniform final length.
    let plaintext = vec![0xA5_u8; 4 * IROH_BLOCK_SIZE + 100];
    let chunked = ChunkedCiphertext::encrypt(&plaintext, &cid, &key).unwrap();
    assert_eq!(
        chunked.chunks().len(),
        5,
        "fixture must be a 5-chunk object"
    );

    let node = EncryptedNode::Chunked {
        plaintext_cid: cid,
        chunked,
    };
    let bytes = encode_encrypted_node(&node).unwrap();

    // Absolute total width. 42 header + 4*(4 + 16417) + (4 + 133).
    assert_eq!(
        bytes.len(),
        65_863,
        "chunked storage-envelope total width drifted — wire-format break"
    );

    // Byte 0 — storage magic.
    assert_eq!(bytes[0], 0x3d, "storage magic drifted");

    // Byte 1 — the Chunked variant discriminant (E-03). The paired
    // `assert_ne!` keeps the discriminant SEPARATION load-bearing: if both
    // arms emitted 0x00 the decoder would route chunked bytes into the Whole
    // path and hand `AeadEnvelope::from_wire_bytes` a length-prefixed stream.
    assert_eq!(
        bytes[1], 0x01,
        "Chunked variant tag drifted — MUST be 0x01 (wire-format break)"
    );
    assert_ne!(
        bytes[1], 0x00,
        "Chunked variant tag collapsed onto the Whole discriminant 0x00"
    );

    // Bytes 2..38 — plaintext CID, raw 36-byte form.
    assert_eq!(
        &bytes[2..CHUNK_COUNT_OFFSET],
        cid.as_bytes(),
        "chunked storage-envelope plaintext_cid segment drifted"
    );

    // Bytes 38..42 — chunk count, u32 BIG-endian (E-04). 5 => BE 00 00 00 05,
    // LE 05 00 00 00; the two are distinguishable, which is the whole point of
    // choosing a fixture count that is not palindromic.
    let count_field = &bytes[CHUNK_COUNT_OFFSET..CHUNK_COUNT_OFFSET + 4];
    assert_eq!(
        count_field,
        &5_u32.to_be_bytes(),
        "chunk count MUST be u32 BIG-endian per M-19 (wire-format break)"
    );
    assert_ne!(
        count_field,
        &5_u32.to_le_bytes(),
        "chunk count is little-endian — M-19 BE regression at aead_wrap.rs:622"
    );

    // Per-chunk framing: { len u32-BE | envelope }. Walk with LITERAL expected
    // lengths; landing exactly on `bytes.len()` closes the layout.
    let expected_lens = [
        FULL_CHUNK_ENVELOPE_LEN,
        FULL_CHUNK_ENVELOPE_LEN,
        FULL_CHUNK_ENVELOPE_LEN,
        FULL_CHUNK_ENVELOPE_LEN,
        FINAL_CHUNK_ENVELOPE_LEN,
    ];
    let mut off = CHUNK_COUNT_OFFSET + 4;
    for (i, expected_len) in expected_lens.iter().enumerate() {
        let prefix = &bytes[off..off + 4];
        assert_eq!(
            prefix,
            &expected_len.to_be_bytes(),
            "chunk {i} length prefix MUST be u32 BIG-endian per M-19 (wire-format break)"
        );
        assert_ne!(
            prefix,
            &expected_len.to_le_bytes(),
            "chunk {i} length prefix is little-endian — M-19 BE regression at aead_wrap.rs:628"
        );
        off += 4 + usize::try_from(*expected_len).unwrap();
    }
    assert_eq!(
        off,
        bytes.len(),
        "per-chunk framing walk did not land on end-of-buffer — chunked layout break"
    );
}

/// E-03 — the 64 KiB threshold dispatch actually reaches the `Chunked`
/// discriminant, and the sub-threshold path actually reaches `Whole`. Pins the
/// dispatch and the discriminant TOGETHER, on the real production entry point
/// (`EncryptedNode::encrypt`), which is what storage callers use.
///
/// MUTATIONS THAT MUST MAKE THIS FAIL (each one line, `aead_wrap.rs`):
///   :614  `out.push(0x01);` → `out.push(0x00);`
///   the `if plaintext.len() < WHOLE_AEAD_THRESHOLD` dispatch flipped to `<=`
///   or `>` (the threshold fixture sits exactly ON the boundary)
#[test]
fn threshold_dispatch_reaches_the_chunked_discriminant() {
    let cid = fixed_cid();
    let key = [0x77_u8; 32];

    // Exactly at the 64 KiB threshold — the FIRST size that must chunk.
    let at_threshold = vec![0x11_u8; 4 * IROH_BLOCK_SIZE];
    let encoded =
        encode_encrypted_node(&EncryptedNode::encrypt(&at_threshold, &cid, &key).unwrap()).unwrap();
    assert_eq!(
        encoded[1], 0x01,
        "a 64 KiB object MUST encode under the Chunked discriminant 0x01"
    );

    // One byte below the threshold — the LAST size that must stay whole.
    let below = vec![0x11_u8; 4 * IROH_BLOCK_SIZE - 1];
    let encoded =
        encode_encrypted_node(&EncryptedNode::encrypt(&below, &cid, &key).unwrap()).unwrap();
    assert_eq!(
        encoded[1], 0x00,
        "a 64 KiB - 1 object MUST encode under the Whole discriminant 0x00"
    );
}

/// E-04 **decode side**. The encode-side pin above cannot see a decoder
/// regression: `encode`/`decode` are a bilateral pair, so flipping BOTH
/// `to_be_bytes` and `from_be_bytes` round-trips perfectly — exactly the shape
/// that let the plugin-install-record preimage mutation ship green.
///
/// This test therefore never calls the encoder. It hand-authors the blob from
/// LITERAL big-endian spec bytes and asserts the decoder reads them, then
/// hand-authors the same blob with LITTLE-endian fields and asserts the
/// decoder REJECTS it. The second half is the differential that makes the
/// endianness observable.
///
/// MUTATIONS THAT MUST MAKE THIS FAIL (each one line, `aead_wrap.rs`):
///   :679  `u32::from_be_bytes([bytes[38], bytes[39], bytes[40], bytes[41]])`
///      →  `u32::from_le_bytes([...])`
///         (BE arm: count 0x00000002 reads as 33_554_432, trips the F-01
///          ceiling guard -> `ChunkCountExceedsInput`, the Ok(..) assert fails.
///          LE arm: the poisoned blob would DECODE, the is_err() assert fails.)
///   :724  the per-chunk `u32::from_be_bytes([...])` → `from_le_bytes([...])`
///         (same double-sided detection via the length-prefix poison arm)
#[test]
fn decode_chunked_reads_big_endian_count_and_length_prefixes() {
    let cid = fixed_cid();
    let key = [0x77_u8; 32];

    // Two real chunk envelopes from the production per-chunk sealer. Their
    // wire widths are FIXED and asserted below, so every length prefix in the
    // hand-built blob can be a literal.
    let chunk_0 = vec![0x11_u8; 32];
    let chunk_1 = vec![0x22_u8; 48];
    let env_0 = encrypt_chunk(&chunk_0, 0, 2, &cid, &key).unwrap();
    let env_1 = encrypt_chunk(&chunk_1, 1, 2, &cid, &key).unwrap();
    let wire_0 = env_0.to_wire_bytes();
    let wire_1 = env_1.to_wire_bytes();
    assert_eq!(wire_0.len(), 0x41, "5 hdr + 12 nonce + 32 pt + 16 tag = 65");
    assert_eq!(wire_1.len(), 0x51, "5 hdr + 12 nonce + 48 pt + 16 tag = 81");

    // Hand-authored spec bytes. Every multi-byte integer is a LITERAL
    // big-endian array — nothing here is produced by the encoder.
    let build = |count: [u8; 4], len_0: [u8; 4], len_1: [u8; 4]| -> Vec<u8> {
        let mut blob = Vec::new();
        blob.push(0x3d); // storage magic
        blob.push(0x01); // Chunked variant tag
        blob.extend_from_slice(cid.as_bytes()); // 36 B
        blob.extend_from_slice(&count);
        blob.extend_from_slice(&len_0);
        blob.extend_from_slice(&wire_0);
        blob.extend_from_slice(&len_1);
        blob.extend_from_slice(&wire_1);
        blob
    };

    // --- BE arm: the frozen encoding decodes, and carries real chunks. ---
    let be_blob = build(
        [0x00, 0x00, 0x00, 0x02], // count = 2, big-endian
        [0x00, 0x00, 0x00, 0x41], // chunk 0 length = 65, big-endian
        [0x00, 0x00, 0x00, 0x51], // chunk 1 length = 81, big-endian
    );
    assert_eq!(be_blob.len(), 196, "hand-authored blob width");
    let decoded = decode_encrypted_node(&be_blob).unwrap();
    let EncryptedNode::Chunked {
        plaintext_cid,
        chunked,
    } = decoded
    else {
        panic!("big-endian blob MUST decode to the Chunked arm");
    };
    assert_eq!(plaintext_cid, cid, "decoded plaintext_cid segment drifted");
    assert_eq!(chunked.chunks().len(), 2, "decoded chunk count drifted");
    // Prove the framing carried the real envelopes, not garbage that merely
    // happened to parse: both chunks must AEAD-authenticate and round-trip.
    assert_eq!(
        decrypt_chunk(&chunked.chunks()[0], 0, 2, &cid, &key).unwrap(),
        chunk_0
    );
    assert_eq!(
        decrypt_chunk(&chunked.chunks()[1], 1, 2, &cid, &key).unwrap(),
        chunk_1
    );

    // --- LE poison arm 1: count written little-endian MUST be rejected. ---
    // Under the frozen BE decoder [0x02,0,0,0] reads as 33_554_432, which
    // exceeds the F-01 input-derived ceiling. Under an LE decoder it would
    // read as 2 and decode cleanly — which is what this assertion forbids.
    let le_count_blob = build(
        [0x02, 0x00, 0x00, 0x00], // count = 2 written LITTLE-endian
        [0x00, 0x00, 0x00, 0x41],
        [0x00, 0x00, 0x00, 0x51],
    );
    assert!(
        decode_encrypted_node(&le_count_blob).is_err(),
        "a little-endian chunk count MUST NOT decode — the decoder is BE per M-19"
    );

    // --- LE poison arm 2: per-chunk length prefix written little-endian. ---
    // Under the frozen BE decoder [0x41,0,0,0] reads as 1_090_519_040 and
    // fails `checked_range_end`. Under an LE decoder it would read as 65.
    let le_len_blob = build(
        [0x00, 0x00, 0x00, 0x02],
        [0x41, 0x00, 0x00, 0x00], // chunk 0 length written LITTLE-endian
        [0x00, 0x00, 0x00, 0x51],
    );
    assert!(
        decode_encrypted_node(&le_len_blob).is_err(),
        "a little-endian per-chunk length prefix MUST NOT decode — the decoder is BE per M-19"
    );
}

/// E-03 / E-04 — absolute-hex pin on the DETERMINISTIC 42-byte chunked header
/// (`magic | tag | plaintext_cid | count`). The per-chunk bodies carry a random
/// nonce so they cannot be pinned absolutely from outside the crate; the
/// full-interleave absolute golden lives in
/// `aead_wrap.rs::tests::encode_encrypted_node_chunked_absolute_golden_hex`,
/// which can build fixed envelopes through the `pub(crate)` field.
///
/// This golden catches a COORDINATED mutation that edits the structural pin
/// above and the encoder together — a frozen literal cannot be carried along.
///
/// MUTATION THAT MUST MAKE THIS FAIL: any byte change in the first 42 bytes,
/// including `out.push(0x01)` -> `out.push(0x00)` and
/// `count.to_be_bytes()` -> `count.to_le_bytes()`.
///
/// PROVENANCE: the literal below was CAPTURED FROM THE REAL ENCODER at R6
/// round #1 (M-20 — goldens are never hand-authored) by running:
///
/// ```text
/// CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_BUILD_JOBS=6 \
///   cargo nextest run -p benten-graph --test canonical_bytes_v1_aead_wrap \
///   chunked_storage_envelope_header_absolute_golden_hex
/// ```
///
/// Shape: 84 hex chars = 42 bytes (`3d` `01` + 36-B CID + 4-B count).
///
/// The command is kept so a future maintainer can RE-DERIVE the value when a
/// wire change is deliberate and ratified. **If this test fails and you did not
/// intend a wire change, the encoder regressed — fix `aead_wrap.rs`, not this
/// literal.**
#[test]
fn chunked_storage_envelope_header_absolute_golden_hex() {
    let cid = fixed_cid();
    let key = [0x77_u8; 32];
    let plaintext = vec![0xA5_u8; 4 * IROH_BLOCK_SIZE + 100];
    let chunked = ChunkedCiphertext::encrypt(&plaintext, &cid, &key).unwrap();
    let bytes = encode_encrypted_node(&EncryptedNode::Chunked {
        plaintext_cid: cid,
        chunked,
    })
    .unwrap();

    let got = to_hex(&bytes[..CHUNK_COUNT_OFFSET + 4]);
    let expected =
        "3d0101711e20aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa00000005";
    assert_eq!(
        got, expected,
        "chunked storage-envelope header drifted from the frozen v1-beta bytes.\n\
         GOLDEN-CAPTURE chunked_header = \"{got}\""
    );
}
