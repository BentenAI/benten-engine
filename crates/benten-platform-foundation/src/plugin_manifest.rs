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

use benten_core::{Cid, OperationNode, PrimitiveKind, Subgraph, Value};
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
            // sec-r6r1-7 hardening (R6-FP-A): `SharesPolicyDefault::Any` is
            // the footgun arm — a manifest can fire `shares: { default:
            // Any, rules: None }` and silently delegate ANY required cap
            // to ANY plugin. The threat-model assumes the conservative v0
            // default is `default: None`; the validator now requires
            // `Any` to be paired with an explicit anti-rule or scoping
            // `rules` vector so the consent-UX surface has SOMETHING to
            // present + the cap envelope isn't structurally fail-open.
            SharesPolicyDefault::Any => {
                if self.shares.rules.as_ref().is_none_or(|r| r.is_empty()) {
                    return Err(ErrorCode::PluginManifestInvalid);
                }
            }
            SharesPolicyDefault::None => {}
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

    /// **Phase-4-Foundation R4b-FP-1 Seam 2** — engine-injection-time
    /// validation seam. Plugin authors call [`Self::validate`] WITHOUT
    /// a clock parameter; the engine surface (here +
    /// [`crate::plugin_lifecycle::install_plugin`]) wraps with clock
    /// injection at the load boundary per D-4F-15 transparent-clock-
    /// injection ratification.
    ///
    /// Fail-CLOSED contract (sec-3.5-r1-7 carry): the
    /// `MANIFEST_CLOCK_NOT_INJECTED_SENTINEL = 0` sentinel encodes
    /// "engine built WITHOUT clock injection". If the caller passes
    /// the sentinel AND the manifest declares ANY time-bounded
    /// behavior (today the `host:time:now` cap requirement is the
    /// observable proxy; future UCAN-typed `nbf`/`exp` proofs in the
    /// install record extend this), the seam fail-closes with
    /// [`ErrorCode::UcanClockNotInjected`]. With a non-sentinel clock,
    /// validation proceeds normally + the `now_secs` is threaded
    /// through to downstream UCAN chain-walkers in
    /// [`crate::plugin_lifecycle::install_plugin`].
    ///
    /// **Cross-language rule-mirror discipline (§3.5g)**: the
    /// `E_UCAN_CLOCK_NOT_INJECTED` typed code is the SAME variant
    /// Phase-3 PR #158 minted for `UcanGroundedPolicy`; reusing
    /// preserves a single mental model for "no wallclock injected at
    /// the engine surface".
    ///
    /// # Errors
    ///
    /// - [`ErrorCode::UcanClockNotInjected`] when `now_secs == 0` AND
    ///   the manifest declares time-bounded requirements.
    /// - Propagates [`Self::validate`] + [`Self::verify_peer_signature`]
    ///   structural/cryptographic failures.
    pub fn validate_with_clock(&self, now_secs: u64) -> Result<ValidationOutcome, ErrorCode> {
        self.validate()?;
        self.verify_peer_signature()?;
        if now_secs == MANIFEST_CLOCK_NOT_INJECTED_SENTINEL && self.declares_time_bounded() {
            return Err(ErrorCode::UcanClockNotInjected);
        }
        Ok(ValidationOutcome::Valid)
    }

    /// Whether this manifest declares any time-bounded behavior whose
    /// validation requires a real engine-injected wallclock.
    ///
    /// At G24-D the observable proxy is presence of `host:time:now` in
    /// `requires` (the standard wallclock-cap requirement). Phase-4-Meta
    /// extension: UCAN-typed `nbf`/`exp` proofs carried in the install
    /// record's `granted_caps_bytes` also count as time-bounded.
    #[must_use]
    pub fn declares_time_bounded(&self) -> bool {
        self.requires
            .iter()
            .any(|r| r.scope == "host:time:now" || r.scope.starts_with("host:time:"))
    }
}

/// Sentinel value indicating "engine built without clock injection".
/// Mirrors the `DEFAULT_NOW_SECS = 0` sentinel in
/// `benten-caps::ucan_grounded` for a single mental model across the
/// platform.
pub const MANIFEST_CLOCK_NOT_INJECTED_SENTINEL: u64 = 0;

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
///
/// R6-FP-A sec-r6r1-8 closure (2026-05-13): the prior
/// `SharesTarget::PluginAuthor(Did)` variant was DEAD CODE — `matches()`
/// returned `false` unconditionally with a comment claiming "chain
/// validator resolves the target's manifest peer-DID", but the chain
/// validator at `manifest_envelope_chain_validation` consults the same
/// `SharesPolicy` so the variant never resolved; it silently denied
/// while masquerading as a feature. Removed entirely to prevent future
/// implementer surprise. If author-DID-based targeting is wanted in
/// Phase-4-Meta, mint a NEW typed `SharesTarget::PluginAuthor` variant
/// THEN with the resolver-lookup wiring AND a typed ErrorCode for the
/// resolver-miss case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SharesTarget {
    /// Any plugin permitted (within `cap_pattern`).
    Any,
    /// A specific plugin-DID permitted.
    PluginDid(Did),
}

impl SharesTarget {
    fn matches(&self, target: &Did) -> bool {
        match self {
            SharesTarget::Any => true,
            SharesTarget::PluginDid(d) => d == target,
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

/// Canonical handler-id for a composition-cycle-detection subgraph.
/// Foundation-owned namespace (parallel to `schema:` from
/// [`crate::schema_compiler`] + `plugin-library` from
/// [`crate::plugin_library`]).
pub const HANDLER_ID_COMPOSITION_WALK: &str = "plugin-composition-walk";

/// Edge label used inside the composition-walk subgraph: source
/// (compositing) manifest → target (sub-) manifest. The walker treats
/// this label as the "composes" relation.
pub const EDGE_COMPOSES: &str = "COMPOSES";

/// Property key on a composition-walk node: the manifest CID (bytes).
pub const PROP_NODE_MANIFEST_CID: &str = "manifest_cid";

/// Build a [`Subgraph`] representation of the meta-plugin composition
/// graph rooted at `root_cid`. Each visited manifest is one
/// [`PrimitiveKind::Read`] Node carrying its CID as a property, and
/// each `composes_plugins` reference is one `COMPOSES`-labelled edge.
///
/// The walker BFS-traverses outward from the root via the supplied
/// `resolver`. Unresolvable children (resolver returns `None`) are
/// recorded as terminal nodes — the subgraph still contains them as
/// Read Nodes with no outgoing edges.
///
/// Unlike [`detect_composition_cycle`] this builder does NOT short-
/// circuit on cycle detection — the cycle itself is encoded as an
/// edge back to a previously-visited node. [`detect_composition_cycle`]
/// then walks the built subgraph to surface the typed error.
#[must_use]
pub fn build_composition_subgraph<F>(
    root_cid: Cid,
    root_manifest: &PluginManifest,
    resolver: &F,
) -> Subgraph
where
    F: Fn(&Cid) -> Option<PluginManifest>,
{
    let mut sg = Subgraph::new(HANDLER_ID_COMPOSITION_WALK);
    let mut queue: Vec<(Cid, PluginManifest)> = vec![(root_cid, root_manifest.clone())];
    let mut visited: std::collections::HashSet<Cid> = std::collections::HashSet::new();
    // Always emit the root node FIRST so the resulting subgraph's
    // first node is the canonical root for downstream consumers.
    sg.nodes.push(
        OperationNode::new(composition_node_id(&root_cid), PrimitiveKind::Read).with_property(
            PROP_NODE_MANIFEST_CID,
            Value::Bytes(root_cid.as_bytes().to_vec()),
        ),
    );
    visited.insert(root_cid);

    while let Some((parent_cid, parent_manifest)) = queue.pop() {
        let Some(refs) = &parent_manifest.composes_plugins else {
            continue;
        };
        for child_cid in refs {
            // Always emit the COMPOSES edge — cycle edges (when
            // *child_cid == root_cid or matches any visited node) ARE
            // valid edges in the structural subgraph; the downstream
            // cycle-walker reads them to detect the cycle.
            sg.edges.push((
                composition_node_id(&parent_cid),
                composition_node_id(child_cid),
                EDGE_COMPOSES.to_string(),
            ));
            // Emit child Node ONCE per CID.
            if !visited.contains(child_cid) {
                visited.insert(*child_cid);
                sg.nodes.push(
                    OperationNode::new(composition_node_id(child_cid), PrimitiveKind::Read)
                        .with_property(
                            PROP_NODE_MANIFEST_CID,
                            Value::Bytes(child_cid.as_bytes().to_vec()),
                        ),
                );
                // Recurse only into resolvable children.
                if let Some(child_manifest) = resolver(child_cid) {
                    queue.push((*child_cid, child_manifest));
                }
            }
        }
    }

    sg
}

/// Canonical Node id for a composition-walk Node representing
/// `manifest_cid`.
#[must_use]
pub fn composition_node_id(manifest_cid: &Cid) -> String {
    let mut hex = String::with_capacity(2 + 64);
    hex.push_str("manifest::");
    for b in manifest_cid.as_bytes() {
        use core::fmt::Write;
        let _ = write!(&mut hex, "{b:02x}");
    }
    hex
}

/// Detect a cycle in the meta-plugin composition graph rooted at
/// `root_cid` / `root_manifest`, resolving each `composes_plugins`
/// reference via `resolver`.
///
/// **R6-FP-D lift (cag-ux-r6-r1-4 closure):** the detector is now a
/// two-step pipeline:
///
/// 1. Build a real [`Subgraph`] representation of the composition
///    graph via [`build_composition_subgraph`]. The subgraph has one
///    [`PrimitiveKind::Read`] Node per visited manifest CID + one
///    [`EDGE_COMPOSES`]-labelled edge per composition reference. No
///    new [`PrimitiveKind`] variant is introduced (CLAUDE.md baked-in
///    #1 12-primitive irreducibility).
///
/// 2. Walk the subgraph via [`detect_cycle_in_subgraph`] (DFS
///    visited-on-entry / cleared-on-exit). A cycle = any back-edge
///    encountered during the walk.
///
/// This shape is the "subgraph-aware fallback" path the wave brief
/// names: composition-cycle detection consumes a [`Subgraph`]
/// representation of the composition graph, but uses its own
/// structural walker internally rather than threading through the
/// full engine evaluator (which would require library-as-subgraph +
/// evaluator handoff and is named for Phase-4-Meta).
///
/// Observable behavior is preserved: cycle present →
/// [`ErrorCode::PluginMetaCompositionCycleRejected`]. The
/// `meta_plugin_composition_cycle_rejected_with_typed_error_code` +
/// `meta_plugin_acyclic_composition_admitted_no_typed_error` test
/// pins (in
/// `crates/benten-platform-foundation/tests/plugin_meta_composition_cycle_rejected.rs`)
/// stay green.
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
    let sg = build_composition_subgraph(root_cid, root_manifest, resolver);
    detect_cycle_in_subgraph(&sg, &composition_node_id(&root_cid))
}

/// Walk a composition-shaped Subgraph rooted at `root_node_id` and
/// surface [`ErrorCode::PluginMetaCompositionCycleRejected`] on any
/// back-edge. The walker uses DFS with on-stack tracking — a back-
/// edge to any node currently in the DFS stack is a cycle.
///
/// This function is the engine-evaluator-style structural walk over
/// the composition subgraph: it consults `sg.edges()` and walks
/// outgoing edges by label ([`EDGE_COMPOSES`]) just as the engine
/// evaluator walks operation-node edges. The walker is internal to
/// [`detect_composition_cycle`]; exposed `pub` for substantive test
/// pins that exercise the shape directly.
///
/// # Errors
///
/// [`ErrorCode::PluginMetaCompositionCycleRejected`] when any
/// reachable back-edge is encountered.
pub fn detect_cycle_in_subgraph(sg: &Subgraph, root_node_id: &str) -> Result<(), ErrorCode> {
    use std::collections::HashSet;

    // visited = fully-explored set (post-DFS); on_stack = currently in
    // the DFS recursion stack. A back-edge into on_stack is a cycle.
    let mut visited: HashSet<String> = HashSet::new();
    let mut on_stack: HashSet<String> = HashSet::new();
    // Frame: (node_id, child_iter_index). Hand-rolled iterative DFS
    // avoids unbounded recursion on adversarial input.
    let mut stack: Vec<(String, usize)> = vec![(root_node_id.to_string(), 0)];
    on_stack.insert(root_node_id.to_string());

    while let Some((node_id, ref_idx)) = stack.last().cloned() {
        // Find the (ref_idx)-th outgoing COMPOSES edge from node_id.
        let next_child = sg
            .edges()
            .iter()
            .filter(|(from, _, label)| from == &node_id && label == EDGE_COMPOSES)
            .nth(ref_idx)
            .map(|(_, to, _)| to.clone());
        match next_child {
            Some(child_id) => {
                // Advance the parent's index for the next iteration.
                if let Some(top) = stack.last_mut() {
                    top.1 += 1;
                }
                if on_stack.contains(&child_id) {
                    // Back-edge into DFS-stack = cycle.
                    return Err(ErrorCode::PluginMetaCompositionCycleRejected);
                }
                if !visited.contains(&child_id) {
                    on_stack.insert(child_id.clone());
                    stack.push((child_id, 0));
                }
                // Already-visited child not on stack = diamond
                // (cross-edge); not a cycle; continue.
            }
            None => {
                // Exhausted children — finalize this node.
                on_stack.remove(&node_id);
                visited.insert(node_id);
                stack.pop();
            }
        }
    }

    Ok(())
}

// =====================================================================
// Schema-author trust-list enforcement (R6 R1 sdr-r6-r1-2 closure)
// =====================================================================

/// Enforce a manifest's `requires_schema_authors` trust-list against a
/// schema's peer-DID author.
///
/// Per Ben Q3 ratification (Phase-4-Foundation R1 triage): default
/// EMPTY trust-list (None or empty Vec) returns Ok — the user has not
/// constrained authorship. When the manifest's `requires_schema_authors`
/// is `Some(list)` AND non-empty AND `schema_author_did` is NOT in the
/// list, returns `E_PLUGIN_AUTHOR_NOT_TRUSTED` (re-uses the existing
/// plugin-author trust-list ErrorCode — schema authorship is the same
/// trust class).
///
/// **Where this should be called:** at schema-compile entry or at
/// admin UI v0 install-time (when the install path knows the schema's
/// author DID from manifest signature provenance). The v1 admin UI
/// (Phase-4-Foundation) ships with default-trust-empty so this helper
/// is a no-op on all v1 install paths; the user-prompt UX surface that
/// surfaces `ProvenanceOutcome::UserPromptRequired` for untrusted
/// authors is named in `docs/future/phase-4-backlog.md` §4.19
/// (Phase-4-Meta carry).
///
/// # Errors
///
/// `E_PLUGIN_AUTHOR_NOT_TRUSTED` when `schema_author_did` is outside
/// a non-empty `requires_schema_authors`.
pub fn validate_schema_author_within_manifest_envelope(
    schema_author_did: &Did,
    manifest: &PluginManifest,
) -> Result<(), ErrorCode> {
    match &manifest.requires_schema_authors {
        // None OR empty Vec — default EMPTY (Ben Q3 default-empty).
        None => Ok(()),
        Some(list) if list.is_empty() => Ok(()),
        Some(list) => {
            if list.contains(schema_author_did) {
                Ok(())
            } else {
                Err(ErrorCode::PluginAuthorNotTrusted)
            }
        }
    }
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
