use super::*;
use std::path::PathBuf;

#[test]
fn contains_known_file_length_debt() {
    let root = Path::new("/repo");
    let violation = Violation::new(
        PathBuf::from("/repo/crates/katana-document-viewer/src/viewer/state.rs"),
        1,
        1,
        "file-length",
        "file has 201 lines",
    );

    assert!(LengthBaseline::contains(root, &violation));
}

#[test]
fn rejects_unlisted_file_length_debt() {
    let root = Path::new("/repo");
    let violation = Violation::new(
        PathBuf::from("/repo/crates/katana-document-viewer/src/new_large.rs"),
        1,
        1,
        "file-length",
        "file has 201 lines",
    );

    assert!(!LengthBaseline::contains(root, &violation));
}

#[test]
fn contains_known_function_length_debt() {
    let root = Path::new("/repo");
    let violation = Violation::new(
        PathBuf::from("/repo/crates/katana-document-viewer/src/viewer/media_control_spec.rs"),
        183,
        10,
        "function-length",
        "function `surface_control_svg` has 40 lines",
    );

    assert!(LengthBaseline::contains(root, &violation));
}

#[test]
fn contains_windows_mixed_separator_file_length_debt() {
    let root = Path::new(r"D:\a\kdv\kdv");
    let violation = Violation::new(
        PathBuf::from(r"D:\a\kdv\kdv\crates/katana-document-viewer/src\viewer\state.rs"),
        1,
        1,
        "file-length",
        "file has 201 lines",
    );

    assert!(LengthBaseline::contains(root, &violation));
}

#[test]
fn contains_windows_mixed_separator_function_length_debt() {
    let root = Path::new(r"D:\a\kdv\kdv");
    let violation = Violation::new(
        PathBuf::from(
            r"D:\a\kdv\kdv\crates/katana-document-viewer/src\viewer\media_control_spec.rs",
        ),
        183,
        10,
        "function-length",
        "function `surface_control_svg` has 40 lines",
    );

    assert!(LengthBaseline::contains(root, &violation));
}

#[test]
fn rejects_windows_workspace_prefix_collision() {
    let root = Path::new(r"D:\a\kdv\kdv");
    let violation = Violation::new(
        PathBuf::from(r"D:\a\kdv\kdv-other\crates\katana-document-viewer\src\viewer\state.rs"),
        1,
        1,
        "file-length",
        "file has 201 lines",
    );

    assert!(!LengthBaseline::contains(root, &violation));
}
