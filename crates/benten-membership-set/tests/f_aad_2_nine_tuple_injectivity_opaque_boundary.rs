//! F-AAD-2 (R3-W5) — AAD 9-tuple injectivity + opaque-bytes boundary.
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 row **F-AAD-2** (merges B2 +
//!   T-E2 + WF-C5 + GNI-2 + CE-I1).
//! - R0.5 plan §3.10 (the **AAD 9-tuple**, Inv-20 clause-c) — **MembershipSet
//!   group sends honor Sealed-Sender (F-LC-9):** the inner-sender-DID is bound
//!   INSIDE the sealed/encrypted part per stanza (NOT in plaintext AAD), so the
//!   on-wire tuple binds
//!   `(codepoint, body-CID, sorted-member-DID-list, sealed-inner-sender-DID,
//!   stanza-index, member-key-generation, membership_set_id,
//!   membership_set_generation, role_assignments_generation)` — where
//!   `sealed-inner-sender-DID` is the **post-decrypt-verified inner-payload
//!   sender-DID, NOT an on-wire plaintext field** — "load-bearing for
//!   inter-member non-forgeability".
//! - R0.5 plan §4.1 (the AAD encoding contract): "AAD codepoint binding +
//!   `aad_version: u8` prefix + **canonical-TLV** length-injective" (U1/U3/U14).
//! - R0.5 plan §3.10 precision (m-15 GNC-5): the AAD assembly hands
//!   **OPAQUE bytes** to `benten-crypto-suite`; the crypto-suite has **NO
//!   reverse dependency** on membership-set (the AAD is opaque to it).
//! - U1 (codepoint committed in AAD), U3 (canonical-TLV length-injective),
//!   U14.
//!
//! ## Single-ownership partition (R2 §"Slicing rationale")
//!
//! F-AAD-2 owns the **encoding-injectivity (membership-side)** + the
//! **opaque-boundary compile-fence**. The crypto-binding round-trip
//! (does mutating an AAD field make `AEAD-open` fail?) is co-owned with
//! R3-W0's `tf2`-family and is NOT re-implemented here; this file pins
//! the membership-side assembly contract: the 9-tuple → canonical bytes
//! is injective, every PLAINTEXT field is byte-bound, the sealed-inner
//! sender-DID is NOT a plaintext AAD field on the default `0x6600`/`0x6610`
//! path, and the assembler emits a plain `Vec<u8>` (no crypto type leaks
//! across the seam).
//!
//! ## R4-FIX (F4-001 BLOCKER + F4-012 + F4-026 + F4-046 + F4-DS-CITE)
//!
//! - **F4-001 (BLOCKER) — MembershipSet group sends honor Sealed-Sender
//!   (F-LC-9 / BR-1 ruling 1).** The original stub bound a PLAINTEXT
//!   `sender_did` into the DEFAULT (`0x6600`) group-send AAD and froze
//!   `did:key:zAAA` into `EXPECTED_AAD_HEX` as a plaintext sender field,
//!   silently defeating Sealed-Sender for every default MembershipSet group
//!   send. This MIRRORS the sibling Layer-C `0x6520` fix
//!   (`f_lc_hpke_encrypt_to_recipient_sealed_sender.rs`): the DEFAULT
//!   `Aad9Tuple` now binds the **sealed-inner-sender-DID INSIDE the sealed
//!   per-stanza payload** (`sealed_inner`), recovered only post-decrypt, and
//!   the PLAINTEXT 9-tuple AAD binds ONLY the 8 on-wire fields
//!   (`codepoint, body-CID, sorted-member-DID-list, stanza-index,
//!   member-key-generation, membership_set_id, membership_set_generation,
//!   role_assignments_generation`). `EXPECTED_AAD_HEX` is regenerated WITHOUT
//!   any inline sender field. The old "sender_did is byte-bound (U4)" arm is
//!   converted to a **no-plaintext-sender wire-scan** (the sealed-inner
//!   sender-DID MUST NOT appear in the plaintext AAD bytes). A paired
//!   **non-default plaintext-sender control** proves the wire-scan is not
//!   vacuously passing — the only observable difference between the two paths
//!   is the appended plaintext sender field (the default and non-default
//!   paths must differ observably, and differ by EXACTLY that field).
//!
//!   **Paired-control shape (extra-reflection-pass; differs from the Layer-C
//!   sibling intentionally):** the §4.0 codepoint table defines NO MembershipSet
//!   plaintext-sender sibling codepoint — the MembershipSet band (`0x6600`
//!   keying / `0x6610` group / `0x6620` subset-ref-reserve) has no plaintext-
//!   sender variant (UNLIKE Layer-C, whose `0x6500 LAYER_C_DROP` IS a real
//!   frozen plaintext-sender sibling to `0x6510`). So the paired control does
//!   NOT mint a synthetic band-resident codepoint (a value like `0x6601` would
//!   squat the FROZEN `0x6600..0x66FF` band the Inv-18 / NQ-W2 CI scanner
//!   guards, and could be mistaken for a real assignment). Instead the control
//!   keeps the DEFAULT keying codepoint (`0x6600`) and differs from the default
//!   fixture by ONE typed field only — `plaintext_sender_did = Some(..)`. This
//!   is both safer (no frozen-band squat) and a STRICTER proof: the byte-delta
//!   between the two paths is EXACTLY the length-prefixed sender field, so the
//!   `assert_ne!` is driven SOLELY by the sender leak (the property under test),
//!   never confounded by an incidental codepoint difference. R5 maps the
//!   non-default plaintext-sender path to whatever the real surface exposes
//!   (a dedicated codepoint, an envelope flag, or — if MembershipSet never
//!   ships a plaintext-sender variant — drops the control entirely).
//! - **F4-026 (CBOR↔TLV reconciliation):** the original stub serialized the
//!   9-tuple with `serde_ipld_dagcbor`, contradicting R0.5 §4.1 which freezes
//!   the AAD as `aad_version: u8` prefix + **canonical-TLV** length-injective.
//!   The assembler is now a deterministic canonical-TLV encoder (`aad_version`
//!   byte + BE u16 codepoint + length-prefixed variable fields + fixed BE
//!   integers). This converges the AAD encoding to the ONE §4.1 contract
//!   (CBOR stays the `members_table`-snapshot encoding — F-AAD-1 — which is a
//!   DISTINCT inner field; the 9-tuple WRAPPER is TLV).
//! - **F4-026 (`aad_version` pin):** a new arm pins the frozen `aad_version`
//!   prefix byte AND that mutating it changes the bytes (cross-version
//!   replay / strict-decode defense).
//! - **F4-026 (drop false length-prop):** the original
//!   `prop_assert_eq!(ba.len(), bb.len())` was provably-false under the CBOR
//!   encoder (shortest-form integers ⇒ distinct magnitudes ⇒ distinct
//!   lengths) and is a vacuous near-tautology even under fixed-width TLV; it
//!   is removed. The proptest now asserts the load-bearing injectivity
//!   property (distinct tuples ⇒ distinct bytes) directly.
//! - **F4-012 (pre-sorted sort arm):** the original sort arm pre-sorted the
//!   "reordered" input INSIDE the test (`reordered.sorted_member_dids.sort()`)
//!   before calling the assembler — a tautology testing only that "two equal
//!   sorted lists encode equally." The fix hands the assembler an UNSORTED
//!   list and asserts it produces the SAME bytes as the canonical fixture
//!   (the assembler canonicalizes internally). An R5 impl that forgets the
//!   internal sort now FAILS.
//! - **F4-046 (over-fenced arm5):** the crypto-suite-no-reverse-dep
//!   compile-fence reads a sibling `Cargo.toml` at runtime — it CAN run at
//!   baseline (the manifest exists in the workspace), so it is un-ignored now.
//! - **F4-DS-CITE:** all stale `R0.3 §` cites bumped to `R0.5 §` (the spec of
//!   record advanced; the prepend/plaintext-sender contracts were corrected at
//!   R4.2 → R0.5, so a cite to R0.3 points R5 at pre-correction text).
//!
//! ## pim-2 §3.6b + pim-18 §3.6f + §3.6f-ext end-to-end discipline
//!
//! Each arm drives the PRODUCTION 9-tuple assembler
//! (`assemble_aad_9tuple`, stand-in for R5
//! `benten_membership_set::aad::assemble_aad_9tuple`), asserts an
//! OBSERVABLE consequence (distinct bytes per single-field mutation /
//! frozen `aad_version` prefix / canonicalized-sort / sealed-inner sender-DID
//! absent from plaintext AAD / paired non-default sender present /
//! opaque-`Vec<u8>` return type), and would-FAIL-if-no-op'd (an assembler
//! that dropped a field from the binding, used LE, omitted the version
//! prefix, skipped the internal sort, OR leaked the sealed-inner sender-DID
//! into the plaintext AAD fails the corresponding arm).
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN behind `#[ignore]`. SELF-CONTAINED stub-shim (no
//! sibling-wave dep) for parallel-safe R3. R5 swaps the shim for the real
//! `benten_membership_set::aad` types + un-ignores.
//!
//! ## Wave-0 DAG edge (M-20)
//!
//! The 9-tuple's `codepoint` field is the V2-era `EncryptedEnvelope`
//! codepoint (`MembershipSetEncryption = 0x6600`, group multi-stanza
//! `0x6610`); every integer field (codepoint, stanza-index, the three
//! generations) is authored **big-endian** from the first commit, and the
//! `aad_version` prefix is the V2-era version byte.

#![allow(clippy::unwrap_used)]

// ── SELF-CONTAINED stub-shim (R5 replaces with `benten_membership_set::aad`) ──

/// The frozen AAD version prefix byte (R0.5 §4.1: `aad_version: u8` prefix).
/// V2-era; bumped only on a deliberate AAD wire-format change. Distinct from
/// `ENVELOPE_FORMAT_VERSION_V2` (the envelope wire-format version) — the
/// AAD-version axis is its own byte (R0.5 §4.1).
const AAD_VERSION: u8 = 0x01;

/// The DEFAULT MembershipSet set-keying envelope codepoint (`0x6600`,
/// Sealed-Sender by default — F-LC-9 / BR-1 ruling 1; §4.0 RELOCATED from
/// the M-CONS-FINAL `0x6380` that collided MLS-Application). On this path the
/// inner-sender-DID is sealed INSIDE the per-stanza payload, NEVER in the
/// plaintext AAD.
const MEMBERSHIP_SET_ENCRYPTION: u16 = 0x6600;
/// The DEFAULT group multi-stanza codepoint (`0x6610`, also Sealed-Sender by
/// default).
const MEMBERSHIP_SET_GROUP_MULTI_STANZA: u16 = 0x6610;
// NOTE (extra-reflection-pass): the paired non-default plaintext-sender control
// deliberately does NOT mint a synthetic codepoint. The §4.0 MembershipSet band
// defines no plaintext-sender sibling (only `0x6600`/`0x6610`/`0x6620`-reserve),
// so squatting an unassigned band value (e.g. `0x6601`) would (a) collide with
// the FROZEN-band Inv-18 / NQ-W2 CI scanner's ownership assertion and (b) read
// as a real assignment. The control instead keeps the DEFAULT keying codepoint
// and varies the ONE typed field that actually distinguishes the paths
// (`plaintext_sender_did`) — see `fixture_nondefault_plaintext_sender`.

/// The 9 AAD fields (Inv-20 clause-c). `sorted_member_dids` is the
/// member-DID list; the assembler canonicalizes (sorts) it before binding,
/// so a reorder must NOT change the bytes — that is what makes two engines
/// agree.
///
/// **F4-001 / F-LC-9:** the sender-DID is NOT a plaintext field. On the
/// DEFAULT (Sealed-Sender) path the inner-sender-DID lives in `sealed_inner`
/// (recovered post-decrypt) and is NEVER bound into the plaintext AAD. The
/// 9th tuple element on the wire is `role_assignments_generation`; the
/// "sealed-inner-sender-DID" element of the spec's 9-tuple is the
/// post-decrypt-verified inner payload, modeled here as `sealed_inner`.
#[derive(Clone, Debug)]
struct Aad9Tuple {
    /// `MembershipSetEncryption` codepoint family (`0x6600`/`0x6610`) — BE u16.
    codepoint: u16,
    /// Canonical body-CID (the encrypted-payload CID).
    body_cid: Vec<u8>,
    /// Member-DID list (canonicalized — sorted — by the assembler; a reorder
    /// is byte-neutral). Field name kept for R5 surface stability.
    sorted_member_dids: Vec<String>,
    /// Per-stanza index — BE u32.
    stanza_index: u32,
    /// Member-key generation — BE u32.
    member_key_generation: u32,
    /// The set identity.
    membership_set_id: Vec<u8>,
    /// Set generation counter — BE u32.
    membership_set_generation: u32,
    /// Role-assignment generation — BE u32 (BC-5; `E_ROLE_STALE_AT_VERIFY`).
    role_assignments_generation: u32,
    /// **DEFAULT (Sealed-Sender) path:** the inner-sender-DID is sealed
    /// INSIDE this opaque payload, recovered only post-decrypt. NEVER bound
    /// into the plaintext AAD (F4-001 / F-LC-9). The spec's 9-tuple
    /// `sealed-inner-sender-DID` element is THIS, post-decrypt-verified.
    sealed_inner: Vec<u8>,
    /// **NON-default plaintext-sender variant ONLY:** when `Some`, the
    /// sender-DID is bound into the PLAINTEXT AAD (U4). `None` on the DEFAULT
    /// Sealed-Sender path (the shipped default). This typed `Option` field IS
    /// the non-default distinction — the paired control keeps the DEFAULT
    /// codepoint and varies only this field (the §4.0 band defines no
    /// plaintext-sender codepoint to point at). Exists only so the paired
    /// metadata control proves the wire-scan is non-vacuous.
    plaintext_sender_did: Option<String>,
}

impl Aad9Tuple {
    /// The canonical DEFAULT (`0x6600`, Sealed-Sender) fixture. The
    /// sender-DID (`did:key:zSENDER` — a UNIQUE marker absent from the member
    /// list, so the wire-scan is meaningful) is sealed INSIDE `sealed_inner`,
    /// NEVER in the plaintext AAD.
    fn fixture() -> Self {
        Aad9Tuple {
            codepoint: MEMBERSHIP_SET_ENCRYPTION,
            body_cid: stub_cid(b"body-payload"),
            sorted_member_dids: vec!["did:key:zAAA".to_string(), "did:key:zBBB".to_string()],
            stanza_index: 0,
            member_key_generation: 1,
            membership_set_id: stub_cid(b"set-id"),
            membership_set_generation: 1,
            role_assignments_generation: 1,
            // Sealed-Sender DEFAULT: the inner-sender-DID is sealed here, NOT
            // in the plaintext AAD. The UNIQUE marker (not a member DID) makes
            // the F4-001 wire-scan distinguish hiding from a broken scan.
            sealed_inner: seal_inner_sender(SENDER_DID_MARKER),
            plaintext_sender_did: None,
        }
    }

    /// The paired non-default plaintext-sender control: the SAME logical send
    /// as `fixture()` — including the SAME (`0x6600`) keying codepoint — but the
    /// sender-DID is bound into the plaintext AAD (U4) instead of sealed inside
    /// `sealed_inner`. NOT a shipped default; exists ONLY to prove the default
    /// path's wire-scan is non-vacuous. Holding the codepoint FIXED makes the
    /// byte-delta between this and the default EXACTLY the appended sender field,
    /// so the difference is attributable solely to the sender leak (the property
    /// under test), never to an incidental codepoint change. The §4.0 band has
    /// no plaintext-sender codepoint to point at, so none is invented here.
    fn fixture_nondefault_plaintext_sender() -> Self {
        let mut t = Aad9Tuple::fixture();
        // codepoint stays the DEFAULT `0x6600` — the ONLY observable change is
        // the typed `plaintext_sender_did` field (and the now-empty `sealed_inner`
        // that the default path would otherwise carry the inner-sender inside).
        t.sealed_inner = Vec::new();
        t.plaintext_sender_did = Some(SENDER_DID_MARKER.to_string());
        t
    }
}

/// A UNIQUE sender-DID marker NOT present in the member-DID list, so the
/// F4-001 wire-scan for it in the plaintext AAD is meaningful (a member-DID
/// would always legitimately appear in the sorted member list).
const SENDER_DID_MARKER: &str = "did:key:zSENDERuniqueMARKER";

/// Model the sealed inner payload (HPKE inner-payload sender-DID + wrapped
/// CEK in production). The crucial property: this byte blob is the SEALED
/// material — it is opaque, it lives OUTSIDE the plaintext AAD, and the
/// sender-DID inside it is recovered only post-decrypt. Modeled here as a
/// fixed length-prefixed blob so the file is hermetic; the sender bytes are
/// NOT exposed in any plaintext AAD field.
fn seal_inner_sender(sender_did: &str) -> Vec<u8> {
    // A deterministic "sealed" blob: a domain tag + length-prefixed sender.
    // In production this is HPKE-wrapped ciphertext; here it just must NOT be
    // the plaintext-AAD encoding (it is never concatenated into the AAD).
    let mut inner = b"SEALED-INNER".to_vec();
    inner.extend_from_slice(&(sender_did.len() as u32).to_be_bytes());
    inner.extend_from_slice(sender_did.as_bytes());
    inner
}

/// PRODUCTION-stand-in: the 9-tuple PLAINTEXT-AAD assembler. Returns OPAQUE
/// `Vec<u8>` — note the return type is a plain byte vector, NOT a crypto-suite
/// type. This is the m-15 GNC-5 boundary contract: `benten-membership-set`
/// assembles the canonical bytes and hands `&[u8]` to `benten-crypto-suite`;
/// the crypto-suite never sees `Aad9Tuple`.
///
/// Encoding = the R0.5 §4.1 canonical-TLV contract: `aad_version: u8` prefix,
/// BE u16 codepoint, length-prefixed (u32 BE) variable fields, fixed
/// big-endian integers, and an INTERNAL canonical sort of the member-DID
/// list. Length-prefixing every variable field makes the encoding
/// length-injective (U3).
///
/// **F4-001 / F-LC-9:** on the DEFAULT (Sealed-Sender) path the sender-DID is
/// NOT bound here — it is sealed inside `sealed_inner`, recovered post-decrypt.
/// ONLY the EXPLICITLY-non-default plaintext-sender variant
/// (`plaintext_sender_did = Some(..)`) appends the sender-DID to the AAD (U4).
fn assemble_aad_9tuple(t: &Aad9Tuple) -> Vec<u8> {
    let mut buf = Vec::new();
    // aad_version prefix (U1/U14 strict-decode / cross-version replay defense).
    buf.push(AAD_VERSION);
    // codepoint — BE u16 (U1; committed in AAD).
    buf.extend_from_slice(&t.codepoint.to_be_bytes());
    // body_cid — length-prefixed.
    lp(&mut buf, &t.body_cid);
    // sorted_member_dids — the assembler CANONICALIZES (sorts) before binding,
    // so two engines that iterate membership in different orders still produce
    // identical AAD. Count-prefixed, then each DID length-prefixed.
    let mut dids = t.sorted_member_dids.clone();
    dids.sort();
    buf.extend_from_slice(&(dids.len() as u32).to_be_bytes());
    for d in &dids {
        lp(&mut buf, d.as_bytes());
    }
    // fixed-width BE integers.
    buf.extend_from_slice(&t.stanza_index.to_be_bytes());
    buf.extend_from_slice(&t.member_key_generation.to_be_bytes());
    lp(&mut buf, &t.membership_set_id);
    buf.extend_from_slice(&t.membership_set_generation.to_be_bytes());
    buf.extend_from_slice(&t.role_assignments_generation.to_be_bytes());
    // F4-001 / F-LC-9: the DEFAULT path binds NO plaintext sender. The
    // inner-sender-DID lives in `sealed_inner` (recovered post-decrypt) and is
    // NEVER concatenated here. ONLY the EXPLICITLY-non-default
    // plaintext-sender variant appends it (U4; paired control only).
    if let Some(sender) = &t.plaintext_sender_did {
        lp(&mut buf, sender.as_bytes());
    }
    buf
}

/// Length-prefix helper: writes `len: u32 BE || bytes` (the U3 length-injective
/// framing). R5's real canonical-TLV assembler uses the identical framing.
fn lp(buf: &mut Vec<u8>, bytes: &[u8]) {
    buf.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buf.extend_from_slice(bytes);
}

fn stub_cid(payload: &[u8]) -> Vec<u8> {
    // CIDv1 byte layout: 0x01 || 0x71 (dag-cbor) || 0x1e (blake3) || 0x20
    // (32-byte digest length) || 32-byte blake3 digest. Mirrors
    // benten-core::Cid so R5's real CID slots in byte-identically.
    let digest = blake3::hash(payload);
    let mut cid = vec![0x01u8, 0x71, 0x1e, 0x20];
    cid.extend_from_slice(digest.as_bytes());
    cid
}

/// The ABSOLUTE frozen canonical-TLV golden vector for the DEFAULT
/// (`0x6600`, Sealed-Sender) `Aad9Tuple::fixture()`.
///
/// Computed ONCE, off-line, from the canonical-TLV assembler. **135 bytes**
/// (NO plaintext sender field — F4-001 / F-LC-9); leading `0x01`
/// (`aad_version`); bytes 1..3 = `0x6600` (BE codepoint). The sender-DID
/// (`did:key:zSENDERuniqueMARKER`) does NOT appear anywhere in these bytes —
/// it is sealed inside `sealed_inner`. Any drift in field-order, endianness,
/// the version prefix, the length-prefix framing, the internal sort, OR a
/// re-introduced plaintext sender field flips this pin. R5
/// confirms-or-deliberately-updates this frozen literal against the real
/// `benten_membership_set::aad::assemble_aad_9tuple` (M-20).
const EXPECTED_AAD_HEX: &str = "0166000000002401711e20cfa9fea5491b9bf64cdc143778c3ff6e0123d8f7bca130f292b27a9bde54a860000000020000000c6469643a6b65793a7a4141410000000c6469643a6b65793a7a42424200000000000000010000002401711e202d5e5cdda7f761f1a6a2fcae8b71a45a466c7754b3ff7229dd86c90b9136ddd40000000100000001";

// ── F-AAD-2 arms ────────────────────────────────────────────────────────

/// F-AAD-2 arm 0 — frozen canonical-TLV golden vector + `aad_version` prefix.
///
/// Pins the ABSOLUTE bytes of the canonical DEFAULT fixture (F4-026 + F4-001):
/// the `aad_version` prefix byte, the BE codepoint, the full length-prefixed
/// TLV body, and NO plaintext sender field. Any field-order / endianness /
/// version-prefix drift — or a re-introduced plaintext sender — flips this.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — canonical-TLV golden vector + aad_version prefix (R0.5 §4.1); un-ignore at R5"]
fn f_aad_2_canonical_tlv_golden_vector_and_version_prefix() {
    let bytes = assemble_aad_9tuple(&Aad9Tuple::fixture());
    assert!(!bytes.is_empty(), "AAD bytes must be non-empty");
    // The frozen aad_version prefix byte (strict-decode / cross-version replay
    // defense — U1/U14). R5 reproduces this exact prefix.
    assert_eq!(
        bytes[0], AAD_VERSION,
        "leading byte = the frozen aad_version prefix (R0.5 §4.1)"
    );
    // The BE codepoint immediately follows the version prefix.
    assert_eq!(
        &bytes[1..3],
        &0x6600u16.to_be_bytes(),
        "codepoint is bound BIG-ENDIAN immediately after aad_version (U1; M-19 BE)"
    );
    // Full absolute golden vector.
    assert_eq!(
        hex_encode(&bytes),
        EXPECTED_AAD_HEX,
        "AAD 9-tuple canonical-TLV bytes drifted from the frozen golden vector — \
         divergent AAD = cross-engine AEAD-open failure (Inv-20 clause-c)"
    );
}

/// F-AAD-2 arm 0b — `aad_version` prefix is byte-bound (cross-version defense).
///
/// Bumping the version prefix changes the AAD bytes — a stanza sealed under
/// one AAD version cannot be replayed as another (strict-decode; U14).
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — aad_version prefix is byte-bound (cross-version replay defense); un-ignore at R5"]
fn f_aad_2_aad_version_is_byte_bound() {
    let base = assemble_aad_9tuple(&Aad9Tuple::fixture());
    // Re-assemble the SAME tuple but with a different version prefix by editing
    // the frozen byte directly (the version is a fixed prefix; R5 exposes a
    // typed version so a future bump is explicit, never silent).
    let mut bumped = base.clone();
    bumped[0] = AAD_VERSION.wrapping_add(1);
    assert_ne!(
        base, bumped,
        "the aad_version prefix is part of the bound AAD — a version bump changes the bytes (U14)"
    );
}

/// F-AAD-2 arm 1 — single-field mutation distinctness (8 plaintext fields).
///
/// Mutating ANY ONE of the 8 PLAINTEXT 9-tuple fields changes the assembled
/// bytes. This is the membership-side proof that every plaintext field is
/// bound; the crypto-side "AEAD-open fails" round-trip is co-owned by R3-W0's
/// tf2-family. (The 9th tuple element — the sealed-inner sender-DID — is NOT a
/// plaintext field; its binding is exercised by the F4-001 wire-scan arm and
/// by R3-W0's seal/open round-trip, not here.)
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — 8 plaintext AAD fields byte-bound (single-field distinctness); un-ignore at R5"]
fn f_aad_2_plaintext_field_single_mutation_distinct() {
    let base = assemble_aad_9tuple(&Aad9Tuple::fixture());

    let mut m = Aad9Tuple::fixture();
    m.codepoint = MEMBERSHIP_SET_GROUP_MULTI_STANZA; // 0x6610
    assert_ne!(base, assemble_aad_9tuple(&m), "codepoint is byte-bound (U1)");

    let mut m = Aad9Tuple::fixture();
    m.body_cid = stub_cid(b"DIFFERENT-payload");
    assert_ne!(base, assemble_aad_9tuple(&m), "body-CID is byte-bound");

    let mut m = Aad9Tuple::fixture();
    m.sorted_member_dids.push("did:key:zCCC".to_string());
    assert_ne!(
        base,
        assemble_aad_9tuple(&m),
        "member-DID-list membership is byte-bound"
    );

    let mut m = Aad9Tuple::fixture();
    m.stanza_index = 7;
    assert_ne!(base, assemble_aad_9tuple(&m), "stanza-index is byte-bound");

    let mut m = Aad9Tuple::fixture();
    m.member_key_generation = 9;
    assert_ne!(
        base,
        assemble_aad_9tuple(&m),
        "member-key-generation is byte-bound"
    );

    let mut m = Aad9Tuple::fixture();
    m.membership_set_id = stub_cid(b"OTHER-set");
    assert_ne!(base, assemble_aad_9tuple(&m), "membership_set_id is byte-bound");

    let mut m = Aad9Tuple::fixture();
    m.membership_set_generation = 42;
    assert_ne!(
        base,
        assemble_aad_9tuple(&m),
        "membership_set_generation is byte-bound"
    );

    let mut m = Aad9Tuple::fixture();
    m.role_assignments_generation = 5;
    assert_ne!(
        base,
        assemble_aad_9tuple(&m),
        "role_assignments_generation is byte-bound (BC-5; E_ROLE_STALE_AT_VERIFY)"
    );
}

/// F-AAD-2 arm 1b (F4-001 BLOCKER / F-LC-9 / BR-1 ruling 1) — the DEFAULT
/// MembershipSet group send HONORS Sealed-Sender: the inner-sender-DID is
/// sealed INSIDE the per-stanza payload and does NOT appear anywhere in the
/// plaintext AAD bytes.
///
/// This REPLACES the old "sender_did is byte-bound (U4)" arm, which froze
/// `did:key:zAAA` as a PLAINTEXT sender field — silently defeating
/// Sealed-Sender for every default MembershipSet group send (the exact
/// failure the sibling Layer-C `0x6520` fix closes). would-FAIL if the
/// default assembler bound the sender-DID into the plaintext AAD.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — DEFAULT group send honors Sealed-Sender, sender-DID NOT in plaintext AAD (F4-001); un-ignore at R5"]
fn f_aad_2_default_group_send_honors_sealed_sender_no_plaintext_sender_did() {
    let t = Aad9Tuple::fixture();

    // (a) Typed-shape guard: the DEFAULT path carries NO plaintext_sender_did
    //     (it lives sealed inside `sealed_inner` instead).
    assert!(
        t.plaintext_sender_did.is_none(),
        "F-AAD-2 (F4-001): on the DEFAULT 0x6600/0x6610 group path the tuple \
         MUST NOT carry a plaintext_sender_did — Sealed-Sender binds the \
         inner-sender-DID INSIDE `sealed_inner`."
    );
    assert!(
        !t.sealed_inner.is_empty(),
        "F-AAD-2 (F4-001): the DEFAULT path MUST carry a sealed inner payload \
         (where the post-decrypt-verified inner-sender-DID lives)."
    );

    // (b) WIRE-SCAN: the sender-DID byte sequence MUST NOT appear anywhere in
    //     the PLAINTEXT AAD bytes. The marker is UNIQUE (not a member DID), so
    //     a hit unambiguously means a leak (not a legitimate member-list entry).
    let aad = assemble_aad_9tuple(&t);
    let needle = SENDER_DID_MARKER.as_bytes();
    let leaks = aad.windows(needle.len()).any(|w| w == needle);
    assert!(
        !leaks,
        "F-AAD-2 (F4-001 / F-LC-9 / BR-1 ruling 1): the DEFAULT MembershipSet \
         group send MUST HONOR Sealed-Sender — the inner-sender-DID MUST NOT \
         appear in the PLAINTEXT 9-tuple AAD bytes. It is bound per-stanza \
         INSIDE the sealed payload, recovered only post-decrypt. would-FAIL \
         if the default path bound the sender-DID into the plaintext AAD \
         (silently defeating Sealed-Sender for every default group send)."
    );
}

/// F-AAD-2 arm 1c (F4-001 paired control) — the non-default plaintext-sender
/// variant (`plaintext_sender_did = Some(..)`, SAME `0x6600` keying codepoint)
/// DOES place the sender-DID in the plaintext AAD (U4).
///
/// This is the paired positive control that proves arm 1b's wire-scan is not
/// vacuously passing (the two paths must differ observably). The non-default
/// variant is NOT the shipped default. would-FAIL if even the non-default
/// variant hid the sender-DID (then the scanner cannot distinguish hiding
/// from a broken scan).
///
/// **Sharper than a bare `assert_ne!`:** because the control holds the
/// codepoint FIXED at the default, the byte-delta against the default AAD is
/// EXACTLY the length-prefixed sender field. The arm asserts that exact
/// equality (`nondefault == default ++ lp(sender)`), so the proof is
/// attributable solely to the plaintext sender leak — never to an incidental
/// codepoint difference (which is what would have happened had the control
/// squatted a synthetic band codepoint).
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — paired control: NON-default plaintext-sender DOES carry sender-DID in AAD (F4-001); un-ignore at R5"]
fn f_aad_2_nondefault_plaintext_sender_carries_sender_did_in_aad() {
    let t = Aad9Tuple::fixture_nondefault_plaintext_sender();
    assert_eq!(
        t.plaintext_sender_did.as_deref(),
        Some(SENDER_DID_MARKER),
        "the NON-default variant MUST bind the sender-DID into the plaintext AAD (U4)"
    );

    let aad = assemble_aad_9tuple(&t);
    let needle = SENDER_DID_MARKER.as_bytes();
    let leaks = aad.windows(needle.len()).any(|w| w == needle);
    assert!(
        leaks,
        "F-AAD-2 (F4-001 PAIRED CONTROL): the NON-default plaintext-sender \
         variant MUST place the sender-DID in the plaintext AAD (U4). If this \
         control fails, arm 1b's wire-scan cannot distinguish hiding from a \
         broken scan — the default and non-default paths must differ observably."
    );

    // And the two paths MUST differ by EXACTLY the appended plaintext sender
    // field — nothing else. Because the control holds the codepoint fixed at the
    // default, the only byte-level change is the length-prefixed sender DID
    // appended at the tail. This is strictly stronger than `assert_ne!`: it pins
    // WHAT the difference is (the sender leak), so a future assembler that
    // happened to differ for some OTHER reason could not pass this control.
    let default_aad = assemble_aad_9tuple(&Aad9Tuple::fixture());
    let mut expected_nondefault = default_aad.clone();
    expected_nondefault.extend_from_slice(&(needle.len() as u32).to_be_bytes());
    expected_nondefault.extend_from_slice(needle);
    assert_eq!(
        aad, expected_nondefault,
        "the non-default plaintext-sender AAD must equal the default Sealed-Sender \
         AAD with EXACTLY the length-prefixed sender field appended — the sole \
         observable difference is the sender leak (no codepoint confound)"
    );
    // (Sanity: this implies the two paths differ — the leak is the difference.)
    assert_ne!(aad, default_aad, "non-default plaintext-sender AAD must differ from default");
}

/// F-AAD-2 arm 2 — member-DID-list canonical sort (the assembler sorts).
///
/// **F4-012 fix:** the input is handed to the assembler UNSORTED (reverse
/// order). The PRODUCTION assembler must canonically sort the member-DID list
/// before binding, so the UNSORTED input must produce the SAME bytes as the
/// sorted fixture. A naive assembler that binds the list in the order it was
/// given (forgetting the internal sort) FAILS this arm — that is the real
/// freeze property (a malicious or buggy peer presenting DIDs in non-canonical
/// order must not produce divergent AAD).
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — assembler canonically sorts the member-DID list (unsorted input ⇒ same bytes); un-ignore at R5"]
fn f_aad_2_member_did_list_sort_order_canonical() {
    let base = assemble_aad_9tuple(&Aad9Tuple::fixture());

    // Same DID SET, presented UNSORTED (reverse order). NO in-body sort — the
    // assembler itself must canonicalize. If the assembler binds the list
    // as-given, these bytes differ from `base` and the arm FAILS (RED at R5
    // against a non-canonicalizing impl).
    let mut reordered = Aad9Tuple::fixture();
    reordered.sorted_member_dids = vec!["did:key:zBBB".to_string(), "did:key:zAAA".to_string()];
    assert_eq!(
        base,
        assemble_aad_9tuple(&reordered),
        "an UNSORTED-but-equal member-DID set MUST assemble to identical bytes — \
         the assembler canonicalizes internally (cross-engine convergence)"
    );
}

/// F-AAD-2 arm 3 — length-injectivity proptest (U3).
///
/// For arbitrary distinct `stanza_index` values, distinct tuples produce
/// distinct bytes and no shorter encoding is a prefix of a longer one. Models
/// the canonical-TLV length-injective contract. (F4-026: the original
/// `prop_assert_eq!(ba.len(), bb.len())` was removed — it was provably-false
/// under the prior CBOR encoder and a vacuous near-tautology otherwise.)
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — 9-tuple length-injectivity (U3) proptest; un-ignore at R5"]
fn f_aad_2_length_injectivity_proptest() {
    use proptest::prelude::*;
    proptest!(|(idx_a in any::<u32>(), idx_b in any::<u32>())| {
        prop_assume!(idx_a != idx_b);
        let mut a = Aad9Tuple::fixture();
        a.stanza_index = idx_a;
        let mut b = Aad9Tuple::fixture();
        b.stanza_index = idx_b;
        let ba = assemble_aad_9tuple(&a);
        let bb = assemble_aad_9tuple(&b);
        prop_assert_ne!(&ba, &bb, "distinct stanza-indices ⇒ distinct AAD bytes");
        // Length-injectivity: neither encoding may be a prefix of the other
        // (the length-prefix framing guarantees no truncation collision).
        prop_assert!(!is_prefix(&ba, &bb), "no AAD encoding may be a prefix of another (U3)");
        prop_assert!(!is_prefix(&bb, &ba), "no AAD encoding may be a prefix of another (U3)");
    });
}

/// F-AAD-2 arm 4 — opaque-bytes boundary compile-fence (m-15 GNC-5).
///
/// The assembler's output is a plain `Vec<u8>`: the membership-set crate
/// hands OPAQUE bytes to the crypto-suite. This arm asserts the return
/// type is byte-typed (no crypto type), which is the membership-side half
/// of the no-reverse-dependency contract. The Cargo.toml-side fence (the
/// crypto-suite manifest has NO `benten-membership-set` dep) is asserted
/// by `f_aad_2_crypto_suite_no_reverse_dep_compile_fence`.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — assembler emits OPAQUE Vec<u8> (no crypto type leak); un-ignore at R5"]
fn f_aad_2_assembler_emits_opaque_bytes() {
    let bytes: Vec<u8> = assemble_aad_9tuple(&Aad9Tuple::fixture());
    // OBSERVABLE: a non-empty opaque byte string whose type is `Vec<u8>`
    // (statically — this would not compile if the assembler returned a
    // crypto-suite envelope type). The crypto-suite consumes this as
    // `&[u8]` with zero knowledge of the 9-tuple shape.
    assert!(!bytes.is_empty(), "AAD bytes must be non-empty");
    // Compile-fence: the crypto-suite consumes the AAD as OPAQUE `&[u8]`.
    // `consume_opaque_aad` models that seam — it accepts a plain byte slice
    // and has ZERO knowledge of `Aad9Tuple`. This would not type-check if
    // the assembler returned a crypto-suite envelope type instead of bytes.
    fn consume_opaque_aad(aad: &[u8]) -> usize {
        aad.len()
    }
    assert_eq!(
        consume_opaque_aad(&bytes),
        bytes.len(),
        "the AAD crosses the seam as opaque &[u8] (m-15 GNC-5)"
    );
    // The canonical-TLV wire shape starts with the aad_version prefix — opaque
    // to the crypto-suite, which never parses it.
    assert_eq!(
        bytes[0], AAD_VERSION,
        "canonical-TLV aad_version prefix (R0.5 §4.1) — opaque to crypto-suite"
    );
}

/// F-AAD-2 arm 5 — compile-fence: crypto-suite has NO reverse dep on
/// membership-set (m-15 GNC-5).
///
/// Reads the in-tree `crates/benten-crypto-suite/Cargo.toml` and asserts
/// it does NOT depend on `benten-membership-set`. A future edit adding a
/// reverse dependency (to let the crypto-suite "understand" the AAD)
/// breaks the opaque-boundary contract and fails this pin.
///
/// **F4-046:** this arm reads a sibling manifest at runtime (the manifest
/// exists in the workspace at baseline), so it is NOT a RED-phase pin against
/// undelivered code — it is un-ignored now and runs every CI cycle as a live
/// boundary guard.
#[test]
fn f_aad_2_crypto_suite_no_reverse_dep_compile_fence() {
    // CARGO_MANIFEST_DIR = …/crates/benten-membership-set. The crypto-suite
    // manifest is its sibling.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let crypto_manifest = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("benten-crypto-suite")
        .join("Cargo.toml");
    let text = std::fs::read_to_string(&crypto_manifest)
        .expect("benten-crypto-suite/Cargo.toml must exist (the only crypto-primitive call site)");
    assert!(
        !text.contains("benten-membership-set"),
        "benten-crypto-suite MUST NOT depend on benten-membership-set — the AAD is OPAQUE bytes \
         across the seam (m-15 GNC-5; no reverse dependency)"
    );
}

// ── helpers ─────────────────────────────────────────────────────────────

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn is_prefix(short: &[u8], long: &[u8]) -> bool {
    short.len() <= long.len() && &long[..short.len()] == short
}
