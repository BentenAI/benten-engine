//! Phase 2a R3 doc-completeness — `docs/ERROR-CATALOG.md` covers every
//! Phase-2a ErrorCode variant, bidirectionally.
//!
//! Traces to: `.addl/phase-2a/00-implementation-plan.md` §3 G11-A
//! (`tests/error_catalog_covers_all_phase_2a_codes` — dx-r1 close-out
//! gate; per-group catalog-entry authoring contract) + §1 (9 firing + 5
//! reserved Phase-2a codes).
//!
//! R4 qa-r4-4 + qa-r4-5: the canonical Phase-2a firing + reserved lists
//! live on `benten_errors::PHASE_2A_FIRING_CODES` /
//! `PHASE_2A_RESERVED_CODES`. This test and
//! `crates/benten-errors/tests/phase_2a_error_codes_present.rs` consume
//! the same consts so drift is a compile-level impossibility.
//!
//! The "bidirectional" property: (a) every ErrorCode variant we know about
//! appears as a `### <CODE>` subsection in `docs/ERROR-CATALOG.md`, AND
//! (b) every `### E_*` heading in the catalog round-trips through
//! `ErrorCode::from_str(literal)` to a non-`Unknown` variant. A drift in
//! either direction surfaces as a targeted failure diff.
//!
//! Owned by `qa-expert` per R2 landscape §8.5. TDD red-phase.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::{ErrorCode, PHASE_2A_FIRING_CODES, PHASE_2A_RESERVED_CODES};
use std::fs;
use std::path::PathBuf;

/// Repo-root path to the catalog.
fn catalog_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // tests/
        .and_then(|p| p.parent()) // repo root
        .expect("repo root")
        .join("docs/ERROR-CATALOG.md")
}

/// Parse every `### E_*` heading in the catalog. Trims whitespace.
fn catalog_headings(catalog: &str) -> Vec<String> {
    catalog
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("### ")
                .filter(|rest| rest.starts_with("E_"))
                .map(ToString::to_string)
        })
        .collect()
}

/// Every Phase-2a firing code must be present in `docs/ERROR-CATALOG.md`.
///
/// Direction (a): ErrorCode → catalog. Consumes the canonical list.
#[test]
fn error_catalog_has_every_phase_2a_firing_code() {
    let catalog = fs::read_to_string(catalog_path()).expect("read ERROR-CATALOG.md");
    let missing: Vec<&str> = PHASE_2A_FIRING_CODES
        .iter()
        .map(ErrorCode::as_str)
        .filter(|code| !catalog.contains(&format!("### {code}")))
        .collect();
    assert!(
        missing.is_empty(),
        "ERROR-CATALOG.md is missing entries for Phase-2a firing codes: \
         {missing:?}\n\
         Each group owns its own catalog entries (plan §3 G11-A per-group \
         authoring contract). Add a `### <CODE>` subsection for each \
         missing code with firing_site, fix_hint, and first_phase."
    );
}

/// Every Phase-2a reserved code (firing site in Phase 3) must be present
/// with a clearly marked reserved tag so operators reading the catalog
/// don't confuse it with an active firing code.
#[test]
fn error_catalog_has_every_phase_2a_reserved_code() {
    let catalog = fs::read_to_string(catalog_path()).expect("read ERROR-CATALOG.md");
    let missing: Vec<&str> = PHASE_2A_RESERVED_CODES
        .iter()
        .map(ErrorCode::as_str)
        .filter(|code| !catalog.contains(&format!("### {code}")))
        .collect();
    assert!(
        missing.is_empty(),
        "ERROR-CATALOG.md is missing entries for Phase-2a reserved codes: \
         {missing:?}\n\
         These are HostError discriminants reserved at §9.2 Option A; \
         they must appear in the catalog with a `reserved — fires in \
         Phase 3` tag per dx-r1 per-group authoring contract."
    );
}

/// Bidirectional drift — direction (b): catalog → ErrorCode, restricted to
/// the Phase-2a set. Every Phase-2a firing + reserved code, when found as
/// a heading in the catalog, MUST round-trip through
/// `ErrorCode::from_str(literal)` to the same variant (not `Unknown`). A
/// heading that fails to round-trip signals a catalog entry whose code
/// slot was never wired into the Rust enum — a "paper code" with no
/// reachable fire site.
///
/// The test deliberately scopes to the Phase-2a set: Phase-3 sync codes
/// (E_SYNC_*) and Phase-2 SANDBOX codes are catalog-documented ahead of
/// their enum wiring; flagging them as drift now would break the catalog-
/// first workflow. The inverse direction (every Phase-2a enum variant →
/// catalog heading) is enforced by the two tests above.
#[test]
fn phase_2a_codes_round_trip_through_from_str() {
    let catalog = fs::read_to_string(catalog_path()).expect("read ERROR-CATALOG.md");
    let headings: std::collections::HashSet<String> =
        catalog_headings(&catalog).into_iter().collect();

    let expected: Vec<&ErrorCode> = PHASE_2A_FIRING_CODES
        .iter()
        .chain(PHASE_2A_RESERVED_CODES.iter())
        .collect();

    let mut failures = Vec::new();
    for variant in expected {
        let literal = variant.as_str();
        if !headings.contains(literal) {
            // Covered by the two per-list tests above; skip here.
            continue;
        }
        let parsed = ErrorCode::from_str(literal);
        if parsed != *variant {
            failures.push(format!(
                "catalog heading `### {literal}` round-trips to {parsed:?}, expected {variant:?}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "Phase-2a catalog headings failed bidirectional round-trip:\n{}",
        failures.join("\n")
    );
}

/// Smoke: the catalog file itself exists and is non-empty. Guards against
/// accidentally deleting / renaming the path.
#[test]
fn error_catalog_file_exists_and_is_non_empty() {
    let path = catalog_path();
    let contents = fs::read_to_string(&path).expect("ERROR-CATALOG.md must exist at docs/");
    assert!(
        contents.len() > 1024,
        "ERROR-CATALOG.md looks truncated ({} bytes at {:?})",
        contents.len(),
        path
    );
}
