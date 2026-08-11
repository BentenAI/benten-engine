//! R6 phase-close — the SANDBOX memory axis has a REAL production writer.
//!
//! ## The defect this file exists to prevent
//!
//! `docs/SANDBOX-LIMITS.md` §2 has always listed four enforcement axes,
//! each with a per-call DSL override. Three of them were real:
//! `fuel`, `wallclock_ms` and `output_limit` are read from the SANDBOX
//! OperationNode's property bag at
//! `crates/benten-engine/src/primitive_host.rs::execute_sandbox`.
//!
//! The memory axis was NOT. `SandboxConfig::memory_bytes` was read (it
//! reached `SandboxResourceLimiter`), so it looked wired — but no
//! production path ever assigned it. Every production SANDBOX call ran
//! at the 64 MiB default, and the advertised `memoryLimitBytes` DSL
//! override existed nowhere in the tree: the string appeared only in
//! `docs/SANDBOX-LIMITS.md`, which additionally contradicted itself
//! twelve lines below the table ("not exposed on the DSL in Phase 2b").
//!
//! Per CLAUDE.md rule 15 the doc described the behaviour we want, and
//! the memory axis was the odd one out among four siblings, so the CODE
//! moved rather than the prose.
//!
//! ## Why the existing suite did not catch it
//!
//! `crates/benten-eval/tests/sandbox_memory.rs` constructs a
//! `SandboxConfig { memory_bytes: 64 * 1024, .. }` **directly** and
//! calls the eval-side executor. That proves the LIMITER works; it says
//! nothing about whether any production caller can reach it. This test
//! goes through `Engine::call`, which is the only path a real handler
//! takes.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Cid, Value};
use benten_engine::{Engine, PrimitiveSpec, SubgraphSpec};
use benten_eval::PrimitiveKind;

/// A guest that grows its linear memory by 32 pages (2 MiB) in one call.
/// Comfortably under the 64 MiB default ceiling, comfortably over a
/// tightened 256 KiB one — so the SAME module succeeds or fails purely
/// on whether the per-handler `memory_limit` was honoured.
///
/// `memory.grow` returns -1 on failure rather than trapping, so the
/// module checks the result and executes `unreachable` on refusal; the
/// resource limiter's `MemoryCapExceededMarker` is what actually routes
/// the typed error, but the explicit check keeps the fixture honest if
/// wasmtime ever changes its refusal shape.
fn grow_2mib_module_bytes() -> Vec<u8> {
    wat::parse_str(
        r#"
        (module
          (memory 1)
          (func (export "run") (result i32)
            (if (i32.eq (memory.grow (i32.const 32)) (i32.const -1))
              (then unreachable))
            i32.const 0))
        "#,
    )
    .expect("grow module compiles")
}

fn cid_for_bytes(bytes: &[u8]) -> Cid {
    Cid::from_blake3_digest(*blake3::hash(bytes).as_bytes())
}

/// Build a SANDBOX -> RESPOND spec, optionally carrying a per-handler
/// `memory_limit` property (the canonical eval-side snake_case name the
/// TS DSL's `memoryLimitBytes` translates to).
fn sandbox_spec(handler_id: &str, module_cid_str: &str, memory_limit: Option<i64>) -> SubgraphSpec {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("module".into(), Value::Text(module_cid_str.to_string()));
    props.insert(
        "caps".into(),
        Value::List(vec![Value::Text("host:compute:time".to_string())]),
    );
    if let Some(limit) = memory_limit {
        props.insert("memory_limit".into(), Value::Int(limit));
    }
    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Sandbox,
            properties: props,
        })
        .respond()
        .build()
}

/// Outcome of a dispatch, flattened to the one thing these tests care
/// about: did the guest's `memory.grow` succeed, and if not, under which
/// enforced ceiling?
///
/// A SANDBOX budget breach propagates out of `Engine::call` as `Err`
/// (not as a non-OK `Outcome`), and the typed error's message carries
/// the ceiling that was actually applied — which is precisely the value
/// under test here.
#[derive(Debug)]
enum GrowResult {
    Allowed,
    RefusedAt { code: String, message: String },
}

fn run(engine: &Engine, handler_id: &str) -> GrowResult {
    match engine.call(
        handler_id,
        "run",
        benten_core::Node::new(vec!["input".to_string()], Default::default()),
    ) {
        Ok(outcome) if outcome.is_ok_edge() => GrowResult::Allowed,
        Ok(outcome) => GrowResult::RefusedAt {
            code: outcome.error_code().unwrap_or("<none>").to_string(),
            message: outcome.error_message().unwrap_or_default(),
        },
        Err(e) => GrowResult::RefusedAt {
            code: format!("{:?}", e.error_code()),
            message: format!("{e}"),
        },
    }
}

impl GrowResult {
    fn is_allowed(&self) -> bool {
        matches!(self, GrowResult::Allowed)
    }
    /// True iff refused AND the enforced ceiling reported in the typed
    /// error is exactly `bytes`. Asserting on the NUMBER (not merely on
    /// "something failed") is what makes these guards catch a silent
    /// fall-back to the 64 MiB default.
    fn refused_at_exactly(&self, bytes: u64) -> bool {
        match self {
            GrowResult::RefusedAt { code, message } => {
                code.contains("SandboxMemoryExhausted") && message.contains(&bytes.to_string())
            }
            GrowResult::Allowed => false,
        }
    }
}

/// **Load-bearing falsification guard.**
///
/// FALSIFYING MUTATION (verified to fail): delete the
/// `op.properties.get("memory_limit")` block from
/// `crates/benten-engine/src/primitive_host.rs::execute_sandbox`
/// => the tightened handler inherits the 64 MiB default, its 2 MiB grow
///    succeeds, and the `E_SANDBOX_MEMORY_EXHAUSTED` assertion fails.
///
/// The BASELINE half is what makes the guard non-tautological: the same
/// module with no `memory_limit` must SUCCEED. Without it, a mutation
/// that made every SANDBOX call fail would still "pass" the tightened
/// assertion.
#[test]
fn sandbox_per_handler_memory_limit_is_honoured_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let module_bytes = grow_2mib_module_bytes();
    let module_cid = cid_for_bytes(&module_bytes);
    let module_cid_str = module_cid.to_base32();
    engine
        .register_module_bytes(&module_cid, &module_bytes)
        .unwrap();

    // ---- BASELINE: no per-handler limit => 64 MiB default => grows OK.
    let baseline = engine
        .register_subgraph(sandbox_spec("sbx.mem_default", &module_cid_str, None))
        .unwrap();
    let baseline_result = run(&engine, &baseline);
    assert!(
        baseline_result.is_allowed(),
        "BASELINE: a 2 MiB grow must succeed under the 64 MiB default — if \
         this fails the fixture is wrong, not the limiter. got {baseline_result:?}"
    );

    // ---- TIGHTENED: 256 KiB ceiling => the same 2 MiB grow is refused.
    let tightened = engine
        .register_subgraph(sandbox_spec(
            "sbx.mem_tightened",
            &module_cid_str,
            Some(256 * 1024),
        ))
        .unwrap();
    let tightened_result = run(&engine, &tightened);
    assert!(
        tightened_result.refused_at_exactly(256 * 1024),
        "per-handler `memory_limit` = 256 KiB MUST be observed at the SANDBOX \
         guest boundary, and the typed E_SANDBOX_MEMORY_EXHAUSTED must report \
         262144 — the ceiling actually applied. A 2 MiB grow silently \
         inheriting the 64 MiB default is the pre-R6 behaviour: the memory \
         axis had NO production writer at all. got {tightened_result:?}"
    );
}

/// The memory axis is TIGHTEN-ONLY — deliberately asymmetric with `fuel`
/// and `output_limit`, which accept any value. Memory is the D21
/// priority-1 axis because exhausting it can OOM-kill the host PROCESS
/// shared by every other handler, so a single handler must not be able
/// to raise the bound that protects everyone else.
///
/// FALSIFYING MUTATION (verified to fail): in `execute_sandbox`, replace
/// the guarded match arm with an unconditional
/// `config.memory_bytes = requested;`
/// => the 1 GiB request is applied, the 2 MiB grow succeeds, and the
///    assertion below fails.
#[test]
fn sandbox_per_handler_memory_limit_cannot_raise_the_engine_ceiling() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let module_bytes = grow_2mib_module_bytes();
    let module_cid = cid_for_bytes(&module_bytes);
    engine
        .register_module_bytes(&module_cid, &module_bytes)
        .unwrap();

    // A handler asking for 1 GiB — far above the 64 MiB engine ceiling.
    // The request must be IGNORED (not applied, not fatal): the call
    // still runs, still under 64 MiB.
    let widened = engine
        .register_subgraph(sandbox_spec(
            "sbx.mem_widened",
            &module_cid.to_base32(),
            Some(1024 * 1024 * 1024),
        ))
        .unwrap();
    let widened_result = run(&engine, &widened);
    assert!(
        widened_result.is_allowed(),
        "an over-ceiling `memory_limit` is ignored-and-logged, not fatal — the \
         call proceeds under the enforced 64 MiB. got {widened_result:?}"
    );

    // Proof the ceiling still binds: the same over-ceiling request does
    // NOT let a guest exceed 64 MiB. A 96 MiB grow must still be refused.
    let big_grow = wat::parse_str(
        r#"
        (module
          (memory 1)
          (func (export "run") (result i32)
            (if (i32.eq (memory.grow (i32.const 1536)) (i32.const -1))
              (then unreachable))
            i32.const 0))
        "#,
    )
    .unwrap();
    let big_cid = cid_for_bytes(&big_grow);
    engine.register_module_bytes(&big_cid, &big_grow).unwrap();
    let widened_big = engine
        .register_subgraph(sandbox_spec(
            "sbx.mem_widened_big",
            &big_cid.to_base32(),
            Some(1024 * 1024 * 1024),
        ))
        .unwrap();
    let big_result = run(&engine, &widened_big);
    assert!(
        big_result.refused_at_exactly(64 * 1024 * 1024),
        "a handler MUST NOT be able to raise its own memory ceiling above the \
         engine's 64 MiB: a 96 MiB grow has to stay refused — and refused AT \
         67108864 — even when the handler asked for 1 GiB. Memory is the one \
         axis whose exhaustion OOM-kills the shared host process. \
         got {big_result:?}"
    );
}
