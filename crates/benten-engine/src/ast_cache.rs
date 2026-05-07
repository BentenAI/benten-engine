//! Per-handler TRANSFORM AST cache (Phase-3 G19-E — wave-7b).
//!
//! Closes `docs/future/phase-2-backlog.md` §9.2. The
//! `crate::engine::SubgraphCache` (G2-B / arch-r1-5) caches built
//! `benten_eval::Subgraph` *templates* keyed on
//! `(handler_id, op, subgraph_cid)`. That cache memoises the structural
//! shape — node + edge bag, static property keys — but does NOT eliminate
//! the per-call parse of TRANSFORM expression source: every call into
//! `Engine::call` previously routed through `transform::execute`, which
//! re-parsed the `expr` string on every dispatch (the §9.2 deferral).
//!
//! G19-E adds a sibling cache that stores the *parsed* `Expr` AST for
//! every TRANSFORM node in a registered handler. The lookup key is
//! `(handler_cid, node_id)` — orthogonal to the `SubgraphCache` key
//! because TRANSFORM ASTs are content-defined by the registered subgraph
//! shape, not by the dispatch op. Population happens at
//! `register_subgraph` / `register_subgraph_replace` time (synchronous
//! parse walk through every `PrimitiveKind::Transform` node);
//! invalidation happens at `register_subgraph_replace` (the OLD
//! `handler_cid`'s entries are dropped before the swap).
//!
//! Wired into the dispatch path via
//! [`benten_eval::PrimitiveHost::cached_transform_ast`]: the engine's
//! `impl PrimitiveHost for Engine` resolves the active call's
//! `handler_cid` from the `active_call` stack and returns the cached
//! `Arc<Expr>` for the supplied `node_id`. On miss (or for handlers
//! registered before the cache was populated, e.g. forced re-register
//! via the `testing_force_reregister_with_different_cid` hook) the
//! TRANSFORM executor falls back to the per-call parse so behaviour is
//! identical.
//!
//! # Cache statistics
//!
//! [`AstCacheStats`] surfaces hit / miss counts so the
//! `subgraph_ast_cache_full_wire_up` integration test can verify the
//! wire-up actually serves cached ASTs (defends against the
//! "cache exists but never consulted" failure mode named in the R3-E
//! pin's pim-2 §3.6b end-to-end requirement).
//!
//! # Concurrency
//!
//! `AstCache` uses `RwLock<HashMap<...>>` matching the `SubgraphCache`
//! discipline. Lookups (the hot path) take a read lock; population +
//! invalidation take a write lock. Populating is rare (registration
//! only); lookups happen on every TRANSFORM dispatch.
//!
//! # Cross-pin coordination (stream-r1-3 / stream-r4r1-9)
//!
//! The cache only stores parsed `Expr` ASTs for `PrimitiveKind::Transform`
//! nodes. STREAM / SUBSCRIBE primitives carry no `expr` property and are
//! never inserted, so the cache cannot inadvertently route around the
//! loud-fail discipline that R6FP-G1 r6-stream-3 closed for STREAM
//! eval-side `execute()` (and the symmetric SUBSCRIBE arm).

use std::collections::HashMap;
use std::sync::{Arc, RwLock, atomic::AtomicU64, atomic::Ordering};

use benten_core::Cid;
use benten_eval::expr::{Expr, parser};

/// Cache key — `(handler_cid, node_id)`.
///
/// `handler_cid` is the registered subgraph's content-addressed CID so a
/// re-registration that flips the CID (the load-bearing correctness pin)
/// produces a fresh key namespace and the OLD entries become unreachable
/// (in addition to being explicitly dropped at invalidation time).
#[derive(Clone, PartialEq, Eq, Hash)]
struct AstCacheKey {
    handler_cid: Cid,
    node_id: String,
}

/// Snapshot of the cache's runtime statistics — exposed via the
/// `testing_ast_cache_stats` engine accessor for the wire-up integration
/// test.
#[derive(Debug, Clone, Copy, Default)]
pub struct AstCacheStats {
    /// Cumulative count of `lookup()` calls that returned `Some`.
    pub hits: u64,
    /// Cumulative count of `lookup()` calls that returned `None`.
    pub misses: u64,
    /// Current number of cached `(handler_cid, node_id)` entries.
    pub entries: usize,
}

/// Per-handler TRANSFORM AST cache. Stores parsed `Expr` keyed by
/// `(handler_cid, node_id)`.
#[derive(Default)]
pub(crate) struct AstCache {
    entries: RwLock<HashMap<AstCacheKey, Arc<Expr>>>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl AstCache {
    /// Construct a fresh empty cache.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Look up the parsed AST for the given `(handler_cid, node_id)`.
    ///
    /// Returns `Some(Arc<Expr>)` on a hit; `None` on a miss. Bumps the
    /// internal hit / miss counters as a side effect so the integration
    /// test can verify the wire-up.
    pub(crate) fn lookup(&self, handler_cid: &Cid, node_id: &str) -> Option<Arc<Expr>> {
        let guard = benten_graph::RwLockExt::read_recover(&self.entries);
        let key = AstCacheKey {
            handler_cid: *handler_cid,
            node_id: node_id.to_string(),
        };
        match guard.get(&key) {
            Some(arc) => {
                self.hits.fetch_add(1, Ordering::SeqCst);
                Some(Arc::clone(arc))
            }
            None => {
                self.misses.fetch_add(1, Ordering::SeqCst);
                None
            }
        }
    }

    /// Walk the registered subgraph and insert a parsed `Expr` for every
    /// `PrimitiveKind::Transform` node. Pre-validation guarantees the
    /// `expr` property is parseable (or absent — TRANSFORM nodes without
    /// `expr` route `ON_ERROR` at runtime, so we skip them here so the
    /// cache only carries entries that would otherwise be re-parsed).
    ///
    /// Idempotent on the same `(handler_cid, sg)` pair: re-populating
    /// produces the same Expr instances (different `Arc`s, but
    /// structurally identical) — callers should prefer to populate once
    /// per registration.
    ///
    /// Returns the number of entries inserted (informational; populate
    /// is best-effort). Parse errors are silently ignored — they would
    /// have surfaced from `validate_transform_expressions` already; if
    /// somehow we reach here with an unparseable expr (e.g. test-only
    /// hooks that bypass validation), the runtime executor falls through
    /// to the per-call parse path which surfaces the typed
    /// `TransformSyntax` Err.
    pub(crate) fn populate_for_handler(
        &self,
        handler_cid: &Cid,
        sg: &benten_eval::Subgraph,
    ) -> usize {
        use benten_core::Value;
        use benten_eval::PrimitiveKind;

        let mut guard = benten_graph::RwLockExt::write_recover(&self.entries);
        let mut inserted = 0_usize;
        for node in &sg.nodes {
            if !matches!(node.kind, PrimitiveKind::Transform) {
                continue;
            }
            let Some(Value::Text(src)) = node.properties.get("expr") else {
                continue;
            };
            let Ok(expr) = parser::parse(src) else {
                // Should not happen — validate_transform_expressions runs
                // first. Defensive fallthrough: skip the entry; runtime
                // path will produce the typed parse error.
                continue;
            };
            let key = AstCacheKey {
                handler_cid: *handler_cid,
                node_id: node.id.clone(),
            };
            guard.insert(key, Arc::new(expr));
            inserted += 1;
        }
        inserted
    }

    /// Drop every entry under the given `handler_cid`.
    ///
    /// Used by `register_subgraph_replace` (and the
    /// `testing_force_reregister_with_different_cid` hook) so the new
    /// handler version's calls don't accidentally serve a stale parse
    /// from the prior version's bytes. The new version's entries are
    /// inserted afterward via [`Self::populate_for_handler`].
    pub(crate) fn invalidate_handler(&self, handler_cid: &Cid) {
        let mut guard = benten_graph::RwLockExt::write_recover(&self.entries);
        guard.retain(|key, _| &key.handler_cid != handler_cid);
    }

    /// Snapshot of the cache's hit / miss counters and current entry
    /// count. The counters are read with `SeqCst` so the test surface
    /// observes the same ordering the lookup path stamps.
    pub(crate) fn stats(&self) -> AstCacheStats {
        let entries = benten_graph::RwLockExt::read_recover(&self.entries).len();
        AstCacheStats {
            hits: self.hits.load(Ordering::SeqCst),
            misses: self.misses.load(Ordering::SeqCst),
            entries,
        }
    }

    /// Reset the hit / miss counters to zero. Test-only — used by the
    /// integration tests + benchmark to measure a single dispatch
    /// sequence cleanly.
    #[cfg(any(test, feature = "test-helpers"))]
    pub(crate) fn reset_counters(&self) {
        self.hits.store(0, Ordering::SeqCst);
        self.misses.store(0, Ordering::SeqCst);
    }
}
