//! GAP-KDB Shape-B — CS-1 `kem_cp` ⟺ component-multicodec cross-check
//! (algorithm-confusion reject matrix, design C2). W1 RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` ratified correction **C2**
//! ("Cross-check `kem_cp (0x647a) ⟺ component set {0xec,0x120c}`,
//! fail-closed on disagreement (defeats algorithm-confusion)") + §5
//! (0x120c/0xec self-describing wiring, X25519-first). Fork C default:
//! "wire 0x120c + cross-check `kem_cp`". `GAP-KDB-B-R2-LANDSCAPE.md` §3 W1
//! (CS-1) + §4 TIER-C + `RK-4` (the benten-id resolve_kem consumer of this
//! same cross-check).
//!
//! # Why the cross-check is LOAD-BEARING (not redundant with `from_bytes`)
//!
//! `RecipientPublic::from_bytes(kem_cp, payload)` only sees the
//! concatenated `x25519(32) ‖ mlkem768_ek(1184)` payload AFTER the
//! component multicodec varints are stripped, and dispatches purely on the
//! out-of-band `kem_cp`. It therefore CANNOT catch a multikey whose
//! payload length is correct for `kem_cp` but whose component varints
//! declare the WRONG algorithms (X25519 ↔ ML-KEM swapped / ML-KEM-1024
//! substituted / a signing codec spliced into the KEM slot). The CS-1
//! multikey layer is the ONLY place the component codecs are checked
//! against `kem_cp`. The arms below marked **[from_bytes-blind]** have a
//! payload that `from_bytes` would ACCEPT — only the CS-1 cross-check
//! rejects them.
//!
//! # SELF-CONTAINED (no `benten_id::kdb_testing`) + stub-shim
//!
//! `benten-id` depends on `benten-crypto-suite`, so no cross-crate fixture
//! import. The CS-1 entry is a local `todo!()` stub (identical to
//! `kdb_cs1_kem_recipient_wiring.rs`); R5 replaces the body with the real
//! delegation + un-ignores. Component sizes flow from the named constants
//! (`X25519_PUBLIC_LEN`, `ML_KEM_768_EK_LEN`) — never hardcoded (#5).
//!
//! # would_fail_on_revert
//!
//! Each arm's `assert!(is_err())` flips to `Ok` the moment the component
//! ⟺ `kem_cp` cross-check is dropped: a decoder that strips the varints
//! positionally without checking their VALUES against `kem_cp` accepts a
//! PQ-first multikey (mis-parsing ML-KEM bytes as the x25519 half — the
//! C2 X25519-first freeze), an ML-KEM-1024-tagged component, or a
//! classical suite carrying an ML-KEM half → algorithm confusion succeeds.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_variables)]

use benten_crypto_suite::cipher_suite::{ML_KEM_768_EK_LEN, RecipientPublic, X25519_PUBLIC_LEN};

/// `x25519-pub = 0xec` → varint `[0xec, 0x01]` (the mandated first
/// component of `kem_cp = 0x647a`).
const X25519_PUB_MULTICODEC: [u8; 2] = [0xec, 0x01];
/// `mlkem-768-pub = 0x120c` → varint `[0x8c, 0x24]` (the mandated second
/// component of `kem_cp = 0x647a`).
const MLKEM768_PUB_MULTICODEC: [u8; 2] = [0x8c, 0x24];
/// `mlkem-1024-pub = 0x120d` → varint `[0x8d, 0x24]`. A DIFFERENT
/// ML-KEM parameter set — a `0x647a` doc MUST NOT accept it in the ML-KEM
/// slot (algorithm confusion).
const MLKEM1024_PUB_MULTICODEC: [u8; 2] = [0x8d, 0x24];
/// `ed25519-pub = 0xed` → varint `[0xed, 0x01]`. A SIGNING codec; splicing
/// it into a KEM slot is the classic wrong-role confusion.
const ED25519_PUB_MULTICODEC: [u8; 2] = [0xed, 0x01];

/// Opaque X25519-sized filler. Validity is irrelevant — the cross-check
/// rejects on the component-codec disagreement BEFORE any key parse.
fn x25519_filler() -> Vec<u8> {
    vec![0xA5u8; X25519_PUBLIC_LEN]
}
/// Opaque ML-KEM-768-EK-sized filler.
fn mlkem768_ek_filler() -> Vec<u8> {
    vec![0xC3u8; ML_KEM_768_EK_LEN]
}

fn frame(codec: [u8; 2], payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + payload.len());
    out.extend_from_slice(&codec);
    out.extend_from_slice(payload);
    out
}
fn concat(a: Vec<u8>, b: Vec<u8>) -> Vec<u8> {
    let mut out = a;
    out.extend_from_slice(&b);
    out
}

/// The CS-1 logic-under-test (RED-PHASE stub → real entry at R5). Mirror
/// of `kdb_cs1_kem_recipient_wiring.rs::cs1_shim`.
mod cs1_shim {
    use super::RecipientPublic;
    use benten_crypto_suite::AeadError;

    /// See `kdb_cs1_kem_recipient_wiring.rs` for the full contract. Here
    /// the exercised property is step 2: the `kem_cp ⟺ component-codec`
    /// cross-check that fails closed on algorithm confusion (C2).
    pub fn recipient_public_from_kem_multikey(
        _kem_cp: u16,
        _kem_multikey: &[u8],
    ) -> Result<RecipientPublic, AeadError> {
        todo!(
            "RED-PHASE (CS-1): kem_cp⟺components cross-check (C2 \
             algorithm-confusion defense) lands at R5 (GAP-KDB-B W1). \
             un-ignore then."
        )
    }
}

const HYBRID: u16 = 0x647a;
const CLASSICAL: u16 = 0x6400;

// ── A1 [from_bytes-blind] — PQ-first order rejected (the C2 freeze) ───────

/// A PQ-first multikey `[0x120c‖ek(1184) ‖ 0xec‖x25519(32)]` under
/// `kem_cp=0x647a` MUST reject. This is the operational C2 pin: the
/// stripped payload `ek(1184)‖x25519(32)` is 1216 B — the correct total
/// length for `0x647a` — so `from_bytes` would ACCEPT it and mis-read the
/// leading ML-KEM bytes as the x25519 half. Only the component-ORDER
/// cross-check (x25519 MUST lead) rejects it.
#[test]
#[ignore = "RED-PHASE: CS-1 PQ-first kem multikey rejected (C2 X25519-first freeze) — un-ignore at R5"]
fn cs1_crosscheck_rejects_pq_first_order() {
    let pq_first = concat(
        frame(MLKEM768_PUB_MULTICODEC, &mlkem768_ek_filler()),
        frame(X25519_PUB_MULTICODEC, &x25519_filler()),
    );
    let outcome = cs1_shim::recipient_public_from_kem_multikey(HYBRID, &pq_first);
    assert!(
        outcome.is_err(),
        "PQ-first kem multikey MUST fail closed under kem_cp=0x647a \
         (C2 mandates X25519-first); would-FAIL if the decoder strips \
         varints positionally without enforcing the x25519-leads order — \
         it would mis-parse ML-KEM bytes as the x25519 half"
    );
}

// ── A2 [from_bytes-blind] — wrong ML-KEM parameter codec rejected ─────────

/// `[0xec‖x25519(32) ‖ 0x120d‖1184 B]` under `kem_cp=0x647a` MUST reject:
/// the second component is tagged ML-KEM-1024 (`0x120d`), not the ML-KEM-768
/// (`0x120c`) that `0x647a` mandates. The payload length is correct for
/// `0x647a`, so ONLY the codec cross-check rejects.
#[test]
#[ignore = "RED-PHASE: CS-1 wrong ML-KEM parameter codec (0x120d) rejected under 0x647a — un-ignore at R5"]
fn cs1_crosscheck_rejects_wrong_mlkem_parameter_codec() {
    let confused = concat(
        frame(X25519_PUB_MULTICODEC, &x25519_filler()),
        frame(MLKEM1024_PUB_MULTICODEC, &mlkem768_ek_filler()),
    );
    let outcome = cs1_shim::recipient_public_from_kem_multikey(HYBRID, &confused);
    assert!(
        outcome.is_err(),
        "an ML-KEM-1024-tagged (0x120d) component MUST fail closed under \
         kem_cp=0x647a (which mandates ML-KEM-768 0x120c); would-FAIL if \
         the decoder ignores the second component's codec value \
         (algorithm confusion — the wrong-parameter key would be accepted)"
    );
}

// ── A2b [from_bytes-blind] — signing codec spliced into the KEM slot ──────

/// `[0xed‖x25519(32) ‖ 0x120c‖1184 B]` under `kem_cp=0x647a` MUST reject:
/// the first component is tagged `ed25519-pub` (a SIGNING codec), not the
/// `x25519-pub` KEM codec `0x647a` mandates. Wrong-role confusion; payload
/// length correct so only the cross-check rejects.
#[test]
#[ignore = "RED-PHASE: CS-1 signing codec (0xed) in the KEM slot rejected under 0x647a — un-ignore at R5"]
fn cs1_crosscheck_rejects_signing_codec_in_kem_slot() {
    let confused = concat(
        frame(ED25519_PUB_MULTICODEC, &x25519_filler()),
        frame(MLKEM768_PUB_MULTICODEC, &mlkem768_ek_filler()),
    );
    let outcome = cs1_shim::recipient_public_from_kem_multikey(HYBRID, &confused);
    assert!(
        outcome.is_err(),
        "an ed25519-pub (0xed, a signing codec) in the KEM slot MUST fail \
         closed under kem_cp=0x647a (which mandates x25519-pub 0xec); \
         would-FAIL if the decoder ignores the first component's codec \
         value (wrong-role confusion)"
    );
}

// ── A3 — classical-only component set under a hybrid claim ────────────────

/// `[0xec‖x25519(32)]` alone (no ML-KEM component) under `kem_cp=0x647a`
/// MUST reject: the component set `{0xec}` is missing the mandated ML-KEM
/// `0x120c`. (Also length-caught by `from_bytes`, but the cross-check
/// rejects it earlier at the codec-SET level with an algorithm-confusion
/// error rather than a length error.)
#[test]
#[ignore = "RED-PHASE: CS-1 classical-only component set rejected under hybrid 0x647a — un-ignore at R5"]
fn cs1_crosscheck_rejects_classical_only_components_under_hybrid() {
    let classical_only = frame(X25519_PUB_MULTICODEC, &x25519_filler());
    let outcome = cs1_shim::recipient_public_from_kem_multikey(HYBRID, &classical_only);
    assert!(
        outcome.is_err(),
        "a classical-only component set {{0xec}} MUST fail closed under \
         kem_cp=0x647a (which mandates {{0xec, 0x120c}}); would-FAIL if \
         the hybrid suite accepted a multikey with no ML-KEM component"
    );
}

// ── A4 — hybrid component set under a classical claim (reverse) ───────────

/// `[0xec‖x25519(32) ‖ 0x120c‖1184 B]` under `kem_cp=0x6400` MUST reject:
/// the classical-only suite mandates the component set `{0xec}`, but an
/// ML-KEM `0x120c` component is present. The reverse-direction confusion
/// (a below-PQ-floor suite carrying PQ bytes it will not use).
#[test]
#[ignore = "RED-PHASE: CS-1 hybrid component set rejected under classical 0x6400 — un-ignore at R5"]
fn cs1_crosscheck_rejects_hybrid_components_under_classical() {
    let hybrid_components = concat(
        frame(X25519_PUB_MULTICODEC, &x25519_filler()),
        frame(MLKEM768_PUB_MULTICODEC, &mlkem768_ek_filler()),
    );
    let outcome = cs1_shim::recipient_public_from_kem_multikey(CLASSICAL, &hybrid_components);
    assert!(
        outcome.is_err(),
        "a hybrid component set {{0xec, 0x120c}} MUST fail closed under \
         kem_cp=0x6400 (which mandates {{0xec}} only); would-FAIL if the \
         classical suite silently accepted an unmandated ML-KEM component"
    );
}
