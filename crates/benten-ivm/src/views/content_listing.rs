//! View 3 — Content listing (I5, exit-criterion load-bearing).
//!
//! Maintains a sorted-by-`createdAt` list of Node CIDs for a single label.
//! `crud('post').list` in the TS DSL consumes this view for Phase-1 Exit
//! Criterion #2 (plan §2.7). Paginated reads are O(log n + page_size) via
//! `BTreeMap` range scans.
//!
//! ## Phase 1 scope + compromises
//!
//! - **Single-label per view.** Each `ContentListingView` watches one label
//!   passed at construction. Multi-label listings compose by registering
//!   multiple views.
//! - **Sort key.** Both ingress paths prefer the Node's `createdAt` property
//!   (expected type: `Value::Int`). The trait `update(&ChangeEvent)` path
//!   reads it from `event.node` when present; absence falls back to `tx_id`
//!   (monotonic per-process) so legacy identity-only tests still order
//!   stably. Duplicates under equal `createdAt` are disambiguated by a
//!   per-view monotonic counter. The `on_change(Node)` path skips entries
//!   with no `createdAt` property — the zero-config `crud('post')` path is
//!   contractually required (plan §2.7 row B6) to inject `createdAt` at
//!   WRITE time.
//! - **Duplicates are preserved.** The view has list semantics (the R3 test
//!   `content_listing_all_returned_after_three_writes` asserts 3 creates of
//!   the same CID yield 3 entries).
//! - **Delete semantics.** `Deleted` events remove ALL entries with the
//!   matching CID (regardless of sort key). Matches the R3 test
//!   `content_listing_delete_removes_entry`.
//! - **Budget model.** `with_budget_for_testing(N)` allows `N` successful
//!   `on_change` calls; the `(N+1)`th flips the view to `Stale` without
//!   applying the update. Last-known-good state is preserved for
//!   `read_page_allow_stale`. `rebuild_from_scratch` resets to Fresh and
//!   restores the original budget (g5-cr-3 uniform budget-on-rebuild).

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use benten_core::{Cid, Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};

use crate::{BudgetTracker, View, ViewDefinition, ViewError, ViewQuery, ViewResult, ViewState};

/// Composite sort key: `(createdAt, disambiguator)`. The disambiguator is a
/// per-view monotonic counter so two inserts with equal `createdAt` don't
/// collide in the `BTreeMap` (list semantics, not set semantics).
///
/// The primary key is `u64` (not `i64`) per mini-review g5-ivm-10 so the
/// full `tx_id` range orders without the `unwrap_or(i64::MAX)` clamp that
/// collapsed writes past 2^63 onto the same key. We store an offset applied
/// to `createdAt` when it arrives: negative `createdAt` values (unusual,
/// but permitted by the `Value::Int` type) are biased into the `u64` range.
type SortKey = (u64, u64);

/// Bias applied so `i64::MIN` maps to `u64::MIN` and `i64::MAX` maps to
/// `u64::MAX`. Preserves total order.
const SORT_BIAS: u64 = 1u64 << 63;

/// Convert an `i64` `createdAt` into the `u64` sort key. `i64::MIN` becomes
/// `0`, `0` becomes `SORT_BIAS`, and `i64::MAX` becomes `u64::MAX` — a
/// simple affine bias that preserves total order.
#[inline]
fn bias_i64_to_u64(v: i64) -> u64 {
    (v as u64).wrapping_add(SORT_BIAS)
}

/// Stable view identifier used in `ViewError` payloads and `View::id`.
const VIEW_ID: &str = "content_listing";

/// View 3 — paginated sorted-by-`createdAt` content listing per label.
#[derive(Debug)]
pub struct ContentListingView {
    /// The label this view watches. Set at construction.
    label: String,
    /// Sorted map: `(createdAt, insertion_counter) → Cid`. `BTreeMap`
    /// yields O(log n + page_size) range scans, which is the read-path
    /// cost Exit Criterion #2 targets (<0.1ms for `crud('post').list`).
    entries: BTreeMap<SortKey, Cid>,
    /// Monotonic insertion counter for the composite sort key.
    next_disambiguator: u64,
    /// Shared `remaining/original/stale` tracker — see `crate::budget`
    /// (r6-ref R-major-02). This view wraps `BudgetTracker` to preserve
    /// last-known-good snapshotting on the stale transition.
    budget: BudgetTracker,
    /// Last-known-good snapshot, taken immediately before a `Stale`
    /// transition so relaxed reads have something to return.
    last_known_good: Vec<Cid>,
}

impl ContentListingView {
    /// Construct a view watching `label`, with a generous (effectively
    /// unbounded) budget.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            entries: BTreeMap::new(),
            next_disambiguator: 0,
            budget: BudgetTracker::new(u64::MAX),
            last_known_good: Vec::new(),
        }
    }

    /// Content-addressed definition for the view registry (label
    /// `system:IVMView`).
    pub fn definition() -> ViewDefinition {
        ViewDefinition {
            view_id: VIEW_ID.into(),
            input_pattern_label: Some("post".into()),
            output_label: "system:IVMView".into(),
            strategy: crate::Strategy::A,
        }
    }

    /// Low-budget constructor used by the stale-on-budget edge-case tests.
    /// Accepts exactly `budget` successful updates before the next update
    /// trips the view to `Stale`.
    #[must_use]
    pub fn with_budget_for_testing(budget: u64) -> Self {
        let mut v = Self::new("post");
        v.budget = BudgetTracker::new(budget);
        v
    }

    /// Fallible constructor — refuses `budget == 0` (no room for the first
    /// update, which would produce a view that's `Stale` before any data
    /// arrives).
    ///
    /// # Errors
    ///
    /// Returns [`ViewError::BudgetExceeded`] when `budget == 0`.
    pub fn try_with_budget(budget: u64) -> Result<Self, ViewError> {
        if budget == 0 {
            return Err(ViewError::BudgetExceeded(VIEW_ID.into()));
        }
        Ok(Self::with_budget_for_testing(budget))
    }

    /// Force a full rebuild from scratch. Phase-1 semantics: clears the
    /// indexed state and resets `state` to `Fresh`. The view is "empty but
    /// consistent" post-rebuild; a real rebuild would replay historical
    /// events — Phase 2 wires that against the change-event log.
    ///
    /// # Errors
    ///
    /// Infallible in Phase 1; returns `Result` for forward-compat with
    /// Phase-2 event-log replay which can fail.
    pub fn rebuild_from_scratch(&mut self) -> Result<(), ViewError> {
        self.entries.clear();
        self.next_disambiguator = 0;
        self.last_known_good.clear();
        // Rebuild restores the originally-configured budget and clears
        // the stale flag. Uniform across all 5 views (g5-cr-3).
        self.budget.rebuild();
        Ok(())
    }

    /// Runtime state (`Fresh` or `Stale`).
    #[must_use]
    pub fn state(&self) -> ViewState {
        if self.budget.is_stale() {
            ViewState::Stale
        } else {
            ViewState::Fresh
        }
    }

    /// Ingest a Node-level change directly. Used by edge-case tests that
    /// need to feed properties the `ChangeEvent` doesn't carry (`createdAt`).
    ///
    /// Semantics:
    /// - On `Stale`, no-op. The view stays stale until `rebuild_from_scratch`.
    /// - Budget exhaustion: snapshot last-known-good, flip to `Stale`,
    ///   drop the update.
    /// - Label mismatch: no-op.
    /// - Missing / wrong-typed `createdAt`: no-op.
    /// - Match: insert under composite sort key.
    pub fn on_change(&mut self, node: Node) {
        if self.budget.is_stale() {
            return;
        }
        // Budget models "work per on_change call" — every invocation counts
        // against the budget whether or not the label matches, because the
        // view still spent a probe pattern-match against the event. This
        // matches the `stale_on_budget_exceeded` test's expectation that
        // two label-mismatched updates trip a budget-1 view.
        if self.budget.remaining() == 0 {
            self.trip_to_stale();
            return;
        }
        let _ = self.budget.try_consume(1, VIEW_ID);
        if !node.labels.iter().any(|l| l == &self.label) {
            return;
        }
        let Some(created_at) = extract_created_at(&node) else {
            return;
        };
        let Ok(cid) = node.cid() else {
            return;
        };
        let disambiguator = self.next_disambiguator;
        self.next_disambiguator = self.next_disambiguator.wrapping_add(1);
        self.entries
            .insert((bias_i64_to_u64(created_at), disambiguator), cid);
    }

    /// Strict paginated read. Returns `Err(ViewError::Stale)` when the view
    /// is stale.
    ///
    /// # Errors
    ///
    /// Returns [`ViewError::Stale`] when the view's state is `Stale`.
    pub fn read_page(&self, offset: usize, limit: usize) -> Result<Vec<Cid>, ViewError> {
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
        }
        Ok(self.snapshot(offset, limit))
    }

    /// Relaxed paginated read. On `Stale`, returns the last-known-good
    /// snapshot (the state as of just before the trip). On `Fresh`, returns
    /// the live data.
    ///
    /// # Errors
    ///
    /// Infallible in Phase 1; returns `Result` for forward-compat symmetry
    /// with [`Self::read_page`].
    pub fn read_page_allow_stale(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Cid>, ViewError> {
        if self.budget.is_stale() {
            Ok(paginate_slice(&self.last_known_good, offset, limit))
        } else {
            Ok(self.snapshot(offset, limit))
        }
    }

    // ----- internal helpers -----

    /// Ordered live snapshot, paginated.
    fn snapshot(&self, offset: usize, limit: usize) -> Vec<Cid> {
        self.entries
            .values()
            .skip(offset)
            .take(limit)
            .copied()
            .collect()
    }

    /// Snapshot last-known-good before flipping to `Stale`.
    fn trip_to_stale(&mut self) {
        self.last_known_good = self.entries.values().copied().collect();
        self.budget.mark_stale();
    }

    /// Remove all entries pointing at `cid`. Used for `Deleted` events.
    fn remove_all_with_cid(&mut self, cid: &Cid) {
        self.entries.retain(|_, v| v != cid);
    }
}

impl View for ContentListingView {
    /// Ingest a `ChangeEvent`. Prefers `event.node.createdAt` when present
    /// (post-G5 fix-pass); falls back to `tx_id` (monotonic per-process)
    /// when the event is identity-only. The `tx_id` fallback is
    /// defense-in-depth for callers that still construct ChangeEvents
    /// without a node; chronologically-correct ordering requires that
    /// the emitter populate `event.node`.
    #[allow(
        clippy::print_stderr,
        reason = "Phase 1 fallback warn; Phase 2 routes to tracing"
    )]
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
        }
        if !event.labels.iter().any(|l| l == &self.label) {
            return Ok(());
        }
        match event.kind {
            ChangeKind::Created | ChangeKind::Updated => {
                // ivm-r6-4 (R6 fix-pass): consume budget BEFORE inserting so
                // a tracker that trips on this event leaves `entries`
                // unchanged. The prior order inserted first and consumed
                // last, which let the final write past the budget land in
                // the index even though the view immediately flipped Stale.
                if self.budget.remaining() == 0 {
                    self.trip_to_stale();
                    return Err(ViewError::BudgetExceeded(VIEW_ID.into()));
                }
                self.budget.try_consume(1, VIEW_ID)?;
                // Prefer event.node.createdAt; fall back to tx_id on
                // identity-only events (legacy test harness). The fallback
                // is announced on stderr so operators can notice when the
                // emitter forgot to populate the Node.
                //
                // g5-p2-ivm-1: both sort-key sources flow through
                // `bias_i64_to_u64` so they inhabit the SAME linear
                // order-preserving `u64` space. Without the parity bias a
                // legacy `tx_id=100` event would sort BEFORE a Node-bearing
                // `createdAt=0` event (biased to `2^63`), reversing the
                // intended chronology on mixed streams.
                let sort_primary: u64 = match event.node.as_ref().and_then(extract_created_at) {
                    Some(c) => bias_i64_to_u64(c),
                    None => {
                        eprintln!(
                            "benten-ivm: content_listing received identity-only event (no node, \
                             no createdAt); falling back to tx_id={} — this is a Phase-1 \
                             defense-in-depth path (see module docs)",
                            event.tx_id
                        );
                        // Cast to i64 before biasing: tx_ids are small
                        // monotonic u64 in practice, so reinterpretation
                        // through `as i64` preserves ordering within the
                        // operating range; the bias puts them in the same
                        // `u64` half as non-negative createdAt values.
                        bias_i64_to_u64(event.tx_id as i64)
                    }
                };
                let disambiguator = self.next_disambiguator;
                self.next_disambiguator = self.next_disambiguator.wrapping_add(1);
                self.entries
                    .insert((sort_primary, disambiguator), event.cid);
            }
            ChangeKind::Deleted => {
                // g5-p2-ivm-2: match budget cost to work done. A flood of
                // Deleted events against a large `entries` map is O(n) per
                // event (see `remove_all_with_cid`) — without this charge a
                // delete storm bypasses the stale-on-budget backpressure.
                // Option A per the brief: decrement by the entries removed
                // (or by 1 if none, so every probe still costs something).
                //
                // ivm-r6-5 (R6FP catch-up): mirror the Created branch's
                // consume-before-mutate ordering and propagate `try_consume`
                // failures up to the subscriber. The prior version deleted
                // first, then ignored a tripped `try_consume` Result and
                // returned `Ok(())` — the subscriber's `applied += 1`
                // counter was off by one in the trip case (the subscriber
                // counted the event as applied even though the view had
                // just flipped Stale). Pre-compute the cost via a
                // dry-run scan, charge the budget BEFORE mutating, and
                // surface `BudgetExceeded` so the subscriber's `match`
                // arm marks the view stale instead of bumping `applied`.
                if self.budget.remaining() == 0 {
                    self.trip_to_stale();
                    return Err(ViewError::BudgetExceeded(VIEW_ID.into()));
                }
                let removed: usize = self.entries.values().filter(|v| *v == &event.cid).count();
                let cost = (removed.max(1)) as u64;
                self.budget.try_consume(cost, VIEW_ID)?;
                self.remove_all_with_cid(&event.cid);
                if self.budget.remaining() == 0 && !self.entries.is_empty() {
                    // Optional Phase-2 perf upgrade: `cid → sort_key`
                    // reverse map makes this O(log n). Deferred.
                }
            }
            // Edge events are not relevant to a label-based content listing.
            ChangeKind::EdgeCreated | ChangeKind::EdgeDeleted => {}
        }
        Ok(())
    }

    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError> {
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
        }
        // Phase-1: if the query names a label, it must match this view's
        // watched label. A mismatch is an `E_IVM_PATTERN_MISMATCH` rather
        // than a silently-empty result (r6b §5.5 audit: silent-empty made
        // "no entries" and "queried a different listing" look identical).
        // Queries with no `label` filter still return the full listing.
        if let Some(ref wanted) = query.label
            && wanted != &self.label
        {
            return Err(ViewError::PatternMismatch(
                "content_listing: query label does not match this view's watched label".to_string(),
            ));
        }
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(usize::MAX);
        Ok(ViewResult::Cids(self.snapshot(offset, limit)))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        self.rebuild_from_scratch()
    }

    fn id(&self) -> &str {
        VIEW_ID
    }

    fn is_stale(&self) -> bool {
        self.budget.is_stale()
    }

    fn mark_stale(&mut self) {
        self.budget.mark_stale();
    }
}

/// Extract a Node's `createdAt` property as an `i64`. Returns `None` when
/// the property is absent or the wrong type.
fn extract_created_at(node: &Node) -> Option<i64> {
    match node.properties.get("createdAt") {
        Some(Value::Int(i)) => Some(*i),
        _ => None,
    }
}

/// Paginate an already-ordered slice (used for last-known-good snapshots
/// which are pre-materialized as a flat `Vec`).
fn paginate_slice(src: &[Cid], offset: usize, limit: usize) -> Vec<Cid> {
    src.iter().skip(offset).take(limit).copied().collect()
}
