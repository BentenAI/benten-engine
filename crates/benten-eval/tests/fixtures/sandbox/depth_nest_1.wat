;; Phase-2b G7-B / Inv-4 — depth-1 SANDBOX positive fixture.
;;
;; A minimal wasm module that exports a `run` function. When loaded inside
;; a depth-1 SANDBOX context (no enclosing SANDBOX), this module evaluates
;; cleanly: the enclosing AttributionFrame has `sandbox_depth = 1` (one
;; SANDBOX entry from depth 0), well under the default ceiling of 4.
;;
;; D26-RESOLVED dev-only build:
;;     wat2wasm depth_nest_1.wat -o depth_nest_1.wasm
;; The pre-built `.wasm` is committed alongside per D26 (avoiding CI
;; shell-portability issues per wsa-12); `build_wasm.sh` is the dev-only
;; regenerator. `tests/fixture_wasm_hashes_stable` guards drift between
;; the `.wat` source and the committed bytes.

(module
  (func (export "run") (result i32)
    i32.const 0
  )
)
