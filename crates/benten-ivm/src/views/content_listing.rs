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
//! - **Sort key.** `on_change(Node)` reads the Node's `createdAt` property
//!   (expected type: `Value::Int`). Absent or wrong-typed, the entry is
//!   skipped with no error — the zero-config `crud('post')` path is
//!   contractually required (plan §2.7 row B6) to inject `createdAt` at
//!   WRITE time, so a missing key is a mis-wired caller.
//! - **Trait `update(&ChangeEvent)` fallback.** `ChangeEvent` currently
//!   carries only `(cid, labels, kind, tx_id, ...)` — NOT the Node's
//!   properties. There is no way for the trait method to read `createdAt`,
//!   so it uses `tx_id` (monotonic per-process) as the sort key for
//!   label-matched events. This is a **named Phase-1 compromise**: the
//!   subscriber→view handoff needs to carry the Node (or properties)
//!   alongside the event for a proper end-to-end createdAt path. Phase 2
//!   widens `ChangeEvent` or introduces a property-carrying variant.
//! - **Duplicates are preserved.** The view has list semantics (the R3 test
//!   `content_listing_all_returned_after_three_writes` asserts 3 creates of
//!   the same CID yield 3 entries). We key BTreeMap entries by a composite
//!   `(sort_key, disambiguator)` so equal `createdAt` values don't collide.
//! - **Delete semantics.** `Deleted` events remove ALL entries with the
//!   matching CID (regardless of sort key). Matches the R3 test
//!   `content_listing_delete_removes_entry`.
//! - **Budget model.** `with_budget_for_testing(N)` allows `N` successful
//!   `on_change` calls; the `(N+1)`th flips the view to `Stale` without
//!   applying the update. Last-known-good state is preserved for
//!   `read_page_allow_stale`. `rebuild_from_scratch` resets to Fresh.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use benten_core::{Cid, Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};

use crate::{View, ViewDefinition, ViewError, ViewQuery, ViewResult, ViewState};

/// Composite sort key: `(createdAt, disambiguator)`. The disambiguator is a
/// per-view monotonic counter so two inserts with equal `createdAt` don't
/// collide in the `BTreeMap` (list semantics, not set semantics).
type SortKey = (i64, u64);

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
    /// Budget: max number of `on_change` / `update` calls before the view
    /// flips to `Stale`. `u64::MAX` disables the trip in normal construction.
    remaining_budget: u64,
    /// Runtime state. Reads under `Stale` either error (`read_page`) or
    /// return the last-known-good snapshot (`read_page_allow_stale`).
    state: ViewState,
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
            remaining_budget: u64::MAX,
            state: ViewState::Fresh,
            last_known_good: Vec::new(),
        }
    }

    /// Content-addressed definition for the view registry (label
    /// `system:IVMView`).
    pub fn definition() -> ViewDefinition {
        ViewDefinition {
            view_id: "content_listing".into(),
            input_pattern_label: Some("post".into()),
            output_label: "system:IVMView".into(),
        }
    }

    /// Low-budget constructor used by the stale-on-budget edge-case tests.
    /// Accepts exactly `budget` successful updates before the next update
    /// trips the view to `Stale`.
    #[must_use]
    pub fn with_budget_for_testing(budget: u64) -> Self {
        let mut v = Self::new("post");
        v.remaining_budget = budget;
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
            return Err(ViewError::BudgetExceeded("content_listing".into()));
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
        self.state = ViewState::Fresh;
        // Rebuild restores budget — a stale view recovering must be able to
        // accept new updates. Use the original budget sentinel: if the view
        // was constructed with a finite budget, we can't recover it here
        // without carrying the original around, so Phase 1 restores to
        // unbounded (matches the `view_recovers_on_rebuild_after_stale`
        // test which only checks `state == Fresh` + `read_page` succeeds).
        self.remaining_budget = u64::MAX;
        Ok(())
    }

    /// Runtime state (`Fresh` or `Stale`).
    #[must_use]
    pub fn state(&self) -> ViewState {
        self.state
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
        if self.state == ViewState::Stale {
            return;
        }
        // Budget models "work per on_change call" — every invocation counts
        // against the budget whether or not the label matches, because the
        // view still spent a probe pattern-match against the event. This
        // matches the `stale_on_budget_exceeded` test's expectation that
        // two label-mismatched updates trip a budget-1 view.
        if self.remaining_budget == 0 {
            self.trip_to_stale();
            return;
        }
        self.remaining_budget = self.remaining_budget.saturating_sub(1);
        if !node.labels.iter().any(|l| l == &self.label) {
            return;
        }
        let Some(sort_key) = extract_created_at(&node) else {
            return;
        };
        let Ok(cid) = node.cid() else {
            return;
        };
        let disambiguator = self.next_disambiguator;
        self.next_disambiguator = self.next_disambiguator.wrapping_add(1);
        self.entries.insert((sort_key, disambiguator), cid);
    }

    /// Strict paginated read. Returns `Err(ViewError::Stale)` when the view
    /// is stale.
    ///
    /// # Errors
    ///
    /// Returns [`ViewError::Stale`] when the view's state is `Stale`.
    pub fn read_page(&self, offset: usize, limit: usize) -> Result<Vec<Cid>, ViewError> {
        if self.state == ViewState::Stale {
            return Err(ViewError::Stale {
                view_id: "content_listing".into(),
            });
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
        if self.state == ViewState::Stale {
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
            .cloned()
            .collect()
    }

    /// Snapshot last-known-good before flipping to `Stale`.
    fn trip_to_stale(&mut self) {
        self.last_known_good = self.entries.values().cloned().collect();
        self.state = ViewState::Stale;
    }

    /// Remove all entries pointing at `cid`. Used for `Deleted` events.
    fn remove_all_with_cid(&mut self, cid: &Cid) {
        self.entries.retain(|_, v| v != cid);
    }
}

impl View for ContentListingView {
    /// Ingest a `ChangeEvent`. The event does not carry the Node's
    /// properties, so this path uses `tx_id` as the sort key (Phase-1
    /// compromise documented at the module top). For tests that need real
    /// `createdAt` ordering, use [`ContentListingView::on_change`] which
    /// consumes the full Node.
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        if self.state == ViewState::Stale {
            return Err(ViewError::Stale {
                view_id: "content_listing".into(),
            });
        }
        if !event.labels.iter().any(|l| l == &self.label) {
            return Ok(());
        }
        match event.kind {
            ChangeKind::Created | ChangeKind::Updated => {
                if self.remaining_budget == 0 {
                    self.trip_to_stale();
                    return Err(ViewError::BudgetExceeded("content_listing".into()));
                }
                // Phase-1 compromise: tx_id (i64-cast) as sort key stand-in
                // when the event doesn't carry createdAt. Monotonic within
                // a process, so ordering is still stable.
                let sort_key = i64::try_from(event.tx_id).unwrap_or(i64::MAX);
                let disambiguator = self.next_disambiguator;
                self.next_disambiguator = self.next_disambiguator.wrapping_add(1);
                self.entries
                    .insert((sort_key, disambiguator), event.cid.clone());
                self.remaining_budget = self.remaining_budget.saturating_sub(1);
            }
            ChangeKind::Deleted => {
                self.remove_all_with_cid(&event.cid);
            }
        }
        Ok(())
    }

    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError> {
        if self.state == ViewState::Stale {
            return Err(ViewError::Stale {
                view_id: "content_listing".into(),
            });
        }
        // Phase-1: if the query names a label, it must match this view's
        // watched label. If it doesn't, return empty (the query is for a
        // different listing).
        if let Some(ref wanted) = query.label {
            if wanted != &self.label {
                return Ok(ViewResult::Cids(Vec::new()));
            }
        }
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(usize::MAX);
        Ok(ViewResult::Cids(self.snapshot(offset, limit)))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        self.rebuild_from_scratch()
    }

    fn id(&self) -> &str {
        "content_listing"
    }

    fn is_stale(&self) -> bool {
        self.state == ViewState::Stale
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
    src.iter().skip(offset).take(limit).cloned().collect()
}
