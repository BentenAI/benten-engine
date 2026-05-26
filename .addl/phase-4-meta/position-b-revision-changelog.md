# Position B blog draft v2 — delta-from-original changelog

> Critic-by-critic disposition map for the v2 revision of the PQ-hybrid sig public-stance blog.
> Authored 2026-05-26 by Position B revision agent.
> Companion to: `position-b-blog-draft-v2.md`.
> Source critic JSONs: `.addl/phase-4-meta/critic-lens-l{1..15}-*.json`.
> Synthesis: `.addl/phase-4-meta/r2-critic-15-synthesis-matrix.md`.
> Cryptographer review: `.addl/phase-4-meta/cryptographer-review-bird-of-prey-vs-lamps.md`.

## Disposition vocabulary

Per HARD RULE 12, every critic finding gets exactly one of:

- **CLOSED** — the v2 draft addresses the finding directly. The closure mechanism is cited (section + verbatim text).
- **DEFERRED-NAMED-NOW** — finding is real but out-of-scope for this blog draft; deferred to a NAMED destination (issue / doc / phase) that already exists or is created concurrently.
- **DISAGREE-WITH-EXPLANATION** — the v2 draft consciously diverges from the critic's recommendation; reason is stated.

No "carry to next brief" / "Phase-N follow-up" / phantom destinations.

---

## High-level structural shape

The single most-elegant structural revision is **Inv-15 (the three-layer decomposition: identity = canonical-payload-CID; authentication = codepoint-dispatched signature; revocation = semantic tuple)** appearing as the load-bearing technical content of §4. This shape closes — at the same time — L12 (combiner soundness: EUF-CMA gap is closed at the application layer rather than via algorithm change), L1 (META-message: the load-bearing artifact is the engineering decomposition + library, not the rhetoric), and a substantive portion of L9 (the blog stands on technical merit, not on a cross-link from the LAMPS adopter report). It also positions §6's class boundary as a structural claim about the systems-side, with Inv-15 as the engineering-side closure — neither alone would be sufficient; together they make the argument defensible without requiring algorithm switch.

The second structural revision is **leading with the library** (§2 placed before the threat-model argument, per L14 R-2 + L1's "META-message" framing). This shifts the discoverable artifact from the blog rhetoric to the published code, KAT vectors, and SECURITY.md — which closes L5 (1-contributor sustainability), L10 (audit-discipline standalone-liability), and the bulk of L1 (legitimization risk via blog META-message).

The third structural revision is **the cohort reframe** (L15 R-1): "first-shipping in a converging cohort" replaces "alone." This reframes the entire argument from "small team taking a position the industry isn't taking" to "early adopter in an ecosystem that is converging." Closes L15 and substantively strengthens L11 (RFC 7120 framework support for pre-RFC production deployment).

---

## Per-critic disposition

### L1 — Multiformats-conservative maintainer (round 1 HIGH; F5 = −1)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L1-R-1 | Drop ALL mention of Benten-private multicodec codepoint from the blog body; render did:jwk + JWK composite as canonical Benten v1-beta DID surface | **CLOSED** | The v2 draft does NOT mention any Benten-private multicodec codepoint. The wire-format/codepoint discussion in §4 is about the multicodec/varsig codepoint `0x0001`, the IANA-allocated OID `1.3.6.1.5.5.7.6.48`, and the codepoint-dispatch internal seam Inv-15 wraps. No "Benten-private" framing appears. The L13 finding (drop did:key URI rendering with Benten-private codepoint entirely from the public surface) was independently ratified and is honored: the blog talks about wire-format codepoints `0x0001..0x0005` as internal dispatcher values within `benten-crypto-suite`, never as DID-method-public URI rendering. |
| L1-R-2 | Add explicit "why we did NOT mint a multicodec PR for the hybrid codepoint" paragraph citing PR #400 + PR #403 + lidel's "No multicodec update required" verbatim | **CLOSED** (partial) | §8 explicitly says: "Multicodec maintainers (vmx, lidel, rvagg, MarcoPolo, achingbrain) — PRs #400 and #403 land the container-form direction (pkix-pub / cose-key / jwk codepoints) rather than per-algorithm hybrid codepoints. We endorse this direction unconditionally; it is the right architectural answer to the PQ codepoint-proliferation problem and our v1-beta wire format is structured to interop with it." We did NOT quote lidel's verbatim line because we did not source-verify it within the session; doing so before publication is named in the publication-discipline checklist appended below. |
| L1-R-3 | Replace "alone" framing with "alongside IETF-RFC-track LAMPS" framing; position above TSP-style pre-RFC squat | **CLOSED** | The lede (§1) and §7 both reframe explicitly to "early-shipping in a converging cohort" per L15 R-1 (same disposition; resolves both L1 and L15 simultaneously). §4 explicitly distinguishes the LAMPS WG-adopted Schelling-point construction from non-WG-adopted alternatives (draft-prabel individual submission; Bird-of-Prey one-publication-in-proceedings-prep). No TSP-style framing appears. |
| L1-R-4 | In §8/§9 "what we'd like to see," explicitly call for did:jwk consumers to add LAMPS alg-id support; do NOT call for multicodec to add hybrid per-algorithm codepoints | **CLOSED** | §8: "Multicodec maintainers… We endorse this direction unconditionally; it is the right architectural answer to the PQ codepoint-proliferation problem." §9: no ask for per-algorithm multicodec entries. |
| L1-R-5 | Add Acknowledgments line crediting multicodec maintainers (lidel, vmx, MarcoPolo, rvagg, achingbrain) for landing the container-form direction | **CLOSED** | §10 first paragraph: "The multicodec maintainers (vmx, lidel, MarcoPolo, rvagg, achingbrain) landed the container-form direction in PRs #400 and #403, which makes adopter approaches like ours possible without further codepoint-table proliferation." |
| L1-R-6 | DROP "dual-codepoint hedge" framing entirely | **CLOSED** | The v2 draft does NOT use "dual-codepoint hedge" language anywhere. §4 talks about codepoint `0x0001` (the v1-beta default) and the reserved `0x0002` (future SUF-CMA-preserving), `0x0003` (classical Ed25519 for legacy interop), `0x0004` (NF-1 PQ⊕PQ reserved), `0x0005` (reserved) — but these are framed as the codepoint-dispatched agility seam, not as a "hedge" between two equivalent options. |
| L1 cross-path | Make F6 library the load-bearing artifact; downgrade F5 blog | **CLOSED** (structurally) | §2 (the library) is placed before the threat-model argument per L14 R-2; the library is the first concrete artifact the reader encounters. This is the L14-R-2 + L1-cross-path structural concession both critics asked for. F5 is not "downgraded" but it is no longer the front-of-funnel artifact; the library is. |

### L2 — PQ-skeptic cryptographer (round 1 HIGH)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L2-R-1 | Honesty caveat #6: implementation-vs-algorithm distinction; cite GHSA-hcp2-x6j4-29j7 specifically | **CLOSED** | §5 caveat 6: "The hybrid combiner protects against an *algorithmic* break of either component. It does **not** protect against implementation-side-channel leakage — `RUSTSEC-2025-0144`[^rustsec-0144] (Jan 2026, `ml-dsa` Decompose timing) and the Verification Theatre paper's 13 libcrux vulnerabilities[^vtheatre] (including two FIPS 204 verifier spec violations) are the recent reminders." The cited advisories are RUSTSEC-2025-0144 + IACR 2026/192; we did NOT cite GHSA-hcp2-x6j4-29j7 by that specific identifier because the cryptographer review and L2 itself differ on the canonical advisory ID — both RUSTSEC-2025-0144 (RustSec) and GHSA-5x2r-hc65-25f9 (GitHub) refer to the `ml-dsa` family; we picked the RustSec-canonical identifier and added the libcrux GHSA family via the Verification Theatre paper. |
| L2 RustCrypto-PQ-audit-state | Acknowledge no audited RustCrypto PQ packages | **CLOSED** | §5 caveat 3 verbatim: "Neither `ml-dsa` (RustCrypto) nor `libcrux-ml-dsa` nor `fips204` has had an independent third-party security audit comparable to `ed25519-dalek` or `ring`." |
| L2 alone-in-shipped-P2P-deployment | "alone" framing | **CLOSED** | Reframed to cohort per L15 R-1 (resolves jointly across L1 + L2 + L15). |

### L3 — iroh-maintainer (round 1 HIGH; F5 = soft VETO until 4 framing concessions land)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L3-R-3-1 | Engage with iroh's ACTUAL reasoning (no-HNDL-for-sigs + no-industry-consensus) | **CLOSED** | §3 paragraph 2 quotes the iroh blog verbatim and engages BOTH cited reasons: "The second [reason] — no industry consensus — is real (we picked LAMPS Composite ML-DSA precisely because it is the *emerging* consensus the OpenPGP-PQC RFC will ratify; "emerging," not "settled"). The first — no HNDL equivalent for signatures — we extend rather than dispute: it is correct for a transport-identity key authenticating a *live* handshake, and it does not apply to a UCAN that authorizes an automated capability lookup in 2046." |
| L3-R-3-2 | Argue against (a) [no-HNDL-for-sigs] for our specific use case (delegating-capability-into-automated-trust-chains) WHILE acknowledging (a) is correct for iroh's transport-identity use case | **CLOSED** | Same §3 paragraph closes with: "iroh's `EndpointId` IS a long-lived Ed25519 identity… So our argument *does* apply to that one surface. Reasonable people can read it as friendly disagreement on that single point. We are not saying iroh got it wrong. We are saying we read the same evidence and made a different defensible call for the kind of artifact we sign." |
| L3-R-3-3 | REMOVE the simplistic "iroh = ephemeral session" classification | **CLOSED** | §3 explicitly states: "iroh-blobs is itself content-addressed long-term persistence, with BLAKE3 giving blobs integrity without per-blob signatures; the long-lived classical signing surface in iroh is narrowly `EndpointId`." §7's table shows iroh's row as "`EndpointId` (Ed25519)" not "ephemeral session." |
| L3-R-3-4 | Drop or substantially rewrite the asymmetry table row classifying iroh as ephemeral | **CLOSED** | §7's table treats iroh as: long-lived sig surface = `EndpointId` (Ed25519); PQ-hybrid sig today? = No (KEM is PQ-hybrid); stated reasoning = the verbatim quote. No "ephemeral" classification. The original asymmetry-table structure has been replaced by §7's full snapshot table. |
| L3 "we are NOT critiquing your call" disclaimer | Stop pretending the argument doesn't apply to iroh's EndpointId | **CLOSED** | §3 directly: "our argument *does* apply to that one surface, and reasonable people can read it as a friendly disagreement on that single point." The new framing is "friendly disagreement on a single point" rather than "we are not critiquing." |
| L3 shape_5_response (re: F10 outreach posture) | Posture (a) "we want your critique BEFORE we publish; we may not publish" not posture (b) "heads-up, publishing in 2 weeks" | **DEFERRED-NAMED-NOW** | The blog draft does not control F10 outreach posture; the F10 outreach is a separate workstream (Shape 5 iroh outreach prep agent brief at `.addl/phase-4-meta/shape-5-iroh-outreach-prep-agent-brief.md` already addresses this). §9 of the blog includes "Specific invitations: the iroh team for friendly disagreement on the EndpointId surface" but does NOT pre-commit to publication timing relative to iroh's response — that gate sits with the Shape 5 outreach plan and Ben's call. |

### L4 — Ecosystem-fragmentation skeptic (round 1 HIGH)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L4-R-1 | Flip canonical/fallback DID rendering: did:jwk CANONICAL, Benten-private as internal-only | **DEFERRED-NAMED-NOW** | The DID-rendering choice is a separate R0-plan edit (already ratified by Ben per the synthesis matrix §"3 architectural decisions surfaced #2") — destination is the R0 plan-doc revision-history entry, not this blog. The blog does NOT use any Benten-private did:key URI rendering anywhere (Inv-15's three-layer architecture leaves DID-surface rendering separable from wire-format codepoints). |
| L4 EBSI `jwk_jcs-pub` precedent legitimizes Benten's approach | Cite governance precedent | **CLOSED** | §5 caveat 1: "EBSI's `jwk_jcs-pub` (`0xeb51`) squat is governance-precedent for our private-codepoint approach." Also §7 table row. |

### L5 — Small-team-overreach skeptic (round 1 HIGH; F5 = ✗ STRONG OBJECT)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L5 1-contributor/5-week-old team scale | Anchor every sustainability claim | **CLOSED** | §5 caveat 3: SECURITY.md + 48-hour SLA + RustSec subscription is the published commitment; §10 acknowledgments name the cohort (OpenPGP-PQC WG + Sequoia + Sigstore + Trail of Bits + multicodec maintainers) rather than positioning Benten as the leader. The "early adopter, not leader" framing in §7 + lede explicitly absorbs the team-scale critique. |
| L5 F6 form: in-tree-with-published-package (vs standalone) | Resolve form-divergence | **CLOSED** | §2: "in-tree at `crates/benten-crypto-suite/` and published as a derivative artifact to both crates.io and npm. Both packages are byte-equivalent to the in-tree source; the engine itself depends on the in-tree path." Security-response SLA matches the engine's SLA (caveat 3). |
| L5 F5 STRONG OBJECT (publish at all) | Defer F5 vs ship-without-blog | **DISAGREE-WITH-EXPLANATION** | The decision to publish F5 vs defer is Ben's call (held post-cryptographer-review-ratification). This draft is the substantive revision that addresses every critic finding; whether to publish + when remains a Ben gate (per the publication-discipline checklist in the revision roadmap §"Pre-publication discipline"). The disagreement: even with team-scale honesty, the argument IS defensible and the cohort IS forming — withholding the blog entirely undersells the engineering work the project has done and forfeits the value of being early in a converging cohort. The L5-acceptable version of the blog (lead with library, cohort framing, deep caveats, no leadership-claim) is what this v2 draft is. |

### L6 — HNDL-for-sigs skeptic (round 1 HIGH)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L6-R-2 | Narrow class-boundary from "content-addressed long-term-persistence" to "long-lived sigs delegating capability into automated trust chains without per-use human review" — excludes git/Sigstore | **CLOSED** | The new boundary text is used **canonically** across §3 (paragraph 1 verbatim definition), §5 caveat 4 (PQXDH coherence), §6 (full inclusion/exclusion lists), §7 (industry snapshot). git is explicitly in §6's exclusion list ("human-in-loop attention at install/merge"); Sigstore short-lived OIDC sigs are exclusion-listed; Sigstore receipts are inclusion-listed (because they bind transparency-log entries — the long-lived artifact). |

### L7 — Wire-size pragmatist (round 2 MEDIUM)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L7 concrete numeric breakdown | Add 3373B vs 64B + Drop bundle multiplier | **CLOSED** | §4 paragraph 1: "The signature size is **3373 bytes** (ML-DSA-65 component 3309 bytes + Ed25519 component 64 bytes)." §5 caveat 5: "3373 bytes per signature (~52.7× Ed25519)… A 10-signature drop bundle adds ~33 KiB of signature material — significant for small payloads, negligible for large." |
| L7 acknowledge shape-of-deployment dependence | Not universally favorable | **CLOSED** | §5 caveat 5 closes: "The cost-tradeoff is shape-of-deployment-dependent." Also §3 paragraph 2 re iroh: argument extends to delegating-into-trust-chains use case but not transport-identity. |

### L8 — Signal-protocol-veteran (round 2 MEDIUM)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L8-R-2 | Cite PQXDH §4.8 deniability grounds explicitly | **CLOSED** | §3 paragraph 4 quotes PQXDH §4.8 verbatim (the full "It is tempting to consider adding a post-quantum identity key… designated verifier signature… deniable mutual authentication" passage). §5 caveat 4 names the deniability framing as the canonical interpretation. |
| L8-R-3 | Reframe Signal's reasoning as INTENTIONALLY orthogonal (different design goals; both defensible; not "deferral") | **CLOSED** | §3 paragraph 5: "Signal's reason is *deniability*… a property Benten **explicitly does not want**: plugin manifests, UCAN delegations, and drops are intended to be third-party non-repudiable — that is the point of signing them. Signal's reasoning and ours both serve their respective systems. They do not contradict; they apply to disjoint design goals." §5 caveat 4 strengthens: "we cite Signal as a *coherence* case rather than a deferral example." |
| L8-R-4 | Three specific §2.1 revisions to incorporate Signal's REAL reasoning | **CLOSED** | The L8-recommended asymmetry-table split (Signal-private-1:1 / Signal+CKV / iMessage+CKV / MLS-group) is partially absorbed — §7's snapshot table distinguishes Signal (PQXDH), Apple iMessage (CKV identity keys), and MLS-default as separate rows. We did NOT do the full 4-row split because the relevant property for §7's table is "PQ-hybrid sig today? + stated reasoning"; the deniability framing is captured at §3 paragraph 4-5 and §5 caveat 4 where it is most load-bearing. If the iroh / Signal review process surfaces a stronger ask for the explicit 4-row split, that is an additive edit pre-publication. |
| L8 Signal team review pre-publication | Send blog to Trevor Perrin / Rolfe Schmidt / PQXDH editors for technical-correctness review | **DEFERRED-NAMED-NOW** | Destination = §9 ("Specific invitations: …the Signal cryptography team for confirmation that we have characterized PQXDH §4.8 correctly") — this is the public invitation. The private pre-publication review request to the Signal team is part of the F10 outreach plan + Shape 5 / publication-discipline checklist; not in this blog draft's scope but explicitly named in the published §9. |

### L9 — LAMPS-WG-participant (round 2 MEDIUM)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L9-R-1 | F4 LAMPS adopter report NOT during AUTH48 window | **CLOSED** (named in blog) | §8 explicitly: "LAMPS WG — draft-ietf-lamps-pq-composite-sigs-19 is in the **RFC Editor Queue**. We will file an adopter report **after** RFC publication (not during AUTH48), per RFC-process discipline." |
| L9-R-2 | Explicit conformance commitment to RFC-track | **CLOSED** | §5 caveat 2: "We have committed to track every revision through publication and to RFC-conform on publication." |
| L9-R-3 | NO F5 cross-link from F4 LAMPS adopter report | **CLOSED** | §8: "We will not cross-link the LAMPS adopter report to this blog post; the LAMPS contribution stands on its own technical merit and the blog stands on its argument." |
| L9 NEW slate entry F4a (test-vector deposit) | Highest-leverage adopter contribution | **CLOSED** (named in blog) | §8: "We will deposit Benten-derived test vectors into the LAMPS WG's vectors repository as soon as we have a stable pin against draft-19's PASN.1 module; this is the highest-leverage adopter contribution we can make." |
| L9 IETF-vocabulary precision | Use precise WG-vocabulary | **CLOSED** | §8 uses precise vocabulary throughout: "RFC Editor Queue" (not "RFC-ready"); "WG document" / "individual submission" (CFRG draft-prabel framed as the latter); "AUTH48" named; "early-allocated"; "WG-adopted" vs not. |

### L10 — Audit-discipline skeptic (round 2 MEDIUM)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L10 5 framing additions for F6 standalone | SECURITY.md / response-SLA / RUSTSEC-feed monitoring contract | **CLOSED** | §2: "`SECURITY.md` documenting the bug-report channel + response SLA (matching the engine's SLA)" + "We subscribe to RustSec advisories for `ml-dsa`, `libcrux-ml-dsa`, and `fips204`, and have committed publicly to triage any new advisory within 48 hours." §5 caveat 3 strengthens with NF-2 / C-GM-AUDIT as a written GM-tag gating exit criterion. |
| L10 F6 standalone vs in-tree-with-published-package | Resolved at synthesis level | **CLOSED** | §2: "in-tree at `crates/benten-crypto-suite/` and published as a derivative artifact to both crates.io and npm. Both packages are byte-equivalent to the in-tree source." |

### L11 — Standards-process-conservative (round 2 MEDIUM)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L11 RFC 7120 framing | Pre-RFC production deployment supported by IETF framework | **CLOSED** | §5 caveat 2: "We rely on RFC 7120's framework[^rfc7120] which explicitly contemplates pre-RFC production deployment when a WG has reached technical convergence." §7 closing paragraph: "The RFC 7120 framework supports pre-RFC production deployment of an IANA-early-allocated codepoint when the WG has converged on technical content." |
| L11 F4 sequencing to POST-RFC | Sequence F4 to post-LAMPS-RFC | **CLOSED** | §8 explicitly: "We will file an adopter report **after** RFC publication (not during AUTH48), per RFC-process discipline." |
| L11 IETF-vocabulary precision | Precise WG-vocabulary throughout | **CLOSED** | See L9 disposition. |

### L12 — Combiner-soundness cryptographer (round 2 LOW → HIGH after evidence pass)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L12-R-1 | Rewrite §2.1 "strip-resistance" bullet — use LAMPS's actual qualifiers (WEAK non-separability, not "strip-resistant" unqualified) | **CLOSED** | §4 quotes LAMPS §9.2 + §9.2.2 verbatim; the only place "strip-resistance" appears is implicit in the §4 paragraph "It is **not SUF-CMA secure**: an adversary who has seen one valid composite signature on a message *may* be able to produce a different valid composite signature on the same message from the same key. And per the draft itself, it is **weakly non-separable** but not strongly non-separable." The qualifiers LAMPS itself attaches to those terms are reproduced. |
| L12-R-2 | Add SUF-CMA-vs-EUF-CMA explanation | **CLOSED** | §4 ("The formal-property scope" subsection) + §5 caveat 7 both surface this distinction explicitly. |
| L12-R-3 | Cite LAMPS Security Considerations actual text verbatim | **CLOSED** | §4 quotes §9.2 + §9.2.2 verbatim including the LAMPS-emphasis "NOT RECOMMENDED for use in applications where it has not been shown that EUF-CMA is acceptable." |
| L12-R-4 | Rewrite combiner-section based on cryptographer-review-agent's recommendation; cite specific construction with formal-property scope | **CLOSED** | §4 fully populates the previously-placeholder combiner section: chosen construction (LAMPS Composite ML-DSA at `0x0001`, OID, IANA-allocated-2025-10-20), why LAMPS not alternatives (3-paragraph treatment of draft-prabel §3 / Bird-of-Prey / Silithium with specific reasons), formal-property scope (EUF-CMA + WNS verbatim from LAMPS), the **Inv-15 three-layer decomposition** as the application-layer closure. The cryptographer review's elegant-permanent-shape recommendation (identity = canonical-payload-CID; auth = codepoint-dispatched-sig; revocation = semantic-tuple) is verbatim adopted as the Inv-15 framework name and is the core technical content of §4. |
| L12-R-5 | Remove "committing" / "strip-resistant" framing OR qualify them | **CLOSED** | "Committing" and "strip-resistant" appear ZERO times in the v2 draft. The v2 uses precise LAMPS-aligned language throughout. |
| L12-R-6 (NEW honesty caveat #7 combiner formal-property scope) | Add caveat | **CLOSED** | §5 caveat 7 verbatim: "Combiner formal-property scope (EUF-CMA-only honesty). The construction is EUF-CMA secure and weakly non-separable. It is **not SUF-CMA secure**: LAMPS §9.2.2 verbatim, …" |
| L12 (NEW honesty caveat #8 key-reuse hazard) | Cite LAMPS §10.3 + Benten's enforcement | **CLOSED** | §5 caveat 8 verbatim: "LAMPS §9.3 and §10.3 are explicit that the underlying ML-DSA-65 and Ed25519 keys MUST NOT be reused for any standalone or other-composite purpose… `KeyPair::generate(SigCodepoint::Hybrid_Ed25519_MLDSA65)` mints the composite keypair as a single inseparable unit; component private keys are never exposed via Benten's public API. Implementers wrapping `benten-crypto-suite` MUST honor this restriction; the library README says so loudly." |
| L12 size-byte-discrepancy 3409 vs 3373 | Cite-grep-verify pre-publication; pick source-of-truth | **CLOSED** | The v2 draft uses **3373** consistently throughout (§4 paragraph 1, §5 caveat 5, §2 quickstart example assertion `assert_eq!(sig.len(), 3373)`). The cryptographer review confirmed 3373 = 3309 (ML-DSA-65) + 64 (Ed25519); the original "3409" figure was wrong. |
| L12 §8 explicit asks (IACR-peer-reviewed analysis + CFRG SUF-CMA-preserving adoption + PQUIP guidance) | Add to "what we'd like to see" | **CLOSED** | §9 ask #3: "An IACR-peer-reviewed analysis of the LAMPS composite-sig combiner at the depth of X-Wing's analysis of the hybrid KEM (Barbosa et al., IACR Communications in Cryptology 2024-1-21). The depth-asymmetry between hybrid-KEM analysis and hybrid-sig analysis is real and worth closing publicly." §8 closes the CFRG-adoption ask: "When (if) CFRG opens an adoption call for a SUF-CMA-preserving hybrid signature draft, we will support adoption and we will mint codepoint `0x0002` accordingly." |
| L12 algorithm-switch (Bird-of-Prey default) vs application-layer-fix | Recommended application-layer fix | **CLOSED** (via cryptographer review ratification) | §4 explicitly states the cryptographer review's recommendation: Inv-15 three-layer decomposition is the structural fix; algorithm change does not close the L12 hazards (because the same shortcut would recur under any algorithm); Inv-15 closes them permanently at the application layer with EUF-CMA as the cryptographic floor. The LAMPS-default decision is explicitly named as ratified per cryptographer-review-bird-of-prey-vs-lamps.md §1 + §9. |

### L13 — W3C DID-method skeptic (round 3 LOW)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L13-R-13-1 | Drop Benten-private `did:key` URI rendering entirely from the public DID surface | **CLOSED** (structurally) | The v2 blog does NOT use any Benten-private did:key URI rendering anywhere. Codepoint values appear only as wire-format-internal dispatcher constants within `benten-crypto-suite` (`SigCodepoint::Hybrid_Ed25519_MLDSA65` in the §2 code example), never as part of a DID URI. The R0-plan edit ratifying this is named in the synthesis matrix §3.2 and is a separate workstream (plan-doc revision-history, not blog scope). |

### L14 — Adoption-friction skeptic (round 3 LOW)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L14-R-1 | FIPS 140-3 outside-scope acknowledgment (caveat #6 / now #9) | **CLOSED** | §5 caveat 9 verbatim: "Composite signatures are **explicitly outside** the FIPS 140-3 boundary in current NIST guidance — NIST left composition to IETF. For deployments requiring FIPS 140-3 module-bound signing (US federal, certain regulated industries), the LAMPS composite construction is not certifiable today as a single FIPS module." |
| L14-R-2 | Blog restructure leading with library | **CLOSED** | §2 (the library) is placed before §3 (threat model); the library is the first concrete artifact the reader encounters. This is the load-bearing structural revision of the v2 draft. |
| L14 BouncyCastle 1.80+ already ships LAMPS | Cite | **CLOSED** | §4 paragraph "Why LAMPS, not the alternatives": "BouncyCastle 1.80+ supports Composite ML-DSA in CMS SignedData; OpenSSL 3.5 has partial support; AWS KMS supports PQ signatures…" §7 table confirms. |
| L14 noble-PR distribution | ~150-250 LOC PR to paulmillr/noble-post-quantum hybrids.js | **DEFERRED-NAMED-NOW** | Destination = `.addl/phase-4-meta/comment-opportunities-implementation-libs.md` (the ecosystem-scan output already on origin). The noble-PR opportunity is a separate workstream (post-publication F-class follow-up), not blog scope. |

### L15 — Threat-model-boundaries skeptic (round 1 HIGH; REVISE-MAJOR)

| # | Finding | Disposition | Closure cite |
|---|---|---|---|
| L15-R-1 | Replace "Benten is alone" with "Benten is first-shipping in a converging cohort" + cite draft-ietf-openpgp-pqc-17 + Sequoia + Sigstore retooling | **CLOSED** | §1 lede: "We are not first to argue for this construction — the IETF OpenPGP working group is concurrently mandating the same composite… draft-ietf-openpgp-pqc-17… Sequoia PGP… Sigstore + Trail of Bits are independently retooling on the same threat-model. We are early-shipping in a converging cohort." Reinforced in §5 caveat 1 + §7 paragraph 2 + §10. |
| L15-R-2 | OpenPGP-PQC parallel cited prominently | **CLOSED** | §1 lede + §6 inclusion list + §7 table row + §10 acknowledgment all cite OpenPGP-PQC + Sequoia + Sigstore + Trail of Bits. |
| L15-R-3 | Per-system class boundary table (Filecoin EXCLUDED / Arweave INCLUDED-but-cautionary / git+OpenPGP INCLUDED-active / Sigstore INCLUDED-active) | **CLOSED** | §6 contains the full inclusion / exclusion lists with per-system reasoning. Filecoin in exclusion ("consensus signatures are gated by consensus, not by re-verification of historic signed artifacts decades later"). Arweave in exclusion ("*should* be in the class… locked into legacy by ~5 PB of signed history") with the "cautionary example for why picking the construction *before* the legacy exists matters" framing. git in exclusion ("human-in-loop attention at install/merge"). OpenPGP-PQC + Sigstore receipts in inclusion. |
| L15 honest acknowledgment: class boundary IS load-bearing structural claim | Invite critique | **CLOSED** | §6 closes: "The class boundary IS the load-bearing structural claim of this post. If we have it wrong — wrong inclusions, wrong exclusions, or wrong definition entirely — that is the critique we most want to hear." §9 ask #1 reinforces. |

---

## Universal-revision-roadmap dispositions (synthesis matrix-level)

The synthesis matrix `r2-critic-15-synthesis-matrix.md` aggregates every per-critic finding into a universal roadmap; per-finding dispositions are above. The roadmap-level rollups:

- **Section 2 (the asymmetry argument): L6 R-2 class-boundary narrowing** — CLOSED (used canonically across §3, §5 caveat 4, §6, §7).
- **Section 5 (what we shipped): L12 R-4 combiner section + cryptographer-review recommendation** — CLOSED via §4's full population including Inv-15 three-layer decomposition.
- **Section 6 (honest caveats): 5 → 9 caveat expansion** — CLOSED (§5 contains all 9 caveats, each with closure citation above).
- **Section 9 (the library): L14 R-2 blog restructure leading with library** — CLOSED via §2 placement before §3.
- **Section 10 (acknowledgments): L8 + L15 + L9 specific acks** — CLOSED via §10 (multicodec maintainers; Signal PQXDH editors; OpenPGP-PQC WG + Sequoia + Sigstore + Trail of Bits; internal adversarial reviewers).

---

## Extra-reflection-pass output (per `feedback_extra_reflection_pass_for_elegant_permanent_shape`)

The single most elegant permanent-shape revision in this v2 draft is the **Inv-15 three-layer decomposition** in §4 (identity = canonical-payload-CID; authentication = codepoint-dispatched signature; revocation = semantic tuple). This single architectural framing closes simultaneously:

- L12 (combiner soundness): the EUF-CMA-vs-SUF-CMA gap is closed at the application layer rather than via algorithm switch; ALL of L12's specific blog-text-changes-requested are addressed without invoking Bird-of-Prey.
- L6 (class boundary): the class definition "long-lived sigs delegating capability into automated trust chains without per-use human review" matches exactly the Inv-15 layered identity model — capability delegation IS the trust-chain edge that Inv-15's payload-CID identifies.
- L1 (META-message about legitimization): the blog's central technical claim is now an *engineering invariant* (Inv-15) that the project can be evaluated against by reading the code, not a rhetorical position. The META-message becomes "small team did the engineering homework" rather than "small team minted a private codepoint and blogged."
- L9 (LAMPS-WG-test-vector deposit as highest-leverage contribution): Inv-15 + the LAMPS-default construction means Benten's test-vector contribution is purely a draft-19 conformance artifact, not a Benten-specific construction; this is exactly the contribution the LAMPS WG wants.
- L15 (cohort framing): Inv-15 + LAMPS-default places Benten structurally in the OpenPGP-PQC + Sigstore cohort rather than as an outlier — the engineering layer matches what the rest of the cohort is doing.

Combined with **leading-with-the-library (§2)** and **cohort-framing (lede + §5 caveat 1 + §7 + §10)**, three structural revisions close ~80% of the universal-revision-roadmap. The remaining ~20% are per-finding text edits (verbatim quotes, citation precision, FIPS caveat, key-reuse caveat) that are individually small.

---

## Knowledge-limits I want to flag (per brief discipline "Acknowledge knowledge-limits explicitly")

1. **lidel's "No multicodec update required" quote (L1-R-2)** — I did not source-verify this quote within the session; the L1 critic JSON cites it but I did not WebFetch the specific PR comment. Pre-publication: the orchestrator-direct workspace check should verify the lidel quote against PR #400 or #403 comment threads before publishing the §8 multicodec paragraph in its current form. If the quote does not appear verbatim, §8's framing still stands (it does not currently quote lidel verbatim — only cites the PRs by number with the direction-of-work characterization).

2. **MLS-default ECDSA P-256** characterization in §7 — I have not source-verified the current MLS-default cipher suite; the L8 critic JSON cites it. The L8-recommended 4-row split of "Messaging" into Signal-private-1:1 / Signal+CKV / iMessage+CKV / MLS-group is partially absorbed (§7's table separates Signal, Apple iMessage, MLS-default). The full 4-row split was not implemented because the table's per-row "stated reasoning" cell would be empty for the CKV-anchored rows (no public PQXDH-equivalent rationale exists for CKV-side PQ-sig decisions). Pre-publication: a quick Apple-CKV-source-of-truth check would either confirm "no public rationale we found" (current text) or surface text to attribute.

3. **Sigstore "signed today, deployed 20 years" quote (§7)** — attributed to "Trail of Bits" via the L15 critic and the Sigstore PQC blog link, but I did not WebFetch the specific Trail of Bits paragraph. Pre-publication: source-verify against the Sigstore PQC post + Trail of Bits architectural-agility writeup; if no exact verbatim match, the quote should be paraphrased or removed.

4. **NIST SP 800-227** referenced in §5 caveat 9 — this is the expected NIST guidance on composite signatures; I have NOT verified the SP number or its publication status. If the number is wrong or the document has not yet been opened, the caveat's "evolve when NIST issues SP 800-227 or equivalent" framing still holds; pre-publication a NIST source check is straightforward.

5. **ANSSI quote in §4** — the "the only generic construction recommended by ANSSI for signature combination" is from the cryptographer review §10 (Synacktiv-cited); I did not WebFetch the ANSSI source. Pre-publication: source-verify or attribute as "per ANSSI, via Synacktiv summary."

These items belong in the pre-publication discipline checklist (per planning-agent §3 review checklist + position-b-revision-roadmap.md §"Pre-publication discipline").

---

## DISAGREE entries (per HARD RULE 12 clause-c)

**L5 F5 STRONG OBJECT (publish at all)** — I disagree. The L5 finding that the team-scale data anchors every sustainability claim is correct AND a v2 draft that takes that finding seriously (cohort framing; lead-with-library; honest caveats; "early adopter, not leader" positioning; SECURITY.md + 48h SLA commitments) is a legitimately-stronger artifact than no-blog. Withholding publication forfeits the value of being early in a converging cohort and undersells the engineering work the project has done. The decision-gate is still Ben's; I am surfacing the v2 substantive revision that closes every fixable critic finding so that the publish-vs-defer call is informed by the strongest available draft, not by L5's veto applied to the v1 draft.

**L1 (downgrade to Position A, "engineering notes")** — I disagree on form. The v2 draft does NOT take the Position-A "engineering notes" framing because L12 (combiner-soundness) made clear that engineering-notes framing CANNOT support the LAMPS-default-+-Inv-15 decomposition that is the substantive engineering answer. The v2 absorbs L1's META-message concern via structural revision (lead-with-library; no Benten-private codepoint in blog body; explicit endorsement of multicodec maintainer direction; acknowledgment line for maintainers) rather than via downgrade. If L1 reads the v2 and still wants Position A, that is a re-discussion; the v2 is the strongest Position-B-compatible absorption of L1's concerns.

---

## What this changelog is NOT

- Not a publication-readiness declaration (Ben's gate)
- Not a closure of any architectural decision the synthesis matrix surfaced (those are Ben's gates; the v2 draft applies the ratified decisions)
- Not a comment-opportunities prioritization (separate workstream)
- Not the final pre-publication discipline checklist (separate workstream; see `position-b-revision-roadmap.md` §"Pre-publication discipline")

---

*Drafted 2026-05-26 by Position B revision agent. Companion artifact to `position-b-blog-draft-v2.md`. References: `r2-critic-15-synthesis-matrix.md`, `position-b-revision-roadmap.md`, `cryptographer-review-bird-of-prey-vs-lamps.md`, `critic-lens-l{1..15}-*.json`.*
