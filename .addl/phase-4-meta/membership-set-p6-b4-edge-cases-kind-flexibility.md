# MembershipSet — **P6-B4 edge-cases + Kind-flexibility audit**

> **Top-banner re-orient (HANDOFF discipline).** This document is the **P6-B4 edge-
> case + Kind-flexibility specialist audit** of M-CONS-v2's `enum MembershipSetKind
> { Atrium, DeviceMesh, SingleDevice }` (EXACTLY-3 arms per §15.c). Ben's framing
> motivates: he can imagine future use-cases involving hybrid memberships (humans +
> devices + AI assistants), game backends, federated business structures, etc.;
> he does NOT want the 3-Kind enum to be overly restrictive. The output evaluates
> **sufficiency** + **edge-case fitness** against the elegant-permanent-shape
> reflection-pass discipline (`feedback_extra_reflection_pass_for_elegant_permanent_shape.md`).
>
> Read BEYOND what's named here if you arrive cold: M-CONS-v2 at `e62ff540`
> (especially §1.2 N2 hybrid, §3 F-N2-A/B/C, §7.3 canary enumeration, §15.c
> EXACTLY-3-arm invariant); M-C2-v2 at `0533d0cf` (§5 MCV2-B-4 3-way composition
> blind-spot); N2 at `a1b5a552` (§1.2 hybrid table, §3 24-op classification,
> §15.c HALT-AND-SURFACE); M2 at `6170980b` (typed-variant rationale); M4 at
> `066785b5` (BREAK 5 homogeneous-members / plugin-DIDs-are-NOT-members /
> §11.5); M5 at `15819500` (CGKA survey); CLAUDE.md #18 plugin trust model;
> `docs/V1-FROZEN-INTERFACE.md` §15.c discipline; MEMORY.md
> `feedback_extra_reflection_pass_for_elegant_permanent_shape.md` +
> `feedback_engine_primitives_vs_application_layer.md` +
> `feedback_plugin_extension_trust_model.md` + `feedback_no_defer_HARD_RULE.md`.
>
> Tree HEAD at write: `2172cb6d` (origin/main, fetched + verified at session
> start: clean tree; 0 ahead / 0 behind). Branch:
> `phase-4-meta-core/membership-set-p6-b4-edge-cases-kind-flexibility`. Date:
> 2026-05-28.

- **Role.** P6-B4 specialist — senior domain-modeler + use-case futurist
  evaluating MembershipSetKind sufficiency under foresight of hybrid /
  AI-agent / game-backend / federated futures. ADVISORY; my-pred winners
  named explicitly with HARD RULE 12 dispositions for unselected alternatives.
- **Scope.** Tasks 1-7 per dispatch brief. NOT primitive Rust API drafting
  (N2 / Wave-MS-PRIMITIVE own canonical shape). NOT a full critique
  round (M-C1/M-C2/M-C3 own that). This is a foresight + elegant-shape audit
  feeding into R0 plan-doc + §15.c amendment posture.
- **Out-of-scope.** Re-litigating M-CONS-v2's hybrid HYBRID-RECOMMENDED
  verdict on Authority slot + RBAC (N2 won that battle). Re-litigating the
  homogeneous-per-Kind MemberKey invariant (M4 BREAK 5 is FIXED INPUT for this
  audit's starting position; I evaluate whether it WILL HOLD against the
  edge-cases).

---

## §0 Headline summary (read-cold)

**Verdict on the 3-Kind enum sufficiency.** **The 3-Kind enum is the right
shape for v1-beta IFF augmented with a Type-tagged codepoint-reserve
discipline + a clear additive-extension contract.** The enum itself is
neither "structurally final" (Option A) nor needs to collapse to a runtime tag
(Option C). The 15+ edge-cases I enumerate cleanly fall into **3 disposition
buckets**: (1) **DECOMPOSES** under application-layer composition (the engine
primitive stays 3-arm; the use-case composes over it); (2) **CODEPOINT-RESERVE
SLOT** (the use-case names a Kind shape that needs reserved wire codepoint at
v1-beta + body-impl post-v1-beta); (3) **FRESH-PRIMITIVE-CLASS** (the use-case
is NOT a MembershipSet at all; it's a different engine primitive that may
compose with MembershipSet but isn't one). Ben's specific AI-assistant-as-
member hint resolves to **bucket-2 (codepoint-reserve)** PLUS application-layer
**composition** — a new `MembershipSetKind::HybridAtriumGarden` codepoint slot
(or amenably reuse `Atrium`+UCAN-scope+plugin-DID-via-DIDsubclass) — not a
4th first-class enum arm at v1-beta.

**Recommended winner (Task 3).** **Option E-PRIME — Hybrid-typed at v1-beta
+ Codepoint-reserve for 5-7 future Kinds + Application-layer composition
default + §15.c HALT-AND-SURFACE-TO-BEN gate retained.** Specifically:

- **Active enum:** `MembershipSetKind { Atrium, DeviceMesh, SingleDevice }`
  EXACTLY-3 (per N2 + M-CONS-v2; unchanged).
- **Codepoint-reserve table (NEW M-CONS-v2 add):** **6 reserved codepoint
  slots in `CRYPTO-CODEPOINTS.md` `MEMBERSHIP_SET_KIND` codepoint family** —
  `AtriumWithRotatingGroupKey` (already reserved per N4) + 5 NEW: `Garden`,
  `Grove`, `EphemeralLobby`, `Federation`, `Custom(CustomKindCid)`. Each
  reserved-only at v1-beta with `E_MEMBERSHIP_SET_KIND_UNSUPPORTED`; future
  body-impl requires §15.c HALT-AND-SURFACE-TO-BEN ratification.
- **Application-layer composition default.** Most "future use-cases"
  (Ben's AI-assistant-as-member, hybrid Atrium with IoT devices, game lobby,
  federation, B2B, customer-vendor) **decompose** under the existing engine
  primitives without a new Kind. Documented in MembershipSet-Spec §-N as
  "Composition recipes" — concrete pattern recipes (Atrium-plus-UCAN-scoped-
  plugin-DID-fleet for AI-assistants; DeviceMesh-per-vendor + Atrium-as-
  federation-root for B2B; etc.).
- **§15.c HALT-AND-SURFACE gate.** UNCHANGED. The 6 reserved slots make
  extension WIRE-format-additive but still BODY-impl-requires-Ben.
- **NEW: `MembershipSetKindReserved` typed wire codepoint family.** Existing
  `MembershipSetKind` enum stays 3-arm at the Rust + TS surface. The wire
  format carries a separate `MEMBERSHIP_SET_KIND_CODEPOINT: u16` field
  alongside the enum-variant codepoints in the codepoint-reserve range —
  symmetric with how `BindingContext` + `EnvelopePayload` already work
  (`#[non_exhaustive]` + typed-reject + reserved codepoints per F9/F10).

**Why this wins.** It preserves M-CONS-v2's compile-time-typed-rigor win
(per N2 hybrid recommendation §1.2) WITHOUT precluding the 5+ future
extensions Ben's foresight gestures at, AT zero v1-beta scope cost beyond a
~0.3 wave-day codepoint-reserve table addition. **Strictly less code than
adding 4-5 enum arms now; structurally identical to existing F9/F10
`#[non_exhaustive]` + codepoint-reserve discipline; closes the
"overly-restrictive" concern by explicit advance.**

**Cost.** **~0.3-0.5 wave-days** at v1-beta:
codepoint-reserve table extension in `CRYPTO-CODEPOINTS.md` +
`MembershipSet-Spec` §-add for the 5 new reserved codepoints +
`MembershipSet-Spec` §-add for composition recipes (Ben's AI-assistant
example as canonical recipe) + §15.c amendment to enumerate the reserved
codepoints + 1 NAMED-FORK-NOW backlog row per reserved codepoint pre-mounted
to `V1-FROZEN-INTERFACE-DEFERRED.md`. Folded into Wave-MS-PRIMITIVE canary.

**Confidence.** **HIGH on the directional recommendation** (Option E-PRIME
falls naturally out of existing M-CONS-v2 codepoint-reserve discipline +
F9/F10 + N4 `AtriumWithRotatingGroupKey` precedent; the design pattern is
already in the codebase; this is "do more of what's already working").
**MED-HIGH on the specific 5-codepoint enumeration** (the 5 reserved Kinds
I name below are my best foresight; Ben may add/subtract 1-2 based on his
domain intuition). **HIGH on the rejection of Options A/C/D** as winners.

---

## §1 Task 1 — Enumerate 15+ edge-case use-cases beyond the 3 Kinds

I enumerate **22 use-cases** below — 15 from the orchestrator brainstorm
substantially extended + 7 fresh additions. Each is profiled on the 6 axes
(member-key / authority / rotation / audit / lifecycle / compose-with).

### §1.1 Use-case enumeration table

| # | Use-case | Member-key shape | Authority shape | Rotation policy | Audit semantics | Lifecycle | Composes with |
|---|---|---|---|---|---|---|---|
| **UC-01** | Hybrid Atrium-Garden-Grove (Ben's hint): humans + devices + AI assistants in one community where AI's are "ambient compute executors" | Mixed: `UserDid` + `DeviceDid` + `AgentDid` (NEW) | Multi-admin (human admins only); AI's are members-without-admin-rights | FORK-ONLY at v1-beta | Full disclosure to current members | Long-lived | N1 RestrictedScopeSet (AI gets scoped sub-graph access); plugin trust model (CLAUDE.md #18 — but agents may need richer trust than plugin-DIDs) |
| **UC-02** | Smart-home Atrium: 2-6 humans + 30+ IoT devices + 1-2 AI assistants (Alexa/HomeKit-class) | Mixed: `UserDid` + `DeviceDid` (IoT) + `AgentDid` | 1-2 human admins (parents); device-members have constrained scope | FORK-ONLY (rare); per-device-revoke (frequent for IoT churn) | Privacy-preserving to neighbors; full to family | Long-lived (10+ years) | RestrictedScopeSet (per-device-room sub-graph); UCAN per-device-scope |
| **UC-03** | Shared-workshop Atrium: 5-50 humans (rotating) + shared CNC/3D-printer DeviceMesh + 1 AI "workshop assistant" | Mixed: humans rotate; devices stable; AI stable | Threshold-of-N humans (DAO-lite governance) | AdminKickEpoch + periodic-hygiene quarterly | Full to current members | Long-lived; high member churn | Threshold-admin spec (Sec F-N2-A `threshold_admin: Option<ThresholdAdminSpec>`) |
| **UC-04** | Multiplayer game lobby (ephemeral; 2-16 players + 0-N AI NPCs; ~5min-2hr lifetime) | Mixed: `UserDid` (players) + `AgentDid` (NPCs) | Game-host-as-admin OR no-admin (peer-arbitrated) | NONE (ephemeral; destroyed at game-end) | Public to game observers; private to non-game | Ephemeral (≤ few hours) | Game-state CRDT layer; matchmaking-queue layer |
| **UC-05** | Persistent game world (MMO-class): 1000s of players + admins + moderators + AI NPCs + AI mob-controllers | Mixed: all 3 + `RoleDid` (moderator vs admin vs player) | Multi-admin + moderator-role + NPC-as-non-admin-member | FORK-ONLY (rare); admin-kick (frequent for griefers) | Full to moderators; partial to players | Long-lived (years) | RBAC (per N2 F-N2-A — 3-role insufficient; needs 4-5 roles via codepoint-reserve A3) |
| **UC-06** | Couch co-op (1 device; 2-4 ephemeral player-profiles) | `LocalDevice` + per-session-ephemeral-profile | Device-owner is admin; profiles are session-scoped | NONE at MembershipSet layer (profiles are sub-DeviceMesh entities) | Local-only | Single-session | SingleDevice or DeviceMesh + per-profile-scope via UCAN |
| **UC-07** | Matchmaking queue (pre-match; not yet a game): players sit in queue, get matched, queue-membership destroyed at match-formation | `UserDid` only | No-admin (peer-arbitrated by matchmaker service) | NONE (ephemeral; destroyed at match-form) | Public (queue is open) | Ephemeral (≤ minutes) | NOT a MembershipSet — this is a service-coordinated **pending state**, not a key-sharing set |
| **UC-08** | Tournament bracket (nested sub-brackets; 8-1024 players; time-bound) | `UserDid` + `AgentDid` (AI-judge optional) | Tournament-organizer-admin + bracket-host-moderators | FORK-ONLY between bracket rounds | Public to spectators; private to players | Time-bound (1-day to 1-month) | Composition: Atrium-as-tournament + sub-Atriums-as-brackets via N1 RestrictedScopeSet |
| **UC-09** | Esports clan/team (long-lived; mixed roles: captain, players, coaches, sponsors) | `UserDid` + `AgentDid` (analytics AI optional) | Multi-admin (captains + coaches) | FORK-ONLY (rare); periodic-rotation post-tournament | Full to current members; partial to public sponsors | Long-lived (years) | RBAC (4-role: captain > coach > player > sponsor — needs codepoint-reserve A3) |
| **UC-10** | Federated business: Company-A's Atrium federated with Company-B's Atrium for joint project | Each side homogeneous `UserDid`; federation crosses Kind boundaries | Each side has own admins; federation has shared-admin or no-shared-admin | FORK-ONLY per side; federation-level epoch reset | Cross-org disclosure governed by federation policy | Project-bounded (months to years) | NOT a single MembershipSet — composition of 2 Atriums + cross-set capability delegation (N2 op `cross_set_delegate_capability`, currently P4MC-deferred) |
| **UC-11** | Customer-vendor B2B Atrium: vendor employees + customer employees + AI procurement assistants | Mixed: `UserDid` (both sides) + `AgentDid` | Asymmetric: vendor-admin + customer-admin (parallel governance) | FORK-ONLY; customer-side or vendor-side can fork independently | Per-side-scoped audit; cross-side audit requires both-admin-consent | Contract-bounded | Asymmetric-RBAC (5-role: vendor-admin > vendor-employee > vendor-AI vs customer-admin > customer-employee) |
| **UC-12** | Live broadcast (one publisher; many ephemeral viewers; subscriber churn) | `UserDid` (publisher) + N viewer keys (`PseudonymousDid`?) | Publisher-only-admin | Continuous-rotation (per #54 deferral concern) — viewer-set changes constantly | Public read | Long-lived (publisher); viewer-side ephemeral | NOT a classical MembershipSet — closer to PUB/SUB; subscriber set doesn't need key-share (broadcast is plaintext or per-tier-encrypted) |
| **UC-13** | Public read-only feed (anyone can read; only members write; RSS-class) | Mixed: `UserDid` (writers) + ANYONE (readers) | Writer-admin + reader-anyone | FORK-ONLY for writer set; reader set has NO MembershipSet | Public read | Long-lived | Atrium (write side) + plaintext-or-public-key-encrypted write-content (no membership needed for read) |
| **UC-14** | Time-bound event Atrium (e.g., conference; semester class) | `UserDid` + optional `AgentDid` (course AI) | Multi-admin (organizers) | TIME-BOUND-DESTROY (NEW policy — destroy at expiry; archive cryptographically) | Public to organizers; private to attendees | Time-bound (days to months) | TTL policy field (NEW) — codepoint-reserve candidate |
| **UC-15** | Anonymous/pseudonymous Atrium (zero-knowledge attestation; member identity revealed only on threshold-of-N admin-consent) | `PseudonymousDid` (ZKP-attested) | Threshold-of-N admins for de-anonymization | FORK-ONLY | Strong-privacy: only ZKP-attestation visible | Long-lived | ZKP attestation layer (separate primitive); threshold-admin spec |
| **UC-16** | Threshold DAO Atrium (M-of-N admin actions required for governance) | `UserDid` | M-of-N admins (no single admin) | M-of-N-approved AdminKick (M-of-N threshold required for kick action) | Full to members + on-chain audit | Long-lived | F-N2-A `threshold_admin: ThresholdAdminSpec` + on-chain bridge |
| **UC-17** | Ephemeral collab editing session (Google-Docs-class; 2-100 collaborators; ~mins-hours) | `UserDid` + ephemeral session-key | Document-owner-admin OR no-admin (peer-CRDT) | NONE (ephemeral) | Per-session local; transcript optional | Ephemeral (≤ hours) | NOT a MembershipSet at v1-beta — ephemeral sessions are layered on top via per-session UCAN scope |
| **UC-18** | IoT mesh (autonomous devices; rare human interaction) | `DeviceDid` only | Bootstrapping-user-as-admin OR fully-autonomous-no-admin | Per-device-revoke (continuous) + AdminKickEpoch | Audit to bootstrap-user only | Long-lived | DeviceMesh + extended-DeviceAttestation (autonomous-device sub-class) |
| **UC-19** | Identity-recovery sub-MembershipSet (designated recovery contacts; activated only on user-loss-of-keys) | `UserDid` (recovery contacts) | Subject-user-as-admin (until lost) → recovery-contacts-threshold-of-N | EMERGENCY-ROTATION (key-loss-triggered) | Audit to current recovery set + post-recovery audit | Long-lived; dormant; activated on emergency | NOT a v1-beta primary — this is a layered scheme over Atrium with threshold-admin + emergency-rotation hook (post-v1-beta) |
| **UC-20** | Workflow/queue Atrium: members are workers; admin is the job-queue dispatcher | `UserDid` (workers) + `AgentDid` (queue dispatcher AI) | Single-AI-admin (the dispatcher) | Continuous worker churn; AdminKick frequent | Full audit (Inv-14 attribution per work-item) | Long-lived | Workflow engine layer; AI-as-admin (challenges plugin-DID = bounded-recipient claim) |
| **UC-21** | Cross-chain DAO (member-set spans Ethereum + Solana + Bitcoin) | Multi-chain DIDs (NEW; `did:eth:` + `did:sol:` + `did:btc:`) | Multi-chain governance | M-of-N cross-chain consensus | Public on-chain audit | Long-lived | Bridge layer (separate primitive) + Atrium with multi-chain DID variants |
| **UC-22** | Friend-of-friend Atrium (membership inferred from social-graph; not explicit add) | `UserDid` with social-graph proof | Bootstrap-user as admin; FoF auto-admit | Continuous (social-graph changes) | Privacy-preserving (FoF discovery only) | Long-lived | Social-graph primitive (NEW) + Atrium with auto-admit policy |

### §1.2 Axis observations

**Member-key shape diversity.** Of the 22 use-cases, **13** want a member-key
shape NOT covered by M-CONS-v2's `MemberKey { UserDid | DeviceDid |
LocalDevice }` — they want `AgentDid` (AI assistant), `PseudonymousDid`
(ZKP-attested), `RoleDid` (capability-only), multi-chain DIDs, or ephemeral
session-keys. **BUT** — and this is the key observation — **most of these
"new MemberKey variants" decompose** into existing engine primitives:

- `AgentDid` for AI-assistants → ALREADY-covered by the **`UserDid` variant**
  if we treat the AI agent as a first-class autonomous actor (per W3C Agent
  Identity Registry + draft-singla-agent-identity-protocol-00; AI agents get
  DIDs of the same form as humans). The plugin-DID trust model (CLAUDE.md
  #18) applies to BOUNDED plugins (loaded code with capabilities); an
  AI-assistant with its own DID + autonomous-actor-status is different.
  See §5 detailed analysis.
- `PseudonymousDid` → DID method extension (`did:zkp:...`); MembershipSet
  doesn't need a new MemberKey variant — `UserDid` already abstracts over
  DID methods per N2 §1.4 + `benten-id` multikey support per F15.
- `RoleDid` (capability-only identity) → handled by the RBAC slot (F-N2-A);
  no new MemberKey shape needed.
- multi-chain DIDs → DID method extension; same as `PseudonymousDid`.
- ephemeral session-keys → NOT a MembershipSet member; ephemeral sessions
  are a different primitive class.

**Authority shape diversity.** Of the 22 use-cases, **18** are covered by the
N2 `authorities: BTreeSet<Authority>` slot + 3-role RBAC + optional
`threshold_admin: ThresholdAdminSpec`. The remaining 4 (UC-12 publisher-only;
UC-15 ZKP-threshold; UC-21 cross-chain consensus; UC-22 auto-admit) need
extensions that are EITHER application-layer (UC-12: just designate the
publisher as the sole admin; trivial) OR codepoint-reserve future-Kind (UC-15:
`MembershipSetKind::AnonymousAttested`).

**Rotation policy diversity.** **The biggest gap.** M-CONS-v2 has 2 rotation
policies (`ForkOnly | AdminKickEpoch` per N3) folded into FORK-ONLY at
v1-beta. **9 of the 22 use-cases want a DIFFERENT rotation pattern:**
ephemeral-destroy (UC-04, UC-07, UC-17); continuous (UC-12, UC-18, UC-20);
time-bound-destroy (UC-14); emergency-rotation (UC-19); M-of-N-approved-kick
(UC-16). **Resolution:** all 9 can be handled by extending the
**`RotationTrigger` doc-enum** (N3) at v1-beta with additive codepoint slots,
WITHOUT changing the FORK-ONLY wire-format mechanism. The rotation TRIGGER
is documented; the rotation MECHANISM stays single. (This validates N3's
doc-enum decision.)

**Lifecycle diversity.** M-CONS-v2 has NO explicit lifecycle policy
(implicit "long-lived" default). **6 of the 22 use-cases want ephemeral or
time-bound** (UC-04, UC-07, UC-14, UC-17, UC-19 dormant). **Resolution:**
add a `lifecycle: LifecyclePolicy` codepoint-reserve sub-slot in
`MembershipSetPolicy` for post-v1-beta (NOT v1-beta-LB). Document the
default at `Long-lived`; defer ephemeral/time-bound to post-v1-beta with
HALT-AND-SURFACE-TO-BEN gate.

**Audit semantics diversity.** Covered by the existing
`audit_log_query` op (F-N2-B) + per-Kind audit-event-shape table (M4 BREAK 6
amendment + M-C2-v2 MCV2-B-3 network-observer-scoping). No new shape needed
beyond what M-CONS-v2 already specifies.

**Compose-with diversity.** **Most use-cases compose with existing engine
primitives via Atrium + UCAN-scope + RestrictedScopeSet + plugin-DID-fleet
or DeviceMesh-fleet** at application layer. Only **4 use-cases** require a
genuinely new Kind shape: UC-04 (`EphemeralLobby` for game lobbies), UC-15
(`AnonymousAttested` for ZKP-membership), UC-14 (`TimeBoundEvent` for
expiring sets), UC-21 (`CrossChain` for multi-chain DAOs). All 4 are
post-v1-beta candidates.

---

## §2 Task 2 — Identify which edge-cases break the 3-Kind enum

### §2.1 Bucket disposition per use-case

**Disposition key:**
- **(D)** = **DECOMPOSES via application-layer composition** — current
  3-Kind enum sufficient; use-case implemented as application-layer pattern
  over existing primitives.
- **(R)** = **CODEPOINT-RESERVE SLOT** — needs reserved wire codepoint at
  v1-beta + body-impl post-v1-beta (HALT-AND-SURFACE-TO-BEN per §15.c).
- **(F)** = **FRESH-PRIMITIVE-CLASS** — NOT a MembershipSet; needs a different
  engine primitive (or doesn't need an engine primitive at all).

| # | Use-case | Disposition | Notes |
|---|---|---|---|
| UC-01 | Hybrid Atrium-Garden-Grove (Ben's hint) | **D + R** | DECOMPOSES at v1-beta as `Atrium` + AI-as-`UserDid`-with-`AgentDid`-DID-method; codepoint-reserve `Garden`/`Grove` for future explicit-Kind shape if Ben wants distinct ratification |
| UC-02 | Smart-home Atrium | **D** | `Atrium` + per-device-RestrictedScopeSet via N1; AI as `UserDid` |
| UC-03 | Shared-workshop Atrium | **D** | `Atrium` + `threshold_admin: ThresholdAdminSpec` per N2 |
| UC-04 | Multiplayer game lobby (ephemeral) | **R** | Reserve `EphemeralLobby` codepoint; v1-beta apps use `Atrium` with manual destroy-at-end |
| UC-05 | Persistent game world (MMO) | **D + extends-RBAC** | `Atrium` + extended-RBAC (4-5 roles via N2 §A3 codepoint-reserve) |
| UC-06 | Couch co-op | **D** | `SingleDevice` + per-profile UCAN scope |
| UC-07 | Matchmaking queue | **F** | NOT a MembershipSet — service-coordinated pending state; no key-sharing |
| UC-08 | Tournament bracket | **D** | `Atrium` + sub-`Atrium` via N1 RestrictedScopeSet + composition |
| UC-09 | Esports clan/team | **D + extends-RBAC** | Same as UC-05; 4-role RBAC via codepoint-reserve |
| UC-10 | Federated business | **D** | Composition of 2 `Atrium`s + N2 `cross_set_delegate_capability` (P4MC) |
| UC-11 | Customer-vendor B2B | **D** | Same as UC-10 + asymmetric-RBAC (codepoint-reserve) |
| UC-12 | Live broadcast | **F** | NOT a MembershipSet — PUB/SUB; subscriber-set isn't key-shared |
| UC-13 | Public read-only feed | **D** | `Atrium` (writers) + public-read (no membership for read) |
| UC-14 | Time-bound event Atrium | **R** | Reserve `TimeBoundEvent` codepoint OR add `lifecycle` sub-slot |
| UC-15 | Anonymous/pseudonymous Atrium | **R** | Reserve `AnonymousAttested` codepoint; v1-beta uses `Atrium` + ZKP-DID-method |
| UC-16 | Threshold DAO Atrium | **D** | `Atrium` + N2 `threshold_admin` (v1-beta-LB) |
| UC-17 | Ephemeral collab editing | **F** | NOT a v1-beta MembershipSet — layered ephemeral session |
| UC-18 | IoT mesh (autonomous) | **D** | `DeviceMesh` + DeviceAttestation extension for autonomous-device sub-class |
| UC-19 | Identity-recovery sub-MembershipSet | **R** | Reserve `RecoveryContacts` codepoint; v1-beta uses `Atrium` + threshold-admin |
| UC-20 | Workflow/queue Atrium | **D** | `Atrium` + AI-as-admin (plugin-DID extension; CLAUDE.md #18 trust model needs slight extension — see §5) |
| UC-21 | Cross-chain DAO | **R** | Reserve `CrossChain` codepoint; v1-beta uses `Atrium` + multi-chain-DID-method |
| UC-22 | Friend-of-friend Atrium | **F** | NOT a v1-beta MembershipSet — social-graph primitive (separate) |

### §2.2 Tally

- **(D) DECOMPOSES via application-layer composition:** **13** of 22 (59%).
- **(R) CODEPOINT-RESERVE SLOT for post-v1-beta:** **5** of 22 (23%).
  Specifically: `EphemeralLobby` (UC-04), `TimeBoundEvent` (UC-14),
  `AnonymousAttested` (UC-15), `RecoveryContacts` (UC-19), `CrossChain`
  (UC-21).
- **(F) FRESH-PRIMITIVE-CLASS — not a MembershipSet:** **4** of 22 (18%).
  Specifically: matchmaking queue (UC-07; service state); live broadcast
  (UC-12; PUB/SUB); ephemeral collab editing session (UC-17; session layer);
  friend-of-friend (UC-22; social-graph layer).

### §2.3 Key observation — the 3-Kind enum is NOT the bottleneck

**The bottleneck for "future flexibility" is NOT the
3-Kind-vs-N-Kind question.** It's:

1. **Member-key diversity** — handled cleanly by DID method extension
   (`UserDid` abstracts; `benten-id` multikey supports new methods
   additively per F15).
2. **Authority + RBAC extension** — handled cleanly by N2's
   `authorities: BTreeSet<Authority>` slot + role codepoint-reserve A3.
3. **Lifecycle + rotation diversity** — handled cleanly by additive
   policy-field codepoint-reserves.

**The Kind enum only matters when the use-case has DIFFERENT INVARIANT
SEMANTICS that need different per-Kind constructor validators.** Of the 22
use-cases, only **5** have such genuinely-different invariants (the 5 R-bucket
codepoint-reserve candidates). All 5 are post-v1-beta candidates per Ben's
ratified scope discipline + §15.c HALT-AND-SURFACE.

### §2.4 What WOULD break the 3-Kind enum at v1-beta

I genuinely tried to construct a v1-beta use-case that breaks the 3-Kind
enum. **The closest cases are:**

- **UC-04 game lobby at v1-beta.** A game studio shipping at v1-beta wants
  ephemeral lobby semantics. **Workaround:** use `Atrium` with manual
  destroy-at-game-end + 1-hour TTL. **Verdict:** not breaking — workaround
  cost is ~1 application-layer line of code per game.
- **UC-15 anonymous Atrium at v1-beta.** A privacy-focused app at v1-beta
  wants ZKP-attested anonymous members. **Workaround:** use `Atrium` with
  `did:zkp:...` DID method extension; engine doesn't know the DID is
  ZKP-attested; app layer enforces. **Verdict:** workable but leaves the
  anonymous-attestation discipline at application layer (auditable risk).
- **UC-19 identity-recovery at v1-beta.** A wallet-app wants designated
  recovery contacts. **Workaround:** use `Atrium` with `threshold_admin`
  + emergency-rotation app-layer pattern. **Verdict:** workable; the
  recovery flow is application-layer-orchestrated.

**None of these are v1-beta blockers.** All have application-layer
workarounds that are NOT brittle (they don't violate any v1-beta invariant;
they just don't get a dedicated wire codepoint signaling "this is THE
anonymous Atrium" etc).

---

## §3 Task 3 — Propose the elegant shape

### §3.1 Options A-E re-cast

I re-cast the 5 options from the dispatch brief, then evaluate each against
the 22 use-cases + the elegant-permanent-shape reflection-pass discipline.

**Option A — Keep 3-arm enum + grow as new Kinds emerge.** Static at v1-beta;
§15.c HALT-AND-SURFACE on every new addition; no codepoint-reserve.

**Option B — Extensible Kind via codepoint-reserve at v1-beta.** Active enum
stays 3-arm; codepoint-reserve table reserves slots for N future Kinds; impl
ships with 3 + others additive.

**Option C — Collapse to generic `MembershipSet` with per-set runtime Kind
tag.** No `enum MembershipSetKind`; instead a runtime `kind_tag: u8` field;
loses compile-time invariants.

**Option D — Trait-based abstraction with associated types.** `trait
MembershipSetKind { type Authority; type MemberKey; type Policy; ... }`;
typed-variants implement trait; M-C1 stopped-mid-run finding apparently
mentioned this; rejected at M-CONS-v1 already (see M-C1 history).

**Option E — Hybrid: typed-base-set { Atrium, DeviceMesh, SingleDevice } +
extension via `Kind::Custom(CustomKindCid)`.**

**Option E-PRIME (NEW — my proposed winner) — Hybrid: typed-base-set
{ Atrium, DeviceMesh, SingleDevice } EXACTLY-3 + CODEPOINT-RESERVE TABLE for
5-6 named future Kinds + Application-layer composition default + §15.c
HALT-AND-SURFACE-TO-BEN gate retained.** Concretely:
- Active `enum MembershipSetKind` = 3 arms (unchanged from N2 / M-CONS-v2).
- `CRYPTO-CODEPOINTS.md` codepoint family `MEMBERSHIP_SET_KIND` reserves 6
  numbered codepoints for named future Kinds: `AtriumWithRotatingGroupKey`
  (already reserved per N4 R-N4-1) + `Garden` + `Grove` + `EphemeralLobby` +
  `Federation` + `Custom(CustomKindCid)`. (`Custom` is a wildcard escape;
  see §3.4.)
- Each reserved codepoint at v1-beta: engine refuses with
  `E_MEMBERSHIP_SET_KIND_UNSUPPORTED` (per M-CONS-v2 #53 narrowing pattern;
  same as Willow / iroh-roq codepoints).
- Wire-format pin: codepoint family + EnvelopeShape codepoint axis (per F11
  experimental-range) + `MembershipSetKindCodepoint: u16` field within the
  envelope. Active enum maps {Atrium=1, DeviceMesh=2, SingleDevice=3,
  AtriumWithRotatingGroupKey=4, Garden=5, Grove=6, EphemeralLobby=7,
  Federation=8, Custom=9, ...experimental-range 0xFF00-0xFFFF}.
- §15.c invariant: enum EXACTLY-3-arm at active surface; codepoint-reserve
  table additive-extensible without active-enum amendment; body-impl for
  each reserved codepoint requires HALT-AND-SURFACE-TO-BEN per existing
  §15.c gate.
- MembershipSet-Spec.md §-add "Composition recipes" naming canonical
  application-layer patterns for the 13 (D)-bucket use-cases (Atrium +
  AI-assistant-as-`UserDid`; Atrium + per-device-RestrictedScopeSet for
  smart home; etc).

### §3.2 Evaluation table — Options vs 22 use-cases vs elegant-shape criteria

| Criterion | Opt A (static 3-arm) | Opt B (reserve-table) | Opt C (runtime tag) | Opt D (trait) | Opt E (Custom CID) | **Opt E-PRIME (winner)** |
|---|---|---|---|---|---|---|
| (D)-bucket coverage (13 use-cases) | 13/13 (composition works) | 13/13 | 13/13 | 13/13 | 13/13 | **13/13** |
| (R)-bucket coverage (5 use-cases) | 0/5 at v1-beta (need new arm + ratification per HALT-AND-SURFACE; each adds 4-8 weeks Ben-decision-cycle) | **5/5 via reserved codepoints (wire-additive; body-impl post-v1-beta)** | 5/5 (just add tag; no rigor) | 5/5 (trait extends) | 5/5 (Custom CID + CustomKind metadata) | **5/5 via reserved codepoints (same as Opt B)** |
| (F)-bucket coverage (4 use-cases) | N/A (correctly excluded) | N/A | N/A | N/A | N/A | **N/A (correctly excluded)** |
| Compile-time type-rigor (per-Kind constructor invariants) | HIGH | HIGH | LOW (runtime only) | HIGH | MED (Custom CID is opaque) | **HIGH (same as N2 hybrid)** |
| Wire-format extensibility | LOW (each addition is wire-affecting) | **HIGH (additive)** | HIGH (but loses rigor) | MED | HIGH | **HIGH (additive; same as Opt B)** |
| Compatible with §15.c HALT-AND-SURFACE | YES | YES (gate retained on body-impl) | NO (defeats gate) | YES | YES | **YES (gate retained)** |
| Compatible with N2 HYBRID-RECOMMENDED | YES | YES | NO (overrides N2) | NO (overrides N2 — trait-based is M-C1's rejected alternative) | YES | **YES (extends N2 additively)** |
| Compatible with F9/F10 `#[non_exhaustive]` pattern in codebase | NO (3-arm exhaustive) | **YES (mirrors F9/F10 codepoint-reserve)** | NO | NO | YES | **YES (mirrors F9/F10)** |
| Compatible with N4 R-N4-1 `AtriumWithRotatingGroupKey` precedent | NO (would require new arm) | **YES (already-reserved follows this pattern)** | YES | NO | YES | **YES (extends N4 R-N4-1 precedent)** |
| Cost at v1-beta (wave-days) | ~0 (no change) | ~0.3-0.5 | ~3-5 (re-architecture) | ~5-8 (re-architecture) | ~0.5-1 | **~0.3-0.5** |
| "Overly restrictive" closure (Ben's framing) | NO (still restrictive) | **YES (5+ codepoints reserve)** | YES (but loses rigor) | YES (but high cost) | YES | **YES (5+ codepoints; with elegant existing-pattern mirroring)** |
| Engine-primitives-vs-application-layer alignment | High | High | LOW (encourages putting policy in engine) | LOW (trait surface invites engine-extension) | High | **HIGHEST (composition recipes §-add makes app-layer-first EXPLICIT)** |
| HARD RULE 12 disposition for unselected (D)-bucket use-cases | OUT-OF-SCOPE (composition is correct) | OUT-OF-SCOPE (composition is correct) | OUT-OF-SCOPE | OUT-OF-SCOPE | OUT-OF-SCOPE | **OUT-OF-SCOPE for 13 (D); BELONGS-NAMED-NOW for 5 (R) → V1-FROZEN-INTERFACE-DEFERRED.md rows pre-mounted** |

### §3.3 Winner — Option E-PRIME

**Option E-PRIME wins** on every criterion against the 22 use-cases + the
elegant-shape reflection-pass. Specifically:

- **Strictly less code than Options C/D** at v1-beta (~0.3-0.5 wave-days
  vs ~3-8 wave-days).
- **Closes the "overly restrictive" concern explicitly** by reserving 5+
  named codepoints (matches Ben's foresight intuition; surfaces the
  reservation visibly in the v1-beta spec).
- **Forward-class-of-bug closure:** the 5 reserved codepoints pre-empt the
  "we need to add a 4th Kind in 6 months" scenario, which would otherwise
  require a wire-format change + §15.c HALT-AND-SURFACE Ben-decision-cycle.
  Reserving the codepoint slot at v1-beta means future body-impl is
  pure-additive (no wire-format break).
- **Mirrors existing codebase pattern.** F9/F10 already use
  `#[non_exhaustive]` + codepoint-reserve. N4 R-N4-1 already reserves
  `AtriumWithRotatingGroupKey`. F11 already has experimental-codepoint-range.
  Option E-PRIME is "do MORE of what's already-working" — zero pattern-debt.
- **Preserves M-CONS-v2 N2 HYBRID-RECOMMENDED win.** Compile-time rigor on
  the 3 active Kinds; runtime-validated invariants per constructor; the
  reserved codepoints don't have active Rust enum variants until body-impl
  ratification.
- **Application-layer composition is the DEFAULT** for the 13 (D)-bucket
  use-cases. The "Composition recipes" §-add in MembershipSet-Spec.md
  makes this EXPLICIT (closes the engine-primitives-vs-application-layer
  feedback rule — pushes application-layer composition before engine
  extension).

### §3.4 The `Custom(CustomKindCid)` wildcard slot — DEFER

I considered including `Kind::Custom(CustomKindCid)` as a wildcard escape
hatch (Opt E original). **DEFER — DISAGREE-WITH-EXPLANATION at v1-beta.**

**Reason.** A wildcard `Custom` codepoint that points to a CID-named "Custom
Kind spec" runs against the §15.c HALT-AND-SURFACE-TO-BEN discipline by
allowing applications to introduce arbitrary new Kinds without Ben review.
It also runs against the engine-primitives-vs-application-layer feedback by
inviting application-defined Kind shapes into engine.

**Resolution.** RESERVE 5 NAMED codepoints (`AtriumWithRotatingGroupKey` +
`Garden` + `Grove` + `EphemeralLobby` + `Federation`) at v1-beta. Each named
codepoint has a `MembershipSet-Spec.md` row describing the intended use-case
+ a `V1-FROZEN-INTERFACE-DEFERRED.md` row pre-mounted with body-impl ETA +
ratification trigger. Wildcards/Custom are NOT reserved at v1-beta.
**Add `Custom(CustomKindCid)` to the post-v1-beta watch-list** as a
NAMED-DEFERRED row pointing at `V1-FROZEN-INTERFACE-DEFERRED.md` Row
D-CUSTOM-KIND with revisit-trigger = "≥3 application-layer requests for
custom Kinds in production".

### §3.5 HARD RULE 12 disposition for unselected options

| Option | Disposition | Justification |
|---|---|---|
| Option A (static 3-arm) | **DISAGREE-WITH-EXPLANATION** | Forces wire-format-affecting + 4-8-week Ben-decision-cycle per new Kind; doesn't pre-empt foreseen extensions; Ben's "overly restrictive" concern unaddressed |
| Option B (just codepoint-reserve; no recipes §-add) | **BELONGS-NAMED-NOW** | Subset of Opt E-PRIME; the recipes §-add is the cheap differentiator that closes engine-vs-app-layer alignment |
| Option C (runtime tag) | **DISAGREE-WITH-EXPLANATION** | Loses N2 HYBRID-RECOMMENDED compile-time rigor; defeats §15.c gate; high re-architecture cost; the runtime-tag isn't actually more flexible (it just moves invariant-checking from constructor to op-handler at higher per-op cost) |
| Option D (trait-based) | **DISAGREE-WITH-EXPLANATION** | M-C1 stopped-mid-run already noted this rejection; trait-with-associated-types makes Authority/MemberKey/Policy generic over Kind, which (1) breaks the homogeneous-MemberKey-per-Kind invariant M4 BREAK 5 ratified, (2) invites application-layer trait impls (engine-extension pattern), (3) high re-architecture cost (~5-8 wave-days), (4) trait-object-dispatch complicates wire-format encoding (Trait-object doesn't have wire codepoint stability) |
| Option E (Custom(CustomKindCid)) | **DEFER — DISAGREE-WITH-EXPLANATION at v1-beta** | Wildcard escape defeats §15.c gate; named-codepoint-reserves are stronger discipline; revisit-trigger named on V1-FROZEN-INTERFACE-DEFERRED.md Row D-CUSTOM-KIND |

### §3.6 The elegant-shape reflection pass — does Option E-PRIME close N
findings at once?

Per `feedback_extra_reflection_pass_for_elegant_permanent_shape.md`: "is
there a single elegant structural shape closing N findings at once?"
**YES.** Option E-PRIME closes:

1. Ben's "overly restrictive" concern (motivates this whole audit).
2. 5 R-bucket future use-cases (UC-04, UC-14, UC-15, UC-19, UC-21).
3. The existing N4 R-N4-1 `AtriumWithRotatingGroupKey` reservation gets
   absorbed into the same reservation table (less ad-hoc).
4. The implicit pressure from M-C2-v2 §5 MCV2-B-4 "3-way composition matrix"
   gets a clean home (each reserved codepoint has its own row).
5. The engine-primitives-vs-application-layer feedback rule (P12: "already-
   shipping combinator surfaces = cheapest grant-layer expressiveness") —
   the recipes §-add makes app-layer-first explicit.
6. The §15.c discipline rigor (each reserved codepoint still requires
   HALT-AND-SURFACE-TO-BEN for body-impl).
7. Pattern consolidation: F9/F10 `#[non_exhaustive]` + N4 codepoint-reserve
   + F11 experimental-range + Option E-PRIME = SINGLE unified
   codepoint-reserve discipline across the wire-format.

This is a 7-finding-elegant-shape-closure with **strictly less code than
any of Opts A/C/D**. Confidence HIGH on the elegant-shape verdict.

---

## §4 Task 4 — Game backend specifically

### §4.1 Game backend MembershipSet shape requirements

Per dispatch brief examples + web research:

| Game pattern | Membership shape needed | Lifecycle | Existing 3-Kind fit |
|---|---|---|---|
| Multiplayer game lobby (ephemeral; 2-16 players + 0-N AI NPCs; chat + state-sync) | Mixed `UserDid` + `AgentDid` | Ephemeral (~5min-2hr) | **Workable via `Atrium`** with manual destroy-at-end; codepoint-reserve `EphemeralLobby` post-v1-beta for explicit lifecycle |
| Persistent world (1000s of players + admins + moderators + AI NPCs + AI mob-controllers) | Mixed `UserDid` + `AgentDid` + 4-5 roles | Long-lived (years) | **Workable via `Atrium`** + extended-RBAC (codepoint-reserve A3 per N2 §A3) |
| Tournament bracket (time-bound; nested sub-brackets) | `UserDid` + `AgentDid` (AI-judge) | Time-bound (1-day to 1-month) | **Workable via `Atrium`** + N1 sub-`Atrium` via RestrictedScopeSet composition |
| Couch co-op (1 device; multiple ephemeral player-profiles) | `LocalDevice` + per-session profile | Single-session | **Workable via `SingleDevice`** + per-profile UCAN scope |
| Matchmaking queue (ephemeral; pre-match state) | `UserDid` only | Ephemeral (~minutes) | **NOT a MembershipSet** — service-coordinated pending state (per UC-07 disposition) |
| Esports clan/team (long-lived; mixed roles: captain, players, coaches, sponsors) | `UserDid` + `AgentDid` + 4-role RBAC | Long-lived | **Workable via `Atrium`** + extended-RBAC |
| AI-as-player NPC integration | `AgentDid` (= `UserDid` with `did:agent:...` method) | Per-game lifetime | **Workable via `Atrium`** + AgentDid-as-UserDid pattern (see §5) |

### §4.2 Game backend disposition summary

- **6 of 7 game patterns DECOMPOSE via existing `Atrium` / `DeviceMesh` /
  `SingleDevice`** + composition/RBAC extension.
- **1 pattern (`EphemeralLobby`)** gets a codepoint-reserve slot for
  post-v1-beta body-impl that captures the ephemeral-destroy lifecycle
  explicitly.
- **1 pattern (matchmaking queue) is NOT a MembershipSet** — it's a
  pre-membership service-coordinated state; the engine's MembershipSet
  primitive isn't the right tool.

### §4.3 Game backend recommendation

**For game studios building on Benten at v1-beta:**

1. **Use `Atrium` for any multi-player game** (lobby, persistent world,
   tournament, esports). Destroy or archive at game-end via application-layer
   pattern.
2. **Use `SingleDevice` for couch co-op or local-only games.**
3. **Reserve `EphemeralLobby` codepoint** for the future body-impl when
   game-state requires explicit ephemeral-lifecycle invariants (e.g.,
   "K_Set MUST be destroyed at game-end; no archive"). Post-v1-beta.
4. **Add "Game backend recipes" §-row** to MembershipSet-Spec.md with the 7
   patterns above + their composition recipes (canonical examples for
   game-studio integrators).
5. **AI-NPC integration** uses the `AgentDid` pattern (see §5).

### §4.4 The matchmaking-queue caveat

Matchmaking queue is genuinely NOT a MembershipSet. It's a pending-state
that becomes a MembershipSet (via match-formation) — analogous to how iroh-
gossip topic-bootstrapping isn't a MembershipSet but is a transport layer
that delivers the MembershipSet K_Set. Application-layer pattern (the game
studio's matchmaker service) handles the pending state; MembershipSet
materializes only at match-formation.

This is consistent with the engine-primitives-vs-application-layer feedback
rule and should be documented as a NAMED-NON-MEMBERSHIP-SET-USE-CASE row
in MembershipSet-Spec.md (alongside live-broadcast UC-12, ephemeral-collab
UC-17, friend-of-friend UC-22).

---

## §5 Task 5 — Hybrid AI-agent membership specifically

### §5.1 Ben's hint restated

Ben's exact framing: *"while we have the 3 kinds, I don't want to be overly
restrictive because I could imagine a future where we have an atrium/garden/
grove with many users but also some devices connected as the default
compute/workflow executors/'members' that are fully AI assistants within
that community."*

Decomposed:
1. **Atrium/Garden/Grove** — multi-tier community shapes (Garden bigger than
   Atrium; Grove bigger than Garden; per `docs/GLOSSARY.md`).
2. **Many users** — humans (current `UserDid`).
3. **Some devices** — IoT-class compute that aren't user-controlled
   (current `DeviceDid` extends).
4. **Default compute/workflow executors** — automation/workflow agents that
   do work in the community.
5. **Fully AI assistants** — autonomous AI actors with their own identity.

### §5.2 Design question 1 — AI agent has a DID? Or capability-only identity?

**Recommendation: DID-based.**

Per the web search results: the W3C Agent Identity Registry Protocol
Community Group + draft-singla-agent-identity-protocol-00 (AIP) +
`arxiv.org/abs/2511.02841` (AI Agents with DIDs and VCs) all converge on
**AI agents get full first-class DIDs** + Verifiable Credentials. This is
the emerging standard.

In Benten terms: introduce a `did:agent:...` DID method (sibling to existing
`did:key:` / `did:web:` / etc supported per F15 multikey). AI agents are
admitted to `MembershipSet.members` as `MemberKey::UserDid(AgentDid)`
where `AgentDid` is just a `Did` with the `did:agent:` method prefix.

**Why DID and not capability-only:** capability-only (plugin-DID per
CLAUDE.md #18) is the right shape for BOUNDED PLUGINS (code loaded with
specific capabilities). An AI assistant that:
- has its own conversational context;
- holds long-term memory;
- accumulates reputation;
- can sign messages with its own keys;
- can hold capabilities and re-delegate;
- can be revoked/rotated independent of any user;

is **NOT a plugin** — it's an autonomous actor. Plugin-DID is the wrong
trust model. The right model is "AI agent = first-class actor with DID";
its capabilities are bounded by UCAN/RBAC just like any user.

### §5.3 Design question 2 — AI agent has K_Set decryption ability? Or capability-delegated by user?

**Recommendation: K_Set decryption ability when admitted as a member.**

If an AI assistant is admitted to an `Atrium` as a member (with its own
`AgentDid`), it gets the same K_Set decryption ability as any other member.
This is the homogeneous-MemberKey-per-Kind invariant (M4 BREAK 5; F-N2-A).
The AI agent's encryption-pubkey is what receives the HPKE-wrap of K_Set.

If instead the AI assistant operates as a **delegated capability holder**
(user "loans" capabilities to AI to do a specific task), the user remains
the K_Set-decrypting member; AI gets UCAN-delegated capabilities to specific
resources but NOT K_Set itself. This is the "agent acts on behalf of user"
pattern.

**Both patterns are valid; the choice is application-layer.** MembershipSet
supports the first (AI-as-member); UCAN delegation supports the second
(AI-as-delegate).

**Composition recipe:** "AI assistant as Atrium member" =
`Atrium { members: [user_did_A, user_did_B, agent_did_C], authorities: { user_did_A, user_did_B }, role_assignments: { user_did_A → Admin, user_did_B → Admin, agent_did_C → Member } }`. AI agent has Member role (read+write to Atrium content);
no admin rights (humans can kick the AI; AI can't kick humans).

### §5.4 Design question 3 — AI agent as "ambient compute" with no human-tier member-shape

**This is Ben's specific hint.** "fully AI assistants within that community"
as "default compute/workflow executors/'members'."

**Resolution: ambient-compute AI is just an admission to the Atrium with
Member role + UCAN scope restrictions.** Specifically:

- AI agent has `AgentDid` (full first-class DID).
- AI agent is admitted to Atrium as `MemberKey::UserDid(AgentDid)` (the
  Atrium MemberKey variant accepts any `Did` whose method is in
  `benten-id` multikey allowlist).
- AI agent has `RoleId::Member` (not Admin).
- AI agent has UCAN scope LIMITED to "execute workflows" (the
  `PermissionOperation::ExecuteWorkflow` variant per F20) + RestrictedScopeSet
  granting access only to "workflow-input" sub-graphs.
- AI agent can NOT kick members, fork the Atrium, or change policy.

**This works at v1-beta with ZERO new wire-format additions.** The DID
method extension (`did:agent:`) is application-layer + `benten-id` multikey
trust-allowlist; the UCAN + RestrictedScopeSet scoping is N1 + F20 v1-beta-LB.

### §5.5 Composes with CLAUDE.md #18 plugin trust model

Per `feedback_plugin_extension_trust_model.md`: two-category extensibility
(app-level subgraph plugins with three-layer consent vs engine-level Rust
extensions with compile-time trust).

**AI-agent-as-member is a THIRD trust category beyond plugin + engine-
extension.** It's the **autonomous-actor-with-DID category**. The trust
relationship is:
- User admits AI agent to Atrium (explicit consent at admission time).
- AI agent has its own DID + signing keys (no user-controlled key sharing).
- AI agent can be kicked by user (admin op).
- AI agent's capabilities are bounded by Role (Member) + UCAN delegation
  (just like any other member).

**Key distinction from plugin-DID:**
- Plugin-DID = code loaded into a host process with capabilities the host
  grants. Identity is owned by the host. Lifetime is per-host-session.
- Agent-DID = autonomous actor with own keys, own memory, own reputation.
  Identity is owned by the agent (or its operator org). Lifetime is
  long-term.

**The plugin-DID trust model is unchanged.** Plugins remain bounded-
capability recipients (NOT members; per M4 BREAK 5 / §11.5 amendment).
Agent-DIDs are first-class members.

### §5.6 Refining M4 BREAK 5

M4 BREAK 5 stated: "Plugin-DIDs and agent-DIDs are NOT members." The
plugin-DID part holds. The **agent-DID part needs refinement**:

> **M4 BREAK 5 (refined):** *Plugin-DIDs (CLAUDE.md #18 bounded-capability
> recipients) are NOT members. Agent-DIDs (autonomous AI actors with
> first-class DIDs in `did:agent:` method) ARE members, admitted to
> `Atrium` like any other `UserDid` member, subject to UCAN + RBAC
> scoping.*

This refinement is bounded and consistent with M4 §11.5's
homogeneous-per-Kind invariant: `Atrium.members` is still homogeneously
`MemberKey::UserDid(_)`-typed; the `_` just expands the allowed DID methods
to include `did:agent:`.

### §5.7 New compromise candidate — Compromise #57

**Compromise #57 (NEW; recommended mint).** **AI-agent-as-member trust
category** — Agent-DIDs are admitted to Atrium as first-class members
distinct from plugin-DIDs (which remain bounded-capability recipients per
CLAUDE.md #18). The trust model is "user admits autonomous agent at
admission time; agent has own keys + own DID; agent capabilities bounded by
Role + UCAN; agent can be kicked." This refines M4 BREAK 5 + extends the
CLAUDE.md #18 two-category trust model to three-category. **Application
discipline:** apps integrating AI agents must distinguish bounded plugins
(which load code; trust-via-capability-grant) from autonomous agents
(which act independently; trust-via-DID-membership-admission). MembershipSet
documents this distinction in MembershipSet-Spec.md §-add "AI agent
integration"; engine does NOT distinguish at wire level (an agent-DID is
just a DID); application layer enforces.

Confidence: HIGH on the directional refinement; MED-HIGH on the specific
shape (Ben may want to introduce `did:agent:` as an explicit DID method
codepoint vs leave it as application-layer convention).

### §5.8 Hybrid use-case UC-01 resolution

Ben's UC-01 ("hybrid Atrium-Garden-Grove with humans + devices + AI
assistants"):
- **At v1-beta:** use `Atrium` with `did:agent:` AgentDids admitted as
  `UserDid`-variant Member-role members; per-AI RestrictedScopeSet scopes
  + UCAN-ExecuteWorkflow permission. ZERO new wire-format. Composition
  recipe documented.
- **Post-v1-beta:** if Ben wants explicit `Garden` / `Grove` semantics
  (e.g., Garden = Atrium-of-Atriums via cross-set delegation; Grove = even
  larger composition), the reserved codepoints `Garden` + `Grove` are
  there. HALT-AND-SURFACE-TO-BEN body-impl ratification.

---

## §6 Task 6 — v1-beta wire-format pin

### §6.1 Wire-format design under Option E-PRIME

**`CRYPTO-CODEPOINTS.md` codepoint family `MEMBERSHIP_SET_KIND` (NEW
M-CONS-v2 addition under Option E-PRIME):**

```
# MEMBERSHIP_SET_KIND codepoint family
# u16 codepoints; IANA-disjoint per F8

Active (Rust enum + TS mirror):
  0x0001  Atrium
  0x0002  DeviceMesh
  0x0003  SingleDevice

Codepoint-reserve (slot reserved; engine refuses with
                   E_MEMBERSHIP_SET_KIND_UNSUPPORTED at v1-beta):
  0x0004  AtriumWithRotatingGroupKey     (from N4 R-N4-1; absorbs prior reserve)
  0x0005  Garden
  0x0006  Grove
  0x0007  EphemeralLobby
  0x0008  Federation

Experimental range:
  0xFF00-0xFFFF  Experimental MembershipSet kinds (per F11 escape codepoint
                discipline; application-layer experimentation; NOT for
                production use)
```

**Wire-format slot in envelopes:**

The `MEMBERSHIP_SET_ENCRYPTION = 0x6380` codepoint family already carries a
`MembershipSetKind: u8` field per M-CONS-v2 §3 F1 + F4. **Widen this to
`MembershipSetKindCodepoint: u16`** to accommodate the codepoint-reserve
table (u8 = 256 codepoints is enough for forever-near, but u16 is the
existing pattern per F8 codepoint-registry; small wire-cost difference; +1
byte per envelope).

**WAVE-MS-PRIMITIVE canary additions for Option E-PRIME:**
- `CRYPTO-CODEPOINTS.md` codepoint family addition (5 new reserved
  codepoints; +1 active codepoint for AtriumWithRotatingGroupKey already
  reserved per N4).
- Constructor validators per active Kind (unchanged from N2).
- Constructor refusal helper for reserved codepoints (returns
  `E_MEMBERSHIP_SET_KIND_UNSUPPORTED` per M-CONS-v2 #53 narrowing pattern).
- `MembershipSet-Spec.md` §-add "Reserved codepoints + body-impl ETA"
  (table of 5 reserved codepoints + intended use-case + revisit-trigger).
- `MembershipSet-Spec.md` §-add "Composition recipes" (13 D-bucket
  use-cases + canonical patterns).
- `V1-FROZEN-INTERFACE-DEFERRED.md` 5 Row-D-* additions (one per reserved
  codepoint) with body-impl ETA + ratification trigger.

**~0.3-0.5 wave-days total** (folded into Wave-MS-PRIMITIVE canary).

### §6.2 Future-additive contract

Adding a 4th active Kind post-v1-beta:
- Pick a reserved codepoint from the codepoint-reserve table (one of the 5).
- §15.c HALT-AND-SURFACE-TO-BEN ratification.
- Implement constructor validator + per-Kind invariants.
- Add `MemberKey` variant if needed (cross-language mirror per §3.5g
  cross-language rule-mirror discipline).
- Add Rust enum variant + TS mirror (atomic update per §3.5g).
- **Wire-format: zero-change** (codepoint already reserved at v1-beta).
- Existing client decoders: no break (reserved codepoint already
  refused-with-typed-error; new clients accept; old clients still refuse).

Adding a Kind NOT in the codepoint-reserve table post-v1-beta:
- Use one of the experimental range codepoints (0xFF00-0xFFFF) for
  prototype.
- §15.c HALT-AND-SURFACE-TO-BEN ratification to MOVE it to a numbered
  codepoint outside experimental range.
- All other steps same as above.

### §6.3 The 5 reserved codepoints — body-impl ETA + use-case mapping

| Codepoint | Intended use-case mapping (from §1 table) | Revisit-trigger | Body-impl ETA |
|---|---|---|---|
| `AtriumWithRotatingGroupKey` (0x0004) | Continuous K_Set rotation; UC-15 anonymous attested (partial); regulatory audit drivers | M5 CGKA candidate maturity + Compromise #54 trigger | Post-v1-beta (Phase-5+) |
| `Garden` (0x0005) | Larger-than-Atrium community shapes; Ben's hint hierarchy | ≥3 application requests for Garden-tier semantics in production | Post-v1-beta-GM (Phase-6+) |
| `Grove` (0x0006) | Even-larger-than-Garden multi-Atrium federation; Ben's hint hierarchy top tier | Production demand + Garden body-impl precedent | Phase-7+ |
| `EphemeralLobby` (0x0007) | UC-04 game lobby with explicit ephemeral-destroy lifecycle | Game studio production deployment with K_Set-destroy requirement | Post-v1-beta-GM |
| `Federation` (0x0008) | UC-10 federated business; UC-11 customer-vendor B2B; cross-Atrium meta-shape | N2 `cross_set_delegate_capability` body-impl (currently P4MC-deferred) | Phase-4-Meta-Composing |

---

## §7 Task 7 — Final recommendation + R0 plan-doc implications

### §7.1 Final recommendation (re-stated)

**Option E-PRIME** — Hybrid: 3-arm active `enum MembershipSetKind`
{ Atrium, DeviceMesh, SingleDevice } EXACTLY-3 + 5-codepoint-reserve table
(AtriumWithRotatingGroupKey + Garden + Grove + EphemeralLobby + Federation)
+ application-layer composition recipes §-add + §15.c HALT-AND-SURFACE-TO-BEN
gate retained for body-impl.

### §7.2 Justification recap

- Closes Ben's "overly restrictive" concern explicitly via 5 named reservations.
- Covers 22-use-case foresight: 13 DECOMPOSES via composition, 5 covered
  by reserved codepoints, 4 are correctly-not-MembershipSets.
- Strictly less code than Options C/D (which require re-architecture).
- Mirrors existing M-CONS-v2 codepoint-reserve patterns (F9, F10, N4,
  F11) — zero pattern-debt; "do more of what's working."
- Preserves N2 HYBRID-RECOMMENDED compile-time rigor + §15.c discipline.
- Forward-class-of-bug closure: future Kinds are wire-additive (no
  wire-format break).
- Cheap: ~0.3-0.5 wave-days at v1-beta.

### §7.3 Cost in wave-days

**+0.3-0.5 wave-days** at v1-beta (folded into Wave-MS-PRIMITIVE canary).
Specifically:
- ~0.1 wave-day: `CRYPTO-CODEPOINTS.md` codepoint family extension (5
  new reserved codepoints + widening MembershipSetKindCodepoint to u16).
- ~0.1 wave-day: `MembershipSet-Spec.md` §-add "Reserved codepoints +
  body-impl ETA" table.
- ~0.1-0.2 wave-day: `MembershipSet-Spec.md` §-add "Composition recipes"
  (13 D-bucket recipes; Ben's UC-01 + game backend + B2B as canonical
  exemplars).
- ~0.05-0.1 wave-day: `V1-FROZEN-INTERFACE-DEFERRED.md` 5 Row-D-* additions.
- ~0.05 wave-day: constructor refusal helper + tests.

**Cumulative M-CONS-v2 wave-day delta:** M-CONS-v2 baseline ~88-109
wave-days central + P6-B4 +0.3-0.5 = **~88.3-109.5 wave-days central;
bracket ~85.3-114.5.** Within noise of M-CONS-v2 baseline.

### §7.4 NAMED-DEFER for unselected alternatives

Per HARD RULE 12:

| Alternative | HARD RULE 12 Disposition | Named destination |
|---|---|---|
| Option A static 3-arm | DISAGREE-WITH-EXPLANATION | (rejected here; not deferred) |
| Option B codepoint-reserve only (no recipes §-add) | BELONGS-NAMED-NOW (subset of winner) | Subsumed by Option E-PRIME; recipes §-add cost ~0.1-0.2 is the cheap differentiator |
| Option C runtime tag | DISAGREE-WITH-EXPLANATION | (rejected here; not deferred) |
| Option D trait-based abstraction | DISAGREE-WITH-EXPLANATION | M-C1 stopped-mid-run rejection cited; not deferred |
| Option E original (Custom CID wildcard) | DEFER-NAMED-NOW to V1-FROZEN-INTERFACE-DEFERRED.md Row D-CUSTOM-KIND | Revisit-trigger: ≥3 application-layer requests for custom Kinds in production |
| 4 (F)-bucket use-cases (UC-07, UC-12, UC-17, UC-22) | OUT-OF-SCOPE for MembershipSet primitive (correctly excluded) | Documented as NAMED-NON-MEMBERSHIP-SET-USE-CASE rows in MembershipSet-Spec.md |
| 5 (R)-bucket use-cases (UC-04, UC-14, UC-15, UC-19, UC-21) | BELONGS-NAMED-NOW (reserved codepoint slots) | Specific codepoint reservation rows in `CRYPTO-CODEPOINTS.md` + body-impl ETA rows in V1-FROZEN-INTERFACE-DEFERRED.md |

### §7.5 R0 plan-doc implications

**The M-CONS-v2 → R0-author handoff should incorporate:**

1. **F-NEW row** in §3 amendment registry: F29 (or `F-P6-B4-A`) —
   "MembershipSetKind codepoint-reserve table extension (5 reserved
   codepoints + composition recipes §-add + V1-FROZEN-INTERFACE-DEFERRED
   rows)". Severity LB; wire-affecting CR (codepoint-reserve only at
   v1-beta); disposition v1β-LB (codepoint-reserve table) + v1β-CR
   (body-impl); +0.3-0.5 wave-days.
2. **New §-row** in §10 R0 plan-doc skeleton: "§17.K MembershipSetKind
   codepoint-reserve discipline" naming the 5 reserved codepoints + the
   §15.c HALT-AND-SURFACE gate + the composition-recipes §-add.
3. **Compromise #57 mint** (per §5.7): AI-agent-as-member trust category.
4. **M4 BREAK 5 refinement** (per §5.6): agent-DIDs ARE members; plugin-DIDs
   are not; refines existing M-CONS-v2 §11.5 wording.
5. **Inv-20 clarification:** the homogeneous-MemberKey invariant is at
   DID-method-allowlist level, not DID-method-exact-equality level. (No
   wire-format change; doc-clarification only.)
6. **Update §15.c**: explicit enumeration of the 5 reserved codepoints +
   the HALT-AND-SURFACE-TO-BEN gate for body-impl per reserved codepoint.

### §7.6 Open Ben-call decisions

- **Q-P6-B4-1:** Are the 5 codepoint reserves the right set? Specifically:
  Garden + Grove are speculative (no concrete use-case body-impl ETA yet);
  EphemeralLobby + Federation have clearer driver use-cases. Consider
  cutting Grove + relegating to experimental-range until concrete demand.
  **My-pred:** RESERVE ALL 5; reservation is cheap; revisit-trigger
  documented; better-too-many-than-too-few given §15.c gate retained.
- **Q-P6-B4-2:** Should `did:agent:` be an explicit DID method codepoint
  in `benten-id` at v1-beta, or left as application-layer convention?
  **My-pred:** application-layer convention at v1-beta; promote to explicit
  DID-method codepoint at v1-GM after Compromise #57 use-case maturity
  (≥3 production AI-agent-as-member integrations).
- **Q-P6-B4-3:** Compromise #57 mint (AI-agent-as-member trust category) —
  ratify? **My-pred:** RATIFY; refines M4 BREAK 5 cleanly; aligns Benten with
  emerging W3C Agent Identity standard.
- **Q-P6-B4-4:** Composition recipes §-add scope — 13 (D)-bucket use-cases
  is a lot of recipes; consider trimming to 5-7 canonical exemplars
  (Atrium-with-AI-assistants UC-01 + smart-home UC-02 + game-backend UC-04-08
  composite + B2B UC-10-11 composite + tournament UC-08) and naming the
  others by reference. **My-pred:** TRIM to 5-7 canonical exemplars; the
  rest are NAMED-by-reference; saves ~0.1 wave-day on the §-add cost.

---

## §8 Patterns observed

- **P-NEW-1: Codepoint-reserve discipline is the v1-beta-correct answer to
  "future-flexibility" concerns.** Across the M-CONS-v2 + N4 + F9/F10 + F11
  history + this audit, codepoint-reserve has emerged as the standard
  discipline for "we want to extend additively without breaking wire
  format". Option E-PRIME applies this discipline to the MembershipSetKind
  axis. (Generalizes M-CONS-v2 P18 "codepoint-reserve discipline scales to
  sub-slots".)
- **P-NEW-2: Foresight audits enumerate use-cases to discover bucket
  distributions, not to enumerate variants needed.** This audit produced
  22 use-cases that bucket into 13 (D-composition) + 5 (R-codepoint-reserve)
  + 4 (F-not-a-MembershipSet). The bucket distribution is what informs the
  design choice; the variant-count is not the design driver. (Methodology
  pattern for future foresight audits.)
- **P-NEW-3: AI-agent identity is converging on first-class-DID across
  the industry.** W3C Agent Identity Registry Protocol + IETF AIP +
  arxiv research all point to AI-agent-with-DID + Verifiable Credentials.
  Benten should align (admit `did:agent:` as a DID method + AI-as-Member
  via UserDid variant). The plugin-DID trust model (CLAUDE.md #18) is
  CORRECTLY DISTINCT from agent-DID trust model — both coexist.

---

## §9 Self-assessment + confidence per finding

| § | Finding | Confidence | Basis |
|---|---|---|---|
| §0-§3 | Option E-PRIME wins | HIGH | 7-finding elegant-shape closure; mirrors existing M-CONS-v2 codepoint-reserve patterns; cheapest cost; preserves N2 HYBRID-RECOMMENDED rigor; closes Ben's "overly restrictive" framing |
| §1 | 22 use-cases bucket into D=13/R=5/F=4 | MED-HIGH | Bucket assignment per use-case is defensible but subjective; ~2-3 use-cases could shift bucket assignment under Ben push-back (e.g., UC-22 friend-of-friend could move from F to D depending on social-graph primitive decision) |
| §2 | 3-Kind enum is not the bottleneck (member-key + RBAC + lifecycle extensions are) | HIGH | The 22-use-case analysis shows Kind diversity is small (5 R-bucket); member-key/RBAC/lifecycle diversity is large (handled by additive policy-fields, not new Kinds) |
| §3 | Opt E-PRIME beats A/B/C/D/E | HIGH | Per-criterion evaluation table; ground-truth-verified against existing M-CONS-v2 code patterns (F9, F10, N4) |
| §3.4 | Custom(CustomKindCid) wildcard is wrong at v1-beta | HIGH | Defeats §15.c gate; named-codepoint-reserves are stronger discipline |
| §4 | Game backend mostly DECOMPOSES via Atrium | MED-HIGH | 6 of 7 game patterns; web-research-corroborated (matchmaking-queue-as-service-coordinated pattern; AccelByte/AWS GameLift evidence) |
| §4.4 | Matchmaking-queue is NOT a MembershipSet | HIGH | Web-research-corroborated; service-coordinated pending state pattern |
| §5 | AI-agent-as-member is a 3rd trust category (extending CLAUDE.md #18) | HIGH | W3C + IETF + arxiv evidence converges; distinct from plugin-DID; refines M4 BREAK 5 cleanly |
| §5.3 | AI-as-Member gets K_Set decryption (via UserDid variant) | MED-HIGH | Default reading of homogeneous-MemberKey invariant + multi-DID-method allowlist; Ben may want a stricter sub-Member role for AI ("ReadOnly" + "ExecuteWorkflow" but no other Write) |
| §5.7 | Compromise #57 mint warranted | MED-HIGH | Refinement is bounded; aligns with emerging standards; Ben should ratify or refine |
| §6 | u16 codepoint widening from u8 is correct | HIGH | Mirrors existing F8 codepoint-registry pattern; +1 byte per envelope; bounded cost |
| §7.3 | +0.3-0.5 wave-days cost | MED-HIGH | Itemized estimate based on M-CONS-v2 baseline accounting; ±0.1 uncertainty per item; bracket spans documented |
| §7.6 Q-P6-B4-1 | All 5 reserves vs cut to 3-4 | MED | Speculative — Garden + Grove driver-cases are weak; Ben judgment-call |
| §7.6 Q-P6-B4-2 | `did:agent:` as app-layer convention at v1-beta | MED-HIGH | Conservative posture; promote later if demand emerges |

---

## §10 Sources

External web research:
- [Agent Identity Protocol (AIP) — IETF draft](https://datatracker.ietf.org/doc/draft-singla-agent-identity-protocol/00/)
- [W3C Agent Identity Registry Protocol CG](https://www.w3.org/community/agent-identity/)
- [AI Agents with DIDs and VCs (arxiv 2511.02841)](https://arxiv.org/abs/2511.02841)
- [Zero-Trust Identity Framework for Agentic AI (arxiv 2505.19301)](https://arxiv.org/pdf/2505.19301)
- [4tress.org — Decentralized Identity for AI Agents](https://4tress.org/)
- [RFC 9420 — MLS Protocol](https://datatracker.ietf.org/doc/rfc9420/)
- [RFC 9750 — MLS Architecture](https://www.rfc-editor.org/rfc/rfc9750.html)
- [AccelByte — Game Matchmaking Architecture](https://accelbyte.io/blog/scaling-matchmaking-to-one-million-players)
- [AWS GameLift Serverless Custom Matchmaking](https://aws.amazon.com/blogs/gametech/fitting-the-pattern-serverless-custom-matchmaking-with-amazon-gamelift/)
- [TechLatest — Pub-Sub Architectures for Cross-Regional Matchmaking](https://tech-latest.com/reinventing-real-time-multiplayer-advanced-pub-sub-architectures-for-cross-regional-matchmaking/)
- [Matrix.org Application Service API](https://spec.matrix.org/unstable/application-service-api/)
- [Federated Identity Management in IoT](https://www.conf42.com/Internet_of_Things_IoT_2024_Mahesh_Vankayala_identity_federation)
- [Self-Sovereign Identity for IoT (arxiv 2003.05106)](https://arxiv.org/pdf/2003.05106)

Internal artifacts cited:
- M-CONS-v2 consolidator @ `e62ff540` `.addl/phase-4-meta/membership-set-m-cons-v2-consolidator.md`
- M-C2-v2 critique @ `0533d0cf` `.addl/phase-4-meta/membership-set-m-c2-v2-composability.md`
- N2 generic-MembershipSet @ `a1b5a552` `.addl/phase-4-meta/membership-set-n2-generic-primitive-ops-roles.md`
- M2 primitive design @ `6170980b` `.addl/phase-4-meta/membership-set-m2-primitive-design.md`
- M4 red-team @ `066785b5` `.addl/phase-4-meta/membership-set-m4-red-team-negative-findings.md`
- M5 CGKA candidate survey @ `15819500` `.addl/phase-4-meta/membership-set-m5-cgka-candidate-survey.md`
- `docs/V1-FROZEN-INTERFACE.md` §15.c (Scope-2-arm carve-out + invariant
  discipline pattern)
- `MEMORY.md` feedback rules: `extra_reflection_pass_for_elegant_permanent_shape`,
  `engine_primitives_vs_application_layer`, `plugin_extension_trust_model`,
  `no_defer_HARD_RULE`, `review_finding_ground_truth_verify`
