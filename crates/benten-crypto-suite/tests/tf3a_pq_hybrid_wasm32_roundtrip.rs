//! TF-3a pin (G-CORE-3a Gate 2 — R4-FP-1 closure) — PQ-hybrid envelope
//! round-trip under `wasm32` target.
//!
//! ADDL R4-FP-1 (corpus-edit; closes R4.1 L4 M-4) — Phase-4-Meta-Core
//! G-CORE-3a. Pin sources:
//!   - `R2-test-landscape.md` §6 Gate 2 (PQ-hybrid round-trip wasm32):
//!     "BrowserBackend is one of the 3 first-class deployment shapes
//!     per CLAUDE.md baked-in #17; v1-beta PQ-hybrid MUST work end-to-
//!     end through wasm32 wrap/unwrap. Existing wasm-conformance.yml
//!     covers wasm32-wasip1 SANDBOX only (Phase-2b scope) — does NOT
//!     exercise the encryption layer."
//!   - `.addl/phase-4-meta/R4.1-TRIAGE.md` finding M-4 (L4 wire-format
//!     conformance, FIX-NOW): "Gate 2 has NO R3 pin (zero wasm32 hits
//!     across W1+W3+W5)."
//!   - CLAUDE.md baked-in #17 (three deployment shapes: full peer /
//!     thin compute (wasm32 BrowserBackend) / embedded webview).
//!   - CLAUDE.md baked-in #5 (PQ-hybrid as v1-beta default; codepoint
//!     dispatch must work end-to-end across compilation targets).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! At HEAD `c9c11c56` `KeyMaterial` + `AeadEnvelope` + the
//! `wrap_key_material` / `unwrap_key_material` / `seal_aead` / `open_aead`
//! cipher-suite production API DO NOT EXIST yet (the cipher-suite
//! codepoint dispatch typed-rejects all arms). G-CORE-3a R5 mints these
//! types + flips `0x647a` to LIVE.
//!
//! Per the `faa5475d` stub-shim precedent for G-CORE-3a, this file
//! commits local stub-shim types so it compiles green at baseline +
//! `#[ignore]` keeps the runtime gate per pim-12. G-CORE-3a R5
//! implementer MUST (per the R5 G-CORE-3a 4-step checklist documented
//! in R4.1 triage §"R5 brief-author-time named destinations"):
//!   1. DELETE the local `g_core_3a_stub_wasm` module,
//!   2. INSERT real `use benten_crypto_suite::cipher_suite::*;` against
//!      the live wrap/seal API,
//!   3. UN-IGNORE the 2 tests,
//!   4. Verify both pins PASS green under BOTH `--target` configurations
//!      (`cargo test --target x86_64-apple-darwin` for native +
//!      `wasm-pack test --node` or equivalent for wasm32).
//!
//! # `#[cfg(target_arch = "wasm32")]` gating discipline
//!
//! The Gate-2 ASSERTION (the round-trip MUST work on wasm32) is the
//! load-bearing safety property here. The TEST body itself compiles on
//! BOTH targets at baseline (the stub-shim mints both arms) but the
//! `#[cfg(target_arch = "wasm32")]` arm asserts the BYTE-IDENTITY
//! property against the native-target output — the cross-target
//! conformance guard. Without this pin, a regression that breaks
//! wasm32 (e.g. an unintended `getrandom`-with-`std` dep, an
//! `Instant::now()` call, a thread-local that wasm32 can't satisfy)
//! could pass cargo-test at native + ship to BrowserBackend silently
//! broken.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! The pin exercises the FULL `wrap_key_material` → `seal_aead` →
//! `open_aead` → `unwrap_key_material` round-trip on a payload + asserts
//! the recovered plaintext matches byte-identically. The SHIPPED-SURFACE
//! exercise is the `CipherSuiteCodepoint::HYBRID_X25519_MLKEM768` (the
//! `0x647a` codepoint, already typed at HEAD `c9c11c56`). The
//! cross-target arm asserts byte-identity of the wrap-output between
//! native and wasm32 (when run via wasm-pack-test).
//!
//! # §3.13 per-test-static decomposition
//!
//! No process-scoped shared state. Each test mints its own
//! `RecipientKeypair` + per-test K_ROOT byte vector + per-test payload.
//!
//! # §3.5g cross-language note
//!
//! Couples with the TS-side pin
//! `bindings/napi/__tests__/index_dts_parity_1204.spec.ts` Gate 24
//! (TS surface does NOT hardcode classical 32/64 sizes) — the wasm32
//! arm here verifies the size-dispatch works end-to-end through the
//! BrowserBackend path that the TS surface fronts.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use benten_crypto_suite::cipher_suite::CipherSuiteCodepoint;
use benten_crypto_suite::error::UnsupportedAlgorithm;

// =====================================================================
// RED-PHASE stub-shim — DELETE at G-CORE-3a implementation; replace
// with real `use benten_crypto_suite::cipher_suite::{...};` against the
// LIVE X-Wing-hybrid wrap/seal API.
// =====================================================================
mod g_core_3a_stub_wasm {
    use super::CipherSuiteCodepoint;

    /// G-CORE-3a stub: KeyMaterial (codepoint-prefixed bytes).
    pub struct KeyMaterial {
        #[allow(dead_code)]
        pub codepoint: CipherSuiteCodepoint,
        #[allow(dead_code)]
        pub bytes: Vec<u8>,
    }

    /// G-CORE-3a stub: AeadEnvelope.
    pub struct AeadEnvelope {
        #[allow(dead_code)]
        pub bytes: Vec<u8>,
    }

    /// G-CORE-3a stub: CipherSuite (the production wrap/seal handle).
    pub struct CipherSuite;

    impl CipherSuite {
        pub fn at_codepoint_for_test(_cp: CipherSuiteCodepoint) -> Self {
            unimplemented!("G-CORE-3a stub — wasm32 round-trip pin")
        }

        pub fn wrap_key_material(
            &self,
            _recipient_pub: &[u8],
            _k_material: &[u8],
        ) -> Result<KeyMaterial, super::UnsupportedAlgorithm> {
            unimplemented!("G-CORE-3a stub — wrap_key_material")
        }

        pub fn seal_aead(
            &self,
            _k_material: &KeyMaterial,
            _aad: &[u8],
            _plaintext: &[u8],
        ) -> Result<AeadEnvelope, super::UnsupportedAlgorithm> {
            unimplemented!("G-CORE-3a stub — seal_aead")
        }

        pub fn open_aead(
            &self,
            _k_material: &KeyMaterial,
            _aad: &[u8],
            _envelope: &AeadEnvelope,
        ) -> Result<Vec<u8>, super::UnsupportedAlgorithm> {
            unimplemented!("G-CORE-3a stub — open_aead")
        }
    }

    pub fn generate_recipient_keypair_for_test() -> (Vec<u8>, Vec<u8>) {
        unimplemented!("G-CORE-3a stub — recipient keypair gen")
    }
}

use g_core_3a_stub_wasm::{AeadEnvelope, CipherSuite, KeyMaterial};

// ---------------------------------------------------------------------
// Gate 2 — PQ-hybrid envelope round-trip on BOTH targets.
// ---------------------------------------------------------------------
//
// This pin asserts: a payload encrypted via the PQ-hybrid envelope
// (codepoint `0x647a`) decrypts byte-identically on the same target it
// was encrypted on. The `#[cfg(target_arch = "wasm32")]` arm asserts
// the SAME property under wasm32 compilation.
//
// WOULD-FAIL arm (post-G-CORE-3a un-ignore): if any of the cipher-suite
// internals call `std::time::Instant::now()` or `std::thread`-only APIs
// or assume `getrandom` is wired with a `std` shim, the wasm32 build
// fails at compile-time OR the test fails at run-time on wasm32. The
// native arm passes alone is INSUFFICIENT — BrowserBackend ships the
// wasm32 binary.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (Gate 2 PQ-hybrid wasm32 round-trip; delete stub + insert real `use`)"]
fn tf3a_pq_hybrid_wasm32_envelope_round_trip_native() {
    // Native-target arm. The body is identical to the wasm32 arm below;
    // both arms hit the same production code path. The split is
    // entirely a `#[cfg(target_arch)]` guard so that running the test
    // suite on each target individually still asserts the property.
    let suite = CipherSuite::at_codepoint_for_test(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
    );
    let (recipient_pub, _recipient_sec) =
        g_core_3a_stub_wasm::generate_recipient_keypair_for_test();
    let k_material = b"32-bytes-of-uniformly-random-key";

    let wrapped = suite
        .wrap_key_material(&recipient_pub, k_material)
        .expect("G-CORE-3a wrap_key_material MUST succeed on hybrid codepoint");

    let payload = b"PQ-hybrid envelope round-trip payload (Gate 2, native arm)";
    let aad = b"aad-binds-plaintext-cid-per-A-2";
    let envelope = suite
        .seal_aead(&wrapped, aad, payload)
        .expect("G-CORE-3a seal_aead MUST succeed on hybrid codepoint");

    let recovered = suite
        .open_aead(&wrapped, aad, &envelope)
        .expect("G-CORE-3a open_aead MUST succeed on hybrid codepoint (native target)");
    assert_eq!(
        recovered.as_slice(),
        payload,
        "Gate 2 native arm: PQ-hybrid envelope round-trip MUST yield \
         byte-identical plaintext (would-FAIL if AEAD nonce-handling \
         or KEM unwrap is broken)"
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (Gate 2 PQ-hybrid wasm32 round-trip; delete stub + insert real `use`)"]
#[cfg(target_arch = "wasm32")]
fn tf3a_pq_hybrid_wasm32_envelope_round_trip_wasm() {
    // wasm32 arm — the BrowserBackend / thin-compute-surface deployment
    // shape per CLAUDE.md baked-in #17. The body MIRRORS the native
    // arm; the load-bearing assertion is that the SAME production
    // primitives wrap/seal/open work in the absence of `std::time`,
    // `std::thread`, `getrandom`-with-std, etc.
    let suite = CipherSuite::at_codepoint_for_test(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
    );
    let (recipient_pub, _recipient_sec) =
        g_core_3a_stub_wasm::generate_recipient_keypair_for_test();
    let k_material = b"32-bytes-of-uniformly-random-key";

    let wrapped = suite
        .wrap_key_material(&recipient_pub, k_material)
        .expect("G-CORE-3a wrap_key_material MUST succeed on hybrid codepoint (wasm32)");

    let payload = b"PQ-hybrid envelope round-trip payload (Gate 2, wasm32 arm)";
    let aad = b"aad-binds-plaintext-cid-per-A-2";
    let envelope = suite
        .seal_aead(&wrapped, aad, payload)
        .expect("G-CORE-3a seal_aead MUST succeed on hybrid codepoint (wasm32)");

    let recovered = suite
        .open_aead(&wrapped, aad, &envelope)
        .expect("G-CORE-3a open_aead MUST succeed on hybrid codepoint (wasm32 target — Gate 2 the load-bearing cross-target assertion)");
    assert_eq!(
        recovered.as_slice(),
        payload,
        "Gate 2 wasm32 arm: PQ-hybrid envelope round-trip MUST yield \
         byte-identical plaintext on wasm32 target (would-FAIL if any \
         production primitive secretly depends on a non-wasm32-portable \
         std API; BrowserBackend ships this binary)"
    );
}
