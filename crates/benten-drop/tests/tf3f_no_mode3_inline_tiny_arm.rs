//! TF-3f pins — G-CORE-3f: no Mode-3 inline-tiny arm (deferred to post-v1).
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition.
//!
//! **SPEC-GAP SURFACED:** eventual destination is `benten-drop` (see
//! sibling file `tf3f_drop_bundle_offline_consume.rs` header). Lands in
//! `benten-sync/tests/` as a temporary home; relocate at G-CORE-3f.
//!
//! Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3f (A-1):
//!     "Mode 3 inline-tiny ABSENT: assert no `InlineContent` arm in
//!     DropBundle (defer-to-post-v1 contract). If present, reject
//!     construction."
//!   - `00-implementation-plan.md` §3 G-CORE-3 def input-constraints
//!     refinement #6 L341: "3 sendme deployment modes (online-pull /
//!     offline-Drop / inline-tiny — ship Mode 2 in G-CORE-3f, defer
//!     Mode 3 to post-v1)."
//!   - `00-implementation-plan.md` §3 G-CORE-3f wave def: "mode 2
//!     sendme→Drop ships; mode 3 inline-tiny defers to post-v1".
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-3f (pim-12 / §3.6e).
//! ============================================================================
//! This is a STRUCTURAL pin (compile-time): the `DropBundle` enum (or
//! struct field) MUST NOT carry an `InlineContent` arm. The pin uses a
//! `#[non_exhaustive]` discipline + match-arm enumeration to catch any
//! future variant addition.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::unnested_or_patterns)]

use benten_drop::{DropBundle, DropContentMode};

// ---------------------------------------------------------------------------
// PIN 1 — A-1: NO `InlineContent` arm in `DropContentMode`.
// ---------------------------------------------------------------------------
// Structural pin: enumerate the allowed Modes (1 = online-pull, 2 =
// offline-Drop). The variant `InlineTiny` MUST NOT exist; the test
// match-statement exhaustively covers the allowed modes and DOES NOT
// have an InlineTiny arm. If a future commit adds InlineTiny, this
// match becomes non-exhaustive → compile fail.
#[test]

fn tf3f_drop_content_mode_no_inline_tiny_arm() {
    // The allowed modes per §3 G-CORE-3 def input-constraints refinement #6 L341:
    let mode_online_pull = DropContentMode::OnlinePull;
    let mode_offline_drop = DropContentMode::OfflineDrop;

    // Exhaustive match against the allowed two modes. If a third arm
    // (InlineTiny) is added in a future commit, this match becomes
    // non-exhaustive and rustc will reject the test file. That IS the
    // defer-to-post-v1 contract.
    fn describe_mode(m: &DropContentMode) -> &'static str {
        match m {
            DropContentMode::OnlinePull => "online-pull",
            DropContentMode::OfflineDrop => "offline-Drop",
            // NO InlineTiny arm; #[non_exhaustive] discipline.
        }
    }

    assert_eq!(describe_mode(&mode_online_pull), "online-pull");
    assert_eq!(describe_mode(&mode_offline_drop), "offline-Drop");
}

// ---------------------------------------------------------------------------
// PIN 2 — Construction of an InlineTiny-shaped bundle is rejected.
// ---------------------------------------------------------------------------
// Runtime defense: if any external path attempts to construct a bundle
// with inline-tiny content (e.g. via a CBOR payload that names the
// unknown Mode-3 variant), the parser MUST reject typed.
#[test]

fn tf3f_inline_tiny_synthetic_bundle_rejected_typed() {
    // Synthetic CBOR with `mode = 3` (InlineTiny).
    let synthetic_inline_cbor = DropBundle::synthesize_inline_tiny_cbor_for_test();
    let parsed = DropBundle::parse_cbor_bytes(&synthetic_inline_cbor);

    use benten_drop::DropBundleError;
    assert!(
        matches!(
            parsed,
            Err(DropBundleError::UnsupportedDropMode { .. })
                | Err(DropBundleError::UnsupportedDropVersion { .. })
        ),
        "Synthetic Mode-3 (InlineTiny) CBOR bundle MUST be rejected at \
         parse time (defer-to-post-v1 contract per §1.A.FROZEN item \
         15(h)). NEVER silently accept. got: {:?}",
        parsed.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}
