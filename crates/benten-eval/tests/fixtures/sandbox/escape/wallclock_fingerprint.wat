;; ESC-16 — Wall-clock leak via `time` host-fn fingerprinting.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-16 + §2.1: module calls
;; `time` 10000 times in a tight loop; under D1 monotonic-coarsened-100ms
;; defaults, the deltas across a 50ms window MUST collapse to ≤1 distinct
;; value at 100ms granularity (no fingerprinting surface).
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm wallclock_fingerprint.wat -o wallclock_fingerprint.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  (import "host" "time" (func $time (result i64)))
  (memory (export "memory") 1)
  ;; Run the loop and write each distinct sample into linear memory at
  ;; (offset = i * 8). Driver reads the memory back, deduplicates, and
  ;; asserts the distinct-count is ≤1 across a 50ms window.
  (func (export "run") (result i32)
    (local $i i32)
    (local $t i64)
    (loop $L
      call $time
      local.set $t
      local.get $i
      i32.const 8
      i32.mul
      local.get $t
      i64.store
      local.get $i
      i32.const 1
      i32.add
      local.tee $i
      i32.const 10000
      i32.lt_s
      br_if $L
    )
    local.get $i
  )
)
