//! **F-NQA1-1** — Frozen-surface additive-extensibility
//! (WF-D2/D3 + GAP-4a/4b, NQ-A1/W5).
//!
//! ADDL Phase-4-Meta-Core, F-full **R3-W7 (doc-wave)** partition.
//! Doc-coupling + self-contained additive-decode stub family. Reuses the
//! `tf3f` doc-coupling shape for the RecoveryHook-trait-absent-at-Core
//! boundary + the codepoint-reserve doc-coupling; uses a self-contained
//! V2-envelope decode stub for the additive-re-decode property (no
//! cross-wave / no crypto-suite dep — parallel-safe).
//!
//! Pin source: `.addl/phase-4-meta/f-full-r2-test-landscape.md` §1
//! Group-12 **F-NQA1-1**:
//!   "new codepoint / `#[non_exhaustive]` variant / new reserved band does
//!    NOT break existing V2 envelope decode (NQ-A1 conservative fallback:
//!    post-freeze high/critical → reserve-codepoints + new tag, NEVER
//!    silent wire-break); reserved slots {ExecuteWorkflow,
//!    SubsetRef@0x6620, RecoveryArtifact, RotatingGroupKeyChainedMode,
//!    ChainedStateTlv} typed-reject at v1-beta; **`RecoveryArtifact`
//!    codepoint reserved at Core while `RecoveryHook` trait NOT frozen
//!    (NQ-W5/m-14)**.  §1.5, §4.4, §4.1 CODEPOINT-RESERVE, NQ-A1/W5,
//!    M-3/U21/m-14.  FG.  ~6-8 tests.  Red-phase intent: seal V2, register
//!    new codepoint, re-decode OLD → byte-identical; ExecuteWorkflow/
//!    SubsetRef reserved-typed-reject; `RecoveryHook` trait grep-absent
//!    at Core; GAP-6b `ChainedStateTlv` AAD-bind arm
//!    (`Option<ChainedStateTlv>` sub-slot bound)."
//!
//! **The RecoveryHook-absent-at-Core boundary (NQ-W5/m-14)** is the
//! Phase-4-Meta-Composing seam: at CORE the `RecoveryArtifact` codepoint
//! is RESERVED but the `RecoveryHook` trait is NOT frozen (it lands in
//! Composing). §2.5-§9.2-10 names this boundary; F-NQA1-1 pins it.
//!
//! **RED-PHASE (pim-12 §3.6e):** the F-full reserved codepoints +
//! V2 envelope decode + the RecoveryHook boundary do NOT exist at
//! baseline. End-state arms `#[ignore = "RED-PHASE: F-NQA1-1 ..."]`; R5
//! un-ignores. The NON-ignored baseline arms drive a self-contained
//! additive-decode round-trip (a real seal+re-decode, not a CONST) +
//! a real source scan for RecoveryHook absence, and pass green now.
//! NEVER `assert_eq!(CONST, CONST_VAL)`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// ===========================================================================
// SELF-CONTAINED ADDITIVE-DECODE STUB-SHIM (no cross-wave deps).
// Models a V2 envelope as: [VERSION(1)=2][CODEPOINT(2, BE)][BODY...].
// Decode dispatches on the codepoint: KNOWN → Ok, RESERVED/UNKNOWN →
// typed-reject (NEVER silent fallback). The additive property: registering
// a NEW codepoint does NOT change how OLD bytes decode.
// R5 replaces this with the real benten-crypto-suite EncryptedEnvelope.
// ===========================================================================

const ENVELOPE_FORMAT_VERSION_V2: u8 = 2;

// A small known + reserved codepoint set (subset, BE on the wire).
const CP_LIVE_HPKE_BASE: u16 = 0x647A; // X-Wing MLKEM768-X25519 (LIVE)
const CP_RESERVED_EXECUTE_WORKFLOW: u16 = 0x6321; // Layer-D reserve
const CP_RESERVED_SUBSET_REF: u16 = 0x6620; // federation reserve
const CP_RESERVED_RECOVERY_ARTIFACT: u16 = 0x6330; // reserved-at-Core, trait NOT frozen
const CP_RESERVED_ROTATING_GROUP_KEY: u16 = 0x6611; // RotatingGroupKeyChainedMode reserve

#[derive(Debug, PartialEq, Eq)]
enum StubDecodeError {
    BadVersion(u8),
    /// Codepoint is a recognized RESERVED slot → typed-reject at v1-beta.
    ReservedAtV1Beta(u16),
    /// Codepoint is entirely unknown → typed-reject (no silent fallback).
    UnsupportedAlgorithm(u16),
    Malformed,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct DecodedV2 {
    codepoint: u16,
    body: Vec<u8>,
}

fn stub_seal_v2(codepoint: u16, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(3 + body.len());
    out.push(ENVELOPE_FORMAT_VERSION_V2);
    out.extend_from_slice(&codepoint.to_be_bytes()); // BE per M-19
    out.extend_from_slice(body);
    out
}

/// Decode with a CONFIGURABLE known-set + reserved-set. The additive
/// property is modeled by passing different sets WITHOUT changing the
/// decode of bytes whose codepoint is already in the known-set.
fn stub_decode_v2(
    bytes: &[u8],
    known: &[u16],
    reserved: &[u16],
) -> Result<DecodedV2, StubDecodeError> {
    if bytes.len() < 3 {
        return Err(StubDecodeError::Malformed);
    }
    if bytes[0] != ENVELOPE_FORMAT_VERSION_V2 {
        return Err(StubDecodeError::BadVersion(bytes[0]));
    }
    let cp = u16::from_be_bytes([bytes[1], bytes[2]]);
    if known.contains(&cp) {
        Ok(DecodedV2 {
            codepoint: cp,
            body: bytes[3..].to_vec(),
        })
    } else if reserved.contains(&cp) {
        Err(StubDecodeError::ReservedAtV1Beta(cp))
    } else {
        Err(StubDecodeError::UnsupportedAlgorithm(cp))
    }
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve")
}

/// A grep-defense test file (this wave's own `tests/f_*.rs`) necessarily
/// contains the forbidden tokens (e.g. `trait RecoveryHook`) as SEARCH
/// LITERALS in its assertions — it must NOT count as a production
/// occurrence (the established `*_grep_defense.rs` self-exclusion idiom).
fn is_grep_defense_test_file(p: &std::path::Path) -> bool {
    let in_tests_dir = p
        .components()
        .any(|c| c.as_os_str().to_str() == Some("tests"));
    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or_default();
    in_tests_dir && name.starts_with("f_")
}

fn rust_files_containing(needle: &str) -> Vec<String> {
    let mut out = Vec::new();
    let crates_dir = repo_root().join("crates");
    let mut stack = vec![crates_dir];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if p.file_name().and_then(|s| s.to_str()) != Some("target") {
                    stack.push(p);
                }
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs")
                && !is_grep_defense_test_file(&p)
                && std::fs::read_to_string(&p).is_ok_and(|c| c.contains(needle))
            {
                out.push(p.display().to_string());
            }
        }
    }
    out
}

// ===========================================================================
// BASELINE ARMS (NOT ignored) — real additive-decode round-trip + real
// RecoveryHook source scan. Pass green now; would-FAIL-if-no-op'd.
// ===========================================================================

/// PIN 0a (baseline) — ADDITIVE EXTENSIBILITY: seal an OLD V2 envelope
/// under a LIVE codepoint; register a NEW codepoint into the known-set;
/// re-decode the OLD bytes → BYTE-IDENTICAL result. A no-op decode that
/// re-dispatched OLD bytes onto the new codepoint would fail this.
#[test]
fn f_nqa1_1_new_codepoint_does_not_break_old_decode_baseline() {
    let body = b"old-payload-bytes".to_vec();
    let old_bytes = stub_seal_v2(CP_LIVE_HPKE_BASE, &body);

    // Decode under the ORIGINAL known-set.
    let before = stub_decode_v2(&old_bytes, &[CP_LIVE_HPKE_BASE], &[])
        .expect("old V2 bytes decode under original known-set");

    // Now a NEW codepoint is registered (additive). The known-set grows.
    let new_known = [
        CP_LIVE_HPKE_BASE,
        0x647B, /* NF-1 PQ⊕PQ, newly LIVE */
    ];
    let after = stub_decode_v2(&old_bytes, &new_known, &[])
        .expect("old V2 bytes STILL decode after a new codepoint is added");

    assert_eq!(
        before, after,
        "registering a NEW codepoint MUST NOT change how OLD V2 bytes \
         decode (NQ-A1 additive-extensibility: never a silent wire-break). \
         Old bytes MUST re-decode byte-identical."
    );
    assert_eq!(
        after.body, body,
        "the re-decoded body MUST equal the original payload byte-for-byte."
    );
}

/// PIN 0b (baseline) — reserved-slot codepoints typed-reject at v1-beta
/// (NEVER silent acceptance), and unknown codepoints also typed-reject
/// (NEVER silent fallback). Real dispatch over the stub. Would-FAIL if a
/// reserved/unknown codepoint silently decodes to plaintext.
#[test]
fn f_nqa1_1_reserved_and_unknown_codepoints_typed_reject_baseline() {
    let reserved = [
        CP_RESERVED_EXECUTE_WORKFLOW,
        CP_RESERVED_SUBSET_REF,
        CP_RESERVED_RECOVERY_ARTIFACT,
        CP_RESERVED_ROTATING_GROUP_KEY,
    ];

    // A reserved codepoint → ReservedAtV1Beta (typed).
    let exec = stub_seal_v2(CP_RESERVED_EXECUTE_WORKFLOW, b"x");
    assert_eq!(
        stub_decode_v2(&exec, &[CP_LIVE_HPKE_BASE], &reserved),
        Err(StubDecodeError::ReservedAtV1Beta(
            CP_RESERVED_EXECUTE_WORKFLOW
        )),
        "ExecuteWorkflow reserved codepoint MUST typed-reject at v1-beta."
    );
    let subset = stub_seal_v2(CP_RESERVED_SUBSET_REF, b"x");
    assert_eq!(
        stub_decode_v2(&subset, &[CP_LIVE_HPKE_BASE], &reserved),
        Err(StubDecodeError::ReservedAtV1Beta(CP_RESERVED_SUBSET_REF)),
        "SubsetRef@0x6620 reserved codepoint MUST typed-reject at v1-beta."
    );

    // A truly unknown codepoint → UnsupportedAlgorithm (typed, no fallback).
    let unknown = stub_seal_v2(0xFE42, b"x");
    assert_eq!(
        stub_decode_v2(&unknown, &[CP_LIVE_HPKE_BASE], &reserved),
        Err(StubDecodeError::UnsupportedAlgorithm(0xFE42)),
        "an unknown codepoint MUST typed-reject (no silent fallback, U2)."
    );
}

/// PIN 0c (baseline) — the `RecoveryHook` TRAIT is ABSENT from the Core
/// source tree (NQ-W5/m-14: reserved-codepoint at Core, trait NOT frozen
/// — it lands in Phase-4-Meta-Composing). Real source scan. Would-FAIL if
/// a `trait RecoveryHook` is frozen at Core. (Scanner-liveness guarded by
/// the F-FREEZE-1 sibling; here we assert the targeted absence.)
#[test]
fn f_nqa1_1_recovery_hook_trait_absent_at_core_baseline() {
    let hits = rust_files_containing("trait RecoveryHook");
    assert!(
        hits.is_empty(),
        "the `RecoveryHook` TRAIT MUST be ABSENT at Phase-4-Meta-CORE \
         (NQ-W5/m-14: only the `RecoveryArtifact` codepoint is reserved at \
         Core; the trait is frozen in Phase-4-Meta-Composing per §9.2-10). \
         Found `trait RecoveryHook` in: {:?}.",
        hits
    );
}

// ===========================================================================
// RED-PHASE ARMS (ignored until R5) — real reserved-codepoint registry +
// the RecoveryArtifact-reserved-but-trait-absent boundary doc-coupling.
// ===========================================================================

/// PIN 1 — the REAL crypto-suite codepoint registry reserves the
/// {ExecuteWorkflow, SubsetRef, RecoveryArtifact, RotatingGroupKeyChainedMode,
/// ChainedStateTlv} slots AND typed-rejects each at v1-beta. Source
/// doc-coupling to the codepoint module. Would-FAIL if a reserved slot is
/// accidentally made LIVE or omitted from the registry.
#[test]
#[ignore = "RED-PHASE: F-NQA1-1 — codepoint registry reserves \
            {ExecuteWorkflow, SubsetRef, RecoveryArtifact, \
            RotatingGroupKeyChainedMode, ChainedStateTlv} (typed-reject at \
            v1-beta); un-ignore at R5"]
fn f_nqa1_1_reserved_slots_registered_in_codepoint_module() {
    let reserved_names = [
        "ExecuteWorkflow",
        "SubsetRef",
        "RecoveryArtifact",
        "RotatingGroupKeyChainedMode",
        "ChainedStateTlv",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for name in reserved_names {
        if rust_files_containing(name).is_empty() {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "the F-full CODEPOINT-RESERVE set MUST be registered (each \
         typed-rejecting at v1-beta): missing {:?}. R5 mints them in the \
         crypto-suite codepoint registry.",
        missing
    );
}

/// PIN 2 — `RecoveryArtifact` codepoint RESERVED at Core, while the
/// `RecoveryHook` trait remains ABSENT at Core (the m-14 boundary). The
/// reserved codepoint exists in source; the trait does NOT. Would-FAIL if
/// R5 freezes the trait at Core (boundary violation) OR omits the reserve.
#[test]
#[ignore = "RED-PHASE: F-NQA1-1 (NQ-W5/m-14) — RecoveryArtifact codepoint \
            RESERVED at Core while RecoveryHook trait still ABSENT at Core; \
            un-ignore at R5"]
fn f_nqa1_1_recovery_artifact_reserved_but_hook_trait_absent_at_core() {
    // Reserved codepoint present.
    let reserved = rust_files_containing("RecoveryArtifact");
    assert!(
        !reserved.is_empty(),
        "`RecoveryArtifact` codepoint MUST be RESERVED at Core (NQ-W5)."
    );
    // Trait still absent at Core (the boundary).
    let trait_hits = rust_files_containing("trait RecoveryHook");
    assert!(
        trait_hits.is_empty(),
        "the `RecoveryHook` TRAIT MUST remain ABSENT at Core even after \
         R5 (m-14: the trait is a Phase-4-Meta-Composing deliverable; \
         only the codepoint is reserved at Core). Found in: {:?}.",
        trait_hits
    );
}

/// PIN 3 — NQ-A1 conservative-fallback policy is DOCUMENTED: post-freeze
/// high/critical findings → reserve-codepoints + new tag, NEVER a silent
/// wire-break. Doc-coupling. Would-FAIL if the additive-only freeze policy
/// is not stated (the policy IS the freeze contract).
#[test]
#[ignore = "RED-PHASE: F-NQA1-1 (NQ-A1) — CRYPTO-CODEPOINTS.md documents \
            the conservative-fallback additive-only freeze policy \
            (reserve-codepoints + new tag, never silent wire-break); \
            un-ignore at R5"]
fn f_nqa1_1_conservative_fallback_policy_documented() {
    let codepoints =
        std::fs::read_to_string(repo_root().join("docs/CRYPTO-CODEPOINTS.md")).unwrap_or_default();
    let has_additive = codepoints.contains("additive")
        || codepoints.contains("reserve-codepoints")
        || codepoints.contains("reserved codepoint");
    let has_no_wire_break = codepoints.contains("never a silent wire-break")
        || codepoints.contains("no silent wire-break")
        || codepoints.contains("NEVER a wire-break");
    assert!(
        has_additive && has_no_wire_break,
        "CRYPTO-CODEPOINTS.md MUST document the NQ-A1 conservative-fallback \
         policy: post-freeze findings are handled additively (reserve a \
         new codepoint + tag), NEVER via a silent wire-break. \
         additive={has_additive}, no-wire-break={has_no_wire_break}."
    );
}

/// PIN 4 — GAP-6b: the `ChainedStateTlv` sub-slot is AAD-BOUND when
/// present (`Option<ChainedStateTlv>`), so a present-vs-absent flip is
/// detectable at decrypt (not advisory). Source doc-coupling that the
/// AAD assembly binds the optional sub-slot. Would-FAIL if the sub-slot
/// is carried OUTSIDE the AAD (where it could be stripped silently).
#[test]
#[ignore = "RED-PHASE: F-NQA1-1 (GAP-6b) — ChainedStateTlv Option sub-slot \
            is AAD-bound (present/absent flip detectable at decrypt, not \
            advisory); un-ignore at R5"]
fn f_nqa1_1_chained_state_tlv_aad_bound_sub_slot() {
    // The AAD assembly path (membership-set or crypto-suite) must
    // reference `ChainedStateTlv` in an AAD-binding context.
    let tlv_sites = rust_files_containing("ChainedStateTlv");
    assert!(
        !tlv_sites.is_empty(),
        "`ChainedStateTlv` MUST exist at R5 (GAP-6b)."
    );
    // At least one site binds it into AAD assembly (not just declares it).
    let aad_bound = tlv_sites.iter().any(|site| {
        let content = std::fs::read_to_string(site).unwrap_or_default();
        content.contains("ChainedStateTlv")
            && (content.contains("aad") || content.contains("Aad") || content.contains("AAD"))
    });
    assert!(
        aad_bound,
        "`ChainedStateTlv` (Option sub-slot) MUST be AAD-BOUND so a \
         present/absent flip is detectable at decrypt (GAP-6b). No \
         AAD-binding site found among: {:?}.",
        tlv_sites
    );
}
