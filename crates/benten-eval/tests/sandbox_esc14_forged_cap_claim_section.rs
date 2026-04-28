//! Phase 2b G7-A — ESC-14 forged-cap-claim regression test.
//!
//! **wsa-g7a-mr-9 fix-pass:** ESC-14 closure depends on the ABSENCE of
//! a behavior — the engine MUST silently ignore custom WASM sections
//! when deriving caps. Absence-of-behavior is the easiest regression to
//! land accidentally (e.g. a future wasmtime feature reads custom
//! sections by default), so the design intent is locked in by this
//! test.
//!
//! Strategy: synthesize a wasm module with an injected custom section
//! that names `host:*:*` (a forged universal cap claim) + dispatch the
//! module via `execute()` with a manifest that does NOT grant kv:read.
//! The expected outcome is `E_SANDBOX_HOST_FN_DENIED` (engine consults
//! the call-time manifest exclusively); a regression where the engine
//! reads the custom section would surface as `Ok(_)` (universal grant)
//! or a different denial path that named the embedded claim.
//!
//! The .wat source documenting the strategy lives at
//! `tests/fixtures/sandbox/escape/forged_cap_claim_section.wat`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{CapBundle, ManifestRef, ManifestRegistry, SandboxConfig, execute};

/// Append a custom-section to the given wasm bytes naming `host:*:*`
/// as a forged cap-claim. The wasm binary format permits arbitrary
/// `id=0` "custom sections" trailing the well-formed module body —
/// they are spec-mandated to be IGNORED by execution and any cap
/// derivation. Engineering risk: a future wasmtime feature OR a
/// codegen mistake reads the section + treats it as authoritative.
fn append_forged_custom_section(mut bytes: Vec<u8>) -> Vec<u8> {
    // Custom-section format: id=0 (1 byte) + section size (LEB128 u32) +
    // name length (LEB128 u32) + name (UTF-8) + payload.
    let name = b"benten:forged_caps";
    let payload = b"requires:host:*:*";

    fn leb128_u32(mut x: u32, out: &mut Vec<u8>) {
        loop {
            let mut byte = (x & 0x7f) as u8;
            x >>= 7;
            if x != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if x == 0 {
                break;
            }
        }
    }

    // Build the section payload: name-length + name + payload.
    let mut inner = Vec::new();
    leb128_u32(name.len() as u32, &mut inner);
    inner.extend_from_slice(name);
    inner.extend_from_slice(payload);

    bytes.push(0u8); // id = 0 (custom section)
    leb128_u32(inner.len() as u32, &mut bytes);
    bytes.extend_from_slice(&inner);

    bytes
}

fn dummy_attribution() -> AttributionFrame {
    let zero = Cid::from_blake3_digest([0u8; 32]);
    AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
    }
}

#[test]
fn esc14_forged_custom_section_does_not_grant_caps() {
    // Build the module from the wat source (mirrors the documented
    // .wat at fixtures/sandbox/escape/forged_cap_claim_section.wat
    // but uses an empty body since the executor scaffold doesn't
    // invoke the module — we're locking the cap-derivation pathway).
    let clean = wat::parse_str("(module)").unwrap();

    let forged = append_forged_custom_section(clean.clone());
    assert!(
        forged.len() > clean.len(),
        "appended bytes should be longer than the clean module"
    );

    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();

    // Manifest claims kv:read but the grant doesn't include it. The
    // executor MUST fire SandboxHostFnDenied — confirming cap derivation
    // is exclusively from the call-time manifest, NOT the embedded
    // custom section's `requires:host:*:*` claim.
    let inline = CapBundle::new(vec!["host:compute:kv:read".to_string()], None);
    let err = execute(
        &forged,
        ManifestRef::Inline(inline),
        &registry,
        SandboxConfig::default(),
        // Grant excludes kv:read — would be authorised IF the embedded
        // section's `host:*:*` were honoured.
        &["host:compute:time".to_string()],
        &attribution,
    )
    .unwrap_err();
    assert_eq!(
        err.code(),
        ErrorCode::SandboxHostFnDenied,
        "ESC-14 — engine MUST consult the call-time manifest+grant ONLY; \
         embedded custom section claims MUST be silently ignored"
    );

    // Bonus check: the same call with a CLEAN module + same manifest +
    // same grant ALSO fires SandboxHostFnDenied — confirming the forged
    // section neither grants caps nor changes the denial behavior in
    // any other way (i.e., the engine treats the bytes identically).
    let inline2 = CapBundle::new(vec!["host:compute:kv:read".to_string()], None);
    let err2 = execute(
        &clean,
        ManifestRef::Inline(inline2),
        &registry,
        SandboxConfig::default(),
        &["host:compute:time".to_string()],
        &attribution,
    )
    .unwrap_err();
    assert_eq!(
        err2.code(),
        ErrorCode::SandboxHostFnDenied,
        "clean module produces identical denial — forged section is byte-noise"
    );
}
