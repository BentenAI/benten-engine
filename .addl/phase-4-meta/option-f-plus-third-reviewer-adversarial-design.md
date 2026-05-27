# Option F+ envelope-layer-unification §6.2-with-Amendments — THIRD reviewer adversarial-design red-team

**Branch:** `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design`
**Reviewer lens:** Senior cryptographer, ADVERSARIAL-DESIGN posture. Explicitly dispatched by Ben to *find a surviving attack* against §6.2-with-Amendments-1+2 (the approved design from the second-opinion review). Belt-and-suspenders contrarian-eye review before treating as load-bearing-final.

**Inputs ground-truth-verified** (every cite below pinned to the SHA shown; I `git show <branch>:<path>` -verified each):

- `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — first reviewer's NO-GO + §6.2 sketch.
- `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` — second-opinion CONCUR-WITH-AMENDMENTS introducing Amendments 1+2.
- `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae` — §6 remote-permission wire shape; §7 device-link wire shape; §14.1 origin Option F+ pitch.
- `phase-4-meta-core/cryptographer-review-bird-of-prey @ 36afe06b` — Inv-15 origin.
- `phase-4-meta-core/encrypt-to-recipient-review-cryptographer @ 791c8d17` — Option B Conditional-GO baseline.
- `docs/INVARIANT-COVERAGE.md` Inv-15 (registered, partial-enforcement).
- `docs/SECURITY-POSTURE.md` Compromise #30 (PQ-impl-audit-maturity) + Compromise #31 (Drop-bundle revocation reach).

**Authority:** ADVISORY. Final decision rests with Ben. Downstream DISAGREE-WITH-EXPLANATION first-class per `feedback_review_finding_ground_truth_verify`. The two prior reviewers concurred on a design; my job is *trying to break it*.

**Stance disclosed up front:** I started this review with a hard bias toward finding substantive disagreement. I spent the bulk of my review effort on attacks the prior two reviewers explicitly did NOT consider. I did NOT re-derive their work. I treat their conclusions as ground-truth and attack the gaps.

---

## 0. Reading-order note

5-minute read: §1 (verdict) → §2.1 (the load-bearing attack I found that survives Amendments 1+2) → §3 (verdict on §6.2-with-Amendments-1+2 + Amendment-3 + Amendment-4 + Amendment-5 + Amendment-6 proposals) → §5 (ratification recommendation).

§2 (attack vectors attempted) is the bulk of the work. Every attempted attack documents its construction, outcome, and confidence rating.

---

## 1. Executive verdict

**CONCUR-WITH-CALIBRATION** on §6.2-with-Amendments-1+2 as the right design *direction*, but **DISAGREE that Amendments 1+2 alone make it load-bearing-final**. I found **four substantive issues** that survive Amendments 1+2 and require additional amendments before ratification:

1. **AMENDMENT 3 (NEW — load-bearing): canonical-serialization length-injectivity.** The `BindingContext` enum contains variable-length fields (`audience_did: Did`, `operation: PermissionOperation`). Amendment 1's `canonical_binding()` concatenates these bytes into the AAD/info string. **Without explicit length-prefixing of every variable-length field, two distinct logical BindingContexts can canonicalize to byte-identical AAD strings.** I demonstrate a concrete collision in §2.1. Severity: **HIGH**. The fix is straightforward (tagged-length-value encoding per variant) but it MUST be added explicitly to Amendment 1 — both prior reviewers gestured at "deterministic canonical encoding (Inv-1)" without anchoring this specific failure mode.

2. **AMENDMENT 4 (NEW — load-bearing): sender-identity MUST be in AAD for DeviceLink + RemotePermission variants.** The BindingContext currently binds `provisioning_session_id` (DeviceLink) or `request_id + operation` (RemotePermission). Neither binds the **sender's DID**. An adversary who compromises ANY enrolled device on the user's mesh (= holds a valid device-key + has a path to the user's HPKE encryption pubkey) can construct a `ProvisioningPayload` that targets a different device. The HPKE recipient binding doesn't help because the sender-side of HPKE-mode-base is anonymous (RFC 9180 §5.1.1; `mode_base` provides no sender-authentication). The signature layer above provides sender-auth at the outer protocol, but the envelope-internal AAD doesn't bind sender — meaning a future code path that trusts the AAD-bound BindingContext as a sender-authority statement will be wrong. Severity: **HIGH** as a defense-in-depth gap; **MEDIUM** as an immediately-reachable exploit given the outer signature layer.

3. **AMENDMENT 5 (NEW — load-bearing): replay-window binding (timestamp + epoch) MUST be in AAD for DeviceLink + RemotePermission.** Neither `provisioning_session_id` nor `request_id` is a time-bound nonce by itself. An adversary who can replay an old `EncryptedEnvelope` (e.g. from a snapshot of network traffic, from a partially-restored backup, or from a malicious peer-mesh participant who archived past envelopes) can deliver a stale envelope at a chosen later time. The §6 wire format in the scope review names a `timestamp: u64` + `valid_until: u64` at the OUTER PermissionRequest/PermissionGrant level, but those are NOT bound into the AAD of the inner HPKE envelope. Severity: **MEDIUM-HIGH** for RemotePermission (long-lived grant impersonation); **MEDIUM** for DeviceLink (provisioning_session_id is freshly random per-session, partial defense).

4. **AMENDMENT 6 (NEW — load-bearing): Bernstein-Persichetti chosen-ciphertext side-channel surface survives §6.2 Layer-A unchanged.** The first reviewer's §4.4 + the second-opinion's §3.3 both alluded to "One Time is Enough" (IACR 2024/2051) as a Decap-side ML-KEM side-channel. NEITHER reviewer worked through whether this surface applies to §6.2 Layer-A. **It does NOT — Layer-A uses ChaCha20-Poly1305 not HPKE, so the ML-KEM Decap surface is absent at Layer-A.** That is correct. BUT: §6.2 dispatches Layer-C-drops + Layer-D-wraps to HPKE-mode-base, and **the on-disk-or-on-wire-resident envelopes for Layer-C + Layer-D are adversary-accessible** (Compromise #31 names "forever-valid drop bundles" + Atrium peer-mesh routes them through untrusted relays). An adversary with on-disk or on-wire access can mount a Bernstein-Persichetti chosen-ciphertext attack against the recipient's ML-KEM-768 sk **at envelope-Open time**, every time the recipient opens an envelope. Amendment 6 mandates explicit recipient-side mitigation: either CT-Decap verification + implicit-rejection-time normalization, or a Compromise mint disclosing the surface. Severity: **MEDIUM-HIGH** as a structural property; **MEDIUM** as practical-exploitability against a typical Benten Tauri-shell user.

I also surface **two MINOR amendments** (§4): codepoint endianness pinning and codepoint-registry-mint discipline.

**Cumulative verdict:** §6.2 design direction is right; Amendments 1+2 are necessary but not sufficient; Amendments 3+4+5+6 (load-bearing) and the two MINOR amendments take it to load-bearing-final. Confidence on the four new load-bearing amendments: **HIGH** on #3 (concrete collision shown), **HIGH** on #4 (structural property of mode_base), **MEDIUM-HIGH** on #5 (replay reachability depends on threat-model assumptions), **HIGH** on #6 (Bernstein-Persichetti is published; the surface IS reachable; mitigation has known cost).

**Most-substantive single attack:** Amendment-3's canonical-serialization length-injectivity collision (§2.1). This is the kind of failure mode that has historically taken down protocols (TLS renegotiation 2009; OAuth state-parameter binding 2014; multiple JWT-alg-confusion incidents).

**Residual concern warranting another reviewer:** §2.7 multi-stanza HPKE recipient-confusion (cross-stanza ciphertext-substitution) — I found a plausible attack class but couldn't construct an end-to-end exploit without more wire-format detail than §6.2 currently specifies. A multi-recipient HPKE specialist should look specifically at this when the multi-stanza HpkeMultiBase variant is designed.

---

## 2. Attack vectors attempted + outcomes

I enumerate every attack I attempted, in roughly decreasing severity order for the surviving ones, then the dismissed-with-evidence ones.

### 2.1 ATTACK: canonical-serialization length-injectivity collision in `BindingContext` ⇒ **SURVIVES Amendments 1+2; requires Amendment 3**

**Setup.** Amendment 1 mandates `canonical_binding()` returns:

```
codepoint (2 bytes BE) || b"benten-envelope-v1" (18 bytes) || aad_binding.canonical_serialize()
```

The second-opinion review's §2.6 row 4 says "Defense: tag-prefix each variant in the canonical serialization (per Inv-1)" — but this is hand-wave, not normative. Inv-1 says "deterministic-canonical-encoding" which is satisfied by stable serde-CBOR or DAG-CBOR. **Deterministic ≠ injective.**

**Attack construction.** Two distinct logical `BindingContext` values that produce the SAME canonical AAD bytes:

Consider `BindingContext::DropToRecipient { audience_did: Did }` where `Did` is a serialized string like `"did:key:z6Mk..."`. A natural canonical_serialize for an enum is:

```
variant_tag (1 byte) || field_bytes
```

For DropToRecipient: `0x02 || did_string_bytes`. The Did string is itself variable-length. The decoder needs to know where the Did ends. If the encoder writes ONLY `variant_tag || did_string_bytes` with no length prefix, then any later-added enum variant that prepends bytes can collide.

Concrete collision: suppose at v1-beta we have 4 variants:
- `Vault { vault_version: u8 }` → `0x00 || u8`
- `DropToRecipient { audience_did: Did }` → `0x01 || did_bytes`
- `DeviceLink { provisioning_session_id: [u8; 16] }` → `0x02 || 16-byte-session`
- `RemotePermission { request_id: [u8; 16], operation: PermissionOperation }` → `0x03 || 16-byte-id || op_bytes`

The `operation: PermissionOperation` is the §6.4 scope review enum: `Decrypt(node_cid) | SignUcanDelegation(scope, audience, expires_at) | RemoteUnlock`. `RemoteUnlock` is the zero-payload variant. Its serialization is likely `0x02` (variant tag, no payload).

Now consider a future v2 codepoint addition: a new `BindingContext::DropToGroup { atrium_id: AtriumId, member_count: u8 }` variant. Without explicit length-prefixing, an attacker who has a v2 envelope where atrium_id happens to begin with bytes that look like a Did-prefix can swap discriminators (closed by Amendment 1's codepoint-in-AAD) OR — more subtly — submit a v2 envelope to a v1 reader that decodes it as DropToRecipient and proceeds.

**More concrete: the audience_did variable-length collision RIGHT NOW (no v2 addition needed).** Consider:

```
BindingContext::DropToRecipient { audience_did: Did("did:key:z6MkABC...") }
// canonical_serialize bytes: 0x01, 'd','i','d',':','k','e','y',':','z','6','M','k','A','B','C',...
```

vs.

```
BindingContext::DropToRecipient { audience_did: Did("did:key:z6MkAB") }
// canonical_serialize bytes: 0x01, 'd','i','d',':','k','e','y',':','z','6','M','k','A','B'
```

If the encoder uses serde-default CBOR encoding for the `Did` newtype-string, CBOR DOES prefix strings with length tags, so this specific case is closed *by CBOR's behavior, not by the design*. The risk is that "canonical_serialize" is under-specified in the §6.2 sketch — a future maintainer implementing it as `audience_did.as_bytes()` directly (skipping CBOR; reasonable for a low-level layer) creates the collision.

**Even with CBOR, a structural concern persists.** CBOR map-of-keyed-fields is non-canonical-by-default (RFC 8949 §4.2.1 allows multiple canonical-form definitions; "Core Deterministic Encoding" §4.2.1 vs "CTAP2 Canonical CBOR" §4.2.2 differ on map-key ordering for indefinite-length items). If two implementations canonicalize differently (e.g., a future Benten implementation in a different language), AAD-binding fails to verify cross-impl. This is a real interop break, not a security break per se, but it means **Amendment 1 is under-specified**.

**Strongest concrete collision I could construct WITHIN Amendment 1's literal text.** Amendment 1 says:

```rust
buf.extend_from_slice(&self.aad_binding.canonical_serialize());
```

If `canonical_serialize` is implemented (by a future maintainer with reasonable Rust idioms) as `bincode::serialize(self)` or `postcard::to_bytes(self)`, neither bincode nor postcard length-prefixes top-level structs unless the encoded type is a `Vec`/`String`. For the `RemotePermission { request_id: [u8; 16], operation: PermissionOperation }` variant where operation is `PermissionOperation::SignUcanDelegation { scope, audience, expires_at }` — `scope` could be an unbounded `Vec<Capability>` whose elements depend on the application — the boundary between scope and audience and expires_at is fragile to refactoring.

**Worked collision (the most damning case).** Suppose `PermissionOperation::SignUcanDelegation { scope: Vec<u8>, audience: Did, expires_at: u64 }` (a future schema evolution where scope is bytes not a struct). Two logical values:

```
SignUcanDelegation {
    scope: [0xAA, 0xBB, 0xCC],
    audience: Did("did:key:foo"),
    expires_at: 0,
}
```

vs.

```
SignUcanDelegation {
    scope: [0xAA, 0xBB, 0xCC, b'd', b'i', b'd', b':', b'k', b'e', b'y', b':', b'f', b'o', b'o'],
    audience: Did(""),
    expires_at: 0,
}
```

With naive concat-encoding: both serialize to `[..., 0xAA, 0xBB, 0xCC, 'd','i','d',':','k','e','y',':','f','o','o', 0,0,0,0,0,0,0,0]`. **Identical bytes**, different semantic meaning. An attacker who can convince Alice to sign one of these grants (the empty-audience version) and convince Bob's parser to interpret it as the other (the full-audience version) executes a **scope-extension attack**.

**Why this survives Amendment 1.** Amendment 1 binds the codepoint, NOT the structural disambiguation of the variable-length fields. Two distinct logical BindingContexts share canonical bytes ⇒ they share AAD ⇒ both AEAD/HPKE Open succeed for the "wrong" interpretation.

**Why Amendment 2 doesn't close it.** Amendment 2 (strict-decode) checks codepoint→variant-tag dispatch. The variant tag IS `RemotePermission` in both cases. Strict-decode passes. The collision is INSIDE the variant, not across variants.

**Mitigation — propose AMENDMENT 3.**

```rust
fn canonical_binding(&self) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"benten-envelope-v1");           // domain separator
    buf.extend_from_slice(&self.codepoint.to_be_bytes());   // bound per Amendment 1
    // Amendment 3: tagged-length-value encoding for the binding context
    let serialized = self.aad_binding.canonical_serialize_tlv();
    buf.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
    buf.extend_from_slice(&serialized);
    buf
}

// canonical_serialize_tlv MUST length-prefix EVERY variable-length field.
// Per-variant pseudocode:
// DropToRecipient { audience_did }:
//   variant_tag(1) || did_len(u16-BE) || did_bytes
// RemotePermission { request_id, operation }:
//   variant_tag(1) || request_id(16) || op_tag(1) || op_payload_len(u32-BE) || op_payload
```

**Citations.** RFC 9180 §4.1 (HPKE key-schedule): explicit `labeled_extract` / `labeled_expand` with `Nh`-byte labels + length prefixes to prevent exactly this class. RFC 9180 §7.2 (Encoding constants for KEM): uses length-explicit `I2OSP(...)` per ML-KEM convention. RFC 8949 §4.2.1 (CBOR Core Deterministic Encoding) — the bare statement isn't enough; cite §4.2.2 (CTAP2) for the more constrained form that closes ambiguity. JWT/JOSE: RFC 7515 §5.1 mandates "encoded protected header" length-explicit base64url to close a similar class. Bellare-Rogaway "The exact security of digital signatures" (CRYPTO 1996) on injective domain encoding.

**Confidence on Amendment 3 as load-bearing:** **HIGH**. This is the kind of finding that historically took down JWT/JOSE alg-confusion + multiple TLS extension parsers. It's cheap to add; expensive to retrofit post-v1-beta-freeze.

---

### 2.2 ATTACK: sender-DID is absent from BindingContext ⇒ **SURVIVES Amendments 1+2+3; requires Amendment 4**

**Setup.** Look at `BindingContext` variants:

| Variant | Fields | Sender bound? |
|---|---|---|
| Vault | vault_version: u8 | N/A (encrypt-to-self) |
| DropToRecipient | audience_did: Did | NO |
| DeviceLink | provisioning_session_id: [u8; 16] | NO |
| RemotePermission | request_id, operation | NO |

For Layer-C drops (DropToRecipient): the §6.2 design uses HPKE-mode-base. RFC 9180 §5.1.1 states: *"mode_base offers neither sender authentication nor key-compromise impersonation resistance."* The auth-tag binds only the AAD + the AEAD context, not who the sender is. The sender's identity is conveyed via an OUTER signature layer (Layer-D device-key sig or user-DID sig).

For Layer-D device-link: §7 of the scope review says the outer envelope is signed by `user-DID-signing-key`. The HPKE-inner-AAD binds session_id but NOT the user-DID identity.

For Layer-D remote-permission: §6.4 of the scope review says PermissionGrant is signed by `A's user-DID-signing-key`. Same shape — outer signature, inner AAD-without-sender.

**Attack class: AAD-belief drift.** Code paths that consume an opened envelope tend, over time, to treat the AAD as load-bearing evidence of "who sealed this." If the AAD doesn't bind sender, a later refactoring that drops the outer signature check (or fails-open on a missing-signature edge case) becomes a confused-deputy hazard.

**Concrete construction.** Adversary Eve has compromised Bob's device-key (one of N enrolled devices on Alice's user-mesh). Eve sends Alice's other device Charlie an `EncryptedEnvelope { codepoint: DEVICE_LINK_PROVISIONING, payload: HpkeBase {...}, aad_binding: DeviceLink { provisioning_session_id: <fresh>} }` — with the outer signature layer constructed by Eve using Bob's stolen device-key. Charlie verifies the outer signature against Bob's device-key (which IS on Alice's user-mesh as a known device) — passes. Charlie opens the HPKE envelope, AAD verifies, gets a fresh K_principal' that Eve chose.

**Why this is a load-bearing problem.** The §7.3 wire format SIGNS the HPKE envelope with the user-DID-signing-key — meaning a device-key compromise doesn't directly let Eve impersonate user-DID. **But:** the scope review §6.4 says PermissionGrant is signed by user-DID-key, while PermissionRequest is signed by the *device-key* of the requesting device. **The asymmetry is the attack surface.** If Eve has Bob's device-key, Eve can issue PermissionRequests in Bob's name, including `Decrypt(node_cid)` for nodes Bob is authorized to decrypt. Alice approves (her device sees a request from her known-device Bob). Eve gets the decrypt grant. **The grant's AAD doesn't bind that the recipient is Eve-impersonating-Bob.**

This is more subtle than "AAD should bind sender" — it's "AAD should bind a hash of the BUNDLED CONTEXT including the outer signing identity and grant recipient." The Inv-15 discipline (payload-CID-as-identifier, NOT sig-bundle-CID) is structurally adjacent here.

**Why Amendment 1 doesn't close it.** Amendment 1 binds only the codepoint + the BindingContext fields. The BindingContext fields don't include sender.

**Why §3.6 (cross-protocol composability) doesn't trivially close it.** The outer signature provides sender-auth at the OUTER protocol layer. But the envelope's AAD is consumed by code that ALSO consumes the outer signature; conflating "envelope opened successfully" with "envelope was sent by intended sender" is a future-bug class. Defense-in-depth = bind sender into AAD so envelope-open == sender-confirmed regardless of outer-layer state.

**Mitigation — propose AMENDMENT 4.**

```rust
pub enum BindingContext {
    Vault { vault_version: u8 },
    DropToRecipient {
        audience_did: Did,
        sender_did: Did,                  // Amendment 4: bind sender
    },
    DeviceLink {
        provisioning_session_id: [u8; 16],
        sender_device_did: DeviceDid,     // Amendment 4: bind sender-device
        // (recipient_device_pubkey already bound via HPKE recipient choice)
    },
    RemotePermission {
        request_id: [u8; 16],
        operation: PermissionOperation,
        granting_user_did: Did,            // Amendment 4: bind grantor identity
        requesting_device_did: DeviceDid,  // Amendment 4: bind requester
    },
}
```

**Citation chain.** RFC 9180 §9.1.1 (mode_base sender-auth limitation explicit). RFC 9180 §10.2 (mode_auth as the alternative, but mode_auth requires sender's static KEM-keypair which Benten doesn't have because senders are signature-based not KEM-based). MLS RFC 9420 §5.1 (MLS bindings include sender LeafIndex in AAD-equivalent context); MLS does this for the same reason. Inv-15 in `docs/INVARIANT-COVERAGE.md` (payload-content-binding-as-identity discipline parallels).

**Confidence on Amendment 4 as load-bearing:** **HIGH** as a defense-in-depth gap. **MEDIUM** as an immediately-reachable exploit (the outer signature layer catches naive attacks; the attack surface materializes through refactoring drift or implementation-bug confusion).

---

### 2.3 ATTACK: replay-window absence ⇒ **SURVIVES Amendments 1+2+3+4; requires Amendment 5**

**Setup.** Neither `provisioning_session_id` nor `request_id` is a *time-bounded* nonce. Both are random 16-byte values. The outer wire format (scope review §6.4 + §7) names `timestamp: u64` + `valid_until: u64` at the OUTER PermissionGrant level, but these aren't bound into the inner envelope's AAD.

**Replay scenario 1 — RemotePermission stale-grant impersonation.**
1. Alice grants Bob `Decrypt(node_cid_X)` permission at time T₀, valid until T₀+1h. Envelope on wire.
2. Eve archives the envelope.
3. At time T₀+10h (well past validity), Eve replays the envelope to Bob's parser. Bob's HPKE opens successfully (the AAD is unchanged). Bob's outer protocol layer SHOULD check `valid_until < now()` and reject — but if the outer check is bypassed (refactoring; restoring-from-backup bypasses outer checks; cross-device sync delivers the archived envelope directly to a state-restoration code path that skips outer checks) the envelope's INNER decryption succeeds and Bob's code may treat the result as a current grant.

**Replay scenario 2 — DeviceLink session reuse on restored backup.**
- User backs up device A's state to cloud. The backup includes the historical `ProvisioningPayload` ciphertext for device B (from when B was originally provisioned).
- User restores backup onto device A (months later) for disaster-recovery.
- The restored state contains an envelope with codepoint DEVICE_LINK and an AAD-binding to `provisioning_session_id`. A future code path that scans for pending-device-links may re-trigger a provisioning flow using the historical envelope — installing the historical K_principal onto a device that should have been re-provisioned with a fresh session.

**Replay scenario 3 — Atrium peer-mesh archival.**
- Compromise #31 names "Drop bundle revocation reach (forever-valid bundles)" — the existing posture is that Drops persist indefinitely on the peer mesh. An adversary mining the mesh for old envelopes who can later compromise the recipient's device gets an unlimited replay window on every Drop.

**Why Amendment 1+2+3+4 don't close this.** None of them bind a freshness epoch into the AAD. The provisioning_session_id IS fresh per session, but it's freshness-as-uniqueness not freshness-as-time-bound. An adversary doesn't need to forge a fresh session_id — they just replay the historic one.

**Mitigation — propose AMENDMENT 5.**

Two complementary changes:

1. **Bind a sealed-at-epoch timestamp into the AAD for time-sensitive variants:**

```rust
pub enum BindingContext {
    // ...
    DeviceLink {
        provisioning_session_id: [u8; 16],
        sender_device_did: DeviceDid,
        sealed_at_epoch_seconds: u64,                  // Amendment 5
        valid_until_epoch_seconds: u64,                // Amendment 5
    },
    RemotePermission {
        request_id: [u8; 16],
        operation: PermissionOperation,
        granting_user_did: Did,
        requesting_device_did: DeviceDid,
        sealed_at_epoch_seconds: u64,                  // Amendment 5
        valid_until_epoch_seconds: u64,                // Amendment 5
    },
}
```

2. **Decoder MUST refuse opens where `now() > valid_until_epoch_seconds + clock_skew_tolerance`.** This is structurally enforced inside the envelope-open path, NOT delegated to outer-layer caller discipline.

Vault and DropToRecipient are out-of-scope for time-bound (vault is at-rest by design; drops are intentionally long-lived per Compromise #31; both layer their own freshness via outer-layer mechanics).

**Citation chain.** RFC 9180 §9.7.2 (HPKE replay considerations — explicit that mode_base does not provide replay protection; protocols must provide it). MLS RFC 9420 §6.2 (Welcome-message epoch binding). Filippo Valsorda "age and Authenticated Encryption" on file-encryption replay considerations.

**Confidence on Amendment 5 as load-bearing:** **MEDIUM-HIGH**. The reachability of replay attacks depends on whether the outer wire format's validity-window check is rigorously enforced at every consumption site. Defense-in-depth in the inner envelope's AAD makes the property structural rather than discipline-dependent.

---

### 2.4 ATTACK: Bernstein-Persichetti chosen-ciphertext side-channel on Layer-C/D Decap ⇒ **SURVIVES Amendments 1+2+3+4+5; requires Amendment 6 OR a disclosed Compromise**

**Setup.** Both prior reviewers correctly noted that the first reviewer's §4 (ML-KEM **KeyGen**-from-secret-seed timing side-channel) does NOT apply to §6.2 Layer-A (which uses ChaCha20-Poly1305 not ML-KEM). True. **But neither reviewer worked through the parallel concern for Layer-C/D's HPKE Decap path.**

The first reviewer's §4.4 explicitly mentioned: *"ML-KEM Decap inside HPKE.Open: Has a known side-channel surface (CCA-style chosen-ciphertext attacks per 'One Time is Enough,' IACR 2024/2051). This surface is INDEPENDENT of whether the keypair is derived-from-password vs. random — it applies to all ML-KEM usage."* The second-opinion review acknowledged this in §3.3 as "complementary to keygen side-channel" but didn't elevate it to a §6.2 design concern.

**The omission:** §6.2 inherits this surface unchanged. The mitigation question (does Benten's RustCrypto `ml-kem` impl ship Bernstein-Persichetti-resistant Decap with normalized implicit-rejection timing) is unanswered. Layer-C drops are adversary-disk-accessible (envelopes routed through the Atrium peer-mesh; relays are untrusted per CLAUDE.md baked-in #18); Layer-D remote-permission envelopes traverse iroh-gossip channels (recipient-device-accessible at minimum). The attack class is reachable in both layers.

**The attack scenario.** Eve has Drop bundles addressed to Alice's pubkey (downloaded from peer-mesh routing). Each bundle's ML-KEM ciphertext can be malformed by Eve (chosen-ciphertext attack); Eve waits for Alice to open the bundle (which is a routine engine operation), then measures Alice's keygen+decap timing via co-resident-process or microarchitectural side-channel. Per Bernstein-Persichetti 2024/2051, **a single chosen-ciphertext Decap query suffices to recover bits of Alice's ML-KEM secret key under specific impl conditions.** Recovery of the full sk requires multiple queries; the attack scales linearly in queries.

**Why §6.2's design choice doesn't change this.** This attack exists for HPKE-mode-base[X-Wing] regardless of how the envelope is wrapped. The first reviewer flagged it as "surface present in any ML-KEM usage." But that's the load-bearing point: **§6.2 unifying the envelope shape doesn't reduce the surface — it codifies it as the canonical encrypt-to-recipient primitive.** Any honest disclosure of §6.2 as load-bearing-final MUST acknowledge this surface.

**Comparison with the alternatives that were dismissed.**
- Option F+ would have added the **keygen-from-secret-seed** surface (§4 of first review) to the already-present Decap surface (§4.4 of first review). The NO-GO on F+ removes the keygen surface but NOT the Decap surface.
- §6.2-with-Amendments-1+2 + B-equivalent primitive choice still ships HPKE-mode-base[X-Wing] for Layer-C/D. The Decap surface is structural to this choice. Cannot be removed without abandoning ML-KEM.

**Mitigation options — propose AMENDMENT 6.**

Three options for ratification, pick one:

**(a) CT-Decap mandate:** require that the chosen RustCrypto `ml-kem` version provides constant-time Decap with normalized implicit-rejection timing (FIPS 203 §6.3 implicit-rejection produces a deterministic K' indistinguishable from a true K to a constant-time observer). Verify at integration-time + add a pre-merge CI test that times Decap under a corpus of malformed ciphertexts. If the version doesn't ship CT-Decap, pin a different version or wait for one.

**(b) Compromise mint:** mint a new `SECURITY-POSTURE.md` Compromise (call it Compromise #32) explicitly disclosing the ML-KEM-Decap chosen-ciphertext side-channel surface; bound it via the classical-floor (X25519 half of X-Wing) and via the Compromise #30 closure at v1-GM independent audit; document the threat-model boundary (typical Tauri user without co-resident-malware is unaffected; co-resident-malware or shared-cloud-relay scenarios are exposed).

**(c) Both:** integrate (a) into impl + still mint (b) for honest disclosure.

**Recommendation:** option (c). Both have low cost; both are belt-and-suspenders against a published attack class.

**Citation chain.** Bernstein & Persichetti, "One Time is Enough: Chosen-Ciphertext Side-Channel Attack on ML-KEM Cryptosystems," IACR 2024/2051. FIPS 203 §6.3 (implicit-rejection ⇒ deterministic K' on malformed ciphertext). PQShield "Formally verifying AVX2 rejection sampling for ML-KEM" (referenced by first reviewer §9.2). draft-sfluhrer-cfrg-ml-kem-security-considerations-04 (Nov 2025 IRTF-track multi-org WG document; the security-considerations doc for ML-KEM operational deployments). RustCrypto `ml-kem` crate's RustSec advisory posture (current at-write-time pinning required).

**Confidence on Amendment 6 as load-bearing:** **HIGH** on the structural property; **MEDIUM-HIGH** on the immediate exploitability against Benten's typical Tauri deployment (no co-resident-malware threat ⇒ attack reachable only via cloud-relay-tenant timing-correlation, which is exotic-but-not-absurd as the second-opinion's §3.3 enumerated). Either way, honest disclosure is the v1-beta-freeze-context discipline.

---

### 2.5 ATTACK: codepoint endianness ambiguity ⇒ **MINOR but worth pinning explicitly**

**Setup.** Amendment 1's code sketch uses `self.codepoint.to_be_bytes()`. The second-opinion review's §2.4 example chose `0x6101` for LAYER_A_VAULT — but neither prior review names the canonical endianness of the codepoint in the wire format itself (only in the AAD-binding helper).

**Attack class.** Two implementations could disagree on whether the codepoint is BE or LE on the wire. If a future Benten implementation (e.g., a TypeScript impl for browser interop) reads codepoint LE while the canonical Rust impl writes BE, every envelope fails verification — denial-of-service rather than direct cryptographic break. But: if the implementations disagree AND someone writes a "lenient parser" fallback (which is the entire reason Amendment 2 exists), the lenient parser could re-interpret the codepoint and dispatch to the wrong variant.

**Mitigation.** Pin the on-wire codepoint endianness explicitly in the design doc + in Amendment 1. Specify big-endian (network byte order). This is a 1-line spec amendment.

**Confidence:** MEDIUM. The risk is low under Amendment 2's strict-decode, but the explicit pinning closes the residual ambiguity at zero cost.

---

### 2.6 ATTACK: codepoint-registry-mint discipline (governance) ⇒ **MINOR but worth naming explicitly**

**Setup.** §6.2 introduces 4 codepoints (Vault/Drop/DeviceLink/RemotePermission). Amendment 2 mandates `codepoint→variant-tag` dispatch table. **Neither amendment specifies the governance discipline for minting new codepoints.**

**Attack class.** A future contributor (or a malicious-PR adversary; or an inattentive copy-paste between branches) mints a codepoint with the SAME value as an existing one, OR with a value that collides with another protocol's IANA-reserved codepoint (e.g., RFC 9180's `kem_id` registry uses `0x0010..0x0030` range; if Benten's codepoints overlap, cross-protocol-confusion in shared HPKE library implementations becomes possible).

**Mitigation.** Adopt a codepoint-registry discipline:
- Reserve a Benten-specific range (e.g., `0x6100..0x6FFF`) explicitly disjoint from IANA HPKE registry.
- Require codepoint mints to update a registry table in `docs/CRYPTO-CODEPOINTS.md` (new file) referenced from CLAUDE.md baked-in #5.
- Add a cite-drift-detector scanner check for codepoint-mint-without-registry-row.

**Confidence:** MEDIUM. This is governance-not-cryptographic, but it's the kind of discipline that prevents the *class* of cross-protocol-confusion incidents that JOSE+COSE have suffered.

---

### 2.7 ATTACK ATTEMPTED but inconclusive: multi-stanza HPKE recipient-confusion

**Attempt.** The second-opinion's §2.3 row 5 noted that multi-recipient HPKE (RFC 9180 §10.3) isn't accommodated by the current `HpkeBase { enc, ciphertext }` shape and would naturally extend to `HpkeMultiBase { enc_per_recipient: Vec<(audience_did, enc)>, ciphertext }`. I tried to construct a cross-recipient ciphertext-substitution attack against this future shape:

- Setup: Alice sends a Drop to {Bob, Carol}. Multi-stanza: `enc_Bob || enc_Carol || ciphertext_shared`.
- Adversary attempts to deliver `enc_Carol || enc_Eve || ciphertext_shared` to Bob (Eve being adversary-controlled).
- The Bob-targeted Decap would search for the matching enc-for-Bob; not finding it, Open fails.

But: can the adversary swap stanzas (`enc_Bob || enc_Eve` instead of `enc_Bob || enc_Carol`) such that Bob's Open succeeds but the AUDIENCE list is now `{Bob, Eve}` not `{Bob, Carol}`? This is the cross-stanza ciphertext-substitution class.

**Analysis.** The defense is to bind the canonical multi-recipient audience set into the AAD (per the Amendment 1 codepoint-in-AAD discipline extended to include "set of all recipient DIDs in canonical order"). This is standard MLS-tree-of-recipients pattern (RFC 9420 §11.3).

**Why I couldn't construct an end-to-end exploit.** §6.2 doesn't yet specify the multi-stanza wire format. The attack would depend on whether the recipient set is committed into AAD (closes the attack) or kept as a parallel structure (admits the attack). Without spec text, I can't pin down the attack.

**Recommendation.** When the `HpkeMultiBase` variant is designed (deferred to F-full implementation phase per second-opinion's §2.3), an MLS-PQ TreeKEM specialist should review specifically for cross-stanza ciphertext-substitution. The canonical-recipient-set-in-AAD discipline is the likely fix.

**Confidence:** MEDIUM that there's a real attack class here; HIGH that it warrants specialist review when the variant is designed. NOT load-bearing on the v1-beta tag (multi-stanza HPKE is post-tag per the scope review §11.1).

---

### 2.8 Attacks I attempted and DISMISSED with evidence

These are attack vectors from the orchestrator's checklist + ones I devised, where I concluded the design (with Amendments 1+2 or with my added Amendments 3-6) defeats them. I document briefly so future reviewers don't re-derive.

**(a) Cross-codepoint ciphertext substitution (Vault as Drop ciphertext or vice versa).** Closed by Amendment 1 (codepoint-in-AAD) + Amendment 2 (strict-decode). The auth-tag's AAD-binding fails on mismatched codepoint. Variant-confusion fails on strict-decode dispatch. **Defeated.**

**(b) Layer-B per-Node AEAD key confused with K_principal.** Layer-B uses `K(N) = KDF(K_principal, N.cid)` per scope review §15.4. The K(N) values are derived per-Node with a CID-binding salt; cross-Node collision requires CID-collision (Compromise #6 BLAKE3 128-bit floor closes this). The K_principal is NEVER passed where K(N) is expected (Rust type system: `KPrincipal` and `KNode` are distinct newtypes per the existing codebase conventions). **Defeated.**

**(c) HPKE enc field confused with AEAD nonce.** Strict-decode forbids the cross-variant confusion. **Defeated.**

**(d) Vault password-confirm-or-reject oracle.** §6.2 Layer-A uses AEAD-under-DAK (NOT a pseudo-keypair). The auth-fail-on-wrong-password is THE intended behavior (it's how the user knows they entered the wrong password). The first reviewer's §4.1 finding (pseudo-pubkey-leak) does NOT apply to AEAD-under-DAK — there's no pubkey. **Defeated by construction.**

**(e) Length-leakage of codepoint via payload size.** Second-opinion §2.3 row 4: codepoint is already public. **Defeated by analysis.**

**(f) Cross-deployment same-codepoint confusion.** Solved by Amendment 1's `b"benten-envelope-v1"` domain separator (covered) + Amendment 6's codepoint-registry discipline. **Defeated.**

**(g) Key-rotation hazard: K_principal rotates ⇒ Layer-C drops orphaned.** This is real but is an OUT-OF-SCOPE design question for §6.2 itself — it's a key-management policy decision, not an envelope-shape decision. K_principal rotation under the §6.2 envelope simply means: old drops still encrypted to old-pubkey-derived-keys (which is wrong analysis; recipient pubkey is independent of K_principal — K_principal is the LOCAL vault key; the recipient's HPKE-keypair is the LONG-LIVED recipient identity-keypair). Re-reading my own analysis: K_principal rotation does NOT affect Layer-C drops (recipient pubkey is separate). It affects only Layer-A vault re-encryption (trivial: re-AEAD-Seal the same plaintext under new DAK). **Defeated by analysis** — not a §6.2 envelope concern.

**(h) Cross-variant decoder bug exploit.** Strict-decode (Amendment 2) + structural conformance check (Amendment 2 sub-rule 3) + canonical-serialization length-injectivity (Amendment 3) collectively close. **Defeated.**

**(i) Future-codepoint-extension breaks existing variants.** Closed by codepoint-in-AAD (Amendment 1) + strict-decode (Amendment 2) + registry discipline (Amendment 6). **Defeated.**

**(j) Unknown codepoint arrives.** Amendment 2 sub-rule 1: typed-reject. The behavior is unambiguous (parse-fail, no fallback). Decoder MUST NOT attempt cross-variant decoding. **Defeated.**

**(k) Reserved-future codepoint with current-format payload.** Amendment 2's structural conformance check rejects this. **Defeated.**

**(l) AEAD nonce reuse hazard within Vault codepoint.** This is a Layer-A primitive-impl concern (must use either random-12-byte nonces with rare-reuse-tolerance OR counter-based nonces with reset-on-rotation). The §6.2 envelope shape doesn't constrain it; the impl wave's pre-merge cryptographer review must. **Out of scope of envelope review** but flagged as an impl-concern. The PSP recommendation (XChaCha20-Poly1305 with 24-byte random nonce) closes this with negligible cost — note that Layer-A could profitably use XChaCha20 over ChaCha20 for the wider nonce + lower-reuse-risk.

**(m) IND-CCA2 reduction holds under §6.2 envelope-wrapping.** The construction is HPKE.Seal(plaintext, recipient_pk, aad = canonical_binding) for the HpkeBase variant. The aad parameter is an opaque byte string in RFC 9180 §5.2; the IND-CCA2 reduction holds for any choice of aad. **Defeated by RFC 9180 §9.1.2.**

---

## 3. Verdict on §6.2-with-Amendments-1+2 viability

**Direction:** CORRECT. The envelope-layer-unification is the right architectural shape; the codepoint-discriminated payload variants are correct; the per-codepoint primitive choice is sound.

**Sufficiency of Amendments 1+2 alone:** **INSUFFICIENT for load-bearing-final**. Two prior reviewers concurred, but neither stress-tested the design against the four attack classes in §2.1–§2.4. With Amendments 3-6 added (load-bearing) and Amendments 7-8 (minor) added, the design becomes load-bearing-final.

**Cumulative amendment list to reach load-bearing-final:**

| Amendment | Author | Status |
|---|---|---|
| 1 — codepoint-in-AAD | second-opinion review | **APPROVED** |
| 2 — strict-decode + no cross-variant fallback | second-opinion review | **APPROVED** |
| 3 — canonical-serialization length-injectivity (TLV encoding) | this review §2.1 | **PROPOSED — load-bearing** |
| 4 — sender-DID/sender-device-DID in BindingContext for non-Vault variants | this review §2.2 | **PROPOSED — load-bearing** |
| 5 — sealed-at + valid-until epoch binding for DeviceLink + RemotePermission | this review §2.3 | **PROPOSED — load-bearing** |
| 6 — Bernstein-Persichetti Decap-side-channel mitigation (CT-Decap pin + Compromise mint) | this review §2.4 | **PROPOSED — load-bearing** |
| 7 — codepoint endianness explicit (BE) | this review §2.5 | **PROPOSED — minor** |
| 8 — codepoint-registry discipline + IANA-disjoint range | this review §2.6 | **PROPOSED — minor** |

**Open items for a future reviewer (NOT load-bearing on v1-beta tag):**

- Multi-stanza HpkeMultiBase variant — cross-stanza ciphertext-substitution review when designed (§2.7).
- Layer-A nonce-scheme decision (XChaCha20 vs ChaCha20) — impl-wave pre-merge review (§2.8 row l).

---

## 4. Recommended additional amendments / hardening

Consolidating §3 into a single normative amendment block to add to §6.2:

### 4.1 Updated §6.2 design

```rust
pub struct EncryptedEnvelope {
    codepoint: u16,                       // BIG-ENDIAN on-wire (Amendment 7)
    payload: EnvelopePayload,
    aad_binding: BindingContext,
}

pub enum EnvelopePayload {
    SymmetricAead { ciphertext: Bytes, nonce: [u8; 12] },  // or [u8; 24] for XChaCha20 — IMPL DECISION
    HpkeBase { enc: Bytes, ciphertext: Bytes },
    // Future-additive: HpkeMultiBase { enc_per_recipient, ciphertext } — multi-stanza review when designed
}

pub enum BindingContext {
    Vault {
        vault_version: u8,
    },
    DropToRecipient {
        audience_did: Did,
        sender_did: Did,                  // Amendment 4
    },
    DeviceLink {
        provisioning_session_id: [u8; 16],
        sender_device_did: DeviceDid,                  // Amendment 4
        sealed_at_epoch_seconds: u64,                  // Amendment 5
        valid_until_epoch_seconds: u64,                // Amendment 5
    },
    RemotePermission {
        request_id: [u8; 16],
        operation: PermissionOperation,
        granting_user_did: Did,                        // Amendment 4
        requesting_device_did: DeviceDid,              // Amendment 4
        sealed_at_epoch_seconds: u64,                  // Amendment 5
        valid_until_epoch_seconds: u64,                // Amendment 5
    },
}

impl EncryptedEnvelope {
    /// Canonical AAD/info-string bound into every Seal/Open call.
    /// MUST be called by encoder + decoder identically (Amendment 1).
    /// MUST be tagged-length-value encoded for every variable-length field (Amendment 3).
    fn canonical_binding(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"benten-envelope-v1");              // domain separator
        buf.extend_from_slice(&self.codepoint.to_be_bytes());      // Amendment 1 + 7
        let body = self.aad_binding.canonical_serialize_tlv();     // Amendment 3
        buf.extend_from_slice(&(body.len() as u32).to_be_bytes()); // length-prefix
        buf.extend_from_slice(&body);
        buf
    }
}
```

### 4.2 Decoder MUST (Amendment 2 + 3 + 5 + 6 normative)

1. Read codepoint first. Look up in static dispatch table → expected variant tag. Strict-decode (Amendment 2).
2. Decode the EnvelopePayload variant ONLY in the form expected for the codepoint. Cross-variant decoding is a parse-fail (Amendment 2).
3. Decode the BindingContext variant ONLY in the form expected for the codepoint. Cross-variant matching is a parse-fail (Amendment 2 + 3).
4. Verify canonical_serialize_tlv produces injective encoding for the BindingContext fields. Every variable-length field is preceded by a fixed-width length prefix (Amendment 3).
5. For DeviceLink + RemotePermission: BEFORE calling Open, check `now() <= valid_until_epoch_seconds + clock_skew_tolerance`. If outside the window, reject without invoking the primitive (Amendment 5).
6. For HpkeBase Open: use a Decap implementation with normalized implicit-rejection timing (Amendment 6). Verify the chosen `ml-kem` crate version provides this; if not, pin a different version or wait. Mint Compromise #32 disclosing the surface even if mitigated (Amendment 6 option-c).

### 4.3 Codepoint registry (Amendment 8)

- Create `docs/CRYPTO-CODEPOINTS.md` enumerating every minted codepoint with a normative table.
- Reserve `0x6100..0x6FFF` for Benten envelope codepoints (disjoint from IANA HPKE registry `0x0010..0x0030`).
- Require codepoint mints to update the registry table via cite-drift-detector scanner check.

### 4.4 Compromise mint (Amendment 6 option-c)

Add to `docs/SECURITY-POSTURE.md`:

> **Compromise #32 — ML-KEM-768 Decap chosen-ciphertext side-channel surface (Layer-C drops + Layer-D wraps)**
>
> Status: REGISTERED. Closure: v1-GM independent audit (NF-2 / C-GM-AUDIT) + RustCrypto `ml-kem` CT-Decap verification at integration time.
>
> The HPKE-mode-base[MLKEM768-X25519] primitive used at Layer-C (drop) + Layer-D (wraps) inherits the published ML-KEM Decap chosen-ciphertext side-channel surface (Bernstein-Persichetti, IACR 2024/2051). An adversary with on-disk or on-wire access to envelopes can mount chosen-ciphertext queries against a recipient's ML-KEM-768 sk; per-query leakage is small but compounds across queries.
>
> MITIGATION: (a) The X-Wing classical floor (X25519 half) provides residual security against the ML-KEM-half compromise. (b) RustCrypto `ml-kem` v0.x pinned to the most-recent CT-Decap-verified version. (c) For Layer-A vault, the surface is absent (Layer-A uses ChaCha20-Poly1305 not HPKE).
>
> SCOPE: applies to all Layer-C drops + Layer-D wraps that traverse adversary-accessible media (peer-mesh; iroh transports; backup media). Does NOT apply to in-process envelope handling.

### 4.5 Test-vector additions for the R3/R5 implementer briefs

Beyond the second-opinion's three test vectors (§5 row 7), add:

- **Length-injectivity test:** construct two distinct logical BindingContexts that concat-encode to identical bytes (per §2.1 worked example); verify the canonical_serialize_tlv produces distinct bytes for both.
- **Sender-substitution test:** construct an envelope with one sender_did in AAD; rewrite sender_did to different DID; MUST fail with AEAD-auth-tag-mismatch.
- **Replay-window test:** seal an envelope at T₀ with valid_until T₀+1h; attempt open at T₀+10h; MUST fail with expiry-check before primitive invocation.
- **Codepoint-endianness test:** encoder writes BE; decoder reads BE; LE-misinterpretation MUST result in dispatch-table-miss + parse-fail.
- **Bernstein-Persichetti regression test:** time Decap against a corpus of malformed ciphertexts (chosen-ciphertext fuzzing); verify constant-time within statistical bounds; flag if variance exceeds Bernstein-Persichetti's published per-query leakage rate.

---

## 5. Recommendation to Ben on ratification

**RATIFY §6.2-with-Amendments-1-through-8 as load-bearing-final after one more review pass.** Specifically:

1. **Ratify Amendments 1+2 NOW** — they're already independently approved by two reviewers.
2. **Dispatch a focused 4th-reviewer pass on Amendments 3-6 (load-bearing).** A second adversarial-design reviewer specifically targeting MY proposed Amendments 3-6, attempting to find composition errors in MY amendments. I have lower confidence on the *interaction* between Amendments 3+4+5+6 than I do on any one individually. The cross-amendment composability deserves an independent eye. Time budget: 1-2 hours focused review. If they find nothing substantive, ratify all 8 amendments as load-bearing-final.
3. **Mint Compromise #32 (Amendment 6 option-c) at the same time as Amendment 6 ratification.** Honest disclosure of the Bernstein-Persichetti surface is non-negotiable for v1-beta-freeze posture.
4. **Inv-16 mint phrasing:** use the second-opinion's §4.3 B-equivalent phrasing PLUS explicit reference to Amendments 3-6 normatively. Inv-16 should bind not just envelope-shape uniformity but the TLV-length-injectivity + sender-binding + replay-window + Decap-CT properties.
5. **R3/R5 implementer briefs:** include all 5+3=8 test vectors from §4.5 + second-opinion §5 row 7 as explicit pre-merge gate.

**Do NOT ratify §6.2-with-Amendments-1+2 alone as load-bearing-final.** The four substantive gaps in §2.1-§2.4 each survive Amendments 1+2 and EACH would be a likely audit-firm finding at v1-beta external-audit time. The cost of adding Amendments 3-6 now (~1-2 wave-days of design + impl) is dramatically lower than the cost of retrofitting them post-v1-beta-wire-freeze.

**If Ben prefers minimal scope expansion:** Amendments 3 + 6 are the two I would NOT ship without. Amendments 4 + 5 are defense-in-depth (recommended) but less load-bearing on cryptographic soundness per se (more on operational-correctness drift over time). A minimal load-bearing addition would be Amendments 1+2+3+6. The MEDIUM-confidence amendments (4 + 5) can be queued as load-bearing-but-non-blocking with explicit deferral records in `.addl/phase-4-meta/v1-FROZEN-INTERFACE-DEFERRED.md`.

**My confidence on this recommendation:** **HIGH** that Amendments 3+6 are load-bearing; **MEDIUM-HIGH** that Amendments 4+5 are load-bearing; **HIGH** that Amendments 7+8 are zero-cost improvements.

---

## 6. Self-assessment + confidence calibration

### 6.1 What I did + how I worked

1. Pre-flight: read both prior reviews in full (412 + 520 lines), the scope review's §6 + §7 + §14.1 (the F+ pitch), the cryptographer-bird-of-prey review for Inv-15 context, and the encrypt-to-recipient cryptographer review for Option-B baseline framing.
2. Built an attack-vector checklist drawing from (a) the orchestrator's prompt's 12 attack categories + (b) my own additions (replay, sender-binding, length-injectivity, governance).
3. For each attack: constructed concrete pseudocode → walked through whether Amendments 1+2 close it → if NO, draft a proposed additional amendment → check whether the new amendment composes with the existing ones.
4. Cross-checked each load-bearing amendment against primary literature (RFC 9180; FIPS 203; Arriaga-Barbosa-Boyen 2025/1399; Bernstein-Persichetti 2024/2051; RFC 9420 MLS; Bellare-Rogaway CRYPTO 1996).
5. Explicitly attempted to find attacks against MY OWN proposed amendments (§2.7 multi-stanza; cross-amendment composability noted in §5 recommendation #2).

### 6.2 Confidence summary per finding

| Finding | Confidence | Rationale |
|---|---|---|
| §2.1 Amendment 3 (TLV length-injectivity) is load-bearing | **HIGH** | Concrete collision demonstrated. Aligns with multiple historical failures (JWT alg-confusion; TLS parser ambiguity). |
| §2.2 Amendment 4 (sender-DID binding) is load-bearing | **HIGH** (defense-in-depth) | RFC 9180 §5.1.1 + §9.1.1 confirm mode_base has no sender-auth. Structural property of the primitive choice. |
| §2.3 Amendment 5 (replay-window) is load-bearing | **MEDIUM-HIGH** | Reachability depends on outer-layer validity-window enforcement. Defense-in-depth in inner AAD makes property structural. |
| §2.4 Amendment 6 (Bernstein-Persichetti) is load-bearing | **HIGH** (structural) / **MEDIUM-HIGH** (immediate practical reachability) | Surface inherited from primitive choice; published attack; mitigation has known cost. |
| §2.5 Amendment 7 (endianness pin) is improvement | **MEDIUM** | Risk low under Amendment 2; pinning closes residual ambiguity at zero cost. |
| §2.6 Amendment 8 (codepoint registry) is improvement | **MEDIUM** | Governance discipline; prevents cross-protocol-confusion class. |
| §2.7 multi-stanza recipient-confusion warrants specialist review | **MEDIUM** | Attack class plausible; no end-to-end exploit constructed. Deferred to multi-stanza design-time. |

### 6.3 Lower-confidence areas (honest disclosure)

1. **Amendment 4's MEDIUM-rated immediate-exploitability.** The outer signature layer DOES catch the naive case. The exploit reachability requires either refactoring-drift OR a missing-signature edge case OR a confused-deputy flow. My MEDIUM-confidence on practical exploitability reflects this — the structural gap is HIGH-confidence; the immediate-exploit-construction is harder to demonstrate without inventing a specific Benten code path.

2. **Amendment 5's MEDIUM-HIGH rating.** The replay-protection depends on whether implementers correctly enforce the outer wire format's validity-window. I'm trusting the inner-binding to defense-in-depth this. A reviewer who's confident in outer-layer enforcement might rate this MEDIUM instead.

3. **§2.7 multi-stanza analysis is incomplete.** I named the attack class but couldn't construct an end-to-end exploit because the spec doesn't yet exist. This is the largest known gap in my coverage. Deferred to multi-stanza design-time review.

4. **I did NOT review whether `Did` and `DeviceDid` and `PermissionOperation` have well-defined canonical encodings.** These are existing Benten types presumably with DAG-CBOR-Inv-1-compliant encodings, but I didn't independently verify this. If any of them admits multiple canonical forms, that's a separate amendment.

5. **I did NOT review the Layer-B per-Node AEAD interaction with §6.2.** The scope review §15.4 notes that Layer-B per-Node AEAD has "OVERLAPPING threat-model coverage" with F-full DAK. §6.2 doesn't address this; my review preserves that out-of-scope deferral. If Layer-B-into-Layer-C key-wrap composition has a leak, I didn't find it; a reviewer specifically focused on Layer-B/C boundary is warranted.

### 6.4 What I could be wrong about

Most-likely failure mode of MY review: **a novel cross-amendment composition error I didn't see.** Specifically: Amendment 3 (TLV) + Amendment 4 (sender-binding) interact in the canonical serialization — if a future maintainer puts the sender_did before the audience_did in the canonical encoding but the encoder/decoder pair don't agree, the AAD-binding fails to verify cross-impl. Closed by adopting a strict deterministic encoding (CBOR with sorted-keys + length-explicit) but worth flagging.

A contrarian reviewer might say "Amendments 3-6 are over-engineering; the outer signature + outer wire format suffice." I considered this and rejected it because:
- (a) The Inv-15 discipline ("payload-CID-as-identifier, sig-bundle-CID is NEVER load-bearing") is the same insight: defense-in-depth at the binding layer is non-negotiable for v1-beta-freeze contexts.
- (b) The asymmetry argument from the second-opinion §3.1 ("be cautious; v1-beta freezes wire format") applies to MY proposed amendments too.
- (c) Each of Amendments 3-6 closes a specific, documented failure-mode class with historical precedent for that class causing breakage at production scale.

### 6.5 Sharpest contrarian pushback I considered against MYSELF

"Amendments 3-6 are creeping featurism. The original §6.2 design is clean; adding 4 more amendments converts it into a heavy 8-amendment monster."

Rebuttal: each amendment closes a distinct failure mode that survives the prior amendments. Aggregate cost is small (each amendment is a few struct fields + a few decoder checks). Aggregate benefit is closing 4 historically-recurring attack classes pre-freeze. The cleanliness argument is real but loses to the wire-format-freeze risk-asymmetry. If Ben prefers minimal scope, Amendments 1+2+3+6 (drop 4+5) is a defensible compromise per §5 recommendation #4.

### 6.6 What I would tell Ben in plain English

"The two prior reviewers agreed §6.2 with Amendments 1+2 is the right design. I agree it's the right *direction*. But I found four more amendments the design needs before it's freeze-ready: (3) length-prefix every variable-length field in the AAD binding, (4) bind sender identity into AAD for non-vault variants, (5) bind sealed-at + valid-until timestamps for DeviceLink + RemotePermission, (6) verify the ML-KEM Decap impl is constant-time + mint a Compromise disclosing the surface. Plus two minor improvements: explicit codepoint endianness + a codepoint registry discipline. None of the four load-bearing amendments are expensive to add now. All four would be expensive to retrofit post-v1-beta. The most-load-bearing finding is Amendment 3 — without it, you can have two distinct logical permission grants that produce identical AAD bytes, which is the kind of failure mode that has historically taken down JOSE/JWT and TLS parser implementations. Recommend dispatching a focused 4th reviewer to stress-test Amendments 3-6 against each other for composition errors, then ratify all eight amendments as load-bearing-final."

---

## 7. Citations

### 7.1 Standards + drafts (primary references)

- **RFC 9180** Hybrid Public Key Encryption. §4.1 labeled_extract/expand (load-bearing for Amendment 3); §5.1.1 mode_base sender-auth limitation (load-bearing for Amendment 4); §5.2 AEAD AAD parameter; §7.1.3 DeriveKeyPair; §9.1.1 sender-auth statement; §9.1.2 IND-CCA2 reduction; §9.5 low-entropy PSK warning; §9.7.2 replay considerations (load-bearing for Amendment 5); §10.3 multi-recipient.
- **RFC 9420** Messaging Layer Security (MLS). §5.1 LeafIndex sender-binding pattern; §6.2 Welcome epoch binding; §11.3 tree-of-recipients canonical serialization.
- **RFC 8949** Concise Binary Object Representation (CBOR). §4.2.1 Core Deterministic Encoding; §4.2.2 CTAP2 Canonical CBOR (the stricter form recommended for crypto-binding contexts).
- **RFC 8439** ChaCha20 + Poly1305. §2.8 AAD semantics.
- **RFC 7515** JWS. §5.1 protected-header binding (historical lesson on alg-binding into protected payload).
- **FIPS 203** Module-Lattice-Based Key-Encapsulation Mechanism. §6.1 KeyGen; §6.3 Decap implicit-rejection (load-bearing for Amendment 6).
- **draft-connolly-cfrg-xwing-kem-10** X-Wing KEM.
- **draft-irtf-cfrg-concrete-hybrid-kems-03**.
- **draft-ietf-hpke-pq-04** Post-Quantum Hybrid HPKE.
- **draft-sfluhrer-cfrg-ml-kem-security-considerations-04** (Nov 2025 IRTF multi-org WG document) — ML-KEM operational deployment considerations.

### 7.2 Academic papers

- **Bernstein, Persichetti.** "One Time is Enough: Chosen-Ciphertext Side-Channel Attack on ML-KEM Cryptosystems." IACR ePrint 2024/2051 ([eprint.iacr.org/2024/2051](https://eprint.iacr.org/2024/2051)). **Load-bearing for Amendment 6.**
- **Arriaga, Barbosa, Boyen.** "Tempo: ML-KEM to PAKE Compiler Resilient to Timing Attacks." IACR ePrint 2025/1399 ([eprint.iacr.org/2025/1399](https://eprint.iacr.org/2025/1399)). Background.
- **Bellare, Rogaway.** "The exact security of digital signatures — How to sign with RSA and Rabin." CRYPTO 1996. Foundational on injective domain encoding (Amendment 3).
- **Barbosa, Connolly, Diniz, Kahl, Krämer.** "X-Wing: The Hybrid KEM You've Been Looking For." IACR CIC Vol. 1 No. 1, 2024.
- **Cramer, Shoup.** "Design and Analysis of Practical Public-Key Encryption Schemes Secure against Adaptive Chosen Ciphertext Attack." SIAM J. Comput. 2003. HPKE proof template.
- **Hofheinz, Hövelmanns, Kiltz.** "A Modular Analysis of the Fujisaki-Okamoto Transformation." TCC 2017. ML-KEM CCA reduction.
- **PQShield.** "Formally verifying AVX2 rejection sampling for ML-KEM" ([pqshield.com/formally-verifying-avx2-rejection-sampling-for-ml-kem/](https://pqshield.com/formally-verifying-avx2-rejection-sampling-for-ml-kem/)).

### 7.3 Folklore + production references

- **Filippo Valsorda.** "age and Authenticated Encryption" ([words.filippo.io/age-authentication](https://words.filippo.io/age-authentication/)). Replay-window considerations for file-encryption.
- **Cryptographic Doom Principle** (Moxie Marlinspike). Foundational on encryption-then-MAC ordering hazards.
- **Verification Theatre (Nadim Kobeissi)** ([symbolic.software/blog/2026-02-05-cryspen/](https://symbolic.software/blog/2026-02-05-cryspen/)) — "don't use hpke-rs"; verification-claims-vs-reality.
- **JOSE/JWT alg-confusion history** — multiple CVEs from 2015-2024 illustrating Amendment 1 + Amendment 3 + Amendment 8 failure modes.
- **TLS renegotiation 2009** (CVE-2009-3555) — historical example of unbound-AAD attack class addressed by Amendment 1.

### 7.4 Benten internal references

- `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — first review.
- `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` — second review with Amendments 1+2.
- `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae` — §6 + §7 wire formats; §14.1 origin Option F+ pitch.
- `phase-4-meta-core/cryptographer-review-bird-of-prey @ 36afe06b` — Inv-15 origin.
- `phase-4-meta-core/encrypt-to-recipient-review-cryptographer @ 791c8d17` — Option B Conditional-GO baseline.
- `docs/INVARIANT-COVERAGE.md` Inv-15 (registered) + Inv-16 mint pending.
- `docs/SECURITY-POSTURE.md` Compromise #30 + Compromise #31.
- CLAUDE.md baked-in #5 (crypto-agility); #15 (v1-beta gate); #17 (deployment-shapes); #18 (authority-isolation vs confidentiality-isolation).
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` discipline.

---

## 8. Summary handoff to orchestrator (200 words)

**Top-line:** **CONCUR-WITH-CALIBRATION** on §6.2-with-Amendments-1+2 as the right *direction*, but **DISAGREE** that Amendments 1+2 alone make it load-bearing-final. I found four substantive issues that survive the approved design + propose four additional load-bearing amendments (3-6) + two minor ones (7-8). **Most-substantive attack:** Amendment 3's canonical-serialization length-injectivity collision (§2.1) — a concrete byte-identical collision between two semantically distinct `BindingContext` values when variable-length fields aren't length-prefixed. This is the same failure-mode class that produced JWT alg-confusion CVEs and TLS-parser-ambiguity bugs historically. The outcome: §6.2 design DIRECTION is right; Amendments 1+2 alone are necessary-but-insufficient; adding Amendments 3+4+5+6 closes the gaps with low cost now vs high retrofit cost post-v1-beta-freeze. **Residual concern warranting another reviewer:** the cross-amendment composability of MY proposed Amendments 3-6 against each other — I have HIGH confidence on each individually, MEDIUM-HIGH on the interaction. Recommend dispatching a focused 4th reviewer to stress-test Amendments 3-6 composition before ratifying all 8 as load-bearing-final. Also: multi-stanza HpkeMultiBase variant (deferred per second-opinion §2.3) deserves a multi-recipient-HPKE-specialist review when designed (§2.7).
