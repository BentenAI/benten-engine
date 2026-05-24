//! `Engine::walk_share_scope` — engine wrapper around the canonical
//! `benten_core::subgraph_spec::walker::walk` BFS-order enumerator.
//!
//! Per V1-FROZEN-INTERFACE.md row 4 / §1.A.FROZEN item 15(h): the
//! engine consumer surface for the SubgraphSpec walker. The walker
//! itself lives in `benten_core` (data-not-evaluator-extension —
//! preserves CLAUDE.md baked-in #1 12-primitive irreducibility); this
//! engine method is the thin orchestration that exposes the walk
//! through the `Engine` namespace alongside the other principal-aware
//! surfaces (`read_node_as`, `call_as`).
//!
//! The walker is itself a Subgraph composed of the existing 12
//! operation primitives — see
//! [`benten_core::subgraph_spec::walker::walker_as_subgraph`] for the
//! fractal-property pin.
//!
//! **What "frozen" means here (per V1-FROZEN-INTERFACE item 15.h):**
//! - The method signature `pub fn walk_share_scope(&self, spec: &Spec)
//!   -> Result<WalkResult, EngineError>` is the v1-beta surface.
//! - The walker delegates to the data-walker; the engine does NOT
//!   reimplement the BFS — that would risk drift between the producer's
//!   enumeration order and the recipient's.
//! - BFS-order enumeration is wire-bytes-load-bearing (path-tagged keys
//!   depend on it; see [`benten_core::EncryptionClass`] +
//!   `RATIFIED-S&C` §R4 + 15(h) freeze).

use benten_core::subgraph_spec::{Spec, SubgraphSpecError, WalkResult, walker};
use benten_errors::ErrorCode;

use crate::engine::Engine;
use crate::error::EngineError;

impl Engine {
    /// Walk the [`Spec`] rooted at the given root CIDs + return the
    /// BFS-enumerated `(Cid, StructuralPath)` set per RATIFIED-S&C
    /// §R4.
    ///
    /// The walk is data-not-evaluator-extension: the engine delegates
    /// to the `benten_core::subgraph_spec::walker` (which IS itself a
    /// Subgraph composed of the existing 12 primitives — no new
    /// `PrimitiveKind` variant is minted; see
    /// [`benten_core::subgraph_spec::walker::walker_as_subgraph`]).
    ///
    /// This is the v1-beta public consumer surface for the SubgraphSpec
    /// walker per V1-FROZEN-INTERFACE row 4 / §1.A.FROZEN item 15(h).
    /// The §1.A.FROZEN signature freeze pins the `(&Spec) ->
    /// Result<WalkResult, EngineError>` shape; principal-bearing
    /// variants (e.g. `walk_share_scope_as(&self, principal, spec)`)
    /// are ADDITIVE Composing-time enhancements that may co-exist with
    /// this method.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::Other`] wrapping the inner
    /// [`SubgraphSpecError`] when the spec is malformed or the walker
    /// surfaces a typed error. The wrapper carries
    /// [`ErrorCode::SubgraphSpecWalkFailed`] so the napi + drift-detect
    /// surfaces have a stable code.
    pub fn walk_share_scope(&self, spec: &Spec) -> Result<WalkResult, EngineError> {
        let _ = self; // engine handle reserved for principal-threading enrichment in Composing
        walker::walk(spec).map_err(spec_err_to_engine)
    }
}

fn spec_err_to_engine(err: SubgraphSpecError) -> EngineError {
    EngineError::Other {
        code: ErrorCode::SubgraphSpecWalkFailed,
        message: err.to_string(),
    }
}
