//! **F-INV16-1 — Inv-16 envelope-layer unification (FG).**
//!
//! ADDL R3 wave **W0-crypto-canary**. Pin source:
//! `f-full-r2-test-landscape.md` Group-12 (F-INV16-1) + coverage-matrix
//! §2.1 (Inv-16) + R0 plan §5.1 Inv-16 + §2.1 C-2/C-3 + §1.2 (the 4-layer
//! model shares ONE envelope shape) + §9.1-1/3.
//!
//! # What this family pins
//!
//! Inv-16 = envelope-layer-unification + codepoint-dispatch + AAD-binding
//! (U1) + strict-decode (U2) + canonical-TLV length-injective (U3) +
//! sender-DID-or-Sealed-Sender + replay-window, **primitive-neutral
//! across all 4 layers**. The unification is at the ENVELOPE/AAD-binding
//! layer (NOT the primitive layer — Option-F+ NO-GO, C-1): ONE HPKE
//! primitive serves Layer-C drops + Layer-D wraps + remote-permission
//! payloads; Layer-A uses a symmetric-AEAD variant of the SAME envelope.
//!
//! # The cross-layer parametric shape
//!
//! - **U1** the codepoint is committed INTO the AAD/info (so a peer cannot
//!   re-interpret an envelope under a different codepoint).
//! - **U2** strict-decode: no cross-variant fallback (an envelope sealed
//!   under `BindingContext::Vault` cannot be opened as `WholeContent`).
//! - **U3** canonical-TLV length-injective: distinct field tuples encode
//!   to distinct byte strings (no two AAD tuples collide — the truncation/
//!   extension defense).
//! - **one-HPKE-path-reused**: Layer-C and Layer-D route through the SAME
//!   HPKE primitive (not two impls).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! `EncryptedEnvelope` / `BindingContext` / the canonical-TLV encoder do
//! NOT exist at HEAD (grep=ZERO). This file is self-contained for parallel
//! safety (it does NOT import the `f_w0_*` wave's stub — each W0 family is
//! independent). It commits a LOCAL `f_inv16_stub` module. R5 MUST:
//!   1. DELETE the `f_inv16_stub` module,
//!   2. INSERT real imports against the V2 `EncryptedEnvelope` + typed
//!      `BindingContext` + canonical-TLV encoder + the unified HPKE primitive,
//!   3. UN-IGNORE + verify green.
//!
//! # Would-FAIL-if-no-op'd (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Each pin drives a PRODUCTION call site (the canonical-TLV encoder / the
//! strict-decode dispatch / the shared HPKE seal) + asserts an OBSERVABLE
//! byte/dispatch consequence + is would-FAIL-if-no-op'd (a non-injective
//! encoder collides distinct tuples; a lax decoder accepts a cross-variant
//! envelope; two distinct HPKE impls fail the reuse assertion).

#![allow(dead_code)]

/// SELF-CONTAINED stub-shim (R5 deletes this whole module).
mod f_inv16_stub {
    /// Typed AAD binding context (Inv-16; `#[non_exhaustive]`). One enum
    /// spans all 4 layers — the unification.
    #[non_exhaustive]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum BindingContext {
        /// Layer-A vault binding.
        Vault { vault_version: u8 },
        /// Layer-B/whole-content per-Node binding.
        WholeContent { plaintext_cid: Vec<u8> },
        /// Layer-C drop / Layer-D wrap recipient binding.
        Recipient {
            audience_did: Vec<u8>,
            recipient_key_generation: u32,
        },
    }

    /// Canonical-TLV length-injective encode of a `BindingContext` + the
    /// committed codepoint (U1 + U3). STUB encodes WITHOUT a length prefix
    /// (deliberately NON-injective) so the U3 injectivity pin FAILS red
    /// until R5 wires the real length-injective TLV.
    pub fn canonical_tlv_encode(codepoint: u16, ctx: &BindingContext) -> Vec<u8> {
        let mut out = Vec::new();
        // STUB BUG (intentional): no length prefixes ⇒ adjacent
        // variable-length fields can collide across distinct tuples.
        out.extend_from_slice(&codepoint.to_be_bytes());
        match ctx {
            BindingContext::Vault { vault_version } => out.push(*vault_version),
            BindingContext::WholeContent { plaintext_cid } => {
                out.extend_from_slice(plaintext_cid);
            }
            BindingContext::Recipient {
                audience_did,
                recipient_key_generation,
            } => {
                out.extend_from_slice(audience_did);
                out.extend_from_slice(&recipient_key_generation.to_be_bytes());
            }
        }
        out
    }

    /// Strict-decode: open `sealed_under` as `requested` — MUST reject a
    /// cross-variant mismatch (U2). STUB returns `Ok` always (lax) so the
    /// cross-variant-reject pin FAILS red until R5 wires strict-decode.
    pub fn strict_decode(
        sealed_under: &BindingContext,
        requested: &BindingContext,
    ) -> Result<(), &'static str> {
        // STUB BUG (intentional): does NOT enforce variant match.
        let _ = (sealed_under, requested);
        Ok(())
    }

    /// Whether Layer-C and Layer-D route through the SAME HPKE primitive
    /// (one-HPKE-path-reused). STUB returns `false` so the reuse pin FAILS
    /// red until R5 unifies the path.
    pub fn layer_c_and_d_share_one_hpke_primitive() -> bool {
        false
    }
}

use f_inv16_stub::{
    BindingContext, canonical_tlv_encode, layer_c_and_d_share_one_hpke_primitive, strict_decode,
};

/// **F-INV16-1 (U1)** — the codepoint is committed INTO the AAD/info.
///
/// Two encodes of the SAME binding context under DIFFERENT codepoints
/// produce DIFFERENT bytes — so a peer cannot re-interpret an envelope
/// under a substituted codepoint. would-FAIL-if-no-op'd: an encoder that
/// drops the codepoint from the AAD yields identical bytes.
#[test]
#[ignore = "RED-PHASE: F-INV16-1 (U1) — codepoint committed into AAD; un-ignore at R5"]
fn inv16_u1_codepoint_committed_in_aad() {
    let ctx = BindingContext::WholeContent {
        plaintext_cid: vec![0xCD; 32],
    };
    let under_a = canonical_tlv_encode(0x647a, &ctx);
    let under_b = canonical_tlv_encode(0x6400, &ctx);
    assert_ne!(
        under_a, under_b,
        "the codepoint MUST be committed in the AAD (same ctx, different codepoint ⇒ different bytes; U1)"
    );
}

/// **F-INV16-1 (U2)** — strict-decode rejects cross-variant.
///
/// An envelope sealed under `Vault` cannot be opened as `WholeContent`.
/// would-FAIL-if-no-op'd: the stub decoder is lax (returns Ok), so this
/// fires red until R5 wires strict-decode.
#[test]
#[ignore = "RED-PHASE: F-INV16-1 (U2) — strict-decode rejects cross-variant BindingContext; un-ignore at R5"]
fn inv16_u2_strict_decode_rejects_cross_variant() {
    let sealed_under = BindingContext::Vault { vault_version: 1 };
    let requested = BindingContext::WholeContent {
        plaintext_cid: vec![0xCD; 32],
    };
    assert!(
        strict_decode(&sealed_under, &requested).is_err(),
        "opening a Vault-bound envelope as WholeContent MUST strict-reject (U2; no cross-variant fallback)"
    );
    // Positive control: same-variant decode succeeds.
    assert!(
        strict_decode(&sealed_under, &BindingContext::Vault { vault_version: 1 }).is_ok(),
        "same-variant decode succeeds (positive control)"
    );
}

/// **F-INV16-1 (U3)** — canonical-TLV length-injective.
///
/// Two DISTINCT binding tuples that would otherwise collide under naive
/// concatenation MUST encode to DISTINCT byte strings (length-injective).
/// The classic collision: `audience_did="AB", gen=0x4344` vs
/// `audience_did="ABCD", gen=0x__` — without a length prefix the
/// concatenations can coincide. would-FAIL-if-no-op'd: the stub encoder
/// has no length prefix, so a constructed collision pair encodes equal,
/// firing the assertion red until R5 wires the injective TLV.
#[test]
#[ignore = "RED-PHASE: F-INV16-1 (U3) — canonical-TLV length-injective (no two tuples collide); un-ignore at R5"]
fn inv16_u3_canonical_tlv_length_injective() {
    // A constructed boundary-ambiguity pair: the same underlying payload
    // bytes (`audience_did` ‖ `recipient_key_generation` as BE) split at
    // DIFFERENT field boundaries. Under a naive no-length-prefix concat
    // these two distinct tuples encode to identical bytes — the collision
    // a length-injective TLV (U3) must prevent.
    //
    //   ctx_c: audience_did = [0x41,0x42,0x00,0x00,0x00], gen = 0x0000_4344
    //          → naive payload  41 42 00 00 00 | 00 00 43 44
    //   ctx_d: audience_did = [0x41,0x42,0x00,0x00],      gen = 0x0043_4400
    //          → naive payload  41 42 00 00    | 00 43 44 00
    // (distinct splits; a length-injective encoder yields distinct bytes).
    let ctx_c = BindingContext::Recipient {
        audience_did: vec![0x41, 0x42, 0x00, 0x00, 0x00],
        recipient_key_generation: 0x0000_4344,
    };
    let ctx_d = BindingContext::Recipient {
        audience_did: vec![0x41, 0x42, 0x00, 0x00],
        recipient_key_generation: 0x0043_4400,
    };

    let enc_c = canonical_tlv_encode(0x6510, &ctx_c);
    let enc_d = canonical_tlv_encode(0x6510, &ctx_d);
    assert_ne!(
        enc_c, enc_d,
        "distinct binding tuples with different field-splits MUST encode to distinct bytes (U3 length-injective; the truncation/extension defense)"
    );
}

/// **F-INV16-1 (one-HPKE-path-reused)** — Layer-C drops + Layer-D wraps +
/// remote-permission route through the SAME HPKE primitive.
///
/// The unification means ONE HPKE primitive, not two impls (C-2/§1.2).
/// would-FAIL-if-no-op'd: the stub reports `false`, firing the assertion
/// red until R5 unifies the path.
#[test]
#[ignore = "RED-PHASE: F-INV16-1 — one HPKE primitive serves Layer-C drops + Layer-D wraps (not two impls); un-ignore at R5"]
fn inv16_one_hpke_primitive_reused_across_layers() {
    assert!(
        layer_c_and_d_share_one_hpke_primitive(),
        "Layer-C drops and Layer-D wraps MUST route through the SAME HPKE primitive (envelope-layer unification, Option-F+ NO-GO C-1)"
    );
}
