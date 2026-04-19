//! [`GrantBackedPolicy`] — a Phase-1 capability policy that reads
//! `system:CapabilityGrant` / `system:CapabilityRevocation` Nodes through a
//! [`GrantReader`] handle and denies writes whose scope has no unrevoked
//! grant.
//!
//! # Phase-1 scope + compromises
//!
//! - **Scope derivation is label-based.** A write whose primary label is
//!   `"post"` derives required scope `"store:post:write"`. The grants issued
//!   by `Engine::grant_capability` / `revoke_capability` use exactly that
//!   scope string, so the two sides line up without threading a `requires`
//!   property through the evaluator (per-primitive `requires` enforcement is
//!   Phase-2 scope per R1 triage SC4).
//! - **Actor is not yet checked.** Phase-1 `Engine::call` does not yet thread
//!   an actor through `WriteContext`; the policy treats any unrevoked grant
//!   for the derived scope as sufficient. Phase-3 `benten-id` swaps in a
//!   typed principal and the policy tightens to actor-scoped lookups.
//! - **Revocation is observed via presence of a `system:CapabilityRevocation`
//!   node** with the same `scope` property as the grant. The two Nodes are
//!   written through the engine-privileged path so forgeries would require
//!   the system-zone bypass — which user subgraphs cannot reach.
//! - **The `check_read` leg honours named compromise #2 (Option A):**
//!   returns `CapError::DeniedRead` for targeted reads without a matching
//!   read grant. Phase-3 revisits once the identity surface lands.
//!
//! See `docs/ENGINE-SPEC.md` §9 and `docs/SECURITY-POSTURE.md` for the
//! capability posture; see R1 triage §SC4 for the declared-AND-checked
//! contract this policy implements for writes.

use std::sync::Arc;

use crate::error::CapError;
use crate::policy::{CapabilityPolicy, PendingOp, ReadContext, WriteContext};

/// Read-only handle into the backing store used by [`GrantBackedPolicy`] to
/// resolve grants at commit time.
///
/// Kept behind a trait so the policy does not take a direct dep on
/// `benten-graph` (which would be a layering break — `benten-caps` is a
/// lower-level crate than the orchestrator that composes the two). The
/// engine implements this trait against its own [`benten_graph::RedbBackend`]
/// and injects the handle at assembly time.
pub trait GrantReader: Send + Sync {
    /// Does the backend contain at least one unrevoked
    /// `system:CapabilityGrant` Node whose `scope` property equals `scope`?
    ///
    /// # Errors
    ///
    /// Returns [`CapError::Denied`] when the backend read fails; the
    /// `required` payload names the scope and the `entity` payload is empty.
    /// Wrapping a backend failure as a denial is the correct Phase-1 posture
    /// — a policy that cannot read its grant list cannot safely permit the
    /// write.
    fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, CapError>;
}

/// Capability policy that keys off `system:CapabilityGrant` /
/// `system:CapabilityRevocation` Nodes stored through the engine-privileged
/// path.
///
/// Construction is via [`GrantBackedPolicy::new`] — the caller supplies the
/// `Arc<dyn GrantReader>` that the policy consults at commit time. In the
/// standard Engine path the builder bootstraps the Arc after the backend is
/// opened (see `benten_engine::EngineBuilder::capability_policy_grant_backed`).
pub struct GrantBackedPolicy {
    grants: Arc<dyn GrantReader>,
}

impl GrantBackedPolicy {
    /// Construct a policy backed by the supplied `GrantReader`.
    #[must_use]
    pub fn new(grants: Arc<dyn GrantReader>) -> Self {
        Self { grants }
    }

    /// Derive the Phase-1 required scope from a label. The mapping is the
    /// canonical `"store:<label>:write"` form used by the Phase-1 zero-config
    /// `crud('<label>')` path. An empty label collapses to `"store:write"`.
    fn derive_write_scope(label: &str) -> String {
        if label.is_empty() {
            "store:write".to_string()
        } else {
            format!("store:{label}:write")
        }
    }

    /// Derive the Phase-1 required read scope from a label. Mirrors
    /// [`Self::derive_write_scope`] but with the `:read` suffix.
    fn derive_read_scope(label: &str) -> String {
        if label.is_empty() {
            "store:read".to_string()
        } else {
            format!("store:{label}:read")
        }
    }
}

impl std::fmt::Debug for GrantBackedPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrantBackedPolicy").finish_non_exhaustive()
    }
}

impl CapabilityPolicy for GrantBackedPolicy {
    fn check_write(&self, ctx: &WriteContext) -> Result<(), CapError> {
        // Engine-privileged writes bypass the policy entirely (system-zone
        // API). The privileged flag is set only by the Engine's internal
        // grant / revoke / create_view path.
        if ctx.is_privileged {
            return Ok(());
        }

        // Derive scopes from the batch: for each PutNode we scope by its
        // primary (first) label; if no pending ops (pre-G3 synthetic ctx
        // shape) we fall back to the convenience `label` field.
        let scopes: Vec<String> = if ctx.pending_ops.is_empty() {
            if ctx.label.is_empty() && ctx.scope.is_empty() {
                return Ok(());
            }
            let scope = if ctx.scope.is_empty() {
                Self::derive_write_scope(&ctx.label)
            } else {
                ctx.scope.clone()
            };
            vec![scope]
        } else {
            let mut v = Vec::new();
            for op in &ctx.pending_ops {
                match op {
                    PendingOp::PutNode { labels, .. } => {
                        // Skip system-zone writes — they reach the policy only
                        // if something went wrong higher up; denying them here
                        // would double-fire the guard. Be conservative.
                        let primary = labels.first().cloned().unwrap_or_default();
                        if primary.starts_with("system:") {
                            continue;
                        }
                        v.push(Self::derive_write_scope(&primary));
                    }
                    PendingOp::PutEdge { label, .. } => {
                        v.push(Self::derive_write_scope(label));
                    }
                    PendingOp::DeleteNode { .. } | PendingOp::DeleteEdge { .. } => {
                        // Deletes in Phase-1 use the same write-scope family
                        // as creates of the same label. Without the Node body
                        // we cannot recover the label from a CID alone; skip
                        // the check for now (Phase-2 widens the PendingOp shape
                        // to include the target label).
                    }
                }
            }
            v
        };

        for scope in &scopes {
            let ok = self.grants.has_unrevoked_grant_for_scope(scope)?;
            if !ok {
                return Err(CapError::Denied {
                    required: scope.clone(),
                    entity: ctx.label.clone(),
                });
            }
        }
        Ok(())
    }

    fn check_read(&self, ctx: &ReadContext) -> Result<(), CapError> {
        // Phase-1 read-side gating: targeted reads against a named label
        // require a `store:<label>:read` grant. An empty label (bare query
        // or introspection read) permits by default — the compromise #2
        // surface is about targeted reads.
        if ctx.label.is_empty() {
            return Ok(());
        }
        // System-zone reads are always permitted; the engine is the only
        // caller that reaches this label space.
        if ctx.label.starts_with("system:") {
            return Ok(());
        }
        let scope = Self::derive_read_scope(&ctx.label);
        let ok = self.grants.has_unrevoked_grant_for_scope(&scope)?;
        if ok {
            Ok(())
        } else {
            Err(CapError::DeniedRead)
        }
    }
}
