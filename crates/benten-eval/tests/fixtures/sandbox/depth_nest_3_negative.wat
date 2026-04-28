;; Phase-2b G7-B / Inv-4 — depth-3 nested SANDBOX negative fixture.
;;
;; A wasm module that recursively invokes `sandbox_invoke` to drive the
;; `AttributionFrame.sandbox_depth` counter past the runtime ceiling.
;; When loaded inside an outer chain that puts depth at 4, this fixture's
;; `sandbox_invoke` call pushes depth to 5 — one past the default
;; `max_sandbox_nest_depth = 4`. The runtime SANDBOX entry checker
;; (`invariants::sandbox_depth::check_runtime_entry`) refuses with
;; `ErrorCode::InvSandboxDepth` BEFORE wasmtime instantiates the depth-5
;; module (D22 cold-start discipline — no Module compile cost paid for a
;; rejected depth).
;;
;; "Negative" in the fixture name = the fixture is EXPECTED to fail at
;; runtime, used by the rejection-path test in `invariant_4_runtime.rs`.
;;
;; D26-RESOLVED dev-only build:
;;     wat2wasm depth_nest_3_negative.wat -o depth_nest_3_negative.wasm

(module
  (import "benten" "sandbox_invoke" (func $invoke (param i32) (result i32)))
  (func (export "run") (result i32)
    ;; Invoke nested SANDBOX. Whether the call succeeds or trips Inv-4 is
    ;; decided by the host-side current `sandbox_depth`; the test wraps
    ;; the call in a depth-4 chain to force the depth-5 trip.
    i32.const 0
    call $invoke
    ;; If we get here, depth was within ceiling — return 0 (success).
    drop
    i32.const 0
  )
)
