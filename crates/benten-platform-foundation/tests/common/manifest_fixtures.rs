//! Manifest builder fixtures. F3 owns, F1/F2 + Family G consume.
//!
//! Per R2 §13.5. RED-PHASE stub — G24-D wave fills real signing /
//! validation / DAG-CBOR encoding.

use benten_core::Cid;
use benten_id::did::Did;
use benten_platform_foundation::{
    CapRequirement, InstallRecord, PluginManifest, RendererBackend, RendererConfig, SharesPolicy,
    SharesPolicyDefault, SharesRule, SharesTarget,
};

/// Stub CID for tests; not derived from real bytes at R3.
pub fn stub_cid_zero() -> Cid {
    Cid::from_blake3_digest([0u8; 32])
}

/// Second stub CID for distinct-CID tests.
pub fn stub_cid_one() -> Cid {
    let mut digest = [0u8; 32];
    digest[0] = 1;
    Cid::from_blake3_digest(digest)
}

/// Third stub CID for content-author-rotation tests.
pub fn stub_cid_two() -> Cid {
    let mut digest = [0u8; 32];
    digest[0] = 2;
    Cid::from_blake3_digest(digest)
}

/// Stub peer-DID for tests.
pub fn stub_peer_did_alice() -> Did {
    Did::from_string_unchecked("did:key:z6MkAlice".to_string())
}

/// Second peer-DID — used in T6b "substituted peer-DID" tests.
pub fn stub_peer_did_attacker() -> Did {
    Did::from_string_unchecked("did:key:z6MkAttacker".to_string())
}

/// Stub user-DID — root of cap chain per CLAUDE.md #18 Layer 1.
pub fn stub_user_did() -> Did {
    Did::from_string_unchecked("did:key:z6MkUser".to_string())
}

/// Stub plugin-DID — UCAN audience handle minted at install. NOT an
/// attested sub-identity of user-DID per CLAUDE.md #18 retense.
pub fn stub_plugin_did() -> Did {
    Did::from_string_unchecked("did:key:z6MkPlugin".to_string())
}

/// Minimal valid-shape manifest (no signing logic at R3).
pub fn minimal_manifest() -> PluginManifest {
    PluginManifest {
        plugin_name: "test-plugin".to_string(),
        content_cid: stub_cid_zero(),
        peer_did: stub_peer_did_alice(),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement {
            scope: "store:notes:read".to_string(),
        }],
        shares: SharesPolicy {
            default: SharesPolicyDefault::None,
            rules: None,
        },
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    }
}

/// Admin-UI-v0-shaped manifest with renderer config (browser_wasm32 +
/// tauri_embedded_webview both supported per Ben D-4F-4).
pub fn admin_ui_v0_manifest() -> PluginManifest {
    PluginManifest {
        plugin_name: "admin-ui-v0".to_string(),
        content_cid: stub_cid_zero(),
        peer_did: stub_peer_did_alice(),
        peer_signature: vec![0u8; 64],
        requires: vec![
            CapRequirement {
                scope: "private:admin-ui-private:*".to_string(),
            },
            CapRequirement {
                scope: "store:plugins:read".to_string(),
            },
            CapRequirement {
                scope: "host:time:now".to_string(),
            },
        ],
        shares: SharesPolicy {
            default: SharesPolicyDefault::None,
            rules: None,
        },
        renderer_config: Some(RendererConfig {
            output_format: "html_json".to_string(),
            renderer_backends: Some(vec![
                RendererBackend::BrowserWasm32,
                RendererBackend::TauriEmbeddedWebview,
            ]),
            hosting_target: Some("browser_wasm32".to_string()),
            bundle_size_budget_kb: Some(512),
        }),
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    }
}

/// Manifest with `shares: any` for delegation tests.
pub fn manifest_with_shares_any() -> PluginManifest {
    PluginManifest {
        shares: SharesPolicy {
            default: SharesPolicyDefault::Any,
            rules: None,
        },
        ..minimal_manifest()
    }
}

/// Manifest with explicit matching rule for delegation tests.
pub fn manifest_with_shares_matching_rule(
    cap_pattern: &str,
    target: SharesTarget,
) -> PluginManifest {
    PluginManifest {
        shares: SharesPolicy {
            default: SharesPolicyDefault::Matching,
            rules: Some(vec![SharesRule {
                cap_pattern: cap_pattern.to_string(),
                target,
            }]),
        },
        ..minimal_manifest()
    }
}

/// Manifest with `host:sandbox:exec` requires — used for heterogeneity
/// rejection tests (ds-r1-8).
pub fn manifest_requires_sandbox_exec() -> PluginManifest {
    PluginManifest {
        requires: vec![CapRequirement {
            scope: "host:sandbox:exec".to_string(),
        }],
        ..minimal_manifest()
    }
}

/// Stub install record — user-DID signed (signature is stub bytes at R3).
pub fn stub_install_record(manifest_cid: Cid) -> InstallRecord {
    InstallRecord {
        manifest_cid,
        consenting_user_did: stub_user_did(),
        user_signature: vec![0u8; 64],
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce: vec![0u8; 16],
        granted_caps_bytes: vec![],
    }
}

/// Manifest declaring `accepts_content` (CID-keyed cross-plugin
/// reference per CLAUDE.md #18 Q4 ratification).
pub fn manifest_with_accepts_content(refs: Vec<Cid>) -> PluginManifest {
    PluginManifest {
        accepts_content: Some(refs),
        ..minimal_manifest()
    }
}
