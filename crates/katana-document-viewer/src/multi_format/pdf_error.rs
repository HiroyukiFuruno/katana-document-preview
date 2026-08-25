use super::PdfResourceLimitKind;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PdfViewerError {
    #[error("PDF MIME type is unsupported")]
    UnsupportedMime,
    #[error("PDF document is encrypted or password protected")]
    PasswordProtected,
    #[error("PDF document is invalid")]
    InvalidDocument,
    #[error("PDF page {requested} is outside page count {page_count}")]
    PageOutsideDocument { requested: usize, page_count: usize },
    #[error("PDF render scale must be finite and greater than zero")]
    InvalidScale,
    #[error("PDF resource limit `{kind:?}` exceeded: {actual} > {limit}")]
    ResourceLimitExceeded {
        kind: PdfResourceLimitKind,
        actual: u64,
        limit: u64,
    },
    #[error("PDF rendered page cannot be decoded")]
    RenderDecode,
}
