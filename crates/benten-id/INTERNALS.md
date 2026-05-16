# `benten-id` — Crate Internals

Plain-English deep-dive for the 9th workspace crate. Identity primitives: Ed25519 keypairs, `did:key` DIDs, UCAN delegation chains, Verifiable Credentials, DID rotation attestations, signed device-DID capability envelopes, **plugin-DID mint/store (G24-D)**, and the **sibling `GrantReader` trait with CID-keyed companion (G27-C)**. Read-only audit — no compile / no cargo / no claims about CI state.

State as of HEAD `8141b94` (2026-05-14). Substantive content added since the prior revision: Phase-3-close maturation (RotationLog HLC-monotonic-strict + verbatim-replay defense at G24-D-FP-2) + Phase-4-Foundation G24-D plugin-DID surface + G27-C sibling GrantReader trait.

---

## 1. What this crate does

The cryptographic identity foundation Phase 3 grew up around, **extended by Phase-4-Foundation G24-D (plugin-DID mint/store)** and **G27-C (sibling GrantReader trait at the identity layer)**. The CapabilityPolicy commitment (CLAUDE.md item #7) names UCAN as "one backend"; this crate ships the in-memory UCAN primitive itself plus everything the durable backend in `benten-caps` needs upstream of it.

Four concentric rings of surface:

- **Inner ring (G14-A1):** Ed25519 keypair with secret-bytes hygiene, `did:key` (W3C method-key spec) encode/decode, UCAN claim envelope + chain-walk validation with `nbf`/`exp`/attenuation/audience-binding enforcement.
- **Middle ring (G14-A2):** Verifiable Credentials (hand-rolled DAG-CBOR over W3C VC v1.1-INSPIRED fields), DID rotation attestations, signed device-DID capability envelopes (`DeviceAttestation`), DID revocation primitives.
- **Outer ring (G14-A2 + G16-D wave-6b):** the `Acceptor` runtime gate for device-DID attestations — freshness window + nonce-store replay defense + signature verification + revocation check + parent-DID expected-issuer pin. CLAUDE.md item #18's "per-plugin DID" surface viewed through the multi-device-sync lens (criterion 16). **Compromise #23's signed envelope V2 with session-nonce replay defense lives here (LIVE at Phase-3 close).**
- **Phase-4-Foundation ring (G24-D + G27-C):** `plugin_did` module — mint/store for the 3rd of CLAUDE.md #18's four identity concepts (plugin-DID = UCAN audience handle, NOT an attested sub-identity); `grant_reader` module — sibling `GrantReader` trait at the `benten-id` layer with a CID-keyed companion that closes the §13.11 structural lesson surfaced by PR #199's UCAN revocation namespace mismatch.

The crate is the load-bearing identity primitive layer underneath four downstream consumers: `benten-caps` (UCAN durable backend at `benten_caps::backends::ucan` + the manifest-envelope chain validator at G24-D-FP-2), `benten-sync` (peer-discovery + handshake; iroh endpoint identity derives from the same Ed25519 bytes), `benten-engine` (typed-CALL dispatch for keypair / UCAN / VC operations + plugin-install lifecycle wiring), and `benten-platform-foundation` (composition-cycle detection at install time consults `PluginDidStore` for known plugin identities; cf. CLAUDE.md #18 final paragraph).

---

## 2. Dependency chain

**Upstream (what `benten-id` pulls in):**
- `benten-core` — **NEW since prior revision.** Pulled in at G27-C wave for the `Cid` typed handle the new `GrantReader::has_unrevoked_grant_for_grant_cid(&Cid)` method takes (Cargo.toml:44). Mirrors the precedent at `crates/benten-caps/Cargo.toml`. `benten-core` is the foundational data-shape crate + is NOT in the `arch-r1-10` forbidden set.
- `benten-errors` — **NEW since prior revision.** Pulled in at R6-FP-3 (cap-r6-r3-1 defensive-return hardening) so `PluginDidStore::insert` can return `Result<(), ErrorCode>` with `ErrorCode::PluginDidHandleDuplicate` (Cargo.toml:50). Pure-data crate at the bottom of the workspace dep graph; not in the forbidden set.
- `ed25519-dalek` — the actual Ed25519 signing/verifying primitive. Re-exports `Signature`.
- `zeroize` — `Zeroize + ZeroizeOnDrop` derives on `SecretKey`. Hand-rolled redacted Debug instead of `secrecy` (Cargo.toml:58-62 note explains: dep-surface minimal, raw-bytes test pin contract).
- `subtle` — `ConstantTimeEq` for the `ct_signature_eq` helper. Wrapped at one site (`crates/benten-id/src/ucan.rs:250`) and called from every security-decision compare across `ucan.rs` / `did_rotation.rs` / `device_attestation.rs`. The grep audit `ucan_chain_walk_constant_time_comparison_audit` polices uniformity.
- `getrandom` + `rand_core` — OS CSPRNG path. Both direct deps so the source-grep tests can pin the call site.
- `serde` + `serde_bytes` + `serde_ipld_dagcbor` — canonical-bytes envelope encoding (RFC 8949 deterministic encoding via sorted-keys). Same DAG-CBOR shape the rest of the engine uses.
- `bs58` — base58btc encoder for the `did:key:z` body.
- `thiserror` — typed errors.

Dev-only: `proptest` (4 proptests), `hex` (test bytes), `toml` (dep-edge audit test reads Cargo.toml itself), `serde_json` (used by integration tests in `tests/`).

**Cargo feature: `testing`** (Cargo.toml:14-19, NEW at R6-FP-3). Exposes test-only constructors needed by integration-test suites that exercise defensive-return paths uncoveable via production-only mint(). Today's exposed surface: `plugin_did::handle_with_did_for_test`. NOT for production callers. The `plugin_did_store_insert_duplicate_rejected` integration test (Cargo.toml:29-31) is `required-features = ["testing"]` so `cargo test --workspace` silently skips when the feature isn't enabled (rather than failing the import).

**Downstream (what depends on `benten-id`):**
- `benten-caps` — UCAN backend (`crates/benten-caps/src/backends/ucan.rs`) imports `Ucan`, `Did`, `Capability`, `validate_chain_*` functions, plus `DeviceRevocation` for the durable revocation-set construction. The Phase-4-Foundation `manifest_envelope_chain_validation` module (G24-D-FP-2) consumes `validate_chain_for_capability` + `RotationLog::accept_rotation_event` for HLC-monotonic-strict rotation acceptance.
- `benten-sync` — `transport.rs` + `peer_discovery.rs` + `handshake.rs` use `Keypair`, `Did`, `PublicKey`, `Signature`, `Capability`, `Ucan` for the iroh endpoint + on-the-wire handshake frames.
- `benten-engine` — typed-CALL dispatch (`typed_call_dispatch.rs::keypair_generate` + `keypair_from_seed`) surfaces the raw key bytes via the Phase 3 typed-CALL op set. Plugin-install lifecycle (G24-D-FP-1) consumes `PluginDidStore`.
- `benten-platform-foundation` — `plugin_manifest::detect_composition_cycle` (Phase-4-Foundation) consults known plugin identities + manifest envelope.
- `bindings/napi` — Node bindings re-expose keypair / DID / UCAN through the TS DSL.

**Forbidden (enforced as a test):** `crates/benten-id/tests/dependency_edges.rs:17-25` reads its own Cargo.toml and asserts none of `benten-graph`, `benten-engine`, `benten-eval`, `benten-caps`, `benten-ivm`, `benten-sync`, `benten-dsl-compiler` appear in `[dependencies]`. This is the `arch-r1-10` pin baked into the test suite. **The two new deps added at G27-C / R6-FP-3 (`benten-core` + `benten-errors`) are intentionally NOT in the forbidden set** — `benten-core` is the foundational data-shape crate and `benten-errors` is a pure-data crate at the bottom of the dep graph.

---

## 3. Files in `src/`

**Ten modules** (up from 8 at prior revision; `plugin_did` + `grant_reader` added). Roughly in dependency order within the crate:

### `lib.rs` (89 LOC)
Crate root. Declares the 10 sub-modules, re-exports the 7 error types at top-level for ergonomic `use benten_id::UcanError;` shapes. Module-level docstring narrates G14-A1 vs G14-A2 scope split, **plus the NEW G27-C scope section** (lib.rs:60-69) calling out the sibling `GrantReader` trait and the `arch-r1-10` reason it's a sibling rather than an extension of `benten-caps`'s trait. Plus three CLAUDE.md baked-in commitments the crate is sensitive to (#3 code-as-graph, #17 deployment shapes, arch-r1-10 dependency-edge). `#![deny(missing_docs)] + #![forbid(unsafe_code)]` at the crate level.

### `errors.rs` (419 LOC; up from 7-error to **9-error** taxonomy)
Nine typed-error enums, one per public surface module + two specialized sub-error shapes. Interesting shape decisions:

- `SeedImportError` — six DISTINCT variants for the envelope-import path. Drives the proptest at `prop_keypair_from_seed_bytes_arbitrary_input_no_panic`.
- `UcanError` — **11 variants** covering time-window (NotYetValid / Expired), audience binding (AudienceMismatch), chain integrity (ChainLinkBroken / BadSignature / EmptyChain), attenuation (AttenuationViolated for both authority AND time-window axes), durable-backend interactions (IssuerKeypairSuperseded / IssuerDeviceRevoked / DeviceEnvelopeViolated), and the typed-CALL leaf-claim gate (CapabilityNotGranted).
- `DeviceAttestationError` — 8 variants. `IncompatibleWithRuntime { detail: &'static str }` closes `br-r4-r1-4` / `br-r4-r2-3` MAJOR with catalog code `E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME`. `code()` method returns stable `&'static str` for the ErrorCode catalog (errors.rs:402-419).
- **`DidRotationError` — 5 variants (up from 3).** `BadSignature` + `PreviousDidMismatch` + `DecodeFailed` from G14-A2, **plus two NEW variants from G24-D-FP-2** (per `docs/future/phase-4-backlog.md §4.10`): `HlcNotStrictlyMonotonic { prev_did, incoming_hlc, latest_hlc }` (errors.rs:312-322) defends against replay-at-same-HLC + nonce-swap attacks; `VerbatimReplay { prev_did, hlc }` (errors.rs:325-332) defends against byte-identical replay.
- `VcError`, `DidError`, `MultiSigError`, `KeypairError` — narrower surfaces (substantively unchanged from prior revision).

### `keypair.rs` (385 LOC)
The Ed25519 primitive. Three load-bearing decisions baked into the type design:

1. **`SecretKey` newtype with `Zeroize + ZeroizeOnDrop` derives + no Clone + redacted Debug** (`crypto-blocker-1`). Pinned by three tests — source-grep `ZeroizeOnDrop` derive, source-grep no `Clone`, redaction round-trip.
2. **`Keypair::generate` pinned to `rand_core::OsRng`** (keypair.rs:181-195) which routes to `getrandom`. The 1000-case proptest + 2000-call aggregate-set distinctness test guard against CSPRNG regressions.
3. **`Keypair::from_seed_bytes` consumes a DAG-CBOR envelope** of shape `{version: u8, alg: "Ed25519", secret_bytes: Bytes(32)}`. Round-trip via `export_seed_envelope`. Two-layer pre-check (length bounds → CBOR decoder) so pathological inputs reject fast with typed variants before hitting the CBOR machine.

Notable test-only escape hatches (`#[doc(hidden)]`):
- `bytes_for_test` + `bytes_ptr_for_test` on `SecretKey` — test-only raw-byte accessors that the in-tree tests use for the zeroize source-grep + redaction round-trip.
- `secret_bytes_unprotected` on `Keypair` (keypair.rs:244-247) — production-shape alias. Documented use sites: `typed_call_dispatch.rs::keypair_generate` + `keypair_from_seed` (typed-CALL output schema today surfaces raw bytes in `Value::Bytes`; phase-3-backlog §2.5 (e) tracks the `Value::SensitiveBytes` extension). Also used in `benten-sync` transport + peer-discovery for iroh keypair construction. **The doc comment warns the caller is responsible for wrapping in `Zeroizing` if the value lives past the immediate dispatch.**

### `did.rs` (125 LOC)
W3C did-method-key encode/decode. Three constants pin the spec compliance (did.rs:26-29): `ED25519_MULTICODEC = [0xed, 0x01]`, `DID_KEY_PREFIX = "did:key:z"`, multibase prefix `z` = base58btc. Encoded shape: `"did:key:z" + base58btc(0xed01 || <32 pubkey bytes>)`.

The `Did` newtype implements `Serialize`/`Deserialize` as `#[serde(transparent)]` (did.rs:47-49) — round-trips the string form, **does NOT validate on deserialize**. Callers needing validate-on-deserialize call `Did::resolve` explicitly. Used by `benten-sync`'s `HandshakeFrame` wire format (`net-blocker-4`).

`Did::from_string_unchecked` (did.rs:110-112) is the post-deserialize "trust the string" entry. The W3C-vector test carries 3 pinned hex pubkey → DID-string fixtures + fail-closed wrong-multicodec.

### `ucan.rs` (704 LOC)
The chain-walk validator. Largest module. Public surface:

- **`Capability { resource, ability }`** — the unit of authority.
- **`Ucan` envelope** + **`UcanClaims`** payload. Signature is over the DAG-CBOR canonical bytes of claims (the `signature` field is excluded from the signed bytes).
- **`UcanBuilder`** — `issuer`/`audience`/`capability`/`not_before`/`expiry`/`proof`/`sign`. Doesn't enforce issuer-DID matches keypair-DID at build time so adversarial-fixture tests can be authored.
- **Six chain-walk validators:** `validate_chain_no_time_check` / `validate_chain_at` / `validate_chain_for_audience` / `validate_chain_for_capability` / `validate_chain_with_rotation_log` / `validate_chain_with_device_revocations` / `validate_chain_with_attestations`. Composition via the private `validate_chain_inner(chain, now, expected_audience)` (ucan.rs:388).

The chain-walk does five things per link inside `validate_chain_inner`:

1. **Time-window check** at every link (`crypto-blocker-2` BLOCKER — renew-the-leaf-forever attack defense).
2. **Signature verification** against the link's issuer DID resolved via `Did::resolve`.
3. **Chain-link integrity**: parent's `aud` MUST equal child's `iss`. Ordering is leaf-first.
4. **Authority attenuation**: every child capability MUST be subsumed by some parent capability per `caps_match_or_subsume` (exact match, parent-wildcard-ability, parent-path-prefix-resource).
5. **Time-window narrowing** (G16-B-B-rest closure of `cap-r4-2` + `tcc-r1-5` R3-A): child's `[nbf, exp]` MUST be a subset of every ancestor's window. Backdating and forward-dating both reject with `AttenuationViolated` carrying time-window-shaped diagnostic strings.

`ct_signature_eq(a, b)` (ucan.rs:250-255) wraps `subtle::ConstantTimeEq`. Made `pub(crate)` at g14-a2-mr-2 fix-pass so device-attestation, DID-rotation, and the new `RotationLog::accept_rotation_event` use the same helper at security-decision sites.

`validate_chain_for_capability` is the typed-CALL-layer integration: composes audience-bind + chain-walk + leaf-claim check using the SAME subsume relation the internal attenuation walk uses (exposed as `capability_satisfies_requirement` for engine-side queries — single source of truth).

### `vc.rs` (486 LOC)
Verifiable Credential issuance + verification. The single most important docstring fact: **"W3C VC v1.1-INSPIRED field shape over DAG-CBOR + Ed25519. NOT wire-format-compatible with external W3C JSON-LD VC consumers."** Dates are `u64` epoch seconds (not ISO 8601); encoding is DAG-CBOR (not JSON-LD); `proof: Vec<u8>` is a flat 64-byte Ed25519 sig (not the LDP `Ed25519Signature2020` envelope).

The wire-interop layer (full `ssi` integration with JSON-LD / Linked-Data-Proofs) is deferred to G14-B per `docs/future/phase-3-backlog.md §2.1-followup`. The vc.rs module docstring carries an explicit Q3 DISAGREE-WITH-EXPLANATION rationale per HARD RULE rule-12 disposition (c).

Surface: `Credential` envelope + `CredentialClaims` payload (W3C field shape). `CredentialSubject` is intentionally narrow (subject-DID + single `(claim_name, claim_value)` pair).

Four verifier entry points: `verify` / `verify_at` (`expirationDate` gate) / `verify_with_registry` (revocation registry lookup) / `verify_in_trust_domain` (issuer allow-list) / `verify_bytes_in_trust_domain` (raw-input parsing-then-verify; drives the 10,000-case malformed-input proptest).

Two in-RAM helpers: `TrustDomain` (HashSet allow-list of issuer DIDs) and `RevocationRegistry` (Mutex<HashSet<String>>). Both tagged "G14-B replaces with durable backing."

### `multi_sig.rs` (158 LOC)
`MultiSigSurface` trait + `Ed25519SingleKey` default impl. Five-method trait surface (`sign` / `verify` / `threshold` / `participants` + the `Signature` / `Error` associated types). Phase 3 ships only the single-key impl with `threshold() = 1`, `participants() = 1`. The `ThresholdMultiSig` placeholder demonstrates the trait extension point (non-sealed); its bodies return `MultiSigError::PostPhase3` per D-PHASE-3-24.

Two architectural pins live here:
- `multi_sig_surface_trait_signature_pinned` — `const _: fn() = || { … }` block doing compile-time signature-shape assertion. Trait drift = compile failure.
- `multi_sig_surface_no_recovery_protocol_specific_behavior_in_phase_3` — source-grep audit. Comments are stripped first (per `crypto-r4-r1-minor-2`).

The `cag-5` + D-PHASE-3-24 commitment: identity-recovery protocol choice deferred to post-Phase-3 v1-assessment-window. **Per Phase-4-Foundation R1 Ben-ratification #6 (SelfRevocation attestation MVP):** the actual identity-recovery protocol path now lands at the Kith effort (deferred to Phase 5+); MVP recovery uses SelfRevocation attestation. `MultiSigSurface` is positioned to absorb threshold-based protocols when the Kith effort needs them.

### `did_rotation.rs` (276 LOC — **substantially extended at G24-D-FP-2**)
Old-DID → New-DID rotation events. `RotationAttestation` carries `previous_did`, `next_did`, `superseded_at`, and a 64-byte Ed25519 signature **by the OLD keypair**.

`rotate_keypair(did, old_kp, new_kp, superseded_at)` (did_rotation.rs:134-158) is the constructor.

`RotationLog` is an in-RAM `Vec<RotationAttestation>` consulted by the chain-walker's `validate_chain_with_rotation_log`. G14-B replaces the in-RAM log with a durable backing; G14-B propagation is still RED-PHASE-ignored (did_rotation.rs:53 backlog destination at §2.1-followup).

**NEW at G24-D-FP-2 (per `phase-4-backlog.md §4.10`); authenticity gate added at umbrella #1171 (Safe-1 #509 / F-FWD-2-01 #1051):** `RotationLog::accept_rotation_event(&mut self, &RotationAttestation) -> Result<(), DidRotationError>`. Three composed defenses:
0. **Authenticity (signature-verify) gate** (umbrella #1171): BEFORE any ordering check, `previous_did` (a self-resolving `did:key`) is resolved to its public key and the attestation's Ed25519 signature is verified via `RotationAttestation::verify_signature_with`. Unresolvable `previous_did` OR signature mismatch ⇒ `DidRotationError::BadSignature`. Closes the #509 auth-bypass: pre-fix a synthesized byte-blob with ANY 64-byte signature was silently accepted and could perpetually revoke a victim's DID at any consuming peer. Mirrors `Acceptor::accept_at` step-4.
1. **Verbatim replay defense**: rejects byte-identical `(previous_did, next_did, superseded_at, signature)` as `VerbatimReplay`. DID + signature compares routed through `ct_signature_eq` per crypto-major-4 UNIFORMITY.
2. **HLC-monotonic-strict defense**: an incoming event for `previous_did` whose `superseded_at` is NOT strictly greater than the latest accepted `superseded_at` for the same `previous_did` rejects as `HlcNotStrictlyMonotonic`. This is the defense against nonce-swap attacks: even if the attacker mutates the nonce / signature, the HLC of the replay event matches the original, so the strict-monotonic check rejects it.

`is_superseded` (did_rotation.rs:259-264) uses `ct_signature_eq` per the uniformity rule, even though DIDs are public.

### `device_attestation.rs` (608 LOC)
The Phase-3 multi-device-sync (criterion 16) surface plus the runtime-target enforcement layer. **Compromise #23 LIVE at Phase-3 close.** Eight public types, the load-bearing flow:

- **`CapabilityEnvelope`** — 4-dimension declaration: `runs_sandbox: bool`, `holds_zones: ZoneScope` (`Full` / `CacheOnly` / `Specific(Vec<String>)`), `online_uptime: UptimePolicy` (`AlwaysOn` / `SessionBounded`), `runs_atrium_peer: bool`. The thin-client minimum-capability envelope (browser tab per CLAUDE.md baked-in #17) is `runs_sandbox=false, holds_zones=CacheOnly, uptime=SessionBounded, runs_atrium_peer=false` and is preset via `issue_for_browser_target`.
- **`DeviceAttestation`** — `device_did` + `parent_did` + `envelope` + 32-byte nonce + `issued_at` epoch seconds + 64-byte parent-signed signature. The signature is over DAG-CBOR canonical bytes of `(device_did, parent_did, envelope, nonce, issued_at)`. The `signature: Vec<u8>` field's public visibility is load-bearing (device_attestation.rs:151-163 docstring): the `acceptor_rejects_attestation_with_forged_signature` test mutates `signature[0] ^= 0x01` to drive the bad-signature negative pin; canonical-bytes round-trip also touches it.
- **Five issuance constructors:** `issue` (zero-init `issued_at`), `issue_at` (caller-controlled epoch), `issue_with_nonce` (caller-controlled epoch + nonce; production callers go through `issue_at`), `issue_for_browser_target` (auto-asserts minimum envelope), `issue_with_runtime_check` (rejects `Browser` target + `runs_sandbox=true` OR `runs_atrium_peer=true` at construction time), `issue_with_authority` (rejects with `EnvelopeWidening` if device claims wider authority than the supplied parent envelope — `cap-r4-7` closure).
- **`envelope_widens` matrix** (device_attestation.rs:333-361) — exhaustive 3×3 over `(parent, device) holds_zones` per g14-a2-mr-6 fix-pass.
- **`Acceptor`** — Compromise #23 runtime gate. Five steps in `accept_at(attestation, now)` (device_attestation.rs:524-582):
  1. Expected-parent pin (if configured via `with_parent_lookup`).
  2. Revocation check (constant-time eq over device_did).
  3. Freshness gate (`now - issued_at <= window`).
  4. **Signature verification against the parent_did's resolved pubkey** (added at g14-a2-mr-1).
  5. Nonce-store replay defense (`(parent_did, nonce)` tuple insertion; replay = duplicate = `NonceReplay`).
- **`DeviceRevocation`** — signed by parent, carries `device_did` + `parent_did` + `RevocationReason` (`DeviceLoss` / `Compromise` / `Decommissioned`). The chain-walker's `validate_chain_with_device_revocations` rejects UCANs signed by a revoked device.

The `generate_fresh_nonce` helper (device_attestation.rs:598-608) composes 4 × `OsRng::next_u64().to_le_bytes()` instead of a `[0u8; 32]` zero-init buffer — works around a CodeQL false-positive.

### `plugin_did.rs` (229 LOC) — **NEW since prior revision; G24-D / Phase-4-Foundation**

Phase-4-Foundation G24-D plugin-DID mint + store. Implements the 3rd of CLAUDE.md baked-in #18's four identity concepts:

1. content-CID (what the plugin IS — canonical bytes)
2. peer-DID signature on original content (provenance)
3. **plugin-DID minted at install** — a UCAN audience handle AND constrained issuer within the manifest envelope
4. user-DID (trust anchor)

**Critical framing (plugin_did.rs:1-24):** plugin-DID is a UCAN audience handle, NOT an attested sub-identity of user-DID. The attestation-chain patterns that belong to device-DIDs (which represent physical hardware) explicitly do NOT apply to plugin-DIDs (which are code running inside the user's engine). This module intentionally does NOT mirror `device_attestation`'s surface — no `Acceptor`, no envelope-widening, no parent-DID signature binding.

Public surface:
- **`PluginDidHandle`** (plugin_did.rs:37-57) — `did: Did` + `keypair: Keypair`. Cloning the handle SHARES the keypair via the underlying SigningKey storage; production callers should NOT clone — the store owns the canonical instance.
- **`mint() -> PluginDidHandle`** (plugin_did.rs:70-75) — the ONLY production surface that mints plugin-DIDs. Per D-4F-16: `did:key:z...` shape with engine-held Ed25519 keypair generated via OsRng. One keypair per install. Does NOT compute attestation envelope, does NOT bind to parent user-DID via signature, does NOT consult `RotationLog`. The binding to user happens at the `InstallRecord` layer (user-DID signs an envelope referencing plugin-DID), NOT at this minting layer.
- **`handle_with_did_for_test(did)`** (plugin_did.rs:86-91) — test-only constructor gated behind `cfg(any(test, feature = "testing"))`. The DID and keypair will NOT be cryptographically bound (DID supplied independently); used to test code paths needing two `PluginDidHandle` values with byte-equal DIDs — primarily the `PluginDidStore::insert` duplicate-rejection arm at `ErrorCode::PluginDidHandleDuplicate` (R6-FP-3 cap-r6-r3-1).
- **`audience_matches_plugin_did(audience, plugin_did) -> bool`** (plugin_did.rs:100-103) — SHAPE-check only. Does NOT traverse an attestation chain. The cap-policy backend at `benten-caps::manifest_envelope_chain_validation` walks the actual UCAN chain.
- **`PluginDidStore`** (plugin_did.rs:110-192) — in-memory `Vec<PluginDidHandle>`. Production code persists this via redb (Phase-4-Foundation `ManifestStore` — shares storage with `GrantStore` per cap-r1-15); at G24-D wave the in-memory shape is the canonical type.
  - `new()` / `mint_and_store() -> Did` / `get(&did) -> Option<&PluginDidHandle>` / `iter()` / `revoke(&did) -> bool` / `len()` / `is_empty()`.
  - **`insert(handle) -> Result<(), ErrorCode>`** (plugin_did.rs:155-161) — **R6-FP-A caller-mint-first contract** per `docs/PLUGIN-MANIFEST.md §3 Plugin-DID minting protocol`. Install path mints the plugin-DID via `mint` (so the receiver can return the `LibraryEntry`'s plugin-DID immediately) and then persists the handle into the store via `insert` — uninstall-time `revoke` can then succeed. **R6-FP-3 (cap-r6-r3-1 defensive-return hardening):** returns `Err(ErrorCode::PluginDidHandleDuplicate)` if a handle with the same DID is already present. Indicates either caller bug (double-mint or double-insert) or adversarial collision (computationally infeasible to find two Ed25519 keypairs whose `did:key:` encodings collide).

### `grant_reader.rs` (316 LOC) — **NEW since prior revision; G27-C / Phase-4-Foundation §4.3**

`benten-id` sibling `GrantReader` trait with CID-keyed companion method. Closes the **§13.11 structural lesson** surfaced by PR #199's UCAN revocation namespace mismatch.

**Architectural framing — why a sibling trait, not an extension** (grant_reader.rs:1-23): per `arch-r1-10`, `benten-id` MUST NOT depend on `benten-caps`. The existing reader surface lives at `benten_caps::grant_backed::GrantReader::has_unrevoked_grant_for_scope(scope: &str)`. Lifting that trait into `benten-id` would violate the layering contract. Instead, this module mints a SIBLING trait — same shape at the API surface plus the CID-keyed companion the §13.11 structural lesson surfaces as missing. The two traits coexist; concrete types may implement both, disambiguating via `<T as benten_id::grant_reader::GrantReader>::…` / `<T as benten_caps::grant_backed::GrantReader>::…` UFCS.

**The structural gap §13.11 closes** (grant_reader.rs:25-40): pre-G27-C the reader API is scope-string-keyed only. PR #199's fail-OPEN root cause was that callers holding a `&Cid` (canonical content-addressed grant handle) had no typed reader API to consult — they had to round-trip through the engine seam, resolve the grant Node, pull its `scope` property, then call back into the scope-keyed reader. Any caller that skipped that round-trip (or got scope resolution wrong, as PR #199 originally did via namespace mismatch) silently fell back to "no matching scope → no revocation → grant still active." The `has_unrevoked_grant_for_grant_cid(&Cid)` companion forecloses this class of bug at the trait surface.

**Consistency invariant** (grant_reader.rs:43-63): for any logical grant `(scope, cid)`, `reader.has_unrevoked_grant_for_scope(scope)? == reader.has_unrevoked_grant_for_grant_cid(&cid)?` under every revocation state. The trait does not enforce this structurally (independent dispatch sites); implementations are responsible for consulting the SAME revocation substrate from both call paths. The `grant_reader_cid_keyed_companion_matches_scope_keyed_for_consistent_inputs.rs` RED-PHASE pin (un-ignored at G27-C wave-time, 2026-05-11) exercises the invariant.

Public surface:
- **`ReaderError`** (grant_reader.rs:76-99) — two variants: `BackendFailed { detail }` (substrate / I/O error; fail-CLOSED signal) and `RecordMalformed { detail }` (record's canonical structure unrecognized; distinct so consume sites can distinguish "I/O is broken" from "data shape on disk is unexpected"). Sibling to `benten_caps::error::CapError::Denied` at the `benten-caps::GrantReader` boundary.
- **`trait GrantReader: Send + Sync`** with two methods:
  - `has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, ReaderError>` (grant_reader.rs:145) — sibling shape to `benten_caps::grant_backed::GrantReader::has_unrevoked_grant_for_scope`; preserved at the lift so consume sites that hold scope strings have a typed handle into the `benten-id`-layer reader without round-tripping through `benten-caps`.
  - `has_unrevoked_grant_for_grant_cid(&self, grant_cid: &Cid) -> Result<bool, ReaderError>` (grant_reader.rs:182) — the new CID-keyed companion. Used by engine's `Engine::revoke_capability_by_grant_cid` surface + napi `revokeCapability(grantCid, actor)` seam + future content-addressed lookup sites.

Why the two-method shape instead of `(Option<&Cid>, &str)` (grant_reader.rs:117-122): a single method taking both keys would force every consume site to construct the OTHER half — defeating the point of the lift, which is to let CID-holding sites consult the reader WITHOUT scope resolution.

Inline unit tests (grant_reader.rs:185-316): in-memory `InMemoryReader` exercising the round-trip + consistency invariant + unknown-key handling + variant Display.

---

## 4. Public API surface

Grouped by intent rather than module:

**Keypair lifecycle (G14-A1).** `Keypair::generate` (CSPRNG path) / `Keypair::from_seed_bytes` (envelope import) / `Keypair::export_seed_envelope` (envelope export) / `Keypair::sign` / `Keypair::public_key` / `Keypair::secret_bytes_unprotected` (caveat: caller responsibility for zeroize wrapping).

**DID encoding (G14-A1).** `Did::from_public_key` / `Did::resolve` (typed-error reverse decode) / `Did::from_string_unchecked` (deserialize boundary trust) / `PublicKey::to_did` (convenience).

**UCAN issuance + chain validation (G14-A1).** `Ucan::builder` → chained builder. Validation entry points: `(time? × audience? × capability? × revocation? × rotation? × device-attestation?)`. The composed `validate_chain_for_capability(chain, expected_audience, required, now)` is the engine-typed-CALL surface; narrower entries for the single-gate cases. `capability_satisfies_requirement` exposes the internal subsume relation.

**Verifiable Credentials (G14-A2).** `Credential::builder` → `sign`. Verify-side: `verify` / `verify_at` / `verify_with_registry` / `verify_in_trust_domain` / `verify_bytes_in_trust_domain`. `TrustDomain::new` + `RevocationRegistry::new` + `RevocationRegistry::revoke`.

**DID rotation (G14-A2 + G24-D-FP-2).** `rotate_keypair(did, old_kp, new_kp, superseded_at)` → `RotationAttestation`. `RotationLog::from_entries` + `RotationLog::append` + `RotationLog::is_superseded` + **`RotationLog::accept_rotation_event` (NEW; HLC-monotonic-strict + verbatim-replay defense per phase-4-backlog §4.10)**. `is_did_superseded` free function alias.

**Device-DID attestation (G14-A2 + G16-D wave-6b; Compromise #23 LIVE).** Issuance entry points (`issue` / `issue_at` / `issue_with_nonce` / `issue_for_browser_target` / `issue_with_runtime_check` / `issue_with_authority`). Canonical-bytes round-trip via `canonical_bytes` + `from_canonical_bytes`. `verify_signature_with(parent_pk)`. `Acceptor::new` + `Acceptor::new_with_revocations` + `Acceptor::with_parent_lookup` + `Acceptor::accept_at` / `accept`. `DeviceRevocation::issue` + `verify_signature_with` (**wired at umbrella #1171 / Hyg-1 #336**: previously zero callers; now invoked at BOTH revocation-consumption sites — the `Acceptor::accept_at` revocation step AND the chain-walker `validate_chain_with_device_revocations` — which resolve `revocation.parent_did` to a public key and `verify_signature_with` it before honoring the revocation. A forged revocation that only matches `device_did` no longer revokes; fail-CLOSED-against-forgery skips the bogus revocation). Chain-walker hooks: `validate_chain_with_attestations` (envelope-violation gate) + `validate_chain_with_device_revocations` (revocation gate, now signature-authenticated). `issue_with_authority` is wired at the napi boundary (`JsDeviceAttestation::issue_with_authority` factory) — construction-time cap-r4-7 envelope-widening defense, the companion to the chain-walker's consume-time attenuation gate (closes Hyg-1 #333).

**Multi-sig extension surface (G14-A2 + post-Phase-3).** `MultiSigSurface` trait + `Ed25519SingleKey` default impl + `ThresholdMultiSig` placeholder. Open trait — downstream crates can implement. Per Phase-4-Foundation R1 Ben-ratification #6, identity-recovery MVP via SelfRevocation; Kith threshold-based protocols deferred to Phase 5+.

**Plugin-DID mint + store (G24-D / Phase-4-Foundation) — NEW.** `plugin_did::mint() -> PluginDidHandle` + `PluginDidHandle::did()` / `keypair()` + `audience_matches_plugin_did` + `PluginDidStore::{new, mint_and_store, insert (caller-mint-first contract; returns Err(PluginDidHandleDuplicate) on duplicate), get, iter, revoke, len, is_empty}`. Test-only: `handle_with_did_for_test` (gated on `cfg(any(test, feature = "testing"))`).

**Sibling `GrantReader` trait (G27-C / Phase-4-Foundation §4.3) — NEW.** `benten_id::grant_reader::{GrantReader, ReaderError, ReaderError::BackendFailed, ReaderError::RecordMalformed}`. Two-method trait: `has_unrevoked_grant_for_scope(&str)` (lift of existing benten-caps shape) + `has_unrevoked_grant_for_grant_cid(&Cid)` (CID-keyed companion closing §13.11 structural gap). Sibling-not-extension shape per arch-r1-10.

---

## 5. Tests inventory

**21 test files; ~3,100 LOC of test surface** (up from 14 files / 2,328 LOC at prior revision). Substantial expansion driven by G24-D plugin-DID + G27-C GrantReader + R6-FP-3 defensive-return hardening + R6-FP-BF un-ignore sweep.

- **`dependency_edges.rs` (35 LOC)** — 1 test. Reads its own Cargo.toml and asserts none of 7 forbidden workspace crates appear in `[dependencies]`. The `arch-r1-10` pin. (Verifies the new `benten-core` + `benten-errors` deps are intentionally NOT in the forbidden set.)
- **`keypair.rs` (259 LOC)** — 7 tests. Round-trip + zeroize source-grep + no-Clone source-grep + redaction round-trip + clone-doesn't-widen-lifetime + OsRng source-grep.
- **`keypair_seed.rs` (171 LOC)** — 6 tests + 1 proptest. Round-trip / short-input / long-input / corrupted / unknown-version / source-grep-no-tracing / DAG-CBOR canonical-bytes stability. Proptest: 2,000 cases.
- **`did_key.rs` (222 LOC)** — 5 tests. Deterministic-from-pubkey / multibase-prefix-z / multicodec-0xed01 + W3C vectors + fail-closed wrong-multicodec.
- **`prop_did_key.rs` (30 LOC)** — 10,000-case round-trip byte-identity proptest.
- **`prop_keypair_generate.rs` (47 LOC)** — 1,000-case distinctness proptest + 2,000-call aggregate-set distinctness test.
- **`ucan.rs` (284 LOC)** — 10 tests. Empty chain rejects / single-token round-trip / attenuation rejects overgrant / nbf / exp / chain-walk propagates expiry / audience-binding rejects cross-atrium replay / constant-time-eq source-grep audit / ucan_chain_revocation_propagates RED-PHASE (`#[ignore]`'d for §2.1-followup).
- **`prop_ucan_attenuation.rs` (249 LOC)** — 1,000-case proptest on both authority-attenuation AND time-window-narrowing axes.
- **`vc.rs` (147 LOC)** — 5 tests. Round-trip / expiration / revocation / trust-domain / tamper detection.
- **`prop_vc_arbitrary.rs` (22 LOC)** — 10,000-case malformed-input no-panic proptest.
- **`multi_sig.rs` (127 LOC)** — 4 tests. Trait-signature compile-time pin + Ed25519SingleKey round-trip + ThresholdMultiSig PostPhase3 stub / recovery-protocol source-grep with comment-stripping.
- **`did_rotation.rs` (113 LOC)** — 4 tests. Emit-attestation / propagation-to-backend RED-PHASE-ignored (§2.1-followup) / superseded-rejected / canonical-bytes-stable.
- **`device_attestation.rs` (569 LOC)** — 12+ tests. Round-trip / envelope-consumed-at-chain-walk / freshness-window / nonce-replay / parent-revocation / revoked-device-cannot-sign / envelope-attenuation / widening-rejected / self-re-attestation / runtime-recheck / browser-auto-assert / browser-construction-time-rejection + forged-signature-rejection + 9-cell zone-scope-matrix.
- **`graph_encoded.rs` (53 LOC)** — 4 RED-PHASE tests, all `#[ignore]`'d with rationale strings routing to G14-B + G14-C.
- **`plugin_did_install_uses_os_rng_not_seed_derivation.rs` (22 LOC) — NEW.** Un-ignored at R6-FP-BF (closes R6 R1 test-coverage-auditor tc-1+tc-2). Two-mint distinctness assertion — defends D-4F-16's "OsRng minting, NOT deterministic seed derivation from user-DID".
- **`plugin_did_install_no_hkdf_from_user_did_grep_assert.rs` (47 LOC) — NEW.** Un-ignored at R6-FP-BF. Source-file grep-assert against `plugin_did.rs` for forbidden patterns (`hkdf`, `Hkdf`, `HKDF`, `derive_from_user`, `derive_from_seed`, `DeriveFromUserDid`, `plugin_did_from_user_did`). Defends D-4F-16 at the source-bytes level.
- **`plugin_did_store_insert_duplicate_rejected.rs` (94 LOC) — NEW; `required-features = ["testing"]`.** R6-FP-3 (cap-r6-r3-1) substantive arm. Three tests: duplicate-DID `Err(PluginDidHandleDuplicate)` + state-preservation; distinct-DID negative control; canonical ErrorCode string `"E_PLUGIN_DID_HANDLE_DUPLICATE"` cross-language rule-mirror check (§3.5g).
- **`grant_reader_has_unrevoked_grant_for_grant_cid_round_trip.rs` (132 LOC) — NEW.** G27-C pin (un-ignored 2026-05-11). Round-trip on the CID-keyed companion: pre-revoke `Ok(true)` → post-revoke `Ok(false)` via in-memory `GrantReader` impl on `HashSet<Cid>` substrate.
- **`grant_reader_cid_keyed_companion_matches_scope_keyed_for_consistent_inputs.rs` (137 LOC) — NEW.** G27-C pin (un-ignored 2026-05-11). Consistency-invariant pin: scope-keyed + CID-keyed paths AGREE on revocation state for consistent inputs, pre-revoke + post-revoke. Defends against fail-OPEN race window between the napi revoke-by-CID seam + the policy scope-string check.
- **`resolve_did_for_cid_round_trip.rs` (7 LOC) — NEW; RED-PHASE.** `#[ignore]`'d at HEAD with HARD RULE rule-12 clause-(b) BELONGS-NAMED-NOW destination at `phase-4-backlog.md §4.26` (Phase-4-Meta RotationLog rehydration + `resolve_did_for_cid` round-trip). Substantive surface lands at §4.26; body deferred.
- **`rotation_log_rehydrated_at_engine_open.rs` (7 LOC) — NEW; RED-PHASE.** `#[ignore]`'d at HEAD with HARD RULE rule-12 clause-(b) BELONGS-NAMED-NOW destination at `phase-4-backlog.md §4.26`. G24-D-FP-2 shipped HLC-monotonic-strict integration into `RotationLog` but the engine-open rehydration seam couples to §4.20 engine-builder seam (Phase-4-Meta).

---

## 6. Benches inventory

No `benches/` directory present. Correctness-pinned; cryptographic performance benching (if needed) would live downstream where the surface is actually wired into a request path — `benten-engine` typed-CALL dispatch, `benten-sync` handshake critical path.

---

## 7. Thin-engine + composable-graph philosophy check

The crate sits clean. Identity is foundational by nature — it can't compose from other primitives, it IS the primitive other things compose against.

**Well-respected surfaces.**

- The `arch-r1-10` test is the single most important architectural pin in the crate. The G27-C SIBLING-not-extension trait shape preserved this when lifting the GrantReader surface to the identity layer (couldn't extend `benten-caps`'s trait without violating `arch-r1-10`; coexistence resolves it cleanly).
- **`capability_satisfies_requirement` exposed as the single subsume relation.** Engine-side query uses the SAME relation the chain-walker uses internally — single source of truth.
- **DAG-CBOR canonical-bytes everywhere.** Seed envelope, UCAN payload, VC claims, device attestation sig input, rotation attestation sig input — all use `serde_ipld_dagcbor::to_vec`. CLAUDE.md item #5 (BLAKE3 + DAG-CBOR + CIDv1) consistently observed.
- **Typed-error discipline.** Every public function returns a `Result<_, ConcreteErrorEnum>`. No `Box<dyn Error>`, no `String` errors. The 11-variant `UcanError` is the largest enum; every variant is hit by at least one test pin. New error surfaces (`DidRotationError::HlcNotStrictlyMonotonic` + `VerbatimReplay`; `ReaderError::BackendFailed` + `RecordMalformed`; `ErrorCode::PluginDidHandleDuplicate`) all follow the same discipline.
- **Cap-attenuation rule is exhaustive on both axes.** Authority axis AND time-window axis. Both have their own AttenuationViolated diagnostic shape but join the same error variant family.
- **`envelope_widens` made fully exhaustive across all 9 zone-scope cases at g14-a2-mr-6 fix-pass.** No implicit fallthrough.
- **G24-D-FP-2 rotation-event defense composes two defenses** (verbatim-replay + HLC-monotonic-strict) into a single `accept_rotation_event` entry point. The nonce-swap-attack defense (HLC-strict) is the non-obvious load-bearing piece: even if the attacker mutates the signature, the HLC tells the truth.

**Constant-time discipline uniformity.** `ct_signature_eq` is wrapped at one site and used at every security-decision compare across `ucan.rs`, `did_rotation.rs`, `device_attestation.rs`. The `ucan_chain_walk_constant_time_comparison_audit` grep test enumerates a forbidden-pattern list and walks all modules' source. The new `RotationLog::accept_rotation_event` body uses `ct_signature_eq` for DID + signature compares per the UNIFORMITY rule even though DIDs are public. **Umbrella #1171 closed the last two uniformity-drift sites** (Safe-3 #599 `rotate_keypair` caller-DID-vs-derived-DID compare + Safe-1 #515 `Acceptor::accept_at` expected_parent compare — both early-rejection arms that used `!=` / `.as_str() !=` forms the `==`-only forbidden-pattern list did not catch). The grep audit was widened with the `!=` family (`previous_did !=`, `parent_did !=`, `device_did !=`, `nonce !=`, `signature !=`, …) plus the two specific drift expressions, so a future early-rejection arm cannot silently re-drift.

**Plugin-DID surface is deliberately NOT a device-attestation shape.** `plugin_did.rs:1-24` is explicit: plugin-DID is a UCAN audience handle, NOT an attested sub-identity. No `Acceptor`, no `parent_did` envelope-signature binding, no `RotationLog` consultation at the minting layer. This is the CLAUDE.md #18 four-identity-concepts model surfaced at the type level — the module structure REJECTS the parallel-to-device-DID pattern by construction.

**Worth flagging — potential pluggability friction (CLAUDE.md item #19 perspective).**

- **Ed25519 hardcoded throughout.** The DAG-CBOR envelope tags `alg: "Ed25519"` and the import path returns `UnknownAlg` for anything else. `MultiSigSurface` is the cleanest extension trait but the Phase-3 default carries Ed25519 in its type signature. Engine-level extension (#19) for alternate signature schemes would need to either implement `MultiSigSurface` with a different `Signature` associated type or thread a generic over `Keypair` / `Did` / UCAN. Per Phase-4-Foundation R1 Ben-ratification #6, identity-recovery MVP via SelfRevocation defers the threshold-multi-sig path; the Kith effort (Phase 5+) will exercise this extension surface in earnest. Not a defect today.

- **`SecretKey::bytes_for_test` + `secret_bytes_unprotected` are documented escape hatches.** Production use sites: `typed_call_dispatch.rs` for keypair / from_seed (phase-3-backlog §2.5 (e) carries the `Value::SensitiveBytes` extension); `benten-sync` transport / peer-discovery (iroh keypair construction). Gap acknowledged with named destinations.

**No identity-layer/sync-layer coupling drift.** `benten-sync` consumers grep cleanly: `transport.rs` + `peer_discovery.rs` + `handshake.rs` import the expected types. The reverse direction is forbidden by `arch-r1-10`.

**No cap-attenuation logic that should live in benten-caps.** The chain-walk lives here because it has to. `benten-caps`'s `UCANBackend<B>` composes ON TOP of it. The new G27-C sibling `GrantReader` trait shape preserves this clean split: the IDENTITY-layer reader surface is here (CID-keyed; arch-r1-10-correct); the CAP-layer reader surface stays in `benten-caps` (scope-keyed; existing).

**Device-attestation envelope V2 internals don't leak.** `DeviceAttestation`'s `signature` field is publicly accessible — required for test-side bit-flipping — but `canonical_bytes` uses an internal `SigInput<'a>` struct that excludes the sig field. Construction goes through `issue_*` functions only.

**One small thing to flag for retrospective:** the `validate_chain_no_time_check` docstring is unusually candid about its time-handling ambiguity; production code paths in `benten-caps`'s UCANBackend use `validate_chain_at`. Fine as-is.

---

## 8. Phase-4-Foundation + Phase-4-Meta expectations

Several knowable forward-looking surfaces touch this crate:

**Phase-4-Foundation status (LIVE — substantial G24-D + G27-C work already shipped).**

- ✅ **G24-D plugin-DID mint + store** — landed at this revision (`src/plugin_did.rs`); per CLAUDE.md #18 four-identity-concepts model; D-4F-16 OsRng-minting discipline; R6-FP-A caller-mint-first contract; R6-FP-3 defensive-return hardening (`PluginDidHandleDuplicate`).
- ✅ **G24-D-FP-2 RotationLog HLC-monotonic-strict + verbatim-replay defense** — landed at this revision (`RotationLog::accept_rotation_event` + 2 new `DidRotationError` variants); closes phase-4-backlog §4.10.
- ✅ **G27-C sibling GrantReader trait** — landed at this revision (`src/grant_reader.rs`); closes §13.11 structural gap surfaced by PR #199; sibling-not-extension per arch-r1-10.
- ✅ **UCAN revocation observance closure** — Track B PR #109/#199; the durable observance composes against `validate_chain_with_device_revocations` + `validate_chain_with_rotation_log` here.

**Phase-4-Foundation plugin manifest schema (CLAUDE.md item #18).** The per-plugin DID + UCAN model lives on this crate's surface. Each plugin gets a `Did` (via `plugin_did::mint`). Manifest verification flows:
- (a) user-as-root: `Keypair` for user identity; `Did` is chain anchor; user-DID signs `InstallRecord` (per CLAUDE.md #18 implementation refinement: "user-as-source signing model").
- (b) install-time manifest: signed by plugin-author keypair; verified at install. VC shape (`vc.rs`) maps to "plugin author claims X about plugin" but the actual manifest signing today lives in `benten-platform-foundation`.
- (c) runtime delegation: UCAN chain — `Ucan::builder` + `validate_chain_for_capability`. The `caps_match_or_subsume` subsume relation evaluates the runtime-delegation gate. The `benten-caps::manifest_envelope_chain_validation` module (G24-D-FP-2) is the cross-plugin chain validator.

**Phase-4-Meta deferrals (per `phase-4-backlog.md §4.26` and elsewhere):**
- **`benten_id::resolve_did_for_cid` round-trip surface** — RED-PHASE at HEAD (`tests/resolve_did_for_cid_round_trip.rs`). G24-D shipped `plugin_did::mint` + cap-r1-16 was triaged into G24-F's `DidKeyedSession::resolve`; the standalone seam is a separate Phase-4-Meta concern coupled to RotationLog rehydration.
- **RotationLog rehydration at engine-open** — RED-PHASE at HEAD (`tests/rotation_log_rehydrated_at_engine_open.rs`). G24-D-FP-2 shipped HLC-monotonic-strict integration into `RotationLog` but the engine-open rehydration seam couples to §4.20 engine-builder seam.
- **Identity-recovery protocol choice** — `MultiSigSurface` shape-stable extension point exists; SelfRevocation attestation MVP per Phase-4-Foundation R1 Ben-ratification #6 (Kith deferred to Phase 5+).

**Device-DID cross-machine sync (already shipped at Phase-3 close).** Compromise #23 + `DeviceAttestationEnvelope V2` shipped at G16-D wave-6b PR #163. The `Acceptor` runtime gate is the boundary. `AtriumHandle` exposes `set_local_device_attestation` + `set_acceptor` (per phase-3-backlog §2136). The capability-envelope `runs_sandbox / holds_zones / online_uptime / runs_atrium_peer` shape is forward-stable.

---

## 9. Open questions / unresolved internals

A handful of things worth surfacing for retrospective:

1. **`SeedImportError::InvalidSecret` is reserved but unreachable today.** ed25519-dalek 2.x accepts any 32 bytes as a SigningKey seed. Variant exists to keep the typed-error surface stable across future algorithm extensions. Not a defect; flagging because coverage tools might mark it as dead-arm.

2. **`UcanClaims::aud` is a `String` not a typed `Did`.** Audience is compared via `ct_signature_eq` over UTF-8 bytes — works correctly. Looser type accommodates non-DID audiences (some UCAN ecosystems use URLs). Convenience method `audience_did(&Did)` exists. Coherent design choice.

3. **`validate_chain_no_time_check` is ambiguous re. `nbf` handling.** Docstring directs callers to `validate_chain_at` for production. Used primarily by the basic-validation test pin. Possible future cleanup: gate behind `#[cfg(any(test, debug_assertions))]`.

4. **`AttestationKind` enum has one variant** (`SupersededBy`). Keeps shape across future rotation-event types (e.g. multi-sig-rotation, threshold-revoke per Kith). `kind()` returns a constant.

5. **No compile-time guard against `#[cfg(target_arch = "wasm32")]` thinning.** The crate compiles on wasm32 today (consumer-side gating in `benten-sync` + `benten-caps::ucan`). If a future contributor adds something native-only into this crate, the deployment-shape contract from CLAUDE.md #17 silently breaks. The `dependency_edges.rs` test catches workspace-crate adds but not external-crate native-only adds. Defensive measure: CI check that runs `cargo check --target wasm32-unknown-unknown -p benten-id`. Flagging because the crate's load-bearing position makes any wasm32-incompatible regression high-blast-radius.

6. **`secret_bytes_unprotected` named uses are typed-CALL surface + iroh keypair construction.** Both have phase-3-backlog destinations (§2.5 (e) for `Value::SensitiveBytes` extension; iroh's expectation of raw bytes is upstream). Worth confirming on the next phase-close audit pass that no NEW callsites accumulated.

7. **`PluginDidHandle::keypair()` exposes the signing keypair.** The plugin-DID's signing keypair is held in `PluginDidStore` and the engine consumes it via `PluginDidStore::get(&did).keypair()` for UCAN issuance within the manifest envelope. The keypair material itself is NOT secret in the same sense as user-DID's keypair (plugin-DID is "code running inside the user's engine"); however, the docstring's "production callers should NOT clone — the store owns the canonical instance" caveat is the implicit boundary. A future contributor adding a `PluginDidHandle::clone()` impl would silently break this discipline. The `#[derive(Debug)]` (plugin_did.rs:37) does NOT compose a Clone derive, so the discipline holds at HEAD by construction.

8. **`GrantReader` consistency invariant is not enforced structurally.** The two methods (`has_unrevoked_grant_for_scope` + `has_unrevoked_grant_for_grant_cid`) are independent dispatch sites; the invariant relies on implementations consulting the SAME revocation substrate from both call paths. The integration test pin asserts this on the canonical in-memory shape, but a wrong implementation that diverges between the two paths would not be caught at the trait level. A future hardening (linker-time? trait-default-method? compile-time-derived?) could close this but would substantively reshape the trait. Not a defect today; flagging because the §13.11 fail-OPEN class of bug is what motivates the trait, and the trait's structural shape is the second line of defense.

9. **`testing` Cargo feature surface is currently single-purpose.** Only exposes `plugin_did::handle_with_did_for_test`. As more defensive-return paths land that need test-only constructors, the feature will accumulate surface. Cargo.toml comment (line 17-19) names the contract clearly; worth re-auditing at every phase-close to confirm `testing` hasn't drifted into production-leak territory.
