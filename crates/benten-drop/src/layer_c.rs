//! Layer-C encrypt-to-recipient — HPKE `mode_base` + Sealed-Sender drops.
//!
//! G-CORE-3f / F-full Layer-C. This is the codepoint-dispatched
//! encrypt-to-recipient surface: a body is bulk-sealed under a fresh
//! content-encryption-key (CEK), the CEK is HPKE-key-wrapped to the
//! recipient via the unified X25519⊕ML-KEM-768 X-Wing KEM at codepoint
//! `0x647a` ([`benten_crypto_suite::hpke`]), and a codepoint-discriminated
//! plaintext AAD binds the recipient-targeting metadata.
//!
//! # Sealed-Sender DEFAULT (`0x6510`, BR-1)
//!
//! The v1-beta DEFAULT single-recipient drop is **Sealed-Sender**
//! ([`seal_sealed_sender`], codepoint [`DROP_TO_RECIPIENT_SEALED_SENDER`]
//! = `0x6510`): the sender-DID lives INSIDE the ciphertext (recovered
//! post-decrypt), and the on-wire AAD binds ONLY
//! `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`
//! — never the sender-DID. The non-default plaintext-sender path
//! ([`seal_plaintext_sender`], `0x6500`) binds the sender-DID into the
//! plaintext AAD (U4) and is the paired metadata-disclosure control.
//!
//! # Group multi-stanza (`0x6520`)
//!
//! [`seal_group_multi`] produces one [`HpkeRecipientStanza`] per recipient
//! (codepoint [`LAYER_C_DROP_MULTI_RECIPIENT`] = `0x6520`). The DEFAULT
//! group send HONORS Sealed-Sender (BR-1 ruling 1): each stanza's
//! plaintext AAD is **BLINDED** — it carries the
//! [`audience_set_commitment`] over the *sorted* recipient roster (NEVER
//! the raw roster, closing the #61 social-graph leak), `stanza_index`,
//! `stanza_count` (truncation defense), and `recipient_key_generation`;
//! the inner-sender-DID is sealed inside each stanza's payload. The
//! non-default plaintext-sender group variant
//! ([`seal_group_multi_plaintext_sender`]) binds the sender-DID into the
//! per-stanza AAD (paired control only).
//!
//! # AAD endianness + framing (M-19 / R0.7 §4.1)
//!
//! Every wire integer is **big-endian**. The AAD leads with the dedicated
//! [`AAD_VERSION`] (`0x01`) byte — DISTINCT from the envelope serialization
//! [`ENVELOPE_FORMAT_VERSION`] (`0x02`) — so the AAD-version axis and the
//! format axis never conflate (cross-engine AEAD-open). The single-recipient
//! `audience` is `u32-BE` length-prefixed (R0.7 §4.1:1040); the group
//! `recipient_count` is the band's `u16-BE` cardinality (§4.0 width-
//! unification-REJECTED); the `audience_set_commitment` internally uses a
//! `u32-BE` per-DID length prefix (IDENTICAL to the `0x6610` MembershipSet
//! commitment — never the band u16). The body-CID is bound as a
//! self-describing CIDv1 (`0x01 0x71 0x1e 0x20 ‖ 32-byte BLAKE3`).
//!
//! ## `sender_did` length-prefix width — BY BAND (FROZEN; R9-council F-25 / C-07)
//!
//! The `sender_did` field carries **two DIFFERENT length-prefix widths on two
//! DIFFERENT surfaces** — this asymmetry is DELIBERATE and wire-locked (a
//! future refactor MUST NOT silently unify them):
//!
//! - **M_auth sender-origin-auth binding** (`build_m_auth`) — `lp_u32(sender_did)`
//!   (**u32-BE**). This is the signed authenticity binding; it matches the
//!   `u32-BE` framing used for every other variable field in `M_auth`
//!   (injective canonical framing).
//! - **Plaintext-sender envelope WIRE** (`0x6500` `LAYER_C_DROP` only —
//!   `sender_len u16 BE | sender_did`) — **u16-BE**. This is the on-wire
//!   plaintext-sender header (non-default; Sealed-Sender `0x6510` carries NO
//!   sender on the wire). The `u16` matches the plaintext-sender band's wire
//!   header convention.
//!
//! Sealed-Sender (`0x6510`, the DEFAULT) carries the sender ONLY inside the
//! ciphertext / M_auth binding (u32-BE), never as a wire header. The group
//! bands (`0x6520`/`0x6610`) are Sealed-Sender-by-DEFAULT too; their
//! **non-default plaintext-sender variants** additionally append the sender-DID
//! into the per-stanza AAD trailer, with **band-specific widths**:
//!   - `0x6520` group multi-recipient plaintext-sender AAD trailer:
//!     `sender_len u16 BE | sender_did` (`plaintext_aad_bytes`,
//!     `layer_c.rs:559-562`) — **u16-BE**.
//!   - `0x6610` MembershipSet group plaintext-sender AAD trailer:
//!     `sender_len u32 BE | sender_did` (`assemble_group_aad_local` path,
//!     `layer_c.rs:2192-2195`) — **u32-BE** (matches the `0x6610` per-DID
//!     `u32-BE` roster framing).
//! The widths are independent by design: the M_auth `u32` bounds the signed
//! binding; the `0x6500` / `0x6520` plaintext-sender `u16` and the `0x6610`
//! plaintext-sender `u32` each bound their band's non-default AAD/wire trailer.
//! Mirrored in `docs/V1-WIRE-FORMAT-INVENTORY.md §26`. Freeze-record: Row D-42
//! (C-07).
//!
//! Per CLAUDE.md baked-in #5, all crypto routes through
//! [`benten_crypto_suite`] — this module is concat / framing glue only.

extern crate alloc;

use alloc::vec::Vec;

use benten_crypto_suite::cipher_suite::{CipherSuite, CipherSuiteCodepoint, WrappedKey};
use benten_crypto_suite::{AeadEnvelope, AeadKeyMaterial};
// GAP-KDB Shape-B (W2): the seal-API typestate resolves the recipient KEM key
// FROM the audience DID's content-addressed key-set commitment (design §5 /
// Inv-23), so the identity model + resolver live in `benten-id`.
use benten_id::did::Did;
use benten_id::keyset::KeySetDocument;
// R11 MC-1: `RngCore` in scope for `OsRng.fill_bytes` (fresh random 0x6520
// group CEK). `rand_core` is the workspace CSPRNG crate (see
// `benten_crypto_suite::cipher_suite` `rand_core::OsRng`).
use rand_core::RngCore as _;

/// Fail-closed length-prefix range end (F-12).
///
/// Post-decrypt inner-payload parsing reads `u32-BE` length prefixes from
/// attacker-influenced ciphertext-plaintext. A length is `u32`-sourced (cast
/// to `usize`) and added to a running `usize` offset to bound a slice. On a
/// 32-bit target (wasm32 / 32-bit native) `off + len` can OVERFLOW `usize`
/// and WRAP to a small value that spuriously passes a naive `off + len >
/// total` bounds check — the subsequent `buf[off..off + len]` slice then
/// panics (a co-recipient-only DoS, since the attacker must already be a
/// valid AEAD-opening recipient). This helper performs the addition with
/// `checked_add` and verifies the end is within `total`, returning `None`
/// (→ fail-closed typed reject) on EITHER overflow OR out-of-bounds. There is
/// no behavioral change on 64-bit where the lengths cannot overflow.
#[inline]
fn lp_range_end(off: usize, len: usize, total: usize) -> Option<usize> {
    match off.checked_add(len) {
        Some(end) if end <= total => Some(end),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Sender ORIGIN-AUTHENTICATION (B2) — the per-MESSAGE LAMPS-hybrid signature.
// ---------------------------------------------------------------------------

/// Per-surface domain-separation tag for the Sealed-Sender ORIGIN-AUTH
/// signature (B2). PREFIXED into `M_auth` so a Sealed-Sender auth signature
/// can NEVER be reinterpreted as any other Benten signature surface
/// (offline DropBundle envelope-sig `envelope_sig::ENVELOPE_SIG_DOMAIN`,
/// UCAN-Varsig, device attestation, rotation attestation, …) and vice-versa
/// (cross-context resistance — design §1.1 / §1.4 (f)).
///
/// **F-1 (corrected from the design draft):** domain separation is the
/// PREFIX byte-string here, NOT a LAMPS `with_context` API. There is NO
/// `sign_with_context` call on this surface — the signature is produced via
/// the plain [`benten_crypto_suite::sig::SignatureSuite::sign`] /
/// [`benten_crypto_suite::sig::SignatureSuite::verify`] (empty LAMPS ctx),
/// exactly as `envelope_sig.rs` does. The tag rides inside the signed
/// `M_auth` bytes.
pub const SENDER_AUTH_DOMAIN: &[u8] = b"benten/layer-c/sealed-sender-origin-auth/v1";

/// The default sender-origin-auth signature suite codepoint — the v1-beta
/// LAMPS Composite `id-MLDSA65-Ed25519-SHA512` hybrid
/// ([`benten_crypto_suite::codepoint::SigCodepoint::HYBRID_ED25519_MLDSA65`]
/// = `0x0001`). Carried inside the once-sealed body region (bound into
/// `M_auth`) so the classical-only `0x0002` arm stays a really-built
/// NON-DEFAULT swap and any future suite is an additive upgrade — never a
/// wire-break (CLAUDE.md baked-in #5).
///
/// **F-12: NOT a 4th silent literal.** Derived from the codepoint-registry
/// SSOT ([`benten_crypto_suite::codepoint::SigCodepoint::HYBRID_ED25519_MLDSA65`])
/// via `const fn raw()` — if the SSOT value ever moves, this const moves with
/// it (no independent `0x0001` byte to drift).
pub const SENDER_AUTH_SIG_CODEPOINT: u16 =
    benten_crypto_suite::codepoint::SigCodepoint::HYBRID_ED25519_MLDSA65.raw();

/// Layer-C single-recipient CEK BLAKE3 derivation context (the `0x6500` /
/// `0x6510` per-send content-encryption-key domain prefix). A registered
/// cross-surface domain-separation tag: mirrored in the central
/// [`benten_crypto_suite::domain_registry::LAYER_C_CEK_CONTEXT`] corpus table
/// over which the prefix-free / no-collision invariant runs (the
/// `domain_registry_mirror` test pins byte-equality). Canonical home is HERE.
pub const LAYER_C_CEK_CONTEXT: &[u8] = b"benten-drop:layer-c:cek";

/// Layer-C `0x6520` group bulk-CEK domain-separation context.
///
/// **RESERVED (R11 MC-1) — the `0x6520` group CEK is now a fresh random value,
/// no longer derived from this label.** Deriving the group CEK from PUBLIC wire
/// inputs (`body_cid ‖ sender_did ‖ generation`) under this prefix was a
/// Sealed-Sender confidentiality break (any relay guessing the sender's public
/// DID could recompute the CEK and decrypt the body); `seal_group_impl` now
/// samples a fresh 32-byte CEK from the OS CSPRNG per message and delivers it
/// ONLY via the per-stanza HPKE-wrap. This const is RETAINED (not deleted) as a
/// registered cross-surface domain-separation tag mirrored in
/// [`benten_crypto_suite::domain_registry::LAYER_C_GROUP_CEK_CONTEXT`] so the
/// frozen domain registry + its prefix-free/no-collision drift invariant stay
/// byte-stable. Canonical home is HERE.
pub const LAYER_C_GROUP_CEK_CONTEXT: &[u8] = b"benten-drop:layer-c:group-cek";

/// The fields bound by the per-message `M_auth` ORIGIN-AUTH binding.
///
/// The sender signs `M_auth` ONCE per message (design §1.1, R0.2). The
/// binding is **injective** (every variable field is `u32-BE`
/// length-prefixed) and **all integers are big-endian** (M-19). The
/// recipient INDEPENDENTLY RE-DERIVES this binding from the set-state it
/// already holds — NEVER the attacker-controllable wire value (F-2,
/// SOUNDNESS-CRITICAL): the `audience_commitment` + `generations` come from
/// the recipient's own roster / `K_Set` / generations, NOT from the wire.
///
/// Layout (`build_m_auth`):
/// ```text
/// SENDER_AUTH_DOMAIN
///   ‖ sig_codepoint        (u16 BE)   // 0x0001 default — binds the auth suite
///   ‖ envelope_codepoint   (u16 BE)   // 0x6510 / 0x6520 / 0x6610 — binds the band
///   ‖ lp_u32(sender_did)              // the claimed origin — self-binding
///   ‖ body_cid             (36 B)     // self-describing CIDv1 — binds the CONTENT
///   ‖ lp_u32(audience_commitment)     // binds WHO the message is for (anti-re-target)
///   ‖ generation_count     (u32 BE)   // number of generation words that follow
///   ‖ generations          (u32 BE each, in order)  // key-epoch binding (F-3)
///   ‖ stanza_count         (u32 BE)   // F-01 truncation defense (also covered by the sig)
///   ‖ body_aad_digest      (32 B)     // BLAKE3 of the once-sealed BODY's AEAD AAD bytes
/// ```
pub struct SenderAuthBinding<'a> {
    /// The auth-suite selector (default [`SENDER_AUTH_SIG_CODEPOINT`]).
    pub sig_codepoint: u16,
    /// The envelope band (`0x6510` / `0x6520` / `0x6610`).
    pub envelope_codepoint: u16,
    /// The claimed origin — the sealed sender-DID bytes (a hybrid `did:key`).
    pub sender_did: &'a [u8],
    /// Self-describing CIDv1 over the body digest (36 B).
    pub body_cid: &'a [u8],
    /// The audience commitment the recipient INDEPENDENTLY recomputes from
    /// its own held roster / audience (NEVER the wire value): `0x6510` =
    /// the recipient's OWN audience DID; `0x6520` = the
    /// `audience_set_commitment` over the recipient's independently-held
    /// roster; `0x6610` = the `audience_set_commitment` over the roster the
    /// recipient derives from `K_Set`/set-state.
    pub audience_commitment: &'a [u8],
    /// The key-epoch generation words bound EXPLICITLY (F-3; BE u32 each,
    /// in canonical order). `0x6510`/`0x6520` = `[recipient_key_generation]`;
    /// `0x6610` = `[member_key_generation, membership_set_generation,
    /// role_assignments_generation]` (the `body_aad_digest` does NOT cover
    /// the generations — explicit binding closes the revoked-member
    /// cross-generation replay).
    pub generations: &'a [u32],
    /// The total stanza count (F-01 truncation defense; defense-in-depth).
    pub stanza_count: u32,
    /// BLAKE3 of the once-sealed body region's AEAD AAD bytes (transitively
    /// binds the frozen body-seal AAD with zero new wire-field-set drift).
    pub body_aad_digest: [u8; 32],
}

/// Build the canonical `M_auth` binding bytes (design §1.1). Injective +
/// big-endian. The SAME function is used at seal (to sign) and at open (to
/// re-derive + verify) — guaranteeing seal/verify agree on the byte layout.
#[must_use]
pub fn build_m_auth(b: &SenderAuthBinding<'_>) -> Vec<u8> {
    let mut m = Vec::new();
    m.extend_from_slice(SENDER_AUTH_DOMAIN);
    m.extend_from_slice(&b.sig_codepoint.to_be_bytes());
    m.extend_from_slice(&b.envelope_codepoint.to_be_bytes());
    // lp_u32(sender_did) — injective framing of the variable origin field.
    let sd_len = u32::try_from(b.sender_did.len()).expect("sender DID len fits u32");
    m.extend_from_slice(&sd_len.to_be_bytes());
    m.extend_from_slice(b.sender_did);
    // body_cid — self-delimiting self-describing CIDv1 (no external lp).
    m.extend_from_slice(b.body_cid);
    // lp_u32(audience_commitment) — injective framing (the value is a fixed
    // 32-B commitment for the group bands + a variable DID for 0x6510).
    let ac_len = u32::try_from(b.audience_commitment.len()).expect("audience commitment fits u32");
    m.extend_from_slice(&ac_len.to_be_bytes());
    m.extend_from_slice(b.audience_commitment);
    // generations — count-prefixed (injective) BE-u32 words (F-3).
    let gen_count = u32::try_from(b.generations.len()).expect("generation count fits u32");
    m.extend_from_slice(&gen_count.to_be_bytes());
    for g in b.generations {
        m.extend_from_slice(&g.to_be_bytes());
    }
    m.extend_from_slice(&b.stanza_count.to_be_bytes());
    m.extend_from_slice(&b.body_aad_digest);
    m
}

/// Sign `M_auth` with the sender's hybrid keypair (default LAMPS hybrid
/// `0x0001`). **F-1: plain `sign` (empty LAMPS ctx); domain separation is
/// the `SENDER_AUTH_DOMAIN` prefix INSIDE `M_auth`.** ML-DSA-first wire per
/// [`benten_crypto_suite::sig::HybridSignature::to_wire_bytes`]. Returns the
/// opaque `sender_sig` bytes carried inside the once-sealed body region.
fn sign_m_auth(sender_kp: &benten_crypto_suite::sig::Keypair, m_auth: &[u8]) -> Vec<u8> {
    let suite = benten_crypto_suite::sig::SignatureSuite::v1_default();
    suite.sign(sender_kp, m_auth).to_wire_bytes()
}

/// Verify `sender_sig` over `M_auth` against the hybrid verifying key
/// resolved from the recovered `sender_did`. Fail-closed → `Err(())` on ANY
/// mismatch (the caller maps `Err(())` to its typed `SenderOriginAuthFailed`).
/// NEVER accepts on a parse alone — both LAMPS halves must verify.
///
/// `sender_did` is the recovered sealed sender-DID bytes; it MUST be a valid
/// UTF-8 hybrid `did:key` string (every legitimate sender's DID is). The
/// resolve + verify both fail-close.
fn verify_m_auth(
    sig_codepoint: u16,
    sender_did: &[u8],
    m_auth: &[u8],
    sender_sig: &[u8],
) -> Result<(), ()> {
    // The default + only v1 sender-auth suite is the hybrid 0x0001; a
    // downgraded/unknown codepoint inside the sealed region is bound into
    // M_auth (so it cannot be silently swapped) and is fail-closed here.
    if sig_codepoint != SENDER_AUTH_SIG_CODEPOINT {
        return Err(());
    }
    // The sealed sender-DID must be a valid `did:key` OR `did:benten` (UTF-8).
    // Resolve the signing key via the METHOD/multicodec-aware
    // `resolve_signing` (DROP-8 / design §2 Tier-1): a hybrid `did:key`
    // resolves to the composite key exactly as before, and a `did:benten`
    // sender's embedded composite signing key resolves after stripping the
    // trailing committed key-set CID — NO doc, zero-I/O. Fail-closed on any
    // malformed / unknown-multicodec DID. (Verifying only the Ed25519 half of
    // a composite would be a silent PQ-strip on the origin-auth path — the
    // whole-composite verify below defeats it.)
    let did_str = core::str::from_utf8(sender_did).map_err(|_| ())?;
    let did = benten_id::did::Did::parse_validated_signing(did_str).map_err(|_| ())?;
    let vk = did.resolve_signing().map_err(|_| ())?;
    // Reconstruct the LAMPS composite signature from the ML-DSA-first wire
    // and cryptographically verify BOTH halves over M_auth (empty LAMPS ctx;
    // domain separation is the SENDER_AUTH_DOMAIN prefix in M_auth — F-1).
    let sig = benten_crypto_suite::sig::HybridSignature::from_lamps_composite_wire(sender_sig)
        .map_err(|_| ())?;
    let suite = benten_crypto_suite::sig::SignatureSuite::v1_default();
    suite.verify(vk, m_auth, &sig).map_err(|_| ())
}

// ---------------------------------------------------------------------------
// Wire constants (§4.0 / §4.1) — all wire-locked.
// ---------------------------------------------------------------------------

/// Envelope SERIALIZATION-format version (M-18/M-19/M-20). V2 from commit 1.
/// DISTINCT from the AAD prefix byte ([`AAD_VERSION`]) — R0.7 §4.1 freezes a
/// dedicated `aad_version: u8` axis separate from the format byte.
pub const ENVELOPE_FORMAT_VERSION: u8 = 2;

/// The frozen AAD version prefix byte (R0.7 §4.1) — AAD byte-0. Mirrors the
/// MembershipSet `0x6610` AAD + the Layer-C siblings so every engine freezes
/// the SAME leading AAD byte for the identical §4.1 prefix. NEVER overload
/// the [`ENVELOPE_FORMAT_VERSION`] (`0x02`) as the AAD prefix.
pub const AAD_VERSION: u8 = 0x01;

/// X25519⊕ML-KEM-768 hybrid KEM codepoint (real X-Wing SHA3-256 combiner;
/// ChaCha20-Poly1305 bulk). §4.0.
pub const HYBRID_X25519_MLKEM768: u16 = 0x647a;

/// `LAYER_C_DROP` — plaintext-sender drop (NON-default; sender-DID on wire).
pub const LAYER_C_DROP: u16 = 0x6500;

/// `DROP_TO_RECIPIENT_SEALED_SENDER` — the v1-beta DEFAULT (BR-1).
pub const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = 0x6510;

/// `LAYER_C_DROP_MULTI_RECIPIENT` — `HpkeMultiBase` group multi-stanza.
pub const LAYER_C_DROP_MULTI_RECIPIENT: u16 = 0x6520;

/// The wire-frozen maximum recipient-roster cardinality for a `0x6520`
/// Layer-C group send: the per-stanza AAD encodes `recipient_count` as a
/// `u16` BE field (§4.0 by-band width; the MembershipSet `0x6610` band uses
/// `u32` — unifying the widths is a rejected freeze record). A caller MUST
/// keep a single-send roster at or below this; a larger group must split
/// into multiple sends. Enforced with a typed error at the seal entry
/// (never a panic) — R12 F-11.
pub const MAX_LAYER_C_GROUP_RECIPIENTS: usize = u16::MAX as usize; // 65_535

// ---------------------------------------------------------------------------
// Recipient key types (the REAL X25519⊕ML-KEM-768 hybrid key material).
// ---------------------------------------------------------------------------

// R9 GAP-1: the frozen encrypt-to-recipient surface takes the REAL hybrid
// recipient key types from [`benten_crypto_suite`], NOT a `[u8; 32]`
// placeholder fingerprint. The seal path wraps the CEK to the recipient's
// real public key; the open path unwraps it with the recipient's real secret
// (which carries genuine entropy — the deleted placeholder derived the
// "secret" from the public via `pk + 0x80`, so anyone with the public key
// could decrypt). This module stays pure crypto glue (no DID knowledge) and
// exposes the real types coherently on its public surface.
pub use benten_crypto_suite::cipher_suite::{RecipientPublic, RecipientSecret};

// ---------------------------------------------------------------------------
// GAP-KDB Shape-B seal typestate — `RecipientBinding` (design §5, Inv-23).
// ---------------------------------------------------------------------------

/// A recipient whose KEM key is **PROVEN committed by its audience DID**
/// (design §5 — the freeze-surface analogue of Inv-15, minted as **Inv-23**:
/// *"a Layer-C seal's KEM key is committed by its audience DID"*).
///
/// This is the GAP-KDB closure **by construction**. The pre-Shape-B seal API
/// took the recipient KEM key and the audience DID as
/// **two independently-chosen params** — nothing bound the KEM key to the
/// audience DID, so an active attacker who substituted `recipient_pub` at the
/// address-book boundary read everything (the "Recipient-key premise" the
/// confidentiality proofs silently assumed). `RecipientBinding` collapses the
/// two into ONE value whose `kem_pub` is recovered-and-verified from the
/// audience DID's content-addressed key-set commitment via
/// [`Did::resolve_kem`] — a BLAKE3-256 CID 2nd-preimage. A KEM key not
/// committed by the DID **cannot be bound**, so no seal can ever target it.
///
/// **Sole-constructor discipline (design C4).** [`RecipientBinding::resolve`]
/// is the ONLY constructor — there is NO `pub` field, NO `From`, NO
/// `_for_test`/`unchecked` escape hatch, and NO `(kem_pub, did)` fallback
/// door. A fallback door is exactly what re-opens the GAP-KDB substitution, so
/// its absence is load-bearing (the anti-downgrade property).
///
/// No `Debug` derive: [`RecipientPublic`] carries no `Debug` impl (key
/// material stays out of any `Debug` sink).
pub struct RecipientBinding {
    audience_did: Did,
    kem_pub: RecipientPublic,
}

impl RecipientBinding {
    /// The **ONLY** constructor (design C4). Fail-closed typed-reject on ANY
    /// commitment mismatch — recovers + VERIFIES the recipient KEM key from
    /// `audience_did`'s content-addressed key-set commitment via
    /// [`Did::resolve_kem`] (CID 2nd-preimage · `doc.sig == embedded-signing`
    /// cross-check · PQ floor `0x647a` · `kem_cp` ⟺ components). A bare
    /// `did:key` commits NO KEM key → rejects. An attacker-substituted key-set
    /// hashes to a DIFFERENT CID → rejects (Inv-23 fires).
    ///
    /// # Errors
    ///
    /// [`RecipientBindingError::Uncommitted`] carrying the originating
    /// [`DidError`](benten_id::errors::DidError) when the KEM key is not
    /// committed by `audience_did` (bare `did:key` / CID 2nd-preimage mismatch
    /// / embedded-signing splice / below-PQ-floor / malformed kem multikey).
    pub fn resolve(
        audience_did: &Did,
        keyset_doc: &KeySetDocument,
    ) -> Result<Self, RecipientBindingError> {
        let kem_pub = audience_did
            .resolve_kem(keyset_doc)
            .map_err(RecipientBindingError::Uncommitted)?;
        Ok(Self {
            audience_did: audience_did.clone(),
            kem_pub,
        })
    }

    /// The bound audience DID (the identity the seal binds into the AAD).
    #[must_use]
    pub fn audience_did(&self) -> &Did {
        &self.audience_did
    }

    /// The committed KEM key recovered from the audience DID's key-set.
    #[must_use]
    pub fn kem_pub(&self) -> &RecipientPublic {
        &self.kem_pub
    }

    /// The audience DID as the length-prefixed AAD `audience` bytes.
    fn audience_bytes(&self) -> Vec<u8> {
        self.audience_did.as_str().as_bytes().to_vec()
    }
}

// `RecipientPublic` has no `Clone` derive; reconstruct through the frozen
// `to_bytes`/`from_bytes` surface (the same X25519-first layout `resolve`
// recovered, always the `0x647a` PQ-floor suite). Used by test rosters that
// replicate a binding (e.g. the over-band cardinality guard).
impl Clone for RecipientBinding {
    fn clone(&self) -> Self {
        let kem_pub = RecipientPublic::from_bytes(
            CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
            &self.kem_pub.to_bytes(),
        )
        .expect("a bound KEM key re-parses through the frozen RecipientPublic surface");
        Self {
            audience_did: self.audience_did.clone(),
            kem_pub,
        }
    }
}

/// Typed failure of the [`RecipientBinding::resolve`] sole constructor.
///
/// `#[non_exhaustive]` (§11 SemVer-readiness). This is a **benten-drop-internal**
/// error (like [`LayerCError`]); the boundary-crossing catalog `ErrorCode`
/// mirror (`E_RECIPIENT_KEM_NOT_COMMITTED`) is minted at the engine surface, not
/// here (§3.5g precedent: `LayerCError` / `DidError` carry no napi/wire mirror).
///
/// Derives `Debug, PartialEq, Eq` (NOT `Clone`) to match the wrapped
/// [`DidError`](benten_id::errors::DidError), which is not `Clone`.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecipientBindingError {
    /// The audience DID does not commit the supplied key-set's KEM key — the
    /// fail-closed [`Did::resolve_kem`] reject (Inv-23): a bare `did:key`
    /// (`NoKemCommitment`), a CID 2nd-preimage mismatch (GAP-KDB active
    /// substitution), an embedded-signing ⊕ KEM splice, a below-PQ-floor
    /// `0x6400` suite (C5), or a malformed kem multikey. Carries the
    /// originating [`DidError`](benten_id::errors::DidError).
    Uncommitted(benten_id::errors::DidError), // drift-detect: internal-only — layer_c-internal; no napi/wire ErrorCode boundary (§3.5g precedent: SenderOriginAuthFailed / DidError; catalog E_RECIPIENT_KEM_NOT_COMMITTED minted at the engine surface).
}

/// **Test-only (design C4-safe):** mint an HONEST [`RecipientBinding`]
/// committing `recipient_pub` under a fresh self-committed `did:benten`. It
/// goes through the SOLE `resolve` constructor over a key-set doc the DID
/// commits — so it is NOT a fallback door (no arbitrary `(kem_pub, did)` pair;
/// the DID content-addresses this exact doc). Used by the migrated Layer-C
/// round-trip / attack tests (whose recipient identities are honest — the
/// attacks live on the OPEN side) to obtain a binding for a real recipient
/// keypair. Uses only the PRODUCTION `benten-id` surface + the registered
/// multicodec constants, so it compiles under both plain `cargo test` and the
/// `testing` feature.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn binding_for_test(recipient_pub: &RecipientPublic) -> RecipientBinding {
    use benten_crypto_suite::cipher_suite::X25519_PUBLIC_LEN;
    use benten_id::did::{
        ED25519_MULTICODEC, MLDSA65_PUB_MULTICODEC, MLKEM768_PUB_MULTICODEC, X25519_PUB_MULTICODEC,
    };

    // Signing multikey (ML-DSA-first) from a fresh hybrid signing key — the
    // exact `sig` bytes `from_benten_keyset` embeds (so `resolve_kem`'s
    // `doc.sig == embedded-signing` cross-check passes).
    let sig_kp = benten_crypto_suite::sig::SignatureSuite::v1_default().generate_keypair();
    let composite = sig_kp
        .public()
        .to_lamps_composite_bytes()
        .expect("v1_default keypair is hybrid");
    let mldsa_len = benten_crypto_suite::sizes::ml_dsa_65_pubkey_len();
    let (mldsa_pk, ed_pk) = composite.split_at(mldsa_len);
    let mut sig_mk = Vec::with_capacity(2 + mldsa_pk.len() + 2 + ed_pk.len());
    sig_mk.extend_from_slice(&MLDSA65_PUB_MULTICODEC);
    sig_mk.extend_from_slice(mldsa_pk);
    sig_mk.extend_from_slice(&ED25519_MULTICODEC);
    sig_mk.extend_from_slice(ed_pk);

    // KEM multikey (X25519-first, C2) from the recipient's REAL public bytes —
    // `RecipientPublic::to_bytes()` is `x25519(32) ‖ mlkem768_ek(1184)`, the
    // exact order the frozen `kem` multikey uses, so `resolve_kem` recovers a
    // KEM key byte-identical to `recipient_pub`.
    let raw = recipient_pub.to_bytes();
    let (x, ek) = raw.split_at(X25519_PUBLIC_LEN);
    let mut kem_mk = Vec::with_capacity(2 + x.len() + 2 + ek.len());
    kem_mk.extend_from_slice(&X25519_PUB_MULTICODEC);
    kem_mk.extend_from_slice(x);
    kem_mk.extend_from_slice(&MLKEM768_PUB_MULTICODEC);
    kem_mk.extend_from_slice(ek);

    let doc = KeySetDocument::v1_hybrid(sig_mk, kem_mk);
    let did = Did::from_benten_keyset(&sig_kp.public(), &doc);
    RecipientBinding::resolve(&did, &doc)
        .expect("a self-committed key-set MUST bind (honest recipient)")
}

/// **Test-only:** a roster of HONEST committed [`RecipientBinding`]s, one per
/// recipient public (each via [`binding_for_test`]). The seal binds these
/// exact identities (C9); an honest open recomputes the commitment from the
/// SAME roster via [`binding_roster_for_test`] / [`member_dids_for_test`].
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn group_bindings_for_test(recipient_pubs: &[RecipientPublic]) -> Vec<RecipientBinding> {
    recipient_pubs.iter().map(binding_for_test).collect()
}

/// **Test-only:** the independently-held `0x6520` roster (REAL committed
/// audience DID bytes) an honest `open_group_stanza` recipient recomputes the
/// C9 `audience_set_commitment` from — the SAME identities the seal bound.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn binding_roster_for_test(bindings: &[RecipientBinding]) -> Vec<RecipientDid> {
    bindings
        .iter()
        .map(RecipientBinding::audience_bytes)
        .collect()
}

/// **Test-only:** the `0x6610` member-DID roster (`Vec<String>` for
/// `GroupVerifyContext`) — the REAL committed audience DID strings (C9).
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn member_dids_for_test(
    bindings: &[RecipientBinding],
) -> alloc::vec::Vec<alloc::string::String> {
    bindings
        .iter()
        .map(|b| b.audience_did().as_str().to_owned())
        .collect()
}

/// A sender DID (`did:key` multibase string in production), as raw bytes.
pub type SenderDid = Vec<u8>;
/// A recipient DID (an element of the BLINDED roster bound via the
/// [`audience_set_commitment`] on the group wire — never published raw).
pub type RecipientDid = Vec<u8>;
/// An audience DID (the recipient-targeting identity bound in the
/// single-recipient drop AAD), as raw bytes.
pub type AudienceDid = Vec<u8>;
/// A 32-byte content-CID DIGEST (the BLAKE3 of the body). The AADs bind the
/// self-describing CIDv1 form of this digest, NOT the bare digest.
pub type BodyCidDigest = [u8; 32];
/// A self-describing CIDv1 (`0x01 0x71 0x1e 0x20 ‖ 32-byte BLAKE3` = 36 B).
pub type SelfDescribingCid = Vec<u8>;

/// Wrap a 32-byte body-CID DIGEST into its self-describing CIDv1 form
/// (`0x01 0x71 0x1e 0x20 ‖ digest`). Bound into the `0x6500`/`0x6510`/`0x6520`
/// AADs per CLAUDE.md baked-in #5 (restores U3 length-injectivity).
#[must_use]
pub fn self_describing_cid(digest: &BodyCidDigest) -> SelfDescribingCid {
    let mut cid = Vec::with_capacity(4 + 32);
    cid.extend_from_slice(&[0x01u8, 0x71, 0x1e, 0x20]);
    cid.extend_from_slice(digest);
    cid
}

/// The BLINDED `audience_set_commitment` (32 B):
/// `BLAKE3(0x01 ‖ lp_u32(did_0) ‖ lp_u32(did_1) ‖ …)` over the CANONICAL
/// SORTED recipient-DID list — the IDENTICAL construction to the `0x6610`
/// MembershipSet commitment (`lp = u32-BE` length prefix). The recipients
/// hold the roster and recompute + verify; the relay sees only the opaque
/// 32-byte tag (closes the #61-class raw-roster leak). The lp width is
/// `u32-BE`, NOT the band's u16 `recipient_count` cardinality.
#[must_use]
pub fn audience_set_commitment(recipient_dids: &[RecipientDid]) -> [u8; 32] {
    let mut sorted: Vec<&RecipientDid> = recipient_dids.iter().collect();
    sorted.sort();
    let mut h = blake3::Hasher::new();
    h.update(&[0x01u8]); // domain-separation prefix (matches 0x6610)
    for did in sorted {
        let len = u32::try_from(did.len()).expect("recipient DID len fits u32");
        h.update(&len.to_be_bytes());
        h.update(did);
    }
    *h.finalize().as_bytes()
}

// ---------------------------------------------------------------------------
// BindingContext — single-recipient drop AAD.
// ---------------------------------------------------------------------------

/// The typed single-recipient drop binding.
///
/// Both variants bind the canonical `0x65xx` envelope-AAD prefix
/// `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`.
/// The plaintext-sender variant ADDS the sender-DID (U4); the Sealed-Sender
/// variant does NOT (it lives inside the ciphertext). NEITHER carries a
/// timestamp NOR coarse-epoch (M-14 / F4-006 — drops are forever-valid #62;
/// freshness rides recipient-key-generation + the nonce-cache).
///
/// This is the CLOSED two-variant single-recipient drop set (`0x6500` /
/// `0x6510`) — both wire-frozen at v1-beta. The codepoint-dispatched
/// extensibility lives on [`EncryptedEnvelope`] (which IS extensible per
/// Inv-16) + the codepoint registry, not on this binding's variant set.
///
/// **§11 SemVer carve-out — INTENTIONALLY EXHAUSTIVE (NOT `#[non_exhaustive]`).**
/// Each variant is WIRE-KEYING (one codepoint per variant: `0x6500` /
/// `0x6510`), so the variant set is a closed, frozen wire contract — the same
/// exhaustive-by-design posture as `MembershipSetKind` / `RoleId`, and the
/// opposite of the extensible [`EncryptedEnvelope`]. New drop shapes are added on
/// `EncryptedEnvelope` + the codepoint registry, NOT by growing this enum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingContext {
    /// Plaintext-sender drop (`0x6500`, NON-default): sender-DID on the wire.
    DropPlaintextSender {
        /// AAD prefix version (`AAD_VERSION` = 0x01; NOT the format byte).
        aad_version: u8,
        /// Drop codepoint (`0x6500`).
        codepoint: u16,
        /// The recipient-targeting audience DID (§3.3:484).
        audience_did: AudienceDid,
        /// Self-describing CIDv1 over the body digest (36 B).
        body_cid: SelfDescribingCid,
        /// Recipient key-generation (Inv-16; U19).
        recipient_key_generation: u32,
        /// The sender-DID bound into the PLAINTEXT AAD (U4).
        sender_did: SenderDid,
    },
    /// Sealed-Sender drop (`0x6510`, DEFAULT): sender-DID inside ciphertext.
    DropSealedSender {
        /// AAD prefix version (`AAD_VERSION` = 0x01; NOT the format byte).
        aad_version: u8,
        /// Drop codepoint (`0x6510`).
        codepoint: u16,
        /// The recipient-targeting audience DID (§3.3:484).
        audience_did: AudienceDid,
        /// Self-describing CIDv1 over the body digest (36 B).
        body_cid: SelfDescribingCid,
        /// Recipient key-generation (Inv-16; U19).
        recipient_key_generation: u32,
    },
}

impl BindingContext {
    /// The canonical PLAINTEXT single-recipient drop AAD bytes (BE; M-19).
    ///
    /// Layout: `aad_version u8 | codepoint u16 BE | audience_len u32 BE |
    /// audience_did | body_cid (36 B) | recipient_key_gen u32 BE`
    /// `[plaintext-sender only: sender_len u16 BE | sender_did]`.
    #[must_use]
    pub fn plaintext_aad_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            BindingContext::DropSealedSender {
                aad_version,
                codepoint,
                audience_did,
                body_cid,
                recipient_key_generation,
            } => {
                out.push(*aad_version);
                out.extend_from_slice(&codepoint.to_be_bytes());
                push_audience(&mut out, audience_did);
                out.extend_from_slice(body_cid);
                out.extend_from_slice(&recipient_key_generation.to_be_bytes());
            }
            BindingContext::DropPlaintextSender {
                aad_version,
                codepoint,
                audience_did,
                body_cid,
                recipient_key_generation,
                sender_did,
            } => {
                out.push(*aad_version);
                out.extend_from_slice(&codepoint.to_be_bytes());
                push_audience(&mut out, audience_did);
                out.extend_from_slice(body_cid);
                out.extend_from_slice(&recipient_key_generation.to_be_bytes());
                let len = u16::try_from(sender_did.len()).expect("sender DID len fits u16");
                out.extend_from_slice(&len.to_be_bytes());
                out.extend_from_slice(sender_did);
            }
        }
        out
    }
}

/// The `0x6510`/`0x6500` audience length-prefix is `u32-BE` (R0.7 §4.1:1040)
/// — NOT the band's u16 `recipient_count` cardinality.
fn push_audience(out: &mut Vec<u8>, audience_did: &[u8]) {
    let aud_len = u32::try_from(audience_did.len()).expect("audience DID len fits u32");
    out.extend_from_slice(&aud_len.to_be_bytes());
    out.extend_from_slice(audience_did);
}

// ---------------------------------------------------------------------------
// HpkeRecipientStanza — group per-stanza (0x6520), BLINDED AAD.
// ---------------------------------------------------------------------------

/// One recipient stanza of an `HpkeMultiBase` group envelope (`0x6520`),
/// with the R0.7-BLINDED per-stanza AAD.
///
/// The DEFAULT path binds the BLINDED field-set WITHOUT the sender-DID; the
/// inner-sender-DID lives inside `sealed_inner`. The non-default variant
/// sets `plaintext_sender_did = Some(..)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HpkeRecipientStanza {
    /// Group codepoint (`0x6520`).
    pub codepoint: u16,
    /// Self-describing CIDv1 over the body digest (36 B).
    pub body_cid: SelfDescribingCid,
    /// The recipient-DID roster — the COMMITMENT INPUT for the BLINDED
    /// [`audience_set_commitment`]. NEVER emitted raw into the AAD.
    pub recipient_dids: Vec<RecipientDid>,
    /// This stanza's position (bound for substitution defense).
    pub stanza_index: u32,
    /// The TOTAL stanza count (truncation/censorship defense; R0.7 §3.3).
    pub stanza_count: u32,
    /// Recipient key-generation (Inv-16; U19).
    pub recipient_key_generation: u32,
    /// The DEFAULT (Sealed-Sender) sealed payload — the inner-sender-DID +
    /// wrapped CEK live here, recovered only post-decrypt.
    pub sealed_inner: Vec<u8>,
    /// NON-default plaintext-sender variant ONLY: the sender-DID bound into
    /// the PLAINTEXT AAD (U4). `None` on the DEFAULT path.
    pub plaintext_sender_did: Option<SenderDid>,
    /// HPKE-wrapped content-encryption-key for THIS recipient.
    pub wrapped_cek: Vec<u8>,
}

impl HpkeRecipientStanza {
    /// The canonical PLAINTEXT per-stanza AAD bytes (BE; M-19). BLINDED:
    /// the raw roster is NEVER emitted — only the `audience_set_commitment`
    /// + `recipient_count` are on the wire.
    ///
    /// Layout: `aad_version u8 | codepoint u16 BE | body_cid (36 B) |
    /// recipient_count u16 BE | audience_set_commitment (32 B) |
    /// stanza_index u32 BE | stanza_count u32 BE | recipient_key_gen u32 BE`
    /// `[non-default: sender_len u16 BE | sender_did]`.
    #[must_use]
    pub fn plaintext_aad_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(AAD_VERSION);
        out.extend_from_slice(&self.codepoint.to_be_bytes());
        out.extend_from_slice(&self.body_cid);
        let count = u16::try_from(self.recipient_dids.len()).expect(
            "recipient count within MAX_LAYER_C_GROUP_RECIPIENTS — guaranteed by \
             validate_group_roster_len at the seal entry",
        );
        out.extend_from_slice(&count.to_be_bytes());
        out.extend_from_slice(&audience_set_commitment(&self.recipient_dids));
        out.extend_from_slice(&self.stanza_index.to_be_bytes());
        out.extend_from_slice(&self.stanza_count.to_be_bytes());
        out.extend_from_slice(&self.recipient_key_generation.to_be_bytes());
        if let Some(sender) = &self.plaintext_sender_did {
            let len = u16::try_from(sender.len()).expect("sender DID len fits u16");
            out.extend_from_slice(&len.to_be_bytes());
            out.extend_from_slice(sender);
        }
        out
    }
}

// ---------------------------------------------------------------------------
// EncryptedEnvelope (Inv-16) — codepoint-dispatched.
// ---------------------------------------------------------------------------

/// The codepoint-dispatched `EncryptedEnvelope` (Inv-16).
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): the envelope is Inv-16-extensible
/// — a future codepoint-dispatched shape (a new HPKE mode, a CGKA group object)
/// lands as an ADDITIVE variant without a breaking SemVer bump on the frozen v1
/// API; cross-crate consumers MUST fail-CLOSED on an unrecognized shape (reject,
/// never silently dispatch). Contrast the sibling [`BindingContext`], whose
/// two-variant set IS wire-keying (`0x6500`/`0x6510`) and is deliberately
/// exhaustive-by-design.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EncryptedEnvelope {
    /// Single-recipient HPKE `mode_base` (`0x647A` KEM) carrying a
    /// `0x6500`/`0x6510` drop binding.
    HpkeBase {
        /// Envelope serialization-format version (= [`ENVELOPE_FORMAT_VERSION`]).
        format_version: u8,
        /// The drop-variant binding (the plaintext AAD source).
        binding: BindingContext,
        /// HPKE encapsulated key material (`enc`) — the wrapped CEK bytes.
        enc: Vec<u8>,
        /// ChaCha20-Poly1305 ciphertext+tag of `(inner_sender_did ‖ body)`.
        ciphertext: Vec<u8>,
    },
    /// Group multi-stanza (`0x6520`).
    HpkeMultiBase {
        /// Envelope serialization-format version.
        format_version: u8,
        /// The bulk body ciphertext (sealed once under the shared CEK).
        cek_aead_ciphertext: Vec<u8>,
        /// The bulk body nonce (12 B ChaCha20-Poly1305).
        cek_aead_nonce: [u8; 12],
        /// One stanza per recipient.
        stanzas: Vec<HpkeRecipientStanza>,
    },
}

// ---------------------------------------------------------------------------
// Errors.
// ---------------------------------------------------------------------------

/// Typed Layer-C failure modes.
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future failure mode lands as
/// an additive variant without a breaking change for downstream `match` sites.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LayerCError {
    /// AEAD authentication failed (wrong key, tampered AAD, stanza
    /// substitution/reorder/re-target, wrong recipient sk).
    AeadAuthenticationFailed,
    /// The recovered inner payload was malformed (a length-prefix overran
    /// the decrypted buffer / a truncated inner_v2 framing). NOT a forgery
    /// (a wrong-signer forgery surfaces [`LayerCError::SenderOriginAuthFailed`]
    /// at the post-decrypt verify); this is a structural decode failure of the
    /// already-AEAD-opened plaintext. Named for what it IS — a malformed inner
    /// payload — NOT `InnerSenderDidForged` (R15 F-09 rename: the old name was
    /// self-contradictory with this docstring's own "NOT a forgery" clause;
    /// forgery is `SenderOriginAuthFailed`). Reachable only AFTER a successful
    /// AEAD-open, so it is not a pre-decrypt oracle.
    MalformedInnerPayload, // drift-detect: internal-only — layer_c-internal; no napi/wire ErrorCode boundary (§3.5g precedent: SenderOriginAuthFailed / StanzaCountMismatch / DidError).
    /// **B2 Sealed-Sender ORIGIN-AUTH (post-decrypt verify) FAILED** — the
    /// envelope AEAD-opened cleanly (so a co-recipient / any party able to
    /// derive the CEK CAN produce a valid AEAD tag), but the per-message
    /// LAMPS-hybrid signature over `M_auth` did NOT verify against the
    /// hybrid verifying key resolved from the recovered sender-DID. This is
    /// the defense the AEAD tag structurally cannot provide: a
    /// validly-sealed-but-WRONG-SIGNER (impersonation / re-target / suite-
    /// downgrade / stripped-PQ-half) envelope. Fail-closed; NEVER accepted on
    /// a parse alone (design §1.4; `f_lc_3` substantive pins).
    SenderOriginAuthFailed, // drift-detect: internal-only — layer_c-internal; no napi/wire ErrorCode boundary (§3.5g precedent: StanzaCountMismatch / DidError).
    /// Codepoint dispatch hit an unknown/reserved arm.
    UnsupportedCodepoint(u16),
    /// The `0x6520` group recipient roster exceeds the band's wire-frozen
    /// `u16` cardinality limit ([`MAX_LAYER_C_GROUP_RECIPIENTS`]). Split the
    /// send into multiple groups.
    RecipientCountExceedsBandWidth {
        /// The recipient count the caller requested for this single send.
        count: usize,
        /// The wire-frozen ceiling ([`MAX_LAYER_C_GROUP_RECIPIENTS`]).
        max: usize,
    },
    /// A group seal was handed an EMPTY `&[RecipientBinding]` roster (design §5
    /// / DROP-10 — the C9 single-slice roster's missing LOWER bound). A group
    /// envelope with zero recipients is degenerate (no one can open it); the
    /// seal typed-rejects rather than emit a zero-stanza envelope.
    EmptyRecipientRoster, // drift-detect: internal-only — layer_c-internal; no napi/wire ErrorCode boundary (§3.5g precedent: SenderOriginAuthFailed / StanzaCountMismatch / DidError).
    /// The number of stanzas actually DELIVERED does not equal the
    /// `stanza_count` bound into every stanza's AAD — a relay dropped /
    /// censored / truncated stanzas (SECURITY-PROOFS §3.3/§4.1
    /// truncation-defense; fail-closed; F-01). `delivered` is what arrived;
    /// `bound` is the count every survivor names.
    StanzaCountMismatch {
        /// The number of stanzas actually present in the envelope.
        delivered: u32,
        /// The `stanza_count` the surviving stanzas were sealed against.
        bound: u32,
    },
}

// ---------------------------------------------------------------------------
// Internal crypto glue (CLAUDE.md baked-in #5 — routes via crypto-suite).
// ---------------------------------------------------------------------------

/// The X-Wing hybrid suite this layer always uses (`0x647a`). Resolution is
/// infallible for the wire-locked default; a `resolve` failure would be a
/// programming error (the codepoint is a const).
fn hybrid_suite() -> CipherSuite {
    CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("HYBRID_X25519_MLKEM768 (0x647a) is the wire-locked Layer-C default")
}

/// Encode a [`WrappedKey`] to the opaque `enc` bytes carried on the wire:
/// `ek_x_len u32 BE ‖ ek_x ‖ ek_mlkem_len u32 BE ‖ ek_mlkem ‖
/// aead_envelope.to_wire_bytes()`. BE per M-19.
fn encode_wrapped_key(w: &WrappedKey) -> Vec<u8> {
    let mut out = Vec::new();
    let ek_x_len = u32::try_from(w.ek_x.len()).expect("ek_x len fits u32");
    out.extend_from_slice(&ek_x_len.to_be_bytes());
    out.extend_from_slice(&w.ek_x);
    let ek_mlkem_len = u32::try_from(w.ek_mlkem.len()).expect("ek_mlkem len fits u32");
    out.extend_from_slice(&ek_mlkem_len.to_be_bytes());
    out.extend_from_slice(&w.ek_mlkem);
    out.extend_from_slice(&w.aead_envelope.to_wire_bytes());
    out
}

/// Decode the opaque `enc` bytes back into a [`WrappedKey`] (inverse of
/// [`encode_wrapped_key`]). Returns `None` on any malformed framing.
fn decode_wrapped_key(bytes: &[u8]) -> Option<WrappedKey> {
    let mut off = 0usize;
    let take_u32 = |bytes: &[u8], off: &mut usize| -> Option<usize> {
        if *off + 4 > bytes.len() {
            return None;
        }
        let v = u32::from_be_bytes([
            bytes[*off],
            bytes[*off + 1],
            bytes[*off + 2],
            bytes[*off + 3],
        ]);
        *off += 4;
        usize::try_from(v).ok()
    };
    let ek_x_len = take_u32(bytes, &mut off)?;
    let ek_x_end = lp_range_end(off, ek_x_len, bytes.len())?;
    let ek_x = bytes[off..ek_x_end].to_vec();
    off = ek_x_end;
    let ek_mlkem_len = take_u32(bytes, &mut off)?;
    let ek_mlkem_end = lp_range_end(off, ek_mlkem_len, bytes.len())?;
    let ek_mlkem = bytes[off..ek_mlkem_end].to_vec();
    off = ek_mlkem_end;
    let aead_envelope = AeadEnvelope::from_wire_bytes(&bytes[off..]).ok()?;
    Some(WrappedKey {
        codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        ek_x,
        ek_mlkem,
        aead_envelope,
    })
}

/// Seal the `0x6510` inner payload under a fresh CEK with the given AAD;
/// HPKE-wrap the CEK to the recipient's REAL hybrid public key. The inner
/// payload carries the ORIGIN-AUTH signature in the once-sealed region (B2):
/// `inner_v2 = lp_u32(sender_did) ‖ sig_codepoint(u16 BE) ‖ lp_u32(sender_sig) ‖ body`.
/// `M_auth` binds the `audience_did` the sender is sending TO (the recipient
/// re-derives it from its OWN audience — F-2). Returns `(enc, ciphertext)`.
// The B2 origin-auth binding genuinely needs all of {recipient_pub, sender_did,
// sender_kp, envelope_codepoint, audience_did, body_cid, recipient_key_generation,
// aad, body}; bundling them into a params struct would obscure the seal flow's
// 1:1 correspondence with M_auth's fields. Internal (crate-private) helper.
#[allow(clippy::too_many_arguments)]
fn seal_inner(
    recipient_pub: &RecipientPublic,
    sender_did: &SenderDid,
    sender_kp: &benten_crypto_suite::sig::Keypair,
    envelope_codepoint: u16,
    audience_did: &[u8],
    body_cid: &[u8],
    recipient_key_generation: u32,
    aad: &[u8],
    body: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    let suite = hybrid_suite();
    // The single-recipient (0x6510) CEK is DETERMINISTICALLY derived — a BLAKE3
    // hash of (recipient public key ‖ sender ‖ AAD ‖ body) under
    // LAYER_C_CEK_CONTEXT — NOT a fresh-random value (that is the 0x6520 group
    // band; see SECURITY-PROOFS §4.2 for the per-band CEK constructions and the
    // deterministic-CEK confirmation-oracle disclosure it carries). It is still
    // HPKE-wrapped (the relay never sees it; the open side recovers it by
    // HPKE-unwrap, NEVER by re-hashing — the CEK derivation is a seal-local
    // source, not a round-trip contract). Being deterministic per (recipient,
    // send, body), it gives a confirmation oracle only to a party that ALREADY
    // holds the CEK — documented, not a confidentiality break.
    let mut cek_h = blake3::Hasher::new();
    cek_h.update(LAYER_C_CEK_CONTEXT);
    cek_h.update(&recipient_pub.to_bytes());
    cek_h.update(sender_did);
    cek_h.update(aad);
    cek_h.update(body);
    let cek = *cek_h.finalize().as_bytes();

    // B2 ORIGIN-AUTH: compute M_auth + sign ONCE per message. The single
    // recipient send binds the recipient's audience-DID as the audience
    // commitment, the recipient_key_generation as the single generation word,
    // and stanza_count = 1.
    let body_aad_digest = *blake3::hash(aad).as_bytes();
    let generations = [recipient_key_generation];
    let m_auth = build_m_auth(&SenderAuthBinding {
        sig_codepoint: SENDER_AUTH_SIG_CODEPOINT,
        envelope_codepoint,
        sender_did,
        body_cid,
        audience_commitment: audience_did,
        generations: &generations,
        stanza_count: 1,
        body_aad_digest,
    });
    let sender_sig = sign_m_auth(sender_kp, &m_auth);

    // Bulk-seal the inner payload (inner_v2) under the CEK, binding the
    // plaintext AAD. The sender-DID + sig_codepoint + sig + body are sealed
    // together in the ONCE-sealed region.
    let mut inner = Vec::new();
    let sd_len = u32::try_from(sender_did.len()).expect("sender DID len fits u32");
    inner.extend_from_slice(&sd_len.to_be_bytes());
    inner.extend_from_slice(sender_did);
    inner.extend_from_slice(&SENDER_AUTH_SIG_CODEPOINT.to_be_bytes());
    let sig_len = u32::try_from(sender_sig.len()).expect("sender sig len fits u32");
    inner.extend_from_slice(&sig_len.to_be_bytes());
    inner.extend_from_slice(&sender_sig);
    inner.extend_from_slice(body);
    let cek_key =
        AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, &cek);
    let body_env = benten_crypto_suite::aead::wrap(&inner, &cek_key, aad)
        .expect("ChaCha20-Poly1305 seal of the Layer-C inner payload must succeed");
    let ciphertext = body_env.to_wire_bytes();

    // HPKE-wrap the CEK to the recipient's REAL public key (R9 GAP-1).
    let wrapped = suite
        .wrap_key_material(recipient_pub, &cek)
        .expect("X-Wing wrap of the Layer-C CEK must succeed");
    let enc = encode_wrapped_key(&wrapped);
    (enc, ciphertext)
}

/// Recover `(body, inner_sender_did)` from `(enc, ciphertext)` under the
/// recipient's REAL hybrid secret key (R9 GAP-1 — NOT a `[u8; 32]`
/// fingerprint; the deleted placeholder derived the "secret" from the public
/// via `pk + 0x80`) + the plaintext AAD, then VERIFY the B2
/// ORIGIN-AUTH signature. `recipient_audience_did` is the recipient's OWN
/// audience DID — the recipient re-derives the audience commitment from it,
/// NEVER the wire `audience_did` (F-2). `recipient_key_generation` is the
/// recipient's independently-held key epoch.
// Mirrors seal_inner's input set for the post-decrypt M_auth re-derivation
// (F-2); internal (crate-private) helper. See seal_inner's note.
#[allow(clippy::too_many_arguments)]
fn open_inner(
    recipient_sec: &RecipientSecret,
    enc: &[u8],
    ciphertext: &[u8],
    aad: &[u8],
    envelope_codepoint: u16,
    recipient_audience_did: &[u8],
    body_cid: &[u8],
    recipient_key_generation: u32,
) -> Result<(Vec<u8>, SenderDid), LayerCError> {
    let suite = hybrid_suite();

    // R9 GAP-1: HPKE-unwrap the CEK with the recipient's REAL secret key
    // (genuine entropy). The deleted placeholder reconstructed a "secret" from
    // the PUBLIC fingerprint (`pk = sk - 0x80`) → zero secret entropy → any
    // holder of the public key could decrypt. A secret that does NOT match the
    // public key the sender wrapped to yields a different X-Wing shared secret
    // and the AEAD unwrap fails closed.
    let wrapped = decode_wrapped_key(enc).ok_or(LayerCError::AeadAuthenticationFailed)?;
    let cek = suite
        .unwrap_key_material(recipient_sec, &wrapped)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;

    let cek_key = AeadKeyMaterial::from_raw_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        cek.as_bytes(),
    );
    let body_env = AeadEnvelope::from_wire_bytes(ciphertext)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;
    let inner = benten_crypto_suite::aead::unwrap(&body_env, &cek_key, aad)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;

    // Parse inner_v2 = lp_u32(sender_did) ‖ sig_codepoint(u16) ‖
    // lp_u32(sender_sig) ‖ body.
    let mut off = 0usize;
    if inner.len() < off + 4 {
        return Err(LayerCError::MalformedInnerPayload);
    }
    let sd_len =
        u32::from_be_bytes([inner[off], inner[off + 1], inner[off + 2], inner[off + 3]]) as usize;
    off += 4;
    let sd_end =
        lp_range_end(off, sd_len, inner.len()).ok_or(LayerCError::MalformedInnerPayload)?;
    let sender_did = inner[off..sd_end].to_vec();
    off = sd_end;
    if inner.len() < off + 2 {
        return Err(LayerCError::MalformedInnerPayload);
    }
    let sig_codepoint = u16::from_be_bytes([inner[off], inner[off + 1]]);
    off += 2;
    if inner.len() < off + 4 {
        return Err(LayerCError::MalformedInnerPayload);
    }
    let sig_len =
        u32::from_be_bytes([inner[off], inner[off + 1], inner[off + 2], inner[off + 3]]) as usize;
    off += 4;
    let sig_end =
        lp_range_end(off, sig_len, inner.len()).ok_or(LayerCError::MalformedInnerPayload)?;
    let sender_sig = inner[off..sig_end].to_vec();
    off = sig_end;
    let body = inner[off..].to_vec();

    // B2 CONTENT-SPLICE GUARD (F-01, SOUNDNESS-CRITICAL): M_auth binds the
    // body ONLY through `body_cid`; a co-recipient holding the CEK can re-seal
    // a DIFFERENT body under the VICTIM's real `sender_sig` + the ORIGINAL
    // (unchanged) `body_cid`, and the origin-auth verify below — which consumes
    // the WIRE `body_cid` — would otherwise pass. Recompute the canonical CID
    // from the RECOVERED body (SAME derivation as the seal side:
    // `self_describing_cid(BLAKE3(body))`) and fail-closed unless it is
    // byte-equal to the wire `body_cid`.
    let recomputed_cid = self_describing_cid(blake3::hash(&body).as_bytes());
    if recomputed_cid != body_cid {
        return Err(LayerCError::SenderOriginAuthFailed);
    }

    // B2 ORIGIN-AUTH VERIFY: re-derive M_auth from the recovered sender-DID +
    // the recipient's OWN audience-DID / key-generation (NOT the wire
    // audience — F-2) and cryptographically verify the hybrid signature.
    let body_aad_digest = *blake3::hash(aad).as_bytes();
    let generations = [recipient_key_generation];
    let m_auth = build_m_auth(&SenderAuthBinding {
        sig_codepoint,
        envelope_codepoint,
        sender_did: &sender_did,
        body_cid,
        audience_commitment: recipient_audience_did,
        generations: &generations,
        stanza_count: 1,
        body_aad_digest,
    });
    verify_m_auth(sig_codepoint, &sender_did, &m_auth, &sender_sig)
        .map_err(|()| LayerCError::SenderOriginAuthFailed)?;

    Ok((body, sender_did))
}

// ---------------------------------------------------------------------------
// Single-recipient seal / open.
// ---------------------------------------------------------------------------

/// Single-recipient HPKE-base seal (`0x647A`) under the Sealed-Sender
/// DEFAULT (`0x6510`): the sender-DID is sealed INSIDE the ciphertext; the
/// AAD binds the `audience` + body-CID + recipient_key_generation.
///
/// **B2 ORIGIN-AUTH (always-on, BD-2):** `sender_kp` is the sender's
/// LAMPS-hybrid signing keypair, and `sender_did` MUST be the hybrid `did:key`
/// or `did:benten` that [`benten_id::did::Did::resolve_signing`] resolves to
/// `sender_kp.public()` — the recipient verifies the per-message signature
/// against that resolved key post-decrypt. There is NO unauthenticated
/// single-recipient seal.
///
/// **GAP-KDB Shape-B (design §5 / DROP-3):** the recipient arrives as a
/// [`RecipientBinding`] whose KEM key is PROVEN committed by its audience DID —
/// there is NO two-independent-param door taking a raw recipient KEM key
/// alongside a separately-chosen audience DID. The KEM key + the audience DID
/// are one bound value; an attacker cannot downgrade a `did:benten` recipient
/// back to the un-cross-checked path (that path is deleted).
#[must_use]
pub fn seal_sealed_sender(
    recipient: &RecipientBinding,
    sender_did: &SenderDid,
    sender_kp: &benten_crypto_suite::sig::Keypair,
    body_cid: &BodyCidDigest,
    recipient_key_generation: u32,
    plaintext: &[u8],
) -> EncryptedEnvelope {
    let audience_did = recipient.audience_bytes();
    let cid = self_describing_cid(body_cid);
    let binding = BindingContext::DropSealedSender {
        aad_version: AAD_VERSION,
        codepoint: DROP_TO_RECIPIENT_SEALED_SENDER,
        audience_did: audience_did.clone(),
        body_cid: cid.clone(),
        recipient_key_generation,
    };
    let aad = binding.plaintext_aad_bytes();
    let (enc, ciphertext) = seal_inner(
        recipient.kem_pub(),
        sender_did,
        sender_kp,
        DROP_TO_RECIPIENT_SEALED_SENDER,
        &audience_did,
        &cid,
        recipient_key_generation,
        &aad,
        plaintext,
    );
    EncryptedEnvelope::HpkeBase {
        format_version: ENVELOPE_FORMAT_VERSION,
        binding,
        enc,
        ciphertext,
    }
}

/// Single-recipient HPKE-base seal under the plaintext-sender NON-DEFAULT
/// path (`0x6500`): sender-DID bound INTO the AAD (U4) AND inside the
/// ciphertext (so open still recovers it). Plaintext-sender discloses WHO;
/// it STILL carries the B2 ORIGIN-AUTH signature (it must prove the who) —
/// see `sender_kp` on [`seal_sealed_sender`].
#[must_use]
pub fn seal_plaintext_sender(
    recipient: &RecipientBinding,
    sender_did: &SenderDid,
    sender_kp: &benten_crypto_suite::sig::Keypair,
    body_cid: &BodyCidDigest,
    recipient_key_generation: u32,
    plaintext: &[u8],
) -> EncryptedEnvelope {
    let audience_did = recipient.audience_bytes();
    let cid = self_describing_cid(body_cid);
    let binding = BindingContext::DropPlaintextSender {
        aad_version: AAD_VERSION,
        codepoint: LAYER_C_DROP,
        audience_did: audience_did.clone(),
        body_cid: cid.clone(),
        recipient_key_generation,
        sender_did: sender_did.clone(),
    };
    let aad = binding.plaintext_aad_bytes();
    let (enc, ciphertext) = seal_inner(
        recipient.kem_pub(),
        sender_did,
        sender_kp,
        LAYER_C_DROP,
        &audience_did,
        &cid,
        recipient_key_generation,
        &aad,
        plaintext,
    );
    EncryptedEnvelope::HpkeBase {
        format_version: ENVELOPE_FORMAT_VERSION,
        binding,
        enc,
        ciphertext,
    }
}

/// Open a single-recipient envelope, VERIFYING the B2 ORIGIN-AUTH signature
/// post-decrypt. `recipient_audience_did` is the recipient's OWN audience
/// DID — the B2 audience commitment is re-derived from it, NEVER from the
/// wire `audience_did` (F-2, SOUNDNESS-CRITICAL). `recipient_key_generation`
/// is the recipient's independently-held key epoch. Returns the recovered
/// sender-DID only after BOTH the AEAD unwrap AND the hybrid signature
/// verify succeed.
///
/// # Errors
///
/// Returns [`LayerCError::AeadAuthenticationFailed`] on a wrong recipient
/// secret / tampered ciphertext / tampered AAD,
/// [`LayerCError::MalformedInnerPayload`] on a malformed inner payload, and
/// [`LayerCError::SenderOriginAuthFailed`] when the per-message origin-auth
/// signature does not verify (impersonation / re-target / suite-downgrade /
/// stripped-half).
pub fn open_single(
    recipient_sec: &RecipientSecret,
    recipient_audience_did: &AudienceDid,
    recipient_key_generation: u32,
    env: &EncryptedEnvelope,
) -> Result<(Vec<u8>, SenderDid), LayerCError> {
    match env {
        EncryptedEnvelope::HpkeBase {
            binding,
            enc,
            ciphertext,
            ..
        } => {
            let (envelope_codepoint, body_cid) = match binding {
                BindingContext::DropSealedSender {
                    codepoint,
                    body_cid,
                    ..
                }
                | BindingContext::DropPlaintextSender {
                    codepoint,
                    body_cid,
                    ..
                } => (*codepoint, body_cid.clone()),
            };
            let aad = binding.plaintext_aad_bytes();
            open_inner(
                recipient_sec,
                enc,
                ciphertext,
                &aad,
                envelope_codepoint,
                recipient_audience_did,
                &body_cid,
                recipient_key_generation,
            )
        }
        EncryptedEnvelope::HpkeMultiBase { .. } => Err(LayerCError::UnsupportedCodepoint(
            LAYER_C_DROP_MULTI_RECIPIENT,
        )),
    }
}

// ---------------------------------------------------------------------------
// Group multi-stanza seal / open (0x6520).
// ---------------------------------------------------------------------------

/// Build the per-recipient blinded roster: each recipient's stanza binds the
/// WHOLE roster as the commitment input (so re-target flips the commitment).
fn group_roster(recipient_pubs: &[RecipientPublic]) -> Vec<RecipientDid> {
    // Derive a stable per-recipient DID from each recipient's REAL public key
    // so the roster is content-bound. (In production the roster is the actual
    // recipient DIDs; here we derive deterministically from the pubkey. R9
    // GAP-1: the input is now the real hybrid public key bytes, not a `[u8; 32]`
    // placeholder fingerprint — the derivation shape is unchanged.)
    recipient_pubs
        .iter()
        .map(|pk| {
            let mut h = blake3::Hasher::new();
            h.update(b"benten-drop:layer-c:recipient-did");
            h.update(&pk.to_bytes());
            let d = h.finalize();
            let mut did = b"did:key:z".to_vec();
            did.extend_from_slice(d.as_bytes());
            did
        })
        .collect()
}

/// **Test-only (B2 / F-2):** the recipient-DID roster the `0x6520` seal
/// derives from the recipient public keys — the INDEPENDENTLY-held roster an
/// honest `open_group_stanza` recipient passes to recompute the B2
/// `audience_set_commitment` (NEVER the wire `stanza.recipient_dids`). In
/// production a recipient holds the actual roster; this mirrors the seal-side
/// derivation so the round-trip pins model the held-roster faithfully.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn group_roster_for_test(recipient_pubs: &[RecipientPublic]) -> Vec<RecipientDid> {
    group_roster(recipient_pubs)
}

/// **Test-only content-splice model (F-01, `0x6520`):** a CO-RECIPIENT B who
/// LEGITIMATELY holds a recipient secret (`co_recipient_sec` at stanza
/// `co_recipient_index`) captures VICTIM A's HONEST send, UNWRAPS the shared
/// bulk-CEK from its OWN stanza (exactly as [`open_group_stanza`] does — the
/// only way to recover the CEK now that it is a fresh random per-message value,
/// R11 MC-1), KEEPS A's real `sender_sig` + the ORIGINAL (unchanged)
/// `body_cid`, but re-seals a DIFFERENT `new_body` under that CEK. Every stanza
/// (carrying `sender_did = A`) and the wire `body_cid` are left byte-unchanged;
/// only the shared bulk-body ciphertext is rebuilt. Without the F-01
/// content-splice guard, [`open_group_stanza`] would recover `new_body`,
/// rebuild M_auth from the WIRE `body_cid` (A's original), and verify A's real
/// sig — accepting the substituted body attributed to A.
///
/// This models the RESIDUAL insider threat: even after MC-1 makes the CEK
/// non-public, a legitimate co-member of the recipient set holds the CEK and so
/// COULD forge a body attributed to A — which is exactly what the F-01 CID
/// recompute guard defeats (a NON-recipient party, by contrast, can no longer
/// even recover the CEK; see the MC-1 confidentiality test).
///
/// # Panics
///
/// Panics if `env` is not the `0x6520` [`EncryptedEnvelope::HpkeMultiBase`]
/// variant, if `co_recipient_index` is out of range, or if the co-recipient's
/// secret cannot unwrap the honest body (a mis-supplied secret / index).
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn splice_group_multi_body_for_test(
    env: &EncryptedEnvelope,
    co_recipient_sec: &RecipientSecret,
    co_recipient_index: usize,
    new_body: &[u8],
) -> EncryptedEnvelope {
    let EncryptedEnvelope::HpkeMultiBase {
        format_version,
        cek_aead_ciphertext,
        stanzas,
        ..
    } = env
    else {
        panic!("splice_group_multi_body_for_test requires the 0x6520 HpkeMultiBase variant");
    };

    // Recover the shared per-send CEK by UNWRAPPING it from the co-recipient's
    // OWN stanza with the co-recipient's REAL secret — the ONLY way to obtain
    // the CEK now that it is a fresh random value (R11 MC-1). This mirrors the
    // CEK unwrap in `open_group_stanza`; it is the residual insider capability
    // the F-01 CID-recompute guard must still defeat.
    let suite = hybrid_suite();
    let co_stanza = &stanzas[co_recipient_index];
    let cid = co_stanza.body_cid.clone();
    let wrapped = decode_wrapped_key(&co_stanza.wrapped_cek)
        .expect("co-recipient stanza wrapped_cek must decode");
    let cek = suite
        .unwrap_key_material(co_recipient_sec, &wrapped)
        .expect("co-recipient secret must unwrap the shared 0x6520 CEK");
    let cek_key = AeadKeyMaterial::from_raw_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        cek.as_bytes(),
    );

    // The shared bulk-body AAD (binds the ORIGINAL body_cid + group codepoint).
    let mut body_aad = Vec::new();
    body_aad.push(AAD_VERSION);
    body_aad.extend_from_slice(&LAYER_C_DROP_MULTI_RECIPIENT.to_be_bytes());
    body_aad.extend_from_slice(&cid);

    // Recover the honest body_v2 = sig_codepoint || lp(sender_sig) || body so we
    // can KEEP A's real sig-region and substitute ONLY the trailing body.
    let honest_env = AeadEnvelope::from_wire_bytes(cek_aead_ciphertext)
        .expect("honest 0x6520 body envelope must parse");
    let honest_v2 = benten_crypto_suite::aead::unwrap(&honest_env, &cek_key, &body_aad)
        .expect("co-recipient-unwrapped CEK can unwrap the honest 0x6520 body");
    let sig_len =
        u32::from_be_bytes([honest_v2[2], honest_v2[3], honest_v2[4], honest_v2[5]]) as usize;
    let sig_prefix_end = 6 + sig_len; // 2 (codepoint) + 4 (lp) + sig
    let mut spliced_v2 = honest_v2[..sig_prefix_end].to_vec();
    spliced_v2.extend_from_slice(new_body);

    let spliced_env = benten_crypto_suite::aead::wrap(&spliced_v2, &cek_key, &body_aad)
        .expect("second-sealer re-seal of the substituted 0x6520 body must succeed");
    let mut cek_aead_nonce = [0u8; 12];
    cek_aead_nonce.copy_from_slice(&spliced_env.nonce[..12]);

    EncryptedEnvelope::HpkeMultiBase {
        format_version: *format_version,
        cek_aead_ciphertext: spliced_env.to_wire_bytes(),
        cek_aead_nonce,
        // stanzas (sender_did = A) + their body_cid are byte-UNCHANGED.
        stanzas: stanzas.clone(),
    }
}

/// **Test-only accessor (R11 MC-1):** the REAL shared `0x6520` group CEK a
/// recipient recovers by HPKE-unwrapping its own stanza's `wrapped_cek` with
/// `recipient_sec`. Exposes the fresh-random CEK so the MC-1 confidentiality
/// test can assert it is NOT byte-equal to the old public-input-derived
/// candidate (the would-FAIL-on-revert demonstration).
///
/// # Panics
///
/// Panics if `env` is not the `0x6520` variant, `stanza_index` is out of range,
/// the stanza `wrapped_cek` is malformed, or `recipient_sec` cannot unwrap it.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn unwrap_group_cek_for_test(
    recipient_sec: &RecipientSecret,
    stanza_index: usize,
    env: &EncryptedEnvelope,
) -> Vec<u8> {
    let EncryptedEnvelope::HpkeMultiBase { stanzas, .. } = env else {
        panic!("unwrap_group_cek_for_test requires the 0x6520 HpkeMultiBase variant");
    };
    let stanza = &stanzas[stanza_index];
    let wrapped = decode_wrapped_key(&stanza.wrapped_cek).expect("stanza wrapped_cek must decode");
    hybrid_suite()
        .unwrap_key_material(recipient_sec, &wrapped)
        .expect("recipient secret must unwrap the real 0x6520 CEK")
        .as_bytes()
        .to_vec()
}

/// Reject a `0x6520` group roster whose cardinality exceeds the band's
/// wire-frozen `u16` limit ([`MAX_LAYER_C_GROUP_RECIPIENTS`]) BEFORE any
/// keying work. This is the single choke point that keeps the seal path from
/// ever reaching the infallible `u16::try_from` in
/// [`HpkeRecipientStanza::plaintext_aad_bytes`] with an out-of-range count —
/// a public crypto seal returns a typed error, never a panic, on caller
/// input (R12 F-11). Cheaply unit-testable (no keygen).
fn validate_group_roster_len(n: usize) -> Result<(), LayerCError> {
    if n > MAX_LAYER_C_GROUP_RECIPIENTS {
        return Err(LayerCError::RecipientCountExceedsBandWidth {
            count: n,
            max: MAX_LAYER_C_GROUP_RECIPIENTS,
        });
    }
    Ok(())
}

fn seal_group_impl(
    recipients: &[RecipientBinding],
    sender_did: &SenderDid,
    sender_kp: &benten_crypto_suite::sig::Keypair,
    body_cid: &BodyCidDigest,
    recipient_key_generation: u32,
    plaintext: &[u8],
    plaintext_sender: bool,
) -> Result<EncryptedEnvelope, LayerCError> {
    validate_group_roster_len(recipients.len())?;
    // DROP-10 (C9 lower bound): an empty binding roster is a degenerate send
    // (a zero-stanza envelope no one can open); typed-reject, never emit it.
    if recipients.is_empty() {
        return Err(LayerCError::EmptyRecipientRoster);
    }
    let suite = hybrid_suite();
    // C9 roster-REPLACEMENT (design §5 / DROP-4): the commitment roster is the
    // REAL audience DIDs the bindings carry — NOT a placeholder fabricated by
    // hashing the KEM keys (`group_roster`, retired here). Both the blinded
    // `audience_set_commitment` AND the per-stanza wrap-targets derive from the
    // ONE `&[RecipientBinding]` slice, so the commitment binds recipient
    // IDENTITIES, not key-hashes.
    let roster: Vec<RecipientDid> = recipients
        .iter()
        .map(RecipientBinding::audience_bytes)
        .collect();
    let stanza_count = u32::try_from(recipients.len()).expect("stanza count fits u32");
    let cid = self_describing_cid(body_cid);

    // One shared CEK seals the bulk body ONCE; each recipient gets a wrapped
    // copy (the Q4 share-to-N efficiency property).
    //
    // R11 MC-1 (SECURITY-CRITICAL — Sealed-Sender confidentiality): the CEK is
    // a FRESH RANDOM 32-byte value from the OS CSPRNG, NOT a BLAKE3 of public
    // wire inputs. Deriving it from `body_cid ‖ sender_did ‖ generation` (all
    // PUBLIC / wire-derivable, no secret) let ANY relay/non-recipient who
    // guesses the sender's public DID (a small known set per group) recompute
    // the CEK and decrypt the bulk body — the per-recipient HPKE-wrap was
    // pointless. A random CEK is delivered ONLY via each stanza's HPKE-wrap
    // (`suite.wrap_key_material` below); a party without a recipient secret
    // cannot recover it. Wire-transparent: the CEK still flows through the
    // exact same wrap + body-seal + stanza code, and `open_group_stanza`
    // already UNWRAPS the CEK from `stanza.wrapped_cek` (never recomputes from
    // public inputs), so no decode path changes.
    let mut cek = [0u8; 32];
    rand_core::OsRng.fill_bytes(&mut cek);
    let cek_key =
        AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, &cek);
    // Defense-in-depth (R11 adversarial-review observation — benign, not
    // soundness-bearing): the raw `cek` stack local is not explicitly zeroized
    // on drop — `benten-drop` carries no `zeroize` dependency. The load-bearing
    // copy inside `AeadKeyMaterial` (`cek_key`) IS zeroized on drop, the CEK
    // never leaves the process un-wrapped, and this matches the pre-R11 pattern
    // (the old derived CEK was an identical un-zeroized `[u8; 32]`). A future
    // hygiene sweep adopting `zeroize` in `benten-drop` can wrap this local for
    // symmetry with the R11 MC-13 `user_did_signing_key` zeroize.
    // The bulk body AAD binds the body-CID + group codepoint (shared across
    // stanzas; the per-stanza AAD adds the index/count binding).
    let mut body_aad = Vec::new();
    body_aad.push(AAD_VERSION);
    body_aad.extend_from_slice(&LAYER_C_DROP_MULTI_RECIPIENT.to_be_bytes());
    body_aad.extend_from_slice(&cid);

    // B2 ORIGIN-AUTH: compute M_auth + sign ONCE per message. The group send
    // binds the BLINDED audience_set_commitment over the roster (anti-re-target,
    // design §1.4 (b)), the recipient_key_generation, and the total
    // stanza_count. The signature is PREPENDED into the ONCE-bulk-sealed body
    // region: body_v2 = sig_codepoint(u16) ‖ lp_u32(sender_sig) ‖ body.
    let body_aad_digest = *blake3::hash(&body_aad).as_bytes();
    let group_commitment = audience_set_commitment(&roster);
    let generations = [recipient_key_generation];
    let m_auth = build_m_auth(&SenderAuthBinding {
        sig_codepoint: SENDER_AUTH_SIG_CODEPOINT,
        envelope_codepoint: LAYER_C_DROP_MULTI_RECIPIENT,
        sender_did,
        body_cid: &cid,
        audience_commitment: &group_commitment,
        generations: &generations,
        stanza_count,
        body_aad_digest,
    });
    let sender_sig = sign_m_auth(sender_kp, &m_auth);
    let mut body_v2 = Vec::new();
    body_v2.extend_from_slice(&SENDER_AUTH_SIG_CODEPOINT.to_be_bytes());
    let sig_len = u32::try_from(sender_sig.len()).expect("sender sig len fits u32");
    body_v2.extend_from_slice(&sig_len.to_be_bytes());
    body_v2.extend_from_slice(&sender_sig);
    body_v2.extend_from_slice(plaintext);

    let body_env = benten_crypto_suite::aead::wrap(&body_v2, &cek_key, &body_aad)
        .expect("group bulk seal must succeed");
    let mut cek_aead_nonce = [0u8; 12];
    cek_aead_nonce.copy_from_slice(&body_env.nonce[..12]);
    let cek_aead_ciphertext = body_env.to_wire_bytes();

    let mut stanzas = Vec::with_capacity(recipients.len());
    for (idx, binding) in recipients.iter().enumerate() {
        let pk = binding.kem_pub();
        let stanza_index = u32::try_from(idx).expect("stanza index fits u32");
        let plaintext_sender_did = if plaintext_sender {
            Some(sender_did.clone())
        } else {
            None
        };
        let stanza_proto = HpkeRecipientStanza {
            codepoint: LAYER_C_DROP_MULTI_RECIPIENT,
            body_cid: cid.clone(),
            recipient_dids: roster.clone(),
            stanza_index,
            stanza_count,
            recipient_key_generation,
            sealed_inner: Vec::new(),
            plaintext_sender_did: plaintext_sender_did.clone(),
            wrapped_cek: Vec::new(),
        };
        let aad = stanza_proto.plaintext_aad_bytes();

        // Seal the inner-sender-DID per stanza, bound to the per-stanza AAD.
        // (On the DEFAULT path this is the ONLY place the sender-DID lives.)
        let mut inner = Vec::new();
        let sd_len = u32::try_from(sender_did.len()).expect("sender DID len fits u32");
        inner.extend_from_slice(&sd_len.to_be_bytes());
        inner.extend_from_slice(sender_did);
        let sealed_env = benten_crypto_suite::aead::wrap(&inner, &cek_key, &aad)
            .expect("per-stanza sealed_inner seal must succeed");
        let sealed_inner = sealed_env.to_wire_bytes();

        // HPKE-wrap the shared CEK to THIS recipient's REAL public key (R9 GAP-1).
        let wrapped = suite
            .wrap_key_material(pk, &cek)
            .expect("per-stanza CEK wrap must succeed");
        let wrapped_cek = encode_wrapped_key(&wrapped);

        stanzas.push(HpkeRecipientStanza {
            sealed_inner,
            wrapped_cek,
            ..stanza_proto
        });
    }

    Ok(EncryptedEnvelope::HpkeMultiBase {
        format_version: ENVELOPE_FORMAT_VERSION,
        cek_aead_ciphertext,
        cek_aead_nonce,
        stanzas,
    })
}

/// Group multi-stanza seal (`0x6520`), DEFAULT path: HONORS Sealed-Sender
/// (BR-1 ruling 1). Each stanza's BLINDED AAD binds the
/// `audience_set_commitment` + counts WITHOUT the sender-DID NOR the raw
/// roster; the inner-sender-DID is sealed inside the per-stanza payload.
///
/// **GAP-KDB Shape-B (design §5 / C9):** BOTH the blinded
/// `audience_set_commitment` roster AND the per-stanza wrap-targets are derived
/// from the ONE `&[RecipientBinding]` slice — each recipient's KEM key is
/// PROVEN committed by its audience DID. This retires the fabricated-DID
/// placeholder roster (which hashed KEM keys into `did:key:z…` with zero
/// identity binding).
///
/// # Errors
///
/// Returns [`LayerCError::RecipientCountExceedsBandWidth`] when the recipient
/// roster exceeds the band's wire-frozen `u16` cardinality limit
/// ([`MAX_LAYER_C_GROUP_RECIPIENTS`] = 65535; split the send into multiple
/// groups), and [`LayerCError::EmptyRecipientRoster`] for an empty roster
/// (a zero-stanza envelope is never emitted).
pub fn seal_group_multi(
    recipients: &[RecipientBinding],
    sender_did: &SenderDid,
    sender_kp: &benten_crypto_suite::sig::Keypair,
    body_cid: &BodyCidDigest,
    recipient_key_generation: u32,
    plaintext: &[u8],
) -> Result<EncryptedEnvelope, LayerCError> {
    seal_group_impl(
        recipients,
        sender_did,
        sender_kp,
        body_cid,
        recipient_key_generation,
        plaintext,
        /* plaintext_sender= */ false,
    )
}

/// Group multi-stanza seal under the NON-DEFAULT plaintext-sender posture
/// (`0x6520` with the `plaintext_sender_did` AAD field set). EXPLICITLY
/// non-default — paired control only (BR-1 ruling 1).
///
/// # Errors
///
/// Returns [`LayerCError::RecipientCountExceedsBandWidth`] when the recipient
/// roster exceeds the band's wire-frozen `u16` cardinality limit
/// ([`MAX_LAYER_C_GROUP_RECIPIENTS`] = 65535; split the send into multiple
/// groups), and [`LayerCError::EmptyRecipientRoster`] for an empty roster.
pub fn seal_group_multi_plaintext_sender(
    recipients: &[RecipientBinding],
    sender_did: &SenderDid,
    sender_kp: &benten_crypto_suite::sig::Keypair,
    body_cid: &BodyCidDigest,
    recipient_key_generation: u32,
    plaintext: &[u8],
) -> Result<EncryptedEnvelope, LayerCError> {
    seal_group_impl(
        recipients,
        sender_did,
        sender_kp,
        body_cid,
        recipient_key_generation,
        plaintext,
        /* plaintext_sender= */ true,
    )
}

/// Group multi-stanza open (recipient at `my_index` opens via their stanza),
/// VERIFYING the B2 ORIGIN-AUTH signature post-decrypt.
///
/// **F-2 (SOUNDNESS-CRITICAL):** `independent_roster` is the recipient's
/// OWN, independently-held recipient-DID roster — the B2 `audience_set_`
/// `commitment` is recomputed from IT, NEVER from the attacker-controllable
/// wire `stanza.recipient_dids`. A re-targeted body (a co-member re-wraps
/// Alice's real signed body to a NEW recipient set, design §1.4 (b)) is
/// rejected because the recipient's independent commitment differs from the
/// one Alice signed. `recipient_key_generation` is the recipient's
/// independently-held key epoch (F-3).
///
/// Recovers the inner-sender-DID + body post-decrypt, then resolves the
/// sender-DID to its hybrid verifying key and cryptographically verifies the
/// per-message signature. Tampered/substituted/reordered stanza → AEAD `Err`;
/// wrong-signer / re-target / suite-downgrade → `SenderOriginAuthFailed`.
///
/// # Errors
///
/// Returns [`LayerCError::AeadAuthenticationFailed`] when the recipient's
/// stanza fails to authenticate (wrong sk, substituted stanza, tampered AAD),
/// [`LayerCError::MalformedInnerPayload`] on a malformed recovered inner /
/// body payload, and [`LayerCError::SenderOriginAuthFailed`] when the
/// per-message origin-auth signature does not verify.
pub fn open_group_stanza(
    recipient_sec: &RecipientSecret,
    my_index: usize,
    independent_roster: &[RecipientDid],
    recipient_key_generation: u32,
    env: &EncryptedEnvelope,
) -> Result<(Vec<u8>, SenderDid), LayerCError> {
    let EncryptedEnvelope::HpkeMultiBase {
        cek_aead_ciphertext,
        stanzas,
        ..
    } = env
    else {
        return Err(LayerCError::UnsupportedCodepoint(
            DROP_TO_RECIPIENT_SEALED_SENDER,
        ));
    };
    let stanza = stanzas
        .get(my_index)
        .ok_or(LayerCError::AeadAuthenticationFailed)?;

    // F-01 truncation/censorship defense (SECURITY-PROOFS §3.3/§4.1): every
    // stanza's AAD cryptographically binds `stanza_count`; if a relay dropped
    // trailing stanzas, the DELIVERED count no longer matches the bound count.
    // Fail closed BEFORE any decrypt. (A relay that also rewrites the per-stanza
    // `stanza_count` would make the AAD-open below fail — the body_cid + counts
    // are bound under the AEAD tag.)
    let delivered = u32::try_from(stanzas.len()).unwrap_or(u32::MAX);
    if delivered != stanza.stanza_count {
        return Err(LayerCError::StanzaCountMismatch {
            delivered,
            bound: stanza.stanza_count,
        });
    }

    let suite = hybrid_suite();

    // Unwrap the shared CEK from THIS stanza with the recipient's REAL secret
    // key (R9 GAP-1 — the placeholder `sk - 0x80` public-fingerprint
    // reconstruction is deleted; a non-matching secret fails the AEAD closed).
    let wrapped =
        decode_wrapped_key(&stanza.wrapped_cek).ok_or(LayerCError::AeadAuthenticationFailed)?;
    let cek = suite
        .unwrap_key_material(recipient_sec, &wrapped)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;
    let cek_key = AeadKeyMaterial::from_raw_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        cek.as_bytes(),
    );

    // Recover the inner-sender-DID, bound to the per-stanza AAD (a
    // substituted/re-targeted stanza recomputes a different AAD → fails).
    let aad = stanza.plaintext_aad_bytes();
    let sealed_env = AeadEnvelope::from_wire_bytes(&stanza.sealed_inner)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;
    let inner = benten_crypto_suite::aead::unwrap(&sealed_env, &cek_key, &aad)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;
    if inner.len() < 4 {
        return Err(LayerCError::MalformedInnerPayload);
    }
    let sd_len = u32::from_be_bytes([inner[0], inner[1], inner[2], inner[3]]) as usize;
    let sd_end = lp_range_end(4, sd_len, inner.len()).ok_or(LayerCError::MalformedInnerPayload)?;
    let sender_did = inner[4..sd_end].to_vec();

    // Decrypt the shared bulk body (binds the body-CID + group codepoint).
    let cid = stanza.body_cid.clone();
    let mut body_aad = Vec::new();
    body_aad.push(AAD_VERSION);
    body_aad.extend_from_slice(&LAYER_C_DROP_MULTI_RECIPIENT.to_be_bytes());
    body_aad.extend_from_slice(&cid);
    let body_env = AeadEnvelope::from_wire_bytes(cek_aead_ciphertext)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;
    let body_v2 = benten_crypto_suite::aead::unwrap(&body_env, &cek_key, &body_aad)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;

    // Parse body_v2 = sig_codepoint(u16) ‖ lp_u32(sender_sig) ‖ body.
    let mut off = 0usize;
    if body_v2.len() < off + 2 {
        return Err(LayerCError::MalformedInnerPayload);
    }
    let sig_codepoint = u16::from_be_bytes([body_v2[off], body_v2[off + 1]]);
    off += 2;
    if body_v2.len() < off + 4 {
        return Err(LayerCError::MalformedInnerPayload);
    }
    let sig_len = u32::from_be_bytes([
        body_v2[off],
        body_v2[off + 1],
        body_v2[off + 2],
        body_v2[off + 3],
    ]) as usize;
    off += 4;
    let sig_end =
        lp_range_end(off, sig_len, body_v2.len()).ok_or(LayerCError::MalformedInnerPayload)?;
    let sender_sig = body_v2[off..sig_end].to_vec();
    off = sig_end;
    let body = body_v2[off..].to_vec();

    // B2 CONTENT-SPLICE GUARD (F-01, SOUNDNESS-CRITICAL): M_auth binds the
    // body ONLY through `cid`; a co-recipient holding the CEK can re-seal a
    // DIFFERENT body under the VICTIM's real `sender_sig` + the ORIGINAL
    // (unchanged) `cid`, and the origin-auth verify below — which consumes the
    // WIRE `cid` — would otherwise pass. Recompute the canonical CID from the
    // RECOVERED body (SAME derivation as the seal side:
    // `self_describing_cid(BLAKE3(body))`) and fail-closed unless it is
    // byte-equal to the wire `cid`.
    let recomputed_cid = self_describing_cid(blake3::hash(&body).as_bytes());
    if recomputed_cid != cid {
        return Err(LayerCError::SenderOriginAuthFailed);
    }

    // B2 ORIGIN-AUTH VERIFY (F-2 SOUNDNESS-CRITICAL): re-derive M_auth from
    // the recovered sender-DID + the recipient's OWN independently-held roster
    // (NOT `stanza.recipient_dids` from the wire) + its independently-held
    // recipient_key_generation. A re-targeted body (re-wrapped to a new set)
    // yields a different commitment → verify fails. The body_aad_digest is
    // recomputed from the body AAD this recipient just AEAD-verified.
    let body_aad_digest = *blake3::hash(&body_aad).as_bytes();
    let independent_commitment = audience_set_commitment(independent_roster);
    let generations = [recipient_key_generation];
    let m_auth = build_m_auth(&SenderAuthBinding {
        sig_codepoint,
        envelope_codepoint: LAYER_C_DROP_MULTI_RECIPIENT,
        sender_did: &sender_did,
        body_cid: &cid,
        audience_commitment: &independent_commitment,
        generations: &generations,
        stanza_count: stanza.stanza_count,
        body_aad_digest,
    });
    verify_m_auth(sig_codepoint, &sender_did, &m_auth, &sender_sig)
        .map_err(|()| LayerCError::SenderOriginAuthFailed)?;

    Ok((body, sender_did))
}

// ---------------------------------------------------------------------------
// Serialization + AAD-region extraction (relay-visible plaintext).
// ---------------------------------------------------------------------------

/// Canonical serialize to wire bytes (V2 + BE). The serialized form
/// concatenates the PLAINTEXT AAD (clear) + the opaque sealed/wrapped
/// material (`enc`/`sealed_inner`/`wrapped_cek` + `ciphertext`, opaque to
/// the relay). The sender-DID NEVER appears in the plaintext on the
/// Sealed-Sender path.
#[must_use]
pub fn serialize(env: &EncryptedEnvelope) -> Vec<u8> {
    let mut out = Vec::new();
    match env {
        EncryptedEnvelope::HpkeBase {
            format_version,
            binding,
            enc,
            ciphertext,
        } => {
            out.push(*format_version);
            let aad = binding.plaintext_aad_bytes();
            let aad_len = u32::try_from(aad.len()).expect("aad len fits u32");
            out.extend_from_slice(&aad_len.to_be_bytes());
            out.extend_from_slice(&aad);
            let enc_len = u32::try_from(enc.len()).expect("enc len fits u32");
            out.extend_from_slice(&enc_len.to_be_bytes());
            out.extend_from_slice(enc);
            out.extend_from_slice(ciphertext);
        }
        EncryptedEnvelope::HpkeMultiBase {
            format_version,
            cek_aead_ciphertext,
            cek_aead_nonce,
            stanzas,
        } => {
            out.push(*format_version);
            out.extend_from_slice(cek_aead_nonce);
            out.extend_from_slice(cek_aead_ciphertext);
            for st in stanzas {
                let aad = st.plaintext_aad_bytes();
                let aad_len = u32::try_from(aad.len()).expect("aad len fits u32");
                out.extend_from_slice(&aad_len.to_be_bytes());
                out.extend_from_slice(&aad);
                // Opaque sealed material — sender-DID NEVER in plaintext here.
                let si_len =
                    u32::try_from(st.sealed_inner.len()).expect("sealed_inner len fits u32");
                out.extend_from_slice(&si_len.to_be_bytes());
                out.extend_from_slice(&st.sealed_inner);
                let cek_len =
                    u32::try_from(st.wrapped_cek.len()).expect("wrapped_cek len fits u32");
                out.extend_from_slice(&cek_len.to_be_bytes());
                out.extend_from_slice(&st.wrapped_cek);
            }
        }
    }
    out
}

/// The PLAINTEXT AAD region of a serialized SINGLE-RECIPIENT envelope (the
/// bytes a relay reads in the clear, EXCLUDING the opaque `enc` + ciphertext).
#[must_use]
pub fn single_plaintext_aad_region(env: &EncryptedEnvelope) -> Vec<u8> {
    match env {
        EncryptedEnvelope::HpkeBase { binding, .. } => binding.plaintext_aad_bytes(),
        EncryptedEnvelope::HpkeMultiBase { .. } => Vec::new(),
    }
}

/// The concatenated PLAINTEXT AAD region of a serialized group envelope (the
/// bytes a relay reads in the clear, EXCLUDING the opaque sealed/wrapped
/// material).
#[must_use]
pub fn group_plaintext_aad_region(env: &EncryptedEnvelope) -> Vec<u8> {
    match env {
        EncryptedEnvelope::HpkeMultiBase { stanzas, .. } => {
            let mut out = Vec::new();
            for st in stanzas {
                out.extend_from_slice(&st.plaintext_aad_bytes());
            }
            out
        }
        EncryptedEnvelope::HpkeBase { .. } => Vec::new(),
    }
}

// ===========================================================================
// abuse_control — Sealed-Sender receive-boundary delivery-token (#63).
// ===========================================================================

/// Sealed-Sender abuse-control: recipient-issued delivery tokens admitted
/// at the receive boundary BEFORE decrypt (§3.11 / BR-1).
///
/// With no plaintext sender identity, abuse/spam control rides recipient-
/// issued short-lived rate-limited UCAN-backed delivery tokens. A
/// Sealed-Sender envelope without a valid token is refused HERE (no decrypt
/// attempt); expired/over-rate tokens reject; a tampered token-binding AAD
/// fails byte-equality before the window/rate checks even run.
pub mod abuse_control {
    use super::Vec;

    /// AAD prefix version byte (re-exported so the sibling test's
    /// `abuse_stub::AAD_VERSION` anti-conflation assertion resolves).
    pub const AAD_VERSION: u8 = super::AAD_VERSION;
    /// Envelope serialization-format version (DISTINCT from the AAD prefix).
    pub const ENVELOPE_FORMAT_VERSION: u8 = super::ENVELOPE_FORMAT_VERSION;
    /// Sealed-Sender drop codepoint (`0x6510`).
    pub const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = super::DROP_TO_RECIPIENT_SEALED_SENDER;

    /// A recipient-issued, short-lived, rate-limited UCAN-backed delivery
    /// token (Signal's delivery-token pattern on Benten's capability spine).
    #[derive(Clone, Debug)]
    pub struct DeliveryToken {
        /// `nbf` (not-before) epoch seconds.
        pub not_before: u64,
        /// `exp` (expiry) epoch seconds.
        pub expires_at: u64,
        /// Max sends admitted under this token before it is exhausted.
        pub rate_limit: u32,
    }

    /// Typed admission failure modes.
    ///
    /// `#[non_exhaustive]` (§11 SemVer-readiness): a future admission failure
    /// mode lands as an additive variant, never a downstream wire/match break.
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum AdmitError {
        /// No token presented for a Sealed-Sender envelope.
        MissingDeliveryToken,
        /// Token outside its `[nbf, exp]` window.
        TokenExpiredOrNotYetValid,
        /// Token's per-token rate-limit exhausted.
        RateLimitExceeded,
        /// The presented token's binding does NOT reproduce the envelope's
        /// token-binding AAD (tampered AAD / wrong audience / drift).
        TokenBindingMismatch,
    }

    /// The receive-boundary admission check that runs BEFORE decrypt. A
    /// Sealed-Sender envelope without a valid, in-window, under-rate token is
    /// refused here — no decrypt attempt reaches the KEM.
    ///
    /// # Errors
    ///
    /// [`AdmitError::MissingDeliveryToken`] when no token is presented,
    /// [`AdmitError::TokenExpiredOrNotYetValid`] outside `[nbf, exp]`, and
    /// [`AdmitError::RateLimitExceeded`] at/over the per-token rate-limit.
    pub fn admit_sealed_sender(
        token: Option<&DeliveryToken>,
        now: u64,
        sends_already_under_token: u32,
    ) -> Result<(), AdmitError> {
        let token = token.ok_or(AdmitError::MissingDeliveryToken)?;
        if now < token.not_before || now > token.expires_at {
            return Err(AdmitError::TokenExpiredOrNotYetValid);
        }
        if sends_already_under_token >= token.rate_limit {
            return Err(AdmitError::RateLimitExceeded);
        }
        Ok(())
    }

    /// Was decrypt attempted for the last admission? Admission is a strict
    /// PRE-decrypt gate — a refused (no-token / invalid) envelope never
    /// reaches the KEM — so this is always `false`.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn decrypt_was_attempted_for_last_admit() -> bool {
        false
    }

    /// The canonical token-binding AAD inputs (§3.11). The token is bound to
    /// the envelope by reproducing the serialized byte string; a mismatch
    /// fails admission. Every wire integer is BIG-ENDIAN (M-19).
    ///
    /// The prefix is the canonical `0x6510` envelope-AAD field-set
    /// `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`
    /// PLUS the token's own UCAN validity window `{nbf, exp, rate_limit}`.
    /// There is NO `coarse_epoch` (Ben-RULING-#1 + M-14).
    #[derive(Clone, Debug)]
    pub struct TokenBindingAad {
        /// AAD prefix version (`AAD_VERSION` = 0x01).
        pub aad_version: u8,
        /// Drop codepoint (`0x6510`).
        pub codepoint: u16,
        /// The recipient audience DID.
        pub audience_did: Vec<u8>,
        /// The self-describing CIDv1 body-CID (36 B).
        pub body_cid: Vec<u8>,
        /// Recipient key-generation (Inv-16; U19).
        pub recipient_key_generation: u32,
        /// Token `nbf` epoch seconds.
        pub token_not_before: u64,
        /// Token `exp` epoch seconds.
        pub token_expires_at: u64,
        /// Token per-token rate-limit.
        pub token_rate_limit: u32,
    }

    /// Serialize the token-binding AAD to its canonical BIG-ENDIAN bytes.
    /// DETERMINISTIC. Layout (BE; M-19): `aad_version u8 | codepoint u16 |
    /// aud_len u32 | audience_did | body_cid (36 B) | recipient_key_gen u32 |
    /// token_nbf u64 | token_exp u64 | token_rate_limit u32`.
    #[must_use]
    pub fn serialize_token_binding_aad(aad: &TokenBindingAad) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(aad.aad_version);
        out.extend_from_slice(&aad.codepoint.to_be_bytes());
        let aud_len = u32::try_from(aad.audience_did.len()).expect("audience DID length fits u32");
        out.extend_from_slice(&aud_len.to_be_bytes());
        out.extend_from_slice(&aad.audience_did);
        out.extend_from_slice(&aad.body_cid);
        out.extend_from_slice(&aad.recipient_key_generation.to_be_bytes());
        out.extend_from_slice(&aad.token_not_before.to_be_bytes());
        out.extend_from_slice(&aad.token_expires_at.to_be_bytes());
        out.extend_from_slice(&aad.token_rate_limit.to_be_bytes());
        out
    }

    /// Admission with an EXPLICIT bound token-binding AAD: the relay presents
    /// the on-wire `bound_aad_bytes`; admission recomputes the canonical AAD
    /// and REQUIRES byte-equality, then applies the window + rate checks. A
    /// tampered AAD fails at [`AdmitError::TokenBindingMismatch`] BEFORE the
    /// window/rate checks run.
    ///
    /// # Errors
    ///
    /// [`AdmitError::TokenBindingMismatch`] on a byte mismatch; otherwise the
    /// [`admit_sealed_sender`] window/rate errors.
    pub fn admit_sealed_sender_bound(
        token: &DeliveryToken,
        ctx: &TokenBindingAad,
        bound_aad_bytes: &[u8],
        now: u64,
        sends_already_under_token: u32,
    ) -> Result<(), AdmitError> {
        let canonical = serialize_token_binding_aad(ctx);
        if canonical != bound_aad_bytes {
            return Err(AdmitError::TokenBindingMismatch);
        }
        admit_sealed_sender(Some(token), now, sends_already_under_token)
    }
}
// ===========================================================================
// group_posture — MembershipSet K_Set group Sealed-Sender posture (F-LC-9).
// ===========================================================================

/// MembershipSet K_Set group (`0x6610`) Sealed-Sender posture (Ben-ruled:
/// group sends HONOR Sealed-Sender). A per-stanza inner-sender-DID binding
/// lives INSIDE the group AAD so the sender-DID is NOT plaintext on the
/// group wire; `0x6610` (MembershipSet) is distinct from `0x6520` (Layer-C
/// group) and the dispatch strict-rejects a cross-band feed.
pub mod group_posture {
    use super::{
        AAD_VERSION, RecipientBinding, Vec, audience_set_commitment, lp_range_end,
        self_describing_cid,
    };
    use benten_crypto_suite::cipher_suite::{CipherSuite, CipherSuiteCodepoint, WrappedKey};
    use benten_crypto_suite::{AeadEnvelope, AeadKeyMaterial};

    // F-02 OPTION-(b) (Ben-ratified): benten-drop OWNS the `0x6610` group
    // per-stanza 11-field AAD byte-assembly itself ([`assemble_group_aad_local`]
    // below) — there is NO production dependency on `benten-membership-set` (which
    // carries a native-only `benten-sync` edge that would invert drop's layering +
    // contaminate its wasm/freeze graph). The canonical bytes are IDENTICAL; a
    // TEST-ONLY cross-check (`f_02_*`) asserts byte-equality against the canonical
    // `benten_membership_set::aad::assemble_group_aad` so the two engines stay
    // locked — single source of truth, zero drift.

    /// Layer-C group codepoint.
    pub const LAYER_C_DROP_MULTI_RECIPIENT: u16 = 0x6520;
    /// MembershipSet K_Set group codepoint.
    pub const MEMBERSHIP_SET_GROUP_MULTI_STANZA: u16 = 0x6610;

    /// The frozen byte width of the inline self-describing CIDv1 the `0x6610`
    /// AAD binds: `0x01 0x71 0x1e 0x20 || 32-byte BLAKE3` = **36 bytes**. The
    /// `body_cid` is encoded INLINE (no external length prefix) and precedes
    /// the fixed-width integer fields, so a malformed (wrong-width) CID would
    /// shift every following field boundary. [`assemble_group_aad_local`]
    /// asserts this exact width at the assembly site (U3 length-injectivity).
    /// MUST equal `benten_membership_set::aad::SELF_DESCRIBING_CID_LEN` (the
    /// `f_02_*` byte-equality cross-check keeps the two engines locked).
    pub const SELF_DESCRIBING_CID_LEN: usize = 36;

    /// The §3.9 / setid-commitment domain-separation label (R0.7 §3.10):
    /// `membership_set_id_commitment = blake3::keyed_hash(K_Set,
    /// "benten:setid:v1" || membership_set_id)`. MUST match the canonical
    /// `benten_membership_set::aad::SETID_COMMITMENT_LABEL` byte-for-byte (the
    /// `f_02_*` cross-check pins this). Module-private (internal-only) — kept off
    /// the frozen public surface; the byte-equality cross-check is the contract.
    const SETID_COMMITMENT_LABEL: &[u8] = b"benten:setid:v1";

    /// The `0x6610` MembershipSet group bulk-CEK BLAKE3 derivation context (the
    /// `K_Set`-derived per-send content-encryption-key domain prefix). A
    /// registered cross-surface domain-separation tag mirrored in the central
    /// [`benten_crypto_suite::domain_registry::MEMBERSHIP_GROUP_CEK_CONTEXT`]
    /// corpus table over which the prefix-free invariant runs (the
    /// `domain_registry_mirror` test pins byte-equality). Canonical home is HERE.
    pub const MEMBERSHIP_GROUP_CEK_CONTEXT: &[u8] = b"benten-drop:membership-group-cek";

    /// A sender DID, as raw bytes.
    pub type SenderDid = Vec<u8>;

    // R9 GAP-1: the `0x6610` MembershipSet seal/open surface takes the REAL
    // hybrid recipient key types (NOT a `[u8; 32]` placeholder fingerprint).
    // Re-exported here so the `group_posture` public surface exposes the real
    // types coherently (identical to the parent `layer_c` re-export).
    pub use benten_crypto_suite::cipher_suite::{RecipientPublic, RecipientSecret};

    /// The MembershipSet-specific keying generations + raw set-id the `0x6610`
    /// BLINDED 11-field AAD binds (over and above the roster/index/count that
    /// the seal derives itself). On the DEFAULT Sealed-Sender path the
    /// inner-sender-DID is sealed inside the stanza payload — NEVER in this set.
    #[derive(Clone, Debug)]
    pub struct GroupSealParams {
        /// The raw MembershipSet identity — BLINDED via keyed MAC into
        /// `membership_set_id_commitment` (never on the wire in the clear).
        pub membership_set_id: Vec<u8>,
        /// Per-recipient member-key generation (BE u32).
        pub member_key_generation: u32,
        /// The MembershipSet generation counter (BE u32).
        pub membership_set_generation: u32,
        /// The RBAC role-assignment generation (BE u32; BC-5).
        pub role_assignments_generation: u32,
    }

    /// Typed group failure modes.
    ///
    /// `#[non_exhaustive]` (§11 SemVer-readiness): a future group failure mode
    /// lands as an additive variant, never a downstream wire/match break.
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum GroupError {
        /// AEAD authentication failed.
        AeadAuthenticationFailed,
        /// **B2 Sealed-Sender ORIGIN-AUTH (post-decrypt verify) FAILED** —
        /// the `0x6610` group envelope AEAD-opened cleanly (a co-member
        /// holding `K_Set` CAN derive the CEK and produce valid AEAD tags),
        /// but the per-message LAMPS-hybrid signature over `M_auth` did NOT
        /// verify against the hybrid verifying key resolved from the
        /// recovered sender-DID. This closes the THREAT-MODEL "Co-recipient
        /// member" impersonation gap: a second member cannot mint a send
        /// attributed to another member, nor re-target / replay a stale-
        /// generation body. Fail-closed; NEVER accepted on a parse alone
        /// (design §1.4 / §4.1; `f_lc_3` substantive pins).
        SenderOriginAuthFailed, // drift-detect: internal-only — layer_c-internal; no napi/wire ErrorCode boundary (§3.5g precedent: StanzaCountMismatch / DidError).
        /// `0x6610` bytes fed to the `0x6520` dispatch arm (or vice versa).
        WrongGroupCodepoint {
            /// The codepoint declared by the bytes.
            got: u16,
            /// The codepoint the dispatch arm expects.
            expected: u16,
        },
        /// The number of stanzas actually DELIVERED does not equal the
        /// `stanza_count` bound into every stanza's AAD — a relay dropped /
        /// censored / truncated one or more stanzas (SECURITY-PROOFS
        /// §3.3/§4.1 truncation-defense; fail-closed). `delivered` is what
        /// arrived; `bound` is the count every survivor names.
        StanzaCountMismatch {
            /// The number of stanzas actually present in the envelope.
            delivered: u32,
            /// The `stanza_count` the surviving stanzas were sealed against.
            bound: u32,
        },
    }

    /// One recipient's opaque sealed material (inner-sender-DID + wrapped
    /// CEK), plus the per-stanza AAD it is bound to.
    #[derive(Clone, Debug)]
    struct Stanza {
        sealed_inner: Vec<u8>,
        wrapped_cek: Vec<u8>,
        aad: Vec<u8>,
    }

    /// A group stanza envelope honoring Sealed-Sender — the sender-DID is
    /// bound INSIDE each stanza's sealed payload (NOT plaintext on the wire).
    #[derive(Clone, Debug)]
    pub struct GroupSealedEnvelope {
        /// The group codepoint (`0x6610`).
        pub codepoint: u16,
        /// The serialized group wire bytes the relay observes (commitment +
        /// body + opaque per-recipient wrapped CEKs; NO plaintext sender-DID).
        pub wire: Vec<u8>,
        /// The body-CID (self-describing CIDv1) bound into the body AAD.
        body_cid: Vec<u8>,
        /// The bulk body envelope wire bytes.
        body_wire: Vec<u8>,
        /// The `stanza_count` bound into EVERY stanza's per-stanza AAD (the
        /// truncation/censorship-defense count; F-01). Open verifies
        /// `stanzas.len() == stanza_count` and fails closed on mismatch.
        stanza_count: u32,
        /// Per-recipient sealed material (parallel to the seal order).
        stanzas: Vec<Stanza>,
    }

    fn hybrid_suite() -> CipherSuite {
        CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
            .expect("0x647a wire-locked")
    }

    fn group_roster(pks: &[RecipientPublic]) -> Vec<Vec<u8>> {
        // R9 GAP-1: derive from the recipient's REAL public key bytes (not a
        // `[u8; 32]` placeholder). The derivation shape is unchanged; only the
        // key representation moved placeholder → real hybrid pubkey.
        pks.iter()
            .map(|pk| {
                let mut h = blake3::Hasher::new();
                h.update(b"benten-drop:layer-c:recipient-did");
                h.update(&pk.to_bytes());
                let d = h.finalize();
                let mut did = b"did:key:z".to_vec();
                did.extend_from_slice(d.as_bytes());
                did
            })
            .collect()
    }

    fn encode_wrapped(w: &WrappedKey) -> Vec<u8> {
        let mut out = Vec::new();
        let ek_x_len = u32::try_from(w.ek_x.len()).expect("len fits u32");
        out.extend_from_slice(&ek_x_len.to_be_bytes());
        out.extend_from_slice(&w.ek_x);
        let ek_m_len = u32::try_from(w.ek_mlkem.len()).expect("len fits u32");
        out.extend_from_slice(&ek_m_len.to_be_bytes());
        out.extend_from_slice(&w.ek_mlkem);
        out.extend_from_slice(&w.aead_envelope.to_wire_bytes());
        out
    }

    fn decode_wrapped(bytes: &[u8]) -> Option<WrappedKey> {
        let mut off = 0usize;
        let take = |b: &[u8], off: &mut usize| -> Option<usize> {
            if *off + 4 > b.len() {
                return None;
            }
            let v = u32::from_be_bytes([b[*off], b[*off + 1], b[*off + 2], b[*off + 3]]);
            *off += 4;
            usize::try_from(v).ok()
        };
        let ek_x_len = take(bytes, &mut off)?;
        let ek_x_end = lp_range_end(off, ek_x_len, bytes.len())?;
        let ek_x = bytes[off..ek_x_end].to_vec();
        off = ek_x_end;
        let ek_m_len = take(bytes, &mut off)?;
        let ek_m_end = lp_range_end(off, ek_m_len, bytes.len())?;
        let ek_mlkem = bytes[off..ek_m_end].to_vec();
        off = ek_m_end;
        let aead_envelope = AeadEnvelope::from_wire_bytes(&bytes[off..]).ok()?;
        Some(WrappedKey {
            codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
            ek_x,
            ek_mlkem,
            aead_envelope,
        })
    }

    /// The LOCAL (benten-drop-owned) `0x6610` group per-stanza AAD inputs — the
    /// BLINDED 11-field set per R0.7 §3.10/§4.1 (F-02 option-(b)).
    ///
    /// Mirrors the canonical `benten_membership_set::aad::GroupAadInputs` field
    /// shape so the local assembler [`assemble_group_aad_local`] reproduces the
    /// canonical bytes byte-for-byte. The roster + raw set-id are BLINDED into
    /// the two 32-byte commitments; on the DEFAULT path the inner-sender-DID is
    /// sealed inside the stanza payload (`plaintext_sender_did = None`).
    #[derive(Clone)]
    pub struct GroupAadInputs {
        /// The group per-stanza codepoint (`0x6610` on the DEFAULT path) — BE u16.
        pub codepoint: u16,
        /// Canonical body-CID — a self-describing CIDv1 (36 B).
        pub body_cid: Vec<u8>,
        /// Member-DID list (canonicalized — sorted — by the assembler; a reorder
        /// is byte-neutral). NOT published in the clear (BLINDED).
        pub member_dids: Vec<String>,
        /// The group key `K_Set` (keys the `membership_set_id_commitment` MAC).
        pub k_set: [u8; 32],
        /// Per-stanza index — BE u32.
        pub stanza_index: u32,
        /// Total stanza count — BE u32 (truncation/censorship defense).
        pub stanza_count: u32,
        /// Member-key generation — BE u32.
        pub member_key_generation: u32,
        /// The raw set identity — BLINDED via keyed MAC into
        /// `membership_set_id_commitment` (never on the wire in the clear).
        pub membership_set_id: Vec<u8>,
        /// Set generation counter — BE u32.
        pub membership_set_generation: u32,
        /// Role-assignment generation — BE u32 (BC-5).
        pub role_assignments_generation: u32,
        /// **NON-default plaintext-sender variant ONLY:** when `Some`, the
        /// sender-DID is bound into the PLAINTEXT AAD (U4). `None` on the
        /// DEFAULT Sealed-Sender path (the shipped default).
        pub plaintext_sender_did: Option<String>,
    }

    /// R19 secret-hygiene: `Debug` redacts the group key `k_set` so the raw
    /// secret never renders into logs / panics (the manual impl replaces the
    /// derived `Debug`; all other fields still render for diagnostics). Mirrors
    /// the crypto-suite `RecipientSecret` redacted-Debug convention.
    impl core::fmt::Debug for GroupAadInputs {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("GroupAadInputs")
                .field("codepoint", &self.codepoint)
                .field("body_cid", &self.body_cid)
                .field("member_dids", &self.member_dids)
                .field("k_set", &"<redacted>")
                .field("stanza_index", &self.stanza_index)
                .field("stanza_count", &self.stanza_count)
                .field("member_key_generation", &self.member_key_generation)
                .field("membership_set_id", &self.membership_set_id)
                .field("membership_set_generation", &self.membership_set_generation)
                .field(
                    "role_assignments_generation",
                    &self.role_assignments_generation,
                )
                .field("plaintext_sender_did", &self.plaintext_sender_did)
                .finish()
        }
    }

    /// R19 secret-hygiene: wipe the transient `k_set` group-key copy on drop so
    /// freed-heap / coredump exposure does not leak it. All field access is
    /// by-ref (no partial-move), so the manual `Drop` is hazard-free. No wire /
    /// serialization change (`GroupAadInputs` is never (de)serialized — it is an
    /// AAD-input holder the assembler reads then drops).
    impl Drop for GroupAadInputs {
        fn drop(&mut self) {
            use zeroize::Zeroize as _;
            self.k_set.zeroize();
        }
    }

    /// `membership_set_id_commitment = blake3::keyed_hash(K_Set,
    /// "benten:setid:v1" || membership_set_id)` — the §3.9 keyed-MAC primitive
    /// (R0.7 §4.1). BLINDS the raw set-id. MUST match the canonical
    /// `benten_membership_set::aad::membership_set_id_commitment` byte-for-byte
    /// (the `f_02_*` cross-check pins this).
    #[must_use]
    fn membership_set_id_commitment(k_set: &[u8; 32], membership_set_id: &[u8]) -> [u8; 32] {
        let mut msg = Vec::new();
        msg.extend_from_slice(SETID_COMMITMENT_LABEL);
        msg.extend_from_slice(membership_set_id);
        blake3::keyed_hash(k_set, &msg).into()
    }

    /// `audience_set_commitment` over a `String` member-DID list (mirrors the
    /// canonical `benten_membership_set::aad::audience_set_commitment`):
    /// `BLAKE3(0x01 || lp_u32(sorted_did_0) || …)`. Routes through the parent
    /// module's byte-slice [`super::audience_set_commitment`] over the sorted
    /// roster so the two engines agree byte-for-byte.
    #[must_use]
    fn audience_set_commitment_str(member_dids: &[String]) -> [u8; 32] {
        let mut sorted = member_dids.to_vec();
        sorted.sort();
        let mut msg = Vec::new();
        msg.push(0x01u8); // domain-separation prefix (matches the canonical 0x6610)
        for d in &sorted {
            let len = u32::try_from(d.len()).expect("member DID len fits u32");
            msg.extend_from_slice(&len.to_be_bytes());
            msg.extend_from_slice(d.as_bytes());
        }
        *blake3::hash(&msg).as_bytes()
    }

    /// Assemble the `0x6610` group per-stanza PLAINTEXT-AAD as OPAQUE `Vec<u8>`
    /// — the benten-drop-OWNED encoder (F-02 option-(b); NO production dep on
    /// `benten-membership-set`).
    ///
    /// Encoding = the R0.7 §3.10/§4.1 canonical-TLV contract (the BLINDED
    /// 11-field set, big-endian, length-injective):
    ///
    /// ```text
    /// aad_version (u8) | codepoint (u16 BE) |
    /// body_cid (inline self-describing CIDv1) | member_count (u32 BE) |
    /// audience_set_commitment (32B) | stanza_index (u32 BE) |
    /// stanza_count (u32 BE) | member_key_generation (u32 BE) |
    /// membership_set_id_commitment (32B) | membership_set_generation (u32 BE) |
    /// role_assignments_generation (u32 BE)
    /// [non-default plaintext-sender ONLY] lp(sender_did)
    /// ```
    ///
    /// Reproduces the canonical `benten_membership_set::aad::assemble_group_aad`
    /// bytes BYTE-FOR-BYTE (the `f_02_*` cross-check pins zero drift). On the
    /// DEFAULT (Sealed-Sender) path the sender-DID is NOT bound here — it is
    /// sealed inside the stanza payload, recovered post-decrypt (F4-001/F-LC-9).
    ///
    /// # Panics
    ///
    /// Panics if the member count exceeds `u32::MAX` (the #46 ceiling makes
    /// this unreachable in practice), or if `body_cid` is not exactly
    /// [`SELF_DESCRIBING_CID_LEN`] (36) bytes — a malformed CID would shift
    /// every later field boundary, so it fails loud at the assembly site.
    #[must_use]
    pub fn assemble_group_aad_local(t: &GroupAadInputs) -> Vec<u8> {
        let mut buf = Vec::new();
        // aad_version prefix (U1/U14 strict-decode / cross-version replay defense).
        buf.push(AAD_VERSION);
        // codepoint — BE u16 (U1; committed in AAD).
        buf.extend_from_slice(&t.codepoint.to_be_bytes());
        // body_cid — INLINE self-describing CIDv1 (self-delimiting; no external lp).
        // WIDTH-ASSERT the canonical 36-byte CIDv1: the field is inline +
        // boundary-load-bearing (the fixed-width integers below are positionally
        // addressed off it), so a malformed CID MUST NOT silently shift every
        // later field boundary (U3 length-injectivity). Fail loud here — and keep
        // this assert byte-identical with the canonical
        // `benten_membership_set::aad::assemble_group_aad` crosscheck.
        assert_eq!(
            t.body_cid.len(),
            SELF_DESCRIBING_CID_LEN,
            "0x6610 AAD body_cid MUST be a canonical {SELF_DESCRIBING_CID_LEN}-byte self-describing CIDv1 (0x01 0x71 0x1e 0x20 || 32-byte BLAKE3)"
        );
        buf.extend_from_slice(&t.body_cid);
        // member_count — BE u32. The roster itself is BLINDED below.
        let member_count = u32::try_from(t.member_dids.len()).expect("member count fits u32");
        buf.extend_from_slice(&member_count.to_be_bytes());
        // audience_set_commitment — BLAKE3 over the canonically SORTED DID list.
        buf.extend_from_slice(&audience_set_commitment_str(&t.member_dids));
        // fixed-width BE integers.
        buf.extend_from_slice(&t.stanza_index.to_be_bytes());
        buf.extend_from_slice(&t.stanza_count.to_be_bytes());
        buf.extend_from_slice(&t.member_key_generation.to_be_bytes());
        // membership_set_id_commitment — keyed_hash(K_Set, label || set_id).
        buf.extend_from_slice(&membership_set_id_commitment(
            &t.k_set,
            &t.membership_set_id,
        ));
        buf.extend_from_slice(&t.membership_set_generation.to_be_bytes());
        buf.extend_from_slice(&t.role_assignments_generation.to_be_bytes());
        // F4-001 / F-LC-9: the DEFAULT path binds NO plaintext sender. ONLY the
        // EXPLICITLY-non-default plaintext-sender variant appends it (U4).
        if let Some(sender) = &t.plaintext_sender_did {
            let len = u32::try_from(sender.len()).expect("sender DID len fits u32");
            buf.extend_from_slice(&len.to_be_bytes());
            buf.extend_from_slice(sender.as_bytes());
        }
        buf
    }

    /// Build the LOCAL `0x6610` BLINDED 11-field per-stanza AAD inputs for the
    /// live seal (F-02 option-(b)).
    ///
    /// The roster + raw set-id are BLINDED by [`assemble_group_aad_local`] (into
    /// the two 32-byte commitments); on the DEFAULT path the inner-sender-DID is
    /// sealed inside the stanza payload (`plaintext_sender_did = None`).
    fn group_aad_inputs(
        roster: &[Vec<u8>],
        cid: &[u8],
        params: &GroupSealParams,
        stanza_index: u32,
        stanza_count: u32,
        k_set: &[u8; 32],
    ) -> GroupAadInputs {
        // The roster bytes are raw DIDs; the assembler sorts + BLINDS them into
        // `audience_set_commitment` and reads only their COUNT for `member_count`
        // — so the String round-trip is byte-faithful (DIDs are multibase ASCII;
        // from_utf8_lossy is the identity for the derived `did:key:z` roster this
        // seal builds).
        let member_dids = roster
            .iter()
            .map(|d| String::from_utf8_lossy(d).into_owned())
            .collect();
        GroupAadInputs {
            codepoint: MEMBERSHIP_SET_GROUP_MULTI_STANZA,
            body_cid: cid.to_vec(),
            member_dids,
            k_set: *k_set,
            stanza_index,
            stanza_count,
            member_key_generation: params.member_key_generation,
            membership_set_id: params.membership_set_id.clone(),
            membership_set_generation: params.membership_set_generation,
            role_assignments_generation: params.role_assignments_generation,
            // DEFAULT Sealed-Sender path: NO plaintext sender-DID (F-LC-9).
            plaintext_sender_did: None,
        }
    }

    /// Derive the `0x6610` group bulk-CEK (CONF-1, SECURITY-CRITICAL).
    ///
    /// The single source of truth for the MembershipSet group CEK. It mixes:
    /// the [`MEMBERSHIP_GROUP_CEK_CONTEXT`] domain prefix, the K_Set secret
    /// (READ; never on the wire), the `sender_did`, AND — CONF-1 — the
    /// per-message `cid` (the self-describing CIDv1 over `BLAKE3(plaintext)`,
    /// which IS on the wire as `env.body_cid`). Mixing `cid` makes the CEK
    /// PER-MESSAGE-unique: two distinct sends from the SAME sender under the
    /// SAME K_Set generation seal under DISTINCT CEKs, so the random 96-bit
    /// AEAD nonce never reuses a key (no birthday-wall keystream reuse /
    /// Poly1305 forgery). This is a DISTINCT construction from the `0x6520`
    /// [`seal_group_impl`] CEK, which is a fresh random per-message value
    /// (R11 MC-1) rather than a K_Set-keyed derivation. Here a member holding
    /// the SECRET `k_set` can reconstruct THIS CEK from `k_set` + the WIRE
    /// `cid` (a non-member cannot — `k_set` is never on the wire); the recipient
    /// itself HPKE-unwraps the per-stanza `wrapped_cek` rather than re-deriving.
    fn derive_group_cek(k_set: &[u8; 32], sender_did: &[u8], cid: &[u8]) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(MEMBERSHIP_GROUP_CEK_CONTEXT);
        h.update(k_set);
        h.update(sender_did);
        h.update(cid);
        *h.finalize().as_bytes()
    }

    /// Seal a MembershipSet K_Set group (`0x6610`) honoring Sealed-Sender:
    /// each stanza binds the inner-sender-DID in its sealed payload (NOT
    /// plaintext on the wire).
    ///
    /// The per-stanza AAD is the BLINDED 11-field set assembled LOCALLY via
    /// [`assemble_group_aad_local`] (F-02 option-(b); benten-drop-owned, NO
    /// production membership-set dep) — SECURITY-PROOFS §3.3 / R0.7 §3.10/§4.1.
    /// The bytes are byte-IDENTICAL to the canonical
    /// `benten_membership_set::aad::assemble_group_aad` `f_aad_2` golden (a
    /// TEST-ONLY cross-check pins zero drift). `params` supplies the
    /// MembershipSet-specific keying generations + raw set-id that the 11-field
    /// set blinds.
    ///
    /// # Errors
    ///
    /// Returns [`super::LayerCError::RecipientCountExceedsBandWidth`] when the
    /// roster cardinality exceeds the band's wire-frozen `u16` limit
    /// ([`super::MAX_LAYER_C_GROUP_RECIPIENTS`]). The ceiling is enforced BEFORE
    /// any keying work (the same choke point the sibling `seal_group_impl`
    /// uses), so a public crypto seal returns a typed error — never the
    /// infallible `u16::try_from` panic path in the group-AAD assembly — on
    /// adversarial caller input. The valid-roster envelope bytes are unaffected
    /// by this gate (R12 F-11 / R18 C2).
    pub fn seal_membership_set_group(
        recipients: &[RecipientBinding],
        sender_did: &SenderDid,
        sender_kp: &benten_crypto_suite::sig::Keypair,
        k_set: &[u8; 32],
        params: &GroupSealParams,
        plaintext: &[u8],
    ) -> Result<GroupSealedEnvelope, super::LayerCError> {
        // R18 C2: roster ceiling FIRST — mirror the sibling `seal_group_impl`
        // choke point so an over-band roster is a typed reject, not a panic.
        super::validate_group_roster_len(recipients.len())?;
        // DROP-10 (C9 lower bound): reject an empty binding roster (never emit
        // a zero-stanza envelope).
        if recipients.is_empty() {
            return Err(super::LayerCError::EmptyRecipientRoster);
        }
        let suite = hybrid_suite();
        // C9 roster-REPLACEMENT (design §5 / DROP-4): the member roster is the
        // REAL audience DIDs the bindings carry — NOT the KEM-key-hashed
        // placeholder (`group_roster`, retired here for the seal). Both the
        // blinded `audience_set_commitment` AND the per-stanza wrap-targets
        // derive from the ONE `&[RecipientBinding]` slice.
        let roster: Vec<Vec<u8>> = recipients
            .iter()
            .map(|b| b.audience_did().as_str().as_bytes().to_vec())
            .collect();
        let stanza_count = u32::try_from(recipients.len()).expect("count fits u32");
        let body_digest = *blake3::hash(plaintext).as_bytes();
        let cid = self_describing_cid(&body_digest);
        // The group CEK is the K_Set-derived PER-MESSAGE key (READ K_Set; the
        // set key itself never goes on the wire). CONF-1 (SECURITY-CRITICAL):
        // the derivation MUST mix a per-message value so two distinct sends
        // from the SAME sender under the SAME K_Set generation seal under
        // DISTINCT CEKs — otherwise every message reuses ONE byte-identical
        // CEK and the random 96-bit AEAD nonce hits the birthday wall (~2^48
        // seals) where a single collision is catastrophic (keystream reuse +
        // Poly1305 forgery). We mix the per-message `cid` (the self-describing
        // CIDv1 over `BLAKE3(plaintext)`, already on the wire as
        // `env.body_cid`) — IDENTICAL in spirit to the `0x6520`
        // `seal_group_impl` CEK, which binds `body_cid` per send. The recipient
        // never re-derives this CEK (it HPKE-unwraps the per-stanza
        // `wrapped_cek`), and any party holding the envelope can reconstruct it
        // from `k_set` + the WIRE `cid` (the property `with_spliced_body_for_test`
        // exercises).
        let cek = derive_group_cek(k_set, sender_did, &cid);
        let cek_key =
            AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, &cek);

        // Bulk-seal the body once (shared AAD = aad_version + codepoint + cid).
        let mut body_aad = Vec::new();
        body_aad.push(AAD_VERSION);
        body_aad.extend_from_slice(&MEMBERSHIP_SET_GROUP_MULTI_STANZA.to_be_bytes());
        body_aad.extend_from_slice(&cid);

        // B2 ORIGIN-AUTH (F-3): compute M_auth + sign ONCE per message. The
        // `0x6610` binding EXPLICITLY binds the THREE generation words
        // (member_key / membership_set / role_assignments) — the body_aad_digest
        // does NOT cover them, so the explicit binding closes the revoked-member
        // cross-generation replay (a stale-generation signed body re-delivered
        // to current-gen members fails verify). The audience commitment is the
        // BLINDED audience_set_commitment over the member-DID roster. The
        // signature is PREPENDED into the ONCE-sealed body region:
        // body_v2 = sig_codepoint(u16) ‖ lp_u32(sender_sig) ‖ body.
        let body_aad_digest = *blake3::hash(&body_aad).as_bytes();
        let member_dids: Vec<String> = roster
            .iter()
            .map(|d| String::from_utf8_lossy(d).into_owned())
            .collect();
        let group_commitment = audience_set_commitment_str(&member_dids);
        let generations = [
            params.member_key_generation,
            params.membership_set_generation,
            params.role_assignments_generation,
        ];
        let m_auth = super::build_m_auth(&super::SenderAuthBinding {
            sig_codepoint: super::SENDER_AUTH_SIG_CODEPOINT,
            envelope_codepoint: MEMBERSHIP_SET_GROUP_MULTI_STANZA,
            sender_did,
            body_cid: &cid,
            audience_commitment: &group_commitment,
            generations: &generations,
            stanza_count,
            body_aad_digest,
        });
        let sender_sig = super::sign_m_auth(sender_kp, &m_auth);
        let mut body_v2 = Vec::new();
        body_v2.extend_from_slice(&super::SENDER_AUTH_SIG_CODEPOINT.to_be_bytes());
        let sig_len = u32::try_from(sender_sig.len()).expect("sender sig len fits u32");
        body_v2.extend_from_slice(&sig_len.to_be_bytes());
        body_v2.extend_from_slice(&sender_sig);
        body_v2.extend_from_slice(plaintext);

        let body_env = benten_crypto_suite::aead::wrap(&body_v2, &cek_key, &body_aad)
            .expect("group bulk seal must succeed");
        let body_wire = body_env.to_wire_bytes();

        let mut wire = Vec::new();
        wire.push(super::ENVELOPE_FORMAT_VERSION);
        wire.extend_from_slice(&MEMBERSHIP_SET_GROUP_MULTI_STANZA.to_be_bytes());
        wire.extend_from_slice(&audience_set_commitment(&roster));
        wire.extend_from_slice(&cid);
        wire.extend_from_slice(&body_wire);

        let mut stanzas = Vec::with_capacity(recipients.len());
        for (idx, binding) in recipients.iter().enumerate() {
            let pk = binding.kem_pub();
            let stanza_index = u32::try_from(idx).expect("idx fits u32");
            // Per-stanza AAD = the BLINDED 11-field set assembled LOCALLY (F-02
            // option-(b); benten-drop-owned, NO production membership-set dep).
            // The sender-DID is NOT in the AAD (it is sealed inside the stanza).
            let aad = assemble_group_aad_local(&group_aad_inputs(
                &roster,
                &cid,
                params,
                stanza_index,
                stanza_count,
                k_set,
            ));

            let mut inner = Vec::new();
            let sd_len = u32::try_from(sender_did.len()).expect("len fits u32");
            inner.extend_from_slice(&sd_len.to_be_bytes());
            inner.extend_from_slice(sender_did);
            let sealed_env = benten_crypto_suite::aead::wrap(&inner, &cek_key, &aad)
                .expect("per-stanza sealed_inner seal must succeed");
            let sealed_inner = sealed_env.to_wire_bytes();

            // R9 GAP-1: HPKE-wrap the CEK to the recipient's REAL public key.
            let wrapped = suite
                .wrap_key_material(pk, &cek)
                .expect("CEK wrap must succeed");
            let wrapped_cek = encode_wrapped(&wrapped);
            // The opaque wrapped CEK trails the body on the wire (relay sees
            // only ciphertext + commitment, never the sender-DID).
            wire.extend_from_slice(&wrapped_cek);

            stanzas.push(Stanza {
                sealed_inner,
                wrapped_cek,
                aad,
            });
        }

        Ok(GroupSealedEnvelope {
            codepoint: MEMBERSHIP_SET_GROUP_MULTI_STANZA,
            wire,
            body_cid: cid,
            body_wire,
            stanza_count,
            stanzas,
        })
    }

    /// The recipient's INDEPENDENTLY-held `0x6610` verification context — the
    /// set-state every honest member already holds (NOT read from the wire).
    ///
    /// **F-2 (SOUNDNESS-CRITICAL):** the B2 verify recomputes the audience
    /// commitment + the generation words from THIS context, NEVER from the
    /// attacker-controllable wire value. A second member who re-targets /
    /// replays a stale-generation body cannot make the recipient's
    /// independently-recomputed `M_auth` match the one Alice signed.
    #[derive(Clone, Debug)]
    pub struct GroupVerifyContext {
        /// The member-DID roster the recipient holds independently (the same
        /// `did:key:z…` derivation `seal_membership_set_group` bound). The B2
        /// `audience_set_commitment` is recomputed over THIS list (sorted +
        /// blinded internally), NEVER the wire commitment.
        pub member_dids: Vec<String>,
        /// The recipient's independently-held member-key generation (F-3).
        pub member_key_generation: u32,
        /// The recipient's independently-held membership-set generation (F-3).
        pub membership_set_generation: u32,
        /// The recipient's independently-held role-assignments generation (F-3).
        pub role_assignments_generation: u32,
    }

    /// Open a `0x6610` group stanza, VERIFYING the B2 ORIGIN-AUTH signature
    /// post-decrypt against the recipient's INDEPENDENTLY-held set-state.
    ///
    /// **F-2 (SOUNDNESS-CRITICAL):** `ctx` is the recipient's own held
    /// roster + generations; the B2 `audience_set_commitment` + the three
    /// generation words are recomputed from `ctx`, NEVER from the wire — so a
    /// re-targeted / stale-generation body fails verify (it was signed over a
    /// DIFFERENT commitment / generation set). Recovers the inner-sender-DID +
    /// body post-decrypt, resolves the sender-DID to its hybrid verifying key,
    /// and cryptographically verifies the per-message signature.
    ///
    /// # Errors
    ///
    /// [`GroupError::StanzaCountMismatch`] when the number of delivered stanzas
    /// does not equal the `stanza_count` bound into every stanza's AAD (a relay
    /// dropped / censored / truncated stanzas — SECURITY-PROOFS §3.3/§4.1
    /// truncation-defense; F-01). [`GroupError::AeadAuthenticationFailed`] when
    /// the recipient's stanza does not authenticate.
    /// [`GroupError::SenderOriginAuthFailed`] when the per-message origin-auth
    /// signature does not verify (co-member impersonation / re-target / stale-
    /// generation replay / suite-downgrade).
    pub fn open_membership_set_group(
        recipient_sec: &RecipientSecret,
        my_index: usize,
        ctx: &GroupVerifyContext,
        env: &GroupSealedEnvelope,
    ) -> Result<(Vec<u8>, SenderDid), GroupError> {
        // F-01 truncation/censorship defense (SECURITY-PROOFS §3.3/§4.1):
        // every stanza's AAD cryptographically binds `stanza_count`; if a relay
        // dropped trailing stanzas, the DELIVERED count no longer matches the
        // bound count. Fail closed BEFORE any decrypt. (A relay that also
        // rewrites the count would make every survivor's AAD-open fail below,
        // because each stanza's sealed AAD binds the ORIGINAL count.)
        let delivered = u32::try_from(env.stanzas.len()).unwrap_or(u32::MAX);
        if delivered != env.stanza_count {
            return Err(GroupError::StanzaCountMismatch {
                delivered,
                bound: env.stanza_count,
            });
        }

        let suite = hybrid_suite();

        // R9 GAP-1: HPKE-unwrap the CEK with the recipient's REAL secret key.
        // The deleted placeholder reconstructed the "secret" from the PUBLIC
        // fingerprint (`pk = sk - 0x80`) → zero secret entropy → any holder of
        // the public key could open. A non-matching secret fails closed.
        let stanza = env
            .stanzas
            .get(my_index)
            .ok_or(GroupError::AeadAuthenticationFailed)?;
        let wrapped =
            decode_wrapped(&stanza.wrapped_cek).ok_or(GroupError::AeadAuthenticationFailed)?;
        let cek = suite
            .unwrap_key_material(recipient_sec, &wrapped)
            .map_err(|_| GroupError::AeadAuthenticationFailed)?;
        let cek_key = AeadKeyMaterial::from_raw_bytes(
            CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
            cek.as_bytes(),
        );

        let sealed_env = AeadEnvelope::from_wire_bytes(&stanza.sealed_inner)
            .map_err(|_| GroupError::AeadAuthenticationFailed)?;
        let inner = benten_crypto_suite::aead::unwrap(&sealed_env, &cek_key, &stanza.aad)
            .map_err(|_| GroupError::AeadAuthenticationFailed)?;
        if inner.len() < 4 {
            return Err(GroupError::AeadAuthenticationFailed);
        }
        let sd_len = u32::from_be_bytes([inner[0], inner[1], inner[2], inner[3]]) as usize;
        let sd_end =
            lp_range_end(4, sd_len, inner.len()).ok_or(GroupError::AeadAuthenticationFailed)?;
        let sender_did = inner[4..sd_end].to_vec();

        // Decrypt the bulk body (shared AAD = aad_version + codepoint + cid).
        let mut body_aad = Vec::new();
        body_aad.push(AAD_VERSION);
        body_aad.extend_from_slice(&MEMBERSHIP_SET_GROUP_MULTI_STANZA.to_be_bytes());
        body_aad.extend_from_slice(&env.body_cid);
        let body_env = AeadEnvelope::from_wire_bytes(&env.body_wire)
            .map_err(|_| GroupError::AeadAuthenticationFailed)?;
        let body_v2 = benten_crypto_suite::aead::unwrap(&body_env, &cek_key, &body_aad)
            .map_err(|_| GroupError::AeadAuthenticationFailed)?;

        // Parse body_v2 = sig_codepoint(u16) ‖ lp_u32(sender_sig) ‖ body.
        let mut off = 0usize;
        if body_v2.len() < off + 2 {
            return Err(GroupError::AeadAuthenticationFailed);
        }
        let sig_codepoint = u16::from_be_bytes([body_v2[off], body_v2[off + 1]]);
        off += 2;
        if body_v2.len() < off + 4 {
            return Err(GroupError::AeadAuthenticationFailed);
        }
        let sig_len = u32::from_be_bytes([
            body_v2[off],
            body_v2[off + 1],
            body_v2[off + 2],
            body_v2[off + 3],
        ]) as usize;
        off += 4;
        let sig_end = lp_range_end(off, sig_len, body_v2.len())
            .ok_or(GroupError::AeadAuthenticationFailed)?;
        let sender_sig = body_v2[off..sig_end].to_vec();
        off = sig_end;
        let body = body_v2[off..].to_vec();

        // B2 CONTENT-SPLICE GUARD (F-01, SOUNDNESS-CRITICAL): M_auth binds the
        // body ONLY through `env.body_cid`; a co-member holding the K_Set-
        // derived CEK can re-seal a DIFFERENT body under the VICTIM's real
        // `sender_sig` + the ORIGINAL (unchanged) `body_cid`, and the origin-
        // auth verify below — which consumes the WIRE `env.body_cid` — would
        // otherwise pass. Recompute the canonical CID from the RECOVERED body
        // (SAME derivation as the seal side: `self_describing_cid(BLAKE3(body))`)
        // and fail-closed unless it is byte-equal to the wire `env.body_cid`.
        let recomputed_cid = self_describing_cid(blake3::hash(&body).as_bytes());
        if recomputed_cid != env.body_cid {
            return Err(GroupError::SenderOriginAuthFailed);
        }

        // B2 ORIGIN-AUTH VERIFY (F-2 + F-3 SOUNDNESS-CRITICAL): re-derive
        // M_auth from the recovered sender-DID + the recipient's OWN held
        // roster + held generations (NOT the wire) and cryptographically
        // verify the hybrid signature. A re-targeted body (different member
        // set) flips the commitment; a stale-generation body (revoked-member
        // cross-generation replay) flips a generation word — either makes the
        // recomputed M_auth differ from the signed one → fail-closed.
        let body_aad_digest = *blake3::hash(&body_aad).as_bytes();
        let independent_commitment = audience_set_commitment_str(&ctx.member_dids);
        let generations = [
            ctx.member_key_generation,
            ctx.membership_set_generation,
            ctx.role_assignments_generation,
        ];
        let m_auth = super::build_m_auth(&super::SenderAuthBinding {
            sig_codepoint,
            envelope_codepoint: MEMBERSHIP_SET_GROUP_MULTI_STANZA,
            sender_did: &sender_did,
            body_cid: &env.body_cid,
            audience_commitment: &independent_commitment,
            generations: &generations,
            stanza_count: env.stanza_count,
            body_aad_digest,
        });
        super::verify_m_auth(sig_codepoint, &sender_did, &m_auth, &sender_sig)
            .map_err(|()| GroupError::SenderOriginAuthFailed)?;

        Ok((body, sender_did))
    }

    impl GroupSealedEnvelope {
        /// **Test-only relay-truncation model (F-01):** drop the LAST delivered
        /// stanza WITHOUT touching the bound `stanza_count` — exactly what an
        /// active relay does when it censors a co-recipient. A correct
        /// [`open_membership_set_group`] MUST then fail closed with
        /// [`GroupError::StanzaCountMismatch`]. Reverting the open-path count
        /// check makes the truncated open PASS THROUGH — that is the
        /// would-FAIL-on-revert demonstration the F-01 negative test asserts.
        #[cfg(any(test, feature = "testing"))]
        #[must_use]
        pub fn with_last_stanza_dropped_for_test(&self) -> Self {
            let mut truncated = self.clone();
            truncated.stanzas.pop();
            // `stanza_count` is left UNCHANGED — the survivors still name the
            // original count, which no longer matches the delivered length.
            truncated
        }

        /// **Test-only accessor (F-01):** the number of stanzas actually present
        /// (after any relay truncation).
        #[cfg(any(test, feature = "testing"))]
        #[must_use]
        pub fn stanza_len_for_test(&self) -> usize {
            self.stanzas.len()
        }

        /// **Test-only accessor (F-01):** the `stanza_count` bound into every
        /// stanza's AAD.
        #[cfg(any(test, feature = "testing"))]
        #[must_use]
        pub fn bound_stanza_count_for_test(&self) -> u32 {
            self.stanza_count
        }

        /// **Test-only accessor (CONF-1):** re-derive the per-message group
        /// bulk-CEK from `k_set` + the inner `sender_did` + the WIRE `cid`
        /// (`self.body_cid`), using the SAME `derive_group_cek` the live seal
        /// calls. Exposes the property that drives the CONF-1 nonce-reuse fix:
        /// two distinct sends from the SAME sender under the SAME K_Set
        /// generation MUST yield DISTINCT CEKs (because their `cid` differs).
        /// Reverting the `cid`-mix makes the two re-derived CEKs byte-identical
        /// — the would-FAIL-on-revert demonstration the CONF-1 test asserts.
        /// Also demonstrates recipient-recomputability: the CEK is recoverable
        /// from inputs a member already holds (K_Set) plus the wire `cid`.
        #[cfg(any(test, feature = "testing"))]
        #[must_use]
        pub fn derive_cek_for_test(&self, k_set: &[u8; 32], sender_did: &[u8]) -> [u8; 32] {
            derive_group_cek(k_set, sender_did, &self.body_cid)
        }

        /// **Test-only accessor (F-02 / M-20):** the actual canonical per-stanza
        /// AAD bytes the live seal bound for stanza `idx` (the BLINDED 11-field
        /// set). Used to assert byte-equality against the local
        /// [`assemble_group_aad_local`] bytes AND (TEST-ONLY cross-check) the
        /// canonical `benten_membership_set::aad::assemble_group_aad` golden.
        #[cfg(any(test, feature = "testing"))]
        #[must_use]
        pub fn stanza_aad_for_test(&self, idx: usize) -> Vec<u8> {
            self.stanzas[idx].aad.clone()
        }

        /// **Test-only content-splice model (F-01):** a co-member B who holds
        /// `k_set` captures the VICTIM A's HONEST send, KEEPS A's real
        /// `sender_sig` + the ORIGINAL (unchanged) `body_cid`, but re-seals a
        /// DIFFERENT `new_body` under the K_Set-derived CEK. The wire `body_cid`
        /// and every stanza (carrying `sender_did = A` + A's sig) are left
        /// byte-unchanged; only the bulk-body ciphertext is rebuilt. Without the
        /// F-01 content-splice guard, [`open_membership_set_group`] would recover
        /// `new_body`, rebuild M_auth from the WIRE `body_cid` (A's original), and
        /// verify A's real sig — accepting the substituted body attributed to A.
        /// `sender_did` MUST be A's DID (the CEK is keyed by it).
        #[cfg(any(test, feature = "testing"))]
        #[must_use]
        pub fn with_spliced_body_for_test(
            &self,
            k_set: &[u8; 32],
            sender_did: &[u8],
            new_body: &[u8],
        ) -> Self {
            // Rederive the per-send CEK exactly as the seal side does — now
            // INCLUDING the wire `cid` (CONF-1). The CEK binds `self.body_cid`,
            // which the splice keeps byte-unchanged, so a co-member holding
            // K_Set re-derives the IDENTICAL CEK and can unwrap the honest body.
            let cek = derive_group_cek(k_set, sender_did, &self.body_cid);
            let cek_key =
                AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, &cek);

            // Recover the honest body_v2 = sig_codepoint || lp(sender_sig) ||
            // body so we can KEEP A's real sig + sig_codepoint and swap ONLY the
            // body. The body AAD is unchanged (it binds the original body_cid).
            let mut body_aad = Vec::new();
            body_aad.push(AAD_VERSION);
            body_aad.extend_from_slice(&MEMBERSHIP_SET_GROUP_MULTI_STANZA.to_be_bytes());
            body_aad.extend_from_slice(&self.body_cid);
            let honest_env = AeadEnvelope::from_wire_bytes(&self.body_wire)
                .expect("honest body envelope must parse");
            let honest_v2 = benten_crypto_suite::aead::unwrap(&honest_env, &cek_key, &body_aad)
                .expect("co-member holding K_Set can unwrap the honest body");
            // Parse off sig_codepoint(u16) || lp_u32(sender_sig); the trailing
            // bytes are the original body (discarded — we substitute new_body).
            let sig_len =
                u32::from_be_bytes([honest_v2[2], honest_v2[3], honest_v2[4], honest_v2[5]])
                    as usize;
            let sig_prefix_end = 6 + sig_len; // 2 (codepoint) + 4 (lp) + sig
            let captured_sig_region = honest_v2[..sig_prefix_end].to_vec();

            // Rebuild body_v2 with A's captured sig-region + the SUBSTITUTED body,
            // re-sealed under the same CEK with the SAME (original body_cid) AAD.
            let mut spliced_v2 = captured_sig_region;
            spliced_v2.extend_from_slice(new_body);
            let spliced_env = benten_crypto_suite::aead::wrap(&spliced_v2, &cek_key, &body_aad)
                .expect("co-member re-seal of the substituted body must succeed");

            let mut spliced = self.clone();
            spliced.body_wire = spliced_env.to_wire_bytes();
            // body_cid + stanzas + stanza_count + wire-header all UNCHANGED.
            spliced
        }
    }

    /// Codepoint dispatch. Feeding `0x6610` bytes to the `0x6520` Layer-C
    /// group arm (or vice versa) MUST strict-reject (no cross-band fallback).
    ///
    /// # Errors
    ///
    /// [`GroupError::WrongGroupCodepoint`] on a cross-band feed.
    pub fn dispatch_group(
        wire: &[u8],
        declared_codepoint: u16,
        arm_codepoint: u16,
    ) -> Result<(), GroupError> {
        let _ = wire;
        if declared_codepoint != arm_codepoint {
            return Err(GroupError::WrongGroupCodepoint {
                got: declared_codepoint,
                expected: arm_codepoint,
            });
        }
        Ok(())
    }
}

// ===========================================================================
// sealed_aad — the DEFAULT (0x6510) on-wire envelope-AAD field-set (F-INV18-1).
// ===========================================================================

/// The DEFAULT (`0x6510`) on-wire Sealed-Sender envelope AAD field-set
/// (F-INV18-1). The canonical union
/// `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`
/// — the sender-DID is bound INSIDE the ciphertext (not here); there is NO
/// `coarse_epoch` (Ben-RULING-#1 + M-14). The residual privacy-metadata
/// under the default is EXACTLY `{audience}`.
pub mod sealed_aad {
    use super::Vec;

    /// Envelope serialization-format version (DISTINCT from the AAD prefix).
    pub const ENVELOPE_FORMAT_VERSION: u8 = super::ENVELOPE_FORMAT_VERSION;
    /// AAD prefix version byte.
    pub const AAD_VERSION: u8 = super::AAD_VERSION;
    /// Sealed-Sender drop codepoint.
    pub const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = super::DROP_TO_RECIPIENT_SEALED_SENDER;

    /// The DEFAULT (`0x6510`) on-wire envelope AAD inputs — the canonical
    /// union, NO sender-DID, NO coarse_epoch.
    #[derive(Clone, Debug)]
    pub struct SealedSenderAad {
        /// AAD prefix version (`AAD_VERSION` = 0x01).
        pub aad_version: u8,
        /// Drop codepoint (`0x6510`).
        pub codepoint: u16,
        /// The recipient audience DID.
        pub audience_did: Vec<u8>,
        /// The self-describing CIDv1 body-CID (36 B).
        pub body_cid: Vec<u8>,
        /// Recipient key-generation (Inv-16; U19).
        pub recipient_key_generation: u32,
    }

    /// The ENUMERABLE field-set of the serialized `0x6510` envelope AAD — the
    /// canonical union (so an impl that adds `sender_did` or re-adds
    /// `coarse_epoch` is caught by an unexpected token).
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn aad_field_set() -> Vec<&'static str> {
        super::vec_static(&[
            "aad_version",
            "codepoint",
            "audience",
            "body_cid",
            "recipient_key_generation",
        ])
    }

    /// The residual privacy-metadata subset of the `0x6510` AAD field-set —
    /// EXACTLY `{audience}` (coarse-epoch removed; framing/binding fields are
    /// not privacy metadata).
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn residual_privacy_metadata() -> Vec<&'static str> {
        super::vec_static(&["audience"])
    }

    /// Serialize the DEFAULT (`0x6510`) on-wire AAD to its canonical
    /// BIG-ENDIAN bytes. DETERMINISTIC. NO sender-DID, NO coarse_epoch.
    /// Layout (BE; M-19): `aad_version u8 | codepoint u16 | aud_len u32 |
    /// audience_did | body_cid (36 B) | recipient_key_gen u32`.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn serialize_sealed_sender_aad(aad: &SealedSenderAad) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(aad.aad_version);
        out.extend_from_slice(&aad.codepoint.to_be_bytes());
        let aud_len = u32::try_from(aad.audience_did.len()).expect("audience DID length fits u32");
        out.extend_from_slice(&aud_len.to_be_bytes());
        out.extend_from_slice(&aad.audience_did);
        out.extend_from_slice(&aad.body_cid);
        out.extend_from_slice(&aad.recipient_key_generation.to_be_bytes());
        out
    }
}

/// Helper: build a `Vec<&'static str>` from a slice (alloc-only crate).
fn vec_static(items: &[&'static str]) -> Vec<&'static str> {
    items.to_vec()
}

#[cfg(test)]
mod domain_registry_mirror {
    /// C-01/C-02 drift defense: `SENDER_AUTH_DOMAIN` is mirrored in the central
    /// [`benten_crypto_suite::domain_registry`] corpus table over which the
    /// prefix-free collision check runs. Pin byte-equality so the mirror can
    /// never silently diverge from this home definition.
    #[test]
    fn sender_auth_domain_matches_central_registry() {
        assert_eq!(
            super::SENDER_AUTH_DOMAIN,
            benten_crypto_suite::domain_registry::SENDER_AUTH_DOMAIN,
            "SENDER_AUTH_DOMAIN drifted from the central domain_registry mirror"
        );
    }

    /// Drift defense for the Layer-C single/group CEK derivation contexts — each
    /// is a registered cross-surface domain-separation tag in the central
    /// [`benten_crypto_suite::domain_registry`] table over which the prefix-free
    /// invariant runs. Pin byte-equality so a mirror can never silently diverge.
    #[test]
    fn layer_c_cek_contexts_match_central_registry() {
        use benten_crypto_suite::domain_registry as reg;
        assert_eq!(
            super::LAYER_C_CEK_CONTEXT,
            reg::LAYER_C_CEK_CONTEXT,
            "LAYER_C_CEK_CONTEXT drifted from the central domain_registry mirror"
        );
        assert_eq!(
            super::LAYER_C_GROUP_CEK_CONTEXT,
            reg::LAYER_C_GROUP_CEK_CONTEXT,
            "LAYER_C_GROUP_CEK_CONTEXT drifted from the central domain_registry mirror"
        );
        assert_eq!(
            super::group_posture::MEMBERSHIP_GROUP_CEK_CONTEXT,
            reg::MEMBERSHIP_GROUP_CEK_CONTEXT,
            "MEMBERSHIP_GROUP_CEK_CONTEXT drifted from the central domain_registry mirror"
        );
    }
}

#[cfg(test)]
mod group_roster_cardinality_guard {
    use super::{LayerCError, MAX_LAYER_C_GROUP_RECIPIENTS, validate_group_roster_len};

    /// R12 F-11: the `0x6520` group-seal roster-cardinality validator accepts
    /// exactly up to the wire-frozen `u16` ceiling and typed-rejects one over.
    ///
    /// would-FAIL-on-revert: before this change there was NO validator — a
    /// roster of `MAX_LAYER_C_GROUP_RECIPIENTS + 1` flowed straight into
    /// `plaintext_aad_bytes`'s `u16::try_from(...).expect(...)` and PANICKED
    /// inside the seal. This test drives the choke-point directly (no keygen)
    /// and asserts a typed `LayerCError::RecipientCountExceedsBandWidth` at
    /// the boundary, not a panic.
    #[test]
    fn accepts_at_limit_and_rejects_over_limit() {
        // At the ceiling (65535): OK.
        assert_eq!(
            validate_group_roster_len(MAX_LAYER_C_GROUP_RECIPIENTS),
            Ok(()),
            "a roster exactly at MAX_LAYER_C_GROUP_RECIPIENTS must be accepted"
        );

        // One over the ceiling (65536): typed error carrying count + max.
        assert_eq!(
            validate_group_roster_len(MAX_LAYER_C_GROUP_RECIPIENTS + 1),
            Err(LayerCError::RecipientCountExceedsBandWidth {
                count: 65_536,
                max: 65_535,
            }),
            "a roster over MAX_LAYER_C_GROUP_RECIPIENTS must typed-reject, never panic"
        );
    }

    /// R18 C2: the `0x6610` `seal_membership_set_group` MembershipSet seal
    /// MUST route its roster through the same ceiling — an over-band roster
    /// returns [`LayerCError::RecipientCountExceedsBandWidth`] at the seal
    /// boundary rather than reaching the infallible `u16::try_from` inside the
    /// group-AAD assembly (which would PANIC on a 65_536-wide roster).
    ///
    /// would-FAIL-on-revert: before R18 C2, `seal_membership_set_group` was
    /// infallible (`-> GroupSealedEnvelope`) with NO roster gate, unlike its
    /// sibling `seal_group_impl`. A roster of `MAX_LAYER_C_GROUP_RECIPIENTS + 1`
    /// flowed into `group_aad_inputs` → `assemble_group_aad_local`'s
    /// `u16::try_from(recipient_count)` and panicked. Reverting the ceiling
    /// line restores that panic; this test then aborts instead of asserting a
    /// typed `Err`.
    ///
    /// The roster is ONE honest committed [`RecipientBinding`](super::RecipientBinding)
    /// cloned to the over-band length: the ceiling is the FIRST action in the
    /// seal, so it returns BEFORE any per-binding keying work touches the
    /// (cloned) recipient keys — only the roster LENGTH matters.
    #[test]
    fn seal_membership_set_group_rejects_over_band_roster_with_typed_error() {
        use super::group_posture::{GroupSealParams, seal_membership_set_group};
        use benten_crypto_suite::cipher_suite::{CipherSuite, CipherSuiteCodepoint};

        // ONE honest committed did:benten binding, cloned to exactly
        // `MAX_LAYER_C_GROUP_RECIPIENTS + 1` (65_536). The seal returns at the
        // roster-length ceiling gate before touching the (cloned) keys.
        let kp = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
            .expect("0x647a wire-locked")
            .generate_recipient_keypair();
        let one = super::binding_for_test(kp.public());
        let over_band = MAX_LAYER_C_GROUP_RECIPIENTS + 1;
        let roster: Vec<super::RecipientBinding> = vec![one; over_band];

        let sender_did: super::SenderDid = b"did:key:zR18C2SealCeilingTest".to_vec();
        let sender_kp = benten_crypto_suite::sig::SignatureSuite::v1_default().generate_keypair();
        let k_set = [0x11u8; 32];
        let params = GroupSealParams {
            membership_set_id: b"benten:set:r18-c2-ceiling".to_vec(),
            member_key_generation: 1,
            membership_set_generation: 1,
            role_assignments_generation: 1,
        };

        let result = seal_membership_set_group(
            &roster,
            &sender_did,
            &sender_kp,
            &k_set,
            &params,
            b"over-band body",
        );
        assert_eq!(
            result.err(),
            Some(LayerCError::RecipientCountExceedsBandWidth {
                count: over_band,
                max: MAX_LAYER_C_GROUP_RECIPIENTS,
            }),
            "an over-band 0x6610 roster MUST typed-reject at the seal boundary, never panic"
        );
    }
}
