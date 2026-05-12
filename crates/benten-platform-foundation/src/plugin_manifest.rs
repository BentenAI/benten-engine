//! Phase-4-Foundation R3 RED-PHASE stub for the FULL plugin manifest.
//!
//! Type-shape only. NO validation, NO signing, NO serialization logic
//! lives here at R3. G24-D wave fills all of it.
//!
//! This stub exists so that:
//! - Family F3 (this family) test pins compile-but-fail at the `use`
//!   line per canonical RED-PHASE shape;
//! - Family G + Family F1 + Family F2 — which consume this type to
//!   author their own test pins (cap-policy backend chain validation,
//!   atrium-merge manifest-envelope recheck, sync defenses) — have a
//!   stable type to import.
//!
//! Schema per `docs/PLUGIN-MANIFEST.md` §3 + CLAUDE.md baked-in #18
//! "Implementation refinements" four-identity-concepts model.

use benten_core::Cid;
use benten_id::did::Did;

/// FULL plugin manifest. Content-addressed; canonical-bytes DAG-CBOR
/// at G24-D landing.
///
/// **Stub.** At R3, the constructor and all methods `unimplemented!()`.
/// Test pins exercise the type-shape only at this stage.
#[derive(Debug, Clone)]
pub struct PluginManifest {
    // Identity (per docs/PLUGIN-MANIFEST.md §3)
    pub plugin_name: String,
    pub content_cid: Cid,
    pub peer_did: Did,
    pub peer_signature: Vec<u8>,

    // Capability envelope
    pub requires: Vec<CapRequirement>,
    pub shares: SharesPolicy,

    // Renderer + composition
    pub renderer_config: Option<RendererConfig>,
    pub composes_plugins: Option<Vec<Cid>>,

    // Cross-references (content-CID-keyed; per CLAUDE.md #18
    // "Cross-plugin/schema references use content-CID, not author-DID")
    pub accepts_content: Option<Vec<Cid>>,
    pub requires_schema_authors: Option<Vec<Did>>,
    pub requires_plugin_authors: Option<Vec<Did>>,
}

impl PluginManifest {
    /// Validate the manifest envelope against schema rules + signature.
    ///
    /// **Stub.** G24-D fills.
    pub fn validate(&self) -> Result<(), ()> {
        unimplemented!("R3 RED-PHASE stub — G24-D fills validation logic")
    }

    /// Compute the content-CID over the canonical-bytes DAG-CBOR
    /// encoding of the manifest body.
    ///
    /// **Stub.** G24-D fills.
    pub fn compute_content_cid(&self) -> Cid {
        unimplemented!("R3 RED-PHASE stub — G24-D fills CID computation")
    }
}

/// Capability requirement entry — typed scope the plugin needs.
///
/// Scope shape per `docs/PLUGIN-MANIFEST.md` §6 cap-scope grammar
/// (`requires:<plugin_did>:<requirement_path>` or domain-typed shapes
/// such as `store:notes:read` / `host:time:now`).
#[derive(Debug, Clone)]
pub struct CapRequirement {
    pub scope: String,
}

/// Delegation policy envelope (the `shares` half of the manifest).
///
/// Per `docs/PLUGIN-MANIFEST.md` §3.2.
#[derive(Debug, Clone)]
pub struct SharesPolicy {
    pub default: SharesPolicyDefault,
    pub rules: Option<Vec<SharesRule>>,
}

/// Default share-disposition when no rule matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharesPolicyDefault {
    /// Conservative v0 default — no delegation permitted.
    None,
    /// Any delegation permitted (rare; used by trust-anchor plugins).
    Any,
    /// Matching `rules` permitted; non-matching denied.
    Matching,
}

/// Per-cap or per-target delegation rule.
#[derive(Debug, Clone)]
pub struct SharesRule {
    pub cap_pattern: String,
    pub target: SharesTarget,
}

/// Target of a `SharesRule`.
#[derive(Debug, Clone)]
pub enum SharesTarget {
    /// Any plugin permitted (within `cap_pattern`).
    Any,
    /// A specific plugin-DID permitted.
    PluginDid(Did),
    /// Any plugin authored by a specific peer-DID permitted.
    PluginAuthor(Did),
}

/// Renderer configuration (optional; per `docs/PLUGIN-MANIFEST.md` §7).
#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub output_format: String,
    pub renderer_backends: Option<Vec<RendererBackend>>,
    pub hosting_target: Option<String>,
    pub bundle_size_budget_kb: Option<u32>,
}

/// Renderer backend handle. Concrete enumeration deferred to G24-D.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RendererBackend {
    BrowserWasm32,
    TauriEmbeddedWebview,
    Other(String),
}

/// Install-time consent record. User-DID signs over (manifest_cid +
/// timestamp + nonce). G24-D fills the actual `Hlc` / signature surface.
///
/// Per `docs/PLUGIN-MANIFEST.md` §3 InstallRecord schema.
#[derive(Debug, Clone)]
pub struct InstallRecord {
    pub manifest_cid: Cid,
    pub consenting_user_did: Did,
    pub user_signature: Vec<u8>,
    /// Stub `Hlc` shape at R3 — G24-D wires the real `benten-core::Hlc`.
    pub timestamp_stub_nanos: u64,
    pub nonce: Vec<u8>,
    /// Granted UCAN capability grants from user-DID to plugin-DID.
    /// Concrete `CapGrant` shape lives in `benten-caps`; this is a
    /// payload-bytes placeholder at R3.
    pub granted_caps_bytes: Vec<Vec<u8>>,
}
