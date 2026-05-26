# Encrypt-to-recipient at v1-beta — architectural review input package

> Shared context for the 3 parallel reviewers (senior cryptographer + P2P-systems architect + standards-maturity-and-ecosystem skeptic) evaluating whether Benten should add encrypt-to-recipient encryption at v1-beta scope, and if so via which construction.
>
> Ben-ratified dispatch 2026-05-26 post-JOSE-archaeology-agent-return + post-F3-JOSE-comment-X-Wing-vs-HPKE-PQ-disambiguation discussion. Same review shape as the cryptographer-review-of-bird-of-prey-vs-lamps that landed clean 2026-05-26 at branch `phase-4-meta-core/cryptographer-review-bird-of-prey @ 36afe06b`.

## The architectural question

**Should Benten add encrypt-to-recipient encryption (data encrypted under a specific other party's pubkey so they and only they can decrypt) as v1-beta scope? If yes, via which construction?**

This is a pre-wire-format-freeze decision — v1-beta will tag a frozen wire format that locks default cipher suite choices. Adding encrypt-to-recipient post-freeze is possible via the crypto-agility framework (CLAUDE.md baked-in #5: additive codepoints, never wire-break), but the v1-beta default choices have outsized influence on what content gets shipped + what later content has to interop with.

## Project context (compact)

**Benten Engine**: content-addressed graph engine (Rust + TS DSL) with decentralized identity (Atrium peer-mesh; did:jwk + did:key + private-DID variants); preparing to tag `v1-beta`. The project is 5 weeks old / 1 contributor / 0 stars / pre-public-launch, but on AI-accelerated dev tempo (8 agent dispatches landed in the last 24 hours including a 15-critic adversarial slate + cryptographer review + Position B blog v2 + iroh outreach prep).

**Confidentiality-half-of-Principal-primitive is explicitly v1-beta-blocking** per CLAUDE.md baked-in #15: *"the v1 tag is now explicitly gated on the confidentiality half of the Principal primitive existing"* + baked-in #18: *"capability-gating only binds a cooperating engine; the moment a principal's subgraph rests on hardware another principal controls, capabilities give ZERO protection — encryption is the load-bearing substrate there."* So encryption-on-untrusted-hosts is committed for v1-beta scope; the question is which construction.

**Current encryption state at v1-beta**:
- **Per-DID storage partition** (`WriteContext::namespace_did` per `docs/SECURITY-POSTURE.md` Per-Node AEAD section, Inv-11 / C1)
- **Per-Node AEAD** with K(N) derived from `K_principal` (Spike-E Interpretation-B path-tagged derivation: `K(N) = KDF(K_principal, N.cid)`)
- **AAD-binds-plaintext-CID** + per-chunk-AEAD for Nodes ≥ 64 KiB (rebinding-attack-prevention + cross-chunk-truncation-prevention per `§1.A.FROZEN item 15(g)`)
- **Hybrid KEM combiner**: X-Wing (`draft-irtf-cfrg-xwing`, X25519 + ML-KEM-768, IACR Communications in Cryptology 2024-1-21, Barbosa et al.) at multicodec codepoint `0x647A`. Vendored ~30-LOC combiner per CLAUDE.md baked-in #5.
- **Bulk AEAD**: ChaCha20-Poly1305 (NCC-audited per Compromise #30 mitigation)

This is symmetric AEAD with derived keys; works for "my data, encrypted with keys derived from my K_principal so my devices can read it." Does NOT work for "encrypt this so a different party can read it on untrusted infrastructure" — there's no recipient-pubkey-encryption path.

**Vision use cases requiring encrypt-to-recipient** (per CLAUDE.md baked-in #17 + #18 + the Atrium peer-mesh model):
1. **Atrium-shared content** — communities are distributed copies; member peers should be able to read community content, non-member peers (who may still store the bytes per P2P storage participation) should not
2. **Drop bundles encrypted to specific recipient** — Alice sends a Drop to Bob via untrusted intermediate peers; only Bob can decrypt
3. **Multi-device sync with non-cooperating-engine-store-and-forward** — peer A's encrypted content passes through peer B (who can't read it) en route to peer A's other device
4. **Garden-Grove untrusted-host** (Phase 7+) — peer hosts (cloud, friend's machine, etc.) hold ciphertext-only
5. **Kith selective-disclosure** — disclose specific content to specific recipients

Ben's framing (2026-05-26): *"P2P is as secured/untrusted as possible should just be our default since that's very much what we're building towards as our hyper-scaling feature."*

## Option space (your job: evaluate all of these + recommend)

### Option A — Status quo (defer encrypt-to-recipient to Phase-4-Meta-Composing or later)

- Ship v1-beta with X-Wing storage-substrate AEAD only
- Encrypt-to-recipient lands at a later phase (Phase-4-Meta-Composing / 5 / 7+)
- Crypto-agility seam allows later additive codepoints without wire-format break
- Trade-off: v1-beta is shippable + lighter scope, but the "P2P-untrusted-default" vision isn't realized at v1-beta tag

### Option B — HPKE-RFC-9180 envelope + X-Wing as KEM combiner

- Use HPKE (RFC 9180, published standard) as the encrypt-to-recipient envelope shape
- Use X-Wing as the KEM combiner inside HPKE-mode-base (single-recipient) or HPKE-mode-base-extended (multi-recipient)
- Reuses the existing X-Wing combiner we ship at storage substrate (consistency across layers)
- Registers Benten-internal codepoints (`SigCodepoint::HYBRID_*_ENCRYPT_TO_RECIPIENT_*` or equivalent)
- Future-additive: Skokan-draft HPKE-PQ-PQT codepoints land later at JOSE-export surface per the 3-layer-decomposition principle
- Trade-off: HPKE-RFC-9180 is published + analyzed; X-Wing has IACR CIC 2024 peer-reviewed tight security proof; but using X-Wing-as-KEM-inside-HPKE-mode-base is a Benten-specific combination + needs cryptographer-validation that the composition is sound

### Option C — Skokan-draft HPKE-PQ-PQT-specific codepoints

- Adopt the exact construction Skokan's `draft-skokan-jose-hpke-pq-pqt-05` registers
- Uses `I-D.ietf-hpke-pq` KEM combiner (NOT X-Wing)
- Native JOSE/COSE/MLS-stack interop; we become a "current direct adopter" of the JOSE adoption call
- Trade-off: ships a pre-WG-adopted construction at v1-beta default; same maturity-risk profile as Bird-of-Prey was (which the recent cryptographer-review CONDITIONAL-NO-GO'd); HPKE-PQ KEM combiner is younger + less analyzed than X-Wing; locks us to JOSE-JWE-shape envelopes for storage-substrate use cases that don't actually need JOSE-JWE shape

### Option D — Benten-tailored wire format with X-Wing KEM (no HPKE envelope)

- Implement a Benten-specific encrypt-to-recipient envelope using X-Wing for KEM + ChaCha20-Poly1305 for bulk AEAD
- No HPKE envelope; just X-Wing KEM(pubkey) → shared secret → AEAD(plaintext)
- Smallest implementation; tightest security analysis (X-Wing's tight proof carries directly)
- Trade-off: not JOSE/COSE/MLS-shaped envelope; cross-ecosystem interop story is "Benten has its own envelope; you can implement it via X-Wing reference impl"; possible L4-style ecosystem-fragmentation concern (multicodec maintainers explicitly recommended container-form not Benten-specific envelopes)

### Option E — MLS-style group key derivation for community-shared content

- Adopt MLS-style group key derivation (CGKA — Continuous Group Key Agreement) for Atrium/community-shared content
- Recipients are GROUP members, group key rotates as members join/leave
- Different conceptual shape than encrypt-to-recipient — designed for groups not individual recipients
- May complement Option B/C/D rather than replace them — Drop-to-individual-Bob is encrypt-to-recipient; Atrium-member-only-content might be group-key
- Trade-off: bigger conceptual scope; MLS-PQ extensions are themselves draft-stage; would substantially expand Benten's crypto surface; arguably what foundational P2P encryption should be

### Option F — Something else you think is better

If your analysis surfaces a better permanent shape than the 5 options above, that's the most valuable contribution. Per the `feedback_extra_reflection_pass_for_elegant_permanent_shape` discipline: take an extra pass after evaluating A-E to consider if there's a strictly-stronger 6th option.

## Evaluation criteria (your job: weight + apply)

For each option, evaluate:

1. **Cryptographic soundness** — security properties (KEM SUF-CMA + AEAD INT-CTXT + IND-CCA + non-malleability + etc.); analysis depth; known attacks; required cryptographer-review depth before production ship
2. **Implementation hazard surface** — LOC; complexity; side-channel-resistance requirements; constant-time discipline; audit surface; how it compares to the recent L2-finding ml-dsa CVE GHSA-hcp2-x6j4-29j7 class
3. **Use-case fit for Benten's vision** — does it serve Atrium-shared / Drop-to-recipient / multi-device-via-untrusted-peers / Garden-Grove / Kith use cases?
4. **Ecosystem interop** — JOSE/COSE/MLS frameworks; web-browser-stack consumability; P2P-system peer alignment (Signal/iroh/MLS-implementations)
5. **Standards maturity** — RFC vs WG-document vs individual-draft vs academic-paper; pre-WG-adoption-risk per the L12-cryptographer-review-of-bird-of-prey-vs-lamps lesson
6. **Wire-format freeze risk** — what does v1-beta-tag lock; what's recoverable via additive codepoints later vs structurally locked
7. **Implementation scope estimate** — LOC; weeks of agent-dispatch on AI-dev tempo; cryptographer-review-pre-merge effort
8. **Composition with existing Benten crypto stack** — interaction with Inv-15 3-layer-decomposition / per-DID storage partition / per-Node AEAD with K_principal / X-Wing storage substrate / LAMPS Composite ML-DSA signature default
9. **Position B blog framing impact** — does this option strengthen or weaken Benten's public stance? "We ship the full Principal-confidentiality story at v1-beta" vs "we shipped a Benten-specific construction we couldn't validate"

## Your task — perspective-specific

Each reviewer (senior cryptographer / P2P-systems architect / standards-maturity-and-ecosystem skeptic) brings their own perspective to the same option space. Apply your domain expertise; the orchestrator synthesizes after all three return.

## Return contract (per reviewer)

Write findings to:
- Senior cryptographer: `/Users/benwork/Documents/benten-engine/.addl/phase-4-meta/encrypt-to-recipient-review-cryptographer.md`
- P2P-systems architect: `/Users/benwork/Documents/benten-engine/.addl/phase-4-meta/encrypt-to-recipient-review-p2p-architect.md`
- Standards-maturity-and-ecosystem skeptic: `/Users/benwork/Documents/benten-engine/.addl/phase-4-meta/encrypt-to-recipient-review-standards-skeptic.md`

Structure (Markdown):
1. **Executive recommendation** (~5 sentences): preferred option + confidence level + ~3 most-load-bearing reasons
2. **Per-option assessment** (A through F): GO / NO-GO / CONDITIONAL + reasoning + your most important critique
3. **The cross-perspective question**: where does your expertise specifically apply (vs the other two reviewers' lenses)?
4. **Position B blog framing impact** if you pursue your recommended option
5. **Implementation guidance** (if your recommendation is to ship something): what disciplines / test surfaces / audit gates does it require?
6. **Risks + mitigations**
7. **Extra-reflection-pass output**: more elegant permanent shape if one exists (per `feedback_extra_reflection_pass_for_elegant_permanent_shape`)
8. **Evidence base**: every claim cite-anchored to spec/paper/RFC/draft section + URL
9. **Self-assessment**: confidence level + what additional review you'd want before committing
10. **Honest disagreement**: where you disagree with the framing of the question, the option space, or anything else; surface unconditionally

## Disciplines (load-bearing; same as the bird-of-prey-vs-lamps review)

- **READ-ONLY**: no code edits, no implementation, no PRs. Your work product is the findings file only.
- **Cite-anchor every claim** to spec section + URL OR paper section + URL OR mailing-list message + date. Do NOT paraphrase Security Considerations sections — QUOTE them with section pointers.
- **Apply `feedback_review_finding_ground_truth_verify`**: don't trust prior framings; verify against primary sources via WebFetch.
- **Apply `feedback_industry_folklore_vs_spec_text_distinction` (§3.5r)**: when a claim attributes a position to a system/spec/RFC, verify against actual text; don't extrapolate.
- **Apply `feedback_extra_reflection_pass_for_elegant_permanent_shape`**: after per-option analysis, take an extra pass — is there a more elegant permanent shape (Option F territory)?
- **Acknowledge knowledge-limits explicitly** — say "I'm inferring from X" vs "I'm citing spec section Y."
- **Stay inside `${WORKTREE_ROOT}`** — do not cd to or write paths outside the worktree.
- **Commit + push before return** per `feedback_agent_output_must_commit_before_return`:
  - `git add` + commit + push to your assigned branch
  - Report commit SHA in your return summary
- **No AI attribution** in any output.

## Tools

You have `*` (all tools). Use WebFetch + WebSearch freely to verify cite-anchors. Use Read for input-package files. Use Write for the findings file. Use Bash for git operations.

Key resources for cite-verification:
- HPKE RFC 9180: https://datatracker.ietf.org/doc/html/rfc9180
- X-Wing IRTF CFRG draft: https://datatracker.ietf.org/doc/draft-irtf-cfrg-xwing/
- X-Wing IACR paper: search IACR ePrint for "X-Wing" or Barbosa et al. 2024
- I-D.ietf-hpke-pq: https://datatracker.ietf.org/doc/draft-ietf-hpke-pq/
- draft-skokan-jose-hpke-pq-pqt: https://datatracker.ietf.org/doc/draft-skokan-jose-hpke-pq-pqt/
- MLS RFC 9420: https://datatracker.ietf.org/doc/rfc9420/
- MLS PQ extensions (search current drafts): https://datatracker.ietf.org/wg/mls/documents/
- Signal PQXDH spec: https://signal.org/docs/specifications/pqxdh/ (for the "Signal doesn't do PQ identity sigs because deniability" framing per L8 critic finding)
- CLAUDE.md baked-in #5 / #15 / #17 / #18 (in `CLAUDE.md` on local; or in the orchestration branch `phase-4-meta-core/orchestration-2026-05-26`)
- SECURITY-POSTURE Per-Node AEAD section (in `docs/SECURITY-POSTURE.md` on main)
- Cryptographer review of Bird-of-Prey-vs-LAMPS at `.addl/phase-4-meta/cryptographer-review-bird-of-prey-vs-lamps.md` on branch `phase-4-meta-core/cryptographer-review-bird-of-prey @ 36afe06b`

Background mode + isolation:worktree (per the dispatch). Estimated runtime: 2-4 hours. Prioritize depth + cite-anchoring over speed. The v1-beta-tag-affecting architectural decision deserves the careful upstream investigation.
