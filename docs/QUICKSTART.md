# Quickstart

**Status:** Planning (engine code lands in Phase 1). This document specifies what the 10-minute developer experience will look like.

The goal is that a developer lands on the README, follows three commands, and has a working Benten instance creating and querying Nodes within 10 minutes. No Rust knowledge required. No community/governance/marketplace concepts required.

## The 10-minute path (target)

### 1. Install

```sh
npx create-benten-app my-app
cd my-app
npm install
npm run dev
```

`create-benten-app` will (when it ships) scaffold a TypeScript project with:
- `@benten/engine` (the napi-rs wrapped engine)
- A minimal handler registered via `crud()`
- A dev server with hot reload (`bentend dev` equivalent)
- A sample handler file to edit

### 2. Write your first handler

The zero-config path:

```typescript
import { crud } from '@benten/engine/operations';

// Register a content type with defaults:
// - schema inferred from first write
// - `public` capability (no auth required)
// - local storage only
export const postHandlers = crud('post');
```

That's the simplest useful Benten code. `crud('post')` registers `create`, `read`, `update`, `delete`, and `list` handlers with sensible defaults.

### 3. Use it

```typescript
// Create
await engine.call('post:create', { title: 'Hello Benten', body: 'First post.' });

// Read
const posts = await engine.call('post:list', { limit: 10 });
console.log(posts);
```

## Adding capabilities (when you're ready)

When you need authentication, add a capability:

```typescript
export const postHandlers = crud('post', {
  capability: 'store:post:*',
});
```

Now every mutation requires a grant to that capability. The engine ships with a `NoAuthBackend` by default; swap in UCAN or custom policy when you need it.

## Adding schema validation

```typescript
export const postHandlers = crud('post', {
  schema: {
    title: { type: 'string', required: true, maxLength: 200 },
    body: { type: 'string', required: true },
    publishedAt: { type: 'date', optional: true },
  },
});
```

## Viewing your operation subgraph

Benten's handlers are subgraphs you can inspect:

```typescript
console.log(postHandlers.create.toMermaid());
// Prints a Mermaid diagram you can paste into any Markdown viewer
```

```typescript
const trace = await engine.trace('post:create', { title: 'Test', body: 'Trace me' });
console.log(trace.steps);
// Array of { node, inputs, outputs, durationUs }
```

## Next steps

- [`DSL-SPECIFICATION.md`](DSL-SPECIFICATION.md) -- the full TypeScript API
- [`validation/paper-prototype-handlers.md`](validation/paper-prototype-handlers.md) -- 5 realistic handler examples
- [`GLOSSARY.md`](GLOSSARY.md) -- Benten-specific terms
- [`VISION.md`](VISION.md) -- what Benten is and isn't

## What works / what doesn't (honest status as of 2026-04-17)

Phase 1 delivered:

- [x] `npx create-benten-app` scaffolder -- produces a project that runs `npm install && npm test && npm run build && npm run dev` green on a clean machine (`tools/create-benten-app/test/scaffolder.test.ts`).
- [x] `@benten/engine` npm package -- the TypeScript DSL wrapper over `@benten/engine-native`. Exposes `Engine`, `crud()`, `PolicyKind`, the 12 primitive helpers, and the typed error catalog.
- [x] `crud('post')` zero-config path -- one-liner that produces a five-action handler (`create`, `get`, `list`, `update`, `delete`). Optional `capability` / `hlc` / `label` overrides.
- [x] `handler.toMermaid()` visualization -- renders the registered subgraph as a Mermaid flowchart, sourced authoritatively from the Rust engine.
- [x] `engine.trace()` debugging -- per-step records carrying `nodeCid`, `primitive`, `durationUs`, plus `inputs` / `outputs` when the native tracer surfaces them.
- [x] Capability enforcement via `Engine.openWithPolicy(path, PolicyKind.GrantBacked)` + `engine.grantCapability({ actor, scope })` + `engine.revokeCapability(grantCid, actor)` -- the Phase-1 revocation-aware grant policy.

Deferred:

- [ ] Dev server with hot reload -- Phase 2 (needs registered-subgraph reload semantics).
- [ ] Version chains (`anchor.createVersion(...)`) as a first-class DSL shape -- Phase 2.
- [ ] P2P sync across Atriums -- Phase 3 (`benten-sync` crate, iroh transport).
- [ ] UCAN capabilities -- Phase 3 (`PolicyKind.Ucan` opens but its check path surfaces `E_CAP_NOT_IMPLEMENTED` today).
- [ ] WASM `SANDBOX` primitive -- Phase 2 (the primitive is type-defined; executor throws `E_PRIMITIVE_NOT_IMPLEMENTED`).

If you hit anything in the "delivered" list that doesn't behave as documented, please file an issue against `benten-engine` -- the scaffolder's smoke gate is the stability contract.
