# N2 — Generic-MembershipSet refactor + RBAC + full ops enumeration

> **Top-banner re-orient (HANDOFF discipline).** This is N2 of 4 parallel post-M-CONS
> refinement specialists. Ben surfaced three substantive refinements after seeing M2's
> typed-variant design + M-CONS consolidation: (1) push the unification further — Admin in
> a multi-user MembershipSet and User in a single-user MembershipSet are the same
> primitive thing at the authentication level; (2) push beyond admin-vs-user binary to a
> generic role/permission system; (3) enumerate the full op surface members + admins
> want, beyond M2's 7-op API. Read BEYOND what's named here if you arrive cold: the
> 3 cataloger outputs (M1a/M1b/M1c), the M3 amendment-transformation
> (`1ba3a4c3`), the M5 CGKA-survey (`15819500`), the M6 transport-configurability
> (`e5046c87`), AtriumPolicy `e90900b4`, Path-A.5 winner `5f50a028`, Q3 Option D
> `23f76e24`. Tree HEAD at write: `2172cb6d`.
>
> Branch: `phase-4-meta-core/membership-set-n2-generic-primitive-ops-roles`.
> Date: 2026-05-28. Author: N2 specialist on isolation-worktree dispatch.

- **Role.** N2 of 4 (N1 generic-primitive-only / N2 generic + ops + roles / N3 ops-only /
  N4 fresh-eyes critique). The HEAVY lane — Ben singled out N2 as the most substantial
  of the 4 refinement specialists.
- **Posture.** DEFAULT TO DISAGREE if the brief framing leans further than the substrate
  warrants. M2's typed-variant is already-ratified at M-CONS; the Ben-surfaced "even
  more generic" framing must EARN its win against the surface M2 + M-CONS pinned.
- **Tasks.** 5 per the brief. Task 1 (generic-vs-typed evaluation); Task 2 (RBAC); Task 3
  (full ops enumeration); Task 4 (recommended final primitive); Task 5 (R0 plan-doc
  implications).
- **Out-of-scope.** Authoring the full R0 plan-doc (M-R0 owns); re-running the M-CONS
  reconciliation (M-CONS owns); re-running the M4 red-team (closed); changing the
  Q1/Q2/Q3-DUAL-CID/Q4/Am4 Ben-ratified decisions (orthogonal); landing impl code
  (this is a design doc, not a wave-MS-PRIMITIVE landing).

---

## §0 Headline verdict (read-cold)

**Verdict.** **HYBRID-RECOMMENDED.** Keep M2/M-CONS's typed-variant `MembershipSetKind`
discriminator (Atrium / DeviceMesh / SingleDevice) at v1-beta — DO NOT collapse to a
pure generic — BUT generalize the WITHIN-VARIANT shape along TWO axes Ben's refinement
correctly identified: (a) introduce a uniform `authorities: BTreeSet<Authority>` field
collapsing the admin-vs-user-vs-self binary into "the principals who can authorize
operations on this set" — same conceptual primitive across all 3 Kinds, with Kind-
specific *cardinality + signing-policy* defaults; (b) introduce a v1-beta-minimal
`RoleAssignments` shape (3 fixed roles: `Admin / Member / Viewer`) inside
`MembershipSetPolicy` — extensible via additive codepoint to richer role-systems
post-v1-beta. The generic-primitive impulse is RIGHT directionally — the admin/user
collapse IS real — but the typed-Kind discriminator carries load-bearing threat-model
+ per-Kind UX + cryptographic-key-source invariants the M2 + M-CONS + M4 panel
unanimously surfaced. We add generality WITHIN the typed shell, not by removing
the shell.

**Confidence.** HIGH on the hybrid recommendation (3 of the 5 brief-tasks point this
way once compared against M2+M-CONS+M4 evidence). MED-HIGH on the specific shape of
the `Authority` + `RoleAssignments` types (alternative shapes defensible; defaulting
to the simpler 3-role v1-beta shape with codepoint-reserve for richer-RBAC is
conservative + reversible). HIGH on the full-ops table (24 ops surveyed; 9 v1-beta-LB,
8 v1-beta-codepoint-reserve, 7 Phase-4-Meta-Composing-or-later).

**Net cost delta vs M2 + M-CONS baseline.** **+1.5 to +2.5 wave-days** for the
Authority + RoleAssignments + ops surface refinement; **NEUTRAL** on wire-format (the
shape lands within MembershipSet's existing CBOR slot per F17); **+~150 LOC** in
`benten-membership-set` crate; **~0 audit-page delta** (replaces parts of M-CONS F-A1
+ F-A2 + parts of D4 walkthrough; absorbs Discord-style permissions reference into
SECURITY-POSTURE.md).

**Key Ben-call decisions surfaced (full §9):** 7 ratify-bundles + 2 keep-defaults.
None are arch-fork-class. The HARDEST is §9.3 — whether to ship the 3-role RBAC at
v1-beta (my-pred: YES with codepoint-reserve for richer systems) vs ship only-admin-vs-
member at v1-beta (M-CONS default) vs ship full ABAC-style attribute-roles at v1-beta
(my-pred: NO — over-engineered for 5-week-old engine + 17-22 weeks to v1-beta).

**Convergence vs M2 + M-CONS.** The Authority generalization replaces the
KindPolicyAdminEntity enum from M-CONS §5.1 (cleaner shape). The RoleAssignments slot
replaces the implicit "admin can do anything; member can do anything else"
binary that M2 + M-CONS shipped. The 24-op enumeration EXTENDS M2's 7-op API; only 2
of M2's 7 ops move (`update_policy` → `update_policy_value` + per-field setters;
`add_member` → `add_member` + `propose_member`/`accept_invite` invitation flow at
codepoint-reserve). M-CONS F-A1 + F-A2 absorb cleanly into the new shape.

---

## §1 Task 1 — Generic-MembershipSet refactor evaluation

### §1.1 Restating Ben's hint

> Admin in a multi-user MembershipSet and User in a single-user/multi-device
> MembershipSet is the **same primitive thing** at the authentication level.

This is correct. In M2's typed shape, `KindPolicy::Atrium { admin_pubkey }` carries
the admin-DID + sig-pubkey; `KindPolicy::DeviceMesh { user_did, envelope_ceiling }`
carries the user-DID (which signs as admin); `KindPolicy::SingleDevice { device_did }`
carries the device-DID (which signs as self-admin). At the authentication level —
"who is the authority entitled to sign membership-mutation operations?" — these are
all the same shape: ONE-OR-MORE DIDs paired with their hybrid-sig-pubkeys whose
signature validates a `MembershipMutation`. Three different field names + structurally
different per-Kind member-shapes hide what is structurally a uniform `authorities`
slot.

### §1.2 Comparison: M2 typed-variants vs Pure generic vs Hybrid

| Aspect | M2 typed-variants (M-CONS) | Pure generic | **Hybrid (recommended)** |
|---|---|---|---|
| Kind discriminator | Compile-time type-tag in `MembershipSetKind` + `KindPolicy` + `MemberKey` enums | Runtime policy-tag in `policy.kind: KindTag` field; no enum dispatch | **Compile-time `kind: MembershipSetKind` enum kept; per-Kind subfields collapsed** |
| `authorities` shape | Per-Kind enum variant (3 distinct shapes) | Uniform `Vec<Authority>` | **Uniform `BTreeSet<Authority>` (M-CONS F-A1 absorbed; cleaner)** |
| `MemberKey` shape | Per-Kind enum variant (3 distinct shapes) | Uniform `Member { did, hpke_pk, sig_pk, attestation: Option<DeviceAttestation> }` | **Per-Kind enum kept (M4 BREAK 5 + threat-model invariants); attestation slot Option-shaped** |
| Threat-model invariants | Statically per-Kind | Documented + runtime-checked | **Statically per-Kind (kept)** |
| Code complexity | More types; clear per-Kind paths | Less types; uniform paths but with runtime branches | **Slight reduction vs M2; uniform `authorities` + `roles` cuts ~3 enum variants** |
| Flexibility | Mixed-member-types disallowed by type | Mixed-member-types possible at compile time (runtime-checked) | **Mixed-member-types disallowed by type (kept)** |
| Per-Kind operations (e.g., `fork` Atrium-only) | Runtime-reject for non-Atrium (M2) | Same | **Same — `fork` runtime-rejects for non-Atrium (or sibling-trait per M3 §8.4 v1.x option)** |
| K_Set source | Per-Kind compile-time variant (CSPRNG vs K_principal) | Per-Kind runtime tag | **Per-Kind compile-time variant (kept; load-bearing per M4 §4.3)** |
| Forward extensibility (Garden 4th arm) | EXACTLY-3-arm enum; HALT-AND-SURFACE-TO-BEN per §15.c | Easier (just add a tag) | **EXACTLY-3-arm kept; +codepoint-reserve slot for AtriumWithCGKA (M-CONS); Garden = future ratification** |

### §1.3 Why NOT pure generic — 5 substantive objections

These are the M-CONS + M4 + N2 reasoning recap, with N2-specific sharpening:

1. **Threat-model differences are categorical, not gradients.** Atrium has multi-user
   insider-threat; DeviceMesh has none (all devices are "you"); SingleDevice is
   degenerate. M4 §2 + M2 §5 catalog these per-Kind. A pure-generic primitive would
   force every operation to dispatch on runtime policy-tags to apply per-threat-model
   defenses. The type system loses an axis of safety. This is exactly the pattern
   §3.5j (stable clippy gate) + Inv-15 (typed dispatch never silent-fallback) name
   as a hazard.

2. **K_Set source differs categorically.** Atrium = CSPRNG-random K_Atrium minted on
   create. DeviceMesh = K_principal loaded from per-DID vault. SingleDevice = same as
   DeviceMesh (degenerate). M4 §4.3 explicitly identifies this as a "substrate-incorrect"
   collapse if a single generic K_Set field tries to cover both. The per-Kind variant
   is the ONLY shape that respects v1-FROZEN substrate.

3. **MemberKey shapes differ.** UserDid (no attestation) vs DeviceDid (with DeviceAttestation
   envelope per CLAUDE.md #17 4-dim) vs LocalDevice (degenerate). The DeviceAttestation
   slot is load-bearing for envelope-ceiling enforcement (M4 §6.3 + §11.5). A pure-generic
   `MemberKey` with an `Option<DeviceAttestation>` field DOES technically work — but it
   forces every encrypt-to-set call site to runtime-check whether attestation is needed,
   instead of the type system enforcing that DeviceMesh members always carry attestation.
   This is the same "homogeneous-per-Kind" finding M4 BREAK 5 surfaced.

4. **UX language collapses badly.** M4 §3.1 catalogs how "Membership Set" UX language
   is wrong for SingleDevice ("Your Engine"), wrong-ish for DeviceMesh ("Your Devices"),
   right for Atrium ("Shared with [Atrium-Name]"). The KIND discriminator surfaces the
   per-Kind UX dispatch cleanly. A pure-generic primitive that's PURELY runtime-tagged
   forces the UX layer to do the same dispatch with weaker compile-time guarantees.

5. **The convergence-signal across 5 specialists.** M2 + M3 + M4 + M5 + M6 all
   independently arrived at typed-variants (per M-CONS §0 convergence verdict). N1 + N2
   + N3 + N4 are not yet landed but the brief's framing acknowledges M-CONS's verdict.
   Per `feedback_iterate_critical_reviews_to_convergence.md` § P1 codification: when
   N independent specialists converge, the convergence itself is corroborating evidence.
   Overturning that convergence requires substantive new evidence — and Ben's
   refinement-hint is a DIRECTIONAL signal, not new substrate evidence.

### §1.4 Why YES generic on the `authorities` axis

But: Ben's hint is RIGHT on one substantive axis the M2 + M-CONS panel missed.

**Across all 3 Kinds, "the principals who can sign membership-mutation operations"
collapses to ONE shape.** The current M2 + M-CONS shape carries three different
field-names for the same concept:

- `KindPolicy::Atrium.admin_pubkey: HybridSigPubkey` (per-Kind variant)
- `KindPolicy::DeviceMesh.user_did: Did` (per-Kind variant; sig-pubkey resolved via DID)
- `KindPolicy::SingleDevice.device_did: Did` (per-Kind variant; sig-pubkey resolved via DID)

These three fields all encode the same concept: **"the DID(s) whose signature(s)
validate a membership-mutation."** Renaming them via a per-Kind enum hides the
unification; using a uniform `authorities` BTreeSet field surfaces it.

**This is the M-CONS §5.1 KindPolicyAdminEntity enum that this N2 doc REPLACES** with
a cleaner shape:

```rust
/// A signing-authority entry on a MembershipSet.
/// Same shape across all 3 Kinds; per-Kind cardinality + default-pop differ.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Authority {
    /// The DID whose signature(s) validate mutation operations under this Authority slot.
    pub did: Did,
    /// The hybrid-sig pubkey for that DID at the binding generation.
    /// Cached here so verifiers don't need to re-resolve the DID for every mutation.
    pub sig_pubkey: HybridSigPubkey,
    /// What role-bundle this authority entry holds.
    /// At v1-beta: `RoleId::Admin` (canonical); future role-systems extend additively.
    pub role: RoleId,
    /// Timestamp this authority was admitted (HLC-stamped for ordering).
    pub admitted_at_hlc: BentenHlc,
}
```

The `BTreeSet<Authority>` field on `MembershipSet` (or `MembershipSetPolicy`) then
generalizes:

- **Atrium:** `authorities = {Authority { did: founder, role: Admin, ... }}` (or
  many for threshold-admin per AtriumPolicy D7).
- **DeviceMesh:** `authorities = {Authority { did: user_did, role: Admin, ... }}`
  (the user-DID is the sole admin; could become threshold-of-N-devices per future
  threshold-admin extension).
- **SingleDevice:** `authorities = {Authority { did: device_did, role: Admin, ... }}`
  (the device is self-admin).

The Kind discriminator stays — it determines K_Set source, MemberKey variant, fork
behavior, threat model. But the `authorities` field is uniform across all 3 Kinds.
This is a SIGNIFICANT reduction in per-Kind enum complexity vs M-CONS §5.1
KindPolicyAdminEntity (which had 3 distinct variants).

### §1.5 Verdict on Task 1

**Hybrid wins.** Keep typed-Kind discriminator + per-Kind K_Set + per-Kind MemberKey
+ per-Kind fork-semantic. Generalize the `authorities` slot to uniform `BTreeSet<Authority>`.
This is what Ben's hint correctly identifies — and the unification lands cleanly
without overturning the load-bearing M2 + M-CONS + M4 invariants.

Cost: ~-30 LOC (collapses KindPolicyAdminEntity 3-variant enum) + ~+50 LOC (Authority
struct + BTreeSet ops); net ~+20 LOC.

---

## §2 Task 2 — Generic RBAC / role-based-access within MembershipSet

### §2.1 What Ben asked for

> Beyond admin-vs-member binary, should we give generic control over role definitions /
> permissions they grant (read vs write vs etc.) + how people/devices get assigned to
> roles? Generic RBAC-class abstraction within MembershipSet rather than admin-vs-member
> binary.

This is the substantive Task 2 question. There are at least 6 design surfaces here
(role definitions; permission types; role assignment; role hierarchy; UCAN composition;
threshold roles). I walk each.

### §2.2 Reference designs (web research)

Brief findings from the major reference designs:

**Matrix room power-levels** ([Matrix Spec — Room Version 10](https://spec.matrix.org/v1.3/rooms/v10/)).
Integer power-levels (0-100). Standard permissions (ban / kick / invite / redact) +
event permissions (per-event-type m.room.*) + state permissions (state events). Default:
creator = 100; new joiners = 0. Administrative actions require higher power-level than
target. Single-axis integer hierarchy.

**Keybase team roles** ([Keybase Book — Teams](https://book.keybase.io/teams)). 5 roles:
Owner (can delete team) > Admin (manage members + subteams) > Writer (read + write) >
Reader (read-only) + Bot (special; only owners/admins add). Implicit Admin = admin of
parent subteam but not this subteam (limited access). Closed enum.

**Discord role hierarchy** ([Discord Support — Permissions Hierarchy](https://support.discord.com/hc/en-us/articles/206141927-How-is-the-permission-hierarchy-structured)).
47 unique permissions; roles ordered top-to-bottom; user gets UNION of all assigned
roles' permissions. Server-wide + channel-specific layer; channel overrides role.
Permission tri-state (allow / deny / inherit). Extensible role set.

**ATProto custom feeds + RBAC literature** (Oso / Pangea / Pathlock blog series).
RBAC vs ABAC vs ReBAC distinction. RBAC = coarse roles; aggregates by role. ABAC = policy
on attributes (user.dept + resource.classification + context.time). ReBAC = walks a
relationship graph. Most production systems compose RBAC + ReBAC; ABAC adds edge cases.

**UCAN capabilities** ([UCAN spec](https://github.com/ucan-wg/spec)). Capability-based;
delegation-tree with attenuation. Composition follows set semantics (additive).
`canDelegateResource` + `canDelegateAbility` functions configure delegation validity.
UCAN wildcard = super-user RBAC analog.

**MLS group membership** ([RFC 9420 — MLS Protocol](https://datatracker.ietf.org/doc/rfc9420/)).
Add / Update / Remove operations proposed by members; Commit message applies. No two
clients need to be online simultaneously. Ratchet tree; log-N rekeying. NO native role
system (any member can propose; commit-authorship determines who applied).

### §2.3 Design-space scan

**Question A: Fixed role set vs extensible role system at v1-beta?**

Options:
- (A1) Fixed 3-role set (`Admin / Member / Viewer`) hardcoded as an enum.
- (A2) Fixed 5-role set (Keybase shape: `Owner / Admin / Writer / Reader / Bot`).
- (A3) Extensible role system (user-defined roles via `MembershipSetPolicy.roles:
  BTreeMap<RoleId, RolePermissions>`).
- (A4) Integer power-level (Matrix shape; admin actions require higher power-level than
  target).
- (A5) UCAN-derived roles (capabilities are roles; no separate role system).

**Question B: Permission granularity?**

Options:
- (B1) Coarse: 3-permission set (Read / Write / Admin).
- (B2) Medium: ~8 permissions (read / write / share / kick / invite / role-change /
  policy-change / fork).
- (B3) Fine: Discord-style ~47 permissions (per-op + per-content-type).
- (B4) ABAC-style attribute-based (no fixed permission set; policies on attributes).

**Question C: Role assignment shape?**

Options:
- (C1) Per-member single role (`Member { did, role: RoleId }`).
- (C2) Per-member role-set (`Member { did, roles: BTreeSet<RoleId> }`).
- (C3) Capability-derived (UCAN grants determine de-facto roles).
- (C4) Time-bound roles (`Member { did, role: RoleId, expires_at: Option<HlcTimestamp> }`).

**Question D: Role hierarchy / inheritance?**

Options:
- (D1) Flat (no inheritance; Admin and Member are independent role-bundles).
- (D2) Linear hierarchy (Admin > Writer > Reader; higher includes lower).
- (D3) Lattice / poset (richer DAG of role relationships).
- (D4) Matrix power-level integer (no named roles; integer dominance).

**Question E: Composition with UCAN capabilities?**

Options:
- (E1) Roles SUPERSEDE UCAN (RBAC is the primary gate; UCAN is a finer detail).
- (E2) Roles COMPOSE WITH UCAN (RBAC for coarse policy; UCAN for resource-grain
  attenuation; intersection-of-allows wins).
- (E3) Roles DERIVED FROM UCAN (no separate role system; UCAN scope = role).

**Question F: Threshold-roles for sensitive ops?**

Options:
- (F1) None at v1-beta (single-admin always sufficient).
- (F2) M-of-N admin signoff for specific ops (e.g., kick / policy-change requires
  M-of-N admins).
- (F3) Per-op threshold-policy (configurable per-MembershipSet which ops require
  thresholds).

### §2.4 N2 recommendations per question

| Q | Recommendation | Confidence | Rationale |
|---|---|---|---|
| **A — role set** | **(A1) Fixed 3-role enum at v1-beta: `Admin / Member / Viewer`** + codepoint-reserve for (A3) extensible | HIGH | Matches Keybase precedent + minimal viable RBAC. 3 roles cover 95% of real use-cases. Extensibility via codepoint-reserve preserves future optionality without v1-beta scope expansion. (A2)+(A3)+(A4) all over-engineered for 17-22-week-to-v1-beta + 5-week-old engine. (A5) loses the "coarse policy" RBAC win + entangles RBAC churn with UCAN-scope churn. |
| **B — permission granularity** | **(B1) Coarse 3-permission set `Read / Write / Admin`** + (B3) Discord-style 47-permission set as codepoint-reserve for Phase-4-Meta-Composing | HIGH | Matches the role-set choice. Coarse permissions are auditable + cognitively simple. Fine permissions are over-engineered for v1-beta. The 3 permissions map cleanly to the 3 roles: Admin = {Read, Write, Admin}; Member = {Read, Write}; Viewer = {Read}. |
| **C — role assignment** | **(C1) Per-member single role** + (C2) role-set as v1.x refactor option per `feedback_extra_reflection_pass_for_elegant_permanent_shape.md` (E ext)| MED-HIGH | Per-member single role is simpler + matches the Keybase / Discord / Matrix precedent on most use-cases. Role-set is needed for Discord-style multi-role overlays; not v1-beta-load-bearing. (C4) time-bound roles fold into MembershipSetPolicy.refresh_required_secs (already at v1-beta-codepoint-reserve per AtriumPolicy D3). |
| **D — role hierarchy** | **(D2) Linear hierarchy: `Admin > Member > Viewer`** | HIGH | Matches the 3-role choice. Linear hierarchy is the simplest sound RBAC shape. Higher role includes-all-lower permissions automatically. Matches Keybase + Discord intuition. |
| **E — UCAN composition** | **(E2) Roles COMPOSE WITH UCAN; intersection-of-allows** | HIGH | UCAN ALREADY provides resource-grain capability semantics for Benten (per `benten-caps` chain-authority). Roles are coarser. The intersection-of-allows means: a member with `Write` role can write only IF their UCAN ALSO grants write on the target resource. Roles set the floor; UCAN attenuates. This avoids the "RBAC supersedes capabilities" trap (E1) where role-as-Admin trumps a UCAN that revokes write on a specific subgraph. |
| **F — threshold-roles** | **(F1) None at v1-beta; codepoint-reserve only for (F2) per AtriumPolicy D7 deferral** | HIGH | M-CONS §5 already deferred D7 with per-Kind variant pinned. v1-beta single-admin is sufficient; threshold-admin is post-v1-beta. |

### §2.5 The recommended v1-beta RBAC shape

```rust
//! crates/benten-membership-set/src/role.rs

/// EXACTLY-3-role enum at v1-beta. NOT #[non_exhaustive]; 4th = HALT-AND-SURFACE-TO-BEN
/// per §15.c discipline (same as MembershipSetKind). Future RoleId::Custom(RoleSpecCid)
/// = additive codepoint per F-N2-A (this doc's amendment proposal §5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RoleId {
    /// Can: read + write + admin-ops (add/remove members + change policy + fork).
    Admin = 2,
    /// Can: read + write (post content; cannot add/remove members; cannot change policy).
    Member = 1,
    /// Can: read only (no writes; no admin-ops).
    Viewer = 0,
}

impl RoleId {
    /// Permission flags this role grants. Linear hierarchy: Admin ⊇ Member ⊇ Viewer.
    pub fn permissions(&self) -> PermissionSet {
        match self {
            RoleId::Admin =>  PermissionSet::READ | PermissionSet::WRITE | PermissionSet::ADMIN,
            RoleId::Member => PermissionSet::READ | PermissionSet::WRITE,
            RoleId::Viewer => PermissionSet::READ,
        }
    }
    /// Hierarchy: returns true iff self dominates (or equals) other.
    pub fn dominates(&self, other: RoleId) -> bool { (*self as u8) >= (other as u8) }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct PermissionSet: u32 {
        const READ  = 1 << 0;   // open Drop bundles + read shared subgraph
        const WRITE = 1 << 1;   // encrypt-to-set + write into shared subgraph
        const ADMIN = 1 << 2;   // add/remove members + change policy + fork (Atrium only)
        // 1 << 3 .. 1 << 31 — codepoint-reserved for Phase-4-Meta-Composing + post-v1-beta extensions
    }
}
```

Per-Kind role default:

- **Atrium:** `authorities = {Authority { role: Admin, ... }}` (founder); members
  default to `RoleId::Member`; threshold-admin codepoint-reserve.
- **DeviceMesh:** `authorities = {Authority { role: Admin, ... }}` (user-DID); other
  devices default to `RoleId::Member` (they can write but not change membership of the
  mesh — only the user-DID-as-admin device can add a new device).
- **SingleDevice:** `authorities = {Authority { role: Admin, ... }}` (self-device); no
  other members.

### §2.6 RBAC composition with UCAN (E2 ratification)

The intersection-of-allows rule:

```rust
/// Check if `actor_did` can perform `op` on this MembershipSet.
/// Conjunction: (a) RBAC role grants the permission; (b) UCAN capability chain grants
/// the resource-scope ability. BOTH must allow.
pub fn check_permission(
    &self,
    actor_did: &Did,
    op: Operation,
    ucan_chain: &UcanChain,
) -> Result<(), MembershipError> {
    // (a) RBAC check
    let actor_role = self.role_of(actor_did)?;
    let required_perm = op.required_permission();
    if !actor_role.permissions().contains(required_perm) {
        return Err(MembershipError::RbacDeny { actor: actor_did.clone(), op, role: actor_role });
    }
    // (b) UCAN check (composes via `benten-caps` chain-authority)
    benten_caps::chain_authority::check_capability(
        actor_did,
        ucan_chain,
        op.required_capability(),
    ).map_err(MembershipError::UcanDeny)?;

    Ok(())
}
```

This is the canonical RBAC + ReBAC composition pattern that production systems
adopt (per Oso / Pangea references). Roles set the coarse floor (Admin can do
admin-ops; Member can write; Viewer can read); UCAN attenuates per-resource.

### §2.7 Cross-language rule-mirror (per §3.5g + Inv-15)

TS side mirror at `packages/engine/src/membership.ts`:

```typescript
export const enum RoleId { Viewer = 0, Member = 1, Admin = 2 }
export type PermissionFlag = "read" | "write" | "admin";
export const ROLE_PERMISSIONS: Record<RoleId, ReadonlyArray<PermissionFlag>> = {
  [RoleId.Admin]:  ["read", "write", "admin"],
  [RoleId.Member]: ["read", "write"],
  [RoleId.Viewer]: ["read"],
};
```

ErrorCode mirror: `E_RBAC_DENY`, `E_UCAN_DENY`, `E_ROLE_NOT_ASSIGNED`,
`E_ROLE_CHANGE_REQUIRES_ADMIN` — these become first-class entries in
`crates/benten-error-codes` + `packages/engine/src/errors.generated.ts` per the
§3.5g pub-error-variant first-class mirror discipline.

### §2.8 Verdict on Task 2

**Ship 3-role + 3-permission RBAC at v1-beta** with linear-hierarchy + UCAN-composed
intersection-of-allows. Codepoint-reserve richer role systems + finer permissions +
threshold-roles + role-sets for Phase-4-Meta-Composing / post-v1-beta.

Cost: ~+150 LOC (RoleId + PermissionSet + Authority slot integration + check_permission
helper + TS mirror) + ~+1 wave-day inside Wave-MS-PRIMITIVE (no new wave). Replaces
M-CONS F-A2 + part of D4 walkthrough.

---

## §3 Task 3 — Full enumeration of MembershipSet operations

M2 ships 7 ops. Let me enumerate the FULL surface members + admins might want, then
classify each (v1-beta-LOAD-BEARING vs v1-beta-CODEPOINT-RESERVE vs
Phase-4-Meta-Composing vs Post-v1-beta).

### §3.1 The 24-op enumeration

| # | Op | Category | Existing in M2? | Recommended v1-beta tier | Wire-affecting? | LOC est. |
|---|---|---|---|---|---|---|
| 1 | `create` | Lifecycle | YES | **v1-beta-LB** | YES | ~80 |
| 2 | `add_member` (direct, admin-just-adds) | Member-mgmt | YES | **v1-beta-LB** | YES (envelope) | ~80 |
| 3 | `propose_member` / `accept_invite` (invite-then-accept flow) | Member-mgmt | NO | **v1-beta-CODEPOINT-RESERVE** | YES (codepoint slot for InviteEnvelope shape; impl deferred) | reserve ~20 LOC for codepoint slot |
| 4 | `remove_member` (admin-kick, fork-semantic) | Member-mgmt | YES | **v1-beta-LB** | YES (envelope) | ~80 |
| 5 | `suspend_member` (temporary kick; reversible) | Member-mgmt | NO | **Phase-4-Meta-Composing** | NO at v1-beta | defer |
| 6 | `readmit_member` (re-add previously-removed) | Member-mgmt | NO (via add_member; no special path) | **v1-beta-LB (no-op; covered by `add_member`)** | NO | 0 LOC (alias) |
| 7 | `change_role` (promote/demote) | Member-mgmt | NO | **v1-beta-LB** (new with §2 RBAC) | YES (envelope) | ~60 |
| 8 | `member_key_rotation` (member rotates own keypair without K_Set rotation) | Member-mgmt | NO | **v1-beta-LB** | YES (per-member envelope; no K_Set distribution) | ~70 |
| 9 | `self_leave` (member voluntarily leaves) | Member-mgmt | NO | **v1-beta-LB** (semantic = `remove_member` with self-signing-key) | YES (same envelope as remove_member) | ~30 |
| 10 | `threshold_admin_signoff` (M-of-N admins required) | Member-mgmt | NO | **v1-beta-CODEPOINT-RESERVE** (per D7 deferral) | YES (codepoint slot) | reserve ~20 LOC |
| 11 | `encrypt_to_set` | Cryptographic | YES | **v1-beta-LB** | YES (envelope shape per F17) | ~100 |
| 12 | `dedup_blind_cid` | Cryptographic | YES | **v1-beta-LB** | YES (per F18 DUAL-CID) | ~30 |
| 13 | `fork` | Lifecycle | YES (Atrium only; reject other Kinds) | **v1-beta-LB** | YES (envelope + parent_set_id linkage) | ~80 |
| 14 | `merge` (combine two Atriums) | Lifecycle | NO | **Post-v1-beta** | NO at v1-beta | defer |
| 15 | `archive` (read-only freeze) | Lifecycle | NO | **v1-beta-CODEPOINT-RESERVE** | YES (codepoint slot for archived bit) | reserve ~10 LOC |
| 16 | `destroy` (encrypted-wipe; tombstone) | Lifecycle | NO | **v1-beta-CODEPOINT-RESERVE** | YES (tombstone slot) | reserve ~20 LOC |
| 17 | `rename` / `update_metadata` (display name + description) | Metadata | NO (M2 had no metadata) | **v1-beta-LB** | YES (string fields under MembershipSetPolicy) | ~40 |
| 18 | `update_policy` (full-policy mutation) | Policy | YES | **v1-beta-LB** | YES | ~60 |
| 19 | `update_policy_value` (per-field setter; finer than `update_policy`) | Policy | NO | **v1-beta-LB** (sugar over `update_policy`) | NO (same wire as update_policy) | ~40 |
| 20 | `create_subset` (sub-MembershipSet; Garden Phase 7) | Composition | NO | **Post-v1-beta** (Phase 7 Gardens) | YES (codepoint slot at v1-beta-CODEPOINT-RESERVE for nesting bit) | reserve ~10 LOC |
| 21 | `cross_set_share` (share content Atrium-A → Atrium-B as set) | Composition | NO | **Phase-4-Meta-Composing** | YES (codepoint slot) | defer |
| 22 | `cross_set_delegate_capability` (UCAN-chain across sets) | Composition | NO | **Phase-4-Meta-Composing** (lands via `benten-caps`) | NO new MembershipSet field | defer |
| 23 | `audit_log_query` (membership-history query; signed; tamper-evident) | Governance | NO | **v1-beta-LB** (via MembershipEvent stream per M-CONS F-A2 + this doc) | NO (read-only API) | ~60 |
| 24 | `propose_vote` / `cast_vote` / `tally` (governance proposals) | Governance | NO | **Post-v1-beta** | NO at v1-beta | defer |

**Total v1-beta-LB ops:** 11 (incl. RBAC-sugar `readmit_member` + `update_policy_value`
+ `change_role` + `member_key_rotation` + `self_leave` + `rename` + `audit_log_query`).

**Total v1-beta-CODEPOINT-RESERVE:** 6 (`propose_member`/`accept_invite` +
`threshold_admin_signoff` + `archive` + `destroy` + `create_subset`).

**Total Phase-4-Meta-Composing:** 3 (`suspend_member` + `cross_set_share` +
`cross_set_delegate_capability`).

**Total Post-v1-beta:** 4 (`merge` + `create_subset` body + `propose_vote` / governance
proposal flow).

### §3.2 Operation grouping into traits

Per §3.5j discipline + clean code surface, the 11 v1-beta-LB ops cluster naturally
into 5 trait families. This is the M3 §8.4 sibling-trait reservation finally cashed
in for the operations beyond M2's 7:

```rust
/// Lifecycle ops (subset of M2's 7).
pub trait MembershipSetLifecycle {
    fn create(...) -> Result<Self, MembershipError>;
    fn fork(&self, ...) -> Result<...>; // Atrium-only; runtime-reject for other Kinds
    fn update_policy(&self, ...) -> Result<...>;
    fn update_policy_value<F: PolicyFieldSetter>(&self, ...) -> Result<...>; // sugar
    fn rename(&self, new_name: String, ...) -> Result<...>;
}

/// Member-management ops (RBAC-gated; check_permission inside).
pub trait MembershipSetMembers {
    fn add_member(&self, new: MemberKey, role: RoleId, ...) -> Result<...>;
    fn remove_member(&self, target: &Did, rotation_policy: RotationPolicy, ...) -> Result<...>;
    fn change_role(&self, target: &Did, new_role: RoleId, ...) -> Result<...>;
    fn member_key_rotation(&self, new_pubkeys: NewKeyMaterial, ...) -> Result<...>;
    fn self_leave(&self, member_signing_key: &Keypair) -> Result<...>;
}

/// Cryptographic ops.
pub trait MembershipSetCrypto {
    fn encrypt_to_set(&self, payload: &[u8], ...) -> Result<EncryptedEnvelope, MembershipError>;
    fn dedup_blind_cid(&self, plaintext: &[u8]) -> [u8; 16];
}

/// Audit + observability ops.
pub trait MembershipSetAudit {
    fn audit_log_query(&self, filter: AuditQueryFilter) -> Result<Vec<MembershipEvent>, MembershipError>;
}

/// All 4 traits implemented for MembershipSet inherent impl.
impl MembershipSetLifecycle for MembershipSet { ... }
impl MembershipSetMembers for MembershipSet { ... }
impl MembershipSetCrypto for MembershipSet { ... }
impl MembershipSetAudit for MembershipSet { ... }
```

**This addresses M-CONS §1.6 sibling-trait extraction recommendation cleanly**:
the traits ARE the v1.x-reserved extraction; we ship them as inherent + trait blanket
at v1-beta (so callers can use both syntaxes); future extraction-to-traits-only
is a v1.x refactor that doesn't change the public surface.

### §3.3 Operation-specific notes

**Op 3 — invite-then-accept flow.** M2 + M-CONS ship admin-just-adds at v1-beta. The
invite-flow (expiring invite-token + new-user accepts via accept_invite) is needed for
realistic Atrium UX (you can't add someone whose user-DID you don't know yet). My-pred:
codepoint-reserve at v1-beta + impl in Phase-4-Meta-Composing. The reserved-codepoint
slot lives in `MembershipMutationOp::InviteProposed = 0x06` + `InviteAccepted = 0x07`.
Bounded ~20 LOC of codepoint-table-only.

**Op 5 — suspend (reversible kick).** Distinct from `remove_member` because suspension
preserves the member's slot for later re-admission without re-key. UI need: "we're not
sure if X is hostile yet; suspend their access while we investigate." DOES NOT exist in
M2 + M-CONS. Defer to Phase-4-Meta-Composing.

**Op 7 — `change_role`.** With §2 RBAC landing, this is a first-class op. Atrium uses
it to promote a Member to Admin (or demote). DeviceMesh uses it to change device-role
(rarely; all devices default to same role). SingleDevice rejects. Implementation:
`MembershipSetPolicy.role_assignments: BTreeMap<Did, RoleId>` field; mutating it
requires Admin role + emits MembershipEvent.

**Op 8 — `member_key_rotation` (NO K_Set rotation).** A member's per-DID encryption
keypair gets rotated (regular hygiene + post-compromise response per device). The
MembershipSet records the new pubkey; K_Set stays unchanged; the new pubkey gets used
for FUTURE `encrypt_to_set` stanzas to that member. The old pubkey works for past
content (member retained K_Set via the old keypair). Required ops site M2 §4 missed.
v1-beta-LB.

**Op 9 — `self_leave`.** Member voluntarily leaves. Semantically = `remove_member` with
target = self + admin-signing-key = self. Per RBAC: any role can self-leave (no
permission gate — you can always quit). Wire shape = same as `remove_member`. ~30 LOC
sugar.

**Op 17 — `rename` / `update_metadata`.** Atrium has a human-name + description. M2's
struct has no `name` / `description` field — a load-bearing UX gap. The naming-as-
documentation pattern (M-CONS §0 `Atrium` vs `AtriumFork`) needs an actual name slot
to live in. Adds `MembershipSetPolicy.metadata: MembershipSetMetadata { name: Option<String>,
description: Option<String>, created_by_human_label: Option<String> }`. All Option-shaped
so DeviceMesh + SingleDevice don't need to populate. v1-beta-LB.

**Op 23 — `audit_log_query`.** M-CONS F-A2 mints `MembershipEvent` typed-enum. This op
queries that event stream. Implementation: every membership-mutation emits a signed
`MembershipEvent` Node stored in the Atrium itself; this op reads them back filtered by
predicate. v1-beta-LB at ~60 LOC.

### §3.4 Operations that MOVED outside M2 ownership

Restating M2 §4.5 cleanly with my-pred for any borderline:

- `register_zone` → STAYS on AtriumHandle (M2 §7.1 + M-CONS).
- `set_envelope_freshness_window` → STAYS on AtriumHandle (M2 §7.2 + M-CONS).
- Capability revocation → STAYS in `benten-caps::chain_authority` (M2 §7.3 + M-CONS).
- K(N) derivation → STAYS in `benten-crypto-suite::structural_kdf` (M2 §7 + M-CONS).
- DAK derivation + Layer-A vault → STAYS in F-full Layer-D module (M2 + M-CONS).
- Hybrid-sig keypairs → STAYS in `benten-id` (M2 + M-CONS).

All 6 stay; no movement vs M2 + M-CONS.

### §3.5 Verdict on Task 3

**Ship 11 v1-beta-LB ops** (4 new beyond M2: `change_role`, `member_key_rotation`,
`self_leave`, `rename`/`update_metadata`; +1 sugar: `update_policy_value`; +1
audit-query op; M2's `readmit_member` falls out as alias of `add_member`).
**Codepoint-reserve 6 ops** (`propose_member`/`accept_invite` flow + `threshold_admin`
+ `archive` + `destroy` + `create_subset` slot).
**Defer 3 to Phase-4-Meta-Composing** (`suspend_member` + `cross_set_share` +
`cross_set_delegate_capability`).
**Defer 4 to Post-v1-beta** (`merge` + `create_subset` body + governance vote-flow).

Cost: ~+400 LOC for the 4 new v1-beta-LB ops + per-op tests + ~+0.7 wave-day inside
Wave-MS-PRIMITIVE. Replaces M-CONS F-A1.

---

## §4 Task 4 — Recommended final primitive design

### §4.1 The hybrid shape (Rust)

```rust
//! crates/benten-membership-set/src/lib.rs
//!
//! v1-beta hybrid MembershipSet primitive. Per N2 doc 2026-05-28.
//! Typed Kind discriminator + uniform Authority + RoleAssignments + extended op surface.

use benten_core::{Cid, BentenHlc};
use benten_crypto_suite::{
    aead::AeadEnvelope, codepoint::{CipherSuiteCodepoint, SigCodepoint},
};
use benten_id::{Did, DeviceAttestation, HybridSigPubkey, HybridKemPubkey};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

// =============================================================================
// §A — Identity + Kind discriminator (kept from M2 + M-CONS)
// =============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MembershipSetId(pub Cid);

/// EXACTLY-3 arms; NOT #[non_exhaustive] per §15.c. 4th = HALT-AND-SURFACE-TO-BEN.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MembershipSetKind {
    Atrium = 0,
    DeviceMesh = 1,
    SingleDevice = 2,
}

// =============================================================================
// §B — Authority + Role (N2 generalization)
// =============================================================================

/// EXACTLY-3 roles at v1-beta. NOT #[non_exhaustive]. Future custom roles =
/// additive codepoint per F-N2-A.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RoleId {
    Viewer = 0,
    Member = 1,
    Admin = 2,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct PermissionSet: u32 {
        const READ  = 1 << 0;
        const WRITE = 1 << 1;
        const ADMIN = 1 << 2;
        // 1 << 3 .. 1 << 31 reserved for additive codepoints (Phase-4-Meta-Composing+)
    }
}

/// A signing-authority on this MembershipSet.
/// Uniform shape across all 3 Kinds (N2 §1.4 generalization replacing M-CONS §5.1
/// KindPolicyAdminEntity enum).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Authority {
    pub did: Did,
    pub sig_pubkey: HybridSigPubkey,
    pub role: RoleId,
    pub admitted_at_hlc: BentenHlc,
}

// =============================================================================
// §C — MemberKey (kept typed-per-Kind; M4 BREAK 5 + N2 §1.3 invariant)
// =============================================================================

/// Typed-per-Kind member element. Homogeneous-per-Kind invariant enforced by type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberKey {
    /// Atrium member: user-DID.
    UserDid {
        did: Did,
        hpke_pubkey: HybridKemPubkey,
        sig_pubkey: HybridSigPubkey,
    },
    /// DeviceMesh member: device-DID with DeviceAttestation envelope.
    DeviceDid {
        did: Did,
        hpke_pubkey: HybridKemPubkey,
        sig_pubkey: HybridSigPubkey,
        attestation: DeviceAttestation, // CLAUDE.md #17 4-dim CapabilityEnvelope inside
    },
    /// SingleDevice degenerate.
    LocalDevice {
        did: Did,
    },
}

impl MemberKey {
    pub fn did(&self) -> &Did { match self { Self::UserDid { did, .. } | Self::DeviceDid { did, .. } | Self::LocalDevice { did } => did } }
    pub fn hpke_pubkey(&self) -> Option<&HybridKemPubkey> { match self {
        Self::UserDid { hpke_pubkey, .. } | Self::DeviceDid { hpke_pubkey, .. } => Some(hpke_pubkey),
        Self::LocalDevice { .. } => None,
    } }
}

// =============================================================================
// §D — Policy + RoleAssignments + Metadata
// =============================================================================

/// MembershipSetPolicy. Per-Kind sub-fields collapsed under Authority generalization.
/// D1-D7 walkthrough lives inline.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipSetPolicy {
    /// Monotonic; admin-mutation increments. Defends admin-replay (D1 per M-CONS §5).
    pub policy_version: u32,

    /// THE signing authorities for this MembershipSet's mutation operations.
    /// All 3 Kinds: at least one entry; typically 1 (admin-DID for Atrium; user-DID
    /// for DeviceMesh; device-DID for SingleDevice). Per-Kind cardinality enforced
    /// by validators in MembershipSet constructors.
    pub authorities: BTreeSet<Authority>,

    /// Role assignments per-member-DID. NOT containing Authorities (those have role
    /// in their own struct). Members default to RoleId::Member; Viewer = explicit
    /// downgrade; Admin = explicit promote (with bookkeeping: if `Admin` here, the
    /// did MUST also appear in `authorities` set).
    pub role_assignments: BTreeMap<Did, RoleId>,

    /// Per-Kind default policy applies the right cardinality; constructor validates.
    pub kind_policy: KindPolicySubfields,

    /// AtriumPolicy D3 v1-beta-CODEPOINT-RESERVE.
    pub refresh_required_secs: Option<u64>,

    /// Threshold-admin (M-of-N): v1-beta-CODEPOINT-RESERVE per AtriumPolicy D7.
    pub threshold_admin: Option<ThresholdAdminSpec>,

    /// Human-readable metadata (Atrium has a name; DeviceMesh + SingleDevice may not).
    pub metadata: MembershipSetMetadata,
}

/// Per-Kind sub-fields that don't generalize cleanly into `authorities`.
/// Smaller surface than M-CONS §5.1 KindPolicyAdminEntity (since Authority handles
/// the admin-DID question).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum KindPolicySubfields {
    /// Atrium: nothing extra beyond `authorities` (threshold_admin separately).
    Atrium {},
    /// DeviceMesh: envelope_ceiling caps any device's claim (CLAUDE.md #17 4-dim).
    DeviceMesh { envelope_ceiling: CapabilityEnvelope },
    /// SingleDevice: degenerate.
    SingleDevice {},
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipSetMetadata {
    pub name: Option<String>,           // "Marketing Q1 Atrium", "Alice's Devices", "My Engine"
    pub description: Option<String>,    // optional longer description
    pub created_by_human_label: Option<String>, // "Alice from her laptop on 2026-04-01"
}

/// Threshold-admin spec. v1-beta-CODEPOINT-RESERVE only; impl Phase-4-Meta-Composing+.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThresholdAdminSpec {
    pub threshold: u8,           // M (M-of-N)
    pub n: u8,                   // N
    pub eligible_admins: BTreeSet<Did>,
}

// =============================================================================
// §E — The primitive
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MembershipSet {
    pub version: u8, // = 0x01
    pub kind: MembershipSetKind,

    #[serde(skip)]
    pub id: Option<MembershipSetId>,

    pub members: Vec<MemberKey>, // canonical-sorted by did.as_bytes()
    #[serde(skip_serializing)]
    pub shared_key: KSet,        // skip default; per-recipient wrap on wire
    pub policy: MembershipSetPolicy,
    pub parent_membership_set_id: Option<MembershipSetId>,
    pub created_at_hlc: BentenHlc,
    pub membership_attestation: MembershipAttestation,
    pub transport_config: TransportConfig, // M6 / M-CONS F27
}

/// Cryptographic shared key. Per-Kind source preserved (M4 §4.3 invariant).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KSet {
    pub key_material: AeadKeyMaterial,
    pub codepoint: CipherSuiteCodepoint,
    pub generation: u32, // CRDT-vector at L9-A4 (M-CONS F19) elaborated elsewhere
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MembershipAttestation {
    pub set_id: MembershipSetId,
    pub signer_did: Did,
    pub signature: HybridSignature,
    pub sig_codepoint: SigCodepoint,
}
```

### §4.2 Canonical CBOR wire shape

Per V1-FROZEN-INTERFACE §15 + Inv-15. Replaces M2 §2.2:

```cbor
; canonical CBOR map; keys lexicographic; integer values where possible
{
  "v":     1,
  "kind":  0|1|2,                            ; MembershipSetKind discriminator
  "mem":   [ {"did": "did:key:z...", "hpke": h'...', "sig": h'...',
              "att": <DeviceAttestation CBOR | null>}, ... ],
  "pol":   {
              "pv": <u32>,                   ; policy_version
              "auth": [                       ; uniform Authorities BTreeSet
                  { "did": "did:key:z...", "sigpk": h'...', "role": 0|1|2,
                    "hlc": "<physical>:<logical>:<node>" },
                  ...
              ],
              "ra":  { "did:key:z...": 0|1|2, ... },  ; role_assignments map
              "ksub": <KindPolicySubfields CBOR>,  ; per-Kind extras
              "rr":  <u64 | null>,           ; refresh_required_secs
              "ta":  <ThresholdAdminSpec | null>, ; threshold_admin
              "meta": { "name": <str | null>, "desc": <str | null>, "by": <str | null> }
           },
  "gen":   <u32>,                            ; KSet.generation
  "cp":    <u16 LE>,                         ; CipherSuiteCodepoint
  "par":   <MembershipSetId | null>,
  "hlc":   "<physical>:<logical>:<node>",
  "tc":    <TransportConfig CBOR>,           ; M6 / M-CONS F27
  "att":   { "signer": "did:key:z...", "sigcp": <u16 LE>, "sig": h'...' }
}
```

`MembershipSetId = BLAKE3(canonical_cbor(..., att excluded))`. The `att` field signs
over `BLAKE3(canonical_cbor_minus_att)`. Identity = canonical-payload-CID per Inv-15.

### §4.3 Per-Kind constructor defaults

```rust
impl MembershipSet {
    /// Atrium constructor: founder = first admin Authority.
    pub fn new_atrium(
        founder_member: MemberKey,    // must be MemberKey::UserDid
        founder_sig_pubkey: HybridSigPubkey,
        founder_keypair: &Keypair,
        name: Option<String>,
    ) -> Result<Self, MembershipError> {
        let authorities = BTreeSet::from([Authority {
            did: founder_member.did().clone(),
            sig_pubkey: founder_sig_pubkey,
            role: RoleId::Admin,
            admitted_at_hlc: BentenHlc::now_local(),
        }]);
        let role_assignments = BTreeMap::from([(founder_member.did().clone(), RoleId::Admin)]);
        // ... ctor continues with K_Set = CSPRNG K_Atrium, kind_policy = Atrium{}, ...
    }

    /// DeviceMesh constructor: user-DID is the sole admin Authority.
    pub fn new_device_mesh(
        user_did: Did,
        user_sig_pubkey: HybridSigPubkey,
        first_device: MemberKey,      // must be MemberKey::DeviceDid
        envelope_ceiling: CapabilityEnvelope,
        user_keypair: &Keypair,
    ) -> Result<Self, MembershipError> {
        let authorities = BTreeSet::from([Authority {
            did: user_did.clone(),
            sig_pubkey: user_sig_pubkey,
            role: RoleId::Admin,
            admitted_at_hlc: BentenHlc::now_local(),
        }]);
        let role_assignments = BTreeMap::from([(first_device.did().clone(), RoleId::Member)]);
        // user_did is admin Authority; the device is Member of the mesh
        // ... ctor with K_Set = K_principal from vault, kind_policy = DeviceMesh{envelope_ceiling}, ...
    }

    /// SingleDevice constructor: degenerate self-admin.
    pub fn new_single_device(
        device_did: Did,
        device_sig_pubkey: HybridSigPubkey,
        device_keypair: &Keypair,
    ) -> Result<Self, MembershipError> {
        let authorities = BTreeSet::from([Authority {
            did: device_did.clone(),
            sig_pubkey: device_sig_pubkey,
            role: RoleId::Admin,
            admitted_at_hlc: BentenHlc::now_local(),
        }]);
        let role_assignments = BTreeMap::from([(device_did.clone(), RoleId::Admin)]);
        // ... ctor with K_Set = K_principal from vault, kind_policy = SingleDevice{}, ...
    }
}
```

### §4.4 Type-system invariants enforced

1. **Atrium MUST have at least one Authority with role=Admin.** Constructor validates.
2. **DeviceMesh authorities ALL have did matching kind_policy implicit user_did.**
   Constructor validates (the user-DID is the sole admin).
3. **SingleDevice authorities = {self-device-DID with role=Admin}.** Constructor validates.
4. **MemberKey variant matches Kind:** Atrium → only `UserDid`; DeviceMesh → only
   `DeviceDid`; SingleDevice → only `LocalDevice`. Enforced via `add_member` per-Kind
   dispatch.
5. **role_assignments keys ⊆ members ∪ authorities-dids.** Validator at every mutation.
6. **A did's role in `role_assignments` matches its role in `authorities` if both
   present.** Validator at every mutation.

### §4.5 Comparison with M2 + M-CONS

| Shape concern | M2 + M-CONS | N2 hybrid |
|---|---|---|
| KindPolicy enum 3-variants | YES (admin-DID per Kind in 3 different field shapes) | NO (collapsed into `authorities: BTreeSet<Authority>`; KindPolicySubfields enum now has 0-1 small fields per Kind) |
| Admin-vs-Member binary | Implicit (M2: admin is M2's `admin_pubkey`; everyone else is "member") | EXPLICIT 3-role RBAC via `role_assignments` |
| Audit/observability | M-CONS F-A2 mints `MembershipEvent` typed-enum | Absorbed; same `MembershipEvent` enum lives here + audit_log_query op |
| Naming metadata | Missing in M2 + M-CONS | EXPLICIT via `MembershipSetMetadata` |
| Threshold-admin slot | M-CONS D7 deferred | Same; explicit `Option<ThresholdAdminSpec>` field at codepoint-reserve |
| Per-Kind cardinality enforcement | M2 KindPolicy enum encoded structurally | Constructors validate; type system slightly weaker (BTreeSet doesn't statically enforce |authorities|=1 for DeviceMesh) |

### §4.6 Trade-off: type-system rigor vs uniform surface

The N2 hybrid trades a small amount of compile-time rigor (Authority is a uniform
struct; constructors validate per-Kind cardinality at runtime, not at type level) for
substantial uniformity (one `authorities` slot across all 3 Kinds; one `role` enum;
one `change_role` op). The compile-time loss is bounded:

- M2's `KindPolicy::Atrium { admin_pubkey: HybridSigPubkey }` enforces "Atrium has
  exactly-1 admin-pubkey" at the type level.
- N2's `authorities: BTreeSet<Authority>` + constructor validation enforces the same
  at runtime + emits a typed `MembershipError::AuthorityCardinalityViolation` if a
  caller bypasses the constructor.

**Net:** the N2 shape preserves the invariant via constructor + every mutation op
validation; it does NOT preserve it via the type system alone. This is consistent with
how `BTreeSet<MemberKey>` cardinality (Atrium must have ≥1 member) is already
runtime-enforced in M2, not type-enforced. The Kind-specific cardinality of
`authorities` is a runtime invariant, same shape.

**My-pred on the trade-off:** WORTH IT. The uniform `authorities` slot enables
clean threshold-admin growth (future M-of-N admins for an Atrium = `authorities`
grows; no new struct variant). The runtime-validation cost is bounded (~10 LOC of
constructor + ~5 LOC per mutation op) + matches the existing per-Kind member
cardinality validation surface.

---

## §5 Task 5 — R0 plan-doc implications

### §5.1 Cost estimate vs M2 + M-CONS baseline

| Component | M-CONS baseline | N2 delta | New estimate |
|---|---|---|---|
| Wave-MS-PRIMITIVE canary | ~3.5-4.5 wave-days | **+1.0** for Authority + RoleId + RoleAssignments + 4 new ops + validators | ~4.5-5.5 wave-days |
| 4 sub-wave parallel agents | ~5.5 wave-days (parallel) | **+0.3** total parallel (mostly absorbed in canary) | ~5.5-6.0 wave-days |
| Wave-MS-FINAL integration test | ~0.5 day | **+0.2** (more ops × Kinds matrix) | ~0.7 day |
| Wave-MS-TRANSPORT (M6 codepoint-reserve) | ~3.5 wave-days | **unchanged** | ~3.5 wave-days |
| Wave-G (AtriumPolicy → folded into Wave-MS-PRIMITIVE) | DELETED | unchanged | DELETED |
| Wave-C / Wave-D / Wave-Q3 / Wave-L11-CRDT reductions | per M-CONS | **+0** (no impact) | per M-CONS |
| **Total wave-days** | **~78-100 (M-CONS central)** | **+1.5 to +2.5** | **~79.5-102.5** |

**Net delta vs M-CONS baseline: +1.5 to +2.5 wave-days. Net vs original ~80-101
baseline: -0.5 to +1.5 wave-days (still NET-NEUTRAL after factoring in M-CONS's
–0.1 to –1.3 central).** Modest cost increase for substantial generality gain.

### §5.2 LOC delta vs M-CONS

| LOC site | M-CONS baseline | N2 delta |
|---|---|---|
| `benten-membership-set` crate | ~1500-2000 LOC | **+150** (Authority + RoleId + PermissionSet + RoleAssignments + 4 new ops + validators) |
| TS-side mirror | ~200-300 LOC | **+50** (TS RoleId enum + ROLE_PERMISSIONS lookup + new op signatures) |
| ErrorCode catalog (per §3.5g) | per M-CONS | **+4 entries** (E_RBAC_DENY, E_UCAN_DENY, E_ROLE_NOT_ASSIGNED, E_AUTHORITY_CARDINALITY) |
| Audit docs (SECURITY-POSTURE.md + THREAT-MODEL.md) | per M-CONS | **+~3-5 pages** (RBAC section; UCAN-composition section; role-assignment invariants; per-Kind authority-cardinality discipline) |
| Golden vectors + kani harness | per M-CONS | **+~5 vectors** (per-Kind role-assignment + RBAC-permission-check + Authority-cardinality-violation cases) |

**Net LOC delta: +~200 LOC + 4 ErrorCode entries + ~3-5 audit pages.** Inside
~+0.2 audit-page-delta from M-CONS.

### §5.3 Amendment registry impact (F-row table)

The N2 design proposes 2 new F-rows + replaces 1 M-CONS F-row:

| F# | Title | Sev | Wire | Disp | Origin |
|---|---|---|---|---|---|
| **F-A1 (M-CONS)** | `remove_member` per-Kind contract doc-only sharpening | DEFERRABLE | NO | v1-beta-LB | M-CONS Task 1 |
| **F-A2 (M-CONS)** | `MembershipEvent` typed-enum (per-Kind audit-event variants) | MED-HIGH | NO | v1-beta-LB | M-CONS Task 1 |
| **F-N2-A (NEW)** | **Authority + RoleId + RoleAssignments + 3-role RBAC at v1-beta** | LB | YES | v1-beta-LB | This N2 doc §2 + §4 |
| **F-N2-B (NEW)** | **4 new ops at v1-beta (`change_role` + `member_key_rotation` + `self_leave` + `rename`/`update_metadata`) + `audit_log_query` + 6 codepoint-reserve ops** | LB | YES (ops surface) + CODEPOINT-RESERVE (reserve-slots) | v1-beta-LB + v1-beta-CODEPOINT-RESERVE | This N2 doc §3 |
| **F-N2-C (NEW)** | **`MembershipSetMetadata` (name + description + created_by_human_label)** | LB | YES | v1-beta-LB | This N2 doc §3.3 op 17 |

**F-A1 + F-A2 absorb into F-N2-A + F-N2-B.** F-A1's "doc-only sharpening of
remove_member contract" lives now as part of the §3.1 op-23 enumeration entry +
SECURITY-POSTURE.md row per F-N2-B. F-A2's `MembershipEvent` typed-enum lives now
inside `benten-membership-set` per F-N2-A scope + emits per `audit_log_query` op
(F-N2-B item 23).

### §5.4 Compromise registry impact

The N2 design does NOT mint new Compromise#-class disclosures. The 3-role RBAC is a
positive control (gates what members can do); it does not introduce new attack
surfaces beyond what UCAN-chain-authority already disclosed. Composition with UCAN
(intersection-of-allows) is documented in SECURITY-PROOFS.md per F25 audit-deliverable
pack — no new Compromise # needed.

**Possible Compromise # NEW (low-confidence): #54 — RBAC role-assignment race condition.**
If a member promoted to Admin races a kick-of-themselves admin-mutation, ordering
matters. HLC ordering + signed mutation envelopes + policy_version monotonic increment
should resolve cleanly per existing infrastructure. **My-pred: NO new Compromise # at
v1-beta.** The disciplines already in place handle it. If audit firm flags during
Phase-5+, mint at that time.

### §5.5 Invariant set impact

The N2 design extends Inv-20 (M-CONS minted) with 2 new clauses:

> **Inv-20 (N2 ext).** Every Benten encrypt-to-N-recipients use site dispatches through
> single `MembershipSet` primitive with `MembershipSetKind` discriminator providing:
> (a) `shared_key: KSet`...
> ...(existing clauses a-h from M-CONS)...
> **(i) uniform `authorities: BTreeSet<Authority>` + `role_assignments: BTreeMap<Did,
> RoleId>` slot for signing-authority + RBAC composition; admin-DID-per-Kind concern
> resolves at constructor + mutation-op validation, not at type level.**
> **(j) 3-role RBAC (Admin > Member > Viewer) + 3-permission set (Read | Write | Admin)
> + UCAN-composed intersection-of-allows; richer role systems = additive codepoint
> (Phase-4-Meta-Composing+).**

Inv-20 now has 10 clauses. No new first-order invariant beyond Inv-20.

### §5.6 V1-FROZEN-INTERFACE.md impact

Two new sub-sections under §15.x (MembershipSet primitive row):

- **§15.x.RBAC** — RoleId enum codepoints (0/1/2) + PermissionSet flag layout
  (1<<0/1<<1/1<<2) + 1<<3..1<<31 reserved-for-additive.
- **§15.x.Authority** — Authority struct CBOR encoding + per-Kind cardinality
  constraints + role-binding.

V1-FROZEN-INTERFACE-DEFERRED.md row D-N2-1: extensible RoleId beyond 3 (codepoint-
reserve at v1-beta; impl Phase-4-Meta-Composing+).

### §5.7 R0 plan-doc skeleton additions

Adds to M-CONS §8 17-section skeleton:

- **§2.5 (NEW)** — Authority + RoleAssignments slot (this N2 doc §4)
- **§2.6 (NEW)** — 3-role RBAC (this N2 doc §2)
- **§2.7 (NEW)** — 11 v1-beta-LB ops (this N2 doc §3)
- **§7.4 (NEW)** — extensible-role-codepoint-reserve under §15.x.RBAC

### §5.8 Migration path

No production callers exist yet (MembershipSet is a new primitive; Wave-MS-PRIMITIVE
is the canary). The N2 shape lands fresh at v1-beta with no migration cost. Future
post-v1-beta richer-RBAC extensions land via additive codepoint per the existing
discipline.

---

## §6 Open Ben-call decisions + my-pred per (decision-ready)

### §6.1 Decision 1 — Hybrid generic-vs-typed (Task 1 result)

**my-pred:** **RATIFY HYBRID** — keep typed Kind discriminator (M2 + M-CONS); add
uniform `authorities: BTreeSet<Authority>` slot (N2 §1.4); preserve per-Kind MemberKey
+ K_Set source invariants (M4 BREAK 5 + M4 §4.3 + M-CONS).

**Confidence:** HIGH. Ben's hint is right ON the admin-vs-user collapse; M2+M-CONS+M4
unanimity is right on the per-Kind threat-model + K_Set source. Hybrid captures both
wins.

### §6.2 Decision 2 — Ship 3-role RBAC at v1-beta (Task 2 result)

**my-pred:** **RATIFY 3-ROLE RBAC** (Admin / Member / Viewer; READ/WRITE/ADMIN
permissions; linear hierarchy; intersection-of-allows with UCAN). Codepoint-reserve
richer systems for Phase-4-Meta-Composing+.

**Confidence:** HIGH. Matches Keybase precedent + minimal viable RBAC. Discord's 47-
permission richer model is over-engineered for v1-beta. UCAN-derived (E5) loses the
coarse-policy clarity wins.

### §6.3 Decision 3 — 11 v1-beta-LB ops (Task 3 result)

**my-pred:** **RATIFY 11 v1-beta-LB ops** beyond M2's 7. The 4 new ops + sugar +
audit-query are bounded ~400 LOC + ~0.7 wave-day cost increase + close real UX/op
gaps M2 + M-CONS missed (`rename` / `member_key_rotation` / `self_leave` /
`change_role` / `audit_log_query`).

**Confidence:** HIGH on the 4 new ops being v1-beta-LB; MED-HIGH on which of the 6
codepoint-reserve slots to actually pin codepoints for (which Ben can pick from §3.1
table).

### §6.4 Decision 4 — Codepoint-reserve which of the 6 reserve-slot ops?

**my-pred:** **RATIFY ALL 6** (`propose_member` + `accept_invite` + `threshold_admin` +
`archive` + `destroy` + `create_subset`). Each is bounded ~20 LOC of codepoint-table
slot; no impl at v1-beta; impl lands Phase-4-Meta-Composing+ per the reserve-discipline.

**Confidence:** MED-HIGH. All 6 surface real future needs that codepoint-reserve
discipline is designed for. The +120 LOC of reserved slots is bounded; future-impl
cost compounds savings.

### §6.5 Decision 5 — F-A1 + F-A2 absorption into F-N2-A + F-N2-B

**my-pred:** **RATIFY ABSORB.** M-CONS F-A1 (remove_member doc sharpening) folds into
N2 §3 op-23 audit-log row + SECURITY-POSTURE.md per-Kind kick-table. M-CONS F-A2
(MembershipEvent typed-enum) folds into N2 `benten-membership-set` crate scope per
F-N2-A + emitted by all 11 v1-beta-LB ops.

**Confidence:** HIGH. Absorption is mechanical; no semantic loss; F-N2-A + F-N2-B
+ F-N2-C are tighter than F-A1 + F-A2 + open ops gaps.

### §6.6 Decision 6 — `MembershipSetMetadata` (name + description) at v1-beta-LB

**my-pred:** **RATIFY.** Adds the missing UX name slot M-CONS §0 "Atrium vs AtriumFork"
naming-as-documentation question lives in. Bounded ~40 LOC.

**Confidence:** HIGH. Real UX gap; bounded cost; no wire-format risk (optional fields
under MembershipSetPolicy CBOR map).

### §6.7 Decision 7 — Inv-20 extension to 10 clauses

**my-pred:** **RATIFY** the 2 new clauses (i) + (j) added to M-CONS's 8-clause Inv-20.

**Confidence:** HIGH on the mint; MED-HIGH on the exact phrasing (refinable
alongside the primitive's final API per Ben's typical R6 R3 pass).

### §6.8 Decision 8 — Sibling-trait extraction into 4 trait families

**my-pred:** **KEEP DEFAULT (M2 + M-CONS) of inherent-impl at v1-beta** + ship the 4
traits alongside as `impl MembershipSetLifecycle for MembershipSet { ... }` so callers
have both surfaces. v1.x refactor option preserved per M3 §8.4.

**Confidence:** MED. Both shapes defensible. The 4-trait grouping IS the v1.x-extract
shape; shipping them alongside at v1-beta gives free optionality.

### §6.9 Decision 9 — UCAN composition rule (intersection-of-allows vs supersedes)

**my-pred:** **RATIFY intersection-of-allows (E2)**. RBAC sets coarse floor; UCAN
attenuates per-resource. Both must allow for the op to proceed.

**Confidence:** HIGH. This is the production-system standard (Oso / Pangea / Pathlock
references); avoids RBAC-trumps-UCAN trap; composes cleanly with `benten-caps`
existing chain-authority.

### §6.10 Summary table

| # | Decision | my-pred | Confidence |
|---|---|---|---|
| 1 | Hybrid generic-vs-typed (uniform Authority + typed Kind) | RATIFY | HIGH |
| 2 | Ship 3-role RBAC at v1-beta | RATIFY | HIGH |
| 3 | 11 v1-beta-LB ops (4 new beyond M2 + audit-query + sugar) | RATIFY | HIGH |
| 4 | 6 codepoint-reserve ops | RATIFY ALL 6 | MED-HIGH |
| 5 | F-A1 + F-A2 absorption into F-N2-A + F-N2-B | RATIFY-ABSORB | HIGH |
| 6 | MembershipSetMetadata (name + description) | RATIFY | HIGH |
| 7 | Inv-20 10-clause extension | RATIFY | HIGH |
| 8 | 4 sibling-traits alongside inherent-impl at v1-beta | KEEP-DEFAULT (inherent + traits both) | MED |
| 9 | UCAN composition = intersection-of-allows | RATIFY | HIGH |

**Net: 9 decisions; 7 RATIFY-bundle; 1 RATIFY-ALL-6 sub-bundle; 1 KEEP-DEFAULT. None
arch-fork-class per `feedback_surface_arch_decisions_under_auth.md`.** All 9 confirmable
in one Ben-pass cold.

---

## §7 Pattern-induction meta-findings

What does the N2 lens reveal that M2 + M-CONS missed?

### §7.1 Pattern N2-P1 — "Admin / User / Self are the same primitive concept"

The 5-specialist panel + M-CONS unanimously typed admin-DID per-Kind as 3 different
enum variants (`admin_pubkey: HybridSigPubkey` / `user_did: Did` /
`device_did: Did`). Ben's refinement-hint caught the unification the panel missed.

**Pattern.** When 3+ enum variants of a type each represent "the entity who can sign
operations on X," they're often the same primitive concept under per-Kind cardinality
+ default-population differences. **Codify:** at every R6 R3 pass, scan typed-variant
enums for "this variant is the same role across all variants but with different
default cardinality" — collapse to uniform field + per-Kind constructor when found.

### §7.2 Pattern N2-P2 — "Convergence-signal is not authoritative; Ben's refinement-hints can still find generality"

M-CONS §10.1 (Pattern P1) codified "convergent shape across 5 specialists = HIGH-confidence
ratification." This N2 doc proves the convergence-signal isn't infallible — 5
specialists all-typed the admin-DID per-Kind shape; Ben caught a real generality.

**Pattern.** Convergence signals strong but not perfect. **Codify (sharpens M-CONS P1):**
when convergent specialist outputs land, ALSO surface to Ben + ask "do you see a
generalization the panel missed?" as a first-class step. The post-M-CONS-N1/N2/N3/N4
pipeline IS this discipline operationalized — but it didn't exist before N1-N4 was
spawned. Codify as a phase-close pattern.

### §7.3 Pattern N2-P3 — "RBAC is the missing primitive in CLAUDE.md baked-in"

Benten ships UCAN capability-bound trust model per CLAUDE.md #18, but has no explicit
RBAC primitive at v1-beta. Most production access-control systems compose RBAC + UCAN
(per Oso reference: "Almost everyone layers 2 or 3 models"). Shipping a 3-role RBAC
at v1-beta gives Benten the coarse-policy layer + lets UCAN focus on resource-grain
attenuation.

**Pattern.** When a primitive (UCAN) ships fine-grained capabilities but the system
lacks a coarse-policy layer, role-system pressure arises naturally in
user-mental-models ("who's the admin?"). **Codify:** Phase-4-Meta-Composing R0 plan-doc
must include a "what's the COARSE policy layer above UCAN?" section that documents
the RBAC + UCAN composition discipline.

### §7.4 Pattern N2-P4 — "Op-surface enumeration is load-bearing for v1-beta scope-lock"

M2 + M-CONS shipped 7 ops; this N2 enumeration found 4 v1-beta-LB ops they missed
(`change_role` + `member_key_rotation` + `self_leave` + `rename`/`update_metadata`).
These aren't optional sugar — they're real-UX-need ops that v1-beta lacking them would
force application-layer hacks.

**Pattern.** When designing a primitive crate, explicitly enumerate ALL ops members +
admins might want, then classify each (v1-beta-LB / codepoint-reserve / defer). The
enumeration discipline catches missing ops that the "implement the obvious 7"
heuristic skips. **Codify:** every new-primitive R0 plan-doc must include a §
"Full operations enumeration" section that classifies all conceivable ops, not just
the minimum.

### §7.5 Pattern N2-P5 — "Naming-metadata slots are foundational, not optional"

M2 + M-CONS shipped MembershipSet without a `name` field. The M-CONS §0 "Atrium vs
AtriumFork" naming-as-documentation question is unresolvable without a name slot.
Every social-platform reference design (Discord / Matrix / Keybase / Slack) ships
name slots at the founding primitive level.

**Pattern.** Naming/labeling metadata is foundational primitive surface, not optional
UX add-on. Even DeviceMesh + SingleDevice benefit from `created_by_human_label` slots
("Alice's main laptop, created 2026-04-01"). **Codify:** every named-entity primitive
gets a `MembershipSetMetadata { name, description, created_by_human_label }` shape
at the ROOT struct level; never defer to "the UI will figure out names."

---

## §8 Self-assessment + confidence per finding

| Finding | Confidence | Reasoning |
|---|---|---|
| §1 Hybrid (typed-Kind + uniform Authority) recommendation | **HIGH (~85%)** | Triangulates Ben's hint + M2+M-CONS+M4 convergence + 5 substantive objections to pure-generic |
| §2 3-role RBAC at v1-beta | **HIGH (~85%)** | Keybase precedent + minimal-viable-RBAC + UCAN-composition standard |
| §3 11 v1-beta-LB ops + 6 codepoint-reserve + 3 defer + 4 post-v1-beta | **HIGH on the 4 new v1-beta-LB ops; MED-HIGH on the 6 codepoint-reserve choices** | The 4 new ops close real UX gaps; codepoint-reserve choices are slightly more discretionary |
| §4 Recommended primitive design (Rust + CBOR) | **MED-HIGH (~75-80%)** | Concrete shape; alternative shapes defensible; constructor-validation trade-off documented |
| §5 R0 plan-doc cost (~+1.5-2.5 wave-days) | **MED (~65-70%)** | Wave-day estimates compound; N2 might be optimistic on the 4-new-ops absorption inside Wave-MS-PRIMITIVE canary |
| §6 9-decision Ben-call summary | **HIGH on bundling; MED-HIGH on per-decision my-pred** | None arch-fork-class; all confirmable in one Ben-pass |
| §7 5 pattern-induction findings | **MED-HIGH** | Each pattern grounded in specific N2 observation; codification recommendations bounded |

### §8.1 What I could be wrong about

- **N2 §1.4 Authority generalization might over-collapse.** If the per-Kind cardinality
  invariant (Atrium 1+ admin; DeviceMesh exactly-1 user-DID admin; SingleDevice exactly-1
  self-admin) is load-bearing enough that the type-system enforcement matters more than
  uniformity, the M-CONS §5.1 KindPolicyAdminEntity enum is the better shape. My-pred:
  the constructor-validation cost is bounded + the threshold-admin growth case favors
  the uniform set; N2 hybrid wins. Confidence on this trade-off: MED-HIGH.

- **N2 §2 3-role RBAC might be either too coarse (Discord-style 47-permission needed
  earlier) OR over-engineered (UCAN-derived sufficient).** My-pred: the 3-role floor is
  the right v1-beta size — Discord 47-permission is over-engineered for 17-22-week-to-
  v1-beta; UCAN-derived loses coarse-policy clarity. Confidence: HIGH on the floor;
  MED on whether richer extensions actually land Phase-4-Meta-Composing.

- **N2 §3 op-3 `propose_member`/`accept_invite` deferral might be wrong.** Realistic
  Atrium UX may REQUIRE invite-flow at v1-beta (you can't add someone whose DID you don't
  know). If so, op-3 moves from codepoint-reserve to v1-beta-LB, adding ~+100 LOC + ~+0.5
  wave-day. Confidence on the deferral: MED. Ben might push back; my-pred is to
  surface this as a sub-decision.

- **N2 §4 `MembershipSetMetadata` placement.** I put it inside `MembershipSetPolicy`;
  arguably it should be a top-level field on `MembershipSet`. The argument for inside-
  Policy: metadata-mutation should go through the same admin-signed `update_policy`
  flow as other policy changes. The argument for top-level: metadata is non-security-
  critical + querying it shouldn't require Policy-version-walk. My-pred: inside-Policy
  is safer (admin gates rename); top-level is arguably more ergonomic. Confidence: MED.

- **N2 §5 wave-day arithmetic.** The +1.5-2.5 estimate compounds 4 new ops + RBAC
  surface + validators + tests + TS mirror + audit pages. Could easily be ±1 day off.
  Confidence: MED.

### §8.2 What this N2 doc does NOT cover

- Authoring the full F-N2-A + F-N2-B + F-N2-C amendment doc-prose (lands at
  Wave-MS-PRIMITIVE).
- Critique-round outputs (N4 fresh-eyes critique reviews this N2 doc; orchestrator
  consolidates across N1/N2/N3/N4).
- Per-amendment doc-prose into the appropriate audit-deliverable doc (SECURITY-POSTURE.md
  + THREAT-MODEL.md + CRYPTO-CODEPOINTS.md sections).
- Final wave-day adjudication after N4 critique.

### §8.3 Convergence verdict (process-level)

The N2 lens found 3 substantive refinements vs M2 + M-CONS: (a) Authority
generalization; (b) 3-role RBAC; (c) 4 missing v1-beta-LB ops + metadata slot.
**Each is bounded + composes cleanly with the M2 + M-CONS shape.** The hybrid
shape is decision-ready for Ben with the 9 my-pred ratifications surfaced in §6.

---

## §9 Citations

### §9.1 5-panel + M-CONS artifacts (frozen SHAs)

- M2 primitive design: `6170980b` — `/tmp/m2.md` (901 LOC; read in full)
- M-CONS consolidator: `74580ee6` — `/tmp/mcons.md` (1081 LOC; read in full)
- M3 amendment-transformation: `1ba3a4c3` (cited; not re-read in this lane)
- M4 red-team negative-findings: `066785b5` — `/tmp/m4.md` (764 LOC; read in full)
- M5 CGKA-candidate-survey: `15819500` (cited via M-CONS)
- M6 transport-configurability: `e5046c87` (cited via M-CONS)

### §9.2 3-cataloger artifacts (frozen SHAs)

- M1a multi-device-sync: `3618e051`
- M1b Atrium-membership-sharing: `1816ea60`
- M1c key-management: `50eb901d`

### §9.3 Background (per brief)

- 9-eyes consolidated registry: `fbdfeb16`
- 5 critique-round outputs: c1 `3f5a4351` / c2 `9c548e5f` / c3 `79c99aa5` / c4 `6ea9718a` / c5 `8e374a9d`
- Q3-revisit Option D: `23f76e24`
- AtriumPolicy design: `e90900b4`
- Path-A vs Path-B specialist (Path-A.5 winner): `5f50a028`

### §9.4 Web research

- Matrix Spec — Room Version 10 power-levels: https://spec.matrix.org/v1.3/rooms/v10/
- Matrix Spec — Client-Server API: https://matrix.org/docs/spec/client_server/r0.2.0.html
- Keybase Book — Teams roles + design: https://book.keybase.io/teams , https://book.keybase.io/docs/teams/design
- Discord Support — Permissions Hierarchy: https://support.discord.com/hc/en-us/articles/206141927-How-is-the-permission-hierarchy-structured
- Discord Support — Roles and Permissions: https://support.discord.com/hc/en-us/articles/214836687-Discord-Roles-and-Permissions
- UCAN spec — capability semantics: https://github.com/ucan-wg/spec
- Oso — RBAC vs ABAC vs ReBAC: https://www.osohq.com/learn/rbac-vs-abac-vs-rebac-what-is-the-best-access-policy-paradigm
- Pangea — RBAC vs ReBAC vs ABAC: https://pangea.cloud/blog/rbac-vs-rebac-vs-abac/
- Pathlock — RBAC comprehensive guide: https://pathlock.com/blog/role-based-access-control-rbac/
- IETF RFC 9420 — MLS Protocol: https://datatracker.ietf.org/doc/rfc9420/
- IETF RFC 9750 — MLS Architecture: https://www.rfc-editor.org/rfc/rfc9750.html

### §9.5 Tree-pinned (HEAD `2172cb6d`)

- `crates/benten-id/INTERNALS.md` — identity primitives
- `crates/benten-sync/INTERNALS.md` — Atrium transport + CRDT
- `crates/benten-crypto-suite/INTERNALS.md` — crypto-suite integration
- `crates/benten-caps/src/chain_authority.rs` — UCAN chain-walker seam (§2.6 composition site)
- `crates/benten-engine/src/engine_sync.rs` — AtriumHandle 12 public methods
- `docs/V1-FROZEN-INTERFACE.md` §15 — frozen surfaces
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` — deferred-with-name destinations
- `docs/INVARIANT-COVERAGE.md` — Inv-14 + Inv-15 + Inv-16-mint + Inv-20 (post-M-CONS)
- `docs/SECURITY-POSTURE.md` — Compromise # registry
- `docs/GLOSSARY.md` — 4-identity-concepts tree per CLAUDE.md #18

### §9.6 CLAUDE.md baked-in + MEMORY.md disciplines

- CLAUDE.md baked-in #1 (12-primitive irreducibility)
- CLAUDE.md baked-in #5 (crypto-agility seam + LAMPS hybrid floor)
- CLAUDE.md baked-in #15 (v1-beta interface freeze)
- CLAUDE.md baked-in #17 (multi-process + device-shape (a)/(b)/(c) + 4-dim CapabilityEnvelope)
- CLAUDE.md baked-in #18 (4-identity-concepts + plugin trust model)
- CLAUDE.md baked-in #19 (engine-level extensions vs app-level plugins)
- `feedback_extra_reflection_pass_for_elegant_permanent_shape.md` — N2's Authority
  generalization IS the elegant-permanent-shape extra-reflection pass against M-CONS's
  KindPolicyAdminEntity (closes 3-row enum → uniform field)
- `feedback_engine_primitives_vs_application_layer.md` — RBAC ships as engine-level
  primitive (not application-layer composition) because it's load-bearing for the
  Authority + check_permission seam
- `feedback_iterate_critical_reviews_to_convergence.md` — sharpened by N2-P2 (§7.2)
- `feedback_no_defer_HARD_RULE.md` — every codepoint-reserve has NAMED destination
  (Phase-4-Meta-Composing+ per §3.1 + §5.3)
- `feedback_handoff_top_banner_re_orient.md` — honored at file top
- §3.5g cross-language rule-mirror — TS-side RoleId + PermissionSet + ErrorCode
  catalog mirrors named in §2.7

---

**End of N2 specialist document.** Output to be reviewed alongside N1 / N3 / N4 by
orchestrator post-spawn-consolidation, then handed to the R0-author for F-full
plan-doc substantive content per the ADDL pipeline.
