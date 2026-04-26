//! Algorithm B within 20% of hand-written gate test (G8-A).
//!
//! Pin source: `.addl/phase-2b/00-implementation-plan.md` §3 G8-A bench gate.
//! Landscape source: `.addl/phase-2b/r2-test-landscape.md` §4 row
//! `algorithm_b_vs_handwritten_per_view`.
//!
//! Reads per-view ratio ceilings from
//! `crates/benten-ivm/benches/algorithm_b_thresholds.toml` (per Phase-2a
//! convention: numeric thresholds live in `.toml` config so they can be
//! tightened independently of the bench source). After the companion
//! `algorithm_b_vs_handwritten` Criterion bench runs, this test parses
//! `target/criterion/algorithm_b_vs_handwritten/<view>/strategy/<A|B>/estimates.json`
//! and asserts `(B mean) / (A mean) ≤ ceiling` for each of the 5 views.
//!
//! The gate is intentionally separate from the bench harness so it runs in
//! `cargo test` (with bench results pre-staged in CI) instead of requiring
//! `cargo bench` in every test run.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

/// One row of the per-view ratio ceiling table.
struct RatioCeiling {
    view: &'static str,
    max_ratio: f64,
}

fn load_ceilings() -> Vec<RatioCeiling> {
    let toml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("algorithm_b_thresholds.toml");
    let raw = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", toml_path.display()));
    // R5 wires the actual TOML parse; for the red-phase the loader is a
    // placeholder so the file shape is exercised + the absent surface
    // produces a clear diagnostic.
    assert!(
        raw.contains("[gate]") && raw.contains("[per_view]"),
        "thresholds TOML missing required [gate] + [per_view] sections"
    );
    vec![
        RatioCeiling {
            view: "content_listing",
            max_ratio: 1.20,
        },
        RatioCeiling {
            view: "governance_inheritance",
            max_ratio: 1.20,
        },
        RatioCeiling {
            view: "version_current",
            max_ratio: 1.20,
        },
        RatioCeiling {
            view: "capability_grants",
            max_ratio: 1.20,
        },
        RatioCeiling {
            view: "event_dispatch",
            max_ratio: 1.20,
        },
    ]
}

/// Read a Criterion estimates file for the given view + strategy and return
/// the mean nanoseconds. Future API (G8-A): `benten_ivm::testing::criterion_estimates_mean_ns`
/// centralizes this so the eight-or-so call-sites across the workspace don't
/// each re-implement the JSON shape.
fn criterion_mean_ns(view: &str, strategy: &str) -> f64 {
    benten_ivm::testing::criterion_estimates_mean_ns(
        "algorithm_b_vs_handwritten",
        view,
        "strategy",
        strategy,
    )
    .unwrap_or_else(|e| {
        panic!(
            "missing Criterion estimates for view `{view}` strategy `{strategy}`: {e}\n\
             Run `cargo bench --bench algorithm_b_vs_handwritten` first to populate \
             target/criterion/."
        )
    })
}

#[test]
#[ignore = "Phase 2b G8-A pending — requires cargo bench prelude + TOML parser"]
fn algorithm_b_vs_handwritten_per_view_within_threshold() {
    let ceilings = load_ceilings();

    let mut violations: Vec<String> = Vec::new();

    for c in &ceilings {
        let a_ns = criterion_mean_ns(c.view, "A");
        let b_ns = criterion_mean_ns(c.view, "B");
        assert!(a_ns > 0.0, "view {} strategy A mean is non-positive", c.view);
        let ratio = b_ns / a_ns;
        if ratio > c.max_ratio {
            violations.push(format!(
                "view `{}`: B/A = {:.3} exceeds ceiling {:.3} \
                 (A = {:.1}ns, B = {:.1}ns)",
                c.view, ratio, c.max_ratio, a_ns, b_ns
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "Algorithm B exceeded the per-view ratio ceiling:\n  - {}\n\
         G8-A gate per `.addl/phase-2b/00-implementation-plan.md` §3.",
        violations.join("\n  - ")
    );
}
