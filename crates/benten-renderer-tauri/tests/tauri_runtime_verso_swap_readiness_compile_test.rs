//! G24-E wave-7 RED-PHASE pin — gap #1b closure (br-r1-9;
//! `r2-test-landscape.md` §5).
//!
//! Compile-test proving the `Renderer` trait's transport-agnostic
//! methods compile against a `tauri-runtime-verso`-shape mock. The
//! engine's renderer-backend swappability requires that the trait
//! surface does NOT leak Tauri-2.x-specific webview-runtime types
//! across the boundary; if it does, swapping to `tauri-runtime-verso`
//! (Servo-based webview, per `docs/future/phase-3-backlog.md` §15)
//! later in Phase 4-Foundation or beyond becomes a breaking refactor
//! instead of a one-line `Cargo.toml` swap.
//!
//! ## Why a compile-test
//!
//! The cheapest assertion that the boundary is transport-agnostic is
//! "the same trait impl compiles against a verso-shape mock". If a
//! Tauri-2.x runtime type leaked through, the mock impl would fail
//! to compile.
//!
//! ## RED-PHASE status
//!
//! `#[ignore]` until G24-E wave-7 lands the `Renderer` trait + a
//! `mock_verso` test-helper that mirrors the eventual
//! `tauri-runtime-verso` runtime shape.
//!
//! ## Closes
//!
//! Gap #1b (br-r1-9 in `r2-test-landscape.md` §5 gap-list)

#![allow(clippy::unwrap_used, dead_code, unused_imports)]

use benten_renderer_tauri as _renderer;

// `mock_verso` is the test-helper shape produced at G24-E wave-7. It
// mirrors the public types of `tauri-runtime-verso` to the level of
// detail the `Renderer` trait names. At R3 stub the module doesn't
// exist yet; the test is `#[ignore]`'d and won't reach the code path.
//
// mod mock_verso {
//     pub struct VersoWebview;
//     pub struct VersoIpcChannel;
//     // ... impls that mirror tauri-runtime-verso's public surface
// }

#[test]
#[ignore = "RED-PHASE: closes at R5 G24-E wave-7 (Renderer trait + mock_verso helper landing)"]
fn renderer_trait_compiles_against_tauri_runtime_verso_shape_mock() {
    // Production arm (G24-E wave-7):
    //
    //   use benten_renderer_tauri::Renderer;
    //
    //   struct VersoRendererImpl;
    //   impl Renderer for VersoRendererImpl {
    //       type Webview = mock_verso::VersoWebview;
    //       type IpcChannel = mock_verso::VersoIpcChannel;
    //       fn dispatch_ipc(&self, _req: IpcRequest) -> Result<IpcResponse, IpcError> {
    //           todo!("verso impl lives in benten-renderer-tauri-verso crate post-Verso-GA")
    //       }
    //       fn webview_csp_header(&self) -> &str { "default-src 'none'" }
    //   }
    //
    //   // Compile-test: if Renderer trait leaked tauri-2.x-specific
    //   // associated types, this impl block would not compile and
    //   // `cargo check` would fail before the test even runs.
    //   let _r: VersoRendererImpl = VersoRendererImpl;
    panic!("RED-PHASE: production surface lands at G24-E wave-7");
}
