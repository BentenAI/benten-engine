;; ESC-2 — Linear-memory grow beyond per-call cap.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-2: the module calls memory.grow
;; in a loop until exceeding the per-call memory limit (default candidate:
;; 64 MiB). E_SANDBOX_MEMORY_EXHAUSTED MUST fire deterministically before host
;; OOM.
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm linmem_grow_to_limit.wat -o linmem_grow_to_limit.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  (memory (export "memory") 1)
  ;; Each memory.grow request adds 64KiB (1 page). Looping until grow fails
  ;; (-1) or until the host-side budget trap fires.
  (func (export "run") (result i32)
    (local $i i32)
    (loop $L
      i32.const 1
      memory.grow
      i32.const -1
      i32.eq
      br_if 1
      local.get $i
      i32.const 1
      i32.add
      local.tee $i
      i32.const 100000
      i32.lt_s
      br_if $L
    )
    local.get $i
  )
)
