//! E-02 closure — ABSOLUTE literal-byte golden + per-field anti-little-endian
//! guards over `InstallRecord::signing_payload()`, the CLAUDE.md baked-in #18
//! install-consent SIGNING PRE-IMAGE.
//!
//! # Why this file exists (the R6 falsification finding, E-02)
//!
//! The mutation sweep flipped ALL FIVE `to_be_bytes()` calls in
//! `signing_payload()` to `to_le_bytes()` and `benten-platform-foundation`
//! stayed **227/227 PASS**. Three independent reasons nothing fired:
//!
//! 1. `signing_payload()` is a ONE-WAY pre-image. There is no decoder, so no
//!    round-trip test can ever disagree with the encoder.
//! 2. Signer and verifier BOTH call `signing_payload()`, so a layout change is
//!    BILATERAL — `verify_user_signature()` keeps verifying happily.
//! 3. The only test that touched these bytes
//!    (`f_inj_1_install_record_signing_payload_injective.rs`) asserts
//!    INJECTIVITY (`assert_ne!` across two payloads). Byte-order is a
//!    permutation, and permutations preserve injectivity EXACTLY — so that
//!    pin is structurally incapable of catching an endianness flip.
//!
//! A layout drift is therefore invisible in-tree and surfaces only as
//! install-record signatures that stop verifying ACROSS engine builds — i.e.
//! after the `phase-4-meta-core-close` freeze, un-retrofittably. This is the
//! artifact recording a user's consent to a plugin's capability envelope; if
//! it stops verifying, the whole Layer-1 user-as-root trust anchor breaks.
//!
//! # What the three tests here do, and why one golden is not enough
//!
//! - `install_record_signing_payload_golden_hex` — the absolute byte pin.
//! - `install_record_signing_payload_fields_are_big_endian` — per-field BE
//!   guards. These exist BECAUSE a lone golden is defeated by the obvious
//!   wrong fix: flip the encoder, re-capture the golden, ship. The per-field
//!   guards are computed from `to_be_bytes()` at assert-time, so they keep
//!   firing no matter how the golden is re-captured.
//! - `d84_nonce_and_plugin_did_are_length_prefixed_exactly_once` — the Row
//!   D-84 landmine guard (see the header on that test).
//!
//! Plus `fixture_values_are_endianness_discriminating`, which pins the pins:
//! every fixture integer must have `to_be_bytes() != to_le_bytes()`, so the
//! anti-LE guards can never silently decay into tautologies.

#![allow(clippy::unwrap_used)]

use benten_core::{CID_LEN, Cid};
use benten_id::did::Did;
use benten_platform_foundation::plugin_manifest::InstallRecord;

// ---------------------------------------------------------------------------
// Fixture — every field literal so `signing_payload()` is fully deterministic.
//
// Each pinned integer is deliberately NON-PALINDROMIC under byte reversal
// (no zeros, no symmetric values) so that BE and LE encodings differ. A
// palindromic value (e.g. a length of 0) would make the anti-LE assertions
// vacuously true; `fixture_values_are_endianness_discriminating` enforces this.
// ---------------------------------------------------------------------------

const FIXTURE_MANIFEST_DIGEST: [u8; 32] = [0x33; 32];
const FIXTURE_TS: u64 = 0x0011_2233_4455_6677;
const FIXTURE_NONCE: [u8; 16] = [
    0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF,
];
const FIXTURE_PLUGIN_DID: &str = "did:key:zPLUGINFIXTURE";
const FIXTURE_CAP_0: [u8; 3] = [0xC0, 0xC1, 0xC2];
const FIXTURE_CAP_1: [u8; 1] = [0xD0];

/// Field offsets into the pre-image. Derived (not magic numbers) so the layout
/// this file pins is readable as the layout `signing_payload()` documents:
/// `manifest_cid(36) ‖ ts(8) ‖ len(nonce) ‖ nonce ‖ len(did) ‖ did ‖
///  cap_count ‖ (len(cap) ‖ cap)*`.
const OFF_CID: usize = 0;
const OFF_TS: usize = OFF_CID + CID_LEN;
const OFF_NONCE_LEN: usize = OFF_TS + 8;
const OFF_NONCE: usize = OFF_NONCE_LEN + 4;
const OFF_DID_LEN: usize = OFF_NONCE + FIXTURE_NONCE.len();
const OFF_DID: usize = OFF_DID_LEN + 4;
const OFF_CAP_COUNT: usize = OFF_DID + FIXTURE_PLUGIN_DID.len();
const OFF_CAP_0_LEN: usize = OFF_CAP_COUNT + 4;
const OFF_CAP_0: usize = OFF_CAP_0_LEN + 4;
const OFF_CAP_1_LEN: usize = OFF_CAP_0 + FIXTURE_CAP_0.len();
const OFF_CAP_1: usize = OFF_CAP_1_LEN + 4;

/// Total pre-image length for this fixture. Pinned explicitly because a
/// DROPPED, ADDED or DOUBLE-PREFIXED field changes it even when every
/// surviving field still looks individually well-formed.
const EXPECTED_TOTAL_LEN: usize = OFF_CAP_1 + FIXTURE_CAP_1.len();

fn fixture_record() -> InstallRecord {
    InstallRecord {
        manifest_cid: Cid::from_blake3_digest(FIXTURE_MANIFEST_DIGEST),
        // `signing_payload()` reads `plugin_did.as_str()` verbatim (no
        // resolution), so the fixture string IS the exact pre-image segment.
        plugin_did: Did::from_string_for_test_fixture(FIXTURE_PLUGIN_DID.to_string()),
        // NOT in the pre-image by design (r2-cp-5 omit-by-design: the signer is
        // bound by pubkey-verify, not by literal bytes). Held constant anyway.
        consenting_user_did: Did::from_string_for_test_fixture("did:key:zUSER".to_string()),
        user_signature: Vec::new(),
        timestamp_stub_nanos: FIXTURE_TS,
        nonce: FIXTURE_NONCE.to_vec(),
        granted_caps_bytes: vec![FIXTURE_CAP_0.to_vec(), FIXTURE_CAP_1.to_vec()],
    }
}

/// Lowercase-hex encoder (no `hex` crate dep in this workspace).
fn to_hex(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn be_u32_at(payload: &[u8], off: usize) -> u32 {
    u32::from_be_bytes(payload[off..off + 4].try_into().unwrap())
}

// ---------------------------------------------------------------------------
// 1. The absolute byte pin.
// ---------------------------------------------------------------------------

/// PROVENANCE: this constant was PRODUCED BY THE REAL ENCODER at R6 round #1,
/// never hand-authored (M-20 throwaway-compute discipline), by running:
///
/// ```text
/// CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_BUILD_JOBS=6 \
///   cargo nextest run -p benten-platform-foundation \
///   --test canonical_bytes_v1_install_record_signing_preimage \
///   install_record_signing_payload_golden_hex --nocapture
/// ```
///
/// Shape: 212 hex chars = 106 bytes. Field-by-field, the captured value is
/// `manifest_cid(36) ‖ ts(8, BE) ‖ 00000010 ‖ nonce(16) ‖ 00000016 ‖
/// plugin_did(22) ‖ cap_count 00000002 ‖ 00000003 c0c1c2 ‖ 00000001 d0` —
/// every u32 prefix visibly big-endian.
///
/// The command is kept so a future maintainer can RE-DERIVE the value when a
/// signing-format change is deliberate and ratified. **If this test fails and
/// you did not intend a format change, the encoder regressed — fix
/// `plugin_manifest.rs`, not this literal.** Re-capturing to clear a red is the
/// wrong fix here specifically: this is a SIGNATURE PRE-IMAGE, so a silent
/// change invalidates every `InstallRecord` ever signed.
const GOLDEN_SIGNING_PAYLOAD_HEX: &str = "01711e203333333333333333333333333333333333333333333333333333333333333333001122334455667700000010a0a1a2a3a4a5a6a7a8a9aaabacadaeaf000000166469643a6b65793a7a504c5547494e464958545552450000000200000003c0c1c200000001d0";

/// E-02: the literal frozen bytes of the install-consent signing pre-image.
///
/// MUTATION THAT MUST FAIL THIS PIN — any one of these one-line edits in
/// `crates/benten-platform-foundation/src/plugin_manifest.rs`:
///   - `:631` `self.timestamp_stub_nanos.to_be_bytes()` -> `.to_le_bytes()`
///   - `:637` `nonce_len.to_be_bytes()`                 -> `.to_le_bytes()`
///   - `:640` `plugin_did_len.to_be_bytes()`            -> `.to_le_bytes()`
///   - `:646` `cap_count.to_be_bytes()`                 -> `.to_le_bytes()`
///   - `:649` `cap_len.to_be_bytes()`                   -> `.to_le_bytes()`
/// ...or any field reorder, any dropped/added field, any prefix-width change.
/// Every one of those left the crate 227/227 PASS before this file existed.
#[test]
fn install_record_signing_payload_golden_hex() {
    let payload = fixture_record().signing_payload();
    let got = to_hex(&payload);

    assert_eq!(
        payload.len(),
        EXPECTED_TOTAL_LEN,
        "E-02: pre-image length drifted — a field was added, dropped or \
         re-prefixed.\nactual> {got}"
    );
    assert_eq!(
        got, GOLDEN_SIGNING_PAYLOAD_HEX,
        "E-02: the install-consent signing pre-image drifted from the frozen \
         v1-beta bytes. A frozen SIGNATURE pre-image just changed, so every \
         previously-signed InstallRecord stops verifying. Fix the encoder in \
         plugin_manifest.rs; re-derive this literal ONLY if the format change \
         is deliberate and ratified.\nactual> {got}"
    );
}

// ---------------------------------------------------------------------------
// 2. Per-field big-endian guards (independent of the golden).
// ---------------------------------------------------------------------------

/// E-02 / M-19: every integer in the pre-image is big-endian.
///
/// These guards are computed from `to_be_bytes()` AT ASSERT TIME, so unlike the
/// golden they cannot be neutralised by re-capturing a constant. This is the
/// pin that survives the wrong fix ("flip the encoder, re-record the golden").
///
/// MUTATION THAT MUST FAIL THIS PIN: flip ANY of the five `to_be_bytes()` calls
/// at `plugin_manifest.rs:631/637/640/646/649` to `to_le_bytes()`. Each field is
/// asserted independently, so flipping one, some, or all five fires here.
#[test]
fn install_record_signing_payload_fields_are_big_endian() {
    let payload = fixture_record().signing_payload();

    // manifest_cid — fixed 36 bytes, no prefix (boundary unambiguous).
    let expected_cid = Cid::from_blake3_digest(FIXTURE_MANIFEST_DIGEST);
    assert_eq!(
        &payload[OFF_CID..OFF_CID + CID_LEN],
        &expected_cid.as_bytes()[..],
        "E-02: manifest_cid must lead the pre-image, verbatim"
    );

    // timestamp_stub_nanos — BE u64 (M-19; migrated from LE at F-full Wave-0).
    assert_eq!(
        &payload[OFF_TS..OFF_TS + 8],
        &FIXTURE_TS.to_be_bytes()[..],
        "E-02/M-19: timestamp_stub_nanos must be BIG-endian"
    );
    assert_ne!(
        &payload[OFF_TS..OFF_TS + 8],
        &FIXTURE_TS.to_le_bytes()[..],
        "E-02/M-19: timestamp_stub_nanos is LITTLE-endian — plugin_manifest.rs:631 regressed"
    );

    // nonce length prefix — BE u32 (F-INJ-1 seam).
    let nonce_len = u32::try_from(FIXTURE_NONCE.len()).unwrap();
    assert_eq!(
        &payload[OFF_NONCE_LEN..OFF_NONCE_LEN + 4],
        &nonce_len.to_be_bytes()[..],
        "E-02/M-19: len(nonce) must be BIG-endian"
    );
    assert_ne!(
        &payload[OFF_NONCE_LEN..OFF_NONCE_LEN + 4],
        &nonce_len.to_le_bytes()[..],
        "E-02/M-19: len(nonce) is LITTLE-endian — plugin_manifest.rs:637 regressed"
    );

    // plugin_did length prefix — BE u32 (F-INJ-1 seam).
    let did_len = u32::try_from(FIXTURE_PLUGIN_DID.len()).unwrap();
    assert_eq!(
        &payload[OFF_DID_LEN..OFF_DID_LEN + 4],
        &did_len.to_be_bytes()[..],
        "E-02/M-19: len(plugin_did) must be BIG-endian"
    );
    assert_ne!(
        &payload[OFF_DID_LEN..OFF_DID_LEN + 4],
        &did_len.to_le_bytes()[..],
        "E-02/M-19: len(plugin_did) is LITTLE-endian — plugin_manifest.rs:640 regressed"
    );

    // granted_caps element count — BE u32 (M-2b).
    let cap_count = 2u32;
    assert_eq!(
        &payload[OFF_CAP_COUNT..OFF_CAP_COUNT + 4],
        &cap_count.to_be_bytes()[..],
        "E-02/M-19: granted_caps count must be BIG-endian"
    );
    assert_ne!(
        &payload[OFF_CAP_COUNT..OFF_CAP_COUNT + 4],
        &cap_count.to_le_bytes()[..],
        "E-02/M-19: granted_caps count is LITTLE-endian — plugin_manifest.rs:646 regressed"
    );

    // per-cap length prefixes — BE u32 (M-2b). Two caps of DIFFERENT lengths so
    // a swapped-order or shared-prefix bug cannot pass by coincidence.
    let cap_0_len = u32::try_from(FIXTURE_CAP_0.len()).unwrap();
    let cap_1_len = u32::try_from(FIXTURE_CAP_1.len()).unwrap();
    assert_eq!(
        &payload[OFF_CAP_0_LEN..OFF_CAP_0_LEN + 4],
        &cap_0_len.to_be_bytes()[..],
        "E-02/M-19: len(granted_caps[0]) must be BIG-endian"
    );
    assert_ne!(
        &payload[OFF_CAP_0_LEN..OFF_CAP_0_LEN + 4],
        &cap_0_len.to_le_bytes()[..],
        "E-02/M-19: len(granted_caps[0]) is LITTLE-endian — plugin_manifest.rs:649 regressed"
    );
    assert_eq!(
        &payload[OFF_CAP_1_LEN..OFF_CAP_1_LEN + 4],
        &cap_1_len.to_be_bytes()[..],
        "E-02/M-19: len(granted_caps[1]) must be BIG-endian"
    );
    assert_ne!(
        &payload[OFF_CAP_1_LEN..OFF_CAP_1_LEN + 4],
        &cap_1_len.to_le_bytes()[..],
        "E-02/M-19: len(granted_caps[1]) is LITTLE-endian — plugin_manifest.rs:649 regressed"
    );

    // Field ORDER + payload bodies. Catches the AAD-order mutation class
    // (swapping two adjacent variable segments) which endianness guards alone
    // would not see.
    assert_eq!(
        &payload[OFF_NONCE..OFF_NONCE + FIXTURE_NONCE.len()],
        &FIXTURE_NONCE[..],
        "E-02: nonce body must follow its length prefix, in order"
    );
    assert_eq!(
        &payload[OFF_DID..OFF_DID + FIXTURE_PLUGIN_DID.len()],
        FIXTURE_PLUGIN_DID.as_bytes(),
        "E-02: plugin_did body must follow the nonce, in order"
    );
    assert_eq!(
        &payload[OFF_CAP_0..OFF_CAP_0 + FIXTURE_CAP_0.len()],
        &FIXTURE_CAP_0[..],
        "E-02: granted_caps[0] body must follow its length prefix"
    );
    assert_eq!(
        &payload[OFF_CAP_1..OFF_CAP_1 + FIXTURE_CAP_1.len()],
        &FIXTURE_CAP_1[..],
        "E-02: granted_caps[1] body must follow granted_caps[0]"
    );
}

// ---------------------------------------------------------------------------
// 3. ★ Row D-84 landmine guard.
// ---------------------------------------------------------------------------

/// ★ Row D-84 guard — `nonce` and `plugin_did` are length-prefixed EXACTLY ONCE.
///
/// `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-84 previously stated that this
/// seam "is NOT length-delimited" and queued hardening work to length-prefix
/// both fields. That statement was STALE: F-INJ-1 had already landed the
/// prefixes (`plugin_manifest.rs:636-641`). An implementer executing the row as
/// written would have added a SECOND prefix layer to a FROZEN signature
/// pre-image, silently invalidating every previously-signed `InstallRecord`.
/// The row is now recorded AS-BUILT; this test is what makes the correction
/// enforceable rather than merely written down.
///
/// MUTATION THAT MUST FAIL THIS PIN: add a redundant second length prefix, e.g.
/// insert `out.extend_from_slice(&nonce_len.to_be_bytes());` a second time after
/// `plugin_manifest.rs:637`. The byte at `OFF_NONCE` then starts another u32
/// length instead of the nonce body, and the total length grows by 4.
#[test]
fn d84_nonce_and_plugin_did_are_length_prefixed_exactly_once() {
    let payload = fixture_record().signing_payload();

    // Exactly ONE BE-u32 precedes the nonce, and it equals the nonce length.
    assert_eq!(
        be_u32_at(&payload, OFF_NONCE_LEN) as usize,
        FIXTURE_NONCE.len(),
        "D-84: the nonce carries exactly one BE-u32 length prefix"
    );
    // ...and what follows it is the RAW nonce, not a second prefix.
    assert_eq!(
        &payload[OFF_NONCE..OFF_NONCE + FIXTURE_NONCE.len()],
        &FIXTURE_NONCE[..],
        "D-84: a SECOND length-prefix layer was added to a frozen pre-image — \
         every previously-signed InstallRecord would stop verifying"
    );

    assert_eq!(
        be_u32_at(&payload, OFF_DID_LEN) as usize,
        FIXTURE_PLUGIN_DID.len(),
        "D-84: plugin_did carries exactly one BE-u32 length prefix"
    );
    assert_eq!(
        &payload[OFF_DID..OFF_DID + FIXTURE_PLUGIN_DID.len()],
        FIXTURE_PLUGIN_DID.as_bytes(),
        "D-84: a SECOND length-prefix layer was added ahead of plugin_did"
    );

    // A doubled prefix on either field adds 4 bytes each; pin the exact total.
    assert_eq!(
        payload.len(),
        EXPECTED_TOTAL_LEN,
        "D-84: pre-image total length drifted — check for a doubled length prefix"
    );
}

// ---------------------------------------------------------------------------
// 4. Pin the pins.
// ---------------------------------------------------------------------------

/// The anti-LE assertions above compare against `to_le_bytes()`. If any fixture
/// integer were palindromic under byte reversal (a zero length, a symmetric
/// value), `to_be_bytes() == to_le_bytes()` and those `assert_ne!` guards would
/// be VACUOUSLY satisfied — a passing test proving nothing, which is the exact
/// failure mode E-02 exists to close.
///
/// MUTATION THAT MUST FAIL THIS PIN: set `FIXTURE_TS` to `0`, or shrink
/// `FIXTURE_CAP_1` to an empty slice (length 0 encodes as `00000000` in both
/// byte orders).
#[test]
fn fixture_values_are_endianness_discriminating() {
    assert_ne!(
        FIXTURE_TS.to_be_bytes(),
        FIXTURE_TS.to_le_bytes(),
        "fixture timestamp must distinguish BE from LE"
    );
    for (label, value) in [
        ("len(nonce)", u32::try_from(FIXTURE_NONCE.len()).unwrap()),
        (
            "len(plugin_did)",
            u32::try_from(FIXTURE_PLUGIN_DID.len()).unwrap(),
        ),
        ("cap_count", 2u32),
        ("len(cap[0])", u32::try_from(FIXTURE_CAP_0.len()).unwrap()),
        ("len(cap[1])", u32::try_from(FIXTURE_CAP_1.len()).unwrap()),
    ] {
        assert_ne!(
            value.to_be_bytes(),
            value.to_le_bytes(),
            "fixture {label} = {value} is palindromic under byte reversal — the \
             anti-LE guard for this field would be vacuous"
        );
    }
}
