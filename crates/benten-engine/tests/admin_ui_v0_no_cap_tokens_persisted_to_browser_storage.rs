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

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-F wave-7 wires this. Pin source: r2-test-landscape.md §2.11 row 3 + T2 defense 2. Grep-assert: no cap-token-related write to any browser-storage surface in admin UI source."]
fn admin_ui_v0_no_cap_tokens_persisted_to_browser_storage() {
    // G24-F wave wires this. Substantive shape:
    //
    //   let admin_ui_ts_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..")
    //       .join("..")
    //       .join("packages")
    //       .join("admin-ui-v0")
    //       .join("src");
    //
    //   // Cap-token-related identifier prefixes that MUST NOT appear
    //   // as keys of any browser-storage write per T2 defense 2:
    //   let forbidden_key_prefixes = [
    //       "cap_", "ucan_", "secret_", "key_", "private_key", "grant_",
    //   ];
    //
    //   // Browser-storage write surfaces:
    //   let storage_write_patterns = [
    //       "localStorage.setItem(",
    //       "sessionStorage.setItem(",
    //       "document.cookie",
    //       "indexedDB",
    //   ];
    //
    //   for entry in walkdir::WalkDir::new(&admin_ui_ts_root) {
    //       let entry = entry.unwrap();
    //       if !entry.file_type().is_file() { continue; }
    //       let src = std::fs::read_to_string(entry.path()).unwrap();
    //
    //       // For each storage write surface, find call sites + inspect
    //       // their first argument (the key) for forbidden prefix:
    //       for storage_pat in &storage_write_patterns {
    //           let mut idx = 0_usize;
    //           while let Some(pos) = src[idx..].find(storage_pat) {
    //               let abs = idx + pos;
    //               // Extract the first arg (between `(` and `,`):
    //               let arg_start = abs + storage_pat.len();
    //               let arg_window = &src[arg_start..(arg_start + 60).min(src.len())];
    //               let first_arg = arg_window
    //                   .split(',')
    //                   .next()
    //                   .unwrap_or("")
    //                   .trim()
    //                   .trim_matches('\'')
    //                   .trim_matches('"');
    //
    //               for prefix in &forbidden_key_prefixes {
    //                   assert!(
    //                       !first_arg.starts_with(prefix),
    //                       "Admin UI MUST NOT persist cap-token-shaped \
    //                        key `{}` to browser storage per T2 defense 2; \
    //                        found in {} at {}",
    //                       first_arg, entry.path().display(), abs,
    //                   );
    //               }
    //
    //               idx = abs + storage_pat.len();
    //           }
    //       }
    //   }
    //
    // OBSERVABLE consequence: cap tokens stay at the full peer. Even
    // a successful XSS gets only an opaque, origin-bound, time-bound
    // session token — no cap-bytes to exfiltrate.
    unimplemented!(
        "G24-F wires admin UI no-cap-tokens-in-browser-storage grep-assert \
         per T2 defense 2"
    );
}
