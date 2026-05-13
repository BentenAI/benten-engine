//! Phase 4-Foundation R3 (Family A — R1-FP wave-1 G22-FP-3 regression-
//! defense). Grep-assert paired with the
//! `crates/benten-caps/tests/grant_backed_policy_check_read_denies_when_actor_cid_lacks_cap.rs`
//! production-runtime acceptance test (SHIPPED at PR #209 closing
//! cap-r1-2 + cap-r1-10 BLOCKERs).
//!
//! # Charter
//!
//! Per R3 dispatch brief (Family A R1-FP regression-defense section).
//! The G22-FP-3 closure landed the principal-aware read check inside
//! `BackendGrantReader::has_unrevoked_grant_for_scope_and_actor` at
//! `crates/benten-engine/src/builder.rs`. The first-round implementation
//! during PR #209 carried a property-name BUG: it consulted
//! `node.properties.get("grantee")` while the engine's grant-write path
//! (`Engine::grant_capability_with_proof` at
//! `crates/benten-engine/src/engine_caps.rs:186`) stores the principal
//! under the property name `"actor"`. The mismatch caused the
//! principal-aware filter to always skip — degenerate to "no grant
//! matches" — silently denying every principal-bound read. The
//! property-name was corrected to `"actor"` in PR #209 before merge.
//!
//! # What this pin asserts (would-FAIL-if-no-op'd per §3.6b)
//!
//! Source-level invariants on `crates/benten-engine/src/builder.rs`:
//!
//! 1. The `BackendGrantReader` body MUST read the principal-binding
//!    property by the literal name `"actor"` (matching the write side
//!    at `engine_caps.rs::grant_capability_with_proof` —
//!    `props.insert("actor".into(), actor.as_value())`).
//! 2. The `BackendGrantReader` body MUST NOT read the principal-
//!    binding property by the legacy/buggy name `"grantee"` —
//!    that property name is NOT what the engine's write path stores.
//!
//! Reverting the field name back to `"grantee"` re-introduces the
//! PR #209-first-round regression — the production-runtime test
//! `grant_backed_policy_check_read_denies_when_actor_cid_lacks_cap`
//! would catch it at runtime; this test catches it at source-level
//! for defense-in-depth.
//!
//! # Cross-references
//!
//! - **Write side:** `crates/benten-engine/src/engine_caps.rs::grant_capability_with_proof`
//!   stores `props.insert("actor".into(), actor.as_value())`.
//! - **Read side:** `crates/benten-engine/src/builder.rs::BackendGrantReader::has_unrevoked_grant_for_scope_and_actor`
//!   reads `node.properties.get("actor")`.
//! - **Production-runtime arm:** `crates/benten-caps/tests/grant_backed_policy_check_read_denies_when_actor_cid_lacks_cap.rs`.
//!
//! # Status
//!
//! NOT RED-PHASE — this is a regression-defense pin guarding a SHIPPED
//! closure (PR #209). Runs unconditionally in CI; fails if anyone
//! reverts to the buggy `"grantee"` property-name read.
//!
//! # Owned by
//!
//! Phase 4-Foundation R3 Family A test-writer (regression-defense set).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn builder_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("builder.rs");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

fn engine_caps_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("engine_caps.rs");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

#[test]
fn backend_grant_reader_reads_actor_property_name_not_grantee() {
    let body = builder_source();
    assert!(
        body.contains("properties.get(\"actor\")"),
        "expected `node.properties.get(\"actor\")` in \
         crates/benten-engine/src/builder.rs (cap-r1-2 + cap-r1-10 \
         regression-defense): the principal-aware read check MUST use \
         the same property name (`actor`) that the write path stores. \
         A revert to `\"grantee\"` re-introduces the PR #209-first-round \
         BUG where the principal filter always skipped because the \
         property-name didn't match the durable grant Node's stored \
         field — every principal-bound read silently DENIED.",
    );
}

#[test]
fn backend_grant_reader_does_not_read_grantee_property_name() {
    let body = builder_source();
    // We're looking for the buggy field read in BackendGrantReader.
    // The string `grantee` may legitimately appear in docstrings /
    // narrative comments (the legacy buggy name is a historical
    // reference), so we don't ban the string outright — we ban the
    // SUBSTANTIVE access pattern `properties.get("grantee")`.
    assert!(
        !body.contains("properties.get(\"grantee\")"),
        "found `node.properties.get(\"grantee\")` in \
         crates/benten-engine/src/builder.rs — cap-r1-2 regression \
         (PR #209-first-round BUG): the engine's write path stores \
         the principal as `\"actor\"`, NOT `\"grantee\"`. Reading the \
         wrong field name silently fails every principal-bound \
         check_read. Restore the `actor` property-name read; re-run \
         the `grant_backed_policy_check_read_denies_when_actor_cid_lacks_cap` \
         production-runtime arm.",
    );
}

#[test]
fn engine_caps_grant_capability_stores_actor_property_name() {
    let body = engine_caps_source();
    // Pin the symmetric write side: the
    // `Engine::grant_capability_with_proof` body must write
    // `"actor"` as the principal property name. If the write side
    // ever drifts to `"grantee"`, the read side's `"actor"` lookup
    // would silently fail every principal-bound check — same
    // observable consequence, different drift direction.
    assert!(
        body.contains("\"actor\".into()"),
        "expected `props.insert(\"actor\".into(), actor.as_value())` in \
         crates/benten-engine/src/engine_caps.rs::grant_capability_with_proof \
         (cap-r1-2 regression-defense, write-side mirror): the principal \
         binding property name is `\"actor\"`. If the write side drifts to \
         `\"grantee\"` while the read side stays on `\"actor\"`, every \
         principal-bound read denies silently. Pair this with the \
         BackendGrantReader read-side defense to pin the cross-file \
         property-name agreement.",
    );
}
