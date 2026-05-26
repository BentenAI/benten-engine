# Position B blog revision-roadmap (sketch)

> Section-by-section revision-roadmap for the Position B blog draft based on universal-revision-roadmap from all 15 critic returns.
> Drafted 2026-05-26 as INPUT for the Position B revision agent (to be dispatched after L12 combiner choice ratifies + cryptographer-review-agent returns).
> **Combiner-section (Section 5) is PLACEHOLDER pending cryptographer-review-agent outcome.**

## Source documents

- Strategy plan + blog outline: [pq-hybrid-sig-public-stance-strategy.md](pq-hybrid-sig-public-stance-strategy.md) §3 (blog outline)
- 15 critic returns: `.addl/phase-4-meta/critic-lens-l{1..15}-*.json`
- 15-critic synthesis matrix: [r2-critic-15-synthesis-matrix.md](r2-critic-15-synthesis-matrix.md)
- Verified L12 evidence: see [NIGHT-SHIFT-2026-05-26.md](NIGHT-SHIFT-2026-05-26.md) for verify results

## High-level structural revisions

**L14 R-2 blog restructure** (RATIFIED in synthesis): from ~5000 words argument + 200 words library → **~1800 words leading with library + 30-line install/example + condensed argument**. Reorder sections so library is the first concrete artifact reader encounters; argument follows.

**L1 + L5 + L10 form-divergence on F6 RESOLVED**: in-tree-with-published-package. Library published as `benten-crypto-suite` to crates.io + `@bentenai/crypto-suite` to npm; security-response SLA = engine's SLA; source-of-truth in-tree.

**L6 R-2 class-boundary narrowing** (the SINGLE MOST-ELEGANT REVISION): replace "content-addressed long-term-persistence systems" with **"long-lived signatures delegating capability into automated trust chains without per-use human review"**. Resolves L6 + L15 + L1 simultaneously by excluding git/Sigstore (human-in-loop) while preserving Benten's plugin-manifest + AI-agent-autonomy + Atrium-class scope. Codify as the canonical class-boundary going forward.

**L15 R-1 cohort reframe**: replace "Benten is alone in shipped P2P deployment" with "Benten is first-shipping in a converging cohort" — cite draft-ietf-openpgp-pqc-17 + Sequoia PGP commitment + Sigstore retooling. This is a stance-strengthening not stance-weakening reframe.

## Blog outline (revised structure per L14 R-2)

| # | Section | Words (approx) | Source |
|---|---|---|---|
| 1 | Lede | 200 | unchanged |
| 2 | **The library + install + 30-line example** | 300 | NEW (L14 R-2) — moved up |
| 3 | Why we built it: the threat model (compressed) | 400 | condensed from original §3 |
| 4 | What we shipped + how (combiner section) | 500 | PENDING cryptographer-review |
| 5 | Honest caveats (load-bearing section) | 500 | EXPANDED from original 5 → 9 caveats |
| 6 | The class of systems this argues about | 300 | NEW (L6 R-2 narrowing + L15 R-3 class table) |
| 7 | What the industry is doing | 400 | REVISED with cohort + parallels |
| 8 | Where this leaves the standards bodies | 200 | condensed |
| 9 | What we'd like to see | 200 | unchanged |
| 10 | Acknowledgments | 100 | EXPANDED with L8 + L15 + L9 specific acks |

Total: ~3100 words (vs original ~5200; L14 R-2 compressed); library leads.

## Per-section revision instructions

### Section 1 — Lede (200 words)

**Keep**: One-line summary + "we may be wrong; here's our reasoning upfront" honest stance + pointer to library.

**Add**: Cohort framing — "first-shipping in a converging cohort that includes OpenPGP-PQC mandating same construction" per L15 R-1.

**Remove**: Any "alone in this position" language.

### Section 2 — The library + install + 30-line example (NEW, 300 words)

**New section per L14 R-2**.

Content:
- One-paragraph "what the library is" (Benten Crypto Suite — codepoint-dispatched PQ-hybrid signature crate; vendored agility seam)
- crates.io + npm install commands
- 30-line example: import + sign a message + verify a signature
- Pointer to README + tests + KAT vectors
- Source-of-truth in-tree at `crates/benten-crypto-suite/` (transparency about derivative-artifact relationship)

Per L10 framing additions: include SECURITY.md pointer + response-SLA pointer + RUSTSEC-feed monitoring statement.

### Section 3 — Why we built it: the threat model (condensed, 400 words)

**Compressed from original §2.1** (was ~800 words).

The asymmetry argument MUST survive 11 critics' revision requests. Key changes:

- **L6 R-2 class-boundary narrowing**: use the new boundary text throughout
- **L8 R-2 + R-3 + R-4**: cite PQXDH §4.8 deniability grounds explicitly when distinguishing from messaging; reframe as "Signal's design goals are INTENTIONALLY orthogonal" not "Signal hasn't done this yet"
- **L15 R-2 OpenPGP-PQC parallel**: cite draft-ietf-openpgp-pqc-17 as parallel example of long-lived-signatures-on-persistent-artifacts class
- **L3 R-3-1 + R-3-2**: engage with iroh's actual reasoning (a) no-HNDL-equivalent-for-sigs + (b) no-industry-consensus; argue against (a) for our specific use case (delegating-capability-into-automated-trust-chains) WHILE acknowledging (a) is correct for iroh's transport-identity use case
- **L3 R-3-3**: REMOVE the simplistic "iroh = ephemeral session" classification; iroh-blobs IS content-addressed long-term-persistence; the relevant distinction is transport-identity vs persistent-attestation
- **L7**: include concrete numeric mention (53× expansion; 3409B sig)

### Section 4 — What we shipped + how (combiner section, 500 words) **PLACEHOLDER**

**HOLD until cryptographer-review-agent (`a694fcec`) returns.** This section depends entirely on the L12 combiner choice + the chosen construction's formal-property scope.

Structure (pending):
- The chosen combiner construction (LAMPS-with-revoke-by-tuple OR Bird-of-Prey OR alternative)
- Codepoint layout (single arm OR swap-matrix multiple arms)
- Wire-size honesty (concrete numbers)
- Formal-property scope (EUF-CMA / SUF-CMA / WNS / SNS / BUFF) explicit
- Standards posture (LAMPS WG status / Bird-of-Prey paper / draft-prabel WG status)

Cryptographer-review-agent's findings will produce specific language for this section.

### Section 5 — Honest caveats (load-bearing, 500 words)

**Expanded from 5 caveats to 9** per critic synthesis.

1. **Cohort framing** (L15 R-1 replacement for original "alone" caveat): "We're first-shipping in a converging cohort that includes OpenPGP-PQC (draft-ietf-openpgp-pqc-17 mandating same composite Ed25519+ML-DSA-65, expected H1-2026); Sigstore + Trail of Bits retooling on this threat-model; et al."

2. **Standards maturity** (L11 framing): "LAMPS draft is in RFC Editor Queue. We're treating it as the v1-beta default within the RFC 7120 framework supporting pre-RFC production deployment when WG has converged on technical content."

3. **Implementation maturity** (L10 EXPANDED): "ml-dsa Rust crate is not independently audited. Our security-response SLA matches the engine's. We monitor RUSTSEC feeds. SECURITY.md documents our threat model + response timeline."

4. **Different threat model from messaging** (L8 STRENGTHENED): "We are NOT arguing Signal/iMessage/MLS should adopt this. PQXDH §4.8 explicitly rejected PQ identity sigs on DENIABILITY grounds — Signal's reasoning is INTENTIONALLY orthogonal to ours; different design goals; both defensible."

5. **Wire-size cost is real** (L7 EXPANDED): "3409 bytes per LAMPS Composite ML-DSA hybrid sig vs Ed25519's 64 bytes (53× expansion). For Atrium drops carrying N signatures, multiplier compounds. Correct cost-tradeoff for our use case where signatures rest persistent; not all uses cases share this profile."

6. **Implementation-vs-algorithm distinction** (L2 NEW): "Hybrid combiner protects against algorithm-level failure of either component (Ed25519 OR ML-DSA), but does NOT protect against implementation-level side-channel leakage of either private key. CVE GHSA-hcp2-x6j4-29j7 (ml-dsa timing side-channel; patched in 0.1.0-rc.3, we pin 0.1.0) is the cautionary example."

7. **Combiner formal-property scope** (L12 NEW — PLACEHOLDER): explicit framing of what unforgeability + non-separability + BUFF properties the chosen construction provides + does NOT provide. Text depends on cryptographer-review outcome.

8. **Key-reuse hazard** (L12 NEW): cite LAMPS Sec §10.3 (or equivalent for chosen construction) about prohibition on key reuse between composite and single-algorithm contexts; explain Benten's enforcement.

9. **FIPS 140-3 outside scope** (L14 NEW): "Composite signatures are EXPLICITLY OUTSIDE FIPS 140-3 (NIST left composition to IETF). For FIPS-bound adopters, the LAMPS opt-in arm covers the pre-FIPS-composite landscape; Bird-of-Prey-class constructions are similarly outside-FIPS until adopted upstream."

### Section 6 — The class of systems this argues about (NEW, 300 words)

**New section per L6 R-2 + L15 R-3 + L3 R-3.**

Content:
- **Class definition** (L6 R-2): "Long-lived signatures delegating capability into automated trust chains without per-use human review"
- **Concrete inclusion list** (L15 R-3 table):
  - Benten Engine (us): UCAN delegations, plugin manifests, Atrium drops, sync merge proofs
  - OpenPGP-PQC (planned H1-2026): persistent OpenPGP-signed artifacts
  - Sigstore receipts (planned): code-signing receipts that bind transparency-log entries
  - Hypothetical: Willow Protocol attestations, Atrium-class CRDT-sync systems
- **Concrete exclusion list**:
  - git commits (human-in-loop at install/merge; not auto-capability-delegating)
  - Sigstore short-lived OIDC sigs (ephemeral)
  - Filecoin blockchain consensus sigs (signs rounds; not persistent artifacts)
  - Arweave RSA-PSS (locked into legacy; cautionary example not target)
  - TLS / QUIC / messaging (ephemeral session)
- Honest acknowledgment: this class boundary IS our argument's load-bearing structural claim; we welcome critique

### Section 7 — What the industry is doing (revised, 400 words)

**Revised from original §2 with cohort framing throughout.**

- Honest table showing per-system: iroh / Signal / Apple / MLS / libp2p / Veilid all keep transport-identity classical
- Quote iroh's "waiting for industry consensus to emerge" verbatim + acknowledge their use case differs from Benten's per L3 R-3-1
- Honest table showing converging cohort: OpenPGP-PQC mandating same composite (RFC H1-2026); Sigstore + Trail of Bits retooling on same threat-model; EBSI already squats `jwk_jcs-pub` (L4 finding)
- L11 RFC-7120 framing: pre-RFC production deployment is supported by IETF's own framework when WG has technical convergence

### Section 8 — Where this leaves the standards bodies (200 words)

Compressed from original. Updates:

- LAMPS RFC Editor Queue status (L11 framing precision)
- JOSE WG draft-ietf-jose-pq-composite-sigs-01 (active early-stage; F3 contribution candidate)
- Multicodec maintainer direction on container-form (PRs #400 #403; we endorse)
- W3C did:key emerging "any multikey allowed" consensus (L13 framing)
- IF cryptographer-review picks Bird-of-Prey: cite draft-prabel-cfrg-suf-hybrid-sigs INDIVIDUAL-SUBMISSION status precisely (L9 IETF-vocabulary discipline)

### Section 9 — What we'd like to see (200 words)

Keep open questions + invitations for critique. L3-influenced framing: explicitly invite iroh team + LAMPS WG + W3C CCG to critique.

### Section 10 — Acknowledgments (100 words)

**Expanded acknowledgments:**

- L8 acknowledgment: "Signal's PQXDH §4.8 design discipline (deniability-first) informed our framing of how content-addressed systems differ from messaging without implying critique of either"
- L15 acknowledgment: "draft-ietf-openpgp-pqc-17 working group + Sequoia PGP team + Sigstore team — for shipping/planning-to-ship the same construction independently; this work is in conversation with theirs"
- L9 acknowledgment: "LAMPS WG for the composite-sigs construction we're shipping under" (NO F4 cross-link per L9 R-3; pure acknowledgment)
- Critics list (anonymized; we don't reveal critic agents): "Thanks to internal-review participants for adversarial scrutiny of this draft"

## Revision agent dispatch — when to fire

**Wait until**:
1. Cryptographer-review-agent (`a694fcec`) returns → Section 5 (combiner section) language can be drafted
2. Ben ratifies L12 combiner choice → final framing locked
3. 5 ecosystem-scan agents return → updated industry-cohort data may inform Section 7 + Section 8

**Then dispatch Position B revision agent with this roadmap + the cryptographer-review findings + the synthesis matrix + the scan findings as input.** Brief instructs producing a REVISED full-draft blog post + a delta-from-original change-log so we can review what shifted.

## Revision agent will NOT do

- Not author the F2/F7 + comment-opportunity comments (separate workstreams)
- Not implement the combiner change (G-CORE-PQ-WIRE wave; separate)
- Not draft iroh-outreach email (Shape 5 outreach prep agent; separate)
- Not unilaterally choose the L12 combiner (cryptographer-review + Ben ratify; revision agent applies the choice)

## Pre-publication discipline (per planning agent §3 review checklist)

After revision agent produces draft:
1. Read-through by orchestrator-direct (full-depth quality pass)
2. Ben review + sign-off
3. External outside-Benten reader cold-review (per L5 sustainability framing; planning-agent §3 mandates)
4. Shape 5 iroh outreach BEFORE publish (per L3 strong-endorse framing)
5. Iroh response is decision-gate (substantive engagement → publish with co-author/co-sig ask; declines → defer F5 or solo per Shape 1 fallback)
6. F6 library beta-published one week before tag
7. Publish AT v1-beta tag (or per Shape 5 outcome) — within 48 hours of tag for momentum if going

---

*Drafted 2026-05-26. Combiner-section (Section 5) is PLACEHOLDER until cryptographer-review-agent returns. Other sections are revision-agent-ready input.*
