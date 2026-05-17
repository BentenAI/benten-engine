//! Inv-14 / Inv-13 string-mirror drift-defense pin.
//!
//! Refinement-audit-2026-05 #1160 (Safe-3 #597 + #601). Two cross-crate
//! wire-string contracts had been declared independently in multiple
//! crates, so a future edit to one declaration could silently drift the
//! on-the-wire string apart from the other consumers:
//!
//! - #597: `ATTRIBUTION_PROPERTY_KEY` was declared independently in
//!   `benten-core::subgraph` AND `benten-eval::invariants::attribution`.
//!   Fixed by making `benten-eval` re-export the `benten-core` constant
//!   (single source of truth).
//! - #601: `LABEL_CURRENT` / `LABEL_NEXT_VERSION` were duplicated as bare
//!   string literals at the `benten-ivm` `version_current` view consumers
//!   rather than referencing the `benten-core` constants. Fixed by routing
//!   the consumers through `benten_core::LABEL_NEXT_VERSION`.
//!
//! This pin asserts the *values* of the single-source-of-truth constants.
//! It would FAIL if any future edit changed a contract string in
//! `benten-core` without the deliberate intent of changing the wire
//! contract (which would also have to update this pin). Combined with the
//! `benten-eval` re-export (compile-time SSoT), this guards both ends:
//! the re-export guarantees `benten-eval` cannot independently redeclare,
//! and this pin freezes the canonical value so an intentional benten-core
//! edit is a visible, reviewed change.

use benten_core::{ATTRIBUTION_PROPERTY_KEY, LABEL_CURRENT, LABEL_NEXT_VERSION};

#[test]
fn attribution_property_key_wire_string_is_frozen() {
    // Inv-14: the attribution-declaration property key. Changing this
    // value is an on-the-wire / canonical-bytes break — it must be a
    // deliberate, reviewed edit that also updates this pin.
    assert_eq!(
        ATTRIBUTION_PROPERTY_KEY, "attribution",
        "Inv-14 attribution property key drifted from the frozen wire \
         contract; if this is intentional, update this pin AND audit every \
         producer/consumer (benten-core SubgraphBuilder stamp + \
         benten-eval registration validator)"
    );
}

#[test]
fn version_edge_label_wire_strings_are_frozen() {
    // Inv-13: the version-chain edge-label contract. The benten-ivm
    // version_current view matches on `LABEL_NEXT_VERSION`; the anchor →
    // current pointer uses `LABEL_CURRENT`. Both are wire-compatible
    // edge-label contracts — drift here silently breaks IVM view matching
    // against any peer or persisted graph encoded with the old label.
    assert_eq!(
        LABEL_CURRENT, "CURRENT",
        "Inv-13 LABEL_CURRENT drifted from the frozen edge-label contract"
    );
    assert_eq!(
        LABEL_NEXT_VERSION, "NEXT_VERSION",
        "Inv-13 LABEL_NEXT_VERSION drifted from the frozen edge-label \
         contract; the benten-ivm version_current view + algorithm_b \
         canonical-label table both key off this constant"
    );
}
