# MembershipSet — M-C3 (v2) fresh-eyes cryptographer critique of M-CONS-v2

> **Top-banner re-orient (HANDOFF discipline).** This is the **M-C3 fresh-eyes
> cryptographer critique** of the post-N1/N2/N3/N4 M-CONS-v2 consolidator
> (`phase-4-meta-core/membership-set-m-cons-v2-consolidator @ e62ff540`, 1295 LOC).
> M-C3 of 3 parallel critique-round agents. Distinct task: re-read the consolidated
> WHOLE with fresh cryptographer eyes after all 31 F-amendments + Compromise mints +
> invariants are composed; the per-specialist panel + M-CONS-v2 consolidator
> structurally couldn't see the full composed picture. Ben-ratifications are FIXED
> inputs; this critique is ADVISORY, not load-bearing.
>
> Read BEYOND what's named here if you arrive cold: M-CONS-v1 at `74580ee6`, N1 at
> `ed592770`, N2 at `a1b5a552`, N3 at `298c80d9`, N4 at `2ee24e9f`, AtriumPolicy
> design `e90900b4`, Q3 Option D `23f76e24`, Path-A.5 winner `5f50a028`, the prior
> M-C3 fresh-eyes critique against M-CONS-v1 (sibling-history if branched), plus
> CLAUDE.md baked-in #1/#5/#15/#17/#18/#19 + MEMORY.md feedback rules.
>
> Tree HEAD at write: `2172cb6d` (origin/main, fetched + verified at session
> start). Branch: `phase-4-meta-core/membership-set-m-c3-v2-fresh-eyes-cryptographer`.
> Date: 2026-05-28.

- **Role.** M-C3 fresh-eyes cryptographer — re-evaluate the M-CONS-v2 composed
  whole through cryptographic-property + adversary-mental-model lenses, then
  re-check Ben's ratifications in the composed context.
- **Posture.** ADVISORY. DISAGREE-WITH-EXPLANATION is first-class. Ben's
  ratifications (Q1 / Q2 / Q3 / Q4 / Am4 / Path-A.5 / 24-op classification /
  Compromise #55 framing / Option B chained-mode / AtriumWithRotatingGroupKey
  rename) are FIXED — I do NOT relitigate them; I check whether the M-CONS-v2
  integration preserves the rationale, or whether composition introduced a subtle
  conflict that warrants surfacing.
- **Out-of-scope.** Re-running M-CONS-v2's reconciliation. Authoring R0 plan-doc
  substantive content (M-R0 owns that). Operational/engineering critiques
  belonging to M-C1 (elegant-shape) or M-C2 (composability).

---

## §0 Headline summary (read-cold)

**Verdict.** **M-CONS-v2 is CRYPTOGRAPHICALLY SOUND-AS-COMPOSED** with HIGH
confidence on directional reduction-style security claims (IND-CCA2 + INT-CTXT)
across all 4 layers + 3 MembershipSetKind variants, and MED-HIGH confidence on the
per-clause Inv-20 composition. **No DISAGREE-WITH-EXPLANATION on Ben's ratified
decisions.** All 5 Ben-ratifications preserve their rationale under the M-CONS-v2
integration; I flag 0 of 5 as conflict-introduced.

**Key fresh-eyes findings (5 substantive, all SURFACE-not-block).**

1. **F-FE-1 (LB-clarification).** Inv-20 clause-c AAD tuple now binds 8 elements
   `(codepoint, body-CID, sorted-member-DID-list, sender_did, stanza-index,
   member-key-generation, membership_set_id, membership_set_generation)`. The
   8-tuple composes correctly under TLV length-injectivity (F3) BUT the audit
   deliverable (SECURITY-PROOFS.md §-add) must state which dimensions are LIVE
   per-stanza vs which are constant-across-stanzas-within-one-envelope. Without
   that split, the kani injectivity harness risks under-specifying the
   per-stanza-distinct-but-envelope-constant decomposition that makes
   inter-member non-forgeability follow from EUF-CMA on the inner suite. **MINT
   audit-doc clarification, not a code change.** Confidence HIGH.

2. **F-FE-2 (MED — new Compromise candidate #57).** **RBAC × UCAN intersection-of-
   allows admits a "Viewer-Pinged-As-Writer" downgrade-deny pattern**: when a
   Viewer's UCAN chain CARRIES Write delegation but RBAC says Viewer, the denial
   is correct (RBAC floor) but the inverse — `change_role` op promoting a Viewer
   to Member without re-issuing UCAN chain — leaves the verifier in a state where
   `check_permission` says ALLOW (RBAC + UCAN both pass on next op) yet the
   admitting Authority had no visibility into the pre-existing UCAN attenuation
   shape. **Not a vulnerability**, but an honest-architectural-disclosure
   candidate: "UCAN-chain state is NOT reset by RBAC role transitions." Mint as
   Compromise #57 (RBAC-role-transition-does-not-invalidate-prior-UCAN-attenuations).
   Confidence MED — surface to Ben as MINT-or-SKIP. **Substrate-Guarantee-
   Disclosure class per P14**, not Accepted-Trade-Off.

3. **F-FE-3 (MED — invariant candidate Inv-21).** **MembershipSet-generation
   monotonicity at the per-member-DID partition is asserted (Inv-20 clause-e)
   but NOT pinned by an invariant separate from MembershipSet shape.** Under
   concurrent-fork tie-break (N4 R-N4-3a; lexicographically-smaller HLC wins;
   archived fork retained for audit), the per-member generation CRDT-vector
   could in principle accept an "archived-fork generation increment" from a
   losing fork if a member's local replica processed the losing fork before the
   tie-break decision propagated. **Recommend** minting **Inv-21
   (MembershipSet-fork-tie-break-invariant)**: archived-fork generation
   increments MUST NOT merge into winning-fork's CRDT-vector partition; the tie-
   break is a HARD partition boundary. Currently this is implicit in N4 R-N4-3a
   prose; a first-order invariant pins it for the kani harness + property tests.
   Confidence MED-HIGH. **MINT Inv-21 candidate; surface to Ben.**

4. **F-FE-4 (LOW — codepoint-reserve sub-slot variance composition).** The
   `RotatingGroupKeyChainedMode` sub-slot (Ben Option B, N4 R-N4-2) reserves
   {`NoChain`, `SsKChain`, `MlsChain`, `DcgkaChain`} variants. **Question:** how
   does the outer `AtriumWithRotatingGroupKey` codepoint + inner chained-mode
   codepoint compose for AAD-binding purposes when neither codepoint ships at
   v1-beta? The codepoint-table row exists (F1 dictates AAD-bind every
   codepoint), but no live AAD-binding test pin exercises the slot. **My-pred:
   ratify the codepoint-reserve slot but pin an explicit "future-ratification-
   gate" test arm asserting that AAD-binding shape is reserved + decode-strict
   per F2 even before impl lands.** Currently implicit. Bounded ~0.1 wave-day to
   add the gate. Confidence MED.

5. **F-FE-5 (LOW — IND-CCA2 composition under DeviceMesh-with-non-empty role-
   assignments).** Under Inv-20 clause-j (RBAC + UCAN intersection-of-allows),
   when DeviceMesh enforces `exactly-1 admin` (user-DID) but `role_assignments`
   may carry `Viewer` mappings for the user's OWN devices (e.g., a borrowed
   device limited to read-only), the device-key-wrap layer needs to commit to
   the role under which the wrap was performed. **Question:** does
   `member_key_rotation` op (N2 §3.1) rotate K_Set per-device-DID partition
   when a device's RBAC role changes from Member to Viewer? If not, a now-Viewer
   device retains decrypt capability for content sealed at Member-role —
   acceptable per Compromise #52 (MembershipSet no-PCS-against-removed-members
   semantic carries over) but NOT explicitly stated for the in-set role-
   downgrade case. **Recommend** clarifying #52 to subsume "role-downgrade
   without removal" semantic; OR mint a sub-#52 row. Confidence MED-HIGH on
   need-to-clarify; LOW on whether new # number warranted.

**Re-confirmations (substance).**

- **MembershipSet unification preserved at HIGH confidence.** M-CONS-v2 keeps
  the structural skeleton; N2's Authority slot is a strictly cleaner shape for
  cryptographic-modeling purposes (uniform admin-set means uniform signing-
  authority abstraction in proofs). **RE-CONFIRM HIGH.**
- **F+ NO-GO (10-of-10 prior C3 panel).** **RE-CONFIRM HIGH.** Nothing in M-CONS-v2
  changes the F+ analysis. The N2 RBAC + N1 RestrictedScopeSet additions
  strengthen the case AGAINST F+ (more attack-surface to model under typed-
  variant + uniform Authority + 3-role RBAC than F+ would have offered).
- **Path-A.5 preserved.** **RE-CONFIRM HIGH.** Inv-20 clause-f (Path-A.5 K(V)
  discipline at API boundary) is structurally unchanged across all 3 Kinds and
  under the future AtriumWithRotatingGroupKey codepoint-reserve. The
  immutable-Version-Node-CID property is invariant to MembershipSet shape; the
  rotation discipline is orthogonal.

**Open Ben-call signals (4 from M-CONS-v2 §9.3 + 2 new from this review).**

- Pre-existing 4: #56 mint / refresh_required_secs rename / N2-D8 traits-placement
  / Q1-D1 transport-scope clarification (all per M-CONS-v2 §9.3).
- **NEW from M-C3 v2:** Compromise #57 mint (F-FE-2; RBAC-role-transition-does-
  not-invalidate-UCAN) — surface to Ben as MINT or SKIP.
- **NEW from M-C3 v2:** Inv-21 mint (F-FE-3; fork-tie-break HARD partition
  boundary) — surface to Ben as MINT or absorb-into-Inv-20-clause-e.

**Confidence overall.** HIGH on directional reduction-style soundness;
MED-HIGH on the 5 fresh-eyes findings; HIGH on the 3 re-confirmations.

---

## §1 Task 1 — Fresh-eyes cryptographic-property assessment of COMPOSED whole

I build the mental model from first principles: a Benten encrypt-to-N-recipients
op produces an `EncryptedEnvelope` whose structure is dispatched by codepoint
(F1/F8/F11 codepoint-discipline), AAD-binds the 8-tuple per Inv-20 clause-c, uses
multi-stanza-HPKE-Encap to distribute `shared_key = K_Set` (Inv-20 clause-a), and
seals the body under per-Kind keying material with the per-stanza AAD committing
to (`sender_did`, `stanza_index`, `member_key_generation`, `membership_set_id`,
`membership_set_generation`). Layer-A vault wraps `K_principal`; Layer-B per-Node
AEAD binds to immutable Version-Node-CID per Path-A.5 (Inv-20 clause-f); Layer-C
single-recipient drops use the MultiRecipientSealing codepoint (paired Sealed-
Sender slot per F21 + Inv-18b); Layer-D multi-device-key-wrap + remote-permission-
call closes the device-fanout.

### §1.1 IND-CCA2 across all 4 layers + 3 MembershipSetKind variants?

**Layer-A (vault, K_principal store).** Uses Argon2idParameterTier per-Kind for
DeviceMesh+SingleDevice vault per F24. IND-CCA2 reduces cleanly to the underlying
AEAD (XChaCha20-Poly1305 or AES-GCM per codepoint) once codepoint + KDF inputs
are committed. **HIGH confidence — unchanged from M-CONS-v1.**

**Layer-B (per-Node AEAD; Path-A.5 K(V)).** K is derived from the immutable
Version-Node-CID. IND-CCA2 reduces to the inner AEAD assuming the KDF is a PRF
on the (master-secret, Version-Node-CID) pair (clear from Path-A.5 winner doc).
Inv-19 closure under MembershipSet primitive composition preserves the keying-
function CRDT-input discipline. **HIGH confidence.**

**Layer-C (multi-recipient drops via MultiRecipientSealing codepoint).** This is
where the M-CONS-v2 composition deserves the most fresh-eyes scrutiny because
MultiRecipientSealing now dispatches via MembershipSetKind discriminator into
3 distinct AAD-shape compositions:
- **Atrium-Kind:** sorted-member-DID-list ≤ 32 elements per F22 size-class
  buckets; AAD commits to the full DID-list (per F4); per-recipient unlinkability
  is achieved by per-stanza-distinct HPKE Encap output + per-stanza AAD shape.
  Adversary distinguishing recipient identity reduces to multi-recipient HPKE
  KEM-CCA2 security per RFC 9180. Standard.
- **DeviceMesh-Kind:** ≤ 5 element MemberKey set, ALL DeviceDid carrying
  DeviceAttestation. The per-stanza AAD additionally binds the device-attestation
  (implicit through MemberKey = DeviceDid). Adversary forging a stanza for a
  non-attested device reduces to EUF-CMA on the attestation signature scheme
  AND IND-CCA2 on HPKE base.
- **SingleDevice-Kind:** exactly-1 MemberKey (LocalDevice). Layer collapses to
  single-recipient HPKE-CCA2.

Under N2's Authority slot, the `authorities` set's cryptographic role is in
issuing signed mutations to `MembershipSet` shape (F19 generation CRDT-vector;
F-N2-A `MembershipEvent` audit-stream). The Authority signatures DO NOT directly
gate IND-CCA2 of Layer-C; they gate the integrity of the recipient SET itself
(Layer-C+1 / membership integrity layer). So IND-CCA2 of Layer-C is **NOT
weakened** by the N2 RBAC addition; the RBAC × UCAN intersection-of-allows acts
at the AUTHORIZATION boundary (who can call `encrypt_to_set`), not at the
CONFIDENTIALITY boundary (who can decrypt). **Clean separation; HIGH confidence.**

**Layer-D (DAK + multi-device-key-wrap + remote-permission-call).** Unchanged
from M-CONS-v1 under M-CONS-v2. Composes with DeviceMesh-Kind cleanly. **HIGH
confidence.**

**Per-Kind composition.** All 3 Kinds reduce to standard primitives (HPKE-Base
+ AEAD + signature). `AtriumWithRotatingGroupKey` codepoint-reserve does NOT
ship at v1-beta; F2 strict-decode rejects + does not under-mine v1-beta
IND-CCA2 closure. **HIGH confidence.**

**Verdict: IND-CCA2-secure across all 4 layers + 3 Kinds. HIGH.**

### §1.2 INT-CTXT-secure at envelope-format layer?

INT-CTXT requires that no adversary can produce a ciphertext under the same key
that decrypts to a non-prior-encrypted message. Under M-CONS-v2:
- F1 (codepoint in AAD) + F2 (strict-decode; no cross-variant fallback) + F3
  (TLV length-injectivity) + F14 (`aad_version: u8` prefix) compose to make the
  AAD shape injective + strict.
- F12 (nonce-length variant discrimination) prevents variant-confusion across
  AEAD ciphersuites.
- F4 (sender-DID + member-DID-list in AAD) + F5 (replay-window per-Kind table)
  compose under Inv-20 clause-c (8-tuple AAD) to make per-stanza AAD globally
  unique per (envelope, recipient-index, generation).
- INT-CTXT of each stanza reduces to INT-CTXT of the inner AEAD given AAD-
  injectivity. **HIGH confidence.**

**Subtle composition concern (F-FE-1; surfaced in §0).** The 8-tuple AAD has
varying-vs-constant decomposition across stanzas within one envelope: `codepoint,
body-CID, sorted-member-DID-list, sender_did, membership_set_id,
membership_set_generation` are envelope-constant; `stanza-index, member-key-
generation` are per-stanza-varying. The kani injectivity harness (F25) must
prove TWO disjoint properties: (a) AAD-tuple is globally injective across all
envelopes, (b) AAD-tuple uniquely identifies (envelope, stanza-index) WITHIN one
envelope's stanza-stream. F3 TLV length-injectivity gives (a); the
stanza-index varying-component gives (b). **Recommend audit-deliverable §-add
making this 2-part decomposition explicit.** **MINT audit-doc clarification.**

**Verdict: INT-CTXT-secure at envelope-format layer. HIGH; mint audit
clarification per F-FE-1.**

### §1.3 Replay-resistant per layer + per Kind?

**Replay-window per-Kind (F5).** Atrium uses a generation-CRDT-vector partition
per-member-DID + a bounded replay-window. DeviceMesh uses a smaller bounded
window with per-device serial-counters. SingleDevice uses a strict monotonic
sender-counter. All 3 compose under Inv-20 clause-e (generation-CRDT-vector
partition).

**Composition with N2 audit-log-query op (op-23).** The audit log stores
`MembershipEvent` records; an adversary replaying an old audit-record cannot
forge a new event because the event includes Authority signatures over event-
constant fields including timestamp + counter. Audit-log replay-resistance
reduces to EUF-CMA on the signing scheme. **HIGH.**

**Composition with iroh-gossip transport (M-CONS-v2 SCOPE-IN).** A gossip
adversary CAN replay stanzas at the network layer; the protocol-level F5 + Inv-
20 clause-e dedup at envelope-format layer above transport. The crucial property
is: replay-window dedup happens BEFORE the application sees a "fresh message"
event. iroh-gossip topic-name-canonicalization MUST NOT collapse stanzas with
different (envelope-CID, stanza-index) tuples into the same dedup key. **HIGH
confidence on protocol layer; MED confidence on iroh-gossip integration
preserving this — surface as Wave-MS-TRANSPORT e2e pin requirement (pim-2
substantive arm).**

**Composition with RBAC role-transition.** F-FE-5 (§0): a role-downgrade from
Member to Viewer for an in-set device retains decrypt capability for prior
content. This is NOT a replay-resistance concern (the device legitimately holds
K_Set for prior generations) but IS a confidentiality-against-role-downgrade
concern. **Surface as clarification of Compromise #52.**

**Verdict: replay-resistant. HIGH on protocol; MED on iroh-gossip integration
pending Wave-MS-TRANSPORT e2e pin.**

### §1.4 Inv-20 10-clause composition correct?

Walking each clause for composition errors:
- **(a) shared_key = K_Set via multi-stanza-HPKE-Encap.** Composes with HPKE-
  Base IND-CCA2. ✓
- **(b) FORK-ONLY rotation.** Composes with RotationTrigger doc-enum (N3); no
  semantic change. ✓
- **(c) per-stanza AAD 8-tuple binding.** Composes with F1+F2+F3+F4+F14+F12;
  load-bearing for inter-member non-forgeability per N4 R-N4-3. F-FE-1 audit
  clarification needed. ✓ with-note.
- **(d) per-recipient unlinkability.** Composes with size-class buckets F22 +
  AAD shape clause-c. ✓
- **(e) generation-CRDT-vector partition.** Composes with N4 R-N4-3a concurrent-
  fork tie-break. **F-FE-3 Inv-21 candidate** for the archived-fork partition
  boundary. ✓ with-mint-candidate.
- **(f) Path-A.5 K(V) discipline.** Composes orthogonally with MembershipSet
  primitive (key derivation is per-Version-Node-CID independent of Kind). ✓
- **(g) TransportConfig codepoint-reserve + iroh-gossip ships at v1β.**
  Composes; #53 narrowed. ✓
- **(h) homogeneous-per-Kind MemberKey + RestrictedScopeSet at
  Scope::RestrictedSelector.** RestrictedScopeSet algebra (union containment-lift)
  is decidable + transitive + reflexive per N1 §5.4 proptest at g-core-3w
  `5c2947c8`; verified via local grep at HEAD that `combinators::union` /
  `intersect` / `filter` are pub fns in `crates/benten-core/src/subgraph_spec/
  combinators.rs:99 / :37 / :140`. ✓ (grounded in code).
- **(i) Authorities slot + per-Kind cardinality at constructors.** Constructor-
  enforced runtime invariants composable with kani-harness coverage (F25
  audit-pack). ✓
- **(j) 3-role RBAC + UCAN intersection-of-allows.** Composes with benten-caps::
  chain_authority seam. **F-FE-2 Compromise #57 candidate** for RBAC-role-
  transition non-invalidating prior UCAN attenuations. ✓ with-mint-candidate.

**Net composition: HIGH on 10/10 clauses internally; 2 mint candidates surfaced
(Inv-21 + Compromise #57); 1 audit clarification mint (F-FE-1).**

### §1.5 Crypto-agility preserved under MembershipSet typed-variants + N2 Authority/RoleId?

CLAUDE.md baked-in #5 demands crypto-agility (codepoint-dispatch + version-
discriminated transitions). M-CONS-v2 preserves:
- **MembershipSetEncryption codepoint family** (F8): per-Kind discriminator
  allows per-Kind crypto-suite migration without wire-format break.
- **N1 RestrictedScopeSet backward-compat codepoint-dispatch** (F28): single-
  scope-codepoint = old shape; multi-scope-codepoint = new shape; both accepted
  for one release. ✓
- **N2 RoleId enum (Admin/Member/Viewer)** + PermissionSet bitflags: codepoint
  reservations 0/1/2 for RoleId + `1<<3..1<<31` reserved for PermissionSet per
  R0 §13. Richer role systems = additive codepoint (Phase-4-Meta-Composing+)
  per Inv-20 clause-j. ✓
- **AtriumWithRotatingGroupKey + RotatingGroupKeyChainedMode sub-slot** (N4):
  codepoint-reserve preserves optionality on inner-variant choice. F-FE-4
  (codepoint-reserve sub-slot AAD-bind future-ratification-gate test arm)
  recommended. ✓ with-recommendation.

**Verdict: crypto-agility preserved. HIGH.**

### §1.6 Freeze-permanence under codepoint reservations?

All codepoint reservations are additive (HALT-AND-SURFACE-TO-BEN discipline per
§15.c amendment); reserved codepoints decode-strict-reject (F2) at v1-beta. No
freeze-permanence violation. **HIGH.**

### §1.7 N1 RestrictedScopeSet composes with K(N) chain?

`RestrictedScopeSet` lives at the GRANT boundary (benten-caps::scope `Scope::
RestrictedSelector` arm). K(N) Path-A.5 lives at the ENCRYPTION boundary (Layer-
B per-Node AEAD; key derived from immutable Version-Node-CID). The two are
orthogonal at the `(MembershipSet, RestrictedScopeSet)` seal-seam tuple per N1
§3.6 row.

**Subtle composition.** RestrictedScopeSet enables grants over UNIONS of
sub-graphs. K(N) keys each Node independently. A grant `RestrictedScopeSet(A ∪
B)` does NOT key-share a single material; the grantee receives access to
`{K(N_i)}` for N_i in the underlying Nodes reachable from `A ∪ B`. **The K(N)
chain is per-Node-independent**, so `union` of grant-sets is union of
key-discovery-sets, not union of keying-material. ✓

**Adversary scenario:** can a grantee with `RestrictedScopeSet(A)` collude with
a grantee with `RestrictedScopeSet(B)` to access `A ∩ B` content? Yes,
trivially — that's the intended grant-composition semantic. Can they access
`A ∪ B` content not in either grant? No, because each Node's K(N) is derived
from its CID independently. **Composition is sound.** HIGH.

---

## §2 Task 2 — Re-evaluate Ben's ratifications in M-CONS-v2 context

### §2.1 Q1 iroh-gossip scope-in: composes with N1 RestrictedScopeSet grant-time composition + N2 audit_log_query op?

**Composition check.** iroh-gossip ships at v1-beta behind `GossipPlusBlobs`
codepoint per F27. The +5–8 wave-day Wave-MS-TRANSPORT addition contains:
peer-discovery + topic-per-MembershipSet + multi-stanza-HPKE-envelope pub/sub +
NAT-traversal + relay-fallback + e2e pin.

- **vs N1 RestrictedScopeSet:** RestrictedScopeSet operates at grant-time; iroh-
  gossip operates at transport-time. The two are at different layers; no
  interaction. ✓
- **vs N2 audit_log_query op:** the audit-log is a `MembershipSet` LOCAL state
  surface; iroh-gossip is the WIRE-DISTRIBUTION substrate of MembershipEvents.
  Composition: gossip-replayed audit-events are deduped at protocol layer per
  F5 + Inv-20 clause-e BEFORE audit-log mutation. The audit_log_query op reads
  AFTER dedup. ✓ — but the M-CONS-v2 doc does not explicitly state this
  ordering. **Recommend audit-doc clarification: dedup-before-audit-log-mutation
  ordering.** Bounded.

**Verdict: CONFIRM-WITH-EVIDENCE.** No conflict introduced by integration.
**Recommendation:** add ordering clarification to MembershipSet-Spec
(dedup-before-audit-log-mutation).

### §2.2 Q3 DUAL-CID: composes with N1 RestrictedScopeSet + N2 dedup-blind ops?

Q3 DUAL-CID: `envelope_blob_cid` + `plaintext_cid_set` (HMAC-blinded); recipient
computes BLAKE3-of-plaintext locally as impl detail; `plaintext_cid_local`
LOCAL-ONLY per F18.

- **vs N1 RestrictedScopeSet:** RestrictedScopeSet narrows grant-scope but
  DUAL-CID is at envelope-layer (a per-envelope shape). No interaction. ✓
- **vs N2 dedup_blind_cid op (op-10 in v1β-LB classification):** the dedup-blind
  op uses the HMAC-blinded `plaintext_cid_set` to detect duplicate-content
  encryptions without revealing plaintext content. Under DUAL-CID, the HMAC key
  is keyed off `(K_Set, generation)`. **Subtle composition concern:** when
  K_Set rotates (FORK-ONLY rotation per Inv-20 clause-b), the HMAC key changes,
  so dedup detection scope is bounded per-generation. **This is the correct
  semantic** (cross-generation dedup would leak that the same plaintext was
  encrypted under both generations). **Acceptable. ✓**

**Verdict: CONFIRM-WITH-EVIDENCE.** DUAL-CID + dedup-blind compose
correctly. No surfacing needed.

### §2.3 Am4 Sealed-Sender DEFAULT: correct for all 3 Kinds under N2 Authority/RoleId?

Am4: Sealed-Sender DEFAULT (per F21 + Inv-18b paired-slot discipline).

- **Atrium-Kind:** Sealed-Sender hides sender_did from intermediaries while
  preserving inter-member non-forgeability via clause-c AAD-bind-of-sender_did
  AFTER seal-open. Sealed-Sender DEFAULT = correct. ✓
- **DeviceMesh-Kind:** All members are user-controlled devices; Sealed-Sender
  hiding sender device_did from other devices is questionable (the user owns
  all 5 devices; hiding doesn't add adversary protection within the user's
  threat model). **Subtle observation:** Sealed-Sender DEFAULT may be over-
  conservative for DeviceMesh, but is NOT incorrect — it preserves a useful
  property (server-side observer cannot distinguish which device sent) at no
  cost. ✓ with-note (defensible to leave as DEFAULT).
- **SingleDevice-Kind:** trivially the sender IS the (only) recipient; Sealed-
  Sender is a no-op semantic but not harmful. ✓

**Verdict: CONFIRM-WITH-EVIDENCE.** Am4 holds for all 3 Kinds. DeviceMesh
observation is a NIT (could be sub-Kind-Configurable post-v1-beta) but does not
contest the DEFAULT ratification.

### §2.4 Path-A.5: K(N)-to-immutable-Version-Node-CID under all 3 Kinds?

Path-A.5 ratification: K is keyed off immutable Version-Node-CID; Anchor +
Version + CURRENT preserved; Path-B DISAGREE-WITH-REASONING.

- **Atrium-Kind:** K(N) per immutable Version-Node-CID, with K_Atrium-distribution
  via MembershipSet primitive (shared_key in clause-a). ✓
- **DeviceMesh-Kind:** K(N) per immutable Version-Node-CID; multi-device-key-wrap
  layer distributes per-device. ✓
- **SingleDevice-Kind:** K(N) per immutable Version-Node-CID; single-device
  local-only key. ✓

Under N2 typed-variant + N1 grant-composition + N4 codepoint-reserve rename: no
clause of Path-A.5 K(V) discipline interacts with the post-M-CONS-v1 additions.
Inv-20 clause-f remains structurally invariant.

**Verdict: CONFIRM-WITH-EVIDENCE.** Path-A.5 holds across all 3 Kinds + future
AtriumWithRotatingGroupKey codepoint-reserve. HIGH.

### §2.5 Net Ben-ratification audit

| Ratification | Composition status under M-CONS-v2 | Action |
|---|---|---|
| Q1 libcrux-ml-kem + iroh-gossip SCOPE-IN | Preserved + ordering-clarification recommended (§2.1) | Recommend doc-add |
| Q2 BE codepoint endianness | Preserved (unchanged) | None |
| Q3 DUAL-CID | Preserved; dedup-blind composes correctly | None |
| Q4 HPKE-11-KE commit + JOSE NAMED-deferred | Preserved (unchanged) | None |
| Am4 Sealed-Sender DEFAULT | Preserved; DeviceMesh observation is a NIT | None substantive |
| Path-A.5 | Preserved across all 3 Kinds | None |
| MembershipSet typed-variant primitive | Preserved + strengthened by N2 Authority | None |
| N2 24-op classification | Preserved; my-pred MINT #57 candidate (§0 F-FE-2) | Surface MINT |
| Compromise #55 P2P-by-design framing | Preserved | None |
| Option B chained-mode sub-slot | Preserved + future-ratification-gate test recommended (F-FE-4) | Bounded gate-add |
| AtriumWithRotatingGroupKey rename | Preserved (mechanical) | None |

**Net: 0/11 ratifications introduce composition conflict.** 4/11 attract bounded
recommendations (Q1 ordering clarification + N2 mint candidate + Option B gate
+ F-FE-1 audit clarification). All are SURFACE-not-block.

---

## §3 Task 3 — What did M-CONS-v2 structurally MISS?

### §3.1 New Compromise candidates from recent literature

**Candidate Compromise #57 (RBAC-role-transition-does-not-invalidate-UCAN; F-FE-2).**
Already surfaced above. Substrate-Guarantee-Disclosure class per P14.

**Candidate Compromise #58 (KEM-Key-Confirmation under multi-stanza-HPKE-Encap;
LITERATURE).** Recent literature (Bernstein-Persichetti 2023; Bhargavan-Cremers-
Mavridis 2024 on hybrid KEM key-confirmation) flags subtle "key-confirmation
failure across the multi-stanza fan-out" as a source of mis-attributed decrypt
errors. Benten's HPKE-11-KE commit (Q4) gives EUF-CMA at construction; the
multi-stanza shape gives multiple Encap outputs per envelope. If one stanza
decap fails (e.g., recipient device-key drift) but other stanzas succeed, the
sender has no key-confirmation that ALL recipients decapped successfully.

**My-pred: NOT a substantive vulnerability** (the recipient-side decap-failure
is bounded by F2 strict-decode + F6 Bernstein-Persichetti CT mitigation already
covered under Compromise #32); but **honest disclosure** that multi-recipient
envelopes lack global-key-confirmation is auditable. **MINT candidate (low
priority); surface to Ben as MINT-or-SKIP.** Substrate-Guarantee-Disclosure class.

**Candidate Compromise #59 (MembershipSet-fingerprint-leak via gossip topic
membership; M-CONS-v2 Q1 SCOPE-IN).** Iroh-gossip topic-per-MembershipSet (per
§7.4) means a passive gossip-network observer can enumerate WHICH gossip-topics
a device subscribes to. Even with per-relay-unlinkability (F22), the
correlation of (device-IP, topic-membership-pattern) leaks set-membership
metadata. **This is already implicit in #48 (MembershipSet-shape-leak) at the
shared-key-compromise threshold, but #59 is at the WEAKER threshold of
passive-network-observation.** Distinct attack model.

**My-pred: MINT.** This is the canonical "gossip-substrate-side-channel" honest-
disclosure that the Wave-MS-TRANSPORT scope-in introduces. Auditors will ask;
better to disclose proactively. Surface to Ben as MINT.

### §3.2 New invariant candidates

**Candidate Inv-21 (fork-tie-break HARD partition boundary; F-FE-3).** Already
surfaced above.

**Candidate Inv-22 (authority-set-change emits MembershipEvent with prior-
authority-signature).** N2's Authority slot allows authorities to be
admitted/revoked, but the M-CONS-v2 doc does not explicitly state the
"authority-change op MUST be signed by a prior-state authority" invariant. The
constructor invariants (§1.2) cover initial state; mutation-op validation
covers steady state; but the bootstrapping of "who signs the first admission
of a new authority" is implicit. **Recommend** Inv-22 candidate naming this
explicitly: every Authority-set mutation in a MembershipSet emits a
MembershipEvent signed by a current-set-Authority (cardinality + role per
Kind). Auditor-clear; bounded.

**My-pred: MINT Inv-22 candidate; surface to Ben.** Confidence MED-HIGH.

### §3.3 Subtle composition concerns

**Concern §3.3.1 — gossip-replay vs Atrium-fork-rotation race.** When an Atrium
forks (FORK-ONLY rotation per Inv-20 clause-b) and the new MembershipSet
emerges, in-flight stanzas from the OLD MembershipSet may still be propagating
through iroh-gossip. A recipient who has updated to the NEW MembershipSet may
receive an old stanza encrypted under the OLD K_Set. The recipient's local
state must retain decap capability for the OLD K_Set until a quiescence-
boundary is observed. **The M-CONS-v2 doc does not explicitly enumerate the
"retain-old-K_Set-until-quiescence" requirement.** This is a Wave-MS-TRANSPORT
implementation detail but a SECURITY-POSTURE.md disclosure candidate.

**My-pred: add to Wave-MS-TRANSPORT e2e pin requirements** (pim-2 substantive
arm; bounded ~0.1 wave-day).

**Concern §3.3.2 — RoleId 0/1/2 codepoint stability under future RBAC
extensions.** N2 reserved `1<<3..1<<31` for PermissionSet, but the RoleId enum
itself (3 variants at v1-beta) does not explicitly state reservation of RoleId
codepoints 3..255 for future role additions. **Recommend** explicit codepoint
reservation per CLAUDE.md baked-in #5; bounded ~0.05 wave-day. **Already
implicit but make explicit.**

**Concern §3.3.3 — MembershipSetMetadata not in AAD?** F-N2-C places
`MembershipSetMetadata { name, description, created_by_human_label }` inside
`MembershipSetPolicy`. **Question:** does the name/description bind via
clause-c AAD-tuple to envelope contents? **My read:** no — the metadata is
mutable via `update_metadata` op (admin-gated) and is local-display surface,
not security-relevant. Binding to AAD would prevent rename without forcing
re-encryption. **Acceptable to NOT bind to AAD**, but worth explicit
audit-doc statement to prevent future confusion. **Recommend audit-doc
clarification.**

### §3.4 Alternative architectural framings from composed whole

**Framing: the 4-Layer Benten model + MembershipSet primitive resembles
"hierarchical AEAD-with-publisher-set" pattern** that has appeared in recent
literature (Mavridis-Cremers 2024 on hybrid AEAD-fan-out). Benten's specific
shape — typed-variant Kind + uniform Authority + 3-role RBAC + UCAN × RBAC
intersection-of-allows + per-stanza AAD 8-tuple — is more conservative than
the typical fan-out pattern (no shared-secret derivation tree; FORK-ONLY
rotation; per-recipient unlinkability without sender-key-derivation
broadcasts). **The conservative posture is a strength, not a weakness.** The
M-CONS-v2 doc does not benchmark Benten against this literature shape; the
SSK-comparison §-row (N4 R-N4-3) is the closest. **Recommend extending the
SSK-comparison §-row to a broader "MembershipSet-shape vs literature-shapes
comparison" §** at audit-deliverable level. Bounded ~0.3 wave-day.

### §3.5 Specific R0 plan-doc structural advice

For M-R0 author (R0 plan-doc owner downstream):

1. **§12 SECURITY-PROOFS.md skeleton:** add explicit "AAD-tuple injective
   decomposition" sub-§ separating envelope-constant vs per-stanza-varying
   components (per F-FE-1). Bounded; cleans up kani harness scope.

2. **§14 ENVELOPE-V1-SPEC.md skeleton:** add ordering specification "dedup-
   before-audit-log-mutation" (per §2.1 above).

3. **§11 THREAT-MODEL.md skeleton:** add Compromise #57 + #58 + #59 if minted
   (per §3.1); add Inv-21 + Inv-22 if minted (per §3.2); add MembershipSet-
   shape vs literature-shapes comparison §-row (per §3.4).

4. **§9 TransportConfig:** add Wave-MS-TRANSPORT e2e pin requirement for
   gossip-replay vs fork-rotation race (per §3.3.1); add gossip-topic
   metadata-leak disclosure (per Compromise #59 if minted).

5. **§2.5 Authority + RoleAssignments + RBAC:** add explicit RoleId codepoint
   reservation 3..255 (per §3.3.2); add MembershipSetMetadata-not-in-AAD
   audit-doc clarification (per §3.3.3); add UCAN-attenuations-not-reset-by-
   role-transition disclosure (per Compromise #57 if minted).

6. **§17 Decisions recorded:** add Inv-22 mint if ratified; add #57 / #58 / #59
   mints if ratified.

---

## §4 Task 4 — Re-confirm or contest

### §4.1 MembershipSet unification (M-CONS-v1 ratified; M-CONS-v2 preserved)

**RE-CONFIRM HIGH.** Three reasons:
1. Cryptographic-modeling-cleanliness — typed-variant Kind + uniform Authority
   gives a single abstraction for proofs across 3 Kinds, with per-Kind
   instantiation in constructor invariants.
2. Composition with N1 RestrictedScopeSet is orthogonal at the seal-seam tuple
   (verified §1.7); MembershipSet governs K_Set distribution + RestrictedScopeSet
   governs sub-graph walk-shape; no cross-interaction.
3. Composition with N2 RBAC × UCAN is at AUTHORIZATION boundary, not
   CONFIDENTIALITY boundary; IND-CCA2 of Layer-C is preserved (§1.1).

### §4.2 F+ NO-GO (10-of-10 by prior C3)

**RE-CONFIRM HIGH.** M-CONS-v2's additions (N1 RestrictedScopeSet + N2 RBAC + 4
new ops + Authority slot) STRENGTHEN the case AGAINST F+ because more attack
surface gets cryptographic modeling under the typed-variant primitive than F+
would have offered. F+ would have required modeling N orthogonal primitives
(one per fanout site); the typed-variant gives 1 primitive + per-Kind
specialization. The post-M-CONS-v2 attack surface is larger but more
structured. F+ remains NO-GO.

### §4.3 Path-A.5 (preserves Benten value-prop)

**RE-CONFIRM HIGH.** Path-A.5's value-prop (immutable-Version-Node-CID +
Anchor + Version + CURRENT preserved) is structurally invariant to MembershipSet
shape (§2.4). No M-CONS-v2 amendment touches Path-A.5 derivation; Inv-20
clause-f remains the API boundary enforcement.

### §4.4 N2 24-op classification (Ben-ratified)

**CONFIRM-WITH-EVIDENCE.** The 11 v1-beta-LB ops cover the legitimate primitive
surface; the 6 codepoint-reserve ops preserve future-shape; the 3 P4MC + 4
post-v1β are NAMED-deferred per HARD RULE 12. My-pred MINT Compromise #57
(F-FE-2) does not contest the classification; it adds a disclosure to the
NOW-shipping 11 v1-beta-LB ops.

### §4.5 Compromise #55 P2P-by-design framing (Ben-ratified)

**CONFIRM-WITH-EVIDENCE.** This framing is cryptographically correct — content-
addressed P2P CANNOT unilaterally erase already-replicated content (that's the
substrate guarantee). Audit firms processing this disclosure under the
Substrate-Guarantee-Disclosure class per P14 will see it as a property
documented, not a vulnerability accepted.

### §4.6 Option B codepoint-reserve for per-message ratcheting (Ben-ratified)

**CONFIRM-WITH-EVIDENCE.** The chained-mode sub-slot under
`AtriumWithRotatingGroupKey` preserves optionality on inner-variant choice
{`NoChain`, `SsKChain`, `MlsChain`, `DcgkaChain`} without baking in choice.
F-FE-4 future-ratification-gate test recommendation is the only bounded
addition.

### §4.7 AtriumWithCGKA → AtriumWithRotatingGroupKey rename (Ben-ratified)

**CONFIRM-WITH-EVIDENCE.** Neutral name; covers SSK / MLS / DCGKA shapes;
un-bakes the future-shape commitment. P13 truth-in-naming-for-codepoint-
reserves codification is sound.

---

## §5 Task 5 — Anything else fresh-eyes surfaces

### §5.1 Subtle composition concerns (consolidated)

- **§3.3.1 gossip-replay vs fork-rotation race.** Add to Wave-MS-TRANSPORT e2e
  pin. Bounded.
- **§3.3.2 RoleId codepoint stability 3..255 reservation.** Add to R0 §13
  CRYPTO-CODEPOINTS.md skeleton. Bounded.
- **§3.3.3 MembershipSetMetadata-not-in-AAD audit-doc clarification.** Bounded.

### §5.2 Missing-in-design properties

- **Per-stanza-AAD-tuple injective decomposition (envelope-constant vs
  per-stanza-varying)** — F-FE-1 audit clarification. Bounded.
- **Authority-set-change emit-discipline** — Inv-22 candidate. MED-HIGH.

### §5.3 New Compromise # mint candidates

- **#57 RBAC-role-transition-does-not-invalidate-UCAN.** Substrate-Guarantee-
  Disclosure class. F-FE-2. MED — surface to Ben.
- **#58 KEM-key-confirmation-failure across multi-stanza fan-out.**
  Substrate-Guarantee-Disclosure class. LOW priority; surface to Ben as
  MINT-or-SKIP.
- **#59 Gossip-topic-membership-fingerprint via Wave-MS-TRANSPORT iroh-gossip.**
  Substrate-Guarantee-Disclosure class. MED-HIGH — auditor will ask; surface to
  Ben as MINT (RECOMMENDED).

### §5.4 New invariant candidates

- **Inv-21 fork-tie-break HARD partition boundary.** F-FE-3. MED-HIGH —
  surface to Ben as MINT or absorb-into-Inv-20-clause-e.
- **Inv-22 authority-set-change emit-discipline.** §3.2. MED-HIGH — surface to
  Ben as MINT.

### §5.5 Specific R0 plan-doc advice (re-collected)

Per §3.5 above. 6 enumerated R0 §-add items, all bounded.

### §5.6 Process-level observation

The M-CONS-v2 + N1–N4 pipeline pattern (P17 codified in §11.7 of M-CONS-v2 doc)
is itself the load-bearing process discovery. **Fresh-eyes observation:** the
pipeline pattern revealed structural generalizations (N1 combinator-surface-
promotion; N2 Authority slot) that the convergent M-panel missed. **My-pred:**
codify P17 as a phase-close DISCIPLINE not a phase-close PATTERN — i.e., for
every major primitive consolidation, the orchestrator's default expectation is
"convergent-panel + Ben-refinement-pass + critique-round" not just "convergent-
panel + critique-round." This sharpens M-CONS-v2 P17.

---

## §6 Self-assessment + confidence per finding

| Finding | Confidence | Reasoning |
|---|---|---|
| §0 + §1.1 IND-CCA2 across 4 layers + 3 Kinds | **HIGH** | Each layer reduces to standard primitives (HPKE-Base + AEAD + signature); RBAC × UCAN at authz boundary not confidentiality boundary; cleanly composed |
| §0 + §1.2 INT-CTXT at envelope-format | **HIGH (with F-FE-1 mint)** | F1/F2/F3/F12/F14 + Inv-20 clause-c compose under AAD-tuple injectivity; F-FE-1 audit clarification needed for kani-harness scope |
| §0 + §1.3 Replay-resistance | **HIGH on protocol; MED on iroh-gossip integration** | F5 + Inv-20 clause-e dedup before audit-log; iroh-gossip e2e pin required (pim-2) |
| §0 + §1.4 Inv-20 10-clause composition | **HIGH on 10/10 internally; 2 mint candidates surfaced** | Inv-21 + Compromise #57 candidates emerged from clause-e + clause-j composition |
| §1.5 Crypto-agility | **HIGH** | Codepoint-dispatch preserved; backward-compat via single-vs-multi-codepoint at F28 |
| §1.6 Freeze-permanence | **HIGH** | All codepoint reservations additive + decode-strict-reject |
| §1.7 N1 K(N) composition | **HIGH** | Per-Node-independent K(N) derivation; union-of-grants = union-of-key-discovery-sets |
| §2.1–2.4 Ben-ratification audit (Q1/Q3/Am4/Path-A.5) | **HIGH on each; 0/11 introduce conflict** | 4/11 attract bounded recommendations (audit clarifications + gate tests); none contest |
| §3.1 Compromise mint candidates (#57/#58/#59) | **MED on #57+#58; MED-HIGH on #59** | #59 (gossip-topic fingerprint) directly emerges from Q1 SCOPE-IN; auditor will ask |
| §3.2 Invariant mint candidates (Inv-21/Inv-22) | **MED-HIGH on Inv-21; MED-HIGH on Inv-22** | Both grounded in M-CONS-v2 composition gaps; absorbable into Inv-20 if Ben prefers |
| §3.3 Subtle composition concerns (3) | **HIGH (all bounded recommendations)** | None block; all <0.5 wave-day fixes |
| §3.5 R0 plan-doc structural advice (6 items) | **HIGH on each; all bounded** | Each is a specific §-add to existing R0 skeleton sections |
| §4.1–4.7 Re-confirmation of 7 prior ratifications | **HIGH on each (6/7); MED-HIGH on N2 24-op (mint-candidate caveat)** | M-CONS-v2 composition strengthens or preserves each |

### §6.1 What I could be wrong about

- **Compromise #58 (KEM-key-confirmation-failure)** may be over-engineered if
  the multi-stanza per-recipient decap-failure is fully covered by Compromise
  #32 (Bernstein-Persichetti CT). My-read is they're distinct attack models
  (CT mitigation = side-channel; key-confirmation = correctness disclosure)
  but defensible to skip.
- **Inv-22 authority-set-change emit-discipline** may be subsumable into Inv-20
  clause-i (Authority slot + per-Kind cardinality). My-pred separate Inv-22
  for kani-harness clarity, but absorption is defensible.
- **Compromise #57 (RBAC-role-transition-does-not-invalidate-UCAN)** may be
  judgment call. My-pred MINT because audit firms processing the RBAC × UCAN
  composition will ask "what happens to in-flight UCAN attenuations on
  role-transition?" — and the answer is "they're independent until re-issued."
  Audit-clear via explicit disclosure.
- **MembershipSet-shape vs literature-shapes comparison §-row** may be
  redundant with N4 R-N4-3 SSK-comparison §-row. My-pred extend not replace.
- **Wave-MS-TRANSPORT gossip-replay vs fork-rotation race** may be already
  covered in M6 §10.1 footgun-mitigation items; my-read it's not explicit, but
  my coverage of M6 is via M-CONS-v2 § references not direct read.

### §6.2 What this critique does NOT cover

- **Re-running the M-CONS-v2 reconciliation.** I take Ben-ratifications + M-CONS-v2
  integration as inputs.
- **Authoring full R0 plan-doc content.** M-R0 owns substantive content.
- **M-C1 elegant-shape + M-C2 composability critiques.** Those are sibling
  agents; I only run the cryptographic-property + adversary-model lens.
- **Per-amendment doc-prose at audit-deliverable level.** Each F-row needs
  prose; I surface § structure, not prose.
- **Wave-MS-TRANSPORT iroh-gossip integration per-task plan.** Wave execution
  scope.

### §6.3 Convergence verdict (cryptographer-level)

The M-CONS-v2 composed whole is **CRYPTOGRAPHICALLY SOUND** with:
- HIGH confidence on directional IND-CCA2 + INT-CTXT + replay-resistance
  + crypto-agility + freeze-permanence reductions.
- 0/11 Ben-ratifications introduce conflict under M-CONS-v2 integration.
- 5 fresh-eyes findings, all SURFACE-not-block; 3 new Compromise mint
  candidates (#57 / #58 / #59); 2 new invariant mint candidates (Inv-21 /
  Inv-22); 6 R0 plan-doc structural recommendations; 3 subtle composition
  concerns; all bounded.
- 3 prior ratifications RE-CONFIRMED HIGH: MembershipSet unification + F+ NO-GO
  + Path-A.5.

**The M-CONS-v2 output is DECISION-READY for next-stage R0 plan-doc authoring**
with the surfaced mint candidates + recommendations integrated.

---

## §7 Summary table — fresh-eyes findings

| # | Finding | Class | Action | Confidence |
|---|---|---|---|---|
| F-FE-1 | AAD-tuple injective decomposition (envelope-constant vs per-stanza-varying) | LB-clarification (audit-doc) | Add R0 §12 sub-§ + kani harness 2-part scope | HIGH |
| F-FE-2 | RBAC role-transition does NOT invalidate prior UCAN attenuations | Substrate-Guarantee-Disclosure | Mint Compromise #57; surface to Ben as MINT-or-SKIP | MED |
| F-FE-3 | Fork-tie-break HARD partition boundary (archived-fork generation must NOT merge into winning-fork CRDT-vector partition) | New invariant | Mint Inv-21; surface to Ben as MINT-or-absorb | MED-HIGH |
| F-FE-4 | Codepoint-reserve sub-slot AAD-binding future-ratification-gate test | Bounded gate | Add test arm pinning AAD-bind shape pre-impl | MED |
| F-FE-5 | RBAC role-downgrade Member→Viewer retains decrypt cap for prior-content; subsume into Compromise #52 | Audit-clarification | Extend #52 prose to subsume role-downgrade case | MED-HIGH |
| C-FE-58 | KEM-key-confirmation-failure across multi-stanza fan-out (Mavridis-Cremers 2024 literature line) | Substrate-Guarantee-Disclosure | Mint Compromise #58; surface to Ben as MINT-or-SKIP | LOW |
| C-FE-59 | Gossip-topic-membership-fingerprint via Wave-MS-TRANSPORT | Substrate-Guarantee-Disclosure | Mint Compromise #59; surface to Ben as MINT (RECOMMENDED) | MED-HIGH |
| I-FE-22 | Authority-set-change emit-discipline | New invariant | Mint Inv-22; surface to Ben as MINT-or-absorb | MED-HIGH |
| O-FE-1 | Q1 ordering clarification (dedup-before-audit-log-mutation) | R0 §-add | Add MembershipSet-Spec ordering note | HIGH |
| O-FE-2 | RoleId codepoint 3..255 reservation | R0 §13 CRYPTO-CODEPOINTS.md add | Explicit codepoint reservation | HIGH |
| O-FE-3 | MembershipSetMetadata-not-in-AAD audit-doc clarification | R0 §-add | Audit-doc statement | HIGH |
| O-FE-4 | Gossip-replay vs fork-rotation race; Wave-MS-TRANSPORT e2e pin | Wave-pin | Add to pim-2 substantive-arm | MED-HIGH |
| O-FE-5 | MembershipSet-shape vs literature-shapes comparison §-row (extend N4 SSK-comparison) | R0 §11 add | Audit-deliverable §-row | MED |
| RE-1 | MembershipSet unification preserved at HIGH (re-confirm) | Re-confirmation | None (informational) | HIGH |
| RE-2 | F+ NO-GO re-confirmed (re-confirm) | Re-confirmation | None (informational) | HIGH |
| RE-3 | Path-A.5 preserved across 3 Kinds (re-confirm) | Re-confirmation | None (informational) | HIGH |

---

## §8 Surface-to-Ben bundle

**RECOMMENDED surfacing (5 substantive items + 4 bounded R0 mechanical):**

1. **Mint Compromise #59 (gossip-topic-membership-fingerprint).** STRONGLY
   RECOMMENDED — auditor will ask given Q1 SCOPE-IN.
2. **Mint Inv-21 (fork-tie-break HARD partition boundary)** or absorb into
   Inv-20 clause-e. RECOMMENDED.
3. **Mint Compromise #57 (RBAC role-transition does NOT invalidate UCAN
   attenuations).** RECOMMENDED — Substrate-Guarantee-Disclosure class.
4. **Mint Inv-22 (authority-set-change emit-discipline)** or absorb into
   Inv-20 clause-i. RECOMMENDED.
5. **Extend Compromise #52 prose to subsume RBAC role-downgrade case.**
   RECOMMENDED — audit clarification.
6. **(optional, low-priority)** Mint Compromise #58 (KEM-key-confirmation-
   failure). MINT-or-SKIP per Ben judgment.
7. **R0 mechanical adds:** §12 AAD-tuple injective decomposition sub-§; §14
   dedup-before-audit-log-mutation ordering note; §13 RoleId codepoint 3..255
   reservation; §11 MembershipSet-shape-vs-literature-shapes §-row.

**Net Ben-surface bundle (combined with M-CONS-v2 §9.3's pre-existing 4):**
- M-CONS-v2 pre-existing: #56 mint / refresh_required_secs / N2-D8 traits /
  Q1-D1 transport scope. (4)
- M-C3 v2 new: #57 / #58 / #59 / Inv-21 / Inv-22 + #52 prose extension. (6)
- M-C3 v2 R0 mechanical: 4 R0 §-adds. (4)

**Total ~14 surfaceable items**, all bounded, none arch-fork-class. All 14
surfaceable in 1–2 Ben passes. M-C1 + M-C2 critiques may add more.

---

## §9 Citations

### §9.1 M-CONS-v2 + background artifacts (frozen SHAs)

- M-CONS-v2 consolidator: `phase-4-meta-core/membership-set-m-cons-v2-consolidator
  @ e62ff540` → `.addl/phase-4-meta/membership-set-m-cons-v2-consolidator.md`
  (1295 LOC; read in full pass)
- M-CONS-v1 consolidator: `74580ee6` (referenced via M-CONS-v2 §13.1)
- N1: `ed592770`; N2: `a1b5a552`; N3: `298c80d9`; N4: `2ee24e9f`
- M2: `6170980b`; M3: `1ba3a4c3`; M4: `066785b5`; M5: `15819500`; M6: `e5046c87`
- M1a: `3618e051`; M1b: `1816ea60`; M1c: `50eb901d`
- 9-eyes consolidated registry: `fbdfeb16`; critique-round c1–c5: `3f5a4351 /
  9c548e5f / 79c99aa5 / 6ea9718a / 8e374a9d`
- Q3 Option D: `23f76e24`; AtriumPolicy: `e90900b4`; Path-A.5 winner: `5f50a028`

### §9.2 Tree-pinned (HEAD `2172cb6d`; verified at session start)

- `crates/benten-caps/src/scope.rs` — `Scope::{Hashes | RestrictedSelector}`
- `crates/benten-caps/src/restricted_spec.rs` — `RestrictedScope` 6-dim
- `crates/benten-caps/src/chain_authority.rs` — UCAN seam (RBAC × UCAN composition site)
- `crates/benten-core/src/subgraph_spec/combinators.rs` — pub fns `intersect` (:37),
  `union` (:99), `filter` (:140) (verified via local grep at HEAD)

### §9.3 CLAUDE.md baked-in + MEMORY.md disciplines

- CLAUDE.md baked-in #1 (12-primitive irreducibility — preserved)
- CLAUDE.md baked-in #5 (crypto-agility — codepoint-dispatch backward-compat
  preserved + RoleId codepoint reservation 3..255 recommended)
- CLAUDE.md baked-in #15 (v1-beta interface freeze — F28 + F-N2-A/B/C additive)
- CLAUDE.md baked-in #17 (multi-process + device-shape — DeviceMesh per-Kind
  invariants preserved)
- CLAUDE.md baked-in #18 (4-identity-concepts + plugin trust model — MemberKey
  homogeneous-per-Kind preserved)
- CLAUDE.md baked-in #19 (engine-level extensions vs app-level plugins — N2
  RBAC at engine-level justified)
- `feedback_engine_primitives_vs_application_layer.md` — N1 honors via
  combinator-promotion
- `feedback_extra_reflection_pass_for_elegant_permanent_shape.md` — applied at
  M-CONS-v2 composition pass
- `feedback_no_defer_HARD_RULE.md` — all surfacing items NAMED-deferred to R0
  + revisit-trigger; surface-to-Ben items NOT "defer," they are RECOMMEND-MINT
- `feedback_review_finding_ground_truth_verify.md` — every cite verified at
  HEAD via local grep (combinators.rs pub fns; cited file paths exist)
- `feedback_handoff_top_banner_re_orient.md` — honored at file top
- `feedback_plain_english_surfaces.md` — surface-to-Ben bundle in §8 follows
  plain-situation + my-pred shape
- `feedback_surface_arch_decisions_under_auth.md` — none of the 14 surfaced
  items are arch-fork-class

---

**End of M-C3 v2 fresh-eyes cryptographer critique document.** Output for
synthesis with M-C1 (elegant-shape) and M-C2 (composability) critique outputs
into a single Ben-decision-ready bundle.
