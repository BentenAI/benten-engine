//! cag-mr-g12c-cont-1 (D5) — verify that the relocated subgraph types do
//! NOT expose generic `serde::Serialize` / `serde::Deserialize` impls, and
//! that the canonical-bytes round-trip is unaffected.
//!
//! # What this test pins
//!
//! Phase-2b G12-C-cont relocated `Subgraph`, `OperationNode`, `NodeHandle`,
//! and `PrimitiveKind` from `benten-eval` to `benten-core`. The relocated
//! types arrived carrying `#[derive(Serialize, Deserialize)]` from their
//! pre-relocation home — a SECONDARY (non-canonical) encoding that did NOT
//! match the authoritative `canonical_subgraph_bytes` shape. A caller using
//! `serde_ipld_dagcbor::to_vec(&sg)` produced bytes whose BLAKE3 differed
//! from `sg.cid()`, and a caller using `serde_json::to_string(&sg)`
//! produced a JSON shape that did not round-trip through
//! `Subgraph::load_verified`.
//!
//! This test pins the fix-pass:
//!
//! 1. **Negative trait-impl probe** (autoref-specialisation): proves at
//!    runtime that none of the four relocated types impl
//!    `serde::Serialize` or `serde::de::DeserializeOwned`.
//! 2. **Canonical round-trip pin**: re-asserts that the canonical encode
//!    (`canonical_bytes` / `to_dagcbor` / `to_dag_cbor`) and decode
//!    (`load_verified` / `from_dagcbor` / `load_verified_with_cid`) entry
//!    points produce byte-identical CIDs across single-node and
//!    multi-node fixtures — the absence of generic-serde impls has not
//!    regressed the canonical path.
//!
//! # Negative-impl probe technique
//!
//! Stable Rust does not have `static_assertions::assert_not_impl_any` (and
//! we do not pull `static_assertions` into dev-deps for one test). The
//! `probe::*` module below uses the **autoref-specialisation** trick
//! popularised by dtolnay (`dtolnay/case-studies/autoref-specialization`):
//! method resolution walks the auto-deref ladder considering candidates
//! at each step, so an impl on `&Probe<T>` requiring `T: Serialize` is
//! preferred over an impl on `Probe<T>` requiring no bound. Calling the
//! method via `(&&Probe::<T>::new()).impls_serialize()` — note the
//! double reference — engages the ladder: when `T: Serialize`, the
//! bounded impl on `&Probe<T>` matches at the auto-ref step and returns
//! `true`; when it does not, the bounded candidate is filtered out and
//! the bare-`Probe<T>` impl returns `false`. Sanity tests pin both
//! directions of the pattern.
//!
//! See cag-mr-g12c-cont-1 in `.addl/phase-2b/r5-decisions-log.md` D5 for
//! design rationale.

use benten_core::{Subgraph, SubgraphBuilder};

// -------------------------------------------------------------------------
// Canonical round-trip property: encode + decode + re-encode produces
// byte-identical bytes and a stable CID. Pins that the absence of
// generic-serde impls has not regressed the canonical path.
// -------------------------------------------------------------------------

#[test]
fn canonical_round_trip_single_node_subgraph_byte_identical_cid() {
    let mut b = SubgraphBuilder::new("rt-single");
    let _ = b.read("entry");
    let sg = b.build_unvalidated_for_test();

    let bytes_a = sg.canonical_bytes().expect("encode A");
    let bytes_b = sg.to_dagcbor().expect("encode B (alias)");
    let bytes_c = sg.to_dag_cbor().expect("encode C (alias)");
    assert_eq!(bytes_a, bytes_b, "canonical_bytes must equal to_dagcbor");
    assert_eq!(bytes_a, bytes_c, "canonical_bytes must equal to_dag_cbor");

    let cid_a = sg.cid().expect("cid A");
    let decoded = Subgraph::load_verified(&bytes_a).expect("decode");
    let cid_b = decoded.cid().expect("cid B");
    assert_eq!(
        cid_a, cid_b,
        "encode -> load_verified -> cid must round-trip byte-identical"
    );

    // load_verified_with_cid (integrity-enforcing variant) must also accept.
    let decoded2 =
        Subgraph::load_verified_with_cid(&cid_a, &bytes_a).expect("load_verified_with_cid");
    assert_eq!(
        decoded2.cid().expect("cid C"),
        cid_a,
        "load_verified_with_cid must round-trip byte-identical"
    );
}

#[test]
fn canonical_round_trip_multi_node_subgraph_byte_identical_cid() {
    let mut b = SubgraphBuilder::new("rt-multi");
    let r = b.read("entry");
    let t = b.transform(r, "noop");
    let _resp = b.respond(t);
    let sg = b.build_unvalidated_for_test();

    let bytes = sg.canonical_bytes().expect("encode");
    let cid_before = sg.cid().expect("cid pre-decode");
    let decoded = Subgraph::load_verified(&bytes).expect("decode");
    let bytes_again = decoded.canonical_bytes().expect("re-encode");

    assert_eq!(
        bytes, bytes_again,
        "encode -> decode -> encode must produce byte-identical bytes"
    );
    assert_eq!(
        cid_before,
        decoded.cid().expect("cid post-decode"),
        "encode -> decode round-trip must preserve CID"
    );
}

// -------------------------------------------------------------------------
// Negative trait-impl probe via the dtolnay autoref-specialisation trick.
//
// The receiver of the method call is constructed with TWO levels of
// reference: `(&&Probe::<T>::new()).check()`. Method resolution walks the
// auto-deref ladder considering candidates at each step:
//
//   step 1: receiver `&&Probe<T>`. Candidates:
//     - `Auto<T>::check(&self)` where `Self = &Probe<T>` (auto-ref to
//        get `&&Probe<T>`). Bound: `T: Serialize`. **Wins** if the
//        bound holds.
//   step 2: receiver `&Probe<T>` (one deref). Candidates:
//     - `Bare<T>::check(&self)` where `Self = Probe<T>` (auto-ref).
//       No bound. **Always wins** at this step if step 1 fired no
//       candidate.
//
// When `T: Serialize`, step 1 fires `Auto::check`, returning `true`.
// When it does not, step 1 has no candidate; step 2 fires
// `Bare::check`, returning `false`.
// -------------------------------------------------------------------------

mod probe {
    use core::marker::PhantomData;
    pub struct Probe<T>(PhantomData<T>);
    impl<T> Probe<T> {
        pub const fn new() -> Self {
            Self(PhantomData)
        }
    }

    pub trait AutoSerialize {
        fn impls_serialize(&self) -> bool {
            true
        }
    }
    impl<T: serde::Serialize> AutoSerialize for &Probe<T> {}

    pub trait BareSerialize {
        fn impls_serialize(&self) -> bool {
            false
        }
    }
    impl<T> BareSerialize for Probe<T> {}

    pub trait AutoDeserialize {
        fn impls_deserialize(&self) -> bool {
            true
        }
    }
    impl<T: serde::de::DeserializeOwned> AutoDeserialize for &Probe<T> {}

    pub trait BareDeserialize {
        fn impls_deserialize(&self) -> bool {
            false
        }
    }
    impl<T> BareDeserialize for Probe<T> {}
}

use benten_core::{NodeHandle, OperationNode, PrimitiveKind};
use probe::{
    AutoDeserialize as _, AutoSerialize as _, BareDeserialize as _, BareSerialize as _, Probe,
};

#[test]
fn sanity_pattern_detects_a_known_serialize_type() {
    // Sanity: `String: Serialize` so the auto arm fires (returns true).
    // If this fails, the negative-impl tests below are vacuously passing.
    assert!(
        (&&Probe::<String>::new()).impls_serialize(),
        "sanity: String IS Serialize; the autoref-specialisation pattern \
         must report true for it. If this fails, the negative-impl tests \
         in this file are vacuously passing."
    );
}

#[test]
fn sanity_pattern_detects_a_known_deserialize_type() {
    assert!(
        (&&Probe::<String>::new()).impls_deserialize(),
        "sanity: String IS DeserializeOwned; the autoref-specialisation \
         pattern must report true for it. If this fails, the negative-impl \
         tests in this file are vacuously passing."
    );
}

#[test]
fn sanity_pattern_detects_a_known_non_serialize_type() {
    // Sanity: a local struct with no `Serialize` impl must report false.
    struct NotSerialize;
    assert!(
        !(&&Probe::<NotSerialize>::new()).impls_serialize(),
        "sanity: NotSerialize does NOT impl Serialize; the autoref \
         pattern must report false for it. If this fails, the negative \
         arm of the pattern is broken and the whole probe is unsound."
    );
}

#[test]
fn sanity_pattern_detects_a_known_non_deserialize_type() {
    struct NotDeserialize;
    assert!(
        !(&&Probe::<NotDeserialize>::new()).impls_deserialize(),
        "sanity: NotDeserialize does NOT impl DeserializeOwned; the \
         autoref pattern must report false for it."
    );
}

#[test]
fn subgraph_does_not_implement_serialize() {
    assert!(
        !(&&Probe::<Subgraph>::new()).impls_serialize(),
        "cag-mr-g12c-cont-1: Subgraph MUST NOT impl serde::Serialize. \
         A Serialize impl would re-introduce the silent non-canonical- \
         encoding footgun. Encode via Subgraph::canonical_bytes / \
         to_dag_cbor / to_dagcbor instead."
    );
}

#[test]
fn subgraph_does_not_implement_deserialize_owned() {
    assert!(
        !(&&Probe::<Subgraph>::new()).impls_deserialize(),
        "cag-mr-g12c-cont-1: Subgraph MUST NOT impl \
         serde::de::DeserializeOwned. Decode via Subgraph::load_verified / \
         from_dagcbor / load_verified_with_cid instead."
    );
}

#[test]
fn operation_node_does_not_implement_serialize() {
    assert!(
        !(&&Probe::<OperationNode>::new()).impls_serialize(),
        "cag-mr-g12c-cont-1: OperationNode MUST NOT impl \
         serde::Serialize. The canonical encoding routes via \
         canonical_subgraph_bytes' CanonNodeRef projection."
    );
}

#[test]
fn operation_node_does_not_implement_deserialize_owned() {
    assert!(
        !(&&Probe::<OperationNode>::new()).impls_deserialize(),
        "cag-mr-g12c-cont-1: OperationNode MUST NOT impl \
         serde::de::DeserializeOwned. Decode flows through \
         Subgraph::load_verified -> from_canonical_owned."
    );
}

#[test]
fn primitive_kind_does_not_implement_serialize() {
    assert!(
        !(&&Probe::<PrimitiveKind>::new()).impls_serialize(),
        "cag-mr-g12c-cont-1: PrimitiveKind MUST NOT impl serde::Serialize. \
         The canonical encoding routes via PrimitiveKind::canonical_tag \
         (stable string tag — 'READ', 'WRITE', ...)."
    );
}

#[test]
fn primitive_kind_does_not_implement_deserialize_owned() {
    assert!(
        !(&&Probe::<PrimitiveKind>::new()).impls_deserialize(),
        "cag-mr-g12c-cont-1: PrimitiveKind MUST NOT impl \
         serde::de::DeserializeOwned. Decode via \
         PrimitiveKind::from_canonical_tag (private; reached through \
         Subgraph::load_verified)."
    );
}

#[test]
fn node_handle_does_not_implement_serialize() {
    assert!(
        !(&&Probe::<NodeHandle>::new()).impls_serialize(),
        "cag-mr-g12c-cont-1: NodeHandle MUST NOT impl serde::Serialize. \
         Handles are transient builder-time indices, never part of the \
         canonical-bytes shape."
    );
}

#[test]
fn node_handle_does_not_implement_deserialize_owned() {
    assert!(
        !(&&Probe::<NodeHandle>::new()).impls_deserialize(),
        "cag-mr-g12c-cont-1: NodeHandle MUST NOT impl \
         serde::de::DeserializeOwned. Handles do not survive a round trip \
         through canonical bytes."
    );
}
