//! Criterion benchmark: `ErrorCode` catalog hot-path baselines.
//!
//! This bench is **informational, not gated** (same disposition as
//! `benten-core`'s `hash_only` baseline). It exists to make the
//! "minimal-allocation catalog" design intent (per INTERNALS.md §1)
//! verifiable at runtime rather than only asserted in prose (Fwd-1 #939),
//! and to track the `from_str` length-bucketed-memcmp cost as the catalog
//! grows toward the Phase-5+ >500-variant watch threshold (Fwd-1 #944).
//!
//! **Target source:** non-budget — there is no ENGINE-SPEC latency target
//! for catalog stringification because it is not on a hot path today
//! (errors are exceptional). The value is the regression delta: if a
//! future refactor turns `as_static_str` into anything other than a jump
//! table, or `from_str` into anything worse than the current
//! length-bucketed memcmp chain, the delta surfaces here.
//!
//! ## Why informational
//!
//! `as_static_str` is a pure `match` over unit variants (the compiler
//! lowers it to a jump table — zero allocation). `from_str` is a 168-arm
//! string match (length-bucketed memcmp, ~150ns/call per #944). Neither
//! is in any §14.6 budget; CI must not fail on a swing here, but the
//! delta is worth watching as the catalog grows.
//!
//! ## Run
//!
//! ```ignore
//! cargo bench -p benten-errors --bench catalog_baseline
//! ```

use std::hint::black_box;
use std::str::FromStr;

use benten_errors::ErrorCode;
use criterion::{Criterion, criterion_group, criterion_main};

/// A spread of variants across the enum-declaration order so the
/// length-bucketed memcmp chain in `from_str` is exercised across short,
/// medium and long catalog strings rather than only the first few arms.
const SAMPLE: &[ErrorCode] = &[
    ErrorCode::InvCycle,
    ErrorCode::CapDenied,
    ErrorCode::WriteConflict,
    ErrorCode::WaitTimeout,
    ErrorCode::SandboxHostFnDenied,
    ErrorCode::StreamBackpressureDropped,
    ErrorCode::SubscribeDeliveryFailed,
    ErrorCode::SyncHashMismatch,
    ErrorCode::ThinClientHandshakeInvalid,
    ErrorCode::PluginManifestInvalid,
    ErrorCode::MaterializerSchemaMismatch,
    ErrorCode::RegistryDiscoveryTimeout,
];

fn bench_as_static_str_all_variants(c: &mut Criterion) {
    c.bench_function("as_static_str_all_variants", |b| {
        b.iter(|| {
            // Walk the representative spread; `as_static_str` should be a
            // zero-allocation jump table.
            for code in SAMPLE {
                black_box(black_box(code).as_static_str());
            }
        });
    });
}

fn bench_from_str_roundtrip(c: &mut Criterion) {
    // Pre-compute the string forms so the bench measures only `from_str`
    // (the length-bucketed memcmp chain), not the stringification. Benches
    // build on `std` targets so a plain `Vec` is fine here even though the
    // library crate itself is `no_std + alloc`.
    let strings: Vec<&'static str> = SAMPLE.iter().map(|c| c.as_static_str()).collect();
    c.bench_function("from_str_roundtrip", |b| {
        b.iter(|| {
            for s in &strings {
                let _ = black_box(ErrorCode::from_str(black_box(s)));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_as_static_str_all_variants,
    bench_from_str_roundtrip
);
criterion_main!(benches);
