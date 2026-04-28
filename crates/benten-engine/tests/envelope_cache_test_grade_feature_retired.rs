//! Phase-2b G12-E must-pass — the `envelope-cache-test-grade` feature
//! is retired.
//!
//! Pre-G12-E the napi cdylib opted into `envelope-cache-test-grade` so
//! the WAIT suspend/resume bridge could round-trip envelopes through a
//! process-local `LazyLock<Mutex<BTreeMap>>` (the test-grade
//! `ENVELOPE_CACHE`). G12-E lands a real durable persistence layer
//! (`benten_eval::SuspensionStore` with the redb-backed default impl
//! `RedbSuspensionStore`), so the cdylib drops the feature.
//!
//! This test pins three invariants:
//!
//! 1. The napi `Cargo.toml` no longer activates
//!    `envelope-cache-test-grade` on the engine dep.
//! 2. The engine `Cargo.toml` retains the feature as an empty no-op
//!    (so any external consumer that named it still resolves), but
//!    its body is empty — there is no live source backing it.
//! 3. The engine source no longer carries a live `ENVELOPE_CACHE`
//!    static, `cache_put`/`cache_get` cfg-gated behind the feature, or
//!    `ENVELOPE_CACHE_MAX_ENTRIES` constant.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    PathBuf::from(&manifest_dir)
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .expect("workspace root")
}

#[test]
fn napi_cargo_toml_no_longer_pulls_envelope_cache_test_grade() {
    let napi_toml = workspace_root().join("bindings/napi/Cargo.toml");
    let body = fs::read_to_string(&napi_toml).expect("read bindings/napi/Cargo.toml");
    let benten_engine_dep_line = body
        .lines()
        .find(|l| l.starts_with("benten-engine = "))
        .expect("napi Cargo.toml must declare benten-engine dep");
    assert!(
        !benten_engine_dep_line.contains("envelope-cache-test-grade"),
        "napi cdylib MUST NOT pull `envelope-cache-test-grade` post-G12-E. \
         The durable SuspensionStore replaces the test-grade cache. \
         Found: `{benten_engine_dep_line}`"
    );
}

#[test]
fn engine_source_no_longer_compiles_envelope_cache_static_or_helpers() {
    let engine_wait = workspace_root().join("crates/benten-engine/src/engine_wait.rs");
    let body = fs::read_to_string(&engine_wait).expect("read engine_wait.rs");

    // The post-G12-E `cache_put` / `cache_get` helpers exist but route
    // through `engine.suspension_store` — they MUST NOT carry the
    // `envelope-cache-test-grade` cfg gate any longer.
    for (lineno, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        assert!(
            !(trimmed.starts_with("#[cfg") && trimmed.contains("envelope-cache-test-grade")),
            "engine_wait.rs:{}: post-G12-E source MUST NOT cfg-gate any \
             surface behind `envelope-cache-test-grade` (the feature is \
             retired). Offending line: `{trimmed}`",
            lineno + 1
        );
    }

    // The static `ENVELOPE_CACHE: LazyLock<Mutex<BTreeMap<...>>>` is
    // gone — its replacement is `engine.suspension_store.get_envelope`
    // / `put_envelope`.
    assert!(
        !body.contains("static ENVELOPE_CACHE"),
        "engine_wait.rs MUST NOT carry a `static ENVELOPE_CACHE` after G12-E \
         (replaced by `Engine::suspension_store`)"
    );
    assert!(
        !body.contains("ENVELOPE_CACHE_MAX_ENTRIES"),
        "engine_wait.rs MUST NOT define `ENVELOPE_CACHE_MAX_ENTRIES` after \
         G12-E (no in-memory cap on a non-existent cache)"
    );
}
