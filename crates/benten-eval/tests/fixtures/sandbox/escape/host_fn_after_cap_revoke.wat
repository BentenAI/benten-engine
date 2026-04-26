;; ESC-9 — Call host-fn after cap revoked mid-primitive.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-9: caller grants module
;; host:compute:kv:read at primitive entry; mid-execution the orchestrator
;; revokes the cap; module makes a subsequent kv:read call. Per D7 hybrid
;; (sec-pre-r1-02) the live cap-string check fires per host-fn call; the
;; second call MUST observe the revoked cap and fire E_SANDBOX_HOST_FN_DENIED.
;;
;; Pairs with a Rust-side driver that uses
;; `testing_revoke_cap_mid_call(engine, &CapScope::host_compute_kv_read())`
;; between calls; D18-RESOLVED defaults `kv:read` to `cap_recheck = "per_call"`.
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm host_fn_after_cap_revoke.wat -o host_fn_after_cap_revoke.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  (import "host" "kv_read" (func $kv_read (param i32 i32 i32 i32) (result i32)))
  ;; Driver-supplied: invoked between the two kv:read calls. Implementation
  ;; calls the driver's hook to revoke the cap on the engine side.
  (import "host" "testing_yield_for_revoke"
    (func $yield (param i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "key1")
  (func (export "run") (result i32)
    (local $first i32)
    (local $second i32)
    ;; Call 1 — should succeed (cap still granted).
    i32.const 0
    i32.const 4
    i32.const 16
    i32.const 256
    call $kv_read
    local.set $first
    ;; Yield to driver — driver revokes the cap.
    i32.const 0
    call $yield
    drop
    ;; Call 2 — D18 per_call recheck observes the revoked cap and denies.
    i32.const 0
    i32.const 4
    i32.const 16
    i32.const 256
    call $kv_read
    local.set $second
    local.get $second
  )
)
