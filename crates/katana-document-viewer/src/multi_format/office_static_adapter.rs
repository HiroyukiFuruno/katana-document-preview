use super::office_worker_parent::OfficeWorkerRunner;
use super::{
    BinaryDocumentSource, OfficeDocumentFormat, OfficeDocumentSource, OfficeStaticDocumentArtifact,
    OfficeStaticItemArtifact, OfficeWorkerConfig, OfficeWorkerError, PdfPageRenderRequest,
    PdfRenderedPage, PdfViewerError, PdfViewerSession, ViewerDiagnostic, ViewerDiagnosticCode,
    ViewerDiagnosticSeverity, ViewerQualityProfile,
};

pub struct OfficeStaticViewerSession {
    artifact: OfficeStaticDocumentArtifact,
    pdf: PdfViewerSession,
    conversion_key: super::office_conversion_key::OfficeConversionKey,
}

impl std::fmt::Debug for OfficeStaticViewerSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OfficeStaticViewerSession")
            .field("artifact", &self.artifact)
            .finish_non_exhaustive()
    }
}

impl OfficeStaticViewerSession {
    pub fn open(
        source: OfficeDocumentSource,
        config: OfficeWorkerConfig,
    ) -> Result<Self, OfficeWorkerError> {
        let _open = super::debug_trace::DebugTrace::start("office.session_open");
        let conversion_key =
            super::office_conversion_key::OfficeConversionKey::new(&source, &config);
        let profile = static_profile(source.format)?;
        let output = OfficeWorkerRunner::convert(&source, &config)?;
        let mut diagnostics = profile.diagnostics();
        diagnostics.extend(output.preflight_diagnostics);
        diagnostics.extend(output.warnings.into_iter().map(engine_warning));
        let pdf_source =
            BinaryDocumentSource::new(source.identity.clone(), "application/pdf", output.pdf);
        let pdf = {
            let _decode = super::debug_trace::DebugTrace::start("office.pdf_decode");
            PdfViewerSession::open(pdf_source)?
        };
        let items = static_items(&pdf);
        let artifact = static_artifact(source, profile, diagnostics, items);
        Ok(Self {
            artifact,
            pdf,
            conversion_key,
        })
    }

    #[must_use]
    pub const fn artifact(&self) -> &OfficeStaticDocumentArtifact {
        &self.artifact
    }

    #[cfg(test)]
    pub(super) const fn conversion_key(
        &self,
    ) -> &super::office_conversion_key::OfficeConversionKey {
        &self.conversion_key
    }

    pub fn render_item(
        &mut self,
        request: PdfPageRenderRequest,
    ) -> Result<PdfRenderedPage, PdfViewerError> {
        let _render = super::debug_trace::DebugTrace::start("office.frame");
        super::debug_trace::DebugTrace::event(
            "office.frame_artifact",
            format_args!("key={:?}", self.conversion_key),
        );
        self.pdf.render_page(request)
    }
}

fn static_profile(format: OfficeDocumentFormat) -> Result<ViewerQualityProfile, OfficeWorkerError> {
    match format {
        OfficeDocumentFormat::Docx => Ok(ViewerQualityProfile::static_page()),
        OfficeDocumentFormat::Pptx => Ok(ViewerQualityProfile::static_slide_with_chart_fallback()),
        OfficeDocumentFormat::Xlsx => Err(OfficeWorkerError::UnsupportedFormat(format)),
    }
}

fn static_items(pdf: &PdfViewerSession) -> Vec<OfficeStaticItemArtifact> {
    pdf.artifact()
        .pages
        .iter()
        .map(|page| OfficeStaticItemArtifact {
            index: page.index,
            width: page.width,
            height: page.height,
            rotation: page.rotation,
        })
        .collect()
}

fn static_artifact(
    source: OfficeDocumentSource,
    profile: ViewerQualityProfile,
    diagnostics: Vec<ViewerDiagnostic>,
    items: Vec<OfficeStaticItemArtifact>,
) -> OfficeStaticDocumentArtifact {
    OfficeStaticDocumentArtifact {
        identity: source.identity,
        format: source.format,
        mime: source.mime,
        item_count: items.len(),
        items,
        capabilities: profile.capabilities,
        diagnostics,
    }
}

fn engine_warning(message: String) -> ViewerDiagnostic {
    ViewerDiagnostic {
        code: ViewerDiagnosticCode::DegradedRendering,
        severity: ViewerDiagnosticSeverity::Warning,
        feature: None,
        status: None,
        message,
    }
}

#[cfg(test)]
#[path = "office_static_adapter_tests.rs"]
mod tests;
