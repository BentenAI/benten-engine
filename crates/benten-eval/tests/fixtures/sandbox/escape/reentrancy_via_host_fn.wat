;; ESC-10 — Host-fn re-entrancy denial.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-10: host-fn callback (e.g.,
;; kv:read) attempts to dispatch back into a SANDBOX primitive on the same
;; Store. Cap-context confusion via SANDBOX → CALL → SANDBOX chain
;; (sec-pre-r1-08); also defense-in-depth against historical wasmtime
;; reentrancy bugs. E_SANDBOX_NESTED_DISPATCH_DENIED MUST fire at the inner
;; SANDBOX dispatch attempt (D19 RESOLVED rename from
;; E_SANDBOX_REENTRANCY_DENIED).
;;
;; Pairs with a Rust-side driver that supplies a host-fn body which calls
;; engine.call() back through the dispatcher.
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm reentrancy_via_host_fn.wat -o reentrancy_via_host_fn.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  ;; Driver-supplied host-fn that internally invokes engine.call().
  (import "host" "testing_call_engine_dispatch"
    (func $reenter (param i32) (result i32)))
  (memory (export "memory") 1)
  (func (export "run") (result i32)
    i32.const 0
    call $reenter
  )
)
