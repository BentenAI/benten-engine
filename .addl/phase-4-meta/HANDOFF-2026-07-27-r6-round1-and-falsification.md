# HANDOFF — Phase-4-Meta-Core R6, post-Round-1 + post-falsification-sweep

**Written 2026-07-27 for compaction.** This is the authoritative resume point. Read this end to end, then the
reading list in §9, then verify state per §8 before acting.

---

## 1. WHERE WE ARE IN ONE PARAGRAPH

We are in the R6 phase-close convergence loop for **Phase-4-Meta-Core**, the phase whose tag
(`phase-4-meta-core-close`) PERMANENTLY FREEZES the v1-beta public interface and wire formats. The freeze base
is **`7bb1a9fa`** on branch `phase-4-meta-core/r9-base`, backing **PR #1382**, with **all 13 required CI checks
green**. Round #1 of the required two-consecutive-CONVERGED pair came back **NOT-CONVERGED** (0 BLOCKER, 26
MAJOR, 129 minor/obs, 6 missed lenses) and a subsequent mutation-based falsification sweep found **17 of 36
enforcement claims unenforced**. The counter is at **0**. The next work is landing those fixes in isolable
waves, then Round #2 with a widened 31-lens panel. **Merging #1382 and creating the tag are HELD FOR BEN and
are never autonomous.**

---

## 2. THE HEADLINE FINDING (this frames everything else)

**The cryptographic substrate is sound. The freeze RECORD and its ENFORCEMENT are not.**

- Round #1 refuted 9 findings outright, downgraded 39 as mis-severity, and one lens independently re-derived
  seven prior-phase properties (canonical fixture CID, the 12-primitive guard, the UCAN depth fix, the golden
  regeneration, the BE sweep, all 116 corpus stems) and found **every one intact**.
- But **17 of the 26 standing MAJORs are one shape: the record asserts an enforcement that does not exist.**
- The falsification sweep then proved this experimentally: **17 of 36 claims escaped mutation.** Five
  big-endian prefixes flipped to little-endian in a *signing preimage* left the whole crate 227/227 green.
  Deleting `impl Drop` from a secret type left 214/214 green. A **required** CI job certified a dependency's
  unrelated `compile_error!` as its own gate and printed PASSED.

**Net: the document Ben would sign at the tag currently overstates what the binary enforces.** That is the
thing this phase must fix before tagging.

---

## 3. HARD STATE (verify, do not trust)

| Fact | Value |
|---|---|
| Freeze base | `7bb1a9fa` — branch `phase-4-meta-core/r9-base`, == PR #1382 head |
| Required CI checks | **13** (was 12; `frozen-bytes corpus (v1-beta wire freeze)` added to LIVE protection 2026-07-26) |
| CI status at base | all 13 required PASS — verified per-context that *pass* means pass, not skip |
| Non-required reds | CodeQL (0 open alerts) + `webview-e2e` (known tauri-driver flake) |
| Two-consecutive counter | **0** |
| Active worktree | `/Users/benwork/Documents/benten-engine/.claude/worktrees/r6-r1-foldin` @ `7bb1a9fa`, clean |
| Branch-protection rollback snapshot | `scratchpad/branch-protection-BEFORE-20260726.json` (12 contexts, strict, enforce_admins) |
| Orchestration branch | `phase-4-meta-core/orchestration-2026-05-26` (decision log lives here) |

**Disk discipline (learned the hard way — we hit 100% and killed an agent):** a full gate needs **~40 GiB**
headroom, not 20. One workspace `--all-targets` build plus 15 `cargo +nightly public-api` runs consumed ~35
GiB. Reclaim with `cargo clean --manifest-path <worktree>/Cargo.toml` (raw `rm` and `git worktree remove` are
permission-denied). Check `df -h /Users` before every cargo step; stop below ~12-15 GiB.

---

## 4. CORRECTIONS — things in the docs that are WRONG, and would mislead you

These matter more than the new content, because a fresh agent will otherwise trust them.

1. **D-93's "no required check runs the test suite" is SUPERSEDED.** It was true when written. The S-4 fix
   landed since: `cross-leg byte-equality gate (T9)` (required) carries `needs: [build-and-test]` +
   `if: always()` + `assert-needs-succeeded`, so the `--workspace` nextest at `ci.yml:220` **transitively
   blocks merge**. Six of six required aggregates use that composite action — **the whole freeze currently
   rests on it.** Four of the sweep's five specs inherited the stale premise; the runner caught it.
2. **F-066's "the content exists in no artifact" is WRONG.** 62 lens JSONs are recoverable from git history
   (force-committed, later removed from the tree). The council agents cannot read gitignored `.addl/` and
   inferred absence from invisibility. Recovery: `git log --all --diff-filter=A --name-only --pretty=format:
   -- '.addl/**/*.json' | sort -u`, then `git show <commit>:<path>`.
3. **`.github/branch-protection.yml` does NOT contain the frozen-bytes context.** It was added to LIVE
   protection via the API only. The declarative spec has 30 entries, zero matching `frozen`. Ben ruled:
   **promote the 18 declared-but-not-live contexts.**
4. **The architect's convergence-bar text says "32 lenses"; the panel reported 25/25.** Unreconciled —
   confirm no 7 lenses were composed-then-dropped before trusting round #1's completeness.

---

## 5. BEN'S RULINGS THIS SESSION (all binding)

- **HARD-RULE-12 reasserted, sharply:** do everything NOW unless we disagree or it belongs to a *specific
  named later phase*. Cost is not a deferral reason. This flipped several items I had been leaning to defer.
- **Four wire/API changes APPROVED** (all permanent-after-tag): F-020 Drop-bundle per-Recipe AAD
  bundle-unique binding · F-042+F-043 handshake `peer_id`↔`peer_did` binding + responder identity in signed
  bytes + `is_authenticated` · GAP-6 vault frame generation counter · F-059 `#[non_exhaustive]` sweep.
- **`#[non_exhaustive]` per-type rule (Ben flagged the path-blocking risk):** apply freely to types the
  library RETURNS (outcomes/stats/reports — consumers only read them); for types consumers BUILD and pass in,
  apply **only** where a constructor lands in the same commit; anything where literal construction is the
  intended ergonomic, leave and list for Ben.
- **Round #2 = FULL 31 lenses, not lean.** Reversal of my earlier advice, for three reasons: the
  two-consecutive rule is a *stability* measurement and changing the panel breaks comparability;
  "always clean" is ambiguous between sound-surface and weak-lens; and **the surfaces the drop-candidates own
  are precisely the ones about to change** (AAD, handshake bytes, vault frame, ~95 structs). Economize by
  scoping confirmatory lenses to "re-derive this named list", not by cutting lenses.
- **Composing = 3-way split** (Substrate → Admin → Surface), and **each sub-phase gets its own tag** —
  Composing is foundational enough for later work that solidity beats ceremony-avoidance.
- **UI/UX Fork C reshaped by Ben and it is better:** not "primary modality" but **three peer editing surfaces
  over ONE canonical graph** (visual composer, DSL compiling to native graph structure, graph-canvas), which
  creates a **round-trip-losslessness invariant** and one sharp R1 question: is `graph → DSL` a deterministic
  pretty-print, or is the DSL one-way? If one-way, a visual edit orphans someone's DSL.
- **Forks A/D/E/G ratified; B = watch-list with triggers.** A: Svelte outright, no pre-sanctioned fallback —
  a pivot is a Ben conversation. D: constrained typed style tokens (widening is additive, narrowing is
  breaking). E: accept the bespoke-IR cost. G: rebuild accepted pending an honest LOC/time estimate at R1.
- **Recovery direction ratified:** extensible `RotationAuthKind` codepoint + LOUD `NoRecovery` default +
  social-guardian as first authorizer + the timelock/broadcast/veto/duress envelope + authority-recovery
  split from confidentiality-recovery in the type system.
- **The 93 orphaned findings:** recover from git history (see §4.2), don't write them off.
- **Argon2id floor:** keep at 8 (decoder/panic-guard bound, not a security floor). Protection moved to the
  binding provisioning contract in Row D-69. See D-94/D-95.

---

## 6. THE WORK QUEUE (in order)

### 6a. Fix waves — land SEPARATELY with a re-gate between each, so a regression stays isolable

| Wave | Contents |
|---|---|
| **W-CI** | F-001 spec-file omission · F-015 spec-check warn-by-default + PAT-less + not required · F-032 fourteen napi pins in zero lanes · **F-060 `rust-toolchain.toml` `channel="stable"` shadows the MSRV selection across `msrv.yml` + `ci.yml` (4 legs) + `determinism.yml` + `admin-shell-e2e.yml` — MSRV has never actually been tested** · CE-07/CE-08 one-word grep fix (`grep -qF 'baked-in #17'`) · CE-02 a REQUIRED context that is `exit 0` · promote the 18 declared-not-live contexts |
| **W-SUB** | F-014 catalog tautology (**on the merge-blocking path**) · F-072/FS-11 my own untested D-95 seal-side guard · F-073 25 stale ignore arms across 15 files + register the materializer-determinism gate · FS-05 the `contains("-8")` predicate · FS-06/FS-07 unfalsifiable grep-defenses |
| **W-BYTES** | E-02 plugin-manifest signing preimage (5 BE fields, **no golden anywhere**) · E-07/E-08 AEAD AAD layout goldens · E-03/E-04 chunked storage envelope · E-05/E-06 remote-permission operation tags · extend the `--lib` arm beyond 2 of 13 corpus crates |
| **W-REC** | F-009/010/011/056/057/063 freeze-record integrity (incl. three trait methods `git log -S` proves never existed) |
| **W-AUTH** | F-028 fail-OPEN default documented as fail-CLOSED · F-034 mirror-scanner root · F-042 handshake binding · F-045 Compromise #25 **MECHANISM-ABSENT on all three cited defenses** · F-048 |
| **W-WIRE** | The four Ben-approved wire changes (F-020, F-042/043 wire half, GAP-6, F-059) |
| **W-DOC** | F-071 five invariants misdescribed · F-049 `SwapDecrypted` zeroize · C66 zeroize half unenforced · GAP-5 `FileVaultStore` doc honesty |
| **W-MINOR** | The 129-item tail. Partitions M1-M15 are pre-computed in the council output (task file `wpfddi0vj.output`). **86 of them are UNVERIFIED — do not fix from finding text alone; several MAJORs were 30-90% wrong on mechanism while right on location.** |

### 6b. Then Round #2 — 31 lenses (25 existing + 6 new)
New dimensions, each proved necessary by a completeness-critic gap: `cross-target-and-deployment-shape-byte-parity`
(nothing proves ANY Phase-4-Meta-Core frozen byte is identical on a second architecture) ·
`required-check-set-completeness` · `at-rest-durability-atomicity-and-rollback` ·
`availability-key-loss-and-unrecoverability` · `local-host-adversary` · `concurrency-and-toctou-on-layer-a-d`.

### 6c. Still parked (no clock)
Composing spikes 2 and 3 (UI taxonomy stress-test; strict-CSP interpreter) — spike 1 (confidentiality
recovery) is DONE and its one pre-tag item (`#[non_exhaustive]` on `AttestationKind`) has landed. The
toolchain-pinning mechanism (commit workspace lockfile + pin tsc + `--frozen-lockfile` + dated nightly +
versioned `cargo-public-api`) — **all four parts or none**; the STANDING PROHIBITION covering it landed at
`7bb1a9fa`.

---

## 7. THINGS THAT NEED BEN (do not decide these)

1. **Merging PR #1382** and **creating the tag** — always, no exceptions.
2. **The napi read-principal design** (F-030/F-026): "fix the code to match the docs" is settled, but
   sentinel-vs-caller-principal is an architectural fork. Surface the shape before implementing.
3. **GAP-5 / Row D-65:** does the in-memory `FileVaultStore` warrant a numbered Compromise? (Doc honesty is
   fix-now regardless; building a real encrypted-at-rest store is genuine Composing work because it needs the
   provisioning path that does not exist yet.)
4. Any *new* wire-affecting change beyond the four already approved.

---

## 8. VERIFY BEFORE ACTING (state drifts)

```
gh pr view 1382 --json headRefOid,mergeStateStatus
gh pr checks 1382                       # confirm per-context; "skipping" is NOT "pass"
gh api repos/BentenAI/benten-engine/branches/main/protection --jq '.required_status_checks.contexts'
git -C .claude/worktrees/r6-r1-foldin log --oneline -1 && git -C ... status --porcelain
df -h /Users                            # need ~40 GiB before a full gate
```
Also check for in-flight agents by **transcript mtime**, never by notification absence — workflows survive
app restarts and machine sleep (both happened this session and work continued).

---

## 9. READING LIST (in order)

1. **`CLAUDE.md`** — end to end. The status banner is current as of 2026-07-27.
2. **`.addl/phase-4-meta/NIGHT-SHIFT-DECISIONS-2026-07-04.md` — D-93 through D-99.** This is the live record
   and the single densest source. D-99 (falsification sweep) and D-98 (round #1) are the most load-bearing.
3. **This handoff.**
4. **`.addl/phase-4-meta/NIGHT-SHIFT-2026-05-27.md`** — the resume-contract block at the TRUE BOTTOM.
5. **Memory index** `~/.claude/projects/.../memory/MEMORY.md`, then at minimum the Foundational tier plus:
   `feedback_falsification_sweep_before_every_phase_close`, `feedback_findings_carry_substance_not_bare_ids`,
   `feedback_signed_wire_change_sweep_all_embedding_goldens`,
   `feedback_close_the_minor_tail_before_the_next_round`,
   `feedback_two_consecutive_converged_and_minor_every_round`, `feedback_full_phase_review_every_round`,
   `feedback_review_finding_ground_truth_verify`.
6. **The raw council + sweep outputs** (huge, but authoritative for per-finding detail):
   `scratchpad/.../tasks/wpfddi0vj.output` (round #1) and `.../wjep08tzv.output` (falsification sweep).
   Per-agent detail in `subagents/workflows/wf_7f228598-d11/journal.jsonl` and `wf_1621fd70-94c/journal.jsonl`.
7. **Composing pre-work** (parked, Ben-facing): `COMPOSING-SPLIT-PROPOSAL.md`,
   `COMPOSING-SPIKE1-UIUX-R0.md` (+ its addendum carrying Ben's fork rulings),
   `COMPOSING-SPIKE2-RECOVERY-PRESENTATION.md`, `R6-LEAN-ROUND-LENS-ANALYSIS.md`.
8. **Then read whatever else looks relevant** — this list names what was load-bearing *at write time*. The
   docs under `docs/` that the freeze actually governs (`V1-FROZEN-INTERFACE.md`,
   `V1-WIRE-FORMAT-INVENTORY.md`, `SECURITY-POSTURE.md`, `INVARIANT-COVERAGE.md`, `CRYPTO-CODEPOINTS.md`) are
   the artifact under review and are worth sampling directly.

---

## 10. METHOD LESSONS EARNED THIS SESSION (apply them)

- **A passing test is not evidence of enforcement.** Only failure-on-mutation proves it. Run the
  falsification sweep BEFORE the council, not after. (17/36 escaped; ~1.7M tokens vs ~19M for a round.)
- **Ground-truth everything yourself, including your own tooling.** I reported "all 12 required checks pass"
  from a script I wrote that treated `skipping` as acceptable — I coded my own blind spot. Three separate
  confident agent claims inverted on direct inspection this session, in both directions (a verifier wrongly
  said a workflow file did not exist; an analyst rightly refuted *me*).
- **When an agent says something does not exist, check whether it merely could not look.** Gitignored paths
  and git history are invisible to subagents.
- **Cite-repointing must verify the CLAIM, never preserve the prior target.** "Content-preserving repoint"
  guarantees an already-wrong cite stays wrong while the detector goes green. Prefer `path::symbol`.
- **Prefer under-claiming.** Every over-claim this session came from an agent writing more confident prose
  than the code earned — including one that read a workflow's aspirational header comment instead of its body.
- **Salvage, don't restart.** Agents died three times (app restart, API error, machine sleep); partial
  worktree state was coherent and correct every time.
