# F4 Design — CRITIC-1 (security / threat-model adversarial) — Round 1

> **Branch:** `phase-4-meta-core/r6-r1-f4-design-critic-1`
> **Base SHA:** `a0b75637` (origin/main).
> **Primary review targets:**
> - `origin/phase-4-meta-core/r6-r1-f4-design-staging:.addl/phase-4-meta/F4-DESIGN-TRIAGED-DRAFT-v1.md` (synthesis)
> - `origin/phase-4-meta-core/r6-r1-f4-design-staging:.addl/phase-4-meta/F4-DESIGN-architectural-purist-PLANNER-A.md`
> - `origin/phase-4-meta-core/r6-r1-f4-design-staging:.addl/phase-4-meta/F4-DESIGN-conservative-minimal-PLANNER-B.md`
> **Persona:** security/threat-model adversarial. Goal: find attacks the design admits, false-honesty claims that would survive F4 landing, and cross-surface composition gaps.
> **HARD RULE 12:** every finding dispositioned OUT-OF-SCOPE / BELONGS-NAMED-NOW / DISAGREE-WITH-EXPLANATION / FIX-NOW. No "defer to later".

---

## Executive summary

**Overall disposition: APPROVE-WITH-FIX-NOW.** The synthesis is materially sound on 5 of the 6 surfaces (S2/S3a/S3b/S3c/S4) and the bundling of D-6 + D-18 with S4 is correct. The pre-decision on the EngineBuilder location (F6 Path-(ii)) is correct.

**However**, the synthesis contains **ONE BLOCKER-class architectural false-premise** that propagates through pre-decision F1 (hybrid S1 scope) and is the foundational rationale the synthesis uses to drop ~11 WRITE entry points from the wire-up cascade:

> **F1 RATIONALE FALSE.** The synthesis claims "engine_crud's direct-user-write path defended by `CapabilityPolicy::pre_write` per Compromise #2 sub-narrative." Verification at `crates/benten-engine/src/engine_crud.rs:47-194`: there is **no `pre_write` trait method** (`policy.rs:354-573`); the trait method is `check_write`. **And `Engine::create_node` / `update_node` / `delete_node` / `create_edge` / `delete_edge` do NOT consult `check_write` AT ALL.** They go directly to `self.backend.transaction(|tx| tx.put_node(...))` or `self.backend.put_edge(...)`. The only production `check_write` call sites at HEAD are: `engine.rs:1413` (apply_atrium_merge per-row), `primitive_host.rs:613` (evaluator WRITE primitive), and `engine_wait.rs:899` (resume protocol). **The direct-user-CRUD path is structurally un-gated by `CapabilityPolicy::check_write` at HEAD.**

This finding makes F1's "engine_crud naturally defended → skip it" rationale **false** and means the synthesis's hybrid scope leaves the **single largest WRITE-admission attack surface in the engine** (Engine::create_node / update_node / delete_node / create_edge / delete_edge — the public napi-exposed CRUD APIs) unprotected by the substrate F4 is wiring. This invalidates pre-decision F1 as written. The engineering action required is either (a) flip F1 to PLANNER-A's full ~13-site cascade, OR (b) keep the hybrid but explicitly disclose the engine_crud gap as a new DEFERRED row + retense Compromise #26 + tighten CLAUDE.md #18 to honestly disclose that the direct CRUD API is gated only by Inv-11 system-zone probe, not by the CapabilityPolicy hook.

The fix-now-actionable findings below name the surgical changes needed.

---

## Section 1 — Per-surface adversarial analysis

### S1 — WriteBoundaryChainValidator at WRITE admission (D-1) — **CLAIMS-DEFEAT-BUT-DOESNT**

**Synthesis decision (F1):** hybrid — wire at `apply_atrium_merge:1413-adjacent` + `delegate_capability` Step 4 (`engine_caps.rs:683`-ish). Skip ~11 other WRITE entry points; engine_crud relies on "existing `CapabilityPolicy::pre_write` per Compromise #2 sub-narrative."

**Attacks defeated:**
- Adversarial peer in atrium merge ingress presenting a chain whose root is not a registered user-DID: ✅ rejected by `validate_chain(...)` outcome consumed at apply_atrium_merge.
- `delegate_capability` Step 4 path crafting a delegation grant whose `source_grant_cid` chain doesn't trace to user-root: ✅ rejected at the delegate-side wire.

**Attacks the synthesis ADMITS but doesn't acknowledge:**
- **(A1, BLOCKER-class)** Direct CRUD attack via the public `Engine::create_node` / `update_node` / `delete_node` / `create_edge` / `delete_edge` APIs (and their napi mirrors `bindings/napi/src/lib.rs::Engine::put_node`). These are pub-API entry points at HEAD; verification:
  - `crates/benten-engine/src/engine_crud.rs:47` `create_node` → `self.backend.transaction(|tx| tx.put_node(node))` — **no `policy.check_write` call**.
  - `engine_crud.rs:153` `update_node` → same pattern.
  - `engine_crud.rs:165` `delete_node` → `self.backend.transaction(|tx| tx.delete_node(cid))`.
  - `engine_crud.rs:175` `create_edge` → `self.backend.put_edge(&edge)` — **no transaction even, no cap-check**.
  - `engine_crud.rs:189` `delete_edge` → same.
  The only gate is Inv-11's system-zone-label probe (lines 58-66) — this only blocks system-zone label writes, not the general capability-policy contract. An attacker (or a misbehaving plugin that has somehow gotten a code path to call these APIs, OR a napi caller bypassing `read_node_as`) can write arbitrary user-label Nodes with no chain validation AND no `check_write` consultation.
- **(A2)** `Engine::register_user_view` / `create_view` / `register_handler` and similar non-CRUD WRITE entry points (per PLANNER-A's enumeration: `engine_caps.rs:683`, `engine_views.rs:976`, `engine_modules.rs:{259,325}`, `engine_diagnostics.rs:541`, `engine_wait.rs:{888,1070}`, `engine.rs:3972`, `handler_versions.rs:202`) — most of these write privileged or `system:*`-labeled Nodes, but several (modules-related) write user-visible Nodes without chain validation.

**Attacks the synthesis CLAIMS to defeat but doesn't (false-honesty risk):**
- The synthesis (and DEFERRED.md Row D-1 lines 78-82) claims engine_crud is "defended by the runtime enforcement at `CapabilityPolicy::pre_write` (already live via Compromise #2 sync-replica sub-narrative)." Verification: **`pre_write` is not a trait method** and **Compromise #2's "every transaction commit via `CapabilityPolicy::check_write`" claim refers to the evaluator/primitive_host WRITE-Node execution path (`primitive_host.rs:613`), NOT the direct `Engine::create_node` CRUD API**. The CRUD API and the eval-driven WRITE primitive are **different code paths**; the former is what napi exposes for `engine.putNode(...)` direct use.

**Fix-now-actionable (FIX-1, BLOCKER):** the synthesis must EITHER
- (a) flip F1 to PLANNER-A's full ~13-site cascade (every WRITE entry point gets `validate_chain` + sibling `check_write` consultation), OR
- (b) keep the hybrid wire-up of validator but explicitly add an engine_crud `policy.check_write` wire-up to all five CRUD APIs (mechanical: build a `CapWriteContext` from the node's first label + `actor_cid=None`-routed-to-user-root principal + call `policy.check_write(&ctx)?` before `self.backend.put_node`), OR
- (c) explicitly file a new DEFERRED row "engine_crud direct-API check_write wiring" + retense CLAUDE.md #18 + Compromise #26 + Row D-1 narrative to honestly disclose that engine_crud direct-API is gated ONLY by Inv-11 system-zone probe at v1-beta, with the cap-policy gate deferred.

DISPOSITION: FIX-NOW required before F4 implementer dispatch. PLANNER-A's full cascade rationale was correct.

---

### S2 — InstallRecordReplayStore lifecycle wiring (D-2) — **DEFEATED-CLEANLY**

**Synthesis decision (F2):** drop `Option<>` on `InstallPorts.install_record_replay_check`. New `noop_replay_check()` helper. Rename `manifest_store::install_plugin` → `install_verified_record_unchecked`. ~12-fixture cascade.

**Attacks defeated:**
- Parallel-presentation TOCTOU at install (two threads present same install_record bytes; one admits + one rejects atomically): ✅ closed end-to-end (substrate `parallel_presentation_serialized_one_admits_one_rejects` test exists at `crates/benten-engine/tests/g_core_8_install_record_replay_atomic_record_and_check.rs`).
- "I forgot to wire the closure" silent-admit footgun: ✅ closed structurally (compile error if a caller passes `Option::None`).

**Attacks ADMITS:**
- **(A3)** `install_verified_record_unchecked` (renamed side-door) is still a callable function. If a future caller routes through that path (intentionally or accidentally), the replay defense bypasses. The SAFETY doc helps but doesn't enforce. Recommend a `#[doc(hidden)]` + `#[deprecated(note = "internal-only; bypasses TOCTOU defense — use install_plugin")]` attribute OR move the function inside a `#[cfg(test)]`-only module if no production caller needs it. **(MINOR — FIX-NOW INLINE during F4 wave)**.
- **(A4)** The `InstallPorts` Option drop closes the manifest install path. But `accept_atrium_share` (Row D-5) is NOT yet built — when it lands at G-COMP-1, the implementer MUST repeat the wiring discipline. The Option-drop doesn't help here because the function doesn't exist yet to hold the port. **Acknowledged** by DEFERRED Row D-5; no F4 action.

**Attacks CLAIMS-DEFEAT-BUT-DOESNT:** none.

DISPOSITION: APPROVE; add FIX-NOW (A3) to the F4 implementer brief.

---

### S3a — check_install_consent (D-3-a) — **DEFEATED-CLEANLY**

**Synthesis decision (F3):** new Step 3c at `plugin_lifecycle.rs:905`, after replay-check. Mint NEW `PluginInstallConsentDenied` (CATALOG 192→193; 8-surface mirror).

**Attacks defeated:**
- A custom CapabilityPolicy override of `check_install_consent` (e.g. a curated-trust-list policy denying installs not on the allowlist) is currently silently ignored. ✅ closed structurally — Step 3c calls the hook + maps `CapError` → `PluginInstallConsentDenied`.
- Forensic-discrimination: the new code distinguishes "consent-policy denial" from "consent-record missing/null" (`PluginInstallConsentRequired`) and from cryptographic record-mismatch codes. ✅ preserves the R6-FP-A typed-discrimination invariant.

**Attacks ADMITS:**
- **(A5)** Step 3c fires AFTER `install_record.verify_user_signature()` at `plugin_lifecycle.rs:877`. Per the doc-comment at lines 861-870, the design intent is "Consent gate FIRST" — but the actual ordering in CODE is `verify_user_signature` (line 877) → mismatch checks (878-883) → `replay_check` (901-905) → THEN Step 3c will land at ~906. Question: is `verify_user_signature` (the cryptographic gate) on a different consent surface than `check_install_consent` (the policy gate)? Yes — they're orthogonal (one binds the install record to the user; the other lets the policy override the install decision). But the synthesis should DOCUMENT that Step 3c fires AFTER the cryptographic gate, not before — there's a small DoS-amplification surface: an attacker presenting a forged install_record causes signature-verify CPU work before the policy gets to deny. **(OBSERVABLE; FIX-NOW DOC-COUPLING in Step 3c doc-comment + DEFERRED-row note)**.
- **(A6)** Per `policy.rs:485-503`, the trait doc explicitly says install is the "SOFT cap boundary" and "institutional install-pipeline enforces the hard guarantees." Wiring `check_install_consent` upgrades this from soft to hard at the hook seam — good, but **a custom policy that intentionally DENIES installs (e.g. corporate allowlist) is now structurally able to refuse user-requested installs the user already signed**. This is the intended behavior under CLAUDE.md #18 Layer-2; the trade-off should be documented in the INTERNALS.md note.

**Attacks CLAIMS-DEFEAT-BUT-DOESNT:** none.

DISPOSITION: APPROVE; add FIX-NOW (A5/A6) doc-coupling to F4 brief.

---

### S3b — check_per_delegation (D-3-b) — **DEFEATED-CLEANLY**

**Synthesis decision:** single insertion at `engine_caps.rs:559` between Step 2b and Step 3.

**Attacks defeated:**
- A custom policy overriding `check_per_delegation` (rate-limiting, time-bounded delegation, audit) is no longer silently ignored. ✅

**Attacks ADMITS:**
- **(A7)** Insertion at `:559` is AFTER Step 2b (manifest-shares resolver) which already calls `resolver.resolve_delegation(...)` (lines 541-558). Question: should `check_per_delegation` fire BEFORE Step 2b (so the policy can deny ahead of the resolver call, saving the resolver's DAG-walk cost)? **Recommendation: keep AFTER per the design (policy's per-delegation check should compose with the manifest-shares envelope, not override it; CLAUDE.md #18 Layer-3 strictly on top of Layer-2)** — but document the ordering decision explicitly. **(OBSERVABLE; doc-coupling, FIX-NOW INLINE)**.
- **(A8)** The reuse of `PluginDelegationOutsideManifestEnvelope` ErrorCode conflates "manifest-envelope-shape denial" with "per-delegation runtime-policy denial." Forensic discrimination is REGRESSED relative to the typed-discrimination invariant the synthesis explicitly preserves at S3a. **Recommendation: mint NEW `PluginPerDelegationDenied` ErrorCode in parallel with the S3a mint** (CATALOG 192→194; cheap; preserves the invariant uniformly). **(FIX-NOW; this is a self-consistency rebuttal to F3 — if mint-discriminator-codes is right for S3a, it's right for S3b too)**.

**Attacks CLAIMS-DEFEAT-BUT-DOESNT:** none.

DISPOSITION: APPROVE; FIX-NOW (A8) requires a SECOND ErrorCode mint per the synthesis's own forensic-discrimination logic.

---

### S3c — check_write_with_audience (D-3-c) — **CLAIMS-DEFEAT-BUT-DOESNT (partial)**

**Synthesis decision (F4+F5):** workspace sweep `check_write` → `check_write_with_audience`; `audience_did = None` at non-natural-audience sites. Partial-close per FORK-A.

**Attacks defeated (partial):**
- The hook is no longer silently ignored at `engine.rs:1413` (the canonical apply_atrium_merge call site). ✅
- The seam becomes uniform across the workspace (`check_write` → `_with_audience` everywhere). ✅

**Attacks the synthesis CLAIMS to defeat but doesn't:**
- **(A9, MAJOR)** **The substantive Layer-3 audience-aware enforcement is NOT live after F4.** Per `policy.rs:571-573`, the trait default is `fn check_write_with_audience(&self, ctx) { self.check_write(ctx) }` — it IGNORES the audience field. The synthesis decides `audience_did = None` at non-natural-audience sites (which is most sites). NET EFFECT: at every existing call site, the behavior is **byte-identical** to pre-F4 (`_with_audience` defaults to `check_write`; `audience_did = None` provides no audience information; the audience-aware impl path is never exercised).

  This is a **swap that pretends to wire**. The honesty claim D-3-c "PARTIAL CLOSE" hides the fact that a custom audience-aware policy override is STILL silently ignored at every site — because no site passes `audience_did = Some(...)`. The hook can fire, but the ONLY thing it sees is `None`, which any audience-aware impl will treat as "no audience constraint." A defender writing a custom policy to deny plugin-DID writes outside the install-time envelope has no signal at runtime.

  PLANNER-B (FORK-A doc) honestly disclosed this: "Partial-close: hook no longer silently ignored, but **substantive audience-population deferred**." The synthesis adopted FORK-A but did NOT preserve this honesty disclosure in the DEFERRED.md retense wording — it just says "PARTIAL CLOSE; tighten DEFERRED.md narrative." That tightening MUST be explicit.

- **(A10)** The architectural-purist FORK-B path (synthesize/plumb audience_did from delegate_capability + apply_atrium_merge sites where it IS naturally available — peer-DID at sync ingress, target plugin_did at delegate) was rejected. But the synthesis's stated rationale ("audience-population deferred to G-COMP-1") doesn't engage with whether this is correct. At `apply_atrium_merge:1413`, the `peer_actor_cid` is known (line 1404); a natural audience_did COULD be populated. At `delegate_capability:559` (and the new S3b call site), the `plugin_did` is the natural audience. F4 LEAVES MONEY ON THE TABLE by deferring this.

**Fix-now-actionable (FIX-2, MAJOR):** the synthesis must add to the F4 brief:
- (a) explicit DEFERRED.md retense wording for D-3-c clearly disclosing "the hook is wired but receives `audience_did = None` at every call site, so audience-aware policy overrides remain silently ignored at runtime; full audience-population deferred to G-COMP-1" (closes the false-honesty gap).
- (b) Populate `audience_did = Some(peer_did)` at `apply_atrium_merge:1413` (cheap; the peer_did string is already computed at line 1416) and `audience_did = Some(plugin_did)` at `delegate_capability` (where plugin_did is the function arg). This converts D-3-c from "partial close that's behaviorally a no-op" to "partial close with two live audience-population sites."
- (c) Retense Compromise #26 to disclose the runtime-audience-deferred gap.

DISPOSITION: APPROVE-WITH-FIX-NOW (A9 + A10) — FIX-2 makes the partial-close actually mean something.

---

### S4 + D-6 + D-18 — ProductionManifestEnvelopeRechecker + EngineBuilder — **DEFEATED-CLEANLY with caveats**

**Synthesis decision (F6 + bundling):** ship `ProductionManifestEnvelopeRechecker` in `benten-platform-foundation`. EngineBuilder Path-(ii): foundation owns the canonical production builder; raw `Engine::default()` stays Noop. Bundle D-6 (handshake.rs §4.25 UnresolvedDeny) + D-18 (synthesized-`node-id:` rejection).

**Attacks defeated:**
- Adversarial peer presenting a row whose plugin attribution falls outside the locally-stored manifest envelope (after user revoked + re-published manifest with narrowed `shares`): ✅ rejected via `validate_chain_with_manifest_envelope`.
- Adversarial peer presenting an unresolvable peer-DID at sync-merge boundary: ✅ rejected with typed `UnresolvedDeny` at apply_atrium_merge AND at the §4.25 sync-hydrate path (D-6 bundle).
- Adversarial peer crafting a synthesized `node-id:N` DID hoping the Noop admits: ✅ rejected once substantive rechecker is installed (D-18 bundle).

**Attacks ADMITS:**
- **(A11, MAJOR — false-honesty residual)** Raw `Engine::default()` keeps Noop. The synthesis intent is "test/embedded posture" but **there is no compile-time pin preventing a release binary from accidentally using `Engine::default()` (or `EngineBuilder::open` without going through the foundation builder)**. If a future binary author (or napi binding maintainer) calls `Engine::open(path)` (which exists per `builder.rs:96`) without going through `benten_platform_foundation::EngineBuilder::build()`, **production ships Noop and the Layer-3 defense is silently absent**. This is exactly the v1-beta security-claim-honesty gap F4 is trying to close, and the F6 Path-(ii) choice leaves it open at a DIFFERENT seam.

  **Fix-now-actionable (FIX-3, MAJOR):** the F4 brief MUST include a test pin that asserts the napi binding constructs Engine via `benten_platform_foundation::EngineBuilder::build()` (NOT via `Engine::open` / `Engine::builder()` directly) — a build-time grep-walk or trybuild assertion catches binary regression. PLANNER-A's Path-(iii) (both) would close this; the synthesis chose Path-(ii) without the pin.

- **(A12)** D-18 closure depends on "substantive-rechecker-installed detection" (per DEFERRED.md Row D-18 lines 386-394). The proposed mechanism is: "the synthesized-fallback reject couples to substantive-rechecker-installed detection (NOT the always-mounted Noop path)." How is this detected at runtime? Via `is::<NoopManifestEnvelopeRechecker>()`? Via a marker trait? The design should specify the discriminator mechanism explicitly. **(OBSERVABLE; FIX-NOW INLINE in S4 brief)**.

- **(A13)** PLANNER-B noted "Row D-18 (synthesized-fallback hardening) naturally closes via D-4" but the synthesis correctly rejected this — D-18 needs an explicit synthesized-DID rejection branch. The reject branch goes WHERE? Inside `ProductionManifestEnvelopeRechecker::recheck_row` (foundation crate), inside the engine-side `apply_atrium_merge` after rechecker dispatch, or inside the substrate `manifest_envelope_recheck.rs`? The synthesis says "D-18 closes" but doesn't specify the rejection-branch site. **(FIX-NOW in F4 implementer brief — specify "Production impl's `recheck_row` body rejects `node-id:`-prefixed DIDs with `UnresolvedDeny`")**.

**Attacks CLAIMS-DEFEAT-BUT-DOESNT:** none structurally; the gaps above are about implementation-brief specificity.

DISPOSITION: APPROVE; FIX-3 (test pin for production-binary-uses-EngineBuilder-not-Engine::open) + FIX-NOW-inline (A12, A13) required.

---

## Section 2 — Cross-surface attack composition

### Chain C1 — engine_crud bypass + delegate bypass (composed) **[BLOCKER]**

An attacker (or compromised plugin code path) with napi `engine.putNode(...)` access can:
1. Call `engine.put_node(node_with_plugin_did_attribution)` — bypasses S1 (no check at engine_crud), bypasses S3c (no check at engine_crud — `_with_audience` isn't called either).
2. The Node lands in the backend with no chain validation and no envelope check.
3. Later, a different actor reads the Node via `read_node_as(some_principal, cid)` — the read-side cap-policy ALLOWS it because the read is gated by `check_read`, not by "was this written legitimately."

This chain composes to a **chain-validation-bypass-at-write-time + read-time-acceptance** that defeats Layer-1 (user-as-root invariant) entirely if you can reach the napi `put_node` surface. Direct mitigation requires FIX-1 closure.

### Chain C2 — install consent + per-delegation discrimination

After F4, S3a mints `PluginInstallConsentDenied` (typed) but S3b reuses `PluginDelegationOutsideManifestEnvelope` (untyped for the per-delegation runtime case). An attacker watching the operator-log signal can distinguish "consent-policy denied my install" vs "manifest-envelope denied my install/delegation" — but CANNOT distinguish "per-delegation-runtime-policy denied my delegation" vs "shares-resolver denied my delegation." Forensic-discrimination is asymmetric. Closed by FIX (A8): mint `PluginPerDelegationDenied`.

### Chain C3 — ordering: apply_atrium_merge per-row gates

After F4, `apply_atrium_merge`'s per-row loop will run (in order):
1. Revoked-actor in-memory mirror check (existing).
2. `policy.check_write(&ctx)` → S3c swap → `check_write_with_audience(&ctx with audience_did=None)` (post-F4).
3. ManifestEnvelopeRechecker dispatch (existing seam; substantive impl post-F4).
4. WriteBoundaryChainValidator (new at F4 per F1 hybrid).
5. Envelope-ceiling AND check (existing post-COLLAPSE P3).

Question: does this ordering allow any bypass via ordering-mismatch? Inspection: gate 4 (chain-validation) fires AFTER gate 3 (envelope-recheck), so an envelope-passing-but-chain-not-user-rooted row is correctly rejected by gate 4 BEFORE landing. ✅ No ordering bypass at apply_atrium_merge.

BUT: at `delegate_capability`, the ordering is:
1. Source-grant resolve (Step 1).
2. Private-namespace clause (Step 2a).
3. Shares-resolver (Step 2b).
4. `check_per_delegation` (new at S3b, between 2b and 3).
5. Effective-scope pick (Step 3).
6. WriteBoundaryChainValidator (Step 4 per S1 hybrid — at `:683`-ish).
7. Final `privileged_put_node` (Step 4 current).

Gate 6 (chain validation) fires AFTER gate 4 (per-delegation check) — correct (chain validation is a structural gate; per-delegation is a policy gate; structural-after-policy means a malformed-chain-but-policy-permitted delegation is still rejected). ✅

### Chain C4 — test corpus pass even if production reverted

**Vulnerability:** the substrate tests at `g_core_8_write_boundary_user_root_chain_validator_4_23.rs` test the **trait + outcome shape**, NOT integration with engine call sites. If F4 implementer wires the validator at `apply_atrium_merge` but a future commit accidentally reverts the wire-up (e.g. a refactor accidentally drops the `validator.validate_chain(...)` call), the substrate tests still PASS — they test the trait, not the wire-up.

**Fix-now-actionable (FIX-4, MAJOR — sweep-completeness self-verify per §3.6j):** every F4 surface MUST ship a test pin that EXERCISES the production call site and would-FAIL-if-reverted. The substrate-pin pattern is necessary but not sufficient. The synthesis's "Net scope" table allocates ~150 test LOC for S1 — verify each pin is a production-arm assertion, not a substrate-arm assertion.

---

## Section 3 — Honesty-gap residuals after F4 lands per triaged draft

Post-F4 (as currently designed), the following SECURITY-POSTURE.md + CLAUDE.md #18 claims STILL ride ahead of as-built code:

| # | Claim | What as-built actually does post-F4 | Suggested DEFERRED row |
|---|---|---|---|
| H1 | CLAUDE.md #18 Layer-1 "user-as-root" structurally enforced | Enforced at `apply_atrium_merge` + `delegate_capability` ONLY; bypassed at all direct `engine_crud::create_node` / `update_node` / `delete_node` / `create_edge` / `delete_edge` calls + napi `put_node` mirror | NEW Row D-25 "engine_crud direct-API chain-validation + check_write wiring" |
| H2 | DEFERRED.md Row D-1 line 81 "[engine_crud] runtime enforcement is the Layer-1 `CapabilityPolicy::pre_write` check (already live via Compromise #2 sync-replica sub-narrative)" | False at the code state — `pre_write` doesn't exist; `check_write` isn't called from engine_crud | RETENSE Row D-1 with honest disclosure of the engine_crud gap |
| H3 | CLAUDE.md #18 Layer-3 audience-aware enforcement live | `check_write_with_audience` is called but always with `audience_did = None`; trait default delegates to `check_write` — substantive audience-aware policy override is silently ignored at every call site | RETENSE Row D-3 c-half + ADD new Row D-26 "audience_did population at natural-audience sites" |
| H4 | CLAUDE.md #5 v1-beta-default signature = hybrid Ed25519⊕ML-DSA-65 | `plugin_manifest.rs:178` `verify_peer_signature` is hardcoded to `ed25519_dalek::Signature` 64-byte classical-only; `plugin_manifest.rs:597` `verify_user_signature` same | Already named (L2-R6-MAJOR-2 G-COMP-1 destination per backlog) — VERIFY the row exists OR ADD Row D-27 "PluginManifest + InstallRecord PQ-hybrid signature plumbing" |
| H5 | CLAUDE.md #18 Layer-2 install-time consent gated by policy | LIVE post-F4 at S3a ✅ — closed |
| H6 | F3 anti-replay (FrameReplayMarker) atomic | Already named at D-8; F4 does not close — non-F4 scope |
| H7 | Compromise #26 "the substantive defense at the merge boundary is NOT live in shipped binaries" | RESOLVED post-F4 if F6 + EngineBuilder-Path-(ii) lands AND napi binding is migrated AND a test pin enforces it (per FIX-3); risk: silent regression if napi or downstream binding bypasses the foundation builder |

**Net:** 4 honesty residuals (H1, H2, H3, H4) survive F4 as designed. H1 + H2 collapse to one fix (FIX-1). H3 reduces to "partial close that's a no-op" risk (FIX-2). H4 is non-F4-scope per the brief (G-COMP-1 destination). **H1/H2/H3 must be addressed in F4 implementation OR via explicit DEFERRED-row mints + retensed claim wording — they cannot ride post-F4 without violating the v1-beta security-claim-honesty cluster the whole F4 design pipeline exists to close.**

---

## Section 4 — Test corpus adequacy (per §3.6j sweep-completeness self-verify)

### Per-surface test-pin adequacy

| Surface | Synthesis test allocation | Adequate? | Gaps |
|---|---|---|---|
| S1 | ~150 LOC (production-arm + adversarial + ordering + sweep-walker) | NO (per F1 hybrid scope) | Missing: engine_crud direct-API test pin (FIX-1). The "sweep-walker" test pin is critical and MUST enumerate ALL WRITE entry points workspace-wide, not just the 2 wired ones — otherwise it's a self-confirming pass that misses the engine_crud gap. |
| S2 | ~100 LOC (e2e + parallel-replay + back-compat + trybuild) | YES | Add: `install_verified_record_unchecked` trybuild doc-warning per A3 |
| S3a | ~120 LOC | YES if includes: production-arm admit + custom-policy-deny + before-cap-cascade ordering + ordering-vs-verify_user_signature documentation pin |
| S3b | ~120 LOC | NO until A8 closed (additional pin for the new `PluginPerDelegationDenied` if minted) |
| S3c | ~100 LOC | NO — must include a would-FAIL-IF-AUDIENCE-AWARE-POLICY-IGNORED pin per FIX-2(b); the synthesis allocation will pass even if `audience_did = None` everywhere |
| S4 + D-6 + D-18 | ~300 LOC | NO until A11 closed — must include "napi binding uses foundation builder, not Engine::open" trybuild or grep-walk pin per FIX-3 |

### Tests that would pass if production wiring was reverted

Per the §3.6j discipline: the synthesis-proposed test pin set has at least 3 vulnerabilities to silent-regression:

1. **S1 hybrid sweep-walker:** if the walker only enumerates the 2 wired sites, an accidental revert at one of them passes the walker (because the walker only checks the 2). Walker MUST enumerate workspace-wide WRITE entry points + assert each is gated OR explicitly exempt.
2. **S3c partial-close:** if test pins only assert `check_write_with_audience` is CALLED (not that audience-aware behavior fires), production can revert the swap and tests still pass (the trait default delegates to `check_write`).
3. **S4 EngineBuilder Path-(ii):** if test pins only assert `EngineBuilder::build()` wires the production rechecker, but the napi binding regresses to `Engine::open`, production ships Noop and tests pass.

All 3 are closed by the FIX-1 / FIX-2(b) / FIX-3 actions above.

### Tests conflating "substrate works" with "consumer wired"

The substrate tests at `g_core_8_*` (existing pre-F4) test the substrate trait/outcome shape. F4's new tests MUST be **end-to-end production-pin tests** that exercise the substrate THROUGH the consumer wire-up. The synthesis allocation appears to budget for this, but the brief MUST be explicit: every F4 test pin EXERCISES the production code path AND would-FAIL-if-no-op'd at the production site.

---

## Section 5 — Pre-decision rebuttals (F1-F6 + F7-F8)

| Fork | Synthesis decision | Critic rebuttal | Disposition |
|---|---|---|---|
| **F1 — S1 scope** | Hybrid (2 wire sites) | **SUBSTANTIVE REBUTTAL** — rationale "engine_crud defended by `pre_write`" is **architecturally false at HEAD** (verified at `engine_crud.rs:47-194`; `pre_write` is not a trait method). Either flip to PLANNER-A full cascade OR honestly disclose the gap (FIX-1). | **FIX-NOW; surface to Ben as a real fork** (full-cascade vs honest-disclose-the-gap) |
| **F2 — S2 Option shape** | Drop Option | Accept; structural-always-on is correct. PLANNER-B's "keep Option" preserves the silent-admit-by-omission footgun. | ACCEPT |
| **F3 — S3 ErrorCode** | Mint `PluginInstallConsentDenied` | Accept; preserves typed-discrimination invariant. **BUT** apply uniformly: mint `PluginPerDelegationDenied` at S3b too (FIX A8). | ACCEPT + EXTEND |
| **F4 — S5 sweep scope** | Workspace sweep | Accept; uniform seam is correct. | ACCEPT |
| **F5 — S5 audience_did** | `None` at non-natural-audience | **PARTIAL REBUTTAL** — `apply_atrium_merge:1413` (peer_did available) + `delegate_capability` (plugin_did is the function arg) ARE natural-audience sites. F5 should populate at THESE sites; remain `None` only at sites where no natural audience exists (FIX-2(b)). Otherwise S3c is a swap-that-pretends-to-wire. | **FIX-NOW; partial accept (None at sites with no natural audience) but populate at sites with natural audience** |
| **F6 — EngineBuilder location** | Foundation Path-(ii) | Accept; PARTIAL — must add a compile-time / test-time pin that the napi binding goes through foundation builder, not raw `Engine::open` (FIX-3). PLANNER-A's Path-(iii) was a safer choice in this respect. | ACCEPT + ADD PIN |
| **F7 — Bundle D-6 + D-18 with S4** | Bundle | Accept; architecturally inseparable. | ACCEPT |
| **F8 — D-24 cross-wave coupling** | Include in F4 | **REBUTTAL — phantom destination**. Row D-24 does NOT exist in `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (verified: rows go D-1..D-21 + gaps at D-22/D-23/D-24; the highest row is D-21). PLANNER-A's "FORK D" references D-22/D-23/D-24 but the rows are unminted. Either MINT the row with explicit content before bundling OR DROP F8. | **HARD-ESCALATE per HARD RULE 12: phantom destination — fork must name what is actually being bundled, OR drop the line, OR mint the row first** |

---

## Section 6 — New surfaces missed (beyond the 6)

### S7 — `PluginManifest::verify_peer_signature` classical-Ed25519-only **[OUT-OF-SCOPE for F4; BELONGS-NAMED-NOW]**

- Verified at `crates/benten-platform-foundation/src/plugin_manifest.rs:163-184`: hardcoded `ed25519_dalek::Signature::from_bytes(&sig_bytes)` 64-byte signature; no codepoint-dispatch; no PQ-hybrid path.
- L2-R6-MAJOR-2 honesty gap per the R6 R1 lens. CLAUDE.md #5 says v1-beta default = hybrid Ed25519⊕ML-DSA-65; this code is classical-only.
- F4 scope correctly excludes this (per brief — G-COMP-1 destination per the crypto-agility seam).
- **BELONGS-NAMED-NOW** — verify a DEFERRED row exists ("PluginManifest + InstallRecord PQ-hybrid signature plumbing"); if not, MINT one as part of F4's doc-touch pass.

### S8 — `InstallRecord::verify_user_signature` classical-Ed25519-only **[OUT-OF-SCOPE for F4; BELONGS-NAMED-NOW]**

- Verified at `plugin_manifest.rs:592-602`: same shape as S7.
- Same disposition; same destination.

### S9 — T10-upgrade gap (L2-R6-MAJOR-1) **[BELONGS-NAMED-NOW]**

- Per L2 R6 lens output: rotation-log-aware variant (`validate_with_rotation_log_and_clock`) is deferred per `phase-4-backlog.md §4.10`; at v1-beta a rotated peer-DID still passes install (D-4F-12: rotation → WARNING not hard-reject) but the WARNING-emitting path isn't called yet.
- F4 scope correctly excludes this (per brief — different concern).
- VERIFY this is in a DEFERRED row; not in F4's wire-up.

### S10 — FrameReplayMarker TOCTOU (D-8) **[ACKNOWLEDGED OUT-OF-SCOPE]**

- DEFERRED.md Row D-8 exists; out of F4 scope; correctly excluded.

### S11 — Direct backend access via `Engine::backend()` **[VERIFIED CLEAN]**

- `crates/benten-engine/src/engine.rs:2094` — `Engine::backend()` is `pub(crate)`.
- `crates/benten-engine/src/engine.rs:2107` — `Engine::backend_for_test()` is `pub` but name-pinned for test-only use.
- napi binding spot-check: no `backend` accessor exposed.
- ✅ Backend-raw bypass is structurally closed at HEAD; F4 wire-up cannot be bypassed via direct backend access.

---

## Section 7 — Overall disposition

**APPROVE-WITH-FIX-NOW** + **1 HARD-ESCALATE on F8** (phantom destination per HARD RULE 12).

### Required fix-now actions before F4 implementer dispatch (ordered by severity)

1. **FIX-1 (BLOCKER)** — F1 pre-decision rationale is architecturally false. Choose: (a) flip to PLANNER-A full ~13-site cascade, OR (b) add explicit `check_write` wire-up to `Engine::create_node` / `update_node` / `delete_node` / `create_edge` / `delete_edge` PLUS the validator wire-up to those sites, OR (c) MINT a new DEFERRED row honestly disclosing the engine_crud gap + retense Compromise #26 + tighten Row D-1 wording (delete "pre_write" — it doesn't exist) + retense CLAUDE.md #18. **Surface to Ben as a real fork; the synthesis pre-decision rests on a false premise.**
2. **FIX-2 (MAJOR)** — S3c partial-close must populate `audience_did = Some(peer_did)` at `apply_atrium_merge:1413` and `audience_did = Some(plugin_did)` at `delegate_capability` + the new S3b call site (natural-audience sites). Otherwise S3c is a swap-that-pretends-to-wire and the partial-close claim is false-honesty.
3. **FIX-3 (MAJOR)** — F6 Path-(ii) must include a compile-time or test-time pin that the napi binding constructs Engine via `benten_platform_foundation::EngineBuilder::build()`. Otherwise production binaries can silently regress to Noop via `Engine::open` / raw `Engine::builder()`.
4. **FIX-4 (MAJOR)** — sweep-completeness self-verify per §3.6j. Every F4 test pin EXERCISES the production code path + would-FAIL-if-no-op'd. Substrate-arm pins don't count. Specifically: S1 sweep-walker MUST enumerate workspace-wide WRITE entry points; S3c MUST assert audience-aware-policy denial fires; S4 MUST assert napi-via-foundation-builder.
5. **FIX-5 (MAJOR)** — S3b mints `PluginPerDelegationDenied` (mirrors F3 ratification at S3a for self-consistency).
6. **FIX-6 (MINOR)** — A3: `install_verified_record_unchecked` gets `#[doc(hidden)]` + `#[deprecated]` + a trybuild compile-warning pin.
7. **FIX-7 (MINOR)** — A5/A6: doc-coupling at S3a Step 3c explaining ordering vs `verify_user_signature` + the "soft cap boundary upgraded to hard" trade-off.
8. **FIX-8 (MINOR)** — A12/A13: F4 implementer brief specifies the discriminator mechanism for D-18 "substantive-rechecker-installed detection" AND the synthesized-`node-id:` rejection site (inside `ProductionManifestEnvelopeRechecker::recheck_row`).

### HARD-ESCALATE

**F8 (D-24 cross-wave coupling)** — Row D-24 does NOT exist in DEFERRED.md (verified via `grep -n "^### " docs/V1-FROZEN-INTERFACE-DEFERRED.md`). Per HARD RULE 12 clause-(b), naming a "BELONGS-NAMED-NOW" destination requires the destination to receive the entry NOW — naming D-22/D-23/D-24 without minting them violates the rule. **Surface to Ben: either mint D-22/D-23/D-24 with explicit content as part of F4's doc-touch pass, or drop F8 entirely.** This is small but it's the HARD RULE; the synthesis cannot ride a phantom destination.

### Honesty-residual disclosure (post-F4 retense obligations)

If FIX-1 is closed via option (c) (disclose the gap + retense), the SECURITY-POSTURE.md + CLAUDE.md #18 + DEFERRED.md retense pass MUST include:
- Compromise #26 retense disclosing the engine_crud cap-gate gap.
- CLAUDE.md #18 Layer-1 retense disclosing that user-as-root is enforced at apply_atrium_merge + delegate_capability ONLY, not at direct CRUD APIs.
- DEFERRED.md Row D-1 retense deleting the `pre_write` claim and naming the engine_crud gap honestly.
- New Row D-25 (engine_crud direct-API chain + check_write wiring) + Row D-26 (audience_did population at natural-audience sites) + verify Row mints for S7/S8/S9.

---

## Footnote — Provenance verification log

- `crates/benten-engine/src/engine_crud.rs:47-218` — confirmed no `check_write` / `pre_write` call in CRUD APIs.
- `crates/benten-caps/src/policy.rs:354-573` — confirmed trait has `check_write` (no `pre_write`); `check_write_with_audience` default delegates to `check_write`.
- `crates/benten-engine/src/engine.rs:1400-1499` — confirmed apply_atrium_merge per-row check_write + manifest-envelope-recheck wiring (post-F4 will add validator).
- `crates/benten-engine/src/engine_caps.rs:429-591` — confirmed delegate_capability shape (no check_per_delegation / validator at HEAD).
- `crates/benten-platform-foundation/src/plugin_lifecycle.rs:860-905` — confirmed install-pipeline ordering (verify_user_signature → mismatch checks → replay_check → [Step 3c lands here post-F4]).
- `crates/benten-platform-foundation/src/plugin_manifest.rs:163-184, 592-602` — confirmed classical-Ed25519-only signature paths (S7/S8).
- `crates/benten-caps/src/chain_authority.rs:404-421` — confirmed FrameReplayMarker TOCTOU (D-8).
- `crates/benten-engine/src/builder.rs:35-757` — confirmed EngineBuilder shape; no set_manifest_envelope_rechecker / set_write_boundary_chain_validator call.
- `crates/benten-engine/src/engine.rs:1923-1937` — confirmed default-builder installs Noop for both validators.
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` — confirmed rows D-1..D-21 exist; D-22/D-23/D-24 do NOT exist (F8 phantom).
- `docs/SECURITY-POSTURE.md:42, 2043-2128` — confirmed Compromise #26 wording references "CapabilityPolicy::pre_write" (the false claim).
