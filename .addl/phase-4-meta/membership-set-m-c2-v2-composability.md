# MembershipSet **M-C2-v2** — cross-amendment composability critique of M-CONS-v2

> **Top-banner re-orient (HANDOFF discipline).** This document is M-C2 of 3 critique-round
> agents running in parallel against `phase-4-meta-core/membership-set-m-cons-v2-consolidator
> @ e62ff540` (M-CONS-v2; 1295 LOC; 31 F-amendments + 26 Compromise mints + 10-clause Inv-20).
> Tree HEAD at write: `2172cb6d` (origin/main; fetched + verified at session start).
> Branch: `phase-4-meta-core/membership-set-m-c2-v2-composability`. Date: 2026-05-28.
>
> Read BEYOND what's named here if you arrive cold: the earlier C2-prior composability
> critique `9c548e5f` (3 contradictions + 4 composition failures + 5 blind spots; pattern
> reference for this M-C2-v2 lens); M-CONS-v1 baseline `74580ee6`; N1 `ed592770`; N2
> `a1b5a552`; N3 `298c80d9`; N4 `2ee24e9f`; AtriumPolicy `e90900b4`; Path-A.5 `5f50a028`;
> the CLAUDE.md baked-in #1/#5/#15/#17/#18/#19 + MEMORY.md feedback rules. The 5-panel +
> 3-cataloger background per M-CONS-v1 §12.
>
> **Role.** SENIOR CRYPTOGRAPHER + SYSTEMS ARCHITECT. Build the FULL encoder/decoder/
> key-derivation/validation chain that results from adopting all 31 F-amendments + 26
> Compromise mints + 10-clause Inv-20 + MembershipSetPolicy D1-D7 + TransportConfig +
> RotationPolicy + Authority + RoleId + RoleAssignments + per-Kind specialization + 5
> Ben-ratifications **together**, then attack the composed shape. Distinct from M-C1
> (elegant-shape) and M-C3 (fresh-eyes).
>
> **Posture.** ADVISORY. CONFIRM / CONTEST / REFINE the consolidator's recommendations
> per `feedback_review_finding_ground_truth_verify`. DISAGREE-WITH-EXPLANATION first-class.
> HARD RULE 12 dispositions explicit for every finding.

---

## §1 Executive verdict + confidence

**Top-line.** M-CONS-v2 is **COMPOSABLE WITH 9 LOAD-BEARING REFINEMENTS** (see §6). It is
NOT composable as-is. **4 cross-amendment contradictions** + **5 composition failures
in adversarial scenarios** + **4 interaction blind spots** are reachable from a literal
reading of the registry. All are closable by adding ~9 explicit composition rules,
NOT by re-architecting the M-CONS-v2 direction or rejecting any N-specialist's contribution.

Confidence on the composability-blockers: **HIGH** on all 4 contradictions + 4 of 5
composition failures; **MED-HIGH** on Scenario D (Garden / Atrium-of-Atriums); **HIGH**
on each individual refinement R-MCV2-1..R-MCV2-9.

**Comparison to the earlier C2-prior critique.** The prior C2 (`9c548e5f`) found 3
contradictions + 4 composition failures + 5 blind spots against the 9-eyes consolidated
registry pre-MembershipSet-unification. M-CONS-v1 resolved most of those (the
Sealed-Sender × multi-stanza AAD seam closed under Inv-18b; the `replay-window-per-Kind`
table closed C-2; the DUAL-CID closed C-3; etc.). **M-CONS-v2 introduces NEW cross-
amendment seams** — N1's RestrictedScopeSet × N2's RBAC × N3's RotationTrigger ×
Q1's iroh-gossip impl — that the M-CONS-v2 author surfaced individually but did
NOT cross-compose. This document constructs those compositions and stresses them.

**Headline composability blockers (require fix-now refinement, not deferral):**

- **MCV2-C-1 (CONTRADICTION):** **N2 `change_role` op × F18 DUAL-CID write-permission ×
  Inv-20 clause-c AAD-binding sender_did × MembershipSet-generation CRDT (F19).**
  Bob's role mutation from Member→Viewer is a CRDT-merge event partitioned per-member-DID;
  Bob's concurrent edit dual-CID-write carries `(plaintext_cid_local,
  plaintext_cid_membership_set, envelope_blob_cid)` sealed under K_Set with AAD binding
  `sender_did=Bob` + `member-key-generation=Bob_gen_N`. The role-mutation does NOT bump
  K_Set (F-N2-B `change_role` is NOT in the K_Set-rotation set per N3 §7) and does NOT
  bump `member-key-generation` (which is per-MemberKey, not per-role). Therefore the
  receiver verifies Bob's edit as valid AEAD + Bob's `member-key-generation` is current,
  but Bob's role at the time-of-seal vs the time-of-verify is **temporally ambiguous**.
  Per Inv-20 clause-j RBAC × UCAN intersection-of-allows, the verifier must check
  `check_permission(Bob, Write, ucan_chain)` — but against which generation of
  `role_assignments`? R-MCV2-1 closes (mint `role_generation: u32` per-RoleAssignment +
  bind into AAD).

- **MCV2-C-2 (CONTRADICTION):** **N1 RestrictedScopeSet × Inv-20 clause-h orthogonality
  claim × F18 DUAL-CID.** Inv-20 clause-h states `(MembershipSet, RestrictedScopeSet)`
  compose orthogonally at the seal-seam tuple. But the DUAL-CID write
  `plaintext_cid_membership_set` is blinded under `K_Set` of the MembershipSet, while
  the RestrictedScopeSet selects sub-graphs at the GRANT boundary — there is no defined
  composition rule for "Eve has a RestrictedScopeSet grant for sub-graph X∪Y∪Z **inside**
  MembershipSet M whose K_Set Eve is NOT a member of." Per F28's "single-element
  RestrictedScopeSet is wire-equivalent to old RestrictedScope" + N1 §3.6 row's "compose
  orthogonally at the (MembershipSet, RestrictedScopeSet) tuple", the composition
  claims Eve gets K_Set via her UCAN-grant-from-member — but UCAN attenuates allows
  per N2 §2.6 / Inv-20 clause-j; UCAN cannot mint K_Set membership unilaterally without
  violating Inv-19 (K_Set is via multi-stanza-HPKE-Encap to a member). **The orthogonality
  claim is unsound** unless RestrictedScopeSet grants compose with a side-channel K_Set
  reveal protocol (e.g., Eve-gets-K_Set-from-Alice-via-Drop-bundle-Layer-C). R-MCV2-2
  closes (explicit (MembershipSet, RestrictedScopeSet, K_Set-acquisition-path) triple
  + per-MembershipSet-Kind composition table).

- **MCV2-C-3 (CONTRADICTION):** **Q1 iroh-gossip transport × F18 DUAL-CID × per-stanza
  per-recipient unlinkability (Inv-20 clause-d) × iroh-gossip topic-per-MembershipSet
  scheme.** Q1 SCOPE-IN ratification ships iroh-gossip impl at v1-beta. iroh-gossip's
  topic-id is necessarily PUBLIC at the transport layer (it's how subscribers find the
  topic). If the topic-id is derived from `membership_set_id` (the obvious scheme per
  M-CONS-v2 §7.4 "topic-per-MembershipSet"), then **every iroh-gossip relay observes
  the membership_set_id of every published envelope** — violating Inv-20 clause-d
  per-recipient unlinkability at the TRANSPORT layer even though the per-recipient
  HPKE stanzas preserve it at the CRYPTOGRAPHIC layer. The `plaintext_cid_membership_set`
  blinding (Q3 DUAL-CID) protects content addressability against the storage host but
  does NOT blind the GOSSIP topic-name. **Adversary correlates across forks:** two
  forks of the same parent MembershipSet share a deterministic-derivable predecessor;
  gossip-topic-name predecessor-chain leaks the entire MembershipSet history. R-MCV2-3
  closes (gossip topic-name discipline: blind topic = `BLAKE3(membership_set_id ||
  K_Set || generation)` truncated; rotates with each fork; rendezvous through
  out-of-band Drop).

- **MCV2-C-4 (CONTRADICTION):** **N2 `member_key_rotation` op × F19 generation-CRDT-vector
  × F-N2-B `self_leave` × concurrent admin `change_role` × AdminKickEpoch (M-CONS-v1
  RotationPolicy 2-arm folded into FORK-ONLY at v1-beta).** Inv-20 clause-e mandates
  generation-CRDT-vector PER-MEMBER-DID partition. But `member_key_rotation` bumps
  Bob's `member_key_generation`; `self_leave` is Bob removing himself (a member-driven
  event); admin `change_role` mutates `role_assignments[Bob]`; AdminKickEpoch removes
  Bob (admin-driven fork). **The CRDT-partition discipline does NOT specify ordering
  between member-driven and admin-driven mutations on the SAME member-DID.** If Bob
  rotates his key at HLC=t1 + Admin kicks Bob at HLC=t1+ε, the CRDT merge sees:
  Bob's partition has (rotation@t1, self-state@t1+δ); Admin's partition has
  (kick@t1+ε). Which wins? Inv-20 clause-e is silent. The `parent_membership_set_id`
  fork chain (Inv-20 clause-h + §1.2 admin-set-cardinality validators) gives ONE fork
  path; concurrent kick-vs-self-leave gives TWO fork paths racing. R-MCV2-4 closes
  (HLC-tie-break + canonical-fork-winner extends N4's R-N4-3a to member-driven races).

**Composition failures (Scenarios A-F per brief):**

- **MCV2-F-A (SCENARIO A — Bob's role-change race):** See MCV2-C-1. Bob's edit at
  HLC=t1 carries old role; admin's change_role at HLC=t1-ε reaches receivers first.
  Receivers verify Bob's edit against the new (post-change_role) `role_assignments`
  → Bob's edit REJECTED on permissions even though Bob believed-himself-Member at
  time-of-seal. **This is a CORRECT eventual-consistency outcome**, but the
  M-CONS-v2 design has NO discipline naming this: the receiver-side rejection emits
  what ErrorCode? F-N2-A §2.7 lists E_RBAC_DENY, E_UCAN_DENY, E_ROLE_NOT_ASSIGNED,
  E_AUTHORITY_CARDINALITY — none of these distinguishes "permission valid at seal,
  invalid at verify" from "permission never valid." Bob's UX is silently broken.
  R-MCV2-5 closes (mint E_ROLE_STALE_AT_VERIFY + author-time `role_generation` AAD-bind
  + receiver-side replay-window per AAD's `role_generation`).

- **MCV2-F-B (SCENARIO B — Alice's key rotation + Carol's encrypt-to-set race):**
  Alice rotates her keypair via N2 `member_key_rotation` at HLC=t1 (bumps Alice's
  `member_key_generation` from gen-N to gen-N+1). Carol concurrently encrypts to
  the set at HLC=t1+δ, using the MembershipSet snapshot at HLC=t1-ε (sees Alice
  at gen-N). Carol's HpkeMultiBase envelope contains a stanza for Alice keyed
  at gen-N; per-stanza AAD binds `member-key-generation=N`. Alice's gen-N+1 device
  receives the envelope, observes the AAD-bound gen-N, and **must accept** to
  preserve eventual consistency. But Alice's gen-N key material — does Alice's
  client RETAIN gen-N decryption capability after rotation? Per `member_key_rotation`
  semantics in N2, **this is unspecified**. The M-CONS-v2 design does not pin
  the key-retention discipline post-rotation. If Alice destroys gen-N, Carol's
  envelope becomes unreadable (silent data loss); if Alice retains gen-N forever,
  the rotation provides ZERO forward-secrecy benefit. R-MCV2-6 closes (mandate
  gen-N retention for a `key_retention_window_secs` per MembershipSetPolicy;
  after-window decrypt yields E_MEMBER_KEY_GENERATION_EXPIRED with named
  application-layer recovery).

- **MCV2-F-C (SCENARIO C — Atrium fork × N2 MembershipSetMetadata rename
  preservation):** M-CONS-v1 fork-op carries forward `policy.metadata` (N2's name/
  description/created_by_human_label) into the forked MembershipSet — OR fresh?
  M-CONS-v2 §6.1 D5 says fork inherits `permissive_default(kind)` per-Kind
  constructor; F-N2-C says `rename` is admin-gated. **Composition not specified.**
  If fork inherits metadata, an AdminKick fork that's downstream of a name-change
  may carry STALE metadata (admin-renamed pre-kick at HLC=t1; kick at t1+ε on a
  fork at parent's HLC=t1-δ — fork picks up pre-rename name). If fork resets
  metadata, the UX is broken (kicked-member fork suddenly shows "Untitled Atrium").
  R-MCV2-7 closes (mandate fork-inherits-metadata + admin can `rename` on the new
  fork; CRDT-merge of metadata uses HLC-LWW per-field).

- **MCV2-F-D (SCENARIO D — Garden / Atrium-of-Atriums codepoint-reserve composition):**
  M-CONS-v2 reserves `AtriumWithRotatingGroupKey` codepoint + recovery-from-fork.
  Garden / Atrium-of-Atriums is codepoint-reserve-only per L9 O7. Sub-Atrium-A has
  K_Set-A; sub-Atrium-B has K_Set-B. Content at Garden-level uses what? K_Set-G
  (third independent K_Set with Garden-membership shared-key) OR composed
  (encrypt-to-A and encrypt-to-B independently)? **M-CONS-v2 does not specify.**
  If composed encrypt-to-both, every Garden-level content carries TWO HpkeMultiBase
  envelopes — wire-cost doubles; per-recipient unlinkability across A and B is
  broken because the same `plaintext_cid_local` references both. If independent
  K_Set-G, the membership composition rule "Garden member ⊆ Sub-A ∪ Sub-B" must
  be enforced — but the runtime validation site is unspecified. R-MCV2-8 closes
  (pin Garden composition shape at codepoint-reserve: per N1 §1.5 P12 codify, use
  RestrictedScopeSet to attenuate Garden-level grants into per-Sub-Atrium grants;
  K_Set-G is a separate primary K_Set; Garden membership is its own
  MembershipSet whose member-set is itself a set of MembershipSets — recurse via
  codepoint).

- **MCV2-F-E (SCENARIO E — AdminKickEpoch fires during self_leave race):** Admin
  initiates AdminKickEpoch fork at HLC=t1 (excludes Bob); Bob executes `self_leave`
  at HLC=t1+ε. Per Inv-20 clause-h's "FORK-ONLY rotation (AdminKickEpoch folded into
  FORK-ONLY per M5)", both events trigger a fork. **TWO concurrent forks at the
  SAME parent MembershipSet, both signed-valid.** N4 R-N4-3a (concurrent-fork
  tie-break: lex-smaller HLC wins; equal-HLC, lex-smaller MembershipSetId wins;
  losing fork archived) was minted for ADMIN-vs-ADMIN concurrent forks. But Bob
  is NOT an admin (per Inv-20 clause-i, only Admin role can mutate authorities;
  but `self_leave` is a member-self-mutation op per F-N2-B). N4 tie-break does
  NOT name member-driven self_leave. **Does Bob's self_leave fork count for
  tie-break?** Or does only Admin-fork count? R-MCV2-9 closes (extend N4 R-N4-3a
  tie-break to all member-driven fork-events: self_leave, member_key_rotation if
  it bumps K_Set, AND admin-driven forks).

- **MCV2-F-F (SCENARIO F — RestrictedScopeSet × Path-A.5 immutable Version-Node-CID
  semantics × CRDT-merge admin-removes-Y-child-of-X1):** Eve has a RestrictedScopeSet
  grant for `X + Y-children-of-X1`. Per Path-A.5 (CLAUDE.md baked-in #18 — Version-Node-CID
  immutability + K(V) discipline), Version-Nodes are immutable; "removing" a Y-child
  of X1 in CRDT-merge means a NEW Version-Node-CID for X1 is minted that does NOT
  include the removed Y-child. **Eve's grant references the OLD X1.** Per F28
  RestrictedScopeSet containment, Eve's spec is structurally-decided against the
  Subgraph rooted at X1's Version-Node-CID at grant-time. Eve continues to see
  the OLD X1 subtree forever — including the "removed" Y-child. The admin believed
  the removal was effective; Eve still sees the data. **This is correct by
  Path-A.5 design** (immutable Version-Node-CID; admin's intent expressed via
  the LATEST X1 version-node, not retroactive deletion). But the M-CONS-v2 design
  does not surface this as a Compromise mint or honest-architectural-disclosure
  (analogous to #55 GDPR-RTBF). R-MCV2-(B-1) closes (mint Compromise #57:
  RestrictedScopeSet grants are content-addressed-immutable; admins must rotate
  the grant or fork the MembershipSet to revoke; analogous-to-#55 honest-
  architectural-disclosure).

**Interaction blind spots (cross-amendment, not visible to individual N-specialists):**

- **MCV2-B-1 (BLIND SPOT):** **N2 RBAC × N1 RestrictedScopeSet × Path-A.5 immutable
  Version-Node-CID** as a 3-way composition (see Scenario F above). The R-MCV2-(B-1)
  refinement mints Compromise #57.

- **MCV2-B-2 (BLIND SPOT):** **N4 RotatingGroupKeyChainedMode codepoint-reserve sub-slot
  × current MultiRecipientSealing wire format × per-stanza AAD-binding.** N4 R-N4-2
  reserves `{NoChain | SsKChain | MlsChain | DcgkaChain}` as sub-slot under
  `AtriumWithRotatingGroupKey`. **The current MultiRecipientSealing wire format (Inv-20
  clauses a/b/c) does NOT have a slot for the chained-mode's per-message ratchet state**
  — that state would live INSIDE the recipient stanza (e.g., MLS "epoch" + "leaf
  index" tuples). Either (a) post-v1-beta breaking-wire-format change when the slot
  lands, or (b) requires v1-beta to reserve per-stanza CHAINED_STATE OPTIONAL TLV
  field NOW. R-MCV2-(B-2) closes (reserve `Option<ChainedStateTlv>` per-stanza at
  v1-beta as codepoint-reserve TLV; doc-only; preserves wire-format-additivity for
  post-v1-beta landing).

- **MCV2-B-3 (BLIND SPOT):** **F-N2-B `audit_log_query` × Inv-20 clause-d per-recipient
  unlinkability × N3 #55 GDPR-RTBF P2P-by-design disclosure.** The audit-log query
  surface reveals MembershipEvent history (admin-kicks, role-changes, member-key-
  rotations, self-leaves, renames). **Per-recipient unlinkability is broken at the
  audit surface** — the audit log is by-design queryable by ANY current member
  (per N2 §3.1 audit_log_query has no role-gate other than "current member"). A
  current member can correlate (HLC, member-DID, op-type) across all history,
  defeating Inv-20 clause-d's promise against a single MEMBER. **The unlinkability
  promise is across NETWORK-OBSERVERS, not insiders** — but the M-CONS-v2 design
  does not surface this. R-MCV2-(B-3) closes (clarify Inv-20 clause-d threat-model
  as network-observer-only + scope-down audit_log_query to admin-role per F-N2-B
  RBAC gate + mint Compromise #58 as honest-architectural-disclosure).

- **MCV2-B-4 (BLIND SPOT):** **F-N2-A 4 sibling-traits (Lifecycle/Members/Crypto/Audit)
  × N1 RestrictedScopeSet at `benten-caps::scope` × Per-Kind specialization invariants
  (Atrium/DeviceMesh/SingleDevice).** The sibling-trait families operate at the
  benten-membership-set crate boundary; RestrictedScopeSet operates at benten-caps::scope
  boundary; per-Kind constructor validators operate at the inherent-impl boundary.
  **Three different module-boundary enforcement sites for related invariants.**
  When the constructor validator says "DeviceMesh exactly-1 Authority with role=Admin",
  the sibling-trait `Members::add_member(MemberKey)` must enforce "MemberKey variant
  matches Kind"; the RestrictedScopeSet grant via `Crypto::encrypt_to_set` must
  enforce "DeviceMesh cannot have RestrictedScopeSet grants of certain shapes (e.g.,
  cross-user-Atrium-sub-graph) because DeviceMesh is single-user-by-construction."
  **The 3-way composition rule is unspecified.** R-MCV2-(B-4) closes (mint per-Kind
  composition matrix table in MembershipSet-Spec §-add: (Kind × Trait-op ×
  RestrictedScopeSet-shape) → allowed/error; pre-flight invariant check at API
  boundary in inherent-impl).

**Disagreements with the M-CONS-v2 consolidator (additional):**

- **CONTEST §1.3's N3 RotationTrigger doc-only framing.** The 6-trigger doc-enum
  (`AdminKick | MemberDepart | AdminDepart | DeviceRevoke | CompromiseResponse |
  PeriodicHygiene`) is "doc-only; no wire-format change." **But the AAD-binding
  per Inv-20 clause-c could carry `trigger: RotationTrigger` cheaply** (1 byte
  codepoint) and would close MCV2-B-3 partially by making the audit-trail intent
  cryptographically-bound. M-CONS-v2 ships the doc-enum without binding it
  cryptographically. R-MCV2-9 partial-extension addresses.

- **CONFIRM consolidator's §5.3 "no new first-order invariants beyond Inv-20"
  verdict.** Confirmed. The 4 candidates (RBAC-invariant; Authority-cardinality;
  iroh-gossip-transport; MembershipEvent-stream) are correctly absorbed into Inv-20
  clauses or scoped out. **BUT** Inv-20 clause-d ("per-recipient unlinkability")
  needs the network-observer-only scoping per MCV2-B-3.

- **REFINE consolidator's §11.4 Pattern P14 codification.** The `disposition_class`
  field is well-named (`Accepted-Trade-Off` vs `Substrate-Guarantee-Disclosure`).
  **Add a third class** `Composition-Hazard-Honest-Disclosure` for compromises
  like the proposed #57 + #58 above: these are not "we made a trade-off" nor
  "the substrate works this way" — they are "amendments compose in a way that
  surprises non-experts; disclose explicitly." R-MCV2-9 mentions.

**On the consolidator's "M-CONS-v2 is DECISION-READY for the M-C1/M-C2/M-C3 critique
round" claim (§12.3):** REFINE. The doc is decision-ready on the 31 amendments + 26
Compromises + 10-clause Inv-20 individually + on the N1/N2/N3/N4 integration directions.
It is NOT decision-ready on cross-amendment composition rules. Adding 9 explicit
composition-rules (R-MCV2-1..R-MCV2-9) takes the registry to genuinely-decision-ready.
**~2.5-3.5 wave-days of refinement** (mostly doc-prose + 2 new ErrorCodes + 1 AAD-field
mint + audit-deliverable §-rows), no scope expansion. None require Wave-MS-PRIMITIVE
canary re-design.

---

## §2 Composition model — full encoder / decoder / key-derivation / validation chain
under M-CONS-v2 (all 3 MembershipSetKind variants)

This section constructs the model that results from applying all 31 F-amendments + 26
Compromises + 10-clause Inv-20 + MembershipSetPolicy D1-D7 + TransportConfig +
RotationPolicy + Authority + RoleId + RoleAssignments + per-Kind specialization + Ben's
5 ratifications **together**, for ALL 3 MembershipSetKind variants. Basis for §3-§5.

### §2.1 The composed `MembershipSet` struct (M-CONS-v2 final shape)

```rust
// Per F17 + F-N2-A + F-N2-C + N1 F28 references
// Canonical CBOR wire shape per N2 §4.2; DAG-CBOR outer framing per F23
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MembershipSet {
    pub id: MembershipSetId,                  // BLAKE3(canonical-CBOR(struct))
    pub kind: MembershipSetKind,              // EXACTLY-3-arm (§15.c HALT-AND-SURFACE)
    pub members: BTreeSet<MemberKey>,         // homogeneous-per-Kind (Inv-20 clause-h)
    pub shared_key: K_SetCiphertext,          // multi-stanza-HPKE-Encap to members
    pub policy: MembershipSetPolicy,          // N2 Authority + RBAC + Metadata
    pub generation: GenerationCrdtVector,     // F19 per-member-DID partition
    pub parent_membership_set_id: Option<MembershipSetId>,  // fork-chain (Inv-20 h)
    pub created_at_hlc: Hlc,                  // tie-break per N4 R-N4-3a
    pub membership_attestation: SignedAttestation,  // by ≥1 Authority per Kind
    pub transport_config: TransportConfig,    // F27 (Q1 iroh-gossip impl ships v1β)
}

#[non_exhaustive]  // EXACTLY-3 active arms at v1-beta
pub enum MembershipSetKind {
    Atrium,
    DeviceMesh,
    SingleDevice,
    // AtriumWithRotatingGroupKey: codepoint-reserve only (F13 + N4 R-N4-1)
    //   sub-slot RotatingGroupKeyChainedMode (N4 R-N4-2 + Ben Option-B):
    //     NoChain | SsKChain | MlsChain | DcgkaChain
}

pub enum MemberKey {
    UserDid(Did),                             // Atrium only
    DeviceDid { did: Did, attestation: DeviceAttestation },  // DeviceMesh only
    LocalDevice(LocalDeviceHandle),           // SingleDevice only
    // (homogeneous-per-Kind via constructor invariant)
}

pub struct MembershipSetPolicy {
    pub authorities: BTreeSet<Authority>,     // N2 §1.4 uniform slot
    pub role_assignments: BTreeMap<Did, RoleId>,  // N2 RBAC
    pub kind_policy: KindPolicySubfields,     // per-Kind-specific options
    pub refresh_required_secs: Option<u64>,   // D3 (attestation refresh; NOT K_Set)
    pub threshold_admin: Option<ThresholdAdminSpec>,  // D7 v1β-CR
    pub metadata: MembershipSetMetadata,      // F-N2-C name/desc/label
    pub policy_version: u32,                  // D1 (AAD-bound)
    // <<< MCV2-C-1 ADDS role_assignments_generation: u32 here >>>
}

pub struct Authority {
    pub did: Did,
    pub sig_pubkey: HybridPubkey,             // F17 (PQ + classical)
    pub role: RoleId,
    pub admitted_at_hlc: Hlc,
}

#[repr(u8)]
pub enum RoleId {
    Viewer = 0,
    Member = 1,
    Admin  = 2,
}

bitflags! {
    pub struct PermissionSet: u8 {
        const READ  = 1 << 0;
        const WRITE = 1 << 1;
        const ADMIN = 1 << 2;
        // 1<<3..1<<7 reserved (M-CONS-v2 §13 CRYPTO-CODEPOINTS row)
    }
}

pub struct MembershipSetMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_by_human_label: Option<String>,
}

// N1 F28: at benten-caps::scope (composes via UCAN chain_authority)
pub enum Scope {
    Hashes(BTreeSet<Cid>),
    RestrictedSelector(RestrictedScopeSet),   // N1 §3.6 expanded from RestrictedScope
}

pub struct RestrictedScopeSet {
    pub scopes: Vec<RestrictedScope>,         // disjunction (union semantics)
    pub audit_commitment: Option<Cid>,        // N1 §5.2 G-AUDIT-SIBLING; defer slot
}
```

### §2.2 Per-Kind specialization table (M-CONS-v2 final)

| Kind | Authorities cardinality | Members typing | K_Set semantic | RBAC active? | RestrictedScopeSet applicable? | TransportConfig default | Fork-op allowed? |
|---|---|---|---|---|---|---|---|
| **Atrium** | ≥1 with role=Admin | all `MemberKey::UserDid` | shared K_Atrium distributed by multi-stanza-HPKE-Encap | YES (all 3 roles) | YES — cross-member grants | `GossipPlusBlobs` (iroh-gossip per Q1) | YES |
| **DeviceMesh** | exactly-1 with did=user_did, role=Admin | all `MemberKey::DeviceDid` + DeviceAttestation | K_DeviceMesh ≡ derived from user_principal_key per Q3 Option D K_Atrium-distribution | DEGENERATE (user-is-self-admin) | DEGENERATE (single-user scope; no cross-user sharing semantics) | Local Argon2id Vault (no remote transport at v1β); iroh-gossip codepoint-reserve | NO (no fork-semantic for own devices) |
| **SingleDevice** | exactly-1 with did=device_did, role=Admin | exactly-1 `MemberKey::LocalDevice` | K_SingleDevice ≡ local Argon2id Vault | DEGENERATE | DEGENERATE | Argon2id Vault local-only | NO |

**Composition observation (MCV2-B-4 origin).** RestrictedScopeSet semantics differ
substantively across Kinds; "applicable" is YES only for Atrium. M-CONS-v2 does not
encode this in the type system — runtime validation at constructor + at
`benten-caps::scope` grant boundary. The brief asked "do per-Kind specializations need
RestrictedScopeSet at all? Or only Atrium kind?" — **only Atrium needs it
semantically; the other 2 carry the type but never exercise it.** R-MCV2-(B-4)
formalizes the per-Kind allowed-shape matrix.

### §2.3 Encoder chain (encrypt-to-set, all 3 Kinds, M-CONS-v2 final)

```
1. Caller invokes membership_set.encrypt_to_set(plaintext_drop, sender_did,
                                                ucan_chain, restricted_scope_set_opt)
2. Validate sender_did's RBAC role:
     role = policy.role_assignments[sender_did]  // None ⇒ E_ROLE_NOT_ASSIGNED
     check role.permission_set() ⊇ {WRITE}       // ⇒ E_RBAC_DENY if not
3. Validate UCAN chain attenuation (Inv-20 clause-j intersection-of-allows):
     resolved_scope = ucan_chain.resolve(sender_did, WRITE)  // ⇒ E_UCAN_DENY if not
     ASSERT: restricted_scope_set_opt ⊆ resolved_scope       // ⇒ E_UCAN_DENY
4. Compute plaintext_cid_local = BLAKE3(canonical-CBOR(plaintext_drop))   [Q3 LOCAL]
5. Compute plaintext_cid_membership_set = BLAKE3(plaintext_cid_local || K_Set)
                                                                           [Q3 WIRE]
6. Generate symmetric CEK (32-byte random); XChaCha20 nonce (24-byte random).
7. Per recipient r ∈ members:
     hpke_encap[r] = HPKE-Encap(r.kem_pubkey)
     wrapped_cek[r] = HPKE-Aead(hpke_encap[r], CEK, aad = per_stanza_aad(r))
     per_stanza_aad(r) = canonical-TLV(
         codepoint:             u16 BE (Inv-20 a),
         body_cid:              plaintext_cid_membership_set (F18),
         sorted_member_did_list:[r0.did, r1.did, ...],
         sender_did:            sender_did,
         stanza_index:          r.idx,
         member_key_generation: generation_vector[r.did],
         membership_set_id:     id,
         membership_set_gen:    generation_vector_summary,
         // <<< MCV2-C-1 + R-MCV2-1 mints: role_assignments_generation here >>>
         // <<< MCV2-C-3 + R-MCV2-3: NOT here — gossip topic is separate >>>
     )
8. ciphertext = XChaCha20-Poly1305-Encrypt(CEK, nonce, plaintext_drop, aad=above)
9. Envelope = EnvelopePayload::HpkeMultiBase {
       cek_aead_ciphertext: ciphertext,
       cek_aead_nonce:      nonce,
       stanzas:             vec_of_stanzas,
   }
10. Outer framing per F23 DAG-CBOR + codepoint BE u16 per F7.
11. TransportConfig dispatch:
    if kind == Atrium && transport_config.kind == GossipPlusBlobs:
       gossip_topic = MCV2-C-3 blinded form: BLAKE3(id || K_Set || generation)
       publish to iroh-gossip topic
    else if kind == DeviceMesh || SingleDevice:
       Argon2id Vault local write (no remote transport at v1β)
```

### §2.4 Decoder + validation chain (receiver-side)

```
1. Outer DAG-CBOR framing decode (strict per F23).
2. Read codepoint (F1/F7 BE u16); dispatch via F2 strict-decode + F16 CodepointLifecycle.
3. If MembershipSetEncryption codepoint family (F8): dispatch on MembershipSetKind sub-byte.
4. Locate own stanza in stanzas[] by member_key.
5. Read per_stanza_aad TLV; CANONICAL-TLV decode + length-injectivity per F3.
6. Validate AAD components against current MembershipSet snapshot:
     ASSERT: membership_set_id matches
     ASSERT: membership_set_gen ⊆ own_known_generation (or HLC future-ok)
     ASSERT: sorted_member_did_list matches snapshot
     ASSERT: sender_did is in members (E_SENDER_NOT_MEMBER if not)
     ASSERT: sender_did has WRITE permission per RBAC + UCAN
              <<< MCV2-C-1 + R-MCV2-1: check role_assignments_generation AAD field
                  against own snapshot; mint E_ROLE_STALE_AT_VERIFY per R-MCV2-5 >>>
7. HPKE-Decap with own kem_secret_key at member_key_generation indicated in AAD:
     <<< MCV2-F-B + R-MCV2-6: if member_key_generation < own_current_gen,
         look up retained key per key_retention_window_secs; otherwise
         E_MEMBER_KEY_GENERATION_EXPIRED >>>
8. Recover CEK; XChaCha20-Poly1305-Decrypt with aad=per_stanza_aad.
9. Compute plaintext_cid_local = BLAKE3(plaintext); verify against expected.
10. Verify F18 plaintext_cid_membership_set = BLAKE3(plaintext_cid_local || K_Set).
11. Replay-window check per F5 per-Kind exclusion table.
12. If RestrictedScopeSet applicable (Atrium only):
     <<< MCV2-C-2 + R-MCV2-2: composition tuple validation — Eve's K_Set acquisition
         path must be one of: native-member, Drop-bundle-Layer-C-reveal,
         Atrium-share-via-RestrictedScope-grant; NONE imply automatic K_Set membership >>>
     ASSERT: receiver's RestrictedScopeSet (if scoped grant) covers walk-spec.
13. If kind == Atrium + transport_config.kind == GossipPlusBlobs:
     <<< MCV2-C-3 + R-MCV2-3: gossip topic-name validation; blinded topic check >>>
```

### §2.5 Key-derivation chain (Atrium / DeviceMesh / SingleDevice)

```
ATRIUM:
  K_Atrium = randomly-generated at create() or fork() time
  Distributed by multi-stanza-HPKE-Encap to each MemberKey::UserDid.kem_pubkey
  Per-fork: K_Atrium' regenerated, parent_membership_set_id pinned

DEVICEMESH:
  K_DeviceMesh = HKDF(user_principal_key, "benten/devicemesh", user_did)
                (deterministic from user_principal_key per Q3 Option D)
  Distributed to MemberKey::DeviceDid.kem_pubkey via multi-stanza-HPKE-Encap
  Per-rotation: user_principal_key rotates ⇒ K_DeviceMesh rotates ⇒
                fork-event mints new MembershipSetId

SINGLEDEVICE:
  K_SingleDevice = HKDF(local_argon2id_vault_secret, "benten/singledevice", device_did)
  No multi-stanza distribution (member set is exactly {self}).
  No rotation triggers (vault-managed locally).
```

**MCV2-B-4 observation:** the three derivations have different rotation triggers
(Atrium = create/fork; DeviceMesh = user_principal_key rotation; SingleDevice =
none). The MembershipSet primitive's `member_key_rotation` op (F-N2-B) has DIFFERENT
semantics per Kind, but this is implicit per-Kind-impl rather than enumerated in
the spec.

---

## §3 Contradictions surfaced by composition (detail)

### §3.1 MCV2-C-1 — Role-mutation × DUAL-CID write × AAD-binding × generation-CRDT

**Setup.** Bob is Member of Atrium A (10 members). Admin issues N2 `change_role`
mutating `role_assignments[Bob]` from Member → Viewer at HLC=t1. Bob concurrently
publishes a drop via `encrypt_to_set` at HLC=t1+ε using snapshot at HLC=t1-δ
(Bob believes he's Member). Bob's drop carries:

```
per_stanza_aad = {
    codepoint, body_cid, sorted_member_did_list,
    sender_did=Bob, stanza_index=i, member_key_generation=Bob_gen_N,
    membership_set_id=A.id, membership_set_gen=A_gen
    // <<< NO role_assignments_generation field at M-CONS-v2 spec! >>>
}
```

**Receiver-side (Carol) verification:**
1. AEAD-decrypts successfully (Bob's gen-N key + AAD all valid).
2. `check_permission(Bob, WRITE, ucan_chain)` against CURRENT `role_assignments`
   (post-change_role) → Bob is Viewer → READ-only → **REJECT.**
3. Bob's edit is silently dropped.

**Failure analysis.** Bob believed he was authorized at time-of-seal. Carol's check
against post-mutation `role_assignments` correctly rejects per eventual-consistency
semantics. But:
- Bob's UX shows "sent OK" (his client didn't see the change_role yet); Carol's
  UX shows nothing (no error surface for "valid seal, stale RBAC").
- The M-CONS-v2 ErrorCode catalog (F-N2-A §2.7) lists E_RBAC_DENY,
  E_UCAN_DENY, E_ROLE_NOT_ASSIGNED, E_AUTHORITY_CARDINALITY — none distinguishes
  "permission valid at seal, invalid at verify" from "permission never valid."
- **Bigger problem:** if `role_assignments` is NOT cryptographically bound into
  AAD, an adversary who learns K_Set (per Compromise #48 — K_Set compromise leaks
  MembershipSet-shape) could REPLAY Bob's old envelope after Bob's role-change,
  with no AAD-trail evidencing "this seal was authorized at a stale role
  generation." The replay-window per F5 is keyed on `(membership_set_id, generation,
  stanza_index)` — `role_assignments_generation` is independent of these.

**Disposition.** **FIX-NOW per HARD RULE 12.** Cheap (~1 byte AAD addition + 1
ErrorCode mint + 1 validator). R-MCV2-1 + R-MCV2-5.

### §3.2 MCV2-C-2 — RestrictedScopeSet × MembershipSet orthogonality claim unsound

**Setup.** Alice has Atrium A with K_Set-A and 10 members; Eve is NOT a member.
Alice wants to share a sub-graph (X + Y-children-of-X1) with Eve. Per N1 F28,
Alice creates RestrictedScopeSet({scope_X, scope_X1_Y_children}) and grants Eve
the UCAN-attenuated capability. Per Inv-20 clause-h: "RestrictedScopeSet algebra
at the grant boundary; orthogonal to MembershipSet at the (MembershipSet,
RestrictedScopeSet) seal-seam tuple."

**The orthogonality claim:** Eve gets RestrictedScopeSet via UCAN-grant; Eve gets
K_Set-A via "membership composition" — but Eve is NOT a member. **How does Eve
acquire K_Set-A?**

Three candidate paths:
- (a) **K_Set delegation:** Alice mints a per-stanza HPKE-Encap of K_Set-A to
  Eve's kem_pubkey. **But this is structurally adding Eve as a member at the
  K_Set distribution layer** without admitting her in `members` — Inv-20 clause-a
  states K_Set is distributed via multi-stanza-HPKE-Encap to **members**. UCAN
  cannot mint membership unilaterally.
- (b) **Per-Drop encryption:** Alice encrypts the specific drop (the sub-graph
  Eve has scope-grant for) under a SEPARATE one-shot key (e.g., Layer-C single-
  recipient drop with HPKE-Base), bypassing K_Set-A entirely. **This is the
  Drop-bundle path** (`benten-drop`); compatible with M-CONS-v2 but NOT what
  Inv-20 clause-h claims to compose with.
- (c) **RestrictedScopeSet-as-MembershipSet-fork:** Alice forks A into a new
  MembershipSet A' whose members = {Eve}, K_Set-A' is fresh, and the sub-graph
  is migrated/re-encrypted. **Expensive** + doesn't preserve the unification
  benefit.

**Failure analysis.** Inv-20 clause-h's "orthogonal at the (MembershipSet,
RestrictedScopeSet) tuple" framing implies K_Set comes from somewhere (the
MembershipSet) while the walk-shape comes from RestrictedScopeSet. **But Eve
is not in the MembershipSet.** The composition path is unspecified. Without
explicit specification, implementers will pick path (a) — which violates Inv-19
+ Inv-20 clause-a — or path (b) — which doesn't compose with K_Set at all.

The N1 specialist's framing (`9c548e5f` C2-prior pattern observation): N1
assumed RestrictedScopeSet is a grant-time refinement INSIDE an Atrium Eve is
ALREADY a member of (per-member sub-graph view restriction). That semantic is
clean. But the M-CONS-v2 framing is more ambitious — and that ambition opens
the composition gap.

**Disposition.** **FIX-NOW per HARD RULE 12.** Doc-only refinement (~0.5 wave-days):
specify the (MembershipSet, RestrictedScopeSet, K_Set-acquisition-path) triple in
F28 + N1 §3.6 + Inv-20 clause-h. R-MCV2-2.

### §3.3 MCV2-C-3 — iroh-gossip topic-id × MembershipSet unlinkability

**Setup.** Per Q1 SCOPE-IN, Wave-MS-TRANSPORT ships iroh-gossip impl at v1-beta.
The natural scheme is "one gossip topic per MembershipSet": subscribers find the
topic by `membership_set_id`, publish HpkeMultiBase envelopes to the topic,
receive via gossip protocol.

**iroh-gossip topic-id is PUBLIC.** That's the protocol's job: peers need to
agree on a topic-id to subscribe to. A network observer (relay, IP-level
adversary) can:
- Enumerate ALL topic-ids seen across all peers.
- For each topic-id, observe the set of subscribers (peer-ids) over time.
- Correlate (topic-id, subscriber-set, publication-timing).

**If topic-id = membership_set_id directly:**
- The membership_set_id (a public CID per F17) is observable at the transport
  layer.
- Per Inv-20 clause-d "per-recipient unlinkability" promise — across NETWORK
  OBSERVERS — is **violated at the transport layer** even though per-stanza
  unlinkability is preserved cryptographically.
- Per Compromise #48 "MembershipSet-shape-leak": K_Set compromise leaks
  MembershipSet-fingerprint. With topic-id leak ON THE WIRE, the adversary
  doesn't even need K_Set compromise — they get the fingerprint free.
- Per F19 fork-on-event: forks mint NEW `membership_set_id` but predecessor
  chain is observable via `parent_membership_set_id`. Topic-id history rebuilds
  the fork tree.

**Failure analysis.** This is structurally the SAME failure as the prior C2-prior
C-3 (`9c548e5f`) — per-recipient unlinkability claimed at cryptographic layer is
defeated at non-cryptographic layer. M-CONS-v2 narrowed Compromise #53 (transport
codepoint-reserve) to "iroh-gossip ships at v1-beta" but did NOT carry the
unlinkability discipline forward.

**Disposition.** **FIX-NOW per HARD RULE 12.** R-MCV2-3 (gossip topic-name
discipline: blind topic via `BLAKE3(membership_set_id || K_Set || generation_summary)`
truncated to e.g. 32 bits; rotates per fork; bootstrap-rendezvous via out-of-band
Drop-bundle; documented as Compromise #59 mint OR fold into #53-narrowed).

### §3.4 MCV2-C-4 — member-driven mutation × admin-driven mutation × generation-CRDT race

**Setup.** Bob is a member. At HLC=t1, Bob executes `member_key_rotation` (bumps
Bob's `member_key_generation` from gen-N to gen-N+1). At HLC=t1+ε, Admin executes
`change_role` from Member → Viewer on Bob. At HLC=t1+2ε, Bob executes
`self_leave`. At HLC=t1+3ε, Admin (unaware of Bob's self_leave because of network
partition) executes AdminKick-via-`remove_member` on Bob.

**4 events; per Inv-20 clause-e they CRDT-merge per-member-DID partition.** But
the events have DIFFERENT effects on the MembershipSet state:
- `member_key_rotation` bumps `generation_vector[Bob.did]`; does NOT fork; Bob
  remains a member.
- `change_role` mutates `policy.role_assignments[Bob.did]`; does NOT fork; does
  NOT bump generation_vector.
- `self_leave` removes Bob from `members`; per F-N2-B + M5 v1β, MIGHT trigger
  fork (per RotationPolicy = ForkOnly: yes; per AdminKickEpoch: also fork).
- AdminKick `remove_member` removes Bob from `members`; per RotationPolicy =
  ForkOnly: forks.

**Race.** Two mutually-exclusive removal events: `self_leave` (member-driven)
and `remove_member` (admin-driven). Both produce a fork at the SAME
`parent_membership_set_id`. Two forks compete. N4 R-N4-3a tie-break (lex-smaller
HLC wins) was specifically called out as ADMIN-vs-ADMIN concurrent forks. Is
self_leave eligible for tie-break?

**Inv-20 clause-h** specifies "FORK-ONLY rotation" but does not name member-vs-
admin fork-event arbitration. Inv-20 clause-e specifies CRDT-per-member-DID
partition, but Bob's partition contains BOTH `member_key_rotation` AND
`self_leave`; Admin's partition contains `change_role` AND `remove_member` on
Bob. **The CRDT-merge needs to resolve these in a deterministic order.**

If `self_leave` and `remove_member` are PUTS to `members - {Bob}`, both
idempotent, the merge converges on the same removed-state. But the FORK SEMANTIC
diverges: which `created_at_hlc` is canonical? Which signer is canonical
(Bob signed self_leave; Admin signed remove_member)? Which RotationTrigger is
recorded in MembershipEvent (`MemberDepart` vs `AdminKick`)?

**Failure analysis.** M-CONS-v2 has the tie-break primitives (HLC, MembershipSetId
lex order) but does NOT specify the cross-event-class arbitration. Implementers
will pick one rule; cross-implementation interop drifts.

**Disposition.** **FIX-NOW per HARD RULE 12.** Extend N4 R-N4-3a to ALL fork-events
regardless of authorship (admin-driven AND member-driven); pin the rule in audit-
deliverable §-row. ~0.5 wave-days. R-MCV2-4 + R-MCV2-9.

---

## §4 Composition failures (Scenarios A-F detail) — see §1 headlines

Scenarios A-F summarized in §1 with R-MCV2-X mappings. The detailed encoder/decoder
walkthroughs above (§2.3, §2.4) make each scenario's failure mode concrete. I do not
repeat them here.

**Cross-scenario observation.** Scenarios A (role-race), B (key-rotation-race), and
E (member-vs-admin-fork-race) are all instances of a SINGLE underlying pattern:
**M-CONS-v2 specifies eventual-consistency primitives (CRDT per-member-DID partition,
HLC tie-break) but does not specify the ERROR-SURFACE for stale-at-verify cases.**
The cryptographic primitives are sound; the application-layer UX is not specified.
This is the load-bearing finding cluster of M-C2-v2.

R-MCV2-5 (mint E_ROLE_STALE_AT_VERIFY) is the canary refinement for the cluster.
R-MCV2-6 (mandate key_retention_window_secs) is its DeviceMesh-and-Atrium analogue.
Together they comprise the "stale-at-verify error-surface discipline."

---

## §5 Interaction blind spots — see §1 headlines (B-1 through B-4)

Detailed in §1. Cross-referenced refinements:
- B-1 (RBAC × RestrictedScopeSet × Path-A.5 immutability) → R-MCV2-(B-1) → Compromise #57.
- B-2 (RotatingGroupKeyChainedMode × current wire) → R-MCV2-(B-2) → per-stanza
  ChainedStateTlv codepoint-reserve.
- B-3 (audit_log_query × unlinkability scope) → R-MCV2-(B-3) → admin-gate audit
  + clarify Inv-20 clause-d threat-model + Compromise #58.
- B-4 (sibling-traits × per-Kind × RestrictedScopeSet) → R-MCV2-(B-4) → per-Kind
  composition matrix.

Cluster observation: the 4 blind spots all involve THREE-WAY compositions (not
just two-amendment seams). Individual N-specialists could not see them because
each N owned one or two amendment families. M-CONS-v2 surfaced the amendments but
did not run the three-way crosses.

---

## §6 Refinement recommendations (R-MCV2-1 .. R-MCV2-9 + R-MCV2-B-1..B-4)

| ID | Statement | Wave-day cost | Wire-affecting? | Closes |
|---|---|---|---|---|
| **R-MCV2-1** | Mint `role_assignments_generation: u32` field in MembershipSetPolicy + bind into per-stanza AAD (Inv-20 clause-c extended). Bump on every `change_role` or `role_assignments` mutation. | ~0.3 (AAD field + validator + policy mutation) | YES (1 byte AAD; v1β-LB if minted now; CODEPOINT-RESERVE-incompatible if deferred) | MCV2-C-1 |
| **R-MCV2-2** | Specify the (MembershipSet, RestrictedScopeSet, K_Set-acquisition-path) triple in F28 §3.6 + Inv-20 clause-h amendment text. Enumerate 3 acquisition paths (native-member; Drop-bundle-Layer-C-reveal; fork-and-migrate). Pin "RestrictedScopeSet applies WITHIN a MembershipSet, NOT across" as v1β semantic. | ~0.5 (doc-only) | NO | MCV2-C-2 |
| **R-MCV2-3** | iroh-gossip blinded topic-name discipline: topic = `BLAKE3(membership_set_id \|\| K_Set \|\| generation_summary)` truncated to 32-bit topic-id; bootstrap-rendezvous via Layer-C Drop-bundle invite. Fold into F27 + Compromise #53-narrowed OR mint #59. | ~0.7 (impl in Wave-MS-TRANSPORT + audit-deliverable §-row) | YES (transport layer) | MCV2-C-3 |
| **R-MCV2-4** | Extend N4 R-N4-3a concurrent-fork tie-break rule to ALL fork-events (admin-driven AND member-driven). Pin in audit-deliverable §-row + MembershipSet-Spec doc. | ~0.2 (doc-only) | NO | MCV2-C-4 |
| **R-MCV2-5** | Mint ErrorCode `E_ROLE_STALE_AT_VERIFY` in F-N2-A §2.7 catalog + TS-mirror per §3.5g cross-language rule-mirror discipline. Distinguishes "valid seal, stale RBAC" from "RBAC never valid." | ~0.2 (1 ErrorCode + TS-mirror) | NO | MCV2-F-A |
| **R-MCV2-6** | Mint `key_retention_window_secs: u64` field in MembershipSetPolicy. Default 30 days. After window, decrypt of envelope at expired gen yields `E_MEMBER_KEY_GENERATION_EXPIRED`. | ~0.3 (1 policy field + 1 ErrorCode + validator) | YES (1 policy field; v1β-LB if minted now) | MCV2-F-B |
| **R-MCV2-7** | Specify fork-metadata-inheritance: forked MembershipSet inherits parent's `policy.metadata` verbatim; admin can `rename` on the fork; CRDT-merge of metadata uses HLC-LWW per-field. | ~0.2 (doc-only) | NO | MCV2-F-C |
| **R-MCV2-8** | Pin Garden composition shape at codepoint-reserve: K_Set-G is independent of K_Set-A + K_Set-B; Garden membership is its own MembershipSet whose member-set is a set of MembershipSets — recursive via codepoint sub-slot. Use RestrictedScopeSet to attenuate Garden-level grants. | ~0.3 (codepoint-reserve doc + composition rule) | YES (codepoint-reserve sub-slot) | MCV2-F-D |
| **R-MCV2-9** | (a) Mint `RotationTrigger: u8` AAD-bound field on fork-event sealed envelope (per Inv-20 clause-c extension) — cheap audit-trail. (b) Extend Compromise # disposition_class with `Composition-Hazard-Honest-Disclosure` third class. (c) Optional: bind `policy.metadata.name_hash` into AAD for non-forgeability of name displayed at receivers. | ~0.4 (1 AAD field + 1 disposition class doc) | YES (1 byte AAD for fork-events) | MCV2-F-E + B-3 partial |
| **R-MCV2-B-1** | Mint Compromise #57 (`Composition-Hazard-Honest-Disclosure` class): "RestrictedScopeSet grants are content-addressed-immutable; admins must rotate the grant (UCAN delegation revoke) or fork the MembershipSet to revoke. Path-A.5 immutability is the structural reason." Analogous-to-#55 framing per Ben's P14 codification. | ~0.2 (doc-only) | NO | MCV2-B-1 (Scenario F detail) |
| **R-MCV2-B-2** | Reserve `Option<ChainedStateTlv>` per-stanza TLV slot at v1-beta as codepoint-reserve (preserves wire-format-additivity for post-v1-beta `RotatingGroupKeyChainedMode` landing). | ~0.2 (1 TLV slot + codepoint registry entry) | YES (TLV reserve at v1β-CR) | MCV2-B-2 |
| **R-MCV2-B-3** | (a) Scope-down `audit_log_query` to Admin-role per F-N2-B RBAC gate (Member-role gets MembershipEvent stream only for events they're a participant in). (b) Clarify Inv-20 clause-d threat-model: "per-recipient unlinkability against NETWORK observers; insider-member observability of MembershipEvent history is by-design." (c) Mint Compromise #58 (`Composition-Hazard-Honest-Disclosure` class): audit-log insider-correlation honest-disclosure. | ~0.3 (1 RBAC gate + 1 doc clarification + 1 Compromise mint) | NO | MCV2-B-3 |
| **R-MCV2-B-4** | Mint per-Kind composition matrix table in MembershipSet-Spec: `(Kind × Trait-op × RestrictedScopeSet-shape) → Allowed/Error`. Pre-flight invariant check at API boundary in inherent-impl. | ~0.4 (1 matrix table + per-Kind validator) | NO | MCV2-B-4 |

**Total refinement cost: ~4.2 wave-days central; bracket ~3.5-5.5.** All
absorb into Wave-MS-PRIMITIVE canary (none require new wave; none change wave
sequencing). 4 wire-affecting (R-MCV2-1, R-MCV2-3, R-MCV2-6, R-MCV2-B-2,
R-MCV2-9-a-c partial) — but 3 of those are 1-byte AAD additions or codepoint
reserves; only R-MCV2-3 (gossip topic-name) is a transport-layer impl change.

**Disposition class summary (per HARD RULE 12):**
- 9 refinements are **FIX-NOW** (R-MCV2-1..R-MCV2-9): close named contradictions
  + composition failures.
- 4 refinements are **FIX-NOW** of blind-spot class (R-MCV2-B-1..B-4): close named
  3-way compositions.

None DISAGREE-WITH-EXPLANATION. None NAMED-DEFERRED (per `feedback_no_defer_HARD_RULE`).

---

## §7 Verdict + composability stance

**Is M-CONS-v2 composable AS-IS?** **NO.** 4 contradictions reachable from literal
reading; 5 composition failures in adversarial scenarios; 4 three-way blind spots.

**Is M-CONS-v2 composable WITH refinements?** **YES, with 13 named refinements**
(R-MCV2-1..R-MCV2-9 + R-MCV2-B-1..B-4) costing ~4.2 wave-days central, none
requiring re-architecture or wave-resequencing.

**Recommended disposition:** **REFINE BEFORE R0 PLAN-DOC AUTHORING.** The R0 author
(M-R0) should compose against M-CONS-v2-refined, not M-CONS-v2 as-is. The 13
refinements are small enough to land in a single M-CONS-v2.1 commit (consolidator
re-runs against M-C2-v2 + M-C1-v2 + M-C3-v2 critique outputs); cumulative
M-CONS-v2.1 wave-day estimate becomes ~89-115 wave-days central (vs M-CONS-v2's
~85-111 central) — a +4.2 wave-day adjustment that's load-bearing for sound
composition.

**Confidence on verdict:** HIGH on the contradiction set (each is grounded in a
specific cited amendment cross-product); HIGH on 12 of 13 refinement
recommendations; MED-HIGH on R-MCV2-B-2 (per-stanza ChainedStateTlv reserve;
defensible to defer the slot if N4 R-N4-2 wire-format-additivity holds without
in-stanza reserve, but at minimal cost it's safer to reserve).

**Process-level disagreement with M-CONS-v2 §12.3:** the consolidator's confidence
that "M-CONS-v2 is DECISION-READY for the M-C1/M-C2/M-C3 critique round" is
TECHNICALLY correct (M-CONS-v2 IS the input to this critique round), but the
phrasing implies "decision-ready for Ben to ratify before R0." That second
implication is FALSE — Ben should ratify M-CONS-v2-refined, not M-CONS-v2 as-is.
This M-C2-v2 critique IS the gating discipline that prevents shipping
M-CONS-v2-as-is into R0.

---

## §8 Self-assessment + confidence per finding

| Finding | Confidence | Reasoning |
|---|---|---|
| MCV2-C-1 (role-race × AAD-binding) | **HIGH** | Concrete attack: K_Set compromise → replay of pre-mutation envelope with no AAD trail; cited Compromise #48 substrate. |
| MCV2-C-2 (RestrictedScopeSet × K_Set-acquisition) | **HIGH** | Three candidate paths enumerated; none cleanly composes with Inv-20 clause-h's orthogonality framing; doc-only fix bounded. |
| MCV2-C-3 (iroh-gossip topic × unlinkability) | **HIGH** | Same structural pattern as prior C2-prior C-3; Q1 SCOPE-IN ratification introduces the seam M-CONS-v1 had finessed via codepoint-reserve-only. |
| MCV2-C-4 (member-vs-admin fork race) | **HIGH** | Concrete 4-event race; existing N4 R-N4-3a covers only admin-vs-admin; extension is mechanical. |
| MCV2-F-A (Scenario A, role-race UX) | **HIGH** | Closure via R-MCV2-1 + R-MCV2-5 is small + non-controversial. |
| MCV2-F-B (Scenario B, key rotation race) | **HIGH** | `member_key_rotation` retention discipline genuinely unspecified in N2/N3; closure via R-MCV2-6 is small. |
| MCV2-F-C (Scenario C, fork metadata) | **HIGH** | F-N2-C admin-gated rename + M-CONS-v1 fork semantic do not compose cleanly without R-MCV2-7. |
| MCV2-F-D (Scenario D, Garden composition) | **MED-HIGH** | Garden is codepoint-reserve-only at v1-beta; the composition rule has time to mature; R-MCV2-8 names the shape now to prevent drift. |
| MCV2-F-E (Scenario E, AdminKick × self_leave) | **HIGH** | Concrete race; closure via R-MCV2-4 extension of N4 R-N4-3a. |
| MCV2-F-F → MCV2-B-1 (RestrictedScopeSet × Path-A.5 × revoke semantic) | **HIGH** | Path-A.5 immutability is ratified + load-bearing; honest disclosure analogous to #55 is structurally correct. |
| MCV2-B-2 (ChainedStateTlv reserve) | **MED-HIGH** | Defensible to skip the per-stanza TLV reserve if outer codepoint-reserve suffices; safer to reserve at minimal cost. |
| MCV2-B-3 (audit_log_query × unlinkability) | **HIGH** | N2 §3.1 spec for audit_log_query is "current member" with no RBAC gate; this is a genuine over-disclosure surface. |
| MCV2-B-4 (per-Kind composition matrix) | **HIGH** | Constructor + trait-op + RestrictedScopeSet are three boundary-sites for related invariants; the matrix is a clarity addition, not a substantive change. |
| Refinement cost estimates (~4.2 wave-days central) | **MED-HIGH** | Each individual estimate is small + bounded; cumulative is well within Wave-MS-PRIMITIVE canary slack. |

### §8.1 What I could be wrong about

- **R-MCV2-3 (gossip topic-name)** may be over-engineered — if iroh-gossip provides
  per-subscriber blinding primitives natively, the discipline simplifies. Implementer
  verifies during Wave-MS-TRANSPORT spike.
- **R-MCV2-6 (key_retention_window_secs default 30 days)** is a judgment call. May
  want per-Kind defaults (Atrium 90 days for collaborative-edit-via-re-drop per #47;
  DeviceMesh 30 days; SingleDevice N/A).
- **MCV2-C-2's RestrictedScopeSet × K_Set-acquisition framing** may be addressed
  elsewhere in N1 that I missed; the N1 doc was 1067 LOC and I read excerpted
  sections. If the path is already named in N1 §X (X > 5), R-MCV2-2 collapses to a
  doc-cross-reference.
- **MCV2-B-2 (per-stanza TLV reserve)** is conservatively-recommended; if N4 R-N4-2's
  codepoint-reserve-only-at-v1-beta is wire-format-additive without in-stanza TLV
  reserve, B-2 is redundant. Per N4 specialist's framing, the outer codepoint IS
  expected to be additive; the inner state per-stanza is what's at risk.
- **MCV2-F-D Garden composition** is codepoint-reserve-only; defensible to defer
  the R-MCV2-8 rule-pin to a future Phase-N. My-pred MINT NOW because P12 codifies
  the "name the shape early" lesson from the M2-M6 panel; defensible to skip.
- **Cumulative ~4.2 wave-day estimate** may be off by ±1 day depending on how
  audit-deliverable §-row docs accumulate.

### §8.2 What this critique does NOT cover

- M-C1 elegant-shape critique (separate critique-round agent).
- M-C3 fresh-eyes critique (separate critique-round agent).
- Authoring R0 plan-doc (M-R0 owns).
- Iroh-gossip footgun catalog (Wave-MS-TRANSPORT spike owns).
- Per-amendment doc-prose for each F-row (audit-deliverable wave owns).
- Re-running M-CONS-v2 consolidation against the refinements (M-CONS-v2.1
  consolidator job, after the 3-agent critique round closes).

### §8.3 Convergence vs the prior C2-prior critique

The prior C2 (`9c548e5f`) found 3 contradictions + 4 composition failures + 5
blind spots against the 9-eyes pre-MembershipSet-unification registry. M-CONS-v1
+ M-CONS-v2 closed most:
- Prior C-1 (Sealed-Sender × multi-stanza AAD) → closed under Inv-18b + F21.
- Prior C-2 (epoch buckets × replay-window) → closed under F5 per-Kind exclusion.
- Prior C-3 (per-recipient unlinkability × shared CID) → closed under F18 DUAL-CID.
- Prior C-4..C-7 → various closures or refinements absorbed.

**M-C2-v2 finds NEW seams that M-CONS-v1 + M-CONS-v2 introduced or did not close:**
- New N1/N2/N3/N4 amendment cross-products (MCV2-C-1, C-4) that didn't exist pre-N.
- Q1 iroh-gossip SCOPE-IN reopened a prior closed seam (MCV2-C-3) at the transport
  layer that was deferred under M-CONS-v1's codepoint-reserve-only posture.
- 3-way compositions (MCV2-B-1..B-4) that individual N-specialists couldn't see.

**This is the expected pattern.** Each consolidation round closes prior seams +
introduces new ones at the seams between newly-integrated amendments. M-C2-v2's
job is to surface those NEW seams; M-CONS-v2.1 closes them; subsequent critique
rounds find any remaining; iterate to convergence per
`feedback_iterate_critical_reviews_to_convergence`.

**Expected residual after M-CONS-v2.1:** likely 1-2 new seams from M-CONS-v2.1's
own integration of the 13 R-MCV2-X refinements; one more critique-round iteration
should close to zero substantive findings.

---

## §9 Citations

### §9.1 Primary M-CONS-v2 inputs (frozen SHAs)

- M-CONS-v2 consolidator: `phase-4-meta-core/membership-set-m-cons-v2-consolidator @ e62ff540`
  → `.addl/phase-4-meta/membership-set-m-cons-v2-consolidator.md` (1295 LOC; read in full).

### §9.2 Background (per brief)

- M-CONS-v1 baseline: `phase-4-meta-core/membership-set-m-cons-consolidator @ 74580ee6`
  → `.addl/phase-4-meta/membership-set-m-cons-consolidator.md` (1081 LOC; read in section).
- N1 sub-graph-sharing-elegance: `phase-4-meta-core/membership-set-n1-subgraph-sharing-elegance @ ed592770`
  → `.addl/phase-4-meta/membership-set-n1-subgraph-sharing-elegance.md` (1067 LOC; key sections read).
- N2 generic-primitive-ops-roles: `phase-4-meta-core/membership-set-n2-generic-primitive-ops-roles @ a1b5a552`
  → `.addl/phase-4-meta/membership-set-n2-generic-primitive-ops-roles.md` (1399 LOC; head + ops read).
- N3 continuous-rotation-edge-cases: `phase-4-meta-core/membership-set-n3-continuous-rotation-edge-cases @ 298c80d9`
  → `.addl/phase-4-meta/membership-set-n3-continuous-rotation-edge-cases.md` (506 LOC; rotation-race read).
- N4 Signal-Sender-Keys-comparison: `phase-4-meta-core/membership-set-n4-signal-sender-keys-comparison @ 2ee24e9f`
  → `.addl/phase-4-meta/membership-set-n4-signal-sender-keys-comparison.md` (494 LOC; concurrent-fork read).
- 3 catalogers + 5 specialists per M-CONS-v1 §12.
- Earlier C2-prior composability review: `9c548e5f` (pattern reference).

### §9.3 Tree-pinned (HEAD `2172cb6d`)

- `crates/benten-caps/src/scope.rs` — `Scope::{Hashes | RestrictedSelector}` (N1 F28 site).
- `crates/benten-caps/src/restricted_spec.rs` — `RestrictedScope` 6-dim (N1 F28 lift to set).
- `crates/benten-caps/src/chain_authority.rs` — UCAN chain-walker (RBAC × UCAN seam).
- `crates/benten-core/src/subgraph_spec/combinators.rs` — `intersect/union/filter`.
- `docs/V1-FROZEN-INTERFACE.md` §15 — frozen surfaces.
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` — deferred-with-name destinations.
- `docs/INVARIANT-COVERAGE.md` — Inv-20 enforcement plan.
- `docs/SECURITY-POSTURE.md` — Compromise # registry (target for #57/#58/#59 mints).

### §9.4 CLAUDE.md baked-in + MEMORY.md disciplines

- CLAUDE.md baked-in #1 (12-primitive irreducibility): R-MCV2 refinements all
  honor — no new primitives, only composition rules on existing ones.
- CLAUDE.md baked-in #5 (crypto-agility): R-MCV2-1 + R-MCV2-6 + R-MCV2-9 add
  AAD fields under codepoint-dispatch backward-compat.
- CLAUDE.md baked-in #15 (v1-beta interface freeze): R-MCV2-1 + R-MCV2-3 +
  R-MCV2-6 + R-MCV2-B-2 are minted PRE-FREEZE; landing as v1β-LB.
- CLAUDE.md baked-in #18 (Path-A.5 immutable Version-Node-CID): R-MCV2-B-1
  honors via honest-architectural-disclosure (Compromise #57).
- CLAUDE.md baked-in #19 (engine-level vs app-level): all R-MCV2 refinements
  ship engine-level (composition rules, not new primitives).
- `feedback_canary_first_parallel_implementation`: all R-MCV2 absorb into
  Wave-MS-PRIMITIVE canary; no new waves.
- `feedback_no_defer_HARD_RULE`: every refinement is FIX-NOW; no NAMED-DEFERRALS
  or "later" dispositions.
- `feedback_iterate_critical_reviews_to_convergence`: M-C2-v2 IS the discipline
  applied; expected one more iteration after M-CONS-v2.1.
- `feedback_review_finding_ground_truth_verify`: all findings cite specific
  amendment IDs + composition cross-products; orchestrator can independently
  verify by re-reading M-CONS-v2 §X.
- `feedback_plain_english_surfaces`: each finding + refinement carries plain-
  situation + my-pred resolution.
- `feedback_extra_reflection_pass_for_elegant_permanent_shape`: the cluster
  observation in §4 (stale-at-verify error-surface discipline) is the elegant
  shape that closes 3 findings (F-A, F-B, F-E) with one R-MCV2-5 + R-MCV2-6
  pattern.
- §3.5g cross-language rule-mirror: R-MCV2-5 (E_ROLE_STALE_AT_VERIFY) + R-MCV2-6
  (E_MEMBER_KEY_GENERATION_EXPIRED) require TS-mirror per discipline.

---

**End of M-C2-v2 cross-amendment composability critique document.** Output to be
considered by M-CONS-v2.1 re-consolidation alongside M-C1-v2 (elegant-shape) and
M-C3-v2 (fresh-eyes) parallel critiques.
