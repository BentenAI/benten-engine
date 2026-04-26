#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: STREAM persist option (phil-r1-1) (G6-A).
//!
//! Pin source: phil-r1-1 — opt-in `stream({ persist: true })` materializes
//! aggregate Node at completion; aggregate Node CID is content-addressed
//! over chunk concatenation. Locked into G6-A files-owned per plan §3.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::primitives::stream::{StreamPersistMode, StreamPrimitiveSpec};
use benten_eval::testing::{testing_collect_stream_aggregate_node, testing_run_stream_persist};

/// `stream({ persist: true })` materializes an aggregate Node at completion
/// containing every chunk. Default-ephemeral STREAMs do NOT.
#[test]
#[ignore = "Phase 2b G6-A pending — phil-r1-1 STREAM persist materialization"]
fn stream_persist_true_materializes_aggregate_node() {
    let chunks: Vec<Vec<u8>> = vec![b"hello ".to_vec(), b"world".to_vec(), b"!".to_vec()];

    let spec = StreamPrimitiveSpec {
        persist: StreamPersistMode::Persist,
        ..StreamPrimitiveSpec::default()
    };

    let outcome = testing_run_stream_persist(spec, chunks.clone());
    let aggregate_cid = outcome
        .aggregate_node_cid
        .expect("persist:true must materialize an aggregate Node CID");

    let node = testing_collect_stream_aggregate_node(&aggregate_cid)
        .expect("aggregate Node must be readable from the graph");
    assert_eq!(node.chunk_count(), chunks.len());

    // Default (ephemeral) variant materializes nothing.
    let ephemeral_spec = StreamPrimitiveSpec::default();
    let eph = testing_run_stream_persist(ephemeral_spec, chunks);
    assert!(
        eph.aggregate_node_cid.is_none(),
        "default ephemeral STREAM must NOT materialize an aggregate Node"
    );
}

/// Aggregate Node CID is content-addressed over the chunk-byte concatenation;
/// two STREAMs that emit the same chunk-byte sequence (regardless of timing)
/// produce the same CID. phil-r1-1 stability pin.
#[test]
#[ignore = "Phase 2b G6-A pending — phil-r1-1 CID-stability"]
fn stream_persist_aggregate_node_cid_is_content_addressed_over_chunk_concatenation() {
    let chunks_a: Vec<Vec<u8>> = vec![b"abc".to_vec(), b"def".to_vec()];
    let chunks_b: Vec<Vec<u8>> = vec![b"abc".to_vec(), b"def".to_vec()];
    let chunks_c: Vec<Vec<u8>> = vec![b"abcdef".to_vec()];

    let spec = StreamPrimitiveSpec {
        persist: StreamPersistMode::Persist,
        ..StreamPrimitiveSpec::default()
    };

    let cid_a = testing_run_stream_persist(spec.clone(), chunks_a)
        .aggregate_node_cid
        .unwrap();
    let cid_b = testing_run_stream_persist(spec.clone(), chunks_b)
        .aggregate_node_cid
        .unwrap();
    let cid_c = testing_run_stream_persist(spec, chunks_c)
        .aggregate_node_cid
        .unwrap();

    assert_eq!(
        cid_a, cid_b,
        "identical chunk-byte sequence → identical aggregate CID"
    );
    // The phil-r1-1 recommendation is content-addressed-over-concatenation,
    // i.e. chunk boundaries are NOT part of the CID; (b'abc' + b'def') ≡
    // (b'abcdef'). G6-A brief MAY refine this to chunk-boundary-aware
    // CID; this assertion is the spec-pinned shape per R1 phil-r1-1.
    assert_eq!(
        cid_a, cid_c,
        "aggregate CID is over chunk-byte concatenation, not chunk boundaries"
    );
}
