//! GAP-KDB Shape-B — RK-8 `recipient_key_generation` ⟺ committed-key-set
//! precedence (C8), id-side. W0-canary RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` C8 + `R2-LANDSCAPE` RK-8 + §5
//! **S2** (this is the plan-acknowledged UNDER-SPECIFIED item).
//!
//! # ⚠️ S2 HOLD — the exact disagreement arm is DESIGN-GATED
//! Per R2-LANDSCAPE §5 S2: the *precise* "what counts as disagreement, and
//! the exact typed reject" for `recipient_key_generation` vs the committed
//! key-set is under-specified and needs a design nail BEFORE its red test
//! can assert a concrete `expect_err`. This file pins ONLY what is
//! decidable now — the **role separation** and the **fail-closed-on-
//! disagreement direction** — and explicitly HOLDS the concrete
//! disagreement `expect_err` (it lives at the Drop layer, DROP-5, once S2
//! lands). Do NOT block the W2 flagships on this file.
//!
//! # Role separation pinned now (C8)
//! - The **committed key-set** (the DID + its key-set doc) identifies WHICH
//!   KEM keypair. On the id side, `resolve_kem` is a PURE function of `(DID,
//!   doc)` — it takes NO `recipient_key_generation`; the recovered key is
//!   fully determined by the commitment (deterministic across calls).
//! - `recipient_key_generation` stays the intra-keypair freshness / nonce-
//!   cache index (a Layer-C AAD field), NOT a key selector — so it cannot
//!   silently redirect which key is used.
//!
//! # would_fail_on_revert
//! If resolve_kem became non-deterministic over the committed key-set (e.g.
//! a generation index leaked into key selection on the id side), the
//! determinism assert flips.
//!
//! # R5 un-ignore (role-separation arm only); DROP-5 un-ignore after S2.
//! Mint `Did::resolve_kem` (pure over the commitment); drop `#[ignore]` on
//! the role-separation arm. The concrete disagreement arm stays HELD until
//! the S2 design decision, then lands as DROP-5.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_id::kdb_testing as kdb;

#[test]
fn rk8_resolve_kem_is_pure_over_the_committed_key_set() {
    // The committed key-set fully determines the KEM key: resolving the
    // same (DID, doc) twice yields byte-identical keys. No freshness index
    // participates in id-side key selection (C8 role separation).
    let (did, doc) = kdb::honest_recipient_scenario();
    let first = kdb::resolve_kem(&did, &doc).expect("committed key-set resolves");
    let second = kdb::resolve_kem(&did, &doc).expect("committed key-set resolves again");
    assert_eq!(
        first.to_bytes(),
        second.to_bytes(),
        "RK-8: resolve_kem MUST be a pure function of the committed key-set \
         (the committed key-set = which keypair; NOT a generation index)"
    );
    assert_eq!(first.codepoint().raw(), second.codepoint().raw());
}

// NOTE (S2 HOLD): the concrete `recipient_key_generation`-vs-committed-
// key-set DISAGREEMENT arm (a specific generation index that must
// fail-closed against the committed key-set, with the precise typed
// reject) is intentionally NOT written here — it is design-gated (R2 §5
// S2) and lands as DROP-5 at the Drop layer once the exact semantics are
// nailed. Writing a concrete `expect_err` before that decision would pin
// an arbitrary interpretation.
