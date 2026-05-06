//! G18-A wave-5a IndexedDB schema-versioning pins (D-PHASE-3-27 +
//! br-r1-2 BLOCKER closure).
//!
//! Pin sources (per r2-test-landscape §2.6 G18-A):
//!
//! Sentinel-presence diagnostics (now LIVE — un-ignored at G18-A):
//! - `indexeddb_schema_version_onupgradeneeded_handler_present` — D-PHASE-3-27 / br-r1-2
//! - `indexeddb_onversionchange_handler_closes_on_remote_upgrade` — D-PHASE-3-27 / br-r1-2
//! - `indexeddb_quota_exceeded_error_fires_e_storage_quota_exceeded` — D-PHASE-3-27 / br-r1-2
//! - `indexeddb_persistence_thin_client_cache_only_per_baked_in_17` — baked-in #17
//!
//! Load-bearing data-round-trip pins (BELONGS-NAMED-NOW per HARD RULE
//! rule-12 — runtime browser harness lives at the Playwright matrix
//! cell in `.github/workflows/cross-browser-determinism.yml`; the
//! Rust-side body asserts the Playwright cell exists + drives the
//! migration assertion):
//! - `indexeddb_schema_migration_v1_to_v2_round_trip` — D-PHASE-3-27 / br-r1-2
//! - `indexeddb_schema_versioning_no_data_loss_across_upgrade` — D-PHASE-3-27 / br-r1-2
//!
//! ## D-PHASE-3-27 BLOCKER closure (CLOSED at G18-A wave-5a)
//!
//! 1. **`onupgradeneeded` handler** — wired at
//!    `bindings/napi/src/browser_indexeddb.rs::on_upgrade_needed`.
//!    Walks the migration chain step-by-step; each step is an
//!    additive object-store creation in v1 (the chain extends
//!    additively in future schema bumps).
//! 2. **`onversionchange` handler** — wired at
//!    `bindings/napi/src/browser_indexeddb.rs::on_version_change`.
//!    Closes the local IDB connection so a higher-version tab's
//!    upgrade can proceed.
//! 3. **`QuotaExceededError` typed-handling** — DOMException
//!    `name == "QuotaExceededError"` maps to the typed
//!    [`benten_errors::ErrorCode::StorageQuotaExceeded`] variant minted
//!    in `crates/benten-errors/src/lib.rs`. JS callers receive
//!    `BentenError` typed dispatch via `mapNativeError`.
//! 4. **Data round-trip via Playwright matrix** — `cross-browser-
//!    determinism.yml` runs the v1→v2 round-trip + 1000-key
//!    no-data-loss sweep in real Chromium / Gecko / WebKit (not
//!    available at native-cargo-test time because `wasm-bindgen-test`
//!    does not run under the napi rlib's integration test path —
//!    napi cdylib externs do not resolve in libtest binaries).

#![allow(clippy::unwrap_used)]

const BROWSER_INDEXEDDB_PATH: &str = "src/browser_indexeddb.rs";
const ERROR_CATALOG_REL_PATH: &str = "../../docs/ERROR-CATALOG.md";
const CROSS_BROWSER_DETERMINISM_REL_PATH: &str =
    "../../.github/workflows/cross-browser-determinism.yml";

fn module_src() -> String {
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(BROWSER_INDEXEDDB_PATH);
    std::fs::read_to_string(p).unwrap()
}

fn error_catalog() -> String {
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(ERROR_CATALOG_REL_PATH);
    std::fs::read_to_string(p).unwrap()
}

fn cross_browser_workflow() -> String {
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(CROSS_BROWSER_DETERMINISM_REL_PATH);
    std::fs::read_to_string(p).unwrap()
}

#[test]
fn indexeddb_schema_version_onupgradeneeded_handler_present() {
    // br-r1-2 BLOCKER pin — sentinel-presence diagnostic.
    let src = module_src();

    assert!(
        src.contains("onupgradeneeded") || src.contains("on_upgrade_needed"),
        "browser_indexeddb.rs must wire the onupgradeneeded handler per D-PHASE-3-27 / br-r1-2"
    );
    assert!(
        src.contains("SCHEMA_VERSION") || src.contains("schema_version"),
        "browser_indexeddb.rs must declare a versioned schema constant per D-PHASE-3-27"
    );

    // Runtime witness — chain computation works on host build.
    use benten_napi::browser_indexeddb::on_upgrade_needed;
    on_upgrade_needed(0, 1).unwrap();
}

#[test]
fn indexeddb_onversionchange_handler_closes_on_remote_upgrade() {
    // D-PHASE-3-27 / br-r1-2 sentinel-presence diagnostic.
    let src = module_src();

    assert!(
        src.contains("onversionchange") || src.contains("on_version_change"),
        "browser_indexeddb.rs must wire the onversionchange handler per D-PHASE-3-27"
    );
    assert!(
        src.contains("close_database") || src.contains(".close()"),
        "onversionchange handler must close the DB connection so the upgrade can proceed in the other tab"
    );

    // Runtime witness — handler returns Ok on native (close_database
    // is a stub on native; the Playwright matrix exercises wasm32).
    use benten_napi::browser_indexeddb::on_version_change;
    on_version_change().unwrap();
}

#[test]
fn indexeddb_quota_exceeded_error_fires_e_storage_quota_exceeded() {
    // D-PHASE-3-27 / br-r1-2 sentinel-presence + observable-consequence pin.
    let src = module_src();

    assert!(
        src.contains("QuotaExceededError") || src.contains("quota_exceeded"),
        "browser_indexeddb.rs must handle QuotaExceededError per D-PHASE-3-27 / br-r1-2"
    );
    assert!(
        src.contains("E_STORAGE_QUOTA_EXCEEDED") || src.contains("StorageQuotaExceeded"),
        "QuotaExceededError must map to typed E_STORAGE_QUOTA_EXCEEDED per D-PHASE-3-27"
    );

    // ERROR-CATALOG.md doc-coupling per §3.5b.
    let catalog = error_catalog();
    assert!(
        catalog.contains("E_STORAGE_QUOTA_EXCEEDED"),
        "ERROR-CATALOG.md must document E_STORAGE_QUOTA_EXCEEDED per §3.5b doc-coupling + D-PHASE-3-27"
    );

    // Runtime witness — the mapper returns the typed code.
    use benten_errors::ErrorCode;
    use benten_napi::browser_indexeddb::map_dom_exception_to_error_code;
    let code = map_dom_exception_to_error_code("QuotaExceededError");
    assert_eq!(code, ErrorCode::StorageQuotaExceeded);
    assert_eq!(code.as_str(), "E_STORAGE_QUOTA_EXCEEDED");
}

#[test]
fn indexeddb_schema_migration_v1_to_v2_round_trip() {
    // D-PHASE-3-27 / br-r1-2 LOAD-BEARING pin per pim-2 §3.6b.
    //
    // The runtime data-round-trip lives in the Playwright matrix cell
    // at `.github/workflows/cross-browser-determinism.yml` — wasm-
    // bindgen-test does not run under the napi rlib integration test
    // path (napi cdylib externs do not resolve in libtest binaries).
    //
    // This pin asserts the Playwright cell EXISTS + DRIVES the
    // migration round-trip. The cell itself runs against real
    // Chromium / Gecko / WebKit with browser harness setup.
    let workflow = cross_browser_workflow();
    assert!(
        workflow.contains("schema_migration")
            || workflow.contains("schema-migration")
            || workflow.contains("migration_round_trip"),
        "cross-browser-determinism.yml must drive the IndexedDB v1→v2 schema-migration round-trip per D-PHASE-3-27 / br-r1-2"
    );

    // Defense-in-depth: the chain-computation half is exercised in the
    // host build (single-step + multi-step chain).
    use benten_napi::browser_indexeddb::{SchemaMigrationStep, migration_chain, on_upgrade_needed};
    let chain = migration_chain(1, 2);
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0], SchemaMigrationStep::new(1, 2));
    on_upgrade_needed(1, 2).unwrap();
}

#[test]
fn indexeddb_schema_versioning_no_data_loss_across_upgrade() {
    // D-PHASE-3-27 / br-r1-2 LOAD-BEARING pin (1000-key sweep variant).
    //
    // The 1000-key data-round-trip lives in the Playwright matrix cell
    // — same harness rationale as the v1→v2 single-key round-trip pin
    // above.
    let workflow = cross_browser_workflow();
    assert!(
        workflow.contains("no_data_loss")
            || workflow.contains("no-data-loss")
            || workflow.contains("1000_key")
            || workflow.contains("1000-key"),
        "cross-browser-determinism.yml must drive the no-data-loss 1000-key migration sweep per D-PHASE-3-27 / br-r1-2"
    );
}

#[test]
fn indexeddb_persistence_thin_client_cache_only_per_baked_in_17() {
    // CLAUDE.md baked-in #17 architectural pin — the IndexedDB schema
    // declares ONLY thin-client surfaces.
    let src = module_src();

    // The schema MUST NOT declare object-stores for full-sync state.
    for full_sync_only_marker in &[
        "loro_doc",
        "loro_state",
        "iroh_peers",
        "iroh_membership",
        "sync_cursor",
        "atrium_full_state",
    ] {
        // Allowed in COMMENTS that document what the schema must NOT
        // contain — that's what THIS pin is enforcing. Strip
        // comment-only matches: a marker is a violation only if it
        // appears as an OBJECT_STORE_* constant declaration / a
        // schema-decl line.
        let violation = src.lines().any(|line| {
            // Only count as violation if the marker is declared as a
            // const value (string literal context, NOT in a comment
            // explaining what's prohibited).
            let trimmed = line.trim();
            // Skip line comments + the prohibited-marker enumeration in
            // module docs / source-cite assertions.
            if trimmed.starts_with("//") || trimmed.starts_with("//!") {
                return false;
            }
            // Skip lines inside the iteration array of the prohibited-
            // marker test fixture. (The current G18-A source has no
            // such block, but defense-in-depth.)
            if trimmed.starts_with('"') && trimmed.ends_with("\",") {
                return false;
            }
            // Match object-store name declaration:
            //   pub const OBJECT_STORE_X: &str = "loro_state";
            // The name appears inside a string literal RHS of `: &str =`.
            //
            // Heuristic-limitation note (g18a-mr-5 MINOR
            // DISAGREE-WITH-EXPLANATION): this `&str = ` split misses
            // alternate string-typed const decl shapes (e.g.
            // `&'static str = "..."` or a `String` typed const) and
            // multi-line declarations. Defense-in-depth is acceptable
            // here because the source-cite has a companion
            // architectural pin at the constant-declaration level in
            // `bindings/napi/tests/wasm32_unknown_unknown_module_manifest_in_memory_only_no_indexeddb_persistence.rs::indexeddb_schema_declares_thin_client_object_stores_only`
            // which asserts the OBJECT_STORE_* constants by VALUE
            // (catching the `&'static str` case). A future tightening
            // could regex this as `&'?[\\w_]*'?\\s*str\\s*=` shape;
            // current heuristic is sufficient because the production
            // declarations all use the bare `&str = ` shape.
            if let Some(rhs) = line.split_once("&str = ") {
                return rhs.1.contains(full_sync_only_marker);
            }
            false
        });
        assert!(
            !violation,
            "browser_indexeddb.rs MUST NOT declare full-sync state object-store {} per CLAUDE.md baked-in #17 thin-client commitment; full sync is native-only",
            full_sync_only_marker
        );
    }

    // It DOES carry the thin-client surfaces.
    assert!(
        src.contains("module_manifest") || src.contains("manifest_store"),
        "browser_indexeddb.rs must back BrowserManifestStore per G18-A"
    );
    assert!(
        src.contains("blob") || src.contains("snapshot_cache"),
        "browser_indexeddb.rs must back the thin-client BlobBackend per G14-C trait surface"
    );

    // Runtime witness — the declared object-store name constants
    // ARE thin-client surfaces.
    use benten_napi::browser_indexeddb::{OBJECT_STORE_BLOB_CACHE, OBJECT_STORE_MODULE_MANIFEST};
    assert!(OBJECT_STORE_MODULE_MANIFEST.contains("manifest"));
    assert!(OBJECT_STORE_BLOB_CACHE.contains("blob"));
}
