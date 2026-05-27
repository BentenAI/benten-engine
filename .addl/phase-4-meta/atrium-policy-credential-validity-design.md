# AtriumPolicy — per-Atrium-configurable credential-validity design

**Authoring branch:** `phase-4-meta-core/atrium-policy-credential-validity-design`
**Author:** senior cryptographer + systems-architect lens (Claude Opus 4.7)
**Date:** 2026-05-27
**Origin:** Ben articulation 2026-05-27 — *"maybe how long credentials are valid for is a setting of the community so some could say valid forever while others say you have to revalidate regularly or something?"*
**Inputs ref-pinned at:**
- `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` (28 unified amendments + 13 Compromise mints + 3 unified invariants)
- L9 Atrium-integration lens `@ 1670aa03` (A1-A5 + forkability composition)
- L5 threat-model audit-readiness lens `@ 3f27f8e0` (T-01..T-25 + Compromise registry)
- Critique rounds c1 (3f5a4351) / c2 (9c548e5f) / c3 (79c99aa5) / c4 (6ea9718a) / c5 (8e374a9d)
- `docs/INVARIANT-COVERAGE.md` (Inv-1..Inv-15 + Inv-14 plugin-DID matrix) at HEAD `2172cb6d`
- `docs/SECURITY-POSTURE.md` Compromise #31 (forever-valid Drop bundles; OPEN at v1-beta+v1-GM)
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-17 / D-SS-1 / D-COVER-1 pattern

**Sibling artifact (not yet at HEAD; named as a future-loadable):** RATIFIED-sharing-and-confidentiality-2026-05-21.md referenced from the brief is not present on `main` at SHA `2172cb6d` (verified `find .addl -name "RATIFIED-sharing*"` empty); SECURITY-POSTURE.md "RATIFIED-S&C §R6" + Compromise #31 text are the surviving anchors and are quoted faithfully here.

---

## §1 Executive recommendation + confidence

### §1.1 One-line recommendation

**AtriumPolicy is a v1-beta-CODEPOINT-RESERVE + Phase-4-Meta-Composing-pre-v1-beta-tag implementation** — codepoint slot + AAD field lock at Phase-4-Meta-Core (pre-interface-freeze), default-policy fields locked at Phase-4-Meta-Core, full `refresh_required` semantics implemented at Phase-4-Meta-Composing (pre-v1-beta-tag UX wave). Total cost ~5-7 wave-days. Slots in as **Wave-G** under §7 of the F-full R0 plan-doc skeleton without expanding the existing ~65-72 wave-day v1-beta envelope.

### §1.2 Five load-bearing decisions (Ben call surface)

| # | Decision | Recommendation | Confidence | Reasoning anchor |
|---|---|---|---|---|
| **D1** | Is `atrium_policy_cid` in envelope AAD? | **NO** at v1-beta (upper-layer policy); **YES** to a `policy_version_at_seal: u32` field in `BindingContext::DropToRecipient` + `RemotePermission` for honest grandfathering. | HIGH | §5 — wire-format-affecting analysis; minimizes wire scope to ONE u32 field (4 bytes) sufficient for the load-bearing property |
| **D2** | Policy-update grandfathering vs retroactive | **Hybrid (recommended): grandfather `valid_until` ceilings but apply NEW `refresh_required` policy on first verifier-touch.** Policy-update can SHORTEN required-refresh cadence (defense gains immediate effect) but CANNOT retroactively shorten an already-sealed `valid_until` (UX + cryptographic-property stability). | HIGH | §2.4 + §3.3 — composes with Compromise #31; matches Matrix `m.room.history_visibility` precedent (state-event policy applies forward) |
| **D3** | `refresh_required` v1-beta or deferred | **Wire-format slot at v1-beta-CODEPOINT-RESERVE; impl at Phase-4-Meta-Composing.** `refresh_required: bool` + `refresh_window_seconds: u64` ship as Atrium config fields at Phase-4-Meta-Core; the verifier-side check + refresh-token mint flow ship at Phase-4-Meta-Composing. | MED-HIGH | §6 — UX-coupled; Compromise #31 closure is incremental not at v1-beta |
| **D4** | AtriumPolicy entity identity | **Content-addressed signed CBOR-tagged entity replicated as an Atrium-replicated Node** (per L9/A4 K_principal-rotation precedent) — `AtriumPolicy { atrium_did, policy_version, fields..., signed_by: PolicyAdminDid, signature }`. CID = BLAKE3 over canonical-DAG-CBOR-payload (Inv-15 payload-identity discipline). | HIGH | §2.1 + §2.5 — mirrors `KPrincipalRotation` L9/A4 Atrium-replicated-Node shape |
| **D5** | Default policy when Atrium has no explicit policy | **`AtriumPolicy::permissive_default()` — `default_validity_seconds = u32::MAX`, `max_validity_seconds = u32::MAX`, `refresh_required = false`** — semantically "no Atrium-level ceiling beyond UCAN scope's own `exp`". Atriums opt INTO restriction; default is "behaves identically to v1-beta-without-AtriumPolicy". | HIGH | §2.3 + §7.1 — minimizes migration surface; aligns with "Atrium-as-forkable" Ben-ratified semantic (forks default to parent policy) |

### §1.3 Confidence on the overall design

**MED-HIGH overall.** Specifically:
- HIGH confidence: entity shape (§2), composition with Amendments U4+U5+U19+U20+U21 (§4), wire-format placement (§5).
- MED-HIGH confidence: refresh-required semantics + revocation-list-vs-fresh-attestation choice (§6) — depends on Phase-4-Meta-Composing UX wave decisions Ben has not yet ratified.
- MED confidence: cost estimate (§8). 5-7 wave-days assumes Wave-G runs in parallel with Wave-C (Layer-C HPKE-multi-stanza); if Wave-G blocks on Wave-C, sequential cost is ~8-10 wave-days.

### §1.4 What I am NOT confident about + would defer to Ben

- Whether `refresh_required = true` Atriums should use **revocation-list pull** (CRL-style) OR **fresh-attestation pull** (OCSP-style) OR **CGKA-epoch-ratchet** (post-v1-beta MLS-PQ alignment). My recommendation is **fresh-attestation** for v1-beta (§6.2) but Ben may have a different posture on Atrium-relay-traffic cost.
- Whether Atrium admin = one DID or threshold-of-N DIDs (M-of-N multi-sig admin). My recommendation is **single-admin-DID with admin-rotation seam** at v1-beta; threshold-admin at Phase-N+1 if user demand emerges (§2.5).

---

## §2 AtriumPolicy entity design

### §2.1 Schema (Rust definition; canonical DAG-CBOR)

```rust
// crates/benten-atrium-policy/src/lib.rs  (new crate at Phase-4-Meta-Core Wave-G)

/// Per-Atrium credential-validity policy.
///
/// Content-addressed via canonical DAG-CBOR encoding per Inv-10.
/// Identifier (CID) is derived from the PAYLOAD bytes only (excludes
/// the signature sidecar) per Inv-15 sig-bundle-CID discipline.
///
/// Replicated across Atrium members as a system-zone Node:
/// `Node { labels: [system:AtriumPolicy], properties: { cbor: ... } }`
/// per existing Inv-11 system-zone reserved-prefix discipline.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]                                  // U10 #[non_exhaustive] discipline
#[serde(deny_unknown_fields)]
pub struct AtriumPolicyPayload {
    /// Schema version of this AtriumPolicyPayload itself.
    /// Distinct from `policy_version` — `schema_version` lets the
    /// AtriumPolicy struct evolve; `policy_version` lets a given
    /// Atrium's policy choice evolve over time.
    pub schema_version: u8,                        // = 1 at v1-beta

    /// DID of the Atrium this policy governs. Binds the policy to a
    /// specific Atrium-as-forkable identity; forks inherit parent
    /// policy until child mints a child-AtriumPolicy with its own
    /// atrium_did.
    pub atrium_did: Did,

    /// Monotonic policy-version. Updates by admin MUST strictly
    /// increment. Verifier rejects out-of-order policy-application
    /// (defense against admin-replay).
    pub policy_version: u32,

    /// Default validity window applied when issuer does not specify
    /// an explicit valid_until. Seconds, wall-clock-bound.
    /// Permissive default: u32::MAX (≈136 years — effectively forever).
    pub default_validity_seconds: u32,

    /// Hard ceiling. Issuers MUST clamp:
    ///   envelope.valid_until = min(requested, sealed_at + max_validity_seconds)
    /// Verifiers MAY enforce additionally at receive-time (subject to
    /// D2 grandfathering rule — see §2.4).
    pub max_validity_seconds: u32,

    /// If true, credentials MUST be re-issued / re-attested at least
    /// every `refresh_window_seconds`. Verifier checks:
    ///   if refresh_required && (now - sealed_at) > refresh_window_seconds {
    ///     reject as NEEDS_REFRESH (typed E_CRED_NEEDS_REFRESH)
    ///   }
    pub refresh_required: bool,

    /// If `refresh_required`, max time between refreshes. Ignored
    /// when refresh_required = false.
    pub refresh_window_seconds: u32,

    /// Refresh-attestation pull source — DID of a peer expected to
    /// serve fresh attestation tokens. Multiple admins / refresh
    /// authorities supported via Vec; verifier accepts any one.
    /// Ignored when refresh_required = false.
    pub refresh_attestation_authorities: Vec<Did>,

    /// Admin-rotation seam — current admin pubkey.
    /// AtriumPolicy.policy_version increment MUST be signed by this
    /// admin's secret key. Admin-rotation = mint NEW AtriumPolicy
    /// with new admin_pubkey, signed by OLD admin (rotation
    /// continuity); or by emergency-recovery key (out-of-scope at
    /// v1-beta — Compromise #45 candidate, see §4.7).
    pub admin_pubkey: HybridSigPubKey,             // Inv-17 hybrid floor

    /// Effective-at timestamp. Verifier applies policy whose
    /// effective_at_epoch_seconds <= now. Lets admin pre-announce
    /// future-effective policies.
    pub effective_at_epoch_seconds: u64,
}

/// Wire-format wrapper: payload + admin signature.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtriumPolicy {
    pub payload: AtriumPolicyPayload,
    /// Signature over canonical DAG-CBOR(payload) by
    /// payload.admin_pubkey (the PRIOR admin if this is an
    /// admin-rotation step; otherwise the same admin advancing
    /// policy_version).
    pub admin_signature: HybridSignature,
}

impl AtriumPolicy {
    /// Inv-15: identifier = payload-CID, NOT sig-bundle-CID.
    pub fn payload_cid(&self) -> Cid {
        Cid::blake3_dag_cbor(&self.payload)
    }
}
```

### §2.2 Why these specific fields (rationale per field)

| Field | Why it exists | Alternative-considered + why rejected |
|---|---|---|
| `schema_version: u8` | Future-additive schema evolution per L8/U14 `aad_version` discipline; bound into the payload itself rather than via codepoint because AtriumPolicy is at upper layer not envelope wire | Codepoint per-schema-version (rejected: AtriumPolicy is not in the envelope so codepoint-discrimination misaligns) |
| `atrium_did: Did` | Atrium-as-forkable: each fork has its own AtriumPolicy bound to its own atrium_did; forks inherit but DO NOT share | Anchor-by-CID (rejected: forking would require pre-fork policy CID to remain stable, conflicting with policy-evolution) |
| `policy_version: u32` | Monotonic ordering of policy updates; defense against admin-replay; lets verifier disambiguate "which policy was effective at sealed_at" | Use `effective_at_epoch_seconds` alone (rejected: clock-skew ambiguity; admin-rotation gets murky) |
| `default_validity_seconds: u32` | Issuer convenience: caller doesn't have to think about valid_until on every operation | Force every issuer to think about it (rejected: bad UX; UCAN already has `exp` so consistency wins) |
| `max_validity_seconds: u32` | Atrium ceiling for issuer-side clamp; defense against rogue-issuer or compromised-issuer minting forever-credentials | No ceiling (rejected: this IS the load-bearing property Ben articulated) |
| `refresh_required: bool` | Compromise #31 partial closure; lets Atrium opt-in to active credential lifecycle | Always-on refresh (rejected: violates Ben's "some Atriums say valid-forever" framing) |
| `refresh_window_seconds: u32` | Required iff `refresh_required`; configurable per Atrium threat-model | Hard-coded constant (rejected: per-Atrium-policy IS the design point) |
| `refresh_attestation_authorities: Vec<Did>` | Source-of-truth for "this credential is still live"; multi-DID supports availability + admin-rotation continuity | Single authority (rejected: single-point-of-failure for Atrium liveness) |
| `admin_pubkey: HybridSigPubKey` | Inv-17 hybrid mandatory; admin-rotation seam | Pure-classical or pure-PQ (rejected: violates Inv-17) |
| `effective_at_epoch_seconds: u64` | Pre-announcement window (admin can publish "new policy effective 7 days from now"); UX clarity for members | Immediate-effect-on-publish (rejected: members offline at publish-time get hit by policy they never saw; the pre-announcement window aligns with L9's "6-months-offline scenario" pattern) |

### §2.3 Default policy (`AtriumPolicy::permissive_default()`)

```rust
impl AtriumPolicy {
    /// Used when Atrium has never published an explicit AtriumPolicy.
    /// Semantically equivalent to "v1-beta-pre-AtriumPolicy" behavior:
    /// the envelope's own valid_until governs; no Atrium-level
    /// ceiling; no refresh requirement.
    ///
    /// Inv-17 hybrid-mandatory still applies to admin_pubkey;
    /// permissive_default() sets admin_pubkey to the Atrium's
    /// constitutional founder-DID per L9/§3.3 forkability semantic.
    pub fn permissive_default(atrium_did: Did, founder_pubkey: HybridSigPubKey) -> Self {
        AtriumPolicy {
            payload: AtriumPolicyPayload {
                schema_version: 1,
                atrium_did,
                policy_version: 0,
                default_validity_seconds: u32::MAX,
                max_validity_seconds: u32::MAX,
                refresh_required: false,
                refresh_window_seconds: 0,
                refresh_attestation_authorities: vec![],
                admin_pubkey: founder_pubkey,
                effective_at_epoch_seconds: 0,
            },
            // permissive_default() returns an UNSIGNED variant; verifier
            // checks "is this the well-known permissive_default for this
            // atrium_did?" via constructive check, not signature verify.
            // This is the ONE exception to the "every AtriumPolicy must
            // be signed" rule. Documented as Compromise #45 candidate
            // (§4.7) if the unsigned-default-exemption proves too clever.
            admin_signature: HybridSignature::zero_marker(),
        }
    }
}
```

### §2.4 Versioning + grandfathering rule (D2)

**Rule**: when verifier receives an envelope with `policy_version_at_seal: u32` in `BindingContext::DropToRecipient`, the verifier resolves the AtriumPolicy effective at that policy_version and applies:

1. **Grandfathered ceilings**: `valid_until` field is honored as-sealed. A v2-policy with `max_validity_seconds = 30d` does NOT retroactively shorten a v1-policy-sealed envelope with `valid_until = sealed_at + 90d`.
2. **Forward-applied refresh requirements**: if CURRENT policy (not at-seal policy) sets `refresh_required = true`, the verifier additionally checks `now - sealed_at > refresh_window_seconds → reject`. Refresh tightening DOES apply retroactively to in-flight credentials because it's a defense-gain that doesn't break content-addressing.
3. **Pre-fork policy continuity**: when Atrium A forks into A' at time T, all envelopes sealed pre-T continue to verify under the pre-fork AtriumPolicy (the one at `policy_version_at_seal`). Post-fork A' may evolve its own AtriumPolicy chain (`atrium_did = A'_did`); pre-T envelopes are unaffected.

**Why hybrid grandfathering** (NOT Option A pure-grandfather, NOT Option B pure-retroactive):
- Pure Option A (grandfather everything): adversary-issued forever-credentials minted under permissive earlier policy stay live forever even after Atrium tightens — defeats Ben's articulated goal ("some say valid forever, others say revalidate regularly"). Specifically: if compromised-admin minted credentials under v1 with `valid_until = u32::MAX` and v2 policy adds `refresh_required = true`, the v2 policy MUST be able to force those credentials to revalidate or expire.
- Pure Option B (retroactive everything): violates content-addressing stability + UX expectation. An envelope's `valid_until = T+90d` field is part of the AAD-bound payload; retroactively shortening it to T+30d means the verifier-side decision diverges from what the issuer + recipient agreed at seal-time.
- **Hybrid**: `valid_until` (issuer's explicit commitment) is grandfathered; `refresh_required` + `refresh_window_seconds` (live-state attestation requirement) applies forward. Matches Matrix `m.room.history_visibility` precedent ([Matrix spec](https://spec.matrix.org/latest/) — state changes apply forward to future events but past events retain their at-event visibility decision).

**Edge case**: what if v2 policy adds `max_validity_seconds = 30d` AND ALSO the v1-sealed envelope has `valid_until > now + 30d`? The valid_until is grandfathered (per rule 1) BUT the verifier still applies the `refresh_required + refresh_window_seconds` if v2 set those (per rule 2). So tightening max_validity is effectively "soft" (only applies to FUTURE seals) while tightening refresh is "hard" (applies retroactively). This asymmetry is intentional and is the load-bearing usability property.

### §2.5 Identity + distribution + update

**Identity**: AtriumPolicy is a system-zone Node (Inv-11) with `labels = ["system:AtriumPolicy"]` and `properties = { atrium_policy_cbor: <bytes> }`. The Node CID is derived from the canonical Node bytes (existing Phase-2a discipline). The Atrium-level policy-chain identity is `(atrium_did, policy_version)` tuple — verifier resolves via "find the system:AtriumPolicy Node with this (atrium_did, policy_version)".

**Distribution**: AtriumPolicy Nodes replicate via the existing Atrium peer-mesh (CRDT-merge per `engine_sync.rs::apply_atrium_merge`). New Atrium members bootstrap by:
1. Joining Atrium (existing flow per `AtriumHandle::join_atrium`).
2. Pulling the current Atrium state from a known peer (existing `sync_subgraph` flow).
3. Among the pulled Nodes, finding all `system:AtriumPolicy` Nodes with `atrium_did = this_atrium`; selecting the one with the highest `policy_version` whose `effective_at_epoch_seconds <= now`.
4. Verifying the admin signature against the prior policy_version's `admin_pubkey` (or against the Atrium founder for `policy_version = 1` if no `permissive_default` Atrium-genesis variant); rejecting on signature mismatch.

**Update**: admin posts a new `AtriumPolicy` with `policy_version = prev + 1`, signed by `prev.admin_pubkey`. The new policy replicates via standard CRDT-merge. Members verify admin-signature; on success, the new policy becomes effective at its `effective_at_epoch_seconds`.

**Admin-rotation**: admin posts a new AtriumPolicy with `admin_pubkey = new_pubkey`, signed by OLD admin's secret key (continuity). Successor admin can then issue future policy updates. This is a single-DID-admin v1-beta model; threshold-admin (M-of-N) is a Phase-N+ extension if user demand emerges (Section §4.7).

### §2.6 What is NOT in this schema (NAMED-DEFERRED to post-v1-beta)

- **Per-PermissionOperation-variant policy** (e.g. "Drop bundles get 90-day max but ExecuteWorkflow gets 7-day max"). Future-additive via `Vec<PerOperationPolicy>` field added under `#[non_exhaustive]` discipline. Deferred until concrete demand emerges; v1-beta ships a single ceiling.
- **Threshold-admin (M-of-N multi-sig)**. Single-admin-DID at v1-beta; future-additive via `admin_pubkey` becoming an enum `Admin::Single | Admin::Threshold { ... }`. Deferred.
- **Policy-attribution-frame in AttributionFrame**. Could thread `policy_version_at_seal` into `AttributionFrame` for audit-trail completeness — deferred unless audit-firm flags as gap (L5 §5 audit-deliverables didn't flag).
- **Per-recipient-policy override**. The Atrium-level policy applies uniformly; no per-recipient carve-outs at v1-beta. Defer.

---

## §3 Issuer + verifier flow

### §3.1 Issuer-side flow (minting a credential targeted at Atrium A)

```rust
// crates/benten-atrium-policy/src/issuer.rs
pub fn mint_credential_to_atrium(
    issuer: &IssuerHandle,
    atrium: AtriumDid,
    requested_valid_until: u64,
    permission_op: PermissionOperation,
) -> Result<EncryptedEnvelope, IssuerError> {
    // (1) Resolve current AtriumPolicy
    let policy = issuer
        .atrium_state(atrium)
        .resolve_current_policy()?;                // §2.5 resolution
    let now = wall_clock_now_seconds();

    // (2) Compute effective valid_until per policy ceiling
    let ceiling = now.saturating_add(policy.payload.max_validity_seconds as u64);
    let effective_valid_until = requested_valid_until.min(ceiling);

    // (3) Bind policy_version_at_seal into BindingContext (D1)
    let binding = BindingContext::DropToRecipient {
        // ... existing U17 + U19 fields ...
        sealed_at_epoch_hour: epoch_hour(now),     // U28 coarse bucket
        valid_until_epoch_hour: epoch_hour(effective_valid_until),
        policy_version_at_seal: policy.payload.policy_version,
        // ...
    };

    // (4) Seal with codepoint + AAD per Inv-16
    issuer.seal_envelope(binding, permission_op)
}
```

**Issuer-side OBSERVABLE consequence test (pim-2 §3.6b end-to-end pin)**:

```rust
#[test]
fn mint_with_request_above_ceiling_clamps_to_ceiling() {
    let policy = test_atrium_policy(max_validity_seconds = 30 * 24 * 3600);
    let env = mint_credential_to_atrium(
        &issuer,
        atrium_did,
        requested_valid_until = wall_clock_now() + 90 * 24 * 3600,
        op,
    ).unwrap();
    assert_eq!(env.binding().valid_until_epoch_hour(),
               epoch_hour(wall_clock_now() + 30 * 24 * 3600));
    // would-FAIL-on-revert: issuer side would set 90d without clamp
}
```

### §3.2 Verifier-side flow (receiving an envelope from issuer)

```rust
// crates/benten-atrium-policy/src/verifier.rs
pub fn verify_envelope_against_atrium_policy(
    verifier: &VerifierHandle,
    env: &EncryptedEnvelope,
) -> Result<(), VerifyError> {
    let binding = env.binding_context();
    let atrium = binding.target_atrium_did();
    let now = wall_clock_now_seconds();

    // (1) Resolve at-seal AND current AtriumPolicy
    let at_seal_policy = verifier
        .atrium_state(atrium)
        .resolve_policy_at_version(binding.policy_version_at_seal())?;
    let current_policy = verifier
        .atrium_state(atrium)
        .resolve_current_policy()?;

    // (2) Basic expiry per Amendment 5 / U5
    if now > binding.valid_until_epoch_seconds() {
        return Err(VerifyError::Expired);
    }

    // (3) Per-D2 hybrid grandfather: confirm at-seal envelope was
    //     consistent with at-seal policy ceiling (defense against
    //     rogue-issuer ignoring policy at seal-time)
    let at_seal_ceiling = binding.sealed_at_epoch_seconds()
        .saturating_add(at_seal_policy.payload.max_validity_seconds as u64);
    if binding.valid_until_epoch_seconds() > at_seal_ceiling {
        return Err(VerifyError::RogueIssuerIgnoredPolicyCeiling);
    }

    // (4) Per-D2 hybrid grandfather: apply CURRENT refresh policy
    //     (refresh tightens forward; defense gains immediate effect)
    if current_policy.payload.refresh_required {
        let age = now.saturating_sub(binding.sealed_at_epoch_seconds());
        if age > current_policy.payload.refresh_window_seconds as u64 {
            return Err(VerifyError::NeedsRefresh);
        }
    }

    Ok(())
}
```

**Verifier-side OBSERVABLE consequence test**:

```rust
#[test]
fn rogue_issuer_envelope_above_at_seal_ceiling_rejects() {
    let policy = test_atrium_policy(max_validity_seconds = 30 * 24 * 3600);
    // Mint envelope BYPASSING issuer clamp (simulating rogue/compromised issuer)
    let env = forge_envelope_with_valid_until(
        valid_until = sealed_at + 90 * 24 * 3600,  // > ceiling
        policy_version_at_seal = policy.version,
    );
    let err = verify_envelope_against_atrium_policy(&verifier, &env).unwrap_err();
    assert!(matches!(err, VerifyError::RogueIssuerIgnoredPolicyCeiling));
    // would-FAIL-on-revert: verifier without check (4) accepts it
}
```

### §3.3 Policy version-handling answer (D2)

Per §2.4 hybrid rule — `valid_until` is grandfathered, `refresh_required` applies forward. Concrete example:

| Scenario | v1 policy | v2 policy | Envelope sealed at v1, verified at v2 | Outcome |
|---|---|---|---|---|
| Alice minted at v1 max=90d, v2 tightens max=30d | max=90d | max=30d | `valid_until = sealed_at + 90d` (legit at v1) | **Verifier accepts until natural 90d expiry** (grandfathered per rule 1). v2's tighter ceiling applies to NEW seals only. |
| Alice minted at v1 refresh=false, v2 adds refresh=true window=7d, sealed_at = 30 days ago | refresh=false | refresh=true, window=7d | sealed 30 days ago, valid_until = sealed_at + 90d | **Verifier rejects with `NeedsRefresh`** (forward-applied per rule 2). 30 days > 7 day window. Alice must refresh. |
| Rogue issuer minted at v1 with `valid_until = sealed_at + 200d` ignoring v1 max=90d | max=90d | max=90d | `valid_until = sealed_at + 200d` exceeds v1 ceiling | **Verifier rejects with `RogueIssuerIgnoredPolicyCeiling`** (defense against compromised-issuer; per rule check (3) §3.2). |

---

## §4 Composition with existing design (per Task 3)

### §4.1 Amendment 5 (sealed_at + valid_until in AAD) — U5

AtriumPolicy **does not change wire format** for the sealed_at + valid_until fields themselves (per D1). What it adds:
- A NEW `policy_version_at_seal: u32` field in `BindingContext::DropToRecipient` + `BindingContext::RemotePermission` (wire-format-affecting per L9 variant scoping — `Vault` + `PerNodeAead` excluded).
- Issuer-side clamp + verifier-side rule-(3)+rule-(4) checks per §3.

**Wire-format delta**: 4 bytes (one `u32`) added per envelope of those two variant types. Vault + PerNodeAead unchanged. Per Inv-16 strict-decode + codepoint-discrimination discipline — the 4-byte addition lives inside the canonical TLV per U3, length-prefixed.

### §4.2 L9 Amendment A5 (ExecuteWorkflow) — U21

AtriumPolicy composes naturally: `ExecuteWorkflow.max_decrypt_count` is a per-grant constraint, AtriumPolicy provides an Atrium-wide ceiling on `valid_until` for any ExecuteWorkflow grant. Issuer clamps `ExecuteWorkflow.valid_until = min(requested, sealed_at + atrium_policy.max_validity_seconds)`. The `max_decrypt_count` is orthogonal (count-based vs time-based). No conflict.

### §4.3 UCAN scope `exp` semantics

AtriumPolicy is **AND-composed** with UCAN's `exp` field — the effective expiry is `min(ucan.exp, envelope.valid_until)`. AtriumPolicy adds:
- An Atrium-level ceiling that bounds UCAN-issuer's flexibility ("UCAN `exp` cannot exceed AtriumPolicy.max_validity_seconds")
- A refresh requirement that supplements UCAN's monotonic-time `exp` ("even within UCAN's `exp` window, the envelope-bound credential is invalidated if not refreshed within AtriumPolicy.refresh_window_seconds")

This is intentional layering: UCAN `exp` is the **issuer's commitment** of "I intend this delegation to be live until T"; AtriumPolicy is the **community's commitment** of "regardless of issuer intent, no delegation in this community is live longer than X seconds or without refresh every Y seconds". Both must agree for the credential to verify.

### §4.4 CGKA-deferred

AtriumPolicy distribution **does NOT require CGKA**. It uses the existing Atrium CRDT-merge path. AtriumPolicy ≠ a key-rotation primitive — it's a signed-policy primitive. Members verify admin-signature; no shared-secret-key-rotation across membership needed. CGKA remains a Phase-N+ post-v1-beta deferral (per U13 codepoint reservation).

**Where AtriumPolicy intersects with future CGKA**: when CGKA lands (post-v1-beta MLS-PQ alignment), Atrium epoch-ratchets become an additional defense layer; AtriumPolicy.refresh_window_seconds could be tied to the CGKA epoch cadence ("refresh_window_seconds = CGKA epoch duration"). But this is post-v1-beta — at v1-beta, AtriumPolicy stands alone.

### §4.5 Forkability (Ben-ratified 2026-05-27)

**Forkability semantic preserved**: when Atrium A forks into A' (at fork-time T):
- Pre-fork content (Drop bundles sealed pre-T with `policy_version_at_seal` from A's pre-T policy chain) continues to verify under A's pre-T AtriumPolicy. The atrium_did binding in the AtriumPolicy entity means A's policy chain is identified by `atrium_did = A_did`, A''s by `atrium_did = A'_did`. Pre-fork envelopes are bound to A_did (via the BindingContext's existing Atrium identifier per L9/A2 dual-CID) — they continue resolving against A's policy chain forever.
- Post-fork: A' mints its own AtriumPolicy genesis (`policy_version = 0` permissive_default; OR `policy_version = 1` admin-signed against A's last-pre-fork admin OR against A''s constitutional founder). Members in A' (whether they were in A or new joiners) use A''s policy chain.
- **Refinement**: child Atriums MAY explicitly inherit parent policy by setting `admin_pubkey = parent.admin_pubkey` at genesis — this is a UX-level convention, not a structural mandate. Some forks want continuity with parent; others want fresh start. AtriumPolicy supports both.

### §4.6 L5 audit-readiness — what's the audit-deliverable?

AtriumPolicy adds the following audit-deliverables (slotting into existing L5 §5.2 corpus):

| Audit deliverable | Location | Slot in L5 §5 |
|---|---|---|
| **AtriumPolicy schema spec** | `docs/CRYPTO-CODEPOINTS.md` new §"AtriumPolicy" subsection | §5.1 (parameter doc) |
| **Issuer-side clamp + verifier-side check test corpus** | `crates/benten-atrium-policy/tests/atrium_policy_*.rs` (~10 test cases per §3 flows) | §5.2 (test corpus) |
| **Rogue-issuer adversarial test** | `crates/benten-atrium-policy/tests/rogue_issuer_envelope_above_ceiling_rejects.rs` | §5.2 (negative tests) |
| **Policy-version downgrade-attack test** | `crates/benten-atrium-policy/tests/policy_version_downgrade_rejected.rs` (defense against verifier resolving older policy_version than the seal claimed) | §5.2 (negative tests) |
| **Admin-rotation continuity test** | `crates/benten-atrium-policy/tests/admin_rotation_continuity_chain.rs` | §5.2 (audit-narrative test) |
| **Threat-model section addition** | `docs/THREAT-MODEL.md` new §T-26 "AtriumPolicy admin compromise" | §5.3 (threat-model) |
| **Compromise # mint** | `docs/SECURITY-POSTURE.md` new Compromise #45 "Single-admin-DID failure modes at v1-beta" (or absorb into #41 if Ben prefers) | §5.4 (Compromise mints) |

### §4.7 L6 privacy / metadata-leak — `policy_version` fingerprinting

**Concern**: `policy_version_at_seal: u32` in `BindingContext` is plaintext-observable to wire-observers. An adversary observing many envelopes from many Atriums learns "Atrium-X is on policy_version N, Atrium-Y is on policy_version M" — this lets the adversary fingerprint per-community policy-update cadence. Small concern but real per L6 §4 metadata-archive treatment.

**Mitigation options**:
1. **Accept + disclose** (recommended at v1-beta): document in Compromise #43 extension ("AtriumPolicy.policy_version_at_seal in envelope BindingContext leaks per-community policy-update cadence to wire-observers"). Cost: zero. Honest-disclosure-at-mint per L5 audit-readiness pattern.
2. **Bucket the policy_version** (rejected for v1-beta): collapse u32 to ~4-bit (16 buckets) by hashing — defeats fingerprinting but breaks the admin-replay-rejection property (which depends on monotonic versioning).
3. **Bind policy_version inside HpkeMultiBase ciphertext stanza** (post-v1-beta Sealed-Sender-mode extension per U22): if AtriumPolicy ships post-Sealed-Sender, the policy_version can live INSIDE ciphertext like sender-DID does. Compose naturally with U22.

**Recommendation**: option 1 at v1-beta; option 3 post-v1-beta when U22 Sealed-Sender impl lands. Mint Compromise #43 extension.

### §4.8 Compromise #31 (forever-valid Drop bundles)

**Does AtriumPolicy close Compromise #31?** **PARTIAL CLOSURE.** Specifically:

| Compromise #31 aspect | Closed by AtriumPolicy? | How |
|---|---|---|
| Already-derived keys remain decryptable forever | **NO** — structural property of encryption-at-rest; only CGKA closes this | — |
| Drop bundles forever-valid once distributed (no producer-side revocation callback) | **PARTIAL via refresh_required = true** | Atrium opts into requiring credentials revalidate every refresh_window; this doesn't take away decryption capability but invalidates the AUTHORIZATION layer (the UCAN scope is no longer accepted by verifier-side `CapabilityPolicy::check_read`) |
| Tight UCAN `nbf`/`exp` is the current mitigation | **STRENGTHENED** — AtriumPolicy adds an Atrium-level ceiling on top of UCAN-issuer choice | Defense-in-depth |
| Key rotation discipline | **COMPLEMENTS** — Atrium can require K_principal rotation cadence via separate Atrium-replicated Node | Composes; orthogonal mechanism |

**Net**: AtriumPolicy moves Compromise #31 from "OPEN, mitigated by UCAN tight windows" to "OPEN at decryption-capability layer, CLOSED-at-authorization-layer for Atriums opting into `refresh_required`". The structural property (already-derived-keys-decrypt-forever) stays open — CGKA closes that post-v1-beta. The authorization-layer property closes per-Atrium opt-in.

This is honest. Update Compromise #31 narrative at the SECURITY-POSTURE.md row to reflect the partial closure path.

---

## §5 Wire-format-affecting? (load-bearing v1-beta decision)

### §5.1 The load-bearing question

The brief's critical question: does AtriumPolicy need to be in the envelope wire format?

**Three sub-questions**:
1. Does the AtriumPolicy CID itself need to be in the envelope? — **NO**.
2. Does a policy version reference need to be in the envelope's BindingContext? — **YES, a single `policy_version_at_seal: u32` field**.
3. Can AtriumPolicy be entirely an upper-layer policy with no envelope changes? — **PARTIALLY: the policy entity is upper-layer (no AAD CID needed), but defense against rogue-issuer requires the policy-version to be AAD-bound so verifier knows which policy ceiling to apply**.

### §5.2 Why ONE u32 field (not full CID, not zero fields)

| Option | Bytes added per envelope | Property gained | Why rejected/chosen |
|---|---|---|---|
| **Zero AAD fields** | 0 | Pure upper-layer policy; verifier reads CURRENT policy + applies | **Rejected**: verifier doesn't know which AT-SEAL policy applied → cannot enforce grandfather rule 3 (rogue-issuer defense) |
| **`atrium_policy_cid: Cid` (32 bytes)** | 32 | Full pin to specific policy version | **Rejected**: 32 bytes per envelope is bandwidth + storage cost; policy-resolution still requires AtriumPolicy chain walk; CID over-pins (no need for full content-identity in AAD) |
| **`policy_version_at_seal: u32` (4 bytes)** ⭐ | 4 | Verifier resolves `(atrium_did, policy_version)` from at-seal claim; can enforce rogue-issuer defense; can apply hybrid grandfather rule | **Chosen**: minimal AAD scope-creep; the atrium_did is already in `BindingContext::DropToRecipient`'s target-Atrium-identifier per L9/A2 (no new field needed for atrium binding); policy_version is the strictly-needed disambiguator |

### §5.3 Phase placement decision (D3)

| Component | Phase | Rationale |
|---|---|---|
| `policy_version_at_seal: u32` field in `BindingContext::DropToRecipient` + `RemotePermission` | **Phase-4-Meta-Core LOAD-BEARING** | Wire-format-affecting; must land pre-interface-freeze; lock-now (4 bytes; impl can default to 0 if no AtriumPolicy ever published; cheap migration path) |
| `AtriumPolicyPayload` schema + `AtriumPolicy` wire struct + admin signature flow + admin-rotation | **Phase-4-Meta-Core LOAD-BEARING** | Schema lock at interface-freeze; ship the constructor + serialization but the verifier-side enforcement can default to permissive at v1-beta if Phase-4-Meta-Composing slips |
| Issuer-side clamp (§3.1) | **Phase-4-Meta-Core LOAD-BEARING** | Defense against rogue-issuer is at-seal-time |
| Verifier-side basic expiry (§3.2 step 2) | Already shipped via U5 | — |
| Verifier-side rogue-issuer defense (§3.2 step 3) | **Phase-4-Meta-Core LOAD-BEARING** | Required for §3.3 rule (3) to be effective |
| Verifier-side `refresh_required` enforcement (§3.2 step 4) | **Phase-4-Meta-Composing pre-v1-beta-tag** | UX-coupled (needs refresh-token mint flow + user-facing "credential needs refresh" surfaces); not wire-format-affecting beyond the already-locked u32; can ship in the UX-wave |
| AtriumPolicy distribution + CRDT-merge integration | **Phase-4-Meta-Core LOAD-BEARING** | Reuses existing `apply_atrium_merge` path; thin shim |
| Admin-rotation seam | **Phase-4-Meta-Composing pre-v1-beta-tag** | Single-admin v1-beta is fine; rotation can ship later |

### §5.4 v1-beta-CODEPOINT-RESERVE vs LOAD-BEARING bucket assignment

Per §6 of the consolidated registry's bucket framework:
- The `policy_version_at_seal: u32` AAD field is **wire-format-affecting** → MUST be LOAD-BEARING.
- The `AtriumPolicy` schema fields are wire-format-affecting for distributed-policy bytes → MUST be LOAD-BEARING.
- The `refresh_required` enforcement is impl-only → can be **CODEPOINT-RESERVE** (shape locked, impl deferred to Phase-4-Meta-Composing).

**Net classification**:
- **22 → 23 amendments LOAD-BEARING** at v1-beta-tag (adds U41 "AtriumPolicy schema + `policy_version_at_seal: u32` AAD field" to the consolidated registry's LOAD-BEARING bucket per §6 table).
- **6 → 7 amendments CODEPOINT-RESERVE** (adds U42 "AtriumPolicy refresh_required enforcement; shape locked, impl in Phase-4-Meta-Composing").

---

## §6 Refresh-required semantics

### §6.1 How does refresh work?

**Recommended (v1-beta): fresh-attestation pull (OCSP-style)**.

Mechanism:
1. Credential holder Bob notices `now - sealed_at > policy.refresh_window_seconds * 0.8` (proactive refresh at 80% of window).
2. Bob fetches a `FreshAttestation` from one of `policy.refresh_attestation_authorities`:
   ```rust
   pub struct FreshAttestation {
       credential_cid: Cid,           // CID of the credential being refreshed
       attesting_authority: Did,      // which authority issued
       attested_at: u64,              // wall-clock seconds
       attestation_signature: HybridSignature,  // signed by authority
   }
   ```
3. Bob bundles `FreshAttestation` alongside the original envelope when presenting to verifier.
4. Verifier, in §3.2 step 4, checks:
   - If `current_policy.refresh_required = true` AND `now - sealed_at > refresh_window_seconds`:
     - Then look for a FreshAttestation in the presented bundle with `credential_cid = env.cid()`, `attesting_authority IN current_policy.refresh_attestation_authorities`, `now - attested_at < refresh_window_seconds`, and valid signature.
     - If found, accept. If not, reject with `NeedsRefresh`.

**Why OCSP-style (fresh-attestation pull) over CRL-style (revocation-list pull)**:
- Latency: OCSP-style means Bob's pull cost is O(1) per credential; CRL-style means Bob pulls the entire revocation list (O(N) per refresh, where N grows monotonically).
- Privacy: OCSP-style leaks "Bob is using credential X" to the authority; CRL-style leaks no per-credential request. **Concession**: CRL-style is more privacy-preserving at the credential-presentation layer. **Mitigation**: refresh_attestation_authorities can be multiple DIDs; Bob can shard requests across them; future-additive cover-traffic per U26 NAMED-DEFERRED.
- Availability: OCSP-style requires authority to be online at Bob's refresh-time; CRL-style allows Bob to cache the revocation list. **Concession**: less robust to authority offline.
- Hybrid: a sensible v2.0 design ships both — fresh-attestation for time-sensitive credentials + CRL for high-volume credentials. v1-beta picks fresh-attestation only.

### §6.2 Does Atrium need to maintain per-credential validity-status (revocation-list-equivalent)?

**For v1-beta refresh_required Atriums: YES, implicitly via the FreshAttestation flow**. The `refresh_attestation_authorities` peer maintains an internal "currently-valid-credentials" set (in-memory; persisted in its own engine). Bob requests fresh attestation; authority looks up Bob's credential in the set; mints FreshAttestation if found; refuses if revoked.

The revocation-list itself is NOT replicated across Atrium — it's the authority's local responsibility. This is intentional: per-credential revocation state is privacy-sensitive (knowing "credential X is revoked" reveals "credential X exists") and not all Atrium members should see it.

### §6.3 Composition with Compromise #31 forever-valid Drop bundles

**Closure path**:
- v1-beta-default Atrium (no AtriumPolicy or permissive_default): Compromise #31 stays OPEN as today.
- v1-beta refresh_required Atrium: Compromise #31 closes at the AUTHORIZATION layer — credentials become invalid after `refresh_window_seconds` unless refreshed. Drop bundles BYTES remain forever-decryptable (structural, not closable without CGKA), but the authorization-layer property is closed.

**Concrete example**: an Atrium says `refresh_required = true, refresh_window = 7d`. Bob holds a Drop bundle from 30 days ago. Bob's local Drop-bundle decryption still works (he holds the keys; structural). But when Bob tries to USE the decrypted content (e.g., to authorize a follow-on read or share), the verifier-side `CapabilityPolicy::check_read` resolves the AtriumPolicy, sees `refresh_required = true`, checks Bob's credential age (30d > 7d), and rejects unless Bob has a FreshAttestation.

This is the load-bearing UX: "credential" = "authorization to act in the Atrium", not "ability to read raw bytes". Active-Atrium-lifecycle (refresh_required = true) closes the authorization gap.

### §6.4 Composition with L9 A3 recipient-key-rotation

L9/A3 introduces `recipient_key_generation: u32`. AtriumPolicy's `refresh_required` composes:
- Recipient-key-rotation is about WHICH key-generation can decrypt (key-management layer).
- Refresh-required is about WHEN an authorization is still live (policy-layer).
- A credential could be valid-cryptographically (recipient key matches) but invalid-policy-wise (refresh expired). Verifier checks both; reject on either fails.

No conflict; orthogonal axes.

### §6.5 UX implications

**User expectations**:
- **Background refresh**: most users should never see refresh as a manual step. Client-side auto-refresh at 80% of window when application is online; silent failure UX when offline-past-window.
- **Surface "credential expired" only on**: (a) offline-too-long; (b) authority unavailable + window-exceeded; (c) explicit revocation. Otherwise refresh is invisible.
- **Atrium admins** see `refresh_required` as a config knob: "yes / no", with explanation "credentials revalidate every N days; protects against compromised devices but requires periodic connectivity".
- **End users** see surfaces like: "You've been offline 8 days; your access expires in 2 days. Connect to refresh." (when window = 10d).

---

## §7 UX + admin-surface design

### §7.1 Atrium admin config surface

```
[Atrium Settings]
├── Credential lifecycle
│   ├── ☐ Credentials valid forever (DEFAULT)
│   ├── ☑ Credentials expire after [90] days
│   └── ☑ Require revalidation every [7] days
│       └── Refresh authority: [admin-DID-1, admin-DID-2]
├── Admin
│   ├── Admin DID: did:key:abc...
│   └── [Rotate admin]
└── Effective from: [now] [in 7 days]  ← supports pre-announce
```

### §7.2 End-user-facing surfaces

| Event | UX surface | When fires |
|---|---|---|
| Refresh needed soon (proactive) | Subtle banner: "Refreshing your access..." | At 80% of refresh_window |
| Refresh in-flight | Silent (background) | Always when online |
| Refresh failed (offline) | None until window-exceeded | — |
| Window exceeded + offline | Banner: "You've been offline N days. Connect to refresh access." | now > sealed_at + refresh_window |
| Window exceeded + authority refused (revoked) | Modal: "Your access to <Atrium> has been revoked. Contact admin." | Authority returned refusal |
| Atrium admin published new policy | Notification: "Atrium <name> updated its credential policy. New: <summary>" | First merge of new policy after effective_at |

### §7.3 Atrium-relay-traffic cost

For an active Atrium with N members + refresh_window = 7d, refresh-attestation requests = ~N per week per refresh authority. Per-request cost: 1 round-trip + ~256-byte FreshAttestation envelope. For N = 10,000 members, ~1400 requests per authority per day — well within iroh-blobs / sendme capacity.

For Atriums with `refresh_required = false` (default): zero overhead.

---

## §8 Phase placement + cost estimate

### §8.1 Phase placement (consolidated)

| Component | Wave | Phase | Reasoning |
|---|---|---|---|
| `policy_version_at_seal: u32` AAD field | **Wave-G (new)** | Phase-4-Meta-Core LOAD-BEARING | Wire-format; locks pre-freeze |
| `AtriumPolicy` + `AtriumPolicyPayload` schema | Wave-G | Phase-4-Meta-Core LOAD-BEARING | Schema-on-wire (CRDT-replicated bytes) |
| Issuer-side clamp + verifier rogue-issuer check | Wave-G | Phase-4-Meta-Core LOAD-BEARING | Defense at envelope-mint + envelope-verify time |
| AtriumPolicy distribution (system-zone Node + CRDT-merge integration) | Wave-G | Phase-4-Meta-Core LOAD-BEARING | Reuses existing `apply_atrium_merge` |
| Verifier-side `refresh_required` enforcement | Wave-G' (Composing wave) | Phase-4-Meta-Composing pre-v1-beta-tag | UX-coupled; not wire-affecting beyond u32 |
| FreshAttestation flow + refresh-token mint authority | Wave-G' | Phase-4-Meta-Composing pre-v1-beta-tag | UX-coupled |
| Admin-rotation seam | Wave-G' | Phase-4-Meta-Composing pre-v1-beta-tag | Single-admin OK at v1-beta-tag |
| Atrium admin config UI | Wave-G' | Phase-4-Meta-Composing pre-v1-beta-tag | UX-only |
| End-user refresh surfaces | Wave-G' | Phase-4-Meta-Composing pre-v1-beta-tag | UX-only |

### §8.2 Cost estimate (wave-days)

| Task | Wave-days | Confidence |
|---|---|---|
| `benten-atrium-policy` new crate scaffold + types + canonical-encoding | 0.5 | HIGH |
| `AtriumPolicyPayload` schema definition + serde + DAG-CBOR canonicalization + property tests | 0.5 | HIGH |
| `policy_version_at_seal: u32` added to `BindingContext::DropToRecipient` + `RemotePermission` (4-byte AAD field, TLV per U3) | 0.5 | HIGH (mechanical extension of Inv-16) |
| Issuer-side clamp logic (§3.1 flow) + tests | 0.75 | HIGH |
| Verifier-side rogue-issuer defense (§3.2 step 3) + tests | 0.75 | HIGH |
| AtriumPolicy CRDT distribution + system-zone Node integration | 0.5 | MED-HIGH (composes with existing engine_sync) |
| Admin signature seam + admin-rotation continuity flow | 0.75 | HIGH |
| `permissive_default` + the "is well-known unsigned default?" check | 0.25 | HIGH |
| Audit-deliverable docs (CRYPTO-CODEPOINTS.md addition + THREAT-MODEL.md T-26 + Compromise #45 mint + INVARIANT-COVERAGE.md cross-link) | 0.5 | HIGH |
| Verifier-side `refresh_required` enforcement (Phase-4-Meta-Composing) | 1.0 | MED-HIGH |
| FreshAttestation flow + refresh-token mint authority (Phase-4-Meta-Composing) | 1.0 | MED |
| Admin UI + end-user surfaces (Phase-4-Meta-Composing) | 0.5 | MED (depends on existing Tauri UI scaffolding) |
| **Subtotal Phase-4-Meta-Core (Wave-G)** | **~5.0 wave-days** | HIGH |
| **Subtotal Phase-4-Meta-Composing (Wave-G')** | **~2.5 wave-days** | MED-HIGH |
| **Buffer +30%** | **~2.25 wave-days** | — |
| **GRAND TOTAL** | **~9.75 wave-days ≈ 2 calendar-weeks** | MED-HIGH |

**Within the consolidated registry's 7-15 week v1-beta window**: yes — adds ~2 weeks if sequential to existing waves; ~0 weeks if parallelized with Wave-C (Layer-C HPKE-multi-stanza). Recommend **parallelize Wave-G with Wave-C** since Wave-G's `BindingContext::DropToRecipient` extension touches the SAME struct as Wave-C's U17/U19 fields — bundling reduces double-touch.

### §8.3 Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Wave-G blocks on Wave-C completion (BindingContext layout coordination) | MED | Canary Wave-C first per `feedback_canary_first_parallel_implementation`; Wave-G consumes the locked layout |
| Phase-4-Meta-Composing UX wave slips → refresh_required ships only as wire-format-reserved-but-unimplemented at v1-beta | LOW | This is FINE per D3 (Compromise #31 closure is incremental); document in V1-FROZEN-INTERFACE-DEFERRED.md Row D-RPM-1 |
| Single-admin compromise = full Atrium policy-takeover | MED | Document as Compromise #45 mint; future-additive threshold-admin path |
| Policy-version replay (admin replays old policy_version) | LOW | Monotonic policy_version + signature-on-prior-admin discipline rejects replay |
| Adversary forks Atrium to dodge policy | MED | This is the forkability semantic; honest disclosure that "forking is a feature" |

---

## §9 R0 plan-doc + doc structure additions

### §9.1 Slot into the F-full R0 plan-doc skeleton (§7 of the consolidated registry)

```diff
 §6 EncryptedEnvelope wire format
   - DAG-CBOR outer framing with Benten-private CBOR-tag `0xBE54`
   - `EncryptedEnvelope { codepoint, payload, aad_binding }`
+  - `BindingContext::DropToRecipient` and `BindingContext::RemotePermission`
+    include `policy_version_at_seal: u32` field per U41 (NEW; AtriumPolicy
+    grandfathering disambiguator); TLV-encoded per U3
   - AAD canonicalization via `canonical_binding()` + `aad_version: u8`
   - ...

 §8 Wave decomposition for R5 implementation
   - Wave A: envelope shape + canonical_binding + TLV + amendments 1-3
   - Wave B: Layer-A vault + Argon2id tiering + XChaCha20 + amendments 4-5 + U20
   - Wave C: Layer-C HPKE-mode-base + libcrux + multi-stanza + amendments 17-19 + 22 + 25
   - Wave D: Layer-D device-link + remote-permission + amendments 21 + ExecuteWorkflow
   - Wave E: cross-ecosystem adapters (U29) + DAG-CBOR (U30) + golden vectors (U37) + dudect CI (U38) + kani (U39)
   - Wave F: audit-deliverable docs (U40 THREAT-MODEL.md + KEY-LIFECYCLE.md + CRYPTO-PARAMETERS.md + AUDIT-SCOPE-STATEMENT.md + SECURITY-POSTURE.md Compromise mints + INVARIANT-COVERAGE.md Inv-16/17/18 mints)
+  - **Wave G (NEW): AtriumPolicy schema + policy_version_at_seal AAD field + issuer
+    clamp + verifier rogue-issuer defense + AtriumPolicy CRDT distribution.
+    Parallelize with Wave-C (shared BindingContext::DropToRecipient touch).
+    ~5 wave-days at Phase-4-Meta-Core; ~2.5 additional wave-days at
+    Phase-4-Meta-Composing for refresh_required impl + admin UI**

 §11 v1-beta-LOAD-BEARING subset vs v1-GM-DEFER vs NAMED-DEFERRED
-  - 22 LOAD-BEARING wire-affecting amendments
+  - 23 LOAD-BEARING wire-affecting amendments (NEW U41 AtriumPolicy)
-  - 6 CODEPOINT-RESERVE amendments
+  - 7 CODEPOINT-RESERVE amendments (NEW U42 AtriumPolicy refresh_required impl-reserve)

 §12 Compromise # mints to land at v1-beta tag
   - ...
+  - **#45 (NEW; AtriumPolicy single-admin failure modes — Atrium policy-takeover
+    if admin secret-key compromised; mitigation = future-additive threshold-admin
+    path; honest-disclosure at v1-beta)**
+  - + EXTENSION to existing Compromise #43 — AtriumPolicy.policy_version_at_seal
+    metadata-leak (per-community policy-update cadence fingerprinting)
+  - + EXTENSION to existing Compromise #31 — partial closure at authorization-layer
+    for refresh_required = true Atriums (decryption-layer property stays OPEN
+    pending CGKA)
```

### §9.2 New doc files

| Path | Content | Phase |
|---|---|---|
| `crates/benten-atrium-policy/src/lib.rs` | Schema + types | Phase-4-Meta-Core Wave-G |
| `crates/benten-atrium-policy/src/issuer.rs` | Issuer clamp | Phase-4-Meta-Core Wave-G |
| `crates/benten-atrium-policy/src/verifier.rs` | Verifier checks | Phase-4-Meta-Core Wave-G (basic) + Phase-4-Meta-Composing (refresh) |
| `crates/benten-atrium-policy/src/distribution.rs` | CRDT-merge integration | Phase-4-Meta-Core Wave-G |
| `crates/benten-atrium-policy/tests/*` | ~12-15 test files per §3 + §4 + §6 flows | Phase-4-Meta-Core Wave-G + Composing |

### §9.3 Modifications to existing docs

| Path | Change |
|---|---|
| `docs/CRYPTO-CODEPOINTS.md` | New §"AtriumPolicy" subsection covering `policy_version_at_seal: u32` AAD field, `AtriumPolicy` schema, and `permissive_default` exemption |
| `docs/THREAT-MODEL.md` | New §T-26 "AtriumPolicy admin compromise" (single-admin failure, mitigation via threshold-admin future path) |
| `docs/THREAT-MODEL.md` | New §T-27 "AtriumPolicy rogue-issuer" (issuer ignoring policy ceiling, mitigation via verifier-side rule (3)) |
| `docs/SECURITY-POSTURE.md` | New Compromise #45 mint per §9.1 |
| `docs/SECURITY-POSTURE.md` | Compromise #31 narrative update — add "partial closure at authorization-layer for refresh_required Atriums" |
| `docs/SECURITY-POSTURE.md` | Compromise #43 narrative update — add `policy_version_at_seal` metadata-leak |
| `docs/INVARIANT-COVERAGE.md` | Cross-link Inv-15 (payload-CID identity) to AtriumPolicy.payload_cid() discipline |
| `docs/V1-FROZEN-INTERFACE.md` | Add `policy_version_at_seal: u32` to `BindingContext::DropToRecipient` + `RemotePermission` item 15(x) per existing freeze-row discipline |
| `docs/V1-FROZEN-INTERFACE-DEFERRED.md` | Row D-RPM-1 "Verifier-side `refresh_required` enforcement deferred to Phase-4-Meta-Composing" |
| `docs/V1-FROZEN-INTERFACE-DEFERRED.md` | Row D-ADMIN-1 "Threshold-admin (M-of-N multi-sig) deferred to Phase-N+1" |
| `dispatch-conventions §3.6` | Pim-N candidate: "AtriumPolicy-touching designs MUST grandfather-rule-disclose at design-time" — defer unless 3+ recurrence per pim-N codification discipline |

---

## §10 Self-assessment + confidence

### §10.1 Where my analysis is HIGH confidence

- **Entity shape (§2)**. The schema is conservative-additive — every field has a clear single purpose; `#[non_exhaustive]` discipline; mirrors L9/A4 K_principal-rotation Node shape; aligns with Inv-10 canonical-encoding + Inv-15 payload-identity. Cross-checked against L9 Atrium-integration lens A1-A5 + the K_principal-rotation precedent.
- **Wire-format placement (§5)**. ONE u32 AAD field is the minimal-viable wire delta that preserves rogue-issuer defense. Larger options (full CID) are over-pinning; smaller (zero fields) defeats §2.4 rule (3). Confident this is the right knob.
- **Composition with U4/U5/U17/U19/U20/U21 (§4)**. AtriumPolicy is upper-layer; doesn't conflict with any L9 amendment; composes additively. UCAN composition is AND-merged. Confident.
- **Default policy = permissive (§2.3, D5)**. Migration safety from v1-beta-pre-AtriumPolicy is critical; permissive_default behaves identically to "no AtriumPolicy". Confident.

### §10.2 Where my analysis is MED-HIGH confidence

- **D2 hybrid grandfathering (§2.4)**. The hybrid rule (grandfather `valid_until`, forward-apply `refresh_required`) is the right shape per the precedents (Matrix forward-applied state-policy + content-addressing stability) but the specific asymmetry might land differently if Ben weights UX-stability higher (then full grandfather wins) or weights defense-immediacy higher (then full retroactive wins). MED-HIGH on hybrid; could go pure-Option-A under different weighting.
- **Refresh-required v1-beta or deferred (D3)**. Splitting into wire-reserve-now + impl-Phase-4-Meta-Composing is the right phasing per "wire-format-affecting" vs "UX-coupled" decomposition, but Ben may want the full refresh flow at v1-beta tag for marketing posture ("refresh is part of v1-beta"). MED-HIGH on the split.
- **Cost estimate (§8.2)**. 5-7 wave-days Phase-4-Meta-Core + 2.5 wave-days Phase-4-Meta-Composing assumes Wave-G parallelizes with Wave-C. Sequential is ~8-10 wave-days. MED-HIGH on the parallel estimate.

### §10.3 Where my analysis is MED confidence (needs Ben call)

- **OCSP-style fresh-attestation vs CRL-style revocation-list (§6.1)**. I recommend OCSP for v1-beta but the privacy trade-off (Bob's "I'm using credential X" leak to authority) is real. CRL-style has higher pull cost but better privacy. MED on this — Ben may have a different posture.
- **Single-admin vs threshold-admin at v1-beta (§2.5, §8.3)**. I recommend single-admin + Compromise #45 mint for honest-disclosure. Some users may push back ("a single-admin community-policy is a single-point-of-failure" is structurally true). MED on whether the future-additive deferral is acceptable v1-beta posture.
- **Per-Atrium per-PermissionOperation policy** (rejected from v1-beta per §2.6). I deferred this; if Ben wants per-Op ceilings at v1-beta the schema expands (~1 more wave-day).

### §10.4 Where I'd recommend extra review

- **L9 lens re-look**: this design extends `BindingContext::DropToRecipient` + `RemotePermission` (L9-owned variants). A focused L9-style review of "does AtriumPolicy compose with A1-A5 without conflict?" would catch anything missed.
- **L5 audit-firm re-look**: AtriumPolicy adds a NEW privileged role (Atrium admin) — audit firm will want to enumerate its threat-model T-26 + T-27. The doc additions in §9.3 satisfy this but a focused L5 pass would confirm.
- **Compromise #31 narrative update** is delicate — the "partial closure" framing must be carefully worded to not over-promise (decryption-layer property stays OPEN).

### §10.5 Mini-pattern-induction sweep (per `feedback_pattern_induction_meta_sweep`)

Five candidate patterns I noticed authoring this design (would surface to a phase-close meta-sweep):

1. **Pattern: every "per-Atrium-configurable X" design needs the same 4-decision template** — schema entity shape; wire-format-affecting-or-not; grandfather-vs-retroactive policy; default-permissive-or-restrictive. AtriumPolicy is the first; SubscriptionPolicy / RetentionPolicy / NamingPolicy would follow the same template. Worth codifying as a dispatch-conventions pim-N candidate at 3+ recurrence.
2. **Pattern: signed admin-controlled state-events match Matrix `m.room.*` state precedent** — for several future Atrium-admin-controlled-knobs, the Matrix-state-event shape is the right precedent. Worth documenting in ARCHITECTURE.md as a reusable pattern.
3. **Pattern: 4-byte version-disambiguator in AAD beats 32-byte CID-pin** — same trade-off may recur for KPrincipalRotation, AtriumMembershipPolicy, etc.
4. **Pattern: Compromise # extensions are cleaner than new Compromise mints when the new closure is partial** — AtriumPolicy partially closes #31; extending #31 (per L5-C9 precedent) is cleaner than minting "#31a closure path".
5. **Pattern: refresh-required-AND-permissive-default-OFF is the only safe migration shape** — if AtriumPolicy ever defaulted to refresh_required=true, every existing Atrium would suddenly break at v1-beta upgrade. The permissive-default is structural, not stylistic.

---

## §11 Citations

### §11.1 Internal (Benten) sources

- Consolidated registry: `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16`, file `.addl/phase-4-meta/option-f-plus-9-eyes-consolidated-registry.md` (939 lines; 28 amendments + 14 Compromises + 3 invariants).
- L9 Atrium-integration: `phase-4-meta-core/option-f-plus-lens-l9-atrium-integration @ 1670aa03` — A1-A5 amendments + forkability + recipient-key-rotation + ExecuteWorkflow.
- L5 threat-model + audit-readiness: `phase-4-meta-core/option-f-plus-lens-l5-threat-model-audit-readiness @ 3f27f8e0` — T-01..T-25 + Compromise registry + audit-deliverable shape.
- Critique c1 (3f5a4351), c2 (9c548e5f), c3 (79c99aa5), c4 (6ea9718a), c5 (8e374a9d).
- `docs/INVARIANT-COVERAGE.md` (at SHA `2172cb6d`) — Inv-1..Inv-15 + Phase-4-Foundation plugin-DID matrix.
- `docs/SECURITY-POSTURE.md` (at SHA `2172cb6d`) — Compromise #31 narrative, especially "forever-valid Drop bundles" + RATIFIED-S&C §R6 references.
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (at SHA `2172cb6d`) — Row D-17 / D-SS-1 / D-COVER-1 pattern for AtriumPolicy deferred-impl rows.
- CLAUDE.md baked-in #5 (crypto-agility), #15 (v1-beta interface freeze), #18 (plugin trust model).
- Memory: `feedback_engine_primitives_vs_application_layer.md` (push application-layer composition before engine extension — AtriumPolicy is an app-layer entity, not engine primitive); `feedback_canary_first_parallel_implementation.md` (Wave-C canary before Wave-G fan-out).

### §11.2 External sources

- [Matrix.org Rooms & Events](https://matrix.org/docs/matrix-concepts/rooms_and_events/) — state-event precedent for per-community policy via signed CRDT-like state.
- [Matrix Specification](https://spec.matrix.org/latest/) — `m.room.*` state event semantics; forward-applied state-policy precedent for D2 hybrid grandfathering.
- [Matrix Policy Server MSC4284](https://github.com/matrix-org/policyserv) — community-level policy enforcement architecture.
- [ATProto Labels spec](https://atproto.com/specs/label) — labeler `exp` field precedent for per-community credential expiry; services persist expired labels but don't hydrate.
- [Bluesky Moderation Architecture](https://docs.bsky.app/blog/blueskys-moderation-architecture) — decentralized policy with community labelers; precedent for `refresh_attestation_authorities: Vec<Did>` multi-authority shape.
- [draft-ietf-lamps-pq-composite-sigs-19](https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-sigs/) — admin signature hybrid composition (Inv-17 hybrid-mandatory floor); same construction as Inv-15 cross-link.
- RFC 9180 (HPKE) §5.1.1 + §9.7.3 — mode-base provides no replay protection beyond same-stream, motivating AtriumPolicy.refresh_required as authorization-layer replay-defense.
- RFC 6960 (OCSP) — fresh-attestation pull precedent (§6.1).
- RFC 5280 §5 (CRL) — alternative revocation-list pull (rejected for v1-beta per §6.1).

---

**End of design doc. Total: ~6700 words. Decision-ready for Ben ratification at next R0 plan-doc authoring pass.**
