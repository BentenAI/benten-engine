//! G24-E wave-7 RED-PHASE pin — gap #1c closure (br-r1-4 + br-r1-13;
//! `r2-test-landscape.md` §5).
//!
//! Extends the 3-rung baked-in-#17 defense (PR #166 G16-B B precedent
//! per Phase-3 R6 fp wave; per CLAUDE.md baked-in commitment #17 three
//! deployment shapes — full peer / thin compute surface / embedded
//! webview) to cover the `benten-renderer-tauri` crate's wasm32 build.
//! The forbidden-prefix list applies to the wasm32-unknown-unknown
//! admin UI v0 bundle that the Tauri webview loads; native-only
//! sync-runtime symbols (Loro, iroh, tokio multi-threaded runtime,
//! SANDBOX wasmtime host) must NOT appear in the bundle.
//!
//! ## Why this pin matters
//!
//! Gap #1c in `r2-test-landscape.md` §5: the original G24-E pin set
//! enumerated webview-CSP and IPC-allowlist defenses (which guard the
//! webview's runtime behavior) but no static check that the loaded
//! wasm bundle is the thin-compute-surface bundle. Without this pin,
//! a build-system regression could quietly ship a wasm bundle that
//! linked native-only sync-runtime symbols (e.g., via a misconfigured
//! `[target.'cfg(target_arch = "wasm32")']` cfg-guard) and the Tauri
//! webview would silently load it.
//!
//! ## RED-PHASE status
//!
//! `#[ignore]` until G24-E wave-7 lands the wasm32 build pipeline for
//! `benten-renderer-tauri` AND the `wasm32-objdump` forbidden-prefix
//! discipline (`tools/wasm-symbol-check` or equivalent; precedent at
//! PR #166).
//!
//! ## Closes
//!
//! Gap #1c (br-r1-4 + br-r1-13 in `r2-test-landscape.md` §5)

#![allow(clippy::unwrap_used, dead_code, unused_imports)]

use benten_renderer_tauri as _renderer;

#[test]
#[ignore = "RED-PHASE: closes at R5 G24-E wave-7 (wasm32-objdump forbidden-prefix sweep on tauri bundle)"]
fn benten_renderer_tauri_wasm32_bundle_does_not_contain_forbidden_prefixes() {
    // Production arm (G24-E wave-7):
    //
    //   const FORBIDDEN_PREFIXES: &[&str] = &[
    //       // Native-only sync-runtime symbols (CLAUDE.md baked-in #17):
    //       "loro::",
    //       "iroh::",
    //       "iroh_net::",
    //       // SANDBOX wasmtime host (full peer only):
    //       "wasmtime::",
    //       "wasmtime_runtime::",
    //       // Multi-threaded tokio runtime (full peer only):
    //       "tokio::runtime::scheduler::multi_thread::",
    //       // redb durable backend (full peer only):
    //       "redb::",
    //   ];
    //
    //   let bundle_path = std::env::var("ADMIN_UI_V0_WASM_PATH")
    //       .expect("set ADMIN_UI_V0_WASM_PATH to the wasm32 bundle under test");
    //   let symbols = std::process::Command::new("wasm-objdump")
    //       .arg("-x").arg(&bundle_path)
    //       .output().unwrap();
    //   let symbols_text = String::from_utf8(symbols.stdout).unwrap();
    //
    //   for forbidden in FORBIDDEN_PREFIXES {
    //       assert!(!symbols_text.contains(forbidden),
    //           "Forbidden prefix {:?} found in {} — native-only symbol \
    //            leaked into wasm32 bundle (per CLAUDE.md #17 thin compute \
    //            surface deployment shape).",
    //           forbidden, bundle_path);
    //   }
    //
    // Would-FAIL-if-no-op'd: any forbidden prefix appearing in the
    // bundle would cause an explicit assertion failure with the
    // forbidden-prefix string in the failure message.
    panic!("RED-PHASE: production surface lands at G24-E wave-7");
}
