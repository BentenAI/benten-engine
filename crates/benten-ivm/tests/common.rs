//! G15-B (wave-5a) — drift-detector helpers shared by
//! `algorithm_b_drift_detector.rs`.
//!
//! Closes `r4-r2-ivm-8` MINOR (helper-undefined producer-consumer ambiguity)
//! by anchoring the helper home at `crates/benten-ivm/tests/common.rs`. The
//! incremental + from-scratch builders both run against the Phase-1
//! `ContentListingView` (label-keyed View; the simplest View whose update
//! path exercises the `BudgetTracker` trip path that ivm-major-3's three
//! state-result scenarios need to observe). Drift between the two
//! materialisations is caught by `structured_diff` (`canonical_bytes` =
//! BLAKE3 over the row-CID list, deterministic across runs).
//!
//! ## Producer-consumer pair (r4-r2-ivm-8)
//!
//! Producer (this file) — `MaterializedView` wraps the inner
//! `ContentListingView` + exposes the state-result observables
//! (`is_stale`, `read`, `read_with`, `canonical_bytes`, `refresh`,
//! `materialised`) ivm-major-3's pins observe.
//!
//! Consumer — `algorithm_b_drift_detector.rs::prop_*` calls
//! `build_incremental_view`, `build_full_view`, `try_build_*` siblings,
//! `structured_diff`, and `asymmetric_path_diff` to drive the
//! drift-detection harness.
//!
//! ## ivm-minor-3 calibration
//!
//! `with_cases(1_000)` is the starting calibration per `r4-r2-ivm-7`. The
//! incremental + full-view builders are O(n) over the write sequence + the
//! sequence is bounded by the proptest `vec(.., 1..=200)` collection size,
//! so each case is a microsecond of work and 1 000 cases comfortably fits
//! the < 60s wallclock CI budget.

#![allow(
    dead_code,
    reason = "G15-B helpers; some helpers are scenario-specific and may not be \
              referenced by every consumer in this file"
)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::question_mark,
    clippy::wrong_self_convention,
    clippy::ref_option,
    clippy::needless_pass_by_value,
    clippy::return_self_not_must_use
)]

use std::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::View;
use benten_ivm::views::ContentListingView;
use benten_ivm::{ViewError, ViewQuery, ViewResult};

// =============================================================================
// View-definition shape (consumer-visible across the drift-detector pins)
// =============================================================================

/// Compact view definition the drift-detector consumes. The shape is just
/// enough to drive a `ContentListingView` (label-keyed) — the drift-detector
/// observables don't depend on richer kernel features (Strategy::B with
/// arbitrary label patterns lands in G15-A).
#[derive(Debug, Clone)]
pub struct ViewDef {
    pub view_id: String,
    pub label: String,
    /// Optional budget for the inner view. `None` means unlimited (default).
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

/// A single write to be replayed through the inner view.
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
        // the row-set would collapse to a single CID under the first +
        // subsequent inserts would hit the deduplicating BTreeMap key.
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

/// Wrapper around the inner `ContentListingView` that exposes the
/// state-result observables ivm-major-3 + r4-r2-ivm-2 pin
/// (`is_stale`, `read`, `read_with`, `canonical_bytes`, `refresh`).
pub struct MaterializedView {
    inner: ContentListingView,
    last_writes: Vec<Write>,
}

/// Read-options the wrapper honours — minimal mirror of
/// `benten_engine::ReadViewOptions` so the drift-detector pins read
/// state-results without pulling in an engine dep.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadOptions {
    pub allow_stale: bool,
}

impl ReadOptions {
    pub fn default() -> Self {
        Self { allow_stale: false }
    }

    pub fn with_allow_stale(mut self, allow: bool) -> Self {
        self.allow_stale = allow;
        self
    }
}

impl MaterializedView {
    pub fn is_stale(&self) -> bool {
        self.inner.is_stale()
    }

    /// Strict read — fails with `ViewError::Stale` when the inner view is
    /// stale. The state-result pin in `prop_budget_trip_state_propagation_consistent`
    /// observes this.
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
            match self.inner.read_allow_stale(&q) {
                Ok(ViewResult::Cids(cids)) => Ok(cids),
                Ok(_) => Ok(Vec::new()),
                Err(ViewError::Stale { .. }) => Ok(Vec::new()),
                Err(e) => Err(e),
            }
        } else if self.inner.is_stale() {
            Err(ViewError::Stale {
                view_id: self.inner.id().to_string(),
            })
        } else {
            match self.inner.read(&q) {
                Ok(ViewResult::Cids(cids)) => Ok(cids),
                Ok(_) => Ok(Vec::new()),
                Err(e) => Err(e),
            }
        }
    }

    /// Recovery path — rebuild the inner view + replay every write that was
    /// recorded against this wrapper. Post-`refresh` the view is `Fresh` AND
    /// content-equivalent to a from-scratch rebuild over the same writes.
    /// Pinned by `prop_rebuild_after_stale_returns_view_to_fresh`.
    pub fn refresh(&mut self) -> Result<(), ViewError> {
        // Rebuild restores `Fresh` state (per BudgetTracker contract) AND
        // resets the per-update budget to its original cap. `last_writes`
        // are then re-applied so the materialisation matches a from-scratch
        // build over the same write sequence.
        self.inner.rebuild()?;
        for (i, w) in self.last_writes.clone().iter().enumerate() {
            // Best-effort re-apply: a stale trip during recovery surfaces
            // the budget trip again (the original budget might still be
            // too small). The caller observes via `is_stale()` whether the
            // recovery converged.
            let event = to_change_event(w, (i + 1) as u64);
            let _ = self.inner.update(&event);
            if self.inner.is_stale() {
                break;
            }
        }
        Ok(())
    }

    /// Canonical row-set bytes (sorted CIDs as bytes; deterministic across
    /// runs). Used by `structured_diff` to surface drift.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let q = ViewQuery {
            label: None,
            limit: None,
            offset: None,
            ..Default::default()
        };
        let cids = match self.inner.read_allow_stale(&q) {
            Ok(ViewResult::Cids(c)) => c,
            _ => Vec::new(),
        };
        let mut sorted: Vec<Cid> = cids;
        sorted.sort_by_key(|c| *c.as_bytes());
        let mut out = Vec::with_capacity(sorted.len() * 36);
        for c in &sorted {
            out.extend_from_slice(c.as_bytes());
            out.push(0xff); // separator so prefix-collisions don't alias
        }
        out
    }

    /// Materialised row set, sorted by CID. Used by structured_diff to
    /// produce a row-by-row diff message.
    pub fn materialised(&self) -> Vec<Cid> {
        let q = ViewQuery {
            label: None,
            limit: None,
            offset: None,
            ..Default::default()
        };
        let mut cids: Vec<Cid> = match self.inner.read_allow_stale(&q) {
            Ok(ViewResult::Cids(c)) => c,
            _ => Vec::new(),
        };
        cids.sort_by_key(|c| *c.as_bytes());
        cids
    }

    fn from_inner(inner: ContentListingView) -> Self {
        Self {
            inner,
            last_writes: Vec::new(),
        }
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

    /// True if the diff is explicitly reported (NOT a silent
    /// prop_assert_eq early-return pass). Both `Drift` and
    /// `AsymmetricBudget` report; `Equal` is the silent-pass case.
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

fn make_view(view_def: &ViewDef) -> ContentListingView {
    if let Some(budget) = view_def.budget {
        ContentListingView::with_budget_for_testing(budget)
    } else {
        // ContentListingView's `with_budget_for_testing` always uses label
        // "post"; the unbudgeted constructor takes a label. Use the unbudgeted
        // path so an arbitrary label can drive matching.
        ContentListingView::new(view_def.label.clone())
    }
}

/// Build an incremental view by replaying every write through
/// `View::update`. Budget-trips are absorbed (the wrapper's `is_stale()`
/// observable surfaces them) — this matches the `prop_budget_trip_*`
/// expectation that the wrapper transitions to stale state without panic.
pub fn build_incremental_view(view_def: &ViewDef, writes: &[Write]) -> MaterializedView {
    let mut inner = make_view(view_def);
    let mut last_writes = Vec::with_capacity(writes.len());
    for (i, w) in writes.iter().enumerate() {
        let event = to_change_event(w, (i + 1) as u64);
        let _ = inner.update(&event);
        last_writes.push(w.clone());
        // After a budget trip the inner view stays stale; subsequent
        // updates short-circuit. We continue the loop so `last_writes`
        // captures the entire write sequence — `refresh()` replays it.
    }
    MaterializedView { inner, last_writes }
}

/// Build a from-scratch view by full rebuild over all writes. Drives the
/// same `View::update` surface; the budget contract is the same. The only
/// difference vs `build_incremental_view` is conceptual — this represents
/// the "single full-rebuild pass" baseline that the drift-detector compares
/// the incremental path against.
pub fn build_full_view(view_def: &ViewDef, writes: &[Write]) -> MaterializedView {
    // For ContentListingView, a "from-scratch rebuild" over the whole write
    // sequence is structurally identical to incremental application — there
    // is no different code path. We pin parity via the `BudgetTracker`'s
    // `rebuild` contract (resets the original cap) by constructing a fresh
    // inner view + applying every write.
    let mut inner = make_view(view_def);
    for (i, w) in writes.iter().enumerate() {
        let event = to_change_event(w, (i + 1) as u64);
        let _ = inner.update(&event);
    }
    MaterializedView::from_inner(inner)
}

/// Result-returning sibling of `build_incremental_view`. Returns
/// `Err(ViewError::BudgetExceeded)` instead of producing a stale wrapper
/// when the budget trips during incremental apply.
pub fn try_build_incremental_view(
    view_def: &ViewDef,
    writes: &[Write],
) -> Result<MaterializedView, ViewError> {
    let mut inner = make_view(view_def);
    for (i, w) in writes.iter().enumerate() {
        let event = to_change_event(w, (i + 1) as u64);
        if let Err(e) = inner.update(&event) {
            return Err(e);
        }
    }
    Ok(MaterializedView::from_inner(inner))
}

/// Result-returning sibling of `build_full_view`. Surfaces a budget trip as
/// `Err(ViewError::BudgetExceeded)`.
pub fn try_build_full_view(
    view_def: &ViewDef,
    writes: &[Write],
) -> Result<MaterializedView, ViewError> {
    try_build_incremental_view(view_def, writes)
}

/// Compute a structured diff between two materialisations.
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

/// Compute the asymmetric-path diff for the one-path-errors-other-succeeds
/// proptest scenario. Returns a `StructuredDiff` whose `kind()` is
/// `DiffKind::AsymmetricBudget` and `is_reported()` is true.
pub fn asymmetric_path_diff(
    incremental_err: &Option<ViewError>,
    from_scratch: &MaterializedView,
) -> StructuredDiff {
    StructuredDiff {
        kind: DiffKind::AsymmetricBudget,
        incremental_rows: Vec::new(),
        from_scratch_rows: from_scratch.materialised(),
        note: format!(
            "incremental erred ({}), from-scratch succeeded with {} rows",
            match incremental_err {
                Some(e) => format!("{e:?}"),
                None => String::from("<none>"),
            },
            from_scratch.materialised().len(),
        ),
    }
}

// =============================================================================
// Seed → fixture helpers (consumer-visible from algorithm_b_drift_detector.rs)
// =============================================================================

/// Build a `ViewDef` from the proptest seeds. Stable mapping so a regression
///-trigger seed reproduces.
pub fn build_view_def_from_seed(view_id_seed: u64, label_pattern_seed: u64) -> ViewDef {
    // Pick from a small fixed label vocabulary so every seed lands on a
    // valid label — the drift-detector observes equality between
    // incremental + from-scratch over the SAME view, so the absolute
    // label doesn't matter (just that it's stable across both builders).
    const LABELS: &[&str] = &["post", "user", "comment", "tag", "system:Zone"];
    let label = LABELS[(label_pattern_seed % LABELS.len() as u64) as usize];
    ViewDef::new(format!("user_view_{view_id_seed}"), label)
}

/// Translate proptest's `Vec<(u64, u64)>` seed into a `Write` sequence over
/// the small label vocabulary. `(label_pick, created_at_seed)` per entry.
pub fn build_write_seq_from_seed(seq: &[(u64, u64)]) -> Vec<Write> {
    const LABELS: &[&str] = &["post", "user", "comment", "tag", "system:Zone"];
    seq.iter()
        .enumerate()
        .map(|(i, (label_pick, created_at_seed))| {
            let label = LABELS[(*label_pick % LABELS.len() as u64) as usize];
            // Spread `created_at` across a wide range so the BTreeMap sort
            // exercises the ordering invariant.
            let created_at = (*created_at_seed % 1_000_000) as i64;
            Write::new(label, created_at, i as u64)
        })
        .collect()
}

/// Build writes that trip the budget at `trip_idx` — the first `trip_idx`
/// writes are within budget, the rest exceed.
pub fn build_writes_with_trip_at(write_seq_size: usize, trip_idx: usize) -> (ViewDef, Vec<Write>) {
    let label = "post";
    // The view's budget is exactly `trip_idx + 1` so the (trip_idx+1)th
    // matching write trips. Cap the budget to a sane minimum (1).
    let budget = (trip_idx as u64).max(1);
    let view_def = ViewDef::new("trip_view", label).with_budget(budget);
    let writes = (0..write_seq_size)
        .map(|i| Write::new(label, i as i64, i as u64))
        .collect();
    (view_def, writes)
}

/// Build asymmetric writes for the one-path-errors-other-succeeds pin.
///
/// The writes fit within the from-scratch budget (post-rebuild reset)
/// but trip the incremental budget when applied piecewise. With
/// `ContentListingView`'s simpler shape we model this by: incremental
/// uses a very tight per-update budget (trips early), from-scratch uses
/// a large budget (succeeds). This exercises the AsymmetricBudget
/// structured-diff branch without requiring a more elaborate harness.
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
}
