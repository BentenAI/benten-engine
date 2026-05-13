//! LOAD-BEARING per plan §3 G24-D row + post-R1-triage Q4.
//!
//! Verifies cross-plugin/schema references use `accepts_content:
//! [hash, ...]` (CID-keyed), NOT `accepts_author: [did, ...]` (DID-
//! keyed). Would-FAIL if the reference shape were author-DID-keyed.
//!
//! Per CLAUDE.md #18 "Cross-plugin/schema references use content-CID":
//! authors can rotate keys without breaking downstream references —
//! the references are CID-keyed.

mod common;

use common::manifest_fixtures::{manifest_with_accepts_content, stub_cid_one, stub_cid_two};

#[test]
fn manifest_accepts_content_field_is_cid_list_not_did_list() {
    let manifest = manifest_with_accepts_content(vec![stub_cid_one(), stub_cid_two()]);

    // POSITIVE shape: accepts_content carries CIDs.
    let refs = manifest.accepts_content.as_ref().expect("set");
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0], stub_cid_one());
    assert_eq!(refs[1], stub_cid_two());

    // NEGATIVE shape: at the type level, there is no
    // `manifest.accepts_author` field of Vec<Did>. This is a compile-
    // time guarantee: if a future change added that field, this test
    // file's mod common::manifest_fixtures::manifest_with_accepts_content
    // signature would need to change OR the field would be unused —
    // either way, the substantive shape would be exercised.
    //
    // The `requires_schema_authors` + `requires_plugin_authors` fields
    // are DISTINCT: they are install-time TRUST LISTS for which peer-DIDs
    // the user trusts to author schemas/plugins. They are NOT cross-plugin
    // reference shapes. The cross-plugin reference shape is content_cid-
    // keyed via accepts_content.
    assert!(
        manifest.requires_schema_authors.is_none(),
        "accepts_content (CID-keyed) is the cross-plugin reference shape; requires_schema_authors is a separate trust-list concept"
    );
}

#[test]
fn manifest_canonical_bytes_dag_cbor_encodes_accepts_content_as_cid_array() {
    let cid_one = stub_cid_one();
    let cid_two = stub_cid_two();
    let manifest = manifest_with_accepts_content(vec![cid_one, cid_two]);

    // G24-D-FP-2 surface: PluginManifest::to_canonical_bytes() ->
    // Vec<u8> via DAG-CBOR. SUBSTANTIVE arm: encode the manifest +
    // round-trip via serde_ipld_dagcbor::from_slice, then inspect the
    // accepts_content field on the decoded value.
    let bytes = manifest.to_canonical_bytes();
    assert!(
        !bytes.is_empty(),
        "to_canonical_bytes emits non-empty DAG-CBOR"
    );

    // Round-trip through DAG-CBOR; the decoded manifest must preserve
    // accepts_content as a Vec<Cid> (not a Vec<String>). FAILS-IF-NO-OP
    // because a no-op that returned empty bytes would not deserialize.
    let decoded: benten_platform_foundation::PluginManifest =
        serde_ipld_dagcbor::from_slice(&bytes).expect("DAG-CBOR round-trip");
    let refs = decoded
        .accepts_content
        .as_ref()
        .expect("accepts_content preserved through canonical-bytes round-trip");
    assert_eq!(
        refs.len(),
        2,
        "accepts_content array length preserved through canonical-bytes round-trip"
    );
    assert_eq!(
        refs[0], cid_one,
        "accepts_content[0] preserved as Cid (not string-encoded DID per CLAUDE.md #18 Q4)"
    );
    assert_eq!(
        refs[1], cid_two,
        "accepts_content[1] preserved as Cid (not string-encoded DID per CLAUDE.md #18 Q4)"
    );

    // OBSERVABLE: the encoding is CID-byte-shaped not DID-string-shaped.
    // benten-core's `Cid` (36 bytes; v1 header + multicodec + multihash
    // + 32-byte BLAKE3 digest) serializes via `serde_bytes::Bytes` as a
    // CBOR byte-string (major type 2). A 36-byte byte string under CBOR
    // is `0x58 0x24` (length-as-1-byte = 36). Search for at least 2
    // occurrences of the byte-string marker pair — the test fixture
    // emits two accepts_content CIDs, each emitted as its own byte
    // string. A DID-string-shaped encoding would use major-type-3
    // (text string, 0x60..0x77 short / 0x78 long) instead.
    let cbor_36byte_marker_hits = bytes
        .windows(2)
        .filter(|w| w[0] == 0x58 && w[1] == 0x24)
        .count();
    assert!(
        cbor_36byte_marker_hits >= 2,
        "DAG-CBOR canonical-bytes must contain at least 2 36-byte CBOR byte strings \
         (the accepts_content Cid pair); got {cbor_36byte_marker_hits}. \
         Confirms accepts_content is CID-byte-shaped (not DID-string-shaped) per CLAUDE.md #18 Q4."
    );
    // Type-level guarantee: at the rust type level `accepts_content` is
    // `Option<Vec<Cid>>` (not `Option<Vec<Did>>` or `Option<Vec<String>>`).
    // The round-trip-decoded value (asserted equal to the original Cid
    // values above) closes the negative: if it were string-encoded a
    // Vec<Cid> deserialize would fail.
    let _: Option<Vec<benten_core::Cid>> = decoded.accepts_content;
}
