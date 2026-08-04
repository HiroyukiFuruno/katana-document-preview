use super::PortablePath;
use std::path::Path;

#[test]
fn path_matching_is_separator_independent() {
    let windows = Path::new(r"C:\repo\crates\katana-document-viewer\src\viewer\media_action.rs");
    let unix = Path::new("/repo/crates/katana-document-viewer/src/viewer/media_action.rs");

    assert!(PortablePath::new(windows).contains("crates/katana-document-viewer/src/viewer/"));
    assert!(PortablePath::new(unix).contains(r"crates\katana-document-viewer\src\viewer\"));
    assert!(PortablePath::new(windows).ends_with("viewer/media_action.rs"));
    assert!(PortablePath::new(unix).ends_with(r"viewer\media_action.rs"));
    assert!(!PortablePath::new(windows).contains("document_surface"));
    assert!(!PortablePath::new(unix).ends_with("media_control_spec.rs"));
}
