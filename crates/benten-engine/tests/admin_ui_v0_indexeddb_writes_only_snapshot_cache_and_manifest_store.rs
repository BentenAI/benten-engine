//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for admin UI v0
//! IndexedDB writes containing ONLY snapshot cache + manifest store
//! data (no UCAN cap bytes, no plugin secrets, no direct sync state).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 5; closes br-r1-7 + T2 (browser-shape data-at-rest hygiene).
//!
//! ## What this pin establishes
//!
//! Per CLAUDE.md baked-in #17 (3 deployment shapes): in shape (b)
//! browser-wasm32, IndexedDB persistence is for **snapshot cache +
//! manifest store ONLY**. NOT for full sync state (no Loro / iroh
//! state); NOT for UCAN cap bytes (caps stay at the full peer); NOT
//! for plugin secrets (private-namespace data lives in full peer's
//! durable storage).
//!
//! Per T2 defense 2 (`admin-ui-v0-threat-model.md` §T2): even a
//! successful XSS gets only an opaque session token. Cap bytes / plugin
//! secrets in browser storage would defeat that defense.
//!
//! ## Pin style
//!
//! Grep-assert against admin UI source (Rust + TS): no `indexedDB.transaction`
//! / `indexedDB.open` invocations against forbidden store names
//! (`caps`, `ucan`, `secrets`, `sync_state`, `loro_state`, `iroh_state`).
//! Plus a positive-side check that the only allowed object stores are
//! `snapshot_cache` + `manifest_store`.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A wave-6 wires this. Pin source: r2-test-landscape.md §2.6 row 5 + br-r1-7 + T2. Grep-assert: no UCAN/secrets/sync-state object stores in admin UI IndexedDB writes."]
fn admin_ui_v0_indexeddb_writes_only_snapshot_cache_and_manifest_store() {
    // G24-A wave wires this. Substantive shape:
    //
    //   // Admin UI v0 bundle source roots (browser-wasm32 surface):
    //   let admin_ui_ts_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..")
    //       .join("..")
    //       .join("packages")
    //       .join("admin-ui-v0")
    //       .join("src");
    //
    //   // Object stores that MUST NEVER appear in admin UI IndexedDB
    //   // ops per CLAUDE.md baked-in #17 + T2 defense 2:
    //   let forbidden_stores = [
    //       "caps", "cap_tokens", "ucan", "ucan_tokens",
    //       "secrets", "private_namespace", "plugin_secrets",
    //       "sync_state", "loro_state", "iroh_state",
    //   ];
    //
    //   // Object stores admin UI IS allowed to write per baked-in #17:
    //   let allowed_stores = ["snapshot_cache", "manifest_store"];
    //
    //   let mut found_allowed = 0_usize;
    //   for entry in walkdir::WalkDir::new(&admin_ui_ts_root) {
    //       let entry = entry.unwrap();
    //       if !entry.file_type().is_file() { continue; }
    //       let src = std::fs::read_to_string(entry.path()).unwrap();
    //
    //       for store in &forbidden_stores {
    //           // Match indexedDB.open('caps', ...) /
    //           // db.transaction(['caps'], ...) / db.createObjectStore('caps')
    //           let needle1 = format!("'{}'", store);
    //           let needle2 = format!("\"{}\"", store);
    //           assert!(
    //               !src.contains(&needle1) && !src.contains(&needle2),
    //               "Admin UI MUST NOT touch IndexedDB object store \
    //                `{}` per CLAUDE.md baked-in #17 + T2; found in {}",
    //               store,
    //               entry.path().display(),
    //           );
    //       }
    //
    //       for store in &allowed_stores {
    //           let needle1 = format!("'{}'", store);
    //           let needle2 = format!("\"{}\"", store);
    //           if src.contains(&needle1) || src.contains(&needle2) {
    //               found_allowed += 1;
    //           }
    //       }
    //   }
    //
    //   // Positive side: admin UI is touching the snapshot/manifest
    //   // stores it is supposed to:
    //   assert!(
    //       found_allowed > 0,
    //       "Admin UI MUST touch snapshot_cache OR manifest_store at \
    //        least once; ZERO references found"
    //   );
    //
    // OBSERVABLE consequence: data-at-rest hygiene in browser-shape
    // deployment. Defends against the failure shape where admin UI
    // caches a cap token "for performance" + XSS exfiltrates it.
    unimplemented!(
        "G24-A wires admin UI IndexedDB allowed-stores grep-assert per \
         br-r1-7 + T2"
    );
}
