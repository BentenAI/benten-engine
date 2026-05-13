//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for admin UI v0
//! thin-client carrying NO cap-tokens in any browser storage at runtime.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 10 (grep-assert + runtime); closes T2 defense 2.
//!
//! ## Distinction from `admin_ui_v0_no_cap_tokens_persisted_to_browser_storage.rs`
//!
//! That sibling pin is **source-grep-only** — proves admin UI source
//! contains no cap-token write API call sites. This pin is the
//! **runtime variant**: actually instantiate admin UI thin client in
//! a headless browser harness; exercise dogfood paths; inspect
//! browser-storage at runtime; assert NO storage entry has a
//! cap-token-shaped value.
//!
//! Together they form the SHAPE-not-SUBSTANCE pair per pim-18 §3.6f
//! (source-grep + runtime observation).

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-F wave-7 wires this. Pin source: r2-test-landscape.md §2.6 row 10 + T2 defense 2. RUNTIME variant of cap-token-storage pin: drive admin UI in headless browser through real dogfood path; inspect browser storage post-execution; assert ZERO entries match cap-token shape."]
fn admin_ui_v0_thin_client_no_cap_tokens_in_browser_storage_at_runtime() {
    // G24-F wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::
    //       new_thin_client_against_full_peer();
    //   let browser = harness.spawn_headless_browser_with_admin_ui();
    //   let origin_a = "https://benten.localhost:8443";
    //
    //   // Drive the full lifecycle that would tempt naive impls to
    //   // cache cap-tokens for performance:
    //   browser.establish_session(origin_a);
    //   browser.navigate_to("/workflows");
    //   browser.create_workflow("test_workflow");
    //   browser.navigate_to("/views");
    //   browser.create_composed_view(&["notes"]);
    //   browser.refresh(); // forces re-session-establish
    //
    //   // Inspect all 4 browser storage surfaces at runtime:
    //   let local_storage_dump = browser.dump_local_storage();
    //   let session_storage_dump = browser.dump_session_storage();
    //   let cookies_dump = browser.dump_cookies();
    //   let indexed_db_dump = browser.dump_indexed_db();
    //
    //   // Cap-token byte-shape recognizer: UCAN JWT tokens look like
    //   // base64-eyJ; cap-grant CIDs look like bafyr...; long random
    //   // strings ≥32 chars in keys named anything like cap/grant/ucan/
    //   // secret/key are suspect. Strict regexp:
    //   let cap_token_shape_pattern = regex::Regex::new(
    //       r"(?i)(cap|grant|ucan|secret|priv_?key)[\w_]*"
    //   ).unwrap();
    //
    //   for (label, dump) in [
    //       ("localStorage", &local_storage_dump),
    //       ("sessionStorage", &session_storage_dump),
    //       ("cookies", &cookies_dump),
    //       ("indexedDB", &indexed_db_dump),
    //   ] {
    //       for (key, value) in dump.iter() {
    //           assert!(
    //               !cap_token_shape_pattern.is_match(key),
    //               "Admin UI MUST NOT store cap-token-shaped key in {} \
    //                per T2 defense 2; saw key `{}` with value `{:.40}...`",
    //               label, key, value,
    //           );
    //
    //           // Heuristic for cap-bytes in value: long base64 strings.
    //           // Anything >= 100 chars of [A-Za-z0-9+/=] in a value is
    //           // suspicious (UCAN JWTs are typically 300-600 chars):
    //           let suspicious_b64 = regex::Regex::new(
    //               r"[A-Za-z0-9+/=]{100,}"
    //           ).unwrap();
    //           assert!(
    //               !suspicious_b64.is_match(value),
    //               "Admin UI {} key `{}` has suspicious base64-shaped \
    //                value (>=100 chars) — likely a cap-token leak per \
    //                T2 defense 2",
    //               label, key,
    //           );
    //       }
    //   }
    //
    //   // Positive sanity — session-establishment did populate SOMETHING
    //   // (otherwise admin UI is broken, not the defense passing):
    //   assert!(
    //       local_storage_dump.len() + session_storage_dump.len()
    //         + cookies_dump.len() + indexed_db_dump.len() > 0,
    //       "Sanity: admin UI session-establish populated NO browser \
    //        storage entries — pin is vacuously passing"
    //   );
    //
    // OBSERVABLE consequence: real defense, not just source-text
    // hygiene. Defends against transitive deps writing cap-shaped
    // values to storage.
    unimplemented!(
        "G24-F wires admin UI runtime cap-tokens-in-browser-storage \
         pin per T2 defense 2 (SUBSTANCE half pairing with grep-assert \
         sibling)"
    );
}
