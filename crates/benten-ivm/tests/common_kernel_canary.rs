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
//! `tests/common.rs` is the established drift-detector helper module —
//! large, scenario-specific, tied to the G15-A `Algorithm::register`
//! shape. The G23-0a generalization introduces a NEW input shape
//! (`SubgraphSpec` — schema-shaped view definition, NOT
//! `(label_pattern, projection)` triple) that requires its own canary
//! file so Family C consumers import a stable, isolated surface.
//!
//! ## GREEN-PHASE (R5 G23-0a landed)
//!
//! These fixtures now thread through `benten_ivm::subgraph_spec::SubgraphSpec` +
//! `benten_ivm::Algorithm::register_subgraph` (production symbols shipped
//! in R5 G23-0a canary commit). Consumer pins un-ignore per pim-12 §3.6e.
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
/// **GREEN-PHASE (R5 G23-0a landed):** threads the canary spec into the
/// production `benten_ivm::SubgraphSpec` + drives the registered view
/// through `Algorithm::walk_writes`. Consumer pins un-ignore per pim-12
/// §3.6e.
pub fn register_and_walk_to_completion(
    spec: &CanarySubgraphSpec,
    writes: &[KernelInput],
) -> Result<KernelOutput, String> {
    use benten_ivm::algorithm_b::LabelPattern;
    use benten_ivm::{Algorithm, SubgraphSpec};

    // Build the production SubgraphSpec from the canary shape.
    let prod_spec = if spec.is_canonical {
        SubgraphSpec::for_canonical_view(&spec.view_id)
            .map_err(|e| format!("SubgraphSpec::for_canonical_view: {e}"))?
    } else {
        // User-defined views default to LabelPattern::Exact("post") for
        // the canary contract — matches the canary's KernelInput label
        // convention. Family C round-trip pins can override via
        // SubgraphSpec::with_label_pattern at the construction site.
        SubgraphSpec::user_view(spec.view_id.clone(), LabelPattern::exact("post"))
            .map_err(|e| format!("SubgraphSpec::user_view: {e}"))?
    };
    let prod_spec = if spec.self_referential {
        prod_spec.with_self_reference()
    } else {
        prod_spec
    };

    // Convert canary inputs to production inputs.
    let prod_writes: Vec<benten_ivm::KernelInput> = writes
        .iter()
        .map(|w| benten_ivm::KernelInput::new(w.label.clone(), w.created_at, w.disambiguator))
        .collect();

    let mut view = Algorithm::register_subgraph(prod_spec).map_err(|e| format!("register: {e}"))?;
    let prod_output = view
        .walk_writes(&prod_writes)
        .map_err(|e| format!("walk: {e}"))?;
    // Convert production KernelOutput → canary KernelOutput (same shape;
    // re-wrap to keep canary surface isolated from cross-crate type
    // dependency).
    Ok(match prod_output {
        benten_ivm::KernelOutput::Rows(bytes) => KernelOutput::Rows(bytes),
        benten_ivm::KernelOutput::Rules(bytes) => KernelOutput::Rules(bytes),
        benten_ivm::KernelOutput::Current(opt) => KernelOutput::Current(opt),
    })
}

/// Assertion helper Family C round-trip pins call to verify the canary
/// output matches the corresponding hand-written view's output for the
/// same write sequence. The RIGHT-hand side is computed via
/// [`handwritten_baseline_via_register_g15a`] (the G15-A
/// `Algorithm::register` API path which drives the SAME hand-written
/// inner kernel for canonical view ids); the LEFT-hand side is the
/// G23-0a `Algorithm::register_subgraph(SubgraphSpec)` path.
///
/// Drift between the two paths surfaces as a byte-inequality assertion
/// failure.
pub fn assert_round_trip_equivalent_to_handwritten(
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
    let baseline = handwritten_baseline_via_register_g15a(spec, writes).unwrap_or_else(|e| {
        panic!(
            "handwritten_baseline_via_register_g15a failed for `{}`: {e}",
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
        assert_round_trip_equivalent_to_handwritten;
}
