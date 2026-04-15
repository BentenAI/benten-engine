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

## What works / what doesn't (honest status as of 2026-04-14)

- [ ] `npx create-benten-app` scaffolder -- Phase 1 deliverable, not yet built
- [ ] `@benten/engine` npm package -- Phase 1 deliverable, engine crates not yet built
- [ ] `crud('post')` zero-config path -- designed, not yet implemented
- [ ] `toMermaid()` visualization -- Phase 1 deliverable
- [ ] `engine.trace()` debugging -- Phase 1 deliverable
- [ ] Dev server with hot reload -- Phase 2 deliverable (needs registered-subgraph reload semantics)

If you're reading this pre-Phase-1, treat it as a specification of intent rather than a working tutorial.
