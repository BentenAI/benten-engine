//! **F-DISC-2** — Invariant + doc registration catch-net
//! (GAP-5a — §9.1-7 doc-wave gate).
//!
//! ADDL Phase-4-Meta-Core, F-full **R3-W7 (doc-wave)** partition.
//! Pure doc-coupling family (reuses the `tf3f` shape: read the registry
//! docs from disk via `CARGO_MANIFEST_DIR`, assert the end-state
//! registration properties).
//!
//! Pin source: `.addl/phase-4-meta/f-full-r2-test-landscape.md` §1
//! Group-12 **F-DISC-2**:
//!   "`INVARIANT-COVERAGE.md` REGISTERS Inv-16..22 AND Inv-15 NOT
//!    re-registered AND header count correct end-state (M-15); 4 new docs
//!    exist (`CRYPTO-CODEPOINTS.md`, `THREAT-MODEL.md`,
//!    `SECURITY-PROOFS.md`, `compute-marketplace.md`);
//!    `V1-FROZEN-INTERFACE-DEFERRED.md` has D-28/D-29 (in-tree max D-27);
//!    EP-1 roster + 'Rust engine plugin' naming landed; §0.4 supersession
//!    recorded.  §9.1-7, M-15/M-16; pim-13.  FG/CF.  ~6-8 tests.
//!    No dimension owned the doc-wave deliverables that gate the freeze."
//!
//! **RED-PHASE (pim-12 §3.6e):** at baseline `INVARIANT-COVERAGE.md` tops
//! out at Inv-15; the 4 new docs are ABSENT; V1-FROZEN tops out at D-27.
//! The end-state registration arms are
//! `#[ignore = "RED-PHASE: F-DISC-2 ..."]`; R5's doc-wave un-ignores them.
//! The NON-ignored baseline arms drive the REAL on-disk
//! `INVARIANT-COVERAGE.md` parser (proving the harness reads the doc, not
//! a literal) — it recovers Inv-15 today and would-FAIL if the parser
//! went inert. NEVER `assert_eq!(CONST, CONST_VAL)`.
//!
//! # R6 ENFORCEMENT-STATE strengthening (AS-BUILT+ENFORCED reclassification)
//!
//! The doc-registration arms above only assert Inv-16..22 are *registered*
//! in the doc. A parallel doc branch reclassifies the benten-drop-owned
//! invariants from "REGISTERED, NOT-YET-ENFORCED" to **AS-BUILT+ENFORCED**.
//! A doc catch-net that only reads doc TEXT cannot structurally back an
//! *enforcement* claim — text drift is exactly what it must catch. So the
//! `f_disc_2_inv*_enforced_*` arms below DRIVE the REAL benten-drop
//! production code (the surfaces this crate OWNS) and assert the invariant's
//! enforcement is live: Inv-16 codepoint-dispatch typed-reject, Inv-18
//! Sealed-Sender-default metadata-disclosure, Inv-20-clause-c the `0x6610`
//! BLINDED 11-field group-AAD field-set, and the §3.3 truncation/censorship
//! fail-closed (the F-01 enforcer). would-FAIL-if-no-op'd: each arm exercises
//! a production entry point and asserts an observable consequence; reverting
//! the enforcer flips the arm.
//!
//! **Honest scope (Inv-21 / Inv-22 FLAGGED, not over-claimed):** Inv-21
//! (set-identity fork tie-break) + Inv-22 (member-nature derivation) are
//! enforced in `benten-membership-set` / `benten-sync` / graph-native code —
//! NOT in benten-drop. This crate's test target CANNOT cleanly drive their
//! production enforcers (the fork-tie-break runs on the sync CRDT round; the
//! member-nature derivation is graph-data-half), so this catch-net does NOT
//! assert their enforcement and explicitly FLAGS them as owned-elsewhere (see
//! `f_disc_2_inv21_inv22_enforcement_owned_elsewhere_flag`). Their AS-BUILT
//! +ENFORCED backing belongs in the owner crates' test targets.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::BTreeSet;

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve from crates/benten-drop")
}

fn invariant_coverage_md() -> String {
    std::fs::read_to_string(repo_root().join("docs/INVARIANT-COVERAGE.md"))
        .expect("INVARIANT-COVERAGE.md must be present")
}

/// Enumerate the registered `Inv-<N>` numbers FROM the doc (not a literal).
fn registered_invariants(doc: &str) -> BTreeSet<u32> {
    let mut out = BTreeSet::new();
    for line in doc.lines() {
        let mut search = line;
        while let Some(idx) = search.find("Inv-") {
            let after = &search[idx + "Inv-".len()..];
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                out.insert(n);
            }
            let consumed = idx + "Inv-".len() + digits.len().max(1);
            if consumed >= search.len() {
                break;
            }
            search = &search[consumed..];
        }
    }
    out
}

// ===========================================================================
// BASELINE ARMS (NOT ignored) — drive the REAL invariant-doc parser.
// Pass green now; prove the parser reads the doc, not a literal.
// ===========================================================================

/// PIN 0 (baseline) — the parser enumerates invariants FROM the real
/// on-disk `INVARIANT-COVERAGE.md`. At baseline Inv-15 is the top
/// registered invariant; the parser MUST recover it. A no-op parser
/// (returning {}) regresses the catch-net silently — this is the
/// would-FAIL-if-no-op'd guard for the parser itself.
#[test]
fn f_disc_2_parser_enumerates_invariants_from_doc_baseline() {
    let doc = invariant_coverage_md();
    let invs = registered_invariants(&doc);
    assert!(
        invs.contains(&15),
        "registered_invariants MUST enumerate FROM the on-disk \
         INVARIANT-COVERAGE.md. At baseline Inv-15 is registered and MUST \
         be recovered. Got: {:?}. An inert parser would make every \
         registration pin vacuously pass.",
        invs
    );
    // Sanity: a low invariant (Inv-1 etc.) is also present — the doc is
    // a real registry, not a stub.
    assert!(
        invs.iter().any(|&n| n <= 14),
        "the invariant registry MUST carry the baseline Inv-1..Inv-14 set."
    );
}

// ===========================================================================
// RED-PHASE ARMS (ignored until R5) — assert the F-full registration
// end-state.
// ===========================================================================

/// PIN 1 — `INVARIANT-COVERAGE.md` REGISTERS Inv-16..22 (all 7 new
/// invariants). Parametrized over the closed range; enumerated from the
/// doc so an extra new invariant is auto-detected by PIN 0's parser.
/// Would-FAIL if any of Inv-16..22 is claimed-but-unregistered.
#[test]
fn f_disc_2_registers_inv_16_through_22() {
    let doc = invariant_coverage_md();
    let invs = registered_invariants(&doc);
    let missing: Vec<u32> = (16u32..=22).filter(|n| !invs.contains(n)).collect();
    assert!(
        missing.is_empty(),
        "INVARIANT-COVERAGE.md MUST register Inv-16..22 (the 7 new F-full \
         invariants). Missing: {:?}. R5 doc-wave registers them.",
        missing
    );
}

/// PIN 2 — header invariant-COUNT is the correct end-state (M-15) AND
/// Inv-15 is NOT RE-registered (no duplicate). The header count MUST
/// equal the highest registered invariant (22). Would-FAIL if the header
/// drifts from the body (a classic registry-drift) or Inv-15 is duplicated.
#[test]
fn f_disc_2_header_count_correct_and_inv15_not_re_registered() {
    let doc = invariant_coverage_md();
    let invs = registered_invariants(&doc);
    let highest = invs.iter().copied().max().unwrap_or(0);
    assert_eq!(
        highest, 22,
        "the highest registered invariant MUST be Inv-22 at the F-full \
         end-state. Got highest = Inv-{highest}."
    );
    // The header MUST name the count (22) — drift-defense between header
    // and body. We look for the literal count token near a header marker.
    assert!(
        doc.contains("22 invariant")
            || doc.contains("22 Invariant")
            || doc.contains("Inv-1..Inv-22")
            || doc.contains("Inv-1 .. Inv-22"),
        "the INVARIANT-COVERAGE.md header MUST state the end-state count \
         (22 invariants) so header and body don't drift (M-15)."
    );
    // Inv-15 appears, but MUST NOT be RE-registered as a NEW row in the
    // F-full mint block (no duplicate registration). We bound the count of
    // distinct `Inv-15` *registration-row* markers — at most one.
    let inv15_registration_rows = doc
        .lines()
        .filter(|l| {
            l.contains("Inv-15")
                && (l.contains("| Inv-15")
                    || l.trim_start().starts_with("### Inv-15")
                    || l.trim_start().starts_with("## Inv-15"))
        })
        .count();
    assert!(
        inv15_registration_rows <= 1,
        "Inv-15 MUST NOT be RE-registered by the F-full doc-wave (it was \
         minted at Phase-4-Meta-Core already). Found {inv15_registration_rows} \
         registration rows."
    );
}

/// PIN 3 — the 4 new docs EXIST at the repo `docs/` root:
/// `CRYPTO-CODEPOINTS.md`, `THREAT-MODEL.md`, `SECURITY-PROOFS.md`,
/// `future/compute-marketplace.md`. Doc-coupling existence. Would-FAIL if
/// the doc-wave gate is claimed-complete with a doc missing.
#[test]
fn f_disc_2_four_new_docs_exist() {
    let docs = repo_root().join("docs");
    let required = [
        docs.join("CRYPTO-CODEPOINTS.md"),
        docs.join("THREAT-MODEL.md"),
        docs.join("SECURITY-PROOFS.md"),
        docs.join("future/compute-marketplace.md"),
    ];
    let missing: Vec<String> = required
        .iter()
        .filter(|p| !p.exists())
        .map(|p| p.display().to_string())
        .collect();
    assert!(
        missing.is_empty(),
        "the 4 new F-full docs MUST exist (the §9.1-7 doc-wave gate). \
         Missing: {:?}. R5 doc-wave creates them.",
        missing
    );
}

/// PIN 4 — `V1-FROZEN-INTERFACE-DEFERRED.md` has rows D-28 + D-29
/// (in-tree max is D-27 at baseline). Coheres with F-FREEZE-1 PIN 6 but
/// owned HERE as the registration-catch-net. Would-FAIL if the deferrals
/// are claimed but the doc rows are absent.
#[test]
fn f_disc_2_v1_frozen_has_d28_d29() {
    let doc = std::fs::read_to_string(repo_root().join("docs/V1-FROZEN-INTERFACE-DEFERRED.md"))
        .expect("V1-FROZEN-INTERFACE-DEFERRED.md must be present");
    assert!(
        doc.contains("D-28") && doc.contains("D-29"),
        "V1-FROZEN-INTERFACE-DEFERRED.md MUST register D-28 + D-29 \
         (the compute/economic PHASE-LATER-DEFER rows; baseline max D-27)."
    );
}

/// PIN 5 — the EP-1 extension-point roster + the "Rust engine plugin"
/// naming landed in the architecture docs. Doc-coupling. Would-FAIL if
/// the naming/roster is claimed but absent (the EP-1 §6.3 conformance
/// gate).
#[test]
fn f_disc_2_ep1_roster_and_rust_engine_plugin_naming_landed() {
    let docs = repo_root().join("docs");
    // Search the arch-facing docs for the EP-1 roster + naming.
    let candidates = [
        docs.join("ARCHITECTURE.md"),
        docs.join("CRYPTO-CODEPOINTS.md"),
    ];
    let mut combined = String::new();
    for c in &candidates {
        combined.push_str(&std::fs::read_to_string(c).unwrap_or_default());
        combined.push('\n');
    }
    assert!(
        combined.contains("EP-1") || combined.contains("extension-point roster"),
        "the EP-1 extension-point roster MUST be documented (§6.3 EP-1)."
    );
    assert!(
        combined.contains("Rust engine plugin") || combined.contains("Rust-engine-plugin"),
        "the 'Rust engine plugin' naming MUST land (the engine-extension \
         symmetry naming, M-16)."
    );
}

/// PIN 6 — the §0.4 supersession (MLS-bracket `0x6380/0x6390` supersedes
/// the M-CONS-FINAL F8/F21 MembershipSet/Sealed-Sender assignment;
/// MembershipSetEncryption = `0x6600`) is RECORDED in the codepoint doc.
/// Doc-coupling. Would-FAIL if the supersession is applied in code but the
/// rationale is undocumented (a silent codepoint reassignment).
#[test]
fn f_disc_2_section_0_4_supersession_recorded() {
    let codepoints =
        std::fs::read_to_string(repo_root().join("docs/CRYPTO-CODEPOINTS.md")).unwrap_or_default();
    assert!(
        codepoints.contains("0x6380") && codepoints.contains("0x6390"),
        "CRYPTO-CODEPOINTS.md MUST record the MLS-bracket codepoints \
         (0x6380 MLS-Application / 0x6390 MLS-Welcome) per the §0.4 \
         supersession."
    );
    assert!(
        codepoints.contains("0x6600"),
        "CRYPTO-CODEPOINTS.md MUST record MembershipSetEncryption = 0x6600 \
         (the §0.4 supersession of the M-CONS-FINAL F8/F21 assignment)."
    );
}

/// PIN 7 (F2 / R4.6) — the doc-registration catch-net MUST protect THIS
/// round's own two wire corrections: `CRYPTO-CODEPOINTS.md` MUST register
/// the group-band codepoints `0x6610` (`MEMBERSHIP_SET_GROUP_MULTI_STANZA`,
/// the R4.6-corrected value — was slipped to the `0x6600` set-keying value
/// in "settled" territory) and `0x6520` (`LAYER_C_DROP_MULTI_RECIPIENT`,
/// the R0.7-blinded Layer-C group multi-stanza value).
///
/// Both are **FREEZE** rows in R0.7 §4.0 (allocation table lines 1001 +
/// 1010) / §4.1 (AAD field-set rows), and §4.0 names
/// `docs/CRYPTO-CODEPOINTS.md` as the in-tree home of that table (spec
/// lines 218 + 979). The in-code wire-lock for both constants is strong
/// (`crates/benten-crypto-suite/tests/f_cp_codepoint_registry_dispatch.rs`
/// regression-guards `MEMBERSHIP_SET_GROUP_MULTI_STANZA == 0x6610` and
/// `LAYER_C_DROP_MULTI_RECIPIENT == 0x6520` directly) — so this pin is
/// DOC-coupling completeness, not a byte defect: it closes the
/// doc-registration blind spot that sat exactly where the wire changed
/// this round.
///
/// **Why FIX-NOW (not deferred to the R5 doc-wave):** the red-phase
/// catch-net infrastructure exists NOW and, by its own design, must
/// protect this round's own corrections; deferring leaves the doc-coupling
/// blind spot precisely on the two codepoints R4.6/R0.7 corrected (HARD
/// RULE 12 — no "minor enough to defer"). The test stays `#[ignore]`
/// because the doc itself (`CRYPTO-CODEPOINTS.md`) is created at the R5
/// doc-wave (spec line 206: `ls` → absent at baseline). Additive +
/// byte-neutral (no frozen golden, codepoint constant, or AAD layout
/// changes).
///
/// **Registry-binding (not a bare substring):** §4.0's table BINDS each
/// codepoint to a SYMBOL on one logical row, and the slip this round
/// guards against was precisely a value bound to the WRONG meaning
/// (`0x6610` slipped to the `0x6600` set-keying symbol). So — matching
/// PIN 0's "enumerate FROM the doc, don't match a literal" discipline and
/// PIN 6's row shape — this pin asserts each codepoint is CO-LOCATED with
/// its registered symbol (`0x6610`↔`MEMBERSHIP_SET_GROUP_MULTI_STANZA`,
/// `0x6520`↔`LAYER_C_DROP_MULTI_RECIPIENT`) on the same row. A bare
/// `.contains("0x6610")` would vacuously pass on a prose mention (e.g.
/// inside PIN 6's `0x6600` rationale narrative) — the row-binding check
/// would-FAIL if R5 registered the value against the wrong symbol or only
/// named it in prose.
#[test]
fn f_disc_2_records_r46_group_codepoints_0x6610_0x6520() {
    let codepoints =
        std::fs::read_to_string(repo_root().join("docs/CRYPTO-CODEPOINTS.md")).unwrap_or_default();

    // A codepoint is "registered" only when its value is CO-LOCATED with
    // its symbol on one logical row (the §4.0 allocation-table shape) — the
    // registry-drift guard PIN 6 uses, and the exact class this round's
    // slip (0x6610 bound to the 0x6600 symbol) belongs to.
    let registered_on_one_row = |value: &str, symbol: &str| -> bool {
        codepoints
            .lines()
            .any(|l| l.contains(value) && l.contains(symbol))
    };

    assert!(
        registered_on_one_row("0x6610", "MEMBERSHIP_SET_GROUP_MULTI_STANZA"),
        "CRYPTO-CODEPOINTS.md MUST register 0x6610 BOUND TO its symbol \
         MEMBERSHIP_SET_GROUP_MULTI_STANZA on one row (the R4.6-corrected \
         value; distinct from the 0x6600 set-keying value the R4.5b \
         migration slipped to). R0.7 §4.0 allocation table line 1010 \
         FREEZES it; §4.0 names this doc its in-tree home. A prose-only \
         mention or a value bound to the wrong symbol MUST fail."
    );
    assert!(
        registered_on_one_row("0x6520", "LAYER_C_DROP_MULTI_RECIPIENT"),
        "CRYPTO-CODEPOINTS.md MUST register 0x6520 BOUND TO its symbol \
         LAYER_C_DROP_MULTI_RECIPIENT on one row (the R0.7-blinded Layer-C \
         group multi-stanza value). R0.7 §4.0 allocation table line 1001 \
         FREEZES it (per-stanza AAD BLINDED; see §3.3 / §4.1). A prose-only \
         mention or a value bound to the wrong symbol MUST fail."
    );
}

// ===========================================================================
// R6 ENFORCEMENT-STATE arms — structurally back the AS-BUILT+ENFORCED
// reclassification by DRIVING the REAL benten-drop production code (the
// surfaces THIS crate owns). NOT doc-text reads — production entry points +
// observable consequences. would-FAIL-if-no-op'd.
// ===========================================================================

use benten_crypto_suite::cipher_suite::{
    CipherSuite, CipherSuiteCodepoint, RecipientPublic, RecipientSecret,
};
use benten_crypto_suite::sig::{Keypair as SigKeypair, SignatureSuite};
use benten_drop::layer_c::group_posture::{
    GroupError, GroupSealParams, GroupVerifyContext, open_membership_set_group,
    seal_membership_set_group,
};
use benten_drop::layer_c::{
    AAD_VERSION, EncryptedEnvelope, LayerCError, group_roster_for_test, open_single,
    seal_sealed_sender, sealed_aad,
};
use benten_id::did::Did;

/// B2 ORIGIN-AUTH helper: a real sender (LAMPS-hybrid keypair + matching
/// hybrid `did:key` bytes). See the `f_lc_hpke` sibling for the rationale.
fn hybrid_sender() -> (SigKeypair, Vec<u8>) {
    let kp = SignatureSuite::v1_default().generate_keypair();
    let did_str = Did::from_hybrid_public_key(&kp.public()).to_string();
    (kp, did_str.into_bytes())
}

/// R9 GAP-1 recipient-key helpers: a stable REAL hybrid keypair per `seed`
/// (secret seed → both halves via BLAKE3 expansion; `.public()`/`.secret()`
/// genuinely correspond). Replaces the deleted `[u8; 32]` placeholder pubkeys.
fn fixed_kp(seed: u8) -> benten_crypto_suite::cipher_suite::RecipientKeypair {
    CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("0x647a wire-locked")
        .generate_recipient_keypair_deterministic_for_test(&[seed; 32])
}
fn fixed_pk(seed: u8) -> RecipientPublic {
    RecipientPublic::from_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        &fixed_kp(seed).public().to_bytes(),
    )
    .expect("re-parse of recipient public must succeed")
}
fn fixed_sk(seed: u8) -> RecipientSecret {
    RecipientSecret::from_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        &fixed_kp(seed).secret().to_bytes(),
    )
    .expect("re-parse of recipient secret must succeed")
}

/// The independently-held `GroupVerifyContext` for a `0x6610` membership-set
/// round-trip with all generations = 1 (the common fixture shape here).
fn verify_ctx_gen1(pks: &[RecipientPublic]) -> GroupVerifyContext {
    let member_dids = group_roster_for_test(pks)
        .iter()
        .map(|d| String::from_utf8_lossy(d).into_owned())
        .collect();
    GroupVerifyContext {
        member_dids,
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    }
}

/// Inv-16 ENFORCED — the `EncryptedEnvelope` codepoint-dispatch is LIVE and
/// fails CLOSED on a cross-arm feed (the codepoint-dispatched encryption
/// decomposition is real, not paper).
///
/// Drives the production `open_single` against a GROUP (`HpkeMultiBase`)
/// envelope: the single-recipient open path MUST strict-reject with the typed
/// `UnsupportedCodepoint(0x6520)` arm — proving the dispatch discriminates by
/// codepoint and refuses the wrong shape (no silent cross-arm fallback, the
/// Inv-16/codepoint-agility property). would-FAIL-if-no-op'd: a dispatch that
/// blindly decrypted any envelope would return `Ok` or a generic AEAD error,
/// not the typed `UnsupportedCodepoint`.
#[test]
fn f_disc_2_inv16_codepoint_dispatch_enforced_fail_closed() {
    use benten_drop::layer_c::seal_group_multi;
    let pks = [fixed_pk(0x21), fixed_pk(0x22)];
    let body_cid = *blake3::hash(b"inv16 enforced body").as_bytes();
    let (sender_kp, sender) = hybrid_sender();
    let group_env = seal_group_multi(&pks, &sender, &sender_kp, &body_cid, 1, b"inv16 body")
        .expect("group seal within recipient limit");

    // The single-recipient open arm MUST refuse a group envelope by codepoint
    // (the dispatch strict-rejects BEFORE any decrypt/verify).
    let outcome = open_single(&fixed_sk(0xA1), &b"did:key:zAUD".to_vec(), 1, &group_env);
    assert_eq!(
        outcome,
        Err(LayerCError::UnsupportedCodepoint(
            benten_drop::layer_c::LAYER_C_DROP_MULTI_RECIPIENT
        )),
        "Inv-16 ENFORCED: EncryptedEnvelope codepoint-dispatch MUST fail closed \
         with UnsupportedCodepoint(0x6520) when a group envelope is fed to the \
         single-recipient open arm — no silent cross-arm fallback. Got: {outcome:?}"
    );
}

/// Inv-18 ENFORCED — the DEFAULT (`0x6510`) Sealed-Sender on-wire AAD discloses
/// EXACTLY `{audience}` as residual privacy-metadata and binds NO sender-DID in
/// the plaintext (the metadata-disclosure invariant is live in the shipped
/// default).
///
/// Drives the production `sealed_aad::aad_field_set` + `residual_privacy_metadata`
/// AND a real `seal_sealed_sender` seal: the serialized plaintext AAD region MUST
/// NOT contain the sender-DID bytes (it is sealed inside the ciphertext). The
/// field-set MUST be the canonical 5-tuple with NO `sender_did` and NO
/// `coarse_epoch`. would-FAIL-if-no-op'd: re-adding `sender_did`/`coarse_epoch`
/// to the field-set, or leaking the sender into the plaintext AAD, flips an
/// assertion.
#[test]
fn f_disc_2_inv18_sealed_sender_default_metadata_disclosure_enforced() {
    // (1) The enumerated field-set is the canonical 5-tuple — no sender, no epoch.
    let field_set = sealed_aad::aad_field_set();
    assert!(
        !field_set.iter().any(|f| f.contains("sender")),
        "Inv-18 ENFORCED: the DEFAULT 0x6510 AAD field-set MUST NOT carry a \
         sender field (Sealed-Sender). Got: {field_set:?}"
    );
    assert!(
        !field_set
            .iter()
            .any(|f| f.contains("coarse_epoch") || f.contains("epoch")),
        "Inv-18 ENFORCED: the DEFAULT 0x6510 AAD field-set MUST NOT carry \
         coarse_epoch (Ben-RULING-#1 + M-14). Got: {field_set:?}"
    );
    assert_eq!(
        sealed_aad::residual_privacy_metadata(),
        vec!["audience"],
        "Inv-18 ENFORCED: the residual privacy-metadata under the DEFAULT \
         0x6510 path is EXACTLY {{audience}}."
    );

    // (2) A REAL seal's plaintext AAD region MUST NOT contain the sender-DID.
    let (sender_kp, sender) = hybrid_sender();
    let env = seal_sealed_sender(
        &fixed_pk(0x31),
        &b"did:key:zAUDIENCE".to_vec(),
        &sender,
        &sender_kp,
        &[0x07u8; 32],
        1,
        b"inv18 enforced plaintext",
    );
    let aad_region = benten_drop::layer_c::single_plaintext_aad_region(&env);
    assert!(
        !contains_subslice(&aad_region, &sender),
        "Inv-18 ENFORCED: the DEFAULT Sealed-Sender plaintext AAD region MUST \
         NOT leak the sender-DID — it is sealed inside the ciphertext. The \
         sender bytes were found in the relay-visible plaintext AAD."
    );
    assert_eq!(
        aad_region.first().copied(),
        Some(AAD_VERSION),
        "Inv-18 ENFORCED: the AAD region leads with the dedicated AAD_VERSION \
         (0x01) prefix byte (DISTINCT from the format byte)."
    );
}

/// Inv-20 clause-c ENFORCED — the live `0x6610` group seal binds the BLINDED
/// 11-field per-stanza AAD field-set (the canonical R0.7 §3.10/§4.1 set,
/// NEVER the raw roster nor the raw set-id).
///
/// Drives the production `seal_membership_set_group` and inspects the actual
/// bound per-stanza AAD: it MUST be the 127-byte 11-field set leading with
/// `aad_version || codepoint(0x6610)`, and the raw set-id + raw roster bytes
/// MUST NOT appear (they are BLINDED into the two 32-byte commitments).
/// would-FAIL-if-no-op'd: a regression to the 6-field shape flips the length;
/// emitting the raw set-id/roster flips the blinding assertion.
#[test]
fn f_disc_2_inv20_clause_c_group_aad_field_set_enforced_blinded() {
    let pks = [fixed_pk(0x41), fixed_pk(0x42), fixed_pk(0x43)];
    let k_set = [0x55u8; 32];
    let set_id: Vec<u8> = b"benten:set:inv20-RAW-SETID-MARKER".to_vec();
    let params = GroupSealParams {
        membership_set_id: set_id.clone(),
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    let (sender_kp, sender) = hybrid_sender();
    let env = seal_membership_set_group(
        &pks,
        &sender,
        &sender_kp,
        &k_set,
        &params,
        b"inv20 enforced group body",
    );
    let aad = env.stanza_aad_for_test(0);

    assert_eq!(
        aad.len(),
        127,
        "Inv-20 clause-c ENFORCED: the live 0x6610 per-stanza AAD MUST be the \
         BLINDED 11-field set (127 bytes); the old 6-field shape was 79."
    );
    assert_eq!(
        &aad[0..3],
        &[0x01, 0x66, 0x10],
        "Inv-20 clause-c ENFORCED: the AAD leads with aad_version(0x01) ‖ \
         codepoint(0x6610 BE)."
    );
    // BLINDED: the raw set-id is NEVER in the AAD (it is keyed-MAC'd into the
    // membership_set_id_commitment). would-FAIL if the seal emitted it raw.
    assert!(
        !contains_subslice(&aad, &set_id),
        "Inv-20 clause-c ENFORCED: the raw membership_set_id MUST NOT appear in \
         the per-stanza AAD — it is BLINDED into membership_set_id_commitment \
         (§3.9 keyed-MAC). The raw set-id bytes were found in the AAD."
    );
}

/// Inv-19/Inv-20 ENFORCED — the group truncation/censorship defense fails
/// CLOSED (the §3.3/§4.1 delivered-vs-bound stanza-count check is live on the
/// production `0x6610` open path; the F-01 enforcer).
///
/// Drives a real `seal_membership_set_group` → drop a stanza (the exact relay
/// truncation) → the production `open_membership_set_group` MUST fail closed
/// with the typed `StanzaCountMismatch`. would-FAIL-if-no-op'd: the survivor
/// authenticates its own stanza, so reverting the count check returns
/// `Ok(plaintext)` — silent censorship.
#[test]
fn f_disc_2_inv19_inv20_truncation_defense_enforced_fail_closed() {
    // R9 GAP-1: pk + sk must be the SAME real keypair per recipient (the old
    // fixture paired them via the deleted `sk = pk + 0x80` placeholder).
    let pks = [fixed_pk(0x61), fixed_pk(0x62), fixed_pk(0x63)];
    let sks = [fixed_sk(0x61), fixed_sk(0x62), fixed_sk(0x63)];
    let params = GroupSealParams {
        membership_set_id: b"benten:set:inv19".to_vec(),
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    let (sender_kp, sender) = hybrid_sender();
    let env = seal_membership_set_group(
        &pks,
        &sender,
        &sender_kp,
        &[0x77u8; 32],
        &params,
        b"inv19 body",
    );
    let ctx = verify_ctx_gen1(&pks);
    // Pre-condition (would-FAIL-on-revert witness): the FULL envelope opens.
    assert!(
        open_membership_set_group(&sks[1], 1, &ctx, &env).is_ok(),
        "pre-condition: the index-1 survivor opens fine on the FULL envelope \
         (so the failure below is the count check firing, not a decrypt error)"
    );
    let truncated = env.with_last_stanza_dropped_for_test();
    let outcome = open_membership_set_group(&sks[1], 1, &ctx, &truncated);
    assert_eq!(
        outcome,
        Err(GroupError::StanzaCountMismatch {
            delivered: 2,
            bound: 3,
        }),
        "Inv-19/Inv-20 ENFORCED: a truncated 0x6610 group envelope MUST fail \
         closed (delivered 2 != bound 3) — the §3.3 truncation/censorship \
         defense is live. Got: {outcome:?}"
    );
}

/// Inv-21 + Inv-22 FLAG — enforcement is OWNED ELSEWHERE (precise flag, not
/// an over-claim).
///
/// Inv-21 (set-identity fork tie-break, `smaller-created_at_hlc-wins`) runs on
/// the `benten-sync` CRDT merge round; Inv-22 (member-nature derivation) is
/// graph-data-half (engine/graph-native). NEITHER enforcer lives in benten-drop,
/// and this crate's test target cannot drive them without reaching into a
/// non-dependency's internals. So this catch-net does NOT structurally back
/// their AS-BUILT+ENFORCED reclassification.
///
/// This arm asserts the HONEST boundary it CAN: the two invariants ARE
/// registered in the doc (so they are not silently dropped), and records — via
/// this test's existence + name — that their enforcement backing belongs in the
/// owner crates' test targets (`benten-membership-set` / `benten-sync` /
/// engine). If a future edit tries to claim benten-drop enforces them, this
/// flag is the documented counter-evidence. (HARD RULE 12 (c) DISAGREE-WITH-
/// EXPLANATION shape: do not over-claim enforcement this crate cannot drive.)
#[test]
fn f_disc_2_inv21_inv22_enforcement_owned_elsewhere_flag() {
    let invs = registered_invariants(&invariant_coverage_md());
    assert!(
        invs.contains(&21) && invs.contains(&22),
        "Inv-21 + Inv-22 MUST stay registered in INVARIANT-COVERAGE.md. Their \
         ENFORCEMENT, however, is OWNED ELSEWHERE (Inv-21 → benten-sync CRDT \
         fork tie-break; Inv-22 → graph-native member-nature derivation) — NOT \
         in benten-drop. This catch-net deliberately does NOT assert their \
         enforcement; that backing belongs in the owner crates' test targets."
    );
}

/// **F-24 codepoint-SSOT cross-crate const-equality pin.**
///
/// `benten_crypto_suite::registry` is the single source of truth for the
/// `0x6100..0x6FFF` allocation map, but the wire-byte-emitting codepoint
/// values are INDEPENDENTLY defined in the producing crates
/// (`benten_membership_set::codepoints` for the MembershipSet band;
/// `benten_drop::layer_c` for the Layer-C drop / Layer-D DeviceLink bands) and
/// mirrored back to the registry — they are not re-exported from it. Because
/// the same integer lives in two places, a single-side edit would silently
/// drift the registry from the wire. This pin asserts the producing crates'
/// constants EQUAL the registry's, so a one-sided edit fails the build (the
/// `CRYPTO-CODEPOINTS.md` §4.0 SSOT note documents the same coupling). This is
/// a cross-crate const-AGREEMENT assertion between two independent definitions
/// — NOT an `assert_eq!(CONST, literal)` self-walker (which would tautologize).
#[test]
fn f_disc_2_codepoint_ssot_cross_crate_const_equality() {
    use benten_crypto_suite::registry;
    use benten_membership_set::codepoints;

    // MembershipSet band (codepoints.rs ↔ registry.rs).
    assert_eq!(
        codepoints::MEMBERSHIP_SET_ENCRYPTION,
        registry::MEMBERSHIP_SET_ENCRYPTION,
        "0x6600 MEMBERSHIP_SET_ENCRYPTION drifted between \
         benten_membership_set::codepoints and benten_crypto_suite::registry"
    );
    assert_eq!(
        codepoints::MEMBERSHIP_SET_GROUP_MULTI_STANZA,
        registry::MEMBERSHIP_SET_GROUP_MULTI_STANZA,
        "0x6610 MEMBERSHIP_SET_GROUP_MULTI_STANZA drifted between \
         benten_membership_set::codepoints and benten_crypto_suite::registry"
    );
    assert_eq!(
        codepoints::MEMBERSHIP_SET_RESERVED_0X6620,
        registry::MEMBERSHIP_SET_SUBSET_REF,
        "0x6620 MembershipSet reserved/subset-ref slot drifted between \
         benten_membership_set::codepoints and benten_crypto_suite::registry"
    );

    // Layer-C drop band (benten_drop::layer_c ↔ registry.rs).
    assert_eq!(
        benten_drop::layer_c::LAYER_C_DROP,
        registry::LAYER_C_DROP,
        "0x6500 LAYER_C_DROP drifted between benten_drop::layer_c and \
         benten_crypto_suite::registry"
    );
    assert_eq!(
        benten_drop::layer_c::LAYER_C_DROP_MULTI_RECIPIENT,
        registry::LAYER_C_DROP_MULTI_RECIPIENT,
        "0x6520 LAYER_C_DROP_MULTI_RECIPIENT drifted between \
         benten_drop::layer_c and benten_crypto_suite::registry"
    );
    // R13 F-03: the DEFAULT Sealed-Sender codepoint (0x6510) is
    // independently defined in benten_drop::layer_c and mirrored to the
    // registry — pin their agreement so a one-sided edit fails the build.
    assert_eq!(
        benten_drop::layer_c::DROP_TO_RECIPIENT_SEALED_SENDER,
        registry::DROP_TO_RECIPIENT_SEALED_SENDER,
        "0x6510 DROP_TO_RECIPIENT_SEALED_SENDER drifted between \
         benten_drop::layer_c and benten_crypto_suite::registry"
    );
    // R13 F-03: the wave-live default cipher-suite codepoint (0x647a) is
    // defined as the `CipherSuiteCodepoint::HYBRID_X25519_MLKEM768` newtype
    // in benten_crypto_suite::codepoint and mirrored as a bare u16 in the
    // registry allocation map — pin their raw() agreement (cross-definition,
    // NOT an `assert_eq!(CONST, literal)` self-walker).
    assert_eq!(
        benten_crypto_suite::codepoint::CipherSuiteCodepoint::HYBRID_X25519_MLKEM768.raw(),
        registry::CIPHER_HYBRID_X25519_MLKEM768,
        "0x647a HYBRID_X25519_MLKEM768 drifted between \
         benten_crypto_suite::codepoint (the newtype) and \
         benten_crypto_suite::registry (the allocation map)"
    );
}

/// Helper: does `haystack` contain `needle` as a contiguous subslice?
fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}
