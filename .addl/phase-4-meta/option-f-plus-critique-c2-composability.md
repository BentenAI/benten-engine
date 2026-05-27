# Option F+ §6.2 envelope-layer-unification — CRITIQUE C2: cross-amendment COMPOSABILITY lens

**Branch:** `phase-4-meta-core/option-f-plus-critique-c2-composability`
**Role:** SENIOR CRYPTOGRAPHER + SYSTEMS ARCHITECT, composition-stress-test lens. Distinct from the 9 per-lens reviewers and the consolidator: I build the FULL encoder/decoder/security model resulting from adopting U1..U40 + C32..C44 + Inv-16/17/18 *together*, then I attack the composed shape.
**Date:** 2026-05-27
**Tree-state pre-flight:** worktree clean @ `2172cb6d` against `origin/main`; branched from `origin/phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16`.
**Authority:** ADVISORY. CONFIRM / CONTEST / REFINE the consolidator's recommendations per `feedback_review_finding_ground_truth_verify`. DISAGREE-WITH-EXPLANATION first-class.

---

## §1 Executive verdict + confidence

**Top-line.** The unified registry is **COMPOSABLE WITH 7 LOAD-BEARING REFINEMENTS** (see §6). It is NOT composable as-is. Three contradictions and four composition failures are reachable from a literal reading of U1..U40 + C32..C44 + Inv-16/17/18; all are closable by tightening cross-amendment seams, none requires re-architecting the §6.2 direction or rejecting any lens's contribution. Confidence on the composability-blockers identified: **HIGH**. Confidence on each individual refinement: **HIGH** on R-C1, R-C2, R-C3, R-C5, R-C7; **MED-HIGH** on R-C4, R-C6.

**Headline composability blockers (require fix-now refinement, not deferral):**

- **C-1 (CONTRADICTION):** **U22 Sealed-Sender vs U4 sender-DID-binding vs U17 multi-stanza cross-stanza AAD.** The cross-stanza substitution defense (per L9-A1) binds `sender_did` into per-stanza AAD. A Sealed-Sender envelope (U22) by definition does NOT bind sender_did into AAD. Therefore the Sealed-Sender multi-stanza variant (U17 ∘ U22 ∘ U25 — Signal SSv2 pattern) does not have a defined cross-stanza substitution defense in the unified registry. R-C1 (§6) closes.
- **C-2 (CONTRADICTION):** **U28 coarse 1-hour epoch buckets vs U5 ≤60s clock-skew tolerance vs Inv-17 hybrid-mandatory replay-cache.** U5 mandates a 60-second skew tolerance; U28 quantizes timestamps to 1-hour buckets with random jitter. After U28's bucketization the 60s skew becomes ill-defined (the wire value is `(unix_seconds + jitter[0..3600]) / 3600` — there is no granular `valid_until` to ±60s against). The replay-defense the consolidator says is preserved "via outer UCAN nonce-cache" is not specified anywhere in U5 / U28 / U17 / U19; UCAN is a separate authorization layer. R-C2 closes.
- **C-3 (CONTRADICTION):** **U17 dual-CID + U25 per-recipient-unlinkable vs U18 dual-CID model + Drop bundle composition.** U25 (Signal SSv2 pattern) says "each recipient gets a distinct, unlinkable copy with NO visible cross-recipient structure on the wire." But U18 defines `plaintext_cid` = BLAKE3 over canonical DropBundlePayload as the graph-stable referent. If every recipient gets a distinct unlinkable copy, either (a) every recipient sees a different `plaintext_cid` ⇒ no shared graph reference ⇒ defeats the central reason U18 exists; or (b) the plaintext_cid is shared across recipients ⇒ recipients can collude and observe "we got the same plaintext_cid" ⇒ per-recipient-unlinkability fails. R-C3 closes via explicit composition rule.
- **C-4 (COMPOSITION FAILURE — SCENARIO C):** **U19 recipient_key_generation vs U20 k_principal_generation vs L9 multi-device-key-wrap.** When Alice adds a new device while K_principal is rotating, the multi-device-key-wrap (Layer-D) wraps which K_principal generation? The unified registry binds `recipient_key_generation` to the *recipient's KEM key* (U19) and `k_principal_generation` to the Layer-A vault (U20), but there is no third field for "which K_principal generation is being delivered through this Layer-D wrap to the new device." A new device receiving K_principal=v2 may see Atrium-replicated rotation-log entries pointing to v1 — without a wrap-side generation pin, the device cannot validate the wrap is for the K_principal generation it expects. R-C4 closes.
- **C-5 (COMPOSITION FAILURE — SCENARIO D):** **U11 escape codepoint `0xFFFF` vs U2 strict-decode vs U29 cross-ecosystem-emit vs U16 CodepointLifecycle.** A v2 envelope using escape codepoint `0xFFFF` followed by varint-u32 lands at a v1 reader. U11 says "v1-beta MUST refuse `0xFFFF` (typed-reject `UnsupportedAlgorithm::Envelope`)". U2 strict-decode reads codepoint first ⇒ `0xFFFF` ⇒ refuse. **But what does U29 (cross-ecosystem-emit) do?** When v1 Benten translates a `0xFFFF` envelope to JWE/COSE/age, the cross_ecosystem_map has no row for `0xFFFF` (it's not a real codepoint; it's an escape sentinel). U29's discipline is "translate Benten-internal codepoint into matching cross-ecosystem identifier as content" — `0xFFFF` has none. Cross-ecosystem-emit fails silently or panics. R-C5 closes.
- **C-6 (COMPOSITION FAILURE — SCENARIO F):** **U4 sender-DID-in-AAD vs U5 replay-window-in-AAD vs Inv-17 hybrid-mandatory floor.** Eve replays an old envelope AND tampers with sender-DID-in-AAD simultaneously. The composed validation does correctly reject (AEAD tag-verify fails because either tamper invalidates the auth tag). **But the order of error reporting matters** for security-property logging — the registry does not specify which error wins (`SenderDidMismatch` vs `ReplayWindowExpired`). For audit-readiness per U40 THREAT-MODEL.md, error-disambiguation matters; for cryptographic security it does not. R-C6 closes (audit-deliverable refinement).
- **C-7 (INTERACTION BLIND SPOT):** **U24 padding-class disclosure × U29 cross-ecosystem-emit × U22 Sealed-Sender.** Size-class buckets per codepoint (U24) mean the bucket choice ITSELF reveals the codepoint family even when envelopes are Sealed-Sender (U22) — because the U24 specification ties buckets *per codepoint*. A Layer-A 4-KiB envelope and a Layer-C 1-KiB envelope are distinguishable by size even when both use Sealed-Sender. The L6 privacy promise "Sealed-Sender hides sender" composes with "U24 bucket discloses operation-type-via-size-distribution-correlation." R-C7 closes.

**Disagreements with consolidator (in addition to the contradictions/composition failures above):**

- **CONTEST consolidator's MF1 framing.** MF1 says "sender-authentication / metadata-privacy is a structural tension." That is correct for the *encryption substrate alone*. But the registry as composed *also* has L9 forkability semantics (Ben-ratified 2026-05-27), and forkability+sealed-sender compose into a *third* axis I name as a new finding (§5 Blind-Spot-2: forkability + sealed-sender + recipient-key-rotation form a permissions-revocation trilemma not visible to any single lens).
- **CONFIRM consolidator's MF4.** Compromise #31's cascading downstream-hazards (#32, #35, #42, #43) is correctly identified. I extend: **Compromise #31 ALSO cascades into the C-3 contradiction above** (per-recipient-unlinkable copies of forever-valid drops compound metadata-archive correlator power — adversary correlates *multiple* unlinkable copies of the same logical drop across years).
- **REFINE consolidator's Inv-18 merge.** The 3-way merge of L5+L6+L8 sibling invariants into a single Inv-18 is over-consolidated. Splitting into Inv-18a (registry-discipline) + Inv-18b (metadata-disclosure pairing) + Inv-18c (CodepointLifecycle typed-state) maps better to enforcement-site granularity (cite-drift-detector scanner for 18a; `PlaintextSenderInAadPattern` scanner for 18b; runtime decode-dispatch for 18c). See R-C8 (§6).

**On the consolidator's recommendation that the registry is "decision-ready for Ben to ratify before R0":** REFINE. The registry is decision-ready on the 28 amendments + 13 Compromises + 3 invariants individually; it is NOT decision-ready on cross-amendment composition rules. Adding ~12 explicit composition-rules (R-C1..R-C7 + Inv-18 split + 3 wave-decomposition cross-references) takes the registry to genuinely-decision-ready. ~1-2 wave-days of refinement work, no scope expansion.

---

## §2 Composition model — full encoder / decoder / validation paths with all 28 amendments applied

This section constructs the model that results from applying U1..U40 + C32..C44 + Inv-16/17/18 *together*. The model is the basis for §3-§5.

### §2.1 The composed `EncryptedEnvelope` struct

After all amendments, `EncryptedEnvelope` becomes (Rust-ish pseudocode; comments cite amendment origin):

```rust
// Outer framing per U30 (DAG-CBOR, CBOR-tag 0xBE54)
// per RFC 8949 §4.2.1 deterministic encoding; U7 BE multi-byte ints
#[derive(serde::Serialize, serde::Deserialize)]
#[cbor(tag = 0xBE54)]
pub struct EncryptedEnvelope {
    // U7 BE u16; U11 escape sentinel 0xFFFF; U16 lifecycle-state checked at decode
    pub codepoint: u16,

    // U9 non-exhaustive; U2 strict-dispatch
    pub payload: EnvelopePayload,

    // U10 non-exhaustive; U1 codepoint bound into canonicalized AAD via canonical_binding()
    pub aad_binding: BindingContext,
}

#[non_exhaustive]  // U9
pub enum EnvelopePayload {
    // U12 [u8;12] nonce variant
    SymmetricAead {
        ciphertext: Bytes,
        nonce: [u8; 12],
    },
    // U12 [u8;24] XNonce variant; U32 XChaCha20 for Layer-A + Layer-B
    SymmetricAeadXNonce {
        ciphertext: Bytes,
        nonce: [u8; 24],
    },
    // Single-recipient HPKE
    HpkeBase {
        enc: Bytes,
        ciphertext: Bytes,
    },
    // U17 multi-recipient stanza composition (Atrium A1)
    HpkeMultiBase {
        cek_aead_ciphertext: Bytes,
        cek_aead_nonce: [u8; 24],          // U32 XChaCha20
        stanzas: Vec<HpkeRecipientStanza>, // O(N) per-recipient
    },
    // U13 reserved (post-v1-beta impl, slot locked now):
    //   MlsApplication, MlsWelcome, CgkaCommit, BirdOfPreyAkem
    // U22 reserved: HpkeAuthSealedSender
    // U24 reserved: size-class padding bucket SHAPE per codepoint
    //   (padding is INSIDE ciphertext, length-prefixed; U24 §2 statement)
}

pub struct HpkeRecipientStanza {
    pub recipient_did: Did,                    // U15 multikey codepoint form
    pub recipient_key_generation: u32,         // U19
    pub hpke_encap: Bytes,
    pub wrapped_cek: Bytes,
    // Per-stanza AAD bound at stanza-seal-time per U17:
    //   (codepoint, plaintext_cid_OR_envelope_blob_cid, sorted_recipient_did_list,
    //    sender_did, stanza_index, recipient_key_generation)
    // NOTE: the (sender_did, sorted_recipient_did_list) elements are the
    //   load-bearing axis of contradiction C-1 against U22 Sealed-Sender.
}

#[non_exhaustive]  // U10
pub enum BindingContext {
    Vault {
        vault_version: u8,                     // L8 §2.7 + U16 lifecycle
        k_principal_generation: u32,           // U20 (Layer-A vault L9-A4)
        argon2_tier: u8,                       // L4 §2.2.2
        // NOTE: NO sender_did binding per U4 (Vault is at-rest, not transmitted)
        // NOTE: NO replay-window per U5 (Vault is at-rest)
    },
    PerNodeAead {
        node_cid: Cid,                         // BLAKE3 per Inv-5
        chunk_index: u32,                      // L4 §2.3.3
        k_principal_generation: u32,           // U20 (Layer-B AEAD AAD L9-A4)
        // NO sender_did, NO replay-window (per-Node-at-rest like Vault)
    },
    DropToRecipient {
        sender_did: Did,                       // U4
        audience_did: Did,                     // already-existing
        recipient_key_generation: u32,         // U19
        sealed_at_epoch_hour: u32,             // U28 (refines U5); jittered [0..3600s]
        valid_until_epoch_hour: u32,           // U28 (refines U5); ALL DropToRecipient
                                              //   per U5 "Vault + DropToRecipient EXCLUDED";
                                              //   BUT U28 still applies — composition C-2
    },
    DropToRecipientSealedSender {              // U22 codepoint slot 0x6510
        audience_did: Did,
        sealed_at_epoch_hour: u32,             // U28
        // NO sender_did in AAD (sender bound INSIDE ciphertext per Signal pattern)
    },
    DeviceLink {
        sender_device_did: Did,                // U4
        recipient_device_did: Did,             // U4
        recipient_key_generation: u32,         // U19
        sealed_at_epoch_hour: u32,             // U28 (refines U5)
        valid_until_epoch_hour: u32,           // U5
    },
    RemotePermission {
        granting_user_did: Did,                // U4
        requesting_device_did: Did,            // U4
        operation: PermissionOperation,        // U21 extends with ExecuteWorkflow
        recipient_key_generation: u32,         // U19
        sealed_at_epoch_hour: u32,             // U28
        valid_until_epoch_hour: u32,           // U5
    },
    // U13 reserved variant slots (locked at v1-beta):
    //   DropToGroup, MlsGroupApplication, MlsWelcome, AtriumSync,
    //   CapabilityRevocation, SealedSender, MetadataPrivateRouting,
    //   ExecuteWorkflow (U21 — slot reserved; impl post-v1-beta-day-one)
}

#[non_exhaustive]  // U10
pub enum PermissionOperation {
    ReadNode { node_cid: Cid },
    WriteNode { node_cid: Cid, expected_parent: Option<Cid> },
    DelegateCapability { capability_cid: Cid },
    ExecuteWorkflow {                          // U21
        workflow_cid: Cid,
        input_node_cids: Vec<Cid>,
        max_decrypt_count: u32,
        result_recipient_pubkey: HybridKemPubKey,
        executor_did: Did,
    },
}

pub enum Did {
    Multikey {                                 // U15
        codepoint: u64,                        // varint multikey codepoint
        raw_key_bytes: Bytes,
    },
    Unknown(u64, Bytes),                       // U15 typed-rejection
}
```

### §2.2 The composed `canonical_binding()` function — byte-by-byte

Per U1 + U3 + U14 + U7, the canonical_binding output stream for any encrypt-site is:

```
byte 0:                     aad_version  (U14; v1-beta = 0x01)
bytes 1-2:                  codepoint    (U7 BE u16; e.g. 0x6300 LAYER_C_DROP)
                                         (U11: if 0xFFFF, varint follows per escape)
bytes 3-end:                TLV-encoded BindingContext fields (U3 length-injective):
  tag-byte (U10 enum-variant ordinal mapped per L8 §2.3 registry)
  length-prefix (fixed-width u32 BE — U7)
  value-bytes (opaque per L8 §2.9)
  ... repeat for each field ...
  reserved tag 0xFF: extended canonicalization marker (U14; v1-beta MUST reject)
```

For a `DropToRecipient` example (default codepoint 0x6300; not Sealed-Sender):

```
0x01                                                            // aad_version
0x63 0x00                                                       // codepoint BE
0x01 [len][sender_did_multikey_bytes]                           // tag 1 = sender_did
0x02 [len][audience_did_multikey_bytes]                         // tag 2 = audience_did
0x03 [len][recipient_key_generation BE u32]                     // tag 3 = U19
0x04 [len][sealed_at_epoch_hour BE u32]                         // tag 4 = U28
0x05 [len][valid_until_epoch_hour BE u32]                       // tag 5 = U28
```

For a `Vault` example (codepoint 0x6100 = LAYER_A_VAULT_XCHACHA20):

```
0x01                                                            // aad_version
0x61 0x00                                                       // codepoint BE
0x01 [len=1][vault_version u8]                                  // tag 1
0x02 [len=4][k_principal_generation BE u32]                     // tag 2 = U20
0x03 [len=1][argon2_tier u8]                                    // tag 3
```

**Key observation:** the AAD stream is purely additive across BindingContext variants because TLV tag-bytes are per-variant-ordinal-derived per L8 §2.3.

### §2.3 Encoder path (composed)

1. **Caller** selects intent: `seal_vault` | `seal_per_node` | `seal_drop_to_recipient` | `seal_drop_to_recipient_sealed_sender` | `seal_device_link` | `seal_remote_permission`.
2. **Codepoint resolution** (U8 + U16 + U29):
   - Map intent → codepoint via `EncCodepoint` registry; assert `CodepointLifecycle::Live` (U16); refuse if `Deprecated` (writes refuse per U16), `Quarantined`, or `Burned`.
   - If caller targets cross-ecosystem boundary: invoke `cross_ecosystem_map(codepoint)` per U29; emit ecosystem-native identifier (JWE alg-name / COSE alg-id / age stanza name / multicodec / LAMPS OID); the Benten codepoint MUST NEVER appear directly at ecosystem boundary.
3. **BindingContext construction:** populate per-variant fields per §2.1; for non-Vault non-PerNodeAead variants bind sender_did per U4 (EXCEPT DropToRecipientSealedSender per U22 which omits sender_did).
4. **Replay-window field population** (U5 + U28):
   - DeviceLink, RemotePermission: `sealed_at_epoch_hour = (unix_seconds + random_jitter[0..3600]) / 3600`, `valid_until_epoch_hour = sealed_at + caller-specified-validity-window-hours`.
   - DropToRecipient: per U5 explicit EXCLUSION ("Vault + DropToRecipient EXCLUDED"), no replay-window fields. **BUT** the composed model has DropToRecipient with sealed_at_epoch_hour per U28 because U28 refines U5 across all variants where temporal-context is meaningful, and forever-valid-drops per Compromise #31 still benefit from a *creation*-time-bucket for audit-trail. **THIS IS CONTRADICTION C-2** — see §3.
5. **Multi-recipient path** (U17):
   - Generate CEK (32 random bytes).
   - For each recipient_i: HPKE-Seal(recipient_pubkey_i, AAD_i) where AAD_i = canonical_binding-of(codepoint, plaintext_cid_or_envelope_blob_cid, sorted_recipient_did_list, sender_did, stanza_index=i, recipient_key_generation_i).
   - The `wrapped_cek_i` is the per-stanza HPKE output; ciphertext-body uses CEK-derived AEAD with single shared `cek_aead_nonce`.
   - **Per-recipient-unlinkable mode (U25, Signal SSv2 pattern, future):** each recipient sees a distinct payload structurally indistinguishable from a single-recipient Sealed-Sender envelope; cross-recipient correlation invisible on wire. **CONTRADICTION C-3** arises: how does U25's distinct-copy-per-recipient compose with U18's shared plaintext_cid? — see §3.
6. **canonical_binding()** runs over AAD as §2.2.
7. **AEAD-Seal or HPKE-Seal** invoked with canonical_binding output as AAD/info.
8. **DAG-CBOR encode** the EncryptedEnvelope with CBOR-tag 0xBE54 per U30; produce wire bytes.
9. **Pad to size-class bucket** per U24 (random AEAD-encrypted fill inside ciphertext; length prefix INSIDE ciphertext).
10. **Compute CIDs** per U18: `plaintext_cid = BLAKE3(canonical(DropBundlePayload))` (stable; what graph references), `envelope_blob_cid = BLAKE3(encoded_envelope_bytes)` (transport handle for iroh-blobs).
11. **TransportEnvelope wrap** per U23 if traversing iroh-transport (post-v1-beta impl): `TransportEnvelope { transport_blinded_id: noise_per_hop_blind(envelope_blob_cid), envelope: <inner> }`.

### §2.4 Decoder path (composed)

1. **Receive bytes** (potentially U23-wrapped; strip transport blinding first).
2. **DAG-CBOR decode** per U30; assert CBOR-tag 0xBE54.
3. **Read codepoint** (U7 BE u16).
4. **Escape check** (U11): if codepoint == 0xFFFF → at v1-beta, REFUSE with `UnsupportedAlgorithm::EnvelopeEscapeCodepoint`. v2+ readers MAY read a varint-u32/u64 codepoint here. **COMPOSITION FAILURE C-5** at cross-ecosystem boundary — see §4.
5. **Lifecycle check** (U16): look up CodepointLifecycle; refuse if Burned; warn-then-admit if Quarantined; admit-readonly if Deprecated; full admit if Live.
6. **Strict-decode dispatch** (U2): codepoint → expected EnvelopePayload variant; structurally refuse if mismatched. NO cross-variant fallback.
7. **Strict-decode dispatch BindingContext** (U2 + U10): codepoint also pins expected BindingContext variant; refuse if mismatched.
8. **canonical_binding() reconstruct** per §2.2 from received BindingContext + codepoint + aad_version.
9. **AEAD-Open / HPKE-Open** with reconstructed AAD/info; AEAD tag-verify is the load-bearing integrity gate.
10. **Replay-window enforcement** (U5 + U28; only for DeviceLink + RemotePermission per U5 explicit exclusion of Vault + DropToRecipient):
    - assert `now_unix_hour <= valid_until_epoch_hour + clock_skew_hours` where `clock_skew_hours = ceil(60s / 3600s) = 1` per consolidator's "outer UCAN nonce-cache provides freshness inside 1-hour window" — but this is *under-specified*: see C-2 (§3).
11. **For multi-recipient** (U17): scan stanzas for matching `recipient_did + recipient_key_generation`; HPKE-Open the matched stanza; AEAD-Open the body using recovered CEK.
12. **Recipient-key generation check** (U19): if `recipient_key_generation` in stanza != any sk Bob currently retains (post-rotation; ≥1-year grace policy per L9-A3 + Compromise #31 extension), return `RecipientKeyGenerationOrphaned` (envelope-sealed-to-generation-I-no-longer-have).
13. **K_principal generation lookup** (U20) for Layer-A + Layer-B Open: read `k_principal_generation` from AAD; load matching K_principal from vault HashMap<u32, [u8;32]>; KDF K(N) for Layer-B per `K(N) = KDF(K_principal[gen], N.cid, gen)`.
14. **Strip size-class padding** (U24): read length prefix inside ciphertext; truncate.
15. **For multi-stanza Sealed-Sender (U22 + U25 + U17 composed):** post-HPKE-Open verify sender identity from inside-ciphertext sender_did (Signal Sealed Sender pattern); reject if signature/MAC mismatch.

### §2.5 Validation path (composed)

For audit-readiness per U40 THREAT-MODEL.md, the validation path is:

- **Inv-16 envelope-layer-unification** (U1-U20): every encrypt site dispatches through canonical_binding(); cite-drift-detector enforces no raw-AEAD calls bypassing the envelope shape.
- **Inv-17 hybrid-mandatory** (NEW invariant): every KEM site uses X-Wing (X25519+ML-KEM-768) or future-additive equivalent; cite-drift-detector scans for pure-classical or pure-PQ KEM codepoint mints and fails.
- **Inv-18 codepoint-registry-discipline + metadata-disclosure + CodepointLifecycle** (this critique REFINES to Inv-18a/18b/18c — see R-C8): scanner enforces (a) every `pub const ... = 0x...;` in crypto-suite has registry row; (b) every BindingContext variant placing identity-DIDs in AAD has paired Sealed-Sender sibling codepoint (PlaintextSenderInAadPattern scanner); (c) every codepoint has CodepointLifecycle state Live/Deprecated/Quarantined/Burned.

### §2.6 Full key-derivation chain (composed)

```
Password
  ↓ Argon2id(salt=vault.salt, params=tier_params[vault_version])
DAK (Derivation-Authentication-Key, ~32 bytes; ephemeral; in zeroize-wrapped memory)
  ↓ HKDF-Expand(DAK, info="benten/v1/k_principal", L=32)
K_principal[generation=g]                                  (U20: HashMap<u32, [u8;32]>)
  ↓ XChaCha20-Poly1305-Open(ciphertext=vault_blob[g], nonce=vault.nonce[g],
                            aad=canonical_binding(codepoint=0x6100, Vault{vault_version, k_principal_generation=g, argon2_tier}))
per-Node K(N) = HKDF-Expand(K_principal[g], info="benten/v1/per-node" || N.cid || g.to_be_bytes(), L=32)
  ↓ XChaCha20-Poly1305-Open(node.ciphertext, node.nonce,
                            aad=canonical_binding(codepoint=0x6200, PerNodeAead{node_cid=N.cid, chunk_index, k_principal_generation=g}))
plaintext Node body
```

For Layer-C drops:
```
sender's K_principal[g_sender] (loaded from vault as above)
  ↓ derive CEK = random 32 bytes (per envelope)
For each recipient_i in recipient_set:
  HPKE-Seal(recipient_pubkey_i,
            info=canonical_binding(codepoint=0x6301, DropToRecipient or DropToRecipientSealedSender variant fields),
            psk_id=stanza-aad-fields-per-U17)
      → (enc_i, wrapped_cek_i)
CEK-AEAD-Seal(body, cek_aead_nonce, aad=canonical_binding-of-envelope)
```

For Layer-D wraps to a new device:
```
admin-device's K_principal[g] AND new-device's hybrid KEM pubkey
  ↓ HPKE-Seal(new_device_pubkey,
              info=canonical_binding(codepoint=0x6320, DeviceLink variant fields),
              plaintext = K_principal[g] || metadata)
      → (enc, wrapped_K_principal_for_new_device)
```

### §2.7 The composed docs landscape

After all amendments, the docs produced are:
- `docs/THREAT-MODEL.md` (NEW; U40) — 25-row T-01..T-25 matrix + promises/non-promises
- `docs/CRYPTO-CODEPOINTS.md` (NEW; U8 + Inv-18a) — registry with hex/variant/primitive/AAD-spec/since-version/rationale/lifecycle/cross-ecosystem-row per row
- `docs/SECURITY-POSTURE.md` (EXISTING; mint #32-#44 + extend #31)
- `docs/KEY-LIFECYCLE.md` (NEW or extend; U19 + U20 + L4 §2.2.2)
- `docs/INVARIANT-COVERAGE.md` (EXISTING; mint Inv-16/17/18 or 18a/18b/18c per R-C8)
- `docs/V1-FROZEN-INTERFACE.md` (EXISTING; item 6 EnvelopeShape axis per U11)
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (EXISTING; Row D-SS-1 per U22, D-PAD-1 per U24, D-COVER-1 per U26)
- `docs/CRYPTO-PARAMETERS.md` (NEW; L5 deliverable)
- `docs/AUDIT-SCOPE-STATEMENT.md` (NEW; L5 deliverable)
- `crates/benten-crypto-suite/src/cross_ecosystem.rs` (NEW; U29 mapping table)

### §2.8 The composed test corpus

Per U37 + U38 + U39 + L5 §5:
- ~96 golden vectors across codepoints × amendments × primitives
- KAT tests against RFC 9180 §B.1 + X-Wing draft + libcrux corpus + age MLKEM768X25519 stanza vectors
- Negative tests: cross-codepoint substitution, variant confusion, sender substitution, replay-window expired, LE-vs-BE codepoint, TLV length-injectivity collision, escape codepoint refused at v1-beta, lifecycle Burned codepoint refused, recipient_key_generation orphaned, k_principal_generation mismatch
- Property tests: AAD round-trip injectivity (proven via kani U39); HpkeMultiBase cross-stanza substitution defense; per-recipient-unlinkable Signal-SSv2-pattern test
- CT-validation: nightly dudect + per-primitive harnesses (U38); excluding cargo-llvm-cov instrumentation per L4 §2.10.4
- **NEW per this critique:** composition-property tests per R-C1..R-C7 (see §6).

### §2.9 The composed invariant chain

```
Inv-1 (deterministic canonical encoding) — existing
  ↳ Inv-16 envelope-layer-unification (registers canonical_binding per U1)
Inv-5 (BLAKE3 CID) — existing
  ↳ Inv-15 signature-bundle-CID-NEVER-load-bearing — existing
  ↳ Inv-16 envelope-layer-unification — NEW (this consolidation; sibling to Inv-15)
    ↳ Inv-17 hybrid-mandatory — NEW (KEM-floor invariant)
    ↳ Inv-18a registry-discipline — NEW (per R-C8 split)
    ↳ Inv-18b metadata-disclosure pairing — NEW (per R-C8 split)
    ↳ Inv-18c CodepointLifecycle typed-state — NEW (per R-C8 split)
```

Compromise chain rooted at #31:
```
#31 (forever-valid Drop bundles) — existing
  ↳ #32 (Bernstein-Persichetti CCA queries scale with replay opportunity)
  ↳ #35 (compromised-device retroactive decryption against forever-valid)
  ↳ #42 (Layer-C FS-gap: long-term sk decrypts forever-valid drops)
  ↳ #43 (metadata-leak archive: forever-archived metadata)
  ↳ #31-extension (recipient-key-rotation grace window per L9-A3)
  ↳ NEW PER THIS CRITIQUE: cascades into C-3 (per-recipient-unlinkable copies compound metadata-archive over years)
```

---

## §3 Contradictions found

### C-1 (HIGH severity): U22 Sealed-Sender ⊥ U17 cross-stanza substitution defense

**Statement of contradiction.** U17's cross-stanza substitution defense (L9-A1 §2.5) binds `sender_did` into the per-stanza AAD as one of the load-bearing fields:

> "The canonical AAD/info passed to each stanza's HPKE-Seal MUST bind: (a) the codepoint; (b) the CEK-AEAD-ciphertext's CID; (c) the sorted recipient DID list (canonicalized); **(d) the sender's DID**; (e) the stanza index; (f) the recipient's expected key generation." — L9-A1 §2.5

U22 (Sealed-Sender per L6-Am7) defines a codepoint variant in which "sender DID is bound INSIDE the ciphertext via HPKE-mode-auth's psk_id binding ... AAD carries ONLY audience + coarse epoch (no sender_did in AAD)." — L6 §3.1.

**The composed multi-stanza Sealed-Sender pattern (U17 + U22 + U25 — Signal SSv2)** therefore has *two* incompatible requirements for the per-stanza AAD: U17 says bind sender_did; U22 says don't.

**Why no individual lens caught it.** L9 designed multi-stanza assuming default DropToRecipient codepoint (sender_did in AAD per U4). L6 designed Sealed-Sender assuming single-recipient. L8 designed `#[non_exhaustive]` permanence at the type level, not at the cross-stanza-AAD-shape level. The consolidator merged but did not stress the multi-stanza ∘ Sealed-Sender intersection.

**Why this matters.** L6/U25 LOCKS the per-recipient-unlinkable Signal-SSv2 pattern as **Inv-18 invariant at v1-beta** (consolidator §6 bucket B). Inv-18 invariants do not have an opt-out. If U22 is the only Sealed-Sender path and U22 conflicts with U17's defense, then either (a) the multi-stanza Sealed-Sender pattern is unsealable; or (b) the cross-stanza substitution defense degrades; or (c) a *new* substitution-defense shape is required for Sealed-Sender stanzas.

**Resolution (R-C1, §6).** The Sealed-Sender multi-stanza pattern uses HPKE-mode-auth's `psk_id` field for the cross-stanza binding instead of the AAD `sender_did` field. The Signal SSv2 design (https://signal.org/blog/sealed-sender-multi-recipient/) handles this exact constraint via per-recipient delivery tokens that uniquely bind each stanza without revealing sender to wire-observers. Benten can adopt: per-stanza AAD binds `(codepoint, CEK-CID, audience_did, stanza_index, recipient_key_generation, delivery_token=H(sender_sk_signing || stanza_index))`; the delivery_token is verifier-checkable post-decrypt against sender_did extracted from inside ciphertext.

### C-2 (HIGH severity): U28 1-hour buckets ⊥ U5 60-second clock-skew tolerance

**Statement of contradiction.** U5 mandates clock-skew tolerance of 60 seconds (L4 §2.8: "clock-skew tolerance 60s; wall-clock semantics"). U28 quantizes timestamps to 1-hour buckets with random jitter [0..3600s] subtracted before bucketization. After U28's bucketization, the wire value is `floor((unix_seconds + random_jitter[0..3600]) / 3600)`.

**The composed validation rule** would be: `now_unix_hour <= valid_until_epoch_hour + clock_skew_hours`. But what is `clock_skew_hours`? The 60-second tolerance from U5 is now sub-precision. Two valid readings:
1. `clock_skew_hours = 1` (round up) — admits any envelope that was valid in the *previous* hour. Replay-window is now effectively ~1 hour + caller-specified-validity. Replay-defense degraded by ~60x.
2. `clock_skew_hours = 0` (strict) — rejects envelopes at the bucket boundary even if they would have been valid 30 seconds ago. Liveness degraded.

**The consolidator notes** at U28: "outer UCAN nonce-cache provides freshness inside 1-hour window." But UCAN nonce-cache is a separate authorization layer, not specified in U5 / U28 / U17 / U19 / any of the registry rows.

**Why this matters.** U5 is the structural replay-defense for DeviceLink + RemotePermission. If U28 lands without specifying the composed clock-skew rule, the replay-window becomes per-implementation-decision, which is exactly the cross-implementation drift the consolidator's permanence amendments (U9-U14) are designed to prevent.

**Resolution (R-C2, §6).** Specify in U28 explicitly: replay-defense for DeviceLink/RemotePermission is enforced via the *outer* UCAN nbf/exp on the AuthorizationGrant (UCAN spec uses hours/days; matches naturally); the AAD-bound `valid_until_epoch_hour` is *defense-in-depth* with `clock_skew_hours = 1` (admit 1-bucket lookback). For Vault + DropToRecipient (U5 EXCLUDED), U28's `sealed_at_epoch_hour` is informational only — never validated at decrypt-time, only logged for audit-trail. Document this in the v1-beta registry row for U28 + cross-reference U5.

### C-3 (HIGH severity): U18 dual-CID ⊥ U25 per-recipient-unlinkable

**Statement of contradiction.** U18 defines `plaintext_cid` = BLAKE3 over canonical DropBundlePayload as the graph-stable referent (consolidator §2 U18: "stable across reseal + recipient-set evolution + cipher rotation — what the user's graph references"). U25 (Signal SSv2 pattern, codified as Inv-18 invariant clause at v1-beta) says: "each recipient MUST get a distinct, unlinkable copy with NO visible cross-recipient structure on the wire" (L6 §3.5).

If every recipient gets a distinct unlinkable copy, what is the shared `plaintext_cid`?
- **Option (a):** all recipients see the same `plaintext_cid` (because they decrypt the same plaintext) ⇒ recipients colluding observe "we got the same plaintext_cid" ⇒ per-recipient-unlinkability fails *retroactively after decryption*.
- **Option (b):** each recipient sees a different `plaintext_cid` (because each is distinct encoding) ⇒ no shared graph reference ⇒ defeats U18's "stable across reseal + recipient-set evolution + cipher rotation" semantic.

**Why this matters.** L9's forkability semantic (Ben-ratified 2026-05-27: "member-leaves-keeps-past-content") depends on pre-fork Drop CIDs surviving fork events. If per-recipient-unlinkable per U25 means per-recipient-distinct-CIDs, the forkability semantic breaks — Bob (left) and Alice (stayed) cannot reference the *same* pre-fork drop in their graphs.

**Note on Signal SSv2.** Signal SSv2 does not have a "stable plaintext CID" property because Signal envelopes are not content-addressed. Benten's IPLD/BLAKE3-CID model is different from Signal's message-bus model; the SSv2 pattern as L6 cited needs adaptation, not direct adoption.

**Resolution (R-C3, §6).** Distinguish three notions:
1. `plaintext_cid` (U18) — graph-stable; SAME across all recipients of the same logical drop; this is what makes the IPLD model work. Recipients holding `plaintext_cid` after decryption *can* collude — that's a feature for content-addressing, not a privacy break.
2. `envelope_blob_cid` (U18) — transport-mutable; CAN differ per recipient if U25 per-recipient-unlinkable is enabled.
3. **Per-recipient-unlinkability semantic (U25)** — applies to *wire-observability before decryption*, not to *post-decryption recipient collusion*. The Signal SSv2 threat model is "untrusted relay cannot link Alice→Bob and Alice→Carol as the same drop"; it does NOT promise "Bob and Carol cannot determine they got the same drop after both decrypting." Document this explicitly in U25's statement; document that U18's plaintext_cid + U25's per-recipient-unlinkability compose by having U25 apply only to the OUTER envelope-blob-cid axis (per-recipient-distinct), NOT to the INNER plaintext-cid axis (shared).

Without R-C3 the Inv-18 metadata-disclosure invariant (per L6 §6.2) over-claims a privacy property the design cannot deliver. The consolidator's MF1 framing under-spots this third axis.

---

## §4 Composition failures (per-scenario)

### Scenario A: 100 Drop bundles + recipient sk rotation

**Setup.** Alice sends 100 Drop bundles to Bob over 6 months. Bob is offline. Bob comes back online; his ML-KEM-768 sk rotated at month 3 per L9-A3. Decoder applies all amendments to process the 100 envelopes.

**Walk-through.** Bob's vault holds:
- `recipient_kem_sk[gen=1]` (active months 0-3; ≥1-year grace per Compromise #31 extension)
- `recipient_kem_sk[gen=2]` (active months 3-onwards)

Envelopes 1-50 (months 0-3) bear `recipient_key_generation = 1`; envelopes 51-100 bear `recipient_key_generation = 2`.

For each envelope: decoder reads codepoint → strict-dispatch (U2) → reconstruct canonical_binding (U1+U3+U14) → AEAD/HPKE-Open. Recipient_key_generation lookup (U19): envelope 1-50 → use sk[gen=1]; envelope 51-100 → use sk[gen=2]. Successful decode for all 100. **PASS.**

**What could go wrong but doesn't:** U20 k_principal_generation is independent of U19 recipient_key_generation. K_principal rotation on Bob's side is for Bob's own vault (Layer-A); the recipient_kem_sk rotation is for Layer-C. The two generations are orthogonal. No composition failure.

**What does go wrong:** if any of the 100 envelopes uses U22 Sealed-Sender multi-stanza per U25 Signal SSv2 pattern, **C-1 hits**: cross-stanza substitution defense is undefined. **R-C1 closes.**

### Scenario B: Eve compromises relay; 5-year archive; B-P CCA attack

**Setup.** Eve has all Bob-addressed envelopes for 5 years (10⁵+ envelopes per Bob per Compromise #43). Eve constructs CCA per L3/Am6 (Bernstein-Persichetti).

**Walk-through with U6 + U31 + Compromise #32 composed.** Bob's HPKE-Open uses libcrux-ml-kem Decap with `check-secret-independence` verified at compile time. Per L4 IMPL-A1 + L5/§4.5 AF-27, the Decap implementation has compile-time-verified secret-independence; each individual Decap call is CT. The CCA attack's per-query leakage is bounded structurally. **Mitigation is CT at the primitive layer.**

**But:** Compromise #32 is honestly disclosed; the *aggregate* CCA-query budget across 5 years × 10⁵ envelopes is large. The CT-at-each-call defense is the well-trodden mitigation; aggregate residual leakage requires a *separate* threat-model bound. The composed registry says "PARTIAL via libcrux + Compromise #32 disclosure" — this is correct as honest-disclosure, but the THREAT-MODEL.md (U40) row T-25 should explicitly bound aggregate-query CCA leakage. **PASS with audit-deliverable refinement (R-C6).**

**What does go wrong:** if U28's coarse 1-hour epoch buckets are interpreted as `clock_skew_hours = 0` (strict bucket boundary), envelopes at month 3 epoch transitions may be rejected as "expired" when Bob comes online. Bob misses content. Replay-defense degrades to per-implementation-decision. **C-2 hits; R-C2 closes.**

### Scenario C: New device + concurrent K_principal rotation

**Setup.** Alice adds a new device per L9-A4 multi-device-key-wrap. Concurrently, K_principal rotates (admin device triggers v1 → v2; Atrium-replicated rotation-log entry minted).

**Walk-through.** Admin device executes Layer-D wrap:
```
HPKE-Seal(new_device_kem_pubkey,
          info=canonical_binding(codepoint=0x6320,
                                 BindingContext::DeviceLink{
                                   sender_device_did, recipient_device_did,
                                   recipient_key_generation, sealed_at_epoch_hour, valid_until_epoch_hour
                                 }),
          plaintext = K_principal[??] || metadata)
```

**Which K_principal generation goes into the wrap?** U20 binds `k_principal_generation` into `BindingContext::Vault` (Layer-A) and Layer-B AAD. **Layer-D's `BindingContext::DeviceLink` has NO `k_principal_generation` field in the composed model (§2.1).** The new device, after HPKE-Open, receives raw `K_principal || metadata` bytes; it must look at Atrium-replicated rotation-log to determine which generation it just received.

**The composition failure:** the new device is asked to trust the Atrium-replicated rotation-log entry's `generation=2` claim to label the received K_principal. But the rotation-log is a separate signed Node; if the new device's Atrium-sync hasn't caught up, it sees `generation=1` claim. The wrap is for `generation=2`. The new device labels K_principal[2] as K_principal[1] in its local vault. Subsequent Layer-B Open for nodes encrypted with K(N)=KDF(K_principal[2], N.cid, 2) fails (KDF uses wrong generation input).

**Resolution (R-C4, §6).** Extend `BindingContext::DeviceLink` with `k_principal_generation: u32` field (NEW field, additive per U10 `#[non_exhaustive]`). The wrap directly self-identifies which K_principal generation it delivers. New device labels correctly regardless of Atrium-sync state. **Reservation cost: zero LOC at v1-beta if codepoint variant reserves the field slot now.**

### Scenario D: v2 envelope at v1 reader + cross-ecosystem-emit

**Setup.** v2 envelope uses U11 escape codepoint `0xFFFF` followed by varint-u32. v1 Benten reader receives v2 envelope. Caller asks v1 Benten to translate to JWE for a JOSE consumer.

**Walk-through.**
- v1 strict-decode per U2 reads codepoint = 0xFFFF.
- U11 says "v1-beta MUST refuse 0xFFFF (typed-reject `UnsupportedAlgorithm::Envelope`)".
- v1 reader returns `Err(UnsupportedAlgorithm::EnvelopeEscapeCodepoint)`. **CORRECT.**

**The composition failure:** the caller's intent was cross-ecosystem-emit per U29, not local decrypt. The cross_ecosystem_map(codepoint=0xFFFF) has no row (escape sentinel is not a real codepoint). U29's cite-drift-detector enforces "every codepoint has a cross-ecosystem-map row." For 0xFFFF, the registry either:
- (a) Has no row ⇒ cite-drift-detector fails the build of any v1+v2-aware reader.
- (b) Has a sentinel row `{ lamps_oid: None, jose_alg_name: None, ... }` ⇒ adapter must handle "no mapping" explicitly. None of U7, U8, U11, U29 specify the adapter behavior on escape sentinels.

**Resolution (R-C5, §6).** Specify in U29 (cross_ecosystem_map) that:
- Escape sentinel codepoint 0xFFFF + experimental range 0xFE00..0xFFFE have *explicit None-mapping* entries.
- Cross-ecosystem-emit adapters MUST return `Err(CrossEcosystemEmitError::EscapeCodepointNotEmittable)` rather than silently emitting an empty alg-name or panicking.
- v1 reader MAY (depending on caller intent) extract the inner varint-u32 codepoint and look up its cross_ecosystem_map row — but only if v1 has been forward-extended with a `CrossEcosystemEscapeMap` registry; at v1-beta, refuse.

### Scenario E: Vault K_principal rotation; unlock with new password

**Setup.** Alice's vault is encrypted under K_principal v1. Vault is re-keyed: new K_principal v2 minted, all per-Node K(N) re-derived under v2 (or held under v1 with rotation-log entry).

**Walk-through with U20 composed.**
- Vault layout: `HashMap<u32, [u8;32]>` of generation → K_principal[g] under DAK (per L9-A4).
- Vault root: `BindingContext::Vault { vault_version, k_principal_generation: u32 }`. AAD binds the generation per U20.
- Alice unlocks: Password → Argon2id(salt, params[vault_version]) → DAK → HKDF → … fetches K_principal[v1] AND K_principal[v2] from vault as separate AEAD-Open per-generation operations.
- Layer-B reads: each Node carries `k_principal_generation` in AAD per U20; decoder dispatches to correct K_principal[g]; KDF K(N) = KDF(K_principal[g], N.cid, g.to_be_bytes()).

**No composition failure.** The U20 design correctly handles vault re-keying. **PASS.**

**Subtle interaction:** if vault_version (e.g., Argon2id-param-tier upgrade) and k_principal_generation rotate independently, the AAD binds BOTH per §2.1. Wrong-tier Argon2id parameters produce wrong DAK → wrong K_principal → wrong KDF → AEAD-Open fails. Auth-tag verify catches. **Defense-in-depth works.**

### Scenario F: Eve replays + tampers sender-DID

**Setup.** Eve replays an old envelope (per Compromise #31; or against DeviceLink/RemotePermission per U5 valid_until). Eve also tampers with sender-DID-in-AAD (per U4) — substituting Alice→Bob with Mallory→Bob to confuse Bob's parser.

**Walk-through.** AAD = canonical_binding(codepoint, BindingContext{sender_did=Mallory, ...}). Bob's parser reconstructs AAD with sender_did=Mallory. AEAD tag-verify: tag was computed under original AAD with sender_did=Alice. Tag mismatch ⇒ AEAD-Open fails ⇒ entire decode fails with `AeadError::TagVerificationFailed`. **BOTH ATTACK VECTORS REJECTED via the same auth-tag verify.** No mask.

**Audit-deliverable concern (R-C6).** The error code is `TagVerificationFailed`, not `SenderDidMismatch` or `ReplayWindowExpired`. For THREAT-MODEL.md T-18 (Drop-bundle-replay) and T-20 (Cross-device-sync compromise), the audit firm will want disambiguation in test corpus + observability surface. Add diagnostic-only error-classification (post-tag-verify-failure, run cheap re-validation to determine which AAD field was tampered) for logging/test purposes ONLY — never as a security gate. Document in U40 THREAT-MODEL.md test corpus.

### Scenario G (NEW per this critique): forever-archive correlator over years × per-recipient-unlinkable copies

**Setup.** Compromise #43 says relay archives metadata forever. U25 per-recipient-unlinkable means each recipient sees a distinct envelope_blob_cid. Eve archives 5 years of relay traffic.

**Walk-through.** Eve sees ~10⁹ distinct envelope_blob_cids over 5 years. Each is structurally indistinguishable from independent Sealed-Sender envelopes (per U25 promise). Eve cannot trivially correlate "these 10 copies are the same logical drop to 10 recipients."

**BUT:** Eve also sees per-recipient size-class buckets (U24) AND per-recipient delivery timing (untrusted relay can timestamp arrivals). U24 buckets correlate across the 10 copies (same bucket size for same logical drop). Per-recipient delivery timing within ~1-hour bucket (U28) correlates if Alice sends to all 10 recipients in close succession.

**Composition consequence:** U25's per-recipient-unlinkability promise degrades against an *aggregate-traffic-analysis* adversary with size+timing side-channels even though it holds against a *single-envelope-link* adversary. This is the **forever-archive correlator extension** I flag as NEW downstream hazard of Compromise #31 (per §1 cascading-Compromise note).

**Resolution.** Document in #43 + U25 the bounded promise: "U25 per-recipient-unlinkability holds against a single-envelope-cross-recipient-link adversary; against an aggregate-traffic-analysis adversary with multi-year archive, residual correlation surface via U24 size + U28 timing remains. Closure of aggregate-traffic-analysis requires Sphinx-style onion-routing (post-v1; gated per U26)." Honest-disclosure refinement; no v1-beta engineering work.

---

## §5 Interaction blind spots (cross-amendment, not per-lens-visible)

### Blind-Spot-1: U24 padding-class × U22 Sealed-Sender ⇒ codepoint-leak-via-size

**The interaction.** U24 size-class buckets are specified per-codepoint. A Layer-A vault envelope pads to 4KiB/16KiB/64KiB; a Layer-C drop pads to 1KiB/4KiB/16KiB/64KiB; a Layer-D wrap pads to fixed 1KiB. When U22 Sealed-Sender hides sender_did, the envelope's *size* still discloses codepoint family. An adversary observing a 1KiB envelope can confidently classify it as Layer-D wrap (since no other codepoint family includes 1KiB as its default bucket).

**Why no lens caught it.** L6 designed U22 to hide sender. L6 also designed U24 to defeat size-correlation. The two amendments compose into "hides sender_did but not codepoint-family"; L6 did not stress the composition.

**Resolution (R-C7, §6).** Specify U24 buckets globally (not per-codepoint). Use *universal* size-class buckets (e.g., 1KiB, 4KiB, 16KiB, 64KiB, 256KiB, 1MiB) across ALL codepoints. Layer-D fixed-1KiB becomes Layer-D-pads-to-1KiB-bucket; Layer-A vault pads to a bucket ≥ vault-content-size; etc. Cost: small bandwidth overhead at Layer-D (since most Layer-D wraps are <1KiB, this is the existing case); zero overhead at Layer-A/C.

### Blind-Spot-2: Forkability semantic × U22 Sealed-Sender × U19 recipient-key-rotation = permissions-revocation trilemma

**The interaction.** Ben ratified 2026-05-27 the forkability semantic: "Atrium-as-forkable, member-leaves-keeps-past-content." This composes with:
- U19 recipient-key-rotation: after fork, Bob (left) holds pre-fork sks; pre-fork drops still decryptable by Bob.
- U22 Sealed-Sender: pre-fork drops in Sealed-Sender mode obscure sender Alice's identity from wire-observers; but post-decryption Bob sees Alice's sender_did inside ciphertext.

**The trilemma.** Choose any two; the third breaks:
1. **Forkability + retroactive-content-revocation:** requires un-decryptable pre-fork content; conflicts with forkability semantic.
2. **Forkability + Sealed-Sender:** requires inside-ciphertext sender_did to be revocable post-fork; conflicts with HPKE-mode-base FS-gap (Compromise #42).
3. **Sealed-Sender + retroactive-content-revocation:** requires CGKA-style epoch ratcheting; deferred per Compromise #35.

**Why no lens caught it.** L9 designed forkability (with prior-Ben-ratification) assuming default-codepoint sender_did-in-AAD. L6 designed Sealed-Sender assuming Signal-style ephemeral-message model (no forkability semantic). The intersection is not in any single lens's surface.

**Resolution.** This is an **honest-disclosure refinement, not a fixable contradiction.** Document in SECURITY-POSTURE.md as a composed-property limitation: "Benten's forkability semantic + Sealed-Sender pattern compose by accepting that post-decryption, sender identity is recoverable by recipient devices (even post-fork). Full revocation of past sender-identity attribution requires post-v1 CGKA-class machinery." Cross-link Compromise #42 + #35 + the L9 forkability-semantic ratification.

### Blind-Spot-3: U16 CodepointLifecycle × U29 cross-ecosystem-emit × U11 escape codepoint

**The interaction.** U16 lifecycle states are Live/Deprecated/Quarantined/Burned. When a Benten codepoint transitions from Live to Deprecated (e.g., ML-KEM-768 receives partial cryptanalysis in 2030), the cross-ecosystem-emit adapter (U29) for that codepoint translates to the matching ecosystem identifier (JWE `HPKE-11-KE`, age `mlkem768x25519`, etc.). But ecosystem registries don't have a lifecycle-state notion — JWE alg-names don't move from Live to Deprecated; they're just listed in IANA. The Benten lifecycle state is *not propagated* through the cross-ecosystem-emit boundary.

**Why no lens caught it.** L8 designed U16 for Benten-internal lifecycle. L7 designed U29 for cross-ecosystem-emit. The lifecycle-state-erasure at adapter boundary is a composition seam neither lens stressed.

**Resolution.** Document in U29 that cross-ecosystem-emit MUST:
1. At ENCODE time: if codepoint is Quarantined or Burned, refuse emit (writes refuse per U16).
2. At DECODE time (receiving from ecosystem): the Benten lifecycle state of the *received* codepoint must be re-checked locally; ecosystem identifier alone does not carry lifecycle metadata.

Low effort; document-only refinement.

### Blind-Spot-4: U30 DAG-CBOR × U37 golden vectors × U7 BE codepoint × multi-CBOR-implementation interop

**The interaction.** U30 mandates DAG-CBOR outer framing with Benten-private CBOR-tag 0xBE54. U37 golden vectors are CBOR-canonical-encoded per RFC 8949 §4.2.1. U7 mandates BE u16 codepoints. **But:** different CBOR implementations have varied canonicalization defaults (`cbor-x` for TS, `serde_cbor` for Rust, `dcbor` for IPLD-strict). Golden vectors that pass `serde_cbor` may fail `cbor-x` if e.g. small-int encoding differs.

**Why no lens caught it.** L7 chose DAG-CBOR for ecosystem-alignment. L4 designed golden vectors per U37 + tested in Rust. L7 noted "TypeScript port" as a future-additive benefit but did not stress that the v1-beta-frozen wire format MUST validate identically across multiple CBOR implementations to deliver that benefit.

**Resolution.** Specify in U30 + U37 that golden-vector validation in CI MUST include round-trip through at least 2 CBOR implementations (Rust `dcbor` + TS `cbor-x` once NAPI-RS layer ships). Pre-v1-beta-freeze, cross-implementation drift is auditable.

### Blind-Spot-5 (consolidator's MF5 extension): no formal-methods lens on composed model

**The gap consolidator noted.** No formal-methods lens evaluated HPKE-mode-base[X-Wing] IND-CCA2 proof tractability. I extend: **no formal-methods lens has evaluated the COMPOSED security property** (Inv-16 + Inv-17 + Inv-18). The individual amendments are tractable; the composed invariant chain may have an under-specified security game (e.g., what does "envelope-layer-unification" mean cryptographically, beyond a Rust-type-system claim?).

**Resolution.** For v1-beta this is acceptable per L5 (informal-rigorous prose). For v1-GM external audit, the composed-invariant security game should be formalized as ~5-page narrative in CRYPTO-PARAMETERS.md + a reduction sketch showing Inv-16 implies a multi-AAD-bound-IND-CCA2 property. Defer to v1-GM-prep wave; document as NAMED-DEFERRED row in V1-FROZEN-INTERFACE-DEFERRED.md.

---

## §6 Recommended amendments to add/refine for composability

### R-C1 — Specify Sealed-Sender multi-stanza cross-stanza binding via HPKE-mode-auth psk_id (NEW; closes C-1)

Refine U22 statement to include:
> Sealed-Sender multi-stanza composition uses HPKE-mode-auth's `psk_id` field for cross-stanza binding. Per-stanza AAD binds `(codepoint, CEK-AEAD-CID, audience_did, stanza_index, recipient_key_generation, delivery_token)` where `delivery_token = MAC(sender_signing_sk, stanza_index || envelope_blob_cid)`. Sender_did itself remains inside ciphertext per U22 default semantic. The Signal SSv2 delivery-token pattern (https://signal.org/blog/sealed-sender-multi-recipient/) is the direct precedent.

Severity: **LOAD-BEARING** for v1-beta (since U22 codepoint slot is locked at v1-beta per consolidator). Reservation cost zero; impl is post-v1-beta per U22 disposition.

### R-C2 — Specify U28 composed clock-skew rule (NEW; closes C-2)

Refine U28 statement to include:
> Composed replay-defense for DeviceLink + RemotePermission: enforced via outer UCAN nbf/exp on AuthorizationGrant (UCAN spec uses hours/days; matches naturally); AAD-bound `valid_until_epoch_hour` is defense-in-depth with `clock_skew_hours = 1` (admit 1-bucket lookback). For Vault + DropToRecipient (U5 EXCLUDED per design), U28's `sealed_at_epoch_hour` is informational only — never validated at decrypt-time, only logged for audit-trail.

Severity: **LOAD-BEARING** for v1-beta (clock-skew rule is wire-format-affecting in the sense that v1-beta readers MUST agree on the rule). Documentation-only change.

### R-C3 — Specify U25 per-recipient-unlinkability semantic bound (NEW; closes C-3)

Refine U25 statement to include:
> Per-recipient-unlinkability applies to wire-observability BEFORE decryption (envelope_blob_cid axis); does NOT promise post-decryption recipient-collusion-resistance (plaintext_cid axis remains shared per U18). Forkability semantic (Ben-ratified 2026-05-27) depends on shared plaintext_cid; this is intentional. Adversaries with multi-year aggregate-traffic-analysis capability + size-side-channel + timing-side-channel may correlate per-recipient copies via U24 bucket and U28 timing; full closure requires Sphinx-style onion-routing (post-v1).

Severity: **LOAD-BEARING** for v1-beta as honest-disclosure prose. Documentation-only change.

### R-C4 — Add `k_principal_generation: u32` field to `BindingContext::DeviceLink` (NEW; closes C-4 Scenario C)

Refine U20 statement to include:
> `k_principal_generation: u32` is bound into Layer-B per-Node AEAD AAD AND into `BindingContext::Vault` AND into `BindingContext::DeviceLink` (the Layer-D multi-device-key-wrap delivers K_principal[gen]; the wrap must self-identify the delivered generation regardless of Atrium-sync state of receiving device).

Severity: **LOAD-BEARING** at v1-beta interface-freeze (additive field to existing variant; cheap if locked now; expensive wire-break if discovered post-v1-beta). Reservation cost: ~5 LOC + golden-vector regen.

### R-C5 — Specify cross-ecosystem-emit behavior on escape codepoints (NEW; closes C-5 Scenario D)

Refine U29 statement to include:
> `cross_ecosystem_map(codepoint=0xFFFF)` returns explicit None-mapping; cross-ecosystem-emit adapters MUST return `Err(CrossEcosystemEmitError::EscapeCodepointNotEmittable)`. v1-beta readers MUST refuse codepoint 0xFFFF locally per U11; cross-ecosystem-emit at v1-beta is undefined for escape codepoints; documented at U29 mapping table.

Severity: **LOAD-BEARING** for v1-beta (cross-ecosystem-emit discipline is part of §3.5s mandate). Documentation + ~10 LOC adapter-guard.

### R-C6 — Add diagnostic-only error-disambiguation for U40 audit-deliverable (NEW; closes C-6 Scenario F audit-readiness concern)

Refine U40 (THREAT-MODEL.md) test corpus to include:
> Post-AEAD-tag-verify-failure diagnostic re-validation: re-run canonical_binding reconstruction with each AAD field individually substituted from candidate values (e.g., known-historical sender_dids, prior-hour epoch buckets) to determine which AAD field was tampered. Diagnostic ONLY; never a security gate (the structural defense is the tag-verify-failure itself).

Severity: **RECOMMENDED** for v1-beta audit-deliverable; impl cost ~1 wave-day.

### R-C7 — Specify U24 size-class buckets as universal (not per-codepoint) (NEW; closes Blind-Spot-1)

Refine U24 statement to:
> Size-class buckets are universal across ALL codepoints: 1KiB / 4KiB / 16KiB / 64KiB / 256KiB / 1MiB. Per-codepoint bucket choice MUST NOT differ (prevents codepoint-leak-via-bucket-size when composed with U22 Sealed-Sender). Layer-D 1KiB fixed becomes Layer-D-pads-to-1KiB-bucket (existing case); Layer-A vault pads to ≥vault-content-size bucket; etc.

Severity: **LOAD-BEARING** for v1-beta bucket-shape lock (per consolidator). ~5 LOC config change vs. consolidator's per-codepoint spec.

### R-C8 — Split Inv-18 into Inv-18a + Inv-18b + Inv-18c (REFINE; not new contradiction)

Refine §4 invariant set to split Inv-18 into three sibling invariants:
- **Inv-18a registry-discipline:** every minted Benten codepoint MUST appear in `docs/CRYPTO-CODEPOINTS.md` registry with: hex (BE per U7); variant name; primitive choice; AAD-binding spec; since-version; rationale; CodepointLifecycle state; cross-ecosystem-identifier row. Enforcement: cite-drift-detector scanner.
- **Inv-18b metadata-disclosure pairing:** every envelope shape placing identity-DIDs in plaintext AAD MUST have paired Sealed-Sender sibling codepoint slot reserved. Enforcement: `PlaintextSenderInAadPattern` scanner.
- **Inv-18c CodepointLifecycle typed-state:** every codepoint has lifecycle state Live/Deprecated/Quarantined/Burned; transitions enforced at runtime decode-dispatch. Enforcement: codepoint-registry-row + runtime check.

Split rationale: each Inv-18 sub-invariant has distinct enforcement-site granularity (cite-drift-detector / pattern-scanner / runtime). Consolidator's 3-way merge over-consolidated. Cost: ~0 LOC (docs-only refinement).

### R-C9 — Add a composition-properties test family to U37 golden vectors (NEW)

Refine U37 to include composition-property test family:
- multi-stanza Sealed-Sender per R-C1 (U17 ∘ U22)
- Layer-D wrap with k_principal_generation per R-C4 (U20 ∘ U10 ∘ DeviceLink)
- v2-envelope-at-v1-reader cross-ecosystem-emit refused per R-C5 (U11 ∘ U29)
- per-recipient-unlinkable share-plaintext-cid per R-C3 (U18 ∘ U25)
- escape-codepoint-lifecycle-check refused per Blind-Spot-3 (U11 ∘ U16)

Cost: ~5 additional vectors + ~1 wave-day for harness.

### R-C10 — Add formal-methods lens deferral row to V1-FROZEN-INTERFACE-DEFERRED.md (NEW; addresses Blind-Spot-5)

Mint Row D-FORMAL-1: "Composed-invariant security game (Inv-16 + Inv-17 + Inv-18a/b/c) formalization deferred to v1-GM-prep wave. v1-beta ships informal-rigorous prose per L5; v1-GM ships ~5-page narrative + reduction sketch."

Cost: ~0 at v1-beta (docs row); ~3-5 wave-days at v1-GM-prep.

### Summary of refinements

| Refinement | Closes | Severity | Cost |
|---|---|---|---|
| R-C1 Sealed-Sender ∘ multi-stanza | C-1 | LOAD-BEARING | docs + reservation |
| R-C2 U28 composed clock-skew rule | C-2 | LOAD-BEARING | docs |
| R-C3 U25 per-recipient-unlinkability bound | C-3 | LOAD-BEARING | docs |
| R-C4 k_principal_generation in DeviceLink | Scenario C | LOAD-BEARING | ~5 LOC + golden-vector regen |
| R-C5 cross-ecosystem-emit escape behavior | Scenario D | LOAD-BEARING | ~10 LOC |
| R-C6 diagnostic error-disambiguation | Scenario F | RECOMMENDED | ~1 wave-day |
| R-C7 universal U24 buckets | Blind-Spot-1 | LOAD-BEARING | ~5 LOC |
| R-C8 split Inv-18 into 18a/18b/18c | Inv-18 over-merge | LOAD-BEARING | docs |
| R-C9 composition-property test family | composability discipline | RECOMMENDED | ~5 vectors + 1 day |
| R-C10 formal-methods v1-GM defer row | Blind-Spot-5 | DEFERRED | 0 at v1-beta |

**Aggregate cost:** ~1-2 wave-days of refinement, no v1-beta scope expansion beyond what registry already commits.

---

## §7 Verdict on registry composability

**Composable AS-IS:** **NO.** Three contradictions (C-1, C-2, C-3) and four composition failures (Scenarios C, D, F, G) are reachable from the literal text of U1..U40 + C32..C44 + Inv-16/17/18.

**Composable WITH R-C1..R-C10 refinements:** **YES.** All seven composability blockers close via docs refinement + ~25 LOC + golden-vector regen + one additive field (DeviceLink.k_principal_generation). No re-architecting; no rejection of any lens's contribution.

**Recommendation to Ben.** Ratify the consolidated registry as the v1-beta-LOAD-BEARING subset AND apply R-C1..R-C10 refinements BEFORE F-full R0 plan-doc authoring. Cost: ~1-2 wave-days; benefit: avoids late-discovery composition rework at R5 (which per Phase-4-Meta-Core experience adds ~2-4 weeks per `feedback_addl_pipeline_full_observance`).

**On the consolidator's MEDIUM-HIGH on amendment-interaction.** The third reviewer's MEDIUM-HIGH rating on amendment-interaction was correct; this critique surfaces seven concrete interactions (C-1..C-7 + four scenarios) that justify the rating. The consolidator's response (consolidating but not stress-testing interactions) was appropriate to the consolidator's brief; this critique's brief was to do the stress-test the consolidator declined.

**Disposition:** **REFINE consolidated registry by adopting R-C1..R-C10 in a follow-up consolidation pass; ratify the REFINED registry at R0.**

---

## §8 Self-assessment + confidence

### What I did + how I worked

1. Tree-state pre-flight on worktree (clean @ 2172cb6d; branched from consolidated registry @ fbdfeb16).
2. Read consolidated registry in full (~939 lines; 3 passes — initial read for amendment inventory, second pass for invariant + Compromise mapping, third pass to identify cross-amendment seams).
3. Cross-checked critical cross-amendment seams against origin lens reviews via `git show` greps: L8 escape-codepoint mechanics; L9 multi-stanza + dual-CID + k_principal_generation; L6 Sealed-Sender + padding + epoch-bucket details; L7 DAG-CBOR + cross-ecosystem-emit; L3 Bernstein-Persichetti; L4 XChaCha20 + LE-vs-BE; L5 hybrid + metadata invariants.
4. Built composition model (§2) by walking encoder/decoder/validation paths with all 28 amendments applied; identified seams empirically.
5. Identified 3 contradictions (C-1, C-2, C-3) by composition-model walk; identified 4 composition failures (Scenarios C, D, F, G) by scenario-driven attack.
6. Identified 5 interaction blind spots not in any single lens's surface.
7. Synthesized 10 refinements (R-C1..R-C10) targeted at each composability blocker; verified each refinement closes its target without opening new contradictions.

### Confidence summary

| Section | Confidence | Rationale |
|---|---|---|
| §2 Composition model | **HIGH** on structure; **MED-HIGH** on Rust pseudocode exact-field-ordering | Pseudocode is illustrative; exact TLV tag-byte assignment is L8 §2.3 responsibility |
| §3 C-1 Sealed-Sender ∘ multi-stanza | **HIGH** | Direct field-conflict; both lens texts cited |
| §3 C-2 U28 ∘ U5 clock-skew | **HIGH** | Under-specification verifiable in registry text |
| §3 C-3 U18 ∘ U25 plaintext_cid | **HIGH** on contradiction; **MED-HIGH** on R-C3 resolution | Signal SSv2 adaptation needs more design at R0; R-C3 is direction not full spec |
| §4 Scenarios A-G | **HIGH** on identification; **MED-HIGH** on Scenario G novelty | Scenarios A-F verifiable from model walk; Scenario G is original cross-amendment composition |
| §5 Blind spots 1-5 | **HIGH** on Blind-Spot-1, -3, -4; **MED-HIGH** on Blind-Spot-2 (trilemma may overstate trilemma vs simpler trade-off) | Blind-Spot-2 forkability-trilemma framing is my synthesis; reasonable cryptographers may frame differently |
| §6 R-C1..R-C10 refinements | **HIGH** on R-C1, R-C2, R-C4, R-C5, R-C7, R-C8; **MED-HIGH** on R-C3, R-C6, R-C9, R-C10 | The HIGH set are direct-fix-for-direct-contradiction; the MED-HIGH set are direction + design call at R0 |
| §7 Verdict NOT-composable-as-is | **HIGH** | At least one of C-1, C-2, C-3 is unambiguous; the verdict holds even if 2 of 3 dissolve under more charitable reading |

### What I could be wrong about

1. **C-3 (per-recipient-unlinkable + dual-CID) framing.** I read U25 invariant clause as "wire-observable" axis; an alternative reading could be "U25 applies only to envelope-blob-cid by construction, no contradiction." If Ben + R0 author share my reading, R-C3 is needed; if the alternative reading holds, R-C3 reduces to docs-clarification.
2. **C-2 (U28 ∘ U5 clock-skew) severity.** Consolidator's note "outer UCAN nonce-cache provides freshness" may be implicit common knowledge among reviewers I didn't access. If UCAN nonce-cache is documented elsewhere as the load-bearing freshness gate, C-2 reduces to "specify cross-reference."
3. **R-C4 (k_principal_generation in DeviceLink) bypass.** Atrium-sync may be assumed-eventually-consistent strongly enough that the new device CAN wait for rotation-log sync before labeling K_principal generation. If so, R-C4 is defense-in-depth not load-bearing.
4. **Blind-Spot-2 trilemma framing.** May overstate as trilemma what is actually a bilemma + honest-disclosure. The composed-property limitation is real; the "trilemma" word choice may be too dramatic.
5. **U30 DAG-CBOR cross-implementation interop** (Blind-Spot-4) assumes TS port arrives at v1-beta; if TS port is post-v1-beta-only, the cross-implementation-validation refinement is post-v1-beta concern not v1-beta-blocker.

### Lower-confidence areas (honest disclosure)

- I did NOT independently run the §2 pseudocode through a Rust compiler — it's illustrative-pseudocode for composition reasoning, not buildable. Exact TLV tag-byte numbering may need L8 §2.3 cross-check at R0.
- I did NOT verify the Signal SSv2 delivery-token MAC construction I cited in R-C1 against the actual Signal blog post (https://signal.org/blog/sealed-sender-multi-recipient/) — I cited it from memory of the L6 review's citation. R0 author should ground-truth-verify before adopting R-C1's specific delivery_token formula.
- I did NOT enumerate every (amendment-pair × amendment-pair) for higher-order composition contradictions. 28 amendments × 28 amendments = 784 pairs; I focused on the ~25 pairs the §2 composition model surfaced as load-bearing seams. Higher-order (3+-amendment) interactions may exist beyond Blind-Spot-2.
- My "composability" lens is necessarily synthetic; it draws on my prior knowledge of HPKE-multi-stanza interop hazards (e.g., age vs Saltpack precedent cited by L9), Signal SSv2 design, and IPLD content-addressing semantics. I have not read the prior P2P-architect review @ 8cfb079c directly; I read its conclusion via the consolidator's §5 Q3 summary.

### What this critique does NOT cover

- Per brief: I am NOT proposing new amendments to the substrate (U1..U40 are taken as adopted; R-C1..R-C10 are *refinements* of existing amendments).
- I did NOT re-do construction soundness (L1 + L2 + L3 authoritative).
- I did NOT re-do impl-engineering (L4 authoritative).
- I did NOT independently audit each Compromise mint narrative (L5 authoritative).
- I did NOT re-do cross-ecosystem-interop analysis (L7 authoritative; I only extended at the U29 ∘ U16 seam per Blind-Spot-3).
- I did NOT enumerate higher-order (4+-amendment) interaction blind spots.

---

## §9 Citations

### §9.1 Lens reviews + consolidated registry (frozen SHAs)

- L1: `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f`
- L2: `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b`
- L3: `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3`
- L4: `phase-4-meta-core/option-f-plus-lens-l4-impl-engineering @ 4d4aae5f`
- L5: `phase-4-meta-core/option-f-plus-lens-l5-threat-model-audit-readiness @ 3f27f8e0`
- L6: `phase-4-meta-core/option-f-plus-lens-l6-privacy-metadata-leak @ 986e50bb`
- L7: `phase-4-meta-core/option-f-plus-lens-l7-cross-ecosystem-interop @ 208f98bb`
- L8: `phase-4-meta-core/option-f-plus-lens-l8-wire-format-stability @ d8d3c41c`
- L9: `phase-4-meta-core/option-f-plus-lens-l9-atrium-integration @ 1670aa03`
- Consolidator: `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16`
- e2r scope: `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae`

### §9.2 Benten internal references

- `docs/INVARIANT-COVERAGE.md` (Inv-15 existing; Inv-16/17/18a/18b/18c pending)
- `docs/SECURITY-POSTURE.md` (Compromise #31 existing; #32..#44 pending; #31 extension pending)
- `docs/V1-FROZEN-INTERFACE.md` + `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (Row D-SS-1, D-PAD-1, D-COVER-1, D-FORMAL-1)
- CLAUDE.md baked-in #5 (crypto-agility codepoint-dispatch); #15 (v1-beta gates); #17 (deployment-shapes); #18 (authority-isolation)
- `.addl/dispatch-conventions.md` §3.5s (cross-ecosystem-identifier-as-content)
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` (load-bearing for this critique's scope)
- `feedback_review_finding_ground_truth_verify` (DISAGREE first-class)
- `feedback_pim_cross_language_rule_mirror` §3.5g
- `feedback_addl_pipeline_full_observance` (R0→R6 discipline)

### §9.3 External references composing into composability findings

- RFC 9180 HPKE (mode-base, mode-auth, KEM API; mode-auth `psk_id` field per §5.1.4 — load-bearing for R-C1)
- RFC 8949 CBOR + RFC 8949 §4.2.1 deterministic encoding (load-bearing for U30 ∘ U37 ∘ Blind-Spot-4)
- RFC 9000 §16 IETF varint encoding pattern (cited by L8/Am11 escape-codepoint)
- FIPS 203 ML-KEM
- Bernstein-Persichetti, "One Time is Enough: Chosen-Ciphertext Side-Channel Attack on ML-KEM Cryptosystems," IACR ePrint 2024/2051
- Signal Sealed Sender V2 (https://signal.org/blog/sealed-sender-multi-recipient/) — load-bearing for R-C1 delivery-token pattern
- age multi-recipient encoding discussion #463 (cited by L6)
- C2SP age.md `mlkem768x25519` recipient stanza (cited by L7)
- libcrux-ml-kem `check-secret-independence` feature (cited by L4 + L5)
- draft-connolly-cfrg-xwing-kem-10
- BSI TR-02102-1 (long-term confidentiality classes; cited by L5 + Compromise #44)
- NIST SP 800-227 §4.4 PQ/T hybrid combiners (cited by L5)
- Compromise #31 cascade pattern (consolidator MF4; extended by this critique to include Scenario G)

---

**End of composability critique.**
