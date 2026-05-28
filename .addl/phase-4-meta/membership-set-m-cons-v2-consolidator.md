# MembershipSet unification — **M-CONS-v2** (post-N1/N2/N3/N4 + Ben-ratifications consolidator)

> **Top-banner re-orient (HANDOFF discipline).** This document is the **second-pass**
> integration of 4 post-M-CONS-v1 N-refinement specialists (N1 sub-graph-sharing-elegance /
> N2 generic-primitive-ops-roles / N3 continuous-rotation-edge-cases / N4 Signal-Sender-Keys-
> comparison) plus Ben's intervening ratifications (Q1 iroh-gossip SCOPE-IN at v1-beta;
> N2 24-op classification as-is; #55 GDPR-RTBF honest-disclosure framing; Option-B
> per-message-ratcheting codepoint-reserve; AtriumWithCGKA → AtriumWithRotatingGroupKey
> rename) into the **M-CONS-v1 baseline** (74580ee6). The output is the final-final
> consolidated design — decision-ready for the M-C1/M-C2/M-C3 critique round.
>
> Read BEYOND what's named here if you arrive cold: M-CONS-v1 at `74580ee6`, N1 at
> `ed592770`, N2 at `a1b5a552`, N3 at `298c80d9`, N4 at `2ee24e9f`, the 5-panel + 3-
> cataloger background per M-CONS-v1 §12, Q3-revisit Option D `23f76e24`, AtriumPolicy
> `e90900b4`, Path-A.5 winner `5f50a028`, plus the CLAUDE.md baked-in #1/#5/#15/#17/#18/#19
> + MEMORY.md feedback rules (canary-first / no-defer HARD RULE / elegant-shape reflection
> pass / engine-primitives-vs-application-layer / iterate-to-convergence).
>
> Tree HEAD at write: `2172cb6d` (origin/main, fetched + verified at session start).
> Branch: `phase-4-meta-core/membership-set-m-cons-v2-consolidator`.
> Date: 2026-05-28.

- **Role.** M-CONS-v2 consolidator — single coherent reconciliation of M-CONS-v1 +
  N1/N2/N3/N4 + Ben ratifications. Decision-ready input for Ben + the subsequent M-C1/
  M-C2/M-C3 critique round.
- **Posture.** ADVISORY. Where N-refinements conflict with M-CONS-v1, surface explicitly
  + my-pred resolution. Ben's intervening ratifications are FIXED inputs (don't
  relitigate). HARD RULE 12 dispositions explicit for every divergence.
- **Scope.** Tasks 1–11 per the dispatch brief. NOT primitive Rust API drafting (N2
  owns canonical Rust shape). NOT M-C1/C2/C3 critique (those run against this doc).
- **Out-of-scope.** Authoring the full F-full R0 plan-doc (M-R0 owns the substantive
  content; this doc updates the skeleton). Re-running the M-CONS-v1 reconciliation.

---

## §0 Headline summary (read-cold)

**Verdict.** **M-CONS-v1 baseline HOLDS as the structural skeleton; N1/N2/N3/N4 all
land as bounded refinements; 5 Ben-ratifications integrate cleanly.** No N-specialist
overturned any M-CONS-v1 ratification. The cumulative refinement-set is:

1. **N1 Option H NESTED-SPEC (RestrictedScopeSet)** — promote the already-shipping
   `combinators::union` from internal combinator surface to grant-time composition
   primitive. New field-type inside `Scope::RestrictedSelector` arm; preserves all 5
   frozen-surface disciplines; +0.95–1.2 wave-days. **F28 minted.**
2. **N2 HYBRID-RECOMMENDED** — keep M-CONS-v1's typed `MembershipSetKind` + add uniform
   `authorities: BTreeSet<Authority>` slot (replacing M-CONS-v1 §5.1 `KindPolicyAdminEntity`
   enum) + 3-role RBAC (`Admin > Member > Viewer`) + 3-permission (`Read|Write|Admin`)
   + 11 v1-beta-LB ops (4 new beyond M2) + 6 codepoint-reserve + 3 P4MC + 4 post-v1-beta
   + `MembershipSetMetadata` (name/description). **+1.5–2.5 wave-days; F-N2-A/B/C
   minted; absorbs F-A1+F-A2; extends Inv-20 to 10 clauses.**
3. **N3 sharpenings (doc-only)** — `RotationTrigger` doc-enum (`AdminKick | MemberDepart
   | AdminDepart | DeviceRevoke | CompromiseResponse | PeriodicHygiene`); clarify
   `policy.refresh_required_secs` semantic (attestation-refresh, NOT K_Set rotation);
   mint Compromise #54 (continuous-rotation deferral risk + post-v1-beta revisit
   trigger); mint Compromise #55 (GDPR-RTBF **per Ben's framing: P2P-by-design semantic;
   applications layer strong-RTBF if needed** — honest-architectural-disclosure, not
   "limitation we should fix"); optional Compromise #56 (journalist per-message FS
   deferral). **~2 wave-days; doc-only.**
4. **N4 doc-only refinements** — `AtriumWithCGKA` → `AtriumWithRotatingGroupKey`
   codepoint-reserve rename (Ben-ratified); add `RotatingGroupKeyChainedMode`
   codepoint-reserve sub-slot (Ben Option-B for per-message-ratcheting); add
   SSK-comparison §-row to `docs/SECURITY-POSTURE.md`. **~1 wave-day; doc-only.**
5. **Q1 iroh-gossip SCOPE-IN at v1-beta (Ben-ratified)** — overrides M6's codepoint-
   reserve-only recommendation. Wave-MS-TRANSPORT now ships the actual iroh-gossip
   integration, not just the codepoint slot. **+5–8 wave-days vs M-CONS-v1 estimate
   (was +3.5 codepoint-only).**

**Cumulative cost.** M-CONS-v1 baseline ~78–100 wave-days + Q1 iroh-gossip-impl
(+5–8) + N1 (+0.95–1.2) + N2 (+1.5–2.5) + N3 (~2 doc) + N4 (~1 doc) =
**~88.45–114.7 wave-days central; bracket ~85–118 with uncertainty.**

**Confidence.** HIGH on the directional integration (no N-specialist contradicts
M-CONS-v1; all 4 are bounded refinements). MED-HIGH on cumulative cost arithmetic
(many small compounds; iroh-gossip-impl variance dominates the bracket width). HIGH
on the F-row renumbering + Compromise-# renumbering (mechanical with explicit
collision-resolution rules).

**Open Ben-call decisions remaining (post-N-refinement).** 3 substantive + 1
clarifying — see §9. Most M-CONS-v1 Ben-calls are now resolved by N-refinement
outputs or Ben's intervening ratifications.

---

## §1 Task 1 — Integrate N1–N4 refinements + Ben ratifications

### §1.1 N1 — RestrictedScopeSet (Option H NESTED-SPEC)

**Action.** Add F-row F28 to amendment registry (§3). Extend Inv-20 clause-h to name
the `RestrictedScopeSet` algebra at the grant boundary. Fold into Wave-MS-PRIMITIVE
canary (no new wave). Add §17.X row to R0 plan-doc skeleton (§10).

**Decisions absorbed:**
- `Scope::{Hashes | RestrictedSelector}` EXACTLY-2-arm preserved (the change is
  INSIDE the `RestrictedSelector` arm — the field type swaps from `RestrictedScope`
  to `RestrictedScopeSet`).
- `RestrictedScopeSet { scopes: Vec<RestrictedScope>, audit_commitment: Option<Cid> }`
  (the optional Merkle audit-commitment is Option-G-AUDIT-SIBLING per N1 §5.2;
  named-defer at v1-beta-LB landing-status, surfaced as F25 audit-pack sub-row).
- Containment-algebra lift: `RestrictedScopeSet(A).contains(RestrictedScopeSet(B))`
  ⇔ ∀ b ∈ B.scopes, ∃ a ∈ A.scopes s.t. `a.contains(b)`. Decidable, transitive,
  reflexive (proptest-verified at g-core-3w `5c2947c8`).
- Backward-compat: single-element `RestrictedScopeSet` is wire-equivalent to the old
  `RestrictedScope` shape via codepoint-discriminated encoding (single-scope codepoint
  = old shape; multi-scope codepoint = new shape; accept both for one release per
  CLAUDE.md #5 crypto-agility seam).

**Disagreements with M-CONS-v1 baseline:** none. N1 is purely additive to M-CONS-v1
§2 amendment registry. The Option H mechanism does not change MembershipSet shape
(MembershipSet governs K_Set distribution; RestrictedScopeSet governs sub-graph
walk-shape; they compose orthogonally at the `(MembershipSet, RestrictedScopeSet)`
seal-seam tuple per N1 §3.6 row).

**HARD RULE 12 disposition for N1 alternatives.** N1 §5.2:
- Option C-LITE (incremental named dimensions): NAMED-DEFER to V1-FROZEN-INTERFACE-
  DEFERRED.md Row D-NEW-1; revisit-trigger = use-case not cleanly expressible as
  union of sub-scopes.
- Option G as PRIMARY (committed Merkle enumeration): DEFER — DISAGREE-WITH-EXPLANATION;
  pre-commits future Nodes, defeats incremental-Atrium growth.
- Options D/E/F + Anchor-rooted: DEFER — DISAGREE-WITH-EXPLANATION with reasons
  cited (CP-ABE pairing-crypto + Inv-17 violation; per-Node access-tree breaks
  key-IS-the-access property; selector-evaluator violates CLAUDE.md #1; Anchor-
  rooted breaks Path-A.5 K(V) invariant).

### §1.2 N2 — HYBRID with Authority + RBAC + extended ops + metadata

**Action.** Major absorption into M-CONS-v1. Key replacements:

| M-CONS-v1 element | N2 replacement |
|---|---|
| §5.1 `KindPolicyAdminEntity` 3-variant enum | `Authority { did, sig_pubkey, role, admitted_at_hlc }` struct + `authorities: BTreeSet<Authority>` uniform slot |
| Implicit "admin can do anything; everyone else is member" binary | EXPLICIT 3-role RBAC: `RoleId { Admin = 2, Member = 1, Viewer = 0 }` + `PermissionSet { READ | WRITE | ADMIN }` + linear hierarchy `Admin ⊇ Member ⊇ Viewer` |
| F-A1 (remove_member doc sharpening) | Absorbed into F-N2-B op-23 audit-log row + SECURITY-POSTURE.md per-Kind kick-table |
| F-A2 (MembershipEvent typed-enum) | Absorbed into F-N2-A scope (lives in `benten-membership-set` crate) + emitted by all 11 v1-beta-LB ops + queried by op-23 `audit_log_query` |
| M2's 7-op API | 11 v1-beta-LB ops + 6 codepoint-reserve + 3 P4MC + 4 post-v1-beta (24 total enumerated per N2 §3.1; **Ben ratified the 24-op classification as-is**) |
| Missing UX name slot | `MembershipSetMetadata { name, description, created_by_human_label }` inside `MembershipSetPolicy` (closes the M-CONS-v1 §0 "Atrium vs AtriumFork" naming-as-documentation question) |
| Inv-20 8-clause | Inv-20 10-clause (adds clause-i uniform-Authority + clause-j 3-role-RBAC + UCAN-intersection-of-allows) |

**UCAN composition.** RATIFIED-INTERSECTION-OF-ALLOWS (N2 Decision 9; my-pred HIGH).
RBAC sets coarse floor; UCAN attenuates per-resource via `benten-caps::chain_authority`.
Both must allow for the op to proceed.

**Per-Kind constructor invariants (runtime-validated; not type-system-enforced).**
- Atrium: ≥1 Authority with role=Admin; MemberKey variant ALL `UserDid`.
- DeviceMesh: exactly-1 Authority with did=user_did, role=Admin; MemberKey variant
  ALL `DeviceDid` carrying `DeviceAttestation`.
- SingleDevice: exactly-1 Authority with did=device_did, role=Admin; MemberKey
  variant ALL `LocalDevice`.
- `role_assignments` keys ⊆ members ∪ authorities-dids.
- If a DID's role in `role_assignments` is `Admin`, the DID MUST also appear in
  `authorities`.

**Type-system rigor trade-off** (N2 §4.6). The N2 hybrid trades slight compile-time
rigor (per-Kind admin cardinality is runtime-enforced via constructor, not type-
enforced via enum variant) for substantial uniformity (one `authorities` slot, one
`change_role` op, threshold-admin grows via set-cardinality not new variant). **my-
pred WORTH IT — runtime validation is bounded (~10 LOC constructor + ~5 LOC per
mutation op); future threshold-admin growth case + 3-role RBAC cleanliness both
favor the uniform set.**

**Disagreements with M-CONS-v1 baseline.** §5.1 KindPolicyAdminEntity is the only
substantive replacement. M-CONS-v1 §5 D1–D7 walkthrough is re-walked under the
Authority-slot generalization in §6 below.

### §1.3 N3 — RotationTrigger + refresh_required_secs clarification + Compromise mints

**Action.** Doc-only sharpenings. No code-shape change at v1-beta wire format.

**RotationTrigger doc-enum (N3 §7.1).** Added to MembershipSet-Spec doc as a typed
enum naming the 6 fork-event reason categories:

```rust
/// Doc-enum surfaced in MembershipSet-Spec; carried as `reason` field on
/// MembershipEvent::AtriumFork (F-A2 absorbed into F-N2-A). NOT a wire-format
/// change; it's the AUDIT-EVENT classifier.
pub enum RotationTrigger {
    AdminKick,           // adversarial member kicked by admin
    MemberDepart,        // voluntary leave by member
    AdminDepart,         // admin with key-management access departs
    DeviceRevoke,        // DeviceMesh device revocation (e.g., stolen phone)
    CompromiseResponse,  // suspected key compromise; emergency fork
    PeriodicHygiene,     // scheduled cadence-driven fork (HIPAA 90-day / PCI annual)
}
```

All 6 triggers map to the SAME wire-format mechanism at v1-beta: `MembershipSet::fork()`
with a `MembershipEvent` audit record carrying the trigger reason. NO wire-format
change. NO `RotationPolicy` enum change (stays 2-arm `ForkOnly | AdminKickEpoch`
per M-CONS-v1; AdminKickEpoch folded-into-FORK-ONLY at v1-beta per M5).

**`policy.refresh_required_secs` semantic clarification (N3 §5.3 + §7.2).** Add
explicit MembershipSet-Spec note: `refresh_required_secs` denotes the cadence for
**membership-attestation re-signing** per AtriumPolicy D3. It is **NOT** a K_Set
rotation cadence. If/when continuous-rotation is added post-v1-beta, it will use
a separate policy field (additive). N3 §8.2 noted "rename to `attestation_refresh_secs`
if M2 hasn't frozen yet" — **my-pred: KEEP existing field name + add clarification
note**, because M2's struct is the v1-beta-pinned shape and any rename would be a
wire-format change. The clarification note is sufficient. Confidence MED-HIGH on
this resolution.

**Compromise mints** (full registry in §4 below):
- **#54** — Continuous-rotation deferral risk (N3 §7.4 watch-list). Names silent-
  undetected-CURRENT-member-compromise + one-shot key leakage as the canonical
  post-v1-beta `AtriumWithRotatingGroupKey` revisit-trigger.
- **#55** — **GDPR right-to-be-forgotten honest-architectural-disclosure per Ben's
  framing.** P2P-by-design semantic: a content-addressed P2P system cannot
  unilaterally erase content that has been replicated to other devices' local stores
  — that's the substrate guarantee that gives Benten its trustlessness. Applications
  that need strong-RTBF (legal compliance contexts) layer it on top via per-subject
  crypto-shredding sub-encryption layer (separate per-subject keys; delete subject's
  key shreds only their data); this lives at the Atrium-data layer, not
  MembershipSet. **This is the honest architectural posture; not "a limitation we
  should fix."** Framing matters for audit deliverable.
- **#56 (optional)** — Journalist per-message FS deferral (N3 §3.2). Names
  per-message ephemeral-key forward-secrecy (Signal-double-ratchet pattern at
  per-message granularity) as a SEPARATE design problem not served by membership-
  rotation; defer to post-v1-beta `MembershipSetKind::EphemeralRatchet` codepoint-
  reserve consideration. **my-pred: MINT (separate from #54) because it's a
  structurally-different design class; auditor would otherwise conflate.**

**Disagreements with M-CONS-v1 baseline.** None. N3 explicitly held M5's conclusion
at HIGH 85% confidence. The 3 sharpenings + 3 mints are pure additions.

### §1.4 N4 — codepoint-reserve rename + chained-mode sub-slot + SSK-comparison §-row

**Action.** Apply per Ben ratifications.

- **AtriumWithCGKA → AtriumWithRotatingGroupKey rename (R-N4-1; Ben-ratified).**
  Neutral; covers SSK-shape OR MLS-shape OR DCGKA-shape without baking in choice.
  `MembershipSetKind::AtriumWithRotatingGroupKey` is the codepoint-reserve slot
  (NOT a 4th enum arm at v1-beta; future ratification = HALT-AND-SURFACE-TO-BEN
  per §15.c discipline; preserves M-CONS-v1's EXACTLY-3-arms invariant for the
  active enum).
- **`RotatingGroupKeyChainedMode` codepoint-reserve sub-slot (R-N4-2; Ben Option-B
  ratified).** Inner-codepoint slot under the `AtriumWithRotatingGroupKey` reserve
  for {`NoChain`, `SsKChain`, `MlsChain`, `DcgkaChain`} variants. Preserves the
  wire-format slot for post-v1-beta per-message-ratcheting of group secret if a
  future regulatory-audit-driven use-case requires it. v1-beta-LB = NO; landing =
  post-v1-beta.
- **SECURITY-POSTURE.md §-row "Comparison to Signal Sender Keys" (R-N4-3).**
  Pre-empts the audit-firm question "why isn't this CGKA-LITE? Why not SSK?".
  Includes 10-row subset of N4 §3 22-row comparison matrix + explicit naming of
  Inv-20 clause-c AAD-binding-of-sender_did as the inter-member-non-forgeability
  defense (Benten's analogue to SSK's Ed25519 sig) + concurrent-fork tie-break
  rule (N4 §4.2) + anonymous-credentials trade-off (N4 §4.5) + multi-device
  content-addressed-DAG cleanliness (N4 §4.8). **~80-line §-add; ~0.5 wave-day.**

**N4 R-N4-3a sub-recommendation (small).** Add audit-deliverable §-row documenting
concurrent-fork tie-break rule: "if two admin-signed forks at same
`parent_membership_set_id` have differing `created_at_hlc`, the lexicographically-
smaller HLC wins; if equal-HLC, lexicographically-smaller MembershipSetId wins;
clients MUST archive (not discard) the losing fork for audit." Already implicit in
M-CONS-v1 Inv-20 + HLC discipline; **my-pred: ADD as audit-deliverable §-row to
prevent implementation drift; bounded ~0.5 wave-day; absorb into N4 R-N4-3.**

**Disagreements with M-CONS-v1 baseline.** Only the codepoint-reserve rename. The
M-CONS-v1 F13 (FS-gap disclosure + CGKA codepoint reservation) now reads
"`AtriumWithRotatingGroupKey` slot" not "`AtriumWithCGKA` slot." Mechanical rename.

### §1.5 Ben ratifications since M-CONS-v1

All 5 are FIXED INPUTS — not relitigated. Restated for context:

1. **Q1 iroh-gossip SCOPE-IN at v1-beta** (overrides M6's codepoint-reserve-only
   recommendation). Wave-MS-TRANSPORT now ships actual iroh-gossip integration impl.
   See §7 for updated wave-day cost; §2 below for what the change means in absolute
   terms (it does not change the wire-format codepoint-table; it changes WHAT lands
   AT v1-beta behind those codepoints from "reserved + refusal" to "ships").
2. **N2's 24-op classification ratified as-is** (11 v1-beta-LB + 6 codepoint-reserve
   + 3 Phase-4-Meta-Composing + 4 post-v1-beta per N2 §3.1).
3. **Compromise #55 GDPR-RTBF honest-architectural-disclosure framing** (P2P-by-
   design semantic; applications layer strong-RTBF if needed) — NOT "limitation we
   should fix."
4. **Option B codepoint-reserve for per-message ratcheting** (`RotatingGroupKeyChainedMode`
   sub-slot per N4 R-N4-2).
5. **AtriumWithCGKA → AtriumWithRotatingGroupKey rename** (N4 R-N4-1).

---

## §2 Task 2 — Ben's iroh-gossip SCOPE-IN ratification (impact on Wave-MS-TRANSPORT)

### §2.1 What changes

M6's recommendation was **Option A (codepoint-reserve only at v1-beta)**: freeze the
`TransportConfig` wire shape + types + default-selection logic, but defer the
production iroh-gossip / Willow / iroh-roq / iroh-live impl to Phase-4-Meta-Composing
/ Phase-5+. Cost: ~3.5 wave-days. Engine refuses `GossipPlusBlobs` / `Reserved` at
v1-beta with `E_TRANSPORT_KIND_UNSUPPORTED`.

**Ben overrode to scope-in iroh-gossip IMPL at v1-beta.** What this changes:

| Aspect | M-CONS-v1 (M6 Option A) | M-CONS-v2 (Ben SCOPE-IN) |
|---|---|---|
| Codepoint-reserve | YES (wire shape frozen) | YES (unchanged) |
| `TransportConfig` types in `benten-engine` | YES | YES (unchanged) |
| Default-selection logic | YES | YES (unchanged) |
| iroh-gossip production impl behind `GossipPlusBlobs` codepoint | **DEFERRED** (refusal with E_TRANSPORT_KIND_UNSUPPORTED) | **SHIPS at v1-beta** |
| Willow / iroh-roq / iroh-live impl | DEFERRED | DEFERRED (unchanged) |
| Compromise #53 (TransportConfig-codepoint-reserve-at-v1-beta) | Honest disclosure | **NARROWS** — only Willow / iroh-roq / iroh-live codepoints are reserve-only at v1-beta; `GossipPlusBlobs` now ships |
| Wave-MS-TRANSPORT wave-day cost | ~3.5 | **~5–8** (M6 §10.1 + iroh-gossip integration spike + e2e pin) |

### §2.2 Cost re-estimate for Wave-MS-TRANSPORT

Iroh-gossip integration adds these sub-deliverables on top of M6 §10.1's 10 items:

1. iroh-gossip peer-discovery integration with Benten DID-to-peer-id mapping (~1.5
   wave-days; net-new pattern).
2. Multi-stanza HPKE envelope publishing/subscribing over gossip topic per
   MembershipSet (~1 wave-day).
3. Gossip topic-name canonicalization + cite-drift cross-language mirror (~0.3
   wave-days).
4. RED-PHASE staged-pin un-ignore for production gossip path (~0.5 wave-days; pim-12
   discipline).
5. e2e pin: full Atrium-create + member-add + encrypt-to-set + receive-on-second-
   device flow over actual iroh-gossip transport (per pim-2 substantive-arm
   discipline; ~0.5 wave-days).
6. Spike C-evidenced footgun mitigations (per M6 §7.2; ~0.5–1 wave-day depending
   on which footguns surface during impl).
7. NAT-traversal + relay-fallback integration test discipline (~0.5–1 wave-days).
8. Operational hygiene + observability hooks (~0.5 wave-days; bounded).

**Subtotal addition vs M6 Option A: +1.5–4.5 wave-days (median +3).** Final
Wave-MS-TRANSPORT cost: **~5–8 wave-days central; bracket ~4.5–9** (the variance
is dominated by which iroh-gossip footguns surface during impl + how clean the
NAT-traversal story is in production code).

### §2.3 Compromise #53 narrows

**M-CONS-v1 #53.** "TransportConfig-codepoint-reserve-at-v1-beta — `TransportConfig`
wire shape frozen at v1-beta but production iroh-gossip / Willow / iroh-roq /
iroh-live impl deferred to Phase-4-Meta-Composing / Phase-5+. Engine refuses
`GossipPlusBlobs` / `Reserved` at v1-beta with `E_TRANSPORT_KIND_UNSUPPORTED`."

**M-CONS-v2 #53 (NARROWED).** "TransportConfig-codepoint-reserve-at-v1-beta —
`TransportConfig` wire shape frozen at v1-beta; iroh-gossip production impl SHIPS
at v1-beta behind `GossipPlusBlobs` codepoint per Ben SCOPE-IN ratification.
Willow / iroh-roq / iroh-live impl REMAIN DEFERRED to Phase-4-Meta-Composing /
Phase-5+. Engine refuses `Reserved` codepoints at v1-beta with
`E_TRANSPORT_KIND_UNSUPPORTED`; `GossipPlusBlobs` accepts as production path."

### §2.4 Wave-MS-TRANSPORT dispatch position

Per M-CONS-v1 §6.4: **dispatches AFTER Wave-MS-PRIMITIVE** (depends on MembershipSet
struct existing). M-CONS-v2 preserves the dispatch ordering; the wave still goes
canary-then-parallel-fanout per `feedback_canary_first_parallel_implementation.md`,
but with substantially more code shipping inside the wave.

---

## §3 Task 3 — Final-final amendment registry (F1–F28 + F-N2-A/B/C)

Renumbering scheme: M-CONS-v1 F1–F27 + F-A1/F-A2 → M-CONS-v2 F1–F28 + F-N2-A/B/C
with F-A1/F-A2 ABSORBED into F-N2-A/F-N2-B per N2 §5.3.

Format: `F# | origin | statement | severity | wire-affecting | disposition |
depends-on | Compromise-or-Inv-impact | confidence`.

**Severity** key: `LB` (LOAD-BEARING) / `MH` (MED-HIGH) / `DEF` (DEFERRABLE).
**Wire** key: `Y` / `N` / `CR` (CODEPOINT-RESERVE-ONLY).
**Disposition** key: `v1β-LB` / `v1β-CR` / `v1-GM-DEF` / `post-v1β-DEF`.

| F# | Origin | Statement | Sev | Wire | Disp | Depends-on | Impact | Conf |
|---|---|---|---|---|---|---|---|---|
| F1 | M-CONS-v1 | Codepoint committed in AAD/info | LB | Y | v1β-LB | — | aead.rs + CRYPTO-CODEPOINTS.md | HIGH |
| F2 | M-CONS-v1 | Strict-decode; no cross-variant fallback | LB | Y | v1β-LB | F1 | aead.rs + ENVELOPE-V1-SPEC.md | HIGH |
| F3 | M-CONS-v1 | Canonical TLV length-injectivity | LB | Y | v1β-LB | F1 | aead.rs | HIGH |
| F4 | M-CONS-v1 | Sender-DID + member-DID-list in AAD (non-Vault MembershipSet variants) | LB | Y | v1β-LB | F1, F17 | aead.rs + AAD layout doc | HIGH |
| F5 | M-CONS-v1 | Replay-window per-MembershipSet-Kind exclusion table | LB | Y | v1β-LB | F1, F17 | aead.rs + R-C2 + U28 | HIGH |
| F6 | M-CONS-v1 | Bernstein-Persichetti Decap CT mitigation | LB | N | v1β-LB | — | libcrux-ml-kem + Compromise #32 | HIGH |
| F7 | M-CONS-v1 | BE codepoint on-wire | LB | Y | v1β-LB | F1 | aead.rs (Q2 BE migration in progress) | HIGH |
| F8 | M-CONS-v1 | Codepoint registry IANA-disjoint + cite-drift scanner + MembershipSetEncryption family | LB | Y | v1β-LB | F1 | CRYPTO-CODEPOINTS.md | HIGH |
| F9 | M-CONS-v1 | `EnvelopePayload` `#[non_exhaustive]` + typed-reject + MembershipSetEncryption variant | LB | Y | v1β-LB | F8, F17 | benten-crypto-suite | HIGH |
| F10 | M-CONS-v1 | `BindingContext` `#[non_exhaustive]` + typed-reject + MembershipSetSeal variant | LB | Y | v1β-LB | F1, F17 | benten-crypto-suite | HIGH |
| F11 | M-CONS-v1 | Escape codepoint + experimental range + EnvelopeShape codepoint axis | LB | Y | v1β-LB | F8 | CRYPTO-CODEPOINTS.md | HIGH |
| F12 | M-CONS-v1 | Nonce-length variant discrimination | LB | Y | v1β-LB | F1 | aead.rs | HIGH |
| **F13** | M-CONS-v1 + **N4 rename** | FS-gap honest-disclosure + MLS-PQ + **AtriumWithRotatingGroupKey** codepoint reservation (renamed from AtriumWithCGKA per N4 R-N4-1; Ben-ratified) | LB | CR | v1β-LB (disclosure) + v1β-CR (MLS-PQ + AtriumWithRotatingGroupKey slots) | F11 | SECURITY-POSTURE.md #42 + #52 + #54 + CRYPTO-CODEPOINTS.md | HIGH |
| F14 | M-CONS-v1 | `aad_version: u8` prefix | LB | Y | v1β-LB | F1 | aead.rs | HIGH |
| F15 | M-CONS-v1 | Did multikey + Did::Unknown | LB | Y | v1β-LB | — | benten-id | HIGH |
| F16 | M-CONS-v1 | CodepointLifecycle typed-state | LB | N | v1β-LB | F8 | CRYPTO-CODEPOINTS.md | HIGH |
| **F17** | M-CONS-v1 | **MembershipSet primitive (typed-variant Atrium/DeviceMesh/SingleDevice; multi-stanza-HPKE distribution; FORK-ONLY rotation; per-stanza AAD; generation-CRDT-vector; transport-config codepoint-reserve)** | **LB** | **Y** | **v1β-LB** | F1, F8 + own crate | NEW `benten-membership-set` crate + V1-FROZEN-INTERFACE.md row + MembershipSet-Spec doc + TS mirror | **HIGH** |
| F18 | M-CONS-v1 | Triple-CID model parameterized over MembershipSet (`plaintext_cid_local` LOCAL-ONLY + `plaintext_cid_membership_set` + `envelope_blob_cid`) per Q3 DUAL-CID-on-wire | LB | Y | v1β-LB | F17 | ENVELOPE-V1-SPEC.md + benten-graph local-index | HIGH |
| F19 | M-CONS-v1 | MembershipSet-generation tracking (CRDT-vector per-member-DID partition) | LB | Y | v1β-LB | F17 | benten-membership-set | HIGH |
| F20 | M-CONS-v1 | `PermissionOperation::ExecuteWorkflow` variant + per-Kind sub-arms + per-request-nonce | LB | Y | v1β-LB | F10, F17 | benten-caps + UCAN spec | HIGH |
| F21 | M-CONS-v1 | Sealed-Sender additive codepoint family slot (paired with MembershipSetEncryption family per Inv-18b) | LB (slot) + RECOMMENDED (impl) | Y (slot) | v1β-CR + v1-GM-DEF impl | F8, F17 | CRYPTO-CODEPOINTS.md + Inv-18b + Am4 | HIGH |
| F22 | M-CONS-v1 | Per-relay-unlinkability + universal size-class buckets + cover-traffic deferral + DID-rotation deferral | mixed | Y (buckets) / N (cover-traffic) | mixed | F1, F8 | THREAT-MODEL.md | MH |
| F23 | M-CONS-v1 | Cross-ecosystem-identifier emit-discipline (per-Kind map row) + DAG-CBOR outer framing + strict-deterministic-CBOR-decode | LB | Y | v1β-LB | F8 | CROSS-ECOSYSTEM.md + DAG-CBOR doc | HIGH |
| F24 | M-CONS-v1 | Impl-engineering pack (libcrux-ml-kem per Q1 + XChaCha20 + NAPI canonical_binding + NAPI opaque-handle + MembershipSetHandle + wasm_js cfg + cancel-safety + Argon2idParameterTier per-Kind for DeviceMesh+SingleDevice Vault) | LB | mixed | v1β-LB | F17 | impl + INTERNALS.md | HIGH |
| F25 | M-CONS-v1 + **N1 audit-commitment sub-row** + **N4 SSK-comparison §-row** | Audit-deliverable pack (golden-vector corpus + dudect CI + kani injectivity + THREAT-MODEL.md + SECURITY-PROOFS.md + perf-bench infra + offline-replay carve-out + drop-bundle predecessor-CID chain + MAL-BIND-K-PK doc + cite-drift cross-language config-mirror + per-Kind discipline + MembershipSetPolicy generalization + **N1 `audit_commitment` field** + **N4 SSK-comparison §-row + concurrent-fork tie-break rule**) | mixed | N | v1β-LB | F17, F26 | THREAT-MODEL.md + SECURITY-PROOFS.md + CRYPTO-CODEPOINTS.md + ENVELOPE-V1-SPEC.md + cite-drift scanner | HIGH |
| **F26** | M-CONS-v1 + **N2 absorbed** | **MembershipSetPolicy generalization (D1–D7 per-Kind; uniform `authorities: BTreeSet<Authority>` slot per N2 §1.4 replacing M-CONS-v1 KindPolicyAdminEntity; permissive_default per-Kind constructor + MembershipSetMetadata)** | **LB** | **Y** | **v1β-LB** | F17 | benten-membership-set + MembershipSet-Spec | **MH** |
| **F27** | M-CONS-v1 + **Ben Q1 SCOPE-IN** | **TransportConfig + iroh-gossip impl at v1-beta (TransportKind + SubInheritPolicy + TransportExtension enums; default-selection logic; `GossipPlusBlobs` ships actual iroh-gossip integration; Willow + iroh-roq + iroh-live remain codepoint-reserve-only deferred Phase-4-Meta-Composing / Phase-5+)** | **LB** | **Y** | **v1β-LB (Gossip + reserve-others)** | F17 | benten-engine + V1-FROZEN-INTERFACE.md + TRANSPORT-CONFIG.md + Compromise #53-narrowed | **MH** |
| **F28** (NEW M-CONS-v2 from N1) | N1 §3.6 + §5.4 | **`RestrictedScopeSet` (sub-graph asymmetric-shape; `SubgraphSpec::union` exposed at grant boundary; backward-compat via codepoint-dispatch; preserves all 5 frozen-surface disciplines)** | **LB** | **Y** | **v1β-LB** | F8, F17 | benten-caps::scope + RestrictedScope + V1-FROZEN-INTERFACE §15.c amendment | **HIGH** |
| **F-N2-A** (NEW M-CONS-v2 from N2) | N2 §1.4 + §2 + §4 | **Authority struct + `authorities: BTreeSet<Authority>` slot + 3-role RBAC (`Admin > Member > Viewer`) + 3-permission set (`Read|Write|Admin`) + linear hierarchy + UCAN-composed intersection-of-allows + `RoleAssignments: BTreeMap<Did, RoleId>` + `MembershipEvent` enum hosted here (absorbs F-A2)** | **LB** | **Y** | **v1β-LB** | F17 | benten-membership-set + SECURITY-POSTURE.md RBAC §-add + benten-caps composition seam | **HIGH** |
| **F-N2-B** (NEW M-CONS-v2 from N2) | N2 §3 + Ben 24-op classification ratified | **11 v1-beta-LB ops (create / add_member / remove_member / change_role / member_key_rotation / self_leave / readmit_member-as-alias / rename-update_metadata / encrypt_to_set / dedup_blind_cid / fork / update_policy / update_policy_value / audit_log_query) + 6 v1-beta-CR ops (propose_member + accept_invite + threshold_admin_signoff + archive + destroy + create_subset) + 3 P4MC defer (suspend_member + cross_set_share + cross_set_delegate_capability) + 4 post-v1β defer (merge + create_subset body + propose_vote + governance) + RotationTrigger doc-enum from N3** | **LB** | **Y (ops surface)** + **CR (reserve-slots)** | **v1β-LB + v1β-CR** | F17, F-N2-A | benten-membership-set + 4 sibling-traits (Lifecycle/Members/Crypto/Audit) at inherent-impl + audit-log query API | **HIGH (on 11 v1β-LB; MH on 6 CR slots)** |
| **F-N2-C** (NEW M-CONS-v2 from N2) | N2 §3.3 op-17 | **`MembershipSetMetadata { name: Option<String>, description: Option<String>, created_by_human_label: Option<String> }` inside MembershipSetPolicy + admin-gated `rename`/`update_metadata` op (closes M-CONS-v1 §0 Atrium-vs-AtriumFork naming-as-documentation gap)** | LB | Y | v1β-LB | F17, F26 | benten-membership-set | HIGH |

**Net count:** 28 numbered F-rows + 3 N2 lettered (F-N2-A/B/C) = **31 final rows.**
F-A1 + F-A2 absorbed (not separately enumerated).

**Wave-day accumulation** (central; per F-row delta vs M-CONS-v1):
- F1–F16 + F22 + F23 + F24 (baseline + impl-engineering): unchanged from M-CONS-v1 (in-baseline; net –0.6 simplification per M3).
- F17 (MembershipSet primitive crate): unchanged base +3.5–4.5 wave-days; **N2 absorbs +1.0 within canary** (Authority + RBAC + 4 new ops + validators).
- F18 (DUAL-CID): unchanged in F17 wave; –1.5 vs Q3 baseline.
- F19 (generation): unchanged; –2.5 vs U19+U20+U42-L11 baseline.
- F20 (ExecuteWorkflow): +0.3 unchanged.
- F21 (Sealed-Sender slot): –0.2 codepoint registry unchanged.
- F25 (audit-deliverable pack): unchanged –2.0 net; **N1 audit-commitment sub-row +0.1; N4 SSK-comparison §-row +0.5; N4 concurrent-fork tie-break +0.0 (absorbed in §-row).**
- F26 (MembershipSetPolicy): **+1.0 unchanged (D1–D7 walkthrough re-walked under Authority slot per §6 below); N2 generalization absorbs into Wave-MS-PRIMITIVE.**
- **F27 (TransportConfig + iroh-gossip SCOPE-IN per Ben): +5–8 wave-days** (vs M-CONS-v1 +3.5; the +1.5–4.5 delta is the iroh-gossip integration impl per §2 above).
- F28 (RestrictedScopeSet from N1): **+0.95–1.2 wave-days** (per N1 §5.3; folded into Wave-MS-PRIMITIVE canary).
- F-N2-A (Authority + RBAC): **+0.7 wave-days** (in Wave-MS-PRIMITIVE canary; replaces F-A2 ~+0.5 baseline; net +0.2).
- F-N2-B (4 new ops + audit-query + codepoint-reserve slots): **+0.7 wave-days** (in Wave-MS-PRIMITIVE canary; absorbs F-A1 ~+0.2 baseline; net +0.5).
- F-N2-C (MembershipSetMetadata): **+0.2 wave-days** (in Wave-MS-PRIMITIVE canary; new).

**Cumulative M-CONS-v2 wave-day delta vs original ~80-101 baseline:**

`M-CONS-v1 ~–0.1 to –1.3` + `Q1 iroh-gossip-impl +1.5 to +4.5` + `N1 +0.95 to +1.2`
+ `N2 +1.5 to +2.5` (after F-A1/A2 absorption credit) + `N3 ~+2.0` (doc-only) +
`N4 ~+1.0` (doc-only) = **+6.85 to +9.9 wave-days central vs original baseline;
i.e. ~88–109 wave-days central; bracket ~85–114.**

See §8 for full reconciled cost re-estimate.

---

## §4 Task 4 — Final-final Compromise # registry

### §4.1 Canonical collision resolution (final)

Per M-CONS-v1 §3.1 collision resolution + N3 mints + Ben framing for #55:

| # | Origin | Title |
|---|---|---|
| #31-ext | M3 §3 | SIMPLIFIED — per-MembershipSet-Kind retention-window language (revocation reach) |
| #32 | C3 §6.1 Bernstein-Persichetti | ML-KEM Decap CT mitigation (libcrux-ml-kem) |
| #33–#34 | M3 §3 | UNCHANGED — coercion + password-knowledge OUT-OF-SCOPE |
| #35 | M3 §3 | SIMPLIFIED — per-Kind retroactive-decryption framing |
| #36–#40 | M3 §3 | UNCHANGED — RAM/TEE/physical/supply-chain OUT-OF-SCOPE |
| #41 | M3 §3 RENAMED | "MembershipSet-Kind::DeviceMesh compromised-member-on-set" |
| #42 | M3 §3 | UNCHANGED — Layer-C FS-gap; MLS-PQ codepoint-reserve already named |
| #43 | M3 §3 | SIMPLIFIED — per-Kind metadata-leak disclosure |
| #44 | M3 §3 | UNCHANGED |
| #45 | C3 §6.1 (chronological priority) | ML-KEM-768 MAL-BIND-K-CT/K-PK binding-properties |
| #46 | L10 #45 rename | HpkeMultiBase O(N) wire-cost asymptotic > 32 recipients (per-Kind: Atrium 32; DeviceMesh 5; SingleDevice 1) |
| #47 | L11 #45 rename | Collaborative-edit-via-re-drop accepted v1-beta trade-off |
| #48 | Q3 #45 rename + GENERALIZED | **MembershipSet-shape-leak** (shared_key compromise → MembershipSet-fingerprint forward+backward for that generation; per-Kind severity; recovery via fork) |
| #49 | Q3 §7.1 #46 rename | MembershipSet-member-acting-as-storage-host trust-boundary collapse |
| #50 | C3 §6.6 #46 rename | Permanence-stewardship dependency disclosure |
| #51 | C3 §6.9 #47 rename | Tauri NAPI-RS marshaling-boundary side-channels |
| #52 (M-CONS-v1 mint from M5) | M-CONS-v1 §3.1 | **MembershipSet-no-PCS-against-removed-members** — Atrium semantic is fork-on-kick + recipient-set-exclusion; kicked member retains pre-kick K_Atrium and reads pre-kick content forever but cannot read post-kick content. Use-cases requiring stronger semantics defer to post-v1-beta `AtriumWithRotatingGroupKey` per F13. |
| **#53 (M-CONS-v1 mint from M6; M-CONS-v2 NARROWED per Ben Q1 SCOPE-IN)** | M-CONS-v1 §3.1 + Ben Q1 ratification | **TransportConfig-codepoint-reserve-at-v1-beta** — `TransportConfig` wire shape frozen at v1-beta; **iroh-gossip production impl SHIPS at v1-beta** behind `GossipPlusBlobs` codepoint per Ben SCOPE-IN ratification. **Willow + iroh-roq + iroh-live impl remain DEFERRED** to Phase-4-Meta-Composing / Phase-5+. Engine refuses `Reserved` codepoints at v1-beta with `E_TRANSPORT_KIND_UNSUPPORTED`. |
| **#54 (NEW M-CONS-v2 from N3 §7.4 watch-list)** | N3 §7.4 | **Continuous-rotation deferral risk + post-v1-beta `AtriumWithRotatingGroupKey` revisit trigger.** Threshold: any Benten production deployment that materially serves (a) journalism source-protection coordination, (b) AI-agent-rich multi-tenant Atriums where MemberKey ever admits agent-DIDs, (c) HIPAA / PCI-DSS / classified contexts requiring sub-90-day key rotation cadence, (d) production incidents involving silent-undetected-CURRENT-member-compromise. Phase-N revisit when MLS-PQ ciphersuites reach RFC + OpenMLS-PQ stable + DCGKA / p2panda-encryption audit-complete. |
| **#55 (NEW M-CONS-v2 from N3 §2.3 + Ben framing)** | N3 §2.3 + Ben 2026-05-28 framing | **GDPR right-to-be-forgotten honest-architectural-disclosure.** P2P-by-design semantic: a content-addressed P2P system cannot unilaterally erase content that has been replicated to other devices' local stores — that's the substrate guarantee that gives Benten its trustlessness. Applications that need strong-RTBF for legal-compliance contexts layer it on top via per-subject crypto-shredding sub-encryption (separate per-subject keys; delete subject's key shreds only their data). This lives at the Atrium-data layer, NOT MembershipSet. **This is the honest architectural posture, not "a limitation we should fix."** |
| **#56 (NEW M-CONS-v2 OPTIONAL from N3 §3.2)** | N3 §3.2 + §7.3 | **Journalist per-message FS deferral.** Per-message ephemeral-key forward-secrecy (Signal-double-ratchet pattern at per-message granularity) is a SEPARATE design class not served by membership-rotation. Defer to post-v1-beta as a candidate `MembershipSetKind::EphemeralRatchet` codepoint-reserve consideration. Distinct from #54 (continuous-rotation-of-group-key) because per-message-FS is sender-side ephemeral-key discipline, NOT group-key rotation. |

**Total: 25 first-order mints + #31-extension = 26.** Up from M-CONS-v1's 22 mints +
#31-ext = 23. The +3 are #54 + #55 + #56 from N3 (Ben confirmed #55 framing; #56
mint is N3's optional recommendation, **my-pred MINT because it's a structurally-
distinct design class that audit firms would otherwise conflate with #54**).

### §4.2 Per-Compromise transformation summary (M-CONS-v2)

Same as M-CONS-v1 §3.2 with the following M-CONS-v2 additions:
- **#52 unchanged** (M-CONS-v1 mint preserved).
- **#53 NARROWED** (per Ben Q1 SCOPE-IN; iroh-gossip ships at v1-beta).
- **#54 NEW** (M-CONS-v2 mint from N3 watch-list).
- **#55 NEW** (M-CONS-v2 mint from N3 + Ben framing; honest-architectural-disclosure).
- **#56 NEW** (M-CONS-v2 mint from N3 optional; my-pred MINT for audit-clarity).

---

## §5 Task 5 — Final-final invariant set (5 first-order + 3 sub)

| Inv | Title | Transformation (vs M-CONS-v1) | Composes-with |
|---|---|---|---|
| Inv-15 | Signature-bundle-CID identifier discipline (Compromise #30 closure; 3-layer signature decomposition) | UNCHANGED | Inv-16/17/18/19/20 |
| Inv-16 | Envelope-layer-unification + codepoint-dispatch + AAD-binding + strict-decode + canonical-TLV + sender-DID + replay-window | UNCHANGED | Inv-15/18/20 |
| Inv-17 | Hybrid-cryptography-mandatory floor (PQ + classical) | UNCHANGED | Inv-15/16/18 |
| Inv-18a | Codepoint-registry-discipline + IANA-disjoint range + CodepointLifecycle | UNCHANGED | Inv-16 |
| Inv-18b | Metadata-disclosure paired-SealedSender-slot discipline (Am4: Sealed-Sender DEFAULT) | UNCHANGED | Inv-16 |
| Inv-18c | CodepointLifecycle typed-state | UNCHANGED | Inv-16 |
| Inv-19 | Encryption-substrate keying-function CRDT-input discipline (Path-A.5 + MembershipSet) | UNCHANGED | Inv-20, Path-A.5 |
| **Inv-20** | **MembershipSet primitive invariant — 10 clauses (M-CONS-v2 extension from M-CONS-v1's 8)** | **EXTENDED — +2 clauses from N2; clause-h reworded for N1 algebra; clause-c referenced by N4 §3 R-N4-3** | SUBSUMES Inv-19 clause-b; SPECIALIZES Inv-16 multi-recipient sub-clause; INSTANTIATES Inv-18a/b for MembershipSetEncryption codepoint family |

### §5.1 Inv-20 (M-CONS-v2 — 10 clauses; final phrasing)

> **Inv-20 (M-CONS-v2).** Every Benten encrypt-to-N-recipients use site dispatches
> through single `MembershipSet` primitive with `MembershipSetKind` discriminator
> providing:
>
> **(a)** `shared_key: K_Set` distributed via multi-stanza-HPKE-Encap;
> **(b)** FORK-ONLY rotation (AdminKickEpoch folded into FORK-ONLY per M5);
> **(c)** per-stanza AAD-binding tuple `(codepoint, body-CID, sorted-member-DID-list,
> sender_did, stanza-index, member-key-generation, membership_set_id,
> membership_set_generation)` — load-bearing for inter-member non-forgeability per
> N4 §3 R-N4-3 (Benten's analogue to SSK's Ed25519 sig);
> **(d)** per-recipient unlinkability;
> **(e)** generation-CRDT-vector per-member-DID partition;
> **(f)** Path-A.5 K(V) discipline at API boundary (type-restricts payload to
> immutable Version-Node-CID);
> **(g)** TransportConfig wire-format codepoint-reserve (M-CONS-v1) + iroh-gossip
> production impl ships at v1-beta behind `GossipPlusBlobs` codepoint (M-CONS-v2 per
> Ben Q1 SCOPE-IN);
> **(h)** homogeneous-per-Kind MemberKey variant (M-CONS-v1 from M4 §11.5) +
> **`RestrictedScopeSet`-as-disjunction-of-`RestrictedScope` at `Scope::RestrictedSelector`
> arm (M-CONS-v2 from N1) — union-algebra-containment-lifted; orthogonal to
> MembershipSet at the `(MembershipSet, RestrictedScopeSet)` seal-seam tuple**;
> **(i) NEW M-CONS-v2 from N2.** Uniform `authorities: BTreeSet<Authority>` slot
> for signing-authority + `role_assignments: BTreeMap<Did, RoleId>` slot for RBAC
> composition; admin-DID-per-Kind concern resolves at constructor + mutation-op
> validation (runtime), not at type-system level. Per-Kind authority cardinality
> validated by constructors (Atrium ≥1 admin; DeviceMesh exactly-1 user-DID admin;
> SingleDevice exactly-1 self-admin);
> **(j) NEW M-CONS-v2 from N2.** 3-role RBAC (`Admin > Member > Viewer`) + 3-permission
> set (`Read | Write | Admin`) + linear hierarchy (`Admin ⊇ Member ⊇ Viewer`) + UCAN-
> composed intersection-of-allows (RBAC sets coarse floor; UCAN attenuates per-resource
> via `benten-caps::chain_authority`; both must allow for op to proceed). Richer role
> systems = additive codepoint (Phase-4-Meta-Composing+).

### §5.2 Composition checks

**Inv-20 clause-h subsumes Inv-19 clause-b (unchanged from M-CONS-v1).** Verified.

**Inv-20 clause-c is now load-bearing for inter-member non-forgeability per N4
§3 R-N4-3.** Audit-deliverable §-row explicitly names this as Benten's analogue
to SSK's per-message Ed25519 signature. Already implicit in M-CONS-v1; M-CONS-v2
makes it explicit in audit doc.

**Inv-20 clause-h N1 extension preserves Spike H+1.1 opaque-arm rejection.** Each
sub-scope in a `RestrictedScopeSet` is itself a fully-structured `RestrictedScope`
decidable in all 6 dimensions; the change exposes a STRUCTURAL combinator, not
an opaque-refinement-witness arm.

### §5.3 New invariants beyond Inv-20?

**No.** Four candidates considered + rejected:

1. **RBAC-invariant.** The 3-role + 3-permission RBAC composes cleanly under Inv-20
   clause-j; doesn't merit its own first-order invariant.
2. **Authority-cardinality-invariant.** Runtime-validated by constructors; structural-
   to-Inv-20 clause-i; no new first-order invariant.
3. **iroh-gossip-transport-invariant.** Production transport substrate invariant
   would belong in a transport-layer invariant set; out-of-scope for the MembershipSet
   primitive-invariant family.
4. **MembershipEvent-stream-invariant.** Audit-event surface; absorbed into F-N2-A
   scope + Inv-20 clause-j composition; no new first-order invariant.

**Final invariant set:** 5 first-order (Inv-15/16/17/18/19/20) + 3 sub-invariants
of Inv-18 (18a/b/c). Confidence: HIGH.

---

## §6 Task 6 — MembershipSetPolicy D1–D7 walkthrough (under N2 Authority slot)

Re-walks M-CONS-v1 §5 with N2's Authority-slot generalization + N3's RotationTrigger
naming + N4's codepoint-reserve rename. Format: `D# | Decision | Generalization |
my-pred | Rationale`.

| D# | Decision | Generalization (M-CONS-v2) | my-pred | Rationale |
|---|---|---|---|---|
| **D1** | `policy_version_at_seal: u32` in AAD | SIMPLIFIED — same field name + semantic across all 3 Kinds; verifier-side `(membership_set_id, policy_version)` tuple resolution | **RATIFY AS-IS** (rename "atrium" → "membership_set"; field stays `policy_version` not `policy_version_at_seal` per N2's struct shape) | M-CONS-v1 verdict + N2 §4 design + AtriumPolicy `e90900b4` D1 ratification carry over cleanly |
| **D2** | Hybrid grandfather rule (grandfather `valid_until` ceilings + new `refresh_required` applies forward on first verifier-touch) | UNCHANGED per-Kind semantic | **RATIFY AS-IS** | Matrix `m.room.history_visibility` precedent applies identically; no Kind-specific behavior |
| **D3** | `refresh_required_secs: Option<u64>` v1-beta-CODEPOINT-RESERVE + impl at Phase-4-Meta-Composing | **CLARIFIED per N3 §5.3** — field denotes membership-**attestation re-signing** cadence, NOT K_Set rotation cadence. MembershipSet-Spec doc note pinned at v1-beta. | **RATIFY AS-IS + add clarification note** | UX-coupled per-Kind impl scheduling identical; clarification doc-note prevents the "is this K_Set rotation?" misread per N3 §5.3 |
| **D4** | Admin entity per-Kind | **REPLACED by N2 Authority slot.** Was `KindPolicyAdminEntity` enum (M-CONS-v1 §5.1) with 3 variants (`AtriumAdminDid` / `DeviceMeshUserPrincipal` / `SingleDeviceNone`); now uniform `authorities: BTreeSet<Authority>` with per-Kind cardinality validated by constructors. | **RATIFY N2 GENERALIZATION** (overrides M-CONS-v1 §5.1 KindPolicyAdminEntity) | Cleaner shape; absorbs the "Admin/User/Self is the same primitive concept" insight Ben caught (N2 §1.1); enables clean threshold-admin growth via set-cardinality (no new variant) per D7; runtime-validation cost bounded |
| **D5** | Default policy = `permissive_default()` (no Atrium-level ceiling) | UNCHANGED per-Kind semantic; per-Kind constructor `MembershipSetPolicy::permissive_default(kind: MembershipSetKind)` | **RATIFY AS-IS** | No semantic change beyond per-Kind constructor signature; absorbed into N2's per-Kind constructors |
| **D6** (deferred) | Refresh source mechanism = OCSP-style fresh-attestation pull | UNCHANGED per-Kind; per-Kind authority-resolution rule (Atrium = `refresh_attestation_authorities` set drawn from `authorities` field; DeviceMesh = user's principal-DID from `authorities`; SingleDevice = N/A) | **RATIFY DEFERRAL** with per-Kind authority-resolution rule pinned now (Phase-4-Meta-Composing impl) | Deferral preserves; the per-Kind authority rule is a 1-paragraph addendum in MembershipSet-Spec; absorbed into Authority slot |
| **D7** (deferred) | Admin = single-DID or threshold-of-N | **CLEANER UNDER N2** — threshold-admin = `authorities.len() > 1` for Atrium-Kind via Authority set-cardinality + `ThresholdAdminSpec` field at v1-beta-CR; DeviceMesh = user IS sole-admin by construction; SingleDevice = N/A | **RATIFY DEFERRAL + pin per-Kind variant + Ratify N2 ThresholdAdminSpec slot at v1β-CR** | Threshold-of-user's-own-devices for DeviceMesh would be a device-recovery concern, not admin concern; cleanly bounded by N2's set-cardinality shape |

### §6.1 Net D1–D7 walkthrough under M-CONS-v2

- **D1/D2/D3/D5/D6:** RATIFY AS-IS (D3 with added clarification doc-note per N3).
- **D4:** RATIFY N2 GENERALIZATION (overrides M-CONS-v1 §5.1 KindPolicyAdminEntity).
- **D7:** RATIFY DEFERRAL + pin per-Kind variant (Atrium-Kind-only threshold-admin
  at v1-beta-CR via `ThresholdAdminSpec` struct in `MembershipSetPolicy`).

**my-pred bundle:** 7-decision RATIFY-bundle (D1/D2/D3/D5/D6 as-is + D4 N2-generalization
+ D7 defer-with-pin). Confidence: HIGH on D1/D2/D3/D5; MED-HIGH on D4/D6/D7 (D4 is
the substantive change; D6/D7 deferrals depend on Authority-slot composition).

### §6.2 MembershipSetPolicy crate placement (unchanged from M-CONS-v1 §5.2)

Lives INSIDE `benten-membership-set` crate. Wave-G (AtriumPolicy) DELETED; folded
into Wave-MS-PRIMITIVE canary.

---

## §7 Task 7 — Final wave-sequencing

### §7.1 Waves (M-CONS-v2)

Per M-CONS-v1 §6 with M-CONS-v2 updates absorbed.

| Wave | Composition | Est. wave-days (M-CONS-v2) | Net Δ vs original ~80-101 baseline |
|---|---|---|---|
| **Wave 0** (tracked-doc cascade; Ben pre-authorized) | — | — | 0 |
| **Wave-MS-PRIMITIVE** (canary + 4 parallel + final) | `benten-membership-set` crate canary lands FIRST: F17 + F18 + F19 + F26 + F28 (N1 RestrictedScopeSet) + F-N2-A (Authority + RBAC) + F-N2-B (11 v1-beta-LB ops + 6 CR slots + RotationTrigger doc-enum) + F-N2-C (MembershipSetMetadata) + 4 sibling-traits + MembershipEvent enum + Golden vectors + kani harness + property tests + NAPI bindings (MembershipSetHandle) + TS mirror. **Canary ~4.5–6 wave-days sequential.** **4 fan-out sub-waves parallel ~5.5–6.5 wave-days clock-time ~1.5 wave-days.** **Final integration ~0.7 wave-days.** | **~10.5–13** wave-days (vs M-CONS-v1 ~9–10; +1.5–3 absorbs N1+N2 inside canary) | **+11.5–14 NEW** offset by **–12.5–14.75 absorbed** (Wave-G + Wave-Q3 + L11-CRDT-partial) = **–1 to +1 net** |
| **Wave-MS-TRANSPORT** (iroh-gossip SCOPE-IN per Ben Q1) | M6 §10.1 10-items + iroh-gossip integration impl per §2 above | **~5–8 wave-days** (vs M-CONS-v1 ~3.5 codepoint-only; +1.5–4.5 for iroh-gossip impl per Ben) | **+5–8 NEW** |
| **Wave-C** (Layer-C single-recipient drop flow) | reduced per M-CONS-v1 | reduced | **–1.5** |
| **Wave-D** (DAK + device-link + remote-permission) | partial reduction per M-CONS-v1 | reduced | **–1.75** |
| **Wave-G** (AtriumPolicy) | DELETED — folds into Wave-MS-PRIMITIVE canary | — | **–5 to –7** |
| **Wave-Q3** (Q3 Option D) | ABSORBED into Sub-wave-MS-D | — | **–5.75** |
| **Wave-L11-CRDT** (L11 U41–U44 + Inv-19) | PARTIALLY ABSORBED | partial | **–4.5** |
| **Net wave-day delta** | | | **central +6.85 to +9.9 vs original baseline** |

**Net wave-count Δ:** +2 NEW waves (MS-PRIMITIVE + MS-TRANSPORT), 1 wave DELETED
(Wave-G), 3 waves shrink (Wave-C + Wave-D + Wave-L11-CRDT). **Net wave-count Δ = +1**
(unchanged from M-CONS-v1).

### §7.2 Canary discipline (unchanged from M-CONS-v1)

**Wave-MS-PRIMITIVE MUST go canary-first** because `benten-membership-set` is the
owner of the new primitive API the 4 fanout sites consume. Parallel dispatch of all
4 sub-waves without canary risks 4-way merge-conflict on the primitive's API shape.
**N2's Authority + RBAC + 4 new ops absorbed inside canary scope per §3 F-N2-A/B
absorption** — the canary now ships substantially more code, but the canary-first
discipline is unchanged.

### §7.3 Wave-MS-PRIMITIVE canary contents (M-CONS-v2 enumeration)

Per M-CONS-v1 §6.2 + N1/N2 absorptions:

**Canary (sequential first; ~4.5–6 wave-days):** `benten-membership-set` crate lands
first. Contents:

- `MembershipSet { id, kind, members, shared_key, policy, generation,
  parent_membership_set_id, created_at_hlc, membership_attestation,
  transport_config }` struct (N2 §4.1 canonical shape).
- `MembershipSetKind { Atrium, DeviceMesh, SingleDevice }` (EXACTLY-3-arm; §15.c
  HALT-AND-SURFACE-TO-BEN; `AtriumWithRotatingGroupKey` codepoint-reserve slot per
  N4 R-N4-1).
- `MemberKey { UserDid | DeviceDid | LocalDevice }` (homogeneous-per-Kind invariant).
- `MembershipSetPolicy` with `authorities: BTreeSet<Authority>` + `role_assignments:
  BTreeMap<Did, RoleId>` + `kind_policy: KindPolicySubfields` + `refresh_required_secs`
  + `threshold_admin: Option<ThresholdAdminSpec>` + `metadata: MembershipSetMetadata`.
- `Authority` struct + `RoleId` enum + `PermissionSet` bitflags + `check_permission`
  helper composing RBAC + UCAN intersection-of-allows.
- **11 v1-beta-LB ops** (per Ben-ratified N2 §3.1 classification): `create`,
  `add_member`, `remove_member`, `change_role`, `member_key_rotation`, `self_leave`,
  `rename`/`update_metadata`, `encrypt_to_set`, `dedup_blind_cid`, `fork`,
  `update_policy` (+ `update_policy_value` sugar; + `audit_log_query`).
- **6 codepoint-reserve ops:** `propose_member`/`accept_invite` flow, `threshold_admin_signoff`,
  `archive`, `destroy`, `create_subset`.
- **4 sibling-trait families** (Lifecycle / Members / Crypto / Audit) implemented
  alongside inherent-impl per N2 §3.2.
- **`MembershipEvent` typed-enum** (F-N2-A absorbs M-CONS-v1 F-A2) emitted by all
  11 v1-beta-LB ops + carrying `RotationTrigger` from N3.
- **`RestrictedScopeSet`** (N1 F28) at the grant boundary in `benten-caps::scope`
  (canary lands in benten-membership-set scope of work; the `benten-caps` mod lands
  alongside as cross-crate edit within the canary wave).
- Per-Kind constructors with cardinality + invariant validation.
- `TransportConfig` types + codepoint-dispatch layout (M6 §5.2 + §10.3).
- `MultiStanzaHpkeEnvelope` shape (M2 §2.2).
- Golden vectors + kani harness + property tests (F25).
- NAPI bindings (`MembershipSetHandle` per F24 U34-expansion).
- TS mirror at `packages/engine/src/membership.ts` + `errors.generated.ts` (with 4
  new ErrorCode entries per F-N2-A §2.7).

**Fan-out (4 parallel agents AFTER canary merges; ~5.5–6.5 wave-days total at
parallel clock-time of ~1.5–1.7 wave-days):**

- Sub-wave-MS-A: `benten-crypto-suite` migrates HpkeMultiBase usage to MembershipSet (~1.2 day)
- Sub-wave-MS-B: `benten-drop` migrates Drop-bundle multi-recipient to MembershipSet (~1.7 day)
- Sub-wave-MS-C: `benten-engine` migrates multi-device-key-wrap + RBAC seam to MembershipSet (~1.7 day)
- Sub-wave-MS-D: `benten-id` (or wherever Atrium-membership lives) migrates K_Atrium-distribution to MembershipSet (~1.7 day)

**Wave-MS-FINAL (single agent; ~0.7 day):** cross-crate integration test + final
goldens + TS-side full integration + concurrent-fork-tie-break audit-doc.

### §7.4 Wave-MS-TRANSPORT contents (M-CONS-v2; SCOPE-IN per Ben Q1)

Per M6 §10.1 + §2 above iroh-gossip integration delta. Dispatches AFTER
Wave-MS-PRIMITIVE.

Composition:
- `TransportConfig` types per M6 §5.2 (in `benten-engine` per M6 §10.1 item 1).
- Default-selection logic per M6 §6.1.
- `E_TRANSPORT_KIND_UNSUPPORTED` error code per §3.5g cross-language rule-mirror.
- **iroh-gossip production integration** (M-CONS-v2 new): peer-discovery + topic-
  per-MembershipSet + multi-stanza-HPKE-envelope publishing/subscribing + NAT-traversal
  + relay-fallback + e2e pin per pim-2 substantive-arm.
- RED-PHASE staged-pin un-ignore for production gossip path per pim-12 §3.6e.
- V1-FROZEN-INTERFACE.md §15.6 row per M6 §10.4.
- Compromise #53 disclosure updated to NARROWED form per §4.

**Wave-MS-TRANSPORT cost: ~5–8 wave-days central; bracket ~4.5–9.**

---

## §8 Task 8 — Final cost re-estimate

### §8.1 Baseline → M-CONS-v2 final

Per brief Task 8 arithmetic check:

| Layer | Wave-day delta | Cumulative |
|---|---|---|
| Original baseline (per critique-round + 9-eyes consolidator §6.5) | ~80–101 | ~80–101 |
| M-CONS-v1 central (–4.3 to –5.5 simplification offset by +3.5 M6 + +0.7 F-A1/F-A2) | ~–0.1 to –1.3 | ~78–100 (M-CONS-v1) |
| **Q1 iroh-gossip SCOPE-IN (Ben ratification; Wave-MS-TRANSPORT 3.5 → 5–8)** | **+1.5 to +4.5** | ~79.5 to 104.5 |
| **N1 RestrictedScopeSet (F28; absorbed into Wave-MS-PRIMITIVE canary)** | **+0.95 to +1.2** | ~80.5 to 105.7 |
| **N2 hybrid (F-N2-A/B/C; absorbs F-A1+F-A2 ~–0.7 credit + adds Authority + RBAC + 4 ops + metadata ~+2.2–3.2)** | **+1.5 to +2.5 net** | ~82 to 108.2 |
| **N3 sharpenings (doc-only RotationTrigger + clarifications + #54/#55/#56 mints)** | **~+2.0** | ~84 to 110.2 |
| **N4 doc-only refinements (rename + chained-mode codepoint + SSK-comparison §-row + concurrent-fork tie-break)** | **~+1.0** | ~85 to 111.2 |

**Final M-CONS-v2 central estimate: ~85–111 wave-days** (vs original baseline
~80–101); **+5 to +10 wave-days central; bracket ~83–115.**

**Net delta vs original baseline:** **+6.85 to +9.9 wave-days central** (i.e. the
N-refinements + Ben SCOPE-IN cost ~5–10 wave-days of incremental scope; M-CONS-v1's
net-neutral simplification is preserved, but Ben's substantive scope expansions
(iroh-gossip impl + RBAC + 4 new ops + RestrictedScopeSet + audit-deliverable
expansions) compound to a net positive of ~5–10 wave-days.

**vs M-CONS-v1 baseline (~78–100):** **+7 to +11 wave-days net** — the cumulative
incremental scope from Q1 SCOPE-IN + N1 + N2 + N3 + N4.

### §8.2 LOC delta vs M-CONS-v1

Per M-CONS-v1 §7.2 baseline + N additions:
- M-CONS-v1 baseline: ~4400–5100 LOC.
- **N1 RestrictedScopeSet:** +~160 LOC (~80 type + ~30 codepoint-migration + ~30
  TS mirror + ~20 cite-drift scanner).
- **N2 hybrid (after F-A1/F-A2 absorption credit):** +~200 LOC net (~150
  benten-membership-set + ~50 TS + 4 ErrorCode entries; offsets ~50 LOC saved by
  collapsing KindPolicyAdminEntity 3-variant enum into uniform Authority slot).
- **N3 doc-only:** ~+50 LOC (RotationTrigger doc-enum stub + MembershipEvent reason-
  field plumbing inside F-N2-A).
- **N4 doc-only:** ~+20 LOC (codepoint-reserve rename + sub-slot constants).
- **Q1 iroh-gossip SCOPE-IN:** **+~1500–2500 LOC** (peer-discovery + topic mgmt +
  envelope pub/sub + NAT-traversal + relay-fallback + integration tests). This is
  the LOC dominant addition vs M-CONS-v1.

**M-CONS-v2 final LOC central: ~6300–7800.** Δ vs M-CONS-v1 ~+1900–2700; dominated
by iroh-gossip integration.

### §8.3 Audit-page delta vs M-CONS-v1

Per M-CONS-v1 §7.3 (–6 to –10 from baseline) + M-CONS-v2 additions:
- **N1 audit-commitment sub-row:** +0 pages (audit_commitment is field-level note).
- **N2 RBAC §-add:** +3–5 pages (SECURITY-POSTURE.md + THREAT-MODEL.md).
- **N3 doc-only:** +1 page (RotationTrigger §-add + #54 + #55 + #56 disclosures).
- **N4 SSK-comparison + concurrent-fork tie-break + R-N4-3 audit-deliverable §-add:** +1–2 pages.
- **Q1 iroh-gossip SCOPE-IN:** +0–1 page (most of the doc-impact is in CRYPTO-CODEPOINTS.md
  + TRANSPORT-CONFIG.md which were already counted in F27 baseline; +1 page for
  e2e-pin documentation of the production gossip path).

**M-CONS-v2 audit-page net:** ~–1 to –3 pages vs original baseline (i.e. the
M-CONS-v1 –6 to –10 savings are partly consumed by N2 + N3 + N4 + Q1 additions,
but the consolidation wins still net positive on audit-page count).

### §8.4 Cross-cutting consolidation wins (preserved from M-CONS-v1)

All unchanged:
- 4 fanout codepoints → 1 MembershipSetEncryption family with Kind discriminator.
- 4 parallel implementations → 1 substrate crate.
- 4 amendment-clusters → 1 set of F-rows.
- AtriumPolicy + MembershipSetPolicy unified (Wave-G folded).
- Q3 Option D K_Atrium key-management absorbed into MembershipSet shared_key.
- Path-A.5 K(V) discipline enforced at MembershipSet API boundary (compile-time).
- DUAL-CID-on-wire clarified.

**M-CONS-v2 adds:**
- 3-variant KindPolicyAdminEntity enum → 1 uniform Authority slot (N2; +clean
  threshold-admin growth path).
- Implicit admin-vs-member binary → explicit 3-role RBAC (N2; +UCAN-composition
  seam).
- Per-Kind missing UX name → uniform MembershipSetMetadata slot (N2; closes
  Atrium-vs-AtriumFork question).
- Internal `combinators::union` → grant-time RestrictedScopeSet primitive (N1;
  closes asymmetric-shape limitation via already-shipping structural lever).
- AtriumWithCGKA misframing → AtriumWithRotatingGroupKey neutral name (N4; un-bakes
  future-shape).

**The wave-day delta is positive (+5–10); the structural consolidation is substantially
more substantial than M-CONS-v1's already-meaningful consolidation.**

---

## §9 Task 9 — Open Ben-call decisions remaining (post-N-refinement)

Most M-CONS-v1 Ben-calls are now resolved by N-refinement outputs or Ben's intervening
ratifications. Walking the M-CONS-v1 §9 9-decision list + adding M-CONS-v2-new
decisions:

### §9.1 M-CONS-v1 Ben-calls — disposition under M-CONS-v2

| M-CONS-v1 § | Decision | Status under M-CONS-v2 |
|---|---|---|
| §9.1 | MultiRecipientSealing rename | **RESOLVED** (M-CONS-v1 ratified; N4 re-confirmed) |
| §9.2 | TransportConfig codepoint-reserve | **OVERRIDDEN** by Ben Q1 SCOPE-IN ratification — iroh-gossip ships at v1-beta; #53 narrowed |
| §9.3 | D1–D7 walkthrough | **RESOLVED** under N2 Authority slot generalization per §6 above |
| §9.4 | Compromise #52 + #53 mints | **RESOLVED** (M-CONS-v1 ratified; #53 narrowed by Q1 SCOPE-IN) |
| §9.5 | Inv-20 mint | **RESOLVED — EXTENDED** to 10 clauses (M-CONS-v1's 8 + N2's 2 + N1's clause-h extension) |
| §9.6 | Wave-MS-PRIMITIVE canary-first | **RESOLVED** (M-CONS-v1 ratified; M-CONS-v2 absorbs N1/N2 inside canary) |
| §9.7 | F-A1 + F-A2 amendments | **RESOLVED — ABSORBED** into F-N2-A + F-N2-B per N2 §5.3 |
| §9.8 | Sibling-trait vs inherent-method | **RESOLVED** under N2 §3.2 — ship 4 sibling-traits ALONGSIDE inherent-impl at v1-beta (free optionality; v1.x refactor preserved) |
| §9.9 | Atrium vs AtriumFork name | **RESOLVED** (M-CONS-v1 KEEP `Atrium`; N4 re-confirmed; AtriumWithRotatingGroupKey codepoint-reserve names the fork-semantic explicitly for the future-codepoint case) |

### §9.2 M-CONS-v2 NEW Ben-calls

| # | Decision | my-pred | Confidence |
|---|---|---|---|
| **N1-D1** | Ratify N1 Option H NESTED-SPEC (`RestrictedScopeSet` + audit-commitment sub-row); add F28; extend Inv-20 clause-h; fold into Wave-MS-PRIMITIVE canary | **RATIFY** | HIGH |
| **N2-D1** | Ratify N2 hybrid (uniform `authorities` slot + 3-role RBAC + 4 new v1-beta-LB ops + MembershipSetMetadata); mint F-N2-A/B/C; absorb F-A1+F-A2; extend Inv-20 to 10 clauses; override M-CONS-v1 §5.1 KindPolicyAdminEntity | **RATIFY** (per Ben already ratifying the 24-op classification as-is) | HIGH |
| **N3-D1** | Mint Compromise #54 (continuous-rotation deferral) + #55 (GDPR-RTBF P2P-by-design honest-disclosure per Ben framing) + #56 (journalist per-message FS deferral; optional but my-pred MINT for audit clarity) + RotationTrigger doc-enum + refresh_required_secs clarification | **RATIFY-BUNDLE** | HIGH on #54+#55; MED-HIGH on #56 optional |
| **N4-D1** | Apply codepoint-reserve rename (AtriumWithCGKA → AtriumWithRotatingGroupKey per Ben) + add RotatingGroupKeyChainedMode sub-slot (per Ben Option B) + SECURITY-POSTURE §-row + concurrent-fork tie-break rule | **RATIFY** (Ben already ratified rename + Option B; remaining mechanical) | HIGH |
| **Q1-D1 (open clarification, not full ratification)** | Confirm Wave-MS-TRANSPORT scope-in is iroh-gossip-IMPL-only (not also Willow / iroh-roq / iroh-live) at v1-beta | **CONFIRM** Ben Q1 SCOPE-IN ratification covers iroh-gossip only; Willow/iroh-roq/iroh-live remain codepoint-reserve | HIGH (per brief framing; surface as clarification) |

### §9.3 Substantive open Ben-calls remaining

After applying all N-refinements + Ben ratifications, the substantive remaining
calls are:

1. **N3 Compromise #56 mint (optional).** N3 listed as "optional"; my-pred MINT for
   audit clarity. **OPEN — surface to Ben as MINT or SKIP.**
2. **N3 `refresh_required_secs` field rename.** N3 §8.2 flagged: "rename to
   `attestation_refresh_secs` if M2 hasn't frozen yet." My-pred KEEP existing name
   + add clarification note (rename would be a wire-format change). **OPEN —
   surface to Ben as KEEP+CLARIFY or RENAME (since M2 is pre-freeze the rename is
   technically affordable).**
3. **N2-D8 (sibling-trait extraction).** N2 §6.8 my-pred KEEP-DEFAULT inherent-
   impl + ship 4 sibling-traits alongside. Defensible either way. **OPEN — surface
   to Ben as ALONGSIDE (default) or TRAITS-ONLY (refactor) or INHERENT-ONLY (M2's
   original shape).**
4. **Q1-D1 clarification.** Confirm scope-in = iroh-gossip only; Willow/iroh-roq/
   iroh-live stay codepoint-reserve. **OPEN — surface to Ben as clarification of
   Q1 SCOPE-IN scope.**

**Net open Ben-calls (M-CONS-v2):** 4 (N3 #56 mint; N3 rename clarification;
N2-D8 traits placement; Q1-D1 transport-scope-in clarification). None are arch-
fork-class per `feedback_surface_arch_decisions_under_auth.md`. All 4 surfaceable
in one Ben-pass.

---

## §10 Task 10 — R0 plan-doc skeleton updated for M-CONS-v2

Per M-CONS-v1 §8 17-section structure + N1/N2 additions. (Structure-only; M-R0
owns the substantive content per ADDL pipeline.)

```markdown
# F-full R0 plan-doc — Phase-4-Meta-Core (MembershipSet-unified, M-CONS-v2)

## §1 Architectural framing
- 4-Layer model + MembershipSet primitive as the central novel structural addition
- v1-beta freeze posture; codepoint-reserve discipline; HARD RULE 12 disposition
- Ben-ratified decisions inventory (Q1 / Q2 / Q3-DUAL-CID / Q4 / Am4 / Path-A.5
  + M-CONS-v2 ratifications: iroh-gossip-SCOPE-IN + N2-24-op-classification +
  #55-GDPR-RTBF-framing + Option-B-chained-mode + AtriumWithRotatingGroupKey-rename)
- This R0 supersedes the 9-eyes-consolidated-registry + M-CONS-v1 as the canonical scope

## §2 MembershipSet primitive
### §2.1 The primitive (typed-variant + uniform Authority slot + 3-role RBAC)
- `MembershipSetKind { Atrium, DeviceMesh, SingleDevice }` EXACTLY-3 + `AtriumWithRotatingGroupKey` codepoint-reserve slot per N4 R-N4-1
- Struct shape per N2 §4.1 (canonical Rust) with `authorities: BTreeSet<Authority>`
- Canonical CBOR wire shape per N2 §4.2
- 11 v1-beta-LB ops (per Ben-ratified N2 §3.1 classification)
- Per-Kind specialization table
- MemberKey typed-per-Kind variant
- Type-system + constructor invariants enforced (per N2 §4.4)
- Composition with 4-identity-concepts tree

### §2.2 New `benten-membership-set` crate (15th workspace crate)
- Crate boundary + dependency graph
- Public type surface (Authority + RoleId + PermissionSet + MembershipSetMetadata)
- Operations + per-Kind dispatch
- Sibling-trait families (4 traits: Lifecycle / Members / Crypto / Audit) at inherent-impl
- NAPI surface (MembershipSetHandle)
- TS-side mirror with RoleId enum + ROLE_PERMISSIONS lookup

### §2.3 Inv-20 minting (10-clause M-CONS-v2 invariant)
- 10-clause invariant per §5 above
- Composition with Inv-15/16/17/18/19
- Enforcement plan (crate-discipline scanner + cite-drift extension + property tests)

### §2.4 Wire-format codepoint family
- `MEMBERSHIP_SET_ENCRYPTION = 0x6380` codepoint family with `MembershipSetKind: u8`
- `MembershipSetEncryptionSealedSender = 0x6390` paired slot per F21
- `AtriumWithRotatingGroupKey` codepoint slot at v1-beta-CR per F13 + N4
- `RotatingGroupKeyChainedMode` sub-slot per N4 R-N4-2 + Ben Option-B

### §2.5 (NEW from N2) Authority + RoleAssignments + RBAC
- `Authority { did, sig_pubkey, role, admitted_at_hlc }` struct
- `authorities: BTreeSet<Authority>` uniform slot (replaces M-CONS-v1 §5.1 KindPolicyAdminEntity)
- `RoleId { Admin, Member, Viewer }` + `PermissionSet { READ, WRITE, ADMIN }`
- Linear hierarchy + UCAN-composed intersection-of-allows
- `role_assignments: BTreeMap<Did, RoleId>` slot

### §2.6 (NEW from N2) 3-role RBAC composition with UCAN
- `check_permission(actor_did, op, ucan_chain)` flow
- RBAC sets coarse floor; UCAN attenuates per-resource
- 4 new ErrorCode entries: E_RBAC_DENY, E_UCAN_DENY, E_ROLE_NOT_ASSIGNED, E_AUTHORITY_CARDINALITY

### §2.7 (NEW from N2) 11 v1-beta-LB ops enumeration + 6 v1-beta-CR + 3 P4MC + 4 post-v1β
- Full 24-op classification per N2 §3.1 + Ben ratification

### §2.8 (NEW from N3) RotationTrigger doc-enum
- 6 trigger categories (AdminKick / MemberDepart / AdminDepart / DeviceRevoke / CompromiseResponse / PeriodicHygiene)
- All triggers map to same v1-beta wire mechanism (MembershipSet::fork + MembershipEvent reason)
- refresh_required_secs semantic clarification (attestation-refresh, NOT K_Set rotation)

## §3 Layer-A vault design (K_principal store) — unchanged from M-CONS-v1

## §4 Layer-B per-Node AEAD (Path-A.5 immutable Version-Node-CID) — unchanged

## §5 Layer-C encrypt-to-recipient drops
- Per M-CONS-v1 + MultiRecipientSealing
- (NEW from N4) SECURITY-POSTURE.md SSK-comparison §-row + AAD-binding-of-sender_did clause-c naming + concurrent-fork tie-break rule

## §6 Layer-D DAK + multi-device-key-wrap + remote-permission-call — unchanged

## §7 Wire format
### §7.1 EncryptedEnvelope — unchanged
### §7.2 DUAL-CID model (Q3 ratification) — unchanged
### §7.3 Amendments F1–F28 + F-N2-A/B/C — per §3 above (this M-CONS-v2 doc)
### §7.4 (NEW from N1) RestrictedScopeSet at Scope::RestrictedSelector arm
- backward-compat codepoint-dispatch
- containment lift
- audit_commitment Option-G-AUDIT-SIBLING

## §8 MembershipSetPolicy (D1–D7 per-Kind under Authority slot per §6 above)

## §9 TransportConfig (iroh-gossip SCOPE-IN at v1-beta per Ben Q1)
- TransportConfig struct + TransportKind + SubInheritPolicy + TransportExtension enums
- Codepoint-dispatch layout
- Default-selection logic
- E_TRANSPORT_KIND_UNSUPPORTED error code
- (NEW M-CONS-v2 from Ben Q1) iroh-gossip production integration impl ships at v1-beta
- Willow / iroh-roq / iroh-live impl deferred per F27 + #53-narrowed
- V1-FROZEN-INTERFACE.md §15.6 row per M6 §10.4

## §10 ml-kem crate choice — unchanged (libcrux-ml-kem per Q1)

## §11 docs/THREAT-MODEL.md skeleton
- Per-MembershipSet-Kind threat-model rows
- Cross-Kind composition + storage-host insider threats (#49)
- Coercion / xkcd-538 out-of-scope per #33
- (UPDATED M-CONS-v2) MembershipSet-shape-leak disclosure per #48
- (UPDATED M-CONS-v2) MembershipSet-no-PCS-against-removed-members disclosure per #52
- (NEW M-CONS-v2) Continuous-rotation deferral risk per #54
- (NEW M-CONS-v2) GDPR-RTBF P2P-by-design honest-disclosure per #55 + Ben framing
- (NEW M-CONS-v2) Journalist per-message FS deferral per #56 (if minted)
- (NEW M-CONS-v2) RBAC + UCAN composition trust-boundaries per N2 §2

## §12 docs/SECURITY-PROOFS.md skeleton
- (UPDATED M-CONS-v2) Inv-20 10-clause properties (was 8-clause)
- AAD-binding tuple injectivity (kani harness per F25) — clause-c LOAD-BEARING per N4
- per-recipient unlinkability (Inv-20 clause-d)
- (NEW M-CONS-v2) RestrictedScopeSet containment-algebra lift proofs (N1)
- (NEW M-CONS-v2) RBAC × UCAN intersection-of-allows soundness sketch (N2)

## §13 docs/CRYPTO-CODEPOINTS.md skeleton
- MembershipSetEncryption codepoint family table per F8
- Sealed-Sender paired-slot per F21 + Inv-18b
- (UPDATED M-CONS-v2) AtriumWithRotatingGroupKey codepoint-reserve per F13 + N4 (renamed)
- (NEW M-CONS-v2) RotatingGroupKeyChainedMode sub-slot per N4 R-N4-2 + Ben Option-B
- TransportKind + TransportExtension codepoint layout per M6 §10.3 + F27
- (NEW M-CONS-v2) RoleId codepoints (0/1/2) + PermissionSet flag layout + 1<<3..1<<31 reserved
- (NEW M-CONS-v2) Authority CBOR encoding + per-Kind cardinality constraints
- IANA-disjoint range per F8

## §14 docs/ENVELOPE-V1-SPEC.md skeleton
- EnvelopePayload variants (incl. MembershipSetEncryption)
- BindingContext variants (incl. MembershipSetSeal)
- DUAL-CID-on-wire shape per F18
- DAG-CBOR canonical encoding per F23
- AAD layout per F4 + F14
- Replay-window per-Kind exclusion table per F5
- (NEW M-CONS-v2) RestrictedScopeSet encoding + codepoint-dispatch backward-compat per F28

## §15 Wave-sequencing (per §7 above)
- Wave 0: tracked-doc cascade (Ben pre-authorized)
- Wave-MS-PRIMITIVE (canary + 4 parallel + final) ~10.5–13 wave-days
- Wave-MS-TRANSPORT (iroh-gossip SCOPE-IN per Ben Q1) ~5–8 wave-days
- Reduced/folded waves (C/D/G/Q3/L11-CRDT)

## §16 Cost estimate (per §8 above)
- Wave-days: ~85–111 (vs original baseline ~80–101); net +5 to +10 central
- LOC: ~6300–7800 (consolidated; iroh-gossip-impl dominant)
- Audit pages: ~–1 to –3 net
- Cross-cutting consolidation wins (M-CONS-v1 + M-CONS-v2 additions; un-counted)

## §17 Decisions recorded
- Q1: libcrux-ml-kem (ratified) + **iroh-gossip transport SCOPE-IN at v1-beta** (ratified M-CONS-v2)
- Q2: BE codepoint endianness migration (ratified; in aead.rs)
- Q3: DUAL-CID simplification (ratified; per F18)
- Q4: HPKE-11-KE commit now; JOSE-emit adapter NAMED-deferred (ratified)
- Q5: tbd (if any open; surface to Ben)
- Am4: Sealed-Sender DEFAULT (ratified; per F21)
- Path-A.5 (ratified; Path-B DISAGREE-WITH-REASONING)
- AtriumPolicy D1–D7 (ratified per §6 above; D6/D7 deferred with named destination + per-Kind variant pinned)
- MembershipSet Kind decisions (M-CONS-v1 inherited + M-CONS-v2 additions):
  - 3-arm EXACTLY-3 enum (Atrium/DeviceMesh/SingleDevice)
  - AtriumWithRotatingGroupKey codepoint-reserve at v1-beta (renamed per N4)
  - RotatingGroupKeyChainedMode sub-slot codepoint-reserve (Ben Option-B per N4)
  - Forkability Atrium-only (runtime-reject; sibling-trait reserve v1.x)
  - DUAL-CID-on-wire; plaintext_cid_local LOCAL-ONLY
  - MultiRecipientSealing (not CGKA-LITE) naming
  - **TransportConfig + iroh-gossip impl SHIPS at v1-beta per Ben Q1** (Willow/iroh-roq/iroh-live remain codepoint-reserve)
  - F-A1 + F-A2 absorbed into F-N2-A + F-N2-B
  - **RestrictedScopeSet (N1) F28 — sub-graph asymmetric-shape closure**
  - **Authority + 3-role RBAC + 4 new v1-beta-LB ops + MembershipSetMetadata (N2) F-N2-A/B/C**
  - **RotationTrigger doc-enum + refresh_required_secs clarification (N3)**
  - **Compromise #54 + #55 + #56 (N3 + Ben framing)**
  - **SSK-comparison §-row + concurrent-fork tie-break (N4 R-N4-3)**
```

---

## §11 Task 11 — Pattern-induction meta-findings (M2–M6 + N1–N4 aggregate)

What does the M2–M6 + N1–N4 panel reveal in aggregate that individual specialists
missed? Cross-panel patterns worth codifying.

### §11.1 Pattern P11 — "Ben's refinement-hints can catch generality the convergent panel missed"

M-CONS-v1 §10.1 (Pattern P1) codified "convergent shape across N specialists = HIGH-
confidence ratification signal." N2 §7.2 (Pattern N2-P2) sharpened this: convergence
signals strong but not perfect. The M-CONS-v1 panel + 3 catalogers all-typed the
admin-DID per-Kind shape as 3 different enum variants; Ben caught the real
generalization (Authority slot). **This is structurally important:** N1–N4 was
spawned precisely BECAUSE Ben suspected the convergent panel might have missed a
generalization.

**Codify (sharpens M-CONS-v1 P1).** When convergent specialist outputs land, surface
to Ben + ask "do you see a generalization the panel missed?" as a first-class step
BEFORE the critique-round. The post-M-CONS-N1/N2/N3/N4 pipeline IS this discipline
operationalized. Codify as a phase-close pattern: every multi-specialist
consolidation gets one explicit Ben-pass for generalization-catch BEFORE
critique-round. The cost is one round of refinement-spawning + integration; the
payoff is catching real generalizations the convergent panel missed.

### §11.2 Pattern P12 — "Already-shipping combinator surfaces are the cheapest source of grant-layer expressiveness"

N1's load-bearing finding was that `combinators::union` already ships (g-core-3w @
5c2947c8) + is proptest-verified containment-preserving. The elegant move was to
EXPOSE this internal surface at the wire boundary, not invent a new primitive
(ABE / access-tree / selector / Merkle-set). **Pattern.** When asked "should we
add a new primitive for X?", first audit whether an existing combinator-surface
(intersect / union / filter / map) can be promoted from internal to wire-layer.

**Codify.** Sharpens `feedback_engine_primitives_vs_application_layer.md`:
before adopting any new engine primitive, run a "combinator-surface audit" — does
an internal structural combinator already exist that, if promoted to the wire/grant
boundary, would close the gap? This is N1's exact discipline applied prospectively.

### §11.3 Pattern P13 — "Truth-in-naming applies to codepoint-reserves too"

M-CONS-v1 §10.3 (Pattern P3) codified truth-in-naming as a first-class consolidation
lever (CGKA-LITE → MultiRecipientSealing). N4 R-N4-1 extended this: codepoint-reserve
names ALSO need truth-in-naming. `AtriumWithCGKA` baked in a future-design choice
(CGKA-shape) that we hadn't made; renaming to `AtriumWithRotatingGroupKey` un-bakes
the choice. **Pattern.** When a name promises properties or shape not yet committed
to, the misleading name leaks into downstream analyses + induces incorrect
future-design pressure.

**Codify (extends M-CONS-v1 P3).** Truth-in-naming applies at EVERY surface, not
just types. Codepoint-reserve names, error-code names, doc-section names — all
need the "does this name promise what we deliver / commit to?" audit. N4's rename
is the exemplar.

### §11.4 Pattern P14 — "Honest-architectural-disclosure framing distinguishes 'limitation' from 'substrate guarantee'"

N3's Compromise #55 (GDPR-RTBF) is a structural disclosure: a content-addressed
P2P system cannot unilaterally erase content. Ben's framing intervention reframed
this from "a limitation we should fix" to "a P2P-by-design semantic; applications
layer strong-RTBF if needed." **Pattern.** Audit-deliverable Compromise mints carry
TWO categorically different shapes: (a) "vulnerability we accept as a trade-off"
(#42 FS-gap; #46 wire-cost; #52 no-PCS-against-removed-members), (b) "architectural
substrate guarantee we honestly disclose" (#55 GDPR-RTBF; #56 journalist per-message
FS — both NOT vulnerabilities, but substrate properties applications must layer
around).

**Codify.** Compromise # registry entries should carry an explicit `disposition_class`
field: `Accepted-Trade-Off` vs `Substrate-Guarantee-Disclosure`. Auditors process
these categorically differently. The former is "we considered fix, deferred"; the
latter is "this is how the system works; applications layer compensating
controls." Add to `docs/SECURITY-POSTURE.md` Compromise # template.

### §11.5 Pattern P15 — "Op-surface enumeration discipline catches missing ops the 'implement the obvious 7' heuristic skips"

N2 §3 enumerated 24 ops + classified each (v1-beta-LB / codepoint-reserve / P4MC /
post-v1β). Found 4 v1-beta-LB ops M2 + M-CONS-v1 missed (`change_role`,
`member_key_rotation`, `self_leave`, `rename`/`update_metadata`). These weren't
optional sugar — they're real-UX-need ops that v1-beta lacking would force
application-layer hacks. **Pattern.** When designing a primitive crate, explicitly
enumerate ALL ops members + admins might want, then classify each. The
enumeration discipline catches missing ops that "implement the obvious N"
heuristic skips.

**Codify.** Every new-primitive R0 plan-doc must include a § "Full operations
enumeration" section that classifies ALL conceivable ops, not just the minimum.
N2's 24-op enumeration is the exemplar; the brief framing of "what do members +
admins want to do?" surfaces ops the implementation-driven framing misses.

### §11.6 Pattern P16 — "Naming-metadata slots are foundational primitive surface"

N2's `MembershipSetMetadata` (name / description / created_by_human_label) closes
the M-CONS-v1 §0 "Atrium vs AtriumFork" naming-as-documentation gap. **Pattern.**
Naming/labeling metadata is foundational primitive surface, not optional UX
add-on. Even DeviceMesh + SingleDevice benefit from `created_by_human_label`
slots. Every social-platform reference design (Discord / Matrix / Keybase / Slack)
ships name slots at the founding primitive level.

**Codify.** Every named-entity primitive gets a `Metadata { name, description,
created_by_human_label }` shape at the ROOT struct level; never defer to "the UI
will figure out names." N2's `MembershipSetMetadata` is the exemplar.

### §11.7 Pattern P17 — "Convergent + parallel-N specialists + Ben-refinement-pass is the optimal consolidation pipeline"

Aggregated process meta-finding. The M-panel (M1a/M1b/M1c catalog + M2/M3/M4/M5/M6
panel) → M-CONS-v1 → N1/N2/N3/N4 refinement → M-CONS-v2 pipeline produced:
- Strong convergence on substantive shape (M-panel + N3 + N4 all-confirm).
- Genuine generalization-catch through N1 (RestrictedScopeSet) + N2 (Authority +
  RBAC) that the convergent M-panel missed.
- Bounded doc-only refinements (N3 + N4) that close audit-deliverable + future-
  scope gaps without code change.
- Critique-round (M-C1/M-C2/M-C3) still ahead — but the integration is already
  decision-ready for Ben in one pass.

**Codify.** For major primitive consolidations, the optimal pipeline is:
catalog-N → primitive-panel-N (with explicit red-team specialist) → consolidator-N →
Ben-refinement-pass-N (parallel N-specialist refinement) → consolidator-v2 →
critique-round-N (M-C1/M-C2/M-C3). The Ben-refinement-pass is the load-bearing
step that the typical critique-round-only flow skips. M-CONS-v2 is the exemplar.

### §11.8 Pattern P18 — "Codepoint-reserve discipline scales to sub-slots"

N4 R-N4-2's `RotatingGroupKeyChainedMode` sub-slot under the `AtriumWithRotatingGroupKey`
reserve is a sub-codepoint-reserve. **Pattern.** Codepoint-reserve discipline isn't
just first-level; you can nest reserve-slots inside reserve-slots when the outer
slot's eventual landing has variant-shapes you want to preserve flexibility on.
The cost is bounded (sub-slot is doc-only at v1-beta); the value is preserving
optionality on inner-variant choice at the cost of one codepoint-table row.

**Codify (extends M-CONS-v1 P4).** Codepoint-reserve discipline is RECURSIVE.
Every phase-close R6 R3 asks "is this a codepoint-reserve-able deferral?" — and
WHEN the reserve is itself a polymorphic slot, ALSO ask "should we reserve
sub-codepoints within this reserve?" N4's `RotatingGroupKeyChainedMode` is the
exemplar.

### §11.9 Pattern-induction meta-summary

**M-CONS-v1 patterns P1–P10 preserved unchanged.** M-CONS-v2 adds patterns P11–P18.
Cumulative count: 18 phase-close patterns codified across the MembershipSet
unification work. Each is grounded in specific cross-panel observation;
codification recommendations are bounded.

---

## §12 Self-assessment + confidence per finding

| Finding-class | Confidence | Reasoning |
|---|---|---|
| §1 N1 integration (RestrictedScopeSet F28) | **HIGH** | N1's "structural lever already ships" finding is load-bearing; all 5 frozen-surface preservation pins explicit; cost bounded |
| §1 N2 integration (Authority + RBAC + 24-op + metadata) | **HIGH on direction; MED-HIGH on F-N2-A/B/C exact wave-day cost** | Ben already ratified 24-op classification; Authority generalization replaces M-CONS-v1 §5.1 cleanly; absorption of F-A1+F-A2 is mechanical |
| §1 N3 integration (RotationTrigger + #54/#55/#56 mints) | **HIGH** | All doc-only; Ben framing for #55 ratified; #54 watch-list grounded in attack-scenarios; #56 my-pred MINT for audit-clarity (defensible) |
| §1 N4 integration (rename + sub-slot + SSK-comparison §-row) | **HIGH** | Ben already ratified rename + Option B; remaining mechanical |
| §2 Q1 iroh-gossip SCOPE-IN cost (+5–8) | **MED** | Wave-day estimate compounds many sub-deliverables; iroh-gossip integration footgun variance dominates bracket width |
| §3 Final-final amendment registry (F1–F28 + F-N2-A/B/C) | **HIGH on row structure; MED-HIGH on F-N2-A/B/C absorption credit accounting** | F-A1/F-A2 absorption is mechanical; F-N2-A/B/C wave-day delta uses N2 §5.1 estimates |
| §4 Compromise # registry (#52–#56) | **HIGH on #52/#53-narrowed/#54/#55; MED-HIGH on #56-optional** | Ben framing for #55 + N3 grounding for #54; #56 mint discretionary but my-pred MINT |
| §5 Inv-20 10-clause invariant | **HIGH on mint; MED-HIGH on exact clause-i + clause-j phrasing** | Clause-i (uniform Authority) + clause-j (3-role RBAC + UCAN intersection-of-allows) extend M-CONS-v1's 8-clause cleanly; phrasing refinable at R6 R3 |
| §6 D1–D7 walkthrough under N2 Authority slot | **HIGH on D1/D2/D3/D5/D6; MED-HIGH on D4 (substantive change) + D7 (defer with pinned variant)** | D4 is the load-bearing change vs M-CONS-v1 §5.1; cleaner shape but trade-off documented |
| §7 Wave-sequencing (Wave-MS-PRIMITIVE expanded canary + Wave-MS-TRANSPORT iroh-gossip impl) | **HIGH on shape; MED on per-wave-day estimates** | Canary discipline preserved; sub-wave parallelism preserved; iroh-gossip integration variance noted |
| §8 Cost re-estimate (~85–111 wave-days central; ~83–115 bracket) | **MED** | Many small compounds; iroh-gossip variance + N2 RBAC absorption inside canary both contribute to bracket width |
| §9 Open Ben-call decisions (4 remaining) | **MED-HIGH on bundling; HIGH on each being non-arch-fork-class** | All 4 surfaceable in one Ben-pass |
| §10 R0 plan-doc skeleton updated | **HIGH on structure** | Brief enumerated 17 sections + M-CONS-v2 additions; structural-only |
| §11 Pattern-induction (P11–P18 added) | **MED-HIGH** | Each pattern grounded in specific cross-panel observation; codification recommendations bounded |

### §12.1 What I could be wrong about

- **Cumulative wave-day arithmetic may be off by ±3 days.** Many small per-amendment
  estimates compound. Iroh-gossip integration variance is the largest uncertainty
  (+1.5–4.5 spread for that single item); N2 absorption-credit accounting could
  shift ±0.5 depending on whether F-A1/F-A2 doc-only sharpenings really cost 0.7
  wave-days as the M-CONS-v1 baseline assumed.
- **Compromise #56 mint** may be over-engineered if audit firm clusters it under
  #54. My-pred MINT separately for structural clarity; defensible to skip.
- **N1's audit-commitment sub-row** (Option G-AUDIT-SIBLING) may be wasted if
  no regulatory-audit requirement materializes by Phase 5+. Cost is bounded
  (~+0.1 wave-day); preserves optionality.
- **N2's `MembershipSetMetadata` placement inside `MembershipSetPolicy`** may be
  wrong — top-level on `MembershipSet` is arguably more ergonomic for non-security-
  critical metadata. My-pred inside-Policy (admin-gated rename); defensible either
  way.
- **Wave-MS-TRANSPORT iroh-gossip integration estimate (5–8 wave-days)** may be
  optimistic if Spike C-evidenced footguns prove more substantial during impl.
- **`refresh_required_secs` keep-vs-rename decision** is judgment call; my-pred
  KEEP+CLARIFY but RENAME is defensible since M2 is pre-freeze.

### §12.2 What this consolidation does NOT cover

- **Authoring the full R0 plan-doc** (skeleton only per §10; M-R0 owns substantive
  content).
- **M-C1/M-C2/M-C3 critique-round outputs** (those run against this M-CONS-v2 doc).
- **Final wave-day adjudication** after the critique round.
- **Per-amendment doc-prose** (each F-row needs to land in the appropriate audit-
  deliverable doc per the wave-day budget).
- **Iroh-gossip integration spike + production-hardening per-task plan** (lands
  in Wave-MS-TRANSPORT execution; not this consolidation).

### §12.3 Convergence verdict (process-level)

The M-CONS-v2 reconciliation found:
- **STRONG cross-panel convergence preserved from M-CONS-v1** (10/10 cross-axis
  agreement on substantive shape; no N-specialist overturned any M-CONS-v1
  ratification).
- **N2 caught a genuine generalization** (uniform Authority slot replacing
  KindPolicyAdminEntity 3-variant enum) that the M-panel + M-CONS-v1 missed —
  validating the Ben-refinement-pass discipline.
- **N1 closed an asymmetric-shape limitation** Ben surfaced via an already-shipping
  combinator surface (`combinators::union`), not via a new primitive — exemplary
  elegant-shape pass.
- **N3 + N4 + Ben framing intervention** sharpened doc-deliverable surface without
  any code-shape change.
- **Q1 SCOPE-IN ratification** expands v1-beta scope by ~+5–8 wave-days but cleanly
  composes with the existing F27 codepoint-reserve shape.

**The M-CONS-v2 output is DECISION-READY for the M-C1/M-C2/M-C3 critique round**
with the 4 remaining open Ben-calls surfaced in §9.3.

---

## §13 Citations

### §13.1 M-CONS-v1 baseline + N-refinement artifacts (frozen SHAs)

- M-CONS-v1 consolidator: `phase-4-meta-core/membership-set-m-cons-consolidator @ 74580ee6`
  → `.addl/phase-4-meta/membership-set-m-cons-consolidator.md` (1081 LOC; read in full)
- N1 sub-graph-sharing-elegance: `phase-4-meta-core/membership-set-n1-subgraph-sharing-elegance @ ed592770`
  → `.addl/phase-4-meta/membership-set-n1-subgraph-sharing-elegance.md` (1067 LOC; read in full)
- N2 generic-primitive-ops-roles: `phase-4-meta-core/membership-set-n2-generic-primitive-ops-roles @ a1b5a552`
  → `.addl/phase-4-meta/membership-set-n2-generic-primitive-ops-roles.md` (1399 LOC; read in full)
- N3 continuous-rotation-edge-cases: `phase-4-meta-core/membership-set-n3-continuous-rotation-edge-cases @ 298c80d9`
  → `.addl/phase-4-meta/membership-set-n3-continuous-rotation-edge-cases.md` (506 LOC; read in full)
- N4 Signal-Sender-Keys-comparison: `phase-4-meta-core/membership-set-n4-signal-sender-keys-comparison @ 2ee24e9f`
  → `.addl/phase-4-meta/membership-set-n4-signal-sender-keys-comparison.md` (494 LOC; read in full)

### §13.2 5-panel + 3-cataloger background (frozen SHAs; via M-CONS-v1 §12)

- M2 primitive design: `6170980b`
- M3 amendment-transformation: `1ba3a4c3`
- M4 red-team: `066785b5`
- M5 CGKA-candidate-survey: `15819500`
- M6 transport-configurability: `e5046c87`
- M1a multi-device-sync: `3618e051`
- M1b Atrium-membership-sharing: `1816ea60`
- M1c key-management: `50eb901d`

### §13.3 Background (per brief)

- 9-eyes consolidated registry: `fbdfeb16`
- 5 critique-round outputs: c1 `3f5a4351` / c2 `9c548e5f` / c3 `79c99aa5` / c4 `6ea9718a` / c5 `8e374a9d`
- Q3-revisit Option D: `23f76e24`
- AtriumPolicy design: `e90900b4`
- Path-A vs Path-B specialist (Path-A.5 winner): `5f50a028`

### §13.4 Tree-pinned (HEAD `2172cb6d`)

- `crates/benten-id/INTERNALS.md` — identity primitives
- `crates/benten-sync/INTERNALS.md` — Atrium transport + CRDT
- `crates/benten-crypto-suite/INTERNALS.md` — crypto-suite integration
- `crates/benten-caps/src/chain_authority.rs` — UCAN chain-walker seam (RBAC × UCAN composition site per N2 §2.6)
- `crates/benten-caps/src/scope.rs` — `Scope::{Hashes | RestrictedSelector}` (N1 F28 modification site)
- `crates/benten-caps/src/restricted_spec.rs` — `RestrictedScope` 6-dim (N1 F28 lifted to `RestrictedScopeSet`)
- `crates/benten-core/src/subgraph_spec/combinators.rs` — `intersect/union/filter` proptest-verified (N1's load-bearing structural lever)
- `crates/benten-engine/src/engine_sync.rs` — AtriumHandle 12 public methods
- `docs/V1-FROZEN-INTERFACE.md` §15 — frozen surfaces
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` — deferred-with-name destinations
- `docs/INVARIANT-COVERAGE.md` — Inv-14 + Inv-15 + Inv-16-mint + Inv-20 (post-M-CONS-v1 + extended M-CONS-v2)
- `docs/SECURITY-POSTURE.md` — Compromise # registry (extended with #54/#55/#56 + SSK-comparison §-row from N4)

### §13.5 CLAUDE.md baked-in + MEMORY.md disciplines

- CLAUDE.md baked-in #1 (12-primitive irreducibility — N1 honors via combinator-surface-promotion; N2 RBAC ships at engine-level per #19)
- CLAUDE.md baked-in #5 (crypto-agility — codepoint-dispatch backward-compat for N1 F28)
- CLAUDE.md baked-in #15 (v1-beta interface freeze — F28 + F-N2-A/B/C all additive)
- CLAUDE.md baked-in #17 (multi-process + device-shape (a)/(b)/(c) + 4-dim CapabilityEnvelope — preserved in DeviceMesh KindPolicySubfields)
- CLAUDE.md baked-in #18 (4-identity-concepts + plugin trust model — MemberKey homogeneous-per-Kind preserved)
- CLAUDE.md baked-in #19 (engine-level extensions vs app-level plugins — N2 RBAC ships engine-level; #11 P12 codifies combinator-surface-audit before primitive-extension)
- `feedback_canary_first_parallel_implementation.md` — Wave-MS-PRIMITIVE canary-first preserved
- `feedback_engine_primitives_vs_application_layer.md` — N1 honors via combinator-promotion; N2 RBAC justified as engine-level
- `feedback_extra_reflection_pass_for_elegant_permanent_shape.md` — N1 + N2 + N3 + N4 all apply this lens
- `feedback_iterate_critical_reviews_to_convergence.md` — sharpened by N2-P2 + M-CONS-v2 P11
- `feedback_no_defer_HARD_RULE.md` — every deferral has NAMED destination + revisit-trigger (N1 §5.2; N3 #54 trigger; N4 codepoint sub-slot)
- `feedback_handoff_top_banner_re_orient.md` — honored at file top
- `feedback_plain_english_surfaces.md` — plain-situation + concrete-options + my-prediction + confirm-or-redirect preserved throughout §1 + §9
- `feedback_surface_arch_decisions_under_auth.md` — 4 remaining open Ben-calls in §9.3 all confirmed non-arch-fork-class
- §3.5g cross-language rule-mirror — TS-side RoleId + PermissionSet + RestrictedScopeSet + ErrorCode catalog mirrors named in F-N2-A + F28
- §3.5h pre-merge JSON validation + cite-drift discipline (every cite verified at author-time)

---

**End of M-CONS-v2 consolidator document.** Output to be reviewed by the M-C1
elegant-shape / M-C2 composability / M-C3 fresh-eyes critique round, then handed
to the R0-author for the F-full plan-doc substantive content per the ADDL pipeline.
