# F-full Layer-D NQ-cluster — spec report + ratification (pending Ben)

Spec agent `a46b2274e63680bfe` (2026-06-02 NIGHT). **Overarching finding: the R4 "premature-pin" concern does NOT hold for this cluster** — all 4 corpus tests are exemplary RED-PHASE stubs that carry explicit `## NQ-* OPEN-SPEC FLAG` doc-comments and pin only the R0-stated default behind a single `_nq_*_gated` arm (per R2 §5.B). The corpus parked them correctly. So the disposition is **ratify the R0 defaults → R5 deletes the `_nq_*_gated` suffix + un-ignores**, NOT "downgrade to shape-only."

**Agent recommendation: SPEC-NOW on all 4** (each is self-contained; the R0 default is the correct permanent shape; keeping them open needlessly blocks R5 from un-ignoring 4 freeze-gating arms).

| NQ | Freeze-gating? | Disposition | Ratifiable rule |
|---|---|---|---|
| **NQ-T2** | YES (wire field + enforcement) | SPEC-NOW | 1-hr bucket (privacy, round-down) ⊥ `valid_until` (full-1s enforcement); `valid_until` enforced **STRICTLY** (`now > valid_until → reject`, NO grace/skew window — any ε re-opens the coercion/replay window NQ-T2 exists to close); bucket never consulted for expiry. Corpus arm (60s `valid_until` + 90s present → reject) is correct; R5 adds a positive-control (+30s → admit). |
| **NQ-T3** | YES (AAD scope only; runtime enforcement deferred, NOT freeze-gating) | SPEC-NOW | The frozen 3-field `ExecuteWorkflow` AAD `(executor_did, max_decrypt_count, result_recipient_pubkey)` is **SUFFICIENT** to express the no-egress/bounded-decrypt constraint; no later wire field needed → slot may freeze. Runtime enforcement stays post-v1-beta (§10.2). R5 sharpens the arm to enumeration-completeness (these 3 == the full constraint param set). |
| **NQ-T4** | PARTIAL (retention/durability freeze-locked, already pinned in R0) | SPEC-NOW + disclose | jti-keyed, durable (survives restart), retention ≥ full 1-hr bucket; scope = per-device-durable **guaranteed** + user-global **best-effort-eventual-via-sync** (NOT synchronous — that needs per-grant consensus, out of scope); nonce-keyed not time-keyed. The pre-sync cross-device replay window is the one genuinely-soft residual → **disclose as a named Compromise** (best-effort-eventual IS the permanent answer, not a placeholder), NOT keep-open. |
| **NQ-C5** | YES (permanent bucket bytes) | SPEC-NOW | bucket = `(raw/3600)*3600`, deterministic, `% 3600 == 0`, round-DOWN, **NO jitter** (avoids ≤2×jitter+skew widening); nonce-cache window does **NOT** need widening (orthogonal — bucket=privacy, nonce-cache=replay). |

**Cross-cutting:** the `≥1-year grace` at R0 §3.3 is recipient-side drop key-retention (#62), NOT a `valid_until` grace — do not conflate. On ratification, R5 deletes the 4 `_nq_*_gated` suffixes + un-ignores; the NQ-T4 pre-sync window gets a one-line SECURITY-POSTURE disclosure at the doc-wave.

**STATUS: pending Ben ratification.** If ratified, these 4 leave the R4-fix scope (they're correctly-parked, not defects) — only the small sharpenings (NQ-T2 positive-control, NQ-T3 enumeration-completeness, NQ-T4 Compromise mint) carry forward, mostly R5-fill.
