use crate::multi_format::OfficeDocumentFormat;
use office2pdf::config::{ConvertOptions, Format};
use std::path::PathBuf;

pub(super) fn conversion_options(font_path: PathBuf) -> ConvertOptions {
    ConvertOptions {
        font_paths: vec![font_path],
        ..ConvertOptions::default()
    }
}

pub(super) const fn engine_format(format: OfficeDocumentFormat) -> Format {
    match format {
        OfficeDocumentFormat::Docx => Format::Docx,
        OfficeDocumentFormat::Pptx => Format::Pptx,
        OfficeDocumentFormat::Xlsx => Format::Xlsx,
    }
}
