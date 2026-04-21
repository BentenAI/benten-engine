//! Tests for [`Cid::from_str`] — the base32-lower-nopad multibase decoder
//! that is the inverse of [`Cid::to_base32`].
//!
//! Closes R7 audit finding F-R7-004 / backlog §6.1. Previously `from_str`
//! unconditionally returned `CoreError::CidParse` with a "Phase 2 deliverable"
//! message while the catalog claimed Phase 1 accepted base32; these tests
//! pin the round-trip contract so future refactors cannot drift.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests may use unwrap/expect per workspace policy"
)]

use std::fs;
use std::path::PathBuf;

use benten_core::testing::canonical_test_node;
use benten_core::{Cid, CoreError};

/// Round-trip: encode → decode must yield the same bytes.
#[test]
fn cid_from_str_roundtrip_of_canonical_node() {
    let cid = canonical_test_node().cid().unwrap();
    let encoded = cid.to_base32();
    let parsed = Cid::from_str(&encoded).expect("from_str must accept its own output");
    assert_eq!(parsed, cid, "round-trip must preserve CID bytes");
    assert_eq!(parsed.as_bytes(), cid.as_bytes());
}

/// The committed D2 fixture string must decode cleanly — the catalog hint
/// promises Phase 1 accepts this exact form.
#[test]
fn cid_from_str_parses_canonical_fixture() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("canonical_cid.txt");
    let recorded = fs::read_to_string(&fixture_path)
        .expect("D2 fixture must exist at tests/fixtures/canonical_cid.txt");
    let recorded = recorded.trim();

    let parsed = Cid::from_str(recorded).expect("fixture string must parse");
    let expected = canonical_test_node().cid().unwrap();
    assert_eq!(parsed, expected, "fixture CID must equal the spike hash");

    // And the fixture matches the live encoder — this is belt-and-suspenders
    // with the existing D2 test, but pinned at the decoder side.
    assert_eq!(parsed.to_base32(), recorded);
}

/// from_str and from_bytes must agree on valid inputs.
#[test]
fn cid_from_str_parity_with_from_bytes() {
    let cid = canonical_test_node().cid().unwrap();
    let via_str = Cid::from_str(&cid.to_base32()).unwrap();
    let via_bytes = Cid::from_bytes(cid.as_bytes()).unwrap();
    assert_eq!(via_str, via_bytes);
    assert_eq!(via_str.as_bytes(), via_bytes.as_bytes());
}

/// Reject strings that do not start with the `b` multibase prefix. Common
/// lookalikes: `B` (base32-upper), `z` (base58btc), `f` (base16), empty.
#[test]
fn cid_from_str_rejects_non_b_multibase_prefix() {
    // Empty input — no prefix at all.
    let err = Cid::from_str("").unwrap_err();
    assert!(
        matches!(err, CoreError::CidParse(_)),
        "empty string must fire CidParse, got {err:?}"
    );

    // Uppercase B (base32 upper) — close enough to `b` to be a real mistake.
    let mut upper = canonical_test_node().cid().unwrap().to_base32();
    assert!(upper.starts_with('b'));
    upper.replace_range(..1, "B");
    let err = Cid::from_str(&upper).unwrap_err();
    match err {
        CoreError::CidParse(msg) => {
            assert!(
                msg.contains("base32-lower-nopad") || msg.contains("b"),
                "prefix-rejection message should mention the expected prefix; got {msg}"
            );
        }
        other => panic!("expected CoreError::CidParse, got {other:?}"),
    }

    // base58btc prefix.
    let err = Cid::from_str("zQmSomething").unwrap_err();
    assert!(matches!(err, CoreError::CidParse(_)));

    // base16 prefix.
    let err = Cid::from_str("f01711e20").unwrap_err();
    assert!(matches!(err, CoreError::CidParse(_)));
}

/// Invalid alphabet characters must fail with CidParse, not panic.
#[test]
fn cid_from_str_rejects_invalid_alphabet() {
    // `0` and `1` are deliberately excluded from RFC 4648 base32.
    let err =
        Cid::from_str("bafyr4i0000000000000000000000000000000000000000000000000000").unwrap_err();
    match err {
        CoreError::CidParse(msg) => {
            assert!(
                msg.contains("alphabet") || msg.contains("a-z2-7"),
                "alphabet-rejection message should hint at the expected alphabet; got {msg}"
            );
        }
        other => panic!("expected CoreError::CidParse, got {other:?}"),
    }

    // Uppercase letter mid-string (this alphabet is lowercase).
    let err =
        Cid::from_str("bafyr4iFlzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda").unwrap_err();
    assert!(matches!(err, CoreError::CidParse(_)));

    // Punctuation / padding.
    let err = Cid::from_str("bafyr4i=").unwrap_err();
    assert!(matches!(err, CoreError::CidParse(_)));
    let err = Cid::from_str("bafyr4i!").unwrap_err();
    assert!(matches!(err, CoreError::CidParse(_)));
}

/// A valid-alphabet string that decodes to the wrong number of bytes must
/// surface as one of the two Phase-1 catalog codes that both map to
/// `E_CID_PARSE`: `CoreError::InvalidCid` (from the `from_bytes` length
/// check, when decoded bytes are a well-formed multiple of 8 bits) or
/// `CoreError::CidParse` (from the decoder's own non-zero-padding-bits
/// check, when truncation leaves junk in the trailing 5-bit group). Both
/// are legitimate rejections; the contract is no panic and the same stable
/// error code.
#[test]
fn cid_from_str_rejects_wrong_length_payload() {
    use benten_errors::ErrorCode;

    fn assert_cid_parse_class(err: &CoreError) {
        assert!(
            matches!(err, CoreError::InvalidCid(_) | CoreError::CidParse(_)),
            "malformed-length input must surface as InvalidCid or CidParse, got {err:?}"
        );
        assert_eq!(err.code(), ErrorCode::CidParse);
    }

    // Just the prefix — zero decoded bytes, length 0 != 36.
    let err = Cid::from_str("b").unwrap_err();
    assert_cid_parse_class(&err);

    // Short but nonzero.
    let err = Cid::from_str("bafya").unwrap_err();
    assert_cid_parse_class(&err);

    // Truncate the canonical fixture by one character — this commonly
    // leaves non-zero trailing padding bits in the final 5-bit group.
    let full = canonical_test_node().cid().unwrap().to_base32();
    let truncated = &full[..full.len() - 1];
    let err = Cid::from_str(truncated).unwrap_err();
    assert_cid_parse_class(&err);

    // Oversize: append two valid chars (10 extra bits → one extra byte plus
    // non-zero padding, so either arm of the class is valid).
    let mut oversized = full.clone();
    oversized.push('a');
    oversized.push('a');
    let err = Cid::from_str(&oversized).unwrap_err();
    assert_cid_parse_class(&err);
}
