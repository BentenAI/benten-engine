//! R3-D RED-PHASE pin for `random` host-fn cap-policy check
//! constant-time discipline (G17-A1 wave 5b; sec-r1-3 + CLAUDE.md
//! baked-in #16 narrative).
//!
//! Pin source: r2-test-landscape §2.5 G17-A1 row
//! `random_host_fn_cap_policy_check_constant_time_no_fingerprint_leak`;
//! sec-r1-3 (constant-time check on entropy budget per call).
//!
//! ## Constant-time shape (sec-r1-3)
//!
//! Phase-3 wires the `random` host-fn (D-PHASE-3-11 RESOLVED-at-R1
//! workspace CSPRNG via `getrandom` direct + capability-gated entropy
//! budget). The cap-policy check on the entropy budget MUST run in
//! constant time relative to the requested number of bytes.
//!
//! WHY: a non-constant-time cap-policy check (e.g. one that early-
//! exits when the budget is exhausted vs. proceeds-then-rejects) leaks
//! a side-channel: a malicious guest that calls `random(N_bytes)` for
//! varied N can fingerprint the per-manifest budget remaining by
//! timing the rejection.
//!
//! Defense: the cap-policy check on `random` does the SAME work
//! regardless of whether the budget is exhausted — it always evaluates
//! the budget arithmetic, then conditionally returns the result.
//!
//! Pairs with G17-A2's `random_host_fn_capability_gated_entropy_budget`
//! (which pins the budget enforcement; this pin is about TIMING not
//! correctness).

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-A1 wave 5b wires constant-time cap-policy check on random host-fn per sec-r1-3"]
fn random_host_fn_cap_policy_check_constant_time_no_fingerprint_leak() {
    // sec-r1-3 pin. G17-A1 implementer wires this:
    //
    //   use std::time::Instant;
    //
    //   let sandbox = build_sandbox_with_random_cap_budget(/* budget = 4096 */);
    //
    //   // Measure timing of a request that's WITHIN budget:
    //   let mut within_times = Vec::new();
    //   for _ in 0..10_000 {
    //       let t = Instant::now();
    //       let _ = sandbox.invoke_random_host_fn(/* request 8 bytes, well within */);
    //       within_times.push(t.elapsed().as_nanos());
    //   }
    //
    //   // Measure timing of a request that EXCEEDS budget (rejected):
    //   let mut exceed_times = Vec::new();
    //   for _ in 0..10_000 {
    //       let t = Instant::now();
    //       let _ = sandbox.invoke_random_host_fn(/* request 99_999 bytes */);
    //       exceed_times.push(t.elapsed().as_nanos());
    //   }
    //
    //   // The medians + p99 timings overlap within statistical noise:
    //   //   (specifically, the difference is small enough that it
    //   //    cannot reliably fingerprint budget remaining via 100s of
    //   //    samples — Welch's t-test or similar)
    //
    //   let within_median = median(&within_times);
    //   let exceed_median = median(&exceed_times);
    //   // Implementer pins the threshold (e.g. 2x median is the
    //   // implausible-fingerprintable bound; want closer to 1.0x):
    //   let ratio = (within_median.max(exceed_median) as f64)
    //       / (within_median.min(exceed_median) as f64);
    //   assert!(ratio < 1.5,
    //       "random host-fn cap-policy check timing ratio {} suggests non-constant-time \
    //        per sec-r1-3 — within-budget median {} ns vs exceed-budget median {} ns",
    //       ratio, within_median, exceed_median);
    //
    // OBSERVABLE consequence: a regression that adds an early-exit
    // optimization to the cap-policy check (which would be welcomed
    // by performance-minded reviewers) is caught by this pin. Defends
    // sec-r1-3 directly.
    //
    // CAVEAT: timing-based tests are statistical; the implementer
    // selects a threshold + sample size that distinguishes the
    // "constant-time" vs "linear-in-budget" shape with high confidence
    // while tolerating CI host noise. This pin's BODY must include
    // explicit retry logic or a known-flaky-but-bounded budget per CI
    // best practices (mirrors Phase-2b proptest_sandbox_fuel timing
    // patterns).
    unimplemented!(
        "G17-A1 wires constant-time-discipline statistical assertion against random host-fn cap-policy check"
    );
}
