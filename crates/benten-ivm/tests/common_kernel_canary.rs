//! Shared test fixtures for the G23-0a IVM-subgraph generalization kernel —
//! **Family B canary surface** (R5 G23-0a + G23-0b WIRED-TO-PRODUCTION).
//!
//! ## Purpose (Family B → Family C dependency)
//!
//! Family B (G23-0a — generalized Algorithm B kernel that consumes a
//! `SubgraphSpec` as view definition) owns this canary shape. Family C
//! (G23-0b — re-express the 5 canonical hand-written views as
//! `SubgraphSpec` consumers) imports these helpers from its 5 canonical-
//! view round-trip test files + the proptest equivalence pin.
//!
//! ## G23-0b: wired to production
//!
//! At G23-0a + G23-0b landing time this module's stub
//! `register_and_walk_to_completion` is replaced with the production
//! call shape — `benten_ivm::Algorithm::register_subgraph(spec)?
//! .walk_writes(&writes)?`. The canary types `KernelInput` /
//! `KernelOutput` / `TypedOutputProjection` are now re-exports of
//! production types from `benten_ivm`. `CanarySubgraphSpec` is preserved
//! as a thin convenience wrapper consumers already import; conversion
//! to production `benten_ivm::SubgraphSpec` happens inside the helper.
//!
//! ## §3.6b would-FAIL-if-no-op'd
//!
//! Each consumer pin observes a substantive arm of the generalization:
//! kernel-input shape (SubgraphSpec round-trip), no-new-Strategy-variant
//! count, self-reference rejection, hot-path ≤20% overhead. A no-op
//! G23-0a/G23-0b implementation (e.g. returning an empty view set,
//! panicking, or quietly succeeding with stub bytes) would FAIL each
//! consumer pin because the helper drives PRODUCTION code paths.

#![allow(
    dead_code,
    reason = "canary helpers are scenario-specific; not every helper is \
              referenced by every consumer in Family B + Family C"
)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::missing_panics_doc,
    clippy::needless_pass_by_value
)]

use benten_ivm::algorithm_b::AlgorithmError;
use benten_ivm::{
    Algorithm, LabelPattern, Projection, SubgraphSpec, ViewError,
    is_canonical_view_id as production_is_canonical_view_id,
};

// =============================================================================
// Re-exports of production types — Family C consumes these
// =============================================================================

/// Re-export of [`benten_ivm::KernelInput`] — the kernel-input record
/// the production `walk_writes` consumes.
pub use benten_ivm::KernelInput;

/// Re-export of [`benten_ivm::KernelOutput`] — the kernel-output
/// materialisation the production `walk_writes` returns.
pub use benten_ivm::KernelOutput;

/// Re-export of [`benten_ivm::TypedOutputProjection`] — View 4 + View 5
/// typed-output projection shapes.
pub use benten_ivm::TypedOutputProjection;

// =============================================================================
// Compile-time pin: production symbols that MUST exist post-G23-0a
// =============================================================================

/// Canonical view ids — stable strings the kernel routes on. Family C
/// constructs `SubgraphSpec` instances keyed by these ids; the kernel
/// dispatches to the matching hand-written inner per `Algorithm::register_subgraph`.
pub const CANONICAL_VIEW_IDS: &[&str] = &[
    "capability_grants",
    "event_dispatch",
    "content_listing",
    "governance_inheritance",
    "version_current",
];

/// Module rustdoc surface name pin — the G23-0a architecture sketch lives
/// at `benten_ivm::subgraph_spec` per `.addl/phase-4-foundation/00-implementation-plan.md`
/// §3 G23-0a + the consumer-visible re-export `benten_ivm::SubgraphSpec`.
pub const EXPECTED_MODULE_PATH: &str = "benten_ivm::subgraph_spec";

/// Strategy variant set EXPECTED post-G23-0a:
/// **`{ A, B, Reserved }`** (rename of `C` → `Reserved` per arch-r1-14).
/// Family C's `strategy_c_renamed_to_reserved_grep_assert` pin asserts
/// the rename is COMPLETE: no `Strategy::C` references remain in source.
pub const EXPECTED_STRATEGY_VARIANTS: &[&str] = &["A", "B", "Reserved"];

// =============================================================================
// `CanarySubgraphSpec` — thin convenience wrapper consumers already import
// =============================================================================

/// Thin convenience wrapper Family C consumers construct + then pass to
/// [`register_and_walk_to_completion`]. The wrapper records the view-id,
/// canonical-classification flag, typed-output declaration, and a
/// self-reference flag; conversion to production [`SubgraphSpec`]
/// happens inside the helper.
///
/// **Stability commitment to Family C consumers:** the field set below
/// is the canary contract. Subsequent iterations MAY add fields
/// (additive change) but MUST NOT remove or rename.
#[derive(Debug, Clone)]
pub struct CanarySubgraphSpec {
    /// Stable view id; canonical view ids route to fast-path classification,
    /// user-defined ids route to generic kernel.
    pub view_id: String,
    /// Whether `view_id` is one of `CANONICAL_VIEW_IDS`. Computed at
    /// construction. The kernel uses this to classify Strategy::A
    /// (canonical fast-path) vs Strategy::B (generic generic-kernel walk).
    pub is_canonical: bool,
    /// Per-mat-r1-13: subgraph-shaped views MUST NOT reference themselves.
    /// `register_subgraph` rejects at register-time when this flag is set.
    pub self_referential: bool,
    /// Per-mat-r1-1: View 4 (governance_inheritance) + View 5
    /// (version_current) carry a typed-output projection shape — distinct
    /// from the row-keyed Cids output of Views 1/2/3.
    pub typed_output_projection: Option<TypedOutputProjection>,
    /// Optional per-update budget. `None` ⇒ unbounded. Used by tests
    /// that exercise budget trip / stale recovery.
    pub budget: Option<u64>,
    /// Override label pattern. For `content_listing` (the only canonical
    /// view that honors a non-hardcoded label) Family C consumers may
    /// override `"post"` with a different label; for the four
    /// hardcoded-label canonical views this MUST be the canonical
    /// hardcoded label (or `None`, which the helper resolves to the
    /// hardcoded value).
    pub label_override: Option<String>,
}

impl CanarySubgraphSpec {
    /// Build the canary spec for one of the 5 canonical view ids.
    /// Panics on unknown id (Family C tests pass known canonical ids).
    pub fn for_canonical_view(view_id: &str) -> Self {
        assert!(
            CANONICAL_VIEW_IDS.contains(&view_id),
            "canary fixture only knows the 5 canonical view ids; got `{view_id}`. \
             User-defined-id fixtures use `Self::for_user_view` instead."
        );
        let typed_output_projection = match view_id {
            "governance_inheritance" => Some(TypedOutputProjection::Rules),
            "version_current" => Some(TypedOutputProjection::Current),
            _ => None,
        };
        Self {
            view_id: view_id.to_string(),
            is_canonical: true,
            self_referential: false,
            typed_output_projection,
            budget: None,
            label_override: None,
        }
    }

    /// Build the canary spec for a user-defined view id (Strategy::B
    /// generic-kernel path).
    pub fn for_user_view(view_id: impl Into<String>) -> Self {
        let view_id = view_id.into();
        assert!(
            !CANONICAL_VIEW_IDS.contains(&view_id.as_str()),
            "for_user_view rejects canonical view ids; use for_canonical_view. \
             Got `{view_id}`."
        );
        Self {
            view_id,
            is_canonical: false,
            self_referential: false,
            typed_output_projection: None,
            budget: None,
            label_override: None,
        }
    }

    /// Mark this spec as self-referential — the register-time guard
    /// (mat-r1-13) rejects it. Used by Family B's
    /// `subgraph_shaped_view_self_reference_rejected_at_register_time`
    /// pin.
    #[must_use]
    pub fn with_self_reference(mut self) -> Self {
        self.self_referential = true;
        self
    }

    /// Attach a per-update budget. Used by budget-trip / stale recovery
    /// pins.
    #[must_use]
    pub fn with_budget(mut self, budget: u64) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Override the label pattern for `content_listing` (the canonical
    /// view that accepts an arbitrary label) or supply the canonical
    /// hardcoded label for the four hardcoded-label canonical views.
    #[must_use]
    pub fn with_label_override(mut self, label: impl Into<String>) -> Self {
        self.label_override = Some(label.into());
        self
    }

    /// Convert this canary spec into a production
    /// [`benten_ivm::SubgraphSpec`] suitable for
    /// `Algorithm::register_subgraph`.
    pub fn into_production(self) -> Result<SubgraphSpec, String> {
        if production_is_canonical_view_id(self.view_id.as_str()) {
            let mut spec = SubgraphSpec::for_canonical_view(self.view_id.as_str())?;
            if let Some(label) = self.label_override {
                // Canonical lane — only `content_listing` accepts an
                // override. For the other 4 canonical views the helper
                // resolves to the hardcoded label so passing a matching
                // label is a no-op; passing a mismatching label is
                // rejected at register-time by `register_subgraph`.
                spec = spec.with_label_pattern(LabelPattern::exact(label));
            }
            if self.self_referential {
                spec = spec.with_self_reference();
            }
            if let Some(b) = self.budget {
                spec = spec.with_budget(Some(b));
            }
            Ok(spec)
        } else {
            // User-defined view id. Default label = "post" (most common
            // label in test corpora); consumers can override.
            let label = self.label_override.unwrap_or_else(|| "post".to_string());
            let mut spec = SubgraphSpec::user_view(self.view_id, LabelPattern::exact(label))?;
            if self.self_referential {
                spec = spec.with_self_reference();
            }
            if let Some(b) = self.budget {
                spec = spec.with_budget(Some(b));
            }
            Ok(spec)
        }
    }
}

// =============================================================================
// End-to-end helpers — register + walk to completion (PRODUCTION-WIRED)
// =============================================================================

/// End-to-end helper Family B + Family C consumers invoke to drive the
/// kernel with a `CanarySubgraphSpec` + a write sequence. Wired to the
/// production [`Algorithm::register_subgraph`] + `walk_writes` surface
/// at G23-0a/G23-0b.
///
/// Returns `Ok(KernelOutput)` on a successful register + walk; returns
/// `Err(...)` carrying a stable string when registration or walk surfaces
/// an error. Consumers that expect specific error variants (self-reference
/// rejection, typed-output-projection mismatch) use
/// [`register_subgraph_returning_algorithm_error`] which preserves the
/// typed error.
pub fn register_and_walk_to_completion(
    spec: &CanarySubgraphSpec,
    writes: &[KernelInput],
) -> Result<KernelOutput, String> {
    let prod_spec = spec
        .clone()
        .into_production()
        .map_err(|e| format!("CanarySubgraphSpec::into_production: {e}"))?;
    let mut view = Algorithm::register_subgraph(prod_spec)
        .map_err(|e| format!("Algorithm::register_subgraph: {e}"))?;
    view.walk_writes(writes)
        .map_err(|e| format!("AlgorithmBView::walk_writes: {e}"))
}

/// Variant of [`register_and_walk_to_completion`] that returns the typed
/// [`AlgorithmError`] from `Algorithm::register_subgraph` directly. Used
/// by Family B pins that match on specific error variants (e.g.
/// `SelfReferentialSubgraphRejected`).
///
/// On successful registration this returns the registered
/// `AlgorithmBView` ready for callers to drive directly via
/// `walk_writes` + `materialize`.
pub fn register_subgraph_returning_algorithm_error(
    spec: &CanarySubgraphSpec,
) -> Result<benten_ivm::AlgorithmBView, AlgorithmError> {
    let prod_spec = spec
        .clone()
        .into_production()
        .expect("into_production rejects only at canonical-id+user-id boundary; canary tests stay inside contract");
    Algorithm::register_subgraph(prod_spec)
}

/// Walk a registered view through a sequence of [`KernelInput`] records;
/// surfaces the typed `ViewError` from `walk_writes`. Used by budget-trip
/// pins that match on `ViewError::BudgetExceeded`.
pub fn walk_writes_returning_view_error(
    view: &mut benten_ivm::AlgorithmBView,
    writes: &[KernelInput],
) -> Result<KernelOutput, ViewError> {
    view.walk_writes(writes)
}

/// Run the SAME write sequence through TWO independent registrations of
/// the SAME canonical view — the LEFT side uses the G23-0a
/// `Algorithm::register_subgraph(SubgraphSpec)` API; the RIGHT side uses
/// the G15-A `Algorithm::register(view_id, label_pattern, projection)`
/// API. Family C's round-trip pins use this to assert post-G23-0b
/// generalization preserves the wrapper-construction shape — i.e., the
/// G23-0a SubgraphSpec input shape compiles to the SAME `AlgorithmBView`
/// wrapper + the SAME inner-kernel handle that the G15-A triple input
/// shape produces.
///
/// **What this baseline DOES NOT prove (per G23-0b mr-1):** both paths
/// route to the same wrapper construction code path + the same inner
/// kernel for canonical view ids, so byte-equivalence at the wrapper's
/// `walk_observable` is established by construction-identity, not by
/// independent inner-kernel-read assertions. A future regression to the
/// inner kernel's `read` emission shape would NOT surface here. The
/// inner-kernel-read equivalence arm lands at G24-A when the
/// materializer pipeline consumes inner-kernel output (see RED-PHASE pin
/// `inner_kernel_read_equivalence_post_subgraph_spec_round_trip.rs`).
pub fn algorithm_register_baseline_via_g15a_path(
    canary: &CanarySubgraphSpec,
    writes: &[KernelInput],
) -> Result<KernelOutput, String> {
    let label = if let Some(l) = &canary.label_override {
        LabelPattern::exact(l.clone())
    } else if canary.view_id == "content_listing" {
        LabelPattern::exact("post")
    } else {
        // The 4 hardcoded-label canonical views: surface the canonical
        // label so `Algorithm::register` accepts it.
        let hardcoded = benten_ivm::algorithm_b::hardcoded_label_for_id(&canary.view_id)
            .ok_or_else(|| format!("no hardcoded label for `{}`", canary.view_id))?;
        LabelPattern::exact(hardcoded)
    };
    let mut view = match canary.budget {
        Some(b) => {
            Algorithm::register_with_budget(&canary.view_id, label, Projection::all_props(), b)
        }
        None => Algorithm::register(&canary.view_id, label, Projection::all_props()),
    }
    .map_err(|e| format!("Algorithm::register (G15-A baseline): {e}"))?;
    view.walk_writes(writes)
        .map_err(|e| format!("baseline walk_writes: {e}"))
}

/// Assertion helper Family C round-trip pins call to verify the
/// SubgraphSpec input path constructs an equivalent wrapper to the G15-A
/// `Algorithm::register` triple input path for canonical view ids. The
/// RIGHT-hand side is computed via
/// [`algorithm_register_baseline_via_g15a_path`] (the G15-A
/// `Algorithm::register` API path); the LEFT-hand side is the G23-0a
/// `Algorithm::register_subgraph(SubgraphSpec)` path.
///
/// **Substance per G23-0b mr-1:** what this asserts is *wrapper-
/// construction-equivalence* (the two API surfaces produce the same
/// `AlgorithmBView` wrapper + same inner-kernel handle) — NOT
/// inner-kernel-read byte-equivalence. The wrapper's `walk_observable`
/// is the observable; the inner kernel's `read` emission shape is NOT
/// exercised here. Inner-kernel-read equivalence is asserted at G24-A
/// when the materializer pipeline wires the inner-read seam (RED-PHASE
/// pin: `inner_kernel_read_equivalence_post_subgraph_spec_round_trip.rs`).
///
/// Drift between the two API surfaces (e.g., SubgraphSpec input shape
/// diverging from G15-A triple input shape under canonical view ids)
/// surfaces as a byte-inequality at the wrapper's `walk_observable`.
pub fn assert_subgraph_spec_path_construction_equivalent_to_g15a_register_path(
    spec: &CanarySubgraphSpec,
    writes: &[KernelInput],
    _expected_handwritten_output_placeholder: &KernelOutput,
) {
    let actual = register_and_walk_to_completion(spec, writes).unwrap_or_else(|e| {
        panic!(
            "register_and_walk_to_completion failed for `{}`: {e}",
            spec.view_id
        )
    });
    let baseline = algorithm_register_baseline_via_g15a_path(spec, writes).unwrap_or_else(|e| {
        panic!(
            "algorithm_register_baseline_via_g15a_path failed for `{}`: {e}",
            spec.view_id
        )
    });
    assert_eq!(
        actual, baseline,
        "subgraph-shaped view output must match the hand-written G15-A \
         baseline for the same write sequence — drift observed between \
         generalized kernel + canonical view `{}`.\nLEFT (register_subgraph): {:?}\nRIGHT (register G15-A): {:?}",
        spec.view_id, actual, baseline
    );
}

// =============================================================================
// Compile-time signature pin — helpers stable across iterations
// =============================================================================

#[allow(dead_code, reason = "compile-time signature pin")]
fn _assert_canary_signatures_compile() {
    let _: fn(&str) -> CanarySubgraphSpec = CanarySubgraphSpec::for_canonical_view;
    let _: fn(&CanarySubgraphSpec, &[KernelInput]) -> Result<KernelOutput, String> =
        register_and_walk_to_completion;
    let _: fn(&CanarySubgraphSpec, &[KernelInput], &KernelOutput) =
        assert_subgraph_spec_path_construction_equivalent_to_g15a_register_path;
}
