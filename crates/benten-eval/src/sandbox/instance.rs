//! Per-call SANDBOX wasmtime lifecycle (Phase 2b G7-A).
//!
//! D3-RESOLVED + wsa-20 — per-call scope clarification:
//!   - [`wasmtime::Engine`]: shared singleton, constructed at first-use,
//!     reused for the lifetime of the benten engine process.
//!   - [`wasmtime::Module`]: content-CID-cached. Compile-once per module
//!     CID; reused across primitive calls. The cache key is the BLAKE3 of
//!     the module bytes so the wasmtime compile cache is content-addressed
//!     in the same shape as the rest of the workspace.
//!   - [`wasmtime::Store`] + [`wasmtime::Instance`]: PER-CALL. Constructed
//!     fresh for each primitive call, dropped at completion. No
//!     cross-call state retention by construction (closes the "trusted
//!     boundary" sub-question of D3 entirely).
//!
//! Without the Engine + Module shared discipline, the per-call cold-start
//! budget (D22 ≤2ms p95 Linux x86_64) is unmeetable — recompiling per
//! call would dominate. The Store + Instance per-call discipline closes
//! cross-call leakage by definition.
//!
//! This module is `#[cfg(not(target_arch = "wasm32"))]`-gated per
//! sec-pre-r1-05; the wasm32 build cuts SANDBOX entirely.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use wasmtime::{Config, Engine, Module};

/// Process-wide shared [`wasmtime::Engine`] (D3-RESOLVED + wsa-20).
///
/// Constructed at first call via [`shared_engine`]; reused across all
/// SANDBOX primitive calls within the process. Configured with:
///   - `consume_fuel(true)` — D21 fuel-axis enforcement.
///   - `epoch_interruption(true)` — D24 wallclock-axis enforcement (the
///     epoch ticker is driven by a separate thread; see [`crate::sandbox::primitives_sandbox`].
///   - `async_support(true)` — D27-RESOLVED forward-compat for Phase-3
///     iroh async host-fns (no async host-fn ships in 2b; the feature
///     is gated at the wasmtime API surface so enabling it doesn't
///     change runtime behavior).
///   - NO `pooling-allocator` per D3-RESOLVED.
///   - NO `component-model` per wsa-3 (plan uses core-wasm
///     `wasmtime::Instance`).
static SHARED_ENGINE: OnceLock<Engine> = OnceLock::new();

/// Acquire the shared [`Engine`]. Lazily constructs on first call.
///
/// # Panics
/// Panics if [`Config`] construction or [`Engine::new`] fails. Both
/// failures indicate a wasmtime build / feature-flag mismatch and are
/// not recoverable from user code.
#[must_use]
pub fn shared_engine() -> &'static Engine {
    SHARED_ENGINE.get_or_init(|| {
        let mut cfg = Config::new();
        cfg.consume_fuel(true);
        cfg.epoch_interruption(true);
        cfg.async_support(true);
        // Defense-in-depth: cap stack size so ESC-5 (recursion-overflow)
        // surfaces as a wasmtime trap, not a host-process abort.
        cfg.max_wasm_stack(512 * 1024);
        Engine::new(&cfg).expect("wasmtime Engine construction failed")
    })
}

/// Module CID — BLAKE3 over the module's wasm bytes. Used as the cache
/// key in the process-wide module cache (see [`module_for_bytes`] +
/// [`module_cache_size`]).
///
/// Defined locally (not `benten_core::Cid`) because the cache key is a
/// content hash of raw `&[u8]`; we do not want to construct a full
/// `Cid` envelope per cache lookup.
pub type ModuleCidHash = [u8; 32];

/// Compute the [`ModuleCidHash`] for a wasm module's bytes.
#[must_use]
pub fn hash_module_bytes(bytes: &[u8]) -> ModuleCidHash {
    *blake3::hash(bytes).as_bytes()
}

/// Process-wide compile cache for [`wasmtime::Module`]. Keyed by
/// [`ModuleCidHash`] (BLAKE3 over the module bytes). D22 cold-start
/// budget is unmeetable without this cache.
///
/// The cache is `Mutex<HashMap<...>>`-protected because [`Module::new`]
/// is the slow path; concurrent compiles would each pay the full cost
/// for the same CID. The mutex is held for the duration of the compile
/// (single-flight discipline).
static MODULE_CACHE: OnceLock<Mutex<HashMap<ModuleCidHash, Arc<Module>>>> = OnceLock::new();

fn module_cache_inner() -> &'static Mutex<HashMap<ModuleCidHash, Arc<Module>>> {
    MODULE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Compile (or fetch from cache) a [`wasmtime::Module`] for the given
/// wasm bytes. The cache key is BLAKE3 over `bytes`.
///
/// Subsequent calls with the same bytes return the cached `Arc<Module>`
/// without recompilation. The single-flight discipline holds the mutex
/// during compile so concurrent first-touch on the same CID does not
/// double-compile.
///
/// # Errors
/// Returns the wasmtime error on `Module::new` failure (typically
/// `E_SANDBOX_MODULE_INVALID` at the SANDBOX executor boundary).
pub fn module_for_bytes(bytes: &[u8]) -> Result<Arc<Module>, wasmtime::Error> {
    let cid = hash_module_bytes(bytes);
    let cache = module_cache_inner();
    {
        // Fast path: cache hit, no compile needed.
        let guard = cache.lock().expect("module cache mutex poisoned");
        if let Some(m) = guard.get(&cid) {
            return Ok(Arc::clone(m));
        }
    }
    // Slow path: compile under the mutex (single-flight). Re-check the
    // cache inside the lock in case a concurrent call beat us to it.
    let engine = shared_engine();
    let mut guard = cache.lock().expect("module cache mutex poisoned");
    if let Some(m) = guard.get(&cid) {
        return Ok(Arc::clone(m));
    }
    let module = Arc::new(Module::new(engine, bytes)?);
    guard.insert(cid, Arc::clone(&module));
    Ok(module)
}

/// Cache size — for diagnostic / drift tests. Returns the current
/// number of compiled modules retained.
#[must_use]
pub fn module_cache_size() -> usize {
    module_cache_inner()
        .lock()
        .expect("module cache mutex poisoned")
        .len()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    /// Trivial valid wasm module — `(module)` empty. wat → wasm via
    /// wasmtime's bundled `wat` feature.
    fn empty_module_wasm() -> Vec<u8> {
        wat::parse_str("(module)").expect("empty module compiles")
    }

    #[test]
    fn shared_engine_returns_same_arc_across_calls() {
        let a = shared_engine();
        let b = shared_engine();
        // Pointer equality on the &'static reference.
        assert!(std::ptr::eq(a, b), "wsa-20 — Engine must be a singleton");
    }

    #[test]
    fn module_cache_hits_on_repeated_bytes() {
        let bytes = empty_module_wasm();
        let m1 = module_for_bytes(&bytes).expect("module compiles");
        let m2 = module_for_bytes(&bytes).expect("module compiles");
        // Arc pointer equality — second call returns the cached entry.
        assert!(
            Arc::ptr_eq(&m1, &m2),
            "wsa-20 — Module cache must reuse compiled artifact"
        );
    }
}
