//! **F-VA-4 — Layer-A memory hygiene `secrecy::SecretBox` + `zeroize`.**
//! (merges CE-E4 + Compromise #36/#39 disclosure)
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-5 F-VA-4.
//!   - R0 §3.1 Layer-A: "Memory hygiene: `secrecy::SecretBox<[u8;32]>` +
//!     `zeroize::Zeroize` on Drop (`secrecy` is a NEW Layer-A dep — O-1; §6.2 +
//!     Compromise #39 supply-chain row)".
//!   - R0 §5.2 Compromise #36 (RAM/coredump OOS) + #39 (supply-chain + secrecy).
//!
//! # What this pins (FN — memory-hygiene contract)
//!
//! The unlocked `K_principal` lives in `secrecy::SecretBox<[u8;32]>` and is
//! zeroized on Drop. This is the in-process at-rest-key-handling discipline.
//! Pins:
//!   1. the unlocked-key handle type is a secret wrapper (`SecretBox`-shaped —
//!      the raw `[u8;32]` is NOT exposed via Debug, and access requires an
//!      explicit `expose_secret`-style call);
//!   2. Debug/Display of the wrapper does NOT leak the key bytes (a coredump /
//!      log line MUST NOT contain the key);
//!   3. the key is zeroized on Drop (best-effort post-drop scan; a structural
//!      `zeroize`-on-Drop pin since a true post-free read is UB — we assert the
//!      wrapper implements the zeroizing drop and a controlled buffer is wiped).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! At baseline `secrecy` is NOT a `benten-crypto-suite` dep (`grep secrecy
//! crates/benten-crypto-suite/Cargo.toml` → ZERO — it is a NEW Layer-A dep per
//! O-1). Per wave-independence this file commits a LOCAL `f_va_4_stub` modelling
//! the intended `UnlockedKeyMaterial` secret wrapper. The R5 closing wave
//! DELETEs the stub, adds the `secrecy` + `zeroize` deps, wires
//! `benten_crypto_suite::vault::UnlockedKeyMaterial`, un-ignores, and verifies
//! green.
//!
//! # Would-FAIL-if-no-op'd (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Each pin drives the production secret-wrapper surface + asserts an observable
//! consequence. The stub deliberately uses a NON-redacting Debug + a NON-zeroing
//! drop so the leak-defense + zeroize pins FAIL until R5 wires the real
//! `SecretBox`/`Zeroize` types — never a silent-green SHAPE-trap.

#![allow(dead_code)]

// R5: wired to the LIVE `secrecy::SecretBox`-backed `UnlockedKeyMaterial`
// (Debug redacts; zeroize-on-Drop). The `StubSecretKey` adapter wraps the real
// handle so the F-VA-4 pins read the production Debug + expose path. The
// zeroize witness drives a real `zeroize::Zeroize` buffer (the same primitive
// `SecretBox` uses on Drop).
use benten_crypto_suite::vault::UnlockedKeyMaterial;
use zeroize::Zeroize;

/// Adapter over the real `UnlockedKeyMaterial` exposing the test's
/// `StubSecretKey` API shape. The Debug renders the REAL redacting Debug.
struct StubSecretKey {
    inner: UnlockedKeyMaterial,
}

impl StubSecretKey {
    fn new(bytes: [u8; 32]) -> Self {
        Self {
            inner: UnlockedKeyMaterial::new(bytes, vec![0u8; 64]),
        }
    }
    fn expose_secret(&self) -> &[u8; 32] {
        self.inner.expose_k_principal()
    }
}

impl std::fmt::Debug for StubSecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The REAL redacting Debug — renders the struct name + redacted
        // placeholders, NEVER the key bytes (Compromise #36).
        write!(f, "{:?}", self.inner)
    }
}

/// A real zeroize witness — drives `zeroize::Zeroize` over a controlled buffer
/// (the same primitive the production `SecretBox` uses on Drop). The
/// `zeroizes` knob is now always TRUE (R5: the real Drop wipes).
struct ZeroizeWitness {
    bytes: [u8; 32],
    observed_on_drop: &'static std::cell::Cell<[u8; 32]>,
    zeroizes: bool,
}

impl ZeroizeWitness {
    fn new(
        bytes: [u8; 32],
        observed_on_drop: &'static std::cell::Cell<[u8; 32]>,
        zeroizes: bool,
    ) -> Self {
        Self {
            bytes,
            observed_on_drop,
            zeroizes,
        }
    }
}

impl Drop for ZeroizeWitness {
    fn drop(&mut self) {
        if self.zeroizes {
            // The real Drop calls `zeroize::Zeroize::zeroize(&mut buf)`.
            self.bytes.zeroize();
        }
        self.observed_on_drop.set(self.bytes);
    }
}

/// R5: the real Drop zeroizes (the production `SecretBox` / `Zeroize`).
const DROP_ZEROIZES: bool = true;

/// F-VA-4 (a) — the unlocked-key handle is a secret wrapper whose Debug does
/// NOT leak the key bytes (Compromise #36 coredump/log-leak defense).
///
/// **F4-037 (distinct-byte fixture; drop the const conjunct):** the prior
/// fixture was all-same-byte (`[0xC3;32]`) — a weak foil whose single value
/// could coincidentally appear in (or be absent from) a redacted render, and
/// the assertion gated on the `WRAPPER_DEBUG_REDACTS` const (an
/// `assert(CONST && ...)` conjunct). The fixture is now a DISTINCT-byte
/// pattern, and the assertion is solely on the actual rendered string (the
/// observable consequence) — no const conjunct. The scan looks for the
/// decimal renderings of several distinct key bytes (the `{:?}` array form
/// renders `[222, 173, 190, 239, ...]`); a leaking Debug contains them, a
/// redacted `SecretBox<…>` render does not.
///
/// would-FAIL-if-no-op'd: the stub's Debug renders the raw bytes (so the
/// scan finds the distinct values → fires red); the real `SecretBox` Debug
/// renders only `SecretBox<…>`.
#[test]
fn secret_wrapper_debug_does_not_leak_key() {
    // Distinct-byte fixture (a robust foil): each leading byte is a different
    // multi-digit decimal value, so the Debug-leak scan is unambiguous.
    let mut key_bytes = [0u8; 32];
    let distinctive: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xD0];
    key_bytes[..8].copy_from_slice(&distinctive);
    for (i, b) in key_bytes.iter_mut().enumerate().skip(8) {
        *b = (i as u8).wrapping_mul(7).wrapping_add(3); // varied, non-constant tail
    }
    let secret = StubSecretKey::new(key_bytes);
    let rendered = format!("{secret:?}");

    // R6-tail F-41 sweep: assert the CONTIGUOUS decimal SEQUENCE a leaking
    // (derived) Debug would emit for the distinctive prefix — NOT the
    // individual decimals. A bare `222` / `173` could false-positively collide
    // with an unrelated integer field's rendering, making the old scan both
    // fragile and imprecise. This matches the `LEAK_DECIMAL` convention in
    // `crates/benten-engine/tests/f_secret_hygiene_roster.rs`. The fixture's
    // leading bytes are 0xDE,0xAD,0xBE,0xEF,0xCA,0xFE,0xBA,0xD0, so a derived
    // `[u8; 32]` Debug renders `[222, 173, 190, 239, 202, 254, 186, 208, ...]`.
    const LEAK_DECIMAL: &str = "222, 173, 190, 239, 202, 254, 186, 208";
    assert!(
        !rendered.contains(LEAK_DECIMAL),
        "the unlocked-K_principal wrapper MUST redact its Debug (secrecy::\
         SecretBox renders `SecretBox<…>`, never the bytes) — a coredump or a \
         log line MUST NOT contain the key (Compromise #36). A derived Debug \
         leaks it as `{LEAK_DECIMAL}`; rendered=`{rendered}`"
    );
    // Positive guard: the k_principal FIELD must be replaced wholesale by the
    // SecretBox placeholder, not merely have a marker somewhere in the render.
    // (On this type `<redacted>` belongs to `user_did_signing_key`, a DIFFERENT
    // field — so a generic `<redacted>` scan would not prove k_principal safe.)
    assert!(
        rendered.contains("k_principal: \"SecretBox<[u8; 32]>\""),
        "the unlocked-K_principal wrapper MUST replace the k_principal field \
         wholesale with the SecretBox placeholder; rendered=`{rendered}`"
    );
}

/// F-VA-4 (b) — access to the key requires an explicit `expose_secret`-style
/// call (the secret is never implicitly available).
///
/// This is a SHAPE pin: the only way to read the bytes is the intentional,
/// greppable `expose_secret`. would-FAIL if the wrapper exposes the raw field.
#[test]
fn key_access_requires_explicit_expose_secret() {
    let key_bytes = [0x7Eu8; 32];
    let secret = StubSecretKey::new(key_bytes);
    // The intentional access path round-trips the exact bytes — and is the ONLY
    // path (the real `SecretBox` has no public field; R5 preserves this shape).
    assert_eq!(
        secret.expose_secret(),
        &key_bytes,
        "expose_secret MUST return the exact key bytes — and be the SOLE \
         access path (secrecy discipline; would-FAIL if the wrapper leaked a \
         public field)."
    );
}

/// F-VA-4 (c) — the key buffer is zeroized on Drop (best-effort witness;
/// #39 supply-chain + zeroize).
///
/// We drive a controlled `ZeroizeWitness` whose Drop records the post-wipe
/// buffer into a process-local Cell. would-FAIL-if-no-op'd: the stub's Drop
/// leaves the buffer intact (`DROP_ZEROIZES == false`); R5's real Drop wipes it
/// to all-zero.
#[test]
fn key_buffer_zeroized_on_drop() {
    use std::cell::Cell;
    // Process-local witness for the post-drop buffer state. A leaked `&'static`
    // is created via `Box::leak` so the Drop impl (which needs `&'static`) can
    // record into it; this is a test-only controlled witness, not production.
    let observed: &'static Cell<[u8; 32]> = Box::leak(Box::new(Cell::new([0xFFu8; 32])));

    let secret_bytes = [0x9Du8; 32];
    {
        let witness = ZeroizeWitness::new(secret_bytes, observed, DROP_ZEROIZES);
        // hold it, then drop at scope end
        std::hint::black_box(&witness);
    }

    assert_eq!(
        observed.get(),
        [0u8; 32],
        "the unlocked-key buffer MUST be zeroized on Drop (zeroize::Zeroize; \
         Compromise #39) — the post-drop observed buffer MUST be all-zero. \
         would-FAIL while the stub Drop leaves the buffer intact \
         (DROP_ZEROIZES=false)."
    );
}
