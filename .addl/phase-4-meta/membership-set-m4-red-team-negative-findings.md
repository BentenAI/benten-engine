# MembershipSet unification — M4 red-team negative-findings

> **Role.** M4 of 5 parallel specialists. Brief = adversarial-design architect; default
> to DISAGREE if substantive grounds. Authored 2026-05-27.
>
> **Inputs status.** M1a / M1b / M1c cataloger outputs + M2 primitive design + M3
> transformation analysis are all **PENDING** at write-time of this document (no
> `.addl/phase-4-meta/membership-set-m{1,2,3}-*.md` artifacts present at HEAD
> `2172cb6d`). M4 proceeds against (a) the four cataloger complications named in the
> brief — "4-identity-concepts tree (CLAUDE.md #18)", "2 ops do NOT fit + 2 with
> caveats", "K(N) path-tagged chain doesn't collapse under MembershipSet" — plus
> (b) docs-grounded substrate I CAN verify at HEAD: per-DID partition seam
> (`WriteContext::namespace_did`); K_principal scoping (per-DID, NOT per-Atrium —
> `K_principal = BLAKE3-keyed-hash(domain_tag, namespace_did.as_bytes())` per
> `docs/SECURITY-POSTURE.md:2556`); device-DID + DeviceAttestation chain
> (`crates/benten-id/INTERNALS.md:148`); plugin-DID model (`docs/GLOSSARY.md:115`);
> Atrium kick / admin-rotate semantics (absent from `docs/SECURITY-POSTURE.md`
> + `docs/ADMIN-UI.md` grep — i.e. NOT YET DESIGNED at v1, see §3.6 below).
>
> When M1/M2/M3 land, M4's findings below should be re-grounded against the actual
> M2 primitive shape; specific shape claims about M2 are marked **M2-PENDING**.

## §1 Executive verdict + confidence

**Verdict: CONCUR-WITH-AMENDMENTS-TO-PRIMITIVE (high confidence the unification is
DIRECTIONALLY correct; medium-high confidence it needs ≥4 substantive amendments
before R1-PLAN).**

I tried hard to break the unification across 8 attack surfaces and found 5 places
where a naive `MembershipSet { kind, members, admins, … }` enum-of-three primitive
**genuinely fails** to compose with substrate that's already-frozen at v1 (the
v1-FROZEN `WriteContext::namespace_did` field; the K_principal-per-namespace_did
keying surface; the Atrium-fork "copy past content exclude future" semantics
ratified 2026-05-27; the device-DID-attestation chain that's load-bearing for Inv-14
attribution; the plugin-DID-as-bounded-issuer trust model under CLAUDE.md #18
D-4F-12/D-4F-16). These aren't deal-breakers — each has a clean amendment path —
but **a naive collapse that elides the existing typed distinctions WILL regress
load-bearing v1-frozen surface** unless amendments §11.{1,2,3,4,5} land.

The strongest single objection (§4.4 below): **DAK and K_Atrium are NOT the same
concept** — DAK is a *device-grain* keying primitive bound to DeviceAttestation
chains for Inv-14 attribution + per-device cap-envelope, whereas K_Atrium is a
*set-grain* content-encryption primitive bound to AEAD-wrap of Node bodies. They
share the word "key" but compose on orthogonal axes (attribution-vs-confidentiality;
device-vs-content; signing-vs-encrypting). Conflating them under a uniform "MemberKey"
slot in `MembershipSet.members` would be a category error.

Secondary verdict: **forkability is genuinely Atrium-only** (§5) — DeviceMesh
and SingleDevice have NO fork semantics, and a uniform `fork()` API on
MembershipSet would be vacuous or wrong for 2 of 3 kinds.

## §2 Threat-model differences (Task 1)

### §2.1 Atrium insider-threat vs DeviceMesh no-insider

**This is the single biggest threat-model gap and it doesn't collapse.**

| axis | Atrium | DeviceMesh | SingleDevice |
|---|---|---|---|
| member-vs-member adversary | YES (multi-user; Bob may be hostile to Alice within the same Atrium) | NO (all devices owned by same user; no in-set adversary) | N/A (no set) |
| insider-data-exfil threat | LIVE (one member can dump shared content to third parties) | non-applicable (all devices are the same trust principal) | non-applicable |
| compromised-member containment | REQUIRED (kick + re-key forward-only) | REQUIRED but DIFFERENT (lost-device revoke; user IS still the trust root) | non-applicable |
| audit posture | multi-tenant audit logs (who-saw-what across users) | single-tenant logs (which-device, but user is unambiguous) | none |

**Why this breaks naive unification.** A uniform `MembershipSet.kick(member)` operation
under MembershipSet semantics says "remove member from set + re-key forward." But the
*semantics* of removal differ:

- **Atrium kick:** "this member is now adversarial; assume they hold every K(N) they
  ever derived; future content MUST be inaccessible to them; past content remains
  decryptable to them (compromise #31 in `docs/SECURITY-POSTURE.md:91` — already
  documented as OPEN at v1-beta + v1-GM); admin DID signs the kick → policy frame
  update → forward keys re-rooted from a fresh K_Atrium." Multi-actor policy
  question: WHICH admin can kick? Quorum? Single-admin? Founder-DID-only? These are
  Atrium-policy semantics with no DeviceMesh analogue.
- **DeviceMesh kick (a.k.a. "revoke lost phone"):** "this *device* of mine is no
  longer mine; revoke its DeviceAttestation; future-derived DAKs invalid for
  attribution; the user-DID (parent) signs the revoke." Single-actor policy: the
  user owns the device-mesh trust root unilaterally. No quorum question; no
  insider-threat axis.
- **SingleDevice kick:** vacuous (no other members).

**M2 cannot collapse the kick operation** unless it admits a typed
`MembershipPolicy` parameter that varies per `kind`. This is a substantive amendment
(see §11.1).

### §2.2 Coercion / xkcd-538-wrench differences

xkcd-538-wrench attack (https://xkcd.com/538/) = adversary with $5 wrench beats key
out of victim. Threat-model differs:

- **Atrium:** the coerced member is ONE member among N; remaining N−1 members can
  detect (out-of-band signal) + kick the coerced member + re-key. Containment is
  POSSIBLE at the set level.
- **DeviceMesh:** the user IS the trust root; if the user is coerced, every device
  in the mesh capitulates. No containment at the set level. Mitigation is per-device
  (duress-PIN unlocks a decoy partition; per-device biometric gating).
  Set-level operations don't help.
- **SingleDevice:** maximally vulnerable — single point of failure.

The relevant *defense* primitive differs per kind. MembershipSet as a uniform shape
does not surface this distinction; an amendment would add a typed
`CoercionPosture` (e.g. `MembershipSet::coercion_posture() -> CoercionPosture`) that
varies per kind, but that's a leaky abstraction — better to keep these as separate
kinds with separate ops surfaces (see §11.1).

### §2.3 Compliance regimes (multi-tenant audit vs single-tenant)

- Atrium subject to GDPR-style data-controller-data-processor splits + multi-tenant
  audit (e.g. SOC-2 controls). The Atrium is the boundary of WHO can see WHAT;
  audit needs WHO-saw-WHAT-WHEN cross-tenant.
- DeviceMesh subject to single-tenant audit (per-device attribution via Inv-14's
  `device_did` slot is sufficient; user is unambiguous; no cross-tenant question).
- SingleDevice subject to no multi-actor audit.

Audit-trail SHAPES differ. If MembershipSet emits a uniform `MembershipEvent` stream,
the consumer downstream needs to know the `kind` to interpret correctly. This is
soluble (tag events with kind) but it's a wart on the "uniform primitive" pitch.

### §2.4 Admin-DID compromise vs DAK compromise — different failure modes

| compromise | scope | recovery |
|---|---|---|
| Admin-DID of an Atrium | catastrophic AT the Atrium level (admin can rotate policy, add/kick members, rotate K_Atrium) | requires out-of-band quorum recovery (multi-admin quorum re-sign? founder-DID escalation? unsolved at v1) |
| DAK of a device | scoped to one device's attributable writes; remaining devices unaffected; user-DID parent revokes DeviceAttestation | revoke device, re-issue attestation to a fresh device-key |
| K_Atrium leaked | FUTURE-only forward compromise (re-key); past content leaks |  re-key Atrium; rotate policy; |
| user-DID of single user | catastrophic across ALL their identity | identity recovery is out-of-scope at v1 (mnemonic / hardware-key recovery — Phase-6+) |

These are *categorically different* failure modes and recovery paths. A
"MembershipSet-level admin rotation" operation that papers over the distinction
between "rotate Atrium admin (multi-actor consensus required)" and
"rotate DeviceMesh root (user does it unilaterally on their other device)" would be
DANGEROUS — it conflates a multi-actor governance op with a single-actor utility op.

**Verdict for §2.** The unification can SURFACE the right shape only if it keeps
`kind`-typed dispatch for kick / admin-rotate / coercion-posture / compromise-recovery
operations. A flat `MembershipSet::kick(member)` is wrong.

## §3 UX implications (Task 2)

### §3.1 SingleDevice user seeing "Membership Set" UX language

**Concrete break.** A user on a single-device install (Phase-1 default; admin UI v0
target shape per `docs/HOW-IT-WORKS.md:142` deployment shape (c)) opens the admin UI
and sees a panel labeled "Your Membership Set." This is *confusing* for two reasons:

1. There is no "set" they participate in — they are alone. The cardinality is
   N=1, and the noun "set" implies plurality.
2. "Membership" implies a thing to BE A MEMBER OF, which implies a group with
   other members. The user has no group.

The right UX language for SingleDevice is **"Your Engine"** or **"Your Account"** —
unary, no plurality implication. For DeviceMesh: **"Your Devices"** — still
user-scoped, plurality but no governance implication. For Atrium:
**"Shared with [Atrium-Name]"** — multi-user, group-scoped, governance implication.

A uniform MembershipSet UX widget would force one of these three framings on all
three cases, and ALL three uniform choices are bad for ≥2 of 3 cases.

**Resolution.** Either:
- (a) UX dispatches on `kind` at the presentation layer (cleanly separated from the
  engine primitive) — this is FINE; primitive can stay unified.
- (b) MembershipSet UX is genuinely uniform — REJECTED, produces bad UX.

The unification is salvageable if (a) is acceptable, but it weakens the "uniform
primitive" pitch — if the UX must dispatch on kind anyway, what does the engine
gain from collapsing the three primitives into one with a kind-tag? Less than it
looks.

### §3.2 Admin ops on DeviceMesh — do they make sense?

The brief asks specifically. Walking each:

| op | Atrium | DeviceMesh | SingleDevice |
|---|---|---|---|
| add_member | invite-flow + new-user accepts | enroll-device (QR scan? device-pairing) | n/a |
| kick member | admin signs revoke | user revokes lost-device's DeviceAttestation | n/a |
| rotate admin policy | multi-actor (founder + quorum?) | unilateral (user has root) | n/a |
| rotate K_Set | governance event + re-key | utility (lost device → re-key DeviceMesh content) | n/a |
| update membership-set metadata | governance | personal preference | n/a |

The OPS exist on both Atrium and DeviceMesh, but the *governance shape* is
fundamentally different — multi-actor consent vs single-actor unilateral. A uniform
admin-ops surface that presents the same UI for both cases would mislead Atrium
admins (no quorum gate visible) OR mislead DeviceMesh users (governance-flow
ceremony for what should be one-click "remove device").

Same resolution as §3.1: UX dispatches on kind, engine primitive can stay unified
IF it admits a typed governance-policy parameter.

### §3.3 Atrium-of-Atriums / Gardens UX (M1b L9 O7)

If `MembershipSet.members` can include other MembershipSets (Gardens), what UI does
the user see? A tree-of-trees? A flat list with grouping? This becomes a
visualization question with no precedent in v1. **Defer this question to Phase 7**
(Gardens are explicitly Phase-7 per `docs/SECURITY-POSTURE.md:82` Compromise #22 +
`docs/HOW-IT-WORKS.md:142`) — but flag NOW that the M2 primitive design MUST NOT
preclude Gardens recursion (i.e. M2's `members: Vec<MemberKey>` slot's
`MemberKey` type must be expressible-as-Set-CID, not strictly DID-CID).

## §4 Key-distribution mechanism differences (Task 3)

### §4.1 HPKE-wrap target differences

**Atrium add-member:** the new member's user-DID has an *encryption pubkey* (X25519
or Kyber-hybrid). The admin HPKE-wraps K_Atrium to the user's encryption pubkey;
the user unwraps on any of their devices (because their user-DID's encryption
private key is replicated across their DeviceMesh per the DeviceAttestation chain).

**DeviceMesh add-member:** the new device's device-DID has a *device-grain*
encryption pubkey. The user HPKE-wraps the DeviceMesh shared secret to the device
pubkey directly. The user-DID isn't the wrap target — the device-DID is.

**Why these differ keying-surface-wise.** Atrium add-member wraps to USER-grain
pubkey (one wrap, N-device-fanout from there). DeviceMesh add-member wraps to
DEVICE-grain pubkey directly. The fanout shape differs (1 wrap → user's mesh
takes care of internal replication for Atrium; 1 wrap per device for DeviceMesh).

**Does M2's `members: Vec<MemberKey>` handle both?** Depends on what `MemberKey`
IS (M2-PENDING). If `MemberKey` = `DidCid` then it COLLAPSES the distinction —
incorrectly. The wrap-target type differs:
- Atrium member-key is a user-DID-bound encryption-pubkey reference.
- DeviceMesh member-key is a device-DID-bound encryption-pubkey reference.

These resolve to DIFFERENT secp/X25519/Kyber pubkeys; they're NOT interchangeable.
The HPKE-wrap recipient set is morphologically different per kind.

**Amendment.** `MemberKey` must be a typed sum: `MemberKey = UserMember(user_did,
enc_pk) | DeviceMember(device_did, parent_user_did, enc_pk) | …`. M2 must NOT
collapse the wrap-target type. See §11.2.

### §4.2 DAK vs K_Atrium — same concept under MembershipSet?

**No — they're orthogonal.** DAK (Device Attestation Key) is per-device,
*signing-grain*, used for Inv-14 attribution (the `device_did` slot stamped on
TraceStep per `crates/benten-id/INTERNALS.md:337` Compromise #23). K_Atrium is
per-Atrium, *encryption-grain*, used for AEAD-wrap of Node bodies under the
encryption-as-confidentiality substrate (`docs/SECURITY-POSTURE.md:2526`).

| axis | DAK | K_Atrium |
|---|---|---|
| grain | per-device | per-set (per-Atrium) |
| crypto role | signing key (attestation) | symmetric encryption key (AEAD wrap) |
| derivation | issued by parent user-DID; signed device-attestation | derived from K_principal per #1301 substrate |
| scope of use | every TraceStep attribution stamp | every Node body ciphertext under that Atrium |
| compromise blast radius | one device's attribution | every Node body under that Atrium |

Conflating them under a uniform "MembershipSet has a key" slot would be a
**category error** — it bundles two crypto-roles that the type system at v1
ALREADY keeps separate for sound reasons (composition with the codepoint-dispatched
crypto-agility seam per #1301 / `docs/SECURITY-POSTURE.md` G-CORE-3 lane).

This is **the single strongest objection** to a naive MembershipSet unification.

### §4.3 K_principal scoping is per-DID, NOT per-Atrium

Verified at `docs/SECURITY-POSTURE.md:2556`:
```
K_principal = BLAKE3-keyed-hash(domain_tag, info = namespace_did.as_bytes())
K(root)     = HKDF-SHA256(K_principal, info = "root" || plaintext_cid)
```

K_principal is keyed on `namespace_did` — a single DID identifier. The current
v1-frozen substrate (`WriteContext::namespace_did: Option<Cid>` per
`docs/ARCHITECTURE.md:503`) is per-DID, not per-Atrium-membership-set. There is NO
per-Atrium K_principal in the v1 freeze — Atriums are MULTI-DID; the K_principal
seam pre-supposes ONE namespace_did.

**This is a load-bearing freeze-level mismatch.** If M2's MembershipSet wants a
`K_Set` derivable from K_principal per the unified shape, it would need EITHER:
- (i) Pick ONE namespace_did to serve as the Atrium's K_principal source (whose?
  founder? rotating? admin-quorum-derived?) — opens governance questions M2
  isn't equipped to answer.
- (ii) Layer a *separate* K_Atrium that is NOT derived from any one user's
  K_principal — but then K_Atrium has its OWN secret-storage seam to design,
  which contradicts the "unify" pitch by adding ANOTHER keying surface.
- (iii) Multi-recipient HPKE-wrap K_Atrium to each member's user-DID-bound
  encryption-pubkey — this is the SANE choice but it's exactly the mechanism that
  already exists conceptually for Atrium add-member; it's NOT a unification with
  DeviceMesh, it's a specialization for the Atrium case.

**Verdict.** The K_principal substrate at v1 is per-DID. K_Atrium at the
MembershipSet level is a SEPARATE keying surface from K_principal — not a
specialization, not a derivation, a parallel construct. Treating them as unified is
substrate-incorrect.

### §4.4 (Bonus) PQ-hybrid signature default

Phase-4-Meta-Core ships PQ-hybrid Ed25519⊕ML-DSA-65 default (per
`docs/HOW-IT-WORKS.md:142`). MembershipSet primitive would need to admit
hybrid-keyed members. If `MemberKey` is sized for classical-only (e.g. 32-byte
Ed25519-only `[u8; 32]`), it CAN'T hold ML-DSA-65 hybrid (which is multi-kilobyte).
M2 must use the codepoint-dispatched crypto-agility seam, not a fixed-size key
field. Likely already handled in M2 design but worth a sanity-pin.

## §5 Forkability semantics differences (Task 4)

### §5.1 Per-kind fork mapping

| kind | fork semantics | sensible? |
|---|---|---|
| Atrium | Ben-ratified 2026-05-27: copy past content, exclude future content | YES — load-bearing for "I split off a sub-group from a big Atrium and our new sub-group sees old shared content but doesn't see what big-Atrium publishes after the fork" |
| DeviceMesh | "fork your devices"? VACUOUS — user doesn't fork their own device-mesh; they add/remove devices | NO |
| SingleDevice | "fork yourself"? VACUOUS — single user, single device | NO |

**Forkability is Atrium-only.** A uniform `MembershipSet::fork()` API would be
vacuous or undefined for 2 of 3 kinds.

### §5.2 What does this mean for the unification?

Options:
- (a) `fork()` is a method on `MembershipSet` that returns `Err(NotForkable)` for
  DeviceMesh and SingleDevice. **REJECTED** — runtime-typed dispatch on a primitive
  shape that's supposed to be uniform is a code-smell.
- (b) `fork()` lives on a separate `ForkableAtrium` trait that only Atrium
  implements. **ACCEPTABLE** — but then Atrium has surface that the unified
  primitive doesn't, which IS a sign the kinds aren't actually unifiable into a
  single primitive shape.
- (c) `fork()` lives ENTIRELY OUTSIDE MembershipSet, as an Atrium-specific
  application-layer composition. **PREFERRED** — matches the
  feedback_engine_primitives_vs_application_layer principle from MEMORY.md ("push
  application-layer composition before engine extension"). Forkability becomes
  an APPLICATION-LAYER op that takes (Atrium-CID, fork-policy) and emits a new
  Atrium-CID; the engine primitive doesn't carry fork semantics at all.

This is a genuine **break-the-unification** finding: the M2 primitive design CANNOT
elegantly carry `fork()` semantics across all three kinds; it must either externalize
or admit kind-typed surfaces.

### §5.3 Companion: merge semantics?

If Atriums can be forked, can they be merged back? (the "we split off but want to
re-merge later" UX.) DeviceMeshes don't merge (each user's mesh is theirs);
SingleDevices don't merge. Merge is Atrium-only too. Same outcome as §5.2.

## §6 Composition with 4-identity-concepts (Task 5)

Per `docs/GLOSSARY.md:19` (CLAUDE.md baked-in #18 / D-4F-12 retense):
- content-CID
- peer-DID (signature on original content)
- plugin-DID (minted at install)
- user-DID

Plus from `docs/GLOSSARY.md:51`: device-DID (per-device attribution carrier).
Plus from `docs/GLOSSARY.md:115`: plugin-DID specifically as UCAN audience AND
constrained issuer within manifest `shares` envelope.

That's actually **5 identity concepts** if we add device-DID (and arguably 6+ if
we count agent-DID under the Phase-6+ Agents lane — currently unshipped).

### §6.1 Can MembershipSet members include plugin-DIDs?

Concrete scenario: a workflow Atrium called "Q1 marketing campaign" includes
3 user-DIDs (the marketers) + 1 plugin-DID for a third-party AI agent (e.g. an
analytics plugin that needs to read campaign data + emit reports).

Is the plugin-DID a *member* of the Atrium?

**Argument YES:** the plugin holds a UCAN bound to that Atrium; it sees a subset
of Atrium content; it can write back via its bounded `shares` envelope. Functionally
it's a member with attenuated caps.

**Argument NO:** the plugin-DID's authority is *bounded by the manifest envelope*
(per `docs/GLOSSARY.md:115` D-4F-16) — it has NO inherent authority, only
delegated. Granting it "membership" implies set-level rights (vote in admin
quorum? receive K_Atrium directly?) that plugin-DIDs structurally shouldn't have.

**Resolution.** The plugin-DID is a *bounded-capability recipient*, NOT a member.
It receives an attenuated UCAN scoped to a subgraph of the Atrium content; it does
NOT join the membership set. The distinction matters: members are governance
principals (can vote, can be in admin set, share in K_Atrium derivation); plugins
are capability-bounded consumers.

If M2's MembershipSet admits plugin-DIDs as first-class members, it BREAKS
the CLAUDE.md #18 plugin-trust model. Plugin-DIDs must be a separate concept —
a `CapabilityRecipient` adjacent to (not within) MembershipSet.

### §6.2 Agent-DIDs as ephemeral members

Phase-6+ Agents (not v1). If/when Agents land, the question recurs: agent-DID =
member or capability-recipient? Same answer — agents are time-bounded, capability-
attenuated principals; not members. (If an agent IS a member, it has all the
governance rights of a user; that's a privilege escalation surface we don't want.)

Defer to Phase-6 — but flag: M2's design must NOT preclude the distinction.

### §6.3 Mixed-DID-type membership

Scenario: "this Atrium has user-DIDs Alice + Bob, plugin-DID for analytics
plugin, and agent-DID for night-shift assistant." Is the MembershipSet
heterogeneous?

**Verdict.** No — per §6.1 + §6.2 the set is HOMOGENEOUS in user-DIDs (and
DeviceMesh is homogeneous in device-DIDs); plugin-DIDs and agent-DIDs are
capability-recipients in an adjacent structure, not set members.

This means MembershipSet IS homogeneous per kind (Atrium = user-DIDs; DeviceMesh
= device-DIDs of one user); the *kind* of `MemberKey` is determined by the
MembershipSet's `kind`. M2's design should pin this: typed-per-kind member element,
not a single uniform `MemberKey` slot.

### §6.4 The "5-identity" tree is load-bearing — keep it

M1a was right to flag the 4-identity-concepts tree as load-bearing complication.
Adding "MembershipSet" as a 5th-concept (a *grouping* of identity-bearers) is fine
ONLY IF MembershipSet does NOT collapse the existing distinctions. The
homogeneous-per-kind constraint above achieves this.

## §7 Path-A.5 + cross-MembershipSet K(N) composition (Task 6)

### §7.1 Current K(N) derivation at v1-FROZEN

Per `docs/V1-FROZEN-INTERFACE.md:1607` + `docs/SECURITY-POSTURE.md:2556-2570`:

```
K_principal = BLAKE3-keyed-hash(domain_tag, info = namespace_did.as_bytes())
K(root)     = HKDF-SHA256(K_principal, info = "root" || plaintext_cid)
K(N)        = HKDF-SHA256(K(predecessor), info = "step" || edge_label || ...)
```

The K(N) derivation chain is rooted at K_principal, which is rooted at
namespace_did (ONE DID, not a set).

### §7.2 The break: K(N) under MembershipSet's K_Set?

If M2 wants K(N) rooted at K_Set instead of K_principal, you'd have:

```
K(root)_atrium = HKDF-SHA256(K_Set_atrium, info = "root" || plaintext_cid)
```

But this BREAKS the v1-FROZEN K(N) shape — K_principal is the named seam at
`docs/V1-FROZEN-INTERFACE.md` item 15(i); rotating it to K_Set would be a
post-v1-beta breaking change requiring V1-BETA-BREAKING-CHANGES.md row.

**M1c's concern is correct.** The K(N) path-tagged chain does NOT collapse under
MembershipSet — it's currently rooted at K_principal per the freeze, and re-rooting
at K_Set would be a v1-incompatible change.

### §7.3 Cross-MembershipSet Node sharing

Scenario: one Node lives in BOTH a user's personal DeviceMesh AND a shared Atrium
(e.g., a calendar event the user puts in their personal cal that also gets shared
to a "family" Atrium).

Under separate-K-per-set:
- The user's DeviceMesh encrypts the Node body under K_Mesh-derived K(N).
- The Atrium encrypts the same plaintext Node body under K_Atrium-derived K(N).
- These produce DIFFERENT ciphertexts of the SAME plaintext.
- Storage cost: 2× (two ciphertexts).
- Dedup blinding (a separate v1 surface) does NOT collapse them — the AEAD nonce +
  derivation context differ.

Is this OK? **YES, semantically** — it's the cost of multi-tenant confidentiality
under separate keying domains. But it's a 2× storage cost the M2 design should
SURFACE as a known cost, not hide.

Alternative: derive K(N) from the content-CID alone (so the same plaintext always
produces the same K(N)) and HPKE-wrap that K(N) to each tenant. This is
content-deduplicable but loses per-tenant K-independence (one tenant's compromised
key reveals plaintext to anyone holding the same ciphertext).

The Path-A.5 design (per `docs/V1-FROZEN-INTERFACE.md:1633`) is two-path:
owner-derivable K(N) from K_principal + recipient-derivable K(N) from edge-walk.
Under MembershipSet, the *owner* axis is the question — who owns K(N)? The owning
member-DID? The set? An admin? This is genuinely unanswered at v1.

### §7.4 The right amendment

K(N) at v1 stays rooted at K_principal (per-DID). MembershipSet introduces a
SEPARATE layer of access-control on TOP of K(N), implemented via HPKE-wrap of K(N)
to each member's encryption-pubkey at the K(root) granularity. K(N) derivation
stays unchanged; what differs per set is the *recipient set* the K(root)s are
distributed to.

This preserves the v1 freeze + cleanly composes with MembershipSet. But it means
MembershipSet is NOT a "unified replacement for Atrium/DeviceMesh/SingleDevice
keying primitives" — it's an ADJACENT primitive that adds distribution semantics
on top of the existing per-DID key-derivation.

This is a substantive demotion of the M2 pitch. See §11.3.

## §8 Perf / wire-size implications (Task 7)

### §8.1 Typed-variant overhead

If `MembershipSet { kind: MembershipSetKind, members: Vec<MemberKey>, … }` adds a
`kind` discriminant byte + typed-per-kind member elements, the wire overhead vs
current per-primitive shapes is bounded (~1 byte for kind tag + typed-member
overhead per element). Acceptable in absolute terms but...

### §8.2 Codepoint dispatch surface

The codepoint-dispatched crypto-agility seam (#1301 per
`docs/HOW-IT-WORKS.md:142`) already carries kind-dispatch overhead. Adding
MembershipSet-kind-dispatch creates a SECOND independent kind-dispatch axis.
Composition: `codepoint × membership_kind = N×3` matrix of (cipher-suite,
membership-kind) tuples. Each combination needs test coverage.

This is a substantive composition cost — currently the codepoint matrix is
1-dimensional. Adding MembershipSet-kind makes it 2-dimensional.

### §8.3 #[non_exhaustive] + escape codepoint compatibility

If MembershipSet's `kind` enum is `#[non_exhaustive]` (per the L8 pattern), every
match site (engine-side + napi-side + TS-side per the cross-language rule-mirror
§3.5g) needs an unreachable arm. That's a maintenance burden but mechanical.

The escape codepoint (per #1301 substrate) lets future kinds (e.g. Garden in
Phase 7) compose without breaking v1 wire format. **OK** — non-exhaustive +
escape is the standard pattern; MembershipSet should follow it.

### §8.4 CodepointLifecycle composition

CodepointLifecycle (per crypto-agility plan) defines `Stable / Experimental /
Deprecated / Retired` lifecycle stages. MembershipSet kinds would have their OWN
lifecycle (e.g. is "Garden" Stable or Experimental at Phase 7?). Two parallel
lifecycle ladders. Composition: cipher-suite Lifecycle × MembershipSet-kind
Lifecycle. Mechanical but doubles the lifecycle bookkeeping.

**Verdict for §8.** No deal-breaker; the perf + wire-size overhead is small. The
composition-axis multiplication (codepoint × membership-kind) IS a real cost but
acceptable IF M2 ships the kind-dispatch test matrix as part of the wave.

## §9 Atrium-of-Atriums recursion (Task 8)

### §9.1 Recursive depth

If MembershipSet members can include other MembershipSet-CIDs (Gardens per L9 O7
+ Phase 7), what's the recursion depth limit?

- **Cycle-detection** is REQUIRED — a self-referential MembershipSet (A contains
  B contains A) breaks fork + admin-rotation + dedup-blinding (infinite walk).
  Use CID-set during walk to break cycles. Standard graph-walk discipline.
- **Depth limit** for performance: unbounded recursion would let an adversary
  construct a deep tree that DoSes the K-derivation walker. Need a cap (suggest
  16 — matches BLAKE3 tree-depth + MST depth conventions).
- **Practical depth**: most Gardens scenarios are 1-2 levels (Garden = set of
  Atriums; Grove = set of Gardens). Cap at 3-4 for v1, raise if needed.

### §9.2 Forkability composes recursively?

Forking a Garden = forking each contained Atrium individually? Forking the
top-level Garden membership only? Both? This is a UX + governance design question
that's NOT answered at v1 — Phase 7 design surface.

### §9.3 Admin-rotation composes recursively?

If Garden-admin rotates, does that propagate to contained Atrium admins? NO —
each Atrium retains its own admin set independently. Garden-admin only governs
the GARDEN-LEVEL membership (which Atriums are in the Garden), not the Atriums'
internal membership. This is a Liskov-substitution constraint on the recursion:
the recursive composition is NOT homomorphic for admin-rotation.

### §9.4 Dedup-blinding composes recursively?

If Garden-A and Garden-B both contain Atrium-X, do they SHARE the K_X distribution?
YES — K_X is Atrium-X's key, not Garden-A's; Gardens are just groupings.

### §9.5 Verdict for §9

Recursion is workable but has at least 4 distinct composition rules that are NOT
homomorphic — admin-rotation, fork, dedup, K-derivation each compose differently
under recursion. M2's primitive design MUST NOT pretend recursion is transparent;
each op needs an explicit "what does this mean at depth N?" decision. **Phase 7
design surface; v1 must only ensure the M2 shape doesn't preclude it.**

## §10 Specific breakable cases (Task 9) — 5 things M2's primitive design definitely can't elegantly handle

### §10.1 BREAK 1 — Forkability is Atrium-only (per §5)

Naive `MembershipSet::fork()` is vacuous for DeviceMesh + SingleDevice. M2 cannot
expose `fork()` uniformly without runtime kind-check or trait-segregation. The
clean answer is to externalize forkability as an application-layer op on top of
the primitive, NOT bake it into MembershipSet. This contradicts the "unify
fork-semantics" pitch if M2 made that pitch.

### §10.2 BREAK 2 — DAK ≠ K_Atrium (per §4.2)

DAK is per-device signing key; K_Atrium is per-set encryption key. Conflating
under a uniform "set has a key" slot is a category error. M2 must keep these
typed-separately. If M2's design has a single `MembershipSet.key` field, it's
WRONG.

### §10.3 BREAK 3 — K_principal substrate is per-DID, not per-Atrium (per §4.3 + §7.1)

v1-FROZEN K(N) derivation is rooted at K_principal which is per-`namespace_did`.
MembershipSet's notion of "K_Set" can't replace K_principal without breaking the
v1 freeze. The right composition is HPKE-wrap of K(N) to members on top of the
existing per-DID derivation — not re-rooting K(N) at K_Set.

### §10.4 BREAK 4 — Kick semantics differ structurally per kind (per §2.1)

Atrium kick = multi-actor governance event (admin signs + re-key forward + member
cannot decrypt future content). DeviceMesh kick = single-actor utility (user
revokes DeviceAttestation). SingleDevice kick = vacuous. M2 cannot expose
`MembershipSet::kick(member)` uniformly. The cleanest answer is per-kind ops
surfaces (`AtriumOps::kick` + `DeviceMeshOps::revoke_device`), with MembershipSet
providing only READ-side uniformity (e.g. `MembershipSet::is_member(did)`).

### §10.5 BREAK 5 — Plugin-DIDs and agent-DIDs are NOT members (per §6.1 + §6.2)

Per CLAUDE.md #18 D-4F-12/D-4F-16, plugin-DIDs are bounded-capability recipients,
NOT members. If M2 admits plugin-DIDs as first-class members in
`MembershipSet.members`, it breaks the plugin trust model. The set MUST be
homogeneous per kind (user-DIDs only for Atrium; device-DIDs only for DeviceMesh).
Plugins and agents live in an ADJACENT structure.

### §10.6 BREAK 6 (bonus) — Audit-event shape differs per kind (per §2.3)

If MembershipSet emits a uniform `MembershipEvent` stream, downstream auditors
need kind-tag to interpret correctly. Multi-tenant audit logs (Atrium) and
single-tenant device-attribution logs (DeviceMesh) are categorically different;
a uniform event stream is leaky.

## §11 Recommended amendments to M2's primitive design

### §11.1 Amendment — typed governance policy per kind

`MembershipSet { kind, members, governance: GovernancePolicy, … }` where
`GovernancePolicy` is a typed sum:
- `Multiactor { admin_did_set, quorum: u8 }` — Atrium
- `SingleActor { root_user_did }` — DeviceMesh
- `Unary` — SingleDevice

Ops like `kick`, `rotate_admin`, `add_member` dispatch on `governance` (NOT on
`kind` directly), keeping the surface uniform-looking but correctly typed.

### §11.2 Amendment — typed `MemberKey` sum (per §4.1, §6.3)

`enum MemberKey { UserMember { user_did, enc_pk }, DeviceMember { device_did,
parent_user_did, enc_pk } }`. `MembershipSet.members: Vec<MemberKey>` stays
uniform-shaped but typed-elements differ. Plugin-DIDs + agent-DIDs are NOT
admitted into MemberKey — they live in an adjacent `CapabilityRecipient`
structure.

### §11.3 Amendment — K_Set is HPKE-wrap layer ON TOP OF K_principal, not a replacement (per §4.3, §7.4)

MembershipSet K_Set is the recipient-set keying layer: K(root)s are
HPKE-wrap-distributed to member encryption-pubkeys. K(N) derivation itself stays
rooted at K_principal per the v1 freeze. M2 should pin this composition explicitly
("MembershipSet does NOT replace K_principal; it adds HPKE-distribution on top").

### §11.4 Amendment — forkability lives at application layer, not on MembershipSet (per §5.2, §10.1)

`fork()` is an Atrium-specific application-layer op (`AtriumOps::fork(atrium_cid,
fork_policy) -> AtriumCid`); MembershipSet does NOT carry fork semantics. This
matches the engine_primitives_vs_application_layer feedback from MEMORY.md.

### §11.5 Amendment — homogeneous-per-kind member type (per §6.3, §10.5)

MembershipSet.kind determines MemberKey variant; the set is HOMOGENEOUS per kind.
No mixed user-DID + device-DID + plugin-DID sets. Plugins + agents are
capability-recipients in an adjacent structure.

### §11.6 Amendment — recursion design deferred to Phase 7, but shape doesn't preclude it (per §9.5)

M2's `MemberKey` should be expressible as `Set(SetCid)` in a future post-v1
extension (for Gardens at Phase 7), via the escape-codepoint pattern. v1 ships
with `Set(_)` NOT enabled; the wire format reserves space for future enablement.

## §12 Self-assessment + confidence per finding

| § | finding | confidence | basis |
|---|---|---|---|
| §2.1 | Insider-threat differs structurally | HIGH | multi-actor vs single-actor governance is canonical security distinction |
| §2.2 | Coercion threat differs | MEDIUM-HIGH | the set-level vs root-level containment distinction is sharp |
| §2.4 | Admin-DID vs DAK compromise are different failure modes | HIGH | crypto-roles + recovery paths are categorically distinct |
| §3.1 | SingleDevice UX language ambiguity | MEDIUM | partly UX preference; primitive can stay uniform if UX dispatches |
| §4.1 | HPKE wrap-target type differs | HIGH | wrap-target is user-DID-bound vs device-DID-bound; not interchangeable |
| §4.2 | DAK ≠ K_Atrium | HIGH | signing vs encryption; per-device vs per-set; categorically distinct |
| §4.3 | K_principal is per-DID per v1 freeze | HIGH | grounded in `docs/SECURITY-POSTURE.md:2556` |
| §5 | Forkability is Atrium-only | HIGH | vacuous for DeviceMesh + SingleDevice |
| §6.1 | Plugin-DIDs are NOT members | HIGH | grounded in CLAUDE.md #18 D-4F-12/16 |
| §6.4 | 5-identity tree is load-bearing | HIGH | direct from M1a complication |
| §7.2 | K(N) doesn't collapse under MembershipSet | HIGH | direct from M1c complication + v1 freeze |
| §7.4 | K_Set is HPKE-wrap layer on top of K_principal | MEDIUM-HIGH | shape claim about M2; M2-PENDING |
| §8 | Perf/wire overhead acceptable | MEDIUM | speculative without M2 wire format |
| §9 | Recursion needs 4 distinct composition rules | MEDIUM | Phase 7 design surface; v1 only needs to not preclude |
| §10.1-5 | 5 specific breaks named | HIGH | each grounded above |
| §11.1-6 | Amendments named | MEDIUM-HIGH | M2-PENDING; needs to be reconciled with actual M2 shape when landed |

### Self-skepticism — what could I be wrong about?

1. **M2 may already have these amendments.** I'm working without M2's actual
   document — my "BREAK"s may be straw-manning a primitive that already typed
   governance + MemberKey + externalized fork. If M2 ships exactly the
   amendments I'm recommending, my verdict softens from CONCUR-WITH-AMENDMENTS to
   CONCUR-WITH-EVIDENCE. **REVISIT WHEN M2 LANDS.**
2. **The "unification" pitch may be weaker than I'm imagining.** If M2's pitch is
   only "let the read-side `is_member` op be uniform" (not "unify all keying +
   governance + fork ops"), most of my objections evaporate. Same revisit.
3. **§4.3 / §7.1 may be redesigned post-v1.** If post-v1-beta the K_principal
   substrate is allowed to migrate to per-Atrium K_Set, the freeze-incompat
   objection weakens. Currently I'm grounded in the v1 freeze as written.
4. **§5 fork-only-for-Atrium** may be too strong if DeviceMesh "fork" is
   reinterpreted as "split device-set into two device-meshes" (e.g., personal vs
   work). I don't see evidence this is on the roadmap but it's conceivable.
5. **§6.1 plugin-DID-NOT-member** may have edge-case exceptions (a plugin so
   trusted it gets governance rights? — but that would require the user-DID's
   explicit consent each time, defeating the bounded-issuer model).

I'd rate overall confidence in **the directional verdict** (unification works
with amendments) at **75-80%**. Confidence in **each individual amendment** at
60-85% depending on how much depends on M2's actual landed shape.

## §13 Citations

- `docs/ARCHITECTURE.md:503` — `WriteContext::namespace_did` per-DID storage
  partition seam frozen at G-CORE-9 for v1
- `docs/SECURITY-POSTURE.md:13-30` — substrate-only posture in v1-beta;
  CLAUDE.md #18 Layer-1/2/3 plugin trust model
- `docs/SECURITY-POSTURE.md:82-95` — Compromise table including #22 (public-relay
  metadata leak; Garden-relays Phase 7 closure path) and #31 (revocation reach in
  encryption-at-rest)
- `docs/SECURITY-POSTURE.md:2526-2580` — K_principal derivation chain
  (`K_principal = BLAKE3-keyed-hash(domain_tag, namespace_did.as_bytes())`;
  `K(root) = HKDF-SHA256(K_principal, "root" || plaintext_cid)`); per-DID
  scoping; the K_principal-per-DID seam
- `docs/V1-FROZEN-INTERFACE.md:1607` — `K(N) = HKDF-SHA256(K(predecessor),
  info = "step" || edge_label || …)` (Path-A.5 / two-path derivation)
- `docs/V1-FROZEN-INTERFACE.md:1633` — two-path symmetry (owner-derivable vs
  recipient-walks-edge-labels)
- `docs/V1-FROZEN-INTERFACE.md:1743` — Compromise #31 cross-reference (already-
  derived K(N) remain decryptable post-revocation)
- `docs/GLOSSARY.md:19` — 4-identity-concepts post-D-4F-12 retense (content-CID
  + peer-DID signature + plugin-DID + user-DID); plugin trust three layers
- `docs/GLOSSARY.md:51` — device-DID per Phase-3 device-heterogeneity contract
  (one per device; parent-issued DeviceAttestation; Inv-14 `device_did` slot)
- `docs/GLOSSARY.md:75` — Install record (user-DID-signed; LOCAL anchor;
  plugin's original content carries peer-DID signatures from sharers/authors)
- `docs/GLOSSARY.md:115` — Plugin DID = UCAN audience AND constrained issuer
  within manifest `shares` envelope (D-4F-12 + D-4F-16; NOT just audience handle;
  NO device-DID-style attestation chain back to user-DID)
- `docs/HOW-IT-WORKS.md:142` — Phase 4-Meta-Core scope (PQ-hybrid signature
  default; codepoint-dispatched crypto-agility; encryption-as-confidentiality;
  storage-partition seam; 13th + 14th workspace crates `benten-crypto-suite` +
  `benten-drop`); 3 deployment shapes (full peer / thin compute / embedded
  webview)
- `crates/benten-id/INTERNALS.md:148-150` — CapabilityEnvelope 4-dimension
  declaration; 5 issuance constructors; thin-client minimum-envelope preset
- `crates/benten-id/INTERNALS.md:337` — device-DID cross-machine sync at
  Phase-3 close; Compromise #23 + DeviceAttestationEnvelope V2 G16-D wave-6b
  PR #163
- `crates/benten-sync/INTERNALS.md:13-25` — `benten-sync` 10th crate;
  Atrium-as-peer-mesh; member-mesh networking posture; CLAUDE.md #9 + #17
  invocations
- `docs/future/phase-4-backlog.md:171-173` — G-CORE-3d/e graph-AEAD layer +
  K_principal-per-DID secret-material backend carry-over
- `docs/future/phase-4-backlog.md:185` — encryption-by-default two-path key
  derivation; `K(N) = KDF(K_principal, N.cid)` owner-derivable + `K(N) =
  KDF(K(predecessor), edge_label)` path-walker
- `docs/SECURITY-POSTURE.md:1709-1738` — Compromise #22 narrative (public-relay
  metadata leak; Garden-relay Phase 7 closure path; Phase 9 hardened-deployment
  fallback)
- xkcd-538 — https://xkcd.com/538/ (rubber-hose cryptanalysis reference for §2.2)
- MEMORY.md `feedback_engine_primitives_vs_application_layer.md` (push
  application-layer composition before engine extension — referenced for §5.2
  and §11.4)

---

**End of M4 red-team negative-findings document.** Output to be reconciled
against M1a/M1b/M1c + M2 + M3 when those documents land.
