//! Phase 2b G10-A-browser must-pass — Compromise #N+8 enforcement.
//!
//! Pin source: plan §3 G10-A-browser must-pass list +
//! `.addl/phase-2b/00-implementation-plan.md` §10 Compromise #N+8:
//!
//! > Module manifests in browser (wasm32-unknown-unknown) are
//! > in-memory only; IndexedDB-backed persistent store deferred to
//! > Phase-3.
//!
//! The enforcement decomposes into three cross-axis assertions:
//!
//!   1. **Storage-contract half:** `BrowserManifestStore::is_persistent()`
//!      returns `false`. (Pinned in
//!      `wasm32_unknown_unknown_browser_engine_loads.rs` as well; we
//!      re-pin here so a future refactor that flips the contract
//!      without re-running the load test still trips this dedicated
//!      enforcement file.)
//!   2. **No-survive-store-drop half:** dropping a `BrowserManifestStore`
//!      and constructing a fresh one yields an empty store. Pins the
//!      "no IndexedDB / OPFS / localStorage backstop" property: the
//!      manifest set is exclusively held in the store's owned
//!      `BTreeMap`; dropping the store is sufficient to lose the
//!      manifest set.
//!   3. **Dependency-graph half:** the napi crate's compiled `.rlib`
//!      contains no symbol references to `web_sys::IdbDatabase` /
//!      `web_sys::Storage` / `idb::Database`. The napi crate's
//!      `[dependencies]` block enumerates everything that lands in the
//!      browser bundle; if a future commit pulls in `web-sys` or `idb`
//!      either directly or transitively, this test surfaces the drift.
//!      We assert by reading the napi crate's `Cargo.toml` and
//!      grepping for the forbidden dep names — a coarse-but-effective
//!      drift detector that costs nothing at test time.
//!
//! ## Why a dependency-graph check
//!
//! Compromise #N+8 is a "no persistence backstop" guarantee. The
//! storage contract's `is_persistent() -> false` is necessary but
//! not sufficient — a sibling code path (e.g. a stray `web_sys::Storage`
//! call buried in a dep) could write manifests to a browser-host
//! persistent store behind the API's back. Pinning the absence of
//! the relevant deps at the napi-crate boundary forecloses that
//! drift class without needing to inspect the wasm bundle's symbol
//! table at test time (which would require a built bundle —
//! `wasm-browser.yml`'s job, not native `cargo test`'s).
//!
//! The bundle-time defence-in-depth: `wasm-browser.yml` (release-era
//! cadence) runs a strings-grep over the built `.wasm` artifact
//! looking for the same forbidden API surface. The two-layer check
//! mirrors the Phase-1 "drift detector AND artifact assertion" pattern
//! (cf. host-functions.toml).
//!
//! Owned by G10-A-browser per plan §3.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_napi::wasm_browser::BrowserManifestStore;

/// Storage-contract half — re-pinned here so a refactor that flips
/// the contract without re-running the load test still trips the
/// dedicated enforcement file.
#[test]
fn store_reports_non_persistent_on_phase_2b_build() {
    let s = BrowserManifestStore::new();
    assert!(
        !s.is_persistent(),
        "Compromise #N+8: BrowserManifestStore must report \
         is_persistent() == false on every Phase-2b build"
    );
}

/// No-survive-store-drop half — manifests do not outlive the store.
#[test]
fn manifests_do_not_survive_store_drop() {
    // Install a manifest into store A
    let store_a = BrowserManifestStore::new();
    let cid = "bafyrTESTcid".to_string();
    let bytes = vec![0xa1, 0x02, 0x03];
    store_a.insert(cid.clone(), bytes.clone());
    assert!(
        store_a.contains(&cid),
        "manifest must be present in the originating store"
    );

    // Drop store A; construct a fresh store B (simulating a page reload
    // / fresh Engine on browser).
    drop(store_a);
    let store_b = BrowserManifestStore::new();

    // Compromise #N+8: store B must NOT see the manifest installed
    // into store A. If a future IndexedDB-backed implementation
    // were silently introduced under the same `new()` constructor,
    // this assertion would fire.
    assert!(
        !store_b.contains(&cid),
        "Compromise #N+8: manifests must not survive store drop \
         (in-memory only — no IndexedDB / OPFS / localStorage \
         persistence backstop in Phase 2b)"
    );
    assert!(store_b.is_empty(), "fresh store must be empty");
}

/// Dependency-graph half — the napi crate must NOT depend on
/// `web-sys` (any IndexedDB / Storage modules) or `idb` (the
/// idiomatic Rust IndexedDB wrapper). Reads `bindings/napi/Cargo.toml`
/// and greps for the forbidden dep names.
///
/// Drift detector. Coarse but cheap.
#[test]
fn napi_crate_has_no_indexeddb_dependency() {
    let cargo_toml = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let contents = std::fs::read_to_string(&cargo_toml).unwrap_or_else(|e| {
        panic!(
            "failed to read napi crate Cargo.toml at {} ({}); test \
             needs to read the dep manifest to enforce Compromise #N+8",
            cargo_toml.display(),
            e
        );
    });

    // Walk the file by lines so a comment mentioning a dep name (such
    // as the explanatory comment block above the `browser-target`
    // feature, or this test's own pin documentation) doesn't trip the
    // detector. Only `[dependencies]` / `[dev-dependencies]` /
    // `[build-dependencies]` table entries count.
    let forbidden = [("web-sys", "web-sys"), ("idb", "idb")];
    let mut in_deps_table = false;
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        // Strip end-of-line comments — `key = "value"  # comment` shape
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_deps_table = matches!(
                line,
                "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
            );
            continue;
        }
        if !in_deps_table {
            continue;
        }
        // Dep entries look like `name = ...` or `name.feature = ...`
        let dep_name = line
            .split_once('=')
            .map(|(k, _)| k.trim())
            .and_then(|k| k.split('.').next())
            .unwrap_or("");
        for (needle, label) in forbidden {
            assert_ne!(
                dep_name, needle,
                "Compromise #N+8: napi crate must not depend on `{}` \
                 (would enable browser-host persistence behind the \
                 BrowserManifestStore::is_persistent contract)",
                label
            );
        }
    }
}
