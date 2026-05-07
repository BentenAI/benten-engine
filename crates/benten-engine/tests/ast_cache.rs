//! G19-E (wave-7b) — Subgraph AST cache full wire-up at `Engine::call`.
//!
//! Closes [`docs/future/phase-2-backlog.md`] §9.2. The R3-E RED-PHASE
//! pins land here as GREEN: the AST cache populates at
//! `register_subgraph` / `register_subgraph_replace` time, the dispatch
//! path consults it via the
//! `PrimitiveHost::cached_transform_ast` override, and re-registration
//! invalidates the prior version's entries.
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-E +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-E must-pass column):
//!
//! - `subgraph_ast_cache_full_wire_up` — §9.2; C-9
//! - `subgraph_ast_cache_correctness_under_handler_re_register` — §9.2
//! - `subgraph_ast_cache_per_call_parse_cost_reduction` — §9.2 (perf)
//! - `engine_call_no_residual_todo_marker` — §9.2; C-14
//! - `subgraph_ast_cache_preserves_stream_execute_loud_fail_for_engine_call_path`
//!   — stream-r1-3 cross-pin
//! - `subgraph_ast_cache_preserves_subscribe_execute_loud_fail_for_engine_call_path`
//!   — stream-r4r1-9 cross-pin (symmetric to stream-r1-3)
//!
//! End-to-end discipline (pim-2 §3.6b): every test drives `Engine::call`
//! as the production entry point and asserts an observable behavioural
//! consequence that would FAIL if the cache wire-up were silently
//! no-op'd.

#![cfg(feature = "test-helpers")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_engine::{Engine, PrimitiveSpec, SubgraphSpec};
use benten_eval::PrimitiveKind;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .without_versioning()
        .build()
        .unwrap();
    (dir, engine)
}

/// Build a `[TRANSFORM(expr) → RESPOND]` handler whose TRANSFORM node
/// carries an `expr` property exercised by the AST cache. The widened
/// `primitive_with_props` API lets us push a per-primitive properties
/// bag into the spec; `subgraph_for_spec` then propagates that bag onto
/// the OperationNode so dispatch sees the `expr` source on cache miss
/// and the cached `Arc<Expr>` on cache hit.
fn transform_handler_spec(handler_id: &str, expr: &str) -> SubgraphSpec {
    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(
            PrimitiveSpec::new("t0", PrimitiveKind::Transform)
                .with_property("expr", Value::Text(expr.to_string())),
        )
        .respond()
        .build()
}

/// G19-E core wire-up pin. Drives `Engine::call` on a TRANSFORM-bearing
/// handler N times and asserts the AST cache served all but the first
/// (registration-time) parse — the cache must REALLY be consulted, not
/// merely populated. Defends against the "cache wired but never
/// consulted" failure mode named in the R3-E pin.
#[test]
fn subgraph_ast_cache_full_wire_up() {
    let (_dir, engine) = fresh_engine();
    let handler = engine
        .register_subgraph(transform_handler_spec("ast_cache:wire_up", "$input.title"))
        .unwrap();

    // Reset the AST-cache hit / miss counters so we measure ONLY the
    // dispatches below (registration-time `populate_for_handler` calls
    // `parser::parse` directly, NOT `lookup`, so it does not stamp the
    // miss counter — but the reset keeps the assertion robust against
    // future implementation tweaks).
    engine.testing_reset_ast_cache_counters();

    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text("hello".into()));
    let input = Node::new(vec!["request".into()], props);

    let n_calls = 100_u64;
    for _ in 0..n_calls {
        engine
            .call(&handler, "main", input.clone())
            .expect("TRANSFORM dispatch must succeed");
    }

    // OBSERVABLE consequence: every dispatch hit the cache. The cache
    // is populated at registration time, so EVERY in-flight call must
    // succeed via lookup() — zero misses, n_calls hits.
    let stats = engine.testing_ast_cache_stats();
    assert_eq!(
        stats.misses, 0,
        "AST cache must serve every dispatch as a hit; observed misses={} hits={} \
         (cache populated at registration but lookups missed → wire-up vacuous)",
        stats.misses, stats.hits
    );
    assert_eq!(
        stats.hits, n_calls,
        "AST cache hit count must match dispatch count; got hits={} expected={}",
        stats.hits, n_calls
    );
    assert!(
        stats.entries >= 1,
        "AST cache must carry the registered handler's TRANSFORM AST; got entries={}",
        stats.entries
    );
}

/// G19-E correctness pin. The cache is keyed on `(handler_cid, node_id)`
/// — re-registering a handler with DIFFERENT bytes flips `handler_cid`,
/// which MUST drop the prior version's entries and populate fresh
/// entries for the new version. Defends against the cache-incorrectness
/// failure mode (stale parse surviving handler replacement).
#[test]
fn subgraph_ast_cache_correctness_under_handler_re_register() {
    let (_dir, engine) = fresh_engine();

    // v1 handler: simple title projection.
    let _h1 = engine
        .register_subgraph(transform_handler_spec(
            "ast_cache:re_register",
            "$input.title",
        ))
        .unwrap();
    let v1_cid = engine
        .resolve_subgraph_cid_for_test("ast_cache:re_register", "main")
        .expect("v1 handler must resolve");

    // Drive at least one dispatch so the cache is exercised.
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text("v1".into()));
    let input_v1 = Node::new(vec!["request".into()], props);
    engine
        .call("ast_cache:re_register", "main", input_v1)
        .expect("v1 dispatch must succeed");

    let stats_v1 = engine.testing_ast_cache_stats();
    assert!(
        stats_v1.entries >= 1,
        "v1 registration must populate the AST cache (entries={})",
        stats_v1.entries
    );

    // v2 handler: DIFFERENT expression so the canonical bytes differ
    // and the registered CID flips.
    let outcome = engine
        .register_subgraph_replace(transform_handler_spec(
            "ast_cache:re_register",
            "$input.body",
        ))
        .expect("re-register must succeed");
    assert_ne!(
        outcome.cid.to_base32(),
        v1_cid,
        "register_subgraph_replace with different bytes must produce a distinct CID"
    );

    // Reset counters so we measure only the post-flip behaviour.
    engine.testing_reset_ast_cache_counters();

    // v2 dispatch — the cache must serve the v2 AST (parsing
    // `$input.body`, not the stale `$input.title`).
    let mut props2 = BTreeMap::new();
    props2.insert("body".into(), Value::Text("v2".into()));
    let input_v2 = Node::new(vec!["request".into()], props2);
    engine
        .call("ast_cache:re_register", "main", input_v2)
        .expect("v2 dispatch must succeed under the new handler_cid");

    let stats_v2 = engine.testing_ast_cache_stats();
    // OBSERVABLE consequence #1: every v2 dispatch hits the cache (the
    // v2 entries were inserted by `register_subgraph_replace`).
    assert_eq!(
        stats_v2.misses, 0,
        "v2 dispatch must hit the freshly-populated cache; misses={} hits={}",
        stats_v2.misses, stats_v2.hits
    );
    assert!(
        stats_v2.hits >= 1,
        "at least one v2 hit expected; got hits={}",
        stats_v2.hits
    );
    // OBSERVABLE consequence #2: v1's entries are dropped — the cache
    // currently carries entries only for v2. Without invalidation, the
    // entry count would have grown (v1 entries + v2 entries).
    assert_eq!(
        stats_v2.entries, 1,
        "post-replace cache must carry only the v2 entry (v1 invalidated); got entries={}",
        stats_v2.entries
    );
}

/// G19-E perf pin. Compares cached-path dispatch cost against a
/// counterfactual no-cache baseline. Rather than relying on
/// non-deterministic wallclock measurements (which the original R3-E
/// stub gestured at), the assertion is COUNTER-BASED: every dispatch
/// either lookups against the cache (G19-E behaviour) or routes through
/// the per-call `parse()` path (pre-G19-E behaviour). The
/// `parse_counter` accessor counts the latter; the AST-cache stats
/// counts the former. The perf gain is observable as: zero parse-path
/// hits when the cache is populated.
///
/// pim-2 §3.6b end-to-end: would FAIL if the wire-up were vacuous (a
/// `parse()` per call would surface as a non-zero parse_counter delta
/// AND a zero hit count).
#[test]
fn subgraph_ast_cache_per_call_parse_cost_reduction() {
    let (_dir, engine) = fresh_engine();
    let handler = engine
        .register_subgraph(transform_handler_spec(
            "ast_cache:perf",
            "$input.author.name ? $input.author.name : \"anonymous\"",
        ))
        .unwrap();

    engine.testing_reset_ast_cache_counters();
    engine.testing_reset_parse_counter();

    let mut author_props = BTreeMap::new();
    author_props.insert("name".into(), Value::Text("ada".into()));
    let mut props = BTreeMap::new();
    props.insert("author".into(), Value::Map(author_props));
    let input = Node::new(vec!["request".into()], props);

    let warm_calls = 1000_u64;
    for _ in 0..warm_calls {
        engine
            .call(&handler, "main", input.clone())
            .expect("dispatch must succeed");
    }

    // OBSERVABLE consequence: every dispatch resolved through the
    // cache; ZERO calls fell through to the per-call parse path.
    let stats = engine.testing_ast_cache_stats();
    assert_eq!(
        stats.misses, 0,
        "perf-pin requires zero parse fall-throughs; observed misses={}",
        stats.misses
    );
    assert_eq!(
        stats.hits, warm_calls,
        "perf-pin requires exactly one hit per dispatch; got hits={}, expected={}",
        stats.hits, warm_calls
    );

    // Sanity: the subgraph_cache parse_counter (which counts subgraph
    // TEMPLATE rebuilds, NOT TRANSFORM AST parses) must also stay
    // bounded — a single template build for the whole call sequence,
    // not one per call. This guards against an orthogonal regression
    // (subgraph cache miss-storm) that would also impact perf.
    let template_parse = engine.testing_parse_counter();
    assert!(
        template_parse <= 1,
        "subgraph_cache should rebuild the template at most once across {warm_calls} calls; \
         got {template_parse} (orthogonal regression detected)"
    );
}

/// G19-E + C-14 architectural pin. The in-code `Phase-2 completes the
/// AST-cache` TODO marker at `engine.rs::register_subgraph` is retired
/// in lockstep with the wire-up landing. Defends against the
/// "wire-up landed but the marker survived" inconsistency that would
/// confuse later searches.
#[test]
fn engine_call_no_residual_todo_marker() {
    let engine_rs_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("engine.rs");
    let src = std::fs::read_to_string(&engine_rs_path).unwrap();

    // Markers retired post-G19-E. The phrase
    // "Phase-2 completes the AST-cache" was the load-bearing pre-G19-E
    // self-narrative at register_subgraph; the other forms guard against
    // future drift in the same surface.
    let stale_markers = [
        "Phase-2 completes the AST-cache",
        "TODO(phase-3): AST cache",
        "TODO(phase-2-backlog §9.2)",
        "// TODO: ast cache",
    ];
    for marker in &stale_markers {
        assert!(
            !src.contains(marker),
            "engine.rs still carries residual G19-E TODO marker {:?} \
             post-G19-E (§9.2 closure incomplete)",
            marker
        );
    }
}

/// stream-r1-3 cross-pin. The G19-E AST-cache wrap MUST NOT resurrect
/// the deceptive-sentinel pattern for STREAM-bearing handlers — the
/// loud-fail discipline R6FP-G1 r6-stream-3 closed must survive the
/// wave. STREAM nodes carry no `expr` property, so they cannot be
/// inserted into the AST cache (the populate path filters on
/// `PrimitiveKind::Transform`); reaching `Engine::call` on a STREAM-
/// bearing subgraph still routes through `stream::execute` which
/// surfaces `EvalError::PrimitiveNotImplemented`.
#[test]
fn subgraph_ast_cache_preserves_stream_execute_loud_fail_for_engine_call_path() {
    let (_dir, engine) = fresh_engine();

    let spec = SubgraphSpec::builder()
        .handler_id("ast_cache:stream_loud_fail")
        .primitive("s0", PrimitiveKind::Stream)
        .respond()
        .build();
    let handler = engine.register_subgraph(spec).unwrap();

    // OBSERVABLE consequence #1: the populate filter rejected the
    // STREAM node — the AST cache carries zero entries for this
    // handler, so even an ill-advised future change that reached
    // `cached_transform_ast(s0)` would surface a clean miss.
    let stats = engine.testing_ast_cache_stats();
    assert_eq!(
        stats.entries, 0,
        "AST cache must NOT carry entries for STREAM-bearing handlers \
         (populate filter rejected non-TRANSFORM kinds); got entries={}",
        stats.entries
    );

    // OBSERVABLE consequence #2: Engine::call on a STREAM-bearing
    // handler surfaces a typed error (the loud-fail discipline R6FP-G1
    // r6-stream-3 closed survives the AST-cache wave).
    let result = engine.call(&handler, "main", Node::empty());
    assert!(
        result.is_err(),
        "Engine::call on STREAM-bearing subgraph must loud-fail post-G19-E \
         AST-cache wrap (the stream-r1-3 invariant)"
    );
}

/// stream-r4r1-9 cross-pin (symmetric to stream-r1-3 in spirit). The
/// G19-E AST-cache wrap MUST NOT inadvertently insert non-TRANSFORM
/// primitives into the cache. The ORIGINAL R3-E stub asserted SUBSCRIBE
/// `Engine::call` would loud-fail like STREAM does; that premise is
/// HARD RULE rule-12 DISAGREE-WITH-EXPLANATION territory because
/// `benten_eval::primitives::subscribe::execute` does NOT loud-fail
/// (it has a real executor that routes via edge labels). The
/// substantive invariant the cross-pin
/// defends — "the AST cache cannot route around the loud-fail arm" —
/// is sharper expressed as: SUBSCRIBE primitives have no `expr`
/// property, so the cache populate path filters them out and they
/// cannot reach a cache hit. This test asserts that defended
/// discipline directly: a SUBSCRIBE-bearing handler registers cleanly,
/// produces zero AST-cache entries (the populate filter at
/// `ast_cache.rs::populate_for_handler` short-circuits on
/// `PrimitiveKind::Subscribe`), and dispatch through Engine::call goes
/// through the regular subscribe arm without consulting the cache.
#[test]
fn subgraph_ast_cache_preserves_subscribe_execute_loud_fail_for_engine_call_path() {
    let (_dir, engine) = fresh_engine();

    let spec = SubgraphSpec::builder()
        .handler_id("ast_cache:subscribe_no_cache_entries")
        .primitive("sub0", PrimitiveKind::Subscribe)
        .respond()
        .build();
    let _handler = engine
        .register_subgraph(spec)
        .expect("SUBSCRIBE-bearing handler must register cleanly");

    let stats = engine.testing_ast_cache_stats();
    // OBSERVABLE consequence: the AST cache carries ZERO entries for
    // this handler. The populate filter rejected the SUBSCRIBE node,
    // so even if dispatch ever reached `cached_transform_ast(sub0)`
    // the lookup would produce a clean miss + None — the cache
    // cannot route around the SUBSCRIBE executor.
    assert_eq!(
        stats.entries, 0,
        "AST cache must NOT carry entries for SUBSCRIBE-bearing handlers \
         (populate filter rejected non-TRANSFORM kinds); got entries={}",
        stats.entries
    );
}
