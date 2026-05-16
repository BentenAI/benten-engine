//! Phase-3 G19-D §7.10 — LOAD-BEARING DSL-args-vs-eval-primitive-properties
//! parity meta-test (wave-7; D-PHASE-3-9 EXPANDED + pim-12).
//!
//! The structural fix that converges the long-tail 24-instance
//! producer/consumer drift recurrence at the structural layer. Walks
//! every `*Args` interface in `packages/engine/src/dsl.ts` against the
//! corresponding eval primitive's `op.properties.get("...")` reads
//! (`crates/benten-eval/src/primitives/<p>.rs` + the SANDBOX reader at
//! `crates/benten-engine/src/primitive_host.rs::execute_sandbox`).
//!
//! ## What this meta-test catches at structural layer
//!
//! For every primitive whose DSL surface differs from the eval-side
//! property-bag keyspace, a `translateXxxArgs` helper bridges the two.
//! This test verifies, mechanically, that:
//!
//!   1. **Every translator output key is read by the eval-side primitive.**
//!      A translator-output orphan (DSL helper writes a key no eval reader
//!      consults) is the silent-value-loss shape that the 23rd p/c drift
//!      (`r6-r5-pcds-2` WaitArgs.duration) and the 24th p/c drift
//!      (G17-C wallclockMs ↔ wallclock_ms) both exhibited.
//!
//!   2. **Every translator helper exists for primitives whose DSL surface
//!      keys differ from eval-side keys.** A missing translator means the
//!      DSL spread copies user-facing field names verbatim into the
//!      property bag while the eval-side reader looks up DIFFERENT keys
//!      (the canonical-key orphan shape — Phase-2b 6 pre-existing DSL
//!      Args drifts that G19-D §7.9 fixes).
//!
//! ## RED-PHASE → GREEN-PHASE
//!
//! Pre-G19-D: 6 pre-existing Args drifts (BranchArgs / ReadArgs /
//! IterateArgs / TransformArgs / RespondArgs / CallArgs) spread DSL
//! field names verbatim into the property bag while the eval-side
//! primitives read DIFFERENT keys (silent value-loss). The 6 surface
//! fixes land in the same G19-D wave; this test pins the structural
//! defense against recurrence.
//!
//! ## Pin sources
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
//! ## Implementation strategy
//!
//! Source-of-truth = `packages/engine/src/dsl.ts` (TS DSL surface +
//! translator helpers) + `crates/benten-eval/src/primitives/<p>.rs`
//! (eval-side `op.properties.get("<key>")` reads) +
//! `crates/benten-engine/src/primitive_host.rs::execute_sandbox` for the
//! SANDBOX reader.
//!
//! The eval-side keyspace per primitive is captured below as a literal
//! table — the test walks each primitive in turn, extracts the
//! translator's output keyspace from dsl.ts, and asserts:
//!
//!   - Every key the translator emits is in the eval-side keyspace
//!     (no translator-output orphan).
//!
//! Note: the inverse direction (every eval-read key has a translator
//! producer) is NOT enforced as a hard fail because some eval reads
//! are populated by the engine COMPILE PATH (e.g. BRANCH `cases`,
//! BRANCH `has_default`, BRANCH `conditions`, ITERATE `requires`, CALL
//! `parent_scope` / `requires` / `timeout_ms` / `elapsed_ms`,
//! TRANSFORM `input`) NOT by the DSL surface. The test annotates which
//! eval-read keys are compile-path-supplied vs DSL-supplied below.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn read_dsl_ts() -> String {
    let p = workspace_root()
        .join("packages")
        .join("engine")
        .join("src")
        .join("dsl.ts");
    std::fs::read_to_string(&p).unwrap_or_else(|e| {
        panic!(
            "packages/engine/src/dsl.ts not found at {} ({})",
            p.display(),
            e
        )
    })
}

fn read_mermaid_ts() -> String {
    let p = workspace_root()
        .join("packages")
        .join("engine")
        .join("src")
        .join("mermaid.ts");
    std::fs::read_to_string(&p).unwrap_or_else(|e| {
        panic!(
            "packages/engine/src/mermaid.ts not found at {} ({})",
            p.display(),
            e
        )
    })
}

// ---------------------------------------------------------------------------
// Canonical eval-side keyspace per primitive
// ---------------------------------------------------------------------------
//
// Source-of-truth: every `op.properties.get("<key>")` read in
// `crates/benten-eval/src/primitives/<p>.rs` (and
// `crates/benten-engine/src/primitive_host.rs::execute_sandbox` for
// SANDBOX). Updating this table requires an accompanying eval-side
// change OR a DSL translator update — drift in one direction without
// the other fires the meta-test.

/// Returns the full set of `op.properties.get("<key>")` reads for
/// `<primitive>`. Includes BOTH DSL-supplied keys AND compile-path-
/// supplied keys; the test annotates which is which when narrowing the
/// translator-output check.
fn canonical_eval_keyspace(primitive: &str) -> BTreeSet<&'static str> {
    let keys: &[&'static str] = match primitive {
        // crates/benten-eval/src/primitives/read.rs
        "read" => &["query_kind", "target_cid", "label"],
        // crates/benten-eval/src/primitives/branch.rs
        // - condition_value: compile-path (boolean BRANCH)
        // - cases / has_default / conditions: compile-path (edge-table-driven)
        "branch" => &[
            "condition_value",
            "match_value",
            "cases",
            "has_default",
            "conditions",
        ],
        // crates/benten-eval/src/primitives/iterate.rs
        // - requires: compile-path (capability decl)
        // - max: reads via `op.properties.get("max")` at line 64-66 (Inv-9
        //   iteration budget — DSL `IterateArgs.max` translates verbatim).
        "iterate" => &["items", "max", "requires"],
        // crates/benten-eval/src/primitives/transform.rs
        // - input: compile-path (upstream binding)
        "transform" => &["expr", "input", "result"],
        // crates/benten-eval/src/primitives/call.rs
        // - parent_scope / requires / timeout_ms / elapsed_ms: compile-path
        "call" => &[
            "child_scope",
            "requires",
            "parent_scope",
            "timeout_ms",
            "elapsed_ms",
            "target",
            "call_op",
            "input",
        ],
        // crates/benten-eval/src/primitives/respond.rs
        "respond" => &["status", "body"],
        // crates/benten-eval/src/primitives/emit.rs
        "emit" => &["channel", "payload", "handler"],
        // crates/benten-eval/src/primitives/wait.rs
        // (reads via `wait_node.property("<key>")` which is the same
        // OperationNode property bag).
        "wait" => &["signal", "duration_ms", "timeout_ms", "signal_shape"],
        // crates/benten-eval/src/primitives/subscribe.rs
        "subscribe" => &["pattern", "handler"],
        // crates/benten-engine/src/primitive_host.rs::execute_sandbox
        "sandbox" => &[
            "module",
            "manifest",
            "caps",
            "fuel",
            "wallclock_ms",
            "output_limit",
            "input",
        ],
        // WRITE: spreads verbatim, eval-side reads `label` + `properties` +
        // `requires` (compile-path via WriteSpec extraction at
        // bindings/napi/src/subgraph.rs::extract_write_args).
        "write" => &["label", "properties", "requires"],
        // STREAM: dispatched via the engine compile path through
        // `StreamPrimitiveSpec` at `crates/benten-engine/src/engine_stream.rs::build_stream_handle`,
        // not via direct `op.properties.get()`. The DSL surface field
        // `source` lands as a SubgraphNode arg the COMPILE PATH consumes
        // when constructing the spec; `chunkSize` is the DSL-side hint
        // forwarded to the spec's chunking. Mermaid renders `source` for
        // at-a-glance display.
        "stream" => &["source", "chunkSize"],
        other => panic!("canonical_eval_keyspace: unknown primitive `{other}`"),
    };
    keys.iter().copied().collect()
}

/// Eval-side keys that are populated by the ENGINE COMPILE PATH (not
/// the DSL surface). The DSL translator MUST NOT emit these (a phantom
/// emission would conflict with the compile-path producer). The test
/// uses this table to filter out compile-path keys when checking the
/// orphan-eval-reader direction.
fn compile_path_supplied_keys(primitive: &str) -> BTreeSet<&'static str> {
    let keys: &[&'static str] = match primitive {
        "branch" => &["cases", "has_default", "conditions", "condition_value"],
        "iterate" => &["requires"],
        "transform" => &["input"],
        "call" => &["parent_scope", "requires", "timeout_ms", "elapsed_ms"],
        "write" => &["properties", "requires"],
        _ => &[],
    };
    keys.iter().copied().collect()
}

// ---------------------------------------------------------------------------
// dsl.ts translator extractor
// ---------------------------------------------------------------------------

/// Extract the output-keyspace of a `translateXxxArgs` helper from
/// dsl.ts. The keyspace is the set of property-keys the helper assigns
/// onto its `props: Record<string, JsonValue>` accumulator before
/// returning.
///
/// Recognized assignment shapes (matched at line scope):
///
///   - `props.<key> = ...`
///   - `props["<key>"] = ...`
///   - `props.<key> = <conditional> ? ... : ...` (the ternary is
///     irrelevant for keyspace extraction).
///
/// String-literal assignments (`props.query_kind = "by_cid"`) count
/// the LHS key only — the assigned value is irrelevant for the
/// keyspace check.
pub(crate) fn extract_translator_output_keys(dsl: &str, fn_name: &str) -> BTreeSet<String> {
    // Find the function body. Tolerates `function translateXxxArgs(\n  args: ...,\n): ... {`.
    let needle = format!("function {fn_name}(");
    let Some(start) = dsl.find(&needle) else {
        return BTreeSet::new();
    };
    // Walk forward to the first `{` after the signature.
    let after_sig = &dsl[start..];
    let Some(brace_offset) = after_sig.find('{') else {
        return BTreeSet::new();
    };
    let body_start = start + brace_offset + 1;
    // Walk forward, brace-depth-aware, to the matching `}`.
    let mut depth = 1i32;
    let mut end = body_start;
    for (i, ch) in dsl[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = body_start + i;
                    break;
                }
            }
            _ => {}
        }
    }
    let body = &dsl[body_start..end];

    // Extract `props.<key> = ...` and `props["<key>"] = ...` patterns.
    let mut keys = BTreeSet::new();
    for line in body.lines() {
        let trimmed = line.trim();
        // Skip comments to avoid false matches inside JSDoc / inline.
        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }
        // `props.<key>` form.
        if let Some(rest) = trimmed.strip_prefix("props.") {
            // Identifier up to first `=` / `[` / whitespace.
            let mut key = String::new();
            for c in rest.chars() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    key.push(c);
                } else {
                    break;
                }
            }
            if !key.is_empty() {
                // Filter: must be followed by `=` to count as an assignment
                // (excludes `props.foo` reads — defensive).
                let after_key = rest.trim_start_matches(&key);
                if after_key.trim_start().starts_with('=') {
                    keys.insert(key);
                }
            }
            continue;
        }
        // `props["<key>"]` form.
        if let Some(rest) = trimmed.strip_prefix("props[\"")
            && let Some(end) = rest.find('"')
        {
            keys.insert(rest[..end].to_string());
        }
    }
    keys
}

/// `*Args` interface → translator function name + primitive identifier.
/// The DSL builders route specific Args interfaces through specific
/// translators; this table captures the routing so the meta-test can
/// walk every interface with no naming guesswork.
fn args_interface_translator_table() -> Vec<(&'static str, &'static str, &'static str)> {
    // (Args-interface-name, translator-fn-name, primitive-id)
    vec![
        ("ReadArgs", "translateReadArgs", "read"),
        ("BranchArgs", "translateBranchArgs", "branch"),
        ("IterateArgs", "translateIterateArgs", "iterate"),
        ("TransformArgs", "translateTransformArgs", "transform"),
        ("CallArgs", "translateCallArgs", "call"),
        ("RespondArgs", "translateRespondArgs", "respond"),
        ("WaitArgs", "translateWaitArgs", "wait"),
        ("SubscribeArgs", "translateSubscribeArgs", "subscribe"),
        ("SandboxArgs", "translateSandboxArgs", "sandbox"),
        // EmitArgs has no helper — dsl.ts builders write { channel, payload }
        // inline. Treated as a special-case in the LOAD-BEARING walk.
        // WriteArgs spreads verbatim — also inline.
    ]
}

// ---------------------------------------------------------------------------
// LOAD-BEARING test: every translator output key is in canonical eval keyspace
// ---------------------------------------------------------------------------

#[test]
fn dsl_args_vs_eval_properties_parity_meta_test_no_drift_across_all_primitives() {
    let dsl = read_dsl_ts();
    let mut violations: Vec<String> = Vec::new();
    let mut translators_walked = 0usize;

    for (args_name, fn_name, primitive) in args_interface_translator_table() {
        let translator_keys = extract_translator_output_keys(&dsl, fn_name);
        if translator_keys.is_empty() {
            violations.push(format!(
                "translator `{fn_name}` (for {args_name} → {primitive}) — \
                 extractor returned EMPTY output keyspace. Either the \
                 translator was renamed/removed (silent value-loss \
                 recurrence) or the extractor regressed."
            ));
            continue;
        }
        translators_walked += 1;
        let canonical = canonical_eval_keyspace(primitive);
        for key in &translator_keys {
            if !canonical.contains(key.as_str()) {
                violations.push(format!(
                    "{args_name} translator-output orphan: \
                     `{fn_name}` emits `{key}` but eval-side {primitive} \
                     primitive does not read it (canonical keyspace: {canonical:?}). \
                     Closes the 24-instance recurrence at structural layer — \
                     either drop the translator emission or wire the \
                     eval-side read."
                ));
            }
        }
    }

    // Special-case EMIT: dsl.ts builders write { channel: args.event,
    // payload: args.payload } inline (no helper). Verify those keys
    // are in the canonical EMIT keyspace.
    let emit_inline_keys: BTreeSet<&str> = ["channel", "payload"].iter().copied().collect();
    let emit_canonical = canonical_eval_keyspace("emit");
    for key in &emit_inline_keys {
        assert!(
            emit_canonical.contains(*key),
            "EMIT inline-builder writes `{key}` but eval-side emit does \
             not read it (canonical: {emit_canonical:?})"
        );
    }

    // Verify we walked every translator the table declares.
    let expected = args_interface_translator_table().len();
    assert!(
        translators_walked >= expected.saturating_sub(1),
        "walked {translators_walked} translators but expected ~{expected} — \
         table-vs-extractor mismatch (silent skip recurrence)"
    );

    assert!(
        violations.is_empty(),
        "DSL-args-vs-eval-properties parity drift detected ({} violations):\n{}",
        violations.len(),
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Synthetic-drift-fixture rejection (pim-2 §3.6b end-to-end pin)
// ---------------------------------------------------------------------------
//
// Defends against the failure mode where the meta-test silently no-ops
// (the extractor returns an empty set so every assertion vacuously
// passes). Each synthetic fixture INJECTS a known-bad shape and asserts
// the parity-check logic REJECTS it.

#[test]
fn dsl_args_vs_eval_parity_meta_test_rejects_synthetic_drift_fixture() {
    // Mode-1 synthetic-drift: a translator emits a key that the eval
    // primitive does not read. The parity check MUST detect.
    let synthetic_dsl = r#"
function translateReadArgs(args) {
  const props = {};
  props.label = args.label;
  props.query_kind = "by_cid";
  // Drift: emit a phantom key the eval-side read primitive never reads.
  props.orphan_field_phantom = "drift";
  return props;
}
"#;
    let extracted = extract_translator_output_keys(synthetic_dsl, "translateReadArgs");
    assert!(
        extracted.contains("orphan_field_phantom"),
        "extractor SILENT NO-OP — `orphan_field_phantom` not surfaced \
         from synthetic translator (extractor regression that would \
         make every parity check vacuously pass)"
    );

    let canonical = canonical_eval_keyspace("read");
    let drift_caught = !canonical.contains("orphan_field_phantom");
    assert!(
        drift_caught,
        "parity check SILENT NO-OP — `orphan_field_phantom` not \
         detected as out-of-canonical"
    );

    // Verify the LOAD-BEARING test would have caught this if it had
    // been present in the real dsl.ts: walk the violation-formation
    // path explicitly.
    let mut would_violate = false;
    for key in &extracted {
        if !canonical.contains(key.as_str()) {
            would_violate = true;
            break;
        }
    }
    assert!(
        would_violate,
        "LOAD-BEARING parity check would NOT have caught the synthetic \
         drift — meta-test is silently no-op"
    );
}

#[test]
fn dsl_args_vs_eval_parity_meta_test_rejects_synthetic_translation_layer_incorrect_mapping_fixture()
{
    // Mode-2 (translation-layer-incorrect-mapping; pcds-2 shape):
    // synthesize a translator that writes `duration` (the raw user-
    // facing key) into the property bag instead of `duration_ms`. The
    // eval-side WAIT primitive reads `duration_ms` — so a `duration`
    // emission goes to a key the eval doesn't read (silent drop;
    // R6-R5 pcds-2 shape).
    let synthetic_dsl = r"
function translateWaitArgs(args) {
  const props = {};
  // Drift: writes raw `duration` instead of `duration_ms` (mode-2:
  // value-correct shape but key-incorrect — the highest-recurrence
  // shape per Phase-2b retrospective).
  props.duration = args.duration;
  return props;
}
";
    let extracted = extract_translator_output_keys(synthetic_dsl, "translateWaitArgs");
    let canonical = canonical_eval_keyspace("wait");

    // The drift: extracted contains `duration` (which is NOT in canonical
    // — canonical has `duration_ms`). The check must catch this.
    assert!(
        extracted.contains("duration"),
        "extractor SILENT NO-OP — `duration` not surfaced from \
         synthetic translator (mode-2 shape)"
    );
    assert!(
        !canonical.contains("duration"),
        "canonical WAIT keyspace MUST NOT contain raw `duration` — the \
         eval-side reader at primitives/wait.rs reads `duration_ms`. \
         If this assertion fails, the mode-2 fixture has been \
         invalidated (canonical keyspace was widened to accept \
         `duration`)."
    );

    let mut violation = None;
    for key in &extracted {
        if !canonical.contains(key.as_str()) {
            violation = Some(key.clone());
            break;
        }
    }
    assert!(
        violation.is_some(),
        "mode-2 (translation-layer-incorrect-mapping) shape NOT caught \
         — meta-test silently no-op against value-level mapping drift"
    );
    assert_eq!(
        violation.unwrap(),
        "duration",
        "violation must name the specific drift coordinate"
    );
}

#[test]
fn dsl_args_vs_eval_parity_meta_test_rejects_synthetic_casing_drift_fixture() {
    // Mode-4 (casing-drift; 24th-instance shape): synthesize a
    // translator that writes `wallclockMs` (camelCase, DSL-style)
    // instead of `wallclock_ms` (snake_case, eval-style). The eval-side
    // SANDBOX reader at primitive_host.rs::execute_sandbox reads
    // `wallclock_ms` — a `wallclockMs` emission silently drops.
    let synthetic_dsl = r"
function translateSandboxArgs(args) {
  const props = {};
  props.module = args.module;
  // Drift: writes camelCase `wallclockMs` instead of snake_case
  // `wallclock_ms` (mode-4: casing-drift — 24th p/c instance shape;
  // §6.6 acceptance criterion).
  props.wallclockMs = args.wallclockMs;
  return props;
}
";
    let extracted = extract_translator_output_keys(synthetic_dsl, "translateSandboxArgs");
    let canonical = canonical_eval_keyspace("sandbox");

    assert!(
        extracted.contains("wallclockMs"),
        "extractor SILENT NO-OP — `wallclockMs` not surfaced from \
         synthetic translator (mode-4 shape)"
    );
    assert!(
        canonical.contains("wallclock_ms"),
        "canonical SANDBOX keyspace MUST contain `wallclock_ms` (the \
         snake_case form the eval-side reader uses)"
    );
    assert!(
        !canonical.contains("wallclockMs"),
        "canonical SANDBOX keyspace MUST NOT contain `wallclockMs` — \
         the eval-side reader at primitive_host.rs::execute_sandbox \
         reads snake_case only. If this fails, the mode-4 fixture has \
         been invalidated."
    );

    let mut violation = None;
    for key in &extracted {
        if !canonical.contains(key.as_str()) {
            violation = Some(key.clone());
            break;
        }
    }
    assert!(
        violation.is_some(),
        "mode-4 (casing-drift) shape NOT caught — meta-test silently \
         no-op against per-field casing drift"
    );
    let v = violation.unwrap();
    assert!(
        v.contains("wallclock") && (v.contains("Ms") || v.contains("MS")),
        "violation must name the casing-drift coordinate; got `{v}`"
    );
}

// ---------------------------------------------------------------------------
// Consumer projection: mermaid producer (D-PHASE-3-9 expanded; pcds-r4-r1-3)
// ---------------------------------------------------------------------------

/// Strip TS block (`/* ... */`) + line (`// ...`) comments from a source
/// string. Used by the per-case-arm extractor so a comment body that
/// quotes `pick("...")` doesn't fire as a phantom reference.
fn strip_block_and_line_comments(src: &str) -> String {
    let mut cleaned = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'/') => {
                for c2 in chars.by_ref() {
                    if c2 == '\n' {
                        cleaned.push('\n');
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                while let Some(c2) = chars.next() {
                    if c2 == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
                    }
                }
            }
            _ => cleaned.push(c),
        }
    }
    cleaned
}

/// Extract per-case-arm `pick("<field>")` references from mermaid.ts.
/// Returns a map of `<primitive>` → `Vec<<field>>` so the parity check
/// can walk per-case-arm rather than file-level (the failure shape
/// pcds-r4-r1-3 named).
pub(crate) fn extract_per_case_arm_pick_refs(
    mermaid: &str,
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut out: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut current_case: Option<String> = None;
    let mut depth_in_case: i32 = 0;

    // Strip TS comments to avoid false matches like `// pick("on")` inside
    // a comment body explaining the post-cascade rename.
    let cleaned = strip_block_and_line_comments(mermaid);

    for raw_line in cleaned.lines() {
        let line = raw_line.trim();
        // Detect `case "<name>":` arm start.
        if let Some(rest) = line.strip_prefix("case ") {
            let rest = rest.trim();
            let name_with_quotes = rest.split(':').next().unwrap_or("").trim();
            let name = name_with_quotes.trim_matches('"').to_string();
            if !name.is_empty() {
                current_case = Some(name);
                depth_in_case = 0;
                continue;
            }
        }
        // Track brace depth within case (so a nested `{ ... }` doesn't
        // accidentally close the case scope).
        for c in line.chars() {
            match c {
                '{' => depth_in_case += 1,
                '}' => depth_in_case -= 1,
                _ => {}
            }
        }
        // `return ...` or next `case` ends the current case scope.
        if (line.starts_with("return ") || line.starts_with("default:"))
            && depth_in_case <= 0
            && current_case.is_some()
            && !line.starts_with("return")
        {
            // `default:` resets to None.
            current_case = None;
        }
        // Collect every `pick("<field>")` reference within the current case.
        if let Some(case) = &current_case {
            let mut search_from = 0;
            while let Some(pos) = line[search_from..].find("pick(\"") {
                let abs = search_from + pos + "pick(\"".len();
                if let Some(end) = line[abs..].find('"') {
                    let field = &line[abs..abs + end];
                    out.entry(case.clone()).or_default().push(field.to_string());
                    search_from = abs + end + 1;
                } else {
                    break;
                }
            }
        }
    }
    out
}

#[test]
fn parity_meta_test_consumer_projection_mermaid_no_drift() {
    // Walk mermaid.ts per-case-arm and assert each `pick("<field>")`
    // call references a key the corresponding primitive ACTUALLY writes
    // into the OperationNode property bag — i.e. is in the eval-side
    // canonical keyspace (since those are the keys the DSL builder
    // emits post-translation).
    let mermaid = read_mermaid_ts();
    let arm_refs = extract_per_case_arm_pick_refs(&mermaid);

    assert!(
        !arm_refs.is_empty(),
        "extractor SILENT NO-OP — per-case-arm pick refs returned empty \
         from mermaid.ts (extractor regression)"
    );

    let mut violations: Vec<String> = Vec::new();
    for (primitive, fields) in &arm_refs {
        // Skip the `default` arm — it's a fallthrough, not a per-primitive arm.
        if primitive == "default" {
            continue;
        }
        let canonical = canonical_eval_keyspace(primitive);
        for field in fields {
            if !canonical.contains(field.as_str()) {
                violations.push(format!(
                    "mermaid case `{primitive}` reads `pick(\"{field}\")` \
                     but {primitive} canonical eval keyspace ({canonical:?}) \
                     has no such key — per-case-arm consumer-projection \
                     drift (mode-3 / pcds-r4-r1-3 per-case-arm shape)."
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "mermaid consumer-projection drift ({} violations):\n{}",
        violations.len(),
        violations.join("\n")
    );
}

#[test]
fn parity_meta_test_mermaid_subscribe_arm_drift_detected_post_simulated_dsl_rename() {
    // pcds-r4-r1-3 per-case-arm regression pin: synthetic mermaid
    // blob renames the SUBSCRIBE arm's `pick("pattern")` to
    // `pick("event")` (the Phase-2b 22nd-instance shape). A naive
    // file-level extractor would see `event` referenced elsewhere
    // (in the `case "emit":` arm) and falsely PASS. The per-case-arm
    // walk MUST reject — `event` is NOT in the SUBSCRIBE primitive's
    // canonical keyspace.
    let synthetic_mermaid = r#"
function shortArgs(n) {
  const a = n.args;
  const pick = (k) => a[k] === undefined ? "" : String(a[k]);
  switch (n.primitive) {
    case "emit":
      return pick("channel");
    case "subscribe":
      // Drift: reads `event` (mirrors pcds-r4-r1-3 per-case-arm shape).
      // EmitArgs.event exists elsewhere → naive file-scan falsely passes.
      return pick("event");
    default:
      return "";
  }
}
"#;
    let arm_refs = extract_per_case_arm_pick_refs(synthetic_mermaid);
    let subscribe_fields = arm_refs
        .get("subscribe")
        .expect("synthetic mermaid has a `subscribe` case arm — extractor regression if missing");
    assert!(
        subscribe_fields.iter().any(|f| f == "event"),
        "extractor did not surface `event` from the SUBSCRIBE case arm \
         — extractor regression (silent no-op against per-case-arm drift)"
    );

    // Per-case-arm parity check: `event` is NOT in subscribe canonical
    // keyspace (it's in EMIT). The check MUST reject.
    let canonical = canonical_eval_keyspace("subscribe");
    let drift_caught = subscribe_fields
        .iter()
        .any(|f| !canonical.contains(f.as_str()));
    assert!(
        drift_caught,
        "per-case-arm parity check SILENT NO-OP — SUBSCRIBE arm drift \
         not caught despite `event` being out-of-canonical for \
         subscribe (canonical: {canonical:?})"
    );

    // Defense against the file-level-coercion failure shape: verify
    // that `event` IS legitimately in the EMIT canonical keyspace as a
    // PRODUCER-translated key (channel) — NOT as a raw field. This
    // confirms the extractor's per-case-arm scoping is NEEDED.
    assert!(
        canonical_eval_keyspace("emit").contains("channel"),
        "EMIT canonical keyspace must contain `channel` (the post-
         translation key — the DSL `event` field translates to this)"
    );
}

// ---------------------------------------------------------------------------
// Consumer projection: drift-detector synthesized event keys (D-PHASE-3-9)
// ---------------------------------------------------------------------------
//
// G15-B's drift-detector at `crates/benten-ivm/tests/algorithm_b_drift_detector.rs`
// (via `crates/benten-ivm/tests/common.rs`) synthesizes Node fixtures
// via `Write::to_node` which writes `createdAt` + `disambiguator` into
// the Node property bag. These are NODE-property keys (not OperationNode
// primitive-args keys); the drift-detector projection walks ChangeEvents
// derived from those Nodes, NOT primitive-args reads. The consumer-
// projection axis here is: the drift-detector synthesizes event-shape
// fields that the IVM views (the IVM consumer) read.
//
// The narrow guarantee this test pins: the drift-detector's
// synthesized-Node keyspace stays in sync with the canonical IVM-side
// reader keyspace — i.e. it does not drift to write Node properties
// that `ContentListingView` / `Algorithm B` projection logic does not
// consult.

#[test]
fn parity_meta_test_consumer_projection_drift_detector_no_drift() {
    let common_path = workspace_root()
        .join("crates")
        .join("benten-ivm")
        .join("tests")
        .join("common.rs");
    let common = std::fs::read_to_string(&common_path).unwrap_or_else(|e| {
        panic!(
            "drift-detector helper at {} not found ({})",
            common_path.display(),
            e
        )
    });

    // Walk the `to_node` body in common.rs and extract every `props.insert("<key>", ...)`
    // call's <key>. These are the keys the drift-detector's synthesized
    // Nodes carry into the ChangeEvent stream.
    let synthesized_keys = extract_props_insert_keys(&common, "to_node");

    assert!(
        !synthesized_keys.is_empty(),
        "drift-detector helper synthesized-key extractor returned empty \
         — extractor regression OR `to_node` was renamed in common.rs"
    );

    // The canonical ChangeEvent-bound Node keyspace per Phase-1 graph
    // schema: anchor Nodes carry stable user-facing properties +
    // engine-stamped `createdAt`. The drift-detector reuses this
    // surface; legitimate keys: `createdAt`, `disambiguator` (the
    // CID-widening key the helper uses to avoid Node-CID collision),
    // plus any future widening that lands via an explicit common.rs
    // edit. New keys land here; new keys NOT IN the table fire.
    let canonical_drift_detector_keys: BTreeSet<&str> =
        ["createdAt", "disambiguator"].iter().copied().collect();

    let mut violations: Vec<String> = Vec::new();
    for key in &synthesized_keys {
        if !canonical_drift_detector_keys.contains(key.as_str()) {
            violations.push(format!(
                "drift-detector `to_node` synthesizes Node property \
                 `{key}` not in canonical drift-detector keyspace \
                 ({canonical_drift_detector_keys:?}). If this is an \
                 intentional widening, update the canonical table in \
                 this test alongside the helper edit."
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "drift-detector consumer-projection drift ({} violations):\n{}",
        violations.len(),
        violations.join("\n")
    );
}

/// Helper: extract `props.insert("<key>", ...)` keys from inside a
/// named function body in a Rust source file.
fn extract_props_insert_keys(rust_src: &str, fn_name: &str) -> BTreeSet<String> {
    let needle = format!("fn {fn_name}");
    let Some(start) = rust_src.find(&needle) else {
        return BTreeSet::new();
    };
    // Walk forward to the first `{`.
    let after = &rust_src[start..];
    let Some(brace_off) = after.find('{') else {
        return BTreeSet::new();
    };
    let body_start = start + brace_off + 1;
    let mut depth = 1i32;
    let mut end = body_start;
    for (i, ch) in rust_src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = body_start + i;
                    break;
                }
            }
            _ => {}
        }
    }
    let body = &rust_src[body_start..end];

    // Match `props.insert(\n  "<key>", ...)` and `props.insert("<key>", ...)`
    // patterns — multi-line tolerant.
    let mut keys = BTreeSet::new();
    let pattern = "props.insert(";
    let mut search_from = 0usize;
    while let Some(pos) = body[search_from..].find(pattern) {
        let abs = search_from + pos + pattern.len();
        // Walk to the first `"` after the open paren (skip whitespace).
        let after_open = &body[abs..];
        let mut quote_start = None;
        for (i, c) in after_open.char_indices() {
            if c.is_whitespace() {
                continue;
            }
            if c == '"' {
                quote_start = Some(i);
                break;
            }
            break;
        }
        if let Some(qs) = quote_start {
            let key_start = abs + qs + 1;
            if let Some(qe) = body[key_start..].find('"') {
                keys.insert(body[key_start..key_start + qe].to_string());
            }
        }
        search_from = abs;
    }
    keys
}

// ---------------------------------------------------------------------------
// Consumer projection: ChangeEvent translation (pcds-r4-r1-3 4th projection)
// ---------------------------------------------------------------------------
//
// `crates/benten-engine/src/builder.rs` translates `graph::ChangeEvent`
// → `eval::primitives::subscribe::ChangeEvent` for SUBSCRIBE delivery.
// This is a fixed-shape STRUCT-to-STRUCT translation — pcds-r4-r1-3 named
// it as a "consumer projection" because its forwarding fidelity matters
// for SUBSCRIBE delivery correctness.
//
// The narrow guarantee this test pins: the translation forwards every
// non-edge field of `graph::ChangeEvent` cleanly into the eval-side
// `ChangeEvent` (no field-collapse, no field-drop). Codified after
// Phase-2b Round-2 Instance 6 BLOCKER (the `labels: Vec<String>` →
// `primary_label: String` collapse that caused multi-label SUBSCRIBE
// delivery loss).

// Required forwarded fields per Round-2 Instance 6 BLOCKER closure:
// anchor_cid + kind + seq + payload_bytes + labels + tx_id + actor_cid +
// handler_cid + capability_grant_cid (9; edge_endpoints stays out per the
// in-tree comment, anchor-centric eval-side struct).
const CHANGE_EVENT_REQUIRED_FIELDS: [&str; 9] = [
    "anchor_cid",
    "kind",
    "seq",
    "payload_bytes",
    "labels",
    "tx_id",
    "actor_cid",
    "handler_cid",
    "capability_grant_cid",
];

/// Extract the brace/paren/bracket-balanced body that follows `needle`
/// inside `src` (the char immediately after `needle` is depth 1).
fn balanced_body_after<'a>(src: &'a str, needle: &str, ctx: &str) -> &'a str {
    let start = src
        .find(needle)
        .unwrap_or_else(|| panic!("{ctx}: `{needle}` not found"));
    let body_start = start + needle.len();
    let mut depth = 1i32;
    let mut end = body_start;
    for (i, ch) in src[body_start..].char_indices() {
        match ch {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => {
                depth -= 1;
                if depth == 0 {
                    end = body_start + i;
                    break;
                }
            }
            _ => {}
        }
    }
    &src[body_start..end]
}

/// (1) builder.rs side: the bridge forwards all 9 source expressions
/// positionally into `ChangeEvent::for_bridge`, in signature order, with
/// the label + attribution args being real `event.*` sources (not a
/// hardcoded empty/None — the exact lossy shape of the original BLOCKER).
fn verify_bridge_call_forwards_all_9() {
    let builder_path = workspace_root()
        .join("crates")
        .join("benten-engine")
        .join("src")
        .join("builder.rs");
    let builder = std::fs::read_to_string(&builder_path)
        .unwrap_or_else(|e| panic!("builder.rs not found at {} ({})", builder_path.display(), e));
    let call_args = balanced_body_after(
        &builder,
        "subscribe::ChangeEvent::for_bridge(",
        "builder.rs ChangeEvent::for_bridge call (SUBSCRIBE wiring removed/refactored?)",
    );
    let forwarded: Vec<String> = call_args
        .split(',')
        .map(|a| {
            a.lines()
                .map(str::trim)
                .filter(|l| !l.starts_with("//") && !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string()
        })
        .filter(|a| !a.is_empty())
        .collect();
    assert_eq!(
        forwarded.len(),
        9,
        "ChangeEvent::for_bridge call in builder.rs forwards {} args, \
         expected 9. A dropped arg is a recurrence of Round-2 Instance 6 \
         BLOCKER (multi-label SUBSCRIBE delivery loss). Args seen: {forwarded:?}",
        forwarded.len()
    );
    for (idx, expected_substr) in [
        (4usize, "event.labels"),
        (5, "event.tx_id"),
        (6, "event.actor_cid"),
        (7, "event.handler_cid"),
        (8, "event.capability_grant_cid"),
    ] {
        assert!(
            forwarded[idx].contains(expected_substr),
            "ChangeEvent::for_bridge arg {idx} = {:?} does not forward \
             `{expected_substr}` — SUBSCRIBE delivery would silently \
             lose it (Round-2 Instance 6 BLOCKER shape).",
            forwarded[idx]
        );
    }
}

/// (2) benten-core side: `ChangeEvent::for_bridge` assigns all 9 fields
/// in its `Self { .. }` body (no field silently dropped at the ctor).
fn verify_for_bridge_ctor_assigns_all_9() {
    let change_stream_path = workspace_root()
        .join("crates")
        .join("benten-core")
        .join("src")
        .join("change_stream.rs");
    let change_stream = std::fs::read_to_string(&change_stream_path).unwrap_or_else(|e| {
        panic!(
            "change_stream.rs not found at {} ({})",
            change_stream_path.display(),
            e
        )
    });
    let ctor_start = change_stream
        .find("pub fn for_bridge(")
        .expect("ChangeEvent::for_bridge constructor not found in benten-core change_stream.rs");
    let self_body = balanced_body_after(
        &change_stream[ctor_start..],
        "Self {",
        "for_bridge has no `Self { .. }` body",
    );
    let mut fields = BTreeSet::new();
    for tok in self_body.split(',') {
        if let Some(name) = parse_struct_field_init(tok.trim()) {
            fields.insert(name);
        }
    }
    let required: BTreeSet<&str> = CHANGE_EVENT_REQUIRED_FIELDS.iter().copied().collect();
    let missing: Vec<&&str> = required.iter().filter(|r| !fields.contains(**r)).collect();
    assert!(
        missing.is_empty(),
        "ChangeEvent::for_bridge drops required fields {missing:?} — \
         SUBSCRIBE delivery would silently lose them. Recurrence of \
         Round-2 Instance 6 BLOCKER (multi-label SUBSCRIBE delivery loss) \
         shape. Constructor-assigned fields seen: {fields:?}"
    );
}

#[test]
fn parity_meta_test_consumer_projection_change_event_translation_no_drift() {
    // `benten_core::ChangeEvent` is `#[non_exhaustive]` (ST-CORE lane),
    // so the cross-crate struct-expression form the bridge used is no
    // longer legal; builder.rs now calls the full-fidelity `for_bridge`
    // constructor. The no-field-drop guarantee (Round-2 Instance 6
    // BLOCKER closure) therefore splits across two sites — verify both.
    verify_bridge_call_forwards_all_9();
    verify_for_bridge_ctor_assigns_all_9();
}

fn parse_struct_field_init(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // Strip leading line/block comments. Use rfind so we get the last
    // non-comment line (the actual field-init line in struct literals).
    let s = s.lines().rfind(|l| {
        let t = l.trim();
        !t.starts_with("//") && !t.is_empty()
    })?;
    // `<name>: <expr>` → <name>; or shorthand `<name>` → <name>.
    let head = s.split(':').next()?.trim();
    if head.is_empty() {
        return None;
    }
    // Take last token (skip `pub` etc).
    let tok = head.split_whitespace().next_back()?;
    let is_ident = !tok.is_empty()
        && tok
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && tok.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if is_ident {
        Some(tok.to_string())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Consumer projection: DSL helper modules (pcds-r4-r1-3 5th projection;
//                                          pim-11 translation-layer pre-empt)
// ---------------------------------------------------------------------------
//
// Walks the dsl.ts `translateXxxArgs` helpers as a SECOND pass — the
// LOAD-BEARING test above already covers the canonical-keyspace check;
// this test pins a sharper guarantee: every helper is referenced from
// at least one `addNode` site in the SubgraphBuilder + CaseBuilder
// classes. A helper that exists but is never called is itself a
// translation-layer-phantom (pim-11 shape — the helper produces
// nothing because the call site spreads raw args).

#[test]
fn parity_meta_test_consumer_projection_dsl_helper_modules_no_drift() {
    let dsl = read_dsl_ts();
    let mut violations: Vec<String> = Vec::new();

    for (args_name, fn_name, _primitive) in args_interface_translator_table() {
        // Verify the helper is invoked from at least one builder site.
        let invocation = format!("{fn_name}(");
        let count = dsl.matches(&invocation).count();
        // Every translator is defined ONCE (the `function <name>(` site)
        // + invoked at TWO sites (SubgraphBuilder + CaseBuilder) → 3
        // total occurrences of `<name>(`. If only the definition appears,
        // the helper is phantom (pim-11 shape).
        if count < 2 {
            violations.push(format!(
                "translator `{fn_name}` (for {args_name}) is defined but \
                 invoked only {count} time(s) — pim-11 translation-layer-\
                 phantom shape. The DSL builder spreads raw args bypass \
                 the translator → silent value-loss recurrence."
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "DSL helper consumer-projection drift ({} violations):\n{}",
        violations.len(),
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------------
// host:atrium:publish_view_result capability cite-discipline
// ---------------------------------------------------------------------------
//
// D-PHASE-3-21 trust-policy via UCAN: the new capability is introduced
// by the trust-policy resolution. This pin verifies cite-discipline
// across the cross-cutting surfaces: Rust capability constants, TS
// errors.ts, and (when present) docs/ENGINE-SPEC + SECURITY-POSTURE.
//
// Phase-3-aware: the test is tolerant of the capability not yet being
// declared — D-PHASE-3-21's wiring lands in a later wave. The pin
// asserts that IF the capability is declared anywhere, it's declared
// consistently across all cite sites (no half-landed widening).

#[test]
fn parity_meta_test_consumer_projection_host_atrium_publish_view_result_capability_cite_discipline()
{
    const CAP: &str = "host:atrium:publish_view_result";

    // Walk a small cross-cutting set of files that would carry the
    // capability cite once D-PHASE-3-21 wiring lands.
    let candidate_files = [
        ("docs/ERROR-CATALOG.md", false),
        ("docs/SECURITY-POSTURE.md", false),
        ("packages/engine/src/errors.ts", false),
    ];

    let root = workspace_root();
    let mut hits: Vec<&str> = Vec::new();
    let mut missing_when_others_present: Vec<&str> = Vec::new();

    let any_hit = candidate_files
        .iter()
        .map(|(rel, _)| root.join(rel))
        .any(|p| std::fs::read_to_string(&p).is_ok_and(|s| s.contains(CAP)));

    if !any_hit {
        // D-PHASE-3-21 wiring not yet landed — the test is informational
        // only at this stage. Once any cite site declares the capability,
        // the cross-cutting consistency check fires.
        return;
    }

    for (rel, _) in candidate_files {
        let p = root.join(rel);
        let body = std::fs::read_to_string(&p).unwrap_or_default();
        if body.contains(CAP) {
            hits.push(rel);
        } else if any_hit {
            missing_when_others_present.push(rel);
        }
    }

    // If any site declares the cap but another high-visibility site
    // doesn't, that's a half-landed widening → fire.
    assert!(
        missing_when_others_present.is_empty() || hits.is_empty(),
        "host:atrium:publish_view_result capability cite-discipline \
         drift: declared at {hits:?} but missing from {missing_when_others_present:?}. \
         D-PHASE-3-21 trust-policy via UCAN must cite the capability \
         consistently across cross-cutting surfaces."
    );
}

// ---------------------------------------------------------------------------
// Defensive smoke pins for the extractors
// ---------------------------------------------------------------------------

#[test]
fn translator_output_extractor_handles_real_dsl_translators() {
    let dsl = read_dsl_ts();
    // Every translator listed in the table MUST have a non-empty output
    // keyspace — confirms the extractor handles real-world dsl.ts
    // formatting (multi-line bodies, conditional blocks, comments).
    for (args_name, fn_name, _primitive) in args_interface_translator_table() {
        let keys = extract_translator_output_keys(&dsl, fn_name);
        assert!(
            !keys.is_empty(),
            "translator `{fn_name}` (for {args_name}) — extractor \
             returned EMPTY keyspace from real dsl.ts (extractor \
             regression OR translator was renamed)"
        );
    }
}

#[test]
fn per_case_arm_extractor_distinguishes_emit_subscribe_arms_in_real_mermaid() {
    let mermaid = read_mermaid_ts();
    let arm_refs = extract_per_case_arm_pick_refs(&mermaid);

    // emit arm reads `channel` (post-pcds-1 fix); subscribe arm reads
    // `pattern` (post-pcds-2 fix). The per-case-arm extractor MUST
    // distinguish these.
    let emit = arm_refs
        .get("emit")
        .expect("real mermaid.ts has `emit` arm");
    assert!(
        emit.iter().any(|f| f == "channel"),
        "EMIT arm pick refs do not include `channel` (post-pcds-1 \
         fix invalidated): {emit:?}"
    );

    let subscribe = arm_refs
        .get("subscribe")
        .expect("real mermaid.ts has `subscribe` arm");
    assert!(
        subscribe.iter().any(|f| f == "pattern"),
        "SUBSCRIBE arm pick refs do not include `pattern` (post-pcds-2 \
         fix invalidated): {subscribe:?}"
    );
}
