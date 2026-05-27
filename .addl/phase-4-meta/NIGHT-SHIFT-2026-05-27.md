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
