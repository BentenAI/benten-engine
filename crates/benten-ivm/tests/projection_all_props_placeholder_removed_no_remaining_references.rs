//! R4-FP-3 RED-PHASE pin: `Projection::AllProps` placeholder removed
//! post-G23-0b — no remaining references in production source.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.3 G23-0b row
//!   (Projection::AllProps placeholder removal grep-assert).
//! - `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-0b
//!   must-pass: "remove `Projection::AllProps` placeholder
//!   (CRATES-DEEP-DIVE §4)".
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter
//!   (closes r4-tc-6 Family C missing IVM pin #2 of 3).
//! - CRATES-DEEP-DIVE §4: the `AllProps` no-op identity projection was
//!   a Phase-3 G15-A scaffold for the typed-output projection landing
//!   in Phase-4-Foundation G23-0b.
//!
//! ## What this pin asserts
//!
//! Post-G23-0b the typed-output projection shape covers all 5 canonical
//! views (including View 4 Rules + View 5 Current per mat-r1-1). The
//! `AllProps` identity projection is no longer needed and is removed
//! from the production enum + every callsite.
//!
//! Grep-assert form: walk `crates/benten-ivm/src/` (production source
//! only) for the identifier `AllProps`. Zero matches required.
//!
//! ## RED-PHASE staged-pin discipline (pim-12 §3.6e)
//!
//! Un-ignored at G23-0b wave-3 close (placeholder retirement is the
//! last step of the 5-view re-expression). Pin source AT HEAD: the
//! identifier IS present at `crates/benten-ivm/src/algorithm_b.rs:205`
//! per HEAD grep — RED-PHASE is correct because removal lands at
//! G23-0b, not before.
//!
//! ## §3.6f SHAPE-not-SUBSTANCE
//!
//! SHAPE: walked root exists + non-empty. SUBSTANCE: grep finds zero
//! `AllProps` occurrences in production source AFTER G23-0b lands.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn ivm_src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn walk_rs_files(dir: &std::path::Path, visit: &mut dyn FnMut(&std::path::Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_rs_files(&p, visit);
        } else if p.extension().is_some_and(|e| e == "rs")
            && let Ok(body) = std::fs::read_to_string(&p)
        {
            visit(&p, &body);
        }
    }
}

#[test]
#[ignore = "phase-4-foundation R4-FP-3 RED-PHASE — G23-0b wave-3 un-ignores at close. \
    Pin source: r2-test-landscape.md §2.3 G23-0b + CRATES-DEEP-DIVE §4. Family C IVM \
    AllProps-placeholder-retirement residual (was orphaned by R3 family charter per r4-tc-6). \
    AllProps identifier present at HEAD per benten-ivm/src/algorithm_b.rs:205 — un-ignore \
    after G23-0b strips the variant + cleans callsites."]
fn projection_all_props_placeholder_removed_no_remaining_references() {
    let src = ivm_src_root();
    assert!(
        src.is_dir(),
        "crates/benten-ivm/src/ MUST exist (smoke-check against vacuous-truth — \
         pim-18 §3.6f vacuous-truth-defense)"
    );

    let mut walked_files: usize = 0;
    let mut offenders: Vec<String> = Vec::new();
    walk_rs_files(&src, &mut |path, body| {
        walked_files += 1;
        for (lineno, line) in body.lines().enumerate() {
            // Match the identifier `AllProps` as a word — both
            // `Projection::AllProps` and bare `AllProps` qualifiers.
            // Permissive substring is fine here because the identifier
            // is intentionally specific and unique.
            if line.contains("AllProps") {
                offenders.push(format!("{}:{} {}", path.display(), lineno + 1, line.trim()));
            }
        }
    });

    assert!(
        walked_files > 0,
        "vacuous-truth defense: walked zero .rs files under {} — pin is vacuously \
         green (pim-18 §3.6f)",
        src.display()
    );

    assert!(
        offenders.is_empty(),
        "Projection::AllProps placeholder MUST be removed from benten-ivm/src/ \
         post-G23-0b (CRATES-DEEP-DIVE §4). Found {} remaining references:\n{:#?}\n\
         The G23-0b 5-view re-expression typed-output projections (per mat-r1-1) \
         supersede the AllProps identity projection. Remove the variant from the \
         Projection enum AND every callsite + match arm.",
        offenders.len(),
        offenders,
    );
}
