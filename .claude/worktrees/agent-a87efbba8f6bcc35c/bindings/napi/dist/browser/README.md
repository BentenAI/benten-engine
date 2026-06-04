# browser bundle artifact directory

Phase-3 G20-A3 wave-8a — committed per
`docs/future/phase-3-backlog.md` §7.3.A.9 (sub-cluster 9b).

`benten_engine.wasm.gz` is a stable artifact path consumed by:

- `crates/benten-engine/tests/integration/browser_target_bundle_size.rs::wasm32_unknown_unknown_bundle_size_under_threshold`
  (asserts the gzipped bundle is ≤500KB per wasm-r1-7).
- `.github/workflows/wasm-browser.yml` (CI rebuilds + republishes the
  bundle on every push; the committed file is a SEED placeholder that
  the workflow overwrites in CI artifacts and that tests use as the
  on-disk pin point).

## Why a committed placeholder?

The bundle-size pin is structural: the Phase-3 commitment is that the
browser bundle path EXISTS, that the file shape is `.wasm.gz`, that
the size cap is enforceable from a stable filesystem location. The
seed placeholder file (a minimum-viable gzipped wasm module) lets the
Rust test driver read from a stable path without depending on a CI
artifact-download step in local development. CI replaces the seed
with the real build output for the actual size cap to be enforced
against the production bundle.

## What lives here

- `benten_engine.wasm.gz` — gzipped wasm32-unknown-unknown bundle.
  Placeholder seed at G20-A3; replaced by the wasm-browser.yml
  workflow with the real build output.

## What does NOT live here

- `*.node` files — the napi node-target binary is built separately
  (`bindings/napi/index.node` or platform-specific paths). Bundling
  `*.node` here would silently bloat the browser bundle (wasm-r1-6
  forbids; the test driver enforces).
