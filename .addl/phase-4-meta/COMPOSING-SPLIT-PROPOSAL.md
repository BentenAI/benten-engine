# Phase-4-Meta-Composing — Split Proposal + Pre-Work Map

**Status:** PROPOSAL for Ben's ratification — NOT a locked plan. Drafted 2026-07-21 (while `phase-4-meta-core-close` is in its final convergence round). This is the *inverted pre-work* for Composing (consolidate + sequence + decide-the-split of already-named scope), mirroring how Phase-4-Meta-Core's pre-work consolidated the refinement-audit campaign's rows rather than discovering greenfield scope.

**What this doc decides (with Ben):** (1) whether to split Composing, and into what; (2) the sub-phase sequencing + boundaries; (3) the two design/decision spikes that are under-defined and can start early. **What it does NOT do:** freeze anything, or re-open any Core decision. Everything in Composing is *additive on the frozen v1-beta substrate* (Core froze the wire at `phase-4-meta-core-close`).

---

## 0. The situation

Phase-4-Meta-Core is the wire-freeze tier (it terminates by freezing the v1-beta public interface). **Composing is everything that builds ON that frozen substrate**, up to the `v1-beta` tag. Its scope is *enumerated but not consolidated* — pieces live scattered across CLAUDE.md baked-ins, `docs/future/phase-4-backlog.md`, the F-full UX-coupled deferrals, and the named-deferred rows the R6 councils verified (D-52/D-64/D-90/G-COMP-1/G-CORE-PQ-WIRE-1/D-75/D-76 + the v1-assessment-window list).

**The problem:** as one phase, Composing bundles **four genuinely different lens surfaces**. A single R1 spec-council or R6 quality-council can't coherently review crypto-engine-wiring *and* meta-circular-admin-architecture *and* device-pairing-UX *and* a crypto-protocol-choice-plus-cleanup in one pass. That lens-incoherence is exactly split-criterion **(i)** — the same criterion (alongside (ii) crypto-unknown contamination and (iii) later-tier-needs-settled-foundation) that justified the Foundation→Meta and Meta-Core/Meta-Composing splits.

**Honest calibration of the split driver:** unlike Core/Composing (which split on a *hard wire-freeze boundary*, P-III), the fault lines *inside* Composing are **lens-coherence + sequencing only** — every cluster is additive post-freeze, so there is no wire-break gate between them. That makes the split a *right-sizing* question, not a correctness gate. The recommendation below is calibrated accordingly (a moderate 3-way split, with a lighter 2-way and a no-split-staged-waves alternative both spelled out).

---

## 1. The four clusters

### Cluster A — Substrate-wiring completions
*Finish what the freeze deferred, on the frozen substrate. Crypto/engine lens. Mechanical-ish. Risk: LOW–MEDIUM (additive).*

| Item | Source | Note |
|---|---|---|
| **Engine encrypt-to-recipient wiring** | Row **D-64** | The crypto (Drop/Layer-C seal/open) is built + frozen; only the *engine* write/read-path wiring is deferred. Makes the encryption substrate usable end-to-end. Verified NOT-a-Compromise by the R6 councils. |
| **G-CORE-PQ-WIRE-1** | `.addl/phase-4-meta/g-core-pq-wire-r0-plan.md` | Canary already has an R0 plan; scope bundles the Inv-15 audit + hardening (~+400–600 LOC). |
| **Inv-21 production caller** | INVARIANT-COVERAGE register-then-enforce carve-out | `fork_total_order_key`/`fork_a_wins` has zero production callers today (live merge path = benten-sync LWW). Wire a real caller *or* keep the disclosed carve-out. |
| **G-COMP-1** (§8-B MerkleRangeProof trait in `benten-sync`) | phase-4-backlog | Underpins light-client mode-(b) range-query + mode-(c) signed-checkpoint proofs (Phase-3 deferrals). |
| **Zeroize-residual hardening** | Rows **D-75 / D-76** (Compromise #36) | Best-effort secret-buffer zeroize (combiner preimage, vault CBOR, device_link inner_bytes) surfaced as OBS across the R6 councils. |
| **Additive `Argon2idParams::new()->Result`** | Row **D-90** (R6-1c F-10) | The validated constructor; non-breaking; the reachable panic path is already closed at `parse_vault_frame`. Optional here vs v1-GM. |

**Why it goes first:** the D-64 e2r-wiring makes the frozen encryption substrate *actually usable* — Clusters B and C both want to encrypt real data through the engine. A is the unblocker.

### Cluster B — The self-composing meta-circular admin
*The conceptual centerpiece. Admin/UI-architecture lens. NOVEL. Risk: MEDIUM–HIGH.*

| Item | Source | Note |
|---|---|---|
| **Self-composing admin** | CLAUDE.md #18 + FULL-ROADMAP Phase-4-Meta exit criteria | The admin UI is *itself* a graph composition that admins edit *through* the admin UI — built ON the Foundation admin-UI-v0 + the frozen v1 surface. This is the "admins are editing the admin UI in the admin UI" exit criterion. |
| **Full Tauri T3 hardening** | Phase-4-Foundation deferral (CLAUDE.md #17 renderer note) | IPC allowlist + CSP-locked-at-webview-load + per-method manifest cap-binding. Foundation shipped only *minimal* Tauri; the full defenses land here. |

**Why it needs a spike:** the meta-circularity (editing-the-editor: bootstrap, self-reference, consistency-under-self-edit) is genuinely novel and under-designed. → **Spike 1** (below). This is the exit-criterion centerpiece for `phase-4-meta-close`.

### Cluster C — Identity / device UX flows
*The F-full UX-coupled deferrals (~1.3–2.4K LOC). UX + crypto-UX lens. Design-heavy, user-facing. Risk: MEDIUM.*

| Item | Note |
|---|---|
| **Device-link UX** (QR + approval) | The on-device pairing flow *on top of* the frozen `0x631x` device_link wire. |
| **Remote-permission-call UX** | The approve-on-another-device flow on top of the frozen `0x632x` remote_permission wire. |
| **Biometric layer** | Local unlock / approval gating. |
| **Stronghold optional backend** | Alternative secret-store backend (vs the keyring-core default). |
| **Identity-recovery `RecoveryHook` stub trait** | The pluggable seam the Cluster-D protocol decision plugs into. |

**Key property:** the wire shapes are FROZEN in Core; C builds *UX* on them. The flows must not weaken the frozen wire's security properties (the M-6 chosen-recipient surface that the Layer-D lens verified is safe-to-freeze is exactly what these UX flows drive).

### Cluster D — v1-assessment-window (decision + cleanup + validation)
*Mixed lens. The identity-recovery decision is MEDIUM–HIGH risk; the rest LOW. Ends at → `v1-beta`.*

| Item | Source | Note |
|---|---|---|
| **Identity-recovery protocol CHOICE** | Phase-3 deferral (`.addl/phase-3/exploration-device-mesh.md`) | Shamir threshold vs social-recovery (UCAN delegation) vs hardware-key-escrow vs MLS device-groups. **Deliberately deferred for end-user feedback** ("we've been saying Shamir without verifying"). → **Spike 2**. |
| wasmtime Component-Model re-evaluation | phase-4-backlog §3.4 | GA-ready now? re-enable + un-ignore, or re-defer with IFF clause. |
| Engine impl-block generic-cascade lift | phase-4-backlog §3.4 | ~1500–3000 LOC; only if a concrete alternate-backend consumer surfaces. |
| `missing_docs` sweep + small arch cleanups | CLAUDE.md #15 | Monolithic file splits, stale comments. |
| Browser IndexedDB persistence | phase-4-backlog (Compromise #19/#20) | Closes the browser thin-client persistence compromises. |
| Tauri 2.x dep-migration | phase-4-backlog §3.6 | Security-advisory carry; tighten `deny.toml` as upstream closes. |
| Workspace feature-less test ergonomics | phase-4-backlog §3.4 (R6-1c F-15) | Cross-crate `testing`-fixture cfg-refactor so `cargo test -p <crate>` works flag-free. |
| **Comprehensive end-to-end use-test** | CLAUDE.md #15 | Dogfood the whole platform across user-owned hardware — the real v1 gate. |

---

## 2. Proposed sub-phase mapping

### ★ Recommended — 3 sub-phases (A | B | C+D)

1. **Phase-4-Meta-Composing-Substrate** = **Cluster A**. Ships first; makes the frozen substrate usable end-to-end. Coherent crypto/engine lens; light ADDL (mostly R5+R6, since the R0 designs mostly exist — e.g. G-CORE-PQ-WIRE already has one). Tag candidate: `phase-4-meta-composing-substrate-close` (or fold into the next tag if small).
2. **Phase-4-Meta-Composing-Admin** = **Cluster B**. The novel centerpiece; runs **Spike 1** as its R0, then full ADDL. Admin/UI-architecture lens. This is where "admins edit the admin in the admin" becomes true.
3. **Phase-4-Meta-Composing-Surface** = **Cluster C + Cluster D**. UX flows + the v1-assessment-window (incl. **Spike 2**). Shared UX/validation lens; ends at **`phase-4-meta-close`** → then **`v1-beta`**.

**Why this shape:** A is mechanical + unblocks; B is design-heavy + novel + wants its own R0 + council; C and D share a UX/human-in-the-loop/validation lens and both terminate the phase. Each sub-phase gets a *coherent* lens set for its R1/R6 councils — the whole point of the split.

### Alternative (lighter) — 2 sub-phases (A+B | C+D)
- **Composing-Build** = A + B (wire the substrate + build the composable admin).
- **Composing-Surface** = C + D → `phase-4-meta-close` → `v1-beta`.
Cost: the Build council spans mechanical-wiring *and* novel-architecture (mild lens-incoherence). Benefit: one fewer phase boundary.

### Alternative (lightest) — no split, staged waves (A→B→C→D)
Keep one Composing phase; sequence the clusters as waves with one R1 up front and one R6 at close. Cost: the single R1/R6 councils span all four lens surfaces (the incoherence this proposal exists to avoid). Benefit: least ceremony. **Not recommended** — but it's the honest floor if Ben wants to minimize phase overhead.

**My recommendation: the 3-way split**, because A/B/C+D each earn a coherent council, A-first de-risks the substrate early, and B (the hard part) gets undivided design attention. I'd accept the 2-way as a reasonable lighter call.

---

## 3. The two spikes (can start early — not gated on the tag)

Both are *design/decision* work, not implementation, so **neither is blocked on `phase-4-meta-core-close`** — they can run as pre-work whenever Ben wants, even in parallel with the tag's final steps.

### Spike 1 — Meta-circular admin R0 design
**Question:** how does the admin UI, expressed as a graph subgraph, edit *itself* through itself? Concretely: the bootstrap (how the admin subgraph loads + becomes editable), self-reference (an admin composition that references the admin composition), consistency-under-self-edit (editing the editor without bricking it — versioning/rollback of the admin itself via the DAG-shaped anchor pattern), and the Tauri T3 trust boundary (which admin actions cross the IPC boundary + how they're cap-bound). **Produces:** a ratified R0 architecture for the self-composition + the T3 boundary — the design input to the Cluster-B ADDL. **Shape:** 1 planning agent + 2–3 critics (meta-circular-consistency / capability-boundary / UI-composition lenses), then triage → Ben ratification. ~1–2 days pre-work scale.

### Spike 2 — Identity-recovery protocol survey + decision
**Question:** which recovery protocol (or which *pluggable* shape that defers the choice)? Survey **Shamir threshold key recovery** vs **social recovery via UCAN delegation** vs **hardware-key escrow** vs **MLS device-groups**, against Benten's actual model (self-sovereign, P2P, no central authority, the frozen did:benten + RotationLog + device-attestation substrate). **Produces:** a surveyed recommendation + a decision — either a specific protocol, or a `RecoveryHook` trait shape that keeps the choice pluggable and defers the concrete protocol to post-v1 with a documented default. **Why it's under-defined:** Ben deliberately parked it for end-user feedback to sharpen the requirements; the survey itself is research that can run now. **Shape:** 1 research/survey agent (multi-modal: academic protocols + shipped-system precedents like Signal/1Password/Ledger/MLS) + a synthesis → Ben decision. **Note:** the `RecoveryHook` *stub trait* is a Cluster-C item; this spike decides what fills it.

---

## 4. Tag sequence + where the split fits

```
phase-4-meta-core-close  ← NOW (final round in flight; freezes the v1-beta interface)
   │
   ├─ Composing-Substrate (A)   [substrate usable end-to-end]
   ├─ Composing-Admin (B)       [Spike 1 → meta-circular admin + Tauri T3]
   └─ Composing-Surface (C+D)   [UX flows + Spike 2 + v1-assessment]
   │
phase-4-meta-close       ← Composing done
   │
v1-beta                  ← PQ-hybrid default exercised end-to-end; independent ml-dsa/ml-kem audit COMMISSIONED
   │  (independent audit lands during the beta window; C-GM-AUDIT exit criterion)
   │
v1-GM                    ← audit-gated: no unresolved high/critical on the hybrid trust path; pinned==audited; Ben sign-off
```

Then Phases 5–8 (First Reference Application → Personal AI Assistant MVP → Gardens/untrusted-host → decentralized plugin discovery) as committed post-v1 scope.

---

## 5. Open decisions for Ben

> **★ RATIFIED 2026-07-26 (Ben, AskUserQuestion):**
> - **§5.1 SPLIT SHAPE = 3-WAY.** Composing-Substrate (A) → Composing-Admin (B) → Composing-Surface (C+D). Each sub-phase gets a coherent lens set for its R1/R6 councils. Honest calibration retained from §0: unlike the Core/Composing split (a hard wire-freeze boundary, P-III), these fault lines are **lens-coherence + sequencing only** — every cluster is additive post-freeze, so there is no wire gate between them and the sub-phase boundaries are right-sizing, not correctness gates.
> - **§5.5 FULL-ROADMAP refresh is now UNBLOCKED** — its content was gated on §5.1 and §5.1 is decided. The tracked roadmap still shows a single "Phase 4-Meta" and predates the Core/Composing split entirely; the refresh must fold in the 3-way Composing split **and** the `phase-4-meta-core-close → phase-4-meta-close → v1-beta → v1-GM` sequence with the C-GM-AUDIT gate. Scheduled to ride the pre-review doc-coupling closure sweep.
>
> **STILL OPEN (orchestrator recommendation noted; not yet explicitly ratified):** §5.2 fold Composing-Substrate into the Admin tag rather than giving A its own · §5.3 run the confidentiality/DAK-ratcheting spike + the two UI R1 spikes (taxonomy stress-test, CSP-strict interpreter) early, since they are cheap and de-risk load-bearing unknowns before R1 locks scope · §5.4 take all three Cluster-A membership calls (Inv-21 production-caller wiring, D-90 Argon2id constructor, G-COMP-1 light-client mode-(b)/(c) scope) into Cluster A.

### Original open-decision list (retained)

1. **Split shape:** 3-way (recommended) / 2-way / no-split-staged-waves.
2. **Sub-phase tag granularity:** does Composing-Substrate (A) warrant its own tag, or fold into the Admin tag? (A is small-ish; a fold is reasonable.)
3. **Spike timing:** run Spike 1 (meta-circular admin R0) and/or Spike 2 (identity-recovery survey) *now/early* as pre-work, or wait until Composing opens? (Both can start immediately; Spike 2's survey especially benefits from lead time.)
4. **Cluster-A membership calls:** Inv-21 production-caller (wire vs keep-carve-out); D-90 Argon2id constructor (here vs v1-GM); G-COMP-1 light-client mode-(b)/(c) scope.
5. **`FULL-ROADMAP.md` refresh:** the tracked roadmap predates the Core/Composing split and still shows a single "Phase 4-Meta" — fold this split + the v1-beta/v1-GM sequence in at the pre-tag doc sweep.

---

## 6. Provenance / references

- Split precedent + criteria (i)/(ii)/(iii): CLAUDE.md status-table "Phase-4-Meta" row (2026-05-18 split ratification).
- Cluster scope sources: CLAUDE.md baked-in #15 (v1-gate) / #17 (deployment shapes + Tauri) / #18 (self-composing admin); `docs/future/phase-4-backlog.md` §3.4 + §3.6; the F-full UX-coupled deferrals (CLAUDE.md night-shift 2026-05-27 wave-sequencing block); `.addl/phase-4-meta/g-core-pq-wire-r0-plan.md`; the R6 council named-deferrals (D-52/D-64/D-90/G-COMP-1/G-CORE-PQ-WIRE-1/D-75/D-76) in `.addl/phase-4-meta/r6-r1c-council-triage.json`.
- Identity-recovery deferral rationale: `.addl/phase-3/exploration-device-mesh.md` (2026-05-04); FULL-ROADMAP Phase-3 identity-recovery note (line 77).
- Tag sequence: CLAUDE.md baked-in #15 (v1-beta → v1-GM, C-GM-AUDIT gate) + the PQ-default reframe (`.addl/pq-research/RATIFIED-pq-default-reframe-2026-05-19.md`).
