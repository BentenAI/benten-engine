//! Phase-4-Meta-Core — TF-10 RED-PHASE (R3-B6) — #838 renderer-tauri
//! IPC-seam SHAPE pin: the deliberate compile-time const-allowlist
//! T3-defense, NOT a registration API/gap.
//!
//! ============================================================================
//! RED-PHASE STATUS — un-ignore at the **G-CORE-9-coupled freeze**
//! (the §1.A.FROZEN item 13 decision for #838).
//! ============================================================================
//!
//! `#[ignore = "RED-PHASE: un-ignore at G-CORE-DSL/G-CORE-9 freeze"]`.
//! §3.6e: the closing wave sweeps + un-ignores; the reviewer verifies
//! LANDING-STATUS, not just spec-pin presence.
//!
//! ============================================================================
//! ⚠️ FREEZE-BIAS HAZARD GUARD (r2 §4-A TF-10 trap — LOAD-BEARING)
//! ============================================================================
//!
//! #838 is **NOT a missing-registration-seam gap**. At ed03729a the
//! Tauri IPC surface is a DELIBERATE T3-defense: a compile-time
//! `pub const IPC_METHODS: &[IpcMethod]` allowlist
//! (`crates/benten-renderer-tauri/src/lib.rs:127`) with an explicit
//! baseline-update + admin-UI-v0 manifest-review gate. Silent IPC
//! surface expansion is a manifest-bypass risk; the const-allowlist is
//! the defense, by construction.
//!
//! Per r2 §4-A: a test that presumes a registration API
//! mischaracterizes the deliberate const-allowlist as a "gap". This
//! file therefore asserts the **const-allowlist PROPERTY HOLDS**
//! (compile-time `&'static [IpcMethod]`, every entry a static name +
//! cap binding, no runtime registration mutator) — it does NOT assert
//! that a registration seam exists, and it MUST NOT be rewritten to do
//! so. The §1.A.FROZEN item 13 freeze DECIDES whether v1 keeps the
//! const-allowlist property or adds a registration affordance; this
//! pin asserts the property the freeze is deciding ABOUT is intact and
//! unchanged-by-accident going into that decision (so the freeze
//! decision is made deliberately, not drifted into).
//!
//! ============================================================================
//! GROUND-TRUTH (synced HEAD ed03729a — verified by the R3 author)
//! ============================================================================
//!
//!   crates/benten-renderer-tauri/src/lib.rs
//!     :105  pub struct IpcMethod { name: &'static str, cap: CapRequirement }
//!     :127  pub const IPC_METHODS: &[IpcMethod] = &[ ...8 entries... ]
//!     :173  pub fn ipc_method(name) -> Option<&'static IpcMethod>
//!           (the SINGLE lookup; rung-1 allowlist + rung-2 cap binding)
//!     :180  pub fn ipc_method_names() -> impl Iterator<Item=&'static str>
//!   8 entries: engine.read_node_as / engine.call_as /
//!     engine.subscribe_via_on_change_as_with_cursor / engine.list_caps
//!     / engine.identity.user_did / plugin.manifest.review /
//!     plugin.install.consent / ui.notify
//!
//!   There is NO registration mutator (`register_ipc_method`,
//!   `add_method`, an `&mut` accessor, an interior-mutable registry)
//!   anywhere in the crate — by design. The drift-detector
//!   `ipc_method_name_stability_drift_detector.rs` already couples the
//!   name-set to `docs/public-api/benten-renderer-tauri.json`; THIS
//!   pin is the complementary SEAM-SHAPE invariant (the allowlist is a
//!   COMPILE-TIME CONST, not a runtime-registered set) for the
//!   §1.A.FROZEN item 13 freeze decision.
//!
//! ============================================================================
//! SHAPE-not-SUBSTANCE (pim-18): the assertions exercise the real
//! public `IPC_METHODS` const + `ipc_method` lookup (the production
//! T3-defense surface), not a sentinel. WOULD-FAIL if the allowlist
//! is silently turned into a runtime-mutable registry (the exact
//! manifest-bypass regression the const-allowlist defends against).
//! ============================================================================

#![allow(clippy::unwrap_used)]
// The `let _x: T = ...;` bindings below are DELIBERATE type-level
// compile-time assertions — the type annotation IS the seam-shape
// guard (e.g. `&'static [IpcMethod]` only coerces from a const, not a
// runtime registry). The bound value is intentionally unused; the
// binding's TYPE is the load-bearing check. Suppress clippy's
// no-effect-underscore-binding here (the "no effect" is the point —
// the effect is at typeck, not runtime).
#![allow(clippy::no_effect_underscore_binding)]

use benten_renderer_tauri::{IPC_METHODS, IpcMethod, ipc_method, ipc_method_names};

// ---------------------------------------------------------------------------
// SEAM-SHAPE arm 1 — IPC_METHODS is a compile-time `&'static
// [IpcMethod]` const. This binding itself is the assertion: a
// `&'static` reference can only be produced by a `const`/`static`
// (not a runtime-registered collection). If a future change turns the
// allowlist into a `Vec`/`OnceLock<Vec<..>>`/registry, the `&'static
// [IpcMethod]` type no longer holds and this test FAILS TO COMPILE —
// which IS the would-FAIL signal for "the const-allowlist property
// regressed into a registration API".
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-DSL/G-CORE-9 freeze"]
fn ipc_838_allowlist_is_compile_time_static_const_not_runtime_registry() {
    // Type-level assertion: `IPC_METHODS` coerces to `&'static
    // [IpcMethod]`. This binding fails to compile if the const is
    // replaced by a non-'static / runtime-built registry — the
    // structural guard the freeze-bias hazard requires.
    let _static_slice: &'static [IpcMethod] = IPC_METHODS;

    // The set is non-empty and stable at the known 8 production
    // entries (the deliberate T3 surface — NOT a gap to be "filled").
    assert_eq!(
        IPC_METHODS.len(),
        8,
        "#838: the IPC allowlist is the deliberate T3-defense surface \
         (8 production methods at ed03729a); a count change here is a \
         deliberate baseline+manifest-review event, NOT a 'fill a gap' \
         — see §1.A.FROZEN item 13."
    );

    // Every entry is a compile-time static binding (static name +
    // static cap requirement) — the property the freeze decides about.
    for m in IPC_METHODS {
        let _name: &'static str = m.name;
        assert!(
            !m.name.is_empty(),
            "#838: every allowlist entry carries a static method name"
        );
    }
}

// ---------------------------------------------------------------------------
// SEAM-SHAPE arm 2 — the single `ipc_method` lookup IS the seam (rung-1
// allowlist membership + rung-2 cap binding). An unknown method is NOT
// resolvable — there is NO registration path by which it could become
// resolvable at runtime. This pins the "no runtime registration
// affordance" property WITHOUT presuming one should exist.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-DSL/G-CORE-9 freeze"]
fn ipc_838_unknown_method_unresolvable_no_runtime_registration_path() {
    // A known method resolves (rung-1) and carries its static cap
    // binding (rung-2) — exercising the real production lookup.
    let known = ipc_method("engine.read_node_as")
        .expect("#838: a known allowlist method resolves via the const seam");
    let _cap = known.cap; // rung-2 cap binding is a compile-time field

    // An arbitrary method name is NOT on the const allowlist and there
    // is NO API to register it. `ipc_method` returns None — and stays
    // None, because the allowlist is compile-time-fixed. (This asserts
    // the const-allowlist PROPERTY; it does NOT assert a registration
    // seam is missing-and-should-exist — that is the §1.A.FROZEN
    // item 13 freeze DECISION, not a gap.)
    assert!(
        ipc_method("attacker.exfiltrate").is_none(),
        "#838: a non-allowlisted method is unresolvable; the \
         const-allowlist has no runtime registration path (the \
         deliberate manifest-bypass T3-defense)."
    );

    // The name-set iterator reflects exactly the compile-time const
    // (the drift-detector couples this to the public-api baseline; this
    // pin asserts the iterator is over the CONST, closing the
    // seam-shape side of #838).
    let names: Vec<&'static str> = ipc_method_names().collect();
    assert_eq!(
        names.len(),
        IPC_METHODS.len(),
        "#838: ipc_method_names() enumerates exactly the compile-time \
         const IPC_METHODS (no hidden runtime-registered entries)."
    );
}
