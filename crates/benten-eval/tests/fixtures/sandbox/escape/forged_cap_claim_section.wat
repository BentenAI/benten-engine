;; ESC-14 — Cap-claim forge in module bytes (.wat source).
;;
;; Per pre-r1-security-deliverables.md §1, ESC-14: a module embeds a
;; custom WASM section claiming `requires: "host:*:*"` and (incorrectly)
;; the engine trusts the embedded claim instead of the manifest passed to
;; `Engine::sandbox_call`. The engine MUST silently ignore embedded
;; sections for cap purposes; cap derivation is exclusively from the
;; manifest passed at call time. A subsequent kv:read call from such a
;; module MUST still fire E_SANDBOX_HOST_FN_DENIED if the manifest didn't
;; include the cap.
;;
;; The forged custom section cannot be expressed in vanilla WAT (wat2wasm
;; doesn't emit arbitrary custom-section bytes). The fixture .wasm is
;; produced by `tests/security/forge_cap_claim_wasm_builder.rs` (G7-B
;; testing helper `testing_inject_forged_cap_claim_section(wasm_bytes)`).
;; This .wat documents the WASM body that gets the forged section
;; appended — the module itself is just a kv:read call site so the
;; engine's silent-ignore behavior is observable end-to-end.
;;
;; Build (D26 dev-only regenerator):
;;     wat2wasm forged_cap_claim_section.wat -o /tmp/_clean.wasm
;;     # then:
;;     # cargo run --bin forge_cap_claim_wasm_builder -- /tmp/_clean.wasm \
;;     #   forged_cap_claim_section.wasm
;; Pre-built bytes are committed alongside per D26-RESOLVED.

(module
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
