//! F-LD-8 — Layer-D timestamp exclusion `DropToRecipient` (RED-PHASE; byte-pinning).
//!
//! R3 wave **W3-layer-d**. Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 8
//!     F-LD-8: "`DropToRecipient`/Sealed-Sender struct carries NO `sealed_at`/
//!     `valid_until` (L6 finding closed-by-exclusion); 1-hr bucket on DeviceLink+
//!     RemotePermission ONLY; round-down-no-jitter (NQ-C5 RATIFIED); bucket ⊥
//!     `valid_until` clock (NQ-T2 RATIFIED)." Red-phase: "structural: drop struct
//!     has NO timestamp field; DeviceLink/RemotePermission DO; `bucket % 3600 == 0`
//!     no-jitter."
//!   - R0.5 plan §3.10 (`...f-full-r0-plan.md:757-800`): "DropToRecipient carries
//!     NO sealed_at/valid_until — drops are forever-valid (per #62) ... The U28
//!     1-hour coarse bucket applies ONLY to the DeviceLink + RemotePermission
//!     epoch fields ... Default = round-down, NO jitter."
//!   - Compromise #62 (revocation-reach), L6 drop-timestamp HIGH-leak finding.
//!
//! ## NQ-C5 — RATIFIED (Ben 2026-06-02)
//!
//! NQ-C5 (§10.6:1439) is **RATIFIED**: the epoch bucket =
//! `(raw_unix_secs / 3600) * 3600` — **round-DOWN, NO jitter**, deterministic
//! (`bucket % 3600 == 0`). With no jitter there is no jitter↔skew interaction to
//! double-widen the effective window (m-9). The **nonce-cache window does NOT
//! need widening** — bucket (metadata privacy) and nonce-cache (replay defense)
//! are **orthogonal** and tuned independently. The
//! `..._round_down_no_jitter_nq_c5_gated` arm pins this ratified default; R5
//! un-ignores it against the same ratified rule (the open-question framing is
//! resolved — no R5 re-decision pending).
//!
//! ## byte-pinning (M-20)
//!
//! V2 + BE from the first commit. The structural pin here is the ABSENCE of a
//! timestamp field on the drop struct — enforced via a serialized-shape check.
//!
//! ## R4.4 falsifiability strengthening (F-LD-8-TAUTOLOGY)
//!
//! The earlier `..._has_no_timestamp_bytes` arm cloned `drop_a` into `drop_b`
//! and asserted byte-equality — VACUOUS: a clone is byte-identical regardless of
//! whether a timestamp field exists, so a timestamp-leaking impl would still
//! pass. That arm is replaced below by a **differential** check: two drops built
//! in two distinct wall-clock contexts (modelled by a `sealed_at_context` an
//! impl might capture) with identical generation+ciphertext MUST serialize to
//! IDENTICAL bytes. A timestamp-carrying `to_wire_be` would emit the two contexts
//! as differing bytes and FAIL the equality — so this arm is would-FAIL-on-no-op
//! against the exact regression M-14 forbids. The structural field-name invariant
//! remains carried by `..._carries_no_timestamp_field` below.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![cfg(not(target_arch = "wasm32"))]

// ---------------------------------------------------------------------------
// SELF-CONTAINED STUB-SHIM — the three Layer-D timestamp postures.
// ---------------------------------------------------------------------------
mod shim {
    /// 1-hour bucket width (M-14: DeviceLink + RemotePermission ONLY).
    pub const LAYER_D_BUCKET_SECS: u64 = 3600;

    /// `DropToRecipient` (Sealed-Sender drop) — carries NO timestamp. Drops are
    /// forever-valid (#62); freshness rides recipient-key-generation + the
    /// nonce-cache, NOT a timestamp. The struct has NO `sealed_at`/`valid_until`
    /// field BY CONSTRUCTION (L6 closed-by-exclusion).
    #[derive(Clone)]
    pub struct DropToRecipient {
        pub version: u8,
        pub recipient_key_generation: u32,
        pub hpke_ciphertext: Vec<u8>,
        // INTENTIONALLY NO sealed_at / valid_until fields.
        //
        // `sealed_at_context` is a SHIM-ONLY field that models the wall-clock an
        // implementer *might* be tempted to capture at seal time. It is NOT a
        // wire field: `to_wire_be` / `wire_field_names` MUST ignore it. The
        // differential arm builds two drops with DIFFERENT contexts and asserts
        // their wire bytes are EQUAL — a timestamp-leaking `to_wire_be` would
        // serialize the two contexts differently and fail (would-FAIL-on-no-op).
        pub sealed_at_context: u64,
    }

    impl DropToRecipient {
        /// Canonical wire bytes (V2, BE). The field set is exactly
        /// {version, recipient_key_generation, ciphertext} — no timestamp.
        ///
        /// NOTE: `sealed_at_context` is DELIBERATELY NOT serialized. That is the
        /// whole point of the exclusion. A regression that leaks the timestamp
        /// would append/encode `sealed_at_context` here and break the
        /// differential equality arm.
        pub fn to_wire_be(&self) -> Vec<u8> {
            let mut b = Vec::new();
            b.push(self.version);
            b.extend_from_slice(&self.recipient_key_generation.to_be_bytes());
            b.extend_from_slice(&(self.hpke_ciphertext.len() as u32).to_be_bytes());
            b.extend_from_slice(&self.hpke_ciphertext);
            // INTENTIONALLY: self.sealed_at_context is NOT encoded.
            b
        }

        /// The wire field-name set the struct serializes (for the structural
        /// exclusion pin). R5 swaps this for a serde-introspection / golden-CBOR
        /// field-name check against the real struct. `sealed_at_context` is a
        /// shim-only non-wire field and MUST NOT appear here.
        pub fn wire_field_names() -> &'static [&'static str] {
            &["version", "recipient_key_generation", "hpke_ciphertext"]
        }
    }

    /// DeviceLink + RemotePermission DO carry a coarse 1-hr epoch bucket (U5).
    #[derive(Clone)]
    pub struct DeviceLinkEpoch {
        pub version: u8,
        pub granted_at_bucket: u64,
    }

    /// Round a raw unix-seconds time DOWN to the 1-hr bucket — NO jitter
    /// (RATIFIED per m-9/NQ-C5). The result is always a multiple of 3600.
    pub fn round_down_to_bucket(now_secs: u64) -> u64 {
        (now_secs / LAYER_D_BUCKET_SECS) * LAYER_D_BUCKET_SECS
    }
}

use shim::{round_down_to_bucket, DeviceLinkEpoch, DropToRecipient, LAYER_D_BUCKET_SECS};

/// F-LD-8 STRUCTURAL EXCLUSION: `DropToRecipient` has NO `sealed_at`/`valid_until`
/// field. An implementer following the L6 struct would leak ~1-sec timestamps
/// the bucket never reaches; the exclusion is structural so the struct is never
/// built. would-FAIL-if-no-op'd: adding a timestamp field name flips this pin.
#[test]
#[ignore = "RED-PHASE: F-LD-8 — DropToRecipient has NO timestamp field (L6 closed-by-exclusion); un-ignore at R5"]
fn f_ld_8_drop_to_recipient_carries_no_timestamp_field() {
    let names = DropToRecipient::wire_field_names();
    assert!(
        !names.contains(&"sealed_at") && !names.contains(&"valid_until"),
        "DropToRecipient MUST NOT carry sealed_at/valid_until (forever-valid per #62; L6 closed-by-exclusion); fields = {names:?}"
    );
    // The shim-only timestamp context MUST NOT have leaked into the wire field set.
    assert!(
        !names.contains(&"sealed_at_context"),
        "sealed_at_context is a non-wire shim field; it MUST NOT appear in the wire field-name set"
    );
    // Positive shape: it DOES carry the generation field that freshness rides on.
    assert!(
        names.contains(&"recipient_key_generation"),
        "DropToRecipient freshness rides recipient_key_generation, NOT a timestamp"
    );
}

/// F-LD-8 STRUCTURAL EXCLUSION (wire-byte level, DIFFERENTIAL): no timestamp
/// bytes leak into the serialized drop. Two drops "sealed" in two DIFFERENT
/// wall-clock contexts (`sealed_at_context` 1 hour apart) — but with identical
/// generation + ciphertext — MUST serialize to IDENTICAL bytes, proving the
/// seal-time wall-clock is excluded from the wire.
///
/// would-FAIL-if-no-op'd: a `to_wire_be` that encodes `sealed_at_context`
/// (i.e. a timestamp-leaking impl) would emit the two contexts as DIFFERING
/// bytes and fail this equality. This is the exact regression M-14 / Compromise
/// #62 / the L6 HIGH-leak finding forbid. (The earlier R3 form cloned the
/// struct and asserted self-equality — vacuous; replaced here per R4.4
/// F-LD-8-TAUTOLOGY. The struct-level field-name invariant is carried by
/// `f_ld_8_drop_to_recipient_carries_no_timestamp_field` above.)
#[test]
#[ignore = "RED-PHASE: F-LD-8 — no timestamp bytes in serialized drop (differential); un-ignore at R5"]
fn f_ld_8_drop_serialization_has_no_timestamp_bytes() {
    // Two distinct seal-time contexts, exactly one bucket apart, so a
    // bucket-granularity leak would ALSO be caught (not just sub-second).
    let early_ctx = 1_900_001_234u64;
    let late_ctx = early_ctx + LAYER_D_BUCKET_SECS + 777; // > 1 hour later, off-bucket
    assert_ne!(
        round_down_to_bucket(early_ctx),
        round_down_to_bucket(late_ctx),
        "test scaffold sanity: the two contexts fall in DIFFERENT buckets, so even a \
         bucket-granularity timestamp leak would diverge the wire bytes"
    );

    let drop_early = DropToRecipient {
        version: 2,
        recipient_key_generation: 7,
        hpke_ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
        sealed_at_context: early_ctx,
    };
    // Same generation + ciphertext; ONLY the seal-time context differs.
    let drop_late = DropToRecipient {
        version: 2,
        recipient_key_generation: 7,
        hpke_ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
        sealed_at_context: late_ctx,
    };

    assert_eq!(
        drop_early.to_wire_be(),
        drop_late.to_wire_be(),
        "two drops sealed >1 hour apart (identical generation+ciphertext) MUST serialize \
         IDENTICALLY — the seal-time wall-clock is NOT on the wire (M-14; L6 closed-by-exclusion). \
         A timestamp-leaking to_wire_be would diverge these bytes and fail."
    );

    // Cross-check the differential is meaningful: the wire bytes are exactly the
    // {version, generation, ciphertext} encoding and contain NEITHER context's
    // little/big-endian byte-pattern (no timestamp magnitude leaked).
    let wire = drop_early.to_wire_be();
    for ctx in [early_ctx, late_ctx] {
        let be = ctx.to_be_bytes();
        let le = ctx.to_le_bytes();
        assert!(
            !wire.windows(8).any(|w| w == be || w == le),
            "no seal-time wall-clock magnitude ({ctx}) may appear in the drop wire bytes"
        );
    }

    // V2 framing pin (M-20).
    assert_eq!(drop_early.version, 2, "drop wire is V2 from first commit");
}

/// F-LD-8 DeviceLink/RemotePermission DO carry the bucket (the inverse pin):
/// the 1-hr bucket applies ONLY to these Layer-D surfaces, not to drops.
#[test]
#[ignore = "RED-PHASE: F-LD-8 — DeviceLink/RemotePermission DO carry the 1-hr bucket; un-ignore at R5"]
fn f_ld_8_device_link_carries_the_one_hour_bucket() {
    let epoch = DeviceLinkEpoch {
        version: 2,
        granted_at_bucket: round_down_to_bucket(1_900_001_234),
    };
    // The bucket is present + non-zero on the DeviceLink surface.
    assert_ne!(epoch.granted_at_bucket, 0);
    assert_eq!(
        epoch.granted_at_bucket % LAYER_D_BUCKET_SECS,
        0,
        "the DeviceLink bucket field is bucket-aligned"
    );
}

/// F-LD-8 NQ-C5 round-down-NO-jitter (RATIFIED, Ben 2026-06-02): the bucket is
/// computed by pure round-down (multiple of 3600), with NO jitter added.
/// Jitter↔skew can double-widen the effective window to ≤2×jitter+skew;
/// round-down-no-jitter avoids it. The bucket is a pure function of the raw time
/// (deterministic; same input → same bucket; no randomness).
///
/// NQ-C5 (§10.6:1439) is RATIFIED: round-down, no jitter; the nonce-cache window
/// is NOT widened (bucket and nonce-cache are orthogonal). This arm pins that
/// ratified rule; R5 un-ignores it against the same rule (no R5 re-decision).
#[test]
#[ignore = "RED-PHASE: F-LD-8 — round-down NO-jitter bucket (NQ-C5 RATIFIED §10.6); un-ignore at R5"]
fn f_ld_8_bucket_is_round_down_no_jitter_nq_c5_gated() {
    // Any raw time → a multiple of 3600 (no jitter offset).
    for raw in [0u64, 1, 3599, 3600, 3601, 1_900_001_234, 1_900_004_799] {
        let bucket = round_down_to_bucket(raw);
        assert_eq!(
            bucket % LAYER_D_BUCKET_SECS,
            0,
            "bucket({raw}) MUST be a clean multiple of {LAYER_D_BUCKET_SECS} (round-down, NO jitter)"
        );
        assert!(bucket <= raw, "round-DOWN: bucket never exceeds the raw time");
        assert!(raw - bucket < LAYER_D_BUCKET_SECS, "bucket is within one window of raw");
    }
    // Determinism: same input → same bucket (no randomized jitter).
    assert_eq!(
        round_down_to_bucket(1_900_001_234),
        round_down_to_bucket(1_900_001_234),
        "the bucket is deterministic — NO jitter randomness (avoids the ≤2×jitter+skew widening)"
    );
}
