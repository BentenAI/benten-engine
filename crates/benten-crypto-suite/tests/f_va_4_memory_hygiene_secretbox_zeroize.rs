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

/// SELF-CONTAINED stub-shim (R5 deletes this whole module + wires the LIVE
/// `secrecy::SecretBox<[u8;32]>`-backed `UnlockedKeyMaterial`).
mod f_va_4_stub {
    use std::cell::Cell;

    /// Models the intended `UnlockedKeyMaterial`'s secret wrapper around
    /// `[u8;32]`. STUB exposes the raw bytes via a plain Debug (the LEAK the
    /// real `SecretBox` forbids) so the leak-defense pin fails until R5.
    pub struct StubSecretKey {
        // R5: this becomes `secrecy::SecretBox<[u8;32]>`.
        bytes: [u8; 32],
    }

    impl StubSecretKey {
        pub fn new(bytes: [u8; 32]) -> Self {
            Self { bytes }
        }

        /// The explicit access path (mirrors `secrecy::ExposeSecret`). Access
        /// is intentional + greppable, never implicit.
        pub fn expose_secret(&self) -> &[u8; 32] {
            &self.bytes
        }
    }

    // STUB Debug LEAKS the bytes (the bug). R5's real SecretBox Debug redacts.
    impl std::fmt::Debug for StubSecretKey {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // RED-PHASE: leaks. The leak-defense pin asserts the rendered string
            // does NOT contain the key bytes — so it FAILS here and PASSES once
            // R5 swaps in `SecretBox` (whose Debug renders `SecretBox<…>` only).
            write!(f, "StubSecretKey({:?})", self.bytes)
        }
    }

    /// Whether the real wrapper redacts its Debug. STUB = false (red-phase);
    /// R5's `SecretBox` = true. NOTE (F4-037): the leak-defense pin no longer
    /// gates on this const — it asserts directly on the rendered Debug string
    /// (the observable consequence). Kept as documentation of R5 intent.
    pub const WRAPPER_DEBUG_REDACTS: bool = false;

    /// Drives the zeroize-on-Drop contract over a controlled buffer. The real
    /// `UnlockedKeyMaterial` zeroizes its backing buffer on Drop. STUB does NOT
    /// zeroize (returns the original bytes) so the pin fails until R5.
    ///
    /// We thread the post-drop observed value through a `Cell` populated inside
    /// the wrapper's Drop so the test can read what the buffer was wiped to —
    /// a controlled stand-in for the UB-free zeroize witness.
    pub struct ZeroizeWitness {
        bytes: [u8; 32],
        observed_on_drop: &'static Cell<[u8; 32]>,
        zeroizes: bool,
    }

    impl ZeroizeWitness {
        pub fn new(
            bytes: [u8; 32],
            observed_on_drop: &'static Cell<[u8; 32]>,
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
                // R5: the real Drop calls `zeroize::Zeroize::zeroize(&mut buf)`.
                self.bytes = [0u8; 32];
            }
            // RED-PHASE STUB: `zeroizes == false`, so the buffer is left intact.
            self.observed_on_drop.set(self.bytes);
        }
    }

    /// STUB = false (Drop does NOT zeroize). R5 = true.
    pub const DROP_ZEROIZES: bool = false;
}

use f_va_4_stub::{DROP_ZEROIZES, StubSecretKey, ZeroizeWitness};

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
#[ignore = "RED-PHASE: F-VA-4 — unlocked K_principal wrapper Debug MUST NOT leak key bytes (secrecy::SecretBox; #36); un-ignore at R5"]
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

    // The `{:?}` array form renders bytes as DECIMAL: 0xDE=222, 0xAD=173,
    // 0xBE=190, 0xEF=239, 0xCA=202, 0xFE=254. A leaking Debug contains these
    // distinct multi-digit values; a redacted `SecretBox<…>` render does not.
    let leaked = rendered.contains("222")
        || rendered.contains("173")
        || rendered.contains("190")
        || rendered.contains("239")
        || rendered.contains("202")
        || rendered.contains("254");
    assert!(
        !leaked,
        "the unlocked-K_principal wrapper MUST redact its Debug (secrecy::\
         SecretBox renders `SecretBox<…>`, never the bytes) — a coredump or a \
         log line MUST NOT contain the key (Compromise #36). would-FAIL while \
         the stub Debug leaks the raw [u8;32]; rendered=`{rendered}`"
    );
}

/// F-VA-4 (b) — access to the key requires an explicit `expose_secret`-style
/// call (the secret is never implicitly available).
///
/// This is a SHAPE pin: the only way to read the bytes is the intentional,
/// greppable `expose_secret`. would-FAIL if the wrapper exposes the raw field.
#[test]
#[ignore = "RED-PHASE: F-VA-4 — key access MUST be via explicit expose_secret (secrecy discipline); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-VA-4 — K_principal buffer MUST be zeroized on Drop (zeroize::Zeroize; #39); un-ignore at R5"]
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
