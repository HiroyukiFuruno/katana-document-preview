use crate::PreviewAssetLoader;
use crate::{ArtifactFormat, KdvThemeSnapshot, KrrDiagramRenderEngine, ViewerNodePlanner};
use std::path::Path;

use super::test_support::{
    ErrorDiagramEngine, FakeDiagramEngine, output_for, output_for_document, temp_image_path,
};

#[test]
fn loader_materializes_visible_diagram_asset() -> Result<(), Box<dyn std::error::Error>> {
    let output = output_for("```mermaid\ngraph TD\n  A --> B\n```")?;
    let plan = ViewerNodePlanner::create(&output.input, output.scroll_offset);
    let request = plan.asset_requests.first().ok_or("asset request missing")?;

    let (loaded, report) = PreviewAssetLoader::new(FakeDiagramEngine)
        .load_requested(&output, &KdvThemeSnapshot::katana_light())?;

    assert_eq!(1, report.loaded_artifact_count);
    assert_eq!(0, report.failed_artifact_count);
    assert_eq!(1, loaded.input.artifacts.len());
    assert_eq!(request.artifact_id, loaded.input.artifacts[0].manifest.id);
    assert_eq!(
        ArtifactFormat::Svg,
        loaded.input.artifacts[0].manifest.format
    );
    assert!(loaded.input.artifacts[0].bytes.bytes.starts_with(b"<svg"));
    Ok(())
}

#[test]
fn loader_records_diagram_error_artifact() -> Result<(), Box<dyn std::error::Error>> {
    let output = output_for("```mermaid\ngraph TD\n  A --> B\n```")?;

    let (loaded, report) = PreviewAssetLoader::new(ErrorDiagramEngine)
        .load_requested(&output, &KdvThemeSnapshot::katana_light())?;

    assert_eq!(0, report.loaded_artifact_count);
    assert_eq!(1, report.failed_artifact_count);
    assert_eq!(
        1,
        loaded.input.artifacts[0].manifest.diagnostics.entries.len()
    );
    assert!(String::from_utf8(loaded.input.artifacts[0].bytes.bytes.clone())?.contains("mermaid"));
    Ok(())
}

#[test]
fn loader_materializes_visible_direct_image_asset() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/direct/kdv-icon.png")
        .canonicalize()?;
    let output = output_for_document(&file_uri_for_path(&path), &path.display().to_string())?;

    let (loaded, report) = PreviewAssetLoader::new(FakeDiagramEngine)
        .load_requested(&output, &KdvThemeSnapshot::katana_light())?;

    assert_eq!(1, report.loaded_artifact_count);
    assert_eq!(0, report.failed_artifact_count);
    assert_eq!(
        ArtifactFormat::Png,
        loaded.input.artifacts[0].manifest.format
    );
    assert!(!loaded.input.artifacts[0].bytes.bytes.is_empty());
    Ok(())
}

#[test]
fn loader_materializes_encoded_direct_image_file_uri_asset()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_image_path("kdv encoded icon.png")?;
    let bytes = b"not-a-real-png-but-an-asset";
    std::fs::write(&path, bytes)?;
    let uri = file_uri_for_path(&path).replace(' ', "%20");
    let output = output_for_document("", &uri)?;

    let (loaded, report) = PreviewAssetLoader::new(FakeDiagramEngine)
        .load_requested(&output, &KdvThemeSnapshot::katana_light())?;

    assert_eq!(1, report.loaded_artifact_count);
    assert_eq!(0, report.failed_artifact_count);
    assert_eq!(
        ArtifactFormat::Png,
        loaded.input.artifacts[0].manifest.format
    );
    assert_eq!(bytes.to_vec(), loaded.input.artifacts[0].bytes.bytes);
    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn loader_records_missing_direct_image_as_error_artifact() -> Result<(), Box<dyn std::error::Error>>
{
    let output = output_for_document(
        "file:///tmp/kdv-missing-image.png",
        "/tmp/kdv-missing-image.png",
    )?;

    let (loaded, report) = PreviewAssetLoader::new(FakeDiagramEngine)
        .load_requested(&output, &KdvThemeSnapshot::katana_light())?;

    assert_eq!(0, report.loaded_artifact_count);
    assert_eq!(1, report.failed_artifact_count);
    assert_eq!(1, loaded.input.artifacts.len());
    assert_eq!(
        ArtifactFormat::Png,
        loaded.input.artifacts[0].manifest.format
    );
    assert!(
        loaded.input.artifacts[0]
            .manifest
            .diagnostics
            .entries
            .iter()
            .any(|entry| entry.message.contains("asset file read failed"))
    );
    Ok(())
}

#[test]
fn loader_materializes_visible_math_asset() -> Result<(), Box<dyn std::error::Error>> {
    let output = output_for("```math\nE = mc^2\n```")?;

    let (loaded, report) = PreviewAssetLoader::new(FakeDiagramEngine)
        .load_requested(&output, &KdvThemeSnapshot::katana_light())?;

    assert_eq!(1, report.loaded_artifact_count);
    assert_eq!(0, report.failed_artifact_count);
    assert_eq!(
        ArtifactFormat::Svg,
        loaded.input.artifacts[0].manifest.format
    );
    assert!(loaded.input.artifacts[0].bytes.bytes.starts_with(b"<svg"));
    Ok(())
}

#[test]
fn loader_records_math_error_artifact_for_empty_source() -> Result<(), Box<dyn std::error::Error>> {
    let output = output_for("```math\n\n```")?;

    let (_loaded, report) = PreviewAssetLoader::new(FakeDiagramEngine)
        .load_requested(&output, &KdvThemeSnapshot::katana_light())?;

    assert_eq!(0, report.loaded_artifact_count);
    assert_eq!(1, report.failed_artifact_count);
    Ok(())
}

#[test]
fn loader_skips_html_requests_by_design() -> Result<(), Box<dyn std::error::Error>> {
    let output = output_for(r#"<p align="center">Centered HTML</p>"#)?;
    let plan = ViewerNodePlanner::create(&output.input, output.scroll_offset);
    assert!(
        plan.asset_requests
            .iter()
            .any(|request| request.format == ArtifactFormat::Html),
        "HTML viewer node must still expose an HTML asset request"
    );

    let (loaded, report) = PreviewAssetLoader::new(FakeDiagramEngine)
        .load_requested(&output, &KdvThemeSnapshot::katana_light())?;

    assert_eq!(0, report.loaded_artifact_count);
    assert_eq!(0, report.failed_artifact_count);
    assert!(loaded.input.artifacts.is_empty());
    Ok(())
}

#[test]
fn loader_supports_default_clone_and_cache_root() -> Result<(), Box<dyn std::error::Error>> {
    let loader = PreviewAssetLoader::<KrrDiagramRenderEngine>::default();

    assert!(loader.diagram_cache_root().is_none());

    let loader = PreviewAssetLoader::<KrrDiagramRenderEngine>::krr();
    assert!(loader.diagram_cache_root().is_none());

    let custom =
        PreviewAssetLoader::new(7u8).with_diagram_cache_root("/tmp/kdv-asset-loader-cache-test");
    assert_eq!(
        Some(std::path::Path::new("/tmp/kdv-asset-loader-cache-test")),
        custom.diagram_cache_root()
    );

    Ok(())
}

fn file_uri_for_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let local = normalized.strip_prefix("//?/").unwrap_or(&normalized);
    let prefix = if local.starts_with('/') { "" } else { "/" };
    format!("file://{prefix}{local}")
}
