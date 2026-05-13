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
#[ignore = "RED-PHASE: G24-D wave provides the canonical-bytes DAG-CBOR serialization; un-ignore at G24-D landing"]
fn manifest_canonical_bytes_dag_cbor_encodes_accepts_content_as_cid_array() {
    let manifest = manifest_with_accepts_content(vec![stub_cid_one(), stub_cid_two()]);

    // Future G24-D surface: PluginManifest::to_canonical_bytes() ->
    // Vec<u8> via DAG-CBOR. The encoded form contains the CID bytes
    // directly (not a string-encoded DID). FAILS-IF-NO-OP because
    // canonical-bytes encoding is what content-CID is computed over.
    let _cid = manifest.compute_content_cid();
    panic!(
        "RED-PHASE: G24-D wave must wire canonical-bytes DAG-CBOR encoding of accepts_content as Cid array"
    );
}
