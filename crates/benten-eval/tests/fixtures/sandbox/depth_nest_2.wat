;; Phase-2b G7-B / Inv-4 — depth-2 SANDBOX positive fixture.
;;
;; A wasm module that imports a `sandbox_invoke` host-fn (G7-A surface).
;; When invoked from inside a depth-1 SANDBOX, calling `sandbox_invoke`
;; pushes the AttributionFrame to `sandbox_depth = 2`, still under the
;; default ceiling of 4.
;;
;; The G7-A `sandbox_invoke` host-fn signature (per plan §3 G7-A) takes a
;; module identifier (i32 handle into the host-side module table) and
;; returns the invocation status code.
;;
;; D26-RESOLVED dev-only build:
;;     wat2wasm depth_nest_2.wat -o depth_nest_2.wasm
;; The pre-built `.wasm` is committed alongside.

(module
  (import "benten" "sandbox_invoke" (func $invoke (param i32) (result i32)))
  (func (export "run") (result i32)
    ;; Invoke nested SANDBOX with module handle 0 (placeholder; real
    ;; handle is supplied by the G7-A host-side dispatch table at the
    ;; runtime test site).
    i32.const 0
    call $invoke
  )
)
