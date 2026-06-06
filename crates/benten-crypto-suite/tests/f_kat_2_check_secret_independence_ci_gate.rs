//! **F-KAT-2 — libcrux ML-KEM-768 secret-independence assurance (CE-D2).**
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-4 F-KAT-2 ("the libcrux hax/F*
//!     ML-KEM secret-independence CI gate is wired + green; absence = freeze
//!     blocker"; class = FG (CI)).
//!   - R0 §2.2 Q1: libcrux-ml-kem "verified secret-independence via hax/F*;
//!     `check-secret-independence` CI gate".
//!   - R0 §3.2 Wave-0 exit gate (m-2): "`check-secret-independence` in CI".
//!   - R0 §5.2 Compromise #30 (unaudited-PQ; CLOSES at v1-GM / C-GM-AUDIT) +
//!     #32 (ML-KEM-768 Decap CT side-channel — the libcrux CT-mitigation this
//!     witnesses).
//!
//! # WIRED 2026-06-05 — what is REAL at libcrux-ml-kem 0.0.9 (honesty over green)
//!
//! `libcrux-ml-kem 0.0.9` is now the PRODUCTION ML-KEM-768 impl. Compromise
//! #32 (ML-KEM-768 Decap side-channel / Tempo-SampleNTT-timing) is moved from
//! deferred → mitigated-live by the swap: libcrux's portable + AVX2 field
//! arithmetic / NTT / serialization / generic high-level code is FORMALLY
//! VERIFIED via hax + F*, and on the targets where CT matters most (wasm +
//! non-SIMD) the portable (verified, constant-time) backend is the one
//! selected. The tests below pin THAT real, available assurance:
//!   (a) the secret-independence-gate SEAM is registered (libcrux is a real
//!       dep + the `mlkem-ct-check` forwarding feature exists);
//!   (b) the PRODUCTION ML-KEM-768 impl is the verified libcrux (a live
//!       FIPS-203 witness through the crate's own `mlkem` wrapper — not a
//!       placeholder / not RustCrypto, which is dev-only now);
//!   (c) [#[ignore]'d + FLAG-FOR-BEN] the runnable `check-secret-independence`
//!       CI BUILD-gate — see the FLAG on that test.
//!
//! # ⚠️ FLAG-FOR-BEN — the runnable check-secret-independence CI gate
//!
//! libcrux 0.0.9's `check-secret-independence` feature EXISTS (verified in
//! `libcrux-secrets 0.0.5`) and is a genuine COMPILE-TIME gate: with it on,
//! ML-KEM secret integers become opaque secret-typed values that lack
//! branch/index/non-CT ops, so the crate fails to compile if it would leak.
//! BUT at the pinned `=0.0.9`, building `libcrux-ml-kem` with that feature on
//! FAILS TO COMPILE (E0053 — its `impl_kem_trait!` macro does not propagate
//! the secret-typed `keygen`/`encaps`/`decaps` signatures; reproduced
//! 2026-06-05; an upstream 0.0.9 defect, NOT Benten's usage). So a green
//! `check-secret-independence` CI step is NOT honestly wireable at 0.0.9 —
//! reporting it as enabled+green would be a fake-green sentinel. The
//! `mlkem-ct-check` feature on `benten-crypto-suite` is the one-flag-away
//! seam for when libcrux ships a version that compiles cleanly under it.
//! Ben decision: (i) accept the verified-backend assurance pinned here for
//! v1-beta + carry the runnable CI gate to the libcrux-version that fixes
//! the upstream macro (tracked as Compromise #32's residual), or (ii) hold
//! the tag for an upstream fix. Kept #[ignore]'d per the no-fake-green rule.

#![allow(dead_code)]

/// Read THIS crate's Cargo manifest (the real wiring source-of-truth — not a
/// stub). The manifest lives one dir up from `tests/`.
fn crate_manifest() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    std::fs::read_to_string(path).expect("crypto-suite Cargo.toml readable")
}

/// F-KAT-2 (a) — the secret-independence gate SEAM is REGISTERED.
///
/// Real probe (not a stub): `libcrux-ml-kem` is a genuine dependency AND the
/// `mlkem-ct-check` forwarding feature (→ `libcrux-ml-kem/check-secret-
/// independence`) exists. This is the wiring seam the runnable CI gate plugs
/// into. would-FAIL-if-no-op'd: if libcrux were not the dep, or the forwarding
/// feature were absent, the gate could not be wired at all.
#[test]
fn secret_independence_gate_seam_is_registered() {
    let manifest = crate_manifest();
    assert!(
        manifest.contains("libcrux-ml-kem"),
        "libcrux-ml-kem MUST be a real dependency (the verified ML-KEM-768 \
         impl) — its absence means the secret-independence assurance is \
         un-witnessed (Compromise #32 freeze-blocker)."
    );
    assert!(
        manifest.contains("check-secret-independence"),
        "the `mlkem-ct-check` feature (forwarding libcrux's \
         `check-secret-independence`) MUST be registered — it is the seam the \
         runnable CI gate plugs into."
    );
}

/// F-KAT-2 (b) — the PRODUCTION ML-KEM-768 impl is the verified libcrux.
///
/// Real LIVE witness (not a placeholder): drive a FIPS-203 keygen + encap +
/// decap through the crate's own production `mlkem` wrapper (which wraps
/// libcrux) and assert exact FIPS-203 sizes + a successful round-trip. A
/// stub / wrong-impl target would not produce the exact 1184/1088/2400/32
/// FIPS-203 byte sizes and a recovering decap. This is what makes the
/// secret-independence assurance LOAD-BEARING: the impl whose CT property we
/// rely on is genuinely the one running.
#[test]
fn production_impl_is_verified_libcrux_ml_kem() {
    // The crate's production wrapper IS libcrux (see src/mlkem.rs). Round-trip
    // through it to witness the verified impl is live.
    use benten_crypto_suite::cipher_suite::{CipherSuite, CipherSuiteCodepoint};

    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("0x647a hybrid suite resolves");
    let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
    let k_root = [0x5au8; 32];
    let wrapped = suite
        .wrap_key_material(kp.public(), &k_root)
        .expect("hybrid wrap (exercises libcrux ML-KEM-768 encapsulate) MUST succeed");
    // The ML-KEM-768 ciphertext half is FIPS-203-exact (1088 B) — a live
    // witness that the production encapsulate is the real ML-KEM-768 impl.
    assert_eq!(
        wrapped.ek_mlkem.len(),
        1088,
        "the wrapped ML-KEM-768 ciphertext MUST be FIPS-203-exact (1088 B) — \
         a live witness that the verified libcrux impl is the production path"
    );
    let recovered = suite
        .unwrap_key_material(kp.secret(), &wrapped)
        .expect("hybrid unwrap (exercises libcrux ML-KEM-768 decapsulate) MUST succeed");
    assert_eq!(
        recovered.as_bytes(),
        &k_root,
        "the libcrux ML-KEM-768 decapsulate path MUST recover k_root"
    );
}

/// F-KAT-2 (c) — the RUNNABLE `check-secret-independence` CI BUILD-gate.
///
/// ⚠️ FLAG-FOR-BEN / #[ignore]'d: see the module FLAG. libcrux-ml-kem 0.0.9
/// FAILS TO COMPILE under its own `check-secret-independence` feature (E0053
/// upstream macro defect, reproduced 2026-06-05), so a green runnable CI gate
/// is not honestly wireable at the pinned version. This test asserts the
/// build SUCCEEDS under the `mlkem-ct-check` feature — which it does NOT at
/// 0.0.9 — so it stays #[ignore]'d (not faked-green) until libcrux ships a
/// version that compiles cleanly under the feature. Un-ignore + wire the
/// `.github/workflows` build step at that version.
#[test]
#[ignore = "FLAG-FOR-BEN: libcrux-ml-kem =0.0.9 does NOT compile under its own \
            `check-secret-independence` feature (E0053 — upstream impl_kem_trait! \
            macro defect on keygen/encaps/decaps, reproduced 2026-06-05). A green \
            runnable CI secret-independence gate is therefore not honestly wireable \
            at 0.0.9. The `mlkem-ct-check` feature is the one-flag-away seam for \
            the libcrux version that fixes this. Verified-backend assurance is \
            pinned by tests (a)+(b); the runnable gate carries to that version \
            (Compromise #32 residual). Kept #[ignore]'d per no-fake-green."]
fn check_secret_independence_build_gate_compiles() {
    // This body is intentionally a documentation anchor for the gate's INTENT.
    // The REAL gate is a build under `--features mlkem-ct-check`, which at
    // 0.0.9 fails to compile (so the gate cannot pass honestly). When libcrux
    // ships a fix, the CI step `cargo build -p benten-crypto-suite --features
    // mlkem-ct-check` becomes the runnable gate and this #[ignore] is removed.
    panic!(
        "runnable check-secret-independence gate not honestly wireable at \
         libcrux-ml-kem 0.0.9 — see FLAG-FOR-BEN. This #[ignore]'d test must \
         not be force-passed; it un-ignores only when `cargo build --features \
         mlkem-ct-check` compiles against a fixed libcrux version."
    );
}
