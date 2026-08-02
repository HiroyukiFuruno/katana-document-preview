use super::{
    PdfDocumentArtifact, PdfPageArtifact, PdfPageRotation, ViewerCapabilities,
    ViewerQualityProfile, ViewerSourceIdentity,
};
use hayro::hayro_syntax::Pdf;
use hayro::hayro_syntax::page::Rotation;

pub(super) struct PdfDocumentBuilder;

impl PdfDocumentBuilder {
    pub(super) fn build(
        identity: ViewerSourceIdentity,
        mime: String,
        pdf: &Pdf,
    ) -> PdfDocumentArtifact {
        let pages = pdf
            .pages()
            .iter()
            .enumerate()
            .map(Self::page)
            .collect::<Vec<_>>();
        PdfDocumentArtifact {
            identity,
            mime,
            page_count: pages.len(),
            pages,
            capabilities: ViewerCapabilities::static_page(),
            diagnostics: ViewerQualityProfile::static_page().diagnostics(),
        }
    }

    fn page((index, page): (usize, &hayro::hayro_syntax::page::Page)) -> PdfPageArtifact {
        let (width, height) = page.render_dimensions();
        PdfPageArtifact {
            index,
            width,
            height,
            rotation: Self::rotation(page.rotation()),
        }
    }

    const fn rotation(rotation: Rotation) -> PdfPageRotation {
        match rotation {
            Rotation::None => PdfPageRotation::None,
            Rotation::Horizontal => PdfPageRotation::Clockwise90,
            Rotation::Flipped => PdfPageRotation::Clockwise180,
            Rotation::FlippedHorizontal => PdfPageRotation::Clockwise270,
        }
    }
}

#[cfg(test)]
#[path = "pdf_document_tests.rs"]
mod tests;
