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

// The band is READ from the registry, never re-typed here: a scanner that
// hard-codes the range it scans can drift out from under the thing it guards.
use benten_crypto_suite::registry::BENTEN_ENVELOPE_RANGE;

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

fn crypto_codepoints_doc() -> String {
    std::fs::read_to_string(repo_root().join("docs/CRYPTO-CODEPOINTS.md"))
        .expect("CRYPTO-CODEPOINTS.md must be present — it carries the O-8 tally")
}

// ===========================================================================
// STRUCTURAL CODEPOINT SCANNER (FS-06 / FS-07 closure).
//
// PINs 2 and 4 below used to grep for four hand-guessed identifier spellings:
// `MEMBERSHIP_EVENT_CODEPOINT`, `MembershipEventCodepoint`, `GARDEN_CODEPOINT`,
// `GROVE_CODEPOINT`. A spelling list is not a defense. The R6 round-#1
// falsification sweep (2026-07-27) reintroduced
// `#[repr(u16)] enum MembershipEvent { Joined = 0x6611 }` PLUS
// `const MEMBERSHIP_EVENT_CP: u16 = 0x6611;`, and minted
// `GARDEN_SUB_CP: u16 = 0x66A0` — and both pins stayed GREEN, because none of
// those is one of the four guessed strings.
//
// These scanners key off the CLASS instead: a declaration binding a `u16`
// literal inside `BENTEN_ENVELOPE_RANGE` (`0x6100..=0x6FFF`, the real const
// from `benten_crypto_suite::registry`, not a re-typed number), or a
// `#[repr(..)]`-tagged enum. Identifier spelling only decides which DOMAIN a
// hit is attributed to, never whether it is seen at all — and attribution
// normalises case and `_`, so `MEMBERSHIP_EVENT_CP`, `MembershipEventCodepoint`
// and `MembershipEventCp` are the same token to it.
//
// The line-scan idiom, `is_comment_line` and `parse_u16_literal` mirror
// `benten-crypto-suite/src/registry.rs::scan_u16_const_declarations`
// deliberately: same shape, wider input.
//
// WHAT THIS DOES NOT PROVE. It is a source scan, not a wire test. It cannot
// see a codepoint assembled at runtime (`BASE + 0x11`), one written as a
// decimal outside the literal forms parsed here, or one introduced in a file
// this wave's self-exclusion skips. It is a regression floor on the shape the
// sweep actually used, not a proof of absence.
// ===========================================================================

/// Whether a source line is a comment / doc line (after trimming). Mirrors
/// `registry.rs::is_comment_line`: prose ABOUT a declaration is not a
/// declaration, so the worked examples in the comment block above cannot
/// smuggle themselves into the scanned set.
fn is_comment_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with("/*") || t.starts_with('*')
}

/// Parse a `u16` initializer: hex `0x66A0` (optionally `u16`-suffixed) or a
/// decimal literal; `_` separators tolerated. `None` for anything computed.
fn parse_u16_literal(raw: &str) -> Option<u16> {
    let cleaned = raw
        .trim()
        .trim_end_matches(';')
        .trim()
        .trim_end_matches("u16")
        .replace('_', "");
    match cleaned
        .strip_prefix("0x")
        .or_else(|| cleaned.strip_prefix("0X"))
    {
        Some(digits) => u16::from_str_radix(digits, 16).ok(),
        None => cleaned.parse::<u16>().ok(),
    }
}

/// Upper-case and drop `_` so every spelling of a domain token compares equal.
fn normalized_ident(name: &str) -> String {
    name.chars()
        .filter(|c| *c != '_')
        .flat_map(char::to_uppercase)
        .collect()
}

/// Strip a leading `pub` / `pub(crate)` / `pub(super)` / `pub(in ...)`.
fn strip_visibility(line: &str) -> &str {
    let t = line.trim();
    if let Some(rest) = t.strip_prefix("pub(") {
        return rest
            .split_once(')')
            .map_or(t, |(_, after)| after.trim_start());
    }
    t.strip_prefix("pub ").unwrap_or(t)
}

/// Every `[vis] const|static <NAME>: u16 = <literal>;` in `content` whose
/// value lands inside the Benten envelope band.
fn in_band_u16_bindings(content: &str) -> Vec<(String, u16)> {
    let mut out = Vec::new();
    for line in content.lines() {
        if is_comment_line(line) {
            continue;
        }
        let t = strip_visibility(line);
        let Some(rest) = t
            .strip_prefix("const ")
            .or_else(|| t.strip_prefix("static "))
        else {
            continue;
        };
        let Some((name, tail)) = rest.split_once(':') else {
            continue;
        };
        let Some(value_src) = tail.trim_start().strip_prefix("u16 = ") else {
            continue;
        };
        let Some(value) = parse_u16_literal(value_src) else {
            continue;
        };
        if BENTEN_ENVELOPE_RANGE.contains(&value) {
            out.push((name.trim().to_string(), value));
        }
    }
    out
}

/// Names of enums carrying a `#[repr(..)]` attribute. A repr-tagged enum is a
/// wire enum whether or not a codepoint const accompanies it — which is the
/// half of the FS-06 mutation a const-only scan misses.
fn repr_tagged_enums(content: &str) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if is_comment_line(line) || !line.trim_start().starts_with("#[repr(") {
            continue;
        }
        // Attributes and derives may sit between `#[repr(..)]` and the item.
        for probe in lines.iter().skip(i + 1).take(8) {
            if is_comment_line(probe) {
                continue;
            }
            let t = strip_visibility(probe);
            if let Some(rest) = t.strip_prefix("enum ") {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    out.push(name);
                }
                break;
            }
            if t.starts_with('#') || t.is_empty() {
                continue;
            }
            break;
        }
    }
    out
}

/// Every in-band `u16` binding, and every `#[repr(..)]` enum, anywhere under
/// `crates/`, whose normalised identifier contains one of `domains`.
///
/// Self-exclusion (`is_grep_defense_test_file`) still applies: this file names
/// the domain tokens as search literals.
fn in_band_declarations_naming(domains: &[&str]) -> Vec<String> {
    let mut hits = Vec::new();
    for path in all_rust_sources() {
        if is_grep_defense_test_file(&path) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (name, codepoint) in in_band_u16_bindings(&content) {
            if domains.iter().any(|d| normalized_ident(&name).contains(d)) {
                hits.push(format!("{}: `{name}` = {codepoint:#06x}", path.display()));
            }
        }
        for enum_name in repr_tagged_enums(&content) {
            if domains
                .iter()
                .any(|d| normalized_ident(&enum_name).contains(d))
            {
                hits.push(format!("{}: #[repr] enum `{enum_name}`", path.display()));
            }
        }
    }
    hits
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

/// PIN 2 — `MembershipEvent` is NOT a frozen wire-enum: it is version-Node
/// content (the −1 wire-enum win in the −8 tally).
///
/// MUTATION THAT MUST MAKE THIS FAIL (this is the literal mutation the R6
/// round-#1 sweep ran against the previous version of this pin, which stayed
/// GREEN): add to any file under `crates/`
///
/// ```ignore
/// #[repr(u16)]
/// pub enum MembershipEvent { Joined = 0x6611 }
/// pub const MEMBERSHIP_EVENT_CP: u16 = 0x6611;
/// ```
///
/// Either declaration alone trips it now: the const because its value is in
/// `BENTEN_ENVELOPE_RANGE` and its normalised name contains the domain token,
/// the enum because it is `#[repr]`-tagged. Verified against a planted tree,
/// 2026-07-27; verified NOT to fire on the real tree at `df0c8287`, and NOT
/// to fire on the neighbouring in-band const
/// `MEMBERSHIP_SET_GROUP_MULTI_STANZA = 0x6610`.
#[test]
fn f_freeze_1_membership_event_not_a_frozen_wire_enum() {
    let regressions = in_band_declarations_naming(&["MEMBERSHIPEVENT"]);
    assert!(
        regressions.is_empty(),
        "`MembershipEvent` MUST be version-Node CONTENT, not a frozen \
         wire-enum. Found codepoint-band declaration(s) / repr-tagged enum(s) \
         naming it, which regresses the −1 wire-enum win (O-8 net −8 tally): \
         {regressions:?}",
    );
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

/// PIN 4 — NO Garden/Grove SUB-codepoints (−2 in the −8 tally). Garden/Grove
/// are GovernanceConfig signed-Node CONTENT, not crypto-wire sub-codepoints.
///
/// MUTATION THAT MUST MAKE THIS FAIL (the sweep's actual mutation, which the
/// previous `GARDEN_CODEPOINT` / `GROVE_CODEPOINT` string grep did not catch):
/// add `pub(crate) const GARDEN_SUB_CP: u16 = 0x66A0;` to any file under
/// `crates/`. Any in-band `u16` binding whose normalised name contains
/// `GARDEN` or `GROVE` fires, whatever the suffix. Verified against a planted
/// tree, 2026-07-27; verified NOT to fire on the real tree at `df0c8287`.
#[test]
fn f_freeze_1_no_garden_grove_sub_codepoints() {
    let regressions = in_band_declarations_naming(&["GARDEN", "GROVE"]);
    assert!(
        regressions.is_empty(),
        "Garden/Grove MUST be GovernanceConfig signed-Node CONTENT, not \
         sub-codepoints (−2 in the O-8 −8 tally). Found in-band declaration(s): \
         {regressions:?}",
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

/// PIN 7 — the net-frozen-surface tally is documented as −8 (NOT R0.1's
/// double-counted −6), in the doc that declares itself the canonical one, with
/// components that actually sum to it.
///
/// The previous predicate was `combined.contains("-8")` over
/// CRYPTO-CODEPOINTS.md concatenated with V1-FROZEN-INTERFACE-DEFERRED.md.
/// Deleting the entire §4.4 heading and its bullets from CRYPTO-CODEPOINTS.md
/// left it PASSING, because the unrelated string `"Row D-8"` in the sibling
/// document satisfies `contains("-8")` on its own (confirmed: that substring
/// is present in the sibling at `df0c8287`).
///
/// MUTATIONS THAT MUST MAKE THIS FAIL — all three checked by hand against the
/// real doc, 2026-07-27:
///   1. delete the `## §4.4` heading (or the `net -8` sentence) from
///      `docs/CRYPTO-CODEPOINTS.md` — the sibling doc can no longer stand in,
///      because the canonical text is required in the canonical file;
///   2. edit any one component (e.g. `**−4** AuditAccessGradation` → `**−3**`)
///      without editing the total — the components then sum to 7, not 8;
///   3. restore the superseded `net -6` — the over-claim guard fires.
#[test]
fn f_freeze_1_net_frozen_surface_tally_documented_minus_eight() {
    const TALLY_HEADING: &str =
        "## §4.4 — Net frozen-surface summary (O-8 — single canonical tally)";
    const TALLY_TOTAL: &str = "The single canonical tally is **`net -8`**:";

    // The canonical tally lives in the canonical doc. Scoping this to
    // CRYPTO-CODEPOINTS.md is the point: the concatenation is what let an
    // unrelated `-8` in a sibling file keep this green.
    let codepoints = crypto_codepoints_doc();
    assert!(
        codepoints.contains(TALLY_HEADING),
        "docs/CRYPTO-CODEPOINTS.md MUST carry the canonical O-8 tally heading \
         verbatim: {TALLY_HEADING}",
    );
    assert!(
        codepoints.contains(TALLY_TOTAL),
        "docs/CRYPTO-CODEPOINTS.md §4.4 MUST state the net total verbatim: \
         {TALLY_TOTAL}",
    );

    // The components must add up to the total. A tally whose parts and whose
    // sum can drift apart is a number, not an accounting.
    let section = codepoints
        .split_once(TALLY_HEADING)
        .expect("heading asserted present above")
        .1;
    let section = section
        .split_once("\n## ")
        .map_or(section, |(head, _)| head);
    let magnitudes: Vec<u32> = section
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- **\u{2212}"))
        .filter_map(|rest| {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            digits.parse::<u32>().ok()
        })
        .collect();
    assert_eq!(
        magnitudes.len(),
        4,
        "§4.4 MUST itemise exactly the 4 GN-win components (−1 audit structure, \
         −1 MembershipEvent wire-enum, −4 AuditAccessGradation, −2 Garden/Grove). \
         Parsed: {magnitudes:?}",
    );
    assert_eq!(
        magnitudes.iter().sum::<u32>(),
        8,
        "the §4.4 components MUST sum to the stated net −8. Parsed: {magnitudes:?}",
    );

    // Over-claim guard: the corrected accounting must NOT still assert −6.
    // Kept broad (both docs) — a negative assertion is safe to widen.
    let combined = format!("{codepoints}\n{}", v1_frozen_doc());
    assert!(
        !combined.contains("net -6") && !combined.contains("net −6"),
        "the corrected tally MUST NOT still claim the R0.1 double-counted \
         −6 net (O-8 correction).",
    );
}
