use std::fs;

const FORBIDDEN_VIEWER_DEPENDENCIES: [&str; 2] = ["winit", "vello"];
const FORBIDDEN_PUBLIC_API_FRAGMENTS: [&str; 3] = ["katana_ui_core", "winit::", "vello::"];

#[test]
fn viewer_manifest_scopes_presentation_dependencies_to_the_egui_feature()
-> Result<(), Box<dyn std::error::Error>> {
    let value = viewer_manifest()?;
    let dependencies = manifest_dependencies(&value)?;

    assert_neutral_dependencies(dependencies);
    assert_optional_registry_dependency(dependencies, "egui", "0.35")?;
    assert_optional_registry_dependency(dependencies, "katana-ui-core", "0.3.0")?;
    assert_egui_feature(&value)?;
    Ok(())
}

fn viewer_manifest() -> Result<toml::Value, Box<dyn std::error::Error>> {
    let manifest_path = format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(manifest_path)?;
    Ok(toml::from_str(&manifest)?)
}

fn manifest_dependencies(value: &toml::Value) -> Result<&toml::Table, Box<dyn std::error::Error>> {
    value
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| std::io::Error::other("dependencies section missing").into())
}

fn assert_neutral_dependencies(dependencies: &toml::Table) {
    for dependency in FORBIDDEN_VIEWER_DEPENDENCIES {
        assert!(!dependencies.contains_key(dependency));
    }
    assert!(
        !dependencies
            .keys()
            .any(|dependency| dependency.starts_with("katana-document-viewer-"))
    );
}

fn assert_egui_feature(value: &toml::Value) -> Result<(), Box<dyn std::error::Error>> {
    let feature = value
        .get("features")
        .and_then(toml::Value::as_table)
        .and_then(|features| features.get("egui"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| std::io::Error::other("egui feature is missing"))?;
    let feature = feature
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(vec!["dep:egui", "dep:katana-ui-core"], feature);
    Ok(())
}

fn assert_optional_registry_dependency(
    dependencies: &toml::Table,
    name: &str,
    version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let dependency = dependencies
        .get(name)
        .and_then(toml::Value::as_table)
        .ok_or_else(|| std::io::Error::other(format!("{name} dependency is missing")))?;
    assert_eq!(
        Some(version),
        dependency.get("version").and_then(toml::Value::as_str)
    );
    assert_eq!(
        Some(true),
        dependency.get("optional").and_then(toml::Value::as_bool)
    );
    assert!(!dependency.contains_key("path"));
    assert!(!dependency.contains_key("git"));
    Ok(())
}

#[test]
fn viewer_public_api_does_not_expose_kuc_or_vendor_types() -> Result<(), Box<dyn std::error::Error>>
{
    let lib_path = format!("{}/src/lib.rs", env!("CARGO_MANIFEST_DIR"));
    let lib = fs::read_to_string(lib_path)?;

    for fragment in FORBIDDEN_PUBLIC_API_FRAGMENTS {
        assert!(
            !lib.contains(fragment),
            "{fragment} must stay out of katana-document-viewer public API"
        );
    }
    Ok(())
}
