use super::{
    OfficeDocumentFormat, OfficePreflightError, OfficePreflightLimits, OfficePreflightSupport,
    OfficeResourceLimitKind, ViewerDiagnosticCode, ViewerFeature, ViewerFeatureStatus,
};

#[test]
fn strict_limits_are_the_default_and_mime_profiles_are_fixed() {
    assert_eq!(
        OfficePreflightLimits::strict(),
        OfficePreflightLimits::default()
    );
    let cases = [
        (
            OfficeDocumentFormat::Docx,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ),
        (
            OfficeDocumentFormat::Xlsx,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
        (
            OfficeDocumentFormat::Pptx,
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ),
    ];
    for (format, mime) in cases {
        assert_eq!(mime, OfficePreflightSupport::expected_mime(format));
    }
}

#[test]
fn structural_preflight_errors_keep_typed_diagnostics() {
    let errors = [
        (
            OfficePreflightError::UnsupportedMime {
                format: OfficeDocumentFormat::Docx,
                mime: "application/octet-stream".to_owned(),
            },
            ViewerDiagnosticCode::UnsupportedFormat,
        ),
        (
            OfficePreflightSupport::invalid_archive("broken zip".to_owned()),
            ViewerDiagnosticCode::InvalidDocument,
        ),
    ];
    for (error, code) in errors {
        assert_eq!(code, error.diagnostic().code);
    }
}

#[test]
fn unsafe_and_oversized_preflight_errors_keep_typed_diagnostics() {
    let errors = [
        (
            OfficePreflightError::UnsafeEntryName {
                entry: "../escape".to_owned(),
            },
            ViewerDiagnosticCode::InvalidDocument,
        ),
        (
            OfficePreflightSupport::resource_limit(
                OfficeResourceLimitKind::SourceBytes,
                2,
                1,
                None,
            ),
            ViewerDiagnosticCode::ResourceLimitExceeded,
        ),
    ];
    for (error, code) in errors {
        assert_eq!(code, error.diagnostic().code);
    }
}

#[test]
fn blocked_feature_diagnostics_preserve_feature_and_status() {
    let errors = [
        (
            OfficePreflightError::ActiveContentBlocked {
                entry: "word/vbaProject.bin".to_owned(),
            },
            ViewerDiagnosticCode::ActiveContentBlocked,
            ViewerFeature::Macro,
        ),
        (
            OfficePreflightError::ExternalResourceBlocked {
                entry: "word/_rels/document.xml.rels".to_owned(),
                target: "https://example.invalid/image.png".to_owned(),
            },
            ViewerDiagnosticCode::ExternalResourceBlocked,
            ViewerFeature::ExternalResource,
        ),
    ];
    for (error, code, feature) in errors {
        let diagnostic = error.diagnostic();
        assert_eq!(code, diagnostic.code);
        assert_eq!(Some(feature), diagnostic.feature);
        assert_eq!(Some(ViewerFeatureStatus::Blocked), diagnostic.status);
    }
}
