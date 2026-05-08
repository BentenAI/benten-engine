# `@benten/engine` examples

Runnable examples for the three Phase-2b primitives + the Phase-3
Atrium DSL surface. Phase-2b examples pair an `*-handler.ts` (the
handler definition — pure DSL, no engine startup) with an
`*-example.ts` (the runner that wires the handler into an in-memory
`Engine` and exercises it end-to-end). Phase-3 Atrium examples are
single-file runners (the Atrium DSL composes via factory + handle
methods, not via subgraph definitions, so there's no separate handler
module).

Phase-2b primitive examples:

| Primitive | Handler | Runner |
|-----------|---------|--------|
| STREAM | [`stream-handler.ts`](./stream-handler.ts) | [`stream-example.ts`](./stream-example.ts) |
| SUBSCRIBE | [`subscribe-handler.ts`](./subscribe-handler.ts) | [`subscribe-example.ts`](./subscribe-example.ts) |
| SANDBOX | [`sandbox-handler.ts`](./sandbox-handler.ts) | [`sandbox-example.ts`](./sandbox-example.ts) |

Phase-3 Atrium examples (each exports a `run()` async function +
gates direct CLI invocation via `import.meta.url`):

| Surface | Runner |
|---------|--------|
| Peer management (D1 B-prime factory + `trustPeer` / `revokePeer` / `onPeerJoin` / `onPeerLeave`) | [`atrium-peer-mgmt.ts`](./atrium-peer-mgmt.ts) |
| Subscribe / sync trigger (per-handle `subscribe(path, cb)`) | [`atrium-sync-trigger.ts`](./atrium-sync-trigger.ts) |
| UCAN grant flow (`Engine.openWithPolicy(path, PolicyKind.Ucan)` + `grantCapability` / `revokeCapability` / `callAs`) | [`ucan-grant-flow.ts`](./ucan-grant-flow.ts) |
| DID resolution (`did:key` round-trip + `declareDeviceAttestation`) | [`did-resolution.ts`](./did-resolution.ts) |

## Type-check

```sh
cd packages/engine
npx tsc --noEmit -p examples/tsconfig.json
```

## Run

The `*-example.ts` files import from the `dist/` build of
`@benten/engine`; build the package once first:

```sh
cd packages/engine
npm run build
node --experimental-strip-types examples/stream-example.ts
node --experimental-strip-types examples/subscribe-example.ts
node --experimental-strip-types examples/sandbox-example.ts    # requires WASM module file

# Phase-3 Atrium examples (require an Atrium full peer to be online —
# the DSL surface compiles + types-checks on its own; running
# end-to-end requires UCANBackend + iroh + a configured Atrium DID).
node --experimental-strip-types examples/atrium-peer-mgmt.ts
node --experimental-strip-types examples/atrium-sync-trigger.ts
node --experimental-strip-types examples/ucan-grant-flow.ts
node --experimental-strip-types examples/did-resolution.ts
```

The SANDBOX example uses hardcoded placeholder CIDs for both the
module and manifest. Without registering real wasm bytes for the
placeholder module CID via `Engine.installModule(...)` over a
manifest authored against compiled module bytes, the
`engine.callAs(...)` dispatch surfaces a typed napi error at SANDBOX
entry (likely `E_SANDBOX_MANIFEST_UNKNOWN` or a manifest-resolution
error from the named-manifest registry). The example demonstrates
the call SHAPE, not an end-to-end run. See
`packages/engine/test/sandbox.test.ts` for end-to-end pins that ship
real wasm bytes via test fixtures, and
`docs/future/phase-3-backlog.md` §6.6 for the named-manifest
registration roadmap.

## Structure rationale

- **Handler files (`*-handler.ts`)** are pure DSL composition —
  exporting a `Subgraph` value via `subgraph(...).build()`. They have
  zero runtime dependencies beyond `@benten/engine`'s typed DSL
  surface. A scaffolder or test harness can import them without
  starting an engine.
- **Runner files (`*-example.ts`)** demonstrate the call shape — how
  to register the handler, drive the primitive (call / callStream /
  install module / etc.), and consume the response. They DO start an
  engine.

This split mirrors the production pattern: handlers live in your
codebase as data; engine startup happens once at process boot.
