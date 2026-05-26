# Shape 5 iroh-outreach prep package

> Drafted 2026-05-26 by Shape 5 iroh-outreach prep agent.
> Input package: `critic-lens-l3-iroh-maintainer.json` (L3 4-concession soft-VETO) + `r2-critic-15-synthesis-matrix.md` (cross-lens evidence) + `position-b-revision-roadmap.md` (blog v2 framing instructions) + `comment-opportunities-content-addressed-systems.md` (3 iroh-cluster threads flagged WATCH-ONLY-PENDING-SHAPE-5) + `NIGHT-SHIFT-2026-05-26.md` (current state + HOLD list).
> Status: PREP ONLY. Ben sends the email; orchestrator decides timing relative to Position B blog v2 draft.

## Contents

1. [Deliverable 1 — Draft outreach email](#deliverable-1--draft-outreach-email)
2. [Deliverable 2 — Sequencing plan](#deliverable-2--sequencing-plan)
3. [Deliverable 3 — iroh-team roster](#deliverable-3--iroh-team-roster)
4. [Deliverable 4 — Framing-mistakes-to-avoid checklist](#deliverable-4--framing-mistakes-to-avoid-checklist)
5. [Deliverable 5 — Decision-gate criteria for orchestrator](#deliverable-5--decision-gate-criteria-for-orchestrator)
6. [Pre-send checklist](#pre-send-checklist)
7. [Concerns about the Shape 5 strategy](#concerns-about-the-shape-5-strategy)

---

## Deliverable 1 — Draft outreach email

### A. Recommended sequencing: TWO outreach emails, not one

Per L3 `shape_5_recommended_amendment`: the right shape is **(1) bare-framing-question first** — "would you comment on whether our framing accurately characterizes your team's reasoning?" — sent BEFORE the v2 draft is locked, used to shape the draft. Then **(2) full draft + critique request** sent after revisions land, with a 4-week review window and explicit veto.

Single-email "here's our draft, please critique" works only if Email-1 was already implicit (e.g. prior conversation, public engagement). It is not implicit here — there is no prior n0 ↔ Benten relationship. So we send both.

Drafts for both below.

### B. Email-1 — framing-check (send BEFORE v2 blog draft is locked)

**To**: Rüdiger Klaehn `<rklaehn@protonmail.com>` (author of the PQ blog; primary technical voice on iroh's PQ posture) — see roster for verification of contact method. CC: Philipp Krüger (matheus23) — author of the PQ-KEX-vs-PQ-sig comment on iroh#4195; engaged with the structural question already.

**Subject**: `iroh PQ post — framing-check before we draft a public response`

**Body** (~280 words):

> Rüdiger, Philipp —
>
> We work on Benten Engine, a content-addressed graph + capability system in Rust (10 crates, 12-primitive engine + UCAN-style delegation). We use iroh as transport substrate (iroh-blobs, iroh-gossip, iroh-net) and are preparing to tag `v1-beta` with PQ-hybrid signatures as the default — specifically the LAMPS Composite ML-DSA construction (`id-MLDSA65-Ed25519-SHA512`), with classical-only and SLH-DSA as opt-in swap arms behind multicodec codepoint dispatch.
>
> We're drafting a public blog explaining our reasoning. The post engages directly with your 2026-05-19 "Iroh Post-Quantum Handshakes" piece because Benten depends on iroh and our reasoning has to live alongside yours coherently in the same ecosystem.
>
> Before we write the draft, we want your framing-check on one specific question, because we're worried about getting it wrong:
>
> 1. **Do we represent your team's stated reasoning correctly?** Our read of your post is that you cited two reasons for deferring PQ signatures: (a) "no harvest-now-decrypt-later equivalent for signatures" and (b) "no industry consensus yet which one to use" plus "active research" + significant cost/size downsides. We want to engage with (a) for one specific system class (long-lived signatures that delegate capability into automated trust chains without per-use human review — e.g. UCAN-chained plugin authority, AI-agent delegated authority) while agreeing with (b) and acknowledging LAMPS Composite ML-DSA is pre-RFC. Does that representation sound accurate? If we have your reasoning wrong, we'd rather find out now than after we publish.
>
> 2. **Where does iroh's EndpointId sit in this?** Our current read is that iroh-blobs blob integrity comes from BLAKE3 content addressing (no Ed25519 needed), and the only long-lived signed iroh surface is the EndpointId. That seems to be inside the class we're arguing about, even though it's adjacent to your transport-identity use case. Is that read accurate, or are we misreading the EndpointId's lifecycle?
>
> No deadline for response — we will not draft past the framing skeleton until we hear back (or hear that you're not the right people to comment). If there's someone else on the n0 team better-placed to answer, please point us. We'd rather slow our timeline than misrepresent your reasoning.
>
> — Ben Tan-Brown <ben@benten.ai>, Benten Engine
> https://github.com/BentenAI/benten-engine (currently private; can share access)

### C. Email-2 — draft critique request (send AFTER v2 blog draft incorporating L3's 4 framing concessions; AFTER Email-1 response or T+14 no-response)

**To**: Same recipients as Email-1, plus any n0 team member they redirected us to in their Email-1 response.

**Subject**: `Benten Engine PQ-hybrid blog — draft for technical critique (NOT a publication notice)`

**Body** (~420 words):

> Rüdiger, Philipp —
>
> Following up on the framing-check from [DATE]. Thanks for [response substance / acknowledgment of where they redirected us]. Attaching the v2 draft below.
>
> **What this email is**: a request for technical critique that may change whether we publish this post, and how. We are NOT giving you a publication notice. If you read the draft and conclude the framing misrepresents iroh or the argument is wrong for the system class we're claiming, that response from you should change what we do next — we want to know that BEFORE the post is public, not after.
>
> **What we're shipping**: At `v1-beta` (target ~6-8 weeks out), Benten signs content-addressed artifacts and capability grants with hybrid Ed25519 + ML-DSA-65 (LAMPS Composite, `id-MLDSA65-Ed25519-SHA512`). Hybrid is default; classical-only and SLH-DSA are non-default opt-in swap arms behind multicodec codepoint dispatch with typed-reject on unknown algorithms (no silent fallback). Open-source library at `crates/benten-crypto-suite` + published crate (forthcoming) + planned `@bentenai/crypto-suite` npm package. Bidirectional swap matrix is part of the conformance suite. Independent `ml-dsa` / `ml-kem` audit is funded; lands during the v1-beta window and gates v1-GM.
>
> **Where the post engages with iroh's reasoning**: §[X] of the draft cites your post on the two specific arguments you made — (a) no-HNDL-for-signatures and (b) no-industry-consensus-on-PQ-sig-scheme. We argue against (a) only for the specific system class of long-lived signatures delegating capability into automated trust chains without per-use human review. We acknowledge (b) is correct as-stated and that LAMPS Composite is pre-RFC — we frame our shipping decision as "first-adopter accepting RFC-divergence risk with a documented migration path," not "consensus exists."
>
> **What we want from you, concretely**:
>
> 1. Technical critique on whether we represent your reasoning accurately. The L3-style read of your post (an early outside reviewer wrote it for us) said our prior framing was Benten's interpretive overlay — that you did NOT use the "ephemeral session vs persistent artifact" distinction at all. We've cut that framing from the v2 draft. Did we cut enough?
> 2. Technical critique on whether the argument is correct for the system class we're arguing about. We're claiming the threat model is different for delegated-capability-content-addressed-systems specifically; we're explicitly NOT claiming iroh chose wrong on transport identity.
> 3. **Veto power on the framing if you read it as misrepresenting iroh.** If after reading the draft you say "this still mischaracterizes our reasoning," we will rewrite or pull specific iroh references entirely. If you say "the entire premise mischaracterizes the class boundary," we will not publish in this form — we'll either fully restructure or shelve the post and publish only the library.
> 4. **Open question on co-authorship / co-signature**, if you have appetite for it: would you (n0 or the specific authors of your PQ post) be open to co-authoring the part of the argument about content-addressed-blob-persistence being structurally distinct from transport identity? This is genuinely a place where the two systems reached different defensible conclusions under uncertainty, and a joint "two systems in adjacent design spaces reached different defensible conclusions" framing would be more honest about the actual state of the question than either of us writing solo. If no appetite, that is completely fine — we'll publish solo and cite your post as the deferral reasoning we engaged with, with the framing you sign off on.
>
> **What we are NOT asking for**: endorsement. We want critique. Telling us "we still disagree with your framing but didn't catch any misrepresentation" is exactly the response we need; we will publish that as the acknowledgments.
>
> **Timeline**: We are not on a fixed publication deadline. Our internal target was 4 weeks from this email, which gives you a 28-day review window. If you need more time, say so and we will extend. If we don't hear back in 4 weeks, we will follow up before doing anything. We will not publish without your awareness, and we will not name iroh / n0 / specific team members in the post without your sign-off on the language we use.
>
> Draft link: [PRIVATE LINK — pre-publication access, do not distribute]
>
> — Ben Tan-Brown <ben@benten.ai>, Benten Engine

---

## Deliverable 2 — Sequencing plan

Timeline anchored to **T+0 = Email-1 send**, NOT to blog draft completion. The Email-1 send should happen as soon as we know what Position B v2's argument is going to be in skeleton form (we don't need the full draft; we need the bare framing claim to ask "do we have your reasoning right?").

### Stages

**T-14 to T-0** (BEFORE Email-1):
- Position B revision agent completes v2 blog draft skeleton (just the argument structure + the section about iroh, not full prose).
- Orchestrator + Ben review the iroh-engagement section against L3's 4 framing concessions:
  1. Drop "ephemeral session vs persistent" sharpness; reframe as "extending n0's reasoning to a class they did not address" OR own that this is a substantive technical disagreement on argument (a).
  2. Break out iroh-EndpointId as a separate row acknowledging it IS persistent.
  3. Engage with "no industry consensus" argument directly; cite LAMPS as emerging-not-fully-consensus.
  4. Acknowledge n0's defer-decision is defensible under uncertainty, not a misjudgment.
- L3 concessions verified-present in Email-1's framing-check questions.

**T+0**: Send Email-1 (framing-check). Set internal calendar reminder for T+10.

**T+10** (if no response): Send polite single-line follow-up:
> "Hi Rüdiger / Philipp — circling back on the framing-check question from [DATE]. Happy to extend timeline; just want to know if you're the right people to comment or if we should redirect."
- No new content. Reminder only.

**T+14** (if still no response):
- DECISION POINT 1 — see Deliverable 5.
- Default path: assume their team is over-committed; proceed to draft v2 incorporating L3's 4 framing concessions exactly as written; send Email-2 anyway (with caveat that we're proceeding without their framing-check input but still want critique).

**T+14 to T+21** (if Email-1 response received):
- Incorporate their framing-check response into v2 draft.
- Position B revision agent revises blog per their input.
- Orchestrator verifies revised draft still passes the other 14 critic lenses (L1+L2+L4 etc.).

**T+21**: Send Email-2 (full draft critique request) with 28-day review window. (If Email-1 was no-response, this is T+14.)

**T+21 to T+49** (4-week review window):
- No publication during this window. No further outreach unless they initiate.
- If they respond with substantive technical critique: revise per their input; if revisions are substantial, send a delta-summary back ("here's what we changed; sound right?") with another 7-day mini-window.
- If they respond with VETO ON FRAMING: stop. See Decision-gate 5 in Deliverable 5.

**T+49 (or earlier if substantive sign-off lands)**: DECISION POINT 2 — see Deliverable 5. Publication or deferral.

### Iroh-substantive-engagement → publish with co-author path estimate

If n0 engages substantively AND offers co-authorship: add **+2-4 weeks** for joint authoring (n0 will need their own internal review for anything they put their name on; matches their blog cadence of ~monthly substantive posts). Publication target: T+70 to T+84.

### Iroh-declines → solo publish path estimate

If n0 engages substantively but declines co-authorship: revisions land in the T+21 to T+49 window; publication target ~T+56 to T+63 (small buffer after 28-day window for final orchestrator + Ben polish).

Trade-off vs co-author path: solo gets us to publication ~3-4 weeks faster but loses the "first of two" framing that L15 flagged as STRENGTHENING Position B. Solo is fine; co-authored is better.

### Iroh-silence → follow-up cadence

T+10 (Email-1 follow-up, single line) → T+14 (proceed without framing-check, send Email-2) → T+28 (Email-2 follow-up, single line if needed) → T+42 (Email-2 second follow-up; explicit "we will publish in 14 days unless we hear from you") → T+49 (publication or defer; see Decision-gate 4).

### "Never silently re-prompt" floor

Per `feedback_arm_schedulewakeup_on_ci_wait`-style discipline: every outbound email + follow-up gets a calendar reminder. Going silent waiting for a response that never comes is the failure mode we want to design out.

---

## Deliverable 3 — iroh-team roster

> Verified 2026-05-26 against the iroh blog post + GitHub n0-computer org public_members list + per-user public GitHub profile data + WebSearch corroboration. All claims here are from public sources; methodology notes in §"Verification methodology" below.

### Primary recipients (Email-1 + Email-2)

#### Rüdiger Klaehn — `rklaehn`
- **Role**: Independent hacker contracting with n0-computer; primary author of the iroh PQ blog post.
- **Why primary recipient**: He wrote "Iroh Post-Quantum Handshakes" (2026-05-19); his name is on the bylin. He is the author whose stated reasoning Benten's post engages with — addressing the framing-check question to him is the engineer-respect default.
- **GitHub**: https://github.com/rklaehn
- **Public profile data**: "Old grumpy hacker. I like simple things that work." — Independent hacker, Transylvania, Romania.
- **Contact method**: Best inferred channel = email. **Cannot find a verified public email**; will need to ask Ben whether he has it or whether we ask via iroh's discord (https://iroh.computer/discord). His GitHub profile lists no public email. **NOT VERIFIED — flagging as orchestrator-decision-point.**
- **Twitter/Bluesky/Mastodon**: None listed on public GitHub profile.
- **Caution**: bio "old grumpy hacker / likes simple things that work" — engineer-respect tone is even more load-bearing for this recipient than the default.

#### Philipp Krüger — `matheus23`
- **Role**: n0-computer team member (verified via public_members API: confirmed public organization member). Iroh contributor.
- **Why CC primary**: He wrote the comment on iroh#4195 that the content-addressed-systems scan flagged as the closest public statement of iroh's PQ-sig-skeptical reasoning ("if PQ matters, classical doesn't fulfill threat model"). He has already engaged the structural question on a public iroh thread, so the framing-check question is in-scope for him.
- **GitHub**: https://github.com/matheus23 — Company `@n0-computer`; Baden-Württemberg, Germany.
- **Blog**: https://irreactive.com
- **Contact method**: blog + GitHub. **No public email verified**; same channel question as Rüdiger.
- **Twitter/Bluesky/Mastodon**: None listed on public GitHub profile.

### Likely secondary engagement (consider for Email-2 CC if Email-1 redirects there)

#### Friedel Ziegelmayer — `dignifiedquire`
- **Role**: Top iroh contributor by commit count (verified 785 contributions, highest among recent contributors). Likely engineering lead based on volume + the issue-priority calls (e.g. `dignifiedquire` made the call on iroh#3849 endpoint-ID-extensibility being "will do this not for 1.0"). 
- **GitHub**: https://github.com/dignifiedquire — Germany.
- **Twitter**: `dignifiedquire`.
- **Caution**: NOT listed as public org member, only as contributor; his exact relationship to n0 (employee / contractor / volunteer) is not verifiable from public data. **Do NOT name as "n0 team member" in any external doc unless verified.**

#### Floris Bruynooghe — `flub`
- **Role**: n0-computer-affiliated (Company `@n0-computer` in public bio); 476 contributions to iroh. Engaged on iroh#3535 NodeId-validation API design.
- **GitHub**: https://github.com/flub — http://devork.be — "The Mountains."
- **NOT listed as public org member**; bio claims affiliation. Lower-confidence than the public_members list.

#### Franz Heinzmann — `Frando`
- **Role**: Public n0-computer org member. 257 contributions to iroh.
- **GitHub**: https://github.com/Frando — Company `@n0-computer @arso-project`; Freiburg, Germany.

#### `ramfox`
- **Role**: Public n0-computer org member. 123 contributions to iroh.
- **GitHub**: https://github.com/ramfox — Company `n0.computer`; NYC.

#### Brendan O'Brien — `b5`
- **Role**: Public bio "Caretaker at @n0-computer" — likely founder / business lead (subject of devtools.fm episode 148 "Brendan O'Brien — n0, Iroh and the Future of Peer to Peer").
- **GitHub**: https://github.com/b5 — Canada.
- **Twitter**: `b_fiive`.
- **Caution**: business / community-lead role, not primary technical voice on PQ posture. Do not CC on Email-1 (would dilute the technical-question framing); could be relevant for Email-2 if the discussion broadens to coordination scope.

### Other recent contributors (NOT recipients; named for completeness)

- **Asmir Avdicevic — `Arqu`** (138 contributions; Bosnia). Not a public org member.
- **Diva Martínez — `divagant-martian`** (124 contributions). Not a public org member.

### Filippo Valsorda relationship: NOT VERIFIED

The brief asked whether Filippo Valsorda (Go cryptography maintainer, prominent PQ advocate) has any relationship to iroh / n0 as advisor or consultant.

WebSearch 2026-05-26 surfaced no evidence of any formal Valsorda↔n0 relationship. His public PQ-advocacy work (https://words.filippo.io/) overlaps thematically with the iroh PQ-KEX work but his name does not appear in iroh's blog post bylines or on the n0 public_members list. **No connection found; do NOT reference any presumed advisory relationship in the email.**

### Org context

- **Name**: n0, inc. (DBA "number zero"); GitHub org `n0-computer` (created 2022-03-05).
- **Public communication channels**: Discord (https://iroh.computer/discord); Twitter `@iroh_n0`; Bluesky `@iroh.computer`; Mastodon `@n0iroh`; YouTube `@n0computer`.
- **Engagement-channel recommendation**: **email** if recipient public email can be sourced; **Discord DM as fallback** (n0 runs an active Discord); **NOT** Twitter / public Bluesky / public Mastodon (these are inappropriate for the "request for technical critique that may change whether we publish" framing — converts private outreach to public spectacle).
- **GitHub-issue channel is NOT appropriate** for this initial outreach — even a private GitHub issue would surface eventually + we explicitly want a private channel per Shape 5.

### Verification methodology

- iroh PQ blog author verified via WebFetch of https://www.iroh.computer/blog/iroh-post-quantum-handshakes 2026-05-26 → "Author: Rüdiger Klaehn (published May 19, 2026)" + verbatim quote-block of stated reasoning extracted.
- Public org members verified via `gh api orgs/n0-computer/public_members` 2026-05-26: exactly 4 public members (b5, Frando, matheus23, ramfox).
- Contributor activity verified via `gh api repos/n0-computer/iroh/contributors` 2026-05-26.
- Per-user profile data verified via `gh api users/<login>` 2026-05-26.
- Filippo Valsorda relationship: WebSearch 2026-05-26 with explicit "Filippo Valsorda iroh n0-computer advisor cryptography post-quantum" query; zero results connecting Valsorda to n0.
- Per `feedback_review_finding_ground_truth_verify`: any role/relationship not in this section's verification list is NOT verified and should not appear in the email or in any public Benten artifact about iroh.

---

## Deliverable 4 — Framing-mistakes-to-avoid checklist

Each item: **mistake** + **how to detect it in a draft** + **fix**.

### Group A — L3 4-concessions (load-bearing; soft-VETO on F5 lifts only when all 4 land)

**A.1 — "Heads-up before publish" framing**
- *Mistake*: Email reads as "we're going to publish in N weeks; here's the draft as a courtesy."
- *Detect*: Search the email for "heads up" / "courtesy" / "before we publish" with no caveat that "your response may change whether we publish." If the response from the recipient could be ignored without consequence to whether/what we publish, the framing is wrong.
- *Fix*: Explicit "this is a request for technical critique that may change whether we publish." First-paragraph framing, not buried.

**A.2 — Strawmanning iroh's reasoning ("ephemeral session vs persistent artifact" overlay)**
- *Mistake*: Draft attributes to iroh a distinction iroh did not make.
- *Detect*: Search blog draft for phrases "ephemeral session" / "session encryption" attributed to iroh; verify against the VERBATIM iroh blog text (only n0-stated reasons: "no HNDL-equivalent for signatures" + "no industry consensus" + "active research" + cost/size downsides).
- *Fix*: Drop the "ephemeral session vs persistent artifact" framing wholesale. Engage with iroh's ACTUAL two-part reasoning. If we want to argue the threat model is different for delegated-capability-content-addressed systems, that is a CLAIM ABOUT BENTEN's class, not an interpretation of iroh's reasoning.

**A.3 — Treating iroh-blobs as "ephemeral session"**
- *Mistake*: Draft places iroh wholesale in the "ephemeral session" column of any asymmetry table.
- *Detect*: Search blog draft for any table/section that classifies iroh as ephemeral. Cross-check: iroh-blobs IS content-addressed long-term-persistence (BLAKE3 content-addressing means blobs don't need signatures for integrity — the CID IS the integrity guarantee); the only long-term-signed surface in iroh is the EndpointId.
- *Fix*: Drop the wholesale-iroh classification. Break out iroh-EndpointId as a separate row that explicitly acknowledges it IS persistent and IS within Benten's argument's scope. Acknowledge n0 has a defensible position to defer hybrid-sig even for the EndpointId case under uncertainty about which scheme to use.

**A.4 — Under-engaging with "no industry consensus" argument**
- *Mistake*: Draft waves off n0's strongest argument (no industry consensus on which PQ-sig scheme); claims LAMPS Composite is consensus when LAMPS is at draft v19, not RFC.
- *Detect*: Search blog draft for phrases like "industry has converged" / "consensus has emerged" without qualification. Verify against the actual LAMPS state (draft-ietf-lamps-pq-composite-sigs-19; not yet RFC; AUTH48 / RFC-Editor-Queue depending on exact week).
- *Fix*: Honest framing: "LAMPS Composite is the emerging consensus that we believe will be the answer; we ship it pre-RFC because content-addressed-persistent artifacts can't easily migrate later; we accept RFC-divergence risk; if the WG changes the construction post-RFC we will ship a v2 wire format and migrate." Acknowledge n0's wait-for-RFC position is defensible.

### Group B — Tone / posture mistakes (broader than L3; orthogonal to the 4 concessions)

**B.1 — Asking for endorsement vs critique**
- *Mistake*: Email reads as "would you co-sign / co-promote?" with critique as a side-ask.
- *Detect*: Search the email for "endorse" / "co-sign" / "amplify" / "share" without "critique" / "veto" / "may change" elsewhere. Verify the email leads with the critique-request, not with the co-author-question.
- *Fix*: Critique-request is first. Co-author-question is at the END, framed as optional, framed as "we recognize you may decline and that is fine." Per Email-2 §"What we want from you, concretely" — critique is items 1-3; co-authorship is item 4.

**B.2 — "We'd love to collaborate" / marketing voice**
- *Mistake*: Email reads as a marketing pitch dressed up as technical outreach.
- *Detect*: Search the email for: "love to," "excited to," "thrilled," "amazing," "incredible," any em-dash-as-rhythm, any phrase that wouldn't survive being read aloud by an engineer to an engineer at a whiteboard.
- *Fix*: Engineer-to-engineer tone. Lead with technical content. State the ask plainly. No marketing register.

**B.3 — L5 small-team-overreach: over-claiming Benten's standing in the conversation**
- *Mistake*: Email implies Benten is a peer-scale project to n0 (which has ~3000 GitHub stars on iroh, multiple full-time engineers, and ships at Cloudflare-substrate scale).
- *Detect*: Search the email for: "we lead," "first in the industry," "ecosystem-wide," any claim that overstates the size of Benten's standing relative to n0's.
- *Fix*: Honest framing. Benten is small (Ben is the primary developer; 5 weeks old as of public visibility; pre-v1-beta). The email's authority comes from "we did the work, we're shipping this construction, here is what we found" — not from claimed industry-standing. The recipient should never finish the email thinking Benten claimed something it isn't.

**B.4 — Treating n0 as a marketing target rather than technical peer**
- *Mistake*: Email reads as "we want your name attached for legitimacy."
- *Detect*: Test: would the email be different if iroh had 50 GitHub stars instead of 3000? If yes, the email is treating them as a marketing target. The technical-critique value of their input should not depend on their size.
- *Fix*: Frame the ask on TECHNICAL grounds (they wrote the post we're engaging with; they thought carefully about the same question and reached a different conclusion; their critique is technically valuable). NOT on social grounds (their name lends weight, their amplification reaches X audience).

**B.5 — Over-claiming what "shipping" means**
- *Mistake*: Email claims Benten has "shipped" PQ-hybrid; we are pre-`v1-beta`-tag.
- *Detect*: Search for: "we ship" / "we shipped" / "in production" / "real users." Verify against actual state: code lands at `crates/benten-crypto-suite`; no public package yet; no external adopters; pre-tag.
- *Fix*: Use "shipping at v1-beta" / "decided" / "landed on" / "the construction we're tagging at v1-beta." Acknowledge pre-tag, pre-audit, pre-adoption status.

### Group C — Technical-claim cite-drift mistakes (per `feedback_review_finding_ground_truth_verify`)

**C.1 — Wrong attribution for the iroh PQ blog**
- *Mistake*: Email attributes the iroh PQ-blog reasoning to a person who didn't write it.
- *Detect*: Verify against WebFetch of https://www.iroh.computer/blog/iroh-post-quantum-handshakes — author = Rüdiger Klaehn, published 2026-05-19.
- *Fix*: Address Email-1 To: Rüdiger directly; CC matheus23 (Philipp) only because of his iroh#4195 substantive comment (not because of the blog post).

**C.2 — Misquoting iroh's blog text**
- *Mistake*: Email paraphrases iroh's reasoning into a form n0 will dispute.
- *Detect*: Compare every sentence in the email that describes iroh's reasoning against the verbatim blog text:
  - "While post-quantum key exchange algorithms are standardized and come with relatively modest overhead, post-quantum signature algorithms are the subject of active research."
  - "All current options have significant downsides regarding computational cost, key size or signature size. So there is no industry consensus yet which one to use."
  - "There is also less urgency because there is no harvest-now-decrypt-later equivalent for signatures."
- *Fix*: When citing their reasoning, prefer verbatim quote-blocks over paraphrase. Paraphrase only where strictly necessary, and pin the paraphrase against the verbatim text. Per `feedback_industry_folklore_vs_spec_text_distinction` (pim-N candidate).

**C.3 — Imagined contact methods**
- *Mistake*: Email is sent to an address / handle that doesn't actually reach the named recipient.
- *Detect*: Per Deliverable 3, public emails for both Rüdiger and matheus23 are NOT verified at the time of this brief. Sending to a guessed email risks bounce / silent-drop.
- *Fix*: Orchestrator confirms with Ben (or via iroh Discord polite-DM-to-team-bot-ask channel) what the correct private contact method is before sending. Decision-point flagged in Pre-send checklist.

**C.4 — Imagined relationships (Filippo Valsorda etc.)**
- *Mistake*: Email cites someone as an iroh advisor / consultant who is not in fact in that role.
- *Detect*: WebSearch + `gh api users/<login>` + iroh blog bylines should support every named relationship. Per Deliverable 3: Filippo Valsorda has NO verifiable relationship to iroh / n0.
- *Fix*: Do not reference any presumed advisory relationship. If we want to credit Valsorda's PQ-advocacy work as inspiration for Benten's posture, do that in the blog post acknowledgments (where the claim is "we read his work" not "he advised iroh"), not in the iroh outreach email.

### Group D — Process discipline mistakes (L3-flagged "what would make me say NO")

**D.1 — Publishing within 2 weeks of the conversation regardless of response**
- *Mistake*: Outreach happens but publication timeline doesn't move based on response.
- *Detect*: If publication is calendared independently of T+0, the conversation is performative.
- *Fix*: Calendar publication relative to T+49 (Email-2 + 28-day window + buffer), explicitly NOT relative to v2 draft completion. If they respond with substantive concerns, calendar moves out further.

**D.2 — Stripping their "research in progress" caveat from quotes**
- *Mistake*: Blog quotes n0 in a way that makes them look more committed to non-adoption than they are.
- *Detect*: If any blog quote of iroh's reasoning omits qualifiers like "active research" / "no industry consensus yet" / "less urgency," the quote is strip-shortened. Per L3: "any quote from our blog post that strips the 'we are researching alternatives' caveat — we are NOT a deferral example, we are an explicit-research-in-progress example."
- *Fix*: Quote the FULL hedged-reasoning sentence, not the bare claim. If a blog block needs a shorter quote for flow, paraphrase with the qualifier intact.

**D.3 — Implying n0 didn't think carefully**
- *Mistake*: Draft implies n0's defer-decision was lazy / underconsidered / a misjudgment.
- *Detect*: Search draft for phrasing like "n0 chose to wait" / "n0 hasn't addressed" / "n0 hasn't engaged with" → these read as criticism. Phrasing like "n0 cited the following reasons" / "n0 considered the same question and reached a defensible different conclusion" reads as respect.
- *Fix*: Per L3 R-3-4: "iroh's posture is a defensible defer-decision under uncertainty, not a misjudgment." Use the more-explicit-respect phrasing.

**D.4 — "We chose right, they chose to wait"**
- *Mistake*: Blog frames the comparison as right-vs-wrong rather than two-defensible-conclusions-in-adjacent-design-spaces.
- *Detect*: Search draft for phrases like "we chose / they didn't," "we recognized / they missed," "we adopted / they deferred."
- *Fix*: "Two systems in adjacent design spaces reached different defensible conclusions under uncertainty" — L3 R-13-shape verbatim. The blog argues OUR class needs hybrid; it does NOT argue iroh chose wrong on transport identity. Where the EndpointId case overlaps, blog acknowledges defensible-disagreement, not chose-wrong.

### Group E — Co-author specific mistakes (only relevant if path B engages)

**E.1 — Pushing for co-authorship before they're ready**
- *Mistake*: Co-author question lands in Email-1 framing-check, treating it as a near-term plan.
- *Detect*: Co-author question appears before Email-2.
- *Fix*: Co-author is item 4 of Email-2's "what we want" list and is the END (after the critique requests). Framing-check email (Email-1) does NOT mention co-authorship.

**E.2 — Co-authorship → strip their "research in progress" reservations**
- *Mistake*: If they agree to co-author, joint post papers over their actual reservations to make joint position cleaner.
- *Detect*: Joint draft tracks one perspective; minority view is footnoted out.
- *Fix*: Joint framing is structurally "two systems, two contexts, two defensible conclusions" — their reservations remain first-class in the post even if it has Benten + iroh bylines. Probably side-by-side framing rather than blended-voice.

### Group F — Internal-process mistakes

**F.1 — Sending Email-1 without v2 blog skeleton complete**
- *Mistake*: Email-1 sent before Benten knows what argument the v2 blog is going to make.
- *Detect*: Can the orchestrator answer "what does Benten's blog argue about iroh, in one sentence?" If no, Email-1 isn't ready.
- *Fix*: V2 blog skeleton has to land first; the framing-check questions need to refer to claims Benten will ACTUALLY make, not claims we're "considering."

**F.2 — Treating "no response in T+10" as silent assent**
- *Mistake*: No response → proceed as if they agreed.
- *Detect*: Email-2 sent at T+10/T+14 with no acknowledgment of the silence.
- *Fix*: Per Sequencing plan: T+10 follow-up reminder; T+14 only proceeds with explicit "we're proceeding without your input but still want your critique" framing in Email-2.

**F.3 — Posting on iroh public threads during outreach window**
- *Mistake*: Benten comments on iroh#4195 / #3849 / #2355 (the 3 WATCH-ONLY-PENDING-SHAPE-5 threads from `comment-opportunities-content-addressed-systems.md` §1) while the outreach conversation is in flight.
- *Detect*: Comment-opportunity batch-posting cadence is decoupled from Shape 5 outreach cadence.
- *Fix*: All 3 iroh-cluster threads STAY WATCH-ONLY until the entire Shape 5 outreach concludes (publish OR defer). If Shape 5 lands with co-authorship, the threads become candidates for a follow-up coordinated public comment. If Shape 5 lands with solo publish, the threads become candidates for low-key Benten-perspective comments AFTER publication. If Shape 5 defers, threads remain WATCH-ONLY indefinitely.

---

## Deliverable 5 — Decision-gate criteria for orchestrator

Triggered by n0's response to Email-1 or Email-2. Each gate names the response shape, the decision rule, and the next action.

### Gate 1 — Email-1 response (T+0 to T+14)

**Response shape A — substantive framing-check feedback (best case)**
- They engage with the framing question; agree / disagree / nuance about whether Benten represents their reasoning accurately; named specific concerns or accepted corrections.
- Decision: incorporate their framing-check directly into v2 draft. Position B revision agent revises iroh-engagement section per their input. Send Email-2 within T+21 of original send.

**Response shape B — redirect to different team member**
- They say "Rüdiger is the right person, not Philipp" or "ask matheus23 about the EndpointId question."
- Decision: re-send Email-1 to the redirected recipient (or add them to Email-2's recipient list). Reset T+0 if substantial redirect; pivot if minor.

**Response shape C — polite decline ("we don't have bandwidth for this right now")**
- Decision: thank them; proceed with v2 draft incorporating L3's 4 concessions exactly as the L3 critic specified (since we don't have n0's own framing-check); send Email-2 anyway with explicit "we appreciate the bandwidth constraint and proceed without your framing-check, but still want your critique on the actual draft." Their decline on Email-1 doesn't preclude Email-2 engagement.

**Response shape D — strong pushback on the entire premise**
- They write back "this is a misframing of the entire question; you shouldn't write this post at all."
- Decision: PAUSE. Orchestrator surfaces full message + reasoning to Ben for direct judgment. Possible outcomes: (a) revise v2 fundamentally; (b) defer F5 indefinitely; (c) pivot to F6-library-only public stance. Convergent-paths-within-auth does NOT cover this; this is a divergent-path requiring Ben.

**Response shape E — silence at T+14**
- Decision: per Sequencing plan, send Email-2 with explicit "we're proceeding without your framing-check input but still want critique on the actual draft." Single-line follow-up at T+10 first; if still silence at T+14, proceed.

### Gate 2 — Email-2 response (T+21 to T+49)

**Response shape F — substantive engagement, no veto, no co-author offer**
- They engage with critique; flag specific concerns; do not raise veto-level objections; do not offer co-authorship.
- Decision: revise per their input; cite their critique in acknowledgments; publish solo at T+56-63. Per L15: blog still has "first-shipping in converging cohort" framing (the converging cohort is the OpenPGP-PQC / Sigstore / VC-DI parallel — not iroh).

**Response shape G — substantive engagement + co-author / co-signature offer (best case)**
- They engage substantively + offer to put their name on the part of the argument about content-addressed-blob-persistence vs transport identity.
- Decision: extend timeline +2-4 weeks for joint authoring. Publication at T+70-84. Per L3 R-13-shape: "two systems in adjacent design spaces reached different defensible conclusions under uncertainty." Blog reframes from "Benten alone" to "Benten + iroh first of two."

**Response shape H — pushback on framing without veto**
- They say "this still doesn't represent our reasoning accurately" or "the EndpointId framing isn't right" — but don't explicitly veto.
- Decision: revise per their specific concerns. Send a delta-summary back: "here's what we changed; sound right?" with a 7-day mini-window. If they sign off, publish. If they push back again, escalate to Gate 3.

**Response shape I — VETO ON FRAMING**
- They explicitly say "we object to the post mentioning iroh in this way" or "we read this as misrepresenting us and request that iroh references be cut."
- Decision: STOP. Per L3: "Benten must give n0 actual VETO power over the framing that mentions us — anything less is using our name without consent." Options:
  - (a) Cut all specific iroh references; publish without them (the structural argument can stand without naming iroh; per L15 the converging-cohort framing already has 3 other reference points: OpenPGP-PQC / Sigstore / VC-DI).
  - (b) Restructure fundamentally — pivot to F6-library-only public stance (per L1 INVERT-priority recommendation; ship library + ship engineering-notes A-grade post about Benten's construction; no broad public-stance argument about content-addressed systems class).
  - (c) Defer F5 indefinitely; revisit after independent ml-dsa audit lands (would be ~6-12 months out, giving evidence-base for any future public-stance attempt).
  - Orchestrator surfaces options + tradeoffs to Ben for direct judgment. Default prediction: (a) cut references and publish IF the structural argument survives without them; otherwise (c).

**Response shape J — silence at T+49**
- Decision: T+28 single-line follow-up; T+42 explicit "we will publish in 14 days unless we hear from you"; T+49 publication or defer. **Per L3 explicit caution: silent-publish-after-private-conversation converts "private heads-up" into "using their name without consent."** If n0 fundamentally doesn't engage at all (no Email-1 response AND no Email-2 response), the email chain itself becomes evidence we made the courtesy attempt; publication can proceed BUT the blog draft must reflect that no n0 engagement happened (cite "we reached out to the n0 team for technical critique pre-publication; we did not receive a response; we proceeded with revisions per [the L3 critic / our own best-faith read]" honestly).

**Response shape K — conflicting signals across team members**
- Different n0 team members respond with different verdicts (one says "OK," another says "veto").
- Decision: do NOT pick a winner. Surface conflict to Ben for direct judgment. Default prediction: defer publication; ask n0 internally to resolve their own position; we don't insert ourselves into their internal disagreement.

### Gate 3 — Mid-window revision delta sign-off (Response H follow-up)

**Sub-shape H.1 — they sign off on revised delta**: publish per Gate 2 Response F path.

**Sub-shape H.2 — they push back again on revised delta**: escalate to Gate 2 Response I (VETO ON FRAMING) — at this point the iterative-revision approach is failing and we should treat it as functional veto.

### Decision gate matrix summary

| Response | Action | Surface to Ben? |
|----------|--------|----|
| A (framing-check substantive) | Incorporate; send Email-2 at T+21 | No (convergent path) |
| B (redirect) | Re-send to redirect | No (convergent path) |
| C (Email-1 polite decline) | Proceed with v2 + L3 4 concessions; send Email-2 anyway | No (convergent path) |
| D (Email-1 strong pushback on entire premise) | PAUSE | **YES — divergent** |
| E (Email-1 silence at T+14) | Proceed; send Email-2 with explicit no-framing-check ack | No (convergent path) |
| F (Email-2 substantive, no veto, no co-author) | Publish solo | No (convergent path) |
| G (Email-2 substantive + co-author offer) | Joint authoring | **YES — surface co-author terms** |
| H (Email-2 pushback without veto) | Revise + delta sign-off | No initially; YES if H.2 escalates |
| I (Email-2 VETO ON FRAMING) | STOP; 3 options (cut / restructure / defer) | **YES — divergent** |
| J (Email-2 silence at T+49) | Publish with honest "no response" framing | **YES — confirm publication decision** |
| K (conflicting signals) | Defer; ask n0 to resolve | **YES — divergent** |

Convergent paths (auth = orchestrator can proceed): A, B, C, E, F, H.1 (with delta sign-off).
Divergent paths (auth = surface to Ben): D, G, I, K, J. H.2 escalates from convergent to divergent.

Per `feedback_surface_arch_decisions_under_auth`: ALL of these divergent-path triggers surface to Ben with plain-English options + orchestrator-prediction + clear ask.

---

## Pre-send checklist

Orchestrator + Ben work through these BEFORE sending any email. Each unchecked item is a fix-now per HARD RULE 12.

### Before Email-1

- [ ] V2 blog skeleton complete (argument structure + iroh-engagement section drafted; not full prose).
- [ ] L3's 4 framing concessions verified-present in v2 blog skeleton:
  - [ ] Concession 1: "ephemeral vs persistent" overlay dropped OR owned as substantive disagreement.
  - [ ] Concession 2: iroh-EndpointId broken out as separate row acknowledging persistence.
  - [ ] Concession 3: n0's "no industry consensus" argument engaged with directly + LAMPS pre-RFC status acknowledged.
  - [ ] Concession 4: n0's posture characterized as "defensible under uncertainty" not "misjudgment."
- [ ] Email-1 read aloud (or sub-vocally) — does it sound like one engineer talking to another at a whiteboard? If marketing-voice surfaces anywhere, rewrite.
- [ ] Email-1 reviewed against Framing-mistakes checklist Groups A + B + C (Group D / E / F apply to Email-2 / process; Group A.2 and A.3 verified against iroh blog VERBATIM text).
- [ ] Recipients confirmed:
  - [ ] Rüdiger Klaehn public email VERIFIED (or alternative private channel confirmed — Discord DM with n0-team-route, or asked-Ben).
  - [ ] matheus23 (Philipp Krüger) public email VERIFIED or alternative.
  - [ ] No imagined-relationship references (Filippo Valsorda, etc.).
- [ ] Subject line: "iroh PQ post — framing-check before we draft a public response" (NOT marketing-y; states the ask).
- [ ] No CC of non-technical n0 contacts (b5 / etc.); critique-question stays at technical level.
- [ ] No public-thread comments on iroh#4195 / #3849 / #2355 in the past 14 days OR planned for the next 14 days (per F.3).
- [ ] Calendar reminder set for T+10 (follow-up).
- [ ] Decision-gate criteria (Deliverable 5) reviewed; orchestrator knows what to do on each response shape.

### Before Email-2

- [ ] All Pre-Email-1 boxes checked AND Email-1 either had a response (Response A/B/C) or silence-at-T+14 was reached.
- [ ] V2 blog draft FULL TEXT complete + incorporates Email-1 response (if any).
- [ ] Email-2 reviewed against ALL framing-mistakes Groups A + B + C + D + E + F.
- [ ] Private draft link active (link-only access, no public discoverability).
- [ ] 28-day timeline confirmed; calendar holds at T+28 (mid-window check), T+42 (penultimate follow-up), T+49 (decision gate).
- [ ] Co-author question included as item 4 (last) of "what we want," explicitly optional.
- [ ] Veto clause included as item 3 of "what we want" — verbatim "if you read it as misrepresenting iroh, we will rewrite or pull specific iroh references entirely."
- [ ] No public iroh-thread comments planned during T+0 to T+49 outreach window.

### Before publication (any path)

- [ ] All Email-1 + Email-2 boxes checked.
- [ ] Final blog text reviewed against framing-mistakes Group D (publish-vs-defer-vs-cut decision).
- [ ] If n0 engaged: all their critique incorporated OR explicit "we declined to incorporate X because [reason]" surfaced to them with response window.
- [ ] If n0 silent: blog text includes honest "we reached out to the n0 team; we did not receive a response; we proceeded with revisions per [source]" disclosure.
- [ ] If n0 vetoed framing (Response I): all specific iroh references either cut OR restructured per their veto; explicit confirmation from them on the restructured form.
- [ ] Publication-day-of: no surprise additional iroh references inserted after final sign-off.
- [ ] Comment-opportunity threads (iroh#4195 / #3849 / #2355): post-publication treatment decided per outcome:
  - Co-author path: coordinated comments with n0.
  - Solo path: low-key Benten-perspective comments after publication.
  - Defer path: remain WATCH-ONLY indefinitely.

---

## Concerns about the Shape 5 strategy

Per the brief's mandate to surface concerns: three substantive concerns about the Shape 5 strategy as currently shaped, with HARD RULE 12 dispositions.

### Concern 1 — Email-1 vs Email-2 split increases timeline by ~3-4 weeks vs single-email approach

The L3 amendment recommends two emails. This adds ~3 weeks to the publication timeline (Email-1 + 14-day window + revise + Email-2 + 28-day window vs single-email + 28-day window). The single-email approach (sending the full draft directly with the critique request + veto clause) IS L3-compliant in form if the "request for technical critique that may change publication" framing is sufficiently clear.

**Disposition**: BELONGS-NAMED-NOW. Concession to L3's explicit amendment text — L3 was clear that Email-1's bare-framing-check is distinct from Email-2's full-draft-critique and that doing the framing-check FIRST is what shapes the v2 draft, not the other way around. Cutting Email-1 risks landing a v2 draft that n0 then has to push back on, which puts them in a more adversarial posture than the bare-framing-check question does. Accept the timeline cost.

### Concern 2 — Co-authorship asks may be too forward for first contact

The brief and the L3 reasoning explicitly contemplate co-author / co-signature as a "best case" outcome. But asking a team with no prior relationship to put their names on a post you've already drafted is a significant ask — even with the L3-correct framing, n0 may read it as "we have a draft and we want your reputation attached." A more conservative version would ask only for critique in Email-2 and surface co-authorship only IF their response opens the door (e.g. they say "this is interesting; we'd consider X").

**Disposition**: BELONGS-NAMED-NOW into Email-2's item 4 framing. The draft above presents co-authorship as item 4 (last) of "what we want," explicitly framed as "if you have appetite for it" and "completely fine if no appetite." This is the L3-amendment-compliant form. But orchestrator + Ben should know it's a calibrated guess; if Ben prefers, item 4 can be cut from Email-2 entirely and the co-author conversation moved to a hypothetical Email-3 contingent on their Email-2 engagement.

### Concern 3 — The structural-argument-survives-without-iroh test is load-bearing

If Gate 2 lands at Response I (VETO ON FRAMING), the default fallback "cut all iroh references and publish without them" only works if the structural argument doesn't depend on iroh as the named example. Per `comment-opportunities-content-addressed-systems.md` §6.3: "iroh is the deliberate outlier on PQ-signatures" — meaning iroh is the most-distinctive example of a system that explicitly deferred. Cutting iroh references may leave the post structurally weaker (the converging-cohort framing has 3 other reference points but no canonical "system that deferred" example to argue against).

**Disposition**: BELONGS-NAMED-NOW into Position B revision agent's v2 draft scope. The v2 draft should be authored so that the structural argument stands on its own without iroh as a named example — iroh is one example among several, not the only example. If iroh references are cut at Gate 2 Response I, the post should not collapse. Position B revision agent verifies this property at v2 completion.

### Out-of-scope concerns (NOT surfaced; flagged here for completeness)

- **Whether the entire F5 blog should be downgraded to F6-library-only** (L1's INVERT-priority recommendation). Out of scope for this prep package; this is a Position B revision agent + 15-critic-synthesis question that Ben already engages with the Position B blog-revision agent on.
- **Whether Benten should defer publication entirely until post-LAMPS-RFC** (L11's standards-process-conservative position). Out of scope.
- **Whether the broader F2-F10 portfolio sequencing should shift** (L9 F4/F4a sequencing flip, etc.). Out of scope per night-shift HOLD list.

---

## Source citations

- Iroh PQ blog: WebFetch of https://www.iroh.computer/blog/iroh-post-quantum-handshakes 2026-05-26 — author Rüdiger Klaehn (`rklaehn`), published 2026-05-19.
- L3 critic JSON: `.addl/phase-4-meta/critic-lens-l3-iroh-maintainer.json` (`disposition: PARTIAL-CONCERN`; F5 −1 soft VETO; F10 +1 conditional Shape 5).
- L3 shape_5_recommended_amendment: two-stage outreach (T+1-2w bare framing + T+5-6w full draft + 4-week review) — adopted in Sequencing plan.
- L3 framing_corrections_required: 4 concessions (drop ephemeral-overlay; break out EndpointId; engage no-industry-consensus directly; acknowledge n0 defensible-under-uncertainty) — adopted in Framing-mistakes-to-avoid Group A.
- 15-critic synthesis: `.addl/phase-4-meta/r2-critic-15-synthesis-matrix.md` (L15 OpenPGP-PQC parallel; L8 PQXDH §4.8; L5 small-team-overreach).
- Content-addressed scan: `.addl/phase-4-meta/comment-opportunities-content-addressed-systems.md` §1 (3 iroh threads WATCH-ONLY-PENDING-SHAPE-5) + §6.3 ("iroh is the deliberate outlier on PQ-signatures").
- Iroh org public_members + per-user public profiles: `gh api orgs/n0-computer/public_members` + `gh api users/<login>` 2026-05-26.
- Iroh top contributors: `gh api repos/n0-computer/iroh/contributors` 2026-05-26.
- Filippo Valsorda non-relationship: WebSearch "Filippo Valsorda iroh n0-computer advisor cryptography post-quantum" 2026-05-26; zero results.
- CLAUDE.md disciplines applied: `feedback_no_defer_HARD_RULE`, `feedback_surface_arch_decisions_under_auth`, `feedback_review_finding_ground_truth_verify`, `feedback_industry_folklore_vs_spec_text_distinction` (pim-N candidate), `feedback_arm_schedulewakeup_on_ci_wait`-style timeline discipline.

*Drafted 2026-05-26 by Shape 5 iroh-outreach prep agent on branch `phase-4-meta-core/shape-5-iroh-outreach-prep`. Not a sent email; orchestrator + Ben gate publication via the Pre-send checklist + Decision gates.*
