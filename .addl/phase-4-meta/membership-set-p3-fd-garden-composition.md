# MembershipSet P3 — F-D Garden / Atrium-of-Atriums composition design

> **Banner for compact-survivors:** This is a *codepoint-reserve* design doc for the
> Garden composition shape (M-C2-v2 Scenario F-D / R-MCV2-8). Garden as a *feature*
> ships post-v1-beta per L9 O7 fractal architecture + N2 row 20 (`create_subset`) +
> N2 §1.2 footnote. This doc pins the **wire-format-additive sub-slot** at v1-beta-
> freeze. Read **with** M-CONS-v2 (`e62ff540`) for Compromise # registry through #56,
> M-C2-v2 (`0533d0cf`) for R-MCV2-1..9 + B-1..4 cross-amendment refinements,
> L9 O7 (`1670aa03` §6.5.4) for the fractal-architecture observation, and N1
> (`ed592770`) RestrictedScopeSet §3.6 orthogonality claim (now refined by R-MCV2-2).
> HARD RULE 12 §15.c: EXACTLY-3-arm enum (Atrium/DeviceMesh/SingleDevice) is
> v1-beta-frozen; "Garden 4th arm" = HALT-AND-SURFACE-TO-BEN. This doc does NOT
> propose to extend the 3-arm enum; it proposes a **codepoint-reserve sub-slot
> under existing Atrium variant**, additive at the policy.metadata layer.

---

## §1 Framing + ground-state restatement

### §1.1 What F-D actually asks

M-C2-v2 §4 Scenario F-D: M-CONS-v2 reserves codepoint for Garden / Atrium-of-Atriums
(per L9 O7 fractal architecture). Sub-Atrium-A has K_Set-A (its own MembershipSet's
random-CSPRNG K_Atrium); Sub-Atrium-B has K_Set-B. Garden-level content (content
posted at the Garden tier, addressed to "any Garden member regardless of sub-Atrium")
encrypts under what?

Four named candidates (brief lists them as Options 1–4):

| # | Mechanism | One-line tradeoff |
|---|---|---|
| 1 | Independent K_Set-G | New K_Set, distribution at Garden-membership-add |
| 2 | Composed encrypt-to-A + encrypt-to-B | Wire doubles; unlinkability breaks at plaintext_cid |
| 3 | RestrictedScopeSet-attenuated grants | Composes with N1; not central-encrypted; different shape than F28 |
| 4 | Recursive MembershipSet (MembershipSet of MembershipSets) | Unlinkability preserved; needs recursion bound |

### §1.2 What this doc decides vs defers

| Decided at v1-beta-freeze (this doc) | Deferred to Garden-impl (Phase-5-7+) |
|---|---|
| Garden composition **shape** (which of 4 options) | Garden impl code (membership add, gossip topic, recovery) |
| Wire-format **codepoint-reserve sub-slot** for Garden | Per-Kind Garden-specialization invariants |
| Recursion **depth limit** value + enforcement site | UX for Garden member browsing |
| Cross-sub-Atrium membership **semantic** (subset vs independent) | Garden-level RBAC + audit-log RBAC compose |
| **Compromise # mint** for "Garden-shape-locked-impl-deferred" honest-disclosure | Concrete protocol for K_Set-G distribution at Garden-membership-add |

### §1.3 Why this is a real-and-now decision, not a deferral

The v1-beta wire-format-additivity-requirement (M-CONS-v2 §3) means: every
codepoint-reserve sub-slot we don't lock at v1-beta forces a wire-format-breaking
change later. The 4 Garden options have **non-isomorphic wire encodings**:

- Option 1 needs `policy.metadata.garden_k_set_anchor: Option<MembershipSetId>` slot.
- Option 2 needs the MultiRecipientSealing variant to allow `Vec<HpkeMultiBase>`
  (a nested vector — a Vec of envelopes, each itself with Vec of stanzas), AND
  a content-side per-MembershipSet duplicate-emission protocol.
- Option 3 needs no new wire (N1 RestrictedScopeSet already shipped).
- Option 4 needs `policy.metadata.member_kind_tag: MemberOrSubsetTag` per MemberKey
  (a 1-byte discriminator), AND a recursion-depth-limit invariant in Inv-20.

Picking now = additive sub-slot ships at v1-beta-freeze. Punting now = either a
wire-breaking change later (Option 2 or 4) or a forced choice of Options 1/3
(losing the architectural option-space).

### §1.4 Ground-state restatement

- Inv-20 clauses (a)..(j) per M-CONS-v2 §3.4 hold.
- EXACTLY-3-arm `MembershipSetKind` enum per N2 §1.2 + HARD RULE 12 §15.c.
- `AtriumWithRotatingGroupKey` is a future-codepoint-reserve, NOT a current arm.
- `RestrictedScopeSet` (N1) ships at v1-beta as the per-recipient sub-graph
  attenuation surface, composes with MembershipSet under (Inv-20 clause-h refined
  by R-MCV2-2 K_Set-acquisition-path triple).
- N2 row 20 (`create_subset`) is post-v1-beta with codepoint-reserve nesting bit.
- L9 O7: Atrium-as-Subgraph fractal; Drop bundle for SubgraphSpec_X may contain
  references to SubgraphSpec_Y (nested Subgraph); UCAN-chain transitivity is the
  downstream concern flagged at L9.

---

## §2 Task 1 — Cross-system precedent survey

| System | Hierarchy mechanism | Crypto shape at parent (group-of-groups) | Membership relation | Verdict for Benten |
|---|---|---|---|---|
| **Keybase teams (sub-teams)** | Parent team admin writes `team.new_subteam` link; subteam writes `team.subteam_head` link in parent's namespace. | **Independent per-team symmetric key** (XOR of user-share + server-share); sub-team has its own key chain; parent admins are "implicit admins" who can *add themselves* but DON'T auto-decrypt subteam content. | Independent: "you can invite someone into a subteam even if they're not in the parent team." Implicit admins hold *control* (add/remove) but not *content keys* until they explicitly join. | **Strong precedent for Option 4 + Option 1 hybrid.** Sub-Atrium membership is independent of Garden membership; control (add/remove) inheritance ≠ content-key inheritance. |
| **Matrix spaces** | Spaces ARE rooms with `m.space.child` and `m.space.parent` state events; bidirectional canonical-parent field; cycle-detection mandatory. | **No inherited encryption**; spaces are typically public (Matrix spec explicitly says encrypted spaces have no advantage because history is unsearchable). Each child room has its own Megolm session per device. | **Independent**; suggested via `"suggested": true` field; multiple canonical parents tie-broken by lex-smallest room ID. | **Precedent for cycle-detection invariant** (must spot+break cycles) + "no inherited key at group-of-groups layer" pattern (Matrix essentially chose to NOT solve the encrypted-space problem). |
| **Discord servers/categories/channels** | Categories contain channels; permissions inherited from category → channel unless overridden at channel level; server > category > channel resolution order with channel taking precedence on conflict. | **No E2EE**; plaintext on Discord servers; permission inheritance is **authorization-only**, not crypto. | Hierarchical with override (Discord's "grey slash" sentinel = "look up the inheritance ladder"). | **Precedent for Option 3** (UCAN-attenuated grant ladder; auth-not-crypto layer). |
| **Slack Enterprise Grid** | Multi-workspace channels exist in N workspaces simultaneously; Org Owners/Admins set permissions; data encrypted at rest + in transit; "enterprise key management" support. | **Single channel-level key per shared channel**, with Org-level key-management overlay. A shared channel is logically one channel that appears in N workspaces — closest to **Option 2's "duplicate-presence in each sub-Atrium"** model but Slack has central server, so no double-envelope. | Independent workspace memberships; shared-channel members can span workspaces. | **Anti-precedent for Option 2** in P2P: Slack's "channel in N workspaces" works because of central server dedup; in P2P, duplicate envelopes wreck unlinkability and wire-cost. |
| **Microsoft Teams (channels + private channels)** | Team contains channels; private channels have **independent SharePoint sites + independent keys** from parent team; encryption key hierarchy: customer key → tenant key → site key → file key. | **Independent per-private-channel keys**; key hierarchy is server-side wrapping, not group-of-groups crypto-composition. | Private-channel members ⊂ parent team members (constrained). | **Precedent for Option 1** (independent K_Set-G with cardinality constraint: parent-set must be a superset of sub-set if MS Teams model adopted; or independent like Keybase otherwise). |
| **iroh-willow (Workspaces)** | Workspaces structure data via meaningful hierarchical Paths; 4 stores (Entry/Secret/Capability/Payload). Read+Write capabilities define area-of-interest as `(subspace, path-prefix, time-range)`; capabilities delegate. | **Capability-attenuation only**; Willow doesn't define payload-at-rest encryption (per L9 6.5.3); workspace-level access composes with sub-area capabilities transparently via delegation. | Capabilities partially-order; sub-area capability ⊆ parent capability. | **Precedent for Option 3** (Willow's capability attenuation = N1's RestrictedScopeSet UCAN attenuation; same architectural pattern). |
| **ATProto labelers + custom feeds** | Labelers/feeds are independent services identified by DIDs; users subscribe; clients choose which to honor; hierarchical config (static / instance / request) at AppView/PDS level. | **Public services; no E2EE**; "user subscribing to a labeler is not public" but PDS+AppView know. | Independent (user-chosen overlay). | **Anti-precedent**: shows that for federated organizations, the "membership-of-membership" framing collapses to per-user subscription; doesn't generalize to encrypted-shared-content. |
| **Jazz CoValue groups** | Every CoValue has Group/Account owner; nested data inherits permissions; `extend` role mapping resolves parent roles via group chains. | **Recursive group-key derivation via extends**; the `extend` mapping turns a child group's role-lookup into a walk up the parent chain; group encryption derives accordingly. | Recursive: child group can extend parent group → child members effectively gain parent roles. | **STRONGEST precedent for Option 4** (recursive MembershipSet); Jazz is local-first, P2P, capability-flavored — closest peer to Benten's threat model. |
| **MLS (RFC 9420)** | Binary tree per group; logarithmic key updates; single group key per group. **No native inter-group composition** in RFC 9420. | Single group's tree-key; intergroup messaging requires application-layer composition (e.g., MIMI). | Single group. | **Anti-precedent for in-MLS group-of-groups**; confirms the IETF answer is "compose at application layer, not in the crypto primitive" — strong endorsement of Garden = composition at MembershipSet layer (Option 4), not at HPKE layer (Option 2). |
| **Federated organizations (LDAP-style nested groups)** | LDAP `member`/`memberOf` recursion; AD nested-group expansion; well-known DoS hazard at scale; JumpCloud cites "downsides and risks" of nested groups. | N/A (authentication layer, not encryption). | Recursive membership-resolution with depth limits + cycle detection. | **Precedent for depth limit + cycle detection** discipline. |

**Cross-system synthesis (5 patterns):**

1. **Independent crypto, inherited control** (Keybase, MS Teams private channels): the
   subteam/private-channel has its own key, but parent admins have *governance*
   rights (add themselves; rotate keys). Membership semantically independent.
2. **No crypto at parent tier** (Matrix spaces): spaces don't encrypt; they're
   navigation/discovery. Crypto lives only at the leaf room/channel.
3. **Auth-not-crypto inheritance** (Discord, Willow, Jazz partial): hierarchy is
   permission-attenuation; no key derivation across tiers.
4. **Central-server dedup duplicates** (Slack Enterprise Grid shared channels):
   one logical channel in N workspaces is fine *with a central server*; doesn't
   port to P2P.
5. **Recursive group-extension via role-walk** (Jazz extends; MLS-application-
   layer; LDAP nested): groups can contain group-references; resolution walks
   the chain. The strongest pattern for E2EE-P2P group-of-groups.

**Key takeaway:** No mainstream system implements Option 2 (composed encrypt-to-both)
because it's **the worst architectural choice** at every system that's actually
built. The patterns split into Option 1 (independent at each tier; Keybase / MS Teams),
Option 3 (capability-attenuation only; Discord / Willow), or Option 4 (recursive
membership-of-membership; Jazz / LDAP / MLS-app-layer / Keybase combined with #1).

---

## §3 Task 2 — Evaluate 4 options + propose a 5th if cleaner

### §3.1 Evaluation matrix

For each option: cryptographic soundness · composability with N1/N2/N3 · recursion
support · per-recipient unlinkability · v1-beta wire impact · Garden-impl post-v1-beta
wave-days.

| Dimension | Opt 1: Indep K_Set-G | Opt 2: Composed envelope | Opt 3: RestrictedScopeSet attenuation | Opt 4: Recursive MembershipSet |
|---|---|---|---|---|
| **Crypto soundness** | HIGH — clean primary K_Set under existing MembershipSet construction; no new primitive. | MED — sound per-stanza but cross-envelope sealing creates a 3-way AAD seam (same plaintext_cid shared across N envelopes) | HIGH — UCAN attenuation is well-trodden | HIGH — recursion is data-shape only; crypto = same Atrium K_Atrium per layer |
| **Composability with N1 (RestrictedScopeSet)** | Orthogonal | DOUBLY-orthogonal — RestrictedScopeSet per-envelope per-Sub-Atrium | **Identity** — Option 3 IS RestrictedScopeSet | Orthogonal AND recursion-additive |
| **Composability with N2 (24-op generic)** | Sibling-trait ops apply at the Garden K_Set; no special case | Op surface needs branch per envelope variant (BAD) | Garden ops = UCAN-grant ops at app layer; not membership-ops at all | Sibling-trait ops apply transparently; depth parameterizes |
| **Composability with N3 (rotation triggers)** | K_Set-G rotates per Inv-20 clause-e independently; sub-Atrium rotation does NOT cascade | Each envelope rotates independently; semantic of partial-rotation murky | N/A (no central key) | Per-layer rotation; child rotation does NOT cascade up (parent re-walk picks new child member-set on next add) |
| **Per-recipient unlinkability (Inv-20 d)** | PRESERVED (per-stanza HPKE under K_Set-G) | **BROKEN** — same plaintext_cid_local across N envelopes lets network observer correlate Sub-A members with Sub-B members | PRESERVED (no central content) | PRESERVED (same as Option 1 but per layer) |
| **Recursion-depth support** | NO (Garden = one parent over N children) | NO | YES (UCAN chains delegate transitively; depth bounded by chain-length) | **YES NATIVELY** — depth bounded by recursion-limit invariant |
| **Wire cost at Garden tier** | 1 envelope per Garden post | **N envelopes** (one per sub-Atrium) | 0 (Garden tier = no Garden-encrypted content; all content is per-Sub-Atrium) | 1 envelope per Garden post (same as Opt 1) |
| **v1-beta wire-format impact** | +1 optional `garden_k_set_anchor` slot in policy.metadata; codepoint-reserve sub-slot for `MembershipSetKind::AtriumGardenAnchor` (additive) | **BREAKING** — `MultiRecipientSealing` variant needs `Vec<HpkeMultiBase>` (nested vec) which doesn't fit current Inv-20-a wire shape additively | NONE (RestrictedScopeSet already shipped at v1-beta) | +1 byte `member_kind_tag: MemberOrSubsetTag` per MemberKey; codepoint-reserve sub-slot `MembershipSetKind::AtriumGardenRecursive` (additive) |
| **Recursion-bound enforcement site** | N/A | N/A | UCAN-chain-depth (already bounded by §3.5g UCAN chain validation; today: 16) | NEW invariant Inv-20 clause-k (`MEMBERSHIP_RECURSION_MAX_DEPTH = 4`); enforced at sibling-trait `Members::add_subset_member` API boundary |
| **Garden-impl wave-days post-v1-beta** | ~6-8 days (new sub-arm + K_Set-G distribution + membership-add protocol + tests) | ~12-15 days (wire-format hazard remediation + duplicate-content-emission protocol + cross-envelope unlinkability mitigation — likely UNSOLVABLE without rotating to Opt 1 anyway) | ~2-3 days (purely doc + grant-template; uses existing RestrictedScopeSet machinery; Garden is "just" a UCAN-grant-prefix) | ~7-9 days (recursion-aware sibling-trait + depth-check pre-flight + recursive K_Set acquisition protocol via Layer-C Drop bundles per L9 O7) |
| **Use-case coverage breadth** | Family/Company Gardens (linear-hierarchy) | Awkward for all cases | Open-source / Game-League (RBAC-flavored) | All 4 named use-cases (see §6) |
| **Failure mode if deferred (codepoint-reserve only)** | Garden simply non-existent until Phase 5+ | Same | Same; RestrictedScopeSet works at v1-beta for "permission-flavored Garden" cases | Same; recursion sub-slot reserved |

### §3.2 Per-option deep-dive

#### Option 1 — Independent K_Set-G

**Shape.** Garden has its own `MembershipSet` (Kind=Atrium per existing 3-arm enum)
whose `members` are *Users*, not sub-Atriums. K_Set-G is independent CSPRNG K_Atrium.
Membership composition rule "Garden member ⊆ ∪(Sub-Atrium memberships)" enforced
at API boundary (constructor validator). Sub-Atrium-A's K_Set-A and Sub-Atrium-B's
K_Set-B are completely separate.

**Pro.** Simple. Same MembershipSet primitive across all tiers. No new wire shape.

**Con.** Garden membership ⊄ sub-Atrium membership *requires runtime check* — there
is no cryptographic enforcement (an admin could add a non-sub-Atrium-member to
the Garden). Doesn't express "this person joins the Garden via being in any sub-team"
intuitively; admin must do double-bookkeeping.

**Verdict.** Workable but uninspired. **Effectively a pure-flat MembershipSet with
a name "Garden"** — the architecture doesn't recognize the hierarchy.

#### Option 2 — Composed encrypt-to-A + encrypt-to-B

**Shape.** Garden-level content carries `Vec<HpkeMultiBase>`, one per sub-Atrium.
No Garden-level K_Set.

**Pro.** No new K_Set distribution at all. Existing per-Sub-Atrium key material
sufficient.

**Con.** **Per-recipient unlinkability BROKEN** at the network layer — same
`plaintext_cid_local` references both envelopes. Even with Q3 DUAL-CID, the
content-side fan-out across two MembershipSets creates an observable correlation
("the post that exists in Sub-A also exists in Sub-B" leaks the Garden's existence
+ approximate sub-Atrium pairing). **Wire cost N× linear** in number of sub-Atriums
(Forest-of-Sub-Atriums = N×wire). **No dedup at storage layer** (each Sub-Atrium
holds its own copy). **Worst on every dimension.**

**Verdict.** REJECTED. Confirmed by zero mainstream system adopting this pattern.

#### Option 3 — RestrictedScopeSet-attenuated grants

**Shape.** No Garden-level encrypted content at all. "Garden" is purely a UCAN-grant
abstraction: a top-level UCAN-chain root that delegates to per-Sub-Atrium UCANs.
Each "Garden member" gets a UCAN-grant that attenuates to per-Sub-Atrium grants
via `RestrictedScopeSet`. Garden-level content posted = posted to all sub-Atriums
the poster has grants to.

**Pro.** Zero new wire format. Composes perfectly with N1. Already shipping.

**Con.** **Garden is not "a thing" at the cryptographic layer** — it's a UCAN-
template-name. There's no Garden-level K_Set, no Garden-level membership-set,
no Garden-level CRDT-merge of MembershipEvent. "Posting to the Garden" = posting
N times (once per sub-Atrium the poster has grant for). Doesn't satisfy the L9 O7
fractal claim ("Atrium IS a Subgraph; SubgraphSpec is an Atrium-recursable primitive")
because Atrium-recursable means Atrium-can-contain-Atrium, not Atrium-can-be-
referenced-by-UCAN-template.

**Verdict.** WORKS for "permission-flavored Garden" (Discord-style hierarchy).
DOESN'T satisfy the architectural fractal claim. Should COMPOSE with the winning
shape (Garden grants can still be RestrictedScopeSet-attenuated for sub-Atrium
fan-out), not REPLACE it.

#### Option 4 — Recursive MembershipSet (MembershipSet whose members are themselves MembershipSets)

**Shape.** Extend the existing `MemberKey` enum (per-Kind member shape per N2 §1.2)
with a recursion variant. A "MembershipSet member" carries `(MembershipSetId, K_Set_acquisition_path)`.
The Garden's MembershipSet has K_Set-G; its "members" are sub-Atrium-A and sub-Atrium-B
as *MembershipSet-references*. Posting to the Garden = sealing under K_Set-G via
multi-stanza-HPKE; recipients are the *members of the referenced sub-Atriums*
(walked via L9 O7-style Drop-bundle), each holding an HPKE pk known via her sub-Atrium's
ledger. The walker resolves "Garden recipients" = ∪(walk(MembershipSetId_A.members),
walk(MembershipSetId_B.members)).

**Pro.** **Native fractal**: Atrium-of-Atriums has uniform crypto shape with Atrium.
Recursion-depth-limit invariant gives Garden-of-Gardens, Forest-of-Gardens, … all
the same way. Per-recipient unlinkability preserved (still one stanza per leaf
recipient). Wave-days post-v1-beta is moderate (~7-9). Composes with N1+N2+N3 +
existing F18 DUAL-CID + Inv-20.

**Con.** Recursion needs **depth limit** + **cycle detection** invariant (per LDAP/
Protocol-Buffers/Underscore-DoS precedents in §2). K_Set-G distribution at Garden-
membership-add becomes: the Garden admin walks the sub-Atrium membership ledgers
(via L9 O7 Drop transitivity) and multi-stanza-seals K_Set-G to each leaf member's
HPKE pk. This **leaks Garden-set-existence to sub-Atrium ledger-holders** (they
see "an outside Atrium asked for our members' HPKE pks"). Privacy-of-Garden-existence
is at MED level — mitigated by Q3 DUAL-CID for the Garden's own MembershipEvent
ledger, but NOT mitigated for the "I am a Garden recipient" inference (Sub-Atrium
admin sees their member receiving Garden-K_Set distribution stanzas).

**Verdict.** **STRONGEST candidate.** Matches L9 O7 fractal claim. Matches Jazz
CoValue extends-pattern precedent (the closest peer-system). The Garden-existence-
leak at K_Set-G distribution is the one real con — addressed in §6 by the **hybrid**
recommendation.

#### Option 5 (NEW PROPOSAL) — **Hybrid: Independent K_Set-G + Recursive Member-Source**

**Shape.** Garden is an independent `MembershipSet` (Atrium-Kind) with its own
CSPRNG-random K_Set-G (Option 1 shape for the *K_Set*). The Garden's `members`
field carries **MemberKey-or-SubsetRef** discriminator (Option 4's recursive
membership-source). Posting to Garden = seal under K_Set-G to per-stanza HPKE of
**resolved leaf members** (members of the recursively-walked sub-Atrium
memberships). Membership composition rule: "Garden member = ∪(walk(SubsetRef_i))
∪ {Direct Garden-only members}".

**Pro.** Combines the **cryptographic simplicity of Opt 1** (one K_Set per
Garden — clean rotation; clean per-stanza unlinkability) with the **fractal-
architectural cleanness of Opt 4** (membership-source is recursive; the Garden
"contains" sub-Atriums as first-class members). Garden-of-Gardens composes
identically. RestrictedScopeSet (Opt 3) layers on top *for free* — Garden members
get UCAN-grants that may attenuate to specific sub-graph scopes.

**Con.** Codepoint-reserve at v1-beta needs **both** slots: `garden_k_set_anchor`
(Opt 1's slot) AND `member_kind_tag` (Opt 4's slot). 2 codepoint-reserves vs 1
for either pure option. Mitigation: both are 1-byte additive sub-slots; total
wire-cost ~2 bytes per Garden-tier MembershipSet (zero cost at non-Garden tier).

**Verdict.** **THIS IS THE RECOMMENDED WINNER.** See §7.

### §3.3 Cost-and-wire-format summary

| Option | v1-beta wire-format reserve slots | Post-v1-beta Garden impl wave-days | Use-case breadth | Composability with N1/N2/N3 | Unlinkability | Recommended |
|---|---|---|---|---|---|---|
| 1 | 1 byte (`garden_k_set_anchor`) | 6-8 | linear-hierarchy | HIGH | PRESERVED | Workable |
| 2 | breaking (nested Vec<HpkeMultiBase>) | 12-15 (likely unsolvable) | weak | LOW | BROKEN | REJECTED |
| 3 | 0 (already shipped via N1) | 2-3 | permission-only | HIGH | PRESERVED | Composes-with-winner |
| 4 | 1 byte (`member_kind_tag`) + depth-limit invariant | 7-9 | broad | HIGH | PRESERVED | Strong candidate |
| **5 (hybrid)** | **2 bytes** (both slots above) + depth-limit Inv-20 clause-k | **7-9** (one extra distribution path) | **broadest** | HIGH | PRESERVED | **WINNER** |

---

## §4 Task 3 — Recursion depth limit

### §4.1 The question

Per L9 O7 fractal architecture: Atrium-of-Atriums. Logical extension: Garden-of-Gardens,
Forest-of-Gardens. How deep can recursion go? What enforces a sane bound at v1-beta-
freeze?

### §4.2 Cross-system precedents on depth limits

- **GraphQL query depth**: enforced server-side via max-depth + complexity-threshold;
  exploiting query depth is a textbook DoS vector (Checkmarx).
- **Protocol Buffers JSON parsing**: CVE-2026-0994 — bypass of recursion-depth-limit
  was a security vulnerability *because the limit existed*. Standard practice =
  hard upper bound enforced at parse time.
- **Underscore.js `_.flatten`/`_.isEqual`**: unlimited recursion = DoS via stack
  exhaustion (CVE-2026-27601). Fix was depth-limit parameter.
- **AutoMapper deeply-nested object graphs**: no default max-depth = StackOverflowException;
  fix was a configurable depth limit.
- **LDAP / AD nested groups**: AD enforces a tree-depth limit (effectively, but
  performance degrades non-linearly with depth; JumpCloud warns about "downsides
  and risks" before hitting hard limits).

**Pattern**: every system that adopts recursion-as-data needs a hard upper bound,
enforced at parse / API-boundary, and small enough that the worst-case walk fits
in available stack/memory/wall-clock.

### §4.3 Proposed bound for Benten Garden

**Invariant `MEMBERSHIP_RECURSION_MAX_DEPTH = 4`**, with enforcement sites:

- **Constructor validator** at `MembershipSet::from_policy_and_members`: depth-walk
  via fixed-depth DFS over `SubsetRef`-typed members; reject if any path exceeds
  4 levels.
- **Sibling-trait `Members::add_subset_member` API boundary**: pre-flight walk of
  the to-be-added subset's depth; reject with `E_MEMBERSHIP_RECURSION_DEPTH_EXCEEDED`
  if (depth(subset) + 1) > 4.
- **Cycle detection**: per Matrix-spaces precedent, walker must detect cycles
  (subset-A includes subset-B includes subset-A). Implementation: HashSet-of-
  MembershipSetIds visited during walk; cycle = reject with `E_MEMBERSHIP_RECURSION_CYCLE`.
- **`policy.metadata.recursion_depth: u8`** auto-computed cached field on each
  MembershipSet (depth-of-deepest-leaf), included in `policy_cid` so manipulation
  is signature-bound.

**Why 4 specifically:**

- Depth 1 = ordinary Atrium (sub-Atriums = members).
- Depth 2 = Garden (Atrium-of-Atriums).
- Depth 3 = Forest (Garden-of-Gardens) — supports cross-organization federations.
- Depth 4 = Ecosystem-of-Forests — supports inter-organization-coalition (e.g.,
  industry-consortium of multiple federation-of-orgs).
- Depth 5+ = no known real use case. The wave-days-to-design-a-use-case-doc would
  be greater than the wave-days-to-rebuild-the-recursion-limit if/when it materializes
  in 10+ years. Per HARD RULE 12 + "do the proper fix not the band-aid" — pick the
  smallest bound that fits enumerated use cases.

**v1-beta-freeze action:**

- Mint **Inv-20 clause-k**: `MEMBERSHIP_RECURSION_MAX_DEPTH = 4` + cycle-detection
  invariant; enforced at API boundary + constructor validator.
- Reserve `policy.metadata.recursion_depth: u8` slot (1 byte, additive).
- Document **revisit-trigger** at Compromise # mint: if production deployment
  surfaces a depth-5+ requirement, revisit invariant (analogous to #54's revisit-
  trigger discipline per N3 §7.4).

### §4.4 HARD RULE 12 §15.c compliance

§15.c says EXACTLY-3-arm `MembershipSetKind` is v1-beta-frozen; Garden 4th arm =
HALT-AND-SURFACE-TO-BEN. The recursion-limit invariant does NOT extend the enum.
It adds a **codepoint-reserve sub-slot under `Atrium`** (via the existing
`AtriumWithRotatingGroupKey`-style codepoint-reserve pattern, with a NEW sub-slot
`AtriumGardenRecursive`). Garden is therefore implemented post-v1-beta as a
*specialization of the Atrium arm*, not a 4th arm.

§15.c compliance verified: the 3-arm enum REMAINS EXACTLY 3 arms; Garden is the
codepoint-reserve sub-slot `MembershipSetKind::Atrium { specialization:
AtriumSpecialization::AtriumGardenRecursive }` (where `AtriumSpecialization` is
itself codepoint-reserve-extensible per the M-CONS-v2 codepoint-reserve discipline).

---

## §5 Task 4 — Cross-sub-Atrium membership semantics

### §5.1 The question

Can a user be a member of Garden-G but only of Sub-Atrium-A (not B)? Is Garden
membership ⊆ ∪(sub-Atrium memberships), or independent?

### §5.2 Cross-system precedents

- **Keybase**: subteam membership independent (can include non-parent members);
  implicit-admins (parent admins) don't auto-get content keys.
- **MS Teams private channels**: private-channel members ⊂ parent team members
  (must be parent member first).
- **Matrix spaces**: independent (canonical-parent is metadata, not membership-
  constraint).
- **Discord**: category membership IS server membership; channel sub-set defined
  by permission-overrides.
- **Slack Enterprise Grid shared channels**: members can span workspaces (one
  shared channel; multiple workspace memberships).

### §5.3 Two viable semantic models

**Model A (Subset-constrained): Garden member ⊆ ∪(Sub-Atrium memberships).**

Every Garden member must be a member of at least one Sub-Atrium. Constructor
validator enforces this at the API boundary (`Members::add_member(garden, user_X)`
checks `∃ Sub-Atrium S ∈ garden.subset_refs : user_X ∈ S.members`).

- **Pro**: Matches MS-Teams private-channel intuition. Defends against admin
  accidentally adding a stranger to the Garden. Enforces "Garden = roof over
  pre-existing sub-Atriums."
- **Con**: Sub-Atrium-A's admin removing user_X kicks them out of the Garden too
  (via the subset constraint). This is **a cross-MembershipSet causal coupling**
  — sub-Atrium-A's MembershipEvent (remove user_X) triggers a Garden-level
  MembershipEvent (cascade-remove user_X). Concurrent-event semantic per N4
  R-N4-3a needs extension to "cross-MembershipSet cascade events."

**Model B (Independent): Garden membership orthogonal to sub-Atrium memberships.**

Garden has its own membership. A Garden member need not be in any sub-Atrium.
The "fractal" semantic is purely architectural — sub-Atriums are *content-source
groups* that aggregate into the Garden; Garden members are *audience* who can
read Garden-tier content.

- **Pro**: Matches Keybase + Matrix + Slack precedents (the majority). No cross-
  MembershipSet causal coupling. Simpler N4 concurrent-fork tie-break.
- **Con**: Doesn't enforce "Garden = roof over sub-Atriums" intuition. Admin
  could add a stranger to Garden without realizing they're getting access to
  aggregated sub-Atrium content.

### §5.4 Recommendation: **Model B (Independent) at v1-beta-freeze; Model A as opt-in via per-instance `policy.metadata.subset_constraint: bool`**

**Reasoning.**

1. Model B matches MAJORITY of cross-system precedents (Keybase, Matrix, Slack,
   ATProto).
2. Model A's cross-MembershipSet causal coupling creates an N4-extension burden
   (tie-break rules for cross-set cascade events) that's NOT yet designed; better
   to leave A as opt-in than to make it default and surface fork-tie-break design
   debt at Garden-impl time.
3. The "stranger added to Garden" concern is addressed by Garden admin's existing
   capability (`add_member` is admin-gated per Inv-20 clause-i) + plain-English
   UI surface ("Adding user_X who is not in any sub-Atrium of this Garden — confirm?").

**Codepoint-reserve action at v1-beta:**

- Reserve `policy.metadata.subset_constraint: Option<bool>` slot (1 byte; defaults
  to `None` ≡ Model B).
- Document that Model A is post-v1-beta and requires N4 R-N4-3a extension to
  cross-MembershipSet cascade events (a separate post-v1-beta design pass; reserve
  in Compromise # registry).

### §5.5 Inv-20 clause-? assignment

The existing Inv-20 clauses (a)..(j) don't name a Garden-membership-cardinality
rule. Two new clauses minted at v1-beta-freeze:

- **Inv-20 clause-k** (Task 3): `MEMBERSHIP_RECURSION_MAX_DEPTH = 4` + cycle-
  detection.
- **Inv-20 clause-l** (Task 4): Garden-membership defaults to Model B independent;
  Model A subset-constrained semantics is opt-in via `policy.metadata.subset_constraint`
  AND is post-v1-beta (requires cross-MembershipSet cascade-event N4 extension).

These are codepoint-reserve-additive (1 byte each at policy.metadata layer). Sum
total v1-beta wire-format additions for Garden: 4 bytes (anchor + member-kind-tag +
recursion-depth + subset-constraint).

---

## §6 Task 5 — Use-case validation

For each option (focusing on winners — Opt 1, Opt 3, Opt 4, and Opt 5 hybrid):

### §6.1 Use-case: Open-source project Garden

**Spec.** Project = Garden; teams (frontend, backend, infra) = sub-Atriums;
contributors join Garden via any team.

| Option | Fit |
|---|---|
| 1 | OK — Garden = pure-flat with all contributors. Doesn't express "frontend-team" structure. |
| 3 | GREAT — pure UCAN-grant approach: contributor's UCAN grants attenuate to per-team scopes; "Garden-level" = top-level UCAN-grant ancestor. |
| 4 | GREAT — Garden lists frontend/backend/infra as sub-MembershipSets; contributor in any team = recipient of Garden-tier posts. |
| 5 | GREAT — combines #4's structure with #1's central K_Set-G for Garden-level announcements (e.g., "weekly all-hands message"). |

### §6.2 Use-case: Family Garden

**Spec.** Family = Garden; sub-families (parents + kids, grandparents, in-laws)
= sub-Atriums; some members in just one branch.

| Option | Fit |
|---|---|
| 1 | POOR — would require manually re-adding every family member at Garden level. |
| 3 | OK — purely permission-based; "family-wide" content posted N times to each sub-family. |
| 4 | GREAT — Family-Garden contains sub-family MembershipSets; "family-wide" announcement posted once, walked-to all leaf members. |
| 5 | GREAT — same as #4 plus enables family-wide K_Set-G rotation independent of any sub-family rotation (e.g., divorce → re-form Garden membership independently of sub-family memberships). |

### §6.3 Use-case: Company Garden

**Spec.** Company = Garden; departments = sub-Atriums; cross-department members
(execs, contractors) common.

| Option | Fit |
|---|---|
| 1 | OK — Garden is the all-company channel; cross-department members must be admin-added. |
| 3 | GREAT for Discord-style company chat: per-channel permissions; no central E2EE. |
| 4 | GREAT — company has departments; CEO's "to: company" message → walk all departments. |
| 5 | **BEST** — supports both "to: all-employees-via-departments" (recursive walk) AND "to: cross-org-board-of-directors-not-in-any-department" (direct Garden members; Model B independent). |

### §6.4 Use-case: Game-League Garden

**Spec.** League = Garden; teams = sub-Atriums; players in exactly one team but
can see league-level content (rules, schedule, standings).

| Option | Fit |
|---|---|
| 1 | OK — Garden = league-channel with all players. |
| 3 | GREAT — UCAN-grants: each player's team-grant attenuates from a league-level grant. |
| 4 | GREAT — League contains teams; "league announcement" walks all teams. |
| 5 | GREAT — supports both "all-players-via-teams" (recursive walk) AND "league-commissioners-not-in-any-team" (direct Garden member). |

### §6.5 Synthesis

**Option 5 hybrid wins on use-case breadth.** It's the only shape that supports
*all four named use-cases cleanly* without forcing a domain into the wrong
abstraction:

- Direct Garden members (Model B independent) for "VIPs/admins not in any sub-team."
- Recursive sub-Atrium aggregation for "audience derives from leaf-team memberships."
- Subset-constraint opt-in (Model A) for use-cases that want it (post-v1-beta).
- Composes with RestrictedScopeSet (Opt 3) for permission-level attenuation.

Option 4 is a close second (only loses the "VIPs not in any sub-team" case if
strict-recursive-only). Option 1 is workable but doesn't capture fractal structure.
Option 3 is an attenuation layer that should compose with the winner, not
substitute for it. Option 2 is rejected on unlinkability + wire-cost grounds at
every use-case.

---

## §7 Task 6 — Final recommendation + v1-beta wire-format-pin

### §7.1 Winner: **Option 5 (Hybrid: independent K_Set-G + recursive member-source)**

**Justification synthesized.**

- **Cryptographic soundness HIGH** (clean independent K_Set per Garden tier;
  per-stanza HPKE preserves Inv-20 clause-d).
- **Composability HIGH** with N1 (RestrictedScopeSet layers freely), N2 (24-op
  surface parameterizes over recursive members trivially), N3 (per-tier
  independent rotation triggers).
- **Use-case breadth BROADEST** (all 4 named use-cases supported cleanly).
- **Wave-days post-v1-beta MODERATE** (~7-9 days), absorbed into Wave-MS-PRIMITIVE
  canary or a future Phase-5+ wave.
- **v1-beta wire-format-pin COMPACT** (~4 bytes additive in `policy.metadata`).
- **Recursion-bound CLEAN** (Inv-20 clause-k: depth ≤ 4 + cycle detection).
- **Independent + recursive together** = the architectural fractal claim from
  L9 O7 satisfied (Atrium recursable into Atrium) AND the Keybase/MS-Teams
  precedent for "independent crypto, inherited control" satisfied.

### §7.2 v1-beta wire-format-pin (CONCRETE)

The following additive sub-slot additions at v1-beta-freeze ensure Garden-impl
post-v1-beta is purely additive:

#### §7.2.1 `MembershipSetPolicy.metadata` additions (codepoint-reserve)

```rust
// In benten-membership-set::policy::PolicyMetadata
pub struct PolicyMetadata {
    // ... existing fields (M-CONS-v2 §4) ...

    /// (Garden codepoint-reserve, Inv-20 clause-l)
    /// If `Some(true)`, this MembershipSet enforces Model A subset-constraint
    /// semantics — Garden members ⊆ ∪(sub-Atrium memberships). If `None` or
    /// `Some(false)`, Model B independent semantics.
    /// **At v1-beta:** field is RESERVED; engine MUST refuse `Some(true)` with
    /// `E_MEMBERSHIP_SUBSET_CONSTRAINT_UNSUPPORTED` (post-v1-beta opt-in).
    pub subset_constraint: Option<bool>,

    /// (Garden codepoint-reserve, Inv-20 clause-k)
    /// Auto-computed cached depth-of-deepest-leaf for recursion-bound enforcement.
    /// 0 = leaf MembershipSet (no SubsetRef members). Max value: 4.
    /// **At v1-beta:** field is RESERVED; engine MUST refuse non-zero with
    /// `E_MEMBERSHIP_RECURSION_UNSUPPORTED` (post-v1-beta opt-in).
    pub recursion_depth: u8,

    /// (Garden codepoint-reserve)
    /// Optional pointer to a parent Garden's MembershipSetId for back-link
    /// discovery (Matrix-spaces precedent: bidirectional parent-child).
    /// **At v1-beta:** field is RESERVED; engine MUST refuse non-None with
    /// `E_MEMBERSHIP_GARDEN_ANCHOR_UNSUPPORTED`.
    pub garden_parent_anchor: Option<MembershipSetId>,
}
```

#### §7.2.2 `MemberKey` enum variant addition (codepoint-reserve)

```rust
// In benten-membership-set::member::MemberKey
pub enum MemberKey {
    // ... existing per-Kind variants (UserDid, DeviceDid, etc.) ...

    /// (Garden codepoint-reserve, R-MCV2-8 / this doc §3.2 Option 5)
    /// A "member" that is itself a MembershipSet. Resolves to its leaf members
    /// via recursive walk (bounded by Inv-20 clause-k recursion depth).
    /// **At v1-beta:** variant is RESERVED; engine MUST refuse construction
    /// with `E_MEMBERSHIP_SUBSET_REF_UNSUPPORTED` (post-v1-beta opt-in).
    SubsetRef {
        membership_set_id: MembershipSetId,
        /// K_Set acquisition path per R-MCV2-2 triple discipline.
        k_set_acquisition: KSetAcquisitionPath,
    },
}

/// Codepoint-reserve for K_Set-acquisition strategies (R-MCV2-2).
pub enum KSetAcquisitionPath {
    /// Garden admin walks sub-Atrium ledger, distributes K_Set-G via per-leaf
    /// HPKE-multi-stanza-seal (Option 5 default).
    AdminWalkDistribute,
    /// Sub-Atrium admin grants Garden-K_Set acquisition via UCAN-delegation
    /// at sub-Atrium-membership-add time (alternative for higher Sub-Atrium-
    /// admin sovereignty).
    SubAdminGrantDelegate,
    // ... future codepoint-reserve slots ...
}
```

#### §7.2.3 `MembershipSetKind` codepoint-reserve sub-slot addition

```rust
// In benten-membership-set::kind::MembershipSetKind  (UNCHANGED 3-arm enum per §15.c)
pub enum MembershipSetKind {
    Atrium { specialization: AtriumSpecialization },  // EXISTING — adds specialization slot
    DeviceMesh,                                        // EXISTING (unchanged)
    SingleDevice,                                      // EXISTING (unchanged)
}

/// (M-CONS-v2 + this doc §7.2.3)
/// Codepoint-reserve sub-slot under Atrium for post-v1-beta specializations.
/// At v1-beta-freeze, only `Plain` is permitted; engine refuses other variants
/// with their per-variant `E_..._UNSUPPORTED` code.
pub enum AtriumSpecialization {
    Plain,                                             // v1-beta default
    AtriumWithRotatingGroupKey,                        // M-CONS-v2 N4 (codepoint-reserve)
    AtriumGardenRecursive,                             // THIS DOC (codepoint-reserve)
    // ... future codepoint-reserve slots ...
}
```

#### §7.2.4 New error codes (cross-language mirrored per §3.5g)

```rust
// In benten-errors::ErrorCode
pub enum ErrorCode {
    // ... existing variants ...
    EMembershipSubsetConstraintUnsupported,    // v1-beta gate for subset_constraint
    EMembershipRecursionUnsupported,           // v1-beta gate for recursion_depth
    EMembershipGardenAnchorUnsupported,        // v1-beta gate for garden_parent_anchor
    EMembershipSubsetRefUnsupported,           // v1-beta gate for SubsetRef variant
    EMembershipRecursionDepthExceeded,         // post-v1-beta: > Inv-20 clause-k bound
    EMembershipRecursionCycle,                 // post-v1-beta: cycle detected
}
```

Each MUST have TS-side mirror per §3.5g cross-language rule-mirror.

#### §7.2.5 Total wire-format additions at v1-beta

- `PolicyMetadata`: 3 new fields (`subset_constraint: Option<bool>` ≈ 1 byte;
  `recursion_depth: u8` = 1 byte; `garden_parent_anchor: Option<MembershipSetId>`
  ≈ 33 bytes if set, ~1 byte if None).
- `MemberKey::SubsetRef` variant: 0 bytes if unused; codepoint discriminator
  reserved in MemberKey enum (~1 byte tag space).
- `MembershipSetKind::Atrium::specialization`: 1 byte sub-tag (currently absorbs
  AtriumWithRotatingGroupKey + AtriumGardenRecursive + future slots).
- `KSetAcquisitionPath`: codepoint-reserve enum within SubsetRef.
- 6 new error codes (Rust + TS mirrors).

**Total v1-beta-resident overhead for non-Garden MembershipSets: ~3 bytes per policy
(all optional-None or zero-valued).** Garden-impl post-v1-beta: ~7-9 wave-days
absorbing slot-binding + walker + distribution protocol + tests.

#### §7.2.6 AAD-binding (M-C2-v2 R-MCV2 absorption)

Per R-MCV2-1 (role_generation in AAD), R-MCV2-9a (RotationTrigger in AAD): the new
`policy_cid` (which includes the above metadata fields via canonical-CBOR) is
already AAD-bound per Inv-20 clause-c (the policy_cid binds the entire MembershipSet
policy into every sealed envelope's AAD). Therefore Garden codepoint-reserve fields
inherit AAD-binding for free; no per-field AAD additions needed.

### §7.3 Engine refusal discipline at v1-beta

At v1-beta-freeze, the engine SHALL:

- Accept `PolicyMetadata` with the 3 new fields present but set to defaults
  (None/0/None). This preserves wire-format-additivity (a v1-beta engine reads
  a future Garden-impl policy and refuses it cleanly, but doesn't crash).
- Refuse non-default values for those fields with the appropriate
  `E_..._UNSUPPORTED` code (per Compromise #53-narrowed-style discipline).
- Accept `AtriumSpecialization::Plain` only; refuse `AtriumGardenRecursive` and
  `AtriumWithRotatingGroupKey` with `E_..._UNSUPPORTED`.
- Refuse `MemberKey::SubsetRef` construction with `E_MEMBERSHIP_SUBSET_REF_UNSUPPORTED`.

This **doc-only** discipline ships at v1-beta-freeze (zero code beyond the field
additions + error-code mints + 5 unit tests asserting the refusal). Garden-impl
post-v1-beta flips the engine-refusal branches to engine-acceptance branches.

---

## §8 Task 7 — R0 plan-doc + Compromise mint

### §8.1 M-CONS-v2 §-row update for Garden codepoint-reserve

Add to M-CONS-v2's §4 final-final F-amendment table (replacing R-MCV2-8 as a
"REFINED" entry per the M-CONS-v2.1 re-consolidator pattern):

```
| F-G1 | M-CONS-v2 + this doc P3 F-D | Garden / Atrium-of-Atriums codepoint-reserve sub-slot under MembershipSetKind::Atrium::specialization::AtriumGardenRecursive + recursive MemberKey::SubsetRef variant + independent K_Set-G primary + 3 PolicyMetadata fields (subset_constraint, recursion_depth, garden_parent_anchor) + Inv-20 clauses-k+l (MEMBERSHIP_RECURSION_MAX_DEPTH = 4 + Model B default + Model A opt-in post-v1-beta) | LB | CR | v1β-CR (codepoint-reserve only; impl post-v1-beta Phase-5-7+) | F17 + F-N2-A + N1-RestrictedScopeSet | new MembershipSet-Spec §-add + V1-FROZEN-INTERFACE.md row + Compromise #59 mint | MH (codepoint-reserve discipline well-precedented; impl-deferral honest-disclosure) |
```

### §8.2 New Compromise mint — Compromise #59

```
| #59 (NEW this doc) | this doc §8 + M-CONS-v2.1 R0 plan-doc | **Garden / Atrium-of-Atriums composition shape PINNED-IMPL-DEFERRED.** Composition shape is Option 5 hybrid (independent K_Set-G + recursive MemberKey::SubsetRef + Inv-20 clauses k+l recursion-bound + Model B independent default). Codepoint-reserve sub-slots ship at v1-beta-freeze (additive ~3 bytes per policy.metadata + 1 byte MemberKind enum tag + 6 new error codes). Engine refuses non-default values with `E_..._UNSUPPORTED` codes per Compromise-#53-narrowed-style discipline. Garden-impl wave (~7-9 wave-days) sequenced post-v1-beta at Phase-5-7+. **Honest-architectural-disclosure framing:** v1-beta SHIPS the Garden *shape-lock* not the Garden *feature*; this is the L9 O7 fractal-architecture commitment in additive-wire-form. Revisit-trigger: any Phase-5+ deployment requiring inter-organization-aggregation feature surfaces before Phase 7 → re-prioritize Garden-impl wave. **Disposition class:** Composition-Hazard-Honest-Disclosure (per R-MCV2-9b NEW class). |
```

### §8.3 Disposition class

Per R-MCV2-9b (M-C2-v2 mint of `Composition-Hazard-Honest-Disclosure` third
disposition class), Compromise #59 is appropriately classified as that class —
NOT as #54-style "deferral-risk" (which names a specific revisit-trigger for a
deferred feature) NOR as #55-style "P2P-by-design semantic" (which names a
structural invariant accepted as cost). #59 instead is "we have decided the SHAPE
now to avoid future wire-breakage, but the IMPL is deferred."

This class composition aligns with R-MCV2-B-1's Compromise #57 mint (RestrictedScopeSet
immutability) and R-MCV2-B-3's Compromise #58 mint (audit-log insider-correlation)
— all three being honest-architectural-disclosure-class.

### §8.4 R0 plan-doc obligations

The M-CONS-v2.1 R0 plan-doc (per ADDL pipeline) MUST include the following for
Garden codepoint-reserve compliance:

1. **F-G1 row** in §4 F-amendment table (per §8.1 above).
2. **Compromise #59 entry** in §4 Compromise # registry (per §8.2 above).
3. **Inv-20 clauses-k+l minted** in §3 Invariants table (per §4 + §5 above).
4. **MembershipSet-Spec §-add**: new sub-section under "Codepoint-reserve sub-slots"
   pinning the wire-format (per §7.2 above) — explicit module boundary check at
   inherent-impl + sibling-trait + RestrictedScopeSet 3-way validators per
   R-MCV2-B-4 closure.
5. **V1-FROZEN-INTERFACE.md row** (per §3.5g cross-doc mirror discipline): list
   each of `PolicyMetadata.{subset_constraint, recursion_depth, garden_parent_anchor}`
   + `MemberKey::SubsetRef` + `MembershipSetKind::Atrium::specialization` enum +
   `KSetAcquisitionPath` enum as Y (frozen at v1-beta-freeze) + Reserve-disposition
   (codepoint-reserve only; refuse-with-`E_..._UNSUPPORTED` at v1-beta-freeze).
6. **V1-FROZEN-INTERFACE-DEFERRED.md Row D-NEW-Garden**: Garden-impl wave (~7-9
   wave-days; Phase-5-7+; revisit-trigger per #59).
7. **Cross-language rule-mirror per §3.5g**: 6 new ErrorCodes (Rust + TS).
8. **Test pins**:
   - 1 RED-PHASE test asserting `E_MEMBERSHIP_SUBSET_REF_UNSUPPORTED` on
     SubsetRef construction at v1-beta engine (covers all 4 v1-beta refusal
     codes via parametric pin).
   - 1 wire-format-round-trip test: PolicyMetadata with all 3 Garden fields
     present (non-default) parses + re-encodes byte-identical (additive-wire-
     format pin).
   - 1 phantom-cite scanner gate on this doc's path-style cites.

### §8.5 Wave-day cost at v1-beta-freeze for Garden codepoint-reserve only

| Item | Wave-day cost |
|---|---|
| 3 PolicyMetadata fields + MemberKey::SubsetRef variant + AtriumSpecialization enum + KSetAcquisitionPath enum (Rust + TS mirror) | +0.4 |
| 6 ErrorCode variants (Rust + TS mirror per §3.5g) | +0.2 |
| Engine-refusal branches + 5 unit tests | +0.2 |
| MembershipSet-Spec §-add doc | +0.1 |
| V1-FROZEN-INTERFACE.md row + V1-FROZEN-INTERFACE-DEFERRED.md row | +0.1 |
| Compromise #59 + Inv-20 clauses-k+l doc updates | +0.1 |
| **Total v1-beta-freeze impact** | **~1.1 wave-days** |

Absorbs into Wave-MS-PRIMITIVE canary cleanly. **Garden IMPL** wave (~7-9 wave-
days) sequenced post-v1-beta per Compromise #59.

### §8.6 Composition with M-C2-v2 refinements

This doc's recommendation **explicitly composes** with M-C2-v2's R-MCV2-2
(RestrictedScopeSet × MembershipSet × K_Set-acquisition-path triple) by adopting
`KSetAcquisitionPath` as a first-class enum on `SubsetRef`. R-MCV2-2's mandated
per-MembershipSet-Kind composition table extends to a third dimension at Garden-
tier:

```
(Kind, RestrictedScopeSet-shape, K_Set-acquisition-path) → Allowed/Error
```

becomes at Garden tier:

```
(Kind, RestrictedScopeSet-shape, K_Set-acquisition-path,
 recursion-depth-at-this-tier) → Allowed/Error
```

The matrix at depth-0 (no recursion) is the M-C2-v2 R-MCV2-2 + R-MCV2-B-4 matrix;
the matrix at depth-1..4 adds rows for the recursion-aware K_Set acquisition
sub-table. Documented in MembershipSet-Spec §-add per §8.4 item 4.

---

## §9 Summary

**Garden / Atrium-of-Atriums shape PINNED at v1-beta as Option 5 hybrid**: an
independent CSPRNG-random `K_Set-G` on a Garden-tier `MembershipSet` whose `members`
field carries a new `SubsetRef` variant pointing recursively at sub-MembershipSets
(per L9 O7 fractal-architecture + Jazz CoValue extends-pattern precedent +
Keybase subteam independent-crypto precedent). Recursion bounded at depth-4 +
cycle-detection (Inv-20 clause-k; LDAP/Protocol-Buffers/GraphQL DoS precedents).
Default cross-tier membership-semantic is Model B independent (Inv-20 clause-l;
Keybase/Matrix/Slack majority precedent); Model A subset-constrained is post-v1-beta
opt-in via `policy.metadata.subset_constraint`. Wire-format-pin at v1-beta-freeze:
3 `policy.metadata` fields + 1 `MemberKey::SubsetRef` variant + 1
`MembershipSetKind::Atrium::specialization` enum + 1 `KSetAcquisitionPath` enum +
6 new ErrorCodes (cross-language mirrored per §3.5g). Total v1-beta-freeze cost
~1.1 wave-days (codepoint-reserve doc + field additions + engine-refusal branches
+ 5 unit tests). Garden-impl wave ~7-9 wave-days, sequenced post-v1-beta at
Phase-5-7+. **New Compromise #59** minted as `Composition-Hazard-Honest-Disclosure`
class (per R-MCV2-9b new class) — "Garden shape-lock at v1-beta; impl deferred."
Inv-20 clauses-k+l minted. Composes with M-C2-v2 R-MCV2-1..9 + B-1..4 refinements;
absorbs into Wave-MS-PRIMITIVE canary. M-CONS-v2.1 R0 plan-doc obligations
itemized in §8.4. Option 2 (composed encrypt-to-both) REJECTED on per-recipient
unlinkability + wire-cost + zero-mainstream-precedent grounds.

---

## Sources (web research)

- [Keybase Book — Teams Design](https://book.keybase.io/docs/teams/design)
- [Keybase Book — Teams Details](https://book.keybase.io/docs/teams/details)
- [Matrix-spec-proposals — Spaces and Room Organization](https://deepwiki.com/matrix-org/matrix-spec-proposals/4.1-spaces-and-room-organization)
- [Discord — Channel Permissions and Inheritance](https://support.discord.com/hc/en-us/community/posts/4417761550487-Channel-Permissions-and-Inheritance)
- [Discord — Permissions by Categories, Channels and Roles Explained](https://www.mava.app/blog/discord-permissions-by-categories-channels-and-roles-explained)
- [Slack — Manage multi-workspace channels on Enterprise Grid](https://slack.com/help/articles/115004485887-Manage-multi-workspace-channels-on-Enterprise-Grid)
- [Microsoft — Encryption for SharePoint and OneDrive, Microsoft Teams, and Exchange](https://learn.microsoft.com/en-us/compliance/assurance/assurance-encryption-for-microsoft-365-services)
- [Microsoft — Private channels in Microsoft Teams](https://learn.microsoft.com/en-us/microsoftteams/private-channels)
- [Jazz — Groups as permission scopes](https://jazz.tools/docs/react/permissions-and-sharing/overview)
- [Jazz — permissions.ts source](https://github.com/garden-co/jazz/blob/main/packages/cojson/src/permissions.ts)
- [iroh-willow crate docs](https://docs.rs/iroh-willow/latest/iroh_willow/)
- [Willow Protocol — Comparison to Other Protocols](https://willowprotocol.org/more/willow_compared/index.html)
- [AT Protocol — Protocol Overview](https://atproto.com/guides/overview)
- [Cryspen — MLS Three (thousand) may keep a secret](https://cryspen.com/post/mls-introduction/)
- [Signal Private Group System paper](https://eprint.iacr.org/2019/1416.pdf)
- [GHSA — Underscore.js unlimited recursion DoS (CVE-2026-27601)](https://github.com/jashkenas/underscore/security/advisories/GHSA-qpx9-hpmf-5gmw)
- [GitLab Advisory — protobuf JSON recursion depth bypass (CVE-2026-0994)](https://advisories.gitlab.com/pkg/pypi/protobuf/CVE-2026-0994/)
- [Checkmarx — Exploiting GraphQL Query Depth](https://checkmarx.com/blog/exploiting-graphql-query-depth/)
- [JumpCloud — Downsides and Risks of Nested Groups](https://jumpcloud.com/blog/nested-groups)
