use crate::canvas::Canvas;
use crate::layout::StorybookPreviewArea;
use crate::preview::PreviewScene;
use crate::preview_theme_bridge::KucThemeBridge;
use katana_document_viewer::KdvThemeSnapshot;

const DARK_PREVIEW_BACKGROUND: u32 = 0x151515;
const LIGHT_PREVIEW_BACKGROUND: u32 = 0xffffff;

pub struct StorybookSmokeAssertions;

impl StorybookSmokeAssertions {
    pub fn assert_fixture_visible(
        label: &str,
        scene: &PreviewScene,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if scene.node_count == 0 && scene.image_surface_count == 0 {
            return Err(format!("fixture produced no viewer content: {label}").into());
        }
        if is_direct_text_fixture(label) && scene.node_count == 0 {
            return Err(format!("text fixture did not reach KUC node plan: {label}").into());
        }
        if !is_direct_media_fixture(label) {
            return Ok(());
        }
        if scene.image_surface_count == 0 {
            return Err(format!("media fixture did not reach KUC ImageSurface: {label}").into());
        }
        if scene.failed_asset_count > 0 {
            return Err(format!("media fixture has failed assets: {label}").into());
        }
        Ok(())
    }

    pub fn assert_bottom_tail_space(
        label: &str,
        canvas: &Canvas,
        dark: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme_background = theme_background_rgb(dark)?;
        let ratio = tail_background_ratio(canvas, theme_background, dark);
        if ratio <= 90 {
            return Err(format!(
                "bottom scroll did not expose viewer tail space: {label} ratio={ratio}"
            )
            .into());
        }
        Ok(())
    }
}

fn is_direct_text_fixture(label: &str) -> bool {
    matches!(
        extension(label),
        Some("htm" | "html" | "markdown" | "md" | "txt")
    )
}

fn is_direct_media_fixture(label: &str) -> bool {
    matches!(
        extension(label),
        Some(
            "bmp"
                | "drawio"
                | "drowio"
                | "gif"
                | "jpeg"
                | "jpg"
                | "mermaid"
                | "mmd"
                | "plantuml"
                | "png"
                | "puml"
                | "svg"
                | "webp"
        )
    )
}

fn extension(label: &str) -> Option<&str> {
    label.rsplit_once('.').map(|(_, extension)| extension)
}

fn tail_background_ratio(canvas: &Canvas, theme_background: u32, dark: bool) -> usize {
    let area = StorybookPreviewArea::for_window(canvas.width(), canvas.height(), 0.0);
    let mut background = 0usize;
    let mut total = 0usize;
    for y in area.y + area.height / 2..area.y + area.height {
        for x in area.x..area.x + area.width {
            total += 1;
            let pixel = canvas.pixels()[y * canvas.width() + x];
            if is_tail_pixel(pixel, theme_background, dark) {
                background += 1;
            }
        }
    }
    if total == 0 {
        return 0;
    }
    background * 100 / total
}

fn is_tail_pixel(pixel: u32, theme_background: u32, dark: bool) -> bool {
    let preview_background = if dark {
        DARK_PREVIEW_BACKGROUND
    } else {
        LIGHT_PREVIEW_BACKGROUND
    };
    pixel == preview_background || pixel == theme_background
}

fn theme_background_rgb(dark: bool) -> Result<u32, Box<dyn std::error::Error>> {
    let kdv_theme = if dark {
        KdvThemeSnapshot::katana_dark()
    } else {
        KdvThemeSnapshot::katana_light()
    };
    let theme = KucThemeBridge::from_kdv(&kdv_theme)?;
    let rgba = theme
        .color("background")
        .ok_or("missing KUC theme color token: background")?;
    Ok(((rgba[0] as u32) << 16) | ((rgba[1] as u32) << 8) | rgba[2] as u32)
}

#[cfg(test)]
mod tests {
    use super::StorybookSmokeAssertions;
    use crate::preview::PreviewScene;
    use katana_document_viewer::ViewerMode;
    use katana_ui_core::atom::Text;
    use katana_ui_core::render_model::UiTree;
    use katana_ui_core::theme::ThemeSnapshot;

    #[test]
    fn smoke_requires_markdown_to_reach_node_plan() {
        let scene = scene_with_counts(0, 0, 0);

        let result =
            StorybookSmokeAssertions::assert_fixture_visible("direct/sample.markdown", &scene);

        assert!(result.is_err());
    }

    #[test]
    fn smoke_requires_media_to_reach_image_surface() {
        let scene = scene_with_counts(1, 0, 0);

        let result = StorybookSmokeAssertions::assert_fixture_visible("direct/sample.png", &scene);

        assert!(result.is_err());
    }

    fn scene_with_counts(nodes: usize, images: usize, failed: usize) -> PreviewScene {
        PreviewScene {
            document_id: "test.md".to_string(),
            tree: UiTree::new(Text::new("scene")),
            theme: ThemeSnapshot::light(),
            host_action_cache: Default::default(),
            node_count: nodes,
            mode: ViewerMode::Document,
            typography: Default::default(),
            asset_request_count: 0,
            asset_request_key: String::new(),
            loaded_asset_count: images,
            failed_asset_count: failed,
            image_surface_count: images,
            surface: None,
            content_height: 0.0,
            scroll_redraw_sensitive_rects: Vec::new(),
            scroll_redraw_diagram_boundary_rects: Vec::new(),
            slideshow_current_page: 0,
            slideshow_max_page: 0,
            diagram_viewports: Default::default(),
            diagram_node_ids: Default::default(),
            search_targets: Vec::new(),
            targets: Vec::new(),
            target_lookup: Default::default(),
            internal_anchor_lookup: Default::default(),
            warnings: Vec::new(),
        }
    }
}
