//! R3-E RED-PHASE pin for G19-D LOAD-BEARING DSL-args-vs-eval-primitive-properties
//! parity meta-test (wave-7 parallel; §7.10 + D-PHASE-3-9 + pim-12).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-D +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-D must-pass column +
//! R4-R1 expansion per `pcds-r4-r1-3` instance-27 PRE-EMPTION +
//! `stream-r4r1-2` 5-mode coverage):
//!
//! - `tests/dsl_args_vs_eval_properties_parity_meta_test_no_drift_across_all_primitives` — §7.10 LOAD-BEARING; pim-12
//! - `tests/parity_meta_test_consumer_projection_mermaid_no_drift` — D-PHASE-3-9; stream-r1-5
//! - `tests/parity_meta_test_consumer_projection_drift_detector_no_drift` — D-PHASE-3-9; stream-r1-5
//! - `tests/parity_meta_test_consumer_projection_change_event_translation_no_drift` — pcds-r4-r1-3 (4th projection)
//! - `tests/parity_meta_test_consumer_projection_dsl_helper_modules_no_drift` — pcds-r4-r1-3 (5th projection; pim-11 translation-layer)
//! - `tests/dsl_args_vs_eval_parity_meta_test_rejects_synthetic_drift_fixture` — stream-r1-5 mode-1 synthetic-drift
//! - `tests/dsl_args_vs_eval_parity_meta_test_rejects_synthetic_translation_layer_incorrect_mapping_fixture` — stream-r4r1-2 mode-2 (pcds-2 shape)
//! - `tests/dsl_args_vs_eval_parity_meta_test_rejects_synthetic_casing_drift_fixture` — stream-r4r1-2 mode-4 (24th-instance shape)
//! - `tests/parity_meta_test_mermaid_subscribe_arm_drift_detected_post_simulated_dsl_rename` — pcds-r4-r1-3 per-case-arm regression
//!
//! ## What G19-D establishes (§7.10 + D-PHASE-3-9 EXPANDED)
//!
//! Per stream-r1-5 + r1-napi-3: the meta-test walks each `*Args` interface
//! in `dsl.ts` against the eval primitive's `execute()` property reads.
//! For DSL-compiler-mediated primitives (BranchArgs / IterateArgs /
//! CallArgs etc.) the meta-test walks against the DSL-compiler output
//! (`crates/benten-dsl-compiler/src/lib.rs::compile_str`) rather than directly
//! against eval primitive reads.
//!
//! Per stream-r1-5 expansion (D-PHASE-3-9 RECOMMEND) + R4-R1 pcds-r4-r1-3
//! 5-projection enumeration: consumer projection sweep covers:
//!
//!   1. eval primitive `execute()` property reads (the canonical sink)
//!   2. mermaid producer projection (`packages/engine/src/mermaid.ts`)
//!   3. drift-detector projection
//!      (`crates/benten-ivm/tests/algorithm_b_drift_detector.rs` from G15-B)
//!   4. ChangeEvent translation bridge (`crates/benten-engine/src/builder.rs`
//!      OperationNode property bag → ChangeEvent translation for SUBSCRIBE
//!      delivery; pcds-r4-r1-3 expansion)
//!   5. DSL helper translation modules (`packages/engine/src/dsl.ts`
//!      `translateXxxArgs` helpers — translateWaitArgs / translateSandboxArgs
//!      per G17-C / future translateBranchArgs etc per G19-D 6-Args drift
//!      fixes; pre-empts pim-11 process-shape extension)
//!
//! ## 5 drift shape-mode coverage (stream-r4r1-2)
//!
//! The §7.10 LOAD-BEARING parity meta-test enumerates 5 drift shape-modes
//! from the Phase-2b 24-instance retrospective. Each mode MUST be covered
//! by a synthetic-drift-fixture rejection meta-meta test (pim-2 §3.6b
//! end-to-end: the meta-test itself is a runtime arm whose closed-claim
//! needs an end-to-end pin — would FAIL if meta-test silently no-op'd):
//!
//!   1. translation-layer-phantom (mode 1): WaitArgs orphan-field shape
//!      — covered by `dsl_args_vs_eval_parity_meta_test_rejects_synthetic_drift_fixture`
//!   2. translation-layer-incorrect-mapping (mode 2): WaitArgs.duration →
//!      duration_ms wrong-value shape (R6-R5 pcds-2 — took 5 deep-sweeps
//!      to find) — covered by
//!      `dsl_args_vs_eval_parity_meta_test_rejects_synthetic_translation_layer_incorrect_mapping_fixture`
//!      (NEW per stream-r4r1-2)
//!   3. consumer-projection-silent-coerce (mode 3): mermaid SUBSCRIBE arm
//!      file-level scan vs per-case-arm AST walk — covered by
//!      `parity_meta_test_mermaid_subscribe_arm_drift_detected_post_simulated_dsl_rename`
//!      (NEW per pcds-r4-r1-3)
//!   4. casing-drift (mode 4): wallclockMs ↔ wallclock_ms (24th-instance
//!      shape; per-instance fix at G17-C wave-5b; meta-test must include
//!      synthetic-casing-drift rejection at structural layer) — covered
//!      by `dsl_args_vs_eval_parity_meta_test_rejects_synthetic_casing_drift_fixture`
//!      (NEW per stream-r4r1-2)
//!   5. schema-parity-missing-field (mode 5): Rust producer widening +
//!      TS consumer interface not widened (Instance 18 sandboxDepth +
//!      Instance 25 candidate AttributionFrame Phase-3) — covered by
//!      `dsl_args_vs_eval_properties_parity_meta_test_no_drift_across_all_primitives`
//!      walking every napi `#[napi]` exported struct against TS .d.ts
//!
//! ## Per-case-arm AST walk discipline (pcds-r4-r1-3)
//!
//! The mermaid + DSL-helper-modules consumer projections require
//! PER-CASE-ARM AST walking (NOT file-level literal scanning) so the
//! Phase-2b 22nd-instance shape (mermaid SUBSCRIBE arm reading
//! `pick("event")` while `EmitArgs.event` exists elsewhere → spurious
//! global-match) is caught. The meta-test extractors named below
//! (`extract_per_case_arm_pick_refs`, `extract_dsl_helper_translation_keys`)
//! are the load-bearing implementation contract — a naive file-level
//! extractor would silently coerce per-case-arm drift to a global-match
//! false-pass.
//!
//! Per scope-real-09 + r2-test-landscape §3.D: G19-D mini-review reviewer
//! roster MUST include `producer-consumer-deep-sweep` lens — the structural
//! fix attempt closes the 24-instance recurrence; pre-merge sweep mandatory.
//!
//! ## RED-PHASE discipline
//!
//! Meta-test does NOT yet exist. R5 implementer wires it.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — LOAD-BEARING DSL-args-vs-eval-primitive-properties parity meta-test"]
fn dsl_args_vs_eval_properties_parity_meta_test_no_drift_across_all_primitives() {
    // §7.10 LOAD-BEARING pin. G19-D implementer wires this:
    //
    //   // 1. Extract every *Args interface from packages/engine/src/dsl.ts:
    //   let dsl_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("packages").join("engine")
    //       .join("src").join("dsl.ts");
    //   let dsl = std::fs::read_to_string(&dsl_path).unwrap();
    //   let dsl_args_interfaces = extract_args_interfaces(&dsl);
    //
    //   // 2. For each *Args interface, walk against the corresponding
    //   //    eval primitive's property reads. For DSL-compiler-mediated
    //   //    primitives (Branch, Iterate, Call), walk against the
    //   //    DSL compiler's output keyspace; for simple-translation
    //   //    primitives (Wait, Sandbox, Subscribe), walk against eval
    //   //    primitive reads directly.
    //   for args in dsl_args_interfaces {
    //       let primitive_keyspace = if args.is_compiler_mediated() {
    //           extract_dsl_compiler_output_keys(&args.primitive_name)
    //       } else {
    //           extract_eval_primitive_property_reads(&args.primitive_name)
    //       };
    //
    //       // Each DSL field reaches an eval/compiler keyspace key:
    //       for dsl_field in &args.fields {
    //           let translated_key = args.translate_to_keyspace(&dsl_field.name);
    //           assert!(primitive_keyspace.contains(&translated_key),
    //               "*Args drift: {}::{} (DSL) → {} (translated) not in \
    //                primitive keyspace {:?} — closes the 24-instance recurrence \
    //                at structural layer",
    //               args.primitive_name, dsl_field.name,
    //               translated_key, primitive_keyspace);
    //       }
    //
    //       // Each eval/compiler keyspace key has a DSL-args producer:
    //       for primitive_key in &primitive_keyspace {
    //           let dsl_origin = args.fields.iter()
    //               .find(|f| args.translate_to_keyspace(&f.name) == *primitive_key);
    //           assert!(dsl_origin.is_some(),
    //               "primitive key {}::{} has no DSL-args producer — \
    //                consumer-projection-silent-coerce shape",
    //               args.primitive_name, primitive_key);
    //       }
    //   }
    //
    // OBSERVABLE consequence: the 24-instance long-tail recurrence
    // converges at the structural layer. Defends against future drift
    // by mechanically checking parity at every PR.
    unimplemented!("G19-D wires LOAD-BEARING DSL-args-vs-eval-properties parity meta-test");
}

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — parity meta-test consumer projection: mermaid no drift (D-PHASE-3-9 expanded; pcds-r4-r1-3 per-case-arm walk)"]
fn parity_meta_test_consumer_projection_mermaid_no_drift() {
    // stream-r1-5 / D-PHASE-3-9 EXPANSION pin (sharpened by R4-R1
    // pcds-r4-r1-3 to require PER-CASE-ARM AST walking, NOT file-level
    // literal scanning). G19-D implementer wires this:
    //
    //   // Walk packages/engine/src/mermaid.ts (the producer projection
    //   // that emits a mermaid diagram for a Subgraph):
    //   let mermaid_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("packages").join("engine")
    //       .join("src").join("mermaid.ts");
    //   let mermaid = std::fs::read_to_string(&mermaid_path).unwrap();
    //
    //   // PCDS-R4-R1-3 LOAD-BEARING: per-case-arm AST walk associating
    //   // each `pick(...)` call with its enclosing `case '<primitive>':`
    //   // statement. A naive file-level literal scan would treat the
    //   // file as one global namespace + falsely match `pick("event")`
    //   // in the SUBSCRIBE arm against `EmitArgs.event` elsewhere
    //   // (Phase-2b 22nd-instance shape — caught post-merge).
    //   let mermaid_emitter_refs: std::collections::HashMap<String, Vec<String>> =
    //       extract_per_case_arm_pick_refs(&mermaid);
    //   for (primitive, fields) in &mermaid_emitter_refs {
    //       // Each (primitive, field) pair must reach the primitive's
    //       // OWN canonical Args interface — NOT a sibling primitive's
    //       // Args even if the field name happens to coincide globally:
    //       let dsl_args = canonical_dsl_args_for(primitive);
    //       for field in fields {
    //           assert!(dsl_args.contains_field(field),
    //               "mermaid emitter case '{}' references field `{}` \
    //                but {}Args has no such field — per-case-arm \
    //                consumer-projection drift (mode-3 / pcds-r4-r1-3 \
    //                per-case-arm shape)",
    //               primitive, field, primitive);
    //       }
    //   }
    //
    // OBSERVABLE consequence: mermaid producer projection stays in sync
    // with DSL Args interfaces, AT THE PER-CASE-ARM granularity. Defends
    // against the failure mode where mermaid silently drops or mis-maps
    // a field rendering when it's added to DSL (the shape that R6-FP
    // r6-r5-pcds-1 named for SUBSCRIBE rendering). Per pcds-r4-r1-3:
    // file-level scanning is INSUFFICIENT — naive scanning falsely
    // matches `pick("event")` in case 'subscribe' arm against
    // EmitArgs.event leaving the SUBSCRIBE arm drift unflagged.
    unimplemented!("G19-D wires parity meta-test PER-CASE-ARM AST walk against mermaid.ts");
}

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — parity meta-test consumer projection: drift-detector no drift (D-PHASE-3-9 expanded)"]
fn parity_meta_test_consumer_projection_drift_detector_no_drift() {
    // stream-r1-5 / D-PHASE-3-9 EXPANSION pin (meta-meta — meta-test
    // walks the drift-detector projection from G15-B).
    //
    //   // G15-B's algorithm_b_drift_detector.rs proptest constructs
    //   // synthetic events by walking the OperationNode property bag.
    //   // The synthetic-event keyspace MUST stay in sync with the eval
    //   // primitive's actual property reads.
    //   let drift_detector_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("crates").join("benten-ivm")
    //       .join("tests").join("algorithm_b_drift_detector.rs");
    //   let detector_src = std::fs::read_to_string(&drift_detector_path).unwrap();
    //   let detector_synthesized_keys = extract_synthesized_event_keys(&detector_src);
    //
    //   for (primitive, synthesized_keys) in &detector_synthesized_keys {
    //       let canonical_keys = canonical_eval_primitive_keys(primitive);
    //       for key in synthesized_keys {
    //           assert!(canonical_keys.contains(key),
    //               "drift-detector synthesizes orphan key {}::{} not \
    //                read by eval — projection drift", primitive, key);
    //       }
    //   }
    //
    // OBSERVABLE consequence: G15-B's drift-detector stays in sync with
    // the eval primitive keyspace. Defends against the meta-meta failure
    // mode where the drift-detector itself drifts from production reads.
    unimplemented!(
        "G19-D wires parity meta-test consumer projection sweep against algorithm_b_drift_detector"
    );
}

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — parity meta-test rejects synthetic injected drift (stream-r1-5 end-to-end pin)"]
fn dsl_args_vs_eval_parity_meta_test_rejects_synthetic_drift_fixture() {
    // stream-r1-5 pin per "the meta-test is itself a runtime arm whose
    // closed-claim needs an end-to-end pin (§3.6b)." Synthetic-drift
    // fixture verifies the meta-test ACTUALLY catches drift (would FAIL
    // if meta-test were silently no-op'd).
    //
    //   // Inject a synthetic Args drift fixture:
    //   let mut synthetic_dsl_args = real_dsl_args_interfaces();
    //   synthetic_dsl_args.find_mut("WaitArgs").push_field("orphan_field_phantom");
    //
    //   // Run the meta-test logic against the fixture:
    //   let result = run_dsl_args_parity_check(synthetic_dsl_args, real_eval_keyspace());
    //
    //   assert!(result.is_err(),
    //       "meta-test must REJECT synthetic injected drift (§3.6b \
    //        end-to-end: would FAIL if meta-test were silently no-op'd)");
    //   let err_msg = format!("{:?}", result.err().unwrap());
    //   assert!(err_msg.contains("orphan_field_phantom"),
    //       "meta-test reject message must name the specific drift");
    //
    // OBSERVABLE consequence: the meta-test is verifiably effective
    // against the 5 enumerated drift shape-modes (per stream-r1-5
    // RECOMMEND).
    unimplemented!("G19-D wires synthetic-drift-fixture rejection (meta-meta closure pin)");
}

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — parity meta-test rejects synthetic mode-2 (translation-layer-incorrect-mapping; pcds-2 shape) (stream-r4r1-2)"]
fn dsl_args_vs_eval_parity_meta_test_rejects_synthetic_translation_layer_incorrect_mapping_fixture()
{
    // stream-r4r1-2 mode-2 closure pin per pim-2 §3.6b end-to-end
    // requirement. G19-D implementer wires this:
    //
    //   // Inject a synthetic translation-layer-incorrect-mapping fixture:
    //   // the TS DSL field `duration: '5m'` is supposed to translate to
    //   // `duration_ms: 300000` (5 minutes in ms). Inject a translator
    //   // that produces the WRONG value (`duration_ms: 5000` — 5 seconds)
    //   // — mirror of R6-R5 pcds-2 shape that took 5 deep-sweeps to find.
    //   let mut synthetic_translator_table = real_translator_table();
    //   synthetic_translator_table.set_translator("WaitArgs", "duration",
    //       Box::new(|s: &str| {
    //           // Wrong: returns ms-as-seconds rather than parsing minutes
    //           let n: u64 = s.trim_end_matches("m").parse().unwrap_or(0);
    //           serde_json::json!({"duration_ms": n * 1000}) // wrong: ×1000 not ×60000
    //       }));
    //
    //   // Run the meta-test logic against the synthetic translator:
    //   let result = run_translation_layer_parity_check(synthetic_translator_table);
    //
    //   assert!(result.is_err(),
    //       "meta-test must REJECT synthetic translation-layer-incorrect-mapping \
    //        fixture (mode-2; pim-2 §3.6b end-to-end: would FAIL if meta-test \
    //        silently no-op'd against value-level mapping drift)");
    //   let err_msg = format!("{:?}", result.err().unwrap());
    //   assert!(err_msg.contains("WaitArgs") && err_msg.contains("duration"),
    //       "meta-test reject message must name the specific drift coordinate");
    //
    // OBSERVABLE consequence: mode-2 (translation-layer-incorrect-mapping —
    // the pcds-2 shape) is verifiably caught at structural layer. Defends
    // against the failure shape where translation produces a syntactically-
    // correct but semantically-wrong mapping (the highest-recurrence-likelihood
    // shape per Phase-2b retrospective).
    unimplemented!(
        "G19-D wires synthetic translation-layer-incorrect-mapping (mode-2) rejection per stream-r4r1-2"
    );
}

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — parity meta-test rejects synthetic mode-4 (casing-drift; 24th-instance shape) (stream-r4r1-2)"]
fn dsl_args_vs_eval_parity_meta_test_rejects_synthetic_casing_drift_fixture() {
    // stream-r4r1-2 mode-4 closure pin per pim-2 §3.6b end-to-end
    // requirement. G19-D implementer wires this:
    //
    //   // Inject a synthetic casing-drift fixture: the TS DSL declares
    //   // `wallclockMs: 30000` (camelCase); the eval-side reads
    //   // `wallclock_ms` (snake_case). Per the §6.6 24th p/c drift
    //   // acceptance criterion (G17-C wave-5b translateSandboxArgs),
    //   // the translator MUST normalize across casing styles. Inject
    //   // a translator that drops the conversion:
    //   let mut synthetic_translator_table = real_translator_table();
    //   synthetic_translator_table.set_translator("SandboxArgs", "wallclockMs",
    //       Box::new(|v| serde_json::json!({"wallclockMs": v}))); // wrong: should be `wallclock_ms`
    //
    //   // Run the meta-test logic against the synthetic translator:
    //   let result = run_casing_translation_parity_check(synthetic_translator_table);
    //
    //   assert!(result.is_err(),
    //       "meta-test must REJECT synthetic casing-drift fixture (mode-4; \
    //        24th-instance shape — pim-2 §3.6b end-to-end: would FAIL if \
    //        meta-test silently no-op'd against per-field casing drift)");
    //   let err_msg = format!("{:?}", result.err().unwrap());
    //   assert!(err_msg.contains("wallclock") &&
    //           (err_msg.contains("Ms") || err_msg.contains("_ms")),
    //       "meta-test reject message must name the casing-drift coordinate");
    //
    // OBSERVABLE consequence: mode-4 (casing-drift — the 24th-instance
    // shape that took the §6.6 acceptance criterion to pin) is verifiably
    // caught at structural layer rather than only via per-instance fixes.
    // Defends against the failure shape where camelCase TS DSL fields
    // silently translate to wrong-cased eval-side properties.
    unimplemented!("G19-D wires synthetic casing-drift (mode-4) rejection per stream-r4r1-2");
}

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — parity meta-test consumer projection: ChangeEvent translation no drift (pcds-r4-r1-3 4th projection)"]
fn parity_meta_test_consumer_projection_change_event_translation_no_drift() {
    // pcds-r4-r1-3 4th-consumer-projection pin. G19-D implementer wires this:
    //
    //   // Walk crates/benten-engine/src/builder.rs (the producer-projection
    //   // that translates an OperationNode property bag → ChangeEvent
    //   // payload for SUBSCRIBE delivery; surfaced as a consumer projection
    //   // by Phase-2b R6-R3):
    //   let builder_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("benten-engine").join("src").join("builder.rs");
    //   let builder_src = std::fs::read_to_string(&builder_path).unwrap();
    //
    //   // For each primitive the ChangeEvent translation handles, every
    //   // OperationNode property the bridge reads must exist in the
    //   // canonical primitive keyspace (no orphan reads, no silent skips):
    //   let translation_refs = extract_change_event_translation_property_reads(&builder_src);
    //   for (primitive, properties) in &translation_refs {
    //       let canonical_keys = canonical_eval_primitive_keys(primitive);
    //       for property in properties {
    //           assert!(canonical_keys.contains(property),
    //               "ChangeEvent translation for {} reads orphan property `{}` \
    //                — consumer-projection drift (4th projection per pcds-r4-r1-3)",
    //               primitive, property);
    //       }
    //   }
    //
    // OBSERVABLE consequence: the ChangeEvent translation bridge stays
    // in sync with the canonical primitive keyspace. Defends against the
    // failure mode where SUBSCRIBE delivery silently drops a property
    // because the translation bridge skipped it (Phase-2b R6-R3 surfaced
    // this as a producer-consumer drift surface).
    unimplemented!(
        "G19-D wires parity meta-test 4th consumer projection — ChangeEvent translation bridge"
    );
}

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — parity meta-test consumer projection: DSL helper modules no drift (pcds-r4-r1-3 5th projection)"]
fn parity_meta_test_consumer_projection_dsl_helper_modules_no_drift() {
    // pcds-r4-r1-3 5th-consumer-projection pin (pre-empts pim-11
    // translation-layer process-shape extension). G19-D implementer
    // wires this:
    //
    //   // Walk packages/engine/src/dsl.ts for translateXxxArgs helpers
    //   // (translateWaitArgs, translateSandboxArgs per G17-C, future
    //   // translateBranchArgs etc per G19-D 6-Args drift fixes):
    //   let dsl_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("packages").join("engine")
    //       .join("src").join("dsl.ts");
    //   let dsl = std::fs::read_to_string(&dsl_path).unwrap();
    //
    //   // For each translateXxxArgs helper, the per-helper output keyspace
    //   // MUST land in the corresponding eval primitive's canonical keyspace
    //   // (every translation produces a key the eval side reads + every
    //   // canonical key has a translator producer):
    //   let helper_translations = extract_dsl_helper_translation_keys(&dsl);
    //   for (primitive, output_keys) in &helper_translations {
    //       let canonical_keys = canonical_eval_primitive_keys(primitive);
    //       for output_key in output_keys {
    //           assert!(canonical_keys.contains(output_key),
    //               "translate{}Args produces orphan output key `{}` — \
    //                consumer-projection drift (5th projection / pim-11 \
    //                translation-layer mode per pcds-r4-r1-3)",
    //               primitive, output_key);
    //       }
    //   }
    //
    // OBSERVABLE consequence: the DSL helper translation modules stay
    // in sync with the canonical eval primitive keyspace. Defends against
    // the pim-11 translation-layer-phantom failure shape where a helper
    // produces a key the eval side never reads (silent value-loss in
    // the runtime payload).
    unimplemented!(
        "G19-D wires parity meta-test 5th consumer projection — DSL helper translation modules"
    );
}

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — parity meta-test catches mermaid SUBSCRIBE arm drift via simulated DSL rename (pcds-r4-r1-3 per-case-arm regression pin)"]
fn parity_meta_test_mermaid_subscribe_arm_drift_detected_post_simulated_dsl_rename() {
    // pcds-r4-r1-3 per-case-arm regression pin. Mirrors the Phase-2b
    // 22nd-instance shape: pre-fix mermaid.ts contained
    // `case 'emit': pick('channel')` AND `case 'subscribe': pick('event')`.
    // A literal-scanning extractor sees `event` referenced in the file
    // + `EmitArgs.event` exists in DSL → false PASS. The meta-test must
    // walk per-case-arm to catch the per-primitive drift.
    //
    // G19-D implementer wires this:
    //
    //   // Simulate a DSL rename that should fire the meta-test:
    //   // Rename SubscribeArgs.pattern → SubscribeArgs.foo (synthetic).
    //   // The mermaid case 'subscribe' arm still reads `pick("pattern")`.
    //   // A naive file-level extractor would see `pattern` referenced
    //   // somewhere else in the file (e.g. WAIT_PATTERN constant) +
    //   // falsely PASS. The per-case-arm walk MUST reject:
    //   let synthetic_renamed_args = synthetic_dsl_args_with_subscribe_pattern_renamed_to_foo();
    //   let result = run_mermaid_per_case_arm_parity_check(
    //       synthetic_renamed_args, real_mermaid_src());
    //
    //   assert!(result.is_err(),
    //       "meta-test must REJECT the simulated DSL rename — per-case-arm \
    //        walk catches the SUBSCRIBE arm drift even when `pattern` \
    //        appears elsewhere in mermaid.ts");
    //   let err_msg = format!("{:?}", result.err().unwrap());
    //   assert!(err_msg.contains("subscribe") && err_msg.contains("pattern"),
    //       "reject message must name the SUBSCRIBE arm + drifted field");
    //
    // OBSERVABLE consequence: the mermaid SUBSCRIBE arm regression
    // (Phase-2b 22nd instance) is structurally pinned. Defends against
    // file-level extractors silently coercing per-case-arm drift to a
    // global match.
    unimplemented!(
        "G19-D wires per-case-arm regression test for mermaid SUBSCRIBE arm drift per pcds-r4-r1-3"
    );
}

#[test]
#[ignore = "RED-PHASE: G19-D wave-7 — host:atrium:publish_view_result capability cite-discipline (D-PHASE-3-21 trust-policy via UCAN)"]
fn parity_meta_test_consumer_projection_host_atrium_publish_view_result_capability_cite_discipline()
{
    // D-PHASE-3-21 (trust-policy via UCAN) consumer-projection axis.
    // The `host:atrium:publish_view_result` capability is introduced by
    // the trust-policy resolution and MUST be cited consistently across
    // the producer/consumer surfaces:
    //
    //   1. The Rust capability constant declared at
    //      `crates/benten-caps/src/host_capabilities.rs` (or successor)
    //   2. CODE_TO_CTOR / ERROR-CATALOG enumeration at
    //      `crates/benten-engine/src/error_codes.rs` if a capability-denial
    //      typed error is produced for it
    //   3. TS-side `packages/engine/src/errors.ts` typed-class table
    //   4. docs/ERROR-CATALOG.md textual enumeration
    //   5. docs/SECURITY-POSTURE.md / SANDBOX-LIMITS.md docs (if applicable)
    //
    // G19-D implementer wires this:
    //
    //   const CAP: &str = "host:atrium:publish_view_result";
    //
    //   // (a) Rust capability constant present:
    //   let caps_rs = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("benten-caps").join("src").join("host_capabilities.rs"),
    //   ).unwrap();
    //   assert!(caps_rs.contains(CAP),
    //       "host:atrium:publish_view_result capability MUST be declared as Rust constant");
    //
    //   // (b) ERROR-CATALOG.md enumeration:
    //   let catalog = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join("docs").join("ERROR-CATALOG.md"),
    //   ).unwrap();
    //   assert!(catalog.contains(CAP),
    //       "ERROR-CATALOG.md MUST cite host:atrium:publish_view_result capability");
    //
    //   // (c) TS-side errors.ts cites the capability (typed-error subclass
    //   //     name OR documentation comment ref):
    //   let ts_errors = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join("packages").join("engine")
    //           .join("src").join("errors.ts"),
    //   ).unwrap();
    //   assert!(ts_errors.contains(CAP),
    //       "TS errors.ts MUST cite host:atrium:publish_view_result capability \
    //        (D-PHASE-3-21 trust-policy via UCAN cite-discipline)");
    //
    // OBSERVABLE consequence: the new capability is consistently cited
    // across Rust + docs + TS surfaces. Defends against the failure shape
    // where a capability is declared in code but undocumented in TS / docs
    // (silent-undefined + DX regression — the same shape as Edge.cid
    // phantom but for capability strings).
    unimplemented!(
        "G19-D wires host:atrium:publish_view_result cite-discipline meta-test per D-PHASE-3-21"
    );
}
