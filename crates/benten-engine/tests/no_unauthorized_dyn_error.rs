//! R3-A RED-PHASE pin: no unauthorized `Box<dyn std::error::Error>` at
//! the engine boundary outside the SINGLE authorized site
//! (G13-B wave-2; D-B).
//!
//! Pin source: r2-test-landscape §2.1 G13-B row
//! `no_unauthorized_box_dyn_std_error_at_engine_boundary`; D-B.
//!
//! ## What this pins
//!
//! Per D-B resolution, the engine boundary uses
//! `Box<dyn std::error::Error + Send + Sync>` at EXACTLY ONE site:
//! the `EngineError::Backend` variant (or whatever the canonical
//! erasure point is named at G13-B landing time).
//!
//! Other public surface MUST stay typed:
//!
//! - Engine config errors → typed `EngineConfigError`.
//! - Module install errors → typed `ModuleInstallError`.
//! - Cap-policy errors → typed `CapError`.
//! - Subgraph errors → typed `SubgraphError`.
//!
//! A future PR that lazily wraps a typed error in `Box<dyn Error>`
//! at a NEW boundary (instead of adding it as a variant on the
//! existing typed error) would silently break the structured-error
//! contract that downstream callers (napi error-mapping, structured
//! tracing) rely on.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-B introduces the SINGLE authorized erasure site"]
fn no_unauthorized_box_dyn_std_error_at_engine_boundary() {
    // G13-B implementer wires this:
    //
    //   let src = std::fs::read_to_string("crates/benten-engine/src/error.rs").unwrap();
    //   let mut authorized = 0usize;
    //   let mut unauthorized: Vec<(usize, String)> = Vec::new();
    //   for (i, line) in src.lines().enumerate() {
    //       let trimmed = line.trim_start();
    //       if trimmed.starts_with("//") || trimmed.starts_with("///") { continue; }
    //       if trimmed.contains("Box<dyn std::error::Error") {
    //           // Allowed only inside the EngineError::Backend variant
    //           // declaration (single contiguous block).
    //           let allowed = (some_window_of_lines_around_i).contains("Backend");
    //           if allowed {
    //               authorized += 1;
    //           } else {
    //               unauthorized.push((i + 1, line.to_string()));
    //           }
    //       }
    //   }
    //   assert!(unauthorized.is_empty(),
    //       "unauthorized Box<dyn std::error::Error> sites: {:?}", unauthorized);
    //   assert!(authorized >= 1,
    //       "authorized erasure site (EngineError::Backend) must exist");
    //
    // OBSERVABLE consequence: a future PR adding
    // `Box<dyn Error>` to a non-Backend-variant location fails this
    // test loudly. The pin is a regression guard, not a capability —
    // legitimate new erasure sites can extend the allow-list with a
    // CITED rationale in the test.
    unimplemented!(
        "G13-B wires source-grep assertion limiting Box<dyn Error> to EngineError::Backend"
    );
}
