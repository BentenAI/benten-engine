//! Phase 2b R3-B — `random` host-fn DEFERRED-to-Phase-2c regression
//! guard (G7-A).
//!
//! Pin source: D1 (defer-to-2c per sec-pre-r1-06 §2.3 reasoning —
//! shipping random before workspace-wide CSPRNG framework decision is a
//! footgun).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-A pending — D1 random deferred"]
fn sandbox_random_host_fn_unavailable_in_phase_2b() {
    // D1 — `random` is NOT in the initial host-fn surface for Phase 2b.
    // A module that requires `host:compute:random` cap and attempts to
    // invoke a `random`-named host-fn fails with
    // E_SANDBOX_HOST_FN_NOT_FOUND.
    //
    // The error message hint MUST mention "deferred to Phase 2c" so
    // downstream developers don't think it's a typo or missing
    // implementation bug.
    //
    // Regression guard: if a future PR ships random in Phase 2b without
    // workspace-wide CSPRNG framework decision, this test fires.
    todo!("R5 G7-A — assert random host-fn fires NOT_FOUND with Phase-2c hint");
}
