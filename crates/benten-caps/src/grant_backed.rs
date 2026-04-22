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
/// engine implements this trait against its own `benten_graph::RedbBackend`
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

    /// Phase 2a ucca-6: batched variant — return `true` iff any scope in
    /// `scopes` has an unrevoked grant. The default impl fans out to N
    /// single-scope calls; concrete implementations override to perform a
    /// single backend read to bound resume-time CPU cost under adversarial
    /// deep chains.
    ///
    /// TODO(phase-2a-G9-A): concrete single-read override.
    fn has_unrevoked_grant_for_any(&self, scopes: &[&str]) -> Result<bool, CapError> {
        for s in scopes {
            if self.has_unrevoked_grant_for_scope(s)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// Phase 2a ucca-6: `GrantReader` configuration overrides (plan §3 G9-A).
/// Currently carries the `max_chain_depth` bound; Phase 2b may grow more
/// knobs (e.g. batch size).
#[derive(Debug, Clone)]
pub struct GrantReaderConfig {
    /// Maximum attenuation-chain depth before firing
    /// [`CapError::ChainTooDeep`]. Default 64.
    pub max_chain_depth: usize,
}

impl Default for GrantReaderConfig {
    fn default() -> Self {
        Self {
            max_chain_depth: 64,
        }
    }
}

/// Phase 2a ucca-6 test harness: a concrete `GrantReader`-alike handle
/// exposing a synthetic attenuation chain. Lives alongside the trait because
/// the `grant_reader_max_chain_depth` test references this type (see
/// consolidation report for the shape compromise).
///
/// TODO(phase-2a-G9-A): replace this synthetic harness with the real grant
/// reader's chain-depth checker backed by `system:CapabilityGrant` /
/// `system:CapabilityRevocation` lookups.
pub struct GrantReaderChain {
    chain: Vec<crate::grant::CapabilityGrant>,
    config: GrantReaderConfig,
}

impl GrantReaderChain {
    /// Construct a chain-backed test harness with the default
    /// [`GrantReaderConfig`] (max depth 64).
    #[must_use]
    pub fn with_chain_for_test(chain: Vec<crate::grant::CapabilityGrant>) -> Self {
        Self {
            chain,
            config: GrantReaderConfig::default(),
        }
    }

    /// Override the config (test only).
    #[must_use]
    pub fn with_config(mut self, config: GrantReaderConfig) -> Self {
        self.config = config;
        self
    }

    /// Combined constructor: chain + config in one call.
    #[must_use]
    pub fn with_chain_and_config_for_test(
        chain: Vec<crate::grant::CapabilityGrant>,
        config: GrantReaderConfig,
    ) -> Self {
        Self { chain, config }
    }

    /// Walk the chain and return [`CapError::ChainTooDeep`] when the depth
    /// exceeds the configured bound.
    ///
    /// # Errors
    /// Fires [`CapError::ChainTooDeep`] on depth > `max_chain_depth`.
    pub fn check_attenuation_for_test(&self, _scope: &str) -> Result<(), CapError> {
        if self.chain.len() > self.config.max_chain_depth {
            return Err(CapError::ChainTooDeep {
                depth: self.chain.len(),
                limit: self.config.max_chain_depth,
            });
        }
        Ok(())
    }
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

        // Derive scopes from the batch: for each op we scope by its primary
        // label; if no pending ops (pre-G3 synthetic ctx shape) we fall back
        // to the convenience `label` / `scope` fields. r6-sec-8: when the
        // caller arrives with neither pending ops nor any scope hint, the
        // grant-backed policy DENIES rather than permit-by-default — an
        // unstructured WriteContext reaching the policy is an error mode,
        // not a legitimate no-op, and denying closes the fail-open surface.
        let scopes: Vec<String> = if ctx.pending_ops.is_empty() {
            if ctx.label.is_empty() && ctx.scope.is_empty() {
                return Err(CapError::Denied {
                    required: "store:write".to_string(),
                    entity: String::new(),
                });
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
                    PendingOp::DeleteNode { labels, .. } => {
                        // r6-sec-8: labels captured via read-before-delete in
                        // benten-graph are threaded through the caps PendingOp
                        // so the policy can derive the same store:<label>:write
                        // scope used for the create side. Empty labels means
                        // an idempotent-miss delete — no scope needed.
                        let primary = labels.first().cloned().unwrap_or_default();
                        if primary.is_empty() || primary.starts_with("system:") {
                            continue;
                        }
                        v.push(Self::derive_write_scope(&primary));
                    }
                    PendingOp::DeleteEdge { label, .. } => {
                        let Some(label) = label else {
                            continue; // idempotent miss
                        };
                        if label.starts_with("system:") {
                            continue;
                        }
                        v.push(Self::derive_write_scope(label));
                    }
                }
            }
            v
        };

        for scope in &scopes {
            // r6b-dx-C1: wildcard-aware match. A stored grant whose scope is
            // an ancestor wildcard of `scope` (e.g. stored `store:post:*`
            // satisfies required `store:post:write`) must permit. We
            // enumerate every wildcard variant of the required concrete scope
            // and check whether the reader has any of them. This preserves
            // the opaque `has_unrevoked_grant_for_scope(exact)` reader
            // contract while letting the policy share the wildcard semantics
            // used by `attenuation::check_attenuation`.
            let mut granted = false;
            for candidate in wildcard_variants(scope) {
                if self.grants.has_unrevoked_grant_for_scope(&candidate)? {
                    granted = true;
                    break;
                }
            }
            if !granted {
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
        // r6b-dx-C1: same wildcard enumeration used by `check_write` — a
        // grant of `store:post:*` satisfies a required `store:post:read`.
        let mut granted = false;
        for candidate in wildcard_variants(&scope) {
            if self.grants.has_unrevoked_grant_for_scope(&candidate)? {
                granted = true;
                break;
            }
        }
        if granted {
            Ok(())
        } else {
            Err(CapError::DeniedRead {
                required: scope,
                entity: ctx.label.clone(),
            })
        }
    }
}

/// Enumerate every parent-scope spelling that would attenuate to the
/// concrete required scope, including the scope itself.
///
/// For an N-segment required scope, returns up to 2^N candidates: each
/// segment position may either stay concrete or be replaced with `"*"`.
/// Ordered for readability (exact match first, bare `"*"` last). Empty
/// input yields one empty candidate so trivial inputs still round-trip.
///
/// This mirrors the semantics in [`crate::attenuation::check_attenuation`]
/// without requiring the opaque [`GrantReader`] to carry any wildcard
/// awareness — callers that already store a typed `GrantScope` can use
/// `check_attenuation` directly; the `GrantBackedPolicy` path goes
/// through this enumerator because the backend reader only answers
/// exact-match queries.
fn wildcard_variants(required: &str) -> Vec<String> {
    if required.is_empty() {
        return vec![String::new()];
    }
    let segments: Vec<&str> = required.split(':').collect();
    let n = segments.len();
    // Small-N path: N is typically 2-4 for Phase 1 scopes
    // (`store:<label>:write` is 3; future namespaced scopes might reach 5).
    // 2^5 = 32 candidates — cheap to enumerate. For unexpectedly large N we
    // bail out to just the exact match + bare `"*"` to avoid quadratic
    // reader traffic.
    if n > 6 {
        return vec![required.to_string(), "*".to_string()];
    }
    let total = 1_usize << n;
    let mut out: Vec<String> = Vec::with_capacity(total);
    // i == 0 → all segments concrete (the required scope itself).
    // i == (1<<n)-1 → every segment is `"*"`.
    // Bit k set means segment k is replaced by `"*"`.
    for i in 0..total {
        let mut parts: Vec<&str> = Vec::with_capacity(n);
        for (k, seg) in segments.iter().enumerate() {
            if (i >> k) & 1 == 1 {
                parts.push("*");
            } else {
                parts.push(seg);
            }
        }
        // Collapse trailing `"*"` runs into a single trailing `"*"`:
        // `store:*:*` is semantically equivalent to `store:*` under the
        // trailing-wildcard rule, so we deduplicate to reduce reader
        // traffic. Non-trailing wildcard spellings (`store:*:write`)
        // are preserved — they still match via the mid-scope wildcard
        // rule in `check_attenuation`, and some callers may have
        // stored a grant in exactly that form.
        let mut end = parts.len();
        while end > 1 && parts[end - 1] == "*" && parts[end - 2] == "*" {
            end -= 1;
        }
        parts.truncate(end);
        out.push(parts.join(":"));
    }
    // Dedupe while preserving first-seen order so the exact scope is tried
    // first (hot path) and the catch-all `"*"` last.
    let mut seen = std::collections::BTreeSet::new();
    out.retain(|s| seen.insert(s.clone()));
    out
}

#[cfg(test)]
mod wildcard_tests {
    use super::wildcard_variants;

    #[test]
    fn three_segment_required_enumerates_expected_wildcards() {
        let got = wildcard_variants("store:post:write");
        // Must contain the exact match, the conventional Phase-1 wildcards,
        // and the bare catch-all. Order is not asserted — only membership.
        assert!(got.contains(&"store:post:write".to_string()));
        assert!(got.contains(&"store:post:*".to_string()));
        assert!(got.contains(&"store:*".to_string()));
        assert!(got.contains(&"*".to_string()));
    }

    #[test]
    fn exact_match_is_always_first_candidate() {
        let got = wildcard_variants("store:post:write");
        assert_eq!(got[0], "store:post:write");
    }

    #[test]
    fn empty_input_yields_single_empty_candidate() {
        assert_eq!(wildcard_variants(""), vec![String::new()]);
    }
}
