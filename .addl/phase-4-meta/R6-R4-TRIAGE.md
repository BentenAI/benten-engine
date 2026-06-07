# R6 Round-4 Triage — Phase-4-Meta-Core phase-close (artifact main b93b2efc)

Run `wf_d7a07941-6c5` (Task w9tytv6ju, 27 agents / 17 lenses incl sender-origin-auth). **VERDICT: NOT-CONVERGED — 0 BLOCKER · 2 CONFIRMED MAJOR · ~12 MINOR · ~12 OBS.** Freeze SUBSTRATE re-verified byte-correct; **B2 sender-origin-auth HOLDS** (f_lc_3 arms substantive, fail-closed); §11 non_exhaustive class confirmed CLOSED. Down from round-3's 7 MAJOR. The 2 MAJOR are 1 freeze-fork + 1 doc-cite.

## ⛔ C-01 (MAJOR) — BEN FREEZE-FORK
`crates/benten-engine/src/layer_d/device_link.rs::provisioning_signing_bytes` is the LONE member of a 6-surface same-key (user-DID Ed25519) signature family that OMITS a per-surface domain-separation prefix. The 5 siblings all carry an explicit domain tag (`ENVELOPE_SIG_DOMAIN`, `SENDER_AUTH_DOMAIN`, `REQUEST_DOMAIN`/`GRANT_DOMAIN`, device-attestation DAG-CBOR map). Missing the prefix = cross-context signature-confusion surface (same key signs provisioning bytes + 5 other contexts; a provisioning sig could be re-interpreted in another context). **It's on the FROZEN public API (benten-engine.txt:1259), so the fix mutates a frozen signed-bytes wire = Ben freeze-fork.** **Fix = add `PROVISIONING_DOMAIN` prefix** (cheap PRE-freeze; best landed via the C-02 central domain-tag registry shape). **ORCH REC: FIX IT** — real gap, free now (tag not landed), makes the family consistent, closes the confusion class.

## C-05 (MAJOR) — doc-only, mechanical (no fork)
Three test-file cites name the wrong crate prefix at the freeze ref (files exist, not at cited path): SECURITY-POSTURE.md:1883 (`crates/benten-engine/tests/integration/atrium_two_device.rs` → workspace-root `tests/integration/...`), INVARIANT-COVERAGE.md:42, V1-WIRE-FORMAT-INVENTORY.md:530. Fix-now mechanical. Consider extending cite-drift-detector to validate bare full-path `.rs` cites (the blind-spot class).

## MINOR — fix-now (round-5 wave)
C-06 (`is_unlocked()` always-false `&self` — document the contract OR &mut; **borderline Ben &self-tension**) · C-10 ("9-codepoint"→"11" V1-FROZEN:446) · C-11 (route 0x647c AEAD through aead::wrap or note dup) · C-14 (codepoint.rs:64→:82 cite) · C-15 (phantom `c4a37bb` SHA) · C-16 (self-HEAD `fdfda621`→`b93b2efc`, 8 occ) · C-17 (row-19 "doc-tests"→`f_inv19_1_*.rs`) · C-18 (f_disc_2 retense).

## belongs-named-now / OBS (doc rows)
C-02 (central domain-tag registry + prefix-free cross-surface test + THREAT-MODEL row — pairs with C-01) · C-03 (widen M-19 WIRE_PATH_SOURCES scanner to layer_c/aad/layer_d — premise "only 4 files" was overstated, live LE-survivor=0) · C-04 (ExecuteWorkflow `input_node_cids` bound into no signed/AEAD surface — add to constraint_aad OR named NQ-T3 deferral; reserved/typed-reject at v1-beta so no live exploit) · C-07/08/09/12/13/19/20/22/23..26 (wire-asymmetry freeze-note / golden-pin carries / regression-guard strengthenings / doc-staleness / gossip §3.9 FLAGGED-FOR-BEN residue).

## BEN-GATED (pre-tag bundle, unchanged + new)
5→18 cargo-vet ratification · §16 §1.A.FROZEN (C-21 confirms §16 correctly flags it) · gossip §3.9-vs-§3.10 (C-26 FLAGGED-FOR-BEN rustdoc residue) · multicodec supersession · DropContentMode §11-table · **NEW: C-01 PROVISIONING_DOMAIN freeze-fork** · C-06 is_unlocked &self-tension (minor).

## Sequence
SURFACE C-01 → Ben ratify → round-5 fix wave (C-01 per Ben + C-05 + fix-now cites C-10/14/15/16/17/18 + C-06 + C-02/03/04 doc + named-carries) → re-run R6 round 5 (re-point b93b2efc→new main) → iterate to 0 BLK/MAJ → pre-tag bundle → tag (HOLD Ben).
