//! G13-E pin (LIVE since Phase-3 R5 wave-3): `DurabilityMode::default()`
//! returns [`DurabilityMode::Group`] (was [`DurabilityMode::Immediate`]
//! through Phase-2b). Plan §3 G13-E + Compromise #12 closure.
//!
//! Pin source: r2-test-landscape §2.1 G13-E row
//! `durability_mode_group_default_for_crud_fast_path`; plan §3 G13-E;
//! S-3 / C-8 (security-posture Compromise #12 → CLOSED at G13-E).
//!
//! ## What G13-E did
//!
//! Phase-1 + Phase-2 shipped `DurabilityMode::Immediate` as the default
//! (every commit fsyncs); the existing
//! `crates/benten-graph/tests/durability_group_enum_preserved.rs`
//! pinned the `Group` variant existed but did not flip the default.
//!
//! G13-E flipped `DurabilityMode::default()` to `Group` for the CRUD
//! fast-path: writes target a batched fsync window at the engine
//! surface (per S-3 / C-8 spec) instead of fsyncing per commit.
//! Closes Compromise #12 (APFS fsync floor) per
//! `docs/SECURITY-POSTURE.md`.
//!
//! ## redb v4 backend caveat
//!
//! redb v4 still only exposes `Durability::Immediate` and
//! `Durability::None`; `DurabilityMode::Group` collapses to
//! `Durability::Immediate` at the redb mapping (see
//! [`crates/benten-graph/src/redb_backend.rs::to_redb_durability`]).
//! The default flip is load-bearing at the engine-surface posture
//! level — it declares the right Compromise-#12-closing default
//! for non-redb backends and for whenever redb grows native
//! batched-commit support. The bench harness at
//! `crates/benten-graph/benches/crud_post_create_dispatch_group_durability.rs`
//! still reserves the shape and the
//! [`crates/benten-graph/src/redb_backend.rs::warn_if_group_durability_collapsed`]
//! one-shot warning still fires on explicit Group requests.

#![allow(clippy::unwrap_used)]

#[test]
fn durability_mode_group_default_for_crud_fast_path() {
    // OBSERVABLE consequence per pim-2 §3.6b: opening a backend without
    // explicitly setting durability gets the Group fast-path default.
    // Defends against a regression that lands the supporting machinery
    // (`to_redb_durability` mapping, `warn_if_group_durability_collapsed`,
    // SECURITY-POSTURE narrative) but forgets to flip the actual default
    // value at `crates/benten-graph/src/backend.rs::DurabilityMode::default`.
    let default = benten_graph::DurabilityMode::default();
    assert_eq!(
        default,
        benten_graph::DurabilityMode::Group,
        "G13-E flipped DurabilityMode::default() to Group per S-3 / C-8 / \
         SECURITY-POSTURE.md Compromise #12 closure; if this assertion \
         fails the CRUD fast-path posture has regressed back to Immediate"
    );
}

/// Companion: explicit Immediate construction still works (capability
/// grants depend on it; see
/// [`crates/benten-graph/tests/capability_grant_writes_immediate.rs`]).
#[test]
fn durability_mode_group_default_does_not_remove_immediate_variant() {
    let immediate = benten_graph::DurabilityMode::Immediate;
    assert_ne!(
        immediate,
        benten_graph::DurabilityMode::default(),
        "Immediate is still the explicit-opt-in for capability grants + \
         pin-precise crash semantics; default is now Group post-G13-E"
    );
}
