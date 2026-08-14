//! GAP-KDB Shape-B — `KeySetDocument` strict-canonical decode reject
//! matrix (KSD-3 incl. S1 dev-field-reject, KSD-4 round-trip, KSD-5
//! version pin, KSD-8 kem-multikey malformed). W0-canary RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §1.2 + C3 (strict-canonical
//! decode, Row-D-13 injectivity — never a raw-byte compare) + S1
//! (KeySetDocument is a CLOSED 5-field map; any extra field incl. `dev`
//! REJECTS). `GAP-KDB-B-R2-LANDSCAPE.md` KSD-3/4/5/8 + §5 S1.
//!
//! # would_fail_on_revert
//! - KSD-3/5: a lax decoder (serde default / non-strict CBOR) accepts
//!   indefinite-length / duplicate-key / unsorted / non-minimal-int /
//!   trailing / extra-`dev` / forward-version inputs as Ok → each
//!   `expect_err` arm flips.
//! - KSD-4: a decoder that is not the inverse of the encoder breaks the
//!   round-trip equality.
//! - KSD-8: `resolve_kem` decoding a wrong-length / wrong-codec kem
//!   multikey returns a key instead of failing closed.
//!
//! # R5 un-ignore
//! Mint the strict-canonical `KeySetDocument::from_canonical_bytes` +
//! `Did::resolve_kem`'s kem-multikey decode; drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_id::kdb_testing as kdb;

// ─── minimal DAG-CBOR assembler (author-controlled malformed inputs) ──────

fn cbor_uint_minimal(n: u64) -> Vec<u8> {
    if n < 24 {
        vec![n as u8]
    } else if let Ok(n8) = u8::try_from(n) {
        vec![0x18, n8]
    } else if let Ok(n16) = u16::try_from(n) {
        let [hi, lo] = n16.to_be_bytes();
        vec![0x19, hi, lo]
    } else {
        let mut v = vec![0x1a];
        v.extend_from_slice(&(n as u32).to_be_bytes());
        v
    }
}

/// Force a NON-MINIMAL 2-byte uint encoding of a small value (e.g. 1 as
/// `0x19 0x00 0x01`) — a strict DAG-CBOR decoder MUST reject this.
fn cbor_uint_nonminimal_u16(n: u16) -> Vec<u8> {
    vec![0x19, (n >> 8) as u8, n as u8]
}

fn cbor_text(s: &str) -> Vec<u8> {
    let mut v = vec![0x60 | (s.len() as u8)]; // text len < 24 for our keys
    v.extend_from_slice(s.as_bytes());
    v
}

fn cbor_bytes(b: &[u8]) -> Vec<u8> {
    let mut v = Vec::new();
    let n = b.len();
    if n < 24 {
        v.push(0x40 | (n as u8));
    } else if let Ok(n16) = u16::try_from(n) {
        v.push(0x59);
        v.extend_from_slice(&n16.to_be_bytes());
    } else {
        v.push(0x5a);
        v.extend_from_slice(&(n as u32).to_be_bytes());
    }
    v.extend_from_slice(b);
    v
}

/// A canonical `(key,value)` pair appended to `out`.
fn pair(out: &mut Vec<u8>, key: &str, val: Vec<u8>) {
    out.extend(cbor_text(key));
    out.extend(val);
}

/// The canonical field VALUES for a valid tiny v1-beta doc (small opaque
/// sig/kem so the assembled bytes stay inspectable).
fn tiny_fields() -> (Vec<u8>, Vec<u8>) {
    (
        kdb::det_bytes("ksd-strict/sig", 8),
        kdb::det_bytes("ksd-strict/kem", 6),
    )
}

/// A CANONICAL 5-field map (the well-formed baseline).
fn canonical_map() -> Vec<u8> {
    let (sig, kem) = tiny_fields();
    let mut m = vec![0xa5]; // map(5)
    pair(&mut m, "v", cbor_uint_minimal(1));
    pair(&mut m, "kem", cbor_bytes(&kem));
    pair(&mut m, "sig", cbor_bytes(&sig));
    pair(&mut m, "kem_cp", cbor_uint_minimal(0x647a));
    pair(&mut m, "sig_cp", cbor_uint_minimal(0x0001));
    m
}

// ── KSD-4 — round-trip byte-stability (positive) ──────────────────────────

#[test]
fn ksd4_round_trip_byte_stable() {
    let (sig, kem) = tiny_fields();
    let doc = kdb::KeySetDocument::v1_hybrid(sig, kem);
    let encoded = doc.to_canonical_bytes();
    let decoded =
        kdb::KeySetDocument::from_canonical_bytes(&encoded).expect("a canonical doc MUST decode");
    assert_eq!(
        decoded.to_canonical_bytes(),
        encoded,
        "encode→decode→encode MUST be byte-stable (strict-canonical inverse)"
    );
    assert_eq!(decoded, doc, "decoded document MUST equal the original");
}

// ── KSD-3 — strict-canonical decode REJECT matrix (design C3) ─────────────

#[test]
fn ksd3_indefinite_length_map_rejects() {
    // Indefinite-length map (0xbf … 0xff) instead of the definite 0xa5.
    let mut m = canonical_map();
    m[0] = 0xbf;
    m.push(0xff);
    assert!(
        kdb::KeySetDocument::from_canonical_bytes(&m).is_err(),
        "strict-canonical decode MUST reject an indefinite-length map (DAG-CBOR)"
    );
}

#[test]
fn ksd3_s1_extra_dev_field_rejects() {
    // S1: Shape-B DROPS the `dev` field at v1-beta — the map is a CLOSED
    // 5-field map. A canonically-sorted 6-field map that adds `dev`
    // (len-3, sorts as `dev` < `kem`) MUST reject.
    let (sig, kem) = tiny_fields();
    let mut m = vec![0xa6]; // map(6)
    pair(&mut m, "v", cbor_uint_minimal(1));
    pair(&mut m, "dev", cbor_bytes(&[])); // extra field, canonically placed
    pair(&mut m, "kem", cbor_bytes(&kem));
    pair(&mut m, "sig", cbor_bytes(&sig));
    pair(&mut m, "kem_cp", cbor_uint_minimal(0x647a));
    pair(&mut m, "sig_cp", cbor_uint_minimal(0x0001));
    assert!(
        kdb::KeySetDocument::from_canonical_bytes(&m).is_err(),
        "S1: an extra `dev` field MUST reject — KeySetDocument is a CLOSED 5-field map at v1-beta"
    );
}

#[test]
fn ksd3_unsorted_keys_reject() {
    // Emit `sig` before `kem` (bytewise-unsorted at equal length).
    let (sig, kem) = tiny_fields();
    let mut m = vec![0xa5];
    pair(&mut m, "v", cbor_uint_minimal(1));
    pair(&mut m, "sig", cbor_bytes(&sig)); // WRONG: sig before kem
    pair(&mut m, "kem", cbor_bytes(&kem));
    pair(&mut m, "kem_cp", cbor_uint_minimal(0x647a));
    pair(&mut m, "sig_cp", cbor_uint_minimal(0x0001));
    assert!(
        kdb::KeySetDocument::from_canonical_bytes(&m).is_err(),
        "strict-canonical decode MUST reject unsorted map keys (design C3, Row-D-13)"
    );
}

#[test]
fn ksd3_non_minimal_int_rejects() {
    // Encode `v = 1` as a non-minimal 2-byte uint (0x19 0x00 0x01).
    let (sig, kem) = tiny_fields();
    let mut m = vec![0xa5];
    pair(&mut m, "v", cbor_uint_nonminimal_u16(1)); // WRONG: non-minimal
    pair(&mut m, "kem", cbor_bytes(&kem));
    pair(&mut m, "sig", cbor_bytes(&sig));
    pair(&mut m, "kem_cp", cbor_uint_minimal(0x647a));
    pair(&mut m, "sig_cp", cbor_uint_minimal(0x0001));
    assert!(
        kdb::KeySetDocument::from_canonical_bytes(&m).is_err(),
        "strict-canonical decode MUST reject non-minimal integer encodings (DAG-CBOR)"
    );
}

#[test]
fn ksd3_duplicate_key_rejects() {
    let (sig, kem) = tiny_fields();
    let mut m = vec![0xa6]; // map(6): duplicate `v`
    pair(&mut m, "v", cbor_uint_minimal(1));
    pair(&mut m, "v", cbor_uint_minimal(1)); // duplicate
    pair(&mut m, "kem", cbor_bytes(&kem));
    pair(&mut m, "sig", cbor_bytes(&sig));
    pair(&mut m, "kem_cp", cbor_uint_minimal(0x647a));
    pair(&mut m, "sig_cp", cbor_uint_minimal(0x0001));
    assert!(
        kdb::KeySetDocument::from_canonical_bytes(&m).is_err(),
        "strict-canonical decode MUST reject duplicate map keys"
    );
}

#[test]
fn ksd3_trailing_bytes_reject() {
    let mut m = canonical_map();
    m.push(0x00); // trailing byte after a complete map
    assert!(
        kdb::KeySetDocument::from_canonical_bytes(&m).is_err(),
        "strict-canonical decode MUST reject trailing bytes after the map (exact-consume)"
    );
}

// ── KSD-5 — version pin + forward-version reject ──────────────────────────

#[test]
fn ksd5_forward_version_rejects() {
    // v = 2 is a future format the v1-beta decoder MUST typed-reject
    // (fail-closed, never a best-effort parse).
    let (sig, kem) = tiny_fields();
    let mut m = vec![0xa5];
    pair(&mut m, "v", cbor_uint_minimal(2)); // forward version
    pair(&mut m, "kem", cbor_bytes(&kem));
    pair(&mut m, "sig", cbor_bytes(&sig));
    pair(&mut m, "kem_cp", cbor_uint_minimal(0x647a));
    pair(&mut m, "sig_cp", cbor_uint_minimal(0x0001));
    assert!(
        kdb::KeySetDocument::from_canonical_bytes(&m).is_err(),
        "a KeySetDocument with v=2 MUST typed-reject at the v1-beta decoder (forward-version)"
    );
}

// ── KSD-8 — kem multikey malformed / wrong-length / wrong-codec reject ────

#[test]
fn ksd8_kem_multikey_wrong_length_rejects() {
    // A kem field whose ML-KEM component is truncated (wrong length) MUST
    // fail closed when resolve_kem decodes it. Build a self-committed DID
    // (steps 1+2 satisfied) so the ONLY failure source is the bad kem.
    let mut ek = kdb::det_mlkem768_ek("ksd8/ek");
    ek.truncate(ek.len() - 1); // one byte short
    let bad_kem = kdb::kem_multikey_hybrid(&kdb::det_x25519_pub("ksd8/x"), &ek);
    let doc = kdb::KeySetDocument::v1_hybrid(kdb::det_signing_multikey("ksd8"), bad_kem);
    let did = kdb::self_committed_did(&doc);
    assert!(
        kdb::resolve_kem(&did, &doc).is_err(),
        "resolve_kem MUST reject a kem multikey whose ML-KEM component is the wrong length"
    );
}

#[test]
fn ksd8_kem_multikey_wrong_codec_rejects() {
    // A kem field whose second component carries a WRONG multicodec (not
    // mlkem-768-pub 0x120c) MUST fail closed.
    let x = kdb::det_x25519_pub("ksd8b/x");
    let ek = kdb::det_mlkem768_ek("ksd8b/ek");
    let mut bad_kem = Vec::new();
    bad_kem.extend_from_slice(&kdb::X25519_PUB_MULTICODEC);
    bad_kem.extend_from_slice(&x);
    bad_kem.extend_from_slice(&[0xed, 0x01]); // WRONG codec (ed25519, not mlkem)
    bad_kem.extend_from_slice(&ek);
    let doc = kdb::KeySetDocument::v1_hybrid(kdb::det_signing_multikey("ksd8b"), bad_kem);
    let did = kdb::self_committed_did(&doc);
    assert!(
        kdb::resolve_kem(&did, &doc).is_err(),
        "resolve_kem MUST reject a kem multikey whose second component is not mlkem-768-pub (0x120c)"
    );
}
