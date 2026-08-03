use super::{DocumentSurfaceError, DocumentSurfaceFrame};
use crate::PdfRenderedPage;
use katana_ui_core::atom::ImageSurface;

impl DocumentSurfaceFrame {
    pub fn from_rendered_page(
        label: impl Into<String>,
        rendered: PdfRenderedPage,
    ) -> Result<Self, DocumentSurfaceError> {
        let surface = rendered.surface;
        let node = ImageSurface::from_rgba(
            label,
            surface.fingerprint,
            surface.width,
            surface.height,
            surface.rgba,
        )
        .map_err(|error| DocumentSurfaceError::InvalidPage {
            detail: format!("{error:?}"),
        })?
        .content_scale(surface.content_scale)
        .display_size_exact(surface.display_width, surface.display_height)
        .accessibility_label(format!("Page {}", rendered.page_index + 1))
        .into();
        Ok(Self::from_node(node))
    }
}

#[cfg(test)]
mod tests {
    use super::DocumentSurfaceFrame;
    use crate::{DocumentSurfaceError, DocumentSurfaceKind, PdfRenderedPage, ViewerImageSurface};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn rendered_page_becomes_a_document_surface_without_reinterpreting_pixels() -> TestResult {
        let frame = DocumentSurfaceFrame::from_rendered_page("PDF page", rendered_page())?;
        let node = frame.node();

        assert_eq!(DocumentSurfaceKind::Page, frame.kind());
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
    fn invalid_kdv_raster_is_reported_by_the_document_surface_contract() -> TestResult {
        assert!(matches!(
            DocumentSurfaceFrame::from_rendered_page(
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
            Err(DocumentSurfaceError::InvalidPage { .. })
        ));
        Ok(())
    }
}
