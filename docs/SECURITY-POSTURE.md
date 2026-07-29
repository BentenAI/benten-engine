# Security Posture — Benten Engine (Phase-4-Meta-Core close)

## Compromise disposition-class index (canonical first-occurrence registry)

> This index is the **canonical first occurrence** of every `Compromise #N` token in this document and binds each
> to its `disposition_class` per the F-full posture taxonomy (`F-DISC-1` disclosure-coherence catch-net). The five
> classes: **`ATO`** = Accepted-Trade-Off · **`SGD`** = Substrate-Guarantee-Disclosure · **`CHD`** =
> Composition-Hazard-Honest-Disclosure · **`OOS`** = Out-Of-Scope · **`MIT`** = Mitigated-Open. The full narrative
> for each lives in its `### Compromise #N` detail section below (the summary tables + cross-references that follow
> ride this index). #30–#66 dispositions are R0.7 §5.2 (#65/#66 are the R13/R14 mints); #1–#29 reflect the
> Phase-1→4-Foundation closure state; **#67** is the Phase-4-Meta-Core GAP-KDB Shape-B mint (`SGD`;
> first-contact / TOFU DID-authenticity residual) whose canonical registration is its detail section at the
> foot of this document.

| Compromise # | class | Compromise # | class | Compromise # | class |
|---|---|---|---|---|---|
| Compromise #1 | `MIT` | Compromise #2 | `MIT` | Compromise #3 | `MIT` |
| Compromise #4 | `MIT` | Compromise #5 | `ATO` | Compromise #6 | `ATO` |
| Compromise #7 | `MIT` | Compromise #8 | `MIT` | Compromise #9 | `MIT` |
| Compromise #10 | `MIT` | Compromise #11 | `MIT` | Compromise #12 | `MIT` |
| Compromise #13 | `ATO` | Compromise #14 | `ATO` | Compromise #15 | `ATO` |
| Compromise #16 | `MIT` | Compromise #17 | `MIT` | Compromise #18 | `MIT` |
| Compromise #19 | `MIT` | Compromise #20 | `MIT` | Compromise #21 | `MIT` |
| Compromise #22 | `SGD` | Compromise #23 | `MIT` | Compromise #24 | `MIT` |
| Compromise #25 | `MIT` | Compromise #26 | `MIT` | Compromise #27 | `OOS` |
| Compromise #28 | `OOS` | Compromise #29 | `SGD` | Compromise #30 | `MIT` (unaudited PQ / v1-GM audit) |
| Compromise #31 | `MIT` (LAMPS Composite ML-DSA EUF-CMA-only) | Compromise #32 | `MIT` | Compromise #33 | `OOS` |
| Compromise #34 | `ATO` | Compromise #35 | `ATO` | Compromise #36 | `OOS` |
| Compromise #37 | `SGD` | Compromise #38 | `OOS` | Compromise #39 | `SGD` |
| Compromise #40 | `SGD` | Compromise #41 | `ATO` | Compromise #42 | `ATO` |
| Compromise #43 | `ATO` | Compromise #44 | `OOS` | Compromise #45 | `ATO` |
| Compromise #46 | `ATO` | Compromise #47 | `ATO` | Compromise #48 | `ATO` |
| Compromise #49 | `ATO` | Compromise #50 | `SGD` | Compromise #51 | `ATO` |
| Compromise #52 | `ATO` | Compromise #53 | `ATO` (TransportConfig reserve; gossip ships, Willow/iroh-roq/iroh-live reserved) | Compromise #54 | `ATO` |
| Compromise #55 | `SGD` | Compromise #56 | `SGD` | Compromise #57 | `SGD` |
| Compromise #58 | `CHD` | Compromise #59 | `SGD` | Compromise #60 | `SGD` |
| Compromise #61 | `CHD` | Compromise #62 | `SGD` (revocation-reach; Drops forever-valid) | Compromise #63 | `ATO` |
| Compromise #64 | `SGD` (cross-device best-effort nonce / jti replay window) | Compromise #65 | `SGD` (wave-3e per-Node AEAD publicly-derivable-`K_principal` confidentiality limit at v1-beta) | Compromise #66 | `SGD` (recovered-secret `Debug`/heap hygiene; CLOSED at v1-beta — redact+zeroize on all secret types, enforced by `f_secret_hygiene_roster`) |

This document records the security claims Benten makes through Phase
4-Foundation close and the known compromises those claims rest on. This
document is the written, referenceable form. The Phase-3-close table
below is preserved verbatim (Phase 3 surfaces remain LOAD-BEARING under
the post-Phase-4-Foundation engineering); Compromise #26 (Phase-4-Foundation
manifest-envelope recheck at the sync merge boundary) and Compromise #30
(Phase-4-Meta-Core unaudited PQ primitives in the v1-beta hybrid default,
per the 2026-05-19 PQ-default reframe) are appended at the end of the
table narrative.

> ### Reader's note — substrate-only posture in v1-beta (CLAUDE.md baked-in #18 Layer-1/2/3 plugin trust)
>
> The plugin trust model defined at CLAUDE.md baked-in #18
> ((a) user-as-root, (b) install-time manifest envelope, (c) runtime
> delegation within manifest envelope) is **SUBSTRATE-ONLY at v1-beta** in
> the artifacts this doc covers. The structural seams — `CapabilityPolicy`
> sealed trait + 3 new defaulted hooks (`check_install_consent` /
> `check_per_delegation` / `check_write_with_audience`) + the
> `ManifestEnvelopeRecheckOutcome` 4-arm + the
> `WriteBoundaryChainValidator` consumption hook — all exist and are
> wire-format-locked at G-CORE-9 freeze. **Consumer wire-in is sequenced**
> across these named-deferred destinations:
>
> - **Compromise #26** — the `ProductionManifestEnvelopeRechecker` adapter
>   TYPE + its `ProductionEngineBuilder` wiring LANDED at Core (Row D-4
>   CLOSED at R6 R1 FP-F4 §S4); what is still deferred to G-COMP-1 is its
>   substantive per-DID `PluginLibrary` + `UserDidRegistry` chain walk.
>   The DEFAULT engine still mounts `NoopManifestEnvelopeRechecker`
>   returning `NotApplicable` for resolvable peer-DIDs; the mounted
>   production adapter returns `UnresolvedDeny` for synthesized
>   `node-id:N` DIDs, and the Layer-A empty-DID short-circuit IS live.
> - **Compromise #62** — Drop bundle revocation reach (forever-valid
>   once distributed; OPEN ARCHITECTURAL TRADE-OFF mitigated by tight
>   `nbf`/`exp` + key rotation per RATIFIED-S&C §R6).
> - **V1-FROZEN-INTERFACE-DEFERRED.md Row D-1 — CLOSED** (R6 R1 FP-F4 §S1;
>   SHARPENED at R6 R2 FP-B). `WriteBoundaryChainValidator` consumption is
>   structurally-always-on at 14 WRITE entry points via the
>   `Engine::admit_write_chain` helper + the sealed `WriteAdmissionFrame`
>   (2 of the 14 are chain-bearing: `delegate_capability` + the
>   `apply_atrium_merge` per-row loop). Residual: the always-mounted
>   `NoopWriteBoundaryChainValidator` default admits every chain, so
>   Layer-1 user-as-root fires only where a deployment installs a
>   substantive validator; that validator's `UserDidRegistry`-backed
>   chain walk is the G-COMP-1 deliverable.
> - **V1-FROZEN-INTERFACE-DEFERRED.md Row D-3** — all three §8-E
>   CapabilityPolicy hooks are WIRED at v1-beta (per the Compromise #26
>   R6 R1 FP-F4 retense below): `check_install_consent` at
>   `plugin_lifecycle::install_plugin` step 3c (typed
>   `PluginInstallConsentDenied` reject), `check_per_delegation` at
>   `EngineCapsHandle::delegate_capability` (typed
>   `PluginPerDelegationDenied` reject), and `check_write_with_audience`
>   routed at all four production write sites. D-3-c is PARTIAL: the
>   `audience_did` stays `None` at the sweep write sites (per Δv3-2) and
>   is populated at `delegate_capability` only as the G-COMP-1
>   deliverable, so audience-aware `check_write` is not yet exercised
>   with a concrete audience end-to-end. (Trait signatures +
>   sealed-discipline are locked, so the remaining wire-up is
>   non-breaking.)
> - **V1-FROZEN-INTERFACE-DEFERRED.md Row D-5** — `accept_atrium_share`
>   cross-peer install seam not live; plugins consume through user-DID-
>   signed install records ONLY at v1-beta.
> - **V1-FROZEN-INTERFACE-DEFERRED.md Row D-6 — CLOSED** (R6 R1 FP-F4 §S4;
>   WIRED at R6 R2 FP-B). `benten_sync::handshake::sync_hydrate_consume_recheck_outcome`
>   is the §4.25 surface and is consumed from production `benten-engine`
>   merge code; the §4.36 merge half and the §4.25 sync-hydrate half now
>   both route through the shared `ManifestEnvelopeRecheckUnresolvedDeny`
>   ErrorCode + typed reject.
> - **V1-FROZEN-INTERFACE-DEFERRED.md Row D-88** — module-manifest
>   signatures are structurally **did:key-only 64-byte Ed25519 by format**
>   at v1-beta (the `ManifestSignature { ed25519 }` field + the exact-64-byte
>   decode in `manifest_signing.rs` + the classical `Did::resolve` issuer
>   path). A hybrid `did:benten` author CANNOT sign a verifiable manifest,
>   so — unlike the Fork-A authority path (UCAN chain-walk / rotation /
>   device-attestation / VC) — there is NO composite wire to silent-PQ-strip;
>   the manifest verify correctly stays classical `PublicKey::verify` and
>   does NOT route through `benten_id::authority_verify`. Hybrid-manifest-
>   author support is name-carried to Phase-4-Meta-Composing.
>
> The honest disclosure shape: **Layer-1/2/3 is structurally encoded at
> v1-beta as a frozen substrate; the substantive runtime enforcement
> across all admission boundaries lands at G-COMP-1**. This is per
> CLAUDE.md baked-in #15 (v1-beta tags on the frozen interface +
> structural substrate; G-COMP-1 closes the consumption gap before
> `v1-GM`). Operators / deployers consuming this binary should treat
> Layer-3 (runtime delegation envelope-recheck) as **declared-but-not-
> structurally-enforced at v1-beta** unless they wire a custom
> `ProductionManifestEnvelopeRechecker` themselves.

## Phase 4-Foundation close — compromise table

| # | Title | Phase | Status |
|---|-------|-------|--------|
| 1 | TOCTOU window bound at CALL entry + ITERATE batch boundary | 1 | Open (bounded; documented threat model). **Revisit at v1-window** per phase-3-backlog §10.1. |
| 2 | Symmetric-None + diagnostic capability (Option C) | 1 | **CLOSED** at Phase 2a 5d-J |
| 3 | `ErrorCode` enum lives in `benten-core` | 1 | **CLOSED** at Phase 1 R6 |
| 4 | WASM runtime is compile-check only | 1 | **CLOSED** at Phase 2b G7 |
| 5 | No write rate-limits; metric recorded only | 1 | Open (architectural). **Revisit at v1-window** per phase-3-backlog §10.2. |
| 6 | BLAKE3 128-bit effective collision resistance | 1 | Open (architectural bound). **Revisit at v1-window** per phase-3-backlog §10.3. |
| 7 | `[[bin]]` `required-features` gating | 1 | **CLOSED** at Phase 1 R6 |
| 8 | `Engine::call` bypasses the evaluator for CRUD handlers | 1 | **CLOSED** at Phase 2a G4-A |
| 9 | Dedup writes pure-read (sec-r1-4 / atk-3) | 1 | **CLOSED** at Phase 2b G12-E |
| 10 | Resume-time capability re-verification | 2a | **CLOSED** at Phase 2b G12-E |
| 11 | IVM views coarse-grained read-gate | 2a | **CLOSED** at Phase-3 G15-A wave-5a (per-row `IvmViewReadGate` + addendum at G20-A3 documenting `read_view_with` heuristic bound) |
| 12 | `DurabilityMode::Group` gate 5 — engine-surface default flip + bench CI promotion | 1 | **CLOSED** at Phase 3 G13-E |
| 13 | System-zone reserved-prefix rejection surface | 2a | Open (documented; minor-3). **Revisit at v1-window** per phase-3-backlog §10.4. |
| 14 | SANDBOX cold-start cost (no opt-in pool) | 2b | Open (D3 RESOLVED — additive Phase-3 change if real-workload bottleneck). **Revisit at v1-window** per phase-3-backlog §10.5. |
| 15 | `register_runtime` reserved with deferred error | 2b | Deferred to Phase 8 (marketplace) — named destination per phase-3-backlog. |
| 16 | `random` host-fn deferred (no CSPRNG framework chosen) | 2b | **CLOSED** at Phase-3 G17-A2 wave-5b (CSPRNG via `getrandom` direct + capability-gated entropy budget per call: 4096 bytes default + per-manifest override at `host_fns.random.budget_bytes_per_call` per r1-wsa-8; constant-time cap-policy check per sec-r1-3) |
| 17 | In-memory module-bytes registry (`Engine::register_module_bytes`) | 2b | **CLOSED** at Phase-3 G14-C wave-4b (durable `RedbBlobBackend` + CID-validating entry point) |
| 18 | In-memory handler-version chain (`Engine::register_subgraph_replace`) | 2b | **CLOSED** at Phase-3 G14-C wave-4b (durable `system:HandlerVersion` zone + extensible canonical-bytes encoding per arch-r1-4 / D-C) |
| 19 | Browser-target persistent storage absent — manifests in-memory only on `wasm32-unknown-unknown` | 2b | **PARTIALLY CLOSED** at Phase-3 G18-A wave-5a (IndexedDB schema + handler scaffolding; full closure deferred per phase-3-backlog §4.3) |
| 20 | Cross-browser determinism CI cadence not yet established | 2b | **PARTIALLY CLOSED** at Phase-3 G18-A wave-5a (Playwright matrix workflow exists; fixture bodies deferred per phase-3-backlog §4.3) |
| 21 | Module manifest minimal CID-pin in Phase 2b; full Ed25519 deferred | 2b | **CLOSED** at Phase-3 G14-C wave-4b (Ed25519 sign + UCAN-proof-chain primary + publisher-key-registry fallback per D-PHASE-3-20 + crypto-minor-5) |
| 22 | Peer-DID + connection metadata leakage to public iroh relays | 3 | Introduced at Phase 3 (Phase 7 Garden-relay closure target). **Revisit at v1-window** per phase-3-backlog §10 (Phase-7 Garden-relays primary closure path; Phase-9 hardened-deployment fallback). |
| 23 | Wire device-attestation envelope cryptographic closure | 3 | **SUPERSEDED-BY-COLLAPSE** (refinement-audit-2026-05 S3; device-trust pipe deleted, provenance + ceiling retained on unified spine) |
| 24 | Wallclock fail-closed posture (no default-clock-zero expiration bypass) | 3 | **CLOSED** at Phase-3 G16-B-B-rest (PR #158); engine refuses to initialize UCAN backend without explicit clock injection — surfaces `E_UCAN_CLOCK_NOT_INJECTED` |
| 25 | HLC-monotonic enforcement at sync layer (adversarial-peer wallclock-injection defense) | 3 | **CLOSED** at Phase-3 sync-attack test family (HLC monotonicity + nonce-cache for replay defense + HLC bound inside signed envelope) |
| 26 | Manifest-envelope recheck at sync merge boundary (Phase-4-Foundation plugin-DID principal extension) | 4-Foundation | **PARTIALLY CLOSED** at Phase-4-Foundation R4b-FP-1 Seam 3 (post-Q4 ratification 2026-05-13). The `ManifestEnvelopeRechecker` port + always-firing default-flip ship; the production-default `NoopManifestEnvelopeRechecker` returns `Outcome::NotApplicable` for every row at HEAD, so the substantive Layer-2 defense is NOT live in shipped binaries — only the per-row `CapabilityPolicy::pre_write` check from Compromise #2 sync-replica sub-narrative is. **RETENSED at R6 R1 FP-F4 §S4 (Row D-4 CLOSED):** the `ProductionManifestEnvelopeRechecker` adapter TYPE + its `ProductionEngineBuilder` wiring LANDED at Phase-4-Meta-Core (`crates/benten-engine/src/production_manifest_envelope_rechecker.rs`), returning `UnresolvedDeny` for synthesized `node-id:N` peer-DIDs and `NotApplicable` for resolvable peers. What remains G-COMP-1-deferred per `docs/future/phase-4-backlog.md §4.36` is that adapter's substantive per-DID chain walk (consuming `PluginLibrary` + `UserDidRegistry` + invoking `manifest_envelope_chain_validation::validate_chain_with_manifest_envelope`) — and the DEFAULT engine still mounts the Noop, so Layer-2 is opt-in at v1-beta. See body for the full seam-vs-adapter shape. |
| 27 | (RESERVED for META #669 closure — Plugin trust model Layers 2+3 + T10-upgrade paper-only at HEAD) | 4-Foundation | **OPEN; tracking via [META #669](https://github.com/BentenAI/benten-engine/issues/669) + [#1118](https://github.com/BentenAI/benten-engine/issues/1118) Compromise #27 mint task.** Reserved row; row body lands when META #669 closure or honest-disclosure mint lands. |
| 28 | (RESERVED for META #629 closure — DoS-via-unbounded-decode workspace pattern; 26 instances / 9 crates) | 4-Foundation | **OPEN; tracking via [META #629](https://github.com/BentenAI/benten-engine/issues/629) + [#1126](https://github.com/BentenAI/benten-engine/issues/1126) Compromise #28 mint task.** Reserved row; row body lands when META #629 closure or honest-disclosure mint lands. |
| 29 | Engine-level extensions — compile-time trust posture (CLAUDE.md baked-in #19) | 4-Foundation | **OPEN ARCHITECTURAL COMMITMENT; registry-tracked for cross-reference completeness.** Engine extensions are Rust crates compile-time linked into the engine binary; trust is `cargo` + code review, not the type system. Future post-Ed25519 / post-iroh / post-redb / post-wasmtime engine-extension migrations land under this Compromise's namespace per Phase-9+ scope. The trust model is comprehensively narrated below at §"Engine-level extensions — compile-time trust"; this row makes the claim registry-discoverable for the §3.12 R7-equivalent audit walk. Tracking via [#1131](https://github.com/BentenAI/benten-engine/issues/1131). |
| 30 | Unaudited PQ primitives in the v1-beta hybrid default (`ml-dsa` signature half / `libcrux-ml-kem` KEM half have no independent third-party audit yet) | 4-Meta-Core | **OPEN; MITIGATED by hybrid construction.** v1-beta ships PQ-hybrid by default (sig Ed25519⊕ML-DSA-65 byte-faithful IETF LAMPS composite `id-MLDSA65-Ed25519-SHA512`, both-must-verify, NO commitment trailer; enc X25519⊕ML-KEM-768 at codepoint `0x647a` + ChaCha20-Poly1305) where the PQ halves are not yet independently audited. Mitigation: the **classical half is the audited security floor** (Ed25519 / X25519 + the NCC-audited ChaCha20-Poly1305 AEAD); the **signature** hybrid is strip-resistant via the shared-`M'`/ctx=Label binding + both-halves-required, and the **encryption** combiner is **strip-resistant** (both shared secrets — `ss_M` ‖ `ss_X` — are mixed into the KEK, so neither KEM half can be stripped without changing the derived key; full AEAD key-commitment in the robustness sense is OUT OF SCOPE at v1-beta) — both fail closed (the typed-unsupported-arm-never-silent-fallback contract enforces it) so **unaudited PQC is never the SOLE trust path**. The KEM half is now `libcrux-ml-kem` (the 13 net-new transitive crates — 10 Cryspen/libcrux/hax + 3 general proc-macro support — join the C-GM-AUDIT scope as honest cargo-vet exemptions, budget cap 5→18). **CLOSES at v1-GM** when the independent `ml-dsa`/`ml-kem`(`libcrux-ml-kem`) audit lands (NF-2 / C-GM-AUDIT exit criterion). Per the 2026-05-19 PQ-default reframe (`.addl/pq-research/RATIFIED-pq-default-reframe-2026-05-19.md`; CLAUDE.md baked-in #5 / #15). Tracking via the v1-beta PQ-audit issue [#1302](https://github.com/BentenAI/benten-engine/issues/1302) + [#1300](https://github.com/BentenAI/benten-engine/issues/1300) / [#1301](https://github.com/BentenAI/benten-engine/issues/1301). |
| 31 | LAMPS Composite ML-DSA combiner is EUF-CMA-only NOT SUF-CMA (CLOSED-equivalent via Inv-15 application-layer 3-layer decomposition) | 4-Meta-Core | **OPEN at construction layer; CLOSED-EQUIVALENT at application layer via Inv-15.** The v1-beta default signature combiner (LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512` at `SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001`) is EUF-CMA-secure but NOT SUF-CMA-preserving (Weakly-Non-Separable per `draft-ietf-lamps-pq-composite-sigs-19` §10). The SUF-CMA gap is closed at the application layer via **Inv-15** (sig-bundle CIDs are never load-bearing identifiers; identity = canonical-payload-CID, authentication = codepoint-dispatched signature, revocation = semantic tuple). SUF-CMA-preserving combiners (Bird-of-Prey) are reserved as future-additive codepoints. See the body section "Compromise #31 — LAMPS Composite ML-DSA combiner …" + [`INVARIANT-COVERAGE.md`](INVARIANT-COVERAGE.md) Inv-15. |
| 32 | ML-KEM-768 Decap chosen-ciphertext side-channel (libcrux CT-mitigation) — a Decap-axis refinement of #30 | 4-Meta-Core | **OPEN; MITIGATED-LIVE.** Decap-axis refinement of the unaudited-PQ window (#30); explicitly cross-linked #30↔#32↔C-GM-AUDIT (M-5). `libcrux-ml-kem` is now the LANDED production ML-KEM-768 impl (hax/F*-verified, constant-time portable backend selected on wasm + non-SIMD) — moving #32 from deferred → mitigated-live. **#32-residual:** the runnable `check-secret-independence` CI gate is NOT wireable at `libcrux-ml-kem 0.0.10` (upstream E0053 macro defect — first reproduced at 0.0.9, PERSISTS at 0.0.10, re-verified 2026-07-20); the verified portable backend is the live mitigation, and the runnable gate (`mlkem-ct-check` feature seam) carries to the libcrux version that fixes the macro (f_kat_2 FLAG-FOR-BEN). Closes with the #30 v1-GM audit. Encryption arc (9-eyes panel). See body section. |
| 33 | Coercion / wrench attack OUT-OF-SCOPE — incl. Layer-D approval-coercion (coerced-approving-device `RemoteUnlock`/`SignUcanDelegation`) | 4-Meta-Core | **OUT-OF-SCOPE (disclosed).** Password/physical coercion is outside the cryptographic threat model. m-5 extension: a coerced approving-device makes a coerced grant look legitimate forever via the audit-Node (distinct from #34 password-coercion). 9-eyes. |
| 34 | Password-knowledge implies full access (Argon2id defense-in-depth) | 4-Meta-Core | **ACCEPTED TRADE-OFF.** Whoever knows the principal password derives the DAK; Argon2id raises the offline-guess cost but does not change the knowledge-implies-access property. 9-eyes. |
| 35 | Compromised-device retroactive decryption (no past-content forward-secrecy at v1-beta; CGKA deferred) | 4-Meta-Core | **ACCEPTED TRADE-OFF.** A device whose long-term key is compromised can retro-decrypt content it held; full PCS/CGKA is deferred post-v1-beta. 9-eyes. |
| 36 | RAM-residency / coredump / swap forensic-extraction OUT-OF-SCOPE (`zeroize`+`secrecy` best-effort) | 4-Meta-Core | **OUT-OF-SCOPE (disclosed).** Plaintext-in-RAM extraction via coredump/swap is outside scope; `zeroize` + `secrecy` are best-effort hardening, not a guarantee. 9-eyes. |
| 37 | No TEE / sealed-enclave attestation at v1-beta + v1-GM | 4-Meta-Core | **SUBSTRATE-GUARANTEE DISCLOSURE.** Benten makes no TEE/enclave attestation claim; key material rests in process memory protected only by OS boundaries. 9-eyes. |
| 38 | Physical-presence side-channels OUT-OF-SCOPE | 4-Meta-Core | **OUT-OF-SCOPE (disclosed).** EM/power/acoustic/timing physical side-channels are outside the threat model. 9-eyes. |
| 39 | Supply-chain dependency-pinning posture (PARTIAL; `cargo deny` + RustSec; new `secrecy` Layer-A dep) | 4-Meta-Core | **SUBSTRATE-GUARANTEE DISCLOSURE.** Dependency pinning is PARTIAL: `cargo deny` + RustSec advisory gate. **The live path uses NO `hpke` crate** (R10-council GAP-A correction) — the Benten-supplied X-Wing KEM-DEM is the wire (NQ-C1); the McMillion `hpke` (NOT Cryspen `hpke-rs` w/ 13 CVEs) pin is RESERVED for the additive RFC-9180-faithful key-schedule branch ONLY, if/when NQ-C1 ratifies it. O-1: `secrecy` is a new Layer-A dependency disclosed here. 9-eyes; O-1. |
| 40 | Build-time / reproducible-builds + SLSA-3+ posture (post-v1-GM) | 4-Meta-Core | **SUBSTRATE-GUARANTEE DISCLOSURE.** Reproducible-builds + SLSA-3+ provenance are post-v1-GM commitments, not v1-beta guarantees. 9-eyes. |
| 41 | Cross-device-sync UX-vs-cryptographic boundary — incl. revocation-propagation-lag (O-4) | 4-Meta-Core | **ACCEPTED TRADE-OFF.** The cross-device sync UX surface and the cryptographic boundary differ; O-4 sub-clause: a revoked device can exercise a stale grant during a partition (bounded by tight `exp`). 9-eyes; O-4. |
| 42 | Layer-C forward-secrecy gap (HPKE-mode-base recipient long-term sk decrypts forever) | 4-Meta-Core | **ACCEPTED TRADE-OFF.** HPKE-mode-base is structurally non-FS at the long-term-sk axis: a 2030 sk-compromise recovers 2026 envelopes. Partial FS via app-layer key rotation; full per-message FS is the SEPARATE #56 journalist class. 9-eyes (U13). |
| 43 | Envelope metadata leakage to untrusted relays — IMPROVED by Sealed-Sender DEFAULT (BR-1) | 4-Meta-Core | **ACCEPTED TRADE-OFF; IMPROVED.** Sealed-Sender DEFAULT (`0x6510`) removes plaintext sender-DID on the default path; NO coarse-epoch on the Drop wire (1-hr bucket is Layer-D-only — RULING-1 / M-14); group-AAD set-identifying material is BLINDED (audience_set_commitment + membership_set_id_commitment per #61). Residual on the default Drop wire = recipient DID + linkable-but-blinded group tags + the plaintext **`body_cid`** (unsalted `BLAKE3(body)` — a confirmation-oracle + equality-linker for LOW-ENTROPY bodies; app-layer padding / additive per-send salt mitigate) (roadmap U22–U28; full per-send unlinkability = U25 v1-GM-reserve). 9-eyes (L6); BR-1; #61. |
| 44 | Long-term-confidentiality posture (BSI TR-02102-1; X-Wing/MLKEM768-X25519 acceptable-migration-window) | 4-Meta-Core | **OUT-OF-SCOPE (disclosed).** The very-long-term (decades) confidentiality horizon is outside the v1-beta posture; the hybrid KEM acceptable-migration-window is disclosed per BSI TR-02102-1. 9-eyes. |
| 45 | ML-KEM-768 MAL-BIND-K-CT / K-PK binding-properties (connects to IND-CCA2-adversarial-recipient-seed) | 4-Meta-Core | **ACCEPTED TRADE-OFF.** ML-KEM-768 binding properties (MAL-BIND-K-CT / K-PK) connect to the M-6 IND-CCA2-adversarial-recipient-seed audit line; this is an external-cryptographer-audit disclosure surface (NOT a unit-test "proof"). MembershipSet panel; M-6. |
| 46 | `HpkeMultiBase` O(N) wire-cost above 32 recipients (Atrium 32 / DeviceMesh 5 / SingleDevice 1) | 4-Meta-Core | **ACCEPTED TRADE-OFF.** Multi-stanza HPKE wire-cost grows linearly with recipient count; per-Kind cardinality caps bound it (Atrium 32 / DeviceMesh 5 / SingleDevice 1). MembershipSet panel. |
| 47 | Collaborative-edit-via-re-drop accepted v1-beta trade-off | 4-Meta-Core | **ACCEPTED TRADE-OFF.** Collaborative edits propagate via re-drop rather than a shared mutable cipher object at v1-beta. MembershipSet panel. |
| 48 | MembershipSet-shape-leak (shared_key compromise → fingerprints that generation; recovery via fork) | 4-Meta-Core | **ACCEPTED TRADE-OFF.** A `shared_key` compromise reveals that generation's membership-set shape (a fingerprint); recovery is via FORK-ONLY rotation (Inv-20/Inv-21). MembershipSet panel. |
| 49 | MembershipSet-member-acting-as-storage-host trust-boundary collapse | 4-Meta-Core | **ACCEPTED TRADE-OFF.** When a set-member is also the storage host, the member↔host trust boundary collapses (the host sees member-visible plaintext it was already entitled to). MembershipSet panel. |
| 50 | Permanence-stewardship dependency disclosure | 4-Meta-Core | **SUBSTRATE-GUARANTEE DISCLOSURE.** Content permanence depends on at-least-one-peer-stewards-the-bytes; Benten makes no centralized-durability guarantee. MembershipSet panel. |
| 51 | Tauri / NAPI-RS marshaling-boundary side-channels | 4-Meta-Core | **ACCEPTED TRADE-OFF.** The Tauri/NAPI-RS FFI marshaling boundary is a potential timing/side-channel surface for key material crossing the JS↔Rust line; disclosed, not closed at v1-beta. MembershipSet panel. |
| 52 | MembershipSet-no-PCS-against-removed-members (fork-on-kick) + in-set role-downgrade subsume | 4-Meta-Core | **ACCEPTED TRADE-OFF.** Removing a member does NOT provide post-compromise security against that member for content they already held; recovery is fork-on-kick (re-key via FORK). In-set role-downgrade is subsumed (a downgraded member retains prior-derived keys). MembershipSet panel (M-C3 F-FE-5). |
| 53 | TransportConfig-codepoint reserve (NARROWED): iroh-gossip SHIPS; Willow / iroh-roq / iroh-live RESERVE | 4-Meta-Core | **ACCEPTED TRADE-OFF.** Only `GossipPlusBlobs` ships at v1-beta; the Willow / iroh-roq / iroh-live `TransportConfig` variants are reserved-and-typed-rejected (additive codepoints, no wire-break when added). MembershipSet panel (Ben Q1). |
| 54 | Continuous-rotation deferral + post-v1-beta `AtriumWithRotatingGroupKey` revisit-trigger | 4-Meta-Core | **ACCEPTED TRADE-OFF.** Continuous group-key rotation (CGKA-style) is deferred; the named revisit-trigger is `AtriumWithRotatingGroupKey` post-v1-beta. MembershipSet panel (N3). |
| 55 | GDPR-RTBF honest-architectural-disclosure (P2P-by-design; apps-layer crypto-shredding) | 4-Meta-Core | **SUBSTRATE-GUARANTEE DISCLOSURE.** Right-to-be-forgotten is architecturally constrained by P2P-by-design (no central delete); the apps-layer remedy is crypto-shredding (destroy keys, leave ciphertext). MembershipSet panel. |
| 56 | Journalist per-message forward-secrecy deferral (SEPARATE design class from group-key rotation) | 4-Meta-Core | **SUBSTRATE-GUARANTEE DISCLOSURE.** Per-message FS (journalist threat class) is a SEPARATE design class kept sharply distinct from #42 (HPKE non-FS) and #52 (fork-on-kick) per R1-Q-8; deferred post-v1-beta. MembershipSet panel (MINT confirmed). |
| 57 | RestrictedScopeSet / grant immutability honest-disclosure (absorbs / cross-links #62) | 4-Meta-Core | **SUBSTRATE-GUARANTEE DISCLOSURE.** A RestrictedScopeSet grant is immutable once issued; narrowing requires re-issue. Cross-links / absorbs the #62 revocation-reach property. MembershipSet panel. |
| 58 | Audit-log insider-correlation (admin full visibility; per-recipient-unlinkability is network-observer-only) | 4-Meta-Core | **COMPOSITION-HAZARD HONEST DISCLOSURE.** An admin with audit-log visibility can correlate members; per-recipient-unlinkability is **network-observer-only** (m-7), NOT admin-hidden. Threshold-admin opt-in closes the insider vector. MembershipSet panel (M-C2-B-3 + P5). |
| 59 | KEM-key-confirmation under multi-stanza-HPKE-Encap — re-scoped to Sealed-Sender abuse-control surface (BR-1; see #63) | 4-Meta-Core | **SUBSTRATE-GUARANTEE DISCLOSURE.** KEM key-confirmation under multi-stanza HPKE-Encap; re-scoped by BR-1 to the Sealed-Sender abuse-control surface (the abuse-control residual lands at #63). MembershipSet panel (M-C3); BR-1. |
| 60 | RBAC-role-transition does not invalidate prior UCAN attenuations — composition note (m-6) | 4-Meta-Core | **SUBSTRATE-GUARANTEE DISCLOSURE.** An RBAC role-transition does NOT retroactively invalidate already-issued UCAN attenuations; under ship-all-5 + remote-permission an ephemeral UCAN survives a role-downgrade and is bounded ONLY by `exp` ⇒ tight-`exp` default mandate (§3.4; cross-link #52). MembershipSet panel (M-C3 F-FE-2); m-6. |
| 61 | MembershipSet-fingerprint-leak via iroh-gossip topic (CLOSED by HMAC-blinded topic) | 4-Meta-Core | **COMPOSITION-HAZARD HONEST DISCLOSURE; CLOSED.** The iroh-gossip topic would have leaked a membership-set fingerprint; CLOSED by P2 D6 HMAC-blinded (`blake3::keyed_hash`) gossip topic — set-identifying material is never published in the clear. MembershipSet panel (M-C3). |
| 62 | Revocation reach in encryption-at-rest (already-derived keys remain decryptable; Drop bundles forever-valid once distributed) | 4-Meta-Core | **RE-POINTED from the in-tree #31 occupant (BR-2). OPEN ARCHITECTURAL TRADE-OFF; MITIGATED by tight UCAN `nbf`/`exp` + key rotation.** Per RATIFIED-S&C §R6 + V1-FROZEN-INTERFACE.md item 15(i): UCAN revocation cuts FUTURE serves (the per-request `CapabilityPolicy::check_read` consultation fails for subsequent requests against the granted CID), but already-derived keys remain decryptable forever. Once Bob has derived `K(N)` for some Node, Bob can decrypt any ciphertext he obtains for that Node, regardless of subsequent UCAN revocation. Re-keying the Node requires Alice to re-encrypt + re-issue (a heavy operation; per-Node + per-recipient cost scales). Drop bundles are forever-valid once distributed — the producer has no callback to revoke an already-distributed Drop. **Mitigation:** tight UCAN `nbf`/`exp` windows + key rotation discipline + the typed `E_UCAN_BLOBS_REQUEST_REJECTED` server-side gate. **Stays OPEN at v1-beta + v1-GM** — this is an inherent property of encryption-at-rest where the reader holds plaintext key material; closing it would require structural changes (e.g. forward-secret re-keying on every revocation; MLS-style per-message keys) that are out of scope for v1. Authored at G-CORE-9 V1-FROZEN-INTERFACE row 8e per Ben morning queue item; tracking via the V1-FROZEN-INTERFACE.md item 15(i) FREEZE-WAVE FIX-NOW. Cross-linked #57. |
| 63 | Sealed-Sender abuse-control trade-off (no plaintext sender ⇒ abuse-control rides recipient-issued delivery tokens) | 4-Meta-Core | **NEW (BR-1; §3.11).** The DEFAULT Sealed-Sender path (`0x6510`) carries no plaintext sender identity, so abuse/spam control cannot use per-sender filtering; it rides recipient-issued short-lived rate-limited UCAN-backed delivery tokens (refused at the receive boundary BEFORE decrypt). Residual: a recipient who over-issues tokens re-admits spam (mitigated by default-conservative token rate-limits + per-token `nbf`/`exp` + revocation). See body section. |
| 64 | Cross-device best-effort-eventual nonce-rejection window (NQ-T4) | 4-Meta-Core | **NEW (NQ-T4; Ben-ratified 2026-06-02; minted this cascade). `SGD` substrate-guarantee disclosure.** The `jti`-keyed nonce-cache is per-device-durable-GUARANTEED **via the durable-CAS-marker + `from_durable` hydration seam** (`JtiNonceCache`; the engine honors a caller-contract to persist `durable_snapshot()` + re-hydrate on restart — the full disk-persistence wiring is deferred with the remote-permission wiring, Row D-64-adjacent) but user-global only best-effort-eventual-via-sync (NOT synchronous): a nonce consumed on device B is rejected on device C only after the consumed-`jti` set propagates via sync. The pre-sync cross-device window admits a one-time replay of a remote-permission / DeviceLink token across the user's own devices. Mitigated by: durable per-device rejection via the seam (no same-device replay once persisted+hydrated), tight `valid_until` (full-second granularity, strict, no skew window — NQ-T2), and short delivery-token `exp`. **Stays OPEN at v1-beta + v1-GM** — synchronous user-global rejection would require a consensus/online-coordinator the P2P model deliberately avoids. See body section. |
| 65 | Wave-3e per-Node AEAD publicly-derivable-`K_principal` confidentiality limit at v1-beta (untrusted host CAN read partition plaintext) | 4-Meta-Core | **NEW (R13 F-07; `SGD` substrate-guarantee disclosure; minted to match the THREAT-MODEL untrusted-host honesty retense, R13 F-06).** At v1-beta the per-Node AEAD wrap does NOT provide confidentiality against a malicious *storage host*: the wave-3e `K_principal = blake3::keyed_hash(K_PRINCIPAL_DOMAIN_KEY, namespace_did)` is derived from a **publicly-known** 32-byte domain-tag constant (`K_PRINCIPAL_DOMAIN_KEY`, `crates/benten-graph/src/redb_backend.rs:181`) + the **publicly-known** `namespace_did`, so `K_principal` — and thus `K(N)` + the per-Node AEAD key — is **publicly derivable**: any party holding `(namespace_did, ciphertext_blob)` can derive the key and decrypt. Per CLAUDE.md baked-in #18 the **confidentiality half** of the Principal primitive (per-principal encryption of the storage partition; the #1301 / D-64 substrate) is **DEFERRED — NOT built at v1-beta**; the LIVE protection is the **AUTHORITY half** (capability / namespace isolation) which binds only a **cooperating** engine. So per-Node AEAD is a publicly-derivable-`K_principal` **STAND-IN** keeping the substrate shape stable for the production `K_principal`-store swap-in, NOT real untrusted-host confidentiality. Mitigated in the interim by namespace-isolation at the storage backend (the AUTHORITY half) + the local device's Layer-A vault (Argon2id-DAK-sealed, protecting the *local* vault at rest). **Stays OPEN at v1-beta; CLOSES when the #1301 / D-64 per-DID secret-material `K_principal` backend lands** (the swap-in replaces only the `K_principal` synthesis step — the function signature + AEAD-wrap layer + per-chunk size are all stable). Full narration: the "⚠️ Confidentiality limit at this wave" disclosure in the **Per-Node AEAD wrap layer** section below (`derive_test_seam_key_from_cid_with_namespace`). Cross-linked from `docs/THREAT-MODEL.md` §1 (the untrusted-host row + honesty note). Named carry: `docs/future/phase-4-backlog.md §3.10`. |
| 66 | Recovered-secret `Debug`-render + freed-heap hygiene across the crypto-suite secret roster | 4-Meta-Core | **MINTED R14 GAP-1; CLOSED-at-v1-beta (hardened in the R19/#3 secret-hygiene sweep; `SGD` substrate-guarantee disclosure — now a positive guarantee, not an open gap).** History: R14 disclosed that `UnwrappedKey` (`crates/benten-crypto-suite/src/cipher_suite.rs`; the recovered `k_root` from `unwrap_key_material`) then carried `#[derive(Debug)]` (a `{:?}` render would print recovered KEY BYTES) with no zeroize-on-drop. The R19/#3 sweep HARDENED the whole recovered-secret roster: `UnwrappedKey` (redacting `impl Debug` → `<redacted>` + zeroizing `impl Drop`, `cipher_suite.rs`), `DecryptedPlaintext` (`cipher_suite.rs`), `VaultPayload` (`k_principal` + `user_did_signing_key` redacted + zeroized, `vault.rs`), `ProvisioningInnerPayload` (`device_link.rs:120-149`), and `PurePqMlKemKeypair` (zeroize-on-drop landed R18 C3). Enforced by the LIVE meta-test `crates/benten-engine/tests/f_secret_hygiene_roster.rs` (470 LOC, zero `#[ignore]`) — a runtime Debug-does-not-leak assertion over the full roster + a source-anchored zeroize-coverage grep-defense — plus an in-crate `<redacted>`-render assertion at `cipher_suite.rs`. So the recovered-secret `Debug`/heap hygiene is a positive v1-beta guarantee; a revert (e.g. re-deriving `Debug`) re-fires the meta-test. The residual v1-GM nicety is narrower: R6-reround `Zeroizing`-wrapped the vault seal/open transient plaintext buffers (`serialize_vault` + `decode_vault`/`open_vault` `pt`, each holding the full `k_principal` + `user_did_signing_key`, `vault.rs`) and the `benten-drop` Layer-C seal-side CEKs (`layer_c.rs`), so no transient full-secret plaintext or seal-side CEK is left un-wiped; what remains Row-D-75-deferred is the freeze-coupled still-bare-`Vec<u8>` copy sites (`unwrap_key_from_recipient` return, the `device_link.rs` `recovered` binding, `swap_matrix.rs:563`) — tracked at `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-75. The separate `derive_member_key` raw-`Vec<u8>` hardening rides Row D-76. See body section. |
| 67 | First-contact / TOFU DID-authenticity bootstrap (GAP-KDB Shape-B residual) | 4-Meta-Core | **MINTED at the Phase-4-Meta-Core GAP-KDB Shape-B identity-model closure; `SGD` substrate-guarantee disclosure; OPEN residual — REDUCED, not eliminated.** Shape-B (the content-addressed `did:benten` key-set + `Did::resolve_kem` + the `RecipientBinding` sole-constructor typestate + Inv-23) closes *post*-first-contact key substitution: swapping the recipient KEM key under a `did:benten` a sender already holds requires a BLAKE3-256 2nd-preimage over the canonical DAG-CBOR key-set. It does **NOT** close *first-contact* DID-authenticity — whoever controls the channel where a sender FIRST learns "Alice ↔ `did:benten:…`" can hand them their own self-consistent DID, which then resolves and verifies cleanly. The confidentiality trust window narrows from continuous to **bind-once**, the identical residual carried by Signal safety numbers, MLS and PGP fingerprints; authenticating the initial DID↔principal binding is **out-of-band and the user's responsibility** (Benten ships no PKI / CA and cannot authenticate the first contact). The bare-`did:key` / Shape-A fallback has the SAME residual. **Stays OPEN at v1-beta + v1-GM.** Cross-ref: Inv-23 in [`INVARIANT-COVERAGE.md`](INVARIANT-COVERAGE.md); [`SECURITY-PROOFS.md`](SECURITY-PROOFS.md) §4.1; [`THREAT-MODEL.md`](THREAT-MODEL.md) (the DISTINCT seal-path revocation-reach residual is #62, NOT this one). Full narrative in the body section at the foot of this document. |

**Refinement-audit-2026-05 delta:** Compromise #29 (engine-extension trust model, narrative-only at HEAD; now registry-tracked) + reserved rows #27 / #28 added post-tag to anchor META #669 + META #629 closure mints. The v1-platform-shippable BLOCKER cluster framing lives in the local-only campaign-summary `refinement-audit-2026-05.md §15` (gitignored under `docs/future/*` — internal methodology artifact, not publicly shipped; see `docs/future/phase-4-backlog.md §4.86`, the tracked-repo destination that owns the §15.5 pim-N catalog tracking).

**PQ-default-reframe-2026-05-19 delta:** Compromise #30 (unaudited PQ primitives in the v1-beta hybrid default) is the sole net-new addition from the 2026-05-19 PQ-default reframe — it names the pre-audit window as a tracked, registry-discoverable compromise that is MITIGATED by the classical-floor hybrid construction and CLOSES at the v1-GM independent audit (NF-2 / C-GM-AUDIT). The hash posture (Compromise #6) is **UNAFFECTED** by the reframe (the reframe is signature + encryption only). See `.addl/pq-research/RATIFIED-pq-default-reframe-2026-05-19.md`.

**Phase-4-Foundation delta:** Compromise #26 is the sole Phase-4-Foundation-era net-new addition. The remaining surfaces (#1-25) inherit Phase-3 posture unchanged. Phase-4-Foundation engineering shipped without introducing a sixth-class principal type at the evaluator boundary — the plugin-DID principal extension routes through the existing `CapabilityPolicy` + manifest-envelope-chain-validation seams.

**Phase-2b net delta:** Compromises #4 + #9 + #10 closed (3 net
closures); 8 new Phase-2b deferrals enumerated (#14, #15, #16, #17,
#18, #19, #20, #21) — all named, all destination-tagged. Compromises
#19, #20, #21 were lifted from MODULE-MANIFEST.md's local "#N+X" table
into the global numbering at R6 phase-close so cross-doc references
resolve to a single authoritative compromise table.

**Phase-3 G13-E delta (this row's landing):** Compromise #12 closed
at Phase-3 R5 wave-3 G13-E — `DurabilityMode::default()` flipped
`Immediate` → `Group` at the engine surface +
`.github/workflows/bench.yml` promoted from informational to
required (PR-trigger compile gate + CRUD fast-path APFS-relevant
bench subset). See the Compromise #12 section below for the full
closure narrative + the redb-collapse caveat.

**Phase-3 additive delta (introduced at phase-3 close, this row's
landing):** Compromise #22 records the peer-DID + connection-metadata
leakage to public iroh relays exposed by the Atrium P2P sync transport.
Full narrative below; closure target is the Phase-7 Garden-relay
infrastructure (with Phase-9 hardened-deployment as the brutal-but-
correct fallback if Garden-relays slip).

The detailed text for each numbered Compromise follows below. Phase-2b
additions (#14-#16) appear at the end of the Named Compromises
section. Phase-3 addition (#22) appears at the very end.

**Attack-surface matrix cross-reference.** The doc-level enumeration of
every named attack surface (Phase-2b SANDBOX ESC-1..16 + Phase-3 P2P-sync
surfaces) lives at [`docs/ATTACK-SURFACE-MATRIX.md`](ATTACK-SURFACE-MATRIX.md)
(authored at Phase-3 R5 wave-9 W9-T2 closing
`docs/future/phase-3-backlog.md` §7.13 sec-r4r2-2 / sec-r4r1-4). This
file remains the authoritative single-source-of-truth for the
per-compromise prose; the matrix complements it by serving as the
meta-completeness audit destination at every R6 phase-close (a checklist
that every named attack surface has at least one driving test pin).

## Named Compromises

### Compromise #6 — BLAKE3 128-bit effective collision resistance

Benten uses **BLAKE3-256** with a 32-byte digest embedded in every CIDv1. The academic collision-resistance bound for any cryptographic hash is `2^(n/2)` (birthday bound), giving BLAKE3-256 a **128-bit effective collision resistance**. This is the bound that every Benten Phase-1 security argument rests on — NOT the full `2^256` preimage bound.

**Where this matters:**

- **Content-addressed Nodes (`Cid`).** A collision would allow a malicious writer to forge a Node that hashes to the same CID as a legitimate Node — a "masquerade" attack. 128-bit resistance requires ~`2^128` hashes to find a collision; infeasible under any classical threat model.
- **Version-chain `prior_head` threading** (`benten_core::version::append_version`). The API uses CIDs to name the head each writer observed. A collision on a CID used as `prior_head` could, in principle, let an attacker smuggle an alternative chain past the fork-detection check. The same 128-bit bound applies.
- **Phase 3 UCAN-by-CID.** Phase 3 references capability grants by CID (landed at G14-B wave-5a durable UCAN backend). Revoke-by-CID paths assume the CID of a grant is unique; again, 128-bit collision resistance is the assumption.
- **Inv-21 fork-tie-break antisymmetry (R15 F-24 cross-ref).** The MembershipSet fork-tie-break comparator (`docs/INVARIANT-COVERAGE.md` Inv-21) requires distinct-fork-event ⇒ distinct-CID for its totality/antisymmetry (the CID final tie-break byte); a collision between two genuinely-distinct forks would defeat the agreed-winner property. This 128-bit collision-resistance bound is exactly that assumption for the fork-tie-break axis.

**What this posture does NOT claim:**

- **Quantum resistance.** Grover's algorithm reduces the effective collision bound to `2^64` under a quantum adversary. This is still infeasible for the current state of quantum hardware, but it is no longer "categorically" secure. A post-quantum hash option is a Phase N+ consideration; BLAKE3 is not post-quantum.
- **Second-preimage resistance stronger than 128 bits.** For adversaries who already know a target CID and wish to construct a colliding Node, the dominant-term bound is still ~`2^128`. Benten does not rely on the higher 256-bit preimage bound.

**Phase 2 action items:**

- Mirror this posture into end-user docs (`docs/QUICKSTART.md` security section).
- Document the same assumption in the TypeScript wrapper's JSDoc for `@benten/engine` node-creation APIs.
- When Phase 3 introduces the UCAN-by-CID path, restate the bound at that integration point.

#### Hash algorithm choice — BLAKE3 (options considered)

Benten uses **BLAKE3-256** specifically (multihash code `0x1e`), not SHA-256 (`0x12`). The decision was made with explicit awareness of the interop tradeoffs:

- **Option A — BLAKE3 only (chosen).** Native `iroh` P2P transport (Phase 3) uses BLAKE3 throughout; zero hash-translation at the network boundary. ~10× faster than SHA-256 on modern CPUs (SIMD + tree hashing). Parallel-chunkable — large blobs verify in parallel via BLAKE3's tree structure. IPLD-format compatible (CIDv1 with multihash `0x1e`).
- **Option B — SHA-256 only.** Maximum compatibility with default IPFS gateways and broader ecosystem tooling (Filecoin, web3 wallets, blockchain indexers). Loses the 10× speed advantage and the iroh-native alignment.
- **Option C — Dual-hash (publish both CIDs).** Content addressed by both BLAKE3 and SHA-256. Double storage cost. Complete ecosystem reach. Adds a verification step per access path.
- **Option D — BLAKE3 internal + SHA-256 translation at boundary.** Internal paths use BLAKE3; when content is published to a SHA-256-expecting network (e.g., public IPFS gateways), a SHA-256 CID is computed over the same canonical bytes. Preserves speed + iroh alignment internally; adds Phase-2+ complexity at the publish boundary.

**Why Option A for Phase 1-3:** Benten's deployment model is peer-to-peer meshes (Atriums, Gardens, Groves) synced via iroh, not public IPFS gateways. Content stays within Benten-speaking peer networks. The speed + iroh-alignment of BLAKE3 dominates; the cost (reduced default-IPFS-gateway verification) doesn't hit our Phase 1-3 deployment model.

**Interop caveats (Phase 1 honest disclosure):**

- Our CIDs ARE valid CIDv1 per the IPLD spec. Any multiformat-aware parser reads the structure: `[0x01 version][0x71 dag-cbor][0x1e BLAKE3][0x20 length][32-byte digest]`.
- A public IPFS gateway (e.g., `ipfs.io/ipfs/bafyr4i...`) can fetch and serve our content by CID; it does NOT need BLAKE3 support for routing/storage.
- Verification of fetched bytes (the content-addressing integrity check) requires BLAKE3 support in the reader. Modern `kubo` (go-ipfs) ships with BLAKE3 support since ~2023. Older gateways or custom builds may route without verifying.
- Pure content integrity inside Benten peer networks — where every peer speaks BLAKE3 — is unaffected.

**Phase-N reconsideration triggers:** If Benten ever commits to "content must be verifiable on default public IPFS gateways without plugin-level BLAKE3 support" as an adoption requirement, revisit with Option C (dual-hash) or Option D (boundary translation). Until then, Option A stands.

**Revisit at v1-window** (per `docs/future/phase-3-backlog.md` §10.3):
- *Question to ask:* Does the v1 deployment posture (full peer + thin compute surfaces) introduce content-addressing surfaces that require post-quantum-floor collision resistance — i.e., do any v1 audit assumptions rest on `2^128` being out of reach for nation-state adversaries with future quantum hardware?
- *What changed pre-v1 vs Phase-1 framing:* the threat model widened from single-machine engine to multi-device Atriums + thin-client adjacents (browsers, edge runtimes); content-addressing identity now feeds DID-bound capability chains + UCAN-by-CID grants whose forgery cost matters for cap-policy integrity.
- *No-action option:* keep BLAKE3-256 as the architectural floor; document the post-quantum reconsideration as a Phase-N+ trigger (parallel to the IPFS-gateway trigger above). The 128-bit bound is sufficient for the personal-AI-assistants threat model + standard-classical-adversary v1 audit.

---

### Compromise #2 — Symmetric-None + diagnostic capability (Option C) — CLOSED

**Status (2026-04-17, 5d-J workstream 1):** migrated from Option A (honest-but-existence-leaking `E_CAP_DENIED_READ`) to **Option C** (symmetric `None` on denial, diagnostic-capability escape hatch). The existence-leak surface the prior posture named is no longer live; the escape hatch gives operators the signal they need without exposing it to ordinary callers.

**Status (2026-05-05, Phase-3 G14-D wave-5a):** D5 SUBSCRIBE per-event read-cap-coverage CLOSED at G14-D. The Phase-2b coarse-boolean cap-recheck shape (consulted only `is_actor_active`) is replaced by a per-event closure constructed via [`benten_engine::cap_recheck::CapRecheckFn`] + the durable UCAN backend at G14-B; a partial revoke that strikes the actor's read coverage observably cancels the affected subscription path mid-stream via the `E_SUBSCRIBE_REVOKED_MID_STREAM` typed error. The dual-layer per CLR-2 / cap-major-2 — subscribe-time gate AND delivery-time per-row gate — is wired via the new `Engine::on_change_with_cap_recheck` entry point in `crates/benten-engine/src/engine_subscribe.rs`. Cross-trust-boundary filtering happens at DELIVERY (registration is open per plan §3 G14-D); the load-bearing reversal of the Phase-2b interim shape. Composes with the G15-A IVM materialization-time per-row read-gate at `crates/benten-engine/src/ivm_view_read_gate.rs` (both consumers share the `cap_recheck.rs` G13-pre-C scaffold per ds-r4r2-7). Every test under `crates/benten-engine/tests/subscribe_cap_recheck.rs` is the regression surface.

**Status (2026-05-09, Phase-3 G16-B-F PR #161):** sec-r4r1-2 BLOCKER — sync-replica WRITE per-write cap-recheck-at-delivery (the symmetric counterpart to D5's SUBSCRIBE-side CLR-2 dual-layer recheck) — CLOSED end-to-end. Pre-G16-B-F, the SUBSCRIBE side carried 3 cap-recheck pins driving the `on_change_with_cap_recheck` delivery-time gate; the symmetric sync-replica WRITE side carried ZERO production-driven cap-recheck. PR #161 wires structural-always-on per-row cap-recheck inside `Engine::apply_atrium_merge`'s post-merge loop (Ben's ratified Option (a) — NO source enum, NO bypass flag, NO caller-picks privileged shortcut; "err on the side of security generally and then open safe QOL paths later"). Every row produced by Loro merge passes through the engine's `CapabilityPolicy::check_write` hook + an in-memory `revoked_actor_zone_pairs` set; recheck fires BEFORE Version Node mint so a single revoked row vetoes the whole merge atomically (CURRENT does not advance). Typed rejection via new `E_SYNC_REVOKED_DURING_SESSION` ErrorCode + `EngineError::SyncRevokedDuringSession { peer_did, zone, cid }` variant; observable via `Engine::sync_replica_cap_recheck_calls()` AtomicU64 counter (cap-r4-3 / r4b-cap-4 reinforcement). Both pins at `crates/benten-engine/tests/sync_replica_attribution.rs::sync_replica_write_cap_recheck_at_delivery_against_local_grant_store` + `sync_replica_write_after_local_grant_revoke_post_handshake_rejected_with_e_sync_revoked_during_session` un-ignored end-to-end. CLR-2 dual-layer recheck now wired symmetrically at both SUBSCRIBE delivery AND sync-replica WRITE delivery — both halves landed pre-tag per phase-3-backlog §6.12 item 6.

**Primary path — symmetric None.** `Engine::get_node`, `Engine::edges_from`, `Engine::edges_to`, and `Engine::read_view` now collapse a `CapabilityPolicy::check_read` denial onto `Ok(None)` / `Ok(vec![])` / an empty-list `Outcome` — byte-identical with the response an unauthorised caller would see if the CID were genuinely absent. An attacker probing the CID space cannot distinguish denial from not-found through any of these surfaces.

**Escape hatch — `Engine::diagnose_read`.** A new public method surfaces the distinction, but is itself gated on a `debug:read` capability: the configured policy's `check_read` is consulted with label `"debug"` and the target CID; a denial there collapses the probe into `Err(CapError::Denied)` so ordinary callers see the same `E_CAP_DENIED` shape that every other capability denial wears. When permitted, the method returns:

```rust
pub struct DiagnosticInfo {
    pub cid: Cid,
    pub exists_in_backend: bool,
    pub denied_by_policy: Option<String>,  // `"store:<label>:read"` on denial
    pub not_found: bool,
}
```

Three distinguishable states:

- `existsInBackend: false, notFound: true, deniedByPolicy: null` — never written (or deleted).
- `existsInBackend: true, deniedByPolicy: Some("store:<label>:read")` — exists, reader lacks the scope.
- `existsInBackend: true, deniedByPolicy: None` — exists and is readable by this caller.

**TypeScript surface:** `engine.diagnoseRead(cid)` returns `{ cid, existsInBackend, deniedByPolicy, notFound }`. The `CrudOptions.debugRead` flag on `crud('post', { debugRead: true })` is an informational hint for tooling that the handler's operator expects to hold the diagnostic grant; the real gate is `engine.grantCapability({ actor, scope: "store:debug:read" })`.

**Posture claim:**

- The public read API does NOT surface an existence signal to unauthorised callers under any input.
- The diagnostic signal IS available, but is itself capability-gated — an attacker who lacks the grant sees `E_CAP_DENIED` (not `E_NOT_FOUND`, not `null`).
- NoAuth deployments (no policy configured) treat `diagnose_read` as open; this matches the embedded / single-user trust model where the caller already has full backend access.

**What this posture does NOT claim:**

- **Change-stream parity.** `Engine::subscribe_change_events` still fans out every committed ChangeEvent without a per-event `check_read` gate — see the separate "Change-stream subscription bypasses capability read-checks" section below. The Option-C gate covers the four read surfaces named above; the subscribe path stays as-is for Phase 1 because the Engine instance itself is the security boundary.
- **Evaluator-path gating of READ primitives inside a user subgraph.** Option C gates the engine-orchestrator public API. The evaluator's `PrimitiveHost::check_read_capability` hook is now wired (5d-J workstream 1 added the trait method with a permissive default); Phase-2 threads it into the READ primitive's execute path so `crud:post:get` dispatched through `Engine::call` honours Option C end-to-end without a separate gate at the public API.
- **SUBSCRIBE D5 per-event read-cap-coverage.** `Engine::on_change_as_with_cursor` (Phase 2b G6-A / wave-8c) builds the delivery-time cap-recheck closure as `move |_event| -> bool { inner.is_actor_active(&actor_cid) }` inside `crates/benten-engine/src/engine_subscribe.rs::on_change_as_with_cursor`. The closure consults a flat `revoked_actors` set — a coarse boolean per-actor-revoked check — and **does NOT re-evaluate per-event read-cap-coverage** against the event's anchor CID. The eval-side `TestPrincipal::has_read_cap_for` (defined at `crates/benten-eval/src/primitives/subscribe.rs::TestPrincipal::has_read_cap_for`) has the right anchor-CID-keyed shape, but production napi/TS paths inherit the engine wrapper's boolean shape. Consequence: a *partial* revoke (operator removes the specific grant `store:post:read` from an active actor while leaving the actor active) does NOT auto-cancel an in-flight `onChange` subscription. *Full* actor revocation IS honoured. The per-event read-cap-coverage closure lands in Phase 3 alongside the durable grant-store / `benten-id` work; carry-forward destination is `docs/future/phase-2-backlog.md` §7.4 (Durable grant-store + SUBSCRIBE delivery-time cap-recheck). **Composition note (R6FP-G1 multi-label fix).** R6FP-G1 (PR #62) widened the SUBSCRIBE delivery matcher to walk every label of the source Node — a multi-labeled Node `["User","Admin"]` now correctly fires for both `User:*` and `Admin:*` subscribers (the prior single-primary-label behaviour silently dropped multi-label deliveries). Composed with the coarse-boolean cap-recheck above, a `User:*`-pattern subscriber whose actor is still active receives the FULL payload of multi-labeled Nodes including any Admin-tier labels — even if the actor lacks Admin-tier caps. Pre-R6FP-G1 the matcher consulted only the primary label so this widening surface was masked; post-fix the multi-label walk is correct (the prior single-label behaviour was the bug) AND the cap-recheck coarseness becomes more visible. Closes when Phase-3 `phase-2-backlog.md` §7.4 lands per-event read-cap-coverage.

**`E_CAP_DENIED_READ` code:** retained in the catalog (`docs/ERROR-CATALOG.md`) because Phase-2 evaluator-path READ enforcement still needs a typed denial code for the evaluator-visible leg — the Option-C public API mapping is an engine-orchestrator concern, not a catalog removal. The `CapError::DeniedRead` variant remains the signal policies use to communicate "denied" to the engine; the engine maps it onto `Ok(None)` at the public boundary.

**Regression tests:**

- `crates/benten-eval/tests/read_denial.rs` — six Option-C tests covering symmetric-None on `get_node`, `edges_from`, the three `diagnose_read` outcomes (`exists_but_denied`, `not_found`, NoAuth-open), and the `debug:read` gate.
- `crates/benten-engine/tests/integration/compromises_regression.rs::compromise_2_option_c_symmetric_none_plus_diagnose_read` — engine-level regression.
- `crates/benten-eval/tests/read_denial.rs::compromise_2_option_c_is_documented` (the eval-side doc-grep regression that keeps this section load-bearing — asserts the SECURITY-POSTURE Compromise #2 narrative remains in the doc).

**Phase 3 revisit (federation / sync):** sync replicas cross trust boundaries; Phase 3 revisits whether a reader CAN observe existence through a sibling peer (the Phase-3 `CapRevoked` scenario) and may upgrade `diagnose_read` to require a federation-aware principal handle. The Option-C surface introduced here stays stable; Phase 3 layers scope on top.

---

### Compromise #1 — TOCTOU window bound at CALL entry + ITERATE batch boundary

Phase-1 capability checks refresh the grant snapshot at THREE distinct
boundaries: (a) every transaction commit via `CapabilityPolicy::check_write`
in `benten-engine`, (b) CALL primitive entry via
`PrimitiveHost::check_capability`, and (c) ITERATE batch boundaries —
every `host.iterate_batch_boundary()` iterations (default 100), inclusive
of iter 0. A revocation that lands mid-batch is therefore visible to the
evaluator at the NEXT batch boundary; a revocation that lands between
handler registration and CALL entry is visible at the CALL entry.

**Why the batch cadence:** per-iteration policy lookup would impose an
O(N) backend read against the grant table on every step of every
iterate. The batch-refresh amortizes that cost to O(N/100) while keeping
the worst-case TOCTOU window bounded at 99 iterations.

**What this posture does NOT claim:**
- Per-iteration revocation visibility inside a batch. A grant revoked at
  iter 50 will still authorize writes 50..=100; write 101 is the first
  to see the revocation.
- Real-time revocation across a federation (that's the Phase-3
  `CapRevoked` code, distinct from Phase-1's `CapRevokedMidEval`).

**What IS guaranteed:**
- Transaction commits see the current policy state (per-commit).
- CALL entry observes a revocation that landed before the outer
  handler reached the CALL primitive; the denial routes `ON_DENIED`.
- Writes past an ITERATE batch boundary observe any revocation that
  landed within the previous batch; the denial routes `ON_DENIED`.
- The batch-boundary / CALL-entry denials surface the policy's error
  code string (e.g. `E_CAP_REVOKED_MID_EVAL`) in the edge payload so
  operators can distinguish batch-boundary revocation from generic
  `E_CAP_DENIED`.

**Regression tests:**
- `crates/benten-engine/tests/integration/cap_toctou.rs::capability_revocation_at_batch_boundary_surfaces_mid_eval_code`
  — engine-level per-commit refresh.
- `crates/benten-eval/tests/cap_refresh_toctou.rs` — seven tests
  covering CALL-entry refresh (permit + deny), ITERATE entry refresh,
  batch-boundary refresh, no-spurious-refresh on single-batch, and
  host-supplied boundary override.

**Phase-2 revisit:** configurable per-handler batch size (0 =
per-iteration check, at the cost of the O(N) backend read) and
wall-clock bound on the TOCTOU window (auditor finding
[g4-p2-uc-2](../.addl/phase-1/r5-g4-pass2-ucan-capability-auditor.json)
— TRANSFORM-heavy handlers can push the 100-iteration cap past 10
minutes of wall-clock time).

**R6 R2 FP-B (2026-05-25) — wall-clock half WIRED.** The
`CapabilityPolicy::wallclock_refresh_ceiling()` trait method (default
300s per §9.13) is now consumed by `primitive_host.rs::check_capability`
at every batch boundary — previously the method existed but had ZERO
production callers, leaving the wall-clock TOCTOU half UNCLOSED in
shipped binaries. Post-FP-B a revocation-sensitive backend can tighten
the bound observably: a policy returning a 60-second ceiling forces a
refresh every 60s of monotonic elapsed regardless of iteration count.
Pinned at
`crates/benten-engine/tests/tf_compromise_1_wallclock_refresh_fires.rs`
(grep-based source pin asserts the consumer call site exists; trait-
surface arms pin the policy method's reachability). The remaining
`schedule_revocation_at_iteration` API on GrantReader + populated
`iterate_write_handler` fixture (the iteration-count half) stays
deferred. The deferred integration tests
`capability_revoked_mid_iteration_denies_subsequent_batches` and
`writes_in_current_batch_are_not_retroactively_denied` in
`crates/benten-caps/tests/toctou_iteration.rs` remain `#[ignore]`
pending the Phase-2 `schedule_revocation_at_iteration` API on
GrantReader + a populated `iterate_write_handler` fixture.

---

### Compromise #3 — `ErrorCode` enum lives in `benten-core` — CLOSED

Originally open: the canonical catalog enum `ErrorCode` lived in `benten-core`
instead of a dedicated `benten-errors` crate, which forced every workspace crate
that only needed the stable string identifiers to carry a `benten-core`
dependency edge.

**Closure (2026-04-17).** `ErrorCode` (plus `as_str` / `as_static_str` /
`from_str`) extracted to a new [`benten-errors`](../crates/benten-errors/src/lib.rs)
root crate with zero workspace dependencies. Every workspace crate now
depends directly on `benten-errors` for the catalog; `benten-core` keeps its
own `CoreError::code()` mapping but is no longer the source of truth for the
enum itself. The drift-detector (`scripts/drift-detect.ts`) reads the enum
from its new home; the codegen script's comment is updated to match.

**Posture claim (unchanged):** the `ErrorCode` string forms (`"E_CAP_DENIED"`,
`"E_INV_CYCLE"`, …) remain **frozen**. Drift between this enum and
`docs/ERROR-CATALOG.md` is detected by the drift lint in CI. Adding a
variant requires (a) the enum entry, (b) a catalog doc entry, (c) the
`.code()` mapping in the owning crate.

**Regression test:** `compromise_3_error_code_enum_in_benten_errors` in
`crates/benten-engine/tests/integration/compromises_regression.rs` pins
the type path via `std::any::type_name` (the assertion now requires
`benten_errors::` — any accidental re-introduction of an `ErrorCode` back
in `benten_core` fails the test). A second pin lives in
`crates/benten-errors/tests/stable_shape.rs` which counts variants and
round-trips the catalog-code strings through `as_str` / `from_str`.

---

### Compromise #4 — WASM runtime is compile-check only — CLOSED

**Closure provenance:** Phase 2b waves G7 + 8b + 8h (SANDBOX wire-through). The Phase-2b R4b post-impl audit surfaced that the prior G7-A scaffold left the production dispatch gate returning `PrimitiveNotImplemented` and the executor body returning an empty `SandboxResult` without ever instantiating wasmtime — the closure narrative was aspirational. **Wave-8b** wired the production dispatcher (`crates/benten-eval/src/primitives/mod.rs:96`) to `sandbox::execute(...)` and replaced the executor body with the real `Store + Linker + Instance` lifecycle, fuel/epoch/memory limiters, host-fn trampoline, `CountedSink` PRIMARY+BACKSTOP D17 enforcement, and trap → typed-error mapping with the D21 priority resolver. **Wave-8h** then closed the docs-vs-code audit's three audit-gap drifts (manifest-registry hydration, EMIT broadcast, IVM Algorithm B production registration) so the Named-manifest dispatch path consults the engine's `installed_modules` state.

**Original scope (Phase 1):** the `bindings/napi` crate compiled with `--target wasm32-unknown-unknown` in CI (`wasm-checks.yml`) but did NOT execute a WASM runtime (browser / `wasmtime`) at test time. The Phase-1 WASM surface existed only to guarantee that the napi bindings built for a browser target so Thrum (the Phase-4 consumer) could compile them into its web bundle.

**What now ships at Phase 2b (post-wave-8b/8h):**
- A live `wasmtime` host inside `crates/benten-eval/src/primitives/sandbox.rs` runs guest WebAssembly modules per-call (D17 instance lifecycle), with the four enforcement axes (memory / wallclock / fuel / output) bounded by the defaults documented in `docs/SANDBOX-LIMITS.md`. The production engine path routes through the `impl PrimitiveHost for Engine::execute_sandbox` override at `crates/benten-engine/src/primitive_host.rs` which reads module bytes via `Engine::module_bytes_for(cid)`, hydrates the `ManifestRegistry` from `installed_modules`, builds the `SandboxConfig` from the engine's policy + the operation node's properties, and invokes `benten_eval::sandbox::execute`.
- A capability-derived host-function manifest (`host-functions.toml` at the workspace root, G7-A owned) controls which host-fns each guest may import. Capability resolution happens at instance-init time per call; revocation between calls is honoured. Wave-8h hydrates the registry from the engine's `installed_modules` set so Named-manifest dispatch (e.g. `manifest: "compute-power"`) resolves through the same path that `Engine::install_module` persists state into.
- ESC defense matrix (16 named escape vectors per `pre-r1-security-deliverables.md` §1). The canonical numbering in the inventory + the test corpus at `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs` is authoritative; this matrix uses the same numbering. The 16 vectors split into four buckets post-wave-8b/8h:

  | # | Vector | Defense mechanism | Runtime status | Test pin |
  |---|--------|-------------------|----------------|----------|
  | ESC-1 | OOB linear-memory read | wasmtime bounds-check trap → `trap_to_typed::map_call_error` → `SandboxModuleInvalid` | Fully wired | `sandbox_escape_attempts_denied.rs:76` (`sandbox_escape_oob_linmem_read_traps`) |
  | ESC-2 | Linear-memory grow beyond per-call cap | `SandboxResourceLimiter::memory_growing` returns `Err(MemoryCapExceededMarker)` → marker downcast at `trap_to_typed.rs:120-126` → `SandboxMemoryExhausted` | Fully wired (fixture re-authored wave-8d-narrative; see wave-8 §r6-wsa-4 dead-branch nit) | `sandbox_escape_attempts_denied.rs:85` (`sandbox_escape_linmem_grow_to_limit_kills`) |
  | ESC-3 | Host-buffer overrun via host-fn output write | `kv:read` trampoline bounds-check inside `crates/benten-eval/src/primitives/sandbox.rs::register_default_host_fns` → `Trap::MemoryOutOfBounds` → `SandboxModuleInvalid` | Fully wired | `sandbox_escape_attempts_denied.rs::sandbox_escape_host_buf_overrun_rejected` |
  | ESC-4 | Infinite loop without fuel | `Store::set_fuel(config.fuel)` → `Trap::OutOfFuel` → `SandboxFuelExhausted` | Fully wired | `sandbox_escape_attempts_denied.rs:147` (`sandbox_escape_infinite_loop_fuel_bound`) |
  | ESC-5 | Recursive-call stack overflow | wasmtime `Config::max_wasm_stack(512 KiB)` → `Trap::StackOverflow` → **Phase-3 G17-A1 wave-5b: dedicated `SandboxStackOverflow` typed variant** (formerly catalog-folded into `SandboxModuleInvalid`; r6-wsa-8 BELONGS-NAMED-NOW deferral retired). Maps to `E_SANDBOX_STACK_OVERFLOW` per phase-3-backlog §6.4 + r1-wsa-7 BLOCKER closure. Cascade through `crates/benten-eval/src/sandbox/trap_to_typed.rs::map_call_error` + napi error-mapping at `bindings/napi/src/error.rs::engine_err`. | Fully wired (dedicated typed variant landed at G17-A1) | `sandbox_escape_attempts_denied.rs:170` (`sandbox_escape_recursive_call_overflow_traps`) + `sandbox_stack_overflow.rs::sandbox_stack_overflow_routes_to_e_sandbox_stack_overflow_typed_variant` |
  | ESC-6 | Fuel-counter overflow regression | wasmtime saturated fuel bookkeeping; per-call `set_fuel` budget independent of guest run-time → `SandboxFuelExhausted` | Fully wired | `sandbox_escape_attempts_denied.rs:199` (`sandbox_escape_fuel_overflow_regression_held`) |
  | ESC-7 | Fuel-refill via host-fn re-entry | **Phase-3 wave-5c: Fully wired end-to-end.** Per-call `Store` lifecycle (D3-RESOLVED no-pool) + `SandboxStoreData.esc_defense_state: EscDefenseState` carries `re_entry_count` + `guest_active` flag (set by `enter_guest`/`exit_guest` immediately around `func.call` in `crates/benten-eval/src/primitives/sandbox.rs::execute_with_live_cap_check`). The host-fn boundary `run_all_checks` invocation surfaces `EscapeAttemptMarker` which `map_call_error` unwraps to `SandboxError::EscapeAttempt(Esc7FuelRefillViaReEntry)`. Routes through dedicated `E_SANDBOX_ESCAPE_ATTEMPT` catalog code per phase-3-backlog §6.1 + §6.1-followup task #3 + r1-wsa-1 BLOCKER closure + D-E (R1-revision triage). | Fully wired (end-to-end pin against `Sandbox::execute` driven through `wasmtime::Module` + `Instance::call`) | `sandbox_esc_runtime_arms_e2e.rs::esc_7_runtime_arm_fires_via_time_host_fn_re_entry_injection` (end-to-end) + `sandbox_esc_7.rs::esc_7_fuel_refill_via_host_fn_re_entry_blocked` + `..._traps_typed_error` (SHAPE pins; superseded by the e2e pin) — green at wave-5c |
  | ESC-8 | Call host-fn not in manifest | `Linker::func_wrap` only registers manifest-allowlisted host-fns; missing import → wasmtime "unknown import" → `SandboxHostFnNotFound` | Fully wired | `sandbox_escape_attempts_denied.rs:247` (`sandbox_escape_host_fn_not_on_manifest`) |
  | ESC-9 | Cap-revoke mid-call (TOCTOU between cap-grant and cap-use) | **Phase-3 wave-5c: Fully wired end-to-end.** D18 `PerCall` live-recheck via `LiveCapCheck` callback (`Arc<dyn Fn(&str) -> bool + Send + Sync>`) consulted from the trampoline `cap_check` helper BEFORE EVERY host-fn invocation per r1-wsa-3 MAJOR (no caching window). The engine override at `crates/benten-engine/src/primitive_host.rs::execute_sandbox` constructs the callable as a closure capturing `Arc<Mutex<HashSet<Cid>>>` cloned from the engine's revoked-actors set + the dispatching actor CID; mid-call revocation flips the actor's revoke bit and the next host-fn invocation surfaces `SandboxError::HostFnDenied`. Cadence is once-per-host-fn-entry (cadence (a) per r1-wsa-3 disposition + r4-r1-wsa-4 — within a single host-fn call, the recheck does NOT re-fire per loop iteration). Closes phase-3-backlog §6.3 + §6.1-followup task #5. | Fully wired (end-to-end pin against `Sandbox::execute` driving `kv_read` twice with mid-call revoke) | `sandbox_esc_runtime_arms_e2e.rs::esc_9_runtime_arm_fires_via_live_cap_check_revoke_mid_call` (end-to-end) + `sandbox_capability_check_per_call_after_revoke.rs::sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent` + `sandbox_esc_9.rs::esc_9_live_cap_check_fires_at_every_host_fn_boundary_no_caching_window` + `..._within_kv_read_loop_consults_once_per_call_not_per_iteration` — green at wave-5c |
  | ESC-10 | Re-entrancy via host-fn (cap-context confusion via SANDBOX → CALL → SANDBOX) | `AttributionFrame.sandbox_depth` runtime threading bumps depth at SANDBOX entry (see `crates/benten-engine/src/primitive_host.rs::execute_sandbox` saturating-bump on the parent `ActiveCall`); `SandboxError::NestedDispatchDepthExceeded` fires above ceiling at `crates/benten-eval/src/primitives/sandbox.rs::execute` → `E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED`. Wired via R6FP-G1 (PR #62) 3-lens convergent fix | Wired-defense / simulation-driven pin — defense fires; the adversarial pin is LIVE (un-ignored at G20-A1 wave-8a, widened at G21-T3) but SIMULATION-driven: it drives the `testing_call_engine_dispatch` helper's `EscDefenseState` transition rather than a real host-fn re-entry, because no production host-fn re-enters `Engine::call` (D19-RESOLVED) | `sandbox_escape_attempts_denied.rs::sandbox_escape_reentrancy_via_host_fn_denied` — LIVE `#[test]`, asserts `EscapeAttempt(Esc7FuelRefillViaReEntry)` |
  | ESC-11 | Component-Model type mismatch | wasmtime component-model linker type-check → `SandboxModuleInvalid`. wasmtime workspace dep at `Cargo.toml:321` ships `["runtime", "cranelift", "std", "async"]` — explicitly NO `component-model` feature; defense IS the cut | Component-model gated (`#[cfg(feature = "component-model")]` + `#[ignore]`) | `sandbox_escape_attempts_denied.rs:313` (`sandbox_escape_component_type_mismatch_rejected`) — feature-gated |
  | ESC-12 | Resource handle forgery | wasmtime component-model resource-handle table validates handles → `SandboxModuleInvalid` or `SandboxHostFnDenied` | Component-model gated (same cut as ESC-11) | `sandbox_escape_attempts_denied.rs:330` (`sandbox_escape_resource_handle_forgery_rejected`) — feature-gated |
  | ESC-13 | Trap during fuel-meter callback / Store-state corruption | **Phase-3 wave-5c: Fully wired end-to-end.** A `std::panic::catch_unwind` wrapper around `func.call` in `crates/benten-eval/src/primitives/sandbox.rs::execute_with_live_cap_check` catches host-side panics (fuel-meter callback OR any panicking host-fn closure); the wrapper sets `esc_defense_state.fuel_meter_callback_trapped = true` + surfaces `SandboxError::EscapeAttempt(Esc13StorePoison)` directly (no wasmtime trap unwinds through host frames). Pairs with D3-RESOLVED per-call `Store` lifecycle: the (potentially-poisoned) `Store` is dropped on return; the next SANDBOX call gets a fresh `Store` (poison-recovery pin: `esc_13_recovery_path_next_call_fresh_store_no_poison_leak`). Closes r1-wsa-1 BLOCKER half-b + §6.1-followup task #4 + D-E. | Fully wired (end-to-end pin against `Sandbox::execute` + recovery-path pin proving the next call is uncontaminated) | `sandbox_esc_runtime_arms_e2e.rs::esc_13_runtime_arm_fires_via_panic_in_host_fn_callback` (end-to-end) + `..._recovery_path_next_call_fresh_store_no_poison_leak` (recovery) + `sandbox_esc_13.rs::esc_13_trap_during_fuel_meter_callback_store_poison_observable` (SHAPE pin) — green at wave-5c |
  | ESC-14 | Cap-claim forge in module bytes | Engine ignores embedded WASM custom sections for cap purposes; cap derivation is exclusively from the manifest passed at call time. `forged_cap_claim_section.wat` (committed) verifies that a forged section is silently ignored AND that subsequent `kv:read` calls fire `SandboxHostFnDenied` if the manifest didn't include them. D26 `.wasm`-bytes shipping for the escape corpus is a wave-8 noted gap (r6-wsa-5) | Partial / eval-side smoke (forged-section helper carry-forward; manifest-authoritative defense IS structurally correct in production code) | `sandbox_escape_attempts_denied.rs:378` (`sandbox_escape_forged_cap_claim_section_ignored`) |
  | ESC-15 | Named-manifest spoofing (typo / non-existent name) | `manifest_ref.resolve(&registry)` returns `Unknown`; no permissive fallback → `SandboxManifestUnknown` | Fully wired | `sandbox_escape_attempts_denied.rs:403` (`sandbox_escape_named_manifest_spoofing_rejected`) |
  | ESC-16 | Wall-clock leak via `time` host-fn fingerprinting | **Phase-3 wave-5c: Fully wired end-to-end.** `time` host-fn returns monotonic-coarsened values (100 ms granularity) AND the trampoline calls `crates/benten-eval/src/sandbox/fingerprint.rs::record_wallclock_write` on each invocation (populating the per-call `SandboxStoreData.tainted_addresses` side-table) + `read_collapse_state` (incrementing `esc_defense_state.fingerprint_correlated_reads` for tainted-cell hits). At the host-fn boundary `run_all_checks` fires `SandboxError::EscapeAttempt(Esc16FingerprintCollapse)` once the read counter reaches `FINGERPRINT_COLLAPSE_THRESHOLD` (3 reads-within-one-call) BEFORE the side-channel becomes guest-observable. Closes r1-wsa-4 + phase-3-backlog §6.1 + §6.1-followup task #2. | Fully wired (end-to-end pin: 3-call WAT fixture trips the threshold; below-threshold pin proves the defense is silent on legitimate use) | `sandbox_esc_runtime_arms_e2e.rs::esc_16_runtime_arm_fires_after_threshold_time_host_fn_calls` (end-to-end) + `esc_16_silent_below_threshold_two_time_calls_pass` (below-threshold) + `sandbox_escape_attempts_denied.rs::sandbox_escape_wallclock_fingerprint_via_time_coarsened` (1000-call loop) + `sandbox_host_fn_time.rs::sandbox_host_fn_time_returns_monotonic_coarsened_100ms` (host-fn-level coarsening) — green at wave-5c |

  **Bucket totals (16 vectors, each in exactly one bucket; updated at Phase-3 wave-5c close):** **Fully wired (12):** ESC-1, -2, -3, -4, -5, -6, -7, -8, -9, -13, -15, -16 — production runtime defense + end-to-end integration test passing. *(Note: ESC-5 routes through the dedicated `SandboxStackOverflow` typed variant + `E_SANDBOX_STACK_OVERFLOW` catalog code per phase-3-backlog §6.4 + r1-wsa-7. ESC-7 / ESC-9 / ESC-13 / ESC-16 promoted from "Wired-defense + simulation pin green (helper SURFACE)" at Phase-3 G17-A1 wave-5b → **Fully wired** at wave-5c via the production runtime arms wired in `crates/benten-eval/src/primitives/sandbox.rs::execute_with_live_cap_check` + the engine override in `crates/benten-engine/src/primitive_host.rs::execute_sandbox`; end-to-end pins drive `Sandbox::execute` and assert observable typed-error firing per pim-2 §3.6b in `tests/sandbox_esc_runtime_arms_e2e.rs`.)* **Partial (1):** ESC-14 (production manifest-authoritative defense structurally correct — embedded WASM custom sections silently ignored for cap purposes; the integration pins are LIVE — `sandbox_escape_forged_cap_claim_section_ignored` plus the G21-T3 helper-driven `sandbox_escape_forged_cap_claim_section_helper_driven`, after G17-A1 shipped the `testing_inject_forged_cap_claim_section` SURFACE and G20-A1 filled the body — but the coverage stays STRUCTURAL, an absence-of-custom-section-parsing assertion, not a forged-section end-to-end execution, so the bucket stays Partial). **Wired-defense / live-but-simulation-driven adversarial pin (1):** ESC-10 — `AttributionFrame.sandbox_depth` runtime threading wired in `crates/benten-engine/src/primitive_host.rs::execute_sandbox` (R6FP-G1 / PR #62); the eval-side runtime arm in `crates/benten-eval/src/primitives/sandbox.rs::execute` fires `SandboxError::NestedDispatchDepthExceeded` once `attribution.sandbox_depth > config.max_nest_depth`; the adversarial integration test is LIVE (`#[test]`, un-ignored at G20-A1 wave-8a after G17-A1 shipped the `testing_call_engine_dispatch` SURFACE, widened at G21-T3) but remains SIMULATION-driven — it drives the helper's `EscDefenseState` transition and asserts the typed `EscapeAttempt(Esc7FuelRefillViaReEntry)` reject, rather than a real host-fn re-entry (no production host-fn re-enters `Engine::call`, D19-RESOLVED), which is why the bucket stays distinct from "Fully wired". **Component-model gated (2):** ESC-11, -12 — `#[cfg(feature = "component-model")]` + `#[ignore]`; the wasmtime workspace dep at `Cargo.toml:321` explicitly omits the feature. **Total:** 12 + 1 + 1 + 2 = 16 (no double-counting). The honest headline: **13 of 16 vectors fire typed-error defense end-to-end against the production executor at Phase-3 wave-5c close** (12 with full integration tests + ESC-10 with runtime defense and a LIVE but simulation-driven adversarial pin; ESC-14 covered structurally by live pins). The remaining 2 (ESC-11, -12) are component-model feature-cut. Wave-5c closes r1-wsa-1 BLOCKER (ESC-7 + ESC-13 end-to-end) + r1-wsa-3 MAJOR (ESC-9 cap-revoke mid-call cadence + production override) + r1-wsa-4 MAJOR (ESC-16 fingerprint-collapse). Wave-5b's r1-wsa-7 BLOCKER (ESC-5 stack-overflow catalog) remains closed.
- Cross-platform behaviour:
  - **Native targets (Linux x86_64, macOS arm64, Windows x86_64):** SANDBOX executes guest modules. Per-call cold-start budget gated by `bench_thresholds.toml` per the D22 RESOLVED tiered numerics (see `docs/SANDBOX-LIMITS.md` §6).
  - **wasm32-unknown-unknown / wasm32-wasip1:** the SANDBOX executor is compile-time absent (`#[cfg(not(target_arch = "wasm32"))]`). The DSL surface (`subgraph(...).sandbox(...)`) stays present so authoring works in browsers; invocation surfaces the typed error `E_SANDBOX_UNAVAILABLE_ON_WASM` at execution time, with the wsa-14 actionable text directing operators to either Phase-3 P2P sync against a Node-resident peer or local-development via @benten/engine in a Node.js process.

**Regression tests pinning the closure:**
- `crates/benten-engine/tests/integration/sandbox_compile_time_disabled_on_wasm32.rs` — pins both halves of the compile-time gate (executor present on native, surfaces typed error on wasm32).
- `bindings/napi/test/sandbox_napi_bridge.test.ts` — pins the napi bridge's cfg-gated symbol set + the `sandboxTargetSupported()` introspection probe.
- `packages/engine/test/wasm_browser_target.test.ts` — pins the browser-target UX (DSL stays present, registration succeeds, invocation fails with the typed error).
- `packages/engine/test/sandbox.test.ts` — pins the DSL composition surface (no top-level `engine.sandbox(...)`; `SandboxArgsByName` vs `SandboxArgsByCaps` discriminated union) and the D24 default-knobs surfacing through `engine.describeSandboxNode(...)`.
- `crates/benten-engine/tests/security_posture_md_phase_2b_compromises_documented.rs::security_posture_compromise_4_marked_closed` — asserts THIS section header carries `— CLOSED`.

**Posture claim now in force:** the SANDBOX runtime is a load-bearing primitive. It is expected to run in Phase 2b deployments. The four enforcement axes and the capability-derived host-fn manifest constitute the supply-chain and runtime-isolation perimeter for untrusted-code execution; operators who require additional defence-in-depth (process-level isolation, separate `wasmtime::Engine` per tenant) layer those on top of — not in place of — the in-engine bounds.

**Inv-4 runtime threading — fully wired at R6FP-G1 (PR #62).** Both Inv-4 enforcement arms are now active at Phase 2b close. (1) **Registration arm:** `invariants::sandbox_depth::validate_registration` at `structural.rs:215, 387` walks the static-graph at registration time (was already wired pre-wave-8). (2) **Runtime arm:** `crates/benten-engine/src/primitive_host.rs::execute_sandbox` mutates the parent `ActiveCall.sandbox_depth` via `frame.sandbox_depth = frame.sandbox_depth.saturating_add(1)` on every production SANDBOX entry; the dispatching `AttributionFrame` is constructed with `sandbox_depth: nested_depth` in both match arms of the same function. Subsequent CALL pushes inherit the bumped depth via the dispatcher-inheritance read in `crates/benten-engine/src/engine.rs::dispatch_call_with_mode_and_trace` (`let parent_sandbox_depth = guard.last().map_or(0, |f| f.sandbox_depth)` immediately before the new `ActiveCall` push). The eval-side runtime arm in `crates/benten-eval/src/primitives/sandbox.rs::execute` fires `SandboxError::NestedDispatchDepthExceeded` when `attribution.sandbox_depth > config.max_nest_depth` (default `max_nest_depth = 4` admits depths 1..=4, depth 5 fires) — surfaces as `E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED` through `trap_to_typed`. Residual (retensed): the ESC-10 adversarial integration test `sandbox_escape_attempts_denied.rs::sandbox_escape_reentrancy_via_host_fn_denied` is LIVE (`#[test]`, un-ignored at G20-A1 wave-8a — the §7.3.A.7 destination in [`docs/future/phase-3-backlog.md`](future/phase-3-backlog.md) is CLOSED) and drives the `testing_call_engine_dispatch` helper end-to-end into the typed `EscapeAttempt` reject. It stays SIMULATION-driven rather than a true nested-dispatch driver, because no production host-fn re-enters `Engine::call` (D19-RESOLVED).

**Posture claim — per-call-only instance lifecycle is a security win by construction (D3-RESOLVED, sec-pre-r1-12).** Phase 2b ships the SANDBOX executor with a per-call `wasmtime::Instance` lifecycle (D17-RESOLVED) and explicitly NO opt-in instance pool (D3-RESOLVED). This is not solely a DX or perf decision — it is a security posture claim. With per-call instantiation:

- **No cross-call wasm-linear-memory leakage by construction.** A pooled instance shared across two SANDBOX calls would surface a hazard whenever a guest module wrote to a wasm-global without realising the global persisted; the per-call lifecycle removes that hazard by forcing every call to start from a freshly-instantiated module.
- **No cross-call capability-resolution leakage.** Host-fn closures resolve capability grants at instance-init time; per-call init means a capability revoked between two calls is honoured on the second call without per-checkout revalidation logic that a pool would require.
- **Cross-tenant isolation reduces to call ordering.** Two tenants sharing the same engine still get distinct instances per call — the absence of pooling means there is no shared instance whose internal state could be probed across a tenant boundary.

The cold-start cost is bounded by the D22 tiered numerics (≤2 ms p95 / ≤5 ms p99 Linux x86_64; ≤5 ms p95 / ≤10 ms p99 macOS arm64 + Windows x86_64 — see `docs/SANDBOX-LIMITS.md` §6); a measured breach is the trigger that would re-open D3 with real-workload data, NOT an arbitrary regression. If pooling ever lands in a future phase the security claim above must be re-stated at that point — Phase 2b's posture rests on the per-call lifecycle.

**Non-regression notes — Phase-2a security closures stay closed (sec-pre-r1-13).** Phase 2b MUST NOT re-open the Phase-2a security closures listed below; the closures continue to hold under the SANDBOX-bearing surface. Each closure has its own regression test that fires red on regression.

- **sec-r6r1-01 — Inv-14 attribution wired through every primitive frame.** The AttributionFrame routing path is the carrier for the D20 Inv-4 nested-dispatch depth counter (`AttributionFrame.sandbox_depth: u8` field, G7-B owned). Engine-side SANDBOX plumbing (`engine_sandbox.rs`, this G7-C file) routes through the evaluator's `PrimitiveHost` dispatch — there is no SANDBOX execution path that bypasses AttributionFrame propagation. Pinned by the existing Phase-2a Inv-14 regression suite plus the Phase-2b `tests/engine_sandbox_end_to_end_via_dsl_composition_only` integration test (the SANDBOX call inherits its parent attribution frame).
- **sec-r6r2-02 — `test-helpers` cfg-gating on the engine surface.** The `Engine::describe_sandbox_node(...)` accessor (G7-C; ts-r4-3 finding) is `#[cfg(any(test, feature = "test-helpers"))]`-gated so the napi cdylib (which opts into the narrower `envelope-cache-test-grade` feature only) does NOT compile this surface into production. The G12-C subgraph type relocation MUST preserve every existing `testing_*` surface gate; G7-C's new accessor adds to the gated set rather than relaxing any prior gate.
- **sec-r6r3-02 — parse-counter cfg-gate.** The G12-A BudgetExhausted runtime emission wiring (Phase-2b R5 wave-1) routes through the AttributionFrame path and does NOT bypass the parse-counter cfg-gate. The G7-C SANDBOX gate is independent of the parse-counter gate; both must remain.

**Wave-8h adjacencies — EMIT broadcast + IVM Algorithm B production registration.** The wave-8h audit-gap fixes addressed two surfaces orthogonal to the SANDBOX wasmtime-invocation axis but adjacent to Compromise #4's overall "Phase-2 primitives are wired end-to-end" posture:

- **EMIT engine wrapper** at `crates/benten-engine/src/primitive_host.rs::emit_event` was previously a documented no-op (`Phase-1 EMIT is a no-op at the host level`); a handler with a standalone EMIT primitive (no backing WRITE) silently dropped the payload. Wave-8h wired EMIT to publish through a dedicated `EmitBroadcast` channel (separate from the `ChangeBroadcast` channel which carries storage WRITE events). Public API: `Engine::subscribe_emit_events`. The two channels are intentionally separate — extending `benten_graph::ChangeEvent` with an emit variant would conflate two distinct event shapes (storage commits vs handler-emitted events) that currently serve different downstream consumers (IVM views + cap-recheck pipelines vs ad-hoc observers + log shippers). Phase-3 may converge them when iroh sync introduces a unified P2P event stream; until then the two channels stay distinct and `EmitBroadcast` lives in `crates/benten-engine/src/emit_broadcast.rs`.
- **IVM Algorithm B production registration** at `crates/benten-engine/src/engine_views.rs` was previously a `ContentListingView` fallback for every `Strategy::B`-declared user view (the view ran on the Phase-1 ContentListingView shape regardless of the spec's declared Strategy::B). Wave-8h wired the dispatch to construct `AlgorithmBView::for_id(spec.id())` for the 5 canonical view IDs that `AlgorithmBView` supports natively. **Phase-3 G15-A + G15-B + R5 wave-9 W9-T1 closure:** the non-canonical-view fallback is RETIRED — `Algorithm::register` (and the budget-aware sibling `Algorithm::register_with_budget`) instantiates a generic single-loop kernel (`GenericKernel`) for non-canonical view ids keyed on `(label_pattern, projection)`. The Phase-2b coverage compromise no longer applies: non-canonical view IDs no longer silently coerce to `ContentListingView`. The drift-detector proptest harness (`crates/benten-ivm/tests/algorithm_b_drift_detector.rs`, 5 pins × 1 000 cases each) is the verification surface for "incremental updates equal from-scratch rebuild" parity across the merged kernel. Closure tracking + cross-references in [`docs/future/phase-3-backlog.md` §5.1](../docs/future/phase-3-backlog.md).

**Cross-refs.** `docs/SANDBOX-LIMITS.md` (axes + per-platform cold-start); `docs/HOST-FUNCTIONS.md` (host-fn surface).

---

### Compromise #5 — No write rate-limits; metric recorded only

Phase 1 does not enforce a write-rate limit on the engine's ingress. A misbehaving or adversarial caller can submit arbitrary writes as fast as the capability policy permits.

**What IS recorded:** `engine.metrics_snapshot()` surfaces four write counters (both Rust + napi surfaces):

- `benten.writes.committed` — aggregate count of transactions the capability policy permitted (tick once per committed batch, not per op).
- `benten.writes.denied` — aggregate count of transactions the capability policy rejected.
- `benten.writes.committed.<scope>` — per-capability-scope fan-out. The scope key is `store:<label>:write`, derived from the batch's `PendingOp` labels (mirrors `GrantBackedPolicy`'s internal derivation so the counters line up with the enforcement-side key space).
- `benten.writes.denied.<scope>` — per-capability-scope denial fan-out.

Typed accessors `Engine::capability_writes_committed() -> BTreeMap<String, u64>` and `Engine::capability_writes_denied() -> BTreeMap<String, u64>` expose the per-scope maps directly without the flattened key projection. napi callers get the same shape via `engine.capabilityWritesCommitted()` / `engine.capabilityWritesDenied()`. Operators can detect abnormal rates per scope out-of-band.

The counters increment regardless of whether a capability policy is plumbed in — under the zero-config `NoAuthBackend` default a batch that writes `post`-labelled Nodes still bumps `benten.writes.committed.store:post:write`. System-zone writes (`system:*` labels) are intentionally excluded from the per-scope tally because user subgraphs cannot reach them and crediting privileged grant/revoke paths to the per-scope key would make the metric misleading.

**Why no enforcement:** a proper rate-limit needs a **scoped** budget (per-actor, per-capability, per-handler) — not a global one. Phase-1 lacks the actor-identity machinery (Phase-3 `benten-id`) to make the scoped variant meaningful; a global rate-limit would punish legitimate bulk-import workflows more than it protects against abuse. Recording the per-scope counter now means Phase-3 can layer enforcement on top without re-deriving the scope key space.

**What this posture does NOT claim:**
- Protection against DoS via write-flood at the engine ingress.
- A ceiling on backend write throughput.
- Bounded memory for the per-scope map. Scope keys derive from user-supplied Node labels, so an adversarial writer who creates Nodes with (say) 10k distinct fresh labels will grow the map to 10k entries. Phase-3's rate-limit pass adds eviction; Phase-1 accepts the unbounded-growth surface because (a) realistic label cardinality is bounded, and (b) the attack class is identical to spamming distinct CIDs in the backend, which the operator already has to manage.

**Phase-2 / Phase-3 revisit:** Phase-2 introduces a policy-layer budget trait so `CapabilityPolicy` implementations can enforce per-actor rate limits. Phase-3 ties the identity shape (actor Cid) to the budget so the rate is scoped correctly across a federation, and adds eviction for the per-scope counter map.

**Revisit at v1-window** (per `docs/future/phase-3-backlog.md` §10.2):
- *Question to ask:* Does the personal-AI-assistants threat model produce write-flood attack surfaces that can reach the engine ingress through the cap-policy permit gate? If yes: does a per-actor rate-limit (now that `benten-id` ships actor-Cid identity) close the flood vector without breaking legitimate bulk-import / federation-replay workflows?
- *What changed pre-v1 vs Phase-1 framing:* `benten-id` actor-Cid identity now exists (Phase-3 G14-A); the per-scope counter map persists across Phase-3 (still no eviction). Eviction + per-actor budget threads through the same `CapabilityPolicy` gate that Phase-3 hardened end-to-end.
- *LOC estimate at v1-window:* ~150-300 (per-actor budget enum on `CapabilityPolicy` + eviction policy on the per-scope counter map + integration tests for budget-exhaustion `ON_DENIED` routing).

**Regression tests:**
- `writes_committed_metric_is_recorded` + `per_capability_write_metrics_increment` + `denied_writes_surface_on_denied_metric` in `crates/benten-engine/tests/metrics.rs` pin the Rust recording shape.
- `compromise_5_no_write_rate_limits_but_metric_recorded` in `crates/benten-engine/tests/integration/compromises_regression.rs` pins the "no rate-limit enforcement" half.
- `metricsSnapshot surfaces per-capability write counters` in `bindings/napi/index.test.ts` pins the TS round-trip.

---

### Compromise #7 — `[[bin]]` `required-features` gating — CLOSED

Originally open: `benten-graph`'s `write-canonical-and-exit` test-fixture
bin was declared with `test = false` / `bench = false` but no
`required-features` gate, so `cargo install benten-graph` compiled an
unnecessary test-fixture binary alongside the library crate.

**Closure (2026-04-17).** `crates/benten-graph/Cargo.toml` now declares a
`test-fixtures` feature (default-enabled) and gates the bin with
`required-features = ["test-fixtures"]`. Downstream consumers doing
`cargo install benten-graph --no-default-features` skip the bin entirely;
the workspace-wide `cargo test` / `cargo nextest run --workspace` path
keeps it via the default feature so `d2_cross_process_graph.rs` still
resolves `CARGO_BIN_EXE_write-canonical-and-exit`.

**Regression test:**
`compromise_7_benten_graph_bin_is_required_features_gated` in
`crates/benten-engine/tests/integration/compromises_regression.rs` reads
`benten-graph/Cargo.toml` and asserts (a) the `required-features`
clause, (b) the `test-fixtures` feature declaration, (c) the default
membership. Removing any of the three flips the test red and re-opens
this compromise.

---

### Compromise #8 — `Engine::call` bypasses the evaluator for CRUD handlers — CLOSED

Originally open: during G7 the `Engine::call` dispatch for
`register_crud`-registered handlers took a "CRUD fast-path" that
synthesised a transaction directly against the backend and skipped the
`benten-eval` evaluator walk entirely. The fast-path mirrored the
capability hook and change-event emission of the full dispatch, but it
was a parallel code path — any invariant check or primitive-level hook
added to the evaluator would not fire for CRUD handlers.

**Closure (R5 pass-5b).** The `PrimitiveHost` trait was extracted and
`benten-engine` now implements it; `Engine::call` drives
`Evaluator::run_with_trace` for every registered handler (CRUD and
SubgraphSpec alike) and replays buffered host-side WRITE / DELETE ops
atomically inside a single transaction after the walk completes. The
CRUD fast-path is retired; there is no dispatch path that reaches the
backend without walking the evaluator first.

**Why it matters (security framing):** the bypass was a latent
backdoor that would have let a Phase-2 invariant (e.g. invariant 8
cumulative iteration budget, invariant 11 system-zone reachability)
ship green against SubgraphSpec handlers while silently not firing for
the zero-config `crud('<label>')` registration that most applications
use. Closing the compromise eliminates that backdoor before the
Phase-2 invariant set lands.

**Regression test:**
`compromise_8_primitive_host_is_sole_dispatch` in
`crates/benten-engine/tests/integration/compromises_regression.rs`
pins the "evaluator is sole dispatch path" contract. Re-opening the
CRUD fast-path flips the test red.

---

## `requires` property is Phase-1 advisory (r6-sec-1)

Handler subgraphs can declare a `requires` property on each primitive
(e.g. `write.requires("store:post:write")`). In Phase 1 this property is
**declarative-only**: the engine does NOT use the declared string to gate
the operation at evaluation time. What IS enforced is the **derived
per-op scope**: `GrantBackedPolicy` re-derives `store:<label>:write` (or
`store:<label>:read`) from the actual `PendingOp` the transaction
commits, and requires an unrevoked capability grant for that scope. The
attack class where a handler declares `requires: "post:read"` but writes
to an `admin`-labelled Node is therefore already closed — the policy
sees `store:admin:write` in the PendingOp batch, finds no grant, and
denies.

What Phase 1 does NOT close:

- **Declared-vs-actual mismatch surfacing.** A handler that declares
  `requires: "post:read"` but actually writes admin data registers and
  runs; the write is denied at commit, but the registration itself gives
  no warning. Operator tooling + the mermaid diagram DO show the
  declared string, so a human reviewing the registered handler sees the
  lie.
- **CALL-attenuation via `requires`.** The `isolated: false` call path
  that would attenuate the caller's capability context to the
  intersection of the outer grant and the callee's declared `requires`
  is Phase-2 scope (named compromise contract, R1 triage SC4). The
  Phase-1 posture: every CALL runs under the outer actor's grants; a
  compromised callee that issues a wider write sees the same per-op
  derived-scope check as any other handler.

The pair of tests at `crates/benten-eval/tests/requires_enforcement.rs`
remain `#[ignore]`-gated on the Phase-2 register-time static analysis
pass that would elevate declared-vs-actual to a registration-time
error (`E_REQUIRES_SCOPE_MISMATCH`). The test pair proves the Phase-2
closure once the static analyzer lands; the Phase-1 defensive line is
the GrantBackedPolicy derived-scope check exercised by
`crates/benten-caps/tests/grant_backed_policy.rs`.

---

## Change-stream subscription bypasses capability read-checks

**Phase-1 posture.** `Engine::subscribe_change_events` returns a
`ChangeProbe` that drains every committed `ChangeEvent` the engine has observed — including events for Nodes the subscriber does not hold a
`store:<label>:read` capability for. No `check_read` is applied on the
subscriber path. This is a deliberate Phase-1 simplification, not a bug:

- **The Engine instance is itself the security boundary.** Phase 1 ships
  the embedded / single-process trust model (the engine lives in the
  caller's address space; there is no daemon).
  Every caller of `subscribe_change_events` is already trusted with full
  read access to the backing store — they could open the `redb` file
  directly and observe the same data. Gating the subscribe surface would
  give false assurance without closing the real exfiltration path.
- **Existence-leak parity with Compromise #2.** The same "denied reads
  reveal the CID exists" surface that Compromise #2 documents for
  `check_read` already applies to the change stream: a subscriber can
  enumerate committed CIDs regardless of whether a read capability is
  granted. The two surfaces are intentionally co-located because the
  Phase-3 fix is the same: scoped subscriptions over a trust boundary.
- **Attribution is preserved.** Every `ChangeEvent` carries the
  `actor_cid` / `handler_cid` / `capability_grant_cid` triple (r6-sec-3),
  so a Phase-3 policy layer can retroactively filter by observer identity
  without breaking the wire format.

  **Phase-1 field status.** `capability_grant_cid` is present in the
  wire format but is always `None` in Phase 1 — grant-resolution on the
  write path is Phase-3 `benten-id` scope. The field is frozen now so
  audit consumers written today (forward-compatibility). Phase-1 audit
  code MUST NOT rely on the value being populated; consuming code that
  needs grant attribution should wait on the Phase-3 identity surface.

**Phase-3 revisit.** Alongside Compromise #2 — once `benten-id` lands a
typed principal and sync / federation cross the trust boundary, the
engine will:

1. Accept a principal handle at `subscribe_change_events` time.
2. Apply `CapabilityPolicy::check_read` per event before yielding it.
3. Decide between Option A (surface `E_CAP_DENIED_READ` — consistent with
   the read path) or Option B (silent drop — matches the "indistinguishable
   from not-found" posture).

Operators who need a tighter bound today can:
- Deploy with `.without_ivm()` + avoid calling
  `subscribe_change_events` — no probe, no disclosure.
- Run the engine behind a process boundary and gate the subscribe RPC at
  the mux layer.

---

## napi input-limit enforcement (r6-sec-7)

The TypeScript→Rust boundary is the engine's hottest surface and the
primary DoS vector for a hosted deployment. Two classes of input-size
attack are live in Phase 1:

1. **Oversized JSON strings.** A caller who supplies a single
   multi-gigabyte `Value::Text` can force the Rust side to allocate the
   full string before any downstream check fires. The JSON boundary in
   `bindings/napi/src/node.rs` now rejects any string longer than
   `JSON_MAX_BYTES` (1 MiB) with `E_INPUT_LIMIT` before the `Value::Text`
   lands in the tree.
2. **Aggregate payload size.** A JSON tree whose total text-byte weight
   exceeds the per-request budget is similarly rejected with
   `E_INPUT_LIMIT` — the check runs during tree-walk so deeply-nested
   payloads cannot evade the cap by fragmenting across many small values.

**CLOSED 2026-07-29 (R6 round #1, B8).** The DAG-CBOR side of this
boundary is real. `bindings/napi/src/input_limits.rs` runs a bounded
pre-scan over the raw wire bytes — map keys, list items, byte-string
length, text length, nesting depth, declared-length amplification, and an
aggregate item cap — and only then hands the payload to the canonical
decoder. `testing::deserialize_value_from_js_like` is a thin delegation to
it, so the harness exercises the real checker rather than a parallel copy;
the 5 R3 contract tests went 0-pass/5-fail → pass, and 12 boundary pins
were added alongside them (17 total, on the required
`napi in-process pins (rlib mode)` lane).

Two things this closure does NOT claim, stated so the record does not
overstate the binary:

- **The prescribed remedy is not what landed.** The `CoreError::InputLimit`
  variant named above was never added — `benten-core` is frozen. The code
  lives in the non-frozen napi crate behind a napi-local error carrier
  (`NapiInputError`), which discriminates a limit rejection
  (`E_INPUT_LIMIT`) from malformed framing (`E_SERIALIZE`). No frozen
  crate's public API was touched.
- **The DAG-CBOR checker does not fire in the shipped cdylib.** It compiles
  under `napi-export`, but its only callers are `testing::deserialize_*`
  (`lib.rs:2444`, `:2453`) inside `#[cfg(any(test, feature =
  "in-process-test"))] mod testing`, and `default = ["napi-export"]` does
  not enable that feature. **Compiling is not firing.** B8 closed the
  Phase-1 R3 contract exactly as that contract was written — against
  `benten_napi::testing::*` — which is a genuine closure of a genuine gap,
  and is a different claim from production coverage. Wiring a production
  entry point for the DAG-CBOR path is a named residual. What DOES fire in
  production today is the JSON path below.

- **The enforced depth is 64 everywhere, and now cannot drift.**
  `NAPI_MAX_DEPTH` is DERIVED from `benten_core::MAX_VALUE_DECODE_DEPTH`
  (byte-pinned at 64), because a napi-side cap of 128 could never fire on
  the CBOR path — the canonical decoder refuses first — i.e. it would have
  been a defense that looks present and is not.
  **`JSON_MAX_DEPTH` in `bindings/napi/src/node.rs` was `128` and is now
  derived from the same constant.** That mismatch was not cosmetic: the
  JSON path is the LIVE production write path, so a property bag nested
  65..=128 deep was accepted, hashed and persisted — and then could not be
  decoded on read. Silent write-side data loss, on a shipped surface,
  because the boundary admitted a value it could never hand back. Both caps
  now derive from the decoder's own bound, so they cannot diverge again.

---

## `ExecutionStateEnvelope::envelope_cid` does not cover `schema_version` (Phase 2a G3-A / G3-A-mini-review Minor-2)

In Phase 2a `ExecutionStateEnvelope::envelope_cid` returns `payload_cid`
— the BLAKE3 over the DAG-CBOR bytes of `ExecutionStatePayload`. The
envelope's `schema_version: u8` byte is **not** covered by this CID.

**Implication.** An attacker who re-wraps the same payload under a
future `schema_version = 2` produces an envelope whose `envelope_cid`
is byte-identical to the `schema_version = 1` form. Today this is
purely hypothetical — `schema_version = 1` is the only valid value
and the resume path rejects mismatches — but if Phase 2b/3 grows the
envelope shape additively, the re-wrap attack becomes reachable
unless the envelope hash includes the full envelope (not just the
payload).

**Mitigation path (Phase 2b / Phase 3).** Either (a) redefine
`envelope_cid` to hash the full envelope bytes (including
`schema_version`), or (b) ship a separate `envelope_hash` field
alongside `payload_cid` so callers can ask the right question.
Option (a) would change the CID contract and requires coordination
with any already-persisted `ExecutionStateEnvelope` in storage;
Option (b) is additive and preferred.

**Phase 2a status.** Phase 2a pins `schema_version = 1`; the single
call-site that checks re-wrap tampering (`resume_from_bytes` re-
computes `payload_cid` and asserts equality) fires correctly for
Phase 2a's closed shape. Forward-compat concern only.

**Cross-refs.** §9.1 of `.addl/phase-2a/00-implementation-plan.md`
(envelope shape frozen); G3-A mini-review Minor-2 (captured 2026-
04-22).

---

*Future compromises with security implications will be appended as sections here, each tagged with the compromise number from the R1 Triage Addendum.*

## Phase 2a — Inv-13 immutability firing matrix (5-row)

Phase 2a G5-A adopts the firing matrix decided at R1 close (plan §9.11)
for Invariant 13 (immutability). Five rows cover the three
`WriteAuthority` variants plus a resume-time pre-check:

| # | WriteAuthority / Path | Content matches registered bytes | Outcome |
|---|---|---|---|
| 1 | `User` | yes | `E_INV_IMMUTABILITY` — canonical unprivileged immutability violation. Users cannot observe dedup on system-controlled surfaces. |
| 2 | `User` | no | `E_INV_IMMUTABILITY` — vacuous under content-addressing (CID-match ⇔ bytes-match); reached only via the test-only `put_node_at_cid_for_test` backdoor that injects bytes at a caller-supplied CID. Error naming kept for forward-compat with mutable-id extensions. |
| 3 | `EnginePrivileged` (version-chain append) | yes | `Ok(cid_dedup)` — pure-read dedup. Does NOT emit `ChangeEvent` and does NOT advance the audit sequence (Compromise "Dedup writes pure-read" below). |
| 4 | `SyncReplica { origin_peer }` (Phase-3 sync-receive) | yes | `Ok(cid_dedup)` — same no-event + no-audit semantics as row 3. Shape reserved in 2a; wired at Phase 3 receive-path. |
| 5 | WAIT-resume stale-pin pre-check (any authority) | (`pinned_subgraph_cids` no longer matches the anchor's CURRENT) | `E_RESUME_SUBGRAPH_DRIFT` fires BEFORE any write. Distinct code; mirrors arch-1 resume-step-3 (§9.1) in the Inv-13 matrix explicitly. |

`WriteContext` carries a `WriteAuthority` enum
(`User | EnginePrivileged | SyncReplica { origin_peer }`).
`EnginePrivileged` replaces the Phase-1 `privileged: bool` and is set
by the engine orchestrator for capability-grant-authorised
version-chain `NEXT_VERSION` appends; user subgraphs never reach it.
`SyncReplica` reserves a Phase-3 shape for replicated writes.

### Compromise #9 — Dedup writes pure-read (sec-r1-4 / atk-3) — CLOSED at Phase 2b G12-E

**Status (2026-04-27).** **CLOSED at Phase 2b G12-E.** The R5 wave-6 G12-E
landing replaces every process-local suspend / resume state surface
(`OnceLock<Mutex<HashMap>>` in `wait::registry`,
`LazyLock<Mutex<BTreeMap>>` in `engine_wait::ENVELOPE_CACHE`, the
SUBSCRIBE persistent-cursor in-memory placeholder) with a single
durable [`benten_eval::SuspensionStore`] port (default impl:
[`benten_engine::RedbSuspensionStore`] over the engine's existing
`Arc<RedbBackend>`). The dedup-row-3 audit-sequence pure-read
contract (this compromise's body) was already enforced at the
storage layer; G12-E removes the residual cross-process surface
where a privileged re-put racing against an envelope-cache lookup
could observe inconsistent state, completing the closure narrative
per plan §3.2 + R2 §6 row. The dedup invariant tests
(`crates/benten-graph/tests/inv_13_dedup_*`) continue to pin the
storage-layer guarantee unchanged; the new G12-E tests
(`crates/benten-engine/tests/g12_e_suspension_store_round_trips.rs`)
pin the cross-process persistence layer the dedup contract sits on
top of.



**Class.** Audit-log forgery and audit-sequence side-channel leak
via the dedup path.

**Shape.** Row 3 (`EnginePrivileged` + content matches) returns
`Ok(cid_dedup)` as a successful idempotent dedup. If the
transaction machinery still pushed the dedup into `pending_ops` and
fanned out a `ChangeEvent`, an attacker with privileged-write reach
(version-chain append, grant re-issuance) could manufacture a
succession of audit events carrying fresh timestamps but bit-
identical content — inflating the audit trail and making
re-issuance look like distinct authorisations.

A companion side-channel: if the dedup path silently advanced the
audit sequence counter, an observer who can read the sequence
learns "a privileged actor re-visited this CID" even when the
visible audit log is empty.

**Mitigation.** Row 3 (and the Phase-3 row 4) **branch before**
`pending_ops.push`, before any ChangeEvent construction, and before
any audit-sequence advance. The privileged re-put with matching
bytes returns the existing CID with no observable effect on the
ChangeEvent stream or the audit counter. Tests:

- `crates/benten-graph/tests/inv_13_dedup_does_not_emit_changeevent.rs`
- `crates/benten-graph/tests/inv_13_dedup_path_does_not_advance_audit_sequence.rs`
  (engine-side accessor `testing_audit_sequence` lands in G11-2a)
- `crates/benten-graph/tests/inv_13_matrix.rs` (Row 3 no-event)

**SUBSCRIBE persistent-cursor retention bookkeeping — Phase-2b is
process-local; durable retention is a Phase-3 lift.** The G12-E port
makes suspended-WAIT envelopes durable across process restart. The
SUBSCRIBE persistent-cursor retention window (1000-events / 24h) is
enforced via the `SuspensionStore::is_retention_exhausted` trait method
on the in-memory test impl, but `RedbSuspensionStore` (the production
impl backing cross-process re-subscribe) does NOT override the trait
method — the default `false` means the in-memory `delivered_count` +
`registered_at` counters are reset on each process boot. Consequence: a
cross-process re-subscribe past the 1000-events / 24h window does NOT
surface `E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED` today. Operationally
bounded: process restarts during the retention window are rare, and
the in-process retention enforcement is unchanged. The durable
retention bookkeeping is a Phase-3 lift paired with the durable
grant-store + per-event read-cap-coverage work — see
[`docs/future/phase-3-backlog.md` §6.5 RedbSuspensionStore retention-window override](future/phase-3-backlog.md).

**Residual risk.** The dedup pure-read contract is enforced at the
storage-layer entry points (`RedbBackend::put_node_with_context`).
A future code-path that accumulates its own ChangeEvent before
calling into the storage layer would need to re-check the row-3
branch at its own entry point; the `WriteContext::authority` enum
makes this explicit at the type level so reviewers catch
regressions.

**Residual risk (G5-A major — concurrent-writer TOCTOU).** Row 3
takes the dedup branch AFTER the `WriteContext::authority ==
EnginePrivileged` check against the in-memory `registered_cids`
set. Under concurrent writers, writer A can observe `cid ∉
registered_cids`, start the insert path, and race against writer B
who registers the same CID in between. The storage layer
serialises the two redb transactions so both end with the same
on-disk bytes, but the second writer's per-transaction audit-log
fan-out may already have emitted a ChangeEvent before the dedup
check re-runs under the transaction lock. The window is narrow
(two writers must race on the SAME CID, which under content-
addressing means the same bytes — dedup is the correct outcome
either way) and the emitted ChangeEvent carries legitimate
attribution, but the audit trail will show one extra event for the
racing writer. Tightening the window to bit-exact "exactly one
ChangeEvent per CID" is Phase-3 scope where the sync-replica row
needs the same invariant across peers anyway.

**Cross-refs.** plan §9.11 5-row matrix; R1 triage `sec-r1-4`,
atk-3, Code-as-graph Major #4; ERROR-CATALOG E_INV_IMMUTABILITY.

---

## Dual-layer read-capability explanation (ucca-10)

Phase 2a closes the gap the pre-R1 ucca-10 review opened: what
exactly gates a read, and at what layer?

The answer has two layers that enforce different parts of the
contract and that must both be present for the full posture to
hold.

### Layer 1 — Sync-receive gate (Phase 3)

At the network boundary, incoming replicated reads (Phase-3
`benten-sync` receive path) consult `CapabilityPolicy::check_read`
before the bytes are handed to the evaluator. This is the gate
that keeps a peer from force-feeding the engine Nodes its operator
did not authorise to observe — the federation / atrium boundary.
Phase 2a reserves the shape (see `SyncReplica { origin_peer }`
variant on `WriteContext::authority`); the wire is Phase-3 scope.

### Layer 2 — Evaluator-dispatch gate (Phase 2a Option C)

Inside the evaluator, every READ primitive routed through a
registered user subgraph calls
`PrimitiveHost::check_read_capability` before resolving the CID.
Denial collapses to `Ok(None)` — byte-identical with a miss — per
Compromise #2 Option C. This is the layer that keeps a user
subgraph, running under a partial grant, from using TRANSFORM-
computed CIDs to probe the existence of Nodes the caller cannot
read.

**Why both layers are necessary.** A sync-receive gate alone lets
a local compromised-but-unprivileged actor probe the store via
evaluator dispatch (the trust boundary is INSIDE the engine, not
at its edge). An evaluator gate alone lets a malicious peer
force-replicate Nodes that the operator had declined to grant
read — the data arrives and sits in the backend even though no
local caller can observe it. Both gates together pin both trust
boundaries.

**Phase 2a status.** Layer 2 is live (G4-A Option C flanking, sec-
r1-5 IVM views at coarse-grained per-view read; sec-r1-5
fine-grained per-row is Phase 3). Layer 1 is shape-reserved; the
wire lands with `benten-sync` in Phase 3.

**Cross-refs.** Compromise #2 (symmetric-None) closure; Compromise
"IVM views coarse-grained read-gate" below; plan §G4-A Option C
flanking; ucca-10 pre-R1 review.

---

### Compromise #10 — Resume-time capability re-verification (G3-A / G5-B-i Decision 4) — CLOSED at Phase 2b G12-E (cross-process metadata arm) and CLOSED at Phase 3 G14-D wave-5a (engine-side asymmetry arm)

**Status (2026-04-27).** The cross-process metadata arm of this
compromise is **CLOSED at Phase 2b G12-E**. The orchestrator
state log + brief refer to this closure as "Compromise #9" by
sequencing in the open-compromise tracker; the canonical doc
reference is #10.

**Status (2026-05-05, Phase-3 G14-D wave-5a).** The engine-side
asymmetry between WAIT-suspend and WAIT-resume is now **CLOSED**.
G14-D wires `cap_snapshot_hash` derivation
([`benten_engine::cap_snapshot_hash::compute(actor_cid, proof_chain_cids)`])
+ persisted-policy-metadata into a new [`benten_eval::CapSnapshot`]
side-table on the `SuspensionStore` (keyed by envelope CID). The
`resume_from_bytes_*` family recomputes the hash against the
chain currently in the durable cap store and rejects with
`E_CAP_SNAPSHOT_HASH_MISMATCH` when the chain materially changed
(e.g. one UCAN was revoked between suspend and resume) per
CLR-2 §11. A historical-policy metadata blob is preserved across
the suspend/resume boundary so the resumed continuation runs
against the policy in effect at suspend (rate-limit budgets,
attenuation depth). Regression surface lives at
`crates/benten-engine/tests/wait_resume_cross_process.rs` +
`crates/benten-engine/tests/wait_resume_policy.rs` +
`crates/benten-engine/tests/ucan_replay_audience.rs`.

**Class.** Stale-authority resume + cross-process metadata gap.

**Shape.** The WAIT resume protocol re-verifies the caller's
capability at step 4 of the 4-step protocol (`resume_from_bytes` →
envelope integrity check → actor pin check → subgraph-pin check →
**capability re-check** against the current policy state).

This is the intended defence against a grant revoked between
suspend and resume: even if the suspended envelope names a valid
actor CID, the re-check catches a policy that has since removed
the underlying grant.

**Residual gap — cross-process persisted metadata (CLOSED at G12-E).**
Phase-2a parked WAIT suspend metadata (deadline + signal-shape) and
the persisted `ExecutionStateEnvelope` bytes in two ad-hoc
process-local surfaces — `OnceLock<Mutex<HashMap<Cid, WaitMetadata>>>`
in `benten-eval/src/primitives/wait.rs` and
`LazyLock<Mutex<BTreeMap<Cid, ExecutionStateEnvelope>>>` in
`benten-engine/src/engine_wait.rs` (the test-grade `ENVELOPE_CACHE`
gated behind `envelope-cache-test-grade`). Either surface dropped
its state on `Engine::drop`, so a resume in a fresh process either
silently completed the WAIT (the eval-layer permissive
`Complete(value)` fallback) or failed with `E_NOT_FOUND` on
`suspend_to_bytes` (the engine-layer cache miss). Phase 2b G12-E
collapses both surfaces into a single durable port:
[`benten_eval::SuspensionStore`] with the redb-backed default impl
[`benten_engine::RedbSuspensionStore`] over the engine's existing
`Arc<RedbBackend>`. WAIT metadata, envelope bytes, AND SUBSCRIBE
persistent cursors all round-trip through this store; the
`OnceLock` registry + `ENVELOPE_CACHE` static + the
`envelope-cache-test-grade` feature are retired. The Phase-2a
permissive `resume_with_meta` fallback is rewritten to fail loud
with `E_HOST_BACKEND_UNAVAILABLE` so a missing metadata entry
post-G12-E surfaces a typed error rather than silently admitting
an attacker-supplied payload.

**Honest disclosure — fail-closed asymmetry between eval-side and engine-side resume surfaces.** The fail-closed semantics applies to the eval-side `benten_eval::resume_with_meta` API (callers with stricter integrity expectations — the public API used by tests and any future direct-eval consumers). The engine-side bytes-only resume surfaces (`Engine::resume_from_bytes`, `resume_from_bytes_as`, `resume_from_bytes_unauthenticated`) treat a missing `WaitMetadata` entry as best-effort *skip the inline-deadline check and proceed* rather than fail-closed — see the deliberate inline comment inside `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner` (the metadata-store lookup arm) for the rationale. The shape tolerates legitimate cross-process eviction without breaking resume (a fabricated test envelope, a non-WAIT resume reaching this surface, or a store miss after legitimate eviction in cross-process scenarios all skip the deadline-check rather than failing). The downstream Step 2 principal-binding + Step 3 capability re-check still run, so an attacker cannot use the asymmetry to bypass the cap re-check — only the inline deadline check is skipped on missing metadata. The narrow attack class (attacker who forges an envelope + shared SuspensionStore eviction window must coincide) is bounded by the cap re-check arm; the divergence is documented here so operators reading Compromise #10 don't assume both surfaces are uniformly fail-closed.

Regression tests:
`crates/benten-engine/tests/g12_e_suspension_store_round_trips.rs`
(`wait_resume_cross_process_metadata_survives_restart`,
`resume_with_meta_fails_closed_when_metadata_missing`,
`subscribe_persistent_cursor_survives_engine_restart`,
`subscribe_max_delivered_seq_round_trips_via_suspension_store`,
`suspension_store_handles_both_wait_and_cursor_keys_without_collision`).

**Capability re-check arm (UNCHANGED — Phase 3 scope).** G12-E
ships the durable persistence the re-check sits on top of; the
re-check itself still consults the resuming process's live
`CapabilityPolicy`. The original Decision-4 federation-aware
`cap_snapshot_hash` (envelope embeds the hash; resume asserts the
fresh policy's snapshot matches) is Phase-3 scope alongside the
`benten-id`-typed-principal work — sec-r1-5 deferral.

**Mitigation (Phase 2a).** For in-process resume, the check is
correct. For cross-process the Phase-2a default policy is to
refuse — `Engine::resume_from_bytes_as` distinguishes actor CIDs
but does not reach across the process boundary on its own; the
operator must explicitly hand the envelope to a new engine
instance and accept that the re-check semantics change. No
silent regression.

**Cross-refs.** Phase-2a resume protocol; the `ExecutionState`
envelope shape; the Phase-2b SuspensionStore landing (closed at Phase
2b).

---

### Compromise #11 — IVM views coarse-grained read-gate (sec-r1-5) — **CLOSED at G15-A** (Phase-3 R5 wave-5a)

**Class.** Over-read through an IVM view under a per-view grant.

**Status.** **CLOSED at G15-A** (Phase-3 R5 wave-5a, 2026-05).
Per-row read-gate at materialization time landed via
[`crates/benten-engine/src/ivm_view_read_gate.rs::IvmViewReadGate`]
which composes label-hint extraction with the
[`crates/benten-engine/src/cap_recheck.rs::CapRecheckFn`] actor-cap-set
check; the engine surfaces this through
[`crates/benten-engine/src/engine_views.rs::Engine::materialize_view_with_gate`].
The closure is end-to-end: G15-A's materialization-time gate composes
with G14-D's delivery-time gate at SUBSCRIBE per `cap-r4-3` —
deny-from-either-layer wins. The closure is pinned by:

- `crates/benten-engine/tests/ivm_read_gate.rs::ivm_view_per_row_read_gate_against_actor_cap_set`
  (LOAD-BEARING #11 closure pin — 100-row 50/50 fixture yields
  exactly 50 rows for an actor with public-only READ caps).
- `crates/benten-engine/tests/ivm_read_gate.rs::ivm_view_read_gate_fires_at_materialization_separately_from_g14_d_delivery_gate`
  (`ivm-major-2` — gate is independent of SUBSCRIBE delivery layer).
- `crates/benten-engine/tests/ivm_read_gate.rs::materialize_view_with_gate_filters_rows_per_actor_cap_set_at_engine_entry_point_e2e`
  (LOAD-BEARING pim-2 §3.6b end-to-end pin — drives the production
  `Engine::materialize_view_with_gate` boundary with mixed-label
  Nodes written through the engine's transaction surface; asserts
  row-level filtering behavior that would FAIL if the gate were
  silently bypassed or if `materialize_view_with_gate` returned an
  empty list unconditionally).

The G15-B drift-detector proptest harness at
`crates/benten-ivm/tests/algorithm_b_drift_detector.rs` is the
companion verification surface for the merged
`benten_ivm::algorithm_b::Algorithm::register` kernel post-G15-B; it
does not pin Compromise #11's G15-A closure (which stands on the
materialization gate alone). The 5 active proptest pins are
`prop_algorithm_b_incremental_equals_rebuild_for_arbitrary_label_pattern`,
`prop_budget_trip_state_propagation_consistent`,
`prop_rebuild_after_stale_returns_view_to_fresh`,
`prop_drift_detector_observes_label_pattern_extension`,
`prop_drift_detector_reports_one_path_errors_other_succeeds`. R5
wave-9 W9-T1 retensed the harness's headline pin description in
`docs/future/phase-3-backlog.md` §5.1 alongside the
`Algorithm::register_with_budget` budget-knob lift.

**Shape (historical).** Phase 2a G4-A Option C threading gated
`Engine::read_view` at the per-view level: a caller who held a
`store:<view>:read` grant for view X could read ALL rows the view
returned. The gate did not differentiate between rows in the view
that come from source Nodes the caller could directly read vs.
source Nodes the caller could not. A view over a mixed-sensitivity
label set could therefore surface row data the caller lacked the
underlying per-Node grant for.

**Mitigation (historical Phase 2a).** Per-view grants were treated
as an explicit operator opt-in — granting `store:<view>:read` was a
conscious "this view is ok to read, whatever its underlying Nodes"
decision. The view-ID registry (user-authored views landed in
Phase 2b under `P2.ivm.user-views`) made it explicit that a view's
scope was defined at registration; operators were instructed not
to grant view-level read to actors they would not grant the union
of the source-label reads to.

**G15-A closure narrative.** Phase 3 G15-A retires the residual
risk by wiring per-row READ gating at materialization time (this
was the `Phase-3` deferred resolution path the original sec-r1-5
review named). The materialization gate consults the actor's
cap-set per row via [`benten_engine::cap_recheck::CapRecheckFn`]
— the same shared scaffold G14-D consumes for delivery-time
recheck per `ds-r4r2-7`. Because both layers compose:

- **Materialization deny wins:** a row whose underlying Node the
  actor cannot READ does NOT enter the materialised view, so the
  delivery layer never sees it.
- **Delivery deny wins:** a row that passed materialization is
  still subject to G14-D's per-event recheck at SUBSCRIBE; a
  partial-revoke mid-stream cancels delivery on rows the
  materialization gate had already admitted.

**Cross-refs.** Compromise #2 Option C; plan §G4-A; sec-r1-5
pre-R1 review; dual-layer read-cap section above; G14-D F6
SUBSCRIBE filtering at delivery boundary.

**Phase-3 G16-B-A canary deepening (2026-05-08).** G16-B-A's canary
landed three structural-surface pins that complete Compromise #11's
device-grain composition story alongside the Phase-3 sync-merge path:

- **Materialization-only structural pin** —
  `crates/benten-engine/tests/ivm_view_subscribe_compose.rs::compromise_11_materialization_deny_wins_over_delivery_admit_at_view_layer`
  asserts the materialization gate's row filtering operates
  independently of any SUBSCRIBE delivery state. The pin stands GREEN
  at canary scope.
- **Mat-deny-wins composition pin** — same file, asserts that a row
  the materialization gate denies is observably absent from the
  SUBSCRIBE delivery surface (mat-deny-wins composition over
  G14-D delivery-time recheck). Stands GREEN at canary scope.
- **Delivery-gate-registration pin** — pins that
  `Engine::materialize_view_with_gate` correctly registers its
  per-view gate hook against the same `CapRecheckFn` scaffold G14-D
  consumes for SUBSCRIBE delivery, so the layered defense is wired
  through one shared dispatch. Stands GREEN at canary scope.

**Phase-3 G16-B-D deepest-pin closure (2026-05-09).** The deepest
end-to-end composition pin —
`compromise_11_both_gates_compose_observable_delivery_end_to_end` —
is GREEN at G16-B-D using Option (a): a new
`Engine::testing_subscribe_observable_change_events` helper that
exposes the eval-side `ChangeEvent` directly to test callbacks
(bypassing the chunk-encoding bridge that the production
`OnChangeCallback` adapter applies). The helper is strictly cfg-gated
under `cfg(any(test, feature = "test-helpers"))` and lives in the
same `engine_subscribe.rs` module as the production
`on_change_with_cap_recheck`, sharing the same eval-side
`DeliveryCapRecheck` bridge — so the test surface does NOT widen the
production surface (echoes `sandbox_helpers_no_widening.rs` discipline;
see `docs/future/phase-3-backlog.md` §10.6 for the harder Cargo-feature-
graph defense-in-depth that may carry to v1-gate).

The pin asserts the load-bearing dual-gate composition shape:

- mat-admit ∩ delivery-admit = {row_A} (admitted by BOTH gates → in view AND delivered)
- mat-deny ∩ delivery-admit = {row_B} (mat-denied → suppressed at view layer; delivery-admitted → delivered to observer; proves delivery layer is independent of mat layer)
- mat-admit ∩ delivery-deny = {row_C} (delivery-denied → suppressed at delivery; mat-admitted → still in view; proves delivery-deny wins over mat-admit)

End-to-end intersection (rows admitted by BOTH layers) = {row_A}, the
load-bearing closure assertion of the dual-gate composition narrative.

The would-fail-if-no-op'd discipline (pim-2 §3.6b) is satisfied
because the per-row CapRecheckFn closures distinguishably differ from
the structural pins above — the test would fail if either gate were
silently no-op'd in a future regression.

**Sync-grain interaction.** Compromise #11's row-level gate composes
with the Phase-3 G16-B `AttributionFrame` extensions
(`peer_did_set` + `device_did` + `sync_hop_depth`, see
`docs/INVARIANT-COVERAGE.md` "Inv-14 Phase-3 G16-B device-grain
extension"): the gate's per-row check is grain-orthogonal — it gates
on label-derived hints and actor cap-set, NOT on the merge frame's
device origin. Per-device read policy is a separate Phase-3 surface
tracked at `docs/future/phase-3-backlog.md` §6.12 item 3
(production-runtime threading of `device_cid` through engine
WriteContext construction sites). Both surfaces share the
`CapRecheckFn` scaffold but resolve at different layers.

**Phase-2b R6 Round-3 surfacing — `read_view_with` view-id-prefix
heuristic — CLOSED at Phase-3 G20-A3 wave-8a.** R6 Round 3's
`r6-r3-ivm-2` finding observed that `Engine::read_view_with`
(`crates/benten-engine/src/engine_views.rs::read_view_with`) derived
its `label_hint` for the `CapabilityPolicy::check_read` gate
exclusively by stripping the `content_listing_<label>` (or
`system:ivm:content_listing_<label>`) prefix from `view_id`. When
`label_hint` was non-empty the cap-policy `check_read` hook fired and
`DeniedRead` collapsed to an empty list. When `label_hint` was empty
— which was the case for ALL view ids that didn't match the
`content_listing_*` prefix, including the 4 canonical hardcoded-label
views (`capability_grants`, `version_current`, `event_dispatch`,
`governance_inheritance`) and ALL user-defined views — the
`check_read` hook was NOT invoked at the view-level read path. The
underlying `Subscriber::read_view` still runs and returns Node CIDs.
A user can register a custom view subscribing to any label (including
`system:` prefixes — `register_user_view` does not validate input
patterns against the system-zone reservation; the system-zone
reservation is a write-side reservation per Inv-7) and read those
Node CIDs via `engine.read_view`, all without `check_read` firing on
the view-level read path. **Bounded by:** Phase 2b ships only
`NoAuthBackend` (default; permits all reads) and `GrantBackedPolicy`
(which only gates capability-grant chain reads, where the per-row
denial behaviour is provided by the lower-level CID reads in
`primitive_host.rs::read_node`). A user-installed custom
`CapabilityPolicy` that intends to gate reads by label patterns
through view-id-derived hints will silently NOT fire on user-defined
view reads. Default-config users see no impact. The Inv-7 write-side
system-zone reservation is intact (users cannot WRITE `system:` Nodes)
— this finding was purely about the read path's check-firing
asymmetry. **Phase-3 G20-A3 wave-8a closure:** `read_view_with` now
extracts the `label_hint` via a registry helper
(`Engine::resolve_read_view_label_hint`) that consults the
`benten_ivm::CanonicalViews` registry-query type
(`CanonicalViews::registry().lookup(id).and_then(|e| e.hardcoded_label())`,
post-G-CORE-4 D1 A2 collapse — the 4 pre-collapse leaked helpers are
now `pub(crate)`) for canonical ids first, then
the engine's `user_view_input_labels` map (populated at
`register_user_view` time) for user-defined views, falling back to
the `content_listing_` prefix-strip only as a final resort for
pre-canonical-registry tests. End-to-end pin lives at
`crates/benten-engine/tests/view_id_label_hint_refactor.rs::view_id_to_label_hint_consults_input_pattern_label_not_string_prefix`
(per dispatch-conventions §3.6b — drives `Engine::read_view_with`
through a deny-reads-on-`post` cap policy and asserts the silent-deny
empty-list path fires). Anti-regression for canonical 5-view path at
`content_listing_views_still_route_through_registry_post_g20a3`.

---

### Compromise #12 — `DurabilityMode::Group` gate 5 — **CLOSED-AT-G13-E** (Phase-3 R5 wave-3)

**Class.** Durability / audit-freshness tradeoff under batch
commits.

**Status.** **CLOSED at G13-E** (Phase-3 R5 wave-3, 2026-05).
`DurabilityMode::default()` flipped from `Immediate` to `Group`
at the engine surface (see
[`crates/benten-graph/src/backend.rs::DurabilityMode`]); the
benchmark CI workflow `.github/workflows/bench.yml` was promoted
from informational to required + grew the APFS-relevant CRUD
fast-path timing benchmarks. The closure is pinned by three
tests:
- [`crates/benten-graph/tests/durability_default.rs::durability_mode_group_default_for_crud_fast_path`]
  (the default-flip itself);
- [`crates/benten-graph/tests/security_posture_compromise_12_marked_closed`]
  (this section's CLOSED marker);
- [`crates/benten-graph/tests/crud_fast_path_apfs_timing_within_target`]
  (informational wall-clock gate; bench is the authoritative perf signal).

**Shape (historical).** The Phase-2a `redb` backend committed under
`Durability::Immediate` — every write-bearing transaction fsynced
the redb journal before `Engine::call` returned. This was the
correct posture for the Phase-1 / 2a trust model (the ChangeEvent
and the persisted bytes both on disk before the caller observed
success), but it dominated the 150-300 µs §14.6 target on macOS
APFS (~4 ms fsync floor; see Phase-1 named compromise at the
bench-layer docs).

`DurabilityMode::Group` — grouped commit across a configurable
window — was considered for Phase 2a but deferred: the gate-5
interaction (how does the ChangeEvent fan-out interact with a
deferred fsync?) needed its own invariant pass. If a grouped
transaction's ChangeEvent reached a subscriber before the fsync
landed, and the process crashed, the subscriber observed an event
for a write that did not exist on disk at restart.

**Closure (Phase-3 G13-E).** Resolution chosen: **(c) leaving
Group as the engine-surface default while the redb backend
collapses Group → `Durability::Immediate` until redb grows
native batched-commit support.** The engine-level posture is the
right surface to declare; backend-specific mapping is a separate
concern. Three load-bearing claims that this closure makes:
1. Non-redb backends (in-RAM thin-client per
   [`crates/benten-graph/src/browser_backend.rs`] when G13-C lands;
   future peer-sync) can implement true grouped fsync without
   changing call sites — the default is already correct for them.
2. When redb itself grows the capability — see redb tracking
   issue history at the bench-layer docs — the on-disk behavior
   improves transparently with no semver break.
3. ChangeEvent gate-5 invariant: capability-grant writes still
   force `Durability::Immediate` at the redb mapping layer (see
   [`crates/benten-graph/tests/capability_grant_writes_immediate.rs`]),
   so the audit-freshness path that motivated the original
   deferral is preserved. The CRUD fast-path is the only surface
   that adopts the Group default.

**Mitigation (Phase-3 + later).** Capability-grant writes pin
`Durability::Immediate` per
[`crates/benten-graph/tests/capability_grant_writes_immediate.rs`];
operators wanting the historic per-commit-fsync posture
explicitly construct via
[`crates/benten-graph/src/redb_backend.rs::RedbBackend::open_or_create_with_durability`]
with `DurabilityMode::Immediate`. The
[`crates/benten-graph/src/redb_backend.rs::warn_if_group_durability_collapsed`]
one-shot warning still fires on benches so the redb-collapse
caveat is operator-visible.

**Residual risk.** Operators running on macOS dev hardware
against the redb backend still see the ~4 ms APFS fsync floor
per commit because redb v4 collapses Group → Immediate. This
is correct / on-posture for the redb backend specifically; it is
no longer a Compromise of the engine-level posture (the engine
surface declares Group as the default; backend-specific
mappings are a separate concern with their own visibility).
The §14.6 target remains aspirational for the redb backend
until upstream redb (or a Benten write-batching layer) lands;
non-redb backends are not constrained by it.

**Cross-refs.** plan §arch-r1-1; plan §3 G13-E; historical ENGINE-SPEC
§14.6 macOS caveat (the ENGINE-SPEC doc was MERGED-into-ARCHITECTURE.md
at Phase-4-Foundation per cs-r1-2 disposition; the macOS-fdatasync content
now lives in `docs/ARCHITECTURE.md` durability section);
[`crates/benten-graph/benches/crud_post_create_dispatch_group_durability.rs`];
[`crates/benten-graph/benches/durability_modes.rs`];
[`docs/future/phase-2-backlog.md`] §9.1 (CLOSED-IN-PHASE-3-G13-E);
`.github/workflows/bench.yml` (promoted to required at G13-E).

---

### Compromise #13 — System-zone reserved-prefix rejection surface (G5-B-i Decision 6 / Minor 3)

**Class.** DX + defence-in-depth on the reserved `system:` prefix.

**Shape.** Phase-2a G5-B-i enforces Inv-11 in two registration-time
walkers:

1. A READ or WRITE operation Node whose `"label"` property is a
   `system:*` literal is rejected — the traditional Inv-11 surface.
2. An operation Node whose node-ID itself starts with `system:*`
   is ALSO rejected — the G5-B-i Decision 6 reserved-prefix DX
   improvement (Minor 3). A user who types a node-ID starting with
   `system:` gets a pointed error ("IDs cannot begin with the
   reserved `system:` prefix") rather than a downstream confusing
   failure when the node's resolved label happens to miss the
   system-zone.

Row 2 is defence-in-depth: the label-only probe already catches
the real violation, but the ID-prefix check surfaces the mistake at
the earliest possible point and removes a whole class of
"inadvertently used `system:` as a namespace" bugs.

**Mitigation / posture claim.** Both checks are live in
`crates/benten-eval/src/invariants/system_zone.rs::validate_registration`
and `crates/benten-engine/src/system_zones.rs::SYSTEM_ZONE_PREFIXES`.
The runtime probe (`resolved_cid_in_system_zone`) still covers the
TRANSFORM-computed-CID case at evaluation time.

**Residual risk.** A handler that builds a node-ID via TRANSFORM
concat (e.g. `"system:" + $input.key`) slips past the
registration-time reserved-prefix walker — the string is only
known at runtime. The runtime label probe catches the eventual
storage-layer impact (symmetric-None collapse at the user surface),
but the handler author may be confused about which invariant fired.
DX polish; no security gap.

**Cross-refs.** plan §G5-B-i Decision 6 / Minor 3; ERROR-CATALOG
`E_INV_SYSTEM_ZONE`; `SYSTEM_ZONE_PREFIXES` at
`crates/benten-engine/src/system_zones.rs`.

**Revisit at v1-window** (per `docs/future/phase-3-backlog.md` §10.4):
- *Question to ask:* Does the v1 DSL surface (TS DSL + napi + future-DX layers) hit the TRANSFORM-computed-CID system-zone-prefix slip-through often enough that the runtime-probe-only path produces confusing operator UX? If yes: does a runtime TRANSFORM-result reserved-prefix probe (raised symmetrically at WRITE-staging, not just registration) close the DX gap?
- *What changed pre-v1 vs Phase-2a framing:* Inv-13 row-4 SPLIT classifier (G16-B sync-receive arm) now enforces system-zone immutability at the merge boundary, so the security gap is closed; remaining work is purely DX-level error-message clarity for the TRANSFORM-concat case.
- *LOC estimate at v1-window:* ~50-100 (runtime-staging reserved-prefix probe + typed `E_INV_SYSTEM_ZONE_RUNTIME_PREFIX` variant + integration test).

---

### Compromise #14 — SANDBOX cold-start cost (no opt-in pool) — Phase-2b additive

**Class.** Performance / DX. Per-call SANDBOX dispatch always pays
the wasmtime `Store` + `Instance` construction cost.

**Shape.** D3-RESOLVED — Phase 2b ships SANDBOX with **per-call**
hosting only. Each SANDBOX primitive call constructs a fresh
`wasmtime::Store` + `wasmtime::Instance` (the `wasmtime::Engine` and
`wasmtime::Module` are singletons cached by content CID for the
process lifetime — Engine is expensive to construct, Module is
hash-cached, so cold-start cost is the per-Store + per-Instance work
only).

**Why per-call only:**

1. **Easier to add than to remove.** A pool-now / remove-later
   transition would be a breaking change; pool-later / no-pool-now
   is additive.
2. **Closes the "trusted boundary" sub-question entirely** — no need
   to define subgraph-annotation vs cap-grant vs engine-builder vs
   manifest-bound semantics for opt-in.
3. **Don't add features for hypothetical future requirements** — no
   data shows cold-start is a real workload problem at Phase 2b close.
4. **Misuse vector is real** — a developer slapping `pool: true`
   without understanding isolation implications is a silent security
   regression; pre-1.0 should not ship that footgun.

**Mitigation / posture claim.** If Phase-3+ workload telemetry
surfaces cold-start as a real bottleneck, an **opt-in pool** lands
as an additive Phase-3+ change without breaking existing handlers.
The G11-2b paper-prototype revalidation
(`docs/PAPER-PROTOTYPE-REVALIDATION.md`) at 16.7% SANDBOX rate gives
no signal that cold-start is a hot path for typical workloads.

**Cross-refs.** `docs/SANDBOX-LIMITS.md` per-call defaults; per-call
scope clarification (Engine + Module shared, Store + Instance
per-call).

---

### Compromise #15 — `register_runtime` reserved with deferred error — Phase-2b additive

**Class.** Surface area gap (intentional); deferred to Phase 8
marketplace.

**Shape.** D2-RESOLVED — Phase 2b's named-manifest registry is
**codegen-emitted** (a static `HashMap<String, CapBundle>` built at
`ManifestRegistry` construction from `host-functions.toml`). The
`register_runtime(name, bundle)` API is RESERVED in 2b — calls return
`E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED` typed-error.

**Why deferred:** dynamic manifest registration is the
marketplace-layer concern (Phase 8). Before the marketplace ships,
nobody is providing 3rd-party WASM modules with associated manifests;
shipping the dynamic registration API now would invite the
runtime-registered cap-bundle to drift from the codegen baseline
without tooling discipline (cap-bundle fingerprinting, lifecycle, GC).

**Mitigation / posture claim.** Phase-2b's manifest set
(`compute-basic`, `compute-with-kv`) covers the in-tree
host-fn surface; the typed deferral surface gives early-adopter
marketplace builders a concrete error to grep on. Phase 8 lifts the
deferral as part of the marketplace launch.

**Cross-refs.** D2-RESOLVED; `docs/HOST-FUNCTIONS.md` "Named
manifests" section; `host-functions.toml` `[manifest.*]` entries;
`E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED` in `docs/ERROR-CATALOG.md`.

---

### Compromise #16 — `random` host-fn deferred to Phase 3 — CLOSED at Phase-3 G17-A2 wave-5b

**Class.** Capability gap (intentional) — CLOSED at Phase-3 G17-A2
wave-5b (`benten-wt-r5-g17-a2`).

**Closure shape.** D-PHASE-3-11 RESOLVED-at-R1 picked **`getrandom`
direct** as the workspace CSPRNG (NOT `rand` ecosystem; NOT a
deterministic seed). G17-A2 wires the `random` host-fn alongside
`time` / `log` / `kv:read` in the codegen-default surface
(`crates/benten-eval/src/sandbox/host_fns.rs::default_host_fns`); the
trampoline at `register_default_host_fns` invokes `getrandom::getrandom`
to fill a guest buffer per call. Cap-string is `host:random:read`
(4-segment shape mirroring `kv:read`); `cap_recheck` is `per_call`.
Per-call entropy budget defaults to **4096 bytes** (per **r1-wsa-8**);
a module manifest may tighten or widen the override via the additive
optional `host_fns.random.budget_bytes_per_call` field on
`ModuleManifest`. Budget overrun fires the typed
`E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED` variant (routed through the
`ON_DENIED` family per the `cap_string_for_routing` rules in
`benten-errors`). The validate-time deferral guard at
`crates/benten-eval/src/primitives/sandbox.rs::execute` (the
sec-g7a-mr-5 `DEFERRED_HOST_FN_RANDOM_CAP_PREFIX` arm) is RETIRED;
`crates/benten-eval/tests/sandbox_host_fn_random_deferred.rs` is
deleted; the new green-phase regression guards are at
`crates/benten-eval/tests/random_host_fn.rs` (4 tests including the
load-bearing source-cite anti-regression
`sandbox_host_fn_random_no_longer_returns_deferred_error`).

**Shape (historical pre-G17-A2 narrative; preserved for audit).**
D1-RESOLVED — Phase 2b's host-fn set shipped `time`, `log`, and
`kv:read`. `random` was **deferred to Phase 3** because the workspace
CSPRNG framework choice had not been made (rand_chacha vs OS-CSPRNG
vs hardware-RDRAND fallback). Shipping `random` before that decision
would have baked in a footgun — a module that depended on weak
randomness then would be a silent security regression on a future
swap. Picking the wrong CSPRNG is a hard-to-reverse decision. A
SANDBOX module that attempted to call a `random` import received
`E_SANDBOX_HOST_FN_NOT_FOUND` with an operator-actionable hint citing
`phase-3-backlog.md §6.10`.

**Why CLOSED at Phase-3 G17-A2:** the workspace CSPRNG decision
landed at R1 (D-PHASE-3-11 RESOLVED). `getrandom` direct is
appropriate because (a) the rest of the workspace already pins it for
`ed25519-dalek` keypair generation, (b) it doesn't bake the engine
into the broader `rand` ecosystem trait shape, (c) it's
deterministic-seed-free by construction (the OS CSPRNG draws — the
class of footgun the deferral was protecting against). Wiring
`random` through a centralised cap-gated trampoline (NOT inline in
each module) preserves the Phase-2b posture that host-fn surface
additions go through the documented capability + audit-frame
discipline.

**Cross-refs.** D-PHASE-3-11 RESOLVED-at-R1; `host-functions.toml`
`[host_fn.random]` entry; `crates/benten-eval/tests/random_host_fn.rs`
green-phase regression guards;
`crates/benten-eval/src/sandbox/host_fns.rs::DEFAULT_RANDOM_BUDGET_BYTES_PER_CALL`
(4096 byte default per r1-wsa-8); `docs/HOST-FUNCTIONS.md` operator
section; `docs/MODULE-MANIFEST.md` per-manifest override field.

---

### Compromise #17 — In-memory module-bytes registry — CLOSED at Phase-3 G14-C wave-4b

**Class.** Persistence gap — CLOSED at Phase-3 G14-C wave-4b.

**Closure shape.** `Engine::register_module_bytes(&cid, &bytes)` now
recomputes `BLAKE3(bytes)` against the caller-supplied CID at the
entry point (D-PHASE-3-12 RESOLVED: typed
`E_MODULE_BYTES_CID_MISMATCH` rejection on mismatch), persists the
bytes via the durable
`crates/benten-graph/src/backends/blob_backend.rs::RedbBlobBackend`
(`system:ModuleBytes` zone Nodes), and mirrors them into the
in-memory hot-path cache. On `Engine::open`,
`Engine::rehydrate_module_bytes_from_zone` (called from
`EngineBuilder::assemble`) walks the zone and rebuilds the in-memory
cache so SANDBOX dispatch resolves without an operator re-call.

**Shape (historical pre-G14-C narrative; preserved for audit).**
`Engine::register_module_bytes(cid, bytes)` — the API
that registers compiled WebAssembly module bytes for SANDBOX
dispatch — used to store into a process-local `BTreeMap<Cid, Vec<u8>>`
guarded by a `Mutex` (the `module_bytes` field on `crates/benten-engine/src/engine.rs::Engine`). The
bytes were NOT persisted to the engine's redb backend; on `Engine::open`
the registry started empty regardless of what was registered against
the prior process.

**Workflow asymmetry with `Engine::install_module`.** This is the
load-bearing operator-facing surprise: `install_module(manifest,
expected_cid)` writes the manifest's canonical-DAG-CBOR bytes into the
`system:ModuleManifest` zone (a privileged Node write that survives
engine restart and is sync-eligible for Phase-3 federation). But the
underlying *wasm bytes* the manifest references (each `modules[i].cid`)
must be re-registered via `register_module_bytes(cid, bytes)` after
every engine open — there is no symmetric durable path for blob bytes
in Phase 2b. A SANDBOX dispatch whose module bytes haven't been
re-registered fires `E_SANDBOX_MODULE_NOT_INSTALLED` at the engine's
lookup step (the wave-8d-types typed variant), distinct from
`E_SANDBOX_MODULE_INVALID` which fires when bytes ARE present but fail
wasmtime structural validation.

**Why deferred to Phase 3 rather than fixed in 2b:**

1. **Blob storage is the load-bearing Phase-3 lift.** The `BlobBackend`
   trait — content-addressed mutable storage with iroh-fetchable shape —
   is on the Phase-3 critical path for P2P sync (Atriums fetch wasm
   modules from peer caches). Building an interim Phase-2b durable
   blob store would be wasted work since Phase-3 replaces it.
2. **Operator footprint is bounded.** Production deployments register
   modules at engine-open time (alongside `register_handler` /
   `install_module` calls); the in-memory shape mirrors how
   `register_subgraph` works (subgraph specs live in process memory and
   are re-registered on restart). Operators who need durable wasm
   storage in 2b can persist `Vec<u8>` against their own backing store
   and call `register_module_bytes` from their bootstrap path.
3. **No security-class hazard.** `register_module_bytes` does not
   consult capability policy (registering wasm bytes does NOT authorise
   any caller to invoke them — authority flows through the SANDBOX
   node's manifest cap-set + dispatching grant, both checked at execute
   time). The in-memory shape doesn't widen the trust boundary; it just
   widens the operator boilerplate at engine-open.

**Posture claim — non-validating API + lazy validation discipline.**
`register_module_bytes` does NOT verify the supplied CID matches
`blake3(bytes)` — content integrity is the caller's responsibility,
mirroring the pattern `Engine::install_module` uses for manifest CIDs.
Validation fires lazily at SANDBOX dispatch time when wasmtime parses
the bytes (`Module::new(&engine, &bytes)` in
`crates/benten-eval/src/sandbox/instance.rs::module_for_bytes`); a
malformed module surfaces as `E_SANDBOX_MODULE_INVALID`. Phase-3's
durable `BlobBackend` may add content-addressing-based validation at
register time (recompute BLAKE3 over the bytes; reject mismatch
upfront).

**Phase-3 G14-C closure** (LANDED). The Phase-3 promotion path the
Phase-2b plan described is the closure that landed: the in-memory
`BTreeMap` is mirrored against a `BlobBackend` impl
(`RedbBlobBackend`) that writes blobs as `system:ModuleBytes` zone
Nodes; `register_module_bytes` recomputes the CID + asserts
content-integrity + delegates to `BlobBackend::put`; `Engine::open`
rehydrates the in-memory active set via
`Engine::rehydrate_module_bytes_from_zone`. The trait surface is
defined at `crates/benten-graph/src/backends/blob_backend_trait.rs`;
the redb-native impl at `crates/benten-graph/src/backends/blob_backend.rs`;
the IndexedDB-browser impl landed in Phase 3 under the engine's
thin-client commitment (browser tabs hold snapshot cache only).

**Cross-refs.** `crates/benten-engine/src/engine.rs::register_module_bytes`
docstring; `crates/benten-engine/src/engine_modules.rs::install_module`
docstring; Compromise #4 closure narrative (execute-time validation
discipline); `docs/ERROR-CATALOG.md` `E_SANDBOX_MODULE_NOT_INSTALLED`
row; `docs/MODULE-MANIFEST.md` install lifecycle;
`crates/benten-engine/tests/module_bytes_cid.rs` (G14-C end-to-end
pin per §3.6b pim-2).

---

### Compromise #18 — In-memory handler-version chain — CLOSED at Phase-3 G14-C wave-4b

**Class.** Persistence gap — CLOSED at Phase-3 G14-C wave-4b
(sibling to Compromise #17; both closed in the same fix-pass).

**Closure shape.** Each `register_subgraph` /
`register_subgraph_replace` invocation persists a
`system:HandlerVersion` zone Node carrying `(handler_id, version_cid,
predecessor_cid?, seq)` per
`crates/benten-engine/src/handler_versions.rs`. The encoding is
additively extensible per arch-r1-4 / D-C — Phase-3 G16-B's
Loro-merge attribution variant slot lands without breaking existing
chain CIDs. On `Engine::open`,
`Engine::rehydrate_handler_version_chains_from_zone` (called from
`EngineBuilder::assemble`) walks the zone, groups by `handler_id`,
sorts by `seq`, and rebuilds the in-memory newest-first `Vec<Cid>`
chain. The full audit history survives engine restart.

**Shape (historical pre-G14-C narrative; preserved for audit).**
`Engine::register_subgraph_replace(spec)` — the wave-8f
hot-replace API — maintained an in-memory `BTreeMap<HandlerId, Vec<Cid>>`
of version-chain heads (newest-first). Each successful replace prepended
the new handler-CID onto the chain; `Engine::handler_version_chain(id)`
exposed the chain for devserver + audit consumers. The chain was
process-local and was NOT written to the redb backend; on `Engine::open`
the chain started empty regardless of how many replace calls happened
in the prior process.

**Why a separate compromise from #17 rather than a single bullet.** The
content is structurally identical (in-memory `BTreeMap` lost on engine
restart, Phase-3 promotion path the same), but the *audit class*
differs:

- Compromise #17 covers the **wasm payload** — bytes that wasmtime
  loads. Lost data: the wasm module's binary content. Operator recovery:
  re-call `register_module_bytes` from bootstrap.
- Compromise #18 covers the **handler hot-replace audit metadata** —
  the temporal sequence of "v1 → v2 → v3" handler swaps that devserver
  + operators rely on to answer "what was v3 of this handler?". Lost
  data: the historical CIDs. Operator recovery: there is no recovery —
  the historical CIDs are gone unless the operator captured them
  out-of-band (e.g. logs, devserver session state).

The user-visible loss surface differs enough that bundling under #17
would obscure the audit-trail-erasure aspect.

**Phase-3 G14-C closure** (LANDED). The promotion path the Phase-2b
plan described is the closure that landed:
`benten_core::version::Anchor` Node + Version-Node chain shape; each
`register_subgraph_replace` call writes a `system:HandlerVersion`
zone Node carrying the handler-CID + per-handler `seq` (insertion
order); `Engine::handler_version_chain_with_anchor` walks the
rebuilt chain and returns a `core::version::Anchor` rooted at the
oldest registered version. Phase-3 sync forwards the chain verbatim
across peers (the Version Nodes are content-addressed). G16-B
Loro-merge attribution lands as an additive variant slot per
arch-r1-4 / D-C without breaking existing chain CIDs.

**Posture claim.** The hot-replace contract itself is unchanged by the
in-memory shape: in-flight `Engine::call` invocations DO NOT see the
swap (handler_cid resolves once at call entry; the spec Mutex
re-lookup at `dispatch_call_inner` uses that CID as the third axis of
the subgraph-cache key). The in-memory shape only erases the
**historical** chain on restart; the **current** chain is durable for
the engine's lifetime, which is the contract devserver hot-reload
relies on.

**Cross-refs.**
`crates/benten-engine/src/engine.rs::register_subgraph_replace`
docstring (in-memory note);
`crates/benten-engine/src/engine.rs::handler_version_chain` docstring;
Compromise #17 (sibling persistence gap); Phase-2b R5 wave-8f mini-review
finding 8f-dx-10; [`docs/future/phase-3-backlog.md` §1.4 (Compromise #17
durable module-bytes registry) + §1.5 (Compromise #18 durable
handler-version chain)](../docs/future/phase-3-backlog.md) — both lift
to durable Anchor + Version-Node chain backed by the GraphBackend
umbrella trait (PHASE-3-BUNDLE-1).

---

### Compromise #19 — Browser-target persistent storage — PARTIALLY CLOSED at Phase-3 G18-A wave-5a

**Status:** **PARTIALLY CLOSED** at Phase 3; **FULL CLOSURE** deferred per `docs/future/phase-3-backlog.md` §4.3 (when the wasm32 `web-sys` / `js-sys` / `wasm-bindgen-futures` plumbing lands). **Partially closed via:** IndexedDB schema + handler scaffolding under the engine's thin-client commitment, plus schema-versioning groundwork.

**What landed at G18-A (scaffolding half).** Two new modules ship the persistence-layer architectural surface on `wasm32-unknown-unknown`:

- `bindings/napi/src/browser_indexeddb.rs` — IndexedDB schema-versioning layer (handler scaffolding). Declares schema-version constant `INDEXEDDB_SCHEMA_VERSION = 1`, the `module_manifest_store` + `blob_cache` object stores, the `on_upgrade_needed` migration handler (walks the v→v+1 chain — chain-computation half is wired; the wasm32 IDB-side dispatch is a stub), the `on_version_change` handler (stub on wasm32; the host build exercises the chain logic), and the `map_dom_exception_to_error_code` helper that maps `DOMException(name="QuotaExceededError")` to the typed [`ErrorCode::StorageQuotaExceeded`] variant (`E_STORAGE_QUOTA_EXCEEDED` per `docs/ERROR-CATALOG.md`).
- `bindings/napi/src/browser_blob_store.rs` — `IndexedDbBlobBackend` handle declaration mirroring the `BlobBackend` trait surface locked at G13-pre-B (`crates/benten-graph/src/backends/blob_backend_trait.rs`). Mirrors the redb-native `RedbBlobBackend`'s defense-in-depth CID validation per D-PHASE-3-12. The `IndexedDbBlobBackend::is_persistent()` returns `false` honestly at G18-A — the native arm uses an in-RAM `BTreeMap` mirror (native consumers must use `RedbBlobBackend`); the wasm32 arm has no IDB plumbing yet.

**What is DEFERRED to G18-A-followup (per `docs/future/phase-3-backlog.md` §4.3).**

- The wasm32 `web-sys` / `js-sys` / `wasm-bindgen-futures` deps that issue real `IDBDatabase.open` / `IDBObjectStore.put` / `IDBObjectStore.get` calls. The wasm32 arms of `apply_migration_step` + `close_database` are stubs today. Until those wire, `BrowserManifestStore::is_persistent()` and `IndexedDbBlobBackend::is_persistent()` BOTH return `false` honestly per the disclosure principle Compromise #19 originally articulated ("honest disclosure protects operators from assuming durability where none exists").
- The `BlobBackend` trait integration through the `Engine::open_with_browser_blob_backend(...)` constructor. The handle ships; the engine wire-up is the follow-up scope.

**Thin-client commitment.** The IndexedDB schema declares ONLY thin-client surfaces (`module_manifest_store` + `blob_cache`) — full-sync state (`loro_doc`, `iroh_peers`, `sync_cursor`, `atrium_full_state`) is explicitly absent and forbidden by the architectural pin at `bindings/napi/tests/indexeddb_schema.rs::indexeddb_persistence_thin_client_cache_only_per_baked_in_17`. Browser tabs participate in sync as authenticated thin-client views into a user's full peer; they do NOT carry sync state of their own.

**OPFS deferral per D-PHASE-3-27 / br-r1-11.** IndexedDB is primary at G18-A (broad browser support); OPFS / File System Access API is deferred to post-Phase-3. Future Phase-4+ may add an `OpfsBlobStore` sibling via the `BlobBackend` trait surface.

**Cross-refs.** `docs/MODULE-MANIFEST.md` §3.2; `docs/ERROR-CATALOG.md::E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE` + `E_STORAGE_QUOTA_EXCEEDED`; D-PHASE-3-27; br-r1-2 BLOCKER scaffolding; br-r1-8 MINOR honest-disclosure principle; `docs/future/phase-3-backlog.md` §4.3 (G18-A-followup wave named destination).

---

### Compromise #20 — Cross-browser determinism CI cadence — PARTIALLY CLOSED at Phase-3 G18-A wave-5a

**Status:** **PARTIALLY CLOSED** at Phase-3 G18-A wave-5a (this commit); **FULL CLOSURE** deferred to G18-A-followup wave (per `docs/future/phase-3-backlog.md` §4.3) when the Playwright fixture bodies are authored. **Partially closed via:** `.github/workflows/cross-browser-determinism.yml` Playwright matrix workflow + matrix cell structure per D-PHASE-3-7 + br-r1-4 + br-r1-10.

**What landed at G18-A (workflow + matrix cell structure).** A Playwright matrix workflow runs under Chromium, Gecko (Firefox), and WebKit (Safari engine) on per-PR cadence with the matrix-cell structure for the assertions documented below. Per HONEST DISCLOSURE: every matrix cell currently emits `::warning::...harness fixture not yet wired (G18-A-followup)` — the cells are STRUCTURAL anchors only at G18-A and do NOT execute the asserted determinism logic. A regression that broke canonical-bytes determinism in the wasm32 bundle would NOT be caught by this workflow at G18-A as currently structured. The Rust-side workflow-pin tests (`bindings/napi/tests/cross_browser_determinism_workflow_pins.rs`) verify the YAML contains the expected strings — the YAML strings themselves are no-ops at G18-A.

**Matrix cells the structure pins (full-closure-eligible at G18-A-followup).**

1. **Canonical-bytes determinism per the 7 distinct engine-determinism failure-surfaces** (br-r4-r1-5): Node envelope, handler-version-chain, AttributionFrame-with-device-DID, canonical-fixture corpus CID, BLAKE3 byte identity (SIMD/non-SIMD path), Ed25519 signature byte identity, and floating-point canonicalization under DSL eval (NaN bit-pattern + denormal + round-to-even per IEEE 754 edge cases).
2. **CID-pin equivalence across the three browsers** via an explicit reduce step (br-r1-4 WHAT FAILS framing) — a divergence indicates a CRDT/DAG-CBOR encoding non-determinism that would silently corrupt cross-browser sync.
3. **IndexedDB schema-migration round-trip + 1000-key no-data-loss sweep** (D-PHASE-3-27 / br-r1-2 LOAD-BEARING per pim-2 §3.6b): exercise the `on_upgrade_needed` handler under real Chromium / Gecko / WebKit IndexedDB.
4. **`QuotaExceededError → E_STORAGE_QUOTA_EXCEEDED` typed-error mapping** (D-PHASE-3-27 / br-r1-2): write oversized data + assert the error surfaces as `BentenError(code=E_STORAGE_QUOTA_EXCEEDED)`.

**What is DEFERRED to G18-A-followup (per `docs/future/phase-3-backlog.md` §4.3).** The Playwright fixture bodies that drive each matrix cell. Estimated ~200-400 LOC of test infrastructure — the cells go from `::warning::...harness fixture not yet wired` to real assertions that would FAIL on regression per pim-2 §3.6b end-to-end test pin requirement.

**Cadence + flake-budget retry policy per br-r1-10.** Per-PR cadence (NOT release-era — Phase-2b's release-era posture is RETIRED at G18-A). Retry policy: 1 retry on browser-launch failure (`PLAYWRIGHT_BROWSER_LAUNCH_RETRIES=1`); budget = 3 launches per 24h via workflow-concurrency cap; promotion-to-required-per-PR after 30 days informational green via `branch-protection.yml` update.

**Composition with #19.** Compromises #19 + #20 PARTIALLY close together at G18-A — the Playwright matrix is the CI cell that WILL prove the IndexedDB persistence is byte-deterministic across browsers once both halves' G18-A-followup work lands. The matrix workflow's Rust-side anchors live at `bindings/napi/tests/cross_browser_determinism_workflow_pins.rs` (12 source-cite assertions covering per-browser cells + CID-equivalence reduce + flake-budget retry + the 7 br-r4-r1-5 engine-determinism surfaces) — these pins assert the WORKFLOW STRUCTURE is in place; they do not assert the fixture bodies execute.

**Cross-refs.** Compromise #19 (the durability-half companion); `.github/workflows/cross-browser-determinism.yml`; `bindings/napi/tests/cross_browser_determinism_workflow_pins.rs`; D-PHASE-3-7; br-r1-4 / br-r1-10 / br-r4-r1-5; `docs/future/phase-3-backlog.md` §4.3 (G18-A-followup wave named destination).

---

### Compromise #21 — Module manifest signing — CLOSED at Phase-3 G14-C wave-4b (BLOCKER fix-pass)

**Status:** CLOSED at Phase-3 G14-C wave-4b BLOCKER fix-pass. Full
Ed25519 manifest signing landed via
`crates/benten-engine/src/manifest_signing.rs` (`sign_manifest` +
`verify_manifest_with_mode` + [`PublisherRegistry`]) AND wired through
the production `Engine::install_module(manifest, expected_cid,
verify_args)` entry point. UCAN-proof-chain primary +
publisher-key-registry fallback per D-PHASE-3-20 + crypto-minor-5.
Audience-binding rejection via
`benten_id::ucan::validate_chain_for_audience` per CLR-2 / cap-major-2.

**g14-c-mr-1 / mr-2 BLOCKER fix-pass (this commit):**
- `Engine::install_module` now takes a third `verify_args:
  ManifestVerifyArgs` argument; the production install path invokes
  `verify_manifest_with_mode` BEFORE persisting the manifest.
  Pre-fix-pass the helper existed but was never called from
  `install_module`, making the audience-binding closure narrative
  vacuous. End-to-end pin at
  `crates/benten-engine/tests/manifest_signing.rs::install_module_rejects_unsigned_when_verification_required`
  drives the production entry point and asserts unsigned + bad-sig
  manifests reject without persisting.
- `PublisherRegistry::new` now takes a third `registry_audience_did`
  argument (the engine's own audience DID, supplied at construction).
  `require_ucan_delegation` validates the chain against this
  pre-configured DID — no more `audience_from_chain(d) == d.claims.aud`
  tautology. Cross-atrium-replay regression at
  `crates/benten-engine/tests/manifest_signing.rs::publisher_registry_rejects_cross_atrium_replay`
  asserts a UCAN signed by admin but audience-bound to Atrium-A
  rejects when replayed at Atrium-B's registry.

**What ships at Phase 2b.** `Engine::install_module(manifest, expected_cid: Cid)` REQUIRES the `expected_cid` argument (D16-RESOLVED-FURTHER — not Optional, prevents the lazy `install_module(m, None)` footgun). The engine recomputes the canonical-bytes CID over the manifest, compares against `expected_cid`, and fires `E_MODULE_MANIFEST_CID_MISMATCH` (with a 1-line manifest summary so an operator can diff without source-code dive) on disagreement. This is the minimal CID-pin integrity gate.

**What's NOT shipped (pre-G14-C narrative; preserved for audit).**
Ed25519 manifest signing — i.e. the manifest carrying a signature
field that the engine verifies against a publisher public key before
installing — was deferred to Phase-3 at Phase-2b close. The
`signature` field WAS reserved in the canonical encoding
(omitted-when-`None` so future signed manifests don't break the wire
format) but was not consumed by the install path.

**What's NOW SHIPPED (G14-C wave-4b).** The
`crates/benten-engine/src/manifest_signing.rs::sign_manifest` helper
populates the `signature.ed25519` field using
`benten_id::keypair::Keypair`. Verification flows through
`verify_manifest_with_mode(manifest, ucan_chain, registry_pubkey,
engine_audience_did, mode, now)`:

- **`ManifestVerifyMode::All`** — BOTH UCAN AND registry paths must
  verify (security-critical posture).
- **`ManifestVerifyMode::Any`** (default) — EITHER path is sufficient
  (operator-flexibility posture for non-UCAN deployments).
- **UCAN check FIRST** when both paths are present (per
  crypto-minor-5).
- **Audience-binding rejection** via
  `validate_chain_for_audience` (CLR-2 / cap-major-2: cross-atrium
  replay defended).
- **Canonical-bytes excludes signature** (crypto-major-1):
  `manifest_signed_bytes` clears `signature → None` before
  re-encoding so the bytes the signature signs are stable across
  signed-vs-unsigned manifests.

Mutations to the durable [`PublisherRegistry`] require a UCAN
delegation rooted at the registry-admin DID (crypto-minor-5; defends
"anyone can publish").

**Threat model deltas.**
- *Ships at 2b:* tampering with manifest bytes between source and `install_module` call is detected (CID mismatch → typed rejection). This protects against in-transit corruption + simple substitution attacks where the operator has the expected CID out-of-band (e.g. from a published release manifest).
- *Deferred to Phase 3:* publisher authentication. A manifest with a forged-but-byte-consistent payload installs without complaint; the engine has no per-publisher trust anchor. Trust is established via the `expected_cid` arg the operator supplies; the manifest itself doesn't carry an unforgeable origin claim.

**Phase-3 G14-C closure** (LANDED). Ed25519 signing per D16:
`manifest.signature: Option<ManifestSignature>` populated via
`sign_manifest`; verification consults UCAN proof chain primary +
publisher key registry fallback (per D-PHASE-3-20 + crypto-minor-5)
through `verify_manifest_with_mode`. The canonical-bytes encoding
preserves the reserved-field (D9-RESOLVED) discipline; the
verification arm is additive — no wire-format break.

**napi cdylib boundary (META #684 — refinement-audit-2026-05 Wave-E
HELD).** META #684 flagged that the Compromise-#21 CLOSED claim was
*engine-side only*: at HEAD the napi cdylib binding exposed ONLY the
unsigned `installModule` (hard-coded
`ManifestVerifyArgs::unsigned_development()`), so a module installed
through the Node.js binding skipped signature verification — the
signed-manifest enforcement did not survive crossing the cdylib
construction site. **Resolved on main (PR #1282/#1290):** the
`installModuleSigned` napi surface (`bindings/napi/src/lib.rs`)
threads `ManifestVerifyArgs::registry(registry_pubkey,
engine_audience_did, now)` into the same enforced
`Engine::install_module` path (Ed25519 verify against the
publisher-registry key + audience-binding; `installModule` is
retained as the explicit unsigned-development opt-in, documented as
such at the call site). Wave-E HELD added the would-FAIL-if-reverted
cdylib closure-pin
`bindings/napi/tests/install_module_signed_napi_cdylib_enforcement_1205_closure_pin.rs`
(tampered registry signature rejected; valid registry-signed manifest
admits — not degenerate deny-all). Compromise #21 stays **CLOSED**,
now honestly inclusive of the napi cdylib boundary.

**Renumbering note.** This was `Compromise #N+5` in `docs/MODULE-MANIFEST.md`'s local table prior to R6 phase-close; lifted to global #21 here.

**Cross-refs.** `docs/MODULE-MANIFEST.md` §6 + §7; `docs/ERROR-CATALOG.md::E_MODULE_MANIFEST_CID_MISMATCH`.

---

### Compromise #22 — Peer-DID + connection metadata leakage to public iroh relays — Phase-3 additive

**Status:** Introduced at Phase 3 close (P2P sync via iroh transport landed). **Closure target:** Phase 7 Garden-relay infrastructure (Garden-protocol-controlled relays replacing public iroh relays for sensitive peer-discovery + connection metadata) — failing that, Phase 9 hardened-deployment posture.

**Class.** Network-layer metadata exposure; sibling to Compromise #11's IVM coarse-grained read-gate posture but at the transport layer rather than the eval-layer.

**What ships at Phase 3.** Atrium peer-to-peer sync uses iroh's QUIC + relay protocol for NAT traversal. iroh's default relay infrastructure is *public* (operated by n0 / community relays); peers connecting through these relays expose:

- *Peer DIDs* — `did:key` / future `did:plc` identifiers visible to the relay during connection establishment (the relay sees who is talking to whom, even though it cannot read the encrypted payload).
- *Connection metadata* — endpoint pairs, timing, peer-availability windows, which Atriums a peer participates in (inferred from connection patterns).
- *Membership topology* — which DIDs co-occur in connection sessions hints at Atrium membership without the relay decrypting any application-layer content.

End-to-end *content* confidentiality is preserved (iroh's QUIC payload is encrypted; the relay is a forwarder, not an endpoint). The leak is exclusively at the transport-metadata layer.

**What's NOT shipped.** Garden-protocol-controlled relays — relay infrastructure operated under the Atrium's own trust model (Phase 7 Gardens) where relay metadata stays within the Garden's social graph rather than going to a public third-party relay. Also not shipped: relay-bypass via direct hole-punched connections only (would require giving up the NAT-traversal fallback — operationally unviable for many home network topologies).

**Threat model deltas.**
- *Ships at Phase 3:* an adversary running or compromising a public iroh relay can build a social graph of who-connects-to-whom across Atriums using the engine's default sync transport. This is a metadata-correlation attack class, not a content-disclosure class. The CIDs being exchanged stay encrypted.
- *Deferred to Phase 7 / 9:* relay-trust posture. Until Garden-relays land, operators with stricter metadata threat models (whistle-blowers, journalists, threatened communities) MUST self-host iroh relay infrastructure for their Atriums or use the engine's full peer (laptop / phone-OS app) shape exclusively on trusted networks where NAT traversal is not needed.

**Phase 7 promotion path.** Wire Garden-protocol relays per the Phase-7 Gardens design: relay infrastructure becomes a first-class Garden resource (a Garden-controlled iroh relay node, accessible only to Atrium members of the Garden, with its operator-set being the Garden's quorum of admins rather than n0 / community). The Atrium-config surface gains a `relays: Vec<RelayDescriptor>` field where each `RelayDescriptor` is either `PublicIroh` (current default — the leaky path with a documented warning) or `GardenRelay { garden_id, relay_did }`. The Atrium join handshake (per the device-heterogeneity contract and the engine's thin-client posture) extends with a relay-trust negotiation step where peers agree on the relay set before falling back to public infrastructure.

**Phase 9 hardened-deployment fallback.** If Phase 7 Garden-relays slip, the Phase-9 hardened deployment posture takes the conservative path: *no* public iroh relays in production builds; full peers MUST be on networks reachable directly OR through self-hosted relays. The hardened-deployment cargo feature flag gates the public-iroh-relay code paths out entirely. This is the brutal but correct fallback if Garden-relays don't land.

**Posture claim.** Compromise IS introduced at Phase 3 close — the public-relay metadata leak goes from theoretical (no P2P sync at Phase 2b) to live (Atriums actually sync through iroh). Operators reading SECURITY-POSTURE.md see this honestly disclosed alongside the named closure target rather than discovering it via post-Phase-3 surveillance. Defends against the failure shape "compromise silently introduced at phase-close while metadata leakage is undocumented."

**Cross-refs.** `tests/phase_3_workspace/security_posture_compromises.rs::compromise_22_public_relay_metadata_leakage_introduced_at_phase_3_close_with_named_phase_7_garden_relay_destination` (RED-PHASE assertion); `tests/phase_3_workspace/security_posture_phase_3_close.rs::security_posture_phase_3_close_compromise_table_present` (phase-close compromise-table presence pin). Phase 7 Gardens own relay infrastructure as a Garden resource; the engine's deployment-shape commitment (full peer vs thin compute surface) is described in `docs/ARCHITECTURE.md`.

**Revisit at v1-window** (per `docs/future/phase-3-backlog.md` §10; Phase-7 Garden-relays primary closure path; Phase-9 hardened-deployment fallback):
- *Question to ask:* Does v1's deployment posture (full peer + thin compute surface) ship with public iroh relays as the *only* sync path, or does v1 require operators with adversarial-relay threat models to self-host? Does the v1 audit need a "default-on metadata-leak warning" UX surface (in DSL / napi / Engine builder) so adopters self-classify against the relay-metadata threat?
- *What changed pre-v1 vs Phase-3 framing:* Phase 7 Garden-relays + Phase 9 hardened-deployment cargo-feature flag are still future scope; the public-relay leak is the live reality at v1 unless operators self-host. The v1 readiness audit must surface this with operator-facing UX, not just SECURITY-POSTURE.md prose.
- *No-action option:* keep public-iroh-relays as the default at v1 with the current honest disclosure prose; document operator self-host as the conservative deployment recommendation; defer Garden-relay infrastructure to Phase 7. Phase-9 hardened-deployment cargo feature gating remains the brutal-but-correct fallback if Phase 7 slips.

---

### Compromise #23 — Wire device-attestation envelope: COLLAPSED to provenance-label + unified-spine authority (was: cryptographic closure at Phase-3 G16-D)

**Status:** **SUPERSEDED-BY-COLLAPSE** (refinement-audit-2026-05, S3 trust-cluster, owner-ratified 2026-05-15). The Phase-3 G16-D wave-6b cryptographic closure (V2 signed `DeviceAttestationEnvelope` + `Acceptor::accept_at` + `FreshnessPolicy`) was **correct for the trust model as it stood at Phase-3 close** — it honestly closed three real wire-format gaps (DID forgery / replay / frame-pair non-binding) by composing existing hardened primitives rather than inventing a parallel unsigned transport. That closure is **not being reversed because it was wrong**; it is being **deleted because the model it served was a redundant parallel authority pipe**. The post-Phase-4-Foundation trust-model reframe (DECISION-RECORD-trust-model-reframe.md §4, RATIFIED) established that a device is not a distinct trust-root — it is a key holding capabilities over a graph, every chain rooted at the user-DID. Under that unified model the device-attestation envelope's *authority* job (deciding whether an inbound device's writes are trusted) is **the same job the user-root-anchored UCAN capability chain already performs**, and the standalone envelope was a second pipe enforcing it in parallel — the structural shape META #707 exists to eliminate.

**What COLLAPSE deletes:** the `Acceptor`/`accept_at` acceptance pipeline, the `DeviceRevocation` device-revocation pipe (its un-anchored bare-device-DID match was the #1230 BLOCKER — a *symptom of not having collapsed*), and `device-as-a-distinct-trust-root` framing. The standalone `validate_chain_with_device_revocations` walker is removed; device-key revocation is now user-root UCAN-grant revocation (`benten-caps::revoke`, already correct and self-anchored) + `RotationLog` (J4, unchanged).

**What COLLAPSE preserves (the security property is NOT weakened):**
1. **Device-DID as a provenance label.** The wire envelope's device-DID-binding signature is kept — `AttributionFrame.device_did` is still recorded and still cryptographically attributable (a peer still cannot forge another device's DID without that device's key). The Inv-14 device-grain provenance / compromised-device-quarantine *audit trail* survives; only the *trust decision* moves to the spine. (A post-COLLAPSE successor audit, #1234, confirms every remaining device-DID attribution use is elegant under the unified model.)
2. **The capability-envelope ceiling (D-PHASE-3-25 / CLAUDE.md #17).** The `runs_sandbox=false` / `holds_zones=CacheOnly` thin-shape ceiling is **retained as one signed envelope-ceiling attenuation** AND-ed into the inbound writer's effective capabilities at the single chain-validation seam — structurally unified with the plugin-manifest envelope (#669) as ONE ceiling-check. A `runs_sandbox=false` device still cannot exercise `host:sandbox:*` even with an otherwise-valid chain.
3. **Anti-replay.** The freshness-window + nonce-replay defenses are re-homed into the unified chain-validation seam's existing time-window + durable-revocation-marker machinery — not dropped. (At HEAD the time-window/freshness half is live in P3; the durable replay-marker re-home is tracked as the P2/P5 unified-ceiling deliverable — see DECISION-RECORD §4b F3.)

**F3 anti-replay durable replay-marker — atomic compare-and-swap LANDED at R6 R2 batch-A Item 8 (DEFERRED.md Row D-8 CLOSED).** The F3 marker's atomicity property is **whole** at v1-beta. `FrameReplayMarker::mark_and_check_frame` (`crates/benten-caps/src/chain_authority.rs`; symbol-form per §3.5b HARDENED point 3) routes through `benten_graph::KVBackend::compare_and_insert` (trait method in `crates/benten-graph/src/backend.rs`, redb implementation in `crates/benten-graph/src/redb_backend.rs`), which is txn-atomic on the redb-backed backend — the get + insert + commit run inside ONE redb write transaction. The prior shape (a `get` followed by a `put` across **two separate KVBackend transactions**, with the `crates/benten-engine/src/engine.rs::apply_atrium_merge` caller holding no per-engine serializing lock) admitted an in-window race where two concurrent presentations of the *same* `session_nonce` both observed absent-marker and both proceeded to admission. That race is CLOSED: the second concurrent presentation now observes `Ok(true)` (REPLAY) from the single-transaction compare-and-insert and the caller rejects the frame.

**Attack class (CLOSED).** Two concurrent peer-presented frames carrying the same nonce arriving at the receiver used to both clear the get→put window, both pass the staleness check, and both reach the per-row cap-recheck stage. Post-Item-8 the single-transaction compare-and-insert serializes them: exactly one observes first-insertion, the other is reported as a replay and rejected. Defense-in-depth (HLC-monotonic + nonce-cache + per-row cap-recheck per Compromise #25) remains as the composed backstop; the F3 layer is no longer *itself* racy.

**Composed defenses (unchanged; now defense-in-depth rather than the sole mitigation):** tight `nbf`/`exp` UCAN windows + HLC-monotonic enforcement + nonce-cache + per-row cap-recheck (Compromise #25). These held the line while the F3 get→put window was open and continue to hold behind the now-atomic marker.

**Closure path taken (Row D-8).** Of the three candidate paths — (a) extend the KVBackend trait with a typed `compare_and_insert(key, value) -> Result<bool, _>` and route `mark_and_check_frame` through it; (b) wrap both get + put inside one `GraphBackend::transaction(|tx| ...)` invocation; (c) document a serializing per-engine lock around `apply_atrium_merge`'s marker call — **path (a) LANDED** at R6 R2 batch-A Item 8. `KVBackend::compare_and_insert` is on the trait with a redb txn-atomic implementation, and `mark_and_check_frame` is its only production consumer.

**Labels.** **LIVE at v1-beta** (atomic compare-and-swap; the in-window race is closed). Retained for operator-visibility of the closure.

**Net threat-model statement (honest):** *Pre-COLLAPSE:* device-trust was enforced correctly but through a second, separately-reasoned pipe, whose revocation half was un-anchored (the #1230 perpetual-DoS). *Post-COLLAPSE:* there is exactly one authority seam (user-root UCAN chain + the unified envelope-ceiling); the device-revocation DoS is dissolved (no separate pipe to forge a revocation into); the thin-shape ceiling and device provenance are preserved. The security posture is **strengthened by collapse**, not traded away: fewer parallel pipes = fewer asymmetric-enforcement gaps (#707), and the un-anchored revocation pipe that was the live BLOCKER no longer exists.

**Honest disclosure of the closed-phase rewrite:** this section rewrites a `phase-3-close`-tagged compromise narrative. The owner explicitly authorized the closed-phase rewrite (DECISION-RECORD §4). The Phase-3 closure is preserved in the git history of this file and in `docs/history/PHASE-3.md`; this is not a retroactive denial that the Phase-3 work happened or was correct — it is the honest record that a correct closure of a redundant mechanism is best resolved by deleting the redundancy, per CLAUDE.md #5 ("delete, don't shim — fresh project").

**OPERATOR ACTION (superseded):** the Phase-3 `FreshnessPolicy` production-override operator residual no longer applies — `Acceptor`/`FreshnessPolicy` as an operator-tunable surface is deleted; freshness is now an internal property of the unified chain-validation seam (not an operator-configured acceptor).

**Cross-refs (post-COLLAPSE):** `crates/benten-caps/...` unified envelope-ceiling chain-walk (was `benten-id::validate_chain_with_attestations`); `crates/benten-engine/src/engine_sync.rs::DeviceAttestationEnvelope::verify` (provenance-binding signature retained, `Acceptor` consumption replaced by spine ceiling-AND); `crates/benten-id/src/did_rotation.rs` (J4 rotation, unchanged); successor audit issue #1234 (device-attribution-uses audit); META #707 (asymmetric-pipe class this collapse closes); #1230 (DoS dissolved by deletion); DECISION-RECORD-trust-model-reframe.md §4.

---

### Compromise #24 — Wallclock fail-closed posture (no default-clock-zero expiration bypass) — CLOSED at Phase-3 G16-B-B-rest

**Status:** CLOSED at Phase-3 G16-B-B-rest (PR #158, 2026-05-09). Earlier shape (pre-G16-B-B-rest) used `DEFAULT_NOW_SECS = 0` as an implicit fallback at `UcanGroundedPolicy`; any code path that constructed a `UcanGroundedPolicy`-evaluating chain WITHOUT injecting a wall-clock would silently evaluate the chain at epoch second zero — which falsely admitted expired UCANs (their `expires_at` invariably > 0; clock 0 vs expiry N always passes the "not yet expired" check). The G16-B-B-rest closure inverts the fall-back: any chain with `nbf > 0` OR `exp > 0` against `now_secs == 0` aborts with the typed `CapError::UcanClockNotInjected` (`E_UCAN_CLOCK_NOT_INJECTED`) rather than silently passing.

**Class.** Fail-open clock regression at any cap-evaluating surface. Sibling to Compromise #1 (TOCTOU bounded windows) at the cap-policy layer rather than the evaluator layer.

**Closure shape.**

- `crates/benten-caps/src/ucan_grounded.rs` — `DEFAULT_NOW_SECS = 0` remains as a sentinel constant, but the policy now refuses chains-with-time-bounds when `now_secs == 0` (the inversion). The `chain_has_time_bounds` helper drives the check.
- `crates/benten-engine/src/builder.rs` — engine builder threads explicit clock-inject through the `crates/benten-engine/src/builder.rs::EngineBuilder` `ucan_grounded_now_secs` field; `Engine::open` refuses to initialize the UCAN backend without a clock when a `UcanGroundedPolicy` is configured (rustdoc on the field documents the inversion).
- Typed error `CapError::UcanClockNotInjected` → `ErrorCode::E_UCAN_CLOCK_NOT_INJECTED` (catalog entry).

**Threat model closed.**

- *Pre-closure:* a developer wires `UcanGroundedPolicy` into an engine without injecting a wallclock; engine silently uses clock=0; ALL UCAN proofs with positive expiration timestamps pass as "not yet expired" regardless of when they were minted. Effective bypass of the entire UCAN expiration model. Failure mode is INVISIBLE in normal tests — every expired proof admits without warning.
- *Post-closure:* the same misconfiguration surfaces typed `E_UCAN_CLOCK_NOT_INJECTED` at the first chain evaluation. Developer cannot ship a UCAN-using engine without confronting clock injection. Production and test code both inject a real wallclock via `with_now_secs`.

**Test pin.** `crates/benten-caps/src/ucan_grounded.rs::default_now_secs_zero_fails_closed_when_chain_has_time_bounds` (inline test asserts the fail-closed branch fires when `DEFAULT_NOW_SECS=0` AND the chain has time bounds) + companion `default_now_secs_zero_walks_chain_when_no_time_bounds` (asserts the unbounded-chain branch remains permissive so the sentinel doesn't false-positive on time-unbounded grants).

**Production discipline (couples to Phase 4-Foundation).** Every new cap-evaluating surface in Phase 4-Foundation (admin UI install path; materializer pipeline; plugin manifest verify; schema compiler walk-time gating) MUST thread injected clock. Source-side discipline: no `SystemTime::now()` / `Instant::now()` in the four new crate surfaces; CI grep audit catches regressions per `.addl/dispatch-conventions.md` §3.5g cross-language-rule-mirror application. The transparent-clock-injection-at-manifest-load-surface ratification (per Phase 4-Foundation D-4F-15, Ben Q6 2026-05-11) inherits this discipline at engine-side rather than requiring plugin authors to thread clock themselves.

**Cross-refs.** `crates/benten-caps/src/ucan_grounded.rs::UcanGroundedPolicy` (the fail-closed inversion); `crates/benten-caps/src/ucan_grounded.rs::DEFAULT_NOW_SECS` (the sentinel constant); `crates/benten-caps/src/ucan_grounded.rs::with_now_secs` (the injection-at-builder surface, renamed from the misleading `with_now_for_test` per #793/P-II; production injection threads through the same `now_secs` field via the policy builder); `crates/benten-caps/src/ucan_grounded.rs::default_now_secs_zero_fails_closed_when_chain_has_time_bounds` (load-bearing test pin asserting the fail-closed branch fires when `DEFAULT_NOW_SECS=0` AND the chain has time bounds); `docs/ERROR-CATALOG.md::E_UCAN_CLOCK_NOT_INJECTED` (typed-code surface); `docs/future/phase-3-backlog.md §2.3 (i)` (the v1-assessment-window deliverable that retires the sentinel by threading `WriteContext::now` through every cap-evaluating call site — current state is operator-discipline via injection at builder; future state is per-call wallclock binding).

---

### Compromise #25 — HLC-monotonic enforcement at sync layer (adversarial-peer wallclock-injection defense) — CLOSED at Phase-3 sync-attack test family

**Status:** CLOSED at Phase-3 sync attack-test family. The defense composes three Phase-3-shipped primitives at the sync boundary: HLC-monotonic enforcement (peer cannot publish HLC values that go backward beyond their own previous publication); nonce-cache (per-session nonce store rejects replay of previously-seen sync envelopes); HLC bound inside the signed device-attestation envelope V2 (per Compromise #23 — the envelope's signature covers the HLC values, so adversarial wallclock-injection at the envelope layer is detected at signature-verify time before reaching the application-layer Loro merge). *(See Compromise #23 **SUPERSEDED-BY-COLLAPSE**: the device-attestation envelope's HLC-binding signature is retained as a provenance-binding signature on the unified spine; the HLC-monotonic + nonce-cache defenses described here are unchanged — only the device-trust-decision half of #23 collapsed, not the envelope-signature-covers-HLC binding this compromise depends on.)*

**Class.** Adversarial-peer-controlled wallclock injection at the sync transport layer. Distinct from Compromise #24 (engine-internal clock injection discipline) — this compromise addresses a peer-controlled threat surface rather than an in-process developer-configuration surface. Sibling to Compromise #23 at the HLC-payload boundary rather than the device-attestation-envelope boundary.

**Closure shape (three composed defenses).**

1. **HLC-monotonic enforcement** at `crates/benten-sync/src/handshake.rs` + `apply_atrium_merge` path. Inbound sync frames carry HLC values; the receiver's HLC oracle tracks per-peer max-seen HLC; frames whose HLC is below a peer's previous max are rejected with typed `E_HLC_SKEW_EXCEEDED`. Test pin at `crates/benten-sync/tests/attack_hlc_skew_revocation_ordering.rs` (the `hlc_skew_exceeded_in_inbound_sync_frame_rejected_with_e_hlc_skew_exceeded` test exercises an adversarial peer attempting to publish revocation-ordering past a previously-seen HLC bound; receiver rejects).
2. **Nonce-cache for replay defense** at the device-attestation `Acceptor::accept_at` path (per Compromise #23 closure). Each envelope carries a 32-byte session-nonce; the receiver's nonce-store rejects replay of any nonce already seen within the `FreshnessPolicy` window. Defends against captured-envelope-replay-with-stale-HLC.
3. **HLC bound inside signed envelope** — the device-attestation envelope V2's signed bytes include HLC fields. An adversarial peer cannot mutate HLC without invalidating the signature. Combined with defense 1, this means an adversarial peer can publish at-most their own honest HLC values (forging HLC requires forging the envelope signature, which requires holding the peer's secret key).
4. **G16-B-F structural-always-on per-row cap-recheck (PR #161)** — even if an adversarial peer manages to push a frame that passes defenses 1-3 (e.g., a peer with a legitimately-issued cap that has since been revoked re-shares an old frame), the per-row cap-recheck at `apply_atrium_merge` denies the merge against the current revocation state. Defense in depth.

**Threat model closed.**

- *Pre-closure (theoretical):* an adversarial peer pumps HLC values to suppress concurrent honest writes (HLC LWW resolution favors higher HLC); replays previously-captured envelopes to retroactively re-introduce already-revoked authority; injects fabricated HLC to forge causality.
- *Post-closure:* all three vectors close. HLC monotonicity bounds adversarial publishing to the peer's own honest progression. Nonce-cache rejects exact-bytes replay. Signed envelope binds HLC to peer identity (cannot forge HLC without forging envelope signature → requires secret-key holding). Per-row cap-recheck denies merges against revoked authority even if a frame passes envelope verification.

**Test pins (Phase-3 sync-attack family).** `crates/benten-sync/tests/attack_hlc_skew_revocation_ordering.rs` (HLC-skew + revocation-ordering); `crates/benten-sync/tests/attack_loro_op_log_inv_13.rs` (Loro op-log integrity under attack); `crates/benten-sync/tests/attack_mst_diff_cid_mismatch.rs` (MST CID-mismatch attack class); also exercised end-to-end at `tests/integration/atrium_two_device.rs` (the device-attestation envelope V2 narrative tests cover the HLC-bound-inside-signature shape).

**Posture claim.** Adversarial-peer wallclock-injection IS a real threat class — the engine's sync layer cannot trust peers to publish honest HLC values, just as it cannot trust them to declare honest device-DIDs (Compromise #23). The defense composes Phase-3-shipped primitives at sync receive time + at the cap-recheck boundary; no net-new mechanism is needed at Phase-4-Foundation. Future plugin-share boundary (Phase-4-Foundation G24-D) inherits these defenses transparently — plugin-share is just Atrium-share with a manifest envelope on top.

**Cross-refs.** `crates/benten-sync/tests/attack_hlc_skew_revocation_ordering.rs` (HLC skew + monotonicity + revocation ordering test pins); `crates/benten-sync/src/handshake.rs` + `crates/benten-sync/src/handshake_wire.rs` (HLC + nonce defenses in the handshake state machine); `crates/benten-errors/src/lib.rs::ErrorCode` (the `HlcSkewExceeded` variant of the typed `ErrorCode` enum; stable-code string `E_HLC_SKEW_EXCEEDED`); `crates/benten-engine/src/engine_sync.rs::DeviceAttestationEnvelope::verify` (envelope signature covering HLC fields; Compromise #23 cross-reference); `crates/benten-engine/src/apply_atrium_merge` (G16-B-F structural-always-on per-row cap-recheck PR #161; defense-in-depth). Plugin-share boundary in Phase 4-Foundation (G24-D plan §3) inherits via `plugin_share` calling through the same sync infrastructure.

---

## Content-hash verify-on-read at every Node-bytes surface (W9-T6 Phase-3 R5 wave-9)

**Defense added (W9-T6, ratified 2026-05-08).** `RedbBackend::get_node` now verifies the content-hash of stored bytes against the requested CID before returning the decoded Node. The redb file is treated as a system boundary; CID semantics ("self-validating identifier") are honored on every read.

**Threat model closed.** Local-disk tamper (an attacker with filesystem access to the redb file) and hardware bit-flip (cosmic-ray / disk-controller corruption) on Node-rehydration paths — handler_versions chain rehydration on `Engine::open`, engine_modules manifest+wasm-bytes registry rehydration, IVM materialise paths that rehydrate Node bodies. Pre-W9-T6, `RedbBackend::get_node` decoded the stored bytes and returned the wrong-but-decodable Node; an attacker who could swap bytes at rest could substitute one Node for another at a given CID slot, and the engine would happily execute the substituted Node as if it were the legitimate one.

**Closure shape.** `RedbBackend::get_node` routes through `benten_core::Node::load_verified(cid, &bytes)` — the same hash-then-decode helper that subgraph-load uses. Three-outcome contract pinned at the type level:

- `Ok(None)` — clean miss; CID never written.
- `Err(GraphError::Core(CoreError::ContentHashMismatch))` — bytes present but corrupted/tampered (`E_INV_CONTENT_HASH`).
- `Err(GraphError::Core(CoreError::Serialize))` — bytes hash-match but fail to decode (genuine codec drift, `E_SERIALIZE`).
- `Ok(Some(node))` — clean roundtrip; bytes hash-match and decode.

End-to-end pin lives at `crates/benten-graph/tests/get_node_verifies_content_hash_on_read.rs` (5 tests, including a "would-FAIL on silent no-op" pin per dispatch-conventions §3.6b that uses the test-only `corrupt_node_bytes_for_test` hook to mutate on-disk bytes after `put_node` and assert the next `get_node` fires `E_INV_CONTENT_HASH`).

**Out of scope (already defended elsewhere).** Cross-peer Node ingestion is defended by `Mst::apply_entries` per-entry rehash (sec-r4r2-1) — every entry's `payload` is BLAKE3-rehashed and compared byte-for-byte against the declared `cid` before insertion. Subgraph-load is defended by `Subgraph::load_verified_with_cid` (`RedbBackend::load_subgraph_verified` graph-layer wrapper). W9-T6 closes the remaining `Node`-read on-disk surface.

**Performance.** BLAKE3 over canonical DAG-CBOR Node bytes adds ~3-10 µs per `get_node` call on Apple Silicon (the budget accepted at the §6.2 ratification). Hot-loop callers (IVM materialise, repeated rehydration) absorb the cost; if future perf measurement shows the cost is load-bearing, an internal escape hatch (e.g. `get_node_unverified` for trusted callers that have already verified upstream) can be added with documented justification — but no `get_node_verified` opt-in shipped today (verify-on-read is unconditional).

**Cross-refs.** `crates/benten-graph/src/redb_backend.rs::get_node` (the verify-on-read site); `crates/benten-graph/src/lib.rs::corrupt_node_bytes_for_test` (test-only tamper hook); `crates/benten-graph/tests/get_node_verifies_content_hash_on_read.rs` (end-to-end pin); `docs/ERROR-CATALOG.md::E_INV_CONTENT_HASH` (Thrown-at line enumerates all three firing surfaces); `docs/future/phase-2-backlog.md` §6.2 (closure narrative); `crates/benten-sync/src/mst.rs::Mst::apply_entries` (sec-r4r2-1 cross-peer ingest precedent that this PR mirrors at the on-disk boundary).

---

## Repository security configuration (Phase 2a §3.1 hardening pass)

**CodeQL code scanning.** Workflow at `.github/workflows/codeql.yml` runs
on every push to `main`, every PR, and weekly cron. Findings appear in
the GitHub Security tab; not a required CI check (informational-only per
ci-decisions-2026-04-22.md §4). The workflow file *is* the configuration —
GitHub auto-enabled code scanning on first SARIF upload from the
`analyze` action; no Settings toggle was needed.

**Private Vulnerability Reporting (PVR).** Enabled at the repo level on
2026-04-25 (Settings → Code security and analysis → Private vulnerability
reporting). Gives external researchers an in-platform path to report
security issues privately rather than opening a public issue. No
maintainer action needed beyond the toggle; reports route to the
Security tab. See <https://docs.github.com/en/code-security/security-advisories/working-with-repository-security-advisories/configuring-private-vulnerability-reporting-for-a-repository>.

**Branch protection on `main`.** Spec at `.github/branch-protection.yml`;
drift-check workflow at `.github/workflows/branch-protection-spec-check.yml`.
Apply runbook + PAT setup live in the spec file's header comment. Closes
the CI-1 deferral.

**Third-party action SHA-pinning.** All workflows pin third-party actions
at commit SHAs (rather than mutable tag/branch refs). Dependabot rotates
weekly via `.github/dependabot.yml`'s github-actions ecosystem entry.
Closes the CI-3 deferral.

---

## Plugin trust model (Phase-3-close pre-v1 commitment)

**Ratified 2026-05-10.** Pre-v1-cleanup architectural commitment fixing the trust
model for app-level plugins ahead of Phase-4 plugin-manifest + admin-UI work.
Two distinct extensibility categories with deliberately separate trust shapes:

### App-level plugins — three-layer consent

App-level plugins are **subgraphs** of operation Nodes (handlers, materializers,
SANDBOX nodes), content-addressed and shared peer-to-peer through Atriums. Each
plugin has its own DID + an attenuated UCAN delegated by the user at install
time. The engine evaluator walks plugin subgraphs the same way it walks any
handler, with the active principal switched to the plugin's identity for the
walk's duration.

The trust model is layered:

1. **User-as-root.** Every capability chain traces back to a user-issued root
   grant. Phase-8 P2P plugin discovery does not weaken this — a signed,
   content-addressed plugin manifest is still rooted in user consent at install.
   No object-capability-style "possession of a token IS the right" semantics
   above the user's mint operation.
2. **Install-time manifest envelope.** The manifest carries `requires` (caps the
   plugin needs) and `shares` (policy for what other plugins are allowed to
   receive from this one). Both are signed by the plugin author so they cannot
   drift post-install. The user reviews the manifest and consents to the
   *envelope*, not to each runtime access. This is the v1 install UX surface.
3. **Runtime UCAN delegation within manifest envelope.** Plugin A may delegate a
   UCAN to plugin B if and only if B's request fits A's manifest `shares`
   policy. The CapabilityPolicy backend validates the chain at access-time:
   chain traces to user-root + each delegation step fits source plugin's policy
   + requested cap is within attenuation envelope. Plugin-to-plugin delegation
   inside the envelope does not require additional user prompts.

**Engine-side surface.** The evaluator's read pathway threads the active
principal through `Engine::read_node_as(principal, cid)` — the public surface
for any read attributed to a non-trusted principal. Engine internals (IVM,
sync, view materialization, audit, change-event fanout) reach the unchecked
storage read via `self.backend.get_node(cid)` directly — the backend field
+ accessor are both `pub(crate)`, so external crates physically cannot bypass
the policy gate. Plugin authors do not call either path directly: they
author graph nodes; the evaluator is the only caller of `_as`. Mirrors the
established `Engine::call_as` precedent at
`crates/benten-engine/src/engine.rs::call_as`. **Implementation landed in
the pre-v1 cleanup window** (closing Phase-2a-era debt: the 4 `todo!()`
stubs at `crates/benten-engine/src/engine_wait.rs:1011-1311` — `put_node`
+ `read_node_with_policy` (renamed to `read_node_as`) + the test-only
read-grant helper + the dead bench-helper sibling — closed under
`docs/future/phase-3-backlog.md §13.7`). The engine surface is independent
of the Phase-4 plugin manifest schema; both can be designed and shipped
without sequencing dependencies.

**Private namespaces.** A plugin's writes go to a DID-scoped namespace whose
cap is held by the plugin's DID. Manifest `shares=none` for that namespace
blocks delegation; the engine refuses to issue cross-plugin caps for it.
Provides a sovereign space for plugin internals (AI agents' working memory,
intermediate state, scratchpads) without breaking the cross-plugin sharing
model — same UCAN machinery, different policy.

**Storage-partition substrate (Phase-4-Meta-Core G-CORE-1 / #989).** The
`WriteContext::namespace_did: Option<Cid>` field threads a per-DID partition
selector through the storage layer. A `Some(did)` write lands in a per-DID
keyspace partition keyed on `Cid::as_bytes()` of the DID; a `Backend::scoped(did)`
view reads only that partition. The cross-DID non-leak invariant (plan §1.A
C1) holds structurally for point read, label range scan, raw key iterate,
edge keyspace, and post-commit change-subscriber fan-out — a write under
`namespace_did = Some(X)` is invisible to a view scoped to
`namespace_did = Some(Y)` for `X != Y`. The default `None` path is
byte-identical to the pre-#989 keyspace, so the Inv-13 5-row dispatch
matrix, durability tiers, SC1 system-zone ban, and the #843 canonical-bytes
golden CID are all preserved for legacy callers. This seam closes the
*authority* half of the multi-tenant isolation primitive; the
*confidentiality* half — per-DID encryption of partition bytes so an
untrusted-host or peer-holding-ciphertext peer cannot read across
partitions even with raw redb access — is the Phase-4-Meta-Core #1301
encryption-as-confidentiality substrate that this seam unblocks.

**Threat model.**

- **Scope creep at install** — defended by signed manifest. The user reviews
  the manifest at install; later changes require the user re-consent on
  upgrade. Author cannot retroactively widen the envelope.
- **Plugin-to-plugin smuggling** — defended by manifest `shares` policy
  validation at delegation time. Plugin A cannot mint a cap to plugin B that
  exceeds A's manifest envelope; the policy backend rejects the chain.
- **Object-capability bypass** (a malicious plugin tries to construct a cap
  out of thin air) — defended by chain-traces-to-user-root validation. Any
  cap presented at access time must trace back to a user-mint root. There is
  no engine sentinel principal callable from outside the engine crate.
- **Engine-internal-as-principal forgery** (a plugin tries to call the
  evaluator's unchecked read path) — at HEAD defended by the
  `Engine::read_node_as(principal, cid)` attribution discipline at the napi
  binding layer; the engine-internal un-attributed `Engine::get_node(cid)`
  is `pub` today (originally intended `pub(crate)` per CLAUDE.md #18 baked-in
  framing); visibility-tighten to `pub(crate)` is a v1-API-stabilization
  decision deferred to Phase-4-Meta per `docs/future/phase-4-backlog.md
  §4.43`. External callers who use `get_node` directly bypass attribution
  but cannot escalate caps via that path (the cap-policy check fires on the
  WRITE pathway, not the unchecked READ — at worst they read content they
  don't have attribution for).
- **Plugin-DID forgery at install / chosen-DID substitution** (a malicious
  caller tries to install a plugin with a `did:key:` it didn't actually mint
  the keypair for) — defended structurally by the **caller-mint-first
  contract** (per `docs/PLUGIN-MANIFEST.md §3 Plugin-DID minting protocol`):
  Step 8 of `install_plugin` asserts BOTH `install_record.plugin_did ==
  *ctx.expected_plugin_did` (rejects record-substitution; surfaces
  `E_PLUGIN_INSTALL_RECORD_PLUGIN_DID_MISMATCH`) AND
  `plugin_did_store.get(expected_plugin_did).is_some()` (rejects
  orphan-handle path; surfaces `E_PLUGIN_DID_HANDLE_NOT_PRE_INSERTED`). The
  Ed25519-derives-DID-from-public-key property makes this structurally
  adversary-resistant — to substitute identities an attacker must mint a
  keypair whose `did:key:` encoding matches the chosen string
  (computationally infeasible).
- **Plugin-DID duplicate-insert (caller bug or adversarial collision attempt)**
  — defended by **R6-FP-3 defensive-return hardening at
  `PluginDidStore::insert`**: returns `Err(ErrorCode::PluginDidHandleDuplicate)`
  if a handle with the same DID is already present. Closes the caller-bug
  failure mode where double-mint or double-insert in the install path would
  silently overwrite (pre-R6-FP-3 return was `()`). The same defense surfaces
  any computationally-infeasible Ed25519 keypair collision attempt as a
  typed error rather than silent overwrite.

**Trajectory alignment.** v1 (Phase 4 — Phase 4-Foundation ships the
manifest schema + admin UI v0 + install-time consent; Phase 4-Meta layers
self-composing admin on top) — small N, user reviews each manifest, simple.
Phase 6 (AI agents) — an assistant declares "I integrate with calendar
/ notes / email" in its manifest; user consents at install; the agent runs
autonomously without per-action prompts. Phase 8 (decentralized plugin
discovery) — plugins are signed by author, content-addressed, discovered
through Atrium peer groups; users trust the signed manifest, not a central
registry.

**Phase-4-Foundation R1-triage refinements (2026-05-11 night).** The base
three-layer model survives unchanged; the implementation specifics for the
plugin-identity model are refined:

- **Four distinct identity concepts** (D-4F-12 retense per
  `.addl/phase-4-foundation/r1-triage.md` Q4): Content-CID (what the plugin
  IS) + peer-DID signature on original content (provenance; `benten-id`
  RotationLog handles peer-DID rotation) + plugin-DID minted at install
  (UCAN audience AND constrained issuer within manifest envelope; per
  D-4F-16 `did:key:...` shape with engine-held Ed25519 keypair via OsRng,
  per-install fresh) + user-DID (trust anchor + signs install records).
  Cross-plugin/schema references use **content-CID, not author-DID**
  (`accepts_content: [hash, ...]`).
- **NO Benten-project-key infrastructure.** User-DID signs install records
  (the user is the source of install consent). Peer-DID signs original
  content (provenance). No central project key infrastructure for plugin
  signing.
- **Manifest schema versioning DROPPED** (D-4F-13). CID covers shape;
  pull-not-push obviates a schema-version field; T10-upgrade defense list
  no longer includes "manifest-schema-version-downgrade."
- **Plugin manifest v0 — Phase 4-Foundation implementation state.**
  Manifest envelope verified at every load (T5a defense-in-depth verify
  points: boot + per-load + per-Atrium-merge per sec-4f-r1-9). Cap-change-
  triggered fresh consent for all upgrades (silent within-lineage subset;
  full re-consent if `requires` GREW; cross-fork = user-initiated merge).
  Meta-plugin composition cycle detection AS REJECTION at install time
  (new ErrorCode `E_PLUGIN_META_COMPOSITION_CYCLE_REJECTED`); handler-call-
  graph cycle detection at handler-registration time stays Phase 4-Meta
  per `docs/future/phase-3-backlog.md §15.2`.
- **Compromise #11 (IVM views coarse-grained read-gate) closure floor
  REAFFIRMED** against the new materializer surface. Materializer SHARES
  `IvmViewReadGate` machinery (D-4F-NEW-MATERIALIZER-READ-GATE resolved
  option SHARE — materializer view IS an IVM view per D-4F-2). The
  Compromise #11 closure does not regress with the new surface.
- **Compromise #24 (wallclock fail-closed) + Compromise #25 (HLC-monotonic
  sync) REAFFIRMED** against new manifest-load surface (clock injection is
  transparent at engine-side per D-4F-15; HLC-monotonic-strict acceptance
  for peer-DID rotation per sec-4f-r1-10 T9b race-defense).
- **MVP rotation mechanism — Phase 4-Foundation** (ratification #6):
  `SelfRevocation` attestation + out-of-band new-key trust. Old-key signs
  timestamped revocation; propagates via Atrium sync; peers reject content
  signed by revoked key after revocation timestamp. **Kith** (working name;
  Phase 5+ exploratory) is the richer decentralized-identity-and-attestation
  substrate that would supersede the MVP; scaffold at
  `docs/future/kith-decentralized-identity.md`.
- **Decentralized self-discovered registry → Phase 4-Meta** (ratification #3).
  Phase 4-Foundation v0 uses direct content-addressed-share over Atriums
  (out-of-band handshake; user pulls from peer they trust). T10-discover
  threat surface FULLY N/A for v0; carries to `docs/future/phase-4-backlog.md
  §3.1`.

Full plan + implementation seams at `docs/PLUGIN-MANIFEST.md` (Phase-4-
Foundation companion doc).

**What this rules out.**

- *Pure user-as-root with per-action prompts.* Notification fatigue;
  combinatorial explosion as N plugins grow; AI-agent ergonomics fail.
- *Pure UCAN-native peer delegation without the envelope.* "I installed plugin
  A; I did not agree to A handing my data to plugin B." User loses meaningful
  control after install.
- *Plugin runtime separate from the engine evaluator.* No JS loader / FFI
  bridge / embedded interpreter. Plugins are graph; the evaluator is the only
  runtime.

### Engine-level extensions — compile-time trust

Engine extensions are **Rust crates** linked into the engine binary at compile
time. For custom IVM strategies, alternate transports (post-iroh — shaped
relays, Tor, Nostr), alternate persistence backends (post-redb — sled, fjall,
cloud-KV), custom signature schemes (post-Ed25519 — X25519, BLS, post-quantum),
performance-critical primitives that need raw Rust speed beyond SANDBOX.

**Trust posture.** "You compiled this into your engine binary." Same trust as
Benten core. There is no UCAN, no manifest envelope, no `read_node_as`
boundary. An engine extension that wants to violate invariants can — the
boundary is `cargo` and code review, not the type system.

**Audience.** People building the platform itself, not app users. The two
extensibility categories are intentionally separate worlds; trust models do
not transfer between them in either direction. Future proposals to extend the
app-level plugin trust model to engine-level extensions (or vice versa) must
be rejected with reference to the architectural commitment captured here.

**Cross-refs.** `docs/ARCHITECTURE.md` "Plugins and engine extensions" (the
architectural surface); `docs/HOW-IT-WORKS.md` "Plugins, in plain English"
(the orientation tour); `docs/GLOSSARY.md` ("App-level plugin," "Engine
extension," "Manifest envelope," "Plugin DID," "Plugin manifest");
`crates/benten-engine/src/engine_wait.rs::read_node_as` (the Class B β
surface, SHIPPED at PR #184 during the pre-v1 cleanup window — closing
Phase-2a-era debt; the four `todo!()` stubs at the historic
`engine_wait.rs:1011-1026` addresses are CLOSED at HEAD with the real
implementations of `get_node_label_only` / `put_node` / `read_node_as`
/ `resolve_subgraph_cid_for_test`); `crates/benten-engine/src/engine.rs::call_as`
(the precedent the read-side mirror follows). The Phase-4-Foundation plugin
manifest schema work shipped independently of the Class B β surface; both
landed in the same pre-Phase-4-Foundation-close window.

### Compromise #26 — Manifest-envelope recheck at sync merge boundary — SUBSTANTIVE SUBSTRATE WIRED at R6 R1 FP-F4 (Rows D-1/D-2/D-3/D-4/D-6/D-18 close); SUBSTANTIVE PluginLibrary-driven CHAIN-WALK still G-COMP-1 deferred

> **R6 R1 FP-F4 retense (2026-05-24).** Per Ben PM-ratified F1 path-(a)
> full ~13-site cascade + the F4 design pipeline synthesis v3:
>
> - **Layer-1 user-as-root WriteBoundaryChainValidator** = STRUCTURALLY
>   WIRED at every WRITE entry point via the new `Engine::admit_write_chain`
>   helper + sealed `WriteAdmissionFrame` (Row D-1 closure). The
>   always-mounted Noop default preserves Phase-3 baseline; production
>   deployments installing a substantive
>   `ProductionWriteBoundaryChainValidator` via the
>   `ProductionEngineBuilder` get fail-CLOSED at the WRITE boundary.
>   The substantive validator's `UserDidRegistry`-backed chain walk is
>   the G-COMP-1 deliverable.
>
> - **Layer-3 manifest-envelope substantive rechecker** = SUBSTRATE
>   WIRED with synthesized-fallback hardening (Row D-4 + Row D-18
>   closure). `ProductionManifestEnvelopeRechecker` returns
>   `UnresolvedDeny` on `node-id:N` synthesized DIDs; the full
>   PluginLibrary-driven chain walk for resolvable peers is the
>   G-COMP-1 deliverable per Row D-4 narrative.
>
> - **§8-E CapabilityPolicy hooks** = ALL THREE WIRED (Row D-3 close):
>   `check_install_consent` at `plugin_lifecycle::install_plugin`
>   step 3c with typed `PluginInstallConsentDenied` reject;
>   `check_per_delegation` at `EngineCapsHandle::delegate_capability`
>   with typed `PluginPerDelegationDenied` reject;
>   `check_write_with_audience` routed at all 4 production write sites
>   (audience_did stays None at sweep sites per Δv3-2; populate at
>   delegate_capability is G-COMP-1).
>
> - **§4.37 TOCTOU replay defense** = STRUCTURALLY WIRED at every
>   install (Row D-2 close): `InstallPorts.install_record_replay_check`
>   drops `Option<>` for `&mut Fn`; the `None` silent-disable arm is
>   eliminated.
>
> - **§4.25 sync-hydrate consumption** = WIRED (Row D-6 close):
>   `handshake.rs::sync_hydrate_consume_recheck_outcome` is the
>   §4.25 surface; the §4.36 + §4.25 consumption sites both route
>   through the shared `ManifestEnvelopeRecheckUnresolvedDeny`
>   ErrorCode + typed reject.
>
> The remaining G-COMP-1-deferred narrative below stays
> retrospective; the v1-beta posture is now "substrate-wired with
> a substantive PluginLibrary-driven chain-walk follow-up" (no
> longer "substrate-only").

### Compromise #26 (HISTORICAL) — Manifest-envelope recheck at sync merge boundary — SEAM SHIPPED + SUBSTANTIVE-ADAPTER DEFERRED at Phase-4-Foundation R4b-FP-1 (v1-beta posture retensed at G-CORE-9 FREEZE; cross-peer install verification NOT live at v1-beta)

**G-CORE-9 FREEZE v1-beta posture (2026-05-24 retense).** Per the
G-CORE-9 R1 triage Fork 2 doc-tighten ratification, this Compromise
explicitly documents the v1-beta-shipped state and the G-COMP-1
deferred destinations:

- **Layer-3 manifest-envelope substantive rechecker (per-DID
  PluginLibrary + UserDidRegistry consult)** = NOT LIVE at v1-beta;
  default engine ships `NoopManifestEnvelopeRechecker` returning
  `NotApplicable` for every input → admit-via-Layer-3. The structural
  empty-peer-DID fail-CLOSED at `crates/benten-engine/src/engine.rs::apply_atrium_merge` (the `ManifestEnvelopeRecheckUnresolvedDeny` arm; grep-discoverable per pim-13 §3.12 / §3.6j cite-grep-verify discipline — line numbers omitted per §3.5b HARDENED point 3 high-churn-surface rule) IS live (covers
  the "I cannot identify the writer" case); the per-resolvable-DID
  substantive recheck is deferred. Destination: `docs/V1-FROZEN-INTERFACE-DEFERRED.md`
  row "ProductionManifestEnvelopeRechecker production impl +
  default-builder wiring".
- **Cross-peer install verification (`accept_atrium_share` seam)** =
  NOT LIVE at v1-beta; the function does NOT exist as a public
  surface. The platform-foundation install pipeline at v1-beta
  consumes plugins through user-DID-signed install records ONLY
  (no cross-peer ingest). Destination: `docs/V1-FROZEN-INTERFACE-DEFERRED.md`
  row "accept_atrium_share cross-peer install seam (G24-D-FP-1
  follow-up wave)".

The remaining narrative below describes the seam half (LIVE at
v1-beta), the substantive-adapter shape (G-COMP-1 destination), and
the three G-CORE-8 R5 hardening deltas that landed in
Phase-4-Meta-Core.

### Compromise #26 — narrative (seam-shipped half, substantive-adapter deferred half)

**Status.** **SEAM SHIPPED at R4b-FP-1 (Seam 3)** (post-Q4 ratification
2026-05-13): the `apply_atrium_merge` path invokes a
`ManifestEnvelopeRechecker::recheck_row(...)` port AFTER the per-row
`CapabilityPolicy::pre_write` check (Layer-3 manifest-envelope refinement
on Layer-1 revocation defense per CLAUDE.md #18; matches engine.rs:1396-
1400 inline comment + the body Closure-shape section below). Compromise
#2 sync-replica sub-narrative already covered the Layer-1 cap check via
G16-B-F PR #161. The
production-default backend is `NoopManifestEnvelopeRechecker` (returns
`Outcome::NotApplicable` for every row) — so the seam fires + the
contract surface is wired, but at HEAD the per-row decision is
"recheck not applicable, defer to the per-row cap-policy check below."
**Substantive adapter (a `ProductionManifestEnvelopeRechecker` that
consults `PluginLibrary` + `UserDidRegistry` + invokes
`manifest_envelope_chain_validation::validate_chain_with_manifest_envelope`)
is DEFERRED to Phase-4-Meta** per `docs/future/phase-4-backlog.md §4.36`.

**Class.** Cross-boundary policy-recheck on sync ingress. Sibling to
Compromise #25 (HLC-monotonic at sync layer) but at the
manifest-envelope-shape boundary rather than the wallclock-injection
boundary. The cap-policy check already covered the leaf grant; the
manifest-envelope recheck (once the substantive adapter lands) closes
the loophole where a plugin-attributed write could arrive from a peer
whose locally-stored manifest envelope no longer authorises the
operation (e.g. user revoked + republished the manifest with `shares`
narrowed between the peer's last-seen and the merge ingress).

**Closure shape (seam half).** `crates/benten-engine/src/engine.rs::Engine`
holds a `manifest_envelope_rechecker: Arc<dyn ManifestEnvelopeRechecker>`
defaulting to `Arc::new(NoopManifestEnvelopeRechecker::new())` (engine.rs
~1729); `apply_atrium_merge` calls `recheck_row(...)` per row **AFTER**
the per-row cap-revocation check (Layer-3 manifest-envelope refinement on
top of Layer-1 revocation defense per CLAUDE.md #18 + engine.rs:1396-1400
comment). On `OutsideEnvelope`, the row rejects with typed
`E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE` (NOT a generic
`E_CAP_DENIED` — the typed code distinguishes envelope-shape denials from
leaf-grant denials at the operator-log layer). The seam composes with
Compromise #25 (HLC + envelope + nonce store defenses) — the envelope
verifies cryptographically THEN the per-row leaf grant is rechecked THEN
the manifest shape is rechecked (via the rechecker port). The
`E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE` ErrorCode was minted at
Phase-4-Foundation G24-D for the runtime delegation surface + reused at
the sync-merge boundary.

**Substantive-adapter shape (Phase-4-Meta target).** The
`ProductionManifestEnvelopeRechecker` implementation will live in
`benten-platform-foundation`, consume `PluginLibrary` (for the source
plugin's manifest `shares` policy) + `UserDidRegistry` (for chain root
verification) + invoke
`crates/benten-caps/src/manifest_envelope_chain_validation.rs::validate_chain_with_manifest_envelope`
for per-step audit. See backlog §4.36 for acceptance criteria. Until
then, the production-default `Noop` returning `NotApplicable` for every
row means **the substantive defense at the merge boundary is NOT live
in shipped binaries** — only the per-row `CapabilityPolicy::pre_write`
check from Compromise #2 sync-replica sub-narrative is.

**G-CORE-8 R5 delta (Phase-4-Meta-Core).** Three structural hardening
shipped on top of the R4b-FP-1 seam without yet wiring the substantive
production adapter (which still lands in G-COMP-1 (per docs/V1-FROZEN-INTERFACE-DEFERRED.md; phantom-wave 'G-CORE-8.2' retargeted at R6-FP-D to Row D-4); see backlog
§4.36):

1. **Typed-arm split at the `Outcome` enum
   (`ManifestEnvelopeRecheckOutcome::UnresolvedDeny`).** Pre-R5 the
   enum had only `{ OutsideEnvelope, NotApplicable }`. The
   `NotApplicable` arm admit-everythings (mapped to `Ok(())` by
   `outcome_to_row_reject`) — correct for the "rechecker has no
   context" case, but a footgun if a substantive rechecker fails to
   resolve a peer. The new typed `UnresolvedDeny` arm fails CLOSED
   with the matching new ErrorCode
   `ManifestEnvelopeRecheckUnresolvedDeny` so the security-r1-2
   distinction "I cannot decide" vs "I admit" is preserved at the
   wire-typed layer. Substantive rechecker impls landing at G-COMP-1 (per docs/V1-FROZEN-INTERFACE-DEFERRED.md; phantom-wave 'G-CORE-8.2' retargeted at R6-FP-D to Row D-4)
   are required to emit `UnresolvedDeny` rather than `NotApplicable`
   when their own internal resolution fails (e.g. missing-manifest /
   chain-root-not-loaded).
2. **Structural empty-peer-DID fail-CLOSED at
   `crates/benten-engine/src/engine.rs::apply_atrium_merge:1448-1484`
   — the two-layer defense the mini-reviewer ground-truth-verified.**
   Layer-A: BEFORE delegating to the installed rechecker, the merge
   loop calls `atrium.resolve_peer_dids(&seed.peer_node_ids).await`
   and short-circuits with
   `ErrorCode::ManifestEnvelopeRecheckUnresolvedDeny` when the
   resolved set is empty — closes the case where a Noop-defaulted
   engine would otherwise admit any merge whose peer-DID is
   unresolvable. Layer-B: the substantive rechecker impl emits
   `UnresolvedDeny` from its own internal resolution failure. Both
   layers route to the same typed code → the operator-log signal is
   indistinguishable from either layer's perspective ("the merge
   refused because the engine could not identify the writer"). The
   Noop default still returns `NotApplicable` when a peer-DID IS
   resolvable — that is the deliberate "rechecker has no context"
   shape preserved from R4b-FP-1; only the unresolvable-peer arm is
   structurally upgraded to fail-CLOSED.
3. **Three additional typed ErrorCodes minted at G-CORE-8** in the
   adjacent surfaces, each closing a previously-untyped admit/silent
   path:
   - `PluginInstallRecordAlreadyApplied` — idempotency at the install
     replay surface (`benten-engine/src/install_record_replay.rs`);
     prevents double-application of an already-installed plugin
     manifest at replay time.
   - `WriteBoundaryChainNotUserRooted` — the user-as-root invariant
     (CLAUDE.md #18 layer (a)) typed at the write boundary; the
     `WriteContext::chain` MUST trace to a user-DID root or the write
     rejects with this code rather than silently passing the
     CapabilityPolicy gate on an attenuation-only chain.
   - `ThinClientBridgePrincipalUnresolved` — the thin-client (shape
     (b) per CLAUDE.md #17) bridge surface
     (`benten-engine/src/thin_client_bridge.rs`) fail-CLOSED when the
     bridged principal cannot be resolved against the local engine's
     `UserDidRegistry`; mirrors the same I-cannot-decide-so-I-refuse
     posture as `ManifestEnvelopeRecheckUnresolvedDeny` but at the
     thin-client / IPC boundary.

The G-CORE-8 delta is `CapabilityPolicy`-trait soft-seal +
manifest-envelope-recheck typed-arm-split + the three adjacent typed
fail-CLOSED ErrorCodes. The substantive `ProductionManifestEnvelopeRechecker`
adapter remains DEFERRED to G-COMP-1 (per docs/V1-FROZEN-INTERFACE-DEFERRED.md; phantom-wave 'G-CORE-8.2' retargeted at R6-FP-D to Row D-4) per backlog §4.36; the seam +
typed-arm + structural-empty-peer-DID fail-CLOSED layer landing at
G-CORE-8 is what makes a substantive adapter drop-in-safe rather than
a wire-shape change.

**Cross-refs.** R4b-FP-1 implementer brief (`r4b/fp-1` branch); G-CORE-8
implementer brief (`g-core-8/security-surface-sealed-rebased` branch +
its mini-rev MAJ+MIN fix-pass); Inv-14 plugin-DID principal extension
(`docs/INVARIANT-COVERAGE.md`);
`crates/benten-engine/src/manifest_envelope_recheck.rs::{ManifestEnvelopeRechecker,
ManifestEnvelopeRecheckOutcome, NoopManifestEnvelopeRechecker,
outcome_to_row_reject}`;
`crates/benten-engine/src/engine.rs::apply_atrium_merge` (the
structural empty-peer-DID fail-CLOSED arm at
[`engine.rs:1448-1484`](../crates/benten-engine/src/engine.rs#L1448-L1484));
`crates/benten-caps/src/manifest_envelope_chain_validation.rs::validate_chain_with_manifest_envelope`
(the function the G-COMP-1 (per docs/V1-FROZEN-INTERFACE-DEFERRED.md; phantom-wave 'G-CORE-8.2' retargeted at R6-FP-D to Row D-4) production adapter will call into);
`benten-errors::ErrorCode::{ManifestEnvelopeRecheckUnresolvedDeny,
PluginInstallRecordAlreadyApplied, WriteBoundaryChainNotUserRooted,
ThinClientBridgePrincipalUnresolved}`.

### Compromise #30 — Unaudited PQ primitives in the v1-beta hybrid default — OPEN; MITIGATED by hybrid construction; CLOSES at v1-GM

**Code anchors (grep-discoverable per pim-13 §3.12 audit-trail-cite discipline; L14-MIN-3 close at R6-FP-D; line numbers omitted per §3.5b HARDENED point 3 / R6-R2-FP-C §3.6j cite-grep-verify discipline — symbol-form citations are the load-bearing surface):** the PQ-hybrid signature codepoint `crates/benten-crypto-suite/src/codepoint.rs::HYBRID_ED25519_MLDSA65 = 0x0001`; the PQ-hybrid KEM codepoint `crates/benten-crypto-suite/src/codepoint.rs::HYBRID_X25519_MLKEM768 = 0x647a`; the audit-gated typed-reject arm `try_pure_pq_sole_trust_path -> AuditNotLandedPurePqRejected` (the C11b safety gate ensuring unaudited PQC is never the SOLE trust path at v1-beta); the bidirectional swap-matrix conformance suite at `crates/benten-crypto-suite/tests/tf4_gcore3c_full_swap_matrix_strip_resistance_pure_pq_nondefault.rs` + `crates/benten-crypto-suite/tests/tf4_gcore3c_swap_matrix_conformance_additional.rs` + `crates/benten-crypto-suite/tests/tf4_pure_pq_gated_audit_landed.rs` covering all 7 swap-matrix arms × both wire directions per V1-FROZEN-INTERFACE item 14(a).

**Status.** **OPEN; MITIGATED.** Per the 2026-05-19 PQ-default reframe
(`.addl/pq-research/RATIFIED-pq-default-reframe-2026-05-19.md`; CLAUDE.md
baked-in #5 / #15), `v1-beta` ships PQ-hybrid by default for both
signature (byte-faithful IETF LAMPS composite Ed25519⊕ML-DSA-65
`id-MLDSA65-Ed25519-SHA512`, shared-`M'`/ctx=Label binding,
both-must-verify, NO commitment trailer) and encryption (hybrid
X25519⊕ML-KEM-768 KEM-wrap at codepoint `0x647a` + ChaCha20-Poly1305
bulk). The PQ primitive crates (`ml-dsa` RustCrypto-class for the
signature half; **`libcrux-ml-kem` for the production constant-time
ML-KEM-768**) **have no independent third-party security audit yet** —
this is the sole unmet security bar in the PQ stack (the classical AEAD
bulk layer IS NCC-audited). The compromise is the pre-audit window
between `v1-beta` and `v1-GM`.

**Class.** Dependency-on-unaudited-cryptographic-primitive, scoped to a
defined release window. Distinct from Compromise #6 (BLAKE3 collision
bound — a hash *architectural* bound; the 2026-05-19 reframe is signature
+ encryption only, so Compromise #6 / the hash posture is UNAFFECTED).

**App-layer wire-in gap (R6 R1 L2-R6-MAJOR-2 finding; OPEN; NAMED to
G-CORE-PQ-WIRE wave).** The 2026-05-19 reframe and the v1-beta default
above are SUBSTRATE-LAYER true: `benten_crypto_suite::SignatureSuite`
ships hybrid Ed25519⊕ML-DSA-65 + verify-both-must-succeed semantics
and the swap matrix exercises all 7 cipher/sig suites at G-CORE-3c
terminal wave. The APP-LAYER SHIPPED state at v1-beta is NARROWER:
4 production sites still call classical-only `ed25519_dalek::SigningKey::sign`
/ verify — `benten-drop::envelope_sig::{sign,verify}_envelope` +
`benten-platform-foundation::PluginManifest::verify_peer_signature` +
`benten-platform-foundation::InstallRecord::verify_user_signature` +
`benten-caps::AuthorizationGrant::binding_sig` (already at Row D-15e).
Wire-in is deferred to the `G-CORE-PQ-WIRE` wave (sequence: post R6 R1
FP consolidation + post R6 R2 dispatch; see DEFERRED.md Row D-26 for
the wave brief). The classical Ed25519 layer at these 4 sites IS the
audited security floor per the same hybrid-construction safety invariant
above (the classical-only-baseline-when-PQ-half-fails argument applies
to the substrate AND to these app-layer sites that use the classical
primitive directly). Pre-G-CORE-PQ-WIRE-wave-close, the v1-beta posture
on app-layer signatures is: "audited classical Ed25519 floor; hybrid
PQ defense-in-depth pending wire-in" — narrower than the substrate
posture but cryptographically sound at the audited-floor level.
**Ben 2026-05-24 PM ratification:** "do everything now is really my
default stance"; wave is queued ACTIVE not exploratory (per
`feedback_orchestrator_defer_prediction_bias` codification).

**Mitigation (why this is shippable at `v1-beta`).** The hybrid
construction means **unaudited PQC is never the SOLE trust path**:

1. **Classical floor.** The classical half of each hybrid (Ed25519 for
   signature, X25519 for the KEM, plus the NCC-audited ChaCha20-Poly1305
   AEAD) is fully present and is the audited security floor. A total
   break of the unaudited ML-DSA / ML-KEM primitive does not, by itself,
   drop security below the classical baseline the platform would have had
   shipping classical-only.
2. **Shared-`M'` / strip-resistant combiner.** The signature hybrid is the
   byte-faithful IETF LAMPS composite `id-MLDSA65-Ed25519-SHA512`: both
   halves sign a shared message representative `M'` (with context = the
   composite Label) and BOTH must verify, so neither half can be stripped
   or substituted without the verify failing closed (the prior Benten-own
   NF-4 SHA3-256 commitment trailer is dropped — the IETF composite wire
   has no slot for it; strip-resistance rests on the shared-`M'`/ctx=Label
   binding + both-halves-required).
   The encryption hybrid uses the vendored ~30-LOC X-Wing-style combiner
   over `ml-kem` + `x25519-dalek` + `sha3`. The
   typed-unsupported-algorithm-arm-never-silent-fallback contract clause
   (CLAUDE.md #5) is what enforces fail-closed — there is no silent
   downgrade path to the classical half.
3. **No content migration on close.** Because the codepoint seam is
   self-describing and old codepoints are supported forever, closing this
   compromise (or any later algorithm change) never strands or migrates
   already-written content.

**Closure condition.** **CLOSES at `v1-GM`** when the independent
third-party security audit of the pinned `ml-dsa` + `ml-kem` versions
lands during the `v1-beta` window — NF-2 / **C-GM-AUDIT** written exit
criterion: audit delivered; findings triaged per HARD-RULE-12; no
unresolved high/critical finding affects the hybrid trust path (any
high/critical remediated *and* re-verified); pinned versions ==
audited versions (no post-audit trust-path version drift without
re-audit); Ben sign-off on the actual findings. The audit is a
**committed `v1-beta` deliverable that gates the `v1-GM` tag** (not a
discretionary post-spike decision-point).

**C-GM-AUDIT scope addition (libcrux/Cryspen dependency landing).** The
production ML-KEM-768 impl is now `libcrux-ml-kem` (Compromise #32), which
pulls in **13 net-new transitive crates** — of which **10 are Cryspen/libcrux/hax**
(`libcrux-ml-kem` / `libcrux-sha3` / `libcrux-secrets` / `libcrux-traits` /
`libcrux-intrinsics` / `libcrux-platform` / `core-models` / `hax-lib` /
`hax-lib-macros` / `hax-lib-macros-types`) and **3 are general proc-macro support
crates** (`pastey` / `proc-macro-error2` / `proc-macro-error-attr2`, pulled by the
hax proc-macro layer; NOT Cryspen-authored). (The 0.0.10 bump additionally
pulls `crabgrind 0.2.6` → bindgen/clang-sys as HOST build-tooling via
libcrux-secrets, but that chain is `cfg(valgrind_ct_test)`-gated and NOT
compiled in Benten's build — see Compromise #39 for the full accounting.)
These are added to the v1-GM
C-GM-AUDIT scope as honest `cargo-vet` exemptions (the exemption-budget
*cap* was raised **5 → 18** on 2026-06-05 to cover them — a policy decision
**Ben-RATIFIED** (see the consolidated flag at
Compromise #39); pinned by `supply-chain/exemptions.toml` +
`crates/benten-engine/tests/cargo_vet_policy_self_test.rs`). The `5 → 18`
is a *cap* raise, not an entry count: the file holds **13** exemption
entries (13-of-18 cap used; the prior cap was 5, with zero
exemption entries carried over). They are
under the same audit-gated window: the v1-GM independent audit must cover
the `libcrux-ml-kem` trust path alongside `ml-dsa`/`ml-kem`.

**C-GM-AUDIT conformance-corpus addition (R16 F-17).** The v1-GM ML-KEM /
ML-DSA conformance scope must add the official **NIST `.rsp` KAT response-file
corpus** (the FIPS-203 / FIPS-204 known-answer-test response files) to the
conformance test set. At v1-beta the cross-impl KAT (`f_kat_1` — libcrux ↔
RustCrypto FIPS-203 deterministic-constructor byte-equality) proves the two
production impls agree, but it does NOT check either impl against the
authoritative NIST-published `.rsp` vectors. Adding the official `.rsp` corpus
(vs the current cross-impl-agreement witness) is a v1-GM conformance-scope
deliverable, co-scheduled with the independent PQ audit (NF-2 / C-GM-AUDIT).

**XW-NO-EXTERNAL-KAT conformance-corpus addition (R18; extends the R16 F-17 item).**
The `0x647a` **X-Wing combiner** (`SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel)`,
`crates/benten-crypto-suite/src/cipher_suite.rs`) has **no external KAT witness**
at v1-beta: it is exercised only by internal round-trip + strip-resistance +
construction-order pins (`tf2_*` / the `x_wing_combiner_preimage` witness), NOT
against an authoritative `draft-connolly-cfrg-xwing-kem-10` published test
vector. The combiner is the vendored ~30-LOC hybrid glue whose byte-exactness
against the IETF construction is load-bearing (a divergence at the reserved
codepoint is a silent interop break). Add an **X-Wing external-KAT witness**
(the draft-connolly published combiner test vectors — SharedSecret KAT over the
draft's fixed `(ss_M, ss_X, ct_X, pk_X)` inputs) to the `f_kat_*` conformance
family, co-scheduled with the independent PQ audit (NF-2 / C-GM-AUDIT).
**Cross-ref:** Compromise #30 (unaudited PQ window; the audit-gated close) +
Compromise #32 (ML-KEM Decap CT); the `f_kat_1`/`f_kat_2` conformance family.

**Rejected alternative (named, per the reframe).** "PQ-TLS as a
quantum-resistant transport envelope buys time" (Matrix's public
position) does NOT transfer to Benten: Benten's vision (baked-in #18 —
peers-hold-ciphertext / untrusted-host / at-rest replicated objects)
defeats the transport-envelope argument because Benten ciphertext rests
*at rest on peer disks*, not just in transit; Benten's actual transport
(iroh, baked-in #17) is itself classical-only with no PQ roadmap. This
is why PQ-hybrid is the default at the object layer rather than relying
on transport — and why this compromise exists rather than being
side-stepped by a transport-envelope claim.

**Cross-refs.** `.addl/pq-research/RATIFIED-pq-default-reframe-2026-05-19.md`
(authoritative reframe record + NF-1..NF-5);
`.addl/pq-research/RATIFIED-crypto-agility-2026-05-18.md` (the
build-now stack + framing, superseded-in-part); CLAUDE.md baked-in #5 /
#15; the v1-beta PQ-audit issue #1302 + issues #1300 / #1301; the four
`.addl/pq-research/landscape-*-2026-05-19.md` corroboration passes <!-- cite-drift-exempt: `.addl/` is gitignored; these planning artifacts exist locally for orchestrator workflows but are not tracked in the public tree. -->;
`.addl/pq-research/landscape-pq-algorithm-diversity-2026-05-19.md` (the
NF-1 PQ⊕PQ post-classical-death documented end-state + FIPS-207
build-trigger). **G-CORE-3c runtime gate (Phase-4-Meta-Core terminal
swap-matrix wave):**
`crates/benten-crypto-suite/src/swap_matrix.rs::SwapMatrix::try_pure_pq_sole_trust_path`
fires `SwapMatrixError::AuditNotLandedPurePqRejected` (catalog code
`E_AUDIT_NOT_LANDED_PURE_PQ_REJECTED`) when a caller attempts to
construct a pure-PQ-sole-trust-path arm (NF-1 ML-DSA-65⊕SLH-DSA sig +
ML-KEM-768-only enc, with the classical halves removed) while the
workspace-baseline `AUDIT_LANDED_PURE_PQ_FLAG` is `false` — the
runtime enforcement of the C11b safety invariant. The flag is a
compile-time `pub const`; flipping it to `true` is a v1-GM coupled
action that REQUIRES the independent audit deliverable on disk + Ben
sign-off + pinned crate versions matching the audited versions. The
named typed-arm is what the v1-GM-gating CI lane greps for (a generic
`Err` would silently regress the C-GM-AUDIT gate).

### Compromise #31 — LAMPS Composite ML-DSA combiner is EUF-CMA-only NOT SUF-CMA — OPEN at construction layer; CLOSED-EQUIVALENT at application layer via Inv-15 (Phase-4-Meta-Core mint)

**Code anchors:** the v1-beta default signature codepoint `crates/benten-crypto-suite/src/codepoint.rs::SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001` (LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512` per `draft-ietf-lamps-pq-composite-sigs-19`, OID `1.3.6.1.5.5.7.6.48`, IANA early-allocated 2025-10-20); the Inv-15 application-layer closure described in [`INVARIANT-COVERAGE.md`](INVARIANT-COVERAGE.md) "Inv-15 Phase-4-Meta-Core mint + 3-layer decomposition" section; the existing payload-CID surfaces at `crates/benten-engine/src/engine_caps.rs::Engine::revoke_capability_by_grant_cid` + `crates/benten-platform-foundation/src/plugin_manifest.rs::manifest_cid`; the G-CORE-PQ-WIRE-1 audit + property-test cluster + cite-drift-detector `LoadBearingSigBundleCidPattern` scanner extension (planned).

**Status.** **OPEN at construction layer; CLOSED-EQUIVALENT at application layer.** Per LAMPS draft-19 §9.2.2: *"NOT RECOMMENDED for use in applications where it has not been shown that EUF-CMA is acceptable."* The construction is EUF-CMA-secure (both components must verify) but NOT SUF-CMA-preserving (cannot prevent a malicious holder from minting a different-bytes signature on the same payload). It provides only Weakly-Non-Separable per LAMPS draft §10 (NOT Strongly-Non-Separable). For systems that key revocation, dedupe, or audit-uniqueness off signature bytes, the EUF-only scope admits a malleability bypass — an attacker with valid `(payload, sig)` could in principle mint `(payload, sig')` and observe different behavior wherever sig-CID was load-bearing. **The v1-beta default `0x0001` impl is now BYTE-FAITHFUL to the cited IETF construction** (the byte-faithful LAMPS composite `id-MLDSA65-Ed25519-SHA512` wire `mldsaSig(3309) ‖ tradSig(64)`, shared-`M'`/ctx=Label binding, NO commitment trailer — the prior Benten-own NF-4 SHA3-256 commitment is dropped); the §9.2.2/§10 EUF-CMA-only / Weakly-Non-Separable analysis therefore applies DIRECTLY (the commitment trailer was never the SUF-CMA mechanism — Inv-15 is).

**Why we ship LAMPS despite this** (per cryptographer-review-bird-of-prey-vs-lamps 2026-05-26 + Ben ratification "all yes across the board"):
1. **Ecosystem interop**: OpenPGP-PQC `draft-ietf-openpgp-pqc-17` mandates the same `ML-DSA-65+Ed25519` composite (RFC publication expected H1-2026; Sequoia PGP committed ship-on-publication); BouncyCastle 1.80+ / OpenSSL 3.5 / AWS KMS / Thales HSM all ship LAMPS composite.
2. **Implementation maturity**: production Rust impls (`ml-dsa` 0.1.0) + reference test vectors exist for LAMPS; not yet for SUF-CMA-preserving alternatives.
3. **WG-adopted vs individual**: LAMPS is WG document; alternatives (`draft-prabel-cfrg-suf-hybrid-sigs-01`, Bird-of-Prey per Bossuat et al. EUROCRYPT 2026 IACR 2025/1844) are individual submissions or unpublished-as-Internet-Draft constructions.
4. **The application-layer fix is cheaper and structurally cleaner** than betting v1-beta on fresh academic crypto (cf. ml-dsa CVE precedent GHSA-hcp2-x6j4-29j7 = implementation-vs-algorithm risk class L2 surfaced).

**Mitigation (why this is shippable at `v1-beta`) — Inv-15 3-layer decomposition.** Benten closes the SUF-CMA gap at the application layer via the project-wide invariant Inv-15 (Phase-4-Meta-Core mint per Ben ratification 2026-05-26): every CID-based identifier in Benten refers to a canonical PAYLOAD, never a sig-inclusive bundle. The 3-layer decomposition:

| Layer | Decision space | Benten today |
|---|---|---|
| **Identity** | What canonical bytes uniquely name "this thing" | payload-CID (Node bytes / canonical-manifest bytes / canonical-UCAN-claims bytes) |
| **Authentication** | Who attests + by which sig algorithm | codepoint-dispatched signature (LAMPS at `0x0001`; agility seam per baked-in #5) |
| **Revocation** | "This thing no longer authorizes X" | semantic tuple `(issuer, subject, cap, audience, validity)` — NOT sig-bundle-CID |

Verified 2026-05-26 (Q1+Q2 ground-truth-verify): `Engine::revoke_capability_by_grant_cid` uses Node-content-addressed CID (sig sidecar excluded); plugin `manifest_cid` is computed-then-signed (consent record signs over `(manifest_cid || ...)`). The hazard is mostly pre-mitigated by Benten's existing architecture at the load-bearing surfaces; the G-CORE-PQ-WIRE-1 wave bundles the cross-surface audit (UCAN backend / device attestation / Atrium Drop / sync merge proofs / EMIT envelopes) + the per-surface MallorySigner property tests + the cite-drift-detector scanner extension + the pim-N codification.

**Net.** SUF-CMA-equivalent application-layer security despite EUF-CMA-only construction-layer scope. The malleability bypass enumerated in the L12 finding does NOT obtain in Benten because the load-bearing identifiers + revocation semantics don't key off sig-bundle bytes.

**Closure condition (construction-layer half).** **CLOSES at `v1.x` future-additive codepoint when SUF-CMA-preserving constructions mature.** Bird-of-Prey (Bossuat et al., EUROCRYPT 2026, IACR 2025/1844) proposes a SUF-CMA-preserving combiner for the exact EdDSA+ML-DSA pair with smaller sigs (smaller than sum of components). `draft-prabel-cfrg-suf-hybrid-sigs-01` is the IETF-side individual submission. When EITHER (a) Bird-of-Prey-class construction receives WG adoption + production-quality reference impls + independent impl audit, OR (b) draft-prabel achieves CFRG-adoption + similar maturity, Benten adds the construction as an additive codepoint via the crypto-agility framework (CLAUDE.md baked-in #5) — pure additive upgrade, no wire-format break, no re-sign of historic content. Construction-layer EUF-CMA-only scope persists in historic LAMPS-signed artifacts indefinitely; new content can opt into the SUF-CMA-preserving codepoint when available.

**Closure condition (application-layer half).** **CLOSES at G-CORE-PQ-WIRE-1 wave when** (a) cross-surface audit confirms all 7 enumerated signature surfaces use payload-CID-or-tuple keyed identity/revocation (Q1+Q2 already verified; UCAN backend / device attestation / Atrium Drop / sync merge proofs / EMIT envelopes pending); (b) per-surface MallorySigner property tests land at `tests/inv15_sig_malleability_does_not_change_identifier.rs`; (c) cite-drift-detector `LoadBearingSigBundleCidPattern` scanner ships; (d) pim-N "Future signed-data designs MUST 3-layer-decompose" codified in dispatch-conventions.

**Class.** Construction-formal-property-scope-vs-application-requirements. Distinct from Compromise #30 (impl-audit-maturity scope of the underlying primitives — different threat class; both apply simultaneously to the v1-beta hybrid). Distinct from Compromise #6 (BLAKE3 hash collision bound — unrelated property at the hash layer).

**Cross-refs.** [`INVARIANT-COVERAGE.md`](INVARIANT-COVERAGE.md) "Inv-15 Phase-4-Meta-Core mint + 3-layer decomposition" (the load-bearing closure mechanism); `.addl/phase-4-meta/cryptographer-review-bird-of-prey-vs-lamps.md` (origin finding — the senior cryptographer's CONDITIONAL NO-GO on Bird-of-Prey + the elegant-permanent-shape recommendation); `.addl/phase-4-meta/critic-lens-l12-combiner-soundness.json` (initial discovery by L12 adversarial critic); CLAUDE.md baked-in #5 retense (LAMPS-default + Inv-15 + Bird-of-Prey-future-additive); `.addl/phase-4-meta/NIGHT-SHIFT-2026-05-26.md` LATE-AFTERNOON #1 ADDENDUM (Ben ratification record).

**Honest-disclosure standing.** The blog framing for Position B (in revision; tracked at `.addl/phase-4-meta/position-b-revision-roadmap.md` + dispatched revision agent at branch `phase-4-meta-core/position-b-revision-v2`) MUST surface this compromise + the Inv-15 closure mechanism + the future-additive path as honesty caveats #7 + #8 + #9 per the cryptographer-review's blog-revision-requirements. The compromise is publicly defensible; the framing is "we shipped LAMPS for interop + closed the SUF-CMA gap at the application layer via Inv-15 + Bird-of-Prey-class is on our roadmap."

### Compromise #32 — ML-KEM-768 Decap chosen-ciphertext side-channel (Decap-axis refinement of #30)

**Status.** OPEN; MITIGATED-LIVE. **Class.** Dependency-on-unaudited-cryptographic-primitive, Decap-axis (`MIT`).

This is a **refinement of Compromise #30** (unaudited PQ primitives), scoped specifically to the ML-KEM-768
**decapsulation** path. Chosen-ciphertext side-channels against ML-KEM Decap (timing / fault on the
re-encryption + FO-transform comparison) are a known implementation-risk class for the primitive. The substrate
mitigation is constant-time decapsulation (libcrux CT-Decap). **`libcrux-ml-kem` is now the LANDED production
ML-KEM-768 impl** (no longer "substrate defense" pending — the swap shipped at `ee73bb5c`/`02cc6cdc`): its
portable + AVX2 field arithmetic / NTT / serialization / generic high-level code is FORMALLY VERIFIED via
hax + F*, and on the targets where CT matters most (wasm + non-SIMD) the verified, constant-time **portable
backend** is the one selected. This is what moves #32 from deferred → **mitigated-LIVE**.

**#32-residual (the runnable CI gate; f_kat_2 FLAG-FOR-BEN).** The runnable `check-secret-independence`
CI build-gate is NOT honestly wireable at the pinned `libcrux-ml-kem =0.0.10`: building it with the
`check-secret-independence` feature on FAILS TO COMPILE (E0053 — its `impl_kem_trait!` macro does not
propagate the secret-typed `keygen`/`encaps`/`decaps` signatures; first reproduced 2026-06-05 at 0.0.9,
**E0053 PERSISTS at 0.0.10, re-verified 2026-07-20**; an upstream defect, NOT Benten's usage). The
**verified portable backend is the live mitigation** at v1-beta; the runnable
gate (the one-flag-away `mlkem-ct-check` feature seam on `benten-crypto-suite`) **carries to the libcrux
version that fixes the upstream macro** — kept `#[ignore]`'d per the no-fake-green rule rather than reported
as fake-green. Witness:
`crates/benten-crypto-suite/tests/f_kat_2_check_secret_independence_ci_gate.rs`.

**The #30↔#32↔C-GM-AUDIT chain is explicit (M-5):** #32 is the Decap-specific axis of the same
unaudited-primitive window that #30 names broadly; both **CLOSE at v1-GM** when the independent
`ml-dsa`/`ml-kem` (now incl. `libcrux-ml-kem`) audit lands (NF-2 / C-GM-AUDIT), with #32 specifically
requiring the audit to cover the CT-Decap claim. **Cross-ref:** Compromise #30 (the parent window); R0.7 §5.2
M-5.

### Compromise #53 — TransportConfig-codepoint reserve (NARROWED)

**Status.** OPEN; ACCEPTED TRADE-OFF (`ATO`). **Source.** MembershipSet panel (Ben Q1); R0.7 §3.9.

Only the `GossipPlusBlobs` transport (iroh-gossip broadcast + iroh-blobs content transfer) ships at v1-beta.
The `Willow` / `iroh-roq` / `iroh-live` `TransportConfig` variants are **reserved-and-typed-rejected** at
v1-beta (`benten-sync::transport_config::TransportConfig`): selecting a reserved transport yields a typed
`TransportReservedAtV1Beta` rejection — **never a silent accept, never a silent fallback to gossip**. The
reserves become LIVE **additively** at unused codepoints, **with no wire-break when added** (the NQ-A1
conservative-fallback discipline). **Residual:** operators whose threat models or media needs require a
reserved transport must wait for the additive v1-GM+ activation. **Cross-ref:** `docs/CRYPTO-CODEPOINTS.md`
(transport-config reserve); the F-TRANS-1 family pins the typed-reject + this disclosure.

### Compromise #63 — Sealed-Sender abuse-control trade-off

**Status.** OPEN; ACCEPTED TRADE-OFF (`ATO`). **Source.** NEW (BR-1; R0.7 §3.11).

Because the DEFAULT Layer-C path (`0x6510`, Sealed-Sender) carries NO plaintext sender identity, abuse/spam
control cannot rely on per-sender filtering. The v1-beta-Core abuse-control mechanism (Signal's delivery-token
pattern adapted to Benten's capability discipline): a recipient (or a MembershipSet admin on behalf of members)
issues short-lived, rate-limited UCAN-backed **delivery tokens**; a Sealed-Sender envelope without a valid token
is refused at the receive boundary **BEFORE decrypt**. This binds abuse-control to the capability spine without
re-exposing the sender identity. Per-token rate-limit + revocation ride the existing UCAN `nbf`/`exp` +
revocation substrate. **The token-binding AAD carries NO `coarse_epoch`** (freeze record): freshness rides the
delivery token's own UCAN `nbf`/`exp` + the `jti`-keyed nonce-cache (§3.10), never a time-bucket. **Residual
(the compromise):** a recipient who over-issues delivery tokens re-admits spam — an accepted trade-off,
mitigated by default-conservative token rate-limits. **Cross-ref:** Compromise #59 (KEM-key-confirmation
re-scoped to this abuse surface); R0.7 §3.11.

### Compromise #64 — Cross-device best-effort-eventual nonce-rejection window (NQ-T4)

**Status.** OPEN; SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`). **Source.** NEW — minted this cascade per NQ-T4
(R0.7 §10.5, Ben-ratified 2026-06-02). **Class.** Eventual-consistency window in replay-defense, scoped to the
user's own device mesh — `SGD` (same honest-architectural-disclosure class as its sibling #62 revocation-reach
+ #57: the substrate GUARANTEES per-device-durable rejection and DISCLOSES that user-global rejection is
best-effort-eventual-via-sync, not synchronous; R0.7 §5.2 dispositions #64 as `SGD`).

The `jti`-keyed nonce-cache (the replay-defense substrate that re-uses the Compromise #25 durable-CAS-marker
pattern) has two consistency tiers:

- **Per-device-durable — GUARANTEED via the durable-CAS-marker SEAM.** The cache is `jti`-keyed and retained
  for ≥ the full 1-hour bucket window. Durability is delivered through a **seam**, not intrinsic disk
  persistence at v1-beta-core: the `JtiNonceCache` (`crates/benten-sync/src/handshake.rs`) holds a
  `durable_store` consumed-`jti` set + exposes `durable_snapshot()` / `from_durable(...)` (the hydration seam a
  restart OR a cross-device sync feeds). Per-device durability is GUARANTEED **once the engine honors the caller
  contract** — persist `durable_snapshot()` to disk and re-hydrate via `from_durable` on restart — after which a
  nonce consumed on a device cannot be replayed against that same device. The FULL disk-persistence wiring (and
  the `accept_grant` `nonce_cache` backing, currently a caller-supplied in-RAM `HashSet` on the
  zero-production-caller path) is **deferred with the remote-permission / engine-encrypt-to-recipient wiring**
  (`docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-64-adjacent). (The genuinely graph-backed-durable
  `benten_caps::FrameReplayMarker<B: GraphBackend>` is a DISTINCT mechanism — it defends inbound sync FRAMES,
  not the Layer-D `jti` grant nonce.)
- **User-global — best-effort-eventual-via-sync (NOT synchronous).** A nonce consumed on device B is rejected on
  device C **only after** the cache entry propagates to C via sync. Between consumption-on-B and
  propagation-to-C there is a window in which the same remote-permission / DeviceLink token can be replayed once
  against C.

**Why this exists (and is accepted).** Synchronous user-global nonce rejection would require an online
coordinator / consensus step that the P2P-by-design model deliberately avoids (there is no always-online
authority across a user's own devices). The window is therefore an inherent property of eventual-consistency
replay-defense, disclosed honestly rather than papered over.

**Mitigations.**
1. **Durable per-device rejection** eliminates same-device replay entirely.
2. **Strict `valid_until` (NQ-T2):** the enforcement clock is full-1-second granularity, enforced strictly
   (`present > valid_until → reject`) with NO grace/skew window; the coarse 1-hour metadata bucket is never
   consulted for expiry, so it cannot widen this window.
3. **Short delivery-token / grant `exp`** (the tight-`exp` default mandate, §3.4 / Compromise #60) bounds how
   long any single token is replayable at all.

**Stays OPEN at v1-beta + v1-GM** — closing it (synchronous cross-device rejection) is out of scope for the P2P
model; the bounded window is the accepted residual. **Cross-ref:** Compromise #25 (shipped nonce-cache
substrate it re-uses); NQ-T2 (`valid_until` strict-enforcement); Compromise #60 (tight-`exp`); R0.7 §10.5
(NQ-T4 ratification) + §3.10 (nonce-cache spec).

### Compromise #65 — Wave-3e per-Node AEAD publicly-derivable-`K_principal` confidentiality limit at v1-beta

**Status.** OPEN; SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`). **Source.** NEW — minted at R13 (F-07), coupled to the
THREAT-MODEL untrusted-host honesty retense (R13 F-06). **Class.** `SGD` — the substrate GUARANTEES the AUTHORITY
half (capability / namespace isolation) and DISCLOSES that the CONFIDENTIALITY half is deferred, so per-Node AEAD
is a publicly-derivable-`K_principal` stand-in at v1-beta (same honest-disclosure class as #62 revocation-reach +
#64 nonce-window).

At v1-beta the per-Node AEAD wrap does **NOT** provide confidentiality against a malicious **storage host**. The
wave-3e `K_principal = blake3::keyed_hash(K_PRINCIPAL_DOMAIN_KEY, namespace_did)` is derived from a
**publicly-known** 32-byte domain-tag constant (`K_PRINCIPAL_DOMAIN_KEY`, `crates/benten-graph/src/redb_backend.rs:181`) + the **publicly-known** `namespace_did`, so
`K_principal` — and thus `K(N)` + the per-Node AEAD key — is **publicly derivable**: any party holding
`(namespace_did, ciphertext_blob)` can derive the key and decrypt (see the "⚠️ Confidentiality limit at this wave"
disclosure in the **Per-Node AEAD wrap layer** section of this document — `derive_test_seam_key_from_cid_with_namespace`).

Per CLAUDE.md baked-in #18 the Principal primitive has two isolation halves, and only ONE is live at v1-beta: the
**AUTHORITY half** (capability / namespace isolation, cooperating-engine-only) is the live protection; the
**CONFIDENTIALITY half** (per-principal encryption of the storage partition — the #1301 / D-64 substrate) is
**DEFERRED, NOT built at v1-beta**. So per-Node AEAD is a publicly-derivable-`K_principal` **STAND-IN** keeping the
substrate shape stable for the production `K_principal`-store swap-in, NOT real untrusted-host confidentiality.

**Why this is accepted at v1-beta.** The wave-3e use-case is holding the substrate shape stable for the production
`K_principal`-store swap-in (the swap-in replaces only the `K_principal` synthesis step; the function signature +
AEAD-wrap layer + per-chunk size are all stable). In the interim: the AUTHORITY half (namespace isolation at the
storage backend) is the live protection on a cooperating engine, and the local device's **Layer-A vault**
(Argon2id-DAK-sealed) protects the *local* vault at rest.

**Stays OPEN at v1-beta; CLOSES** when the #1301 / D-64 per-DID secret-material `K_principal` backend lands.
**Cross-ref:** `docs/THREAT-MODEL.md` §1 (untrusted-host tier row + honesty note); CLAUDE.md baked-in #18; the
per-Node AEAD "⚠️ Confidentiality limit at this wave" section (this document); Row D-64 / #1301 (the deferred
confidentiality substrate); `docs/future/phase-4-backlog.md §3.10` (K_principal-per-DID secret-material backend).
Contrast the Tier-1 network-observer "sees plaintext = NO" (Layer-C encrypt-to-recipient — a DIFFERENT, live
mechanism, NOT this stand-in).

### Compromise #66 — Recovered-secret `Debug`-render + freed-heap hygiene across the crypto-suite secret roster (CLOSED-at-v1-beta)

**Status.** CLOSED at v1-beta (hardened in the R19/#3 secret-hygiene sweep); SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`)
— now a POSITIVE guarantee (redact-on-`Debug` + zeroize-on-drop for the recovered-secret roster), no longer an open
footgun. **Source.** MINTED at R14 (GAP-1); CLOSED at R19/#3.

**History (R14 mint).** R14 disclosed that `UnwrappedKey` (`crates/benten-crypto-suite/src/cipher_suite.rs`) — the
recovered `k_root` returned by `unwrap_key_material` — then carried `#[derive(Debug)]`, so a `{:?}` render would
have printed the recovered KEY BYTES in the clear, and had **NO** zeroize-on-drop, so its bytes lingered on the
freed heap. At mint-time it was the asymmetric outlier vs its sibling `RecipientSecret` (which already did both).
That gap is now CLOSED.

**What landed (R19/#3 secret-hygiene sweep).** The whole recovered-secret roster now has a redacting hand-written
`impl Debug` (rendering `<redacted>` / `[REDACTED]`, never the raw bytes) + an explicit zeroizing `impl Drop`:

- **`UnwrappedKey`** — redacting `impl Debug` (`cipher_suite.rs`) + zeroizing `impl Drop`
  (`cipher_suite.rs`). `UnwrappedKey` now MATCHES `RecipientSecret` — no longer an outlier.
- **`DecryptedPlaintext`** (recovered Node plaintext) — redacting `impl Debug` (`cipher_suite.rs`) +
  zeroizing `impl Drop` (`cipher_suite.rs`).
- **`VaultPayload`** — redacting `impl Debug` (`k_principal` + `user_did_signing_key` → `<redacted>`,
  `vault.rs`) + zeroizing `impl Drop` (`vault.rs`); also protects the derived-`Debug` cascade
  through `DecodedVault`.
- **`ProvisioningInnerPayload`** (Layer-D device-link recovered payload) — redacting `impl Debug`
  (`device_link.rs:120-134`) + zeroizing `impl Drop` (`device_link.rs:143-149`).
- **`PurePqMlKemKeypair`** — zeroize-on-drop landed earlier at R18 C3.

**Enforcement.** `crates/benten-crypto-suite`'s hygiene is held by the LIVE meta-test
`crates/benten-engine/tests/f_secret_hygiene_roster.rs` (470 LOC, ZERO `#[ignore]`): a runtime
Debug-does-not-leak assertion (constructs each secret type with a distinctive `0xDEADBEEF` marker, `format!`s it,
asserts the decimal-array rendering a leaking derived `Debug` would emit is ABSENT) + a source-anchored
zeroize-coverage grep-defense (asserts a `Drop`/`zeroize()`/`ZeroizeOnDrop`/`zeroize`-feature wiring is present in
source for every roster type). An in-crate assertion at `cipher_suite.rs` additionally asserts the `Debug`
render contains `<redacted>`. A revert (e.g. re-deriving `Debug` on any roster type) re-fires the meta-test.

**Residual (v1-GM nicety, narrower).** The `Debug`-render + freed-heap hygiene is CLOSED. R6-reround additionally
wrapped the transient PLAINTEXT buffers that briefly hold the full serialized/decoded secret bytes around the
redacting/zeroizing `VaultPayload`: `serialize_vault`'s pre-seal `pt` and `decode_vault`/`open_vault`'s
decrypted-plaintext `pt` (each holds the whole `k_principal` + `user_did_signing_key` payload before/after the
struct parse) are now `Zeroizing`-wrapped (`crates/benten-crypto-suite/src/vault.rs`), and the `benten-drop`
Layer-C seal-side CEKs — the single-recipient BLAKE3-derived CEK and the fresh-random `0x6520` group CEK — are
`Zeroizing`-wrapped (`crates/benten-drop/src/layer_c.rs`). So NO transient full-secret plaintext buffer in the
vault seal/open path nor a seal-side Layer-C CEK is left un-wiped. What remains Row-D-75-deferred is the narrower
tidy for the still-bare-`Vec<u8>` copy sites whose zeroizing return is freeze-coupled (frozen trait/return-type
signatures): `unwrap_key_from_recipient`'s return (`crates/benten-crypto-suite/src/hpke.rs`), the `recovered`
binding in `crates/benten-engine/src/layer_d/device_link.rs`, and the `k_root` copy in
`crates/benten-crypto-suite/src/swap_matrix.rs`. Tracked at `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-75. The
separate `derive_member_key` raw-`Vec<u8>` hardening rides Row D-76.
**Cross-ref:** `crates/benten-crypto-suite/src/cipher_suite.rs` (the redact+zeroize roster);
`crates/benten-engine/tests/f_secret_hygiene_roster.rs` (enforcing meta-test);
`docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-75 + Row D-76 (remaining bare-`Vec<u8>` tidy).

> **Compromise #62 detail (revocation reach)** lives at the renumbered in-tree section
> "Revocation reach (§R6) — Compromise #62 detail (RE-POINTED from in-tree #31 per BR-2)" below +
> the "Revocation reach — online-pull vs offline-Drop asymmetry (G-CORE-3f)" section — verbatim-preserved
> from the in-tree #31 body, renumbered per BR-2 (this is a renumber, not a rewrite).

The honest-disclosure rows **#33–#52, #54–#58, and #61** each carry a stand-alone detail section below (the
F-full doc-wave authored them from R0.7 §5.2 so the `F-DISC-1` disclosure-coherence catch-net resolves the
literal `Compromise #N` token + the word-boundary `disposition_class` abbreviation per row). Every section names
its class as one of the five F-full taxonomy tokens — `ATO` (Accepted-Trade-Off), `SGD`
(Substrate-Guarantee-Disclosure), `CHD` (Composition-Hazard-Honest-Disclosure), `OOS` (Out-Of-Scope), `MIT`
(Mitigated-Open) — matching the summary-table disposition. Scoped-gap rows (`OOS`/`SGD`) are disclosed honestly,
never over-claimed as fully closed.

### Compromise #33 — Coercion / wrench attack OUT-OF-SCOPE (+ Layer-D approval-coercion)

**Status.** OUT-OF-SCOPE (disclosed) (`OOS`). **Source.** 9-eyes; m-5 disclosure-scope extension (R0.7 §5.2).

Password / physical-coercion ("wrench") attacks are outside the cryptographic threat model: no cryptosystem
defends a principal who is compelled to unlock. **m-5 extension (Layer-D approval-coercion):** a coerced
approving-device (forced to answer a `RemoteUnlock` / `SignUcanDelegation` prompt) makes a coerced grant look
legitimate forever via the audit-Node — distinct from the #34 password-coercion axis. This is disclosed, not
mitigated; the residual is inherent to any human-in-the-loop authority. **Cross-ref:** Compromise #34
(password-knowledge); R0.7 §5.2 (m-5).

### Compromise #34 — Password-knowledge implies full access (Argon2id defense-in-depth)

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** 9-eyes (R0.7 §5.2).

Whoever knows the principal password derives the DAK (Device-Authentication-Key) and therefore the vault. Argon2id
(RFC 9106; OWASP params `m_cost=19456, t_cost=2, p_cost=1`) raises the **offline-guess** cost substantially but
does not change the underlying knowledge-implies-access property: a known password is full access by design. This
is the accepted trade-off of password-derived key material; the mitigation is cost-hardening, not a different
trust model. **Cross-ref:** Compromise #33 (coercion); R0.7 §2.2 / §3.4 (DAK substrate).

### Compromise #35 — Compromised-device retroactive decryption (no past-content FS at v1-beta)

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** 9-eyes (R0.7 §5.2).

A device whose long-term key is compromised can retro-decrypt content it already held — there is no past-content
forward-secrecy at v1-beta (CGKA/MLS-PQ is deferred post-v1-beta, codepoint-bracket-reserved per U13). The
attacker gains exactly what that device was entitled to, no more (the blast-radius is the per-device ladder in
THREAT-MODEL.md, not the whole principal). Full PCS / past-content FS is the deferred CGKA work. **Cross-ref:**
Compromise #42 (Layer-C FS-gap); Compromise #52 (no-PCS-against-removed-members); `THREAT-MODEL.md` (O-6
blast-radius ladder).

### Compromise #36 — RAM-residency / coredump / swap forensic-extraction OUT-OF-SCOPE

**Status.** OUT-OF-SCOPE (disclosed) (`OOS`). **Source.** 9-eyes (R0.7 §5.2).

Plaintext key material in process RAM extracted via coredump / swap / cold-boot forensics is outside the
cryptographic threat model. `zeroize` (best-effort memory-wipe on drop) + `secrecy` (the O-1 Layer-A dep wrapping
secret bytes to resist accidental logging / `Debug` leakage) are **best-effort hardening, not a guarantee** — a
privileged local attacker who can read process memory or a swap-file is outside scope. Disclosed honestly so
operators do not assume RAM-secrecy. **Cross-ref:** Compromise #39 (the `secrecy` dep disclosure); R0.7 §5.2.

### Compromise #37 — No TEE / sealed-enclave attestation at v1-beta + v1-GM

**Status.** SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`). **Source.** 9-eyes (R0.7 §5.2).

Benten makes **no** TEE / sealed-enclave / remote-attestation claim at v1-beta or v1-GM: key material rests in
ordinary process memory protected only by OS process boundaries. There is no hardware root-of-trust, no
SGX/TrustZone/Secure-Enclave sealing in the v1 substrate. This is disclosed as a substrate guarantee boundary
(what the substrate does NOT provide), not over-claimed — a deployment requiring hardware attestation must layer
it externally. **Cross-ref:** Compromise #36 (RAM-residency); R0.7 §5.2.

### Compromise #38 — Physical-presence side-channels OUT-OF-SCOPE

**Status.** OUT-OF-SCOPE (disclosed) (`OOS`). **Source.** 9-eyes (R0.7 §5.2).

EM / power / acoustic / micro-architectural-timing side-channels that require physical presence (or co-resident
hardware) are outside the threat model. The constant-time disciplines Benten DOES adopt (libcrux secret-independent
ML-KEM per the `check-secret-independence` gate; constant-time AEAD) defend the **remote/network** adversary, not
a physically-present attacker with measurement apparatus. Disclosed, not closed. **Cross-ref:** Compromise #32
(Decap CT-mitigation, the in-scope timing axis); Compromise #51 (FFI marshaling side-channels); R0.7 §5.2.

### Compromise #39 — Supply-chain dependency-pinning posture (PARTIAL)

**Status.** SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`). **Source.** 9-eyes; O-1 (R0.7 §5.2).

Dependency pinning is **PARTIAL** at v1-beta: `cargo deny` + the RustSec advisory gate run in CI. **The live path
uses NO `hpke` crate** (R10-council GAP-A correction): there is no `hpke` crate dependency in any `Cargo.toml` and
no HPKE key-schedule on the live path — Layer-C is the Benten-supplied X-Wing **KEM-DEM** (NQ-C1). The **McMillion
`hpke`** (NOT Cryspen `hpke-rs`, which carried 13 CVEs Feb 2026) is a **RESERVED** pin for the additive
RFC-9180-faithful key-schedule branch ONLY — the standing posture recorded so the choice does not drift IF/WHEN
NQ-C1 ratifies that branch, not a currently-linked dependency. The KEM half is **libcrux-ml-kem** (verified
secret-independence; the production impl). **O-1 disclosure:** `secrecy` is a NEW Layer-A
dependency introduced this arc (wrapping secret bytes), disclosed here as a supply-chain surface. **F-full
disclosure (2026-06-05):** the production ML-KEM-768 swap to **libcrux-ml-kem** (Compromise #32 mitigation) pulls
13 net-new transitive crates (the `libcrux-*` / `hax-lib*` / `pastey` / `proc-macro-error2*` / `core-models`
family — all Cryspen / well-known, all Apache-2.0 / MIT-OR-Apache-2.0). Separately, the 0.0.9→0.0.10 bump
(2026-07-18) pulls **`crabgrind 0.2.6`** (a Valgrind constant-time-test C-FFI binding — the least-well-known crate
in the tree) via **libcrux-secrets 0.0.6**, and crabgrind build-depends on **bindgen + clang-sys** (+ cc /
pkg-config / …). This entire `crabgrind → bindgen/clang-sys` chain is **`cfg(valgrind_ct_test)`-gated** — a cfg
Benten NEVER sets — so it is present in `Cargo.lock` but is **NOT compiled** in Benten's build (host build-tooling,
feature-gated-OFF, non-crypto-trust-path; `cargo tree -i bindgen` on the default target prints nothing). It carries
no `cargo-vet` exemption entry (nothing compiles it); it is enumerated in the `supply-chain/exemptions.toml` header
for accounting completeness. The 13 runtime crates are **unaudited-by-Benten** and
recorded HONESTLY as accepted-unaudited `cargo-vet` exemptions in `supply-chain/exemptions.toml` (the
exemption-budget raised 5 → 18 on 2026-06-05 — a policy decision now **Ben-RATIFIED**;
pinned by
`crates/benten-engine/tests/cargo_vet_policy_self_test.rs::cargo_vet_exemption_budget_within_ratified_cap`).
**✅ RATIFIED (authoritative site):** the 5 → 18 exemption-budget cap raise is Ben-RATIFIED; the ratified cap is
**18**, and every other site referencing the budget bump defers to this ratified value. The exemptions are
interim until the independent
ML-DSA/ML-KEM audit (NF-2 / C-GM-AUDIT) that GATES v1-GM covers the pinned ML-KEM impl. Full reproducible-builds + SLSA-3+ provenance is the SEPARATE post-v1-GM commitment (#40).
This row discloses the partial-pinning substrate honestly; it is not a closed guarantee. **Cross-ref:**
Compromise #32 (ML-KEM production impl); Compromise #40 (reproducible-builds); R0.7 §2.2 (tactical picks);
§5.2 (O-1).

> **Dependency-posture note (Layer-C — no `hpke` crate; R10-council GAP-A).** At v1-beta the Layer-C `0x647a`
> X-Wing **KEM-DEM** is implemented over Benten's own vetted-primitive call site (`libcrux-ml-kem` (via
> `benten_crypto_suite::mlkem`; RustCrypto `ml-kem` is the dev-only KAT witness) + `x25519-dalek` + `sha3` +
> `chacha20poly1305`; see `docs/CRYPTO-CODEPOINTS.md` §"HPKE KEM-extensibility (NQ-C1)"). There is **NO `hpke`
> crate and no HPKE key-schedule** on this live path. The McMillion-`hpke`-not-Cryspen choice is a **RESERVED**
> dependency-pinning posture for the additive RFC-9180-faithful HPKE key-schedule/AEAD branch ONLY (a Ben-gated
> NQ-C1 wire decision), recorded so the choice does not drift if that branch is later adopted.

### Compromise #40 — Build-time / reproducible-builds + SLSA-3+ posture (post-v1-GM)

**Status.** SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`). **Source.** 9-eyes (R0.7 §5.2).

Reproducible-builds + SLSA-3+ supply-chain provenance are **post-v1-GM** commitments, not v1-beta guarantees. At
v1-beta there is no bit-reproducible build attestation and no signed provenance chain from source to artifact. This
is disclosed as a substrate boundary (the guarantee does not yet exist), tracked toward post-v1-GM, and not
over-claimed. **Cross-ref:** Compromise #39 (dependency-pinning, the v1-beta partial posture); R0.7 §5.2.

### Compromise #41 — Cross-device-sync UX-vs-cryptographic boundary (+ revocation-propagation-lag)

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** 9-eyes; O-4 (R0.7 §5.2).

The cross-device sync UX surface (what the user perceives as "this is revoked / linked now") and the cryptographic
boundary (what the bytes actually enforce) differ. **O-4 sub-clause (revocation-propagation-lag):** a revoked
device can exercise a stale grant during a network partition — bounded by tight UCAN `exp`, but non-zero. This is
the same eventual-consistency residual the #64 nonce-window names, here at the grant-revocation axis. Accepted at
v1-beta; mitigated by the tight-`exp` default mandate (§3.4). **Cross-ref:** Compromise #64 (cross-device
nonce-window); Compromise #60 (tight-`exp`); R0.7 §5.2 (O-4).

### Compromise #42 — Layer-C forward-secrecy gap (HPKE-mode-base long-term sk)

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** 9-eyes (U13) (R0.7 §3.3 / §5.2).

HPKE-mode-base is structurally **non-forward-secret at the long-term-sk axis**: a recipient's long-term secret key
decrypts every envelope ever sent to it, so a 2030 sk-compromise recovers 2026 envelopes. Partial FS is available
via application-layer key rotation; full per-message / per-epoch FS is the CGKA/MLS-PQ work deferred post-v1-beta
(codepoint-bracket-reserved `0x6380..0x63CF` per U13). The journalist per-message-FS threat is a **SEPARATE** design
class (#56), kept sharply distinct so the audit does not read the two as duplicates. **Cross-ref:** Compromise #35
(compromised-device retro-decrypt); Compromise #56 (journalist per-message FS); R0.7 §3.3.

### Compromise #43 — Envelope metadata leakage to untrusted relays — IMPROVED by Sealed-Sender DEFAULT

**Status.** ACCEPTED TRADE-OFF (`ATO`); IMPROVED. **Source.** 9-eyes (L6); BR-1; #61 (R0.7 §3.8 / §5.2).

Envelope metadata observable to an untrusted relay is an **accepted trade-off**, materially **IMPROVED** by the
Sealed-Sender DEFAULT (BR-1). The default Layer-C path (`0x6510`, Sealed-Sender) **removes the plaintext sender-DID**
from the wire; there is **NO coarse-epoch on the Drop wire** (the 1-hour bucket is Layer-D-only per RULING-1 / M-14);
and group-AAD set-identifying material is **BLINDED** — the `membership_set_id_commitment` is a K_Set-**KEYED** BLAKE3 MAC
(per Compromise #61) while the `audience_set_commitment` is an **UNKEYED** `BLAKE3` over the sorted roster. The
**residual** observable on the default Drop wire is the recipient DID plus linkable-but-blinded group tags; and because
the `audience_set_commitment` is unkeyed it hides only a **high-entropy** roster — for a **guessable / low-entropy**
roster (the guess space narrowed by the plaintext `member_count`) it is itself a confirmation-oracle + equality-linker
(recompute `BLAKE3(sorted-roster)` to confirm a guess; identical rosters → identical tags → linkable), the audience-axis
sibling of the `body_cid` residual disclosed below. Full per-send unlinkability is roadmap (U22–U28; **U25 is the
v1-GM-reserve** for the linkage half; a keyed `audience_set_commitment` per Row D-36 is the additive guess-confirmation
half). Disclosed as an honest, scoped residual — not an over-claim of network-observer invisibility.

**`body_cid` low-entropy confirmation/equality-linkability residual (R10 F-01, honest disclosure).** The Layer-C
AAD carries the wire `body_cid` as an **unsalted** `self_describing_cid(BLAKE3(plaintext))`
(`benten_drop::layer_c::self_describing_cid` over `blake3::hash(&body)`), emitted **in plaintext**. For **low-entropy /
guessable** bodies this is (a) a **confirmation oracle** for a Tier-1 network observer (guess a candidate body → recompute
`BLAKE3` → compare against the wire `body_cid`, no key material required) and (b) a **plaintext-equality linker**
(two sends of the identical body carry the identical `body_cid`, independent of the random-nonce ciphertext
distinctness). This is bounded to low-entropy payloads — high-entropy bodies keep the guess space intractable — and
the `body_cid`-in-AAD is load-bearing (origin-auth binding + U3 length-injectivity; `open_group_stanza` recomputes
and fail-closes on mismatch, so it is NOT droppable). **Mitigations:** application-layer padding / randomization for
low-entropy payloads; a **per-send `body_cid` salt** is additively reservable (codepoint-reserve, no wire-break per
CLAUDE.md baked-in #5) if judged load-bearing. DOC-ONLY disclosure — no wire byte changes. **Cross-ref:**
`docs/SECURITY-PROOFS.md` §4.2; `docs/THREAT-MODEL.md` §1 (Tier-1 row `body_cid` note). **Cross-ref:** Compromise #58 (insider-correlation boundary — unlinkability is network-observer-only);
Compromise #61 (gossip-topic blinding); Compromise #63 (Sealed-Sender abuse-control trade-off); `THREAT-MODEL.md`
(network-observer-only unlinkability scoping); R0.7 §3.8.

**Positive as-built property — inter-member sender-origin non-forgeability (B2 ORIGIN-AUTHENTICATION; ENFORCED in
code, B2 #1366).** Complementary to the metadata-leakage trade-off above, the same Sealed-Sender path POSITIVELY
guarantees that a **co-member / co-sealer CANNOT forge sender origin** on the online Layer-C path: removing the
plaintext sender-DID did NOT weaken attribution. Each Sealed-Sender send carries, inside the once-sealed body, a
per-MESSAGE LAMPS-hybrid `id-MLDSA65-Ed25519-SHA512` sender signature over a domain-separated `M_auth` binding
(sender-DID ‖ `body_cid` ‖ audience commitment ‖ key-epoch generations ‖ `stanza_count` ‖ body-AAD digest); each
recipient verifies BOTH hybrid halves post-decrypt against the set-state it INDEPENDENTLY holds, **fail-closed**
(`SenderOriginAuthFailed`). A holder of `K_Set` can produce valid AEAD tags but CANNOT mint a send attributed to
another member nor re-target another member's body to a set that member never chose — forging requires the target's
hybrid (post-quantum) signing key. This is an enforced strength, not a residual. **Cross-ref:**
`docs/SECURITY-PROOFS.md` §4.1 (the inter-member non-forgeability decomposition + the substantive `f_lc_3_*`
defense arms: second-sealer-spoof / second-member-spoof / re-target / stale-generation / strip-PQ-half);
`docs/THREAT-MODEL.md` §1 (the positive-property tier note).

**Generation-staleness / anti-re-target defense of record (R9 F-07).** The cross-generation-replay / stale-generation / anti-re-target defense IS the B2 `M_auth` recompute-on-open described above: each recipient re-derives the key-epoch generation words (`recipient_key_generation` for `0x6510`/`0x6520`; `member_key_generation` ‖ `membership_set_generation` ‖ `role_assignments_generation` for the `0x6610` group path) from its OWN independently-held set-state (NEVER the wire) and fail-closes the hybrid LAMPS verify (`crates/benten-drop/src/layer_c.rs`, `open_group_stanza` / `open_membership_set_group`). The earlier `benten_sync::two_cid_store::verify_stanza_generation` — a plaintext, unsigned, monotonic `<` compare with NO seal-side producer — was a strictly-weaker parallel model with no live call site and was **DROPPED (R9 F-07)**; its `DualCidStore` / `reseal` / `blind_set_cid` siblings were byte-for-byte redundant with the live `body_cid` recompute + `membership_set_id_commitment` and were dropped with it. The `k_principal_generation` (U20) axis it modeled is subsumed **by construction**: `K_principal` is the at-rest / vault (encrypt-to-**self**) key and is ABSENT from the recipient Layer-C path, so no per-stanza `k_principal_generation` rides the live wire; K_principal rotation is reseal-heavy (a rotated principal yields a fresh envelope that an old-generation body cannot verify under), and sender signing-key rotation needs no live counter-compare inside `verify_m_auth` (`crates/benten-drop/src/layer_c.rs:283`): both v1-beta identity methods are self-certifying — **`did:key`**, where the DID *is* the verifying key, and **`did:benten`**, whose method-specific-id EMBEDS the composite signing multikey ahead of the committed key-set CID, so `Did::resolve_signing` recovers the sender key zero-I/O from the DID itself; either way there is no DID→key indirection to consult and no `RotationLog` reference exists anywhere in `benten-drop`. `RotationLog` applies only to *rotatable* DID methods and is an out-of-band identity-resolution concern (resolved before the sender key reaches `verify_m_auth`), not a live consult inside it. No standalone counter-compare is needed.

### Compromise #44 — Long-term-confidentiality posture (BSI TR-02102-1; acceptable-migration-window)

**Status.** OUT-OF-SCOPE (disclosed) (`OOS`). **Source.** 9-eyes (R0.7 §5.2).

The very-long-term (decades-horizon) confidentiality guarantee is outside the v1-beta posture. The X-Wing /
MLKEM768-X25519 hybrid is disclosed as an **acceptable-migration-window** choice per BSI TR-02102-1 — adequate for
the foreseeable migration horizon, not a decades-proof guarantee. Operators with multi-decade confidentiality
requirements must plan for the additive PQ⊕PQ end-state (#30 / NF-1) as it matures. Disclosed, not closed.
**Cross-ref:** Compromise #30 (unaudited-PQ / NF-1 end-state); R0.7 §5.2.

### Compromise #45 — ML-KEM-768 MAL-BIND-K-CT / K-PK binding-properties

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** MembershipSet panel; M-6 (R0.7 §3.3 / §5.2).

The ML-KEM-768 binding properties (MAL-BIND-K-CT / MAL-BIND-K-PK) connect to the M-6
IND-CCA2-under-adversarially-chosen-recipient-seed audit line: the device-link / remote-permission flows DO admit a
chosen-recipient-pubkey surface (a malicious device B supplying an adversarial pubkey). This is named as an
**external-cryptographer-audit disclosure surface** (a §9.3 audit-deliverable line), NOT a unit-test "proof" — no
formal-methods lens has confirmed tractability under that model at v1-beta. Accepted-and-disclosed pending the
independent audit. **Cross-ref:** Compromise #59 (KEM-key-confirmation); Compromise #30 (audit-gated window);
`SECURITY-PROOFS.md` (the AAD per-stanza binding); R0.7 §3.3 (M-6).

### Compromise #46 — `HpkeMultiBase` O(N) wire-cost above 32 recipients

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** MembershipSet panel (R0.7 §5.2).

The multi-stanza `HpkeMultiBase` group send carries one HPKE stanza per recipient, so wire-cost grows **linearly**
(O(N)) with recipient count. Per-Kind cardinality caps bound it: Atrium ≤32 / DeviceMesh ≤5 / SingleDevice =1. Above
those caps the linear cost is the accepted trade-off of the per-recipient-stanza design (the alternative — a shared
mutable group object — is the deferred CGKA class). Accepted at v1-beta. **Cross-ref:** Compromise #42 (FS-gap, same
no-shared-mutable-object posture); R0.7 §3.3 (`HpkeMultiBase`).

**DOS-6520-QUADRATIC-SEAL sub-note (R18; SGD resource-bound disclosure — SENDER-side, typed cap at the hard limit).**
Beyond the O(N) *wire* cost, the `0x6520` / `0x6610` group **seal** has an O(N²) *compute* cost at the extreme:
each stanza's per-stanza AAD assembly re-derives the roster / `audience_set_commitment` over the full N-member
member-DID list (`group_roster` + `audience_set_commitment` per stanza, `crates/benten-drop/src/layer_c.rs`), so
sealing an N-recipient group is O(N) stanzas × O(N) per-stanza roster work = **O(N²)**. This is **SENDER-driven**
(only a sender who chooses a large roster pays it — a relay / recipient cannot inflict it), and it is HARD-CAPPED:
the roster ceiling `MAX_LAYER_C_GROUP_RECIPIENTS` (65 535, `u16::MAX`) is enforced with a typed
`LayerCError::RecipientCountExceedsBandWidth` at the single seal-entry choke point `validate_group_roster_len`
(applied to BOTH `seal_group_impl` (`0x6520`) AND `seal_membership_set_group` (`0x6610`) as of R18 C2) — an
over-cap roster typed-rejects; it never panics or unbounded-loops inside the seal. So the worst-case compute is a
sender's own choice, bounded by the typed cap. Not a network-edge DoS (no relay/recipient amplification); the
per-Kind cardinality caps (Atrium ≤32 / DeviceMesh ≤5) keep production rosters far below the ceiling. Accepted at
v1-beta. **Cross-ref:** row 46 above; Compromise index row 46 (O(N) wire-cost); R12 F-11 / R18 C2
(`validate_group_roster_len` typed ceiling); `docs/V1-WIRE-FORMAT-INVENTORY.md §26` (F-11 by-band width note).

**F-33 seal-band-vs-construction-ceiling decoupling note (R20).** The seal-band `u16::MAX` roster ceiling
(`MAX_LAYER_C_GROUP_RECIPIENTS`, enforced by `validate_group_roster_len`) is **decoupled BY DESIGN** from any
MembershipSet-construction ceiling (the `wire_cost_ceiling` / per-Kind cardinality bounds in `benten-membership-set`).
The seal-band cap bounds a **sender-only CPU cost** (the O(N²) per-stanza roster work a sender pays when it *chooses*
a large roster) — it is NOT a construction-time bound on how large a `MembershipSet` may be built, nor a
recipient-/relay-inflictable limit. The two ceilings are independent knobs: the construction-side per-Kind
cardinality caps (Atrium ≤32 / DeviceMesh ≤5 / SingleDevice =1) govern what a well-formed MembershipSet may hold; the
`u16::MAX` seal-band cap is only the hard upper bound on the sender's own choke-point cost. Neither is derived from
the other.

### Compromise #47 — Collaborative-edit-via-re-drop accepted v1-beta trade-off

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** MembershipSet panel (R0.7 §5.2).

Collaborative edits propagate via **re-drop** (re-encrypt-and-redistribute) rather than a shared mutable cipher
object at v1-beta. This is simpler + composes with the content-addressed Drop substrate, at the cost of re-encrypt
overhead on each edit. The shared-mutable-object alternative is the deferred CGKA design class. Accepted at v1-beta.
**Cross-ref:** Compromise #46 (O(N) wire-cost); R0.7 §5.2.

### Compromise #48 — MembershipSet-shape-leak (shared_key compromise fingerprints a generation)

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** MembershipSet panel (R0.7 §5.2).

A `shared_key` compromise reveals that generation's membership-set **shape** (a fingerprint of who is in the set at
that generation). Recovery is **fork-only** rotation (a new generation via FORK, per Inv-20 / Inv-21) — there is no
in-place re-key that hides the prior-generation shape from a holder of the compromised key. Accepted at v1-beta;
the blast-radius is one generation's shape, bounded by rotation discipline. **Cross-ref:** Compromise #52
(no-PCS-against-removed-members / fork-on-kick); `THREAT-MODEL.md` (set shared_key blast-radius rung); R0.7 §5.2.

### Compromise #49 — MembershipSet-member-acting-as-storage-host trust-boundary collapse

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** MembershipSet panel (R0.7 §5.2).

When a set-member is ALSO the storage host for the set's content, the member↔host trust boundary collapses: the host
sees member-visible plaintext — but only content it was **already entitled to** as a member, so no new information
crosses the boundary. The trade-off is that you cannot use a member as an untrusted-storage-host and expect
host-blindness for that member's own entitlements. Accepted and disclosed. **Cross-ref:** Compromise #50
(permanence-stewardship); R0.7 §5.2.

### Compromise #50 — Permanence-stewardship dependency disclosure

**Status.** SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`). **Source.** MembershipSet panel (R0.7 §5.2).

Content permanence depends on **at-least-one-peer stewards the bytes** — Benten makes no centralized-durability
guarantee (P2P-by-design). If every peer holding a content-addressed blob goes offline / garbage-collects it, the
content is unrecoverable. This is disclosed as a substrate boundary: durability is a stewardship property of the
peer mesh, not an engine guarantee. **Cross-ref:** Compromise #55 (GDPR-RTBF, the inverse no-central-delete
property); Compromise #49 (member-as-host); R0.7 §5.2.

### Compromise #51 — Tauri / NAPI-RS marshaling-boundary side-channels

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** MembershipSet panel (R0.7 §5.2).

The Tauri / NAPI-RS FFI marshaling boundary (key material crossing the JS↔Rust line) is a potential timing /
side-channel surface: copies, allocations, and `Debug`-formatting at the boundary are not all constant-time. This
is disclosed, not closed, at v1-beta — the deployment-shape choice (full peer vs embedded webview) determines whether
this boundary is even exercised for key material. Accepted trade-off of the multi-language deployment surface.
**Cross-ref:** Compromise #38 (physical side-channels); Compromise #36 (`secrecy`/`zeroize` best-effort); R0.7 §5.2.

### Compromise #52 — MembershipSet-no-PCS-against-removed-members (fork-on-kick)

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** MembershipSet panel (M-C3 F-FE-5) (R0.7 §5.2).

Removing a member does **NOT** provide post-compromise security against that member for content they already held:
a removed member retains all prior-derived keys and can decrypt any ciphertext they already obtained. Recovery is
**fork-on-kick** — re-key the set via a FORK (new generation) so future content uses a key the removed member never
held. In-set role-downgrade is **subsumed** by the same property (a downgraded member retains prior-derived keys
until the next fork). Accepted at v1-beta; the deferred CGKA class is what would provide true PCS. **Cross-ref:**
Compromise #48 (shape-leak / fork rotation); Compromise #60 (role-transition does not invalidate prior UCANs);
Compromise #35 (compromised-device retro-decrypt); R0.7 §5.2.

### Compromise #54 — Continuous-rotation deferral (+ `AtriumWithRotatingGroupKey` revisit-trigger)

**Status.** ACCEPTED TRADE-OFF (`ATO`). **Source.** MembershipSet panel (N3) (R0.7 §5.2).

Continuous group-key rotation (CGKA-style automatic re-keying on every membership change) is **deferred** at v1-beta;
the set re-keys on FORK, not continuously. The named **post-v1-beta revisit-trigger** is `AtriumWithRotatingGroupKey`
— when continuous rotation is built, it lands additively at reserved codepoints with no wire-break. Accepted at
v1-beta as the simpler fork-on-change model. **Cross-ref:** Compromise #52 (fork-on-kick); Compromise #42 (FS-gap);
R0.7 §5.2 (N3).

### Compromise #55 — GDPR-RTBF honest-architectural-disclosure (P2P-by-design)

**Status.** SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`). **Source.** MembershipSet panel (R0.7 §5.2).

Right-to-be-forgotten (GDPR Art. 17) is **architecturally constrained** by P2P-by-design: there is no central
authority that can guarantee deletion of content already replicated across an arbitrary peer mesh. The apps-layer
remedy is **crypto-shredding** — destroy the keys and leave the (now-undecryptable) ciphertext; this approximates
erasure without a central delete. This is disclosed honestly as a substrate boundary, not over-claimed as compliant
erasure. **Cross-ref:** Compromise #50 (permanence-stewardship, the durability inverse); R0.7 §5.2.

### Compromise #56 — Journalist per-message forward-secrecy deferral (SEPARATE design class)

**Status.** SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`). **Source.** MembershipSet panel (MINT confirmed; R1-Q-8)
(R0.7 §3.3 / §5.2).

Per-message forward-secrecy (the journalist / high-risk-source threat class, where each message must be
independently FS so a single key-compromise does not unravel a conversation) is a **SEPARATE design class** kept
sharply distinct from #42 (HPKE long-term-sk non-FS) and #52 (fork-on-kick PCS) per the R1-Q-8 confirmed mint. It is
**deferred post-v1-beta** (the CGKA/MLS-PQ bracket). Disclosed as its own row precisely so the audit does not fold it
into #42/#52 and under-count the threat surface. **Cross-ref:** Compromise #42 (HPKE FS-gap); Compromise #52
(fork-on-kick); R0.7 §3.3 (FS honest-disclosure).

### Compromise #57 — RestrictedScopeSet / grant immutability honest-disclosure

**Status.** SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`). **Source.** MembershipSet panel (R0.7 §5.2).

A `RestrictedScopeSet` grant is **immutable once issued**: narrowing or re-scoping requires re-issue (a new grant),
not mutation of the existing one. This is a substrate-guarantee disclosure — it absorbs / cross-links the #62
revocation-reach property (an issued grant's already-derived keys stay valid; revocation cuts future serves only).
Disclosed so operators understand grant lifecycle is issue-and-replace, not edit-in-place. **Cross-ref:** Compromise
#62 (revocation reach); Compromise #64 (the same SGD eventual-consistency disclosure class); R0.7 §5.2.

### Compromise #58 — Audit-log insider-correlation (per-recipient-unlinkability is network-observer-only)

**Status.** COMPOSITION-HAZARD HONEST DISCLOSURE (`CHD`). **Source.** MembershipSet panel (M-C2-B-3 + P5; m-7)
(R0.7 §3.8 / §5.2).

An admin (or any insider) holding the audit-log + the `members_table` **CAN correlate members** — the
per-recipient-unlinkability property is **network-observer-only** (m-7), NOT admin-hidden. The Layer-C / `0x6610`
group-AAD blinding hides a **high-entropy** roster from a **network observer / untrusted relay** — the
`membership_set_id_commitment` is K_Set-keyed, but the `audience_set_commitment` is **unkeyed** and therefore
itself guess-confirmable for a **low-entropy / guessable** roster (Compromise #43 / Row D-36) — while a
member-or-admin who holds `K_Set` + the member list recomputes the commitments and sees the correlation. This is a composition-hazard honest disclosure: the unlinkability claim is scoped, not
absolute. The **threshold-admin opt-in** (no single admin sees the full audit-log) closes the insider vector for
deployments that adopt it. This row is the load-bearing #58 the `THREAT-MODEL.md` network-observer-only scope
cross-links — the boundary that keeps the unlinkability claim honest, not over-claimed. **Cross-ref:**
`THREAT-MODEL.md` (network-observer-only unlinkability scoping); Compromise #43 (envelope-metadata leakage);
Compromise #61 (gossip-topic blinding); R0.7 §3.8 (m-7).

### Compromise #59 — Delivery-token / KEM-key-confirmation abuse-control residual

**Status.** SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`). **Source.** MembershipSet / Layer-C abuse-control panel (R0.7
§3.11 / §5.2). Detail section added at R18 (F-DISC-1 ledger-coherence — the index row + cross-refs pre-existed; the
body is authored here from R0.7 §5.2 to match the F-DISC-1 disclosure-coherence contract).

The Sealed-Sender delivery-token admit path provides **KEM-key-confirmation** (a recipient's delivery token binds
the recipient's own KEM key, so a non-recipient cannot mint an admittable token), but it does **NOT** by itself
bound the RATE at which a *legitimate* recipient over-issues delivery tokens. A recipient who over-issues delivery
tokens can re-admit spam through its own admit gate — the KEM-key-confirmation property is re-scoped to this abuse
surface: it confirms *who* may issue, not *how many*. **Residual (the compromise):** recipient-side
over-issuance re-admits spam. Mitigated by default-conservative per-token rate-limits + the delivery token's own
UCAN `nbf`/`exp` + revocation substrate (the token-binding AAD carries NO `coarse_epoch` — freshness rides the
UCAN window + the `jti`-keyed nonce-cache, §3.10, never a time-bucket). Accepted at v1-beta; the bounded
abuse-surface is the residual. **Cross-ref:** Compromise #63 (Sealed-Sender abuse-control trade-off); Compromise
#30 (audit-gated PQ window); §3.10 (nonce-cache spec); R0.7 §3.11 / §5.2.

### Compromise #60 — Role/generation transitions do not invalidate prior-issued UCANs (tight-`exp` bound)

**Status.** SUBSTRATE-GUARANTEE DISCLOSURE (`SGD`). **Source.** MembershipSet RBAC / Layer-D grant panel (R0.7
§3.4 / §5.2). Detail section added at R18 (F-DISC-1 ledger-coherence — the index row + cross-refs pre-existed; the
body is authored here from R0.7 §5.2 to match the F-DISC-1 disclosure-coherence contract).

A MembershipSet role-transition (a role downgrade / re-assignment) or a generation bump does **NOT** synchronously
invalidate UCANs already issued under the prior role/generation: a UCAN is valid until its own `exp` (or explicit
revocation), so a member downgraded at time T can still exercise a grant minted before T until that grant expires.
This is the grant-axis analogue of the #52 fork-on-kick property (a removed/downgraded member retains
prior-derived keys until the next fork) and the #64/O-4 revocation-propagation-lag (a stale grant is exercisable
during a partition). **Residual (the compromise):** a window — bounded by the grant's `exp` — in which a prior-role
grant remains exercisable after the role/generation transition. Mitigated by the **tight-`exp` default mandate**
(§3.4): short delivery-token / grant `exp` bounds how long any single prior-role grant is exercisable at all, plus
explicit revocation for the immediate case. Accepted at v1-beta; closing it synchronously (cross-device
grant-invalidation on every role change) is out of scope for the P2P model. **Cross-ref:** Compromise #52
(fork-on-kick — the keying-axis analogue); Compromise #64 (cross-device nonce-window) + its O-4
revocation-propagation-lag sub-clause; §3.4 (tight-`exp` default mandate); R0.7 §3.4 / §5.2.

### Compromise #61 — MembershipSet-fingerprint-leak via iroh-gossip topic (CLOSED by HMAC-blinded topic)

**Status.** COMPOSITION-HAZARD HONEST DISCLOSURE (`CHD`); CLOSED. **Source.** MembershipSet panel (M-C3) (R0.7
§3.9 / §5.2).

A naive iroh-gossip topic derived from the raw membership-set id would have leaked a membership-set **fingerprint**
to any gossip participant — a composition hazard between the transport (gossip topic) and the set-identity. **CLOSED**
by the P2 D6 **HMAC-blinded gossip topic**: the topic is `blake3::keyed_hash(K_Set, set_id ‖ BE(generation))` (the
unlabelled §3.9 construction), so set-identifying material is never published in the clear — only set-members holding
`K_Set` can derive the topic. This is the SAME blinding construction the `0x6610` group-AAD
`membership_set_id_commitment` uses (the labelled §3.10 / AAD construction), keeping the anti-fingerprint posture
consistent across the gossip + AAD surfaces. Honest scope: identity-HIDING, not unlinkability (a static set's topic
recurs; full per-send unlinkability = U25 v1-GM-reserve). **Cross-ref:** Compromise #58 (insider-correlation
boundary); Compromise #43 (envelope-metadata); `CRYPTO-CODEPOINTS.md` (gossip-topic vs AAD-commitment distinction);
R0.7 §3.9 (gossip topic) / §3.3 (`0x6610` AAD blinding).

### Test-debt note — `f_audit_1` arm-(a) model-shape (F-full R6 R1 finding F-14)

**Status.** TEST-DEBT (honest disclosure); NOT a security gap — the property
IS enforced in the substrate. The MembershipSet audit-emit invariant (Inv-20
clause-h / F-AUDIT-1: an audit event MUST flow through the ENFORCED engine-API
WRITE path so the `(actor_cid, handler_cid, capability_grant_cid)` attribution
triple is SET; a bare backend `put_node` leaves the triple `None` and is NOT
the audit path) is pinned by two complementary arms in
`crates/benten-engine/tests/f_audit_1_enforced_write_path_attribution_triple.rs`:

- **Arm (a)** — the `enforced_write_*` arms drive the **model-shape**
  `benten_membership_set::audit::emit_audit_event_via_engine` /
  `emit_audit_event_via_bare_put` helpers (both
  `#[cfg(any(test, feature = "testing"))]`-gated OFF the frozen v1-beta public
  surface at the R6 tail fold-in F11 — zero production callers), which model the
  enforced-vs-bare distinction by constructing the `AuditEmitResult` directly (the enforced helper
  fills the full `Some(...)` triple + advances the chain; the bare helper leaves
  all three `None`). This pins the SHAPE of the property but is a model, not a
  drive of the real engine.
- **Arm (b)** — the `engine_enforced_path_*` arm drives the **REAL** `Engine`:
  `audit_sequence()` advances on an enforced grant WRITE but NOT on a dedup-replay,
  proving the enforced-vs-unenforced distinction is live in the real substrate the
  W6 audit chain rides on.

**Test-debt:** arm-(a)'s model-shape should be upgraded to drive the real
membership-set → engine enforced-WRITE audit-emit wiring end-to-end (so the
attribution triple is populated by the genuine production emit handler, not the
model helper), closing the gap that arm-(b) already covers for the engine half.
Tracked as a follow-up at the membership-set audit build-out (couples to the
governance/audit graph-native exit criterion R0.5 §9.1-6). No exploit at v1-beta:
the real-engine arm-(b) already proves the enforced path; arm-(a) is a redundant
model that should be hardened to real-drive for defense-in-depth, not a missing
control.

## Per-Node AEAD wrap layer — rebinding-attack-prevention (G-CORE-3d / #1301)

**Section landed at Phase-4-Meta-Core G-CORE-3d wave (per R0.8
multitenant-r1.4-2 corrective + Spike G/H + R3 ratification of the
SubgraphSpec / `RATIFIED-sharing-and-confidentiality-2026-05-21.md` R2
two-CID + per-Node AEAD contract).** Companion to baked-in #5
(crypto-agility / PQ-default reframe), baked-in #15 (v1-gate
encryption-as-confidentiality), and baked-in #18 (per-Node AEAD as the
confidentiality half of the Principal primitive — the structural arm
that capability-gating cannot provide on an untrusted host).

### What rebinding-attack-prevention is

Three load-bearing AEAD-layer defenses ride on the per-Node AEAD wrap
+ two-CID mapping that G-CORE-3d ships:

1. **AAD-binds-plaintext-CID (whole-content arm).** The AAD passed to
   ChaCha20-Poly1305 at seal time is
   `aad_whole_content(plaintext_cid) = b"benten-aead:whole:" || plaintext_cid_bytes`.
   At decrypt time the recipient reconstructs the AAD from the
   envelope's `plaintext_cid` field. If an attacker takes a valid
   ciphertext for plaintext-CID *P* and mounts it under plaintext-CID
   *Q* (the **rebinding attack** — relocating a ciphertext under a
   different plaintext identity), the AEAD authenticator fails because
   the reconstructed AAD (binding *Q*) does not match the AAD bound at
   seal time (binding *P*).
2. **AAD-binds-(plaintext-CID, chunk-index, total_chunks) (per-chunk
   arm for Nodes ≥ 64 KiB).** Per `§1.A.FROZEN item 15(g)` the
   per-chunk AEAD uses
   `aad_per_chunk(plaintext_cid, chunk_index, total_chunks) =
   b"benten-aead:chunk:" || plaintext_cid_bytes || chunk_index_u64_be
   || total_chunks_u32_be`. Shuffling chunk-N's ciphertext to
   index-M (the **cross-chunk rebinding attack** — silently
   reordering content within a Node) fails because the reconstructed
   AAD (binding `chunk_index=M`) doesn't match the seal-time AAD
   (binding `chunk_index=N`). Truncating an N-chunk ciphertext to N'
   chunks (the **cross-chunk truncation attack** — silently dropping
   content from a Node) ALSO fails because the seal-time AAD
   committed to `total_chunks=N` but the truncated-presentation
   recipient reconstructs the AAD with `total_chunks=N'`, and
   per-chunk AEAD authentication fails at every chunk boundary.
   **R6 R1 fix-pass (2026-05-24):** the `total_chunks` segment was
   added at R6 R1, retracting the prior G-CORE-9 R1 triage Fork-1
   disposition that had deferred this defense to G-COMP-1; see
   V1-FROZEN-INTERFACE-DEFERRED.md Row D-15 revision-history.
3. **Two-CID mapping integrity (defense-in-depth at the storage layer).**
   The mapping table row `d:<did>:m:<plaintext_cid> → ciphertext_cid`
   is validated structurally: the envelope's `plaintext_cid` field
   MUST match the mapping-claimed plaintext_cid; mismatch surfaces as
   `TwoCidMapError::IntegrityMismatch`. A tampered mapping that
   redirects `plaintext_a → ciphertext_b` is caught because *B*'s
   envelope carries `plaintext_cid = B`, not `A`.

### Inv-11 strengthening (cross-DID leak invariant)

Inv-11 (the cross-DID-no-leak invariant from G-CORE-1 #989) is
STRENGTHENED by the per-Node AEAD layer per the C1+C2 composition
(multitenant-r1-5):

- **C1 (authority isolation, G-CORE-1):** `WriteContext::namespace_did`
  partitions storage keys (`d:<did>:n:<cid>` + label_index +
  property_index + change_event fan-out + iter_node_cids). Cross-DID
  reads structurally miss at the key-prefix layer; the partition
  isolation fires BEFORE any AEAD work, surfacing `NotFound` rather
  than `AeadAuthenticationFailed` (which would imply the cross-DID
  caller saw the ciphertext bytes — a confidentiality leak).
- **C2 (confidentiality isolation, G-CORE-3d):** per-Node AEAD with
  K(N) derived from `K_principal` (Spike-E Interpretation-B
  path-tagged derivation) means even an attacker who bypasses the
  partition layer (e.g. via raw redb file read) cannot decrypt without
  the foreign DID's `K_principal`. The classical-half hybrid floor
  (X25519 + ChaCha20-Poly1305 / NCC-audited) holds even when the PQC
  half is unaudited (see Compromise 30).

The composition: partition isolation is the first line; per-Node AEAD
with `K_principal`-rooted key derivation is the second. The §4-A
cross-wave test set (R3-W3 partition-before-crypto pin family) pins
both layers + their composition order.

### Key-derivation source — G-CORE-3e production HKDF-SHA256 swap-in

The G-CORE-3e wave swaps the BLAKE3-of-plaintext-CID test-seam K(N)
derivation for the production HKDF-SHA256 structural-KDF substrate
from `benten_crypto_suite::structural_kdf` (Spike-E Interpretation-B
path-tagged derivation per the RATIFIED-S&C §R-key-derivation
contract). The new derivation chain in
`crates/benten-graph/src/redb_backend.rs::derive_test_seam_key_from_cid_with_namespace`:

```text
K_principal = BLAKE3-keyed-hash(domain_tag, info = namespace_did.as_bytes())
K(root)     = derive_root(K_principal, root_cid = plaintext_cid.as_bytes())
            = HKDF-SHA256(K_principal, info = "root" || plaintext_cid)
K(N)        = K(root)   // single-Node walk at this seam; the
                        // multi-edge derive_step chain through a
                        // SubgraphSpec walk lands at the future
                        // subgraph-walk wire-up
```

The seal-side `put_node_with_context` AEAD-wrap call + the unseal-
side `read_decrypt_inner` AEAD-unwrap call BOTH route through the
same `derive_test_seam_key_from_cid_with_namespace(namespace_did,
plaintext_cid)` helper — byte-for-byte agreement is load-bearing
(AEAD authenticate-on-decrypt fails closed if the keys diverge).
The unseal side recovers the namespace_did from the
TWO_CID_MAP_TABLE row's key prefix via the new
`two_cid_lookup_with_namespace` lookup (`d:<did>:m:<plaintext_cid>`
shape).

**K_principal seam.** The wave-3e per-deployment `K_principal`
material is deterministically derived from the namespace_did via
BLAKE3 keyed-hash over a stable domain tag — this lets the wave-3e
per-recipient seal/unseal path produce stable keys without the
K_principal storage seam landing first (the K_principal-per-DID
secret-material backend is the #989 / #1301 substrate). The
production wire-up reads K_principal from the per-DID secret store
at the same boundary; the helper signature
`(namespace_did, plaintext_cid)` is the stable seam. Named at
`docs/future/phase-4-backlog.md §3.10` for the per-DID secret-store
upgrade.

**⚠️ Confidentiality limit at this wave.** The deterministic
`K_principal` synthesis at this wave —
`blake3::keyed_hash(&K_PRINCIPAL_DOMAIN_KEY, did_bytes)` over a
publicly-known 32-byte domain-tag constant + the publicly-known
`namespace_did` `Cid` bytes — means **any party holding
`(namespace_did, ciphertext_blob)` can derive `K(N)` and decrypt**.
This is acceptable ONLY because the wave-3e use-case is keeping
the substrate-shape stable for the production `K_principal`-store
swap-in at the next wave; do **NOT** rely on the wave-3e
confidentiality envelope for any data not also protected by
namespace-isolation at the storage backend. The function name
retains the `derive_test_seam_key_from_cid_with_namespace` "test
seam" hint precisely to mark this wave-state on every caller. Named
carry destination: `docs/future/phase-4-backlog.md §3.10`
(K_principal-per-DID secret-material backend) — the swap-in
replaces only the `K_principal` synthesis step; the function
signature + the AEAD-wrap layer + the per-chunk size are all
stable.

**This limit is now a NUMBERED Compromise (#65; R13 F-07)** so it is
registry-tracked + auto-swept by the `f_disc_1` compromise-disclosure
catch-net. The companion `docs/THREAT-MODEL.md` §1 untrusted-host tier
row + honesty note match this disclosure: per CLAUDE.md baked-in #18
the confidentiality half of the Principal primitive is DEFERRED
(#1301 / D-64), so per-Node AEAD is a publicly-derivable-`K_principal`
STAND-IN at v1-beta and an untrusted *storage host* CAN read the
partition plaintext — the LIVE protection is the AUTHORITY half
(capability / namespace isolation, cooperating-engine-only). The
relay-facing "sees plaintext = NO" (Tier-1) is a DIFFERENT, live
mechanism (Layer-C encrypt-to-recipient), not this stand-in.

### Per-chunk-AEAD chunk size = `IROH_BLOCK_SIZE` (16 KiB)

Per `§1.A.FROZEN item 15(g)` the per-chunk AEAD chunk size MUST equal
`iroh-blobs::IROH_BLOCK_SIZE` (16 KiB). Spike H+1.2 validated this:
divergent chunk size = double-chunking overhead at iroh's wire
transport (iroh would re-chunk Benten's AEAD chunks to its own block
size, doubling the per-chunk overhead). The constant lives at
`benten_crypto_suite::aead::IROH_BLOCK_SIZE` (re-exported at
`benten_graph::aead_wrap::IROH_BLOCK_SIZE` for the storage layer pin).
A drift here is caught by the
`tf3d_per_chunk_aead_iroh_block_size::tf3d_iroh_block_size_is_16_kib_exactly`
regression pin.

### Mechanism: ChaCha20-Poly1305

The AEAD primitive is **ChaCha20-Poly1305** (RFC 8439) per CLAUDE.md
baked-in #5 crypto-agility refinement. Dispatched via
`benten_crypto_suite::aead::wrap` / `::unwrap` over the
codepoint-tagged `AeadKeyMaterial` (MLKEM768-X25519-hybrid `0x647a` v1-beta default;
classical-only X25519 `0x6400` downgrade arm; both feed the same
ChaCha20-Poly1305 bulk layer). The integration crate is the ONLY
crypto-primitive call site (crypto-agility-contract:6). Never
hardcoded key/nonce/tag sizes outside the cipher-suite dispatch arm
(the nonce length of 12 B is algorithm-parameter-fixed for
ChaCha20-Poly1305, not a CLAUDE.md #5 "no-hardcoded-sizes" violation).
Note: the secret-size scanner intentionally EXEMPTS ephemeral `[u8; 32]`
values (e.g. random ephemeral / nonce-seed material) — these are
fixed-width ephemeral bytes, not agility-bearing algorithm key/sig sizes,
so a literal `32` there is not a "no-hardcoded-sizes" violation either.

### Cross-refs

- `00-implementation-plan.md` §1.A.FROZEN item 15 + item 15(g) (the
  two-CID + per-chunk-AEAD frozen contract).
- `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
  R2 (per-chunk-AEAD chunk-size = `IROH_BLOCK_SIZE` ratification).
- Spike G + Spike H + Spike H+1.2 sandbox findings.
- `crates/benten-graph/src/aead_wrap.rs` (the storage-layer AEAD-wrap
  glue) + `crates/benten-graph/src/two_cid_map.rs` (the typed mapping
  error envelope).
- `crates/benten-graph/tests/tf3d_*.rs` (the 15 R3 RED-PHASE pins
  un-ignored at G-CORE-3d landing).

## UCAN-gated iroh-blobs custom-ALPN handler — per-request validation (G-CORE-3e / #1301)

**Section landed at Phase-4-Meta-Core G-CORE-3e wave (per
`RATIFIED-sharing-and-confidentiality-2026-05-21.md` R2 online-share
contract + Flavor B per-request UCAN check finding).** Terminal wave
for the G-CORE-3 online path.

### What it defends

The wave-3e UCAN-gated iroh-blobs custom-ALPN handler
(`crates/benten-sync/src/ucan_blobs_protocol.rs::UcanBlobsHandler`)
validates the requester's `AuthorizationGrant` on EVERY incoming
request BEFORE dispatching to upstream
`iroh_blobs::provider::handle_connection`. This is the Flavor B
contract per Spike A2: a previously-validated grant is NEVER trusted
across requests; every request re-runs the full validation pipeline.

### Six-arm validation pipeline (fail-closed at first reject)

The handler runs validation arms IN ORDER, fail-closed at the first
rejection. The typed reject IS the defense per §R2 + §R3 +
audience-binding:

1. **Unresolvable peer-DID short-circuit** (security-r1-2 sentinel).
   If the grant's UCAN references an unresolvable peer-DID, return
   typed `UnresolvedDeny` BEFORE any cryptographic work. We can't
   validate the binding-sig if we can't even know who's asking.
2. **Audience binding** (Spike A2 zero-conversion identity-cast).
   The connection's verified `EndpointId` (iroh `EndpointId` IS
   `ed25519_dalek::VerifyingKey` per Spike A2 — no parse, no
   conversion) MUST equal the grant's `audience_pubkey` bytes
   exactly. Mismatch → typed `UcanAudienceMismatch`. This is the
   F-1 unauthorised-requester + A-2 audience-substitution defense.
3. **UCAN time-bounded validity** (replay-attack defense). `nbf <=
   now < exp` per injected clock. Past-`exp` → `UcanExpired`;
   future-`nbf` → `UcanNotYetValid`.
4. **Revocation observance** (§R6 reach). If the grant CID is in
   the handler's revocation store → typed `GrantRevoked`. Already-
   decrypted plaintext at the recipient side is NOT revoked — that's
   the documented cryptographic limit at §R6.
5. **Binding-sig verification** (delegates to wave-3b
   `AuthorizationGrant::verify_binding` — covers A-1 stolen-UCAN-
   without-keys, A-2 stolen-keys-without-UCAN, A-3 wrong-audience-
   swap uniformly).
6. **Scope check** (F-2 arm). The requested `ciphertext_hash` MUST
   be in the granted `RestrictedScope`'s `roots` allowlist (per
   `with_hashes` constructor). Out-of-scope → typed `NotInScope`.

ONLY after all six arms pass does the handler dispatch to
`iroh_blobs::provider::handle_connection` (the production wire-up
arm; the wave-3e test seam increments
`UcanBlobsHandler::dispatch_count_for_test`).

### Zero-conversion plumbing — iroh `EndpointId` IS `VerifyingKey`

Per Spike A2: iroh's `EndpointId` (= `iroh_base::PublicKey`) is a
byte-identical wrapper around `CompressedEdwardsY` — the same
32-byte Ed25519 public-key encoding as `ed25519_dalek::VerifyingKey`.
The wave-3e handler exploits this identity at the audience-binding
arm: the connection's verified `EndpointId` bytes ARE the requester's
UCAN audience pubkey bytes; no parsing, no `VerifyingKey::from_bytes`
round-trip in the hot path. The identity-cast helpers
`verifying_key_to_endpoint_id` + `endpoint_id_to_verifying_key`
codify the contract; the wave-3e pin
`tf3e_endpoint_id_round_trips_through_verifying_key_byte_identical`
enforces it.

### Revocation reach (§R6) — Compromise #62 detail (RE-POINTED from in-tree #31 per BR-2)

**Code anchors (grep-discoverable per pim-13 §3.12 audit-trail-cite discipline; L14-MIN-3 close at R6-FP-D):** the future-serve-cut assertion at `crates/benten-engine/tests/resume_with_revoked_grant_denies.rs` + `crates/benten-sync/tests/tf3e_replay_attack_ucan_expired.rs`; the forever-valid-once-distributed Drop bundle property documented at `crates/benten-drop/tests/tf3f_revocation_reach_forever_valid_documented.rs`; the `E_UCAN_BLOBS_REQUEST_REJECTED` server-side gate ErrorCode at `crates/benten-errors/src/lib.rs::ErrorCode::UcanBlobsRequestRejected` (the typed mitigation arm); the offline-Drop asymmetry section "Revocation reach — online-pull vs offline-Drop asymmetry (G-CORE-3f)" below (grep the section title to locate at HEAD per §3.5b HARDENED point 3 / §3.6j cite-grep-verify discipline; line number omitted to avoid future drift).

UCAN revocation cuts FUTURE serves only — already-decrypted
plaintext at the recipient side remains decryptable (cryptographic
limit). Mitigation: tight `nbf`/`exp` windows + key rotation per
the §R6 contract. The wave-3e
`tf3e_revoked_grant_yields_typed_revoked` pin asserts the future-
serve cut; the documented limit on already-derived plaintext is
recorded here.

### What is NOT in this wave

- **The actual `iroh_blobs::provider::handle_connection` call.** The
  brief explicitly says "reuses" the upstream call; landing
  iroh-blobs as a workspace dependency is OUT-OF-SCOPE for G-CORE-3e.
  The wave proves the per-request UCAN validation + scope check +
  identity-cast plumbing all work end-to-end against the test seam;
  the production wire-up at a future iroh-blobs-integration wave
  swaps the test-instrumented dispatch counter for the real
  upstream call at the named `serve_request_for_test` boundary in
  `ucan_blobs_protocol.rs`.
- **Real RotationLog peer-resolution.** The wave-3e `unresolved_peer`
  flag is a sentinel; the production wire-up swaps it for a real
  `benten-id::RotationLog::resolve` call at the validation pipeline's
  Arm 1.

### Cross-refs

- `crates/benten-sync/src/ucan_blobs_protocol.rs` (the per-request
  handler) + `crates/benten-sync/src/two_cid_store.rs` (the
  ciphertext-bytes store wrapping the two-CID mapping, with the
  named iroh-blobs `FsStore` swap-point at `ciphertext_bytes`).
- `crates/benten-sync/tests/tf3e_*.rs` (the 4 R3 wave-3e RED-PHASE
  pin files, 11 pins total un-ignored at G-CORE-3e landing).
- `crates/benten-caps/src/authorization_grant.rs` (wave-3b
  `verify_binding` substrate, extended with wave-3e `audience_pubkey`
  + `scope` fields + `issue_for_test` / `issue_with_nbf_for_test` /
  `issue_with_unresolved_peer_for_test` / `malformed_for_test`
  helpers).
- `00-implementation-plan.md` §3 G-CORE-3e wave definition.
- `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R2 (online
  share + Flavor B per-request UCAN check) + §R3 (audience-binding
  property) + §R6 (revocation reach).

## Revocation reach — online-pull vs offline-Drop asymmetry (G-CORE-3f)

Per `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
§R6, the revocation surface for the Sharing & Confidentiality stack has
an inherent **asymmetry** between the two delivery modes:

### Online-pull (Mode 1, G-CORE-3e ALPN custom-handler)

When a recipient pulls live from a publisher over the iroh ALPN custom
handler, the publisher's cap-policy validates the carried UCAN at every
request. Revocation **cuts future serves**: a previously-issued UCAN
that the issuer has revoked is refused on the next inbound request, and
all subsequent requests for that grant fail typed. The
`benten-id::RotationLog`-backed UCAN revocation surface (shipped in
Phase 3) gives publishers a write-once durable record consulted at the
ALPN boundary.

### Offline-Drop (Mode 2, `benten-drop::DropBundle`, G-CORE-3f)

When the recipient consumes a Drop bundle from the filesystem (no
network; no live publisher; sealed CBOR-on-disk artifact), there is no
opportunity for the issuer to refuse the request — **already-derived
keys remain decryptable**. The key material is already in the
recipient's hands (carried inside the bundle's
`AuthorizationGrant.key_material`); revocation cannot retroactively
take it back. The cryptographic property:

> **Drop bundles are forever-valid once distributed.** A Drop bundle
> whose embedded UCAN is revoked AFTER distribution still decrypts at
> the recipient's end. This is not a defect; it is the cryptographic
> reality of any sealed offline artifact.

**Mitigations (operator discipline):**

- **Tight `nbf`/`exp` on issued UCANs.** A short-lived UCAN naturally
  bounds the forever-valid window: a Drop bundle whose UCAN has
  `exp = issue_time + 24h` is decryptable indefinitely BUT readers can
  observe the `exp` as a freshness hint when re-using the carried
  material (the engine surfaces the `exp` at consume time).
- **Periodic key rotation.** Rotating the per-DID wrapped-key seed at
  a regular cadence means each Drop bundle's `key_material` covers
  only a bounded slice of the principal's content history. A
  compromised Drop bundle exposes only the content sealed under the
  rotation epoch's keys; future content under rotated keys remains
  confidential.
- **Don't distribute Drop bundles you'd want to revoke later.** The
  online-pull (Mode 1) path is the right shape when revocation
  semantics are load-bearing; the Drop format is for share-and-forget
  artifacts (recipe collections, time-frozen reports, archival
  snapshots) where revocation is not the trust foundation.

### Construction sites

- `crates/benten-drop/src/bundle.rs` —
  `DropBundle::consume_offline` is the load-bearing surface; the
  function returns recovered plaintext IFF the envelope-sig + grant
  binding-sig + per-Node AEAD tags all verify. There is NO
  revocation-store consultation step (this is the §R6 reality made
  observable in code).
- `crates/benten-drop/tests/tf3f_revocation_reach_forever_valid_documented.rs`
  pins the property end-to-end:
  - `tf3f_drop_bundle_decrypts_after_ucan_revocation_forever_valid` —
    publishes + revokes + asserts the Drop still decrypts.
  - `tf3f_security_posture_md_documents_revocation_reach_section` —
    asserts this section is present + names the load-bearing claims.
  - `tf3f_security_posture_md_names_online_vs_offline_revocation_asymmetry`
    — asserts BOTH halves of the asymmetry are named.

### Cross-refs

- `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
  §R6 "Revocation reach" (the ratification record).
- `.addl/phase-4-meta/00-implementation-plan.md` §3 G-CORE-3 def
  input-constraints refinement #6 L341 (the three sendme deployment
  modes; this asymmetry follows from mode 2 being offline-by-construction).
- Online-pull revocation surface lives in
  `crates/benten-id/src/did_rotation.rs` (`RotationLog`) +
  `crates/benten-caps/src/grant_backed.rs` (the UCAN-gated cap policy
  consulted at the ALPN boundary).

---

### Compromise #67 — First-contact / TOFU DID-authenticity bootstrap (GAP-KDB Shape-B residual)

**Status.** SUBSTRATE-GUARANTEE HONEST DISCLOSURE (`SGD`); OPEN residual — NOT eliminated (the honest
bind-once boundary). **Source.** GAP-KDB Shape-B identity-model council (design
`.addl/phase-4-meta/GAP-KDB-B-DESIGN-R1.md` §8 / R1 §5; disposition per the R0.7 §5.2 registry conventions).

Shape-B — the content-addressed `did:benten` key-set + `Did::resolve_kem` + the `RecipientBinding`
sole-constructor typestate + Inv-23 — closes *key-substitution-given-a-known-DID*: an active attacker can no
longer swap the recipient KEM key under a `did:benten` a sender already holds, because doing so requires a
BLAKE3-256 2nd-preimage over the canonical DAG-CBOR key-set (infeasible). It does **NOT** shut
*DID-authenticity-at-first-contact*: whoever controls the channel where a sender **first learns**
"Alice ↔ `did:benten:…`" can hand them their own self-consistent `did:benten`, which then resolves and verifies
cleanly.

Shape-B therefore **reduces the** confidentiality trust window from **continuous** (swap the address-book KEM
key at any time) to **bind-once** (substitute the DID only at the single moment of first contact) — the
identical posture to Signal safety numbers, MLS, and PGP fingerprints, all of which share this exact residual.
Authenticating that initial DID↔principal binding is **out-of-band** and the user's responsibility; Benten
provides no PKI / CA for it and **cannot authenticate the first** contact. The bare-`did:key` / Shape-A fallback
has the *same* residual — no reviewer's push for A can frame it as "eliminating" this.

**Not eliminated — reduced.** The honest statement is: *post*-first-contact key substitution is infeasible
(a 2nd-preimage); *first-contact* DID-authenticity is a bind-once, TOFU boundary the user bootstraps out-of-band.
**Cross-ref:** Inv-23 (`INVARIANT-COVERAGE.md`); `SECURITY-PROOFS.md` §4.1 (the recipient-key premise now cites
the binding rather than assuming an honest address book); `THREAT-MODEL.md` (recipient-key closure + the
DISTINCT seal-path revocation-reach residual, Compromise #62 — a separate open residual, NOT this one); design
§8 / R1 §5.
