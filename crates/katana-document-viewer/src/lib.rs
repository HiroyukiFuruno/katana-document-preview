//! Document viewer runtime, presentation, and export foundation for KatanA.
//!
//! KDV receives KMM public DTOs, delegates supported rendering to KRR,
//! and owns the document surface presented by host applications.

pub mod artifact;
pub mod backend;
pub mod browser_session;
pub mod cli_api;
pub mod document;
mod document_surface;
mod emoji_text;
pub mod evaluation;
mod export_assets;
#[path = "export_html/export_block_payload.rs"]
mod export_block_payload;
#[path = "export_html/export_code_payload.rs"]
mod export_code_payload;
pub mod export_contract;
mod export_contract_entries;
#[path = "export_html/export_details_payload.rs"]
mod export_details_payload;
#[path = "export_html/export_footnote_payload.rs"]
mod export_footnote_payload;
#[path = "export_html/export_heading_payload.rs"]
mod export_heading_payload;
#[path = "export_html/export_html_ops.rs"]
mod export_html_ops;
#[path = "export_html/export_html_payload.rs"]
mod export_html_payload;
#[path = "export_html/export_html_style.rs"]
mod export_html_style;
#[path = "export_payload/export_image_payload.rs"]
mod export_image_payload;
#[path = "export_html/export_inline_payload.rs"]
mod export_inline_payload;
#[path = "export_html/export_legacy_note_payload.rs"]
mod export_legacy_note_payload;
#[path = "export_html/export_list_payload.rs"]
mod export_list_payload;
#[path = "export_html/export_math_payload.rs"]
mod export_math_payload;
#[path = "export_payload/export_payload.rs"]
mod export_payload;
#[path = "export_payload/export_pdf_payload.rs"]
mod export_pdf_payload;
mod export_postprocess;
mod export_quality;
mod export_semantics;
mod export_surface;
#[path = "export_surface/export_surface_code.rs"]
mod export_surface_code;
#[path = "export_surface/export_surface_font.rs"]
mod export_surface_font;
#[path = "export_surface/export_surface_helpers.rs"]
mod export_surface_helpers;
#[path = "export_surface/export_surface_line.rs"]
mod export_surface_line;
#[path = "export_surface/export_surface_math.rs"]
mod export_surface_math;
#[path = "export_surface/export_surface_span.rs"]
mod export_surface_span;
#[path = "export_surface/export_surface_svg.rs"]
mod export_surface_svg;
#[path = "export_surface/export_surface_text.rs"]
mod export_surface_text;
#[path = "export_html/export_table_payload.rs"]
mod export_table_payload;
pub mod forge;
mod forge_diagram_render;
mod forge_diagram_render_types;
mod forge_types;
mod html_sanitizer;
mod html_style;
mod markdown_fence_normalizer;
pub mod multi_format;
mod preview_runtime;
mod preview_surface;
mod render_runtime;
mod theme;
pub mod viewer;

pub use artifact::{
    Artifact, ArtifactBytes, ArtifactDiagnostic, ArtifactDiagnostics, ArtifactFactory,
    ArtifactFormat, ArtifactId, ArtifactKind, ArtifactManifest, ArtifactTextExtraction,
    ArtifactUri, DiagnosticSeverity,
};
pub use backend::diagram::{KrrDiagramInputFactory, KrrRenderOutputFactory};
pub use backend::math::KrrMathRenderEngine;
pub use cli_api::{
    CliApi, CliBuildRequest, CliDiagnostics, CliExportDebugRequest, CliExportRequest, CliOutput,
    CliRequest, CliThemeMode,
};
pub use document::{
    DocumentId, DocumentKind, DocumentMetadataView, DocumentModelError, DocumentOutline,
    DocumentOutlineItem, DocumentSnapshot, DocumentSnapshotFactory, DocumentSource, SourceKind,
    SourceRevision, SourceUri,
};
pub use document_surface::{
    DocumentGridBorderSide, DocumentGridCell, DocumentGridCellAppearance, DocumentGridCellBorders,
    DocumentGridCommand, DocumentGridCoordinate, DocumentGridDataBar, DocumentGridEvent,
    DocumentGridHorizontalAlignment, DocumentGridIcon, DocumentGridNavigation, DocumentGridRating,
    DocumentGridSurfaceFrame, DocumentGridVerticalAlignment, DocumentGridViewport,
    DocumentPageSurfaceFrame, DocumentRect, DocumentSurfaceCommand, DocumentSurfaceError,
    DocumentSurfaceFrame, DocumentSurfaceKind, DocumentViewport, PdfOutlineItem,
    SpreadsheetGridSurface,
};
pub use evaluation::{
    BackendCapability, BackendCapabilityMatrix, CoverageStatus, EvaluationCoverageMatrix,
    EvaluationFeatureCoverage, EvaluationFixture, EvaluationFixtureMatrix, FixtureCategory,
};
pub use export_contract::{HtmlExportContractEntry, HtmlExportContractMatrix, HtmlExportReadiness};
pub use export_postprocess::{
    ExportPostprocessEvaluationReport, ExportPostprocessEvaluationRequest,
    ExportPostprocessEvaluationService, ExportPostprocessMode, ExportPostprocessPolicy,
    ExportPostprocessStatus, KaruiPdfPostprocessAdapter, PdfPostprocessAdapter,
    PdfPostprocessError, PdfPostprocessInput, PdfPostprocessOutput,
};
pub use export_quality::{
    ExportFormatQualityScore, ExportQualityArtifacts, ExportQualityCheck, ExportQualityGate,
    ExportQualityReport, SurfaceEquivalenceArtifacts, SurfaceEquivalenceGate,
    SurfaceEquivalenceImage, SurfaceEquivalenceReport,
};
pub use forge::{
    BuildGraph, BuildProfile, BuildRequest, ExportFormat, ExportOutput, ExportRequest,
    ForgeBackend, ForgeDiagnostics, ForgeError, ForgePipeline, ManifestOnlyBackend,
    MarkdownEvaluationTarget, RenderedDiagram, TransformStep,
};
pub use forge_diagram_render_types::{
    DiagramRenderCacheOptions, DiagramRenderEngine, DiagramRenderRequest, DiagramRenderingBackend,
    KrrDiagramRenderEngine,
};
pub use html_sanitizer::HtmlFragmentNormalizer;
pub use katana_markdown_model::{
    ByteRange, KmmNodeId, LineColumn, LineColumnRange, RawSnippet, SourceSpan,
};
pub use markdown_fence_normalizer::MarkdownFenceNormalizer;
pub use multi_format::{
    BinaryDocumentSource, DocumentFitMode, DocumentFrame, DocumentResourceSnapshot,
    DocumentSession, DocumentSessionCommand, DocumentSessionCommandKind, DocumentSessionConfig,
    DocumentSessionError, DocumentSessionEvent, DocumentSessionInfo, DocumentViewerCommand,
    DocumentViewerEvent, DocumentViewerState, DocumentViewerStateError, OfficeDocumentFormat,
    OfficeDocumentSource, OfficePackagePreflight, OfficePreflightError, OfficePreflightLimits,
    OfficePreflightReport, OfficeResourceLimitKind, OfficeStaticDocumentArtifact,
    OfficeStaticItemArtifact, OfficeStaticViewerSession, OfficeWorkerConfig,
    OfficeWorkerEntrypoint, OfficeWorkerError, PdfDocumentArtifact, PdfPageArtifact,
    PdfPageRenderRequest, PdfPageRotation, PdfRenderedPage, PdfResourceLimitKind, PdfViewerError,
    PdfViewerLimits, PdfViewerSession, SpreadsheetAutoFilterArtifact,
    SpreadsheetBorderSideArtifact, SpreadsheetCellArtifact, SpreadsheetCellBorderArtifact,
    SpreadsheetCellStyleArtifact, SpreadsheetCellValue, SpreadsheetConditionalFormattingArtifact,
    SpreadsheetCoordinate, SpreadsheetDataBarArtifact, SpreadsheetDocumentArtifact,
    SpreadsheetFilterColumnArtifact, SpreadsheetFilterCommand, SpreadsheetFilterCriterion,
    SpreadsheetFilterEvent, SpreadsheetFilterRange, SpreadsheetFrameMetadata,
    SpreadsheetHorizontalAlignment, SpreadsheetIconArtifact, SpreadsheetMergedCellArtifact,
    SpreadsheetRatingArtifact, SpreadsheetSheetArtifact, SpreadsheetTrackArtifact,
    SpreadsheetVerticalAlignment, SpreadsheetViewerLimits, SpreadsheetViewerSession,
    SpreadsheetWorkerEntrypoint, ViewerCapabilities, ViewerDiagnostic, ViewerDiagnosticCode,
    ViewerDiagnosticSeverity, ViewerDocumentFormat, ViewerFeature, ViewerFeatureStatus,
    ViewerQualityProfile, ViewerQualityProfileKind, ViewerSource, ViewerSourceIdentity,
};
pub use preview_runtime::{
    DirectHtmlPreviewRenderer, MarkdownPreview, MarkdownSource, PreviewAssetLoadReport,
    PreviewAssetLoader, PreviewConfig, PreviewDiagnostics, PreviewError, PreviewOutput,
    PreviewOutputFactory, PreviewRenderEngine, PreviewSurfaceImage, PreviewTheme, RenderTarget,
};
pub use preview_surface::{
    KDV_INTERACTIVE_PREVIEW_SURFACE_HORIZONTAL_PADDING_PX,
    KDV_INTERACTIVE_PREVIEW_SURFACE_PADDING_PX, KDV_VIEWER_SURFACE_PADDING_PX,
    KdvPdfSurfaceFactory, KdvPreviewSurface, KdvPreviewSurfaceFactory,
};
pub use theme::{KdvThemeMode, KdvThemeSnapshot};
pub use viewer::{
    CopyTextCommand, CopyTextSource, DiagramControlCommand, DiagramControlParity,
    DiagramControlRequirement, DiagramPanCommand, DiagramPanSource, DiagramViewportState,
    DiagramZoomCommand, DiagramZoomSource, HostCommand, ImageControlAction, ImageControlCommand,
    SlideshowCommand, SlideshowSettingsUpdate, SlideshowState, TaskStateCommand,
    VIEWER_DIAGRAM_DISPLAY_MAX_WIDTH, VIEWER_DIAGRAM_DISPLAY_SCALE, VIEWER_TEXT_COLOR_CHANNELS,
    ViewerArtifactSearchResolver, ViewerArtifactTextExtraction, ViewerAssetLoadPriority,
    ViewerAssetLoadRequest, ViewerAssetLoadResult, ViewerAssetPipeline, ViewerAssetReference,
    ViewerAssetState, ViewerCodeBlockMetrics, ViewerCodeHighlighter, ViewerCommand,
    ViewerCommandFactory, ViewerConfigRevision, ViewerDiagramControlSlot, ViewerDiagramKind,
    ViewerHitTestIndex, ViewerHitTestResponse, ViewerHtmlAlignment, ViewerHtmlRole,
    ViewerImageSurface, ViewerImageSurfaceError, ViewerImageSurfaceFactory, ViewerInput,
    ViewerInteractionConfig, ViewerLayoutEngine, ViewerLayoutResult, ViewerMediaControlAction,
    ViewerMediaControlKind, ViewerMediaControlSet, ViewerMediaControlSpec, ViewerMode,
    ViewerModeSwitch, ViewerNode, ViewerNodeKind, ViewerNodePlan, ViewerNodePlanner, ViewerPoint,
    ViewerRect, ViewerRectFactory, ViewerRenderedAnchor, ViewerScrollCommand, ViewerSearchCommand,
    ViewerSearchDirection, ViewerSearchEngine, ViewerSearchHighlight, ViewerSearchHighlightKind,
    ViewerSearchLayoutResolver, ViewerSearchMatch, ViewerSearchMatchId, ViewerSearchState,
    ViewerSearchTarget, ViewerSearchTextMatch, ViewerSearchTextMatcher, ViewerSession,
    ViewerSettingsField, ViewerSettingsState, ViewerSettingsUpdate, ViewerSettingsUpdateError,
    ViewerSettingsValue, ViewerSlideshowControlAction, ViewerStateEngine, ViewerStateSnapshot,
    ViewerTarget, ViewerTaskControlTarget, ViewerTaskState, ViewerTextRange, ViewerTextSpan,
    ViewerTextStyle, ViewerTocCommandFactory, ViewerTocItem, ViewerTocModel,
    ViewerTypographyConfig, ViewerVector, ViewerViewport, ViewerVisibleRange,
};
#[cfg(test)]
mod dependency_tests;
#[cfg(test)]
mod test_support;
