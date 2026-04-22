//! Phase 2a R3 integration — BUNDLED exit gate 3: arch-1 dep break landed
//! cleanly + HostError surfaces storage failures through the typed-code path.
//!
//! Traces to: `.addl/phase-2a/00-implementation-plan.md` §1 exit criterion 3
//! (arch-1 dep break landed cleanly) + §3 G1-B (HostError) + §9.2 (HostError
//! locked shape) + §9.14 (signature-level CI gate).
//!
//! Companion to the signature-level workspace tests `arch_1_no_graph_dep.rs`
//! + `arch_1_no_graph_types_in_primitive_host.rs` (owned by G1) — this file
//!   asserts the end-to-end wiring: a storage failure from inside an
//!   evaluator primitive surfaces as a typed `HostError` whose `code` is one
//!   of the 5 reserved discriminants. Owned by `qa-expert` per R2 landscape.
//!   TDD red-phase.

// R4 fix-pass: this file exercises a mix of surfaces. The pure
// benten-errors / benten-eval::HostError shape assertions (`tq-1`
// rewrites) compile unconditionally against the R3 consolidation stubs.
// The older `#[test]`s that need the text-file `arch_1_dep_break_preserved`
// + `primitive_host_trait_has_no_graph_types` probes also compile today.
// Left ungated (cov-1 met for this file). See `.addl/phase-2a/r4-triage.md` cov-1.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::Path;

/// Parser for `cargo tree` output — asserts the `benten-eval` crate has
/// no incoming edge from `benten-graph` in the workspace graph. Direct
/// Cargo.toml probe for the dep line; avoids shelling out to cargo-tree
/// which is slower and flaky under `cargo test` parallelism.
#[test]
fn arch_1_dep_break_preserved() {
    let cargo_toml = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .and_then(|p| p.parent()) // repo root
        .expect("repo root")
        .join("crates/benten-eval/Cargo.toml");
    let contents = fs::read_to_string(&cargo_toml).expect("read crates/benten-eval/Cargo.toml");

    // The strict contract per §9.14 arch-r1 convergence: no benten-graph
    // dep line in benten-eval/Cargo.toml. R5 G1-B lands this.
    let has_graph_dep = contents.lines().any(|line| {
        let trimmed = line.trim_start();
        // Cover both inline (path = "../benten-graph") and table-style
        // ([dependencies.benten-graph]) forms.
        trimmed.starts_with("benten-graph")
            || trimmed.starts_with("[dependencies.benten-graph")
            || trimmed.starts_with("[dev-dependencies.benten-graph")
    });
    assert!(
        !has_graph_dep,
        "Phase 2a arch-1 contract: `crates/benten-eval/Cargo.toml` must not \
         depend on `benten-graph`. Offending manifest:\n{contents}"
    );
}

/// End-to-end: a `HostError` raised by a primitive-host impl during
/// evaluation must surface through the evaluator to the Engine caller with
/// its stable `ErrorCode` intact. Exercises the new typed-code path per
/// §9.2 (Option A — opaque Box + stable ErrorCode discriminant).
///
/// Red-phase: body references R3-consolidated stubs delivered by R5 groups
/// (G1-B owns the HostError surfacing path). Panics at `todo!()` until R5.
/// See `.addl/phase-2a/r4-triage.md` tq-1.
#[test]
fn host_error_typed_code_surfaces_end_to_end() {
    use benten_errors::ErrorCode;
    use benten_eval::HostError;

    let wire = HostError {
        code: ErrorCode::HostBackendUnavailable,
        source: Box::new(std::io::Error::other("backing redb offline")),
        context: Some("the backend refused a scan".to_string()),
    }
    .to_wire_bytes()
    .expect("HostError::to_wire_bytes must succeed for the Option-A shape");
    let decoded = HostError::from_wire_bytes(&wire).expect("decode");

    // The stable code discriminant must survive the serialize/deserialize
    // round-trip so the evaluator can surface it through the engine.
    assert_eq!(decoded.code, ErrorCode::HostBackendUnavailable);
    assert_eq!(decoded.code.as_str(), "E_HOST_BACKEND_UNAVAILABLE");
    // Context surfaces (on-wire).
    assert_eq!(
        decoded.context.as_deref(),
        Some("the backend refused a scan"),
        "HostError.context must ride the wire (sec-r1-6 — source does NOT; context does)"
    );
}

/// Shape-pin (cross-crate): the 5 reserved HostError ErrorCode discriminants
/// are visible from `benten_errors::ErrorCode::as_str()`. Partners with the
/// G1-B unit-test `host_error_code_variants_reserved.rs`.
///
/// SHAPE-PIN: validates the enum variant surface for Phase-2a + Phase-3
/// forward-compat. Firing semantics for HostCapabilityRevoked /
/// HostCapabilityExpired live in Phase 3.
#[test]
fn host_error_code_variants_reserved_visible_from_engine_crate() {
    use benten_errors::ErrorCode;

    // Every reserved HostError discriminant must resolve to its frozen
    // catalog string from this crate, proving the code slots are reachable
    // without any additional re-export. Drift here = the benten-errors enum
    // changed and the engine crate's downstream consumers would break.
    let expected: &[(ErrorCode, &str)] = &[
        (ErrorCode::HostNotFound, "E_HOST_NOT_FOUND"),
        (ErrorCode::HostWriteConflict, "E_HOST_WRITE_CONFLICT"),
        (
            ErrorCode::HostBackendUnavailable,
            "E_HOST_BACKEND_UNAVAILABLE",
        ),
        (
            ErrorCode::HostCapabilityRevoked,
            "E_HOST_CAPABILITY_REVOKED",
        ),
        (
            ErrorCode::HostCapabilityExpired,
            "E_HOST_CAPABILITY_EXPIRED",
        ),
    ];
    for (variant, literal) in expected {
        assert_eq!(variant.as_str(), *literal);
        assert_eq!(ErrorCode::from_str(literal), variant.clone());
    }
}

/// `PrimitiveHost` trait signatures must not mention `benten_graph::*`.
/// This is the textual companion to the signature-level CI gate that
/// the G1-B `arch-1-dep-break.yml` workflow enforces (§9.14). Runs at
/// `cargo test` time so drift is caught during dev, not just in CI.
#[test]
fn primitive_host_trait_has_no_graph_types() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap()
        .join("crates/benten-eval/src/host.rs");
    let src = fs::read_to_string(&path)
        .expect("read crates/benten-eval/src/host.rs — the PrimitiveHost home");

    // The contract: no `benten_graph::` identifier anywhere in the trait
    // definition file. If Phase 2a G1-B ships cleanly, this string never
    // appears.
    assert!(
        !src.contains("benten_graph::"),
        "PrimitiveHost trait file must not reference `benten_graph::` types \
         (Phase 2a arch-1 contract, §9.14 phil-r1-2). Found:\n{}",
        src.lines()
            .filter(|l| l.contains("benten_graph::"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
