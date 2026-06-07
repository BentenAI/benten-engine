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
//! Composing). §3.4 / §9.2-10 names this boundary; F-NQA1-1 pins it.
//!
//! **Codepoint-reserve fidelity (F4-010, R4.3-corrected 2026-06-03).**
//! The reserve integers used below are the ONLY ones the canonical
//! `f-full-r0-plan` **§4.0** allocation table (R0.5) actually blesses:
//!   - `SubsetRef` → **`0x6620`** (`MEMBERSHIP_SET_SUBSET_REF`,
//!     §4.0 line for the MembershipSet band).
//!   - `ExecuteWorkflow` → lives **inside the `0x6320..0x632F`
//!     RemotePermission band** (§4.0: "incl.
//!     `ExecuteWorkflow` reserve"). §4.0 freezes
//!     the *band*, not a standalone integer, so
//!     we register the band base `0x6320` and
//!     assert band-membership, NOT a fabricated
//!     `0x6321`.
//!   - `RecoveryArtifact` → §4.0 names **NO codepoint integer** for
//!     it at Core (it is a *conceptual*
//!     reserve-at-Core per §3.4 / §9.2-10; the
//!     codepoint is allocated in Composing
//!     alongside the trait). We therefore do
//!     NOT invent a `0x6330`; the boundary is
//!     pinned by the source-scan arms (PIN 0c /
//!     PIN 2), not by an integer.
//!   - `RotatingGroupKeyChainedMode` / `ChainedStateTlv` → the chained-mode
//!     reserve lives in the FS-future
//!     `0x6380..0x63CF` MLS/CGKA brackets +
//!     the per-stanza `Option<ChainedStateTlv>`
//!     sub-slot (§3.3 line "Per-stanza
//!     `Option<ChainedStateTlv>` codepoint-
//!     reserve sub-slot"). It is NOT a
//!     standalone `0x6611` (and `0x6610` is the
//!     LIVE-FREEZE `MEMBERSHIP_SET_GROUP_MULTI_
//!     STANZA`, so `0x6611` would alias the
//!     membership band). We register the
//!     FS-future bracket base `0x6380` for the
//!     typed-reject property and pin the
//!     sub-slot AAD-binding by NAME (PIN 4).
//! The prior corpus (`0x6321`/`0x6330`/`0x6611`) invented three integers
//! no §4.0 row blesses — an additive-extensibility test must not invent
//! the very codepoints it claims are reserved.
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

// R6-R3 (F-03/F-13): drive the REAL crypto-suite reserve surface (not just
// name-greps). `benten-crypto-suite` is a production dependency of
// `benten-drop`, so the behavioral arms below exercise the actual
// `ReservedCodepoint::resolve()` typed-reject + the GAP-6b
// `chained_state_tlv_aad_binding` AAD assembly byte-for-byte.
use benten_crypto_suite::codepoint::{ReservedCodepoint, chained_state_tlv_aad_binding};

// ===========================================================================
// SELF-CONTAINED ADDITIVE-DECODE STUB-SHIM (no cross-wave deps).
// Models a V2 envelope as: [VERSION(1)=2][CODEPOINT(2, BE)][BODY...].
// Decode dispatches on the codepoint: KNOWN → Ok, RESERVED/UNKNOWN →
// typed-reject (NEVER silent fallback). The additive property: registering
// a NEW codepoint does NOT change how OLD bytes decode.
// R5 replaces this with the real benten-crypto-suite EncryptedEnvelope.
// ===========================================================================

const ENVELOPE_FORMAT_VERSION_V2: u8 = 2;

// Codepoints below are the §4.0-blessed values ONLY (F4-010). BE on wire.
const CP_LIVE_HPKE_BASE: u16 = 0x647A; // X-Wing MLKEM768-X25519 (LIVE; §4.0)

/// `ExecuteWorkflow` reserve. §4.0 freezes the **`0x6320..0x632F`
/// RemotePermission band** ("incl. `ExecuteWorkflow` reserve"), not a
/// standalone integer. We register the band BASE and assert band-membership.
const CP_BAND_REMOTE_PERMISSION_BASE: u16 = 0x6320; // §4.0 RemotePermission band
const CP_BAND_REMOTE_PERMISSION_END: u16 = 0x632F;

/// `SubsetRef` federation reserve — the ONE standalone integer §4.0 names
/// in this cluster: `MEMBERSHIP_SET_SUBSET_REF = 0x6620` (refused at
/// v1-beta; Inv-20 clause-k).
const CP_RESERVED_SUBSET_REF: u16 = 0x6620; // §4.0 MembershipSet band

/// `RotatingGroupKeyChainedMode` / `ChainedStateTlv` chained-mode reserve.
/// §4.0 places the FS-future / chained reserves in the
/// **`0x6380..0x63CF`** MLS/CGKA brackets (NOT a standalone `0x6611`;
/// `0x6610` is the LIVE-FREEZE `MEMBERSHIP_SET_GROUP_MULTI_STANZA`). We
/// register the bracket BASE for the typed-reject property; the per-stanza
/// sub-slot AAD-binding is pinned by NAME (PIN 4).
const CP_BAND_FS_FUTURE_BASE: u16 = 0x6380; // §4.0 MLS-Application / FS-future bracket

// NOTE (F4-010): `RecoveryArtifact` has NO §4.0 codepoint integer at Core
// (it is a conceptual reserve-at-Core per §3.4 / §9.2-10; the integer is
// allocated in Composing alongside the trait). We deliberately do NOT mint
// a `0x6330` here — the boundary is pinned by the source-scan arms below.

// ---------------------------------------------------------------------------
// §4.0-PROVENANCE GUARD (F4-010 self-enforcing regression guard).
//
// The F4-010 bug was: this additive-extensibility test *invented* three
// codepoints (`0x6321`/`0x6330`/`0x6611`) that no §4.0 row blesses. The
// prose discipline ("use §4.0-blessed values only") is not self-enforcing —
// a future edit could re-introduce a fabricated standalone integer and the
// test would still pass. To make the fix permanent, every codepoint this
// test treats as RESERVED must be provably traceable to §4.0: either a
// §4.0-NAMED STANDALONE integer, or a member of a §4.0-NAMED BAND base.
// `cp_blessed_by_s40` is the single source of truth; PIN 0b asserts its
// reserved set passes it, so any future fabricated standalone integer
// fails the test loudly (not silently).
// ---------------------------------------------------------------------------

/// The §4.0-named STANDALONE reserve integers this test legitimately uses.
/// (Only `SubsetRef@0x6620` qualifies in this cluster — §4.0 line for
/// `MEMBERSHIP_SET_SUBSET_REF`.)
const S40_NAMED_STANDALONE_RESERVES: &[u16] = &[CP_RESERVED_SUBSET_REF];

/// The §4.0-named BAND `(base, end)` ranges this test uses a base of.
/// `0x6320..0x632F` RemotePermission (holds `ExecuteWorkflow`); the
/// `0x6380..0x63CF` FS-future bracket (holds the chained-mode reserve —
/// §4.0 spans MLS-Application `0x6380` through draft-prabel `0x63CF`).
const S40_NAMED_BANDS: &[(u16, u16)] = &[
    (
        CP_BAND_REMOTE_PERMISSION_BASE,
        CP_BAND_REMOTE_PERMISSION_END,
    ), // 0x6320..0x632F
    (CP_BAND_FS_FUTURE_BASE, 0x63CF), // 0x6380..0x63CF FS-future bracket
];

/// True iff `cp` is traceable to §4.0: a NAMED standalone integer OR inside
/// a NAMED band. A fabricated integer (the F4-010 regression) returns false.
fn cp_blessed_by_s40(cp: u16) -> bool {
    S40_NAMED_STANDALONE_RESERVES.contains(&cp)
        || S40_NAMED_BANDS
            .iter()
            .any(|&(base, end)| cp >= base && cp <= end)
}

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
///
/// The reserved set uses ONLY §4.0-blessed values (F4-010): the
/// RemotePermission band base (`0x6320`, holds `ExecuteWorkflow`), the
/// `SubsetRef` integer (`0x6620`), and the FS-future bracket base
/// (`0x6380`, holds the chained-mode reserve). `RecoveryArtifact` is
/// intentionally absent (no §4.0 integer at Core).
///
/// This arm ALSO carries the F4-010 self-enforcing guard: it asserts every
/// reserved codepoint is §4.0-traceable via `cp_blessed_by_s40`, and that
/// the three fabricated integers the prior corpus invented
/// (`0x6321`/`0x6330`/`0x6611`) are PROVABLY rejected by it — so a revert
/// re-minting any of them fails this test loudly.
#[test]
fn f_nqa1_1_reserved_and_unknown_codepoints_typed_reject_baseline() {
    let reserved = [
        CP_BAND_REMOTE_PERMISSION_BASE, // ExecuteWorkflow reserve (band base)
        CP_RESERVED_SUBSET_REF,         // 0x6620, §4.0-named
        CP_BAND_FS_FUTURE_BASE,         // RotatingGroupKeyChainedMode reserve (bracket base)
    ];

    // F4-010 SELF-ENFORCING GUARD: every codepoint we treat as RESERVED MUST
    // trace to §4.0 (a NAMED standalone integer OR a NAMED band member). This
    // is the permanent regression guard — a future edit that re-introduces a
    // fabricated standalone integer (the original `0x6321`/`0x6330`/`0x6611`)
    // fails HERE, loudly, instead of silently passing.
    for &cp in &reserved {
        assert!(
            cp_blessed_by_s40(cp),
            "F4-010: reserved codepoint {cp:#06x} is NOT traceable to the \
             §4.0 allocation table (not a named standalone, not inside a \
             named band). An additive-extensibility test MUST NOT invent \
             the codepoints it claims are reserved."
        );
    }
    // And the converse: the FABRICATED-STANDALONE integers the prior corpus
    // invented are PROVABLY not §4.0-blessed (demonstrates would-FAIL on a
    // revert that re-mints them as STANDALONE reserves). `0x6611` additionally
    // aliases the LIVE `MEMBERSHIP_SET_GROUP_MULTI_STANZA` band (`0x6610`),
    // which is the sharpest reason it could never be a fresh reserve.
    //
    // R5-RECONCILE (F4-010): `0x6321` is REMOVED from this list — it is a
    // genuine MEMBER of the §4.0-named RemotePermission band `0x6320..0x632F`
    // (which holds the `ExecuteWorkflow` reserve), so `cp_blessed_by_s40`
    // CORRECTLY blesses it as a band member. The F4-010 hazard the guard
    // closes is a fabricated *standalone* integer OUTSIDE every named band;
    // `0x6330` (between `0x632F` and `0x6380`) and `0x6611` (between the
    // membership `0x6610` LIVE-freeze and the `0x6620` SubsetRef reserve) are
    // the genuine outside-every-band fabrications. The prior list conflated
    // band-membership (legitimate) with standalone-fabrication (the hazard);
    // asserting `0x6321` is "not blessed" contradicted the band base
    // `0x6320`'s own membership and was a test-internal inconsistency.
    // [FLAG-FOR-BEN — courtesy: this is a corpus test-bug reconciliation, not
    //  a wire/golden change; the F4-010 self-enforcing guard still rejects any
    //  fabricated STANDALONE integer outside the named bands.]
    for &fabricated in &[0x6330u16, 0x6611u16] {
        assert!(
            !cp_blessed_by_s40(fabricated),
            "F4-010: {fabricated:#06x} was a FABRICATED STANDALONE integer (no \
             §4.0 band or standalone row blesses it); the guard MUST reject it."
        );
    }

    // `ExecuteWorkflow` band base → ReservedAtV1Beta (typed).
    let exec = stub_seal_v2(CP_BAND_REMOTE_PERMISSION_BASE, b"x");
    assert_eq!(
        stub_decode_v2(&exec, &[CP_LIVE_HPKE_BASE], &reserved),
        Err(StubDecodeError::ReservedAtV1Beta(
            CP_BAND_REMOTE_PERMISSION_BASE
        )),
        "ExecuteWorkflow reserve (in the §4.0 0x6320..0x632F \
         RemotePermission band) MUST typed-reject at v1-beta."
    );
    // §4.0 freezes the BAND, not a single integer: every value in the
    // band is a RemotePermission codepoint. Pin the band invariant so a
    // future fabricated standalone integer (the F4-010 regression) is
    // caught.
    const {
        assert!(
            CP_BAND_REMOTE_PERMISSION_BASE <= CP_BAND_REMOTE_PERMISSION_END
                && (CP_BAND_REMOTE_PERMISSION_END - CP_BAND_REMOTE_PERMISSION_BASE) == 0x0F,
            "the §4.0 RemotePermission band MUST be 0x6320..0x632F (16 slots); \
             `ExecuteWorkflow` is a reserve WITHIN it, not a standalone integer."
        );
    }

    let subset = stub_seal_v2(CP_RESERVED_SUBSET_REF, b"x");
    assert_eq!(
        stub_decode_v2(&subset, &[CP_LIVE_HPKE_BASE], &reserved),
        Err(StubDecodeError::ReservedAtV1Beta(CP_RESERVED_SUBSET_REF)),
        "SubsetRef@0x6620 reserved codepoint MUST typed-reject at v1-beta \
         (§4.0 MEMBERSHIP_SET_SUBSET_REF)."
    );

    let chained = stub_seal_v2(CP_BAND_FS_FUTURE_BASE, b"x");
    assert_eq!(
        stub_decode_v2(&chained, &[CP_LIVE_HPKE_BASE], &reserved),
        Err(StubDecodeError::ReservedAtV1Beta(CP_BAND_FS_FUTURE_BASE)),
        "RotatingGroupKeyChainedMode reserve (in the §4.0 0x6380..0x63CF \
         FS-future bracket) MUST typed-reject at v1-beta."
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
         crypto-suite codepoint registry (each at its §4.0-named band: \
         ExecuteWorkflow in 0x6320..0x632F, SubsetRef@0x6620, \
         RecoveryArtifact codepoint allocated alongside the Composing \
         trait, chained-mode in 0x6380..0x63CF).",
        missing
    );

    // R6-R3 (F-13) BEHAVIORAL ARM — drive the REAL registry, not just a
    // name-grep. EVERY enumerated `ReservedCodepoint` MUST typed-reject via
    // its production `resolve()` at v1-beta (NEVER a silent accept). A revert
    // that made any slot LIVE-accept (`Ok(())`) fails HERE, loudly. The
    // `#[non_exhaustive]` enum's variants are listed explicitly so a future
    // variant addition that ships an accepting `resolve()` arm is also caught
    // (the match below would need updating + the assertion would fire).
    for reserved in [
        ReservedCodepoint::ExecuteWorkflow,
        ReservedCodepoint::SubsetRef,
        ReservedCodepoint::RecoveryArtifact,
        ReservedCodepoint::RotatingGroupKeyChainedMode,
        ReservedCodepoint::ChainedStateTlv,
    ] {
        assert!(
            reserved.resolve().is_err(),
            "reserved codepoint {reserved:?} MUST typed-reject via its \
             production `resolve()` at v1-beta (NEVER a silent accept). A \
             reserve that returns Ok(()) is a silent wire-acceptance — the \
             exact NQ-A1 failure this freeze contract forbids."
        );
    }
}

/// PIN 2 — `RecoveryArtifact` codepoint RESERVED at Core, while the
/// `RecoveryHook` trait remains ABSENT at Core (the m-14 boundary). The
/// reserved codepoint exists in source; the trait does NOT. Would-FAIL if
/// R5 freezes the trait at Core (boundary violation) OR omits the reserve.
#[test]
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

    // R6-R3 (F-03/F-13) BEHAVIORAL ARM — drive the REAL GAP-6b AAD assembly
    // (`chained_state_tlv_aad_binding`) and byte-assert the exact wire bytes,
    // not just a name-grep. A present sub-slot pushes `0x01 ‖ band_base_be`
    // (FS-future bracket base `0x6380`, big-endian per M-19); an absent
    // sub-slot pushes `0x00`. The present/absent prefix byte DIFFERS, which is
    // precisely what makes a present-vs-absent flip detectable when these
    // bytes are folded into the AEAD AAD. A revert that stopped binding the
    // codepoint (e.g. emitted `[0x01]` with no band base, or carried the
    // sub-slot OUTSIDE the AAD) fails these byte-asserts loudly.
    let present = chained_state_tlv_aad_binding(true);
    assert_eq!(
        present,
        vec![0x01, 0x63, 0x80],
        "GAP-6b: a PRESENT `ChainedStateTlv` sub-slot MUST AAD-bind as \
         `0x01 ‖ 0x6380_be` (present-flag ‖ FS-future bracket base, BE)."
    );
    let absent = chained_state_tlv_aad_binding(false);
    assert_eq!(
        absent,
        vec![0x00],
        "GAP-6b: an ABSENT `ChainedStateTlv` sub-slot MUST AAD-bind as a \
         single `0x00` present-flag byte."
    );
    assert_ne!(
        present[0], absent[0],
        "GAP-6b: the present/absent flag byte MUST differ so a relay that \
         strips (or injects) the sub-slot flips the AAD and fails AEAD-open \
         — the sub-slot cannot be silently removed."
    );
    // Provenance: the bound band base MUST be the §4.0 FS-future bracket base
    // the `ReservedCodepoint::ChainedStateTlv` reserve dispatches from.
    let band_base_be = ReservedCodepoint::ChainedStateTlv
        .band_base()
        .expect("ChainedStateTlv reserve dispatches from a §4.0 band base")
        .to_be_bytes();
    assert_eq!(
        &present[1..3],
        &band_base_be,
        "the AAD-bound band base MUST equal the `ChainedStateTlv` reserve's \
         own §4.0 `band_base()` (no fabricated integer)."
    );
}
