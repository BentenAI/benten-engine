//! T7 canonicalization pin — private namespace cap-scope shape.
//!
//! Per docs/PLUGIN-MANIFEST.md §6 + T7 defense: private namespace
//! cap-scopes are formed as `private:<plugin_did>:<label>` (typed
//! prefix). The cap-policy backend rejects any cap-scope matching the
//! `private:*` prefix where the requester's DID doesn't match
//! `<plugin_did>`.

#[test]
#[ignore = "RED-PHASE: G24-D wave canonicalizes private-namespace scope shape; un-ignore at G24-D landing"]
fn private_namespace_scope_string_requires_plugin_did_segment_match_requester() {
    // Future surface:
    //   private_namespace_policy::canonicalize_scope(scope) ->
    //     Result<CanonicalScope>
    // where the parse extracts the embedded plugin-DID and validates
    // the structural shape. Mismatched plugin-DIDs in the scope
    // string get rejected.
    //
    // FAILS-IF-NO-OP because canonicalization is what binds the
    // scope-string to the policy-enforcement layer.
    panic!("RED-PHASE: G24-D wave must canonicalize private:<plugin_did>:<label> scope strings");
}
