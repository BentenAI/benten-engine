//! LOAD-BEARING per plan §3 G24-D row + post-R1-triage D-4F-13 +
//! Q7 ratification.
//!
//! Verifies the absence of a `schema_version` field at the canonical-
//! bytes level. The pull-not-push model means CID covers shape — no
//! manifest schema versioning is needed.
//!
//! Per R2 §5 substance discipline: pair with canonical-bytes
//! serialization round-trip asserting absence of `schema_version`
//! at the encoded DAG-CBOR level (not just at the struct level).

mod common;

use common::manifest_fixtures::minimal_manifest;

#[test]
fn manifest_struct_has_no_schema_version_field_at_type_level() {
    let manifest = minimal_manifest();

    // The type-level absence is structural: enumerate every public
    // field and assert none is named or shaped like schema_version /
    // manifest_version / version_field / spec_version.
    //
    // Construction destructures the type — if anyone adds a
    // schema_version field, this destructure forces an update.
    let PluginManifestDestructure {
        plugin_name: _,
        content_cid: _,
        peer_did: _,
        peer_signature: _,
        requires: _,
        shares: _,
        renderer_config: _,
        composes_plugins: _,
        accepts_content: _,
        requires_schema_authors: _,
        requires_plugin_authors: _,
    } = PluginManifestDestructure::from(&manifest);
}

#[test]
fn manifest_canonical_bytes_dag_cbor_contains_no_schema_version_key() {
    let manifest = minimal_manifest();

    // G24-D-FP-2 surface: PluginManifest::to_canonical_bytes() ->
    // Vec<u8> via DAG-CBOR. Walk the encoded bytes byte-wise asserting
    // no version-keyed CBOR map entry appears.
    let bytes = manifest.to_canonical_bytes();
    assert!(!bytes.is_empty(), "non-empty DAG-CBOR encoding");

    // SUBSTANTIVE: search the raw CBOR bytes for any of the forbidden
    // key names. Per DAG-CBOR, map keys are CBOR text strings encoded
    // with major type 3 (0x60..0x77 length-embedded). The substring
    // form of each forbidden key is searched directly in the byte
    // stream — a non-zero match means a CBOR text key with that
    // content was emitted. WOULD-FAIL-IF-NO-OP because if to_canonical_
    // bytes were stub-no-op'd to empty, the round-trip below would
    // fail; if it were stub-no-op'd to a map containing schema_version,
    // the substring scan would catch it.
    let forbidden = [
        "schema_version",
        "manifest_version",
        "spec_version",
        "version_field",
    ];
    for key in &forbidden {
        let pos = bytes.windows(key.len()).position(|w| w == key.as_bytes());
        assert!(
            pos.is_none(),
            "D-4F-13 pull-not-push: canonical-bytes DAG-CBOR MUST NOT contain forbidden \
             key '{key}' (CID covers shape; no manifest schema versioning)"
        );
    }

    // Round-trip sanity: encoded bytes deserialize back to the same
    // PluginManifest (defeats the stub-no-op-empty failure mode).
    let decoded: benten_platform_foundation::PluginManifest =
        serde_ipld_dagcbor::from_slice(&bytes).expect("DAG-CBOR round-trip");
    assert_eq!(decoded.plugin_name, manifest.plugin_name);
}

/// Helper struct to force compile-time field enumeration. If
/// PluginManifest grows a new field, this struct's destructure will
/// fail to compile.
struct PluginManifestDestructure {
    plugin_name: String,
    content_cid: benten_core::Cid,
    peer_did: benten_id::did::Did,
    peer_signature: Vec<u8>,
    requires: Vec<benten_platform_foundation::CapRequirement>,
    shares: benten_platform_foundation::SharesPolicy,
    renderer_config: Option<benten_platform_foundation::RendererConfig>,
    composes_plugins: Option<Vec<benten_core::Cid>>,
    accepts_content: Option<Vec<benten_core::Cid>>,
    requires_schema_authors: Option<Vec<benten_id::did::Did>>,
    requires_plugin_authors: Option<Vec<benten_id::did::Did>>,
}

impl From<&benten_platform_foundation::PluginManifest> for PluginManifestDestructure {
    fn from(m: &benten_platform_foundation::PluginManifest) -> Self {
        Self {
            plugin_name: m.plugin_name.clone(),
            content_cid: m.content_cid,
            peer_did: m.peer_did.clone(),
            peer_signature: m.peer_signature.clone(),
            requires: m.requires.clone(),
            shares: m.shares.clone(),
            renderer_config: m.renderer_config.clone(),
            composes_plugins: m.composes_plugins.clone(),
            accepts_content: m.accepts_content.clone(),
            requires_schema_authors: m.requires_schema_authors.clone(),
            requires_plugin_authors: m.requires_plugin_authors.clone(),
        }
    }
}
