# Phase 4-Foundation backlog

**Status:** scaffolded 2026-05-11 as Phase 4-Foundation R1 pre-dispatch artifact (per meth-r1r1-1 closure of phantom-destination concern). Mirrors `phase-3-backlog.md` shape.

**Purpose:** named destination for Phase 4-Foundation R6 phase-close convergence carries + dogfood-validation findings + cross-phase carries that surface during Phase 4-Foundation implementation.

Couples to:
- `docs/future/phase-3-backlog.md` — Phase 4-bound carries surfaced before Phase 4-Foundation opened
- `docs/future/phase-3-backlog.md §14` — Phase 4-Foundation carries that fall to v1-assessment-window after Phase 4-Meta close
- `docs/future/phase-3-backlog.md §15.2` — handler-call-graph cycle detection (Phase 4-Meta-bound; couples to plugin install/registration time)
- `docs/future/kith-decentralized-identity.md` — exploratory decentralized-identity-and-attestation system (Phase 5+ candidate)

---

## §1. R6 phase-close convergence carries

Phase 4-Foundation R6 phase-close council will produce findings that don't gate the `phase-4-foundation-close` tag. Those land here as carries to Phase 4-Meta OR v1-assessment-window.

(Entries land during Phase 4-Foundation R6 — none at this writing.)

---

## §2. Dogfood validation gate carries

Ben's dogfood validation (wave-7 in plan §2 sequencing) will produce UX + interaction findings beyond the FIX-NOW-INLINE scope. Those land here per HARD RULE 12 clause-(b).

(Entries land during Phase 4-Foundation R5+ — none at this writing.)

---

## §3. Phase 4-Foundation → Phase 4-Meta carries

Architectural decisions made during Phase 4-Foundation that explicitly defer related work to Phase 4-Meta land here for tracking.

### §3.1 Decentralized self-discovered registry

**Origin:** Phase 4-Foundation R1 (2026-05-11). Plan §3 originally scoped decentralized self-discovered registry as part of D-4F-1 FULL plugin manifest scope; R1 lenses (plugin-architecture-reviewer + distributed-systems-reviewer + threat-model §8 Q1 cross-cite) surfaced internal contradiction with §3.X T10 deferring discover-flow defenses to Phase 4-Meta. Ben ratified 2026-05-11 evening: **move decentralized self-discovered registry to Phase 4-Meta.** Phase 4-Foundation admin UI v0 installs plugins via direct content-addressed-share over Atriums (peer-to-peer; user pulls from peer they trust).

**Phase 4-Meta scope:** decentralized registry surface (Atrium-substrate publish/subscribe; signed + content-addressed manifest discovery; trust-graph extension); admin UI discovery affordance (search/browse plugins from peers in your network).

**Couples to:** §3.2 Kith (richer identity-and-attestation substrate that the registry's trust-graph would build on).

### §3.2 Kith — decentralized identity & attestation system (EXPLORATORY)

**Origin:** Ben's framing 2026-05-11 evening conversation, during Q6 (peer-DID rotation propagation) discussion. The base Phase-3 peer-DID + RotationLog primitive is insufficient for handling key-rotation in a hostile-old-key scenario. Ben proposed a richer decentralized-identity substrate: "X has designated Y as Z" relational-attestation graph + per-relationship privacy controls + organizational attestations (Gardens/Groves, schools, certifying bodies) + UCAN-mediated contextual sharing.

**Full scope:** see `docs/future/kith-decentralized-identity.md` (exploratory scope-stub).

**Phase target:** **Phase 5+ or its own dedicated design-spike phase**, NOT Phase 4-Foundation (too large; Phase 4-Foundation uses a simpler "old-key revocation attestation + out-of-band new-key trust" MVP rotation mechanism per Q6 ratification).

**Phase 4-Foundation MVP rotation mechanism:**
- Old-key signs a `SelfRevocation` attestation marking itself as revoked (timestamped). Propagates via Atrium sync. Peers reject content signed by the old key after the revocation timestamp.
- New-key trust is NOT transferable from old key. Each peer re-establishes trust via out-of-band side-channel (same channel used for initial bootstrap).
- Grace window during rotation.

This MVP doesn't defeat the purpose of rotation (it doesn't ask receivers to trust the old key for new-key establishment) — it just propagates revocation cleanly.

### §3.3 Self-composing admin UI (meta-circular full scope)

**Origin:** carried from original Phase 4 scope; Phase 4-Foundation ships admin UI v0 that lets users edit workflows + composed views THROUGH it, but does NOT make the admin UI's own subgraph user-editable through itself. That meta-circular self-composing capability is Phase 4-Meta-bound.

### §3.4 Phase 4-Meta inherited carries from Phase 3

- wasmtime Component-Model re-evaluation (Phase-3 D-PHASE-3-6 + D-PHASE-3-16 + r1-wsa-12)
- Engine impl-block generic-cascade lift (Phase-3 §1.2-followup)
- Light-client mode-(b) range-query proof (ds-r4r2-3)
- Light-client mode-(c) signed checkpoint (ds-r4r2-3)
- Handler-call-graph cycle detection at handler-registration time (`phase-3-backlog §15.2`)

---

## §4. Phase 4-Foundation Track B (Class-of-bug audits + cleanups)

Plan G27 wave covers these; entries here for cross-reference.

### §4.1 UCAN class-of-bug audit across napi cap-* entry points

Per D-4F-5 ratification (Phase 4-Foundation Track B). Lateral sweep across napi cap-management entry points for scope-vs-CID-passed-as-string class of mistakes (same root cause as §13.11 fix at PR #199). Plan G27-A wave.

### §4.2 `benten-caps::GrantBackedPolicy::derive_write_scope` lift

Currently hard-codes `store:<label>:write` derivation; thread scope through `WriteContext::scope` (already exists for `UcanGroundedPolicy::check_write`; not yet for `GrantBackedPolicy::check_write`). Plan G27-B wave.

### §4.3 `GrantReader::has_unrevoked_grant_for_grant_cid(&Cid)` CID-keyed companion

Per §13.11 structural lesson — the scope-keyed `has_unrevoked_grant_for_scope(scope: &str)` lacks CID-keyed counterpart at the trait surface, which is what enabled the original `revokeCapability(grantCid, actor)` silent fail-OPEN. Add CID-keyed companion at the trait surface so CID is the canonical typed handle even at the reader API. Plan G27-C wave.

### §4.4 Manifest scope grammar at G27-D

Define mapping from manifest `requires` / `shares` to scope strings; story for `private:<plugin_did>:*` interaction with `wildcard_variants`; install-time-vs-check-time decision. Per cap-r1-3 closure.

### §4.5 `bindings/napi/tests/cap_delegate_napi_resolved_scope_regression_guard.rs` substantive arm at G24-D

R5 G27-A landed the napi class-of-bug audit (PR #224 via R5 wave-g27-a; merged 2026-05-13). The audit confirmed 4 cap-* entry points are the complete enumeration of scope-vs-CID class-of-bug risk surfaces. However, `delegateCapability` is **NOT YET SHIPPED** at the napi layer — the delegate surface lands at G24-D (FULL plugin manifest, via `crates/benten-caps/src/plugin_delegation.rs` runtime UCAN delegation with `audience=plugin-DID` within manifest envelope).

To preserve the un-ignore directive (per the G27-A R5 brief: "4/4 un-ignored") without violating HARD RULE rule-12 (no defer-without-destination), the G27-A implementer un-ignored the test + reshaped its body to assert the audit finding at HEAD (no shipped delegate surface). When G24-D lands the napi delegate surface, **that wave's implementer MUST rewrite `cap_delegate_napi_resolved_scope_regression_guard.rs` body to the substantive 4-step arm**: (1) `delegateCapability(grantCid, plugin_did, attenuated_caps)` over napi; (2) verify the delegation Node is minted with the resolved scope (not the grantCid as a string); (3) attempt a write under the delegated cap; (4) assert the per-row cap-recheck at delivery resolves the scope correctly.

Per HARD RULE rule-12 BELONGS-NAMED-NOW: this entry IS the named destination + the work obligation lands NOW (not "I'll add it later"). G24-D R5 implementer reads this section as part of their dispatch.

Closes G27-A R5 mini-review MINOR finding `g27a-mr-1`.

### §4.6 G23-A strict 4-of-4 input-dialect validation + arbitrary-schema proptest

R5 G23-A landed the `schema_compiler` canary (branch `r5/wave-g23-a`). The canary enforces the 4-mandatory rule (name / required / default / scope) over **emitted** primitive property bags but NOT over the **input** dialect — the JSON input schema fixtures currently omit `default`, so `default` is silently defaulted to JSON-null at parse time (`crates/benten-platform-foundation/src/schema_compiler/parse.rs:234-244` + parse.rs:391-400). The cap-scope deriver correctly schema-derives `<action>:<SchemaName>.<field_path>` from emit, so the input `scope` field is currently informational; cap-scope discipline is preserved end-to-end.

**Carry-criterion (lands when the canonical-fixture generator is auto-derived from the typed IR, OR earlier if a future wave needs strict input-dialect validation):** the `ParsedSchema` field-parser MUST reject schemas missing `default` with `E_SCHEMA_VOCAB_REQUIRED_PROPERTY_MISSING`, mirroring the existing emit-side enforcement. Today the fixture generator hand-writes JSON schemas which makes strict input validation a fixture-rewrite burden; once fixtures derive from the typed IR (proposed Phase 4-Foundation or Phase 4-Meta), strict 4-of-4 validation at the input dialect boundary lands without fixture churn. Same destination owns the explicit-edge dialect (user-declared edge labels beyond the field-tree-implied edges currently used at canary).

**Companion deferral — arbitrary-schema proptest:** `crates/benten-platform-foundation/tests/prop_schema_compile_is_idempotent_arbitrary_schemas.rs` remains `#[ignore]` at G23-A. The arbitrary-schema generator (`arbitrary_valid_schema_bytes(seed: u64) -> Vec<u8>` in `tests/common/schema_fixtures.rs`) needs the strict input-dialect grammar finalized before it can generate property-test inputs that exercise the dialect boundary, not just emit-side idempotency. The canary already covers fixed-fixture round-trip idempotency via `schema_compiler_round_trip_canonical_bytes_stable.rs` (un-ignored, PASS at G23-A); the proptest arm un-ignores when the strict 4-of-4 input-dialect lands per the carry-criterion above.

**Tentative phase target:** Phase 4-Foundation wave-N (TBD) OR Phase 4-Meta. NOT a v1-blocker — fixed-fixture idempotency at G23-A canary + emit-side 4-of-4 enforcement together suffice for the schema-driven-rendering substantive arm.

Per HARD RULE rule-12 BELONGS-NAMED-NOW: this entry IS the named destination + the work obligation lands NOW. G23-A wave's `parse.rs` source comments + the proptest's `#[ignore]` message cite `docs/future/phase-4-backlog.md §4.6` instead of phantom destinations like "wave-4b".

Closes G23-A R5 mini-review BLOCKER finding `g23a-mr-1` + MAJOR finding `g23a-mr-2`.

---

## §5. Phase 4-Foundation Track A (implementation work surfaced post-R1)

R1-FP work items that emerged from R1 critic round (production-vs-plan gaps). These are Phase 4-Foundation implementation, not deferred carries — listed here for traceability.

### §5.1 UCAN audience binding at `UcanGroundedPolicy::permits_typed_proof_for`

`crates/benten-caps/src/ucan_grounded.rs:191-216` currently calls `validate_chain_at` without audience binding. Add audience-binding wiring per cap-r1-1. ~100-200 LOC + tests. Closes load-bearing BLOCKER for the four-identity-concepts model.

### §5.2 `actor_cid` consulted on reads at `GrantBackedPolicy::check_read`

`crates/benten-caps/src/grant_backed.rs:296-327` currently wildcard-enumerates against scope-only. Add `ctx.actor_cid` consultation per cap-r1-2. ~50-100 LOC. Closes materializer dual-gate substance gap.

### §5.3 SUBSCRIBE-delivery cap-recheck closure

`crates/benten-engine/src/engine_subscribe.rs::Engine::on_change_as_with_cursor` (lines 290-327) is scaffold-only — calls `is_actor_active` not per-event `CapabilityPolicy::check_read`. Closure per sec-4f-r1-1; ~100-200 LOC. Closes admin UI dogfood path (d) revoke-cap-mid-session.

### §5.4 `plugin_lifecycle.rs` uninstall-cascade seam

Per plugin-arch-r1-2; ~150-300 LOC. Prevents orphan delegated-cap accumulation at uninstall time.

### §5.5 `manifest_envelope_chain_validation.rs` seam

Per plugin-arch-r1-3; ~200-300 LOC. Wires CLAUDE.md #18 Layer 3 runtime-delegation-within-manifest-envelope structurally.

---

## §6. Doc retense + ErrorCode catalog work

### §6.1 ERROR-CATALOG.md companion-with-canary routing

Per doc-r1-1 + doc-r1-2: 17+ new ErrorCodes for Phase 4-Foundation mint across waves (3 schema + 3 materializer + 9 plugin + new G27 surface). ERROR-CATALOG.md retense MUST land COMPANION-WITH-CANARY per wave, not bundled at G26-A. CATALOG_VARIANT_COUNT expected bump 118 → ~135.

### §6.2 INTERNALS.md retense for new surfaces

Per cross-lens doc-engineer findings: `benten-platform-foundation/INTERNALS.md` (NEW; 12th workspace crate), `benten-renderer-tauri/INTERNALS.md` (NEW; 12th-or-13th crate), updates to `benten-ivm/INTERNALS.md` (post IVM-subgraph generalization), `benten-engine/INTERNALS.md` (post audience-binding + actor_cid wiring + SUBSCRIBE-cap-recheck closure), `benten-caps/INTERNALS.md` (post Q5 plugin-DID-keyed signing-key infrastructure).

---

(Section structure additive; entries land as Phase 4-Foundation work surfaces them.)
