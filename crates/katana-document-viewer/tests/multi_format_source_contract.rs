use katana_document_viewer::{
    BinaryDocumentSource, DocumentFitMode, DocumentViewerCommand, DocumentViewerEvent,
    DocumentViewerState, DocumentViewerStateError, OfficeDocumentFormat, OfficeDocumentSource,
    ViewerDiagnosticCode, ViewerFeature, ViewerFeatureStatus, ViewerQualityProfile,
    ViewerQualityProfileKind, ViewerSource, ViewerSourceIdentity,
};

#[test]
fn source_contract_keeps_identity_revision_mime_and_format() {
    let identity = ViewerSourceIdentity::new("file:///tmp/report.pdf", "sha256:pdf");
    let pdf = ViewerSource::Pdf(BinaryDocumentSource::new(
        identity.clone(),
        "application/pdf",
        vec![1, 2, 3],
    ));
    let office = ViewerSource::Office(OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///tmp/report.docx", "sha256:docx"),
        OfficeDocumentFormat::Docx,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        vec![4, 5, 6],
    ));

    assert_eq!(&identity, pdf.identity());
    assert_eq!("application/pdf", pdf.mime());
    assert_eq!(3, pdf.bytes().len());
    assert_eq!(None, pdf.office_format());
    assert_eq!("file:///tmp/report.docx", office.identity().uri);
    assert_eq!(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        office.mime()
    );
    assert_eq!(Some(OfficeDocumentFormat::Docx), office.office_format());
    assert_eq!(3, office.bytes().len());
}

#[test]
fn approved_profiles_expose_typed_unsupported_features() {
    let pdf = ViewerQualityProfile::static_page();
    let xlsx = ViewerQualityProfile::interactive_grid();
    let pptx = ViewerQualityProfile::static_slide_with_chart_fallback();

    assert_eq!(
        ViewerQualityProfileKind::InteractiveGrid,
        xlsx.capabilities.profile()
    );
    assert_eq!(
        ViewerFeatureStatus::Blocked,
        pdf.status(ViewerFeature::Macro)
    );
    assert_eq!(
        ViewerFeatureStatus::Blocked,
        xlsx.status(ViewerFeature::ExternalResource)
    );
    assert_eq!(
        ViewerFeatureStatus::Blocked,
        pptx.status(ViewerFeature::Macro)
    );
    assert_eq!(
        ViewerFeatureStatus::Unsupported,
        xlsx.status(ViewerFeature::Chart)
    );
    assert_eq!(
        ViewerFeatureStatus::Unsupported,
        xlsx.status(ViewerFeature::PivotTable)
    );
    assert_eq!(
        ViewerFeatureStatus::Unsupported,
        xlsx.status(ViewerFeature::PrintLayout)
    );
    assert_eq!(
        ViewerFeatureStatus::Unsupported,
        pptx.status(ViewerFeature::Chart)
    );
    assert_eq!(
        ViewerFeatureStatus::Supported,
        pptx.status(ViewerFeature::SlideNavigation)
    );
    assert!(xlsx.diagnostics().iter().any(|diagnostic| {
        diagnostic.code == ViewerDiagnosticCode::UnsupportedFeature
            && diagnostic.feature == Some(ViewerFeature::Chart)
    }));
}

#[test]
fn document_state_applies_navigation_zoom_and_fit_without_silent_clamping() {
    let mut state = DocumentViewerState::new(3);

    assert_eq!(
        Ok(DocumentViewerEvent::IndexChanged(1)),
        state.apply(DocumentViewerCommand::Next)
    );
    assert_eq!(
        Ok(DocumentViewerEvent::IndexChanged(2)),
        state.apply(DocumentViewerCommand::JumpTo(2))
    );
    assert_eq!(
        Ok(DocumentViewerEvent::IndexChanged(1)),
        state.apply(DocumentViewerCommand::Previous)
    );
    assert_eq!(
        Ok(DocumentViewerEvent::CopyRequested),
        state.apply(DocumentViewerCommand::CopySelection)
    );
    assert_eq!(
        Ok(DocumentViewerEvent::OpenRequested),
        state.apply(DocumentViewerCommand::OpenTarget)
    );
    assert_eq!(
        Err(DocumentViewerStateError::IndexOutsideDocument {
            requested: 3,
            item_count: 3,
        }),
        state.apply(DocumentViewerCommand::JumpTo(3))
    );
    assert_eq!(
        Ok(DocumentViewerEvent::ZoomChanged(1.5)),
        state.apply(DocumentViewerCommand::SetZoom(1.5))
    );
    assert_eq!(
        Ok(DocumentViewerEvent::FitChanged(DocumentFitMode::Width)),
        state.apply(DocumentViewerCommand::Fit(DocumentFitMode::Width))
    );
    assert_eq!(
        Err(DocumentViewerStateError::InvalidZoom),
        state.apply(DocumentViewerCommand::SetZoom(f32::NAN))
    );
    assert_eq!(1, state.active_index);
    assert_eq!(1.5, state.zoom);
    assert_eq!(Some(DocumentFitMode::Width), state.fit);

    let mut empty = DocumentViewerState::new(0);
    assert_eq!(
        Ok(DocumentViewerEvent::None),
        empty.apply(DocumentViewerCommand::Previous)
    );
    assert_eq!(
        Ok(DocumentViewerEvent::None),
        empty.apply(DocumentViewerCommand::Next)
    );
}
