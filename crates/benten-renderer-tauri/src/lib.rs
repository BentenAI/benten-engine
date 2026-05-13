//! Phase-4-Foundation Tauri 2.x renderer backend.
//!
//! # R3 RED-PHASE stub
//!
//! This crate is the engine-level [Tauri 2.x](https://v2.tauri.app/)
//! renderer backend per `CLAUDE.md` baked-in commitment #19 (engine
//! extensions are Rust crates compile-time-linked; trust = "you compiled
//! this in"). It hosts an embedded webview that loads the same
//! `wasm32-unknown-unknown` admin UI v0 bundle the browser-tab
//! deployment shape uses, communicating with the embedded full peer via
//! in-process IPC instead of `fetch` (deployment shape (c) per
//! `CLAUDE.md` #17).
//!
//! At R3 the crate is a placeholder so Family F2 test pins compile-but-
//! fail at the `use` line (canonical RED-phase shape). G24-E wave-7
//! fills the production logic:
//!
//! - **IPC allowlist** — per-method capability-binding against the admin
//!   UI manifest's `requires` envelope (T3 in `admin-ui-v0-threat-model`).
//! - **CSP enforcement** — `script-src 'self' 'wasm-unsafe-eval'`;
//!   `connect-src 'self' tauri://*`; `style-src 'self'`; `font-src 'self'`;
//!   `default-src 'none'`.
//! - **`tauri-runtime-verso` swap-readiness** — transport-agnostic
//!   `Renderer` trait shape so the engine can swap webview runtimes
//!   when Verso matures (br-r1-9; tracked at
//!   `docs/future/phase-3-backlog.md` §15).
//! - **3-rung baked-in #17 defense extension** — `wasm32-objdump`
//!   forbidden-prefix list for the wasm32 build of this crate's bundle
//!   (br-r1-4 + br-r1-13).
//!
//! # Trust model
//!
//! Engine extensions are out of scope for the Class B β read-side
//! gating boundary. `benten-renderer-tauri` does NOT pass requests
//! through `Engine::read_node_as` — it IS the engine. The boundary is
//! `cargo` and code review.

#![allow(dead_code, missing_docs)]

/// R3 placeholder. G24-E wave-7 fills this surface.
pub fn placeholder() {}
