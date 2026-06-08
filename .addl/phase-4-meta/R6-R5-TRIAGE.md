# R6 Round-5 Triage — Phase-4-Meta-Core phase-close (artifact main 6d340944)

Run `wf_aba7028f-898` (Task wcqkuicvf, 30 agents / 19 lenses). **VERDICT: NOT-CONVERGED — 0 BLOCKER · 2 CONFIRMED MAJOR (F-01, F-03) · F-02 REFUTED · MINOR/OBS cluster.** Byte-state re-verified CORRECT; B2 holds; §11 closed; supply-chain/core-crypto/confidentiality/semver all affirmatively SOUND (F-40..44). The 2 MAJOR are **doc-vs-code OVER-CLAIMS** on frozen security/invariant docs — both **BEN FREEZE-FORKS** (widen-the-code vs narrow-the-doc).

## ⛔ F-01 (MAJOR) — BEN FREEZE-FORK: domain-tag registry scope
The C-01 central registry (`domain_registry.rs`) holds **8 signature-family tags**. But `SECURITY-PROOFS.md §4.1` + `THREAT-MODEL.md §5` + the `domain_registry.rs` docstring **over-claim** it spans the CORPUS-WIDE keying surfaces (Layer-C single/group CEK derivations, chunked-AEAD info strings `benten-aead:{whole,chunk,recipe}:`, the §3.9 gossip-topic, the K(V)/K(N) keying-glue — ~24 live prod tags). The §4.1/§5 "a cek-v2 prefix change would forward-fire the regression test" claim is FALSE (registry doesn't cover CEK/AEAD/gossip/K(V)). **No live prefix collision** (the 1 test hit is scaffold) → MAJOR not BLOCKER. (GAP-1 dedups into this.)
- **(a) WIDEN** `registered_domain_tags()` to all ~24 corpus tags (CEK/AEAD/gossip/K(V)/K(N)) → makes §4.1/§5 TRUE + forward-fires on ANY domain-tag change = a strong permanent prefix-free invariant across the whole corpus.
- **(b) NARROW** the docs to accurately scope the registry to the same-key SIGNATURE family (the real cross-context-confusion risk; the CEK/AEAD/gossip/K(V) surfaces are domain-separated by KEY, noted separately).
- ORCH note: the signature family is the genuine same-key collision risk (C-01's whole point); the other surfaces are key-separated. (b) is accurate+minimal; (a) is the stronger forward-proof permanent shape the docs already describe.

## ⛔ F-03 (MAJOR) — BEN FREEZE-FORK: Inv-21 enforcement honesty
`INVARIANT-COVERAGE.md` classifies Inv-21 fork-tie-break as "AS-BUILT + ENFORCED / no observable bypass". Ground-truth: the comparator `fork_a_wins`/`fork_total_order_key` (set.rs:194-216) has **ZERO production callers**; `resolve_fork` is labeled "PRODUCTION-stand-in / R5 routes through the real merge" (future-tense); the LIVE merge path is benten-sync LWW (opposite direction). So the comparator is built + property-pinned but NOT wired into production merge → the doc's own "active means owning crate consumes it on every relevant path" criterion is unmet.
- **(a) WIRE** the Inv-21 fork-tie-break into the production merge path (make it genuinely enforced) — real distributed-merge integration; the LWW-vs-fork-tie-break reconciliation is non-trivial (likely Phase-4-Meta-Composing scope).
- **(b) DOWN-CLASSIFY** the doc to the honest "register-then-enforce" disclosure the doc ALREADY uses for Inv-15 ("comparator AS-BUILT + property-pinned; production merge-path wiring on the [Composing/G-CORE-PQ-WIRE-1] path").
- ORCH note: comparator built+tested; production-wiring is a genuine distributed-merge concern fitting the Composing phase; Inv-15 precedent for honest register-then-enforce disclosure. (b) is accurate + avoids Composing-scope-creep into Core.

## F-02 — REFUTED (no action freeze-gating; optional hygiene)
crypto-suite has unconditional redb/tempfile + ungated RedbSigHandle, BUT crypto-suite is **native-only by dep-graph exclusion** (engine deps it under `cfg(not(wasm32))`; napi never deps it) → no live wasm leak. OPTIONAL defense-in-depth (cheap, matches engine pattern): gate RedbSigHandle + move redb/tempfile to `[target.'cfg(not(wasm32))'.dependencies]`. Non-blocking.

## MINOR/OBS (round-6 fix wave / named-carry)
F-04 (Inv-16 dead canonical_tlv_encode/strict_decode — doc cites the LIVE aead.rs path so no over-claim; OBS hygiene) · F-05 (ExecuteWorkflow constraint-AAD seal-time-bound not in signing_bytes; NQ-T3-ratified sufficient → MINOR doc-tense) · F-06 (§3.5g ctorRx omits `)` combinator shape — 3 in-crate variants skip the scanner; forward scanner-soundness gap, MINOR §3.5g-drain) · F-07..F-39 MINOR/OBS named-carry (doc-tense/cite-currency/cosmetic, all byte-correct). F-40..44 = affirmative SOUND (no action).

## Sequence
SURFACE F-01 + F-03 → Ben picks (a)/(b) each → round-6 fix wave (F-01 + F-03 per Ben + F-02 hygiene + F-04/05/06 + named-carries) → re-run R6 round 6 (re-point 6d340944→new main) → iterate to 0 BLK/MAJ → pre-tag bundle [5→18 · §16 · gossip · multicodec · DropContentMode] → tag (HOLD Ben).
