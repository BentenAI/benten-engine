//! Extension traits for the relocated `Subgraph` + `SubgraphBuilder` types.
//!
//! Phase-2b G12-C-cont (Phase 2b R6 A1 closure) moves `Subgraph`,
//! `SubgraphBuilder`, `OperationNode`, `PrimitiveKind`, and `NodeHandle` into
//! `benten-core`. The eval-side methods that depended on the `invariants`
//! module (validation, multiplicative-budget walk, Mermaid render) cannot
//! follow because of Rust's "no inherent impls on foreign types" rule and
//! the arch-1 invariant that `benten-core` MUST NOT depend on `benten-eval`.
//!
//! Those methods live here as **extension traits**:
//!
//! - [`SubgraphBuilderExt`] — `build_validated`, `build_validated_with_max_depth`,
//!   `build_validated_aggregate_all`. Re-runs the invariants validator against
//!   the builder's snapshot before returning the finalized [`Subgraph`].
//! - [`SubgraphExt`] — `validate`, `cumulative_budget_for_root_for_test`,
//!   `cumulative_budget_for_handle_for_test`,
//!   `has_multiplicative_budget_tracked_for_test`, `to_mermaid`,
//!   `load_verified` (RegistrationError-typed). Backed by the same `invariants/`
//!   module that the pre-relocation inherent methods called into.
//!
//! Existing callsites import the eval-side surface (`use benten_eval::{Subgraph,
//! SubgraphBuilder};`); to keep `b.build_validated()?` / `sg.validate(&cfg)?`
//! working, callers add `use benten_eval::{SubgraphBuilderExt, SubgraphExt};`
//! (or import from the eval prelude). The trait methods are **distinct names**
//! from any inherent methods on the relocated types — there is no shadowing
//! and no method-resolution ambiguity. `benten-core` deliberately exposes
//! only the data surface (`build_unvalidated_for_test`, `load_verified`,
//! `load_verified_with_cid`), leaving validation + Mermaid + budget walks as
//! the eval-side concern (which depends on the `invariants/` module that
//! cannot follow into benten-core under arch-1).

use benten_core::{Cid, NodeHandle, OperationNode, PrimitiveKind, Subgraph, SubgraphBuilder};

use crate::{
    EvalError, InvariantConfig, InvariantViolation, RegistrationError, SubgraphSnapshot, invariants,
};

// G12-C-cont fix-pass A.6 (arch-mr-g12c-cont-1): seal the extension traits so
// downstream crates cannot impl them on their own types. The relocated types
// (`Subgraph`, `SubgraphBuilder`, `NodeHandle`) live in benten-core; these
// traits exist only to project eval-side methods onto them. Allowing arbitrary
// downstream impls would invite ambiguity at the method-resolution callsites
// (`b.build_validated()?` etc.) and break the Phase-3 OSS surface contract.
mod private {
    use benten_core::{NodeHandle, Subgraph, SubgraphBuilder};
    pub trait Sealed {}
    impl Sealed for SubgraphBuilder {}
    impl Sealed for Subgraph {}
    impl Sealed for NodeHandle {}
}

/// Extension trait for [`benten_core::SubgraphBuilder`] that re-runs the
/// `benten-eval::invariants` validator before producing a finalized
/// [`Subgraph`]. Mirrors the pre-G12-C-cont inherent-method contract.
pub trait SubgraphBuilderExt: private::Sealed {
    /// Build with structural validation (invariants 1/2/3/5/6/9/10/12).
    /// Fails fast on the first invariant violation encountered.
    ///
    /// # Errors
    /// Returns a [`RegistrationError`] carrying per-invariant diagnostic
    /// context when any structural invariant is violated.
    fn build_validated(self) -> Result<Subgraph, RegistrationError>;

    /// Build with a caller-supplied max-depth cap for the Invariant-2 check.
    ///
    /// # Errors
    /// Returns a [`RegistrationError`] when any structural invariant is
    /// violated — in particular when the longest path exceeds `cap`.
    fn build_validated_with_max_depth(self, cap: usize) -> Result<Subgraph, RegistrationError>;

    /// Aggregate-mode build — returns a single error listing every failed
    /// invariant, instead of stopping at the first.
    ///
    /// # Errors
    /// Returns a [`RegistrationError`] with `InvariantViolation::Registration`-
    /// style aggregation populating the `violated_invariants` list when two or
    /// more invariants fail; a single violation still surfaces its specific
    /// code per the `single_violation_uses_specific_code_not_catch_all`
    /// contract.
    fn build_validated_aggregate_all(self) -> Result<Subgraph, RegistrationError>;

    /// Phase 2a G3-B: WAIT signal variant with optional static typing
    /// ([`crate::SignalShape`]). Cannot live on the core-side builder
    /// because `SignalShape` is an eval-side type; ships here as an
    /// extension method.
    fn wait_signal_typed(
        &mut self,
        prev: NodeHandle,
        signal_name: impl Into<String>,
        shape: crate::SignalShape,
    ) -> NodeHandle;
}

/// Build a `SubgraphSnapshot<'_>` borrowing the builder's internal state via
/// the validator-accessor surface that benten-core exposes.
fn snapshot_of(b: &SubgraphBuilder) -> SubgraphSnapshot<'_> {
    SubgraphSnapshot {
        nodes: b.nodes_for_validator(),
        parallel_fanout: b.parallel_fanout_for_validator(),
        iterate_depth: b.iterate_depth_for_validator(),
        edges: b.edges_for_validator(),
        extra_edges: b.extra_edges_for_validator(),
        deterministic: b.deterministic_for_validator(),
        handler_id: b.handler_id_for_validator(),
    }
}

impl SubgraphBuilderExt for SubgraphBuilder {
    fn build_validated(self) -> Result<Subgraph, RegistrationError> {
        let cfg = InvariantConfig::default();
        invariants::validate_builder(&snapshot_of(&self), &cfg, false)?;
        Ok(self.build_unvalidated_for_test())
    }

    fn build_validated_with_max_depth(self, cap: usize) -> Result<Subgraph, RegistrationError> {
        let mut cfg = InvariantConfig::default();
        cfg.max_depth = u32::try_from(cap).unwrap_or(u32::MAX);
        invariants::validate_builder(&snapshot_of(&self), &cfg, false)?;
        Ok(self.build_unvalidated_for_test())
    }

    fn build_validated_aggregate_all(self) -> Result<Subgraph, RegistrationError> {
        let cfg = InvariantConfig::default();
        invariants::validate_builder(&snapshot_of(&self), &cfg, true)?;
        Ok(self.build_unvalidated_for_test())
    }

    fn wait_signal_typed(
        &mut self,
        prev: NodeHandle,
        signal_name: impl Into<String>,
        shape: crate::SignalShape,
    ) -> NodeHandle {
        let h = self.wait_signal(prev, signal_name);
        if let crate::SignalShape::Typed(v) = shape
            && let Some(n) = self.nodes.get_mut(h.0 as usize)
        {
            n.properties.insert("signal_shape".into(), v);
        }
        h
    }
}

/// Extension trait for [`benten_core::Subgraph`] exposing the eval-side
/// validation, multiplicative-budget, and Mermaid rendering methods that
/// stayed in `benten-eval` after the G12-C-cont type relocation.
pub trait SubgraphExt: private::Sealed {
    /// Registration-time structural validation (invariants 1/2/3/5/6/9/10/12).
    /// Delegates to the `invariants::validate_subgraph` finalized-subgraph
    /// path. Returns the first violation as an `EvalError::Invariant`.
    ///
    /// # Errors
    /// Returns [`EvalError::Invariant`] carrying the violated invariant kind
    /// when structural validation fails.
    fn validate(&self, config: &InvariantConfig) -> Result<(), EvalError>;

    /// Phase 2a G4-A test helper: return the cumulative Inv-8 budget at
    /// the subgraph's worst-case path.
    fn cumulative_budget_for_root_for_test(&self) -> u64;

    /// Phase 2a G4-A test helper: cumulative budget at an arbitrary handle.
    /// Returns `None` when the handle does not correspond to a node in this
    /// subgraph.
    fn cumulative_budget_for_handle_for_test(&self, h: NodeHandle) -> Option<u64>;

    /// Phase 2a G4-A test helper: multiplicative Inv-8 budget tracking is
    /// live in Phase 2a.
    fn has_multiplicative_budget_tracked_for_test(&self) -> bool;

    /// Mermaid flowchart serialization. Behind the `diag` feature; without
    /// it returns an empty string.
    fn to_mermaid(&self) -> String;

    /// Reconstruct a Subgraph from content-addressed bytes + declared CID.
    /// The CID is verified against the bytes; mismatch -> `ErrorCode::InvContentHash`.
    /// Mirrors the pre-G12-C-cont inherent-method contract: returns a
    /// `RegistrationError` (vs. the `CoreError`-typed
    /// [`benten_core::Subgraph::load_verified_with_cid`]) so existing eval-side
    /// callers don't need an error-conversion layer.
    ///
    /// Spelled `load_verified_eval` (vs. plain `load_verified`) because the
    /// core-side `Subgraph` already carries an inherent `load_verified(bytes)`
    /// method (1-arg, no CID check, `CoreError`-typed); a same-named trait
    /// method would be ambiguous at `Subgraph::load_verified` static-call
    /// sites.
    ///
    /// # Errors
    /// Returns a [`RegistrationError`] with `InvariantViolation::ContentHash`
    /// when the computed CID does not match the declared one.
    fn load_verified_eval(cid: &Cid, bytes: &[u8]) -> Result<Subgraph, RegistrationError>;
}

impl SubgraphExt for Subgraph {
    fn validate(&self, config: &InvariantConfig) -> Result<(), EvalError> {
        match invariants::validate_subgraph(self, config, false) {
            Ok(()) => Ok(()),
            Err(reg) => Err(EvalError::Invariant(reg.kind)),
        }
    }

    fn cumulative_budget_for_root_for_test(&self) -> u64 {
        invariants::budget::compute_cumulative(self)
    }

    fn cumulative_budget_for_handle_for_test(&self, h: NodeHandle) -> Option<u64> {
        invariants::budget::cumulative_at_handle(self, h)
    }

    fn has_multiplicative_budget_tracked_for_test(&self) -> bool {
        true
    }

    fn to_mermaid(&self) -> String {
        #[cfg(feature = "diag")]
        {
            crate::diag::mermaid::render(self)
        }
        #[cfg(not(feature = "diag"))]
        {
            String::new()
        }
    }

    fn load_verified_eval(cid: &Cid, bytes: &[u8]) -> Result<Subgraph, RegistrationError> {
        let digest = blake3::hash(bytes);
        let actual = Cid::from_blake3_digest(*digest.as_bytes());
        if actual != *cid {
            let mut err = RegistrationError::new(InvariantViolation::ContentHash);
            err.expected_cid = Some(*cid);
            err.actual_cid = Some(actual);
            return Err(err);
        }
        // G12-C-cont fix-pass A.9 (cr-mr-g12c-cont-4 + arch-mr-g12c-cont-2):
        // propagate decode failures rather than swallowing them with an
        // empty-Subgraph placeholder. The hash check above rejects tampered
        // bytes; a decode failure here indicates encoder drift, not tamper —
        // surfacing it as a `ContentHash` violation lets callers diagnose
        // the drift instead of silently operating on a placeholder. Pre-fix-pass
        // code returned `Ok(Subgraph::new("loaded"))` on decode error (verbatim
        // from the deleted eval-side body); both code-reviewer + architect
        // lenses flagged this as a swallow.
        Subgraph::load_verified_with_cid(cid, bytes).map_err(|_| {
            let mut err = RegistrationError::new(InvariantViolation::ContentHash);
            err.expected_cid = Some(*cid);
            err.actual_cid = Some(actual);
            err
        })
    }
}

/// Extension trait for [`benten_core::NodeHandle`] exposing the eval-side
/// `build_validated_for_corruption_test` constructor that the
/// `subgraph_corruption.rs` test reaches in via.
pub trait NodeHandleExt: private::Sealed {
    /// Test-only constructor for the corruption-test path. Produces a
    /// deterministic single-node subgraph (no edges) so two invocations
    /// produce identical canonical bytes.
    fn build_validated_for_corruption_test(self) -> Subgraph;
}

impl NodeHandleExt for NodeHandle {
    fn build_validated_for_corruption_test(self) -> Subgraph {
        Subgraph::new("corruption_test").with_node(OperationNode::new("r", PrimitiveKind::Read))
    }
}
