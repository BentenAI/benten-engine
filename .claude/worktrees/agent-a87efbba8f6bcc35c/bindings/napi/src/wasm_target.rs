//! Phase-2b G10-A-wasip1 — `wasm32-wasip1` runtime path for the
//! `benten-napi` cdylib.
//!
//! ## Surface scope
//!
//! `wasm32-wasip1` is the Node-WASI target the napi-rs v3 build matrix
//! materializes when the host is a Node.js process running the `wasi`
//! preview1 ABI (notably the `@bytecodealliance/preview2-shim` runtime
//! and Node 22's `--experimental-wasi-preview1`). The corresponding
//! `wasm32-unknown-unknown` browser target is owned by the sibling
//! `wasm_browser.rs` module (G10-A-browser).
//!
//! On `wasm32-wasip1`:
//!
//! - `wasi_target_kind()` returns the static string `"wasm32-wasip1"`
//!   so a TS caller can branch on the resolved target without parsing
//!   the napi triple.
//! - `wasi_runtime_supports_redb_native()` returns `false` — the
//!   wasm32-wasip1 build does NOT carry the redb-backed `KVBackend`
//!   (no filesystem access). Snapshot-blob hand-off via
//!   `Engine::from_snapshot_blob` is the supported construction path.
//! - `wasi_canonical_fixture_cid()` returns the Phase-1 canonical
//!   fixture CID (`bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`)
//!   recomputed locally from `benten_core::testing::canonical_test_node`,
//!   proving the wasm32-wasip1 build re-derives the SAME CID as native
//!   (wasm-r1-1 dual-target invariant). The base32 string round-trips
//!   through `Cid::to_base32`.
//!
//! On native (`cfg(not(target_arch = "wasm32"))`):
//!
//! - `wasi_target_kind()` returns `"native"` so the same JS callsite
//!   sees a stable answer regardless of which build it loads.
//! - `wasi_runtime_supports_redb_native()` returns `true`.
//! - `wasi_canonical_fixture_cid()` returns the same canonical CID,
//!   so a `assertEqual(native.fixtureCid, wasi.fixtureCid)` test pin
//!   in the wasm-runtime workflow proves cross-target CID stability
//!   without needing to load both binaries in the same process.
//!
//! ## Why both halves of the cfg-gate ship
//!
//! Same defence-in-depth posture as `sandbox.rs` —
//! `typeof native.wasiTargetKind === "function"` is the documented
//! contract; absence of the symbol on either target would force TS
//! callers to special-case the gap.

#![cfg(feature = "napi-export")]

use napi_derive::napi;

// ---------------------------------------------------------------------------
// `wasi_target_kind` — resolved-target probe
// ---------------------------------------------------------------------------

/// Returns `"wasm32-wasip1"` on the Node-WASI build and `"native"` on
/// every other target.
///
/// Pinned by the wasm-runtime workflow (`wasm-runtime.yml`): the
/// workflow asserts that a WASI-target invocation surfaces the wasi
/// string while the native side surfaces `"native"`.
#[cfg(target_arch = "wasm32")]
#[napi(js_name = "wasiTargetKind")]
pub fn wasi_target_kind() -> &'static str {
    "wasm32-wasip1"
}

/// Native answer — see the wasm32 sibling for context.
#[cfg(not(target_arch = "wasm32"))]
#[napi(js_name = "wasiTargetKind")]
pub fn wasi_target_kind() -> &'static str {
    "native"
}

// ---------------------------------------------------------------------------
// `wasi_runtime_supports_redb_native` — feature-availability probe
// ---------------------------------------------------------------------------

/// Returns `true` on native builds (redb file-backed) and `false` on
/// wasm32-wasip1 (no filesystem access; snapshot-blob hand-off is the
/// supported construction path).
#[cfg(target_arch = "wasm32")]
#[napi(js_name = "wasiRuntimeSupportsRedbNative")]
pub fn wasi_runtime_supports_redb_native() -> bool {
    false
}

/// Native answer — see the wasm32 sibling for context.
#[cfg(not(target_arch = "wasm32"))]
#[napi(js_name = "wasiRuntimeSupportsRedbNative")]
pub fn wasi_runtime_supports_redb_native() -> bool {
    true
}

// ---------------------------------------------------------------------------
// `wasi_canonical_fixture_cid` — canonical-CID dual-target invariant
// ---------------------------------------------------------------------------

/// Recompute and return the canonical-fixture CID
/// (`bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`) as a
/// base32 multibase string. Identical on every target — the
/// wasm-runtime workflow (`wasm-runtime.yml`) compares the
/// wasm32-wasip1 answer against the native answer to prove the CID
/// reproduces under the WASI runtime (wasm-r1-1 dual-target invariant).
///
/// Returns the empty string if the canonical encoding fails — never
/// happens in practice (the canonical fixture is a fixed-shape Node
/// the encoding has covered for every prior phase) but we surface a
/// safe sentinel rather than panicking inside the napi boundary.
#[napi(js_name = "wasiCanonicalFixtureCid")]
pub fn wasi_canonical_fixture_cid() -> String {
    let node = benten_core::testing::canonical_test_node();
    match node.cid() {
        Ok(cid) => cid.to_base32(),
        Err(_) => String::new(),
    }
}
