use std::path::{Path, PathBuf};

const FONT_FILES: [(&str, &[u8]); 5] = [
    (
        "Carlito-Regular.ttf",
        include_bytes!("../../assets/fonts/Carlito-Regular.ttf"),
    ),
    (
        "Carlito-Bold.ttf",
        include_bytes!("../../assets/fonts/Carlito-Bold.ttf"),
    ),
    (
        "Carlito-Italic.ttf",
        include_bytes!("../../assets/fonts/Carlito-Italic.ttf"),
    ),
    (
        "Carlito-BoldItalic.ttf",
        include_bytes!("../../assets/fonts/Carlito-BoldItalic.ttf"),
    ),
    (
        "NotoSansJP-VariableFont_wght.ttf",
        include_bytes!("../../assets/fonts/NotoSansJP-VariableFont_wght.ttf"),
    ),
];

pub(super) fn stage_deterministic_fonts(workspace: &Path) -> Result<PathBuf, (String, String)> {
    let font_path = workspace.join("fonts");
    std::fs::create_dir_all(&font_path).map_err(font_failure)?;
    for (name, bytes) in FONT_FILES {
        std::fs::write(font_path.join(name), bytes).map_err(font_failure)?;
    }
    Ok(font_path)
}

fn font_failure(error: std::io::Error) -> (String, String) {
    ("font_setup".to_owned(), error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{FONT_FILES, font_failure, stage_deterministic_fonts};

    #[test]
    fn stages_every_trusted_font_inside_the_worker_workspace() -> Result<(), (String, String)> {
        let workspace = tempfile::tempdir().map_err(font_failure)?;
        let font_path = stage_deterministic_fonts(workspace.path())?;
        assert_eq!(workspace.path().join("fonts"), font_path);
        for (name, expected) in FONT_FILES {
            let actual = std::fs::read(font_path.join(name)).map_err(font_failure)?;
            assert_eq!(expected, actual);
        }
        Ok(())
    }

    #[test]
    fn reports_a_typed_stage_when_the_workspace_is_not_a_directory()
    -> Result<(), Box<dyn std::error::Error>> {
        let workspace = tempfile::NamedTempFile::new()?;
        assert!(matches!(
            stage_deterministic_fonts(workspace.path()),
            Err((stage, message)) if stage == "font_setup" && !message.is_empty()
        ));
        Ok(())
    }
}
