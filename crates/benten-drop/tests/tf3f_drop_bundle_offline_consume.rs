//! TF-3f pins — G-CORE-3f Drop bundle offline-consume round-trip.
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition.
//!
//! **SPEC-GAP SURFACED:** the eventual destination for this test is the
//! `benten-drop` NEW crate (per `00-implementation-plan.md` §3 G-CORE-3f
//! wave def: "NEW `benten-drop` crate (offline-bundle format; mode 2
//! sendme→Drop ships; mode 3 inline-tiny defers to post-v1) ~600 LOC.
//! Files: `crates/benten-drop/*` (NEW crate)"). At origin/main `c9c11c56`
//! the `benten-drop` crate does NOT exist yet (verified: `ls
//! crates/benten-drop` returns nothing). Per R3-BRIEF-common.md §"Return
//! contract" item 4, this test lands in a sibling crate's `tests/` dir
//! (`benten-sync` — selected as the closest sibling because sync owns
//! the online-share path that Drop is the offline-companion to) until
//! the `benten-drop` crate is created at G-CORE-3f. R3 author surfaces
//! this as a spec-gap; R5 G-CORE-3f implementer should relocate this
//! file to `crates/benten-drop/tests/tf3f_drop_bundle_offline_consume.rs`
//! when the crate is created.
//!
//! Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3f (P-1):
//!     "Drop bundle construction (Spike G): 5-Recipe SubgraphSpec → CBOR-
//!     on-disk envelope `DropBundle{version, spec_cid, auth_grant,
//!     content: Vec<EncryptedNode>, key_material, manifest}` ≈ 2688 bytes
//!     (assert ≤4 KiB upper bound)."
//!   - §2 G-CORE-3f (P-2): "Offline consume: read DropBundle from
//!     filesystem; no network; verify envelope-sig → verify per-Node-sigs
//!     → derive keys → decrypt Nodes. End-to-end matches Spike H's
//!     offline scenario."
//!   - §2 G-CORE-3f (F-3): "DropBundle version mismatch (future version)
//!     → typed `UnsupportedDropVersion`; never silent skip."
//!   - `00-implementation-plan.md` §3 G-CORE-3 def input-constraints
//!     refinement #6 L341: Three sendme deployment modes (online-pull /
//!     offline-Drop / inline-tiny — ship Mode 2 in G-CORE-3f, defer Mode 3
//!     to post-v1).
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-3f (pim-12 / §3.6e).
//! ============================================================================

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::map_unwrap_or)]

use benten_id::keypair::Keypair;
// RED-PHASE failure points — the `benten-drop` crate doesn't exist at
// `c9c11c56`. When G-CORE-3f creates the crate, this `use` resolves.
use benten_drop::{
    DROP_BUNDLE_MAX_SIZE_BYTES, DropBundle, DropBundleError, DropBundleVersion, EncryptedContent,
};

// ---------------------------------------------------------------------------
// PIN 1 — P-1: 5-Recipe bundle ≤4 KiB CBOR-on-disk.
// ---------------------------------------------------------------------------
// Production-arm. The Drop bundle CBOR-on-disk envelope for a 5-Recipe
// SubgraphSpec MUST encode to ≤4 KiB (Spike G measurement: ~2688 bytes).
//
// Would-FAIL-IF-NO-OP'd: a stub that emits an empty bundle passes
// vacuously; this pin asserts the bundle is non-trivial (contains the
// 5 Recipe ciphertexts) AND fits the upper bound.
#[test]

fn tf3f_dropbundle_5_recipe_bundle_within_size_envelope() {
    let kp_alice = Keypair::generate();
    let bundle = DropBundle::build_5_recipe_bundle_for_test(&kp_alice);
    let cbor = bundle.to_cbor_bytes().expect("cbor encode OK");

    // Substantive content: 5 Recipe ciphertexts present (not vacuous).
    assert_eq!(
        bundle.content_count(),
        5,
        "5-Recipe bundle MUST have exactly 5 EncryptedContent entries; \
         a vacuous empty bundle is rejected (pim-18 SHAPE-not-SUBSTANCE)."
    );

    // Size bound: ≤4 KiB (Spike G: ~2688 bytes).
    assert!(
        cbor.len() <= DROP_BUNDLE_MAX_SIZE_BYTES,
        "5-Recipe DropBundle MUST encode to ≤{} bytes (Spike G measured \
         ~2688 bytes for this fixture). Got {} bytes — possible \
         envelope bloat or non-canonical CBOR.",
        DROP_BUNDLE_MAX_SIZE_BYTES,
        cbor.len()
    );

    // Anti-empty-stub: ≥1 KiB (5 ciphertexts + envelope-sig must be
    // non-trivial).
    assert!(
        cbor.len() >= 1024,
        "5-Recipe bundle MUST be ≥1 KiB (5 EncryptedContent entries + \
         envelope-sig). A stub that returns an empty CBOR object would \
         trip this assertion."
    );
}

// ---------------------------------------------------------------------------
// PIN 2 — P-2: Offline consume round-trip (no network).
// ---------------------------------------------------------------------------
// Production-arm. Read DropBundle from filesystem; verify envelope-sig;
// verify per-Node-sigs; derive keys from carried canonical-path; decrypt
// Nodes. Matches Spike H's offline scenario.
//
// Would-FAIL-IF-NO-OP'd: a stub that does NOT actually decrypt (returns
// empty Node list) trips the "≥5 Recipes recovered" assertion.
#[test]

fn tf3f_dropbundle_offline_consume_round_trips_5_recipes() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();

    // Alice produces bundle for Bob.
    let bundle = DropBundle::build_5_recipe_bundle_for_recipient(&kp_alice, &kp_bob);
    let cbor = bundle.to_cbor_bytes().unwrap();

    // Persist as in-memory bytes, simulating Drop-via-USB-stick.
    // (Filesystem round-trip exercised in the eventual `benten-drop`
    // crate's own tests once that crate exists; here we exercise the
    // CBOR-level round-trip which is the load-bearing offline-consume
    // property.)
    let bundle_bytes_on_disk: Vec<u8> = cbor.clone();

    // Bob reads from in-memory bytes (post-Drop transfer); no network.
    let bundle_read = DropBundle::parse_cbor_bytes(&bundle_bytes_on_disk).expect("read OK");

    // Bob consumes offline: verify envelope-sig → verify per-Node-sigs
    // → derive keys → decrypt.
    let recovered = bundle_read
        .consume_offline(&kp_bob)
        .expect("offline consume OK");

    assert_eq!(
        recovered.len(),
        5,
        "Offline consume MUST recover all 5 Recipe Nodes. A stub returning \
         empty trips this. (Pim-18 SHAPE-not-SUBSTANCE.)"
    );
}

// ---------------------------------------------------------------------------
// PIN 3 — F-3: future DropBundle version yields typed UnsupportedDropVersion.
// ---------------------------------------------------------------------------
// Forward-compat: a bundle whose version discriminator is a future
// unknown value MUST yield typed `UnsupportedDropVersion`. Never silent
// skip — a silent skip would let a malicious "future-version" bundle
// be ignored without warning.
#[test]

fn tf3f_dropbundle_future_version_yields_typed_unsupported_drop_version() {
    let bundle_with_future_version =
        DropBundle::synthesize_future_version_for_test(DropBundleVersion::Synthetic(0xFFFF));
    let cbor = bundle_with_future_version.to_cbor_bytes().unwrap();

    let parsed = DropBundle::parse_cbor_bytes(&cbor);
    assert!(
        matches!(parsed, Err(DropBundleError::UnsupportedDropVersion { .. })),
        "Future / unknown DropBundle version MUST yield typed \
         UnsupportedDropVersion. NEVER silent skip. got: {:?}",
        parsed.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}
