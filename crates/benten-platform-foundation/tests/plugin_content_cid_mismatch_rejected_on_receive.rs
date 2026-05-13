//! G24-D row pin — pull-model CID verification on receive.
//!
//! Per docs/PLUGIN-MANIFEST.md §4.1 step 2(a): receiver verifies bytes
//! hash to declared content-CID. Mismatch surfaces
//! `E_PLUGIN_CONTENT_CID_MISMATCH`.
//!
//! Defends against T6a substitution-at-transit attacks.

mod common;

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::module_ecosystem::{InstallerShape, install_plugin};
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, RendererBackend, RendererConfig, SharesPolicy, sign_manifest,
};

fn build_signed_manifest(name: &str, author: &Keypair) -> PluginManifest {
    let mut manifest = PluginManifest {
        plugin_name: name.to_string(),
        content_cid: Cid::from_blake3_digest([0u8; 32]),
        peer_did: author.public_key().to_did(),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement::new("store:notes:read")],
        shares: SharesPolicy::none(),
        renderer_config: Some(RendererConfig {
            output_format: "html_json".to_string(),
            renderer_backends: Some(vec![RendererBackend::BrowserWasm32]),
            hosting_target: None,
            bundle_size_budget_kb: Some(256),
        }),
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    };
    manifest.content_cid = manifest.compute_content_cid();
    manifest.peer_signature = sign_manifest(&manifest, author);
    manifest
}

#[test]
fn install_path_rejects_bytes_with_announced_cid_mismatch_with_typed_error() {
    // SUBSTANTIVE per pim-2 §3.6b: build a real signed manifest;
    // pass install_plugin a DIFFERENT claimed CID. Expect typed
    // PluginContentCidMismatch. Would-FAIL if install_plugin skipped
    // step 2(a) (CID verification).
    let author = Keypair::generate();
    let manifest = build_signed_manifest("test-app", &author);
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");

    let mut library = PluginLibrary::new();
    // Claim a CID that does NOT match the manifest's actual CID.
    let bogus_cid = Cid::from_blake3_digest([0xEEu8; 32]);

    let result = install_plugin(
        &mut library,
        &bytes,
        &bogus_cid,
        InstallerShape::FullPeer,
        1,
        &|_| None,
    );
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("MUST reject CID mismatch"),
    };

    assert_eq!(
        err,
        ErrorCode::PluginContentCidMismatch,
        "install path MUST surface typed PluginContentCidMismatch; \
         would-FAIL if step 2(a) CID verification skipped"
    );

    // Defense-in-depth: rejection leaves library state UNCHANGED.
    assert!(library.is_empty(), "rejected install MUST NOT commit");
}

#[test]
fn install_path_admits_bytes_when_announced_cid_matches_signed_manifest() {
    // SUBSTANTIVE boundary per pim-2 §3.6b: complementary positive arm
    // — the CID verification check is not over-strict; matching CID
    // admits. Would-FAIL if verification rejected even matched CIDs.
    let author = Keypair::generate();
    let manifest = build_signed_manifest("ok-app", &author);
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");

    let mut library = PluginLibrary::new();
    let cid = manifest.content_cid;

    let outcome = install_plugin(
        &mut library,
        &bytes,
        &cid,
        InstallerShape::FullPeer,
        1,
        &|_| None,
    );
    assert!(outcome.is_ok(), "matched CID MUST admit");
    assert_eq!(library.len(), 1, "library now holds the entry");
}
