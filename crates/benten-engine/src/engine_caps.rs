//! Capability + IVM-view mutation surface for [`crate::engine::Engine`].
//!
//! Split from `engine.rs` for file-size hygiene. Houses the privileged
//! system-zone writes: `create_principal`, `grant_capability`,
//! `revoke_capability`, `create_view`, plus the private
//! `privileged_put_node` helper that routes writes through the engine's
//! privileged `WriteContext`. Every method is a plain `impl Engine` item.

use std::collections::BTreeMap;

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
    /// engine-privileged path AND — when IVM is enabled — registers a live
    /// view instance with the subscriber so future change events flow into
    /// it (code-reviewer g7-cr-8).
    ///
    /// Idempotent: same `view_id` returns the same content-addressed CID.
    /// The content-listing view family (view_id `"content_listing"` or
    /// `"content_listing_<label>"`) is instantiated with the trailing label
    /// as its input pattern; other canonical ids register their own view.
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
        };
        let node = def.as_node();
        let cid = self.privileged_put_node(&node)?;

        // Register the live view with the IVM subscriber so change events
        // propagate. Skipped when IVM is disabled. We dedupe by view id —
        // re-registering the same id is a no-op at the subscriber level.
        if let Some(ivm) = self.ivm.as_ref() {
            let already_registered = ivm.view_ids().iter().any(|id| id == view_id);
            if !already_registered {
                if let Some(label) = input_pattern_label.as_deref() {
                    let view = benten_ivm::views::ContentListingView::new(label);
                    ivm.register_view(Box::new(view));
                }
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
}
