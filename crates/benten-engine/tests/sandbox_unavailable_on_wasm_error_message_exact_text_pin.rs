//! Phase 2b G7-C — wsa-14 typed-error UX text pin.
//!
//! Pin source: plan §3 G7-C (`tests/sandbox_unavailable_on_wasm_error_message_exact_text_pin`).
//!
//! The `E_SANDBOX_UNAVAILABLE_ON_WASM` error message is load-bearing
//! for browser/wasm32 operators — it MUST name (a) the failure mode,
//! (b) the Phase-3 P2P escape hatch (route to a Node-WASI peer), and
//! (c) the local-development workaround (run via @benten/engine in a
//! Node.js process). Any rename or shortening of the text MUST update
//! this pin in the same commit so a drive-by edit cannot silently
//! degrade the operator UX.
//!
//! The text lives in
//! `crates/benten-engine/src/engine_sandbox.rs::SANDBOX_UNAVAILABLE_ON_WASM_TEXT`
//! and is also documented in `docs/SANDBOX-LIMITS.md` §5. This test
//! asserts the constant is byte-identical to the plan-§3-G7-C-required
//! text.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::SANDBOX_UNAVAILABLE_ON_WASM_TEXT;

/// `sandbox_unavailable_on_wasm_error_message_exact_text_pin` — plan §3 G7-C.
///
/// Pins the EXACT wsa-14 UX text so a drive-by edit cannot silently
/// degrade the operator-actionable message.
#[test]
fn sandbox_unavailable_on_wasm_error_message_exact_text_pin() {
    let expected = "SANDBOX is unavailable in browser/wasm32 builds. Author handlers in browser \
                    context for execution against a Node-WASI peer (Phase 3 P2P sync — see \
                    ARCHITECTURE.md). For local development without a peer, run the engine via \
                    @benten/engine in a Node.js process.";

    assert_eq!(
        SANDBOX_UNAVAILABLE_ON_WASM_TEXT, expected,
        "wsa-14: the SANDBOX_UNAVAILABLE_ON_WASM_TEXT constant must match the plan-§3-G7-C \
         required UX text byte-for-byte. If you intentionally changed the text, update this \
         pin + docs/SANDBOX-LIMITS.md §5 in the same commit."
    );
}

/// Companion check — the constant MUST mention the three load-bearing
/// elements even if the surrounding wording shifts. This is a weaker
/// invariant kept alongside the byte-exact pin so a future
/// well-considered text refresh that updates BOTH the constant and
/// this test still preserves the operator-actionable elements.
#[test]
fn sandbox_unavailable_on_wasm_text_names_three_load_bearing_elements() {
    let text = SANDBOX_UNAVAILABLE_ON_WASM_TEXT;

    assert!(
        text.contains("wasm32") || text.contains("browser"),
        "text must name the failure-mode platform"
    );
    assert!(
        text.contains("Node") && text.contains("Phase 3"),
        "text must name the Phase-3 P2P escape hatch"
    );
    assert!(
        text.contains("@benten/engine"),
        "text must name the local-development workaround entry point"
    );
}
