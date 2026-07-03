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
//! **CI-target scope (R10-council F-04):** the CI gate for this test
//! (`crypto-suite-wasm-roundtrip` in `.github/workflows/wasm-conformance.yml`)
//! runs on **wasm32-wasip1** (via wasmtime), which proves the crypto layer is
//! wasm-CLEAN. BrowserBackend (the thin-compute deployment shape) actually
//! ships on **wasm32-unknown-unknown**; wasm32-wasip1 cleanliness is a strong
//! necessary condition but NOT the same target. A dedicated
//! wasm32-unknown-unknown crypto round-trip drift-gate is a NAMED CI follow-up
//! (see the CI follow-up row in `docs/V1-WIRE-FORMAT-INVENTORY.md`).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! At HEAD `c9c11c56` `AeadKeyMaterial` + `AeadEnvelope` + the
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
//! `#[cfg(target_arch = "wasm32")]` arm asserts a SAME-TARGET
//! self-round-trip proving wasm32-PORTABILITY — a payload sealed on
//! wasm32 opens back to byte-identical plaintext on wasm32. (It does NOT
//! assert cross-target byte-identity of the wrap OUTPUT: each seal mints a
//! FRESH random ephemeral + nonce, so native and wasm32 wire bytes differ
//! by construction — only the recovered PLAINTEXT round-trips identically.)
//! Without this pin, a regression that breaks wasm32 (e.g. an unintended
//! `getrandom`-with-`std` dep, an `Instant::now()` call, a thread-local
//! that wasm32 can't satisfy) could pass cargo-test at native + ship to
//! BrowserBackend silently broken.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! The pin exercises the FULL `wrap_key_material` → `seal_aead` →
//! `open_aead` → `unwrap_key_material` round-trip on a payload + asserts
//! the recovered PLAINTEXT matches byte-identically. The SHIPPED-SURFACE
//! exercise is the `CipherSuiteCodepoint::HYBRID_X25519_MLKEM768` (the
//! `0x647a` codepoint, already typed at HEAD `c9c11c56`). The
//! wasm32 arm asserts a SAME-TARGET self-round-trip (seal-then-open on
//! wasm32 recovers the byte-identical plaintext) — it does NOT assert
//! byte-identity of the wrap OUTPUT between native and wasm32 (fresh random
//! ephemeral + nonce per seal makes the wire bytes differ by construction).
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
//! (TS surface does NOT hardcode classical 32/64 sizes). **Scope-honesty
//! (R9-council F-16):** this test exercises the `benten-crypto-suite`
//! wrap/seal/open/unwrap round-trip COMPILED for the `wasm32` target and
//! asserts a SAME-TARGET self-round-trip of the codepoint-dispatched output
//! (recovered plaintext byte-identity on wasm32; NOT cross-target wire-byte
//! identity — fresh random ephemeral/nonce per seal). It
//! does **NOT** instantiate `BrowserBackend` or the thin-compute-surface stack
//! — it verifies the CRYPTO layer is wasm32-clean (no `std`-only `getrandom` /
//! `Instant::now` / thread-local deps), which is the property BrowserBackend
//! DEPENDS ON, not one this test drives. BrowserBackend end-to-end coverage
//! lives in the browser-runtime test surface, not here.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use benten_crypto_suite::cipher_suite::CipherSuiteCodepoint;
use benten_crypto_suite::error::UnsupportedAlgorithm;

// G-CORE-3a R5: stub module DELETED; real production surface wired.
use benten_crypto_suite::aead::{AeadEnvelope, AeadKeyMaterial};
use benten_crypto_suite::cipher_suite::CipherSuite;

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
fn tf3a_pq_hybrid_wasm32_envelope_round_trip_native() {
    // Native-target arm. The body is identical to the wasm32 arm below;
    // both arms hit the same production code path. The split is
    // entirely a `#[cfg(target_arch)]` guard so that running the test
    // suite on each target individually still asserts the property.
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
    let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
    let k_root = [0xC4u8; 32];
    let wrapped = suite
        .wrap_key_material(kp.public(), &k_root)
        .expect("G-CORE-3a wrap_key_material MUST succeed on hybrid codepoint");
    let recovered_key = suite
        .unwrap_key_material(kp.secret(), &wrapped)
        .expect("G-CORE-3a unwrap_key_material MUST succeed on hybrid codepoint");
    assert_eq!(recovered_key.as_bytes(), &k_root);

    let payload = b"PQ-hybrid envelope round-trip payload (Gate 2, native arm)";
    let plaintext_cid = b"plaintext-cid-native";
    let envelope = suite
        .seal_aead(&k_root, payload, plaintext_cid)
        .expect("G-CORE-3a seal_aead MUST succeed on hybrid codepoint");

    let recovered = suite
        .open_aead(&k_root, &envelope, plaintext_cid)
        .expect("G-CORE-3a open_aead MUST succeed on hybrid codepoint (native target)");
    assert_eq!(
        recovered.as_slice(),
        payload,
        "Gate 2 native arm: PQ-hybrid envelope round-trip MUST yield \
         byte-identical plaintext (would-FAIL if AEAD nonce-handling \
         or KEM unwrap is broken)"
    );

    // Witness the envelope shape — Gate-6 format-version discriminator.
    let wire = envelope.to_wire_bytes();
    assert!(wire.len() >= 5);
    let _ = AeadEnvelope::from_wire_bytes(&wire).expect("envelope parses back from wire bytes");

    // Witness the AeadKeyMaterial type compiles + is reachable from the
    // BrowserBackend wasm32 deployment shape (CLAUDE.md baked-in #17).
    let _km =
        AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, &k_root);
}

#[test]
#[cfg(target_arch = "wasm32")]
fn tf3a_pq_hybrid_wasm32_envelope_round_trip_wasm() {
    // wasm32 arm — the BrowserBackend / thin-compute-surface deployment
    // shape per CLAUDE.md baked-in #17. The body MIRRORS the native
    // arm; the load-bearing assertion is that the SAME production
    // primitives wrap/seal/open work in the absence of `std::time`,
    // `std::thread`, `getrandom`-with-std, etc.
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
    let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
    let k_root = [0xC4u8; 32];
    let wrapped = suite
        .wrap_key_material(kp.public(), &k_root)
        .expect("G-CORE-3a wrap_key_material MUST succeed on hybrid codepoint (wasm32)");
    let _recovered_key = suite
        .unwrap_key_material(kp.secret(), &wrapped)
        .expect("G-CORE-3a unwrap_key_material MUST succeed (wasm32)");

    let payload = b"PQ-hybrid envelope round-trip payload (Gate 2, wasm32 arm)";
    let plaintext_cid = b"plaintext-cid-wasm32";
    let envelope = suite
        .seal_aead(&k_root, payload, plaintext_cid)
        .expect("G-CORE-3a seal_aead MUST succeed on hybrid codepoint (wasm32)");

    let recovered = suite
        .open_aead(&k_root, &envelope, plaintext_cid)
        .expect("G-CORE-3a open_aead MUST succeed on hybrid codepoint (wasm32 target — Gate 2 the load-bearing cross-target assertion)");
    assert_eq!(
        recovered.as_slice(),
        payload,
        "Gate 2 wasm32 arm: PQ-hybrid envelope round-trip MUST yield \
         byte-identical plaintext on wasm32 target (would-FAIL if any \
         production primitive secretly depends on a non-wasm32-portable \
         std API; BrowserBackend ships this binary)"
    );

    let _km =
        AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, &k_root);
}
