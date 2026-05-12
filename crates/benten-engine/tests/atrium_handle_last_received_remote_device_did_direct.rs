//! Phase 4-Foundation R3 (Family A — G22-C REVISED pick #2 per
//! sec-3.5-r1-8). Direct unit tests for the
//! [`benten_engine::engine_sync::AtriumHandle::last_received_remote_device_did`]
//! public API surface (Phase-3 G16-D wave-6b accessor pinning the
//! per-zone Option<Option<String>> contract).
//!
//! # Charter
//!
//! Per `docs/future/phase-3-backlog.md` §13.8 (BLOCKER — public-API
//! direct-test pin gap) + `.addl/phase-4-foundation/r2-test-landscape.md`
//! §2.1 G22-C REVISED row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 wave-1 G22-C.
//!
//! Pre-G22-C the accessor was exercised only through
//! `crates/benten-engine/tests/integration/atrium_two_device.rs`
//! AttributionFrame assertion paths. The 3-state public contract
//! (pre-merge None; post-merge Some(None); post-merge Some(Some(did)))
//! is not directly type-pinned anywhere.
//!
//! # What this pins
//!
//! The `last_received_remote_device_did(zone) -> Option<Option<String>>`
//! contract documented at `crates/benten-engine/src/engine_sync.rs:1055`:
//!
//! - **Outer `None`** — no merge has populated the per-zone slot yet
//!   (or it has been cleared post-AttributionFrame consumption per the
//!   `clear_last_received_remote_device_did` pattern at lines 1070-1073).
//! - **Outer `Some(None)`** — a merge populated the slot but the wire
//!   envelope carried no declared device-DID (legacy / pre-G16-D
//!   wave-6b interop path).
//! - **Outer `Some(Some(did))`** — a merge observed an explicit
//!   device-DID on the wire envelope (the G16-D wave-6b cryptographic
//!   attestation slot the post-merge AttributionFrame consumes).
//!
//! # Coverage matrix
//!
//! - **Pre-merge baseline** — a freshly-opened atrium handle returns
//!   `None` for any zone before any merge has run.
//! - **Per-zone partitioning** — distinct zones carry distinct slots;
//!   a merge into zone A does not leak its device-DID into zone B's
//!   read.
//! - **Unknown zone** — querying a zone never registered on the handle
//!   returns `None` (the `BTreeMap::get` miss).
//!
//! Post-merge populated-slot semantics are exercised by the existing
//! AttributionFrame integration tests at
//! `tests/integration/atrium_two_device.rs` — those drive a real
//! merge path. This direct-test pin sits at the type-level / unmerged
//! baseline boundary that the integration tests don't cover.
//!
//! # §3.6b end-to-end pin
//!
//! The substantive consequence of the accessor returning correct
//! Option<Option<String>> shape is observable in
//! [`benten_engine::Engine::apply_atrium_merge`] downstream — the
//! post-merge AttributionFrame consumes whatever this accessor
//! returns at consumption time. Removing the per-zone partitioning
//! (e.g. flattening to a single Mutex<Option<Option<String>>>) would
//! cause a multi-zone merge to inherit the wrong device-DID into the
//! second zone's frame — visible at the
//! `frame.device_did` field downstream.
//!
//! # RED-PHASE
//!
//! At write-time (R3 Family A; base SHA `f3930e1`) the accessor IS
//! implemented (lines 1055-1063 of engine_sync.rs). R5 G22-C runs the
//! verification pass; these tests stay `#[ignore]`-marked with a
//! RED-PHASE tag until that pass confirms §13.8 direct-test contract
//! coverage.
//!
//! # Owned by
//!
//! Phase 4-Foundation R3 Family A test-writer. Closes at R5 G22-C.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_engine::atrium_api::AtriumConfig;
use benten_engine::engine_sync::AtriumHandle;

#[tokio::test]
#[ignore = "RED-PHASE: closes at R5 G22-C (§13.8 direct-test verify). Un-ignore after G22-C verification pass."]
async fn last_received_remote_device_did_returns_none_for_unknown_zone_pre_merge() {
    let atrium = AtriumHandle::open(AtriumConfig::for_test())
        .await
        .expect("open atrium");

    // No merge has run; no zone has been registered. The outer
    // Option MUST be `None` — distinguishing "never observed" from
    // "observed-but-no-DID" (Some(None)).
    let result = atrium
        .last_received_remote_device_did("/zone/unobserved")
        .await;
    assert!(
        result.is_none(),
        "expected None pre-merge for unknown zone, got {result:?}; \
         the outer Option distinguishes never-observed from \
         observed-without-DID (Some(None)) — a regression that flattens \
         this distinction would silently mis-attribute AttributionFrame \
         device_did slots downstream",
    );
}

#[tokio::test]
#[ignore = "RED-PHASE: closes at R5 G22-C (§13.8 direct-test verify). Un-ignore after G22-C verification pass."]
async fn last_received_remote_device_did_returns_none_for_registered_zone_pre_merge() {
    let atrium = AtriumHandle::open(AtriumConfig::for_test())
        .await
        .expect("open atrium");
    atrium.register_zone("/zone/posts").await;

    // register_zone alone MUST NOT populate the slot — only an
    // actual merge can. The accessor must distinguish "zone known"
    // from "zone has a merge result waiting".
    let result = atrium.last_received_remote_device_did("/zone/posts").await;
    assert!(
        result.is_none(),
        "expected None for registered-but-pre-merge zone, got {result:?}; \
         register_zone is bookkeeping not a merge — must not populate the \
         per-zone last-received slot",
    );
}

#[tokio::test]
#[ignore = "RED-PHASE: closes at R5 G22-C (§13.8 direct-test verify). Un-ignore after G22-C verification pass."]
async fn last_received_remote_device_did_partitions_per_zone() {
    let atrium = AtriumHandle::open(AtriumConfig::for_test())
        .await
        .expect("open atrium");
    atrium.register_zone("/zone/posts").await;
    atrium.register_zone("/zone/comments").await;

    // Pre-merge BOTH zones MUST read `None` independently. The
    // per-zone partitioning contract: a flattened
    // `Mutex<Option<Option<String>>>` would conflate the two reads.
    let posts = atrium.last_received_remote_device_did("/zone/posts").await;
    let comments = atrium
        .last_received_remote_device_did("/zone/comments")
        .await;

    assert!(
        posts.is_none(),
        "expected None for /zone/posts pre-merge, got {posts:?}",
    );
    assert!(
        comments.is_none(),
        "expected None for /zone/comments pre-merge, got {comments:?}",
    );
}
