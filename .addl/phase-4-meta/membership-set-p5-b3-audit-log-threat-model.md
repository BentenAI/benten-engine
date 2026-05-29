# MembershipSet P5 B-3 — `audit_log_query` × Unlinkability Threat Model

**Reviewer lens.** Senior threat-modeling + privacy architect, simulating a Trail-of-Bits / NCC-Group adversary-modeling sweep at v1-beta-tag time. Distinct from the broader L5 audit-readiness lens (which assessed crypto-substrate audit-deliverable shape) — this lens evaluates **what an Admin-role insider, a compromised-Admin-device, a coerced-Admin, or an external attacker holding query-output can learn from `audit_log_query` that Inv-20 clause-d (per-recipient unlinkability) implicitly promises against, where the promise actually breaks, and what mitigations + access-control gradations belong in v1-beta vs codepoint-reserve vs post-v1-beta.**

**Blind spot under analysis.** M-C2-v2 §5 B-3 (`0533d0cf`): F-N2-B `audit_log_query` × Inv-20 clause-d per-recipient unlinkability × N3 #55 GDPR-RTBF P2P-by-design disclosure. M-C2 R-MCV2-(B-3) proposed: (a) scope-down `audit_log_query` to Admin-role per F-N2-B RBAC gate, (b) clarify Inv-20 clause-d as **network-observer-only**, (c) mint Compromise #58 (`Composition-Hazard-Honest-Disclosure` class).

**Inputs cross-referenced.**
- M-CONS-v2 consolidator `e62ff540` — 1295 LOC; 31 F-amendments; 10-clause Inv-20; 26 Compromise mints.
- M-C2-v2 critique `0533d0cf` — §5 B-3 finding; R-MCV2-(B-3) refinement; Compromise #58 candidate.
- M-C3-v2 fresh-eyes `cac10631` — independent Compromise #58 candidate (KEM-key-confirmation; **naming collision flagged** in §6 below).
- N2 generic-primitive-ops-roles `a1b5a552` — op #23 `audit_log_query` spec (~60 LOC; v1-beta-LB; `MembershipSetAudit` trait family; `AuditQueryFilter` parameter).
- N4 Signal Sender Keys comparison `2ee24e9f` — SSK audit-comparison baseline.
- L5 threat-model + audit-readiness `3f27f8e0` — Cure53/ToB-style audit posture + T-06/T-19 insider-with-DAK rows + L5-C3 honest-disclosure precedent.

---

## §1. Verdict

**Top-line.** M-C2-v2 §5 B-3 is **a real over-disclosure surface** and the proposed R-MCV2-(B-3) closure (admin-gate + network-observer-only clarification + Compromise #58 mint) is **NECESSARY-BUT-NOT-SUFFICIENT**. The admin-role gate closes the **member-vs-member** correlation attack but does **not** close the **malicious-admin / compromised-admin-device / coerced-admin / cross-fork-correlation** attacks; those require additional v1-beta-LOAD-BEARING mechanism.

**Confidence.** **HIGH** that the insider-correlation surface is real and not currently surfaced in M-CONS-v2 (verified by reading N2 §3.1 op #23 spec: no role-gate beyond "current member"; verified by reading Inv-20 clause-d framing: "per-recipient" without observer-class scoping). **HIGH** that admin-role gate is necessary minimum. **MEDIUM-HIGH** that the full v1-beta-LB closure shape proposed below is the right shape; the gradation menu (§4) is a choice-point with policy implications that may need Ben's ratification.

**Severity rating (composite across the threat catalog of §2).** **HIGH** as currently specified in M-CONS-v2 (no role-gate on `audit_log_query`); **MEDIUM-LOW** after R-MCV2-(B-3) + the additional 4 v1-beta-LB closures proposed in §7.

**Single most load-bearing recommendation.** **Mint `AuditAccessGradation` enum as `MembershipSetPolicy.audit_access` config field at v1-beta** with default = `AdminOnly` and codepoint-reserved variants `{AdminOnly | MemberOnly_OwnEvents | ThresholdAdmin_M_of_N | TimeLocked | Anonymized_Aggregate}`. This makes the Atrium-creator's audit-posture an explicit ATTPolicy decision rather than a baked-in default; closes the malicious-admin attack via threshold-admin opt-in; surfaces honestly per Compromise #58.

**Compromise #58 collision flagged.** M-C3 also proposed `#58` for KEM-key-confirmation. Recommend: **M-C2's audit-log-insider-correlation = #58** (this brief), **M-C3's KEM-key-confirmation = #59 or #60** (renumber at M-CONS-v2.1 re-consolidation). Rationale: §6 below + B-3 is the structurally-load-bearing one (touches ATTPolicy config); M-C3's #58 candidate was LOW-priority "mint-or-skip" per the M-C3 reviewer's own disposition.

---

## §2. Threat enumeration

For each threat: adversary-class (T-NN), severity (HIGH/MED/LOW), likelihood (HIGH/MED/LOW), mitigation, residual.

### T-B3-01 — Malicious Admin enumerates membership-history to identify dissidents / whistleblowers

**Adversary.** Admin role within a single Atrium (e.g., workplace Atrium where admin = manager; community Atrium where admin = forum-owner).

**Capability.** Calls `audit_log_query(filter: AuditQueryFilter::All)` → receives full `Vec<MembershipEvent>`. Events include `AdminKick { target, when, reason }`, `MemberDepart { who, when }`, `RoleChange { target, from, to, when, by_whom }`, `DeviceRevoke { who, device, when }`, `CompromiseResponseRotation { triggered_by, when }`, `PeriodicHygiene { when }`.

**Attack.** Admin correlates (timestamp HLC, member-DID, op-type) to identify:
- **Voting patterns** if `RoleChange` events embed vote-source (e.g., "Bob's promotion was proposed by Alice, seconded by Carol, rejected by Dave" — would reveal Dave's dissent).
- **Whistleblower identification** if `MemberDepart` correlates with external retaliation timeline (e.g., "Eve left 3 days before the leaked report dropped; her DAK was rotated 1 hour before departure").
- **Social-graph reconstruction** by mapping admin-add timestamps to invitation chains.
- **Compromise-response correlation** revealing which member experienced device-compromise (and by inference, what real-world event triggered it).

**Severity.** **HIGH**. This is the canonical malicious-admin attack against centralized chat systems (Slack admins, Discord owners, Matrix homeserver admins all have analogous capability today; users tolerate it because they trust the admin OR the system is overtly centralized). Benten's positioning as a **P2P privacy-first system** means user expectations are higher — discovering that "any admin can correlate my full history" is an **expectation gap**.

**Likelihood.** **MEDIUM**. Most Atriums won't have malicious admins; but the v1-beta target deployments include personal-AI-assistants where the user IS the admin (low risk) AND community/workplace Atriums where admin ≠ user (high risk for non-admin members). Realistically: 5–20% of v1-beta deployments will have at-least-one event where this capability matters.

**Mitigation.**
1. **Admin-role gate** (R-MCV2-(B-3) baseline): closes the **member-vs-member** version of this attack but NOT this scenario (the attacker IS the admin).
2. **AuditAccessGradation policy field** (this brief's primary recommendation): Atrium-creators choose `ThresholdAdmin_M_of_N` for high-stakes Atriums (e.g., journalism, activism), forcing M-of-N admin co-signing for `audit_log_query` invocation. Closes T-B3-01 for high-stakes use-cases.
3. **AuditQueryFilter constraints** (§3 below): even Admin role only gets aggregated/anonymized queries by default; identified per-member queries require `AdminWithJustification` filter variant that itself emits a `MembershipEvent::AuditAccessExercised { admin, filter_shape, when }` event — making admin-surveillance **itself auditable**.
4. **Compromise #58 honest-disclosure**: Atrium-creators are warned at creation-time UX that "Admin role has full membership-history visibility unless threshold-admin or time-locked audit-access-gradation is selected." Lets users self-select.

**Residual.** Single-admin Atriums with `AdminOnly` (the simplest config) retain T-B3-01 risk. Honest-disclosure + threshold-admin opt-in is the boundary. **This is a Composition-Hazard-Honest-Disclosure-class trade-off** consistent with M-CONS-v2's #55 GDPR-RTBF P2P-by-design framing.

### T-B3-02 — Compromised Admin device leaks `audit_log_query` results to external adversary

**Adversary.** Admin's device is fully compromised (malware exfil, post-PCS-window pre-rotation, evil-maid, etc.).

**Capability.** Same as T-B3-01 (admin's session credentials), but the admin themselves is unaware.

**Attack.** External adversary obtains full membership-history via the compromised admin's audit-log access. Particularly damaging because:
- The admin retains plausible deniability ("I never queried that").
- The audit-log of `audit_log_query` calls itself (per T-B3-01 mitigation #3) gets exfiltrated alongside, BUT the attacker can avoid invoking it by **reading on-disk state directly** (skipping the API surface entirely).

**Severity.** **MEDIUM-HIGH**. Lower than T-B3-01 because requires device-compromise (raises bar); higher than T-B3-04 because the admin is the one trusted MOST in the system.

**Likelihood.** **LOW-MEDIUM**. Aligned with general device-compromise rates (~1–5% over multi-year deployment per admin); but Atriums often have 5–20 admins, raising the per-Atrium probability to ~5–60%.

**Mitigation.**
1. **Audit-log encryption-at-rest** (§5.2 below): on-disk audit-log entries encrypted under a key derived from admin's DAK + Atrium-id. Compromised filesystem ≠ compromised audit-log unless DAK is also compromised. **Reuses Inv-20 clause-f vault-encryption pattern.**
2. **Compromise #58 honest-disclosure** notes T-B3-02 explicitly: "Admin device compromise = audit-log compromise; admin device hygiene is load-bearing."
3. **Audit-log retention-window** (§5.1): per-event TTL (e.g., 90 days for routine RoleChange / PeriodicHygiene; permanent for AdminKick / CompromiseResponseRotation per audit-trail discipline). Reduces blast-radius of any single device-compromise to recent-history-only.
4. **AuditAccessExercised event** (T-B3-01 mitigation #3): if other admins are watching, anomalous audit-query patterns become detectable. Doesn't help if there's only one admin.

**Residual.** Audit-log-at-rest decryption requires the admin's DAK + filesystem access; attacker with both has already won (per L5 T-06 / T-19 "you-have-the-vault-AND-the-password" out-of-scope row). Documented in Compromise #58.

### T-B3-03 — Coerced Admin (legal subpoena, nation-state demand) reveals audit-log

**Adversary.** Nation-state or court-of-law compelling admin to disclose audit-log contents (subpoena, NSL, FVEY equivalent).

**Capability.** Legal compulsion forces admin to invoke `audit_log_query(filter: All)` and turn over results. Admin cannot lie (perjury risk) or refuse (contempt).

**Attack.** State-level adversary obtains structured membership-history under color of law.

**Severity.** **HIGH** for activists / journalists / dissidents who chose Benten precisely for state-resistance properties.

**Likelihood.** **LOW for general v1-beta deployments; MEDIUM-HIGH for journalism/activism deployments** (the same v1-beta target users L5-C3 already flagged for honest-disclosure).

**Mitigation.**
1. **AuditAccessGradation = ThresholdAdmin_M_of_N**: legal compulsion against ONE admin doesn't unlock the log; requires compelling M admins. Significantly raises legal-coercion cost.
2. **AuditAccessGradation = TimeLocked**: admin can invoke query but results are time-delayed (e.g., 7-day delay before plaintext disclosure); during delay, an out-of-band warrant-canary protocol can fire. Aligned with Signal's [warrant-canary precedent](https://signal.org/bigbrother/) and Apple's iCloud-judicial-process patterns.
3. **AuditAccessGradation = Anonymized_Aggregate**: admin can produce only counts ("12 RoleChange events in the last 30 days") without per-member-disclosure. Forces legal process to demand specific named records, raising specificity-of-warrant requirement (4th-amendment-friendly in US contexts; equivalent elsewhere).
4. **GDPR-RTBF P2P-by-design Compromise #55 cross-link**: Compromise #58 notes that audit-log retention is policy-driven; an Atrium can choose 0-retention (no audit-log) and accept the operational cost (no compliance trail). Maximally state-resistant; not the default.

**Residual.** Legal compulsion against ALL admins of a threshold-admin Atrium can still force disclosure if M < |admins|. Documented in Compromise #58. State-resistance is a **gradient**, not a binary.

### T-B3-04 — Audit-vs-privacy tension: legitimate audit needs vs minimal disclosure

**Adversary.** N/A — this is a design-tension, not an adversary scenario.

**Capability.** Legitimate audit use-cases (community-moderation review, debugging, post-mortem after compromise, regulatory compliance for enterprise Atriums) require some level of audit-log visibility.

**Tension.** Maximum-privacy = no audit-log = no accountability. Maximum-accountability = full audit-log access = T-B3-01–03. The sweet spot depends on use-case.

**Severity.** **MEDIUM** as design-risk; if mis-resolved, either kills privacy (over-disclosure) or kills operability (under-accountability).

**Likelihood.** **CERTAINTY** that this tension surfaces at every Atrium-creation UX moment.

**Mitigation.**
1. **AuditAccessGradation as explicit policy choice**: 6 gradations (§4 below) cover the spectrum from no-audit through fully-public-audit. Atrium-creators choose at creation-time; can be changed by Admin role + ratification ceremony.
2. **Sensible defaults per Kind**: SingleDevice = `Public_All_Members` (only-self exists); DeviceMesh = `AdminOnly` (single-user; the admin IS the user); Atrium = `AdminOnly` (DEFAULT) + UX-prompt to upgrade to `ThresholdAdmin_M_of_N` for shared/community Atriums.
3. **Documentation**: `docs/AUDIT-POLICY-GUIDANCE.md` explains the trade-offs per gradation (codepoint-reserve at v1-beta; full doc at v1-GM).

**Residual.** Atrium-creators who don't understand audit-trade-offs may select inappropriate gradations. UX surface design (post-v1-beta scope) carries the load.

### T-B3-05 — Insider-correlation across forks (cross-Atrium membership-mobility tracking)

**Adversary.** Admin of Atrium A + Admin of Atrium B (or single Admin holding both roles, e.g., parent-Atrium admin shared across forks).

**Capability.** Each admin individually invokes `audit_log_query` against their respective Atrium. Cross-correlates outputs to track member-mobility patterns (who joined A and B; who left A but stayed in B; timing patterns).

**Attack.** Constructs a social-graph of cross-Atrium membership across the user's Benten-presence. Particularly damaging because M-CONS-v2 §0 specifies that **Atriums can fork**: parent-Atrium admins can be auto-pre-existing in child-fork admins.

**Severity.** **MEDIUM-HIGH**. Worse than T-B3-01 because crosses the Atrium-isolation boundary that Inv-20 clause-d implicitly promises (per-recipient unlinkability across forks).

**Likelihood.** **MEDIUM**. Cross-fork admin overlap is the common case (forks inherit parent admins by default per M-CONS-v2 §0); attacker just needs to be a parent-fork admin.

**Mitigation.**
1. **Per-fork audit-log isolation**: each fork's audit-log is encrypted under a fork-specific key; parent-fork admin role does NOT automatically grant child-fork audit-log access. Requires explicit role-acceptance in the child fork. (This is the F-N2-A per-Kind cardinality + RBAC isolation pattern, applied to audit-log.)
2. **Compromise #58 cross-fork clause**: documents that "parent-fork admin → child-fork audit-log linkage is NOT automatic; admins must explicitly opt-into child-fork admin role to gain child-fork audit-log access." Closes the implicit cross-fork leak.
3. **DID-rotation guidance**: members concerned about cross-fork correlation should use distinct DIDs per fork. This is operational hygiene, documented per §5.5.

**Residual.** A member who reuses the same DID across forks is correlation-fingerprintable by anyone with audit-log access to multiple forks. Documented in Compromise #58; cross-links to MCV2-C-3 iroh-gossip topic-id leak.

### T-B3-06 — Iroh-gossip topic-id × audit-log composition (MCV2-C-3 × B-3)

**Adversary.** Network observer (iroh-gossip transport layer) + audit-log-output holder.

**Capability.** Per MCV2-C-3 (`0533d0cf`), iroh-gossip topic-id leaks `membership_set_id` at transport layer. Combined with audit-log output: external observer knows "this gossip-topic is membership-set X" + audit-log says "member D was kicked from X at HLC=t" → external observer can correlate D's network-level departure-from-gossip with audit-log timestamp.

**Severity.** **MEDIUM**. The transport-layer leak (MCV2-C-3) is the load-bearing surface; audit-log composes a second-order amplification.

**Likelihood.** **MEDIUM** if both MCV2-C-3 and B-3 surfaces are exposed simultaneously.

**Mitigation.**
1. **Closing MCV2-C-3 closes B-3 amplification too**: R-MCV2-(C-3) proposed iroh-gossip topic-id hashing/blinding. Once closed, audit-log timestamps no longer correlate to observable network-events.
2. **Compromise #58 cross-link**: explicitly cross-references Compromise candidate for MCV2-C-3 (likely #57 or #59).

**Residual.** Closed by MCV2-C-3 closure; B-3 in isolation does not add residual.

### T-B3-07 — Compromise #48 K_Set-shape-leak × audit-log composition

**Adversary.** Holder of K_Set-shape information (per Compromise #48; K_Set size leakage to non-members under certain conditions).

**Capability.** Compromise #48 leaks K_Set cardinality; combined with audit-log timestamp data, can refine cardinality-over-time graph.

**Severity.** **LOW-MEDIUM**. Compromise #48 already documented; audit-log composition is a third-order amplification.

**Likelihood.** **LOW**. Requires both Compromise #48 disclosure path AND audit-log access; orthogonal mitigations.

**Mitigation.**
1. **Compromise #58 cross-references Compromise #48** as a known composition-hazard. No new mitigation needed beyond existing #48 closure path.

**Residual.** Documented composition-hazard; closes with #48's existing closure trajectory.

### T-B3-08 — Member-role enumerates own history (Member_OwnEvents gradation)

**Adversary.** Member (non-admin) querying own MembershipEvent history.

**Capability.** Under `Member_OwnEvents` gradation, a member can query `audit_log_query(filter: AuditQueryFilter::OwnEvents)` to see their own RoleChange / DeviceRevoke / etc.

**Severity.** **LOW**. By construction, the events ABOUT a member are already observed-by-that-member at the time they happen (you know when you were promoted; you know when you rotated your device). Querying-own-history just gives them a structured view.

**Likelihood.** **N/A** (this is a legitimate use-case, not an attack).

**Mitigation.** None needed. **This is the GOOD case** — `Member_OwnEvents` is the privacy-respecting baseline that any audit-system should support per GDPR Article 15 "right of access" alignment.

**Residual.** None.

### Threat summary table

| # | Threat | Severity | Likelihood | Closure |
| --- | --- | --- | --- | --- |
| T-B3-01 | Malicious Admin enumerates dissidents | HIGH | MED | ThresholdAdmin gradation + AuditAccessExercised event + Compromise #58 |
| T-B3-02 | Compromised admin device exfiltrates log | MED-HIGH | LOW-MED | Audit-log encryption-at-rest + retention TTL + Compromise #58 |
| T-B3-03 | Coerced admin (subpoena/nation-state) | HIGH (target users) | LOW-MED | ThresholdAdmin + TimeLocked + Anonymized_Aggregate gradations + Compromise #58 |
| T-B3-04 | Audit-vs-privacy design tension | MED (design) | CERTAIN | AuditAccessGradation policy choice + sensible defaults |
| T-B3-05 | Cross-fork insider-correlation | MED-HIGH | MED | Per-fork audit-log isolation + DID-rotation guidance + Compromise #58 |
| T-B3-06 | Iroh-gossip topic-id × audit-log | MED | MED | Closed by MCV2-C-3 closure |
| T-B3-07 | Compromise #48 K_Set-shape × audit-log | LOW-MED | LOW | Closes with #48 trajectory |
| T-B3-08 | Member queries own history | LOW (legit) | N/A | None needed (good case) |

---

## §3. `AuditQueryFilter` query-shape design

N2 §3.1 op #23 specifies `audit_log_query(filter: AuditQueryFilter) -> Result<Vec<MembershipEvent>, MembershipError>` but does NOT specify the `AuditQueryFilter` enum body. This brief proposes:

```rust
/// Audit-log query filter shapes. The `AuditAccessGradation` policy field
/// (per §4) gates which filter variants a given role can invoke; the per-event
/// `MembershipEvent::AuditAccessExercised` audit-entry records every filter
/// invocation for transparency.
pub enum AuditQueryFilter {
    /// Full enumeration of all events. ADMIN ONLY (per AuditAccessGradation).
    /// Auto-emits AuditAccessExercised{filter: All, requester, at_hlc}.
    All,

    /// Member's own history only. Available to MEMBER role under
    /// AuditAccessGradation::Member_OwnEvents (legitimate per GDPR Art. 15).
    OwnEvents { since: Option<HlcTimestamp> },

    /// Filter by member-DID. ADMIN ONLY + emits AuditAccessExercised.
    /// Used for moderation review ("show me everything Bob did").
    ByMember { target: MemberDid, since: Option<HlcTimestamp> },

    /// Filter by event-type. ADMIN ONLY + emits AuditAccessExercised.
    /// Used for compromise-response review ("show all DeviceRevoke events").
    ByEventType { type_set: EventTypeSet, since: Option<HlcTimestamp> },

    /// Filter by HLC-range. ADMIN ONLY + emits AuditAccessExercised.
    ByHlcRange { from: HlcTimestamp, to: HlcTimestamp },

    /// Anonymized aggregate count. Available to ANY role under
    /// AuditAccessGradation::Anonymized_Aggregate. Returns counts only;
    /// no per-member identification. v1-beta-LB.
    AggregateCount { type_set: EventTypeSet, since: Option<HlcTimestamp> },

    // v1-beta CODEPOINT-RESERVE variants:

    /// Aggregate count with differential-privacy noise. Codepoint-reserve.
    AggregateCountWithDpNoise { type_set: EventTypeSet, epsilon: f64 } = 0xF1,

    /// Threshold-admin co-signed query. Codepoint-reserve.
    ThresholdAdminCoSigned { inner: Box<AuditQueryFilter>, signoffs: Vec<AdminSig> } = 0xF2,

    /// Time-locked query (results plaintext-released after delay).
    /// Codepoint-reserve.
    TimeLocked { inner: Box<AuditQueryFilter>, delay: Duration } = 0xF3,

    /// Zero-knowledge proof of property without per-record disclosure.
    /// Codepoint-reserve. Post-v1-beta.
    ZeroKnowledgeProof { property: AuditProperty } = 0xF4,
}
```

**v1-beta ships:** `All | OwnEvents | ByMember | ByEventType | ByHlcRange | AggregateCount`. Six variants; ~80 LOC additional beyond N2's ~60 LOC bringing total to ~140 LOC for op #23 + filter shape.

**v1-beta codepoint-reserves:** `AggregateCountWithDpNoise | ThresholdAdminCoSigned | TimeLocked | ZeroKnowledgeProof`. Four reserved discriminants (0xF1–0xF4). Doc-only at v1-beta; impl post-v1-beta.

**Rationale.** v1-beta needs enough query-shape coverage to serve legitimate moderation + post-mortem + GDPR-Art-15 use-cases without forcing always-`All`. Codepoint-reserve for the privacy-enhancing variants (DP-noise, threshold-cosigning, time-lock, ZK) preserves wire-format-additivity per §3.6b discipline.

**`EventTypeSet`.** Bitset over the `MembershipEvent` variants (per M-CONS-v2 F-N2-B). Allows queries like "show me all AdminKick + CompromiseResponseRotation events" without exposing all RoleChange.

---

## §4. Audit-log access-control gradations

Six gradations enumerated. Each is a value of `AuditAccessGradation` enum, stored as `MembershipSetPolicy.audit_access` config field.

```rust
/// Audit-log access-control gradation. Stored in MembershipSetPolicy.
/// Default per Kind: see §4-defaults below.
pub enum AuditAccessGradation {
    /// Anyone in MembershipSet (including Viewer role) can invoke any filter.
    /// Maximally transparent; minimally private.
    /// Use case: open communities, transparency-first organizations.
    Public_All_Members,

    /// Members + Admins can query OwnEvents + AggregateCount.
    /// Only Admins can query All/ByMember/ByEventType/ByHlcRange.
    /// Viewers cannot query at all.
    /// Use case: moderate-trust groups; standard chat.
    Member_OwnEvents,

    /// Only Admin role can query any non-OwnEvents filter.
    /// Member role can still query OwnEvents (GDPR Art. 15).
    /// v1-beta DEFAULT for Atrium Kind.
    AdminOnly,

    /// M-of-N admin co-signing required for any per-member-identifying filter.
    /// Single-admin queries restricted to AggregateCount + OwnEvents.
    /// CODEPOINT-RESERVE at v1-beta; impl post-v1-beta.
    ThresholdAdmin_M_of_N { m: u8, n_from_admins: bool },

    /// Admin queries return ciphertext immediately; plaintext released after
    /// delay (e.g., 7 days). Allows warrant-canary intervention.
    /// CODEPOINT-RESERVE at v1-beta; impl post-v1-beta.
    TimeLocked { delay_secs: u32 },

    /// Any role can query AggregateCount with differential-privacy noise.
    /// Identifying queries fail with permission error.
    /// CODEPOINT-RESERVE at v1-beta; impl post-v1-beta.
    Anonymized_Aggregate { epsilon: f64 },
}
```

**v1-beta ships:** `Public_All_Members | Member_OwnEvents | AdminOnly` (three variants). Default = `AdminOnly` for Atrium Kind; `AdminOnly` for DeviceMesh (admin=self); `Public_All_Members` for SingleDevice (only-self exists; vacuously private).

**v1-beta codepoint-reserves:** `ThresholdAdmin_M_of_N | TimeLocked | Anonymized_Aggregate`. Three reserved variants (discriminants 0x03/0x04/0x05). Doc-only; impl post-v1-beta. Closes T-B3-01 / T-B3-03 / T-B3-04 mitigation paths at codepoint level so post-v1-beta impl doesn't require wire-format change.

**Per-Kind defaults.**

| Kind | Default AuditAccessGradation | Rationale |
| --- | --- | --- |
| Atrium | AdminOnly | Standard expectation; multi-user; admin coordinates |
| DeviceMesh | AdminOnly | Single-user; admin = user; trivially permissive |
| SingleDevice | Public_All_Members | Only-self exists; vacuously safe |

**Why this needs Ben-ratification.** The choice of DEFAULT for the Atrium Kind is a policy-load-bearing default. `AdminOnly` matches Slack/Discord/Matrix conventions (so it's least-surprise for users coming from existing systems) AND it's the conservative-floor (members can't surveil each other). But it does NOT close T-B3-01 (malicious admin); only `ThresholdAdmin_M_of_N` does, and that's post-v1-beta. **Ben needs to ratify: ship AdminOnly default at v1-beta + reserve ThresholdAdmin codepoint for post-v1-beta? Or push for ThresholdAdmin at v1-beta as additional ~5–8 wave-days of Wave-MS-PRIMITIVE work?** My-pred: **ship AdminOnly default at v1-beta; reserve ThresholdAdmin codepoint; mint Compromise #58 covering the gap**. Rationale: ThresholdAdmin is post-v1-beta scope per N2's own op-classification (op #11 `threshold_admin_signoff` is codepoint-reserve at v1-beta); audit-access ThresholdAdmin gradation should follow the same v1-beta-codepoint-reserve trajectory.

---

## §5. Mitigation strategies

### §5.1 Audit-log retention-window (per-event TTL)

**Proposal.** Per-MembershipEvent-variant TTL configurable via `MembershipSetPolicy.audit_retention`:

```rust
pub struct AuditRetentionPolicy {
    /// Retention for routine hygiene events. Default 90 days.
    pub periodic_hygiene_ttl: Duration,

    /// Retention for role-changes. Default 1 year.
    pub role_change_ttl: Duration,

    /// Retention for admin-kicks. Default PERMANENT (None).
    /// Rationale: audit-trail discipline; cannot be retroactively erased.
    pub admin_kick_ttl: Option<Duration>,

    /// Retention for compromise-response rotations. Default PERMANENT.
    /// Rationale: security-event audit; required for post-mortem.
    pub compromise_response_ttl: Option<Duration>,

    /// Retention for member-departures (self-leave). Default 1 year.
    pub member_depart_ttl: Duration,

    /// Retention for device-revocations. Default 2 years.
    pub device_revoke_ttl: Duration,
}
```

**v1-beta default:** above defaults; configurable per-Atrium by Admin role at policy-update time.

**Implementation:** PeriodicHygiene op (per N2 op #14) sweeps and removes events past TTL; emits a single `MembershipEvent::AuditRetentionPurge { count, when }` per sweep.

**Closes:** T-B3-02 (compromised-device blast-radius limited to recent-history); T-B3-03 (compelled-disclosure can only return retained events).

**GDPR alignment:** finite retention defaults align with GDPR Art. 5(1)(e) storage-limitation principle.

### §5.2 Audit-log encryption-at-rest

**Proposal.** Each audit-log entry stored as:

```rust
pub struct StoredMembershipEvent {
    pub event_id: EventId,  // ULID
    pub hlc: HlcTimestamp,
    pub ciphertext: Vec<u8>,  // ChaCha20-Poly1305 ciphertext of MembershipEvent
    pub nonce: [u8; 12],
    pub signer_sig: Signature,  // signed by emitting admin
    pub prev_hash: [u8; 32],  // hash-chain link to previous event (Merkle log)
}
```

**Encryption key.** Derived per `K_audit = HKDF(K_principal, "benten-audit-log-key" || atrium_id)` where `K_principal` is the user's principal key per Layer-A vault. Per-Atrium derivation ensures cross-Atrium audit-log isolation (closes T-B3-05).

**Decryption.** Available to any role with `Read` permission on the audit-log per `AuditAccessGradation`. The `AuditQueryFilter` evaluator decrypts only the events matching the filter shape, minimizing exposure.

**Closes:** T-B3-02 (on-disk filesystem-compromise without DAK doesn't expose audit-log).

### §5.3 Audit-log tamper-evidence

N2 spec says "signed; tamper-evident" but does not specify construction. This brief proposes:

**Construction.** Hash-chained Merkle log (per [Crosby & Wallach, "Efficient Data Structures for Tamper-Evident Logging", USENIX Security 2009](https://static.usenix.org/event/sec09/tech/full_papers/crosby.pdf)):

1. Each `StoredMembershipEvent.prev_hash = SHA-256(prev_event.ciphertext || prev_event.signer_sig || prev_event.hlc)`.
2. Periodic checkpoint: every N events (default N=64), emit `MembershipEvent::AuditCheckpoint { merkle_root, range_start_hlc, range_end_hlc, signer: admin_did, sig }`. The Merkle root commits to all events in the range.
3. Verifier can produce a **logarithmic-size membership proof** for any event-in-log via standard Merkle audit-path.

**Merkle-vs-pure-hash-chain choice.** Per [Crosby & Wallach](https://static.usenix.org/event/sec09/tech/full_papers/crosby.pdf), Merkle trees produce 3 KB proofs for 80M-event logs vs 800 MB for pure hash-chains. v1-beta uses the simpler hash-chain (since per-Atrium audit-logs are bounded ~1K–10K events typical); v1-GM upgrades to Merkle with codepoint-additive AuditCheckpoint events (already reserved above).

**Tamper-detection.** Any verifier reconstructing the chain detects modification (prev_hash mismatch). Periodic checkpoints make divergence detection logarithmic.

**Aligns with:** [Keybase team-signature-chain pattern](https://book.keybase.io/docs/teams/details) — Keybase already uses signature-chained team-membership history; Benten uses analogous construction adapted for ChaCha20-Poly1305-encrypted entries.

**Closes:** historical-record-tampering attacks; supports post-mortem forensic reconstruction.

### §5.4 Differential-privacy noise on aggregate queries

**Proposal (codepoint-reserve for v1-beta; impl post-v1-beta).** For `AggregateCountWithDpNoise { type_set, epsilon }`:

1. Compute true count C.
2. Sample Laplace noise L ~ Laplace(0, 1/epsilon).
3. Return ⌊C + L⌋ as result.

**Epsilon choice.** epsilon=1.0 default; epsilon=0.1 strong privacy; epsilon=10.0 weak. Per [Dwork & Roth, "The Algorithmic Foundations of Differential Privacy", 2014].

**Closes:** T-B3-03 (legal-compulsion returns DP-noised count, not exact; warrant must specify exact-count for evidence-quality, narrowing scope).

**Not v1-beta scope.** Differential-privacy primitives are not currently in `benten-crypto-suite`; implementation requires careful PRNG + numerical-stability work. Codepoint-reserve at v1-beta is sufficient.

### §5.5 DID-rotation guidance + cross-fork hygiene

**Proposal.** Documentation-only at v1-beta in `docs/AUDIT-POLICY-GUIDANCE.md`:

> If you participate in multiple forked Atriums and are concerned about cross-fork admin-correlation, consider using **distinct DIDs per fork**. Benten's DID-rotation primitive (per N2 op #8 `member_key_rotation`) allows you to present different DIDs to different Atriums. This is operational hygiene; the system does not enforce DID-distinctness automatically (cross-fork DID-deduplication is a usability feature post-v1-beta).

**Closes:** T-B3-05 cross-fork DID-correlation residual.

---

## §6. Compromise #58 mint text

### Compromise #58 — Audit-log insider-correlation surface

**Disposition class.** `Composition-Hazard-Honest-Disclosure` (per M-C2 R-MCV2-9b proposed extension to disposition_class enum).

**Numbering note.** M-C3-v2 `cac10631` independently proposed `#58` for KEM-key-confirmation-failure across multi-stanza fan-out. **Collision resolution recommended at M-CONS-v2.1 re-consolidation: `#58 = audit-log-insider-correlation` (this disclosure); `#59 = KEM-key-confirmation-failure` (M-C3 candidate renumbered).** Rationale: B-3 is structurally load-bearing (touches `MembershipSetPolicy.audit_access` config field, `AuditQueryFilter` enum, per-Kind defaults, retention-policy struct, encryption-at-rest derivation); M-C3's candidate was self-rated LOW-priority "mint-or-skip" with cleaner orthogonality to surrounding amendments.

#### Statement

`MembershipSet::audit_log_query` (N2 op #23; F-N2-B) reads back the `MembershipEvent` typed-enum stream containing per-Atrium membership-history (admin-kicks, role-changes, member-key-rotations, self-leaves, device-revocations, compromise-response rotations, periodic-hygiene events). This stream is by-design queryable by any role authorized by `MembershipSetPolicy.audit_access` (defaults to `AdminOnly` for Atrium Kind).

**Insider-correlation is by-design, not a defect.** An authorized querier (Admin role under default config) learns the full structured (HLC, member-DID, op-type, by-whom) tuple for every event. This composes with:

1. **Inv-20 clause-d per-recipient unlinkability** — clause-d preserves unlinkability against **network-observer** adversaries (cryptographic non-correlation at the wire). Clause-d **DOES NOT** preserve unlinkability against **authorized-insider** adversaries (an Admin who legitimately holds K_Set + Audit role).
2. **Honest-disclosure required:** Inv-20 clause-d framing must scope explicitly to "per-recipient unlinkability against NETWORK-OBSERVER adversaries; insider-member observability of MembershipEvent history is by-design and governed by `MembershipSetPolicy.audit_access`."

#### Threat surface

Eight enumerated threats; full table in M-CONS-v2.1 §11 THREAT-MODEL skeleton:

- **T-B3-01 Malicious Admin** enumerates membership-history to identify dissidents/whistleblowers (HIGH severity).
- **T-B3-02 Compromised Admin device** exfiltrates audit-log to external adversary (MED-HIGH).
- **T-B3-03 Coerced Admin** (subpoena/nation-state) reveals audit-log under legal compulsion (HIGH for activism/journalism users).
- **T-B3-04 Audit-vs-privacy tension** as a design choice surfaced through `AuditAccessGradation` policy field.
- **T-B3-05 Cross-fork insider-correlation** via parent-fork admin role overlap (MED-HIGH).
- **T-B3-06 Iroh-gossip topic-id × audit-log** composition (MED; closes with MCV2-C-3).
- **T-B3-07 Compromise #48 K_Set-shape × audit-log** composition (LOW-MED).
- **T-B3-08 Member queries own history** (legitimate, not an attack).

#### Mitigation (shipped at v1-beta)

1. **R-MCV2-(B-3) admin-role RBAC gate**: `audit_log_query` requires `Admin` role OR `Member` role with `OwnEvents` filter under `AuditAccessGradation::Member_OwnEvents`. Closes member-vs-member correlation.
2. **`AuditAccessGradation` policy field**: 3 v1-beta-LB variants (`Public_All_Members | Member_OwnEvents | AdminOnly`) + 3 codepoint-reserve variants (`ThresholdAdmin_M_of_N | TimeLocked | Anonymized_Aggregate`). Per-Kind defaults: Atrium=AdminOnly; DeviceMesh=AdminOnly; SingleDevice=Public_All_Members.
3. **`AuditQueryFilter` shape constraints**: 6 v1-beta-LB filter variants (`All | OwnEvents | ByMember | ByEventType | ByHlcRange | AggregateCount`) + 4 codepoint-reserve. Each invocation auto-emits `MembershipEvent::AuditAccessExercised { admin, filter_shape, hlc }` making admin-surveillance itself auditable.
4. **`AuditRetentionPolicy` config**: per-event-type TTL (defaults 90d hygiene, 1y role-change, permanent admin-kick + compromise-response, 2y device-revoke).
5. **Audit-log encryption-at-rest**: ChaCha20-Poly1305 under per-Atrium derived key `K_audit = HKDF(K_principal, "benten-audit-log-key" || atrium_id)`.
6. **Tamper-evidence**: hash-chained MembershipEvent entries + periodic AuditCheckpoint events (Merkle-tree-upgradeable at v1-GM).
7. **Cross-fork audit-log isolation**: per-fork `K_audit` derivation; parent-fork admin role does NOT grant child-fork audit-log access without explicit role-acceptance in child fork.

#### Mitigation (codepoint-reserved for post-v1-beta)

8. **`ThresholdAdmin_M_of_N` gradation** (codepoint 0x03): M-of-N admin co-signing required for per-member-identifying queries. Closes T-B3-01 (malicious-admin) + T-B3-03 (single-admin coercion) for high-stakes Atriums.
9. **`TimeLocked` gradation** (codepoint 0x04): query results ciphertext-immediate, plaintext-delayed. Warrant-canary-compatible. Closes T-B3-03 (compelled-disclosure delay).
10. **`Anonymized_Aggregate` gradation with DP-noise** (codepoint 0x05; `AggregateCountWithDpNoise` filter codepoint 0xF1): differential-privacy noised aggregate counts. Closes T-B3-03 (counts-not-records narrows warrant scope).
11. **`ZeroKnowledgeProof` filter** (codepoint 0xF4): post-v1-beta zero-knowledge audit primitives. Aligned with [zk-audit literature 2025](https://arxiv.org/abs/2512.14737).

#### Cross-references

- **Inv-20 clause-d** — scope-clarification mandated: "per-recipient unlinkability against NETWORK-OBSERVER adversaries; insider observability is by-design."
- **Compromise #48** (K_Set-shape-leak) — composition-hazard documented in T-B3-07.
- **Compromise #55** (GDPR-RTBF P2P-by-design) — analogous Composition-Hazard-Honest-Disclosure pattern; audit-log retention policy aligns with GDPR Art. 5(1)(e) storage-limitation.
- **Compromise #57 / #59** (iroh-gossip topic-id leak per MCV2-C-3) — composition-hazard documented in T-B3-06.
- **N2 op #23** — `audit_log_query` spec; this Compromise governs its access-control surface.
- **N2 op #14** — `periodic_hygiene` op; performs AuditRetention sweep.
- **F-N2-B** — `MembershipEvent` typed-enum (per M-CONS-v2 F-N2-B absorbing F-A2).
- **F-N2-A** — sibling-trait families (`MembershipSetAudit` carries op #23).

#### Revisit triggers

- Ratification of `ThresholdAdmin_M_of_N` impl (post-v1-beta wave) → update Compromise #58 to mark item 8 as closed.
- Ratification of `TimeLocked` impl → close item 9.
- Ratification of `Anonymized_Aggregate` + DP-noise impl → close item 10.
- New cross-amendment composition surface discovered (e.g., MLS chained-mode per N4 changing `MembershipEvent` shape) → revisit T-B3-06 amplification.
- External audit (per L5) finds residual disclosure surface not enumerated in §2 → mint follow-on Compromise.

#### Honest disclosure footprint

Recommended `docs/THREAT-MODEL.md` row (per L5 §5 prescriptive output):

> **Audit-log insider-correlation (Compromise #58, by-design).** Benten's `audit_log_query` op exposes structured membership-history to authorized queriers per the Atrium's `AuditAccessGradation` policy. By default, only Admin role can perform per-member-identifying queries. Member role can query own-history (GDPR Art. 15 alignment). Per-recipient unlinkability (Inv-20 clause-d) is preserved against network-observers but is not preserved against authorized-insider queriers. For high-stakes deployments (journalism, activism, sensitive workplaces), Atrium-creators SHOULD select `ThresholdAdmin_M_of_N` (post-v1-beta) or set `AuditRetentionPolicy` to minimize retention. See `docs/AUDIT-POLICY-GUIDANCE.md`.

---

## §7. Final recommendation + R0 plan-doc implications

### §7.1 v1-beta-LOAD-BEARING audit-log discipline (Wave-MS-PRIMITIVE additions)

Beyond N2's existing ~60 LOC `audit_log_query` + F-N2-B `MembershipEvent` enum:

1. **`AuditAccessGradation` enum** (3 v1-beta variants + 3 codepoint-reserve) — ~30 LOC.
2. **`AuditQueryFilter` enum** (6 v1-beta variants + 4 codepoint-reserve) — ~50 LOC.
3. **`AuditRetentionPolicy` struct** (6 fields) — ~25 LOC.
4. **`MembershipSetPolicy.audit_access` + `audit_retention` fields** — ~15 LOC.
5. **Per-Kind defaults logic** (Atrium=AdminOnly; DeviceMesh=AdminOnly; SingleDevice=Public_All_Members) — ~10 LOC + tests.
6. **`MembershipEvent::AuditAccessExercised` variant** — ~5 LOC.
7. **`MembershipEvent::AuditCheckpoint` variant + Merkle-upgrade-path** — ~15 LOC.
8. **`MembershipEvent::AuditRetentionPurge` variant** — ~5 LOC.
9. **`K_audit` HKDF derivation + at-rest encryption** — ~30 LOC + test vectors.
10. **Hash-chain `prev_hash` tamper-evidence + checkpoint logic** — ~40 LOC + tests.
11. **`audit_log_query` filter evaluator + decryption-on-demand** — ~80 LOC (replaces N2's ~60).
12. **Per-fork `K_audit` isolation** — ~10 LOC + cross-fork tests.

**Total additional v1-beta-LB:** ~315 LOC + ~200 LOC of tests + ~60 LOC of test-vectors. **~1.5 wave-days incremental** beyond N2's baseline Wave-MS-PRIMITIVE estimate.

### §7.2 Wave-MS-PRIMITIVE composition revision

M-CONS-v2 §-add Wave-MS-PRIMITIVE central estimate `~10.5-13 wave-days`. With this brief's recommendations:

- N2 audit_log_query baseline: included.
- +1.5 wave-days for §7.1 audit-log discipline.
- +0.3 wave-days for Compromise #58 mint + Inv-20 clause-d clarification doc.

**Revised Wave-MS-PRIMITIVE central:** `~12.0-14.5 wave-days` (bracket; was `~10.5-13` pre-B-3-closure).

### §7.3 M-CONS-v2.1 re-consolidation §-add list

For M-R0 plan-doc authoring against M-CONS-v2.1:

1. **§7.1 list above** as new MembershipSet-Spec §10 "Audit-log access-control" subsection.
2. **Inv-20 clause-d scope clarification**: amend "per-recipient unlinkability" to read "per-recipient unlinkability **against network-observer adversaries**; insider-member observability of MembershipEvent history is by-design and governed by MembershipSetPolicy.audit_access (per Compromise #58)."
3. **Compromise #58 mint** per §6 above; place in §17 Decisions recorded.
4. **Compromise #58 collision resolution**: rename M-C3's candidate to `#59`.
5. **F-amendment row F-N2-D**: "Audit-log access-control discipline (AuditAccessGradation + AuditQueryFilter + AuditRetentionPolicy + at-rest encryption + tamper-evidence + cross-fork isolation)." Place after F-N2-C in the amendment registry.
6. **R-MCV2-(B-3) closure trail**: mark closed by F-N2-D + Compromise #58 in the MCV2 refinements registry.
7. **`docs/AUDIT-POLICY-GUIDANCE.md`** new doc stub: gradation-selection guidance for Atrium-creators (post-v1-beta UX work; v1-beta = doc stub only).
8. **`docs/THREAT-MODEL.md` §-add row** per §6 "Honest disclosure footprint" above.

### §7.4 AtriumPolicy/MembershipSetPolicy configuration field

```rust
/// Extends MembershipSetPolicy with audit-log access-control configuration.
/// Added per Compromise #58 + F-N2-D.
pub struct MembershipSetPolicy {
    // ... existing fields per M-CONS-v2 ...

    /// Audit-log access-control gradation. Per-Kind default (§4 above).
    /// Can be changed by Admin role at policy-update time (op #16
    /// update_policy_value); change is itself audited.
    pub audit_access: AuditAccessGradation,

    /// Audit-log retention TTL per event-type. Default per §5.1.
    /// Can be changed by Admin role at policy-update time.
    pub audit_retention: AuditRetentionPolicy,
}
```

### §7.5 Single most load-bearing recommendation (restated)

**Mint Compromise #58 + F-N2-D + `AuditAccessGradation` config field + `AuditQueryFilter` enum + `AuditRetentionPolicy` struct + at-rest encryption + hash-chain tamper-evidence + per-fork K_audit isolation as v1-beta-LOAD-BEARING bundle in Wave-MS-PRIMITIVE.** Closes T-B3-01 partially (admin-role-gate baseline); T-B3-02 fully (encryption + retention); T-B3-04 fully (explicit policy choice); T-B3-05 fully (per-fork isolation); T-B3-06 / T-B3-07 by cross-reference. Codepoint-reserves T-B3-01-full-closure + T-B3-03 closures for post-v1-beta `ThresholdAdmin_M_of_N` / `TimeLocked` / `Anonymized_Aggregate` impl. Honest-disclosure via Compromise #58 makes the residual gradient explicit to Atrium-creators.

---

## §8. Confidence + scoping caveats

**Confidence ratings.**

| Claim | Confidence |
| --- | --- |
| §1 verdict: B-3 is real over-disclosure surface, R-MCV2-(B-3) is necessary-not-sufficient | **HIGH** |
| §2 threat enumeration covers the principal surfaces | **HIGH** for T-B3-01–05; **MEDIUM-HIGH** for T-B3-06/07 composition; **HIGH** for T-B3-08 |
| §3 AuditQueryFilter shape (6 v1-beta + 4 codepoint-reserve) | **MEDIUM-HIGH** — covers principal use-cases; specific variant set is reviewable |
| §4 AuditAccessGradation 3+3 split | **MEDIUM-HIGH** — 3 v1-beta is conservative; some users may want ThresholdAdmin at v1-beta (Ben-decision-point) |
| §5 mitigation menu (retention TTL + encryption-at-rest + tamper-evidence + DP-noise + cross-fork isolation) | **HIGH** on retention/encryption/tamper; **MEDIUM** on DP-noise impl-detail (post-v1-beta scope) |
| §6 Compromise #58 mint text + collision resolution recommendation | **HIGH** on B-3-takes-#58; **MEDIUM-HIGH** on collision-rename recommendation (Ben-ratification preferred) |
| §7 Wave-MS-PRIMITIVE +1.5–2 wave-days estimate | **MEDIUM-HIGH** — bracketed; could expand if test-vector work scales |

**Caveats + uncertainty.**

1. **Threshold-admin-at-v1-beta vs post-v1-beta** is a Ben-decision-point. Recommending codepoint-reserve at v1-beta is conservative; Ben may prefer impl-at-v1-beta for stronger T-B3-01 / T-B3-03 closure. ~5–8 wave-days additional if impl-at-v1-beta.
2. **Per-Kind defaults** are policy choices, not technical inevitabilities. The proposed defaults (Atrium=AdminOnly; DeviceMesh=AdminOnly; SingleDevice=Public_All_Members) are reviewable; an alternative would be `Member_OwnEvents` as the Atrium default (more privacy-friendly; less moderation-friendly).
3. **`AuditAccessExercised` event emission frequency** could become noisy in high-traffic Atriums (every query emits an event; admin-surveillance reviews emit events about reviews-of-events). Possible amplification spiral. Mitigation: emit-once-per-session not per-query; or use bloom-filter aggregation post-v1-beta. Not closed at v1-beta.
4. **Tamper-evidence Merkle upgrade-path** at v1-GM is codepoint-additive (AuditCheckpoint variant reserved). If v1-beta hash-chain construction has a flaw discovered post-ship, upgrade is breaking-change-on-wire. Mitigation: pin the hash-chain construction to Crosby & Wallach 2009 published spec exactly.
5. **DP-noise parameter choice** (epsilon default) is post-v1-beta. Wrong default could leak too much (epsilon too large) or be too noisy to be useful (epsilon too small). Defer to careful impl + literature review.
6. **Cross-fork DID-correlation residual** depends on user discipline (DID-rotation per fork). System cannot enforce; documentation guidance is the closure. Acceptable per Compromise #58 honest-disclosure framing.

**Out-of-scope for this brief.**

- Full design of post-v1-beta ThresholdAdmin / TimeLocked / Anonymized_Aggregate impls (codepoint-reserved here; impl is post-v1-beta wave work).
- Specific UX for Atrium-creator audit-policy selection (post-v1-beta UI scope; not Wave-MS-PRIMITIVE).
- Integration with external compliance frameworks (SOC2, HIPAA, GDPR audit-requirements deep-dive — separate compliance-doc work post-v1-GM).

---

## §9. Synthesis + handoff

**To M-CONS-v2.1 re-consolidator.**

1. Add **F-N2-D** F-amendment row (Audit-log access-control discipline) to amendment registry.
2. Update Inv-20 clause-d framing to "network-observer-only" scoping.
3. Mint Compromise #58 per §6 text above.
4. Renumber M-C3's KEM-key-confirmation candidate to #59 (resolve collision).
5. Expand Wave-MS-PRIMITIVE estimate `~10.5-13` → `~12.0-14.5` wave-days.
6. Add `docs/AUDIT-POLICY-GUIDANCE.md` stub to deliverable list.
7. Add `docs/THREAT-MODEL.md` row per §6 honest-disclosure footprint.
8. Mark R-MCV2-(B-3) closed by F-N2-D + Compromise #58.

**To M-R0 plan-doc author (Wave-MS-PRIMITIVE owner).**

1. Adopt §7.1 12-item additional v1-beta-LB list (~315 LOC + ~200 LOC tests + ~60 LOC vectors).
2. Implement `AuditAccessGradation` config field with 3 v1-beta-LB variants + 3 codepoint-reserve.
3. Implement `AuditQueryFilter` enum with 6 v1-beta-LB variants + 4 codepoint-reserve.
4. Implement `AuditRetentionPolicy` struct + per-Kind defaults + PeriodicHygiene sweep logic.
5. Implement K_audit HKDF derivation + ChaCha20-Poly1305 encryption-at-rest + hash-chain tamper-evidence.
6. Implement per-fork K_audit isolation + cross-fork explicit-opt-in role-acceptance.
7. Emit AuditAccessExercised on every query invocation.
8. Pin Merkle-upgrade codepoint-additive AuditCheckpoint for v1-GM.

**To Ben (decision-points).**

| Decision | Recommendation | Confidence |
| --- | --- | --- |
| Ship `ThresholdAdmin_M_of_N` at v1-beta impl, or codepoint-reserve only? | **codepoint-reserve only**; impl post-v1-beta. ~5–8 wave-days saved at v1-beta; Compromise #58 covers residual honestly. | MEDIUM-HIGH |
| Default `AuditAccessGradation` for Atrium = `AdminOnly` or `Member_OwnEvents`? | **`AdminOnly`** — matches Slack/Discord/Matrix least-surprise; conservative-floor; covered by Compromise #58 disclosure. | MEDIUM-HIGH |
| Default `audit_retention.admin_kick_ttl` = PERMANENT or 5 years? | **PERMANENT (`None`)** — audit-trail discipline; compromise-response forensic alignment. | HIGH |
| Compromise #58 numbering collision (M-C2 audit-log vs M-C3 KEM-key-confirmation) | **#58 = audit-log (this brief); #59 = KEM-key-confirmation (M-C3 renumber)**. Rationale §1 + §6. | MEDIUM-HIGH |
| Add `docs/AUDIT-POLICY-GUIDANCE.md` stub at v1-beta? | **YES** — minimal doc; ~0.2 wave-days; enables Atrium-creator informed-consent at creation-time UX (post-v1-beta UI consumes it). | HIGH |

**Sources cross-referenced.**

- [Crosby & Wallach 2009 — Efficient Data Structures for Tamper-Evident Logging](https://static.usenix.org/event/sec09/tech/full_papers/crosby.pdf) (Merkle log construction).
- [Signal Private Group System](https://signal.org/blog/pdfs/signal_private_group_system.pdf) (AuthCredentials + UID-blinding).
- [Quarkslab — Secure Messaging Apps and Group Protocols Part 2](https://blog.quarkslab.com/secure-messaging-apps-and-group-protocols-part-2.html) (Sender Keys baseline; PCS limitations).
- [Membership Privacy for Asynchronous Group Messaging — IACR 2022/046](https://eprint.iacr.org/2022/046.pdf) (formal membership-privacy framework).
- [Sender Keys analysis — Balbas/Collins/Gajland 2023](https://arxiv.org/pdf/2301.07045) (SSK gaps).
- [Keybase team-signature-chain](https://book.keybase.io/docs/teams/details) (signature-chained team-membership precedent).
- [Microsoft Insider Risk Management audit log](https://learn.microsoft.com/en-us/purview/insider-risk-management-audit-log) (privacy-by-design pseudonymization + RBAC pattern).
- [Audit-LLM multi-agent log-based insider threat detection — arXiv 2408.08902](https://arxiv.org/pdf/2408.08902) (modern ITD methodology).
- [RFC 9420 MLS Protocol](https://datatracker.ietf.org/doc/html/rfc9420/) (epoch-based group-state transcript pattern).
- [Zero-Knowledge Audit for Internet of Agents — arXiv 2512.14737](https://arxiv.org/abs/2512.14737) (ZK-audit literature; v1-GM trajectory).
- [Show Me You Comply Without Showing Me Anything — arXiv 2510.26576](https://arxiv.org/html/2510.26576v1) (ZK-software-auditing).
- [Signal warrant-canary precedent](https://signal.org/bigbrother/) (TimeLocked gradation motivation).
- [Obsidian Sync Cure53/ToB audit posture](https://obsidian.md/blog/cure53-tob-sync-audits/) (audit-deliverable-shape precedent).

---

**End of M-P5-B-3 audit-log threat-model brief.**
