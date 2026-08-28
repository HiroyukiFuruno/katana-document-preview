use std::fs;

const KUC_REQUIRED_DEPENDENCIES: [&str; 1] = ["katana-ui-core"];
const FORBIDDEN_VIEWER_BACKEND_DEPENDENCIES: [&str; 3] = ["egui", "winit", "vello"];
const FORBIDDEN_PUBLIC_API_FRAGMENTS: [&str; 4] =
    ["katana_ui_core", "egui::", "winit::", "vello::"];

#[test]
fn viewer_manifest_keeps_backend_dependencies_out() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(manifest_path)?;
    let value: toml::Value = toml::from_str(&manifest)?;
    let dependencies = value
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| std::io::Error::other("dependencies section missing"))?;

    for dependency in FORBIDDEN_VIEWER_BACKEND_DEPENDENCIES {
        assert!(
            !dependencies.contains_key(dependency),
            "{dependency} must stay out of katana-document-viewer"
        );
    }
    Ok(())
}

#[test]
fn viewer_manifest_has_kuc_contract_dependency() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(manifest_path)?;
    let value: toml::Value = toml::from_str(&manifest)?;
    let dependencies = value
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| std::io::Error::other("dependencies section missing"))?;

    for dependency in KUC_REQUIRED_DEPENDENCIES {
        assert!(
            dependencies.contains_key(dependency),
            "{dependency} must be present in katana-document-viewer dependencies"
        );
    }
    Ok(())
}

#[test]
fn viewer_public_api_keeps_ui_vendor_types_out() -> Result<(), Box<dyn std::error::Error>> {
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
