//! G24-E wave-7 LANDED pin — gap #1b closure (br-r1-9;
//! `r2-test-landscape.md` §5).
//!
//! Compile-test proving the [`benten_platform_foundation::Renderer`]
//! trait surface is transport-agnostic — no Tauri-2.x-specific
//! webview-runtime types leak across the boundary. A future swap to
//! `tauri-runtime-verso` (Servo-based webview, per
//! `docs/future/phase-3-backlog.md` §15) becomes a sibling [`Renderer`]
//! impl + one-line `Cargo.toml` swap, not a breaking refactor.
//!
//! ## Why a compile-test
//!
//! The cheapest assertion that the trait surface is transport-agnostic
//! is "a verso-shape mock impl compiles against the SAME trait". If a
//! Tauri-2.x runtime type leaked through, this mock impl would fail
//! to compile — `cargo check` would error before the test ran.
//!
//! ## Closes
//!
//! Gap #1b (br-r1-9 in `r2-test-landscape.md` §5 gap-list)

#![allow(dead_code)]

use benten_platform_foundation::{MaterializerOutput, RenderError, Renderer};

mod mock_verso {
    //! Mirror of `tauri-runtime-verso`'s public surface to the level
    //! the [`Renderer`] trait would consume. Today the trait surface
    //! consumes ONLY [`MaterializerOutput`] — no webview-runtime
    //! types — so this mock module stands as documentation of the
    //! swap-target shape rather than a load-bearing dependency.

    /// Stand-in for the Servo-based Tauri webview handle. The real
    /// `tauri-runtime-verso::VersoWebview` would replace this.
    pub struct VersoWebview;

    /// Stand-in for the Verso IPC channel handle. The real
    /// `tauri-runtime-verso::VersoIpcChannel` would replace this.
    pub struct VersoIpcChannel;
}

/// A sibling [`Renderer`] impl that would exist in a hypothetical
/// `benten-renderer-tauri-verso` crate post-Verso-GA. Defined here as a
/// compile-test only.
struct VersoRendererImpl {
    _webview: mock_verso::VersoWebview,
    _channel: mock_verso::VersoIpcChannel,
}

impl Renderer for VersoRendererImpl {
    fn render(&self, _output: &MaterializerOutput) -> Result<(), RenderError> {
        // The real verso impl would call into `VersoWebview::emit` here.
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "tauri-runtime-verso"
    }
}

#[test]
fn renderer_trait_compiles_against_tauri_runtime_verso_shape_mock() {
    // The compile-test is THIS function body: if `VersoRendererImpl`
    // failed to satisfy `Renderer + Send + Sync`, the cast below
    // would fail at compile time before this test ran. The
    // backend_name read keeps the binding from being optimized out
    // and proves the dyn-dispatch surface is reachable.
    let r: &dyn Renderer = &VersoRendererImpl {
        _webview: mock_verso::VersoWebview,
        _channel: mock_verso::VersoIpcChannel,
    };
    assert_eq!(r.backend_name(), "tauri-runtime-verso");
}

#[test]
fn tauri_renderer_and_verso_renderer_satisfy_same_trait_via_dyn_dispatch() {
    use benten_renderer_tauri::{AdminUiManifest, TauriRenderer};

    // Both shapes implement the SAME trait — `dyn Renderer` accepts
    // either. The dispatch boundary stays one trait; the deployment
    // shape determines which backend ships in the binary (per
    // CLAUDE.md #17 + #19).
    let tauri = TauriRenderer::new_with_manifest(AdminUiManifest::default());
    let verso = VersoRendererImpl {
        _webview: mock_verso::VersoWebview,
        _channel: mock_verso::VersoIpcChannel,
    };

    let backends: Vec<&dyn Renderer> = vec![&tauri, &verso];
    let names: Vec<&str> = backends.iter().map(|r| r.backend_name()).collect();
    assert_eq!(names, vec!["tauri-2.x", "tauri-runtime-verso"]);
}
