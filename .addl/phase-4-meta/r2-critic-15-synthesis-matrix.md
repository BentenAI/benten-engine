# 15-critic synthesis matrix — Position B + 7 PURSUE-set paths

> Synthesis of all 15 adversarial critic returns evaluating the public-stance portfolio (F2 / F3 / F4 / F5 / F6 / F7 / F10) for Benten's PQ-hybrid sig v1-beta tagging.
> Drafted 2026-05-26 from JSONs at `.addl/phase-4-meta/critic-lens-l{1..15}-*.json`.
> Authoritative for the Position B blog revision agent's input + the F2-F7 + newly-discovered comment-opportunity prioritization + arch-decision triage.

## Headline finding

**0 of 15 critics approve F5 blog as-drafted.** Every single one wants revision. Failure Mode E gate (4+ HIGH lenses strongly-negative) FIRED at L1 + L2 + L3 + L5 (round-1 HIGH); REINFORCED at L10 + L12 (round-2 substantive).

**Universal endorse**: F2 (multicodec PR endorsement) + F7 (W3C CCG comments) + F6 (open-source library) + F10 (iroh outreach with Shape-5-caveats).

**Form-divergence on F6 library**: most want standalone-canonical; L5 wants in-tree-with-published-package hybrid; L10 wants standalone-only-if-Benten-takes-security-SLA. Resolution: in-tree-with-published-package per L5 compromise = strictly stronger than either pure standalone or pure in-tree.

**New evidence STRENGTHENING Position B argument** (surprises that flipped expected attacks):
- **L15**: draft-ietf-openpgp-pqc-17 mandates SAME Ed25519+ML-DSA-65 composite (RFC H1-2026; Sequoia PGP committed ship-on-publication) → Benten's "alone" framing flips to "first-shipping in converging cohort"
- **L6**: TNFL (Trust-Now-Forge-Later) is 8-year-old NAMED threat class with industry literature (PQShield, postquantum.com, EverTrust, NIST circles, Josefsson-2024 SPHINCS+-for-git proposal) → "speculative" attack flips to "industry-recognized"
- **L4**: EBSI (European Blockchain Services Infrastructure) ALREADY squats `jwk_jcs-pub` (`0xeb51`) in production per W3C peacekeeper public comments → Benten's private-codepoint approach is governance-cleared
- **L8**: PQXDH §4.8 explicitly rejected PQ identity sigs on DENIABILITY grounds (NOT the wire-size folklore Benten's blog assumed) → Signal's reasoning is INTENTIONALLY orthogonal; strengthens Benten's "different system class" argument
- **L11**: RFC 7120 explicitly contemplates pre-RFC production deployment as legitimate → Benten's pre-RFC LAMPS shipping is supported by IETF's own framework
- **L13**: emerging "any multikey allowed without spec amendment" consensus in W3C CCG #70 directly supports Benten's extensibility-without-spec-amendment posture

**New evidence we MUST honestly disclose**:
- **L2**: ml-dsa CVE GHSA-hcp2-x6j4-29j7 (timing side-channel; we're patched but bug-class lurks) + 0 audited RustCrypto PQ packages → implementation-vs-algorithm distinction must be honesty caveat #6
- **L5**: 5-week-old / 1-contributor / 0-star team-scale data anchors every "sustained engagement" / "security response" / "industry leadership" claim
- **L12 BOMBSHELL** (verified): LAMPS Composite ML-DSA is EUF-CMA-only NOT SUF-CMA; Benten's UCAN-revocation-by-sig-CID has real bypass; combiner-choice question is LIVE
- **L14**: composite sigs are EXPLICITLY OUTSIDE FIPS 140-3 (NIST left composition to IETF) → honesty caveat for FIPS-bound adopters

## Per-critic findings (1-2 sentence strongest finding each)

| # | Critic | Strongest finding |
|---|---|---|
| L1 | Multiformats-conservative (round 1 HIGH) | INVERT F5/F6 priority — META-MESSAGE of "small team shipped private-use codepoint + blogged" propagates regardless of blog framing-discipline; F6 library should be load-bearing, F5 downgraded to engineering-notes Position A |
| L2 | PQ-skeptic cryptographer (round 1 HIGH) | GHSA-hcp2-x6j4-29j7 + 0 audited RustCrypto PQ packages — Benten's "classical Ed25519 is audited security floor" defends algorithm-failure but NOT implementation-side-channel leakage of ML-DSA private key; require honesty caveat #6 |
| L3 | iroh-maintainer (round 1 HIGH) | Benten's "ephemeral session vs persistent artifact" is BENTEN'S INTERPRETIVE OVERLAY — iroh actually reasoned (a) no-HNDL-for-sigs + (b) no-industry-consensus; engage with iroh's REAL argument; Shape 5 outreach STRONG-ENDORSE if framed as "request for technical critique that may change whether we publish" NOT "heads-up before publishing in 2 weeks" |
| L4 | Ecosystem-fragmentation skeptic (round 1 HIGH) | Flip canonical/fallback DID rendering: did:jwk CANONICAL, Benten-private as wire-size-optimization arm; EBSI `jwk_jcs-pub` precedent legitimizes Benten's private-codepoint approach |
| L5 | Small-team-overreach (round 1 HIGH) | 5-week-old / 1-contributor team-scale anchors EVERY claim; STRONG OBJECT to F5 + F6-standalone + broad F10; F6-in-tree-with-published-package compromise is the structural fix |
| L6 | HNDL-for-sigs skeptic (round 1 HIGH) | TNFL is 8-year named class; Benten's class-boundary "content-addressed long-term-persistence" is too broad (includes git/Sigstore/Filecoin/Arweave that DON'T adopt); narrow to "long-lived sigs delegating capability into automated trust chains without per-use human review" → EXCLUDES git/Sigstore, matches Benten's actual use case |
| L7 | Wire-size pragmatist (round 2 MEDIUM) | 3409B sig vs 64B Ed25519 (53× expansion) is real cost; Benten's claim "wire-size cost is real" caveat #5 is sufficient HONESTY but blog should add concrete numeric breakdown + show Drop bundle multiplier scenarios |
| L8 | Signal-protocol-veteran (round 2 MEDIUM) | Persona's expected attack COLLAPSED — PQXDH §4.8 deniability grounds (NOT wire-size folklore); 3 specific §2.1 revisions to incorporate Signal's REAL reasoning; STRENGTHENS Benten's "different design goals" framing |
| L9 | LAMPS-WG-participant (round 2 MEDIUM) | F4 LAMPS adopter STRONGLY POSITIVE but: NOT during AUTH48 window; explicit conformance commitment; **test-vector deposit is highest-leverage adopter contribution** (planning agent missed); no F5 cross-link; INVERT F4/F5 priority; **F4a test-vector deposit by v1-beta-tag = NEW slate entry** |
| L10 | Audit-discipline skeptic (round 2 MEDIUM) | F6 NEUTRAL-LEANING-NEGATIVE (DIVERGES from L2 +) — standalone library extends Benten's security-response liability; 5 framing additions ABOVE L2's; SECURITY.md / response-SLA / RUSTSEC-feed monitoring contract above strategy-doc scope |
| L11 | Standards-process-conservative (round 2 MEDIUM) | Persona's attack PARTIALLY COLLAPSED — RFC 7120 contemplates pre-RFC production deployment; single highest-impact change: sequence F4 to POST-LAMPS-RFC-publication, NOT v1-beta-tag (resolves 60% of objection) |
| L12 | Combiner-soundness cryptographer (round 2 LOW → HIGH after evidence-pass) | **BOMBSHELL**: LAMPS Composite ML-DSA EUF-CMA-only NOT SUF-CMA; "applications where SUF-CMA critical SHOULD NOT use Composite ML-DSA" per LAMPS draft itself; Bird-of-Prey paper proposes SUF-CMA-preserving combiner for EdDSA+ML-DSA with SMALLER sigs; draft-prabel-cfrg-suf-hybrid-sigs as IETF response; combiner-choice question LIVE |
| L13 | W3C DID-method skeptic (round 3 LOW) | DROP Benten-private `did:key` URI rendering ENTIRELY — generic resolvers will silently misparse `did:key:z<Benten-private-codepoint || ...>`; keep `SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001` as wire-format-INTERNAL only; pre-v1-beta-tag mandatory per critical-sequencing-constraint |
| L14 | Adoption-friction skeptic (round 3 LOW) | BouncyCastle 1.79+ ALREADY SHIPS LAMPS COMPSIG (all 18 OIDs) — enterprise-Java friction lower than expected; Rust/TS/Go/Python all primitives-only; **~150-250 LOC PR to paulmillr/noble-post-quantum hybrids.js** = "Benten lands a PR" not "Benten maintains a port"; FIPS 140-3 outside-scope caveat #6 needed; R-2 blog restructure leading with library |
| L15 | Threat-model-boundaries (round 1 HIGH) | **draft-ietf-openpgp-pqc-17 mandates SAME composite as Benten** (RFC H1-2026); Sigstore + Trail of Bits independently retooling on same threat-model; per-system class-boundary: Filecoin EXCLUDED / Arweave INCLUDED-but-cautionary-example / git+OpenPGP INCLUDED-WG-active / Sigstore INCLUDED-retooling-active — flips "alone" to "first of converging cohort"; REVISE-MAJOR (3 revisions); NOT veto |

## Per-path × per-critic verdict matrix

Format: ✅ = strongly+ / + = endorse / · = neutral / − = downgrade / ✗ = veto. NEW = new slate entry surfaced by critic.

| Critic | F2 multicodec | F3 JOSE WG | F4 LAMPS adopter | F5 blog | F6 library | F7 W3C CCG | F10 iroh outreach |
|---|---|---|---|---|---|---|---|
| L1 multiformats | ✅ | + | + | − DOWNGRADE | ✅ | + | + |
| L2 PQ-skeptic | + | + | · | − (revise OR downgrade) | + | + | + |
| L3 iroh-maintainer | + | + | + | − soft-VETO (4 framing concessions) | + (Shape 5 STRONG-ENDORSE) | + | + (Shape 5 STRONG-ENDORSE if framed right) |
| L4 ecosystem | + | · | + | − conditional | ✅ | ✅ | + |
| L5 small-team | + | · narrower scope | · narrower scope | ✗ STRONG OBJECT | − (standalone) / + (in-tree-with-published-package) | + | · iroh-only acceptable |
| L6 HNDL-skeptic | + | + | + | + PURSUE-BUT-REVISE §2.1 narrowing | + | + | + |
| L7 wire-size | + | + | + | + REVISE add numeric breakdown | + | + | + |
| L8 Signal-veteran | + | + | + | + REVISE §2.1 PQXDH §4.8 framing | + | + | + |
| L9 LAMPS-WG | + | + | ✅ but NOT-during-AUTH48 + NEW F4a test-vectors | − (NO F5 cross-link from F4) | + | + | + |
| L10 audit-discipline | + | + | · | − (5 framing additions) | · LEANING-NEG standalone-liability concern | + | + |
| L11 standards-process | + | · | + REVISE sequence to POST-RFC | − (RFC-7120 framing) | + | + | + |
| L12 combiner | + | + | + | − REVISE-MAJOR (combiner section + caveats #7 #8) + COMBINER CHOICE | + | + | + |
| L13 W3C DID | + | + | + | + REVISE drop did:key URI rendering | + | ✅ | + |
| L14 adoption-friction | + | + | + | + REVISE R-2 restructure lead-with-library + caveat #6 FIPS-140-3 | ✅ + noble-PR distribution shift | + | + |
| L15 threat-model | ✅ | + | + | + REVISE-MAJOR 3 revisions (incl OpenPGP-PQC parallel + per-system class boundary) | + | + | + |
| **TOTAL** | **15/15 endorse** | **13/15 endorse 2 neutral** | **13/15 endorse 1 neutral 1 conditional** | **0/15 approve as-drafted; 11 revise; 4 strong-negative** | **13/15 endorse 1 neutral 1 negative-on-standalone-form** | **15/15 endorse** | **15/15 endorse 1 narrow-to-iroh** |

## 3 architectural decisions surfaced (load-bearing)

### 1. **L12 BOMBSHELL — v1-beta combiner choice**

LAMPS Composite ML-DSA is EUF-CMA-only NOT SUF-CMA. Benten's UCAN-revocation-by-sig-CID has malleability bypass under EUF-only.

Three options:
- **A**: Ship LAMPS-default + revoke-by-tuple hardening (sidesteps gap; broad interop; conservative)
- **B'**: Ship Bird-of-Prey-default + LAMPS-opt-in (Ben's tentative preference; stronger leadership stance; takes academic-crypto-in-prod implementation risk)
- **C**: Bird-of-Prey-only (most-credible-stance but sacrifices LAMPS interop entirely; non-WG-adopted construction as sole default)

Cryptographer-review-agent (`a694fcec`) evaluating; recommendation may overturn Ben's tentative B'. Awaiting return.

### 2. **L1+L4+L13 convergence — DID surface**

Drop Benten-private `did:key` URI rendering ENTIRELY at public DID surface. Render `did:jwk` as canonical (per L4 R-1). Keep `SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001` as wire-format-INTERNAL only (per L13 R-13-1).

L13 wire-format-untouched claim VERIFIED via grep: rendering doesn't exist in code yet; R0-plan edit only, zero code-impact.

Ben ratified L13 R0-plan edit ("plan edit sounds fine"); to be batched with other R0-plan edits after L12 lands.

### 3. **L9+L11 F4 sequencing flip + NEW F4a**

L11: F4 LAMPS adopter report posting pre-LAMPS-RFC-publication is wrong-timing (RFC 7120 supports pre-RFC production deployment ≠ adopter report DURING AUTH48). Defer F4 to post-RFC.

L9: NEW slate entry F4a = test-vector deposit to LAMPS WG by v1-beta-tag. "Highest-leverage adopter contribution in the entire slate"; planning agent missed.

Ben holding both pending L12 settle.

## Universal-revision-roadmap for F5 blog

Aggregating every text change requested by every critic. Position B blog revision agent's input.

### Section 2 (the asymmetry argument)
- **L6 R-2**: narrow class-boundary from "content-addressed long-term-persistence" to **"long-lived sigs delegating capability into automated trust chains without per-use human review"** — excludes git/Sigstore (human-in-loop); matches Benten plugin-manifest + AI-agent autonomy use cases. **Single most-elegant revision; resolves L6 + L15 + L1 simultaneously.**
- **L15 R-1**: replace "Benten is alone" with **"Benten is first-shipping in a converging cohort"** + cite draft-ietf-openpgp-pqc-17 + Sequoia PGP + Sigstore retooling
- **L15 R-3**: per-system class boundary table (Filecoin EXCLUDED / Arweave INCLUDED-but-cautionary / git+OpenPGP INCLUDED-active / Sigstore INCLUDED-active) instead of broad "content-addressed systems"
- **L3 R-3-1+R-3-2+R-3-3+R-3-4**: engage with iroh's ACTUAL reasoning (no-HNDL-for-sigs + no-industry-consensus) rather than strawman; remove "ephemeral session" classification of iroh-blobs (incorrect; iroh-blobs IS content-addressed long-term-persistence)
- **L8 R-2 + R-3 + R-4**: cite PQXDH §4.8 deniability grounds explicitly; reframe Signal's reasoning as INTENTIONALLY orthogonal (different design goals); 3 specific §2.1 paragraph rewrites
- **L12 R-1 + R-2 + R-3**: rewrite §2.1 "strip-resistance" bullet to remove LAMPS-attached qualifiers + add SUF-CMA-vs-EUF-CMA explanation + cite LAMPS Sec Considerations actual text

### Section 5 (what we shipped)
- **L12 R-4 + R-5 + R-6**: rewrite combiner-section based on cryptographer-review-agent's recommendation; cite specific construction with formal-property scope; remove "committing" / "strip-resistant" framing or qualify them with LAMPS-attached WNS-vs-SNS + EUF-vs-SUF distinction
- **L7**: add concrete numeric breakdown (3409B vs 64B; Drop bundle multiplier scenarios)
- **L14 R-1**: include FIPS 140-3 outside-scope acknowledgment (caveat #6)

### Section 6 (honesty caveats)
Original 5 caveats + new ones to add:

1. (original) Benten is alone in shipped P2P deployment → **L15 REVISE to "first-shipping in converging cohort"**
2. (original) LAMPS draft in RFC-Editor-Queue → **L11 PRESERVE but add RFC-7120 framing context**
3. (original) ml-dsa not independently audited → **L10 EXPAND security-response SLA + RUSTSEC-feed monitoring contract**
4. (original) Our threat model differs from messaging → **L8 STRENGTHEN with PQXDH §4.8 specific cite**
5. (original) Wire-size cost is real → **L7 EXPAND numeric breakdown**
6. **NEW (L2)**: Implementation-vs-algorithm distinction — combiner protects against algorithm failure NOT implementation-side-channel leakage
7. **NEW (L12)**: Combiner formal-property scope — explicit EUF-vs-SUF or WNS-vs-SNS framing for chosen construction (text depends on cryptographer-review-agent outcome)
8. **NEW (L12)**: Key-reuse hazard — cite LAMPS Sec §10.3 (or equivalent for chosen construction) about prohibition on key reuse between composite and single-algorithm contexts
9. **NEW (L14)**: Composite sigs are EXPLICITLY OUTSIDE FIPS 140-3 (NIST left composition to IETF) — relevant for FIPS-bound adopters

### Section 9 (the library)
- **L14 R-2 blog restructure**: from ~5000 words argument + 200 words library → ~1800 words leading with library + 30-line install/example + condensed argument
- **L1 + L5 + L10 form-divergence**: in-tree-with-published-package per L5 (resolves L10 security-SLA concern; resolves L1 META-message concern; respects L5 team-scale)

### Section 10 (acknowledgments)
- Cite L8 PQXDH design discipline acknowledgment (Signal's reasoning AS-IS, not strawmanned)
- Cite L15 OpenPGP-PQC + Sequoia PGP + Sigstore retooling parallel
- Cite L9 LAMPS WG (no F4 cross-link per L9 R-3; pure acknowledgment)

## F6 library form decision (resolved per critic synthesis)

**In-tree-with-published-package** wins. Reasoning:
- L5 STRONG OBJECT to standalone-form: 1-human team can't sustain standalone security-response SLA
- L10 NEUTRAL-LEANING-NEGATIVE to standalone: extends security-response liability
- L1 STRONGLY+ on F6 generally (load-bearing public artifact); doesn't specify form
- L4 ✅ on F6 generally
- L14 ✅ on F6 + RECOMMENDS PR to noble-post-quantum hybrids.js as distribution path

Resolution shape:
- Source-of-truth in-tree at `crates/benten-crypto-suite/` (where it's already implemented for v1-beta)
- Published packages: `benten-crypto-suite` to crates.io; `@bentenai/crypto-suite` to npm; both as derivative artifacts
- Security-response SLA = same as engine (resolves L10 + L5)
- Public adopters can use package; package + source are byte-equivalent
- L14 noble-PR opportunity: ALSO contribute LAMPS-COMPSIG to noble-post-quantum hybrids.js as separate distribution path (estimated ~150-250 LOC PR; "Benten lands a PR" rather than "Benten maintains a port")

## Pim-N candidates surfaced (cross-critic patterns)

1. **`feedback_extra_reflection_pass_for_elegant_permanent_shape`** — already codified 2026-05-25; L6 R-2 class-boundary narrowing is the exemplar (one revision closes L6 + L15 + L1)
2. **`feedback_industry_folklore_vs_spec_text_distinction`** — NEW candidate from L3+L8 finding-class: critic-personas reasoned from INDUSTRY FOLKLORE about why X system did Y (iroh "ephemeral session" reasoning; Signal "wire-size pre-keys-signed" reasoning); ACTUAL spec/blog text was different. Discipline: verify-by-primary-source any "system X chose Y because Z" claim before incorporating into Benten's framing
3. **`feedback_ietf_vocabulary_precision`** — NEW candidate from L9: when commenting on IETF specs / drafts, use precise WG-vocabulary ("WG document" vs "individual submission"; "adoption call" vs "WG-LC"; "AUTH48" vs "RFC publication"); folksy terminology marks Benten as not-WG-fluent + creates real comprehension gaps with IETF readers
4. **`feedback_planning_agent_anticipation_audit`** — NEW candidate from L9 finding "planning agent missed F4a test-vector deposit": post-planning-agent return, run discipline audit: did planning agent capture all the major opportunities a domain-expert critic would surface? L9's F4a miss + L14's noble-PR miss are two instances

## Convergence trajectory

7 first-round HIGH critics (L1+L2+L3+L4+L5+L6+L15) returned; Failure Mode E gate FIRED with 4 strongly-negative on F5 (L1+L2+L3+L5).

8 second-round MEDIUM/LOW critics (L7+L8+L9+L10+L11+L12+L13+L14) returned; convergence pattern: same Failure-Mode-E gate REINFORCED (L10+L12 added substantive); 3 NEW load-bearing findings (L8 PQXDH §4.8; L9 F4a test-vector deposit; L12 LAMPS EUF-only); 2 collapsed personas (L8+L11 attacks partially-collapsed on evidence).

Net: 15 critics; 0 approve F5 as-drafted; 11 want revision; 4 want downgrade; universal-revision-roadmap captured here.

## What this matrix is NOT

- Not a blog draft (Position B revision agent's job)
- Not a final-decision document (architectural decisions still await Ben + cryptographer-review)
- Not a comment-opportunities list (5 scan agents producing that separately)
- Not a closure of any critic; ALL findings tracked through Position B revision

---

*Drafted 2026-05-26 from `.addl/phase-4-meta/critic-lens-l{1..15}-*.json`. Will be referenced by Position B blog revision agent + R0-plan revision + Compromise #30 + CLAUDE.md #5 retense + dispatch-conventions pim-N codification.*
