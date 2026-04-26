;; ESC-8 — Call host-fn not in manifest.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-8: module declares manifest
;; "compute-basic" (covers `time` + `log` only) but attempts to call
;; `kv:read`. Privilege-escalation attempt by a module that knows the
;; host-fn name. The link MUST fail with E_SANDBOX_HOST_FN_NOT_FOUND
;; (preferred) or call-time E_SANDBOX_HOST_FN_DENIED (fallback).
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm host_fn_not_on_manifest.wat -o host_fn_not_on_manifest.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  ;; kv:read import — the manifest-bound test invocation does NOT grant
  ;; host:compute:kv:read, so wasmtime linkage MUST refuse this import.
  (import "host" "kv_read" (func $kv_read (param i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "key1")
  (func (export "run") (result i32)
    i32.const 0
    i32.const 4
    i32.const 16
    i32.const 256
    call $kv_read
  )
)
