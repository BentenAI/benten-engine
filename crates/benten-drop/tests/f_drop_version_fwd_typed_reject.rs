//! F-DROP-VER-FWD (Row D-83) closure pin: a genuine FUTURE on-the-wire
//! Drop-bundle version tag (e.g. `{"tag":"V2"}`) must deserialize to the
//! advertised typed [`DropBundleError::UnsupportedDropVersion`] — NOT a
//! generic serde codec error.
//!
//! The `DropBundleVersion` enum carries a `#[serde(other)] UnknownVersion`
//! catch-all so every unrecognized version tag maps to it, and the
//! version-check in `DropBundle::parse_cbor_bytes` routes it to the typed
//! reject. Adding the catch-all is DESERIALIZE-only — the `V1` /
//! `Synthetic` wire bytes are byte-identical (covered by the freeze/golden
//! round-trips + the byte-identity pin below).

#![allow(clippy::unwrap_used)]

use benten_drop::DropBundle;
use benten_drop::bundle::{DROP_BUNDLE_MAX_SIZE_BYTES, DropBundleError, DropBundleVersion};

/// Encode a `DropBundleVersion`, then flip the version-tag STRING from
/// `"V1"` to `"V2"` inside the serialized CBOR — synthesizing a bundle
/// that names a version this reader has never seen. The DAG-CBOR
/// encoding of the version field is the map `{"tag": "V1"}`, whose
/// `"V1"` text-string bytes are `62 56 31` (`text(2) 'V' '1'`); the
/// only difference for `"V2"` is the final `0x31` → `0x32`.
fn patch_v1_tag_to_v2(cbor: &mut [u8]) {
    // Locate the `62 56 31` ("V1" text-string) sequence and bump the
    // trailing '1' to '2'. There is exactly one version field.
    let needle = [0x62u8, b'V', b'1'];
    let pos = cbor
        .windows(3)
        .position(|w| w == needle)
        .expect("serialized bundle must carry the V1 version tag");
    cbor[pos + 2] = b'2';
}

#[test]
fn future_version_tag_decodes_to_typed_unsupported_drop_version() {
    // Build a real V1 bundle, serialize, then rewrite its version tag to
    // a future "V2" the reader does not know.
    let issuer_kp = benten_id::keypair::Keypair::generate();
    let recipient_kp = benten_id::keypair::Keypair::generate();
    let bundle = DropBundle::build_5_recipe_bundle_for_recipient(&issuer_kp, &recipient_kp);
    let mut cbor = bundle.to_cbor_bytes().unwrap();

    patch_v1_tag_to_v2(&mut cbor);

    let result = DropBundle::parse_cbor_bytes(&cbor);
    match result {
        Err(DropBundleError::UnsupportedDropVersion { seen }) => {
            // The `#[serde(other)]` catch-all discards the real tag
            // string, so `seen` carries the documented sentinel — the
            // load-bearing property is the TYPED reject, not the number.
            assert_eq!(
                seen,
                u16::MAX,
                "UnknownVersion routes through the documented sentinel"
            );
        }
        other => panic!(
            "future version tag must yield typed UnsupportedDropVersion (not a generic \
             codec error), got {other:?}"
        ),
    }
}

#[test]
fn unknown_version_variant_deserializes_from_future_tag_directly() {
    // Direct enum-level pin: a bare `{"tag":"V2"}` map deserializes to
    // the catch-all rather than failing as an unknown-variant codec
    // error. Encode a known version, patch to V2, decode.
    let mut cbor = serde_ipld_dagcbor::to_vec(&DropBundleVersion::V1).unwrap();
    patch_v1_tag_to_v2(&mut cbor);
    let decoded: DropBundleVersion = serde_ipld_dagcbor::from_slice(&cbor).unwrap();
    assert_eq!(
        decoded,
        DropBundleVersion::UnknownVersion,
        "an unknown future tag must map to the UnknownVersion catch-all"
    );
}

#[test]
fn parse_cbor_bytes_rejects_oversized_input_before_decode_row_d80() {
    // Row D-80 / D-67 (R19): an oversized/hostile blob must be a TYPED
    // reject BEFORE the CBOR deserialize — fail-closed bounded-decode,
    // not an OOM. Feed a blob well over DROP_BUNDLE_MAX_SIZE_BYTES.
    let oversized = vec![0u8; DROP_BUNDLE_MAX_SIZE_BYTES + 1];
    let result = DropBundle::parse_cbor_bytes(&oversized);
    match result {
        Err(DropBundleError::CodecError(msg)) => {
            assert!(
                msg.contains("DROP_BUNDLE_MAX_SIZE_BYTES"),
                "oversized reject must name the size cap, got {msg:?}"
            );
        }
        other => {
            panic!("oversized bundle must reject typed (not OOM/generic decode), got {other:?}")
        }
    }

    // A legitimately-sized bundle must still parse cleanly (the cap must
    // not reject valid input).
    let issuer_kp = benten_id::keypair::Keypair::generate();
    let recipient_kp = benten_id::keypair::Keypair::generate();
    let bundle = DropBundle::build_5_recipe_bundle_for_recipient(&issuer_kp, &recipient_kp);
    let cbor = bundle.to_cbor_bytes().unwrap();
    assert!(
        cbor.len() <= DROP_BUNDLE_MAX_SIZE_BYTES,
        "a real 5-recipe bundle ({} bytes) must be under the cap ({})",
        cbor.len(),
        DROP_BUNDLE_MAX_SIZE_BYTES
    );
    assert!(
        DropBundle::parse_cbor_bytes(&cbor).is_ok(),
        "a legitimately-sized bundle must parse under the cap"
    );
}

#[test]
fn v1_and_synthetic_wire_bytes_are_byte_identical_after_catch_all() {
    // Freeze guard: adding `#[serde(other)] UnknownVersion` must NOT
    // perturb the serialized bytes of the two live arms. These are the
    // exact byte strings the freeze/golden round-trips depend on.
    let v1 = serde_ipld_dagcbor::to_vec(&DropBundleVersion::V1).unwrap();
    let synth = serde_ipld_dagcbor::to_vec(&DropBundleVersion::Synthetic(7)).unwrap();

    // `a1 63 746167 62 5631` = {"tag":"V1"}
    assert_eq!(
        v1,
        [0xa1, 0x63, 0x74, 0x61, 0x67, 0x62, 0x56, 0x31],
        "V1 wire bytes must be frozen-identical"
    );
    // `a2 63 746167 69 53796e746865746963 65 76616c7565 07` = {"tag":"Synthetic","value":7}
    assert_eq!(
        synth,
        [
            0xa2, 0x63, 0x74, 0x61, 0x67, 0x69, 0x53, 0x79, 0x6e, 0x74, 0x68, 0x65, 0x74, 0x69,
            0x63, 0x65, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x07
        ],
        "Synthetic wire bytes must be frozen-identical"
    );

    // Round-trip must still recover the exact arms.
    assert_eq!(
        serde_ipld_dagcbor::from_slice::<DropBundleVersion>(&v1).unwrap(),
        DropBundleVersion::V1
    );
    assert_eq!(
        serde_ipld_dagcbor::from_slice::<DropBundleVersion>(&synth).unwrap(),
        DropBundleVersion::Synthetic(7)
    );
}
