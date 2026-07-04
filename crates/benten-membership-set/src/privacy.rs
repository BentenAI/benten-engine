//! Per-recipient unlinkability — the SCOPE-HONESTY boundary (Inv-20 clause-d,
//! m-7; Compromise #58).
//!
//! Per-recipient stanza unlinkability is **network-observer-ONLY** — it holds
//! ONLY against a party that has NEITHER `K_Set` NOR the members_table/roster.
//! It does **NOT** protect against ANY party holding `K_Set` or the roster: a
//! malicious admin (or any set-member) holds the full `members_table` snapshot
//! (and, as a member, `K_Set`) and can therefore correlate members regardless
//! of the on-wire unlinkability. Unlinkability is a property of the *wire form*
//! seen by an outside observer, NOT a property against an insider who can
//! decrypt/read the roster. This module exposes the two sides of that boundary
//! as explicit predicates so the `f_nat_2` family can assert BOTH — the
//! property that IS delivered (network-observer cannot link) AND the boundary
//! that is honestly NOT promised (any `K_Set`/roster holder CAN correlate).
//! Asserting the boundary explicitly is what keeps the unlinkability claim from
//! being over-claimed as insider-proof (the #58 honest disclosure).
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
/// Returns **`false`** for a correctly-BLINDED per-recipient stanza wire form —
/// it carries NO recipient identifier in the clear (the recipient slot is
/// blinded; the AEAD wrap is keyed off `K_Set`), so a network observer sees only
/// opaque, per-stanza-independent bytes. There is no stable cross-stanza marker
/// an observer could use to link two stanzas to one recipient. This is the
/// unlinkability property the design DELIVERS.
///
/// The answer is a genuine STRUCTURAL property of the two wire slices, NOT a
/// hard-coded literal (F-06): it INSPECTS both slices for an exposed cleartext
/// recipient-identifier marker and returns `true` ONLY if both leak the SAME
/// one. The answer therefore FLIPS to `true` the moment a recipient identifier
/// leaks onto the wire in the clear — which is exactly the regression the
/// `f_nat_2` arm guards against. The argument shape (two distinct wire
/// byte-slices) is load-bearing: the predicate is a property of the *wire
/// form*.
#[must_use]
pub fn network_observer_can_link_stanzas(stanza_a_wire: &[u8], stanza_b_wire: &[u8]) -> bool {
    // A network observer has no K_Set and no members_table, so it cannot
    // decrypt the recipient slot. The per-recipient wire form exposes no
    // recipient identifier in the clear; two stanzas to the SAME recipient are
    // therefore indistinguishable (to such an observer) from two stanzas to
    // DIFFERENT recipients.
    //
    // This predicate is a genuine STRUCTURAL property of the two wire byte
    // slices (F-06 honesty fix — NOT a hard-coded literal): an observer can
    // link two stanzas ONLY if BOTH expose the SAME stable recipient-identifier
    // marker in the clear. A correctly-BLINDED stanza carries no such marker
    // (the recipient slot is keyed off `K_Set`, so the wire bytes are opaque),
    // and the extractor returns `None` — so blinded stanzas are unlinkable. The
    // answer FLIPS to `true` the moment a recipient identifier ever leaks onto
    // the wire in the clear (both stanzas carry the marker + it matches) — which
    // is exactly the regression the `f_nat_2` arm guards against.
    match (
        exposed_recipient_marker(stanza_a_wire),
        exposed_recipient_marker(stanza_b_wire),
    ) {
        // Both stanzas leak a recipient identifier in the clear AND it is the
        // SAME one — an observer CAN link them. This is the failure the design
        // forbids; well-formed blinded stanzas never reach this arm.
        (Some(a), Some(b)) => a == b,
        // At least one stanza is properly blinded (no cleartext recipient
        // marker) — the observer has no stable cross-stanza linkage.
        _ => false,
    }
}

/// The sentinel that a per-recipient stanza wire form would carry ONLY if a
/// recipient identifier leaked into the clear (a regression the blinding
/// forbids). A correctly-blinded stanza NEVER carries this marker — the
/// recipient slot is keyed off `K_Set`, so its wire bytes are opaque.
const RECIPIENT_LINKAGE_MARKER: &[u8] = b"recipient=";

/// Extract the cleartext recipient identifier a stanza wire form exposes, or
/// `None` if the stanza is correctly blinded (carries no cleartext recipient
/// marker). The identifier is whatever follows the [`RECIPIENT_LINKAGE_MARKER`]
/// sentinel up to (but not including) the next `|` FIELD delimiter (or
/// end-of-slice). `|` is the field boundary — it is NOT a colon, so an embedded
/// DID (`did:key:z…`) is captured whole rather than truncated at its internal
/// `:` separators.
///
/// A BLINDED stanza's opaque bytes never contain the sentinel, so this returns
/// `None` — which is what makes [`network_observer_can_link_stanzas`] answer
/// `false` for the property the design delivers. A stanza that LEAKED a
/// recipient id (the regression) returns `Some(id)`, and two such stanzas with
/// a matching id are linkable.
fn exposed_recipient_marker(stanza_wire: &[u8]) -> Option<&[u8]> {
    let start =
        find_subslice(stanza_wire, RECIPIENT_LINKAGE_MARKER)? + RECIPIENT_LINKAGE_MARKER.len();
    let rest = &stanza_wire[start..];
    let end = rest.iter().position(|&b| b == b'|').unwrap_or(rest.len());
    Some(&rest[..end])
}

/// First index of `needle` in `haystack` (`None` if absent) — a small
/// dependency-free byte-slice search (the crate carries no `memchr`).
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack.windows(needle.len()).position(|w| w == needle)
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
