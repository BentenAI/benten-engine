;; ESC-5 — Recursion-depth overflow via deep WASM call stack.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-5: module recurses to a depth
;; that would exhaust the host thread's stack. Distinct from Inv-4
;; (E_INV_SANDBOX_DEPTH counts SANDBOX-primitive nest depth) — this is
;; intra-module call depth. wasmtime's max-stack-depth setting MUST trap
;; deterministically and surface as E_SANDBOX_MODULE_INVALID (or a reserved
;; E_SANDBOX_STACK_EXHAUSTED if R1 grants one — current 12-variant set folds
;; into MODULE_INVALID).
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm recursive_call_overflow.wat -o recursive_call_overflow.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  (func $recurse (export "run") (result i32)
    ;; Each call adds a frame; with no base case the stack overflows.
    call $recurse
  )
)
