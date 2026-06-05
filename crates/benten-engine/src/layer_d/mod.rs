//! Layer-D — DAK + device-auth + remote-permission-call + multi-device
//! key-wrap + `ExecuteWorkflow` reserve (e2r §§2,6,7,8; R0.7 §3.4).
//!
//! Layer-D is the device-authentication + cross-device-permission substrate of
//! the F-full encryption stack:
//!
//! - [`device_auth`] — the `DeviceAuthBackend` sealed trait (CLAUDE.md #7) +
//!   the Benten-vended headless Argon2id+XChaCha20-Poly1305 default
//!   ([`BENTEN_VAULT_PASSWORD`](device_auth::BENTEN_VAULT_PASSWORD) headless
//!   unlock; F-VA-3 constant-time wrong-password path);
//! - [`remote_permission`] — the `PermissionRequest`/`PermissionGrant` wire
//!   protocol (codepoint band `0x6320..0x632F`) incl. the `ExecuteWorkflow`
//!   CODEPOINT-RESERVE (M-3);
//! - [`device_link`] — multi-device key-wrap-on-device-link (codepoint band
//!   `0x6310..0x631F`); the `Provisioning*` structs HPKE-wrap `K_principal` to
//!   device B's fresh hybrid keypair (reuses the Layer-C HPKE primitive,
//!   Inv-16);
//! - [`grant_acceptance`] — the §6.7 / §9.1-4 six-class grant-acceptance
//!   freeze-gate harness (the pre-merge security mini-review reads this);
//! - [`secret_store`] — the `keyring-core` + file-vault DAK-wrap store;
//! - [`drop_timestamp`] — the Layer-D timestamp postures (drops carry NO
//!   timestamp; DeviceLink/RemotePermission carry the 1-hr bucket; round-down
//!   NO jitter — NQ-C5).
//!
//! # HPKE reuse (Inv-16)
//!
//! The device-link key-wrap + the remote-permission HPKE-wrapped key material
//! both route through the unified Layer-C/Layer-D HPKE primitive
//! ([`benten_crypto_suite::hpke::wrap_key_to_recipient`]) — one KEM-DEM impl,
//! not two.

pub mod device_auth;
pub mod device_link;
pub mod drop_timestamp;
pub mod grant_acceptance;
pub mod remote_permission;
pub mod secret_store;
