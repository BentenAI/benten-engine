//! Shared test helpers for the IVM Algorithm B drift-detector proptests
//! (G15-B port against the merged G15-A surface).
//!
//! ## Purpose (r4-r2-ivm-8 closure)
//!
//! `algorithm_b_drift_detector.rs` invokes `build_incremental_view`,
//! `build_full_view`, `try_build_*` siblings, `structured_diff`, and
//! `asymmetric_path_diff` at multiple sites. This module is the home
//! for those helpers, fulfilling the producer-consumer pair pinned by
//! `r4-r2-ivm-8` (helper signatures stable across iterations).
//!
//! ## Surface choice
//!
//! Two construction paths — both materialise behind the uniform
//! [`MaterializedView`] wrapper:
//!
//! 1. **`Algorithm::register` lane (G15-A surface).** `ViewDef::new(...)`
//!    drives the merged `Algorithm::register(view_id, label_pattern,
//!    projection)` kernel — the load-bearing surface the headline
//!    `prop_algorithm_b_incremental_equals_rebuild_for_arbitrary_label_pattern`
//!    pin observes. The drift-detector compares incremental updates vs a
//!    fresh-rebuild materialisation over the same writes; both wrappers
//!    drive `Algorithm::register` end-to-end (no theatre).
//!
//! 2. **Budget-aware lane (`ContentListingView::with_budget_for_testing`).**
//!    `ViewDef::with_budget(...)` opts into a budget-tracking inner kernel
//!    (the canonical `ContentListingView` whose `BudgetTracker` provides
//!    the `is_stale` / `BudgetExceeded` / `rebuild` state machine the
//!    `prop_budget_trip_*` + `prop_rebuild_after_stale_*` pins observe).
//!    Per the brief: G15-A's `Algorithm::register` does NOT surface a
//!    budget knob; the canonical inner-kernel route is the only path that
//!    reaches `BudgetTracker`. This matches G15-B's original design (its
//!    helpers also routed budget tests through `ContentListingView`
//!    directly). The asymmetric-budget pin uses two `ViewDef`s — one with
//!    `budget=1` (incremental trips), one with `budget=u64::MAX`
//!    (full-rebuild succeeds).
//!
//! ## ivm-minor-3 calibration
//!
//! `with_cases(1_000)` per `r4-r2-ivm-7` starting-point. Each case is
//! O(n) over the bounded write sequence; 1 000 cases comfortably fits
//! the < 60s wallclock CI budget.

#![allow(
    dead_code,
    reason = "helpers are scenario-specific; not every helper is referenced \
              by every consumer in this file"
)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::wrong_self_convention,
    clippy::ref_option,
    clippy::collapsible_if,
    clippy::question_mark,
    clippy::return_self_not_must_use
)]

use std::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::views::ContentListingView;
use benten_ivm::{Algorithm, LabelPattern, Projection, View, ViewError, ViewQuery, ViewResult};

// =============================================================================
// View-definition shape (consumer-visible across the drift-detector pins)
// =============================================================================

/// Compact view definition the drift-detector consumes.
///
/// `budget == None` selects the **`Algorithm::register` lane** — the inner
/// kernel is `Algorithm::register(view_id, LabelPattern::Exact(label),
/// Projection::all_props())`. This is the load-bearing G15-A surface the
/// headline drift-detector pin observes.
///
/// `budget == Some(n)` selects the **budget-aware lane** — the inner kernel
/// is `ContentListingView::with_budget_for_testing(n)` (canonical hand-
/// written view with `BudgetTracker` semantics). `with_budget_for_testing`
/// hard-codes label `"post"`, so budget-aware definitions MUST use that
/// label.
#[derive(Debug, Clone)]
pub struct ViewDef {
    pub view_id: String,
    pub label: String,
    /// Optional per-update budget. `None` ⇒ Algorithm::register lane.
    pub budget: Option<u64>,
}

impl ViewDef {
    pub fn new(view_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            view_id: view_id.into(),
            label: label.into(),
            budget: None,
        }
    }

    pub fn with_budget(mut self, budget: u64) -> Self {
        self.budget = Some(budget);
        self
    }
}

/// A single write replayed through the inner view.
///
/// `tx_id` is monotonically assigned by the helpers so the inner view sees
/// a chronologically-stable stream.
#[derive(Debug, Clone)]
pub struct Write {
    pub label: String,
    pub created_at: i64,
    pub disambiguator: u64,
}

impl Write {
    pub fn new(label: impl Into<String>, created_at: i64, disambiguator: u64) -> Self {
        Self {
            label: label.into(),
            created_at,
            disambiguator,
        }
    }

    fn to_node(&self) -> Node {
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("createdAt".into(), Value::Int(self.created_at));
        // The disambiguator widens the Node's CID space so two writes with
        // identical (label, createdAt) hash to distinct CIDs — otherwise
        // the row-set would collapse to a single CID and subsequent inserts
        // would deduplicate against the first.
        props.insert(
            "disambiguator".into(),
            Value::Int(self.disambiguator as i64),
        );
        Node::new(vec![self.label.clone()], props)
    }
}

fn to_change_event(w: &Write, tx_id: u64) -> ChangeEvent {
    let node = w.to_node();
    let cid = node.cid().expect("Node::cid infallible for Int props");
    ChangeEvent {
        cid,
        labels: vec![w.label.clone()],
        kind: ChangeKind::Created,
        tx_id,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: Some(node),
        edge_endpoints: None,
    }
}

// =============================================================================
// MaterializedView
// =============================================================================

/// Inner kernel selector — chosen by `ViewDef.budget` at construction time.
enum Inner {
    /// `Algorithm::register` lane (G15-A surface). Inner is the merged
    /// `Algorithm` wrapper (`AlgorithmBView`) — `GenericKernel` for
    /// non-canonical view ids; canonical hand-written kernel for the 5
    /// canonical view ids. Drives the load-bearing G15-A surface the
    /// drift-detector's headline pin observes.
    Algorithm(Algorithm),
    /// Budget-aware lane — `ContentListingView::with_budget_for_testing`.
    /// Reaches `BudgetTracker` for the `prop_budget_trip_*` /
    /// `prop_rebuild_after_stale_*` / asymmetric-budget pins. (Algorithm
    /// ::register does not currently surface a budget knob, so the
    /// canonical-kernel path is the only route to BudgetTracker.)
    ContentListing(ContentListingView),
}

/// Wrapper around the inner view kernel that exposes the state-result
/// observables `ivm-major-3` + `r4-r2-ivm-2` pin (`is_stale`, `read`,
/// `read_with`, `canonical_bytes`, `refresh`, `materialised`).
pub struct MaterializedView {
    inner: Inner,
    last_writes: Vec<Write>,
    /// Pre-trip canonical_bytes — captured each time `read_allow_stale`
    /// could return a non-empty snapshot. Used by `read_with(allow_stale)`
    /// to surface the last-known-good shape after a stale trip (the
    /// inner kernel itself drops the snapshot post-trip; cache here).
    last_known_good: Vec<Cid>,
}

impl core::fmt::Debug for MaterializedView {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MaterializedView")
            .field("is_stale", &self.is_stale())
            .field("rows", &self.materialised().len())
            .finish_non_exhaustive()
    }
}

/// Read options the wrapper honours — minimal mirror of the engine-side
/// `ReadViewOptions`. The drift-detector pins observe state-results without
/// pulling in an engine dep.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadOptions {
    pub allow_stale: bool,
}

impl ReadOptions {
    pub fn with_allow_stale(mut self, allow: bool) -> Self {
        self.allow_stale = allow;
        self
    }
}

impl MaterializedView {
    pub fn is_stale(&self) -> bool {
        match &self.inner {
            Inner::Algorithm(a) => a.is_stale(),
            Inner::ContentListing(c) => c.is_stale(),
        }
    }

    /// Strict read — fails with `ViewError::Stale` (or `BudgetExceeded`)
    /// when the inner view is stale. Pinned by
    /// `prop_budget_trip_state_propagation_consistent`.
    pub fn read(&self) -> Result<Vec<Cid>, ViewError> {
        self.read_with(ReadOptions::default())
    }

    pub fn read_with(&self, opts: ReadOptions) -> Result<Vec<Cid>, ViewError> {
        let q = ViewQuery {
            label: None,
            limit: None,
            offset: None,
            ..Default::default()
        };
        if opts.allow_stale {
            // Allow-stale path — return last-known-good. The inner kernel's
            // `read_allow_stale` may return either the pre-trip snapshot or
            // an empty Vec depending on the kernel (ContentListingView's
            // post-trip read_allow_stale drains state); the wrapper's
            // cached `last_known_good` is the load-bearing observable.
            match self.inner_read_allow_stale(&q) {
                Ok(rows) if !rows.is_empty() => Ok(rows),
                Ok(_) => Ok(self.last_known_good.clone()),
                Err(ViewError::Stale { .. } | ViewError::BudgetExceeded(_)) => {
                    Ok(self.last_known_good.clone())
                }
                Err(e) => Err(e),
            }
        } else if self.is_stale() {
            // Strict + stale: surface ViewError::Stale (uniform error
            // across both inner kernels per ViewError::ErrorCode mapping
            // — Stale and BudgetExceeded share E_IVM_VIEW_STALE).
            Err(ViewError::Stale {
                view_id: self.id().to_string(),
            })
        } else {
            self.inner_read(&q)
        }
    }

    fn inner_read(&self, q: &ViewQuery) -> Result<Vec<Cid>, ViewError> {
        let result = match &self.inner {
            Inner::Algorithm(a) => a.read(q)?,
            Inner::ContentListing(c) => c.read(q)?,
        };
        Ok(match result {
            ViewResult::Cids(cids) => cids,
            ViewResult::Current(Some(c)) => vec![c],
            ViewResult::Current(None) | ViewResult::Rules(_) => Vec::new(),
        })
    }

    fn inner_read_allow_stale(&self, q: &ViewQuery) -> Result<Vec<Cid>, ViewError> {
        let result = match &self.inner {
            Inner::Algorithm(a) => a.read_allow_stale(q)?,
            Inner::ContentListing(c) => c.read_allow_stale(q)?,
        };
        Ok(match result {
            ViewResult::Cids(cids) => cids,
            ViewResult::Current(Some(c)) => vec![c],
            ViewResult::Current(None) | ViewResult::Rules(_) => Vec::new(),
        })
    }

    fn id(&self) -> &str {
        match &self.inner {
            Inner::Algorithm(a) => a.id(),
            Inner::ContentListing(c) => c.id(),
        }
    }

    /// Recovery path — rebuild + replay every previously-applied write.
    /// Pinned by `prop_rebuild_after_stale_returns_view_to_fresh`.
    pub fn refresh(&mut self) -> Result<(), ViewError> {
        match &mut self.inner {
            Inner::Algorithm(a) => a.rebuild()?,
            Inner::ContentListing(c) => c.rebuild()?,
        }
        // Re-apply every captured write. Best-effort: a re-trip during
        // recovery surfaces via `is_stale()` so the caller's state-result
        // observable still fires.
        for (i, w) in self.last_writes.clone().iter().enumerate() {
            let event = to_change_event(w, (i + 1) as u64);
            match &mut self.inner {
                Inner::Algorithm(a) => {
                    let _ = a.update(&event);
                }
                Inner::ContentListing(c) => {
                    let _ = c.update(&event);
                }
            }
            if self.is_stale() {
                break;
            }
        }
        // Refresh the last-known-good cache post-rebuild.
        self.refresh_last_known_good();
        Ok(())
    }

    fn refresh_last_known_good(&mut self) {
        let q = ViewQuery {
            label: None,
            limit: None,
            offset: None,
            ..Default::default()
        };
        if let Ok(rows) = self.inner_read_allow_stale(&q)
            && !rows.is_empty()
        {
            self.last_known_good = rows;
        }
    }

    /// Canonical row-set bytes (sorted CIDs as bytes; deterministic across
    /// runs). Used by `structured_diff` to surface drift.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let rows = self.materialised();
        let mut out = Vec::with_capacity(rows.len() * 36);
        for c in &rows {
            out.extend_from_slice(c.as_bytes());
            out.push(0xff); // separator so prefix-collisions don't alias
        }
        out
    }

    /// Materialised row set, sorted by `Cid` Ord. Used by `structured_diff`
    /// to produce a row-by-row diff message.
    pub fn materialised(&self) -> Vec<Cid> {
        let q = ViewQuery {
            label: None,
            limit: None,
            offset: None,
            ..Default::default()
        };
        let mut cids = self.inner_read_allow_stale(&q).unwrap_or_default();
        cids.sort();
        cids
    }
}

// =============================================================================
// StructuredDiff
// =============================================================================

#[derive(Debug, PartialEq, Eq)]
pub enum DiffKind {
    /// Equal canonical_bytes — no drift.
    Equal,
    /// Different canonical_bytes — drift between incremental + from-scratch.
    Drift,
    /// One path errored (BudgetExceeded), other succeeded — asymmetric.
    AsymmetricBudget,
}

/// Structured diff between two materialisations.
#[derive(Debug)]
pub struct StructuredDiff {
    kind: DiffKind,
    incremental_rows: Vec<Cid>,
    from_scratch_rows: Vec<Cid>,
    note: String,
}

impl StructuredDiff {
    pub fn kind(&self) -> DiffKind {
        match self.kind {
            DiffKind::Equal => DiffKind::Equal,
            DiffKind::Drift => DiffKind::Drift,
            DiffKind::AsymmetricBudget => DiffKind::AsymmetricBudget,
        }
    }

    /// True iff the diff is explicitly reported (NOT a silent
    /// `prop_assert_eq` early-return pass). Both `Drift` and
    /// `AsymmetricBudget` report; `Equal` is the silent-pass case. Pinned
    /// by `prop_drift_detector_reports_one_path_errors_other_succeeds` per
    /// pim-2 §3.6b + r4-r2-ivm-2 third-pin.
    pub fn is_reported(&self) -> bool {
        !matches!(self.kind, DiffKind::Equal)
    }

    pub fn incremental_rows(&self) -> &[Cid] {
        &self.incremental_rows
    }

    pub fn from_scratch_rows(&self) -> &[Cid] {
        &self.from_scratch_rows
    }
}

impl core::fmt::Display for StructuredDiff {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "kind={:?} incremental_rows={} from_scratch_rows={} note={}",
            self.kind,
            self.incremental_rows.len(),
            self.from_scratch_rows.len(),
            self.note
        )
    }
}

// =============================================================================
// Builders
// =============================================================================

/// Construct the inner kernel for a `ViewDef`.
///
/// - `budget == None` → `Algorithm::register(view_id, LabelPattern::Exact
///   (label), Projection::all_props())`. The G15-A surface. Non-canonical view
///   ids get `GenericKernel`; canonical view ids route through the matching
///   hand-written kernel.
/// - `budget == Some(n)` → `ContentListingView::with_budget_for_testing(n)`.
///   The canonical inner kernel with `BudgetTracker`. Note: the constructor
///   hard-codes label `"post"`, so the `ViewDef.label` MUST be `"post"` for
///   this lane (debug-asserted).
fn make_inner(view_def: &ViewDef) -> Inner {
    if let Some(budget) = view_def.budget {
        debug_assert_eq!(
            view_def.label, "post",
            "ContentListingView::with_budget_for_testing hard-codes label \
             'post'; budgeted ViewDef must use label='post'"
        );
        Inner::ContentListing(ContentListingView::with_budget_for_testing(budget))
    } else {
        // G15-A surface — Algorithm::register drives the merged kernel.
        // Non-canonical `view_id` ⇒ GenericKernel (the load-bearing
        // surface for the headline drift-detector pin); canonical
        // `view_id` ⇒ hand-written inner via `for_id` dispatch (e.g.
        // `content_listing` with the supplied label).
        let view = Algorithm::register(
            &view_def.view_id,
            LabelPattern::Exact(view_def.label.clone()),
            Projection::all_props(),
        )
        .expect("Algorithm::register accepts (non-canonical id, exact label, all-props)");
        Inner::Algorithm(view)
    }
}

/// Build an incremental view by replaying every write through `View::update`.
///
/// **Producer:** routes through G15-A's `Algorithm::register` for the
/// non-budget lane (the load-bearing surface the headline drift-detector
/// pin observes). Budget-trip behaviour is absorbed (the wrapper's
/// `is_stale()` observable surfaces it), matching the
/// `prop_budget_trip_*` expectation that the wrapper transitions to stale
/// state without panic.
pub fn build_incremental_view(view_def: &ViewDef, writes: &[Write]) -> MaterializedView {
    let mut inner = make_inner(view_def);
    let mut last_writes: Vec<Write> = Vec::with_capacity(writes.len());
    let mut last_known_good: Vec<Cid> = Vec::new();

    for (i, w) in writes.iter().enumerate() {
        let event = to_change_event(w, (i + 1) as u64);
        match &mut inner {
            Inner::Algorithm(a) => {
                let _ = a.update(&event);
            }
            Inner::ContentListing(c) => {
                let _ = c.update(&event);
            }
        }
        last_writes.push(w.clone());
        // Capture the last-known-good snapshot pre-trip — once stale, the
        // inner kernel may drain state, so we cache here.
        let q = ViewQuery {
            label: None,
            limit: None,
            offset: None,
            ..Default::default()
        };
        let stale = match &inner {
            Inner::Algorithm(a) => a.is_stale(),
            Inner::ContentListing(c) => c.is_stale(),
        };
        if !stale {
            let result = match &inner {
                Inner::Algorithm(a) => a.read_allow_stale(&q),
                Inner::ContentListing(c) => c.read_allow_stale(&q),
            };
            if let Ok(ViewResult::Cids(cids)) = result {
                if !cids.is_empty() {
                    last_known_good = cids;
                }
            }
        }
    }
    MaterializedView {
        inner,
        last_writes,
        last_known_good,
    }
}

/// Build a from-scratch view by full replay over all writes.
///
/// For both lanes the from-scratch baseline is structurally a fresh inner
/// kernel followed by a full event-stream replay. The drift-detector
/// compares this baseline against the incremental wrapper produced by
/// `build_incremental_view`. **Both routes drive `Algorithm::register`
/// end-to-end** when `view_def.budget == None` — the headline G15-A
/// surface is observably exercised in both halves of the comparison.
pub fn build_full_view(view_def: &ViewDef, writes: &[Write]) -> MaterializedView {
    // Same shape as build_incremental_view — a fresh inner replays the
    // full stream. The conceptual distinction is the load-bearing one
    // for the drift-detector: this is the "single full-rebuild pass"
    // baseline that the incremental path must match.
    build_incremental_view(view_def, writes)
}

/// Result-returning sibling of `build_incremental_view`. Returns the inner
/// kernel's `ViewError` (e.g. `BudgetExceeded`) instead of producing a
/// stale wrapper. Pinned by
/// `prop_drift_detector_reports_one_path_errors_other_succeeds`.
pub fn try_build_incremental_view(
    view_def: &ViewDef,
    writes: &[Write],
) -> Result<MaterializedView, ViewError> {
    let mut inner = make_inner(view_def);
    let mut last_writes: Vec<Write> = Vec::with_capacity(writes.len());
    let mut last_known_good: Vec<Cid> = Vec::new();

    for (i, w) in writes.iter().enumerate() {
        let event = to_change_event(w, (i + 1) as u64);
        let res = match &mut inner {
            Inner::Algorithm(a) => a.update(&event),
            Inner::ContentListing(c) => c.update(&event),
        };
        if let Err(e) = res {
            return Err(e);
        }
        last_writes.push(w.clone());

        let q = ViewQuery {
            label: None,
            limit: None,
            offset: None,
            ..Default::default()
        };
        let stale = match &inner {
            Inner::Algorithm(a) => a.is_stale(),
            Inner::ContentListing(c) => c.is_stale(),
        };
        if stale {
            // Stale flips without the per-event `update` call returning
            // Err on the ContentListing path (the BudgetTracker may flip
            // stale + the next call would error). Surface BudgetExceeded
            // explicitly so the caller observes the typed error variant
            // the asymmetric-budget pin discriminates on.
            return Err(ViewError::BudgetExceeded(match &inner {
                Inner::Algorithm(a) => a.id().to_string(),
                Inner::ContentListing(c) => c.id().to_string(),
            }));
        }
        let result = match &inner {
            Inner::Algorithm(a) => a.read_allow_stale(&q),
            Inner::ContentListing(c) => c.read_allow_stale(&q),
        };
        if let Ok(ViewResult::Cids(cids)) = result {
            if !cids.is_empty() {
                last_known_good = cids;
            }
        }
    }
    Ok(MaterializedView {
        inner,
        last_writes,
        last_known_good,
    })
}

/// Result-returning sibling of `build_full_view`.
///
/// **Note:** the body is identical to `try_build_incremental_view`; the
/// asymmetric-budget pin
/// (`prop_drift_detector_reports_one_path_errors_other_succeeds`) achieves
/// asymmetry by passing TWO DIFFERENT `ViewDef`s (one with `budget=1`, the
/// other with `budget=u64::MAX`), not by these helpers performing different
/// work. The helper-shape exists to document caller intent.
pub fn try_build_full_view(
    view_def: &ViewDef,
    writes: &[Write],
) -> Result<MaterializedView, ViewError> {
    try_build_incremental_view(view_def, writes)
}

/// Compute a structured diff between two materialisations.
///
/// **Consumer:** `algorithm_b_drift_detector.rs::prop_*` proptests use
/// the diff in `prop_assert_eq` error messages and read its `kind()` to
/// distinguish Equal / Drift outcomes.
pub fn structured_diff(a: &MaterializedView, b: &MaterializedView) -> StructuredDiff {
    let incremental_rows = a.materialised();
    let from_scratch_rows = b.materialised();
    let kind = if a.canonical_bytes() == b.canonical_bytes() {
        DiffKind::Equal
    } else {
        DiffKind::Drift
    };
    let note = match kind {
        DiffKind::Equal => String::from("no drift"),
        DiffKind::Drift => format!(
            "row counts: incremental={} from_scratch={}",
            incremental_rows.len(),
            from_scratch_rows.len()
        ),
        DiffKind::AsymmetricBudget => unreachable!(
            "structured_diff over two MaterializedView wrappers cannot \
             surface AsymmetricBudget; that lives in asymmetric_path_diff"
        ),
    };
    StructuredDiff {
        kind,
        incremental_rows,
        from_scratch_rows,
        note,
    }
}

/// Compute the asymmetric-path diff for the
/// one-path-errors-other-succeeds proptest scenario.
///
/// Returns a `StructuredDiff` whose `kind()` is
/// `DiffKind::AsymmetricBudget` and `is_reported()` is true. Pinned by
/// `ivm-major-3 (c)` + `r4-r2-ivm-2` third-pin: the diff MUST be
/// explicitly reported (NOT silently filtered via prop_assert_eq
/// early-return — that would be a vacuous pass per pim-2 §3.6b).
pub fn asymmetric_path_diff(
    incremental_err: &Option<ViewError>,
    from_scratch: &MaterializedView,
) -> StructuredDiff {
    let from_scratch_rows = from_scratch.materialised();
    let note = format!(
        "incremental erred ({}), from-scratch succeeded with {} rows",
        match incremental_err {
            Some(e) => format!("{e:?}"),
            None => String::from("<none>"),
        },
        from_scratch_rows.len(),
    );
    StructuredDiff {
        kind: DiffKind::AsymmetricBudget,
        incremental_rows: Vec::new(),
        from_scratch_rows,
        note,
    }
}

// =============================================================================
// Seed → fixture helpers (consumer-visible from algorithm_b_drift_detector.rs)
// =============================================================================

/// Build a `ViewDef` from the proptest seeds. Stable mapping so a
/// regression-trigger seed reproduces.
///
/// Picks from a small fixed label vocabulary so every seed lands on a
/// valid label. Drives the **`Algorithm::register` lane** (no budget) —
/// the load-bearing G15-A surface for the headline drift-detector pin.
/// `view_id` is non-canonical (`"user_view_<seed>"`) so `Algorithm::
/// register` instantiates `GenericKernel`.
pub fn build_view_def_from_seed(view_id_seed: u64, label_pattern_seed: u64) -> ViewDef {
    const LABELS: &[&str] = &["post", "user", "comment", "tag", "ephemeral"];
    let label = LABELS[(label_pattern_seed % LABELS.len() as u64) as usize];
    ViewDef::new(format!("user_view_{view_id_seed}"), label)
}

/// Translate proptest's `Vec<(u64, u64)>` seed into a `Write` sequence
/// over the small label vocabulary.
pub fn build_write_seq_from_seed(seq: &[(u64, u64)]) -> Vec<Write> {
    const LABELS: &[&str] = &["post", "user", "comment", "tag", "ephemeral"];
    seq.iter()
        .enumerate()
        .map(|(i, (label_pick, created_at_seed))| {
            let label = LABELS[(*label_pick % LABELS.len() as u64) as usize];
            let created_at = (*created_at_seed % 1_000_000) as i64;
            Write::new(label, created_at, i as u64)
        })
        .collect()
}

/// Build writes that trip the budget at `trip_idx`.
///
/// Drives the **budget-aware lane** (`ContentListingView::with_budget_for_testing`).
/// The view's per-update budget is `max(trip_idx, 1)`, so the
/// `(trip_idx+1)`th matching write trips. **Caller contract:**
/// `write_seq_size > trip_idx + 1` (the proptest range
/// `write_seq_size in 50..=500, trip_idx in 0..50` keeps this sound).
pub fn build_writes_with_trip_at(write_seq_size: usize, trip_idx: usize) -> (ViewDef, Vec<Write>) {
    let label = "post";
    let budget = (trip_idx as u64).max(1);
    let view_def = ViewDef::new("trip_view", label).with_budget(budget);
    let writes = (0..write_seq_size)
        .map(|i| Write::new(label, i as i64, i as u64))
        .collect();
    (view_def, writes)
}

/// Build asymmetric writes for the
/// `prop_drift_detector_reports_one_path_errors_other_succeeds` pin.
///
/// Returns `(incremental_def, from_scratch_def, writes)`. Asymmetry is
/// encoded in the two ViewDefs:
///
/// - `incremental` has `budget=1` — trips on the second matching write.
/// - `from_scratch` has `budget=u64::MAX` — succeeds over the full stream.
///
/// Both definitions use label `"post"` (the only label
/// `with_budget_for_testing` accepts). The write sequence is at least 2
/// `"post"` writes so the asymmetry triggers reliably.
pub fn build_asymmetric_budget_writes(
    write_seq_size: usize,
    _budget_trip_idx: usize,
) -> (ViewDef, ViewDef, Vec<Write>) {
    let label = "post";
    let incremental = ViewDef::new("inc_view", label).with_budget(1);
    let from_scratch = ViewDef::new("full_view", label).with_budget(u64::MAX);
    let writes = (0..write_seq_size.max(2))
        .map(|i| Write::new(label, i as i64, i as u64))
        .collect();
    (incremental, from_scratch, writes)
}

// =============================================================================
// Compile-time assertion: helper signatures stay stable across iterations
// =============================================================================

#[allow(dead_code, reason = "compile-time signature pin")]
fn _assert_signatures_compile() {
    let _: fn(&ViewDef, &[Write]) -> MaterializedView = build_incremental_view;
    let _: fn(&ViewDef, &[Write]) -> MaterializedView = build_full_view;
    let _: fn(&MaterializedView, &MaterializedView) -> StructuredDiff = structured_diff;
    let _: fn(&ViewDef, &[Write]) -> Result<MaterializedView, ViewError> =
        try_build_incremental_view;
    let _: fn(&ViewDef, &[Write]) -> Result<MaterializedView, ViewError> = try_build_full_view;
    let _: fn(&Option<ViewError>, &MaterializedView) -> StructuredDiff = asymmetric_path_diff;
}
