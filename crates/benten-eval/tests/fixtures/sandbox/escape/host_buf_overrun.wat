;; ESC-3 — Host-buffer overrun via host-fn output write.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-3: the module calls kv:read
;; with a (ptr, len) combination where len exceeds the buffer size at ptr.
;; The host-fn MUST validate length against the module's declared memory
;; layout and return E_SANDBOX_MODULE_INVALID rather than silently writing
;; past the declared buffer.
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm host_buf_overrun.wat -o host_buf_overrun.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  ;; Imported kv:read host-fn shape — placeholder signature pending G7-A
  ;; final wasmtime trampoline pin. Tracked under wsa-21 for canonical
  ;; signature decision.
  (import "host" "kv_read" (func $kv_read (param i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  ;; "run" passes (key_ptr, key_len, out_ptr=0, out_len=0xFFFFFFFF) to the
  ;; host-fn. out_len greatly exceeds the single-page memory; the host-fn
  ;; MUST refuse and return a typed-error code rather than write past the
  ;; declared buffer.
  (func (export "run") (result i32)
    i32.const 0          ;; key_ptr
    i32.const 4          ;; key_len
    i32.const 0          ;; out_ptr
    i32.const 0xFFFFFFFF ;; out_len — pathological, exceeds memory size
    call $kv_read
  )
  (data (i32.const 0) "\01\02\03\04")
)
