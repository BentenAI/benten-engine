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
//!   an actor through `CapWriteContext`; the policy treats any unrevoked grant
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

use benten_core::Cid;

use crate::error::CapError;
use crate::policy::{CapWriteContext, CapabilityPolicy, PendingOp, ReadContext};

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
    /// TODO(phase-3 — has_unrevoked_grant_for_any single-read
    /// override): provide concrete single-read override. Carried from
    /// Phase-2a G9-A; pairs with §2.1 Durable UCAN backend.
    fn has_unrevoked_grant_for_any(&self, scopes: &[&str]) -> Result<bool, CapError> {
        for s in scopes {
            if self.has_unrevoked_grant_for_scope(s)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Phase 4-Foundation R1 cap-r1-2 / cap-r1-10: principal-aware grant
    /// lookup. Returns `true` iff the backend contains at least one
    /// unrevoked `system:CapabilityGrant` Node whose `scope` property
    /// equals `scope` AND whose `grantee` matches `actor_cid`.
    ///
    /// When `actor_cid` is `None` the call collapses to the scope-only
    /// [`Self::has_unrevoked_grant_for_scope`] behavior — preserving
    /// pre-Phase-4 semantics for callers that do not yet thread an actor
    /// (Phase-1 / Phase-2 fixtures + the
    /// [`crate::NoAuthBackend`] default-permit path).
    ///
    /// # Why a separate method instead of `Option<&Cid>` on the base call
    ///
    /// The default impl delegates to the existing scope-only method so
    /// every external `GrantReader` implementation (test fixtures, the
    /// `benten-engine` `BackendGrantReader`, the
    /// [`crate::ucan_grounded`] backend) continues to compile + behave
    /// exactly as before — the unbounded-permit semantics that Phase-1
    /// shipped are preserved by construction. Implementations that DO
    /// have actor-aware grant storage (the engine's
    /// `BackendGrantReader`) override this method to filter on the
    /// stored `grantee` property.
    ///
    /// # Errors
    ///
    /// Returns [`CapError::Denied`] when the backend read fails.
    fn has_unrevoked_grant_for_scope_and_actor(
        &self,
        scope: &str,
        actor_cid: Option<&Cid>,
    ) -> Result<bool, CapError> {
        // Default impl: ignore `actor_cid` and fall back to the scope-only
        // path. This preserves NoAuthBackend semantics AND any GrantReader
        // implementation that has not yet been taught to filter on
        // grantee. Backends that DO store grantee binding (notably the
        // engine's `BackendGrantReader`) override.
        let _ = actor_cid;
        self.has_unrevoked_grant_for_scope(scope)
    }

    /// refinement-audit-2026-05 #1141 (Pattern F: Qual-1 #694 + Safe-2
    /// #552 + Fwd-1 #928) — the **single wildcard-aware grant-match
    /// seam**.
    ///
    /// Answers: does the backend hold an unrevoked grant whose stored
    /// scope is `required_scope` itself OR any ancestor-wildcard
    /// spelling that `attenuation::check_attenuation` would admit
    /// (e.g. a stored `store:post:*` satisfies a required
    /// `store:post:write`), optionally filtered by `actor_cid`?
    ///
    /// # Why this method exists (the Pattern-F structural close)
    ///
    /// Pre-#1141, `GrantBackedPolicy::check_write` / `check_read`
    /// inlined a `for candidate in wildcard_variants(scope)` 2^N
    /// bit-iteration loop in the per-write hot path, calling the
    /// exact-match reader once per candidate. That coupled three
    /// independently-surfaced problems at the call site:
    ///
    /// - **Qual-1 #694:** the 2^N bit-iter + `BTreeSet` dedup +
    ///   trailing-`*` collapse is unaudit-friendly inlined into the
    ///   policy hot path; it now lives in exactly one named,
    ///   single-responsibility method (this one + its default body).
    /// - **Safe-2 #552:** the wildcard expansion had an asymmetric
    ///   `n > 6` silent-drop vs `check_attenuation`; the canonical
    ///   expansion ([`wildcard_variants`]) is now fixed in one place
    ///   so every consumer of this seam shares the corrected semantics.
    /// - **Fwd-1 #928:** the `O(C·O·G)` per-write cost cascade — a
    ///   backend with an index (the engine's `BackendGrantReader`)
    ///   can now **override** this method to resolve the wildcard
    ///   match with a single indexed lookup instead of `2^N`
    ///   exact-match reader calls. The default impl below preserves
    ///   the prior behavior exactly (no forced cross-crate cascade),
    ///   so existing implementations keep working unchanged while the
    ///   pushdown seam is available.
    ///
    /// # Errors
    ///
    /// Returns [`CapError::Denied`] when the backend read fails (the
    /// same fail-closed posture as the underlying exact-match
    /// methods).
    fn has_unrevoked_grant_matching(
        &self,
        required_scope: &str,
        actor_cid: Option<&Cid>,
    ) -> Result<bool, CapError> {
        // Default impl: enumerate the canonical wildcard parent
        // spellings (one place — the #552-fixed `wildcard_variants`)
        // and probe the exact-match actor-aware reader for each. This
        // is byte-for-byte the behavior `check_write` / `check_read`
        // had inlined pre-#1141; backends with an indexed grant store
        // override this for the Fwd-1 #928 single-lookup close.
        for candidate in wildcard_variants(required_scope) {
            if self.has_unrevoked_grant_for_scope_and_actor(&candidate, actor_cid)? {
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
/// TODO(phase-3 — GrantReaderChain synthetic harness replacement):
/// replace this synthetic harness with the real grant reader's
/// chain-depth checker backed by `system:CapabilityGrant` /
/// `system:CapabilityRevocation` lookups. Carried from Phase-2a G9-A;
/// pairs with §2.1 Durable UCAN backend.
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

    /// Derive the required write scope from a [`CapWriteContext`].
    ///
    /// # G27-B scope-derivation lift (Phase 4-Foundation)
    ///
    /// Resolution order (top wins):
    ///
    /// 1. **Explicit scope override.** If `ctx.scope` is non-empty, return it
    ///    verbatim. This is the lifted surface: callers (plugin manifest
    ///    grammar, non-CRUD primitive zones, sandbox/handler/view scopes,
    ///    `engine_wait.rs::resume` cap-recheck shape) populate `scope`
    ///    directly and the policy consults the explicit value without
    ///    routing through label derivation. Mirrors how
    ///    [`crate::ucan_grounded::UcanGroundedPolicy::check_write`] keys off
    ///    `ctx.scope` for typed-cap proof lookup
    ///    (`crates/benten-caps/src/ucan_grounded.rs::check_write` line 340).
    /// 2. **Label-derived fallback.** If `ctx.scope` is empty, derive
    ///    `"store:<label>:write"` from `ctx.label`. Empty label collapses to
    ///    `"store:write"`. Preserves the Phase-1 zero-config
    ///    `crud('<label>')` shape that every existing caller relies on
    ///    (see `tests/grant_backed_policy_existing_store_label_write_paths_unchanged.rs`).
    ///
    /// # Why the override is checked first
    ///
    /// Plugin manifest grammar (G24-D, CLAUDE.md #18) introduces non-CRUD
    /// scope shapes that the label-derived path cannot express:
    /// `private:<plugin_did>:*`, `requires:<plugin_did>:<path>`,
    /// `shares:<plugin_did>:<path>`. Honoring the explicit `ctx.scope`
    /// surface lets those shapes flow through `check_write` without
    /// expanding the label derivation grammar (which would couple the
    /// caps crate to plugin-manifest semantics).
    fn derive_write_scope_from_ctx(ctx: &CapWriteContext) -> String {
        if !ctx.scope.is_empty() {
            return ctx.scope.clone();
        }
        Self::derive_write_scope(&ctx.label)
    }

    /// Derive the Phase-1 required scope from a label. The mapping is the
    /// canonical `"store:<label>:write"` form used by the Phase-1 zero-config
    /// `crud('<label>')` path. An empty label collapses to `"store:write"`.
    ///
    /// Retained as the per-`PendingOp` derivation helper for batch shapes
    /// where each op contributes its own label-derived scope; the explicit
    /// [`CapWriteContext::scope`] override (G27-B lift) is handled by
    /// [`Self::derive_write_scope_from_ctx`].
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
    fn check_write(&self, ctx: &CapWriteContext) -> Result<(), CapError> {
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
        // unstructured CapWriteContext reaching the policy is an error mode,
        // not a legitimate no-op, and denying closes the fail-open surface.
        //
        // G27-B scope-derivation lift: when `ctx.scope` is explicitly
        // populated, the override SHORT-CIRCUITS the per-op derivation
        // and the policy consults the single explicit scope. This holds
        // regardless of whether `pending_ops` is empty, because the
        // explicit scope expresses the caller's intent at a higher
        // grain than per-op labels (mirrors `UcanGroundedPolicy`
        // keying off `ctx.scope` at `ucan_grounded.rs::check_write`
        // line 340 — the typed-cap proof-chain walker resolves
        // `ctx.scope` as the lookup key, not the per-op labels). See
        // `tests/grant_backed_policy_derives_scope_from_write_context.rs`
        // for the substantive arm + `tests/grant_backed_policy_non_crud_scope_round_trip.rs`
        // for the plugin-manifest grammar shapes that depend on this
        // override path. Per-op derivation remains the default fallback
        // for the Phase-1 `crud('<label>')` shape (every existing caller
        // leaves `ctx.scope` empty per `CapWriteContext::default()`); see
        // `tests/grant_backed_policy_existing_store_label_write_paths_unchanged.rs`
        // for the backward-compat regression guard.
        //
        // G27-B mini-review MINOR (per `.addl/phase-4-foundation/r5-g27-b-mini-review.json`
        // finding `r5g27b-mr-2`): the override path SHORT-CIRCUITS per-op
        // derivation entirely, including the `system:*` skip + read-before-
        // delete idempotent-miss semantics. Currently the SOLE call site that
        // sets `ctx.scope` with potentially-populated `pending_ops` is
        // `Engine::wait_resume_action` at `engine_wait.rs::884` (path:
        // `wait:resume` scope; `pending_ops` empty in practice). Future call
        // sites that combine an explicit `ctx.scope` with a mixed `pending_ops`
        // batch (e.g., regular op + `system:*` op) MUST audit whether the
        // higher-grain override is the intended semantic — the policy no
        // longer enforces caps per-op when both are set. See
        // `tests/grant_backed_policy_derives_scope_from_write_context.rs`
        // for the substantive arm of the explicit-`ctx.scope`-overrides-
        // label-derived-default path.
        let scopes: Vec<String> = if !ctx.scope.is_empty() {
            vec![ctx.scope.clone()]
        } else if ctx.pending_ops.is_empty() {
            if ctx.label.is_empty() {
                return Err(CapError::Denied {
                    required: "store:write".to_string(),
                    entity: String::new(),
                });
            }
            vec![Self::derive_write_scope_from_ctx(ctx)]
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
            // refinement-audit-2026-05 #1141 (Qual-1 #694 + Safe-2 #552
            // + Fwd-1 #928): wildcard-aware match through the SINGLE
            // `has_unrevoked_grant_matching` seam. A stored grant whose
            // scope is an ancestor wildcard of `scope` (e.g. stored
            // `store:post:*` satisfies required `store:post:write`)
            // permits. The 2^N enumeration + dedup + trailing-`*`
            // collapse logic that used to be inlined here now lives in
            // exactly one place (the seam's default body / a backend's
            // indexed override) so the policy hot path no longer
            // carries it. `check_write` is the original scope-only call
            // site (no actor binding — Phase-1 write-gate posture);
            // pass `None`.
            if !self.grants.has_unrevoked_grant_matching(scope, None)? {
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
        //
        // Phase 4-Foundation R1 cap-r1-2 BLOCKER + cap-r1-10 dual-gate:
        // consult `ctx.actor_cid` via the principal-aware reader method.
        // Pre-fix `check_read` wildcard-enumerated grants against `scope`
        // alone — user-A who lacked `store:post:read` would still see a
        // permit if ANY peer (user-B) held the same scope under the same
        // backend, because the scope-only reader has no actor binding.
        // The new method filters by `grantee == actor_cid`.
        //
        // When `ctx.actor_cid` is `None` the underlying reader method
        // collapses to the scope-only path (default-trait-impl
        // delegation), which preserves NoAuthBackend semantics and the
        // Phase-1 / Phase-2 fixtures that pre-date actor threading.
        // refinement-audit-2026-05 #1141: same single
        // `has_unrevoked_grant_matching` seam as `check_write`, here
        // WITH the read-side actor binding (cap-r1-2 / cap-r1-10
        // dual-gate) — `ctx.actor_cid` is threaded so user-A who lacks
        // `store:post:read` is not falsely permitted by user-B's
        // same-scope grant under a shared backend.
        if self
            .grants
            .has_unrevoked_grant_matching(&scope, ctx.actor_cid.as_ref())?
        {
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
    // 2^5 = 32 candidates — cheap to enumerate. For unexpectedly large N
    // the full 2^N cross-product is avoided (quadratic reader traffic);
    // instead we emit the linear set of trailing-wildcard parent
    // spellings.
    //
    // #552 (Safe-2 boundary) closure: the prior `n > 6` branch returned
    // ONLY `[required, "*"]`, silently dropping every intermediate
    // trailing-wildcard parent (`a:b:c:*`, `a:b:*`, …). That made a
    // legitimately-stored ancestor-wildcard grant un-matchable at
    // `check_write` even though `attenuation::check_attenuation` WOULD
    // admit it — an asymmetric semantic drift between the two wildcard
    // surfaces. The trailing-wildcard prefix set is O(N) (linear, not
    // 2^N), so it is always safe to enumerate it even for large N: the
    // dominant matching mode for deep scopes IS the trailing wildcard
    // (`private:<did>:*`), which is exactly what `check_attenuation`'s
    // trailing-`*` rule admits. Non-trailing interior-wildcard spellings
    // (`a:*:c`) are the combinatorial part that is bounded out for
    // large N — those are rare in practice and the 2^N path still
    // covers them for the common small-N case below.
    if n > 6 {
        let mut out: Vec<String> = Vec::with_capacity(n + 2);
        out.push(required.to_string());
        // Trailing-wildcard parents: `a:b:c:*`, `a:b:*`, …, `a:*`.
        for keep in (1..n).rev() {
            let mut parts: Vec<&str> = segments[..keep].to_vec();
            parts.push("*");
            out.push(parts.join(":"));
        }
        out.push("*".to_string());
        let mut seen = std::collections::BTreeSet::new();
        out.retain(|s| seen.insert(s.clone()));
        return out;
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

    // refinement-audit-2026-05 #552 (Safe-2 boundary) closure-pin: a
    // >6-segment required scope MUST still enumerate its trailing-
    // wildcard parent spellings, not silently collapse to
    // `[exact, "*"]`. Would-FAIL against the pre-#1141 `n > 6` branch
    // which dropped every intermediate `a:b:c:*` parent (asymmetric
    // drift vs `attenuation::check_attenuation`'s trailing-`*` rule).
    #[test]
    fn deep_scope_keeps_trailing_wildcard_parents_not_just_exact_and_star() {
        // 8 segments — well past the old `n > 6` silent-drop boundary.
        let deep = "private:did:key:z6MkAbc:resource:sub:detail:leaf";
        let got = wildcard_variants(deep);
        assert!(got.contains(&deep.to_string()), "exact must be present");
        assert!(got.contains(&"*".to_string()), "bare catch-all present");
        // The load-bearing #552 assertions: the intermediate
        // trailing-wildcard parents a real stored grant would use.
        assert!(
            got.contains(&"private:did:key:z6MkAbc:resource:sub:detail:*".to_string()),
            "deepest trailing-* parent MUST be enumerated (#552)"
        );
        assert!(
            got.contains(&"private:did:key:z6MkAbc:*".to_string()),
            "mid trailing-* parent MUST be enumerated (#552)"
        );
        assert!(
            got.contains(&"private:*".to_string()),
            "shallow trailing-* parent MUST be enumerated (#552)"
        );
        // Bounded: linear (n trailing parents + exact + "*"), NOT 2^8.
        assert!(
            got.len() <= deep.split(':').count() + 2,
            "deep-scope candidate set must stay linear (no 2^N blowup)"
        );
    }

    #[test]
    fn deep_scope_exact_match_still_first() {
        let deep = "a:b:c:d:e:f:g:h";
        let got = wildcard_variants(deep);
        assert_eq!(got[0], deep);
    }
}
