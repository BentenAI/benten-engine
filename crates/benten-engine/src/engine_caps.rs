//! Capability + IVM-view mutation surface for [`crate::engine::Engine`].
//!
//! Split from `engine.rs` for file-size hygiene. Houses the privileged
//! system-zone writes: `create_principal`, `grant_capability`,
//! `revoke_capability`, `create_view`, plus the private
//! `privileged_put_node` helper that routes writes through the engine's
//! privileged `WriteContext`. Every method is a plain `impl Engine` item.

use std::collections::BTreeMap;
use std::sync::Arc;

use benten_core::{Cid, Node, Value};

use crate::engine::Engine;
use crate::error::EngineError;
use crate::outcome::ViewCreateOptions;
use crate::subgraph_spec::{GrantSubject, RevokeScope, RevokeSubject};

impl Engine {
    // -------- System-zone privileged API (N7) --------

    /// Create an actor principal. Phase 1: the principal is stored as a
    /// `system:Principal`-labeled Node; its CID is used as the actor identity
    /// by `grant_capability` / `revoke_capability`.
    pub fn create_principal(&self, name: &str) -> Result<Cid, EngineError> {
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("name".into(), Value::Text(name.into()));
        let node = Node::new(vec!["system:Principal".into()], props);
        self.privileged_put_node(&node)
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
        if !self.caps_enabled {
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
        self.privileged_put_node(&node)
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
        if !self.caps_enabled {
            return Err(EngineError::SubsystemDisabled {
                subsystem: "capabilities",
            });
        }
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("actor".into(), actor.as_value());
        props.insert("scope".into(), Value::Text(scope.as_scope_string()));
        let node = Node::new(vec!["system:CapabilityRevocation".into()], props);
        self.privileged_put_node(&node)?;
        Ok(())
    }

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
            // views default to Strategy::B per `Engine::create_user_view`.
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
    fn privileged_put_node(&self, node: &Node) -> Result<Cid, EngineError> {
        Ok(self.backend.put_node_with_context(
            node,
            &benten_graph::WriteContext::privileged_for_engine_api(),
        )?)
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
    pub fn install_ucan_proof(
        &self,
        ucan: &benten_id::ucan::Ucan,
    ) -> Result<Cid, EngineError> {
        if !self.caps_enabled {
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
        let backend = benten_caps::UCANBackend::new(Arc::clone(&self.backend));
        backend
            .install_proof(ucan)
            .map_err(|e| EngineError::Other {
                code: e.code(),
                message: format!("install_ucan_proof: {e}"),
            })
    }
}
