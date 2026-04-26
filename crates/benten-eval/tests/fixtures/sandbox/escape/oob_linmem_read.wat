;; ESC-1 — Out-of-bounds linear-memory read.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-1: a WASM module reads beyond
;; its declared linear-memory bounds. wasmtime's bounds-check fires a trap;
;; the executor maps the trap to E_SANDBOX_MODULE_INVALID.
;;
;; Build (D26 dev-only regenerator — NOT a CI dependency):
;;     wat2wasm oob_linmem_read.wat -o oob_linmem_read.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  (memory (export "memory") 1)
  ;; Exported "run" attempts to load an i32 from offset 0xFFFFFFF0 which is
  ;; well past the single 64KiB page the module declared. wasmtime traps
  ;; deterministically on the OOB load.
  (func (export "run") (result i32)
    i32.const 0xFFFFFFF0
    i32.load
  )
)
