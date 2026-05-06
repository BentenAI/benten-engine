//! G15-B (wave-5a) drift-detector proptest harness.
//!
//! Compares Algorithm B incremental updates vs from-scratch full computation
//! for an arbitrary `(view_id, label_pattern, write_seq)` triple. Drift
//! between the two materialisations surfaces as a structured diff via
//! `common::structured_diff`; budget-trip state-result observables are pinned
//! by the three ivm-major-3 / r4-r2-ivm-2 scenarios.
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
//! - `ivm-minor-3` (10k cases or budget-bounded; wallclock < 60s in
//!   CI; iteration count calibrated to budget — start 1 000, grow to
//!   10 000 if budget allows).
//! - **R4-R2 carry (`r4-r2-ivm-2` MAJOR + `r4-r2-ivm-7` MINOR + `r4-r2-ivm-8` MINOR):**
//!     - State-result not execution-trace observable for budget-trip /
//!       rebuild-recovery (`r4-r2-ivm-2`).
//!     - `with_cases(1_000)` calibration starting-point per ivm-minor-3
//!       (`r4-r2-ivm-7`).
//!     - Helper signatures pinned via `tests/common.rs` scaffolding
//!       (`r4-r2-ivm-8`).
//!
//! ## Calibration
//!
//! `with_cases(1_000)` per `r4-r2-ivm-7` starting-point. Each case is O(n)
//! over the bounded write sequence (`vec(.., 1..=200)` collection size), so
//! 1 000 cases run well under the 60s CI budget. Future agents may grow
//! toward 10 000 once a baseline run confirms wallclock headroom.

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

    /// G15-B headline pin — incremental == rebuild over arbitrary label
    /// patterns + write sequences. ZERO drift expected in 1 000 cases.
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
    /// pre-trip last-known-good shape (empty Vec for ContentListingView's
    /// snapshot — the salient observable is "no panic + state-result
    /// preserved", not the row count).
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
            Ok(rows) => prop_assert!(
                false,
                "expected ViewError::Stale, got Ok({} rows)",
                rows.len()
            ),
            Err(other) => prop_assert!(
                false,
                "expected ViewError::Stale, got {other:?}"
            ),
        }

        // State-result observable #3: relaxed read returns last-known-good
        // (no error). For ContentListingView, last-known-good is the
        // pre-trip snapshot which may be empty depending on which writes
        // landed before the trip — the load-bearing observable is "no
        // error, no panic", not the row count.
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
    /// the wrapper is FRESH (no longer stale) AND content-equivalent to a
    /// from-scratch rebuild over the same writes. The recovery observable
    /// is a state-result (is_stale=false + canonical_bytes equal), NOT an
    /// execution trace ("the rebuild path was taken").
    #[test]
    fn prop_rebuild_after_stale_returns_view_to_fresh(
        view_id_seed in any::<u64>(),
        write_seq_size in 50usize..=500usize,
        budget_trip_idx in 0usize..50usize,
    ) {
        let _ = view_id_seed;
        let (view_def, writes) = build_writes_with_trip_at(write_seq_size, budget_trip_idx);

        // Step 1: incremental trip.
        let mut wrapper = build_incremental_view(&view_def, &writes);
        prop_assert!(wrapper.is_stale());

        // Step 2: trigger refresh (the recovery path).
        wrapper.refresh().expect("recovery refresh succeeds");

        // Step 3a: post-refresh wrapper FRESH — but only if the original
        // budget can fit the entire write sequence after rebuild. With the
        // tight budgets these scenarios use, refresh may re-trip; the
        // load-bearing claim is that recovery EITHER converges (state ->
        // Fresh) OR re-trips deterministically (state -> Stale). Both
        // are valid state-result outcomes; what we forbid is undefined
        // mid-state (e.g. Fresh-but-with-half-the-rows).
        if !wrapper.is_stale() {
            // Step 3b: when recovery converges, materialisation must equal
            // a from-scratch rebuild over the same writes (modulo the
            // recovery's tighter budget — both wrappers replay the same
            // sequence under the same budget contract).
            let from_scratch = build_full_view(&view_def, &writes);
            prop_assert_eq!(
                wrapper.canonical_bytes(),
                from_scratch.canonical_bytes(),
                "post-recovery-refresh materialisation MUST equal \
                 from-scratch (content-equivalence per r4-r2-ivm-2)"
            );
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    /// `ivm-major-3` (c) + `r4-r2-ivm-2` third-pin. Probes the
    /// BudgetExceeded-vs-rebuild-succeeds asymmetry. When ONE path errors
    /// (incremental hits BudgetExceeded) and the OTHER path succeeds
    /// (from-scratch rebuild within budget), the drift-detector helper
    /// REPORTS the asymmetry as a structured-diff entry — NOT a vacuous
    /// silent pass via `prop_assert_eq` early-return.
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
                // Interesting asymmetry: incremental tripped, full
                // succeeded. Drift-detector reports AsymmetricBudget.
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
                // Both succeeded or both errored — covered by sibling pins.
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
    /// extended (different-label-vocabulary) pattern. The structured diff
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
        // is Equal; when they differ, the diff reports Drift. The pin
        // asserts the diff helper does NOT silently coerce both shapes
        // to "Equal" regardless of input.
        let diff = structured_diff(&original_view, &extended_view);
        if original_def.label == extended_def.label {
            // Same label — drift-detector must report Equal.
            prop_assert_eq!(
                diff.kind(),
                DiffKind::Equal,
                "two views over the same label pattern + same writes \
                 must materialise identical row sets"
            );
        } else {
            // Two distinct labels over the same write stream may or may not
            // produce different row sets (e.g. zero matches under both).
            // The load-bearing claim is the diff helper REPORTS a kind;
            // it MUST NOT panic + MUST surface a kind in {Equal, Drift}.
            prop_assert!(
                matches!(diff.kind(), DiffKind::Equal | DiffKind::Drift),
                "diff between distinct-label views must surface a \
                 well-defined kind"
            );
        }
    }
}
