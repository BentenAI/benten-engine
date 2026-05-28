# MembershipSet unification — M-CONS consolidator (5-panel + 3-cataloger reconciliation)

> **Top-banner re-orient (HANDOFF discipline).** This document consolidates the 5-specialist
> MembershipSet unification panel (M2 primitive-design + M3 amendment-transformation + M4
> red-team + M5 CGKA-candidate-survey + M6 transport-configurability) into a single coherent
> design + simplified amendment registry + R0 plan-doc skeleton. Read BEYOND what's named
> here if you arrive cold: the 3 cataloger outputs (M1a/M1b/M1c), the 9-eyes consolidated
> registry (`fbdfeb16`), the 5 critique-round outputs (`3f5a4351` / `9c548e5f` / `79c99aa5`
> / `6ea9718a` / `8e374a9d`), Q3-revisit (`23f76e24`), AtriumPolicy (`e90900b4`), Path-A.5
> (`5f50a028`), Ben's already-ratified Q1/Q2/Q3-DUAL-CID/Q4/Am4-Sealed-Sender decisions.
>
> Tree HEAD at write: `2172cb6d` (origin/main, fetched + verified at session start).
> Branch: `phase-4-meta-core/membership-set-m-cons-consolidator`.
> Date: 2026-05-27.

- **Role.** M-CONS consolidator — single coherent reconciliation of the 5-panel + 3-cataloger
  outputs. Decision-ready input for Ben + the subsequent critique-round (M-C1 elegant-shape /
  M-C2 composability / M-C3 fresh-eyes).
- **Scope.** Tasks 1-10 per the dispatch brief. NOT primitive Rust API drafting (M2 owns).
  NOT a new amendment-transformation walk (M3 owns; this doc consolidates). NOT a red-team
  re-pass (M4 owns; this doc reconciles M4's BREAKs against M2's actual landed shape).
- **Out-of-scope.** Authoring the full R0 plan-doc (M-R0 owns; this doc gives the skeleton).
  Re-running the critique round (M-C1/C2/C3 do that against this doc).

---

## §0 Headline summary (read-cold)

**Verdict.** Ratify MembershipSet unification at R0 along M2's typed-variant
`MembershipSetKind { Atrium, DeviceMesh, SingleDevice }` shape, with M3's amendment
transformation absorbed, M4's 5 BREAKs verified-already-handled by M2 (with two
sharpening amendments), M5's CGKA-LITE → MultiRecipientSealing rename + AtriumWithCGKA
codepoint-reserve adopted, and M6's TransportConfig codepoint-reserve adopted. Net
delta: **–4.3 to –5.5 wave-days** off the ~80-101 baseline (M3 central estimate) +
**+3.5 wave-days for TransportConfig codepoint-reserve** (M6) + **+0 for CGKA-reserve**
(M5). **Net: –0.8 to –2.0 wave-days** with substantial doc + LOC + audit-page shrinkage.

**Confidence.** MED-HIGH on the directional consolidation; MED on the cost arithmetic
(many small per-amendment estimates compound; M3's "honest hedge" bracket of –1 to –3
remains the conservative-bound); HIGH on the 5-panel reconciliation (M2 + M4 + M5 +
M6 align cleanly; M3's structural analysis is the load-bearing simplification engine).

**Convergence between panelists.** All 5 specialists independently arrived at:
- Typed-variant primitive (M2 explicit; M4 explicit via §11.1; M3 + M5 + M6 assume it).
- Forkability is Atrium-only (M2 §4 op table reject for DeviceMesh/SingleDevice; M4
  BREAK 1; M3 implicit; M5 §6.2 fold ADMIN-KICK into FORK-ONLY).
- K_principal-per-DID stays; K_Set sits ON TOP via HPKE-wrap not in place of (M2 §3
  + §8 + DeviceMesh K_Set=K_principal mapping; M4 §4.3 + §7.4; M3 absorbs in E19).
- DUAL-CID on wire; plaintext_cid_local stays local-only (M2 §9 explicit; M3 E18
  inherits Q3 ratification).
- Per-Kind admin governance dispatch (M2 KindPolicy enum; M4 §11.1 typed governance
  amendment; M3 D4 + D7 EXPANDED per-Kind).
- Plugin-DIDs + agent-DIDs are NOT first-class MembershipSet members (M2 §6.3/6.4;
  M4 BREAK 5; both invoke CLAUDE.md #18).

**The only substantive divergences** (resolved in §1.3 below): (a) M2 keeps `fork` +
`dedup_blind_cid` inside the primitive surface; M3 + M4 lean sibling-trait
externalization. **Resolution: M2's call (keep inside) stands, with sibling-trait
extraction reserved as a v1.x refactor option per M3 §8.4.** (b) M5 wants
`MembershipSetKind::{AtriumFork, DeviceMesh, SingleDevice}` rename (Atrium → AtriumFork
to make semantic explicit); M2 ships `Atrium`. **Resolution: keep M2's `Atrium` name;
audit-deliverable docs spell out "AtriumFork-semantic per Q3 Option D"** — naming-as-
documentation is weaker than naming-as-type-constraint; the Compromise #48 row + the
SECURITY-POSTURE.md disclosure carry the semantic clearly.

---

## §1 Task 1 — Reconcile the 5 panelist outputs

### §1.1 Adopt M2's typed-variant primitive

**The final shape is M2 §2.1 verbatim** with the 5 amendments below absorbed (most are
sharpenings; none require structural redesign of M2's struct). Recapping the M2 shape:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MembershipSetKind {
    Atrium = 0,        // K_Set = K_Atrium (CSPRNG); multi-user; admin-DID governed
    DeviceMesh = 1,    // K_Set = K_principal; single-user N devices; user-IS-admin
    SingleDevice = 2,  // K_Set = K_principal; degenerate N=1; self-signed
}

pub struct MembershipSet {
    pub version: u8,                            // = 0x01
    pub kind: MembershipSetKind,
    pub id: Option<MembershipSetId>,            // canonical-CBOR + BLAKE3
    pub members: Vec<MemberKey>,                // typed-per-Kind member element
    pub shared_key: K_Set,                      // skip_serializing on wire; HPKE-wrapped
    pub policy: MembershipSetPolicy,            // KindPolicy enum inside
    pub parent_membership_set_id: Option<MembershipSetId>,
    pub created_at_hlc: BentenHlc,
    pub membership_attestation: MembershipAttestation,
    pub transport_config: TransportConfig,      // M6 ADDITION; see §1.6
}
```

7-op API per M2 §4: `create / add_member / remove_member / encrypt_to_set /
dedup_blind_cid / fork / update_policy` — kept on the primitive (sibling-trait extraction
reserved as v1.x option per M3).

### §1.2 Address M4's 5 BREAKs against M2's actual landed shape

M4 wrote against M-PENDING; this consolidator verifies each BREAK against M2's published
design.

| M4 BREAK | M2's handling | Status |
|---|---|---|
| **BREAK 1** — Forkability is Atrium-only | M2 §4 op table: `fork()` **rejects for DeviceMesh + SingleDevice** at runtime. M2 keeps `fork` inside the primitive but it's a Kind-typed-error. | **RECONCILED**. M4 prefers sibling-trait externalization; M3 also leans sibling. **Resolution: M2's runtime-typed-reject stands at v1-beta; sibling-trait extraction reserved as v1.x refactor per M3 §8.4 if the runtime-reject ergonomics prove painful.** Audit-deliverable doc names this. |
| **BREAK 2** — DAK ≠ K_Atrium | M2 §3 + §8 + §6.2 keep DAK strictly per-device per-CLAUDE.md #17 (in `MemberKey::DeviceDid.attestation` envelope), while K_Set is the per-set encryption key. M2 §3.4 explicitly: "differ in K_Set provenance". **DAK never appears as `MembershipSet.shared_key`.** | **RECONCILED**. M2's design already keeps them typed-separately. The category-error M4 warned against does not occur. |
| **BREAK 3** — K_principal is per-DID, not per-Atrium | M2 §3.2 maps DeviceMesh `K_Set = K_principal` (the user's existing per-DID K_principal — NOT a new per-Atrium K_principal). For Atrium-Kind, M2 mints fresh CSPRNG K_Atrium (NOT derived from any one user's K_principal). **K_principal substrate stays per-DID at v1-FROZEN.** | **RECONCILED**. M2 explicitly chose M4 §4.3 option (ii) — "a SEPARATE K_Atrium NOT derived from any one user's K_principal" — and the substrate-incorrect option (i) is not chosen. |
| **BREAK 4** — Kick semantics differ per Kind | M2 §4 `remove_member` dispatches on Kind via `RotationPolicy` parameter; Atrium gets ForkOnly default + AdminKickEpoch opt-in; DeviceMesh gets ForkOnly only; SingleDevice rejects. Plus M2 KindPolicy enum gives typed admin-signing-key per Kind. | **RECONCILED + SHARPENED**. **Amendment A1 (new):** rename `remove_member` per-Kind contract clearly in audit-deliverable docs to surface the semantic gap M4 identified (DeviceMesh "revoke lost phone" vs Atrium "kick adversarial member") — this is doc-only, no code change. Per M5 §6.2 ratification, AdminKickEpoch FOLDS into FORK-ONLY at v1-beta (no real CGKA shipped); the enum arm reserves the post-v1-beta CGKA-shape but is not implementation-bound. |
| **BREAK 5** — Plugin-DIDs + agent-DIDs are NOT members | M2 §6.3 + §6.4 explicit: `MemberKey` admits ONLY `UserDid` / `DeviceDid` / `LocalDevice`. **Plugin-DIDs + agent-DIDs are categorically not in `MemberKey`.** They live in adjacent `CapabilityRecipient` / attribution-layer structures per CLAUDE.md #18. | **RECONCILED**. M2's design already enforces homogeneous-per-Kind member element via typed enum. |
| **BREAK 6 (bonus)** — Audit-event shape differs per Kind | Not explicitly addressed in M2 §2; addressed implicitly via `KindPolicy` typed dispatch + per-Kind threat-model section (M2 §5). | **SHARPENED**. **Amendment A2 (new):** mint a `MembershipEvent` typed-enum (NOT a uniform stream) with per-Kind variants (`MembershipEvent::AtriumKick { actor, target, ... } / DeviceMeshRevoke { user_did, device_did, ... } / SingleDeviceNoop`). Audit-deliverable docs name this explicitly. Bounded: ~50 LOC; folds into M2's primitive crate. |

**Verdict on M4's 5 BREAKs.** Three were resolved-as-designed by M2 before M4's audit (BREAKs 2/3/5); one was resolved-by-construction at the op-API layer (BREAK 1; runtime-reject vs sibling-trait is a stylistic choice not a soundness gap); two carry through as new amendments F-A1 + F-A2 to surface the semantic gaps in audit-deliverable docs without changing the type. **M4's verdict CONCUR-WITH-AMENDMENTS-AT-75-80%-CONFIDENCE is preserved + sharpened to CONCUR-WITH-EVIDENCE-AT-~85% under M2's actual landed shape.**

### §1.3 Incorporate M3's amendment transformation

M3's 5 ELIMINATED + 17 SIMPLIFIED + 6 EXPANDED + 2 RENAMED transformation is the
load-bearing simplification engine. M-CONS adopts the M3 §6.1 simplified E1-E25 registry
verbatim, RE-KEYED as F1-FN per the brief's Task 2 directive (see §2 below for the
full final-final registry). The M3 wave-day delta of **–4.3 to –5.5 wave-days** is
adopted as central estimate.

### §1.4 Adopt M5's CGKA-LITE → MultiRecipientSealing rename + AtriumWithCGKA codepoint-reserve

Per M5 §9.1 final recommendation. The R0 plan-doc + SECURITY-POSTURE.md + audit-deliverable
docs all use **`MultiRecipientSealing`** as the canonical name (replacing the misleading
"CGKA-LITE" framing that promised PCS/FS properties not actually delivered). Reserve
**`MembershipSetKind::AtriumWithCGKA`** as a codepoint at v1-beta-CODEPOINT-RESERVE
(NOT a 4th enum arm at v1-beta — kept as a reserved-codepoint-slot per the §15.c
HALT-AND-SURFACE-TO-BEN discipline that M2 §1 cites). The 3-arm enum stays
EXACTLY-3 at v1-beta; future `AtriumWithCGKA` ratification = HALT-AND-SURFACE-TO-BEN.

**Compromise mint:** add Compromise #52 (MembershipSet-no-PCS-against-removed-members)
to SECURITY-POSTURE.md per M5 §9.2 item 6. See §3.

### §1.5 Adopt M6's TransportConfig codepoint-reserve at v1-beta

Per M6 §7.2 Option A. The `TransportConfig` type lands at v1-beta with the wire shape
frozen but production iroh-gossip impl deferred to Phase-4-Meta-Composing / Phase-5.
**Add `pub transport_config: TransportConfig` field to `MembershipSet` struct** (M6
§5.2 + §10.1 deliverable #4). Default-selection logic ships at v1-beta (SingleDevice →
None; DeviceMesh → BlobsPullMst; Atrium-small → BlobsPullMst; Atrium-large → reserved
codepoint).

**Compromise mint:** add Compromise #53 (TransportConfig-codepoint-reserve-at-v1-beta;
gossip-impl-deferred) noting that the gossip-impl behind the codepoint may shift
during Phase-4-Meta-Composing per the §7.2 reversible-up rationale.

### §1.6 Contradiction reconciliation — M3 "fork stays in 7-op primitive" vs M4 "fork is Atrium-only"

**No contradiction; resolved by M2's per-Kind specialization.** M3's "fork stays in
the primitive's op surface" + M4's "fork is Atrium-only" are compatible: the op exists
on the primitive (M3-aligned) but dispatches per-Kind to a typed-reject for DeviceMesh +
SingleDevice (M4-aligned). M2 §4 op-table makes this concrete.

**Stylistic note.** A future sibling-trait refactor (`AtriumOps::fork(&self) -> ...`)
could move `fork` out of `MembershipSet` and onto an Atrium-only trait. Both shapes
are defensible. M2 ships the runtime-reject shape at v1-beta because it preserves the
uniform 7-op primitive cognitive model; sibling-trait extraction reserved as v1.x
option per M3 §8.4 if the runtime-reject ergonomics prove painful at call sites.

### §1.7 Cross-panel composition check (other contradictions)

| Question | M2 says | M3 says | M4 says | M5 says | M6 says | Reconciled? |
|---|---|---|---|---|---|---|
| Forkability primitive surface | inside `MembershipSet::fork()` w/ Kind-reject | sibling-traits leans cleaner | externalize per §11.4 | fold ADMIN-KICK into FORK-ONLY | n/a | **YES** — M2's call stands; sibling reserve as v1.x option |
| DAK vs K_Set | typed-separately (DAK in attestation; K_Set is the shared key) | UNCHANGED | category-error to conflate | n/a | n/a | **YES — agree, typed-separate** |
| K_principal scope | per-DID; DeviceMesh shares user's K_principal | per-DID | per-DID per v1-FROZEN | n/a | n/a | **YES** |
| Plugin-DID membership | NOT MemberKey | UNCHANGED | NOT a member; CLAUDE.md #18 | n/a | n/a | **YES** |
| `Atrium` vs `AtriumFork` name | `Atrium` | accepts | accepts | `AtriumFork` preferred | n/a | **RESOLVED — keep `Atrium`; doc spells out fork-semantic** |
| TransportConfig field on MembershipSet | not in §2.1 | n/a | n/a | n/a | YES (§5.2) | **YES — add per M6** |
| MembershipPolicy admin per-Kind | KindPolicy enum (already typed) | D4 EXPANDED per-Kind | typed governance amendment §11.1 | n/a | n/a | **YES — converged** |
| MemberKey homogeneous per Kind | YES (typed enum) | implicit | YES amendment §11.5 | n/a | n/a | **YES** |
| Gardens recursion at v1-beta | shape doesn't preclude; Phase 7 | reserve via codepoint | recursion needs 4 distinct composition rules; Phase 7 design | n/a | per-sub-Atrium TransportConfig inherit | **YES — Phase 7 deferral; v1-beta shape doesn't preclude** |
| CGKA at v1-beta | RotationPolicy::AdminKickEpoch slot reserved | UNCHANGED | n/a | NO CGKA at v1-beta; codepoint-reserve only | n/a | **YES — converged on M5's recommendation** |

**Net cross-panel consistency.** Strong. 5 panels independently triangulated to the
same structural shape with only stylistic divergences on (a) primitive-vs-sibling-trait
surface placement for fork + dedup_blind_cid, (b) `Atrium` vs `AtriumFork` naming.
Both resolved above without changing the substantive design.

---

## §2 Task 2 — Final-final amendment registry (F1-F27)

Renumbering scheme: original 28 unified + post-critique additions go to F-prefix
("Final-final"). Format: `F# | Title | Severity | Wire-affecting | Disposition |
Depends-on | Doc-impact | Confidence | Origin-cite`.

Severity: **LOAD-BEARING** / **MED-HIGH** / **DEFERRABLE**.
Wire-affecting: **YES** / **NO** / **CODEPOINT-RESERVE-ONLY**.
Disposition: **v1-beta-LOAD-BEARING** / **v1-beta-CODEPOINT-RESERVE** /
**v1-GM-DEFER** / **v1-assessment-window-DEFER**.

| F# | Title | Sev | Wire | Disp | Depends-on | Doc-impact | Conf | Origin-cite |
|---|---|---|---|---|---|---|---|---|
| F1 | Codepoint committed in AAD/info | LB | YES | v1-beta-LB | — | aead.rs + CRYPTO-CODEPOINTS.md | HIGH | M3 E1 / U1 / L2-Am1 |
| F2 | Strict-decode; no cross-variant fallback | LB | YES | v1-beta-LB | F1 | aead.rs + ENVELOPE-V1-SPEC.md | HIGH | M3 E2 / U2 |
| F3 | Canonical TLV length-injectivity | LB | YES | v1-beta-LB | F1 | aead.rs | HIGH | M3 E3 / U3 |
| F4 | Sender-DID + member-DID-list in AAD (non-Vault MembershipSet variants) | LB | YES | v1-beta-LB | F1, F17 | aead.rs + AAD layout doc | HIGH | M3 E4 / U4 (simplified) |
| F5 | Replay-window per-MembershipSet-Kind exclusion table | LB | YES | v1-beta-LB | F1, F17 | aead.rs + R-C2 + U28 | HIGH | M3 E5 / U5+R-C2+U28 |
| F6 | Bernstein-Persichetti Decap CT mitigation | LB | NO | v1-beta-LB | — | libcrux-ml-kem + Compromise #32 | HIGH | M3 E6 / U6 + Q1 ratification (libcrux-ml-kem chosen) |
| F7 | BE codepoint on-wire | LB | YES | v1-beta-LB | F1 | aead.rs (Q2 BE migration in progress) | HIGH | M3 E7 / U7 + Q2 ratification |
| F8 | Codepoint registry IANA-disjoint + cite-drift scanner + MembershipSetEncryption family | LB | YES | v1-beta-LB | F1 | CRYPTO-CODEPOINTS.md | HIGH | M3 E8 / U8 (simplified; 4-fanout codepoints collapse to one MembershipSetEncryption family with Kind discriminator) |
| F9 | `EnvelopePayload` `#[non_exhaustive]` + typed-reject + MembershipSetEncryption variant | LB | YES | v1-beta-LB | F8, F17 | benten-crypto-suite | HIGH | M3 E9 / U9 (expanded) |
| F10 | `BindingContext` `#[non_exhaustive]` + typed-reject + MembershipSetSeal variant | LB | YES | v1-beta-LB | F1, F17 | benten-crypto-suite | HIGH | M3 E10 / U10 (expanded) |
| F11 | Escape codepoint + experimental range + EnvelopeShape codepoint axis | LB | YES | v1-beta-LB | F8 | CRYPTO-CODEPOINTS.md | HIGH | M3 E11 / U11 |
| F12 | Nonce-length variant discrimination | LB | YES | v1-beta-LB | F1 | aead.rs | HIGH | M3 E12 / U12 |
| F13 | FS-gap honest-disclosure + MLS-PQ + CGKA codepoint reservation | LB | CODEPOINT-RESERVE-ONLY | v1-beta-LB (disclosure) + v1-beta-CODEPOINT-RESERVE (MLS-PQ slot per M5 + AtriumWithCGKA slot per M5) | F11 | SECURITY-POSTURE.md Compromise #42 + #52 + CRYPTO-CODEPOINTS.md | HIGH | M3 E13 / U13 + Compromise #42 + M5 §9.1 |
| F14 | `aad_version: u8` prefix | LB | YES | v1-beta-LB | F1 | aead.rs | HIGH | M3 E14 / U14 |
| F15 | Did multikey + Did::Unknown | LB | YES | v1-beta-LB | — | benten-id | HIGH | M3 E15 / U15 |
| F16 | CodepointLifecycle typed-state | LB | NO | v1-beta-LB | F8 | CRYPTO-CODEPOINTS.md | HIGH | M3 E16 / U16 |
| **F17** | **MembershipSet primitive (typed-variant Atrium/DeviceMesh/SingleDevice; multi-stanza-HPKE distribution; FORK-ONLY rotation; per-stanza AAD; generation-CRDT-vector; transport-config codepoint-reserve)** | **LB** | **YES** | **v1-beta-LB** | F1, F8, F17 own crate | NEW `benten-membership-set` crate + V1-FROZEN-INTERFACE.md row + MembershipSet-Spec doc + TS mirror | **HIGH** | M3 E17 (NEW; absorbs U17 + U19 + U25 + R-C3 + L11 U42 + Q3 §7.1 U42-suggest + Path-A.5 K(V)) + M2 primitive design + M6 TransportConfig field |
| F18 | Triple-CID model parameterized over MembershipSet (`plaintext_cid_local` LOCAL-ONLY + `plaintext_cid_membership_set` + `envelope_blob_cid`) per Ben's Q3 DUAL-CID-on-wire ratification | LB | YES | v1-beta-LB | F17 | ENVELOPE-V1-SPEC.md + benten-graph local-index | HIGH | M3 E18 / U18 (simplified) + Q3 Option D + Q3 ratification + M2 §9 (DUAL-CID-on-wire reaffirmed) |
| F19 | MembershipSet-generation tracking (CRDT-vector per-member-DID partition; replaces U19 + U20 + U42-L11) | LB | YES | v1-beta-LB | F17 | benten-membership-set | HIGH | M3 E19 (absorbs U19 + U20 + U42-L11) |
| F20 | `PermissionOperation::ExecuteWorkflow` variant + per-Kind sub-arms + per-request-nonce | LB | YES | v1-beta-LB | F10, F17 | benten-caps + UCAN spec | HIGH | M3 E20 / U21 (expanded) + L11 U21-ext + C3 §6.4 R4 |
| F21 | Sealed-Sender additive codepoint family slot (paired with MembershipSetEncryption family per Inv-18b) | LB (slot) + RECOMMENDED (impl) | YES (slot) | v1-beta-CODEPOINT-RESERVE + v1-GM-DEFER impl | F8, F17 | CRYPTO-CODEPOINTS.md + Inv-18b doc + Am4 ratification | HIGH | M3 E21 / U22 (simplified) + C2 R-C1 + Am4 (Sealed-Sender DEFAULT per Ben; drop non-sealed-sender parallel codepoint per Am4) |
| F22 | Per-relay-unlinkability + universal size-class buckets + cover-traffic deferral + DID-rotation deferral | mixed | YES (buckets) / NO (cover-traffic deferred) | mixed | F1, F8 | THREAT-MODEL.md | MED-HIGH | M3 E22 / U23+U24+R-C7+U26+U27 |
| F23 | Cross-ecosystem-identifier emit-discipline (per-Kind map row) + DAG-CBOR outer framing + strict-deterministic-CBOR-decode | LB | YES | v1-beta-LB | F8 | CROSS-ECOSYSTEM.md + DAG-CBOR doc | HIGH | M3 E23 / U29+U30+C3 §6.3 R3+§6.7 R7 |
| F24 | Impl-engineering pack (libcrux-ml-kem per Q1 + XChaCha20 + NAPI canonical_binding + NAPI opaque-handle + MembershipSetHandle + wasm_js cfg + cancel-safety + Argon2idParameterTier per-Kind for DeviceMesh+SingleDevice Vault) | LB | mixed | v1-beta-LB | F17 | impl + INTERNALS.md | HIGH | M3 E24 / U31+U32+U33+U34+U35+U36+L10 U41 + Q1 ratification |
| F25 | Audit-deliverable pack (golden-vector corpus + dudect CI + kani injectivity + THREAT-MODEL.md + SECURITY-PROOFS.md + perf-bench infra + offline-replay carve-out + drop-bundle predecessor-CID chain + MAL-BIND-K-PK doc + cite-drift cross-language config-mirror + per-Kind discipline + MembershipSetPolicy generalization per §5) | mixed | NO | v1-beta-LB | F17, F26 | THREAT-MODEL.md + SECURITY-PROOFS.md + CRYPTO-CODEPOINTS.md + ENVELOPE-V1-SPEC.md + cite-drift scanner | HIGH | M3 E25 (consolidates U37+U38+U39+U40+C5+L10 U43+L11 U43+L11 U44+C3 §6.1+§6.5+§6.8+AtriumPolicy D1-D7) |
| **F26** | **MembershipSetPolicy generalization (D1-D7 per-Kind variants; KindPolicyAdminEntity typed enum; permissive_default per-Kind constructor)** | **LB** | **YES** | **v1-beta-LB** | F17 | benten-membership-set + MembershipSet-Spec | **MED-HIGH** | M3 §5 + AtriumPolicy `e90900b4` D1-D7 generalization + M2 KindPolicy enum |
| **F27** | **TransportConfig codepoint-reserve at v1-beta (TransportKind + SubInheritPolicy + TransportExtension enums; default-selection logic; production iroh-gossip + Willow + iroh-roq + iroh-live impl deferred Phase-4-Meta-Composing / Phase-5+)** | **LB (slot)** | **YES (slot)** | **v1-beta-CODEPOINT-RESERVE + v1-beta-LB (defaults + refusal-to-use)** | F17 | benten-engine + V1-FROZEN-INTERFACE.md + TRANSPORT-CONFIG.md + Compromise #53 | **MED-HIGH** | M6 §7.2 + §10 + §5.2 |
| **F-A1** (new this consolidation) | `remove_member` per-Kind contract doc-only sharpening (audit-deliverable docs name the Atrium-kick vs DeviceMesh-revoke semantic gap) | DEFERRABLE | NO | v1-beta-LB | F17 | MembershipSet-Spec + SECURITY-POSTURE.md per-Kind kick-table | HIGH | M4 §2.1 + this M-CONS Task 1 |
| **F-A2** (new this consolidation) | `MembershipEvent` typed-enum (per-Kind audit-event variants) | MED-HIGH | NO | v1-beta-LB | F17 | benten-membership-set + audit-log-doc | MED-HIGH | M4 §10.6 + this M-CONS Task 1 |

**Net count:** **27 numbered F-rows + 2 new amendments F-A1/F-A2 = 29 final rows.**
Vs M3's 25 E-rows + 2 absorbed = ~27; the +2 (F26 + F27) and the +2 (F-A1 + F-A2)
are the M-CONS sharpenings.

**Wave-day accumulation** (central; per F-row):
- F1-F16 + F22 + F23 + F24 (base + impl-engineering): in-baseline; net –0.6 simplification per M3
- F17 (MembershipSet primitive crate): **+3.5-4.5 wave-days** (Wave-MS-PRIMITIVE canary)
- F18 (DUAL-CID): in F17 wave; **–1.5 vs Q3 §7.2 baseline**
- F19 (generation): in F17 wave; **–2.5 vs U19+U20+U42-L11 baseline**
- F20 (ExecuteWorkflow): **+0.3**
- F21 (Sealed-Sender slot): **–0.2 codepoint registry**
- F25 (audit-deliverable pack): in F17 wave; **–2.0 net via Path-A.5 + MembershipSet**
- F26 (MembershipSetPolicy): **+1.0** (absorbs Wave-G's 5-7 days → folded into Wave-MS-PRIMITIVE)
- F27 (TransportConfig codepoint-reserve): **+3.5 wave-days** (M6 §10.1 v1-beta scope subtotal)
- F-A1 (doc): **+0.2**
- F-A2 (MembershipEvent enum): **+0.5**

**Cumulative delta: –4.3 to –5.5 (M3 central) + 3.5 (F27 M6) + 0.7 (F-A1+F-A2) ≈
–0.1 to –1.3 wave-days net of the ~80-101 baseline.** Modest net win; the bigger
wins are LOC + audit-page + cross-cutting consolidation. See §7 for full cost
re-estimate.

---

## §3 Task 3 — Final-final Compromise # registry (#31-ext + #32-#53 = 22 mints + 1 extension)

### §3.1 Canonical #45 collision resolution (final)

Per M3 §3 collision resolution + this consolidation's M5/M6 additions:

| Canonical # | Origin | Title |
|---|---|---|
| #45 | C3 §6.1 (chronological priority) | ML-KEM-768 MAL-BIND-K-CT/K-PK binding-properties |
| #46 | L10 #45 rename | HpkeMultiBase O(N) wire-cost asymptotic > 32 recipients (per-Kind: Atrium 32; DeviceMesh 5; SingleDevice 1) |
| #47 | L11 #45 rename | Collaborative-edit-via-re-drop accepted v1-beta trade-off |
| #48 | Q3 #45 rename + GENERALIZED | **MembershipSet-shape-leak** (shared_key compromise → MembershipSet-fingerprint forward+backward for that generation; per-Kind severity; recovery via fork) |
| #49 | Q3 §7.1 #46 rename | MembershipSet-member-acting-as-storage-host trust-boundary collapse |
| #50 | C3 §6.6 #46 rename | Permanence-stewardship dependency disclosure |
| #51 | C3 §6.9 #47 rename | Tauri NAPI-RS marshaling-boundary side-channels |
| **#52** (NEW M5) | M5 §9.2 item 6 | **MembershipSet-no-PCS-against-removed-members** — Benten Atrium MembershipSet does NOT provide forward-secrecy or post-compromise-security against removed members; semantic is fork-on-kick + recipient-set-exclusion; kicked member retains pre-kick K_Atrium and reads pre-kick content forever but cannot read post-kick content. Use-cases requiring stronger semantics deferred to post-v1-beta AtriumWithCGKA-variant per F13. |
| **#53** (NEW M6) | M6 §7 + §10 | **TransportConfig-codepoint-reserve-at-v1-beta** — `TransportConfig` wire shape frozen at v1-beta but production iroh-gossip / Willow / iroh-roq / iroh-live impl deferred to Phase-4-Meta-Composing / Phase-5+. Engine refuses `GossipPlusBlobs` / `Reserved` at v1-beta with `E_TRANSPORT_KIND_UNSUPPORTED`. |

### §3.2 Per-Compromise transformation summary

| # | MembershipSet transformation |
|---|---|
| #31-ext | SIMPLIFIED — per-MembershipSet-Kind retention-window language |
| #32 | UNCHANGED |
| #33-#34 | UNCHANGED (coercion + password-knowledge OUT-OF-SCOPE) |
| #35 | SIMPLIFIED — per-Kind retroactive-decryption framing |
| #36-#40 | UNCHANGED (RAM/TEE/physical/supply-chain OUT-OF-SCOPE) |
| #41 | RENAMED → "MembershipSet-Kind::DeviceMesh compromised-member-on-set" |
| #42 | UNCHANGED (Layer-C FS-gap; ML-PQ codepoint-reserve already named) |
| #43 | SIMPLIFIED — per-Kind metadata-leak disclosure |
| #44 | UNCHANGED |
| #45-#47 | as above |
| #48 | RENAMED + GENERALIZED |
| #49-#51 | as above (renames) |
| **#52** | NEW (M5 ratification) |
| **#53** | NEW (M6 ratification) |

**Total: 22 first-order mints + #31-extension = 23.** Up from 18-20 in the pre-MembershipSet
registry; the +2 net are Compromise #52 + #53 which surface invariants of the chosen
deferral postures (NOT new vulnerabilities, but explicit semantic-floor disclosures
audit firms need to see).

---

## §4 Task 4 — Final-final invariant set (5 first-order + 3 sub)

| Inv | Title | Trans | Composes-with |
|---|---|---|---|
| **Inv-15** | Signature-bundle-CID identifier discipline (Compromise #30 closure; 3-layer signature decomposition) | UNCHANGED — load-bearing at HEAD `2172cb6d` | Inv-16/17/18/19/20 |
| **Inv-16** | Envelope-layer-unification + codepoint-dispatch + AAD-binding + strict-decode + canonical-TLV + sender-DID + replay-window | SIMPLIFIED — phrasing references MembershipSet-realized envelope variants uniformly | Inv-15/18/20 |
| **Inv-17** | Hybrid-cryptography-mandatory floor (PQ + classical) | UNCHANGED | Inv-15/16/18 |
| **Inv-18a** | Codepoint-registry-discipline + IANA-disjoint range + CodepointLifecycle | SIMPLIFIED — registry row count reduces (4 fanout codepoints → 1 MembershipSetEncryption family with Kind discriminator + Sealed-Sender paired slot) | Inv-16 |
| **Inv-18b** | Metadata-disclosure paired-SealedSender-slot discipline (Am4: Sealed-Sender DEFAULT) | SIMPLIFIED — one paired-slot rule for MembershipSetEncryption family | Inv-16 |
| **Inv-18c** | CodepointLifecycle typed-state | UNCHANGED | Inv-16 |
| **Inv-19** | Encryption-substrate keying-function CRDT-input discipline (Path-A.5 + MembershipSet) | SIMPLIFIED — two clean cases: (a) immutable substrate inputs (Version-Node-CID, codepoint, BE encoding, TLV) OR (b) MembershipSet-shared-key whose generation is CRDT-resolved per Inv-20 | Inv-20, Path-A.5 |
| **Inv-20** (NEW per M3 §4.1) | **MembershipSet primitive invariant** — every Benten encrypt-to-N-recipients use site dispatches through single `MembershipSet` primitive with `MembershipSetKind` discriminator; provides (a) `shared_key: K_Set` distributed via multi-stanza-HPKE-Encap; (b) FORK-ONLY rotation; (c) per-stanza AAD-binding tuple `(codepoint, body-CID, sorted-member-DID-list, sender_did, stanza-index, member-key-generation, membership_set_id, membership_set_generation)`; (d) per-recipient unlinkability; (e) generation-CRDT-vector per-member-DID partition; (f) Path-A.5 K(V) discipline at API boundary (type-restricts payload to immutable Version-Node-CID); (g) **TransportConfig wire-format codepoint-reserve (M-CONS extension)**; (h) **homogeneous-per-Kind MemberKey variant (M-CONS extension per M4 §11.5)**. | NEW (M-CONS extended from M3 §4.1) | SUBSUMES Inv-19 clause-b; SPECIALIZES Inv-16 multi-recipient sub-clause; INSTANTIATES Inv-18a/b for MembershipSetEncryption codepoint family |

### §4.1 Inv-20 composition with Inv-19 (M-CONS verification)

M3 §4 explicitly notes Inv-20 SUBSUMES Inv-19 clause-b. Verify: Inv-19's two-clause
form is (a) immutable substrate / (b) MembershipSet-shared-key with CRDT-resolved
generation per Inv-20. Inv-20 clause-e (generation-CRDT-vector per-member-DID
partition) provides the closure for Inv-19 clause-b. **Composition is sound.**

### §4.2 New invariants to add beyond Inv-20?

**No.** Three candidates considered + rejected:
1. *Transport-config invariant.* The TransportConfig codepoint-reserve fits within
   Inv-20 clause-g; doesn't merit its own first-order invariant.
2. *Per-Kind admin governance invariant.* The KindPolicy enum + per-Kind admin-signing
   dispatch is structural-to-Inv-20 (clause-d AAD-binding includes member-key-generation;
   the admin sig flows through the MembershipAttestation field bound to Inv-15's
   3-layer signature decomposition). No new invariant needed.
3. *MembershipEvent typed-enum invariant.* F-A2 amendment is audit-doc + impl, not
   structural invariant. No new invariant needed.

**Final invariant set:** 5 first-order (Inv-15/16/17/18/19/20) + 3 sub-invariants of
Inv-18 (18a/b/c). Confidence: HIGH.

---

## §5 Task 5 — MembershipSetPolicy walkthrough (AtriumPolicy generalization)

Per M3 §5 + the M2 KindPolicy enum + Ben's AtriumPolicy ratification `e90900b4`.
The full D1-D7 walk with my-pred per decision so Ben can ratify in one pass.

| D# | Decision | Generalization | **my-pred** | Rationale |
|---|---|---|---|---|
| **D1** | `policy_version_at_seal: u32` in AAD (single u32; not full atrium_policy_cid) | SIMPLIFIED — same field name + semantic across all 3 Kinds; verifier-side `(membership_set_id, policy_version)` tuple resolution | **RATIFY AS-IS** (just rename "atrium" → "membership_set") | M3 verdict + M2 design + AtriumPolicy `e90900b4` D1 ratification carries over cleanly |
| **D2** | Hybrid grandfather rule (grandfather `valid_until` ceilings + new `refresh_required` applies forward on first verifier-touch) | UNCHANGED per-Kind semantic | **RATIFY AS-IS** | Matrix `m.room.history_visibility` precedent applies identically; no Kind-specific behavior |
| **D3** | `refresh_required` v1-beta-CODEPOINT-RESERVE + impl at Phase-4-Meta-Composing | UNCHANGED phase placement | **RATIFY AS-IS** | UX-coupled per-Kind impl scheduling identical |
| **D4** | Admin entity per-Kind: Atrium = explicit `admin_did: PolicyAdminDid`; DeviceMesh = user IS admin (degenerate `admin_did = principal_did` via DAK; user-keypair signs); SingleDevice = N/A (no policy beyond UCAN scope's own exp) | EXPANDED per-Kind via `KindPolicyAdminEntity` typed enum: `AtriumAdminDid(Did) \| DeviceMeshUserPrincipal(Did) \| SingleDeviceNone` | **RATIFY** — D4 per-Kind dispatch is the substantial generalization | Cleaner than minting a degenerate admin-DID-of-self for DeviceMesh; user-IS-admin via DAK matches the existing per-DID K_principal substrate at v1-FROZEN; SingleDevice has no group governance question |
| **D5** | Default policy = `permissive_default()` (no Atrium-level ceiling) | UNCHANGED per-Kind semantic; per-Kind constructor `MembershipSetPolicy::permissive_default(kind: MembershipSetKind)` | **RATIFY AS-IS** | No semantic change beyond per-Kind constructor signature |
| **D6** (deferred) | Refresh source mechanism = OCSP-style fresh-attestation pull | UNCHANGED per-Kind; per-Kind authority-resolution rule (Atrium = `refresh_attestation_authorities` set; DeviceMesh = user's principal-DID; SingleDevice = N/A) | **RATIFY DEFERRAL** with per-Kind authority-resolution rule pinned now (Phase-4-Meta-Composing impl) | Deferral preserves; the per-Kind authority rule is a 1-paragraph addendum in MembershipSet-Spec |
| **D7** (deferred) | Admin = single-DID or threshold-of-N | EXPANDED per-Kind — threshold-admin ONLY meaningful for Atrium-Kind (multiple potential admins); DeviceMesh = user IS sole-admin by construction; SingleDevice = N/A | **RATIFY DEFERRAL** with per-Kind variant pinned now | Threshold-of-user's-own-devices for DeviceMesh would be a device-recovery concern, not admin concern; cleanly bounded |

### §5.1 KindPolicyAdminEntity enum (M-CONS proposed; D4 generalization)

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum KindPolicyAdminEntity {
    /// Atrium: explicit admin DID + admin sig-pubkey
    AtriumAdminDid {
        admin_did: Did,
        admin_sig_pubkey: HybridSigPubkey,
    },
    /// DeviceMesh: user IS admin via DAK; admin sig is user's principal-keypair
    DeviceMeshUserPrincipal {
        user_did: Did,
    },
    /// SingleDevice: no admin entity (degenerate; UCAN-scope-only governance)
    SingleDeviceNone,
}
```

Bound into `MembershipSetPolicy.admin_entity: KindPolicyAdminEntity`. The field type
encodes the per-Kind admin semantic at the type system level; no runtime ambiguity.

### §5.2 MembershipSetPolicy crate placement

Per M3 §5 wave-folding implication: **MembershipSetPolicy lives INSIDE `benten-membership-set`
crate** (not a separate `benten-atrium-policy` crate). AtriumPolicy's planned Wave-G
(~5-7 wave-days at `e90900b4`) **folds into Wave-MS-PRIMITIVE** (the canary wave per §6).
Net: –1 wave dispatch; ~+1 wave-day per-Kind dispatch overhead absorbed within Wave-MS-PRIMITIVE.

### §5.3 my-pred summary for §5

**D1 / D2 / D3 / D5 / D6:** ratify-as-is (no Ben-call needed beyond the rename).
**D4:** ratify per-Kind via KindPolicyAdminEntity enum (substantive but cleanly typed;
my-pred Ben ratifies in 1 pass).
**D7:** ratify deferral + pin per-Kind variant (threshold-admin Atrium-Kind-only at v1-beta-CODEPOINT-RESERVE).

---

## §6 Task 6 — Wave-sequencing implications (final)

### §6.1 Wave 0 — Bypass-merge-reinstate tracked-doc PR cascade

**Ben pre-authorized.** Lands the tracked-doc PR cascade that's been queued.
Out-of-scope for this consolidator's wave-sequencing but listed for completeness.

### §6.2 Wave 1 — Wave-MS-PRIMITIVE (canary-first; required)

Per M3 §7.2 + `feedback_canary_first_parallel_implementation.md` discipline.

**Canary (sequential first; ~3.5-4.5 wave-days):** `benten-membership-set` crate lands
first. Contents:
- `MembershipSet { id, kind, members, shared_key, policy, generation,
  parent_membership_set_id, created_at_hlc, membership_attestation, transport_config }`
  struct (M2 §2.1 + M-CONS adds `transport_config: TransportConfig` per F27)
- 7-op API: `create / add_member / remove_member / encrypt_to_set / dedup_blind_cid /
  fork / update_policy`
- Per-Kind constructors: `new_atrium / new_device_mesh / new_single_device`
- `MembershipSetPolicy` (D1-D7 per-Kind via `KindPolicyAdminEntity` per §5)
- `TransportConfig` types per F27 (M6 §5.2 + §10)
- `MembershipEvent` typed-enum per F-A2
- `MultiStanzaHpkeEnvelope` shape per M2 §2.2
- Golden vectors + kani harness + property tests (per F25)
- NAPI bindings (`MembershipSetHandle` per F24 U34-expansion)
- TS mirror at `packages/engine/src/membership.ts` + `errors.generated.ts`

**Fan-out (4 parallel agents AFTER canary merges; ~5.5 wave-days total at parallel
clock-time of ~1.5 wave-days):**
- Sub-wave-MS-A: `benten-crypto-suite` migrates HpkeMultiBase usage to MembershipSet (~1 day)
- Sub-wave-MS-B: `benten-drop` migrates Drop-bundle multi-recipient to MembershipSet (~1.5 days)
- Sub-wave-MS-C: `benten-engine` migrates multi-device-key-wrap to MembershipSet (~1.5 days)
- Sub-wave-MS-D: `benten-id` (or wherever Atrium-membership lives) migrates K_Atrium-distribution to MembershipSet (~1.5 days)

**Wave-MS-FINAL (single agent; ~0.5 day):** cross-crate integration test + final
goldens + TS-side full integration.

**Total Wave-MS-PRIMITIVE: ~9-10 wave-days (M3 estimate confirmed).**

### §6.3 Subsequent waves under MembershipSet unification

Per M3 §7.3:

- **Wave-C (Layer-C HPKE-multi-stanza):** SHRINKS by ~–1.5 wave-days (HpkeMultiBase is
  now Sub-wave-MS-A; Wave-C ships Layer-C HPKE-mode-base + Compromise #42 disclosure
  + single-recipient drop flow).
- **Wave-D (Layer-D DAK substrate + device-link + remote-permission + multi-device-key-wrap):**
  PARTIAL SHRINK by ~–1.75 wave-days (multi-device-key-wrap is Sub-wave-MS-C; DAK +
  device-link + remote-permission stay).
- **Wave-G (AtriumPolicy):** DELETED; folds into Wave-MS-PRIMITIVE per §5.2. **–5-7
  wave-days.**
- **Wave-Q3 (Q3 Option D triple-CID + K_Atrium + dedup_scope):** ABSORBED into
  Sub-wave-MS-D. **–5.75 wave-days.**
- **Wave-L11-CRDT (L11 U41-U44 + Inv-19):** PARTIALLY ABSORBED. **–4.5 wave-days.**

### §6.4 New Wave — Wave-MS-TRANSPORT (TransportConfig codepoint-reserve)

Per M6 §10.1 v1-beta scope deliverables (10 items; ~3.5 wave-days). **Dispatches AFTER
Wave-MS-PRIMITIVE** (depends on MembershipSet struct existing).

Composition:
- `TransportConfig` types per M6 §5.2 (in `benten-engine` per M6 §10.1 item 1)
- Default-selection logic per M6 §6.1
- `E_TRANSPORT_KIND_UNSUPPORTED` error code per §3.5g cross-language rule-mirror
- RED-PHASE staged-pin for production gossip path per pim-12 §3.6e
- V1-FROZEN-INTERFACE.md §15.6 row per M6 §10.4

**Wave-MS-TRANSPORT cost: ~3.5 wave-days (M6 §10.1 subtotal).**

### §6.5 Total wave-sequencing summary

| Wave | Estimated wave-days | Net Δ vs baseline |
|---|---|---|
| Wave 0 (tracked-doc cascade) | — (Ben pre-authorized) | 0 |
| Wave-MS-PRIMITIVE (canary + 4 parallel + final) | ~9-10 | **+9.5 NEW + –12.5-14.75 absorbed = –3-5 net** |
| Wave-MS-TRANSPORT (codepoint-reserve only) | ~3.5 | +3.5 NEW |
| Wave-C (Layer-C single-recipient drop flow) | reduced | –1.5 |
| Wave-D (DAK + device-link + remote-permission) | reduced | –1.75 |
| Wave-G (AtriumPolicy) | DELETED | –5 to –7 |
| Wave-Q3 (Q3 Option D) | absorbed into MS-D | –5.75 |
| Wave-L11-CRDT | partial | –4.5 |
| **Net wave-day delta** | | **central –0.1 to –1.3 (M-CONS); brackets –4 to +2 with TransportConfig accounting** |

**Net wave-count Δ:** +2 NEW waves (MS-PRIMITIVE + MS-TRANSPORT), 1 wave DELETED
(Wave-G), 3 waves shrink (Wave-C + Wave-D + Wave-L11-CRDT). **Net wave-count Δ = +1.**

### §6.6 Canary discipline

**Wave-MS-PRIMITIVE MUST go canary-first** because `benten-membership-set` is the
owner of the new primitive API the 4 fanout sites consume. Parallel dispatch of all
4 sub-waves without canary risks 4-way merge-conflict on the primitive's API shape.

---

## §7 Task 7 — Cost re-estimate

### §7.1 Baseline → final

- **Baseline (per critique-round + consolidator §6.5):** ~80-101 wave-days
- **M3 central (–4.3 to –5.5):** ~74.5-96.7 wave-days
- **M5 delta (+0; no CGKA at v1-beta):** ~74.5-96.7 wave-days (unchanged)
- **M6 delta (+3.5 wave-days for TransportConfig codepoint-reserve):** ~78-100.2 wave-days
- **M-CONS F-A1 + F-A2 (+0.7):** ~78.7-100.9 wave-days

**Final central estimate: ~78-100 wave-days.** Net delta vs baseline: **–2 to –1 wave-days**
central; the M3 conservative bracket of –1 to –3 brackets to **±2 wave-days against
baseline** when M6 + F-A1/F-A2 are factored in. **NET-EFFECTIVELY-NEUTRAL on wave-days;
substantial net-positive on LOC + audit-page + cross-cutting consolidation.**

### §7.2 LOC delta

Per M3 §6.4 baseline ~3500-4500 LOC across 4 fanout sites + new crates:
- New `benten-membership-set` crate: ~1500-2000 LOC (M2 estimate)
- New `TransportConfig` types in benten-engine: ~700 LOC (M6 §10.1)
- Consolidation savings across 4 fanout sites: ~500-900 LOC

**Net LOC: ~3700-4400 (M3) + 700 (M6) = ~4400-5100. Δ vs baseline: ~+0 to +600 LOC**
in absolute terms; but the new LOC is in DEDICATED + CONSOLIDATED + TYPED crates
(better audit-readability per LOC than the 4-fanout-site-parallel baseline).

### §7.3 Audit-page delta

Per M3 §6.4: –8 to –12 audit pages via per-Kind-parameterized-row collapse.
Plus M6 +1-2 pages for TRANSPORT-CONFIG.md + Compromise #53.
Plus F25 inheritance: SECURITY-PROOFS.md + THREAT-MODEL.md gain MembershipSet sections
but lose 4-parallel-rows. Net ~–6 to –10 pages.

### §7.4 Cross-cutting consolidation wins (NOT counted above)

- 4 fanout codepoints → 1 MembershipSetEncryption family with Kind discriminator (Inv-18a)
- 4 parallel implementations → 1 substrate crate
- 4 amendment-clusters → 1 set of E-rows (M3 –17 rows registry text reduction)
- AtriumPolicy + MembershipSetPolicy unified (Wave-G folded)
- Q3 Option D K_Atrium key-management absorbed into MembershipSet shared_key
- Path-A.5 K(V) discipline enforced at MembershipSet API boundary (compile-time)
- DUAL-CID-on-wire clarified (M2 §9); plaintext_cid_local stays local-only

**The wave-day delta is modest; the structural consolidation is substantial.**

---

## §8 Task 8 — R0 plan-doc skeleton (F-full)

Per the brief's 17 sections. Structure ONLY; the R0-author produces the substantive
content per ADDL pipeline.

```markdown
# F-full R0 plan-doc — Phase-4-Meta-Core (MembershipSet-unified)

## §1 Architectural framing
- 4-Layer model: Layer-A vault / Layer-B per-Node AEAD / Layer-C encrypt-to-recipient /
  Layer-D DAK + multi-device-key-wrap + remote-permission-call
- MembershipSet primitive as the central novel structural addition (§2)
- v1-beta freeze posture; codepoint-reserve discipline; HARD RULE 12 disposition
- Ben-ratified decisions inventory (Q1 / Q2 / Q3-DUAL-CID / Q4 / Am4 / Path-A.5)
- This R0 supersedes the 9-eyes-consolidated-registry as the canonical scope

## §2 MembershipSet primitive
### §2.1 The primitive (typed-variant)
- `MembershipSetKind { Atrium, DeviceMesh, SingleDevice }` enum (EXACTLY-3; non_exhaustive
  REJECTED per §15.c HALT-AND-SURFACE-TO-BEN; future Garden = 4th arm requires Ben ratification)
- Struct shape per M2 §2.1 (canonical Rust)
- Canonical CBOR wire shape per M2 §2.2
- 7-op API per M2 §4 (create / add_member / remove_member / encrypt_to_set /
  dedup_blind_cid / fork / update_policy)
- Per-Kind specialization table (M2 §3 + §4)
- MemberKey typed-per-Kind variant (UserDid / DeviceDid / LocalDevice)
- Type-system enforcement of per-Kind invariants (M2 §5.4)
- Composition with 4-identity-concepts tree (M2 §6; CLAUDE.md #18)

### §2.2 New `benten-membership-set` crate (15th workspace crate)
- Crate boundary + dependency graph (depends only on benten-id + benten-crypto-suite + benten-core)
- Public type surface
- Operations + per-Kind dispatch
- Sibling primitives NOT in MembershipSet (per M2 §4 + §7): register_zone, freshness_window,
  capability-revocation, K(N), DAK, hybrid-sig keypairs
- NAPI surface (MembershipSetHandle per F24)
- TS-side mirror

### §2.3 Inv-20 minting (the MembershipSet primitive invariant)
- 8-clause invariant per §4 Inv-20 above
- Composition with Inv-15/16/17/18/19
- Enforcement plan (crate-discipline scanner + cite-drift extension + property tests)

### §2.4 Wire-format codepoint family
- `MEMBERSHIP_SET_ENCRYPTION = 0x6380` codepoint family with `MembershipSetKind: u8`
  discriminator (replaces 4 fanout codepoints per F8)
- `MembershipSetEncryptionSealedSender = 0x6390` paired slot per F21
- AtriumWithCGKA codepoint slot at v1-beta-CODEPOINT-RESERVE per F13 + M5

## §3 Layer-A vault design (K_principal store)
- DAK-protected per-device vault (Argon2id tier per F24 + L10 U41)
- K_principal-per-DID seam (per v1-FROZEN; M4 §4.3 substrate-correct)
- Rotation cascade (M1c §4.2 + L9-A4 generation tracking)
- Composition with MembershipSet: DeviceMesh-Kind K_Set = K_principal; Atrium-Kind
  K_Set = CSPRNG (per M2 §3.1 / §3.2)

## §4 Layer-B per-Node AEAD (Path-A.5 immutable Version-Node-CID)
- K(N) derivation chain (path-tagged per M1c §6; Spike-E Interpretation-B)
- K(root) info-tag per-Kind prefix (`root:atrium:` vs `root:principal:`; M2 §8.1
  cross-domain separation)
- K(N) keys to immutable Version-Node-CID per Path-A.5 (M2 §8.3; M3 L11 simplification)
- MembershipSet enforces Path-A.5 at API boundary (compile-time typed payload)
- generation binds into K(root) info-tag (M2 §8.4; F19)
- Compromise #31 reach + per-Kind retention-window framing (#31-ext per §3)

## §5 Layer-C encrypt-to-recipient drops
- HPKE-mode-base (RFC 9180) + ML-KEM-768-X25519 hybrid KEM (libcrux-ml-kem per Q1)
- Multi-stanza MembershipSetEncryption envelope (replaces HpkeMultiBase per F17;
  absorbs L9-A1 / U17)
- Per-stanza AAD-binding tuple (codepoint, body-CID, sorted-member-DID-list,
  sender_did, stanza-index, member-key-generation, membership_set_id,
  membership_set_generation) per Inv-20 clause-c
- MultiRecipientSealing (per M5 rename from CGKA-LITE)
- Per-MembershipSet recommended-max-recipients table (#46: Atrium 32; DeviceMesh 5;
  SingleDevice 1)
- Sealed-Sender DEFAULT per Am4 (drop non-sealed-sender parallel codepoint)

## §6 Layer-D DAK + multi-device-key-wrap + remote-permission-call
- DeviceAttestation chain (CLAUDE.md #17; per-device sig pubkey + parent user-DID sig)
- DAK derivation per L10 U41 Argon2idParameterTier (only DeviceMesh + SingleDevice Vault)
- Multi-device-key-wrap envelope = MembershipSetEncryption for DeviceMesh-Kind
  (per F17; V1-FROZEN-INTERFACE §15.5 preserved as Kind-specific instance)
- RemotePermission envelope per U21 / F20 (per-Kind ExecuteWorkflow sub-arms)
- HPKE-11-KE commit per Q4 (JOSE-emit adapter NAMED-deferred to V1-FROZEN-INTERFACE-DEFERRED.md)

## §7 Wire format
### §7.1 EncryptedEnvelope
- EnvelopePayload `#[non_exhaustive]` enum per F9 (with MembershipSetEncryption variant)
- BindingContext `#[non_exhaustive]` per F10 (with MembershipSetSeal variant)
- aad_version: u8 prefix per F14
- DAG-CBOR outer framing per F23 (strict-deterministic-decode per C3 §6.3 R3)

### §7.2 DUAL-CID model (Ben's Q3 ratification)
- `envelope_blob_cid` + `plaintext_cid_membership_set` on wire (DUAL)
- `plaintext_cid_local` LOCAL-ONLY (engine-internal index; never on wire) per M2 §9
- HMAC-SHA256(K_MembershipSet, plaintext_cid_local) per F18
- Recipient computes BLAKE3-of-plaintext locally as impl detail

### §7.3 Amendments F1-F27 + F-A1/F-A2
- Full final-final amendment registry per §2 above (this M-CONS doc)

## §8 MembershipSetPolicy (D1-D7 per-Kind)
- D1 policy_version_at_seal in AAD (per §5)
- D2 hybrid grandfather rule
- D3 refresh_required v1-beta-CODEPOINT-RESERVE; impl Phase-4-Meta-Composing
- D4 KindPolicyAdminEntity typed enum (per §5.1)
- D5 permissive_default per-Kind constructor
- D6 OCSP-style refresh source mechanism (per-Kind authority-resolution; impl
  Phase-4-Meta-Composing)
- D7 threshold-admin Atrium-Kind-only at v1-beta-CODEPOINT-RESERVE

## §9 TransportConfig (codepoint-reserve at v1-beta)
- `TransportConfig` struct + `TransportKind` + `SubInheritPolicy` + `TransportExtension`
  enums per M6 §5.2
- Codepoint-dispatch layout per M6 §10.3
- Default-selection logic per M6 §6.1
- `E_TRANSPORT_KIND_UNSUPPORTED` per F27 + §3.5g cross-language mirror
- Production iroh-gossip / Willow / iroh-roq / iroh-live impl deferred per F27 + #53
- V1-FROZEN-INTERFACE.md §15.6 row per M6 §10.4

## §10 ml-kem crate choice (libcrux-ml-kem per Q1)
- libcrux-ml-kem (Cryspen formally-verified) vs oqs-rs trade-off (per Q1 ratification)
- Bernstein-Persichetti Decap CT mitigation per F6 + Compromise #32
- ML-KEM-768 MAL-BIND-K-CT/K-PK binding-properties per F25 + Compromise #45

## §11 docs/THREAT-MODEL.md skeleton
- Per-MembershipSet-Kind threat-model rows (M2 §5):
  - Atrium multi-user (T-A1..T-A6)
  - DeviceMesh single-user (T-D1..T-D4)
  - SingleDevice degenerate (T-S1..T-S3)
- Cross-Kind composition + storage-host insider threats (#49)
- Coercion / xkcd-538 out-of-scope per #33
- MembershipSet-shape-leak disclosure per #48
- MembershipSet-no-PCS-against-removed-members disclosure per #52

## §12 docs/SECURITY-PROOFS.md skeleton (per C5)
- Permanence triangle (per C3 §6.5 R5)
- MembershipSet primitive security properties (Inv-20 clauses a-h)
- Path-A.5 K(V) immutability proof sketch
- AAD-binding tuple injectivity (kani harness per F25)
- per-recipient unlinkability (Inv-20 clause-d)

## §13 docs/CRYPTO-CODEPOINTS.md skeleton (per C1 packaging)
- MembershipSetEncryption codepoint family table per F8
- Sealed-Sender paired-slot per F21 + Inv-18b
- AtriumWithCGKA codepoint-reserve per F13 + M5
- TransportKind + TransportExtension codepoint layout per M6 §10.3 + F27
- IANA-disjoint range per F8

## §14 docs/ENVELOPE-V1-SPEC.md skeleton (per C1 packaging)
- EnvelopePayload variants (incl. MembershipSetEncryption)
- BindingContext variants (incl. MembershipSetSeal)
- DUAL-CID-on-wire shape per F18
- DAG-CBOR canonical encoding per F23
- AAD layout per F4 + F14
- Replay-window per-Kind exclusion table per F5

## §15 Wave-sequencing (per §6 above)
- Wave 0: tracked-doc cascade (Ben pre-authorized)
- Wave-MS-PRIMITIVE (canary + 4 parallel + final) ~9-10 wave-days
- Wave-MS-TRANSPORT (codepoint-reserve) ~3.5 wave-days
- Reduced/folded waves (C/D/G/Q3/L11-CRDT)

## §16 Cost estimate (per §7 above)
- Wave-days: ~78-100 (vs baseline ~80-101); net –2 to –1 central
- LOC: ~4400-5100 (consolidated)
- Audit pages: ~70-90 (-6 to -10 vs baseline)
- Cross-cutting consolidation wins (un-counted)

## §17 Decisions recorded (Q1-Q5 + AtriumPolicy D1-D7 + MembershipSet kind decisions)
- Q1: libcrux-ml-kem (ratified)
- Q2: BE codepoint endianness migration (ratified; in aead.rs)
- Q3: DUAL-CID simplification (ratified; per F18)
- Q4: HPKE-11-KE commit now; JOSE-emit adapter NAMED-deferred (ratified)
- Q5: tbd (if any open; surface to Ben)
- Am4: Sealed-Sender DEFAULT (ratified; per F21)
- Path-A.5 (ratified; Path-B DISAGREE-WITH-REASONING per HARD RULE 12 clause-c)
- AtriumPolicy D1-D7 (ratified per §5; D6/D7 deferred with named destination)
- MembershipSet Kind decisions:
  - 3-arm EXACTLY-3 enum (Atrium/DeviceMesh/SingleDevice)
  - Forkability Atrium-only (runtime-reject for other Kinds; sibling-trait reserve v1.x)
  - DUAL-CID-on-wire; plaintext_cid_local LOCAL-ONLY
  - MultiRecipientSealing (not CGKA-LITE) naming
  - AtriumWithCGKA codepoint-reserve at v1-beta
  - TransportConfig codepoint-reserve at v1-beta
  - F-A1 + F-A2 amendments (M-CONS additions)
```

---

## §9 Task 9 — Open Ben-call decisions + my-pred per

For each: **my-pred** + rationale. Walking the brief's enumeration.

### §9.1 M5's CGKA-LITE → MultiRecipientSealing rename

**my-pred:** RATIFY. The "CGKA" naming was misleading (promises PCS/FS we don't
deliver); MultiRecipientSealing is what we ship. Audit-deliverable + CRYPTO-CODEPOINTS.md
+ SECURITY-POSTURE.md docs all use MultiRecipientSealing terminology.

**Confidence:** HIGH. M5 §1.2 critical correction (Cryptree-not-a-CGKA) + §9.1 final
recommendation are well-grounded. Truth-in-naming.

### §9.2 M6's codepoint-reserve transport scope-decision

**my-pred:** RATIFY (Ben already softly signaled OK on M6's recommendation per the
brief's framing). Option A (codepoint-reserve only) at v1-beta; production iroh-gossip
+ Willow + iroh-roq + iroh-live impl deferred to Phase-4-Meta-Composing / Phase-5+.

**Confidence:** HIGH. M6 §7.2 four-reason elegant-shape pass + reversible-up property
+ Spike C-evidenced footguns. Ben's prior soft-signal aligns.

### §9.3 MembershipSetPolicy D1-D7 walkthrough (per §5)

**my-pred per decision:**
- **D1:** RATIFY AS-IS (rename atrium → membership_set)
- **D2:** RATIFY AS-IS
- **D3:** RATIFY AS-IS (deferral)
- **D4:** RATIFY per-Kind via KindPolicyAdminEntity enum
- **D5:** RATIFY AS-IS (per-Kind constructor)
- **D6:** RATIFY DEFERRAL + pin per-Kind authority-resolution rule now
- **D7:** RATIFY DEFERRAL + pin per-Kind variant (Atrium-Kind-only threshold)

**Confidence:** MED-HIGH. D4 is the most-substantive per-Kind generalization; the
KindPolicyAdminEntity typed enum is cleaner than minting degenerate admin-DIDs of
self for DeviceMesh.

### §9.4 New Compromise mints from M5/M6 (renumbered)

**my-pred:**
- **Compromise #52 (MembershipSet-no-PCS-against-removed-members per M5):** RATIFY MINT.
  Audit-deliverable docs explicitly state the semantic-floor. SECURITY-POSTURE.md +
  THREAT-MODEL.md rows.
- **Compromise #53 (TransportConfig-codepoint-reserve-at-v1-beta per M6):** RATIFY MINT.
  Audit-deliverable docs explicitly state the deferral posture + the E_TRANSPORT_KIND_UNSUPPORTED
  refusal behavior at v1-beta. SECURITY-POSTURE.md + TRANSPORT-CONFIG.md rows.

**Confidence:** HIGH. Both are explicit-semantic-floor disclosures (NOT new
vulnerabilities); audit firms need to see them.

### §9.5 Inv-20 minting

**my-pred:** RATIFY MINT. M3 §4.1 phrasing + M-CONS §4 8-clause extension (clauses
g + h added for TransportConfig + homogeneous-per-Kind-MemberKey). Final invariant
set: 5 first-order (Inv-15/16/17/18/19/20) + 3 sub (Inv-18a/b/c).

**Confidence:** HIGH on the mint; MED-HIGH on the exact 8-clause phrasing (M2 will
refine alongside the primitive's final API).

### §9.6 Wave-MS-PRIMITIVE canary-first dispatch decision

**my-pred:** RATIFY. Per `feedback_canary_first_parallel_implementation.md` discipline.
Canary `benten-membership-set` crate FIRST; 4 parallel sub-waves AFTER canary merges;
final integration agent. ~9-10 wave-days total.

**Confidence:** HIGH. Canonical canary-first pattern; G16-A + G21-T1 precedents.

### §9.7 New amendments F-A1 + F-A2 (M-CONS additions)

**my-pred:**
- **F-A1 (`remove_member` per-Kind contract doc-only sharpening):** RATIFY ADD. Bounded
  doc-only sharpening surfacing M4 §2.1's semantic gap (Atrium-kick vs DeviceMesh-revoke).
- **F-A2 (`MembershipEvent` typed-enum):** RATIFY ADD. Bounded ~50 LOC + audit-doc; folds
  into Wave-MS-PRIMITIVE canary. Surfaces per-Kind audit-event shape per M4 §10.6 +
  §2.3.

**Confidence:** MED-HIGH on both. F-A2 may be over-engineered if Ben prefers a simpler
single MembershipEvent struct with kind-tag; my-pred is the typed-enum is worth the
~50 LOC for compile-time safety + audit-readability.

### §9.8 Sibling-trait vs inherent-method placement for `fork` + `dedup_blind_cid`

**my-pred:** KEEP INSIDE PRIMITIVE (M2's call stands at v1-beta). Reserve sibling-trait
extraction as v1.x refactor option per M3 §8.4. Rationale: uniform 7-op primitive
cognitive model + bounded runtime-reject for DeviceMesh/SingleDevice fork is cheap;
if ergonomics prove painful at call sites, v1.x refactor is mechanical.

**Confidence:** MED. Both shapes are defensible; this is a stylistic call. M3 §9.2
flags as "LOW confidence sibling-traits assumption"; M-CONS recommends keeping M2's
shipped shape.

### §9.9 `Atrium` vs `AtriumFork` Kind naming

**my-pred:** KEEP `Atrium` (M2's call stands). Audit-deliverable docs + SECURITY-POSTURE.md
+ Compromise #48 + Compromise #52 disclosures spell out the fork-semantic clearly.
Naming-as-documentation is weaker than naming-as-type-constraint; the disclosures
carry the semantic.

**Confidence:** MED-HIGH. M5's `AtriumFork` rename has merit for truth-in-naming, but
the renaming alone doesn't change behavior — the Compromise #52 row + docs are the
load-bearing semantic-floor surface.

### §9.10 Summary table of open Ben-call decisions

| Decision | my-pred | Confidence |
|---|---|---|
| §9.1 MultiRecipientSealing rename | RATIFY | HIGH |
| §9.2 TransportConfig codepoint-reserve | RATIFY | HIGH |
| §9.3 D1-D7 (5× as-is + D4 per-Kind enum + D6/D7 deferral with per-Kind pin) | RATIFY-BUNDLE | MED-HIGH |
| §9.4 Compromise #52 + #53 mints | RATIFY-BOTH | HIGH |
| §9.5 Inv-20 mint (8-clause M-CONS extension) | RATIFY | HIGH (mint) / MED-HIGH (phrasing) |
| §9.6 Wave-MS-PRIMITIVE canary-first | RATIFY | HIGH |
| §9.7 F-A1 + F-A2 amendments | RATIFY-BOTH | MED-HIGH |
| §9.8 Sibling-trait vs inherent (fork + dedup_blind_cid) | KEEP INSIDE (M2 default) | MED |
| §9.9 Atrium vs AtriumFork name | KEEP `Atrium` | MED-HIGH |

**Net: 9 Ben-decisions; my-pred is ratify-bundle on 7, keep-default on 2. None require
arch-fork-class surfacing per `feedback_surface_arch_decisions_under_auth.md`. All
9 can be confirmed in one Ben-pass cold.**

---

## §10 Task 10 — Pattern-induction meta-findings

What does the M2-M6 panel reveal that individual specialists missed? Cross-panel
patterns worth codifying.

### §10.1 Pattern P1 — "Convergent shape across independent panelists" as a strong-signal

All 5 specialists, working in parallel without seeing each other's outputs, independently
converged on:
- Typed-variant primitive (vs pure-unification)
- Forkability Atrium-only
- DAK ≠ K_Set (per-device vs per-set categorical distinction)
- K_principal stays per-DID; K_Set HPKE-wraps on top
- DUAL-CID-on-wire; plaintext_cid_local local-only
- Per-Kind admin governance dispatch
- Plugin-DIDs + agent-DIDs NOT first-class members

**Pattern.** When N independent specialists converge on the same structural shape,
the convergence itself is corroborating evidence; the residual divergences are
stylistic (sibling-trait vs runtime-reject; naming) rather than substantive. **Codify:
when M1..M5-style parallel-specialist panels converge, treat the convergence as a
HIGH-confidence ratification signal (not just a MED-HIGH).** This sharpens the
`feedback_iterate_critical_reviews_to_convergence.md` discipline.

### §10.2 Pattern P2 — "The 4-site fanout collapse" is the load-bearing simplification

M1c §10.3 named it; M3 quantified it (~-2 to -3 wave-days from U17 + U19 + U25
elimination); M2 realized it (one primitive crate for 4 fanout sites). **The pattern
"multiple parallel codepoints for variants of the same shape collapse into one
codepoint family with kind-discriminator" recurs across the codebase.** Already
documented as `feedback_extra_reflection_pass_for_elegant_permanent_shape.md` exemplar.
**Sharpen:** the M3 § §2 amendment-by-amendment walk is a model methodology for
post-critique-round consolidation; consider codifying this as a phase-close
methodology pattern.

### §10.3 Pattern P3 — Truth-in-naming as a first-class consolidation lever

M5's "CGKA-LITE was wrong naming; it promises PCS/FS we don't deliver" insight
revealed that the original name was actively misleading specialists. **The rename
to MultiRecipientSealing simplified the M3 analysis substantially** (M5's correction
let M3 + M-CONS treat the question as "no-CGKA-at-v1-beta + codepoint-reserve" rather
than "thin-CGKA-wrapper" — much cleaner). **Pattern:** when a name promises properties
not delivered, the misleading name leaks into downstream analyses + induces incorrect
amendments. **Codify:** as a phase-close lens, audit every term in the consolidated
registry against the "does this name promise what we deliver?" question; rename
proactively before audit firms ask.

### §10.4 Pattern P4 — Codepoint-reserve discipline as wave-day-de-risking

M5's CGKA codepoint-reserve + M6's TransportConfig codepoint-reserve both adopt
the same posture: freeze the wire shape at v1-beta but defer impl. **The codepoint-
reserve mechanism is the canonical lever for "we want optionality without v1-beta
scope expansion."** Pattern recurs: Sealed-Sender slot per Inv-18b; MLS-PQ slot
per F13; AtriumWithCGKA per M5; TransportExtension slots per M6. **Codify:** every
phase-close R6 R3 should systematically ask "is this a codepoint-reserve-able
deferral?" — the discipline saves wave-days without losing future optionality.

### §10.5 Pattern P5 — Cross-panel red-team reconciliation reveals "already-handled"

M4 wrote 5 BREAKs against M2-PENDING; M-CONS reconciliation found 3 of the 5 were
already-handled-as-designed by M2 (the others led to F-A1 + F-A2 sharpening
amendments). **Pattern:** when red-team and primitive-design specialists run in
parallel without seeing each other's outputs, the red-team's BREAK list often
over-states the gap because the design was already addressing the concerns. **The
M-CONS reconciliation step is essential** — without it, the panel would have
appeared to mint 5 substantive amendments when only 2 (F-A1 + F-A2) are real.
**Codify:** post-parallel-specialist consolidation MUST include red-team-vs-design
reconciliation; otherwise amendment-count is over-stated.

### §10.6 Pattern P6 — Q3-style "Option D" anticipates MembershipSet (early-hint pattern)

Per M3 §1: "Q3 §5.6 Option I framing already anticipates [MembershipSet] by another
name; ratifying MembershipSet realizes the elegant-shape Q3 named." **Pattern:** the
right consolidation is often visible early (Q3 specialist saw it in passing) but
requires explicit panel-level ratification to land. **Codify:** when individual
lens-reviews surface "this would be cleaner under X" comments, escalate to phase-close
extra-reflection-pass per `feedback_extra_reflection_pass_for_elegant_permanent_shape.md`
PROACTIVELY (don't wait for a critique round to surface it).

### §10.7 Pattern P7 — Cryptree-confusion as the "naming-imports-folk-knowledge" hazard

M5 §1.2 + §3.6 named that Ben's "Cryptree forkable by design" intuition conflated
two things (Atrium-fork semantic ≠ Cryptree mechanism). **Pattern:** when a user-
identified candidate technology gets surface-matched on a single property word
("forkable"), the deeper semantic mismatch can be invisible. M5's correction was
load-bearing; without it the panel might have spent wave-days on Cryptree adoption
that solved a different problem. **Codify:** for any user-identified candidate
technology, run an explicit "does this primitive actually solve the named problem?"
audit BEFORE estimating adoption cost.

### §10.8 Pattern P8 — Path-A.5 API-boundary enforcement as compile-time invariant

M2 §8.3 + M3 §2.12 row #86 + Inv-20 clause-f: MembershipSet's
`encrypt_to_set(payload: VersionNodeCid, ...)` type-restricts the payload to
immutable Version-Node-CID at the API boundary. **Pattern:** when an invariant
can be moved from "runtime convention" to "type-system constraint", do so. The
compile-time enforcement of Path-A.5 K(V) keying via MembershipSet's API signature
discharges an entire class of L11-style runtime-check-might-be-missed hazards.
**Codify:** at every R6 R3, ask "is this invariant type-system-enforceable?" — if
yes, the type-system enforcement is strictly stronger than runtime convention.

### §10.9 Pattern P9 — Per-Kind dispatch cost is bounded + predictable

M3 §2 quantified per-Kind sub-arm cost as "~12 sites; mostly mechanical but not
free". M2 §5.4 + §6.5 showed the type-system makes per-Kind dispatch tractable
(KindPolicy enum + MemberKey typed enum). M6 §5 + §6 added per-Kind TransportConfig
defaults. **Pattern:** typed-variant per-Kind dispatch has a bounded ~+0.3 wave-day
cost per affected amendment that gets EXPANDED — predictable + mechanical. **Codify:**
when designing a primitive that needs to handle N variants, the typed-enum-per-N
shape is generally preferable to the runtime-tag-on-uniform-shape shape; the +0.3-day
per-amendment cost is bounded.

### §10.10 Pattern P10 — "Brief framing as load-bearing scope-bound"

The brief's framing "MembershipSet unification = the 12-primitives-irreducible
commitment" set the panel's expectation toward a dramatic structural shift.
M3 + M-CONS verdict: NET-ELEGANCE-WIN (MODEST), not dramatic. **Pattern:** when
a brief framing promises dramatic transformation, the actual consolidation is
often more modest; honest-reckoning verdicts ("MODEST" not "DRAMATIC") are
load-bearing for setting Ben's expectations. **Codify:** specialist verdicts
SHOULD explicitly bracket "what I found" against "what the brief framing
predicted" — the gap is itself a signal.

---

## §11 Self-assessment + confidence per finding

| Finding-class | Confidence | Reasoning |
|---|---|---|
| §1 Reconciliation of M4 BREAKs against M2 design | **HIGH** | Every BREAK traced to M2's specific design choice; 3 of 5 already-handled-as-designed; 2 led to bounded sharpening amendments |
| §1.7 Cross-panel composition (10-axis table) | **HIGH** | Each axis verified across panelists; agreement-row dominates |
| §2 Final-final amendment registry (F1-F27 + F-A1/F-A2) | **HIGH** on rows; **MED-HIGH** on the F26/F27/F-A1/F-A2 additions | F1-F25 are direct adoption of M3 E1-E25; F26 + F27 + F-A1 + F-A2 are M-CONS additions |
| §3 Compromise renumbering + #52 + #53 mints | **HIGH** | Mechanical renumbering per M3 §3; #52/#53 are explicit semantic-floor disclosures (not vulnerabilities) |
| §4 Invariant set (Inv-20 8-clause extension) | **HIGH** on mint; **MED-HIGH** on the 8-clause phrasing | Extension of M3 §4.1 to add TransportConfig + homogeneous-per-Kind-MemberKey clauses |
| §5 MembershipSetPolicy D1-D7 + KindPolicyAdminEntity | **MED-HIGH** | D4 + D7 per-Kind expansion is the substantive change; D1/D2/D3/D5/D6 ratify-as-is |
| §6 Wave-sequencing (canary + MS-TRANSPORT + reduced waves) | **HIGH** on shape; **MED** on per-wave-day estimates | Wave-sequencing shape converged across M3 + M6; per-wave estimates inherit M3's MED-confidence bracket |
| §7 Cost re-estimate (~78-100 wave-days; –2 to –1 central) | **MED** | M3's central –4.3 to –5.5 + M6's +3.5 + M-CONS's +0.7 compounds bracketwise; honest hedge |
| §8 R0 plan-doc skeleton (17 sections) | **HIGH** on structure | Brief enumerated the 17 sections; this is structural-only |
| §9 Open Ben-call decisions + my-pred per (9 decisions) | **MED-HIGH** on my-pred | 7 of 9 are RATIFY-bundle; 2 are KEEP-DEFAULT; no arch-fork-class surfacing needed |
| §10 Pattern-induction (10 patterns) | **MED-HIGH** | Each pattern is grounded in specific cross-panel observation; codification recommendations are bounded |

### §11.1 What I could be wrong about

- **Wave-day arithmetic may be off by ±2 days.** Many small per-amendment estimates
  compound; if the +3.5 M6 estimate is conservative or the –4.3-5.5 M3 estimate is
  optimistic, the central could shift to –4 or +2.
- **F-A2 MembershipEvent typed-enum may be over-engineered.** Ben might prefer a
  simpler single struct with kind-tag; my-pred is the typed-enum is worth the ~50
  LOC for compile-time safety + audit-readability.
- **Sibling-trait extraction for `fork` + `dedup_blind_cid`** may be cleaner than
  M-CONS's keep-inside-primitive recommendation; M3 §9.2 flags as LOW-confidence;
  v1.x refactor is named-deferral.
- **KindPolicyAdminEntity enum** may not be the cleanest D4 shape; M3 §5 also
  considered a flat `signed_by: MembershipSetPolicyAdminEntity` with per-Kind
  variants; M-CONS picks the typed-enum based on M2 KindPolicy precedent.
- **F-A1 + F-A2** may be M-CONS over-reach if M2 was always-going-to-handle these.
  If they're already in M2's draft impl, F-A1/F-A2 fold into Wave-MS-PRIMITIVE
  canary at zero marginal cost.

### §11.2 What this consolidation does NOT cover

- **Authoring the full R0 plan-doc** (skeleton only per §8; M-R0 owns the
  substantive content).
- **Critique-round outputs** (M-C1 elegant-shape / M-C2 composability / M-C3
  fresh-eyes review this M-CONS output).
- **Final wave-day adjudication** after the critique round.
- **Per-amendment doc-prose** (each F-row needs to land in the appropriate audit-
  deliverable doc per the wave-day budget).
- **Migration path for existing production callers** of AtriumHandle to MembershipSet-
  based shape (M2 §11.3 flagged; bounded by additive-codepoint discipline).

### §11.3 Convergence verdict (process-level)

The M-CONS reconciliation found **STRONG cross-panel convergence** on the substantive
design (10/10 cross-axis agreement-or-resolved-divergence per §1.7 table). The
residual divergences (sibling-trait vs runtime-reject; Atrium vs AtriumFork name)
are stylistic; both resolved without changing the substantive design. **The panel's
output is DECISION-READY for Ben** with the 9 my-pred ratifications surfaced
in §9.

---

## §12 Citations

### §12.1 5-panel artifacts (frozen SHAs)

- M2 primitive design: `phase-4-meta-core/membership-set-m2-primitive-design @ 6170980b`
  → `/tmp/panel/m2-primitive-design.md` (901 LOC; via `git show`)
- M3 amendment-transformation: `phase-4-meta-core/membership-set-m3-amendment-transformation @ 1ba3a4c3`
  → `/tmp/panel/m3-amendment-transformation.md` (833 LOC)
- M4 red-team: `phase-4-meta-core/membership-set-m4-red-team-negative-findings @ 066785b5`
  → `/tmp/panel/m4-red-team-negative-findings.md` (764 LOC)
- M5 CGKA-candidate-survey: `phase-4-meta-core/membership-set-m5-cgka-candidate-survey @ 15819500`
  → `/tmp/panel/m5-cgka-candidate-survey.md` (533 LOC)
- M6 transport-configurability: `phase-4-meta-core/membership-set-m6-transport-configurability @ e5046c87`
  → `/tmp/panel/m6-transport-configurability.md` (531 LOC)

### §12.2 3-cataloger artifacts (frozen SHAs)

- M1a multi-device-sync: `phase-4-meta-core/membership-set-cataloger-m1a-multi-device-sync @ 3618e051`
- M1b Atrium-membership-sharing: `phase-4-meta-core/membership-set-cataloger-m1b-atrium-membership-sharing @ 1816ea60`
- M1c key-management: `phase-4-meta-core/membership-set-cataloger-m1c-key-management @ 50eb901d`

### §12.3 Background (per brief)

- 9-eyes consolidated registry: `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16`
- 5 critique-round outputs: c1 `3f5a4351` / c2 `9c548e5f` / c3 `79c99aa5` / c4 `6ea9718a` / c5 `8e374a9d`
- Q3-revisit Option D: `phase-4-meta-core/q3-revisit-option-d-community-lens @ 23f76e24`
- AtriumPolicy design: `phase-4-meta-core/atrium-policy-credential-validity-design @ e90900b4`
- Path-A vs Path-B specialist (Path-A.5 winner): `phase-4-meta-core/path-a-vs-path-b-specialist-review @ 5f50a028`

### §12.4 Tree-pinned (HEAD `2172cb6d`)

- `crates/benten-id/INTERNALS.md` — identity primitives
- `crates/benten-sync/INTERNALS.md` — Atrium transport + CRDT
- `crates/benten-crypto-suite/INTERNALS.md` — crypto-suite integration
- `crates/benten-engine/src/engine_sync.rs` — AtriumHandle 12 public methods
- `docs/V1-FROZEN-INTERFACE.md` §15 — frozen surfaces
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` — deferred-with-name destinations
- `docs/INVARIANT-COVERAGE.md` — Inv-14 + Inv-15 + Inv-16-mint
- `docs/SECURITY-POSTURE.md` — Compromise #22 + #23 + #30 + #31

### §12.5 CLAUDE.md baked-in + MEMORY.md disciplines

- CLAUDE.md baked-in #1 (12-primitive irreducibility); #5 (crypto-agility); #15
  (v1-beta interface freeze); #17 (multi-process + device-shape); #18 (4-identity-
  concepts + plugin trust); #19 (engine-level extensions vs app-level plugins)
- `feedback_canary_first_parallel_implementation.md` — Wave-MS-PRIMITIVE canary-first
- `feedback_engine_primitives_vs_application_layer.md` — composition vs primitive-extension framing
- `feedback_extra_reflection_pass_for_elegant_permanent_shape.md` — elegant-shape lens
- `feedback_iterate_critical_reviews_to_convergence.md` — sharpened by P1 (§10.1)
- `feedback_no_defer_HARD_RULE.md` — every M5/M6 deferral has NAMED destination
- `feedback_handoff_top_banner_re_orient.md` — honored at file top
- §3.5h pre-merge JSON validation + cite-drift discipline (every cite verified at author-time)
- §3.5g cross-language rule-mirror — TS-side MembershipSetKind + TransportKind mirrors named in F17 + F27

---

**End of M-CONS consolidator document.** Output to be reviewed by the M-C1 elegant-shape /
M-C2 composability / M-C3 fresh-eyes critique round, then handed to the R0-author for
the F-full plan-doc substantive content.
