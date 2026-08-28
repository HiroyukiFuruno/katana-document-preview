use crate::{DocumentSessionCommand, DocumentViewerCommand, ViewerDocumentFormat, ViewerFeature};

pub(super) const fn command_feature(
    format: ViewerDocumentFormat,
    command: DocumentSessionCommand,
) -> Option<ViewerFeature> {
    match command {
        DocumentSessionCommand::Viewer(DocumentViewerCommand::Previous)
        | DocumentSessionCommand::Viewer(DocumentViewerCommand::Next)
        | DocumentSessionCommand::Viewer(DocumentViewerCommand::JumpTo(_)) => {
            Some(navigation_feature(format))
        }
        DocumentSessionCommand::Viewer(DocumentViewerCommand::SetZoom(_)) => {
            Some(ViewerFeature::Zoom)
        }
        DocumentSessionCommand::Viewer(DocumentViewerCommand::Fit(_)) => Some(ViewerFeature::Fit),
        DocumentSessionCommand::Viewer(DocumentViewerCommand::CopySelection) => {
            Some(ViewerFeature::CopyText)
        }
        DocumentSessionCommand::Viewer(DocumentViewerCommand::OpenTarget) => {
            Some(ViewerFeature::OpenLink)
        }
        DocumentSessionCommand::Surface(crate::DocumentSurfaceCommand::Grid(_)) => {
            Some(ViewerFeature::GridNavigation)
        }
        DocumentSessionCommand::Surface(crate::DocumentSurfaceCommand::Resize(_)) => None,
    }
}

const fn navigation_feature(format: ViewerDocumentFormat) -> ViewerFeature {
    match format {
        ViewerDocumentFormat::Pdf | ViewerDocumentFormat::Docx => ViewerFeature::PageNavigation,
        ViewerDocumentFormat::Xlsx => ViewerFeature::SheetNavigation,
        ViewerDocumentFormat::Pptx => ViewerFeature::SlideNavigation,
    }
}
