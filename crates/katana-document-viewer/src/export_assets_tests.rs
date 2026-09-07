use super::*;

#[test]
fn resolves_relative_image_from_source_markdown_directory() {
    let source_uri = SourceUri("file:///workspace/docs/README.md".to_string());

    let resolved = ExportAssetResolver::resolve_src(&source_uri, "assets/icon.png");

    assert_eq!(resolved, "file:///workspace/docs/assets/icon.png");
}

#[test]
fn resolves_windows_style_relative_image_as_file_url() {
    let source_uri = SourceUri("file:///workspace/docs/README.md".to_string());

    let resolved = ExportAssetResolver::resolve_src(&source_uri, r"assets\icon.png");

    assert_eq!(resolved, "file:///workspace/docs/assets/icon.png");
}

#[test]
fn resolves_relative_image_from_plain_source_uri_directory() {
    let source_uri = SourceUri("docs/README.md".to_string());

    let resolved = ExportAssetResolver::resolve_src(&source_uri, "assets/icon.png");

    assert!(resolved.ends_with("/docs/assets/icon.png"));
    assert!(resolved.starts_with("file://"));
}

#[test]
fn keeps_relative_image_when_source_uri_has_no_base_directory() {
    let source_uri = SourceUri(String::new());

    let resolved = ExportAssetResolver::resolve_src(&source_uri, "assets/icon.png");

    assert_eq!(resolved, "assets/icon.png");
}

#[test]
fn keeps_remote_and_data_images_unchanged() {
    let source_uri = SourceUri("file:///workspace/docs/README.md".to_string());

    assert_eq!(
        ExportAssetResolver::resolve_src(&source_uri, "https://example.com/icon.png"),
        "https://example.com/icon.png"
    );
    assert_eq!(
        ExportAssetResolver::resolve_src(&source_uri, "data:image/png;base64,AA=="),
        "data:image/png;base64,AA=="
    );
}

#[test]
fn resolves_source_file_path_with_absolute_source_as_file_url() {
    let source_uri = SourceUri("file:///workspace/docs/notes.md".to_string());

    let resolved = ExportAssetResolver::resolve_file_path(&source_uri, absolute_asset_path());

    assert_eq!(
        resolved,
        Some(std::path::PathBuf::from(absolute_asset_path()))
    );
}

#[test]
fn resolves_file_url_with_absolute_src() {
    let source_uri = SourceUri("file:///workspace/docs/notes.md".to_string());

    let url = ExportAssetResolver::resolve_file_url(&source_uri, absolute_asset_path());

    assert_eq!(url, Some(absolute_asset_file_url().to_string()));
}

#[cfg(windows)]
#[test]
fn resolves_windows_drive_file_uri_as_local_path() {
    let source_uri = SourceUri("file:///workspace/docs/notes.md".to_string());

    let resolved = ExportAssetResolver::resolve_file_path(
        &source_uri,
        "file:///C:/Users/runner/AppData/Local/Temp/icon.png",
    );

    assert_eq!(
        Some(std::path::PathBuf::from(
            r"C:\Users\runner\AppData\Local\Temp\icon.png"
        )),
        resolved
    );
}

#[test]
fn resolves_absolute_file_uri_as_local_path() -> Result<(), Box<dyn std::error::Error>> {
    let source_uri = SourceUri("file:///workspace/docs/notes.md".to_string());

    let resolved = ExportAssetResolver::resolve_file_path(&source_uri, "file:///tmp/icon.png");

    assert_eq!(
        resolved,
        Some(expected_root_relative_file_uri_path("/tmp/icon.png")?)
    );
    Ok(())
}

#[test]
fn resolves_relative_file_uri_against_current_directory() -> Result<(), Box<dyn std::error::Error>>
{
    let source_uri = SourceUri("file:///workspace/docs/notes.md".to_string());

    let resolved = ExportAssetResolver::resolve_file_path(&source_uri, "file://assets/icon.png");

    assert_eq!(
        resolved,
        Some(std::env::current_dir()?.join("assets/icon.png"))
    );
    Ok(())
}

#[test]
fn resolves_file_uri_with_query_and_fragment_as_local_path()
-> Result<(), Box<dyn std::error::Error>> {
    let source_uri = SourceUri("file:///workspace/docs/notes.md".to_string());

    let resolved =
        ExportAssetResolver::resolve_file_path(&source_uri, "file:///tmp/icon.png?cache=1#preview");

    assert_eq!(
        resolved,
        Some(expected_root_relative_file_uri_path("/tmp/icon.png")?)
    );
    Ok(())
}

#[test]
fn resolves_file_url_for_root_relative_source() {
    let source_uri = SourceUri("file:///README.md".to_string());
    let url = ExportAssetResolver::resolve_file_url(&source_uri, "icon.png");

    assert_eq!(url, Some("file:///icon.png".to_string()));
}

#[test]
fn resolves_file_url_for_windows_style_source_path() {
    let source_uri = SourceUri("file:///workspace/docs/notes.md".to_string());
    let url = ExportAssetResolver::resolve_file_url(&source_uri, r"..\\assets\\icon.png");

    assert_eq!(
        url,
        Some("file:///workspace/docs/..//assets//icon.png".to_string())
    );
}

#[test]
fn file_url_normalizes_windows_backslashes() {
    let path = std::path::Path::new(r"c:\tmp\assets\icon.png");
    let normalized = ExportAssetResolver::file_url(path);

    assert_eq!(normalized, "file:///c:/tmp/assets/icon.png");
}

#[cfg(unix)]
fn absolute_asset_path() -> &'static str {
    "/tmp/assets/icon.png"
}

#[cfg(windows)]
fn absolute_asset_path() -> &'static str {
    r"C:\tmp\assets\icon.png"
}

#[cfg(unix)]
fn absolute_asset_file_url() -> &'static str {
    "file:///tmp/assets/icon.png"
}

#[cfg(windows)]
fn absolute_asset_file_url() -> &'static str {
    "file:///C:/tmp/assets/icon.png"
}

#[cfg(unix)]
fn expected_root_relative_file_uri_path(value: &str) -> std::io::Result<std::path::PathBuf> {
    Ok(std::path::PathBuf::from(value))
}

#[cfg(windows)]
fn expected_root_relative_file_uri_path(value: &str) -> std::io::Result<std::path::PathBuf> {
    Ok(std::env::current_dir()?.join(std::path::Path::new(value)))
}
