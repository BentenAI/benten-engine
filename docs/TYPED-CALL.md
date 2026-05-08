# Typed-CALL — engine-known dispatch surface

**Status:** Phase-3 G21-T1 (engine-side dispatch surface landed; PR #145, main `0d2d8e1`). T2 wires the napi/DSL surface; T3 lands handler examples + paper-prototype + the reserved-namespace registration reject.

This document is the canonical engineer-facing reference for **typed-CALL**, the engine's registry of named operations dispatched through the existing CALL primitive.

Typed-CALL is **NOT a new primitive.** The 12-primitive commitment (`CLAUDE.md` baked-in #1) holds. Typed-CALL is a *seam* on top of the CALL primitive: when a CALL operation node's `target` (handler id) starts with the reserved [`TYPED_CALL_PREFIX`](#reserved-namespace) of `engine:typed:`, the eval-side dispatch fork routes the call through the typed-CALL registry instead of the user-handler registry.

---

## Why typed-CALL exists

Phase 3 ships an Atrium / UCAN / DID / VC story; handlers need crypto operations (Ed25519 sign / verify, BLAKE3 hash, multibase, DID resolve, UCAN chain validation, VC verify). Per `CLAUDE.md` baked-in **#16** the SANDBOX host-fn surface stays minimum-viable (`time` / `log` / `kv:read` / `random` only); storage-mutating or domain-specific host-fns are explicitly NOT engine concerns. Crypto ops fit the CALL surface because they're input → typed result with no side effects on engine state. Adding them as typed-CALL ops gives handlers a low-friction call shape (an existing CALL Node) without widening the SANDBOX host surface or inventing a 13th primitive.

Decision tree for "where does this compute belong?":

| Compute shape | Belongs in |
|---|---|
| Composes from existing engine primitives (READ/WRITE/TRANSFORM/BRANCH/ITERATE/CALL/RESPOND/EMIT/WAIT/SUBSCRIBE/STREAM) | A user-handler subgraph; dispatch via `engine.call("<handler_id>", "<action>", input)`. |
| Engine-known op with a fixed input → typed-result shape, no side effects, fits in `benten-id` / `benten-core` / `benten-graph` already | **Typed-CALL.** Add a `TypedCallOp` variant + per-op `validate_input` + per-op `dispatch_typed_call` arm. Closed registry (no user-registered typed-CALL ops). |
| Heavy compute that wasm runtime is needed for (ML inference, custom transformers, image processing) | SANDBOX. Module manifest declares `time` / `log` / `kv:read` / `random` cap subset. |
| Storage mutation, capability gating, event emission | NOT host-fns. The other 11 primitives already cover these; SANDBOX modules return values that the engine's WRITE primitive persists, emit events that EMIT broadcasts, signal completion via RESPOND. |

---

## Reserved namespace

The handler-id prefix `engine:typed:` is **reserved** by the engine for typed-CALL dispatch. The eval-side dispatch fork at the CALL primitive recognises this prefix and routes through [`benten_eval::TypedCallOp::parse`](../crates/benten-eval/src/typed_call.rs). The trailing segment after the prefix is the op name; an unrecognised op name surfaces `E_TYPED_CALL_UNKNOWN_OP` (NOT `E_NOT_FOUND` — the latter is the user-handler-registry miss code).

User handler registration into the `engine:typed:` namespace is rejected at registration time (`E_RESERVED_HANDLER_NAMESPACE`, landing at G21-T3 per `docs/future/phase-3-backlog.md` §2.5(d)). At G21-T1 the registration-time reject is not yet in place; the eval-side dispatch fork pre-empts user-handler routing for the prefix, so a handler registered against `engine:typed:foo` is dead code (pinned by `crates/benten-engine/tests/typed_call_engine_dispatch.rs::typed_call_namespace_pre_empts_user_handler_registry_for_unknown_op`).

---

## The 10 ops (G21-T1)

The closed registry ships exactly 10 ops at Phase-3 G21-T1. Extension is a Rust-only engine concern (a new variant on the `#[non_exhaustive]` `TypedCallOp` enum + its per-op `validate_input` / `required_cap` / `is_deterministic` arms + a corresponding `dispatch_typed_call` arm in `benten-engine`). The registry is closed deliberately: typed-CALL is for engine-shipped ops, not user-extensible computation.

| Op name | Variant | Required cap | Deterministic | Input shape | Output shape |
|---|---|---|---|---|---|
| `ed25519_sign` | `Ed25519Sign` | `cap:typed:crypto-sign` | yes | `{ private_key: Bytes(32), message: Bytes }` | `{ signature: Bytes(64) }` |
| `ed25519_verify` | `Ed25519Verify` | `cap:typed:crypto-verify` | yes | `{ public_key: Bytes(32), message: Bytes, signature: Bytes(64) }` | `{ valid: Bool }` |
| `keypair_generate` | `KeypairGenerate` | `cap:typed:crypto-keygen` | **no** (OS CSPRNG) | `{}` (or `{ seed: Null }`) | `{ private_key: Bytes(32), public_key: Bytes(32) }` |
| `keypair_from_seed` | `KeypairFromSeed` | `cap:typed:crypto-keygen` | yes | `{ seed: Bytes(32) }` | `{ private_key: Bytes(32), public_key: Bytes(32) }` |
| `blake3_hash` | `Blake3Hash` | `cap:typed:hash` | yes | `{ data: Bytes }` | `{ hash: Bytes(32) }` |
| `multibase_encode` | `MultibaseEncode` | `cap:typed:codec` | yes | `{ data: Bytes, base: Text }` (`b` or `z`) | `{ encoded: Text }` |
| `multibase_decode` | `MultibaseDecode` | `cap:typed:codec` | yes | `{ encoded: Text }` | `{ data: Bytes, base: Text }` |
| `did_resolve` | `DidResolve` | `cap:typed:did-resolve` | **no** (conservative; see §[did_resolve method validation](#did_resolve-did-method-validation)) | `{ did: Text }` | `{ method: Text, public_key: Bytes(32) }` |
| `ucan_validate_chain` | `UcanValidateChain` | `cap:typed:ucan-validate` | yes | `{ tokens: List<Bytes>, audience: Text, capability: Text, now: Int }` | `{ valid: Bool, reason: Text }` |
| `vc_verify` | `VcVerify` | `cap:typed:vc-verify` | yes | `{ credential: Bytes, expected_issuer_did: Text, now: Int }` | `{ valid: Bool, issuer: Text, subject: Text }` |

Authoritative per-op rustdoc lives at [`benten_eval::TypedCallOp`](../crates/benten-eval/src/typed_call.rs); the dispatch implementations live at [`benten_engine::typed_call_dispatch::dispatch`](../crates/benten-engine/src/typed_call_dispatch.rs).

---

## Determinism (Inv-9)

Per Inv-9 (engine-known determinism declarations) the bare CALL primitive classifies `is_deterministic = true`. Typed-CALL ops have per-op classifications layered on top: when a CALL Node's `target` starts with `engine:typed:`, the Inv-9 registration-time walker consults [`TypedCallOp::is_deterministic`](../crates/benten-eval/src/typed_call.rs) on top of the primitive's own classification.

Two ops classify **non-deterministic**:

- `keypair_generate` — OS CSPRNG; the returned keypair varies per invocation.
- `did_resolve` — conservative classification. `did:key:` resolution is provably pure (the public key is in the DID body), but the op's input space includes future methods (`did:web:` / `did:plc:` / etc.) that hit the network; conservative answer keeps Inv-9 honest under method extension.

All other typed-CALL ops are pure functions of their inputs (no clock, no RNG, no network).

---

## Capability model

Each typed-CALL op declares a per-op required capability of shape `cap:typed:<group>`. The host's [`PrimitiveHost::check_capability`](../crates/benten-eval/src/host.rs) hook gates the op BEFORE the underlying `benten-id` / `benten-core` op is invoked — a denied call has zero observable side effect.

**Backends:**

- **`NoAuthBackend`** (default) — all `cap:typed:*` caps are permitted. Suitable for development + single-process deployments where capability checking is off.
- **`UCANBackend`** (Phase-3) — gates per UCAN-claim. Note: at G21-T1 the `UCANBackend` does not yet ship a policy-mapping table from UCAN claim strings to `cap:typed:*` caps (see `docs/future/phase-3-backlog.md` §2.5(c) — "cap:typed:* namespace consumer-side mapping, sec-minor-4"); under UCAN the cap-deny-by-default behavior surfaces because no UCAN claim grants a `cap:typed:*` capability yet. Closed at the G21-T2 napi-UCAN-wireup wave or a sibling cleanup wave.

A denied dispatch routes to `ON_DENIED` per the same precedent as `E_CAP_DENIED` / `E_SANDBOX_HOST_FN_DENIED`. The error code is `E_TYPED_CALL_CAP_DENIED` (catalog row in [`docs/ERROR-CATALOG.md`](ERROR-CATALOG.md)).

---

## Dispatch shapes

### Direct (engine-side, post-G21-T1)

```rust
use benten_engine::Engine;

// A CALL operation node whose `target` property is "engine:typed:blake3_hash"
// + an `input` map property with shape `{ data: Bytes }`.
let outcome = engine.call("engine:typed:blake3_hash", "default", input_value)?;
// outcome.value: { hash: Bytes(32) }
```

### TypeScript DSL (post-G21-T2)

The G21-T2 wiring exposes a typed surface on the napi engine:

```typescript
const { hash } = await engine.typedCall("blake3_hash", { data: bytes });
```

Until T2 lands, callers route through the bare `engine.call("engine:typed:<op>", "default", input)` API.

---

## did_resolve DID-method validation (§2.5(b))

The `did_resolve` op's current implementation hardcodes `method: "key"` in the output map. The input DID string is NOT parsed for its method prefix; a `did:web:example.com` input would surface a wrong `method: "key"` field rather than a typed error.

**Phase-3 target:** parse the method dynamically from the DID prefix (the segment between `did:` and the next `:`); if `did:key:`, route through the current `benten_id::did::Did::resolve` resolver; non-`did:key:` methods either route through future resolvers (`did:web:` / `did:plc:` etc. when added) OR reject with a typed `E_TYPED_CALL_DISPATCH_ERROR` carrying a `"unsupported DID method"` reason. Tracked at `docs/future/phase-3-backlog.md` §2.5(b) — "did_resolve op DID-method validation, sec-minor-3".

The conservative `is_deterministic = false` classification (see §[Determinism](#determinism-inv-9)) is set in anticipation of `did:web:` etc. landing later; current behavior with only `did:key:` would technically be pure, but the op signature must remain stable across method extension.

---

## Error codes

Four engine-layer error codes surface from typed-CALL dispatch. Authoritative entries live in [`docs/ERROR-CATALOG.md`](ERROR-CATALOG.md).

| Code | Routes via | Triggered by |
|---|---|---|
| `E_TYPED_CALL_UNKNOWN_OP` | `ON_ERROR` | `engine:typed:` prefix recognised but the trailing op name is not in the registry. |
| `E_TYPED_CALL_INVALID_INPUT` | `ON_ERROR` | Per-op input shape rejection (missing field, wrong CBOR type, fixed-width byte-length mismatch). |
| `E_TYPED_CALL_CAP_DENIED` | `ON_DENIED` | Per-op `cap:typed:*` cap not held by the dispatching grant. |
| `E_TYPED_CALL_DISPATCH_ERROR` | `ON_ERROR` | Underlying `benten-id` / `benten-core` op returned a typed error (malformed key, malformed UCAN/VC, unsupported DID method post-§2.5(b)). |

A clean negative (Ed25519 verify returns `false`, UCAN chain expired) is **NOT** an error — those return a structured `{ valid: false, ... }` Map with the op-internal failure reason. `E_TYPED_CALL_DISPATCH_ERROR` fires only when the underlying API call cannot produce a well-formed result.

---

## Cross-references

- [`crates/benten-eval/src/typed_call.rs`](../crates/benten-eval/src/typed_call.rs) — closed `TypedCallOp` enum + per-op input validation + capability declarations + determinism classification.
- [`crates/benten-engine/src/typed_call_dispatch.rs`](../crates/benten-engine/src/typed_call_dispatch.rs) — engine-side dispatch implementations against `benten-id` / `benten-core`.
- [`crates/benten-engine/src/primitive_host.rs`](../crates/benten-engine/src/primitive_host.rs) — `PrimitiveHost::dispatch_typed_call` impl + cap-check seam.
- [`docs/ERROR-CATALOG.md`](ERROR-CATALOG.md) — `E_TYPED_CALL_*` rows with `Thrown at` symbol cites.
- [`docs/ATTACK-SURFACE-MATRIX.md`](ATTACK-SURFACE-MATRIX.md) §2.8 — typed-CALL attack-surface enumeration.
- [`docs/future/phase-3-backlog.md`](future/phase-3-backlog.md) §2.5 — typed-CALL fp-mini-review residuals (sec-minor-2 zeroize, sec-minor-3 did_resolve method, sec-minor-4 UCAN cap mapping, corr-minor-3 reserved-namespace reject).

---

*Authored Phase-3 G21-T4 to consolidate the typed-CALL engineer-facing reference. Updates accompany each typed-CALL extension wave.*
