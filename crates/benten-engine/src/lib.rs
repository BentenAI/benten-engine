//! # benten-engine
//!
//! Orchestrator crate composing the Benten graph engine public API.
//!
//! In the spike the surface is intentionally minimal — just enough to serve
//! the napi bindings:
//!
//! - [`Engine::open`] opens a redb-backed graph store.
//! - [`Engine::create_node`] canonicalizes, hashes, and persists a Node.
//! - [`Engine::get_node`] retrieves a Node by CID.
//!
//! Phase 1 proper will wire in the capability hook from `benten-caps`, the
//! IVM subscriber from `benten-ivm`, and the evaluator from `benten-eval`.
//! Those crates exist as stubs today so the workspace compiles against the
//! six-crate plan.

#![forbid(unsafe_code)]

use std::path::Path;

use benten_core::{Cid, CoreError, Node};
use benten_graph::{GraphError, RedbBackend};

// Touch the stub crates so the dependency graph is real, not just declared.
// Every new compile checks they still build. Cheap keep-alives.
const _: &str = benten_caps::STUB_MARKER;
const _: &str = benten_eval::STUB_MARKER;
const _: &str = benten_ivm::STUB_MARKER;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by the engine orchestrator.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// Propagated from `benten-core`.
    #[error("core: {0}")]
    Core(#[from] CoreError),

    /// Propagated from `benten-graph`.
    #[error("graph: {0}")]
    Graph(#[from] GraphError),
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// The Benten engine handle. Thin: owns the storage backend and exposes
/// create/get operations.
pub struct Engine {
    backend: RedbBackend,
}

impl Engine {
    /// Open or create an engine backed by a redb database at `path`.
    ///
    /// # Errors
    /// Returns [`EngineError::Graph`] if the storage backend cannot be opened.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        let backend = RedbBackend::open(path)?;
        Ok(Self { backend })
    }

    /// Hash `node` (CIDv1 over labels + properties only), store it, and return
    /// its CID. Idempotent: storing the same Node twice is a no-op from the
    /// caller's perspective — the second call overwrites a byte-identical
    /// value under the same key.
    ///
    /// # Errors
    /// Returns [`EngineError::Core`] on serialization failure or
    /// [`EngineError::Graph`] on storage failure.
    pub fn create_node(&self, node: &Node) -> Result<Cid, EngineError> {
        Ok(self.backend.put_node(node)?)
    }

    /// Retrieve a Node by CID. Returns `Ok(None)` on a clean miss.
    ///
    /// # Errors
    /// Returns [`EngineError::Graph`] on storage errors.
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, EngineError> {
        Ok(self.backend.get_node(cid)?)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use benten_core::testing::canonical_test_node;

    #[test]
    fn create_then_get_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
        let node = canonical_test_node();
        let cid = engine.create_node(&node).unwrap();
        let fetched = engine.get_node(&cid).unwrap().expect("node exists");
        assert_eq!(fetched, node);
        assert_eq!(fetched.cid().unwrap(), cid);
    }

    #[test]
    fn missing_cid_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
        let cid = canonical_test_node().cid().unwrap();
        assert!(engine.get_node(&cid).unwrap().is_none());
    }
}
