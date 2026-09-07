use crate::preview_runtime::asset_loader_support::PreviewAssetLoaderSupport;
use std::path::PathBuf;

#[test]
fn file_path_from_uri_supports_encoded_file_uri() {
    let path =
        PreviewAssetLoaderSupport::file_path_from_uri("file:///tmp/kdv%20icon.png?cache=1#preview");

    assert_eq!(Some(PathBuf::from("/tmp/kdv icon.png")), path);
}

#[test]
fn file_path_from_uri_preserves_encoded_filesystem_delimiters() {
    let path = PreviewAssetLoaderSupport::file_path_from_uri("file:///tmp/a%20%5Bb%5D%23c%25.png");

    assert_eq!(Some(PathBuf::from("/tmp/a [b]#c%.png")), path);
}

#[test]
fn file_path_from_uri_supports_localhost_file_uri() {
    let path = PreviewAssetLoaderSupport::file_path_from_uri("file://localhost/tmp/kdv-icon.png");

    assert_eq!(Some(PathBuf::from("/tmp/kdv-icon.png")), path);
}

#[test]
fn file_path_from_uri_supports_windows_drive_file_uri() {
    let path = PreviewAssetLoaderSupport::file_path_from_uri(
        "file:///C:/Users/runner/AppData/Local/Temp/kdv%20icon.png?cache=1#preview",
    );

    assert_eq!(
        Some(PathBuf::from(
            "C:/Users/runner/AppData/Local/Temp/kdv icon.png"
        )),
        path
    );
}

#[test]
fn file_path_from_uri_rejects_remote_file_host() {
    let path = PreviewAssetLoaderSupport::file_path_from_uri("file://example.com/tmp/kdv-icon.png");

    assert_eq!(None, path);
}

#[test]
fn file_path_from_uri_rejects_invalid_percent_encoding() {
    assert_eq!(
        Some(PathBuf::from("/tmp/%GG-icon.png")),
        PreviewAssetLoaderSupport::file_path_from_uri("file:///tmp/%GG-icon.png")
    );
}

#[test]
fn file_path_from_uri_rejects_invalid_percent_encoded_bytes() {
    assert_eq!(
        None,
        PreviewAssetLoaderSupport::file_path_from_uri("file:///tmp/%80-icon.png")
    );
}

#[test]
fn file_path_from_uri_supports_uppercase_hex_digits() {
    let path = PreviewAssetLoaderSupport::file_path_from_uri("file:///tmp/space%5Fname%20%2Etxt");

    assert_eq!(Some(PathBuf::from("/tmp/space_name .txt")), path);
}

#[test]
fn file_path_from_uri_supports_lowercase_hex_digits() {
    assert_eq!(
        Some(PathBuf::from("/tmp/lower_name.txt")),
        PreviewAssetLoaderSupport::file_path_from_uri("file:///tmp/lower%5fname%2etxt")
    );
}
