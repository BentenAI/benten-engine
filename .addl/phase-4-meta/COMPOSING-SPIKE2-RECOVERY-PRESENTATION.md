# Benten Identity/Key Recovery — Survey Synthesis for Ben

*13 reviewers (7 stakeholder personas + 6 expert lenses) against 8 options + the frozen v1-beta substrate. Read top-to-bottom; punchline first.*

---

## ★ PUNCHLINE (read this, skip the rest if you must)

1. **One option won cleanly: Social recovery (UCAN guardians).** 7 of 13 reviewers ranked it #1; top-3 for 10 of 13. Even the skeptics who *rank* it lower (architect, ops) independently name it "the thing to fill the seam with." It's the only option that is simultaneously substrate-native, custodian-free, holds-no-stealable-secret, solves lost-all *authority*, and is comprehensible to a non-technical user.

2. **The reviewers converged on the SAME wrapper regardless of which primitive they preferred.** Timelock + independent-channel broadcast + veto + duress/decoy path was proposed *independently* by 11 of 13 reviewers — and the red-team confirmed it from the other side ("loud, slow, cancelable guts my playbook"). **This envelope, not the choice of primitive, is the real product.**

3. **The menu is the wrong framing — the winning answer is a *shape*, not an option.** The single most decision-useful engineering insight (systems-architect): at v1-beta you don't pick a protocol, you land **one irreversible thing — an extensible `RotationAuthKind` codepoint** — and defer every reversible choice. Freeze the seam, defer the fill.

4. **The irreducible tension is real and it has a name: authority-recovery ≠ confidentiality-recovery.** Rotating to a new key restores the right to *act*; it returns *nothing* encrypted to the dead KEM key. "Just recover my old data too" forces KEM-seed escrow = a permanent harvest-now-decrypt-later hole. This is where your sharpest stakeholder conflict lives (grandma wants her photos back; the activist needs her source archive to be truly gone), and — critically — **it cannot be deferred** the way authority-recovery can, because escrow must predate the data.

5. **My recommendation:** ship the **extensible codepoint + a loud NoRecovery default + social-guardian as the first reference authorizer, all wrapped in the timelock/veto/broadcast/duress envelope, with authority and confidentiality split in the type system.** Prototype **two** finalists: (a) social-guardian rotation, (b) seed-phrase/deterministic-keygen-from-genesis as the opt-in sovereign floor. Run **one focused spike** on the confidentiality tier (forward-secure DAK ratcheting), because that's the un-deferrable fork.

---

## 1. AGGREGATE SIGNAL

### Rank matrix (1 = each reviewer's best-for-them; lower avg = broader support)

Reviewer codes — **Stakeholders:** ACT=activist/journalist, GMA=non-technical/forgetful, PWR=power-user, PUR=sovereignty-purist, DUR=coercion/duress-survivor, BIZ=small-business, ACC=accessibility. **Lenses:** CRY=cryptographer, RED=red-team*, ARC=systems-architect, UXS=UX-under-stress, DEC=decentralization-economist, OPS=ops/support.

| Option | ACT | GMA | PWR | PUR | DUR | BIZ | ACC | CRY | RED* | ARC | UXS | DEC | OPS | **Avg** | **Verdict** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **2 Social (guardians)** | 1 | 1 | 6 | 4 | 1 | 1 | 1 | 1 | 5 | 2 | 2 | 3 | 1 | **2.2** | 🥇 consensus winner |
| **1 Shamir (SSS)** | 4 | 4 | 1 | 2 | 3 | 2 | 4 | 2 | 3 | 5 | 6 | 1 | 3 | **3.1** | 🥈 self-hoster's pick |
| **6 Threshold sig (FROST)** | 3 | 7 | 3 | 3 | 2 | 3 | 5 | 4 | 2 | 7 | 7 | 5 | 5 | **4.3** | cleanest crypto, PQ-blocked |
| **8 No recovery** | 2 | 8 | 4 | 1 | 4 | 8 | 6 | 3 | 1 | 1 | 8 | 4 | 6 | **4.3** | ⚡ most polarizing |
| **4 MLS device groups** | 5 | 5 | 2 | 6 | 5 | 6 | 3 | 6 | 4 | 8 | 4 | 7 | 2 | **4.9** | continuity, not recovery |
| **5 Passkey / FIDO2** | 8 | 2 | 8 | 8 | 7 | 5 | 2 | 8 | 7 | 4 | 1 | 8 | 4 | **5.5** | ⚡ custodial split |
| **7 Seed phrase / paper** | 7 | 6 | 5 | 5 | 8 | 7 | 8 | 5 | 8 | 3 | 3 | 2 | 7 | **5.7** | sovereign floor / coercion trap |
| **3 Hardware escrow** | 6 | 3 | 7 | 7 | 6 | 4 | 7 | 7 | 6 | 6 | 5 | 6 | 8 | **6.0** | 🥉-from-bottom, no champion |

*\*RED (red-team) is **inverted** — the attacker ranked options by how fast *they* win, so I flipped it to "defensive strength." Their actual message: they'd *lobby* for passkey-default and seed-default (the two they can steal silently), and they want No-recovery *removed* from the menu because it starves them.*

### Axis ratings (averaged 1–5 across reviewers who rated each in their top-2)

| Option | Security | Usability | Self-sovereignty | Coercion-resist | Signature shape |
|---|---|---|---|---|---|
| **2 Social** | 3.9 | 3.5 | 3.9 | 3.2 | **balanced, no zeros** — the all-rounder |
| **1 Shamir** | 4.2 | 3.4 | **5.0** | 2.2 | max sovereignty, weak coercion |
| **8 No recovery** | **5.0** | 3–5 | **5.0** | **5.0** | maxed everything *except recoverability = 0* |
| **5 Passkey** | 3.0 | **5.0** | 2.0 | 1.0 | max UX, min sovereignty+coercion |
| **4 MLS** | 4.0 | 5.0 | 4.5 | 1.0 | great continuity, coercion-catastrophic |
| **6 FROST** | 4.0 | 2.0 | 4.0 | 4.0 | strong but ceremony-heavy |
| **7 Seed** | 3.0 | 5.0 | 5.0 | 1.0 | maximal sovereignty, worst coercion |

### Who won, who lost, and why

- **Social won** because it is the only option with **no zero on any axis** and it satisfies the widest set of hard requirements at once: recovers lost-all authority, holds no stealable/rot-able secret, needs no custodian, has the best mental model ("my people vouch for me" — GMA, UXS, OPS, ACC all say this independently), and composes cleanly with the timelock/veto envelope everyone wants. Its only real weaknesses — the K-collusion equivocation floor and guardian-set composition — are shared by *every* threshold scheme and are partially defended by the wrapper.
- **Shamir placed second** as the pick of the self-reliant (PWR #1, DEC #1, PUR #2, CRY #2): information-theoretic secrecy, custodian-free, best availability math. It loses on coercion (no duress signal), silent share rot, and opaque UX (UXS #6, ARC #5 — "same authority outcome as social, more crypto for us to build/audit/refresh").
- **Hardware escrow lost outright** — the only option with **zero champions** (best rank anywhere is GMA's #3, and she immediately says she'll forget the PIN). Coercion-catastrophic (one object + one PIN + one person), no 2026 secure element signs ML-DSA (PQ secret transits host RAM anyway), and forgotten-PIN = irreversible brick. OPS ranks it dead last: "false hope then irreversible brick, the maximally painful minimally-resolvable ticket."
- **The two polarizers (⚡)** are the real story of the aggregate: **No-recovery** and **Passkey** have the highest variance. No-recovery is #1 for the purist/architect/red-team and #8 for grandma/business/ux-stress. Passkey is #1 for UX-under-stress and #2 for grandma/accessibility, but #8 for five reviewers on sovereignty grounds. **Averages lie here — these two are not "middling," they are civil wars.** (See §2.)

---

## 2. CONSENSUS vs SHARP DISAGREEMENT

### Strongest shared signal (near-unanimous)

1. **The wrapper beats the primitive.** 11 of 13 independently proposed: **mandatory timelock + broadcast notification to independent channels + veto + duress/decoy.** The red-team's confession seals it: *"Any design where recovery is loud, slow, and cancelable guts my playbook… speed and silence are 80% of my edge."* This is the highest-confidence finding in the whole survey.
2. **No mandatory custodian.** Near-universal (only the red-team wants one). Passkey is rejected by 5 reviewers *specifically* for laundering Apple/Google custody as self-sovereign ("a custodian wearing a biometric costume" — CRY; "the centralization I exist to fight" — DEC).
3. **Authority-recovery ≠ confidentiality-recovery, and naive KEM-seed escrow is a trap.** CRY, ARC, ACT, BIZ, PWR all flag it. Consensus: **separate them**, make confidentiality-escrow opt-in and tiered, never the silent default. CRY's blunt version: *"I would rather ship no confidentiality recovery at all than ship a naive KEM-seed escrow."*
4. **Operational rot, not cryptanalysis, is the real failure mode.** UXS, OPS, DEC, GMA converge: the loss event is a guardian who moved/died, a forgotten PIN, a share that silently rotted — discovered at the worst possible moment. Implies **active monitoring / fire-drills / over-provisioning** as first-class features.
5. **No single option is the sole answer — the pluggable hook holds a *factor*.** The reference predicted this; every reviewer confirmed it.

### Where one stakeholder's need directly CONFLICTS with another's (this defines the design space)

- **⚔️ "Nothing to coerce" vs "bring back my photos."** THE central conflict. **PUR/ACT/DUR** need truthful "I have no recovery" deniability and data that is *genuinely gone* after a seizure (the activist's whole threat model is not becoming the instrument of her sources' exposure). **GMA/BIZ** need the *archive* back or the product has failed them ("You lost a decade of your grandchildren's photos and by design no one can help" — GMA's rage; "bankruptcy with a philosophy attached" — BIZ on No-recovery). **These are the same dial pulled in opposite directions** — and satisfying grandma (escrow the KEM seed) builds exactly the coercion/HNDL surface the activist and cryptographer forbid.

- **⚔️ Guardians = family?** **GMA** *wants* family ("my kids can vouch for me" — the one thing she understands). **DUR** says family is *inside* the threat model: *"In coercive control the family is inside the threat model — he has spent years mapping and controlling exactly those people."* A default that assumes "trusted contacts are safe" is designed for someone else's life. Directly contradictory defaults for guardian composition.

- **⚔️ Compelled biometric: comfort or catastrophe?** **GMA/ACC/UXS** rate biometric the single most accessible, frictionless primitive (a face tap is "the easiest thing in the world"). **ACT/DUR/PUR/CRY** say a compelled biometric has *weaker* legal protection than a memorized passphrase and is trivial under physical force ("he holds the phone to my face — that's not a movie scene, it's Tuesday" — DUR). Same feature, opposite valence.

- **⚔️ Humans vs infrastructure.** **PWR** wants a "quorum of me" (own servers across two countries, no friends to nag). Most others assume human guardians. **DEC** adds the equity wrinkle: many users are *guardian-poor* — "find five reliable friends or accept loss" is a false assumption for a huge slice of users.

- **⚔️ MLS: essential or anti-pattern?** **OPS/PWR/ACC** love it (self-services 90%+ of tickets, the everyday "approve new device" flow). **ARC** calls it an outright deal-breaker: a *parallel* device-identity protocol that needs an Authentication Service Benten structurally lacks (TOFU/no-PKI) and *doesn't even solve lost-all* — "two sources of truth for device identity is an architectural anti-pattern I won't introduce for a problem it doesn't solve."

**Design consequence:** because these are genuine value conflicts, not confusions, the answer *must* be per-user-configurable (tiers + pluggable legs) rather than a single decreed default. The conflicts are the argument for the hook.

---

## 3. NOVEL / HYBRID IDEAS THAT BEAT THE MENU

*The highest-value output. Surfaced prominently with attribution.*

### 🌟 A. Tiered / compartmentalized confidentiality — *dissolve the central conflict*
**Raised independently by four reviewers** (ACT, ACC, CRY, BIZ), which is why it's first. Split the identity into tiers:
- **Everyday tier** (authority + ~95% of the graph): recoverable, KEM lineage carried in recovery → grandma gets her photos.
- **Crown-jewel / source tier**: KEM keys device-only, forward-secret, **escrowed nowhere** → the activist can truthfully tell an interrogator "the source material is unrecoverable, no ceremony can bring it back." 

This turns the universal *"authority recovery ≠ data recovery"* **weakness into a deliberate feature**: the same product serves grandma *and* the activist by letting recoverability be a per-tier choice instead of one global dial. **This is the single most important idea in the survey** — it's the only thing that resolves the ⚔️ central conflict.

### 🌟 B. The "Living-Owner Veto Key" — a credential whose only job is to say *no* (PUR)
At enrollment, mint a tiny memorable secret whose **only** capability is to cancel a *pending* rotation during the timelock. It can't spend, rotate, or read. **This makes it coercion-inert:** torture it out of the user and the attacker gains only the power to cancel their *own* attack. Converts the irreducible **indistinguishable-seizure** problem into a **contestable** one. Genuinely novel — nobody ships this. *"The best part is the part nobody put on the list: a key whose only job is to say no."*

### 🌟 C. "Freeze the seam, defer the fill" — extensible `RotationAuthKind` codepoint (ARC)
The one irreversible thing to land in v1-beta: make RotationLog's authorization field **codepoint-dispatched with a typed-reject arm** (mirroring your existing crypto-agility envelope), reserving a recovery-authorizer codepoint range. Then social / threshold / hardware / passkey-B are all **additive post-freeze**. Without it, "pluggable RecoveryHook" is a marketing line — adding social recovery later would need a wire change to the frozen interface. **This is the concrete answer to your frozen-substrate constraint** and it's cheap. (Pairs with CRY's convergent "heterogeneous M-of-N over one universal 'authorized rotation' sink.")

### 🌟 D. Forward-secure DAK ratcheting to bound HNDL blast radius (CRY)
Encrypt data to a slowly-ratcheting Data-At-rest Key, not the long-term ML-KEM key. Escrow **only the current DAK epoch**, never the long-term secret — and **enforce that in the type system** so permanent-harvest escrow is *structurally impossible*, not merely discouraged. Compromise of a past escrow can't decrypt future data. This is what makes tier A (idea A) safe to build.

### 🌟 E. Deniable-decoy identity + covert duress channel (DUR)
Two genuine DIDs in a hidden-volume shape; the *existence* of the real one is not provable. Under coercion the user recovers a fully-working decoy while silently flagging guardians "proceed only to the decoy." **Crucially distinguished from "decoy theater"** (which CRY rightly rejects — a detectable decoy is worse than none): the property that matters is *true deniability*, no artifact reveals the second identity.

### 🌟 F. Operator / Authorizer role separation (ACC)
The hands physically driving the recovery UI (a caregiver, an assistant) must be **structurally barred from being an authorizer**. Lets a helper *help* without being able to *become* you — the gap the entire menu misses for anyone who depends on others. Additive: it's a policy on who-may-sign vs who-may-drive.

### 🌟 G. Refresh-as-heartbeat + annual Fire Drill + Recovery Health traffic-light (DEC + UXS + OPS)
Three reviewers, one convergent mechanism: proactive share/guardian refresh **doubles as a liveness probe**; an annual low-stakes rehearsal surfaces rot *while the user is calm* and builds muscle memory; a persistent health status + **over-provisioning by default** (3-of-5 tolerating 2 dead, not 2-of-3 on a knife's edge). **Converts silent decay → monitored, repairable decay** — directly attacking the #1 real-world failure. UXS: *"the difference between handing someone a fire extinguisher and running an actual fire drill."*

### 🌟 H. Support Firewall — cryptographically exclude Benten support, no issuer-override (OPS)
Make it *structurally impossible* for Benten's own support desk to recover/reset anyone (hold no share, no escrow, no override). Turns the oldest attack ("hi, I'm locked out, reset me") from *hard* into *impossible*: you can't socially-engineer out of support what support provably doesn't have. An operational invariant, not crypto.

### 🌟 I. Permissionless below-threshold custodian market + per-trust-domain cap invariant (DEC)
For the guardian-poor: a P2P market of custodians paid a tiny recurring fee to hold one *inert below-K* share, **payment liveness-gated** (no heartbeat → no fee → auto-re-provision, self-cleaning). Plus a hard protocol invariant: **no single trust-domain may hold ≥ K shares** — the Argent/Ledger re-centralization lesson encoded as a rule the enrollment *refuses to violate*, not advice users can ignore. Solves the equity gap without reintroducing a custodian.

### 🌟 J. Jurisdictional K-of-N + hidden guardian-set commitment (ACT, echoed by DUR + RED)
Client warns if the quorum is satisfiable within one legal regime; the guardian set is a *hiding* commitment so an adversary reading the public RotationLog **cannot enumerate whom to coerce** (this protects the guardians' safety, not just the user's data). Three reviewers demanded guardian-set privacy independently.

### 🌟 K. Two-tier quorum + succession path (BIZ)
Separate "resume trading" (low K, short timelock, rotate authority fast) from "open the vault" (higher K, must include an institutional cold share, long timelock, decrypt the archive) — plus a named-successor/dead-man's path gated on an incapacity attestation. Businesses need continuity semantics pure self-sovereignty pretends they don't.

---

## 4. HARD CONSTRAINTS — and which options survive

| Constraint | Survivors | Killed by this constraint |
|---|---|---|
| **No mandatory custodian** (company-can-reset forbidden by the platform's founding premise) | Social, Shamir, FROST, Seed, No-recovery, (Hardware self-custodied variant) | **Passkey** (reduces to Apple/Google account recovery), custodial-HSM hardware escrow |
| **Lost-ALL-devices survivability** | Social, Shamir, FROST-*if*-off-device-holders, Seed, Hardware | **MLS** (group state dies with devices), **No-recovery** (by design), Passkey (only via custodian) |
| **Coercion / duress resistance** | No-recovery (best), Social+envelope, FROST (marginally) | **Seed** (worst-in-class $5-wrench), **Hardware** (one PIN, one person), **Passkey** (compelled biometric), **MLS** (coerce any live member) |
| **Frozen-substrate-additive** (no wire break) | Social (RotationLog-native), Shamir, Seed (genesis-time), Passkey-A, No-recovery | **MLS** (parallel device-identity protocol + needs an AS Benten lacks), **FROST** as PQ authority (threshold ML-DSA doesn't exist → classical-only weak link) |

**Options that survive ALL four hard constraints: Social recovery (with the envelope) and Shamir.** No-recovery survives three but *by design* fails lost-all recoverability (that's its thesis, not a bug). Everything else is knocked out on at least one hard constraint as a *sole default* — though several remain valid as **one leg of an M-of-N** (passkey as a convenience factor, hardware as a redundancy share, seed as the cold floor, FROST as a future additive codepoint once PQ-threshold matures).

**Two irreducible truths every reviewer had to accept (name these to Ben honestly):**
- **The K-collusion / equivocation floor is a law of a no-authority world, not a patchable bug.** In a no-consensus system a ≥K malicious rotation is *cryptographically indistinguishable* from a legitimate one. Best achievable defense = make it *loud, timelocked, and vetoable* (convert silent → contestable), not eliminate it.
- **Perfect self-sovereignty and perfect recoverability are in genuine tension.** Every gram of recoverability you add is a gram of coercion/custody surface. The tiered-confidentiality idea (§3-A) doesn't break the tradeoff — it lets the *user* place the dial per-tier.

---

## 5. RECOMMENDED NEXT STEP

**Don't pick a recovery protocol. Ratify a `RecoveryHook` shape, land the one irreversible piece, and prototype two finalists.** This is where the survey actually converges, across stakeholders *and* lenses:

### The shape to ratify (do-it-now, additive on the frozen substrate)

1. **v1-beta MUST land (the only irreversible thing): the extensible `RotationAuthKind` codepoint** — self-describing, typed-reject on unknown, reserving a recovery-authorizer range (§3-C). *Everything else becomes a post-freeze additive drop-in.* This is the item that has a real deadline against your interface freeze; miss it and the hook is decorative.

2. **Default hook = `NoRecoveryHook`, but LOUD.** Unconfigured = the honest, coercion-maximal floor (satisfies PUR/ARC/RED), **but** onboarding treats single-device / no-enrolled-authorizer as an *unhealthy, flagged* state and funnels the user to configure the reference authorizer. This threads the ⚔️ No-recovery civil war: ARC and OPS both land here independently — the floor is a *choice you're nudged out of*, not a surprise you discover at 2am.

3. **First reference authorizer to build: social-guardian rotation (`GuardianQuorumAttested`).** The consensus winner; substrate-native (K attestations → one RotationLog rotation); **carries no threshold-ML-DSA dependency** (guardians sign with real hybrid did:benten keys — N concatenated PQ sigs is verbose but honest, no classical-only hole).

4. **Wrap every authorizer in the convergent envelope (the near-unanimous ask):** mandatory **timelock** + **multi-channel broadcast to independent channels** (guardians + peers, not only the user's own devices, which an attacker compromises first) + **veto** (ideally the Living-Owner Veto Key, §3-B) + **duress/decoy** path. This is the highest-confidence finding and the red-team's confessed nemesis — build it even if you overrule everything else.

5. **Split authority-recovery from confidentiality-recovery in the type system.** Default = authority-only. Confidentiality = separate, opt-in, **tiered per-namespace**, escrowing **only forward-secure DAK epochs, never the long-term KEM secret** (§3-A + §3-D). This is un-deferrable in a way authority-recovery is not.

6. **Ship Recovery Health monitoring + over-provisioning + fire-drill (§3-G) and the Support Firewall (§3-H)** as first-class — they attack the dominant *real* failure (operational rot) and the dominant *real* attack (support social-engineering), both of which the crypto choice alone doesn't touch.

### Two finalists to prototype

- **Finalist 1 — Social-guardian rotation + full envelope.** The default reference authorizer. Prototype to pressure-test: guardian-set privacy (hiding commitment, §3-J), the Operator/Authorizer role split (§3-F), timelock/veto ergonomics under stress, and the duress-decoy.
- **Finalist 2 — Seed-phrase / deterministic-keygen from genesis, as the opt-in sovereign floor and one M-of-N leg.** Cheapest hook, cleanest PQ fit (back up 32 bytes, re-expand via FIPS 203/204), and the *only* option that survives Benten's own death. **Decide this now, not later** — it's a genesis-time keygen commitment, not a deferrable seam-fill (ARC's trap warning). Prototyping it also de-risks the M-of-N heterogeneous-leg model (§3-C).

### The one spike I'd run

**A focused spike on the confidentiality tier: forward-secure DAK ratcheting + per-namespace tiered escrow (§3-A/D).** This is the genuine unresolved fork — it's where your sharpest stakeholder conflict lives (grandma's photos vs the activist's deniable-gone archive) *and* it's the piece you cannot defer past the interface freeze (KEM escrow must predate the data). Authority-recovery can wait behind the codepoint; confidentiality-recovery accrues an unrecoverable-ciphertext window every day you don't decide.

### The honest tradeoff to put in front of Ben

You are not choosing between secure and recoverable — you are choosing **where each user gets to place the dial**, and accepting two things no protocol can erase: recovery in a no-authority world is always *contestable-at-best, never-provably-legitimate* (the equivocation floor), and every path back in is a path an adversary can walk too. The reviewers' unanimous verdict is that the right response is not a stronger vault but a **loud, slow, vetoable, tiered** one — and that the product's job is to make the sovereign choice the *easy* one, so it wins the convenience gradient that re-centralized Argent and every scheme before it.