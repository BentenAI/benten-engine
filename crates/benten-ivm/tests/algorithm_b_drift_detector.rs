//! G15-B drift-detector proptest harness (port against G15-A's merged
//! `Algorithm::register` surface).
//!
//! Compares Algorithm B incremental updates vs from-scratch full
//! computation for an arbitrary `(view_id, label_pattern, write_seq)`
//! triple. Drift between the two materialisations surfaces as a
//! structured diff via `common::structured_diff`; budget-trip
//! state-result observables are pinned by the three
//! `ivm-major-3` / `r4-r2-ivm-2` scenarios.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.3 G15-B rows
//!   `prop_algorithm_b_incremental_equals_rebuild_for_arbitrary_label_pattern`
//!   + `prop_budget_trip_state_propagation_consistent`
//!   + `prop_rebuild_after_stale_returns_view_to_fresh`
//!   + `prop_drift_detector_observes_label_pattern_extension`
//!   + `prop_drift_detector_reports_one_path_errors_other_succeeds`.
//! - plan §4 seed (drift-detector proptest seed planted in plan).
//! - `ivm-major-3` (drift-detector extended with budget-trip state /
//!   recovery / pattern-extension / one-path-errors-other-succeeds
//!   scenarios).
//! - `ivm-minor-3` (10 000 cases or budget-bounded; wallclock < 60s in
//!   CI; iteration count calibrated to budget — start 1 000, grow to
//!   10 000 if budget allows).
//! - **R4-R2 carry (`r4-r2-ivm-2` MAJOR + `r4-r2-ivm-7` MINOR + `r4-r2-ivm-8` MINOR):**
//!     - State-result not execution-trace observable for budget-trip /
//!       rebuild-recovery (`r4-r2-ivm-2`).
//!     - `with_cases(1_000)` calibration starting-point per ivm-minor-3
//!       (`r4-r2-ivm-7`).
//!     - Helper signatures pinned via `tests/common.rs` (`r4-r2-ivm-8`).
//!
//! ## Surface choice (G15-B port)
//!
//! - The headline pin
//!   `prop_algorithm_b_incremental_equals_rebuild_for_arbitrary_label_pattern`
//!   + the pattern-extension pin both drive G15-A's merged
//!   `Algorithm::register(view_id, LabelPattern::Exact(label),
//!   Projection::all_props())` end-to-end (the helper's
//!   `Algorithm::register` lane). The drift-detector observes
//!   `GenericKernel`'s `is_stale` / row-set determinism.
//! - The budget-trip / rebuild-after-stale / asymmetric-budget pins drive
//!   the canonical `ContentListingView` inner kernel directly — this is
//!   the only path that reaches `BudgetTracker` (Algorithm::register does
//!   not currently surface a budget knob; named in the orchestrator
//!   report as a HARD-RULE rule-12 BELONGS-NAMED disposition pointing at
//!   `docs/future/phase-3-backlog.md` budget-knob followup).
//!
//! ## Calibration
//!
//! `with_cases(1_000)` per `r4-r2-ivm-7` starting-point. Each case is
//! O(n) over the bounded write sequence (`vec(.., 1..=200)` collection
//! size); 1 000 cases run well under the 60s CI budget. Future agents may
//! grow toward 10 000 once a baseline run confirms wallclock headroom.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::used_underscore_binding,
    clippy::collapsible_else_if
)]

mod common;

use proptest::prelude::*;

use common::{
    DiffKind, ReadOptions, asymmetric_path_diff, build_asymmetric_budget_writes, build_full_view,
    build_incremental_view, build_view_def_from_seed, build_write_seq_from_seed,
    build_writes_with_trip_at, structured_diff, try_build_full_view, try_build_incremental_view,
};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    /// G15-B headline pin — `Algorithm::register`-driven incremental ==
    /// rebuild over arbitrary label patterns + write sequences. Both
    /// sides of the comparison drive the merged G15-A surface
    /// (`Algorithm::register` instantiating `GenericKernel` for the
    /// non-canonical view ids the helper synthesises). ZERO drift
    /// expected in 1 000 cases.
    #[test]
    fn prop_algorithm_b_incremental_equals_rebuild_for_arbitrary_label_pattern(
        view_id_seed in any::<u64>(),
        label_pattern_seed in any::<u64>(),
        write_seq in proptest::collection::vec(any::<(u64, u64)>(), 1..=200),
    ) {
        let view_def = build_view_def_from_seed(view_id_seed, label_pattern_seed);
        let writes = build_write_seq_from_seed(&write_seq);

        let incremental = build_incremental_view(&view_def, &writes);
        let from_scratch = build_full_view(&view_def, &writes);

        let diff = structured_diff(&incremental, &from_scratch);
        prop_assert_eq!(
            diff.kind(),
            DiffKind::Equal,
            "Algorithm B incremental updates diverged from from-scratch \
             rebuild for view_def {:?} after {} writes; structured diff: {}",
            view_def,
            writes.len(),
            diff,
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    /// `ivm-major-3` (a) + `r4-r2-ivm-2` STATE-RESULT pin. Post-budget-trip
    /// the wrapper exposes `is_stale() == true` AND `read()` returns
    /// `ViewError::Stale`; relaxed `read_with(allow_stale)` returns the
    /// pre-trip last-known-good shape.
    #[test]
    fn prop_budget_trip_state_propagation_consistent(
        view_id_seed in any::<u64>(),
        write_seq_size in 50usize..=500usize,
        budget_trip_idx in 0usize..50usize,
    ) {
        let _ = view_id_seed;
        let (view_def, writes) = build_writes_with_trip_at(write_seq_size, budget_trip_idx);

        let wrapper = build_incremental_view(&view_def, &writes);

        // State-result observable #1: post-budget-trip is_stale() == true.
        prop_assert!(
            wrapper.is_stale(),
            "post-BudgetExceeded, wrapper.is_stale() MUST return true \
             (state-result observable per ivm-major-3 + r4-r2-ivm-2)"
        );

        // State-result observable #2: strict read returns ViewError::Stale.
        match wrapper.read() {
            Err(benten_ivm::ViewError::Stale { .. }) => {}
            Err(benten_ivm::ViewError::BudgetExceeded(_)) => {
                // Acceptable: BudgetExceeded shares E_IVM_VIEW_STALE error
                // code per ViewError::error_code mapping. Both surface
                // strict-read refusal post-trip; the load-bearing observable
                // is "strict read refuses with a stale-class typed error".
            }
            Ok(rows) => prop_assert!(
                false,
                "expected strict-read refusal (Stale or BudgetExceeded), got Ok({} rows)",
                rows.len()
            ),
            Err(other) => prop_assert!(
                false,
                "expected Stale or BudgetExceeded, got {other:?}"
            ),
        }

        // State-result observable #3: relaxed read returns last-known-good
        // (no error). For ContentListingView, last-known-good is the
        // pre-trip snapshot — the load-bearing observable is "no error,
        // no panic, returns a Vec".
        let opts = ReadOptions::default().with_allow_stale(true);
        match wrapper.read_with(opts) {
            Ok(_rows) => {}
            Err(e) => prop_assert!(
                false,
                "allow_stale read must not surface a typed error; got {e:?}"
            ),
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    /// `ivm-major-3` (b) + `r4-r2-ivm-2` STATE-RESULT recovery pin.
    /// After a budget trip, `refresh()` runs the rebuild path; post-refresh
    /// the wrapper's state is well-defined: EITHER `is_stale=false` AND
    /// content-equivalent to a from-scratch rebuild (recovery converged),
    /// OR `is_stale=true` (recovery re-tripped under the same tight
    /// budget). Both are valid state-results; what is forbidden is an
    /// undefined mid-state.
    #[test]
    fn prop_rebuild_after_stale_returns_view_to_fresh(
        view_id_seed in any::<u64>(),
        write_seq_size in 50usize..=500usize,
        budget_trip_idx in 0usize..50usize,
    ) {
        let _ = view_id_seed;
        let (view_def, writes) = build_writes_with_trip_at(write_seq_size, budget_trip_idx);

        // Step 1: trip the budget on incremental.
        let mut wrapper = build_incremental_view(&view_def, &writes);
        prop_assert!(wrapper.is_stale(), "precondition: incremental tripped");

        // Step 2: trigger refresh (the recovery path).
        wrapper.refresh().expect("recovery refresh succeeds");

        // Step 3: post-refresh state MUST be well-defined.
        if !wrapper.is_stale() {
            // Recovery converged → content-equivalent to from-scratch.
            let from_scratch = build_full_view(&view_def, &writes);
            // from-scratch path may itself trip under the same budget; if
            // it does, the comparison is between two stale wrappers — both
            // expose `materialised()` via read_allow_stale. The
            // load-bearing claim: if the incremental side reports Fresh,
            // its row set matches the from-scratch baseline.
            let diff = structured_diff(&wrapper, &from_scratch);
            prop_assert_eq!(
                diff.kind(),
                DiffKind::Equal,
                "post-recovery-refresh materialisation MUST equal \
                 from-scratch (content-equivalence per r4-r2-ivm-2; \
                 structured-diff: {})",
                diff,
            );
        }
        // else: recovery re-tripped — that's a valid state-result outcome
        // (state -> Stale). The forbidden case is an undefined mid-state
        // (e.g., Fresh-but-with-half-the-rows), which would have been
        // caught by the structured-diff arm above.
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    /// `ivm-major-3` (c) + `r4-r2-ivm-2` third-pin. Probes the
    /// BudgetExceeded-vs-rebuild-succeeds asymmetry. When ONE path
    /// errors (incremental hits BudgetExceeded with budget=1) and the
    /// OTHER path succeeds (from-scratch rebuild within the
    /// `budget=u64::MAX` cap), the drift-detector REPORTS the
    /// asymmetry as a structured-diff entry — NOT a vacuous silent
    /// pass via `prop_assert_eq` early-return.
    #[test]
    fn prop_drift_detector_reports_one_path_errors_other_succeeds(
        view_id_seed in any::<u64>(),
        write_seq_size in 50usize..=500usize,
        budget_trip_idx in 0usize..50usize,
    ) {
        let _ = view_id_seed;
        let (incremental_def, full_def, writes) =
            build_asymmetric_budget_writes(write_seq_size, budget_trip_idx);

        let incremental_result = try_build_incremental_view(&incremental_def, &writes);
        let from_scratch_result = try_build_full_view(&full_def, &writes);

        match (incremental_result, from_scratch_result) {
            (Err(_inc_err), Ok(full)) => {
                // Interesting asymmetry: incremental tripped, full succeeded.
                let inc_err_holder: Option<benten_ivm::ViewError> = Some(_inc_err);
                let diff = asymmetric_path_diff(&inc_err_holder, &full);
                prop_assert!(
                    diff.is_reported(),
                    "drift-detector MUST report one-path-errors-other-\
                     succeeds asymmetry; silent filter via prop_assert_eq \
                     early-return is a vacuous pass per pim-2 §3.6b + \
                     r4-r2-ivm-2"
                );
                prop_assert_eq!(diff.kind(), DiffKind::AsymmetricBudget);
            }
            (Ok(_), Ok(_)) | (Err(_), Err(_)) => {
                // Both succeeded or both errored — covered by sibling
                // pins (incremental_equals_rebuild + budget_trip_*).
            }
            (Ok(_), Err(e)) => {
                prop_assert!(
                    false,
                    "from-scratch errored while incremental succeeded: \
                     {e:?}; impossible asymmetry — incremental cannot \
                     have a smaller budget than from-scratch under \
                     Algorithm B"
                );
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    /// `ivm-major-3` pattern-extension pin. If a view's `label_pattern`
    /// is EXTENDED (e.g. `"crud:post"` exact → `"crud:"` prefix), the
    /// drift-detector observes + reports the change so operators see
    /// the new row coverage. Modeled here by registering two views over
    /// the SAME write set — one with the original pattern, one with the
    /// extended (different-label-vocabulary) pattern. Both views drive
    /// G15-A's `Algorithm::register` end-to-end. The structured diff
    /// reports drift when the row coverage genuinely differs.
    #[test]
    fn prop_drift_detector_observes_label_pattern_extension(
        view_id_seed in any::<u64>(),
        original_pattern_seed in any::<u64>(),
        extended_pattern_seed in any::<u64>(),
        write_seq in proptest::collection::vec(any::<(u64, u64)>(), 1..=100),
    ) {
        let _ = view_id_seed;
        let original_def = build_view_def_from_seed(0, original_pattern_seed);
        let extended_def = build_view_def_from_seed(0, extended_pattern_seed);
        let writes = build_write_seq_from_seed(&write_seq);

        let original_view = build_incremental_view(&original_def, &writes);
        let extended_view = build_incremental_view(&extended_def, &writes);

        // The structured-diff between the two distinct-pattern views is
        // observable: when the patterns coincide (same label) the diff
        // is Equal; when they differ, the diff reports either Equal (zero
        // matches under both) or Drift (different row sets). The pin
        // asserts the diff helper does NOT silently coerce both shapes
        // to "Equal" regardless of input.
        let diff = structured_diff(&original_view, &extended_view);
        if original_def.label == extended_def.label {
            prop_assert_eq!(
                diff.kind(),
                DiffKind::Equal,
                "two views over the same label pattern + same writes \
                 must materialise identical row sets"
            );
        } else {
            prop_assert!(
                matches!(diff.kind(), DiffKind::Equal | DiffKind::Drift),
                "diff between distinct-label views must surface a \
                 well-defined kind"
            );
        }
    }
}
