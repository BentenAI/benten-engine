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

use benten_drop::layer_c::{
    AAD_VERSION, BindingContext, BodyCidDigest, DROP_TO_RECIPIENT_SEALED_SENDER,
    ENVELOPE_FORMAT_VERSION, EncryptedEnvelope, HYBRID_X25519_MLKEM768, HpkeRecipientStanza,
    LAYER_C_DROP, LAYER_C_DROP_MULTI_RECIPIENT, LayerCError, audience_set_commitment,
    group_plaintext_aad_region, open_group_stanza, open_single, seal_group_multi,
    seal_group_multi_plaintext_sender, seal_plaintext_sender, seal_sealed_sender,
    self_describing_cid, serialize,
};

/// Hermetic per-seed recipient fingerprints / DID helpers (test-local; the
/// production `seal_*`/`open_*` expand these to a real deterministic hybrid
/// keypair internally). `fixed_sk(seed) = fixed_pk(seed) + 0x80` per byte so
/// the seal-pubkey-fingerprint is recoverable from the open-secret.
fn fixed_pk(seed: u8) -> [u8; 32] {
    [seed; 32]
}
fn fixed_sk(seed: u8) -> [u8; 32] {
    [seed.wrapping_add(0x80); 32]
}
fn fixed_body_cid_digest(seed: u8) -> BodyCidDigest {
    [seed; 32]
}
fn did(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
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
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid_digest(0xC1);
    let plaintext = b"layer-c single recipient payload".to_vec();

    let env = seal_sealed_sender(&pk, &audience, &sender, &body_cid, 0, &plaintext);
    let (recovered, recovered_sender) = open_single(&sk, &env)
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
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid_digest(0xC2);

    let env = seal_sealed_sender(&pk, &audience, &sender, &body_cid, 0, b"secret");
    let outcome = open_single(&wrong_sk, &env);

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
    let env = seal_sealed_sender(
        &fixed_pk(0x03),
        &did("did:key:zRecipientAudience"),
        &did("did:key:zAlice"),
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
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid_digest(0xD0);
    let plaintext = b"group payload".to_vec();

    let env = seal_group_multi(&pks, &sender, &body_cid, 0, &plaintext);

    for (idx, sk) in sks.iter().enumerate() {
        let (recovered, _recovered_sender) = open_group_stanza(sk, idx, &env)
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
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid_digest(0xD1);

    let env = seal_group_multi(&pks, &sender, &body_cid, 0, b"group payload");

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
    let outcome = open_group_stanza(&sks[0], 0, &tampered);
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
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid_digest(0xD2);

    let env = seal_group_multi(&pks, &sender, &body_cid, 0, b"group payload");

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

    let outcome = open_group_stanza(&sks[0], 0, &tampered);
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
    let env = seal_group_multi(
        &pks,
        &did("did:key:zAlice"),
        &fixed_body_cid_digest(0xD3),
        0,
        b"x",
    );

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
    let sender = did("did:key:zGroupSenderUNIQUEMARKER");
    let body_cid = fixed_body_cid_digest(0xD6);

    let env = seal_group_multi(&pks, &sender, &body_cid, 0, b"group payload");

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
    let sender = did("did:key:zGroupSenderUNIQUEMARKER");
    let body_cid = fixed_body_cid_digest(0xD7);

    let env = seal_group_multi_plaintext_sender(&pks, &sender, &body_cid, 0, b"group payload");

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
    let sender = did("did:key:zSenderAliceUNIQUEMARKER");
    let env = seal_sealed_sender(
        &fixed_pk(0x50),
        &audience,
        &sender,
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
    let sender = did("did:key:zSenderAliceUNIQUEMARKER");
    let env = seal_plaintext_sender(
        &fixed_pk(0x51),
        &audience,
        &sender,
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
    let sender = did("did:key:zCarol");
    let env = seal_sealed_sender(
        &pk,
        &audience,
        &sender,
        &fixed_body_cid_digest(0xE2),
        0,
        b"hi",
    );

    let (_pt, recovered_sender) =
        open_single(&sk, &env).expect("recipient MUST open the sealed-sender envelope");
    assert_eq!(
        recovered_sender, sender,
        "F-LC-3: the recipient MUST recover the bound inner sender-DID \
         post-decrypt. would-FAIL if the inner-payload sender-DID is not \
         carried/recovered."
    );
}

/// F-LC-3 PIN 5 — a FORGED inner sender-DID is rejected. The inner
/// sender-DID is bound such that tampering with it fails the post-decrypt
/// verify (Inv-16 sender-DID-or-Sealed-Sender clause). would-FAIL if the
/// inner sender-DID is unauthenticated (then anyone can spoof the sender).
#[test]
fn f_lc_3_forged_inner_sender_did_rejected() {
    let pk = fixed_pk(0x53);
    let sk = fixed_sk(0x53);
    let audience = did("did:key:zRecipientAudience");
    let sender = did("did:key:zCarol");
    let env = seal_sealed_sender(
        &pk,
        &audience,
        &sender,
        &fixed_body_cid_digest(0xE3),
        0,
        b"hi",
    );

    // Adversary tampers the ciphertext (where the inner sender-DID lives).
    let mut tampered = env.clone();
    if let EncryptedEnvelope::HpkeBase { ciphertext, .. } = &mut tampered
        && let Some(b) = ciphertext.first_mut()
    {
        *b ^= 0xFF;
    }

    let outcome = open_single(&sk, &tampered);
    assert!(
        matches!(
            outcome,
            Err(LayerCError::AeadAuthenticationFailed | LayerCError::InnerSenderDidForged)
        ),
        "F-LC-3: a forged/tampered inner sender-DID MUST be rejected at \
         open (AEAD-auth or post-decrypt verify). would-FAIL if the inner \
         sender-DID were unauthenticated. Got: {outcome:?}"
    );
}
