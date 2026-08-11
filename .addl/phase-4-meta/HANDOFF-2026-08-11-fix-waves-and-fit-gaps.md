# HANDOFF — 2026-08-11 · W1/W2 landed + the engine fit-and-gaps arc

**This supersedes `HANDOFF-2026-07-27-r6-round1-and-falsification.md` as the resume point.** That
doc is still the authoritative record of *round #1's findings and the falsification sweep* — read
it for the 26 MAJOR / 129 minor ledger and the 17-of-36 escape list. What it does not know about
is everything below.

**Read order for a fresh session:** CLAUDE.md banner → this doc → `NIGHT-SHIFT-DECISIONS-2026-07-04.md`
D-100→D-107 → the 2026-07-27 handoff for the round-#1 ledger detail.

---

## 0. Where the tag actually stands

| | |
|---|---|
| PR | [#1382](https://github.com/BentenAI/benten-engine/pull/1382) — branch `phase-4-meta-core/r9-base` |
| Head | **`96d73daf`** (was `df0c8287` at the last handoff) |
| Converged-round counter | **0 of 2.** Round #1 was NOT-CONVERGED; nothing has re-run since. |
| Frozen-bytes corpus | 123 targets; entry count, floor assertion, and the floor's prose message all agree |
| Held for Ben, always | merging #1382 · creating the tag `phase-4-meta-core-close` |

**Standing law unchanged:** never `--admin-bypass`, never force-push, NORMAL `--squash` only;
ORCH ground-truth-verifies every convergence-blocking BLOCKER/MAJOR personally (§3.5n); dispatch
agents `run_in_background: true`; HARD-ABORT new dispatch above 97% disk. Branch protection is
modified **only by the orchestrator, additively, after the new job is proven green on #1382** —
never from inside a workflow (a required context that never runs makes the PR permanently
unmergeable: `enforce_admins=true` + no-admin-bypass = no legal escape).

---

## 1. What landed since the last handoff

### W1 — the enforcement layer (`df0c8287`)
Made the freeze's own gates real. The MSRV gate had never actually tested MSRV across four
workflows; a required job's grep was satisfied by a *dependency's* error message; every byte-pin
the sweep proved missing got a golden captured from the real encoder. **Now verified in CI**, not
argued: the job prints `cargo 1.95.0 … OK: MSRV 1.95 toolchain is in effect`.

Four verifier findings closed on landing, including the wave reproducing its own defect
(`admin-shell-e2e.yml` got the toolchain pin on both jobs and neither got the assertion — a pin
with no falsification arm, which is precisely what W1 existed to eliminate).

### W2 — substance (`ceb027ba`)
- **B8 landed real.** Putting 17 previously-unrun napi pins on a lane exposed that Phase-1's
  input-validation deliverable never shipped — 5 DoS vectors unbounded at the napi boundary.
  **0/5 → 17 passing.**
- **A silent data-loss bug, caught by W2's own verifier.** `node.rs` accepted JSON nested to 128
  while the canonical decoder stops at 64, so a property bag nested 65–128 deep was accepted,
  hashed, persisted — and then could not be decoded on read. Fixed by *deriving* the cap from
  `benten_core::MAX_VALUE_DECODE_DEPTH`, which makes the mismatch unrepresentable rather than
  merely corrected, and guarded by
  `crates/benten-core/tests/no_boundary_accepts_deeper_than_the_decoder_returns.rs` — a scan over
  the CLASS of `*_MAX_DEPTH` declarations with a vacuity floor, proven by mutation.
- **W2 introduced a FALSE-RECORD and its own verifier caught it.** The wave described
  `input_limits.rs::limit()` as "the production firing site." It is not — its only callers sit
  inside `#[cfg(any(test, feature = "in-process-test"))] mod testing`, and `default =
  ["napi-export"]` does not enable that feature. B8 genuinely closed the Phase-1 R3 contract *as
  that contract was written* (against `benten_napi::testing::*`); that is a different claim from
  production coverage, and both docs now say which sites fire today.

### The `96d73daf` re-gate fix-pass (2026-08-11)
Two REQUIRED checks failed on `ceb027ba`; neither was the failure it looked like.

1. **Parity** — a cross-language mirror miss (§3.5g): W2 rewrote `E_INPUT_LIMIT`'s fix-hint in
   `ERROR-CATALOG.md` and never regenerated `errors.generated.ts`. Underneath that, the new hint
   text (a) described the JSON depth defect as *live* when the W2 residual had already closed it —
   a FALSE-RECORD in the inverse direction — and (b) carried internal orchestration vocabulary
   ("surfaced for Ben", "R6 round #1, B8", a memory filename) into an artifact that **ships to
   consumers in the npm package**. Provenance moved to the doc-only blockquote; the `Fix:` field
   is now a caller's instruction. Also swept a straggler from my own earlier fix: `node.rs:131`
   still told the user "128-level depth limit" while the limit was 64 — the message is now derived
   from the constant for the same reason the constant is derived from the decoder.
2. **cargo-deny** — four new advisories, one a real vulnerability. **See §3.**

---

## 2. The engine fit-and-gaps sub-project (NEW — the main unrecorded arc)

Two unrelated outside evaluations arrived within days — a **museum revenue-operations system**
(replacing Versai; ticketing, memberships, camps, a statutory-audit ledger, POS tills on one LAN)
and an **LLM inference runtime** (Gemma MoE, content-addressed weights). They share nothing as
products and **failed the engine the same three ways**. That convergence is the finding.

**Canonical tracked ledger: `docs/future/engine-fit-and-gaps.md`** — deliberately
gitignore-allowlisted, because it is read BY review councils and its own §1 banner forbids being
used as a deferral destination, a rule nobody can check inside a gitignored file (F-066 precedent).

**Status ledger:** §3.1 DESIGNED · §3.2 ANSWERED · §3.3 DESIGNED · §3.4 DESIGNED · §3.5 OPEN ·
§3.6 CARRIED (Fork B) · §3.7 OPEN.

**Status vocabulary, deliberately narrow** (Ben caught "ANSWERED" doing two jobs): OPEN <
**DESIGNED** (shape known, *gap still open, nothing built*) < **ANSWERED** (the mechanism already
ships) < **LANDED**. Rightward only with evidence; a written design never earns ANSWERED.

**Three R0-input design records, each tracked, each with a receiving row that exists:**

| Doc | Finding | Receiving row |
|---|---|---|
| `docs/future/binding-grammar.md` | §3.1 — Phase-1 deliverable E3 reopened | phase-4-backlog §4.170 |
| `docs/future/ivm-aggregation.md` | §3.3 — bespoke abelian fold under `Strategy::B` | §4.171 |
| `docs/future/bounded-resources.md` | §3.4 — single-owner admission; also the canonical tracked home for the `executionPolicy` taxonomy, since `PLATFORM-DESIGN.md` is local-only | §4.172 |

**Three conceptual corrections owed to `ARCHITECTURE.md` / `HOW-IT-WORKS.md`** (all mine, all made
during the investigation): there is **one graph, not two**; **bounded-by-construction does NOT
require static operands** (termination rests on DAG structure + step budget + frame cap, none of
which reads an operand — so "not Turing complete" must never be cited to justify "no dataflow");
and the gap is **relative addressing, not dataflow** (`SubgraphSpec` already IS relative
addressing — we gave it to sharing and not to execution).

**The recurring shape:** every pass found the mechanism *already shipped somewhere*, unfinished.
§3.1 found four partial implementations of one feature including a **live `$input` resolver in
STREAM**; §3.2 found `bytes-cid`/`timestamp-hlc` are already schema-declared interpretations over
existing `Value` variants; §3.4 found the serial point is redb's own single-writer transaction.
Generalize, don't invent. Codified as `feedback_discover_before_you_design`.

**Pre-tag debt accumulated by this arc** (all small, all disclosure-shaped, fold into W-REC):
binding-grammar's four items (freeze-record disclosure of the STREAM sigil grammar; the
position-scoped `$` reservation + ErrorCode; the `InfiniteEmptyProducer` wart; a stale comment);
ivm-aggregation's disclosure; bounded-resources' disclosure **plus a new false record found en
route — `WriteContext::enforce_system_zone`, a zero-caller `pub fn` on the frozen baseline whose
only mention anywhere is a test comment citing it as the enforcement layer**; and a
freeze-record clause for `benten_core::Value` itself, which is named nowhere in the freeze record
despite being the property type on every Node and Edge.

**In flight at write-time:** the bounded-resources *spectrum* survey (`wpylqxjlg` /
`wf_8f21533e-dde`) — six mechanism families mapped by trust model and winning regime, widened per
Ben to cover **the whole "at most N" taxonomy, not the museum's regime alone**: timed-entry and
venue capacity, reservation slots, enrolment caps, finite and **decentralized** inventory,
limited-edition minting, coupons, grant and budget pools, discount-usage caps, license seats,
rate limits, quotas, connection-pool caps. Two axes the museum never exercised are first-class in
the brief: **is the allowance costless to move** (borrowing capacity is bookkeeping; borrowing
*stock* means shipping a box, and can fail after being agreed) and **does the bound regenerate**
(rate limits refill, stock is replenished, seats never do).

---

## 3. Open for Ben

1. **wasmtime RUSTSEC-2026-0222 — bump or keep the suppression?** A real vulnerability against
   wasmtime 43.0.2, our SANDBOX runtime. Suppressed on **structural unreachability**, not severity
   and not cost: the advisory needs two or more live `Engine` instances whose `Store`s can be
   confused, and `benten-eval` holds one process-wide `OnceLock<Engine>` with exactly one
   `Engine::new` site. That premise is **enforced, not asserted** —
   `crates/benten-eval/tests/exactly_one_wasmtime_engine_per_process.rs` fails naming file:line if
   a second site appears, with the message pointing back at the `deny.toml` entry (proven by
   mutation). **There is no fix in the 43.x line** — patched ranges are `>=24.0.12/<25`,
   `>=36.0.13/<37`, `>=46.0.2/<47`, `>=47.0.3` — so remediation is a 3-major runtime bump.
   *My recommendation:* keep the guarded suppression through the tag, bump in Composing where the
   conformance lanes can absorb a regression. Received by phase-4-backlog §3.6.
2. **Promote the canonical-bytes / golden-hex tests to REQUIRED CI checks** (carried, still open) —
   a signed-wire change silently drifted a downstream golden and only a non-required lane caught it.
3. **The Composing pre-work parked with no clock** — `COMPOSING-SPLIT-PROPOSAL.md` §5,
   `COMPOSING-SPIKE1-UIUX-R0.md` forks A–G, `COMPOSING-SPIKE2-RECOVERY-PRESENTATION.md`. The
   recovery seam question is RESOLVED (additive post-freeze, does not gate the tag).
4. **npm dependabot tail** — 21 open alerts, but **18 are npm and only 3 are Rust (all medium:
   `serde_with`, `cmov`, `glib`)**. The 4 critical / 7 high are all npm-side, so the required Rust
   lane is not at risk from them. Correcting an earlier overstatement of mine that implied it was.

---

## 4. Remaining path to the tag

1. **W-SUB residuals** — B8's ordering property is unpinned (moving `scan_dag_cbor` after
   `from_slice` keeps 17/17 green, and the module header calls that ordering "the whole point");
   F-073's re-homing didn't reach the arms (§4.169 exists with real content, but 84 arms still cite
   `phase-3-backlog §7.3.D`); FS-06/07 remain evadable by rename.
2. **W-REC** — ~30 false records including COLLAPSE-P3 (`errors.generated.ts` ships to consumers
   describing a replay defence that was deleted), plus the pre-tag debt from §2 above.
3. **W-WIRE** — 8 changes, EngineView first.
4. **W-MINOR** — the 129-item tail; **verify the 86 unverified before fixing**.
5. **Re-run the falsification sweep** — the only thing that converts "these pins look right" into
   evidence. W1's pins are still uncertified: they pass, and the verifier reasoned structurally
   rather than mutating.
6. **Round #2 at 31 lenses** → then a second consecutive CONVERGED round → pre-tag bundle → **hold
   for Ben**.

---

## 5. Operational notes worth carrying

- **`/tmp` does not survive a reboot.** Six workflow result files evaporated; they were
  recoverable only because the durable journals live under `~/.claude/`. Never cite a `/tmp` path
  as an authoritative destination — that is `feedback_findings_carry_substance_not_bare_ids`
  applied to my own handoff, which I got wrong once already.
- **Before resuming a killed workflow, count `result` records in `journal.jsonl`.** A run showing
  N `started` / 0 `result` (the PANEL-EMPTY abort shape) has nothing to replay. When results do
  exist the economics are excellent — the IVM pass replayed 4 lenses (~1.4M banked) for 874K live.
- **A hand-written fan-out can silently hand its synthesizer nothing.** `wf-value-extensibility.js`
  said "below are 4 lens reports" and never interpolated them; 9 agents ran for nobody. Grep the
  synthesizer prompt for the actual interpolation, not the prose describing it.
- **`git stash push -u` / `pop` silently undid a `git rm --cached`.** Caught only by re-checking
  `git ls-files` before committing.
- Shell is **zsh**: `mapfile` needs `bash -c`; quote `--include=` globs; never mask stderr while
  diagnosing (a masked `0` reads as success).
