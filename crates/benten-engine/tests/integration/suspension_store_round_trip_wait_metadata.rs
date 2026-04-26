//! Phase 2b R3 (R3-E) — G12-E generalized `SuspensionStore` wait-side
//! round-trip.
//!
//! TDD red-phase. Pin source: plan §3.2 G12-E (generalized
//! `SuspensionStore` covering BOTH wait metadata AND subscription
//! persistent cursors; trait shape `put_wait` / `get_wait` /
//! `put_cursor` / `get_cursor` / `delete`) + r1-streaming-systems
//! cross-cutting (G12-E generalizes from `WaitMetadataStore` to
//! `SuspensionStore`).
//!
//! This test pins the wait-metadata side of the generalized store:
//! `put_wait → get_wait` round-trips a structurally-identical
//! `WaitMetadata` with no field drift (deadline, signal-shape,
//! envelope-cid all preserved).
//!
//! Companion test in `suspension_store_round_trip_subscription_cursor.rs`
//! pins the subscription-cursor side. A third test
//! (`suspension_store_handles_both_wait_and_cursor_keys_without_collision`)
//! is captured inline here as an integration assertion that the two
//! key-spaces do not alias.
//!
//! **Status:** RED-PHASE (Phase 2b G12-E pending). `SuspensionStore`,
//! `WaitMetadata`, and `SuspensionKey` do not yet exist.
//!
//! Owned by R3-E.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// `suspension_store_round_trip_wait_metadata` — R2 §2.5 (G12-E generalization).
#[test]
#[ignore = "Phase 2b G12-E pending — SuspensionStore + WaitMetadata unimplemented"]
fn suspension_store_round_trip_wait_metadata() {
    let (_dir, engine) = fresh_engine();

    let store = benten_engine::testing::testing_get_suspension_store(&engine);
    let wait_id = benten_engine::testing::testing_make_wait_id("acme.posts.review/wait-001");
    let meta = benten_engine::testing::testing_make_wait_metadata_with_ttl_hours(24);

    store.put_wait(wait_id.clone(), meta.clone()).unwrap();

    let recovered = store
        .get_wait(&wait_id)
        .unwrap()
        .expect("get_wait must surface the just-written metadata");

    assert_eq!(
        meta, recovered,
        "WaitMetadata MUST round-trip byte-for-byte through the \
         SuspensionStore — drift indicates a serialization bug or a \
         lossy field that breaks cross-process resume (Compromise #9 \
         closure depends on this)"
    );
}

/// `suspension_store_handles_both_wait_and_cursor_keys_without_collision`
/// — R2 §2.5 (G12-E generalization).
///
/// The generalized store covers both wait metadata AND subscription
/// cursors. The two key-spaces MUST be disjoint — writing a cursor at
/// `("sub-001",)` MUST NOT shadow a wait at `("sub-001",)` if the keys
/// happened to alias in the underlying redb backend. This is the
/// structural pin that prevents the generalization from regressing
/// per-feature isolation.
#[test]
#[ignore = "Phase 2b G12-E pending"]
fn suspension_store_handles_both_wait_and_cursor_keys_without_collision() {
    let (_dir, engine) = fresh_engine();
    let store = benten_engine::testing::testing_get_suspension_store(&engine);

    // Use a colliding-by-string id to force the issue.
    let shared_id = "shared-id-001";
    let wait_id = benten_engine::testing::testing_make_wait_id(shared_id);
    let sub_id = benten_engine::testing::testing_make_subscriber_id(shared_id);
    let meta = benten_engine::testing::testing_make_wait_metadata_with_ttl_hours(24);
    let seq: u64 = 42;

    store.put_wait(wait_id.clone(), meta.clone()).unwrap();
    store.put_cursor(&sub_id, seq).unwrap();

    let wait_back = store.get_wait(&wait_id).unwrap();
    let cursor_back = store.get_cursor(&sub_id).unwrap();

    assert_eq!(
        wait_back,
        Some(meta),
        "wait-side write must survive a same-string cursor write (no key collision)"
    );
    assert_eq!(
        cursor_back,
        Some(seq),
        "cursor-side write must survive a same-string wait write (no key collision)"
    );
}
