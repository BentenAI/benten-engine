# Invariant Coverage — Phase 4-Foundation Close + Phase-4-Meta-Core Inv-15 Mint + F-full Inv-16..22 (AS-BUILT)

CLAUDE.md commits to **23 invariants** governing the Benten engine
(the CLAUDE.md-committed set spans **Inv-1..Inv-23**: 14 from Phase
4-Foundation + Inv-15 minted at Phase-4-Meta-Core per
Ben ratification 2026-05-26: "sig-bundle CIDs are never load-bearing
identifiers" — the application-layer 3-layer decomposition that closes
the LAMPS Composite ML-DSA EUF-CMA-only construction-scope per L12
finding + the cryptographer-review-of-bird-of-prey-vs-lamps elegant
permanent shape — and Inv-16..22 design-minted at Phase-4-Meta-Core for
the F-full encryption + identity + MembershipSet substrate, REGISTERED
here per the §9.1-7 doc-wave gate with enforcement-completion on the
F-full wave path, matching the Inv-15 register-then-enforce precedent;
**+ Inv-23 minted at Phase-4-Meta-Core GAP-KDB Shape-B closure** — "a
Layer-C seal's KEM key is committed by its audience DID", the
recipient-side freeze-surface analogue of Inv-15, AS-BUILT + ENFORCED
via `Did::resolve_kem` + the `RecipientBinding` sole-constructor
typestate).
This document tracks per-invariant enforcement state, the enforcing
crate, and the regression suite that pins it.

**Phase 4-Foundation status:** 14 of 14 Phase-4-Foundation invariants enforced. Phase-4-Foundation extends Inv-14 with the plugin-DID principal classifier (see `Inv-14 Phase-4-Foundation plugin-DID principal extension` sub-section below) — the principal-type matrix now spans User-local + User-sync-merged + Device-multi-device-sync + Plugin-app-level-subgraph + Plugin-via-materializer-read.

**Phase-4-Meta-Core status:** Inv-15 REGISTERED (in this doc + linked in CLAUDE.md baked-in #5) but **NOT-YET-FULLY-ENFORCED-BY-AUTOMATION** at HEAD. Existing payload-CID discipline at the load-bearing surfaces (`Engine::revoke_capability_by_grant_cid` + plugin `manifest_cid`, per ground-truth-verify 2026-05-26 Q1+Q2 favorable) means the SUF-CMA-equivalent property already holds structurally at those surfaces today; the cross-surface read-audit (UCAN backend revocation + device attestation envelope + Atrium Drop bundles + sync merge proofs + EMIT event envelopes) is **COMPLETE at HEAD** — all eight surfaces ✓ VERIFIED at R6-reround, and the UCAN revocation surface was re-keyed in code onto the signature-EXCLUSIVE `ucan_payload_cid` (F-03), so `revoke` / `is_revoked` take `&Ucan` and a sig-inclusive identifier is structurally impossible to key revocation on; focused would-FAIL-on-revert pins landed at `tests/inv15_sig_malleability_does_not_change_identifier.rs`. What the G-CORE-PQ-WIRE-1 wave still bundles is the FULL per-surface MallorySigner property-test cluster (MallorySigner-generated valid-but-different-bytes sigs on the same payload MUST leave identifiers unchanged) + the `cite-drift-detector` scanner extension (`LoadBearingSigBundleCidPattern` flagging `*_by_*_cid` callers whose arg sources include sig bytes) + the pim-N codification. **Inv-16..22 are AS-BUILT + ENFORCED at HEAD, with FOUR honest carve-outs — Inv-19 + Inv-21 (register-then-enforce: built + property-pinned, zero production callers), Inv-22 (a forward per-consumer-derivation-consistency discipline the struct-fence backstops structurally), and the RBAC admin-op authorization-enforcement deferral — plus the Inv-20 clause-level disclosure below:** the F-full encryption + identity + MembershipSet substrate SHIPPED — `benten-crypto-suite` (Inv-16/17/18), the 15th crate `benten-membership-set` (Inv-19/20/21/22), and `benten-engine/src/layer_d` (Layer-D device-wraps) all exist and are green, with real typed errors + landed regression suites (no longer planned/design-time). **Inv-19 carve-out:** the `K(V)` keying-glue boundary (`derive_kv(CidTarget)`) is AS-BUILT + property-pinned (`F-INV19-1`) but has **zero production callers at HEAD** (the test-driven `derive_kv` is the PRODUCTION-stand-in); production keying-path wiring is **deferred with the engine encrypt-to-recipient wiring** (`docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-64). **Inv-21 carve-out:** the fork-tie-break COMPARATOR (`fork_total_order_key` / `fork_a_wins`) is AS-BUILT + property-pinned (proptest), but it has **zero production callers at HEAD** — the LIVE merge path is benten-sync LWW (`crdt.rs:535`) and the comparator's wiring into the live distributed concurrent-fork merge path is **deferred to Phase-4-Meta-Composing** (`docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-52), matching the Inv-15 register-then-enforce precedent. **Inv-20 clause-level disclosure:** the AAD / fusion / RoleId / federation clauses (c, i, j, k, l) are enforced at HEAD with typed paths + landed pins, but clauses **a/b/e/g/h** (the `K_Set` / `K(N)` / gossip keying-glue) are **golden-pinned-not-live** — `keying::gossip_topic` has zero production callers and `governance::promote_tier` unconditionally returns `k_set_rotated: false`. Same honest disclosure Inv-19 / Inv-21 get; production wiring rides the Composing wave (`docs/V1-FROZEN-INTERFACE-DEFERRED.md` observation O-08). Net: 14 of 15 fully-enforced via Phase-4-Foundation discipline; Inv-15 partially-enforced-via-existing-discipline + on the G-CORE-PQ-WIRE-1 enforcement-completion path; Inv-16/17/18/22 enforced at HEAD (and Inv-20 for clauses c/i/j/k/l) with the per-invariant construction sites + test cites in the table below; Inv-19 keying-glue-AS-BUILT-and-property-pinned with production keying-path wiring on the Row D-64 path; Inv-21 comparator-AS-BUILT-and-proptest-pinned with production-merge-wiring on the Composing path. Inv-4 + Inv-7 went
ACTIVE in Phase 2b alongside the SANDBOX runtime (registration arm
landed in G7-B; runtime arm landed across waves 8b + 8h with a bounded
honest-disclosure for Inv-4 — see the "Inv-4 + Inv-7 runtime arm
status" section below). Phase 3 extended Inv-13 with the row-4 SPLIT
classifier (user-zone vs system-zone divergent-CID handling at the
sync-receive boundary; `crates/benten-sync/src/crdt.rs` +
`crates/benten-engine/tests/inv_13_dispatch.rs`) and widened Inv-14
with three additive sync-boundary attribution slots (`peer_did_set` /
`device_did` / `sync_hop_depth`); the on-the-wire device-DID
attestation envelope (G16-D wave-6b) makes Inv-14 device-grain
attribution **LOAD-BEARING under adversarial-peer assumptions** — see
the "Inv-14 Phase-3 G16-B device-grain extension" section below for
the full retense.

---

## Coverage table

| # | Invariant | Phase | Enforcer | Tests |
|---|-----------|-------|----------|-------|
| 1 | DAG-ness — no cycles in operation graphs | 1 | `benten-eval::invariants::structural::validate_subgraph` (Kahn cycle detect via `find_cycle` / `find_cycle_indices`) | `crates/benten-eval/src/invariants/structural.rs` (cycle test cluster) |
| 2 | Max operation-subgraph depth | 1 | Bounded longest-path walk + per-CALL increment | `structural.rs::depth_*` tests |
| 3 | Max fan-out per node | 1 | Edge enumeration at registration | `structural.rs::fan_out_*` tests |
| 4 | **SANDBOX nest-depth ceiling — ACTIVE (Phase 2b; both arms wired at R6FP-G1 / PR #62)** | 2b | `invariants::sandbox_depth::validate_registration` (registration); `AttributionFrame.sandbox_depth` runtime threading in `crates/benten-engine/src/primitive_host.rs::execute_sandbox` (parent-sandbox_depth+1) + `SandboxError::NestedDispatchDepthExceeded` fires in `crates/benten-eval/src/primitives/sandbox.rs::execute` (runtime arm — both arms now active) | `crates/benten-eval/tests/inv_4_runtime_arm_fires_at_max_depth.rs`, `crates/benten-eval/tests/sandbox_depth_inheritance_regression.rs`, `crates/benten-eval/tests/sandbox_attribution_frame_security.rs` |
| 5 | Max total nodes per subgraph | 1 | Node-count gate at registration | `structural.rs::node_count_*` tests |
| 6 | Max total edges per subgraph | 1 | Edge-count gate at registration | `structural.rs::edge_count_*` tests |
| 7 | **SANDBOX `output_max_bytes` range — ACTIVE (Phase 2b; PRIMARY+BACKSTOP)** | 2b | `invariants::sandbox_output::validate_registration` (registration); `CountedSink::write` (PRIMARY streaming) + `CountedSink::backstop_check` (return-value BACKSTOP), both wired through the host-fn trampoline + primitive boundary | `crates/benten-eval/tests/sandbox_output.rs`, `crates/benten-eval/tests/proptest_sandbox_output.rs`, `crates/benten-eval/tests/integration/inv_7_streaming.rs`, `crates/benten-eval/src/sandbox/counted_sink.rs` |
| 8 | Multiplicative cumulative budget (CALL × ITERATE) | 2a | `invariants::budget` + `BudgetTracker` per evaluator step | `crates/benten-eval/src/invariants/budget.rs` (proptest cluster) |
| 9 | Determinism — handlers declared deterministic reject non-determinism sources | 1 (decl) / 2a (rt) | `structural::validate_subgraph` declaration check + runtime fence | `structural.rs::determinism_*` |
| 10 | Canonical byte encoding (order-independent DAG-CBOR) | 1 | `structural::canonical_bytes` order-independence proptest | `structural.rs::canonical_bytes_*`, `crates/benten-dsl-compiler/tests/dsl_compiler_round_trips_5_primitive_fixtures.rs::dsl_compiler_round_trip_preserves_subgraph_spec_cid_across_compile_serialize_compile` (DSL emission-path canonical-bytes anchor) |
| 11 | System-zone reserved-prefix reject — user code cannot READ/WRITE system labels | 2a | `invariants::system_zone` (G5-B-i) + Engine::put_node_with_context dispatch | `crates/benten-engine/tests/inv_11_*.rs` |
| 12 | Aggregate validation catch-all — multi-invariant violations roll up | 1 | `InvariantViolation::Registration` (at `crates/benten-eval/src/lib.rs::InvariantViolation` — fires when two or more invariants violate simultaneously) | `crates/benten-eval/tests/invariants_9_10_12.rs::registration_catch_all_populates_violated_list` |
| 13 | Immutability — User WRITE re-puts of an already-persisted CID fire `E_INV_IMMUTABILITY` | 2a | `invariants::immutability` + `WriteAuthority` firing matrix | `crates/benten-engine/tests/inv_13_*.rs` |
| 14 | Causal attribution — every primitive frame carries an `AttributionFrame` (Phase-3 G16-B device-grain extension: `peer_did_set` + `device_did` + `sync_hop_depth` slots — see "Inv-14 Phase-3 G16-B device-grain extension" below) | 2a / 3 | `evaluator::attribution` runtime threading + `ATTRIBUTION_PROPERTY_KEY` registration check + `crates/benten-engine/src/engine_sync.rs` sync-merge frame construction (G16-B) | `crates/benten-eval/tests/attribution_*.rs` (glob matches frame_shape + non_regression + sandbox + invariant_14 files), `crates/benten-engine/tests/hlc_attribution_frame.rs` + `sec_r6r1_01_inv_14_attribution_threading_preserved_under_g12_c.rs` + `sync_replica_attribution.rs` + `resume_with_missing_attribution_triple_rejects.rs` + `resume_with_tampered_attribution_rejected.rs`, plus the G16-B sync-merge round-trip suite |
| 15 | **Sig-bundle CIDs are never load-bearing identifiers — REGISTERED (Phase-4-Meta-Core; enforcement-completion at G-CORE-PQ-WIRE-1)** | 4-Meta-Core | Existing discipline at load-bearing surfaces: `crates/benten-engine/src/engine_caps.rs::Engine::revoke_capability_by_grant_cid` (looks up `system:CapabilityGrant` Node by Node-content-addressed CID — labels + properties only; sig sidecar excluded) + `crates/benten-platform-foundation/src/plugin_manifest.rs::manifest_cid` (computed-then-signed; consent record signs over `(manifest_cid \|\| ...)`). G-CORE-PQ-WIRE-1 brief mandates: cross-surface audit + per-surface MallorySigner property tests + cite-drift-detector `LoadBearingSigBundleCidPattern` scanner | At HEAD: existing payload-CID-discipline tests + ground-truth-verify 2026-05-26 (Q1+Q2 favorable) + the LANDED cross-surface pins at `crates/benten-engine/tests/inv15_sig_malleability_does_not_change_identifier.rs` (R6-reround; live `#[test]`s, zero `#[ignore]`) covering the device-attestation sig-EXCLUSIVE `SigInput` projection + the MST payload-CID recompute guard, with the Drop-bundle and EMIT surfaces pinned in their own crates. Still planned at G-CORE-PQ-WIRE-1: the FULL per-surface MallorySigner property-test cluster + the `cite-drift-detector` `LoadBearingSigBundleCidPattern` scanner + the pim-N codification — see "Inv-15 Phase-4-Meta-Core mint + 3-layer decomposition" section below |
| 16 | **Envelope-layer unification — AS-BUILT + ENFORCED (Phase-4-Meta-Core)**. ONE codepoint-dispatched envelope serves all four layers (Layer-A vault / Layer-B per-Node AEAD / Layer-C encrypt-to-recipient drops / Layer-D device-wraps + remote-permission), with: codepoint-dispatch (U1) + AAD-binding + strict-decode no-silent-fallback (U2) + canonical-TLV length-injective encoding (U3) + sender-DID-bound-or-Sealed-Sender + replay-window — primitive-neutral (one HPKE primitive reused, not re-implemented per layer). | 4-Meta-Core | `crates/benten-crypto-suite/src/aead.rs` (`AeadEnvelope` typed codepoint dispatch + `aad_per_chunk`/`aad_per_recipe`/`aad_whole_content` AAD-binding, big-endian per M-19) + `crates/benten-crypto-suite/src/cipher_suite.rs` (codepoint dispatch + typed-reject) + `crates/benten-engine/src/layer_d/` (Layer-D device-wraps reuse the one envelope) — the single `#5` crypto call site. **OBS hygiene note (F-04):** the LIVE shared primitive at HEAD is `AeadEnvelope` / `CipherSuite` / hpke (cited here); the typed `EncryptedEnvelope` / `canonical_tlv_encode` / `strict_decode` surface (`benten-crypto-suite/src/envelope.rs`, the M-18 typed-`BindingContext` lift) is the NOT-YET-ADOPTED typed layer — vault + Layer-C reference it but the production seal/open path runs on the flat `AeadEnvelope` shared primitive, so this row's cite carries no over-claim. | `crates/benten-crypto-suite/tests/f_inv16_1_envelope_unification.rs` (cross-layer U1–U3 parametric + one-HPKE-path-reused), `f_cp_codepoint_registry_dispatch.rs` (strict-reject no cross-variant fallback), `f_w0_envelope_v2_migration.rs` (`EncryptedEnvelope`/`BindingContext` V2/BE migration), `f_lc_*` (HPKE mode_base round-trip in `crates/benten-drop/tests/`), `crates/benten-membership-set/tests/f_aad_2_nine_tuple_injectivity_opaque_boundary.rs` (AAD injectivity + opaque-bytes boundary; **the `nine_tuple` FILENAME is stale** — the file pins the BLINDED 11-field set, per its own body; the rename is tracked at `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-32) |
| 17 | **Hybrid-cryptography-mandatory floor — AS-BUILT + ENFORCED (Phase-4-Meta-Core)**. Every KEM use-site is PQ⊕classical (X25519⊕ML-KEM-768); **no pure-PQ codepoint is LIVE or selectable** at v1-beta or v1-GM. Reserved-named-typed-rejected swap-matrix arms (e.g. `PURE_PQ_MLKEM768_ONLY` at `0x647c`) are permitted for conformance only and remain audit-gated (m-3). The classical half is the audited floor so unaudited PQC is never the SOLE trust path; the combiner is **strip-resistant** (both shared secrets `ss_M ‖ ss_X` are mixed into the KEK — neither KEM half can be stripped without changing the derived key; full AEAD key-commitment in the robustness sense is OUT OF SCOPE at v1-beta). ANSSI/BSI/NIST SP 800-227 §4.4 aligned. | 4-Meta-Core | `crates/benten-crypto-suite/src/cipher_suite.rs` (codepoint dispatch + X-Wing-style combiner = `SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel)`; `0x647c` typed-reject on the default path, audit-flag gated) | `crates/benten-crypto-suite/tests/f_sm_inv17_hybrid_floor_swap_matrix.rs` (hybrid floor; `0x647c` default-path → `Err`), `tf2_hybrid_both_must_verify_strip_resistant.rs` + `tf2_strip_resistance_negative.rs` (strip-resistance: zeroing PQ-half OR classical-half fails), `tf4_gcore3c_full_swap_matrix_strip_resistance_pure_pq_nondefault.rs` + `tf4_pure_pq_gated_audit_landed.rs` (swap-matrix coverage + `0x647c` audit-gate) |
| 18 | **Codepoint-registry discipline + metadata-disclosure invariant + `CodepointLifecycle` typed-state — AS-BUILT + ENFORCED (Phase-4-Meta-Core)**. All crypto codepoints live in `CRYPTO-CODEPOINTS.md`, occupy the Benten-owned `0x6100..0x6FFF` band, are intra-band non-colliding + IANA-disjoint (CI scanner enforced), and traverse the `Live → Deprecated → Quarantined → Burned` lifecycle (Quarantined/Burned reject). Metadata-disclosure clause: any plaintext-sender-DID-in-AAD variant MUST disclose at SECURITY-POSTURE.md AND have a paired Sealed-Sender sibling — **satisfied by Sealed-Sender being the DEFAULT** (the `0x6610` MembershipSet group path + the Layer-C drop path are Sealed-Sender by default). | 4-Meta-Core | `crates/benten-crypto-suite/src/cipher_suite.rs` (codepoint registry + dispatch) + `crates/benten-membership-set/src/codepoints.rs` (`0x6600`/`0x6610`/`0x6620` band ownership) + the IANA-disjoint/non-collision CI scanner (NQ-W2) | `crates/benten-crypto-suite/tests/f_cp_codepoint_registry_dispatch.rs` (registry dispatch + burned-cp → `Err`), `crates/benten-drop/tests/f_lc_abuse_control_group_posture_and_inv18.rs` (paired-Sealed-Sender; `observable_metadata == {audience}`) |
| 19 | **Encryption-substrate keying-function CRDT-input discipline (Path-A.5) — API-boundary AS-BUILT + property-pinned; zero production callers at HEAD (register-then-enforce disclosure, matching Inv-21/Inv-15) (Phase-4-Meta-Core)**. The per-structural-path key-derivation function `K(V)` at its API boundary type-restricts its keyed payload to an **immutable Version-Node-CID** (or a MembershipSet) — encrypting key material to a mutable/Anchor CID is a **RUNTIME typed-reject at the `derive_kv` sole front-door** (`Err(KvError::TargetNotImmutable)` / `E_KV_TARGET_NOT_IMMUTABLE`), NOT a compile-time/type-level impossibility — the `CidTarget::MutableAnchor` variant stays publicly constructible and the BLAKE3-KDF context label is public (`keying::KV_DERIVE_CONTEXT`, plus its `benten_crypto_suite::domain_registry` mirror), so the guarantee is enforced by routing through `derive_kv`, not by the type system. This keeps the encryption substrate's keying inputs content-stable so a fork's key derivation is reproducible from immutable CRDT inputs. **Honest disclosure (register-then-enforce):** the typed `derive_kv(CidTarget) -> Result<_, KvError>` boundary is AS-BUILT at HEAD and property-pinned by `F-INV19-1`, but it has **zero production callers at HEAD** — the test-driven `derive_kv` is the PRODUCTION-stand-in; wiring the keying-glue into the live encrypt-to-recipient / MembershipSet keying path is **deferred with the engine encrypt-to-recipient wiring** (`docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-64), matching the Inv-15 / Inv-21 register-then-enforce precedent. The type-restriction reject is frozen at Core. | 4-Meta-Core | `crates/benten-membership-set/src/keying_kv.rs` (the `K(V)` keying-glue API boundary; rejects a mutable Anchor CURRENT CID) + `benten-core` Anchor/Version/CURRENT version-chain (immutable Version-Node-CIDs) | `crates/benten-engine/tests/f_inv19_1_kv_type_restriction_immutable_version_node_cid.rs` (`F-INV19-1`: `derive_kv` ACCEPTS an immutable Version-Node-CID, REJECTS a mutable Anchor CID with a typed `TargetNotImmutable` error, ACCEPTS a MembershipSet target — the `K(V)` reject path; would-FAIL = encrypting to a mutable/Anchor CID) + `crates/benten-membership-set/tests/f_inv21_fork_tie_break_totality_version_node_cid.rs` (`F-INV21-4`: key → immutable-Version-Node-CID under fork) |
| 20 | **MembershipSet primitive invariant — 12 clauses (a–l) — AS-BUILT, with a clause-level enforcement split (Phase-4-Meta-Core)**. **ENFORCED at HEAD: clauses c/i/j/k/l** (AAD / fusion / RoleId / federation — typed paths + landed pins). **Golden-pinned-not-live: clauses a/b/e/g/h** (the `K_Set` / `K(N)` / gossip keying-glue — `keying::gossip_topic` has zero production callers and `governance::promote_tier` unconditionally returns `k_set_rotated: false`); production wiring rides the Composing wave (`docs/V1-FROZEN-INTERFACE-DEFERRED.md` observation O-08). Clause (f) is the Inv-19 `K(V)` boundary and carries the Inv-19 register-then-enforce carve-out (Row D-64). See the Inv-20 clause-level disclosure in the preamble for the authoritative breakdown. Clause text: (a) K_Set established via multi-stanza-HPKE-Encap; (b) FORK-ONLY key rotation; (c) the `0x6610` group-AAD field-set = the BLINDED **11-field set** (incl. `role_assignments_generation`); (d) per-recipient unlinkability is **network-observer-only** (NOT against a malicious admin — m-7); (e) generation-CRDT; (f) Path-A.5 `K(V)` to immutable Version-Node-CIDs; (g) TransportConfig + gossip; (h) per-member `K(N)` walk-scope; (i) per-DID `MemberEntry` fusion (one DID → one record; `BTreeMap<Did, MemberEntry>`); (j) **5-value RoleId all-5-active** (Invitee=0 … Admin=4; corrected from "3 active 2 reserved" — M-13) + retention; (k) federation recursion-bound (`MEMBERSHIP_RECURSION_MAX_DEPTH = 4` + PATH-CARRIED offline cycle-detect — M-9; `SubsetRef` reserved-and-refused at v1-beta at `0x6620`); (l) Model-B independent-`K_Set`-per-set default (Model-A opt-in is post-v1-beta additive). | 4-Meta-Core | `crates/benten-membership-set` (the 15th crate — EXACTLY-3 `MembershipSetKind` (`src/kind.rs`) + `members_table` CBOR (`src/aad.rs::canonical_members_table_bytes`) + per-Kind constructors (`src/set.rs`) + 0x6610 group-AAD 11-field assembly (`src/aad.rs::assemble_group_aad`) + clause-k bound (`src/federation.rs`); delegates primitives to `benten-crypto-suite`, NEVER forks; depends UP on `benten-sync` for CRDT/HLC/MST/transport) | `crates/benten-membership-set/tests/f_ms_2_constructor_cardinality_memberref.rs` + `f_ms_3_members_table_fusion.rs` (clauses a, i — per-Kind cardinality + one-DID-one-record fusion), `f_aad_1_members_table_canonical_cbor_length_injective.rs` + `f_aad_2_nine_tuple_injectivity_opaque_boundary.rs` (clause c — canonical-CBOR length-injective + 11-field AAD injectivity; **the `nine_tuple` FILENAME is stale** — rename tracked at `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-32), `f_ms_4_5_roleid_ordinal_and_invitee_zero.rs` (clause j — RoleId 5-value ordinal golden-vector), `f_fed_1_2_subset_ref_federation.rs` (clauses k, l — federation depth-4 + Model-B default), `f_inv21_fork_tie_break_totality_version_node_cid.rs` (clause f — key→immutable-Version-Node-CID) |
| 21 | **MembershipSet-fork-tie-break HARD partition — comparator AS-BUILT + property-pinned (proptest); production merge-path wiring deferred to Phase-4-Meta-Composing (Phase-4-Meta-Core; the same register-then-enforce disclosure Inv-15 uses, M-7/M-8)**. On a set-identity fork, **SMALLER `created_at_hlc` wins** (oldest-anchor-wins) — **DELIBERATELY OPPOSITE** to the in-tree LARGER-HLC-wins property LWW at `crates/benten-sync/src/crdt.rs:535` (M-7). The tie-break is **TOTAL** via the forking-event **Version-Node CID** when `created_at_hlc` ties (`MembershipSetId` cannot disambiguate concurrent same-anchor forks — M-8); total order `(created_at_hlc ASC, fork_event_version_node_cid ASC)` (`src/set.rs::fork_total_order_key` + `fork_a_wins`). The losing fork's CRDT-vector MUST NOT merge into the winner (no silent absorption); it is **archived-not-discarded** (the archival-half comparator is pinned by `F-INV21-4`). A later adversarial re-fork with `u64::MAX` HLC never displaces the original. **Honest disclosure (register-then-enforce):** the comparator (`fork_total_order_key` / `fork_a_wins`) is AS-BUILT at HEAD and property-pinned (totality/antisymmetry/transitivity + asymmetry + archival-half) by the `F-INV21-*` proptest family; **the LIVE merge path is benten-sync LWW (`crdt.rs:535`)**, and the comparator has **zero production callers at HEAD** (the test-local `resolve_fork` in `f_inv21_*.rs` is the PRODUCTION-stand-in). Wiring the comparator into the live distributed merge path — the concurrent same-anchor set-creation fork scenario — is **deferred to Phase-4-Meta-Composing** (the distributed concurrent-fork resolution path), tracked at `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-52. The comparator + proptest are frozen at Core. | 4-Meta-Core | `crates/benten-membership-set/src/set.rs` (the fork-tie-break CRDT rule = `fork_total_order_key`/`fork_a_wins`) + `crates/benten-sync` (HLC ordering `crdt.rs`, Loro merge, MST anti-entropy `mst.rs` = the convergence backstop, iroh `Transport`) | `crates/benten-membership-set/tests/f_inv21_fork_tie_break_totality_version_node_cid.rs` (`F-INV21-1` smaller-`created_at_hlc`-wins asymmetry — naive-LWW-passing test FAILs; `F-INV21-2` totality antisymmetry/transitivity/totality pins; `F-INV21-3` convergence proof — `#[cfg(kani)]`-gated `#[kani::proof]` arm with a **proptest surrogate as the v1-beta floor** (kani is a v1-GM strengthening, NOT a v1-beta dependency); `F-INV21-4` losing-fork MUST-NOT-merge + archived-not-discarded), `f_hlc_1_2_admitted_vs_created_hlc_and_skew.rs` (`F-HLC-1` `admitted_at_hlc` LWW vs `created_at_hlc` fork-stamp distinction), `f_crdt_membership_set_convergence.rs` (`F-CRDT-3` LWW-property ∥ fork-set-identity co-existence) |
| 22 | **Member-nature is derived, never stored — AS-BUILT + ENFORCED (Phase-4-Meta-Core)**. There is NO `MemberEntry` nature field, NO Policy nature field, NO wire nature slot; **`member_type` is DELETED**. Member-nature is DERIVED at read time: `is_ai_operated(did) = (did.method() == "agent")` (`did:agent:` is an optional allowlist alias, NOT a stored discriminator); `is_plugin` / `is_autonomous_ai` derive from Inv-14 attribution / manifest; ownership derives from the Inv-14 attribution chain. Any cached nature flag is a derived (recomputed) view, never authoritative. `MemberRef` is **Kind-determined** (UserDid↔Atrium / DeviceDid↔DeviceMesh / LocalDevice↔SingleDevice) — it is NOT a nature discriminator (m-15 GNC-7). In-tree precedent: Inv-14 (attribution derived, not stored as authority). | 4-Meta-Core | `crates/benten-membership-set/src/member.rs` (`MemberEntry` carries no nature field; `derive_member_nature`/`is_ai_operated` from DID-method parse). **F-12 (R12) enforcement attribution:** the ZERO-nature-field rule is enforced by the **exhaustive destructure** `let MemberEntry { role, is_authority, sig_pubkey, admitted_at_hlc, member_ref } = &e;` (NO `..` rest) in `f_ms_3_members_table_fusion.rs` — adding a nature field breaks the pattern at compile time. The `MemberEntry::has_any_nature_field() -> bool { false }` const is a NON-load-bearing NAMING helper (a literal `false`; it cannot itself detect a new field), NOT the enforcement site. | `crates/benten-membership-set/tests/f_ms_3_members_table_fusion.rs` (**the exhaustive-destructure struct-fence — the load-bearing Inv-22 pin**; `is_ai_operated` from method-parse; not-writable-as-authoritative), `f_ms_2_constructor_cardinality_memberref.rs` (`MemberRef` Kind-determined boundary, NOT nature), `f_gossip_transport_placement_and_blinded_topic.rs` (unlinkability scope-honesty — network-observer-only; admin CAN correlate, asserted explicitly so not over-claimed) |
| 23 | **A Layer-C seal's KEM key is committed by its audience DID — AS-BUILT + ENFORCED (Phase-4-Meta-Core, GAP-KDB Shape-B closure)**. The recipient side becomes symmetric to the already-self-certifying sender side: the audience `did:benten` *commits* the recipient's key-set document (including the KEM key) by CID, so the KEM key is recovered-and-VERIFIED **from the DID** — `Did::resolve_kem(keyset_doc)` fail-closed rejects unless (a) `self_describing_cid(BLAKE3-256(canonical(keyset_doc)))` equals the DID's committed CID (the 2nd-preimage bound — the flagship active-substitution defense), (b) the doc's embedded `sig` multikey equals the DID's embedded signing key (spliced-sig reject), (c) `kem_cp` cross-checks the wired `{0xec, 0x120c}` components (algorithm-confusion reject), and (d) the committed suite is at/above the PQ floor (`0x6400` classical-only refused). The type-level guarantee is the `RecipientBinding` **sole-constructor** typestate (private fields, `resolve()` the ONLY constructor, NO `(kem_pub, audience_did)` fallback door) — the substitutable two-param seal API is DELETED, so no code path can seal to an un-committed `(KEM, DID)` pair and a `did:benten` recipient cannot be downgraded back to the un-cross-checked path. Unforgeability reduces to BLAKE3-256 2nd-preimage resistance over canonical DAG-CBOR (safe indefinitely). Native to Inv-15 (identity = canonical-payload-CID) and §3.5s (cross-ecosystem-identifier-as-content). The *invariant* freezes at v1-beta; the exact `RecipientBinding` type-name may evolve. **Honest residual:** this closes post-first-contact key substitution but NOT first-contact DID-authenticity (Compromise #67, TOFU/bind-once). | 4-Meta-Core | `crates/benten-id/src/did.rs` (`Did::resolve_kem` — CID 2nd-preimage + doc.sig cross-check + kem_cp⟺components + PQ-floor; the `did:benten` key-set commitment) + `crates/benten-drop/src/layer_c.rs` (`RecipientBinding` sole-constructor typestate + the single/group seal API taking `&[RecipientBinding]`) | `crates/benten-id/tests/kdb_resolve_kem_fail_closed.rs` (**RK-2 the flagship resolver-layer active-substitution 2nd-preimage reject**; RK-3 spliced-sig / RK-4 kem_cp⟺components / RK-5 PQ-floor / RK-7 bare-did:key→NoKemCommitment), `crates/benten-drop/tests/kdb_drop1_7_gapkdb_seal_flagship.rs` (**DROP-1 seal-to-substituted-key fails closed across 0x6510/0x6520/0x6610 + DROP-7 Inv-23 real production-driving firing**), `crates/benten-drop/tests/kdb_drop2_sole_constructor.rs` (DROP-2 sole-constructor / no-fallback-door) |

---

## Inv-4 + Inv-7 runtime arm status (honest disclosure)

Phase 1 shipped Inv-4 + Inv-7 as **stubs** because the SANDBOX primitive
itself was compile-check only (Compromise #4 in
`docs/SECURITY-POSTURE.md`). Phase 2b G7-B added the registration-time
arms; waves 8b + 8h wired the runtime executor end-to-end. R6FP Round-1
Group-1 (PR #62, 3-lens convergent fix) closed the remaining transitive
threading gap. Both runtime arms are fully active at Phase 2b close:

- **Inv-7 (SANDBOX `output_max_bytes` range)** — **fully active at
  runtime.** The wave-8b host-fn trampoline routes every host-fn
  byte-emit through `CountedSink::write`'s `OutputCheckPath::PrimaryStreaming`
  arm; the primitive boundary runs `CountedSink::backstop_check` against
  the return value (`OutputCheckPath::ReturnBackstop`). Per D17 PRIMARY +
  BACKSTOP. Per-handler ceiling per D15. Default ceiling 1 MiB;
  `SandboxArgs.outputLimitBytes` overrides per-call. Note: the
  `invariants::sandbox_output::check_admission` helper exists and is
  unit-tested but is NOT the production firing site — `CountedSink`
  enforces the same arithmetic directly via `SinkOverflow` →
  `SandboxError::OutputOverflow` mapping. Both paths produce
  `E_INV_SANDBOX_OUTPUT` typed errors with identical context shapes.

- **Inv-4 (SANDBOX nest-depth ceiling)** — **both arms fully active at
  Phase 2b close.** (1) Registration arm: `validate_registration` walks
  the static-graph at registration time. (2) Runtime arm: R6FP-G1 (PR
  #62) wired the `AttributionFrame.sandbox_depth` threading through the
  parent `ActiveCall`. At every production SANDBOX entry,
  `crates/benten-engine/src/primitive_host.rs::execute_sandbox` mutates
  the parent frame via `frame.sandbox_depth = frame.sandbox_depth.saturating_add(1)`;
  the dispatching `AttributionFrame` is constructed with `sandbox_depth:
  nested_depth` in both match arms of the same function so subsequent
  CALL pushes inherit. The eval-side runtime arm in
  `crates/benten-eval/src/primitives/sandbox.rs::execute` fires
  `SandboxError::NestedDispatchDepthExceeded` once `attribution.sandbox_depth
  > config.max_nest_depth`. Default `max_nest_depth = 4` admits depths
  1..=4; depth 5 fires. SANDBOX-inside-CALL-inside-SANDBOX inherits the
  parent's depth correctly. Residual (retensed): the ESC-10
  adversarial integration test (`sandbox_escape_attempts_denied.rs::sandbox_escape_reentrancy_via_host_fn_denied`)
  is LIVE (`#[test]`, un-ignored at G20-A1 wave-8a — the
  `docs/future/phase-3-backlog.md` §7.3.A.7 destination is CLOSED) and
  drives the `testing_call_engine_dispatch` helper into the typed
  `EscapeAttempt` reject. It stays SIMULATION-driven rather than a true
  nested-dispatch driver, because no production host-fn re-enters
  `Engine::call` (D19-RESOLVED).

Both invariants fire as `E_INV_SANDBOX_DEPTH` (Inv-4) and
`E_INV_SANDBOX_OUTPUT` (Inv-7) error codes — both pinned in
`docs/ERROR-CATALOG.md`. The catalog rows now reflect the per-arm
honest disclosure.

The Phase-1 "Phase 2b" stubs that previously appeared in this table
have been removed; Inv-4 + Inv-7 are now first-class active rows.

---

## Where each invariant is enforced

```
┌────────────────────────────────────────┬─────────────────────────────────────────┐
│ Registration-time (one-shot)           │ Runtime-time (per-call, per-frame)      │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ Inv-1 DAG-ness                         │ Inv-4 sandbox_depth runtime counter     │
│ Inv-2 max depth                        │ Inv-7 sandbox_output CountedSink        │
│ Inv-3 fan-out                          │ Inv-8 BudgetTracker step gate           │
│ Inv-4 sandbox_depth declaration        │ Inv-13 WriteAuthority firing matrix     │
│ Inv-5 node count                       │ Inv-14 AttributionFrame propagation     │
│ Inv-6 edge count                       │                                         │
│ Inv-7 sandbox_output declaration       │                                         │
│ Inv-9 determinism declaration          │                                         │
│ Inv-10 canonical-bytes order-indep     │                                         │
│ Inv-11 system-zone literal-CID reject  │                                         │
│ Inv-12 aggregate roll-up               │                                         │
│ Inv-14 ATTRIBUTION_PROPERTY_KEY decl   │                                         │
└────────────────────────────────────────┴─────────────────────────────────────────┘

- Inv-4 runtime counter — fully wired at R6FP-G1 (PR #62). Both
  registration arm + runtime arm are active at Phase 2b close. See
  §"Inv-4 + Inv-7 runtime arm status" above for the wiring trace.
```

---

## IVM Algorithm B production registration (audit-gap closure note)

Wave-8h closed the IVM Algorithm B production-registration drift the
docs-vs-code audit caught: `Engine::create_user_view` previously
forced `ContentListingView` for every `Strategy::B`-declared user
view. Post-wave-8h the dispatch constructs `AlgorithmBView::for_id(spec.id())`
for the **5 canonical view IDs** that `AlgorithmBView` supports
natively (the hand-written single-loop dispatch in
`crates/benten-ivm/src/algorithm_b.rs`).

**Phase-3 G15-A + G15-B + R5 wave-9 W9-T1 closure — Algorithm B
generalized at Phase 3 G15-A.** Algorithm B is no longer
canonical-only; the prior canonical-view-fallback compromise is
RETIRED. User-defined views run under `Strategy::B` with their actual
label patterns rather than being coerced to `ContentListingView`
semantics. `Algorithm::register(view_id, label_pattern, projection)`
(and the budget-aware sibling `Algorithm::register_with_budget`)
instantiates a generic single-loop kernel
(`benten_ivm::algorithm_b::GenericKernel`) for non-canonical view IDs
keyed on `(label_pattern, projection)`. The genuine `AnchorPrefix`
selector lift (post-G15-A) ships in `register_user_view`; the
kernel-side guard refuses canonical-id + AnchorPrefix registrations
with the typed `AlgorithmError::CanonicalIdAnchorPrefixRefused`
variant (mirrored at the engine boundary as
`EngineError::ViewLabelMismatch`). The drift-detector proptest harness
at `crates/benten-ivm/tests/algorithm_b_drift_detector.rs` (5 pins,
1 000 cases each) drives the merged `Algorithm::register` surface
end-to-end + reports incremental-vs-rebuild parity.

---

## Inv-14 Phase-3 G16-B device-grain extension

Phase-3 G16-B widens `AttributionFrame` with three additive sync-boundary
slots so causal attribution carries device-grain + peer-grain provenance
across CRDT merges:

- **`peer_did_set: Option<BTreeSet<Did>>`** — `Some(set)` when the frame
  originates from a Loro CRDT merge; captures contributing peer DIDs
  observed via `benten_sync::crdt::LoroDoc::winning_attribution`. `None`
  for purely-local writes. The peer-node-id → DID resolution lives in
  `crates/benten-engine/src/engine_sync.rs` against the local trust
  store.
- **`device_did: Option<Did>`** — device-grain attribution per the
  D-PHASE-3-25 device-heterogeneity contract. `None` for legacy / local
  writes; `Some(did)` for sync-attributed or device-DID-attested writes.
  Lets multi-device users (laptop ↔ phone-OS-app ↔ desktop, per
  commitment #17) distinguish per-device origins inside a single
  per-user Atrium.
- **`sync_hop_depth: u32`** — bounded merge-hop counter (default cap
  `SYNC_HOP_DEPTH_CAP = 8`, mirrors Inv-4's sandbox-depth precedent).
  Increments at each CRDT merge hop; the typed
  `ErrorCode::SyncHopDepthExceeded` fires at the merge seam when a
  merge would push the depth past the cap.

**Phase-2a CID stability preserved.** All three slots elide from the
canonical Node encoding when default (`None` / `0`) via
`serde(default, skip_serializing_if = ...)`. A purely-local frame
canonicalises to the exact Phase-2a 3- / 4-key Node and produces the
pinned schema-fixture CID
(`bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a`); any
non-default value adds the slot and produces a distinct CID — the
content-addressing security claim is "a sync-bearing attribution chain
is content-distinguishable from a purely-local chain."

**Construction discipline.** Test / bench / legacy callsites use
`AttributionFrame { ..Default::default() }` to spread the new slots;
production callsites in `engine_sync.rs` populate the sync slots
explicitly at the CRDT merge seam. The `Default` impl is intentionally
test-/bench-shaped (all-zero CIDs); production paths construct frames
explicitly per the WRITE-path discipline.

**Phase-3 G16-B-prime engine-side merge callback (§6.12 item 1
closure).** The engine's `apply_atrium_merge` orchestration entry
point composes the structural surface with the in-memory anchor
store: after `AtriumHandle::merge_remote_change_with_hop_depth`
returns a `SyncMergeAttribution` seed, the engine resolves
peer node-ids → peer-DIDs via `AtriumHandle::resolve_peer_dids`
(local trust-store lookup with `node-id:NNN` fallback), constructs an
`AttributionFrame` populated with `peer_did_set` / `device_did`
(from `Engine::device_cid`) / `sync_hop_depth`, and mints a new
"version" Node via `Engine::append_version` against the named
anchor. The Anchor's CURRENT pointer advances atomically via the
prior-threaded `benten_core::version::append_version` discipline.

**Phase-3 G16-B-prime device-DID threading (§6.12 item 3 closure).**
`Engine::set_device_cid` configures the engine's
device-DID-attestation CID; the engine's two production WriteContext
construction sites (`engine_diagnostics.rs::transaction` commit hook
+ `primitive_host.rs::check_capability`) populate
`WriteContext.device_cid` from this setter so heterogeneous
`CapabilityPolicy` impls can dispatch per-device under the SAME
logical-actor identity per D-PHASE-3-25.

**⚠️ SUPERSEDED-BY-COLLAPSE (refinement-audit-2026-05 S3, owner-ratified
2026-05-15 — see `docs/SECURITY-POSTURE.md` Compromise #23).** Under the
ratified trust-model reframe (DECISION-RECORD-trust-model-reframe.md §4),
**device-DID is a provenance label on the unified user-root-anchored
capability spine, NOT a distinct trust-root.** Inv-14 device-grain
attribution is **retained as an audit/provenance property** — the
`AttributionFrame.device_did` slot is still populated and still
cryptographically attributable via the retained envelope provenance-binding
signature (a peer still cannot forge another device's DID without that
device's key). What COLLAPSE deletes is the *trust decision* machinery
(`Acceptor`/`accept_at`/`DeviceRevocation`); the *trust* now flows through
the single chain-validation seam plus one retained envelope-ceiling
attenuation. The device-grain provenance / compromised-device-quarantine
audit trail survives; only the parallel device-trust-root pipe is removed.
A post-COLLAPSE successor audit (#1234) confirms every remaining
device-grain attribution use is elegant under the unified model. The
Phase-3 narrative below is preserved for historical accuracy (it was
correct for the model as it stood at Phase-3 close).

**Phase-3 G16-D wave-6b on-the-wire device-DID-attestation envelope
(plan §1 exit-criterion 16 closure) — cryptographic-attestation closure
at G16-D wave-6b fix-pass.** The handle binds a signed
`benten_id::DeviceAttestation` (parent → device-DID binding) +
device-keypair via `AtriumHandle::set_local_device_attestation` +
`AtriumHandle::set_local_device_keypair`; `sync_subgraph` +
`accept_sync_subgraph` emit a DAG-CBOR `DeviceAttestationEnvelope`
(V2 shape: `(version, attestation, payload_hash, session_nonce,
envelope_signature)`) BEFORE the Loro CRDT export on each leg. The
receiver-side `DeviceAttestationEnvelope::verify` enforces three
defenses cryptographically: (1) the envelope signature verifies
against the public key resolved from `attestation.device_did` (DID
forgery defense — a peer cannot impersonate another device's DID
without holding that device's secret key); (2) the embedded
attestation passes `benten_id::Acceptor::accept_at` (parent signature
+ freshness window + nonce-store replay defense + revocation list);
(3) `BLAKE3(received_payload) == envelope.payload_hash` via
constant-time comparison (frame-pair binding — MITM cannot swap
envelope/payload pairs). All three failure modes reject with
`E_DEVICE_ATTESTATION_FORGED` (`ON_DENIED` routing).
`Engine::apply_atrium_merge` populates `AttributionFrame.device_did`
from the verified wire envelope's declared DID (preferred) and falls
back to the local engine's `device_cid` only when no envelope was
received (legacy V1 / pre-G16-D peer / direct-test path that bypasses
`sync_subgraph`).

**Inv-14 device-grain attribution is now LOAD-BEARING under
adversarial-peer assumptions** (was advisory at PR #163 V1 shape; the
fix-pass closes the cryptographic gaps so the device-grain provenance
defense survives forged-DID / replay / frame-pair-swap attacks). The
"compromised device cannot be quarantined surgically" failure shape is
now defended at the cryptographic boundary, not via cooperating-peer
assumptions. Pinned end-to-end at:

- `tests/integration/atrium_two_device.rs::atrium_two_device_same_identity_selective_zone_sync`
  (multi-device GREEN-path with REAL signed attestations).
- `tests/integration/atrium_two_device.rs::forged_device_did_rejected_at_envelope_verify`
  (DID forgery rejection — `E_DEVICE_ATTESTATION_FORGED`).
- `tests/integration/atrium_two_device.rs::replayed_stale_envelope_rejected_by_freshness_window`
  (stale-envelope replay rejection via the freshness window re-homed onto the
  unified spine per Compromise #23 SUPERSEDED-BY-COLLAPSE; the accept-time
  nonce-store was deleted with the Acceptor cluster — durable replay-marker
  re-home tracked P2/P5 per DECISION-RECORD §4b F3).
- `tests/integration/atrium_two_device.rs::frame_pair_payload_swap_rejected_by_payload_hash_binding`
  (BLAKE3 frame-pair binding violation rejection).
- `tests/integration/atrium_two_device.rs::future_wire_version_rejected_at_decode`
  (decode-time version validation; closes cryptography MINOR-5).

The legacy unsigned shape (V1 `attestation = None`) is preserved for
backward-compat with pre-G16-D-fp peers + the two pre-existing
pinned-CID fixtures (`sync_replica_attribution_carries_device_did_alongside_parent`
+ `sync_replica_explicit_actor_cid_decouples_from_device_cid`) that
bypass the wire envelope path. See SECURITY-POSTURE.md Compromise #23
for the full closure narrative.

---

## Inv-14 Phase-4-Foundation plugin-DID principal extension

Phase-4-Foundation extends the principal-type matrix Inv-14 covers
without altering the device-grain LOAD-BEARING posture. App-level
plugins (CLAUDE.md baked-in #18) run their subgraphs under a freshly
minted `plugin_did` distinct from the user-DID and from any other
plugin's DID. The evaluator's read pathway threads the active
principal via `Engine::read_node_as(principal, cid)` (Class B β
SHIPPED at PR #184); writes attributed to a plugin carry the
plugin-DID in `AttributionFrame.actor_cid`.

The matrix Inv-14 must cover post-Phase-4-Foundation:

| Principal type | actor_cid carries | Authorization seam |
|----------------|-------------------|--------------------|
| User (local) | user-DID | `CapabilityPolicy::pre_write` |
| User (sync-merged) | user-DID | per-row recheck inside `apply_atrium_merge` |
| Device (multi-device sync) | user-DID + `device_did` | `Acceptor::accept_at` + `DeviceAttestationEnvelope::verify` |
| Plugin (app-level subgraph) | plugin-DID | `CapabilityPolicy::pre_write` + `manifest_envelope_chain_validation` (Layer-2 envelope + Layer-3 UCAN delegation) |
| Plugin via materializer-read | plugin-DID | `MaterializerEngine::read_node_as` + `MaterializerCapRecheck` (dual-gate per sec-3.5-r1-1) |

No new ErrorCode is required for the plugin-principal extension —
`E_CAP_DENIED` covers the deny path uniformly; the layer that denied
is observable via the cap-chain trace. The `manifest_envelope_chain_validation`
seam (`crates/benten-caps/src/manifest_envelope_chain_validation.rs`,
G24-D-FP-2) joins manifest-envelope-shape enforcement with the UCAN
chain validator without introducing a sixth-class principal type at
the evaluator boundary.

The R4b-FP-1 Seam 3 `apply_atrium_merge` envelope-recheck-seam (post-Q4
ratification 2026-05-13) is tracked as **Compromise #26 (Phase-4-Foundation
manifest-envelope recheck on merge boundary) — PARTIALLY CLOSED** — see
[`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) "Compromise #26" for the
full seam-vs-adapter narrative (the `ManifestEnvelopeRechecker` port +
default-flip ship; the production-default `NoopManifestEnvelopeRechecker`
returns `NotApplicable` for every row at HEAD, so the substantive Layer-2
defense is NOT live in shipped binaries yet; the
`ProductionManifestEnvelopeRechecker` adapter TYPE + its
`ProductionEngineBuilder` wiring LANDED at Phase-4-Meta-Core (Row D-4
CLOSED at R6 R1 FP-F4 §S4) and returns `UnresolvedDeny` for synthesized
peer-DIDs, while its substantive per-DID `PluginLibrary` chain walk is
what remains deferred to Phase-4-Meta-Composing / G-COMP-1 per
`docs/future/phase-4-backlog.md §4.36`). Inv-14 doesn't gain a new
device-grain slot; the recheck (when live) happens AFTER the per-row
cap-revocation check + before the AttributionFrame is constructed on the
receiver side, so the frame remains the invariant's source of truth.

---

## Inv-15 Phase-4-Meta-Core mint + 3-layer decomposition

**Origin**: surfaced by senior cryptographer review of LAMPS Composite ML-DSA vs Bird-of-Prey for v1-beta default (`.addl/phase-4-meta/cryptographer-review-bird-of-prey-vs-lamps.md`, branch `phase-4-meta-core/cryptographer-review-bird-of-prey @ 36afe06b`). Ben ratification 2026-05-26 ("all yes across the board"): LAMPS Composite ML-DSA at codepoint `0x0001` as v1-beta default + Inv-15 framework + G-CORE-PQ-WIRE-1 bundles audit/hardening.

**Statement**: Every CID-based identifier in Benten refers to a canonical PAYLOAD, never a sig-inclusive bundle. Revocation, dedupe, audit-uniqueness, and any property depending on signature-uniqueness MUST key off either (a) the payload-CID or (b) a semantic tuple — never off the sig-bundle-CID.

**Why it matters**: LAMPS Composite ML-DSA is EUF-CMA-only (NOT SUF-CMA) per `draft-ietf-lamps-pq-composite-sigs-19` §9.2.2 ("NOT RECOMMENDED for use in applications where it has not been shown that EUF-CMA is acceptable") + provides only Weakly-Non-Separable per draft §10 (NOT Strongly-Non-Separable). For systems that key revocation, dedupe, or audit-uniqueness off signature bytes, EUF-only signatures admit malleability bypasses — an attacker holding a valid `(payload, sig)` could in principle mint `(payload, sig')` (different bytes, same payload, both verify) and observe different behavior wherever sig-CID was load-bearing. Inv-15 closes this hazard architecturally — independent of construction choice — by mandating payload-CID identity + semantic-tuple revocation. Net: SUF-CMA-equivalent application-layer security despite EUF-CMA-only construction-layer scope.

**3-layer decomposition** (the elegant permanent shape this invariant enforces):

| Layer | Decision space | Benten today |
|---|---|---|
| **Identity** | What canonical bytes uniquely name "this thing" | payload-CID (Node bytes / canonical-manifest bytes / canonical-UCAN-claims bytes) |
| **Authentication** | Who attests + by which sig algorithm | codepoint-dispatched signature (LAMPS at `0x0001`; agility seam per CLAUDE.md baked-in #5) |
| **Revocation** | "This thing no longer authorizes X" | semantic tuple `(issuer, subject, cap, audience, validity)` — NOT sig-bundle-CID |

Each layer changes orthogonally; you can upgrade authentication (e.g. add Bird-of-Prey-class SUF-CMA-preserving codepoint at `0x0002` when WG-adopted + impl-audited) without touching identity or revocation; you can refine revocation semantics without touching identity or authentication; you can extend identity without touching either of the others.

**R6-R2 phase-close re-surface (GAP-4; council run `wf_040ac861-436`, artifact `d0ccf606`, 2026-06-06):** the R6-R2 post-F-full phase-close council re-surfaced this as GAP-4 — "Inv-15 REGISTERED but NOT-YET-FULLY-ENFORCED-BY-AUTOMATION." Disposition = NAMED-CARRY (HARD-RULE clause-(b)): the gap is correctly carried to the **G-CORE-PQ-WIRE-1 enforcement-completion path enumerated below** (this section IS that ledger). No new destination is minted — the four-part path (cross-surface audit + per-surface MallorySigner property tests + `cite-drift-detector` `LoadBearingSigBundleCidPattern` scanner + dispatch-conventions pim-N codification) is the registered closure. The two load-bearing surfaces verified favorable at 2026-05-26 (Q1+Q2) still hold the SUF-CMA-equivalent property structurally at HEAD; the systematic cross-surface enforcement is on the G-CORE-PQ-WIRE-1 critical path, NOT a v1-beta gate. This re-surface annotation keeps GAP-4 registry-discoverable against the §3.12 R7-equivalent audit walk.

**Enforcement-completion path** (G-CORE-PQ-WIRE-1 brief bundles all of these):

1. **Cross-surface audit** of every signature-touching surface to verify payload-CID discipline — **READ-AUDIT COMPLETE at R6-reround (d4d9db5a): all eight surfaces below are ✓ VERIFIED.** (The automated `cite-drift-detector` scanner (#3), the full MallorySigner property-test cluster (#2), and the pim-N codification (#4) remain on the G-CORE-PQ-WIRE-1 path; R6-reround added focused would-fail-on-revert pins for the read-audited surfaces — see #2.)
   - ✓ `Engine::revoke_capability_by_grant_cid` (Q1 verified 2026-05-26: Node-content-addressed; sig sidecar)
   - ✓ Plugin `manifest_cid` (Q2 verified 2026-05-26: computed-then-signed; consent record signs over `(manifest_cid || ...)`)
   - ✓ **UCAN backend revocation — RESOLVED at Phase-4-Meta-Core (F-03, R6-R1 fix-pass).** The durable revocation seam (`benten_caps::backends::ucan::{revoke, is_revoked}`) keys markers on the **signature-EXCLUSIVE payload CID** (`ucan_payload_cid` = BLAKE3 over the `Ucan` `claims` field alone), NOT the sig-inclusive `ucan_cid` (which remains the `g14b:grant:` store identity only). `revoke`/`is_revoked` take `&Ucan` and derive the payload CID internally, so a sig-inclusive identifier is structurally impossible to key revocation on. This closes the prior INV15-UCAN-CID hazard (R18): a malleated / re-randomized signature over the same claims yielded a distinct `ucan_cid` that dodged the marker; every signature encoding of the same claims now maps to the SAME payload-CID marker. Regression guard: `crates/benten-caps/tests/f03_revocation_payload_cid_sig_independent_regression.rs` (would-FAIL-on-revert of the re-key).
   - ✓ **ED25519 STRICT VERIFY — RESOLVED at Phase-4-Meta-Core (F-03, R6-R1 fix-pass).** The UCAN chain-walk verify (`validate_chain_at` → `benten_id::authority_verify::verify_authority_signature` → `benten_crypto_suite::sig::SignatureSuite::verify`, classical + hybrid arms) now Ed25519-`verify_strict`s the classical half — a non-canonical / malleable `S` scalar is REJECTED, so the classical-half malleability primitive is closed at verify time. This is defense-in-depth with the payload-CID revocation re-key above (the hybrid ML-DSA-65 half is randomized, so for hybrid `did:benten` issuers the payload-CID re-key is the load-bearing revocation defense; strict verify covers the classical half). The prior "benign / no v1-beta gate" disclosure is **RETIRED** — both hardenings landed at Core, so this is no longer a deferred residual. Regression guard: `crates/benten-id/tests/f03_ucan_verify_strict_rejects_malleable_sig_regression.rs` (would-FAIL-on-revert of `verify_strict`). Cite (N-07): the earlier `crates/benten-id/src/ucan.rs:704` reference pointed at a comment block; the live authority verify is the crypto-suite `SignatureSuite::verify` chokepoint reached via `authority_verify.rs`.
   - ✓ **Device attestation envelope V2 — VERIFIED (R6-reround pre-tag Inv-15 read-audit, d4d9db5a).** Both `issue_with_nonce` (`crates/benten-id/src/device_attestation.rs:230`) and `verify_signature_with` (`:324`) sign/verify over `CanonicalBytes::to_canonical_bytes`, whose `SigInput` projection (`:413-432`) EXCLUDES the `signature` field (only `device_did`/`parent_did`/`envelope`/`nonce`/`issued_at`), so a valid-but-different signature over the same payload verifies against the SAME bytes. NO attestation-CID is load-bearing: replay defense = a fresh 32-byte OS-CSPRNG `nonce`; revocation collapses to user-root UCAN revocation `benten_caps::revoke` (`:438-439`) — which keys on the signature-EXCLUSIVE `ucan_payload_cid` (the F-03 payload-CID re-key, already ✓ VERIFIED above), so the device-attestation revocation surface is cross-referenced to that verified UCAN CID surface and is NOT a distinct sig-keyed identifier.
   - ✓ **Sync merge proofs (CRDT log entries) — VERIFIED (R6-reround pre-tag Inv-15 read-audit, d4d9db5a).** `MstEntry.cid = MstCid::from_bytes(&payload)` (BLAKE3 over the value payload; `crates/benten-sync/src/mst.rs:217`); `apply_entries` (`:340-352`) re-hashes each `payload` locally and byte-compares against the declared `cid`, rejecting with `EntryCidByteMismatch` — a declared CID is never trusted. The Merkle root is computed over sorted `(key, cid)` content pairs (`:405-446`). CRDT LWW dedupe/resolution keys on `(property-key, HLC)` with `node_id` = the writing peer's HLC node-id (`crates/benten-sync/src/crdt.rs`) — no signature bytes anywhere in the identity/dedupe path. All sig-EXCLUSIVE.
   - ✓ **Atrium Drop bundles (multi-sig CBOR envelopes) — VERIFIED (R6-reround pre-tag Inv-15 read-audit, d4d9db5a).** The load-bearing identifiers are `spec_cid` (content CID of the `RestrictedScope`) + `audience` (DID CID) (`crates/benten-drop/src/bundle.rs:371-373`) — neither sig-inclusive. `envelope_sig`'s signing input zeroes `envelope_sig` + `issuer_verifying_key` + `content` before encoding (`crates/benten-drop/src/envelope_sig.rs:77-95`), and the envelope_sig is INTEGRITY-only (authority anchors to `auth_grant`, not the sig). On open, Layer-C RECOMPUTES `self_describing_cid(BLAKE3(recovered_body))` and fail-closes unless it equals the wire `body_cid` (the B2 content-splice guard, `crates/benten-drop/src/layer_c.rs:1126-1135`). No whole-bundle sig-inclusive CID keys any identifier.
   - ✓ **Subscription / EMIT event envelopes — VERIFIED (R6-reround pre-tag Inv-15 read-audit, d4d9db5a).** Subscription dedupe = a monotonic engine-assigned `max_delivered_seq: u64` (`seq > max_delivered_seq` ⇒ deliver + bump, else drop; `crates/benten-engine/src/engine_subscribe.rs:99-113`). `EmitEvent { channel: String, payload: Value }` carries NO id and NO signature (`crates/benten-engine/src/emit_broadcast.rs:42-49`). `ChangeEvent.cid` = the affected-Node content CID + a monotonic `tx_id: u64` (`crates/benten-graph/src/store.rs:485-498`). All sig-EXCLUSIVE.
2. **Per-surface MallorySigner property tests** at `crates/benten-engine/tests/inv15_sig_malleability_does_not_change_identifier.rs` — **the file has LANDED at HEAD** (R6-reround; live `#[test]`s, zero `#[ignore]`) carrying focused would-FAIL-on-revert pins for the surfaces reachable from the `benten-engine` test binary; the FULL per-surface cluster (one file per audited surface) is what remains on the G-CORE-PQ-WIRE-1 path. Property: for every audited surface, a MallorySigner generating valid-but-different-bytes sigs on the same canonical payload MUST leave the identifier unchanged + revocation behavior identical + dedupe behavior identical + audit-uniqueness preserved.
3. **cite-drift-detector scanner extension** at `tools/cite-drift-detector/src/`: add `LoadBearingSigBundleCidPattern` kind that flags functions matching `*_by_*_cid` whose parameter sources include signature bytes. Stops regression class at PR-time.
4. **dispatch-conventions pim-N codification**: "Future signed-data designs MUST 3-layer-decompose — every signed-data design proposal MUST explicitly answer (a) identity = payload-CID OR semantic-tuple? (b) authentication = which codepoint? (c) revocation = payload-CID-or-tuple keyed? Designs where identifier = sig-bundle-CID are REJECTED on Inv-15 grounds."

**Failure mode if violated**: a signature surface that keys identity or revocation off sig-bundle-CID admits the EUF-only malleability bypass enumerated in the L12 finding (sigstore/rekor-tiles #425 cross-system parallel; UCAN-revocation-bypass-via-fresh-CID example). The hazard is application-layer, not algorithm-layer — algorithm choice (LAMPS / Bird-of-Prey / draft-prabel) doesn't fix it; only the invariant does.

**Future-additive upgrade path**: when SUF-CMA-preserving constructions like Bird-of-Prey (Bossuat et al. EUROCRYPT 2026; IACR 2025/1844) or `draft-prabel-cfrg-suf-hybrid-sigs` mature + receive WG adoption + receive independent impl audit, Benten adds them as additive codepoints via the crypto-agility framework — pure additive upgrade, no wire-format break, no re-sign of historic content. Inv-15 remains the load-bearing architectural property regardless of construction choice; SUF-CMA-preserving constructions become a defense-in-depth additive layer.

**Cross-references**: `docs/SECURITY-POSTURE.md` Compromise on LAMPS EUF-CMA-only construction-scope (closure via Inv-15); CLAUDE.md baked-in #5 (crypto-agility refinement + LAMPS-default + Inv-15 reference); dispatch-conventions §3 (Future signed-data designs MUST 3-layer-decompose codification); `.addl/phase-4-meta/cryptographer-review-bird-of-prey-vs-lamps.md` (origin finding); `.addl/phase-4-meta/NIGHT-SHIFT-2026-05-26.md` LATE-AFTERNOON #1 ADDENDUM (ratification record).

---

## Inv-16..22 Phase-4-Meta-Core F-full design-mints (AS-BUILT + ENFORCED at HEAD)

**Origin**: the F-full encryption + identity + MembershipSet substrate (`.addl/phase-4-meta/f-full-r0-plan.md`
R0.7, frozen design plan). These seven invariants were design-minted to codify the architectural properties
the F-full Phase-4-Meta-Core waves enforce, and are now **AS-BUILT** — the substrate SHIPPED across TIER-1
(crypto-suite + Layer-C + Layer-D) / TIER-2 (ms-sync + gov-audit) / TIER-3 (crypto docs). The enforcing
crates exist in code at HEAD: `benten-crypto-suite` (Inv-16/17/18), the 15th crate `benten-membership-set`
(Inv-19/20/21/22), and `benten-engine/src/layer_d` (Layer-D device-wraps).

**Status semantics:** AS-BUILT + ENFORCED at HEAD for Inv-16/17/18/22 (and for Inv-20 clauses c/i/j/k/l — clauses a/b/e/g/h are golden-pinned-not-live, see the Inv-20 clause-level disclosure in the preamble); **Inv-19 and Inv-21 are the two
register-then-enforce carve-outs** — the enforcing surface is AS-BUILT + property-pinned but has zero production
callers at HEAD, with production-path wiring deferred (Inv-19 `derive_kv` keying-glue → Row D-64 engine
encrypt-to-recipient wiring; Inv-21 fork-tie-break comparator → Row D-52 Phase-4-Meta-Composing merge path),
matching the Inv-15 register-then-enforce precedent. Each of
Inv-16/17/18/22 (and Inv-20 clauses c/i/j/k/l) has (1) a typed error or typed-reject
path, (2) at least one landed regression test pinning the firing condition, and (3) no observable bypass path —
the table rows above carry the real construction-site + test cites. **Inv-19 honest carve-out:** the typed
`derive_kv(CidTarget) -> Result<_, KvError>` `K(V)` keying-glue boundary
(`crates/benten-membership-set/src/keying_kv.rs`) type-rejects a mutable Anchor CID at the API boundary and is
property-pinned by `F-INV19-1`, but it has **zero production callers at HEAD** (the test-driven `derive_kv` is
the PRODUCTION-stand-in); wiring it into the live encrypt-to-recipient / MembershipSet keying path is deferred
with the engine encrypt-to-recipient wiring (Row D-64). **Inv-21 honest carve-out:** the
fork-tie-break comparator (`fork_total_order_key` / `fork_a_wins`, `src/set.rs:~194-237`) is AS-BUILT and
property-pinned by the `F-INV21-*` proptest family (totality/antisymmetry/transitivity + smaller-`created_at_hlc`
asymmetry + archival-half), but it has **zero production callers at HEAD** — the live distributed merge is
benten-sync LWW (`crdt.rs:535`, LARGER-HLC-wins) and the test-local `resolve_fork`
(`f_inv21_fork_tie_break_totality_version_node_cid.rs`) is the PRODUCTION-stand-in. Wiring the comparator into the
live concurrent same-anchor set-creation fork merge path is deferred to Phase-4-Meta-Composing (Row D-52); the
comparator + proptest are frozen at Core. **Inv-22 honest carve-out (R17 F-11):** member-nature-is-derived is
ENFORCED at Core by the exhaustive-destructure struct-fence (`f_ms_3_members_table_fusion.rs` — a new nature field
breaks the pattern at compile time) plus the DID-method-parse derivation (`is_ai_operated` /
`derive_member_nature`), so there is structurally NO stored/wire nature slot. What the struct-fence does NOT
machine-check is *per-consumer derivation-consistency* — i.e. that every FUTURE nature-reading call site actually
recomputes from the DID/attribution rather than caching an authoritative flag. At HEAD this is a non-issue (there
IS no cached-nature surface to trust — the field does not exist), so the carve-out is a forward discipline the
struct-fence backstops structurally, NOT a live gap; the per-consumer-recompute audit rides with the
Composing-wave membership mutation/read wiring (Row D-52-adjacent), matching the Inv-21 register-then-enforce
precedent. **RBAC admin-op authorization-enforcement carve-out:** membership /
governance *admin-op authorization enforcement* (admit / kick / promote actually gated on a live role check) is
Phase-4-Meta-**Composing**-wired. At this freeze the crate DOES ship one live public membership-MUTATING method —
`member::MembersTable::admit(&mut self, did, entry)` (`crates/benten-membership-set/src/member.rs`), a
one-DID-one-record upsert gated only on the `is_authority ⟹ sig_pubkey` coupling rule (`entry.validate()`), NOT on
any role check — but it is a non-bypassable **data-half** primitive: ZERO production callers at HEAD, the real
membership gates are `K_Set` / `M_auth` (a non-member cannot produce a valid stanza regardless of the members
table), and the engine-layer UCAN / `CapabilityPolicy` admin-op authz wraps it in Composing. `set.rs` itself ships
only the LWW / fork comparators (no live mutation method that could skip a role check) — so there is no bypassable
admin-op-authorization surface to enforce at Core (the same register-then-enforce deferral pattern as Inv-19 /
Inv-21). The R2 test-landscape
(`.addl/phase-4-meta/f-full-r2-test-landscape.md` §2.1) mapped every one of Inv-16..22 to a covering red-phase
test family; the red-phase corpus landed in R3 and impl-to-green landed in R5. The one v1-beta floor honestly
disclosed: Inv-21's `F-INV21-3` kani convergence proof is `#[cfg(kani)]`-gated with a proptest surrogate
standing as the v1-beta floor (kani is a v1-GM strengthening, not a v1-beta dependency).

**Enforcement clustering (which crate owns each):**

- **Inv-16 / Inv-17 / Inv-18** (the encryption arc — "9-eyes" invariants) — enforced in
  **`benten-crypto-suite`**, the single `#5` crypto call site. The Wave-0 envelope canon (`F-W0-*`) mints the
  `EncryptedEnvelope` / `BindingContext` types + the §4.0 codepoint integer pins + the V2/BE migration + the
  real X-Wing combiner; no other wave authors envelope bytes until that red-phase corpus is on origin (M-20).

- **Inv-19 / Inv-20 / Inv-21 / Inv-22** (the MembershipSet arc) — enforced in the **15th crate
  `benten-membership-set`** (a thin Rust-engine-plugin keying-glue crate: EXACTLY-3 `MembershipSetKind` +
  `members_table` CBOR + per-Kind constructors + 0x6610 group-AAD assembly + the Inv-21 fork-tie-break rule +
  the clause-k recursion-bound). It delegates ALL crypto primitives to `benten-crypto-suite` (NEVER forks —
  `#5`) and depends UPSTREAM on **`benten-sync`** for the CRDT/HLC/MST/transport machinery the fork-tie-break
  (Inv-21) and convergence (Inv-20 clause-e/g) run on. The losing-fork archival (Inv-21) and the
  member-nature derivation (Inv-22) are graph-native (data-half) per the SPLIT model.

**Inv-21 directional note (load-bearing — M-7):** Inv-21's "smaller-`created_at_hlc`-wins" for **set-identity
forks** is DELIBERATELY OPPOSITE to the established in-tree LARGER-HLC-wins LWW rule for **member-property
merges** (`crates/benten-sync/src/crdt.rs:535`, keeps the entry where `cmp_lex == Greater`). Both rules
co-exist over two object classes in the same merge round (`F-CRDT-3` pins the co-existence). This is not a
contradiction of Inv-14-era CRDT semantics — it is a second, distinct CRDT rule scoped to set-identity forks,
where oldest-anchor-wins prevents an adversary from re-forking with a fabricated future HLC to steal set
identity. `created_at_hlc` (set-anchor creation, immutable, participates in the tie-break) is a distinct stamp
from `admitted_at_hlc` (per-member admission, LWW, does NOT participate) — `F-HLC-1` pins the distinction.

**Inv-21 tie-break antisymmetry ⇄ Compromise #6 (R15 F-24 cross-ref).** The comparator's *totality +
antisymmetry* — the property that any two distinct forks have a well-defined, agreed winner — rests on the CID
final tie-break byte being distinct between distinct fork events (distinct-fork-event ⇒ distinct-CID). That
distinctness is exactly the **BLAKE3 128-bit effective collision-resistance bound of Compromise #6** in
`docs/SECURITY-POSTURE.md`: a CID collision between two genuinely-distinct forks would defeat antisymmetry (two
forks tie-break-equal yet non-identical). So Inv-21's convergence guarantee inherits the same collision-resistance
assumption #6 already discloses — a distinct anchoring for the fork-tie-break axis, not a new assumption.

**Header-count + freeze-gating dependency:** the F-full doc-wave gate (`F-DISC-2`) asserts the end-state
"INVARIANT-COVERAGE.md REGISTERS Inv-16..22 AND Inv-15 NOT re-registered AND header count correct". The header
preamble reads "**23 invariants**" (bumped from "15 invariants" by the F-full cascade, then to 23 by the
GAP-KDB Shape-B Inv-23 mint), with the Inv-15
reconciliation note preserved. The `crates/...` enforcer paths + test-family destinations in the rows above are
now **verified construction sites at HEAD** — the crate `benten-membership-set` (the 15th crate)
exists and is green, as do `benten-crypto-suite`'s F-full envelope/swap-matrix surfaces and `benten-engine`'s
`layer_d` module. The `f_disc_2` registration catch-net
(`crates/benten-drop/tests/f_disc_2_invariant_and_doc_registration_catch_net.rs`) asserts the doc-wave
end-state AND, since the AS-BUILT+ENFORCED reclassification, now carries the enforced-state arms that drive the
real `benten-drop` production code: `f_disc_2_inv16_codepoint_dispatch_enforced_fail_closed`,
`f_disc_2_inv18_sealed_sender_default_metadata_disclosure_enforced`,
`f_disc_2_inv20_clause_c_group_aad_field_set_enforced_blinded`, and
`f_disc_2_inv19_inv20_truncation_defense_enforced_fail_closed` (all live `#[test]`, not ignored). The Inv-21 /
Inv-22 enforcement backing is owned by the `benten-membership-set` / `benten-sync` test targets per
`f_disc_2_inv21_inv22_enforcement_owned_elsewhere_flag`.

---

## What "active" means in this table

A row is **active** iff:

1. The invariant has a typed `RegistrationError` or `ErrorCode` it
   raises on violation.
2. The invariant has at least one regression test pinning the firing
   condition.
3. The crate that owns enforcement consumes the invariant on every
   relevant code path (i.e. there is no observable code path that
   bypasses it without explicit named-compromise documentation in
   `docs/SECURITY-POSTURE.md`).

All 14 Phase-4-Foundation invariants meet (1) (2) (3) at Phase 4-Foundation close.

**Inv-15 (Phase-4-Meta-Core mint)** is REGISTERED at Phase-4-Meta-Core kickoff. Status at HEAD: the **cross-surface read-audit is COMPLETE** (all eight surfaces ✓ VERIFIED at R6-reround `d4d9db5a`) and two of its findings were HARDENED IN CODE at Core — the F-03 UCAN payload-CID revocation re-key and the Ed25519 `verify_strict` on the classical half — with focused would-FAIL-on-revert pins landed at `crates/benten-engine/tests/inv15_sig_malleability_does_not_change_identifier.rs`. What remains for full (1) (2) (3) status is the FULL per-surface MallorySigner property-test cluster + the cite-drift-detector `LoadBearingSigBundleCidPattern` scanner extension + the pim-N codification, all on the G-CORE-PQ-WIRE-1 path. Until G-CORE-PQ-WIRE-1 closes that remaining automation, Inv-15 status is **"registered-with-active-existing-discipline-at-load-bearing-surfaces-pending-formal-cross-surface-enforcement"** — honest disclosure that the invariant is the right shape but the systematic-enforcement work is on the wave-1 critical path.

**Inv-16/17/18/22 — plus Inv-20 clauses c/i/j/k/l — (Phase-4-Meta-Core F-full design-mints)** are AS-BUILT + ENFORCED at HEAD. For those, all of (1) (2) (3) hold — the typed errors / typed-reject paths, the landed regression pins, and the no-bypass consumption all SHIPPED across the F-full wave sequence (R3 red-phase corpus → R5 impl-to-green → F-full close). **Two register-then-enforce carve-outs do NOT meet (3) at HEAD and are named separately: Inv-19 (see the "Inv-19 honest carve-out" in "Status semantics" above — `derive_kv` is AS-BUILT + property-pinned with zero production callers, wiring deferred to `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-64) and Inv-20 clauses a/b/e/g/h (golden-pinned-not-live — `keying::gossip_topic` has zero production callers and `governance::promote_tier` unconditionally returns `k_set_rotated: false`; production wiring rides the Composing wave per observation O-08).** See "Inv-16..22 Phase-4-Meta-Core F-full design-mints" section above + the table rows for the per-invariant construction sites + test cites. The `benten-membership-set` crate (the 15th crate, owner of Inv-19..22) exists and is green at HEAD; its enforcer-path + test-family cites are verified construction sites.

**Inv-21 (register-then-enforce honest carve-out)** is comparator-AS-BUILT + proptest-pinned but does NOT yet meet (3) at HEAD: the fork-tie-break comparator (`fork_total_order_key` / `fork_a_wins`) is built and property-pinned by the `F-INV21-*` family, but it has **zero production callers** — the live distributed merge path is benten-sync LWW (`crdt.rs:535`), so there IS an observable code path (the live merge) that does not consume the comparator. This is honestly disclosed (not a silent over-claim): the production-merge-path wiring — the concurrent same-anchor set-creation fork scenario exercised by the Composing distributed-sync path — is deferred to Phase-4-Meta-Composing per `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-52, matching the Inv-15 register-then-enforce precedent. The comparator + proptest are frozen at Core. Two honestly-disclosed Inv-21 floors travel together: (a) the production-merge wiring deferral above; (b) `F-INV21-3`'s kani convergence proof is `#[cfg(kani)]`-gated with the proptest surrogate as the v1-beta floor.
