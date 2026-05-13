//! Phase-4-Foundation G24-D — FULL plugin manifest schema.
//!
//! Implements the CLAUDE.md baked-in #18 four-identity-concepts model:
//!
//! 1. **Content-CID** — what the plugin IS (canonical-bytes DAG-CBOR
//!    encoding of the manifest body).
//! 2. **Peer-DID signature on original content** — provenance.
//! 3. **Plugin-DID minted at install** — UCAN audience handle AND
//!    constrained issuer within manifest envelope (NOT an attested
//!    sub-identity of user-DID).
//! 4. **User-DID** — trust anchor + signs `InstallRecord`s + issues
//!    UCAN delegations with `audience = plugin-DID`.
//!
//! See `docs/PLUGIN-MANIFEST.md` for the engineer-facing reference.

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use ed25519_dalek::Verifier;
use serde::{Deserialize, Serialize};

// =====================================================================
// PluginManifest
// =====================================================================

/// FULL plugin manifest. Content-addressed; canonical-bytes DAG-CBOR.
///
/// **Four-identity-concepts model** per CLAUDE.md baked-in #18:
/// - `content_cid` (concept 1) — set after `compute_content_cid()`.
/// - `peer_did` + `peer_signature` (concept 2) — provenance.
/// - Plugin-DID (concept 3) — NOT carried in the manifest; minted at
///   install time per `benten_id::plugin_did::PluginDidStore::mint`.
/// - User-DID (concept 4) — NOT carried in the manifest; signs the
///   `InstallRecord` that references the manifest.
///
/// **Pull-not-push** (D-4F-13): no `schema_version` field — CID covers
/// shape. Version selection is per-user-local.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Human-readable name. Not unique; `content_cid` is the unique
    /// handle.
    pub plugin_name: String,

    /// Content-CID over canonical-bytes DAG-CBOR encoding of the body.
    /// Filled by `compute_content_cid()`; verified at install via
    /// `verify_content_cid_matches`.
    pub content_cid: Cid,

    /// Peer-DID of original author / sharer. Provenance — receiver
    /// verifies signature; rotation handled by RotationLog (warning,
    /// not hard reject per D-4F-12).
    pub peer_did: Did,

    /// Ed25519 signature over `(content_cid_bytes || canonical_body)`.
    /// 64 bytes (Ed25519 detached signature).
    pub peer_signature: Vec<u8>,

    /// Capability requirements — caps the plugin needs to function.
    pub requires: Vec<CapRequirement>,

    /// Delegation policy — what this plugin may delegate to OTHER
    /// plugins at runtime, within the user-consented envelope.
    pub shares: SharesPolicy,

    /// Optional renderer config. `None` for SUBSCRIBE-only plugins.
    pub renderer_config: Option<RendererConfig>,

    /// Meta-plugin composition: CIDs of sub-plugins. Engine walks via
    /// 12-primitive vocabulary. Cycle detection at install.
    pub composes_plugins: Option<Vec<Cid>>,

    /// Cross-plugin / schema content references — CID-keyed per
    /// CLAUDE.md #18 (NOT author-DID-keyed).
    pub accepts_content: Option<Vec<Cid>>,

    /// Trust-list of peer-DIDs for schemas this plugin reads
    /// (PIN-trust to specific authors; default is CID-keyed).
    pub requires_schema_authors: Option<Vec<Did>>,

    /// Trust-list of peer-DIDs for plugins this plugin composes.
    pub requires_plugin_authors: Option<Vec<Did>>,
}

impl PluginManifest {
    /// Validate the manifest envelope: structural shape.
    ///
    /// # Errors
    ///
    /// `E_PLUGIN_MANIFEST_INVALID` on structural violation.
    pub fn validate(&self) -> Result<(), ErrorCode> {
        if self.plugin_name.is_empty() {
            return Err(ErrorCode::PluginManifestInvalid);
        }
        if self.peer_signature.len() != 64 {
            return Err(ErrorCode::PluginManifestInvalid);
        }
        if self.requires.is_empty() {
            return Err(ErrorCode::PluginManifestInvalid);
        }
        match self.shares.default {
            SharesPolicyDefault::Matching => {
                if self.shares.rules.as_ref().is_none_or(|r| r.is_empty()) {
                    return Err(ErrorCode::PluginManifestInvalid);
                }
            }
            SharesPolicyDefault::None | SharesPolicyDefault::Any => {}
        }
        for req in &self.requires {
            if req.scope.is_empty() {
                return Err(ErrorCode::PluginManifestInvalid);
            }
        }
        Ok(())
    }

    /// Compute the content-CID over the canonical-bytes DAG-CBOR
    /// encoding of the body with `content_cid` + `peer_signature`
    /// zeroed (chicken-and-egg + signature-bytes-not-yet-known).
    #[must_use]
    pub fn compute_content_cid(&self) -> Cid {
        let mut copy = self.clone();
        copy.content_cid = Cid::from_blake3_digest([0u8; 32]);
        copy.peer_signature = Vec::new();
        let bytes = serde_ipld_dagcbor::to_vec(&copy)
            .expect("plugin manifest serializes (programmer error if this fires)");
        let digest = blake3::hash(&bytes);
        Cid::from_blake3_digest(*digest.as_bytes())
    }

    /// Verify the stored `content_cid` matches the computed CID.
    ///
    /// # Errors
    ///
    /// `E_PLUGIN_CONTENT_CID_MISMATCH` if computed != stored.
    pub fn verify_content_cid_matches(&self) -> Result<(), ErrorCode> {
        let computed = self.compute_content_cid();
        if computed == self.content_cid {
            Ok(())
        } else {
            Err(ErrorCode::PluginContentCidMismatch)
        }
    }

    /// Verify the peer-DID signature on the manifest content.
    ///
    /// # Errors
    ///
    /// - `E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID` on signature failure.
    /// - `E_PLUGIN_CONTENT_CID_MISMATCH` if CID drift detected.
    pub fn verify_peer_signature(&self) -> Result<(), ErrorCode> {
        self.verify_content_cid_matches()?;
        if self.peer_signature.len() != 64 {
            return Err(ErrorCode::PluginContentPeerSignatureInvalid);
        }
        let pubkey = self
            .peer_did
            .resolve()
            .map_err(|_| ErrorCode::PluginContentPeerSignatureInvalid)?;
        let sig_bytes: [u8; 64] = self
            .peer_signature
            .as_slice()
            .try_into()
            .map_err(|_| ErrorCode::PluginContentPeerSignatureInvalid)?;
        let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        let msg = self.signing_payload();
        pubkey
            .as_verifying_key()
            .verify(&msg, &signature)
            .map_err(|_| ErrorCode::PluginContentPeerSignatureInvalid)
    }

    /// Bytes signed by the peer-DID: `content_cid_bytes || canonical_body`.
    #[must_use]
    pub fn signing_payload(&self) -> Vec<u8> {
        let mut copy = self.clone();
        copy.peer_signature = Vec::new();
        let body =
            serde_ipld_dagcbor::to_vec(&copy).expect("plugin manifest signing-payload serializes");
        let mut out = Vec::with_capacity(36 + body.len());
        out.extend_from_slice(self.content_cid.as_bytes());
        out.extend_from_slice(&body);
        out
    }

    /// Whether this manifest declares the `host:sandbox:exec` cap.
    /// Heterogeneity check fires on thin-compute-surface installs.
    #[must_use]
    pub fn requires_sandbox_exec(&self) -> bool {
        self.requires.iter().any(|r| r.scope == "host:sandbox:exec")
    }

    /// Canonical-bytes DAG-CBOR encoding of this **fully-populated**
    /// manifest (i.e., bytes preserve `content_cid` + `peer_signature`
    /// if set; nothing zeroed).
    ///
    /// G24-D-FP-2 surface (per `docs/future/phase-4-backlog.md §4.9`).
    /// Two distinct serializations exist on this type — don't confuse:
    ///
    /// - **`to_canonical_bytes` (this method)** — round-trippable
    ///   manifest bytes. Used by the chain validator + by test pins
    ///   that inspect the CBOR shape (CID array shape for
    ///   `accepts_content`; absence of `schema_version`-like keys).
    /// - **[`Self::signing_payload`]** — bytes WITH `content_cid` +
    ///   `peer_signature` zeroed (chicken-and-egg: signature can't
    ///   sign over itself). [`Self::compute_content_cid`] hashes the
    ///   signing-payload, NOT `to_canonical_bytes`.
    ///
    /// Test pins inspect the CBOR shape directly via this method.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        serde_ipld_dagcbor::to_vec(self)
            .expect("plugin manifest serializes (programmer error if this fires)")
    }

    /// Validate the manifest envelope, consulting a RotationLog for
    /// peer-DID key-rotation events.
    ///
    /// G24-D-FP-2 surface (per `docs/future/phase-4-backlog.md §4.10`).
    /// Returns:
    /// - `Ok(ValidationOutcome::Valid)` when structural validation +
    ///   signature pass + peer-DID is NOT in the RotationLog.
    /// - `Ok(ValidationOutcome::ValidWithWarning(RotatedKeyWarning))` when
    ///   structural validation + signature pass BUT the peer-DID has been
    ///   superseded in the RotationLog. Per D-4F-12 this is a WARNING by
    ///   default (NOT hard-reject); admin UI surfaces it; user decides
    ///   whether to trust.
    /// - `Err(ErrorCode)` on structural or signature failure.
    ///
    /// # Errors
    ///
    /// Propagates errors from [`Self::validate`] and
    /// [`Self::verify_peer_signature`].
    pub fn validate_with_rotation_log(
        &self,
        rotation_log: &benten_id::did_rotation::RotationLog,
    ) -> Result<ValidationOutcome, ErrorCode> {
        self.validate()?;
        self.verify_peer_signature()?;
        if rotation_log.is_superseded(&self.peer_did) {
            Ok(ValidationOutcome::ValidWithWarning(RotatedKeyWarning {
                rotated_peer_did: self.peer_did.clone(),
            }))
        } else {
            Ok(ValidationOutcome::Valid)
        }
    }
}

// =====================================================================
// ValidationOutcome — G24-D-FP-2 (per phase-4-backlog §4.10 + D-4F-12)
// =====================================================================

/// Outcome of [`PluginManifest::validate_with_rotation_log`].
///
/// Per D-4F-12: rotation surfaces a WARNING by default, not a
/// hard-reject. Strict-mode (future Phase-4-Meta opt-in) returns
/// `Err(E_PLUGIN_CONTENT_PEER_KEY_ROTATED)` instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationOutcome {
    /// All checks passed; no rotation event observed.
    Valid,
    /// All checks passed BUT peer-DID has been rotated. Admin UI
    /// surfaces the warning; user decides whether to trust.
    ValidWithWarning(RotatedKeyWarning),
}

impl ValidationOutcome {
    /// Whether the outcome carries a rotated-key warning.
    #[must_use]
    pub fn has_rotated_key_warning(&self) -> bool {
        matches!(self, ValidationOutcome::ValidWithWarning(_))
    }
}

/// Warning surfaced when manifest peer-DID is found in RotationLog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotatedKeyWarning {
    /// The peer-DID that was found rotated in the RotationLog.
    pub rotated_peer_did: Did,
}

/// Trait-shim for cross-crate content-addressing without forcing a
/// blake3/serde_ipld_dagcbor dep downstream.
pub trait ContentAddressed {
    /// Compute the canonical-bytes-content-CID of this value.
    fn content_cid(&self) -> Cid;
}

impl ContentAddressed for PluginManifest {
    fn content_cid(&self) -> Cid {
        self.compute_content_cid()
    }
}

// =====================================================================
// CapRequirement
// =====================================================================

/// Capability requirement entry — typed scope the plugin needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapRequirement {
    /// Scope string (e.g. `store:notes:read`, `private:<did>:*`).
    pub scope: String,
}

impl CapRequirement {
    /// New requirement with the given scope.
    #[must_use]
    pub fn new(scope: impl Into<String>) -> Self {
        Self {
            scope: scope.into(),
        }
    }

    /// Whether this scope is a private-namespace shape.
    #[must_use]
    pub fn is_private_namespace(&self) -> bool {
        self.scope.starts_with("private:")
    }
}

// =====================================================================
// SharesPolicy
// =====================================================================

/// Delegation policy envelope — the `shares` half of the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharesPolicy {
    /// Default disposition when no rule matches.
    pub default: SharesPolicyDefault,
    /// Per-cap or per-target rules.
    pub rules: Option<Vec<SharesRule>>,
}

impl SharesPolicy {
    /// Conservative v0 default — no delegation permitted.
    #[must_use]
    pub fn none() -> Self {
        Self {
            default: SharesPolicyDefault::None,
            rules: None,
        }
    }

    /// Whether delegation of `cap_pattern` to `target_plugin_did` is
    /// permitted under this policy.
    ///
    /// Private-namespace caps (`private:*`) MUST be checked at the
    /// caller (`plugin_delegation.rs`) BEFORE invoking this — they
    /// are unconditionally denied.
    #[must_use]
    pub fn permits_delegation(&self, cap_pattern: &str, target_plugin_did: &Did) -> bool {
        if let Some(rules) = &self.rules {
            for rule in rules {
                if rule.matches(cap_pattern, target_plugin_did) {
                    return true;
                }
            }
        }
        matches!(self.default, SharesPolicyDefault::Any)
    }
}

/// Default share-disposition when no rule matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SharesPolicyDefault {
    /// Conservative — no delegation permitted (v0 default).
    None,
    /// Any delegation permitted (rare; trust-anchor plugins).
    Any,
    /// Only matching `rules` permit; non-matching denies.
    Matching,
}

/// Per-cap / per-target delegation rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharesRule {
    /// Cap pattern (exact or single trailing `:*` glob).
    pub cap_pattern: String,
    /// Target of the rule.
    pub target: SharesTarget,
}

impl SharesRule {
    /// Whether this rule matches the request.
    #[must_use]
    pub fn matches(&self, cap: &str, target: &Did) -> bool {
        self.cap_pattern_matches(cap) && self.target.matches(target)
    }

    fn cap_pattern_matches(&self, cap: &str) -> bool {
        if self.cap_pattern == cap {
            return true;
        }
        if let Some(prefix) = self.cap_pattern.strip_suffix(":*") {
            return cap.starts_with(prefix)
                && cap.len() > prefix.len()
                && cap.as_bytes()[prefix.len()] == b':';
        }
        false
    }
}

/// Target of a `SharesRule`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SharesTarget {
    /// Any plugin permitted (within `cap_pattern`).
    Any,
    /// A specific plugin-DID permitted.
    PluginDid(Did),
    /// Any plugin authored by a specific peer-DID permitted.
    PluginAuthor(Did),
}

impl SharesTarget {
    fn matches(&self, target: &Did) -> bool {
        match self {
            SharesTarget::Any => true,
            SharesTarget::PluginDid(d) => d == target,
            // PluginAuthor matching at this layer is conservative —
            // chain validator at manifest_envelope_chain_validation
            // resolves the target's manifest peer-DID.
            SharesTarget::PluginAuthor(_) => false,
        }
    }
}

// =====================================================================
// RendererConfig
// =====================================================================

/// Renderer configuration (optional; per `docs/PLUGIN-MANIFEST.md` §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RendererConfig {
    /// Output format: `"html_json"` / `"plaintext"` / others.
    pub output_format: String,
    /// Optional list of renderer backends supported.
    pub renderer_backends: Option<Vec<RendererBackend>>,
    /// Hosting target: `"browser_wasm32"` / `"tauri_embedded_webview"`.
    pub hosting_target: Option<String>,
    /// Bundle size budget in KiB (thin-compute-surface enforcement).
    pub bundle_size_budget_kb: Option<u32>,
}

/// Renderer backend handle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RendererBackend {
    /// Browser wasm32-unknown-unknown bundle (CLAUDE.md #17 shape b).
    BrowserWasm32,
    /// Tauri 2.x embedded webview (CLAUDE.md #17 shape c).
    TauriEmbeddedWebview,
    /// Other backend name (forward-compat).
    Other(String),
}

// =====================================================================
// InstallRecord
// =====================================================================

/// Install-time consent record. User-DID signs over `(manifest_cid ||
/// timestamp_nanos || nonce || plugin_did_bytes)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallRecord {
    /// CID of the manifest this install record consents to.
    pub manifest_cid: Cid,
    /// Plugin-DID minted at install (UCAN audience handle per
    /// CLAUDE.md #18 retense — NOT attested sub-identity).
    pub plugin_did: Did,
    /// User-DID that consented (root of cap chain per Layer 1).
    pub consenting_user_did: Did,
    /// Ed25519 detached signature by `consenting_user_did`.
    pub user_signature: Vec<u8>,
    /// Wall-clock timestamp at install (nanoseconds since UNIX epoch).
    /// Engine injects per Ben Q6 ratification.
    pub timestamp_stub_nanos: u64,
    /// Replay-defense nonce (16 bytes from OsRng).
    pub nonce: Vec<u8>,
    /// UCAN delegations from user-DID to plugin-DID for granted caps.
    pub granted_caps_bytes: Vec<Vec<u8>>,
}

impl InstallRecord {
    /// Bytes signed by the user-DID.
    #[must_use]
    pub fn signing_payload(&self) -> Vec<u8> {
        let plugin_did_bytes = self.plugin_did.as_str().as_bytes();
        let mut out = Vec::with_capacity(36 + 8 + self.nonce.len() + plugin_did_bytes.len());
        out.extend_from_slice(self.manifest_cid.as_bytes());
        out.extend_from_slice(&self.timestamp_stub_nanos.to_le_bytes());
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(plugin_did_bytes);
        out
    }

    /// Verify the user-DID signature on the install record.
    ///
    /// # Errors
    ///
    /// `E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID`.
    pub fn verify_user_signature(&self) -> Result<(), ErrorCode> {
        if self.user_signature.len() != 64 {
            return Err(ErrorCode::PluginInstallRecordUserSignatureInvalid);
        }
        let pubkey = self
            .consenting_user_did
            .resolve()
            .map_err(|_| ErrorCode::PluginInstallRecordUserSignatureInvalid)?;
        let sig_bytes: [u8; 64] = self
            .user_signature
            .as_slice()
            .try_into()
            .map_err(|_| ErrorCode::PluginInstallRecordUserSignatureInvalid)?;
        let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        pubkey
            .as_verifying_key()
            .verify(&self.signing_payload(), &signature)
            .map_err(|_| ErrorCode::PluginInstallRecordUserSignatureInvalid)
    }
}

// =====================================================================
// Composition cycle detection (post-R1-triage Q2 ratification)
// =====================================================================

/// Detect a cycle in the meta-plugin composition graph rooted at
/// `root_cid` / `root_manifest`, resolving each `composes_plugins`
/// reference via `resolver`.
///
/// # Errors
///
/// `E_PLUGIN_META_COMPOSITION_CYCLE_REJECTED` on cycle detection.
pub fn detect_composition_cycle<F>(
    root_cid: Cid,
    root_manifest: &PluginManifest,
    resolver: &F,
) -> Result<(), ErrorCode>
where
    F: Fn(&Cid) -> Option<PluginManifest>,
{
    let mut stack: Vec<(Cid, PluginManifest)> = vec![(root_cid, root_manifest.clone())];
    let mut visited: std::collections::HashSet<Cid> = std::collections::HashSet::new();
    visited.insert(root_cid);

    while let Some((_, m)) = stack.pop() {
        if let Some(refs) = &m.composes_plugins {
            for child_cid in refs {
                if *child_cid == root_cid {
                    return Err(ErrorCode::PluginMetaCompositionCycleRejected);
                }
                if visited.contains(child_cid) {
                    // Already-visited child — diamond shape; not a cycle.
                    continue;
                }
                if let Some(child_manifest) = resolver(child_cid) {
                    visited.insert(*child_cid);
                    stack.push((*child_cid, child_manifest));
                }
            }
        }
    }
    Ok(())
}

// =====================================================================
// Signing helper
// =====================================================================

/// Sign a manifest body with the given keypair. The `content_cid`
/// MUST be pre-populated via `compute_content_cid()`.
#[must_use]
pub fn sign_manifest(manifest: &PluginManifest, keypair: &benten_id::keypair::Keypair) -> Vec<u8> {
    let payload = manifest.signing_payload();
    keypair.sign(&payload).to_bytes().to_vec()
}
