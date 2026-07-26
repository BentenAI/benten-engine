# R6 lean-round lens analysis — which lenses have earned their keep

**Written 2026-07-26** for the round-#2 (lean confirming) composition Ben asked for. Source = per-finding `lens` attribution in the three persisted council triages: `r6-r1c-council-triage.json` (b86dec03), `r6-final-council-triage.json` (31c786e4), `r6-reround-council-triage.json` (d4d9db5a).

**Caveat on the data (be honest about it):** lens keys are NOT normalized across rounds — some rounds record a name (`secret-lifetime-zeroize`), others a number-plus-label (`3 (wire-freeze/little-endian)`), and some findings carry two lenses. So a mechanical per-lens tally under-counts any lens that changed key between rounds. The FAMILY-level read below is sound; a per-key ranking is not.

## Family A — PRODUCTIVE (kept in any lean round)

Every confirmed MAJOR across the last three rounds came from this set.

| Family | Yield across 3 rounds | The catch that earns it |
|---|---|---|
| secret-lifetime / memory-hygiene | **2 MAJOR** + 3 MINOR | the vault transient-plaintext + layer_c CEK zeroize MAJORs |
| wire-freeze byte-correctness (incl. endianness) | **1 MAJOR** | F-01 `to_le_bytes` on the SIGNED binding_message |
| frozen-api-misuse-resistance / crypto-misuse | **1 MAJOR** + 2 MINOR | F-02 the repeating-key-XOR `encrypt_node` doc-claiming to seal |
| public-surface-semver-freeze | **1 MAJOR** + 3 MINOR + 5 OBS | ungated `_for_test` symbols frozen onto baselines |
| codepoint + domain-tag registry integrity | **1 MAJOR** + 1 MINOR | the unenrolled production AAD tags (F-06, sm-aad) |
| at-rest format migration / forward-compat | **1 MAJOR** + 1 MINOR + 4 OBS | the LE-vs-BE frozen-format docstring |
| deferral-honesty (HARD-RULE-12) | **1 MAJOR** + 2 MINOR | disclosure rows claiming OPEN for already-closed work |
| metadata-privacy / blinding | **1 MAJOR** + 2 OBS | the unsalted roster commitment over-claim |
| cite-drift / doc-coupling mechanical integrity | 6 MINOR | stale counts + stale SHAs across tracked docs |
| ucan-attenuation | 2 MINOR + 4 OBS | the false "no production caller" caveat |
| audit-governance + authorization-enforcement semantics | 4 MINOR + 4 OBS | fns whose names assert enforcement they do not perform |
| pattern-induction (unnamed cross-cutting) | 2 MINOR + 2 OBS | the meta-lens; cheap, catches what no named lens owns |
| as-built-doc reconciliation | 1 MINOR + 2 OBS | prose-vs-code currency |

## Family B — PERENNIALLY CLEAN (candidates to drop in the lean round)

Zero BLOCKER, zero MAJOR, ~zero MINOR across all three rounds; their output is OBS-grade "hardening ideas" on code that has been **byte-stable since R8**. These verify *construction correctness of unchanged crypto* — genuinely settled.

`crypto-construction-correctness` · `invariant-semantics-inv15-22` · `sender-origin-authentication-b2` · `did-benten-codec-and-keyset-fidelity` · `recipient-binding-anti-downgrade-inv23` · `membership-set-primitive-fidelity` · `distributed-sync-crdt-hlc-inv21-mst` · `crypto-error-oracle-failure-mode-uniformity` · `entropy-source-and-deterministic-derivation` · `supply-chain-libcrux-cryspen-vet` · `cross-target-wasm-deployment-shapes` · `convergence-readiness`

## Recommendation (for Ben)

- **Round #1 = FULL.** The fix-passes touched real code; a full panel is the proper closure, and it is the round most likely to catch a fix-introduced regression.
- **Round #2 = LEAN = Family A + a small crypto-sentinel subset of Family B + everything touching changed lines + the completeness-critic** (which is also the lens-gap detector — it is what protects a lean round from false convergence). Roughly half the panel.
- **Do NOT drop `wire-freeze-byte-correctness`** even though a sibling key looked clean — the naming collision hides that this family produced the F-01 MAJOR.
- **Keep the Layer-D protocol lens MANDATORY** in both rounds; the architect has dropped it once already and that cost a round.

**Honest flag:** leaning round #2 relaxes `feedback_full_phase_review_every_round` / `feedback_phase_close_final_council_full` for the confirmation round specifically. The argument is that those rules govern the discovery phase and round #2's job is stability-proof, not discovery. It is still a relaxation of a ratified discipline, and it is Ben's call.
