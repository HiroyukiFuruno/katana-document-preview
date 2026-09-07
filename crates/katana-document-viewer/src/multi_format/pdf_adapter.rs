use super::pdf_surface::PdfSurfaceDecoder;
use super::{
    BinaryDocumentSource, PdfDocumentArtifact, PdfPageRenderRequest, PdfRenderedPage,
    PdfResourceLimitKind, PdfViewerError, PdfViewerLimits, pdf_document::PdfDocumentBuilder,
    pdf_render_cache::PdfPageCache,
};
use crate::PdfOutlineItem;
use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::{RenderCache, RenderSettings, render};
pub struct PdfViewerSession {
    pdf: Pdf,
    artifact: PdfDocumentArtifact,
    outline: Vec<PdfOutlineItem>,
    cache: PdfPageCache,
    limits: PdfViewerLimits,
    _artifact_lease: super::resource_metrics::ArtifactByteLease,
}

impl PdfViewerSession {
    pub fn open(source: BinaryDocumentSource) -> Result<Self, PdfViewerError> {
        Self::open_with_limits(source, PdfViewerLimits::strict())
    }

    pub fn open_with_limits(
        source: BinaryDocumentSource,
        limits: PdfViewerLimits,
    ) -> Result<Self, PdfViewerError> {
        validate_pdf_source(&source, limits)?;
        let artifact_lease =
            super::resource_metrics::ArtifactByteLease::acquire(source.bytes.len());
        let pdf = Pdf::new(source.bytes).map_err(map_load_error)?;
        let outline = super::pdf_outline::PdfOutlineBuilder::build(&pdf);
        let artifact = PdfDocumentBuilder::build(source.identity, source.mime, &pdf);
        check_limit(
            PdfResourceLimitKind::PageCount,
            artifact.page_count,
            limits.max_pages,
        )?;
        Ok(Self {
            pdf,
            artifact,
            outline,
            cache: PdfPageCache::new(),
            limits,
            _artifact_lease: artifact_lease,
        })
    }

    #[must_use]
    pub const fn artifact(&self) -> &PdfDocumentArtifact {
        &self.artifact
    }

    #[must_use]
    pub fn outline(&self) -> &[PdfOutlineItem] {
        &self.outline
    }

    pub fn render_page(
        &mut self,
        request: PdfPageRenderRequest,
    ) -> Result<PdfRenderedPage, PdfViewerError> {
        self.validate_request(request)?;
        let key = (request.page_index, request.scale.to_bits());
        if let Some(rendered) = self.cache.get(key) {
            super::debug_trace::DebugTrace::event("pdf.cache", "hit=true");
            return Ok(rendered);
        }
        super::debug_trace::DebugTrace::event("pdf.cache", "hit=false");
        let _render = super::debug_trace::DebugTrace::start("pdf.render");
        let rendered = self.render_uncached(request)?;
        self.cache.insert(key, rendered.clone(), self.limits);
        Ok(rendered)
    }

    #[must_use]
    pub fn cached_page_count(&self) -> usize {
        self.cache.page_count()
    }

    #[must_use]
    pub const fn cached_byte_count(&self) -> usize {
        self.cache.byte_count()
    }

    fn validate_request(&self, request: PdfPageRenderRequest) -> Result<(), PdfViewerError> {
        if request.page_index >= self.artifact.page_count {
            return Err(PdfViewerError::PageOutsideDocument {
                requested: request.page_index,
                page_count: self.artifact.page_count,
            });
        }
        if !request.scale.is_finite() || request.scale <= 0.0 {
            return Err(PdfViewerError::InvalidScale);
        }
        let page = &self.artifact.pages[request.page_index];
        let width = (page.width * request.scale).ceil() as u64;
        let height = (page.height * request.scale).ceil() as u64;
        let dimension = width.max(height);
        check_limit_u64(
            PdfResourceLimitKind::RenderDimension,
            dimension,
            u64::from(self.limits.max_render_dimension),
        )?;
        check_limit_u64(
            PdfResourceLimitKind::RenderPixels,
            width.saturating_mul(height),
            self.limits.max_render_pixels,
        )?;
        Ok(())
    }

    fn render_uncached(
        &self,
        request: PdfPageRenderRequest,
    ) -> Result<PdfRenderedPage, PdfViewerError> {
        let page = &self.pdf.pages()[request.page_index];
        let pixmap = render(
            page,
            &RenderCache::new(),
            &InterpreterSettings::default(),
            &RenderSettings {
                x_scale: request.scale,
                y_scale: request.scale,
                ..RenderSettings::default()
            },
        );
        let png = pixmap
            .into_png()
            .map_err(|_| PdfViewerError::RenderDecode)?;
        let surface = PdfSurfaceDecoder::decode(&self.artifact, request, &png)?;
        Ok(PdfRenderedPage {
            page_index: request.page_index,
            scale: request.scale,
            surface,
        })
    }
}

fn validate_pdf_source(
    source: &BinaryDocumentSource,
    limits: PdfViewerLimits,
) -> Result<(), PdfViewerError> {
    if source.mime != "application/pdf" {
        return Err(PdfViewerError::UnsupportedMime);
    }
    check_limit(
        PdfResourceLimitKind::SourceBytes,
        source.bytes.len(),
        limits.max_source_bytes,
    )
}

fn check_limit(
    kind: PdfResourceLimitKind,
    actual: usize,
    limit: usize,
) -> Result<(), PdfViewerError> {
    check_limit_u64(
        kind,
        u64::try_from(actual).unwrap_or(u64::MAX),
        u64::try_from(limit).unwrap_or(u64::MAX),
    )
}

fn check_limit_u64(
    kind: PdfResourceLimitKind,
    actual: u64,
    limit: u64,
) -> Result<(), PdfViewerError> {
    if actual <= limit {
        return Ok(());
    }
    Err(PdfViewerError::ResourceLimitExceeded {
        kind,
        actual,
        limit,
    })
}

fn map_load_error(error: hayro::hayro_syntax::LoadPdfError) -> PdfViewerError {
    match error {
        hayro::hayro_syntax::LoadPdfError::Decryption(_) => PdfViewerError::PasswordProtected,
        hayro::hayro_syntax::LoadPdfError::Invalid => PdfViewerError::InvalidDocument,
    }
}

#[cfg(test)]
#[path = "pdf_adapter_tests.rs"]
mod tests;
