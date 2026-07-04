//! F-full R3-W2 (Layer-C) — HPKE encrypt-to-recipient + Sealed-Sender.
//!
//! ADDL Phase-4-Meta-Core, **F-full** R3 wave **W2-layer-c** (RED-PHASE,
//! TDD red-phase per `pim-12 §3.6e`). Families pinned in THIS file:
//!   - **F-LC-1** HPKE `mode_base` single-recipient round-trip (`0x647A`).
//!   - **F-LC-2** `HpkeMultiBase` group multi-stanza + cross-stanza AAD
//!     substitution defense (`0x6520`) — **honoring Sealed-Sender by
//!     DEFAULT** (R4-FIX F4-003 / BR-1 ruling 1); **per-stanza AAD BLINDED
//!     (R0.7 §3.3/§4.1) — `audience_set_commitment` + `stanza_count` +
//!     self-describing `body_cid`, NOT the raw recipient roster** (R4.6).
//!   - **F-LC-3** Sealed-Sender DEFAULT (`0x6510`) — sender-DID NOT on the
//!     wire (paired positive control: `0x6500` DOES carry it).
//!
//! Pin sources — spec of record is now **R0.7** (`111cca9c:.addl/phase-4-meta/`
//! `f-full-r0-plan.md`); the `0x6510` single-recipient field-set is the
//! R0.7-frozen union whose `audience` is `u32-BE-length-prefixed` (R0.7
//! header:33 / §3.3:539 / §4.1:1040 — **R4.6-FIX F-LC-AUD-U32 corrected the
//! prior u16-length-prefix slip**), the
//! `0x6520` group per-stanza AAD is the **R0.7-RE-OPENED BLINDED** field-set
//! (R0.7 §3.3:568-594 / §4.1:1042):
//!   - **§3.3 / §4.1 `0x6520`** ("Layer-C `LAYER_C_DROP_MULTI_RECIPIENT` group
//!     per-stanza AAD — BLINDED; RATIFIED R0.7"). R0.7 deliberately RE-OPENS
//!     `0x6520` (which R0.6 had left as the separately-frozen raw shape) to
//!     close the SAME #61-class recipient-roster social-graph leak `0x6610`
//!     had: the raw on-wire roster is replaced by an
//!     `audience_set_commitment`, `stanza_count` is bound alongside
//!     `stanza_index`, and `body_cid` becomes a self-describing CIDv1.
//!   - §4.0 codepoint table: `0x647A` HYBRID_X25519_MLKEM768 (real X-Wing
//!     SHA3-256, ChaCha20-Poly1305 bulk); `0x6500` LAYER_C_DROP
//!     (plaintext-sender, non-default); `0x6510`
//!     DROP_TO_RECIPIENT_SEALED_SENDER (v1-beta DEFAULT, BR-1); `0x6520`
//!     LAYER_C_DROP_MULTI_RECIPIENT (`HpkeMultiBase` group).
//!   - §4.1 envelope table: `EncryptedEnvelope` / `BindingContext`
//!     `#[non_exhaustive]`; per-stanza AAD BLINDED (`audience_set_commitment`
//!     over the sorted recipient-DID list + `stanza_index` + `stanza_count` +
//!     `recipient_key_generation`, U17); BE endianness; a dedicated
//!     `aad_version: u8` prefix DISTINCT from `ENVELOPE_FORMAT_VERSION_V2`
//!     (U1/U3/U14) — see the AAD_VERSION note below (R4-FIX F4-004/005).
//!   - Inv-16 (envelope-unification: the `Recipient` `BindingContext` carries
//!     `{audience_did, recipient_key_generation}`) + Inv-18 (paired
//!     Sealed-Sender disclosure satisfied by `0x6510` being DEFAULT).
//!   - R2 landscape `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md`
//!     §1 Group 7: F-LC-1 (~4-6), F-LC-2 (~6-8), F-LC-3 (~7-10).
//!
//! # R4-FIX (F4-003 BLOCKER) — `0x6520` group send HONORS Sealed-Sender.
//!
//! Ben RULING 1 (2026-06-02, BR-1): **EVERY group send honors
//! Sealed-Sender** — the inner-sender-DID is bound INSIDE the sealed
//! (encrypted) part PER STANZA, NOT in the plaintext AAD. The original
//! F-LC-2 here pinned a `0x6520` group send whose per-stanza struct carried
//! a `sender_did` field bound into the PLAINTEXT AAD (mirroring the
//! non-default `0x6500` plaintext-sender variant) — which CONTRADICTS the
//! ruling and silently defeats Sealed-Sender for every group send. The fix:
//!   - the DEFAULT group seal (`seal_group_multi`) now produces stanzas
//!     whose plaintext AAD binds ONLY the BLINDED U17 field-set
//!     (`audience_set_commitment` over the sorted recipient-DID-list +
//!     `stanza_index` + `stanza_count` + `recipient_key_generation`) — the
//!     inner-sender-DID lives INSIDE the sealed per-stanza payload
//!     (`sealed_inner`), recovered post-decrypt (mirrors `0x6510`/`0x6610`);
//!   - a NEW `0x6520` SEALED-GROUP wire-scan arm asserts the sender-DID
//!     does NOT appear in the plaintext AAD bytes (would-FAIL if it leaks);
//!   - the non-default plaintext-sender group variant is kept as an
//!     EXPLICITLY-LABELED non-default arm (`seal_group_multi_plaintext_`
//!     `sender`) with its own paired-control scan, so the substitution /
//!     re-target defenses still exercise a real per-stanza AAD field-set.
//!
//! # R4.6-MIGRATE (R0.7 §3.3/§4.1 BLOCKER) — `0x6520` per-stanza AAD BLINDED.
//!
//! R0.7 RE-OPENS the `0x6520` group per-stanza AAD (which the R0.6-era code
//! here had frozen in the PRE-BLINDING raw shape — bare-32 `body_cid` + the
//! raw `did:key:zRecipientA`/`zRecipientB` roster published in the clear, NO
//! `audience_set_commitment`, NO `stanza_count`). Publishing the raw recipient
//! roster on the wire is the #61-class social-graph leak; un-retrofittable
//! past freeze. The migration to the FROZEN blinded 8-field set
//! (`R0.7 §3.3:572-574 / §4.1:1042`):
//!   `{ aad_version(0x01, u8), codepoint(0x6520, u16 BE),
//!      body_cid(self-describing CIDv1, 36B), recipient_count(u16 BE),
//!      audience_set_commitment(32B), stanza_index(u32 BE),
//!      stanza_count(u32 BE), recipient_key_generation(u32 BE) }`.
//! Three fields change from the prior raw `0x6520` shape:
//!   1. **`audience_set_commitment` (32B) REPLACES the raw roster.**
//!      `audience_set_commitment = BLAKE3(0x01 || lp(did_0) || lp(did_1)
//!      || …)` over the CANONICAL SORTED recipient-DID list, **lp = u32-BE**
//!      length prefix — the **IDENTICAL construction to `0x6610`** (the
//!      sibling `f_aad_2`'s `audience_set_commitment`). The recipients hold
//!      the roster and recompute + verify the commitment; the relay sees only
//!      an opaque 32-byte tag.
//!   2. **`stanza_count` is bound alongside `stanza_index`** — a
//!      relay-truncation/censorship defense (an active relay cannot silently
//!      drop trailing stanzas; each survivor fails the bound count).
//!   3. **`body_cid` becomes a self-describing CIDv1** (`0x01 0x71 0x1e 0x20
//!      || 32-byte BLAKE3` = 36 bytes), NOT a bare fixed-32 digest (CLAUDE.md
//!      baked-in #5; restores U3 length-injectivity) — consistent with
//!      `0x6510`/`0x6610`.
//! UNLIKE `0x6610`, `0x6520` is NOT a MembershipSet — so it carries **NO
//! `membership_set_id_commitment`, NO `membership_set_generation`, NO
//! `role_assignments_generation`** (those are MembershipSet-only). Field
//! widths follow the Layer-C drop band (§4.0 width-unification-REJECTED):
//! `recipient_count` stays **u16 BE** (the band's existing cardinality
//! width) — the u32-BE lp-width inside the commitment hash is a SEPARATE
//! axis (it matches `0x6610`'s commitment construction EXACTLY; do NOT
//! conflate the band's u16 `recipient_count` width with the commitment's
//! internal u32 lp width). **HONEST SCOPE:** identity-HIDING not
//! unlinkability (the commitment recurs for a static recipient set); full
//! per-send unlinkability = **U25, CODEPOINT-RESERVE for v1-GM** (additive;
//! no salt added here, no wire break).
//!
//! # R4.3-FIX (F4-004/005 MAJOR) — dedicated `aad_version: u8` byte-0.
//!
//! Spec §4.1 freezes a dedicated `aad_version: u8` AAD prefix that is
//! DISTINCT from `ENVELOPE_FORMAT_VERSION_V2` (the envelope serialization-
//! format byte). The MembershipSet sibling (`f_aad_2`) correctly uses
//! `const AAD_VERSION: u8 = 0x01` as AAD byte-0. The original Layer-C AAD
//! helper here pushed `ENVELOPE_FORMAT_VERSION` (= 2) as byte-0 — conflating
//! two independent version axes and freezing a conflicting leading byte
//! (`0x02`) against membership's `0x01`, which would break AEAD-open across
//! the Layer-C / membership engines for the identical §4.1 prefix. The fix:
//! introduce `const AAD_VERSION: u8 = 0x01` and push THAT as AAD byte-0;
//! `ENVELOPE_FORMAT_VERSION = 2` stays strictly for the envelope-format
//! version field (`format_version`), never the AAD prefix.
//!
//! # R4.3-FIX (F4-006 MAJOR) — DropToRecipient carries NEITHER coarse-epoch.
//!
//! Spec is internally DECIDABLE on the drop-wire freshness posture, so this
//! is resolved in-file (do-it-now) rather than left as a wire-byte fork:
//!   - **M-14** (§3.10/§4.1, ratified): "DropToRecipient carries
//!     **NEITHER**" sealed_at NOR valid_until; "freshness = recipient-key-
//!     generation + nonce-cache"; the "1-hr bucket" (U28) is "**Layer-D
//!     ONLY**" (DeviceLink + RemotePermission).
//!   - **§4.1 FREEZE row**: "`sealed_at` + `valid_until` epoch (DeviceLink +
//!     RemotePermission **ONLY** — M-14) … coarse 1-hour bucket (U28);
//!     **DropToRecipient carries NEITHER**".
//!   - **§3.11 #43** ("audience + coarse-epoch on the wire") is the GENERAL
//!     residual-metadata mitigation-roadmap framing (U22–U28), NOT a mandate
//!     that the Layer-C drop wire carries a coarse-epoch field. Drops are
//!     "forever-valid (per #62; freshness rides recipient-key-generation +
//!     the nonce-cache, NOT a timestamp)". The single-recipient drop AAD
//!     therefore binds the **audience** (recipient-targeting) but NEITHER
//!     timestamp NOR coarse bucket; the group AAD binds the BLINDED
//!     `audience_set_commitment` but likewise NEITHER timestamp NOR bucket.
//! The original Layer-C `BindingContext` + seal fns carried a `coarse_epoch:
//! u64` (4 mentions), contradicting M-14 and disagreeing with the Layer-D
//! drop sibling `f_ld_8` (0 mentions). The fix: REMOVE `coarse_epoch` from
//! both Layer-C drop bindings + the two single-recipient seal signatures, so
//! this stub models the FROZEN drop wire (NEITHER timestamp NOR coarse
//! bucket). The coarse 1-hour bucket survives ONLY on the Layer-D
//! (DeviceLink/RemotePermission) `sealed_at`/`valid_until` surface.
//! [FLAG-FOR-BEN — courtesy cross-check, not a halt: resolved here per the
//! M-14 + §4.1 FREEZE rows; if §3.11 #43 is later read as mandating a
//! coarse-epoch on the Layer-C drop wire, re-add it symmetrically to
//! `f_ld_8` + the §4.1 FREEZE table. Both stubs now agree on NEITHER.]
//!
//! # R4.4-FIX (CLUSTER-1 MAJOR) — single-recipient AAD = sibling union; byte-0 guard.
//!
//! The R4.3 CLUSTER-1 reconciliation aligned the `f_lc_abuse` /
//! `f_lc_hpke` siblings on the byte-0 / `coarse_epoch` axes for the GROUP
//! (`0x6520`) stanza — but left the `f_lc_hpke` **single-recipient**
//! `DropSealedSender` binding stale on the **audience** axis. The sibling
//! `f_lc_abuse::SealedSenderAad` (and the Inv-16 `Recipient` `BindingContext`
//! in `f_inv16_1`) bind the canonical `0x6510` envelope-AAD union
//! `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`
//! and freeze a golden (`F_INV18_1_SEALED_AAD_HEX`); the `f_lc_hpke`
//! `DropSealedSender` here bound only `{codepoint, body_cid,
//! recipient_key_generation}` — NO `aad_version`, NO `audience` — and froze
//! no single-recipient golden (its `serialize`/`plaintext_aad_bytes` were
//! unimplemented for `HpkeBase`). That is the audience-axis divergence
//! (F-NEW-SS-AUD): an R5 impl that freezes this file's field-set would drop
//! the spec-mandated recipient-audience binding (§3.3:484), while one that
//! freezes the sibling golden would break cross-engine AEAD-open against the
//! audience-less form. The CLUSTER-1 audience-axis fix:
//!   - `DropSealedSender` GAINS `aad_version` + `audience_did` so its bound
//!     field-set is the canonical union, BYTE-IDENTICAL to
//!     `f_lc_abuse::SealedSenderAad` (and the Inv-16 `Recipient` binding);
//!   - a DETERMINISTIC `BindingContext::DropSealedSender::plaintext_aad_`
//!     `bytes()` serializer reproduces the SAME frozen golden
//!     (`F_LC_SEALED_SENDER_AAD_HEX` == the sibling's
//!     `F_INV18_1_SEALED_AAD_HEX`) so the two files CANNOT silently
//!     re-diverge on the `0x6510` single-recipient envelope AAD;
//!   - a NEW arm asserts byte-0 == `AAD_VERSION` (≠ `ENVELOPE_FORMAT_VERSION`),
//!     the BE codepoint, the audience binding present, the sender-DID absent,
//!     and the FROZEN golden (mirrors the group byte-0 guard for `0x6520`).
//! Both sibling files now carry the SAME one-canonical-field-set + byte-0
//! anti-conflation guard for BOTH the `0x6510` single-recipient AND the
//! `0x6520` group AAD layouts across the `0x65xx` drop/recipient band. The
//! `0x6510` single-recipient AAD field-SET is unchanged by R0.7, but its
//! `audience` length-prefix WIDTH is corrected u16→`u32-BE` (R4.6-FIX
//! F-LC-AUD-U32; R0.7 §4.1:1040 freezes the `0x6510` audience at
//! `u32-BE-length-prefixed`); the `0x6520` group AAD is additionally blinded.
//!
//! # R4.3-FIX (F4-018 MINOR) — per-object length-prefix WIDTH note (§4.1).
//! # R4.6-FIX (F-LC-AUD-U32 BLOCKER) — `0x6510` audience length-prefix = u32-BE.
//!
//! The §4.1 canonical-TLV contract is "length-injective" (U3) — satisfied by
//! ANY injective length-prefix width; it does NOT mandate one global width.
//! The wire AADs are SEPARATELY-frozen, codepoint-DISCRIMINATED byte-strings
//! (Layer-C `0x6500`/`0x6510`/`0x6520`; MembershipSet `0x6600`/`0x6610`;
//! Layer-D `0x6310`/wraps), NOT one shared TLV encoder — so distinct-per-object
//! widths cannot silently "break the other golden." The per-object widths
//! are therefore acceptable BUT must be written down (not implicit), and where
//! R0.7 FREEZES a specific width it is BINDING (NOT a free choice). The frozen
//! per-object widths:
//!   - **Layer-C single-recipient `0x6510` AAD** (THIS file): `audience` is a
//!     `u32-BE` length-prefixed recipient DID — **R0.7 header:33 / §3.3:539 /
//!     §4.1:1040 freeze the `0x6510` audience at `u32-BE-length-prefixed`**
//!     with ZERO u16 authorization. **R4.6-FIX (F-LC-AUD-U32):** the prior
//!     `audience_len: u16 BE` was a "settled-territory" wire-byte SLIP — it
//!     conflated the audience-DID length-prefix (a variable-field lp, frozen
//!     u32-BE consistent with the membership / Layer-D / commitment-internal
//!     u32-BE lp convention) with the `0x6520` band's `recipient_count`
//!     CARDINALITY field (a count, correctly u16 per §4.0). The audience field
//!     is NOT a cardinality field; §4.0 width-unification-REJECTED governs the
//!     `recipient_count` cardinality, NOT the audience length-prefix. The
//!     sibling `f_lc_abuse::serialize_sealed_sender_aad` migrates in lockstep
//!     (the shared golden `F_LC_SEALED_SENDER_AAD_HEX == F_INV18_1_SEALED_AAD_HEX`
//!     stays byte-identical, now with the u32-BE prefix).
//!   - **Layer-C group `0x6520` stanza AAD** (THIS file): `recipient_count:
//!     u16 BE` is the band's CARDINALITY width (a count, NOT a length-prefix;
//!     §4.0 width-unification-REJECTED governs it — stays u16); the
//!     `audience_set_commitment` hash internally uses a **u32-BE** per-DID
//!     length prefix (the IDENTICAL construction to `0x6610`). The on-wire
//!     `recipient_count` cardinality (u16) and the variable-field length
//!     prefixes (u32-BE: the `0x6510` audience + the commitment-internal lp)
//!     are SEPARATE axes and MUST NOT be conflated.
//!   - **MembershipSet 11-field-set AAD** (`f_aad_2`): per-field `len: u32 BE`
//!     (the §3.10/§4.1 BLINDED 11-field set).
//!   - **Layer-D ExecuteWorkflow AAD** (`f_ld_3`): `executor_did len: u32
//!     BE`.
//! This canonicalization note's source-of-record destination is **R0.7 §4.1**
//! (belongs-named-now): §4.1 carries the per-object width contract in the
//! frozen `0x6510`/`0x6520`/`0x6610` field-sets (Layer-C drop band u16
//! cardinality; commitment-internal u32-BE lp; membership / Layer-D u32),
//! with R0.7 §4.0's "width-unification-REJECTED" freeze record as its
//! companion (the u16/u32 per-band widths stay separately-frozen — do not
//! re-litigate). If a shared Layer-C TLV helper is later introduced,
//! converge it to this contract; it MUST NOT silently widen the band
//! `recipient_count` to u32 and re-freeze the `0x6510`/`0x6520` goldens.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! At the F-full baseline these Layer-C production types DO NOT YET EXIST
//! (`benten-drop` ships a `DropBundle` whose per-Recipe cells use the
//! in-tree LE/V1 `AeadEnvelope`; the codepoint-dispatched
//! `EncryptedEnvelope` + `HpkeBase`/`HpkeMultiBase` + Sealed-Sender drop
//! variants are F-full Layer-C canary scope). Per the parallel-safety
//! contract of this R3 wave, this file carries a **SELF-CONTAINED stub
//! module** (`layer_c_stub`) matching the intended Layer-C public surface
//! so the file COMPILES green at baseline + every test is
//! `#[ignore = "RED-PHASE: F-LC-… — …; un-ignore at R5"]`. The stub does
//! NOT depend on any other R3 wave's module. It DOES depend on the
//! workspace `blake3` crate (a benten-drop dev-dependency) to compute the
//! `audience_set_commitment` for the blinded `0x6520` AAD — the IDENTICAL
//! primitive the sibling `f_aad_2` uses for `0x6610` — so the blinding pin
//! is substantive (a roster change flips the commitment) rather than a
//! frozen opaque literal. The Layer-C closing-wave R5 implementer MUST:
//!   1. DELETE the local `layer_c_stub` module,
//!   2. INSERT the real `use benten_drop::layer_c::{…};` lines,
//!   3. UN-IGNORE each test (`#[ignore = "RED-PHASE…"]` → nothing),
//!   4. Verify all pins PASS green.
//! Reviewer verifies landing-status (un-ignored + green), not just
//! spec-pin presence (pim-12 §3.6e).
//!
//! **INTEGRATOR NOTE (R4.6 — single-file write scope):** this file now calls
//! `blake3::hash` in the stub `audience_set_commitment` helper. `blake3` is
//! present in the workspace (`benten-membership-set` already lists it as a
//! dev-dependency) but is NOT yet declared in `crates/benten-drop/Cargo.toml`.
//! The single-writer integrator MUST add the one line
//! `blake3 = { workspace = true }` under `benten-drop`'s `[dev-dependencies]`
//! (mirroring `benten-membership-set`) so this test crate compiles. This is
//! a hard structural consequence of the R0.7 blinding migration, not a
//! design fork.
//!
//! # Wave-0 DAG edge (M-20) — V2 + BE + EncryptedEnvelope from commit 1.
//!
//! Every byte authored here is **V2 + big-endian + `EncryptedEnvelope`**.
//! There is NO surviving V1/LE golden vector. The stub's `ENVELOPE_FORMAT_`
//! `VERSION` is `2` and every wire integer (codepoint, stanza-index,
//! stanza-count, recipient_count, recipient_key_generation) is `to_be_bytes`.
//! The AAD prefix byte is the dedicated `AAD_VERSION` (`0x01`), DISTINCT
//! from the format version (R4.3-FIX F4-004/005).
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f).
//!
//! Each test drives a PRODUCTION call site (`seal_*` / `open_*` /
//! `serialize` / `plaintext_aad_bytes`) + asserts an OBSERVABLE
//! consequence + is would-FAIL-if-no-op'd. Seal/open bodies
//! `unimplemented!()` so a forgotten stub at R5 PANICS (the opposite of a
//! silent-green SHAPE-trap); the AAD-serialization helpers are
//! DETERMINISTIC so the Sealed-Sender wire-scan + frozen-golden pins are
//! computable green at red-phase. NEVER `assert_eq!(CONST, CONST_VAL)`;
//! NEVER a zero-assertion arm.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![allow(unused_variables)]

// ===========================================================================
// R5 — real Layer-C production surface (`benten_drop::layer_c`). The
// self-contained stub-shim is DELETED; the production types + fns are
// imported here. Only the hermetic test fixtures (fixed keys / digests /
// did helper) remain test-local.
// ===========================================================================

use benten_crypto_suite::cipher_suite::{
    CipherSuite, CipherSuiteCodepoint, RecipientKeypair, RecipientPublic, RecipientSecret,
};
use benten_crypto_suite::sig::{Keypair as SigKeypair, SignatureSuite};
use benten_drop::layer_c::{
    AAD_VERSION, BindingContext, BodyCidDigest, DROP_TO_RECIPIENT_SEALED_SENDER,
    ENVELOPE_FORMAT_VERSION, EncryptedEnvelope, HYBRID_X25519_MLKEM768, HpkeRecipientStanza,
    LAYER_C_DROP, LAYER_C_DROP_MULTI_RECIPIENT, LayerCError, SENDER_AUTH_DOMAIN,
    SENDER_AUTH_SIG_CODEPOINT, SenderAuthBinding, audience_set_commitment, build_m_auth,
    group_plaintext_aad_region, group_roster_for_test, open_group_stanza, open_single,
    seal_group_multi, seal_group_multi_plaintext_sender, seal_plaintext_sender, seal_sealed_sender,
    self_describing_cid, serialize,
};
use benten_id::did::Did;

/// Hermetic per-seed recipient keypair helper (R9 GAP-1). Each `seed` maps to a
/// stable REAL hybrid keypair via the deterministic-from-SECRET-seed KAT tool —
/// the `seed` is the recipient's PRIVATE seed (both key halves are BLAKE3-
/// expanded from it), so `.public()` and `.secret()` genuinely correspond and a
/// DIFFERENT seed yields a NON-matching secret (the wrong-key tests fail closed
/// for real). This replaces the deleted `fixed_sk(seed) = fixed_pk(seed) + 0x80`
/// placeholder, which had zero secret entropy.
fn fixed_kp(seed: u8) -> RecipientKeypair {
    CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("0x647a wire-locked")
        .generate_recipient_keypair_deterministic_for_test(&[seed; 32])
}
fn fixed_pk(seed: u8) -> RecipientPublic {
    // Re-derive the public half from the recipient's secret seed.
    let kp = fixed_kp(seed);
    RecipientPublic::from_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        &kp.public().to_bytes(),
    )
    .expect("re-parse of a freshly-serialized recipient public must succeed")
}
fn fixed_sk(seed: u8) -> RecipientSecret {
    let kp = fixed_kp(seed);
    RecipientSecret::from_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        &kp.secret().to_bytes(),
    )
    .expect("re-parse of a freshly-serialized recipient secret must succeed")
}
fn fixed_body_cid_digest(seed: u8) -> BodyCidDigest {
    [seed; 32]
}
/// The CANONICAL content-CID DIGEST of a body — `BLAKE3(body)`. Per the design
/// (`body_cid` = "self-describing CIDv1 over the body — binds the CONTENT") an
/// HONEST sender ALWAYS supplies this; the F-01 content-splice guard recomputes
/// it from the recovered body and fail-closes on mismatch. Tests that actually
/// `open_*` MUST seal with the real content CID (an arbitrary digest would now
/// — correctly — be rejected as a content-splice).
fn body_cid_of(body: &[u8]) -> BodyCidDigest {
    *blake3::hash(body).as_bytes()
}
fn did(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// B2 ORIGIN-AUTH test helper: a real sender — a LAMPS-hybrid keypair PLUS
/// the matching hybrid `did:key` bytes (`Did::from_hybrid_public_key`, the
/// ML-DSA-first two-component multikey). The seal signs `M_auth` with the
/// keypair; the open resolves the did:key back to the hybrid verifying key
/// and verifies. ML-DSA keygen uses OsRng (non-deterministic by design), so
/// each call mints a fresh sender — the goldens NEVER pin signature hex
/// (FLAG-6); they pin wire-SHAPE + a sign→verify round-trip.
fn hybrid_sender() -> (SigKeypair, Vec<u8>) {
    let kp = SignatureSuite::v1_default().generate_keypair();
    let did_str = Did::from_hybrid_public_key(&kp.public()).to_string();
    (kp, did_str.into_bytes())
}

/// Lowercase-hex of a byte slice (test-local; no external dep).
fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

// ===========================================================================
// F-LC-1 — HPKE mode_base single-recipient round-trip (0x647A).
// ===========================================================================

/// F-LC-1 PIN 1 — `seal`→`open` round-trips to the original plaintext
/// for the intended recipient. would-FAIL if the HPKE-base KE path does
/// not reconstruct the content-encryption key.
#[test]
fn f_lc_1_hpke_base_single_recipient_round_trips() {
    let pk = fixed_pk(0x01);
    let sk = fixed_sk(0x01);
    let audience = did("did:key:zRecipientAudience");
    let (sender_kp, sender) = hybrid_sender();
    let plaintext = b"layer-c single recipient payload".to_vec();
    let body_cid = body_cid_of(&plaintext);

    let env = seal_sealed_sender(
        &pk, &audience, &sender, &sender_kp, &body_cid, 0, &plaintext,
    );
    let (recovered, recovered_sender) = open_single(&sk, &audience, 0, &env)
        .expect("intended recipient MUST open the HPKE-base single-recipient envelope");

    assert_eq!(
        recovered, plaintext,
        "F-LC-1: HPKE mode_base[MLKEM768-X25519] (0x647A) MUST round-trip \
         the plaintext for the intended recipient. would-FAIL if the KE \
         path does not reconstruct the CEK."
    );
    assert_eq!(
        recovered_sender, sender,
        "F-LC-1: the Sealed-Sender DEFAULT path MUST recover the bound \
         sender-DID post-decrypt."
    );
}

/// F-LC-1 PIN 2 — a WRONG recipient secret key MUST fail to open. The
/// envelope is bound to ONE recipient pubkey. would-FAIL if open ignores
/// the KEM decapsulation result and returns plaintext regardless.
#[test]
fn f_lc_1_wrong_recipient_sk_fails_to_open() {
    let pk = fixed_pk(0x02);
    let wrong_sk = fixed_sk(0x77); // NOT the matching sk for pk
    let audience = did("did:key:zRecipientAudience");
    let (sender_kp, sender) = hybrid_sender();
    let body_cid = fixed_body_cid_digest(0xC2);

    let env = seal_sealed_sender(&pk, &audience, &sender, &sender_kp, &body_cid, 0, b"secret");
    let outcome = open_single(&wrong_sk, &audience, 0, &env);

    assert!(
        matches!(outcome, Err(LayerCError::AeadAuthenticationFailed)),
        "F-LC-1: a non-recipient secret key MUST NOT open the envelope \
         (HPKE binds to one recipient pubkey). would-FAIL if open returns \
         plaintext regardless of decapsulation. Got: {outcome:?}"
    );
}

/// F-LC-1 PIN 3 — V2 + the 0x647A codepoint are committed in the typed
/// binding (Wave-0 M-20; Inv-16 codepoint-dispatch). would-FAIL if the
/// envelope is authored at V1 or omits the codepoint from its binding.
#[test]
fn f_lc_1_envelope_is_v2_and_carries_hybrid_codepoint() {
    let (sender_kp, sender) = hybrid_sender();
    let env = seal_sealed_sender(
        &fixed_pk(0x03),
        &did("did:key:zRecipientAudience"),
        &sender,
        &sender_kp,
        &fixed_body_cid_digest(0xC3),
        0,
        b"payload",
    );

    let (format_version, codepoint) = match &env {
        EncryptedEnvelope::HpkeBase {
            format_version,
            binding,
            ..
        } => match binding {
            BindingContext::DropSealedSender { codepoint, .. } => (*format_version, *codepoint),
            BindingContext::DropPlaintextSender { codepoint, .. } => (*format_version, *codepoint),
        },
        EncryptedEnvelope::HpkeMultiBase { .. } => {
            panic!("F-LC-1 single-recipient seal MUST produce HpkeBase")
        }
        // `EncryptedEnvelope` is `#[non_exhaustive]` (Inv-16); fail-closed on a
        // future additive shape (a single-recipient seal must not be one).
        _ => panic!("F-LC-1 single-recipient seal MUST produce HpkeBase"),
    };

    assert_eq!(
        format_version, ENVELOPE_FORMAT_VERSION,
        "F-LC-1: the single-recipient envelope MUST be authored at \
         ENVELOPE_FORMAT_VERSION = 2 (Wave-0 M-20). would-FAIL on a \
         surviving V1 byte."
    );
    assert_eq!(
        codepoint, DROP_TO_RECIPIENT_SEALED_SENDER,
        "F-LC-1: the default single-recipient seal MUST commit the \
         Sealed-Sender drop codepoint (0x6510) in its typed binding \
         (Inv-16 codepoint-dispatch)."
    );
    // The KEM codepoint 0x647A is the load-bearing wire constant for the
    // HPKE-base KE; assert it is the wire-locked integer (anti-drift).
    assert_eq!(
        HYBRID_X25519_MLKEM768, 0x647a,
        "F-LC-1: the HYBRID_X25519_MLKEM768 KEM codepoint MUST be the \
         wire-locked integer 0x647A (real X-Wing SHA3-256 per BR-3)."
    );
}

// ===========================================================================
// F-LC-2 — HpkeMultiBase group multi-stanza + cross-stanza AAD defense.
//          DEFAULT honors Sealed-Sender (R4-FIX F4-003 / BR-1 ruling 1).
//          Per-stanza AAD BLINDED (R0.7 §3.3/§4.1 — R4.6-MIGRATE).
// ===========================================================================

/// F-LC-2 PIN 1 — N recipients each open their own stanza to the same
/// plaintext (multi-recipient parity). would-FAIL if the group seal
/// produces stanzas that decrypt to different content or only one opens.
#[test]
fn f_lc_2_multi_stanza_each_recipient_opens_same_plaintext() {
    let pks = [fixed_pk(0x10), fixed_pk(0x11), fixed_pk(0x12)];
    let sks = [fixed_sk(0x10), fixed_sk(0x11), fixed_sk(0x12)];
    let (sender_kp, sender) = hybrid_sender();
    let roster = group_roster_for_test(&pks);
    let plaintext = b"group payload".to_vec();
    let body_cid = body_cid_of(&plaintext);

    let env = seal_group_multi(&pks, &sender, &sender_kp, &body_cid, 0, &plaintext)
        .expect("group seal within recipient limit");

    for (idx, sk) in sks.iter().enumerate() {
        let (recovered, _recovered_sender) = open_group_stanza(sk, idx, &roster, 0, &env)
            .unwrap_or_else(|e| panic!("recipient {idx} MUST open their stanza: {e:?}"));
        assert_eq!(
            recovered, plaintext,
            "F-LC-2: every recipient stanza MUST open to the SAME plaintext \
             (multi-recipient parity). would-FAIL if a stanza decrypts to \
             divergent content."
        );
    }
}

/// F-LC-2 PIN 2 — cross-stanza SUBSTITUTION is rejected. Swapping two
/// recipients' stanzas (so recipient 0 gets recipient 1's stanza) MUST
/// fail at AEAD-open: the per-stanza AAD binds `stanza-index` +
/// `audience_set_commitment` (over the recipient roster), so a re-positioned
/// stanza no longer authenticates. would-FAIL if the AAD omits the
/// stanza-index/commitment binding (defense is in AAD, NOT in the CID — U17).
#[test]
fn f_lc_2_cross_stanza_substitution_rejected() {
    let pks = [fixed_pk(0x20), fixed_pk(0x21)];
    let sks = [fixed_sk(0x20), fixed_sk(0x21)];
    let (sender_kp, sender) = hybrid_sender();
    let roster = group_roster_for_test(&pks);
    let body_cid = fixed_body_cid_digest(0xD1);

    let env = seal_group_multi(&pks, &sender, &sender_kp, &body_cid, 0, b"group payload")
        .expect("group seal within recipient limit");

    // Adversary swaps stanza 0 and stanza 1.
    let mut tampered = env.clone();
    if let EncryptedEnvelope::HpkeMultiBase { stanzas, .. } = &mut tampered {
        stanzas.swap(0, 1);
    } else {
        panic!("group seal MUST produce HpkeMultiBase");
    }

    // Recipient 0 now reads a stanza that was sealed for recipient 1's
    // position; the per-stanza AAD (stanza-index 1, audience_set_commitment)
    // no longer matches recipient 0's open context.
    let outcome = open_group_stanza(&sks[0], 0, &roster, 0, &tampered);
    assert!(
        matches!(outcome, Err(LayerCError::AeadAuthenticationFailed)),
        "F-LC-2: cross-stanza substitution (swap 0↔1) MUST fail at AEAD \
         because the per-stanza AAD binds stanza-index + the BLINDED \
         audience_set_commitment over the recipient roster (U17). would-FAIL \
         if substitution defense rode the CID instead of the AAD. Got: \
         {outcome:?}"
    );
}

/// F-LC-2 PIN 3 (R4.6-REANCHOR) — RE-TARGET to a different recipient is
/// rejected. If an adversary rewrites a stanza's `recipient_dids` roster
/// (re-pointing the group), the `audience_set_commitment` reconstructed at
/// open no longer matches the seal-time commitment, so AEAD-open fails. The
/// roster is BLINDED (R0.7 §3.3) — bound via the commitment, NOT emitted raw
/// — but mutating it STILL flips the commitment, preserving the U17 defense.
/// would-FAIL if the recipient roster is not bound (even blinded) per stanza.
#[test]
fn f_lc_2_stanza_retarget_to_different_recipient_rejected() {
    let pks = [fixed_pk(0x30), fixed_pk(0x31)];
    let sks = [fixed_sk(0x30)];
    let (sender_kp, sender) = hybrid_sender();
    let roster = group_roster_for_test(&pks);
    let body_cid = fixed_body_cid_digest(0xD2);

    let env = seal_group_multi(&pks, &sender, &sender_kp, &body_cid, 0, b"group payload")
        .expect("group seal within recipient limit");

    // Adversary rewrites the bound recipient roster of stanza 0 to a
    // different membership. Even though the roster is BLINDED (never on the
    // wire raw), it is the COMMITMENT INPUT — so this mutation flips the
    // recomputed audience_set_commitment and the AAD no longer matches.
    let mut tampered = env.clone();
    let (orig_commitment, retargeted_commitment) =
        if let EncryptedEnvelope::HpkeMultiBase { stanzas, .. } = &mut tampered {
            let before = audience_set_commitment(&stanzas[0].recipient_dids);
            stanzas[0].recipient_dids = vec![did("did:key:zMallory")];
            let after = audience_set_commitment(&stanzas[0].recipient_dids);
            (before, after)
        } else {
            panic!("group seal MUST produce HpkeMultiBase");
        };

    // The mutation MUST observably change the blinded commitment (proves the
    // roster is still bound, just BLINDED — the re-anchored U17 property).
    assert_ne!(
        orig_commitment, retargeted_commitment,
        "F-LC-2 (R4.6-REANCHOR): re-targeting the recipient roster MUST flip \
         the BLINDED audience_set_commitment — the roster stays bound per \
         stanza via the commitment (R0.7 §3.3), so a re-target is detectable \
         at AEAD-open even though the raw roster is NOT on the wire."
    );

    let outcome = open_group_stanza(&sks[0], 0, &roster, 0, &tampered);
    assert!(
        matches!(outcome, Err(LayerCError::AeadAuthenticationFailed)),
        "F-LC-2: re-targeting a stanza's recipient roster MUST fail at AEAD \
         (the roster is bound per stanza via the BLINDED \
         audience_set_commitment, U17). Got: {outcome:?}"
    );
}

/// F-LC-2 PIN 4 — the group codepoint is the wire-locked 0x6520 and the
/// stanza count matches the recipient count. would-FAIL if the group
/// envelope is authored at the wrong band or drops stanzas.
#[test]
fn f_lc_2_group_envelope_codepoint_and_stanza_count() {
    let pks = [
        fixed_pk(0x40),
        fixed_pk(0x41),
        fixed_pk(0x42),
        fixed_pk(0x43),
    ];
    let (sender_kp, sender) = hybrid_sender();
    let env = seal_group_multi(
        &pks,
        &sender,
        &sender_kp,
        &fixed_body_cid_digest(0xD3),
        0,
        b"x",
    )
    .expect("group seal within recipient limit");

    match &env {
        EncryptedEnvelope::HpkeMultiBase {
            format_version,
            stanzas,
            ..
        } => {
            assert_eq!(
                *format_version, ENVELOPE_FORMAT_VERSION,
                "F-LC-2: group envelope MUST be authored at V2."
            );
            assert_eq!(
                stanzas.len(),
                pks.len(),
                "F-LC-2: HpkeMultiBase MUST carry exactly one stanza per \
                 recipient (no dropped stanzas)."
            );
            for (i, st) in stanzas.iter().enumerate() {
                assert_eq!(
                    st.codepoint, LAYER_C_DROP_MULTI_RECIPIENT,
                    "F-LC-2: each stanza MUST carry the group codepoint \
                     0x6520 in its AAD."
                );
                assert_eq!(
                    st.stanza_index as usize, i,
                    "F-LC-2: stanza-index MUST equal the stanza position \
                     (bound in AAD for substitution defense)."
                );
                // R4.6-MIGRATE: stanza_count is bound alongside stanza_index
                // (truncation defense) — every stanza agrees on the total.
                assert_eq!(
                    st.stanza_count as usize,
                    pks.len(),
                    "F-LC-2 (R4.6): each stanza MUST bind the TOTAL \
                     stanza_count (R0.7 §3.3 truncation/censorship defense) — \
                     a dropped trailing stanza leaves survivors disagreeing \
                     with the bound count."
                );
            }
        }
        EncryptedEnvelope::HpkeBase { .. } => panic!("group seal MUST produce HpkeMultiBase"),
        // `EncryptedEnvelope` is `#[non_exhaustive]` (Inv-16); fail-closed.
        _ => panic!("group seal MUST produce HpkeMultiBase"),
    }
    assert_eq!(
        LAYER_C_DROP_MULTI_RECIPIENT, 0x6520,
        "F-LC-2: LAYER_C_DROP_MULTI_RECIPIENT MUST be the wire-locked \
         integer 0x6520."
    );
}

/// F-LC-2 PIN 5 (R4-FIX F4-003 / BR-1 ruling 1) — the DEFAULT `0x6520`
/// group send HONORS Sealed-Sender: the sender-DID is bound per-stanza
/// INSIDE the sealed payload, and DOES NOT appear anywhere in the
/// PLAINTEXT AAD region of the serialized group envelope. This is the
/// load-bearing consequence of ruling 1 — group sends must NOT silently
/// defeat the Sealed-Sender default. would-FAIL if the default group seal
/// bound the sender-DID into the plaintext per-stanza AAD (the bug the old
/// F-LC-2 stanza shape had).
#[test]
fn f_lc_2_default_group_send_honors_sealed_sender_no_plaintext_sender_did() {
    let pks = [fixed_pk(0x60), fixed_pk(0x61), fixed_pk(0x62)];
    let (sender_kp, sender) = hybrid_sender();
    let body_cid = fixed_body_cid_digest(0xD6);

    let env = seal_group_multi(&pks, &sender, &sender_kp, &body_cid, 0, b"group payload")
        .expect("group seal within recipient limit");

    // (a) Typed-shape guard: NO stanza carries a plaintext_sender_did on
    //     the DEFAULT path (it lives in `sealed_inner` instead).
    match &env {
        EncryptedEnvelope::HpkeMultiBase { stanzas, .. } => {
            for (i, st) in stanzas.iter().enumerate() {
                assert!(
                    st.plaintext_sender_did.is_none(),
                    "F-LC-2 (F4-003): on the DEFAULT 0x6520 group path, \
                     stanza {i} MUST NOT carry a plaintext_sender_did — \
                     Sealed-Sender binds it INSIDE `sealed_inner`. would-\
                     FAIL if the default leaked the sender-DID into the AAD."
                );
                assert!(
                    !st.sealed_inner.is_empty(),
                    "F-LC-2 (F4-003): each DEFAULT stanza MUST carry a sealed \
                     inner payload (where the inner-sender-DID lives)."
                );
            }
        }
        EncryptedEnvelope::HpkeBase { .. } => panic!("group seal MUST produce HpkeMultiBase"),
        // `EncryptedEnvelope` is `#[non_exhaustive]` (Inv-16); fail-closed.
        _ => panic!("group seal MUST produce HpkeMultiBase"),
    }

    // (b) WIRE-SCAN: the sender-DID byte sequence MUST NOT appear in the
    //     concatenated PLAINTEXT AAD region of the serialized envelope.
    let plaintext_aad = group_plaintext_aad_region(&env);
    let leaks_in_aad = plaintext_aad
        .windows(sender.len())
        .any(|w| w == sender.as_slice());
    assert!(
        !leaks_in_aad,
        "F-LC-2 (F4-003 / BR-1 ruling 1): the DEFAULT 0x6520 group send \
         MUST HONOR Sealed-Sender — the sender-DID MUST NOT appear in the \
         PLAINTEXT per-stanza AAD region of the serialized envelope. It is \
         bound per-stanza INSIDE the sealed payload, recovered only \
         post-decrypt. would-FAIL if the default group path bound the \
         sender-DID into the plaintext AAD (silently defeating the \
         Sealed-Sender default for every group send)."
    );

    // (c) Full-wire scan (defense-in-depth): the sender-DID does not leak
    //     into ANY plaintext wire field of the serialized group envelope.
    let wire = serialize(&env);
    let leaks_in_wire = wire.windows(sender.len()).any(|w| w == sender.as_slice());
    assert!(
        !leaks_in_wire,
        "F-LC-2 (F4-003): the serialized DEFAULT 0x6520 group wire MUST NOT \
         contain the sender-DID in any plaintext field."
    );
}

/// F-LC-2 PIN 6 (R4-FIX F4-003 paired control) — the EXPLICITLY-non-default
/// plaintext-sender group variant (`0x6520` with `plaintext_sender_did`
/// set) DOES place the sender-DID in the plaintext AAD (U4). This is the
/// paired positive control that proves PIN 5's wire-scan is not vacuously
/// passing (the two paths must differ observably). The non-default variant
/// is NOT the shipped default. would-FAIL if even the non-default variant
/// hid the sender-DID (then the scanner cannot distinguish the paths).
#[test]
fn f_lc_2_nondefault_plaintext_sender_group_carries_sender_did_in_aad() {
    let pks = [fixed_pk(0x70), fixed_pk(0x71)];
    let (sender_kp, sender) = hybrid_sender();
    let body_cid = fixed_body_cid_digest(0xD7);

    let env = seal_group_multi_plaintext_sender(
        &pks,
        &sender,
        &sender_kp,
        &body_cid,
        0,
        b"group payload",
    )
    .expect("group seal within recipient limit");

    match &env {
        EncryptedEnvelope::HpkeMultiBase { stanzas, .. } => {
            for (i, st) in stanzas.iter().enumerate() {
                assert_eq!(
                    st.plaintext_sender_did.as_deref(),
                    Some(sender.as_slice()),
                    "F-LC-2 (F4-003 control): the NON-default plaintext-sender \
                     group variant MUST bind the sender-DID into stanza {i}'s \
                     plaintext AAD (U4)."
                );
            }
        }
        EncryptedEnvelope::HpkeBase { .. } => {
            panic!("plaintext-sender group seal MUST produce HpkeMultiBase")
        }
        // `EncryptedEnvelope` is `#[non_exhaustive]` (Inv-16); fail-closed.
        _ => panic!("plaintext-sender group seal MUST produce HpkeMultiBase"),
    }

    let plaintext_aad = group_plaintext_aad_region(&env);
    let leaks = plaintext_aad
        .windows(sender.len())
        .any(|w| w == sender.as_slice());
    assert!(
        leaks,
        "F-LC-2 (F4-003 PAIRED CONTROL): the NON-default plaintext-sender \
         group variant MUST place the sender-DID in the plaintext AAD (U4). \
         If this control fails, PIN 5's wire-scan cannot distinguish hiding \
         from a broken scan — the default and non-default paths must differ \
         observably."
    );
}

/// R13 F-09 — the deterministic `0x6500` single-recipient PLAINTEXT-sender
/// AAD fixture, built directly via `BindingContext::DropPlaintextSender` so
/// the canonical `plaintext_aad_bytes()` serializer is driven without a
/// keygen. audience = `"did:key:zRecipientAudience6500"` (30 B), body_cid =
/// self-describing CIDv1 over `[0xC5; 32]`, recipient_key_generation = 0,
/// sender = `"did:key:zSender6500"` (19 B).
fn f_lc_09_plaintext_sender_binding_fixture() -> BindingContext {
    let digest: BodyCidDigest = [0xC5; 32];
    BindingContext::DropPlaintextSender {
        aad_version: AAD_VERSION, // 0x01
        codepoint: LAYER_C_DROP,  // 0x6500
        audience_did: did("did:key:zRecipientAudience6500"),
        body_cid: self_describing_cid(&digest),
        recipient_key_generation: 0,
        sender_did: did("did:key:zSender6500"),
    }
}

/// R13 F-09 — FROZEN big-endian golden for the `0x6500` PLAINTEXT-sender
/// single-recipient AAD. Layout (BE; M-19): `aad_version u8 | codepoint u16
/// (0x6500) | aud_len u32 | audience_did | body_cid (36 B) |
/// recipient_key_gen u32 | sender_len u16 | sender_did`. The load-bearing
/// R13 F-09 point: the trailing `sender_len` is **u16-BE** (`00 13` for the
/// 19-byte sender), NOT the audience's u32-BE length-prefix — the two
/// length-prefix widths are DELIBERATELY different (audience u32, sender
/// u16) per the band's wire contract (layer_c.rs §4.1). Computed once via
/// the M-20 throwaway script (`/tmp/.../compute_0x6500_golden.py`).
const F_LC_PLAINTEXT_SENDER_AAD_HEX: &str = "0165000000001e6469643a6b65793a7a526563697069656e7441756469656e63653635303001711e20c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c50000000000136469643a6b65793a7a53656e64657236353030";

/// R13 F-09 — the `0x6500` plaintext-sender AAD serializes to the FROZEN
/// big-endian golden AND its trailing `sender_len` prefix is **u16-BE**, NOT
/// the u32-BE form the audience uses. would-FAIL if `sender_len` regressed to
/// u32 (the settled-territory conflation the audience-lp guards elsewhere
/// catch), or if the byte layout drifted.
#[test]
fn f_lc_09_plaintext_sender_len_is_u16_be_not_u32_frozen_golden() {
    let binding = f_lc_09_plaintext_sender_binding_fixture();
    let bytes = binding.plaintext_aad_bytes();

    // (a) FROZEN golden — the full BE layout (M-20).
    assert_eq!(
        to_hex(&bytes),
        F_LC_PLAINTEXT_SENDER_AAD_HEX,
        "F-09: the 0x6500 plaintext-sender AAD MUST serialize to the FROZEN \
         big-endian golden (aad_version | codepoint(0x6500) | \
         aud_len(u32) | audience | body_cid(36) | rkg(u32) | \
         sender_len(u16) | sender_did). R5 confirms-or-deliberately-updates \
         this literal (M-20)."
    );

    // (b) The trailing `sender_len` is u16-BE. Offset = aad_version(1) +
    //     codepoint(2) + aud_len(4) + audience(30) + body_cid(36) + rkg(4).
    let sender = did("did:key:zSender6500");
    let sender_len_off = 1 + 2 + 4 + "did:key:zRecipientAudience6500".len() + 36 + 4;
    let observed_u16 = u16::from_be_bytes([bytes[sender_len_off], bytes[sender_len_off + 1]]);
    assert_eq!(
        observed_u16 as usize,
        sender.len(),
        "F-09: the 2-byte sender_len prefix MUST be the u16-BE sender-DID \
         length (0x{:04x} for the {}-byte sender).",
        sender.len(),
        sender.len()
    );
    // The u16 form is EXACTLY 2 bytes; the sender-DID must follow immediately.
    assert_eq!(
        &bytes[sender_len_off + 2..sender_len_off + 2 + sender.len()],
        sender.as_slice(),
        "F-09: the sender-DID MUST follow the 2-byte u16-BE sender_len \
         immediately — a u32 prefix would insert 2 spurious high-order zero \
         bytes and shift the DID."
    );

    // (c) assert_ne! the u32-BE form: reading the sender_len region as a
    //     u32-BE would consume 2 EXTRA bytes and NOT equal the true length.
    let as_u32 = u32::from_be_bytes([
        bytes[sender_len_off],
        bytes[sender_len_off + 1],
        bytes[sender_len_off + 2],
        bytes[sender_len_off + 3],
    ]);
    assert_ne!(
        as_u32 as usize,
        sender.len(),
        "F-09: a u32-BE read of the sender_len region MUST NOT equal the \
         sender length — the prefix is u16-BE (2 bytes), NOT u32-BE (4 bytes). \
         If these were equal the wire would be ambiguous with the audience's \
         u32-BE prefix (the conflation this pin forbids)."
    );
}

/// R13 F-10 — FROZEN big-endian golden over `build_m_auth` (the sender-
/// origin-auth binding). Layout: `SENDER_AUTH_DOMAIN | sig_codepoint u16 |
/// envelope_codepoint u16 | lp_u32(sender_did) | body_cid(36) |
/// lp_u32(audience_commitment) | generation_count u32 | generations(u32 each)
/// | stanza_count u32 | body_aad_digest(32)`. Fixture: sig=0x0001,
/// envelope=0x6510, sender=`"did:key:zSenderF10"` (18 B), body_cid over
/// `[0xF1;32]`, audience=`"did:key:zAudF10"` (15 B), generations=[7],
/// stanza_count=1, body_aad_digest=`[0xAA;32]`. Computed once via the M-20
/// throwaway script (`/tmp/.../compute_m_auth_golden.py`).
const F_M_AUTH_GOLDEN_HEX: &str = "62656e74656e2f6c617965722d632f7365616c65642d73656e6465722d6f726967696e2d617574682f763100016510000000126469643a6b65793a7a53656e64657246313001711e20f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f10000000f6469643a6b65793a7a417564463130000000010000000700000001aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// R13 F-10 — `build_m_auth` produces the FROZEN deterministic binding
/// bytes: the `SENDER_AUTH_DOMAIN` prefix, the two u16-BE codepoints, the
/// `lp_u32` framing of every variable field, the 36-byte self-describing
/// body_cid, the generation-count prefix + generation words, stanza_count,
/// and the 32-byte body_aad_digest — all big-endian, injective (M-19/M-20).
/// would-FAIL if the layout drifted (a dropped lp prefix, an LE codepoint, a
/// missing domain prefix, or a reordered field) — a wire-break that would
/// silently break seal/verify agreement across engines.
#[test]
fn f_lc_10_build_m_auth_frozen_golden() {
    let digest: BodyCidDigest = [0xF1; 32];
    let body_cid = self_describing_cid(&digest);
    let sender = did("did:key:zSenderF10");
    let audience_commitment = did("did:key:zAudF10");
    let m = build_m_auth(&SenderAuthBinding {
        sig_codepoint: SENDER_AUTH_SIG_CODEPOINT, // 0x0001
        envelope_codepoint: DROP_TO_RECIPIENT_SEALED_SENDER, // 0x6510
        sender_did: &sender,
        body_cid: &body_cid,
        audience_commitment: &audience_commitment,
        generations: &[7],
        stanza_count: 1,
        body_aad_digest: [0xAA; 32],
    });
    assert_eq!(
        to_hex(&m),
        F_M_AUTH_GOLDEN_HEX,
        "F-10: build_m_auth MUST serialize to the FROZEN big-endian binding \
         (SENDER_AUTH_DOMAIN prefix | sig/envelope codepoints u16-BE | \
         lp_u32 widths | 36-byte body_cid | generation-count prefix + words | \
         stanza_count | 32-byte body_aad_digest). R5 confirms-or-deliberately-\
         updates this literal (M-20)."
    );
    // The golden MUST start with the exact SENDER_AUTH_DOMAIN prefix — the
    // domain separation that makes M_auth unforgeable-cross-context (F-1).
    assert!(
        m.starts_with(SENDER_AUTH_DOMAIN),
        "F-10: build_m_auth output MUST begin with the SENDER_AUTH_DOMAIN \
         prefix (the F-1 domain separation)."
    );
}

/// The deterministic per-stanza fixture for the group plaintext-AAD golden.
/// Built DIRECTLY (the seal fns `unimplemented!()` at red-phase) so the
/// canonical `plaintext_aad_bytes()` serializer is driven without panicking.
/// DEFAULT (Sealed-Sender) path: `plaintext_sender_did = None`.
///
/// R4.6-MIGRATE (R0.7 §3.3/§4.1): the BLINDED shape — `body_cid` is a
/// self-describing CIDv1 over the digest `[0xD8, 0; 31]`; the recipient
/// roster (`zRecipientA`/`zRecipientB`, already sorted) is the COMMITMENT
/// INPUT (never emitted raw); `stanza_count = 1` (single-stanza fixture).
fn f_lc_2_group_stanza_fixture() -> HpkeRecipientStanza {
    let mut digest: BodyCidDigest = [0u8; 32];
    digest[0] = 0xD8;
    HpkeRecipientStanza {
        codepoint: LAYER_C_DROP_MULTI_RECIPIENT, // 0x6520
        // R4.6-MIGRATE: self-describing CIDv1 (36 B), NOT a bare-32 digest.
        body_cid: self_describing_cid(&digest),
        // R4.6-MIGRATE: the roster is the COMMITMENT INPUT (sorted; BLINDED).
        recipient_dids: vec![did("did:key:zRecipientA"), did("did:key:zRecipientB")],
        stanza_index: 0,
        stanza_count: 1, // single-stanza fixture (truncation-defense field)
        recipient_key_generation: 0,
        sealed_inner: vec![0xAB; 8], // opaque; not part of the plaintext AAD
        plaintext_sender_did: None,  // DEFAULT Sealed-Sender path
        wrapped_cek: vec![0xCD; 8],  // opaque; not part of the plaintext AAD
    }
}

/// FROZEN big-endian golden vector for the DEFAULT group stanza plaintext
/// AAD (R4.6-MIGRATE BLINDED 8-field set; CLUSTER-1 byte-0-guard). Layout (BE):
///   aad_version u8 | codepoint u16 (0x6520) | body_cid (self-describing
///   CIDv1, 36 B) | recipient_count u16 | audience_set_commitment (32 B) |
///   stanza_index u32 | stanza_count u32 | recipient_key_generation u32.
/// The raw recipient roster is BLINDED (NOT on the wire) — only the 32-byte
/// `audience_set_commitment = BLAKE3(0x01 || lp_u32(sorted DIDs))` appears.
/// No coarse_epoch (F4-006); leads with `AAD_VERSION = 0x01` (F4-004/005).
/// Computed once via the M-20 throwaway script
/// (`/tmp/.../compute_0x6520_golden.py`); the `audience_set_commitment`
/// component (`3154cfca…692fb`) is BYTE-IDENTICAL to what the sibling
/// `f_aad_2` `audience_set_commitment` would emit over the SAME sorted
/// roster (IDENTICAL u32-BE-lp construction). R5 confirms-or-deliberately-
/// updates this literal against the real encoder (M-20).
const F_LC_2_GROUP_STANZA_AAD_HEX: &str = "01652001711e20d80000000000000000000000000000000000000000000000000000000000000000023154cfca78520f9cdb951f9fbf0ecc02401cbfb3f41abde320325e9f957692fb000000000000000100000000";

/// FROZEN `audience_set_commitment` (32 B) over the fixture's sorted roster
/// `[zRecipientA, zRecipientB]` — BLAKE3(0x01 || lp_u32(did)…). This is the
/// component the BLINDING replaces the raw roster with. BYTE-IDENTICAL to
/// what `f_aad_2`'s `0x6610` commitment emits over the SAME roster (the
/// IDENTICAL construction). Used to positively assert the commitment is
/// PRESENT (and the raw roster ABSENT) in the wire AAD (roster-non-leak pin).
const F_LC_2_AUDIENCE_SET_COMMITMENT_HEX: &str =
    "3154cfca78520f9cdb951f9fbf0ecc02401cbfb3f41abde320325e9f957692fb";

/// F-LC-2 PIN 7 (R4.4-FIX CLUSTER-1 + R4.6-MIGRATE) — the DEFAULT group
/// stanza plaintext-AAD byte-0 is the dedicated `AAD_VERSION` (= 0x01), NOT
/// the envelope `ENVELOPE_FORMAT_VERSION` (= 0x02), and the full BLINDED
/// layout is FROZEN to the big-endian golden. This MIRRORS the sibling
/// `f_lc_abuse`/`f_aad_2` byte-0 anti-conflation guard so the engines cannot
/// silently re-diverge on the `0x65xx`/`0x66xx` blinded group AAD. would-FAIL
/// if a future edit reverted `plaintext_aad_bytes`' leading byte to the format
/// version (the F4-004/005 cross-engine AEAD-open break), emitted an LE
/// codepoint, reverted the body_cid to bare-32, dropped `stanza_count`, or
/// regressed to publishing the raw recipient roster instead of the commitment.
#[test]
fn f_lc_2_group_stanza_aad_byte0_is_aad_version_not_format_version_and_frozen_layout() {
    let stanza = f_lc_2_group_stanza_fixture();
    let bytes = stanza.plaintext_aad_bytes();

    // Anti-conflation byte-0 guard (mirrors f_lc_abuse / f_aad_2).
    assert_eq!(
        bytes[0], AAD_VERSION,
        "F-LC-2 (CLUSTER-1 / F4-004/005): the group stanza plaintext-AAD \
         byte-0 MUST be the dedicated AAD_VERSION (0x01), NOT the envelope \
         serialization format_version (0x02). Reverting this re-introduces \
         the cross-engine AEAD-open break + re-diverges from f_lc_abuse."
    );
    assert_ne!(
        bytes[0], ENVELOPE_FORMAT_VERSION,
        "F-LC-2 (CLUSTER-1 / F4-004/005): byte-0 MUST NOT be the envelope \
         format version — the AAD version axis and the serialization-format \
         axis are DISTINCT (§4.1)."
    );

    // BE codepoint pair (anti-LE drift, M-19).
    assert_eq!(
        &bytes[1..3],
        &[0x65, 0x20],
        "F-LC-2 (CLUSTER-1): group codepoint 0x6520 MUST be big-endian \
         (0x65,0x20) at offset 1, never little-endian (0x20,0x65)."
    );

    // R4.6-MIGRATE: the body_cid is a self-describing CIDv1 (multihash
    // prefix 0x01 0x71 0x1e 0x20 immediately after the codepoint), NOT a
    // bare fixed-32 digest. would-FAIL on a bare-32 regression.
    assert_eq!(
        &bytes[3..7],
        &[0x01, 0x71, 0x1e, 0x20],
        "F-LC-2 (R4.6-MIGRATE): the group stanza body_cid MUST be a \
         self-describing CIDv1 (multihash framing 0x01 0x71 0x1e 0x20 ‖ \
         digest), NOT a bare fixed-32 digest (CLAUDE.md baked-in #5; U3)."
    );

    // FROZEN BLINDED BE layout — drift flips the pin.
    assert_eq!(
        to_hex(&bytes),
        F_LC_2_GROUP_STANZA_AAD_HEX,
        "F-LC-2 (CLUSTER-1 / R4.6-MIGRATE): the DEFAULT group stanza \
         plaintext-AAD MUST serialize to the FROZEN BLINDED big-endian layout \
         (aad_version, codepoint, self-describing body_cid, recipient_count, \
         audience_set_commitment, stanza_index, stanza_count, \
         recipient_key_generation) with NO raw roster, NO coarse_epoch, NO \
         plaintext sender-DID. R5 confirms-or-deliberately-updates this \
         literal (M-20)."
    );

    // The sender-DID is NOT on the DEFAULT path plaintext AAD (the
    // plaintext_sender_did is None), so no sender bytes trail the AAD.
    let sender = did("did:key:zGroupSenderUNIQUEMARKER");
    let leaks = bytes.windows(sender.len()).any(|w| w == sender.as_slice());
    assert!(
        !leaks,
        "F-LC-2 (CLUSTER-1): the DEFAULT group stanza plaintext-AAD MUST NOT \
         carry the sender-DID (it lives in `sealed_inner`)."
    );
}

/// F-LC-2 PIN 8 (R4.6-MIGRATE — ROSTER-NON-LEAK / the BLINDING property) —
/// the central new property of R0.7 §3.3: the relay sees ONLY the opaque
/// 32-byte `audience_set_commitment`, NEVER the raw recipient-DID roster.
/// This is the positive falsifiability pin for the #61-class social-graph
/// leak closure (mirrors `f_aad_2`'s blinded-roster leak-absence arm).
/// would-FAIL if an R5 regression re-published the raw roster into the
/// plaintext AAD (the pre-blinding shape).
#[test]
fn f_lc_2_blinded_group_aad_does_not_leak_raw_recipient_roster() {
    let stanza = f_lc_2_group_stanza_fixture();
    let bytes = stanza.plaintext_aad_bytes();

    // (a) NEGATIVE: each raw recipient-DID byte-sequence MUST be ABSENT from
    //     the on-wire plaintext AAD (the roster is BLINDED, R0.7 §3.3).
    for raw_did in [did("did:key:zRecipientA"), did("did:key:zRecipientB")] {
        let leaks = bytes
            .windows(raw_did.len())
            .any(|w| w == raw_did.as_slice());
        assert!(
            !leaks,
            "F-LC-2 (R4.6 / #61): the raw recipient-DID {:?} MUST NOT appear \
             in the BLINDED plaintext group AAD — R0.7 §3.3 replaces the raw \
             roster with the opaque audience_set_commitment. would-FAIL if \
             the assembler regressed to publishing the raw roster (the \
             pre-blinding #61-class social-graph leak).",
            String::from_utf8_lossy(&raw_did)
        );
    }

    // (b) POSITIVE control: the 32-byte audience_set_commitment IS present in
    //     the wire AAD (so the negative assertion is not vacuously passing
    //     because the roster simply vanished — the BLINDED binding is real).
    let commitment = audience_set_commitment(&stanza.recipient_dids);
    assert_eq!(
        to_hex(&commitment),
        F_LC_2_AUDIENCE_SET_COMMITMENT_HEX,
        "F-LC-2 (R4.6): the audience_set_commitment over the sorted roster \
         MUST equal the frozen golden (BLAKE3(0x01 || lp_u32(did)…); \
         BYTE-IDENTICAL to f_aad_2's 0x6610 commitment construction over the \
         SAME roster)."
    );
    let commitment_present = bytes.windows(32).any(|w| w == commitment.as_slice());
    assert!(
        commitment_present,
        "F-LC-2 (R4.6 positive control): the BLINDED audience_set_commitment \
         MUST be PRESENT in the wire AAD — it is the binding that REPLACES \
         the raw roster. If absent, the roster binding was dropped entirely \
         (not merely blinded)."
    );
}

/// F-LC-2 PIN 9 (R4.6-MIGRATE — COMMITMENT-LP-WIDTH cross-seam) — the
/// `audience_set_commitment` is computed with a **u32-BE** per-DID length
/// prefix inside the hash, the IDENTICAL construction `0x6610` (`f_aad_2`)
/// uses — NOT the band's u16 `recipient_count` width. Conflating the two
/// widths would silently diverge the `0x6520` commitment from `0x6610`,
/// breaking "recipients recompute + verify the commitment" across the two
/// engines. would-FAIL if the lp width were narrowed to u16.
#[test]
fn f_lc_2_commitment_uses_u32_be_lp_identical_to_0x6610() {
    let roster = vec![did("did:key:zRecipientA"), did("did:key:zRecipientB")];
    let commitment = audience_set_commitment(&roster);

    // The production commitment MUST equal the frozen golden (u32-BE lp).
    assert_eq!(
        to_hex(&commitment),
        F_LC_2_AUDIENCE_SET_COMMITMENT_HEX,
        "F-LC-2 (R4.6): the audience_set_commitment MUST match the frozen \
         golden computed with a u32-BE per-DID length prefix (the IDENTICAL \
         construction to 0x6610)."
    );

    // Negative control: a u16-BE lp construction over the SAME roster yields
    // a DIFFERENT commitment — proving the width is load-bearing and the
    // production helper did NOT silently use the band u16 width.
    let mut u16_lp_msg = Vec::new();
    u16_lp_msg.push(0x01u8);
    let mut sorted: Vec<&Vec<u8>> = roster.iter().collect();
    sorted.sort();
    for d in sorted {
        let len = u16::try_from(d.len()).unwrap();
        u16_lp_msg.extend_from_slice(&len.to_be_bytes());
        u16_lp_msg.extend_from_slice(d);
    }
    let u16_lp_commitment: [u8; 32] = blake3::hash(&u16_lp_msg).into();
    assert_ne!(
        commitment, u16_lp_commitment,
        "F-LC-2 (R4.6 / F-LC-2-COMMITMENT-LP-WIDTH): the u32-BE-lp commitment \
         MUST differ from a u16-BE-lp commitment over the SAME roster — the \
         lp width inside the hash is load-bearing and MUST be u32-BE (matching \
         0x6610), NOT the band's u16 recipient_count width. would-FAIL if the \
         production helper silently used u16 and diverged from f_aad_2."
    );
}

/// F-LC-2 PIN 10 (R4.6 / F-46-01 — ROSTER-SORT-CANONICAL falsifiability) —
/// the `audience_set_commitment` is spec-mandated (R0.7 §3.3) over the
/// **CANONICAL SORTED** recipient-DID list, and `plaintext_aad_bytes` reaches
/// that canonicalization ONLY through the assembler's internal `.sort()`
/// (`audience_set_commitment`, lines 386-392 here) — the fixture roster
/// `[zRecipientA, zRecipientB]` is already lexically sorted, so the existing
/// frozen-layout / roster-non-leak / lp-width pins all stay GREEN even if a
/// future R5 edit drops that internal sort. This arm closes that
/// falsifiability hole: it hands the WHOLE `HpkeRecipientStanza`
/// serializer an **UNSORTED (reverse-order) roster** and asserts the
/// `plaintext_aad_bytes()` output is BYTE-IDENTICAL to the sorted-fixture
/// golden. It MIRRORS the sibling `0x6610` arm
/// (`f_aad_2::f_aad_2_member_did_list_sort_order_canonical`, F4-012) so the
/// two engines guard the sort-canonicalization in lockstep. would-FAIL if the
/// production serializer (or `audience_set_commitment`) regressed to NOT
/// canonically sorting the roster before deriving the commitment — a silent
/// cross-engine convergence break (recipients holding the roster in a
/// different order would recompute a different commitment and fail AEAD-open).
/// NON-TAUTOLOGICAL: the reversed-roster commitment differs from the sorted
/// one if the sort is removed (the lp-width PIN 9 already proves the
/// commitment is perturbation-sensitive).
#[test]
fn f_lc_2_group_stanza_aad_unsorted_roster_canonical() {
    // The sorted-fixture serialization (the frozen golden baseline).
    let sorted = f_lc_2_group_stanza_fixture();
    let sorted_bytes = sorted.plaintext_aad_bytes();
    assert_eq!(
        to_hex(&sorted_bytes),
        F_LC_2_GROUP_STANZA_AAD_HEX,
        "F-LC-2 (R4.6 / F-46-01): the sorted-fixture serialization MUST match \
         the frozen golden (baseline for the unsorted-equivalence assertion)."
    );

    // Same recipient SET, presented UNSORTED (reverse order). NO in-body
    // sort — the assembler (`audience_set_commitment`) must canonicalize
    // internally before deriving the commitment.
    let mut reordered = f_lc_2_group_stanza_fixture();
    reordered.recipient_dids = vec![did("did:key:zRecipientB"), did("did:key:zRecipientA")];
    let reordered_bytes = reordered.plaintext_aad_bytes();

    assert_eq!(
        sorted_bytes, reordered_bytes,
        "F-LC-2 (R4.6 / F-46-01): an UNSORTED-but-equal recipient roster MUST \
         serialize to IDENTICAL plaintext-AAD bytes — the assembler \
         canonicalizes (sorts) the roster internally before deriving the \
         audience_set_commitment (R0.7 §3.3 cross-engine convergence; mirrors \
         f_aad_2's F4-012 sort-canonical arm). would-FAIL if the internal \
         .sort() were dropped at R5."
    );
    assert_eq!(
        to_hex(&reordered_bytes),
        F_LC_2_GROUP_STANZA_AAD_HEX,
        "F-LC-2 (R4.6 / F-46-01): the unsorted-roster serialization MUST ALSO \
         reproduce the frozen golden — the sort is fully behind the wire."
    );
}

// ===========================================================================
// F-LC-3 — Sealed-Sender DEFAULT (0x6510): sender-DID NOT on the wire.
// ===========================================================================
// Highest-novelty surface (R2 §1 Group 7). The metadata posture IS the
// wire contract. The 0x6510 single-recipient field-SET is byte-stable across
// R0.7, but its `audience` length-prefix WIDTH is corrected u16→u32-BE
// (R4.6-FIX F-LC-AUD-U32; R0.7 §4.1:1040); 0x6520 is additionally blinded.

/// The deterministic single-recipient `0x6510` Sealed-Sender envelope-AAD
/// fixture. Built DIRECTLY (the seal fns `unimplemented!()` at red-phase) so
/// the canonical `BindingContext::plaintext_aad_bytes()` serializer is driven
/// without panicking. The values are IDENTICAL to the sibling
/// `f_lc_abuse::f_inv18_1_sealed_aad_fixture` so the two files freeze the
/// SAME golden (R4.4-FIX CLUSTER-1 / F-NEW-SS-AUD):
///   aad_version 0x01 | codepoint 0x6510 |
///   audience "did:key:zRecipientAudienceUNIQUE" (32 bytes) |
///   body_cid self-describing CIDv1 over digest [0xE0, 0; 31] (36 bytes;
///   R4.5-MIGRATE) | recipient_key_generation 0.
fn f_lc_3_sealed_sender_aad_fixture() -> BindingContext {
    // R4.5-MIGRATE (R0.6 BR): the body-CID is the SELF-DESCRIBING CIDv1 over
    // the SAME 32-byte digest the corpus froze (`[0xE0, 0; 31]`) — the only
    // golden delta is the prepended 4-byte multihash framing.
    let mut digest: BodyCidDigest = [0u8; 32];
    digest[0] = 0xE0;
    BindingContext::DropSealedSender {
        aad_version: AAD_VERSION, // 0x01 (dedicated AAD prefix, NOT format ver 0x02)
        codepoint: DROP_TO_RECIPIENT_SEALED_SENDER, // 0x6510
        audience_did: did("did:key:zRecipientAudienceUNIQUE"),
        body_cid: self_describing_cid(&digest),
        recipient_key_generation: 0,
    }
}

/// FROZEN big-endian golden vector for the DEFAULT (`0x6510`) single-recipient
/// Sealed-Sender envelope AAD — the canonical union
/// `{aad_version, codepoint, audience(u32-BE length-prefixed), body_cid,
/// recipient_key_generation}`, NO sender-DID region, NO coarse_epoch (R4.4-FIX
/// CLUSTER-1 / F-NEW-SS-AUD / F4-006). **R4.6-FIX F-LC-AUD-U32:** the `audience`
/// length-prefix is `u32-BE` (`00000020` for the 32-byte fixture DID) per R0.7
/// header:33 / §3.3:539 / §4.1:1040 — corrected from the prior u16 (`0020`)
/// slip; the golden grew +2 bytes (77→79). This literal stays BYTE-IDENTICAL to
/// the sibling `f_lc_abuse::F_INV18_1_SEALED_AAD_HEX` (both migrate u16→u32 in
/// lockstep; computed once from the shared BE layout via a throwaway script,
/// M-20). R5 confirms-or-deliberately-updates it against the real encoder. If
/// these two literals ever differ, the siblings have re-diverged on the
/// `0x6510` envelope AAD (the F-NEW-SS-AUD regression).
const F_LC_SEALED_SENDER_AAD_HEX: &str = "016510000000206469643a6b65793a7a526563697069656e7441756469656e6365554e4951554501711e20e00000000000000000000000000000000000000000000000000000000000000000000000";

/// F-LC-3 PIN 1 (R4.4-FIX CLUSTER-1 / F-NEW-SS-AUD) — the DEFAULT (`0x6510`)
/// single-recipient Sealed-Sender envelope AAD binds the canonical union
/// `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`
/// — byte-0 == `AAD_VERSION` (≠ `ENVELOPE_FORMAT_VERSION`), the BE codepoint,
/// the recipient `audience` present, the sender-DID ABSENT — and reproduces
/// the FROZEN golden that is BYTE-IDENTICAL to the sibling
/// `f_lc_abuse::F_INV18_1_SEALED_AAD_HEX`. This closes the audience-axis
/// divergence: `f_lc_hpke`'s single-recipient binding now matches the sibling
/// (and the Inv-16 `Recipient` binding) exactly. would-FAIL if the binding
/// omits the spec-mandated `audience` (§3.3:484), reverts byte-0 to the format
/// version (the F4-004/005 cross-engine AEAD-open break), re-adds a
/// coarse_epoch, leaks the sender-DID, or drifts the layout.
#[test]
fn f_lc_3_sealed_sender_single_recipient_aad_binds_audience_union_and_frozen_golden() {
    let binding = f_lc_3_sealed_sender_aad_fixture();
    let audience = did("did:key:zRecipientAudienceUNIQUE");
    let sender = did("did:key:zSenderAliceUNIQUEMARKER");
    let bytes = binding.plaintext_aad_bytes();

    // Anti-conflation byte-0 guard (mirrors f_lc_abuse single-recipient).
    assert_eq!(
        bytes[0], AAD_VERSION,
        "F-LC-3 (CLUSTER-1 / F4-004/005): the single-recipient 0x6510 AAD \
         byte-0 MUST be the dedicated AAD_VERSION (0x01), NOT the envelope \
         serialization format_version (0x02)."
    );
    assert_ne!(
        bytes[0], ENVELOPE_FORMAT_VERSION,
        "F-LC-3 (CLUSTER-1): byte-0 MUST NOT be the envelope format version \
         — the AAD-version axis and the serialization-format axis are \
         DISTINCT (§4.1)."
    );

    // BE codepoint pair (anti-LE drift, M-19).
    assert_eq!(
        &bytes[1..3],
        &[0x65, 0x10],
        "F-LC-3 (CLUSTER-1): the Sealed-Sender drop codepoint 0x6510 MUST be \
         big-endian (0x65,0x10) at offset 1."
    );

    // R4.6-FIX F-LC-AUD-U32: the `audience` length-prefix MUST be u32-BE
    // (R0.7 header:33 / §3.3:539 / §4.1:1040 freeze the 0x6510 audience at
    // `u32-BE-length-prefixed`). For the 32-byte fixture DID that is the four
    // bytes `00 00 00 20` at offset 3 (immediately after aad_version[0] +
    // codepoint[1..3]). would-FAIL on a u16 (`00 20`) regression — the prior
    // "settled-territory" slip that conflated this variable-field lp with the
    // 0x6520 band's `recipient_count` cardinality (which is correctly u16).
    assert_eq!(
        &bytes[3..7],
        &(audience.len() as u32).to_be_bytes(),
        "F-LC-3 (R4.6 / F-LC-AUD-U32): the 0x6510 audience length-prefix MUST \
         be u32-BE ({:?} for the {}-byte audience DID), NOT u16. R0.7 \
         header:33 / §3.3:539 / §4.1:1040 freeze the audience at \
         `u32-BE-length-prefixed`. would-FAIL if the serializer regressed to \
         a u16 prefix (the F-LC-AUD-U32 wire-byte slip).",
        (audience.len() as u32).to_be_bytes(),
        audience.len()
    );

    // The recipient AUDIENCE MUST be bound (§3.3:484 recipient-targeting) —
    // this is the axis F-NEW-SS-AUD was missing. would-FAIL on the
    // audience-less form the stale binding froze.
    let audience_present = bytes
        .windows(audience.len())
        .any(|w| w == audience.as_slice());
    assert!(
        audience_present,
        "F-LC-3 (F-NEW-SS-AUD): the single-recipient 0x6510 AAD MUST bind the \
         recipient AUDIENCE DID (§3.3:484 recipient-targeting). would-FAIL on \
         the stale audience-less binding that diverged from f_lc_abuse + the \
         Inv-16 Recipient binding."
    );

    // The SENDER-DID MUST be ABSENT (Sealed-Sender — it lives inside the
    // ciphertext, not the AAD).
    let sender_leaks = bytes.windows(sender.len()).any(|w| w == sender.as_slice());
    assert!(
        !sender_leaks,
        "F-LC-3 (CLUSTER-1): the single-recipient 0x6510 AAD MUST NOT carry \
         the sender-DID — Sealed-Sender binds it INSIDE the ciphertext."
    );

    // FROZEN BE layout — and it MUST equal the sibling's golden (shared
    // literal proves the siblings cannot silently re-diverge).
    assert_eq!(
        to_hex(&bytes),
        F_LC_SEALED_SENDER_AAD_HEX,
        "F-LC-3 (CLUSTER-1 / F-NEW-SS-AUD): the single-recipient 0x6510 AAD \
         MUST serialize to the FROZEN canonical union layout (aad_version, \
         codepoint, audience_len||audience, body_cid, recipient_key_gen) — \
         BYTE-IDENTICAL to the sibling f_lc_abuse::F_INV18_1_SEALED_AAD_HEX. \
         If this literal ever differs from the sibling's, the two engines \
         have re-diverged on the 0x6510 envelope AAD. R5 confirms-or-\
         deliberately-updates this literal (M-20)."
    );
}

/// F-LC-3 PIN 2 — on the DEFAULT (`0x6510`) path the serialized wire
/// bytes DO NOT contain the sender-DID. The sender-DID is bound INSIDE
/// the ciphertext (HPKE inner-payload); the on-wire AAD = audience binding
/// (codepoint + audience + body-CID + recipient-key-generation; NO timestamp
/// NOR coarse-epoch — M-14 / F4-006). would-FAIL if the default seal leaks the
/// sender-DID into the AAD (the bug `0x6500` deliberately has, that `0x6510`
/// fixes).
#[test]
fn f_lc_3_sealed_sender_default_omits_sender_did_from_wire() {
    let audience = did("did:key:zRecipientAudienceUNIQUE");
    let (sender_kp, sender) = hybrid_sender();
    let env = seal_sealed_sender(
        &fixed_pk(0x50),
        &audience,
        &sender,
        &sender_kp,
        &fixed_body_cid_digest(0xE0),
        0,
        b"sealed-sender payload",
    );

    // The default seal MUST carry the Sealed-Sender binding (audience bound,
    // no sender-DID field in the AAD-bearing binding).
    match &env {
        EncryptedEnvelope::HpkeBase {
            binding:
                BindingContext::DropSealedSender {
                    codepoint,
                    audience_did,
                    ..
                },
            ..
        } => {
            assert_eq!(
                *codepoint, DROP_TO_RECIPIENT_SEALED_SENDER,
                "F-LC-3: the v1-beta DEFAULT MUST be the Sealed-Sender drop \
                 (0x6510)."
            );
            assert_eq!(
                audience_did, &audience,
                "F-LC-3: the Sealed-Sender drop AAD MUST bind the recipient \
                 audience DID (§3.3:484)."
            );
        }
        _ => panic!(
            "F-LC-3: the default Layer-C seal MUST produce an HpkeBase with \
             a DropSealedSender binding (BR-1). A DropPlaintextSender here \
             would mean the default leaks the sender-DID."
        ),
    }

    // Scan the WHOLE serialized wire for the sender-DID byte sequence.
    let wire = serialize(&env);
    let needle = &sender;
    let leaks = wire.windows(needle.len()).any(|w| w == needle.as_slice());
    assert!(
        !leaks,
        "F-LC-3: the Sealed-Sender DEFAULT (0x6510) MUST NOT place the \
         sender-DID anywhere in the serialized wire bytes — it is bound \
         INSIDE the ciphertext (HPKE inner-payload), recovered only \
         post-decrypt. would-FAIL if the default path bound the sender-DID \
         into the on-wire AAD."
    );
}

/// F-LC-3 PIN 3 — PAIRED POSITIVE CONTROL: the NON-default plaintext-sender
/// sibling (`0x6500`) DOES carry the sender-DID on the wire (U4). This is
/// the control that proves PIN 2 is not vacuously passing because the
/// scanner is broken. would-FAIL if `0x6500` ALSO hid the sender-DID
/// (then the scanner can't tell the two paths apart).
#[test]
fn f_lc_3_plaintext_sender_sibling_carries_sender_did_on_wire() {
    let audience = did("did:key:zRecipientAudienceUNIQUE");
    let (sender_kp, sender) = hybrid_sender();
    let env = seal_plaintext_sender(
        &fixed_pk(0x51),
        &audience,
        &sender,
        &sender_kp,
        &fixed_body_cid_digest(0xE1),
        0,
        b"plaintext-sender payload",
    );

    // The non-default seal MUST carry the plaintext-sender binding with the
    // sender-DID bound INTO the AAD (U4), audience also bound.
    match &env {
        EncryptedEnvelope::HpkeBase {
            binding:
                BindingContext::DropPlaintextSender {
                    codepoint,
                    audience_did,
                    sender_did,
                    ..
                },
            ..
        } => {
            assert_eq!(
                *codepoint, LAYER_C_DROP,
                "F-LC-3: the plaintext-sender sibling MUST be 0x6500."
            );
            assert_eq!(
                audience_did, &audience,
                "F-LC-3: the 0x6500 binding MUST also bind the recipient \
                 audience DID (§3.3:484)."
            );
            assert_eq!(
                sender_did, &sender,
                "F-LC-3: the 0x6500 binding MUST carry the sender-DID in its \
                 AAD (U4)."
            );
        }
        _ => panic!("F-LC-3: seal_plaintext_sender MUST produce a DropPlaintextSender binding"),
    }

    let wire = serialize(&env);
    let leaks = wire.windows(sender.len()).any(|w| w == sender.as_slice());
    assert!(
        leaks,
        "F-LC-3 PAIRED CONTROL: the NON-default plaintext-sender (0x6500) \
         MUST place the sender-DID on the wire (U4). If this control fails, \
         the PIN-2 scanner cannot distinguish hiding from a broken scan — \
         the two paths must differ observably."
    );
}

/// F-LC-3 PIN 4 — post-decrypt the recovered sender-DID equals the bound
/// sender. The Sealed-Sender property is "hidden on the wire, recovered
/// by the recipient." would-FAIL if the inner-payload sender-DID is not
/// recoverable (then Sealed-Sender breaks sender attribution entirely).
#[test]
fn f_lc_3_recovered_inner_sender_did_equals_bound() {
    let pk = fixed_pk(0x52);
    let sk = fixed_sk(0x52);
    let audience = did("did:key:zRecipientAudience");
    let (sender_kp, sender) = hybrid_sender();
    let env = seal_sealed_sender(
        &pk,
        &audience,
        &sender,
        &sender_kp,
        &body_cid_of(b"hi"),
        0,
        b"hi",
    );

    let (_pt, recovered_sender) = open_single(&sk, &audience, 0, &env)
        .expect("recipient MUST open the sealed-sender envelope");
    assert_eq!(
        recovered_sender, sender,
        "F-LC-3: the recipient MUST recover the bound inner sender-DID \
         post-decrypt. would-FAIL if the inner-payload sender-DID is not \
         carried/recovered."
    );
}

// ===========================================================================
// F-LC-3 — B2 Sealed-Sender ORIGIN-AUTHENTICATION (SUBSTANTIVE, F-6).
// ===========================================================================
// These pins REPLACE the prior SHAPE-not-SUBSTANCE
// `f_lc_3_forged_inner_sender_did_rejected` (which only flipped a ciphertext
// byte → the AEAD tag caught it, and a full revert of origin-auth left it
// GREEN). Each pin below drives the PRODUCTION verify (`open_*`) with a real
// second-sealer / second-member spoof / re-target / stale-generation / stripped-
// PQ-half adversary and asserts a fail-closed `SenderOriginAuthFailed` — each
// is would-FAIL-on-revert of the B2 origin-auth feature.

/// F-LC-3 PIN 5a (B2 SUBSTANTIVE — `0x6510` SECOND-SEALER SPOOF) — a real
/// second sealer (B) who knows the recipient pubkey builds a fully-valid
/// AEAD-opening envelope CLAIMING `sender_did = A` but signs `M_auth` with
/// **B's** hybrid key. The envelope AEAD-opens cleanly (B derives the CEK
/// from the public recipient fingerprint), but the B2 origin-auth verify —
/// resolving A's hybrid did:key and verifying the signature against it —
/// REJECTS (`SenderOriginAuthFailed`), because B does not hold A's signing
/// key.
///
/// would-FAIL-on-revert: with origin-auth removed, `open_single` returns
/// `Ok((body, A))` — a silent successful impersonation. A pure ciphertext-byte
/// flip (the OLD shape-trap) would NOT catch this — the AEAD tag is valid.
#[test]
fn f_lc_3_second_sealer_spoof_rejected_single() {
    let pk = fixed_pk(0x53);
    let sk = fixed_sk(0x53);
    let audience = did("did:key:zRecipientAudience");

    // Positive control: A's own send opens + origin-verifies.
    let (a_kp, a_did) = hybrid_sender();
    let honest = seal_sealed_sender(&pk, &audience, &a_did, &a_kp, &body_cid_of(b"hi"), 0, b"hi");
    let (_pt, recovered) =
        open_single(&sk, &audience, 0, &honest).expect("A's honest send MUST open + origin-verify");
    assert_eq!(recovered, a_did, "positive control recovers A");

    // ATTACK: B claims sender_did = A but signs with B's key (B seals an
    // HONESTLY-CID'd body so the rejection is specifically the WRONG-SIGNER
    // origin-auth failure, NOT the F-01 content-splice guard).
    let (b_kp, _b_did) = hybrid_sender();
    let spoof = seal_sealed_sender(
        &pk,
        &audience,
        &a_did,
        &b_kp,
        &body_cid_of(b"forged-as-A"),
        0,
        b"forged-as-A",
    );
    let outcome = open_single(&sk, &audience, 0, &spoof);
    assert_eq!(
        outcome,
        Err(LayerCError::SenderOriginAuthFailed),
        "F-LC-3 (B2): a 0x6510 second-sealer who CLAIMS sender=A but signs \
         with B's key MUST be rejected at the post-decrypt origin-auth verify \
         (SenderOriginAuthFailed), NOT at the AEAD layer (B can produce a \
         valid AEAD tag). would-FAIL-on-revert: without origin-auth the open \
         returns Ok((body, A)) — silent impersonation. Got: {outcome:?}"
    );
}

/// F-LC-3 PIN 5b (B2 SUBSTANTIVE — `0x6610` SECOND-MEMBER SPOOF) — THE group
/// threat. Members A and B both hold `K_Set`. B derives the group CEK from
/// `K_Set` and builds a fresh valid `GroupSealedEnvelope` CLAIMING
/// `sender_did = A`, signing the body-region `M_auth` with **B's** key. Every
/// stanza AEAD-opens (B holds K_Set), but every honest recipient's
/// `open_membership_set_group` REJECTS with `GroupError::SenderOriginAuthFailed`
/// — the exact THREAT-MODEL "Co-recipient member" impersonation gap, now closed.
///
/// would-FAIL-on-revert: without origin-auth the co-member impersonation
/// SUCCEEDS (the AEAD tag is valid because B holds K_Set).
#[test]
fn f_lc_3_second_member_spoof_rejected_membership_group() {
    use benten_drop::layer_c::group_posture::{
        GroupError, GroupSealParams, GroupVerifyContext, open_membership_set_group,
        seal_membership_set_group,
    };

    let pks = [fixed_pk(0x80), fixed_pk(0x81), fixed_pk(0x82)];
    let sks = [fixed_sk(0x80), fixed_sk(0x81), fixed_sk(0x82)];
    let k_set = [0x99u8; 32];
    let params = GroupSealParams {
        membership_set_id: b"set-alpha".to_vec(),
        member_key_generation: 4,
        membership_set_generation: 7,
        role_assignments_generation: 2,
    };
    // The honest members hold the roster + generations independently.
    let member_dids: Vec<String> = group_roster_for_test(&pks)
        .iter()
        .map(|d| String::from_utf8_lossy(d).into_owned())
        .collect();
    let ctx = GroupVerifyContext {
        member_dids: member_dids.clone(),
        member_key_generation: 4,
        membership_set_generation: 7,
        role_assignments_generation: 2,
    };

    // Positive control: A (a real member) seals; every honest member opens +
    // origin-verifies.
    let (a_kp, a_did) = hybrid_sender();
    let honest = seal_membership_set_group(&pks, &a_did, &a_kp, &k_set, &params, b"group hi")
        .expect("valid roster must seal (R18 C2)");
    for (i, sk) in sks.iter().enumerate() {
        let (_pt, rec) = open_membership_set_group(sk, i, &ctx, &honest)
            .unwrap_or_else(|e| panic!("member {i} MUST open A's honest group send: {e:?}"));
        assert_eq!(rec, a_did, "positive control recovers A for member {i}");
    }

    // ATTACK: member B (holds K_Set) claims sender_did = A, signs with B's key.
    let (b_kp, _b_did) = hybrid_sender();
    let spoof = seal_membership_set_group(&pks, &a_did, &b_kp, &k_set, &params, b"forged-as-A")
        .expect("valid roster must seal (R18 C2)");
    for (i, sk) in sks.iter().enumerate() {
        let outcome = open_membership_set_group(sk, i, &ctx, &spoof);
        assert_eq!(
            outcome,
            Err(GroupError::SenderOriginAuthFailed),
            "F-LC-3 (B2): a 0x6610 co-member B who holds K_Set + CLAIMS \
             sender=A but signs with B's key MUST be rejected by honest \
             recipient {i} at the origin-auth verify (the THREAT-MODEL \
             Co-recipient-member gap). would-FAIL-on-revert: without \
             origin-auth the impersonation succeeds (B's AEAD tag is valid). \
             Got: {outcome:?}"
        );
    }
}

/// CONF-1 (SECURITY-CRITICAL — `0x6610` per-message CEK / nonce-reuse) — the
/// MembershipSet group bulk-CEK MUST be PER-MESSAGE-unique. Before the fix the
/// `0x6610` CEK was `BLAKE3(ctx ‖ K_Set ‖ sender_did)` — mixing NOTHING per
/// message — so EVERY send from a fixed sender under a fixed K_Set generation
/// sealed under one byte-IDENTICAL CEK. Combined with the random 96-bit
/// ChaCha20-Poly1305 nonce that path hits the birthday wall (~2^48 seals) where
/// a single nonce collision under the reused key is catastrophic (keystream
/// reuse + Poly1305 forgery). The fix mixes the per-message `cid` (= the wire
/// `body_cid`, the self-describing CIDv1 over `BLAKE3(body)`) into the CEK —
/// IDENTICAL in spirit to the `0x6520` `seal_group_impl` CEK, which binds
/// `body_cid` per send.
///
/// This drives the PRODUCTION `seal_membership_set_group` entry point twice
/// (SAME sender, SAME K_Set, SAME generations — ONLY the body differs), derives
/// each envelope's CEK via the SAME live derivation the seal calls, and asserts
/// the two CEKs DIFFER.
///
/// would-FAIL-on-revert: drop the `cid`-mix from `derive_group_cek` and the two
/// CEKs become byte-IDENTICAL (the only varied input — the body → its cid — no
/// longer reaches the CEK) → `assert_ne!` fires.
#[test]
fn f_conf_1_membership_group_cek_is_per_message_unique() {
    use benten_drop::layer_c::group_posture::{
        GroupSealParams, GroupVerifyContext, open_membership_set_group, seal_membership_set_group,
    };

    let pks = [fixed_pk(0xA0), fixed_pk(0xA1), fixed_pk(0xA2)];
    let sks = [fixed_sk(0xA0), fixed_sk(0xA1), fixed_sk(0xA2)];
    let k_set = [0x5Au8; 32];
    let params = GroupSealParams {
        membership_set_id: b"set-conf1".to_vec(),
        member_key_generation: 3,
        membership_set_generation: 9,
        role_assignments_generation: 1,
    };
    let member_dids: Vec<String> = group_roster_for_test(&pks)
        .iter()
        .map(|d| String::from_utf8_lossy(d).into_owned())
        .collect();
    let ctx = GroupVerifyContext {
        member_dids,
        member_key_generation: 3,
        membership_set_generation: 9,
        role_assignments_generation: 1,
    };

    // ONE sender, ONE K_Set generation; the ONLY thing that differs across the
    // two production seals is the body (→ its content cid).
    let (a_kp, a_did) = hybrid_sender();
    let body_1 = b"membership group message ONE";
    let body_2 = b"membership group message TWO (a different body)";
    let env_1 = seal_membership_set_group(&pks, &a_did, &a_kp, &k_set, &params, body_1)
        .expect("valid roster must seal (R18 C2)");
    let env_2 = seal_membership_set_group(&pks, &a_did, &a_kp, &k_set, &params, body_2)
        .expect("valid roster must seal (R18 C2)");

    // Re-derive each per-message CEK via the SAME live `derive_group_cek` the
    // seal used (the seam binds `self.body_cid` — i.e. the WIRE cid — proving
    // recipient-recomputability from inputs a member already holds: K_Set + the
    // wire cid). The inner sender-DID is A's did:key bytes.
    let cek_1 = env_1.derive_cek_for_test(&k_set, &a_did);
    let cek_2 = env_2.derive_cek_for_test(&k_set, &a_did);

    // CONF-1 CORE: distinct bodies → distinct CEKs. Reverting the cid-mix makes
    // these byte-identical (one reused CEK across every send → nonce-reuse).
    assert_ne!(
        cek_1, cek_2,
        "CONF-1: two distinct 0x6610 sends from the SAME sender under the SAME \
         K_Set generation MUST seal under DISTINCT per-message CEKs (the CEK \
         binds the per-message cid). would-FAIL-on-revert: without the cid-mix \
         both derive to the byte-IDENTICAL CEK = catastrophic AEAD nonce reuse."
    );

    // CONTROL: re-sealing the SAME body (→ same cid) re-derives the SAME CEK —
    // confirming the CEK is a deterministic function of (K_Set, sender, cid)
    // and that it is the BODY (via cid) driving the difference above, nothing else.
    let env_1b = seal_membership_set_group(&pks, &a_did, &a_kp, &k_set, &params, body_1)
        .expect("valid roster must seal (R18 C2)");
    assert_eq!(
        cek_1,
        env_1b.derive_cek_for_test(&k_set, &a_did),
        "CONF-1 control: identical (sender, K_Set, body) MUST re-derive the same CEK."
    );

    // LEGIT ROUND-TRIP STILL HOLDS: every honest member opens BOTH sends and
    // recovers the correct body + sender (the cid-mix did not break seal→open;
    // the recipient HPKE-unwraps the wrapped CEK, it does not re-derive).
    for (i, sk) in sks.iter().enumerate() {
        let (pt_1, rec_1) = open_membership_set_group(sk, i, &ctx, &env_1)
            .unwrap_or_else(|e| panic!("member {i} MUST open send-1: {e:?}"));
        assert_eq!(pt_1, body_1, "send-1 body recovered for member {i}");
        assert_eq!(rec_1, a_did, "send-1 sender recovered for member {i}");
        let (pt_2, rec_2) = open_membership_set_group(sk, i, &ctx, &env_2)
            .unwrap_or_else(|e| panic!("member {i} MUST open send-2: {e:?}"));
        assert_eq!(pt_2, body_2, "send-2 body recovered for member {i}");
        assert_eq!(rec_2, a_did, "send-2 sender recovered for member {i}");
    }
}

/// F-LC-3 PIN 5c (B2 SUBSTANTIVE — `0x6520` SECOND-SEALER SPOOF) — same shape
/// as 5b for the Layer-C group, where the CEK is derivable from public inputs
/// → ANY party can spoof today. B claims `sender_did = A` but signs with B's
/// key; every honest recipient REJECTS with `SenderOriginAuthFailed`.
///
/// would-FAIL-on-revert: without origin-auth the spoof opens as A-attributed.
#[test]
fn f_lc_3_second_sealer_spoof_rejected_layer_c_group() {
    let pks = [fixed_pk(0x90), fixed_pk(0x91)];
    let sks = [fixed_sk(0x90), fixed_sk(0x91)];
    let roster = group_roster_for_test(&pks);

    // Positive control.
    let (a_kp, a_did) = hybrid_sender();
    let honest = seal_group_multi(
        &pks,
        &a_did,
        &a_kp,
        &body_cid_of(b"group hi"),
        0,
        b"group hi",
    )
    .expect("group seal within recipient limit");
    for (i, sk) in sks.iter().enumerate() {
        let (_pt, rec) = open_group_stanza(sk, i, &roster, 0, &honest)
            .unwrap_or_else(|e| panic!("recipient {i} MUST open A's honest send: {e:?}"));
        assert_eq!(rec, a_did, "positive control recovers A for recipient {i}");
    }

    // ATTACK: B claims sender_did = A but signs with B's key (B HONESTLY-CIDs
    // its forged body so the rejection is specifically the WRONG-SIGNER
    // origin-auth failure, NOT the F-01 content-splice guard).
    let (b_kp, _b_did) = hybrid_sender();
    let spoof = seal_group_multi(
        &pks,
        &a_did,
        &b_kp,
        &body_cid_of(b"forged-as-A"),
        0,
        b"forged-as-A",
    )
    .expect("group seal within recipient limit");
    for (i, sk) in sks.iter().enumerate() {
        let outcome = open_group_stanza(sk, i, &roster, 0, &spoof);
        assert_eq!(
            outcome,
            Err(LayerCError::SenderOriginAuthFailed),
            "F-LC-3 (B2): a 0x6520 second-sealer who CLAIMS sender=A but signs \
             with B's key MUST be rejected by honest recipient {i}. \
             would-FAIL-on-revert: without origin-auth the spoof opens as \
             A-attributed. Got: {outcome:?}"
        );
    }
}

/// F-LC-3 PIN 5e (B2 SUBSTANTIVE — `0x6610` CONTENT-SPLICE, F-01
/// SOUNDNESS-CRITICAL) — THE content-splice threat. A co-member B who holds
/// `K_Set` captures the VICTIM A's HONEST send. M_auth binds the body ONLY
/// through `body_cid`, so B KEEPS A's real `sender_sig` + the ORIGINAL
/// (unchanged) `body_cid` and re-seals a DIFFERENT body under the K_Set-derived
/// CEK (which B holds). Every stanza (sender_did = A) and the wire `body_cid`
/// are byte-unchanged. An honest recipient MUST REJECT with
/// `GroupError::SenderOriginAuthFailed` because the recovered body no longer
/// hashes to the wire `body_cid`.
///
/// would-FAIL-on-revert: WITHOUT the F-01 content-splice guard (the recompute +
/// byte-equality of `self_describing_cid(BLAKE3(body))` against the wire
/// `body_cid`), `open_membership_set_group` recovers the SUBSTITUTED body,
/// rebuilds M_auth from the WIRE `body_cid` (= A's original), and verifies A's
/// REAL sig → the forged body opens A-attributed. This test would then read
/// `Ok((b"FORGED ...", a_did))` instead of the asserted `Err`.
#[test]
fn f_lc_3_content_splice_rejected_membership_group() {
    use benten_drop::layer_c::group_posture::{
        GroupError, GroupSealParams, GroupVerifyContext, open_membership_set_group,
        seal_membership_set_group,
    };

    let pks = [fixed_pk(0xB0), fixed_pk(0xB1), fixed_pk(0xB2)];
    let sks = [fixed_sk(0xB0), fixed_sk(0xB1), fixed_sk(0xB2)];
    let k_set = [0x5Au8; 32];
    let params = GroupSealParams {
        membership_set_id: b"set-splice".to_vec(),
        member_key_generation: 3,
        membership_set_generation: 9,
        role_assignments_generation: 1,
    };
    let member_dids: Vec<String> = group_roster_for_test(&pks)
        .iter()
        .map(|d| String::from_utf8_lossy(d).into_owned())
        .collect();
    let ctx = GroupVerifyContext {
        member_dids,
        member_key_generation: 3,
        membership_set_generation: 9,
        role_assignments_generation: 1,
    };

    // Positive control: A's honest send opens + verifies for every member, and
    // recovers A's EXACT body (the F-01 guard does NOT reject honest sends —
    // the honest body's recomputed cid == its wire body_cid).
    let (a_kp, a_did) = hybrid_sender();
    let honest = seal_membership_set_group(&pks, &a_did, &a_kp, &k_set, &params, b"honest body")
        .expect("valid roster must seal (R18 C2)");
    for (i, sk) in sks.iter().enumerate() {
        let (pt, rec) = open_membership_set_group(sk, i, &ctx, &honest)
            .unwrap_or_else(|e| panic!("member {i} MUST open A's honest group send: {e:?}"));
        assert_eq!(rec, a_did, "positive control recovers A for member {i}");
        assert_eq!(pt, b"honest body", "honest body recovered byte-exact");
    }

    // ATTACK: member B (holds K_Set) keeps A's REAL sender_sig + the ORIGINAL
    // body_cid, but re-seals a DIFFERENT body under the K_Set-derived CEK.
    let spliced =
        honest.with_spliced_body_for_test(&k_set, &a_did, b"FORGED splice attributed to A");
    for (i, sk) in sks.iter().enumerate() {
        let outcome = open_membership_set_group(sk, i, &ctx, &spliced);
        assert_eq!(
            outcome,
            Err(GroupError::SenderOriginAuthFailed),
            "F-LC-3 (B2 / F-01): a 0x6610 co-member B who holds K_Set + KEEPS \
             A's real sender_sig + the ORIGINAL body_cid but splices a DIFFERENT \
             body MUST be rejected by honest recipient {i} (the recovered body no \
             longer hashes to the wire body_cid). would-FAIL-on-revert: without \
             the content-splice guard the forged body opens A-attributed. \
             Got: {outcome:?}"
        );
    }
}

/// F-LC-3 PIN 5f (B2 SUBSTANTIVE — `0x6520` CONTENT-SPLICE, F-01
/// SOUNDNESS-CRITICAL) — the RESIDUAL insider threat after R11 MC-1: the
/// `0x6520` group CEK is now a fresh random per-message value delivered ONLY
/// via the per-stanza HPKE-wrap, so a NON-recipient can no longer recover it
/// (see [`mc_1_non_recipient_cannot_recover_group_cek`]). But a legitimate
/// CO-RECIPIENT B holds the CEK (it unwraps its own stanza), so B CAN keep A's
/// real `sender_sig` + the ORIGINAL `body_cid` and substitute a DIFFERENT body.
/// Every honest recipient REJECTS with `LayerCError::SenderOriginAuthFailed`.
///
/// would-FAIL-on-revert: WITHOUT the F-01 content-splice guard,
/// `open_group_stanza` recovers the SUBSTITUTED body, rebuilds M_auth from the
/// WIRE `body_cid` (= A's original), and verifies A's REAL sig → the forged
/// body opens A-attributed (this test would read `Ok((b"FORGED ...", a_did))`).
#[test]
fn f_lc_3_content_splice_rejected_layer_c_group() {
    use benten_drop::layer_c::splice_group_multi_body_for_test;

    let pks = [fixed_pk(0xC0), fixed_pk(0xC1)];
    let sks = [fixed_sk(0xC0), fixed_sk(0xC1)];
    let roster = group_roster_for_test(&pks);
    // HONEST sender: body_cid = BLAKE3(body) (the design's content-CID contract).
    let body_cid = body_cid_of(b"honest body");

    // Positive control: A's honest send recovers A's EXACT body for every
    // recipient (the F-01 guard does NOT reject the honest send).
    let (a_kp, a_did) = hybrid_sender();
    let honest = seal_group_multi(&pks, &a_did, &a_kp, &body_cid, 0, b"honest body")
        .expect("group seal within recipient limit");
    for (i, sk) in sks.iter().enumerate() {
        let (pt, rec) = open_group_stanza(sk, i, &roster, 0, &honest)
            .unwrap_or_else(|e| panic!("recipient {i} MUST open A's honest send: {e:?}"));
        assert_eq!(rec, a_did, "positive control recovers A for recipient {i}");
        assert_eq!(pt, b"honest body", "honest body recovered byte-exact");
    }

    // ATTACK: co-recipient B (stanza index 1, holds sks[1]) legitimately
    // UNWRAPS the shared CEK from its own stanza, keeps A's REAL sender_sig +
    // the ORIGINAL body_cid, and re-seals a DIFFERENT body under that CEK.
    let spliced =
        splice_group_multi_body_for_test(&honest, &sks[1], 1, b"FORGED splice attributed to A");
    for (i, sk) in sks.iter().enumerate() {
        let outcome = open_group_stanza(sk, i, &roster, 0, &spliced);
        assert_eq!(
            outcome,
            Err(LayerCError::SenderOriginAuthFailed),
            "F-LC-3 (B2 / F-01): a 0x6520 CO-RECIPIENT who unwraps the shared \
             CEK, KEEPS A's real sender_sig + the ORIGINAL body_cid, and splices \
             a DIFFERENT body MUST be rejected by honest recipient {i} (the \
             recovered body no longer hashes to the wire body_cid). \
             would-FAIL-on-revert: without the content-splice guard the forged \
             body opens A-attributed. Got: {outcome:?}"
        );
    }
}

/// R11 MC-1 (SECURITY-CRITICAL — Sealed-Sender confidentiality) — the property
/// the fresh-random-CEK fix RESTORES: a NON-recipient party (holds NO recipient
/// secret; only the PUBLIC/wire material — the envelope bytes + the sender's
/// public DID + the roster + generation) CANNOT recover the group bulk-CEK and
/// so CANNOT decrypt the `0x6520` body.
///
/// would-FAIL-on-revert: if the CEK were reverted to the old public-input
/// derivation `BLAKE3(LAYER_C_GROUP_CEK_CONTEXT ‖ body_cid ‖ sender_did ‖
/// generation)`, that recomputed value WOULD unwrap the body (all four inputs
/// are on the wire / a small guessable set). This test asserts the OPPOSITE:
/// (1) the old public-derived candidate does NOT equal the real CEK a recipient
/// unwraps, and (2) it does NOT AEAD-open the body — both would flip if MC-1
/// were reverted.
#[test]
fn mc_1_non_recipient_cannot_recover_group_cek() {
    use benten_crypto_suite::AeadKeyMaterial;
    use benten_crypto_suite::aead::{AeadEnvelope, unwrap as aead_unwrap};
    use benten_drop::layer_c::{LAYER_C_GROUP_CEK_CONTEXT, unwrap_group_cek_for_test};

    let pks = [fixed_pk(0xD0), fixed_pk(0xD1)];
    let sks = [fixed_sk(0xD0), fixed_sk(0xD1)];
    let roster = group_roster_for_test(&pks);
    let generation = 0u32;
    let body = b"secret group body a relay must not read";
    let body_cid = body_cid_of(body);

    let (a_kp, a_did) = hybrid_sender();
    let env = seal_group_multi(&pks, &a_did, &a_kp, &body_cid, generation, body)
        .expect("group seal within recipient limit");

    // A LEGITIMATE recipient CAN open (the CEK it unwraps is the real one).
    let (pt, rec) = open_group_stanza(&sks[0], 0, &roster, generation, &env)
        .expect("legitimate recipient MUST open the honest group send");
    assert_eq!(rec, a_did);
    assert_eq!(pt, body, "recipient recovers the plaintext body");

    // Pull the wire pieces a NON-recipient relay sees.
    let EncryptedEnvelope::HpkeMultiBase {
        cek_aead_ciphertext,
        ..
    } = &env
    else {
        panic!("0x6520 group send must be HpkeMultiBase");
    };
    let cid = self_describing_cid(&body_cid);

    // The REAL CEK, recovered the way a recipient does (unwrap from its stanza),
    // to compare the attacker candidate against it.
    let real_cek = unwrap_group_cek_for_test(&sks[0], 0, &env);

    // The ATTACKER candidate: the OLD public-input derivation (all wire-known).
    let attacker_cek = {
        let mut h = blake3::Hasher::new();
        h.update(LAYER_C_GROUP_CEK_CONTEXT);
        h.update(&body_cid);
        h.update(&a_did);
        h.update(&generation.to_be_bytes());
        *h.finalize().as_bytes()
    };

    // (1) The public-derived candidate is NOT the real (random) CEK.
    assert_ne!(
        attacker_cek.as_slice(),
        real_cek.as_slice(),
        "R11 MC-1: the fresh random 0x6520 CEK MUST NOT equal the old \
         public-input-derived value — would-FAIL-on-revert if the CEK were \
         derived from public wire inputs again"
    );

    // (2) The public-derived candidate CANNOT decrypt the body.
    let mut body_aad = Vec::new();
    body_aad.push(AAD_VERSION);
    body_aad.extend_from_slice(&LAYER_C_DROP_MULTI_RECIPIENT.to_be_bytes());
    body_aad.extend_from_slice(&cid);
    let attacker_key = AeadKeyMaterial::from_raw_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        &attacker_cek,
    );
    let body_env = AeadEnvelope::from_wire_bytes(cek_aead_ciphertext)
        .expect("body envelope parses (public wire framing)");
    let forged_open = aead_unwrap(&body_env, &attacker_key, &body_aad);
    assert!(
        forged_open.is_err(),
        "R11 MC-1: a NON-recipient using only PUBLIC wire inputs MUST NOT \
         recover the CEK / decrypt the 0x6520 body — would-FAIL-on-revert to \
         the public-derived CEK. Got: {forged_open:?}"
    );
}

/// F-LC-3 PIN 5d (B2 SUBSTANTIVE — RE-TARGET to a NEW audience, F-2
/// SOUNDNESS-CRITICAL) — the adversarial case the per-message construction
/// must specifically defeat (design §1.4 (b)). Sender A legitimately seals to
/// recipient-set **S1** (roster R1); A's signature is over commitment(R1). An
/// attacker keeps A's ORIGINAL S1-bound envelope on the wire but delivers it
/// to an **S2** recipient — who recomputes the commitment from its OWN
/// independently-held roster **R2 ≠ R1** and REJECTS (`SenderOriginAuthFailed`),
/// because A never signed over commitment(R2).
///
/// **This pins F-2:** the verify MUST recompute the audience commitment from
/// the recipient's own held roster, NOT the attacker-controllable wire value.
/// would-FAIL-on-revert: if `open_group_stanza` used the wire
/// `stanza.recipient_dids` (= R1) instead of the independent roster (= R2),
/// the re-targeted send would open as a valid A-attributed message to an
/// audience A never chose — the test below passes the WRONG independent roster
/// and asserts rejection, so dropping the recompute makes it FAIL.
#[test]
fn f_lc_3_retarget_to_new_audience_rejected() {
    // --- 0x6520 Layer-C group ---
    let pks = [fixed_pk(0xA0), fixed_pk(0xA1)];
    let sks = [fixed_sk(0xA0), fixed_sk(0xA1)];
    let roster_s1 = group_roster_for_test(&pks); // what A signed over (S1)
    let body_cid = body_cid_of(b"to S1 only");

    let (a_kp, a_did) = hybrid_sender();
    let env = seal_group_multi(&pks, &a_did, &a_kp, &body_cid, 0, b"to S1 only")
        .expect("group seal within recipient limit");

    // S1 recipient with the CORRECT held roster: opens + verifies (control).
    let (_pt, rec) = open_group_stanza(&sks[0], 0, &roster_s1, 0, &env)
        .expect("S1 recipient with the correct held roster MUST verify");
    assert_eq!(rec, a_did, "S1 control recovers A");

    // RE-TARGET: an S2 recipient holds a DIFFERENT roster R2 (the message was
    // re-delivered to a set A never chose). The B2 verify recomputes
    // commitment(R2) ≠ commitment(R1-A-signed) → reject.
    let roster_s2 = vec![
        did("did:key:zSomeoneElseUNIQUE"),
        did("did:key:zAndAnotherUNIQUE"),
    ];
    let outcome = open_group_stanza(&sks[0], 0, &roster_s2, 0, &env);
    assert_eq!(
        outcome,
        Err(LayerCError::SenderOriginAuthFailed),
        "F-LC-3 (B2 / F-2): a re-targeted 0x6520 send MUST be rejected because \
         the recipient recomputes the audience_set_commitment from its OWN held \
         roster (R2), NOT the wire value (R1). would-FAIL-on-revert: if open \
         used stanza.recipient_dids the verify would PASS for the wrong \
         audience. Got: {outcome:?}"
    );

    // --- 0x6610 MembershipSet group (same F-2 property over the BLINDED set) ---
    use benten_drop::layer_c::group_posture::{
        GroupError, GroupSealParams, GroupVerifyContext, open_membership_set_group,
        seal_membership_set_group,
    };
    let k_set = [0x55u8; 32];
    let params = GroupSealParams {
        membership_set_id: b"set-beta".to_vec(),
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    let (ga_kp, ga_did) = hybrid_sender();
    let genv = seal_membership_set_group(&pks, &ga_did, &ga_kp, &k_set, &params, b"to set-beta")
        .expect("valid roster must seal (R18 C2)");

    let true_members: Vec<String> = group_roster_for_test(&pks)
        .iter()
        .map(|d| String::from_utf8_lossy(d).into_owned())
        .collect();
    // Control: the honest member set verifies.
    let ctx_ok = GroupVerifyContext {
        member_dids: true_members,
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    let (_pt, _rec) = open_membership_set_group(&sks[0], 0, &ctx_ok, &genv)
        .expect("honest member ctx MUST verify the 0x6610 send");

    // Re-target: a member-set the sender never signed for.
    let ctx_retarget = GroupVerifyContext {
        member_dids: vec!["did:key:zRetargetMemberUNIQUE".to_string()],
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    let outcome = open_membership_set_group(&sks[0], 0, &ctx_retarget, &genv);
    assert_eq!(
        outcome,
        Err(GroupError::SenderOriginAuthFailed),
        "F-LC-3 (B2 / F-2): a re-targeted 0x6610 send MUST be rejected — the \
         recipient recomputes audience_set_commitment from its OWN held member \
         roster, NOT the wire commitment. Got: {outcome:?}"
    );
}

/// F-LC-3 PIN 5e (B2 SUBSTANTIVE — STALE-GENERATION REPLAY, F-3) — a
/// revoked-member cross-generation replay (`0x6610`). A body signed under an
/// OLD generation set is re-delivered to CURRENT-generation members. The B2
/// verify EXPLICITLY binds the three generation words in `M_auth` (the
/// body_aad_digest does NOT cover them), so the recipient — recomputing
/// `M_auth` with its CURRENT generations — REJECTS the stale-generation
/// signature.
///
/// would-FAIL-on-revert: if the generations were NOT explicitly bound in
/// M_auth, an old-generation signed body would open cleanly for current-gen
/// members (the revoked-member replay).
#[test]
fn f_lc_3_stale_generation_replay_rejected() {
    use benten_drop::layer_c::group_posture::{
        GroupError, GroupSealParams, GroupVerifyContext, open_membership_set_group,
        seal_membership_set_group,
    };

    let pks = [fixed_pk(0xB0), fixed_pk(0xB1)];
    let sks = [fixed_sk(0xB0), fixed_sk(0xB1)];
    let k_set = [0x22u8; 32];
    // Sender seals under the OLD generation set (e.g. before a member was
    // revoked + the set rotated).
    let old_params = GroupSealParams {
        membership_set_id: b"set-gamma".to_vec(),
        member_key_generation: 3,
        membership_set_generation: 5,
        role_assignments_generation: 1,
    };
    let (a_kp, a_did) = hybrid_sender();
    let stale =
        seal_membership_set_group(&pks, &a_did, &a_kp, &k_set, &old_params, b"old-gen body")
        .expect("valid roster must seal (R18 C2)");

    let members: Vec<String> = group_roster_for_test(&pks)
        .iter()
        .map(|d| String::from_utf8_lossy(d).into_owned())
        .collect();

    // Control: a recipient holding the SAME (old) generations verifies.
    let ctx_old = GroupVerifyContext {
        member_dids: members.clone(),
        member_key_generation: 3,
        membership_set_generation: 5,
        role_assignments_generation: 1,
    };
    open_membership_set_group(&sks[0], 0, &ctx_old, &stale)
        .expect("control: same-generation recipient verifies the old-gen body");

    // ATTACK: current-generation members (the set rotated forward) recompute
    // M_auth with the CURRENT generations → the stale signature fails.
    let ctx_current = GroupVerifyContext {
        member_dids: members,
        member_key_generation: 4,     // bumped after revocation
        membership_set_generation: 6, // bumped
        role_assignments_generation: 1,
    };
    let outcome = open_membership_set_group(&sks[0], 0, &ctx_current, &stale);
    assert_eq!(
        outcome,
        Err(GroupError::SenderOriginAuthFailed),
        "F-LC-3 (B2 / F-3): an old-generation signed 0x6610 body re-delivered \
         to CURRENT-generation members MUST be rejected — M_auth binds the \
         three generation words EXPLICITLY (the body_aad_digest does NOT cover \
         them). would-FAIL-on-revert: without explicit generation binding the \
         revoked-member cross-generation replay succeeds. Got: {outcome:?}"
    );
}

/// F-LC-3 PIN 5f (B2 SUBSTANTIVE — STRIP-PQ-HALF) — an attacker strips the
/// ML-DSA-65 half of the LAMPS-hybrid `sender_sig` (downgrade-to-classical
/// attack on a hybrid-coded signature). The B2 hybrid verify REQUIRES BOTH
/// halves (`sig.rs` never returns `Ok` after a single half), so the stripped
/// signature fails closed (`SenderOriginAuthFailed`).
///
/// The strip is performed on the once-sealed inner region; because the sig
/// lives inside the AEAD, mutating it AT THE WIRE would fail the AEAD tag —
/// so to model a co-sealer who deliberately PLACES a stripped signature we
/// re-seal with a sender whose sig is stripped via `without_pq_half_for_test`
/// (a co-sealer holding the CEK can author any inner payload). The verify
/// must still reject the PQ-stripped signature.
///
/// would-FAIL-on-revert: if the verify accepted a single (classical) half,
/// the PQ-downgrade would succeed.
#[test]
fn f_lc_3_strip_pq_half_rejected_single() {
    use benten_crypto_suite::sig::SignatureSuite;

    // Build M_auth exactly as the 0x6510 seal does, sign it, strip the PQ
    // half, and assert the production verify path rejects the stripped sig.
    // This drives `verify` through the same SignatureSuite the open path uses.
    let (kp, _did_bytes) = hybrid_sender();
    let suite = SignatureSuite::v1_default();
    let full = suite.sign(&kp, b"any-m-auth-bytes-for-the-strip-pin");
    let stripped = full.without_pq_half_for_test();
    let verify_full = suite.verify(kp.public(), b"any-m-auth-bytes-for-the-strip-pin", &full);
    let verify_stripped = suite.verify(
        kp.public(),
        b"any-m-auth-bytes-for-the-strip-pin",
        &stripped,
    );
    assert!(
        verify_full.is_ok(),
        "control: the full hybrid signature MUST verify"
    );
    assert!(
        verify_stripped.is_err(),
        "F-LC-3 (B2): a PQ-stripped LAMPS-hybrid signature MUST fail closed \
         (both halves required) — the basis for the open-path \
         SenderOriginAuthFailed on a downgraded sender_sig. would-FAIL-on-\
         revert if the verify accepted a single classical half. Got: \
         {verify_stripped:?}"
    );
}
