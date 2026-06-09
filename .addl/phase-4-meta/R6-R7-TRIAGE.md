# R6 Round-7 Triage — Phase-4-Meta-Core phase-close (artifact main 81924331)

Run `wf_cf67e2b3-d38` (Task wfuj9ec3a, 29 agents / 19 lenses). **VERDICT: NOT-CONVERGED — 1 BLOCKER (CONF-1) · 4 MAJOR (3 do-it-right + 1 Ben fork) · 2 MISSED-LENSES for round 8 · MINOR/OBS.** The content-splice fix verified sound; the deeper confidentiality lens found a NEW nonce-reuse BLOCKER on 0x6610.

## ⛔ CONF-1 (BLOCKER) — 0x6610 message-invariant CEK (do-it-right, NO Ben fork)
`group_posture::seal_membership_set_group` derives the 0x6610 CEK = `BLAKE3(MEMBERSHIP_GROUP_CEK_CONTEXT ‖ K_Set ‖ sender_did)` — mixes NOTHING per-message (no body/cid/generation/counter), though `body_digest`/`cid` are in scope 2 lines earlier. For a fixed sender + K_Set generation, EVERY message seals under a byte-identical CEK; bulk+per-stanza AEAD use a random 96-bit ChaCha20-Poly1305 nonce → birthday wall ~2^48 seals (accel 1+N/msg); a nonce collision under the reused key is catastrophic (keystream reuse + Poly1305 key recovery). 0x6520 mixes body_cid+recipient_key_generation; 0x6510 mixes aad+body — only 0x6610 is invariant (asymmetry; orchestrator-reproduced lines 2087-2092 + aead.rs:333). **FIX (no Ben fork):** mix the per-message `cid` (already computed line 2083) into the 0x6610 CEK derivation at BOTH seal + open, matching the 0x6520 pattern — recipient re-derives from inputs it already holds (K_Set + cid); NOT a wire-format change (CEK is internal). **→ SHARD A DISPATCHED.**

## ⛔ drop-placeholder-wrap-frozen (MAJOR) — BEN FREEZE FORK
`benten-drop/src/payload.rs::seal_drop_over_subtree` is a PUBLIC frozen-baseline export whose `recipient_wrapped_cek = BLAKE3("benten-drop:bundle-cek-wrap" ‖ recipient_pk ‖ spec_cid)` — a hash over PUBLIC-ONLY inputs, NOT an HPKE/X-Wing wrap → ZERO CEK confidentiality (any party recomputes it). Docstring admits "the real wrap is the X-Wing HPKE key-wrap"; the substantive path is `layer_c::seal_sealed_sender`. Only consumer = one test (header wrongly calls it "the REAL production path"); undisclosed in any freeze/posture/DEFERRED doc. **OPTIONS (Ben):** (a) cfg-gate `seal_drop_over_subtree`+`DropBundlePayload` behind `any(test, feature="testing")` [zero prod callers = test-scaffold; ORCH REC — removes a zero-confidentiality footgun from the frozen surface] · (b) re-point the public fn to delegate to the real layer_c HPKE wrap before freeze · (c) keep public + DEFERRED-ledger row + SECURITY-POSTURE disclosure it is a non-confidential scaffold.

## MAJOR — do-it-right (shard B, no Ben fork)
- **psf-1** the only 2 tests asserting the canonical-12 PrimitiveKind set are BOTH `#[ignore]`'d RED-PHASE stubs (`unimplemented!()`) → ZERO live backstop for CLAUDE.md #1 irreducibility at FREEZE (pim-12 §3.6e lapsed un-ignore). FIX: un-ignore + wire one stub to the real enum-body walk + canonical-12 assert.
- **psf-2** TS parity glob `dist/*.d.ts` is top-level-only → misses `dist/internal/trace.d.ts` (publicly exported via package.json `./internal/trace`); a sig change there wouldn't trip the required parity gate. FIX: glob → `dist/**/*.d.ts`, regen baseline 14→15, fix the "all 15" doc claim (V1-FROZEN :80/:919).
- **cd-1** `SECURITY-PROOFS.md:41+:67` cite a phantom `EnvelopePayload::HpkeMultiBase` (real: `benten_drop::layer_c::EncryptedEnvelope::HpkeMultiBase`; :41 should be the 0x6610 `assemble_group_aad` identifier). FIX: mechanical doc.

## MINOR/OBS + process
- Doc fix-now: cd-2 (phantom refinement-audit cite) · cd-3 (host-functions.toml→root) · as-built-gossip-topic (§4.1 registered-tag overstatement) · PIC-2 (registry taxonomy comment) · sc-1 (cargo-vet.yml stale "budget=5").
- Named-carry cluster: LD-AUTH-1, D30-LINE-DRIFT, psf-3/4, WFB-OBS-1, xtw-1, RGC-TRANS1, CONF-2/gap-osp-1/2, F-LC3 hygiene, §16-FLAG + HEAD re-pin.
- **⚠️ CLUSTER-ROW-OPACITY (D-39/D-40/D-50/D-53/D-54):** per-ID triage MUST be walked BEFORE the tag (the bundled cluster rows are opaque) — fold into the pre-tag Ben sweep.

## ROUND 8 — ADD 2 MISSED-LENSES
1. **crypto-error-oracle / failure-mode-uniformity** — do distinguishable typed-rejects (ClassicalHalfVerifyFailed/PqHalfVerifyFailed/SenderOriginAuthFailed/AEAD-open) form an adaptive-query distinguisher? napi/wire boundary uniform?
2. **at-rest-format-migration / forever-decode-of-prior-schema** — decode of OLD vault/DropBundle/SnapshotBlob schema versions (item-14 never-strand-content) + downgrade-by-schema-byte-rewrite rejection.

## Sequence
Shard A (CONF-1 BLOCKER) DISPATCHED. SURFACE drop-placeholder fork → Ben → shard B (drop-placeholder per Ben + psf-1/psf-2/cd-1 + MINOR doc + named-carry). Integrate A+B → mini-review CONF-1 CEK + drop-placeholder → reconcile PR → Ben squash → R6 ROUND 8 (re-point + ADD the 2 missed-lenses + the CLUSTER-ROW per-ID walk) → iterate to 0 BLK/MAJ → pre-tag bundle → tag (HOLD Ben).
