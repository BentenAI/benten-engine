//! Manifest builder fixtures. F3 owns, F1/F2 + Family G consume.
//!
//! Per R2 §13.5. RED-PHASE stub — G24-D wave fills real signing /
//! validation / DAG-CBOR encoding.

use benten_core::Cid;
use benten_id::did::Did;
use benten_id::did_rotation::{RotationAttestation, rotate_keypair};
use benten_id::keypair::Keypair;
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
///
/// Phase 4-Foundation G24-D: InstallRecord now carries `plugin_did`
/// (UCAN audience handle per CLAUDE.md #18 retense). This fixture
/// supplies the standard test plugin-DID; production install paths
/// mint a fresh plugin-DID via OsRng per D-4F-16.
pub fn stub_install_record(manifest_cid: Cid) -> InstallRecord {
    InstallRecord {
        manifest_cid,
        plugin_did: stub_plugin_did(),
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

// =====================================================================
// G24-D-FP-2 fixtures — RotationLog + key rotation round-trip helpers
// =====================================================================
//
// Per phase-4-backlog §4.10: RotationLog + HLC-monotonic-strict
// integration at install/load surfaces. These helpers ship the real
// Ed25519 keypair construction + signed RotationAttestation used by
// the un-ignored G24-D-FP-2 test pins.

/// Construct a fresh keypair under OsRng. Used in rotation tests where
/// the *identity* of the key matters but the specific bytes don't.
#[allow(dead_code)]
pub fn fresh_keypair() -> Keypair {
    Keypair::generate()
}

/// Build a signed RotationAttestation `prev → next` at the given HLC.
/// Caller controls HLC ordering for the HLC-monotonic-strict tests.
#[allow(dead_code)]
pub fn signed_rotation_event(old: &Keypair, new: &Keypair, hlc: u64) -> RotationAttestation {
    let old_did = old.public_key().to_did();
    rotate_keypair(&old_did, old, new, hlc)
        .expect("rotation event signs cleanly under correctly-paired key")
}

/// Build a manifest whose `peer_did` is the public key of `kp` and
/// whose `content_cid` + `peer_signature` are populated. Used by the
/// rotation-warning round-trip test to exercise
/// `validate_with_rotation_log`.
#[allow(dead_code)]
pub fn manifest_signed_by(kp: &Keypair) -> PluginManifest {
    let mut m = minimal_manifest();
    m.peer_did = kp.public_key().to_did();
    m.content_cid = m.compute_content_cid();
    m.peer_signature = benten_platform_foundation::sign_manifest(&m, kp);
    m
}

/// 15 ErrorCode string forms minted at G24-D wave (post-R5 surface) per
/// §3.5g cross-language rule-mirror.
///
/// At HEAD these do NOT exist in `benten-errors`; the
/// `plugin_error_codes_atomic_rust_ts_mirror_pin` test proves their
/// post-G24-D presence via round-trip through `ErrorCode::from_str` +
/// presence in the TS mirror at `packages/engine/src/errors.generated.ts`
/// + heading entry in `docs/ERROR-CATALOG.md`.
///
/// Per Ben's R4-triage §7 ratification (2026-05-11):
/// - The TS mirror canonical location is
///   `packages/engine/src/errors.generated.ts` (NOT a new
///   `packages/error-codes/` package).
/// - `E_PLUGIN_DEVICE_ATTESTATION_FORGED` (renamed from
///   `E_DEVICE_ATTESTATION_FORGED_AT_PLUGIN_SHARE`) keeps the
///   `E_PLUGIN_*` prefix family for grep + subset-closure clarity.
/// - CATALOG_VARIANT_COUNT math: 27 minted across G23-A (9) + G23-B (3) +
///   G24-D (15) = 27 minted / 10 absorbed / 17 net new (118 → 135).
pub const G24_D_ERROR_CODES: &[&str] = &[
    "E_PLUGIN_MANIFEST_INVALID",
    "E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID",
    "E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID",
    "E_PLUGIN_CONTENT_PEER_KEY_ROTATED",
    "E_PLUGIN_AUTHOR_NOT_TRUSTED",
    "E_PLUGIN_INSTALL_CONSENT_REQUIRED",
    "E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE",
    "E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN",
    "E_PLUGIN_CONTENT_CID_MISMATCH",
    "E_PLUGIN_NEW_VERSION_AVAILABLE",
    "E_PLUGIN_HETEROGENEITY_INCOMPATIBLE",
    "E_PLUGIN_META_COMPOSITION_CYCLE_REJECTED",
    "E_PLUGIN_DEVICE_ATTESTATION_FORGED",
    "E_PLUGIN_LIBRARY_INDEX_TAMPER",
    "E_REGISTRY_DISCOVERY_TIMEOUT",
];

// =====================================================================
// R4b-FP-1 helpers — substantive install-lifecycle fixtures
// =====================================================================

/// Build an honestly-signed manifest by `kp` with the supplied
/// requires-scopes. Used by R4b-FP-1 install_plugin tests.
#[allow(dead_code)]
pub fn signed_manifest_by(
    kp: &Keypair,
    plugin_name: &str,
    requires_scopes: &[&str],
) -> PluginManifest {
    let mut m = PluginManifest {
        plugin_name: plugin_name.to_string(),
        content_cid: stub_cid_zero(),
        peer_did: kp.public_key().to_did(),
        peer_signature: vec![0u8; 64],
        requires: requires_scopes
            .iter()
            .map(|s| CapRequirement {
                scope: (*s).to_string(),
            })
            .collect(),
        shares: SharesPolicy {
            default: SharesPolicyDefault::None,
            rules: None,
        },
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    };
    m.content_cid = m.compute_content_cid();
    m.peer_signature = benten_platform_foundation::sign_manifest(&m, kp);
    m
}

/// Build a properly-signed InstallRecord by `user_kp` over `manifest_cid`.
#[allow(dead_code)]
pub fn signed_install_record(
    user_kp: &Keypair,
    manifest_cid: Cid,
    plugin_did: Did,
    nonce_byte: u8,
) -> InstallRecord {
    let mut record = InstallRecord {
        manifest_cid,
        plugin_did,
        consenting_user_did: user_kp.public_key().to_did(),
        user_signature: vec![0u8; 64],
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce: vec![nonce_byte; 16],
        granted_caps_bytes: vec![],
    };
    let payload = record.signing_payload();
    record.user_signature = user_kp.sign(&payload).to_bytes().to_vec();
    record
}

/// R6-FP-A-fp caller-mint-first helper.
///
/// Mints a real `PluginDidHandle` via `benten_id::plugin_did::mint()`,
/// inserts it into the supplied `PluginDidStore`, and returns the DID.
/// Use this everywhere a test would otherwise build an `InstallRecord`
/// with a `Did::from_string_unchecked` placeholder — the new
/// `install_plugin` Step 8 enforces that `install_record.plugin_did`
/// is present in the store + matches `InstallContext::expected_plugin_did`,
/// so placeholder DIDs no longer work.
#[allow(dead_code)]
pub fn mint_and_insert_plugin_did(store: &mut benten_id::plugin_did::PluginDidStore) -> Did {
    let handle = benten_id::plugin_did::mint();
    let did = handle.did().clone();
    store.insert(handle);
    did
}
