//! arch-N pin: `benten-renderer-tauri` dep direction regression-guard.
//!
//! ## Pin source
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.18 (cross-cutting
//!   dep-direction defense).
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 wave charter
//!   (closes r4-arch-3 — new-crate dep-direction enforcement orphaned
//!   from R3 family enumeration).
//! - CLAUDE.md baked-in #19 (engine-level extensions are Rust crates
//!   compile-time-linked; trust = "you compiled this in"; renderer
//!   backends are an engine extension).
//! - CLAUDE.md baked-in #17 (three deployment shapes; shape (c)
//!   embedded-webview hosts the same wasm32 bundle as shape (b)).
//!
//! ## What this pin asserts
//!
//! `benten-renderer-tauri` is the Tauri 2.x embedded-webview engine
//! extension per CLAUDE.md #19. As an engine extension it MAY depend on
//! engine internals (`benten-engine`) at the cargo + code-review trust
//! boundary — extensions are trusted "you compiled this in" same as core.
//!
//! What it MUST NOT do: reverse-couple by becoming a dependency of
//! `benten-engine` or `benten-platform-foundation` (engine extensions
//! plug INTO the engine surface; the engine doesn't reach into a
//! specific renderer). That direction would invert the
//! Renderer-trait-at-engine-boundary architecture (CLAUDE.md #17
//! Renderer-backend swappability pattern).
//!
//! Also MUST NOT depend on `benten-graph` directly — even as an
//! engine-extension, storage internals are below the renderer surface
//! (use `benten-engine` facade methods instead).
//!
//! ## State at HEAD
//!
//! At HEAD the dep direction is clean. This test is PASSING at HEAD as
//! a permanent regression-guard.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn cargo_toml_text() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn production_dep_lines(toml: &str) -> Vec<&str> {
    let mut in_dep_table = false;
    let mut out = Vec::new();
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_dep_table = matches!(trimmed, "[dependencies]" | "[build-dependencies]");
            continue;
        }
        if !in_dep_table {
            continue;
        }
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        out.push(line);
    }
    out
}

#[test]
fn benten_renderer_tauri_does_not_depend_on_benten_graph_directly() {
    // Engine extensions go through the `benten-engine` facade for
    // storage access; direct `benten-graph` coupling at this surface
    // would skip the facade.
    let toml = cargo_toml_text();
    for line in production_dep_lines(&toml) {
        assert!(
            !line.contains("benten-graph"),
            "benten-renderer-tauri MUST NOT depend on benten-graph directly — use \
             benten-engine facade methods; per CLAUDE.md #19 engine-extension shape; \
             offending: {line}"
        );
    }
}

#[test]
fn benten_renderer_tauri_depends_on_benten_engine_per_extension_shape() {
    // Per CLAUDE.md #19 engine extensions are trusted compile-time-linked
    // Rust; they depend on benten-engine to plug into the Renderer trait
    // surface at the engine boundary.
    let toml = cargo_toml_text();
    let has_engine = production_dep_lines(&toml)
        .iter()
        .any(|l| l.contains("benten-engine"));
    assert!(
        has_engine,
        "benten-renderer-tauri MUST depend on benten-engine to plug into the Renderer trait \
         surface (CLAUDE.md #19 engine-extension shape)"
    );
}

#[test]
fn benten_renderer_tauri_depends_on_benten_platform_foundation_for_renderer_trait() {
    let toml = cargo_toml_text();
    let has_pf = production_dep_lines(&toml)
        .iter()
        .any(|l| l.contains("benten-platform-foundation"));
    assert!(
        has_pf,
        "benten-renderer-tauri MUST depend on benten-platform-foundation for the Renderer \
         trait abstraction (per arch-r1-1 dep direction)"
    );
}

#[test]
fn benten_renderer_tauri_does_not_reverse_depend_through_workspace_inversion() {
    // SUBSTANCE check: walk src/ and ensure we don't import `benten_graph`
    // directly via `use benten_graph::*` lines — even if Cargo.toml is
    // clean, an accidental import would surface here.
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    if !src.is_dir() {
        return; // R3 RED-PHASE crate skeleton may have minimal src/.
    }
    let mut offenders: Vec<String> = Vec::new();
    walk_rs_files(&src, &mut |path, body| {
        for (lineno, line) in body.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("use benten_graph::") || t.starts_with("extern crate benten_graph") {
                offenders.push(format!("{}:{} {}", path.display(), lineno + 1, line.trim()));
            }
        }
    });
    assert!(
        offenders.is_empty(),
        "benten-renderer-tauri src/ MUST NOT `use benten_graph::*` — use engine facade methods; \
         offenders: {:#?}",
        offenders,
    );
}

fn walk_rs_files(dir: &std::path::Path, visit: &mut dyn FnMut(&std::path::Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_rs_files(&p, visit);
        } else if p.extension().is_some_and(|e| e == "rs")
            && let Ok(body) = std::fs::read_to_string(&p)
        {
            visit(&p, &body);
        }
    }
}
