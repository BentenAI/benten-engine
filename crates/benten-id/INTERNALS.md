# `benten-id` — Crate Internals

Plain-English deep-dive for the 9th workspace crate. Identity primitives: Ed25519 keypairs, `did:key` DIDs, UCAN delegation chains, Verifiable Credentials, DID rotation attestations, and signed device-DID capability envelopes. Read-only audit — no compile / no cargo / no claims about CI state.

---

## 1. What this crate does

This is the cryptographic identity foundation Phase 3 grew up around. The CapabilityPolicy commitment (CLAUDE.md item #7) named UCAN as "one backend"; this crate ships the in-memory UCAN primitive itself plus everything the durable backend in `benten-caps` needs upstream of it. Three concentric rings of surface:

- **Inner ring (G14-A1):** Ed25519 keypair with secret-bytes hygiene, `did:key` (W3C method-key spec) encode/decode, UCAN claim envelope + chain-walk validation with `nbf`/`exp`/attenuation/audience-binding enforcement.
- **Middle ring (G14-A2):** Verifiable Credentials (hand-rolled DAG-CBOR over W3C VC v1.1-INSPIRED fields), DID rotation attestations, signed device-DID capability envelopes (`DeviceAttestation`), DID revocation primitives.
- **Outer ring (G14-A2 + G16-D wave-6b):** the `Acceptor` runtime gate for device-DID attestations — freshness window + nonce-store replay defense + signature verification + revocation check + parent-DID expected-issuer pin. This is the per CLAUDE.md item #18 "per-plugin DID" surface viewed through the multi-device-sync lens (criterion 16). Compromise #23's signed envelope V2 with session-nonce replay defense lives here.

The crate is the load-bearing identity primitive layer underneath three downstream consumers: `benten-caps` (UCAN durable backend at `benten_caps::backends::ucan`), `benten-sync` (peer-discovery + handshake; iroh endpoint identity derives from the same Ed25519 bytes), and engine-level typed-CALL dispatch for keypair / UCAN / VC operations.

---

## 2. Dependency chain

**Upstream (what `benten-id` pulls in):**
- `ed25519-dalek` — the actual Ed25519 signing/verifying primitive. Re-exports `Signature`.
- `zeroize` — `Zeroize + ZeroizeOnDrop` derives on `SecretKey`. Hand-rolled redacted Debug instead of `secrecy` (Cargo.toml note explains: dep-surface minimal, raw-bytes test pin contract).
- `subtle` — `ConstantTimeEq` for the `ct_signature_eq` helper. Wrapped at one site (`crates/benten-id/src/ucan.rs::ct_signature_eq`) and called from every security-decision compare across `ucan.rs` / `did_rotation.rs` / `device_attestation.rs`. The grep audit `ucan_chain_walk_constant_time_comparison_audit` polices uniformity.
- `getrandom` + `rand_core` — OS CSPRNG path. Both direct deps so the source-grep tests can pin the call site.
- `serde` + `serde_bytes` + `serde_ipld_dagcbor` — canonical-bytes envelope encoding (RFC 8949 deterministic encoding via sorted-keys). Same DAG-CBOR shape the rest of the engine uses.
- `bs58` — base58btc encoder for the `did:key:z` body.
- `thiserror` — typed errors.

Dev-only: `proptest` (4 proptests), `hex` (test bytes), `toml` (dep-edge audit test reads Cargo.toml itself), `serde_json` (currently unused-looking — confirm before pruning).

**Downstream (what depends on `benten-id`):**
- `benten-caps` — UCAN backend (`crates/benten-caps/src/backends/ucan.rs`) imports `Ucan`, `Did`, `Capability`, `validate_chain_*` functions, plus `DeviceRevocation` for the durable revocation-set construction.
- `benten-sync` — `transport.rs` + `peer_discovery.rs` + `handshake.rs` use `Keypair`, `Did`, `PublicKey`, `Signature`, `Capability`, `Ucan` for the iroh endpoint + on-the-wire handshake frames.
- `benten-engine` — typed-CALL dispatch (`typed_call_dispatch.rs::keypair_generate` + `keypair_from_seed`) surfaces the raw key bytes via the Phase 3 typed-CALL op set.
- `bindings/napi` — Node bindings re-expose keypair / DID / UCAN through the TS DSL.

**Forbidden (enforced as a test):** `crates/benten-id/tests/dependency_edges.rs` reads its own Cargo.toml and asserts none of `benten-graph`, `benten-engine`, `benten-eval`, `benten-caps`, `benten-ivm`, `benten-sync`, `benten-dsl-compiler` appear in `[dependencies]`. This is the `arch-r1-10` pin baked into the test suite.

---

## 3. Files in `src/`

Eight modules. Roughly in dependency order within the crate:

### `lib.rs`
Crate root. Declares the 8 sub-modules, re-exports the 7 error types at top-level for ergonomic `use benten_id::UcanError;` shapes. Module-level docstring narrates G14-A1 vs G14-A2 scope split, plus three CLAUDE.md baked-in commitments the crate is sensitive to (#3 code-as-graph, #17 deployment shapes, arch-r1-10 dependency-edge). `#![deny(missing_docs)] + #![forbid(unsafe_code)]` at the crate level.

### `errors.rs`
Seven typed-error enums, one per public surface module. The interesting shape decisions:

- `SeedImportError` — five DISTINCT variants for the envelope-import path (`ShortInput { got, min }` / `LongInput { got, max }` / `EnvelopeMalformed` / `UnknownVersion { version }` / `UnknownAlg { alg }` / `InvalidSecret`). Each failure mode is observable from outside, no `dyn Error` / `String` blackboxing. Drives the proptest at `prop_keypair_from_seed_bytes_arbitrary_input_no_panic`.
- `UcanError` — 11 variants covering time-window (NotYetValid / Expired), audience binding (AudienceMismatch), chain integrity (ChainLinkBroken / BadSignature / EmptyChain), attenuation (AttenuationViolated for both authority AND time-window axes), durable-backend interactions (IssuerKeypairSuperseded / IssuerDeviceRevoked / DeviceEnvelopeViolated), and the typed-CALL leaf-claim gate (CapabilityNotGranted).
- `DeviceAttestationError` — 8 variants. The `IncompatibleWithRuntime { detail: &'static str }` variant is the closure for `br-r4-r1-4` / `br-r4-r2-3` MAJOR — browser-target + `runs_sandbox=true` rejects at construction time with catalog code `E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME`. The whole enum has a `code()` method returning a stable `&'static str` for the ErrorCode catalog.
- `VcError`, `DidError`, `DidRotationError`, `MultiSigError` — narrower surfaces.

### `keypair.rs`
The Ed25519 primitive. Three load-bearing decisions baked into the type design:

1. **`SecretKey` newtype with `Zeroize + ZeroizeOnDrop` derives + no Clone + redacted Debug.** The `crypto-blocker-1` BLOCKER contract. Pinned by three tests — one source-grep checks the `ZeroizeOnDrop` derive line preceding `pub struct SecretKey`; one source-grep checks no `Clone` in the same derive line; one runtime check formats Debug output and grep-asserts no hex appears + "REDACTED" appears.
2. **`Keypair::generate` pinned to `rand_core::OsRng`** (which routes to `getrandom`). NOT a deterministic seed. The 1000-case proptest `prop_keypair_generate_distinct_across_1k_calls` + the 2000-call aggregate-set distinctness test would catch a CSPRNG regression.
3. **`Keypair::from_seed_bytes` consumes a DAG-CBOR envelope** of shape `{version: u8, alg: "Ed25519", secret_bytes: Bytes(32)}`. Round-trip via `export_seed_envelope`. Two-layer pre-check (length bounds → CBOR decoder) so pathological inputs reject fast with typed variants before hitting the CBOR machine.

Notable test-only escape hatches (`#[doc(hidden)]`):
- `bytes_for_test` + `bytes_ptr_for_test` on `SecretKey` — test-only raw-byte accessors that the in-tree tests use for the zeroize source-grep + redaction round-trip.
- `secret_bytes_unprotected` on `Keypair` — production-shape alias of the test accessor. Documented use sites: `typed_call_dispatch.rs::keypair_generate` + `keypair_from_seed` (typed-CALL output schema today surfaces raw bytes in `Value::Bytes`; phase-3-backlog §2.5 (e) tracks the `Value::SensitiveBytes` extension). Also used in `benten-sync` transport + peer-discovery for iroh keypair construction. **The doc comment warns the caller is responsible for wrapping in `Zeroizing` if the value lives past the immediate dispatch.**

The DAG-CBOR envelope schema is intentionally version-tagged. `SeedImportError::UnknownVersion` rejects forward-incompatible bytes loudly instead of silently mis-parsing.

### `did.rs`
W3C did-method-key encode/decode. Three constants pin the spec compliance: `ED25519_MULTICODEC = [0xed, 0x01]`, `DID_KEY_PREFIX = "did:key:z"`, multibase prefix `z` = base58btc. Encoded shape: `"did:key:z" + base58btc(0xed01 || <32 pubkey bytes>)`.

The `Did` newtype wraps the resolved string and implements `Serialize`/`Deserialize` as `#[serde(transparent)]` — round-trips the string form, **does NOT validate on deserialize**. The docstring is explicit: callers needing validate-on-deserialize call `Did::resolve` explicitly to surface a typed `DidError`. Used by `benten-sync`'s `HandshakeFrame` wire format where both peer-DID + device-DID land at the wire-format level (`net-blocker-4`).

`Did::from_string_unchecked` is the post-deserialize "trust the string" entry — caller asserts they verified. This shape is consistent with `from_seed_bytes`'s typed-error stance: the validating path is its own function, the constructor doesn't silently swallow validation.

The W3C-vector test (`did_key_resolves_against_w3c_test_vectors`) carries 3 pinned hex pubkey → DID-string fixtures: the RFC-8032 example pubkey, a sister keypair, and the all-zero pubkey degenerate vector. Plus the fail-closed `did_key_rejects_wrong_multicodec_per_w3c_spec` that constructs a `did:key:z…` body with `0x00 0x00` multicodec and asserts `UnknownMulticodec(0x00, 0x00)` specifically.

### `ucan.rs`
The chain-walk validator. 705 lines, the largest module. Public surface:

- **`Capability { resource, ability }`** — the unit of authority. `resource` = e.g. `/zone/posts` or `host:sandbox:exec`; `ability` = e.g. `read` / `write` / `*`.
- **`Ucan` envelope** + **`UcanClaims`** payload. Signature is over the DAG-CBOR canonical bytes of claims (the `signature` field is excluded from the signed bytes).
- **`UcanBuilder`** — `issuer`/`audience`/`capability`/`not_before`/`expiry`/`proof`/`sign`. The builder doesn't enforce issuer-DID matches keypair-DID at build time so adversarial-fixture tests can be authored.
- **Six chain-walk validators:** `validate_chain_no_time_check` / `validate_chain_at` / `validate_chain_for_audience` / `validate_chain_for_capability` / `validate_chain_with_rotation_log` / `validate_chain_with_device_revocations` / `validate_chain_with_attestations`. Composition via the private `validate_chain_inner(chain, now, expected_audience)`.

The chain-walk does five things per link inside `validate_chain_inner`:

1. **Time-window check** at every link (the `crypto-blocker-2` BLOCKER — `nbf` and `exp` checked at every link, not just the leaf; renew-the-leaf-forever attack defense).
2. **Signature verification** against the link's issuer DID resolved via `Did::resolve`. Failure surfaces as `BadSignature { link_index }`.
3. **Chain-link integrity**: parent's `aud` MUST equal child's `iss`. Ordering is leaf-first (`chain[0]` = leaf, `chain[idx+1]` = parent). `ChainLinkBroken` on mismatch.
4. **Authority attenuation**: every child capability MUST be subsumed by some parent capability per the `caps_match_or_subsume` rule (exact match, parent-wildcard-ability, parent-path-prefix-resource).
5. **Time-window narrowing** (G16-B-B-rest closure of `cap-r4-2`(a)/(b) + `tcc-r1-5` R3-A): child's `[nbf, exp]` MUST be a subset of every ancestor's window. Backdating (`child.nbf < parent.nbf`) and forward-dating (`child.exp > parent.exp`) both reject with `AttenuationViolated` carrying time-window-shaped diagnostic strings. Absent bounds treated as unbounded.

`ct_signature_eq(a, b)` wraps `subtle::ConstantTimeEq`. Made `pub(crate)` (formerly private) at g14-a2-mr-2 fix-pass so device-attestation and DID-rotation use the same helper at security-decision sites. Comparisons that go through this helper are tagged with `// const-time-eq` markers — the audit test greps for those plus a forbidden-`==` blacklist.

The `validate_chain_for_capability` entry is the typed-CALL-layer integration: composes audience-bind + chain-walk + leaf-claim check using the SAME subsume relation the internal attenuation walk uses (exposed as `capability_satisfies_requirement` for engine-side queries — single source of truth, not a second relation that could drift).

### `vc.rs`
Verifiable Credential issuance + verification. The single most important docstring fact: **"W3C VC v1.1-INSPIRED field shape over DAG-CBOR + Ed25519. NOT wire-format-compatible with external W3C JSON-LD VC consumers."** Dates are `u64` epoch seconds (not ISO 8601); encoding is DAG-CBOR (not JSON-LD); `proof: Vec<u8>` is a flat 64-byte Ed25519 sig (not the LDP `Ed25519Signature2020` envelope).

The wire-interop layer (full `ssi` integration with JSON-LD / Linked-Data-Proofs) is deferred to G14-B per `docs/future/phase-3-backlog.md §2.1-followup`. The vc.rs module docstring carries an explicit Q3 DISAGREE-WITH-EXPLANATION rationale per HARD RULE rule-12 disposition (c) for not pulling `ssi` in at G14-A2 — minimal dep surface and the wire-format-compat is the layer that genuinely needs `ssi`, not the in-RAM verify path.

Surface: `Credential` envelope + `CredentialClaims` payload (W3C field shape — `@context` / `type` array starting with `VerifiableCredential` / `issuer` / `issuanceDate` / `expirationDate` / `credentialSubject` / `credentialStatus`). `CredentialSubject` is intentionally narrow (subject-DID + single `(claim_name, claim_value)` pair) — sufficient for the must-pass test fleet, not a "carry arbitrary JSON-LD" surface.

Four verifier entry points: `verify` / `verify_at` (`expirationDate` gate) / `verify_with_registry` (revocation registry lookup) / `verify_in_trust_domain` (issuer allow-list) / `verify_bytes_in_trust_domain` (raw-input parsing-then-verify; drives the 10,000-case malformed-input proptest).

Two in-RAM helpers: `TrustDomain` (HashSet allow-list of issuer DIDs) and `RevocationRegistry` (Mutex<HashSet<String>>). Both are explicitly tagged "G14-B replaces with durable backing."

### `multi_sig.rs`
The `MultiSigSurface` trait + `Ed25519SingleKey` default impl. Five-method trait surface (`sign` / `verify` / `threshold` / `participants` + the `Signature` / `Error` associated types). Phase 3 ships only the single-key impl with `threshold() = 1`, `participants() = 1`. The `ThresholdMultiSig` placeholder demonstrates the trait extension point (non-sealed); its bodies return `MultiSigError::PostPhase3` per D-PHASE-3-24.

Two architectural pins live here:

- `multi_sig_surface_trait_signature_pinned` — `const _: fn() = || { … }` block doing compile-time signature-shape assertion. Trait drift = compile failure.
- `multi_sig_surface_no_recovery_protocol_specific_behavior_in_phase_3` — source-grep audit: the NON-COMMENT surface of multi_sig.rs MUST NOT name `Shamir` / `social_recovery` / `TPM` / `MLS` / `HardwareEscrow`. Comments are stripped first (per `crypto-r4-r1-minor-2`) so the file's docstring CAN reference the deferred protocols without false-positive. The `cag-5` + D-PHASE-3-24 commitment: identity-recovery protocol choice is deferred to post-Phase-3 v1-assessment-window.

### `did_rotation.rs`
Old-DID → New-DID rotation events. `RotationAttestation` carries `previous_did`, `next_did`, `superseded_at`, and a 64-byte Ed25519 signature **by the OLD keypair** (proving rotation was authorized by whoever held the old secret).

`rotate_keypair(did, old_kp, new_kp, superseded_at)` is the constructor. Rejects with `PreviousDidMismatch` if the supplied `did` ≠ old keypair's `did:key` (defends against caller bugs).

`RotationLog` is an in-RAM `Vec<RotationAttestation>` consulted by the chain-walker's `validate_chain_with_rotation_log` (rejects with `IssuerKeypairSuperseded` for any UCAN whose issuer DID has been rotated). G14-B replaces the in-RAM log with a durable backing. `is_superseded` uses `ct_signature_eq` per the uniformity rule, even though DIDs are public.

The docstring is explicit about the "logical DID stability under rotation" framing: the OLD `did:key` string is the long-lived audience-binding identifier; what rotates is the keypair underneath. The rotation attestation lets verifiers walk forward to find the new keypair. The test `did_rotate_keypair_preserves_did_under_canonical_bytes` pins that the OLD DID's canonical bytes don't change across rotation.

### `device_attestation.rs`
The Phase-3 multi-device-sync (criterion 16) surface plus the runtime-target enforcement layer. Eight public types, the load-bearing flow:

- **`CapabilityEnvelope`** — 4-dimension declaration: `runs_sandbox: bool`, `holds_zones: ZoneScope` (`Full` / `CacheOnly` / `Specific(Vec<String>)`), `online_uptime: UptimePolicy` (`AlwaysOn` / `SessionBounded`), `runs_atrium_peer: bool`. The thin-client minimum-capability envelope (browser tab per CLAUDE.md baked-in #17) is `runs_sandbox=false, holds_zones=CacheOnly, uptime=SessionBounded, runs_atrium_peer=false` and is preset via `issue_for_browser_target`.
- **`DeviceAttestation`** — `device_did` + `parent_did` + `envelope` + 32-byte nonce + `issued_at` epoch seconds + 64-byte parent-signed signature. The signature is over DAG-CBOR canonical bytes of `(device_did, parent_did, envelope, nonce, issued_at)` — signature field is excluded from the signed input (self-reference hygiene). The `signature: Vec<u8>` field's public visibility is load-bearing: the `acceptor_rejects_attestation_with_forged_signature` test mutates `signature[0] ^= 0x01` to drive the bad-signature negative pin; canonical-bytes round-trip also touches it.
- **Four issuance constructors:** `issue` (zero-init `issued_at`), `issue_at` (caller-controlled epoch), `issue_for_browser_target` (auto-asserts minimum envelope), `issue_with_runtime_check` (rejects `Browser` target + `runs_sandbox=true` OR `runs_atrium_peer=true` at construction time with `IncompatibleWithRuntime` carrying catalog code `E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME`), `issue_with_authority` (rejects with `EnvelopeWidening` if device claims wider authority than the supplied parent envelope — `cap-r4-7` closure).
- **`envelope_widens` matrix** — exhaustive 3×3 over `(parent, device) holds_zones` per g14-a2-mr-6 fix-pass. The matrix is the entire enumerated-fork-tree, not implicit defaults: `Specific(_)` → `CacheOnly` is narrowing (OK); `Specific(p)` → `Specific(c)` widens iff any child zone is outside parent; `Specific(_)` → `Full` always widens. Pinned by the `envelope_widens_zone_scope_matrix` test that exercises all 9 cases.
- **`Acceptor`** — Compromise #23 runtime gate. Five steps in `accept_at(attestation, now)`:
  1. Expected-parent pin (if configured via `with_parent_lookup`).
  2. Revocation check (constant-time eq over device_did).
  3. Freshness gate (`now - issued_at <= window`).
  4. **Signature verification against the parent_did's resolved pubkey** (added at g14-a2-mr-1; before this, a forged sig with valid nonce/freshness/parent_did string would pass).
  5. Nonce-store replay defense (`(parent_did, nonce)` tuple insertion; replay = duplicate = `NonceReplay`).
- **`DeviceRevocation`** — signed by parent, carries `device_did` + `parent_did` + `RevocationReason` (`DeviceLoss` / `Compromise` / `Decommissioned`). The chain-walker's `validate_chain_with_device_revocations` rejects UCANs signed by a revoked device with `IssuerDeviceRevoked`.

The `generate_fresh_nonce` helper composes 4 × `OsRng::next_u64().to_le_bytes()` instead of a `[0u8; 32]` zero-init buffer — works around a CodeQL false-positive that pattern-matches the literal as "hardcoded nonce" even when overwritten by `fill_bytes` on the very next line. Documented in the doc comment.

---

## 4. Public API surface

Grouped by intent rather than module, since the public surface composes across modules:

**Keypair lifecycle (G14-A1).** `Keypair::generate` (CSPRNG path) / `Keypair::from_seed_bytes` (envelope import) / `Keypair::export_seed_envelope` (envelope export) / `Keypair::sign` / `Keypair::public_key` / `Keypair::secret_bytes_unprotected` (caveat: caller responsibility for zeroize wrapping).

**DID encoding (G14-A1).** `Did::from_public_key` / `Did::resolve` (typed-error reverse decode) / `Did::from_string_unchecked` (deserialize boundary trust) / `PublicKey::to_did` (convenience).

**UCAN issuance + chain validation (G14-A1).** `Ucan::builder` → chained `issuer`/`audience`/`capability`/`not_before`/`expiry`/`proof`/`sign`. Validation entry points form a small matrix: `(time? × audience? × capability? × revocation? × rotation? × device-attestation?)`. The composed entry `validate_chain_for_capability(chain, expected_audience, required, now)` is the engine-typed-CALL surface; the narrower entries exist for the cases where one gate is checked at a time. `capability_satisfies_requirement` exposes the internal subsume relation for engine-side queries.

**Verifiable Credentials (G14-A2).** `Credential::builder` → `sign`. Verify-side: `verify` / `verify_at` (expiry gate) / `verify_with_registry` (revocation) / `verify_in_trust_domain` (issuer allow-list) / `verify_bytes_in_trust_domain` (untrusted-input path). `TrustDomain::new` + `RevocationRegistry::new` + `RevocationRegistry::revoke`.

**DID rotation (G14-A2).** `rotate_keypair(did, old_kp, new_kp, superseded_at)` → `RotationAttestation`. `RotationLog::from_entries` + `RotationLog::append` + `RotationLog::is_superseded`. `is_did_superseded` free function alias.

**Device-DID attestation (G14-A2 + G16-D wave-6b).** Four issuance entry points (`issue` / `issue_at` / `issue_for_browser_target` / `issue_with_runtime_check` / `issue_with_authority`). Canonical-bytes round-trip via `canonical_bytes` + `from_canonical_bytes`. `verify_signature_with(parent_pk)`. `Acceptor::new` + `Acceptor::new_with_revocations` + `Acceptor::with_parent_lookup` + `Acceptor::accept_at` / `accept`. `DeviceRevocation::issue` + `verify_signature_with`. The chain-walker hooks: `validate_chain_with_attestations` (envelope-violation gate) + `validate_chain_with_device_revocations` (revocation gate).

**Multi-sig extension surface (G14-A2 + post-Phase-3).** `MultiSigSurface` trait + `Ed25519SingleKey` default impl + `ThresholdMultiSig` placeholder. Open trait — downstream crates can implement.

---

## 5. Tests inventory

14 test files; 2,328 LOC of test surface — substantial for a 1,460-LOC source surface (~1.6× ratio). The crate is heavily property-tested and source-grep-pinned.

- **`dependency_edges.rs` (35 LOC)** — 1 test. Reads its own Cargo.toml and asserts none of 7 forbidden workspace crates appear in `[dependencies]`. The `arch-r1-10` pin.
- **`keypair.rs` (259 LOC)** — 7 tests. Round-trip + zeroize source-grep + no-Clone source-grep + redaction round-trip (two variants — one via `SecretKeyDebugProbe` mimic, one via the real `Keypair::Debug` path) + clone-doesn't-widen-lifetime + OsRng source-grep.
- **`keypair_seed.rs` (171 LOC)** — 6 tests + 1 proptest. Round-trip / short-input / long-input / corrupted / unknown-version / source-grep-no-tracing / DAG-CBOR canonical-bytes stability across export → import → re-export. Proptest: `prop_keypair_from_seed_bytes_arbitrary_input_no_panic` (2,000 cases, arbitrary 0-512 bytes).
- **`did_key.rs` (222 LOC)** — 5 tests. Deterministic-from-pubkey / multibase-prefix-z / multicodec-0xed01 + the W3C vectors with three pinned hex pubkey → DID-string fixtures + fail-closed wrong-multicodec.
- **`prop_did_key.rs` (30 LOC)** — 10,000-case round-trip byte-identity proptest. Any 32-byte seed → SigningKey → pubkey → `did:key` encode → decode → byte-identical pubkey.
- **`prop_keypair_generate.rs` (47 LOC)** — 1,000-case distinctness proptest + 2,000-call aggregate-set distinctness test. Defends against CSPRNG cycling regressions.
- **`ucan.rs` (284 LOC)** — 10 tests. Empty chain rejects / single-token round-trip / attenuation rejects overgrant / nbf rejection / exp rejection / chain-walk propagates expiry through attenuation / audience-binding rejects cross-atrium replay / constant-time-eq source-grep audit (the most far-reaching grep — walks 5 modules across the crate's source surface) / ucan_chain_revocation_propagates RED-PHASE (`#[ignore]`'d for §2.1-followup `ssi` re-eval).
- **`prop_ucan_attenuation.rs` (249 LOC)** — 1,000-case (downsized from 2,000 for MSRV 1.95 wall-clock timing — documented in the proptest config) on both axes: authority-attenuation never widens AND time-window-narrowing never widens. Strategy: 25% of cases force a widening (leaf claims `/zone/admin:write` while parent grants `/zone/posts:read`, OR leaf widens both nbf+exp); 75% are uniform chains. Validates that `validate_chain_at`'s outcome agrees with the structural shape.
- **`vc.rs` (147 LOC)** — 5 tests. Round-trip / expiration / revocation / trust-domain / tamper detection.
- **`prop_vc_arbitrary.rs` (22 LOC)** — 10,000-case malformed-input no-panic proptest.
- **`multi_sig.rs` (127 LOC)** — 4 tests. Trait-signature compile-time pin (`const _: fn() = || …`) + Ed25519SingleKey round-trip + ThresholdMultiSig PostPhase3 stub / recovery-protocol source-grep with comment-stripping.
- **`did_rotation.rs` (113 LOC)** — 4 tests. Emit-attestation / propagation-to-backend RED-PHASE-ignored / superseded-rejected / canonical-bytes-stable.
- **`device_attestation.rs` (569 LOC)** — 12 tests. The deepest surface: round-trip / envelope-consumed-at-chain-walk / freshness-window / nonce-replay / parent-revocation / revoked-device-cannot-sign / envelope-attenuation / widening-rejected / self-re-attestation / runtime-recheck / browser-auto-assert / browser-construction-time-rejection + RED-PHASE-ignored `ucan_delegation_to_browser_target_for_sandbox_handler_rejected_at_chain_construction_not_invocation` (G14-B + G14-C wires) + forged-signature-rejection + 9-cell zone-scope-matrix.
- **`graph_encoded.rs` (53 LOC)** — 4 RED-PHASE tests, all `#[ignore]`'d with rationale strings routing to G14-B + G14-C (graph-Node persistence shape requires `benten_core::Node` / `benten_core::Edge` reach, which is downstream; `arch-r1-10` prevents this crate from depending on `benten-graph`).

---

## 6. Benches inventory

No `benches/` directory present. The crate is correctness-pinned; cryptographic performance benching (if needed) would live downstream where the surface is actually wired into a request path — `benten-engine` typed-CALL dispatch for keypair / UCAN, `benten-sync` handshake critical path.

---

## 7. Thin-engine + composable-graph philosophy check

The crate sits clean. Identity is foundational by nature — it can't compose from other primitives, it IS the primitive other things compose against. Specific observations:

**Well-respected surfaces.**

- The `arch-r1-10` test (`dependency_edges.rs`) is the single most important architectural pin in the crate. Identity primitives upstream of everything; the test bakes the invariant into the build.
- **`capability_satisfies_requirement` exposed as the single subsume relation.** Engine-side query "does this chain grant `required`?" uses the SAME relation the chain-walker uses internally. This is the explicit single-source-of-truth shape — there is no parallel implementation in `benten-engine` that could drift from the canonical relation here.
- **DAG-CBOR canonical-bytes everywhere.** Seed envelope, UCAN payload, VC claims, device attestation sig input, rotation attestation sig input — all use `serde_ipld_dagcbor::to_vec`. Same encoding the rest of the engine uses for content addressing. CLAUDE.md item #5 (BLAKE3 + DAG-CBOR + CIDv1) is consistently observed even though nothing in this crate computes a CID — when these envelopes flow into the engine, the bytes-to-CID step is downstream and stable.
- **Typed-error discipline.** Every public function returns a `Result<_, ConcreteErrorEnum>`. No `Box<dyn Error>`, no `String` errors. Each rejection mode is observable from outside. The 11-variant `UcanError` is the largest enum; every variant is hit by at least one test pin.
- **Cap-attenuation rule is exhaustive on both axes.** Authority axis (`caps_match_or_subsume` — exact / wildcard-ability / path-prefix-resource) AND time-window axis (subset on both nbf and exp). Both have their own AttenuationViolated diagnostic shape but join the same error variant family — consistent semantics.
- **`envelope_widens` made fully exhaustive across all 9 zone-scope cases at g14-a2-mr-6 fix-pass.** No implicit fallthrough — every `(parent, device) holds_zones` case has its own arm with rationale.

**Constant-time discipline uniformity.** `ct_signature_eq` is wrapped at one site and used at every security-decision compare across 5 modules. The `ucan_chain_walk_constant_time_comparison_audit` grep test enumerates a forbidden-pattern list (`signature ==`, `audience ==`, `proof_cid ==`, `device_did ==`, `parent_did ==`, `nonce ==`, etc.) and walks all 5 modules' source. A new comparison added in any module hits the audit. DIDs and nonces are public but the UNIFORMITY contract makes future contributors hit the rule by reflex.

**Worth flagging — potential pluggability friction (CLAUDE.md item #19 perspective).**

- **Ed25519 hardcoded throughout.** The DAG-CBOR envelope tags `alg: "Ed25519"` and the import path returns `UnknownAlg` for anything else. The `SeedImportError::InvalidSecret` variant is explicitly named as "reserved for future algorithm extensions" — but the actual extension point (where would X25519 / BLS / post-quantum land?) is not abstracted. `MultiSigSurface` is the cleanest extension trait, but the Phase-3 default carries Ed25519 in its type signature (`type Signature = ed25519_dalek::Signature`). Engine-level extension (#19) for alternate signature schemes would need to either implement `MultiSigSurface` with a different `Signature` associated type (clean) OR thread a generic over `Keypair` / `Did` / UCAN (heavy lift across the public surface). Per CLAUDE.md item #19's compile-time-trust framing, the heavy lift is acceptable when an extension actually lands; flagging here so the post-quantum / hardware-key future isn't a surprise rewrite. Not a defect today — the Phase-3 commitment was a single algorithm. The `Cargo.toml`'s "FORBIDDEN" comment in `dependency_edges.rs` does NOT forbid alternate-signature-scheme crates, so the extension door is open.

- **`SecretKey::bytes_for_test` + `secret_bytes_unprotected` are documented escape hatches.** The first is `#[doc(hidden)]` and explicitly test-only; the second is `#[must_use]` production-named but the docstring narrates the unprotected contract (caller wraps in `Zeroizing` if held past dispatch). Two production use sites are named: `typed_call_dispatch.rs` for keypair / from_seed (phase-3-backlog §2.5 (e) carries the `Value::SensitiveBytes` extension that closes this gap) and `benten-sync` transport / peer-discovery (iroh keypair construction). The Cargo.toml + the production accessor + the backlog-pin form a coherent chain — gap acknowledged with a named destination, not silently widened.

**No identity-layer/sync-layer coupling drift.** The `benten-sync` consumers grep cleanly: `transport.rs` + `peer_discovery.rs` + `handshake.rs` import `Keypair`, `Did`, `PublicKey`, `Signature`, `Capability`, `Ucan` from this crate. The reverse direction is forbidden by `arch-r1-10`. The handshake-frame wire format `Did` carrying via `Serialize` impl is identity-layer-correct — the wire format IS the identity's externalized name, not a sync-layer concern that bled into identity.

**No cap-attenuation logic that should live in benten-caps.** The chain-walk lives here because it has to — `validate_chain_*` is the in-memory primitive, and `benten-caps`'s `UCANBackend<B>` composes ON TOP of it for the durable / revocation-set / atrium-row paths. The split is clean: this crate ships the cryptographic envelope + chain-walk math; `benten-caps` ships the storage + grant-tracking + atrium-merge integration. Not coupled in the wrong direction.

**Device-attestation envelope V2 internals don't leak.** `DeviceAttestation`'s `signature` field is publicly accessible — required for test-side bit-flipping — but the `canonical_bytes` function uses an internal `SigInput<'a>` struct that excludes the sig field. Construction goes through `issue_*` functions only; there is no public constructor that lets callers set the signature directly. The wire-format integrity contract holds.

**One small thing to flag for retrospective:** the `validate_chain_no_time_check` docstring is unusually candid: "We deliberately accept the ambiguity at this entry point and direct callers to `validate_chain_at` for production." This is honest documentation of an entry point that's primarily for the "no-time-check" basic test pin. Production code paths in `benten-caps`'s UCANBackend use `validate_chain_at`. Fine as-is; flagging because a reviewer encountering this entry point cold might wonder if it's a footgun. The docstring routes them correctly.

---

## 8. Phase 3.5 + Phase 4 expectations

Several knowable forward-looking surfaces touch this crate:

**Phase 3.5 (UCAN revocation observance closure — already shipped via PR #109 / Track B).** Per `docs/future/phase-3-backlog.md §13.11`, the root cause traced through the durable `benten-caps` UCANBackend (scope-string namespace mismatch in `revokeCapability` invocation), not this crate. But `validate_chain_with_device_revocations` + `validate_chain_with_rotation_log` here are the upstream surfaces that the durable observance composes against. If §2.1-followup's `ssi`-integration re-evaluation lands, the chain-walker may grow a `validate_chain_with_revocations(chain, revocation_set)` entry — there's currently a doc-comment reference in `ucan.rs:32-36` mentioning the symbol but no body. The RED-PHASE-`#[ignore]`'d tests in `ucan.rs::ucan_chain_revocation_propagates` + `did_rotation.rs::did_rotation_propagates_revocation_to_ucan_backend` are pinned to that wave with rationale.

**Phase 4 plugin manifest schema (CLAUDE.md item #18).** The per-plugin DID + UCAN model lives on this crate's surface. Each plugin gets a `Did` (via `Keypair::generate` + `PublicKey::to_did`). The plugin manifest's `requires` + `shares` halves will be signed by the plugin author — the signing primitive IS `Keypair::sign` over canonical-bytes of the manifest. Manifest verification at install time uses `verify` flow. The user-as-root → install-time-manifest → runtime-delegation chain (CLAUDE.md #18 layers a/b/c) routes:
- (a) user-as-root: `Keypair` for the user identity, `Did` is the chain anchor.
- (b) install-time manifest: signed by plugin author keypair; verified against author's DID at install (`vc.rs` shape is well-positioned — VC's `issuer` + `credentialSubject` model maps clean to "plugin author claims X about plugin").
- (c) runtime delegation: UCAN chain — `Ucan::builder` + `validate_chain_for_capability`. The `caps_match_or_subsume` subsume relation is what the runtime-delegation gate will evaluate.

The `MultiSigSurface` trait is positioned for the v1-assessment-window identity-recovery protocol choice (CLAUDE.md #15) — the trait extension point exists and the `ThresholdMultiSig` placeholder demonstrates downstream crates can implement it. The cag-5 + D-PHASE-3-24 commitment is to defer the concrete protocol; the trait surface is shape-stable.

**Admin UI v0 (Phase 4).** UCAN delegation paths from user → plugin will exercise the typed-CALL `ucan_validate_chain` entry that composes here via `validate_chain_for_capability`. The defense-in-depth that `validate_chain_for_capability` adds (leaf-`att` check beyond audience + chain + time) is what closes the "structurally-sound chain that names wrong cap" hole — load-bearing as the admin UI grows the per-action-grant surface.

**Device-DID cross-machine sync (already shipped).** Compromise #23 + `DeviceAttestationEnvelope V2` shipped at G16-D wave-6b PR #163. The `Acceptor` runtime gate is the boundary; Phase 4 admin UI exposes `set_local_device_attestation` + `set_acceptor` on `AtriumHandle` (per phase-3-backlog §2136 retrospective comment about the `AtriumHandle` surface lacking a single doc-reference). The capability-envelope `runs_sandbox / holds_zones / online_uptime / runs_atrium_peer` shape is forward-stable.

---

## 9. Open questions / unresolved internals

A handful of things worth surfacing for retrospective. Nothing here is a hidden defect — these are observations that might generate follow-up work or merit explicit named-destination entries:

1. **`serde_json` listed in `[dev-dependencies]` (Cargo.toml line 58).** Grep across `tests/*.rs` doesn't surface a `serde_json::` use anywhere. The other dev-deps (`proptest`, `hex`, `toml`) are all consumed visibly. Possible dead dev-dep; possible JIT use I missed. Worth a quick `cargo machete` or grep before any cargo-clean pass.

2. **`SeedImportError::InvalidSecret` is reserved but unreachable today.** ed25519-dalek 2.x accepts any 32 bytes as a SigningKey seed (no rejection at construction). The variant exists to keep the typed-error surface stable across future algorithm extensions. Documented in source comment + variant docstring. Not a defect; flagging because a coverage tool might mark it as dead-arm without context.

3. **`UcanClaims::aud` is a `String` not a typed `Did`.** Audience is compared via `ct_signature_eq` over UTF-8 bytes in `validate_chain_inner` — works correctly. The reason for the looser type: `UcanBuilder::audience(impl Into<String>)` accepts free-form audience strings (some UCAN ecosystems use non-DID audiences — e.g. URLs). The cost: a typo'd audience string at issuance silently produces an "audience mismatch" at verify time instead of a build-time error. The compensation: `audience_did(&Did)` convenience method exists for callers using DIDs. Coherent design choice; flagging because the surface could plausibly be tightened later without breaking the wire format.

4. **`validate_chain_no_time_check` is ambiguous re. `nbf` handling** (the docstring even says so). For "no time check" the implementation passes `None` for now, which skips the nbf/exp gates entirely — not "treat the chain as if all tokens are valid forever," but rather "don't apply the time gate." If a chain has a token with nbf set in the future, no-time-check accepts it. The docstring directs callers to `validate_chain_at` for production paths. The function is primarily used by the basic-validation test pin. Possible future cleanup: rename to `validate_chain_no_time_check_test_only` and gate behind `#[cfg(any(test, debug_assertions))]`? Not a defect; documenting the entry-point semantic ambiguity for retrospective.

5. **`AttestationKind` enum has one variant.** `AttestationKind::SupersededBy` is the only kind today. The enum keeps shape across future rotation-event types (e.g. multi-sig-rotation, threshold-revoke). `kind()` accessor returns a constant. Fine; flagging because clippy + similar tools sometimes flag single-variant enums as redundant — the architectural reason (post-Phase-3 extension surface) is encoded in the source comment.

6. **`benten-id` is the load-bearing crypto surface but has no compile-time guard against `#[cfg(target_arch = "wasm32")]` thinning.** The whole crate compiles on wasm32 today (verified by `benten-sync`'s native-only gating + `benten-caps`'s `ucan` backend gating, both of which gate AT THE CONSUMER not here). If a future contributor adds something like `tokio` or filesystem reach into this crate, the deployment-shape contract from CLAUDE.md #17 silently breaks. The `dependency_edges.rs` test catches workspace-crate adds but not external-crate native-only adds. A defensive measure could be a CI check that runs `cargo check --target wasm32-unknown-unknown -p benten-id`. Not a defect today; flagging because the crate's load-bearing position makes any wasm32-incompatible regression high-blast-radius.

7. **The `secret_bytes_unprotected` named uses are typed-CALL surface + iroh keypair construction.** Both have phase-3-backlog destinations (§2.5 (e) for the `Value::SensitiveBytes` extension; iroh's expectation of raw bytes is upstream). The accessor is `#[must_use]` and the docstring is explicit about caller responsibility. Worth confirming on the next phase-close audit pass that no NEW callsites accumulated post-G14-A2 without their own destination entry.
