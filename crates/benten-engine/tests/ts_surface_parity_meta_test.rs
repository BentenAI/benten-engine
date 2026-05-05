//! R3-E RED-PHASE pin for G19-D LOAD-BEARING TS-surface-parity meta-test
//! (wave-7 parallel; §7.10 + D-PHASE-3-9 expanded).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-D +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-D must-pass column):
//!
//! - `tests/ts_surface_parity_meta_test_walks_napi_struct_surface_no_drift` — §7.10 LOAD-BEARING
//!
//! ## What G19-D establishes (§7.10 + D-PHASE-3-9)
//!
//! Rust-side schema-parity meta-test (~300-500 LOC) walks the napi struct
//! surface against the TS dist `.d.ts` declarations. **LOAD-BEARING**:
//! this is the structural fix that converges the long-tail 24-instance
//! producer/consumer drift recurrence per D-PHASE-3-9.
//!
//! Per stream-r1-5 (D-PHASE-3-9 EXPANDED at R1): the meta-test enumerates
//! 5 distinct p/c drift shape-modes from Phase-2b retrospective:
//! 1. translation-layer-phantom (pim-11)
//! 2. translation-layer-incorrect-mapping (e.g. `WaitArgs` duration drift)
//! 3. consumer-projection-silent-coerce (pim-13 candidate)
//! 4. casing-drift (the 24th p/c instance; §6.6 pattern)
//! 5. schema-parity-missing-field (the §7.9 Edge `cid` phantom + missing properties)
//!
//! ## RED-PHASE discipline
//!
//! Meta-test does NOT yet exist. R5 implementer wires it.
//! Per stream-r1-5: the meta-test is itself a runtime arm whose
//! closed-claim needs an end-to-end pin (§3.6b applied to the meta-test
//! itself — would FAIL if the meta-test were silently no-op'd).
//!
//! ## Scope clarification per D-PHASE-3-9 EXPANSION
//!
//! Walk surfaces:
//! - napi exported `#[napi]` structs (extracted via napi-rs schema)
//! - TS `.d.ts` interface declarations in `packages/engine/dist/`
//! - eval primitive `execute()` property reads (extracted via codegen
//!   or grep against `crates/benten-eval/src/primitives/*.rs`)

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — LOAD-BEARING TS-surface-parity meta-test against .d.ts"]
fn ts_surface_parity_meta_test_walks_napi_struct_surface_no_drift() {
    // §7.10 LOAD-BEARING pin per D-PHASE-3-9. G19-D implementer wires this:
    //
    //   // 1. Read all napi-exported struct shapes from the napi crate:
    //   let napi_schema = benten_engine::testing::extract_napi_struct_schema();
    //
    //   // 2. Read the corresponding TS interface declarations from the
    //   //    built .d.ts files:
    //   let dts_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("packages").join("engine")
    //       .join("dist").join("index.d.ts");
    //   let dts = std::fs::read_to_string(&dts_path).unwrap();
    //   let ts_schema = parse_dts_interfaces(&dts);
    //
    //   // 3. For each napi-exported struct, the corresponding TS interface
    //   //    must declare every field with matching type:
    //   for napi_struct in napi_schema {
    //       let ts_interface = ts_schema.find(|i| i.name == napi_struct.name)
    //           .expect(&format!("napi struct {} missing TS .d.ts \
    //               declaration — schema-parity-missing-field shape (§7.10 mode 5)",
    //               napi_struct.name));
    //       for napi_field in napi_struct.fields {
    //           let ts_field = ts_interface.find_field(&napi_field.name)
    //               .expect(&format!("napi struct {}.{} missing in TS \
    //                   interface — schema-parity drift", napi_struct.name, napi_field.name));
    //           assert_eq!(napi_field.ts_type(), ts_field.ts_type(),
    //               "type drift on {}.{}: napi={:?} ts={:?}",
    //               napi_struct.name, napi_field.name,
    //               napi_field.ts_type(), ts_field.ts_type());
    //       }
    //       // Also: NO orphan TS fields not in napi (the Edge.cid phantom shape):
    //       for ts_field in &ts_interface.fields {
    //           assert!(napi_struct.fields.iter().any(|f| f.name == ts_field.name),
    //               "TS interface {}.{} is phantom (not present in napi struct)",
    //               napi_struct.name, ts_field.name);
    //       }
    //   }
    //
    // OBSERVABLE consequence: the 24-instance recurrence converges at
    // structural layer. End-to-end pin per pim-2 §3.6b — would FAIL if
    // the meta-test were silently no-op'd. Defends against the failure
    // mode where the meta-test runs but never actually walks the schemas.
    unimplemented!("G19-D wires LOAD-BEARING TS-surface-parity meta-test against .d.ts");
}

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — meta-test detects synthetic injected drift fixture"]
fn ts_surface_parity_meta_test_rejects_synthetic_drift_fixture() {
    // stream-r1-5 pin per "the meta-test is itself a runtime arm whose
    // closed-claim needs an end-to-end pin." Inject a synthetic drift
    // and verify the meta-test REJECTS it (would FAIL if the meta-test
    // were silently no-op'd).
    //
    // G19-D implementer wires this:
    //
    //   // Build a synthetic napi schema fixture with a known drift:
    //   let mut synthetic_napi = real_napi_schema();
    //   synthetic_napi.find_struct("Edge").push_field(/* extra phantom field */ "cid_phantom");
    //
    //   // Pass the synthetic schema through the meta-test logic; it
    //   // MUST reject:
    //   let result = run_parity_check(synthetic_napi, real_dts_schema());
    //   assert!(result.is_err(),
    //       "meta-test must REJECT synthetic injected drift (pim-2 §3.6b \
    //        end-to-end pin: would FAIL if meta-test were silently no-op'd)");
    //
    //   // The reject must name the specific drift instance:
    //   let err_msg = format!("{:?}", result.err().unwrap());
    //   assert!(err_msg.contains("cid_phantom"),
    //       "meta-test reject message must name the specific drift");
    //
    // OBSERVABLE consequence: the meta-test is verifiably effective
    // against the documented drift shapes. Defends against the
    // sentinel-presence-only failure mode (meta-test exists but doesn't
    // actually catch drift).
    unimplemented!("G19-D wires synthetic-drift-fixture rejection meta-meta test");
}
