use super::test_support::{FakeDiagramEngine, output_for_document};
use crate::{ArtifactFormat, KdvThemeSnapshot, PreviewAssetLoader};
use std::path::Path;

#[test]
fn loader_materializes_relative_direct_image_asset_from_document_directory()
-> Result<(), Box<dyn std::error::Error>> {
    let image_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/direct/kdv-icon.png")
        .canonicalize()?;
    let document_path = image_path
        .parent()
        .ok_or("direct image fixture directory missing")?
        .join("relative-direct-image.png");
    let output = output_for_document("kdv-icon.png", &document_path.display().to_string())?;

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
