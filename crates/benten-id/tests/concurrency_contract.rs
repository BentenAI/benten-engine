//! Safe-4 #642 — compile-time `Send + Sync` pins for the public
//! `benten-id` surface + the `GrantReader` trait's explicit bound.
//!
//! `static_assertions` is intentionally NOT a dependency of this
//! crate (the dep surface is kept minimal per the Cargo.toml
//! rationale). The stable-Rust equivalent of `assert_impl_all!` is a
//! generic fn bounded `T: Send + Sync` instantiated at each type — if
//! a future refactor adds a non-thread-safe interior (`Rc`,
//! `RefCell`, raw thread-unsafe handle) to any of these types, this
//! test fails to compile loudly.
//!
//! See `crates/benten-id/INTERNALS.md` §4a "Concurrency contract" for
//! the interior-synchronized vs caller-synchronized split this pins.

use benten_id::device_attestation::DeviceAttestation;
use benten_id::did::Did;
use benten_id::did_rotation::{RotationAttestation, RotationLog};
use benten_id::grant_reader::{GrantReader, ReaderError};
use benten_id::keypair::Keypair;
use benten_id::plugin_did::{PluginDidHandle, PluginDidStore};
use benten_id::ucan::Ucan;
use benten_id::vc::{Credential, RevocationRegistry, TrustDomain};

fn assert_send_sync<T: Send + Sync>() {}

/// Object-safe `GrantReader` trait objects must be `Send + Sync`
/// (the trait declares the bound; this pins that a `dyn GrantReader`
/// honors it so cross-thread policy backends compile).
fn assert_grant_reader_obj_send_sync<R: GrantReader + 'static>() {
    assert_send_sync::<R>();
    fn needs_send_sync<T: Send + Sync + ?Sized>() {}
    needs_send_sync::<dyn GrantReader>();
}

#[test]
fn public_surface_is_send_sync() {
    assert_send_sync::<Keypair>();
    assert_send_sync::<Did>();
    assert_send_sync::<Ucan>();
    assert_send_sync::<Credential>();
    assert_send_sync::<RotationAttestation>();
    assert_send_sync::<RotationLog>();
    assert_send_sync::<DeviceAttestation>();
    assert_send_sync::<PluginDidHandle>();
    assert_send_sync::<PluginDidStore>();
    assert_send_sync::<RevocationRegistry>();
    assert_send_sync::<TrustDomain>();
    assert_send_sync::<ReaderError>();

    // The GrantReader trait's explicit `Send + Sync` supertrait bound:
    // pin that `dyn GrantReader` is itself `Send + Sync` so a
    // cross-thread cap-policy backend (Arc<dyn GrantReader>) compiles.
    fn dyn_grant_reader_is_send_sync<T: Send + Sync + ?Sized>() {}
    dyn_grant_reader_is_send_sync::<dyn GrantReader>();

    // Reference the helper so it is not dead (exercises the generic
    // bound path on a concrete impl is unnecessary — the dyn pin
    // above is the load-bearing assertion).
    let _ = assert_grant_reader_obj_send_sync::<DummyReader>;
}

struct DummyReader;
impl GrantReader for DummyReader {
    fn has_unrevoked_grant_for_scope(&self, _scope: &str) -> Result<bool, ReaderError> {
        Ok(true)
    }
    fn has_unrevoked_grant_for_grant_cid(
        &self,
        _grant_cid: &benten_core::Cid,
    ) -> Result<bool, ReaderError> {
        Ok(true)
    }
}
