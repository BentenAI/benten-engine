//! **F-VA-2 — Argon2id DAK derivation (CE-E2).**
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-5 F-VA-2.
//!   - R0 §3.1 Layer-A construction: "KDF: Argon2id v0x13 (RFC 9106; OWASP
//!     params; user-tunable; params persisted alongside salt)"; "Intermediate
//!     HKDF-SHA256 over the Argon2id seed + codepoint label `"benten-dak-v1"` →
//!     the DAK (the HKDF info-tag is the codepoint slot for a future
//!     Argon2id-v2-param-set)".
//!   - R0 §2.2 tactical pick: **DAK KDF = Argon2id v0x13 (RFC 9106; OWASP params
//!     `m_cost=19456, t_cost=2, p_cost=1`; user-tunable)**.
//!   - CLAUDE.md baked-in #5 (never-fork-primitives: Argon2id + HKDF wrap vetted
//!     upstream crates).
//!
//! # What this pins (FREEZE-GATING + FN)
//!
//! `DAK = HKDF-SHA256( Argon2id(pw, salt; m=19456, t=2, p=1), info="benten-dak-v1" )`.
//! The Argon2id params + the HKDF domain-separation info-tag are part of the
//! frozen derivation: a param drift or an info-tag drift changes the DAK,
//! breaking every existing vault. Pins:
//!   1. `derive_dak` is deterministic — same (pw, salt, params) → byte-identical
//!      DAK (KAT-style; a randomized derivation would break vault unlock);
//!   2. a param change (m/t/p) yields a DIFFERENT DAK (params are load-bearing);
//!   3. the HKDF info-tag (`"benten-dak-v1"`) is load-bearing — eliding/changing
//!      it changes the DAK (domain separation; the future-v2-param-set slot).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! At baseline the Argon2id-DAK derivation does not exist (`grep -rn argon2
//! crates/benten-crypto-suite` → ZERO; the DAK is part of the Layer-A vault
//! which R0 §3.1 names a STUB). Per wave-independence this file commits a LOCAL
//! `f_va_2_stub`. The R5 closing wave DELETEs the stub, wires
//! `benten_crypto_suite::vault::derive_dak` (Argon2id wrap + HKDF-SHA256),
//! un-ignores, and verifies green against the real KAT.
//!
//! # Would-FAIL-if-no-op'd (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Each pin drives the production `derive_dak` over a fixed (pw, salt, params)
//! fixture + asserts an observable byte consequence. The stub returns a
//! salt/param/info-tag-INDEPENDENT constant so the determinism pin passes BUT
//! the param-sensitivity and info-tag-sensitivity pins FAIL — exactly the
//! red-phase signal (a real Argon2id mixes pw+salt+params+info, so changing any
//! input changes the output).

#![allow(dead_code)]

// R5: wired to the LIVE vault DAK derivation (Argon2id v0x13 + HKDF-SHA256).
// `derive_dak` returns a zeroize-on-drop `Dak` newtype (F-10); `Dak::expose`
// borrows the raw bytes to compare DAKs across derivations (no `secrecy` dep).
use benten_crypto_suite::vault::{Argon2idParams, DAK_HKDF_INFO_TAG, OWASP_DEFAULT, derive_dak};

fn fixture_password() -> &'static [u8] {
    b"correct horse battery staple"
}

fn fixture_salt() -> [u8; 16] {
    [0x5Au8; 16]
}

/// F-VA-2 (a) — `derive_dak` is deterministic over the same inputs (KAT).
///
/// would-FAIL-if-no-op'd: a randomized DAK derivation would fail every vault
/// unlock; this pin demands the same (pw, salt, params, info) → byte-identical
/// DAK. (The stub passes this arm; the sensitivity arms below are the red-phase
/// failers that force the real Argon2id.)
#[test]
fn derive_dak_is_deterministic() {
    let dak_a = derive_dak(
        fixture_password(),
        &fixture_salt(),
        OWASP_DEFAULT,
        DAK_HKDF_INFO_TAG,
    );
    let dak_b = derive_dak(
        fixture_password(),
        &fixture_salt(),
        OWASP_DEFAULT,
        DAK_HKDF_INFO_TAG,
    );
    assert_eq!(
        dak_a.expose(),
        dak_b.expose(),
        "derive_dak MUST be deterministic for the same (password, salt, \
         params, info-tag) — vault unlock depends on it. would-FAIL on a \
         randomized derivation."
    );
}

/// F-VA-2 (b) — changing the Argon2id params yields a DIFFERENT DAK.
///
/// The params are persisted alongside the salt precisely because they are
/// load-bearing in the derivation. would-FAIL-if-no-op'd: the stub ignores
/// params (returns a constant), so this fails until R5 feeds params into a real
/// Argon2id.
#[test]
fn param_change_yields_different_dak() {
    let dak_default = derive_dak(
        fixture_password(),
        &fixture_salt(),
        OWASP_DEFAULT,
        DAK_HKDF_INFO_TAG,
    );
    let stronger = Argon2idParams {
        m_cost: 65536,
        t_cost: 3,
        p_cost: 1,
    };
    let dak_stronger = derive_dak(
        fixture_password(),
        &fixture_salt(),
        stronger,
        DAK_HKDF_INFO_TAG,
    );
    assert_ne!(
        dak_default.expose(),
        dak_stronger.expose(),
        "changing the Argon2id params (m/t/p) MUST change the DAK — params are \
         persisted-alongside-salt because they are load-bearing. would-FAIL \
         while the stub ignores params and returns a constant."
    );
}

/// F-VA-2 (c) — the HKDF info-tag (`"benten-dak-v1"`) is load-bearing
/// (domain separation; the future-v2-param-set codepoint slot).
///
/// Negative-control: deriving with a DIFFERENT info-tag MUST yield a different
/// DAK. would-FAIL-if-no-op'd: the stub ignores the info-tag.
#[test]
fn hkdf_info_tag_is_load_bearing() {
    let dak_canonical = derive_dak(
        fixture_password(),
        &fixture_salt(),
        OWASP_DEFAULT,
        DAK_HKDF_INFO_TAG,
    );
    let dak_other_tag = derive_dak(
        fixture_password(),
        &fixture_salt(),
        OWASP_DEFAULT,
        b"benten-dak-v2",
    );
    assert_ne!(
        dak_canonical.expose(),
        dak_other_tag.expose(),
        "the HKDF info-tag `benten-dak-v1` MUST be load-bearing — it is the \
         codepoint slot for a future Argon2id-v2 param set (R0 §3.1). \
         Deriving under a different tag MUST yield a different DAK. would-FAIL \
         while the stub ignores the info-tag."
    );
    // Pin the canonical tag's exact bytes so a silent rename is caught.
    assert_eq!(
        DAK_HKDF_INFO_TAG, b"benten-dak-v1",
        "the frozen DAK HKDF info-tag is exactly `benten-dak-v1`"
    );
}
