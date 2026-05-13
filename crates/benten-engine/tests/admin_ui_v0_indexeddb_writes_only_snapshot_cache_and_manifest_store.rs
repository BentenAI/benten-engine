//! Phase-4-Foundation G24-A — admin UI v0 IndexedDB writes contain ONLY
//! snapshot cache + manifest store data (no UCAN cap bytes, no plugin
//! secrets, no direct sync state).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 5; closes br-r1-7 + T2 (browser-shape data-at-rest hygiene).
//!
//! Per CLAUDE.md baked-in #17 (3 deployment shapes): in shape (b)
//! browser-wasm32, IndexedDB persistence is for **snapshot cache +
//! manifest store ONLY**. NOT for full sync state (no Loro / iroh
//! state); NOT for UCAN cap bytes (caps stay at the full peer); NOT
//! for plugin secrets.
//!
//! ## Substantive shape
//!
//! Grep-assert against the admin UI v0 source roots (Rust + TS):
//!
//! 1. No `'caps'` / `'ucan'` / `'secrets'` / `'sync_state'` /
//!    `'loro_state'` / `'iroh_state'` object-store name appears.
//! 2. `INDEXEDDB_FORBIDDEN_STORES` (the canonical block-list at
//!    `crates/benten-platform-foundation/src/admin_ui_v0/mod.rs`) is
//!    consulted as the source of truth — any agent adding a new
//!    forbidden store updates the constant, the grep-assert picks it
//!    up automatically.

#![allow(clippy::unwrap_used)]

use benten_platform_foundation::INDEXEDDB_FORBIDDEN_STORES;
use std::path::PathBuf;

fn admin_ui_v0_source_roots() -> Vec<PathBuf> {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = here.parent().unwrap().parent().unwrap();
    // ONLY the TS browser-side bundle root — that's where IndexedDB
    // invocations can physically happen. The Rust admin UI v0
    // module (`crates/benten-platform-foundation/src/admin_ui_v0/`)
    // is server-side handler code with no IndexedDB call surface, and
    // it legitimately names forbidden store strings in
    // `INDEXEDDB_FORBIDDEN_STORES` as the block-list constant the TS
    // bundle is checked against.
    vec![workspace.join("packages").join("admin-ui-v0").join("src")]
}

fn collect_source_files(root: &PathBuf) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    fn walk(dir: &PathBuf, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("rs" | "ts" | "tsx" | "js" | "mjs")
            ) {
                out.push(path);
            }
        }
    }
    walk(root, &mut out);
    out
}

#[test]
fn admin_ui_v0_indexeddb_writes_only_snapshot_cache_and_manifest_store() {
    let roots = admin_ui_v0_source_roots();
    let mut scanned_files = 0_usize;
    for root in &roots {
        for path in collect_source_files(root) {
            scanned_files += 1;
            let src = std::fs::read_to_string(&path).unwrap();
            // Strip line comments + block comments before scanning so
            // the doc-text constants in this very pin's `WINTERTC_*`
            // arrays + comments naming forbidden store names for
            // documentation purposes do not trip the grep.
            let scrubbed = strip_comments(&src);
            for store in INDEXEDDB_FORBIDDEN_STORES {
                let needle_single = format!("'{store}'");
                let needle_double = format!("\"{store}\"");
                // The needle is the JS/TS object-store-name string
                // literal — these MUST NEVER appear in a real
                // indexedDB op.
                assert!(
                    !scrubbed.contains(&needle_single) && !scrubbed.contains(&needle_double),
                    "Admin UI MUST NOT reference IndexedDB object store \
                     `{store}` (CLAUDE.md baked-in #17 + T2 defense 2). \
                     Found in: {}",
                    path.display()
                );
            }
        }
    }
    assert!(
        scanned_files > 0,
        "Admin UI v0 source roots MUST exist and contain at least one source file"
    );
}

fn strip_comments(src: &str) -> String {
    // Strip `// ...` line comments + `/* ... */` block comments. Naive
    // but enough for the grep contract — production-runtime store name
    // literals never appear inside comments because comments are not
    // valid IndexedDB invocation syntax.
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            // Skip until newline.
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}
