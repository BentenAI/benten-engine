;; ESC-4 — Infinite loop without fuel.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-4: tight `loop ... br 0 ... end`
;; never breaks. wasmtime's fuel meter MUST fire E_SANDBOX_FUEL_EXHAUSTED
;; within the per-call fuel budget.
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm infinite_loop.wat -o infinite_loop.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  (func (export "run") (result i32)
    (loop $L
      br $L
    )
    i32.const 0
  )
)
