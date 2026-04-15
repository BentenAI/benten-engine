# bindings/

Language bindings for the Benten Engine.

- **`napi/`** — Node.js / TypeScript bindings via napi-rs v3. The same codebase also compiles to a WASM target since napi-rs v3 provides first-class WASM support. (Exact target triple — e.g., `wasm32-wasip1-threads` or `wasm32-unknown-unknown` — to be confirmed during the Phase 1 spike based on what napi-rs v3 actually produces for our dependencies.)
- **`wasm/`** — Reserved for standalone WASM builds if needed beyond the napi-rs target. May not be necessary given napi-rs v3's WASM capability.
- **`python/`** — Phase 2+: PyO3 bindings for Python users.

These directories do not yet exist — they will be created when the first bindings come online (napi during Phase 1 spike, others as needed).

See [DSL-SPECIFICATION.md](../docs/DSL-SPECIFICATION.md) for the developer-facing API.
