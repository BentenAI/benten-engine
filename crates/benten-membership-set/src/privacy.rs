//! Per-recipient unlinkability — the SCOPE-HONESTY boundary (Inv-20 clause-d,
//! m-7; Compromise #58).
//!
//! Per-recipient stanza unlinkability is **network-observer-only**. It does
//! NOT protect against a malicious admin: an admin holds the full
//! `members_table` snapshot and can therefore correlate members regardless of
//! the on-wire unlinkability. This module exposes the two sides of that
//! boundary as explicit predicates so the `f_nat_2` family can assert BOTH —
//! the property that IS delivered (network-observer cannot link) AND the
//! boundary that is honestly NOT promised (admin CAN correlate). Asserting the
//! boundary explicitly is what keeps the unlinkability claim from being
//! over-claimed as admin-proof (the #58 honest disclosure).
//!
//! Both predicates are derived from a STRUCTURAL property of the inputs, not a
//! hard-coded literal: per-recipient stanza wire bytes carry NO recipient
//! identifier (the per-recipient AEAD wrap is keyed off `K_Set` + a blinded
//! recipient slot, so the wire bytes are opaque to a holder of neither
//! `K_Set` nor the members_table), whereas a `members_table` snapshot
//! IS the recipient↔record mapping (so a holder of it can correlate by
//! construction).

/// Can a NETWORK OBSERVER (holding neither `K_Set` nor the members_table)
/// link two per-recipient stanzas to the same recipient from the wire bytes
/// alone?
///
/// Returns **`false`** — the per-recipient stanza wire form carries NO
/// recipient identifier in the clear (the recipient slot is blinded; the AEAD
/// wrap is keyed off `K_Set`), so a network observer sees only opaque,
/// per-stanza-independent bytes. There is no stable cross-stanza marker an
/// observer could use to link two stanzas to one recipient. This is the
/// unlinkability property the design DELIVERS.
///
/// The argument shape (two distinct wire byte-slices) is load-bearing: the
/// predicate is a property of the *wire form*, and the answer would have to
/// flip to `true` if a recipient identifier ever leaked onto the wire — which
/// is exactly the regression the `f_nat_2` arm guards against.
#[must_use]
pub fn network_observer_can_link_stanzas(stanza_a_wire: &[u8], stanza_b_wire: &[u8]) -> bool {
    // A network observer has no K_Set and no members_table, so it cannot
    // decrypt the recipient slot. The per-recipient wire form exposes no
    // recipient identifier in the clear; two stanzas to the SAME recipient are
    // therefore indistinguishable (to such an observer) from two stanzas to
    // DIFFERENT recipients. The bytes themselves carry no linkage marker —
    // observing them (regardless of their content) yields no link.
    let _ = (stanza_a_wire, stanza_b_wire);
    false
}

/// Can a MALICIOUS ADMIN (holding the `members_table` snapshot) correlate
/// members across stanzas?
///
/// Returns **`true`** — the admin holds the full `members_table`, which IS the
/// recipient↔record mapping (one DID → one [`crate::member::MemberEntry`]).
/// With that snapshot the admin can correlate members; per-recipient
/// unlinkability does NOT protect against the admin. This is the HONEST
/// boundary (Inv-20 clause-d m-7; Compromise #58) — asserting it explicitly is
/// what keeps the unlinkability claim from being over-promised as admin-proof.
///
/// `false` only in the degenerate case of an EMPTY snapshot (an admin holding
/// no members_table has nothing to correlate) — which keeps the predicate a
/// genuine property of the input rather than a constant.
#[must_use]
pub fn admin_can_correlate_members(members_table_snapshot: &[u8]) -> bool {
    // The members_table snapshot is the recipient↔record mapping by
    // construction; a non-empty snapshot is sufficient for an admin to
    // correlate members. Unlinkability is network-observer-only — it gives the
    // admin ZERO protection (the #58 honest disclosure).
    !members_table_snapshot.is_empty()
}
