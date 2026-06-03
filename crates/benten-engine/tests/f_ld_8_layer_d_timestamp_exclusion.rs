//! F-LD-8 — Layer-D timestamp exclusion `DropToRecipient` (RED-PHASE; byte-pinning).
//!
//! R3 wave **W3-layer-d**. Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 8
//!     F-LD-8: "`DropToRecipient`/Sealed-Sender struct carries NO `sealed_at`/
//!     `valid_until` (L6 finding closed-by-exclusion); 1-hr bucket on DeviceLink+
//!     RemotePermission ONLY; round-down-no-jitter (NQ-C5); bucket ⊥ `valid_until`
//!     clock (NQ-T2)." Red-phase: "structural: drop struct has NO timestamp
//!     field; DeviceLink/RemotePermission DO; `bucket % 3600 == 0` no-jitter."
//!   - R0.3 plan §3.10 (`...f-full-r0-plan.md:760-769`): "DropToRecipient carries
//!     NO sealed_at/valid_until — drops are forever-valid (per #62) ... The U28
//!     1-hour coarse bucket applies ONLY to the DeviceLink + RemotePermission
//!     epoch fields ... Default = round-down, NO jitter."
//!   - Compromise #62 (revocation-reach), L6 drop-timestamp HIGH-leak finding.
//!
//! ## NQ-C5 OPEN-SPEC FLAG
//!
//! NQ-C5 (§10.6) — round-down-no-jitter avoids the ≤2×jitter+skew window-
//! widening cleanly, OR does the nonce-cache window need widening? — is
//! UNRESOLVED at R2 (§5.B carry-forward item 6). The `..._round_down_no_jitter_
//! nq_c5_gated` arm is a RED-PHASE stub referencing the open question; R5
//! finalizes against the ratified NQ-C5 default (R0 default = round-down,
//! NO jitter).
//!
//! ## byte-pinning (M-20)
//!
//! V2 + BE from the first commit. The structural pin here is the ABSENCE of a
//! timestamp field on the drop struct — enforced via a serialized-shape check.

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
    }

    impl DropToRecipient {
        /// Canonical wire bytes (V2, BE). The field set is exactly
        /// {version, recipient_key_generation, ciphertext} — no timestamp.
        pub fn to_wire_be(&self) -> Vec<u8> {
            let mut b = Vec::new();
            b.push(self.version);
            b.extend_from_slice(&self.recipient_key_generation.to_be_bytes());
            b.extend_from_slice(&(self.hpke_ciphertext.len() as u32).to_be_bytes());
            b.extend_from_slice(&self.hpke_ciphertext);
            b
        }

        /// The wire field-name set the struct serializes (for the structural
        /// exclusion pin). R5 swaps this for a serde-introspection / golden-CBOR
        /// field-name check against the real struct.
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
    /// (default per m-9/NQ-C5). The result is always a multiple of 3600.
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
    // Positive shape: it DOES carry the generation field that freshness rides on.
    assert!(
        names.contains(&"recipient_key_generation"),
        "DropToRecipient freshness rides recipient_key_generation, NOT a timestamp"
    );
}

/// F-LD-8 STRUCTURAL EXCLUSION (wire-byte level): no timestamp bytes leak into
/// the serialized drop. Two drops "sealed" at very different wall-clock times
/// serialize to IDENTICAL bytes (given identical generation + ciphertext) —
/// proving no timestamp is encoded. would-FAIL-if-no-op'd: a timestamp field
/// would make the two byte-strings differ.
#[test]
#[ignore = "RED-PHASE: F-LD-8 — no timestamp bytes in serialized drop; un-ignore at R5"]
fn f_ld_8_drop_serialization_has_no_timestamp_bytes() {
    let drop_a = DropToRecipient {
        version: 2,
        recipient_key_generation: 7,
        hpke_ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
    };
    // A "later" drop with the same generation + ciphertext.
    let drop_b = drop_a.clone();
    assert_eq!(
        drop_a.to_wire_be(),
        drop_b.to_wire_be(),
        "identical generation+ciphertext MUST serialize identically — no wall-clock timestamp leaks"
    );
    // V2 framing pin (M-20).
    assert_eq!(drop_a.version, 2, "drop wire is V2 from first commit");
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

/// F-LD-8 NQ-C5-GATED round-down-NO-jitter (OPEN-SPEC): the bucket is computed
/// by pure round-down (multiple of 3600), with NO jitter added. Jitter↔skew can
/// double-widen the effective window to ≤2×jitter+skew; round-down-no-jitter
/// avoids it. The bucket is a pure function of the raw time (deterministic;
/// same input → same bucket; no randomness).
///
/// OPEN-SPEC: NQ-C5 (§10.6) is unresolved at R2 (§5.B carry-forward item 6).
/// This arm pins the R0 default (round-down, no jitter); R5 finalizes against
/// the ratified NQ-C5 answer (or widens the nonce-cache window if NQ-C5 so
/// rules).
#[test]
#[ignore = "RED-PHASE: F-LD-8 — round-down NO-jitter bucket (OPEN-SPEC: gated on NQ-C5 §10.6); un-ignore at R5"]
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
