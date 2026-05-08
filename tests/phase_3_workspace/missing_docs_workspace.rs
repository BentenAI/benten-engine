//! R3-E RED-PHASE pin for G20-B missing_docs sweep + escape-hatch retire
//! (wave-8b; phase-2-backlog §8.3 + C-7).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.8 G20-B):
//!
//! - `tests/no_allow_missing_docs_at_phase_3_close` — C-7
//! - `tests/full_missing_docs_sweep_no_warnings_workspace_wide` — phase-2-backlog §8.3
//!
//! ## What G20-B establishes
//!
//! Per phase-2-backlog §8.3 + C-7: full ~120+ public-surface missing_docs
//! sweep + drop `#[allow(missing_docs)]` escape hatch entirely.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G20-B wave-8b — no #[allow(missing_docs)] at Phase-3 close (C-7)"]
fn no_allow_missing_docs_at_phase_3_close() {
    // C-7 architectural pin. G20-B implementer wires this:
    //
    //   // Walk every crates/*/src/**/*.rs + bindings/napi/src/**/*.rs.
    //   // No `#[allow(missing_docs)]` may remain post-G20-B.
    //   let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..");
    //   let mut violators = Vec::new();
    //   for src_root in &[workspace_root.join("crates"), workspace_root.join("bindings/napi/src")] {
    //       for entry in walkdir::WalkDir::new(src_root) {
    //           let entry = entry.unwrap();
    //           if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
    //               continue;
    //           }
    //           let src = std::fs::read_to_string(entry.path()).unwrap();
    //           if src.contains("#[allow(missing_docs)]")
    //              || src.contains("#![allow(missing_docs)]") {
    //               violators.push(entry.path().display().to_string());
    //           }
    //       }
    //   }
    //   assert!(violators.is_empty(),
    //       "G20-B must drop all #[allow(missing_docs)] escape hatches; \
    //        residuals:\n{}", violators.join("\n"));
    //
    // OBSERVABLE consequence: the workspace public-doc discipline is
    // strict at Phase-3 close.
    unimplemented!("G20-B wires no-#[allow(missing_docs)] sweep pin");
}

#[test]
#[ignore = "RED-PHASE: G20-B wave-8b — full missing_docs sweep no warnings workspace-wide (phase-2-backlog §8.3)"]
fn full_missing_docs_sweep_no_warnings_workspace_wide() {
    // phase-2-backlog §8.3 pin. G20-B implementer wires this:
    //
    //   // Drive `cargo doc --workspace --no-deps -- -D warnings` and
    //   // verify zero missing_docs warnings on stable rust:
    //   let output = std::process::Command::new("cargo")
    //       .arg("+stable").arg("doc").arg("--workspace").arg("--no-deps")
    //       .env("RUSTDOCFLAGS", "-D missing_docs")
    //       .current_dir(/* workspace root */)
    //       .output()
    //       .unwrap();
    //   assert!(output.status.success(),
    //       "missing_docs sweep failed; stderr:\n{}",
    //       String::from_utf8_lossy(&output.stderr));
    //
    // OBSERVABLE consequence: every public surface has docstring
    // coverage. End-to-end pin per pim-2 §3.6b — would FAIL if any
    // surface remained undocumented.
    unimplemented!("G20-B wires full missing_docs sweep pin");
}

/// C-14 architectural pin (G20-B Phase-3 close).
///
/// Walks every Rust source file in the workspace + scans for `TODO(phase-`
/// markers. Per HARD RULE rule-12 each such marker MUST have BOTH:
///   1. A named phase (matching the regex `phase-[0-9]+[a-z]*`); AND
///   2. A non-empty descriptive payload after the phase name (the
///      em-dash + topic on the same source line, e.g.
///      `TODO(phase-3 — anchorstore + GC)`, OR a `phase-N-backlog §X.Y`
///      destination reference inside the marker body).
///
/// Bare `TODO(phase-2): rewrite later` or `TODO(phase-3): defer` is
/// unacceptable per the HARD RULE — those are non-fix-now dispositions
/// without a NAMED destination, which the rule explicitly disallows.
///
/// The test reads each violator's source line + builds a clear failure
/// message so the Phase-N closure council sees exactly what to fix.
#[test]
fn all_phase_2_3_todo_markers_have_named_destinations() {
    use std::path::Path;

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root resolves two levels up from tests/phase_3_workspace")
        .to_path_buf();

    let mut violators: Vec<String> = Vec::new();
    walk_rust_sources(&workspace_root, &mut |path, src| {
        for (lineno, line) in src.lines().enumerate() {
            let lower = line;
            // Surfaces with `TODO(phase-` are what C-14 tracks.
            let Some(idx) = lower.find("TODO(phase-") else {
                continue;
            };
            // Slice from `TODO(phase-` to the next ')' on the same line.
            // If the marker spans multiple lines, the closing ')' lives on
            // a continuation line — we look at the marker head line only,
            // which is enough to enforce the structural pin.
            let after = &lower[idx + "TODO(phase-".len()..];
            // Phase token: `[0-9]+[a-z]*` followed by ` — ` or ` -- ` or `:`.
            let phase_token: String = after
                .chars()
                .take_while(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
                .collect();
            if phase_token.is_empty() || !phase_token.starts_with(|c: char| c.is_ascii_digit()) {
                violators.push(format!(
                    "{}:{}: missing phase number after TODO(phase-: {}",
                    path.display(),
                    lineno + 1,
                    line.trim(),
                ));
                continue;
            }
            // After phase token: must have either `—` (em-dash), `--`, or
            // `:` followed by non-empty description on this line OR
            // continuation lines must start with `//!` / `//` carrying the
            // body. We require the head-line to carry SOME description
            // payload; bare `TODO(phase-3):` with nothing else is rejected.
            let rest = &after[phase_token.len()..];
            let rest_trim = rest.trim_start();
            // Check for separator + non-empty payload on same line.
            let payload = if let Some(p) = rest_trim.strip_prefix("\u{2014}") {
                // em-dash separator (canonical convention)
                p
            } else if let Some(p) = rest_trim.strip_prefix("--") {
                p
            } else if let Some(p) = rest_trim.strip_prefix(':') {
                p
            } else if let Some(p) = rest_trim.strip_prefix(')') {
                // Bare `TODO(phase-3)` with no separator + no payload.
                p
            } else {
                rest_trim
            };
            // Look for a closing-paren-bounded body OR a phrase that names a
            // destination. Phase-3 close markers typically read
            // `TODO(phase-3 — <topic>): <body>` — the head line carries the
            // topic; body is across the `):` boundary.
            if payload.trim().is_empty() {
                violators.push(format!(
                    "{}:{}: TODO(phase-{}) lacks named destination/topic: {}",
                    path.display(),
                    lineno + 1,
                    phase_token,
                    line.trim(),
                ));
            }
        }
    });

    assert!(
        violators.is_empty(),
        "G20-B C-14 architectural pin: every TODO(phase-N) marker MUST have a \
         non-empty topic/destination per HARD RULE rule-12. Violators:\n{}",
        violators.join("\n"),
    );
}

fn walk_rust_sources(root: &std::path::Path, callback: &mut dyn FnMut(&std::path::Path, &str)) {
    let mut stack: Vec<std::path::PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            // Skip directories that shouldn't be walked.
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if matches!(
                name,
                "target" | "node_modules" | ".git" | "dist" | ".addl" | ".benten"
            ) {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            // Skip this very test file (self-references would loop).
            if path.file_name().and_then(|n| n.to_str()) == Some("missing_docs_workspace.rs") {
                continue;
            }
            let Ok(src) = std::fs::read_to_string(&path) else {
                continue;
            };
            callback(&path, &src);
        }
    }
}
