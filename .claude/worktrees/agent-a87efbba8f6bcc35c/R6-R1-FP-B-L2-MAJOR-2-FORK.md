# R6 R1 FP-B Bundle L2-R6-MAJOR-2 — HARD-ESCALATION FORK

**Status:** ESCALATED-WITH-FORK (orchestrator decision required).
**Source finding:** R6 R1 L2 lens finding `L2-R6-MAJOR-2` —
"Compromise #30 claims 'v1-beta ships PQ-hybrid by default for
signature' but ZERO application-layer signing sites use the hybrid
SignatureSuite (operator-misleading disclosure)."
**Author:** R6 R1 fp-b implementer (this branch =
`phase-4-meta-core/r6-r1-fp-b-crypto-security`).
**Date:** 2026-05-24.

---

## TL;DR

Compromise #30 narrative-vs-reality gap: PQ-hybrid `SignatureSuite` is
LANDED in `benten-crypto-suite` (G-CORE-2 commit `ae69c339`) +
conformance-tested, but 3 application-layer signing sites still
classical-only Ed25519 at HEAD:

1. `crates/benten-drop/src/envelope_sig.rs::sign_envelope` (DropBundle
   envelope-sig; via `Keypair::sign` → 64-byte Ed25519)
2. `crates/benten-platform-foundation/src/plugin_manifest.rs::verify_peer_signature`
   (plugin manifest peer-signature; 64-byte Ed25519 via
   `ed25519_dalek::Signature::from_bytes`)
3. `crates/benten-platform-foundation/src/plugin_manifest.rs::InstallRecord::verify_user_signature`
   (install-record user-signature; same shape)

(A fourth site — `AuthorizationGrant.binding_sig` — IS already
disclosed as classical-only at V1-FROZEN-INTERFACE-DEFERRED.md Row
D-15e.)

## Why this is HARD-ESCALATED (not auto-FIX-NOW)

The brief's escalation criterion: "wire-format-changing OR new
envelope versions OR signature size change cascade" ⇒ HARD-ESCALATE.

**All three trigger:**

1. **Signature size change.** `ed25519_dalek::Signature` = 64 bytes
   fixed. `HybridSignature` (Ed25519⊕ML-DSA-65, concatenated +
   commitment): roughly 64 + ~3293 + commitment = ~3357+ bytes. The
   on-wire envelope-sig / peer-signature / user-signature fields are
   currently sized + serialized as `Vec<u8>` but every consumer
   parses them as 64-byte Ed25519 via `try_into::<[u8;32]>` or
   `Signature::from_bytes`. Every type-conversion site rejects
   non-64-byte sigs at the validator boundary.
2. **Wire-format change cascade.** DropBundle + PluginManifest +
   InstallRecord all carry these sig fields in their CBOR canonical
   bytes — they have hex-pinned wire-format tests
   (`tf3f_drop_*.rs`, `plugin_manifest_*.rs`,
   `tf7_install_record_*.rs`). Updating signature shape MUTATES
   canonical bytes ⇒ canonical CIDs ⇒ every fixture-cited CID in the
   workspace becomes stale.
3. **Envelope-version bump cascade.** Each envelope (DropBundleVersion +
   PluginManifest serde-tag + InstallRecord) would need a v1 → v2
   bump to discriminate classical-sig vs hybrid-sig at decode time;
   this couples to the freeze-discipline at G-CORE-9 + the
   V1-WIRE-FORMAT-INVENTORY.md / V1-FROZEN-INTERFACE.md narrative.

Per the L2 lens itself the recommended disposition is
**DEFER-NAMED-NOW + retense Compromise #30 disclosure narrative**
(option-a in the L2 JSON):

> "(a) DEFER-NAMED-NOW (recommended; HARD RULE 12 clause-(b)) — add
> THREE sister-rows to V1-FROZEN-INTERFACE-DEFERRED.md Row D-15e
> (D-15e-1 DropBundle.envelope_sig + D-15e-2 PluginManifest.peer_signature
> + D-15e-3 InstallRecord.user_signature) AND retense Compromise #30's
> lines 2215-2218..."

LOC budget for option (a): ~30-40 LOC of doc + 3 new deferred rows.
LOC budget for option (b) wire-now-with-wire-change: realistic estimate
>1500 LOC across crypto-suite + drop + platform-foundation + every
serialization test + public-api baselines + V1-WIRE-FORMAT-INVENTORY +
V1-FROZEN-INTERFACE + SECURITY-POSTURE + cross-language ErrorCode
mirror potentially — and would block on resolving the envelope-version
bump strategy on three independent envelopes which is itself a P-III
Ben decision-point.

## Fork question

**(a) PATH-A: wire-now-with-wire-change.** Promote 3 app-layer sites
to use `SignatureSuite` + `HybridSignature` at v1-beta. Bump DropBundle
+ PluginManifest + InstallRecord envelope versions to discriminate v1
(classical) vs v2 (hybrid) at decode. Regenerate every wire-format pin
+ every fixture-cited CID. **OUT-OF-SCOPE for R6 fix-pass** per the
brief's escalation criterion; this is a multi-wave initiative that
couples to the V1-FROZEN-INTERFACE freeze contract (G-CORE-9 was
intended to LOCK these envelopes at v1-beta, not mutate them
post-freeze).

**(b) PATH-B: doc-honesty-defer-to-G-COMP-1 (L2 lens recommendation).**
3 new sister-rows under Row D-15e at
`docs/V1-FROZEN-INTERFACE-DEFERRED.md`:
- D-15e-1 — DropBundle.envelope_sig (classical-only at v1-beta)
- D-15e-2 — PluginManifest.peer_signature (classical-only at v1-beta)
- D-15e-3 — InstallRecord.user_signature (classical-only at v1-beta)

Plus retense Compromise #30 narrative at SECURITY-POSTURE.md lines
~2215-2218 to make the producer-side / app-layer split explicit:
- producer-side at the SUITE layer: hybrid IS the v1-beta default
- consumer-side at the APP LAYER: 4 named sites (the binding_sig +
  the 3 above) ride classical-only Ed25519 at v1-beta; promotion =
  G-COMP-1 destination
- HNDL impact: signatures only secure against the harvest-now
  decrypt-later quantum attacker via the encryption-half framing (#5);
  classical Ed25519 forgery requires breaking discrete log over the
  Edwards curve which is NOT PQ-vulnerable on its own; the v1-GM
  audit (NF-2 / C-GM-AUDIT) only certifies ml-dsa + ml-kem, so the
  app-layer sig path has NO PQ-audit certification at v1-beta — that
  is honest disclosure, not a regression.

LOC budget: ~30-40 LOC docs only; no code change; no test cascade.

## Orchestrator action items

1. **DECIDE (a) vs (b) at consolidation time.** Recommended: (b).
2. **If (b):** orchestrator-direct edits at consolidation. The 3
   sister-rows + Compromise #30 retense are well below the §3.5n
   inline-fix threshold; bundling with this branch's other commits is
   trivial.
3. **If (a):** open a new wave-brief for `phase-4-meta-core/g-comp-pq-hybrid-app-layer`
   (or similar) outside the R6 fix-pass window; this is not R6 fix-pass
   shape.

## What this fp-b branch did NOT do

- NO `SignatureSuite` integration at any of the 3 sites.
- NO wire-format change.
- NO new ErrorCode mints.

## What this fp-b branch DID do (the other 5 bundles in this branch)

Per the FP-B brief:
- Bundle F3 — AAD `total_chunks` cross-chunk-truncation defense ✅ (commit `d9cdaf93`).
- Bundle L3-r1-1 — AuthorizationGrant scope in binding_message ✅ (commit `39dd52cf`).
- Bundle L2-R6-MAJOR-1 — T10-upgrade (a) wire-in to install_plugin ✅ (commit `b5a6e758`).
- Bundle L11 — wire-format inventory expansion (14 surfaces) — pending in this branch.
- Bundle L1 — `#[non_exhaustive]` sweep — pending in this branch.

## Disposition recorded

ESCALATED-WITH-FORK. Orchestrator MUST select (a) or (b) before R6 R2.
