//! Phase-4-Meta-Core — TF-10 RED-PHASE (R3-B6) — benten-dsl-compiler
//! `phase-4-meta-deferred` cluster security-correctness + #1020
//! inverse-pretty-printer pins.
//!
//! ============================================================================
//! RED-PHASE STATUS — un-ignore at **G-CORE-DSL**.
//! ============================================================================
//!
//! Every `#[test]` here is `#[ignore = "RED-PHASE: un-ignore at
//! G-CORE-DSL"]`. They FAIL (or, for #496, **abort the process via
//! stack-overflow** — which is itself the RED signal) against the
//! synced baseline `ed03729a` and PASS once G-CORE-DSL lands the
//! bounded-recursion guard (#496), the SANDBOX-fuel upper-bound (#551),
//! and the engine-side `Subgraph → DSL` inverse pretty-printer (#1020).
//!
//! §3.6e: the closing wave (G-CORE-DSL) MUST sweep these `#[ignore]`
//! pins and un-ignore them; the reviewer verifies LANDING-STATUS (the
//! `#[ignore]` is removed and the test passes), not merely that this
//! spec-pin file exists.
//!
//! ============================================================================
//! GROUND-TRUTH (synced HEAD ed03729a — verified by the R3 author)
//! ============================================================================
//!
//!   crates/benten-dsl-compiler/src/lib.rs
//!     :525  fn parse_object  — calls parse_value per entry
//!     :555  fn parse_value   — recurses into parse_object on `{`
//!           => parse_object ⇄ parse_value are MUTUALLY RECURSIVE with
//!              NO depth bound. Deeply-nested `{ a: { a: { ... } } }`
//!              input drives unbounded native-stack recursion →
//!              process stack-overflow (safe-1 #496).
//!     :817  fn validate_shapes — SANDBOX_INT_PROPS =
//!           ["fuel","wallclock_ms","output_limit"]; rejects ONLY
//!           negative / non-int values. NO UPPER BOUND is enforced on
//!           `fuel` → an arbitrarily large fuel budget compiles
//!           cleanly → effectively-unbounded wasmtime budget (safe-2
//!           #551).
//!     (grep) NO `to_dsl` / `pretty_print` / `Subgraph → DSL` inverse
//!           exists anywhere in the crate (#1020 — no inverse
//!           pretty-printer; the engine-side inverse lands in
//!           G-CORE-DSL; the admin-UI consumer wiring is the
//!           Composing G-COMP-1 task and is OUT of Core R3 scope).
//!
//! Public entrypoint: `benten_dsl_compiler::compile_str(&str) ->
//! Result<CompiledSubgraph, CompileError>` (lib.rs:190). The #1020
//! inverse is expected to land as a public `benten_dsl_compiler`
//! function (the exact name is a G-CORE-DSL implementer decision —
//! candidates: `subgraph_to_dsl` / `CompiledSubgraph::to_dsl`); this
//! test is written against the round-trip PROPERTY so it does not
//! over-constrain the as-yet-unbuilt API name (see the #1020 test's
//! inline note + the `compile_error!`-free conditional shape).
//!
//! ============================================================================
//! SHAPE-not-SUBSTANCE (pim-18 / r2 §4-A): these are PRODUCTION-ARM
//! pins against the real `compile_str` path (not a sentinel "a parser
//! exists" check). #496 drives the real recursive descent; #551 drives
//! the real `validate_shapes` pass; #1020 round-trips through the real
//! compile + (future) inverse. WOULD-FAIL if the bound / upper-limit /
//! inverse is a no-op.
//! ============================================================================
//!
//! ============================================================================
//! HARD-RULE-12 clause-(b) BELONGS-NAMED-NOW dispositions — 13 OTHER DSL
//! cluster issues (R4.1 L1 M-1 closure / orchestrator triage 2026-05-22).
//! ============================================================================
//!
//! R2 §2 G-CORE-DSL row names 16 DSL cluster issues; the 3 closure pins
//! above cover #496 / #551 / #1020. The remaining 13 issues each receive
//! a HARD-RULE-12 clause-(b) BELONGS-NAMED-NOW disposition NOW, naming a
//! specific destination per HARD RULE 12 (no phantom "carry to brief").
//!
//! **Named destination for all 13**: the R5 **G-CORE-DSL implementer
//! brief**'s "in-scope" enumeration. The orchestrator authors the
//! G-CORE-DSL implementer brief at G-CORE-DSL wave-dispatch time and
//! MUST include this disposition row as an input-constraint. The
//! corresponding closure pins (one per issue) land at G-CORE-DSL
//! wave-implementation-time (per pim-12 / §3.6e: RED-PHASE staged-pin
//! → un-ignore at the wave that closes it). Captured in R4.1 triage at
//! `.addl/phase-4-meta/R4.1-TRIAGE.md` §"R5 brief-author-time named
//! destinations" → "For R5 G-CORE-DSL brief".
//!
//!   - #1000 (fwd-2) Diagnostic span is point not range — Phase-6 AI
//!     repair-prompt UX needs start_offset+end_offset → BELONGS-NAMED-NOW
//!   - #934  (fwd-1) parse_value numeric literals allocate String
//!     instead of slicing &src → BELONGS-NAMED-NOW
//!   - #931  (fwd-1) peek_at(n) uses chars().nth(n) — O(n) UTF-8 walk;
//!     2nd byte ASCII lookahead → BELONGS-NAMED-NOW
//!   - #929  (fwd-1) no benches exist; `[lib] bench = false`; parse/
//!     emit/round-trip perf unmeasured → BELONGS-NAMED-NOW
//!   - #848  (surf-1) id_for fallback `_ => 'op'` silent on new
//!     #[non_exhaustive] PrimitiveKind variants — defense-in-depth gap
//!     → BELONGS-NAMED-NOW
//!   - #841  (surf-1) Diagnostic.error_code is &'static str but 4 valid
//!     values are pub(crate) consts — consumers hardcode strings →
//!     BELONGS-NAMED-NOW
//!   - #839  (surf-1) CompileError::Io abused at devserver consumer to
//!     wrap non-IO engine-registration errors → BELONGS-NAMED-NOW
//!   - #790  (qual-2) 'Emit' name triple-overloaded across
//!     CompileError::Emit + fn emit + PrimitiveKind::Emit →
//!     BELONGS-NAMED-NOW
//!   - #760  (qual-2) 10 parse_err callsites use post-token cursor;
//!     only 2 capture start position — diagnostic line/column points at
//!     wrong span → BELONGS-NAMED-NOW
//!   - #671  (qual-1) validate_shapes two error arms duplicate ~22 LOC
//!     of Diagnostic construction; collapsible to single helper →
//!     BELONGS-NAMED-NOW
//!   - #663  (qual-1) parse_primitive 154-LOC 12-arm dispatch has
//!     structural duplication reducible via parse-helpers (~50-70 LOC
//!     achievable) → BELONGS-NAMED-NOW
//!   - #608  (safe-3) Rust DSL accepts empty/whitespace handler_id;
//!     TS DSL rejects with EDslInvalidShape — cross-language rule-mirror
//!     gap (§3.5g) → BELONGS-NAMED-NOW
//!   - #545  (safe-2) compile_str + compile_file have no source-length
//!     / file-size cap — sibling DoS to #496 recursion depth →
//!     BELONGS-NAMED-NOW
//!
//! Each destination row above is realified by the G-CORE-DSL implementer
//! brief at wave-dispatch time (orchestrator-authored from R4.1 triage
//! input). No phantom "TODO" carry. ============================================

#![allow(clippy::unwrap_used)]

use benten_dsl_compiler::{CompileError, compile_str};

// ---------------------------------------------------------------------------
// #496 (safe-1) — parse_object/parse_value unbounded recursion →
// process stack-overflow. PRODUCTION-ARM: a deeply-nested object
// literal through the real `compile_str` recursive-descent path.
// WOULD-FAIL (here: the process aborts with a stack overflow, which a
// test-runner reports as the test crashing — the RED signal) if the
// recursion is unbounded. POST G-CORE-DSL: a BOUNDED, typed
// `CompileError` rejection (NOT a process abort).
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-DSL"]
fn dsl_496_deeply_nested_object_is_bounded_rejection_not_stack_overflow() {
    // Build `handler 'x' { write('post', {a:{a:{a:...{a:'v'}...}}}) -> respond }`
    // with depth D large enough to blow a default 8 MiB native stack
    // through the parse_object⇄parse_value mutual recursion. 200_000
    // is comfortably past the unbounded-recursion failure point while
    // staying well within a sane post-fix bound's REJECT path (the fix
    // is expected to cap nesting at a small constant, e.g. 64/128, and
    // return a typed error long before this depth).
    const DEPTH: usize = 200_000;
    let mut body = String::from("'v'");
    for _ in 0..DEPTH {
        body = format!("{{ a: {body} }}");
    }
    let src = format!("handler 'deep' {{ write('post', {body}) -> respond }}");

    // PRE (ed03729a): this call recurses ~DEPTH frames deep and
    // ABORTS the process with a stack overflow — the RED signal.
    // POST G-CORE-DSL: it returns `Err(CompileError::...)` (a bounded,
    // typed depth-limit rejection — Parse/Semantic/Emit, NOT a panic,
    // NOT a process abort).
    let result = compile_str(&src);
    assert!(
        result.is_err(),
        "#496: deeply-nested object MUST be a bounded typed rejection, \
         not an accepted parse (and certainly not a process \
         stack-overflow). got Ok"
    );
    // The rejection MUST be a typed CompileError (the bounded-depth
    // guard), not an unrelated downstream failure.
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            CompileError::Parse(_) | CompileError::Semantic(_) | CompileError::Emit(_)
        ),
        "#496: depth-limit rejection MUST be a typed CompileError; got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// #551 (safe-2) — validate_shapes enforces NO upper bound on the
// SANDBOX `fuel` budget (only rejects negative / non-int). An
// arbitrarily huge fuel value compiles cleanly → effectively-unbounded
// wasmtime budget. PRODUCTION-ARM: the real `compile_str` →
// `validate_shapes` pass. WOULD-FAIL if the budget is
// effectively-unbounded (i.e. an enormous fuel value is accepted).
// POST G-CORE-DSL: an enforced upper bound → typed
// `E_DSL_INVALID_SHAPE` rejection above the cap.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-DSL"]
fn dsl_551_sandbox_fuel_has_enforced_upper_bound() {
    // i64::MAX fuel — a non-negative integer, so it passes the
    // baseline `Value::Int(n) if *n >= 0 => {}` arm at lib.rs:830 and
    // compiles cleanly at ed03729a. A real upper-bound guard MUST
    // reject this.
    let src = format!(
        "handler 'huge-fuel' {{ sandbox(module: 'm', fuel: {}, wallclock_ms: 1000, output_limit: 1024) -> respond }}",
        i64::MAX
    );

    let result = compile_str(&src);
    // PRE (ed03729a): Ok — no upper bound enforced (the RED signal).
    // POST G-CORE-DSL: Err(CompileError::Emit { error_code:
    // E_DSL_INVALID_SHAPE, .. }) — the budget is bounded above.
    assert!(
        result.is_err(),
        "#551: an i64::MAX SANDBOX fuel budget MUST be rejected by an \
         enforced upper bound (effectively-unbounded wasmtime budget is \
         the safe-2 hazard). got Ok at ed03729a"
    );
    let err = result.unwrap_err();
    let diag = err
        .diagnostic()
        .expect("#551: upper-bound rejection carries a typed Diagnostic");
    assert_eq!(
        diag.error_code, "E_DSL_INVALID_SHAPE",
        "#551: over-budget SANDBOX fuel MUST trip E_DSL_INVALID_SHAPE \
         (the existing SANDBOX-shape typed-error surface); got {}",
        diag.error_code
    );
}

// A second #551 arm: the bound is enforced for `wallclock_ms` /
// `output_limit` too (the same SANDBOX_INT_PROPS set the validator
// already iterates) — the guard must cover the whole budget surface,
// not just `fuel`. Pinned separately so a fuel-only fix is caught as
// incomplete (§3.6b sub-rule-4: pin the SPECIFIC arm per property).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-DSL"]
fn dsl_551_sandbox_wallclock_and_output_limit_also_bounded() {
    let src = format!(
        "handler 'huge-wall' {{ sandbox(module: 'm', fuel: 1000, wallclock_ms: {}, output_limit: {}) -> respond }}",
        i64::MAX,
        i64::MAX
    );
    let result = compile_str(&src);
    assert!(
        result.is_err(),
        "#551: i64::MAX wallclock_ms / output_limit MUST also be \
         upper-bounded (the guard covers the whole SANDBOX budget \
         surface, not fuel-only). got Ok at ed03729a"
    );
}

// ---------------------------------------------------------------------------
// #1020 — engine-side `Subgraph → DSL` inverse pretty-printer. At
// ed03729a NO inverse exists (grep-confirmed). The Core deliverable is
// the ENGINE-SIDE inverse; the admin-UI workflow-editing CONSUMER
// wiring is Composing G-COMP-1 and is explicitly OUT of this Core R3
// scope (r2 TF-10 Covers + plan G-CORE-DSL def).
//
// The inverse's public API name is a G-CORE-DSL implementer decision
// (not yet built). This RED-PHASE pin is written against the
// ROUND-TRIP PROPERTY: parse(print(compile_str(SRC))) yields a
// Subgraph with a CID equal to compile_str(SRC)'s — i.e. the inverse
// is a true inverse modulo canonical normalization. The test stays
// `#[ignore]` until the inverse lands; at un-ignore time the
// G-CORE-DSL implementer wires the actual inverse call in place of the
// `unimplemented!()` shim below and removes `#[ignore]` (§3.6e). The
// shim makes the RED state explicit and the PASS condition concrete
// WITHOUT this R3 file inventing a public API name (which would be a
// SHAPE-not-SUBSTANCE over-constraint).
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-DSL (wire the #1020 inverse in place of the shim)"]
fn dsl_1020_subgraph_to_dsl_inverse_round_trips() {
    // A representative non-trivial handler covering body + composition.
    let src = "handler 'inv-rt' { read('post') -> transform({ up: $title }) -> respond }";
    let compiled = compile_str(src).unwrap();
    let cid_forward = compiled.subgraph.cid().unwrap();

    // G-CORE-DSL un-ignore step: replace this `unimplemented!()`
    // statement with the real inverse + bind its result to `dsl_text`,
    // e.g.
    //   let dsl_text: String = benten_dsl_compiler::subgraph_to_dsl(&compiled.subgraph);
    // (final name = implementer's choice; #1020). Kept as a
    // statement-level divergence (not an assigned-from-diverging-expr)
    // so the RED-PHASE shim is clippy-clean.
    unimplemented!("#1020: wire the engine-side Subgraph→DSL inverse here at G-CORE-DSL");

    // parse(print(g)) ≅ g — re-compiling the printed DSL yields the
    // same canonical CID (the round-trip / inverse property; WOULD-FAIL
    // if the printer drops/perturbs any canonical-bearing structure).
    #[allow(unreachable_code)]
    {
        let dsl_text: String = String::new(); // replaced by the #1020 inverse at un-ignore
        let reparsed = compile_str(&dsl_text).expect("#1020: printed DSL must re-compile");
        assert_eq!(
            reparsed.subgraph.cid().unwrap(),
            cid_forward,
            "#1020: Subgraph→DSL→Subgraph round-trip MUST preserve the \
             canonical CID (true inverse modulo normalization)"
        );
    }
}
