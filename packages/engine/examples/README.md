# `@benten/engine` examples

Runnable example handlers for the three Phase-2b primitives. Each
example pairs an `*-handler.ts` (the handler definition — pure DSL,
no engine startup) with an `*-example.ts` (the runner that wires the
handler into an in-memory `Engine` and exercises it end-to-end).

| Primitive | Handler | Runner |
|-----------|---------|--------|
| STREAM | [`stream-handler.ts`](./stream-handler.ts) | [`stream-example.ts`](./stream-example.ts) |
| SUBSCRIBE | [`subscribe-handler.ts`](./subscribe-handler.ts) | [`subscribe-example.ts`](./subscribe-example.ts) |
| SANDBOX | [`sandbox-handler.ts`](./sandbox-handler.ts) | [`sandbox-example.ts`](./sandbox-example.ts) |

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
