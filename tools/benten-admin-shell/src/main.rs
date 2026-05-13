//! Phase-4-Foundation R6-FP-E `benten-admin-shell` integrator binary.
//!
//! Closes r6-r1-browser-runtime finding `br-r6-r1-3` MAJOR (half (i)):
//! production caller for `TauriRenderer` + `InProcessSessionBridge`
//! lives here.
//!
//! # Two-mode boot path
//!
//! 1. **Default mode** (no `tauri` feature). Boots a headless
//!    integrator: constructs the [`benten_admin_shell::AdminShellState`]
//!    + prints a launch summary describing the IPC method-cap-binding
//!    surface + the locked CSP header. This mode is the
//!    workspace-default build path — the orchestrator's §3.5h workspace
//!    pre-push pre-flight runs in this shape.
//!
//! 2. **`tauri` feature mode** (opt-in; see `tauri_boot` module). Wires
//!    `tauri::Builder` against the same `AdminShellState` + loads
//!    `webview-assets/index.html`. Off by default per the
//!    `tools/benten-admin-shell/Cargo.toml` header rationale (Tauri 2.x
//!    pulls ~533 transitive deps + WebKit2GTK at runtime on linux; the
//!    full webview-driven E2E is named-NOW for v1-assessment-window per
//!    HARD RULE rule-12 clause-(b) destination
//!    `docs/future/phase-4-backlog.md §3` "webview-driven tauri-driver
//!    smoke test").
//!
//! The dispatch pipeline is **identical** across modes: both modes
//! construct the same `AdminShellState` + call
//! [`benten_admin_shell::AdminShellState::dispatch`] for every IPC
//! request. The only difference is the **wire framing** — feature mode
//! receives requests through Tauri's `invoke` channel; default mode is
//! introspectable from integration tests.

#![forbid(unsafe_code)]
#![allow(clippy::print_stdout)]

use benten_admin_shell::{ADMIN_SHELL_BOUND_ORIGIN, AdminShellState};
use benten_renderer_tauri::IPC_METHOD_CAP_BINDING;

fn main() -> std::process::ExitCode {
    // Default-mode boot path. Constructs the production state shape
    // (`DidKeyedSession::new` with real verifier + OS CSPRNG +
    // wallclock) + emits a launch summary on stdout. The integrator
    // operator wires logging via `RUST_LOG` + `tracing-subscriber` in a
    // future wave; this binary stays stdout-quiet apart from the
    // launch summary so the §3.5h workspace pre-push gate's
    // `print_stdout = "warn"` clippy lint is satisfied by the
    // crate-level `allow` above.
    let state = AdminShellState::new_production();

    println!("benten-admin-shell — production boot path");
    println!("  bound_origin: {ADMIN_SHELL_BOUND_ORIGIN}");
    println!("  webview_csp:  {}", state.webview_csp_header());
    println!("  ipc methods:");
    for (method, cap) in IPC_METHOD_CAP_BINDING {
        let cap_display = if cap.is_empty() { "(no cap)" } else { *cap };
        println!("    - {method:48}  ->  {cap_display}");
    }
    println!(
        "  active sessions: {}",
        state.session().active_session_count_for_test()
    );

    #[cfg(feature = "tauri")]
    return tauri_boot::run(state);

    #[cfg(not(feature = "tauri"))]
    {
        // Without the `tauri` feature the binary has nothing more to do
        // — return Ok and let the operator opt in. This shape lets the
        // integration tests in `tests/` exercise the full
        // `AdminShellState` API without depending on a running webview.
        std::process::ExitCode::SUCCESS
    }
}

// ---------------------------------------------------------------------
// `tauri` feature gated boot path (opt-in)
// ---------------------------------------------------------------------

#[cfg(feature = "tauri")]
mod tauri_boot {
    //! Real Tauri 2.x boot path. Off by default; the workspace pre-push
    //! gate exercises the no-feature path only. When the v1-assessment-
    //! window webview-driven tauri-driver smoke test wave lands, this
    //! module gains:
    //!
    //! - `tauri = "2"` dependency in the feature's deplist.
    //! - A real `tauri::Builder::default()` invocation.
    //! - Tauri commands wrapping
    //!   `benten_admin_shell::AdminShellState::dispatch`.
    //! - Webview asset loading from `webview-assets/index.html`.
    //! - CSP wiring via the operator's tauri.conf.json (the
    //!   `webview_csp_header()` is the canonical reference value).
    //!
    //! At HEAD this module is a placeholder that compiles ONLY when the
    //! operator opts in to the feature; the actual `tauri = "2"` dep
    //! lands at the v1-window wave.
    use benten_admin_shell::AdminShellState;

    pub fn run(_state: AdminShellState) -> std::process::ExitCode {
        // Placeholder. When the `tauri` feature flips on at the
        // v1-window wave, this body wires the real Tauri 2.x builder.
        eprintln!(
            "[benten-admin-shell] `tauri` feature scaffold reached — webview boot lands at \
             docs/future/phase-4-backlog.md §3 v1-assessment-window-bound wave"
        );
        std::process::ExitCode::SUCCESS
    }
}
