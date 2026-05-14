//! Phase-4-Foundation R6-FP-E webview-driven E2E smoke test.
//!
//! Closes the half (ii) substantive arm of `br-r6-r1-3` MAJOR: a
//! regression in `WebviewWindowBuilder::with_csp()` integration
//! semantics or Tauri command-payload handling would have not
//! surfaced via the Rust-level `e2e_admin_shell_ipc.rs` pins (which
//! exercise `AdminShellState::dispatch` directly, not through the
//! Tauri command-invoke channel). This test drives a real
//! `tauri-driver` WebDriver session against the actual
//! `benten-admin-shell` binary running with the real Tauri 2.x
//! runtime + a real embedded webview (WebKit2GTK on linux; macOS +
//! Windows are platform caveats — see below).
//!
//! # What this asserts (pim-2 §3.6b production-arm + observable-
//! consequence + would-FAIL-if-no-op'd)
//!
//! 1. **PRODUCTION-ARM:** invokes a real `tauri-driver` subprocess +
//!    spawns the real `benten-admin-shell` binary built with the
//!    `tauri` feature on, including the `tauri_build::build()` codegen
//!    pass + a real `tauri::Builder::default().run(...)` boot. The
//!    webview-load path goes through Tauri's actual platform-specific
//!    WebView runtime.
//!
//! 2. **OBSERVABLE-CONSEQUENCE:**
//!    - The webview loads `webview-assets/index.html` successfully —
//!      asserts the page `<title>` resolves to the expected string.
//!    - The Tauri command-invoke channel correctly serializes a
//!      request from JS → Rust handler → JS response — asserts the
//!      `ipc_method_cap_bindings_command` command returns the
//!      canonical method-cap map.
//!    - The CSP applied at webview load forbids `'unsafe-eval'` —
//!      asserts a JS-side `eval("1+1")` evaluation in the webview
//!      is rejected (the strict CSP without `'unsafe-eval'` makes
//!      classic `eval` throw).
//!
//! 3. **WOULD-FAIL-IF-NO-OP'd:**
//!    - A regression dropping the `frontendDist` directive in
//!      `tauri.conf.json` → the webview loads an error page → the
//!      title assertion fails.
//!    - A regression that broke `tauri::generate_handler!` codegen
//!      → the `ipc_method_cap_bindings_command` invoke would error
//!      out → the round-trip assertion fails.
//!    - A regression that loosened the CSP to admit `'unsafe-eval'`
//!      → the `eval()` in the webview would succeed → the
//!      forbid-eval assertion fails.
//!
//! # Platform support matrix
//!
//! - **linux (`ubuntu-latest`):** SUBSTANTIVE — full WebDriver session
//!   via WebKit2GTK + `WebKitWebDriver` driven by `tauri-driver`.
//!   The CI lane `admin-shell-e2e.yml` runs this matrix cell on every
//!   push.
//! - **macOS (`macos-latest`):** BUILD-ONLY — Tauri's own
//!   tauri-driver project documents that macOS WebKit (WKWebView)
//!   lacks WebDriver-compatible bindings for embedded webviews;
//!   see <https://v2.tauri.app/develop/tests/webdriver/>. The CI
//!   lane runs a build-smoke (binary compiles + boots without
//!   panic) on this matrix cell as a substitute. **Not a benten-
//!   engine scope-reduction** — it is an upstream platform
//!   limitation explicitly named in the dispatch brief's "acceptable
//!   scope-reductions" clause.
//! - **Windows (`windows-latest`):** DEFERRED per the dispatch
//!   brief's cross-platform clause. WebView2 + Microsoft Edge
//!   WebDriver is supported by Tauri; CI matrix expansion lands at
//!   a follow-up. The test code path itself is OS-agnostic; only the
//!   CI workflow does not yet include Windows.
//!
//! When this test runs without `tauri-driver` in `PATH`, it skips
//! with a `println!` diagnostic (CI installs `tauri-driver` via
//! `cargo install` so the test always executes substantively under
//! CI). Local-dev runs without `tauri-driver` installed get a clear
//! "skipped" line.

#![cfg(feature = "tauri")]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::print_stdout)]
#![allow(clippy::too_many_lines)]

use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;

use fantoccini::ClientBuilder;
use tokio::time::sleep;

/// Port `tauri-driver` listens on by default. The fantoccini client
/// connects here; tauri-driver forwards to the platform-native
/// WebDriver (`WebKitWebDriver` on linux, `msedgedriver` on Windows).
const TAURI_DRIVER_PORT: u16 = 4444;

/// True if `tauri-driver` is available in PATH AND the host platform
/// supports embedded-webview WebDriver per the Tauri matrix
/// (linux + Windows; explicitly NOT macOS).
fn webdriver_supported_on_host() -> bool {
    if cfg!(target_os = "macos") {
        return false;
    }
    Command::new("tauri-driver")
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Path to the release binary that the E2E test launches via
/// tauri-driver's `--native-binary` argument. The CI lane invokes
/// `cargo build --release -p benten-admin-shell --features tauri`
/// before running the test so this path resolves.
fn admin_shell_binary_path() -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // tools/
    path.pop(); // workspace root
    path.push("target");
    path.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    path.push("benten-admin-shell");
    path
}

#[tokio::test]
async fn e2e_webview_smoke_loads_index_html_and_invokes_ipc_command() {
    if !webdriver_supported_on_host() {
        println!(
            "[e2e_webview_smoke] SKIPPED — webview WebDriver unavailable on this host. \
             macOS is excluded per https://v2.tauri.app/develop/tests/webdriver/ \
             (Apple WKWebView has no embedded-webview WebDriver binding); \
             other hosts skip when `tauri-driver` is not in PATH. \
             CI lane `admin-shell-e2e.yml` installs `tauri-driver` + runs \
             this test substantively on ubuntu-latest every push."
        );
        return;
    }

    let binary = admin_shell_binary_path();
    assert!(
        binary.exists(),
        "expected admin-shell binary at {} — run \
         `cargo build -p benten-admin-shell --features tauri` first",
        binary.display()
    );

    // Spawn tauri-driver subprocess. It launches the native binary +
    // proxies WebDriver commands to the platform's native WebDriver.
    // The Drop guard tears down the subprocess at scope exit even on
    // assertion failure (Rust's panic-unwind drops the guard).
    struct DriverGuard(std::process::Child);
    impl Drop for DriverGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let _guard = DriverGuard(
        Command::new("tauri-driver")
            .args([
                "--port",
                &TAURI_DRIVER_PORT.to_string(),
                "--native-binary",
                binary.to_str().unwrap(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("tauri-driver should spawn"),
    );

    // br-r6-r3-1 closure: active port-probe loop instead of a static 3-sec
    // sleep. The earlier static sleep was racy — tauri-driver had not always
    // bound port 4444 by the time fantoccini connected, surfacing as
    // `ConnectionRefused`. This loop polls until the TCP listener accepts
    // (up to ~10s), then proceeds. Combined with stdout/stderr inherit (above)
    // so any tauri-driver startup error is visible in CI logs rather than
    // silently lost.
    let probe_deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match TcpStream::connect(("127.0.0.1", TAURI_DRIVER_PORT)) {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < probe_deadline => {
                sleep(Duration::from_millis(200)).await;
            }
            Err(e) => {
                panic!(
                    "tauri-driver did not bind port {TAURI_DRIVER_PORT} \
                     within 10s of spawn — last connect error: {e}"
                );
            }
        }
    }

    // rustls 0.23+ requires an explicit `CryptoProvider` install since
    // no default is auto-selected when the feature-flag pinning is
    // disabled. Install ring (already in the workspace dep graph via
    // multiple transitives) before the fantoccini rustls builder
    // constructs its connection pool. Ignore the result — a sibling
    // dep may have installed already, in which case `install_default`
    // returns Err that we treat as "already installed".
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Connect fantoccini client to tauri-driver's WebDriver port.
    // Use `rustls()` constructor (the workspace bans openssl via
    // deny.toml so the fantoccini `native-tls` default-feature is
    // disabled — see Cargo.toml).
    let url = format!("http://127.0.0.1:{TAURI_DRIVER_PORT}");
    let client = ClientBuilder::rustls()
        .expect("rustls client builder")
        .connect(&url)
        .await
        .expect("fantoccini connect to tauri-driver");

    // --- Assertion 1: webview loaded the canonical index.html. -------
    let title = client.title().await.expect("title");
    assert_eq!(
        title.trim(),
        "Benten Admin Shell",
        "webview title must equal index.html <title> — drift indicates \
         either a `frontendDist` regression or index.html was not loaded"
    );

    // --- Assertion 2: Tauri command-invoke round-trip works. --------
    //
    // Calls `window.__TAURI__.core.invoke("ipc_method_cap_bindings_command")`
    // from JS and parses the resulting JSON. The map MUST equal the
    // canonical IPC method-cap-binding map per the renderer constant.
    let raw = client
        .execute_async(
            r#"
            const cb = arguments[arguments.length - 1];
            try {
              window.__TAURI__.core.invoke("ipc_method_cap_bindings_command")
                .then(v => cb(JSON.stringify({ok: v})))
                .catch(e => cb(JSON.stringify({err: String(e)})));
            } catch (e) {
              cb(JSON.stringify({err: "throw: " + String(e)}));
            }
            "#,
            vec![],
        )
        .await
        .expect("invoke ipc_method_cap_bindings_command");
    let parsed: serde_json::Value =
        serde_json::from_str(raw.as_str().expect("string result")).expect("parsed json");
    assert!(
        parsed.get("ok").is_some(),
        "Tauri command-invoke must succeed; got {parsed}"
    );
    let map = parsed.get("ok").unwrap().as_object().expect("map");
    // Spot-check one of the canonical entries.
    assert_eq!(
        map.get("engine.read_node_as").and_then(|v| v.as_str()),
        Some("graph:read"),
        "canonical method-cap binding must round-trip through Tauri command"
    );
    assert_eq!(
        map.get("plugin.install.consent").and_then(|v| v.as_str()),
        Some("plugin:install"),
        "canonical method-cap binding must round-trip through Tauri command"
    );

    // --- Assertion 3: CSP enforcement — classic eval() blocked. -----
    //
    // The renderer's `WEBVIEW_CSP_HEADER` admits `'wasm-unsafe-eval'`
    // (the wasm-only relaxation) but FORBIDS classic `'unsafe-eval'`.
    // A regression that admitted `'unsafe-eval'` would let `eval(...)`
    // succeed in the webview. We assert it throws.
    let eval_raw = client
        .execute_async(
            r#"
            const cb = arguments[arguments.length - 1];
            try {
              const r = eval("1+1");
              cb(JSON.stringify({ok: r}));
            } catch (e) {
              cb(JSON.stringify({err: String(e)}));
            }
            "#,
            vec![],
        )
        .await
        .expect("execute eval probe");
    let eval_parsed: serde_json::Value =
        serde_json::from_str(eval_raw.as_str().expect("string")).expect("parsed");
    assert!(
        eval_parsed.get("err").is_some(),
        "classic eval() must be blocked by CSP forbidding 'unsafe-eval'; \
         instead got: {eval_parsed}"
    );
    let err_str = eval_parsed.get("err").unwrap().as_str().unwrap_or("");
    // The error string from CSP-blocked eval typically contains
    // "Content Security Policy" or "unsafe-eval" or "EvalError"; we
    // accept any of those as the diagnostic surface.
    let looks_like_csp_block = err_str.contains("Content Security Policy")
        || err_str.contains("unsafe-eval")
        || err_str.contains("CSP")
        || err_str.contains("EvalError");
    assert!(
        looks_like_csp_block,
        "eval() rejection should surface a CSP-related diagnostic; got: {err_str}"
    );

    // Clean shutdown.
    client.close().await.expect("close fantoccini client");
}
