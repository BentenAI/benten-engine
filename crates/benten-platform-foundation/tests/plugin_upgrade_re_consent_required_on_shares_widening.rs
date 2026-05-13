//! G24-D row pin — upgrade re-consent on cap growth.
//!
//! Per post-R1-triage ratification #8 cap-change-triggered fresh
//! consent rule:
//!   - silent within-lineage upgrade if `requires` is a STRICT SUBSET
//!     of installed manifest
//!   - full re-consent if `requires` GREW (any cap added or scope
//!     widened)
//!   - cross-fork merge = user-initiated through same consent flow

mod common;

use benten_platform_foundation::CapRequirement;
use benten_platform_foundation::module_ecosystem::{
    UpgradeConsentDecision, decide_upgrade_consent,
};
use common::manifest_fixtures::minimal_manifest;

#[test]
fn upgrade_with_strict_subset_requires_silent_within_lineage_upgrade() {
    // SUBSTANTIVE per pim-2 §3.6b: build old manifest with TWO caps;
    // new manifest with ONE (subset). decide_upgrade_consent at HEAD
    // returns Silent. Would-FAIL if cap-diff logic missed strict-subset
    // path.
    let mut old = minimal_manifest();
    old.requires = vec![
        CapRequirement {
            scope: "store:notes:read".to_string(),
        },
        CapRequirement {
            scope: "store:notes:write".to_string(),
        },
    ];

    let mut new_smaller = minimal_manifest();
    new_smaller.requires = vec![CapRequirement {
        scope: "store:notes:read".to_string(),
    }];

    let decision = decide_upgrade_consent(&old, &new_smaller);
    assert_eq!(
        decision,
        UpgradeConsentDecision::Silent,
        "strict-subset MUST be Silent; would-FAIL if cap-diff misread \
         as growth"
    );
}

#[test]
fn upgrade_with_widened_requires_surfaces_consent_required() {
    // SUBSTANTIVE per pim-2 §3.6b: cap growth (added scope) triggers
    // ConsentRequired. Would-FAIL if cap-diff missed the growth case.
    let mut old = minimal_manifest();
    old.requires = vec![CapRequirement {
        scope: "store:notes:read".to_string(),
    }];

    let mut new_wider = minimal_manifest();
    new_wider.requires = vec![
        CapRequirement {
            scope: "store:notes:read".to_string(),
        },
        CapRequirement {
            scope: "store:notes:write".to_string(),
        }, // NEW
    ];

    let decision = decide_upgrade_consent(&old, &new_wider);
    assert_eq!(
        decision,
        UpgradeConsentDecision::ConsentRequired,
        "cap growth MUST trigger ConsentRequired"
    );
}

#[test]
fn upgrade_with_identical_requires_set_is_silent() {
    // SUBSTANTIVE boundary per pim-2 §3.6b: identical requires set is
    // a degenerate subset → Silent. Would-FAIL if equal-not-strict-
    // subset arm was missed.
    let mut old = minimal_manifest();
    old.requires = vec![CapRequirement {
        scope: "store:notes:read".to_string(),
    }];

    let mut new_same = minimal_manifest();
    new_same.requires = vec![CapRequirement {
        scope: "store:notes:read".to_string(),
    }];

    let decision = decide_upgrade_consent(&old, &new_same);
    assert_eq!(decision, UpgradeConsentDecision::Silent);
}
