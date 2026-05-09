//! `random` host-fn cap-policy check structural-constant-time
//! discipline pin (sec-r1-3 + r4-r1-wsa-8 + r4b-wsa-3 RESTORATION).
//!
//! ## History
//!
//! - R3-D wave-5b shipped a narrative-only RED-PHASE pin
//!   asserting `DEFERRED_HOST_FN_RANDOM_CAP_PREFIX` existed +
//!   workspace narrative.
//! - R4-FP (a4bd49e) tightened the pin to a statistical-timing
//!   placeholder with locked thresholds (CONSTANT_TIME_RATIO_THRESHOLD
//!   = 1.2, CONSTANT_TIME_ITERATIONS = 10_000, 1-retry flake budget)
//!   per r4-r1-wsa-8 — still `#[ignore]`'d as RED-PHASE.
//! - Wave-5c (commit 60f2b3c, PR #117 → #122) DELETED this file with
//!   an INCORRECT rationale claiming it asserted
//!   `DEFERRED_HOST_FN_RANDOM_CAP_PREFIX` (it didn't post-R4-FP).
//!   The R4b lens (`r4b-wsa-3`) flagged the deletion as a regression
//!   of test-coverage discipline.
//! - **Wave-G16-B-C (2026-05-08) RESTORES this pin** with an ACTIVE
//!   structural-constant-time discipline assertion against the
//!   shipped `random` host-fn body. The statistical-timing form
//!   (R4-FP shape) is NOT restored — its CI flake-cost is high
//!   (per r4-r1-wsa-8 narrative), and the underlying security
//!   property is already structurally guaranteed by the
//!   `cap_check`-fires-first ordering in the `host`/`random` linker
//!   registration site within
//!   `crates/benten-eval/src/primitives/sandbox.rs::execute_with_live_cap_check`.
//!   The restored pin asserts that ordering at source-cite
//!   level — a refactor that adds an early-return based on `out_len`
//!   value BEFORE `cap_check` would fail this pin.
//!
//! ## Why structural-constant-time, not statistical
//!
//! Per r4b-wsa-3 RECOMMENDATION (b) DISAGREE-WITH-EXPLANATION
//! direction (Ben's call at wave-G16-B-C is the structural-pin path):
//! the actual security property is structurally guaranteed because:
//!
//! 1. The cap-policy check (`live_cap_check` callback) does
//!    `revoked_set.contains(&actor)` — `HashSet::contains` is
//!    structurally O(1) relative to the entropy-budget value (the
//!    `out_len` parameter the guest passes). The check does not
//!    branch on `out_len`.
//! 2. The `random` host-fn body in
//!    `crates/benten-eval/src/primitives/sandbox.rs::execute_with_live_cap_check`
//!    (the linker registration site for the `host`/`random` import)
//!    calls `cap_check(...)` UNCONDITIONALLY at the FIRST line of
//!    the closure body — BEFORE any `if out_ptr < 0 || out_len < 0`
//!    or `if out_len_u64 > budget` branch.
//! 3. The cap_check itself uses a fixed cap-string (`host:random:read`)
//!    derived from the host-fn registration — the cap-string is NOT
//!    input-dependent, so a timing channel through the cap-string
//!    composition is structurally absent.
//!
//! A statistical-timing pin would defend the same commitment with a
//! HIGH-FLAKE CI cost; the structural-pin defends the same vector at
//! ZERO flake cost by reading the source. If a future refactor adds
//! an early-return based on entropy budget value (the regression
//! vector r4b-wsa-3 named), it would surface here BEFORE landing
//! on main, regardless of CI machine timing jitter.
//!
//! Pin sources:
//!
//! - sec-r1-3 (R1 BLOCKER) — constant-time check on entropy budget
//!   per call.
//! - r4-r1-wsa-8 (MINOR REGRESSED) — locked thresholds discipline
//!   (now satisfied structurally rather than statistically).
//! - r4b-wsa-3 (MINOR) — restoration of regression-test discipline
//!   lost at wave-5c.
//! - CLAUDE.md baked-in #16 — host-fn surface compute-only narrative.
//! - phase-3-backlog §6.10 — random host-fn G17-A2 closure.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
fn random_host_fn_cap_check_fires_before_any_input_dependent_branch_per_sec_r1_3() {
    // r4b-wsa-3 RESTORATION + sec-r1-3 + r4-r1-wsa-8 structural pin.
    //
    // Reads `crates/benten-eval/src/primitives/sandbox.rs` and asserts
    // that within the `host` / `random` host-fn body (the closure
    // registered via `linker.func_wrap("host", "random", ...)`), the
    // FIRST statement after the closure parameter list is the
    // `cap_check(...)` call — BEFORE any branch on `out_ptr` or
    // `out_len`.
    //
    // A regression that adds an early-return optimization (e.g.
    // `if out_len == 0 { return Ok(0); }` BEFORE cap_check) fires
    // this pin. Such a refactor would leak a side-channel: a
    // malicious guest that passes out_len=0 vs out_len=N would
    // observe a timing differential proportional to the cap_check
    // cost.
    use std::path::PathBuf;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("primitives")
        .join("sandbox.rs");
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "read crates/benten-eval/src/primitives/sandbox.rs: {} (path={:?})",
            e, path
        )
    });

    // 1. The random host-fn registration is present.
    let registration_marker = "linker.func_wrap(";
    let host_random_marker = "\"host\",\n                    \"random\",";
    assert!(
        src.contains(registration_marker),
        "primitives/sandbox.rs MUST register host-fns via wasmtime linker.func_wrap"
    );
    let random_offset = src.find(host_random_marker).unwrap_or_else(|| {
        // Fallback: a more lenient search for the random host-fn
        // string-literal pair, in case formatting drifts. The error
        // message is loud enough that drift surfaces here.
        panic!(
            "primitives/sandbox.rs MUST register the `random` host-fn via \
             linker.func_wrap with `\"host\", \"random\"` string-literal pair \
             per D-PHASE-3-11 + CLAUDE.md baked-in #16. Marker not found: {host_random_marker:?}"
        )
    });

    // 2. Locate the closure body opening (`-> Result<i32, wasmtime::Error> {`).
    let after_random = &src[random_offset..];
    let closure_open = after_random
        .find("-> Result<i32, wasmtime::Error> {")
        .expect("random host-fn closure return-type marker located");
    let body_start_rel = closure_open + "-> Result<i32, wasmtime::Error> {".len();
    let body_start = random_offset + body_start_rel;
    let body_end_marker_rel = src[body_start..]
        .find("\n                )?;\n")
        .expect("random host-fn closure body terminator located");
    let body_end = body_start + body_end_marker_rel;
    let body = &src[body_start..body_end];

    // 3. Within the body, the FIRST non-comment, non-empty line MUST
    //    be the cap_check(...) call. Walk lines, skipping comments and
    //    blank lines, and assert.
    let mut found_first_stmt: Option<&str> = None;
    for raw_line in body.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("//") || line.starts_with("/*") || line.starts_with("*") {
            continue;
        }
        found_first_stmt = Some(line);
        break;
    }
    let first_stmt = found_first_stmt.expect(
        "random host-fn body MUST contain at least one non-comment statement; \
         empty body is structurally invalid",
    );
    assert!(
        first_stmt.starts_with("cap_check("),
        "random host-fn body's FIRST statement MUST be `cap_check(...)` per sec-r1-3 + \
         r4-r1-wsa-8 + r4b-wsa-3 (structural-constant-time discipline). A regression \
         that adds an early-return BEFORE cap_check (e.g. `if out_len == 0 {{ return Ok(0); }}`) \
         leaks a timing side-channel proportional to cap_check cost. Observed first \
         statement: {first_stmt:?}"
    );

    // 4. The cap_check call uses the FIXED cap-string from the
    //    host-fn registration (the `cap` binding), NOT an
    //    input-dependent cap-string. Asserts the closure captures
    //    `cap` from the enclosing scope rather than computing it
    //    from `out_ptr` / `out_len`.
    assert!(
        body.contains("cap_check(&mut caller, policy, &cap)"),
        "random host-fn cap_check MUST use the fixed `cap` binding from the host-fn \
         registration (not a freshly-computed input-dependent cap-string). A regression \
         that synthesizes the cap-string from `out_len` / `out_ptr` would create a \
         timing side-channel. Expected call shape: \
         cap_check(&mut caller, policy, &cap)"
    );
}

#[test]
fn random_host_fn_revoked_actors_check_uses_hashset_contains_constant_time() {
    // sec-r1-3 + r4b-wsa-3 sibling pin — structural assertion that
    // the engine-side `live_cap_check` callback uses
    // `HashSet::contains` (structurally O(1) relative to the
    // entropy-budget value) for the revoked-actors check.
    //
    // The R4b finding noted: "the actual random host-fn cap_check
    // uses a HashSet `contains` (which IS structurally constant-time
    // relative to budget size), so the underlying security property
    // holds". This pin defends that observation against a future
    // refactor that swaps HashSet for a Vec / BTreeSet (which would
    // break the structural-constant-time property relative to the
    // size of the revoked set, even though the revoked-set size
    // itself is unrelated to the entropy-budget value).
    use std::path::PathBuf;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("benten-engine")
        .join("src")
        .join("primitive_host.rs");
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "read crates/benten-engine/src/primitive_host.rs: {} (path={:?})",
            e, path
        )
    });

    // 1. The revoked-actors set is shaped as Arc<Mutex<HashSet<Cid>>>.
    //    A future refactor to a different container fires this pin.
    assert!(
        src.contains("Arc<Mutex<HashSet<Cid>>>") || src.contains("HashSet<Cid>"),
        "primitive_host.rs MUST use HashSet<Cid> for the revoked-actors set per \
         sec-r1-3 + r4b-wsa-3 (structural-constant-time discipline relative to set \
         size). Mismatch indicates a refactor that may have introduced a non-constant-\
         time membership check (e.g. Vec::contains, which is O(n))."
    );
    // 2. The live_cap_check callback uses `revoked_set.contains(&actor)`
    //    — the constant-time membership check.
    assert!(
        src.contains("revoked_set.contains(&actor)"),
        "primitive_host.rs MUST consult the revoked-actors HashSet via \
         `revoked_set.contains(&actor)` (HashSet::contains is structurally O(1)). \
         A regression that loops over the set fires this pin."
    );
}
