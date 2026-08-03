use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewerSourceIdentity {
    pub uri: String,
    pub revision: String,
}

impl ViewerSourceIdentity {
    #[must_use]
    pub fn new(uri: impl Into<String>, revision: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            revision: revision.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinaryDocumentSource {
    pub identity: ViewerSourceIdentity,
    pub mime: String,
    pub bytes: Vec<u8>,
}

impl BinaryDocumentSource {
    #[must_use]
    pub fn new(identity: ViewerSourceIdentity, mime: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            identity,
            mime: mime.into(),
            bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfficeDocumentFormat {
    Docx,
    Xlsx,
    Pptx,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfficeDocumentSource {
    pub identity: ViewerSourceIdentity,
    pub format: OfficeDocumentFormat,
    pub mime: String,
    pub bytes: Vec<u8>,
}

impl OfficeDocumentSource {
    #[must_use]
    pub fn new(
        identity: ViewerSourceIdentity,
        format: OfficeDocumentFormat,
        mime: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            identity,
            format,
            mime: mime.into(),
            bytes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewerSource {
    Pdf(BinaryDocumentSource),
    Office(OfficeDocumentSource),
}

impl ViewerSource {
    #[must_use]
    pub const fn identity(&self) -> &ViewerSourceIdentity {
        match self {
            Self::Pdf(source) => &source.identity,
            Self::Office(source) => &source.identity,
        }
    }

    #[must_use]
    pub fn mime(&self) -> &str {
        match self {
            Self::Pdf(source) => &source.mime,
            Self::Office(source) => &source.mime,
        }
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Pdf(source) => &source.bytes,
            Self::Office(source) => &source.bytes,
        }
    }

    #[must_use]
    pub const fn office_format(&self) -> Option<OfficeDocumentFormat> {
        match self {
            Self::Pdf(_) => None,
            Self::Office(source) => Some(source.format),
        }
    }
}

#[cfg(test)]
#[path = "source_tests.rs"]
mod tests;
