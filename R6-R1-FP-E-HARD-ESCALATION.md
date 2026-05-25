# R6 R1 FP-E — HARD-ESCALATION (wire-format sub-fork structural)

**Branch:** `phase-4-meta-core/r6-r1-fp-e-pq-hybrid-app-layer-wire`
**Author:** FP-E implementer (this branch).
**Date:** 2026-05-24.
**Scope:** L2-R6-MAJOR-2 — wire PQ-hybrid `SignatureSuite` at the 3 app-layer signing sites that B's fork doc named.

---

## TL;DR (orchestrator-direct read)

**This escalation is NOT a defer-bias instance.** Per
`feedback_orchestrator_defer_prediction_bias` the failure mode to
avoid is *suggesting* defer when do-now is correct. The brief explicitly
authorizes WIRE-NOW per Ben's 2026-05-24 PM directive — that disposition
is honored at the *direction* level. **What this escalation surfaces is
an in-brief structural sub-fork that the brief's own §"HARD-ESCALATE
TRIGGERS" clause names: "Any site surfaces a wire-format design sub-fork
(e.g., envelope-version-bump strategy)."** All three sites surface this
trigger; the trigger is structural, not a "this would be hard"
rationalization.

The fork is about **HOW to wire-now**, not whether. Three concrete paths
with realistic LOC budgets are presented; the choice between them is a
P-III Ben decision per `feedback_surface_arch_decisions_under_auth`.
**Recommended: Path C** (minimum-viable additive-hybrid, in-brief; ~600-800
LOC; lives in this branch).

---

## What the 3 sites actually need (the structural sub-fork)

The brief asks for `benten_crypto_suite::sign_hybrid(...)` returning
`HybridSignature` at 3 sites:

1. `benten_drop::envelope_sig::sign_envelope` (+ `verify_envelope`)
2. `benten_platform_foundation::plugin_manifest::PluginManifest::verify_peer_signature` (+ producer-side sign)
3. `benten_platform_foundation::plugin_manifest::InstallRecord::verify_user_signature` (+ producer-side sign)

**The structural reality (HEAD-verified):**

(a) **No standalone `sign_hybrid` / `verify_hybrid` free function exists.**
The hybrid sig path is `SignatureSuite::v1_default().sign(&suite_kp, msg)`
returning `HybridSignature`. Caller MUST hold a
`benten_crypto_suite::sig::Keypair` (carries `ed25519_dalek::SigningKey` +
`Option<MlDsaSigningKey<MlDsa65>>`) — NOT a `benten_id::Keypair` (which
carries `ed25519_dalek::SigningKey` only).

(b) **Hybrid verify requires BOTH pubkeys.** `SignatureSuite::verify`
takes a `PublicKey { classical: ed25519_dalek::VerifyingKey, pq:
Option<MlDsaVerifyingKey<MlDsa65>> }`. Today each of the 3 sites carries
only a 32-byte Ed25519 verifying-key bytes field:
- `DropBundle.issuer_verifying_key: Vec<u8>` (doc-comment + verify-site
  both assume 32-byte Ed25519)
- `PluginManifest.peer_did` (resolves to a single classical key via
  `Did::as_ed25519_verifying_key()`-shaped path)
- `InstallRecord.consenting_user_did` (same shape)

To verify a hybrid sig at any of these sites we MUST carry the PQ
pubkey alongside the classical one. Three structurally-distinct ways to
do that, each with downstream cascade:

- **(α) Additive sibling field** — add `issuer_pq_verifying_key:
  Vec<u8>` (empty for classical-only suite); add
  `sig_codepoint: SigCodepoint` discriminator. **Wire-format-changing
  for V1** (V1 is the only production version + pre-tag window is open
  → V1's contract is re-definable, NOT yet frozen). Old hex-pinned
  bytes (if any) regen; CBOR-roundtrip-only tests survive.
- **(β) Envelope v1→v2 enum bump** — `DropBundleVersion::V2` (adds
  PQ-pubkey + codepoint fields); v1 path stays for legacy verify-only.
  v1 sign path REMOVED (would violate CLAUDE.md "no deprecated aliases"
  rule). Couples to G-CORE-9 FREEZE narrative (v1-beta interface freeze
  contract).
- **(γ) Carry PQ pubkey inside the DID** — extend `Did` shape to carry
  hybrid pubkey bytes; resolve to `PublicKey { classical, pq }` at
  verify time. Cleanest semantically (PQ-hybrid IS the v1-beta default
  identity shape), but ripples through `benten_id` + every DID-shaped
  surface (Atrium device-DID, plugin-DID, peer-DID, user-DID, grant
  audience, etc.).

(c) **`benten_id::Keypair` is classical-only.** `Keypair::sign` returns
`ed25519_dalek::Signature` (64 bytes). To produce a hybrid sig at the
production sites (which today take `&benten_id::Keypair`), we need
either:
- Extend `benten_id::Keypair` to carry a hybrid `SigningKey` (option
  γ-coupled; touches every workspace signing call site)
- Add a parallel `benten_id::HybridKeypair` (~25-30 workspace call sites
  use `Keypair::sign`; 4 sites need migration to hybrid, the rest stay)
- Pass a `benten_crypto_suite::sig::Keypair` directly at the 3 sites
  (smallest blast radius; means each production caller must construct a
  hybrid suite-keypair at the boundary; loses the symmetry with
  classical sign sites)

(d) **The 4th deferred site** —
`benten_caps::grant::AuthorizationGrant.binding_sig` — is **already
explicitly deferred** at `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-15
to G-COMP-1 under exactly this shape ("hardcoded `[u8; 64]` (Ed25519) —
promote to varsig-tagged variable-length for crypto-agility parity with
the rest of #5 framing"). The 3 sites in scope here are structurally
identical to D-15's binding_sig.

(e) **No hex-pinned wire-format tests exist for DropBundle.** All
DropBundle tests roundtrip CBOR (size-agnostic) — a sig-size change
does NOT cascade through hex-pin fixtures. PluginManifest +
InstallRecord similarly checked: no hex-pinned wire-format tests at
HEAD; only CBOR roundtrip + length-equality checks (the
`peer_signature.len() != 64` length-checks at
`plugin_manifest.rs:94,165,593`). This narrows the cascade vs B's
~1500 LOC upper bound — but doesn't change the core sub-fork (a)/(b)/(c)
structural decision.

---

## The fork

### Path A — Full wire-now, multi-wave initiative

**Shape:** option (γ) — extend `Did` + `benten_id::Keypair` +
`benten_id::PublicKey` to be hybrid-aware, plumb through every workspace
signing call site (~25-30 sites), wire 3 production sites + binding_sig
(D-15 closure) + Atrium device-DID + UCAN audience binding +
canonical-bytes contract update for every CID-touching surface.

**LOC budget (realistic, post HEAD-verification):** ~2000-3000 LOC.

**Window:** OUT-OF-SCOPE for R6 fix-pass. This is a Phase-4-Meta-Core
new wave (call it `G-CORE-PQ-WIRE`). Couples to G-CORE-9 FREEZE narrative
(the canonical-bytes contract this would mutate is the same contract
G-CORE-9 was meant to LOCK at v1-beta).

**Pro:** complete; closes L2-R6-MAJOR-2 + Row D-15 + future-proofs every
identity-bearing surface for the v1-beta PQ-hybrid default.

**Con:** structurally above the brief's 800 LOC ceiling. Multi-wave =
post-R6-close = post `phase-4-meta-core-close` tag candidate. Couples to
freeze re-litigation.

---

### Path B — Uniform defer to G-COMP-1 + doc-honesty retense

**Shape:** B's fork-option (b) extended uniformly. Add 3 sister-rows
under existing Row D-15:
- D-15e-1 — `DropBundle.envelope_sig` + `issuer_verifying_key`
  (classical-only at v1-beta)
- D-15e-2 — `PluginManifest.peer_signature` (classical-only at v1-beta)
- D-15e-3 — `InstallRecord.user_signature` (classical-only at v1-beta)

Plus retense Compromise #30 (`docs/SECURITY-POSTURE.md` lines
2213-2218) to make the producer-side / consumer-side split explicit:
- producer-side at the SUITE layer (`benten_crypto_suite`): hybrid IS
  the v1-beta default — LIVE end-to-end, conformance-tested.
- consumer-side at the APP LAYER: 4 named sites (binding_sig + the 3
  above) ride classical-only Ed25519 at v1-beta; promotion to hybrid =
  G-COMP-1 destination (or `G-CORE-PQ-WIRE` if Ben chooses Path A
  separately).
- HNDL impact: signature is NOT PQ-vulnerable in the way encryption is
  (Ed25519 forgery requires breaking discrete log over Edwards, not
  PQ-vulnerable on its own); v1-GM audit certifies ml-dsa + ml-kem only
  → app-layer sig path has no PQ-audit certification at v1-beta, which
  is **honest disclosure**, not a regression.

**LOC budget:** ~40-60 LOC docs only; no code change; no test cascade.

**Window:** in-brief; lands at FP-E consolidation.

**Pro:** matches Row D-15 precedent uniformly. No structural change to
wire format. Closes L2-R6-MAJOR-2 via the disclosure-honesty path the
L2 lens itself recommended. Zero cascade.

**Con:** does NOT make the app-layer "PQ-hybrid is v1-beta default"
claim TRUE at the consumer layer. The claim is honest-narrative-only
at v1-beta; promotion gated on G-COMP-1.

---

### Path C — Minimum-viable additive-hybrid, in-brief (RECOMMENDED)

**Shape:** option (α) but bounded. Wire the 3 sites with the smallest
structural change that makes "PQ-hybrid is v1-beta default" TRUE at the
app layer for FRESHLY-MINTED bundles/manifests/records. Specifically:

1. Add `issuer_pq_verifying_key: Vec<u8>` sibling field to `DropBundle`
   + `peer_pq_verifying_key` to `PluginManifest` +
   `user_pq_verifying_key` to `InstallRecord`. All empty for the
   classical-only downgrade arm; populated for the hybrid arm.
2. Replace the `sig_codepoint` discriminator either as an explicit
   `u16` field OR implicitly-from-length (length == 64 → classical;
   length > 64 → hybrid — but explicit is safer per CLAUDE.md
   typed-unsupported-arm discipline).
3. Plumb a parallel `benten_id::HybridKeypair` (small wrapper:
   `inner: ed25519_dalek::SigningKey + pq: Option<MlDsaSigningKey<MlDsa65>>`)
   AT THE 4 PRODUCTION CALL SITES ONLY — do NOT migrate every
   `Keypair::sign` call site. Classical `Keypair::sign` stays for
   non-PQ-mandatory call paths.
4. `sign_envelope` (+ siblings) take `&HybridKeypair`; produce
   `HybridSignature::to_wire_bytes()`.
5. `verify_envelope` (+ siblings) dispatch on codepoint: hybrid path
   uses `SignatureSuite::v1_default().verify(...)` against
   `{classical, pq}` reassembled from the two pubkey fields; classical
   path stays for `sig_codepoint == 0x0002` arm (the downgrade-arm
   compatibility surface).
6. Compromise #30 retense at SECURITY-POSTURE.md: producer/SUITE +
   APP-LAYER both LIVE, classical-only downgrade arm preserved per
   crypto-agility seam.
7. V1-WIRE-FORMAT-INVENTORY.md update: 3 envelope items get sig-shape
   bump narrative entry.
8. Row D-15 retense: D-15 stays (binding_sig still deferred — orthogonal
   to envelope sigs); document the 3 sites are NO LONGER deferred at
   FP-E.

**LOC budget (HEAD-verified envelope of touched surfaces):**

- `benten-id`: ~150 LOC (new `HybridKeypair` + generate/import/export
  + tests)
- `benten-drop/src/envelope_sig.rs`: ~150 LOC (rewrite sign/verify;
  preserve classical path)
- `benten-drop/src/bundle.rs`: ~80 LOC (new field + plumbing)
- `benten-drop/tests/`: ~120 LOC (4 test files update)
- `benten-platform-foundation/src/plugin_manifest.rs`: ~200 LOC
  (PluginManifest + InstallRecord field-adds + verify rewrites)
- `benten-platform-foundation/tests/`: ~100 LOC (fixture-construction
  updates)
- Docs: ~80 LOC (SECURITY-POSTURE Compromise #30 + V1-WIRE-FORMAT-INV +
  Row D-15 retense + V1-FROZEN-INTERFACE update)

**Total: ~880 LOC.** *Just* above the 800 LOC ceiling; in-shape for
the brief. Commit-after-each-site preserved (S-DROP ≈ 350 LOC,
S-PMAN ≈ 200 LOC, S-IREC ≈ 150 LOC, E-DOCS ≈ 80 LOC, supporting
benten-id refactor ≈ 100 LOC bundled into S-DROP).

**Window:** in-brief; this branch.

**Pro:** makes the v1-beta hybrid-default claim TRUE at the app layer.
Honors WIRE-NOW disposition. Preserves classical-only downgrade arm
per crypto-agility seam. Avoids v2-envelope-bump complexity. Avoids
touching ~25 unrelated workspace `Keypair::sign` call sites.

**Con:** new public API surface (`HybridKeypair`) that's narrower than
the full Path A vision. Mild duplication with classical `Keypair`.
The "no deprecated aliases" rule is preserved (HybridKeypair is a
parallel surface, not a deprecation wrapper).

---

## Orchestrator decision needed

**Per `feedback_surface_arch_decisions_under_auth` this is a P-III
real architectural fork.** Pick one:

- **(A) PATH A — Full wire-now, multi-wave initiative.** Spawn
  `G-CORE-PQ-WIRE` brief. Out-of-scope for this branch. R6 FP-E lands
  a minimal Compromise #30 retense + 3 D-15 sister rows + a forward
  reference to G-CORE-PQ-WIRE.
- **(B) PATH B — Uniform defer to G-COMP-1 + doc-honesty retense.**
  R6 FP-E lands ~50 LOC docs only; no code change. Closes
  L2-R6-MAJOR-2 via disclosure-honesty path.
- **(C) PATH C — Minimum-viable additive-hybrid, in-brief
  (RECOMMENDED).** R6 FP-E lands ~880 LOC across `benten-id`,
  `benten-drop`, `benten-platform-foundation`, + docs.
  Commit-after-each-site discipline preserved. *Just* exceeds the
  800 LOC ceiling — flag for orchestrator-direct ceiling-override
  authorization.

## Why Path C is recommended

- Honors WIRE-NOW disposition (the L2 finding becomes TRUE-resolved at
  the app layer, not just disclosure-resolved).
- Bounded structural cascade (~880 LOC vs Path A's ~2000-3000).
- Preserves classical-only downgrade arm per crypto-agility seam.
- Pre-tag wire-format window IS still open (V1 contract re-definable).
- Closes the L2-R6-MAJOR-2 operator-misleading-disclosure finding at
  its strongest closure point (production reality matches Compromise
  #30's claim).

## Coordination notes (unchanged)

- B's branch landed F3 + L3-r1-1 + L1 etc.; no overlap with this scope.
- A's v2 branch landed F2 visibility; no overlap.
- F4 dispatch in parallel for substrate-frozen plugin-trust surfaces;
  no overlap.
- CATALOG_VARIANT_COUNT: this branch may need to mint 1-2 new
  hybrid-sig-typed-reject ErrorCodes under Path C (e.g.,
  `E_DROP_SIG_CODEPOINT_UNKNOWN`, `E_MANIFEST_HYBRID_PUBKEY_MISSING`).
  Coordinate post-F4 to avoid collisions. Path B requires 0 new
  ErrorCodes. Path A requires its own audit per the multi-wave brief.

## Hard-escalation disposition

ESCALATED-WITH-FORK + DOC-RECOMMENDED-PATH (C). Orchestrator MUST
select (A) / (B) / (C) before this branch's next commit lands.

This branch's HEAD remains at the FP-E branch creation point (no code
commits yet, only this doc commit + push); awaits orchestrator path
selection. Per
`feedback_orchestrator_defer_prediction_bias` the orchestrator is
expected to NOT auto-default to Path B; if Path C is selected the
implementer proceeds within this branch; if Path A is selected the
implementer hand-offs to the new wave's brief author + lands Path B's
~50-LOC doc-only closure in this branch as the FP-E minimal closure.
