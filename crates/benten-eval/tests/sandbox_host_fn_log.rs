//! Phase 2b R3-B — `log` host-fn unit test (G7-A).
//!
//! D1 + sec-pre-r1-06 §2.2 — 64 KiB per-call byte-volume cap to prevent
//! spam-based DOS or covert-channel high-bandwidth use.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-A pending — D1 log host-fn byte-volume cap"]
fn sandbox_host_fn_log_respects_byte_volume_cap_64kb() {
    // D1 + sec-pre-r1-06 §2.2 — `log` host-fn declared with
    //   behavior = { kind = "log_sink", per_call_byte_cap = 65536 }
    //
    // Test 1: a single log of 64 KiB succeeds.
    // Test 2: a single log of 64 KiB + 1 byte returns the typed error
    //         E_SANDBOX_HOST_FN_DENIED (or E_INV_SANDBOX_OUTPUT depending
    //         on routing — pin to whichever R5 picks; the per-call cap is
    //         distinct from D17's per-primitive output budget).
    // Test 3: 1000 successive log calls of 100 bytes each succeed
    //         (aggregate 100 KiB > 64 KiB but PER-CALL cap is what's
    //         being enforced; aggregate is enforced by D17 CountedSink
    //         + Inv-7 separately).
    //
    // R5 may decide that the per-call cap is enforced via the
    // CountedSink trampoline (D25 trampoline-counts default) — in which
    // case the assertion routes through CountedSink's per-host-fn limit.
    todo!("R5 G7-A — assert per-call 64 KiB cap with three sub-tests");
}
