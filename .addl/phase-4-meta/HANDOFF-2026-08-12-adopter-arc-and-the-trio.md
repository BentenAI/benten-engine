# HANDOFF — 2026-08-12 · the adopter arc, the trio, and what is owed

**Supersedes `HANDOFF-2026-08-11-fix-waves-and-fit-gaps.md` as the resume point.** That doc is
still correct for W1/W2 and the round-#1 ledger; everything below is newer.

**Read order:** CLAUDE.md banner → this doc → `docs/future/engine-fit-and-gaps.md` (the live
adopter ledger) → `NIGHT-SHIFT-DECISIONS-2026-07-04.md` D-100→D-107.

---

## 0. State

| | |
|---|---|
| PR | [#1382](https://github.com/BentenAI/benten-engine/pull/1382), branch `phase-4-meta-core/r9-base` |
| Head | **`085a0d09`** |
| CI | **UNVERIFIED.** Mid-run at write-time: 50 pending, 9 pass, and **2 REQUIRED showing fail** — `TS API drift detector` and `cargo-audit (belt-and-suspenders)`. Both were green earlier today, so this is either a stale partial run or new drift. **Strictly re-verify before anything approaches the tag.** |
| Converged rounds | **0 of 2.** Round #1 NOT-CONVERGED; nothing has re-run. |
| Disk | 94%, 13 GiB free. `benten-wt-r9base/target` is 32 G and cannot be cleaned while work runs. |
| Held for Ben, always | merging #1382 · creating the tag |

**Standing law unchanged:** never `--admin-bypass`, never force-push, NORMAL `--squash` only;
ORCH ground-truth-verifies every convergence-blocking finding personally (§3.5n); dispatch agents
`run_in_background: true`; branch protection modified only by the orchestrator, additively, after
a job is proven green on #1382.

**⚠️ Verify CI the strict way** — absent and skipped are NOT passes. That mistake has been made
twice. And a red whose *name* you recognise is not a red you have *diagnosed*: the four `@stable`
legs had **three different causes** in one day and I mislabelled two of them before opening a log.

---

## 1. What landed today (all pushed)

| commit | what |
|---|---|
| `fd563edd` | **W3** — SANDBOX return-ABI fails closed (the `v128` catch-all covered SIX `Val` variants, TWO reachable; worst shape `(result i32 v128)` → `[7,0,0,0]++[0;16]`, a correct leading scalar lending credibility to sixteen invented bytes); three phantom config knobs wired; five false records struck. **8 mutations run, all 8 bite.** |
| `e032ee05` | rustdoc broke the `@stable` legs (NOT clippy); `Cid::from_blake3_digest` naming DECIDED — keep the algorithm in the name |
| `b4c03750` | cite-drift: 22 line-anchors took out all eight build legs |
| `ad0b21d4` `4de24489` | the **measured** self-balancing law + the reachability question ANSWERED |
| `3e263a0c` | the strengths record + §3.2b FALSE-RECORD corrected |
| `1de42d17` | GPU-compute boundary + IVM §7b + the trio sequencing §4.174 |
| `085a0d09` | the subgraph-IS-the-program addendum + three decisions I had reasoned and never written |

---

## 2. THE PRIORITY — Ben-set 2026-08-12

**Binding → addressing → aggregation.** `phase-4-backlog.md` **§4.174** is the sequencing row;
§4.170/§4.171 are the receiving rows.

Two unrelated outside evaluations hit the same three gaps, which reframes them from adopter
requests into **v1 completeness questions**. A fourth signal points the same way:
**SANDBOX-frequency**. SANDBOX was the *test* of whether twelve primitives suffice — reaching for
it constantly is evidence of THESE gaps, not of a missing thirteenth. **Falsifiable prediction
recorded: closing the trio should measurably reduce SANDBOX dependence.**

**The tag is NOT blocked by this priority.** Pre-tag exposure is two small items: the
position-scoped `$` reservation (narrowing, no valve) and ONE verification — is READ's
accepted-property set frozen closed? Aggregation needs nothing pre-tag.

**§4.171 may not need building at all.** Ben proposed aggregation as a subgraph triggered on
change minting a new version of a canonical total-node. SUBSCRIBE is post-commit and async, which
dodges all three blockers. Our own foundational rule puts the burden on the ENGINE feature to
prove the app-layer pattern insufficient — **and we never attempted that proof.** Step 3 is now
"build the pattern, then decide," with four failure conditions named in advance
(`ivm-aggregation.md` §10).

---

## 3. The adopter arc — where each answer stands

**Canonical ledger: `docs/future/engine-fit-and-gaps.md`.** Six tracked design records now:
`binding-grammar.md`, `ivm-aggregation.md`, `bounded-resources.md`,
`bounded-resources-spectrum.md`, `bounded-resources-balancing-law.md`, `engine-fit-strengths.md`,
`gpu-compute.md`.

**Museum (3 asks):** decimal ANSWERED with the sharpening that scale belongs in the schema — and
that our answer *beats* `Value::Decimal`, because per-value scale gives the same amount two CIDs.
Bounded resources: single-owner, and their own topology (Ben: "the line just backs up on the other
till") puts them in the **failover** column where single-owner goes from LAST to FIRST. Identity:
the rotation tuple stays unchanged, **DECIDED**.

**LLM (4 asks + §2):** `#[non_exhaustive]` answered (not now-or-never). `v128` **LANDED**. Config
surface **LANDED**, and bigger than they found — `engine.toml` is never loaded at all. CID naming
**DECIDED**. §2 dataflow answered, with the honest caveat that the design covers only the anchor
half of relative addressing.

---

## 4. OPEN FOR BEN

1. **v1-completeness** — does `v1-beta` ship with composition that does not compose and an IVM
   that cannot sum a column? ORCH lean (a): tag as planned, both are additive Composing builds.
   Recorded at fit-gaps §1 so it cannot resolve by expiry.
2. **wasmtime RUSTSEC-2026-0222** — suppressed on structural unreachability (one process-wide
   `OnceLock<Engine>`), premise enforced by a mutation-proven guard. No fix in the pinned 43.x
   line; remediation is a 3-major bump. Recommendation: hold it off the freeze branch.
3. **Promote canonical-bytes / golden-hex to REQUIRED** (carried).
4. **The parked Composing pre-work** (carried).
5. **Kernel sharing, re-asked** — the security pass killed peer-authored MSL permanently, but a
   peer-authored *graph over a fixed op vocabulary we lower* is statically boundable before
   anything reaches the device. Worth re-asking, not settled by the MSL answer.

---

## 5. NOW-OR-NEVER accumulated (all ride W-REC #47 unless noted)

- **`verify_in_trust_domain` accepts EXPIRED credentials** — no clock, structurally unable to
  check, already in the frozen `benten-id` baseline. It is exactly what an offline reciprocity
  gate would call. (It also routes through `Did::from_string_for_test_fixture`.)
- **`CredentialSubject`** — one string claim, not `#[non_exhaustive]`, inside signed bytes.
- **`RotationAttestation`** — byte-pinned, in the frozen baseline, named in NEITHER
  `V1-FROZEN-INTERFACE` nor `V1-WIRE-INVENTORY`. Disclosure owed; tuple unchanged by decision.
- **`V1-FROZEN-INTERFACE` §4.62** — freezes `BlobBackend` naming `put_blob`/`get_blob`/`has_blob`
  at a file that does not exist. Fix the RECORD. (`has_blob` is a genuine gap: no existence check
  exists at all.)
- **`benten_core::Value`** inventory clause · **`DSL-SPECIFICATION.md`** normative claims with
  zero writers · the binding-grammar four · `WriteContext::enforce_system_zone`
  · `register_peer_did`'s present-tense rustdoc · READ addressing-set verification
  · **`CapabilityEnvelope`**: do NOT add `runs_gpu` — one free disclosure sentence instead.

---

## 6. Remaining path to the tag

W-SUB residuals → **W-REC** (~30 false records + everything in §5) → W-WIRE (8, EngineView first)
→ W-MINOR (129, **verify the 86 unverified before fixing**) → re-run the falsification sweep →
Round #2 at 31 lenses → a second consecutive CONVERGED → pre-tag bundle → **hold for Ben**.

---

## 7. Operational lessons that cost real time today

- **`/tmp` survives a macOS reboot** (3-day prune, not boot-wipe) — a killed workflow's artifacts
  and the `~/.claude` journals both survived; resume-from-cache worked three times.
- **Verify against the COMMIT, never the worktree.** Five agents reported five different
  `git status` results for one tree because I was committing to it while they ran. Pin briefs to a
  SHA and read with `git show <sha>:<path>`.
- **Masked-success traps, three variants in one day:** `2>/dev/null` hiding stderr; `timeout`
  absent on macOS; `PIPESTATUS` under zsh. All produced a green `0`.
- **Agents violate emit-to-/tmp** unless the brief makes the worktree structurally unreadable. One
  ran `git checkout -- .` mid-flight and reverted a sibling's work.
- Disk hit **100%** (217 MiB free) mid-session; reclaimed 9.7 G from an idle worktree.
