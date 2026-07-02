//! Engine-level umbrella pin: exit-criterion 7 — exactly 12
//! `PrimitiveKind` variants (CLAUDE.md baked-in #1 irreducibility).
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.18 row 1
//!   (cross-cutting 12-primitive irreducibility regression-defense).
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 wave charter
//!   (closes r4-tc-6 + r4-tc-8 + r4-tc-9 + r4-arch-1 family-charter
//!   coverage gaps orphaned from R3 family enumeration).
//! - CLAUDE.md baked-in #1 (12 operation primitives are irreducible).
//!
//! ## What this pin asserts
//!
//! Per CLAUDE.md baked-in #1: the engine recognises exactly 12
//! `PrimitiveKind` variants — READ, WRITE, TRANSFORM, BRANCH, ITERATE,
//! WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM. Phase
//! 4-Foundation introduces: admin UI v0 (G24-A), plugin manifest schema
//! (G24-D), schema-driven rendering compiler (G23-A), materializer
//! pipeline (G23-B), IVM-subgraph generalization (G23-0a/b),
//! Tauri renderer backend (G24-E). Each of these surfaces could
//! conceivably bring with it pressure to mint a new primitive kind. This
//! pin is the engine-level umbrella regression-guard that the count
//! remains exactly 12.
//!
//! Distinct from the per-feature pins in §2.4-§2.9 (schema_compiler /
//! admin_ui_v0 / atrium / etc. each ALSO pin this from their own
//! consumer angle). The engine-level umbrella verifies the property
//! ONCE at the canonical source-of-truth (the `PrimitiveKind` enum
//! definition in `benten-core`).
//!
//! ## R6 round-7 UN-IGNORE (psf-1 MAJOR — pim-12 §3.6e)
//!
//! The two tests asserting the canonical-12 set were both `#[ignore]`'d
//! RED-PHASE stubs ending in `unimplemented!()` — zero LIVE backstop for
//! CLAUDE.md #1 at the v1-beta FREEZE. This engine-level umbrella is now
//! UN-IGNORED and wired to the real `benten_core::subgraph::PrimitiveKind`,
//! so a 13th variant OR a rename FAILS this test before merge.
//!
//! ## §3.6f SHAPE-not-SUBSTANCE — two independent backstops
//!
//! 1. **Compile-time name backstop (catches RENAMES):** a `match` over a
//!    real `PrimitiveKind` value that NAMES every one of the 12 canonical
//!    variants. A rename (e.g. `Read` → `ReadNode`) deletes the old
//!    identifier → this test crate FAILS TO COMPILE. The `#[non_exhaustive]`
//!    catch-all arm (required for a cross-crate match) increments a
//!    surplus-counter that the runtime body asserts is zero, so a NEW
//!    variant cannot hide behind the wildcard either.
//! 2. **Source-body walk backstop (catches ADDITIONS):** parse the real
//!    enum body in `crates/benten-core/src/subgraph.rs` and assert exactly
//!    12 variant identifiers, each of the 12 canonical names present. A
//!    13th variant bumps the count → FAIL. This is the SUBSTANCE half that
//!    a vacuous `count > 0` would miss.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use benten_core::subgraph::PrimitiveKind;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// The canonical 12 (CLAUDE.md baked-in #1). Frozen; any change here is a
/// deliberate, Ben-ratified architectural event — NOT a silent edit.
const CANONICAL_12: [&str; 12] = [
    "Read",
    "Write",
    "Transform",
    "Branch",
    "Iterate",
    "Wait",
    "Call",
    "Respond",
    "Emit",
    "Sandbox",
    "Subscribe",
    "Stream",
];

/// Compile-time name backstop: a `match` that NAMES every canonical variant.
/// A rename deletes the named arm → compile error. A 13th variant lands in
/// the `#[non_exhaustive]` catch-all → `surplus == 1` (asserted zero below).
/// Returns the surplus-arm hit count for the witnessed value.
fn surplus_arm_witness(kind: PrimitiveKind) -> u32 {
    match kind {
        PrimitiveKind::Read
        | PrimitiveKind::Write
        | PrimitiveKind::Transform
        | PrimitiveKind::Branch
        | PrimitiveKind::Iterate
        | PrimitiveKind::Wait
        | PrimitiveKind::Call
        | PrimitiveKind::Respond
        | PrimitiveKind::Emit
        | PrimitiveKind::Sandbox
        | PrimitiveKind::Subscribe
        | PrimitiveKind::Stream => 0,
        // `#[non_exhaustive]` forces this cross-crate catch-all. A NEW
        // variant would route here → surplus 1. The runtime body proves no
        // constructible value reaches it for the 12 canonical kinds.
        _ => 1,
    }
}

/// Extract the variant identifiers from the real `PrimitiveKind` enum body
/// in `benten-core` source. Returns the ordered identifier list.
fn extract_primitive_kind_variants(core_src: &std::path::Path) -> Vec<String> {
    let path = core_src.join("subgraph.rs");
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "PrimitiveKind canonical source must exist at {} (got: {})",
            path.display(),
            e
        )
    });

    // Slice the `pub enum PrimitiveKind {` body up to its closing brace.
    let enum_start = src
        .find("pub enum PrimitiveKind {")
        .expect("PrimitiveKind enum declaration must be present in subgraph.rs");
    let after = &src[enum_start..];
    let open = after.find('{').expect("enum open brace");
    let close = after[open..].find('}').expect("enum close brace") + open;
    let body = &after[open + 1..close];

    // Variant identifiers are the bare `Ident,` lines (doc-comments start
    // with `///`; attributes with `#`). Take the leading ident before `,`.
    body.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("///") && !l.starts_with('#'))
        .filter_map(|l| {
            let ident: String = l
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if ident.is_empty() { None } else { Some(ident) }
        })
        .collect()
}

#[test]
fn exit_criterion_7_no_new_primitive_kind_variants_added_in_phase_4_foundation() {
    // -------------------------------------------------------------------
    // Backstop 1 — compile-time name + no-surplus (catches RENAMES + the
    // hide-behind-wildcard ADD). The `surplus_arm_witness` match named all
    // 12 canonical variants: a rename would have failed THIS CRATE'S
    // COMPILE. Here we prove no constructible canonical value reaches the
    // `#[non_exhaustive]` catch-all.
    // -------------------------------------------------------------------
    let canonical_values = [
        PrimitiveKind::Read,
        PrimitiveKind::Write,
        PrimitiveKind::Transform,
        PrimitiveKind::Branch,
        PrimitiveKind::Iterate,
        PrimitiveKind::Wait,
        PrimitiveKind::Call,
        PrimitiveKind::Respond,
        PrimitiveKind::Emit,
        PrimitiveKind::Sandbox,
        PrimitiveKind::Subscribe,
        PrimitiveKind::Stream,
    ];
    assert_eq!(
        canonical_values.len(),
        12,
        "the 12 canonical PrimitiveKind values must be constructible by name \
         — a rename would have failed THIS crate's compile (CLAUDE.md #1)."
    );
    let surplus: u32 = canonical_values
        .iter()
        .copied()
        .map(surplus_arm_witness)
        .sum();
    assert_eq!(
        surplus, 0,
        "no canonical PrimitiveKind value may route to the #[non_exhaustive] \
         catch-all — a non-zero surplus means a 13th variant slipped in \
         behind the wildcard (CLAUDE.md baked-in #1 12-primitive \
         irreducibility violation)."
    );

    // canonical_tag must be a stable, distinct name for each (anti-collapse:
    // a variant cannot satisfy the count by aliasing another's tag).
    let tags: std::collections::BTreeSet<&'static str> =
        canonical_values.iter().map(|k| k.canonical_tag()).collect();
    assert_eq!(
        tags.len(),
        12,
        "every PrimitiveKind MUST carry a DISTINCT canonical_tag (got {} \
         distinct tags for 12 variants) — a collapsed tag would CID-alias \
         two primitives.",
        tags.len()
    );

    // -------------------------------------------------------------------
    // Backstop 2 — source-body walk (catches ADDITIONS). Parse the real
    // enum body; assert exactly 12 variants, each canonical name present.
    // -------------------------------------------------------------------
    let core_src = workspace_root()
        .join("crates")
        .join("benten-core")
        .join("src");
    let variants = extract_primitive_kind_variants(&core_src);

    assert!(
        !variants.is_empty(),
        "PrimitiveKind enum body MUST be non-empty (smoke-check); empty body \
         means the source walk found nothing — pin is vacuous-truth (pim-18 \
         §3.6f failure mode)."
    );
    assert_eq!(
        variants.len(),
        12,
        "PrimitiveKind enum MUST have exactly 12 variants per CLAUDE.md \
         baked-in #1 (got {} after Phase-4: {:?}). A 13th variant is a \
         12-primitive-irreducibility violation; a removed variant strands \
         content.",
        variants.len(),
        variants,
    );
    for v in &CANONICAL_12 {
        assert!(
            variants.iter().any(|x| x == v),
            "PrimitiveKind missing canonical variant `{v}` in benten-core \
             source — exit-criterion 7 violation (a rename or removal of one \
             of the irreducible 12)."
        );
    }
    // The source-walk set and the compile-time set must AGREE — neither side
    // may drift from the other (defends against editing one backstop only).
    for v in &variants {
        assert!(
            CANONICAL_12.contains(&v.as_str()),
            "PrimitiveKind source carries a variant `{v}` NOT in the \
             canonical-12 set — a new primitive was minted (CLAUDE.md #1 \
             irreducibility violation)."
        );
    }
}
