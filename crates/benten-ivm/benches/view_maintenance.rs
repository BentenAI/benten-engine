//! Criterion benchmarks: IVM view read + incremental maintenance.
//!
//! Two §14.6 direct/derived gates at the IVM layer:
//!
//! | Benchmark | Target | Source |
//! |---|---|---|
//! | `view_read_content_listing`     | < 1µs hot cache       | **§14.6 direct** — "IVM view read (clean): 0.04–1µs" |
//! | `view_incremental_maintenance`  | < 50µs per write      | **§14.6 derived** — ENGINE-SPEC §14.6 puts the full "Node creation + IVM update" envelope at 100–500µs. Decomposition: reserve ~50µs for the IVM slice and the balance for storage (redb put) + hashing (DAG-CBOR + BLAKE3). |
//!
//! ## Pattern
//!
//! `view_read_content_listing` exercises View 3 (content listing paginated
//! by `createdAt`, the view `crud('post').list` uses — Phase 1 exit-
//! criterion load-bearing). The read is O(log n + page_size).
//!
//! `view_incremental_maintenance` measures the cost of a single
//! `ChangeEvent` propagating through every subscribed view. The baseline
//! is the 5 Phase 1 hand-written views (capability grants, event handler
//! dispatch, content listing, governance inheritance, version-chain
//! CURRENT). Each view's update function is dispatched based on whether
//! the event's `(label, property)` pattern matches the view's input
//! pattern.
//!
//! ## Stub-graceful
//!
//! `benten-ivm` is a STUB crate at spike end (only `STUB_MARKER` is
//! exposed). Both benchmark functions below reference placeholders
//! (`register_content_view_stub`, `apply_change_event_stub`) that `todo!()`
//! with a message directing the R5 implementer to the real deliverable.
//! Running these benches before I1–I8 land WILL panic — which is the
//! correct TDD behavior. CI will mark the bench step red; the gate
//! passes once the real view code lands.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

// ---------------------------------------------------------------------------
// Stubs — replace when I3/I5 land.
// ---------------------------------------------------------------------------

/// Placeholder for the View 3 (content listing) read API.
///
/// Real signature (Phase 1 I5): `fn read_view(view_id: &str, page: usize,
/// page_size: usize) -> Result<Vec<Cid>, IvmError>`. The bench pre-seeds
/// the view with N entries so the read measures the hot-cache path.
fn read_content_view_stub(_page: usize, _page_size: usize) -> Vec<String> {
    todo!(
        "I5 — View 3 (content listing) read API not yet implemented; \
         bench will pass once benten-ivm exposes its read surface."
    )
}

/// Placeholder for the IVM change-event ingestion API.
///
/// Real signature (Phase 1 I2): `fn apply(&self, event: ChangeEvent) ->
/// Result<(), IvmError>`.
fn apply_change_event_stub() {
    todo!(
        "I2 — change-stream subscriber not yet implemented; \
         bench will pass once apply_change_event is wired up."
    )
}

// ---------------------------------------------------------------------------
// Benches
// ---------------------------------------------------------------------------

fn bench_view_read_content_listing(c: &mut Criterion) {
    let mut group = c.benchmark_group("view_read_content_listing");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("hot_cache_page_20", |b| {
        b.iter(|| {
            // Read first page of 20 entries from the already-populated view.
            let page = read_content_view_stub(black_box(0), black_box(20));
            black_box(page);
        });
    });
    group.finish();
}

fn bench_view_read_governance_inheritance(c: &mut Criterion) {
    // View 4 — governance inheritance. §14.6 direct: "IVM view read (clean):
    // 0.04–1µs" applies to HashMap/sorted-list strategies. View 4 walks the
    // effective-rules transitive closure, which is maintained at write time
    // so the read remains O(1). Gate target: < 1µs median hot cache.
    let mut group = c.benchmark_group("view_read_governance_inheritance");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("hot_cache_5_hop_closure", |b| {
        b.iter(|| {
            // Stub: real API will be `read_view("governance:effective", entity_cid)`.
            // Until I6 lands this panics via the shared stub.
            let result = read_content_view_stub(black_box(0), black_box(1));
            black_box(result);
        });
    });
    group.finish();
}

fn bench_view_incremental_maintenance(c: &mut Criterion) {
    let mut group = c.benchmark_group("view_incremental_maintenance");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("all_five_views_per_write", |b| {
        b.iter(|| {
            // One ChangeEvent fans out to every subscribed view.
            apply_change_event_stub();
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_view_read_content_listing,
    bench_view_read_governance_inheritance,
    bench_view_incremental_maintenance
);
criterion_main!(benches);
