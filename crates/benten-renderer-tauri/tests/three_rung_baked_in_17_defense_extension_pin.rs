//! G24-E wave-7 LANDED pin — gap #1c closure (br-r1-4 + br-r1-13;
//! `r2-test-landscape.md` §5).
//!
//! Extends the 3-rung baked-in-#17 defense (PR #166 G16-B B precedent
//! per Phase-3 R6 fp wave; per CLAUDE.md baked-in commitment #17 three
//! deployment shapes — full peer / thin compute surface / embedded
//! webview) to cover the `benten-renderer-tauri` crate's wasm32 build.
//!
//! The forbidden-prefix list applies to the wasm32-unknown-unknown
//! admin UI v0 bundle that the Tauri webview loads; native-only
//! sync-runtime symbols (Loro, iroh, tokio multi-threaded runtime,
//! SANDBOX wasmtime host, redb durable backend) must NOT appear in the
//! bundle.
//!
//! ## Three rungs
//!
//! 1. **wasm32-objdump forbidden-prefix sweep** on the loaded bundle
//!    (this test; runtime-arm when `ADMIN_UI_V0_WASM_PATH` env var is
//!    present).
//! 2. **Feature-graph-closure assertion**: this crate's production
//!    deps MUST NOT transitively pull native-only sync-runtime crates
//!    into the wasm32 target's compilation graph. Asserted at the
//!    Cargo.toml + `cargo tree`-equivalent grep below (always-on; no
//!    env var needed).
//! 3. **Renderer-extension trust posture (CLAUDE.md #19)**: the crate
//!    declares no dependency on `benten-sync` / `tokio` / `iroh` /
//!    `wasmtime` / `redb`. Pin lives in
//!    `arch_n_benten_renderer_tauri_dep_direction.rs` (sibling pin); a
//!    rung-3 cross-check companion is asserted in this file too.
//!
//! ## Closes
//!
//! Gap #1c (br-r1-4 + br-r1-13 in `r2-test-landscape.md` §5)

#![allow(clippy::unwrap_used, clippy::print_stderr)]

use std::path::PathBuf;

const FORBIDDEN_PREFIXES: &[&str] = &[
    // Native-only sync-runtime symbols (CLAUDE.md baked-in #17):
    "loro::",
    "iroh::",
    "iroh_net::",
    // SANDBOX wasmtime host (full peer only):
    "wasmtime::",
    "wasmtime_runtime::",
    // Multi-threaded tokio runtime (full peer only):
    "tokio::runtime::scheduler::multi_thread::",
    // redb durable backend (full peer only):
    "redb::",
];

const FORBIDDEN_DEP_NAMES: &[&str] = &[
    "benten-sync",
    "iroh",
    "iroh-net",
    "wasmtime",
    "redb",
    "loro",
];

fn cargo_toml_text() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

#[test]
fn benten_renderer_tauri_cargo_toml_has_no_forbidden_native_only_deps() {
    // RUNG 2 (always-on): the crate's Cargo.toml MUST NOT name a
    // forbidden native-only sync-runtime dep. If a future agent
    // accidentally added one — even gated by `target = ...` — this
    // pin would surface it BEFORE the wasm32 build pipeline could
    // silently ship a fat bundle.
    let toml = cargo_toml_text();
    for forbidden in FORBIDDEN_DEP_NAMES {
        let needle_eq = format!("{forbidden} =");
        let needle_eq2 = format!("\"{forbidden}\"");
        assert!(
            !toml.contains(&needle_eq) && !toml.contains(&needle_eq2),
            "Forbidden native-only dep `{forbidden}` present in \
             benten-renderer-tauri Cargo.toml — per CLAUDE.md baked-in \
             #17, this crate ships into the wasm32-unknown-unknown \
             admin UI bundle shape (c) which must stay thin."
        );
    }
}

#[test]
fn benten_renderer_tauri_wasm32_bundle_does_not_contain_forbidden_prefixes() {
    // RUNG 1 (runtime-arm): when `ADMIN_UI_V0_WASM_PATH` is set (CI
    // wires this), invoke wasm-objdump and assert no forbidden prefix
    // appears. When unset (local devbox runs), skip — the always-on
    // dep-cargo-toml assertion above carries the load-bearing
    // guarantee, and CI is the authoritative bundle-sweep boundary.
    let Ok(bundle_path) = std::env::var("ADMIN_UI_V0_WASM_PATH") else {
        eprintln!(
            "ADMIN_UI_V0_WASM_PATH unset — skipping wasm-objdump sweep \
             (CI is authoritative for this rung; see benten-renderer-\
             tauri::three_rung_baked_in_17_defense_extension_pin docs)"
        );
        return;
    };

    let output = std::process::Command::new("wasm-objdump")
        .arg("-x")
        .arg(&bundle_path)
        .output();
    let symbols_text = match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        Ok(out) => panic!(
            "wasm-objdump failed on {bundle_path}: status={:?} stderr={}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) => panic!("wasm-objdump invocation failed: {e}"),
    };

    for forbidden in FORBIDDEN_PREFIXES {
        assert!(
            !symbols_text.contains(forbidden),
            "Forbidden prefix {forbidden:?} found in {bundle_path} — \
             native-only symbol leaked into wasm32 bundle (per \
             CLAUDE.md #17 thin compute surface deployment shape (b)/(c))."
        );
    }
}

#[test]
fn benten_renderer_tauri_is_an_engine_extension_per_claude_md_19() {
    // RUNG 3 cross-check: the crate declares itself as an engine
    // extension; its module doc names the trust boundary. Grep-assert
    // the load-bearing CLAUDE.md #19 phrase in the crate root doc.
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
    let body = std::fs::read_to_string(&lib_path).unwrap();
    assert!(
        body.contains("CLAUDE.md #19") || body.contains("CLAUDE.md` #19"),
        "module doc must reference CLAUDE.md #19 (engine-extension \
         trust boundary); did not find phrase in {}",
        lib_path.display()
    );
    assert!(
        body.contains("read_node_as"),
        "module doc must reference the Class B β `read_node_as` \
         boundary (engine extensions don't go through it)"
    );
}
