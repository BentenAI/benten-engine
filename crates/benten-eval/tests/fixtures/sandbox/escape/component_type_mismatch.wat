;; ESC-11 — Component-Model type mismatch with declared interface.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-11: module exports a
;; function with signature (i32) -> i64 but the host imports it as
;; (i64) -> i32. wasmtime's Component-Model type-checker MUST refuse the
;; link and surface E_SANDBOX_MODULE_INVALID.
;;
;; GATING (per R3-C brief + R2 §11.2 microgap 4): wsa-3 removed the
;; `component-model` feature from `wasmtime` Cargo deps. This fixture +
;; its driver are skip-gated on `cfg(feature = "component-model")` so the
;; corpus stays drift-stable without forcing the test to run when the
;; feature is absent (current 2b state).
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm component_type_mismatch.wat -o component_type_mismatch.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  ;; Single export with mismatched-vs-host-import signature.
  (func (export "compute") (param i32) (result i64)
    local.get 0
    i64.extend_i32_s
  )
)
