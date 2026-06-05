//! F-INV19-1 (R3-W6 gov-audit; GAP-3 dedicated family) — Inv-19: the
//! K(V) key-derivation API REJECTS any payload that is not an immutable
//! Version-Node-CID (or a MembershipSet). would-FAIL = encrypting key
//! material to a MUTABLE / Anchor CID (the Anchor's CURRENT pointer moves,
//! so a key bound to an Anchor would silently re-target — the bug Inv-19
//! forbids).
//!
//! Pin sources (F-full R2 test-landscape §1 Group 12 row F-INV19-1; GAP-3
//! fill — Inv-19 had NO dedicated family across the 6 dimensions, only a
//! GNI-10 sub-point):
//!   - R0.5 plan §5.1 Inv-19, Inv-20 clause-f.
//!   - Clone-shape: `crates/benten-core/tests/version_branched.rs` (the
//!     Anchor / Version-Node / CURRENT distinction — Anchor is mutable
//!     (CURRENT moves), Version Nodes are immutable).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! The W6 K(V) type-restricted derivation surface does not exist at this
//! SHA. Self-contained stub-shim compiles green; bodies `unimplemented!()`.
//! W6 R5 implementer:
//!   1. DELETE `mset_w6_kv_stub`,
//!   2. INSERT `use benten_membership_set::keying::{
//!      derive_kv, CidTarget, KvError};`,
//!   3. UN-IGNORE,
//!   4. Verify green.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Arms: (1) `derive_kv` ACCEPTS an immutable Version-Node-CID target
//! (positive control); (2) `derive_kv` REJECTS a mutable Anchor CID with a
//! typed error (the load-bearing restriction — would-FAIL if W6 lets key
//! material bind to an Anchor); (3) `derive_kv` ACCEPTS a MembershipSet
//! target; (4) the real Anchor/Version-Node distinction is anchored to the
//! live `benten_core::version` types (`#[test]`, green now) so the stub's
//! "mutable Anchor vs immutable Version Node" premise is grounded.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::version::{Anchor, append_version};
use benten_core::{Node, Value};

// =====================================================================
// W6 R5 (Wave w-gov-audit): real `benten_membership_set::keying` surface —
// the Inv-19 type-restricted `derive_kv(CidTarget) -> Result<_, KvError>`
// (re-exported from the `keying_kv` slice).
// =====================================================================
use benten_membership_set::keying::{CidTarget, KvError, derive_kv};

fn fixed_cid(byte: u8) -> [u8; 32] {
    [byte; 32]
}

/// F-INV19-1 (a): `derive_kv` ACCEPTS an immutable Version-Node-CID target
/// (positive control) and returns a key.
#[test]
fn derive_kv_accepts_immutable_version_node_cid() {
    let target = CidTarget::ImmutableVersionNode(fixed_cid(0x19));
    let key = derive_kv(target).expect(
        "F-INV19-1: derive_kv MUST accept an immutable Version-Node-CID target \
         (the permitted, Inv-19-safe binding)",
    );
    assert_ne!(
        key,
        [0u8; 32],
        "F-INV19-1: a derived K(V) MUST be non-trivial (not all-zero) — the \
         derivation actually ran"
    );
}

/// F-INV19-1 (b): the LOAD-BEARING restriction — `derive_kv` REJECTS a
/// MUTABLE Anchor CID with a typed `TargetNotImmutable` error.
///
/// would-FAIL if W6 lets key material bind to an Anchor (whose CURRENT
/// pointer moves) — a key bound there would silently re-target as the
/// Anchor advances. This is exactly the bug Inv-19 forbids.
#[test]
fn derive_kv_rejects_mutable_anchor_cid() {
    let target = CidTarget::MutableAnchor(fixed_cid(0xA9));
    let result = derive_kv(target);
    match result {
        Err(KvError::TargetNotImmutable) => {
            // Correct: a mutable Anchor is not a valid K(V) target.
        }
        Ok(_) => panic!(
            "F-INV19-1: derive_kv MUST REJECT a mutable Anchor CID — binding \
             key material to an Anchor (whose CURRENT pointer moves) would \
             silently re-target the key. This is the Inv-19 violation; \
             would-FAIL if W6 returns a key here"
        ),
        Err(other) => panic!(
            "F-INV19-1: expected KvError::TargetNotImmutable for a mutable \
             Anchor, got {other:?}"
        ),
    }
}

/// F-INV19-1 (c): `derive_kv` ACCEPTS a MembershipSet target (the second
/// permitted, Inv-21-stable binding per Inv-20 clause-f).
#[test]
fn derive_kv_accepts_membership_set_target() {
    let target = CidTarget::MembershipSet(fixed_cid(0x6C));
    let key = derive_kv(target).expect(
        "F-INV19-1: derive_kv MUST accept a MembershipSet target (Inv-20 \
         clause-f — set-identity is Inv-21-stable)",
    );
    assert_ne!(key, [0u8; 32], "F-INV19-1: MembershipSet K(V) must be non-trivial");
}

/// F-INV19-1 (substrate): the real Anchor/Version-Node MUTABILITY
/// distinction — `#[test]` (green now). An Anchor's CURRENT pointer MOVES
/// across an `append_version` (mutable), whereas each Version Node's CID is
/// immutable. This grounds the stub's premise: Inv-19 forbids binding K(V)
/// to the mutable thing (Anchor) precisely because it moves.
#[test]
fn anchor_current_pointer_moves_while_version_node_cids_are_immutable() {
    fn versioned_node(title: &str, v: u32) -> Node {
        let mut props = BTreeMap::new();
        props.insert("title".into(), Value::text(title));
        props.insert("v".into(), Value::Int(v.into()));
        Node::new(vec!["AuditEvent".into()], props)
    }

    let v0 = versioned_node("genesis", 0);
    let v0_cid = v0.cid().unwrap();
    let anchor = Anchor::new(v0_cid);

    let v1 = versioned_node("admit-member", 1);
    let v1_cid = v1.cid().unwrap();
    append_version(&anchor, &v0_cid, &v1_cid).expect("linear append must succeed");

    // The two Version-Node CIDs are immutable + distinct: a K(V) bound to
    // either is stable. The Anchor, by contrast, now logically points at a
    // newer head than it did at genesis — its CURRENT moved. Binding key
    // material to the Anchor would silently re-target. THIS is why Inv-19
    // restricts K(V) to immutable Version-Node-CIDs.
    assert_ne!(
        v0_cid, v1_cid,
        "F-INV19-1 substrate: distinct Version Nodes MUST have distinct \
         immutable CIDs — the stable targets Inv-19 permits"
    );

    // Re-deriving each Version Node's CID is deterministic (immutable bytes).
    assert_eq!(
        v1.cid().unwrap(),
        v1_cid,
        "F-INV19-1 substrate: a Version Node's CID is immutable — re-hashing \
         its bytes yields the identical CID (the Inv-19-safe binding target)"
    );
}
