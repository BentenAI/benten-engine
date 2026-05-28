# MembershipSet N3 — Continuous-rotation use-case edge cases

**Author:** N3 specialist (continuous-rotation edge-case stress-tester, 3-of-4 post-M-CONS refinement)
**Date:** 2026-05-28
**Branch:** `phase-4-meta-core/membership-set-n3-continuous-rotation-edge-cases`
**HEAD pre-flight:** `2172cb6d` (clean, up-to-date with origin/main)
**Posture:** Stress-test the M5 + M-CONS "continuous-rotation not needed for v1-beta" conclusion. Find use-cases where it's wrong; identify whether the v1-beta wire format keeps the door open additively; recommend keep / refine / overturn.

---

## §1 — Executive verdict

**M5's conclusion HOLDS for v1-beta with one sharpening + one watch-list mint.**

Stripped down: M5 + M-CONS converged on "no continuous-rotation at v1-beta; ship MultiRecipientSealing + ForkOnly/AdminKickEpoch RotationPolicy + reserve `MembershipSetKind::AtriumWithCGKA` codepoint." After exhaustively probing regulatory contexts (HIPAA / PCI-DSS / GDPR / classified), Benten-specific use-cases (journalism, AI-agent members, long-lived Atriums, cross-jurisdictional), and four subtle attack scenarios, I find:

1. **No regulatory framework MANDATES continuous (CGKA-style) rotation.** Best-practice guidance points at quarterly-to-annual rotation; this is fully served by **PeriodicHygiene fork** (a forked MembershipSet snapshot every N days) WITHOUT a CGKA primitive. GDPR right-to-be-forgotten requires **crypto-shredding** (key destruction) not continuous rotation.

2. **The MOST honest threat that argues for continuous-rotation is silent / undetected endpoint compromise** (Mandiant M-Trends 2025: median dwell time 11 days; IBM 2025: average 207 days for undetected lateral movement). Even here, Benten's "forkable Atrium + leaver-keeps-past-content" semantic structurally **forecloses** PCS-against-leavers; PCS only helps against **silent compromise of a CURRENT member**, where continuous rotation limits the post-compromise window. This is real but narrow.

3. **The v1-beta wire format M2 + M-CONS ship is structurally compatible with adding continuous-rotation post-v1-beta WITHOUT a wire-format break.** The MembershipSet snapshot carries `version: u8` (codepoint-additive), `generation: u32` (already monotonic), `parent_membership_set_id: Option<...>` (fork-aware), `policy.refresh_required_secs: Option<u64>` (cadence hook, already wire-allocated and `None`-at-v1-beta), and the K_Set distribution rides a SEPARATE `MultiStanzaHpkeEnvelope` keyed by `(set_id, generation)`. Frequent `gen++` increments + frequent multi-stanza re-distributions are **wire-format-additive today**; the only thing the `AtriumWithCGKA` codepoint adds is a TreeKEM-shaped efficient log-N rotation primitive, not the wire shape.

4. **Sharpening:** the M2 RotationPolicy enum at HEAD has only TWO arms (`ForkOnly`, `AdminKickEpoch`). The task framing surfaced a richer rotation-trigger taxonomy (DeviceRevoke / MemberDeparts / CompromiseResponse / PeriodicHygiene). I recommend **a doc-only "rotation-trigger taxonomy" appendix** to MembershipSet-Spec at v1-beta that names these triggers and maps each to its v1-beta-shipped mechanism (mostly: "ForkOnly fork-on-event"). This is process hygiene, not a code change — but it forecloses the future "we never thought about PeriodicHygiene" footgun.

5. **Watch-list mint (NEW):** Compromise #54 — **Continuous-rotation deferral risk.** Names the silent-undetected-CURRENT-member-compromise scenario (Scenario A/B/C below) as the canonical post-v1-beta trigger for revisiting `AtriumWithCGKA`. Threshold: any Benten production deployment that materially serves journalism source-protection, AI-agent-rich multi-tenant Atriums, or healthcare-grade regulatory regimes triggers a Phase-N CGKA re-evaluation. Pair with the existing M5 §9.2 disclosure pattern (Compromise #52 "MembershipSet-no-PCS-against-removed-members").

**Confidence on verdict:** HIGH that M5's conclusion holds for v1-beta. MEDIUM-HIGH that wire format is additive (one residual question on the §5.3 "policy refresh_required_secs" semantic). MEDIUM on the rotation-trigger-taxonomy sharpening being worth the doc spend at v1-beta vs Phase-N. HIGH on the watch-list mint.

---

## §2 — Compliance + regulatory analysis

### §2.1 HIPAA (healthcare ePHI)

**What HIPAA actually requires.** HIPAA does NOT prescribe a specific key-rotation cadence. The Security Rule (45 CFR §164.312(a)(2)(iv) "Encryption and Decryption" + §164.312(e)(2)(ii)) requires encryption + key management as a "reasonable and appropriate" safeguard. NIST SP 800-57 + industry best-practice guidance (Censinet, Konfirmity, Kiteworks) consistently land on:

- **90-day rotation for high-risk systems / session keys**
- **Annual rotation for data-encryption keys**
- **Immediate rotation on suspected breach**

**Does this require CGKA-style continuous rotation?** NO. The "90-day rotation" cadence is fully served by a `RotationPolicy::PeriodicHygiene` trigger that forks the MembershipSet every 90 days (mints new K_Set + redistributes via MultiStanzaHpkeEnvelope). This is operationally identical to admin-initiated fork-on-cadence. It does NOT need TreeKEM.

**Verdict for Benten-Atrium-as-HIPAA-Substrate use-case.** ForkOnly + scheduled-PeriodicHygiene fork (admin or cron-bot signed) is HIPAA-aligned. Continuous-rotation is overkill. **M5 verdict holds.**

**Caveat: HIPAA audit-log retention.** HIPAA requires 6-year retention of key-management audit logs. The M-CONS amendment F-A2 (`MembershipEvent` typed-enum per-Kind audit stream) needs to NAME 6-year-retention as a downstream consumer requirement, not as a MembershipSet invariant. Doc-only; out-of-scope for continuous-rotation question.

### §2.2 PCI-DSS (payment card data)

**What PCI-DSS actually requires.** PCI-DSS v4.0 Requirement 3.6.4 mandates "cryptographic key changes at the end of their cryptoperiod" — explicitly NOT prescribing a fixed cadence, but requiring the org to document a cryptoperiod based on NIST SP 800-57 + key strength + exposure volume. Industry baseline: **annual minimum; quarterly for cardholder-data-encryption keys; immediate on suspected compromise or admin departure.**

**Does this require CGKA-style continuous rotation?** NO. Same as HIPAA: a `RotationPolicy::PeriodicHygiene` cron-fork at the documented cryptoperiod is PCI-DSS-compliant. PCI-DSS Requirement 3.6.5 (key changes when key integrity is weakened or known/suspected compromise) maps to `RotationPolicy::CompromiseResponse` — admin-initiated emergency fork. Both ship at v1-beta as fork-on-event triggers.

**Verdict.** ForkOnly + scheduled-PeriodicHygiene + CompromiseResponse covers PCI-DSS. **M5 verdict holds.**

**Open question I cannot resolve from regulatory text alone:** does PCI-DSS care about post-compromise security against a REMOVED admin? Re-reading PCI Req 3.6.5 + 8.3.x suggests YES — when an admin with key-management privileges departs, the org must rotate ALL keys that admin had access to. In Benten this is: `RotationPolicy::AdminDepart` triggers a fork that excludes the departed admin. Fork-on-admin-departure is wire-format-shipping at v1-beta as a special case of `AdminKickEpoch` / `ForkOnly`. So Benten serves PCI-DSS-compliant admin-departure semantics WITHOUT CGKA. **M5 verdict holds.**

### §2.3 GDPR right-to-be-forgotten

**What GDPR Article 17 actually requires.** When a data subject exercises right-to-be-forgotten, the controller must erase or render-permanently-inaccessible the subject's personal data. The legally-recognized cryptographic-erasure pattern is **crypto-shredding**: delete the key used to encrypt the subject's data; the ciphertext becomes permanently unreadable.

**Does this require CGKA-style continuous rotation?** **NO — and worse, CGKA actively FIGHTS this pattern.** A real CGKA forward-secures the SHARED group secret across epochs but does NOT shred per-subject keys. To honor GDPR right-to-be-forgotten in a CGKA-using Atrium, the controller would still need a separate per-subject-keyed sub-encryption layer (so deleting one subject's key shreds only their data, not the whole Atrium). This is orthogonal to CGKA.

**What Benten DOES need for GDPR.** A per-subject-key layer + key-destruction primitive. This is OUT-OF-SCOPE for MembershipSet — it lives at the Atrium-data layer (per-subject content keys). Worth a separate Phase-N design (compromise registry candidate: "GDPR-crypto-shredding-substrate"). I do NOT recommend a v1-beta amendment for this — but I DO recommend the M-CONS §1.4 disclosure mentioning Compromise #52 also disclose Compromise #55 candidate: **"MembershipSet does not provide per-subject crypto-shredding for GDPR right-to-be-forgotten; that lives at a separate Atrium-data-layer per-subject-key design."** Doc-only; v1-beta-LB. **M5 verdict holds.**

### §2.4 Classified / compartmented government contexts

**What classified contexts require.** This is mostly out of Benten's v1-beta target audience — but: NIST SP 800-57 Part 1 Rev 5 cryptoperiods for SECRET / TOP-SECRET range from 1 year (for stored data) down to per-session (for key-establishment material). The cadence is rotation-based, not CGKA-based. Compartmented Information (SCI / SAP) sometimes requires per-compartment-PCS-on-member-departure — which is closer to a real CGKA need.

**Does this require CGKA-style continuous rotation?** **PARTIALLY YES** for some compartmented-info use-cases, but Benten is NOT targeting classified/SAP at v1-beta. **M5 verdict holds with caveat:** if Benten ever targets a USG / classified customer, the `AtriumWithCGKA` codepoint becomes the trigger to revisit. Pre-reserved by M5. Good.

### §2.5 SOX (financial reporting)

**What SOX requires.** SOX Section 404 internal controls over financial reporting; key management is one of several IT general controls. SOX does NOT mandate a cryptographic-key-rotation cadence. Auditors apply industry best-practice (often PCI-DSS-style). **M5 verdict holds; no continuous-rotation trigger.**

### §2.6 Net regulatory verdict

**No regulatory regime in scope for v1-beta REQUIRES continuous (CGKA-style) rotation.** All regulatory rotation requirements are served by `RotationPolicy::PeriodicHygiene` + `CompromiseResponse` + `AdminDepart` triggers that fork-on-event. These fit WITHIN ForkOnly + AdminKickEpoch wire shape today.

**Sharpening recommended (doc-only):** the M2 RotationPolicy enum should EITHER stay as 2-arm `ForkOnly | AdminKickEpoch` with a doc-only "trigger taxonomy" appendix naming the regulatory-triggered fork events, OR expand to a richer enum WITH the same wire-format generation+fork semantics. I lean **2-arm-enum + appendix** because it minimizes v1-FROZEN-INTERFACE surface area + the regulatory-trigger semantics live in the application calling `MembershipSet::fork(reason: RotationTrigger)` not in the wire format.

---

## §3 — Benten-specific use-cases

### §3.1 Time-bound sub-graph sharing (auto-expire)

**Use-case.** "Share /trip/notes/2026 with the family for 30 days, then auto-expire."

**Does this need continuous rotation?** NO. Auto-expire is served by:
- (a) A sub-Atrium with `policy.refresh_required_secs: Some(30 * 86400)` that requires re-attestation every 30 days, OR
- (b) A scheduled fork-then-don't-add-old-members-back at 30 days, OR
- (c) Cap-layer expiry on the share grant (`benten-caps` UCAN-style `not_after` claim).

(c) is the lightest-weight; the cap layer ALREADY supports time-bounded grants per CLAUDE.md #19. (a) is the MembershipSet-layer answer. (b) is the bluntest. None require CGKA.

**Verdict.** Use cap-layer time-bound grants. **M5 verdict holds.**

### §3.2 High-sensitivity Atriums (journalism / whistleblower / activist)

**Use-case.** "I'm a journalist coordinating with a source via Atrium. The source's device might be compromised tomorrow without warning. I want past content I sent to be unreadable to a future-attacker who compromises the source's device."

**This is the strongest case for FS at the Atrium layer.** It's the SecureDrop / Signal-Sealed-Sender pattern. But:

- **Forward-secrecy-against-FUTURE-attacker** requires per-message ephemeral key + post-message zeroization. This is **per-message FS**, not membership-rotation FS. It requires the SOURCE device to discard their reading-key after reading. Continuous-rotation of K_Atrium does NOT achieve this — the source's device still has K_Atrium and the ciphertext it already received.
- **The right primitive for this use-case is Signal-double-ratchet-style per-message ephemeral keys** (or HPKE-mode-PSK with discarded ephemeral receiver state). This is NOT what a CGKA delivers. CGKA forward-secures the SHARED group secret across membership epochs; per-message FS is orthogonal.
- **The Benten-forkable-Atrium semantic explicitly CONTRADICTS this use-case.** Ben 2026-05-27: "kicked member retains pre-kick content forever." A journalist who wants FS-against-source-device-compromise would need a DIFFERENT Atrium semantic (member-leave-shreds-past-content), which is incompatible with the forkable-by-design ratification.

**Verdict.** Journalism source-protection is a DIFFERENT use-case requiring DIFFERENT crypto (per-message FS via ephemeral keys; ratcheting receive-state). It is NOT served by continuous-rotation; it would require either (a) a separate `MembershipSetKind::EphemeralRatchet` design or (b) an application-layer per-message-FS encryption stacked on top of MembershipSet. **M5 verdict holds**, but a watch-list mint is justified for "Benten currently has no answer for journalist-source-protection FS" — this is post-v1-beta scope. I propose adding to Compromise #54 (continuous-rotation deferral risk) a sub-clause naming this as a SEPARATE design problem, not a continuous-rotation problem.

### §3.3 Compliance-driven periodic-hygiene rotation

**Use-case.** "HIPAA-compliant Benten deployment; 90-day key rotation per compliance policy."

**Does this need CGKA?** NO. As §2.1 above: `RotationPolicy::PeriodicHygiene` cron-fork every 90 days serves this exactly. No CGKA. **M5 verdict holds.**

**Sharpening:** the M2 design doesn't explicitly NAME a periodic-fork trigger. Recommended sharpening: rename M2 §4 `remove_member`-with-RotationPolicy to a richer `rotate(trigger: RotationTrigger)` op where `RotationTrigger ∈ {AdminKick, MemberDepart, AdminDepart, DeviceRevoke, CompromiseResponse, PeriodicHygiene}`, AND `RotationPolicy` enum stays at 2 arms (ForkOnly / AdminKickEpoch). Triggers are doc-only metadata that becomes part of the `MembershipEvent` audit stream (F-A2) — they DON'T change the wire format. Wire-format spec: the `MembershipEvent::AtriumKick { reason: RotationTrigger, ... }` field is the audit-log mirror.

### §3.4 AI-agent-as-member scenarios

**Use-case.** "My Atrium has 3 humans + 1 AI agent member. The agent's keys might leak into a training corpus if the agent's host LLM exfiltrates them."

**Threat model.** GitGuardian 2026: 29M leaked secrets in 2025; 89.6% of LLM-agent skills with hardcoded credentials are exploitable. Common-Crawl-poisoning: real. This is a HIGH-confidence threat for AI-agent-rich Atriums.

**Does continuous rotation help?** YES — moderately. If K_Atrium rotates daily and the AI agent leaks its current K_Atrium into a training corpus, the leak window is bounded to 1 day of future content (continuous-rotation gives FS-against-LEAKED-PAST-KEY). Past content the agent already had is still readable to whoever extracts the leaked key — that's the forkable-semantic constraint.

**BUT — does Benten's data model actually admit "AI-agent-as-member"?** Re-checking CLAUDE.md #18 + M2 §6.4 + M-CONS BREAK 5: **NO.** `MemberKey` admits ONLY `UserDid` / `DeviceDid` / `LocalDevice`. **Plugin-DIDs + agent-DIDs are categorically not in `MemberKey`.** AI agents live in `CapabilityRecipient` / attribution-layer, NOT MembershipSet. So the threat is "AI-agent-with-delegated-capability leaks its capability-grant" not "AI-agent-as-member leaks K_Atrium." Capability-revocation (separate primitive in `benten-caps`) is the answer, not MembershipSet rotation.

**Verdict.** The AI-agent-compromise threat is REAL but lives at the **capability layer**, not the MembershipSet layer. CapabilityRevocation + short-lived caps are the v1-beta answer. **M5 verdict holds.** No continuous-rotation pressure.

**Watch-list mint candidate:** Compromise #55 — **AI-agent-capability-leak-via-training-corpus.** Names the threat + Benten's answer (short-lived caps + CapabilityRevocation). Doc-only mint.

### §3.5 Long-lived Atriums (>10 years)

**Use-case.** "An Atrium that exists for 20 years. Members come and go. Does FS accumulate value over multi-decade timescales?"

**Threat model.** Harvest-now-decrypt-later (HNDL): attackers harvest ciphertext today and decrypt when a future cryptographic break arrives (quantum, classical-cryptanalysis breakthrough, key-leakage-after-the-fact). Forward secrecy LIMITS the value of HNDL — past-session-keys are inaccessible even after a long-term-key compromise.

**Does continuous rotation help here?** PARTIALLY. Continuous rotation gives FS against future leakage of CURRENT K_Atrium — but PQ-resistant ciphers (Benten ships ML-KEM-768 + X25519 hybrid per Inv-15) already defend against the most concerning HNDL threat (post-quantum-era decryption of harvested classical-only ciphertext). The marginal value of continuous-rotation on top of PQ-hybrid is: bounded-window FS against silent-leakage-of-K_Atrium-from-a-current-member-device.

**Is this enough to justify continuous-rotation at v1-beta?** NOT for v1-beta. The PQ-hybrid story is the dominant HNDL defense. Continuous-rotation is a SECONDARY defense layer for the silent-current-member-leakage threat (Scenarios A-D below).

**Verdict.** Long-lived Atriums benefit from PQ-hybrid (shipping at v1-beta) MORE than from continuous-rotation. **M5 verdict holds.** Possible Phase-N revisit if Benten production data shows long-lived-Atrium silent-leakage incidents.

### §3.6 Cross-jurisdictional Atriums

**Use-case.** "Atrium with members in US + EU + China. Per-member compromise affects different legal-discovery scope; per-jurisdiction key rotation reduces blast radius."

**Threat model.** A US-jurisdiction subpoena can compel a US member to disclose their K_Atrium copy. Continuous rotation limits the disclosed-content window.

**Does continuous rotation help?** YES, but **Benten's forkable semantic also addresses this differently**: a member subpoenaed in the US can be FORK-EXCLUDED (admin-kick fork), making future content unavailable to that US member. Past content disclosure is unavoidable in any system that ever delivered the content to the US member (no crypto system defeats "we already gave them the content"). Continuous-rotation does NOT improve on this — it only limits the FUTURE-content disclosure, which fork-on-event already does.

**Verdict.** Fork-on-event = continuous-rotation for the cross-jurisdictional threat model. The cadence might be "rotate every $event" not "rotate every $time-period" — but the mechanism is the same. **M5 verdict holds.**

---

## §4 — Subtle attack scenarios

### §4.1 Scenario A — Carol's device silently compromised

**Setup.** Alice + Bob + Carol Atrium. Carol's device is silently compromised at t=0 (zero-day; Carol doesn't know). Attacker has K_Atrium. At t=+6mo, Bob detects anomaly + kicks Carol via AdminKickEpoch fork at t=+12mo (slow response).

**Damage with ForkOnly (M5 verdict).** Attacker reads 12 months of content (6mo pre-detection + 6mo detect-to-kick-response).

**Damage with continuous-rotation (e.g., daily).** Attacker reads ~24 hours of content per K_Atrium-leak — IF the attacker doesn't continuously re-exfiltrate from Carol's compromised device. But: **if Carol's device is silently and continuously compromised, the attacker exfiltrates each new K_Atrium as Carol receives it.** So continuous-rotation does NOT help against a continuously-active silent-compromise — it only helps against ONE-SHOT key leakage (Carol's K_Atrium leaks once but the compromise isn't persistent).

**Realism check.** One-shot key leakage IS realistic (key extracted from a single memory snapshot; a single side-channel; a key paste-into-a-chatbot). Persistent silent-compromise IS ALSO realistic (long-dwell APT).

**Verdict.** Continuous-rotation gives **bounded-window FS against ONE-SHOT key leakage from a current member's device.** Against persistent silent-compromise, it gives little. The realistic-threat scale:
- Persistent silent-compromise: median dwell time 11 days (Mandiant M-Trends 2025), but APT cases extend to 200+ days. For these cases, continuous-rotation gives marginal benefit.
- One-shot key leakage: this is the AI-agent-pastes-to-corpus scenario (§3.4), the side-channel-snapshot scenario, the disk-image-stolen-during-physical-access scenario. Continuous-rotation DOES limit blast radius here.

**Is this enough to justify continuous-rotation at v1-beta?** NOT for v1-beta given Benten's current threat model. The mitigations Benten ships at v1-beta:
- DAK + Layer-A vault encrypt K_Atrium at rest (so disk-image-stolen-during-physical-access doesn't expose K_Atrium without DAK).
- HSM-class device discipline (out of v1-beta scope).
- Capability-layer short-lived grants for AI agents.

Continuous rotation would be a defense-in-depth layer on top. **POST-v1-beta revisit candidate.** Compromise #54 names this.

### §4.2 Scenario B — 50-member Atrium, one compromise → K_Set exfil

**Setup.** 50-member Atrium. One member compromised. Attacker exfiltrates K_Atrium. Can now decrypt past content + future content until next rotation. Detection window: weeks to months.

**Damage with ForkOnly.** Attacker reads all post-exfil content until admin notices + initiates a fork. Detection window = exposure window.

**Damage with continuous-rotation (daily).** Attacker reads 1 day post-exfil; then needs to re-exfiltrate the new K_Atrium. If the compromise is persistent (still has device access), continuous rotation gives no benefit. If the compromise was one-shot (compromise-then-eject; e.g., USB-stick stolen, device returned), continuous-rotation gives substantial benefit.

**Verdict.** Same shape as Scenario A. **M5 verdict holds for v1-beta.**

### §4.3 Scenario C — Silent-undetected-compromise (never detected)

**Setup.** Member's key silently compromised. Compromise is NEVER detected. ForkOnly never triggers.

**Damage with ForkOnly.** Total. Forever.

**Damage with continuous-rotation.** If compromise is persistent: total, forever (attacker keeps re-exfiltrating). If compromise is one-shot: bounded to 1-rotation-window.

**Realism check.** Silent-undetected compromise is a real threat — IBM 2025 reports avg 207 days undetected lateral movement. But in Benten's local-first architecture, the threat surface is different: Benten devices aren't behind an enterprise EDR; they're personal devices. Detection probability is LOWER than the IBM baseline. Continuous-rotation is more valuable here than in a hardened-enterprise context.

**Verdict.** This is the strongest argument for continuous-rotation. **Still not enough to ship at v1-beta** because:
1. The forkable-by-design semantic + cap-layer short-lived grants already give application-driven defenses.
2. Continuous-rotation requires a CGKA primitive (TreeKEM or similar) for efficient log-N rekey distribution — at v1-beta Benten ships linear-N multi-stanza-HPKE which scales O(N) per rotation. Frequent rotations in large Atriums = expensive.
3. The Phase-4-Meta-Composing re-evaluation gate (per M5 §1.1) is the right place to revisit this when DCGKA / MLS-PQ are production-ready.

**Compromise #54 watch-list mint:** disclose this scenario + the deferral rationale.

### §4.4 Scenario D — AI agent leaks K_Atrium to training corpus

**Setup.** AI agent member with K_Atrium leaks K_Atrium to a training corpus. Continuous-rotation would have limited the leak window; explicit-rotation requires admin to detect "the AI just leaked it."

**Re-check Benten's data model.** As §3.4: AI agents are NOT members. They're capability-recipients. They don't have K_Atrium. They have time-bounded capability-grants that the cap layer can revoke.

**Verdict.** Scenario D is structurally inapplicable to Benten's MembershipSet design. The threat exists but lives at the capability layer. **M5 verdict holds.**

**Caveat.** If Benten ever ships "AI-agent-as-member" (M2 §6.4 BREAK 5 explicitly forbids this at v1-beta but a future design might admit it), Scenario D becomes live. The watch-list trigger is: "Benten changes MemberKey to admit non-human members → revisit AtriumWithCGKA AND continuous-rotation."

### §4.5 Scenario E (new) — Departing-admin-with-discovery-obligation

**Setup.** US Atrium admin departs the org under contentious circumstances. Court orders the admin to disclose all keys they had access to. AdminKickEpoch fork excludes them from future content. But the disclosed K_Atrium covers all PAST content the admin had access to.

**Does continuous-rotation help?** PARTIALLY — IF the past-content was encrypted with multiple K_Atrium generations (continuous-rotation-style), and the admin only discloses CURRENT K_Atrium, past-content remains protected by past K_Atrium that was already zeroized. BUT: in Benten's local-first model, the admin's local device probably has all past content already decrypted at rest. So continuous-rotation doesn't help unless past content is re-encrypted on rotation (which is the heavyweight "active re-encryption" pattern, NOT the lightweight "lazy re-encryption" Benten ships).

**Verdict.** Continuous-rotation marginally helps Scenario E only with active re-encryption — which is high-cost + low-value at v1-beta. **M5 verdict holds.**

### §4.6 Scenario F (new) — Quantum-era HNDL retrospective decryption

**Setup.** Attacker harvests Atrium ciphertext today. In year 2040, a CRQC arrives. Attacker decrypts the harvested ciphertext.

**Defense at v1-beta.** Benten ships ML-KEM-768 + X25519 hybrid (Inv-15). The PQ component defends against CRQC-era attacks. Continuous-rotation does NOT meaningfully improve HNDL defense beyond PQ-hybrid.

**Verdict.** PQ-hybrid is the right defense, not continuous-rotation. **M5 verdict holds.**

---

## §5 — Wire-format-stability analysis

### §5.1 Current v1-beta wire shape (per M2 §2.2)

```cbor
MembershipSet snapshot:
{
  "v":     1,                                ; u8 version (codepoint-additive)
  "kind":  0|1|2,                            ; EXACTLY-3 arms
  "mem":   [<MemberKey>...],                 ; canonical-sorted
  "pol":   {                                 ; MembershipSetPolicy
              "pv": <u32>,
              "kp": <KindPolicy>,
              "rr": <u64 | null>             ; refresh_required_secs — ALREADY ALLOCATED
           },
  "gen":   <u32>,                            ; K_Set.generation
  "cp":    <u16 LE>,                         ; cipher codepoint
  "par":   <MembershipSetId | null>,         ; parent for fork-tree
  "hlc":   "<...>",
  "att":   {<MembershipAttestation>}
}

MultiStanzaHpkeEnvelope:
{
  "v":      1,
  "set_id": <MembershipSetId>,
  "gen":    <u32>,
  "stanzas": [<recipient_did, kem_ct, aead_ct>...],
  "issuer_sig": <HybridSignature>
}
```

### §5.2 Continuous-rotation fits ADDITIVELY in this shape

**Mechanism for continuous-rotation in v1-beta wire shape:**
1. Mint new K_Atrium (CSPRNG) per rotation event.
2. Increment `gen` field (`u32` capacity = 4B rotations — adequate even at 1-rotation-per-second for 136 years).
3. Distribute new K_Atrium via new MultiStanzaHpkeEnvelope (O(N) stanzas).
4. New MembershipSet snapshot with same `mem`, same `pol`, new `gen`, same `par`, new `hlc`, new `att`.

**This is wire-format-additive.** No new codepoints needed. No new struct fields needed. The ONLY thing the `AtriumWithCGKA` codepoint reserve adds is a TreeKEM-style log-N efficient rotation distribution (replacing the O(N) multi-stanza envelope with a O(log N) TreeKEM ratchet tree). The semantic of "K_Atrium rotates frequently" is shippable on v1-beta wire WITHOUT the AtriumWithCGKA codepoint — only the EFFICIENCY of large-group rotation requires CGKA.

### §5.3 Open question: `refresh_required_secs` semantic

The M2 wire shape already carries `policy.refresh_required_secs: Option<u64>`. M2 §2.3 notes: "AtriumPolicy D3 reserved for Phase-4-Meta-Composing." This field is the natural hook for declaring continuous-rotation cadence policy in-spec.

**Question:** is `refresh_required_secs` interpreted as:
- (a) "membership-attestation must be re-signed every N seconds" (the original intent per AtriumPolicy D3), OR
- (b) "K_Atrium must rotate every N seconds" (continuous-rotation cadence)?

These are DIFFERENT semantics. The wire field doesn't distinguish. I recommend the v1-beta spec EXPLICITLY clarify (a) and reserve a SEPARATE policy field (or a richer `RefreshPolicy` enum) for (b) IF Benten ever adds continuous-rotation cadence. This is a doc clarification — NOT a wire-format change.

**This is the ONE residual wire-format concern.** A clarifying spec note at v1-beta + leaving the room for future rotation-cadence-field addition (either via codepoint or via additive field within `MembershipSetPolicy`) resolves it.

### §5.4 Verdict: deferring continuous-rotation is wire-format-additive

**YES — continuous-rotation can be added post-v1-beta WITHOUT a wire-format break.** Either:
- (a) Frequent-`gen++` continuous-rotation with O(N) multi-stanza envelopes — ships TODAY on v1-beta wire format, just by application choice to fork-frequently.
- (b) TreeKEM-style efficient continuous-rotation — ships via `MembershipSetKind::AtriumWithCGKA` additive codepoint (reserved at v1-beta).

Both paths are open. **M5's codepoint-reserve is sufficient.** Defer is safe. **M5 verdict holds.**

---

## §6 — Alternative architectural shapes

If continuous-rotation IS added later, what's the right shape?

### §6.1 Per-MembershipSet config

**Shape.** Each MembershipSet declares its rotation policy in `policy`. Some Atriums use continuous; most don't.

**Pros.** Application-controlled; matches the diverse-use-case landscape (HIPAA-compliant 90-day vs journalism daily vs casual indefinite).

**Cons.** Requires a richer policy enum at wire format. Manageable additively.

**Verdict.** **THIS IS THE RIGHT SHAPE.** Recommended for post-v1-beta if continuous-rotation is added.

### §6.2 Per-message option (sender chooses)

**Shape.** Sender of a message can opt into "rotate K_Atrium for this message + onward."

**Pros.** Sender-controlled; fine-grained.

**Cons.** Coordination nightmare in a multi-sender Atrium; conflicting per-message-rotation requests; per-message-rotation requires per-message-redistribution = O(messages × members) cost. Wire-format-heavyweight.

**Verdict.** REJECT. Not Benten-shaped.

### §6.3 Per-content-class config

**Shape.** Sensitive content gets continuous-rotation; regular content doesn't.

**Pros.** Cost-optimized for mixed-sensitivity content.

**Cons.** Requires a content-classification primitive Benten doesn't have. Application-layer concern, not MembershipSet.

**Verdict.** REJECT at MembershipSet layer. If needed, lives at Atrium-data layer as a separate primitive.

### §6.4 Recommendation

**If continuous-rotation is ever added: per-MembershipSet config via additive policy field + (optionally) `AtriumWithCGKA` codepoint for efficient log-N distribution.** The two are SEPARABLE:
- Continuous-rotation SEMANTIC is per-MembershipSet `policy.rotation_cadence_secs: Option<u64>` (additive).
- Continuous-rotation EFFICIENT-DISTRIBUTION is `AtriumWithCGKA` codepoint (reserved at v1-beta).

Application can opt into continuous-rotation SEMANTIC at v1-beta TODAY at O(N) rotation cost via frequent fork-events. Application gets log-N rotation cost AFTER `AtriumWithCGKA` ships post-v1-beta.

**This sharpens M5's "AtriumWithCGKA codepoint-reserve" rationale** — the codepoint reserve is for EFFICIENCY, not for SEMANTIC. The semantic ships at v1-beta on the existing wire shape. Worth surfacing in the M-CONS disclosure.

---

## §7 — Recommendation

**KEEP M5 + AdminKickEpoch at v1-beta.** With three sharpenings + one watch-list mint:

### §7.1 Sharpening 1 (doc-only): rotation-trigger taxonomy

Mint a "RotationTrigger" enum in the MembershipSet-Spec doc that enumerates:
- `AdminKick` — adversarial member kicked by admin
- `MemberDepart` — voluntary leave by member
- `AdminDepart` — admin with key-management access departs
- `DeviceRevoke` — DeviceMesh device revocation (e.g., stolen phone)
- `CompromiseResponse` — suspected key compromise; emergency fork
- `PeriodicHygiene` — scheduled cadence-driven fork (HIPAA 90-day / PCI annual)

All triggers map to the SAME mechanism at v1-beta: `MembershipSet::fork()` with a `MembershipEvent::Atrium...` audit event carrying the trigger reason. NO wire-format change. NO RotationPolicy enum change. Doc-only.

Cost: ~1 wave-day for spec doc + audit-event-variant naming.

### §7.2 Sharpening 2 (doc-only): clarify `refresh_required_secs` semantic

Add an explicit MembershipSet-Spec note: "`refresh_required_secs` denotes the cadence for membership-attestation re-signing per AtriumPolicy D3. It is NOT a K_Atrium rotation cadence. If/when continuous-rotation is added, it will use a SEPARATE policy field."

Cost: ~0.5 wave-day.

### §7.3 Sharpening 3 (doc-only): M-CONS §1.4 disclosure expansion

Extend the M5 §9.1 disclosure (Compromise #52 "MembershipSet-no-PCS-against-removed-members") to add:
- Compromise #54 (NEW): continuous-rotation deferral risk — names silent-undetected-CURRENT-member-compromise + one-shot-key-leakage as the canonical post-v1-beta CGKA trigger.
- Compromise #55 (NEW): per-subject crypto-shredding for GDPR right-to-be-forgotten — names this as a SEPARATE Atrium-data-layer design problem.
- Optional Compromise #56 (NEW): journalist-source-protection per-message FS — names this as a SEPARATE EphemeralRatchet-style design problem.

Cost: ~0.5 wave-day.

### §7.4 Watch-list mint (Compromise #54)

Codify the post-v1-beta CGKA-revisit trigger:

> **Trigger.** Any Benten production deployment that materially serves (a) journalism source-protection coordination, (b) AI-agent-rich multi-tenant Atriums where MemberKey ever admits agent-DIDs, (c) HIPAA / PCI-DSS / classified contexts requiring sub-90-day key rotation cadence, (d) production incidents involving silent-undetected-member-compromise.
>
> **Phase-N revisit.** Re-evaluate `MembershipSetKind::AtriumWithCGKA` codepoint adoption per M5 §1.1 (when MLS-PQ ciphersuites reach RFC + OpenMLS-PQ stable + DCGKA / p2panda-encryption audit-complete).

### §7.5 Net cost + verdict

**Total sharpening cost: ~2 wave-days, all doc-only.** No code changes. No wire-format changes. No M2 / M-CONS overturning.

**Verdict: M5 conclusion is RIGHT for v1-beta.** The sharpenings strengthen the disclosure surface + foreclose the "we never thought about X" future-debate. The watch-list mint codifies the revisit trigger so Phase-N has clear criteria.

---

## §8 — Self-assessment + confidence

### §8.1 What I'm confident about (HIGH)

- **M5's "no continuous-rotation at v1-beta" verdict holds.** No regulatory regime mandates it; Benten's forkable-by-design semantic structurally accommodates regulatory rotation triggers via fork-on-event; the most-cited threat (silent-undetected-compromise) is real but narrow + deferrable.
- **Wire format is additive.** Continuous-rotation can be added post-v1-beta either via O(N) frequent-fork-events on the existing wire shape OR via `AtriumWithCGKA` codepoint additive expansion. No v1-beta wire break needed.
- **Recommendation cost is low** (~2 wave-days, all doc-only).
- **GDPR right-to-be-forgotten is a SEPARATE design problem** (crypto-shredding at Atrium-data layer, not MembershipSet rotation).

### §8.2 What I'm less confident about (MEDIUM)

- **Whether the rotation-trigger-taxonomy doc sharpening is worth the v1-beta doc spend** vs deferring to Phase-N. I lean YES at v1-beta because it forecloses the "M2 RotationPolicy is 2-arm; what does the rich trigger taxonomy look like?" future-debate. But it's a defensible defer.
- **Whether `refresh_required_secs` semantic clarification is enough or whether the field should be RENAMED to `attestation_refresh_secs` to be unambiguous.** I lean toward rename for clarity, but rename = wire-format breaking change if anything has already written this field to disk. At v1-beta-pre-freeze the rename is cheap; post-freeze it's expensive. Rename now if M2 hasn't shipped yet.
- **Whether journalist-source-protection's per-message FS deserves its own Compromise mint vs being a sub-clause of #54.** I lean toward separate mint (#56) because it's a DIFFERENT design problem (per-message FS, not membership rotation). Defensible to defer.

### §8.3 What I'm UNcertain about (LOW)

- **Whether DCGKA / p2panda-encryption is actually production-ready post-Radically-Open-Security-audit completion.** M5 §2.6 flagged this as audit-pending Feb 2025. I did not independently verify audit completion in May 2026. If the audit completed favorably and p2panda-encryption is now production-grade Rust, the calculus on "WHEN to revisit AtriumWithCGKA codepoint" shifts forward. This is a Phase-N revisit detail, not a v1-beta concern.
- **Whether Benten will ever target classified / SAP customers.** If yes, continuous-rotation gets more weight. If no, M5's deferral is permanently right. I lean NO at the v1-beta product-positioning level but don't have authoritative product input.

### §8.4 Self-critique: am I missing a use-case?

Three candidate use-cases I considered + rejected (briefly):

1. **Subpoena-defensive auto-rotation in adversarial-jurisdiction contexts.** Already covered in §3.6 cross-jurisdictional analysis.
2. **Time-bound legal-hold encryption** (e.g., "encrypt this data for exactly 7 years then auto-shred"). This is GDPR-adjacent + lives at Atrium-data-layer crypto-shredding, not MembershipSet rotation. Out of scope.
3. **Multi-Atrium federation key rotation** (M1c §10.3 cross-Atrium federation site). Federation uses MembershipSet as substrate (each Atrium is a MembershipSet; federation = a higher-order MembershipSet?). Federation key rotation = re-key the federation envelope, which is fork-on-event at the federation MembershipSet layer. Same shape as Atrium. **M5 verdict holds.**

I did NOT find a Benten-relevant use-case where M5's conclusion is wrong.

### §8.5 Confidence breakdown

- §1 Executive verdict (M5 HOLDS + sharpenings): **HIGH (88%)**
- §2 Regulatory analysis (no continuous-rotation mandate): **HIGH (92%)**
- §3 Benten use-cases (no v1-beta trigger): **HIGH (85%)**
- §4 Attack scenarios (deferral acceptable): **MEDIUM-HIGH (78%)** — Scenario C is the closest to a sticking point
- §5 Wire-format additive: **HIGH (90%)**
- §6 Alternative shapes (per-MembershipSet config): **MEDIUM-HIGH (80%)**
- §7 Recommendation (keep M5 + 3 sharpenings + 1 watch-list mint): **HIGH (85%)**

**Overall: HIGH confidence that M5's v1-beta verdict is right.** Sharpenings are HIGH-confidence value-add at LOW cost.

---

## §9 — Citations

### §9.1 Internal artifacts (SHA-pinned)

- M2 primitive design: `phase-4-meta-core/membership-set-m2 @ 6170980b` → `.addl/phase-4-meta/membership-set-m2-primitive-design.md`
- M5 CGKA candidate survey: `phase-4-meta-core/membership-set-m5 @ 15819500` → `.addl/phase-4-meta/membership-set-m5-cgka-candidate-survey.md`
- M-CONS consolidator: `phase-4-meta-core @ 74580ee6` → `.addl/phase-4-meta/membership-set-m-cons-consolidator.md`
- M1c key-management cataloger: `phase-4-meta @ 50eb901d` → `.addl/phase-4-meta/membership-set-m1c-...`
- Ben's 2026-05-27 forkable-Atrium ratification: HEAD-pinned via M-CONS §1.1
- CLAUDE.md #18 (4-identity-concepts asymmetry), #19 (plugin/extension trust model)

### §9.2 External (WebSearch results, retrieved 2026-05-28)

**HIPAA key rotation:**
- [Best Practices for Key Rotation in Healthcare Clouds — Censinet](https://censinet.com/perspectives/best-practices-key-rotation-healthcare-clouds)
- [HIPAA Key Management Requirements 2025 — Censinet](https://www.censinet.com/perspectives/hipaa-key-management-requirements-2025)
- [Encryption Key Rotation Best Practices — Kiteworks](https://www.kiteworks.com/regulatory-compliance/encryption-key-rotation-strategies/)

**PCI-DSS:**
- [PCI DSS Key Rotation Requirements — PCI DSS GUIDE](https://pcidssguide.com/pci-dss-key-rotation-requirements/)
- [PCI Requirement 3.6.4 Cryptoperiod — KirkpatrickPrice](https://kirkpatrickprice.com/video/pci-requirement-3-6-4-cryptographic-key-changes-cryptoperiod-completion/)
- [Automated Key Rotation for PCI-DSS 4.0 — Raidiam](https://www.raidiam.com/developers/blog/key-rotation-for-pci-dss)

**GDPR crypto-shredding:**
- [GDPR Right to be Forgotten — ComplyDog](https://complydog.com/blog/right-to-be-forgotten-gdpr-erasure-rights-guide)
- [Crypto shredding — Brent Robinson @ Medium](https://medium.com/@brentrobinson5/crypto-shredding-how-it-can-solve-modern-data-retention-challenges-da874b01745b)
- [GDPR Right of Erasure & Encryption Key Management — Townsend Security](https://info.townsendsecurity.com/gdpr-right-erasure-encryption-key-management)

**MLS / continuous group key rotation:**
- [RFC 9420 The Messaging Layer Security Protocol](https://datatracker.ietf.org/doc/html/rfc9420)
- [Messaging Layer Security overview — phnx.im blog](https://blog.phnx.im/rfc-9420-mls/)
- [Investigation into MLS — dr-knz.net](https://dr-knz.net/mls-investigation.html)

**SecureDrop / journalism source-protection:**
- [SecureDrop Threat Model](https://docs.securedrop.org/en/stable/threat_model/threat_model.html)
- [Introducing SecureDrop Protocol](https://securedrop.org/news/introducing-securedrop-protocol/)

**Dwell time / silent compromise:**
- [2025 Unit 42 Global Incident Response Report — Palo Alto Networks](https://www.paloaltonetworks.com/resources/research/unit-42-incident-response-report-2025)
- [The Collapse of Dwell Time — National CIO Review](https://nationalcioreview.com/articles-insights/extra-bytes/the-collapse-of-dwell-time-cybersecuritys-shrinking-detection-window/)

**Post-compromise security (CGKA literature):**
- [On Post-Compromise Security — Cohn-Gordon, Cremers, Garratt (2016)](https://eprint.iacr.org/2016/221.pdf)
- [ER-CGKA: Efficient and robust CGKA — NCBI/PMC](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11361688/)

**AI agent credential leakage:**
- [29 million leaked secrets in 2025 — Help Net Security](https://www.helpnetsecurity.com/2026/04/14/gitguardian-ai-agents-credentials-leak/)
- [Credential Leakage in LLM Agent Skills — arXiv](https://arxiv.org/html/2604.03070v1)
- [AI Agent Data Leakage — Rafter](https://rafter.so/blog/ai-agent-data-leakage-secrets-management)

**Harvest-now-decrypt-later:**
- [HNDL Quantum-Era Threat — Palo Alto Networks](https://www.paloaltonetworks.com/cyberpedia/harvest-now-decrypt-later-hndl)
- [On the Practical Feasibility of HNDL Attacks — arXiv 2603.01091](https://arxiv.org/pdf/2603.01091)

---

## §10 — Summary

**N3 verdict: M5's "no continuous-rotation at v1-beta + AtriumWithCGKA codepoint-reserve" conclusion HOLDS.** Stress-tested against 6 regulatory frameworks (HIPAA / PCI-DSS / GDPR / SOX / classified / cross-jurisdictional), 6 Benten-specific use-cases (time-bound sharing / journalism / compliance-hygiene / AI-agent / long-lived / cross-jurisdictional), and 6 subtle attack scenarios (Carol silent-compromise / 50-member exfil / silent-never-detected / AI-agent-leak / departing-admin-discovery / quantum-HNDL). **No use-case overturns M5's verdict.**

The wire format is additive: continuous-rotation can ship post-v1-beta via frequent-fork-events (O(N), no codepoint) OR via `AtriumWithCGKA` codepoint (TreeKEM log-N, additive). M5's codepoint-reserve is sufficient.

**Recommended sharpenings (~2 wave-days, doc-only):**
1. Mint a `RotationTrigger` enum in MembershipSet-Spec (AdminKick / MemberDepart / AdminDepart / DeviceRevoke / CompromiseResponse / PeriodicHygiene).
2. Clarify `policy.refresh_required_secs` semantic (attestation re-sign, NOT K_Set rotation cadence); rename to `attestation_refresh_secs` if M2 hasn't frozen yet.
3. Expand M-CONS §1.4 disclosure to mint Compromise #54 (continuous-rotation deferral risk), Compromise #55 (GDPR crypto-shredding deferral), and optional Compromise #56 (journalist-source-protection per-message FS deferral).

**Confidence: HIGH (85%)** on the verdict; **HIGH (90%)** on wire-format additivity.

— end —
