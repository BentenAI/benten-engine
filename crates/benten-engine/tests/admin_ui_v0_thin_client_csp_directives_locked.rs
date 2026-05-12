//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for thin-client CSP
//! directives locked at admin UI serving boundary (browser-wasm32 shape).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.11
//! supplementary pin (per Family F1 brief "3-4 more per R2 §2.11");
//! closes T2 defense 5 + br-r1-11 (admin-ui-v0-threat-model.md §T2:
//! CSP defense-in-depth across browser-tab + embedded-webview shapes).
//!
//! ## What this pin establishes
//!
//! Per T2 defense 5: the thin-client serving boundary applies CSP
//! directives that block arbitrary remote scripts + restrict `script-src`
//! to `'self' 'wasm-unsafe-eval'` + restrict `connect-src` to `'self'
//! tauri://*` + lock `default-src` to `'none'`. Same shape as T3
//! Tauri-CSP — defense-in-depth across both shape (b) and shape (c).

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-F wave-7 wires this. Pin source: r2-test-landscape.md §2.11 supplementary + T2 defense 5 + br-r1-11. Substantive: HTTP response from full-peer admin UI surface carries the canonical CSP header values."]
fn admin_ui_v0_thin_client_csp_directives_locked() {
    // G24-F wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::
    //       new_thin_client_against_full_peer();
    //
    //   // Issue an HTTP GET for the admin UI v0 index from the full
    //   // peer's bound localhost surface:
    //   let response = harness.full_peer_http_get(
    //       "https://benten.localhost:8443/"
    //   );
    //
    //   let csp = response
    //       .header("content-security-policy")
    //       .expect("CSP header MUST be present per T2 defense 5");
    //
    //   // Per T2 defense 5 canonical directives:
    //   let required_directives = [
    //       ("default-src", "'none'"),
    //       ("script-src", "'self' 'wasm-unsafe-eval'"),
    //       ("connect-src", "'self'"),
    //       ("style-src", "'self'"),
    //       ("font-src", "'self'"),
    //   ];
    //   for (directive, value) in &required_directives {
    //       assert!(
    //           csp.contains(&format!("{} {}", directive, value)),
    //           "CSP header MUST contain `{} {}` per T2 defense 5; \
    //            saw `{}`",
    //           directive, value, csp,
    //       );
    //   }
    //
    //   // Forbidden directives (would weaken defense):
    //   let forbidden_substrings = [
    //       "unsafe-eval",  // wasm-unsafe-eval is OK but bare unsafe-eval isn't
    //       "unsafe-inline",
    //       "data:",         // forbidden in script-src
    //       "https://",      // arbitrary remote script src forbidden
    //   ];
    //   // Refine: we only want to forbid these in script-src context.
    //   // Parse the script-src directive specifically:
    //   let script_src = csp.split(';')
    //       .find(|s| s.trim().starts_with("script-src"))
    //       .unwrap_or("");
    //   for forbidden in &forbidden_substrings {
    //       if *forbidden == "unsafe-eval" {
    //           // Allow only the wasm-unsafe-eval variant; bare
    //           // unsafe-eval is a stronger relaxation:
    //           let bare_unsafe_eval_present = script_src
    //               .split_whitespace()
    //               .any(|t| t.trim_matches('\'') == "unsafe-eval");
    //           assert!(
    //               !bare_unsafe_eval_present,
    //               "script-src MUST NOT include bare 'unsafe-eval' \
    //                (wasm-unsafe-eval is permitted); saw `{}`",
    //               script_src,
    //           );
    //       } else {
    //           assert!(
    //               !script_src.contains(forbidden),
    //               "script-src MUST NOT contain `{}` per T2 defense 5; \
    //                saw `{}`",
    //               forbidden, script_src,
    //           );
    //       }
    //   }
    //
    // OBSERVABLE consequence: post-load injected attacker JS is blocked
    // by the browser's CSP enforcement. Defense-in-depth at network
    // boundary even if origin pinning slipped.
    unimplemented!(
        "G24-F wires thin-client CSP directives locked pin per T2 \
         defense 5 + br-r1-11"
    );
}
