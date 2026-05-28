# M2 — MembershipSet Primitive Design

**Pipeline:** MembershipSet unification specialist panel; M2 of 5 (M3 amendment-transformation; M4 red-team; M5 CGKA-candidate-survey; M6 transport-configurability).
**Author:** orchestrator-dispatched specialist on branch `phase-4-meta-core/membership-set-m2-primitive-design`.
**Tree-state at start:** `main` HEAD `2172cb6d`, worktree clean.
**Primary inputs (read in full):**
- M1a multi-device-sync cataloger @ `3618e051` (452 LOC) — 4-identity-concept tree + load-bearing complication.
- M1b Atrium-membership-sharing cataloger @ `1816ea60` (658 LOC) — TENTATIVE STRONG-YES; 8/12 fit; 2 caveats; 2 do NOT fit.
- M1c key-management cataloger @ `50eb901d` (886 LOC) — K_principal STUB; path-tagged K(N); §10.3 4-site recurring shape.

**Posture.** Concrete Rust + canonical CBOR + threat-model + composition. NOT plan-doc surgery — that's M3's lane. NOT red-team — M4. NOT CGKA — M5. NOT transport — M6.

---

## §1 — Executive recommendation + confidence

**Recommendation.** Adopt MembershipSet as a **typed-variant primitive** (`enum MembershipSetKind { Atrium, DeviceMesh, SingleDevice }`) with one shared struct shell, NOT a pure-unification single-shape. Place in a NEW workspace crate `benten-membership` (15th crate) that depends only on `benten-id` + `benten-crypto-suite` + `benten-core`. The primitive UNIFIES the 4 sites M1c §10.3 identified (multi-device-key-wrap + K_Atrium-distribution + Drop-bundle-to-recipients + cross-Atrium-federation) under one shared-key + multi-stanza-HPKE + FORK-ONLY-rotation cryptographic discipline. It does NOT unify capability-revocation, sync-zone-namespacing, freshness-window, K(N) derivation, DAK, or hybrid-sig keypairs — those stay distinct (per M1b §9.2 and M1c §10.2).

**Confidence per finding (full breakdown §10):**
- **HIGH (~85-90%)** on the typed-variants choice (M1a 4-identity-concepts + M1b's 2 non-fits + M1c's per-DID-keypair non-fits triangulate to "kind matters").
- **HIGH** on the recommended struct shape (derived from union of M1b §9.1 sketch + M1c §10.3 recurring shape + M1a §8 pattern table).
- **HIGH** on the 7-operation set (mirrors M1b §9 12-op fit table after dropping the 2 NOT-FITS and the 1 device-config caveat).
- **MED-HIGH (~75-80%)** on K_Set generalization across the 3 kinds (cleanest under FORK-ONLY; rotation policy varies; M1c §11 Q1 + Q7 surfaces real open questions M3/M5 must reconcile).
- **MED-HIGH** on plaintext_cid DUAL-CID-NOT-TRIPLE simplification (Ben's catch; cataloger surfaces support; M3/M4 must verify against U18 / L9-A2 contract).
- **MED (~60-65%)** on recursive composition (Garden ← Atrium ← user-Mesh ← devices). M1a §3.5.5 + M1b §3.5 + M1c §10.1 all want it; the v1-beta wire-format must NOT foreclose, but recursive composition body lands Phase 7.
- **MED** on Inv-15 cross-language rule-mirror (TS side needs companion enum; cite-drift discipline reach).

**Why typed-variants over pure-unification.** Pure-unification (one shape, polymorphic over member kind) would:
1. Collapse the load-bearing 4-identity-concepts asymmetry (CLAUDE.md #18) that Phase-4-Foundation R1 ratified — devices are attested sub-identities; plugin-DIDs are NOT; user-DIDs are trust anchors; agent-DIDs are ephemeral.
2. Erase the per-kind threat-model differences (Atrium = multi-user insider-threat; DeviceMesh = single-user all-mine; SingleDevice = degenerate). Erasure forces every MembershipSet operation to dispatch on policy fields at runtime, replacing structural type-system invariants with runtime predicates — exactly the pattern §3.5j (stable clippy gate) + Inv-15 (typed dispatch never silent-fallback) discourage.
3. Force K_Set source uniformly (CSPRNG-random vs derived-from-DAK vs derived-from-K_principal), losing the per-kind invariant that M1c §10.2 surfaces.
4. Push the M1b 2 non-fits (register_zone + freshness_window) into the polymorphic shape, OR force a parallel "AtriumHandle config" surface anyway — net negative.

Typed-variants preserve type-system invariants per kind while sharing the 7-operation API surface + the shared-key + multi-stanza-HPKE distribution discipline. This is the same shape `Scope::Hashes | RestrictedSelector` chose (EXACTLY-N arm enum, NOT `#[non_exhaustive]`, per V1-FROZEN-INTERFACE §15.c) — a third kind = HALT-AND-SURFACE-TO-BEN.

---

## §2 — MembershipSet struct + operations (Rust + canonical CBOR)

### §2.1 Public type surface

```rust
//! crates/benten-membership/src/lib.rs

use benten_core::Cid;
use benten_crypto_suite::{
    aead::AeadEnvelope,
    codepoint::{CipherSuiteCodepoint, SigCodepoint},
    cipher_suite::WrappedKey,
};
use benten_id::{Did, DeviceAttestation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Content-addressed identifier of a MembershipSet snapshot (canonical CBOR + BLAKE3).
/// Per Inv-15: identity = canonical-payload-CID; CID changes on member-set / policy / generation rotation.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MembershipSetId(pub Cid);

/// EXACTLY-3 arms; NOT #[non_exhaustive]; 4th = HALT-AND-SURFACE-TO-BEN per §15.c discipline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MembershipSetKind {
    /// Multi-user social unit. Members are user-DIDs (principal grain).
    /// K_Set = K_Atrium (CSPRNG-random on create; FORK-ONLY rotate).
    Atrium = 0,
    /// Single-user's devices under one user-DID. Members are device-DIDs.
    /// K_Set = K_principal (real per-DID secret store; distributed via multi-device-key-wrap).
    DeviceMesh = 1,
    /// Degenerate singleton: one device, no peer members.
    /// K_Set = K_principal derived locally from DAK-protected vault; never distributed.
    SingleDevice = 2,
}

/// A member's public-key material + kind-dependent envelope/attestation.
/// All variants carry an HPKE-recipient pubkey (per M1c §10.3 distribution shape).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberKey {
    /// Member is a user-DID (Atrium). HPKE pubkey is the user-DID's resolved hybrid encryption key.
    /// `attestation` is None — Atrium members are user-anchored; no parent attestation chain.
    UserDid {
        did: Did,
        hpke_pubkey: HybridKemPubkey,
        sig_pubkey: HybridSigPubkey,
    },
    /// Member is a device-DID (DeviceMesh). HPKE pubkey is the device's hybrid encryption key.
    /// `attestation` is the DeviceAttestation envelope from parent (user-DID or higher-trust device).
    DeviceDid {
        did: Did,
        hpke_pubkey: HybridKemPubkey,
        sig_pubkey: HybridSigPubkey,
        attestation: DeviceAttestation, // CLAUDE.md #17 4-dim CapabilityEnvelope inside
    },
    /// Member is local-only (SingleDevice singleton). No distribution; degenerate.
    LocalDevice {
        did: Did,
    },
}

/// Generalization of AtriumPolicy. Per-kind variants reflect the threat-model differences (§4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipSetPolicy {
    /// Monotonic; admin-update increments; defends admin-replay (per AtriumPolicy `policy_version`).
    pub policy_version: u32,
    /// Per-kind specifics; #[non_exhaustive] per-arm body to allow additive growth.
    pub kind_policy: KindPolicy,
    /// Default credential-validity window per MembershipSetPolicy (D3 refresh_required pin).
    pub refresh_required_secs: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum KindPolicy {
    /// Atrium-specific: admin-DID + threshold-admin reserved.
    Atrium {
        admin_pubkey: HybridSigPubkey,
        // RESERVED for Phase-N+1: threshold_admin: Option<ThresholdMultiSig>,
    },
    /// DeviceMesh-specific: parent user-DID is THE admin; per-device CapabilityEnvelope ceiling.
    DeviceMesh {
        user_did: Did,
        envelope_ceiling: CapabilityEnvelope, // CLAUDE.md #17 4-dim
    },
    /// SingleDevice degenerate: trivially admin-self.
    SingleDevice {
        device_did: Did,
    },
}

/// The cryptographic shared key for this MembershipSet.
/// Stored encrypted-under-DAK in Layer-A vault per M1c §9.2 + §4.4.
#[derive(Clone, Debug, Serialize, Deserialize)] // NB: Zeroize-on-drop via inner newtype
pub struct K_Set {
    /// CSPRNG-random (Atrium) | per-DID-vault (DeviceMesh) | DAK-derived (SingleDevice)
    pub key_material: AeadKeyMaterial,
    /// Cipher-suite codepoint per V1-FROZEN-INTERFACE.md item 6
    pub codepoint: CipherSuiteCodepoint,
    /// Monotonic generation counter; FORK-ONLY rotation; M1c §3.1 L9-A4 amendment
    pub generation: u32,
}

/// The MembershipSet primitive. Content-addressed via canonical CBOR + BLAKE3 per Inv-15.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MembershipSet {
    /// Wire-format version; codepoint-additive per Inv-15
    pub version: u8, // = 0x01

    /// Kind discriminator (Atrium / DeviceMesh / SingleDevice)
    pub kind: MembershipSetKind,

    /// Content-addressed self-identity; computed lazily from canonical CBOR of remaining fields.
    /// Excluded from canonical CBOR (would be self-referential).
    #[serde(skip)]
    pub id: Option<MembershipSetId>,

    /// Member set; BTreeSet for canonical iteration order (CBOR-canonical).
    /// Use Vec<MemberKey> if order semantics matter (member-order-as-stanza-index).
    pub members: Vec<MemberKey>,

    /// The shared cryptographic key.
    /// IN-MEMORY ONLY when serialized FOR WIRE; on-wire it's per-member multi-stanza-HPKE-wrapped.
    /// At-rest serialization carries it encrypted-under-DAK.
    /// Distinguish on-wire (NEVER includes raw K_Set) vs at-rest (DAK-encrypted) via serde context.
    #[serde(skip_serializing)] // hidden from default to_dag_cbor; carried via on-wire wrap path
    pub shared_key: K_Set,

    /// Generalized policy (admin / envelope-ceiling / refresh-window)
    pub policy: MembershipSetPolicy,

    /// Parent set for forkability semantics. None = root set; Some(cid) = fork-of-parent.
    /// Per Ben 2026-05-27 forkability ratification (M1b §7.1).
    pub parent_membership_set_id: Option<MembershipSetId>,

    /// Hybrid-Logical-Clock at create/fork-event time; ordering across membership changes.
    pub created_at_hlc: BentenHlc,

    /// Membership-attestation chain proving the set was constructed by an authorized party.
    /// For Atrium: signed by admin_pubkey. For DeviceMesh: signed by user_did. For SingleDevice: self-signed.
    pub membership_attestation: MembershipAttestation,
}

/// Signed proof that this MembershipSet snapshot exists at this generation.
/// Replays M1a §8 "membership-claim primitive" question — answer = MembershipAttestation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MembershipAttestation {
    pub set_id: MembershipSetId, // self-referential AT VERIFY time; bound via BLAKE3 over (kind, members, policy, generation, parent, created_at_hlc, signer_did)
    pub signer_did: Did,
    pub signature: HybridSignature, // codepoint-dispatched per benten-crypto-suite SignatureSuite
    pub sig_codepoint: SigCodepoint,
}
```

### §2.2 Canonical CBOR (wire-format) shape

Per V1-FROZEN-INTERFACE §15 + Inv-15 + L9-A2 dual-CID discipline. The CBOR shape that hashes to `MembershipSetId`:

```cbor
; canonical CBOR map; keys lexicographic; integer values where possible
{
  "v":     1,                                ; u8 version
  "kind":  0|1|2,                            ; MembershipSetKind discriminator
  "mem":   [                                 ; canonical-sorted by member.did.as_bytes()
              {"did": "did:key:z...", "hpke": h'<bytes>', "sig": h'<bytes>',
               "att": <DeviceAttestation CBOR | null>},
              ...
           ],
  "pol":   {                                 ; MembershipSetPolicy
              "pv": <u32>,                   ; policy_version
              "kp": <KindPolicy CBOR>,       ; kind-specific (admin_pubkey | user_did | self)
              "rr": <u64 | null>             ; refresh_required_secs
           },
  "gen":   <u32>,                            ; K_Set.generation (NOT key material; never on wire raw)
  "cp":    <u16 LE>,                         ; CipherSuiteCodepoint for K_Set
  "par":   <MembershipSetId | null>,         ; parent
  "hlc":   "<physical>:<logical>:<node>",    ; BentenHlc canonical string
  "att":   {                                 ; MembershipAttestation (signed over EVERYTHING ABOVE)
              "signer": "did:key:z...",
              "sigcp":  <u16 LE>,
              "sig":    h'<bytes>'
           }
}
```

`MembershipSetId = BLAKE3(canonical_cbor(..., att excluded))`. The `att` field signs over `BLAKE3(canonical_cbor_minus_att)` per the "binding_sig FIRST, then UCAN scope, then key material" ordering V1-FROZEN-INTERFACE §15.d ratified for AuthorizationGrant. Inv-15 compliance: identity is canonical-payload-CID; authentication is codepoint-dispatched sig; revocation is a separate `system:MembershipSetRevocation` Node referencing this CID by semantic tuple.

**Wire-distribution of K_Set.** K_Set NEVER appears raw on wire. The MembershipSet CBOR shape above does NOT contain the K_Set bytes — only the cipher codepoint + generation. Distribution rides a SEPARATE multi-stanza HPKE envelope:

```cbor
; MultiStanzaHpkeEnvelope (per L9-A1 / U17 + M1c §10.3)
{
  "v":      1,
  "set_id": <MembershipSetId>,            ; binds this distribution to a specific set snapshot
  "gen":    <u32>,                        ; the K_Set.generation being distributed
  "stanzas": [
    {
      "recipient_did": "did:key:z...",    ; per-stanza recipient
      "kem_ct":        h'<bytes>',        ; HPKE-encap ciphertext (MLKEM768-X25519 KEM)
      "aead_ct":       h'<bytes>',        ; HPKE-AEAD wrapping K_Set bytes + AAD-bound to (recipient_did, stanza_index, total_stanzas, set_id, generation)
    },
    ...
  ],
  "issuer_sig": <HybridSignature>         ; signed by admin/user_did/self per kind
}
```

This envelope is what gets distributed via Drop bundles (M1b §6) / iroh-gossip (Q3 §3) / Atrium sync. The AAD construction extends M1c §2.5 per-Recipe layout:

```
"benten-membership:stanza:" || set_id_bytes || generation_le4 || stanza_index_le4 || total_stanzas_le4
```

### §2.3 Why each field

- **`version`**: codepoint-additive per Inv-15; future shape changes are version bumps not breaking changes.
- **`kind`**: typed-variant per §1 rationale; EXACTLY-3 arms.
- **`members: Vec<MemberKey>`**: ORDER matters (member-index ↔ stanza-index in distribution envelope). Canonical ordering = lexicographic by `did.as_bytes()` for CBOR-canonical hashing.
- **`shared_key` (skipped from default serialize)**: separation of wire-distribution from set-identity. The MembershipSetId hashes ONLY non-key material; rotating K_Set generation alone changes `gen` field → changes CID. Stored at rest under Layer-A DAK-encrypted vault per M1c §4.4.
- **`policy.kind_policy`**: per-kind admin shape; preserves M1a 4-identity-concepts distinction (Atrium has admin-DID; DeviceMesh has user-DID-as-admin; SingleDevice is degenerate).
- **`generation`**: tracks K_Set rotation; combined with `parent_membership_set_id` makes fork-vs-rotate distinguishable (fork = new parent_id + reset generation; rotate = same parent_id + increment generation). FORK-ONLY discipline (M1c §9.1) means generation rarely increments outside fork events.
- **`parent_membership_set_id`**: enables fork-tree traversal; "this set forked from that set"; closes M1b §7.4 fork-event-Node requirement structurally.
- **`created_at_hlc`**: HLC-monotonic ordering across membership changes; defends replay; composes with sync layer per M1a §5.6 revocation-drains-before-data discipline.
- **`membership_attestation`**: signed proof of authorized construction; answers M1a §9 Q3 "membership-claim primitive".

---

## §3 — The 3 kinds + instantiation

### §3.1 Atrium

```rust
MembershipSet {
    version: 1,
    kind: MembershipSetKind::Atrium,
    members: vec![
        MemberKey::UserDid { did: alice_did, hpke_pubkey: alice_hpke, sig_pubkey: alice_sig },
        MemberKey::UserDid { did: bob_did,   hpke_pubkey: bob_hpke,   sig_pubkey: bob_sig },
        // ... canonical-sorted by did
    ],
    shared_key: K_Set {
        key_material: AeadKeyMaterial::from_csprng(32, MLKEM768_X25519), // K_Atrium
        codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        generation: 0, // FORK-ONLY rotation per M1c §9.1
    },
    policy: MembershipSetPolicy {
        policy_version: 0,
        kind_policy: KindPolicy::Atrium { admin_pubkey: founder_sig_pubkey },
        refresh_required_secs: None, // AtriumPolicy D3 reserved for Phase-4-Meta-Composing
    },
    parent_membership_set_id: None, // root Atrium; Some(_) for forks
    created_at_hlc: BentenHlc::now(local_node_id),
    membership_attestation: MembershipAttestation::sign(self_partial, &founder_keypair),
}
```

- **K_Set source:** CSPRNG-random (32 bytes); per Q3 Option D §2.6.
- **Distribution:** multi-stanza-HPKE at create (one stanza per founding member) + at member-add (one stanza per new member).
- **Storage:** each member stores K_Atrium under their local DAK-protected Layer-A vault (M1c §9.2). Same K_Atrium across all of one user's devices (those devices share the user's K_principal-protected vault).

### §3.2 DeviceMesh

```rust
MembershipSet {
    version: 1,
    kind: MembershipSetKind::DeviceMesh,
    members: vec![
        MemberKey::DeviceDid {
            did: laptop_device_did,
            hpke_pubkey: laptop_hpke,
            sig_pubkey: laptop_sig,
            attestation: laptop_attestation, // DeviceAttestation::issue(user_kp, ...)
        },
        MemberKey::DeviceDid {
            did: phone_device_did,
            ...
            attestation: phone_attestation,
        },
    ],
    shared_key: K_Set {
        key_material: K_principal_loaded_from_user_vault,
        codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        generation: K_principal_generation, // tracked per M1c L9-A4
    },
    policy: MembershipSetPolicy {
        policy_version: 0,
        kind_policy: KindPolicy::DeviceMesh {
            user_did: alice_user_did,
            envelope_ceiling: CapabilityEnvelope { // CLAUDE.md #17 4-dim minimum
                runs_sandbox: true, holds_zones: Full, online_uptime: AlwaysOn, runs_atrium_peer: true,
            },
        },
        refresh_required_secs: None,
    },
    parent_membership_set_id: None, // user's device-mesh is a root set
    created_at_hlc: ...,
    membership_attestation: MembershipAttestation::sign(self_partial, &user_keypair),
}
```

- **K_Set source:** K_principal (loaded from Layer-A vault at engine-start; M1c §4.2). NOT CSPRNG-random per-set — K_principal is the user's long-lived per-DID secret.
- **Distribution:** multi-device-key-wrap at device-add — HPKE-encap of K_principal to new device's pubkey + attached to the DeviceAttestation envelope. M1a §4.2 step 10 (currently MISSING; lands at F-full Layer-D).
- **Storage:** each device stores its K_principal copy under its OWN DAK-protected Layer-A vault. Same K_principal value, different DAK encryption per device.
- **Note:** M1a §6.1 identity-tree puts CapabilityEnvelope BELOW Device-DID (per-shape minimum). The `envelope_ceiling` in `KindPolicy::DeviceMesh` is the MAXIMUM envelope any member device can claim — per-device envelope is in `attestation.envelope`.

### §3.3 SingleDevice (degenerate singleton)

```rust
MembershipSet {
    version: 1,
    kind: MembershipSetKind::SingleDevice,
    members: vec![
        MemberKey::LocalDevice { did: local_device_did },
    ],
    shared_key: K_Set {
        key_material: K_principal_loaded_from_user_vault,
        codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        generation: K_principal_generation,
    },
    policy: MembershipSetPolicy {
        policy_version: 0,
        kind_policy: KindPolicy::SingleDevice { device_did: local_device_did },
        refresh_required_secs: None,
    },
    parent_membership_set_id: None,
    created_at_hlc: ...,
    membership_attestation: MembershipAttestation::self_signed(local_device_keypair),
}
```

- **K_Set source:** K_principal (same as DeviceMesh; just degenerate to one member).
- **Distribution:** NEVER distributed — no peer to send to. The whole multi-stanza-HPKE machinery is a no-op for SingleDevice (members.len() == 1 → 1 stanza targeting self → redundant).
- **Forward path:** SingleDevice → DeviceMesh is a `MembershipSet::promote_to_device_mesh(new_member)` transition that minimally changes `kind` discriminator + adds the new member + issues the distribution stanza. Same K_Set survives the promotion (no K_principal rotation).

### §3.4 Why this is collapsible-but-typed

All 3 kinds share:
- The 7-operation API (§4).
- The on-wire multi-stanza-HPKE distribution shape (with SingleDevice as a degenerate no-op).
- The Inv-15 identity-via-canonical-CBOR + signed-attestation chain.
- The parent_membership_set_id forkability semantic.
- The K_Set + generation rotation discipline.

They differ in:
- K_Set provenance (CSPRNG vs vault-loaded vs degenerate).
- Policy.kind_policy (admin-DID vs user-DID-as-admin vs self).
- Member-key variant (UserDid vs DeviceDid with attestation vs LocalDevice).
- Distribution surface (peer-to-peer vs per-device-wrap vs no-op).

This is exactly the shape Rust's enum + struct composition expresses cleanly. Type-system enforces per-kind invariants (a `KindPolicy::Atrium` cannot accidentally carry a `user_did` field; `MemberKey::UserDid` cannot accidentally carry a `DeviceAttestation`).

---

## §4 — Operations (signature + semantics + per-kind specialization)

```rust
impl MembershipSet {

    /// Create a new MembershipSet with one founding member.
    /// Returns the set + the first MembershipAttestation.
    pub fn create(
        kind: MembershipSetKind,
        founder: MemberKey,
        policy: MembershipSetPolicy,
        founder_signing_keypair: &Keypair,
    ) -> Result<Self, MembershipError> { /* ... */ }

    /// Add a member; returns the new MembershipSet snapshot + the distribution envelope
    /// (multi-stanza-HPKE) carrying K_Set wrapped to the new member's HPKE pubkey.
    /// Per FORK-ONLY discipline: K_Set is NOT rotated. The new member gets the existing K_Set.
    pub fn add_member(
        &self,
        new_member: MemberKey,
        admin_signing_keypair: &Keypair, // Atrium: admin; DeviceMesh: user; SingleDevice: REJECT (use promote_to_device_mesh)
    ) -> Result<(MembershipSet, MultiStanzaHpkeEnvelope), MembershipError> { /* ... */ }

    /// Remove a member; rotation_policy determines whether K_Set rotates.
    ///   - RotationPolicy::ForkOnly (default; Atrium/DeviceMesh): K_Set unchanged;
    ///     removed member retains K_Set for past-content access (Ben 2026-05-27 forkability).
    ///   - RotationPolicy::AdminKickEpoch (opt-in; future CGKA-LITE landing per M5):
    ///     mint new K_Set generation + redistribute to remaining members.
    ///     M5 specialist owns whether this lands at v1-beta or post-v1.
    pub fn remove_member(
        &self,
        target: &Did,
        rotation_policy: RotationPolicy,
        admin_signing_keypair: &Keypair,
    ) -> Result<(MembershipSet, Option<MultiStanzaHpkeEnvelope>), MembershipError> { /* ... */ }

    /// Encrypt a payload to this MembershipSet using multi-stanza HPKE (Layer-C drop).
    /// One stanza per member; AAD-binds (recipient_did, stanza_index, total, set_id, generation).
    /// Returns the EncryptedEnvelope per L9-A1 / U17.
    /// SingleDevice degenerate: returns AEAD-wrap-under-K_Set directly (no HPKE — local-only).
    pub fn encrypt_to_set(
        &self,
        payload: &[u8],
        sender_signing_keypair: &Keypair,
    ) -> Result<EncryptedEnvelope, MembershipError> { /* ... */ }

    /// Compute the dedup-blinded plaintext CID per Q3 Option D §2.4.
    ///   plaintext_cid_set = HMAC-SHA256(K_Set, BLAKE3(canonical_bytes(plaintext)))[..16]
    /// Per-kind: Atrium uses K_Atrium; DeviceMesh uses K_principal; SingleDevice still computes
    /// (for plaintext_cid_local↔plaintext_cid_set duality preservation; see §8).
    pub fn dedup_blind_cid(&self, plaintext: &[u8]) -> [u8; 16] { /* ... */ }

    /// Fork: create a child MembershipSet with a NEW K_Set (CSPRNG for Atrium kind;
    /// K_principal_for_new_user for DeviceMesh kind; reject for SingleDevice).
    /// `new_member_set` = the founding member set of the child fork; subset of self.members
    /// in canonical fork semantics (Ben 2026-05-27: "Atrium-as-forkable not messaging-leave-forgets").
    /// Parent unchanged; child.parent_membership_set_id = Some(self.id).
    pub fn fork(
        &self,
        new_member_set: Vec<MemberKey>,
        founder_signing_keypair: &Keypair,
    ) -> Result<(MembershipSet, MultiStanzaHpkeEnvelope), MembershipError> { /* ... */ }

    /// Update policy (admin-signed). Increments policy_version monotonically.
    /// Atrium: admin_pubkey-signed. DeviceMesh: user_did-signed. SingleDevice: self-signed.
    pub fn update_policy(
        &self,
        new_policy: MembershipSetPolicy,
        admin_signing_keypair: &Keypair,
    ) -> Result<MembershipSet, MembershipError> { /* ... */ }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RotationPolicy {
    /// Default; preserves forkability semantics; removed member retains past-content access.
    ForkOnly,
    /// Opt-in; rotates K_Set + redistributes; future CGKA-LITE-or-full landing per M5.
    AdminKickEpoch,
}
```

**Per-operation per-kind specialization summary:**

| Op | Atrium | DeviceMesh | SingleDevice |
|---|---|---|---|
| `create` | CSPRNG K_Atrium + admin-DID set | K_principal-load + user-DID-as-admin | K_principal-load + self-as-admin |
| `add_member` | multi-stanza HPKE wrap of K_Atrium to new member | multi-device-key-wrap of K_principal to new device | REJECT (use promote_to_device_mesh) |
| `remove_member` | ForkOnly default; AdminKickEpoch opt-in (post-v1 per M5) | ForkOnly default (revoked device retains K_principal copy until DAK rotation) | REJECT (degenerate) |
| `encrypt_to_set` | multi-stanza HPKE Layer-C | multi-device HPKE-wrap OR local Layer-B AEAD if member == self | local Layer-B AEAD only |
| `dedup_blind_cid` | HMAC-SHA256(K_Atrium, ...) | HMAC-SHA256(K_principal, ...) | HMAC-SHA256(K_principal, ...) |
| `fork` | CSPRNG new K_Atrium for fork; child set | reject (DeviceMesh fork = user-DID fork; out-of-scope) | reject (degenerate) |
| `update_policy` | admin-signed | user-DID-signed | self-signed |

**Operations NOT on MembershipSet (per M1b §9.2 + §6):**
- `register_zone` — sync-scope namespace; orthogonal cross-cutting control; stays on AtriumHandle.
- `set_envelope_freshness_window` — replay-defense tunable; per-handle; stays on AtriumHandle.
- Capability revocation (`system:CapabilityRevocation` Node + UCAN chain-walker) — separate primitive consuming MembershipSet for scope; stays in `benten-caps`.
- K(N) derivation — derivable function not stored membership state; stays in `benten-crypto-suite::structural_kdf`.
- DAK derivation + Layer-A vault — per-device per-user; stays in F-full Layer-D module.
- Hybrid-sig keypairs — per-DID; stays in `benten-id`.

---

## §5 — Threat models per kind

### §5.1 Atrium threat model (multi-user)

**Threats:**
- T-A1 **Insider-as-adversary.** One member is malicious against other members (Bob writes garbage; admin needs to kick Bob).
- T-A2 **Admin compromise.** Founder/admin key is stolen → adversary mints new InviteEnvelopes / changes policy.
- T-A3 **Admin-kicked-but-still-holds-K_Atrium.** Per Ben 2026-05-27 forkability: this is BY DESIGN. The kicked member can still Open past Drops. M1b §7 documents.
- T-A4 **Storage-host equality oracle** (Q3 Option D motivation). External storage host (iroh relay) sees blinded plaintext_cid_set; cannot link to user-namespace plaintext.
- T-A5 **Replay of stale InviteEnvelope.** Adversary replays an old InviteEnvelope to gain access. Defense: policy_version monotonic + HLC-temporal-ordering at handshake.
- T-A6 **Sybil member-add.** Admin compromise → adversary adds adversary-controlled members + sees future content. Defense: admin-rotation seam (M1b §8.2) + threshold-admin reserve.

**Design implications:**
- `KindPolicy::Atrium { admin_pubkey }` is FIRST-CLASS — defends T-A1/T-A2/T-A6.
- `dedup_blind_cid` uses K_Atrium — defends T-A4.
- `policy_version` monotonic + `created_at_hlc` — defends T-A5.
- T-A3 is ACCEPTED PER POLICY (Compromise #31 reach) + documented; mitigation = fork on perceived risk + rotate K_Atrium at fork.

### §5.2 DeviceMesh threat model (single-user)

**Threats:**
- T-D1 **Device theft.** Lost/stolen device gives adversary the device's DAK-protected Layer-A vault. If adversary brute-forces DAK (weak password), gets K_principal + can act as user.
- T-D2 **Device-supply-chain.** Manufacturer / OS / firmware compromise; pre-installed malware reads DAK on entry. Out-of-scope for v1-beta; documented.
- T-D3 **K_principal extraction.** Engine compromise on one device exposes K_principal in memory → adversary can decrypt ALL of user's content (all Atriums, all Layer-B Nodes).
- T-D4 **Compromised device as Atrium member.** A compromised user-device sees Atrium-content via the user's Atrium membership; from the Atrium's perspective, the user is the unit-of-trust, not the device. Defense out-of-scope at Atrium layer; lands at per-device cap-envelope ceiling (M1a §3.5.4) for shapes (b)/(c) browser/edge devices.

**Design implications:**
- `KindPolicy::DeviceMesh { user_did, envelope_ceiling }` — user-DID is THE admin (no separate admin grain). envelope_ceiling caps any device's claim.
- `MemberKey::DeviceDid { attestation }` — CapabilityEnvelope per-device per CLAUDE.md #17 shape (a)/(b)/(c). Defends T-D4 partially (a thin-client device can be in the mesh but cannot claim runs_atrium_peer=true).
- Insider-threat is N/A (all devices are "you"; one device's compromise = your compromise).
- T-D1 defense: DAK strength (Argon2id tiering per M1c §5.1) + cross-platform credential-store integration (keyring-core) + per-device DAK-rotation discipline.
- Forkability + DeviceMesh: NOT FIRST-CLASS at v1-beta (per §4 fork: rejects for DeviceMesh). A user does not "fork" their device-set; they add/remove devices.

### §5.3 SingleDevice threat model (degenerate)

**Threats:**
- T-S1 **DAK brute-force.** Local-only attacker (e.g. malware) on the device guesses password → opens Layer-A vault. Same defense as T-D1 (Argon2id tiering).
- T-S2 **Local malware.** Reads K_principal from engine memory at runtime. Out-of-scope at v1-beta (process-isolation + OS-sandboxing rely on platform).
- T-S3 **Physical theft + no-DAK.** User chose no-password / weak-password / biometric-only. Trade-off documented; user-configurable; Phase-4-Meta-Composing.

**Design implications:**
- `MemberKey::LocalDevice { did }` — minimal; no HPKE pubkey, no attestation, no peer.
- No insider threats (singleton); no admin threats (self-admin); no Atrium threats.
- Threat surface = LOCAL ONLY = DAK + OS + hardware.

### §5.4 Type-system enforcement of per-kind threat-model differences

The per-kind threat-model differences manifest in the type system via `KindPolicy` enum dispatch:

```rust
match self.policy.kind_policy {
    KindPolicy::Atrium { admin_pubkey } => {
        // T-A2/T-A6 defense at this seam
        require_signature_by(admin_pubkey, payload, sig)?;
    }
    KindPolicy::DeviceMesh { user_did, envelope_ceiling } => {
        // T-D4 defense at this seam
        let claimed_envelope = member_attestation.envelope;
        if !envelope_ceiling.contains(&claimed_envelope) {
            return Err(EnvelopeCeilingViolated);
        }
        require_signature_by_did(user_did, payload, sig)?;
    }
    KindPolicy::SingleDevice { device_did } => {
        require_signature_by_did(device_did, payload, sig)?; // self
    }
}
```

Pure-unification (one shape) would force all 3 defenses to live in runtime predicates over a shared field-set. Typed-variants make per-kind invariants compile-time-enforced. This composes with §3.5g cross-language rule-mirror — the TS-side mirror `MembershipSetKind` enum lands as a class hierarchy or discriminated union.

---

## §6 — Composition with 4-identity-concepts tree (CLAUDE.md #18)

M1a §6.1 names this as the most-load-bearing modeling question. The 4 identity concepts:

```
Content-CID            (what the plugin/data IS — NOT a DID)
   ↓
Peer-DID signature     (provenance signature; not a separate DID grain — it's a sig BY one of the 4 DIDs below)
   ↓
Plugin-DID             (UCAN audience handle; OsRng-minted at install; NOT attested)
   ↓
User-DID               (trust anchor; per-user; the principal-DID grain)
   ↓
Device-DID             (attested sub-identity per CLAUDE.md #17; under user-DID via DeviceAttestation)
   ↓
CapabilityEnvelope     (per-shape minimum; runs_sandbox / holds_zones / online_uptime / runs_atrium_peer)
```

**MembershipSet composition per DID-grain:**

### §6.1 user-DID → MembershipSet member-of-Atrium

A user-DID is a `MemberKey::UserDid { ... }` IN an `Atrium`-kind MembershipSet. A user-DID OWNS a `DeviceMesh`-kind MembershipSet (a user IS the admin of their own DeviceMesh; the user-DID appears in `KindPolicy::DeviceMesh.user_did` field, NOT in `members`).

**Recursive composition (long-horizon Phase 7 Garden):** A Garden's MembershipSet has user-DID-grain members where each user-DID is the admin of a DeviceMesh; effectively `MembershipSet<Atrium>` contains user-DIDs that each project to a `MembershipSet<DeviceMesh>`. This is set-of-sets at the LOGICAL layer, but the WIRE format stays flat — Garden member-set is `Vec<MemberKey::UserDid>`, and each user-DID's DeviceMesh is a SEPARATE MembershipSet known only to that user.

**Forward Phase-7 extension** (M3 plan-doc lane): Garden = `MembershipSet` with a NEW `kind: Garden` variant added when ratified. v1-beta MUST NOT foreclose. The enum is EXACTLY-3 today; future ratification of `Garden` = 4th arm = HALT-AND-SURFACE-TO-BEN per §15.c discipline = explicit Ben decision = NOT a wire-format break (additive codepoint).

### §6.2 device-DID → MembershipSet member-of-DeviceMesh (NOT direct-member-of-Atrium)

A device-DID is a `MemberKey::DeviceDid { ..., attestation }` IN a `DeviceMesh`-kind MembershipSet. **device-DIDs are NEVER direct Atrium members.** The user-DID is the Atrium member; the user's devices are projections of their identity.

This matches M1a §6.1 trust-tree exactly: device-DID is attested-sub-identity of user-DID; Atrium membership is at user-DID grain. The handshake wire `HandshakeFrame { peer_did, device_did }` (M1a §2.2) carries BOTH because the handshake authenticates the device-DID locally + the user-DID at the Atrium grain. The MembershipSet abstraction preserves this split: the Atrium MembershipSet's member-set is user-DIDs; the device-DID identifies WHICH of the user's devices sent THIS handshake.

**Critical implication for cap-envelope:** when device-B (with `runs_atrium_peer=false` envelope, i.e. browser thin-client) sends a write into the Atrium, the receiver verifies (a) device-B's attestation chain → user-A signature; (b) device-B's envelope ceiling allows the action (e.g. SUBSCRIBE may require `online_uptime` ≥ minimum); (c) the user-A's membership in the Atrium MembershipSet. This is THREE checks at three grains — collapsing them into one would lose the M1a-flagged 4-identity-concepts distinction.

### §6.3 agent-DID → NOT a MembershipSet member (per M1b §3.5)

Per M1b §3.5.4 + §3.5: AI-agent-as-Atrium-member is NOT a planned shape at HEAD; AI agents are plugins under the user's principal. The `AttributionFrame.actor_cid` decoupling (Option-A ratification) carries agent identity in the attribution layer, NOT in the membership layer.

**M2 recommendation:** **agent-DIDs are NOT MemberKey variants.** An agent runs on a device that is a member of a DeviceMesh; the agent's actions are attributed via `actor_cid` in `AttributionFrame`, not via Atrium membership. If a future ratification mints AI-agent-as-first-class-Atrium-member, that's a 4th `MemberKey` variant addition (additive codepoint — non-breaking).

### §6.4 plugin-DID → NOT a MembershipSet member (per CLAUDE.md #18)

Per CLAUDE.md #18 baked-in: plugin-DID has NO attestation chain (no parent_did). It's a UCAN audience handle, not a principal. **plugin-DIDs are NEVER MemberKey variants.** A plugin's authority composes with capability grants (UCAN-bound to plugin-DID), not with membership.

### §6.5 Summary: per-grain composition

| DID grain | MembershipSet role |
|---|---|
| user-DID | `MemberKey::UserDid` in `Atrium`; admin in `KindPolicy::DeviceMesh.user_did` |
| device-DID | `MemberKey::DeviceDid` in `DeviceMesh`; `MemberKey::LocalDevice` in `SingleDevice`; NEVER direct-Atrium member |
| agent-DID | NOT a MemberKey variant (attribution-layer only via `actor_cid`); future ratification = additive |
| plugin-DID | NEVER a MemberKey variant (UCAN audience handle; no attestation) |
| content-CID | NOT a DID; content-addressed; orthogonal |

This preserves the 4-identity-concepts asymmetry M1a §6 surfaces, without collapsing them. The MembershipSet primitive is principal-grain-aware (user-DID for Atrium; device-DID for DeviceMesh) via its `kind` discriminator + `MemberKey` variants.

---

## §7 — M1b non-fitting ops + caveats — handling

M1b §9 identified 2 ops that do NOT fit + 2 caveats. Per §1 typed-variants discipline, all stay OUTSIDE the MembershipSet primitive:

### §7.1 `register_zone(zone: &str)` — orthogonal cross-cutting

Per M1b §9.2 #1: "Orthogonal to membership; multiple zones per Atrium; multiple Atriums per zone-namespace." **Disposition: STAYS on `AtriumHandle`.**

Rationale: zones are a sync-scope namespacing concept; they exist independently of who's in the set. An Atrium can have N zones (one per shared subgraph). The zone-name is local to the AtriumHandle config. Folding zone-config into MembershipSet would make MembershipSet a sync-runtime configuration object, not a membership primitive.

**Composition pattern:** `AtriumHandle::open(MembershipSet<Atrium>, vec!["zone1", "zone2"])` — the MembershipSet is the trust-and-key boundary; zones are the sync-scope projections inside.

### §7.2 `set_envelope_freshness_window(secs)` — per-handle replay tunable

Per M1b §9.2 #2: "Per-Atrium-handle setting; NOT a per-member setting." **Disposition: STAYS on `AtriumHandle`.**

Rationale: freshness-window is a per-engine replay-defense tunable; it tunes how aggressively this engine rejects stale device-attestation-envelopes. Different engines in the same Atrium may legitimately have different freshness windows (a long-uptime relay vs a frequently-rebooting laptop). Not membership state.

### §7.3 `revoke-cap` (capability-revocation) — separate primitive, composes

Per M1b §9 #9 caveat: "capability-revocation is a SEPARATE primitive from membership-set; the two compose at the cap-chain layer." **Disposition: STAYS in `benten-caps::chain_authority` (post-COLLAPSE-P2 single seam).**

Rationale: capability revocation is per-grant, not per-member. A member can hold N capability grants; revoking one grant does NOT remove the member from the set. The `system:CapabilityRevocation` Node replicates via Atrium sync (the MembershipSet provides the addressable scope) but isn't part of MembershipSet state.

**Composition pattern:** the per-row cap-recheck at `apply_atrium_merge` consults BOTH (a) MembershipSet to confirm the writer is a current member; (b) cap-chain-authority to confirm the writer's grant is unrevoked. Two-check pattern documented at M1b §5.4.

### §7.4 device-DID setters (caveat) — kind-aware

Per M1b §9 #12 caveat: "device-DID grain is DISTINCT from member-DID grain; either MembershipSet absorbs both grains as instances OR explicitly cleaves at device-vs-principal boundary." **Disposition: CLEAVED via the typed-variants design.** `MembershipSetKind::Atrium` carries user-DID-grain members; `MembershipSetKind::DeviceMesh` carries device-DID-grain members. The 3 setters (`set_local_device_did/keypair/attestation`) stay on AtriumHandle as per-handle config — they configure WHICH device this engine presents, not the membership state of any set.

### §7.5 Sister-primitive vs extension-hooks vs outside-entirely — decision

Per the brief's 3-option framing (extension-hooks vs sister-primitive vs outside-entirely):

- `register_zone`, `set_envelope_freshness_window`, device-DID setters → **OUTSIDE ENTIRELY** on AtriumHandle.
- `revoke-cap` → **SISTER-PRIMITIVE** in `benten-caps` that consumes MembershipSet for scope.
- HLC-temporal-ordering at offline-reconnect (M1b §9.2 #5) → **OUTSIDE** in sync layer (`benten-sync`).
- L9-A2 dual-CID (M1b §9.2 #6) → **MembershipSet OWNS authorization-identity** (envelope-blob-CID computation hook via `encrypt_to_set` output); content-identity (plaintext-CID) stays in graph substrate. This is the §8 simplification.

No extension-hooks needed. The boundary is clean.

---

## §8 — Composition with M1c K(N) path-tagged chain

Per M1c §6 + Spike-E Interpretation-B: `K(N) = HKDF(K(predecessor), "step" || edge_label || N.cid)` is path-tagged; same Node reached by different paths yields different K(N). Per M1c §10.2 #2: "K(N) derivation chain is path-tagged; NOT a stored membership-shared key." Per M1c §11 Q1: open question whether K_Atrium can root a similar chain.

### §8.1 Relationship: K_Set ↔ K_principal ↔ K(N)

For each kind:

| Kind | K_Set is | K(root) derived as | K(N) derived as |
|---|---|---|---|
| Atrium | K_Atrium (CSPRNG) | HKDF-SHA256(K_Atrium, info = "root:atrium:" \|\| codepoint_le \|\| root_cid \|\| generation_le) | HKDF-SHA256(K(pred), info = "step" \|\| edge_label \|\| N.cid) |
| DeviceMesh | K_principal (vault-loaded) | HKDF-SHA256(K_principal, info = "root:principal:" \|\| codepoint_le \|\| root_cid \|\| generation_le) | HKDF-SHA256(K(pred), info = "step" \|\| edge_label \|\| N.cid) |
| SingleDevice | K_principal (vault-loaded) | Same as DeviceMesh | Same as DeviceMesh |

This answers M1c §11 Q1: **K_Atrium uses a DISTINCT info-tag prefix `"root:atrium:"` vs K_principal's `"root:principal:"`** to prevent cross-domain key-reuse attack class (parallel to D-13 cross-codepoint defense). Codepoint + generation already bind into info-tag per M1c §2.4 + L9-A4.

### §8.2 Per-Node K(N) is INDEPENDENT of MembershipSet operations

Per M1c §10.2 #2: K(N) is derivable function of `(K_Set, path)`. **K(N) is NOT stored in MembershipSet; NOT serialized in MembershipSet CBOR; NOT distributed via the multi-stanza-HPKE envelope.** Distribution of K_Set + the canonical path is sufficient for any member to derive K(N).

The MembershipSet primitive's responsibility ends at K_Set distribution + generation tracking. Per-Node AEAD wrap/unwrap lives in `benten-graph::aead_wrap` (M1c §2.5 Layer-Graph), which consumes the derived K(N) per-call. Clean separation.

### §8.3 Per Path-A.5 (M1c §6.3): K(N) keys to immutable Version-Node-CIDs

K(N) is derived using Version-Node-CIDs (immutable), not the mutable CURRENT pointer (per Phase-4-Foundation D-4F-14 + path-a-vs-path-b ratification). This means:
- A recipient who derived K(N) for Version V1 keeps the ability to decrypt V1 even after CURRENT → V2.
- V2 has a different K(N) (different Version-Node-CID); recipient needs re-grant.
- Consistent with Compromise #31 "already-derived keys remain decryptable forever."
- **Forkability of MembershipSet** (M1b §7 Ben 2026-05-27) composes cleanly: fork → new K_Atrium → new K(root) → new K(N) for FUTURE Versions; pre-fork Versions retain their pre-fork K(N) derivation (which both parent and child members can compute via the parent K_Set retained by all original members).

### §8.4 K_principal_generation in K(root) info-tag (M1c §11 Q2 answer)

Per the recommended shape in §8.1: `generation_le` binds into the K(root) info-tag. This makes grants automatically generation-bound at the K_root layer — rotating K_principal generation invalidates all derived K(N) for FUTURE writes. Past Layer-B ciphertexts remain decryptable via K_principal_generation map under DAK (per M1c §4.2 rotation cascade).

This does NOT conflict with Inv-15/Inv-16 layering: identity stays canonical-payload-CID; authentication stays codepoint-dispatched sig; revocation semantic-tuple still works (just adds `generation` as an extra dimension in the tuple).

### §8.5 Composition shape (M3 plan-doc transformation hook)

The MembershipSet primitive INTERSECTS with K_Set + benten-crypto-suite::structural_kdf at ONE clean point: `MembershipSet::encrypt_to_set(payload)` calls `derive_root(K_Set, set_id_as_cid, codepoint)` to get K(root), then `derive_step(...)` chain to get K(N). This matches the existing `derive_root` signature; only the info-tag prefix changes per kind.

---

## §9 — Plaintext_cid simplification (Ben's catch — DUAL-CID not TRIPLE-CID)

### §9.1 The catch

Per the brief: under MembershipSet abstraction, is `plaintext_cid_local` redundant? Per Q3 Option D today, the model has:
- `plaintext_cid_local` = BLAKE3(canonical(plaintext)) — un-blinded, NEVER on wire (M1b §4.2 step 3).
- `plaintext_cid_atrium` = HMAC-SHA256(K_Atrium, plaintext_cid_local)[..16] — blinded, peer-visible (M1b §4.2 step 4).
- `envelope_blob_cid` = BLAKE3(serialized_EncryptedEnvelope_bytes) — iroh-blobs storage CID (M1b §4.2 step 5).

Three CIDs total. Question: under MembershipSet, does the local plaintext_cid still serve a purpose?

### §9.2 Analysis

`plaintext_cid_local` is "what would the un-encrypted Node hash to?" It exists today for:
- (a) **Internal indexing.** The engine needs a stable handle to the plaintext payload across encryption (envelope_blob_cid is post-encryption-bytes; if encryption parameters change, envelope_blob_cid changes).
- (b) **Dedup at the engine.** Local engine deduplicates same-plaintext writes by plaintext_cid.
- (c) **Inv-15 mint.** Identity = canonical-payload-CID; payload here means PLAINTEXT.

Under MembershipSet, `plaintext_cid_set = MembershipSet.dedup_blind_cid(plaintext) = HMAC-SHA256(K_Set, plaintext_cid_local)`. The blinded version is what's peer-visible.

**Is plaintext_cid_local redundant under MembershipSet-blinded plaintext_cid_set?**

**Answer: NO at the engine boundary, but YES on the wire.** Specifically:
- On the wire (peer-to-peer), only `plaintext_cid_set` + `envelope_blob_cid` appear. plaintext_cid_local is redundant on the wire (it would leak the equality oracle Q3 Option D defends against).
- At the local engine (single-process), plaintext_cid_local is needed for internal indexing, dedup-write, and Inv-15 identity. Removing it would force the engine to compute HMAC-K_Set every time it wanted to identify a local plaintext, costing performance + tightly-coupling local-graph identity to K_Set rotation (rotating K_Set would invalidate the local-index entries for ALL prior-encrypted Nodes).

### §9.3 Recommendation: DUAL-CID on the wire; plaintext_cid_local stays local-only

Concrete proposal:

| Identifier | Scope | Computation |
|---|---|---|
| `plaintext_cid_local` | Local-engine only; NEVER on wire; preserved across K_Set rotation | BLAKE3(canonical(plaintext)) |
| `plaintext_cid_set` | Peer-visible on wire; depends on K_Set; rotates with K_Set | HMAC-SHA256(K_Set, plaintext_cid_local)[..16] |
| `envelope_blob_cid` | iroh-blobs storage CID; depends on encryption parameters | BLAKE3(serialized_EncryptedEnvelope_bytes) |

**On-wire formats use DUAL-CID (`plaintext_cid_set` + `envelope_blob_cid`); NOT TRIPLE-CID.** plaintext_cid_local is a local-only optimization.

This matches L9-A2 / U18 contract: "Envelope-CID stability under recipient-set evolution; dual-CID (plaintext_cid + envelope_blob_cid)" — the L9 framing already says DUAL on the wire; the existing 3-CID nomenclature confuses local-optimization with wire-shape.

**Per-Node K(N) derivation question:** does K(N) chain off plaintext_cid_local or plaintext_cid_set? Per M1c §6.2 + V1-FROZEN-INTERFACE §15.f the structural-KDF info-tag uses `N.cid` — currently this is plaintext_cid_local. Under MembershipSet, **K(N) should continue to use plaintext_cid_local** (it's the stable identity across K_Set rotation; otherwise rotating K_Set would force re-derivation of every K(N) — heavy cascade). This is consistent with Path-A.5 (K(N) keyed to immutable Version-Node-CID; M1c §6.3).

### §9.4 Engine-implementation impact

Engine internals:
- `Engine::create_node(plaintext)` → computes `plaintext_cid_local`; stores in local two-CID map indexed by plaintext_cid_local.
- On encrypt-for-share: `let plaintext_cid_set = membership_set.dedup_blind_cid(plaintext);` + `let envelope = membership_set.encrypt_to_set(plaintext);` + `let envelope_blob_cid = blake3(serialize(envelope));`. Maps `plaintext_cid_set → envelope_blob_cid` for iroh-gossip dedup (Q3 §3).
- On receive: peer sees `(plaintext_cid_set, envelope_blob_cid)`. Pulls envelope by envelope_blob_cid via iroh-blobs. Unwraps via `membership_set.decrypt(envelope) → plaintext`. Recomputes `plaintext_cid_local = blake3(plaintext)`. Verifies match against the canonical-bytes invariant.

### §9.5 Simplification verdict

**SIMPLIFICATION REAL. On-wire CID count reduces from 3 to 2.** The plaintext_cid_local stays as a local-engine optimization, never on wire. L9-A2/U18 contract preserved + sharpened.

---

## §10 — R0 plan-doc abstraction implications

(M3 specialist owns the amendment-by-amendment transformation; this §10 gives the abstraction surface they'll use.)

### §10.1 Sections of the F-full R0 plan-doc needing restructuring

Per M1b §3.1 (28 unified amendments; 18 v1-beta-LOAD-BEARING) + M1a §3.1 Phase-4-Meta-Core scope + M1c §3.1 planned-state inventory, the R0 plan-doc has 4 layers (A/B/C/D). MembershipSet abstraction touches each:

**Layer A (K_principal store).** No restructuring; K_principal becomes the K_Set source for `DeviceMesh` + `SingleDevice` kinds. The vault shape is unchanged.

**Layer B (per-Node AEAD with K(N)).** No restructuring; K(N) derivation chain unchanged per §8. The info-tag prefix per-kind change is a 1-line `derive_root` extension.

**Layer C (encrypt-to-recipient HPKE + multi-stanza).** **MAJOR restructuring.** Today's brief: "multi-stanza-HPKE for groups" — under MembershipSet, this becomes `MembershipSet::encrypt_to_set` which IS the multi-stanza primitive. L9-A1/U17 amendment lands inside MembershipSet, not as a separate envelope shape. The 4 sites M1c §10.3 identifies (multi-device-key-wrap + K_Atrium-distribution + Drop-bundle-to-recipients + cross-Atrium-federation) collapse to ONE `MembershipSet::encrypt_to_set` call path.

**Layer D (DAK + multi-device-key-wrap WIRE + remote-permission-call WIRE).** **Partial restructuring.** Multi-device-key-wrap WIRE becomes `MembershipSet<DeviceMesh>::add_member` returning the `MultiStanzaHpkeEnvelope`. The "wire shape" frozen at V1-FROZEN-INTERFACE.md item 15.5 is exactly this envelope shape — the freeze is preserved. Remote-permission-call STAYS as separate primitive (signaling, not membership).

### §10.2 28 unified amendments — likely consolidation per MembershipSet

Per M1b §3.1 table + 9-eyes registry, the 5 Atrium-integration LOAD-BEARING amendments (A1-A5) consolidate:

| Original | MembershipSet-consolidated form |
|---|---|
| U17 / L9-A1 multi-recipient stanza | `MembershipSet::encrypt_to_set` returns MultiStanzaHpkeEnvelope by construction; no separate L9-A1 amendment needed |
| U18 / L9-A2 dual-CID | `MembershipSet::dedup_blind_cid` + `encrypt_to_set` output; DUAL-CID per §9 |
| U19 / L9-A3 recipient-key-generation | `MemberKey.hpke_pubkey` carries the recipient's current pubkey; rotation tracked via `MembershipSet.generation` |
| U20 / L9-A4 K_principal_generation | `K_Set.generation` field; binds into info-tag per §8.4 |
| U21 / L9-A5 ExecuteWorkflow scope | Orthogonal to MembershipSet — stays as UCAN `PermissionOperation::ExecuteWorkflow` arm |
| C44 / Q3-Option-D K_Atrium-blinded plaintext_cid | `MembershipSet::dedup_blind_cid` |
| iroh-gossip codepoint-reserve | Orthogonal transport-binding; stays as transport codepoint |
| **AtriumPolicy D3 refresh_required** | `MembershipSetPolicy.refresh_required_secs` |
| **AtriumPolicy D7 admin-rotation** | `KindPolicy::Atrium.admin_pubkey` + policy_version increment |
| **F-full Layer-A K_principal** | `K_Set.key_material` source for DeviceMesh/SingleDevice |
| **Multi-device-key-wrap envelope SHAPE (V1-FROZEN-INTERFACE 15.5)** | The MultiStanzaHpkeEnvelope shape (§2.2) |

**Net amendment count reduction:** approximately 5-7 of the 18 LOAD-BEARING amendments collapse into the MembershipSet primitive's own surface. M3 specialist does the exact per-amendment audit.

### §10.3 New surfaces the plan-doc needs

- **`benten-membership` crate (15th crate).** Per M1b §10 Q2, this is the natural home. Depends on `benten-id` + `benten-crypto-suite` + `benten-core`.
- **MembershipSet wire-format pin** in `docs/V1-FROZEN-INTERFACE.md` §15.x.
- **MembershipSetKind 3-arm typed-rejection** for 4th-arm scenario (matches §15.c discipline).
- **TS-side mirror** at `packages/engine/src/membership.ts` + `errors.generated.ts` per §3.5g cross-language rule-mirror.

### §10.4 What does NOT change

- 12-primitive irreducibility (CLAUDE.md #1) — MembershipSet is composed of existing primitives (Node + Edge + Cid + Did + AeadEnvelope + ...).
- Inv-15 + Inv-16 + the additive-codepoint discipline (M1c §3.5.3).
- DAK + Layer-A vault shape (M1c §5).
- K(N) derivation (M1c §6) other than the info-tag prefix per-kind change.
- Hybrid-sig + codepoint registry (M1c §7-8).
- UCAN-bound capability grants + chain-walker (M1b §5).

---

## §11 — Self-assessment + confidence per finding

### §11.1 Confidence breakdown

| Section | Confidence | Why |
|---|---|---|
| §1 Recommendation (typed-variants) | **HIGH (~85-90%)** | Triangulated from M1a 4-identity-concepts + M1b 2 non-fits + M1c per-DID-keypair non-fits; type-system invariants align with Phase-4-Foundation R1 ratifications. |
| §2 Struct + CBOR | **HIGH** | Direct derivation from M1b §9.1 sketch + M1c §10.3 recurring shape; CBOR follows V1-FROZEN-INTERFACE §15 + Inv-15 + L9-A2 dual-CID precedent. |
| §3 Three-kind instantiation | **HIGH** | Maps 1:1 onto M1a/M1b/M1c cataloged shapes. |
| §4 Operations (7-op API) | **HIGH** | Mirrors M1b §9 fit-table; `RotationPolicy` enum surfaces M5's CGKA-LITE/full decision space. |
| §5 Threat models | **HIGH** | Direct derivation from M1a §6 + §3.5 + M1b §8 + Q3 Option D; per-kind threat split is uncontroversial. |
| §6 4-identity-concepts composition | **HIGH** | Direct cite-driven from CLAUDE.md #17 + #18 + M1a §6.1; preserves all 4 grains. |
| §7 M1b non-fits | **HIGH** | M1b §9.2 already laid out the dispositions; M2 just formalizes the boundaries. |
| §8 K(N) composition | **MED-HIGH (~75-80%)** | The `"root:atrium:"` vs `"root:principal:"` info-tag prefix choice answers M1c §11 Q1 cleanly, but M3/M5 should verify the cross-domain separation suffices against Spike-E-class collisions. The 1-line `derive_root` extension is bounded. |
| §9 DUAL-CID simplification | **MED-HIGH (~75%)** | Aligns with L9-A2/U18 explicit framing; M3/M4 must verify against any unstated TRIPLE-CID assumption in the 28 amendments. The local-engine optimization rationale is strong. |
| §10 R0 plan-doc implications | **MED (~65-70%)** | M3 owns the amendment-by-amendment transformation; M2's table is a guide, not authoritative. Real consolidation depth depends on per-amendment specifics. |
| AdminKickEpoch / RotationPolicy ergonomics | **MED (~60%)** | Surface for M5's CGKA-survey decision; details TBD per M5's lane. |
| Recursive composition (Garden) | **MED (~60%)** | The EXACTLY-3-arms enum permits future `Garden` 4th arm via §15.c HALT-AND-SURFACE-TO-BEN; the v1-beta wire doesn't foreclose, but the recursive composition body lands Phase 7. |

### §11.2 Areas where M3-M6 specialists should verify

- **M3 (amendment-transformation):** verify §10.2 consolidation table against the 28 amendments; some may not collapse as cleanly as the table suggests.
- **M4 (red-team):** verify §5 threat models per kind; construct attack scenarios that exploit the kind-discriminator (e.g. confused-deputy across kinds — can an attacker submit an Atrium-kind MembershipSet that's actually parsed as DeviceMesh-kind?). Verify §9 DUAL-CID doesn't open an equality-oracle on plaintext_cid_local via timing.
- **M5 (CGKA-survey):** the `RotationPolicy::AdminKickEpoch` arm is a hook for M5's CGKA-LITE-or-full landing. Should this arm be CODEPOINT-RESERVED at v1-beta or designed-out per Q3 §1.2 conclusion 2 (CGKA-deferred)? M5 owns.
- **M6 (transport-configurability per MembershipSet):** is per-MembershipSet transport configuration a first-class concept? (E.g. Atrium-X uses iroh-direct; Atrium-Y uses Tor-relay; DeviceMesh uses LAN-only.) The `AtriumHandle::open(membership_set, transport_config)` shape composes; no MembershipSet field changes.

### §11.3 What I did NOT investigate (within-scope but bounded)

- Detailed cargo-public-api impact (M1b Q9 — affects baselines across 14 crates + new 15th crate).
- Per-DID hybrid encryption pubkey storage shape (whether HybridKemPubkey lives in `benten-id::Did` resolution or in MembershipSet member-key field). I assumed the latter; M1c may inform.
- Migration path: how existing AtriumHandle production callers migrate to MembershipSet-based AtriumHandle constructor without v1-beta-wire break. Bounded by additive-codepoint + V1-FROZEN-INTERFACE freeze discipline.
- Performance characterization: encrypt_to_set with N members = N HPKE-encap operations. Atrium scale at v1-beta is ~10-50 members; encrypt-to-set call cost grows linearly. Phase 7 Gardens (~hundreds/thousands) may motivate the iroh-gossip codepoint reserve + CGKA-future-additive that's already on the planning surface.

---

## §12 — Citations

### §12.1 Cataloger inputs

- M1a multi-device-sync @ `3618e051` → `.addl/phase-4-meta/membership-set-cataloger-m1a-multi-device-sync.md`
- M1b Atrium-membership-sharing @ `1816ea60` → `.addl/phase-4-meta/membership-set-cataloger-m1b-atrium-membership-sharing.md`
- M1c key-management @ `50eb901d` → `.addl/phase-4-meta/membership-set-cataloger-m1c-key-management.md`

### §12.2 Background pinned via the brief

- `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` — 28 unified amendments
- `phase-4-meta-core/option-f-plus-critique-{c1..c5}` — 5 critique-round outputs
- `phase-4-meta-core/q3-revisit-option-d-community-lens @ 23f76e24` — Q3 Option D K_Atrium-blinded
- `phase-4-meta-core/atrium-policy-credential-validity-design @ e90900b4` — AtriumPolicy D1-D5 design
- `phase-4-meta-core/path-a-vs-path-b-specialist-review @ 5f50a028` — Path-A.5 immutable Version-Node-CIDs

### §12.3 Tree-pinned (HEAD `2172cb6d`)

- `crates/benten-id/INTERNALS.md` — identity primitives
- `crates/benten-sync/INTERNALS.md` — Atrium transport + CRDT
- `crates/benten-crypto-suite/INTERNALS.md` — crypto-suite integration
- `crates/benten-crypto-suite/src/structural_kdf.rs` — K(root)/K(N) derivation
- `crates/benten-crypto-suite/src/aead.rs` — AEAD envelope + AAD layouts
- `crates/benten-crypto-suite/src/codepoint.rs` — codepoint table FROZEN
- `crates/benten-graph/src/redb_backend.rs:117-210` — K_principal STUB site
- `crates/benten-engine/src/engine_sync.rs` — AtriumHandle 12 public methods
- `crates/benten-engine/src/engine.rs::apply_atrium_merge` — per-row cap-recheck seam
- `crates/benten-drop/INTERNALS.md` + `src/lib.rs` — Drop bundle format
- `docs/V1-FROZEN-INTERFACE.md` §15 — frozen surfaces (15.a SubgraphSpec; 15.c Scope EXACTLY-2-arm; 15.d AuthorizationGrant; 15.5 multi-device-key-wrap envelope shape; 15.f KDF info-tag; 15.g two-CID + IROH_BLOCK_SIZE)
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` D-13 (CLOSED) + D-15 (post-v1-beta hardening) + D-26 G-CORE-PQ-WIRE + D-27 StampedValueV2
- `docs/INVARIANT-COVERAGE.md` Inv-14 + Inv-15 + Inv-16-mint
- `docs/SECURITY-POSTURE.md` Compromise #22 + #23 + #30 + #31

### §12.4 CLAUDE.md baked-in references

- CLAUDE.md baked-in #1 12-primitive irreducibility
- CLAUDE.md baked-in #5 only-call-site crypto-agility + LAMPS hybrid floor
- CLAUDE.md baked-in #15 v1-milestone-gate + v1-assessment-window identity-recovery
- CLAUDE.md baked-in #17 device-shape (a)/(b)/(c) + CapabilityEnvelope 4-dim
- CLAUDE.md baked-in #18 4-identity-concepts + plugin trust model
- CLAUDE.md baked-in #19 engine-level extensions vs app-level plugins

---

**End of M2 primitive design.** ~9,500 words. Ready for M3 amendment-transformation + M4 red-team + M5 CGKA-survey + M6 transport-configurability specialists.
