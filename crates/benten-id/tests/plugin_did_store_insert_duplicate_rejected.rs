//! R6-FP-3 (Phase-4-Foundation R6 R3 close — cap-r6-r3-1 defensive-return
//! hardening) substantive arm test pin for `PluginDidStore::insert` returning
//! `Err(ErrorCode::PluginDidHandleDuplicate)` when a handle whose DID
//! byte-equals an already-present handle is inserted.
//!
//! # What this asserts (pim-2 §3.6b PRODUCTION-ARM + OBSERVABLE-CONSEQUENCE
//! + WOULD-FAIL-IF-NO-OP'd):
//!
//! 1. **PRODUCTION-ARM**: drives the real production `PluginDidStore::insert`
//!    at `crates/benten-id/src/plugin_did.rs:insert` directly. The duplicate
//!    case is exercised via the test-only `handle_with_did_for_test`
//!    constructor (gated behind `cfg(any(test, feature = "testing"))`).
//! 2. **OBSERVABLE-CONSEQUENCE**: asserts the second insert returns
//!    `Err(ErrorCode::PluginDidHandleDuplicate)` AND the store state stays
//!    at one handle. Pre-R6-FP-3 the second insert silently succeeded
//!    (return type was `()`) — the new error surface + state preservation
//!    are the observable behavioral differences.
//! 3. **WOULD-FAIL-IF-NO-OP'd**: commenting out the duplicate-check at
//!    `plugin_did.rs:insert` would cause this test to fail — the second
//!    insert would push a duplicate handle (store.len() == 2) + return Ok,
//!    contradicting both the assert_eq Err assertion and the len() assert.

use benten_errors::ErrorCode;
use benten_id::plugin_did::{PluginDidStore, handle_with_did_for_test, mint};

#[test]
fn plugin_did_store_insert_rejects_duplicate_did() {
    let mut store = PluginDidStore::new();

    // First insert — fresh-mint handle.
    let first = mint();
    let shared_did = first.did().clone();
    store
        .insert(first)
        .expect("first insert with unique DID should succeed");
    assert_eq!(store.len(), 1);
    assert!(store.get(&shared_did).is_some());

    // Second insert — handle whose DID byte-equals the first.
    let duplicate = handle_with_did_for_test(shared_did.clone());
    let result = store.insert(duplicate);

    // OBSERVABLE-CONSEQUENCE: defensive-return must fire.
    assert_eq!(
        result,
        Err(ErrorCode::PluginDidHandleDuplicate),
        "PluginDidStore::insert MUST return Err(PluginDidHandleDuplicate) \
         on duplicate DID per R6-FP-3 cap-r6-r3-1 defensive-return hardening"
    );

    // STATE PRESERVATION: store unchanged.
    assert_eq!(
        store.len(),
        1,
        "duplicate insert MUST NOT mutate store; len stays at 1"
    );

    // Sanity: original handle still present.
    assert!(store.get(&shared_did).is_some());
}

#[test]
fn plugin_did_store_insert_accepts_distinct_dids() {
    // Negative control: distinct DIDs must succeed (duplicate-rejection
    // must not over-trigger).
    let mut store = PluginDidStore::new();

    let h1 = mint();
    let did1 = h1.did().clone();
    store.insert(h1).expect("first distinct DID");

    let h2 = mint();
    let did2 = h2.did().clone();
    assert_ne!(
        did1, did2,
        "mint() must produce distinct DIDs (OsRng invariant)"
    );
    store.insert(h2).expect("second distinct DID");

    assert_eq!(store.len(), 2);
    assert!(store.get(&did1).is_some());
    assert!(store.get(&did2).is_some());
}

#[test]
fn plugin_did_handle_duplicate_error_string_canonical() {
    // §3.5g cross-language rule-mirror — ErrorCode catalog 4-surface
    // shape check: the variant maps to the expected canonical string.
    // Mirror sites: Rust enum (lib.rs) + as_str (lib.rs) + matches_static
    // (lib.rs) + ALL_CATALOG_VARIANTS (stable_shape.rs) + TS catalog
    // (errors.generated.ts) + ERROR-CATALOG.md preamble.
    let code = ErrorCode::PluginDidHandleDuplicate;
    assert_eq!(code.as_str(), "E_PLUGIN_DID_HANDLE_DUPLICATE");
}
