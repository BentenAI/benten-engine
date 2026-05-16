//! G24-A admin UI v0 → benten-engine adapter.
//!
//! Wires the [`MaterializerEngine`] trait to a real [`Engine`] so the
//! admin UI v0 module's consumer surface
//! ([`benten_platform_foundation::admin_ui_v0::render_category_content`])
//! can route reads through [`Engine::read_node_as`] — Class B β per
//! CLAUDE.md baked-in #18 (cag-r1-9). Production callers (the
//! napi-bridge `delegate_capability` path + the future plugin install
//! bootstrap) construct the exact same adapter shape; this test-side
//! adapter pins the contract.
//!
//! The adapter is intentionally placed in the test crate (NOT in
//! production) per the dep-direction commitment (arch-r1-1 +
//! arch-r1-15): `benten-platform-foundation` does NOT depend on
//! `benten-engine` in production. The test crate has `benten-engine`
//! as a dev-dep — the adapter is reachable from test fixtures here
//! and at the napi binding layer.

#![allow(dead_code)]

use benten_core::{Cid, Node};
use benten_engine::Engine;
use benten_platform_foundation::{MaterializerEngine, MaterializerError};

/// Adapter binding the [`MaterializerEngine`] trait to a real
/// [`benten_engine::Engine`]. Routes EVERY READ through
/// [`Engine::read_node_as`] — never the engine-internal
/// `pub(crate) Engine::read_node` seam (cag-r1-9 + CLAUDE.md #18).
pub struct EngineMaterializerAdapter<'a> {
    pub engine: &'a Engine,
    pub clock_injected: bool,
}

impl<'a> EngineMaterializerAdapter<'a> {
    #[must_use]
    pub fn new(engine: &'a Engine) -> Self {
        Self {
            engine,
            // Default `Engine::open` does NOT inject a clock; callers
            // that want the wallclock-fail-closed posture honoured
            // should construct via `with_clock_injected(true)`.
            clock_injected: true,
        }
    }

    #[must_use]
    pub fn with_clock_injected(mut self, value: bool) -> Self {
        self.clock_injected = value;
        self
    }
}

impl<'a> MaterializerEngine for EngineMaterializerAdapter<'a> {
    fn read_node_as(&self, principal: &Cid, cid: &Cid) -> Result<Option<Node>, MaterializerError> {
        // The CLASS B β seam (CLAUDE.md #18). NEVER call `read_node` —
        // that is `pub(crate)` for engine internals (IVM / sync /
        // audit / view materialization); reaching for it from a
        // plugin-context adapter would BYPASS the cap-recheck
        // boundary at the engine-side.
        self.engine
            .read_node_as(principal, cid)
            .map_err(|e| MaterializerError::SchemaMismatch {
                reason: format!("engine read_node_as backend error: {e}"),
            })
    }

    fn has_clock_injected(&self) -> bool {
        self.clock_injected
    }
}
