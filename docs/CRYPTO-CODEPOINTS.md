# Crypto Codepoints — the Benten suite-selector table + reserve registry

Benten owns **only the thin suite-selector codepoint table** (one
codepoint-per-suite; the MLS RFC 9420 model, NOT the HPKE algorithm-triple).
Component algorithm IDs reference the IANA HPKE/COSE registries — Benten never
mints its own algorithm numbers. This document is the human-readable companion
to `crates/benten-crypto-suite/src/codepoint.rs` (the wire-canonical source of
truth) and `docs/V1-FROZEN-INTERFACE.md` item 6 (the freeze record).

## The conservative-fallback freeze contract (NQ-A1)

The codepoint table integer values are **FROZEN at v1-beta** (per
V1-FROZEN-INTERFACE.md item 6). Algorithms behind each codepoint are swappable
within the multiformats framing; the codepoint values themselves are
wire-canonical and permanent.

Post-freeze security findings are handled **additively** (reserve-codepoints + a new tag) — **never a silent wire-break**.
An unknown / reserved codepoint typed-rejects (fail-closed
`UnsupportedAlgorithm`) — never a silent fallback, never a silent downgrade.
This is the NQ-A1 conservative-fallback policy:

- New algorithms land at **unused reserved codepoints** (additive); existing
  codepoint values are never repurposed.
- A reserved codepoint is typed-rejected at v1-beta and becomes LIVE
  additively at v1-GM+ — adding it is **never a wire-format break** (old
  content with old codepoints still decodes forever).
- `old-codepoints-supported-forever`: a deprecated codepoint still decodes
  existing content (no new content); only `Quarantined`/`Burned` states
  typed-reject permanently.

## LIVE suite-selector codepoints (v1-beta)

| Codepoint | Suite | State |
|-----------|-------|-------|
| `0x647a`  | X25519⊕ML-KEM-768 hybrid KEM (real X-Wing SHA3-256 combiner) + ChaCha20-Poly1305 bulk | **LIVE — default** |
| `0x6400`  | X25519-only classical KEM + ChaCha20-Poly1305 bulk | LIVE (non-default downgrade) |
| `0x0001`  | Ed25519⊕ML-DSA-65 hybrid signature — byte-faithful IETF LAMPS Composite `id-MLDSA65-Ed25519-SHA512` (OID `1.3.6.1.5.5.7.6.48`; `draft-ietf-lamps-pq-composite-sigs-19` + test-vector commit `f0627ab3`; wire `mldsaSig(3309) ‖ tradSig(64)` = 3373 B, ML-DSA-first, NO commitment trailer) | LIVE (default sig) |
| `0x0002`  | Ed25519-only classical signature | LIVE (non-default downgrade) |
| `0x6500`  | Layer-C drop, plaintext-sender (sender-DID on wire) | LIVE (non-default) |
| `0x6510`  | Layer-C drop, **Sealed-Sender** (sender-DID inside ciphertext) | **LIVE — default (BR-1)** |
| `0x6520`  | Layer-C group multi-stanza (`HpkeMultiBase`, blinded per-stanza AAD) | LIVE |
| `0x6600`  | MembershipSet set-keying envelope (`MEMBERSHIP_SET_ENCRYPTION`; every `MembershipSetKind` binds here; Sealed-Sender default) | LIVE |
| `0x6610`  | MembershipSet K_Set group multi-stanza (blinded AAD) | LIVE |

> **`0x0001` draft-not-RFC caveat:** the LAMPS composite is pinned to
> `draft-ietf-lamps-pq-composite-sigs-19` (NOT yet a final RFC — the `M'`
> Prefix embeds the literal `"CompositeAlgorithmSignatures2025"`). Re-verify
> the construction at the final RFC; crypto-agility absorbs any change
> additively (new codepoint, never a wire-break on `0x0001` content already
> in flight).

## RESERVED codepoints (typed-reject at v1-beta; additive at v1-GM+)

| Codepoint / band | Reserve | State |
|------------------|---------|-------|
| `0x647b`         | NF-1 ML-KEM-768⊕HQC PQ⊕PQ end-state | RESERVED (build-trigger = FIPS 207 final) |
| `0x647c`         | Pure-PQ ML-KEM-768-only swap-matrix arm | RESERVED (audit-gated) |
| `0x0003`         | NF-1 ML-DSA-65⊕SLH-DSA PQ⊕PQ signature | RESERVED |
| `0x0000`         | No-encryption (plaintext partition) | RESERVED |
| `0x6320..0x632F` | RemotePermission band (incl. `ExecuteWorkflow` reserve) | RESERVED |
| `0x6620`         | `SubsetRef` federation reserve (`MEMBERSHIP_SET_RESERVED_0X6620`) | RESERVED |
| `0x6380..0x638F` | MLS-Application FS-future bracket (`MLS_APPLICATION_BASE`; NOT MembershipSet) | RESERVED |
| `0x6390..0x639F` | MLS-Welcome FS-future bracket (`MLS_WELCOME_BASE`; NOT Sealed-Sender) | RESERVED |
| `0x63A0..0x63AF` | CGKA-Commit FS-future bracket (`CGKA_COMMIT_BASE`; incl. `RotatingGroupKeyChainedMode` + `ChainedStateTlv`) | RESERVED |
| `0x63B0..0x63BF` | Bird-of-Prey AKEM FS-future bracket (`BIRD_OF_PREY_BASE`) | RESERVED |
| `0x63C0..0x63CF` | draft-prabel FS-future bracket (`DRAFT_PRABEL_BASE`) | RESERVED |
| `0x6700..0x67FF` | Lifecycle / revocation band (`LIFECYCLE_BAND_BASE`) | RESERVED |
| (no Core integer) | `RecoveryArtifact` — reserved-at-Core conceptually; the `RecoveryHook` trait lands in Phase-4-Meta-Composing alongside the allocated codepoint (NQ-W5/m-14) | RESERVED (Composing) |

The reserved set is enumerated in
`crates/benten-crypto-suite/src/codepoint.rs::ReservedCodepoint` — each slot's
`resolve()` ALWAYS typed-rejects at v1-beta (never a silent accept). The
`ChainedStateTlv` per-stanza sub-slot (GAP-6b) is **AAD-bound** when present
(`chained_state_tlv_aad_binding`) so a present-vs-absent flip is detectable at
decrypt (not advisory).

> **Band-base reconciliation (R6 R6 F-04).** `ReservedCodepoint::band_base()`
> for both `RotatingGroupKeyChainedMode` and `ChainedStateTlv` dispatches from
> the **CGKA-Commit FS-future bracket** (`CGKA_COMMIT_BASE == 0x63A0`; the §4.0
> row above), NOT the MLS-Application bracket `0x6380`. Earlier code returned
> `Some(0x6380)`, which disagreed with the §4.0 registry assignment; it is now
> `Some(0x63A0)`. Consequently the GAP-6b AAD binding a PRESENT `ChainedStateTlv`
> sub-slot emits is `0x01 ‖ 0x63A0_be` = bytes `01 63 a0` (was `01 63 80`). No
> LIVE codepoint moved — these are RESERVED, typed-rejected slots, and the
> AAD-byte change touches only the (not-yet-emitted) reserve binding. The byte
> is pinned + cross-checked against `registry::CGKA_COMMIT_BASE` in
> `crates/benten-drop/tests/f_nqa1_1_frozen_surface_additive_extensibility.rs`
> (PIN 4).

## did:key hybrid-pubkey multicodec (NQ-C4 / U15) — RESOLVED

The PQ-**hybrid** public keys carried in `did:key` must reference REGISTERED
multiformats multicodec values, distinct from the Ed25519-only
`ED25519_MULTICODEC = [0xed, 0x01]`, per CLAUDE.md baked-in #5 ("component
algorithm IDs reference the multiformats/IANA registry — never a Benten-private
number").

**Multiformats-registration status (RESOLVED, NQ-C4 / §5.D-9).** There is **no
registered COMPOSITE multicodec** for the hybrid pubkey shapes, but registered
**COMPONENT** codes DO exist:

| Component | Registered multicodec | Unsigned-varint |
|-----------|-----------------------|-----------------|
| ML-DSA-65 pubkey | `mldsa-65-pub = 0x1211` | `[0x91, 0x24]` (`MLDSA65_PUB_MULTICODEC`) |
| Ed25519 pubkey | `ed25519-pub = 0xed` | `[0xed, 0x01]` (`ED25519_MULTICODEC`) |

So the **v1 hybrid `did:key` encodes the hybrid key as TWO registered
component multikeys** (the multi-multikey form), **ML-DSA FIRST** (consistent
with `benten_crypto_suite::sig::PublicKey::from_lamps_composite_bytes`, which
expects `mldsaPK(1952) ‖ tradPK(32)`):

```text
did:key:z + base58btc( varint(0x1211) ‖ mldsaPK(1952) ‖ varint(0xed) ‖ tradPK(32) )
```

This is **#5-clean** (no invented number). Implemented at
`crates/benten-id/src/did.rs::{Did::from_hybrid_public_key, Did::resolve_hybrid}`;
component-codec dispatch is typed-reject (`DidError::UnknownMulticodec` /
`HybridBodyTooShort` / `HybridTrailingBytes` / `InvalidHybridPublicKey`) — never
a silent fallback. (The hybrid **KEM** pubkey, X25519⊕ML-KEM-768, follows the
same two-registered-component-multikey discipline when wired.)

**Fallback-only interim values.** The single-byte private prefixes
`HYBRID_SIG_MULTICODEC = [0xef, 0x01]` + `HYBRID_KEM_MULTICODEC = [0xf0, 0x01]`
in `crates/benten-id/src/did.rs` were the G-CORE-9 NQ-C4 reserved-private
interim, held when no registered code was known. They are **NOT the v1 wire
encoding** — they are retained as **documented fallback-only** and are
**#5-RISKY** (single-byte private values that squat the registered single-byte
multicodec range). New content uses the two-component-multikey form above.

The `did:agent:` method is an **optional allowlist alias** (Inv-22: nature
DERIVED via method-parse; the alias is a hint, never a stored authoritative
discriminator and never authority-bearing).

## Transport-config reserve (Compromise #53 / §3.9)

Transport selection (`benten-sync::transport_config::TransportConfig`) follows
the same additive discipline: `GossipPlusBlobs` (iroh-gossip + iroh-blobs)
ships at v1-beta; the `Willow` / `IrohRoq` / `IrohLive` variants are
reserved-and-typed-rejected — adding them later is additive, never a
wire-break, never a silent fallback to gossip.

## §0.4 — Codepoint source-of-truth resolution (supersession record)

The two source docs (M-CONS-FINAL F8/F21 + the 9-eyes registry U13/U22)
conflicted on three integers. The resolution (later / more-specific +
collision-free), recorded here so the reassignment is **not silent**:

| Integer | M-CONS-FINAL (F8/F21) | 9-eyes registry (U13/U22) | RESOLUTION |
|---|---|---|---|
| `0x6380` | "MembershipSetEncryption family" | **MLS-Application bracket** (`0x6380..0x638F`) | **9-eyes wins** → MLS-Application; **MembershipSetEncryption RELOCATES to `0x6600`** |
| `0x6390` | "Sealed-Sender paired slot" | **MLS-Welcome bracket** (`0x6390..0x639F`) | **9-eyes wins** → MLS-Welcome; **Sealed-Sender = `0x6510`** (Ben ruling #1) |
| Sealed-Sender | `0x6390` | **`0x6510`** | **`0x6510`** (the ONE canonical value) |

**Supersession note.** The M-CONS-FINAL F8/F21 assignment of `0x6380` to
MembershipSetEncryption and `0x6390` to the Sealed-Sender paired slot is
**SUPERSEDED**: `0x6380` / `0x6390` are the **MLS-Application / MLS-Welcome**
FS-future brackets (9-eyes U13), `MembershipSetEncryption` is **relocated to
`0x6600`** (a fresh MembershipSet band disjoint from the MLS bracket), and
Sealed-Sender has exactly one canonical value, **`0x6510`**.

## §4.0 — group/MLS/MembershipSet band registration (symbol-bound rows)

These rows bind each codepoint **value to its symbol on ONE line** (the
registry-drift guard — a value is "registered" only when co-located with its
symbol; a prose-only mention does not count):

| Codepoint | Symbol | Band | State |
|---|---|---|---|
| `0x6100` | `VAULT_ENVELOPE` (`VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT`) | Layer-A vault at-rest band (XChaCha20 24-byte XNonce) | **FREEZE** (vault on-disk; `f_va_1`) |
| `0x6101` | `SYMMETRIC_AEAD_12B` (`SYMMETRIC_AEAD_12B_CODEPOINT`) | symmetric-AEAD 12-byte-nonce sibling | **FREEZE** (12-byte-nonce variant; NOT the vault codepoint) |
| `0x6380..0x638F` | `MLS_APPLICATION_BASE` (MLS-Application) | FS-future bracket | CODEPOINT-RESERVE (9-eyes; NOT MembershipSet) |
| `0x6390..0x639F` | `MLS_WELCOME_BASE` (MLS-Welcome) | FS-future bracket | CODEPOINT-RESERVE (9-eyes; NOT Sealed-Sender) |
| `0x63A0..0x63AF` | `CGKA_COMMIT_BASE` (CGKA-Commit) | FS-future bracket (incl. `RotatingGroupKeyChainedMode` + `ChainedStateTlv`) | CODEPOINT-RESERVE |
| `0x63B0..0x63BF` | `BIRD_OF_PREY_BASE` (Bird-of-Prey AKEM) | FS-future bracket | CODEPOINT-RESERVE |
| `0x63C0..0x63CF` | `DRAFT_PRABEL_BASE` (draft-prabel) | FS-future bracket | CODEPOINT-RESERVE |
| `0x6600` | `MEMBERSHIP_SET_ENCRYPTION` | MembershipSet band (RELOCATED from `0x6380`) | **FREEZE** (§0.4 collision fix) |
| `0x6610` | `MEMBERSHIP_SET_GROUP_MULTI_STANZA` | MembershipSet band | **FREEZE** (group K_Set multi-stanza; R4.6-corrected value) |
| `0x6520` | `LAYER_C_DROP_MULTI_RECIPIENT` | Layer-C drop / recipient band | **FREEZE** (R0.7-blinded Layer-C group multi-stanza; NOT a MembershipSet) |
| `0x6310..0x631F` | `DEVICE_LINK_BAND_BASE`/`DEVICE_LINK_BAND_END` | Layer-D DeviceLink (Signal-Provisioning) band | **FREEZE** (R0.7 §4.1; out-of-band integers typed-reject fail-closed) |
| `0x6320..0x632F` | `REMOTE_PERMISSION_BAND_BASE`/`REMOTE_PERMISSION_BAND_END` | Layer-D RemotePermission band (incl. `ExecuteWorkflow` reserve) | **FREEZE** (R0.7 §4.1 band base; out-of-band integers typed-reject fail-closed; per-slot: `PermissionRequest`/`PermissionGrant` LIVE, `ExecuteWorkflow` reserved-typed-reject at v1-beta) |
| `0x6700..0x67FF` | `LIFECYCLE_BAND_BASE` | Lifecycle / revocation band | CODEPOINT-RESERVE (band base registered; per-slot allocation at Composing) |

The in-code wire-lock for the group-band constants is regression-guarded in
`crates/benten-crypto-suite/tests/f_cp_codepoint_registry_dispatch.rs`
(`MEMBERSHIP_SET_GROUP_MULTI_STANZA == 0x6610` and
`LAYER_C_DROP_MULTI_RECIPIENT == 0x6520`).

> **SSOT note (F-24).** `crates/benten-crypto-suite/src/registry.rs` is the
> single source of truth for the `0x6100..0x6FFF` allocation map (the
> band-ownership registry). The wire-byte-emitting values, however, are
> **independently defined in the producing crates** and mirrored back to the
> registry — they are NOT re-exported from it:
> `crates/benten-membership-set/src/codepoints.rs` defines the MembershipSet
> band values (`MEMBERSHIP_SET_ENCRYPTION == 0x6600`,
> `MEMBERSHIP_SET_GROUP_MULTI_STANZA == 0x6610`,
> `MEMBERSHIP_SET_RESERVED_0X6620 == 0x6620`), and
> `crates/benten-drop/src/layer_c.rs` defines the Layer-C drop values
> (`LAYER_C_DROP == 0x6500`, `DROP_TO_RECIPIENT_SEALED_SENDER == 0x6510`,
> `LAYER_C_DROP_MULTI_RECIPIENT == 0x6520`). The Layer-D DeviceLink base is
> defined SEPARATELY in `crates/benten-engine/src/layer_d/device_link.rs`
> (`DEVICE_LINK_BAND_BASE == 0x6310`) and mirrored to
> `benten_crypto_suite::registry::DEVICE_LINK_BAND_BASE` — it is NOT a `layer_c.rs`
> const. Because the same integer lives in two places, a single-side edit would
> drift the registry from the wire. A cross-crate const-equality regression-pin in
> `crates/benten-drop/tests/f_disc_2_invariant_and_doc_registration_catch_net.rs`
> (`f_disc_2_codepoint_ssot_cross_crate_const_equality`) asserts the producing
> crates' values equal `benten_crypto_suite::registry::*` for the MembershipSet
> band (`MEMBERSHIP_SET_ENCRYPTION` / `MEMBERSHIP_SET_GROUP_MULTI_STANZA` /
> `MEMBERSHIP_SET_RESERVED_0X6620`) and the Layer-C drop band
> (`LAYER_C_DROP` + `LAYER_C_DROP_MULTI_RECIPIENT`); a one-sided edit to any of
> those fails the build. (The `DROP_TO_RECIPIENT_SEALED_SENDER == 0x6510` value is
> wire-locked by the Layer-C drop-band byte-pins in
> `crates/benten-drop/tests/f_lc_hpke_encrypt_to_recipient_sealed_sender.rs`, not
> by this cross-crate const-equality arm.)

The Layer-D DeviceLink band `0x6310..0x631F` is the R0.7 §4.1 Signal-Provisioning
device-link wire band (`crates/benten-engine/src/layer_d/device_link.rs`
`DEVICE_LINK_BAND_BASE == 0x6310` / `DEVICE_LINK_BAND_END == 0x631F`; mirrored as
`crates/benten-crypto-suite/src/registry.rs::DEVICE_LINK_BAND_BASE`). It is a
DIFFERENT band from the RemotePermission and MembershipSet bands;
`device_link::dispatch_device_link_codepoint` typed-rejects (fail-closed,
CLAUDE.md #5) any integer outside `0x6310..=0x631F`. The band base is
registry-presence-guarded in
`crates/benten-crypto-suite/tests/f_cp_codepoint_registry_dispatch.rs`
(`DEVICE_LINK_BAND_BASE` in the scanned codepoint set) + literal-value-locked in
`crates/benten-engine/tests/f_ld_4_multi_device_key_wrap_provisioning.rs`
(`DEVICE_LINK_BAND_BASE == 0x6310` wire-lock). This row closes the R6-R2 F-18
finding (the FREEZE band was absent from this registry doc — a prose-only-not-
symbol-bound gap).

> **NAMED-CARRY obligation (F-full R6 R1 finding F-12; BELONGS-NAMED-NOW).**
> The `0x6380` slot carries a **3-way discrete-value reserve obligation** that
> MUST stay symbol-bound + integer-pinned, not prose-only: (1) `0x6380..0x638F`
> = MLS-Application FS-future bracket (9-eyes; NOT MembershipSet); (2)
> `0x6390..0x639F` = MLS-Welcome FS-future bracket; (3) the historical
> M-CONS-FINAL `0x6380` `MembershipSetEncryption` assignment is RELOCATED to
> `0x6600` (the §0.4 collision fix). Build-out item: a discrete-value
> regression-pin in `f_cp_codepoint_registry_dispatch.rs` asserting each of the
> three discrete reserves holds its integer (so a future round cannot silently
> re-collide `0x6380` with the relocated MembershipSet value). Surfaced at
> R6 R1; lands at the next crypto-suite codepoint build-out sub-pass.

> **REJECTED: width-unification (freeze record — do not re-litigate).** A
> future round MUST NOT unify the u16/u32 per-band length-prefix widths across
> the Layer-C drop band and the MembershipSet band. The two bands are
> separately-frozen, codepoint-discriminated byte-strings; each band's AAD
> field-set (`docs/SECURITY-PROOFS.md`) is frozen as authored. Unifying them
> would be a wire-break for one band.

## §4.4 — Net frozen-surface summary (O-8 — single canonical tally)

The GN-wins (cheap-additive) shrink the frozen surface beyond the keying
minimum. The single canonical tally is **`net -8`**:

- **−1** audit structure (graph-native version-Node content, not a frozen wire structure),
- **−1** `MembershipEvent` wire-enum (version-Node content, not a frozen codepoint-tagged wire enum),
- **−4** AuditAccessGradation codepoints (a `RestrictedScope` arm + UCAN-caveat compositions, NOT codepoints),
- **−2** Garden / Grove sub-codepoints (GovernanceConfig signed-Node content, NOT crypto-wire sub-codepoints),

= **`net -8`**. **The R0.1 `-6` figure double-counted** — this `net -8` is the
single canonical tally (O-8); the corrected accounting supersedes the earlier
double-counted figure. **CE-1 drops the economic reserve** (ZERO MembershipSet freeze hook);
the member side adds **ZERO** new frozen field; the compute side adds **ZERO**
reserve (see `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Rows D-28 / D-29).

## EP-1 — extension-point roster + "Rust engine plugin" naming (§2.7 / §6.3)

Benten has two extension categories (CLAUDE.md baked-in #18 + #19), recognized
+ named here per **EP-1** (recognition + naming + ~zero-code doc formalization;
**no registry**, **trust unchanged**; do NOT mint `EngineExtension` /
`ExtensionRegistry`):

- **"Rust engine plugin"** is the canonical name for the **CLAUDE.md baked-in
  #19** category (the qualifier is always present). Unqualified **"plugin"**
  means a **#18 graph plugin** (an app-level subgraph plugin); **"engine
  extension"** is a retained synonym for "Rust engine plugin".
- **Symmetric two-plugin model:** the engine ships a default roster of each kind.
- **The open-backend seam roster (Tier-1):** `KVBackend` / `BlobBackend` /
  `GraphBackend` / `Renderer` / `Transport` / `Materializer`. **IVM strategy is
  an ENUM (`benten_ivm::Strategy`), NOT a trait seam** (baked-in #2 —
  enum-dispatch, not a backend trait).
- **3 openness tiers:** (1) **open backend seam** (the roster above) / (2)
  **sealed policy seam** (`CapabilityPolicy`, the #830-locked `GrantReader`,
  `DeviceAuthBackend`) / (3) **enum-dispatch crypto** (the codepoint-dispatch
  `match` arm + reserved IANA-disjoint codepoint per #5).

## HPKE KEM-extensibility (NQ-C1) — the Benten-supplies-the-KEM resolution

At v1-beta the Layer-C `0x647a` X-Wing KEM is implemented as **Benten's own
KEM-DEM** over vetted upstream primitives (`ml-kem` + `x25519-dalek` + `sha3`
for the X-Wing SHA3-256 combiner, which directly derives the wrap key — there is
no separate HKDF key-schedule on this path; `chacha20poly1305` for the DEM) —
the **"Benten-supplies-the-KEM"** branch. The
`rozbb/rust-hpke` crate's KEM roster is effectively closed to the
RFC-9180-registered KEMs (a custom X-Wing KEM is not a first-class extension
point there), so X25519MLKEM768 does **not** plug into that crate's KEM trait
as a first-class registered KEM. The dependency-pinning **posture** (McMillion
`hpke` over Cryspen `hpke-rs`) is recorded in `docs/SECURITY-POSTURE.md`
Compromise #39 as the standing choice for the HPKE key-schedule/AEAD surface.

## Gossip topic (§3.9, unlabelled) vs AAD set-id commitment (§3.10, labelled) — do not conflate

Two distinct blinded constructions key off `K_Set`; they are **not** the same
bytes and must not be conflated:

- **§3.9 — the iroh-gossip topic (UNLABELLED):**
  `blake3::keyed_hash(K_Set, set_id ‖ BE(generation))`. This is the broadcast
  rendezvous topic (Compromise #61 closure); it carries NO label, and binds the
  set-generation directly. It is the authoritative gossip-topic derivation.
- **§3.10 — the AAD `membership_set_id_commitment` (LABELLED):**
  `blake3::keyed_hash(K_Set, "benten:setid:v1" ‖ membership_set_id)` (truncated
  to 32 bytes), bound INSIDE the `0x6610` group-AAD field-set. This is a
  labelled domain-separated commitment used as an authenticated-not-encrypted
  AAD field — see `docs/SECURITY-PROOFS.md`.

Both are BLAKE3 keyed-MACs over `K_Set` (BLAKE3-keyed IS a native MAC;
`benten-membership-set` carries no hmac/sha2 dep), but the **domain separation
differs** (the gossip topic is unlabelled + binds the generation; the AAD
commitment is labelled with `"benten:setid:v1"`). The nonce-cache replay-defense
spec (`jti`-keyed, durable, §3.10 / NQ-T4) is documented in
`docs/THREAT-MODEL.md` (NQ-T4) + `docs/SECURITY-POSTURE.md` Compromise #64 — it
is ORTHOGONAL to both of these set-id constructions.

## Cross-references

- `docs/SECURITY-PROOFS.md` — the FROZEN AAD field-sets (`0x6510` / `0x6610` /
  `0x6520`) + the per-stanza-LIVE decomposition.
- `docs/THREAT-MODEL.md` — the trust-tier × operation matrix + the O-6
  blast-radius ladder + the network-observer-only unlinkability scoping.
- `docs/SECURITY-POSTURE.md` — the named Compromise table (#30–#64) +
  disposition-class index.
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` — Rows D-28 / D-29 (compute / economic
  PHASE-LATER-DEFER).
- `docs/INVARIANT-COVERAGE.md` — Inv-16..22.
