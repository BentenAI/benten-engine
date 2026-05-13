//! Shared test fixtures for the G23-0a IVM-subgraph generalization kernel —
//! **Family B canary surface** (R3 RED-PHASE).
//!
//! ## Purpose (Family B → Family C dependency)
//!
//! Family B (G23-0a — generalized Algorithm B kernel that consumes a
//! `SubgraphSpec` as view definition) owns this canary shape. Family C
//! (G23-0b — re-express the 5 canonical hand-written views as
//! `SubgraphSpec` consumers) imports these helpers from its 5 canonical-
//! view round-trip test files + the proptest equivalence pin.
//!
//! ## Why a NEW file (not an extension of `common.rs`)
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
//! G23-0a implementation (e.g. returning an empty view set, panicking,
//! or quietly succeeding with stub bytes) would FAIL each consumer pin.
//!
//! ## Stability commitment
//!
//! The signatures in this file are the consumer-visible canary surface
//! Family C imports. Subsequent R5 G23-0a iterations MUST preserve:
//! - `SubgraphSpec::for_canonical_view(view_id)` — 5 canonical-view
//!   constructors keyed by stable view id string.
//! - `KernelInput` / `KernelOutput` carrier types.
//! - `register_and_walk_to_completion` end-to-end helper.

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

// =============================================================================
// Compile-time pin: production symbols that MUST exist post-G23-0a
// =============================================================================
//
// The constants below are stringly-typed pins — at R5 G23-0a landing time
// they get replaced with actual `use` imports of the production surface.
// Until then they document the consumer-visible API contract Family C
// imports MUST be kept stable across.

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
// `SubgraphSpec` canary shape — Family C consumes
// =============================================================================

/// Canary shape mirroring the production `benten_ivm::SubgraphSpec` that
/// lands at R5 G23-0a. The kernel consumes one of these as its view
/// definition (in lieu of the G15-A `(view_id, label_pattern, projection)`
/// triple that becomes a special case under generalization).
///
/// **Stability commitment to Family C consumers:** the field set below is
/// the canary contract. R5 implementer MAY add fields (additive change) —
/// MUST NOT remove or rename.
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
    /// from `Projection::AllProps` (which Family C's
    /// `projection_all_props_placeholder_removed_no_remaining_references`
    /// pin verifies is REMOVED post-G23-0b).
    pub typed_output_projection: Option<TypedOutputProjection>,
}

/// Typed-output projection shapes that View 4 + View 5 produce. The R5
/// implementer wires these to the actual view-output discriminators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedOutputProjection {
    /// View 4 (governance_inheritance) emits `ViewResult::Rules(...)` —
    /// rule sets, not Cids.
    Rules,
    /// View 5 (version_current) emits `ViewResult::Current(Option<Cid>)`
    /// — a single optional pointer, not a Cid list.
    Current,
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
}

// =============================================================================
// KernelInput / KernelOutput — canary I/O shapes
// =============================================================================

/// Kernel-input record — a single write the kernel consumes. The canary
/// shape mirrors `benten_graph::ChangeEvent` minus the cross-crate type
/// dependency Family C's round-trip tests don't need.
#[derive(Debug, Clone)]
pub struct KernelInput {
    pub label: String,
    pub created_at: i64,
    pub disambiguator: u64,
}

impl KernelInput {
    pub fn new(label: impl Into<String>, created_at: i64, disambiguator: u64) -> Self {
        Self {
            label: label.into(),
            created_at,
            disambiguator,
        }
    }
}

/// Kernel-output materialisation — the view-result observed post-walk.
/// Canary shape isolates Family C consumers from R5-implementer choice
/// of internal materialisation buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelOutput {
    /// Row-set output (Views 1/2/3 + user-defined views). Sorted byte-
    /// representation of the materialised Cid set.
    Rows(Vec<u8>),
    /// Rule-set output (View 4, governance_inheritance). Canonical bytes
    /// of the rule snapshot.
    Rules(Vec<u8>),
    /// Current-pointer output (View 5, version_current). `None` when no
    /// CURRENT pointer exists; `Some(bytes)` for the current Cid.
    Current(Option<Vec<u8>>),
}

// =============================================================================
// End-to-end helper — register + walk to completion
// =============================================================================

/// End-to-end helper Family B + Family C consumers invoke to drive the
/// kernel with a `CanarySubgraphSpec` + a write sequence.
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
/// same write sequence. R5 implementer wires the right-hand side to the
/// pre-generalization baseline (e.g. `ContentListingView::new("post")`
/// fed the same events) so drift is observable.
///
/// **RED-PHASE:** asserts the call shape only — body is a stable failure
/// message Family C consumers `#[ignore]` until G23-0b ships.
pub fn assert_round_trip_equivalent_to_handwritten(
    spec: &CanarySubgraphSpec,
    writes: &[KernelInput],
    expected_handwritten_output: &KernelOutput,
) {
    let actual = register_and_walk_to_completion(spec, writes);
    match actual {
        Ok(output) => assert_eq!(
            &output, expected_handwritten_output,
            "subgraph-shaped view output must match the hand-written \
             baseline for the same write sequence — drift observed \
             between generalized kernel + canonical view `{}`",
            spec.view_id
        ),
        Err(e) => panic!(
            "register_and_walk_to_completion failed for `{}`: {e}",
            spec.view_id
        ),
    }
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
