//! Phase-4-Foundation R6-FP-E `benten-admin-shell` integrator binary.
//!
//! Closes r6-r1-browser-runtime finding `br-r6-r1-3` MAJOR — production
//! caller for `TauriRenderer` + `InProcessSessionBridge`. Ben ratified
//! path-a-FULL on Q-R6-3 (2026-05-13): both halves (the scaffold + the
//! webview-driven E2E) land in this PR.
//!
//! # Two-mode boot path
//!
//! 1. **Default mode** (no `tauri` feature). Boots a headless
//!    integrator: constructs the [`benten_admin_shell::AdminShellState`]
//!    + prints a launch summary describing the IPC method-cap-binding
//!    surface + the locked CSP header. This mode is the
//!    workspace-default build path — the orchestrator's §3.5h workspace
//!    pre-push pre-flight runs in this shape. The `e2e_admin_shell_ipc`
//!    integration tests exercise the SAME `AdminShellState::dispatch`
//!    code path the `tauri` mode wraps in Tauri commands.
//!
//! 2. **`tauri` feature mode** (opt-in; CI lane `admin-shell-e2e.yml`
//!    flips this ON every run). Wires `tauri::Builder` against the
//!    same `AdminShellState`, registers Tauri commands wrapping
//!    `AdminShellState::dispatch`, loads `webview-assets/index.html`
//!    into the embedded webview, and runs the real WebView2 /
//!    WebKit2GTK / WKWebView runtime. The `tests/e2e_webview_smoke.rs`
//!    test drives a `tauri-driver` WebDriver session against the
//!    running binary to verify the full webview load + CSP
//!    enforcement + Tauri command-payload handling end-to-end.
//!
//! The dispatch pipeline is **identical** across modes: both modes
//! construct the same `AdminShellState` + call
//! [`benten_admin_shell::AdminShellState::dispatch`] for every IPC
//! request. The only difference is the **wire framing** — feature mode
//! receives requests through Tauri's `invoke` channel; default mode is
//! introspectable from integration tests.

#![forbid(unsafe_code)]
#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]

use benten_admin_shell::{ADMIN_SHELL_BOUND_ORIGIN, AdminShellState};
use benten_renderer_tauri::IPC_METHOD_CAP_BINDING;

fn main() -> std::process::ExitCode {
    // Default-mode boot path. Constructs the production state shape
    // (`DidKeyedSession::new` with real verifier + OS CSPRNG +
    // wallclock) + emits a launch summary on stdout.
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
        std::process::ExitCode::SUCCESS
    }
}

// ---------------------------------------------------------------------
// `tauri` feature gated boot path — REAL Tauri 2.x runtime
// ---------------------------------------------------------------------

#[cfg(feature = "tauri")]
mod tauri_boot {
    //! Real Tauri 2.x boot path. Active when the `tauri` cargo feature
    //! is on. The CI lane `admin-shell-e2e.yml` flips this ON every
    //! run; the `tests/e2e_webview_smoke.rs` test drives a real
    //! WebDriver session against a running instance of this binary.

    use std::sync::Arc;

    use benten_admin_shell::AdminShellState;
    use benten_renderer_tauri::{IpcRequest, ipc_method_cap_bindings};
    use tauri::Manager;

    /// Tauri command: dispatch an IPC envelope through
    /// `AdminShellState::dispatch`. The webview invokes this via
    /// `window.__TAURI__.core.invoke("dispatch_ipc", {method, payload})`.
    /// Session-token plumbing (challenge → handshake → token) is
    /// exercised by the Rust-level e2e_admin_shell_ipc tests; the
    /// webview-driven smoke exercises the un-authenticated rejection
    /// branches (MissingSession + MethodNotInAllowlist) at the real
    /// Tauri command-invoke surface, which is what br-r6-r1-3 named
    /// as the specific gap.
    #[tauri::command]
    fn dispatch_ipc(
        state: tauri::State<'_, Arc<AdminShellState>>,
        method: String,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let request = IpcRequest {
            method,
            payload,
            session: None,
        };
        match state.dispatch(request) {
            Ok(response) => Ok(response.payload),
            Err(err) => Err(format!("{:?}", err.error_code())),
        }
    }

    /// Tauri command: return the canonical IPC method-cap-binding map
    /// so the webview can sanity-check its surface knowledge. Used by
    /// the E2E smoke as a happy-path round-trip pin (tests Tauri
    /// command framing + JSON serialization end-to-end).
    #[tauri::command]
    fn ipc_method_cap_bindings_command() -> std::collections::BTreeMap<String, String> {
        ipc_method_cap_bindings()
    }

    /// Tauri command: return the canonical bound-origin string for
    /// the admin shell.
    #[tauri::command]
    fn admin_shell_bound_origin() -> &'static str {
        benten_admin_shell::ADMIN_SHELL_BOUND_ORIGIN
    }

    pub fn run(state: AdminShellState) -> std::process::ExitCode {
        let shared = Arc::new(state);
        let result = tauri::Builder::default()
            .manage(shared)
            .invoke_handler(tauri::generate_handler![
                dispatch_ipc,
                ipc_method_cap_bindings_command,
                admin_shell_bound_origin,
            ])
            .setup(|app| {
                if let Some(window) = app.get_webview_window("main")
                    && let Ok(url) = window.url()
                {
                    tracing::info!(
                        target: "benten-admin-shell",
                        "webview URL at boot: {url}"
                    );
                }
                Ok(())
            })
            .run(tauri::generate_context!());
        match result {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("[benten-admin-shell] tauri runtime error: {err}");
                std::process::ExitCode::FAILURE
            }
        }
    }
}
