//! F-LD-5 — Intra-hour-replay-rejected-by-nonce-cache (RED-PHASE; M-1 load-bearing).
//!
//! R3 wave **W3-layer-d**. Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 8
//!     F-LD-5: "re-presented `PermissionGrant`/DeviceLink within same 1-hr
//!     bucket rejected **by nonce-cache** (Compromise #25 substrate
//!     `handshake.rs:72`), NOT by time field; 1-hr bucket Layer-D-ONLY (M-14);
//!     nonce-cache retention ≥ full 1-hr window + durable-across-restart;
//!     rejection nonce-keyed not time-keyed." Red-phase: "present twice
//!     intra-bucket → 2nd `Err` from nonce-cache; **disable-cache negative**
//!     (replay would pass — bucket alone insufficient); restart-durability arm;
//!     clock-manipulation-both-ways → rejection unchanged."
//!   - R0.5 plan §3.4/§3.10 (`...f-full-r0-plan.md:540-545,776-789`): "an OUTER
//!     UCAN-token/`jti`-keyed nonce-cache that is NET-NEW at v1-beta ... re-uses
//!     the Compromise #25 durable-CAS-marker *pattern*, NOT the #25 sync-frame
//!     instance". Scope/retention/durability/multi-device per NQ-T4.
//!
//! ## NQ-T4 RATIFIED (§10.5; Ben 2026-06-02)
//!
//! NQ-T4 (§10.5) — the nonce-cache spec — is **RATIFIED**. The cache is
//! `jti`-keyed, durable (survives engine restart), retention ≥ the full 1-hr
//! bucket. **Scope (ratified default):** **per-device-durable is GUARANTEED**;
//! **user-global is best-effort-eventual-via-sync (NOT synchronous)** — a nonce
//! consumed on device B is rejected on device C **only after** sync propagates
//! the cache entry. The `..._multi_device_shared_rejection_nq_t4_gated` arm
//! below pins this ratified contract in BOTH directions: (a) the load-bearing
//! **pre-sync** positive control — a fresh device C, before sync reaches it,
//! ADMITS the same jti (the disclosed best-effort-eventual window); and
//! (b) the **post-sync** rejection — once B's consumed-jti set syncs to C, C
//! rejects it. The pre-sync window is the disclosed Compromise #64
//! ("best-effort-eventual cross-device nonce-rejection window"); see the
//! SECURITY-POSTURE doc-cascade (M-1). The arm stays `#[ignore]`'d (RED-PHASE)
//! until R5 wires the real Layer-D nonce-cache.
//!
//! ## Compromise #64 — best-effort-eventual cross-device window (FLAG-FOR-BEN)
//!
//! NQ-T4's ratification (§10.5) mandates minting a named Compromise for the
//! pre-sync cross-device replay window. The synthesis assigns **#64**; the
//! SECURITY-POSTURE row + #64 number are FLAGGED-FOR-BEN to ratify in the
//! doc-cascade. The pre-sync sub-arm below is bound to that disclosure: it is
//! the failing would-FAIL-on-no-op control that demonstrates the window is real
//! (a fresh device C admits the jti) rather than synchronous/guaranteed.
//!
//! ## NET-NEW jti-keyed instance (R0.5 §3.10 precision)
//!
//! This is a DISTINCT `jti`-keyed nonce-cache, NOT the shipped #25 sync-frame
//! cache. The shim below models the jti-keyed durable CAS-marker; R5 replaces
//! it with the real Layer-D nonce-cache.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![cfg(not(target_arch = "wasm32"))]

// ---------------------------------------------------------------------------
// SELF-CONTAINED STUB-SHIM — jti-keyed durable nonce-cache (NET-NEW at v1-beta).
// ---------------------------------------------------------------------------
//
// The jti-keyed cache is the UCAN-token `jti` (a 32-byte nonce) keyed durable
// CAS-marker. Retention MUST cover ≥ the full 1-hr bucket; the rejection is
// keyed on the nonce, NOT the timestamp. (R5 deletes this whole module and
// `use`s the real Layer-D nonce-cache.)
mod cache {
    use std::collections::HashSet;

    pub const LAYER_D_BUCKET_SECS: u64 = 3600;

    #[derive(Debug, PartialEq, Eq)]
    pub enum NonceCacheError {
        ReplayedNonce,
    }

    /// jti-keyed durable nonce-cache. `enabled=false` models the
    /// "disable-cache negative" control: with the cache off, a replay is NOT
    /// caught (proves the bucket alone is insufficient).
    pub struct JtiNonceCache {
        seen: HashSet<[u8; 32]>,
        /// The durable backing set (survives a simulated restart).
        durable_store: HashSet<[u8; 32]>,
        enabled: bool,
    }

    impl JtiNonceCache {
        pub fn new(enabled: bool) -> Self {
            Self {
                seen: HashSet::new(),
                durable_store: HashSet::new(),
                enabled,
            }
        }

        /// Hydrate from the durable store (simulates engine restart, OR a
        /// cross-device sync that propagated another device's consumed-jti
        /// set into this device's durable cache).
        pub fn from_durable(durable: HashSet<[u8; 32]>) -> Self {
            Self {
                seen: durable.clone(),
                durable_store: durable,
                enabled: true,
            }
        }

        pub fn durable_snapshot(&self) -> HashSet<[u8; 32]> {
            self.durable_store.clone()
        }

        /// Record-and-check is atomic. The check is keyed ONLY on the jti
        /// nonce — the `_present_at_secs` arg is accepted to PROVE the
        /// rejection is nonce-keyed, NOT time-keyed (it is never read for the
        /// replay decision).
        pub fn admit(&mut self, jti: [u8; 32], _present_at_secs: u64) -> Result<(), NonceCacheError> {
            if !self.enabled {
                // Disable-cache control: always admit (replay leaks through).
                return Ok(());
            }
            if self.seen.contains(&jti) {
                return Err(NonceCacheError::ReplayedNonce);
            }
            self.seen.insert(jti);
            self.durable_store.insert(jti);
            Ok(())
        }
    }
}

use cache::{JtiNonceCache, NonceCacheError, LAYER_D_BUCKET_SECS};

/// F-LD-5 HEADLINE: present a PermissionGrant jti twice within the same 1-hr
/// bucket → the 2nd is rejected BY THE NONCE-CACHE (not by a time field). The
/// two presentations carry the SAME bucket-aligned time, so a time-only check
/// could never distinguish them — only the nonce-cache catches the replay.
/// would-FAIL-if-no-op'd: if `admit` skipped the seen-set check, the 2nd admit
/// returns Ok.
#[test]
#[ignore = "RED-PHASE: F-LD-5 — intra-hour replay rejected by jti nonce-cache (M-1); un-ignore at R5"]
fn f_ld_5_intra_hour_replay_rejected_by_nonce_cache() {
    let mut nc = JtiNonceCache::new(true);
    let jti = [0xAB; 32];
    let bucket_aligned = 1_900_000_800; // both presentations same coarse bucket

    nc.admit(jti, bucket_aligned)
        .expect("first presentation of a fresh jti MUST be admitted");

    let err = nc
        .admit(jti, bucket_aligned)
        .expect_err("intra-bucket replay of the SAME jti MUST be rejected by the nonce-cache");
    assert_eq!(
        err,
        NonceCacheError::ReplayedNonce,
        "rejection is nonce-keyed (jti), NOT time-keyed"
    );
}

/// F-LD-5 DISABLE-CACHE NEGATIVE (the load-bearing control): with the nonce-
/// cache OFF, the same intra-bucket replay is NOT caught — proving the coarse
/// 1-hr bucket ALONE is insufficient + the nonce-cache is the actual replay
/// defense (M-1). If this passed only because of a time check, disabling the
/// cache wouldn't change the outcome — but it does.
#[test]
#[ignore = "RED-PHASE: F-LD-5 — disable-cache control: replay leaks (bucket alone insufficient); un-ignore at R5"]
fn f_ld_5_disable_cache_lets_replay_through_proving_bucket_insufficient() {
    let mut nc = JtiNonceCache::new(false); // cache DISABLED
    let jti = [0xAB; 32];
    let bucket_aligned = 1_900_000_800;

    nc.admit(jti, bucket_aligned).expect("first admit");
    // With the cache off, the SAME jti in the SAME bucket is admitted again —
    // demonstrating the bucket does NOT defend replay on its own.
    nc.admit(jti, bucket_aligned)
        .expect("with the nonce-cache DISABLED, intra-bucket replay leaks through (bucket insufficient)");
}

/// F-LD-5 RESTART-DURABILITY: the nonce-cache survives engine restart. A jti
/// consumed before a restart is still rejected after rehydrating from the
/// durable store — so a restart can't be used to bypass the replay window.
#[test]
#[ignore = "RED-PHASE: F-LD-5 — nonce-cache durable across restart; un-ignore at R5"]
fn f_ld_5_nonce_cache_durable_across_restart() {
    let jti = [0xCD; 32];

    // Pre-restart: consume the jti.
    let mut nc = JtiNonceCache::new(true);
    nc.admit(jti, 1_900_000_800).expect("first admit pre-restart");
    let durable = nc.durable_snapshot();
    drop(nc); // simulate engine shutdown

    // Post-restart: rehydrate from durable store → replay still rejected.
    let mut nc2 = JtiNonceCache::from_durable(durable);
    let err = nc2
        .admit(jti, 1_900_004_400) // later bucket, but jti still cached
        .expect_err("a jti consumed pre-restart MUST still reject after restart");
    assert_eq!(err, NonceCacheError::ReplayedNonce);
}

/// F-LD-5 CLOCK-MANIPULATION BOTH WAYS: moving the presented clock forward OR
/// backward does NOT change the nonce-cache rejection. Because the cache is
/// nonce-keyed, an attacker who rewrites the timestamp cannot evade the replay
/// catch. would-FAIL-if-no-op'd: a time-keyed cache would let a clock-shifted
/// replay through.
#[test]
#[ignore = "RED-PHASE: F-LD-5 — clock manipulation does not bypass nonce-keyed rejection; un-ignore at R5"]
fn f_ld_5_clock_manipulation_does_not_bypass_nonce_cache() {
    let mut nc = JtiNonceCache::new(true);
    let jti = [0xEF; 32];

    nc.admit(jti, 1_900_000_800).expect("first admit");

    // Replay with the clock moved BACKWARD (earlier bucket).
    assert_eq!(
        nc.admit(jti, 1_900_000_800 - LAYER_D_BUCKET_SECS * 10),
        Err(NonceCacheError::ReplayedNonce),
        "clock-backward replay MUST still reject (nonce-keyed)"
    );
    // Replay with the clock moved FORWARD (later bucket).
    assert_eq!(
        nc.admit(jti, 1_900_000_800 + LAYER_D_BUCKET_SECS * 10),
        Err(NonceCacheError::ReplayedNonce),
        "clock-forward replay MUST still reject (nonce-keyed)"
    );
}

/// F-LD-5 distinct-jti admitted: a DIFFERENT jti in the same bucket is admitted
/// (the cache rejects replays, not all intra-bucket traffic). Proves the cache
/// is keyed on the nonce, not the bucket.
#[test]
#[ignore = "RED-PHASE: F-LD-5 — distinct jti in same bucket is admitted; un-ignore at R5"]
fn f_ld_5_distinct_jti_same_bucket_is_admitted() {
    let mut nc = JtiNonceCache::new(true);
    let bucket = 1_900_000_800;
    nc.admit([0x01; 32], bucket).expect("jti #1 admitted");
    nc.admit([0x02; 32], bucket)
        .expect("a DISTINCT jti in the same bucket MUST be admitted (keyed on nonce, not bucket)");
}

/// F-LD-5 NQ-T4-GATED multi-device shared-rejection (RATIFIED §10.5).
///
/// NQ-T4 (§10.5) is **RATIFIED**: per-device-durable is GUARANTEED; user-global
/// is **best-effort-eventual-via-sync (NOT synchronous)**. This arm pins that
/// contract in BOTH directions so it reads as the eventual-not-synchronous
/// guarantee it actually is — NOT as if cross-device rejection were immediate.
///
/// 1. **PRE-SYNC POSITIVE CONTROL (load-bearing; Compromise #64):** a fresh
///    device C that has NOT yet received B's consumed-jti set via sync ADMITS
///    the same jti — `Ok(())`. This is the disclosed best-effort-eventual
///    window (the gap between consumption on B and propagation to C). It is the
///    would-FAIL-on-no-op control: a (false) synchronous-global cache would
///    have to reject here, but the ratified contract admits.
/// 2. **POST-SYNC REJECTION:** once B's durable consumed-jti set has synced into
///    device C, C rejects the same jti — closing the window after propagation.
///
/// The pre-sync arm is bound to **Compromise #64** ("best-effort-eventual
/// cross-device nonce-rejection window"; FLAG-FOR-BEN on the #64 number + the
/// SECURITY-POSTURE row). The exact wire/sync mechanism is finalized at R5
/// against the real Layer-D nonce-cache; the eventual-not-synchronous SHAPE is
/// frozen here.
#[test]
#[ignore = "RED-PHASE: F-LD-5 — NQ-T4-gated multi-device shared rejection (RATIFIED §10.5: per-device GUARANTEED + user-global best-effort-eventual; pre-sync window = Compromise #64); un-ignore at R5"]
fn f_ld_5_multi_device_shared_rejection_nq_t4_gated() {
    let jti = [0x77; 32];

    // Device B consumes the jti.
    let mut device_b = JtiNonceCache::new(true);
    device_b.admit(jti, 1_900_000_800).expect("B admits");

    // (1) PRE-SYNC POSITIVE CONTROL — Compromise #64 best-effort-eventual
    //     window. A fresh device C that has NOT received B's consumed-jti set
    //     (no durable hydration / sync has not propagated) ADMITS the same jti.
    //     This is the disclosed cross-device replay window; a synchronous-global
    //     cache would (wrongly) reject here. would-FAIL-on-no-op: if the cache
    //     were synchronous-global the admit would return Err.
    let mut device_c_pre_sync = JtiNonceCache::new(true); // no sync yet
    device_c_pre_sync
        .admit(jti, 1_900_000_800)
        .expect(
            "PRE-SYNC: a fresh device C (sync has not propagated B's jti) MUST ADMIT the same jti \
             — the disclosed best-effort-eventual cross-device window (Compromise #64)",
        );

    // (2) POST-SYNC REJECTION — after best-effort-global sync, B's durable
    //     consumed-jti set reaches device C; C now rejects the same jti.
    let mut device_c_post_sync = JtiNonceCache::from_durable(device_b.durable_snapshot());
    let err = device_c_post_sync
        .admit(jti, 1_900_000_800)
        .expect_err("POST-SYNC: device C MUST reject a jti device B consumed (NQ-T4 ratified default)");
    assert_eq!(err, NonceCacheError::ReplayedNonce);
}
