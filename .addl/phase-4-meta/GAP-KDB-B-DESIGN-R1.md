# GAP-KDB Shape-B Identity Model — R1 Design Record

Ref 35b27b1c · Ben chose Shape-B (content-addressed key-set DID) · 5-lens adversarial critic council · VERDICT: SOUND, no killer.

---

## DESIGN SPEC (design-architect)

I have everything I need. All sizes, byte layouts, resolve sites, and the GAP-KDB asymmetry are confirmed against real code. Here is the design spec.

---

# Shape-B Identity Model — Concrete Design Spec (GAP-KDB closure, v1-beta freeze)

**Ref:** `35b27b1c` · **Scope:** Phase-4-Meta-Core (freezes the v1 interface) · **Decision basis:** Ben's Shape-B ratification (content-addressed key-set DID; A = documented fallback only)

## 0. The core asymmetry this spec removes (why Shape-B, grounded)

The seal path is already **half self-certifying and half address-book-trusting**, and GAP-KDB is exactly that seam:

- **Sender side is self-certifying today.** `open_inner` resolves the recovered sender-DID to its hybrid verifying key *from the DID string itself* — `Did::parse_validated_hybrid(did_str)` then `.resolve_hybrid()` (`layer_c.rs:295-296`), verified post-decrypt against the recipient-independently-held binding (`layer_c.rs:911-927`, the F-2 soundness property). Zero I/O, zero address book.
- **Recipient side is NOT.** `seal_sealed_sender(recipient_pub: &RecipientPublic, audience_did: &AudienceDid, …)` (`layer_c.rs:947-955`) takes the KEM key and the audience DID as **two independently-chosen params**. Nothing binds `recipient_pub` to `audience_did`. The whole confidentiality stack rests on the "Recipient-key premise" of `SECURITY-PROOFS.md §4.1` (lines 92-102) and rung 4 of `THREAT-MODEL.md §2` (lines 104-110) — both **assume** the KEM key was honestly obtained. An active attacker who substitutes `recipient_pub` at the address-book boundary reads everything, and every proof "silently trusts an honest address book."

The same gap exists at the group level: `audience_set_commitment(recipient_dids)` (`layer_c.rs:390-401`) blinds a DID roster that is a **separate, un-cross-checked list** from the `&[RecipientPublic]` fed to the wrap in `seal_group_impl` (`layer_c.rs:1263`). Substituting KEM keys in the second list is invisible to the first.

**Shape-B makes the recipient side symmetric to the sender side:** the audience DID *commits* the recipient's key-set (including the KEM key), so the KEM key is recovered-and-verified *from the DID* exactly as the sender's verifying key already is. Unforgeability reduces to 2nd-preimage resistance of BLAKE3-256 over canonical DAG-CBOR (safe indefinitely). This is native to Inv-15 (identity = canonical-payload-CID) and dispatch-conventions §3.5s (cross-ecosystem-identifier-as-content).

---

## 1. DID FORMAT — the load-bearing HASH-vs-EMBED call

### Decision: **HYBRID — embed the signing multikey, commit the key-set doc by CID.** New method `did:benten:`. A bare `did:key` is the signing-only degenerate case.

**Why not pure HASH (DID = CID of key-set doc).** Purest Inv-15, tiny DID string (~60 chars) — but it breaks **zero-I/O resolve everywhere**. The UCAN chain-walk resolves the issuer signing key inline, per link, up to depth 32, *before* verifying the signature (`ucan.rs:695-698`: `Did::from_string_for_test_fixture(iss).resolve()`); rotation-verify does the same (`did_rotation.rs:278-282`); sender-origin-auth does the same (`layer_c.rs:295-296`). A hash-only DID forces the key-set document to travel *with every UCAN token* (≈2 KB signing key × 32 links ≈ 64 KB per chain) and to be present at every verify. **This is the "genuine KILLER in the resolve rework" the brief warns about.** Pure HASH triggers it. → REJECTED.

**Why not pure EMBED (multi-key did:key carrying every key inline).** Fully self-certifying, but the DID string carries the ML-DSA (1952 B) + ML-KEM (1184 B) + Ed25519 (32 B) + X25519 (32 B) ≈ 3200 B → ~4400 base58 chars, appearing in *every* `iss`/`aud`/`sender_did`/`prev_did`/`device_did`. The UCAN hot path would carry 1184 B of ML-KEM key **it never uses** (UCAN needs only the signing key), and `bs58::decode` is O(N²) (`did.rs:206`, the reason `MAX_DID_KEY_STRING_LEN` exists at `did.rs:118`). → REJECTED for bloat + carrying-KEM-bytes-the-authority-path-never-touches.

**Why HYBRID wins (the decisive argument).** Of the four DID consumers, **three need only the signing key** (UCAN walk, rotation-verify, sender-origin-auth) and all three are zero-I/O today; **only one needs the KEM key** (Layer-C seal — a cold, once-per-send path). So: **keep the signing key embedded in the DID string (zero-I/O, ZERO rework to the UCAN hot path), and bind the KEM key by committing the key-set doc CID in the DID string.** The doc travels only where the KEM key is needed (Layer-C seal/open), never on the authority path. DID string stays ≈ today's hybrid did:key size (no growth). GAP-KDB closes. Bare did:key becomes the exact degenerate case.

### 1.1 Exact DID string shape

New method `did:benten:` (a distinct method makes "this DID commits my full key-set" lexically un-confusable with a signing-only did:key; did:key stays byte-identical → zero migration for the classical world):

```
did:benten:z<base58btc(
    varint(0x1211) ‖ mldsaPK(1952)              // ML-DSA-65 signing component  (registered mldsa-65-pub)
  ‖ varint(0xed)   ‖ ed25519PK(32)              // Ed25519 signing component    (registered ed25519-pub)
  ‖ varint(0x??KS) ‖ keysetDocCID(36)           // BENTEN_KEYSET_COMMIT framing ‖ 0x01 0x71 0x1e 0x20 ‖ BLAKE3-256(canonical(keyset_doc))
)>
```

- The two signing components are byte-identical to the existing hybrid `did:key` payload (`did.rs:258-285`, ML-DSA-first). **`resolve_hybrid` logic is reused verbatim** for the signing half.
- `keysetDocCID(36)` is the standard Benten CIDv1 `[0x01,0x71,0x1e,0x20, <32-byte BLAKE3>]` (`benten-core/lib.rs:272,459`) of the canonical key-set doc — the KEM commitment.
- `0x??KS` = `BENTEN_KEYSET_COMMIT`, a **Benten-owned structural framing codepoint** reserved in `ReservedCodepoint` (`codepoint.rs:371`). CLAUDE.md #5 permits Benten to own *structural framing* tags (vs component *algorithm* IDs, which stay registered — 0x1211/0xed/0x120c/0xec all reference multiformats). It exists so the decoder is unambiguous rather than relying on "0x01 isn't a multikey varint."
- Decoded payload ≈ 2+1952+2+32+2+36 = **2026 B → ~2762 base58 chars.** Keep `MAX_DID_KEY_STRING_LEN = 4096` (`did.rs:118`) — ample headroom, no valid resolution changes; F2 DoS defense intact.
- **Injectivity (Row-D-13 discipline):** every component length is fixed by its leading varint code, so decode is exact — consume 0x1211+1952, consume 0xed+32, consume 0x??KS+36, **reject trailing bytes** (the exact `DidError::HybridTrailingBytes` discipline at `did.rs:376-380`). No hand-rolled length prefix needed because the self-describing framing already fixes every length.

### 1.2 Key-set document schema (DAG-CBOR, canonical + injective)

```
KeySetDocument  — canonical DAG-CBOR map (sorted keys, definite lengths; injective by construction)
{
  "v":      1,                    // u16  key-set-doc format version (v1-beta freeze = 1)
  "sig":    <bytes>,              // signing multikey:  varint(0x1211)‖mldsaPK(1952) ‖ varint(0xed)‖ed25519PK(32)
                                  //   — byte-identical to the DID's embedded signing components
  "kem":    <bytes>,              // KEM multikey:      varint(0x120c)‖mlkem768EK(1184) ‖ varint(0xec)‖x25519PK(32)   ← 0x120c/0xec WIRED
  "sig_cp": 0x0001,               // u16  signature suite  (LAMPS id-MLDSA65-Ed25519-SHA512)
  "kem_cp": 0x647a,               // u16  cipher suite     (HYBRID_X25519_MLKEM768)
  "dev":    [ <did-string>, … ]   // OPTIONAL committed device-DID list (§4); reserve the key now, populate at Composing
}
```

- Injectivity comes free: DAG-CBOR byte-strings are definite-length (`0x58 <len> …`) and keys are canonically ordered (matches the `CanonicalBytes`/`serde_ipld_dagcbor` discipline in `canonical_bytes.rs`). The internal multikey concatenations are injective because each component length is fixed by its varint code. No `u32-BE` hand-prefix needed *inside* the doc (that discipline, `push_audience` at `layer_c.rs:504-510`, applies to hand-built raw concatenations; DAG-CBOR is strictly stronger).
- The doc CID = `self_describing_cid(BLAKE3-256(canonical_dagcbor(doc)))` — literally the `self_describing_cid` shape at `layer_c.rs:375-380`. **The DID *is* the content-address of the key-set. Pure §3.5s / Inv-15.**
- The embedded signing key is a deliberate, justified **redundancy**: it is the zero-I/O fast-path projection of `doc.sig`, cross-checked against the doc at KEM-resolve time (§2). This redundancy is exactly what buys HYBRID its win over pure-HASH.

---

## 2. RESOLVE MECHANISM — two tiers matching the two key needs

**Tier-1 — signing-key resolve (zero-I/O, the UCAN hot path, ~zero rework):**

```rust
impl Did {
    /// Method-aware signing-key resolve. did:key → resolve()/resolve_hybrid() unchanged.
    /// did:benten → strip the trailing keyset-CID component, decode the two signing
    /// multikeys exactly as resolve_hybrid() does today. NO doc, NO I/O.
    pub fn resolve_signing(&self) -> Result<sig::PublicKey, DidError>;
}
```
The UCAN chain-walk (`validate_chain_inner`, `ucan.rs:679-706`), rotation accept (`did_rotation.rs:278-282`), and sender-origin-auth (`layer_c.rs:295-296`) all switch `resolve()`/`resolve_hybrid()` → `resolve_signing()`. The only new behavior is "if method == benten, drop the trailing 0x??KS+36 component before the signing decode" — **pure string parsing, no doc, no fetch. The killer is neutralized: the authority path stays zero-I/O and needs no key-set document.**

**Tier-2 — KEM-key resolve (doc-required, cold path, Layer-C only):**

```rust
impl Did {
    /// Recover + VERIFY the recipient KEM key from the DID's key-set commitment.
    /// Fail-closed typed-reject on ANY mismatch. Only called at Layer-C seal/open.
    pub fn resolve_kem(&self, keyset_doc: &KeySetDocument) -> Result<RecipientPublic, DidError> {
        // 1. self_describing_cid(BLAKE3(canonical(keyset_doc))) == self.keyset_cid()   (2nd-preimage bound)
        // 2. keyset_doc.sig == self.embedded_signing_multikey()                        (mutual consistency)
        // 3. decode keyset_doc.kem: 0x120c→mlkem768_ek(1184), 0xec→x25519(32)          (0x120c/0xec dispatch)
        // 4. RecipientPublic::from_bytes(0x647a, x25519(32) ‖ mlkem768_ek(1184))        (REORDER — see §5)
    }
}
```
Step 2 (embedded-signing == doc.sig) is load-bearing: it stops a splice that pairs a victim's embedded signing key with an attacker's KEM key — that produces a *different* DID string (different committed CID), and even so, resolve rejects because `doc.sig` (attacker) ≠ embedded (victim). A brand-new attacker DID that is internally consistent (their own sig + their own KEM) grants nothing: UCAN/origin-auth verifies the signing key, which is theirs, so they can't impersonate anyone.

**How the doc travels (the softened residual), priority order:**
1. **In-band with the credential.** The audience's key-set doc rides the same address-book / contact-card / first-contact exchange that today hands over a raw `RecipientPublic` — except now it's *verifiable against the DID*. Primary mechanism.
2. **Cached in the local vault / address book**, keyed by DID, verify-on-store. Immutable once verified (a changed key-set = a changed DID; see §3), so cache-forever is sound.
3. **Fetched by CID over iroh-blobs** (Composing convenience). The doc is a content-addressed blob whose CID *is* the commitment → a fetched doc is self-verifying (BLAKE3(blob)==CID), no PKI. Never on any pre-signature-check path → no F2 DoS regression.

**UCAN chain-walk rework = MINIMAL and doc-free.** UCAN needs only signing keys → Tier-1. The key-set doc does **not** travel with UCANs and is **not** cached for UCAN purposes. The only edits to `validate_chain_inner`: (a) `resolve()` → `resolve_signing()`; (b) the DID parser learns the `did:benten` method. (Integration coupling: today's walk uses `resolve()` = Ed25519-only + 64-byte sigs at `ucan.rs:687-704`; a `did:benten` issuer's embedded signing key is the *composite*, so the verify must dispatch on the sig codepoint against the composite key — see Worry #1.)

---

## 3. ROTATION — genesis + signed key-set-update chain

**Problem:** any key change → new doc → new CID → new DID. RotationLog (`did_rotation.rs`) exists to keep identity stable.

**Design:** identity = the **genesis key-set DID** (immutable, the forever-logical-identity) + a signed **key-set-update chain**. Generalize the existing `RotationAttestation` (`did_rotation.rs:57-67`) — same shape, plus a `genesis` anchor:

```
KeySetRotation  (DAG-CBOR; signature excluded from canonical bytes per canonical_bytes.rs SigInput hygiene)
{
  "v":            1,
  "prev":         <did-string>,   // key-set DID being superseded
  "next":         <did-string>,   // new key-set DID
  "genesis":      <did-string>,   // STABLE logical identity — invariant across the whole chain
  "superseded_at": u64,           // HLC epoch (strict-monotonic per prev, as today)
  "signature":    <bytes>,        // HYBRID signature by prev's SIGNING key over canonical(v,prev,next,genesis,superseded_at)
}
```
The existing `did:key` `RotationAttestation` is the degenerate single-hop case (`genesis == prev`).

**RotationLog rework (minimal, and zero-I/O preserved):**
- `verify_signature_with` resolves `prev`'s signing key via `resolve_signing()` **from the DID string** — zero I/O, exactly as `accept_rotation_event` resolves `previous_did` today (`did_rotation.rs:278-282`). **This is a second reason HYBRID matters: the rotation signature verifies against the prior DID's embedded signing key with no doc fetch.** Pure-HASH would break this too.
- Swap the 64-byte Ed25519 verify (`did_rotation.rs:120-132`) for the codepoint-dispatched hybrid verify. HLC-strict-monotonic + verbatim-replay (`did_rotation.rs:284-321`) unchanged. Add: `next.genesis == log's genesis for that identity` (a rotation cannot re-parent to a different genesis).
- `is_superseded(did)` (`did_rotation.rs:329-334`) unchanged in shape; `current_keyset(genesis) -> Did` walks prev→next to the tip.

**UCAN survival across rotation — two binding modes, both preserved:**
- **`aud = genesis DID` (stable-identity binding):** survives rotation. Genesis is invariant; the holder's current key-set is found by chain-walk; the holder signs new tokens with the current key-set's signing key (`iss = current keyset DID`, which chains to genesis). **Recommended default** for long-lived grants.
- **`aud = specific keyset DID` (pinned):** after rotation, that key-set DID appears as `prev` → `is_superseded` fires → the existing `validate_chain_with_rotation_log` consultation (`chain_authority.rs:96-101`) rejects it. **This is byte-for-byte today's "rotation revokes old UCANs" semantics — preserved.**

So planned key-refresh uses genesis-bound audiences (survive); compromise-response uses the pinned-supersession revocation that already exists, plus (for genesis-bound grants) the self-anchored content-CID UCAN revoke seam (`chain_authority` / `UCANBackend::revoke`). See Worry #2 for the subtlety.

---

## 4. MULTI-DEVICE composition

Devices stay attested sub-identities via the **existing** `DeviceAttestation{device_did, parent_did, envelope, signature}` (`device_attestation.rs:132-155`), generalized to key-set DIDs:

- **Each device = its own `did:benten`** (own signing keypair + own KEM keypair in its own key-set doc). Per-device KEM keys are preserved (not defeated by a shared key-set).
- **Attested under the user:** `DeviceAttestation{ device_did = device's did:benten, parent_did = user genesis/keyset DID, envelope, signature-by-user-signing-key }` — the existing `issue*` path (`device_attestation.rs:177-233`), zero new machinery.
- **The user key-set doc's `dev: [device_did,…]` field** commits the authoritative device list (each entry a device `did:benten` whose KEM key is self-committed). Add/remove a device = a key-set rotation (§3, new doc with updated `dev`).

**Sender picks the right device KEM key = a group send.** Sending to a user = a `0x6520` multi-stanza send to the user's committed device set: resolve the user DID → `dev` list → each device DID → `resolve_kem(device_doc)` → wrap the CEK to each device's verified KEM key. **This is exactly the existing multi-recipient group path** (`seal_group_multi`, `layer_c.rs:1407`); the "audience" is the device set. No new selection engine at v1-beta (fallback/offline-device policy refinements → Composing).

---

## 5. GAP-KDB SEAL-API CLOSURE + 0x120c wiring

Replace the substitutable `(recipient_pub, audience_did)` pair with **one typestate binding** that cannot be constructed with an un-committed KEM key:

```rust
/// A recipient whose KEM key is PROVEN committed by its DID. There is no
/// public constructor that pairs an arbitrary kem_pub with an audience DID.
pub struct RecipientBinding {
    audience_did: Did,
    kem_pub: RecipientPublic,   // resolved+verified from the key-set doc against audience_did
}
impl RecipientBinding {
    /// The ONLY constructor. Fail-closed typed-reject on commitment mismatch.
    pub fn resolve(audience_did: &Did, keyset_doc: &KeySetDocument)
        -> Result<Self, RecipientBindingError>;   // calls Did::resolve_kem (§2)
    // A bare did:key commits NO KEM key → resolve() returns NoKemCommitment.
    // (The Shape-A fallback — did:key + a separately-signed key binding — would live behind THIS door.)
}
```

New seal signature (single-recipient sealed-sender; compare `layer_c.rs:947-955`):

```rust
pub fn seal_sealed_sender(
    recipient: &RecipientBinding,          // WAS: (recipient_pub: &RecipientPublic, audience_did: &AudienceDid)
    sender_did: &SenderDid,
    sender_kp: &sig::Keypair,
    body_cid: &BodyCidDigest,
    recipient_key_generation: u32,
    plaintext: &[u8],
) -> EncryptedEnvelope
// internally: audience_did := recipient.audience_did bytes; recipient_pub := recipient.kem_pub
```

The two independently-choosable params collapse into one binding where `kem_pub` is cryptographically bound to `audience_did`. An active attacker substituting the KEM key must produce a key-set doc that hashes to `audience_did`'s committed CID — a BLAKE3-256 2nd-preimage (infeasible). **GAP-KDB closed by construction.** Group paths (`seal_group_multi` `layer_c.rs:1407`, `seal_membership_set_group` `layer_c.rs:2336`) take `&[RecipientBinding]` — closing the *same* class at the roster level (the `audience_set_commitment` roster and the `&[RecipientPublic]` list, today separate at `layer_c.rs:390-401` vs `1263`, become one coupled list).

**0x120c/0xec wiring (`did.rs:87-97`, `CRYPTO-CODEPOINTS.md:167-168`).** The doc `kem` field = `varint(0x120c)‖mlkem768_ek(1184) ‖ varint(0xec)‖x25519(32)` (PQ-first, mirroring the ML-DSA-first sig convention). `decode_kem_multikey` dispatches 0x120c → consume the 1184-B ML-KEM EK; 0xec → consume the 32-B X25519 key; **REORDER to `x25519(32) ‖ mlkem768_ek(1184)`** for `RecipientPublic::from_bytes(0x647a, …)` — because `from_bytes` splits x25519-first (`cipher_suite.rs:837`, `split_at(X25519_PUBLIC_LEN)`) while the multikey is PQ-first. This reorder is a concrete, frozen wire detail (the critics will check it). This retires the reserved-private `HYBRID_KEM_MULTICODEC = 0xf0` (`did.rs:97`, #5-risky) in favor of the registered components.

**Mint a new invariant (Inv-23, next free after Inv-22 at `did.rs:120`):** *"A Layer-C seal's KEM key is committed by its audience DID"* — the type-level guarantee `RecipientBinding` enforces. This is the freeze-surface analogue of Inv-15.

---

## 6. MIGRATION / bare-did:key degenerate / backward-compat

- **Bare `did:key` = signing-only degenerate identity.** Valid for UCAN issuers / plugins that never *receive* Drops. Commits no KEM key → `RecipientBinding::resolve` rejects it with `NoKemCommitment`. Honest degradation: signing-only principals need no KEM key; Drop recipients MUST be `did:benten`. `did:key` bytes are unchanged → **zero migration for the classical/authority world.**
- **Additive, no wire-break (crypto-agility #5).** `did:benten` is a new method + new codepoints; existing UCANs, did:key rotations, and already-sealed Drop envelopes are untouched. The Drop *wire format* is unchanged — `audience_did` is still length-prefixed opaque bytes in the AAD (`push_audience`, `layer_c.rs:504-510`); a longer `did:benten` audience just fills that field. **No re-encryption of existing content.**
- **Upgrade path (no re-issue):** an existing `did:key` user mints a `did:benten` by publishing a key-set doc `{existing signing key, NEW kem key}` and issuing a `KeySetRotation{ prev = did:key, next = did:benten, genesis = did:key }` signed by the old key. Genesis-bound audiences re-resolve to the `did:benten` tip; the KEM-carrying identity is chained to the old signing identity by the old key's signature.

---

## 7. v1-beta FREEZE-SURFACE vs Composing-additive

**FREEZE NOW (Phase-4-Meta-Core — locks the v1 interface):**
- The `did:benten:` method + method-specific-id byte layout: signing-multikey ‖ `BENTEN_KEYSET_COMMIT` framing ‖ 36-B CIDv1; component order; trailing-bytes-reject.
- `KeySetDocument` v=1 DAG-CBOR schema: field names + canonical key order, `sig`/`kem` multikey layouts (incl. 0x120c/0xec PQ-first + the RecipientPublic reorder rule), `sig_cp`=0x0001, `kem_cp`=0x647a. (Reserve the `dev` key now; populating it is Composing.)
- `KeySetRotation` schema incl. `genesis`, its canonical-bytes / signature domain.
- The seal-API **coupling invariant** Inv-23 (KEM key committed by audience DID) — the *invariant* freezes; the exact `RecipientBinding` type name may evolve.
- Commitment algorithm: BLAKE3-256 over canonical DAG-CBOR → self-describing CIDv1.
- `BENTEN_KEYSET_COMMIT` reserved codepoint in `ReservedCodepoint` (`codepoint.rs:371`).
- **Compromise #67** (next free; SECURITY-POSTURE tops out at #66) — the first-contact/TOFU residual (§8).

**DEFER to Composing (additive, no wire-break):**
- Key-set-doc **distribution/caching transport** (contact-card UX, the iroh-blobs fetch-by-CID convenience). Doc *format* freezes; how it travels is UX-coupled → Composing (wire-vs-UX split rule).
- `dev` committed-list ergonomics + device-link UX (QR/approval).
- Rotation-vs-revocation "reason"/compromise-response UX.
- Multi-device KEM-selection policy (offline/fallback ordering) beyond the basic group-send-to-committed-set.
- `RecoveryHook` / identity-recovery (already Composing per `CRYPTO-CODEPOINTS.md:68`).

---

## 8. Compromise #67 — the honest residual (do NOT pretend to eliminate)

Shape-B closes *key-substitution-given-the-DID* (an active attacker can no longer swap the KEM key under a known DID — it's a 2nd-preimage). It does **not** close *DID-authenticity-at-first-contact*: whoever controls the channel where you **first learn** "Alice's DID" can hand you their own `did:benten`. Shape-B reduces the trust window from "swap the key at any time" to "swap the identity at first contact only" — bind-once instead of continuous-trust, the same posture as Signal safety numbers / MLS / PGP. **Compromise #67 = "first-contact/TOFU DID-authenticity bootstrapping (out-of-band verification of the initial DID↔principal binding is the user's responsibility; Benten provides content-committed key-sets that make post-first-contact key substitution infeasible but cannot authenticate the first contact)."**

---

## Top-3 worry points (my own)

1. **UCAN ↔ hybrid-signature coupling (biggest integration risk).** Today's chain-walk uses `resolve()` (Ed25519-only) + 64-byte sigs (`ucan.rs:687-704`); a `did:benten` embedded signing key is the *composite*. Wiring benten issuers into the walk requires verifying the composite (codepoint-dispatched), NOT extracting only the Ed25519 half — the latter is a **silent PQ-downgrade on the authority path**. This is a real change to `validate_chain_inner` + the sig-suite dispatch, coupled to the existing swap-matrix. Mis-wire = either two parallel resolve paths or a PQ-strip. Must be designed as one codepoint-dispatched verify, not bolted on.

2. **Rotation-vs-revocation semantics under genesis re-resolution.** Genesis-bound audiences *survive* rotation (planned refresh), but that must not become a fail-open on *compromise*: rotating away from a compromised key kills pinned-DID grants (via `is_superseded`), and genesis-bound grants are safe only because the attacker can't produce the *new tip's* signatures — yet a precise statement of "what the old-key holder can still do to a genesis-bound grant" is subtle, and getting it wrong reopens a fail-open. The compromise path must additionally drive the self-anchored UCAN revoke for genesis-bound grants. Needs an explicit, tested state-machine.

3. **Key-set-doc availability at send-time (the softened, not eliminated, killer).** HYBRID removes the doc dependency from the authority path, but to send a first Drop to an **uncached, brand-new** contact you still need their key-set doc (carried/cached/fetched). Offline + uncached + never-contacted = can't send. This is strictly better than today (you'd otherwise hold an *unverified* RecipientPublic) and is the same first-contact problem as everyone else — but it is a genuine ergonomic cost, and it is the operational face of Compromise #67. If reviewers find the in-band/cache/fetch tiers insufficient for a required offline-first-send use case, that is the most likely place a reviewer argues for the Shape-A fallback.

---

**Files grounding this spec (all absolute, ref `35b27b1c` via `/Users/benwork/Documents/benten-wt-r9base`):** `crates/benten-id/src/did.rs` (DID model, resolve_hybrid, 0x120c mention, F2 cap), `crates/benten-id/src/did_rotation.rs` (RotationAttestation + accept_rotation_event + zero-I/O resolve), `crates/benten-id/src/ucan.rs` (validate_chain_inner zero-I/O resolve at :695-698), `crates/benten-caps/src/chain_authority.rs` (is_superseded consultation :96-101), `crates/benten-drop/src/layer_c.rs` (seal_sealed_sender GAP-KDB site :947, sender-origin-auth resolve_hybrid :295-296, audience_set_commitment :390-401, push_audience :504-510), `crates/benten-crypto-suite/src/cipher_suite.rs` (RecipientPublic layout :770-869), `crates/benten-crypto-suite/src/{mlkem.rs,sizes.rs}` (1184/2400/1952 sizes), `crates/benten-id/src/device_attestation.rs` (DeviceAttestation :132-155), `crates/benten-core/src/lib.rs` (CIDv1 form :272/:459), `crates/benten-id/src/canonical_bytes.rs` (DAG-CBOR canonical/injective), `docs/CRYPTO-CODEPOINTS.md` (:167-168 0x120c/0xec, :185 hybrid did:key), `docs/SECURITY-PROOFS.md` §4.1 (:92-102 recipient-key premise), `docs/THREAT-MODEL.md` §2 (:104-110 rung 4). Compromise numbering tops at #66 → #67 free; invariants top at Inv-22 → Inv-23 free.

---

## R1 CONVERGENCE VERDICT + BEN DECISION BRIEF

# R1 DESIGN RECORD + BEN DECISION BRIEF — Shape-B Identity Model (GAP-KDB closure)

*Synthesis of 5 adversarial lenses (crypto-soundness · identity-resolve · rotation-multidevice · wire-freeze-scope · threat-residual-migration), ref `35b27b1c`. R1 council for the v1-beta interface freeze.*

---

## 1. VERDICT — SHAPE-B IS SOUND TO FREEZE. No killer. A stays a documented fallback and is NOT triggered.

**All five lenses returned FIXABLE-CONCERN. None found a killer. None forces the A-now-reserve-B fallback.** I looked specifically for a buried killer because you want B and I owe you the honest "no" if one exists — there isn't one. The load-bearing risk the R0 exploration flagged (the zero-I/O resolve rework) is *neutralized by the design's own central call* (HYBRID embed-signing), confirmed independently by identity-resolve: "HYBRID kills the killer." The commitment binding reduces cleanly to BLAKE3-256 2nd-preimage resistance (crypto-soundness F1: "the reduction holds"), and GAP-KDB is closed *by construction* — the `RecipientBinding` typestate doesn't just add a check, it **deletes the vulnerable API** (threat-residual §A: no code path can seal to an unbound `(KEM, DID)` pair; an attacker can't even downgrade a did:benten recipient back to the old path — it fails closed).

**Where a reviewer could still argue for A — and why none of it rises to killer:** the three genuine pro-A pressure points are (i) device-churn couples to identity-rotation under B (rotation-multidevice F5, the strongest pro-A signal), (ii) genesis-survival is *unbuilt and asserted-as-settled* in the spec's §3 (rotation-multidevice F1), and (iii) offline-first-send to an uncached contact still needs the key-set doc (crypto-soundness Worry #3). **The decisive synthesis fact: all three live entirely in the parts of the spec we should DEFER to Composing** (rotation-chain + multi-device). None touches GAP-KDB closure. So shrinking the freeze (see §2) doesn't dodge them — it correctly homes them where they're additive and resolvable with eyes open, and it makes the *v1-beta* soundness of B **unambiguous**. B ships.

**The one thing the spec got materially wrong** (flagged by 4 of 5 lenses, must be corrected in the record, but is NOT a killer): §2's claim that the UCAN authority-path rework is "MINIMAL and doc-free / ~zero rework" is **false and self-contradicting** — it contradicts the spec's own Worry #1, which is the correct read. The entire authority path (UCAN walk, rotation verify, device attestation) is hardcoded Ed25519 today. That is **pre-existing v1-beta debt, not a Shape-B invention** (the project's own `sealed-sender-auth-design.md:532-534` already named it "a debt that is coming regardless… the honest price of PQ-everywhere"). Shape-B makes it salient but does not require it to close GAP-KDB — which is exactly why it becomes **Open Fork A** below, not a freeze-blocker.

---

## 2. THE KEY SYNTHESIS INSIGHT — shrink the freeze to the recipient surface

The spec's §7 freeze surface is **larger than necessary to close GAP-KDB**, and three lenses converge hard on this (wire-freeze-scope F1/F6, rotation-multidevice F1/F2, threat-residual §C). GAP-KDB is a **seal-path / recipient-side** gap. Closing it requires only: a DID that commits the recipient KEM key, `resolve_kem` + `RecipientBinding`, and the seal-API change. It touches **none** of the classical authority path.

The spec bundled three heavy, mostly-orthogonal things into the freeze that **are not required to close GAP-KDB and forecloses nothing to defer**:
- **Rotation-chain + genesis-survival (§3)** — and this is the spec's *least-built* section: genesis-UCAN-survival is **unbuilt and untested** (the chain-link check is strict byte-equality `ct_signature_eq(parent_aud, child_iss)`, ucan.rs:725-733; "chains to genesis" does not exist). Freezing it now would freeze an unbuilt property AND newly introduce two fail-open surfaces (crypto-soundness F2 rotation-fork, rotation-multidevice F6 mutable-current-tip).
- **Multi-device / `dev` field (§4)** — a CID-committed doc can't "reserve a key"; `{…,dev}` is a *different* CID = different identity (wire-freeze-scope F5).
- **did:benten-as-UCAN-issuer** — forces the Ed25519→hybrid authority-path migration, which is additive-later via method-dispatch (wire-freeze-scope F2).

**Deferring all three (a) closes GAP-KDB cleanly, (b) avoids freezing anything unbuilt, (c) avoids *introducing* the F2/F6 fail-open surfaces at the freeze, and (d) forecloses nothing** (wire-freeze-scope F6 foreclosure-check: PASSES). This is the elegant minimal shape. I am ratifying **Scope-Min** as the recommended freeze and surfacing the one real up-scope decision (authority path) as a fork.

---

## 3. RATIFIED B DESIGN DECISIONS

### 3.1 HASH-vs-EMBED — the load-bearing call: **HYBRID (embed the signing multikey; commit the key-set doc by CID). RATIFIED.**

Unanimous across lenses; this is the decision that neutralizes the killer. Reasoning, grounded:

- **Pure HASH (DID = CID of doc) is the killer the brief warned about.** The UCAN chain-walk resolves each issuer's signing key **inline, per link, zero-I/O, before signature verify** (ucan.rs:695-698); rotation-verify and sender-origin-auth do the same. Hash-only forces the key-set doc to travel with every token (≈2 KB × up to 32 links) and be present at every verify. **REJECTED.**
- **Pure EMBED (all keys inline) bloats every `iss`/`aud`/`sender_did` with 1184 B of ML-KEM key the authority path never touches**, and `bs58::decode` is O(N²). **REJECTED.**
- **HYBRID wins because of an asymmetry in the four DID consumers:** three (UCAN walk, rotation-verify, sender-origin-auth) need **only the signing key** and are zero-I/O today; **only one** (Layer-C seal) needs the KEM key, on a cold once-per-send path. So keep the signing key **in the DID string** (zero-I/O, zero authority-hot-path rework) and bind the KEM key by **committing the key-set doc CID in the DID string**. The doc travels only where the KEM key is needed. GAP-KDB closes; the DID string stays ≈ today's hybrid did:key size; a bare did:key is the exact signing-only degenerate case.

### 3.2 Ratified corrections to the spec (from the critics — these are nails, not forks)

| # | Correction | Source |
|---|---|---|
| **C1** | **DROP `BENTEN_KEYSET_COMMIT` entirely.** It was mis-homed in the cipher-suite `ReservedCodepoint` enum (whose `resolve()` always emits `UnsupportedAlgorithm::CipherSuite` — a category error) AND it's *unnecessary*: after the fixed-length signing multikeys, the next bytes are the CIDv1 self-describing prefix `[0x01,0x71,0x1e,0x20]`, which already disambiguates the decode. No invented Benten-private varint = strictly more #5-clean. | crypto-soundness, wire-freeze-scope F3 |
| **C2** | **Make the doc `kem` field X25519-first** = `varint(0xec)‖x25519(32) ‖ varint(0x120c)‖mlkem768_ek(1184)`, matching the *already-frozen* `RecipientPublic::to_bytes()` order (cipher_suite.rs:792/837). **Deletes the §5 REORDER step** (a frozen foot-gun) while still wiring 0x120c/0xec self-describingly. Cross-check `kem_cp (0x647a) ⟺ component set {0xec,0x120c}`, fail-closed on disagreement (defeats algorithm-confusion). | crypto-soundness F3, wire-freeze-scope F4 |
| **C3** | **`resolve_kem` verifies over a re-canonicalized / strict-canonical decode of the received doc**, never a raw-byte compare — Row-D-13 injectivity discipline (the exact `HybridTrailingBytes` posture at did.rs:376-380). | crypto-soundness F1 |
| **C4** | **`RecipientBinding`: private fields, `resolve()` is the SOLE constructor, no `From`, no `pub` field, no `_for_test` escape, and NO `(kem_pub, did)` fallback door at v1-beta.** This is what makes the anti-downgrade property real (the Shape-A fallback constructor must not ship, or it re-opens GAP-KDB). | threat-residual §A |
| **C5** | **PQ floor:** Drop-receiving key-sets MUST commit `0x647a`. `0x6400` (classical-only X25519 — a *live, resolvable* suite at codepoint.rs:335) is a below-floor, non-PQ, HNDL-exposed commitment; never silently sealed-to (per baked-in #5/#18). | crypto-soundness F3 |
| **C6** | **`resolve_signing` for did:benten is a NEW function**, not a verbatim `resolve_hybrid` reuse: it must *expect* (not reject) the trailing 36-B CID, and peek the leading multicodec (`0xed01`→classical, `0x1211`→composite) rather than dispatch on method string alone (a hybrid did:key is *also* method `did:key`). | identity-resolve FS-2, wire-freeze-scope F3 |
| **C7** | **F2 DoS cap extends to did:benten:** `length_pre_check` must gate the O(N²) `bs58::decode` under the new method's prefix (the `iss` is attacker-controlled per link). Keep `MAX_DID_KEY_STRING_LEN = 4096` (payload ≈2026 B → ≈2762 chars, ample headroom). | crypto-soundness, threat-residual |
| **C8** | **`recipient_key_generation` ⟷ committed-key-set precedence pinned** (it's an already-frozen AAD field, layer_c.rs:413/480): the committed key-set identifies *which* KEM keypair; `recipient_key_generation` stays the intra-keypair freshness/nonce-cache index; fail-closed on disagreement. R2 nails exact semantics. | threat-residual FSC-3 |
| **C9** | **Group path = REPLACE the pubkey-derived placeholder roster** (layer_c.rs:1093-1111 currently *fabricates* audience DIDs by hashing the KEM keys → zero binding), not "couple two lists." Derive both `audience_set_commitment` and wrap-targets from one `&[RecipientBinding]` slice. | threat-residual FSC-4, crypto-soundness |

---

## 4. FREEZE-SURFACE (locks now) vs COMPOSING-ADDITIVE

### FREEZE NOW — v1-beta-critical (Phase-4-Meta-Core, locks the v1 interface)

1. **`did:benten:` method + byte layout**: `[0x1211‖mldsa(1952)] ‖ [0xed‖ed25519(32)] ‖ [CIDv1 0x01,0x71,0x1e,0x20 ‖ blake3(32)]`, trailing-reject. **No framing byte (C1).**
2. **`KeySetDocument` v=1 DAG-CBOR**: exactly `{v, sig, kem, sig_cp=0x0001, kem_cp=0x647a}`. `kem` X25519-first with 0x120c/0xec wired + kem_cp cross-check (C2). **No `dev` field.**
3. **`resolve_signing`** (method-aware, composite arm — used on the Layer-C did:benten **send** path; C6) + **`resolve_kem`** (strict-canonical decode C3 · CID 2nd-preimage check · `doc.sig == embedded_signing` cross-check · kem_cp⟺components C2 · PQ floor C5).
4. **`RecipientBinding`** (sole constructor, private fields, no fallback door — C4) + **new seal API** (single + group `&[RecipientBinding]`, roster-replacement C9). Retires the #5-risky `HYBRID_KEM_MULTICODEC = 0xf0` in favor of registered 0x120c/0xec.
5. **Inv-23** — "a Layer-C seal's KEM key is committed by its audience DID" (freeze the *invariant*; the `RecipientBinding` type-name may evolve).
6. **Commitment algorithm** — BLAKE3-256 over canonical DAG-CBOR → self-describing CIDv1.
7. **`recipient_key_generation` ⟷ key-set precedence rule** (C8).
8. **F2 DoS cap wired for did:benten** (C7).
9. **Compromise #67** (§5 below).
10. **Retire ratified decision #1073's did:key-only-preservation clause** — did:benten now exists in resolve at v1 (your Shape-B ratification supersedes it; the record must say so explicitly).

### DEFER to Composing — additive, no wire-break, forecloses nothing

- **`KeySetRotation` schema + genesis-chain + rotation-survival** — and its must-nails travel WITH it so they aren't lost: **F2 fork=freeze single-successor invariant** (crypto-soundness), **F6 fail-closed genesis fork-resolution** (rotation-multidevice), **augment-vs-supersede distinction** so a KEM-only refresh doesn't revoke signing authority (threat-residual FSC-1/FSC-2), genesis-normalization's **CONSOLIDATE-line placement** (rotation-multidevice F2), the **§4.26 rotation-log-rehydration** dependency (currently `#[ignore]`'d), **domain-separation on the KeySetRotation SigInput** (crypto-soundness F5), and the **schema-sufficiency** question (does proof-of-chain need to ride the token for zero-I/O genesis→tip resolution — decide BEFORE freezing the rotation schema). Re-home #1069/#1076.
- **`dev` field + multi-device KEM selection + device-DID/PeerId/iroh-EndpointId un-conflation** (handshake.rs:582/974) + device-attestation hybrid-sig migration.
- **Key-set-doc distribution/caching transport** (contact-card UX, iroh-blobs fetch-by-CID — the *format* freezes; how it travels is UX-coupled).
- **RecoveryHook / identity-recovery.**
- **THREAT-MODEL addition:** the seal-path revocation-reach gap — rotating away from a compromised key does NOT stop inbound Drops from any sender holding the old key-set doc (rotation-multidevice F7), distinct from Compromise #67.

---

## 5. THE HONEST RESIDUAL — Compromise #67 (do NOT pretend to eliminate)

Shape-B closes *key-substitution-given-a-known-DID* (a BLAKE3-256 2nd-preimage). It does **not** close *DID-authenticity-at-first-contact*. It reduces the confidentiality trust window from **continuous** (swap the address-book KEM key at any time) to **bind-once** (swap the DID only at first contact) — the identical posture to Signal safety numbers / MLS / PGP fingerprints, which all share this residual. **A has the exact same residual — no reviewer's push for A can be framed as "eliminating" it.** Numbering confirmed: SECURITY-POSTURE tops at #66 → **#67 free**; INVARIANT-COVERAGE tops at Inv-22 → **Inv-23 free**.

> **Compromise #67 — First-contact / TOFU DID-authenticity bootstrap.** Shape-B content-committed key-sets make *post-first-contact* key substitution infeasible (a BLAKE3-256 2nd-preimage over the canonical DAG-CBOR key-set), reducing the confidentiality trust window from *continuous* (any address-book KEM-key swap, at any time) to *bind-once* (substitute the DID only at the moment a sender first learns "principal ↔ DID"). Authenticating that initial DID↔principal binding is out-of-band and the user's responsibility; Benten provides no PKI/CA for it. Same residual and posture as Signal safety numbers, MLS, and PGP fingerprints. Not eliminated — a party controlling the first-contact channel can hand you its own self-consistent did:benten, which then verifies cleanly.

---

## 6. OPEN FORKS — genuinely need your call

### FORK A — does the authority-path Ed25519→hybrid migration land in the v1-beta window, or defer to Composing? *(the biggest engineering lift; my #1 surface)*

**What it is:** the entire UCAN authority path (chain-walk, rotation-verify, device-attestation) is hardcoded Ed25519 today (`[u8;64]` sig at ucan.rs:687; Ed25519-only `resolve()` at :697). A did:benten embeds a **LAMPS composite** signing key (≈3373-B sig). So did:benten cannot be a **UCAN issuer** until the walk becomes codepoint-dispatched hybrid-verify (~13 sites + type cascade). It **can** be a full **Drop send+receive** identity at v1-beta without this (Layer-C sender-auth is already hybrid-capable; needs only the method-aware `resolve_signing`).

**Key facts for your decision:** (a) It is **additive** — dispatch on the `iss` DID's multicodec; existing 64-byte did:key tokens verify unchanged; new did:benten tokens branch. **No token wire-break, so it does NOT gate the freeze either way.** (b) It is **pre-existing debt** — the "v1-beta signature default = hybrid" (CLAUDE.md #5) is *already* not true on the authority path, independent of Shape-B. (c) The failure mode to design against: verifying only the Ed25519 half of a composite = a **silent PQ-strip on the authority path** — must be **one** codepoint-dispatched verify, not a bolt-on, with a regression test pinning that a did:benten issuer's Ed25519-only-signed token *rejects*.

**My prediction (surface, don't decide):** I lean **do it in the beta window, before the v1-beta tag** — because freezing a v1 interface whose "hybrid default" doesn't reach the authority surface is internally inconsistent, it's a one-time additive migration cheapest to do while the crypto is fresh, and a "v1 identity" that can't issue caps is an odd half-shape. But it's the largest lift and it's genuinely deferrable (Scope-Min ships did:benten as a Drop identity now, UCAN-issuer later), so it's a real scope/appetite call and I won't make it as-you. If you defer it, did:benten is a **recipient/sender** identity through beta while UCANs still issue from did:key — a *transient* dual-use (not a breaking migration; the same did:benten string becomes an issuer when the walk lands).

### FORK B — confirm rotation-survival + multi-device defer to Composing *(reverses the spec's §7; touches your stated B-rationale)*

Your B-ratification named rotation-stability as a rationale ("RotationLog exists to keep identity STABLE across key changes"). **Scope-Min means v1-beta did:benten has NO rotation-survival: a key change = a new identity, revocation-only — exactly today's behavior.** All five lenses support deferring (it's unbuilt today, and freezing it would introduce the F2/F6 fail-opens). I recommend deferring, but because it touches your explicit rationale I want your explicit sign-off rather than treating it as within night-shift auth. **Prediction:** you'll confirm the defer once you see that (i) it's unbuilt regardless, (ii) deferring *avoids introducing* two fail-open surfaces, and (iii) it forecloses nothing — but the genesis-chain schema must be designed correctly when it lands (schema-sufficiency + fork=freeze), so it's "defer, with named must-nails," not "defer and forget."

### FORK C — KEM field: self-describing multikey (with cross-checked `kem_cp`) vs raw `RecipientPublic::to_bytes` dispatched by `kem_cp` *(minor; can be an R2 nail)*

The brief wants 0x120c "wired." C2 wires it self-describingly (X25519-first, no reorder) **with** `kem_cp` as a cross-checked redundant field. The lighter alternative (wire-freeze-scope F4-i) is raw `RecipientPublic::to_bytes` dispatched by `kem_cp` alone — smaller, no redundant varints. **My recommendation: C2 (wire 0x120c, cross-check kem_cp), per the brief's explicit "wire 0x120c" intent** — the redundancy is cheap and the cross-check is a real algorithm-confusion defense. Flagging only because it's a frozen wire detail; if you'd rather minimize the frozen surface, F4-i is defensible. Default to C2 unless you say otherwise — this one is within R2's remit to finalize.

---

## 7. MINI-ADDL R2→R5 SCOPE + v1-beta-critical-path

This is arch-scope work → full ADDL from here (per `feedback_addl_pipeline_full_observance`). Scope is the **Scope-Min freeze surface** (+ Fork A's authority-path workstream if you pull it in).

- **R2 (test-landscape):** families over the freeze surface — did:benten codec round-trips + injectivity/trailing-reject; `resolve_kem` fail-closed matrix (bad CID / spliced doc.sig / kem_cp-vs-components mismatch / 0x6400-floor-violation / non-canonical encoding); `RecipientBinding` sole-constructor + no-fallback-door; **GAP-KDB active-substitution regression** (the flagship pin — substituted KEM key must fail closed); group-roster replacement; `recipient_key_generation` precedence; F2 DoS cap under did:benten. Finalizes Fork C.
- **R3 (red-phase test-writers):** canary-first slicing (below).
- **R4 (test review) → R5 (impl-to-green) → R4b → R6** phase-close council.

**v1-beta-critical-path (R5 canary-first ordering):**
1. **CANARY — `benten-id`:** did:benten codec + `KeySetDocument` + `resolve_signing`/`resolve_kem`. *Everything downstream depends on the identity + resolver.*
2. **`benten-crypto-suite`:** `kem` field ↔ `RecipientPublic` wiring (X25519-first, C2) + 0x120c/0xec.
3. **`benten-drop` Layer-C:** `RecipientBinding` + seal API (single **then** group roster-replacement C9); the GAP-KDB closure lands here.
4. **Docs + invariants (last group):** Inv-23 pin, Compromise #67, CRYPTO-CODEPOINTS 0x120c-wired, THREAT-MODEL seal-path-revocation-reach note, retire #1073.
5. **(Fork A, if pulled in — parallel workstream):** authority-path one-dispatch hybrid verify (UCAN walk + rotation + device-attestation) with the silent-PQ-strip regression pin + FS-2 classical-from-bytes constructor + FS-1 `MAX_UCAN_ENVELOPE_BYTES` re-sizing for composite chains.

---

## Bottom line

**Ship B.** No killer; A is not triggered. Freeze the **recipient-side GAP-KDB closure** (Scope-Min) with the 9 ratified corrections — this closes the actual security gap, reduces the trust model to Signal/MLS TOFU (Compromise #67), and, by shrinking the freeze, **avoids freezing anything unbuilt and avoids introducing the rotation-fork / mutable-tip fail-opens at the gate**. Two real decisions are yours: **Fork A** (authority-path hybrid migration — beta-window vs Composing; my lean: beta-window, but genuinely your appetite call and it does not gate the freeze) and **Fork B** (confirm rotation/multi-device defer, with named must-nails for when they land). Fork C is a minor wire detail R2 can finalize (default: wire 0x120c + cross-check).