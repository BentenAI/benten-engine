//! GAP-KDB Shape-B — **DOCS-6** doc-conformance catch-net (R3 wave
//! **W4-docs-invariants**). Gate **C** (freeze-completeness).
//!
//! Anchors: design `GAP-KDB-B-DESIGN-R1.md` §7 / R1 §4 FREEZE-NOW items
//! (the `did:benten` method byte layout + `KeySetDocument` v=1 DAG-CBOR
//! schema are FROZEN v1-beta wire). R2 landscape §1 **DOCS-6**:
//!   "`V1-WIRE-FORMAT-INVENTORY` registers `did:benten` + `KeySetDocument`
//!    surfaces (freeze-completeness)."
//!
//! `docs/V1-WIRE-FORMAT-INVENTORY.md` is the freeze-completeness registry:
//! every wire-format-bearing surface gets a numbered row + a byte-pin test
//! citation. At the R3 base it inventories 28 surfaces (Node/Edge CBOR …
//! Layer-A vault) but NOT the two new GAP-KDB freeze surfaces
//! (`did:benten`, `KeySetDocument`) — 0 hits each. The R5 doc-wave adds
//! their rows so the freeze is complete.
//!
//! # RED-PHASE discipline
//! BASELINE (NOT ignored) drives the REAL on-disk parser and recovers
//! existing inventory rows (proving it reads the doc). RED arms
//! (`#[ignore = "RED-PHASE: DOCS-6 … un-ignore at R5"]`) assert the two new
//! surfaces are registered WITH a byte-pin citation. would-FAIL-on-revert:
//! dropping the did:benten / KeySetDocument rows flips each RED arm.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve from crates/benten-drop")
}

fn wire_inventory() -> String {
    std::fs::read_to_string(repo_root().join("docs/V1-WIRE-FORMAT-INVENTORY.md"))
        .expect("V1-WIRE-FORMAT-INVENTORY.md must be present")
}

// ===========================================================================
// BASELINE arm (NOT ignored) — drive the REAL inventory parser.
// ===========================================================================

/// DOCS-6 BASELINE — the parser recovers existing inventory surfaces from
/// the on-disk `V1-WIRE-FORMAT-INVENTORY.md` (UCAN-Varsig header,
/// RotationLog/RotationAttestation, DeviceAttestation, Layer-C drop band).
/// Proves the parser reads the doc; an inert read would make the RED
/// registration arms vacuously pass.
#[test]
fn docs_6_inventory_recovers_existing_surfaces_baseline() {
    let doc = wire_inventory();
    for surface in [
        "UCAN-Varsig v1 header",
        "RotationAttestation",
        "DeviceAttestation",
        "Layer-C drop band",
    ] {
        assert!(
            doc.contains(surface),
            "V1-WIRE-FORMAT-INVENTORY.md MUST inventory the existing surface \
             {surface:?} at the R3 base (proves the parser reads the doc)."
        );
    }
    // Sanity: at the R3 base neither GAP-KDB surface is registered yet.
    assert!(
        !doc.contains("did:benten") && !doc.contains("KeySetDocument"),
        "sanity: at the R3 freeze base the inventory does not yet register \
         did:benten / KeySetDocument; if this fires, re-base the RED arms."
    );
}

// ===========================================================================
// RED arms (ignored until R5) — the two new freeze surfaces registered.
// ===========================================================================

/// DOCS-6 RED — `V1-WIRE-FORMAT-INVENTORY.md` registers the `did:benten`
/// method-specific-id byte layout as a frozen surface WITH a byte-pin test
/// citation (the `.rs` file that pins its golden layout). would-FAIL-on-
/// revert: dropping the row (or citing no byte-pin test) fails.
#[test]
#[ignore = "RED-PHASE: DOCS-6 did:benten inventory row — lands at R5 doc-wave — un-ignore at R5"]
fn docs_6_registers_did_benten_surface() {
    let doc = wire_inventory();
    assert!(
        doc.contains("did:benten"),
        "V1-WIRE-FORMAT-INVENTORY.md MUST register the did:benten method \
         byte layout as a frozen v1-beta wire surface (design §7)."
    );
    // Freeze-completeness = the row cites a byte-pin test (a `.rs` file),
    // matching every other inventory row's shape.
    let cites_byte_pin = doc.lines().any(|l| l.contains("did:benten") && l.contains(".rs"));
    assert!(
        cites_byte_pin,
        "the did:benten inventory row MUST cite a byte-pin test (`.rs`) — the \
         golden-layout pin — matching the inventory's per-surface shape."
    );
}

/// DOCS-6 RED — `V1-WIRE-FORMAT-INVENTORY.md` registers the
/// `KeySetDocument` v=1 canonical DAG-CBOR schema as a frozen surface WITH
/// a byte-pin test citation. would-FAIL-on-revert: dropping the row (or
/// citing no byte-pin test) fails.
#[test]
#[ignore = "RED-PHASE: DOCS-6 KeySetDocument inventory row — lands at R5 doc-wave — un-ignore at R5"]
fn docs_6_registers_keyset_document_surface() {
    let doc = wire_inventory();
    assert!(
        doc.contains("KeySetDocument"),
        "V1-WIRE-FORMAT-INVENTORY.md MUST register the KeySetDocument v=1 \
         canonical DAG-CBOR schema as a frozen v1-beta wire surface (design §7)."
    );
    let cites_byte_pin = doc.lines().any(|l| l.contains("KeySetDocument") && l.contains(".rs"));
    assert!(
        cites_byte_pin,
        "the KeySetDocument inventory row MUST cite a byte-pin test (`.rs`) — \
         the golden-byte + golden-CID pin — matching the inventory's shape."
    );
}
