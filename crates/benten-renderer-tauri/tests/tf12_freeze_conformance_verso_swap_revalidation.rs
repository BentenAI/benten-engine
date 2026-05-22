//! TF-12 obligation (6) — §4.56 Renderer-trait freeze RE-VALIDATION:
//! the `tauri_runtime_verso_swap_readiness_compile_test` still compiles
//! against the chosen Renderer-trait shape post-FREEZE (G-CORE-9).
//!
//! ADDL R3-C1 (Phase-4-Meta-Core; last R3 wave; freeze-time). TDD
//! red-phase. Tests-only — NO production source.
//!
//! ## Disjointness vs the EXISTING G24-E-landed compile-test (HARD)
//!
//! `tauri_runtime_verso_swap_readiness_compile_test.rs` ALREADY EXISTS
//! in this crate (a GREEN, G24-E wave-7 LANDED pin — br-r1-9). R3-C1
//! does NOT recreate or modify it. THIS file is a distinct `tf12_`
//! FREEZE re-validation pin: it asserts the FREEZE-INVARIANT that the
//! §4.56 Renderer-trait shape was NOT silently broken non-agnostic
//! during the G-CORE-9 v1-API-stabilization sweep (the §4.43 cluster
//! includes "§4.56 Renderer (path MUST keep the trait runtime-/
//! transport-agnostic; re-validate
//! `tauri_runtime_verso_swap_readiness_compile_test`)" — plan §0 row).
//! The existing test proves agnosticism TODAY; this pin proves the
//! FREEZE preserved it (a freeze regression-guard).
//!
//! ## §3.6g LITERAL discipline checklist (reproduced, not §-referenced)
//!
//!  1. Land-when = FREEZE. The RED pin carries
//!     `#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]`. The
//!     compile-time agnosticism arm (the dyn-Renderer mock) is a GREEN
//!     compile-guard (it must compile NOW and stay compiling — the
//!     would-FAIL is a *compile* error if the trait gains a
//!     transport-specific method, exactly the existing test's shape).
//!  2. Campaign-tail landed-vs-RED split (§3.5n): R3-C1 ground-truthed
//!     `benten-platform-foundation/src/materializer.rs:530` — the
//!     `Renderer` trait surface at ed03729a is `render(&self,
//!     &MaterializerOutput) -> Result<(), RenderError>` +
//!     `backend_name(&self) -> &'static str` (NO transport-specific
//!     types). The agnostic-shape GREEN arm reflects that. The
//!     FREEZE-RECORD arm (the contract must record §4.56's
//!     keep-agnostic constraint) is a freeze-DELIVERABLE not yet built
//!     → RED.
//!  3. SHAPE-not-SUBSTANCE (pim-18 / §3.6f): the GREEN arm is a REAL
//!     compile-test (a sibling `dyn Renderer` impl with verso-shaped
//!     handles — if a Tauri-2.x runtime type leaks through the trait,
//!     this fails to *compile*, the strongest possible would-FAIL).
//!     The RED arm asserts the FREEZE CONTRACT records the
//!     keep-agnostic invariant — a property invariant where the
//!     substantive consequence is a freeze-lock (same reasoning class
//!     as R3-B6's #838 seam-shape pin).
//!  4. pim-2 sub-rule-4 (§3.6b): exercises the SPECIFIC §4.56
//!     re-validate obligation, not an umbrella "rendering works".
//!  5. §3.13: no shared process-scoped static — per-test locals only.
//!  6. §3.5j: compiles + MSRV-1.95 clippy AND `cargo +stable clippy`
//!     (scoped to benten-renderer-tauri — never `--workspace`).
//!  7. §3.6e: introduces no stranded `#[ignore]` pin; the RED pin's
//!     named un-ignore destination IS G-CORE-9.
//!
//! Pin source: r2-test-landscape.md TF-12 obligation (6) + plan §0
//! §4.x v1-API-stabilization-sweep row (§4.56 Renderer re-validate) +
//! §1.A.FROZEN item 13 (engine↔host runtime-ownership / IPC boundary).

#![allow(dead_code)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use benten_platform_foundation::{MaterializerOutput, RenderError, Renderer};
use std::path::PathBuf;

/// Verso-shaped stand-in handles (mirror of the EXISTING
/// `tauri_runtime_verso_swap_readiness_compile_test`'s `mock_verso`,
/// re-expressed independently so this freeze re-validation pin does NOT
/// share state or a module with the landed test — §3.13 disjoint).
mod verso_shape {
    pub struct VersoWebview;
    pub struct VersoIpcChannel;
}

/// A sibling [`Renderer`] impl with verso-shaped handles. If the
/// G-CORE-9 freeze wave silently broke the `Renderer` trait into a
/// non-transport-agnostic shape (e.g. by adding a Tauri-2.x-specific
/// method or a webview-runtime associated type), THIS STRUCT WOULD FAIL
/// TO COMPILE — `cargo test --no-run` errors before any test executes.
/// That compile failure IS the would-FAIL signal (the strongest form).
struct FreezeRevalVersoRenderer {
    _webview: verso_shape::VersoWebview,
    _channel: verso_shape::VersoIpcChannel,
}

impl Renderer for FreezeRevalVersoRenderer {
    fn render(&self, _output: &MaterializerOutput) -> Result<(), RenderError> {
        Ok(())
    }
    fn backend_name(&self) -> &'static str {
        "tf12-freeze-reval-verso"
    }
}

/// GREEN compile-guard — the `Renderer` trait stays transport-agnostic.
/// This compiles today and MUST keep compiling through and past the
/// G-CORE-9 freeze (the §4.56 re-validate obligation). A transport-
/// specific trait mutation breaks the build here.
#[test]
fn renderer_trait_stays_transport_agnostic_through_freeze() {
    // `dyn Renderer` dispatch proves the trait is object-safe AND that
    // its signature consumes ONLY agnostic types (`&MaterializerOutput`
    // / `RenderError`) — no Tauri/Verso runtime type crosses the
    // boundary. (`MaterializerOutput` has no public ctor by design, so
    // — exactly like the landed G24-E compile-test — the agnosticism
    // proof is the *compilation* of `impl Renderer for
    // FreezeRevalVersoRenderer` + `dyn` dispatch, not a constructed
    // render call. A transport-specific trait mutation breaks the
    // build above; this assert only confirms dispatch executed.)
    let r: &dyn Renderer = &FreezeRevalVersoRenderer {
        _webview: verso_shape::VersoWebview,
        _channel: verso_shape::VersoIpcChannel,
    };
    assert_eq!(r.backend_name(), "tf12-freeze-reval-verso");
}

/// RED — un-ignore at G-CORE-9. The FROZEN-INTERFACE CONTRACT records
/// the §4.56 keep-the-Renderer-trait-runtime-/transport-agnostic
/// constraint AND that the verso swap-readiness compile-test was
/// re-validated at the freeze. Would-FAIL if the freeze ships without
/// recording the constraint (a Composing-time consumer could then add
/// a transport-specific method, foreclosing the Verso/Electron/native-
/// toolkit swap-target thesis — CLAUDE.md #17/#19).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]
fn freeze_contract_records_renderer_agnostic_revalidation() {
    let contract = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs/V1-FROZEN-INTERFACE.md");
    let body = std::fs::read_to_string(&contract).unwrap_or_else(|e| {
        panic!(
            "TF-12 (6): docs/V1-FROZEN-INTERFACE.md must exist at \
             G-CORE-9 and record the §4.56 Renderer-trait keep-agnostic \
             constraint; error: {e}"
        )
    });
    assert!(
        body.contains("Renderer")
            && (body.contains("transport-agnostic")
                || body.contains("runtime-agnostic")
                || body.contains("transport-/transport") // tolerate phrasing
                || body.contains("§4.56")
                || body.contains("4.56")),
        "TF-12 (6): the FROZEN-INTERFACE CONTRACT must record that the \
         §4.56 Renderer trait stays runtime-/transport-agnostic and the \
         verso swap-readiness compile-test was re-validated at the \
         freeze (§1.A.FROZEN item 13 / plan §0 §4.x row)."
    );
}

// ===========================================================================
// R3-W5 EXTENSION (Phase-4-Meta-Core R3 — cross-wave-integration partition).
// W5 owns the freeze-conformance arms per R2 §7. The pre-existing pins
// cover the Verso swap-readiness revalidation. W5 extension adds the
// cross-wave coupling with §1.A.FROZEN item 13 engine↔shell bridged-dual-
// runtime boundary: the renderer-tauri MUST NOT thread a host-shell tokio
// runtime handle through the engine surface (deployment-r1-1 + plan §8-C).
// ===========================================================================

/// R3-W5 EXTENSION — engine↔shell bridged-dual-runtime boundary
/// holds in the renderer-tauri crate. §1.A.FROZEN item 13: the
/// engine owns its tokio runtime; NO shell-runtime handle/ownership
/// crosses the engine API surface. This pin asserts the
/// renderer-tauri does not declare an `Engine::with_runtime(...)` or
/// similar host-runtime-threading entry-point (the deployment-r1-1
/// load-bearing constraint).
#[test]
fn r3_w5_extension_engine_host_shell_dual_runtime_boundary_no_runtime_threaded() {
    // §3.6f waiver: structural source-scan pin (same waiver class as
    // the existing renderer-tauri pins). The forbidden patterns are
    // named so a future agent that adds a `tauri::Builder::with_runtime`
    // hookup that threads INTO the engine fails this assertion.
    let src_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    if !src_dir.exists() {
        // The crate may have its source elsewhere; the test
        // tolerates absence at HEAD (renderer-tauri may not yet have
        // committed its src/ shape). Becomes load-bearing post-build.
        return;
    }
    let mut violating_sites: Vec<String> = Vec::new();
    fn visit_dir(dir: &std::path::Path, violations: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for ent in entries.flatten() {
            let p = ent.path();
            if p.is_dir() {
                visit_dir(&p, violations);
            } else if p.extension().and_then(|e| e.to_str()) == Some("rs")
                && let Ok(body) = std::fs::read_to_string(&p)
            {
                // Forbidden patterns:
                //   `Engine::with_runtime`  — threading runtime into engine.
                //   `Engine::new_with_handle` — threading host handle in.
                //   `tauri::Builder::with_runtime` + an engine surface call
                //     pattern. We conservatively flag any `with_runtime`
                //     in renderer-tauri sources that mentions engine.
                let lc = body.to_ascii_lowercase();
                if lc.contains("engine::with_runtime") || lc.contains("engine::new_with_handle") {
                    violations.push(format!(
                        "{}: engine-runtime-threading method found",
                        p.display()
                    ));
                }
                // The composite check: `with_runtime` AND `engine` in
                // the same file. False-positives possible if the
                // renderer-tauri has unrelated "with_runtime" content,
                // but the §1.A.FROZEN item 13 constraint is strict.
                if lc.contains("with_runtime")
                    && lc.contains("engine")
                    && !lc.contains("// ALLOWED: shell-runtime-only")
                {
                    violations.push(format!("{}: 'with_runtime' co-occurs with 'engine' (audit needed; add `// ALLOWED: shell-runtime-only` comment marker to suppress if the use is genuinely shell-internal)", p.display()));
                }
            }
        }
    }
    visit_dir(&src_dir, &mut violating_sites);
    assert!(
        violating_sites.is_empty(),
        "§1.A.FROZEN item 13 / §8-C bridged-dual-runtime boundary: \
         renderer-tauri MUST NOT thread a host-shell tokio runtime \
         handle through the engine surface (deployment-r1-1). \
         Findings: {violating_sites:?}. The engine owns its own \
         runtime; explicit channel/IPC boundary is the contract. \
         Would-FAIL if a future agent shortcuts the boundary."
    );
}
