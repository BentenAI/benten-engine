//! GAP-KDB Shape-B — **DOCS-5** doc-conformance catch-net (R3 wave
//! **W4-docs-invariants**). Gate **C** (guards freeze-permanent values).
//!
//! Anchors: design `GAP-KDB-B-DESIGN-R1.md` §1.1/§1.2/§5 (the KEM multikey
//! wires the registered components `mlkem-768-pub = 0x120c` + `x25519-pub
//! = 0xec`, X25519-first per C2; the new `did:benten` method; retires the
//! #5-risky private `HYBRID_KEM_MULTICODEC = 0xf0`). R2 landscape §1
//! **DOCS-5**:
//!   "`CRYPTO-CODEPOINTS` `0x120c`/`0xec` **WIRED** + `did:benten` method
//!    **registered** + `0xf0` **retired** (guards freeze-perm values)."
//!
//! At the R3 freeze base `CRYPTO-CODEPOINTS.md` LISTS `0x120c`/`0xec` as
//! registered COMPONENT codes but describes the hybrid-KEM form as used
//! "when wired", still names `HYBRID_KEM_MULTICODEC = [0xf0, 0x01]` as the
//! active fallback-interim, and does NOT register `did:benten`. The R5
//! doc-wave: (1) WIRES `0x120c`/`0xec` as the KeySetDocument `kem` /
//! `did:benten` form; (2) registers the `did:benten` method; (3) RETIRES
//! `0xf0`.
//!
//! # RED-PHASE discipline
//! BASELINE (NOT ignored) drives the REAL on-disk parser and recovers a
//! live codepoint (proving it reads the doc). RED arms
//! (`#[ignore = "RED-PHASE: DOCS-5 … un-ignore at R5"]`) assert the three
//! WIRED/registered/retired end-states. would-FAIL-on-revert: reverting
//! the wiring / registration / retirement flips each RED arm.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve from crates/benten-drop")
}

fn crypto_codepoints() -> String {
    std::fs::read_to_string(repo_root().join("docs/CRYPTO-CODEPOINTS.md"))
        .expect("CRYPTO-CODEPOINTS.md must be present")
}

/// A codepoint is "registered on one row" only when its value is
/// CO-LOCATED with a binding token on a single line (the §4.0
/// allocation-table shape the `f_disc_2` catch-net uses). Guards against a
/// bare prose mention.
fn registered_on_one_row(doc: &str, value: &str, binding: &str) -> bool {
    doc.lines()
        .any(|l| l.contains(value) && l.contains(binding))
}

// ===========================================================================
// BASELINE arm (NOT ignored) — drive the REAL codepoint-doc parser.
// ===========================================================================

/// DOCS-5 BASELINE — the parser recovers live, already-registered
/// codepoints from the on-disk `CRYPTO-CODEPOINTS.md`: the frozen hybrid
/// cipher-suite `0x647a` and the registered component `0x1211`
/// (`mldsa-65-pub`). Proves the parser reads the doc, not a literal; an
/// inert read would make every RED wiring/registration arm vacuously pass.
#[test]
fn docs_5_codepoint_doc_recovers_live_codepoints_baseline() {
    let doc = crypto_codepoints();
    assert!(
        doc.contains("0x647a") || doc.contains("0x647A"),
        "CRYPTO-CODEPOINTS.md MUST carry the frozen hybrid cipher-suite \
         codepoint 0x647a (HYBRID_X25519_MLKEM768) at the R3 base."
    );
    assert!(
        registered_on_one_row(&doc, "0x1211", "mldsa"),
        "CRYPTO-CODEPOINTS.md MUST register the ML-DSA-65 component code \
         0x1211 bound to `mldsa-65-pub` (the signing multikey component the \
         did:benten layout reuses)."
    );
    // Post-R5 (GAP-KDB doc-wave): the did:benten method registration landed —
    // the sanity guard flips from "absent at R3 base" to "now registered", a
    // live post-condition co-pinning the R5 wiring alongside the RED arms.
    assert!(
        doc.contains("did:benten"),
        "post-R5: CRYPTO-CODEPOINTS now registers the did:benten method (the R5 \
         doc-wave WIRED 0x120c/0xec + registered did:benten + retired 0xf0)."
    );
}

// ===========================================================================
// RED arms (ignored until R5) — WIRED + registered + retired end-states.
// ===========================================================================

/// DOCS-5 RED — the registered KEM components `0x120c` (`mlkem-768-pub`) +
/// `0xec` (`x25519-pub`) are WIRED into the GAP-KDB key-set surface: the
/// doc binds them to the `KeySetDocument` `kem` field / `did:benten`
/// (X25519-first per C2), no longer the "when wired" hedge. Both component
/// codes MUST be co-located with the key-set/did:benten wiring.
///
/// would-FAIL-on-revert: reverting the wiring back to the
/// "when-wired"/`0xf0` interim removes the KeySetDocument/did:benten
/// binding of 0x120c/0xec.
#[test]
fn docs_5_wires_0x120c_0xec_into_keyset() {
    let doc = crypto_codepoints();
    let wired = |value: &str| {
        doc.lines().any(|l| {
            l.contains(value)
                && (l.contains("KeySetDocument")
                    || l.contains("key-set")
                    || l.contains("keyset")
                    || l.contains("did:benten")
                    || l.contains("WIRED")
                    || l.contains("wired"))
        })
    };
    assert!(
        wired("0x120c"),
        "CRYPTO-CODEPOINTS.md MUST WIRE 0x120c (mlkem-768-pub) into the \
         GAP-KDB key-set surface (KeySetDocument `kem` / did:benten), not the \
         `when wired`/0xf0 interim (design §5, C2)."
    );
    assert!(
        wired("0xec"),
        "CRYPTO-CODEPOINTS.md MUST WIRE 0xec (x25519-pub) into the GAP-KDB \
         key-set surface (X25519-first per C2)."
    );
}

/// DOCS-5 RED — the `did:benten` method is REGISTERED in the codepoint doc
/// (its method-specific-id byte layout + key-set commitment). Absent at
/// the R3 base (0 hits). would-FAIL-on-revert: dropping the did:benten
/// registration removes the mention.
#[test]
fn docs_5_registers_did_benten_method() {
    let doc = crypto_codepoints();
    assert!(
        doc.contains("did:benten"),
        "CRYPTO-CODEPOINTS.md MUST register the did:benten method (the \
         content-addressed key-set DID; design §1.1). R5 doc-wave adds it."
    );
    // Registration binds the method to its key-set commitment, not a bare
    // mention.
    let bound = doc.lines().any(|l| {
        l.contains("did:benten")
            && (l.contains("key-set")
                || l.contains("keyset")
                || l.contains("KeySetDocument")
                || l.contains("commit")
                || l.contains("CID"))
    });
    assert!(
        bound,
        "the did:benten registration MUST bind the method to its key-set \
         commitment (the KEM key is committed by the DID; design §1.1/§5)."
    );
}

/// DOCS-5 RED — the #5-risky private `HYBRID_KEM_MULTICODEC = 0xf0` is
/// RETIRED in favor of the registered `0x120c`/`0xec` components. The doc
/// marks `0xf0` retired/superseded (co-located with the retirement token).
/// Absent at the R3 base (no "retired" language). would-FAIL-on-revert:
/// removing the retirement note (0xf0 silently stays live) flips the arm.
#[test]
fn docs_5_retires_0xf0_hybrid_kem_multicodec() {
    let doc = crypto_codepoints();
    let retired = doc.lines().any(|l| {
        (l.contains("0xf0") || l.contains("HYBRID_KEM_MULTICODEC"))
            && (l.contains("retired")
                || l.contains("RETIRE")
                || l.contains("Retired")
                || l.contains("superseded")
                || l.contains("no longer")
                || l.contains("removed"))
    });
    assert!(
        retired,
        "CRYPTO-CODEPOINTS.md MUST mark HYBRID_KEM_MULTICODEC = 0xf0 RETIRED \
         (superseded by the registered 0x120c/0xec components; design §5). \
         A silent leave-in fails the freeze-value guard."
    );
}
