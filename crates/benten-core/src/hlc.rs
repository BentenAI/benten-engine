//! Hybrid Logical Clock (HLC) primitives.
//!
//! Phase-3 G14-pre-D landed this module to close the `ds-1` BLOCKER:
//! `crates/benten-core/src/` previously had NO HLC module despite the plan
//! claiming HLC was "already in benten-core per D-PHASE-3-2." The
//! placeholder at `tests/prop_hlc_monotonic_placeholder.rs` (an `#[ignore]`'d
//! empty test) was the only artifact, and multiple Phase-3 deliverables —
//! Loro per-property LWW + Inv-14 device-grain attribution +
//! asymmetric-uptime MST-diff + cross-process WAIT-resume `cap_snapshot_hash`
//! — silently depend on a stable HLC surface.
//!
//! ## Design
//!
//! [`BentenHlc`] is the value type: `(physical_ms, logical, node_id)`. Two
//! [`BentenHlc`]s are compared lexicographically on `(physical_ms, logical,
//! node_id)`, matching the canonical Kulkarni-Demirbas HLC ordering with
//! `node_id` as the tie-breaker for the rare physical-clock-equal +
//! logical-counter-equal case.
//!
//! [`Hlc`] is the state machine: a thread-safe clock instance bound to a
//! `node_id`, an injectable physical-clock callback (`fn() -> u64` returning
//! milliseconds since the UNIX epoch), and an internal `last_emitted` HLC
//! protected by `spin::Mutex` so the surface stays `no_std`-compatible
//! (matching the rest of `benten-core`; see the crate-level `#![no_std]`
//! attribute).
//!
//! ## Operations
//!
//! - [`Hlc::now`] returns a fresh HLC strictly greater than every previously
//!   emitted HLC. If the physical clock has advanced past the last emitted
//!   `physical_ms`, the new HLC adopts the new physical clock and resets
//!   `logical` to `0`. Otherwise the physical clock is held steady at the
//!   last value and `logical` increments by `1` (Lamport-style bump).
//! - [`Hlc::update`] consumes a remote HLC and advances local state to
//!   `max(local, remote, physical_clock) + logical-bump-as-needed`.
//!   Returns [`CoreError::HlcSkewExceeded`] (mapped to
//!   [`ErrorCode::HlcSkewExceeded`]) when the remote's `physical_ms`
//!   exceeds the local physical clock by more than the configured skew
//!   tolerance ([`Hlc::DEFAULT_SKEW_TOLERANCE_MS`] = 5 minutes).
//!
//! ## Why a direct implementation, not the `uhlc` crate
//!
//! The `uhlc 0.2.1` crate was evaluated and rejected (G14-pre-D dispatch
//! finding 2026-05-04). Two blockers:
//!
//! 1. **`async_std` mutex on every operation.** `uhlc::HLC::new_timestamp` /
//!    `update_with_timestamp` are `async fn`s that hold an `async_std::sync::Mutex`
//!    across the await point. The Phase-3 consumers of HLC (Loro per-property
//!    LWW assign, cross-process WAIT-resume `cap_snapshot_hash`,
//!    asymmetric-uptime MST-diff) are synchronous code paths inside the
//!    evaluator and the storage backend; pulling `async-std` (and through it
//!    `polling`, `async-io`, `futures-lite`, `uuid`, `log`) into `benten-core`
//!    would (a) blow up cold compile time on a foundational crate, (b) break
//!    `benten-core`'s `#![no_std]` discipline (`async-std` is `std`-only),
//!    and (c) force every sync caller to thread an executor through the call
//!    site for what is fundamentally a `Mutex`-protected counter bump.
//! 2. **Async surface doesn't match the brief.** The G14-pre-D brief pins the
//!    surface as `hlc::now()` + `hlc::update(remote)` returning typed errors;
//!    `uhlc` returns `Result<(), String>` from `update_with_timestamp` (no
//!    `ErrorCode` mapping) and timestamp objects through an opaque `NTP64`.
//!
//! Implementing the state machine directly is ~150 LOC, no external deps,
//! `no_std`-compatible, and gives us the typed-error surface the brief
//! specifies. The 12-byte canonical `BentenHlc` shape `(u64, u32, u64)` is
//! also more compact than `uhlc::Timestamp` (a 16-byte UUID + an `NTP64`).

use core::fmt;

use spin::Mutex;

use benten_errors::ErrorCode;

use crate::CoreError;

// Map the new HlcSkewExceeded variant onto the existing `CoreError::code()`
// surface. The mapping lives at the bottom of `lib.rs`'s `impl CoreError`
// block; this module only consumes the typed error.

// ---------------------------------------------------------------------------
// BentenHlc — value type
// ---------------------------------------------------------------------------

/// A Hybrid Logical Clock stamp.
///
/// The triple `(physical_ms, logical, node_id)`:
///
/// - `physical_ms`: physical-clock component in milliseconds since the UNIX
///   epoch. Never wrapped backward by [`Hlc::now`] / [`Hlc::update`]: if the
///   underlying physical clock rewinds (NTP slew, container clock-jitter), the
///   HLC holds steady at its last emitted `physical_ms` and bumps the logical
///   counter instead.
/// - `logical`: 32-bit Lamport-style counter, bumped when the physical clock
///   does not advance past the last emitted value. Saturates at `u32::MAX`
///   (cap defended by [`Hlc::now`] returning the saturated value rather than
///   wrapping; the saturation point is `2^32` events at the same wall-clock
///   millisecond, which is unreachable in practice).
/// - `node_id`: 64-bit identifier disambiguating concurrent stamps from
///   different nodes. Phase-3 sync derives this from the peer-identity CID
///   (a 64-bit prefix of the BLAKE3 hash) via
///   [`BentenHlc::node_id_from_peer_id_bytes`].
///
/// Ordering is lexicographic over the triple. Two HLCs are equal iff all
/// three components are equal; the `Hash` impl mirrors the structural
/// equality so `BentenHlc` is safe to use as a `BTreeMap` / `HashMap` key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BentenHlc {
    physical_ms: u64,
    logical: u32,
    node_id: u64,
}

impl BentenHlc {
    /// Construct a stamp from the three components.
    ///
    /// Phase-3 sync layers consume this when reconstructing a remote HLC
    /// from on-wire bytes; in-process callers should prefer [`Hlc::now`].
    #[must_use]
    pub const fn new(physical_ms: u64, logical: u32, node_id: u64) -> Self {
        Self {
            physical_ms,
            logical,
            node_id,
        }
    }

    /// The UNIX-epoch-relative physical-clock component, in milliseconds.
    #[must_use]
    pub const fn physical_ms(&self) -> u64 {
        self.physical_ms
    }

    /// The 32-bit Lamport-style logical counter.
    #[must_use]
    pub const fn logical(&self) -> u32 {
        self.logical
    }

    /// The 64-bit node identifier.
    #[must_use]
    pub const fn node_id(&self) -> u64 {
        self.node_id
    }

    /// Derive a stable 64-bit `node_id` from the first 8 bytes of a peer-id
    /// digest. Phase-3 sync uses this to project a peer's BLAKE3 identity
    /// CID into an HLC `node_id` slot.
    #[must_use]
    pub const fn node_id_from_peer_id_bytes(prefix: [u8; 8]) -> u64 {
        u64::from_be_bytes(prefix)
    }
}

impl fmt::Display for BentenHlc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `<physical_ms>.<logical>@<node_id_hex>` — chosen so the textual
        // form sorts identically to the structural ordering (physical first,
        // logical second, node_id last). Hex node-id keeps the form compact
        // for typical 64-bit ids.
        write!(
            f,
            "{}.{}@{:016x}",
            self.physical_ms, self.logical, self.node_id
        )
    }
}

// ---------------------------------------------------------------------------
// Hlc — state machine
// ---------------------------------------------------------------------------

/// The default skew-tolerance window: 5 minutes (300 000 ms).
///
/// Per the G14-pre-D brief: "configurable max-skew bound (e.g., 5 minutes
/// default). Beyond bound, `update()` returns `Err(E_HLC_SKEW_EXCEEDED)`."
const DEFAULT_SKEW_TOLERANCE_MS: u64 = 5 * 60 * 1000;

/// Type of the injectable physical-clock callback.
///
/// Returns the current wall-clock time in milliseconds since the UNIX epoch.
/// Tests inject a deterministic counter; production callers inject a
/// `std::time::SystemTime`-backed callable wired from `benten-graph` (which
/// already depends on std via redb), keeping `benten-core` free of an
/// implicit std-only feature gate.
pub type PhysicalClockFn = fn() -> u64;

/// HLC state machine.
///
/// Bind one [`Hlc`] per process / per logical node identity. The state
/// machine is `Send + Sync`: every mutating operation takes the internal
/// `spin::Mutex`, so concurrent callers serialize cleanly.
///
/// # Examples
///
/// ```
/// use benten_core::hlc::Hlc;
///
/// fn frozen_clock() -> u64 { 1_000 }
/// let hlc = Hlc::new(42, frozen_clock);
///
/// let a = hlc.now();
/// let b = hlc.now();
/// // Monotonic even when the physical clock does not advance: the
/// // logical counter increments instead.
/// assert!(b > a);
/// ```
pub struct Hlc {
    node_id: u64,
    skew_tolerance_ms: u64,
    physical_clock: PhysicalClockFn,
    last_emitted: Mutex<BentenHlc>,
}

// Safe-4 #636: compile-time pin for the `Hlc` docstring's "`Send + Sync`:
// concurrent callers serialize cleanly" claim. If a future field breaks
// either auto-trait this fails to compile rather than silently regressing
// the documented concurrency contract. Zero-cost: the closure is never
// called.
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Hlc>();
};

impl Hlc {
    /// The default skew-tolerance window: 5 minutes (300 000 ms).
    pub const DEFAULT_SKEW_TOLERANCE_MS: u64 = DEFAULT_SKEW_TOLERANCE_MS;

    /// Construct a new clock bound to `node_id`, the default 5-minute
    /// skew tolerance, and the given physical-clock callback.
    ///
    /// The clock starts at `(0, 0, node_id)`; the first [`Hlc::now`] call
    /// either adopts the physical clock's current reading (the common case)
    /// or emits `(0, 1, node_id)` (the unusual case where the physical
    /// clock callback returns `0`, e.g. a `MockClock::frozen_at(0)`).
    #[must_use]
    pub fn new(node_id: u64, physical_clock: PhysicalClockFn) -> Self {
        Self {
            node_id,
            skew_tolerance_ms: DEFAULT_SKEW_TOLERANCE_MS,
            physical_clock,
            last_emitted: Mutex::new(BentenHlc::new(0, 0, node_id)),
        }
    }

    /// Construct a clock with an explicit skew tolerance.
    ///
    /// Tests use this to drive [`Hlc::update`] across the boundary
    /// deterministically. Production code should prefer [`Hlc::new`] +
    /// the default [`Hlc::DEFAULT_SKEW_TOLERANCE_MS`].
    #[must_use]
    pub fn with_skew_tolerance(
        node_id: u64,
        physical_clock: PhysicalClockFn,
        skew_tolerance_ms: u64,
    ) -> Self {
        Self {
            node_id,
            skew_tolerance_ms,
            physical_clock,
            last_emitted: Mutex::new(BentenHlc::new(0, 0, node_id)),
        }
    }

    /// The node id this clock emits stamps for.
    #[must_use]
    pub fn node_id(&self) -> u64 {
        self.node_id
    }

    /// The configured skew-tolerance window in milliseconds.
    #[must_use]
    pub fn skew_tolerance_ms(&self) -> u64 {
        self.skew_tolerance_ms
    }

    /// Generate a fresh HLC stamp strictly greater than every stamp this
    /// clock has emitted, and strictly greater than every remote stamp
    /// this clock has [`Hlc::update`]d against.
    ///
    /// Algorithm (Kulkarni-Demirbas):
    ///
    /// ```text
    /// pt   = physical_clock()
    /// l'   = max(last.physical_ms, pt)
    /// if l' == last.physical_ms:
    ///     c' = last.logical.saturating_add(1)
    /// else:
    ///     c' = 0
    /// emit (l', c', node_id)
    /// ```
    pub fn now(&self) -> BentenHlc {
        let pt = (self.physical_clock)();
        let mut last = self.last_emitted.lock();
        let new_physical = core::cmp::max(last.physical_ms, pt);
        let new_logical = if new_physical == last.physical_ms {
            // Physical clock did not advance past the last emit — bump the
            // Lamport counter. Saturating semantics: the alternative
            // (wrapping back to 0) would silently violate monotonicity at
            // the `2^32`-th event in the same millisecond, an invariant
            // load-bearing for Loro per-property LWW.
            last.logical.saturating_add(1)
        } else {
            0
        };
        let emitted = BentenHlc::new(new_physical, new_logical, self.node_id);
        *last = emitted;
        emitted
    }

    /// Consume a remote HLC, advancing this clock so the next [`Hlc::now`]
    /// returns a stamp strictly greater than `remote`, and return the
    /// post-update local stamp.
    ///
    /// Algorithm:
    ///
    /// ```text
    /// pt = physical_clock()
    /// if remote.physical_ms > pt + skew_tolerance_ms:
    ///     return Err(HlcSkewExceeded)   # adversarial / mis-configured peer
    /// l' = max(last.physical_ms, remote.physical_ms, pt)
    /// match l':
    ///     == last.physical_ms == remote.physical_ms:
    ///         c' = max(last.logical, remote.logical) + 1
    ///     == last.physical_ms:
    ///         c' = last.logical + 1
    ///     == remote.physical_ms:
    ///         c' = remote.logical + 1
    ///     else (== pt):
    ///         c' = 0
    /// emit (l', c', node_id)
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::HlcSkewExceeded`] when `remote.physical_ms`
    /// exceeds the local physical clock by more than
    /// [`Hlc::skew_tolerance_ms`]. The local state is **not** mutated in
    /// that case — Phase-3 sync should reject the offending message and
    /// continue.
    pub fn update(&self, remote: &BentenHlc) -> Result<BentenHlc, CoreError> {
        let pt = (self.physical_clock)();
        // Skew check: cap how far INTO THE FUTURE a remote stamp may carry
        // us. We do NOT reject stamps in the past — that's the normal case
        // for a peer whose message was queued briefly in the network.
        if remote.physical_ms > pt.saturating_add(self.skew_tolerance_ms) {
            return Err(CoreError::HlcSkewExceeded {
                local_physical_ms: pt,
                remote_physical_ms: remote.physical_ms,
                tolerance_ms: self.skew_tolerance_ms,
            });
        }
        let mut last = self.last_emitted.lock();
        let new_physical = core::cmp::max(core::cmp::max(last.physical_ms, remote.physical_ms), pt);
        let new_logical = if new_physical == last.physical_ms && new_physical == remote.physical_ms
        {
            // Three-way physical-clock tie: take the larger of the two
            // logical counters and bump.
            core::cmp::max(last.logical, remote.logical).saturating_add(1)
        } else if new_physical == last.physical_ms {
            last.logical.saturating_add(1)
        } else if new_physical == remote.physical_ms {
            remote.logical.saturating_add(1)
        } else {
            // Physical clock advanced past both last + remote — fresh
            // start at logical 0.
            0
        };
        let emitted = BentenHlc::new(new_physical, new_logical, self.node_id);
        *last = emitted;
        Ok(emitted)
    }
}

impl fmt::Debug for Hlc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The `physical_clock: fn()` field is uninteresting and not Debug;
        // the locked state field is fine to format under the spin lock —
        // `try_lock` here would risk Debug returning "<locked>" mid-trace.
        let last = self.last_emitted.lock();
        f.debug_struct("Hlc")
            .field("node_id", &self.node_id)
            .field("skew_tolerance_ms", &self.skew_tolerance_ms)
            .field("last_emitted", &*last)
            .finish_non_exhaustive()
    }
}

// `system_time_ms` (a `std::time::SystemTime`-backed [`PhysicalClockFn`])
// intentionally does NOT live in `benten-core`: `benten-core` is `#![no_std]`
// and pulling `std::time` in even behind a feature gate creates a
// cross-cutting `extern crate std` we'd rather not own. Phase-3 sync
// wires the wallclock-backed clock from `benten-graph` (already `std` via
// `redb`) — this module exposes the [`PhysicalClockFn`] alias so callers
// can pass any `fn() -> u64` ms-since-epoch closure.

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "tests and benches may use unwrap per workspace policy"
)]
mod tests {
    use super::*;
    extern crate std;
    use core::sync::atomic::{AtomicU64, Ordering};

    // Deterministic mock clock — the `fn() -> u64` bare-fn-pointer
    // signature precludes closure capture, so each mock-clock state
    // must live in a `static`. Per-test statics (NOT a single shared
    // `MOCK_TIME_MS`) avoid cross-test interference under
    // parallel-scheduled `cargo test` (default scheduler) +
    // `cargo-llvm-cov` (which shells out to plain `cargo test --tests`
    // and does NOT honor nextest test-groups). Sibling decomposition
    // applied to the integration sister at
    // `crates/benten-core/tests/hlc_clock_skew_within_tolerance.rs`
    // per phase-3-backlog §7.18 closure (R6 R2 hlc-r6-r2-1 sibling-
    // site closure: PR #168 cargo-llvm-cov flake `left:2000/right:10000`
    // matched the cross-test reset-value contention between
    // `now_bumps_logical_when_physical_clock_stalls` (reset_mock(2_000))
    // and `update_within_tolerance_advances_local`
    // (reset_mock(10_000)) sharing the prior single `MOCK_TIME_MS`).

    // ---- now_advances_when_physical_clock_advances ----
    static MOCK_NOW_ADV_TIME_MS: AtomicU64 = AtomicU64::new(1_000);
    fn now_adv_clock() -> u64 {
        MOCK_NOW_ADV_TIME_MS.load(Ordering::SeqCst)
    }

    #[test]
    fn now_advances_when_physical_clock_advances() {
        MOCK_NOW_ADV_TIME_MS.store(1_000, Ordering::SeqCst);
        let hlc = Hlc::new(0xAAAA, now_adv_clock);
        let a = hlc.now();
        MOCK_NOW_ADV_TIME_MS.fetch_add(50, Ordering::SeqCst);
        let b = hlc.now();
        assert!(
            b > a,
            "second now() must exceed first when wallclock advances"
        );
        assert_eq!(b.physical_ms(), 1_050);
        assert_eq!(b.logical(), 0, "logical resets to 0 on physical advance");
    }

    // ---- now_bumps_logical_when_physical_clock_stalls ----
    static MOCK_NOW_BUMPS_TIME_MS: AtomicU64 = AtomicU64::new(2_000);
    fn now_bumps_clock() -> u64 {
        MOCK_NOW_BUMPS_TIME_MS.load(Ordering::SeqCst)
    }

    #[test]
    fn now_bumps_logical_when_physical_clock_stalls() {
        MOCK_NOW_BUMPS_TIME_MS.store(2_000, Ordering::SeqCst);
        let hlc = Hlc::new(0xBBBB, now_bumps_clock);
        let a = hlc.now();
        let b = hlc.now();
        let c = hlc.now();
        assert_eq!(a.physical_ms(), b.physical_ms());
        assert_eq!(b.physical_ms(), c.physical_ms());
        assert_eq!(a.logical(), 0);
        assert_eq!(b.logical(), 1);
        assert_eq!(c.logical(), 2);
        assert!(a < b && b < c);
    }

    // ---- now_holds_steady_when_physical_clock_rewinds ----
    static MOCK_NOW_REWIND_TIME_MS: AtomicU64 = AtomicU64::new(5_000);
    fn now_rewind_clock() -> u64 {
        MOCK_NOW_REWIND_TIME_MS.load(Ordering::SeqCst)
    }

    #[test]
    fn now_holds_steady_when_physical_clock_rewinds() {
        // Adversarial NTP slew: wallclock jumps backward between calls.
        // The HLC must NOT regress; it bumps the logical counter from
        // the last emitted physical_ms instead.
        MOCK_NOW_REWIND_TIME_MS.store(5_000, Ordering::SeqCst);
        let hlc = Hlc::new(0xCCCC, now_rewind_clock);
        let a = hlc.now();
        assert_eq!(a.physical_ms(), 5_000);
        MOCK_NOW_REWIND_TIME_MS.store(4_000, Ordering::SeqCst);
        let b = hlc.now();
        assert!(b > a, "rewind must not produce a regressed HLC");
        assert_eq!(b.physical_ms(), 5_000, "physical_ms held at last emit");
        assert_eq!(b.logical(), 1, "logical bumped under rewind");
    }

    // ---- update_within_tolerance_advances_local ----
    static MOCK_UPDATE_WITHIN_TIME_MS: AtomicU64 = AtomicU64::new(10_000);
    fn update_within_clock() -> u64 {
        MOCK_UPDATE_WITHIN_TIME_MS.load(Ordering::SeqCst)
    }

    #[test]
    fn update_within_tolerance_advances_local() {
        MOCK_UPDATE_WITHIN_TIME_MS.store(10_000, Ordering::SeqCst);
        let hlc = Hlc::new(0xDDDD, update_within_clock);
        let local_a = hlc.now();
        // Remote ahead by 1 second — well within the 5-minute default.
        let remote = BentenHlc::new(11_000, 7, 0xEEEE);
        let local_b = hlc.update(&remote).unwrap();
        assert!(local_b > local_a);
        assert!(local_b > remote, "post-update local must exceed remote");
        assert_eq!(local_b.physical_ms(), 11_000);
        assert_eq!(local_b.logical(), 8, "remote.logical + 1");
        assert_eq!(local_b.node_id(), 0xDDDD, "node_id stays local");
    }

    // ---- update_beyond_tolerance_fires_skew_exceeded_and_does_not_mutate ----
    static MOCK_UPDATE_BEYOND_TIME_MS: AtomicU64 = AtomicU64::new(100_000);
    fn update_beyond_clock() -> u64 {
        MOCK_UPDATE_BEYOND_TIME_MS.load(Ordering::SeqCst)
    }

    #[test]
    fn update_beyond_tolerance_fires_skew_exceeded_and_does_not_mutate() {
        MOCK_UPDATE_BEYOND_TIME_MS.store(100_000, Ordering::SeqCst);
        let hlc = Hlc::new(0xFFFF, update_beyond_clock);
        let _local_a = hlc.now();
        // Remote claims it's 6 minutes in the future — beyond the 5 min default.
        let far_future = 100_000 + 6 * 60 * 1000;
        let remote = BentenHlc::new(far_future, 0, 0x1234);
        let err = hlc.update(&remote).expect_err("skew must be rejected");
        match err {
            CoreError::HlcSkewExceeded {
                local_physical_ms,
                remote_physical_ms,
                tolerance_ms,
            } => {
                assert_eq!(local_physical_ms, 100_000);
                assert_eq!(remote_physical_ms, far_future);
                assert_eq!(tolerance_ms, Hlc::DEFAULT_SKEW_TOLERANCE_MS);
            }
            other => panic!("expected HlcSkewExceeded, got {other:?}"),
        }
        // Sanity: local state still serves a sane stamp.
        let after = hlc.now();
        assert_eq!(after.physical_ms(), 100_000);
        assert!(
            after.logical() > 0,
            "local logical bumped (own now() calls)"
        );
    }

    // ---- skew_exceeded_maps_to_e_hlc_skew_exceeded_catalog_code ----
    static MOCK_SKEW_CODE_TIME_MS: AtomicU64 = AtomicU64::new(1_000);
    fn skew_code_clock() -> u64 {
        MOCK_SKEW_CODE_TIME_MS.load(Ordering::SeqCst)
    }

    #[test]
    fn skew_exceeded_maps_to_e_hlc_skew_exceeded_catalog_code() {
        MOCK_SKEW_CODE_TIME_MS.store(1_000, Ordering::SeqCst);
        let hlc = Hlc::with_skew_tolerance(1, skew_code_clock, 100);
        let remote = BentenHlc::new(50_000, 0, 2);
        let err = hlc.update(&remote).unwrap_err();
        assert_eq!(err.code(), ErrorCode::HlcSkewExceeded);
        assert_eq!(ErrorCode::HlcSkewExceeded.as_str(), "E_HLC_SKEW_EXCEEDED");
    }

    #[test]
    fn lexicographic_ordering() {
        let a = BentenHlc::new(100, 0, 1);
        let b = BentenHlc::new(100, 0, 2);
        let c = BentenHlc::new(100, 1, 1);
        let d = BentenHlc::new(101, 0, 1);
        assert!(a < b, "node_id breaks tie when physical+logical equal");
        assert!(b < c, "logical dominates node_id");
        assert!(c < d, "physical dominates logical");
    }

    #[test]
    fn display_format_sorts_with_structural_order() {
        let a = BentenHlc::new(100, 5, 0xCAFE_BABE);
        let s = std::format!("{a}");
        assert_eq!(s, "100.5@00000000cafebabe");
    }

    #[test]
    fn node_id_from_peer_id_bytes_is_be_decoded() {
        let prefix = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let id = BentenHlc::node_id_from_peer_id_bytes(prefix);
        assert_eq!(id, 0x1234_5678_9abc_def0);
    }

    // ---- three_way_tie_picks_max_logical_plus_one ----
    static MOCK_THREE_WAY_TIME_MS: AtomicU64 = AtomicU64::new(7_000);
    fn three_way_clock() -> u64 {
        MOCK_THREE_WAY_TIME_MS.load(Ordering::SeqCst)
    }

    #[test]
    fn three_way_tie_picks_max_logical_plus_one() {
        // Local + remote both at the same physical_ms as the wallclock,
        // with different logical values. The post-update logical should
        // be max(local.logical, remote.logical) + 1.
        MOCK_THREE_WAY_TIME_MS.store(7_000, Ordering::SeqCst);
        let hlc = Hlc::new(1, three_way_clock);
        let _ = hlc.now(); // local: (7000, 0, 1)
        let _ = hlc.now(); // local: (7000, 1, 1)
        let _ = hlc.now(); // local: (7000, 2, 1)
        let remote = BentenHlc::new(7_000, 5, 2);
        let after = hlc.update(&remote).unwrap();
        assert_eq!(after.physical_ms(), 7_000);
        assert_eq!(after.logical(), 6, "max(2,5)+1 = 6");
    }
}
