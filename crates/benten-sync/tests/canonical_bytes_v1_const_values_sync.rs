//! S-3 closure — frozen const VALUE pins for `benten-sync`.
//!
//! The two ALPN byte strings are the literal protocol identifiers negotiated in
//! the QUIC/TLS handshake. A change means peers on the old build and peers on
//! the new build cannot connect at all — and nothing in the tree asserts their
//! value (`ATRIUM_ALPN` and `UCAN_BLOBS_ALPN` have ZERO occurrences under any
//! `crates/*/tests/` directory).

use benten_sync::handshake::{
    DEFAULT_REPLAY_WINDOW_MS, LAYER_D_BUCKET_SECS, MAX_HANDSHAKE_PAYLOAD_BYTES,
};
use benten_sync::handshake_wire::{HANDSHAKE_WIRE_VERSION, MAX_HANDSHAKE_FRAME_BYTES};
use benten_sync::mst_proto::{
    MAX_MST_FRAME_BYTES, MAX_MST_MESSAGE_BYTES, MAX_MST_MESSAGES_PER_FRAME, MST_DIFF_WIRE_VERSION,
};
use benten_sync::peer_id::MAX_PEER_ID_CBOR_BYTES;
use benten_sync::transport::ATRIUM_ALPN;
use benten_sync::ucan_blobs_protocol::UCAN_BLOBS_ALPN;

// ---------------------------------------------------------------------------
// Group 1 — ALPN protocol identifiers.
//
// WHAT BREAKS: peer connectivity, totally and silently. ALPN mismatch is a
// handshake-time rejection, not a data error, so no golden/round-trip test can
// observe it in-process.
// ---------------------------------------------------------------------------

#[test]
fn alpn_protocol_identifiers_are_frozen() {
    assert_eq!(
        ATRIUM_ALPN, b"benten/atrium/1",
        "Atrium ALPN — a change makes new peers unable to connect to existing peers"
    );
    assert_eq!(
        UCAN_BLOBS_ALPN, b"benten/ucan-blobs/1",
        "UCAN-gated blobs ALPN — a change breaks blob transfer negotiation"
    );
}

// ---------------------------------------------------------------------------
// Group 2 — wire version bytes.
// ---------------------------------------------------------------------------

#[test]
fn sync_wire_version_bytes_are_frozen() {
    assert_eq!(
        MST_DIFF_WIRE_VERSION, 1,
        "MST diff protocol wire version byte"
    );
    assert_eq!(HANDSHAKE_WIRE_VERSION, 1, "DID handshake wire version byte");
}

// ---------------------------------------------------------------------------
// Group 3 — frame decode bounds over untrusted peer input (META #629).
//
// WHAT BREAKS: these bound allocation driven by bytes a REMOTE PEER controls.
// Widening any of them re-opens a remote memory-exhaustion DoS, and the
// existing tests construct their oversize fixtures from the constants.
// ---------------------------------------------------------------------------

#[test]
fn sync_frame_decode_bounds_are_frozen() {
    assert_eq!(
        MAX_MST_FRAME_BYTES,
        4 * 1024 * 1024,
        "MST frame cap over remote-controlled bytes"
    );
    assert_eq!(MAX_MST_FRAME_BYTES, 4_194_304, "literal value");
    assert_eq!(MAX_MST_MESSAGE_BYTES, 1024 * 1024, "per-MST-message cap");
    assert_eq!(MAX_MST_MESSAGE_BYTES, 1_048_576, "literal value");
    assert_eq!(
        MAX_MST_MESSAGES_PER_FRAME, 4_096,
        "message-count cap per frame — bounds a remote-driven loop"
    );
    assert_eq!(
        MAX_HANDSHAKE_FRAME_BYTES,
        4 * 1024 * 1024,
        "handshake frame cap"
    );
    assert_eq!(MAX_HANDSHAKE_FRAME_BYTES, 4_194_304, "literal value");
    assert_eq!(
        MAX_HANDSHAKE_PAYLOAD_BYTES,
        256 * 1024,
        "handshake payload cap"
    );
    assert_eq!(MAX_HANDSHAKE_PAYLOAD_BYTES, 262_144, "literal value");
    assert_eq!(
        MAX_PEER_ID_CBOR_BYTES,
        4 * 1024,
        "peer-id CBOR cap over remote-controlled bytes"
    );
    assert_eq!(MAX_PEER_ID_CBOR_BYTES, 4_096, "literal value");
}

// ---------------------------------------------------------------------------
// Group 4 — replay / bucketing windows.
//
// WHAT BREAKS: DEFAULT_REPLAY_WINDOW_MS is the anti-replay acceptance window —
// widening it weakens replay defence with no visible test effect.
// LAYER_D_BUCKET_SECS is duplicated in benten-engine (layer_d::drop_timestamp);
// the two must agree or timestamp buckets disagree across the boundary.
// ---------------------------------------------------------------------------

#[test]
fn sync_replay_and_bucket_windows_are_frozen() {
    assert_eq!(
        DEFAULT_REPLAY_WINDOW_MS, 5_000,
        "handshake anti-replay window (ms) — widening weakens replay defence"
    );
    assert_eq!(
        LAYER_D_BUCKET_SECS, 3_600,
        "Layer-D timestamp bucket (s) — must match benten_engine::layer_d::drop_timestamp"
    );
}
