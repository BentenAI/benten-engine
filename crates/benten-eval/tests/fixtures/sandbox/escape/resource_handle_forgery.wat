;; ESC-12 — Resource handle forgery.
;;
;; Per pre-r1-security-deliverables.md §1, ESC-12: module fabricates an
;; i32 handle and passes it to a host-fn that expects a Component-Model
;; `resource` handle. The Component-Model resource-handle table validates
;; the handle; mismatch MUST fire E_SANDBOX_MODULE_INVALID (or
;; E_SANDBOX_HOST_FN_DENIED if the host-fn validates ownership).
;;
;; GATING (per R3-C brief + R2 §11.2 microgap 4): same as ESC-11 — wsa-3
;; removed `component-model`; fixture + driver are skip-gated on
;; `cfg(feature = "component-model")`.
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm resource_handle_forgery.wat -o resource_handle_forgery.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
  ;; Imported host-fn expecting a resource handle (modeled here as i32
  ;; for the core-wasm fixture-shape; the Component-Model wrapper
  ;; converts to the resource representation host-side).
  (import "host" "use_handle" (func $use_handle (param i32) (result i32)))
  (func (export "run") (result i32)
    ;; Forged handle value — never issued by the host's resource table.
    i32.const 0xCAFEBABE
    call $use_handle
  )
)
