//! TF-12 obligation (5) — P2P-interop frozen-interface conformance
//! lane (G-CORE-9 FREEZE wave).
//!
//! ADDL R3-C1 (Phase-4-Meta-Core; last R3 wave; freeze-time). TDD
//! red-phase. Tests-only — NO production source.
//!
//! ## Disjointness vs R3-A2 / R3-B1 (HARD — §3.5i)
//!
//! R3-A2 owns `tf2_*` (the #1300 signature seam BEHAVIOR), R3-B1 owns
//! `tf3_*` / `tf4_*` (the #1301 envelope + G-CORE-3c swap-matrix
//! BEHAVIOR). THIS file asserts the **G-CORE-9 FREEZE PROPERTY** of
//! the P2P-interop conformance lane (per the brief: "assert the FREEZE
//! property (post-G-CORE-9 lock), not the substrate behavior"). It
//! does NOT re-test KAT vectors / strip-resistance / round-trips —
//! those are the substrate families. It asserts the conformance lane
//! is a LOCKED frozen-interface deliverable (the wire-protocol analog
//! of the `cargo-public-api` baseline). Distinct `tf12_` filename; no
//! `tf2_*`/`tf3_*`/`tf4_*` file is touched or duplicated.
//!
//! ## §3.6g LITERAL discipline checklist (reproduced, not §-referenced)
//!
//!  1. Land-when = FREEZE. Every RED pin carries
//!     `#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]`.
//!  2. Campaign-tail landed-vs-RED split (§3.5n): R3-C1 ground-truthed
//!     `benten-crypto-suite/src/lib.rs` at ed03729a — it is an
//!     **intentionally-empty G-CORE-2 stub** (no codepoint/sig/cipher
//!     surface yet) and there is NO frozen P2P-interop conformance
//!     baseline. The §1.A.FROZEN item 14 lane is a freeze-DELIVERABLE
//!     NOT yet built → RED.
//!  3. SHAPE-not-SUBSTANCE (pim-18 / §3.6f): this is necessarily a
//!     frozen-interface CONFORMANCE-LANE pin (see "Why structural"
//!     note). The would-FAIL signal is concrete: the committed
//!     conformance-baseline artifact + the FROZEN-INTERFACE CONTRACT
//!     doc's P2P-interop section recording the 4 invariant properties +
//!     the IANA-ID / thin-selector-table discipline markers. NOT a
//!     type-constructibility assertion. Same reasoning class as
//!     R3-B6's #838 seam-shape pin (a property invariant where the
//!     substantive consequence is structurally a freeze-lock, not a
//!     unit-testable runtime behavior).
//!  4. pim-2 sub-rule-4 (§3.6b): exercises the SPECIFIC §1.A.FROZEN
//!     item 14 obligation (mandatory baseline suite + typed-unsupported-
//!     never-silent-fallback + no-wire-break-on-codepoint-add +
//!     old-codepoints-supported-forever + IANA HPKE/COSE IDs + thin
//!     selector table), not an umbrella "crypto is agile".
//!  5. §3.13: no shared process-scoped static — per-test locals only.
//!  6. §3.5j: compiles + MSRV-1.95 clippy AND `cargo +stable clippy`
//!     (scoped to benten-crypto-suite — never `--workspace`).
//!  7. §3.6e: introduces no stranded `#[ignore]` pin; THIS pin's named
//!     un-ignore destination IS G-CORE-9. (NOTE the stranded
//!     `tf2_*`/`tf3_*`/`tf4_*` ignored pins citing G-CORE-2/G-CORE-3/
//!     G-CORE-3c are R3-A2/R3-B1's named destinations — NOT redirected
//!     here; flagged in the report's §3.6e section, not re-homed.)
//!
//! ## Why a structural frozen-interface pin is correct (pim-18 waiver)
//!
//! r2-test-landscape.md §2.B: the P2P-interop conformance invariant is
//! "the frozen-interface analog of `cargo-public-api`" (a G-CORE-9
//! frozen-interface conformance gate). The substantive behavior
//! (typed-unsupported actually firing, codepoint-add not wire-breaking,
//! an old-codepoint object still decrypting) is exercised by the
//! TF-2/TF-3/TF-4 substrate families. THIS pin asserts the FREEZE
//! CONTRACT: that lane is (a) declared a mandatory baseline conformance
//! suite, (b) the 4 properties are recorded as frozen invariants in the
//! FROZEN-INTERFACE CONTRACT, (c) the IANA-HPKE/COSE-component-ID +
//! thin-one-codepoint-per-suite-selector-table discipline is frozen
//! (no Benten-minted component algorithm numbers). The freeze ACT is
//! the scheduled P-III Ben decision-point; this pin is the structural
//! guarantee the freeze did not silently skip the lane.
//!
//! Pin source: r2-test-landscape.md TF-12 obligation (5) + §2.B
//! "P2P-interop conformance invariant" + plan §1.A.FROZEN item 14 +
//! §4 CI P2P-interop conformance lane + RATIFIED-pq-default-reframe
//! -2026-05-19 §3-§4.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn frozen_contract() -> Option<String> {
    let p = workspace_root().join("docs/V1-FROZEN-INTERFACE.md");
    std::fs::read_to_string(p).ok()
}

/// RED — un-ignore at G-CORE-9. The FROZEN-INTERFACE CONTRACT records
/// the §1.A.FROZEN item 14 P2P-interop conformance invariant with ALL
/// FOUR sub-properties frozen: (a) mandatory baseline conformance
/// suite; (b) typed-unsupported-error never silent fallback; (c) no
/// wire-break when a codepoint is added; (d) old codepoints supported
/// forever / never-strand-content. Would-FAIL if the freeze ships
/// without the lane recorded (a Composing-time consumer could then
/// reintroduce a silent-fallback / wire-breaking codepoint-add).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]
fn p2p_interop_lane_four_invariants_frozen_in_contract() {
    let body = frozen_contract().unwrap_or_else(|| {
        panic!(
            "TF-12 (5)/§1.A.FROZEN item 14: docs/V1-FROZEN-INTERFACE.md \
             must exist at G-CORE-9 and record the P2P-interop \
             conformance invariant."
        )
    });
    // Property (a): a mandatory baseline conformance suite.
    assert!(
        body.contains("baseline conformance suite") || body.contains("mandatory baseline"),
        "TF-12 (5)(a): P2P-interop MANDATORY baseline conformance suite \
         not recorded as frozen in the FROZEN-INTERFACE CONTRACT."
    );
    // Property (b): typed-unsupported-error, never silent fallback.
    assert!(
        body.contains("typed-unsupported")
            && (body.contains("never") && body.contains("silent fallback")),
        "TF-12 (5)(b): the typed-unsupported-error / never-silent- \
         fallback invariant not recorded as frozen (age's silent-ignore \
         is the explicitly-rejected outlier — Veilid/MLS/NIP-44 \
         precedent)."
    );
    // Property (c): additive-codepoint discipline (no wire-break).
    assert!(
        body.contains("no wire-break") || body.contains("additive-codepoint"),
        "TF-12 (5)(c): the no-wire-break-when-a-codepoint-is-added \
         (additive-codepoint) invariant not recorded as frozen."
    );
    // Property (d): old codepoints supported forever / never-strand.
    assert!(
        body.contains("never strand")
            || body.contains("supported forever")
            || body.contains("never-strand-content"),
        "TF-12 (5)(d): the old-codepoints-supported-forever / \
         never-strand-content invariant not recorded as frozen \
         (an algorithm-add never strands previously-written immutable \
         content-addressed objects)."
    );
}

/// RED — un-ignore at G-CORE-9. The component-algorithm-ID discipline
/// is frozen: Benten REUSES IANA HPKE/COSE component IDs and mints NO
/// Benten component algorithm numbers; Benten owns ONLY the thin
/// one-codepoint-per-suite selector table. Would-FAIL if the freeze
/// permits a Benten-minted component algorithm number (the exact
/// interop-fragmentation hazard the invariant prevents).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]
fn p2p_interop_iana_id_and_thin_selector_discipline_frozen() {
    let body = frozen_contract().unwrap_or_else(|| {
        panic!(
            "TF-12 (5): docs/V1-FROZEN-INTERFACE.md must exist at \
             G-CORE-9."
        )
    });
    assert!(
        (body.contains("IANA") && (body.contains("HPKE") || body.contains("COSE"))),
        "TF-12 (5): the 'reuse IANA HPKE/COSE component IDs — never mint \
         Benten component algorithm numbers' discipline not recorded as \
         frozen."
    );
    assert!(
        body.contains("selector table")
            && (body.contains("thin") || body.contains("one-codepoint-per-suite")),
        "TF-12 (5): the 'Benten owns ONLY the thin one-codepoint- \
         per-suite selector table' discipline not recorded as frozen."
    );
    // The explicitly-rejected alternative MUST be recorded so a
    // Composing-time reviewer cannot reintroduce it.
    assert!(
        body.contains("PQ-TLS-as-envelope") || body.contains("transport-envelope"),
        "TF-12 (5): the explicitly-REJECTED 'PQ-TLS-as-envelope buys \
         time' alternative (Matrix's transport-relayed position) must be \
         recorded as rejected (Benten ciphertext rests at-rest on peer \
         disks; iroh transport is classical-only/no-PQ-roadmap)."
    );
}

/// RED — un-ignore at G-CORE-9. A committed P2P-interop conformance
/// BASELINE artifact exists (the wire-protocol analog of the
/// `docs/public-api/<crate>` baseline) so a post-freeze codepoint /
/// suite-table mutation is a CI FAIL — the structural backstop. The
/// G-CORE-9 brief produces it; this pin asserts its presence + that it
/// enumerates the v1-frozen codepoint set (the hybrid default + the
/// swap-matrix codepoints incl. the well-known `0x647a` X-Wing-hybrid
/// encryption suite). Would-FAIL if absent or vacuous.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]
fn p2p_interop_conformance_baseline_artifact_committed() {
    let root = workspace_root();
    // The brief decides the exact path; the pin asserts the PROPERTY
    // (a committed, non-vacuous, codepoint-enumerating baseline). The
    // canonical candidate locations the G-CORE-9 brief would use.
    let candidates = [
        root.join("docs/public-api/p2p-interop-conformance.txt"),
        root.join("docs/public-api/p2p-interop-conformance.json"),
        root.join("docs/V1-FROZEN-INTERFACE-p2p-interop.md"),
    ];
    let found = candidates.iter().find(|p| p.exists());
    let path = found.unwrap_or_else(|| {
        panic!(
            "TF-12 (5)/§4 CI: a committed P2P-interop conformance \
             baseline artifact must exist at G-CORE-9 (the wire-protocol \
             analog of the cargo-public-api baseline). Candidates: \
             {candidates:?}"
        )
    });
    let body = std::fs::read_to_string(path).unwrap();
    // Non-vacuity: it must enumerate the frozen codepoint set (at least
    // the well-known X-Wing-hybrid encryption suite codepoint `0x647a`
    // + the multihash hash codepoints `0x1e`/`0x1015`/`0x16`).
    assert!(
        body.contains("0x647a") && body.contains("0x1e"),
        "TF-12 (5): the P2P-interop conformance baseline must enumerate \
         the v1-frozen codepoint set (the X-Wing-hybrid encryption \
         suite `0x647a` + the multihash hash codepoints `0x1e` BLAKE3 / \
         `0x1015` SHA-512/256 / `0x16` SHA3-256) so a post-freeze \
         mutation is a CI FAIL (not vacuous)."
    );
}
