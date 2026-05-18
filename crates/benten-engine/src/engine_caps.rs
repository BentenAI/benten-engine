//! Capability + IVM-view mutation surface for [`crate::engine::Engine`].
//!
//! Split from `engine.rs` for file-size hygiene. The capability-grant
//! mutation surface (`create_principal`, `grant_capability`,
//! `grant_capability_with_proof`, `revoke_capability`,
//! `revoke_capability_by_grant_cid`, `install_ucan_proof`,
//! `delegate_capability`, plus the `CapProof`-based `install_proof` /
//! `revoke`) lives **exclusively** on [`EngineCapsHandle`] — the sole
//! canonical capability-mutation surface, returned by
//! [`crate::engine::Engine::caps`]. The IVM-view registration surface
//! (`create_view`) + the `pub(crate)` `privileged_put_node` helper
//! (the shared system-zone write substrate) remain plain `impl Engine`
//! items.

use std::collections::BTreeMap;
use std::sync::Arc;

use benten_core::{Cid, Node, Value};

use crate::engine::Engine;
use crate::error::EngineError;
use crate::outcome::ViewCreateOptions;
use crate::subgraph_spec::{GrantSubject, RevokeScope, RevokeSubject};

/// Phase-3 G16-B-F (sec-r4r1-2 BLOCKER closure) — capability-grant
/// mutation handle returned by [`Engine::caps`]. Wraps a borrow of the
/// engine + exposes thin `install_proof` / `revoke` surfaces that route
/// through the engine's existing privileged
/// [`EngineCapsHandle::grant_capability`] / [`EngineCapsHandle::revoke_capability`] paths.
///
/// The handle is the production-equivalent surface the sec-r4r1-2
/// RED-PHASE pins consume. Distinct from the test-only
/// `testing_revoke_actor_for_subscribe` helper because it ALSO updates
/// the in-memory `(actor, zone)` revocation pair set that
/// [`Engine::apply_atrium_merge`]'s per-row cap-recheck consults — a
/// caller that revokes via this handle observes the rejection at the
/// next sync-replica merge boundary.
pub struct EngineCapsHandle<'eng> {
    /// Engine borrow. Crate-private so external code MUST go through
    /// the public methods.
    pub(crate) engine: &'eng Engine,
}

/// Phase-3 G16-B-F — opaque capability proof shape consumed by
/// [`EngineCapsHandle::install_proof`].
///
/// Carries the `(actor_cid, scope)` pair the grant authorizes. The
/// scope string mirrors the scope handed to
/// [`EngineCapsHandle::grant_capability`] (e.g. `"/zone/posts:write"` or the
/// Phase-1 `"store:<label>:write"` form). The `proof_cid` slot is the
/// CID of the durable grant Node minted by `grant_capability` when
/// `install_proof` runs — callers retain it to address `revoke`
/// surgically.
///
/// Phase-3 wave-5a-style placeholder until G14-B's full UCAN-chain
/// proof shape lands; the public surface name + arity stay stable per
/// the no-refactor-on-G14-B-landing contract.
#[derive(Debug, Clone)]
pub struct CapProof {
    /// Actor CID the grant authorizes.
    pub actor_cid: Cid,
    /// Scope string (e.g. `"/zone/posts:write"`).
    pub scope: String,
    /// CID of the durable grant Node (populated by
    /// [`EngineCapsHandle::install_proof`]; `None` pre-install).
    pub proof_cid: Option<Cid>,
}

impl CapProof {
    /// Construct a fresh `CapProof` for `(actor_cid, scope)`.
    #[must_use]
    pub fn new(actor_cid: Cid, scope: impl Into<String>) -> Self {
        Self {
            actor_cid,
            scope: scope.into(),
            proof_cid: None,
        }
    }
}

impl<'eng> EngineCapsHandle<'eng> {
    /// Install a capability proof — routes through
    /// [`EngineCapsHandle::grant_capability`] internally to mint a
    /// `system:CapabilityGrant` Node.
    ///
    /// On success, populates `proof.proof_cid` with the minted grant's
    /// CID so callers can `revoke` surgically.
    ///
    /// # Errors
    ///
    /// Forwards [`EngineError`] from [`EngineCapsHandle::grant_capability`] —
    /// most commonly [`EngineError::SubsystemDisabled`] when the
    /// engine was built with `.without_caps()`.
    pub fn install_proof(&self, proof: &mut CapProof) -> Result<Cid, EngineError> {
        let cid = self.grant_capability(&proof.actor_cid, &proof.scope)?;
        proof.proof_cid = Some(cid);
        // Recover from any prior revocation: an install observed AFTER
        // a revoke for the same `(actor, scope)` pair lifts the in-memory
        // revocation. Symmetric with the durable
        // `system:CapabilityGrant` Node + `system:CapabilityRevocation`
        // Node ordering — the latest write wins.
        self.engine
            .inner
            .revoked_actor_zone_pairs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&(proof.actor_cid, proof.scope.clone()));
        Ok(cid)
    }

    /// Revoke a previously-installed proof. Updates BOTH the durable
    /// `system:CapabilityRevocation` zone (via
    /// [`EngineCapsHandle::revoke_capability`]) AND the in-memory
    /// `(actor_cid, scope)` revocation pair set consulted by
    /// [`Engine::apply_atrium_merge`]'s per-row cap-recheck.
    ///
    /// # Errors
    ///
    /// Forwards [`EngineError`] from [`EngineCapsHandle::revoke_capability`].
    pub fn revoke(&self, proof: &CapProof) -> Result<(), EngineError> {
        self.revoke_capability(&proof.actor_cid, proof.scope.as_str())?;
        // Mark the in-memory mirror so the next sync-replica merge
        // boundary observes the revocation synchronously per the
        // sec-r4r1-2 BLOCKER closure pattern.
        self.engine
            .inner
            .mark_actor_revoked_for_zone(&proof.actor_cid, proof.scope.clone());
        Ok(())
    }

    // -------- System-zone privileged API (N7) --------

    /// Create an actor principal. Phase 1: the principal is stored as a
    /// `system:Principal`-labeled Node; its CID is used as the actor identity
    /// by `grant_capability` / `revoke_capability`.
    pub fn create_principal(&self, name: &str) -> Result<Cid, EngineError> {
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("name".into(), Value::Text(name.into()));
        let node = Node::new(vec!["system:Principal".into()], props);
        self.engine.privileged_put_node(&node)
    }

    /// Grant a capability. Writes a `system:CapabilityGrant` Node via the
    /// engine-privileged path. The first arg may be a `&Cid`, `&str`, or
    /// owning `Cid`/`String` per the `GrantSubject` impls.
    pub fn grant_capability<A, S>(&self, actor: A, scope: S) -> Result<Cid, EngineError>
    where
        A: GrantSubject,
        S: AsRef<str>,
    {
        self.grant_capability_with_proof(actor, scope, None, None)
    }

    /// Phase-3 G21-T2 — grant a capability carrying optional UCAN
    /// proof-chain attribution (issuer DID + HLC stamp). Closes
    /// audit-6-1 + phase-3-backlog §2.3 (b): the napi parser now
    /// threads `issuer` + `hlc` through to the durable grant Node so
    /// the durable backend's chain-walker can correlate the grant
    /// with its UCAN-chain origin.
    ///
    /// `issuer` is the DID string of the UCAN-chain root (or any
    /// agent that minted the grant); `hlc` is the HLC stamp at issue
    /// time used for replay-window narrowing during chain validation.
    /// Both fields are optional — when `None`, the persisted Node
    /// shape matches the pre-G21-T2 grant and the durable backend
    /// treats the grant as Phase-1-style (actor-bound, no UCAN
    /// chain). When `Some`, the durable backend's chain-walker
    /// consults these fields at write-check time.
    pub fn grant_capability_with_proof<A, S>(
        &self,
        actor: A,
        scope: S,
        issuer: Option<String>,
        hlc: Option<i64>,
    ) -> Result<Cid, EngineError>
    where
        A: GrantSubject,
        S: AsRef<str>,
    {
        if !self.engine.caps_enabled {
            return Err(EngineError::SubsystemDisabled {
                subsystem: "capabilities",
            });
        }
        let scope_str = scope.as_ref().to_string();
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("actor".into(), actor.as_value());
        props.insert("scope".into(), Value::Text(scope_str));
        props.insert("revoked".into(), Value::Bool(false));
        if let Some(iss) = issuer {
            props.insert("issuer".into(), Value::Text(iss));
        }
        if let Some(stamp) = hlc {
            props.insert("hlc".into(), Value::Int(stamp));
        }
        let node = Node::new(vec!["system:CapabilityGrant".into()], props);
        self.engine.privileged_put_node(&node)
    }

    /// Revoke a capability. Phase 1: writes a `system:CapabilityRevocation`
    /// Node naming the `(actor, scope)` pair. The revocation is distinct from
    /// the grant's own `revoked` property so a sync replica that has only
    /// seen the revocation node can still recognize the grant as revoked.
    pub fn revoke_capability<A, S>(&self, actor: A, scope: S) -> Result<(), EngineError>
    where
        A: RevokeSubject,
        S: RevokeScope,
    {
        if !self.engine.caps_enabled {
            return Err(EngineError::SubsystemDisabled {
                subsystem: "capabilities",
            });
        }
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("actor".into(), actor.as_value());
        props.insert("scope".into(), Value::Text(scope.as_scope_string()));
        let node = Node::new(vec!["system:CapabilityRevocation".into()], props);
        self.engine.privileged_put_node(&node)?;
        Ok(())
    }

    /// Phase-3.5 §13.11 closure: revoke a previously-granted capability
    /// identified by its grant CID. Resolves the grant Node by CID,
    /// extracts its `scope` property, then writes the matching
    /// `system:CapabilityRevocation` Node via [`EngineCapsHandle::revoke_capability`].
    ///
    /// This is the canonical seam for callers that hold a grant CID
    /// (e.g. the napi binding's `revokeCapability(grantCid, actor)`
    /// surface). The pre-3.5 napi path passed `grant_cid` AS the scope
    /// string to the engine's `revoke_capability`, producing a
    /// `system:CapabilityRevocation` Node with `scope = "<cid>"` —
    /// which the `crate::builder::BackendGrantReader::revoked_scopes`
    /// walker never matched against the actual write scope
    /// (`store:post:write` et al.), silently fail-OPENing every
    /// post-revoke write. Routing through this method preserves the
    /// scope-keyed revocation contract that
    /// `benten_caps::grant_backed::GrantBackedPolicy::check_write`
    /// consumes.
    ///
    /// # Errors
    ///
    /// - [`EngineError::SubsystemDisabled`] when caps are disabled.
    /// - [`EngineError::Other`] with `benten_errors::ErrorCode::NotFound` when the
    ///   grant CID does not resolve to a stored Node, when the Node is
    ///   not a `system:CapabilityGrant`, or when its `scope` property
    ///   is missing / wrong-typed.
    pub fn revoke_capability_by_grant_cid<A>(
        &self,
        grant_cid: &Cid,
        actor: A,
    ) -> Result<(), EngineError>
    where
        A: RevokeSubject,
    {
        if !self.engine.caps_enabled {
            return Err(EngineError::SubsystemDisabled {
                subsystem: "capabilities",
            });
        }
        // Engine-privileged backend read — `Engine::get_node` would
        // collapse system-zone Nodes to `Ok(None)` per Inv-11 runtime
        // probe. We reach through `self.engine.backend.get_node` directly
        // (same pattern as the system-zone privileged write path).
        let Some(node) = self
            .engine
            .backend
            .get_node(grant_cid)
            .map_err(EngineError::Graph)?
        else {
            return Err(EngineError::Other {
                code: benten_errors::ErrorCode::NotFound,
                message: format!(
                    "revoke_capability_by_grant_cid: grant CID {} not found in backend",
                    grant_cid.to_base32()
                ),
            });
        };
        if !node.labels.iter().any(|l| l == "system:CapabilityGrant") {
            return Err(EngineError::Other {
                code: benten_errors::ErrorCode::NotFound,
                message: format!(
                    "revoke_capability_by_grant_cid: CID {} is not a system:CapabilityGrant Node \
                     (got labels: {:?})",
                    grant_cid.to_base32(),
                    node.labels
                ),
            });
        }
        let scope = match node.properties.get("scope") {
            Some(Value::Text(s)) => s.clone(),
            _ => {
                return Err(EngineError::Other {
                    code: benten_errors::ErrorCode::NotFound,
                    message: format!(
                        "revoke_capability_by_grant_cid: grant Node {} missing or wrong-typed \
                         `scope` property",
                        grant_cid.to_base32()
                    ),
                });
            }
        };
        self.revoke_capability(actor, scope.as_str())
    }
    /// Phase-3 G21-T2 fp-mini-review BLOCKER-3 closure — install a
    /// signed UCAN proof into the durable
    /// [`benten_caps::UCANBackend`] proof-store (`g14b:grant:<cid>`
    /// KV namespace) so [`benten_caps::UcanGroundedPolicy`] can
    /// consult it at write-check time.
    ///
    /// Pre-fp-mini-review there was NO call site for
    /// [`benten_caps::UCANBackend::install_proof`] in the engine; the
    /// chain-walker was reachable only from tests. Wiring the adapter
    /// here lets `PolicyKind::Ucan` actually exercise the durable
    /// chain-walker for `cap:typed:*` capabilities (BLOCKER-2 partial
    /// closure scope, see [`crate::EngineBuilder::capability_policy_ucan_durable`]
    /// for the full composition narrative).
    ///
    /// The persisted UCAN survives engine restarts via the underlying
    /// KV store. Subsequent calls with the same proof are idempotent
    /// (the KV layer overwrites with byte-identical body — same CID).
    ///
    /// # Errors
    ///
    /// Returns [`EngineError`] when the capability subsystem is
    /// disabled or the durable store rejects the write.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn install_ucan_proof(&self, ucan: &benten_id::ucan::Ucan) -> Result<Cid, EngineError> {
        if !self.engine.caps_enabled {
            return Err(EngineError::SubsystemDisabled {
                subsystem: "capabilities",
            });
        }
        // Compose a fresh UCANBackend over the engine's own backend
        // — the underlying KV store is shared so a proof installed
        // here is the same proof
        // `UcanGroundedPolicy::typed_cap_permitted_by_proof` reads at
        // write-check time. Constructing a fresh wrapper per call is
        // cheap (the wrapper holds an Arc ref + the rate-limit plug).
        let backend = benten_caps::UCANBackend::new(Arc::clone(&self.engine.backend));
        backend.install_proof(ucan).map_err(|e| EngineError::Other {
            code: e.code(),
            message: format!("install_ucan_proof: {e}"),
        })
    }

    /// Phase-4-Foundation G24-D-FP-3 — runtime UCAN delegation surface
    /// (audience = plugin-DID, scope = resolved-from-source-grant).
    ///
    /// Consumed by the napi `delegateCapability(grantCid, pluginDid,
    /// attenuatedCaps)` binding. Mirrors the resolve-by-CID discipline
    /// of [`EngineCapsHandle::revoke_capability_by_grant_cid`]: the napi caller
    /// passes the SOURCE grant's CID; this seam resolves the grant
    /// Node + extracts its actual `scope` text + then writes the
    /// delegation grant carrying that **resolved** scope. The CID is
    /// NEVER persisted as the new grant's scope — that would mirror the
    /// pre-PR-#199 napi revoke class-of-bug shape on the delegate
    /// surface (a `system:CapabilityGrant` Node with
    /// `scope = "<source_cid base32>"` that the `GrantReader` walker
    /// never matches against the actual write scope at policy-check
    /// time, silently fail-OPENing every cross-plugin write).
    ///
    /// # Algorithm
    ///
    /// 1. Resolve `source_grant_cid` → load the Node → verify it is a
    ///    `system:CapabilityGrant` → extract its `scope` text. This is
    ///    the **resolved scope**.
    /// 2. Run the single-step manifest-envelope check via
    ///    [`benten_caps::plugin_delegation::check_delegation_within_envelope`]
    ///    against the resolved scope + audience plugin-DID.
    ///
    ///    **Wave-E HELD #1197/#1146 (refinement-audit-2026-05):** the
    ///    pre-Wave-E `AllPermit` policy-view shim is DELETED. The check
    ///    now consults the installed
    ///    [`crate::shares_policy_resolver::SharesPolicyResolver`] port,
    ///    keyed on the resolved source grant's `actor` (the issuing
    ///    principal). The port's concrete impl (in
    ///    `benten-platform-foundation`) resolves the source plugin's
    ///    signed, user-consented manifest `shares` policy. **Fail-
    ///    CLOSED:** a plugin-principal whose manifest cannot be
    ///    resolved, or whose `shares` policy denies the step, is
    ///    REJECTED with
    ///    [`benten_errors::ErrorCode::PluginDelegationOutsideManifestEnvelope`].
    ///    A user-root principal (Layer 1 anchor) is classified
    ///    `NotPluginPrincipal` and proceeds (user-issued caps are
    ///    bounded by attenuation only, per CLAUDE.md #18). Engines with
    ///    no resolver installed fall back to the default no-op
    ///    (observably identical to the old `AllPermit` shim — zero v1
    ///    behavior delta until the platform layer installs the
    ///    adapter).
    ///
    ///    The private-namespace forbidden clause STILL fires here
    ///    (BEFORE the resolver) — that's the class-of-bug defense for
    ///    `private:<plugin_did>:*` source grants per CLAUDE.md baked-in
    ///    #18.
    /// 3. Determine the effective scope for the new delegation grant:
    ///    - If `attenuated_caps` is empty → use the resolved source
    ///      scope unchanged (identity delegation).
    ///    - Otherwise → use `attenuated_caps[0]` as the new scope.
    ///      Each attenuated cap is also stored as a JSON-encoded text
    ///      property on the delegation Node for audit purposes.
    /// 4. Write a new `system:CapabilityGrant` Node via the privileged
    ///    path with `actor = plugin_did` + `scope = <resolved or
    ///    first-attenuated>` + `derived_from = source_grant_cid` text.
    ///    Return its CID.
    ///
    /// # Errors
    ///
    /// - [`EngineError::SubsystemDisabled`] when caps are disabled.
    /// - [`EngineError::Other`] with `benten_errors::ErrorCode::NotFound`
    ///   when the grant CID does not resolve to a stored Node, when
    ///   the Node is not a `system:CapabilityGrant`, or when its
    ///   `scope` property is missing / wrong-typed.
    /// - [`EngineError::Other`] with
    ///   `benten_errors::ErrorCode::PluginPrivateNamespaceDelegationForbidden`
    ///   when the source grant's scope is a `private:*` namespace cap.
    /// - [`EngineError::Other`] with
    ///   `benten_errors::ErrorCode::PluginDelegationOutsideManifestEnvelope`
    ///   when the envelope check denies the delegation.
    #[cfg(not(target_arch = "wasm32"))]
    // Wave-E HELD #1197/#1146: the Layer-3 manifest-`shares` resolver
    // wire-up (private-namespace clause + resolver consultation +
    // fail-CLOSED error mapping) is a single linear security-critical
    // decision sequence — splitting it across helpers would scatter the
    // fail-CLOSED audit trail. The function stays well under the real
    // cognitive-complexity concern; the lint trips on the doc-heavy
    // line count, not branching depth. Same disposition as the
    // Phase-3 C1-fp3 `too_many_lines` precedent.
    #[allow(clippy::too_many_lines)]
    pub fn delegate_capability(
        &self,
        source_grant_cid: &Cid,
        plugin_did: &str,
        attenuated_caps: &[String],
    ) -> Result<Cid, EngineError> {
        use benten_caps::plugin_delegation::{
            DelegationDecision, SharesPolicyView, check_delegation_within_envelope,
        };
        use benten_id::did::Did;

        if !self.engine.caps_enabled {
            return Err(EngineError::SubsystemDisabled {
                subsystem: "capabilities",
            });
        }

        // Step 1 — resolve source grant Node, extract scope (mirrors
        // `revoke_capability_by_grant_cid` class-of-bug defense).
        let Some(node) = self
            .engine
            .backend
            .get_node(source_grant_cid)
            .map_err(EngineError::Graph)?
        else {
            return Err(EngineError::Other {
                code: benten_errors::ErrorCode::NotFound,
                message: format!(
                    "delegate_capability: source grant CID {} not found in backend",
                    source_grant_cid.to_base32()
                ),
            });
        };
        if !node.labels.iter().any(|l| l == "system:CapabilityGrant") {
            return Err(EngineError::Other {
                code: benten_errors::ErrorCode::NotFound,
                message: format!(
                    "delegate_capability: CID {} is not a system:CapabilityGrant Node \
                     (got labels: {:?})",
                    source_grant_cid.to_base32(),
                    node.labels
                ),
            });
        }
        let resolved_scope = match node.properties.get("scope") {
            Some(Value::Text(s)) => s.clone(),
            _ => {
                return Err(EngineError::Other {
                    code: benten_errors::ErrorCode::NotFound,
                    message: format!(
                        "delegate_capability: source grant Node {} missing or wrong-typed \
                         `scope` property",
                        source_grant_cid.to_base32()
                    ),
                });
            }
        };
        // The source grant's `actor` is the issuing principal — the
        // user-root for a directly-user-issued grant, or a plugin-DID
        // for a delegated (`derived_from`-bearing) grant. This is the
        // key the Layer-3 resolver consults to find the source plugin's
        // manifest `shares` policy. A grant with no `actor` text is
        // treated as un-attributable → fail-CLOSED at the resolver
        // (`NoManifest` if the resolver is real; the pre-Wave-E no-op
        // path treats the empty principal as `NotPluginPrincipal`).
        let source_principal_did = match node.properties.get("actor") {
            Some(Value::Text(s)) => s.clone(),
            _ => String::new(),
        };

        // Step 2a — always-on private-namespace clause. Fires BEFORE
        // the manifest-`shares` resolver: `private:<plugin_did>:*` caps
        // NEVER cross plugin boundaries regardless of any `shares`
        // policy (CLAUDE.md baked-in #18 private-namespace clause).
        // `check_delegation_within_envelope` with the unconditional
        // `DenyAllSharesPolicyView` isolates exactly the private-
        // namespace decision; the manifest-`shares` admit/deny call is
        // the Wave-E resolver path below (NOT this always-true/false
        // shim — the shim ONLY exercises the private-namespace branch,
        // which is policy-independent).
        struct DenyAllSharesPolicyView;
        impl SharesPolicyView for DenyAllSharesPolicyView {
            fn permits(&self, _cap: &str, _target: &Did) -> bool {
                false
            }
        }
        let audience = Did::from_string_unchecked(plugin_did.to_string());
        if check_delegation_within_envelope(
            resolved_scope.as_str(),
            &audience,
            &DenyAllSharesPolicyView,
        ) == DelegationDecision::PrivateNamespaceForbidden
        {
            return Err(EngineError::Other {
                code: benten_errors::ErrorCode::PluginPrivateNamespaceDelegationForbidden,
                message: format!(
                    "delegate_capability: private-namespace cap `{resolved_scope}` cannot \
                     cross plugin boundaries (CLAUDE.md #18 private-namespace clause)",
                ),
            });
        }

        // Step 2b — Wave-E HELD #1197/#1146 Layer-3 manifest-`shares`
        // envelope enforcement. Consult the installed
        // `SharesPolicyResolver` keyed on the source grant's `actor`.
        // Fail-CLOSED: `Denied` / `NoManifest` reject; only an explicit
        // `Admitted`, or a `NotPluginPrincipal` classification (the
        // source is the Layer-1 user-root), permits the delegation.
        // Engines with no resolver installed fall back to the default
        // `Some(Noop)` (observably identical to the deleted `AllPermit`
        // shim — zero v1 behavior delta until the platform layer
        // installs the real adapter).
        if let Some(resolver) = self.engine.shares_policy_resolver.as_ref() {
            let resolution = resolver.resolve_delegation(
                source_principal_did.as_str(),
                resolved_scope.as_str(),
                plugin_did,
            );
            if let Err(code) = resolution.clone().into_result() {
                return Err(EngineError::Other {
                    code,
                    message: format!(
                        "delegate_capability: delegation of `{resolved_scope}` to \
                         `{plugin_did}` denied by source principal \
                         `{source_principal_did}`'s manifest `shares` envelope \
                         (Layer-3 enforcement; resolution={resolution:?})",
                    ),
                });
            }
        }

        // Step 3 — pick effective scope for the new delegation grant.
        // Attenuation here is the simplest "narrowed-or-identical
        // scope" form per the G24-D-FP-3 brief; full attenuation
        // semantics (per-segment subset check) land alongside G27-D.
        let effective_scope = if attenuated_caps.is_empty() {
            resolved_scope.clone()
        } else {
            attenuated_caps[0].clone()
        };

        // Step 4 — write the new system:CapabilityGrant Node carrying
        // the resolved (not the CID) scope. The `derived_from` field
        // records the source grant CID for audit + future chain-walker
        // consumption.
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("actor".into(), Value::Text(plugin_did.to_string()));
        props.insert("scope".into(), Value::Text(effective_scope));
        props.insert("revoked".into(), Value::Bool(false));
        props.insert(
            "derived_from".into(),
            Value::Text(source_grant_cid.to_base32()),
        );
        if !attenuated_caps.is_empty() {
            // serde_json::to_string over Vec<String> is infallible in
            // practice (no NaN floats, no non-string keys); fall back
            // to a safe default if the encoder ever surprises us.
            let attenuation_json = serde_json::to_string(attenuated_caps).unwrap_or_default();
            props.insert("attenuation".into(), Value::Text(attenuation_json));
        }
        let new_grant = Node::new(vec!["system:CapabilityGrant".into()], props);
        self.engine.privileged_put_node(&new_grant)
    }
}

impl Engine {
    /// Create an IVM view registration. Writes a `system:IVMView` Node via the
    /// engine-privileged path AND — when IVM is enabled AND the view id
    /// names the content-listing view family — registers a live
    /// [`benten_ivm::views::ContentListingView`] instance with the subscriber
    /// so future change events flow into it (code-reviewer g7-cr-8).
    ///
    /// Idempotent: same `view_id` returns the same content-addressed CID.
    ///
    /// # Live-view registration scope
    ///
    /// - **Content-listing view family** (`view_id == "content_listing"` or
    ///   `view_id` matches `content_listing_<label>`): the view is
    ///   instantiated with the trailing label (or `"post"` for the bare
    ///   `"content_listing"` id) as its input pattern, AND a live view
    ///   instance is registered with the IVM subscriber. The definition
    ///   Node is also persisted.
    /// - **The 4 other canonical Phase-1 view ids** (`capability_grants`,
    ///   `event_dispatch`, `governance_inheritance`, `version_current`):
    ///   the definition Node is persisted via the privileged write path,
    ///   but **no live view instance is registered with the subscriber**.
    ///   A subsequent `read_view(<id>)` falls through to the canonical-id
    ///   whitelist and returns `IvmViewStale` (in strict) or empty
    ///   last-known-good (in allow-stale). This is because those views
    ///   require additional constructor parameters the Phase-1
    ///   `ViewCreateOptions` API doesn't yet surface.
    ///
    /// Lift to live-view registration for the 4 other canonical ids is
    /// `phase-3-backlog.md` §5.1 (R6FP-tail NEW-2 named destination —
    /// non-content-listing canonical view auto-registration). User-
    /// defined views go through [`Engine::register_user_view`] which
    /// IS wired through `AlgorithmBView::for_id` for the canonical ids
    /// (Strategy::B path) — the legacy `create_view` surface is the
    /// Strategy::A entry point for the 5 hand-written views. R6FP-tail
    /// NEW-2 corrects the prior docstring claim that "other canonical
    /// ids register their own view" (which read as "all 5 canonical
    /// ids get a live view instance" — only `content_listing` does).
    pub fn create_view(&self, view_id: &str, _opts: ViewCreateOptions) -> Result<Cid, EngineError> {
        // Derive the input pattern label for content-listing views so the
        // stored definition is stable regardless of subscriber state.
        let input_pattern_label = if let Some(label) = view_id.strip_prefix("content_listing_") {
            Some(label.to_string())
        } else if view_id == "content_listing" {
            Some("post".to_string())
        } else {
            None
        };
        let def = benten_ivm::ViewDefinition {
            view_id: view_id.to_string(),
            input_pattern_label: input_pattern_label.clone(),
            output_label: "system:IVMView".to_string(),
            // Phase 2b G8-A / D8-RESOLVED: hand-written canonical-id views
            // ALWAYS take Strategy::A (the 5-view fate hybrid keep-all-
            // parallel; Algorithm B is opt-in + ADDITIVE). User-registered
            // views default to Strategy::B per `Engine::register_user_view`.
            strategy: benten_ivm::Strategy::A,
        };
        let node = def.as_node();
        let cid = self.privileged_put_node(&node)?;

        // Register the live view with the IVM subscriber so change events
        // propagate. Skipped when IVM is disabled. We dedupe by view id —
        // re-registering the same id is a no-op at the subscriber level.
        if let Some(ivm) = self.ivm.as_ref() {
            let already_registered = ivm.view_ids().iter().any(|id| id == view_id);
            if !already_registered && let Some(label) = input_pattern_label.as_deref() {
                let view = benten_ivm::views::ContentListingView::new(label);
                // refinement-audit #628: NOT migrated to register_view_if_absent
                // here, for the same reason engine.rs:3097 was deliberately left
                // un-migrated — `ContentListingView::id()` returns a constant
                // ("content_listing"), so id-keyed atomic dedup would collapse
                // distinct per-label content-listing views into one (the
                // `already_registered` fast-path above keys on the per-label
                // semantic `view_id`, which is the real discriminator). The
                // unconditional append preserves correct per-label registration.
                ivm.register_view(Box::new(view));
                // Non-content-listing canonical view ids (capability_grants,
                // event_dispatch, governance_inheritance, version_current) are
                // Phase-2 scope for automatic instantiation — the definition
                // Node is still written, but the live view isn't constructed
                // here because those views have additional constructor
                // parameters the Phase-1 API doesn't yet surface.
            }
        }
        Ok(cid)
    }

    /// Internal: write a system-zone Node via the privileged context.
    pub(crate) fn privileged_put_node(&self, node: &Node) -> Result<Cid, EngineError> {
        Ok(self.backend.put_node_with_context(
            node,
            &benten_graph::WriteContext::privileged_for_engine_api(),
        )?)
    }
}
