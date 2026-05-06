//! G14-A1 wave-4a — `benten-id` dependency-edge architectural pin
//! (un-ignored at landing; arch-r1-10).

#![allow(clippy::unwrap_used)]

#[test]
fn benten_id_no_unauthorized_dependency_edges() {
    let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let toml: toml::Value = manifest.parse().unwrap();
    let deps_keys: Vec<String> = toml
        .get("dependencies")
        .and_then(|d| d.as_table())
        .map(|t| t.keys().cloned().collect())
        .unwrap_or_default();

    const FORBIDDEN: &[&str] = &[
        "benten-graph",
        "benten-engine",
        "benten-eval",
        "benten-caps",
        "benten-ivm",
        "benten-sync",
        "benten-dsl-compiler",
    ];

    for dep in &deps_keys {
        for forbidden in FORBIDDEN {
            assert!(
                dep != forbidden,
                "benten-id MUST NOT depend on {forbidden} per arch-r1-10; full dep list: {deps_keys:?}"
            );
        }
    }
}
