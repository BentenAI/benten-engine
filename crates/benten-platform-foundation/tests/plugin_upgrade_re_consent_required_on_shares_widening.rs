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
use common::manifest_fixtures::minimal_manifest;

#[test]
#[ignore = "RED-PHASE: G24-D wave wires upgrade cap-diff logic; un-ignore at G24-D landing"]
fn upgrade_with_strict_subset_requires_silent_within_lineage_upgrade() {
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

    // Future surface:
    //   plugin_lifecycle::upgrade(old_cid, new_cid) ->
    //     Result<UpgradeOutcome>
    //   UpgradeOutcome::SilentWithinLineage(new_cid)
    //   UpgradeOutcome::RequiresReconsent(new_manifest)
    panic!("RED-PHASE: G24-D wave must wire upgrade cap-diff logic");
}

#[test]
#[ignore = "RED-PHASE: G24-D wave wires re-consent on cap growth; un-ignore at G24-D landing"]
fn upgrade_with_widened_requires_surfaces_e_plugin_install_consent_required() {
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

    // FAILS-IF-NO-OP because the cap-diff must explicitly compute
    // the delta and route to re-consent.
    panic!("RED-PHASE: G24-D wave must wire E_PLUGIN_INSTALL_CONSENT_REQUIRED on cap growth");
}
