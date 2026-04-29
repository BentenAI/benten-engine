;; ESC-2 — Linear-memory grow beyond per-call cap.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-2: the module calls
;; memory.grow in a loop until exceeding the per-call memory limit
;; (default candidate: 64 MiB). E_SANDBOX_MEMORY_EXHAUSTED MUST fire
;; deterministically before host OOM via the wave-8b ResourceLimiter +
;; trap_to_typed mapping.
;;
;; Wave-8d-narrative re-author note: the original fixture used
;; `br_if 1` outside a containing block which wasmtime 43 rejects at
;; compile (the `br_if` carries a value; the surrounding loop has no
;; result type to receive it). The new shape uses an explicit
;; `(block $done (result i32) (loop $L ...))` so the `br $done` arm
;; carries the iteration count out of the block when memory.grow
;; returns -1 (limiter-refused), and the loop continues otherwise.
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm linmem_grow_to_limit.wat -o linmem_grow_to_limit.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  (memory (export "memory") 1)
  (func (export "run") (result i32)
    (local $i i32)
    (block $done (result i32)
      (loop $L
        ;; Try to grow memory by 1 page (64 KiB). If grow returns -1
        ;; (ResourceLimiter refused or wasm spec ceiling tripped), exit
        ;; the block carrying the iteration count.
        i32.const 1
        memory.grow
        i32.const -1
        i32.eq
        (if
          (then
            local.get $i
            br $done))
        ;; Increment counter; loop while under the high-water iteration
        ;; cap (defensive — the limiter trip should always fire first).
        local.get $i
        i32.const 1
        i32.add
        local.tee $i
        i32.const 100000
        i32.lt_s
        br_if $L
        ;; Natural loop termination — yield i to the surrounding block.
        local.get $i
        br $done
      )
      ;; Unreachable — both branches of the loop above route through
      ;; $done; this dummy exists to satisfy the block result-type.
      i32.const -1
    )
  )
)
