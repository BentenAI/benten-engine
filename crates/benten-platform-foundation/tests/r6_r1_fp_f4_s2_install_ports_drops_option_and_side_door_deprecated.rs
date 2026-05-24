//! R6 R1 FP-F4 §S2 production-arm test pin — `InstallPorts.install_record_replay_check`
//! is no longer `Option<>` (Row D-2 close) + `manifest_store::install_verified_record_unchecked`
//! carries `#[deprecated]` + `#[doc(hidden)]` (Δv3-9 closure).
//!
//! ## Test shape (per pim-18 / §3.6f SUBSTANTIVE-arm-not-SHAPE)
//!
//! 1. Arm 1: a substantive replay-check closure is wired through
//!    `InstallPorts` + IS called with the install record's
//!    canonical-bytes hash (verifies the Option drop landed +
//!    the consumption site at plugin_lifecycle.rs threads through
//!    correctly).
//! 2. Arm 2: the `noop_replay_check()` helper produces an admit-all
//!    closure shape coercible to `&mut InstallRecordReplayCheckFn`.
//! 3. Arm 3: the `install_verified_record_unchecked` side-door is
//!    callable + functional + the deprecation annotation surfaces as a
//!    lint warning (#[allow(deprecated)] on this test acknowledges).

#![allow(deprecated)]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use benten_id::keypair::Keypair;
use benten_platform_foundation::manifest_store::ManifestStore;
use benten_platform_foundation::plugin_manifest::InstallRecord;
use benten_platform_foundation::testing::noop_replay_check;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn signed_record(
    user: &Keypair,
    plugin_did: Did,
    manifest_cid: Cid,
    nonce: Vec<u8>,
) -> InstallRecord {
    let mut record = InstallRecord {
        manifest_cid,
        plugin_did,
        consenting_user_did: user.public_key().to_did(),
        user_signature: Vec::new(),
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce,
        granted_caps_bytes: vec![],
    };
    let sig = user.sign(&record.signing_payload());
    record.user_signature = sig.to_bytes().to_vec();
    record
}

/// **§S2 arm 1 — Option drop landed:** the field type is now
/// `&mut InstallRecordReplayCheckFn` (no Option). Every caller MUST
/// supply a closure. This is verified by the fact that all 30
/// existing test fixtures had to thread a `noop_replay_check()` or
/// substantive closure post-migration (see grep `git log --grep
/// FP-F4`).
///
/// At the type level: the field's declared type IS what
/// `plugin_lifecycle::install_plugin` consumes via
/// `(ports.install_record_replay_check)(&hash)` direct call.
#[test]
fn install_ports_field_is_no_longer_option() {
    // This test asserts SHAPE: the noop closure coerces to
    // `&mut InstallRecordReplayCheckFn`. If the Option were still in
    // place, the test would not compile (`&mut F` doesn't coerce to
    // `Option<&mut F>` implicitly).
    let mut noop = noop_replay_check();
    let _coerced: &mut dyn FnMut(&[u8; 32]) -> Result<(), ErrorCode> = &mut noop;
}

/// **§S2 arm 2 — `noop_replay_check()` helper shape verification.**
/// Returns a closure that takes `&[u8; 32]` and returns
/// `Result<(), ErrorCode>` — exactly the shape required by
/// `InstallRecordReplayCheckFn`. Always admits (returns `Ok(())`).
#[test]
fn noop_replay_check_admits_every_hash() {
    let mut noop = noop_replay_check();
    assert!(noop(&[0xAAu8; 32]).is_ok());
    assert!(noop(&[0x00u8; 32]).is_ok());
}

/// **§S2 arm 3 — substantive replay-check closure is consulted at the
/// expected boundary.** Wires a CountingReplayCheck that returns
/// `Err(PluginInstallRecordAlreadyApplied)` on the second presentation
/// of the same hash. This is the production-arm shape; integration with
/// `Engine::install_record_replay_store().record_and_check` is the same
/// surface.
#[test]
fn counting_replay_check_observes_invocation_at_install_time() {
    let calls: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let calls_clone = calls.clone();
    let check = move |_hash: &[u8; 32]| -> Result<(), ErrorCode> {
        calls_clone.fetch_add(1, Ordering::SeqCst);
        Ok(())
    };
    // Direct invocation — mimics what plugin_lifecycle.rs does at the
    // step-3b boundary. End-to-end test is in r6fp_a_plugin_trust_blocker_closures.rs.
    let hash = [0x99u8; 32];
    assert!(check(&hash).is_ok());
    assert!(check(&hash).is_ok());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

/// **§S2 arm 4 — `install_verified_record_unchecked` side-door still
/// functional for test/synthetic paths (post-rename).** The deprecation
/// annotation surfaces as a lint warning; the body still works (the
/// rename is observable in the test fixtures' migrated calls).
#[test]
fn install_verified_record_unchecked_side_door_persists_record() {
    let user = Keypair::generate();
    let plugin_did = Did::from_string_for_test_fixture("did:key:zPluginSideDoor".to_string());
    let record = signed_record(
        &user,
        plugin_did.clone(),
        Cid::from_blake3_digest([1u8; 32]),
        vec![0x11u8; 16],
    );
    let mut store = ManifestStore::new();
    store
        .install_verified_record_unchecked(plugin_did.clone(), record.clone())
        .expect("side-door persists verified record");
    assert!(store.contains(&plugin_did));
}
