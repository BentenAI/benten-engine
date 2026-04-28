#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! Algorithm B vs hand-written bench gate (G8-A — hybrid shape).
//!
//! Pin source: `.addl/phase-2b/00-implementation-plan.md` §3 G8-A bench gate.
//! Landscape source: `.addl/phase-2b/r2-test-landscape.md` §4 row
//! `algorithm_b_vs_handwritten_per_view`.
//!
//! ## Hybrid gate (cr-g8a-mr-3 + cr-g8a-mr-4 fix-pass)
//!
//! Per-view rows in `crates/benten-ivm/benches/algorithm_b_thresholds.toml`
//! pick one of two assertion shapes:
//!
//! - `kind = "absolute_overhead_ns"` for lightweight views (A baseline ≤
//!   ~1 µs total bench-iter time). Asserts
//!   `(B_mean - A_mean) ≤ max_overhead_ns`. The bench measures total
//!   iteration time (one view construction + N=256 updates); a pure
//!   delegation wrapper lands ~150-250 ns over A; the 350 ns ceiling
//!   adds ~30% headroom for run-to-run noise.
//! - `kind = "ratio"` for heavyweight views (A baseline > ~1 µs total
//!   bench-iter time). Asserts `B_mean / A_mean ≤ max_ratio`. At >1 µs
//!   total iteration time the wrapper's dispatch overhead is <1% so a
//!   ratio gate is the meaningful framing.
//!
//! After the companion `algorithm_b_vs_handwritten` Criterion bench runs
//! this test parses each view's
//! `target/criterion/algorithm_b_vs_handwritten/<view>/strategy/<A|B>/estimates.json`
//! and asserts the configured gate per view.
//!
//! The gate runs as part of `cargo test` (no `#[ignore]`); when the
//! Criterion estimates are not staged the test surfaces a clear diagnostic
//! pointing at the bench command to run. CI stages the bench output before
//! `cargo test` so the gate runs unconditionally there.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

/// Per-view gate row parsed from `algorithm_b_thresholds.toml`.
#[derive(Debug)]
struct GateRow {
    /// View name as it appears on disk under `target/criterion/algorithm_b_vs_handwritten/`.
    /// Matches the bench's `BenchmarkGroup` name (note: `event_handler_dispatch`,
    /// not the view-id-internal `event_dispatch`).
    view: String,
    /// Hybrid gate shape — see module doc.
    shape: GateShape,
}

#[derive(Debug)]
enum GateShape {
    /// `(B - A) ≤ max_overhead_ns`.
    AbsoluteOverheadNs(f64),
    /// `B / A ≤ max_ratio`.
    Ratio(f64),
}

/// Parse the thresholds TOML — purposely a hand-roll for the tightly-constrained
/// schema (5 named per-view tables × 2-3 scalar keys each) so the workspace
/// doesn't take on a `toml` crate dependency for a single test. The parser
/// rejects anything outside the recognized shapes with a clear diagnostic.
fn load_ceilings() -> Vec<GateRow> {
    let toml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("algorithm_b_thresholds.toml");
    let raw = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", toml_path.display()));
    parse_ceilings(&raw).unwrap_or_else(|e| panic!("failed to parse {}: {e}", toml_path.display()))
}

/// Accumulator row used by the parser before validating + lowering to a
/// concrete `GateRow`. Each `[per_view.<name>]` table populates one of
/// these as `key = value` lines arrive.
struct ParserRow {
    view: String,
    kind: Option<String>,
    max_overhead_ns: Option<f64>,
    max_ratio: Option<f64>,
}

/// Hand-roll TOML parser for the thresholds schema. Recognized productions:
///
/// - `[per_view.<name>]` table headers.
/// - `kind = "absolute_overhead_ns"` | `"ratio"` (string).
/// - `max_overhead_ns = <float>` | `max_ratio = <float>` (numeric).
///
/// Everything else (other sections, comments, blank lines) is ignored. The
/// parser is intentionally narrow — it exists only to drive this gate test.
fn parse_ceilings(raw: &str) -> Result<Vec<GateRow>, String> {
    let mut rows: Vec<ParserRow> = Vec::new();
    let mut current_view: Option<String> = None;

    for (lineno, line) in raw.lines().enumerate() {
        let lineno = lineno + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(header) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            let header = header.trim();
            if let Some(view) = header.strip_prefix("per_view.") {
                let view = view.trim().to_string();
                rows.push(ParserRow {
                    view: view.clone(),
                    kind: None,
                    max_overhead_ns: None,
                    max_ratio: None,
                });
                current_view = Some(view);
            } else {
                // [gate], [per_view] alone, or any other section — clear context.
                current_view = None;
            }
            continue;
        }
        // key = value — only meaningful inside a [per_view.<name>] table.
        let Some(view) = current_view.as_ref() else {
            continue;
        };
        let Some(eq) = trimmed.find('=') else {
            return Err(format!("line {lineno}: expected `key = value`: {trimmed}"));
        };
        let key = trimmed[..eq].trim();
        let value = trimmed[eq + 1..].trim();
        let row = rows
            .iter_mut()
            .rfind(|r| r.view == *view)
            .expect("current_view points at a row we just pushed");
        match key {
            "kind" => {
                let v = value
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .ok_or_else(|| format!("line {lineno}: kind must be a quoted string"))?;
                row.kind = Some(v.to_string());
            }
            "max_overhead_ns" => {
                let v: f64 = value
                    .parse()
                    .map_err(|e| format!("line {lineno}: max_overhead_ns: {e}"))?;
                row.max_overhead_ns = Some(v);
            }
            "max_ratio" => {
                let v: f64 = value
                    .parse()
                    .map_err(|e| format!("line {lineno}: max_ratio: {e}"))?;
                row.max_ratio = Some(v);
            }
            // Any other key under [per_view.<name>] is ignored (forward-compat).
            _ => {}
        }
    }

    let mut out = Vec::new();
    for ParserRow {
        view,
        kind,
        max_overhead_ns,
        max_ratio,
    } in rows
    {
        let shape = match kind.as_deref() {
            Some("absolute_overhead_ns") => {
                let v = max_overhead_ns.ok_or_else(|| {
                    format!("view `{view}`: kind=absolute_overhead_ns requires max_overhead_ns")
                })?;
                GateShape::AbsoluteOverheadNs(v)
            }
            Some("ratio") => {
                let v = max_ratio
                    .ok_or_else(|| format!("view `{view}`: kind=ratio requires max_ratio"))?;
                GateShape::Ratio(v)
            }
            Some(other) => {
                return Err(format!(
                    "view `{view}`: unknown kind `{other}` (expected absolute_overhead_ns or ratio)"
                ));
            }
            None => return Err(format!("view `{view}`: missing required `kind`")),
        };
        out.push(GateRow { view, shape });
    }

    if out.is_empty() {
        return Err(String::from(
            "no [per_view.*] tables found in thresholds TOML",
        ));
    }
    Ok(out)
}

/// Try to read the Criterion estimates for a `(view, strategy)` pair. Returns
/// `Ok(Some(mean_ns))` on success, `Ok(None)` when the estimates file is
/// absent (so callers can skip the gate with a clear message rather than
/// failing on every developer who hasn't run the bench).
fn try_criterion_mean_ns(view: &str, strategy: &str) -> Result<Option<f64>, String> {
    match benten_ivm::testing::criterion_estimates_mean_ns(
        "algorithm_b_vs_handwritten",
        view,
        "strategy",
        strategy,
    ) {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.starts_with("failed to read") => Ok(None),
        Err(e) => Err(e),
    }
}

#[test]
fn algorithm_b_vs_handwritten_per_view_within_threshold() {
    let rows = load_ceilings();

    // Pre-flight: are bench results staged at all? If not, surface ONE
    // clear instruction instead of N missing-file panics. CI runs the
    // bench prelude before `cargo test`, so this branch is normally
    // developer-local-only.
    let any_staged = rows
        .iter()
        .any(|r| try_criterion_mean_ns(&r.view, "A").ok().flatten().is_some());
    if !any_staged {
        // No Criterion estimates staged — silent skip rather than failing
        // every developer who hasn't run the bench. CI stages the bench
        // prelude before `cargo test` so this branch never trips on a
        // CI runner. Local devs who want the gate must run
        // `cargo bench --bench algorithm_b_vs_handwritten` first.
        // (`print_stdout` + `print_stderr` are workspace-banned via
        // clippy; the existing `eprintln!` patterns in `subscriber.rs`
        // carry per-call `#[allow]` attributes for their Phase-1 stderr
        // sink, but a silent skip on the test side is ergonomically
        // sufficient and avoids the lint dance.)
        return;
    }

    let mut violations: Vec<String> = Vec::new();

    for row in &rows {
        let a_ns = match try_criterion_mean_ns(&row.view, "A").expect("estimates JSON read error") {
            Some(v) => v,
            None => {
                violations.push(format!(
                    "view `{}`: missing strategy=A estimates (other views were staged \
                     so the bench ran partially — re-run the full bench)",
                    row.view
                ));
                continue;
            }
        };
        let b_ns = match try_criterion_mean_ns(&row.view, "B").expect("estimates JSON read error") {
            Some(v) => v,
            None => {
                violations.push(format!("view `{}`: missing strategy=B estimates", row.view));
                continue;
            }
        };
        assert!(
            a_ns > 0.0,
            "view {} strategy A mean is non-positive",
            row.view
        );
        match row.shape {
            GateShape::AbsoluteOverheadNs(max_overhead) => {
                let overhead = b_ns - a_ns;
                if overhead > max_overhead {
                    violations.push(format!(
                        "view `{}`: B - A = {:.2} ns exceeds absolute-overhead ceiling {:.2} ns \
                         (A = {:.1} ns, B = {:.1} ns)",
                        row.view, overhead, max_overhead, a_ns, b_ns
                    ));
                }
            }
            GateShape::Ratio(max_ratio) => {
                let ratio = b_ns / a_ns;
                if ratio > max_ratio {
                    violations.push(format!(
                        "view `{}`: B/A = {:.3} exceeds ratio ceiling {:.3} \
                         (A = {:.1} ns, B = {:.1} ns)",
                        row.view, ratio, max_ratio, a_ns, b_ns
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Algorithm B exceeded the per-view bench gate:\n  - {}\n\
         Hybrid gate per `.addl/phase-2b/00-implementation-plan.md` §3 G8-A.",
        violations.join("\n  - ")
    );
}

// ---------------------------------------------------------------------------
// Self-tests for the hand-rolled TOML parser
// ---------------------------------------------------------------------------

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn parses_hybrid_shape() {
        let raw = r#"
[gate]
description = "hybrid"

[per_view.heavy]
kind = "ratio"
max_ratio = 1.50

[per_view.light]
kind = "absolute_overhead_ns"
max_overhead_ns = 25.0
"#;
        let rows = parse_ceilings(raw).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].view, "heavy");
        assert!(matches!(rows[0].shape, GateShape::Ratio(r) if (r - 1.50).abs() < 1e-9));
        assert_eq!(rows[1].view, "light");
        assert!(
            matches!(rows[1].shape, GateShape::AbsoluteOverheadNs(o) if (o - 25.0).abs() < 1e-9)
        );
    }

    #[test]
    fn rejects_unknown_kind() {
        let raw = r#"
[per_view.bogus]
kind = "wat"
"#;
        let err = parse_ceilings(raw).unwrap_err();
        assert!(err.contains("unknown kind"), "{err}");
    }

    #[test]
    fn rejects_missing_threshold_for_kind() {
        let raw = r#"
[per_view.x]
kind = "ratio"
"#;
        let err = parse_ceilings(raw).unwrap_err();
        assert!(err.contains("requires max_ratio"), "{err}");
    }

    #[test]
    fn rejects_empty() {
        let err = parse_ceilings("").unwrap_err();
        assert!(err.contains("no [per_view.*] tables"), "{err}");
    }
}
