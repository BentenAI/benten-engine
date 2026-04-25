//! `inspect-state` subcommand — pretty-print a suspended
//! [`benten_eval::ExecutionStateEnvelope`].
//!
//! Reads DAG-CBOR envelope bytes (typically produced by
//! `Engine::suspend_to_bytes` or surfaced by an SDK suspension handle)
//! and renders a human-readable report covering:
//!
//! - schema version
//! - envelope CID
//! - payload CID (the structural id stamped into the envelope)
//! - resumption principal CID
//! - frame stack (depth + per-frame tag)
//! - pinned subgraph CID list (the subgraphs the resume re-checks
//!   against the registered handler set)
//! - attribution chain length
//! - context-binding-snapshot count
//!
//! The renderer is deliberately a pure function over bytes so it's
//! callable from the CLI binary, from a future LSP-style "explain this
//! suspension" surface, and from integration tests.

use benten_errors::ErrorCode;
use benten_eval::ExecutionStateEnvelope;

/// Render a DAG-CBOR `ExecutionStateEnvelope` as a multi-line debug
/// string. Returns the formatted text.
///
/// # Errors
/// Returns `Err(ErrorCode::Unknown(...))` if the bytes do not decode as
/// a valid envelope or the embedded payload CID does not match the
/// recomputed CID (envelope-tampering signal).
pub fn pretty_print_envelope_bytes(bytes: &[u8]) -> Result<String, ErrorCode> {
    let env = ExecutionStateEnvelope::from_dagcbor(bytes)
        .map_err(|e| ErrorCode::Unknown(format!("inspect_state_decode: {e}")))?;

    let envelope_cid = env
        .envelope_cid()
        .map_err(|e| ErrorCode::Unknown(format!("inspect_state_envelope_cid: {e}")))?;
    let recomputed_payload_cid = env
        .recompute_payload_cid()
        .map_err(|e| ErrorCode::Unknown(format!("inspect_state_payload_cid: {e}")))?;

    if recomputed_payload_cid != env.payload_cid {
        return Err(ErrorCode::Unknown(
            "inspect_state: payload CID mismatch — envelope appears tampered".into(),
        ));
    }

    let payload = &env.payload;

    use std::fmt::Write as _;
    let mut out = String::new();
    out.push_str("benten-dev inspect-state\n");
    out.push_str("------------------------\n");
    writeln!(out, "schema_version       : {}", env.schema_version).expect("string write");
    writeln!(out, "envelope_cid         : {envelope_cid}").expect("string write");
    writeln!(out, "payload_cid          : {}", env.payload_cid).expect("string write");
    writeln!(
        out,
        "resumption_principal : {}",
        payload.resumption_principal_cid
    )
    .expect("string write");
    writeln!(out, "frame_stack_depth    : {}", payload.frame_stack.len()).expect("string write");
    writeln!(out, "frame_index          : {}", payload.frame_index).expect("string write");
    writeln!(
        out,
        "pinned_subgraph_cids : {}",
        payload.pinned_subgraph_cids.len()
    )
    .expect("string write");
    writeln!(
        out,
        "attribution_chain    : {} frame(s)",
        payload.attribution_chain.len()
    )
    .expect("string write");
    writeln!(
        out,
        "context_bindings     : {} snapshot(s)",
        payload.context_binding_snapshots.len()
    )
    .expect("string write");

    if !payload.frame_stack.is_empty() {
        out.push_str("\nframes:\n");
        for (i, frame) in payload.frame_stack.iter().enumerate() {
            let marker = if i == payload.frame_index { ">" } else { " " };
            writeln!(out, "  {marker} [{i:>3}] {}", frame.tag).expect("string write");
        }
    }

    if !payload.pinned_subgraph_cids.is_empty() {
        out.push_str("\npinned subgraphs:\n");
        for cid in &payload.pinned_subgraph_cids {
            writeln!(out, "  - {cid}").expect("string write");
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use benten_core::Cid;
    use benten_eval::{ExecutionStatePayload, Frame};

    fn synth_envelope() -> ExecutionStateEnvelope {
        let mut payload = ExecutionStatePayload::new_with_pinned(vec![
            Cid::from_blake3_digest([0xaa; 32]),
            Cid::from_blake3_digest([0xbb; 32]),
        ]);
        payload.frame_stack = vec![Frame::root(), Frame::root()];
        payload.frame_index = 1;
        payload.resumption_principal_cid = Cid::from_blake3_digest([0xcc; 32]);
        ExecutionStateEnvelope::new(payload).expect("envelope")
    }

    #[test]
    fn pretty_prints_round_tripped_envelope() {
        let env = synth_envelope();
        let bytes = env.to_dagcbor().unwrap();
        let rendered = pretty_print_envelope_bytes(&bytes).expect("pretty");
        assert!(rendered.contains("benten-dev inspect-state"));
        assert!(rendered.contains("schema_version"));
        assert!(rendered.contains("frame_stack_depth    : 2"));
        assert!(rendered.contains("pinned_subgraph_cids : 2"));
        assert!(rendered.contains("> [  1]"), "frame_index marker present");
    }

    #[test]
    fn rejects_corrupt_bytes() {
        assert!(pretty_print_envelope_bytes(&[0u8, 1, 2, 3]).is_err());
    }
}
