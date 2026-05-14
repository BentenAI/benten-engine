//! IVM-subgraph generalization kernel input shape — Phase-4-Foundation G23-0a.
//!
//! ## Architecture sketch — load-bearing for G23-0b consumers
//!
//! G23-0a generalizes the [`crate::algorithm_b::AlgorithmBView`] kernel to
//! consume a [`SubgraphSpec`] as view definition in lieu of the G15-A
//! `(view_id, label_pattern, projection)` triple. The triple becomes a
//! *special case* of [`SubgraphSpec`]: each canonical view id has a
//! constructor on [`SubgraphSpec`] that bakes the matching hardcoded label
//! into the spec; user-defined views construct via
//! [`SubgraphSpec::user_view`] with a caller-supplied label pattern.
//!
//! ### Why a NEW module (not an extension of `algorithm_b.rs`)
//!
//! `algorithm_b.rs` is the G15-A generalized-kernel implementation surface
//! — `Algorithm::register` + `LabelPattern` + `Projection` + canonical
//! dispatch live there. The NEW [`SubgraphSpec`] type is the **input
//! shape** for the kernel — a schema-shaped view definition. Splitting it
//! out keeps the kernel's internals (label-matching, budget tracking,
//! per-event dispatch) separate from the *registration surface* G23-0b's
//! 5 canonical-view round-trip pins + Family C's proptest equivalence
//! pin consume.
//!
//! ### D-4F-2: materializer view IS an IVM view
//!
//! Per Ben's D-4F-2 ratification at Phase-4-Foundation R1 triage, the
//! materializer pipeline (G23-B wave-5) registers its views *through this
//! same Algorithm B kernel*. A SubgraphSpec is the universal kernel-input
//! shape: canonical IVM views, user-defined views, and materializer-view
//! schema instances all flow through `Algorithm::register_subgraph(spec)`.
//! The materializer's `Renderer` trait is the host-side output transform;
//! the kernel itself doesn't know it's serving a materializer.
//!
//! ### Self-reference rejection (mat-r1-13)
//!
//! [`SubgraphSpec`] carries an explicit `self_referential` flag that the
//! register-time guard at [`crate::algorithm_b::AlgorithmBView::register_subgraph`]
//! inspects BEFORE any walk. Self-referential specs are rejected at
//! register time with [`crate::algorithm_b::AlgorithmError::SelfReferentialSubgraphRejected`]
//! — fail-fast semantics per mat-r1-13: rejection MUST surface before any
//! kernel input walks (no partial materialisation, no walk-time-only
//! check). The flag is the canary-level surface; a future richer
//! representation (named referenced sub-views) lifts into the same field
//! shape without breaking the SubgraphSpec contract.
//!
//! ### `Strategy::B` invariant
//!
//! Per CLAUDE.md baked-in #2 + arch-r1-14: `Strategy::B` IS the
//! generalized Algorithm B. G23-0a does NOT mint a `Strategy::Generalized`
//! or `Strategy::Subgraph` variant — the SubgraphSpec input shape lives
//! *under* the existing `Strategy::B` classification. The internal
//! [`crate::algorithm_b::dispatch_for`] router still classifies canonical
//! ids as `Strategy::A` (the canonical fast-path marker) and user-defined
//! ids as `Strategy::B`, but the engine-boundary [`crate::View::strategy`]
//! for either lane returns `Strategy::B` (the wrapper IS Strategy::B).
//!
//! See `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-0a.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::algorithm_b::{LabelPattern, Projection};

/// Canonical view ids — stable strings the kernel routes on. Mirrors
/// [`crate::algorithm_b`]'s internal canonical-id set; re-stated here so
/// the [`SubgraphSpec::for_canonical_view`] constructor's allowed-id
/// contract is local to this module.
pub const CANONICAL_VIEW_IDS: &[&str] = &[
    "capability_grants",
    "event_dispatch",
    "content_listing",
    "governance_inheritance",
    "version_current",
];

/// Typed-output projection shapes that View 4 (governance_inheritance) +
/// View 5 (version_current) carry. Per `mat-r1-1` the typed-output shape
/// supersedes the pre-G23-0b identity-projection placeholder (the
/// `Projection` placeholder variant was removed at G23-0b per CRATES-DEEP-DIVE
/// §4; Family C's `projection_all_props_placeholder_removed_no_remaining_references`
/// pin guards the removal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedOutputProjection {
    /// View 4 emits rule sets (a map of governance-rules, not Cids).
    Rules,
    /// View 5 emits a single optional Cid pointer (the CURRENT pointer).
    Current,
}

/// Schema-shaped view definition the generalized Algorithm B kernel
/// consumes (G23-0a).
///
/// **Stability commitment to G23-0b consumers (canary):** the field set
/// here is the canary contract Family B + Family C round-trip pins
/// import via [`crate::Algorithm::register_subgraph`]. Subsequent
/// iterations MAY add fields (additive) but MUST NOT remove or rename.
///
/// **D-4F-NEW-TYPED-FIELD-NODE-VOCAB readiness:** the field shapes here
/// mirror the typed-field-Node vocabulary the schema language lands at
/// G23-A (8 labels / 5 labeled edges / 8 scalars / 4 properties per Ben's
/// post-R1-triage ratification). A future SubgraphSpec → typed-field-
/// Node lowering pass converts this struct into Node form on persist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubgraphSpec {
    /// Stable view id. Canonical view ids route to fast-path classification
    /// (`Strategy::A` per [`crate::algorithm_b::dispatch_for`] — INTERNAL);
    /// user-defined ids route to the generic-kernel path
    /// (`Strategy::B` per the same router). The engine-boundary strategy
    /// for either lane is `Strategy::B`.
    pub view_id: String,
    /// Label-pattern selector — `LabelPattern::Exact` for canonical views
    /// (the canonical kernel's hardcoded label is enforced as a fail-loud
    /// invariant per `r6-r3-ivm-1`); user views may also use
    /// `LabelPattern::AnchorPrefix`.
    pub label_pattern: LabelPattern,
    /// Output projection. G23-0b ships the identity projection only;
    /// View 4 Rules + View 5 Current typed-output shapes are declared
    /// separately via [`SubgraphSpec::typed_output_projection`] per
    /// mat-r1-1.
    pub projection: Projection,
    /// Typed-output projection for views 4 + 5 (Rules / Current).
    /// `None` for views 1/2/3 + user-defined views (which emit row-set
    /// output via the row-keyed identity-projection path). Wired at
    /// `register_subgraph` time to the inner kernel's `ViewResult`
    /// variant — a declared typed-output projection that does NOT match
    /// the inner kernel's actual variant is a programmer error (caught
    /// at materialisation via fail-loud per g23-0a-mr-3).
    pub typed_output_projection: Option<TypedOutputProjection>,
    /// Self-reference flag — set when the SubgraphSpec body would
    /// reference itself transitively. Inspected at register-time per
    /// mat-r1-13 fail-fast semantics. A future richer representation
    /// (an explicit `referenced_views: Vec<SubgraphSpec>` list with
    /// graph-walk cycle detection) lifts under this flag without
    /// breaking the canary contract.
    pub self_referential: bool,
    /// Optional per-update budget. `None` ⇒ unbounded (equivalent to
    /// `u64::MAX`); `Some(n)` ⇒ per-update cap of `n` matching writes
    /// before stale.
    pub budget: Option<u64>,
}

impl SubgraphSpec {
    /// Build a SubgraphSpec for one of the 5 canonical view ids. Bakes
    /// the matching hardcoded label into the spec.
    ///
    /// # Errors
    ///
    /// Returns `Err(...)` when `view_id` is not one of [`CANONICAL_VIEW_IDS`].
    /// Use [`SubgraphSpec::user_view`] for user-defined ids.
    pub fn for_canonical_view(view_id: &str) -> Result<Self, String> {
        if !CANONICAL_VIEW_IDS.contains(&view_id) {
            return Err(alloc::format!(
                "SubgraphSpec::for_canonical_view: `{view_id}` is not a canonical view id. \
                 Canonical ids: {CANONICAL_VIEW_IDS:?}. Use SubgraphSpec::user_view for \
                 user-defined ids."
            ));
        }
        // Bake the canonical hardcoded label. `content_listing` honors a
        // supplied label (defaults to "post" if not overridden); the other
        // four use a hardcoded fixed label per
        // `crate::algorithm_b::hardcoded_label_for_id`.
        let label_pattern = match view_id {
            "capability_grants" => LabelPattern::exact("system:CapabilityGrant"),
            "event_dispatch" => LabelPattern::exact("system:EventDispatch"),
            "content_listing" => LabelPattern::exact("post"),
            "governance_inheritance" => LabelPattern::exact("system:GovernanceInheritance"),
            "version_current" => LabelPattern::exact("NEXT_VERSION"),
            // Unreachable per CANONICAL_VIEW_IDS contains-check above; the
            // contains-check is the authoritative gate.
            other => {
                return Err(alloc::format!(
                    "SubgraphSpec::for_canonical_view: unknown canonical id `{other}`"
                ));
            }
        };
        let typed_output_projection = match view_id {
            "governance_inheritance" => Some(TypedOutputProjection::Rules),
            "version_current" => Some(TypedOutputProjection::Current),
            _ => None,
        };
        Ok(Self {
            view_id: view_id.to_string(),
            label_pattern,
            projection: Projection::all_props(),
            typed_output_projection,
            self_referential: false,
            budget: None,
        })
    }

    /// Build a SubgraphSpec for a user-defined view id with the supplied
    /// label pattern. Rejects canonical view ids — use
    /// [`SubgraphSpec::for_canonical_view`].
    ///
    /// # Errors
    ///
    /// Returns `Err(...)` when `view_id` matches a canonical view id.
    pub fn user_view(
        view_id: impl Into<String>,
        label_pattern: LabelPattern,
    ) -> Result<Self, String> {
        let view_id = view_id.into();
        if CANONICAL_VIEW_IDS.contains(&view_id.as_str()) {
            return Err(alloc::format!(
                "SubgraphSpec::user_view: `{view_id}` is a canonical view id. Use \
                 SubgraphSpec::for_canonical_view for canonical ids; user-view \
                 constructor rejects them so callers don't accidentally shadow \
                 the hardcoded label."
            ));
        }
        Ok(Self {
            view_id,
            label_pattern,
            projection: Projection::all_props(),
            typed_output_projection: None,
            self_referential: false,
            budget: None,
        })
    }

    /// Mark this spec as self-referential — `Algorithm::register_subgraph`
    /// rejects it at register-time per mat-r1-13 fail-fast.
    #[must_use]
    pub fn with_self_reference(mut self) -> Self {
        self.self_referential = true;
        self
    }

    /// Set the per-update budget. `None` ⇒ unbounded (saturating
    /// arithmetic absorbs all matching writes); `Some(n)` ⇒ cap.
    #[must_use]
    pub fn with_budget(mut self, budget: Option<u64>) -> Self {
        self.budget = budget;
        self
    }

    /// Override the label pattern. Used by `content_listing` callers that
    /// want a non-`"post"` label (the constructor defaults to `"post"`).
    #[must_use]
    pub fn with_label_pattern(mut self, label_pattern: LabelPattern) -> Self {
        self.label_pattern = label_pattern;
        self
    }

    /// Is this spec keyed on a canonical view id?
    #[must_use]
    pub fn is_canonical(&self) -> bool {
        CANONICAL_VIEW_IDS.contains(&self.view_id.as_str())
    }
}

/// Kernel-input record — a single write the kernel consumes via
/// `walk_writes`. Mirrors `benten_graph::ChangeEvent` minus the cross-
/// crate dependency Family B/C consumer pins don't need. The R5
/// implementer's `walk_writes` shape converts `KernelInput` into a
/// `ChangeEvent` internally before feeding the per-event `View::update`
/// path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelInput {
    /// Node label — drives label-pattern matching against the spec's
    /// `label_pattern`.
    pub label: String,
    /// Logical creation timestamp (used by `content_listing` ordering).
    pub created_at: i64,
    /// Disambiguator — ensures distinct CIDs for otherwise-identical
    /// inputs in test corpora.
    pub disambiguator: u64,
}

impl KernelInput {
    /// Construct a new kernel input.
    #[must_use]
    pub fn new(label: impl Into<String>, created_at: i64, disambiguator: u64) -> Self {
        Self {
            label: label.into(),
            created_at,
            disambiguator,
        }
    }
}

/// Kernel-output materialisation — the view-result observed post-walk.
/// Three discriminants matching the three [`crate::view::ViewResult`]
/// shapes the per-view inner kernels can emit:
///
/// - `Rows` — row-set output (Views 1/2/3 + user-defined views).
///   `Vec<u8>` is the canonical serialisation of the materialised CID
///   set (sorted lexicographically by CID bytes for determinism).
/// - `Rules` — rule-set output (View 4, governance_inheritance).
/// - `Current` — current-pointer output (View 5, version_current). `None`
///   when no CURRENT pointer exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelOutput {
    /// Row-set output. Canonical sorted bytes of the materialised CID set.
    Rows(Vec<u8>),
    /// Rule-set output (View 4). Canonical bytes of the rule snapshot.
    Rules(Vec<u8>),
    /// Current-pointer output (View 5). `None` when no CURRENT pointer.
    Current(Option<Vec<u8>>),
}
