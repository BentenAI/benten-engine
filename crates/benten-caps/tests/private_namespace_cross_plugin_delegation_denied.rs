//! T7a defense pin — cross-plugin namespace escape.
//!
//! Plugin Y attempts to issue itself a UCAN cap for plugin X's
//! `admin-ui-private` namespace. Cap-policy backend MUST refuse —
//! private-namespace scope is structurally `shares: none`.
//!
//! Surface: `crates/benten-caps/src/private_namespace_policy.rs` (NEW).

#[test]
#[ignore = "RED-PHASE: G24-D wave wires private_namespace_policy::reject_cross_plugin; un-ignore at G24-D landing"]
fn private_namespace_cap_rejects_cross_plugin_delegation_at_policy_layer() {
    // Future surface:
    //   private_namespace_policy::check_delegation(from_plugin, to_
    //     plugin, cap_scope) -> Result
    // returns ErrorCode::PluginPrivateNamespaceDelegationForbidden
    // when cap_scope matches `private:*` AND from_plugin != to_plugin.
    //
    // FAILS-IF-NO-OP because no-op admits any cap that the source
    // plugin's UCAN chain signature-verifies for.
    panic!("RED-PHASE: G24-D wave must wire private_namespace_policy");
}
