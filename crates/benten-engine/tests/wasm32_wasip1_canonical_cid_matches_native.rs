//! Phase-2b G10-A-wasip1 — wasm32-wasip1 canonical-CID dual-target invariant.
//!
//! ## What this tests
//!
//! The Phase-1 canonical fixture node
//! (`benten_core::testing::canonical_test_node`) hashes to the SAME CID
//! `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` regardless
//! of which target compiles `benten-core`. wasm-r1-1 lifts this to a
//! workflow-level invariant: the wasm32-wasip1 build under wasmtime MUST
//! re-derive the identical CID, otherwise something in the canonical-bytes
//! pipeline (DAG-CBOR encoder, BLAKE3, Cid wire format) drifted.
//!
//! Native-side, this test pins the fixture CID byte-for-byte so a future
//! drift to the canonical-bytes encoding (e.g. `serde_ipld_dagcbor` major
//! bump, `BTreeMap` insertion-order regression, `Cid::from_blake3_digest`
//! wire layout) trips here at native test time, not only when the
//! wasm-runtime workflow runs.
//!
//! The `wasm-runtime.yml` workflow runs the analogous
//! `wasm32-wasip1`-built test under wasmtime and asserts the SAME literal.
//! Because `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`
//! appears in BOTH places (this test + the workflow), drift on either
//! side fires immediately.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::testing::canonical_test_node;

/// Phase-1 canonical fixture CID. Derived from the canonical
/// `Post {title: "Hello, Benten", published: true, views: 42, tags:
/// ["rust", "graph"]}` Node via DAG-CBOR + BLAKE3 + CIDv1 wrap. Pinned
/// here because the wasm32-wasip1 dual-target invariant rests on this
/// literal being byte-stable.
const CANONICAL_FIXTURE_CID_BASE32: &str =
    "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda";

#[test]
fn wasm32_wasip1_canonical_cid_matches_native() {
    let node = canonical_test_node();
    let cid = node.cid().expect("canonical fixture must encode");
    let observed = cid.to_base32();
    assert_eq!(
        observed, CANONICAL_FIXTURE_CID_BASE32,
        "canonical fixture CID drifted from the Phase-1 pin — \
         either the canonical-bytes pipeline (DAG-CBOR encoder, BLAKE3, \
         Cid wire format) regressed OR the canonical fixture's content \
         changed. The wasm-runtime workflow re-runs this test under \
         wasmtime and asserts the SAME literal — drift here breaks the \
         wasm32-wasip1 dual-target invariant (wasm-r1-1)."
    );
}

/// Round-trip the base32 representation through `Cid::from_str` to prove
/// the multibase encoding is symmetric. A wasm32-wasip1 build that drifts
/// the multibase alphabet (e.g. uppercase vs lowercase) fails this side
/// even when the digest itself is correct.
#[test]
fn canonical_fixture_cid_base32_round_trips() {
    use benten_core::Cid;
    let parsed = Cid::from_str(CANONICAL_FIXTURE_CID_BASE32).expect("base32 round-trip");
    let node = canonical_test_node();
    let cid = node.cid().unwrap();
    assert_eq!(parsed, cid);
}
