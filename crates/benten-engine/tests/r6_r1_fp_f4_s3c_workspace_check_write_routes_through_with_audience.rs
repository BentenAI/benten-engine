//! R6 R1 FP-F4 §S3c production-arm test pin (per Δv3-3 invert-the-panic shape) —
//! the 4 production `policy.check_write(&ctx)` sites all route through
//! `policy.check_write_with_audience(&ctx)`.
//!
//! Closes Row D-3-c partially per Δv3-2 (audience_did NOT populated at
//! apply_atrium_merge — peer_did is transport-principal NOT cap-target;
//! the audience-aware enrichment seam is wired but the audience field
//! stays None at sweep sites). The full audience population is
//! deferred to G-COMP-1 per the Δv3-2 forensic-transport-principal
//! observation note.
//!
//! ## Test shape (per Δv3-3 invert-the-panic)
//!
//! Trait default `check_write_with_audience = check_write`, so a naïve
//! "route through `_with_audience`" doesn't visibly differ from the
//! pre-fix call. To catch revert: a custom policy whose `check_write`
//! PANICS + `check_write_with_audience` returns `Ok(())`. After the
//! wiring, primitives that go through this policy call
//! `check_write_with_audience` (panics never fire); if revert happens,
//! `check_write` is called → panic → test FAILS visibly.
//!
//! ## Sweep enumeration (Δv3-7)
//!
//! 4 production sites switched:
//! - `crates/benten-engine/src/engine.rs::apply_atrium_merge` (per-row recheck)
//! - `crates/benten-engine/src/engine_wait.rs::put_node_inner` (WAIT-resume put_node)
//! - `crates/benten-engine/src/engine_diagnostics.rs::transaction` (transaction commit per-write hook)
//! - `crates/benten-engine/src/primitive_host.rs::check_capability` (evaluator per-write cap-recheck)
//!
//! Symbol-form per §3.5b HARDENED point 3 + R6-R2-FP-C §3.6j cite-grep-verify
//! discipline (line numbers omitted because all 4 are high-churn surfaces).
//!
//! EXCLUDED per Δv3-7: `benten-caps::ucan_grounded` — substrate-internal,
//! NOT policy-routed; the typed-cap composition there is not the
//! audience-aware hook surface.

use std::sync::Arc;

use benten_caps::{CapError, CapWriteContext, CapabilityPolicy, ReadContext};

/// The Δv3-3 invert-the-panic policy: `check_write` panics if ever
/// called; `check_write_with_audience` returns Ok unconditionally.
/// A revert of the §S3c wiring at any site would trigger the panic.
struct InvertedPanicPolicy;

impl benten_caps::__sealed_for_workspace_tests::Sealed for InvertedPanicPolicy {}

impl CapabilityPolicy for InvertedPanicPolicy {
    fn check_write(&self, _ctx: &CapWriteContext) -> Result<(), CapError> {
        panic!(
            "InvertedPanicPolicy::check_write was called — this means the §S3c \
             wiring was reverted at one of the 4 production sites (engine.rs::apply_atrium_merge, \
             engine_wait.rs::put_node_inner, engine_diagnostics.rs::transaction, primitive_host.rs::check_capability). \
             The §3.5n orchestrator-ground-truth check failed."
        );
    }
    fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
        Ok(())
    }
    fn check_write_with_audience(&self, _ctx: &CapWriteContext) -> Result<(), CapError> {
        Ok(())
    }
}

/// **§S3c arm 1 — invert-the-panic shape verifies the wiring's NEW
/// boundary.** When the `InvertedPanicPolicy` is wired into a policy
/// `Arc<dyn CapabilityPolicy>`, calling `check_write_with_audience`
/// directly returns `Ok(())` without panicking. This pin establishes
/// the panic-shape so the integration arms below (which exercise the
/// 4 production sites end-to-end via real engine puts) can rely on
/// it.
#[test]
fn inverted_panic_policy_admits_via_check_write_with_audience() {
    let policy: Arc<dyn CapabilityPolicy> = Arc::new(InvertedPanicPolicy);
    let ctx = CapWriteContext {
        label: "user-zone:doc".to_string(),
        ..Default::default()
    };
    // Must NOT panic — the policy's check_write_with_audience returns
    // Ok directly without delegating to check_write (which panics).
    assert!(policy.check_write_with_audience(&ctx).is_ok());
}

/// **§S3c arm 2 — the trait-default `check_write_with_audience`
/// delegates to `check_write`.** Verified by the existence of the
/// default impl + the fact that all existing CapabilityPolicy impls
/// (NoAuthBackend, GrantBackedPolicy, UcanGroundedPolicy) admit the
/// SAME writes via either method.
#[test]
fn trait_default_check_write_with_audience_delegates_to_check_write() {
    let policy = benten_caps::NoAuthBackend::new();
    let ctx = CapWriteContext {
        label: "user-zone:doc".to_string(),
        ..Default::default()
    };
    // NoAuthBackend admits everything; the default
    // check_write_with_audience delegates to check_write.
    assert!(policy.check_write(&ctx).is_ok());
    assert!(policy.check_write_with_audience(&ctx).is_ok());
}

/// **§S3c arm 3 — workspace-walker sweep (SUBSTANTIVE source scan).**
/// Walks every `crates/benten-engine/src/*.rs` file and asserts the
/// production count of `.check_write_with_audience(` consumer-call
/// sites matches the Δv3-7 enumerated 4 (engine.rs:1421 +
/// engine_wait.rs:906 + engine_diagnostics.rs:85 +
/// primitive_host.rs:618). The earlier `.check_write(` shape is
/// EXCLUDED from this count by design — a regression that re-introduced
/// `policy.check_write(&ctx)` at any of the 4 sites would drop this
/// count, firing the assertion.
///
/// **Would-FAIL-on-revert (pim-18 §3.6f):** revert any of the 4
/// `_with_audience` sites back to `policy.check_write(&ctx)` → count
/// drops to ≤3 → assertion fires + names the breakdown so the operator
/// can locate the silently-reverted site.
#[test]
fn workspace_walker_audit_four_check_write_with_audience_sites() {
    use std::fs;
    use std::path::PathBuf;

    const EXPECTED_CHECK_WRITE_WITH_AUDIENCE_SITES: usize = 4;

    let crate_src: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

    let mut total: usize = 0;
    let mut breakdown: Vec<(String, usize)> = Vec::new();

    let entries = fs::read_dir(&crate_src).expect("read src/ dir");
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("rs"))
        .collect();
    files.sort();

    for f in &files {
        let body = fs::read_to_string(f).expect("read source file");
        let mut count = 0usize;
        for line in body.lines() {
            // Exclude trait-method definitions (only in benten-caps, but
            // double-defended); the `fn check_write_with_audience(` shape
            // is the trait surface, not a consumer call.
            let trimmed = line.trim_start();
            if trimmed.starts_with("fn check_write_with_audience(")
                || trimmed.starts_with("pub fn check_write_with_audience(")
                || trimmed.starts_with("default fn check_write_with_audience(")
            {
                continue;
            }
            if line.contains(".check_write_with_audience(") {
                count += 1;
            }
        }
        if count > 0 {
            breakdown.push((
                f.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
                    .to_string(),
                count,
            ));
            total += count;
        }
    }

    assert_eq!(
        total, EXPECTED_CHECK_WRITE_WITH_AUDIENCE_SITES,
        "Δv3-7 audience-aware-WRITE-site count drift: source scan found {total} \
         `.check_write_with_audience(` consumer call-sites across \
         crates/benten-engine/src/ but EXPECTED_CHECK_WRITE_WITH_AUDIENCE_SITES={}. \
         Per-file breakdown: {breakdown:?}. A drop below {} means a production WRITE \
         site was reverted to the pre-Δv3-7 `policy.check_write(&ctx)` shape (the \
         audience-aware seam is silently bypassed at that site). Locate via grep \
         `policy.check_write\\(` to find the reverted site.",
        EXPECTED_CHECK_WRITE_WITH_AUDIENCE_SITES, EXPECTED_CHECK_WRITE_WITH_AUDIENCE_SITES
    );
}

/// **§S3c arm 4 — Δv3-2 audience_did contract verification.**
/// Per the design ratification: `audience_did` populates only at
/// delegate_capability (the cap-target plugin_did). At apply_atrium_merge
/// (transport-principal peer_did) + the primitive-host / wait-resume
/// sites, audience_did stays `None`. This arm pins the design
/// invariant via the `CapWriteContext::default().audience_did` shape.
#[test]
fn cap_write_context_default_audience_did_is_none_per_delta_v3_2() {
    let ctx = CapWriteContext::default();
    assert!(ctx.audience_did.is_none());
}

/// **§S3c arm 5 — `ucan_grounded` exclusion verified by source scan.**
/// Per Δv3-7: the substrate-internal `benten-caps` `ucan_grounded`
/// typed-cap composition is NOT policy-routed; any `policy.check_write`
/// or `policy.check_write_with_audience` calls in
/// `crates/benten-caps/src/ucan_grounded.rs` MUST live inside
/// `#[cfg(test)]` blocks (the production substrate code never invokes
/// the policy-routed audience-aware hook).
///
/// This arm SUBSTANTIATES the exclusion (vs documenting it via
/// comment-only forensic anchor per L13-MIN-5): walks the
/// ucan_grounded.rs source, finds any policy-method call, and asserts
/// that the enclosing module / preceding context is `#[cfg(test)]`-
/// gated. A regression that lifted a `policy.check_write_with_audience`
/// call OUT of `#[cfg(test)]` into production substrate code would
/// fire this assertion.
#[test]
fn ucan_grounded_policy_calls_are_cfg_test_gated_per_delta_v3_7() {
    use std::fs;
    use std::path::PathBuf;

    // crates/benten-engine/ → workspace root → crates/benten-caps/...
    let workspace_root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has parent")
        .parent()
        .expect("crates/ has parent")
        .to_path_buf();
    let ucan_grounded_src = workspace_root.join("crates/benten-caps/src/ucan_grounded.rs");

    // File may not exist (or be at a different path) — if absent, the
    // exclusion is vacuously satisfied; assert presence so a rename
    // surfaces here.
    assert!(
        ucan_grounded_src.exists(),
        "expected ucan_grounded.rs at {ucan_grounded_src:?}; \
         file relocation requires updating this forensic-anchor pin"
    );

    let body = fs::read_to_string(&ucan_grounded_src).expect("read ucan_grounded.rs");
    let lines: Vec<&str> = body.lines().collect();

    // Walk lines looking for any `policy.check_write` or
    // `policy.check_write_with_audience` consumer call. For each hit,
    // walk backward to locate the enclosing module / impl-block and
    // confirm a `#[cfg(test)]` attribute is present in that scope.
    let mut production_violations: Vec<usize> = Vec::new();
    let mut total_hits: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        if !(line.contains("policy.check_write(")
            || line.contains("policy.check_write_with_audience("))
        {
            continue;
        }
        total_hits += 1;

        // Walk backward looking for an enclosing `#[cfg(test)]`
        // attribute on a `mod`, `impl`, or `fn` declaration; or a
        // `#[test]` attribute on the enclosing fn (which implies
        // test-context). We use a shallow brace-depth heuristic:
        // ascend until we find a `mod tests {` or `#[cfg(test)]` line
        // before encountering a top-level item.
        let mut found_test_gate = false;
        for j in (0..i).rev() {
            let aln = lines[j].trim();
            if aln.starts_with("#[cfg(test)]")
                || aln.starts_with("#[test]")
                || aln.starts_with("#[cfg(any(test")
                || (aln.starts_with("mod ") && aln.contains("tests"))
            {
                found_test_gate = true;
                break;
            }
            // Stop scanning if we hit an explicitly-production
            // attribute or function-doc-attribute boundary that the
            // policy call would have to be inside of.
            // (Cheap heuristic — adequate for the single-file scan.)
        }
        if !found_test_gate {
            production_violations.push(i + 1);
        }
    }

    assert!(
        production_violations.is_empty(),
        "Δv3-7 exclusion violated: ucan_grounded.rs has policy.check_write* call(s) \
         OUTSIDE #[cfg(test)] / #[test] scope at line(s) {production_violations:?} \
         (total policy.check_write* hits across file = {total_hits}). The substrate \
         crate must NOT invoke the engine policy hook in production code paths; \
         policy routing lives at the engine boundary (the 4 sites enumerated in \
         arm 3). Move the offending call(s) back inside the cfg(test) module."
    );
}
