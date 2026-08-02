use katana_document_viewer::PdfRenderedPage;
use katana_ui_core::atom::ImageSurface;
use katana_ui_core::render_model::{UiImageSurfaceValidationError, UiNode};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct KucPageSurfaceAdapter;

impl KucPageSurfaceAdapter {
    pub fn adapt(
        label: impl Into<String>,
        rendered: PdfRenderedPage,
    ) -> Result<UiNode, UiImageSurfaceValidationError> {
        let surface = rendered.surface;
        Ok(ImageSurface::from_rgba(
            label,
            surface.fingerprint,
            surface.width,
            surface.height,
            surface.rgba,
        )?
        .content_scale(surface.content_scale)
        .display_size_exact(surface.display_width, surface.display_height)
        .accessibility_label(format!("Page {}", rendered.page_index + 1))
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::KucPageSurfaceAdapter;
    use katana_document_viewer::{PdfRenderedPage, ViewerImageSurface};
    use katana_ui_core::render_model::UiNodeKind;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn rendered_page_becomes_a_kuc_image_surface_without_reinterpreting_pixels() -> TestResult {
        let node = KucPageSurfaceAdapter::adapt("PDF page", rendered_page())?;

        assert_eq!(UiNodeKind::ImageSurface, node.kind());
        let surface = &node.props().image_surface;
        assert_eq!("pdf-page-3", surface.fingerprint);
        assert_eq!((2, 1), (surface.width, surface.height));
        assert_eq!(
            (1_500, 750),
            (surface.display_width_milli, surface.display_height_milli)
        );
        assert_eq!(150, surface.content_scale);
        assert_eq!("Page 3", surface.accessibility_label);
        assert_eq!(8, surface.rgba.len());
        Ok(())
    }

    fn rendered_page() -> PdfRenderedPage {
        PdfRenderedPage {
            page_index: 2,
            scale: 1.5,
            surface: ViewerImageSurface {
                fingerprint: "pdf-page-3".to_owned(),
                width: 2,
                height: 1,
                display_width: 1.5,
                display_height: 0.75,
                content_scale: 150,
                rgba: vec![255, 0, 0, 255, 0, 0, 255, 255],
            },
        }
    }

    #[test]
    fn invalid_kdv_raster_is_reported_by_the_kuc_surface_contract() -> TestResult {
        assert!(matches!(
            KucPageSurfaceAdapter::adapt(
                "Broken page",
                PdfRenderedPage {
                    page_index: 0,
                    scale: 1.0,
                    surface: ViewerImageSurface {
                        fingerprint: String::new(),
                        width: 1,
                        height: 1,
                        display_width: 1.0,
                        display_height: 1.0,
                        content_scale: 100,
                        rgba: vec![0, 0, 0, 255],
                    },
                },
            ),
            Err(katana_ui_core::render_model::UiImageSurfaceValidationError::EmptyFingerprint)
        ));
        Ok(())
    }
}
