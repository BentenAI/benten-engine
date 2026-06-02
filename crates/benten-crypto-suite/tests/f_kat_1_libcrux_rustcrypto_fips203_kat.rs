//! **F-KAT-1 — libcrux ↔ RustCrypto FIPS-203 serialization KAT.**
//! (merges CE-D1 + WF-G2; the m-2 Wave-0 exit gate)
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-4 F-KAT-1 + §4 P0-4 + NQ-C2.
//!   - R0 §2.2 Q1: ML-KEM impl = **libcrux-ml-kem** (verified secret-independence
//!     via hax/F*). "**NQ-C2 (R2):** the swap MUST preserve FIPS-203
//!     encap/ciphertext/decap-key serialization byte-for-byte (cross-impl KAT)
//!     or golden vectors break."
//!   - R0 §3.2 Wave-0 exit gate (m-2): "a FIPS-203 byte-compatibility KAT
//!     (RustCrypto `ml-kem 0.2` ↔ libcrux)".
//!   - CLAUDE.md baked-in #5 (never-fork-primitives; the swap is impl-only, the
//!     serialized bytes are FIPS-203-canonical and MUST survive).
//!
//! # What this pins (FREEZE-GATING / CF — golden-vector survival)
//!
//! The libcrux-ml-kem swap (away from RustCrypto `ml-kem 0.2`) MUST preserve the
//! FIPS-203 encapsulation-key / ciphertext / decapsulation-key serialization
//! byte-for-byte. If it does not, every golden vector + every on-wire ML-KEM-768
//! byte breaks. Pins (against a deterministic synthesized FIPS-203 witness — the
//! `tf4 load_fips_204_kat_vector_for_test` precedent — until the real NIST KAT
//! corpus lands at R5):
//!   1. for a fixed KAT seed, both impls produce byte-identical encapsulation
//!      keys + ciphertexts + decapsulation-key serializations (cross-impl
//!      byte-equality);
//!   2. ML-KEM-768 sizes are FIPS-203-exact (ek=1184, ct=1088, dk=2400, ss=32);
//!   3. negative: a wrong seed → a DIFFERENT ciphertext (the KAT is genuinely
//!      seed-bound, not a constant).
//!
//! ## ORCHESTRATOR-FLAGGED EXTERNAL-VECTOR SEED (R2 §5-D-9; NQ-C2)
//!
//! Ground-truth at HEAD: `libcrux-ml-kem` is NOT yet a workspace dep (`grep -rn
//! libcrux crates/*/Cargo.toml Cargo.toml` → ZERO; RustCrypto `ml-kem` is the
//! in-tree impl). The real NIST FIPS-203 KAT corpus + the libcrux swap arrive at
//! R5. This file uses a **deterministic synthesized witness** (per the
//! `tf4 load_fips_204_kat_vector_for_test` precedent) so the cross-impl
//! byte-equality SHAPE is pinned now; **R5 swaps in the real FIPS-203 corpus +
//! both real impls and verifies byte-equality.** Flagged for R5 real-corpus
//! swap.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! Per wave-independence this file commits a LOCAL `f_kat_1_stub`. The stub's
//! two "impls" deliberately DIVERGE (libcrux witness ≠ rustcrypto witness for
//! the same seed) so the cross-impl byte-equality pin FAILS until R5 wires both
//! real impls against the shared FIPS-203 corpus — never a silent-green
//! SHAPE-trap. R5 DELETEs the stub + wires the real `ml-kem` + `libcrux-ml-kem`
//! KAT round-trip.

#![allow(dead_code)]

/// SELF-CONTAINED stub-shim (R5 deletes + wires real `ml-kem` ↔ `libcrux-ml-kem`
/// FIPS-203 KAT round-trip).
mod f_kat_1_stub {
    /// FIPS-203 ML-KEM-768 serialized sizes (exact; never hardcode-without-pin).
    pub const ML_KEM_768_EK_LEN: usize = 1184; // encapsulation key
    pub const ML_KEM_768_CT_LEN: usize = 1088; // ciphertext
    pub const ML_KEM_768_DK_LEN: usize = 2400; // decapsulation key
    pub const ML_KEM_768_SS_LEN: usize = 32; // shared secret

    /// A deterministic-synthesized FIPS-203 witness for one named KAT seed
    /// (the `tf4 load_fips_204_kat_vector_for_test` precedent). The bytes are
    /// derived from the seed so the witness is reproducible + seed-bound.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MlKem768Kat {
        pub encapsulation_key: Vec<u8>,
        pub ciphertext: Vec<u8>,
        pub decapsulation_key: Vec<u8>,
        pub shared_secret: Vec<u8>,
    }

    fn synthesize(seed: &[u8; 32], impl_tag: u8, len: usize) -> Vec<u8> {
        // Deterministic, seed-bound, impl-tagged filler. The `impl_tag`
        // distinguishes the two stub "impls" so they DIVERGE (red-phase).
        // R5's real impls share the FIPS-203 corpus and AGREE (impl_tag drops).
        (0..len)
            .map(|i| seed[i % 32] ^ impl_tag ^ (i as u8))
            .collect()
    }

    /// Load the FIPS-203 witness as the **libcrux** impl would serialize it.
    pub fn libcrux_witness(seed: &[u8; 32]) -> MlKem768Kat {
        MlKem768Kat {
            encapsulation_key: synthesize(seed, /* libcrux */ 0x01, ML_KEM_768_EK_LEN),
            ciphertext: synthesize(seed, 0x01, ML_KEM_768_CT_LEN),
            decapsulation_key: synthesize(seed, 0x01, ML_KEM_768_DK_LEN),
            shared_secret: synthesize(seed, 0x01, ML_KEM_768_SS_LEN),
        }
    }

    /// Load the FIPS-203 witness as the **RustCrypto `ml-kem`** impl would
    /// serialize it. RED-PHASE: impl_tag=0x02 ≠ libcrux's 0x01, so the two
    /// DIVERGE (the byte-equality pin fails). R5: both share the real corpus.
    pub fn rustcrypto_witness(seed: &[u8; 32]) -> MlKem768Kat {
        MlKem768Kat {
            encapsulation_key: synthesize(seed, /* rustcrypto */ 0x02, ML_KEM_768_EK_LEN),
            ciphertext: synthesize(seed, 0x02, ML_KEM_768_CT_LEN),
            decapsulation_key: synthesize(seed, 0x02, ML_KEM_768_DK_LEN),
            shared_secret: synthesize(seed, 0x02, ML_KEM_768_SS_LEN),
        }
    }
}

use f_kat_1_stub::{
    ML_KEM_768_CT_LEN, ML_KEM_768_DK_LEN, ML_KEM_768_EK_LEN, ML_KEM_768_SS_LEN, libcrux_witness,
    rustcrypto_witness,
};

const KAT_SEED: [u8; 32] = [0x07u8; 32];

/// F-KAT-1 (a) — cross-impl FIPS-203 byte-equality (NQ-C2; the m-2 exit gate).
///
/// For a fixed KAT seed, libcrux + RustCrypto MUST serialize byte-identical
/// encapsulation keys, ciphertexts, decapsulation keys, and shared secrets.
/// would-FAIL-if-no-op'd: the stub's two impls diverge (impl-tagged), so this
/// fails until R5 wires both real impls against the shared corpus. A swap that
/// changed any serialized byte breaks every golden vector.
#[test]
#[ignore = "RED-PHASE: F-KAT-1 — libcrux ↔ RustCrypto FIPS-203 serialization MUST be byte-identical (NQ-C2); un-ignore + real-corpus swap at R5"]
fn libcrux_rustcrypto_fips203_serialization_byte_identical() {
    let libcrux = libcrux_witness(&KAT_SEED);
    let rustcrypto = rustcrypto_witness(&KAT_SEED);

    assert_eq!(
        libcrux.encapsulation_key, rustcrypto.encapsulation_key,
        "FIPS-203 encapsulation-key serialization MUST be byte-identical across \
         libcrux + RustCrypto (the libcrux swap preserves golden vectors; NQ-C2). \
         would-FAIL while the stub impls diverge."
    );
    assert_eq!(
        libcrux.ciphertext, rustcrypto.ciphertext,
        "FIPS-203 ciphertext serialization MUST be byte-identical across impls"
    );
    assert_eq!(
        libcrux.decapsulation_key, rustcrypto.decapsulation_key,
        "FIPS-203 decapsulation-key serialization MUST be byte-identical across impls"
    );
    assert_eq!(
        libcrux.shared_secret, rustcrypto.shared_secret,
        "FIPS-203 shared-secret MUST be byte-identical across impls"
    );
}

/// F-KAT-1 (b) — ML-KEM-768 serialized sizes are FIPS-203-exact.
///
/// Pins ek=1184, ct=1088, dk=2400, ss=32 (never hardcode-without-pin per
/// CLAUDE.md #5). would-FAIL-if-no-op'd: a witness of any other size.
#[test]
#[ignore = "RED-PHASE: F-KAT-1 — ML-KEM-768 serialized sizes are FIPS-203-exact (1184/1088/2400/32); un-ignore at R5"]
fn ml_kem_768_serialized_sizes_are_fips203_exact() {
    let kat = libcrux_witness(&KAT_SEED);
    assert_eq!(
        kat.encapsulation_key.len(),
        ML_KEM_768_EK_LEN,
        "ML-KEM-768 encapsulation key MUST be 1184 bytes (FIPS-203)"
    );
    assert_eq!(
        kat.ciphertext.len(),
        ML_KEM_768_CT_LEN,
        "ciphertext MUST be 1088 bytes"
    );
    assert_eq!(
        kat.decapsulation_key.len(),
        ML_KEM_768_DK_LEN,
        "decapsulation key MUST be 2400 bytes"
    );
    assert_eq!(
        kat.shared_secret.len(),
        ML_KEM_768_SS_LEN,
        "shared secret MUST be 32 bytes"
    );
}

/// F-KAT-1 (c) — the KAT is genuinely seed-bound (negative control).
///
/// A wrong seed MUST produce a different ciphertext — the witness is not a
/// constant. would-FAIL-if-no-op'd: a constant-returning loader.
#[test]
#[ignore = "RED-PHASE: F-KAT-1 — FIPS-203 witness is seed-bound (wrong seed → different ciphertext); un-ignore at R5"]
fn fips203_witness_is_seed_bound() {
    let kat = libcrux_witness(&KAT_SEED);
    let other = libcrux_witness(&[0x09u8; 32]);
    assert_ne!(
        kat.ciphertext, other.ciphertext,
        "a different KAT seed MUST yield a different ciphertext — the witness is \
         seed-bound, not a constant. would-FAIL on a constant loader."
    );
}
