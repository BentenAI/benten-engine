//! S-3 closure — frozen const VALUE pins for `benten-engine` (Layer-D + manifest).
//!
//! The required `cargo-public-api` baseline records const TYPES only, so a
//! value-only edit to any wire version byte, reserved codepoint band edge, or
//! decode bound below is an EMPTY baseline diff.

use benten_engine::engine_config::ENGINE_TOML_WALLCLOCK_MAX_HARD_CAP;
use benten_engine::engine_sync::DeviceAttestationEnvelope;
use benten_engine::handler_versions::{
    HANDLER_ID_PROPERTY, HANDLER_VERSION_LABEL, PREDECESSOR_CID_PROPERTY, SEQUENCE_PROPERTY,
    VERSION_CID_PROPERTY,
};
use benten_engine::layer_d::device_link::{
    DEVICE_LINK_BAND_BASE, DEVICE_LINK_BAND_END, PROVISIONING_WIRE_VERSION,
};
use benten_engine::layer_d::drop_timestamp::LAYER_D_BUCKET_SECS;
use benten_engine::layer_d::remote_permission::{
    AAD_VERSION, EXEC_WORKFLOW_AAD_DOMAIN, GRANT_DOMAIN, REMOTE_PERMISSION_BAND_BASE,
    REMOTE_PERMISSION_BAND_END, REMOTE_PERMISSION_WIRE_VERSION, REQUEST_DOMAIN,
};
use benten_engine::module_manifest::{
    MAX_MODULE_MANIFEST_BYTES, MAX_MODULE_MANIFEST_MIGRATIONS, MAX_MODULE_MANIFEST_MODULES,
};

// ---------------------------------------------------------------------------
// Group 1 — remote-permission signature + AAD domains.
//
// WHAT BREAKS: these are signature-preimage and AAD prefixes for the
// cross-device permission protocol. A change silently invalidates every
// outstanding request/grant, and because both sides derive from the same
// constant, no in-process round-trip test can observe it.
// ---------------------------------------------------------------------------

#[test]
fn remote_permission_domains_are_frozen() {
    assert_eq!(
        REQUEST_DOMAIN, b"benten-remote-permission-request-v2:",
        "remote-permission request signature domain"
    );
    assert_eq!(
        GRANT_DOMAIN, b"benten-remote-permission-grant-v2:",
        "remote-permission grant signature domain"
    );
    assert_eq!(
        EXEC_WORKFLOW_AAD_DOMAIN, b"benten-exec-workflow-v1:",
        "exec-workflow AAD domain — a change invalidates existing AEAD tags"
    );
    assert_eq!(
        AAD_VERSION, 0x01,
        "remote-permission AAD version byte, committed into the tag"
    );
}

// ---------------------------------------------------------------------------
// Group 2 — Layer-D wire versions + reserved codepoint bands.
//
// WHAT BREAKS: cross-device protocol negotiation. The band edges reserve
// 16-codepoint windows in the central registry; moving an edge collides with a
// neighbouring band.
// ---------------------------------------------------------------------------

#[test]
fn layer_d_wire_versions_and_bands_are_frozen() {
    assert_eq!(
        REMOTE_PERMISSION_WIRE_VERSION, 2,
        "remote-permission wire version byte"
    );
    assert_eq!(REMOTE_PERMISSION_BAND_BASE, 0x6320, "band base");
    assert_eq!(REMOTE_PERMISSION_BAND_END, 0x632F, "band end");

    assert_eq!(
        PROVISIONING_WIRE_VERSION, 2,
        "device-link provisioning wire version byte"
    );
    assert_eq!(DEVICE_LINK_BAND_BASE, 0x6310, "band base");
    assert_eq!(DEVICE_LINK_BAND_END, 0x631F, "band end");

    assert_eq!(
        LAYER_D_BUCKET_SECS, 3_600,
        "Layer-D timestamp bucket (s) — must match benten_sync::handshake::LAYER_D_BUCKET_SECS"
    );
}

// ---------------------------------------------------------------------------
// Group 3 — device attestation envelope.
//
// WHAT BREAKS: WIRE_VERSION / MAX_WIRE_VERSION gate acceptance of a signed
// envelope from another device; MAX_ENVELOPE_BYTES bounds a decode of
// remote-supplied bytes.
// ---------------------------------------------------------------------------

#[test]
fn device_attestation_envelope_constants_are_frozen() {
    assert_eq!(
        DeviceAttestationEnvelope::WIRE_VERSION,
        2,
        "emitted device-attestation envelope wire version"
    );
    assert_eq!(
        DeviceAttestationEnvelope::MAX_WIRE_VERSION,
        2,
        "highest accepted envelope wire version — raising it accepts unvalidated future shapes"
    );
    assert_eq!(
        DeviceAttestationEnvelope::MAX_ENVELOPE_BYTES,
        4 * 1024 * 1024,
        "decode cap over remote-supplied envelope bytes"
    );
    assert_eq!(
        DeviceAttestationEnvelope::MAX_ENVELOPE_BYTES,
        4_194_304,
        "literal value"
    );
}

// ---------------------------------------------------------------------------
// Group 4 — module-manifest decode bounds (META #629).
// ---------------------------------------------------------------------------

#[test]
fn module_manifest_decode_bounds_are_frozen() {
    assert_eq!(
        MAX_MODULE_MANIFEST_BYTES,
        256 * 1024,
        "module-manifest byte cap over untrusted input"
    );
    assert_eq!(MAX_MODULE_MANIFEST_BYTES, 262_144, "literal value");
    assert_eq!(
        MAX_MODULE_MANIFEST_MODULES, 4_096,
        "module-count cap — bounds an input-driven loop"
    );
    assert_eq!(
        MAX_MODULE_MANIFEST_MIGRATIONS, 4_096,
        "migration-count cap — bounds an input-driven loop"
    );
    assert_eq!(
        ENGINE_TOML_WALLCLOCK_MAX_HARD_CAP,
        60 * 60 * 1000,
        "hard ceiling on the config-supplied wallclock max (ms)"
    );
    assert_eq!(
        ENGINE_TOML_WALLCLOCK_MAX_HARD_CAP, 3_600_000,
        "literal value (ms)"
    );
}

// ---------------------------------------------------------------------------
// Group 5 — handler-version graph vocabulary (hashed into CIDs).
// ---------------------------------------------------------------------------

#[test]
fn handler_version_vocabulary_is_frozen() {
    assert_eq!(HANDLER_VERSION_LABEL, "system:HandlerVersion");
    assert_eq!(HANDLER_ID_PROPERTY, "handler_id");
    assert_eq!(VERSION_CID_PROPERTY, "version_cid");
    assert_eq!(PREDECESSOR_CID_PROPERTY, "predecessor_cid");
    assert_eq!(SEQUENCE_PROPERTY, "seq");
}
