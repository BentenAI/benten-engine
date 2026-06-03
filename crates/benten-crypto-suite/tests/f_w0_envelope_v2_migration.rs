//! **F-W0-1..5 — Wave-0 / `ENVELOPE_FORMAT_VERSION_V2` migration (ALL FG; hard upstream M-20).**
//!
//! ADDL R3 wave **W0-crypto-canary** (the SOLE upstream canary). Pin
//! source: `f-full-r2-test-landscape.md` Group-1 (F-W0-1..5) + §4 P0-3/P0-4
//! + §10.8 seed + R0 plan §2.0 BR-3 (real X-Wing SHA3-256) + §2.1 C-6 +
//! §2.3 Q2 (BE) + §3.2 + §4.1.MIGRATION (ONE V1→V2 bump) + §7.1 Wave-0.
//!
//! # The Wave-0 DAG edge (M-20) — why every byte family rides this
//!
//! `[Wave-0] ──► [every encryption canary]`. Wave-0 MUST merge before any
//! canary authors envelope bytes, so all new codepoints are authored at
//! **V2 + BE + EncryptedEnvelope** from their FIRST commit. This file
//! authors against the V2/BE/`EncryptedEnvelope` end-state from the first
//! `#[ignore]` commit — there is NO surviving V1/LE golden vector. A
//! family authored against the in-tree LE/V1 `AeadEnvelope`
//! (`aead.rs:145,165`) pins soon-to-be-dead bytes.
//!
//! # Ground-truth at HEAD (`grep`/`sed`-verified)
//!
//! - `aead.rs:60` `ENVELOPE_FORMAT_VERSION_V1: u8 = 0x01` (no V2).
//! - `aead.rs:165` writes the codepoint **LITTLE-ENDIAN**
//!   (`cipher_codepoint.raw().to_le_bytes()`); `:244` chunk_index/
//!   total_chunks LE; `:277` recipe_index/total_recipes LE.
//! - `cipher_suite.rs:404 x_wing_combine` computes **HKDF-SHA256** over
//!   `ss_x‖ss_mlkem‖ek_x‖ek_mlkem‖pub_x‖pub_mlkem` with a SHA3-256-of-
//!   info-tag salt + `info="x-wing-v1-benten-0x647a"` — a Benten-private
//!   combiner MISLABELED "X-Wing", NOT the real draft-connolly construction.
//! - The real construction (spec R0.5 §3.2(a) / C-6, verified against
//!   `draft-connolly-cfrg-xwing-kem-10` §6) is
//!   `SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel)` — the 6-byte
//!   `XWingLabel = 0x5c2e2f2f5e5c` (ASCII `\.//^\`) is **APPENDED** as the
//!   suffix, NOT prepended (R4.2-corrected 2026-06-03; the prepended form
//!   is the superseded v01-v02 construction and would freeze a
//!   non-interoperable KEM at the IETF-reserved `0x647A`).
//! - `grep EncryptedEnvelope|BindingContext` → **ZERO** at HEAD (the
//!   shipped type is the flat `AeadEnvelope` with an untyped `&[u8]` AAD).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! Each W0 family is self-contained for parallel safety — NO cross-wave
//! module dependency. This file commits a LOCAL `f_w0_stub` module so it
//! compiles green at baseline behind `#[ignore]`. The R5 closing wave MUST:
//!   1. DELETE the local `f_w0_stub` module,
//!   2. INSERT real imports against the V2 `EncryptedEnvelope` / real
//!      X-Wing `combine_x_wing` (label APPENDED) / BE serializer / the V2
//!      bounded-decode `from_wire_bytes`,
//!   3. UN-IGNORE the tests + verify all PASS green,
//!   4. Regenerate ALL golden/KAT vectors (construction change ⇒ new keys).
//!
//! # R5 destinations for the local source-scan shims (F4-012)
//!
//! The two local source-scan shims are NOT thrown away at un-ignore — each
//! has a NAMED conformance home the R5 closing wave folds them into:
//!   - `wire_path_to_le_bytes_count()` (the M-19 LE-survivor scanner) →
//!     R5 deletes the stub and re-points the F-W0-3 `assert_eq!(…, 0)` pin
//!     at the **delivered workspace conformance helper**
//!     `benten_crypto_suite::conformance::endianness::wire_path_le_survivor_count()`
//!     (the single canonical zero-`to_le_bytes` M-19 gate; same helper the
//!     other byte-families' BE pins consume). The scanner is a delivered
//!     import, not a per-file local.
//!   - `info_tag_ascii_flagged_by_be_scanner()` (the m-1 ASCII-not-flagged
//!     negative) → folds into the SAME `conformance::endianness` helper as
//!     its `ascii_label_excluded()` query.
//!
//! # Would-FAIL-if-no-op'd (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Every pin drives a PRODUCTION call site (the real combiner / the real
//! BE serializer / the real `EncryptedEnvelope` constructor / the V2
//! bounded-decode decoder) + asserts an OBSERVABLE byte-level consequence
//! + is would-FAIL-if-no-op'd (the negative `assert_ne!` arms FAIL if the
//! in-tree HKDF/LE/`AeadEnvelope` survives). NO `assert_eq!(CONST,
//! CONST_VAL)` shapes; NO zero-assertion arms.

#![allow(dead_code)]

/// SELF-CONTAINED stub-shim (R5 deletes this whole module).
mod f_w0_stub {
    /// The V2 format-version discriminator (NEW at Wave-0; in-tree max is
    /// `ENVELOPE_FORMAT_VERSION_V1 = 0x01`). STUB pins the value R5
    /// introduces; the pins below assert the real `EncryptedEnvelope`
    /// serializes byte-1 == this.
    pub const ENVELOPE_FORMAT_VERSION_V2: u8 = 0x02;
    pub const ENVELOPE_FORMAT_VERSION_V1: u8 = 0x01;
    pub const ENVELOPE_MAGIC: u8 = 0xae;

    /// The real draft-connolly X-Wing `XWingLabel` — the 6 bytes
    /// `0x5c2e2f2f5e5c` (ASCII `\.//^\`). **APPENDED** as the suffix of the
    /// combiner pre-image, NOT prepended (spec R0.5 §3.2(a), verified vs
    /// `draft-connolly-cfrg-xwing-kem-10` §6; R4.2-corrected 2026-06-03).
    pub const XWING_LABEL: [u8; 6] = [0x5c, 0x2e, 0x2f, 0x2f, 0x5e, 0x5c];

    /// The real draft-connolly X-Wing combiner (SHA3-256). STUB returns a
    /// deterministic-but-WRONG output so the positive KAT pin and the
    /// negative "≠ legacy HKDF" pin both fail until R5 wires the real
    /// construction. R5 replaces this with
    /// `SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel)` — label APPENDED.
    pub fn combine_x_wing(
        _ss_mlkem: &[u8],
        _ss_x25519: &[u8],
        _ct_x25519: &[u8],
        _pk_x25519: &[u8],
    ) -> [u8; 32] {
        [0u8; 32]
    }

    /// The exact byte sequence fed to `SHA3-256` by the combiner (the
    /// "pre-image"). The F-W0-1-LABEL construction-order witness pin hashes
    /// over this to confirm `XWingLabel` is the **appended suffix**, not a
    /// prepended prefix. STUB deliberately builds the **PREPENDED** layout
    /// (the superseded/wrong order) so the suffix-assertion fires RED until
    /// R5 wires the real appended construction; R5 re-points this at the
    /// real combiner's pre-image builder.
    pub fn x_wing_combiner_preimage(
        ss_mlkem: &[u8],
        ss_x25519: &[u8],
        ct_x25519: &[u8],
        pk_x25519: &[u8],
    ) -> Vec<u8> {
        let mut pre = Vec::new();
        // STUB BUG (intentional): PREPENDS the label (the wrong, superseded
        // v01-v02 order) so the appended-suffix pin is RED at baseline.
        pre.extend_from_slice(&XWING_LABEL);
        pre.extend_from_slice(ss_mlkem);
        pre.extend_from_slice(ss_x25519);
        pre.extend_from_slice(ct_x25519);
        pre.extend_from_slice(pk_x25519);
        pre
    }

    /// The in-tree LEGACY HKDF-SHA256 combiner output for the SAME inputs.
    /// STUB returns a DISTINCT sentinel so the `assert_ne!` negative arm is
    /// meaningful at R5 (R5 wires this to the real legacy combiner to
    /// prove the construction actually changed).
    pub fn legacy_hkdf_combine(
        _ss_mlkem: &[u8],
        _ss_x25519: &[u8],
        _ek_x: &[u8],
        _ek_mlkem: &[u8],
        _pub_x: &[u8],
        _pub_mlkem: &[u8],
    ) -> [u8; 32] {
        [0xFFu8; 32]
    }

    /// The published draft-connolly X-Wing KAT output for a fixed
    /// fixture (encap-determinism-from-seed). STUB = a sentinel distinct
    /// from `combine_x_wing`'s stub so the KAT pin is RED until R5.
    pub fn draft_connolly_x_wing_kat_for_fixture() -> [u8; 32] {
        [0x11u8; 32]
    }

    /// The classical-only `0x6400` combiner (must stay consistent with the
    /// real-X-Wing rewrite per §3.2(a)). STUB returns a sentinel.
    pub fn classical_combine_for_fixture() -> [u8; 32] {
        [0x22u8; 32]
    }

    /// Typed binding context (the lifted, typed AAD — replaces the
    /// in-tree untyped `&[u8]` AAD). `#[non_exhaustive]` per Inv-16.
    #[non_exhaustive]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum BindingContext {
        /// Layer-A vault binding.
        Vault { vault_version: u8 },
        /// Whole-content per-Node binding.
        WholeContent { plaintext_cid: Vec<u8> },
    }

    /// The lifted codepoint-dispatched envelope (renames + lifts the flat
    /// `AeadEnvelope`). `#[non_exhaustive]` payload + typed `BindingContext`.
    #[derive(Debug, Clone)]
    pub struct EncryptedEnvelope {
        pub format_version: u8,
        pub cipher_codepoint: u16,
        pub aad_binding: BindingContext,
        pub nonce: Vec<u8>,
        pub ciphertext: Vec<u8>,
    }

    /// Hard upper bound on a decoded nonce length (12 or 24 bytes are the
    /// only legal AEAD nonce widths; anything larger is a hostile/garbage
    /// declared length-prefix). The V2 bounded-decode decoder MUST reject a
    /// declared `nonce_len` exceeding this BEFORE allocating/reading
    /// (META #629 unbounded-decode DoS class).
    pub const MAX_NONCE_LEN: usize = 24;

    impl EncryptedEnvelope {
        /// Serialize to wire bytes — V2 layout, codepoint **BIG-ENDIAN**.
        /// STUB writes LITTLE-ENDIAN + V1 deliberately so the BE pin + the
        /// V2 pin both FAIL until R5 wires the real BE/V2 serializer.
        pub fn to_wire_bytes(&self) -> Vec<u8> {
            let mut out = Vec::new();
            out.push(ENVELOPE_MAGIC);
            // STUB BUG (intentional): writes V1 + LE so the red-phase pins fire.
            out.push(ENVELOPE_FORMAT_VERSION_V1);
            out.extend_from_slice(&self.cipher_codepoint.to_le_bytes());
            out.push(self.nonce.len() as u8);
            out.extend_from_slice(&self.nonce);
            out.extend_from_slice(&self.ciphertext);
            out
        }

        /// Decode — V1 bytes MUST be typed-rejected post-V2-freeze. STUB
        /// accepts everything so the V1-rejected pin is RED until R5.
        pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
            if bytes.len() < 5 {
                return Err("too short");
            }
            // STUB BUG (intentional): does NOT reject V1.
            Ok(Self {
                format_version: bytes[1],
                cipher_codepoint: u16::from_le_bytes([bytes[2], bytes[3]]),
                aad_binding: BindingContext::WholeContent {
                    plaintext_cid: Vec::new(),
                },
                nonce: Vec::new(),
                ciphertext: Vec::new(),
            })
        }

        /// Bounded-decode the declared `nonce_len` length-prefix (byte[4])
        /// against the remaining buffer + `MAX_NONCE_LEN` BEFORE reading or
        /// allocating (META #629 flagship). STUB does NO bound check + uses
        /// `with_capacity(declared)` so a hostile declared length triggers
        /// an unbounded pre-allocation — the F4-009-BD pin asserts this is
        /// typed-rejected, which is RED until R5 wires the bounded decoder.
        pub fn decode_nonce_bounded(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
            if bytes.len() < 5 {
                return Err("too short");
            }
            let declared = bytes[4] as usize;
            // STUB BUG (intentional): NO bound check against MAX_NONCE_LEN
            // and NO check against the remaining buffer. A real bounded
            // decoder MUST reject `declared > MAX_NONCE_LEN` and
            // `declared > bytes.len() - 5` BEFORE allocating. The stub
            // instead pre-allocates the attacker-declared capacity and
            // copies whatever is present, modelling the deployed #629 hole.
            let mut buf = Vec::with_capacity(declared);
            let avail = (bytes.len() - 5).min(declared);
            buf.extend_from_slice(&bytes[5..5 + avail]);
            Ok(buf)
        }
    }

    /// Source-scan: count of `to_le_bytes` occurrences surviving on any
    /// wire/AAD path. STUB returns the in-tree count (3 known sites in
    /// `aead.rs` + more across the workspace) so the zero-scanner pin is
    /// RED. R5 deletes this stub and re-points the F-W0-3 pin at the
    /// delivered `conformance::endianness::wire_path_le_survivor_count()`
    /// helper (the named M-19 destination above), which MUST return 0.
    pub fn wire_path_to_le_bytes_count() -> usize {
        13
    }

    /// Whether the X-Wing HKDF info-tag ASCII string was (wrongly) flagged
    /// by the endianness scanner. STUB returns `false` (correct: the
    /// ASCII info-tag is NOT endianness-affected, m-1) — the negative pin
    /// asserts the scanner does NOT touch it. R5 folds into the same
    /// `conformance::endianness` helper (named destination above).
    pub fn info_tag_ascii_flagged_by_be_scanner() -> bool {
        false
    }
}

use f_w0_stub::{
    BindingContext, ENVELOPE_FORMAT_VERSION_V1, ENVELOPE_FORMAT_VERSION_V2, ENVELOPE_MAGIC,
    EncryptedEnvelope, MAX_NONCE_LEN, XWING_LABEL, classical_combine_for_fixture, combine_x_wing,
    draft_connolly_x_wing_kat_for_fixture, info_tag_ascii_flagged_by_be_scanner,
    legacy_hkdf_combine, wire_path_to_le_bytes_count, x_wing_combiner_preimage,
};
use sha3::{Digest, Sha3_256};

// Fixed X-Wing combiner fixture (stable inputs so the KAT is deterministic).
const SS_MLKEM: [u8; 32] = [0xA1; 32];
const SS_X25519: [u8; 32] = [0xB2; 32];
const CT_X25519: [u8; 32] = [0xC3; 32];
const PK_X25519: [u8; 32] = [0xD4; 32];

/// **F-W0-1** — `0x647A` computes the REAL X-Wing SHA3-256 construction,
/// NOT the in-tree HKDF-SHA256 stand-in.
///
/// Positive: the combiner output equals the published draft-connolly KAT
/// for the fixed fixture. Negative (the would-FAIL guard): the output
/// is NOT the legacy HKDF-SHA256 combiner output (`cipher_suite.rs:404`)
/// for the same inputs — if the in-tree HKDF combiner survives, this
/// `assert_ne!` FAILS.
#[test]
#[ignore = "RED-PHASE: F-W0-1 — 0x647A must compute the real X-Wing SHA3-256 construction (draft-connolly), NOT the in-tree HKDF-SHA256 stand-in (cipher_suite.rs:404); un-ignore at R5"]
fn x_wing_0x647a_uses_real_sha3_256_construction_not_hkdf() {
    let real = combine_x_wing(&SS_MLKEM, &SS_X25519, &CT_X25519, &PK_X25519);
    let legacy = legacy_hkdf_combine(
        &SS_MLKEM, &SS_X25519, &PK_X25519, &SS_MLKEM, &PK_X25519, &SS_MLKEM,
    );

    // The construction CHANGED — real X-Wing ≠ the legacy HKDF combiner.
    assert_ne!(
        real, legacy,
        "real X-Wing SHA3-256 combiner MUST differ from the in-tree HKDF-SHA256 stand-in (else the construction never changed)"
    );
    // The real construction is byte-faithful to the draft-connolly KAT.
    assert_eq!(
        real,
        draft_connolly_x_wing_kat_for_fixture(),
        "0x647A combiner must equal the draft-connolly X-Wing KAT for the fixed fixture"
    );
}

/// **F-W0-1-LABEL (F4-002 + F4-003)** — the X-Wing combiner pre-image
/// **APPENDS** the 6-byte `XWingLabel = 0x5c2e2f2f5e5c` (ASCII `\.//^\`)
/// as the trailing suffix `SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel)`,
/// NOT prepended (spec R0.5 §3.2(a) / C-6, verified vs
/// `draft-connolly-cfrg-xwing-kem-10` §6; R4.2-corrected 2026-06-03).
///
/// This is the construction-order witness pin the R4.2 review demanded
/// (F4-003): the corpus previously froze only opaque sentinel KATs, so a
/// wrong/absent label or a prepended ordering was UNCAUGHT. The pin now
/// makes a wrong order observable and would-FAIL-if-no-op'd:
///   (a) the explicit `XWING_LABEL` const equals the published 6 bytes —
///       pins the label value itself (no longer absent from the corpus);
///   (b) the combiner pre-image ENDS WITH the 6 label bytes (appended
///       suffix) and the 4 input shared-secrets/ciphertext/pubkey precede
///       it; the stub builds the PREPENDED (wrong) order, so the
///       suffix-assertion FAILS until R5 wires the real appended pre-image;
///   (c) the label is NOT a prefix of the pre-image (explicit
///       would-FAIL-on-the-superseded-order guard).
#[test]
#[ignore = "RED-PHASE: F-W0-1-LABEL (F4-002/003) — XWingLabel=0x5c2e2f2f5e5c APPENDED as suffix of the combiner pre-image (NOT prepended); stub prepends ⇒ RED; un-ignore at R5"]
fn x_wing_label_is_appended_suffix_not_prepended() {
    // (a) The label value itself is pinned (F4-003: previously absent).
    assert_eq!(
        XWING_LABEL,
        [0x5c, 0x2e, 0x2f, 0x2f, 0x5e, 0x5c],
        "XWingLabel must be the 6 bytes 0x5c2e2f2f5e5c (ASCII \\.//^\\) per draft-connolly-cfrg-xwing-kem-10 §6"
    );

    let pre = x_wing_combiner_preimage(&SS_MLKEM, &SS_X25519, &CT_X25519, &PK_X25519);
    let n = pre.len();
    assert!(
        n >= XWING_LABEL.len() + SS_MLKEM.len(),
        "pre-image must contain the four inputs plus the label"
    );

    // (b) APPENDED: the final 6 bytes of the pre-image ARE the label, and
    //     the four inputs precede it in order. would-FAIL while the stub
    //     prepends the label (the superseded v01-v02 ordering).
    assert_eq!(
        &pre[n - XWING_LABEL.len()..],
        &XWING_LABEL[..],
        "XWingLabel must be APPENDED as the trailing suffix of the combiner pre-image (NOT prepended); a prepended label freezes a non-interoperable KEM at the IETF-reserved 0x647A"
    );
    assert_eq!(
        &pre[..SS_MLKEM.len()],
        &SS_MLKEM[..],
        "the pre-image must begin with ss_M (ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel ordering)"
    );

    // (c) would-FAIL-on-the-superseded-order guard: the label is NOT the
    //     leading prefix of the pre-image.
    assert_ne!(
        &pre[..XWING_LABEL.len()],
        &XWING_LABEL[..],
        "XWingLabel must NOT be prepended (the prepended form is the superseded v01-v02 construction)"
    );

    // Construction-order witness: hashing the real appended pre-image
    // reproduces the combiner output (the label position changes the hash,
    // so a prepended-vs-appended mismatch is byte-observable here).
    let mut h = Sha3_256::new();
    h.update(&pre);
    let preimage_digest: [u8; 32] = h.finalize().into();
    assert_eq!(
        preimage_digest,
        combine_x_wing(&SS_MLKEM, &SS_X25519, &CT_X25519, &PK_X25519),
        "the combiner output must equal SHA3-256 over the appended-label pre-image (construction-order witness)"
    );
}

/// **F-W0-1 (cont.)** — the classical `0x6400` combiner stays consistent
/// after the real-X-Wing rewrite (§3.2(a): "re-derive the classical-
/// downgrade combiner consistently").
///
/// **F4-009 strengthening (was green-when-intended-RED):** the prior arms
/// (`hybrid != classical`, `classical != [0;32]`) both passed green against
/// the stub (`combine_x_wing → [0;32]`, `classical → [0x22;32]`) — so this
/// pin on the SOLE upstream canary asserted nothing about the rewrite. The
/// load-bearing RED arm is now: **the hybrid `0x647A` combiner produces the
/// real draft-connolly X-Wing output** (drives the real SHA3-256
/// construction). The stub's `combine_x_wing` returns `[0;32]` ≠ the KAT
/// `[0x11;32]` → RED; R5's real combiner == the KAT → GREEN. The
/// distinct-arms + well-defined-classical assertions are kept as the
/// consistency property (the rewrite must not collapse the two arms).
#[test]
#[ignore = "RED-PHASE: F-W0-1 — classical 0x6400 combiner re-derived consistently with the real-X-Wing rewrite (hybrid must equal the real draft-connolly KAT); un-ignore at R5"]
fn classical_0x6400_combiner_consistent_after_rewrite() {
    let hybrid = combine_x_wing(&SS_MLKEM, &SS_X25519, &CT_X25519, &PK_X25519);
    let classical = classical_combine_for_fixture();

    // Load-bearing RED arm: the hybrid arm IS the real X-Wing construction
    // (the rewrite happened). would-FAIL while the stub combiner returns
    // [0;32] instead of the real draft-connolly KAT.
    assert_eq!(
        hybrid,
        draft_connolly_x_wing_kat_for_fixture(),
        "the hybrid 0x647A combiner MUST be the real draft-connolly X-Wing SHA3-256 construction (else the classical re-derivation rides a stand-in, not the rewrite)"
    );
    // The rewrite must not collapse the two arms: classical ≠ hybrid.
    assert_ne!(
        hybrid, classical,
        "classical 0x6400 and hybrid 0x647A combiners must derive distinct keys (the rewrite must not collapse arms)"
    );
    // The classical arm is well-defined (non-zero / non-sentinel) — it
    // produces a real key, not a degenerate value.
    assert_ne!(
        classical, [0u8; 32],
        "classical 0x6400 combiner must produce a real derived key"
    );
}

/// **F-W0-2** — X-Wing interop KAT round-trips against the published
/// draft-connolly vectors byte-for-byte (encap-determinism-from-seed).
///
/// Decap-of-encap recovers the same shared output; a wrong recipient key
/// does NOT. (External-vector seed: R5 swaps the synthesized witness for
/// the real draft-connolly corpus per §5-D.)
#[test]
#[ignore = "RED-PHASE: F-W0-2 — X-Wing interop KAT byte-for-byte against published draft-connolly vectors; un-ignore at R5"]
fn x_wing_interop_kat_byte_for_byte() {
    let derived = combine_x_wing(&SS_MLKEM, &SS_X25519, &CT_X25519, &PK_X25519);
    let kat = draft_connolly_x_wing_kat_for_fixture();
    assert_eq!(
        derived, kat,
        "X-Wing combiner output must match the published draft-connolly KAT row byte-for-byte"
    );
    // Negative: a wrong X25519 ciphertext (ct_X) yields a DIFFERENT
    // derived key — the combiner binds all four inputs.
    let wrong_ct = combine_x_wing(&SS_MLKEM, &SS_X25519, &[0u8; 32], &PK_X25519);
    assert_ne!(
        wrong_ct, kat,
        "X-Wing combiner must bind ct_X (wrong ct_X ⇒ different derived key)"
    );
}

/// **F-W0-3** — BE endianness sweep + the **zero-`to_le_bytes`** scanner
/// (the flagship M-19 family; single highest-leverage conformance gate).
///
/// (a) the source scanner reports ZERO `to_le_bytes` survivors on any
/// wire/AAD path; (b) the X-Wing info-tag ASCII string is NOT flagged
/// (m-1: ASCII strings are not endianness-affected). would-FAIL-if-no-
/// op'd: at HEAD the count is non-zero (LE present at `aead.rs:165,244,277`).
///
/// (F4-012) The LE-survivor scanner has a NAMED R5 home — see the module
/// header's "R5 destinations for the local source-scan shims" block: R5
/// re-points this pin at
/// `benten_crypto_suite::conformance::endianness::wire_path_le_survivor_count()`.
#[test]
#[ignore = "RED-PHASE: F-W0-3 — zero `to_le_bytes` survives on any wire/AAD path (M-19 flagship); un-ignore at R5"]
fn zero_to_le_bytes_survives_on_wire_or_aad_paths() {
    assert_eq!(
        wire_path_to_le_bytes_count(),
        0,
        "NO `to_le_bytes` may survive on any wire/AAD path (at HEAD: aead.rs:165,244,277 + aead_wrap.rs + plugin_manifest.rs are LE)"
    );
    // m-1 negative: the X-Wing info-tag ASCII string is NOT an integer
    // field and MUST NOT be flagged by the endianness scanner.
    assert!(
        !info_tag_ascii_flagged_by_be_scanner(),
        "the X-Wing info-tag ASCII string is not endianness-affected and must NOT be flagged (m-1)"
    );
}

/// **F-W0-3 (cont.)** — the codepoint is serialized **BIG-ENDIAN** on the
/// wire (per-field BE hex-pin).
///
/// Drives the real `EncryptedEnvelope::to_wire_bytes` + asserts bytes
/// [2..4] are the codepoint in BE order. would-FAIL-if-no-op'd: the
/// in-tree serializer writes LE (`aead.rs:165`), so the BE-order
/// assertion FAILS until R5.
#[test]
#[ignore = "RED-PHASE: F-W0-3 — codepoint serialized BIG-ENDIAN on the wire (aead.rs:165 is LE today); un-ignore at R5"]
fn codepoint_serialized_big_endian_on_wire() {
    let env = EncryptedEnvelope {
        format_version: ENVELOPE_FORMAT_VERSION_V2,
        // 0x647A — a value whose BE and LE byte orders DIFFER, so the
        // pin actually distinguishes the two.
        cipher_codepoint: 0x647A,
        aad_binding: BindingContext::WholeContent {
            plaintext_cid: vec![0xCD; 32],
        },
        nonce: vec![0u8; 12],
        ciphertext: vec![0xEE; 16],
    };
    let wire = env.to_wire_bytes();
    // bytes[0]=magic, [1]=format_version, [2..4]=codepoint BE.
    assert_eq!(wire[0], ENVELOPE_MAGIC, "byte 0 = envelope magic");
    assert_eq!(
        [wire[2], wire[3]],
        0x647Au16.to_be_bytes(),
        "codepoint must be serialized BIG-ENDIAN (network byte order per RFC 9180 / FIPS 203 / MLS)"
    );
    // Explicit would-FAIL guard: the bytes are NOT in LE order.
    assert_ne!(
        [wire[2], wire[3]],
        0x647Au16.to_le_bytes(),
        "codepoint must NOT be little-endian (the aead.rs:165 LE layout is migrated away)"
    );
}

/// **F-W0-4** — `AeadEnvelope` → `EncryptedEnvelope` rename + lift + typed
/// `BindingContext` (rename+lift of a SHIPPED type, NOT greenfield).
///
/// Constructs an `EncryptedEnvelope` carrying a TYPED `BindingContext`
/// (replacing the untyped `&[u8]` AAD at `aead.rs:145`); pins that the
/// codepoint is committed in the typed binding's serialized form + that a
/// cross-variant `BindingContext` mismatch strict-rejects (U2).
///
/// **F4-009 strengthening (was enum-literal-only):** the prior arms
/// (`matches!(aad_binding, Vault{..})`, `aad_binding != whole`) were both
/// trivially true at construction and pinned nothing on the SOLE upstream
/// canary. The lift's load-bearing observable is the **serialized wire
/// form** — the lifted `EncryptedEnvelope` MUST serialize the V2 format
/// version + the codepoint BIG-ENDIAN (the typed-binding lift rides the
/// SAME single V1→V2 bump). The stub `to_wire_bytes` writes V1 + LE
/// deliberately, so the wire-form assertions fire RED until R5 wires the
/// real lifted serializer. The variant-distinction assertions are kept as
/// the typed-AAD shape property.
#[test]
#[ignore = "RED-PHASE: F-W0-4 — AeadEnvelope→EncryptedEnvelope lift + typed BindingContext (grep=ZERO at HEAD; stub serializes V1/LE ⇒ wire-form arms fire red); un-ignore at R5"]
fn aead_envelope_lifts_to_encrypted_envelope_with_typed_binding() {
    let vault = EncryptedEnvelope {
        format_version: ENVELOPE_FORMAT_VERSION_V2,
        cipher_codepoint: 0x6100,
        aad_binding: BindingContext::Vault { vault_version: 1 },
        nonce: vec![0u8; 24],
        ciphertext: vec![0x01; 16],
    };
    // The typed binding is a Vault binding, not WholeContent — the type
    // system distinguishes the AAD shape (vs the untyped `&[u8]` today).
    assert!(
        matches!(
            vault.aad_binding,
            BindingContext::Vault { vault_version: 1 }
        ),
        "EncryptedEnvelope carries a TYPED BindingContext (Vault) — replaces the untyped &[u8] AAD at aead.rs:145"
    );
    // U2 strict-decode: a WholeContent binding is NOT a Vault binding —
    // the variants are distinct (cross-variant reinterpretation rejected).
    let whole = BindingContext::WholeContent {
        plaintext_cid: vec![0xCD; 32],
    };
    assert_ne!(
        vault.aad_binding, whole,
        "cross-variant BindingContext mismatch must be distinguishable (U2 strict-decode, no cross-variant fallback)"
    );

    // Load-bearing RED arm: the lifted envelope's WIRE FORM rides the
    // single V1→V2 bump + BE codepoint. would-FAIL while the stub
    // serializer writes V1 + LE (the rename/lift never landed).
    let wire = vault.to_wire_bytes();
    assert_eq!(
        wire[0], ENVELOPE_MAGIC,
        "the lifted EncryptedEnvelope serializes byte-0 == envelope magic"
    );
    assert_eq!(
        wire[1], ENVELOPE_FORMAT_VERSION_V2,
        "the lifted EncryptedEnvelope MUST serialize byte-1 == V2 (the lift rides the single V1→V2 bump); would-FAIL while the stub writes V1"
    );
    assert_eq!(
        [wire[2], wire[3]],
        0x6100u16.to_be_bytes(),
        "the lifted EncryptedEnvelope MUST serialize the vault codepoint BIG-ENDIAN (M-19); would-FAIL while the stub writes LE"
    );
}

/// **F-W0-5** — exactly ONE `V1→V2` bump covers W0-1 + W0-3 + W0-4 (NOT
/// three independent bumps); V1 bytes typed-reject post-freeze.
///
/// (a) the new envelope serializes `format_version == V2`; (b) V1 ≠ V2
/// (the bump is real); (c) a V1-framed byte stream is typed-rejected by
/// the V2 decoder. would-FAIL-if-no-op'd: the stub serializer writes V1
/// (so the V2 pin fires) + the stub decoder accepts V1 (so the reject pin
/// fires) until R5 wires the real single-bump migration.
#[test]
#[ignore = "RED-PHASE: F-W0-5 — single V1→V2 bump (covers BR-3 + BE + lift) + V1 typed-rejected; un-ignore at R5"]
fn single_v1_to_v2_bump_and_v1_typed_rejected() {
    // The bump is real (a distinct discriminator value).
    assert_ne!(
        ENVELOPE_FORMAT_VERSION_V2, ENVELOPE_FORMAT_VERSION_V1,
        "V2 must be a distinct format-version discriminator from V1"
    );

    let env = EncryptedEnvelope {
        format_version: ENVELOPE_FORMAT_VERSION_V2,
        cipher_codepoint: 0x647A,
        aad_binding: BindingContext::WholeContent {
            plaintext_cid: vec![0xCD; 32],
        },
        nonce: vec![0u8; 12],
        ciphertext: vec![0xEE; 16],
    };
    let wire = env.to_wire_bytes();
    // The serialized byte-1 is V2 (the single-bump coverage meta-assertion).
    assert_eq!(
        wire[1], ENVELOPE_FORMAT_VERSION_V2,
        "the migrated envelope serializes byte-1 == V2 (BR-3 + BE + lift all ride ONE bump)"
    );

    // A V1-framed stream is typed-rejected by the V2 decoder.
    let v1_bytes = {
        let mut b = vec![ENVELOPE_MAGIC, ENVELOPE_FORMAT_VERSION_V1];
        b.extend_from_slice(&0x647Au16.to_be_bytes());
        b.push(0); // nonce-len
        b
    };
    assert!(
        EncryptedEnvelope::from_wire_bytes(&v1_bytes).is_err(),
        "a V1-framed byte stream must be typed-rejected post-V2-freeze (no silent V1 acceptance)"
    );
}

/// **F-W0-BD-1 (F4-009-BD)** — the V2 bounded-decode decoder REJECTS a
/// hostile declared length-prefix BEFORE allocating/reading (META #629
/// unbounded-decode DoS class; flagship anchor on the SOLE upstream
/// canary).
///
/// # F4-009-BD disposition (HARD RULE 12; flagship-now + already-named home)
///
/// R4.2 surfaced (F4-009/F4-030) that NO test in the F-full corpus asserts
/// a wire decoder rejects a hostile declared length-prefix before
/// allocating — the META #629 deployed-invariant class on the
/// freeze-gating wire surfaces. The R4.3 brief routes F4-009-BD to THIS
/// canary file. Disposition (two parts, both HARD-RULE-valid):
///
///   1. **FIX-NOW (lands here):** the flagship `EncryptedEnvelope` V2
///      nonce-len reject arm below — RED at `#[ignore]` against the stub's
///      unbounded `with_capacity(declared)` (the deployed #629 hole). This
///      closes the "no decoder-reject test anywhere on the upstream
///      canary" gap NOW, not at R5.
///   2. **BELONGS-NAMED-NOW (already-persisted destination):** the
///      per-surface remainder (`HpkeMultiBase.recipient_count`,
///      `DropBundlePayload.{node_count,ct_len,cek_len}`, the 9-tuple
///      permission TLV `aud_len`, the DAG-CBOR vault/members-table decode)
///      is the **F4-009/F4-030 "META #629 decode-side bounded-decode
///      family" already dispositioned R5-fill-named in the committed
///      `.addl/phase-4-meta/r4-2-triage.md`** ("R5-fill-named" bucket +
///      "META #629 decode-side bounded-decode family … per wire surface
///      (HpkeMultiBase / PermissionRequest / DropBundlePayload /
///      TokenBindingAad)"). That triage row IS the landed named home —
///      this anchor does NOT invent a new landscape row (a docstring that
///      claims a doc edit this single-file write-scope cannot perform
///      would be a phantom destination). Each per-surface reject arm is
///      authored in its own W-family file against its own decoder at R5,
///      un-ignored with the rest of that surface's pins.
///
/// # Two ORTHOGONAL bounds (F-BD-001 — half-pinned-contract closure)
///
/// A bounded length-prefix decoder has TWO independent failure modes, and
/// META #629 is only closed when BOTH are pinned:
///   - **(I) absolute bound** — `declared > MAX_NONCE_LEN` (a garbage/hostile
///     width beyond any legal AEAD nonce; the `with_capacity(declared)`
///     unbounded-pre-allocation DoS).
///   - **(II) remaining-buffer bound** — `declared ≤ MAX_NONCE_LEN` yet
///     `declared > bytes_remaining` (a within-MAX width that still slices
///     past the end of the frame → slice-overread PANIC on
///     `&bytes[5..5 + declared]`).
///
/// Pinning ONLY (I) is the half-pinned-contract trap: an R5 implementer
/// could ship `if declared > MAX_NONCE_LEN { reject }`, pass green, and
/// still PANIC on a `declared=20`-with-0-bytes-present frame — the precise
/// panic-DoS this SOLE upstream canary exists to anchor. This pin closes
/// the WHOLE class by iterating a table that exercises each bound
/// independently (the orthogonal "exceeds-remaining-only-within-MAX" row),
/// so a future bound addition extends the table rather than spawning a
/// sibling arm that could silently leave a bound unpinned.
///
/// would-FAIL-if-no-op'd: the stub `decode_nonce_bounded` does NO bound
/// check on EITHER axis and `with_capacity(declared)`-pre-allocates the
/// attacker-declared length (modelling the deployed #629 hole). At baseline
/// `declared=255,0-present` → `Ok(vec![])` (avail=0) and `declared=20,
/// 0-present` → `Ok(vec![])` (avail=0); BOTH hostile rows expect `is_err()`,
/// so BOTH fire RED until R5 wires the bounded decoder that rejects each
/// bound independently.
#[test]
#[ignore = "RED-PHASE: F-W0-BD-1 (F4-009-BD / META #629) — V2 decoder must typed-reject a hostile declared nonce-len prefix on BOTH bounds (absolute MAX + remaining-buffer) BEFORE allocating; stub bounds neither ⇒ RED; un-ignore at R5"]
fn v2_decode_rejects_hostile_length_prefix_before_allocating() {
    // (declared_nonce_len, nonce_bytes_present, expect_ok, label).
    // The table pins the two bounds ORTHOGONALLY (F-BD-001): row 1 violates
    // BOTH bounds, row 2 violates ONLY the remaining-buffer bound (within
    // MAX yet past the frame end), row 3 is the within-both positive
    // control. A future bound extends this table, not a sibling test.
    let cases: [(u8, usize, bool, &str); 3] = [
        (255, 0, false, "exceeds both: declared 255 > MAX_NONCE_LEN(24) AND > 0 present"),
        (
            20,
            0,
            false,
            "exceeds-remaining-only-within-MAX: declared 20 <= MAX_NONCE_LEN(24) yet > 0 present (slice-overread bound)",
        ),
        (12, 12, true, "within-both-OK: declared 12 <= MAX_NONCE_LEN(24) AND == 12 present"),
    ];

    // Fixture sanity: row 2 isolates the remaining-buffer bound — it MUST
    // stay within MAX_NONCE_LEN so it does NOT trip the absolute bound (if
    // it did, an absolute-only decoder would spuriously pass this pin and
    // the orthogonality would be lost).
    assert!(
        (cases[1].0 as usize) <= MAX_NONCE_LEN && (cases[1].0 as usize) > cases[1].1,
        "fixture sanity: the remaining-buffer-only row must be within MAX_NONCE_LEN yet exceed the bytes present"
    );
    // Fixture sanity: row 1 trips the absolute bound (the unbounded-alloc DoS).
    assert!(
        (cases[0].0 as usize) > MAX_NONCE_LEN,
        "fixture sanity: the exceeds-both row must exceed MAX_NONCE_LEN (the unbounded pre-allocation threat)"
    );

    for (declared, present, expect_ok, label) in cases {
        // Build a V2 frame with the declared nonce-len prefix at byte[4]
        // and exactly `present` nonce bytes following it.
        let frame = {
            let mut b = vec![ENVELOPE_MAGIC, ENVELOPE_FORMAT_VERSION_V2];
            b.extend_from_slice(&0x647Au16.to_be_bytes());
            b.push(declared);
            b.extend_from_slice(&vec![0u8; present]);
            b
        };
        let decoded = EncryptedEnvelope::decode_nonce_bounded(&frame);
        if expect_ok {
            // Positive control: the bound rejects hostile inputs, not all
            // inputs — a within-both, fully-present nonce-len decodes to
            // exactly that length.
            assert_eq!(
                decoded
                    .unwrap_or_else(|e| panic!("[{label}] a within-both, fully-present nonce-len must decode, got Err({e})"))
                    .len(),
                declared as usize,
                "[{label}] a within-bounds declared nonce-len that matches the bytes present decodes to exactly that length"
            );
        } else {
            // The bounded decoder MUST typed-reject the hostile declared
            // length BEFORE allocating/slicing, on EITHER bound. would-FAIL
            // while the stub bounds neither axis and returns Ok.
            assert!(
                decoded.is_err(),
                "[{label}] a declared nonce-len that violates the absolute MAX or the remaining-buffer bound MUST be typed-rejected before allocation (META #629 bounded-decode)"
            );
        }
    }
}
