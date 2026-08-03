use crate::KucViewerConfig;
use crate::catalog::StorybookFixture;
use katana_document_viewer::KdvThemeSnapshot;
use katana_document_viewer::{
    DiagramViewportState, MarkdownSource, PreviewConfig, PreviewTheme, ViewerInteractionConfig,
    ViewerMode, ViewerSearchState, ViewerTaskState, ViewerTypographyConfig, ViewerViewport,
};
use katana_ui_core::render_model::{UiNode, UiNodeKind};
use katana_ui_core::theme::ThemeSnapshot;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

const STORYBOOK_WINDOW_ID: &str = "kdv-storybook";
const DOCUMENT_LINE_HEIGHT_RATIO: f32 = 23.0 / 14.0;

pub(crate) struct PreviewBuildSupport;

#[derive(Default)]
pub(crate) struct KucConfigState {
    pub(crate) diagram_viewports: BTreeMap<String, DiagramViewportState>,
    pub(crate) image_viewports: BTreeMap<String, DiagramViewportState>,
    pub(crate) task_state_overrides: BTreeMap<String, ViewerTaskState>,
    pub(crate) accordion_open_overrides: BTreeMap<String, bool>,
    pub(crate) copied_code_node_ids: BTreeSet<String>,
}

impl PreviewBuildSupport {
    pub(crate) fn count_image_surfaces(node: &UiNode) -> usize {
        usize::from(node.kind() == UiNodeKind::ImageSurface)
            + node
                .children()
                .iter()
                .map(Self::count_image_surfaces)
                .sum::<usize>()
    }

    pub(crate) fn source_for_fixture(
        fixture: &StorybookFixture,
    ) -> Result<MarkdownSource, Box<dyn std::error::Error>> {
        let source_path = fixture.path.canonicalize()?;
        let document_id = Self::document_id_for_path(&source_path);
        let content = if Self::is_image_fixture(&source_path) {
            Self::file_uri_for_document_id(&document_id)
        } else {
            std::fs::read_to_string(&source_path)?
        };
        Ok(MarkdownSource {
            content,
            document_id: Some(document_id),
        })
    }

    pub(crate) fn preview_config(
        viewport: ViewerViewport,
        scroll_y: f32,
        dark: bool,
        interaction: ViewerInteractionConfig,
        mode: ViewerMode,
        typography: ViewerTypographyConfig,
        search: ViewerSearchState,
    ) -> PreviewConfig {
        Self::preview_config_for_theme(
            viewport,
            scroll_y,
            Self::kdv_theme(dark),
            interaction,
            mode,
            typography,
            search,
        )
    }

    pub(crate) fn preview_config_for_theme(
        viewport: ViewerViewport,
        scroll_y: f32,
        theme: KdvThemeSnapshot,
        interaction: ViewerInteractionConfig,
        mode: ViewerMode,
        typography: ViewerTypographyConfig,
        search: ViewerSearchState,
    ) -> PreviewConfig {
        let interaction = Self::effective_interaction_for_mode(interaction, &mode);
        PreviewConfig {
            theme: PreviewTheme {
                name: theme.name.clone(),
                fingerprint: Self::theme_fingerprint(&theme),
            },
            mode,
            interaction,
            base_font_size: Some(f32::from(typography.preview_font_size)),
            line_height: Some(f32::from(typography.preview_font_size) * DOCUMENT_LINE_HEIGHT_RATIO),
            search,
            viewport,
            scroll_offset: scroll_y.max(0.0),
        }
    }

    fn effective_interaction_for_mode(
        mut interaction: ViewerInteractionConfig,
        mode: &ViewerMode,
    ) -> ViewerInteractionConfig {
        if *mode == ViewerMode::Slideshow {
            interaction.code_controls_enabled = false;
        }
        interaction
    }

    pub(crate) fn kuc_config(
        config: &PreviewConfig,
        theme: ThemeSnapshot,
        typography: ViewerTypographyConfig,
        state: KucConfigState,
    ) -> Result<KucViewerConfig, String> {
        Ok(KucViewerConfig::new(STORYBOOK_WINDOW_ID, config.viewport)
            .theme(Self::with_document_typography(theme, typography))
            .interaction(config.interaction.clone())
            .diagram_viewports(state.diagram_viewports)
            .image_viewports(state.image_viewports)
            .task_state_overrides(state.task_state_overrides)
            .accordion_open_overrides(state.accordion_open_overrides)
            .copied_code_node_ids(state.copied_code_node_ids))
    }

    pub(crate) fn with_document_typography(
        mut theme: ThemeSnapshot,
        typography: ViewerTypographyConfig,
    ) -> ThemeSnapshot {
        set_font_size(
            &mut theme,
            "document-body",
            f32::from(typography.preview_font_size),
            katana_ui_core::theme::FontFamily::Proportional,
        );
        set_font_size(
            &mut theme,
            "document-export-body",
            f32::from(typography.preview_font_size),
            katana_ui_core::theme::FontFamily::Proportional,
        );
        set_font_size(
            &mut theme,
            "document-code",
            f32::from(typography.preview_font_size.saturating_sub(2).max(10)),
            katana_ui_core::theme::FontFamily::Monospace,
        );
        set_font_size(
            &mut theme,
            "code",
            f32::from(typography.preview_font_size.saturating_sub(2).max(10)),
            katana_ui_core::theme::FontFamily::Monospace,
        );
        theme
    }

    pub(crate) fn kdv_theme(dark: bool) -> KdvThemeSnapshot {
        let mut theme = if dark {
            KdvThemeSnapshot::katana_dark()
        } else {
            KdvThemeSnapshot::katana_light()
        };
        theme.diagram_background = theme.background.clone();
        theme.diagram_text = theme.text.clone();
        theme
    }

    fn theme_fingerprint(theme: &KdvThemeSnapshot) -> String {
        let mut fingerprint = String::new();
        let _ = write!(
            fingerprint,
            "name={};mode={:?};background={};text={};diagram_background={};diagram_text={};diagram_fill={};diagram_stroke={};diagram_arrow={};mermaid={};syntax_dark={};syntax_light={}",
            theme.name,
            theme.mode,
            theme.background,
            theme.text,
            theme.diagram_background,
            theme.diagram_text,
            theme.diagram_fill,
            theme.diagram_stroke,
            theme.diagram_arrow,
            theme.mermaid_theme,
            theme.syntax_theme_dark,
            theme.syntax_theme_light
        );
        fingerprint
    }

    fn is_image_fixture(path: &Path) -> bool {
        path.extension()
            .and_then(std::ffi::OsStr::to_str)
            .map(str::to_ascii_lowercase)
            .is_some_and(|extension| {
                matches!(
                    extension.as_str(),
                    "bmp" | "gif" | "jpeg" | "jpg" | "png" | "svg" | "webp"
                )
            })
    }

    fn document_id_for_path(path: &Path) -> String {
        let normalized = path.display().to_string().replace('\\', "/");
        normalized
            .strip_prefix("//?/")
            .unwrap_or(&normalized)
            .to_owned()
    }

    fn file_uri_for_document_id(document_id: &str) -> String {
        if document_id.starts_with("file://") {
            return document_id.to_string();
        }
        if document_id.starts_with('/') {
            return format!("file://{document_id}");
        }
        if Self::starts_with_windows_drive(document_id) {
            return format!("file:///{document_id}");
        }
        format!("file://{document_id}")
    }

    fn starts_with_windows_drive(value: &str) -> bool {
        let bytes = value.as_bytes();
        bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'/' | b'\\')
    }
}

fn set_font_size(
    theme: &mut ThemeSnapshot,
    name: &str,
    size: f32,
    family: katana_ui_core::theme::FontFamily,
) {
    if let Some(font) = theme.fonts.iter_mut().find(|font| font.name == name) {
        font.size = size;
        return;
    }
    theme.fonts.push(katana_ui_core::theme::FontToken {
        name: name.to_string(),
        family,
        size,
        weight: 400,
    });
}

#[cfg(test)]
mod tests {
    use super::{DOCUMENT_LINE_HEIGHT_RATIO, PreviewBuildSupport};
    use katana_document_viewer::{
        ViewerInteractionConfig, ViewerMode, ViewerSearchState, ViewerTypographyConfig,
        ViewerViewport,
    };

    #[test]
    fn preview_config_uses_kuc_document_line_height_ratio() {
        let typography = ViewerTypographyConfig {
            preview_font_size: 14,
        };

        let config = PreviewBuildSupport::preview_config(
            ViewerViewport {
                width: 640.0,
                height: 320.0,
            },
            0.0,
            false,
            ViewerInteractionConfig::default(),
            ViewerMode::Document,
            typography,
            ViewerSearchState::default(),
        );

        assert_eq!(Some(14.0 * DOCUMENT_LINE_HEIGHT_RATIO), config.line_height);
    }

    #[test]
    fn preview_config_hides_code_copy_controls_in_slideshow_like_katana() {
        let interaction = ViewerInteractionConfig {
            code_controls_enabled: true,
            ..Default::default()
        };

        let config = PreviewBuildSupport::preview_config(
            ViewerViewport {
                width: 640.0,
                height: 320.0,
            },
            0.0,
            false,
            interaction,
            ViewerMode::Slideshow,
            ViewerTypographyConfig::default(),
            ViewerSearchState::default(),
        );

        assert!(!config.interaction.code_controls_enabled);
    }

    #[test]
    fn preview_config_keeps_code_copy_controls_in_document_mode() {
        let interaction = ViewerInteractionConfig {
            code_controls_enabled: true,
            ..Default::default()
        };

        let config = PreviewBuildSupport::preview_config(
            ViewerViewport {
                width: 640.0,
                height: 320.0,
            },
            0.0,
            false,
            interaction,
            ViewerMode::Document,
            ViewerTypographyConfig::default(),
            ViewerSearchState::default(),
        );

        assert!(config.interaction.code_controls_enabled);
    }

    #[test]
    fn storybook_dark_theme_uses_katana_viewer_preview_tokens_for_diagrams() {
        let theme = PreviewBuildSupport::kdv_theme(true);

        assert_eq!(theme.background, "#1e1e1e");
        assert_eq!(theme.text, "#d4d4d4");
        assert_eq!(
            theme.background, theme.diagram_background,
            "storybook diagram render should use KatanA viewer preview background"
        );
        assert_eq!(
            theme.text, theme.diagram_text,
            "storybook diagram render should use KatanA viewer preview text"
        );
    }

    #[test]
    fn preview_config_theme_fingerprint_tracks_diagram_render_tokens() {
        let viewport = ViewerViewport {
            width: 640.0,
            height: 320.0,
        };
        let dark = PreviewBuildSupport::kdv_theme(true);
        let mut changed = dark.clone();
        changed.diagram_text = "#ffffff".to_string();

        let baseline = PreviewBuildSupport::preview_config_for_theme(
            viewport,
            0.0,
            dark.clone(),
            ViewerInteractionConfig::default(),
            ViewerMode::Document,
            ViewerTypographyConfig::default(),
            ViewerSearchState::default(),
        );
        let changed = PreviewBuildSupport::preview_config_for_theme(
            viewport,
            0.0,
            changed,
            ViewerInteractionConfig::default(),
            ViewerMode::Document,
            ViewerTypographyConfig::default(),
            ViewerSearchState::default(),
        );

        assert_ne!(baseline.theme.fingerprint, changed.theme.fingerprint);
        assert!(baseline.theme.fingerprint.contains(&dark.diagram_text));
        assert!(
            baseline
                .theme
                .fingerprint
                .contains(&dark.diagram_background)
        );
    }

    #[test]
    fn direct_image_fixture_source_uses_absolute_file_uri() -> Result<(), Box<dyn std::error::Error>>
    {
        let path = std::env::temp_dir().join("kdv storybook direct image.png");
        std::fs::write(&path, b"image bytes")?;
        let expected_path = path
            .canonicalize()?
            .display()
            .to_string()
            .replace('\\', "/");
        let source = PreviewBuildSupport::source_for_fixture(&crate::catalog::StorybookFixture {
            label: "direct/kdv-icon.png".to_string(),
            path: path.clone(),
        })?;

        assert_eq!(Some(expected_path.clone()), source.document_id);
        assert_eq!(
            PreviewBuildSupport::file_uri_for_document_id(&expected_path),
            source.content
        );
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn relative_direct_image_fixture_source_uses_absolute_file_uri()
    -> Result<(), Box<dyn std::error::Error>> {
        let relative_path =
            std::path::PathBuf::from("target/kdv-storybook-source-tests/kdv relative icon.png");
        if let Some(parent) = relative_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&relative_path, b"image bytes")?;
        let absolute_path = relative_path
            .canonicalize()?
            .display()
            .to_string()
            .replace('\\', "/");
        let source = PreviewBuildSupport::source_for_fixture(&crate::catalog::StorybookFixture {
            label: "direct/kdv-icon.png".to_string(),
            path: relative_path.clone(),
        })?;

        assert!(source.content.starts_with("file:///"));
        assert_eq!(Some(absolute_path), source.document_id);
        let _ = std::fs::remove_file(relative_path);
        Ok(())
    }

    #[test]
    fn windows_direct_image_fixture_source_uses_valid_file_uri() {
        let document_id = "D:/a/katana-document-viewer/assets/fixtures/direct/kdv-icon.png";

        assert_eq!(
            "file:///D:/a/katana-document-viewer/assets/fixtures/direct/kdv-icon.png",
            PreviewBuildSupport::file_uri_for_document_id(document_id)
        );
    }

    #[test]
    fn windows_extended_drive_path_becomes_a_regular_document_id() {
        let path = std::path::Path::new(r"\\?\D:\a\katana-document-viewer\index.html");

        assert_eq!(
            "D:/a/katana-document-viewer/index.html",
            PreviewBuildSupport::document_id_for_path(path)
        );
    }
}
