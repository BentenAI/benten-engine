//! R3-E RED-PHASE pin for G20-B missing_docs sweep + escape-hatch retire
//! (wave 8b; phase-2-backlog §8.3 + C-7).
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

#[test]
#[ignore = "RED-PHASE: G20-B wave-8b — all phase-2/3 TODO markers have named destinations (C-14)"]
fn all_phase_2_3_todo_markers_have_named_destinations() {
    // C-14 architectural pin. G20-B implementer wires this:
    //
    //   // Walk every src + test file; find TODO / FIXME / XXX markers
    //   // referencing phase-2 / phase-3; verify each has a named
    //   // destination per HARD RULE rule-12.
    //   let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..");
    //   let mut residuals = Vec::new();
    //   for entry in walkdir::WalkDir::new(workspace_root) {
    //       let entry = entry.unwrap();
    //       let p = entry.path();
    //       if p.extension().and_then(|e| e.to_str()) != Some("rs") { continue; }
    //       // Skip target/, node_modules/:
    //       if p.components().any(|c| matches!(c.as_os_str().to_str(),
    //                              Some("target") | Some("node_modules") | Some(".git"))) {
    //           continue;
    //       }
    //       let src = std::fs::read_to_string(p).unwrap();
    //       for (lineno, line) in src.lines().enumerate() {
    //           let trimmed = line.trim();
    //           if !(trimmed.contains("TODO") || trimmed.contains("FIXME")) { continue; }
    //           if trimmed.contains("phase-2") || trimmed.contains("Phase 2") ||
    //              trimmed.contains("phase-3") || trimmed.contains("Phase 3") {
    //               // Per HARD RULE rule-12: named destination required.
    //               // Acceptable: explicit phase-N reference + closure status
    //               // or destination-doc reference (phase-3-backlog §X.Y, etc.).
    //               // Unacceptable: bare "TODO(phase-2): rewrite later".
    //               if !looks_like_named_destination(trimmed) {
    //                   residuals.push(format!("{}:{}: {}",
    //                       p.display(), lineno + 1, trimmed));
    //               }
    //           }
    //       }
    //   }
    //   assert!(residuals.is_empty(),
    //       "G20-B must verify every phase-2/3 TODO marker has a named \
    //        destination per HARD RULE rule-12; residuals:\n{}",
    //       residuals.join("\n"));
    //
    // OBSERVABLE consequence: workspace-wide TODO marker discipline at
    // Phase-3 close.
    unimplemented!("G20-B wires phase-2/3 TODO marker named-destination pin");
}
