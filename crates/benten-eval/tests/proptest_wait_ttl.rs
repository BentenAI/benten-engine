// Phase-3 G20-A2 (D12 wave-8a) — D12 WAIT TTL property test.
//
// `phase_2b_landed` cfg gate retired at G20-A2 wave-8a.
//
// Property: for any (ttl, resume_offset) pair with non-zero ttl ≤ 720h,
//   (resume_offset > ttl) ↔ E_WAIT_TTL_EXPIRED.
// No silent expiry, no permissive-Complete fallback.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use proptest::prelude::*;

proptest! {
    // 256 cases — the per-iteration cost involves building a real engine,
    // registering a subgraph, calling to suspension, advancing the clock,
    // and resuming (~80ms each). 256 iterations stays within ~25s; the
    // R2 spec's 10k-case target is documented-deferred to
    // `docs/future/phase-3-backlog.md §7.15` per G20-A2 wave-8a mr-7 —
    // a sibling pure-eval-layer proptest at `proptest_wait_ttl_pure_eval.rs`
    // (no engine boundary, fabricates `WaitMetadata` directly) carries
    // the high-iteration coverage. This engine-boundary proptest stays
    // at 256 cases so it remains load-bearing for cross-process
    // semantics (drives the persistence + resume protocol's full-stack
    // interactions).
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// `prop_wait_ttl_no_silent_expiry_in_resume` — D12 + R2 §3 + R1
    /// streaming-systems must-pass.
    ///
    /// Property: register a WAIT with `ttl_hours = ttl`; suspend; advance
    /// the wait-clock by `offset_hours`; attempt resume. Outcome MUST be:
    ///
    ///   (offset_hours > ttl)  →  resume errors with E_WAIT_TTL_EXPIRED
    ///   (offset_hours <= ttl) →  resume completes cleanly
    #[test]
    fn prop_wait_ttl_no_silent_expiry_in_resume(
        ttl in 1u32..=720,
        offset_hours in 0u32..=2000,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let mut engine = benten_engine::Engine::builder()
            .path(dir.path().join("benten.redb"))
            .build()
            .unwrap();

        // Use a handler_id derived from (ttl, offset) so each iteration
        // registers a fresh handler.
        let handler_id = format!("phase_3_g20_a2_prop_{ttl}_{offset_hours}");
        let mut props = std::collections::BTreeMap::new();
        props.insert(
            "signal".into(),
            benten_core::Value::Text(format!("test:phase-3:g20-a2:prop:{ttl}:{offset_hours}")),
        );
        props.insert(
            "ttl_hours".into(),
            benten_core::Value::Int(i64::from(ttl)),
        );
        let wait_ps = benten_engine::PrimitiveSpec {
            id: "w0".into(),
            kind: benten_engine::PrimitiveKind::Wait,
            properties: props,
        };
        let spec = benten_engine::SubgraphSpec::builder()
            .handler_id(&handler_id)
            .primitive_with_props(wait_ps)
            .respond()
            .build();
        engine.register_subgraph(spec).unwrap();
        let envelope = benten_engine::testing::testing_call_to_suspend(
            &mut engine, &handler_id,
        ).unwrap();

        benten_engine::testing::testing_advance_wait_clock(
            &engine,
            std::time::Duration::from_secs(u64::from(offset_hours) * 3600),
        );

        let result = engine.resume_with_meta(
            &envelope, benten_engine::ResumePayload::None,
        );

        if offset_hours > ttl {
            let err = result.expect_err("offset > ttl MUST fire E_WAIT_TTL_EXPIRED");
            prop_assert!(
                err.to_string().contains("E_WAIT_TTL_EXPIRED"),
                "offset_hours={} ttl={} → expected E_WAIT_TTL_EXPIRED, got: {}",
                offset_hours, ttl, err,
            );
        } else {
            // offset_hours == ttl is the boundary — TTL fires when
            // (now - suspend) >= ttl_hours * 3_600_000. Wall-clock now
            // at suspend is `~suspend_wallclock_ms`; advancing by
            // offset_hours * 3600s sets now to `suspend_wallclock_ms +
            // offset_hours * 3_600_000`. So `(now - suspend) ==
            // offset_hours * 3_600_000`, deadline ==
            // ttl_hours * 3_600_000. When offset_hours == ttl, now ==
            // deadline (>= deadline) so it MUST fire too.
            //
            // Tighten the property: strict less-than, not less-or-equal.
            if offset_hours == ttl {
                let err = result.expect_err("offset == ttl boundary fires (>=, not >)");
                prop_assert!(
                    err.to_string().contains("E_WAIT_TTL_EXPIRED"),
                    "boundary offset_hours == ttl == {} → expected E_WAIT_TTL_EXPIRED, got: {}",
                    ttl, err,
                );
            } else {
                prop_assert!(
                    result.is_ok(),
                    "offset_hours={} ttl={} → expected clean resume, got: {:?}",
                    offset_hours, ttl, result.err(),
                );
            }
        }
    }
}
