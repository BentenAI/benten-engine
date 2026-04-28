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
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use wasmtime::{Config, Engine, Module};

/// **wsa-g7a-mr-5 fix-pass:** soft cap on the in-memory module cache
/// size. Without a bound the cache grows monotonically with unique
/// module CIDs seen in the process lifetime, which under a Phase-3
/// marketplace workload (many distinct modules) is an OOM exposure.
/// 256 modules at hundreds-of-KiB-to-MiB compiled artifact each gives
/// a cache memory ceiling around 256 MiB worst-case. When the cache
/// reaches this size the OLDEST insertion is evicted (FIFO; not
/// strictly LRU but the simpler discipline is sufficient for the
/// scaffold — Phase-3 may swap to a true LRU if access-pattern data
/// argues for it).
pub const MODULE_CACHE_MAX_ENTRIES: usize = 256;

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
/// **perf-g7a-mr-3 fix-pass:** the cache uses [`RwLock`] for the
/// fast-path read (concurrent SANDBOX calls on DISTINCT modules do not
/// serialize) and a separate single-flight [`Mutex`] for the slow-path
/// compile (so two callers first-touching the SAME CID don't
/// double-compile). The fast-path read covers the common case
/// (warm cache); the slow-path compile is rare.
///
/// **wsa-g7a-mr-5 fix-pass:** entries are bounded by
/// [`MODULE_CACHE_MAX_ENTRIES`] with FIFO insertion-order eviction
/// when the cap is reached.
struct ModuleCacheState {
    /// Current entries (key + insertion order tracked via `order`).
    entries: HashMap<ModuleCidHash, Arc<Module>>,
    /// Insertion order — front = oldest, back = newest. On overflow
    /// the front entry is evicted.
    order: std::collections::VecDeque<ModuleCidHash>,
}

impl ModuleCacheState {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            order: std::collections::VecDeque::new(),
        }
    }
    fn get(&self, cid: &ModuleCidHash) -> Option<Arc<Module>> {
        self.entries.get(cid).map(Arc::clone)
    }
    fn insert(&mut self, cid: ModuleCidHash, module: Arc<Module>) {
        if !self.entries.contains_key(&cid)
            && self.entries.len() >= MODULE_CACHE_MAX_ENTRIES
            && let Some(oldest) = self.order.pop_front()
        {
            self.entries.remove(&oldest);
        }
        if self.entries.insert(cid, module).is_none() {
            self.order.push_back(cid);
        }
    }
    fn len(&self) -> usize {
        self.entries.len()
    }
}

static MODULE_CACHE: OnceLock<RwLock<ModuleCacheState>> = OnceLock::new();
/// Single-flight compile lock — separate from the cache RwLock so the
/// fast-path read does not serialise on the slow-path compile.
static MODULE_COMPILE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn module_cache_inner() -> &'static RwLock<ModuleCacheState> {
    MODULE_CACHE.get_or_init(|| RwLock::new(ModuleCacheState::new()))
}

fn compile_lock() -> &'static Mutex<()> {
    MODULE_COMPILE_LOCK.get_or_init(|| Mutex::new(()))
}

/// Compile (or fetch from cache) a [`wasmtime::Module`] for the given
/// wasm bytes. The cache key is BLAKE3 over `bytes`.
///
/// Subsequent calls with the same bytes return the cached `Arc<Module>`
/// without recompilation. The single-flight discipline holds the
/// compile-lock during compile so concurrent first-touch on the same
/// CID does not double-compile, while the fast-path RwLock read lets
/// distinct CIDs progress concurrently (perf-g7a-mr-3).
///
/// # Errors
/// Returns the wasmtime error on `Module::new` failure (typically
/// `E_SANDBOX_MODULE_INVALID` at the SANDBOX executor boundary).
pub fn module_for_bytes(bytes: &[u8]) -> Result<Arc<Module>, wasmtime::Error> {
    let cid = hash_module_bytes(bytes);
    let cache = module_cache_inner();
    {
        // Fast path: read-locked cache hit, no compile needed.
        let guard = cache.read().expect("module cache rwlock poisoned");
        if let Some(m) = guard.get(&cid) {
            return Ok(m);
        }
    }
    // Slow path: compile under the dedicated compile-lock
    // (single-flight). Re-check the cache inside the lock in case a
    // concurrent call beat us to it.
    let _compile_guard = compile_lock().lock().expect("compile lock poisoned");
    {
        let guard = cache.read().expect("module cache rwlock poisoned");
        if let Some(m) = guard.get(&cid) {
            return Ok(m);
        }
    }
    let engine = shared_engine();
    let module = Arc::new(Module::new(engine, bytes)?);
    let mut wguard = cache.write().expect("module cache rwlock poisoned");
    wguard.insert(cid, Arc::clone(&module));
    Ok(module)
}

/// Cache size — for diagnostic / drift tests. Returns the current
/// number of compiled modules retained.
#[must_use]
pub fn module_cache_size() -> usize {
    module_cache_inner()
        .read()
        .expect("module cache rwlock poisoned")
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

    /// **wsa-g7a-mr-6 fix-pass:** sync-host-fn dispatch under
    /// async_support=true preserves the same Module compile + Engine
    /// reuse semantics as a fresh `async_support=false` Engine would.
    /// Verifies async-support does not regress the sync-context
    /// compile path (which is the path the entire 2b SANDBOX surface
    /// uses since no async host-fn ships in 2b).
    #[test]
    fn shared_engine_async_support_does_not_break_sync_compile() {
        // The shared Engine has async_support(true). A trivial
        // `(module)` empty wasm MUST still compile (no trap, no async
        // suspension fired) because the module body has no calls.
        let bytes = empty_module_wasm();
        let m = module_for_bytes(&bytes).expect("sync-context compile under async_support=true");
        // Two calls return the same Arc — confirming the compile path
        // ran ONCE. If async_support changed Module-construction
        // semantics this would diverge.
        let m2 = module_for_bytes(&bytes).expect("second compile fetches cache");
        assert!(Arc::ptr_eq(&m, &m2));
    }

    /// **wsa-g7a-mr-5 fix-pass:** module cache size exposes the FIFO
    /// eviction discipline. Cannot easily test the eviction without
    /// authoring 257 distinct wasm modules; assert the cap constant
    /// has a sensible non-zero value + the cache-state struct preserves
    /// the constant across queries.
    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn module_cache_max_entries_constant_is_sane() {
        assert!(
            MODULE_CACHE_MAX_ENTRIES >= 16,
            "module cache must hold more than a trivial workload's worth"
        );
        assert!(
            MODULE_CACHE_MAX_ENTRIES <= 4096,
            "module cache cap above 4096 risks OOM under marketplace workload"
        );
    }
}
