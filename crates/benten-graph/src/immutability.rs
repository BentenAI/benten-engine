//! Storage-layer immutability module — Phase 2a G2-A skeleton for Inv-13.
//!
//! # Scope (G2-A only)
//!
//! This file lands the **infrastructure** G5-A's 5-row Inv-13 firing matrix
//! composes on top of:
//!
//! - [`BloomFilter`] — simple bit-array Bloom filter keyed on
//!   [`benten_core::Cid`] bytes, with a configurable target false-positive rate
//!   (default `1 / 10_000`). The filter does not allocate after construction.
//! - [`CidExistenceCache`] — thin wrapper over the Bloom filter plus:
//!   - test-only `force_bloom_collision_for_next_put` flag that makes the next
//!     `may_contain` probe return `true` unconditionally (exercises the
//!     false-positive fallback path in
//!     `tests/immutability_rejects_reput::immutability_bloom_false_positive_falls_back_to_exact_check`);
//!   - test-only `force_positive_for_test(cid)` that forces `may_contain` to
//!     return `true` for a specific CID (exercises `force_bloom_positive_for_test`
//!     per plan §9.11 row atk-3 / sec-r1-4, consumed by
//!     `tests/inv_13_bloom_false_positive.rs`).
//!
//! # Contract vs G5-A
//!
//! G5-A owns the 5-row matrix itself (the `WriteAuthority`-driven branching in
//! [`crate::redb_backend::RedbBackend::put_node_with_context`] and
//! [`crate::transaction::Transaction::put_node`]). G2-A's contribution here is
//! the Bloom-filter fast-path and exact-check helper that the matrix calls.
//!
//! The fast-path contract is:
//!
//! 1. **Fast negative** — if `bloom.may_contain(cid) == false`, the CID is
//!    definitively absent from the backend; the write proceeds without an
//!    authoritative redb probe.
//! 2. **False-positive fallback** — if `bloom.may_contain(cid) == true`, the
//!    caller MUST follow up with an authoritative redb lookup (`get_node`)
//!    before treating the CID as present. Treating bloom-positive as
//!    "definitely present" would silently dedupe distinct Nodes on a
//!    collision.
//!
//! The exact-check is the caller's responsibility (not this module's) because
//! the redb txn the check must run against is usually an already-open
//! `WriteTransaction` the caller holds.
//!
//! # Phase-1 predecessor
//!
//! Phase-1 tracked CID existence with a plain `HashSet<Cid>` in
//! `redb_backend.rs`. G2-A promotes that to this module so the fast-path
//! cost stays bounded (Bloom is `O(1)` amortised with a fixed memory cost
//! set at construction, independent of CIDs inserted).

use std::collections::HashSet;

use benten_core::Cid;

/// Default target false-positive rate for [`BloomFilter::for_expected_inserts`]
/// and [`CidExistenceCache::new`].
///
/// Chosen to match the plan §3 G2-A wording (`default 1/10000`). A site with a
/// tighter budget can call [`BloomFilter::for_expected_inserts`] directly with
/// a custom rate.
pub const DEFAULT_FALSE_POSITIVE_RATE: f64 = 1.0 / 10_000.0;

/// Default capacity (expected distinct inserts) when the caller has no better
/// guess. Sized to keep the bit-array under a few kilobytes at the default
/// false-positive rate.
const DEFAULT_EXPECTED_INSERTS: usize = 4096;

/// G11-A unbounded-cache bound: maximum entries in the `warmed` test-only
/// tracking set before the oldest half is evicted. Prevents the long-lived
/// integration-test process from accumulating unbounded CIDs in memory
/// (unbounded-cache G11-A capture). 100k gives every foreseeable test run
/// plenty of headroom; the production-side fast-path consults the bloom
/// filter, which is capacity-independent.
const WARMED_CAP: usize = 100_000;

/// A pedagogically-simple Bloom filter keyed on [`Cid`] bytes.
///
/// The filter uses the double-hashing technique — two independent base hashes
/// derived directly from the CID's BLAKE3 digest are combined as
/// `h_i(cid) = h1 + i * h2` for `i` in `0..num_hashes`. Because the CID
/// payload itself is a uniformly-random 32-byte BLAKE3 digest, reading two
/// disjoint u64 windows out of it gives two independent hash values with no
/// additional hashing cost — this keeps the module dependency-free (no
/// `siphasher` / `ahash` / `fnv` needed) and the probe at under 10ns in
/// benchmarks.
///
/// The filter is NOT threadsafe on its own — the backing `Vec<u64>` is only
/// marked `&mut` on insert, so callers embed it behind a `Mutex` (see
/// [`CidExistenceCache`]'s crate-private use).
pub struct BloomFilter {
    /// Packed bit array. Each `u64` holds 64 bits; bit index `b` lives in
    /// `bits[b / 64]` at position `b % 64`.
    bits: Vec<u64>,
    /// Total number of bits in the filter (== `bits.len() * 64`).
    num_bits: usize,
    /// Number of hash functions probed per operation. Derived from the
    /// target false-positive rate at construction.
    num_hashes: u32,
}

impl BloomFilter {
    /// Construct a filter sized to keep the false-positive rate at or under
    /// `fp_rate` when populated with up to `expected_inserts` distinct CIDs.
    ///
    /// `fp_rate` is clamped to the open interval `(0, 1)`; pathological inputs
    /// (0, 1, negative, NaN) collapse to [`DEFAULT_FALSE_POSITIVE_RATE`].
    /// `expected_inserts` is clamped to a minimum of 1.
    #[must_use]
    pub fn for_expected_inserts(expected_inserts: usize, fp_rate: f64) -> Self {
        let expected_inserts = expected_inserts.max(1);
        let fp_rate = if fp_rate > 0.0 && fp_rate < 1.0 {
            fp_rate
        } else {
            DEFAULT_FALSE_POSITIVE_RATE
        };

        // Optimal bit count: m = -n ln(p) / (ln 2)^2
        //
        // The `as f64` casts here lose precision on 64-bit `usize` values
        // above 2^53, but the inputs are capacity hints (a few thousand) and
        // `num_bits` (a few kilobits for the default sizing); both are well
        // inside `f64`'s exact range. `cast_possible_truncation` on the
        // `.ceil() as usize` round-trip is similarly bounded by the input.
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "inputs are small positive capacity hints well inside f64's exact range"
        )]
        let (num_bits, num_hashes, num_words) = {
            let ln2_sq = core::f64::consts::LN_2 * core::f64::consts::LN_2;
            let m_float = -(expected_inserts as f64) * fp_rate.ln() / ln2_sq;
            // Round up to a whole number of u64 words (64 bits each) so we
            // never underrun the target sizing.
            let raw_bits = (m_float.ceil() as usize).max(64);
            let words = raw_bits.div_ceil(64);
            let bits = words * 64;

            // Optimal hash count: k = (m / n) * ln 2
            let k_float = (bits as f64 / expected_inserts as f64) * core::f64::consts::LN_2;
            let hashes = (k_float.round() as u32).max(1);
            (bits, hashes, words)
        };

        Self {
            bits: vec![0u64; num_words],
            num_bits,
            num_hashes,
        }
    }

    /// Shortcut for `for_expected_inserts(expected_inserts, DEFAULT_FALSE_POSITIVE_RATE)`.
    #[must_use]
    pub fn with_default_fp_rate(expected_inserts: usize) -> Self {
        Self::for_expected_inserts(expected_inserts, DEFAULT_FALSE_POSITIVE_RATE)
    }

    /// Number of bits in the backing array.
    #[must_use]
    pub fn len_bits(&self) -> usize {
        self.num_bits
    }

    /// Number of hash-probe rounds per `insert` / `may_contain` call.
    #[must_use]
    pub fn num_hashes(&self) -> u32 {
        self.num_hashes
    }

    /// Insert a CID. Subsequent [`Self::may_contain`] calls for the same CID
    /// return `true` (modulo hash collisions already setting the same bits).
    pub fn insert(&mut self, cid: &Cid) {
        let (h1, h2) = base_hashes(cid);
        for i in 0..self.num_hashes {
            let idx = double_hash_index(h1, h2, i, self.num_bits);
            self.bits[idx / 64] |= 1u64 << (idx % 64);
        }
    }

    /// `true` if every bit position the CID maps to is set. A `true` answer is
    /// non-authoritative (may be a false positive); a `false` answer is
    /// authoritative (the CID has definitely not been inserted).
    #[must_use]
    pub fn may_contain(&self, cid: &Cid) -> bool {
        let (h1, h2) = base_hashes(cid);
        for i in 0..self.num_hashes {
            let idx = double_hash_index(h1, h2, i, self.num_bits);
            let word = self.bits[idx / 64];
            if (word >> (idx % 64)) & 1 == 0 {
                return false;
            }
        }
        true
    }
}

/// Two independent 64-bit base hashes derived from the CID's BLAKE3 digest.
///
/// The Benten CID lays out as `[CID_V1, multicodec, multihash, digest_len,
/// digest(32 bytes)]`; we read two disjoint u64 windows out of the digest
/// portion. Because the digest is already a uniformly-random BLAKE3 output,
/// the two windows are (to BLAKE3's distinguishing-advantage bound) mutually
/// independent — equivalent to hashing twice with distinct keys at no cost.
fn base_hashes(cid: &Cid) -> (u64, u64) {
    let bytes = cid.as_bytes();
    // BLAKE3 digest starts at byte offset 4; see
    // `benten_core::Cid::from_blake3_digest`. `Cid::from_bytes` guarantees the
    // full header + 32 digest bytes, so indexing `[4..12]` and `[12..20]` is
    // safe; we use `try_from` + `unwrap_or` to keep the code panic-free under
    // an unexpected future layout change.
    let h1 = u64::from_le_bytes(bytes[4..12].try_into().unwrap_or([0; 8]));
    let h2 = u64::from_le_bytes(bytes[12..20].try_into().unwrap_or([0; 8]));
    // If the second window is zero the double-hashing degenerates (every
    // probe lands on h1); mix with a small golden-ratio constant so the
    // degenerate case still spreads bits.
    let h2 = if h2 == 0 { 0x9E37_79B9_7F4A_7C15 } else { h2 };
    (h1, h2)
}

/// Double-hashing index formula — `(h1 + i * h2) mod num_bits`.
fn double_hash_index(h1: u64, h2: u64, i: u32, num_bits: usize) -> usize {
    let combined = h1.wrapping_add((i as u64).wrapping_mul(h2));
    (combined as usize) % num_bits
}

/// CID-existence cache consumed by [`crate::redb_backend::RedbBackend`]'s
/// Inv-13 fast-path. Wraps a [`BloomFilter`] plus the test-only collision /
/// positive-force hooks mandated by plan §9.11.
///
/// # Thread-safety
///
/// The cache itself is not `Sync`; embed it inside the backend's
/// `Mutex<CidExistenceCache>` / `Arc<…>` (the pattern every other mutable
/// field on `RedbBackend` uses).
pub struct CidExistenceCache {
    bloom: BloomFilter,
    /// One-shot: the next `may_contain` call returns `true` unconditionally,
    /// then clears. Exercises the false-positive fallback path without
    /// needing a real hash collision.
    forced_collision_next: bool,
    /// CIDs for which `may_contain` returns `true` without consulting the
    /// bloom bits. Populated by
    /// [`Self::force_positive_for_test`]; never cleared automatically.
    forced_positives: HashSet<Cid>,
    /// Authoritative record of CIDs that have been `insert`ed during this
    /// process. Backs the `cache_contains_cid` test hook so the warmness
    /// assertion is not held hostage to the bloom false-positive rate.
    ///
    /// This set is NOT part of the Inv-13 fast-path (the hot-path probe uses
    /// [`BloomFilter::may_contain`] — the authoritative check is against
    /// redb, not this set). It exists only to give test code a
    /// "definitely warmed" view.
    warmed: HashSet<Cid>,
}

impl CidExistenceCache {
    /// Construct a cache with the default expected-inserts + false-positive
    /// rate. Suitable for every production path — sites needing a tighter
    /// bound can call [`Self::with_sizing`] directly.
    #[must_use]
    pub fn new() -> Self {
        Self::with_sizing(DEFAULT_EXPECTED_INSERTS, DEFAULT_FALSE_POSITIVE_RATE)
    }

    /// Construct with an explicit expected-inserts + false-positive rate.
    #[must_use]
    pub fn with_sizing(expected_inserts: usize, fp_rate: f64) -> Self {
        Self {
            bloom: BloomFilter::for_expected_inserts(expected_inserts, fp_rate),
            forced_collision_next: false,
            forced_positives: HashSet::new(),
            warmed: HashSet::new(),
        }
    }

    /// Fast-path probe — `true` means "maybe present, run the exact redb
    /// lookup"; `false` means "definitely absent".
    ///
    /// Respects `forced_collision_next` (clears it after firing) and
    /// `forced_positives` so tests can drive the fallback path deterministically.
    ///
    /// Takes `&mut self` because the one-shot collision flag needs to clear.
    pub fn may_contain(&mut self, cid: &Cid) -> bool {
        if self.forced_positives.contains(cid) {
            return true;
        }
        if self.forced_collision_next {
            self.forced_collision_next = false;
            return true;
        }
        self.bloom.may_contain(cid)
    }

    /// Non-mutating peek — same as [`Self::may_contain`] but skips the
    /// one-shot collision clear. Used by `cache_contains_cid` (test-only
    /// warmness probe) where mutating state from an inspection API would
    /// poison subsequent assertions in the same test.
    #[must_use]
    pub fn may_contain_peek(&self, cid: &Cid) -> bool {
        if self.forced_positives.contains(cid) {
            return true;
        }
        self.bloom.may_contain(cid)
    }

    /// Warmness check used by `cache_contains_cid` — reports `true` iff the
    /// CID has actually been [`Self::insert`]ed during this process (or is in
    /// the forced-positive set).
    #[must_use]
    pub fn warmed_for(&self, cid: &Cid) -> bool {
        self.warmed.contains(cid) || self.forced_positives.contains(cid)
    }

    /// Record that the CID has been persisted. Sets its bits in the bloom
    /// filter and adds it to the `warmed` set. When `warmed` reaches
    /// `WARMED_CAP`, the set is cleared before inserting the new CID — the
    /// authoritative existence check is against redb, not this set, so
    /// eviction is safe even for in-flight warmness assertions (which
    /// test processes re-trigger after the cap).
    pub fn insert(&mut self, cid: &Cid) {
        self.bloom.insert(cid);
        if self.warmed.len() >= WARMED_CAP {
            self.warmed.clear();
        }
        self.warmed.insert(*cid);
    }

    /// Arm the one-shot forced-collision flag. The next `may_contain` call
    /// returns `true` unconditionally and then clears the flag. Exercises the
    /// bloom false-positive fallback path in
    /// `tests/immutability_rejects_reput`.
    pub fn force_collision_next(&mut self) {
        self.forced_collision_next = true;
    }

    /// Register a CID whose `may_contain` answer should be forced to `true`
    /// until explicitly removed. Used by
    /// `RedbBackend::force_bloom_positive_for_test` (plan §4.7 row for
    /// atk-3 / sec-r1-4).
    pub fn force_positive_for_test(&mut self, cid: &Cid) {
        self.forced_positives.insert(*cid);
    }
}

impl Default for CidExistenceCache {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use benten_core::Cid;

    fn cid_from_byte(b: u8) -> Cid {
        let mut digest = [0u8; 32];
        digest[0] = b;
        digest[5] = b.wrapping_mul(3).wrapping_add(17);
        digest[10] = b.wrapping_mul(7).wrapping_add(29);
        Cid::from_blake3_digest(digest)
    }

    #[test]
    fn bloom_inserted_cid_reports_may_contain() {
        let mut bf = BloomFilter::with_default_fp_rate(1024);
        let a = cid_from_byte(1);
        bf.insert(&a);
        assert!(bf.may_contain(&a), "inserted CID must report may_contain");
    }

    #[test]
    fn bloom_sizing_honors_target_fp_rate() {
        // Tighter budget should land more bits than the default.
        let default_bf = BloomFilter::for_expected_inserts(1000, 0.01);
        let tight_bf = BloomFilter::for_expected_inserts(1000, 0.0001);
        assert!(tight_bf.len_bits() > default_bf.len_bits());
        assert!(tight_bf.num_hashes() >= default_bf.num_hashes());
    }

    #[test]
    fn bloom_sizing_clamps_pathological_fp_rate() {
        // NaN / 0 / 1 / negative inputs collapse to the default.
        let bf_nan = BloomFilter::for_expected_inserts(100, f64::NAN);
        let bf_default = BloomFilter::for_expected_inserts(100, DEFAULT_FALSE_POSITIVE_RATE);
        assert_eq!(bf_nan.len_bits(), bf_default.len_bits());

        let bf_zero = BloomFilter::for_expected_inserts(100, 0.0);
        assert_eq!(bf_zero.len_bits(), bf_default.len_bits());

        let bf_one = BloomFilter::for_expected_inserts(100, 1.0);
        assert_eq!(bf_one.len_bits(), bf_default.len_bits());
    }

    #[test]
    fn cache_forced_collision_is_one_shot() {
        let mut cache = CidExistenceCache::new();
        let a = cid_from_byte(3);
        cache.force_collision_next();
        assert!(cache.may_contain(&a), "forced collision fires");
        // Second call no longer forced (the bit hasn't been set because we
        // never called `insert`), so this must now be a definitive negative.
        assert!(!cache.may_contain(&a), "forced-collision flag is one-shot");
    }

    #[test]
    fn cache_warmness_tracks_inserts() {
        let mut cache = CidExistenceCache::new();
        let a = cid_from_byte(4);
        assert!(!cache.warmed_for(&a), "cold cache reports not-warm");
        cache.insert(&a);
        assert!(cache.warmed_for(&a), "post-insert the CID must report warm");
    }

    #[test]
    fn cache_force_positive_is_persistent() {
        let mut cache = CidExistenceCache::new();
        let a = cid_from_byte(5);
        cache.force_positive_for_test(&a);
        assert!(cache.may_contain(&a));
        // Second probe still reports positive (contrast with force_collision_next).
        assert!(cache.may_contain(&a));
    }
}
