;; ESC-6 — Fuel-counter overflow regression.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-6: long-running computation
;; that, under a buggy fuel implementation, could overflow the u64 fuel
;; counter and silently restart. wasmtime guards this internally; the
;; regression test pins the guarantee against a wasmtime upgrade.
;; E_SANDBOX_FUEL_EXHAUSTED MUST fire at the configured budget regardless
;; of how long the computation has run.
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm fuel_overflow_regression.wat -o fuel_overflow_regression.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  ;; Tight integer-arith loop — a high op-density per fuel tick that, with
  ;; a defective interval, could push the fuel counter past u64::MAX.
  (func (export "run") (result i64)
    (local $i i64)
    (loop $L
      local.get $i
      i64.const 1
      i64.add
      local.tee $i
      i64.const 0
      i64.gt_s
      br_if $L
    )
    local.get $i
  )
)
