//! Phase-3 G14-C wave-4b — anchor-store consolidation (cov-f3).
//!
//! ## What this is
//!
//! Closure of the `cov-f3` residual carried from Phase-2b
//! (`docs/future/phase-2-backlog.md` §6.3). Phase-2 left
//! `benten-core/src/version.rs` exposing two coexisting version-chain
//! shapes (the per-anchor `Arc<Mutex>` chain + the simpler `u64`-id
//! anchor pattern at the crate root) AND a tracked open question of
//! whether a dedicated `AnchorStore` handle would consolidate the
//! several ad-hoc anchor-fetching helpers spread across the engine.
//!
//! G14-C consolidates by exposing **one** anchor-store API on
//! [`Engine`]: [`Engine::anchor_store`]. The store wraps the existing
//! [`benten_core::version::Anchor`] surface with engine-side
//! durable-fetch / list-by-handler accessors so callers don't go
//! crate-shopping for the right entry point.
//!
//! ## What was the residual
//!
//! Per `cov-f3`, the residual was that:
//! 1. `core::version` exposed both `Anchor::new(head)` (CID-threaded)
//!    and the `u64`-id `Anchor::new()` shape — callers had to know
//!    which one to use.
//! 2. The engine's handler-version chain (Compromise #18; closed
//!    above) fetched chains by ad-hoc `inner.handler_version_chain`
//!    map access; there was no single accessor.
//! 3. No site offered a bulk "list all anchors / list anchors by
//!    handler" API.
//!
//! G14-C resolves the third point by exposing
//! [`AnchorStore::list_handler_chains`]; the first two were already
//! resolved by D-PHASE-3-19a (the canonical CID-threaded shape) +
//! the handler_version_chain accessor on `Engine`.

use std::collections::BTreeMap;

use benten_core::Cid;
use benten_core::version::Anchor;

use crate::engine::Engine;
use crate::error::EngineError;
use crate::handler_versions::HandlerVersionChain;

/// G14-C consolidated anchor-store handle. One handle exposes the
/// version-chain accessors that the engine + audit consumers need;
/// callers no longer have to dispatch through the in-memory
/// `handler_version_chain` BTreeMap directly.
pub struct AnchorStore<'a> {
    engine: &'a Engine,
}

impl<'a> AnchorStore<'a> {
    /// Construct a fresh handle bound to `engine`. Cheap (one
    /// reference); no allocation.
    #[must_use]
    pub fn new(engine: &'a Engine) -> Self {
        Self { engine }
    }

    /// Fetch the `core::version::Anchor` rooted at the chain's first
    /// registered version for `handler_id`. Returns `None` when the
    /// handler has no registered versions.
    ///
    /// Equivalent to
    /// [`Engine::handler_version_chain_with_anchor`]'s `anchor`
    /// field; exposed here for callers that only want the anchor.
    #[must_use]
    pub fn fetch_handler_anchor(&self, handler_id: &str) -> Option<Anchor> {
        self.engine
            .handler_version_chain_with_anchor(handler_id)
            .and_then(|c| c.anchor)
    }

    /// Fetch the full [`HandlerVersionChain`] for `handler_id`,
    /// including the anchor + the newest-first version list.
    #[must_use]
    pub fn fetch_handler_chain(&self, handler_id: &str) -> Option<HandlerVersionChain> {
        self.engine.handler_version_chain_with_anchor(handler_id)
    }

    /// List every handler that has at least one registered version
    /// chain, mapping `handler_id` → newest-first `Vec<Cid>`.
    ///
    /// # Errors
    ///
    /// Currently infallible (reads the in-memory rebuild), but the
    /// `Result` shape is preserved for Phase-4 + when the chain reads
    /// move back to the durable backend.
    pub fn list_handler_chains(&self) -> Result<BTreeMap<String, Vec<Cid>>, EngineError> {
        let guard = self.engine.handler_version_chain_in_memory_lock();
        Ok(guard.clone())
    }
}

impl Engine {
    /// Phase-3 G14-C — return a handle to the consolidated
    /// anchor-store API (cov-f3 closure).
    #[must_use]
    pub fn anchor_store(&self) -> AnchorStore<'_> {
        AnchorStore::new(self)
    }
}
