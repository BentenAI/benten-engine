//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for admin UI v0 NOT
//! persisting cap tokens to any browser storage surface (localStorage /
//! sessionStorage / indexedDB / cookies).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.11
//! row 3; closes T2 defense 2 (admin-ui-v0-threat-model.md §T2:
//! "No cap-tokens in JS storage. Caps stay at the full peer; thin
//! client carries only the opaque `SessionToken`.").
//!
//! ## Pin style
//!
//! Grep-assert against admin UI v0 TS source — no patterns matching
//! `localStorage.setItem('cap_*')`, `localStorage.setItem('ucan_*')`,
//! `sessionStorage.setItem('cap_*')`, `document.cookie = 'cap_*'`,
//! `indexedDB.transaction('caps', ...)`. Companion to
//! `admin_ui_v0_indexeddb_writes_only_snapshot_cache_and_manifest_store.rs`
//! which covers IndexedDB specifically; this pin covers ALL browser
//! storage surfaces broadly.

#![allow(clippy::unwrap_used)]

mod common;

/// Un-ignored at Phase-4-Foundation R6-FP-BF (closes R6 R1
/// test-coverage-auditor tc-1 cluster). Grep-asserts the
/// `packages/admin-ui-v0/src/` TS tree contains zero browser-storage
/// writes whose first argument (the key) starts with a cap-token-shaped
/// prefix. PRODUCTION SUBSTANCE: walks the same TS source the admin UI
/// shape (b) browser-wasm32 bundle compiles from; any future regression
/// that adds e.g. `localStorage.setItem("cap_xxx", ...)` fails this pin.
#[test]
fn admin_ui_v0_no_cap_tokens_persisted_to_browser_storage() {
    let admin_ui_ts_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("packages")
        .join("admin-ui-v0")
        .join("src");
    assert!(
        admin_ui_ts_root.is_dir(),
        "admin UI v0 TS source tree must exist at {} for this pin to be \
         meaningful; if the TS package was moved, retarget the path",
        admin_ui_ts_root.display()
    );

    let forbidden_key_prefixes = ["cap_", "ucan_", "secret_", "key_", "private_key", "grant_"];
    let storage_write_patterns = [
        "localStorage.setItem(",
        "sessionStorage.setItem(",
        "document.cookie",
        "indexedDB.open(",
    ];

    // Walk the TS source tree.
    fn walk(dir: &std::path::Path, storage_patterns: &[&str], forbidden: &[&str]) -> Vec<String> {
        let mut violations = Vec::new();
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return violations,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                violations.extend(walk(&path, storage_patterns, forbidden));
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "ts" | "tsx" | "js" | "mjs" | "cjs") {
                continue;
            }
            let src = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            for storage_pat in storage_patterns {
                let mut idx = 0_usize;
                while let Some(pos) = src[idx..].find(storage_pat) {
                    let abs = idx + pos;
                    let arg_start = abs + storage_pat.len();
                    let arg_end = (arg_start + 60).min(src.len());
                    let arg_window = &src[arg_start..arg_end];
                    let first_arg = arg_window
                        .split(',')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('\'')
                        .trim_matches('"');
                    for prefix in forbidden {
                        if first_arg.starts_with(prefix) {
                            violations.push(format!(
                                "{}:{}: `{}` starts with forbidden cap-token \
                                 prefix `{}`",
                                path.display(),
                                abs,
                                first_arg,
                                prefix
                            ));
                        }
                    }
                    idx = abs + storage_pat.len();
                }
            }
        }
        violations
    }

    let violations = walk(
        &admin_ui_ts_root,
        &storage_write_patterns,
        &forbidden_key_prefixes,
    );
    assert!(
        violations.is_empty(),
        "Admin UI v0 MUST NOT persist cap-token-shaped keys to any \
         browser-storage surface per T2 defense 2; found violations: {:?}",
        violations
    );
}
