use std::fs;

const FORBIDDEN_VIEWER_DEPENDENCIES: [&str; 4] = ["eframe", "egui", "winit", "vello"];
const FORBIDDEN_PUBLIC_API_FRAGMENTS: [&str; 4] =
    ["egui::", "katana_ui_core", "winit::", "vello::"];

#[test]
fn viewer_manifest_is_backend_neutral_and_uses_registry_kuc()
-> Result<(), Box<dyn std::error::Error>> {
    let value = viewer_manifest()?;
    let dependencies = manifest_dependencies(&value)?;

    assert_neutral_dependencies(dependencies);
    assert_registry_dependency(dependencies, "katana-ui-core", "=0.3.7")?;
    assert_no_egui_feature(&value);
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
        assert!(
            !dependencies.contains_key(dependency),
            "{dependency} must stay out of katana-document-viewer"
        );
    }
    assert!(
        !dependencies
            .keys()
            .any(|dependency| dependency.starts_with("katana-document-viewer-"))
    );
}

fn assert_no_egui_feature(value: &toml::Value) {
    let feature = value
        .get("features")
        .and_then(toml::Value::as_table)
        .and_then(|features| features.get("egui"));
    assert!(feature.is_none());
}

fn assert_registry_dependency(
    dependencies: &toml::Table,
    name: &str,
    version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(dependency) = dependencies.get(name) else {
        return Err(std::io::Error::other(format!("{name} dependency is missing")).into());
    };
    assert_eq!(Some(version), dependency.as_str());
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

#[test]
fn windows_workers_launch_only_the_workspace_staged_executable()
-> Result<(), Box<dyn std::error::Error>> {
    let source_root = format!("{}/src/multi_format", env!("CARGO_MANIFEST_DIR"));
    let staging = fs::read_to_string(format!("{source_root}/windows_worker_executable.rs"))?;
    let office = fs::read_to_string(format!("{source_root}/office_worker_process_windows.rs"))?;
    let spreadsheet =
        fs::read_to_string(format!("{source_root}/spreadsheet_worker_spawn_windows.rs"))?;
    let spreadsheet_stderr = fs::read_to_string(format!(
        "{source_root}/spreadsheet_worker_spawn_windows_stderr.rs"
    ))?;
    let workspace = fs::read_to_string(format!("{source_root}/office_worker_workspace.rs"))?;
    let profile = fs::read_to_string(format!("{source_root}/windows_worker_profile.rs"))?;

    assert_windows_worker_launch_contract(&staging, &office, &spreadsheet, &workspace, &profile);
    assert_windows_spreadsheet_stdio_contract(&spreadsheet, &spreadsheet_stderr);
    Ok(())
}

fn assert_windows_worker_launch_contract(
    staging: &str,
    office: &str,
    spreadsheet: &str,
    workspace: &str,
    profile: &str,
) {
    assert!(staging.contains("workspace.join(STAGED_WORKER_NAME)"));
    assert!(staging.contains("std::fs::copy(&config.executable, &destination)"));
    assert!(workspace.contains("windows_worker_profile::workspace_root(config)?"));
    assert!(workspace.contains("tempdir_in(root)"));
    assert_windows_worker_profile_contract(profile);
    for worker in [&office, &spreadsheet] {
        assert!(worker.contains("stage_windows_worker(workspace, config)?"));
        assert!(worker.contains("exe: staged_executable.to_path_buf()"));
        assert!(worker.contains("staged_executable.to_string_lossy().into_owned()"));
        assert!(worker.contains("office_worker_protocol::INPUT_NAME"));
        assert!(!worker.contains("ResourcePath::File(config.executable.clone())"));
    }
    assert!(!office.contains("std::process::Command"));
    assert!(!spreadsheet.contains("std::process::Command"));
}

fn assert_windows_spreadsheet_stdio_contract(spreadsheet: &str, stderr: &str) {
    for marker in [
        "stdio: StdioConfig::Pipe",
        "child.stderr.take()",
        "spawn_stderr_reader(stderr, debug_enabled)",
    ] {
        assert!(
            spreadsheet.contains(marker),
            "Windows spreadsheet worker must preserve stderr spawn contract: {marker}"
        );
    }
    for marker in [
        "forward_debug_stderr(&mut source)",
        "forward_stderr_chunks(source, |chunk| {",
        "std::io::stderr().lock()",
        "std::io::sink()",
    ] {
        assert!(
            stderr.contains(marker),
            "Windows spreadsheet worker must preserve stderr drain contract: {marker}"
        );
    }
    assert!(
        !stderr.contains(
            "let mut parent_stderr = std::io::stderr().lock();\n        forward_stderr(&mut source, &mut parent_stderr);"
        ),
        "Windows DEBUG stderr relay must not retain the parent stderr lock until worker EOF"
    );
}

fn assert_windows_worker_profile_contract(profile: &str) {
    for marker in [
        ".join(PROFILE_NAME)",
        ".join(\"AC\")",
        ".join(\"Temp\")",
        "std::env::vars_os()",
        "OsString::from(\"TEMP\")",
        "OsString::from(\"TMP\")",
        "environment.sort_by",
    ] {
        assert!(profile.contains(marker), "missing profile marker: {marker}");
    }
}

#[test]
fn public_document_surface_does_not_expose_kuc_event_types() {
    let surface = include_str!("document_surface/spreadsheet_grid.rs");
    let api = include_str!("document_surface/mod.rs");

    assert!(surface.contains("-> super::DocumentGridEvent"));
    assert!(api.contains("pub enum DocumentGridEvent"));
    assert!(!surface.contains("-> GridEvent"));
}
