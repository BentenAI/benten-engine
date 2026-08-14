//! GAP-KDB Shape-B — RK-8 `recipient_key_generation` ⟺ committed-key-set
//! precedence (C8), id-side. W0-canary RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` C8 + `R2-LANDSCAPE` RK-8 + §5
//! **S2** (this is the plan-acknowledged UNDER-SPECIFIED item).
//!
//! # ✅ S2 RESOLVED (D-47) — the concrete disagreement arm is DROP-5 (benten-drop W2)
//! S2 (the C8 `recipient_key_generation` ⟺ committed-key-set precedence
//! semantics — "what counts as disagreement, and the exact typed reject") is
//! RESOLVED per D-47. This file pins the **id-side role separation** (below);
//! the concrete disagreement `expect_err` — a specific generation index that
//! must fail-closed against the committed key-set — is homed at the Drop
//! layer as **DROP-5 (benten-drop, W2)**, where the
//! `recipient_key_generation` AAD field actually lives, and un-ignores in
//! that wave. `resolve_kem` on the id side takes NO generation index, so the
//! id-side role-separation property is fully pinned here and independent of
//! the Drop-layer disagreement arm.
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
//! # R5 (role-separation arm — non-ignored, id side); DROP-5 lands in benten-drop W2.
//! `Did::resolve_kem` is pure over the commitment (takes no generation
//! index); the role-separation arm below is non-ignored. The concrete
//! disagreement arm is DROP-5 (benten-drop W2), now that S2 is resolved (D-47).

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

// NOTE (S2 RESOLVED, D-47): the concrete `recipient_key_generation`-vs-
// committed-key-set DISAGREEMENT arm (a specific generation index that must
// fail-closed against the committed key-set, with the precise typed reject)
// is homed at the Drop layer as DROP-5 (benten-drop, W2) — the
// `recipient_key_generation` AAD field lives there, not on the pure id-side
// `resolve_kem`. This id-side file pins the role separation only.
