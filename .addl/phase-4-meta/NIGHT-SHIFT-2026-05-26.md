# NIGHT-SHIFT — 2026-05-26 (PQ public-stance critic-synthesis + cryptographer-review + ecosystem-scan)

> Compact-survival authoritative pickup surface for this session. Read top-banner FIRST.
> Last update: 2026-05-26 mid-day post-6-agent-dispatch.

## ⚠️ Read this BEFORE acting

This handoff names what's load-bearing AT WRITE-TIME. Your session may need broader/different context — read whatever else you see fit (plan docs, decisions logs, NIGHT-SHIFT-2026-05-25.md addenda 1-13, mini-review JSONs, memory beyond the index, the strategy-plan / R0-plan / 15 critic JSONs in .addl/phase-4-meta/, the cryptographer-review findings file when it lands). Verify state hasn't drifted + verify in-flight agents before continuing.

The 2026-05-25 LATE-AFTERNOON addendum #13 in [NIGHT-SHIFT-2026-05-25.md](NIGHT-SHIFT-2026-05-25.md) is the prior pickup surface; this file SUPERSEDES it for 2026-05-26+ state.

## ⚠️ HOLD list (DO NOT do these without Ben check-in)

- **Tag `phase-4-meta-core-close`** — Ben: "hold off on actually closing out the phase/giving it a tag before I check in" — pre-tag sweep allowed; actual `git tag` + push waits.
- **L12 combiner final choice** (LAMPS-default + revoke-by-tuple-hardening vs Bird-of-Prey-default + LAMPS-opt-in vs Bird-of-Prey-only) — awaiting cryptographer-review agent return + Ben ratification. Ben's tentative directional preference: Bird-of-Prey-default + LAMPS-opt-in. CRYPTOGRAPHER-REVIEW MAY OVERTURN. Do not commit to v1-beta combiner shape until both return.
- **L9+L11 F4/F4a sequencing** — held until L12 settles (Ben: "I was just holding F4/F4a for us to settle on whether we go with option A or B").
- **Comment posting** on F2/F3/F4/F7 drafted threads — drafts at `.addl/phase-4-meta/f2-f7-comment-drafts.md`; Ben wants to wait for comment-opportunity scan agents to return + batch posting after ratification.
- **Position B blog publication** — Shape 5 (private iroh-coalition-first outreach BEFORE blog publish) is ratified; iroh-outreach prep agent NOT YET dispatched (awaits arch decisions + scan returns).
- **G-CORE-PQ-WIRE wave dispatch** — gated on combiner-choice ratification.
- **R6 R3 dispatch on consolidated main** — held until arch decisions land (don't waste lens-cycle effort on a base that may shift).
- **Real architectural forks under broad auth** — always surface as plain-English options + my-pred + clear ask per `feedback_surface_arch_decisions_under_auth`.

## Current state snapshot (live as of write-time)

- **main HEAD: `462dfe7f`** (PR #1356 Strategy-C consolidation merged 2026-05-26 05:51 UTC)
- **Local branch**: on `main`; stale local branch `phase-4-meta-core/r6-r2-fp-integration-redo @ 43911bab` left in place (`-D` was gated; cosmetic only; squash-merged so commits are reachable via main)
- **Origin branch `phase-4-meta-core/r6-r2-fp-integration-redo`**: still exists; deletion gated; safe to delete via `git push origin --delete` when Ben confirms
- **Open PRs (BentenAI/benten-engine)**: 4 dependabot only (#1355 / #1352 / #1343 / #1316); no R6 / phase-4-meta-core PRs
- **Worktrees**: main + codex + 6 active agent worktrees (each `${WORKTREE_ROOT}/.claude/worktrees/agent-*`); will drop on agent-return
- **Disk**: 86% / 26Gi free at session start; agent worktrees ~12M each = ~72M total + their target/ if they cargo-build (likely won't; all read-only)

## ✅ Ben ratifications this session

1. **Pass 1 housekeeping + L12 ground-truth-verify + L13 verify ratified**: switch to main; verify L12's 3 claims (all VERIFIED); verify L13's wire-format-untouched claim (VERIFIED — no code exists; R0-plan-only edit).
2. **Cryptographer-review agent dispatched with Ben's explicit ratification**: "yes you can dispatch a separate cryptographer-review agent to scrutinize the Bird-of-Prey implementation BEFORE we commit to shipping it at v1-beta"
3. **5 comment-opportunity exploration agents dispatched with Ben's ratification**: "Yes, dispatch 5 as proposed (Recommended)"
4. **Tentative L12 directional preference**: "we're taking the stand to ship Bird of prey as our default/suggested standard in our beta engine but we provide lamps as a backup for people who would specifically prefer it (and to demonstrate the flexibility of the system or whatever)" — i.e. **B' (Bird-of-Prey-default + LAMPS-opt-in)** tentatively preferred over Option A (LAMPS-default + revoke-by-tuple-hardening) or C-only (Bird-of-Prey-only). Cryptographer-review-agent finding may overturn.
5. **L13 R0-plan edit ratified**: "plan edit sounds fine" — will batch with other R0-plan edits after cryptographer-review lands.

## 6 agents in flight (all background; isolation:worktree; read-only; cap-exempt)

| Agent ID | Topic | Findings will land at | ETA |
|---|---|---|---|
| `a694fcec68115cc6f` | **Cryptographer-review**: Bird-of-Prey vs LAMPS vs draft-prabel vs IACR 2025/2059; GO/NO-GO/CONDITIONAL on v1-beta default; codepoint layout; impl hazards; extra-reflection-pass | branch `phase-4-meta-core/cryptographer-review-bird-of-prey`; file `.addl/phase-4-meta/cryptographer-review-bird-of-prey-vs-lamps.md` | 2-4hr |
| `af067bf967dfb8912` | **Multiformats + W3C DID** ecosystem scan: multicodec / multibase / multihash / cid / w3c-ccg / DIF / W3C VC — open issues + PRs + spec discussions | branch `phase-4-meta-core/comment-opps-multiformats-w3c-did`; file `.addl/phase-4-meta/comment-opportunities-multiformats-w3c-did.md` | 45-90 min |
| `a90c9703dc1a9bdf3` | **IETF PQ WGs** scan: LAMPS / JOSE / COSE / CFRG / OpenPGP / PQUIP — drafts + mailing lists + IANA registries; F4a test-vector-deposit verify | branch `phase-4-meta-core/comment-opps-ietf-pq-wgs`; file `.addl/phase-4-meta/comment-opportunities-ietf-pq-wgs.md` | 45-90 min |
| `ab306a4ff9c22dc84` | **Content-addressed + long-term-persistence** scan: iroh / Willow / IPFS / Filecoin / Arweave / libp2p / Sigstore / git-PGP — Shape-5-sequencing-flag on iroh threads | branch `phase-4-meta-core/comment-opps-content-addressed-systems`; file `.addl/phase-4-meta/comment-opportunities-content-addressed-systems.md` | 45-90 min |
| `a61119277a005014c` | **Implementation libraries** scan: noble-post-quantum (verify L14's hybrids.js opportunity) / RustCrypto / BouncyCastle / AWS / Cloudflare; bug-class scan | branch `phase-4-meta-core/comment-opps-implementation-libs`; file `.addl/phase-4-meta/comment-opportunities-implementation-libs.md` | 45-90 min |
| `a1ae45ca720280a79` | **P2P + messaging adjacents** scan: Signal / MLS / Nostr / Veilid / ATProto / Holepunch / Matrix; cohort-evidence + future-Shape-5-candidates | branch `phase-4-meta-core/comment-opps-p2p-messaging`; file `.addl/phase-4-meta/comment-opportunities-p2p-messaging-adjacents.md` | 45-90 min |

Cap check: 6 agents all read-only / research-type; cap is 7 IMPLEMENTER (CLAUDE.md §13); read-only agents exempt. Zero implementer agents active.

## Decisions awaiting Ben (queue)

1. **L12 combiner choice (TENTATIVELY B')** — final ratification waits on cryptographer-review-agent return. Surface their GO/NO-GO/CONDITIONAL recommendation with my-pred + reasoning when it lands.
2. **L9+L11 F4 sequencing flip + F4a mint** — held; surface when L12 settles.
3. **Comment-opportunity triage** — when 5 scan agents return: synthesize cross-cluster + surface prioritized list to Ben for triage/prioritization (some COMMENT-NOW vs WATCH-ONLY vs hold-until-Shape-5).
4. **F2/F7 drafts (already drafted)** — ready for Ben review/post; Ben preference: batch with comment-opportunity scan results.

## Standing law (apply throughout)

- **NEVER `--admin-bypass`** anywhere
- **NEVER force-push** (incl `--force-with-lease`) — exception: orchestrator may force-with-lease on a feature branch when rebasing pre-merge per §2.9 5-step
- **NORMAL `--squash`** for merges
- **HARD RULE 12** disposition discipline (only 3 valid non-fix-now: OUT-OF-SCOPE / BELONGS-NAMED-NOW with specific destination + entry NOW / DISAGREE-WITH-EXPLANATION)
- **Default prediction = do-it-now** NOT defer (per `feedback_orchestrator_defer_prediction_bias`)
- **Full-depth briefs** (per `feedback_orchestrator_brief_thoroughness_dont_cut_for_turn_budget`)
- **§3.5n ground-truth-verify** every review finding before scoping
- **§3.5h pre-push gate** before any push
- **§3.6j sweep-completeness** self-verify on wave's own artifacts
- **Disk** >85% pre-emptive clean / >92% HARD-ABORT
- **Agent disciplines**: isolation:worktree + run_in_background:true + ABSOLUTE-PATH-FORBIDDEN outside `${WORKTREE_ROOT}` + commit-before-return + full-depth briefs
- **Parallelism cap** ≤7 implementer agents (read-only agents exempt)
- **Plain-English-with-prediction lens** on every Ben-facing surface
- **Convergent-paths-within-night-shift-auth** (Ben 2026-05-25); divergent paths surface

## Plain-English context (what we're in the middle of)

We're preparing to tag **v1-beta** for Benten Engine. v1-beta ships PQ-hybrid signatures as default — the choice of WHICH PQ-hybrid construction is the load-bearing question.

We were about to ship **LAMPS Composite ML-DSA** (the IETF-WG-converged construction; broad interop with BouncyCastle + OpenPGP-PQC) as v1-beta default. Then critic L12 surfaced that LAMPS is **EUF-CMA-only NOT SUF-CMA**, and Benten's UCAN-revocation-by-sig-CID pattern has a real bypass under EUF-only signatures.

Two paths to mitigate:
- **A**: ship LAMPS + add `revoke-by-tuple` hardening (sidesteps the SUF-CMA gap; broad interop preserved; conservative)
- **B'**: ship **Bird-of-Prey** (newer SUF-CMA-preserving construction, CRYPTO 2026 paper) as default + LAMPS as opt-in for interop (Ben's tentative preference; the stronger leadership stance; takes implementation risk on fresh academic crypto)

Cryptographer-review-agent is evaluating Bird-of-Prey-vs-LAMPS rigor + recommending the v1-beta default. While that runs, 5 ecosystem-scan agents are sweeping for OTHER places (multiformats / IETF WGs / content-addressed-systems / impl-libs / P2P-adjacents) where Benten as adopter should comment.

After cryptographer-review + scan returns:
1. Ben ratifies combiner choice → R0 plan revision applies L4+L13+L9+L11 batched + chooses v1-beta default
2. We dispatch Position B blog REVISION agent with universal-revision-roadmap from all 15 critics (and the cryptographer-review-agent's findings) + the new ecosystem evidence
3. We dispatch Shape 5 iroh-outreach prep agent (private iroh conversation BEFORE blog publish per L3)
4. Batch-post the F2/F3/F4/F7 + newly-discovered comment opportunities
5. Dispatch G-CORE-PQ-WIRE canary (Wave PQ-WIRE-1) implementing the chosen combiner shape
6. Resume R6 R3 council on consolidated main
7. Iterate to strict-Q5; pre-tag sweep; HOLD on tag until Ben check-in

## Pim-N candidates pending codification (from this session)

Tracked from L12 surfacing pattern + consolidation cycle + cross-lens themes:
1. **`feedback_extra_reflection_pass_for_elegant_permanent_shape` already codified 2026-05-25** — apply throughout
2. **pim-N cite-grep-verify at author-time** (§3.6j sub-rule) — codified per R6-R2-FP-C
3. **pim-N regression-guard-substantive-arm** (§3.6f extension) — codified per R6-R2-FP-D
4. **NEW: any new pub item referencing cfg-gated module MUST inherit same cfg-gate** (per #1356 consolidation regression on `install_consent_adapter`) — codify §3.5p
5. **NEW: D-17-style #[non_exhaustive] cascade PRs MUST sweep source-grep tests** (per COLLAPSE-#605 regression-pin failure) — codify §3.5q
6. **NEW: re-run cargo fmt after EVERY local edit during Strategy-C consolidation babysit** (per fix-up #3 fmt missing) — codify §3.5h sub-rule
7. **NEW: cite-drift-detector --all author-time gate** (per `benten_dsl_compiler::emit` phantom in fix-up #4) — likely already absorbed by R6-R2-FP-C scanner; verify
8. **NEW (L3+L8 finding-class)**: industry-folklore-vs-spec-text-distinction — when a critic-persona's "going-in attack" cites industry-folklore for why X system did Y, verify against actual spec/blog text (often the folklore is wrong); name as research discipline
9. **NEW (L9 finding-class)**: IETF-vocabulary-precision — when commenting on IETF specs / drafts, use precise WG-vocabulary (e.g. "WG document" vs "individual submission"; "adoption call" vs "WG-LC"); folksy terminology marks Benten as not-WG-fluent

## Cap status

6 agents in flight (all read-only / research-type / cap-exempt). 7/7 IMPLEMENTER cap free for any next implementer dispatch (no implementer waves planned until arch decisions land).

## Cross-references

- Strategy plan: [.addl/phase-4-meta/pq-hybrid-sig-public-stance-strategy.md](pq-hybrid-sig-public-stance-strategy.md) (~700 lines)
- R0 plan: [.addl/phase-4-meta/g-core-pq-wire-r0-plan.md](g-core-pq-wire-r0-plan.md) (~700 lines; needs revision after L12 ratification)
- 15 critic JSONs: `.addl/phase-4-meta/critic-lens-l{1..15}-*.json`
- F2/F7 comment drafts: [.addl/phase-4-meta/f2-f7-comment-drafts.md](f2-f7-comment-drafts.md)
- Prior pickup surface: [.addl/phase-4-meta/NIGHT-SHIFT-2026-05-25.md](NIGHT-SHIFT-2026-05-25.md) (addenda 1-13; THIS file supersedes)
- 15-critic synthesis matrix: [.addl/phase-4-meta/r2-critic-15-synthesis-matrix.md](r2-critic-15-synthesis-matrix.md) (drafted this session)
- Position B revision-roadmap sketch: [.addl/phase-4-meta/position-b-revision-roadmap.md](position-b-revision-roadmap.md) (drafted this session; combiner section pending cryptographer-review)

---

## 2026-05-26 LATE-AFTERNOON #1 ADDENDUM — L12 architectural decision RATIFIED

### Ben ratification (load-bearing; "all yes across the board")

After cryptographer-review-agent return + high-level-thinking pass walking through (a) steel-man of 6 LAMPS-vs-alternatives + (b) project-wide EUF-vs-SUF mitigation framework, Ben ratified all 3 questions:

1. **LAMPS Composite ML-DSA at codepoint 0x0001 = v1-beta DEFAULT** (within-LAMPS choice = MLDSA65-Ed25519-SHA512 = `id-MLDSA65-Ed25519-SHA512` per OID `1.3.6.1.5.5.7.6.48` IANA early-allocated 2025-10-20; OpenPGP-PQC parallel; BouncyCastle 1.80+ alignment) — RATIFIED
2. **8-mechanism Inv-15 mitigation framework** (sig-bundle CIDs are never load-bearing identifiers; identity = payload-CID; authentication = codepoint-dispatched sig; revocation = semantic tuple) — RATIFIED
3. **G-CORE-PQ-WIRE-1 wave bundles the audit + hardening scope** (~+400-600 LOC: surface audit + Inv-15 tests + SECURITY-POSTURE/CLAUDE.md/Compromise updates + cite-drift-detector extension + pim-N codification) — RATIFIED

### Implications now-unlocked

- **Bird-of-Prey / draft-prabel deferred** to future additive codepoint when WG-adopted + impl-audited (NOT v1-beta-default)
- **L9+L11 F4 deferral to post-RFC-publication** RATIFIED via inheritance from L12 settlement
- **L9 F4a slate-mint** = test-vector deposit via LAMPS GitHub issue #289 RATIFIED
- **L4+L13 R0-plan edits** (drop Benten-private did:key URI rendering; did:jwk canonical) RATIFIED
- **F2/F7 posting unblocked** (multicodec window narrowing per Multiformats scan)
- **F3 JOSE adoption-call comment** (3-day deadline 2026-05-29) drafting unblocked
- **Position B blog revision agent dispatch** unblocked with FULL input package (synthesis matrix + revision roadmap + cryptographer-review + 5 scan findings)
- **Shape 5 iroh-outreach prep agent dispatch** unblocked
- **R0 plan revision** unblocked (one coherent revision pass: L4/L13 edits + L9/L11 sequencing + F4a mint + LAMPS-default lock + Inv-15 framework + Compromise #N + CLAUDE.md #5 retense)
- **G-CORE-PQ-WIRE-1 canary brief authoring** unblocked (full scope incl. hardening)

### Sequencing for next-action queue

**Time-critical (post first; ~3-day window)**:
1. JOSE adoption-call comment draft (orchestrator-direct; Ben review + post)
2. F2/F7 + multicodec PRs #400/#403 endorsements post (drafts already ready)

**Stable timing (post in coherent batches)**:
3. F4a LAMPS issue #289 negative test vectors comment draft
4. NEW comment opportunities from 5 scans (top ~10-15 triaged)
5. W3C CCG #70/#74 + did-method-* + vc-data-integrity #338 cluster

**Orchestrator-direct doc work (R0 plan revision pass)**:
6. R0 plan revision (one coherent edit pass): L4/L13 / L9/L11 / F4a / LAMPS-default lock / Inv-15 framework reference
7. INVARIANT-COVERAGE.md Inv-15 mint
8. SECURITY-POSTURE.md update (L12 hazard + Inv-15 closure + Bird-of-Prey future-path)
9. CLAUDE.md baked-in #5 retense (LAMPS-default explicit + Inv-15 invariant + Bird-of-Prey-future-additive)
10. Compromise #N mint (or update #30): "LAMPS EUF-CMA-only at construction; SUF-equivalent via Inv-15 application-layer"
11. dispatch-conventions §3.X — pim-N codify: "Future signed-data designs MUST 3-layer-decompose"

**Agent dispatches (after R0 revision lands)**:
12. Position B blog revision agent (full input package = synthesis matrix + revision roadmap + cryptographer-review + scan findings + this ratification)
13. Shape 5 iroh-outreach prep agent (per L3 framing)
14. G-CORE-PQ-WIRE-1 canary implementer (with bundled Inv-15 audit/hardening scope)

**Held**:
15. R6 R3 dispatch (post-R0-revision-merge)
16. Pre-tag sweep + tag phase-4-meta-core-close (after R6 strict-Q5)

### Cap status

Currently 0 agents in flight (6 just landed; cleanup done). Cap-wise can dispatch Position B revision + Shape 5 prep in parallel when their input packages are ready (both read-only / cap-exempt).

---

*Authored 2026-05-26 mid-day post-6-agent-dispatch. L12 architectural ratification appended LATE-AFTERNOON. Updates rolling as work lands.*
