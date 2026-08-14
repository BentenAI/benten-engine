//! R19 secret-hygiene pin (Compromise #66): `UnwrappedKey` (the recovered
//! `k_root` from `unwrap_key_material`) must NOT leak its raw bytes via
//! `Debug`, and must zeroize on drop.
//!
//! Mirrors the `f_va_4_memory_hygiene_secretbox_zeroize.rs` redacted-Debug
//! convention: the assertion is on the actual rendered string (the observable
//! consequence), using a distinct-byte fixture so a leaking Debug would be
//! unambiguous.

#![allow(clippy::unwrap_used)]

use benten_crypto_suite::cipher_suite::{CipherSuite, CipherSuiteCodepoint};

#[test]
fn unwrapped_key_debug_redacts_the_recovered_k_root() {
    // Wrap/unwrap a distinctive k_root through the LIVE hybrid suite, then
    // render the recovered `UnwrappedKey` via Debug and assert the raw bytes
    // do NOT appear.
    let suite =
        CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).expect("0x647a LIVE");
    let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);

    // Distinct-byte k_root so a leaking Debug is unambiguous.
    let mut k_root = [0u8; 32];
    let distinctive: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xD0];
    k_root[..8].copy_from_slice(&distinctive);

    let wrapped = suite.wrap_key_material(kp.public(), &k_root).unwrap();
    let recovered = suite.unwrap_key_material(kp.secret(), &wrapped).unwrap();

    // The recovered bytes are correct (sanity: the surface still works).
    assert_eq!(recovered.as_bytes(), &k_root, "unwrap must recover k_root");

    // But Debug must redact. R6-tail F-41 sweep: assert the CONTIGUOUS decimal
    // SEQUENCE a leaking (derived) `Debug` would emit for the distinctive
    // prefix — NOT the individual decimals. A bare `222` / `173` could
    // false-positively collide with an unrelated integer field's rendering,
    // making the old scan both fragile and imprecise. This matches the
    // `LEAK_DECIMAL` convention in
    // `crates/benten-engine/tests/f_secret_hygiene_roster.rs` and the sibling
    // `crates/benten-membership-set/tests/f_r19_group_aad_inputs_secret_hygiene.rs`.
    // `UnwrappedKey.bytes` is a `Vec<u8>`, so a derived Debug renders
    // `[222, 173, 190, 239, 202, 254, 186, 208, 0, 0, ...]`.
    let rendered = format!("{recovered:?}");
    const LEAK_DECIMAL: &str = "222, 173, 190, 239, 202, 254, 186, 208";
    assert!(
        !rendered.contains(LEAK_DECIMAL),
        "UnwrappedKey Debug MUST redact the recovered k_root (Compromise #66) — \
         a coredump / log line MUST NOT contain the key; a derived Debug leaks \
         it as `{LEAK_DECIMAL}`; rendered=`{rendered}`"
    );
    // Positive guard: the marker must replace the `bytes` FIELD wholesale, not
    // merely appear somewhere in the render.
    assert!(
        rendered.contains("bytes: \"<redacted>\""),
        "UnwrappedKey Debug MUST replace the bytes field wholesale with the \
         redaction marker; rendered=`{rendered}`"
    );
}
