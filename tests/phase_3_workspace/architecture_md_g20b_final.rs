//! R3-E RED-PHASE pin for G20-B FINAL 10-crate ARCHITECTURE.md transition
//! (wave-8b; C-15 + arch-r1-3 final-state).
//!
//! **HISTORICAL ANCHOR (Phase-3 R3-E origin).** This pin asserts the
//! 10-crate FINAL transition that landed at Phase-3 close. Phase-4-Foundation
//! added 2 more crates (benten-platform-foundation + benten-renderer-tauri,
//! making 12 total); the assertions in this file are subset-checks that
//! continue to pass at the 12-crate doc state at HEAD (no regression). The
//! authoritative post-Phase-4-Foundation pin is
//! `crates/benten-engine/tests/architecture_md_12_crate_count_post_phase_4_foundation_canaries.rs`
//! which carries the 12-crate count canary forward. This file is preserved
//! as historical anchor for the G20-B transition + may be retired at a
//! future pre-tag sweep per `docs/future/phase-4-backlog.md §4.44`.
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.8 G20-B):
//!
//! - `tests/architecture_md_lists_10_crates_with_benten_id_and_benten_sync` (FINAL state)
//!
//! ## Ownership
//!
//! Per r2-test-landscape §13 ambiguous-ownership pre-emption:
//! - R3-A: G14-A1 intermediate (9-crate in-flight callout) — already landed
//! - R3-C: G16-A intermediate (10-crate in-flight callout naming benten-sync)
//! - **R3-E (this file): G20-B FINAL** 10-crate transition narrative
//!
//! Disjoint test-fn: this file owns the FINAL pin only; R3-A's
//! `architecture_md_in_flight_callouts_present_for_benten_id_and_benten_sync_native_only`
//! lives in the sibling `architecture_md_state.rs` file.
//!
//! ## What G20-B establishes
//!
//! Final 8 → 10 crate transition narrative + benten-id row + benten-sync
//! row + crate-graph DAG update. Replaces G14-A's + G16-A's in-flight
//! callouts with the durable narrative.

#![allow(clippy::unwrap_used)]

#[test]
fn architecture_md_lists_10_crates_with_benten_id_and_benten_sync() {
    // C-15 + arch-r1-3 FINAL pin. G20-B-wired (wave-8b).
    //
    // OBSERVABLE consequence: the 8→10 crate transition lands as the
    // canonical narrative. Defends against the failure mode where
    // intermediate callouts persist after the phase closes.
    let arch_md_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("ARCHITECTURE.md");
    let arch_md = std::fs::read_to_string(&arch_md_path).unwrap();

    // The doc must enumerate all 10 crates by name:
    let required_crates = [
        "benten-errors",
        "benten-core",
        "benten-graph",
        "benten-ivm",
        "benten-caps",
        "benten-eval",
        "benten-engine",
        "benten-dsl-compiler",
        "benten-id",
        "benten-sync",
    ];
    for c in &required_crates {
        assert!(
            arch_md.contains(c),
            "ARCHITECTURE.md must list crate '{c}' at G20-B FINAL"
        );
    }

    // benten-sync must be marked native-only per CLAUDE.md baked-in #17:
    assert!(
        arch_md.contains("native-only")
            || arch_md.contains("native only")
            || arch_md.contains("NATIVE-ONLY"),
        "ARCHITECTURE.md must declare benten-sync as native-only \
         per CLAUDE.md baked-in #17"
    );

    // The intermediate "in-flight" callouts from G14-A1 + G16-A are
    // GONE (replaced by the durable 10-crate narrative):
    assert!(
        !arch_md.contains("Phase-3 in flight"),
        "G20-B must replace in-flight callouts with durable narrative; \
         residual 'Phase-3 in flight' callout still present"
    );
}
