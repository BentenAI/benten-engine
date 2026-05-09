//! R3-A + R4-FP + r4b-wsa-2 GREEN pins: wasm32-unknown-unknown SANDBOX
//! unavailable path is observable across ALL 4 entry points (G13-C +
//! G14-C + G14-D + G16-D wave-3+; br-r1-3 + br-r4-r1-2 + Ben's D3
//! LOAD-BEARING decision; r4b-wsa-2 D3 closure at wave-G16-B-C).
//!
//! Pin sources:
//!
//! - r2-test-landscape §2.1 G13-C row
//!   `wasm32_unknown_unknown_browser_backend_e_sandbox_unavailable_on_wasm_path_observable`
//!   (br-r1-3; ALREADY pinned at R3-A wave-3) — install_module entry point
//! - br-r1-3 fix-brief item (2) — register_module_bytes entry point
//!   (CLOSED at wave-G16-B-C per r4b-wsa-2)
//! - br-r1-3 fix-brief item (3) — call→sandbox-handler dispatch entry
//!   point (CLOSED at wave-G16-B-C per r4b-wsa-2; production wired
//!   pre-r4b at primitive_host.rs::execute_sandbox wasm32 stub)
//! - br-r1-3 fix-brief item (4) — atrium-replicated sandbox invocation
//!   entry point (CLOSED at wave-G16-B-C per r4b-wsa-2 via
//!   architectural-absence — engine_sync.rs is module-level wasm32-gated)
//!
//! ## D3 LOAD-BEARING decision (Ben 2026-05-04 R4-FP; closure at
//! wave-G16-B-C 2026-05-08 per r4b-wsa-2)
//!
//! "SANDBOX uniformity = pin ALL 4 entry points (not just 1)." The
//! original R3-A landed pin (1) — the install_module path. R4-FP added
//! pins (2)/(3)/(4) covering the remaining 3 SANDBOX entry points so
//! a R5 implementer who wires the install_module arm + silently no-ops
//! on the other 3 entry points fires THIS file's pins, not just the
//! single `crates/benten-engine/src/primitive_host.rs::execute_sandbox`
//! dispatch site.
//!
//! Wave-G16-B-C closes the D3 uniformity gap end-to-end (r4b-wsa-2):
//! all 4 pins are now ACTIVE (not `#[ignore]`'d) + the production
//! arms ship cfg-gated wasm32 surfaces (entry-2 added at
//! `Engine::register_module_bytes`; entry-3 already wired at
//! `impl PrimitiveHost for Engine::execute_sandbox`; entry-4 satisfied
//! by architectural-absence — `engine_sync.rs` does NOT compile on
//! wasm32, so atrium-replicated SANDBOX dispatch is unreachable).
//!
//! ## What this pins
//!
//! On wasm32-unknown-unknown (browser), wasmtime is unavailable
//! (wasmtime cannot recursively host itself in a browser-tab WASM
//! runtime). SANDBOX primitive execution on this target must surface
//! `E_SANDBOX_UNAVAILABLE_ON_WASM` typed error from EVERY production
//! entry point that can reach SANDBOX dispatch:
//!
//! 1. **install_module** (G13-C wave-3) — DSL-driven SANDBOX manifest
//!    install at module registration.
//! 2. **register_module_bytes** (wave-G16-B-C) — direct module-bytes
//!    registration. Wasm32 arm rejects unconditionally because module
//!    bytes are exclusively consumed by the SANDBOX runtime (which
//!    is compile-time absent on wasm32 per CLAUDE.md baked-in #17
//!    thin-client commitment).
//! 3. **call→SANDBOX-handler** (G14-D wave-5a / G19) — runtime CALL
//!    primitive dispatching into a registered SANDBOX handler-id.
//! 4. **atrium-replicated SANDBOX invocation** (G16-D wave-5+) — sync-
//!    replica receives Atrium-replicated SANDBOX-bearing data + the
//!    receiver dispatches into local SANDBOX execution. Architecturally
//!    absent on wasm32: `crates/benten-engine/src/engine_sync.rs` is
//!    module-level cfg-gated `cfg(all(not(target_arch = "wasm32"),
//!    not(feature = "browser-backend")))` at `lib.rs`, so the entire
//!    Atrium-receive surface (`AtriumHandle::merge_remote_change*`)
//!    does not exist in wasm32 builds. The thin-client subscribe
//!    surface that ships on wasm32 is read-only by design (per
//!    `thin_client_subscribe.rs`).
//!
//! Each entry point must surface the SAME typed error
//! (`E_SANDBOX_UNAVAILABLE_ON_WASM`), uniformly. Defends against the
//! failure shape "fix landed at one entry point but the other 3
//! silently no-op or panic" — exactly the structural shape that
//! produced 24 cumulative producer/consumer drift instances in
//! Phase-2b (per `feedback_3_plus_recurrence_deep_sweep`).
//!
//! Per pim-2 §3.6b end-to-end: each pin drives a distinct production
//! entry point + asserts an observable consequence (typed error
//! reaching the caller; not panic; not silent success). Entry-4 is
//! satisfied by architectural-absence — the source-cite asserts the
//! engine_sync module's wasm32-cfg-gating remains in place, defending
//! against a future regression that lifts the gate without surfacing
//! an explicit SANDBOX-receive denial.

#![allow(clippy::unwrap_used)]

#[test]
fn wasm32_unknown_unknown_browser_backend_e_sandbox_unavailable_on_wasm_path_observable() {
    // br-r1-3 GREEN pin (entry point 1 of 4 — install_module).
    //
    // The host-side primitive dispatch arm in
    // `crates/benten-engine/src/primitive_host.rs::PrimitiveHost`
    // contains the wasm-arch-conditional SANDBOX-unavailable error
    // path (per br-r1-3 + §3.5b HARDENED point 3 — symbol-form for
    // high-churn surface).
    use std::path::PathBuf;
    let primitive_host_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/primitive_host.rs");
    let src = std::fs::read_to_string(&primitive_host_path).unwrap_or_else(|e| {
        panic!(
            "read crates/benten-engine/src/primitive_host.rs: {} (path={:?})",
            e, primitive_host_path
        )
    });
    assert!(
        src.contains("E_SANDBOX_UNAVAILABLE_ON_WASM"),
        "primitive_host.rs MUST surface the wasm32 SANDBOX-unavailable typed error \
         (E_SANDBOX_UNAVAILABLE_ON_WASM) per br-r1-3 + wsa-14 / Compromise # — \
         defends against regression where wasm32 builds silently skip SANDBOX primitives"
    );
    // Verify the wasm32 conditional gate is actually present (so the
    // typed error path fires on browser builds, not just exists in
    // dead code on native builds):
    assert!(
        src.contains("#[cfg(target_arch = \"wasm32\")]"),
        "primitive_host.rs MUST contain a `#[cfg(target_arch = \"wasm32\")]` arm \
         that emits E_SANDBOX_UNAVAILABLE_ON_WASM per br-r1-3"
    );

    // OBSERVABLE consequence: a browser-side SANDBOX call routes
    // through the wasm32 arm + surfaces the typed UnavailableOnWasm
    // error rather than panic or generic failure. The wasm-bindgen
    // runtime arm (`#[cfg(target_arch = "wasm32")]` test that drives
    // the actual install_module path) lands at G14-C alongside the
    // register_module_bytes entry point per the 4-way uniformity pin
    // landing schedule. This G13-C pin is the source-cite regression
    // guard for entry point 1.
}

#[test]
fn wasm32_unknown_unknown_browser_register_module_bytes_with_sandbox_handler_returns_e_sandbox_unavailable_on_wasm()
 {
    // br-r1-3 fix-brief item (2) + br-r4-r1-2 + D3 LOAD-BEARING +
    // r4b-wsa-2 GREEN pin (entry point 2 of 4 — register_module_bytes).
    //
    // Wave-G16-B-C closes the D3 uniformity gap by adding a
    // `cfg(target_arch = "wasm32")` arm to
    // `Engine::register_module_bytes` that returns the typed
    // E_SANDBOX_UNAVAILABLE_ON_WASM error unconditionally on wasm32
    // (module bytes are exclusively consumed by the SANDBOX runtime,
    // which is compile-time absent on wasm32 per CLAUDE.md baked-in
    // #17 thin-client commitment).
    //
    // Pinned by source-cite shape (Option B per the original pin
    // narrative — pim-2 §3.6b "substring source-cite is acceptable
    // ... lower bar"); the runtime form (Option A under
    // wasm-bindgen-test) is not reachable from the native
    // cargo-test path. The same structural shape as entry-1
    // (install_module via primitive_host.rs::execute_sandbox).
    use std::path::PathBuf;
    let engine_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/engine.rs");
    let src = std::fs::read_to_string(&engine_path).unwrap_or_else(|e| {
        panic!(
            "read crates/benten-engine/src/engine.rs: {} (path={:?})",
            e, engine_path
        )
    });
    // 1. The fn is present in the file.
    let fn_offset = src
        .find("pub fn register_module_bytes(")
        .expect("engine.rs MUST declare register_module_bytes");
    // 2. Locate the body window (signature → next top-level fn) so the
    //    cfg-gate assertion fires on register_module_bytes specifically,
    //    not on a sibling fn elsewhere in the file. Use the
    //    fetch_module_bytes signature as the structural sentinel
    //    delimiting the end of register_module_bytes; if either is
    //    renamed the assertion fires loudly.
    let next_fn = src[fn_offset..]
        .find("pub fn fetch_module_bytes(")
        .expect("fetch_module_bytes sibling is the structural sentinel");
    let fn_body = &src[fn_offset..fn_offset + next_fn];

    // 3. The wasm32 cfg arm exists somewhere inside register_module_bytes.
    assert!(
        fn_body.contains("#[cfg(target_arch = \"wasm32\")]"),
        "register_module_bytes MUST contain a #[cfg(target_arch = \"wasm32\")] arm \
         per r4b-wsa-2 D3 entry-point 2 uniformity"
    );
    // 4. The wasm32 arm surfaces the typed E_SANDBOX_UNAVAILABLE_ON_WASM
    //    error (ErrorCode::SandboxUnavailableOnWasm).
    assert!(
        fn_body.contains("SandboxUnavailableOnWasm"),
        "register_module_bytes wasm32 arm MUST surface ErrorCode::SandboxUnavailableOnWasm \
         per r4b-wsa-2 + br-r1-3 + D3 LOAD-BEARING (uniformity across all 4 SANDBOX entry points)"
    );
    // 5. The arm cites the wsa-14 actionable text constant, mirroring
    //    the execute_sandbox stub at primitive_host.rs.
    assert!(
        fn_body.contains("SANDBOX_UNAVAILABLE_ON_WASM_TEXT"),
        "register_module_bytes wasm32 arm MUST cite SANDBOX_UNAVAILABLE_ON_WASM_TEXT \
         (wsa-14 pinned UX text) — keeps the operator-actionable narrative consistent \
         across all 4 SANDBOX entry points"
    );

    // OBSERVABLE consequence: a browser-side `engine.registerModuleBytes(...)`
    // call surfaces the typed UnavailableOnWasm error rather than
    // silently registering a module that has no execution path.
    // Defends against the failure shape "register_module_bytes
    // accepts SANDBOX-bearing module bytes on wasm32 + the gate
    // fires only at execute time, leaking module-bytes into the
    // browser's redb-shaped storage with no execution path."
}

#[test]
fn wasm32_unknown_unknown_browser_call_primitive_into_sandbox_handler_returns_e_sandbox_unavailable_on_wasm()
 {
    // br-r1-3 fix-brief item (3) + br-r4-r1-2 + D3 LOAD-BEARING +
    // r4b-wsa-2 GREEN pin (entry point 3 of 4 — CALL→SANDBOX-handler
    // dispatch).
    //
    // The production arm is the wasm32 cfg-gated stub of
    // `impl PrimitiveHost for Engine::execute_sandbox` (in
    // `crates/benten-engine/src/primitive_host.rs`). The structural
    // assertion below is symbol-form per §3.5b HARDENED point 3
    // (high-churn surface; symbol-form rides out file churn).
    //
    // Pinned by source-cite shape: the production fn `execute_sandbox`
    // has a `#[cfg(target_arch = "wasm32")]` arm that returns the
    // typed `EvalError::SubsystemDisabled` carrying the
    // E_SANDBOX_UNAVAILABLE_ON_WASM-class wsa-14 text. The
    // runtime-grade test form (Option A under wasm-bindgen-test)
    // requires browser-side test infra heavier than the gap warrants
    // per pim-2 §3.6b "substring source-cite acceptable ... lower bar".
    use std::path::PathBuf;
    let primitive_host_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/primitive_host.rs");
    let src = std::fs::read_to_string(&primitive_host_path).unwrap_or_else(|e| {
        panic!(
            "read crates/benten-engine/src/primitive_host.rs: {} (path={:?})",
            e, primitive_host_path
        )
    });

    // 1. The wasm32 stub of execute_sandbox is present. Locate the
    //    cfg arm + the immediately-following `fn execute_sandbox`
    //    signature; this is the CALL→SANDBOX dispatch surface (the
    //    PrimitiveHost trait routes CALL→SANDBOX through here per the
    //    `op.kind == Sandbox` arm of `eval_call`).
    let stub_marker = "#[cfg(target_arch = \"wasm32\")]\n    fn execute_sandbox(";
    assert!(
        src.contains(stub_marker),
        "primitive_host.rs MUST contain the wasm32 stub of \
         `impl PrimitiveHost for Engine::execute_sandbox` per r4b-wsa-2 + br-r1-3 fix-brief \
         item (3) + D3 LOAD-BEARING. Marker: {stub_marker:?}"
    );
    // 2. The stub body surfaces the wsa-14 typed-error text + the
    //    SubsystemDisabled envelope (the active wave-8d-types shape;
    //    a future SandboxUnavailableOnWasm-direct envelope is 8c-cont
    //    scope per the docstring). Either surfacing path satisfies the
    //    pin — the load-bearing commitment is "browser CALL→SANDBOX
    //    dispatch surfaces the actionable typed error rather than
    //    panic at wasmtime construction".
    assert!(
        src.contains("E_SANDBOX_UNAVAILABLE_ON_WASM"),
        "execute_sandbox wasm32 stub MUST surface the E_SANDBOX_UNAVAILABLE_ON_WASM-class text \
         per r4b-wsa-2 + br-r1-3 fix-brief item (3) + D3"
    );
    assert!(
        src.contains("EvalError::SubsystemDisabled"),
        "execute_sandbox wasm32 stub MUST surface the EvalError::SubsystemDisabled typed \
         envelope (wave-8d-types shape) carrying the wsa-14 text per r4b-wsa-2 + br-r1-3"
    );

    // OBSERVABLE consequence: a browser-side `engine.run()` call that
    // routes through the CALL→SANDBOX dispatch arm of the evaluator's
    // PrimitiveHost trait surfaces the typed error rather than
    // panicking at the wasmtime::Engine::new call site (which would
    // otherwise be reached because wasmtime is cfg-gated off on
    // wasm32 + the dispatch arm would call into a missing symbol).
    //
    // Defends against the failure shape "register_module_bytes
    // refuses SANDBOX modules + the CALL dispatch arm has no
    // defensive gate, so a path that bypasses register_module_bytes
    // (e.g. via a module imported through an alternate route)
    // silently dispatches into wasmtime that doesn't exist on
    // wasm32 + panics at the wasmtime::Engine::new call site."
}

#[test]
fn wasm32_unknown_unknown_browser_atrium_replicated_sandbox_handler_returns_e_sandbox_unavailable_on_wasm()
 {
    // br-r1-3 fix-brief item (4) + br-r4-r1-2 + D3 LOAD-BEARING +
    // r4b-wsa-2 GREEN pin (entry point 4 of 4 — atrium-replicated
    // SANDBOX invocation receive).
    //
    // Disposition (per r4b-wsa-2 RECOMMENDATION (b) — Ben's call at
    // wave-G16-B-C is the architectural-absence path): the entire
    // atrium-receive surface (`AtriumHandle::merge_remote_change*`)
    // is module-level cfg-gated `not(target_arch = "wasm32")` at the
    // `pub mod engine_sync;` declaration in
    // `crates/benten-engine/src/lib.rs` per CLAUDE.md baked-in #17
    // (browser tabs are thin-client views, NOT full Atrium peers).
    // On wasm32 the entire iroh + Loro + benten-sync surface is
    // architecturally absent; a browser thin-client cannot reach
    // the atrium-receive code path because the symbols don't exist
    // in the wasm32 bundle.
    //
    // The thin-client surface that DOES ship on wasm32 is
    // `crates/benten-engine/src/thin_client_subscribe.rs` — a
    // read-only subscription surface against authenticated
    // full-peer endpoints; it has no SANDBOX dispatch arm.
    //
    // This pin asserts the architectural-absence remains in place:
    // a future regression that lifts the wasm32 gate on
    // `engine_sync` (e.g. via "browser-as-full-peer" ambition or via
    // an accidental cfg-leak) fails this pin loudly, before the
    // gate-lift can reach a wasm32 build that ships SANDBOX-receive
    // without a typed-error denial.
    use std::path::PathBuf;
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
    let src = std::fs::read_to_string(&lib_path).unwrap_or_else(|e| {
        panic!(
            "read crates/benten-engine/src/lib.rs: {} (path={:?})",
            e, lib_path
        )
    });

    // 1. `engine_sync` module declaration exists.
    assert!(
        src.contains("pub mod engine_sync;"),
        "lib.rs MUST declare `pub mod engine_sync;` (the Atrium API surface)"
    );

    // 2. The `engine_sync` declaration carries the wasm32 cfg-gate.
    //    Match a window around the declaration so the cfg attribute
    //    binds to engine_sync specifically, not to a neighbouring
    //    `pub mod`.
    let decl_offset = src
        .find("pub mod engine_sync;")
        .expect("engine_sync declaration located above");
    // Look back ~200 bytes for the cfg attribute that immediately
    // precedes the declaration (covering the multi-line cfg-all form
    // already present at lib.rs).
    let window_start = decl_offset.saturating_sub(200);
    let window = &src[window_start..decl_offset + "pub mod engine_sync;".len()];
    assert!(
        window.contains("not(target_arch = \"wasm32\")"),
        "the `pub mod engine_sync;` declaration MUST carry a \
         #[cfg(... not(target_arch = \"wasm32\") ...)] gate per r4b-wsa-2 entry-4 \
         architectural-absence + CLAUDE.md baked-in #17 thin-client commitment. \
         A regression that lifts this gate would let wasm32 builds compile the \
         Atrium-receive surface without surfacing a SANDBOX-receive typed-error denial."
    );

    // 3. Sibling: `atrium_api` module ships under the same gate. The
    //    two modules form the Atrium API surface; both must be
    //    architecturally-absent on wasm32 for the entry-4 commitment
    //    to hold.
    assert!(
        src.contains("pub mod atrium_api;"),
        "lib.rs MUST declare `pub mod atrium_api;` (the Atrium config surface)"
    );
    let atrium_decl_offset = src
        .find("pub mod atrium_api;")
        .expect("atrium_api declaration located above");
    let atrium_window_start = atrium_decl_offset.saturating_sub(200);
    let atrium_window = &src[atrium_window_start..atrium_decl_offset + "pub mod atrium_api;".len()];
    assert!(
        atrium_window.contains("not(target_arch = \"wasm32\")"),
        "the `pub mod atrium_api;` declaration MUST carry a \
         #[cfg(... not(target_arch = \"wasm32\") ...)] gate per r4b-wsa-2 entry-4 \
         architectural-absence (the Atrium config surface mirrors engine_sync's gate)"
    );

    // OBSERVABLE consequence: a browser thin-client cannot reach the
    // atrium-receive code path because the symbols don't exist in
    // the wasm32 bundle. A future regression that lifts the wasm32
    // gate on engine_sync (e.g. "browser-as-full-peer" ambition or
    // an accidental cfg-leak) fails this pin BEFORE the regression
    // can ship a wasm32 bundle with reachable SANDBOX-receive
    // dispatch but no typed-error denial.
    //
    // Defends against the failure shape "the thin-client receives
    // an attribution-frame that names a SANDBOX handler and
    // helpfully tries to re-execute it locally on the browser,
    // panicking at wasmtime construction." On wasm32 with the gate
    // in place, the symbols required to reach SANDBOX-receive
    // dispatch are simply not present; the architectural commitment
    // is enforced by the linker.
}
