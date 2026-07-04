# benten-membership-set — Internals

A plain-English, code-grounded tour of the `benten-membership-set` crate — the **15th workspace crate** added in Phase-4-Meta-Core (F-full; the MembershipSet keying primitive). Read-only audit. Audience: a developer landing in this crate fresh and trying to find the load-bearing seams. Last authored: 2026-07-04 (R17 doc-integrity fix-pass F-30) against the current HEAD; claims grep-verified against `src/`.

---

## 1. What this crate does

`benten-membership-set` is a **thin keying-glue Rust engine plugin** (the *mechanism-half* per GN-2 / R0 §6.1). It owns the frozen Rust-mechanism minimum needed to KEY a MembershipSet — the group / device-mesh / single-device keying primitive that Layer-C group encryption (`benten_drop::layer_c::group_posture`) uses.

The dividing line is **mechanism-half (here, Rust) vs data-half (graph Nodes)**:

- **Mechanism-half (this crate, frozen Rust):** the `MembershipSetKind` enum + its codepoint band, the multi-stanza-HPKE keying glue, the `members_table` canonical-CBOR snapshot, the per-Kind cardinality constructors, the `0x6610` group AAD assembly, the RBAC `RoleId` ordinal + ability templates, the role-staleness verify gate, and the Inv-21 fork-tie-break comparator.
- **Data-half (graph, NOT here):** `GovernanceConfig`, `RoleId` permission *semantics* (UCAN templates as data), Garden/Grove sub-config, the members-table relation + derived member-nature (IVM views), the audit log, federation `SubsetRef` links, and compute/economics.

Every crypto primitive is DELEGATED to `benten-crypto-suite` (the CLAUDE.md baked-in #5 ONLY-call-site; this crate **NEVER forks** a primitive). The AAD assembler hands **OPAQUE `Vec<u8>`** across the crypto-suite seam — the crypto-suite never sees this crate's types (m-15 GNC-5; no reverse dependency edge).

**Native-only** per CLAUDE.md baked-in #17: the B-1 dep set includes `benten-sync` (iroh + Loro, native-only), so a `#[cfg(target_arch = "wasm32")] compile_error!` fires on any wasm32 build. Thin-client / browser surfaces receive the *materialized* membership-set snapshot via the D-PHASE-3-30 thin-client protocol, NOT the in-bundle keying crate.

### The kind / member / role / AAD model

- **`MembershipSetKind` (`kind.rs`)** — the SCALE / keying axis, **EXACTLY 3** ordinal-stable variants: `Atrium = 0` (a community of user-DIDs, ≥1 admin), `DeviceMesh = 1` (a user's own device set, exactly-1 user-DID admin), `SingleDevice = 2` (a single local self-admin device). Deliberately **NOT** `#[non_exhaustive]` — every `match` must be exhaustive so a 4th arm is a §15.c HALT-AND-SURFACE compile error at every dispatch site (the discriminants are wire-keying-load-bearing). All 3 Kinds bind to the single `0x6600` set-keying codepoint (`kind::codepoint()` is a wildcard-free exhaustive match). Garden/Grove are the orthogonal GOVERNANCE axis, never Kinds.
- **`MemberEntry` / `MembersTable` (`member.rs`)** — the fused per-DID record: one DID → exactly one record (a `BTreeMap<Did, MemberEntry>`). `MemberEntry = { role, is_authority, sig_pubkey, admitted_at_hlc, member_ref }`. The coupling rule is `is_authority ⟹ sig_pubkey.is_some()`. There is **ZERO nature field** (Inv-22 — see below). `MemberRef` is the KEYING/FEDERATION axis reference, **Kind-determined** (homogeneous per Kind), NOT a per-member nature discriminator (m-15 GNC-7).
- **`RoleId` (`role.rs`)** — the RBAC axis, a **5-value ordinal** serialized as its `u8` (AAD-keying-bound, golden-vector-pinned): `Invitee = 0` (pre-acceptance handshake only; derives ZERO content — no `K(N)`, no read cap; M-11 zero-content floor), `Viewer = 1` (read-only), `Member = 2` (read / write-own / share-within-policy), `Moderator = 3` (+ moderate-content), `Admin = 4` (Moderator ∪ the 5 admin-exclusive governance abilities per Ben ruling 2, strict Moderator ⊊ Admin). `RoleId::ALL` ships all 5 active (Inv-20 clause-j).

### K_Set keying

- **`K(V)` / `K(N)` derivation (`keying.rs` + `keying_kv.rs`)** — `K(V) = blake3::derive_key("benten-membership-set:K(V):v1", cid)` (the version-node key) and `K(N)` (the per-Node content key) use the BLAKE3 KDF under DISTINCT context labels so the two derivations never collide. `derive_member_key(node_cid)` produces `K(N)`. `keying_kv::derive_kv(CidTarget)` is the typed `K(V)` boundary that type-REJECTS a mutable Anchor CID (Inv-19 register-then-enforce; property-pinned `F-INV19-1`, zero production callers at HEAD — wired with the engine encrypt-to-recipient path, Row D-64).
- **BLINDED gossip topic (`keying.rs`)** — the iroh-gossip rendezvous topic is `truncate_32(blake3::keyed_hash(K_Set, membership_set_id ‖ BE(generation)))` with **NO time input** (so every current-generation member derives the SAME topic from any wall-clock). The keyed-preimage SHAPE is the domain separator — it is deliberately NOT a `domain_registry` tag (F-GOSSIP-2 / §3.9; closes the #61 social-graph-leak via HMAC-blinding).

### RBAC + the AAD

- **`0x6610` group per-stanza AAD (`aad.rs`)** — `assemble_group_aad` is the canonical-TLV encoder for the R0.7 §3.10/§4.1 **BLINDED 11-field set** (big-endian, length-injective). It HONORS Sealed-Sender: the raw recipient roster + raw set-id are BLINDED into two 32-byte commitments (`audience_set_commitment`, `membership_set_id_commitment = blake3::keyed_hash(K_Set, "benten:setid:v1" ‖ id)`) — the plaintext AAD carries neither raw form. The inline `body_cid` is a self-describing CIDv1 pinned at exactly **36 bytes** (`0x01 0x71 0x1e 0x20 ‖ 32-byte BLAKE3`), asserted at the assembly site so a wrong-width CID cannot shift downstream field boundaries (U3 length-injectivity). The `AAD_VERSION = 0x01` prefix byte leads the AAD (distinct from the envelope `ENVELOPE_FORMAT_VERSION`). `canonical_members_table_bytes` serializes the members-table snapshot as the AAD-bound keying minimum (NQ-W4).

### The codepoints it owns (`0x6600` band)

The MembershipSet band is `0x6600..=0x66FF` (Inv-18 / NQ-W2 FROZEN-band ownership; `codepoints.rs`):

- **`0x6600` `MEMBERSHIP_SET_ENCRYPTION`** — the set-keying envelope codepoint (Sealed-Sender by default; §4.0 RELOCATED from the M-CONS-FINAL `0x6380` that collided MLS-Application). Every `MembershipSetKind` binds to this value.
- **`0x6610` `MEMBERSHIP_SET_GROUP_MULTI_STANZA`** — the group multi-stanza per-stanza AAD codepoint (R0.7 §3.10/§4.1; the value `assemble_group_aad` binds — NOT the `0x6600` set-keying value).
- **`0x6620` `MEMBERSHIP_SET_RESERVED_0X6620`** — RESERVED for a future wire shape (the `SubsetRef` federation shape; additive over the crypto-agility framework, never a wire break; reserved-and-REFUSED / typed-reject at v1-beta).

### Inv-22 — member-nature is derived, never stored

There is NO `MemberEntry` nature field, NO Policy nature field, NO wire nature slot (`member_type` is DELETED). Member-nature is DERIVED at read time: `is_ai_operated(did) = (did.method() == "agent")` (`derive_member_nature` / `is_ai_operated` in `member.rs`; `did:agent:` is an optional allowlist alias, NOT a stored discriminator). The ZERO-stored-nature-field rule is ENFORCED by the **exhaustive-destructure struct-fence** in `tests/f_ms_3_members_table_fusion.rs` (`let MemberEntry { role, is_authority, sig_pubkey, admitted_at_hlc, member_ref } = &e;` with NO `..` rest — adding a nature field breaks the pattern at compile time). The standalone `derive_member_nature` helpers themselves have **zero production callers** at HEAD (R14 F-10); the LIVE nature answer is the IVM-materialized derived view over `(did_method, has_install_manifest)` at the engine + graph layer.

---

## 2. Dependency chain

**Workspace deps (in) — B-1 set, all UPSTREAM:** `benten-crypto-suite` (the ONLY crypto-primitive call site per #5; the AAD assembler hands OPAQUE `Vec<u8>` across this seam), `benten-core` (Node / Version-chain / CID), `benten-caps` (UCAN / grants — RBAC composition), `benten-id` (DID + DeviceAttestation + RotationLog), `benten-graph` (audit-sequence + `ChangeEvent`), `benten-sync` (HLC / Loro / MST / Transport — native-only), `benten-errors` (the `E_ROLE_STALE_AT_VERIFY` catalog mint).

**External deps:** `blake3` (project hash + keyed-MAC for the AAD commitments + gossip topic), `serde` + `serde_bytes` + `serde_ipld_dagcbor` (the canonical-CBOR `members_table` snapshot encoder), `thiserror`. **NO crypto primitive** (`sha3` / `ml_kem` / `chacha20` / `ml_dsa` / `x25519`) is constructed here — all routed via crypto-suite (#5 ONLY-call-site; the F-CRATE-2 grep pin enforces this).

**No reverse edge (F-CRATE-2 compile-fence):** none of `{crypto-suite, core, caps, id, graph, sync}` depend on THIS crate — the direction is strictly `membership-set → {…}`. The crate is a leaf-ish addition.

**Features:** `default = []`; `testing = []` (gates the `pub fn .*_for_test*` constructors per the V1-FROZEN-INTERFACE item-15 convention, matching benten-crypto-suite / benten-core / benten-caps).

---

## 3. Files inventory in `src/` (~2.8k LOC across 16 files)

- **`lib.rs`** — crate-level doc (mechanism-half-vs-data-half + the register-then-enforce honest-disclosure narrative). The `#[cfg(wasm32)] compile_error!` native-only guard. Pub module declarations + re-exports (`MembershipSetKind`, `MemberEntry`, `MembersTable`, `RoleId`, `MembershipSet`, etc.). A `scaffold` marker module retained from the R3-W4 scaffold (proves the 15th crate exists; the authoritative constants live in `codepoints`).
- **`kind.rs`** — the EXACTLY-3 `MembershipSetKind` enum (`VARIANT_COUNT = 3`, `ALL`, wildcard-free `codepoint()` / `ordinal()`), the `dispatch_reserve` reserve-Kind typed-reject, `KindDispatchError` / `RequestedReserveKind`.
- **`codepoints.rs`** — the `0x6600..=0x66FF` band constants (`MEMBERSHIP_SET_ENCRYPTION` / `MEMBERSHIP_SET_GROUP_MULTI_STANZA` / `MEMBERSHIP_SET_RESERVED_0X6620` + band bounds).
- **`member.rs`** — `MemberEntry` (the 5-field fused record; ZERO nature field — Inv-22) + `MembersTable`, `Did`, `Hlc` (the `admitted_at_hlc` member-property clock, distinct from Inv-21 `created_at_hlc`), `SigPubKey`, `MemberRef` (Kind-determined), `MemberNature` + `derive_member_nature` / `is_ai_operated` (derived; zero production callers).
- **`role.rs`** — the 5-value `RoleId` ordinal + `grants_read_cap()` + the per-role UCAN ability-template sets.
- **`aad.rs`** — `assemble_group_aad` (the `0x6610` BLINDED 11-field canonical-TLV AAD) + `canonical_members_table_bytes` (NQ-W4 snapshot) + `AAD_VERSION` / `SETID_COMMITMENT_LABEL` / `SELF_DESCRIBING_CID_LEN = 36`.
- **`keying.rs`** — the multi-stanza-HPKE keying glue: `K(V)` / `K(N)` KDF context labels, `derive_member_key`, and the BLINDED gossip-topic construction (§3.9 / Compromise #61).
- **`keying_kv.rs`** — the typed `K(V)` `derive_kv(CidTarget)` boundary (Inv-19; type-rejects a mutable Anchor CID; property-pinned, zero production callers).
- **`set.rs`** — `MembershipSet` per-Kind cardinality constructors (`new_atrium` / `new_device_mesh` / `new_single_device` + `construct`), `wire_cost_ceiling`, and the `crdt` submodule: `admitted_at_hlc_lww_keeps_a` (property LWW = LARGER-HLC-wins), and the **Inv-21 fork tie-break** `fork_total_order_key` / `fork_a_wins` (SMALLER `(created_at_hlc, fork_event_version_node_cid)` key wins = oldest-anchor-wins, the DELIBERATE opposite of property LWW; antisymmetric via the fork-event Version-Node CID; zero production callers → Row D-52 Composing merge path).
- **`governance.rs`** — `GovernanceTier` / `GovernanceConfig` (data-half models) + `MembershipSetPolicy` (the zero-sized sealed-policy fence / mechanism-half boundary marker, NOT a runtime enforcer). Zero production callers (register-then-enforce; LIVE authority is engine + capability-policy).
- **`federation.rs`** — the offline `K_Set` acquisition-path model (`KSetAcquisitionPath::verify_offline` depth/cycle decidability, `to_wire_v2_be`) + `FederationError` / `FederationModel` + the `SubsetRef` / `0x6620` reserved-and-REFUSED typed-reject at v1-beta (Inv-20 clause-k/l).
- **`verify.rs`** — the role-staleness verify gate (`verify_stanza` → `E_ROLE_STALE_AT_VERIFY`); a data-half model with zero production callers (the LIVE role-staleness enforcement is the `benten_drop::layer_c` open-side recompute).
- **`ucan.rs`** — ephemeral per-role UCAN grants.
- **`error.rs`** — the `MembershipSetError` typed enum (per-Kind admin-cardinality violations, Kind↔MemberRef coupling, etc.) with `error_code()` mapping to the workspace `benten_errors::ErrorCode` catalog + the `E_ROLE_STALE_AT_VERIFY` mirror constant (§3.5g Rust↔TS↔catalog lock-step).
- **`privacy.rs`** — the unlinkability scope-honesty predicates (`network_observer_can_link_stanzas` / `admin_can_correlate_members`) backing the F-NAT-2 disclosure-coherence arms (network-observer-only unlinkability; admin CAN correlate — Compromise #58, asserted explicitly so the claim is not over-claimed).

---

## 4. Invariants owned + honest disclosures

This crate is the enforcement home for the **MembershipSet arc, Inv-19..22**:

- **Inv-19** — encryption-substrate keying-function CRDT-input discipline (`derive_kv` type-rejects a mutable Anchor CID). **Register-then-enforce carve-out:** AS-BUILT + property-pinned (`F-INV19-1`), zero production callers → wired with the engine encrypt-to-recipient path (Row D-64).
- **Inv-20** — clause-c (group AAD), clause-j (ship-all-5-roles-active), clause-k/l (federation recursion-bound / `SubsetRef` refusal), clause-d (unlinkability scope). Enforced.
- **Inv-21** — the fork-tie-break comparator (`fork_a_wins`, SMALLER-key / oldest-anchor-wins). **Register-then-enforce carve-out:** AS-BUILT + proptest-pinned, zero production callers (the live distributed merge is benten-sync LWW at `crdt.rs:535`) → Composing merge path (Row D-52).
- **Inv-22** — member-nature is derived, never stored (the exhaustive-destructure struct-fence + DID-method parse; §1 above).

The `governance` / `role` / `verify` / `audit` model surfaces are RETAINED for property-pinning but have **zero production callers at HEAD** — the LIVE enforcement is engine-layer (the same register-then-enforce honest-disclosure idiom Inv-19 / Inv-21 use; R9-council F-01..F-05, R14 F-10).

---

## 5. v1-FROZEN-INTERFACE.md coupling

This crate's public surface IS frozen as part of **V1-FROZEN-INTERFACE.md §16** (the 15th crate's MembershipSet frozen-surface table). The cargo-public-api baseline at `docs/public-api/benten-membership-set.txt` is the byte-stable v1-beta surface; drift fails CI. The `0x6600`/`0x6610`/`0x6620` codepoint band + the `0x6610` group AAD field-set are pinned in `docs/CRYPTO-CODEPOINTS.md §4.0` + `docs/V1-WIRE-FORMAT-INVENTORY.md` item 25.
