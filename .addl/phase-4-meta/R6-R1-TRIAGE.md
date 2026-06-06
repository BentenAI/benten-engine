# R6 Round-1 Triage — Phase-4-Meta-Core phase-close council (2026-06-06)

**Run:** `f-full-r6-council.js` Task `w80kgq54r` / Run `wf_581ced48-84f` (30 agents, 16 lenses, ~46 min). Artifact = main `84280d31`.

**VERDICT: NOT-CONVERGED — 0 BLOCKER · 7 confirmed MAJOR · 1 MAJOR→OBS-downgraded · MINOR/OBS cluster.**
Crypto substance byte-faithful + freeze sound at substrate level (X-Wing combiner label-appended, LAMPS sig wire, K(V)-full-CID, codepoint registry, ML-KEM/libcrux, Inv-19 — all ground-truth construction-correct). The 7 MAJORs are spec-vs-doc / wire-vs-doc divergences + freeze-record staleness; none re-does implementation.

## SURFACE-TO-BEN — 3 genuine freeze-forks (orchestrator must NOT resolve unilaterally)
- **F-01** (group censorship/truncation defense): SECURITY-PROOFS §3.3/§4.1 + THREAT-MODEL claim a delivered-`stanzas.len()` vs bound-`stanza_count` check is DELIVERED on the group open paths (0x6520/0x6610), but the code lacks it. Fork: **WIRE the check + negative test** vs **DELETE the over-claim + mint a named Compromise**.
- **F-02** (canonical 0x6610 group-AAD): two divergent encoders — live emits **6-field/79B**, golden + SECURITY-PROOFS specify **11-field/127B**. Fork: **11-field is truth** (fix the live encoder; `group_posture` must seal through `assemble_group_aad`) vs **6-field is truth** (shrink the `f_aad_2` golden + SECURITY-PROOFS §3.3 + fix `verify_stanza`'s false "AEAD-bound" doc + reconcile §3.3-vs-§4.1).
- **F-06** (wasm32 crypto CI): the tf3a wasm32-wasip1 PQ-hybrid round-trip is present but **never CI-compiled**; V1-WIRE-FORMAT-INVENTORY:82 overclaims it. baked-in #17 = wasm32 first-class. Fork: **add the wasm32-wasip1 crypto CI gate before the tag** vs **carry to v1-GM + retitle the doc honestly**.

## FIX-NOW — pre-tag sweep (doc/comment/test staleness; no forks; reconcile-2 PR)
- **F-03+F-04+F-18** INVARIANT-COVERAGE.md: repin 2172cb6d→84280d31; Inv-16..22 + Inv-19 → as-built+ENFORCED with real cites; Inv-21 row (ordering-enforced / archival-half comparator-named-to-F-INV21-4 / kani #[cfg(kani)]-gated proptest-floor); extend `f_disc_2` with an enforced-state assertion.
- **F-05** V1-FROZEN-INTERFACE.md 14→15 (item 1, item 9 baseline list + add `benten-membership-set.txt`, Verify-mech-#1) + NEW §16 MembershipSet freeze section (lib.rs pub-use set; 0x6600/0x6610/0x6620 band; EXACTLY-3-Kind / RoleId-5 / one-DID-one-entry carve-outs w/ f_ms_* cites) + ARCHITECTURE.md fourteen→fifteen + crate-count test pin →15. *(freeze-section documents the AS-BUILT shipped surface — flag for Ben/orch eyeball.)*
- **F-07** ERROR-CATALOG.md preamble 192/194 → **199** + V1-FROZEN item 10 + Verify-mech-#5 + TS cross-check.
- **F-08** INTERNALS.md:65/:97 AAD 2-tuple → 4-segment binding total_chunks; drop the G-COMP-1/Row-D-15a defer clause.
- **F-09** aad.rs "little-endian"→"big-endian (M-19)" (aad_per_chunk + aad_per_recipe).
- **F-10** cipher_suite.rs:329 call-site comment HKDF-SHA256→SHA3-256.
- **F-15** exemptions.toml date reconcile to one coherent date (06-05 vs 06-06).
- **F-16** cite-drift cluster (INTERNALS + V1-FROZEN: from_codepoint→from_raw+resolve; ::run; line drifts).
- **F-17** CRYPTO-CODEPOINTS.md:60 cite MEMBERSHIP_SET_SUBSET_REF → MEMBERSHIP_SET_RESERVED_0X6620.
- **F-19** strip/retense red-phase boilerplate doc-comments on landed+green tests.

## NAMED-CARRY — HARD-RULE clause-b destinations (land entries NOW; don't block)
F-11→V1-FROZEN-INTERFACE-BUILD-BACKLOG (registry equality-pin) · F-12→CRYPTO-CODEPOINTS §4.0 (0x6380 3-way reserve discrete obligation) · F-13→V1-FROZEN-INTERFACE-DEFERRED G-COMP-1 (SessionIdReplayed production site) · F-14→SECURITY-POSTURE test-debt (f_audit_1 arm-(a) model-shape note) · F-20→crypto-suite backlog (naming cluster; ek_mlkem rename touches frozen field → freeze-lens+Ben) · F-21→per-item (V1-WIRE-FORMAT-INVENTORY 0x6620-encode-only / benten-drop INTERNALS / SECURITY-POSTURE #36/#39); **F-21(b)/(c)=DISAGREE-fix-now** (no reachable live caller; post-v1-beta hardening).

## MISSED-LENS for round 2
Add **`as-built-doc-reconciliation`** (planned-vs-shipped-prose-currency) — diff each doc's narrative of *what is built+enforced* against the artifact at HEAD. F-03 slipped structural gates because every cite resolves + every ruling is obeyed, yet the prose said the substrate "doesn't exist." (per memory `feedback_full_phase_review_every_round`.) F-03's `f_disc_2` enforced-state assertion is the structural-gate complement.

## Sequence
SURFACE F-01/F-02/F-06 → Ben decisions → reconcile-2 fix wave (FIX-NOW + NAMED-CARRY + the 3 fork-fixes) → re-run R6 round 2 (add the missed-lens) → iterate to 0 BLK/MAJ → pre-tag → tag (HOLD Ben).
