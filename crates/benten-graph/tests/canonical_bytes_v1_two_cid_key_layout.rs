//! W-08b (R6 round-#1 falsification sweep) — FULL-LAYOUT byte-pins for the
//! two-CID at-rest redb key encodings.
//!
//! # Why this file exists
//!
//! `TwoCidMap::partition_table_key(did, cid)` produces the durable redb key
//! `d:<namespace_did> :m: <plaintext_cid>` that every per-DID two-CID mapping
//! row is stored under (`RedbBackend::two_cid_lookup_scoped` /
//! `two_cid_lookup_with_namespace`). It is an AT-REST layout: it is frozen at
//! `phase-4-meta-core-close` in the same sense a wire format is, because an
//! existing redb file written under the old layout stops resolving under a new
//! one.
//!
//! Before this file the ONLY pin on that key was
//! `two_cid_map.rs::tests::partition_table_key_starts_with_did_prefix`, which
//! asserted `starts_with(b"d:")` plus "the 3 bytes `:m:` appear SOMEWHERE".
//! Both survive the highest-consequence mutation available on this surface —
//! swapping the two `extend_from_slice` arguments so the key becomes
//! `d:<plaintext_cid>:m:<namespace_did>`. That mutation silently cross-wires
//! every per-DID partition lookup AND defeats
//! `redb_backend.rs::parse_namespace_from_mapping_key`'s DID recovery (it
//! would recover the plaintext CID and derive K(N) under the wrong
//! K_principal), which is a partition-isolation failure — the invariant
//! `multitenant-r1-5` positively claims is enforced BEFORE the AEAD layer.
//!
//! # Discipline
//!
//! Every pin below names, in a comment, the exact one-line mutation that must
//! make it fail. A pin whose mutation cannot be named is worthless. The
//! absolute-hex goldens come OUT of the real encoder (M-20
//! golden-hex-via-throwaway-compute); they are never hand-written.

#![allow(clippy::unwrap_used)]

use benten_core::Cid;
use benten_graph::two_cid_map::TwoCidMap;

/// Lowercase-hex encoder (no `hex` crate dep in this workspace). Mirrors the
/// helper in `canonical_bytes_v1_aead_wrap.rs`.
fn to_hex(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// `Cid::as_bytes()` is the 36-byte raw multiformats layout
/// (`01 71 1e 20` multicodec/multihash framing + 32-byte BLAKE3 digest). Every
/// offset below is written against this literal width rather than against
/// `cid.as_bytes().len()` so a CID-width drift fails HERE (loudly, with this
/// message) instead of silently sliding every downstream offset.
const CID_WIDTH: usize = 36;

/// The namespace DID and the plaintext CID MUST use DISTINCT digests. With
/// equal digests every assertion in this file would be blind to an argument
/// swap — the single most consequential mutation on this surface.
fn did_fixture() -> Cid {
    Cid::from_blake3_digest([0xAA; 32])
}

fn plaintext_cid_fixture() -> Cid {
    Cid::from_blake3_digest([0x42; 32])
}

#[test]
fn cid_raw_width_is_36_bytes() {
    // Guard for every literal offset in this file. WHAT BREAKS: if
    // `Cid::as_bytes()` stops being 36 bytes, the segment offsets below are
    // wrong and their failures would be misleading.
    //
    // MUTATION THAT MUST MAKE THIS FAIL: any change to the raw CID framing in
    // `benten_core::Cid::as_bytes` (e.g. dropping the 4-byte
    // multicodec/multihash prefix, or swapping BLAKE3-256 for a
    // different-width digest).
    assert_eq!(
        did_fixture().as_bytes().len(),
        CID_WIDTH,
        "raw Cid width drifted — every offset pin in this file assumes 36 B"
    );
}

/// W-08b closure — the FULL `d:<did>:m:<cid>` layout, segment by segment, at
/// exact offsets, with an exact total width.
///
/// MUTATIONS THAT MUST MAKE THIS FAIL (each is exactly one line in
/// `crates/benten-graph/src/two_cid_map.rs`):
///   :80  `k.extend_from_slice(namespace_did.as_bytes());`
///     →  `k.extend_from_slice(plaintext_cid.as_bytes());`   (argument swap —
///        the mutation the old prefix-only pin could not see)
///   :82  `k.extend_from_slice(plaintext_cid.as_bytes());`
///     →  `k.extend_from_slice(namespace_did.as_bytes());`   (mirror swap)
///   :79  `k.extend_from_slice(b"d:");`   → `b"e:"` / `b"d"` / `b"dd:"`
///   :81  `k.extend_from_slice(b":m:");`  → `b":n:"` / `b":m"` / moved before
///        the DID bytes (the "contains `:m:` somewhere" pin survives a MOVE;
///        the exact-offset assertion below does not)
#[test]
fn partition_table_key_full_layout_frozen() {
    let did = did_fixture();
    let cid = plaintext_cid_fixture();
    let key = TwoCidMap::partition_table_key(&did, &cid);

    // Exact total width: `d:` (2) + DID (36) + `:m:` (3) + plaintext CID (36).
    // A stray padding byte, a dropped segment, or a doubled separator fails
    // here first.
    assert_eq!(
        key.len(),
        2 + CID_WIDTH + 3 + CID_WIDTH,
        "partition_table_key total width drifted — at-rest key layout break"
    );

    // Segment 1 — literal `d:` at offset 0.
    assert_eq!(
        &key[0..2],
        b"d:",
        "partition-key DID prefix drifted — at-rest key layout break"
    );

    // Segment 2 — the NAMESPACE DID at offset 2. The paired `assert_ne!` is
    // what makes the argument swap load-bearing: with the swap applied this
    // slice holds the plaintext CID, and the `assert_ne!` fires even if a
    // future refactor also loosens the `assert_eq!`.
    assert_eq!(
        &key[2..2 + CID_WIDTH],
        did.as_bytes(),
        "partition-key segment 2 MUST be the NAMESPACE DID (not the plaintext CID)"
    );
    assert_ne!(
        &key[2..2 + CID_WIDTH],
        cid.as_bytes(),
        "partition-key segment 2 holds the PLAINTEXT CID — the two \
         extend_from_slice arguments are swapped; every per-DID lookup is \
         cross-wired and parse_namespace_from_mapping_key recovers the wrong DID"
    );

    // Segment 3 — literal `:m:` at the EXACT offset 38. The superseded pin
    // only asked whether `:m:` appeared anywhere in the key; that survives
    // relocating the delimiter.
    assert_eq!(
        &key[2 + CID_WIDTH..2 + CID_WIDTH + 3],
        b":m:",
        "partition-key mapping delimiter drifted or moved — at-rest key layout break"
    );

    // Segment 4 — the PLAINTEXT CID at offset 41, through the end.
    assert_eq!(
        &key[2 + CID_WIDTH + 3..],
        cid.as_bytes(),
        "partition-key segment 4 MUST be the PLAINTEXT CID (not the namespace DID)"
    );
    assert_ne!(
        &key[2 + CID_WIDTH + 3..],
        did.as_bytes(),
        "partition-key segment 4 holds the NAMESPACE DID — arguments swapped"
    );
}

/// Absolute-hex golden for `partition_table_key`. The structural pin above is
/// necessarily a re-statement of the producer's own segment order, so it is
/// blind to a COORDINATED mutation that changes producer and expectation
/// together. This golden is not: the literal below is a frozen constant that a
/// coordinated edit cannot carry along.
///
/// MUTATION THAT MUST MAKE THIS FAIL: any byte-level change to
/// `partition_table_key` at all — including one applied simultaneously to the
/// structural pin above.
///
/// PROVENANCE: the literal below was CAPTURED FROM THE REAL ENCODER at R6
/// round #1 (M-20 — goldens are never hand-authored) by running:
///
/// ```text
/// CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_BUILD_JOBS=6 \
///   cargo nextest run -p benten-graph \
///   --test canonical_bytes_v1_two_cid_key_layout \
///   partition_table_key_absolute_golden_hex
/// ```
///
/// Shape: 154 hex chars = 77 bytes (`64 3a` + 36-B DID + `3a 6d 3a` + 36-B CID).
///
/// The command is kept so a future maintainer can RE-DERIVE the value when an
/// at-rest layout change is deliberate and ratified. **If this test fails and
/// you did not intend a layout change, the producer regressed — fix
/// `two_cid_map.rs`, not this literal.** Re-capturing to clear a red is the
/// wrong fix: it converts a caught regression into a silent forever-break of
/// every persisted redb mapping row.
#[test]
fn partition_table_key_absolute_golden_hex() {
    let key = TwoCidMap::partition_table_key(&did_fixture(), &plaintext_cid_fixture());
    let got = to_hex(&key);
    let expected = "643a01711e20aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3a6d3a01711e204242424242424242424242424242424242424242424242424242424242424242";
    assert_eq!(
        got, expected,
        "partition_table_key at-rest bytes drifted from the frozen v1-beta layout.\n\
         GOLDEN-CAPTURE partition_table_key = \"{got}\""
    );
}

/// `table_key` (the UN-namespaced `m:<plaintext_cid>` form) full-layout pin.
///
/// NOTE FOR THE FREEZE RECORD: this surface has ZERO non-test callers at
/// `7bb1a9fa` — see NOTES.md. It is nonetheless `pub` and recorded in
/// `docs/public-api/benten-graph.txt`, so it is inside the permanent freeze
/// contract and is pinned here accordingly. Pinning it is NOT an argument for
/// keeping it; whether to delete it before the tag is a separate decision.
///
/// MUTATIONS THAT MUST MAKE THIS FAIL (each is one line in `two_cid_map.rs`):
///   :62  `k.extend_from_slice(b"m:");` → `b"n:"` / `b"m"` / `b"mm:"`
///   :63  dropping or reordering `k.extend_from_slice(plaintext_cid.as_bytes());`
#[test]
fn table_key_full_layout_frozen() {
    let cid = plaintext_cid_fixture();
    let key = TwoCidMap::table_key(&cid);

    assert_eq!(
        key.len(),
        2 + CID_WIDTH,
        "table_key total width drifted — at-rest key layout break"
    );
    assert_eq!(&key[0..2], b"m:", "table_key prefix drifted");
    assert_eq!(
        &key[2..],
        cid.as_bytes(),
        "table_key payload MUST be exactly the plaintext CID raw bytes"
    );

    // Differential against the namespaced form: the two key spaces MUST NOT
    // collide. WHAT BREAKS if they do: an un-namespaced legacy row would be
    // reachable from a per-DID scoped lookup, defeating partition isolation.
    //
    // MUTATION THAT MUST MAKE THIS FAIL: changing `partition_table_key`'s
    // `b"d:"` prefix to `b"m:"` (or dropping the prefix entirely).
    let namespaced = TwoCidMap::partition_table_key(&did_fixture(), &cid);
    assert_ne!(
        key, namespaced,
        "un-namespaced and per-DID key spaces MUST NOT collide"
    );
    assert!(
        !namespaced.starts_with(b"m:"),
        "per-DID key MUST NOT land in the un-namespaced `m:` key space"
    );
}

/// Absolute-hex golden for `table_key` — same coordinated-mutation rationale
/// as `partition_table_key_absolute_golden_hex`.
///
/// PROVENANCE: captured from the real encoder at R6 round #1 (M-20) by running:
///
/// ```text
/// CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_BUILD_JOBS=6 \
///   cargo nextest run -p benten-graph \
///   --test canonical_bytes_v1_two_cid_key_layout \
///   table_key_absolute_golden_hex
/// ```
///
/// Shape: 76 hex chars = 38 bytes (`6d 3a` + 36-B CID). Re-derive ONLY for a
/// deliberate, ratified layout change — see the sibling golden above.
#[test]
fn table_key_absolute_golden_hex() {
    let key = TwoCidMap::table_key(&plaintext_cid_fixture());
    let got = to_hex(&key);
    let expected = "6d3a01711e204242424242424242424242424242424242424242424242424242424242424242";
    assert_eq!(
        got, expected,
        "table_key at-rest bytes drifted from the frozen v1-beta layout.\n\
         GOLDEN-CAPTURE table_key = \"{got}\""
    );
}
