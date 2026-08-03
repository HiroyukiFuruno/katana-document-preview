#[cfg(test)]
use super::*;

const WORKSPACE_FIXTURE_NAME: &str = "kdv-linter-workspace";
#[cfg(unix)]
const READ_PERMISSION: u32 = 0o755;
#[cfg(unix)]
const NO_PERMISSION: u32 = 0o000;

#[test]
fn load_uses_expected_target_roots() -> Result<(), KdvLintError> {
    let root = workspace_fixture("load");
    let fixture = root.join("crates/katana-document-viewer/src");
    std::fs::create_dir_all(&fixture).map_err(|source| KdvLintError::Read {
        path: fixture.clone(),
        source,
    })?;
    std::fs::write(fixture.join("item.rs"), "pub fn ok() {}").map_err(|source| {
        KdvLintError::Read {
            path: fixture.join("item.rs"),
            source,
        }
    })?;

    let workspace = WorkspaceModel::load(&root)?;
    assert!(!workspace.rust_files().is_empty());

    std::fs::remove_dir_all(&root).map_err(|source| KdvLintError::Read {
        path: root.clone(),
        source,
    })?;
    Ok(())
}

#[test]
fn is_rust_file_only_accepts_rs_extension() {
    assert!(WorkspaceModel::is_rust_file(Path::new("src/lib.rs")));
    assert!(!WorkspaceModel::is_rust_file(Path::new("src/main.txt")));
}

#[test]
fn parse_file_reports_read_error_for_invalid_syntax() -> Result<(), KdvLintError> {
    let (root, file) = invalid_syntax_fixture()?;

    let message = match WorkspaceModel::load(&root) {
        Ok(_) => {
            return Err(KdvLintError::RustParse {
                path: file.clone(),
                line: 1,
                column: 1,
                message: "expected parse failure".to_string(),
            });
        }
        Err(error) => error.to_string(),
    };

    assert!(message.contains("failed to parse Rust syntax"));
    std::fs::remove_dir_all(&root).map_err(|source| KdvLintError::Read {
        path: root.clone(),
        source,
    })?;
    Ok(())
}

#[test]
fn collect_rs_files_ignores_non_rust_files_and_is_sorted() -> Result<(), KdvLintError> {
    let root = workspace_fixture("collect");
    write_collect_fixture(&root)?;
    let files = WorkspaceModel::collect_rs_files(&root)?;
    assert_eq!(files.len(), 2);
    assert_eq!(sorted_file_names(&files), vec!["a.rs", "b.rs"]);

    std::fs::remove_dir_all(&root).map_err(|source| KdvLintError::Read {
        path: root.clone(),
        source,
    })?;
    Ok(())
}

#[test]
fn parse_file_reports_read_error() {
    let path = workspace_fixture("missing-file").join("missing.rs");
    let message = match WorkspaceModel::parse_file(&path) {
        Ok(_) => None,
        Err(error) => Some(error.to_string()),
    };
    assert!(message.is_some_and(|it| it.contains("failed to read")));
}

#[test]
fn workspace_target_roots_is_stable() {
    let root = workspace_fixture("target-roots");
    let targets = WorkspaceModel::target_roots(&root);

    assert_eq!(targets.len(), 3);
    assert!(
        targets
            .first()
            .is_some_and(|path| path.ends_with("crates/kdv-linter/src"))
    );
    assert!(
        targets
            .get(2)
            .is_some_and(|path| path.ends_with("crates/katana-document-viewer/src"))
    );
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn collect_rs_files_ignores_unreadable_entries() -> Result<(), KdvLintError> {
    use std::os::unix::fs::PermissionsExt;
    let root = workspace_fixture("collect-errors");
    let inaccessible = root.join("inaccessible");
    std::fs::create_dir_all(&inaccessible).map_err(|source| KdvLintError::Read {
        path: inaccessible.clone(),
        source,
    })?;
    std::fs::set_permissions(&inaccessible, PermissionsExt::from_mode(NO_PERMISSION)).map_err(
        |source| KdvLintError::Read {
            path: root.clone(),
            source,
        },
    )?;

    let files = WorkspaceModel::collect_rs_files(&root)?;
    assert!(files.is_empty());

    std::fs::set_permissions(&inaccessible, PermissionsExt::from_mode(READ_PERMISSION)).map_err(
        |source| KdvLintError::Read {
            path: root.clone(),
            source,
        },
    )?;
    std::fs::remove_dir_all(&root).map_err(|source| KdvLintError::Read {
        path: root.clone(),
        source,
    })?;
    Ok(())
}

fn workspace_fixture(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{WORKSPACE_FIXTURE_NAME}-{name}"))
}

fn invalid_syntax_fixture() -> Result<(PathBuf, PathBuf), KdvLintError> {
    let root = workspace_fixture("invalid");
    let file = root.join("crates/kdv-linter/src/broken.rs");
    std::fs::create_dir_all(&root).map_err(|source| KdvLintError::Read {
        path: root.clone(),
        source,
    })?;
    let parent = file.parent().ok_or_else(|| KdvLintError::Read {
        path: file.clone(),
        source: std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "fixture file must have parent",
        ),
    })?;
    std::fs::create_dir_all(parent).map_err(|source| KdvLintError::Read {
        path: parent.to_path_buf(),
        source,
    })?;
    std::fs::write(&file, "fn ) {").map_err(|source| KdvLintError::Read {
        path: file.clone(),
        source,
    })?;
    Ok((root, file))
}

fn write_collect_fixture(root: &Path) -> Result<(), KdvLintError> {
    let source = root.join("src");
    std::fs::create_dir_all(&source).map_err(|error| KdvLintError::Read {
        path: source.clone(),
        source: error,
    })?;
    write_text(&root.join("src/b.rs"), "fn b() {}")?;
    write_text(&root.join("src/a.rs"), "fn a() {}")?;
    write_text(&root.join("src/readme.txt"), "skip")?;
    Ok(())
}

fn write_text(path: &Path, body: &str) -> Result<(), KdvLintError> {
    std::fs::write(path, body).map_err(|error| KdvLintError::Read {
        path: path.to_path_buf(),
        source: error,
    })
}

fn sorted_file_names(files: &[SourceFile]) -> Vec<&str> {
    files
        .iter()
        .filter_map(|item| item.path().file_name())
        .filter_map(std::ffi::OsStr::to_str)
        .collect()
}

#[path = "tests_kuc_core.rs"]
mod tests_kuc_core;
