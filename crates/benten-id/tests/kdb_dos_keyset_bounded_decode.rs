//! GAP-KDB Shape-B — KeySetDocument bounded-decode DoS cap (DOS-1).
//! W0-canary RED-PHASE. Mirrors `dos_unbounded_decode_safe2_1172.rs`.
//!
//! Ref `3bea1294`: `GAP-KDB-B-R2-LANDSCAPE.md` DOS-1 — the KeySetDocument
//! is a **SECOND untrusted input** the F2 DID-string cap (C7) does NOT
//! cover (META #629 DoS-via-unbounded-decode). A first-contact / cached /
//! iroh-fetched key-set doc is attacker-influenced, so its strict decode
//! MUST be bounded (never a giant allocation / hang from an attacker-
//! declared or attacker-supplied oversized field).
//!
//! # would_fail_on_revert
//! - `dos1_oversized_but_wellformed_doc_rejects`: a COMPLETE, otherwise-
//!   valid 5-field map whose `kem` field is ~512 KB (≫ the ≈3.2 KB
//!   legitimate max). A bounded decoder rejects it on the size cap
//!   (`is_err`); an UNBOUNDED decoder decodes it successfully (`is_ok`).
//!   Dropping the bound flips `is_err → is_ok` — a real revert-catch (not
//!   just a fast-EOF error).
//! - `dos1_declared_length_bomb_rejects_fast`: a `kem` byte-string header
//!   DECLARING ~2 GB with no data. A decoder that pre-allocates the
//!   declared capacity OOMs/hangs; a bounded pre-check rejects fast.
//!
//! # R5 un-ignore
//! Mint `KeySetDocument::from_canonical_bytes` with a total-input / field
//! -length bound applied BEFORE the allocating decode; drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, Instant};

use benten_id::kdb_testing as kdb;

/// CBOR byte-string HEADER (major type 2) for length `n` (no payload).
fn cbor_bytes_header(n: usize) -> Vec<u8> {
    if let Ok(n16) = u16::try_from(n) {
        let mut v = vec![0x59];
        v.extend_from_slice(&n16.to_be_bytes());
        v
    } else {
        let mut v = vec![0x5a];
        v.extend_from_slice(&(n as u32).to_be_bytes());
        v
    }
}

/// A COMPLETE, canonically-ordered, otherwise-valid 5-field map whose
/// `kem` field is `kem_len` bytes. With a legitimate `kem_len` this is a
/// valid doc; with an oversized `kem_len` it is the DoS input.
fn wellformed_map_with_kem_len(kem_len: usize) -> Vec<u8> {
    let sig = kdb::det_bytes("dos/sig", 8);
    let kem = vec![0u8; kem_len];
    let mut m = vec![0xa5]; // map(5)
    // "v": 1
    m.extend_from_slice(&[0x61, b'v', 0x01]);
    // "kem": <bytes kem_len>
    m.extend_from_slice(&[0x63, b'k', b'e', b'm']);
    m.extend(cbor_bytes_header(kem.len()));
    m.extend_from_slice(&kem);
    // "sig": <bytes 8>
    m.extend_from_slice(&[0x63, b's', b'i', b'g']);
    m.push(0x40 | (sig.len() as u8));
    m.extend_from_slice(&sig);
    // "kem_cp": 0x647a
    m.extend_from_slice(&[0x66, b'k', b'e', b'm', b'_', b'c', b'p', 0x19, 0x64, 0x7a]);
    // "sig_cp": 1
    m.extend_from_slice(&[0x66, b's', b'i', b'g', b'_', b'c', b'p', 0x01]);
    m
}

#[test]
#[ignore = "RED-PHASE: DOS-1 oversized-but-well-formed KeySetDocument bounded reject — un-ignore at R5"]
fn dos1_oversized_but_wellformed_doc_rejects() {
    // A COMPLETE, valid doc whose `kem` is ~512 KB — ≫ the ≈3.2 KB
    // legitimate max. A bounded decoder MUST reject on the size cap; an
    // unbounded decoder would happily return Ok(KeySetDocument{kem: 512KB}).
    let bomb = wellformed_map_with_kem_len(512 * 1024);
    assert!(
        kdb::KeySetDocument::from_canonical_bytes(&bomb).is_err(),
        "DOS-1: an oversized (≫ legitimate) but well-formed key-set doc MUST fail closed on the \
         size bound. If this is Ok, the bound was dropped — an unbounded decoder accepts arbitrary \
         attacker-sized docs (META #629)."
    );
    // Control: a legitimately-sized doc (real kem multikey ≈1220 B) DECODES —
    // the bound must not reject legitimate documents.
    let legit_kem = kdb::kem_multikey_hybrid(
        &kdb::det_x25519_pub("dos/x"),
        &kdb::det_mlkem768_ek("dos/ek"),
    );
    let legit = kdb::KeySetDocument::v1_hybrid(kdb::det_signing_multikey("dos"), legit_kem);
    assert!(
        kdb::KeySetDocument::from_canonical_bytes(&legit.to_canonical_bytes()).is_ok(),
        "DOS-1 control: a legitimately-sized key-set doc MUST decode (the bound is not over-tight)"
    );
}

#[test]
#[ignore = "RED-PHASE: DOS-1 declared-length pre-alloc bomb bounded reject — un-ignore at R5"]
fn dos1_declared_length_bomb_rejects_fast() {
    // A `kem` byte-string header DECLARING ~2 GB but carrying no data. A
    // decoder that pre-allocates the declared capacity OOMs/hangs; a
    // bounded pre-check rejects fast (never honors an attacker-declared
    // multi-GB length).
    let mut bomb: Vec<u8> = vec![0xa5];
    bomb.extend_from_slice(&[0x61, b'v', 0x01]); // "v": 1
    bomb.extend_from_slice(&[0x63, b'k', b'e', b'm']); // "kem":
    bomb.extend_from_slice(&[0x5a, 0x7f, 0xff, 0xff, 0xff]); // bytes, len ~2 GB, NO data

    let start = Instant::now();
    let result = kdb::KeySetDocument::from_canonical_bytes(&bomb);
    let elapsed = start.elapsed();
    assert!(
        result.is_err(),
        "DOS-1: a declared-length bomb MUST fail closed (bounded decode)"
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "DOS-1: the bound MUST reject BEFORE any allocation of the declared length (got {elapsed:?}); \
         a multi-second time / OOM means an attacker-declared ~2 GB length was pre-allocated"
    );
}
