//! Criterion benchmark: parse raw CID bytes back into a `Cid` via
//! [`Cid::from_bytes`].
//!
//! **Target source:** non-§14.6 — informational. `Cid::from_bytes` is on the
//! napi-rs boundary path (TypeScript passes CIDs as 36-byte buffers, Rust
//! validates + wraps) and on any sync-protocol path where CIDs arrive over
//! the wire; a slow validator shows up on both hot paths. ENGINE-SPEC §14.6
//! does not tabulate a parse target, so this bench reports a trend line
//! rather than a gate.
//!
//! **Informational reason:** §14.6 was written around engine-level
//! throughput (lookup, create, view read, handler eval); parse is a
//! substep of lookup-by-CID and its cost is already amortized into the
//! "Node lookup by ID: 1-50µs" direct gate. A dedicated parse gate would
//! double-count. Keeping this bench informational lets us watch the trend
//! without gating on a number that isn't in the spec.
//!
//! ## Why `from_bytes`, not `from_str`
//!
//! `Cid::from_str` is `todo!`-style stubbed in Phase 1 — it returns
//! `CoreError::CidParse` unconditionally because the base32 multibase
//! decoder lands with the `cid`-crate migration (C4, Phase 2). Running a
//! bench against it would `.expect("parse")`-panic on every iteration.
//! `from_bytes` exercises the equivalent binary validation path that IS
//! live in Phase 1 (length check, version byte, multicodec, multihash,
//! digest length) and produces stable, meaningful numbers.
//!
//! When `Cid::from_str` lands in Phase 2, add a sibling
//! `bench_cid_from_str` measurement in this file so the informational
//! trend covers both paths.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;

use benten_core::Cid;
use benten_core::testing::canonical_test_node;
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_cid_parse(c: &mut Criterion) {
    // Pre-compute the canonical CID bytes so the measurement isolates
    // the parse / validation step (not the hash step).
    let cid = canonical_test_node().cid().expect("hash");
    let cid_bytes: Vec<u8> = cid.as_bytes().to_vec();

    let mut group = c.benchmark_group("cid_parse");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("from_bytes_roundtrip", |b| {
        b.iter(|| {
            // Exercises the Phase-1 binary validation path: length, version,
            // multicodec, multihash, digest length. Infallible for the
            // canonical fixture so the timing isolates the validator cost.
            let parsed = Cid::from_bytes(black_box(cid_bytes.as_slice())).expect("parse");
            black_box(parsed);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_cid_parse);
criterion_main!(benches);
