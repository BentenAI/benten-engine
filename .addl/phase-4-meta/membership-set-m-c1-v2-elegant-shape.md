# MembershipSet M-CONS-v2 — **M-C1 elegant-shape critique** (extra-reflection pass)

> **Top-banner re-orient (HANDOFF discipline).** This document is the M-C1 elegant-shape
> critique-round output over M-CONS-v2 (`phase-4-meta-core/membership-set-m-cons-v2-consolidator @ e62ff540`
> → `.addl/phase-4-meta/membership-set-m-cons-v2-consolidator.md`, 1295 LOC). Per
> `feedback_extra_reflection_pass_for_elegant_permanent_shape.md` (Ben-ratified 2026-05-25):
> after the per-finding triage, take ONE holistic pass — is there a single elegant
> structural shape M-CONS-v2 missed that closes N findings at once with strictly less
> code + forward-class-of-bug closure?
>
> M-C1 is one of three parallel critique-round agents (M-C1 elegant-shape /
> M-C2 composability / M-C3 fresh-eyes). M-C1 does NOT re-do M-C2/C3's work.
>
> Read BEYOND what's named here if you arrive cold: M-CONS-v1 baseline at `74580ee6`,
> N1 sub-graph-elegance at `ed592770`, N2 generic-primitive + RBAC + ops at `a1b5a552`,
> N3 continuous-rotation edge cases at `298c80d9`, N4 SSK comparison at `2ee24e9f`;
> the 3-cataloger + 5-specialist M-panel; the 5 prior critique-round outputs;
> Q3-revisit `23f76e24`; AtriumPolicy `e90900b4`; Path-A.5 winner `5f50a028`;
> CLAUDE.md baked-in #1/#5/#15/#17/#18/#19 + MEMORY.md disciplines (engine-primitives-
> vs-application-layer, no-defer HARD RULE, iterate-to-convergence, elegant-shape-
> reflection-pass, plain-English-with-prediction).
>
> Tree HEAD at write: `2172cb6d` (origin/main; fetched + verified at session start).
> Branch: `phase-4-meta-core/membership-set-m-c1-v2-elegant-shape`.
> Date: 2026-05-28.

- **Role.** M-C1 critique-round agent. Extra-reflection elegant-shape pass over
  M-CONS-v2's complete 31-F-amendment + 26-Compromise + 10-clause-Inv-20 +
  per-Kind walkthrough.
- **Posture.** ADVISORY. Each elegant-shape candidate is presented with: where it
  would land vs M-CONS-v2; what it closes; cost vs benefit; my-pred RECOMMEND-ADOPT
  / NAMED-DEFER / DISAGREE-WITH-EXPLANATION; confidence.
- **Scope.** Tasks 1–5 per the dispatch brief: cluster F-amendments by structural
  property; hunt for missed elegant shapes; NAMED-defer rejected candidates;
  identify over-fitting / accidental-complexity; honest assessment if M-CONS-v2
  is already complete.
- **Out-of-scope.** Composability (M-C2 territory). Fresh-eyes (M-C3 territory).
  Re-litigation of M-CONS-v2 ratified shapes. Authoring R0 substantive content.

---

## §0 Headline (read-cold)

**Verdict.** M-CONS-v2 captures the substantive elegant shapes already. **No
single structural collapse closes ≥3 amendments at once with strictly less code.**
Of the 8 candidates investigated, **3 are RECOMMEND-ADOPT (small refinements,
not redux collapses); 1 is RECOMMEND-NAMED-DEFER with explicit trigger; 4 are
DISAGREE-WITH-EXPLANATION (cost > benefit OR already-applied-in-M-CONS-v2).**

The 3 RECOMMEND-ADOPT candidates are:

1. **C-A1 — Collapse `authorities: BTreeSet<Authority>` + `role_assignments: BTreeMap<Did, RoleId>` into a SINGLE `members_table: BTreeMap<Did, MemberEntry>` keyed-by-DID.** Eliminates the cross-table invariant "if Admin in role_assignments → must also be in authorities" by construction, since one DID → one MemberEntry holds BOTH role + sig_pubkey (Option for non-admins). Net ~-40 LOC; -1 constructor-validator clause; -1 class-of-bug (cross-table drift); preserves wire-shape via canonical CBOR encoding. **my-pred RECOMMEND-ADOPT; HIGH confidence.**
2. **C-A2 — Add `disposition_class: { AcceptedTradeOff | SubstrateGuaranteeDisclosure }` field to every Compromise # registry entry.** Codifies M-CONS-v2 §11.4 Pattern P14 into the registry shape itself. Closes the "is #55 a vulnerability or a substrate guarantee?" audit-firm confusion at the data-model level. ~+10 LOC SECURITY-POSTURE template; ~26 lines of one-word field-adds across the registry. **my-pred RECOMMEND-ADOPT; HIGH confidence.**
3. **C-A3 — Move `MembershipSetMetadata` from inside `MembershipSetPolicy` to TOP-LEVEL on `MembershipSet`.** Metadata is admin-mutable but NOT security-relevant; bundling under Policy mis-couples it with policy_version + authorities. Lifting it to top-level (with admin-mutation still admin-gated via op-17 `rename`/`update_metadata`) cleans the policy_version concern (metadata-rename should not need policy_version bump). Net ~0 LOC delta; -1 mis-coupling. **my-pred RECOMMEND-ADOPT; MED-HIGH confidence.**

The 1 RECOMMEND-NAMED-DEFER:

- **C-D1 — Inv-20 6-cluster grouping doc-shape.** The 10 clauses naturally cluster into 6 structural property groups: {a,b,e}=key-lifecycle; {c,d}=stanza-security; {f}=API-boundary-binding; {g}=transport; {h}=recipient-shape; {i,j}=governance. Re-rendering Inv-20 with cluster headings preserves all 10 clauses (no semantic change) + improves audit-pack readability. NAMED-DEFER to Wave-MS-PRIMITIVE final integration doc-pass; bounded cost ~+0.1 wave-day; revisit-trigger = SECURITY-PROOFS.md authoring detects clause-cross-reference fatigue.

The 4 DISAGREE-WITH-EXPLANATION:

- **C-X1 (associated-type-trait shape).** Rejected: wire-format requires runtime
  Kind-tag (CBOR `kind: u8`) regardless; phantom-type parameter ripples through
  Drop + Caps + Sync downstream types; net COST > BENEFIT. N2 §1.3 already
  considered + rejected pure-generic for the same reason.
- **C-X2 (Compromise #54-#56 family-root collapse).** Rejected: per-M-CONS-v2 §11.4
  Pattern P14, these are structurally distinct (`AcceptedTradeOff` for #54 + #56;
  `SubstrateGuarantee` for #55). Family-root collapse hides the categorical
  difference auditors NEED to see. C-A2 (`disposition_class` field) is the elegant
  shape that REPLACES this family-collapse temptation.
- **C-X3 (`RotationPolicy { ForkOnly | ExplicitRotation { trigger } }` enum
  collapse).** Rejected: M-CONS-v2 already folded `AdminKickEpoch → ForkOnly` at
  v1-beta (one rotation operator). `RotationTrigger` is already the audit-event
  classifier on `MembershipEvent::AtriumFork`. Adding an `ExplicitRotation` enum
  arm at v1-beta-wire would PRE-COMMIT a future-shape (`AtriumWithRotatingGroupKey`)
  that explicitly belongs at codepoint-reserve — violates CLAUDE.md #15.
- **C-X4 (DUAL-CID Option I generalization ship-at-v1-beta).** Rejected: DUAL-CID
  Q3 Option D is already ratified for v1-beta; Option I (`Option<DedupScopeId>`
  generic-shape) is codepoint-reserve-able post-v1-beta via additive codepoint;
  early generalization adds wire-shape complexity that buys no current use-case.
  Codepoint-reserve discipline (M-CONS-v2 §11.8 Pattern P18) is the correct lever.

**Reduction recommendation summary.** **3 small-refinement ADOPTs** (C-A1 + C-A2
+ C-A3); **1 doc-shape NAMED-DEFER** (C-D1); **4 candidates rejected with explanation.**
Cumulative net wave-day impact: **~–0.3 to +0.2 wave-days** (C-A1 saves
constructor-validator code + a class of bug; C-A2 + C-A3 are bounded refinements
matching existing PIM disciplines).

**Confidence on "no missed structural redux."** HIGH. M-CONS-v2's M2→M-CONS-v1→N1–N4
pipeline + M-CONS-v2 §11.7 P17 ("convergent + parallel-N + Ben-refinement-pass is
optimal") already does the work of catching shapes a single agent would miss.
N2 caught the load-bearing Authority generalization the M-panel missed; the
remaining candidates in this critique are bounded refinements, not redux-class
collapses.

**Open Ben-call surfaces.** 3 minor Ben-calls from this critique (see §6); all
non-arch-fork-class per `feedback_surface_arch_decisions_under_auth.md`; all
surfaceable in one pass alongside the M-CONS-v2 §9.3 4-call bundle (so 7 total
in one Ben-pass).

---

## §1 Task 1 — Cluster the 31 F-amendments by structural property

Per the brief, group F1–F28 + F-N2-A/B/C by the structural property each one
targets. Clusters surface where M-CONS-v2 has REAL redux already (cluster
already-collapsed) vs where elegant-shape candidates might still exist
(cluster not-collapsed).

### §1.1 Cluster table

| Cluster | F-rows | Structural property | M-CONS-v2 status | Elegant-shape potential |
|---|---|---|---|---|
| **C1 — AAD-binding completeness** | F1 (codepoint-in-AAD), F4 (sender+member-list), F5 (replay-window), F14 (aad_version) | Every encrypt-to-N call binds AAD to the FULL provenance tuple | **ALREADY COLLAPSED** under Inv-16 + Inv-20 clause-c (load-bearing for N4 inter-member non-forgeability) | NONE — the property is already named in one place |
| **C2 — Codepoint-permanence + lifecycle** | F8 (registry IANA-disjoint), F11 (escape codepoint), F16 (CodepointLifecycle typed-state) | Codepoint values are once-allocated, never repurposed; lifecycle is typed | **ALREADY COLLAPSED** under Inv-18a + Inv-18c | NONE |
| **C3 — Forward-additivity (codepoint-reserve)** | F11 (experimental range), F13 (MLS-PQ + AtriumWithRotatingGroupKey), F21 (Sealed-Sender slot), F27 (TransportConfig reserves), F28 (RestrictedScopeSet backward-compat dispatch) | New codepoints land without breaking old verifiers | **ALREADY COLLAPSED** under M-CONS-v2 §11.8 Pattern P18 (codepoint-reserve recursive) | NONE — pattern is doctrinal; reserves track per-row |
| **C4 — MembershipSet primitive operations** | F17 (primitive crate), F19 (generation CRDT), F-N2-B (24 ops + RotationTrigger) | One crate owns membership state-shape + lifecycle | **ALREADY COLLAPSED** into `benten-membership-set` crate | C-A1 candidate (authorities + role_assignments cross-table) |
| **C5 — Per-Kind threat-model + UX** | F17 typed-variants, F24 (per-Kind Argon2id tiers), F25 per-Kind discipline, F26 D1-D7 per-Kind, F-N2-C (per-Kind metadata) | Kind discriminator gates per-Kind threat-model + UX dispatch | **ALREADY COLLAPSED** under MembershipSetKind 3-arm; per-Kind constructor pattern | C-A3 candidate (metadata at top-level vs inside policy) |
| **C6 — Multi-stanza fanout + wire format** | F2 (strict-decode), F3 (canonical-TLV), F6 (Decap-CT mitigation), F9 (EnvelopePayload non_exhaustive), F10 (BindingContext non_exhaustive), F12 (nonce-length), F23 (DAG-CBOR + deterministic decode) | Wire format binds-once + decodes-strictly + AEAD-mitigated | **ALREADY COLLAPSED** under Inv-15 + Inv-16 | NONE |
| **C7 — DUAL-CID** | F18 (triple-CID parameterized over MembershipSet; Q3 Option D) | Plaintext-CID vs dedup-blinded-CID vs envelope-CID separation | **ALREADY COLLAPSED** at Q3 Option D ratification | C-X4 candidate (Option I generalization; REJECTED) |
| **C8 — Authority + RBAC** | F-N2-A (Authority + RBAC + UCAN intersection-of-allows) | Single principal-set model across Kinds | **PARTIALLY COLLAPSED** under N2 generalization; **C-A1 candidate** (authorities + role_assignments fusion) |
| **C9 — Sub-graph access (RestrictedScopeSet)** | F28 (RestrictedScopeSet at Scope::RestrictedSelector arm) | Sub-graph asymmetric-shape via union-of-RestrictedScope | **ALREADY COLLAPSED** via N1's combinator-surface-promotion | NONE — N1 itself is the elegant shape |
| **C10 — Transport (iroh-gossip scope-in + codepoint-reserves)** | F27 (TransportConfig + iroh-gossip impl) | Wire-format frozen + per-Kind transport substrate | **ALREADY SCOPED** at Ben Q1 SCOPE-IN | NONE — separation from primitive wave is structurally correct |
| **C11 — Rotation primitives** | F17 FORK-ONLY rotation, F-N2-B fork op + RotationTrigger doc-enum | One rotation operator; trigger classified at audit level | **ALREADY COLLAPSED** under fold AdminKickEpoch → ForkOnly | C-X3 candidate (RotationPolicy enum; REJECTED) |
| **C12 — Identity primitives** | F15 (Did multikey + Unknown), F24 NAPI MembershipSetHandle | Identity surface + opaque-handle for NAPI | **ALREADY COLLAPSED** under Did multikey | NONE |
| **C13 — Forward-secrecy disclosure family** | F13 (FS-gap + MLS-PQ + AtriumWithRotatingGroupKey codepoint-reserve) | Honest disclosure that v1-beta is no-FS against future K_Set compromise | **ALREADY ABSORBED** under Compromise #42 + #54 + #55 + #56 | C-A2 candidate (Compromise disposition_class field) |
| **C14 — Audit-deliverable pack** | F25 (golden vectors + kani + dudect + per-Kind discipline + audit_commitment + SSK §-row + concurrent-fork tie-break) | One pack lists all audit-deliverable artifacts | **ALREADY COLLAPSED** as F25; SSK §-row absorbed | NONE — pack-as-single-row is the elegant shape already |
| **C15 — Impl-engineering pack** | F24 (libcrux-ml-kem + NAPI + wasm_js cfg + cancel-safety + Argon2idParameterTier) | One row aggregating impl-engineering decisions per primitive | **ALREADY COLLAPSED** as F24 | NONE — pack-as-row is the elegant shape already |
| **C16 — Threat-model disclosures (Compromise family)** | #41/#42/#43/#44/#46/#47/#48/#49/#50/#51/#52/#53/#54/#55/#56 | Disclosure family: each Compromise # is a deferral or substrate guarantee with NAMED revisit trigger | **PARTIALLY COLLAPSED** under M-CONS-v2 P14 (categorical distinction); **C-A2 candidate** (disposition_class field) |

**Cluster count:** 16 clusters cover all 31 F-rows + Compromise #-family + Inv-20.
**Already-collapsed cluster count:** 13/16 (81%); **elegant-shape candidate
clusters:** 3 (C4 / C5 / C8 → C-A1 + C-A3; C16 → C-A2; C13 partially covered by
C-A2); plus 3 REJECTED candidates (C7 → C-X4; C11 → C-X3; C4 alt → C-X1).

### §1.2 Cluster verdict

**Most F-rows are already single-property representatives within their cluster.**
The clustering reveals M-CONS-v2's substantive consolidation work — F25 + F24
are pack-rows aggregating ~5+ sub-items each; F17 is the primitive-crate row;
F26 + F-N2-A consolidate per-Kind policy concerns; F28 lifts a combinator
already in-tree to wire-boundary.

**The 3 remaining elegant-shape candidates are narrow** (one cross-table fusion,
one disposition-class field, one struct-field-move). None close ≥3 amendments
simultaneously. This is consistent with M-CONS-v2's headline verdict that the
pipeline already caught the substantive generalizations.

---

## §2 Task 2 — Hunt for additional elegant shapes M-CONS-v2 missed

Each candidate is presented as: candidate description → what it would close →
cost vs benefit → my-pred → confidence.

### §2.1 C-A1 — Fuse `authorities` + `role_assignments` into single `members_table`

**Status:** my-pred **RECOMMEND-ADOPT**. **Confidence: HIGH.**

**Candidate description.** M-CONS-v2 §1.2 (via N2 §1.4 + §4.1) ships:

```rust
pub struct MembershipSetPolicy {
    pub policy_version: u32,
    pub authorities: BTreeSet<Authority>,            // {did, sig_pubkey, role, admitted_at_hlc}
    pub role_assignments: BTreeMap<Did, RoleId>,     // member_did → role
    pub kind_policy: KindPolicySubfields,
    pub refresh_required_secs: Option<u64>,
    pub threshold_admin: Option<ThresholdAdminSpec>,
    pub metadata: MembershipSetMetadata,
}
```

With M-CONS-v2 §1.2 + N2 §1.5 invariant: "if a DID's role in `role_assignments`
is `Admin`, the DID MUST also appear in `authorities`."

**Observation.** The two fields ENCODE OVERLAPPING STATE. Every DID with
`role = Admin` appears in BOTH:
- `authorities` (carrying its `sig_pubkey` for signature verification)
- `role_assignments` (carrying its role tag)

The invariant is enforced at constructor + mutation-op level (runtime), creating
a class of bug: cross-table-drift between authorities and role_assignments under
concurrent mutations or wire-decode of an externally-constructed `MembershipSet`.

**Proposed elegant shape.** Collapse both into ONE table keyed by DID:

```rust
pub struct MembershipSetPolicy {
    pub policy_version: u32,
    pub members_table: BTreeMap<Did, MemberEntry>,
    pub kind_policy: KindPolicySubfields,
    pub refresh_required_secs: Option<u64>,
    pub threshold_admin: Option<ThresholdAdminSpec>,
    // metadata lifted to top-level per C-A3
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberEntry {
    pub role: RoleId,
    /// REQUIRED iff role == RoleId::Admin. None otherwise. Constructor-validated.
    pub sig_pubkey: Option<HybridSigPubkey>,
    pub admitted_at_hlc: BentenHlc,
}
```

OR, alternative shape preserving the typed-`Authority` distinction:

```rust
pub members_table: BTreeMap<Did, MemberEntry>,

pub enum MemberEntry {
    /// Non-admin member: role ∈ {Member, Viewer}; no sig_pubkey needed.
    NonAdmin { role: RoleId, admitted_at_hlc: BentenHlc },
    /// Admin: carries sig_pubkey for mutation-signature verification.
    Admin { sig_pubkey: HybridSigPubkey, admitted_at_hlc: BentenHlc },
}
```

(The Rust enum shape is preferable — it makes the "Admin has sig_pubkey; non-admin
does not" invariant TYPE-LEVEL rather than constructor-validated runtime.)

**What this closes.**
- Cross-table-drift class-of-bug (Inv-20 clause-i + clause-j compose-correctness):
  Eliminated by-construction. One DID has exactly one MemberEntry.
- Constructor-validator clause "Admin in role_assignments → must be in authorities":
  Eliminated; the variant tag IS the proof.
- "Admin DID appears twice" wire-decode hazard: Eliminated.
- 1 line of cross-table reasoning in every mutation op (add_member, change_role,
  remove_member): replaced with single-table-update.
- 2 SECURITY-POSTURE.md threat-model rows (cross-table drift + concurrent-mutation
  inconsistency) → 0 rows.

**Cost.**
- Wave-format CBOR encoding: change from `auth: [...]` + `roles: {...}` two-map
  to `members: {<did>: {role, [sig_pubkey?], admitted}}` one-map. Wire-shape is
  STRICTLY SMALLER (no DID-string duplicated across two tables).
- Per-Kind constructor: cardinality-of-Admin check still needed (Atrium ≥1 Admin;
  DeviceMesh exactly-1 Admin; SingleDevice exactly-1 Admin) but reduces to ONE
  iteration over `members_table` filtering `role = Admin`, vs current 3-way
  check across authorities + role_assignments + per-Kind cardinality.
- Threshold-admin spec (`ThresholdAdminSpec.eligible_admins`): unaffected; still
  a separate field referencing DIDs in `members_table`.
- TS mirror: change `Authority[]` + `RoleAssignments` to `Map<Did, MemberEntry>`;
  bounded ~50 LOC adjustment in `errors.generated.ts` + member-shape exports.

**Net LOC delta.** Estimate **~–40 to –60 LOC** in `benten-membership-set` +
**~–30 LOC** in TS mirror = **~–70 to –90 LOC net.** Plus removal of 1 invariant
clause + 2 threat-model rows in audit-pack.

**Wave-day delta vs M-CONS-v2.** **~–0.3 wave-days** (saves constructor + mutation
op code; removes invariant + audit-doc rows). Wave-MS-PRIMITIVE canary absorbs
within budget (~10.5–13 → ~10.2–12.7).

**my-pred RECOMMEND-ADOPT.** This is the cleanest elegant shape M-C1 found.
Closes ONE invariant clause + ONE class-of-bug (cross-table drift) with strictly
less code + strictly smaller wire shape.

**Confidence: HIGH.** Per-Kind cardinality validation is preserved; type-system
enforces Admin-has-sig_pubkey via enum variant; wire-shape backward-compat is
N/A (this is pre-freeze; v1-beta wire format is in M-CONS-v2 own scope, not
post-hoc migration).

**Caveat: Ben should confirm.** This is a structural change to the N2 §4.1 canonical
Rust shape Ben ratified at "24-op classification as-is" level. The 24 OPS are
preserved (no op changes); the POLICY STRUCT internal shape changes. Per
`feedback_surface_arch_decisions_under_auth.md`, this is non-arch-fork-class
(internal struct reorganization), but worth one Ben-pass confirmation.

### §2.2 C-A2 — Compromise registry `disposition_class` field

**Status:** my-pred **RECOMMEND-ADOPT**. **Confidence: HIGH.**

**Candidate description.** M-CONS-v2 §11.4 Pattern P14 identifies that Compromise
# registry entries carry TWO categorically different shapes:
- (a) `AcceptedTradeOff` — "vulnerability we accept as a trade-off" (e.g., #42
  Layer-C FS-gap; #46 wire-cost; #52 no-PCS-against-removed-members; #54
  continuous-rotation deferral; #56 journalist per-message FS deferral).
- (b) `SubstrateGuaranteeDisclosure` — "architectural substrate guarantee we
  honestly disclose; applications layer compensating controls" (#55 GDPR-RTBF).

P14 recommends adding `disposition_class` to the SECURITY-POSTURE.md Compromise
# template. The codification has NOT yet been applied to the registry itself —
it's still recommendation-level.

**Proposed elegant shape.** Add the field to the registry CBOR/JSON shape (or
inline-prose template) NOW as part of Wave-MS-PRIMITIVE final-doc step, not
later.

```markdown
### Compromise #55 — GDPR right-to-be-forgotten honest-architectural-disclosure
- **disposition_class:** SubstrateGuaranteeDisclosure
- **statement:** P2P-by-design semantic: ...
- **revisit_trigger:** N/A (substrate property; not deferred)
- **applications_layer_response:** per-subject crypto-shredding sub-encryption ...
```

vs.

```markdown
### Compromise #54 — Continuous-rotation deferral risk
- **disposition_class:** AcceptedTradeOff
- **statement:** Silent-undetected-CURRENT-member-compromise + one-shot key
  leakage are NOT defended at v1-beta...
- **revisit_trigger:** journalism / AI-agent / HIPAA-PCI / production-incident
  watch-list per N3 §7.4
- **named_destination:** post-v1-beta AtriumWithRotatingGroupKey
```

**What this closes.**
- Audit-firm clustering hazard: an auditor processing the Compromise # registry
  cannot tell "which of these are vulnerabilities you'll fix" vs "which are
  substrate properties you're disclosing." The field surfaces this categorically.
- Future-Compromise # mints have unambiguous classification template.
- Aligns SECURITY-POSTURE.md surface with the audit-firm reading model.

**Cost.** ~+1 line per registry entry × 26 entries = ~26 lines of template + ~10
LOC if any tool processes the registry programmatically. SECURITY-POSTURE.md
~+0.5 page (per-class explainer at top).

**Net wave-day delta vs M-CONS-v2.** **~+0.1 wave-days** (bounded doc-pass at
Wave-MS-PRIMITIVE final integration; absorbed within F25 audit-deliverable pack
budget).

**my-pred RECOMMEND-ADOPT.** Codifying M-CONS-v2's own P14 pattern into the
registry shape NOW (rather than naming-it-as-a-pattern-to-be-applied-later) is
the right elegant shape. The "this is a substrate property" vs "this is a
deferred trade-off" distinction is load-bearing for audit-firm reading.

**Confidence: HIGH.** Codification matches existing PIM disciplines (§3.6i
canonical JSON schema; §3.5g cross-language rule-mirror); template field-add
is bounded; carries no wire-shape impact.

### §2.3 C-A3 — Lift `MembershipSetMetadata` to top-level on `MembershipSet`

**Status:** my-pred **RECOMMEND-ADOPT**. **Confidence: MED-HIGH.**

**Candidate description.** M-CONS-v2 + N2 §4.1 places `MembershipSetMetadata`
INSIDE `MembershipSetPolicy`:

```rust
pub struct MembershipSetPolicy {
    pub policy_version: u32,
    pub authorities: BTreeSet<Authority>,
    ...
    pub metadata: MembershipSetMetadata,   // ← inside Policy
}
```

This means renaming the Atrium ("Marketing Q1" → "Marketing FY26") technically
requires policy_version increment + admin-mutation discipline + admin-replay
defense semantics (D1 in M-CONS-v2 §6).

**Observation.** Metadata (name / description / created_by_human_label) is
admin-mutable but NOT security-relevant — it doesn't gate any cryptographic
property, doesn't bind into AAD, doesn't affect K_Set lifecycle. Bundling it
under Policy mis-couples it with policy_version + admin-replay defenses that
EXIST for cryptographic-policy mutations.

If `MembershipSetMetadata` lived at top-level on `MembershipSet`, the rename
op-17 could carry its own bump (`metadata_version: u32`) independent of
`policy_version`, AND admin-signing discipline is still enforceable via the
`MembershipEvent::MetadataChanged` audit-event (carrying the authority's
signature) without needing a full policy_version bump.

**Proposed elegant shape.**

```rust
pub struct MembershipSet {
    pub version: u8,
    pub kind: MembershipSetKind,
    pub id: Option<MembershipSetId>,
    pub members: Vec<MemberKey>,
    pub shared_key: KSet,
    pub policy: MembershipSetPolicy,         // policy_version inside; no metadata
    pub metadata: MembershipSetMetadata,     // ← lifted top-level
    pub parent_membership_set_id: Option<MembershipSetId>,
    pub created_at_hlc: BentenHlc,
    pub membership_attestation: MembershipAttestation,
    pub transport_config: TransportConfig,
}

pub struct MembershipSetMetadata {
    pub metadata_version: u32,               // ← independent bump
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_by_human_label: Option<String>,
}
```

**What this closes.**
- Mis-coupling between metadata-rename and policy-version: eliminated.
- Audit-event clarity: `MembershipEvent::MetadataChanged` doesn't need to read
  policy_version to determine if the rename is the only change (currently has
  to differentiate metadata-only-bump from full-policy-rebind).
- D1 walkthrough simpler: policy_version_at_seal binds cryptographic-policy
  semantic only; metadata changes don't pollute the policy_version space.

**Cost.** ~0 LOC change (one field-move + one independent u32 field). Wire
shape: change CBOR layout slightly — metadata key moves from `pol.meta` to
top-level `meta`. Pre-freeze, this is a free swap.

**Net wave-day delta.** **~0 wave-days** (refactor within canary scope).

**my-pred RECOMMEND-ADOPT.** This is the cleanest separation-of-concerns
elegant shape: security-policy ≠ human-label-metadata. The current bundling
came from "policy holds everything admin-mutable"; the cleaner rule is
"policy holds everything cryptographic-policy-mutable; metadata holds
human-label-mutable."

**Confidence: MED-HIGH.** D1 (policy_version_at_seal in AAD) is unaffected;
metadata-version doesn't need to be in AAD because metadata doesn't gate
crypto. The independent bump is a nice property for replay-defense composition.

**Caveat: Ben should confirm.** This is a tiny re-organization of N2 §4.1
canonical shape. Same Ben-confirmation pass as C-A1.

### §2.4 C-D1 — Inv-20 6-cluster re-rendering (NAMED-DEFER)

**Status:** my-pred **RECOMMEND-NAMED-DEFER**. **Confidence: MED-HIGH.**

**Candidate description.** Inv-20 currently lists 10 clauses (a–j). The brief asks
whether some clauses are really sub-clauses of a single structural property.

**Observation.** The 10 clauses cluster into 6 structural property groups:

| Cluster | Clauses | Structural property |
|---|---|---|
| **Key-lifecycle** | a (shared_key), b (FORK-ONLY rotation), e (generation CRDT-vector) | K_Set is shared via multi-stanza HPKE; rotates via fork; generation-vector witnesses CRDT-partition |
| **Stanza-security** | c (AAD-binding tuple), d (per-recipient unlinkability) | Each stanza binds full provenance + unlinkability across recipients |
| **API-boundary K(V)** | f (Path-A.5 K(V) discipline) | Compile-time restriction at API boundary |
| **Transport** | g (TransportConfig + iroh-gossip impl) | Wire-format codepoint + impl |
| **Recipient-shape** | h (homogeneous-per-Kind MemberKey + RestrictedScopeSet seal-seam) | Per-Kind recipient invariants + sub-graph composition |
| **Governance** | i (Authority slot), j (3-role RBAC + UCAN intersection) | Mutation-authority + permission model |

**Proposed elegant shape.** Re-render Inv-20 in SECURITY-PROOFS.md + INVARIANT-COVERAGE.md
with cluster headings:

```markdown
### Inv-20 — MembershipSet primitive invariant (10 clauses, 6 clusters)

**Cluster 1 — Key-lifecycle**
- (a) shared_key K_Set distributed via multi-stanza HPKE-Encap
- (b) FORK-ONLY rotation (AdminKickEpoch folded into FORK-ONLY)
- (e) generation-CRDT-vector per-member-DID partition

**Cluster 2 — Stanza-security**
- (c) AAD-binding tuple ... (load-bearing for inter-member non-forgeability)
- (d) per-recipient unlinkability

[etc.]
```

**What this closes.**
- Reader fatigue at clause-cross-reference (e.g., "Inv-20 clause-c → N4 §3 R-N4-3
  → SSK-comparison §-row" — readability win if clauses cluster).
- Audit-pack rendering benefits from grouping that mirrors auditors' mental model.

**Why NAMED-DEFER not RECOMMEND-ADOPT.** The 10-clause flat list is the canonical
ratified shape; re-rendering is a doc-pass that lands at Wave-MS-PRIMITIVE final
integration (alongside SECURITY-PROOFS.md authoring). Doesn't change shape;
doesn't close a class-of-bug. Bounded cost; no urgency.

**NAMED destination:** Wave-MS-PRIMITIVE final integration doc-pass (M-CONS-v2
§7.1 last sub-row). **Revisit-trigger:** SECURITY-PROOFS.md authoring agent
encounters clause-cross-reference fatigue OR audit-firm reading model surfaces
clustering benefit.

**my-pred RECOMMEND-NAMED-DEFER per HARD RULE 12 clause-b.** Has a real
destination + a real revisit-trigger. Not "later" or "Phase-N follow-up."

### §2.5 C-X1 — Associated-type-trait shape (REJECTED)

**Status:** my-pred **DISAGREE-WITH-EXPLANATION**. **Confidence: HIGH.**

**Candidate description.** Brief asks: would a Rust associated-type-trait shape

```rust
trait MembershipSetKindTrait {
    type Policy;
    type Member;
    type Authority;
    type RotationSemantic;
    const KIND_TAG: u8;
}

struct AtriumKind; struct DeviceMeshKind; struct SingleDeviceKind;

impl MembershipSetKindTrait for AtriumKind {
    type Policy = AtriumPolicy;
    type Member = UserDidMember;
    type Authority = AtriumAuthority;
    type RotationSemantic = AtriumFork;
    const KIND_TAG: u8 = 0;
}

struct MembershipSet<K: MembershipSetKindTrait> {
    members: Vec<K::Member>,
    policy: K::Policy,
    authorities: BTreeSet<K::Authority>,
    ...
}
```

collapse the per-Kind variation more elegantly than M-CONS-v2's enum-with-policy-tag?

**Detailed evaluation.**

**Substrate constraints that defeat associated-type-trait:**

1. **Wire-format requires runtime Kind-tag.** The CBOR shape (M-CONS-v2 §1.2)
   carries `"kind": 0|1|2` as a u8 discriminant. CBOR decoders need to dispatch
   on this tag at runtime to choose the right sub-shape decoder. Associated-type-
   trait would still need:

   ```rust
   enum MembershipSetDyn {
       Atrium(MembershipSet<AtriumKind>),
       DeviceMesh(MembershipSet<DeviceMeshKind>),
       SingleDevice(MembershipSet<SingleDeviceKind>),
   }
   ```

   ...which IS the current `MembershipSetKind` enum + per-Kind dispatch.
   Associated-type-trait adds machinery without removing the existing dispatch.

2. **Phantom-type-parameter ripple.** Every downstream type that holds a
   `MembershipSet` would need either:
   - A generic parameter (`Drop<K: MembershipSetKindTrait>`, etc.) → ripple
     across `benten-drop`, `benten-caps`, `benten-sync`, `benten-engine`.
   - A dyn-Trait box → defeats the compile-time benefit.
   - The `MembershipSetDyn` enum wrapper → same as current enum.

3. **NAPI bindings.** `MembershipSetHandle` (per F24) is an opaque handle
   exposed to TS. Associated-type-trait requires N×NAPI surfaces (per-Kind
   handle) — defeats the uniform-handle property.

4. **Per-Kind constructor invariants.** N2 §4.4 lists per-Kind cardinality +
   invariant checks (e.g., "Atrium ≥1 admin"). These are RUNTIME checks
   because the Did set is wire-decoded. Type-system-enforced
   constant-cardinality-per-Kind would force separate types per "DeviceMesh-
   exactly-1" vs "Atrium-≥1" — defeats the uniform Authority slot from N2.

5. **N2 §1.3 substantive objections.** N2 already considered + rejected
   pure-generic; the hybrid (typed-Kind discriminator + uniform-Authority slot)
   is the right balance. Associated-type-trait is a STRICTLY MORE GENERIC
   variant of the same idea N2 rejected for the same reasons.

**Cost vs benefit.**
- COST: +500-800 LOC of generic-parameter ripple in benten-drop + benten-caps
  + benten-sync; NAPI surface fragmentation; phantom-type confusion at API
  boundaries.
- BENEFIT: minimal — type-system enforces what runtime check already does
  cheaply (~10 LOC constructor).

**my-pred DISAGREE-WITH-EXPLANATION.** The hybrid (typed-Kind enum + uniform-Authority)
is the right elegant shape; associated-type-trait would be over-engineering at
this primitive level. Reuses N2 §1.3 substantive objections.

**Confidence: HIGH** that this is the right call. The N2 panel + Ben's
refinement-pass already converged on this; M-C1's pass confirms.

### §2.6 C-X2 — Compromise #54-#56 family-root collapse (REJECTED)

**Status:** my-pred **DISAGREE-WITH-EXPLANATION**. **Confidence: HIGH.**

**Candidate description.** Brief asks: "many of these may be one Compromise-family-
root with downstream sub-rows (analogous to Compromise #31's pattern)."

**Evaluation.** #54 + #55 + #56 are structurally distinct, NOT a single
family-root:

| # | Title | Cluster |
|---|---|---|
| #54 | Continuous-rotation deferral risk | AcceptedTradeOff (defense-deferred-to-post-v1-beta-`AtriumWithRotatingGroupKey`) |
| #55 | GDPR RTBF P2P-by-design disclosure | SubstrateGuaranteeDisclosure (NOT a vulnerability; applications layer compensating controls) |
| #56 | Journalist per-message FS deferral | AcceptedTradeOff (separate design class from #54; per-message ephemeral-key vs group-key rotation) |

**Per M-CONS-v2 §11.4 Pattern P14**, #55 is categorically different from #54 +
#56. Family-root collapse would HIDE this distinction — exactly the audit-firm
clustering hazard P14 warns against.

#54 + #56 ARE both `AcceptedTradeOff` class, but they're STRUCTURALLY DIFFERENT
trade-offs: #54 is group-key-rotation-cadence; #56 is per-message-ephemeral-key-
discipline. M-CONS-v2 explicitly notes "auditor would otherwise conflate" #54 +
#56 → mint separately for clarity.

**my-pred DISAGREE-WITH-EXPLANATION.** C-A2 (`disposition_class` field per §2.2)
is the elegant shape that REPLACES the family-collapse temptation — it surfaces
the categorical distinction at the data-model level without forcing the registry
to lose row-granularity.

**Confidence: HIGH.** P14 codification directly applies; C-A2 is the right
elegant move.

### §2.7 C-X3 — `RotationPolicy { ForkOnly | ExplicitRotation { trigger } }` enum collapse (REJECTED)

**Status:** my-pred **DISAGREE-WITH-EXPLANATION**. **Confidence: HIGH.**

**Candidate description.** Brief asks: could M-CONS-v2 collapse to a single
rotation-operator-with-policy-flag?

**Evaluation.**

M-CONS-v2 already folded `AdminKickEpoch → ForkOnly` at v1-beta (per M5 +
M-CONS-v1). **There is only ONE rotation operator at v1-beta wire** —
`MembershipSet::fork()`. `RotationTrigger` is the audit-event classifier on
`MembershipEvent::AtriumFork`, NOT a wire-format enum.

Adding an `ExplicitRotation { trigger: RotationTrigger }` variant at v1-beta-wire
would PRE-COMMIT a future-shape (`AtriumWithRotatingGroupKey` continuous-rotation)
that explicitly belongs at codepoint-reserve per F13 + #54.

**This violates CLAUDE.md #15** (v1-beta interface freeze: don't commit to a
shape we don't ship). The codepoint-reserve for AtriumWithRotatingGroupKey is
already the right shape per N4 R-N4-1 + M-CONS-v2 §11.8 P18.

**my-pred DISAGREE-WITH-EXPLANATION.** M-CONS-v2 already has the right shape:
- Wire: ONE rotation op (`fork`).
- Audit: `RotationTrigger` doc-enum classifies trigger reason on the audit event.
- Future: AtriumWithRotatingGroupKey codepoint-reserve + RotatingGroupKeyChainedMode
  sub-slot covers future continuous-rotation use-cases.

Adding a `RotationPolicy` enum at v1-beta is a temptation that the
codepoint-reserve discipline correctly resists.

**Confidence: HIGH.** This is exactly the trap CLAUDE.md #15 + P18 protect against.

### §2.8 C-X4 — DUAL-CID Option I generalization at v1-beta (REJECTED)

**Status:** my-pred **DISAGREE-WITH-EXPLANATION**. **Confidence: MED-HIGH.**

**Candidate description.** Brief asks: could Option I (`Option<DedupScopeId>` generic-
shape) ship at v1-beta as the generic shape, with Option D Atrium-blinded as one
DedupScopeId variant?

**Evaluation.**

Q3 Option D (Atrium-blinded dedup-CID) is already ratified for v1-beta. Option I
is NAMED-deferred. Promoting Option I to v1-beta would:

- Add an enum at wire: `enum DedupScopeId { AtriumBlinded { atrium_id }, OtherScopeKindFuture { ... } }`.
- Pre-commit to a generic-shape we haven't designed beyond the Atrium-blinded
  case.
- Add codepoint complexity (one more codepoint family for DedupScopeId variants).
- Buy NO current use-case (the only v1-beta-shipping variant is Atrium-blinded).

**The correct elegant shape is the existing codepoint-reserve discipline** (M-CONS-v2
§11.8 P18). At v1-beta: ship Option D under one codepoint; reserve codepoints for
future DedupScopeId variants (e.g., "user-scoped dedup", "device-scoped dedup")
as separate codepoint additions when use-cases materialize.

This matches the recursive codepoint-reserve pattern N4 R-N4-2 demonstrated for
`RotatingGroupKeyChainedMode`.

**my-pred DISAGREE-WITH-EXPLANATION.** Option I belongs at codepoint-reserve, not
at v1-beta wire-shape. Early generalization adds wire-shape complexity that
buys no current use-case; codepoint-reserve preserves the optionality cheaply.

**Confidence: MED-HIGH.** Slight uncertainty because future DedupScopeId variants
may benefit from compositional shape, but P18 codepoint-reserve discipline
covers the additive-evolution case.

---

## §3 Task 3 — NAMED-deferred elegant alternatives (per HARD RULE 12 clause-b)

Per `feedback_no_defer_HARD_RULE.md`: any rejected elegant alternative gets
NAMED-DEFER with specific destination + revisit-trigger.

| Candidate | NAMED destination | Revisit-trigger |
|---|---|---|
| **C-D1 Inv-20 6-cluster re-rendering** | SECURITY-PROOFS.md authoring step at Wave-MS-PRIMITIVE final integration | Audit-firm reading model surfaces clustering benefit, OR clause-cross-reference fatigue during SECURITY-PROOFS.md authoring |
| **C-X1 Associated-type-trait shape** | Phase-5+ (after `benten-drop` + `benten-caps` mature) | NEVER (substrate-incorrect; N2 §1.3 substantive objections stand) — formal NEVER per HARD RULE 12 clause-c DISAGREE |
| **C-X2 Compromise family-root** | NEVER (categorical distinction is load-bearing for audit) — formal NEVER per HARD RULE 12 clause-c DISAGREE; C-A2 supersedes |
| **C-X3 `RotationPolicy` enum** | Post-v1-beta IFF AtriumWithRotatingGroupKey codepoint ships AND a substantive use-case requires multi-variant rotation-policy at wire | AtriumWithRotatingGroupKey codepoint ratification at Phase-5+ design pass |
| **C-X4 DUAL-CID Option I generalization** | Post-v1-beta IFF a second DedupScopeId variant has a real use-case | Use-case for non-Atrium-scoped dedup (e.g., user-scoped or device-scoped dedup) materializes in Phase-5+ |

**No "later" / "carry to next brief" / phantom-destination dispositions.** Every
deferral has a real destination + a real revisit-trigger per HARD RULE 12.

---

## §4 Task 4 — Over-fitting / accidental-complexity findings

Reverse question: did M-CONS-v2 bake-in any complexity that's actually solving
a non-problem?

### §4.1 OF1 — 6 EXPANDED amendments per-Kind sub-arm bloat?

**Investigation.** M-CONS-v2 F1–F28 + F-N2-A/B/C have multiple "per-Kind"
sub-arms (F4 sender+member-list in AAD; F5 replay-window per-Kind exclusion
table; F22 universal size-class buckets per-Kind; F26 D1–D7 per-Kind walkthrough;
F-N2-A Authority cardinality per-Kind; F-N2-B 24-op classification with
per-Kind dispatch).

**Finding.** **NOT over-fitting.** Each per-Kind sub-arm corresponds to a real
threat-model + UX dispatch axis (N2 §1.3 objections 1–4). Removing per-Kind
sub-arms would force runtime-dispatch of the same logic with weaker compile-time
guarantees.

**Verdict.** No over-fitting; the per-Kind specialization is load-bearing.

### §4.2 OF2 — `MembershipSetMetadata` placement (3 fields really needed?)

**Investigation.** `MembershipSetMetadata { name, description, created_by_human_label }`.

**Finding.** All 3 fields are real-UX needs (per N2 §3.3 + M-CONS-v2 §11.6 P16):
- `name`: load-bearing for UX ("Marketing Q1 Atrium" vs Did-CID-display).
- `description`: optional longer-form; matches Discord/Slack/Matrix precedent.
- `created_by_human_label`: load-bearing for audit + UX provenance ("Alice from
  her laptop on 2026-04-01" — important for "who created this Atrium?" tracing).

**Verdict.** Not over-fitting; matches P16 codification. **Placement IS the issue
(C-A3 recommend lift to top-level), but the FIELD-COUNT is not over-fit.**

### §4.3 OF3 — `RotationTrigger` 6 variants over-fit?

**Investigation.** `enum RotationTrigger { AdminKick | MemberDepart | AdminDepart |
DeviceRevoke | CompromiseResponse | PeriodicHygiene }`.

**Finding.** Each variant maps to a distinct audit-firm question:
- AdminKick → "show me the adversarial-kick events for incident response"
- MemberDepart → "show me the voluntary-leave audit trail for compliance"
- AdminDepart → "show me when admin departed; key-management transition"
- DeviceRevoke → "show me device revocations for DeviceMesh hygiene"
- CompromiseResponse → "show me suspected-compromise forks for incident response"
- PeriodicHygiene → "show me scheduled forks for HIPAA/PCI compliance trail"

All 6 are audit-distinguishable. Doc-only enum; no wire-format change.

**Verdict.** Not over-fitting. Each variant has clear audit semantic.

### §4.4 OF4 — Compromise #56 mint (M-CONS-v2 acknowledges defensible-to-skip)

**Investigation.** M-CONS-v2 §12.1 names this as a known uncertainty: "Compromise
#56 mint may be over-engineered if audit firm clusters it under #54."

**Finding.** M-C1 evaluation: with C-A2 (`disposition_class` field) ADOPTED, #56
remains useful because:
- #54 is "group-key-rotation cadence deferral" (defense-deferred class).
- #56 is "per-message ephemeral-key discipline deferral" (separate-design-class
  defense-deferred).
- The two are STRUCTURALLY DIFFERENT defenses, even if both are AcceptedTradeOff.

Auditor clarity benefits from separate mints.

**Verdict.** Not over-fitting given C-A2. **my-pred KEEP #56 MINT;** the
clustering risk M-CONS-v2 flagged is mitigated by C-A2's disposition_class
clarity.

### §4.5 OF5 — `ThresholdAdminSpec` v1-beta-CR overload?

**Investigation.** `ThresholdAdminSpec { threshold, n, eligible_admins: BTreeSet<Did> }`
is v1-beta-CR (codepoint-reserve only at v1-beta; impl Phase-4-Meta-Composing+).

**Finding.** N2 D7 deferred + pinned per-Kind variant (Atrium-only at v1-beta-CR).
Bounded struct; ~20 LOC.

**Verdict.** Not over-fitting; codepoint-reserve discipline is correctly applied.

### §4.6 OF6 — Compromise #55 P2P-by-design semantic — over-disclosure?

**Investigation.** #55 ships a full "P2P content-addressed system cannot
unilaterally erase content" disclosure + "applications layer strong-RTBF via
per-subject crypto-shredding" guidance.

**Finding.** Per Ben's framing (M-CONS-v2 §1.3), this IS the honest architectural
posture. NOT over-disclosure. The Compromise # registry serves audit-firm review
+ regulatory-context positioning; #55 framing matters.

**Verdict.** Not over-fitting; load-bearing for audit deliverable framing.

### §4.7 OF7 — 4 sibling-trait families (Lifecycle/Members/Crypto/Audit) at inherent-impl

**Investigation.** N2 §3.2 + M-CONS-v2 §9.1 (D8 sibling-trait extraction) ships
4 sibling-trait families ALONGSIDE inherent-impl at v1-beta.

**Finding.** M-CONS-v2 §9.3 lists this as an OPEN Ben-call (defensible either
way). M-C1 evaluation:
- ALONGSIDE (default): N2 §3.2 my-pred; small cost (~+30 LOC trait defs); free
  optionality for v1.x refactor to traits-only without breaking inherent users.
- TRAITS-ONLY: forces caller-side `use` discipline; more idiomatic Rust; ~–20 LOC.
- INHERENT-ONLY: M2's original; loses sibling-trait composition benefits.

**Verdict.** Slight over-fitting if ALONGSIDE is chosen (4 traits + inherent impl
= ~2× method surface in docs). But the cost is bounded (~+30 LOC) and the
v1.x-refactor optionality is genuine. **my-pred KEEP ALONGSIDE per N2 default;**
revisit at Phase-4-Meta-Composing when trait-composition use-cases materialize.

### §4.8 OF8 — `Authority::sig_pubkey` cache field — premature optimization?

**Investigation.** N2 §1.4 includes `Authority { did, sig_pubkey, role, admitted_at_hlc }`
where `sig_pubkey` is "cached here so verifiers don't need to re-resolve the DID
for every mutation."

**Finding.** This is a genuine perf optimization (DID resolution is non-trivial).
Justified for hot-path verification.

BUT: under **C-A1 (members_table fusion)**, the `MemberEntry::Admin` variant
carries `sig_pubkey: HybridSigPubkey`; same caching. C-A1 preserves this.

**Verdict.** Not over-fitting; caching is load-bearing. Compatible with C-A1.

### §4.9 Over-fitting findings summary

**Net over-fitting findings: 0 substantive.** **1 minor caveat** (OF7 sibling-trait
alongside-vs-traits-only is open).

M-CONS-v2 is NOT over-engineered. The per-Kind specialization + 6 RotationTrigger
variants + 3-field MembershipSetMetadata + ThresholdAdminSpec + Compromise #55
framing all carry their weight. **All 8 OF1–OF8 candidates evaluate as
load-bearing or compatible-with-an-elegant-shape-adopt.**

---

## §5 Task 5 — Confidence-justified honest assessment

### §5.1 Honest assessment

**Has M-CONS-v2 captured all elegant shapes?** **MOSTLY YES.** The pipeline
(M-panel → M-CONS-v1 → N1-N4 → M-CONS-v2 + Ben refinement-pass) executed exactly
the discipline M-CONS-v2 §11.7 Pattern P17 codifies as optimal.

The 3 RECOMMEND-ADOPT candidates this critique surfaces (C-A1 + C-A2 + C-A3) are
**small refinements, not redux-class collapses.** None close ≥3 amendments at
once; each closes 1 invariant clause or 1 class-of-bug or 1 mis-coupling.

This is consistent with **diminishing-returns past Ben's refinement-pass** — the
N2 generalization (uniform Authority slot) was the LAST major redux available;
the remaining elegant shapes are at the +10–20% improvement level.

### §5.2 Confidence on "no further redux available"

**HIGH.** The investigation pattern:
- 8 candidates investigated.
- 3 RECOMMEND-ADOPT (small refinements).
- 1 RECOMMEND-NAMED-DEFER (doc shape).
- 4 DISAGREE-WITH-EXPLANATION (cost > benefit OR substrate-incorrect).
- 8 over-fitting OF1–OF8 candidates checked: 0 substantive over-engineering.

This SHAPE of evidence ("many small refinements + many rejected candidates +
no major redux") is what we'd expect at the END of an elegant-shape pass over a
well-consolidated design.

### §5.3 Self-check: am I missing something?

M-C1 deliberately considered candidates beyond the brief's named list:

- **Could Inv-20 + Inv-19 fuse?** No — M-CONS-v2 §5.2 already notes "Inv-20
  clause-h subsumes Inv-19 clause-b"; the rest of Inv-19 covers Path-A.5 + MembershipSet
  composition at API boundary, distinct from Inv-20's primitive-internal
  invariants. Keep separate.
- **Could `MembershipSetEncryption` codepoint family + Sealed-Sender slot fuse?**
  No — Inv-18b explicitly maintains them as paired-slot to enable Sealed-Sender
  DEFAULT (Am4 ratified). Separate.
- **Could `parent_membership_set_id` + `generation` collapse?** No — parent_id
  is fork-lineage; generation is CRDT-vector within an unforked lifecycle.
  Different axes.
- **Could `MembershipAttestation` fuse with `Authority`?** No — Attestation is
  the membership PROOF; Authority is the signing PRINCIPAL. Distinct concepts.
- **Could `TransportConfig` move out of `MembershipSet` to a sibling artifact?**
  Considered. M6 + Ben Q1 SCOPE-IN consciously placed it ON `MembershipSet`
  because transport-choice is per-MembershipSet (Atrium might use gossip; DeviceMesh
  might use blobs). Sibling-artifact would force a join-key for transport-config-
  lookup. Keep on MembershipSet.

**No additional candidates found.** Confidence remains HIGH on "no missed
structural redux."

### §5.4 What this critique does NOT cover

- **Composability** (M-C2 territory): how MembershipSet composes with Drop +
  Caps + Sync + Engine across crate boundaries.
- **Fresh-eyes** (M-C3 territory): whether a reader cold-starting on this work
  finds load-bearing gaps the M-CONS-v2 + N-panel insiders missed.
- **R0 substantive content** (M-R0 territory).
- **Implementation-time discoveries** that re-open shape questions.

### §5.5 If Ben wants only ONE adopt

**C-A1 (`members_table` fusion).** It's the highest-value: closes a real
class-of-bug (cross-table drift) + saves real LOC + lands cleanly at canary
scope.

If Ben wants TWO, ADD C-A2 (`disposition_class` field).
If Ben wants THREE, ADD C-A3 (metadata lift).
If Ben wants ZERO additions to M-CONS-v2, that's also defensible — M-CONS-v2 is
DECISION-READY as-is.

---

## §6 R0 implications

Per the dispatch brief's ADDL pipeline + M-CONS-v2 §10:

### §6.1 If C-A1 + C-A2 + C-A3 ADOPTED

**R0 plan-doc skeleton updates** (incremental to M-CONS-v2 §10):

- **§2.1 The primitive:** update N2 §4.1 canonical Rust shape to:
  - `policy.members_table: BTreeMap<Did, MemberEntry>` (replaces both
    `authorities` + `role_assignments`).
  - `MembershipSet.metadata: MembershipSetMetadata` (lifted top-level from
    `policy.metadata`).
- **§2.4 Wire-format codepoint family:** CBOR shape adjustment — `pol.members`
  one-map (was `pol.auth` + `pol.roles` two-maps); top-level `meta` (was `pol.meta`).
- **§2.5 (NEW from N2) Authority + RoleAssignments + RBAC:** rename §-heading to
  "Members table + RBAC" (one table, not two); update Authority struct doc to
  describe `MemberEntry::Admin` variant carrying sig_pubkey; `MemberEntry::NonAdmin`
  carrying just role.
- **§2.6 (NEW from N2) 3-role RBAC composition with UCAN:** RBAC + UCAN
  intersection-of-allows logic unchanged; only the underlying data-shape moves
  from two-table to one-table.
- **§11 docs/THREAT-MODEL.md skeleton:** add `disposition_class` field at the
  top-of-document explainer + apply to every #-row.
- **§13 docs/CRYPTO-CODEPOINTS.md skeleton:** update Authority CBOR encoding row
  to reflect members_table shape.
- **§15 Wave-sequencing:** Wave-MS-PRIMITIVE canary scope unchanged (still ~10.5–13
  wave-days); the 3 ADOPTs are bounded shape-refinements absorbed inside canary.

### §6.2 F-row updates

- **F-N2-A** updated description: "Members table (single `BTreeMap<Did, MemberEntry>`
  replacing M-CONS-v2's `authorities` + `role_assignments` per M-C1 C-A1 fusion)
  + 3-role RBAC + linear hierarchy + UCAN-composed intersection-of-allows +
  MembershipEvent enum hosted here."
- **F-N2-C** updated description: "MembershipSetMetadata at TOP-LEVEL on
  MembershipSet (lifted from inside MembershipSetPolicy per M-C1 C-A3 placement
  refinement) + admin-gated `rename`/`update_metadata` op."
- **F25** updated description: "...includes `disposition_class` field on every
  Compromise # registry entry per M-C1 C-A2 + M-CONS-v2 §11.4 Pattern P14."

### §6.3 Inv-20 clause updates

- **Clause-i** rephrased: "Uniform `members_table: BTreeMap<Did, MemberEntry>`
  fuses signing-authority + RBAC role into a single per-DID entry; Admin-cardinality-
  per-Kind validated by constructors; cross-table drift class-of-bug eliminated
  by construction." (per C-A1)
- **Clause-j** unchanged.

### §6.4 If only C-A2 ADOPTED (Ben prefers minimal change)

- F25 + SECURITY-POSTURE.md add `disposition_class` field per-row.
- Everything else unchanged from M-CONS-v2.
- Net wave-day delta: ~+0.1 wave-day (bounded).

### §6.5 If ZERO ADOPTED

M-CONS-v2 ships as-is. M-C1's outputs are NAMED-deferred to Phase-5+ revisit
with the destinations + triggers per §3. Net wave-day delta: 0.

---

## §7 Open Ben-call surfaces (from M-C1)

| # | Decision | my-pred | Confidence |
|---|---|---|---|
| **M-C1-D1** | Adopt C-A1 `members_table` fusion (replace `authorities` + `role_assignments` with single `BTreeMap<Did, MemberEntry>` enum-variant for Admin-vs-NonAdmin) | **ADOPT** | HIGH |
| **M-C1-D2** | Adopt C-A2 `disposition_class` field on Compromise # registry (`AcceptedTradeOff` vs `SubstrateGuaranteeDisclosure`) | **ADOPT** | HIGH |
| **M-C1-D3** | Adopt C-A3 lift `MembershipSetMetadata` to top-level on `MembershipSet` (separate `metadata_version: u32` independent of policy_version) | **ADOPT** | MED-HIGH |

All 3 surfaceable alongside M-CONS-v2 §9.3's 4 open Ben-calls in ONE Ben-pass
(7 total surfaces). None are arch-fork-class per `feedback_surface_arch_decisions_under_auth.md`.

---

## §8 Self-assessment + confidence per finding

| Finding | Confidence | Reasoning |
|---|---|---|
| **§0 Headline verdict** ("M-CONS-v2 captures substantive elegant shapes already; 3 small ADOPTs + 1 NAMED-DEFER + 4 DISAGREE") | **HIGH** | 8 candidates investigated; pattern of evidence consistent with diminishing-returns past Ben's refinement-pass |
| **§1 Cluster table** (16 clusters covering 31 F-rows; 13/16 already collapsed) | **HIGH** | Mechanical clustering by structural property; M-CONS-v2's pack-rows (F24/F25) are pre-collapsed by design |
| **§2.1 C-A1 fusion** (`members_table`) | **HIGH** on direction; **MED-HIGH** on exact LOC delta | Type-system enforces Admin-has-sig_pubkey via enum; constructor cardinality validation preserved; wire-shape strictly smaller |
| **§2.2 C-A2 disposition_class field** | **HIGH** | Codifies M-CONS-v2 own P14 pattern at data-model level; bounded cost; matches PIM disciplines |
| **§2.3 C-A3 metadata lift** | **MED-HIGH** | Cleaner separation of concerns; D1 walkthrough simpler; metadata_version independent of policy_version |
| **§2.4 C-D1 Inv-20 6-cluster doc-shape NAMED-DEFER** | **MED-HIGH** | Doc-pass benefit; not load-bearing for canary; bounded cost |
| **§2.5 C-X1 associated-type-trait REJECTED** | **HIGH** | Wire-format runtime-tag requirement + downstream phantom-type ripple defeats benefit; N2 §1.3 substantive objections stand |
| **§2.6 C-X2 family-root REJECTED** | **HIGH** | Categorical distinction per P14 is load-bearing for audit; C-A2 replaces |
| **§2.7 C-X3 RotationPolicy enum REJECTED** | **HIGH** | CLAUDE.md #15 (v1-beta freeze; don't commit to a shape we don't ship); codepoint-reserve P18 is the right shape |
| **§2.8 C-X4 Option I generalization REJECTED** | **MED-HIGH** | Codepoint-reserve P18 covers additive evolution cheaply; early generalization buys no current use-case |
| **§4 Over-fitting OF1–OF8** (0 substantive over-engineering; 1 minor caveat OF7) | **MED-HIGH** | Each per-Kind sub-arm corresponds to real threat-model + UX dispatch axis; field-counts justified |
| **§5 Honest assessment** ("MOSTLY YES — M-CONS-v2 captures elegant shapes already") | **HIGH** | Investigation pattern consistent with diminishing-returns; 3 small ADOPTs are +10-20% improvement level |
| **§6 R0 implications under each Ben-call combination** | **HIGH** on shape; **MED-HIGH** on exact wave-day deltas | Skeleton update is structural; wave-day arithmetic compounds small estimates |

### §8.1 What I could be wrong about

- **C-A1 wire-format impact during NAPI binding.** The TS mirror change from
  `Authority[]` + `RoleAssignments` to `Map<Did, MemberEntry>` may cost slightly
  more than estimated if TS callers had ergonomic expectations around the two-
  table shape. Bounded.
- **C-A3 might be wrong on D1 walkthrough subtlety.** Policy_version_at_seal in
  AAD is unaffected because metadata isn't crypto-policy. But if some hypothetical
  future verifier wants to bind metadata into AAD (e.g., "this seal was made when
  the Atrium had name X"), C-A3 would need adjustment. **my-pred: not load-bearing
  at v1-beta;** metadata-binding-in-AAD is post-v1-beta if anywhere.
- **C-A2 disposition_class may not be the only useful enrichment.** Future audit
  framings may add `severity_class` or `mitigation_class` fields. Bounded:
  the field is additive; future enrichments compose.
- **C-D1 cluster grouping** might benefit auditors less than I estimate; doc-pass
  effort might be wasted. Bounded; NAMED-DEFER preserves optionality.
- **OF7 (sibling-trait alongside-vs-traits-only)** is genuinely defensible either
  way; I leaned ALONGSIDE for free optionality, but TRAITS-ONLY (more idiomatic
  Rust) is also reasonable. Surface to Ben.

### §8.2 What this critique does NOT cover

- **M-C2 composability concerns** (cross-crate boundary; trait-composition;
  capability chaining).
- **M-C3 fresh-eyes concerns** (cold-start reader gaps; framing clarity;
  unexpected use-case surfaces).
- **R0 substantive content** authoring.
- **Wave-MS-PRIMITIVE execution** + canary-merge sequencing details.

### §8.3 Convergence verdict (process-level)

The M-CONS-v2 pipeline executed exactly the discipline M-CONS-v2 §11.7 P17
codifies. The Ben-refinement-pass (N1–N4) caught the load-bearing generalization
(N2 Authority slot) the convergent M-panel missed; the critique-round (M-C1
here) is finding +10-20% improvements rather than redux-class collapses. This
is the SHAPE of evidence we'd expect at the end of a well-executed elegant-shape
pass.

**M-C1 confirms M-CONS-v2 is DECISION-READY** with 3 small refinements (or 0 if
Ben prefers minimal change) for Ben's bundle.

---

## §9 Citations

### §9.1 Primary input

- **M-CONS-v2 consolidator:** `phase-4-meta-core/membership-set-m-cons-v2-consolidator @ e62ff540`
  → `.addl/phase-4-meta/membership-set-m-cons-v2-consolidator.md` (1295 LOC; read in full)

### §9.2 Background — pipeline inputs

- M-CONS-v1 consolidator: `74580ee6` → `.addl/phase-4-meta/membership-set-m-cons-consolidator.md` (1081 LOC; read in full)
- N1 sub-graph-elegance: `ed592770` → `.addl/phase-4-meta/membership-set-n1-subgraph-sharing-elegance.md` (1067 LOC; read in full)
- N2 generic-primitive + RBAC + ops: `a1b5a552` → `.addl/phase-4-meta/membership-set-n2-generic-primitive-ops-roles.md` (1399 LOC; read in full)
- N3 continuous-rotation edge cases: `298c80d9` → `.addl/phase-4-meta/membership-set-n3-continuous-rotation-edge-cases.md` (506 LOC; read in full)
- N4 Signal-Sender-Keys comparison: `2ee24e9f` → `.addl/phase-4-meta/membership-set-n4-signal-sender-keys-comparison.md` (494 LOC; read in full)

### §9.3 Tree-pinned (HEAD `2172cb6d`)

- `crates/benten-caps/src/scope.rs` — `Scope::{Hashes | RestrictedSelector}` 2-arm enum (N1 F28 modification site; line 46 `pub enum Scope`)
- `crates/benten-caps/src/restricted_spec.rs` — `RestrictedScope` 6-dim (N1 F28 lifted to `RestrictedScopeSet`; line 103 `pub struct RestrictedScope`; line 132 `impl RestrictedScope`)
- `crates/benten-core/src/subgraph_spec/combinators.rs` — `intersect/union/filter` (N1's structural lever; lines 37/99/140 `pub fn`)
- `crates/benten-caps/src/chain_authority.rs` — UCAN chain-walker seam (RBAC × UCAN composition site per C-A1 + N2 §2.6)

### §9.4 CLAUDE.md baked-in + MEMORY.md disciplines applied

- CLAUDE.md baked-in #1 (12-primitive irreducibility — C-A1 honors via in-place
  refinement, not new primitive)
- CLAUDE.md baked-in #5 (crypto-agility — codepoint-dispatch backward-compat for
  C-X3 + C-X4 rejection reasoning)
- CLAUDE.md baked-in #15 (v1-beta interface freeze — C-X3 + C-X4 rejection
  protects against shape-pre-commit; C-A1 + C-A3 land BEFORE freeze)
- CLAUDE.md baked-in #17 (multi-process + device-shape — preserved unchanged)
- CLAUDE.md baked-in #18 (4-identity-concepts — MemberKey homogeneous-per-Kind
  preserved under C-A1)
- CLAUDE.md baked-in #19 (engine-level extensions — RBAC engine-level per N2
  preserved)
- `feedback_extra_reflection_pass_for_elegant_permanent_shape.md` — this critique's
  governing discipline
- `feedback_engine_primitives_vs_application_layer.md` — C-X1 rejection cites
- `feedback_no_defer_HARD_RULE.md` — every deferral in §3 has NAMED destination
  + revisit-trigger
- `feedback_iterate_critical_reviews_to_convergence.md` — M-C1 is one round in
  the convergence loop
- `feedback_handoff_top_banner_re_orient.md` — honored at file top
- `feedback_plain_english_surfaces.md` — every ADOPT/DEFER/DISAGREE has
  plain-situation + concrete-options + my-prediction + confirm-or-redirect
- `feedback_surface_arch_decisions_under_auth.md` — 3 M-C1 open Ben-calls
  confirmed non-arch-fork-class
- `feedback_review_finding_ground_truth_verify.md` — every M-C1 finding
  ref-pinned to M-CONS-v2 + N1–N4 + code at SHA `2172cb6d`

---

**End of M-C1 elegant-shape critique-round document.** Output for Ben's bundle
with M-C2 (composability) + M-C3 (fresh-eyes) parallel outputs; consolidator
(if any) per ADDL pipeline.
