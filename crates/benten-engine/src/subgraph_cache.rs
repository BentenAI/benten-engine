//! Phase 2a G2-B / arch-r1-5: `SubgraphCacheKey` + `SubgraphCache` — FROZEN
//! key shape.
//!
//! The cache key is `(handler_id, op, subgraph_cid)`: arch-r1-5 requires the
//! subgraph CID to anchor the cache entry so a re-registration under a new
//! CID forces a miss-then-parse cycle rather than serving the stale AST.

use std::cell::RefCell;
use std::collections::HashMap;

use benten_core::Cid;

/// Three-axis cache key. Arch-r1-5 / plan §3 G2-B.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubgraphCacheKey {
    /// Handler id (human-readable alias that may re-resolve).
    pub handler_id: String,
    /// Operation op tag (`"run"`, `"get"`, `"delete"`, …).
    pub op: String,
    /// Authoritative subgraph CID — distinguishes two re-registrations of
    /// the same `(handler_id, op)` pair.
    pub subgraph_cid: Cid,
}

impl SubgraphCacheKey {
    /// Construct a three-axis cache key.
    #[must_use]
    pub fn new(handler_id: String, op: String, subgraph_cid: Cid) -> Self {
        Self {
            handler_id,
            op,
            subgraph_cid,
        }
    }
}

/// Minimal AST cache keyed on [`SubgraphCacheKey`]. Phase-2a stub shape —
/// G2-B lands the wire-through into `Engine::call`.
pub struct SubgraphCache {
    entries: RefCell<HashMap<SubgraphCacheKey, String>>,
}

impl SubgraphCache {
    /// Construct an empty cache for tests.
    #[must_use]
    pub fn new_for_test() -> Self {
        Self {
            entries: RefCell::new(HashMap::new()),
        }
    }

    /// Probe the cache.
    #[must_use]
    pub fn contains(&self, key: &SubgraphCacheKey) -> bool {
        self.entries.borrow().contains_key(key)
    }

    /// Phase-2a test helper: insert an entry whose value is a tag string
    /// identifying the cached AST.
    pub fn insert_for_test(&self, handler_id: &str, op: &str, cid: &Cid, ast_tag: &str) {
        self.entries.borrow_mut().insert(
            SubgraphCacheKey::new(handler_id.into(), op.into(), *cid),
            ast_tag.to_string(),
        );
    }

    /// Phase-2a test helper: probe a key by its three axes.
    pub fn get_for_test(&self, handler_id: &str, op: &str, cid: &Cid) -> Option<String> {
        let key = SubgraphCacheKey::new(handler_id.into(), op.into(), *cid);
        self.entries.borrow().get(&key).cloned()
    }
}

impl Default for SubgraphCache {
    fn default() -> Self {
        Self::new_for_test()
    }
}
