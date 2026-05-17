//! COLLAPSE (P4) — #1230 v1-BLOCKER closure-pin at the benten-id
//! root site (pim-2 §3.6b, would-FAIL-if-the-pipe-reappears).
//!
//! # Charter
//!
//! Spec: `.addl/refinement-audit-2026-05/impl-design-COLLAPSE.md` §1.2
//! (benten-id::ucan deletion set) + §5 + issue #1230 ("Grep evidence
//! (enforced nowhere)") + `DECISION-RECORD-trust-model-reframe.md` §4
//! (RATIFIED — COLLAPSE-WITH-RESIDUAL).
//!
//! # The #1230 root site
//!
//! Issue #1230 names this crate's `validate_chain_with_device_revocations`
//! as **the #1230 BLOCKER root site**: its signature was
//! `(chain, revocations)` keyed on a *bare device-DID* with **no
//! parent context** — "structurally incapable of the binding". An
//! attacker controlling *any* `parent_did` could sign a structurally
//! valid revocation against a victim's device-DID → perpetual
//! victim-DoS. The RATIFIED resolution (DECISION-RECORD §4) is
//! COLLAPSE: the device-revocation pipe is **deleted**, not patched
//! with parent-binding. P1 (merged in #1238) deleted
//! `validate_chain_with_device_revocations`, `DeviceRevocation`,
//! `RevocationReason`, `revocation_canonical_bytes`, `Acceptor`
//! (`accept_at` / `with_parent_lookup` / `new_with_revocations`).
//!
//! # What this pins (would-FAIL-if-no-op'd)
//!
//! The #1230 perpetual-victim-DoS is dissolved iff there is **no
//! benten-id production API that accepts a bare device-DID-keyed
//! revocation** — the forge surface must not merely be defended, it
//! must be *absent*. This pin grep-asserts the deleted root-site
//! symbols do not reappear in benten-id production source (`src/`),
//! excluding the COLLAPSE deletion-marker `//` comments that
//! intentionally name them for the SUPERSEDED-BY-COLLAPSE narrative.
//! If a future change re-introduces the `(chain, revocations)`
//! device-DID-keyed walker (or `DeviceRevocation` / `Acceptor` parent
//! gate) the #1230 forge surface returns and this test FAILs — the
//! pim-2 §3.6b would-FAIL-if-no-op'd contract for a deletion-shaped
//! closure (the absence IS the fix; mirrors the established
//! `cap_r1_1_audience_binding_grep_defense.rs` source-grep idiom).
//!
//! Pairs with the behavioral benten-caps pin
//! `collapse_p4_1230_605_707_single_revocation_seam.rs` (the single
//! self-anchored content-CID-keyed revocation survivor) — defense in
//! depth: behavioral (the one seam works) + structural (the deleted
//! parallel pipe stays deleted).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};

/// The deleted #1230-root-site symbols. Re-introduction of ANY in
/// benten-id production source recreates the perpetual-victim-DoS
/// forge surface.
const DELETED_ROOT_SITE_SYMBOLS: &[&str] = &[
    // The #1230 BLOCKER root site itself — the (chain, revocations)
    // device-DID-keyed walker with no parent context.
    "validate_chain_with_device_revocations",
    // The device-revocation envelope + its forge-able reason enum.
    "struct DeviceRevocation",
    "enum RevocationReason",
    "fn revocation_canonical_bytes",
    // The acceptance pipeline whose only production construction had
    // "no expected-parent gate" (issue #1230 grep evidence) — the
    // parent-binding-incapable acceptor.
    "struct Acceptor",
    "fn accept_at",
    "fn with_parent_lookup",
    "fn new_with_revocations",
];

#[test]
fn collapse_p4_no_bare_device_did_revocation_pipe_in_benten_id_production_source() {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    assert!(
        src_dir.is_dir(),
        "benten-id src dir not found at {}",
        src_dir.display()
    );

    let mut offenders: Vec<String> = Vec::new();
    visit_rs(&src_dir, &mut |path, contents| {
        for (lineno, line) in contents.lines().enumerate() {
            // Skip comment lines: the COLLAPSE deletion narrative in
            // `device_attestation.rs` legitimately NAMES the deleted
            // symbols inside `//` lines (deletion markers cross-ref'd
            // from docs/SECURITY-POSTURE.md Compromise #23). Those are
            // documentation of the deletion, not the pipe itself.
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            for sym in DELETED_ROOT_SITE_SYMBOLS {
                if line.contains(sym) {
                    offenders.push(format!(
                        "{}:{} reintroduces #1230-root-site symbol `{sym}`: {}",
                        path.display(),
                        lineno + 1,
                        line.trim()
                    ));
                }
            }
        }
    });

    assert!(
        offenders.is_empty(),
        "COLLAPSE #1230 REGRESSION (v1-BLOCKER): the deleted bare-device-DID \
         revocation pipe reappeared in benten-id production source. #1230's \
         perpetual-victim-DoS is dissolved ONLY by this pipe's ABSENCE — an \
         un-anchored (chain, revocations) device-DID-keyed walker, or a \
         parent-binding-incapable Acceptor, recreates the forge surface a \
         hostile parent_did exploits for perpetual victim-DoS (pim-2 §3.6b; \
         DECISION-RECORD §4 RATIFIED COLLAPSE-not-patch-with-D). \
         Offenders:\n{}",
        offenders.join("\n")
    );
}

/// Behavioral corollary: post-COLLAPSE-P2 (CONSOLIDATE) the
/// policy-bearing authority walkers MOVED out of benten-id to
/// `benten_caps::chain_authority` (`validate_chain_with_rotation_log`
/// + `validate_chain_with_envelope_ceiling`, was
/// `validate_chain_with_attestations`); benten-id keeps only the pure
/// crypto/structural `validate_chain_*` primitives. NONE of the moved
/// or surviving walkers take a `revocations` argument — there is no
/// longer ANY chain-walker (in benten-id OR benten-caps) that consults
/// a bare device-DID-keyed revocation list. This grep pins the
/// benten-id half: no `&[DeviceRevocation]`-shaped parameter may
/// return to benten-id/src/ucan.rs. This is the type-level proof the
/// #1230 forge is structurally unconstructible: you cannot call an API
/// that does not exist with a shape that was deleted.
///
/// If a `revocations: &[DeviceRevocation]`-shaped parameter were
/// re-added to a benten-id chain-walker, this grep FAILs (the
/// signature shape returns).
#[test]
fn collapse_p4_no_benten_id_chain_walker_takes_a_device_revocation_list() {
    let ucan_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ucan.rs");
    let body = std::fs::read_to_string(&ucan_src)
        .unwrap_or_else(|e| panic!("read {}: {e}", ucan_src.display()));

    // The deleted walker took `revocations: &[DeviceRevocation]`. No
    // surviving walker may take that shape. (Comment lines naming the
    // deletion are excluded as above.)
    for (lineno, line) in body.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        assert!(
            !line.contains("&[DeviceRevocation]")
                && !line.contains("&[crate::device_attestation::DeviceRevocation]"),
            "COLLAPSE #1230 REGRESSION: benten-id/src/ucan.rs:{} re-introduces a \
             chain-walker taking a device-revocation list — the #1230 \
             bare-device-DID forge surface. Revocation MUST flow only \
             through the single self-anchored content-CID-keyed seam \
             (benten-caps UCANBackend::revoke). Offending line: {}",
            lineno + 1,
            line.trim()
        );
    }
}

fn visit_rs(dir: &Path, f: &mut impl FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs(&path, f);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs")
            && let Ok(contents) = std::fs::read_to_string(&path)
        {
            f(&path, &contents);
        }
    }
}
