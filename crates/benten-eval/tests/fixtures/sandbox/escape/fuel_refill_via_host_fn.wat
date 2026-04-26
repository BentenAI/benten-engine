;; ESC-7 — Fuel-refill bypass via host-fn re-entry.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-7: module attempts to refill
;; fuel by triggering a host-fn callback that re-enters the dispatching
;; engine and (incorrectly) refreshes fuel. The host-fn dispatch path is
;; forbidden from touching the wasmtime Store's fuel counter; the re-entry
;; attempt MUST fire E_SANDBOX_NESTED_DISPATCH_DENIED (D19 RESOLVED rename
;; from E_SANDBOX_REENTRANCY_DENIED).
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm fuel_refill_via_host_fn.wat -o fuel_refill_via_host_fn.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  ;; Imported "log" host-fn — Rust-side test driver wires a host-fn body
  ;; that attempts engine.call() back into a SANDBOX dispatch. The driver
  ;; is the testing_invoke_engine_call_from_host_fn helper (G7-A scope).
  (import "host" "log" (func $log (param i32 i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "fuel-refill-attempt")
  (func (export "run") (result i32)
    ;; Burn fuel in a loop, calling log() periodically. The driver-supplied
    ;; log body attempts the nested dispatch on each call.
    (local $i i32)
    (loop $L
      i32.const 0
      i32.const 19
      call $log
      local.get $i
      i32.const 1
      i32.add
      local.tee $i
      i32.const 1000
      i32.lt_s
      br_if $L
    )
    local.get $i
  )
)
