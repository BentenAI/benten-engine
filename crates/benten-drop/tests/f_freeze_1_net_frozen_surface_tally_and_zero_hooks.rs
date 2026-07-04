//! **F-FREEZE-1** — Net-frozen-surface −8 tally + ZERO-hook confirmations
//! (GN-24 + GAP-5b, O-8).
//!
//! ADDL Phase-4-Meta-Core, F-full **R3-W7 (doc-wave)** partition.
//! Doc-coupling / grep-defense family (reuses the `tf3f` shape +
//! the `*_grep_assert.rs` / `*_grep_defense.rs` source-scan idiom).
//!
//! Pin source: `.addl/phase-4-meta/f-full-r2-test-landscape.md` §1
//! Group-12 **F-FREEZE-1**:
//!   "GN wins shrink frozen surface −8 net (−1 audit structure, −1
//!    `MembershipEvent` wire-enum, −4 AuditAccessGradation codepoints,
//!    −2 Garden/Grove sub-codepoints; R0.1 '−6' double-count corrected —
//!    O-8); `economic_policy` DROPPED (ZERO MembershipSet freeze hook);
//!    ZERO new member field; `PeerResource/ResourceKind/OwnerRef/
//!    CommunityEconomicPolicy` absent from frozen wire (D-28/D-29
//!    PHASE-LATER-DEFER).  §4.4 O-8, §2.8 CE-1, §4.3, §9.1-6.  FG/CF.
//!    ~6-8 tests.  Red-phase intent: `MembershipEvent` NOT frozen
//!    wire-enum (version-Node content); 4 AuditAccessGradation NOT
//!    codepoints; no Garden/Grove sub-codepoints; `MembershipSetPolicy`
//!    no `economic_policy` field (grep-defense); compute surfaces absent."
//!
//! **SHIPPED (R17 retense; formerly RED-PHASE pim-12 §3.6e):** the F-full
//! freeze surface (the MembershipSet structs, the AuditAccessGradation type,
//! the GovernanceConfig tiers) now EXISTS at HEAD and every arm is a live
//! `#[test]` (NO `#[ignore]`). The prior RED-PHASE staging — where the
//! end-state tally / ZERO-hook arms were `#[ignore = "RED-PHASE: F-FREEZE-1
//! ..."]` pending the membership-set crate + the V1-FROZEN D-28/D-29 rows +
//! the codepoint doc — is fully discharged; the end-state arms below run green.
//!
//! The baseline arms drive REAL source scans (the absence of
//! the named tokens at HEAD is the would-FAIL-if-no-op'd property: a stub
//! that smuggles in `economic_policy` or a `MembershipEvent` wire-enum
//! fails the baseline absence pin). NEVER `assert_eq!(CONST, CONST_VAL)`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// ===========================================================================
// SELF-CONTAINED SOURCE-SCAN SHIM (no cross-wave module deps).
// Scans the crates/ + docs/ tree from CARGO_MANIFEST_DIR. R5 keeps these
// scanners; they read the real tree, not a literal.
// ===========================================================================

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve from crates/benten-drop")
}

/// Recursively collect `*.rs` files under `crates/`.
fn all_rust_sources() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let crates_dir = repo_root().join("crates");
    collect_ext(&crates_dir, "rs", &mut out);
    out
}

fn collect_ext(dir: &std::path::Path, ext: &str, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // skip target/ build dirs to keep the scan cheap + deterministic
            if path.file_name().and_then(|s| s.to_str()) == Some("target") {
                continue;
            }
            collect_ext(&path, ext, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

/// A grep-defense test file (this wave's own `tests/f_*.rs`) necessarily
/// contains the forbidden tokens as SEARCH LITERALS in its assertions —
/// it must NOT count as a production occurrence (the established
/// `*_grep_defense.rs` self-exclusion idiom). We exclude any `.rs` file
/// living under a `tests/` directory whose name starts with `f_`
/// (the F-full doc-wave grep-defense family).
fn is_grep_defense_test_file(p: &std::path::Path) -> bool {
    let in_tests_dir = p
        .components()
        .any(|c| c.as_os_str().to_str() == Some("tests"));
    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or_default();
    in_tests_dir && name.starts_with("f_")
}

/// Count source files containing `needle` (substring scan over `.rs`),
/// EXCLUDING this wave's own grep-defense test files (self-exclusion).
fn rust_files_containing(needle: &str) -> Vec<String> {
    all_rust_sources()
        .into_iter()
        .filter(|p| !is_grep_defense_test_file(p))
        .filter(|p| std::fs::read_to_string(p).is_ok_and(|c| c.contains(needle)))
        .map(|p| p.display().to_string())
        .collect()
}

fn v1_frozen_doc() -> String {
    std::fs::read_to_string(repo_root().join("docs/V1-FROZEN-INTERFACE-DEFERRED.md"))
        .expect("V1-FROZEN-INTERFACE-DEFERRED.md must be present")
}

// ===========================================================================
// BASELINE ARMS (NOT ignored) — the source scanner runs over the REAL tree.
// At baseline the forbidden tokens are ABSENT; these pass green now and
// are the would-FAIL-if-no-op'd guard (a real token leak fails them even
// pre-R5).
// ===========================================================================

/// PIN 0 (baseline) — `economic_policy` is ABSENT from the source tree
/// today. The MembershipSet freeze MUST keep it that way (ZERO economic
/// freeze hook). The scanner reads the real tree — a no-op scanner
/// returning `[]` would let a leak through silently, so we also assert
/// the scanner finds a KNOWN-present token to prove it isn't inert.
#[test]
fn f_freeze_1_scanner_is_live_and_economic_policy_absent_baseline() {
    // Scanner liveness: a token we KNOW is present in the tree must be found.
    let present = rust_files_containing("PrimitiveKind");
    assert!(
        !present.is_empty(),
        "source scanner MUST be live: `PrimitiveKind` is present in the \
         tree and MUST be found. An inert (always-empty) scanner would \
         make every absence-pin vacuously pass."
    );

    // Baseline absence: `economic_policy` is not in the tree today.
    let hits = rust_files_containing("economic_policy");
    assert!(
        hits.is_empty(),
        "`economic_policy` MUST be ABSENT (ZERO MembershipSet economic \
         freeze hook, O-8). Found in: {:?}. If R5 introduces it on \
         `MembershipSetPolicy`, the field MUST be dropped before freeze.",
        hits
    );
}

// ===========================================================================
// FREEZE END-STATE ARMS (SHIPPED; formerly RED-PHASE, un-ignored) — assert
// the F-full freeze end-state. All live `#[test]` at HEAD.
// ===========================================================================

/// PIN 1 — `MembershipSetPolicy` carries NO `economic_policy` field.
/// (grep-defense, end-state). When the membership-set crate lands at R5,
/// the policy struct MUST NOT regrow the economic hook. Would-FAIL if a
/// `MembershipSetPolicy` with an `economic_policy:` field appears.
#[test]
fn f_freeze_1_membership_set_policy_no_economic_policy_field() {
    // End-state: the policy type EXISTS but the field does NOT.
    let policy_sites = rust_files_containing("MembershipSetPolicy");
    assert!(
        !policy_sites.is_empty(),
        "at R5 the `MembershipSetPolicy` type MUST exist (membership-set \
         crate landed)."
    );
    let econ_field = rust_files_containing("economic_policy");
    assert!(
        econ_field.is_empty(),
        "`MembershipSetPolicy` MUST carry NO `economic_policy` field \
         (ZERO economic freeze hook per O-8). Found `economic_policy` in: \
         {:?}.",
        econ_field
    );
}

/// PIN 2 — `MembershipEvent` is NOT a frozen wire-enum: it is
/// version-Node content (the −1 wire-enum win in the −8 tally). The
/// grep-defense asserts no `#[non_exhaustive]`/codepoint-tagged
/// `MembershipEvent` wire enum is frozen. Would-FAIL if `MembershipEvent`
/// reappears as a serialized wire enum with a codepoint.
#[test]
fn f_freeze_1_membership_event_not_a_frozen_wire_enum() {
    // If `MembershipEvent` exists at R5 it must NOT be paired with a
    // wire-codepoint constant (the marker of a frozen wire enum).
    let sites = rust_files_containing("MembershipEvent");
    for site in &sites {
        let content = std::fs::read_to_string(site).unwrap_or_default();
        // A frozen wire-enum would carry a `MEMBERSHIP_EVENT` codepoint
        // const in the `0x6xxx` band. Its presence is the regression.
        assert!(
            !content.contains("MEMBERSHIP_EVENT_CODEPOINT")
                && !content.contains("MembershipEventCodepoint"),
            "`MembershipEvent` MUST be version-Node CONTENT, not a frozen \
             wire-enum. A codepoint-tagged `MembershipEvent` in {} \
             regresses the −1 wire-enum win (O-8 net −8 tally).",
            site
        );
    }
}

/// PIN 3 — the 4 `AuditAccessGradation` variants are NOT codepoints
/// (−4 in the −8 tally). They are a `RestrictedScope` arm + UCAN-caveat
/// compositions (per F-AUDIT-3). Would-FAIL if `AUDIT_ACCESS_*` codepoint
/// constants are minted in the `0x6xxx` band.
#[test]
fn f_freeze_1_audit_access_gradation_not_codepoints() {
    let sites = rust_files_containing("AuditAccessGradation");
    for site in &sites {
        let content = std::fs::read_to_string(site).unwrap_or_default();
        assert!(
            !content.contains("AUDIT_ACCESS_GRADATION_CODEPOINT")
                && !content.contains("const AUDIT_ACCESS"),
            "`AuditAccessGradation` variants MUST NOT be codepoints (−4 in \
             the O-8 −8 tally); they are a RestrictedScope arm. Codepoint \
             const found in {}.",
            site
        );
    }
}

/// PIN 4 — NO Garden/Grove SUB-codepoints (−2 in the −8 tally).
/// Garden/Grove are GovernanceConfig signed-Node CONTENT, not crypto-wire
/// sub-codepoints. Would-FAIL if a `GARDEN_*`/`GROVE_*` codepoint const
/// appears in the crypto band.
#[test]
fn f_freeze_1_no_garden_grove_sub_codepoints() {
    let garden = rust_files_containing("GARDEN_CODEPOINT");
    let grove = rust_files_containing("GROVE_CODEPOINT");
    assert!(
        garden.is_empty() && grove.is_empty(),
        "Garden/Grove MUST be GovernanceConfig signed-Node CONTENT, not \
         sub-codepoints (−2 in the O-8 −8 tally). Garden codepoint: {:?}; \
         Grove codepoint: {:?}.",
        garden,
        grove
    );
}

/// PIN 5 — compute-marketplace wire types ABSENT from the frozen wire:
/// `PeerResource`, `ResourceKind`, `OwnerRef`, `CommunityEconomicPolicy`
/// are D-28/D-29 PHASE-LATER-DEFER. Would-FAIL if any appears as a frozen
/// wire struct at v1-beta.
#[test]
fn f_freeze_1_compute_marketplace_wire_types_absent() {
    let forbidden = [
        "PeerResource",
        "ResourceKind",
        "OwnerRef",
        "CommunityEconomicPolicy",
    ];
    let mut leaked: Vec<(&str, Vec<String>)> = Vec::new();
    for tok in forbidden {
        let hits = rust_files_containing(tok);
        if !hits.is_empty() {
            leaked.push((tok, hits));
        }
    }
    assert!(
        leaked.is_empty(),
        "compute-marketplace wire types MUST be ABSENT from the frozen \
         wire at v1-beta (D-28/D-29 PHASE-LATER-DEFER). Leaked: {:?}.",
        leaked
    );
}

/// PIN 6 — V1-FROZEN-INTERFACE-DEFERRED.md REGISTERS D-28 + D-29
/// (the compute-marketplace + economic deferrals). In-tree max at baseline
/// is D-27 — R5's doc-wave adds D-28/D-29. Would-FAIL if the deferrals are
/// claimed-but-undocumented.
#[test]
fn f_freeze_1_v1_frozen_doc_registers_d28_d29() {
    let doc = v1_frozen_doc();
    assert!(
        doc.contains("D-28"),
        "V1-FROZEN-INTERFACE-DEFERRED.md MUST register row D-28 \
         (compute-marketplace PHASE-LATER-DEFER). R5 doc-wave adds it."
    );
    assert!(
        doc.contains("D-29"),
        "V1-FROZEN-INTERFACE-DEFERRED.md MUST register row D-29 \
         (economic-policy PHASE-LATER-DEFER). R5 doc-wave adds it."
    );
}

/// PIN 7 — net-frozen-surface tally is documented as −8 (NOT R0.1's
/// double-counted −6). Doc-coupling: the freeze-surface accounting MUST
/// name the −8 net + the O-8 double-count correction. Would-FAIL if the
/// doc still claims −6.
#[test]
fn f_freeze_1_net_frozen_surface_tally_documented_minus_eight() {
    // The tally is recorded in the freeze/codepoint doc-wave artifact.
    // R5 lands `docs/CRYPTO-CODEPOINTS.md` (per F-DISC-2) which carries
    // the accounting; we accept either that doc or V1-FROZEN.
    let codepoints =
        std::fs::read_to_string(repo_root().join("docs/CRYPTO-CODEPOINTS.md")).unwrap_or_default();
    let v1frozen = v1_frozen_doc();
    let combined = format!("{codepoints}\n{v1frozen}");
    assert!(
        combined.contains("-8") || combined.contains("−8") || combined.contains("net -8"),
        "the freeze-surface accounting MUST document the −8 NET shrink \
         (O-8 corrects R0.1's −6 double-count). Neither CRYPTO-CODEPOINTS.md \
         nor V1-FROZEN-INTERFACE-DEFERRED.md names it."
    );
    // Over-claim guard: the corrected accounting must NOT still assert −6.
    assert!(
        !combined.contains("net -6") && !combined.contains("net −6"),
        "the corrected tally MUST NOT still claim the R0.1 double-counted \
         −6 net (O-8 correction)."
    );
}
