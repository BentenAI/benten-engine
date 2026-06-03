# NIGHT-SHIFT — 2026-05-27 (Encrypt-to-recipient + F-full architectural decision cycle)

> Comprehensive compact-survival pickup surface for this session arc. Read top-banner FIRST.
> Last update: 2026-05-27 mid-session.
>
> Supersedes [NIGHT-SHIFT-2026-05-26.md](NIGHT-SHIFT-2026-05-26.md) for current state but the 2026-05-26 LATE-AFTERNOON #1 ADDENDUM captures the load-bearing **L12 / Inv-15 / Compromise #31 / LAMPS-default + 3-layer-decomposition + Bird-of-Prey-as-future-additive** ratifications that are still load-bearing.

## ⚠️ Read this BEFORE acting

This handoff names what's load-bearing AT WRITE-TIME. Your session may need broader context — read whatever else you see fit:
- [NIGHT-SHIFT-2026-05-26.md](NIGHT-SHIFT-2026-05-26.md) LATE-AFTERNOON #1 ADDENDUM (L12 ratification + Inv-15 framework)
- [NIGHT-SHIFT-2026-05-25.md](NIGHT-SHIFT-2026-05-25.md) addenda 1-13 (R6 R2 FP cycle context; PR #1351 / #1356 background)
- [cryptographer-review-bird-of-prey-vs-lamps.md](cryptographer-review-bird-of-prey-vs-lamps.md) on origin branch (the L12 architectural decision evidence)
- The 3 e2r reviewer findings (encrypt-to-recipient architectural review; on origin branches enumerated below)
- CLAUDE.md baked-in #5 / #15 / #17 / #18 (foundational architectural commitments)
- `docs/INVARIANT-COVERAGE.md` Inv-15 section (just landed PR #1357)
- `docs/SECURITY-POSTURE.md` Compromise #30 (PQ primitives unaudited) + Compromise #31 (LAMPS EUF-CMA-only; Inv-15 closure)

## ⚠️ HOLD list (DO NOT do these without Ben check-in)

- **Tag `phase-4-meta-core-close`** — pre-tag sweep + tag awaits Ben check-in (per CLAUDE.md baked-in #15 + standing law)
- **Tag `phase-4-meta-close`** — follows Phase-4-Meta-Composing close; awaits Ben check-in
- **Tag `v1-beta`** — follows Phase-4-Meta-Composing close; awaits Ben check-in
- **G-CORE-PQ-WIRE-1 canary dispatch** — gated on F-full ratification + e2r-ffull-scope-review return
- **R0 plan revision for full F-full wave-set** — gated on Ben F-full ratification
- **R6 R3 dispatch** — held until full F-full wave-cascade lands (so council evaluates post-substrate-freeze state)
- **Position B blog publication** — Shape 5 (private iroh outreach BEFORE blog publish) gates this; blog v2 itself needs §4 refresh post-F-full
- **21 comment drafts posting** — all 21 held until F-full + X-Wing→MLKEM768-X25519 rename ratify (then batch-post in coherent landing per Ben 2026-05-27)
- **F3 JOSE comment v2** — held; deadline 2026-05-29 (3 days from drafting; missing is acceptable cost)
- **Real architectural forks under broad auth** — always surface as plain-English options + my-pred + clear ask per `feedback_surface_arch_decisions_under_auth`

## ✅ Phase ordering (CORRECTED 2026-05-27 after Ben surfaced confusion)

Per CLAUDE.md baked-in #15 + the v1-gate-refactor ratification:

```
Phase 4-Foundation close (tag phase-4-foundation-close — DONE 2026-05-14)
  → Phase 4-Meta begins:
    → Phase-4-Meta-Core (we are HERE; closing in progress)
        - substrate work + crypto + identity + multi-tenant
        - "TERMINATES by FREEZING the v1 public interface" per #15
        - Wire-format-affecting work MUST land here (before interface-freeze)
        - tag phase-4-meta-core-close (HOLD pending Ben check-in)
    → Phase-4-Meta-Composing
        - self-composing meta-circular admin (built strictly ON the frozen v1 surface)
        - v1-assessment-window items (identity-recovery / missing_docs sweep / etc.)
        - tag phase-4-meta-close
  → tag v1-beta
  → (independent crypto audit window per NF-2 / C-GM-AUDIT)
  → tag v1-GM (replaces single v1 per the 2026-05-19 reframe)
```

**Critical**: "Phase-4-Meta-Composing" is NOT post-v1-beta-tag; it's PRE-v1-beta-tag. Both Phase-4-Meta-Core AND Phase-4-Meta-Composing land BEFORE v1-beta-tag. Per Ben (2026-05-27): orchestrator was being imprecise across the conversation about "v1-beta scope" when the correct framing is "Phase-4-Meta-Core scope" or "pre-v1-beta-tag-window scope."

**v1-beta scope decision framework** (Ben-ratified 2026-05-27):
- Default disposition = **do-it-now** per `feedback_orchestrator_defer_prediction_bias`
- "Minimum-viable" framing INVERTS the standing bias; AVOID
- Defer ONLY for specific reason (specific dependency on later-phase work)
- Question for any candidate work item: "is there a specific dependency on Phase-4-Meta-Composing infrastructure that makes Phase-4-Meta-Core landing genuinely impossible?" If no → land in Phase-4-Meta-Core
- Wire-format-affecting → MUST be Phase-4-Meta-Core (before interface-freeze)
- Internal-only changes → can default to either sub-phase

## State at write-time

### Main HEAD: `2172cb6d`

PR #1357 (Inv-15 mint + Compromise #31) merged 2026-05-26. Doc-only PR; bypass-merge-reinstate pattern executed cleanly (temp-remove API drift detector → squash-merge → re-add). Branch protection restored to 12/12 required checks.

### Active workstream: F-full encryption substrate architectural decision

The arc:
1. F3 JOSE adoption-call comment (deadline 2026-05-29) prompted X-Wing-vs-HPKE-PQ disambiguation discussion
2. Ben asked: should we adopt Skokan-draft HPKE-PQ-PQT construction so we're a "current direct adopter"?
3. Discussion surfaced that Benten's encryption use cases need encrypt-to-recipient (not currently shipped); this affects v1-beta scope
4. Dispatched 3-reviewer architectural slate (cryptographer + P2P-architect + standards-skeptic) to evaluate 6-option-space
5. All 3 returned with CONVERGENT recommendation = **Combined Option F**: HPKE-RFC-9180 + MLKEM768-X25519 KEM (real X-Wing math; RG-blessed naming) + multi-stanza-HPKE for groups + CGKA-deferred + Inv-16 3-layer-decomposition + X-Wing-mislabel corrective + pre-tag audit gate
6. **Cryptographer surfaced load-bearing X-Wing-MISLABEL finding** (`crates/benten-crypto-suite/src/cipher_suite.rs` uses HKDF-SHA256-not-SHA3-256; codepoint `0x647a` reserved by IETF for MLKEM768-X25519). **~24 LOC corrective is pre-tag-must-fix REGARDLESS of which encrypt-to-recipient option is chosen.**
7. Ben pushed scope WIDER to **F-full** — encrypt-everything-at-rest-from-local-engine + ephemerally-permission + DAK device authentication + remote-permission-call-from-another-device. 4-layer architecture (A+B+C+D).
8. Dispatched e2r-ffull-scope-review to evaluate full F-full scope + wave-sequencing across Phase-4-Meta-Core + Phase-4-Meta-Composing
9. **Agent rate-limited before completing** (output: "You've hit your limit · resets 4am America/Boise"). No partial state pushed; need to re-dispatch when limits reset.

### Ratifications received (load-bearing; load these as baseline for any new work)

| Ratification | Date | Substance |
|---|---|---|
| **LAMPS Composite ML-DSA as v1-beta sig default** | 2026-05-26 | Per L12-cryptographer-review-CONDITIONAL-NO-GO-on-Bird-of-Prey; Inv-15 application-layer mitigation closes the SUF-CMA gap |
| **Inv-15: sig-bundle CIDs are never load-bearing identifiers** | 2026-05-26 | Three-layer decomposition (identity = canonical-payload-CID; authentication = codepoint-dispatched sig; revocation = semantic tuple) — landed in `docs/INVARIANT-COVERAGE.md` via PR #1357 |
| **Compromise #31 minted** | 2026-05-26 | LAMPS EUF-CMA-only at construction; CLOSED-EQUIVALENT at application layer via Inv-15; landed in `docs/SECURITY-POSTURE.md` via PR #1357 |
| **Bird-of-Prey reserved as future-additive codepoint** | 2026-05-26 | When SUF-CMA-preserving constructions mature (WG-adoption + production-ref-impls + independent impl audit), add via crypto-agility framework as additive |
| **Cross-ecosystem-identifier-as-content principle** | 2026-05-26 | LAMPS OID / JOSE alg-name / multicodec container appear AS CONTENT in envelopes crossing ecosystem boundaries; internal SigCodepoint stays hot-path-dispatch only |
| **CGKA / MLS-PQ deferred** | 2026-05-26 | Atrium has FORKABILITY semantics not messaging-leave-forgets; FS-across-membership-changes is NOT v1-beta scope; multi-stanza-HPKE is v1-beta group fallback (age/Saltpack pattern); CGKA codepoint reserved for post-v1-beta when MLS-PQ matures |
| **Position B v2 2 DISAGREE entries ratified** | 2026-05-26 | L5 STRONG OBJECT to publishing F5 + L1 downgrade-to-Position-A both overridden; v2 ships at Position B confidence level |
| **F-full scope (provisional; awaits e2r-ffull-scope-review)** | 2026-05-27 Ben provisional ratification | Full encryption-everywhere + DAK + remote-permission-call-from-another-device + multi-device-key-wrap; sequencing across Phase-4-Meta-Core + Phase-4-Meta-Composing TBD when e2r-ffull-scope-review returns |
| **Forkability semantics** | 2026-05-27 Ben articulated; awaiting formal codification | "Atrium-as-forkable not messaging-leave-forgets" — when member leaves, they keep copy of past content; future content excludes them via recipient set; matches Benten's content-addressed-graph identity model |
| **Phase-ordering correction** | 2026-05-27 Ben corrected orchestrator imprecision | Phase-4-Meta-Composing is PRE-v1-beta-tag, not "post-v1-beta-defer" |
| **Do-it-now bias on F-full sequencing** | 2026-05-27 Ben | Default = do-everything-pre-v1-beta-tag; defer only for specific reason |
| **Remote-permission-call-from-another-device is v1-beta scope** | 2026-05-27 Ben | "I don't really want to leave it up to the agent to cut the corner of remote permission call from another device" |
| **X-Wing-mislabel corrective is pre-tag-must-fix** | 2026-05-27 implicit ratification | ~24 LOC corrective independent of encrypt-to-recipient decision |

### F-full = 4-layer architecture

```
A. K_principal store
   = REAL K_principal at rest in encrypted form; NOT derivable-from-DID
   Current state: STUB (G-CORE-3e named for Phase-4-Meta wave-set; deferred per pre-2026-05-27 plan)
   Ben's F-full position: PULL FORWARD to land in Phase-4-Meta-Core (pre-tag)
   Estimated scope: ~500-1000 LOC

B. Per-Node AEAD with K(N) derived from K_principal
   Current state: Construction exists; X-Wing-MISLABEL needs correction (~24 LOC)
   Ben's F-full position: Fix the X-Wing-mislabel; rename references to RG-adopted MLKEM768-X25519

C. Encrypt-to-recipient (HPKE-RFC-9180 + MLKEM768-X25519 KEM)
   Current state: NOT BUILT
   Ben's F-full position: Build at Phase-4-Meta-Core; covered by Combined Option F
   Estimated scope: ~1500-2500 LOC
   Includes: multi-stanza-HPKE for groups; capability-bound ephemeral grants (UCAN-scoped wraps)

D. Device authentication + DAK + K_principal-encrypted-at-rest
   Current state: NOT BUILT (entirely)
   Ben's F-full position: Build at Phase-4-Meta-Core or split appropriately
   Sub-pieces:
     - Password-based DAK (Argon2id-derived)
     - Optional biometric layer (additive; can land later within Phase-4-Meta)
     - K_principal + user-DID-private-key encrypted at rest under DAK
     - Engine unlock-on-authentication
     - Cross-platform credential-store integration (keyring-rs / tauri-plugin-stronghold / etc.)
     - **Remote-permission-call-from-another-device** (Ben emphatic: REQUIRED, not cuttable)
     - **Multi-device key-wrap-on-device-link** (encrypt-K_principal-to-new-device-pubkey via HPKE-encap)
   Estimated scope: ~300-1500 LOC (highly dependent on per-platform integration choices)
```

### Returned reviewer findings (all on origin)

**Bird-of-Prey-vs-LAMPS cryptographer review** (`phase-4-meta-core/cryptographer-review-bird-of-prey @ 36afe06b`):
- CONDITIONAL NO-GO on Bird-of-Prey as v1-beta default
- Recommended LAMPS + Inv-15 3-layer-decomposition elegant permanent shape
- Landed Compromise #31 + Inv-15 mint

**5 ecosystem-scan agents** (returned 2026-05-26):
- multiformats+W3C-DID (`phase-4-meta-core/comment-opps-multiformats-w3c-did @ 32ed97c2`)
- IETF PQ WGs (`phase-4-meta-core/comment-opps-ietf-pq-wgs @ a3151dc9`)
- content-addressed systems (`phase-4-meta-core/comment-opps-content-addressed-systems @ 58f00662`)
- implementation libs (`phase-4-meta-core/comment-opps-implementation-libs @ d805c139`)
- P2P + messaging (`phase-4-meta-core/comment-opps-p2p-messaging @ 09203ef4`)

**JOSE archaeology agent** (`phase-4-meta-core/jose-adoption-call-thread-deep-dive @ d58f966d`):
- F3 JOSE adoption call: strong-uncontested-adoption trajectory; 7 supporting replies + 0 objections
- Surfaced X-Wing-vs-HPKE-PQ-construction-conflation in v1 draft → applied §7.2 fix → v2 ready
- Identified Brian Campbell's "algorithm-registry-conservatism" position as cross-thread signal (frame Benten JOSE comments as "stability + cross-stack alignment" not "registry expansion")

**Position B blog v2** (`phase-4-meta-core/position-b-revision-v2 @ 627efcdb`):
- 4480 words + 5402-word changelog
- Lead-with-library + Inv-15 3-layer-decomposition + cohort framing + 9 caveats + verbatim LAMPS/iroh/PQXDH quotes
- 2 DISAGREE entries (L5 STRONG OBJECT + L1 downgrade-to-Position-A) ratified
- 5 knowledge-limits flagged for pre-publication source-verify
- **Note**: §4 will need refresh post-F-full to add encrypt-to-recipient + Inv-16

**Shape 5 iroh-outreach package** (`phase-4-meta-core/shape-5-iroh-outreach-prep @ e57591b6`):
- 533 lines; 5 deliverables + pre-send checklist
- Two-email split: Email-1 (~280 words; framing-check pre-v2-skeleton-lock) + Email-2 (~420 words; full draft + 28-day window + explicit veto clause)
- Rüdiger Klaehn primary recipient; matheus23 CC'd; Filippo Valsorda confirmed NO iroh relationship
- 11 decision-gate response shapes documented; 5 surface to Ben as divergent paths

**3 encrypt-to-recipient reviewers** (returned 2026-05-26-27):
- cryptographer (`phase-4-meta-core/encrypt-to-recipient-review-cryptographer @ 791c8d17`) — CONDITIONAL GO on Option B + X-Wing-MISLABEL load-bearing finding + Option F three-layer codepoint REGISTRY refinement
- P2P-systems-architect (`phase-4-meta-core/encrypt-to-recipient-review-p2p-architect @ 8cfb079c`) — Option F structural-SPLIT (single-recipient ship + CGKA defer + multi-stanza group fallback)
- standards-maturity-skeptic (`phase-4-meta-core/encrypt-to-recipient-review-standards-skeptic @ f5a0d0e4`) — Option B' (RG-blessed MLKEM768-X25519 naming) + Inv-16 mint isomorphic to Inv-15

**e2r-ffull-scope-review (RATE-LIMITED; needs re-dispatch when limits reset 2026-05-27 ~4am America/Boise)**:
- Reframed task: full F-full scope + wave-sequencing across Phase-4-Meta-Core + Phase-4-Meta-Composing + DAK design + remote-permission-call-from-another-device (REQUIRED not cuttable) + cross-platform package landscape
- Supplementary scope clarifications sent: user-DID private key protection + multi-device key-wrap-on-device-link + identity-recovery flag (not full design)
- No partial state pushed
- Next action: re-dispatch when rate limits reset; same brief + reframing

### 21 comment drafts on disk (all held until F-full + rename ratify)

At `.addl/phase-4-meta/`:

| File | Drafts contained |
|---|---|
| [f2-f7-comment-drafts.md](f2-f7-comment-drafts.md) | F2 multicodec PR #400 + #403; F7 W3C CCG #70 + #74 (4 drafts) |
| [f3-jose-comment-draft.md](f3-jose-comment-draft.md) | F3 JOSE adoption-call comment v2 (post §7.2 X-Wing-vs-HPKE-PQ disambiguation) (1 draft) |
| [new-comment-drafts.md](new-comment-drafts.md) | F4a LAMPS issue #289 + 14 NEW thread drafts + CFRG draft-prabel adopter datapoint (16 drafts) |

**Rename audit needed post-F-full** — these drafts cite our specific construction + need X-Wing → MLKEM768-X25519 rename pass:
- F3 JOSE comment v2 (cites "X-Wing-style combiner")
- F4a LAMPS #289 (cites our LAMPS adoption + may need test-vector content polish)
- sigstore/rekor-tiles #425 (cites our EXACT Ed25519+ML-DSA-65 + X-Wing-style combiner)
- w3c-ccg/di-quantum-safe #5 (cites hybrid composite + X-Wing references)
- CFRG draft-prabel adopter datapoint (names our combiner direction)
- Position B v2 blog §1/§2/§4 (cites "X-Wing-style combiner")
- Possibly: sigstore/sigstore #2129; ATProto #3928 + others that mention specific construction

**Rename audit can be done post-F-full** (orchestrator-direct; ~30-45 min batch edit).

**Independent of rename** (no specific cipher-suite naming):
- F2 multicodec PR #400 + #403 endorsements (container-form direction; light touch)
- F7 W3C CCG #70 + #74 (DID-method discussions; light touch)
- Some other adjacent-system drafts

Ben (2026-05-27) directive: **hold all 21 drafts** until F-full ratifies + rename pass completes; batch-post all together in coherent landing.

### Architectural-decisions awaiting Ben (queue)

1. **F-full ratification** — gated on e2r-ffull-scope-review return (re-dispatch when rate limit resets); then synthesize 4-reviewer total picture + surface full F-full + wave-sequencing recommendation
2. **Within-F-full: password-first-vs-biometric-additive** — Ben open to "password-only at v1 + biometric layered later within Phase-4-Meta"; needs explicit ratification once scope-review returns
3. **Within-F-full: per-platform integration scope** — desktop (Tauri) / native CLI / browser-tab / mobile — when scope-review returns we'll know cost per platform
4. **Forkability semantics formalization** — Ben articulated 2026-05-27; not yet a written architectural commitment. Could become Inv-17 / baked-in #18 amendment / new Compromise — orchestrator should propose shape when F-full ratifies
5. **Identity-recovery protocol** — named in v1-assessment-window per CLAUDE.md #15; Phase-4-Meta-Composing scope; would benefit from F-full DAK substrate decisions

### Orchestrator-direct work in flight (parallel to e2r-ffull-scope re-dispatch wait)

Ben (2026-05-27) authorized:
1. **Source-verify 5 Position-B-v2-flagged knowledge-limits** — lidel verbatim quote + MLS default cipher + Sigstore "signed today, deployed 20 years" attribution + NIST SP 800-227 + ANSSI chain. ~30-45 min orchestrator-direct WebFetch.
2. **Compromise #30 cross-link to Compromise #31** in SECURITY-POSTURE.md. ~5 min. (TODO this turn or next)
3. **Codify pim-N §3.5s** (cross-ecosystem-identifier-as-content) in dispatch-conventions. ~15 min. (TODO this turn or next)
4. **Audit which drafts need X-Wing→MLKEM768-X25519 rename** — produce batch-edit list. ~15 min. (TODO this turn or next; partial-audit captured above)
5. **Update all docs** (this handoff doc + CLAUDE.md state banner + others); Ben emphasized this 2026-05-27

### Standing law (apply throughout — UNCHANGED)

- **NEVER `--admin-bypass`** anywhere
- **NEVER force-push** (incl `--force-with-lease`) — exception: orchestrator may force-with-lease on a feature branch when rebasing pre-merge per §2.9 5-step
- **NORMAL `--squash`** for merges (per PR #1356/#1357 precedent: bypass-merge-reinstate pattern for required-failing API drift detector per G-CORE-9 FREEZE)
- **HARD RULE 12** disposition discipline (only 3 valid non-fix-now: OUT-OF-SCOPE / BELONGS-NAMED-NOW with specific destination + entry NOW / DISAGREE-WITH-EXPLANATION)
- **Default prediction = do-it-now** NOT defer (per `feedback_orchestrator_defer_prediction_bias`) — reinforced 2026-05-27 by Ben for F-full scope
- **Full-depth briefs** per `feedback_orchestrator_brief_thoroughness_dont_cut_for_turn_budget`
- **§3.5n ground-truth-verify** every review finding before scoping
- **§3.5h pre-push gate** before any push
- **§3.6j sweep-completeness** self-verify
- **Disk** >85% pre-emptive clean / >92% HARD-ABORT
- **Agent disciplines**: isolation:worktree + run_in_background:true + ABSOLUTE-PATH-FORBIDDEN outside `${WORKTREE_ROOT}` + commit-before-return + full-depth briefs
- **Parallelism cap** ≤7 implementer agents (read-only agents exempt)
- **Plain-English-with-prediction lens** on every Ben-facing surface
- **Convergent-paths-within-night-shift-auth** (Ben 2026-05-25); divergent paths surface
- **NEW 2026-05-26**: §3.5p (Future signed-data designs MUST 3-layer-decompose) + §3.5q (IETF-vocabulary-precision) + §3.5r (industry-folklore-vs-spec-text-distinction)

### Standing posting discipline (applies to all 21 held drafts)

1. Verify links + thread state still resolve before posting
2. Verify cite-anchors (OIDs, draft numbers, dates)
3. Be brief (~150-250 words per draft; resist expansion)
4. Humble framing (5-week-old project; 1 contributor; pre-public-launch)
5. NO "we believe industry should..." language — endorse direction + share shipping evidence
6. NO AI attribution in posted text
7. Cite-anchor every spec/OID/draft/PR reference
8. Apply IETF-vocabulary-precision per §3.5q where applicable

## Pim-N candidates pending codification

1. **§3.5s — cross-ecosystem-identifier-as-content** (Ben-ratified 2026-05-26; should be codified this orchestrator-direct work cycle)
2. **forkability-semantics-formalization** (Ben articulated 2026-05-27; needs design + ratification before codification)
3. **encrypt-everywhere-at-rest-by-default** (Ben articulated 2026-05-27; F-full ratification gates codification)

## Wake-up cadence + dispatch plan

When e2r-ffull-scope-review re-dispatches (when limits reset):
1. Re-dispatch fresh agent with the SAME reframed brief + supplementary clarifications
2. ~2-4hr ETA from re-dispatch
3. On return: synthesize 4-reviewer total picture → surface to Ben → ratify or refine
4. Post-ratification: orchestrator-direct doc cascade (Inv-16 mint + new Compromise # for DAK + CLAUDE.md retense + dispatch-conventions §3.5s + R0 plan major revision + Position B v2 §4 refresh + 21-draft rename pass)
5. Then dispatch wave-cascade: G-CORE-PQ-WIRE-1 canary + G-CORE-3e + DAK-substrate + remote-permission-call + device-link key-wrap + per-platform integrations
6. Throughout: R6 R3 dispatch held until full-wave-cascade lands

## Pre-tag readiness state (HELD)

- `phase-4-meta-core-close` pre-tag readiness: BLOCKED until F-full work cascades through. New blockers added this session:
  - X-Wing-MISLABEL corrective (~24 LOC; cryptographer-review surfaced)
  - F-full encryption substrate decisions ratified
  - Combined-Option-F wire-format additions land
  - Inv-16 mint (3-layer decomposition for encryption isomorphic to Inv-15)
  - DAK substrate landed at appropriate sub-phase
  - Pre-tag external-cryptographer audit on HPKE+MLKEM768-X25519 impl (~1 person-week)
  - R6 R3 + later convergence rounds reach strict-Q5
  - Documentation cascade
- Then `phase-4-meta-close` (after Phase-4-Meta-Composing)
- Then `v1-beta` (after Phase-4-Meta-Composing close)
- Then `v1-GM` (after independent crypto audit completes)

---

## Future-formalization items surfaced this session (not action items yet; surface for awareness)

These are things Ben articulated conceptually but haven't been written as load-bearing commitments anywhere:

### 1. Forkability semantics as architectural commitment

Ben verbatim 2026-05-27: *"as for the future of our project where users can have their data/workflows stored on arbitrary peer engines... I am thinking through our forkability as a right lens it probably actually makes more sense that Alice would be able to see the content she used to share with other still after diverging."*

This translates to architectural commitment: **content-addressed-graph identity model = content is immutable + access control on future content can change + past content stays with whoever already has it**. The opposite of messaging-systems-with-CGKA-leave-forgets semantics.

**Codification options**:
- Inv-17 (sig-bundle-CIDs-not-load-bearing was Inv-15; CGKA-forkability could be Inv-17)
- New baked-in #21 (architectural decision baked in)
- Compromise #N (named honest disclosure that we use forkability not CGKA-leave-forgets)
- Refinement to baked-in #18 (Principal primitive)

**Recommended**: orchestrator surfaces design proposal for Ben ratification after F-full ratifies, then codify wherever lands cleanest.

### 2. Encrypt-everywhere-at-rest as architectural default

Ben verbatim 2026-05-27: *"our default for all nodes whether it's my own engine or whatever should be to have it encrypted at rest from the local engine itself and just ephemerally permissioned as necessary."*

This translates to: **encryption-at-rest is a UNIVERSAL default, not opt-in per use case**. Even on the user's own engine, with their own data, on their own device — data is encrypted at rest + decryption happens via ephemeral capability-bound grants.

**Codification**: baked-in #5 (crypto-agility) amendment OR new baked-in #21 OR Compromise # — to be designed post-F-full.

### 3. Conceptual separation of storage vs access+execution

Ben verbatim 2026-05-27: *"Kind of conceptually separating the storage (local, decentralized, or whatever it happens to be) from access and execution (which could also and separately be local, decentralized, etc)."*

This is the cleanest framing of the F-full architectural vision. Should be codified as a top-level architectural principle in CLAUDE.md, possibly as a new baked-in commitment alongside #17 (engine deployment shapes) since it's the orthogonal axis (storage-shape vs execution-shape, both orthogonal to deployment-shape).

### 4. User-DID-private-key + multi-device-key-wrap design choices

Surfaced in agent supplementary scope clarification:
- User-DID private key MUST be DAK-protected at rest (currently in-process)
- Multi-device key-wrap-on-device-link uses encrypt-to-recipient HPKE-encap (composes with Combined Option F)
- Identity-recovery is DIFFERENT problem (different primitives; later phase)

These would be addressed in the e2r-ffull-scope-review when it returns + Ben ratifies.

### 5. Remote-permission-call-from-another-device design

Ben emphatic 2026-05-27 this is REQUIRED v1-beta scope, not cuttable. The e2r-ffull-scope-review agent's task includes designing this. Specifically: device-discovery + mutual authentication + approval-flow UX shape + cryptographic primitives + any wire-format implications.

### 6. Per-platform credential-store integration

Will be in e2r-ffull-scope-review's package-landscape table. Likely surfaces:
- `keyring-rs` (macOS Keychain / Windows Credential Manager / GNOME Keyring / KDE KWallet / libsecret)
- `tauri-plugin-stronghold` (1Password-like vault for Tauri apps)
- `secrecy` / `zeroize` (memory hygiene)
- `argon2` (password→DAK derivation)
- iOS/Android keystore wrappers (if mobile in scope)
- WebAuthn (for browser-surface auth)
- Hardware-attestation: TPM / secure enclave

Cross-platform abstraction package recommendation will come from the agent.

---

*Authored 2026-05-27 mid-session. Living doc. Updates rolling as work lands + ratifications happen + agent returns.*

---

## 2026-05-27 LATE-SESSION ADDENDUM — End-of-context handoff

### Agent returns + ratifications received this late-session

**e2r-ffull-scope-review RETURNED** (re-dispatch post-rate-limit-reset) at `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae`. Full file `.addl/phase-4-meta/e2r-ffull-scope-review.md` (960 lines).

Recommendation: **Ship full F-full scope across Phase-4-Meta-Core + Phase-4-Meta-Composing, both pre-v1-beta-tag.** Do-it-now bias holds; no piece needs deferring past v1-beta-tag.

**Wave-sequencing**:
- **Phase-4-Meta-Core (~5,000-6,500 LOC)**: X-Wing-mislabel corrective (~24 LOC; INDEPENDENT) + Layer-A real K_principal store (pull G-CORE-3e forward) + Layer-B per-Node AEAD residual + Layer-C encrypt-to-recipient (HPKE-RFC-9180 + MLKEM768-X25519 + multi-stanza + Inv-16) + Layer-D DAK trait + Argon2id substrate + at-rest K_principal/user-DID-key encrypt + multi-device-key-wrap WIRE + remote-permission-call WIRE + minimum desktop platform glue (`keyring-core` + Tauri shell smoke + file-vault fallback)
- **Phase-4-Meta-Composing (~1,300-2,400 LOC)**: Biometric layer + Stronghold optional backend + Device-link UX flow (QR + approval) + Remote-permission-call UX flow + Identity-recovery `RecoveryHook` stub trait

**Decision rule**: wire-format-affecting → Phase-4-Meta-Core (pre-interface-freeze); UX-coupled → Phase-4-Meta-Composing. Both pre-v1-beta-tag.

### 3 concrete tactical picks Ben RATIFIED 2026-05-27

1. **`keyring-core` v1.0.0** (May 2026; NOT legacy `keyring` which self-says "Do not depend on this crate!")
2. **Brendan McMillion's `hpke` crate** (NOT `hpke-rs` from Cryspen which has 13 vulnerabilities Feb 2026)
3. **Signal Provisioning + CTAP 2.2 hybrid-transport inspired protocol shape** for remote-permission-call (QR + ephemeral keypair + signed grant; reuses Layer-C HPKE primitive); pre-merge security mini-review NON-NEGOTIABLE

### Option F+ pseudo-keypair pattern cryptographer review RETURNED 2026-05-27 LATE-SESSION

Agent: `a0bd3aaff8d70cc0e`; pushed at `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f`; full file at `.addl/phase-4-meta/option-f-plus-pseudo-keypair-review.md` (412 lines).

**Recommendation: NO-GO on Option F+ pseudo-keypair pattern.** Pursue Option B-equivalent: **ChaCha20-Poly1305 AEAD-under-DAK for Layer-A vault** + **HPKE-mode-base[MLKEM768-X25519] for Layer-C drop + Layer-D wraps**. The "unified envelope" intuition is correct but **unification belongs at the envelope/codepoint-dispatch layer, NOT at the primitive layer**. Same outer `EncryptedEnvelope { codepoint, payload, aad_binding }` shape, codepoint-discriminated to `SymmetricAead` for vault vs `HpkeBase` for drop/wraps. This IS the CLAUDE.md baked-in #5 crypto-agility pattern operating as designed.

**5 load-bearing findings**:
1. The pattern IS formally sound in the IND-CCA2 reduction sense (RFC 9180 §7.1.3 admits deterministic-derived keypairs; X-Wing's `GenerateKeyPairDerand` is the explicit API). **Soundness is NOT the failure mode.**
2. **The load-bearing concern is a side-channel attack against ML-KEM-768 KeyGen-from-secret-seed via SampleNTT rejection-sampling timing.** Arriaga et al. "Tempo" paper (IACR ePrint 2025/1399) was constructed specifically for this. When seed ρ comes from DAK (password-derived), keygen timing leaks bits of password to co-resident-VM or local-code-execution adversary, enabling online dictionary attack that bypasses Argon2id's memory-hardness. AEAD-under-DAK has no equivalent surface. Mitigations either violate baked-in #5 (vendor patched ml-kem) or wait for upstream constant-time SampleNTT (unscheduled in RustCrypto).
3. **Zero production-system precedent** for "encrypt-to-self under password-derived asymmetric pseudo-keypair." Age, Bitwarden, 1Password, OPAQUE, Molly (Signal fork), IOTA Stronghold — every reviewed vault system uses symmetric AEAD under KDF-derived key. The deterministic-keypair-from-seed pattern exists (BIP32, determin-ed) but only for pubkey-published or signing use cases, never encrypt-to-self vaults.
4. **"One primitive across 4 layers" elegance is superficial.** Layer-B (per-Node AEAD) is structurally symmetric anyway; so it's 3-of-4 not 4-of-4. Audit-surface delta is net-larger not net-smaller under F+ (ADDS pseudo-keypair-from-secret-seed analysis surface).
5. **Right unification = envelope-format layer (codepoint-dispatched), not primitive layer.**

**Net for R0 plan-doc**: structure is now KNOWN — Option B-equivalent with codepoint-dispatched envelope unification. R0 plan-doc authoring can begin once orch-direct doc cascade lands.

### Ben CRITICAL refinement 2026-05-27: ADDL pipeline observance

Ben caught orchestrator proposing to jump directly from "F-full ratified" → "dispatch wave-cascade (R5-style)" without going through the full ADDL pipeline for the F-full new scope.

**Corrected understanding**: F-full is NEW scope that emerged during R6 R2 FP cycle ("tangential exploration"). It has the e2r-ffull-scope-review as R0-INPUT but NOT a proper R0 plan-doc. Per CLAUDE.md ADDL Pipeline section + `feedback_iterate_critical_reviews_to_convergence`, F-full needs its own full ADDL pipeline before R6 R3 can converge the post-F-full state.

**Corrected proceed plan** (ratified by Ben):

```
Phase-4-Meta-Core close path (CORRECTED):

1. Orchestrator-direct doc cascade (CAN DO NOW; records ratified decisions)
   - Tracked-doc PR: Inv-16 mint in INVARIANT-COVERAGE.md (15→16) + new Compromise # in SECURITY-POSTURE.md for DAK substrate + Compromise #30 cross-link to #31 + Category A rename application (per x-wing-to-mlkem768-x25519-rename-audit.md) + bypass-merge-reinstate
   - Orchestration-branch: CLAUDE.md baked-in #5 retense + #18 amendment (forkability + encrypt-everywhere-as-default + storage-vs-access-execution-separation) + dispatch-conventions amendments

2. Wait for Option F+ pseudo-keypair cryptographer review return

3. Author F-full R0 plan-doc consolidating e2r-ffull-scope-review wave-sequencing + Option F+ outcome

4. F-full R1 critic council (5-7 lenses per Pattern 6; iterate to convergence per Q5)

5. F-full R2 test landscape synthesis (1 agent)

6. F-full R3 test-writer dispatch (N parallel per R2; per feedback_r3_agent_count_dynamic)

7. F-full R4 test review (2-3 lenses; iterate to convergence)

8. F-full R5 implementation wave-cascade (canary-first per feedback_canary_first_parallel_implementation):
   - Wave 1 (canary): X-Wing-mislabel corrective + Layer-B residual + 21-draft rename pass (Category A code + comment drafts)
   - Wave 2: Layer-A K_principal store (G-CORE-3e pulled forward)
   - Wave 3 (canary): Layer-C encrypt-to-recipient (HPKE + MLKEM768-X25519 + multi-stanza)
   - Wave 4: Layer-D DAK substrate + Argon2id + at-rest encryption
   - Wave 5: Layer-D wire-format pieces (multi-device-key-wrap + remote-permission-call)
   - Wave 6: Layer-D platform glue (keyring-core + Tauri smoke + file-vault fallback)

9. F-full R4b post-implementation test review (iterate to convergence)

10. R6 R3 phase-close council (full N-lens; evaluates post-F-full state; iterate to strict-Q5)

11. Pre-tag sweep + tag phase-4-meta-core-close (awaits Ben check-in)

Phase-4-Meta-Composing path (CORRECTED):

12. Full ADDL pipeline for Phase-4-Meta-Composing (R0 plan + R1 + R2 + R3 + R4 + R5 [biometric + Stronghold + device-link UX + remote-permission UX + identity-recovery RecoveryHook stub + self-composing admin] + R4b + R6 + pre-tag sweep + tag phase-4-meta-close)

13. Tag v1-beta (after BOTH Phase-4-Meta-Core AND Phase-4-Meta-Composing close)

14. External cryptographer audit (3 person-weeks; parallel-with-some during v1-beta → v1-GM window)

15. Tag v1-GM (after audit lands; replaces single v1 per 2026-05-19 reframe)
```

### Honest timeline estimate (corrected)

The agent's "4-5 weeks to v1-beta-tag" was R5-only and 2-3× optimistic. With full ADDL:

**v1-beta-tag honest estimate: ~7-15 weeks (~2-4 months) from 2026-05-27.**

Breakdown:
- Doc cascade + Option F+ return: ~1-3 days
- F-full R0/R1/R2/R3/R4 (pre-R5): ~10-21 days
- F-full R5 wave-cascade: ~7-14 days
- F-full R4b/R6 R3 + convergence iterations: ~7-20 days
- Phase-4-Meta-Core close subtotal: ~26-60 days
- Phase-4-Meta-Composing full ADDL: ~21-45 days
- Subtotal to v1-beta-tag: ~47-105 days (~7-15 weeks)
- External audit + v1-GM-tag: +21+ days

### 2 new memory files codified 2026-05-27 LATE-SESSION

Both at `/Users/benwork/.claude/projects/-Users-benwork-Documents-benten-engine/memory/`:

1. **`feedback_phase_ordering_precision.md`** — re-read CLAUDE.md baked-in #15 before scope-claim surfaces; v1-beta tagged AFTER both Phase-4-Meta-Core AND Phase-4-Meta-Composing close
2. **`feedback_addl_pipeline_full_observance.md`** — new architectural-scope work MUST go through full ADDL pipeline before R6 council convergence; e2r-style outputs are R0-INPUT not R0-PLAN

MEMORY.md index updated under "Review composition + convergence" subsection.

### Position B v2 + Shape 5 + 21 drafts STATUS

**Ben earlier decision (2026-05-26)**: hold all 21 drafts until F-full + rename ratify; batch-post in coherent landing.

**Late-session refinement (2026-05-27)**: asymmetry surfaced — F2 multicodec PR #400/#403 endorsements don't cite X-Wing AND have narrowing window; Shape 5 Email-1 doesn't cite X-Wing AND could start 28-day iroh-response clock independently. F7 W3C CCG #70/#74 stable-timing. Other 18 drafts batch with rename. Ben deferred decision on whether to advance F2/Shape-5-Email-1 independently.

### Active background agent at end-of-session

| Agent ID | Topic | Branch on completion | ETA |
|---|---|---|---|
| `a0bd3aaff8d70cc0e` | Option F+ pseudo-keypair pattern cryptographer review | `phase-4-meta-core/option-f-plus-pseudo-keypair-review` | ~1.5-3hr from late-session dispatch |

**Critical for next session**: when Option F+ returns, harvest + drop worktree + use findings to inform R0 plan-doc structure (one-primitive vs three-primitives shape).

### IMMEDIATE NEXT-ACTION queue for next session

1. **FIRST**: Verify Option F+ agent state (returned? still running? rate-limited again?). Harvest findings if returned.
2. **Orchestrator-direct doc cascade**: tracked-doc PR (Inv-16 + Compromise # + #30 link + Category A rename) + orchestration-branch CLAUDE.md retense + dispatch-conventions amendments
3. **F-full R0 plan-doc authoring** (once Option F+ returns)
4. **F-full R1 critic council dispatch** (after R0 plan-doc lands)
5. Continue through R1→R2→R3→R4→R5→R4b→R6 R3
6. Then Phase-4-Meta-Composing ADDL
7. Then v1-beta-tag

*Updated 2026-05-27 LATE-SESSION. Compact-survival snapshot for next-session pickup.*

---

## 2026-05-27 POST-COMPACT RE-ORIENT ADDENDUM — F+ second-opinion + 3rd-reviewer dispatched + CT-SampleNTT investigation

Session resumed post-compact 2026-05-27. Ben asked the previously-not-explicitly-surfaced question: **is the Option F+ NO-GO actually settled, or did orchestrator fold it into "ratified" without explicit Ben sign-off?** Surfaced the full first-agent NO-GO reasoning in plain English with my-pred + ground-truth-verify; Ben chose to dispatch a second-opinion crypto agent focused on the §6.2 envelope-layer-unification alternative.

### Second-opinion review RETURNED (agent `af962c76177a0b954`, completed ~528s wall-clock)

**File**: `.addl/phase-4-meta/option-f-plus-second-opinion-cryptographer-review.md` on `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` (520 lines).

**Verdict**: **CONCUR-WITH-AMENDMENTS** on Option F+ NO-GO. **§6.2 envelope-layer-unification design APPROVED with two amendments.**

**The 2 amendments to §6.2 (load-bearing for final design)**:
1. **Codepoint MUST be committed inside the canonicalized AAD / info-string** for every Seal/Open call across both `EnvelopePayload` variants. Without this, an adversary who flips the discriminator byte gets a cross-codepoint attack surface (e.g., rewriting a Layer-A vault file's codepoint to LAYER_C_DROP_TO_RECIPIENT forces engine to dispatch HPKE-Open path on AEAD bytes, exposing fresh side-channel measurement on the vault sk per Bernstein–Persichetti "One Time is Enough" 2024).
2. **Strict-decode discipline with codepoint→variant-tag dispatch + no cross-variant fallback.** Decoder MUST NOT attempt cross-variant decoding on the same bytes.

**3 new independent findings beyond first agent's review**:
- **§3.3 ML-KEM CBD-sampling is a SECOND side-channel surface stacked on SampleNTT** — even hypothetical-CT-SampleNTT wouldn't fully fix F+. Reinforces NO-GO.
- **§4.1 Structural pseudo-pubkey-as-identifier oracle attack INDEPENDENT of timing** — if the vault pseudo-pubkey were ever observed (current design says it won't be, but architectural extensibility could change that — KCV, routing identifier, etc.), each password candidate becomes confirm-or-reject via pubkey-derivation determinism. AEAD-under-DAK has no derived pubkey → no such oracle. **This is the load-bearing finding** — even in a counterfactual "CT-everything" universe, F+ is the worse design because of this structural property.
- **§3.5 PESTO / HPAKE / SPEKE precedent literature** strengthens the "no precedent" argument with cryptographic-research-history (the prior agent's "no precedent" was directionally correct but understated; there IS literature on this class, and it reaches the same NO-GO conclusion).

**Calibration pushback (without reversing conclusion)**: confidence on side-channel magnitude should be MEDIUM not MEDIUM-HIGH on *practical* reachability; but decision-asymmetry still warrants HIGH overall recommendation confidence.

**Inv-16 phrasing recommendation (§4.3)**: **primitive-neutral** framing — articulate the layer/role separation (identity-blob-CID vs ephemeral-permission-CID vs envelope-CID, or analogous) without locking to a primitive choice; leaves crypto-agility seam intact for future-additive codepoints (incl. F+ revisit if its blockers are ever closed).

### Orchestrator-direct CT-SampleNTT investigation findings

Ben's curiosity: *"is constant-time SampleNTT not implemented in RustCrypto ml-kem something other people actually care about? are there existing discussions/whatever? if not should we open one (or even build it as a side-quest PR)?"*

| Signal | Finding |
|---|---|
| IETF [draft-sfluhrer-cfrg-ml-kem-security-considerations-04](https://www.ietf.org/archive/id/draft-sfluhrer-cfrg-ml-kem-security-considerations-04.html) (Nov 2025; Informational; multi-org WG-track — Cisco / NIST / Ericsson / Quantinuum / Arqit) | **Explicitly identifies the F+ use case as the load-bearing exception**: *"One exception is in some methods that implement Password Authenticated Key Exchange with ML-KEM, where the public key may be encrypted with the password. In this rather narrow use case, this variable timing needs to be taken into account."* Then concedes: *"Converting this into a constant time operation is expensive enough that it is rarely done."* |
| Tempo paper IACR ePrint 2025/1399 (Arriaga / Barbosa / Boyen) | Published mitigation blueprint; not yet implemented in production Rust impls |
| RustCrypto/KEMs `ml-kem` README | **"never been independently audited! USE AT YOUR OWN RISK!"** |
| RustCrypto/KEMs [Issue #25](https://github.com/RustCrypto/KEMs/issues/25) "Evaluate whether compilation introduces a secret-dependent branch" | OPEN since 2024-06-03 (~24 months stale); cites Kyber `poly_frommsg` clang bug; **no comments, no Tempo / SampleNTT cross-reference** |
| RustCrypto/KEMs Tempo-specific tracker | **None** — clear contribution gap |
| Maintenance velocity | Active — ml-kem v0.3.0 / v0.3.1 / v0.3.2 cut April–May 2026; PR #289 "avoid UDIV in compiled output" shows CT-awareness in active development |
| Cryspen [`libcrux-ml-kem`](https://docs.rs/libcrux-ml-kem) v0.0.9 | Formally verified via hax + F*; "secret independent" proven; **pre-1.0**, not independently audited, doesn't specifically address SampleNTT in public material. Different crate from `hpke-rs` (which had 13 CVEs Feb 2026); same org (Cryspen), separate codebase, separate maturity. |

**Net for Benten**: the gap is real and acknowledged ecosystem-wide. Per Ben's framing — *"if CT-SampleNTT already existed would it be the more ideal/permanent/elegant/stronger shape than the alternative we're considering?"* — **NO**: even in a counterfactual "CT-everything" universe, F+ would still NOT be Benten's better design choice due to (a) second-opinion's §4.1 structural oracle (timing-independent), (b) first-agent's §5 architectural-utility argument (3-of-4 not 4-of-4; different security shapes per layer), (c) second-opinion's §3.3 second-side-channel-surface (CBD-sampling stacked on SampleNTT).

**Side-quest scope decision (Ben-ratified 2026-05-27 post-compact)**: **Level 1 only — open RustCrypto/KEMs issue (~30 min)**. Title: "ml-kem: track Tempo-class constant-time SampleNTT for non-public-seed use cases (PAKE, vault unlock, etc.)"; body: cite Tempo + IETF draft-sfluhrer + Issue #25 cross-link + libcrux-ml-kem context + Benten's encrypt-to-self design as "we evaluated this and chose symmetric-AEAD-under-KDF" downstream-consumer datapoint. Timing: **after F-full doc cascade lands** (Ben's choice; issue text will be cleaner once Benten's own decision is recorded).

### 3rd adversarial-design reviewer dispatched (`aa60741867edfefc6`; ~1.5-3hr ETA)

Per second-opinion's recommendation + Ben's belt-and-suspenders choice. Adversarial / red-team posture explicitly: try to BREAK §6.2-with-Amendments-1+2. Default to DISAGREE if substantive grounds found; CONCUR-WITH-NEW-EVIDENCE if not. Brief enumerates 12 specific attack vectors as starting points (cross-codepoint confusion / BindingContext substitution / codepoint enum extension hazards / strict-decode edge cases / AAD-binding completeness / domain-separation / AEAD-vs-HPKE composition / §4.1-style structural oracles / cross-layer attacks / long-term key-rotation / impl-bug surface / multi-stanza HPKE composition).

Branch on completion: `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design`.

### Tracked-doc PR cascade STATE

- Branch `phase-4-meta-core/inv-16-compromise-dak-rename-cat-a` CREATED off main `2172cb6d`
- **PAUSED** pending 3rd-reviewer return; Inv-16 phrasing should follow §4.3 primitive-neutral framing
- Ben **pre-authorized** bypass-merge-reinstate for this specific doc-only PR (G-CORE-9 FREEZE precedent: PR #1356/#1357)
- When 3rd-reviewer ratifies: resume + Inv-16 mint (primitive-neutral) + new Compromise # for DAK substrate (incl. Option F+ NO-GO rationale + §4.1 structural-oracle finding + revisit-triggers) + Compromise #30→#31 cross-link + Category A rename application + atrium test count 15→16 + open PR + execute bypass-merge-reinstate

### Standing law refresher (UNCHANGED)

All prior standing law applies. New process datum: **`feedback_review_finding_ground_truth_verify` is what caught the previously-not-surfaced F+ ratification gap** — orchestrator had folded agent NO-GO into "ratified" via the LATE-SESSION ADDENDUM without explicit Ben sign-off. Ben caught it post-compact: *"we never went over that. can you give me all the details? are we sure it's a NO GO?"* Correct discipline going forward: any agent finding that becomes architectural-commitment text MUST be surfaced explicitly with my-pred + plain-English framing for Ben ratification, EVEN IF the agent's finding aligns with orchestrator's expected path. This is `feedback_surface_arch_decisions_under_auth` operating correctly + `feedback_review_finding_ground_truth_verify` as the cross-check.

### IMMEDIATE NEXT-ACTION queue (refreshed)

1. **WAIT**: 3rd-reviewer returns (~1.5-3hr); harvest findings; drop worktree
2. **If 3rd-reviewer CONCUR**: ratify F+ NO-GO final + resume tracked-doc PR cascade (Inv-16 primitive-neutral + Compromise # for DAK + Compromise #30→#31 + Cat A rename + test count 15→16 + open PR + bypass-merge-reinstate)
3. **If 3rd-reviewer DISAGREE**: surface to Ben with full reasoning + my-pred (most likely path: even if DISAGREE, the disagreement focuses on amendments-to-amendments rather than wholesale design reversal; we adapt then proceed)
4. **Orch-direct (post-cascade)**: orchestration-branch CLAUDE.md baked-in #5 retense + #18 amendment + dispatch-conventions §3.5s amendment
5. **Orch-direct (post-cascade)**: Side-quest Level 1 — open RustCrypto/KEMs CT-SampleNTT issue (~30 min)
6. **Author F-full R0 plan-doc** consolidating e2r-ffull-scope-review wave-sequencing + Option F+ NO-GO outcome + §6.2 + Amendments 1+2 into proper R0 plan
7. **F-full R1 critic council** (5-7 lenses; iterate to convergence)
8. Continue ADDL pipeline: R2 → R3 → R4 → R5 (canary-first; Wave 1 = X-Wing-mislabel corrective + Layer-B residual + 21-draft rename pass) → R4b → R6 R3 → pre-tag sweep → tag `phase-4-meta-core-close`
9. Then Phase-4-Meta-Composing full ADDL pipeline → tag `phase-4-meta-close` → tag `v1-beta` → external crypto audit (~3 person-weeks) → tag `v1-GM`

*Updated 2026-05-27 POST-COMPACT. Captures F+ second-opinion outcome + 3rd-reviewer dispatch + CT-SampleNTT investigation. Next compact-survival update lands when 3rd-reviewer returns + ratification decision settles.*

---

## 2026-05-27 LATE-EVENING ADDENDUM — 9-eyes panel + consolidation + critique-round in flight

Session continued through the day. 3rd-reviewer returned CONCUR-WITH-CALIBRATION + 4 load-bearing amendments + 2 minor. Ben asked for comprehensive coverage of all dimensions → dispatched 6 parallel lens reviewers (L4 impl-engineering + L5 threat-model+audit + L6 privacy + L7 cross-ecosystem-interop + L8 wire-format-stability + L9 atrium-integration). All 6 returned successfully. CT-Decap impl evaluation done orchestrator-direct in parallel; Ben open to Option B (libcrux-ml-kem).

### 9-eyes panel COMPLETE — unanimous F+ NO-GO + design-direction concur

| Lens | Branch | Verdict |
|---|---|---|
| L1 1st cryptographer | `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` | NO-GO on F+ pseudo-keypair; §6.2 envelope-layer-unification |
| L2 2nd-opinion cryptographer | `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` | CONCUR-WITH-AMENDMENTS (1+2) |
| L3 adversarial-design | `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3` | CONCUR-WITH-CALIBRATION (Amendments 3-6 + 2 minor) |
| L4 impl-engineering | `phase-4-meta-core/option-f-plus-lens-l4-impl-engineering @ 4d4aae5f` | CONCUR-WITH-IMPL-AMENDMENTS (6 impl-amendments; recommends libcrux-ml-kem; LE-vs-BE conflict; ~9-10wk cost) |
| L5 threat-model+audit-readiness | `phase-4-meta-core/option-f-plus-lens-l5-threat-model-audit-readiness @ 3f27f8e0` | AUDIT-READY-IN-DIRECTION (11 Compromise mints + 3 invariants + THREAT-MODEL.md) |
| L6 privacy/metadata-leak | `phase-4-meta-core/option-f-plus-lens-l6-privacy-metadata-leak @ 986e50bb` | CONCUR-WITH-CALIBRATION + DISAGREE on load-bearing-final (6 amendments + Sealed-Sender slot) |
| L7 cross-ecosystem-interop | `phase-4-meta-core/option-f-plus-lens-l7-cross-ecosystem-interop @ 208f98bb` | INTERNALLY-SOUND-BUT-§3.5s-NON-COMPLIANT (HPKE-11-KE correction + 2 amendments) |
| L8 wire-format-stability | `phase-4-meta-core/option-f-plus-lens-l8-wire-format-stability @ d8d3c41c` | CONCUR-WITH-EXTENSIONS (8 amendments + CodepointLifecycle) |
| L9 atrium-integration | `phase-4-meta-core/option-f-plus-lens-l9-atrium-integration @ 1670aa03` | CONCUR-WITH-CALIBRATION (5 amendments A1-A5; dual-CID disagreement with prior P2P-architect) |

### Consolidation agent landed — unified registry on `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16`

**File**: `.addl/phase-4-meta/option-f-plus-9-eyes-consolidated-registry.md` (939 lines).

Outputs:
- **28 unified amendments** (raw ~46 deduplicated)
  - 18 LOAD-BEARING wire-affecting at v1-beta-freeze
  - 6 RECOMMENDED v1-beta (impl-engineering + doc-deliverable)
  - 4 NAMED-DEFERRED with revisit-triggers
- **13 new Compromise mints (#32-#44)** + extension to #31
  - Most-load-bearing: #32 (B-P Decap) + #43 (envelope metadata leakage)
- **3 unified invariants**: Inv-16 (envelope-unification primitive-neutral) + Inv-17 (hybrid-mandatory) + Inv-18 (codepoint-registry + metadata-disclosure)
- **5 disagreements** for Ben (Q1-Q5; held open per Ben's "let critics surface with full reasoning" choice)
- **4 pattern-induction meta-findings**
- **v1-beta cost estimate**: ~9-10 calendar weeks (~35-45 wave-days; compresses to ~7 weeks if R3/R5 briefs absorb upfront)

### The 5 disagreements (consolidator advisory; held open)

| Q# | Decision | Consolidator advisory | Confidence |
|---|---|---|---|
| Q1 | Option A (oqs-rs) vs Option B (libcrux-ml-kem) | libcrux-ml-kem | HIGH |
| Q2 | LE-vs-BE codepoint endianness in existing aead.rs | Migrate to BE pre-freeze (~1 wave-day) | MED-HIGH |
| Q3 | L9 dual-CID vs prior P2P-architect recipient_set-in-CID | L9 dual-CID | HIGH |
| Q4 | HPKE-11 vs HPKE-11-KE mode | HPKE-11-KE if key-encryption mode; verify at R0 §4 | MED-HIGH |
| Q5 | Sealed-Sender as v1-beta-DEFAULT vs RESERVED-FUTURE-ADDITIVE | Additive slot at v1-beta | MED (Ben call) |
| Q5b (consolidator-surfaced) | Am4 sender-DID-in-AAD vs L6 metadata-leak tension | Both win (keep Am4 default + reserve Sealed-Sender slot per Q5) | — |

### 5 critique-of-consolidation agents dispatched in parallel

Per Ben's direction ("another review round of this triage and the open questions/options and whether they have any other critiques or more elegant final solutions to things for a final re-triage prior to R0 plan writing"). Each sees the consolidator's recommendations + meta-findings (treated as advisory not load-bearing); Q1-Q5 held open per Ben's "let critics surface with full reasoning" choice.

| Critique # | Lens | Branch on completion |
|---|---|---|
| C1 | Elegant-shape extra-reflection-pass (per `feedback_extra_reflection_pass_for_elegant_permanent_shape`) | `phase-4-meta-core/option-f-plus-critique-c1-elegant-shape` |
| C2 | Cross-amendment composability (3rd-reviewer's MED-HIGH residual concern; now actionable with full registry) | `phase-4-meta-core/option-f-plus-critique-c2-composability` |
| C3 | Fresh-eyes cryptographer on composed whole (re-evaluates Q1-Q5 in full-registry context) | `phase-4-meta-core/option-f-plus-critique-c3-fresh-eyes-cryptographer` |
| C4 | Process-discipline + pattern-induction (Pattern 6 effectiveness; consolidator value-add; codification candidates) | `phase-4-meta-core/option-f-plus-critique-c4-process-discipline` |
| C5 | Formal-methods coverage-gap (consolidator flagged no-formal-methods-lens; targets / tools / cost estimate / `docs/SECURITY-PROOFS.md`) | `phase-4-meta-core/option-f-plus-critique-c5-formal-methods` |

ETA: ~1.5-2hr per agent in parallel; total wall-clock ~1.5-2hr from dispatch.

### Active background agents at write-time

| Agent ID | Topic | Branch on completion |
|---|---|---|
| `ade1e7f784026c8cc` | C1 elegant-shape | `phase-4-meta-core/option-f-plus-critique-c1-elegant-shape` |
| `a6c26f0c494e13bbb` | C2 composability | `phase-4-meta-core/option-f-plus-critique-c2-composability` |
| `a849e7cda77948587` | C3 fresh-eyes | `phase-4-meta-core/option-f-plus-critique-c3-fresh-eyes-cryptographer` |
| `a4c3ef3c8e48121b9` | C4 process-discipline | `phase-4-meta-core/option-f-plus-critique-c4-process-discipline` |
| `a8b694f1ff980aaa4` | C5 formal-methods | `phase-4-meta-core/option-f-plus-critique-c5-formal-methods` |

### IMMEDIATE NEXT-ACTION queue (refreshed)

1. **WAIT**: 5 critique agents return (~1.5-2hr; staggered task-notifications)
2. **For each return**: harvest findings + drop worktree
3. **When all 5 return**: synthesize critique-round into final re-triage surface for Ben
4. **Ben final ratification**: Q1-Q5 resolutions + amendment subset + Compromise # mints + invariants + any critique-induced reductions
5. **RESUME tracked-doc PR cascade**: Inv-16/17/18 mints + N new Compromise # mints + Compromise #30→#31/32 cross-link + Cat A rename + atrium test count + new docs (THREAT-MODEL.md per L5; CRYPTO-CODEPOINTS.md per L7+L8; possibly SECURITY-PROOFS.md per C5) + bypass-merge-reinstate (pre-authorized for this PR only)
6. **Orchestration-branch updates**: CLAUDE.md baked-in #5 retense + #18 amendment + dispatch-conventions §3.5s amendment with HPKE-11-KE naming + multicodec codepoint corrections per L7
7. **Side-quest Level 1**: open RustCrypto/KEMs CT-SampleNTT issue (~30 min; post-cascade)
8. **Author F-full R0 plan-doc** per ratified design (consolidator's §7 skeleton + critique-round refinements)
9. **F-full R1 critic council** (Pattern 6 with possibly-new standing lenses per C4); iterate to convergence per Q5
10. Continue full F-full ADDL pipeline + then Phase-4-Meta-Composing pipeline + then v1-beta-tag + external audit + v1-GM-tag

### Compact-survival pointer

If you resume after compaction: read this file + the consolidator's output at `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` first. All 9 + 5 critique reviews are on their respective origin branches. The 9-eyes panel + consolidation are SETTLED; the critique-round is the post-consolidation reflection-pass per `feedback_extra_reflection_pass_for_elegant_permanent_shape`. After critique-round returns: surface final re-triage to Ben + ratify subset + RESUME tracked-doc PR cascade + author R0 plan-doc.

*Updated 2026-05-27 LATE-EVENING. Compact-survival snapshot for next-session pickup or continued live session.*

---

## 2026-05-28 TODAY-SESSION ADDENDUM — MembershipSet unification panel + N-refinements + M-CONS-v2 + critics in flight

Session continued through 2026-05-28. Substantial architectural-decision day. **30+ agent dispatches** across F+ ratifications resolved earlier today + MembershipSet unification panel (3 catalogers + 5 specialists + M-CONS-v1 + 4 N-refinements + M-CONS-v2 + 3 critics in flight). Ben ratified many decisions; final design captured at `phase-4-meta-core/membership-set-m-cons-v2-consolidator @ e62ff540`.

### F+ ratifications resolved earlier today (post-compact)

| Decision | Resolution |
|---|---|
| Q1 ML-KEM impl | **libcrux-ml-kem** (per L4 IMPL-A1) — Ben open + ratified |
| Q2 Codepoint endianness | **Migrate LE → BE** in `aead.rs` pre-v1-beta-freeze |
| Q3 CID design | **DUAL-CID** (envelope_blob_cid + plaintext_cid_set HMAC-blinded with K_Set); recipient computes BLAKE3-of-plaintext locally as impl detail; plaintext_cid_local NOT on wire (Ben's catch). Option I `dedup_scope_id: Option<DedupScopeId>` NAMED-deferred as future-additive elegant superset. |
| Q4 HPKE-11-KE | **Internal commit at v1-beta** (wire-format-affecting; ~0 work); **JOSE-emit adapter NAMED-DEFERRED** to V1-FROZEN-INTERFACE-DEFERRED.md with revisit-trigger = first non-Benten JOSE consumer |
| Am4 Sender-DID-in-AAD | **Sealed-Sender DEFAULT** (drop parallel non-sealed-sender codepoint; aggressive-privacy posture per Ben's "if additive means saving to do later I'd recommend doing it now") |
| Path-A vs Path-B (L11 structural fork) | **Path-A.5 hybrid**: keep Anchor+Version+CURRENT pattern + key encryption to immutable Version-Node-CIDs (vindicates Ben's "versioned nodes precedent" intuition); Path-B (rewrite + Willow-shim) = **DISAGREE-WITH-REASONING** per HARD RULE 12 clause-(c) (no v1-beta or foreseeable use case for Willow-interop) |

### Ben's MembershipSet unification insight (load-bearing)

Ben surfaced 2026-05-27 LATE: *"'Atrium' (with multiple users syncing sub-graphs) AND Single User Multi Device AND Single User Single Device are all versions of the same primitive thing."*

Investigated through the largest agent-panel of the session:

#### 3 catalogers (existing-state)

- **M1a multi-device-sync** `@ 3618e051` (452 LOC) — substantial existing impl (AtriumHandle / HandshakeFrame / Loro CRDT / DeviceAttestation V2 / D-C HYBRID); flagged **4-identity-concepts tree (CLAUDE.md #18)** as load-bearing complication for MembershipSet collapse
- **M1b Atrium-membership-sharing** `@ 1816ea60` (658 LOC) — partial-impl (12 methods); **TENTATIVE STRONG-YES on MembershipSet collapsibility** (8/12 ops fit cleanly; 2 caveats; 2 do NOT fit — register_zone + freshness_window); Garden Atrium-of-Atriums (per L9 O7) recursable
- **M1c key-management** `@ 50eb901d` (886 LOC) — K_principal STUB at `redb_backend.rs:117-210`; structural KDF chain LIVE per Spike E (this IS Cryptree-aligned arbitrary-depth pattern Ben recalled); DAK + K_Atrium + Layer-C HPKE NOT BUILT

#### 5 specialists

- **M2 primitive design** `@ 6170980b` (901 LOC) — typed-variant `enum MembershipSetKind { Atrium, DeviceMesh, SingleDevice }` (EXACTLY-3 arms per §15.c); 7-op API; DUAL-CID per Ben's catch
- **M3 amendment transformation** `@ 1ba3a4c3` (833 LOC) — **NET-ELEGANCE-WIN MODEST** at MED-HIGH confidence; -4.3 to -5.5 wave-days; 5 ELIMINATED + 17 SIMPLIFIED + 6 EXPANDED + 2 RENAMED; new Inv-20; Compromise #45-#51 renumbered
- **M4 red-team** `@ 066785b5` (764 LOC) — CONCUR-WITH-AMENDMENTS @ 75-80%; 5 BREAKs (DAK ≠ K_Atrium / K_principal-cannot-replace / forkability-Atrium-only / kick-differs-per-kind / homogeneous-members-per-kind) — mostly addressed by M2's typed-variants
- **M5 CGKA candidate survey** `@ 15819500` (533 LOC) — **KEEP no-CGKA for v1-beta**; rename CGKA-LITE → **MultiRecipientSealing** (truth-in-naming; design doesn't deliver FS/PCS that "CGKA-LITE" name implied); MLS-PQ **NOT beta-production-ready** (Revised I-D Needed; OpenMLS PQ branch ships X-Wing NOT in draft); **Cryptree NOT a CGKA** (factual correction — it's a key-derivation tree for filesystem read access; Ben's intuition actually corresponds to Atrium-fork semantic Ben ratified 2026-05-27)
- **M6 transport-configurability** `@ e5046c87` (531 LOC) — codepoint-reserve at v1-beta initially; iroh-gossip LAYERS as overlay (not replacement); per-MembershipSet matrix; **Ben subsequently overrode to SCOPE-IN iroh-gossip AT v1-beta (+5-8 wave-days; per Q1 ratification)** so impl ships not just slot-reserved

#### M-CONS-v1 consolidator `@ 74580ee6` (1081 LOC)

10/10 composition axes converge; 28 unified amendments + 13 Compromise mints + 3 invariants → expanded under M3 transformations.

#### 4 N-refinement specialists (post-M-CONS-v1)

- **N1 sub-graph sharing elegance** `@ ed592770` (1067 LOC) — surveyed 9 systems (Cryptree / CP-ABE / PRE / Tahoe-LAFS / UCAN/ZCap / Jazz / IPFS / PESTO/Cwtch/Briar / Yjs-Automerge); recommends **Option H NESTED-SPEC** — promote in-code combinators (already shipping `g-core-3w/subgraph-spec-walker @ 5c2947c8` proptest-verified) to first-class grant-time composition primitive: `Scope::RestrictedSelector { scopes: Vec<RestrictedScope>, audit_commitment: Option<Cid> }`. **Benten is at field state-of-the-art**; no foreign primitive needed. +0.95-1.2 wave-days. Preserves all 5 frozen-surface disciplines.
- **N2 generic-MembershipSet + RBAC + ops** `@ a1b5a552` (1399 LOC) — **HYBRID-RECOMMENDED**: keep typed `MembershipSetKind` discriminator AT v1-beta BUT generalize WITHIN typed shell via **uniform `authorities: BTreeSet<Authority>` slot** replacing M-CONS-v1 §5.1 KindPolicyAdminEntity enum (Ben's "Admin in Atrium = User in DeviceMesh = Self in SingleDevice" intuition correct on signing-authority axis). **3-role RBAC** (Admin > Member > Viewer) + 3-permission (Read/Write/Admin) + UCAN-composed intersection-of-allows. **24 operations enumerated**: 11 v1-beta-LB (incl. 4 new: change_role / member_key_rotation / self_leave / rename/update_metadata; + audit_log_query + update_policy_value sugar) + 6 codepoint-reserve + 3 Phase-4-Meta-Composing + 4 post-v1-beta. Added `MembershipSetMetadata` closing M2's UX gap. +1.5-2.5 wave-days.
- **N3 continuous-rotation edge cases** `@ 298c80d9` (506 LOC) — stress-tested 6 regulatory + 6 use-case + 6 attack scenarios; **M5 HOLDS** at HIGH 85% confidence; wire format is additive (continuous-rotation can ship post-v1-beta without break); 3 doc-only sharpenings (RotationTrigger doc-enum + clarify `policy.refresh_required_secs` is attestation-refresh NOT K_Set rotation + mint Compromise #54/#55/#56). **GDPR-RTBF specifically needs per-subject crypto-shredding at Atrium-data-layer (separate problem; Compromise #55)** — Ben clarified framing: P2P-by-design semantic, NOT "limitation we should fix."
- **N4 Signal Sender Keys comparison** `@ 2ee24e9f` (494 LOC) — **CONCUR-WITH-AMENDMENTS**: Benten's MultiRecipientSealing simpler/stronger than SSK in 9-of-10 scenarios (PQ-hybrid day-one / forkable Atrium / content-addressed DAG / trustless storage / first-class DeviceMesh / recursive composition reserve). SSK has NO PCS at protocol level + member-remove O(N²) + pre-quantum + libcrux-AGPLv3-blocking. 3 doc-only refinements: rename `AtriumWithCGKA` → `AtriumWithRotatingGroupKey`; add `RotatingGroupKeyChainedMode` codepoint-reserve sub-slot; add SSK-comparison §-row to `docs/SECURITY-POSTURE.md`. ~1 wave-day.

#### M-CONS-v2 consolidator `@ e62ff540` (1295 LOC)

Final-final integrated design:
- **31 F-amendments** (F1-F28 + F-N2-A/B/C; F-A1+F-A2 absorbed; F13 renamed AtriumWithRotatingGroupKey; F27 SCOPE-IN per Q1; F28 NEW N1 RestrictedScopeSet)
- **26 Compromise mints** (25 mints + #31-ext; #53 narrowed; #54/#55/#56 NEW)
- **Inv-20 extended to 10 clauses** (clause-i uniform Authority + clause-j 3-role RBAC + UCAN intersection-of-allows from N2)
- **Cost central**: ~85-111 wave-days (vs original ~80-101; net +5 to +10 dominated by Q1 iroh-gossip SCOPE-IN)
- **Wave-MS-PRIMITIVE** ~10.5-13 wave-days canary (absorbs N1 + N2)
- **Wave-MS-TRANSPORT** ~5-8 wave-days (iroh-gossip impl per Q1)
- **4 remaining open Ben-calls** in §9.3 (none arch-fork-class)
- **8 new pattern-induction findings** P11-P18 codified

### Conceptual clarifications surfaced today

- **Loro = CRDT layer; iroh = transport layer** (different layers; both needed). Benten uses regular `iroh` crate v1.0.0-rc.0 + `loro 1.12` (high-perf CRDT for collaborative editing) + tokio for async + `iroh-blobs` integration pending G-CORE-3e. Live-sync foundations ALREADY in place; iroh-gossip adds broadcast-overlay for scaling (Class B "live feel"; sub-100ms via gossip-protocol).
- **Cryptree is NOT a CGKA** (M5 factual correction). Benten's **K(N) structural-KDF chain IS Cryptree-aligned arbitrary-depth pattern** per Spike E `K(N) = KDF(K(predecessor), edge_label || N.cid)` — recipient with K(root) + edge_label allowlist + SubgraphSpec walks the graph + derives keys for all reachable Nodes per allowed paths. This is exactly what Ben recalled. Cryptree's "per-subtree access keys" maps to Benten's "per-edge-label-set access at each Node along walk."
- **Atrium / DeviceMesh / SingleDevice are 3 instances of one MembershipSet primitive** (per Ben's unification insight; ratified by 9-eyes M2-M6 panel + N1-N4 refinements + M-CONS-v2).
- **Admin in Atrium = User in DeviceMesh = Self in SingleDevice** on the signing-authority axis (N2's Authority-slot generalization).
- **GDPR-RTBF is P2P-by-design-semantic** for Benten (not protocol-flaw); recipients of shared content retain their copies; applications can implement crypto-shredding at the per-subject-key layer if they want strong-RTBF semantics. Compromise #55 honest-architectural-disclosure framing.

### Cost trajectory across the day

- **Start of session** (2026-05-27 LATE-EVENING handoff): ~7-15 wk to v1-beta-tag (per phase-ordering precision + ADDL-pipeline-full-observance ratifications)
- **After MembershipSet panel + N-refinements + Q1 SCOPE-IN**: **~17-22 wk to v1-beta-tag (M-CONS-v2 central ~85-111 wave-days)**
- **Trade-off accepted**: foundational correctness over speed; comprehensive coverage per Ben's preference throughout

### Active background agents at write-time

| Agent ID | Topic | Branch on completion |
|---|---|---|
| `a1853b28c3735a226` | M-C1 elegant-shape critique on M-CONS-v2 | `phase-4-meta-core/membership-set-m-c1-v2-elegant-shape` |
| `a0904cbb09c86f949` | M-C2 composability critique on M-CONS-v2 | `phase-4-meta-core/membership-set-m-c2-v2-composability` |
| `a63b0abc22d3aee2f` | M-C3 fresh-eyes cryptographer on M-CONS-v2 | `phase-4-meta-core/membership-set-m-c3-v2-fresh-eyes-cryptographer` |

ETA: ~1.5-2hr each in parallel.

### Open items remaining (at write-time)

1. **3 M-C critics return** (~1.5-2hr) → final synthesis to Ben → final-final ratification of 4 remaining Ben-calls per M-CONS-v2 §9.3 + any critic-induced refinements
2. **Tracked-doc PR cascade**: branch `phase-4-meta-core/inv-16-compromise-dak-rename-cat-a` off `2172cb6d` (CREATED + PAUSED); ready to fill once final-final design ratifies; Ben pre-authorized bypass-merge-reinstate for this PR
3. **Orchestration-branch updates**: CLAUDE.md baked-in #5 retense + #17 amendment (multi-device-mesh / DeviceMesh kind) + #18 amendment (forkability + Authority-unification + encrypt-everywhere) + dispatch-conventions amendments
4. **Memories to codify**: Cryptree-NOT-a-CGKA + K(N)-IS-Cryptree-aligned + MembershipSet typed-variant primitive + Authority-slot unification + iroh-gossip-scope-in + multi-agent-panel-pattern + RBAC-3-role-pattern + per-message-ratcheting-codepoint-reserve + GDPR-RTBF-P2P-by-design
5. **Side-quest Level 1**: open RustCrypto/KEMs CT-SampleNTT issue (~30 min orch-direct; post-cascade)
6. **F-full R0 plan-doc authoring** post-final-ratification
7. **F-full ADDL pipeline**: R1 critic council (Pattern 6) + R2 test landscape + R3 test-writers + R4 test review + R5 implementation waves (canary-first: Wave-MS-PRIMITIVE first, then Wave-MS-TRANSPORT, then X-Wing-mislabel + Layer-A/B/C/D wires) + R4b + R6 R3 + pre-tag sweep + tag `phase-4-meta-core-close`
8. **Phase-4-Meta-Composing** full ADDL pipeline + tag `phase-4-meta-close` + tag `v1-beta` + external crypto audit (~3 person-weeks) + tag `v1-GM`
9. **21 comment drafts held** (F2 multicodec + F3 JOSE [deadline 2026-05-29; acceptable to miss] + F4a LAMPS + F7 W3C CCG + Position B v2 blog + Shape 5 iroh-outreach + 16 other drafts in `new-comment-drafts.md`) — batch-post post-F-full-ratify + post-rename-applied

### Compact-survival pointer

If you resume after compaction: read CLAUDE.md banner + this addendum + the M-CONS-v2 file at `phase-4-meta-core/membership-set-m-cons-v2-consolidator @ e62ff540` FIRST. All N-refinement + cataloger + specialist outputs are on their respective origin branches enumerated above. The MembershipSet design is settled at M-CONS-v2 + Ben's ratifications; the critics are running on this consolidated design + may surface refinements but unlikely to overturn direction. After critics return: final synthesis + Ben ratify remaining 4 Ben-calls + RESUME tracked-doc PR cascade.

*Updated 2026-05-28 (today). Compact-survival snapshot for the MembershipSet panel + N-refinements + M-CONS-v2 day-long architectural-decision session.*

---

## 2026-05-29/30 ADDENDUM — MembershipSet refinement arc (cluster + member/compute agents) → M-CONS-FINAL dispatched

Session continued 2026-05-29 → 2026-05-30. The 3 M-C critics (M-C1/M-C2/M-C3) + 6 P-specialists (P1–P6) returned (M-C2 surfaced 4 contradictions + 5 composition-failures + 4 blind-spots; P1–P6 resolved them). Then **Ben surfaced a LOAD-BEARING conceptual reframe** that drove a further **7-agent refinement arc — ALL RETURNED**. Net effect: the design **CONVERGED + SIMPLIFIED dramatically** — every pass *removed* surface. M-CONS-FINAL is now dispatched to consolidate.

### Ben's reframe (load-bearing; drove the arc)
**"Everything is just a MembershipSet with different rules on how those members can interact."** Corollaries: (1) any graph-sharing-with-ongoing-access *creates* a MembershipSet (Cluster 1); (2) Atrium/Garden/Grove differ by **GOVERNANCE/admin-controls**, not nesting — with **federation + compute as ORTHOGONAL axes** (Cluster 2); (3) agent-nature should be **DERIVED not stored**; (4) **AI-agents are a kind of Plugin**; (5) **compute is a separate first-class resource** decoupled from membership. Grounded in canon: VISION + `docs/archive/exploration/explore-gardens-mvp.md` ("Gardens are Atriums with extra rules; promotion is a config change") + `explore-distributed-compute-vision.md` (compute = metered peer-resource; per-community economics).

### 7 refinement agents (ALL RETURNED; .addl docs force-committed on branches)
| Agent | Branch @ commit | Headline ruling |
|---|---|---|
| CA-1 sharing-as-membership | `…ca1-sharing-as-membership @ 2c60ce0d` | primitive set = **{MembershipSet, Drop}**; RestrictedScopeSet collapses → K(N)+edge-label-allowlist; wire-neutral; −0.4..−0.8 wd. **Caught the #31 collision.** |
| CA-2 structural/gov/federation | `…ca2-structural-governance-federation @ da0627aa` | 3 orthogonal axes; **Garden/Grove = governance PRESETS not crypto-Kinds** (drop from codepoint-Kind reserve); federation = P3 recursion re-filed (Inv-20 k+l survive); **RoleId→5** (ship 3, reserve Moderator+Invitee); promotion = config-change-no-rekey |
| CA-2X compute axis | `…ca2x-compute-axis @ 66c41016` | compute = orthogonal READ-only layer (SUPERSEDED by CM-2/CE-1) |
| CM-1 member model | `…cm1-member-model @ 8855d1c8` | member = existing **Principal**; membership = RELATION; agent-nature **DERIVED** (did:agent:+UCAN), **ZERO stored field**; no MemberKind axis (federation = edge); mint **"nature-derived-never-stored" invariant** |
| CM-2 compute-resource | `…cm2-compute-resource-model @ 063b0f68` | compute = resource Node + `OwnerRef{Member\|Community\|ThirdParty}` (CE-1 supersedes its reserve) |
| PA-1 agent-as-Plugin | `…pa1-plugin-agent-member-unification @ ba5c187d` | **AI-agent = derived predicate on Plugin = derived flavor of Principal**; M4 BREAK-5 = default not prohibition; 3 trust categories → 2; did:agent: = optional external alias; **DELETE member_type**; ZERO new frozen field |
| CE-1 compute elegance | `…ce1-compute-elegance @ 9eacd364` | compute → uniform **PeerResource**; economics **COMPOSES** (UCAN caveats + Credits-ledger-as-graph + signed `CommunityEconomicPolicy` Node); **v1-beta freeze hook = ZERO** (drop CM-2 reserve). Freeze-hook trend CA-2X 2 → CM-2 1 → CE-1 0 |

### Net result: ~ZERO new member/compute wire surface
Member side: **0 new frozen fields** (Principal + derived predicates; nature never stored). Compute side: **0 reserve** (composes from existing primitives). Maximally consistent with code-as-graph + 12-primitives-irreducible + compose-before-extend + derive-don't-store. Cumulative cost trend across the arc: net-CHEAPER than M-CONS-v2.

### Held orch-direct items DISPOSITIONED (surfaced to Ben 2026-05-30)
- **Cluster 3 (iroh-gossip):** D6 hybrid (D1 HMAC-blind + D2 fork-rotation + D5 OOB-rendezvous); D3/D8/D9/D10 rejected; D4/D7 Phase-5+.
- **§9.3 (4 calls):** mint Compromise #56; KEEP `refresh_required_secs` name + clarify; sibling-traits **ALONGSIDE** inherent-impl; CONFIRM transport = iroh-gossip-only.
- **Cluster 4 (audit):** config-surface + suggested-defaults (Atrium/DeviceMesh=AdminOnly; SingleDevice=Public-All-Members).
- **Cluster 7:** `key_retention_window_secs` **DEFAULT 7 days, user-definable**.
- **Cluster 8 (M-C3):** adopt C-5 (AAD-injective doc) + C-6 (mint Inv-21 fork-tie-break) + C-7 (codepoint-AAD-bind).

### #31 COLLISION (VERIFIED on main `2172cb6d`) — cascade must resolve
`docs/SECURITY-POSTURE.md` uses **#31 for BOTH** "Revocation reach in encryption-at-rest" (G-CORE-9; summary table line 91) **AND** "LAMPS Composite ML-DSA EUF-CMA-only" (PR #1357; prose section line 2433). CLAUDE.md baked-in #5 cites #31=LAMPS. **my-pred: LAMPS keeps #31, renumber the revocation-reach compromise** (lowest blast radius). M-CONS-FINAL owns the canonical assignment.

### M-CONS-FINAL DISPATCHED (agent `a5e9734cb2b6d8f6d`; branch `phase-4-meta-core/membership-set-m-cons-final`)
Consolidates: M-CONS-v2 baseline + 3 M-C critics + 6 P-specialists + 7 refinement agents + orch-ratified rulings + #-renumbering (#31 + 5 panel collisions: P3/P4/P5/P6/M-C3 each used #57/#58/#59) + §9.3 + Cluster 3/4 dispositions. **Produces:** final design + ONE consolidated Ben-call list + canonical Compromise-#/Inv tables + v1-beta-freeze inventory + cost estimate + R0-plan skeleton. ~2–4hr ETA. **On return: surface clean board to Ben → final ratification.**

### Next steps (post-M-CONS-FINAL)
1. M-CONS-FINAL returns → surface to Ben → final ratification (+ optional light critic round on FINAL).
2. **Tracked-doc PR cascade** (branch `inv-16-compromise-dak-rename-cat-a` off `2172cb6d`; **bypass-merge-reinstate PRE-AUTHORIZED** for this PR): Inv-16 + new Compromise mints + **#31 resolution** + Cat-A X-Wing→MLKEM768-X25519 rename + atrium test count 15→16.
3. **Orchestration-branch updates** (CLAUDE.md baked-in #5 retense + #17 DeviceMesh + #18 forkability/encrypt-everywhere/Authority-unification + dispatch-conventions) + **memory codification** (~13: Cryptree-not-CGKA, K(N)-Cryptree-aligned, MembershipSet-primitive, Authority-unification, everything-is-a-MembershipSet-governance-axis, agents-are-plugins, derive-nature-not-store, compute-composes-zero-hook, RBAC, iroh-gossip-scope-in, multi-agent-panel-pattern, GDPR-RTBF-P2P-by-design, + inline-gitignored-canon-in-briefs).
4. **F-full R0 plan-doc** → R1 critic council → R2 → R3 → R4 → R5 (canary-first) → R4b → R6 R3 → pre-tag → **tag `phase-4-meta-core-close` (HOLD: Ben check-in)**.
5. **Phase-4-Meta-Composing** ADDL → tag `phase-4-meta-close` → tag `v1-beta` → external audit → tag `v1-GM`.

### Process lesson codified this arc
**Gitignored canon is invisible to fresh-worktree agents.** `CLAUDE.md` + `docs/archive/exploration/*` + `.addl/*` are gitignored. CA-2 couldn't read `explore-gardens-mvp.md` (grounded on VISION + the Ben quote instead; conclusion held). Fix applied from CM-1/CM-2 onward: **inline the relevant gitignored-canon passages directly into agent briefs.** Prior `.addl` agent docs ARE readable via `git show <branch>:<path>` because agents force-add them.

### Standing law UNCHANGED
NEVER --admin-bypass / force-push; NORMAL --squash; HARD RULE 12; do-it-now bias (per `feedback_orchestrator_defer_prediction_bias`); full ADDL observance (per `feedback_addl_pipeline_full_observance`); surface arch forks (per `feedback_surface_arch_decisions_under_auth`); iterate-to-convergence at R1+R4+R4b+R6; agent isolation:worktree + run_in_background + commit-before-return + ABSOLUTE-PATH-FORBIDDEN-outside-${WORKTREE_ROOT}; inline-gitignored-canon-in-briefs. **DO NOT TAG without Ben check-in.**

*Updated 2026-05-30. Compact-survival snapshot for the MembershipSet refinement arc + M-CONS-FINAL dispatch.*

---

## 2026-06-01 ADDENDUM — M-CONS-FINAL RETURNED + Ben RATIFIED the full board

**M-CONS-FINAL landed** at `phase-4-meta-core/membership-set-m-cons-final @ a99dd0c7` (`.addl/phase-4-meta/membership-set-m-cons-final.md`, 823 lines + created `docs/future/compute-marketplace.md` stub). Frozen wire-surface SHRINKS vs M-CONS-v2; cost **~80–104 wave-days** (−5 to −7). **9 consolidated Ben-calls; Ben ratified ALL.**

### Ratification outcome (2026-06-01)
- **BC-1..BC-8: RATIFIED AS-IS** (all my-pred):
  - BC-1 {MembershipSet, Drop}; RestrictedScopeSet → per-member K(N) walk-scope (wire-neutral re-label).
  - BC-2 Garden/Grove = governance presets on Atrium (NOT crypto-Kinds); dropped from MEMBERSHIP_SET_KIND reserve; P3 recursion → Federation (`MemberRef::SubsetRef`).
  - BC-3 member = `Principal` (relation not type); no Agent/Member/MemberKind; **`member_type` DELETED**; nature derived (Inv-22); AI-agent = derived Plugin-predicate; trust categories → 2; `did:agent:` = optional external alias.
  - BC-4 M-C1 fusions: `members_table` + `disposition_class` + metadata/`GovernanceConfig` top-level.
  - BC-5 all M-C2 fix-nows (role-gen AAD + `E_ROLE_STALE_AT_VERIFY` + 7d retention + tie-break-all-authors + fork-inherits-metadata + ChainedStateTlv reserve) + M-C3 surfaces (F-FE-1 doc + **Inv-21** + F-FE-4 gate).
  - BC-6 compute = `PeerResource` composes; **`economic_policy` reserve DROPPED** (v1-beta freeze hook = ZERO); Phase-5+ → `compute-marketplace.md`.
  - BC-7 transport = iroh-gossip ONLY at v1-beta (Willow/iroh-roq/iroh-live codepoint-reserve); P2 D6 privacy (HMAC-blinded topic + fork-rotation + OOB rendezvous).
  - BC-8 Compromise housekeeping: **LAMPS keeps #31; revocation-reach → #62**; mint #56; resolve #57–#61 (P4≡M-C2-B-1→#57; audit-insider→#58; KEM-key-confirm→#59; RBAC-role-transition→#60; fingerprint-leak→#61); `disposition_class` added; KEEP `refresh_required_secs` name + clarify; sibling-traits ALONGSIDE inherent-impl.
- **BC-9 (genuine arch-fork): Ben chose SHIP ALL 5 `RoleId` ACTIVE** (Admin/Moderator/Member/Viewer/Invitee) at v1-beta — overrode orchestrator my-pred (ship-3-reserve-2). **Implication: define Moderator + Invitee permission-sets at v1-beta** (Moderator = subset of admin powers; Invitee = pre-acceptance limited; per Gardens-MVP). Governance *workflows* still Phase-4-Meta-Composing. Small cost add over M-CONS-FINAL's ship-3 assumption; **flag for R0**.

### Invariants/Compromises now ratified for landing
Inv-20 (12 clauses) + **Inv-21** (fork-tie-break HARD partition) + **Inv-22** (member-nature-derived-never-stored). Compromise canonical table: #31=LAMPS (kept), revocation-reach=#62, #56 (journalist FS), #57–#61 (collisions resolved), `disposition_class` column. NOTE: real in-tree state = Inv-1..15 registered on main; Inv-16..22 are design-mints landing via cascade/R0/impl.

### Immediate forward path (post-ratification)
1. **Orchestration-branch updates** (orchestrator-direct; no main-touch): CLAUDE.md baked-in #5 retense + #17 (DeviceMesh) + #18 (forkability + Authority-unification + everything-is-a-MembershipSet + agents-are-plugins + derive-nature + compute-composes) amendments + dispatch-conventions + **memory codification** (~13 incl. everything-is-a-MembershipSet, agents-are-plugins, derive-nature-not-store, compute-composes-zero-hook, inline-gitignored-canon-in-briefs).
2. **Tracked-doc PR cascade** (branch `inv-16-compromise-dak-rename-cat-a` off main `2172cb6d`; bypass-merge-reinstate PRE-AUTHORIZED). **SCOPE DECISION PENDING Ben** (surfaced 2026-06-01): land the F+ encryption-arc decisions now (Inv-16 + DAK compromise + **#31 collision fix** + Compromise #30→#31 cross-link + Cat-A X-Wing→MLKEM768-X25519 rename + atrium test count 15→16), with the MembershipSet mints (#56–#62, Inv-19–22) landing via the F-full R0→implementation pipeline — OR land both together. Numbering-coordination caveat: renumbering revocation-reach→#62 forward-references the not-yet-landed panel #45–#61.
3. **F-full R0 plan-doc authoring** — consolidates the encryption arc (9-eyes registry @ fbdfeb16 + e2r-ffull-scope-review @ 220b5aae + Option-F+ NO-GO §6.2 envelope-unification + Amendments 1–6) AND the MembershipSet primitive (M-CONS-FINAL @ a99dd0c7 §10 skeleton). Then F-full R1 critic council → R2 → R3 → R4 → R5 (canary-first) → R4b → R6 R3 → pre-tag → tag `phase-4-meta-core-close` (HOLD: Ben check-in).

*Updated 2026-06-01. Compact-survival snapshot: M-CONS-FINAL ratified; forward path = orchestration updates + tracked-doc cascade (scope pending) + F-full R0.*

---

## 2026-06-02 ADDENDUM — F-full R0 authored → R1 CONVERGED → R0.3 → R2 dispatched (now using Workflows for the ADDL pipeline)

The ADDL pipeline for F-full is now running, **orchestrated via the Claude Code Workflows feature** (Opus 4.8 research-preview). Progress this session:

### F-full R0 → R1 → R0.2 → R1.2 (CONVERGED)
- **R0 authored** (`phase-4-meta-core/f-full-r0-plan @ 6755ea41`; 1055 lines) — consolidated the encryption arc (4-layer A/B/C/D + §6.2 codepoint-dispatched envelope) + the MembershipSet primitive (M-CONS-FINAL) + GN graph-native wins + EP-1 engine-plugin symmetry. Seeded 9 R1 questions.
- **R1.1 critic council** (Workflow `wk0y82kby`; 8 Opus lenses) → **3 BLOCKER + 20 MAJOR + 15 MINOR + 9 OBS**; 7/9 Qs confirmed; triage persisted at `.addl/phase-4-meta/r1-triage.md` (orch commit `50115446`).
- **Ben ruled 3 forks (2026-06-02):** (1) **Sealed-Sender = DEFAULT** (FREEZE+SHIP-at-Core @ `0x6510`; abuse-control = recipient-issued delivery-tokens, new **Compromise #63**, ~5-8 wd into Core); (2) **Compromise #31 = LAMPS keeps it** (revocation-reach → **#62**; #30 unaudited-PQ stays); (3) **X-Wing = real SHA3-256 construction @ `0x647A`** (LOC ~24 → ~120-220; regen KAT vectors).
- **R0.2** (`phase-4-meta-core/f-full-r0-plan-r1fp @ 477529d0`; 1547 lines) — applied the 3 rulings + every FIX-NOW + re-ran the §0.3 ground-truth log. Notable: M-15 closed via **correct DISAGREE** (Inv-15 IS registered in-tree — INVARIANT-COVERAGE.md row 15 + header "15 invariants"; the R1.1 finding was stale; ground-truth-verified by orchestrator + 4 lenses).
- **Codepoint table BLESSED by Ben** (R0.2 §4.0; canonical home `docs/CRYPTO-CODEPOINTS.md` to be authored AT R2 as a freeze-prereq): Sealed-Sender `0x6510` DEFAULT · MembershipSet relocated to `0x6600` (out of the MLS `0x6380/0x6390` bracket; 9-eyes wins that collision) · X-Wing `0x647a` real construction · DeviceLink `0x6310-0x631F` · RemotePermission `0x6320-0x632F` (+ ExecuteWorkflow reserve).
- **R1.2 convergence council** (Workflow `wnuo0tugv`, fresh all-Opus, batched 2×4): **CONVERGED** — 8/8 APPROVE-FOR-R2 HIGH, **0 BLOCKER + 0 MAJOR**, all 3 BLOCKER + 20 MAJOR CLOSED, 3 trivial new MINOR. No R1.3.
- **R0.3 micro-touch** (`phase-4-meta-core/f-full-r0-plan-r1fp-r03 @ 4fe9236a`) — the 3 MINOR (tight-`exp` §3.4 sentence + nonce-cache "net-new not shipped-instance" precision §3.10/§3.4 + §7.3 DAK-DAG-split). **THIS IS THE CANONICAL POST-R1 R0.**

### R2 IN FLIGHT (Workflow `we66419zk`)
R2 test-landscape synthesis restructured as a Workflow per Ben (multi-modal sweep + completeness critic): 6 discovery dimensions (crypto-envelope / membership-sync-crdt / threat-security / wire-freeze-conformance / privacy-metadata / graph-native-invariant; batched 3+3 all-Opus) → completeness critic ("what Inv/Compromise/codepoint/NQ/exit-criterion has no test family?") → synthesis (catalog + coverage matrix + R3 slicing + freeze-gating priorities). **On return: surface → R3 test-writers.**

### Freeze-gating NQ-* carried to R2 (from R1.2)
NQ-C1 (McMillion-hpke admits PQ KEM into real RFC-9180 context — **gates Canary-ENC-2**) · NQ-C2 (libcrux↔RustCrypto FIPS-203 KAT, Wave-0 gate) · NQ-C3 (cross-ecosystem LAMPS conformance vectors) · NQ-C4 (did:key hybrid-pubkey multicodec — wire-affecting) · NQ-D1 (GossipTransport placement) · NQ-D2 (Inv-21 total-order + kani shape) · NQ-W2 (CRYPTO-CODEPOINTS.md + CI band-collision scanner).

### Workflow-usage lessons codified this session (pim-N candidates)
- **Schema-free for prose-heavy research/review agents** — forced StructuredOutput silently fails ("completed without calling StructuredOutput"); have lenses RETURN findings as their final message instead. (Cost us 4/5 agents on the first research sweep.)
- **Batch parallel fan-out into sub-waves of 3-4** — a single burst of 6-8 concurrent agents trips the transient server-side rate-limit ("Server is temporarily limiting requests · not your usage limit"). Batched 2×4 cleared it.
- **Workflow resume returns CACHED results** — `resumeFromRunId` does NOT re-run rate-limited/failed agents (it replays the cached final result); to re-run failures, dispatch a FRESH run.
- **Consolidators must refuse to certify on a partial panel** — a rate-limited silence is NOT an APPROVE (the R1.2 consolidator correctly self-policed per `feedback_agent_liveness_verify_not_notification`).
- **Ben preference (2026-06-02): Opus-only** for council/agent work (not mixed-model), rigor over the cost trim.

### Still queued (post-pipeline / unchanged)
Tracked-doc cascade (Inv-16 mint + the **#31-fix** + Cat-A X-Wing→MLKEM768-X25519 rename + atrium test 15→16; bypass-merge-reinstate pre-authorized) + orchestration-branch CLAUDE.md/dispatch-conventions updates + ~13 memory codifications + 21 held comment drafts + RustCrypto CT-SampleNTT side-quest. **HOLD on all tags pending Ben.** Next pipeline stages after R2: R3 (test-writers, canary-first) → R4 → R5 (impl waves) → R4b → R6 R3 → pre-tag → `phase-4-meta-core-close`.

*Updated 2026-06-02. Compact-survival snapshot: R1 CONVERGED; canonical R0 = 4fe9236a (R0.3); R2 test-landscape Workflow `we66419zk` in flight.*

---

## 2026-06-02 LATE ADDENDUM — R2 DONE + R3 test-writers IN FLIGHT (pre-compact prep)

### Pipeline position: F-full ADDL is at **R3 (test-writers), in flight**
Sequence so far: design exploration → M-CONS-FINAL (`a99dd0c7`) → **R0** (`f-full-r0-plan @ 6755ea41`) → **R1.1** (8-lens Workflow; 3 BLK + 20 MAJ) → **R0.2** (`f-full-r0-plan-r1fp @ 477529d0`; 3 Ben-rulings + every fix) → **R1.2 CONVERGED** (0 BLK/0 MAJ, 8/8 APPROVE) → **R0.3** (`f-full-r0-plan-r1fp-r03 @ 4fe9236a` — **CANONICAL R0**, off main `2172cb6d`) → **R2 test-landscape** (DONE) → **R3 test-writers** (IN FLIGHT).

### R2 DONE — `.addl/phase-4-meta/f-full-r2-test-landscape.md` (committed on orch branch)
Multi-modal Workflow (6 discovery dims batched 3+3 + completeness critic + synthesis). Output: **~95 unified test families** (deduped from ~163 raw + **22 completeness-critic gap-fills**), **~720-950 red-phase tests**, **~74 of ~95 FREEZE-GATING**. 12 family-groups; full coverage matrix (every Inv-16..22 / Compromise #30-#63 / frozen-codepoint / NQ / §9 exit-criterion → its family; zero uncovered). **8-wave canary-first R3 slicing** (W0 sole-upstream canary; Tier-1 W1/2/3; Tier-2 W4/5/6 = new `benten-membership-set` crate; Tier-3 W7 doc-wave cap-exempt). Freeze-gating P0: NQ-C1 (HPKE-KEM-extensibility, gates Canary-ENC-2), codepoint pins+scanner, zero-`to_le_bytes` BE sweep, real X-Wing KATs.

### F-LC-9 RULED by Ben (2026-06-02): GROUP sends honor Sealed-Sender
The R2 completeness-critic surfaced a real design hole R1 missed: group multi-stanza sends (`0x6520`/`0x6610`) didn't state whether they honor Sealed-Sender. **Ben ruled: group sends HONOR Sealed-Sender** (a per-stanza inner-sender-DID binding inside the group AAD — sender-DID NOT plaintext on group sends). Pre-freeze wire change; R3-W2 tests it (F-LC-9); **small R0 addition still owed** (record the group-AAD inner-sender binding in the R0 group-codepoint section — fold into R3.2 or the doc-wave).

### R3 IN FLIGHT — Workflow `w1miqerfq` (single sequential canary-gated workflow)
Structure: **Canary** (W0 crypto-suite test-writer incl. NQ-C1 investigation → W0 mini-review **GATE** emitting `GATE: PASS`/`FIX-NEEDED`; fan-out fires ONLY on PASS) → **Fanout-A** (W1 crypto-KAT / W2 Layer-C incl. F-LC-9 / W3 Layer-D) → **Fanout-B** (W4 MS-structure / W5 AAD+Inv-21+kani-floor / W6 gov+audit) → **DocWave** (W7 cap-exempt) → **Reviews** (per-wave substantive-pin + seam-disjointness mini-reviews) → **Coverage** (full ~95-family + freeze-gating coverage + gap-list + convergence call). Test-writers base off the orch branch (crate code == main except 1 stale test file), read R0.3 via `git show 4fe9236a:...`, write SELF-CONTAINED red-phase stubs (pim-12; V2/BE/EncryptedEnvelope from first commit per M-20) to branches `r3/w{0..7}-*`.
- **ON RETURN:** if `GATE: FIX-NEEDED` → workflow halts before fan-out, returns W0+findings → orchestrator-led canary fix-pass + re-run. If clean → harvest the 8 wave branches (drop worktrees), review the coverage report, **strategy-C consolidate** the red-phase corpus (rebase onto the canary's landing SHA per R2 tree-divergence note), surface to Ben → **R4 (deeper test-review tier)**.

### Workflow-usage lessons this session (codified as memory `feedback_workflow_tool_addl_pipeline_usage`)
Schema-free for prose agents (forced StructuredOutput silently fails) · batch parallel into sub-waves of 3-4 (a 6-8 burst trips the transient server-side rate-limit; "Server is temporarily limiting requests · not your usage limit") · **resume returns CACHED, does NOT re-run failed agents** (fresh invocation to re-run) · consolidators must refuse to certify on a partial panel (rate-limit silence ≠ APPROVE) · inline canary-gate (mini-review returns `GATE: PASS/FIX`, conditional fan-out) · persist workflow results via `python3 -c "json.load(...)['result']" > artifact.md` · **Opus-only** (Ben pref 2026-06-02) for council/agent work.

### Saved workflow scripts + R3 SURVIVAL (read if R3 hasn't returned yet)
Scripts at `.../workflows/scripts/`: `f-full-r1-2b-convergence-*.js`, `f-full-r2-test-landscape-*.js`, `f-full-r3-test-writers-*.js`. **Live R3 = Task `w1miqerfq` / Run `wf_ab799dee-898`** (script `f-full-r3-test-writers-wf_ab799dee-898.js`).
- **Survival mechanics:** a context COMPACTION (same session) keeps R3 RUNNING — it auto-fires its completion `<task-notification>` to the post-compact orchestrator (no ID needed to receive it; poll early via `TaskOutput({task_id:"w1miqerfq", block:false})` → `local_workflow`/`running`/`completed`). A **NEW session** (after a Claude Code exit) LOSES the in-flight workflow — **RE-LAUNCH from the saved script** (`Workflow({scriptPath: ".../f-full-r3-test-writers-wf_ab799dee-898.js"})`; resume only works same-session). See memory `feedback_workflow_tool_addl_pipeline_usage`.

### Canonical artifacts/branches
Main `2172cb6d` · orch branch HEAD (handoff/triage/landscape docs) `bc592e75`+ · **canonical R0 = R0.3 `4fe9236a`** · R1 triage `.addl/phase-4-meta/r1-triage.md` (orch `50115446`) · R2 landscape `.addl/phase-4-meta/f-full-r2-test-landscape.md`. **Codepoint table BLESSED** (R0.2 §4.0; `CRYPTO-CODEPOINTS.md` authored AT R2 as freeze-prereq): Sealed-Sender `0x6510` DEFAULT, MembershipSet `0x6600`, X-Wing real `0x647a`, MLS keeps `0x6380/0x6390`.

### Next pipeline stages
R3 lands → consolidate → **R4** (test review, iterate-to-convergence) → **R5** (canary-first impl waves, ≤7-cap) → **R4b** → **R6 R3** (phase-close council) → pre-tag sweep → **tag `phase-4-meta-core-close`** (HOLD: Ben). Then Phase-4-Meta-Composing ADDL → `phase-4-meta-close` → `v1-beta` → external audit → `v1-GM`.

### Queued post-pipeline (UNCHANGED; not compact-critical, in-flight design not yet frozen)
Tracked-doc cascade (Inv-16 mint + **the #31-fix [LAMPS keeps #31, revocation→#62]** + Cat-A X-Wing→MLKEM768-X25519 rename + atrium test 15→16; bypass-merge-reinstate pre-authorized) · CLAUDE.md baked-in #5 retense + #17/#18 amendments (the everything-is-a-MembershipSet + agents-are-plugins + derive-nature + compute-composes-zero + Rust-engine-plugin reframes — all RATIFIED, captured in M-CONS-FINAL + R0, to codify in CLAUDE.md at the cascade) · ~13 memory codifications · 21 held comment drafts · RustCrypto CT-SampleNTT side-quest. **HOLD all tags pending Ben.**

### Standing law (UNCHANGED)
NEVER --admin-bypass / force-push; NORMAL --squash; HARD RULE 12; do-it-now bias; full ADDL observance; surface arch forks; iterate-to-convergence R1+R4+R4b+R6; agent isolation:worktree + commit-before-return + ABSOLUTE-PATH-FORBIDDEN; ≤7 implementer cap + scoped-pre-flight; **disk HARD-ABORT new-dispatch at >97%** (raised from ~92% by Ben 2026-06-02; cargo-clean-idle valve at ~90%); Opus-only (council/agent); inline-gitignored-canon-in-briefs; **DO NOT TAG without Ben check-in**.

*Updated 2026-06-02 LATE (pre-compact). Compact-survival: F-full at R3 test-writers (Workflow `w1miqerfq`) in flight; canonical R0 = R0.3 `4fe9236a`; on R3 return → consolidate → R4.*

---

## 2026-06-02 EVENING ADDENDUM — R3 RETURNED + CONVERGED + CONSOLIDATED (steps 1-3 done; R4 next)

**R3 Workflow `w1miqerfq` COMPLETED CLEAN** (17 agents, ~73 min): canary `GATE: PASS` → W1-W7 fan-out → **7/7 mini-reviews APPROVE** → coverage consolidator **CONVERGED** (89/89 families, GAP-LIST empty, **no R3.2 patch needed**). Independently ground-truth-verified: 8 branch tips match, **zero double-implementation** (git set-intersection: 56 distinct test files, none on >1 branch), all 7 verdicts genuine APPROVE.

**Steps 1-3 (harvest + consolidate) DONE:**
- **Step 1** — main un-parked from the **W6 isolation-escape** (3rd codified instance: W6 committed `r3/w6-gov-audit` onto the MAIN repo working tree). Switched main back to orch branch; **disk HARD-ABORT 92%→97%** committed `dd56aee7`; w6 rebuilt clean (`7945b79c`, 1 test-only commit off main 2172cb6d).
- **Step 2 (harvest)** — all **8 clean `r3/w*` branches pushed to origin**; **8 workflow worktrees dropped** (disk 90%→86%).
- **Step 3 (strategy-C consolidate)** — **`phase-4-meta-core/f-full-r3-consolidated @ 50561799`** (off main `2172cb6d`; 60 files = 56 test + new `benten-membership-set` 15th crate + root manifest). **COMPILE-VERIFIED green behind `#[ignore]`** per crate (--no-run; crypto-suite/sync/drop `--features testing`; engine `test-helpers`+`benten-eval/testing`; membership-set `--features testing`). **Real defect caught by compile-verify**: W4+W5's `benten-membership-set` manifests genuinely DIFFERED (consolidator's "identical scaffold" was wrong) — took W4's superset `lib.rs` (`pub mod scaffold`/F-CRATE-2) + merged W5's `[dev-dependencies]` (blake3/serde/serde_bytes/serde_ipld_dagcbor/proptest). Pushed.

**5 R5-FILL carry-items** (named-now for R5 brief; NOT R3 defects — R4 re-examines): (1) W5 F-AAD-1 hex-pin self-referential → hard-code real hex; (2) W5 F-CRDT-3/F-MST-3/F-GOSSIP-1/F-INV21-4 pass-through stubs → real merge path; (3) W4 crate2_b1_dep_set asserts a Cargo.toml comment (count 38 not 39); (4) W0 canary F-INV16-1 U3(+U1/classical_0x6400/aead_lifts) green-against-stub when intended-RED → fix collision-pair; (5) **M-20**: rebase byte-pinning families onto canary landing SHA at R5.

**NEXT = R4 (deeper test review, iterate-to-convergence per rule 9).** Reviews the consolidated corpus (read-only, cap-exempt) before R5 impl. Design = a Workflow round mirroring proven R1 shape (lens council batched 4+4 → adversarial-verify MAJOR/BLOCKER → completeness-critic → consolidator converge-call); iterate across invocations (R4.1→fix→R4.2) until 0 BLOCKER/MAJOR. **Awaiting Ben go to launch R4.** Then R5 (canary-first impl) → R4b → R6 R3 → pre-tag → tag (HOLD: Ben).

*Updated 2026-06-02 EVENING. Compact-survival: R3 CONVERGED+consolidated @ `phase-4-meta-core/f-full-r3-consolidated 50561799` (off main `2172cb6d`); 8 per-wave branches on origin; R4 design surfaced, awaiting Ben go.*

### R4 LAUNCHED (2026-06-02 EVENING) — Workflow `wg7e816oq` IN FLIGHT
Ben ratified the R4 council (15 lenses + adversarial-verify). **Live R4 = Task `wg7e816oq` / Run `wf_ec3fff59-ceb`**; script `/tmp/f-full-r4-review.js` (also auto-saved under `…/subagents/workflows/wf_ec3fff59-ceb/` + `…/workflows/scripts/`; reconstructable from this session's transcript).
- **Structure:** Review (15 lenses, batched 4×, schema-free prose) → Structure (1 consolidator → structured findings, schema) → Verify (1 skeptic per BLOCKER/MAJOR, refute-default, schema) → Completeness-critic (1, fresh gap-hunt) → Converge (1 prose; **refuses CONVERGED on <15-lens partial panel**).
- **15 lenses:** L1 substantive-pin&falsifiability · L2 R5-readiness · L3 determinism/flake/isolation/CI-realism · L4 wire-freeze-byte-correctness · L5 codepoint-registry · L6 crypto-construction · L7 encoding/DAG-CBOR/BE · L8 invariants Inv16-22 · L9 capability/UCAN/RBAC · L10 threat-model/bounded-decode · L11 distributed/sync · L12 MembershipSet-shape-fidelity · L13 coverage/gap-audit · L14 cross-wave-seam/M-20 · L15 ruling-fidelity/cross-lang.
- Agents READ-ONLY (cap-exempt, `git show` only, NO worktrees — escape-proof). Corpus = `phase-4-meta-core/f-full-r3-consolidated` branch; R0.3 = `f-full-r0-plan.md @ 4fe9236a`; R2/R1 on orch branch.
- **ON RETURN:** if CONVERGED (full panel, 0 confirmed BLOCKER/MAJOR) → apply any R4-fix-list to the consolidated corpus + carry R5-fill list → **R5 (canary-first impl)**. If NOT-CONVERGED → orchestrator-led corpus fix-pass → re-run R4.2 (iterate-to-convergence, rule 9). If PANEL-INCOMPLETE → fresh re-run of missing lenses. **Survives a compaction (auto-notifies); a session-exit loses it → re-launch from the saved script.**

*Updated 2026-06-02 EVENING (R4 launched). Compact-survival: R4 Workflow `wg7e816oq` in flight over corpus `50561799`; on return → converge call → R5 or R4.2.*

### R4 RETURNED — NOT-CONVERGED (2026-06-02 NIGHT); 4 Ben-rulings adjudicated; NQ-cluster spec agent in flight
R4 (Workflow `wg7e816oq`, 50 agents, full **15/15** panel, adversarial-verified 22 CONFIRMED/9 PARTIAL/1 REFUTED) → **NOT-CONVERGED**: **8 BLOCKER** (7 panel + 1 completeness-critic = Drop-no-K_Set has NO test) + **~16 MAJOR** + 14 MINOR + 10 OBS. Triage persisted `.addl/phase-4-meta/r4-triage.md` (commit `518cea23`). Corpus is **bimodal** (W0/W1/W2 gold-standard; freeze-gating byte-pins + membership W4/W5 thin); defects are **localized single-arm rewrites, NOT re-architecture**. R4 caught what R3's "89/89" structurally couldn't: self-referential `members_table` hex-pin (tautology), U3 collision that can never fire, **F-LC-2 vs F-LC-9 contradiction**, **pre-existing-on-main LE-vs-BE landmine** (canonical_bytes_v1), Drop-no-K_Set gap. Orchestrator ground-truth-verified all 4 ruling-findings (real).
**4 Ben-rulings (2026-06-02 NIGHT):** (1) **0x6520 Layer-C group send HONORS Sealed-Sender** (per-stanza inner-sender-DID inside sealed part; update R0.3 §3.3 + test; plaintext-sender = non-default variant); (2) **Admin = Moderator set ∪ full governance** {admit/kick/rotate-keys/assign-roles/edit-governance} (satisfies M-11); (3) **0x6101 = ASSIGN distinct codepoint** (the 12-byte `SymmetricAead` ChaCha20-Poly1305 is a FROZEN shipping v1-beta variant per R0.3 §4.1 "ship both" — NOT droppable; formalize in §4.0); (4) **NQ-T2 + cluster = SPEC NOW** → read-only agent `a46b2274e63680bfe` (background) speccing NQ-T2/T3/T4/C5 (freeze-gating? tractable? ratifiable rule vs keep-open-shape-only) → returns proposals for Ben ratify.
**PLAN:** NQ agent returns → Ben ratifies → ONE consolidated **R4-fix pass** (all R4-FIX items in r4-triage.md §3.A: hex golden-vectors for F-AAD-1/F-VA-1/F-GOSSIP-2, U3 collision reconstruct, F-LC-2→Sealed-Sender, LE→BE landmine delete, MemberEntry reconcile to R0.3 §3.5, +2 R3.2 mints F-DROP-NO-KSET + no-encryption-arm, ~12 MINOR cleanups, R0.3 edits §3.3/§4.0-0x6101/§4.1-M19-sitelist-widen/§3.6.B-Admin-set) — this is the **FIRST orchestrator-led fix-pass = the converging-fix-loop SHAKEDOWN** (learn the fix-list shape + harden the loop's integration stage before any autonomous run) → **R4.2 re-review** (iterate-to-convergence, rule 9) → 0 BLK/0 MAJ → harvest → **R5** carrying the R5-FILL list (r4-triage §3.B). Fix-pass shape pending Ben go.

*Updated 2026-06-02 NIGHT. Compact-survival: R4 NOT-CONVERGED (8 BLK/~16 MAJ; triage `518cea23`); 4 rulings adjudicated; NQ-cluster agent `a46b2274e63680bfe` in flight; on its return + Ben ratify → consolidated R4-fix pass (fix-loop shakedown) → R4.2.*

### NQ ratified + R4-FIX PASS DISPATCHED (2026-06-02 NIGHT) — 6 agents emit-to-/tmp; integration pending
NQ-cluster agent RETURNED: **all 4 NQ SPEC-NOW** (corpus did NOT premature-pin — the `_nq_*_gated` arms correctly park R0 defaults; Ben RATIFIED). Record: `.addl/phase-4-meta/nq-cluster-ratification.md` (`c6ea8c50`). NQ-T4 → mint a best-effort-eventual-window Compromise at the doc-cascade.
**Fix-pass = converging-fix-loop SHAKEDOWN (manual).** Pattern validated here: fix-agents are **READ-ONLY-to-repo reasoners that EMIT new file content to `/tmp/r4fix/<cluster>/<repo-path>`** (sidesteps worktree-escape AND notification-truncation); orchestrator = single-writer integrator. Shared brief `/tmp/r4fix/COMMON-BRIEF.md` (rulings + substantive-pin bar + GOLDEN-HEX procedure). **6 agents in flight:** `ac4736f758e91603f` (A-crypto: F4-004 LE→BE, F4-002 U3-collision, F4-009/017/038/040 + no-encryption-arm mint), `af56b84fc16c5e2d0` (B1-ms-aad: **F4-001 flagship golden-hex**, F4-006 MemberEntry→§3.5, F4-012/026), `abd637add6b2ea91e` (B2-ms-crdt: F4-020 Admin-set, F4-010/011/024/025/014 + cleanups), `a4813fb1cc43c8fc4` (C-drop: F4-003 0x6520-Sealed-Sender, F4-028/029 + **F-DROP-NO-KSET mint** = CC-BLK), `aaea08ec2908e230d` (D-engine: F4-015/016 LD-2, F4-042; NQ arms LEFT as-is), `a589a6aa0c0e892f2` (R0.4 plan: emits §3.3/§4.0-0x6101/§4.1-M19/§3.6.B-Admin + NQ-ratify edits → `/tmp/r4fix/R04/edits.md`).
**ON RETURN (integration — orchestrator single-writer):** (1) review each agent's manifest + emitted /tmp files; (2) apply R0.4 edits to plan branch (review first — it's the spec); (3) copy corpus /tmp files onto a fix branch off `f-full-r3-consolidated`, commit; (4) **compile-gate** per crate (`cargo test --no-run` w/ correct features — crypto-suite/sync/drop `--features testing`, engine `test-helpers`+`benten-eval/testing`, membership-set `--features testing`); (5) drop any worktrees; (6) **R4.2 re-review** (re-run affected R4 lenses on edited corpus, iterate-to-convergence rule 9) → 0 BLK/0 MAJ → harvest → R5. Capture loop-shakedown lessons (emit-to-/tmp + golden-hex + single-writer-integrate) for the converging-fix-loop hardening.

*Updated 2026-06-02 NIGHT (R4-fix dispatched). Compact-survival: 6 emit-to-/tmp fix-agents in flight (5 corpus + R0.4); integrate → compile-gate → R4.2. Loop-shakedown pattern: read-only reasoners emit to /tmp, orchestrator single-writer integrates.*

### R4-FIX INTEGRATED (2026-06-02 NIGHT) — all 6 agents back clean; fix branch + R0.4 landed; compile-gate running
**All 5 corpus fix-agents + R0.4 returned CLEAN.** Single-writer integration done: **collision-check PASSED** (29 emitted files, zero repo-path overlap across clusters — disjoint by crate/file), copied onto **`phase-4-meta-core/f-full-r4-fix`** (off corpus `50561799`; **25 files changed, +3027/-483**, incl. NEW `f_drop_no_k_set.rs` 378L). **R0.4 = `6ecf76d2`** on branch `phase-4-meta-core/f-full-r0-plan-r04` (8 ratified decisions; R0.3 anchor `4fe9236a` preserved).
**BLOCKERs closed:** F-AAD-1 self-referential→frozen 272B canonical-CBOR golden literal · U3-collision rebuilt cross-variant byte-identical (fires RED) · LE→BE M-19 landmine migrated · 0x6520 group-send seals inner-sender-DID (ruling 1) + 0x6610 too (R0.4 1b) · **F-DROP-NO-KSET minted** (CC-BLK: byte-scan K_Set absent + leaky-seal negative control) · MemberEntry reconciled to canonical R0.3 §3.5 5-field across F-AAD-1/MS-3/NAT-1 · Admin=Mod+governance (ruling 2) · 0x6101 kept (ruling 3). MAJORs: golden-hex freezes (vault/gossip-topic/remote-permission/token-binding/Inv-18), no-encryption + codepoint-injection arm mints, AAD=canonical-TLV-not-CBOR (B1+D-engine independently converged), dep-set parse, transitivity arms. NQ arms left ratified-as-is.
**SHAKEDOWN VALIDATED:** emit-to-/tmp + single-writer integrate = zero collisions, zero truncation, golden-hex-via-throwaway-compute worked across 4 agents. This is the converging-fix-loop integration pattern, proven.
**Compile-gate IN FLIGHT** (task `b8yvr5i32`; `cargo test --no-run` per changed crate — crypto-suite/membership-set/drop `--features testing`, engine `test-helpers`+`benten-eval/testing`). **ON GREEN → R4.2 re-review** (re-run affected R4 lenses on the f-full-r4-fix corpus; iterate-to-convergence rule 9) → 0 BLK/0 MAJ → harvest (merge f-full-r4-fix → corpus + R0.4→R0-canon) → R5. ON RED → compile-fix mini-pass. **2 minor R5-fill flags noted:** AAD TLV-vs-CBOR (both agents read §4.1 as TLV — confirm at R4.2); golden literals are M-20 (R5 confirm-or-update vs real encoder). Integration worktree `benten-wt-fixint`.

*Updated 2026-06-02 NIGHT (R4-fix integrated). Compact-survival: fix branch `phase-4-meta-core/f-full-r4-fix` (off `50561799`) + R0.4 `6ecf76d2`; compile-gate `b8yvr5i32` running → R4.2.*

### COMPILE-GATE GREEN + R4.2 LAUNCHED (2026-06-02 NIGHT) — Workflow `wxb4rr8bk` in flight
Compile-gate caught ONE real issue: B2's f_gossip used `gen` (reserved keyword in Rust 2024) → fixed `gen`→`generation` (`3f4abbd7`). **All 4 changed crates now compile GREEN behind `#[ignore]`** (24 executables; crypto-suite/membership-set/drop `--features testing`, engine `test-helpers`+`benten-eval/testing`). Fix branch HEAD = **`3f4abbd7`** (`phase-4-meta-core/f-full-r4-fix`, pushed to origin). R0.4 = `6ecf76d2` (`phase-4-meta-core/f-full-r0-plan-r04`, pushed). Disk reclaimed to 88% (cargo-clean main target). **Gate lesson:** the `cargo|tail; rc=${PIPESTATUS}` after-`if` resets PIPESTATUS → use a real gate.sh (bash, not zsh — zsh doesn't word-split unquoted `$feats`); trust error-line presence not masked EXIT.
**R4.2 = convergence re-review, Workflow `wxb4rr8bk` / Run `wf_3dff8f21-234`** (script `.addl/phase-4-meta/workflows/f-full-r4-2-convergence.js` + `/tmp/f-full-r4-review.js`). Same 15-lens + verify + critic + converge, but CORPUS=`f-full-r4-fix`, spec=R0.4, framing = "VERIFY R4.1's 8 BLK + ~26 MAJ are CLOSED + HUNT regressions; convergence = 0 new BLK/MAJ." Known-correct (don't re-flag): AAD=canonical-TLV, golden-hex=M-20-stub-frozen, NQ-arms-ratified.
**ON R4.2 RETURN:** if CONVERGED (0 new BLK/MAJ) → **harvest**: merge `f-full-r4-fix` → corpus (or treat f-full-r4-fix AS the corpus) + promote R0.4 to canonical R0 + drop `benten-wt-fixint` worktree → **R5 (canary-first impl waves)** carrying the R5-FILL list (r4-triage §3.B + the 2 M-20/TLV flags). If NEW BLK/MAJ → another orchestrator-led fix round (same emit-to-/tmp + single-writer pattern; the converging-fix-loop machinery, now proven) → R4.3. **HOLD all tags pending Ben.**

*Updated 2026-06-02 NIGHT (R4.2 launched). Compact-survival: compile-gate GREEN (fix `3f4abbd7`); R4.2 Workflow `wxb4rr8bk` over corpus `f-full-r4-fix` vs spec R0.4; on CONVERGED → harvest → R5.*

### PROCESS NOTE (Ben 2026-06-03): let R4.2 finish standalone, THEN adjust process going forward
Ben caught a real gap: we built `converging-fix-loop.js` (the autonomous iterate-to-convergence loop) but ran R4-fix MANUALLY (agreed shakedown) + R4.2 as a STANDALONE review-only workflow — never actually USING the loop. The loop draft is still a PLACEHOLDER (its `integrate` stage was never hardened with the shakedown's emit-to-/tmp + single-writer + compile-gate lessons). **Ben's directive: let R4.2 (`wxb4rr8bk`) finish as-is, THEN adjust process.** DO NOT preemptively rebuild the loop. **ON R4.2 RETURN:** (1) ground-truth + surface the convergence verdict; (2) **propose the process adjustment** for Ben's ratification = harden `converging-fix-loop.js` (fold in the validated shakedown mechanics; keep the in-workflow git-integration WATCHED on its first real autonomous run per our own codification) + route the genuinely-iterative stages through it — **R5 (impl-to-green loop = its killer use case)**, R6, and any R4.3. The in-workflow autonomous integration (agent does its own git writes + cargo) is the fragile/risky part the shakedown deliberately kept orchestrator-side — harden + watch the first run.

*Updated 2026-06-03 (Ben process note). Compact-survival: R4.2 `wxb4rr8bk` running standalone; on return → surface verdict + PROPOSE process adjustment (harden+adopt converging-fix-loop for R5/R6); do NOT preemptively rebuild the loop.*

### FINALIZED PIPELINE WORKFLOW LIBRARY (Ben "set up all the finalized workflows" 2026-06-03; commit `48b49c39`)
The ADDL pipeline is now **executable methodology** — a finalized, hardened, args-parameterized workflow per stage at `.addl/phase-4-meta/workflows/` (+ generic 5 wired into `.claude/workflows/` named registry; all `node --check` clean):
- **`addl-review-council.js`** — generic R1/R4/R4b/R6 council (N lenses→structure→adversarial-verify→completeness-critic→converge; refuse-on-partial). One council, four tiers (lens-set via args).
- **`addl-r2-test-landscape.js`** — multi-modal discovery sweep + completeness-critic + synthesis (catalog+matrix+R3-slicing).
- **`addl-r3-test-writers.js`** — canary-first + GATE + fan-out + substantive/seam mini-reviews + coverage-verify.
- **`addl-r5-impl-to-green.js`** — canary-first impl waves, each impl→un-ignore→test→fix until GREEN; strategy-C integrate + full-suite (isolation:worktree + escape-contract; ≤7 cap).
- **`converging-fix-loop.js`** — autonomous review→fix→re-review→loop (HARD-RULE-12 fix-all-non-disagreed; decision-log audit-not-gate); **`args.mode`: `'checkpoint'`(SAFE default — emits fixes to /tmp + returns fix-list for orchestrator integrate; the validated shakedown shape) vs `'autonomous'`(in-workflow integrator — WATCH first runs).**
- **`workflow-common.js`** (shared preamble + codification map) + **`README.md`** (stage→workflow map + the two iteration models + hard-won disciplines + reuse mechanics). Concrete `f-full-*.js` kept as worked examples.
**Invoke:** `Workflow({name, args:{cfg, lenses|dimensions|waves}})`; inline project canon into `args.cfg.canon`. **Going forward:** R5 via `addl-r5-impl-to-green`; fix rounds (R4.3/R6) via `converging-fix-loop` (checkpoint mode first, autonomous once trusted); reviews via `addl-review-council`. Memory `feedback_workflows_as_executable_methodology` updated to point here.

*Updated 2026-06-03 (finalized workflow library `48b49c39`). Compact-survival: R4.2 `wxb4rr8bk` still running; pipeline workflows finalized at `.addl/phase-4-meta/workflows/` (README = entry point); on R4.2 return → harvest → R5 via `addl-r5-impl-to-green`.*

### R4.2 NOT-CONVERGED → R0.5 spec corrections → R4.3 via the converging-fix-loop (FIRST real loop run) (2026-06-03)
**R4.2 (`wxb4rr8bk`, 15/15 panel) = NOT-CONVERGED:** R4.1's 8 BLK all verified CLOSED; **2 NEW freeze-gating BLOCKERs** + 7 MAJOR + 11 MIN + 24 OBS (triage `.addl/phase-4-meta/r4-2-triage.md`, commit `48bb4699`). The loop working — R3's 89/89 + R4.1 both missed these. **(1) F4-002 X-Wing label POSITION wrong everywhere** — corpus f_w0:67 + R0.4 plan + CLAUDE.md #5 all PREPEND; **orchestrator WEB-VERIFIED draft-connolly-cfrg-xwing-kem-10 §6 = APPEND**: `SHA3-256(ss_M||ss_X||ct_X||pk_X||XWingLabel)`, XWingLabel=`0x5c2e2f2f5e5c`. **(2) F4-001 Sealed-Sender not propagated to f_aad_2 0x6600** (the gap flagged at R0.4 1b; safety-net caught it). MAJORs: AAD-version byte disagreement (Layer-C uses ENVELOPE_FORMAT_VERSION vs membership AAD_VERSION); DeviceAuthBackend tier (open=6 sealed-#7 per §2.7); XWingLabel-bytes pin; bounded-decode gap; RoleId/MemberRef serde asymmetry.
**Ben ratified (2026-06-03):** correct X-Wing everywhere incl. #5 · RoleId+MemberRef int-discriminant · R4.3 via the converging-fix-loop · "do we fix before R4.3?" → NO: corpus fixes = the loop; **only SPEC corrections land first (mine)**.
**SPEC corrections DONE:** **R0.5 = `phase-4-meta-core/f-full-r0-plan-r05 @ e4fbfe73`** (pushed) — X-Wing APPENDED at §2.3/C-6/§3.2(a) + RoleId/MemberRef int-discriminant ruling at §3.5. **CLAUDE.md baked-in #5** carries the authoritative appended-label construction (supersedes forensic prepend-formula). README invoke-mechanic corrected (scriptPath, not `name` — `Workflow({name})` registry not wired here).
**R4.3 LAUNCHED = converging-fix-loop FIRST real run, Task `w53le30lx` / Run `wf_abc00da7-c2d`** (scriptPath `.addl/phase-4-meta/workflows/f-full-r4-3-fixloop.js` — config INLINED; **checkpoint mode**, corpus=`f-full-r4-fix @ 3f4abbd7`, spec=R0.5, 15 lenses + R4.2-correction canon). **LESSON (first try `wdwez1q9b` FAILED):** passing a large `args` object to the generic `converging-fix-loop.js` did NOT bind — the script saw `args.cfg`/`args.lenses` as `undefined` → trivially "converged" on an empty corpus. **FIX: inline CFG+LENSES into a concrete per-run script (like every f-full-* script), do NOT rely on `args` for large configs.** (Also: `Workflow({name})` registry not wired here — use `scriptPath`.) Flow: review→structure→verify→synthesis→per-file fix-pipeline (brainstorm-as-Ben→review-reasoning→emit-to-/tmp→adversarial-review-fix)→**CHECKPOINT RETURN** the fix-list + `emittedDir`. **ON RETURN (watched first-run):** review the decision-log + emitted /tmp fixes → single-writer integrate onto `f-full-r4-fix` → compile-gate (bash, per-crate features) → **R4.4 re-review** (addl-review-council, the new lens-architect) until 0 new BLK/MAJ → harvest → R5. HOLD all tags pending Ben.

*Updated 2026-06-03 NIGHT. Compact-survival: R4.3 = converging-fix-loop `wdwez1q9b` (checkpoint, corpus `f-full-r4-fix` vs spec R0.5 `e4fbfe73`); X-Wing append-label corrected everywhere (verified draft-connolly-10). On loop return → integrate fix-list + compile-gate → R4.4 re-review.*

### R4.3 converging-fix-loop RETURNED + INTEGRATED + COMPILE-GREEN (2026-06-03 NIGHT) — 4 FLAG-FOR-BEN pending; R4.4 held
**R4.3 loop `w53le30lx` (59 agents) round-1 checkpoint = 13 reviewed file-fixes** (emitted /tmp, adversarial-review-fix APPROVE). Loop found **3 BLK + 9 MAJ + 8 MIN** (deeper than R4.2 against corrected R0.5): escalated **F4-007→BLOCKER** (MemberRef text→int), found 4 NEW MAJ (Inv-20 clause-h no-pin, corpus-wide bounded-decode gap, Admin-ability-count, F-DISC range). **INTEGRATED `59cffd07`** on `f-full-r4-fix` (+1866/-383, 14 files) — single-writer; **COMPILE-GATE GREEN** (all 5 crates, 20 execs). Verified the 2 critical BLK fixes: X-Wing `SHA3-256(ss_M‖ss_X‖ct_X‖pk_X‖XWingLabel)` + `XWING_LABEL:[u8;6]=[0x5c,0x2e,0x2f,0x2f,0x5e,0x5c]` pin (F4-002/003); f_aad_2 binds sealed-inner-sender post-decrypt (F4-001). Decision-log committed in the loop output (audit-not-gate).
**4 FLAG-FOR-BEN (loop applied best-effort + flagged; PENDING Ben):** (1) **F4-006** coarse_epoch on Drop wire? — loop=NO per §4.1/M-14 (my-rec confirm); (2) **F4-004-005-LD2** Layer-D aad_version shared-vs-local — loop=one-shared 0x01 (confirm); (3) **C-MAJOR-1-64** mint **Compromise #64** (best-effort-eventual cross-device nonce window, NQ-T4) + F-DISC range 30..=64 — my-rec mint in tracked-doc-cascade, corpus range-fix lands now; (4) **F4-009-BD** corpus-wide bounded-decode gap — my-rec flagship-now + named R5-carry-row.
**R4.4 (convergence re-review) HELD pending Ben's 4 calls** (3 touch freeze bytes / a new Compromise#; re-reviewing first would just re-surface them). On confirm → R4.4 via `addl-review-council` (lens-architect) until 0 new BLK/MAJ → harvest → R5. **First-real-loop-run VALIDATED the full converging-fix-loop end-to-end** (review→fix-pipeline→emit-/tmp→adversarial-review→checkpoint-return→orchestrator-integrate→compile-green + FLAG-FOR-BEN). HOLD all tags pending Ben.

*Updated 2026-06-03 NIGHT. Compact-survival: R4.3 loop fixes INTEGRATED `59cffd07` + compile-GREEN; 4 FLAG-FOR-BEN pending Ben (coarse_epoch / LD2-aad_version / Compromise-#64 / bounded-decode); R4.4 held pending those.*

### 4 FLAGS CONFIRMED + R4.4 AUTONOMOUS LOOP LAUNCHED (2026-06-03 NIGHT) — proceed-to-convergence
**Ben confirmed all 4 flags** (2026-06-03): coarse_epoch NOT on Drop wire · Layer-D aad_version = one-shared 0x01 · mint Compromise #64 · bounded-decode flagship+R5-carry. The corpus already has the loop's best-effort fixes for these (match the rulings) → no corpus rework. **2 doc-cascade items QUEUED** (tracked-doc, separate from the corpus loop): mint **Compromise #64** in SECURITY-POSTURE (cross-device best-effort-eventual nonce-rejection window, NQ-T4) + a named **bounded-decode R5-carry-row** (the full wire-decoder-reject family).
**Ben asked "why didn't it proceed to convergence?"** — because R4.3 ran in **checkpoint mode (MAX_ROUNDS:1)**, the cautious first-run shape (orchestrator integrates + watches). First run VALIDATED the whole pipeline + integration + FLAG-FOR-BEN valve → caution satisfied → **switched to AUTONOMOUS**. **R4.4 = autonomous loop, Task `w58ga30ih` / Run `wf_4bdf46be-a41`** (script `.addl/phase-4-meta/workflows/f-full-r4-4-autoloop.js`; mode=autonomous, MAX_ROUNDS=3, integWorktree=`benten-wt-fixint` on `f-full-r4-fix @ 59cffd07`; canon marks R4.3-fixes-applied + 4-flags-RESOLVED so it hunts RESIDUAL not re-flags settled). It reviews→(if 0 new BLK/MAJ)converge-exit, else fix→**in-workflow integrator** (the one not-yet-validated part — WATCH)→compile→re-review→loop. **ON RETURN:** review decision-log + verify the in-workflow integrator's commits + compile-state; if CONVERGED → harvest (f-full-r4-fix → corpus + R0.5→canonical) → **R5 via `addl-r5-impl-to-green`**; if hit MAX_ROUNDS without converging → inspect. HOLD all tags pending Ben.

*Updated 2026-06-03 NIGHT. Compact-survival: R4.4 AUTONOMOUS loop `w58ga30ih` (proceed-to-convergence; in-workflow integrator on `benten-wt-fixint`); 4 flags confirmed; #64+bounded-decode queued for doc-cascade; on converge → harvest → R5.*

### R4.4 LIVE-WATCH + LENS-ARCHITECT WIRED INTO THE FIX-LOOP (2026-06-03 NIGHT, post-compact)
**R4.4 progress (watched via journal `…/subagents/workflows/wf_4bdf46be-a41/journal.jsonl` + integ-worktree git):** Round 1 ran end-to-end + **the first autonomous in-workflow integration SUCCEEDED** — `f-full-r4-fix` advanced `59cffd07 → 36247a08` ("test(fixloop r1): apply 5 reviewed fixes"; author Benten-Ben; 5 corpus test files +229/-84; clean worktree; sane). Round-1 panel = full 15/15; found 6 = **2 MAJOR fixed** (F-ABUSE-AAD-COMPOSITE = AAD-version/coarse_epoch sibling-sweep miss; **F-BD-001 = bounded-decode META#629 flagship**) + **1 MAJOR correctly DISAGREED** (F-SEAM-1-EXECWORKFLOW, real:false, reasoned — HARD-RULE-12c) + 3 MINOR fixed; **all 5 fix-reviews APPROVE with would_fail_on_revert=true** (substantive). Now mid **round-2 re-review** on the post-fix corpus (MAX_ROUNDS=3). Still `running`; auto-notifies on completion.
**Ben caught a real gap (2026-06-03):** the as-run R4.4 autoloop (`f-full-r4-4-autoloop.js`) has a **hand-inlined fixed 15-lens set with NO lens-architect Phase-0 and NO completeness-critic** — so its convergence is gated on a fixed panel (could falsely converge if the catching lens was never in the set). Verified by reading the script: Q1 (full panel every round) ✓, Q3 (full as-Ben fix pipeline + iterate-until-0-new-BLK/MAJ) ✓, **Q2 (lens set "decided earlier in the workflow" by a lens-architect) ✗** — the architect lived only in `addl-review-council.js`.
**FIX (Ben-approved "proceed as you see fit"):** wired the lens-architect (Phase-0 architect + architect-reviewer) **+ a per-round completeness-critic whose missed-lenses GROW the panel and block convergence** into the generic **`converging-fix-loop.js`** (commit **`8c40e60a`**; `node --check` clean; all validated mechanics preserved; `args.lenses` still skips the architect). README + memory `feedback_workflows_as_executable_methodology` updated. Editing the generic file does NOT touch the running R4.4 (it executes from its own already-loaded concrete script).
**ON R4.4 RETURN (the plan Ben approved):** (1) verify the in-workflow integrator's final commits + compile-state + decision-log + converge verdict; (2) **architect-gate the actual convergence** = run ONE `addl-review-council` (R4 tier, lens-architect ON) over the loop's output corpus — if 0 new BLK/MAJ + no missed-lens → genuinely converged; if it surfaces a lens-gap the fixed-panel loop missed → one more fix round (now via the upgraded architect-gated loop); (3) on convergence → harvest (`f-full-r4-fix` → corpus, R0.5 → canonical R0) → **R5 via `addl-r5-impl-to-green`** carrying the r4-triage §3.B R5-FILL list. If R4.4 hit MAX_ROUNDS without converging → inspect, don't auto-advance. **Doc-cascade recon (this session):** SECURITY-POSTURE maxes at #31 in-tree; the F-full design reserves the whole #32–#64 panel + #31→#62 split (R2 F-DISC-1 asserts "for EACH #30..#63") → it's a coordinated post-convergence coherent landing, NOT a piecemeal #64-now mint. **HOLD all tags pending Ben.**

*Updated 2026-06-03 NIGHT (post-compact). Compact-survival: R4.4 round-1 integrated clean (`36247a08`) + mid round-2; converging-fix-loop UPGRADED with lens-architect+completeness-critic (`8c40e60a`); on R4.4 return → architect-gated `addl-review-council` convergence check → harvest → R5; doc-cascade is a coordinated post-convergence landing.*

### R4.4 DONE (converged:false @ MAX_ROUNDS) + SS-AAD FREEZE-BYTE FORK → DESIGN COUNCIL (2026-06-03 NIGHT)
**R4.4 autonomous loop COMPLETE: `converged:false, rounds:3`** — 13 fixes integrated across 3 rounds (r1 `36247a08` 5 / r2 `a221160a` 7 / r3 `85ae77bc` 1); **corpus = `phase-4-meta-core/f-full-r4-fix @ 85ae77bc`, compile-GREEN** (all 4 crates, verified via BASH — my first gate was a FALSE-RED from the zsh `$feats` word-split bug, the very gotcha the loop memory warns about; re-ran via `bash` heredoc with `"${@:2}"`). Monotonic (2 MAJ → 1 BLK+1 MAJ → 1 MAJ). The loop caught real freeze-gating things R3/R4.1/R4.2 all missed: **0x6510 AAD carrying coarse_epoch + aad_version=0x02** (both contradicting RULING-1 + the sibling fixes — reconciled), bounded-decode flagship two-bound completion, the audience divergence. It returned un-converged ONLY because round-3's fix IS the flagged one — the autonomous loop + FLAG-FOR-BEN valve worked exactly as designed.
**THE FORK (freeze byte on the DEFAULT codepoint — Ben-gated):** canonical **0x6510/0x6610 Sealed-Sender ENVELOPE AAD field-set.** f_lc_hpke bound `{codepoint, body_cid, recipient_key_generation}` (NO audience); siblings + §3.3 carry audience → cross-engine AEAD-open divergence. Loop best-effort = the UNION `{aad_version, codepoint, audience(sorted-recipient-DID-list), body_cid, recipient_key_generation}` (coarse_epoch removed), ~80% confident (multi-stanza line-474 set + single=degenerate-stanza), flagged because the spec never freezes the single-recipient drop AAD with a golden. **Ben answered the AskUserQuestion: "fine with A (union) + drop token coarse_epoch — UNLESS there's a more ideal/elegant/stronger/permanent shape"** → invoked the extra-reflection-pass. Candidate improvement under test: AAD is authenticated-but-PLAINTEXT + this is Sealed-Sender → raw recipient-DID-list in a 0x6610 GROUP AAD may LEAK the recipient set/social graph → **audience-as-commitment** and/or **single=degenerate-group unification** could be stronger.
**DESIGN COUNCIL LAUNCHED: Task `wee8mn3jv` / Run `wf_2588f623-b41`** (script `…/workflows/scripts/ss-aad-design-wf_2588f623-b41.js`) — 6 design lenses (aead-binding-sufficiency / privacy-metadata-leak / wire-economy-extensibility / unification-elegance / threat-model-forward-bug / spec-interop-fidelity) → adversarial synthesis → recommended permanent shape + migration-delta-vs-Option-A + token-binding coarse_epoch ruling. **ON RETURN:** if it confirms Option A is ideal → keep corpus as-is; if it proposes a stronger shape → surface to Ben (it's still his freeze byte) → apply the migration delta to f_lc_hpke/f_lc_abuse/f_inv16_1/f_aad_2 → **re-run the upgraded converging-fix-loop to confirm clean convergence** (0 new BLK/MAJ + no missed-lens) → harvest (`f-full-r4-fix` → corpus, R0.5 → canonical R0) → **R5 via `addl-r5-impl-to-green`** (carry r4-triage §3.B R5-FILL). **QUEUED spec-correction (tracked-doc cascade):** R0.5 §3.3:484 + Compromise #43:1043 prose ("audience+coarse-epoch on the wire") contradicts §3.10:790 + M-14 + RULING-1 → correct to "audience only; coarse_epoch is Layer-D-only" (X-Wing-class internal-spec-inconsistency fix). **HOLD all tags + the SS-AAD freeze pending Ben's final shape ruling.**

*Updated 2026-06-03 NIGHT. Compact-survival: R4.4 DONE (converged:false@3rounds; corpus `85ae77bc` compile-GREEN holding Option-A union); SS-AAD freeze-byte fork → design council `wee8mn3jv` in flight (extra-reflection-pass for the ideal permanent shape, Ben-invoked); on return → surface shape → migrate if stronger → re-loop to converge → R5. Spec §3.3/#43 coarse-epoch prose correction queued.*

### SS-AAD DESIGN COUNCIL RETURNED + BEN RATIFIED THE FREEZE PACKAGE (2026-06-03 NIGHT)
**Council `wee8mn3jv` (6 lenses + synthesis) DONE — the extra-reflection-pass paid off.** It found Option A is right for `0x6510` BUT the `0x6610` GROUP AAD publishes the **raw membership roster + raw set-id in plaintext**, contradicting the project's OWN §3.9/#61 blinding posture (set-identifying material never in the clear). **Ben RATIFIED the full package (AskUserQuestion):**
- **`0x6510` single = Option A unchanged:** `{aad_version(0x01), codepoint, audience, body_cid, recipient_key_generation}`.
- **`0x6610` group = BLIND the two roster fields:** `sorted_member_dids[]` → **`audience_set_commitment` = BLAKE3(0x01 ‖ lp(did_i)…)** over canonical sorted list (32B); `membership_set_id` → **`membership_set_id_commitment` = HMAC(K_Set,"benten:setid:v1"‖id) trunc 32B** (the §3.9 topic construction). Recipients hold K_Set → recompute+verify; relay sees opaque tags; all bindings preserved. HONEST scope: identity-HIDING not unlinkability (per-send salt = **U25 CODEPOINT-RESERVE v1-GM**, additive).
- **`body_cid` → self-describing CIDv1** (multihash-len-prefixed `0x01 71 1e 20 ‖ 32B`), NOT bare-32 (baked-in #5; restores U3 injectivity). Both codepoints.
- **bind `stanza_count`** (u32 BE) in `0x6610` (relay-truncation/censorship defense).
- **token-binding AAD: DROP coarse_epoch** (already in corpus; freshness = UCAN nbf/exp + nonce-cache).
- **DON'T unify u16/u32 length-prefix widths** (drop-band vs MembershipSet-band separately-frozen; corpus author adjudicated).
**EXECUTION (in flight):** (1) **spec R0.5→R0.6** — spec-editor agent **`a2991d9d41588d827`** (background) emitting precise edits to `/tmp/r06edits/edits.md` (§3.3 field-sets + coarse-epoch prose, §4.0/§4.1 tables, §3.11 token, Compromise #43 prose, U25 honest-scope note, revision-history R0.6); ON RETURN → integrate onto branch `phase-4-meta-core/f-full-r0-plan-r06` off `e4fbfe73` (single-writer) + verify. (2) Then **re-run the UPGRADED converging-fix-loop** (architect ON — no inlined lenses; autonomous; corpus `f-full-r4-fix @ 85ae77bc` vs spec R0.6; canon = the ratified package) → it migrates the corpus (`f_aad_2` group AAD: 2 commitments + stanza_count + body_cid CIDv1; golden-hex regen via throwaway-compute) + re-reviews + converges (0 new BLK/MAJ + no missed-lens). (3) **Verify the new commitment golden-hex BY HAND** (freeze-gating) + compile-GREEN. (4) **Harvest** (`f-full-r4-fix`→corpus, **R0.6→canonical R0**) → **R5 via `addl-r5-impl-to-green`** (carry r4-triage §3.B). **HOLD all tags pending Ben.**

*Updated 2026-06-03 NIGHT. Compact-survival: SS-AAD freeze package RATIFIED by Ben (0x6510=A; 0x6610 blind roster+setid via BLAKE3/HMAC; body_cid=CIDv1; bind stanza_count; token drops coarse_epoch). Spec-editor `a2991d9d41588d827`→R0.6; then upgraded converging-fix-loop migrates corpus→converge→verify golden-hex→harvest→R5.*
