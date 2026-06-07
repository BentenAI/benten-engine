# R6 Round-3 Triage — Phase-4-Meta-Core phase-close (artifact main fdfda621)

Run `wf_43569c29-441` (Task wfrydpsud, 30 agents / 19 lenses + sender-origin-auth). **VERDICT: NOT-CONVERGED — 0 BLOCKER · 7 CONFIRMED MAJOR · ~16 MINOR · ~44 OBS.** Freeze SUBSTRATE re-verified BYTE-CORRECT (X-Wing label-appended, LAMPS M', K(V) full-36B-CID, codepoints, M_auth, 11-field group AAD). **B2 sender-origin-auth HOLDS** — the new adversarial lens found NO spoofing hole; verify_m_auth fail-closed, F-2 recompute confirmed on all 3 paths. Every MAJOR is freeze-hygiene or doc-divergence (additively fixable). **NO Ben-level forks.**

## Round-4 fix wave — 3 file-disjoint shards (base = main fdfda621)

### Shard A — crypto-suite + drop code
- **F-03 (MAJOR)** `crypto-suite/src/codepoint.rs:389` `pub fn chained_state_tlv_aad_binding` frozen with ZERO src callers → DELETE it (preferred; reduces frozen surface) UNLESS it is an intentionally-reserved API (then register a reserved-API row + add an executing test). Couples F-13.
- **F-13 (MINOR)** `crypto-suite/tests/f_nqa1_1_frozen_surface*.rs` reserve-surface pins are NAME-only greps → add behavioral arms (`resolve().is_err()` + byte-assert).
- **F-07 (MINOR)** `crypto-suite/src/vault.rs:352` `structural_kdf_root_key()` ungated → exact-width `from_root_bytes` seam (aead.rs mr-major-2 twin).
- **F-08 (MINOR)** `crypto-suite/src/cipher_suite.rs:290-308` zeroize the X-Wing `ss_mlkem` (Vec) + combiner KEK output.
- **F-12 (MINOR)** `benten-drop/src/layer_c.rs` (open_inner ~L743) post-decrypt length-prefix uses unchecked usize add on a u32 → `checked_add` (wasm32 co-recipient DoS).
- Rebaseline crypto-suite public-api if F-03 deletes a pub fn.

### Shard B — engine layer_d + membership-set code (COMPREHENSIVE non_exhaustive sweep)
- **F-01 (MAJOR)** `membership-set/src/kind.rs` `KindDispatchError` missing `#[non_exhaustive]` (5 siblings carry it) → add + audit arm + rebaseline membership-set.txt.
- **F-02 (MAJOR)** `engine/src/layer_d/{device_auth.rs:44 DeviceAuthError, device_link.rs DeviceLinkError, secret_store.rs SecretStoreError}` → add `#[non_exhaustive]` + add a `g_core_9_non_exhaustive_audit_engine.rs` (or extend an existing engine audit) + rebaseline engine.txt.
- **F-14 (MINOR)** `engine/src/layer_d/remote_permission.rs:8` frozen pub wire-operation enum → add `#[non_exhaustive]` (or §11 carve-out registry if genuinely frozen-cardinality — verify).
- **F-15 (MINOR)** `engine/src/layer_d/grant_acceptance.rs:31` frozen pub enum → add `#[non_exhaustive]` (or carve-out).
- **F-09 (MINOR)** `engine/src/layer_d/device_link.rs:64-67` `ProvisioningInnerPayload` derives plain `Debug` over pub `k_principal:[u8;32]` → manual redacted Debug (no secret bytes).
- **COMPREHENSIVE:** grep ALL pub error enums in engine/layer_d + membership-set; any frozen pub error enum lacking `#[non_exhaustive]` (excluding repr(u8) wire enums [F-58 confirms correct as-is] + frozen-cardinality Kind/RoleId) gets it — close the §11 class so round 5 finds no more.

### Shard C — doc-reconciliation + OBS disposition (DOC only; light gates)
- **F-04 (MAJOR)** `THREAT-MODEL.md` + `SECURITY-POSTURE.md` — add a POSITIVE row/note: co-member CANNOT forge sender origin (B2 #1366 inter-member non-forgeability now enforced) + §4.1 xref.
- **GAP-A (MAJOR)** `V1-FROZEN-INTERFACE.md` item 8 (:780) + item-11 table (:1024) CapWriteContext/ReadContext → re-tense DEFERRED/TBD → LANDED (DEFERRED.md Row D-17 already CLOSED).
- **GAP-B (MAJOR)** `V1-WIRE-FORMAT-INVENTORY.md` — add the vault on-disk row (0x6100/0x6101, Argon2id freeze, XNONCE) citing `f_va_1`.
- **GAP-C (MAJOR)** `CRYPTO-CODEPOINTS.md` — add symbol-bound rows: vault 0x6100/0x6101, lifecycle 0x6700, split the coarse 0x6380..0x63CF band into CGKA/Bird-of-Prey/Prabel symbols.
- **Cheap doc/cite fixes (fix inline):** F-05/F-06 (V1-WIRE-INVENTORY cross-ref D-30/G-COMP-1 + BE-scanner note) · F-18/F-19 (CRYPTO-CODEPOINTS SSOT note accuracy) · F-21/F-22/F-25 (INTERNALS stale cites) · F-23/F-24 (FROZEN-INTERFACE stale line-numbers 403→410 + §15.f parentheticals) · F-33/F-67 (snapshot-SHA 84280d31→fdfda621 currency) · F-36/F-55 (SECURITY-PROOFS §4.1 f_lc_3 cite + .addl gitignored-ref) · F-47 (Cryspen "13"→precise) · F-48 (5→18 pending — consistent) · GAP-F (regen-determinism doc).
- **NAME (HARD-RULE clause-b backlog, do NOT fix — OBS/disclosure/v1-GM):** GAP-D/GAP-E + the code-behavior OBS (F-26/F-39/F-40/F-41/F-43/F-44/F-46/F-60/F-61/F-62/F-63/F-64) + cite/comment OBS (F-10/F-11/F-16/F-17/F-20/F-27/F-29/F-31/F-32/F-34/F-35/F-37/F-38/F-42/F-45/F-49/F-50/F-51/F-52/F-53/F-54/F-56/F-57/F-59/F-65/F-66) — land each as a one-line DEFERRED/OBS row with destination. F-58 = NO-ACTION (repr(u8) carve-out correct).

## Sequence
3 shards → integrate (disjoint) → mini-review code (A,B) → reconcile PR → Ben squash → re-run R6 ROUND 4 (re-point fdfda621→new main; priorMissedLenses = any round-3 missed-lens) → iterate to 0 BLK/MAJ → pre-tag Ben bundle [5→18 ratification · §16 §1.A.FROZEN · gossip §3.9/3.10 · multicodec · DropContentMode §11-table] → tag (HOLD Ben).
