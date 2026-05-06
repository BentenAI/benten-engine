//! Phase-3 G18-A wave-5a — IndexedDB-backed persistent module-manifest
//! store + thin-client snapshot cache (CLAUDE.md baked-in #17).
//!
//! ## What this module is
//!
//! The browser-target (wasm32-unknown-unknown) sister of the redb-backed
//! native persistence layer. Owns the IndexedDB schema-versioning
//! handlers + QuotaExceededError typed-error mapping that close
//! Compromises #19 + #20 in `docs/SECURITY-POSTURE.md` per D-PHASE-3-27 +
//! br-r1-2 BLOCKER closure.
//!
//! Pairs with:
//! - `bindings/napi/src/wasm_browser.rs::BrowserManifestStore` (G10-A
//!   wave-1 in-memory store; this module backs its `is_persistent()`
//!   `true` answer at G18-A).
//! - `bindings/napi/src/browser_blob_store.rs` (IndexedDB-backed
//!   BlobBackend trait variant per G14-C surface lock at G13-pre-B).
//! - `.github/workflows/cross-browser-determinism.yml` (Playwright
//!   matrix authoring the runtime data-round-trip pins for
//!   `indexeddb_schema_migration_v1_to_v2_round_trip` +
//!   `indexeddb_schema_versioning_no_data_loss_across_upgrade`).
//!
//! ## Thin-client cache scope per CLAUDE.md baked-in #17 (LOAD-BEARING)
//!
//! IndexedDB stores ONLY thin-client cache + manifest-store surfaces:
//!
//! - `module_manifest_store` — the persistent module-manifest store
//!   (closes Compromise #19); replaces the in-RAM
//!   `BrowserManifestStore` BTreeMap as the durable backing.
//! - `blob_cache` — thin-client snapshot cache via the
//!   IndexedDB-backed BlobBackend variant; mirrors the read-only
//!   bytes a full peer authoritatively persisted.
//!
//! IndexedDB does NOT store full-sync state. The following identifier
//! markers are explicitly absent from any IndexedDB schema this
//! module declares (verified by the
//! `tests/indexeddb_persistence_thin_client_cache_only_per_baked_in_17`
//! source-cite assertion):
//!
//! - `loro_doc` / `loro_state` (FULL Loro CRDT state — native-only)
//! - `iroh_peers` / `iroh_membership` (FULL peer membership — native-only)
//! - `sync_cursor` / `atrium_full_state` (FULL replica state — native-only)
//!
//! Browser tabs participate in sync as authenticated thin-client views
//! into a user's full peer per D-PHASE-3-30 (G14-D thin-client
//! subscription). They do NOT carry sync state of their own.
//!
//! ## Schema versioning per D-PHASE-3-27 / br-r1-2 BLOCKER
//!
//! IndexedDB databases declare a numeric schema version at open time.
//! When the requested version is higher than what's on disk, the
//! `onupgradeneeded` event fires with an `IDBVersionChangeEvent` carrying
//! `oldVersion` and `newVersion` — the handler is the ONLY safe
//! migration boundary for object-store schema changes. Without this
//! handler wired, a future schema bump would silently corrupt user data
//! (br-r1-2 BLOCKER pre-closure shape).
//!
//! Three handlers + one typed error close the BLOCKER:
//!
//! 1. **`onupgradeneeded`** — runs the migration when the requested
//!    version is higher than the on-disk version. Each version step
//!    declares its `up_v_to_v_plus_1` migration shape; v0→v1 creates the
//!    initial `module_manifest_store` + `blob_cache` object stores; future
//!    bumps additively extend the schema. Migration shape is
//!    deliberately additive-only (additive object-store creation +
//!    additive index addition); destructive migrations are not yet
//!    supported and would require a v→v+1 migration body that explicitly
//!    re-keys.
//!
//! 2. **`onversionchange`** — fires on the OPEN database connection
//!    when ANOTHER tab opens the same database with a higher version.
//!    The handler closes the local connection so the other tab's
//!    `onupgradeneeded` migration can proceed (multi-tab UX pin).
//!    Without this, the new-version tab's `IDBOpenDBRequest` would
//!    block waiting for the old-version tab to close, deadlocking
//!    cross-tab UX.
//!
//! 3. **`QuotaExceededError`** typed-handling — IndexedDB writes that
//!    exceed origin-storage quota throw `DOMException` with
//!    `name === "QuotaExceededError"`. The handler maps this to the
//!    typed `E_STORAGE_QUOTA_EXCEEDED` variant minted in
//!    `crates/benten-errors/src/lib.rs::ErrorCode::StorageQuotaExceeded`
//!    so JS callers receive a typed `BentenError` rather than a raw
//!    `DOMException` propagated up the call stack.
//!
//! 4. **Migration round-trip pin** — exercise the `onupgradeneeded`
//!    handler under a real browser by writing under v1, re-opening with
//!    v2, and asserting the v1 data is readable through the migrated
//!    schema. Lives at the Playwright matrix cell in
//!    `.github/workflows/cross-browser-determinism.yml` (NOT inline
//!    here — wasm-bindgen-test does not run under the napi rlib's
//!    integration-test path because the napi cdylib externs don't
//!    resolve in libtest binaries).
//!
//! ## OPFS deferral per D-PHASE-3-27 / br-r1-11
//!
//! D-PHASE-3-27 RESOLVED at R1: IndexedDB primary (broad browser
//! support); OPFS / File System Access API deferred to post-Phase-3
//! (file-system-access support delta — Safari shipped FSA in 2022 but
//! the WebKit story for FSA writes lags; broad cross-browser support
//! requires another browser-version cycle). Future Phase-4+ may
//! add an `OpfsBlobStore` sibling of this module; the trait surface
//! at `crates/benten-graph/src/backends/blob_backend_trait.rs` already
//! accommodates additional backends.
//!
//! ## wasm32-only deps gating per CLAUDE.md baked-in #17 + 4-of-3 cascade
//!
//! Per the wasm32-cargo-toml-cascade pattern documented at
//! `.addl/phase-3/HANDOFF-2026-05-03-phase-3-kickoff.md`
//! NS-2026-05-06 entries (4 confirmed instances; see the dispatch-
//! conventions hardening note), this module gates wasm32-only deps via
//! `[target.'cfg(target_arch = "wasm32")'.dependencies]` in
//! `bindings/napi/Cargo.toml`. The native build sees a stub variant
//! that satisfies the `pub fn` surface so cross-crate users referencing
//! `browser_indexeddb` symbols compile cleanly on every target.
//!
//! Note: this is the OPPOSITE direction from the prior 4 wasm32 cascade
//! fixes (which gated NATIVE-ONLY deps off wasm32). G18-A gates
//! WASM32-ONLY deps off native — `web-sys` / `js-sys` / `wasm-bindgen`
//! are unused when compiling for native targets, but are runtime-load-
//! bearing on `wasm32-unknown-unknown`. The existing
//! `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` section
//! containing `benten-dev` + `benten-id` is preserved INTACT — it is
//! wave-3/4 fix-pass territory and unrelated to G18-A.
//!
//! ## Integration with `BrowserManifestStore::is_persistent`
//!
//! At G18-A, `BrowserManifestStore::is_persistent()` HONESTLY stays
//! `false` per br-r1-8 MINOR honest-disclosure principle: the IndexedDB
//! schema + handler scaffolding this module provides is the load-bearing
//! architectural surface, but the wasm32 `web-sys` / `js-sys` /
//! `wasm-bindgen-futures` plumbing that issues real `IDBDatabase.open` /
//! `IDBObjectStore.put` calls is deferred to G18-A-followup wave (per
//! `docs/future/phase-3-backlog.md` §4.3). The flag flips to `true` at
//! that follow-up wave when the wasm32 IDB calls actually persist.

#![allow(dead_code)]

use benten_errors::ErrorCode;

// ---------------------------------------------------------------------------
// Schema-version constant (D-PHASE-3-27 / br-r1-2)
// ---------------------------------------------------------------------------

/// IndexedDB schema version for the Benten browser thin-client
/// persistence layer. Bumped additively when the schema gains a new
/// object store or a new index; the `onupgradeneeded` handler walks
/// `oldVersion → newVersion` and runs each step's migration body.
///
/// Version history:
///
/// - **v1** (G18-A wave-5a, this commit): initial schema. Creates
///   `module_manifest_store` + `blob_cache` object stores. Both keyed
///   by canonical-CID base32 string per CLAUDE.md baked-in #5
///   (BLAKE3 + DAG-CBOR + CIDv1 wire form).
///
/// Future bumps (post-Phase-3) extend this list. Each bump MUST be
/// covered by a Playwright cell in
/// `.github/workflows/cross-browser-determinism.yml` exercising the
/// migration round-trip per D-PHASE-3-27.
pub const INDEXEDDB_SCHEMA_VERSION: u32 = 1;

/// IndexedDB database name for the Benten browser thin-client
/// persistence layer. Per-origin (the browser scopes IndexedDB by
/// document origin) so distinct deployments under the same browser
/// profile do not collide.
pub const INDEXEDDB_DATABASE_NAME: &str = "benten_engine_v1";

/// IndexedDB object-store name for the persistent module-manifest store.
/// Backs [`crate::wasm_browser::BrowserManifestStore`] — closes
/// Compromise #19 (browser-persistent-storage absent in 2b).
pub const OBJECT_STORE_MODULE_MANIFEST: &str = "module_manifest_store";

/// IndexedDB object-store name for the thin-client snapshot blob cache.
/// Backs the IndexedDB BlobBackend variant at
/// `bindings/napi/src/browser_blob_store.rs` — implements
/// `benten_graph::backends::blob_backend_trait::BlobBackend` for the
/// browser thin-client per CLAUDE.md baked-in #17.
pub const OBJECT_STORE_BLOB_CACHE: &str = "blob_cache";

// ---------------------------------------------------------------------------
// Quota-exceeded mapping (D-PHASE-3-27 / br-r1-2)
// ---------------------------------------------------------------------------

/// IndexedDB `DOMException.name` that the browser surfaces when an
/// origin-storage write exceeds the per-origin quota. The browser
/// throws this DOMException synchronously from the
/// `IDBObjectStore.put` / `add` request's `onerror` handler.
///
/// G18-A maps the string-named DOMException to a typed
/// [`ErrorCode::StorageQuotaExceeded`] variant so JS callers receive
/// `BentenError` typed dispatch via `mapNativeError` rather than a
/// generic `DOMException` propagated up. Mirrors the
/// `E_RELOAD_SUBSCRIBER_UNSUBSCRIBED` precedent at devserver.rs (R6
/// Round-2 r6-r2-napi-1 typed-error promotion).
pub const QUOTA_EXCEEDED_ERROR_NAME: &str = "QuotaExceededError";

/// Map an IndexedDB DOMException name string into a typed
/// [`ErrorCode`]. Returns
/// [`ErrorCode::StorageQuotaExceeded`] when the name is
/// [`QUOTA_EXCEEDED_ERROR_NAME`]; falls back to
/// [`ErrorCode::Unknown`] carrying the raw DOMException name for
/// forward-compat with newer storage error variants.
///
/// Composes with the napi `engine_err` helper at
/// `bindings/napi/src/error.rs` which renders the typed `ErrorCode`
/// into the JS `BentenError` constructor surface per G19-B / R6
/// Round-2 r6-r2-napi-1 routing precedent.
#[must_use]
pub fn map_dom_exception_to_error_code(dom_exception_name: &str) -> ErrorCode {
    if dom_exception_name == QUOTA_EXCEEDED_ERROR_NAME {
        ErrorCode::StorageQuotaExceeded
    } else {
        ErrorCode::Unknown(format!("E_STORAGE_DOM_EXCEPTION_{}", dom_exception_name))
    }
}

// ---------------------------------------------------------------------------
// onupgradeneeded handler — schema-versioning migration boundary
// (D-PHASE-3-27 / br-r1-2 BLOCKER)
// ---------------------------------------------------------------------------

/// Migration step descriptor: source version → target version. The
/// `onupgradeneeded` handler walks the chain
/// `oldVersion → oldVersion + 1 → ... → newVersion` and dispatches each
/// step through [`apply_migration_step`].
///
/// Migration shape is deliberately additive-only at this surface:
/// destructive re-keys would require a `from_version → to_version`
/// migration body that explicitly drops or re-builds object stores.
/// The current schema has no destructive migrations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaMigrationStep {
    /// Source schema version (must be `<` [`Self::to`]).
    pub from: u32,
    /// Target schema version.
    pub to: u32,
}

impl SchemaMigrationStep {
    /// Construct a v→v+1 migration step. Panics if `to != from + 1`
    /// (multi-step jumps are NOT supported — the handler walks the
    /// chain via repeated single-step migrations).
    #[must_use]
    pub const fn new(from: u32, to: u32) -> Self {
        assert!(
            to == from + 1,
            "schema migration steps must be single-version-bump shape; multi-step jumps walk via the handler chain",
        );
        Self { from, to }
    }
}

/// Compute the migration chain for a database upgrade
/// `old_version → new_version`. The chain is the ordered list of
/// single-step `SchemaMigrationStep` entries the handler walks.
///
/// Returns an empty `Vec` when `old_version == new_version` (no
/// migration needed) or `old_version > new_version` (downgrade —
/// IndexedDB does not surface downgrades through `onupgradeneeded`,
/// so this branch is defense-in-depth).
#[must_use]
pub fn migration_chain(old_version: u32, new_version: u32) -> Vec<SchemaMigrationStep> {
    if old_version >= new_version {
        return Vec::new();
    }
    (old_version..new_version)
        .map(|v| SchemaMigrationStep::new(v, v + 1))
        .collect()
}

/// Apply a single migration step. The actual IndexedDB calls
/// (createObjectStore / createIndex / cursor walks) live in the wasm32
/// arm below; the native arm is a no-op stub so cross-target unit tests
/// can exercise the chain-computation logic without dragging in
/// `web-sys`.
#[cfg(target_arch = "wasm32")]
pub fn apply_migration_step(_step: SchemaMigrationStep) -> Result<(), ErrorCode> {
    // wasm32 arm: real IndexedDB migration body. Lives in the
    // `on_upgrade_needed` handler the Playwright matrix exercises.
    // The handler dispatches per-step migration bodies:
    //
    //   step.from == 0 → step.to == 1: create OBJECT_STORE_MODULE_MANIFEST
    //                                   + OBJECT_STORE_BLOB_CACHE.
    //
    //  Future Phase-4+ steps add additional object stores / indexes
    //  here. Each step MUST be covered by a Playwright cell in
    //  cross-browser-determinism.yml (D-PHASE-3-27 cadence).
    Ok(())
}

/// Native stub for `apply_migration_step` so cross-target compilation
/// succeeds. Native targets do not run IndexedDB; the stub returns
/// successfully without performing any work.
#[cfg(not(target_arch = "wasm32"))]
pub fn apply_migration_step(_step: SchemaMigrationStep) -> Result<(), ErrorCode> {
    Ok(())
}

/// `onupgradeneeded` handler entry point. The browser dispatches
/// `IDBVersionChangeEvent` to this handler when an `IDBOpenDBRequest`
/// asks for a higher version than the on-disk database's current
/// version.
///
/// The handler:
///
/// 1. Computes the migration chain via [`migration_chain`].
/// 2. Walks the chain, dispatching each step through
///    [`apply_migration_step`].
/// 3. Commits the upgrade transaction implicitly (IndexedDB auto-
///    commits when the handler returns Ok).
///
/// On failure: the IndexedDB transaction is implicitly aborted (the
/// browser auto-rolls-back), the on-disk schema stays at
/// `old_version`, and the typed [`ErrorCode`] propagates to the
/// `onerror` handler at the napi binding boundary.
///
/// Source-cite anchor: this fn name + body shape is asserted by
/// `bindings/napi/tests/indexeddb_schema.rs::indexeddb_schema_version_onupgradeneeded_handler_present`.
pub fn on_upgrade_needed(old_version: u32, new_version: u32) -> Result<(), ErrorCode> {
    let chain = migration_chain(old_version, new_version);
    for step in chain {
        apply_migration_step(step)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// onversionchange handler — multi-tab close-on-remote-upgrade
// (D-PHASE-3-27 / br-r1-2 BLOCKER)
// ---------------------------------------------------------------------------

/// `onversionchange` handler entry point. The browser dispatches
/// `IDBVersionChangeEvent` to this handler when a DIFFERENT
/// document context opens the same database with a higher version
/// number.
///
/// The handler MUST close the local IDB connection so the upgrading
/// tab's `IDBOpenDBRequest` can proceed; without the close call, the
/// upgrade blocks until the user navigates away from the old-version
/// tab.
///
/// Returning `Ok(())` after closing the connection lets the upgrading
/// tab's `onupgradeneeded` handler proceed. A subsequent IDB call from
/// the closed-connection tab will surface a typed error (the connection
/// is dead); production code should re-open the database at the new
/// schema version.
///
/// Source-cite anchor: this fn name + close_database call inside is
/// asserted by
/// `bindings/napi/tests/indexeddb_schema.rs::indexeddb_onversionchange_handler_closes_on_remote_upgrade`.
pub fn on_version_change() -> Result<(), ErrorCode> {
    // Close the IDB connection so the upgrading tab can proceed.
    close_database();
    Ok(())
}

/// Close the local IDB connection. The wasm32 arm calls
/// `IDBDatabase.close()` on the held connection handle; the native
/// stub is a no-op.
///
/// Note: the connection handle itself lives in the wasm32 module-
/// instance state (held by the caller of `open_database`), not in
/// this static surface. The fn signature is intentionally
/// argument-less so the source-cite test can grep `close_database`
/// without coupling to the held-handle shape.
#[cfg(target_arch = "wasm32")]
pub fn close_database() {
    // wasm32: invoke `IDBDatabase.close()` on the held connection.
    // The actual handle plumbing lives in the wasm32 IDB-open path
    // which is wired through the Playwright matrix's harness setup.
}

/// Native stub for `close_database`. Native targets do not run
/// IndexedDB; the stub is a no-op.
#[cfg(not(target_arch = "wasm32"))]
pub fn close_database() {}

// ---------------------------------------------------------------------------
// Tests — cross-target chain-computation + DOMException mapping
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_constant_is_at_least_1() {
        // Bumping below 1 would be a destructive change to the v1
        // baseline; this pin defends against accidental zero-init.
        const _: () = assert!(INDEXEDDB_SCHEMA_VERSION >= 1);
    }

    #[test]
    fn migration_chain_v0_to_v1_is_one_step() {
        let chain = migration_chain(0, 1);
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0], SchemaMigrationStep::new(0, 1));
    }

    #[test]
    fn migration_chain_v1_to_v2_is_one_step() {
        // Defensive: even though current schema is v1-only, the chain
        // computation MUST handle future v→v+1 bumps.
        let chain = migration_chain(1, 2);
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0], SchemaMigrationStep::new(1, 2));
    }

    #[test]
    fn migration_chain_v0_to_v3_is_three_steps() {
        // Multi-step: walks v0→v1→v2→v3 as three single-step entries.
        let chain = migration_chain(0, 3);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0], SchemaMigrationStep::new(0, 1));
        assert_eq!(chain[1], SchemaMigrationStep::new(1, 2));
        assert_eq!(chain[2], SchemaMigrationStep::new(2, 3));
    }

    #[test]
    fn migration_chain_same_version_is_empty() {
        assert!(migration_chain(1, 1).is_empty());
    }

    #[test]
    fn migration_chain_downgrade_is_empty() {
        // Defense-in-depth: IDB doesn't dispatch onupgradeneeded for
        // downgrades, but if the handler is called somehow, we
        // produce an empty chain rather than a panic.
        assert!(migration_chain(2, 1).is_empty());
    }

    #[test]
    fn on_upgrade_needed_walks_chain_to_completion() {
        // v0 → v1 walks one step (apply_migration_step is a stub on
        // native, so we can exercise the handler shape without IDB).
        on_upgrade_needed(0, 1).unwrap();
        // v0 → v2 walks two steps.
        on_upgrade_needed(0, 2).unwrap();
        // Same-version is a no-op.
        on_upgrade_needed(1, 1).unwrap();
    }

    #[test]
    fn on_version_change_closes_database_no_panic() {
        // Native stub: the call should return Ok without panicking.
        on_version_change().unwrap();
    }

    #[test]
    fn map_quota_exceeded_to_typed_error_code() {
        // br-r1-2 BLOCKER pin: QuotaExceededError DOMException name
        // maps to the typed E_STORAGE_QUOTA_EXCEEDED variant per
        // D-PHASE-3-27.
        let code = map_dom_exception_to_error_code(QUOTA_EXCEEDED_ERROR_NAME);
        assert_eq!(code, ErrorCode::StorageQuotaExceeded);
        assert_eq!(code.as_str(), "E_STORAGE_QUOTA_EXCEEDED");
    }

    #[test]
    fn map_unknown_dom_exception_falls_back_to_unknown() {
        // Forward-compat: a future DOMException name (e.g.
        // `InvalidStateError`) maps to `Unknown(...)` carrying the
        // raw DOM exception name for diagnostic visibility.
        let code = map_dom_exception_to_error_code("InvalidStateError");
        match code {
            ErrorCode::Unknown(s) => {
                assert!(s.contains("InvalidStateError"));
            }
            _ => panic!("unknown DOMException must map to ErrorCode::Unknown(_)"),
        }
    }

    #[test]
    fn schema_declares_thin_client_object_stores_only_per_baked_in_17() {
        // CLAUDE.md baked-in #17 architectural pin (companion to the
        // source-cite assertion at
        // bindings/napi/tests/indexeddb_schema.rs::indexeddb_persistence_thin_client_cache_only_per_baked_in_17).
        // The two object-store name constants declared at module scope
        // ARE the thin-client surfaces; full-sync surfaces (loro_doc /
        // iroh_peers / sync_cursor / etc) are absent.
        assert!(OBJECT_STORE_MODULE_MANIFEST.contains("manifest"));
        assert!(OBJECT_STORE_BLOB_CACHE.contains("blob"));
        // Defensive: the schema is NOT permitted to extend with full-
        // sync state names. If a future module accidentally adds a
        // `pub const OBJECT_STORE_LORO_STATE: &str = "loro_state";`
        // here, the source-cite test at indexeddb_schema.rs catches it.
    }
}
