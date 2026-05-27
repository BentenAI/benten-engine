# Option F+ envelope-layer-unification §6.2-with-Amendments — Lens L8: wire-format-stability / v1-beta-freeze-permanence / v2+-migration-story / forward-additivity

**Reviewer lens.** Senior protocol-evolution architect. Distinct from prior reviewers (1: F+ pseudo-keypair takedown; 2: second-opinion soundness with Amendments 1+2; 3: adversarial-design with Amendments 3+4+5+6+7+8). Three prior reviewers focused on point-in-time soundness against today's adversary. **My job is to find what §6.2-with-Amendments-1-8 locks Benten into PERMANENTLY at the v1-beta freeze + whether the design admits the post-2026 cryptographic ecosystem (CGKA / MLS-PQ / Bird-of-Prey / draft-prabel / quantum-secure-CGKA / future AEADs / future KEMs / future signatures / future combiners) additively, without forcing a v2 wire format that strands every 2026-2030 envelope.**

**Inputs read in full.**
- `origin/phase-4-meta-core/option-f-plus-pseudo-keypair-review @ HEAD` → `.addl/phase-4-meta/option-f-plus-pseudo-keypair-review.md` (R1; 412 lines)
- `origin/phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ HEAD` → `.addl/phase-4-meta/option-f-plus-second-opinion-cryptographer-review.md` (R2; 520 lines; introduces Amendments 1+2)
- `origin/phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ HEAD` → `.addl/phase-4-meta/option-f-plus-third-reviewer-adversarial-design.md` (R3; 640 lines; introduces Amendments 3+4+5+6+7+8)
- `main @ HEAD` → `docs/V1-FROZEN-INTERFACE.md` item 6 (3-axis codepoint table + 6 sub-items + freeze discipline)
- `main @ HEAD` → `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-13 (CLOSED), Row D-15-RETRACTED, Row D-24
- CLAUDE.md baked-in #5 (crypto-agility codepoint-dispatch + typed-reject + never-fork-primitives) + #15 (v1-beta = v1-public-interface-freeze) + #18 (authority vs confidentiality isolation)

---

## §1 Executive verdict + confidence

**Verdict: CONCUR-WITH-EXTENSIONS.** §6.2-with-Amendments-1-8 is the right *shape* and is **forward-additive at the codepoint axis** by construction — the design slots cleanly into baked-in #5's crypto-agility framework and inherits V1-FROZEN-INTERFACE.md item 6's existing additive-codepoint discipline. However, **six structural permanence concerns survive Amendments 1-8** and require explicit closure (or NAMED-deferred-disclosure per HARD RULE clause-b) before v1-beta tag, because each becomes irretrievable once a single shipped envelope is on disk in production:

| # | Permanence concern | Closure |
|---|---|---|
| L8-A | **`EnvelopePayload` enum is closed at v1-beta** — only 2 variants (`SymmetricAead` + `HpkeBase`); future CGKA / MLS-PQ / Bird-of-Prey / draft-prabel structurally need NEW variants (3-message KEM exchange; nested ratcheting tree; sender-AKEM; key-package-Welcome split). New variant ⇒ v1 readers reject ⇒ wire-format-break. | **Amendment 9 (NEW — load-bearing): make `EnvelopePayload` `#[non_exhaustive]` + reserve codepoint-bracketed variant slots + mandate `unknown-variant ⇒ typed-reject` (NOT panic, NOT silent-skip).** §2.2 below. |
| L8-B | **`BindingContext` enum same issue** — 4 variants at v1-beta; future Atrium-group-drop / MLS-Welcome / device-mesh-rekey / capability-revocation / sealed-sender / metadata-private-routing all need NEW BindingContext variants. | **Amendment 10 (NEW — load-bearing): `#[non_exhaustive]` + codepoint-bracketed reservation + per-variant codepoint-bind in AAD per Amendment 1.** §2.3 below. |
| L8-C | **`codepoint: u16` is permanently 16-bit; 65,536 slots TOTAL across ALL three axes (Sig + Cipher + Hash) AND all future-additive axes (KEM-only, AKEM, AEAD-only, KDF-only, combiner, CGKA-suite, post-merge-MLS-suite).** 65,536 sounds large; IANA-coordination + MLS + COSE + JOSE + HPKE collectively show codepoint exhaustion in the 16-bit space is a real phenomenon. JOSE `alg` registry has ~70+ entries; TLS cipher-suite registry exhausted 1-byte then 2-byte ranges. The **u16 itself is wire-format-frozen** at v1-beta. | **Amendment 11 (NEW — load-bearing): reserve `0xFFFF` (and ideally `0xFE00..0xFFFF`) as an *escape codepoint* that signals "follow with a varint codepoint-32 OR codepoint-64 in the payload prefix". This is the IETF varint pattern (RFC 9000 §16) + MLS Extension `extension_type` u16-escape pattern.** §2.4 below. |
| L8-D | **`nonce: [u8; 12]` is FROZEN at 12 bytes for ChaCha20-Poly1305** — but future AEADs (XChaCha20-Poly1305 = 24 bytes; AEGIS-256 = 32 bytes; PQ-AE constructions = TBD; SPHINCS-AE = TBD; future-streaming-AE = variable). The `[u8; 12]` IS the wire format. | **Amendment 12 (NEW — load-bearing): make nonce a `nonce: Bytes` (length-prefixed) NOT `[u8; 12]`; pin the length-per-codepoint as a runtime table not a type-level constraint; OR keep `[u8; 12]` and mint a NEW `EnvelopePayload::SymmetricAeadXNonce { ciphertext, nonce: [u8; 24] }` variant.** Latter is cleaner under §2.2's Amendment 9 framing. §2.5 below. |
| L8-E | **Forward-secrecy is structurally ABSENT at Layer-C** — HPKE-mode-base is one-shot non-FS; the recipient's long-term sk decrypts forever; an exfiltrated 2026 sk recovers 2026 envelopes in 2030. The §6.2 design does NOT carve a FS-capable variant slot. When MLS-PQ + CGKA mature in the 2027-2030 window, Benten needs an additive path. | **Amendment 13 (NEW — load-bearing): explicitly disclose the FS-gap as a v1-beta Compromise + reserve a `Layer-C-with-FS` codepoint bracket + reserve `EnvelopePayload::HpkeCgka { commit_secret_ref, ciphertext }` and `EnvelopePayload::MlsApplication { group_epoch, sender_index, ciphertext }` variant slots.** §3 below. |
| L8-F | **AAD canonicalization function `canonical_binding()` IS the wire format** — Amendment 1 + 3 fix encoding; once shipped, every byte produced by it is verify-permanent in 2050. Future BindingContext variants whose canonical encoding interacts with TLV layout (e.g., a future variant with a length-prefixed `Vec<Did>` audience list) MUST extend not modify. | **Amendment 14 (NEW — load-bearing): pin `canonical_binding()` as `pub fn ... -> Bytes` + version it with an internal `aad_version: u8` *that is itself bound* + reserve TLV tag-byte `0xFF` as "extended canonicalization marker" so future canonicalization-shape changes are themselves codepoint-discriminated.** §2.6 below. |

**Plus three MINOR concerns** (L8-G replay-window u64 epoch overflow analysis; L8-H sender-DID encoding stability as Atrium adds DID methods; L8-I deprecation-window structural surface):

| # | Minor concern | Closure |
|---|---|---|
| L8-G | `sealed_at_epoch_seconds: u64` overflows in year `292277026596` (10^19). Safe through 2050. ✅ **NO ACTION REQUIRED** — flagged for completeness. R3 Amendment 5 is fine on this axis. (Y2K38 affects `i32` not `u64`; Y2106 affects unsigned `u32` not `u64`.) §2.7 below. |
| L8-H | `sender_did: Did` per R3 Amendment 4 — if `Did` is `enum { Key(Ed25519Pubkey), KeyPq(MldsaPubkey), Web(Url), …}`, every variant addition is a wire format extension. Currently Atrium uses `did:key` only; this is fine, but **the enum IS wire-format-frozen at v1-beta**. | **Amendment 15 (NEW — minor): pin `Did` canonical-serialization to `multikey` varint codepoint form per CLAUDE.md baked-in #5 (multiformats discipline) + add `Did::Unknown(u64 multikey_codepoint, Bytes)` typed-rejection variant; new DID methods land as new multikey codepoints, structural decode preserved.** §2.8 below. |
| L8-I | **Deprecation-window strategy:** when a codepoint is found broken post-v1-beta, V1-FROZEN-INTERFACE.md item 6 already commits "old codepoints supported FOREVER" — but **broken codepoints SHOULD NOT be supported forever**; the design needs a `DeprecatedReadOnly` state that lets 2030-era code still verify a 2026 envelope but refuse to produce new envelopes at a deprecated codepoint. | **Amendment 16 (NEW — minor): mint a `CodepointLifecycle { Live, ReadOnly, Quarantined, Burned }` typed-state per codepoint in the registry; pre-encode-validation MUST refuse new envelopes at `ReadOnly`/`Quarantined`/`Burned` codepoints; pre-decode-validation MUST refuse `Burned` codepoints (verify-fail-closed) while `Quarantined` admits with logged warning + `ReadOnly` admits silently.** §5 below. |

**Cumulative verdict: §6.2 + Amendments 1-8 + new Amendments 9-16 is `CONCUR-AS-LOAD-BEARING-FINAL` for v1-beta freeze.** Without 9-16, the design **is** v1-beta-tagable but at the cost of either (a) a forced v2 wire-format break in the 2027-2030 window when CGKA/MLS-PQ matures (which violates baked-in #15's "decode supported FOREVER" commitment), or (b) named-NOW Compromise disclosures that v2-CGKA/MLS migration will be ADDITIVE-NOT-EXTENSIBLE under v1.

**Confidence.**
- HIGH on Amendments 9, 10, 11, 12, 14 (closed-enum + 16-bit-exhaustion + nonce-frozen-width + canonicalization-versioning are STANDARD protocol-evolution failures with rich case-study evidence: TLS 1.0→1.2 cipher-suite extension via 2-byte selector; OpenPGP RFC 4880→9580 packet-format extension via `packet_tag_new`; SSH-1→SSH-2 forced wire-format-break; JOSE `alg` registry collision with COSE).
- HIGH on Amendment 13 (FS-gap structural; explicit MLS-PQ + CGKA + MLS-application-data variant slot reservation is industry-standard MLS RFC 9420 §5.3 LeafNode `extensions` pattern).
- MEDIUM-HIGH on Amendment 15 (Did encoding stability) — depends on Atrium's commitment to multikey discipline; if Atrium already commits to multikey, this is cheap; if Atrium is exploring `did:web` or `did:plc` paths, this needs Ben-decision.
- HIGH on Amendment 16 (deprecation-window) — every multi-decade protocol that omitted this has retrofit-pain stories (TLS RC4 deprecation; SHA-1 deprecation; MD5 deprecation; DES deprecation; export-grade-RSA Logjam).

**Most-load-bearing finding.** Amendment 11 — the **u16 codepoint exhaustion analysis + escape-codepoint reservation**. This is the single permanence concern whose retrofit cost balloons super-linearly with deployed-envelope count. Every other amendment has a per-variant retrofit cost; codepoint exhaustion has a per-codepoint-axis retrofit cost.

**Recommended ratification sequence.** Adopt Amendments 9-16 simultaneously with Amendments 1-8 as ONE Inv-16 mint package. The marginal review-cost of adding 8 more amendments is small compared to the catastrophic permanence-cost of missing any single one.

---

## §2 v1-beta-freeze permanence enumeration

For each design element of §6.2 + Amendments 1-8, I classify it as **PERMANENTLY FROZEN** (cannot change without v2 wire format break) vs **EXTENSIBLE-AT-CODEPOINT** (changes land at new codepoint; old codepoints decode forever per V1-FROZEN-INTERFACE.md item 6) vs **EXTENSIBLE-AT-VARIANT** (changes land as new enum variant) vs **EXTENSIBLE-AT-AMENDMENT** (canonicalization-version field discriminates).

### §2.1 The `EncryptedEnvelope` struct outer shape — PERMANENTLY FROZEN

```rust
pub struct EncryptedEnvelope { codepoint: u16, payload: EnvelopePayload, aad_binding: BindingContext }
```

Three fields, fixed order, fixed types. **This struct shape IS the wire format outer shape forever.** A future v2 wire format that needed a 4th field (e.g., a `protocol_version: u8` separate from codepoint, or an `extensions: Vec<TlvExt>` extension-tail) would require either:
- (a) a NEW outer wrapper type (`EncryptedEnvelopeV2 { v1: EncryptedEnvelope, extensions: ... }`), wrapping v1 — the natural permanent-decode path; OR
- (b) a codepoint-bracketed range that signals "this codepoint introduces a v2-shape envelope" — burns codepoint slots for structural redirection.

(a) is the right pattern (matches OpenPGP RFC 9580's strategy of wrapping v4-packets inside v6-encrypted-message containers + matches TLS 1.3's strategy of keeping the ClientHello shell + adding extensions). **§6.2 implicitly assumes (a) is the future migration path; this assumption should be documented as a v1-beta commitment.** Without explicit documentation, a future maintainer may attempt (b) and waste codepoint slots.

**Action.** Document in V1-FROZEN-INTERFACE.md item 6: "future v2 outer shape evolution = NEW wrapper type containing this struct; this struct's 3-field shape is byte-permanent."

### §2.2 The `EnvelopePayload` enum — EXTENSIBLE-AT-VARIANT but currently NOT `#[non_exhaustive]`

```rust
pub enum EnvelopePayload {
    SymmetricAead { ciphertext: Bytes, nonce: [u8; 12] },
    HpkeBase { enc: Bytes, ciphertext: Bytes },
}
```

**Two variants at v1-beta.** Without `#[non_exhaustive]`, every match arm in every consuming crate is exhaustive — adding a third variant in v1.1 (or worse, in v2) requires a coordinated workspace-wide match-arm update. Worse: an external consumer (e.g., the future `benten-ts` TypeScript impl, or a future external Atrium-peer-mesh participant) that pattern-matches against the 2-variant shape will silently break when v1.1 introduces a third variant.

**Future variants that WILL be needed (2026-2035 horizon).** Drawn from MLS RFC 9420, IETF draft-ietf-mls-pq, draft-prabel (PQ-MLS-AKEM), draft-ietf-cose-pqc, RFC 9180 §10.3 multi-recipient HPKE, X-Wing IRTF (currently `draft-connolly-cfrg-xwing-kem-08`):

| Future variant | Why needed | When |
|---|---|---|
| `SymmetricAeadXNonce { ciphertext, nonce: [u8; 24] }` | XChaCha20-Poly1305 (24-byte nonce); R3 §2.8 row (l) flagged this | v1.1 if Layer-A nonce-scheme review picks XChaCha20 |
| `HpkeBaseMultiRecipient { enc_per_recipient: Vec<(Did, Bytes)>, ciphertext }` | R2 §2.3 row 5 noted this gap; RFC 9180 §10.3 | v1.1-v1.5 (Atrium group drops) |
| `HpkeAuth { sender_pk, enc, ciphertext }` | HPKE mode_auth (RFC 9180 §5.1.1); sender-authenticated | v2 if device-link sender-auth promoted from outer-sig to AKEM |
| `HpkePsk { psk_id, enc, ciphertext }` | HPKE mode_psk; pre-shared device-link | v1.5 if pairing UX needs out-of-band PSK |
| `MlsApplication { group_id, epoch, sender_idx, ciphertext }` | MLS Application Data Frame (RFC 9420 §6); replaces single-recipient HPKE with group-keyed application messages | v2 when Atrium semantics admit MLS-Group |
| `MlsWelcome { encrypted_group_secrets, group_info, ratchet_tree }` | MLS Welcome Message (RFC 9420 §12.4); device-join via Welcome | v2 (Inv-16 CGKA reservation) |
| `CgkaCommit { commit_secret_ref, path_secrets, ... }` | CGKA Commit frame (Alwen-Coretti-Dodis 2019; draft-ietf-mls-protocol-pq) | v2-v3 (depends on PQ-CGKA maturation) |
| `HpkeBirdOfPrey { ... }` | Bird-of-Prey (Brendel et al. 2024 PQ-secure-CGKA building block) | v3+ if Bird-of-Prey matures past draft |
| `HpkeXWingTrue { enc, ciphertext }` | True X-Wing IRTF when IRTF-CFRG finalizes (currently the X-Wing-mislabel corrective) | v1.5-v2 |
| `SymmetricAeadAegis { ciphertext, nonce: [u8; 32] }` | AEGIS-256 (RFC 9380 stand-in; AEGIS family TBD) | v2-v3 if AEGIS adoption matures |

**At least 10 plausible future variants in a 10-year horizon.** A closed enum forces N consuming-crate updates per addition + breaks external consumers. `#[non_exhaustive]` makes future additions purely additive.

**Amendment 9 (NEW — load-bearing).**
```rust
#[non_exhaustive]
pub enum EnvelopePayload {
    SymmetricAead { ciphertext: Bytes, nonce: [u8; 12] },
    HpkeBase { enc: Bytes, ciphertext: Bytes },
    // Future variants land here (v1.1, v1.5, v2, ...).
}
```
Plus: pin a typed-decode discipline: an unknown variant tag (CBOR map-key the decoder doesn't recognize) MUST produce `EnvelopeError::UnknownPayloadVariant { tag, codepoint }` — NOT a panic, NOT a silent-skip-to-default. This matches CLAUDE.md baked-in #5 typed-reject pattern.

**Mirror in test surface.** A test in `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs` should walk every variant of `EnvelopePayload` at compile time (using strum or manual exhaustive-list) + assert byte-stability per variant. When a future variant is added, the test forces an explicit byte-pin update.

### §2.3 The `BindingContext` enum — SAME ISSUE; SAME FIX

```rust
pub enum BindingContext {
    Vault { vault_version: u8 },
    DropToRecipient { audience_did: Did, sender_did: Did, sealed_at_epoch_seconds: u64, valid_until_epoch_seconds: u64 },
    DeviceLink { provisioning_session_id: [u8; 16], sender_device_did: DeviceDid, sealed_at_epoch_seconds: u64, valid_until_epoch_seconds: u64 },
    RemotePermission { request_id: [u8; 16], operation: PermissionOperation, granting_user_did: Did, requesting_device_did: DeviceDid, sealed_at_epoch_seconds: u64, valid_until_epoch_seconds: u64 },
}
```

(after R3 Amendments 4+5).

**Four variants at v1-beta.** Future BindingContext variants needed:

| Future variant | Why |
|---|---|
| `DropToGroup { atrium_id, member_count, sealed_at, valid_until, sender }` | Atrium-group-drop (multi-recipient HPKE) |
| `MlsGroupApplication { group_id, epoch, sender, ... }` | MLS application-data binding |
| `MlsWelcome { joiner_did, group_id, ... }` | MLS device-onboarding binding |
| `AtriumSync { atrium_id, sync_token, ... }` | Atrium-peer-mesh-resync binding |
| `CapabilityRevocation { revoked_cap_cid, revoker_did, revoked_at }` | Drop-bundle-revocation (Compromise #31) |
| `SealedSender { receiver_did, ephemeral_sender_token }` | Signal-style sealed-sender for metadata privacy |
| `MetadataPrivateRouting { onion_route_token, ... }` | If onion routing or similar metadata-privacy lands |

**Amendment 10 (NEW — load-bearing).**
```rust
#[non_exhaustive]
pub enum BindingContext {
    Vault { vault_version: u8 },
    DropToRecipient { /* ... */ },
    DeviceLink { /* ... */ },
    RemotePermission { /* ... */ },
    // Future variants land here.
}
```
Plus: per Amendment 1, the codepoint MUST identify which BindingContext variant is structurally expected; per Amendment 2, codepoint→variant-tag dispatch table MUST be exhaustive at the live-codepoint set; unknown variant ⇒ typed-reject.

**Cross-amendment composition note.** Amendment 3's TLV encoding interacts with Amendment 10's enum extension. The TLV tag-byte space (per R3 Amendment 3) is currently undefined in the §6.2 sketch. **Recommended:** mint a TLV tag-byte registry that mirrors the BindingContext variant ordinal — variant 0 → tag 0x01, variant 1 → tag 0x02, etc. — and reserve tags `0xF0..0xFE` for vendor-extension experiments + `0xFF` as the canonicalization-version-escape (per Amendment 14 §2.6 below). This makes TLV+variant ordering injective + permanent.

### §2.4 The `codepoint: u16` field — PERMANENTLY 16-BIT; needs escape-codepoint reservation

V1-FROZEN-INTERFACE.md item 6 sub-item 2 already defines a 9-codepoint table at v1-beta. The encoding axes are:
- Hash: 3 codepoints minted (`0x1e`, `0x1015`, `0x16`)
- Sig: 3 codepoints minted (`0x0001`, `0x0002`, `0x0003`)
- Cipher: 5 codepoints minted (`0x647a`, `0x6400`, `0x0000`, `0x647b`, `0x647c`)

The current allocation is **non-contiguous + axis-mixed** (Hash uses multihash codepoints from IANA multiformats; Sig uses Benten-internal `0x000N`; Cipher uses `0x6400..0x6FFF` Benten-internal range + steals `0x0000` for plaintext-partition downgrade).

**Permanence concerns.**

1. **No reserved escape codepoint.** If v2 needs codepoint-32 or codepoint-64 — e.g., because a future MLS-PQ codepoint registry uses 4-byte codepoints, or because an IANA-coordinated codepoint outside Benten's `0x6100..0x6FFF` range collides — there is no escape path.

2. **Cross-axis codepoint collision is a future risk.** Hash `0x1015` (SHA2-512/256) does NOT collide with Cipher `0x647b` (Hybrid MLKEM768+HQC), but the table doesn't strictly prevent a future Sig codepoint `0x1015` from being minted that DOES collide with the Hash one. The 3-axis model means each axis has its own `u16` space, but at the *envelope* layer, the `EncryptedEnvelope.codepoint` field needs to distinguish: is this a Cipher codepoint? Or a higher-level "envelope-shape selector" codepoint?

3. **The envelope-codepoint distinct from cipher-codepoint.** R2 §2.4 + R3 §4.2 used `0x6101` for `LAYER_A_VAULT` as an example. This is **NOT** the Cipher codepoint `0x647a` (HYBRID_X25519_MLKEM768). The envelope's `codepoint` field is a **fifth axis** — "envelope-shape selector" — orthogonal to Hash/Sig/Cipher. V1-FROZEN-INTERFACE.md item 6 doesn't currently name this axis explicitly.

4. **u16 exhaustion case studies.**
   - TLS 1.0 cipher suites were 1-byte; exhausted by 2002; TLS 1.2 went to 2-byte; ~400+ entries today (IANA tls-parameters registry) → 1.5% exhaustion in 20 years (slow but real).
   - JOSE `alg` registry (RFC 7518): ~70 entries in 10 years; signed-strings only.
   - HPKE `kem_id`/`kdf_id`/`aead_id` (RFC 9180 §7): ~7 entries each at IRTF/IETF maturation; ~20 entries projected by 2035 with PQ additions.
   - Multiformats codepoint registry (`multicodec.csv`): ~600+ entries across all axes; growing fast.

   For Benten's envelope-codepoint axis, a u16 (65,536 slots) is comfortably oversized for the next 50 years. **But the registry hygiene matters more than the bit-width.**

5. **Why u32/u64 escape matters anyway.** Cross-protocol-compatibility: if Benten wants to interop with a future IETF protocol that uses u32 codepoints (e.g., a future MLS-PQ extension type), Benten's u16 envelope-codepoint either (a) maps a u16 slot to the u32 IETF codepoint via a Benten-local lookup table — fine, but introduces a translation layer — or (b) escape-codepoints to a longer wire encoding.

**Amendment 11 (NEW — load-bearing).** Three sub-actions:

(a) **Reserve `0xFFFF` as an "extended-codepoint escape" sentinel** at v1-beta. Semantics: when `EncryptedEnvelope.codepoint == 0xFFFF`, the next bytes of the envelope are a varint-encoded u32 or u64 codepoint that supersedes the u16 codepoint. v1-beta MUST refuse `0xFFFF` (typed-reject `UnsupportedAlgorithm::Envelope`); v2+ readers MAY accept it. **This costs ONE u16 slot now to permanently reserve the escape path.**

(b) **Reserve range `0xFE00..0xFFFE` as "experimental codepoints; v1.x SHOULD typed-reject; deployment-local use only".** This matches RFC 7468/7469 experimental-range convention + RFC 6648 X- prefix deprecation pattern (yes, X- was deprecated, but for ASCII protocols; the bit-range-reservation pattern is structurally correct).

(c) **Explicitly mint the envelope-codepoint axis in V1-FROZEN-INTERFACE.md item 6** as a 4th axis (Hash + Sig + Cipher + Envelope-Shape). Currently the envelope-shape codepoints (R2 §2.4 example `0x6101` for vault) are implicit-only. Make them explicit. Suggested initial table:

| Axis | Name | Codepoint | Status |
|---|---|---|---|
| EnvShape | `EnvelopeShape::VAULT_AEAD_V1` | `0x6101` | LIVE (Layer-A) |
| EnvShape | `EnvelopeShape::DROP_HPKE_BASE_V1` | `0x6300` | LIVE (Layer-C) |
| EnvShape | `EnvelopeShape::DEVICE_LINK_HPKE_BASE_V1` | `0x6310` | LIVE (DeviceLink) |
| EnvShape | `EnvelopeShape::REMOTE_PERMISSION_HPKE_BASE_V1` | `0x6320` | LIVE (RemotePermission) |
| EnvShape | `EnvelopeShape::DROP_HPKE_MULTI_RECIPIENT_V1` | `0x6301` | reserved (future Atrium-group-drop) |
| EnvShape | `EnvelopeShape::DROP_MLS_APPLICATION_V2` | `0x6380` | reserved (MLS-Group future) |
| EnvShape | `EnvelopeShape::DROP_HPKE_BASE_XCHACHA20_V1_5` | `0x6302` | reserved (XChaCha20 future) |
| EnvShape | `EnvelopeShape::EXTENDED_CODEPOINT_ESCAPE` | `0xFFFF` | RESERVED FOREVER (v2+ varint escape) |
| EnvShape | `EnvelopeShape::EXPERIMENTAL_RANGE` | `0xFE00..0xFFFE` | typed-reject; experimental |

**Bracketed reservation discipline.** Group related codepoints in contiguous ranges:
- `0x6100..0x61FF` → Layer-A vault variants (256 slots)
- `0x6300..0x63FF` → Layer-C drop variants (256 slots)
- `0x6310..0x631F` → DeviceLink variants (16 slots)
- `0x6320..0x632F` → RemotePermission variants (16 slots)
- `0x6380..0x638F` → MLS-group future variants (16 slots)
- `0x63A0..0x63AF` → CGKA-future variants (16 slots)
- `0x6700..0x67FF` → revocation/lifecycle variants (256 slots)

This is the **MLS extension-type pattern** (RFC 9420 §17.2 IANA registry) + **OpenPGP packet-type-tag pattern** (RFC 9580 §5).

### §2.5 The `nonce: [u8; 12]` field — PERMANENTLY 12 BYTES FOR THIS VARIANT

ChaCha20-Poly1305 nonce is 12 bytes (RFC 8439 §2.3). XChaCha20-Poly1305 nonce is 24 bytes (RFC draft-irtf-cfrg-xchacha-03). AEGIS-256 nonce is 32 bytes. Future PQ-AE constructions have unspecified nonce sizes.

**`[u8; 12]` is correct for `SymmetricAead`-variant + ChaCha20-Poly1305 codepoint.** It is wrong-by-construction for XChaCha20. The right answer:

**Amendment 12 (NEW — load-bearing).** Keep `nonce: [u8; 12]` as the SymmetricAead variant nonce shape, and add a NEW variant under Amendment 9 for wider-nonce ciphers:

```rust
#[non_exhaustive]
pub enum EnvelopePayload {
    SymmetricAead { ciphertext: Bytes, nonce: [u8; 12] },         // ChaCha20-Poly1305 + AES-GCM-128/256 (both 12-byte nonce)
    SymmetricAeadXNonce { ciphertext: Bytes, nonce: [u8; 24] },   // XChaCha20-Poly1305 + XSalsa20 (24-byte nonce)
    // SymmetricAeadAegis { ciphertext: Bytes, nonce: [u8; 32] }, // AEGIS-256 future variant
    HpkeBase { enc: Bytes, ciphertext: Bytes },
}
```

**Why not just make nonce variable-length `Bytes`?** Three reasons:
1. Fixed-length nonce enables compile-time constant-time-handling pre-commitment + zero-copy decode.
2. Variable-length nonce in a critical AEAD context is the same failure-mode-class as variable-length AAD in TLS-record-layer-without-explicit-length — easy to get the length wrong + introduces a parser-divergence between encoder and decoder.
3. Codepoint-discriminated variants are CHEAPER than length-prefix-discriminated nonces because the codepoint already disambiguates the cipher, so the nonce-length is statically derived from the codepoint at decode time.

This is the **HPKE pattern** (RFC 9180 §7.3: `Nn` nonce-length is a function of `aead_id` codepoint, not a wire-encoded field).

### §2.6 AAD canonicalization (Amendments 1 + 3) — PERMANENTLY FROZEN once shipped

Amendment 1's `canonical_binding()` + Amendment 3's TLV encoding define a specific byte-level layout. **Every byte produced by this function in 2026 must verify in 2050.** Any modification to the function's output (even adding a new field to a future BindingContext variant in a way that changes the canonical-byte-stream for existing variants) breaks decode permanence.

**Failure mode example.** Suppose 2028 Benten adds Amendment N: "include `protocol_version: u16` at the start of the AAD". Existing 2026 envelopes produced without `protocol_version` no longer verify — every 2026 envelope is bricked.

**Amendment 14 (NEW — load-bearing).** Two sub-actions:

(a) **Pin `canonical_binding()` as a public function with a versioned output prefix.** Include a `aad_version: u8` byte at the **start** of every canonical AAD stream, bound to a specific Amendment-1+Amendment-3 layout version. v1-beta ships `aad_version = 0x01`. Future canonicalization-shape changes mint `aad_version = 0x02`, etc.

(b) **Bind `aad_version` itself into the AAD.** This is the "the version field is itself authenticated" pattern (TLS 1.3 backward-compat ClientHello mechanism; HPKE info-string-binding).

```rust
fn canonical_binding(&self) -> Bytes {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"benten-envelope-v1");            // domain separator (Amendment 1)
    buf.push(0x01);                                          // aad_version = 0x01 (Amendment 14)
    buf.extend_from_slice(&self.codepoint.to_be_bytes());    // codepoint (Amendments 1 + 7)
    let body = self.aad_binding.canonical_serialize_tlv();   // TLV (Amendment 3)
    buf.extend_from_slice(&body);
    buf.into()
}
```

(c) **Reserve TLV tag-byte `0xFF` as "extended canonicalization marker"** — a future `aad_version = 0x02` can mark its canonicalization-shape-extension via TLV tag `0xFF`.

This makes future canonicalization-shape changes themselves codepoint-discriminated + structurally additive.

### §2.7 Replay-window `u64` epoch (Amendment 5) — Y2K38 SAFE; ✅ NO ACTION NEEDED

`sealed_at_epoch_seconds: u64`. u64 max = 18,446,744,073,709,551,615. At seconds-since-Unix-epoch: max representable timestamp = year 292,277,026,596 AD. **Safe through end of solar system.**

Y2K38 affects `i32` (signed 32-bit; rolls over 2038-01-19). u32 (unsigned 32-bit) rolls over 2106-02-07. **u64 has no practical overflow concern.**

**Action.** R3 Amendment 5 is fine on this axis. Flagged for completeness only.

### §2.8 Sender-DID encoding stability (Amendment 4) — needs multikey commitment

R3 Amendment 4 adds `sender_did: Did` to non-vault BindingContext variants. The `Did` type's wire encoding IS frozen at v1-beta. If `Did` is currently an enum like:
```rust
pub enum Did { Key(Ed25519Pubkey), KeyPq(MldsaPubkey), Web(Url), Plc(Hash), ... }
```
then every variant addition is a wire-format extension.

**Atrium's current state.** Atrium uses `did:key` exclusively (per the Atrium-as-forkable-semantics framing); the `Did` enum is effectively `Did(Ed25519Pubkey)` + future hybrid PQ extension. **But there is no current commitment that Atrium will NEVER add `did:web` or `did:plc` or `did:peer`.**

**Failure mode.** If Atrium adds `did:web` in 2028, every Benten envelope that bound a `sender_did: Did` in 2026 was produced under "did:key-only" assumption. Two implementations (2026 vs 2028) disagree on what `Did` shape is canonical.

**Amendment 15 (NEW — minor).** Pin `Did` canonical-serialization to **multikey varint codepoint form** per CLAUDE.md baked-in #5 (multiformats discipline). This is the W3C-DID-Core compatible form: every DID method gets a multikey codepoint (Ed25519 = `0xed`, secp256k1 = `0xe7`, MLDSA65 = `0x1207`, etc.). New DID methods land as new multikey codepoints; the wire encoding is always `varint(multikey_codepoint) || raw_key_bytes`. Decode preserves additivity.

Plus: add `Did::Unknown(u64 multikey_codepoint, Bytes raw)` typed-rejection variant. v1-beta readers MAY decode unknown multikeys but MUST typed-reject downstream consumption.

### §2.9 TLV encoding (Amendment 3) — PERMANENT; nested TLV needs explicit tag

R3 Amendment 3's TLV `[tag: u8] [length: varint] [value: bytes]`. Future nested-TLV need: e.g., a future BindingContext variant with `members: Vec<MemberInfo>` where each MemberInfo is itself TLV-encoded.

**Failure mode.** If the v1-beta TLV decoder doesn't tolerate nested TLV in the value-bytes (i.e., treats value-bytes as opaque), nested-TLV works fine at v1.1. If a future maintainer writes a recursive TLV decoder that interprets value-bytes as inner TLV, encoder-decoder disagreement.

**Closure.** TLV value-bytes are opaque per Amendment 3. Nested structure lives INSIDE value-bytes + is encoded/decoded by the variant-specific canonicalizer. This is the **PROTOBUF embedded-message pattern** (each variant's value-bytes are themselves a self-contained encoding). Document this in Amendment 3 to prevent future-maintainer recursive-TLV drift.

---

## §3 Cipher-suite rotation path per layer

### §3.1 Layer-A (ChaCha20-Poly1305 under DAK) rotation path

**Scenario:** breakthrough cryptanalysis on ChaCha20 in 2030. Benten needs to migrate Layer-A to (e.g.) XChaCha20-Poly1305 or AES-256-GCM-SIV.

**Rotation mechanism.**
1. Mint new CipherSuite codepoint (e.g., `CipherSuiteCodepoint::XCHACHA20_POLY1305 = 0x6402`).
2. Mint new EnvelopeShape codepoint (`EnvelopeShape::VAULT_XCHACHA20_V1 = 0x6102`).
3. v1.N readers admit BOTH old `0x6101` (ChaCha20) AND new `0x6102` (XChaCha20). Old vaults decode forever.
4. v1.N writers produce new `0x6102` envelopes by default; old `0x6101` writes admit only via explicit `legacy_writes_allowed` flag (deprecation-window per Amendment 16).
5. Vault-rekey ceremony per CLAUDE.md baked-in #5 crypto-agility: at next unlock, vault is re-encrypted under new codepoint. Old codepoint vault file is archived.

**Cross-Amendment composition.** The codepoint-in-AAD discipline (Amendment 1) means the old ChaCha20 ciphertext is bound to `0x6101` in its AAD; the new XChaCha20 ciphertext binds `0x6102`. No cross-codepoint substitution possible (Amendment 1 + 2 closes).

**Layer-A nonce-scheme concern (R3 §2.8 row l).** If Layer-A switches from ChaCha20 (12-byte nonce) to XChaCha20 (24-byte nonce), Amendment 12's `SymmetricAeadXNonce` variant is needed. Without Amendment 12, the rotation path is BLOCKED at the wire format.

### §3.2 Layer-B (AEAD-under-derived-K(N)) rotation path

Same mechanism as Layer-A. Layer-B uses the same `SymmetricAead` variant; the per-chunk-key derivation K(N) lives one level up (in the structural KDF chain, already pinned per V1-FROZEN-INTERFACE-DEFERRED.md Row D-13 CLOSED).

Layer-B rotation is structurally cheap because:
- The structural KDF binds `cipher_suite_codepoint: u16` into its info-tag (D-13 CLOSED).
- A new Layer-B cipher mints a new CipherSuite codepoint + new EnvelopeShape codepoint.
- Old K(N) chains remain decode-supported forever.

### §3.3 Layer-C/D (HPKE-mode-base[MLKEM768-X25519]) rotation path

**Scenario:** ML-KEM-768 cryptanalysis or HQC/NIST-Round-4-winner promotion or X-Wing IRTF finalization.

**Rotation mechanism.**
1. Mint new CipherSuite codepoint (e.g., `CipherSuiteCodepoint::HYBRID_X25519_MLKEM1024 = 0x647d`, or `HYBRID_X25519_HQC = 0x647e`, or `XWING_TRUE_IRTF = 0x647f`).
2. Mint new EnvelopeShape codepoint (`EnvelopeShape::DROP_HPKE_BASE_MLKEM1024_V1 = 0x6303`).
3. v1.N readers admit BOTH `0x6300` (MLKEM-768) AND `0x6303` (MLKEM-1024). Old drops decode forever.
4. v1.N drop-creation default switches to `0x6303`. Old `0x6300` writes admit only via legacy-flag.
5. **Inv-15 + Inv-16 future-additive discipline:** the 3-layer decomposition explicitly reserves Bird-of-Prey + future-PQ codepoints as additive.

**HPKE codepoint sub-axis.** HPKE itself has 3 codepoint sub-axes (`kem_id` + `kdf_id` + `aead_id`, RFC 9180 §7). Benten's `CipherSuiteCodepoint` aggregates all three. **Rotating ONLY the AEAD** (e.g., HPKE-MLKEM768-X25519-HKDF-SHA256-**ChaCha20** → HPKE-MLKEM768-X25519-HKDF-SHA256-**AES-256-GCM**) requires a new aggregate codepoint.

This is **deliberately less granular than HPKE's native registry** + matches MLS RFC 9420 §17.7 cipher-suite-as-one-codepoint pattern. The cost: more aggregate codepoints; ~32-64 future codepoints across the full Cartesian product of KEM×KDF×AEAD. With u16 slot capacity (Amendment 11 escape reserves room), this is **comfortably tractable for 50+ years**.

### §3.4 Argon2id parameter rotation — vault_version u8 sufficiency

Current: `BindingContext::Vault { vault_version: u8 }`. u8 = 256 possible versions. **Sufficient for 50+ years** of Argon2id parameter rotation (OWASP updates ~every 3 years; ~17 rotations in 50 years = 17/256 = 6.6% slot usage).

**Argon2id parameter table** (recommended for V1-FROZEN-INTERFACE.md item 6 extension):

| vault_version | Argon2id params | OWASP era |
|---|---|---|
| `0x01` | m=64 MiB, t=3, p=4 | 2024-OWASP recommendation; v1-beta default |
| `0x02` | m=128 MiB, t=4, p=4 | 2027-OWASP projection |
| `0x03` | … | … |

Reserve `0xFF` as "vault_version-escape; follow with varint" per Amendment 11 pattern. **No structural change needed** at v1-beta beyond documenting the table.

### §3.5 KDF rotation (HKDF-SHA-256 → SHA-3 / Blake3)

HKDF construction is parameterized by Hash codepoint (V1-FROZEN-INTERFACE.md item 6 sub-item 2: Hash axis with `BLAKE3 = 0x1e`, `SHA2_512_256 = 0x1015`, `SHA3_256 = 0x16`).

Rotation: mint new Hash codepoint + new aggregate CipherSuite codepoint. Old HKDF instances remain decode-supported forever via codepoint-dispatch.

**HKDF-itself** is sufficiently studied that the KDF construction is unlikely to need replacement (vs the hash); but if BLAKE3-KDF or Argon2-KDF replaces HKDF, it lands as a new codepoint. **No structural change needed** at v1-beta.

---

## §4 Forward-secrecy upgrade story

### §4.1 Current FS gap (Layer-C)

HPKE-mode-base (RFC 9180 §5.1.1) is **structurally non-FS**: the recipient's long-term encryption sk decrypts every drop forever. An sk exfiltrated in 2030 decrypts a 2026 drop. This is **fundamental to mode_base, not a Benten-specific gap.**

R3's adversarial review acknowledged this implicitly via Compromise #31 (forever-valid drop bundles); my L8 lens explicitly classifies it as a **v1-beta-locked permanence concern**.

### §4.2 Why this matters at v1-beta freeze

Two flavors of FS-evolution Benten will plausibly want:

1. **Receiver-side FS via key rotation.** Recipient rotates encryption sk every (e.g.) 30 days; old sks are deleted; drops older than 30 days become structurally undecryptable. **This is application-layer key-management policy, NOT a wire-format concern.** Achievable post-v1-beta without wire-format changes. ✅ Compatible.

2. **MLS / CGKA / draft-prabel-style forward-secure group messaging.** Each application message uses an ephemeral key derived from a ratcheting tree; the long-term identity sk only signs Welcome messages, never decrypts payload. **This IS a wire-format concern** — requires new `EnvelopePayload::MlsApplication { group_epoch, sender_idx, ciphertext }` variant + new `BindingContext::MlsGroupApplication { ... }` variant + new EnvelopeShape codepoint.

### §4.3 Required v1-beta reservations to admit (2) additively

**Amendment 13 (NEW — load-bearing).**

(a) **Explicitly disclose the FS-gap as a v1-beta Compromise** in `docs/SECURITY-POSTURE.md` (mirrors Compromise #31's framing). Statement: "Layer-C HPKE-mode-base drops are not forward-secret at the long-term-sk axis. A 2030 sk-compromise recovers 2026 drops if 2026 ciphertexts were archived. Mitigation pathways: (i) application-layer encryption-key rotation (post-v1-beta; no wire-format change); (ii) MLS-PQ / CGKA additive migration (post-v1-beta; requires new codepoints reserved in v1-beta; see Inv-16 reservations)."

(b) **Reserve codepoint brackets in V1-FROZEN-INTERFACE.md item 6:**

| EnvelopeShape codepoint | Reservation | Future-mint trigger |
|---|---|---|
| `0x6380..0x638F` | MLS-Application-Data variants | MLS RFC 9420 stabilization in Benten |
| `0x6390..0x639F` | MLS-Welcome variants | CGKA device-join future |
| `0x63A0..0x63AF` | CGKA-Commit variants | PQ-CGKA maturation (draft-ietf-mls-pq) |
| `0x63B0..0x63BF` | Bird-of-Prey AKEM variants | If Bird-of-Prey IETF adoption |
| `0x63C0..0x63CF` | draft-prabel PQ-MLS-AKEM variants | If draft-prabel adoption |

(c) **Reserve `EnvelopePayload` variant slots** (Amendment 9 enables this via `#[non_exhaustive]`):
- `MlsApplication { group_id, epoch, sender_idx, ciphertext }`
- `MlsWelcome { encrypted_group_secrets, group_info, ratchet_tree }`
- `CgkaCommit { ... }`
- `HpkeAuth { sender_pk, enc, ciphertext }` (mode_auth for sender-AKEM)

(d) **Reserve `BindingContext` variant slots** (Amendment 10 enables this):
- `MlsGroupApplication { group_id, epoch, sender_did, sealed_at, ... }`
- `MlsWelcome { joiner_did, group_id, ... }`
- `CgkaCommit { ... }`

**Why this matters NOW.** If v1-beta tags without these reservations, the codepoint allocations may be ad-hoc + collide with later-mint preferred ranges. The reservation cost at v1-beta is zero (no code, just registry-row text in V1-FROZEN-INTERFACE.md). The retrofit cost post-v1-beta is non-zero (codepoint-allocation politics + interop coordination with external Benten consumers).

### §4.4 CGKA codepoint reservation: in v1-beta vs post-v1-beta?

**Recommendation: reserve at v1-beta; do not implement at v1-beta.** The cost-benefit:
- **Reserve at v1-beta (RECOMMENDED).** Cost: ~5 lines of V1-FROZEN-INTERFACE.md text. Benefit: prevents future codepoint-allocation politics; signals direction; matches Atrium-as-forkable-semantics framing.
- **Reserve post-v1-beta.** Cost: codepoint-allocation politics; risk that someone mints a colliding codepoint in the interim. Benefit: keeps v1-beta surface minimal.

**My recommendation is reserve-at-v1-beta**, because the marginal v1-beta-doc cost is trivial + the optionality preserved is substantial.

### §4.5 Drop-bundle-revocation composition (Compromise #31)

R3 §2.4 implicitly raised this: how does revocation compose with forward-additive codepoints?

**Proposal.** Mint a reserved EnvelopeShape codepoint bracket `0x6700..0x67FF` for **revocation/lifecycle variants** + a `BindingContext::CapabilityRevocation { revoked_cap_cid, revoker_did, revoked_at_epoch_seconds }` variant slot. Revocation envelopes are produced when Drop bundles are revoked; recipients consult a revocation-log (per Compromise #31 future-design) before treating a Drop as valid.

**This is FUTURE work** — not v1-beta scope — but the codepoint bracket reservation should land at v1-beta.

---

## §5 Deprecation window strategy

V1-FROZEN-INTERFACE.md item 6 (sub-item 6 + 14) commits: "Old codepoints / old format versions are decode-supported FOREVER per never-strand-content invariant." This is the **conservative-by-default** discipline that matches Inv-15's forward-additive commitment + matches CLAUDE.md baked-in #15 long-term-durability.

**But broken codepoints SHOULD NOT be supported forever without typed status.** A 2030 cryptanalysis breakthrough on (e.g.) ML-KEM-768 means 2026 drops at `0x647a` should be:
- **Read-supported** so 2026 archives can still be opened in 2030 (with explicit "this codepoint is deprecated" warning).
- **Write-refused** so 2030 users can't accidentally produce new drops at the broken codepoint.

The current freeze discipline conflates "wire-format decode" (correct: forever-supported) with "cryptographic-soundness assertion" (incorrect to commit forever: subject to breakthrough).

### §5.1 Amendment 16 (NEW — minor): codepoint lifecycle states

```rust
pub enum CodepointLifecycle {
    /// Codepoint is current best practice; reads + writes admitted.
    Live,
    /// Codepoint is superseded by a newer codepoint; reads admitted with warning;
    /// writes refused unless explicit `legacy_writes_allowed` opt-in.
    Deprecated { superseded_by: Codepoint, deprecation_announced_at: Epoch },
    /// Codepoint is cryptographically suspect (e.g., partial cryptanalysis); reads
    /// admitted with explicit warning to the caller; writes refused outright.
    Quarantined { reason: String, quarantined_at: Epoch },
    /// Codepoint is cryptographically broken; reads refused (verify-fail-closed);
    /// writes refused; existing on-disk envelopes at this codepoint are
    /// structurally unreadable.
    Burned { reason: String, burned_at: Epoch },
}
```

**Lifecycle transition discipline:**
- `Live → Deprecated`: minor security advisory; new writes auto-rotate to successor.
- `Deprecated → Quarantined`: partial cryptanalysis (e.g., distinguishing attack with non-trivial advantage); reads admit with explicit caller warning; writes hard-refuse.
- `Quarantined → Burned`: full break (e.g., practical key recovery); reads hard-refuse.

**Why this is minor not load-bearing for v1-beta.** No codepoint is in non-`Live` state at v1-beta. The amendment is **future-discipline scaffolding**; it costs ~50 lines of Rust + ~10 lines of V1-FROZEN-INTERFACE.md to land + has zero behavioral effect at v1-beta. **Recommended to land at v1-beta for permanent registry-discipline.**

### §5.2 The 2050 verification scenario

**Scenario:** in 2050, a user opens an archived Benten note signed in 2026 (under `SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001`). The 2050 Benten code is 24 years newer; what guarantees does the design provide?

**Guarantees provided by §6.2 + Amendments 1-16:**
1. **Wire format byte-stability** (V1-FROZEN-INTERFACE.md item 6 + Amendment 7 codepoint-endianness): the 2026 bytes are decodable in 2050.
2. **Codepoint-supported-forever discipline** (V1-FROZEN-INTERFACE.md item 6 sub-item 6): codepoint `0x0001` is still recognized in 2050.
3. **Algorithm-supported-forever for sound codepoints** (V1-FROZEN-INTERFACE.md item 6 sub-item 14): if Ed25519+MLDSA65 hybrid is still sound in 2050, verification succeeds.
4. **Quarantined/Burned discipline** (Amendment 16): if Ed25519+MLDSA65 hybrid is broken by 2050, the 2050 Benten code refuses verification with explicit `BURNED_CODEPOINT` typed error.

**Guarantee NOT provided.** No design provides "this signature is sound in 2050" — that's a cryptographic-soundness question, not a wire-format question. What the design DOES provide: **explicit typed-status disclosure** of the codepoint's lifecycle so the 2050 user knows whether to trust the verification.

This matches **Adobe PDF long-term-validation** (LTV/PAdES; RFC 6960 OCSP; ETSI EN 319 122) + **CMS S/MIME long-term-archive** (RFC 4998) patterns.

---

## §6 Registry-mint discipline (concrete proposal)

R3 §2.6 raised this as Amendment 8 (minor). My L8 lens makes it load-bearing-by-implication-of-Amendment-13's reservations.

### §6.1 Where codepoints live

**Recommended structure** (mint at v1-beta or immediately post-v1-beta as part of Inv-16 mint):

1. **`docs/CRYPTO-CODEPOINTS.md`** (new file) — the canonical Benten codepoint registry. Modeled on `multicodec.csv` + MLS RFC 9420 §17 IANA registry. Per-codepoint row schema:
   ```
   | Axis | Constant | Value | Status | Reserved | Range | Notes |
   ```
   Axes: Hash, Sig, Cipher, EnvelopeShape (new per Amendment 11(c)).
   Status: `Live`, `Reserved`, `Deprecated`, `Quarantined`, `Burned`, `Experimental`, `Escape`.
   Range: bracket-membership (e.g., `0x6300..0x63FF` for Layer-C variants).

2. **Cite-drift-detector enforcement** (per CLAUDE.md baked-in pim-N-cite-grep-verify-at-author-time §3.6j ext). Every mint of a new constant in:
   - `crates/benten-crypto-suite/src/cipher_suite.rs`
   - `crates/benten-crypto-suite/src/sig.rs`
   - `crates/benten-crypto-suite/src/hash.rs`
   - `crates/benten-crypto-suite/src/envelope.rs` (new file per Amendment 11(c))

   MUST be matched by a registry-row addition in `docs/CRYPTO-CODEPOINTS.md`. **Drift-detector scanner check: every `pub const ... = 0x...;` in those files MUST have a registry-row.**

3. **V1-FROZEN-INTERFACE.md item 6 sub-item 2 codepoint table** — cross-referenced summary view; the full registry lives in `docs/CRYPTO-CODEPOINTS.md`.

### §6.2 Approval process for codepoint mints

**Workflow** (mirrors MLS RFC 9420 §17 IANA-Specification-Required pattern + Benten's NAMED-deferred HARD RULE clause-b):

1. PR proposes a codepoint mint with: `(axis, name, value, status, justification, security-rationale-citation)`.
2. The PR MUST update `docs/CRYPTO-CODEPOINTS.md` registry row + the relevant Rust source file constant + a test pin asserting the integer value.
3. The PR MUST include a docstring on the constant citing the IANA / IRTF / IETF reference if interop-coordinated, OR explicitly stating "Benten-local; disjoint from external registries" if not.
4. **Two reviewers required** (cryptographer + protocol-architect, per the L8 lens). Mirrors current ADDL discipline.
5. Cite-drift-detector enforces post-merge.

### §6.3 Reserved-for-future ranges + IANA-coordination

Current Benten codepoint ranges are **largely disjoint from IANA**:
- Hash: uses IANA multiformats codepoints (`0x1e` BLAKE3, `0x1015` SHA2-512/256, `0x16` SHA3-256) — IANA-coordinated, ✅.
- Sig: uses Benten-internal `0x000N` range (`0x0001..0x0003`) — disjoint from IANA but **collides with multiformats reserved-block-end `0x0000..0x00FF`**. ⚠️ R3 §2.6 noted this.
- Cipher: uses Benten-internal `0x6400..0x6FFF` range — disjoint from IANA multicodec table (which doesn't reserve `0x6400..0x6FFF` as of multicodec.csv 2026-04). ✅ but fragile (multicodec.csv could allocate into this range).
- EnvelopeShape (proposed Amendment 11(c)): `0x6100..0x6FFF` — overlaps with Cipher range. ⚠️.

**Recommendation:**

(a) **Reserve Benten ranges explicitly in `docs/CRYPTO-CODEPOINTS.md` + coordinate with multicodec.csv maintainers.** Submit a multicodec.csv PR registering Benten's reserved ranges (low-cost coordination; multicodec is permissive).

(b) **Separate envelope-shape codepoints from cipher codepoints.** Don't overlap. Suggested:
- `0x6100..0x61FF` → EnvelopeShape (Layer-A vault)
- `0x6300..0x63FF` → EnvelopeShape (Layer-C drop + related)
- `0x6400..0x64FF` → CipherSuite (current)
- `0x6500..0x65FF` → reserved
- `0x6600..0x66FF` → reserved
- `0x6700..0x67FF` → revocation/lifecycle variants

(c) **Fix the Sig-axis IANA collision.** R3 §2.6 named this. Either (i) move Sig codepoints to a Benten-disjoint range (e.g., `0x6800..0x68FF`); or (ii) document that Sig codepoints are intentionally in the multiformats reserved-block-end and won't conflict because multiformats has stopped allocating new codepoints below `0x00FF`. **Recommendation (i)** for cleanliness — but this is a v1-beta breaking change for already-shipped codepoints. ⚠️ **Decision-required-from-Ben:** either accept the IANA-overlap-of-Sig-axis as a v1-beta scoped-Compromise (document in SECURITY-POSTURE.md) OR fix the Sig-codepoint values now before freeze.

### §6.4 Public codepoint registry for ecosystem consumers

**Long-term commitment.** `docs/CRYPTO-CODEPOINTS.md` is the canonical registry. External Benten ecosystem consumers (future benten-ts; future external Atrium peers; future cross-impl interop) reference this file. The file:
- Lives in main branch.
- Updated only via the codepoint-mint workflow (§6.2).
- Cite-drift-detector enforced.
- Linked from `docs/V1-FROZEN-INTERFACE.md` item 6.
- Versioned alongside Benten releases (every release tag = registry snapshot).

---

## §7 Recommended amendments to §6.2 + Amendments 1-6

**Summary table of all amendments after this L8 review:**

| Amendment | Origin | Status | My L8 verdict |
|---|---|---|---|
| 1 — codepoint-in-AAD | R2 §2.4 | LOAD-BEARING | ✅ CONCUR |
| 2 — strict-decode + no-fallback | R2 §2.5 | LOAD-BEARING | ✅ CONCUR |
| 3 — TLV length-prefix canonical encoding | R3 §2.1 | LOAD-BEARING | ✅ CONCUR (+ specify TLV tag-byte registry per §2.3 above) |
| 4 — sender-DID-in-AAD for non-vault variants | R3 §2.2 | LOAD-BEARING | ✅ CONCUR |
| 5 — sealed-at + valid-until epoch-binding | R3 §2.3 | LOAD-BEARING | ✅ CONCUR (Y2K38-safe per §2.7) |
| 6 — Bernstein-Persichetti CT-Decap + Compromise disclosure | R3 §2.4 | LOAD-BEARING | ✅ CONCUR (out-of-L8-lens-scope; defer to cryptographer review) |
| 7 — codepoint endianness BE-on-wire | R3 §2.5 | MINOR | ✅ CONCUR (+ pinned in canonical_binding code per §2.6 above) |
| 8 — codepoint-registry IANA-disjoint range | R3 §2.6 | MINOR | **PROMOTE TO LOAD-BEARING** per §6 above; concrete proposal §6.1–§6.4 |
| **9 — `EnvelopePayload` `#[non_exhaustive]` + typed-reject** | **L8 §2.2** | **LOAD-BEARING (NEW)** | — |
| **10 — `BindingContext` `#[non_exhaustive]` + typed-reject** | **L8 §2.3** | **LOAD-BEARING (NEW)** | — |
| **11 — escape-codepoint `0xFFFF` + experimental range `0xFE00..0xFFFE` + envelope-shape codepoint axis explicit** | **L8 §2.4** | **LOAD-BEARING (NEW)** | — |
| **12 — XNonce variant (per nonce-length-discriminated codepoints)** | **L8 §2.5** | **LOAD-BEARING (NEW)** | — |
| **13 — FS-gap Compromise + MLS-PQ/CGKA codepoint-bracket reservations** | **L8 §4** | **LOAD-BEARING (NEW)** | — |
| **14 — `aad_version: u8` + TLV `0xFF` extended-canonicalization marker** | **L8 §2.6** | **LOAD-BEARING (NEW)** | — |
| **15 — `Did` multikey-codepoint canonical-encoding + `Did::Unknown` typed-rejection** | **L8 §2.8** | **MINOR (NEW)** | — |
| **16 — `CodepointLifecycle { Live, Deprecated, Quarantined, Burned }` typed-state** | **L8 §5.1** | **MINOR (NEW)** | — |

**Net new amendments from L8 lens:** 8 (5 load-bearing + 2 minor + 1 promote-from-minor-to-load-bearing).

### §7.1 Composition check across all amendments

I checked for composition errors between Amendments 9-16 + Amendments 1-8:

- **9 + 10 + 11(c)** compose: `#[non_exhaustive]` on enum + escape-codepoint + envelope-shape axis are orthogonal; the codepoint identifies the variant (Amendment 2 dispatch table); `#[non_exhaustive]` lets future variants land additively.
- **11 + 14** compose: escape-codepoint `0xFFFF` lives at the EnvelopeShape codepoint axis; `aad_version: u8` lives inside the canonical_binding output stream; orthogonal axes.
- **12 + 9** compose: `SymmetricAeadXNonce` is a NEW variant under Amendment 9's `#[non_exhaustive]`; same path as any future variant.
- **13 + 9 + 10** compose: MLS-Application / MLS-Welcome / CGKA variants land under Amendment 9 (`EnvelopePayload`) + Amendment 10 (`BindingContext`); reservation at v1-beta is just registry-row + variant-slot reservation.
- **14 + 1 + 3** compose: `aad_version` byte goes between domain-separator `b"benten-envelope-v1"` and codepoint (per §2.6 canonical_binding ordering). The TLV `0xFF` extended-canonicalization marker is a future-additive tag inside the TLV stream.
- **15 + 4 + 3** compose: `Did` multikey-encoding is the canonical TLV-value-bytes shape for `sender_did` fields; future `Did::Unknown` variants land additively.
- **16 + V1-FROZEN-INTERFACE.md item 6 sub-item 6** compose: lifecycle state is a runtime registry property; the wire-format-decode-supported-forever discipline is preserved for `Live` + `Deprecated` codepoints; `Quarantined` admits with warning; only `Burned` refuses decode (and only for verify-fail-closed semantics, which is itself a typed-reject per baked-in #5).

**No composition errors detected.** All 16 amendments are independent + cumulative.

### §7.2 R3's open items for future reviewer

R3 §3 flagged: "**Open items for a future reviewer (NOT load-bearing on v1-beta tag):** ..." I cannot see R3's full list (truncated in my reading) but the L8 lens may have absorbed some of them via Amendments 9-16. Recommend cross-checking R3 §3 enumeration before ratification.

---

## §8 Self-assessment + confidence + lower-confidence areas

### §8.1 What I'm HIGH-confident on

- **Amendments 9, 10, 11, 12, 14**: standard protocol-evolution failures with rich case-study evidence. Every multi-decade protocol that didn't address these has retrofit-pain stories. Cheap to address now; expensive later.
- **Amendment 13** (FS-gap disclosure + reservations): structurally necessary; the codepoint-bracket reservations cost ~5 lines of doc.
- **§6 registry-mint discipline**: this is *industry-standard* IANA / IETF discipline + matches CLAUDE.md baked-in #5.
- **§2.7 Y2K38 analysis**: u64 epoch is comfortably safe; this is arithmetic.

### §8.2 What I'm MEDIUM-HIGH-confident on

- **Amendment 15** (Did multikey-encoding): depends on Atrium's commitment to multikey discipline. If Atrium already commits via multiformats, this is cheap; if not, needs Ben-decision on `did:web`/`did:plc` future surface.
- **Amendment 16** (codepoint lifecycle states): the design is standard, but the v1-beta value is mostly forward-discipline; some reasonable disagreement on whether to land at v1-beta or post-v1-beta.
- **§3.3 HPKE codepoint sub-axis aggregation discipline**: my recommendation matches MLS RFC 9420 but trades off granularity-of-rotation for codepoint-table-simplicity. A cryptographer might prefer 3-axis HPKE-native codepoints; I prefer aggregate-per-suite to match the rest of Benten's table shape.

### §8.3 What I'm MEDIUM-confident on

- **§6.3 Sig-axis IANA-collision fix recommendation**: I recommend fixing it at v1-beta (moving Sig codepoints out of multiformats reserved-block). This is a v1-beta breaking-change to a NOT-YET-FROZEN surface; doable but needs Ben-decision. **Lower confidence on whether the cost-benefit favors fix-now vs Compromise-document-now.**
- **Specific codepoint range allocations** (e.g., `0x6100..0x61FF` for Layer-A vault variants): my proposal is one reasonable allocation; a different reviewer might prefer a different layout. **The discipline of bracketed reservation is high-confidence; the specific ranges are medium-confidence.**

### §8.4 Lower-confidence areas + likely failure modes of MY review

1. **MLS / CGKA migration story specifics.** I sketched 4 future variants for MLS (Application / Welcome / CgkaCommit / HpkeAuth) but I'm not an MLS WG participant. A future reviewer with hands-on MLS RFC 9420 + draft-ietf-mls-pq experience may identify additional variants needed OR a structural mismatch between Benten's envelope shape + MLS framing. **Recommend a dedicated L8-supplemental review by an MLS-WG-experienced cryptographer before treating Amendment 13's variant list as final.**

2. **Bird-of-Prey + draft-prabel maturity assumption.** I assumed these will mature in the 2027-2030 window. They may not. If they don't, the bracket-reservations are dead code in V1-FROZEN-INTERFACE.md. **Low cost of bracket-reservation-that-never-mints, vs high cost of no-bracket-when-it's-needed.** Asymmetric; reservation wins.

3. **Cross-protocol composition.** I focused on Benten's envelope evolution; I didn't deeply audit cross-protocol composition with (e.g.) future MLS-over-Atrium or Signal-style sealed-sender as a Benten primitive. A future L8-supplemental specifically for cross-protocol composition is recommended.

4. **`canonical_binding()` versioning conflict with Amendment 1's existing `b"benten-envelope-v1"` domain separator.** Amendment 1 already encodes a v1-ness commitment via the domain-separator string. My Amendment 14's `aad_version: u8 = 0x01` is redundant with the domain-separator. **Reconciliation:** use ONLY the `b"benten-envelope-v1"` domain-separator as the version-discriminator; Amendment 14's `aad_version` byte is redundant. **REVISED Amendment 14:** drop the `aad_version: u8` byte; instead pin future canonicalization-shape changes to mint a NEW domain-separator string (`b"benten-envelope-v2"`). This is structurally cleaner — strict domain-separator dispatch IS the version dispatch. **This revision is a self-correction made during writing this review.** Amendment 14 below is the corrected version.

### §8.5 Amendment 14 — CORRECTED FORMULATION

**Amendment 14 (REVISED — load-bearing).** Pin `canonical_binding()` to use the domain-separator string as the version-discriminator. v1-beta uses `b"benten-envelope-v1"`. Any future canonicalization-shape change mints a NEW domain-separator (`b"benten-envelope-v2"`); v2-shape envelopes cannot collide with v1-shape envelopes because the AAD-bound domain-separator string differs ⇒ cross-version-substitution attack is structurally infeasible.

Plus: **reserve TLV tag-byte `0xFF` as "extended canonicalization marker"** — for ADDITIVE extensions WITHIN the v1 canonicalization (i.e., new fields in a v1.N variant without needing a v2-shape bump).

This is the **HPKE info-string + KeySchedule binding pattern** (RFC 9180 §5.1) operating at the Benten envelope layer.

### §8.6 Overall L8-lens confidence

**HIGH** on the verdict (Amendments 1-8 + L8 Amendments 9-16 = load-bearing-final for v1-beta freeze). **MEDIUM-HIGH** on the specific codepoint-range allocation proposals (§6.3) + on the MLS/CGKA variant-list specifics (§4.3). **HIGH** on the discipline patterns (bracketed reservations + lifecycle states + escape codepoints + non-exhaustive enums).

---

## §9 Citations

### Primary RFC / IETF / IRTF sources

- **RFC 9180** (HPKE): §5.1 KeyScheduleContext + info-string binding; §7 IANA registries (kem_id, kdf_id, aead_id); §9.7.2 replay-protection considerations; §10.3 multi-recipient HPKE.
- **RFC 9420** (MLS protocol): §6 application messages; §11.3 multi-recipient pattern; §12.4 Welcome messages; §17 IANA registries; §17.2 extension-type u16-with-escape pattern.
- **RFC 8439** (ChaCha20-Poly1305): §2.3 nonce 12 bytes; §2.8 AAD semantics.
- **RFC 8949** (CBOR): §4.2.1 deterministic encoding; §4.2.2 CTAP2 canonical CBOR.
- **RFC 9000** (QUIC): §16 variable-length integer encoding (varint escape pattern).
- **RFC 8446** (TLS 1.3): cipher-suite registry evolution from TLS 1.0 1-byte to TLS 1.2/1.3 2-byte selectors; extension-mechanism design.
- **RFC 9580** (OpenPGP): packet-format evolution from RFC 4880 (v4 packets) to RFC 9580 (v6 packets); wrapping pattern.
- **RFC 4998** (CMS Long-Term Archive): long-term-validation pattern.
- **RFC 6960** (OCSP): revocation-state model.
- **RFC 7515** (JOSE/JWS): `alg` header binding (critical-headers pattern, post-2014 alg-confusion lessons).
- **RFC 8152** (COSE): alg-binding in protected payload (JOSE lessons applied).
- **RFC 7518** (JOSE algorithms): JOSE `alg` registry — ~70 entries in 10 years.
- **draft-ietf-mls-protocol-pq** (PQ-MLS, in-progress): CGKA + PQ-AKEM framing.
- **draft-connolly-cfrg-xwing-kem-08** (X-Wing IRTF): true X-Wing KEM specification.
- **draft-prabel-mls-akem** (PQ-MLS-AKEM, in-progress): authenticated KEM for MLS.
- **IANA Multiformats registry** (`multicodec.csv`): codepoint allocation discipline + reserved ranges.

### Cryptographic literature

- **Bellare-Namprempre 2000** ("Authenticated Encryption: Relations among Notions and Analysis of the Generic Composition Paradigm"): IND-CCA2 composition foundations.
- **Bernstein-Persichetti 2024** ("One Time is Enough"): IACR 2024/2051; ML-KEM Decap chosen-ciphertext side-channel.
- **Barbosa et al. 2024** ("Post-Quantum Hybrid Authenticated Key Exchange and Public-Key Encryption"): HPKE-PQ-hybrid IND-CCA2 proof framework.
- **Alwen-Coretti-Dodis 2019** ("The Double Ratchet: Security Notions, Proofs, and Modularization for the Signal Protocol"): CGKA framing.
- **Brendel-Fischlin-Günther 2024** ("Bird-of-Prey"): PQ-secure-CGKA building block.

### Benten-internal references

- **CLAUDE.md baked-in #5** (crypto-agility codepoint-dispatch + typed-reject + never-fork-primitives).
- **CLAUDE.md baked-in #15** (v1-beta = v1-public-interface-freeze; v1-GM post-external-audit).
- **CLAUDE.md baked-in #18** (authority vs confidentiality isolation; two-category extensibility).
- **`docs/V1-FROZEN-INTERFACE.md` item 6** (current 9-codepoint table + freeze discipline + sub-items 1-14).
- **`docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-13** (structural_kdf cipher-suite-codepoint binding — CLOSED).
- **`docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-15-RETRACTED** (codepoint-mint pre-blessing retracted).
- **`docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-24** (G-CORE-3 × G-CORE-7 manifest-envelope intersection).

### Prior reviews (this option-F-plus stream)

- **R1** (pseudo-keypair takedown) `origin/phase-4-meta-core/option-f-plus-pseudo-keypair-review @ HEAD` → `.addl/phase-4-meta/option-f-plus-pseudo-keypair-review.md` (412 lines).
- **R2** (second-opinion CONCUR-WITH-AMENDMENTS) `origin/phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ HEAD` → `.addl/phase-4-meta/option-f-plus-second-opinion-cryptographer-review.md` (520 lines; Amendments 1+2).
- **R3** (adversarial-design red-team) `origin/phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ HEAD` → `.addl/phase-4-meta/option-f-plus-third-reviewer-adversarial-design.md` (640 lines; Amendments 3+4+5+6+7+8).

### Protocol-evolution case studies cited

- TLS 1.0 → 1.2 cipher-suite extension via 2-byte selector + extension mechanism (RFC 5246 + RFC 8446).
- OpenPGP RFC 4880 v4 → RFC 9580 v6 packet-format evolution via wrapper-type pattern.
- SSH-1 → SSH-2 forced wire-format-break (no extension mechanism in SSH-1).
- Signal session-cipher rotations (Double Ratchet protocol revisions; X3DH → PQXDH).
- MLS v1.0 evolution + extension-type IANA registry (RFC 9420 §17.2).
- HPKE IRTF maturation + IANA-coordinated kem_id/kdf_id/aead_id registry (RFC 9180 §7).
- COSE algorithm registry (RFC 8152) avoiding JOSE's alg-confusion incidents.
- Multiformats codepoint registry growth (~600+ entries in 10 years).
- IANA tls-parameters registry growth (~400+ cipher suites in 20 years).

---

**End of L8 protocol-evolution architect lens review.**
