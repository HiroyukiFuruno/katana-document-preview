use super::{ViewerFeature, ViewerFeatureStatus, ViewerQualityProfile, ViewerQualityProfileKind};

const FEATURES: &[ViewerFeature] = &[
    ViewerFeature::PageNavigation,
    ViewerFeature::SlideNavigation,
    ViewerFeature::GridNavigation,
    ViewerFeature::Zoom,
    ViewerFeature::Fit,
    ViewerFeature::CopyText,
    ViewerFeature::TextSelection,
    ViewerFeature::OpenLink,
    ViewerFeature::FormulaValue,
    ViewerFeature::CellStyle,
    ViewerFeature::MergedCell,
    ViewerFeature::ConditionalFormatting,
    ViewerFeature::Chart,
    ViewerFeature::PivotTable,
    ViewerFeature::PrintLayout,
    ViewerFeature::Animation,
    ViewerFeature::Macro,
    ViewerFeature::ExternalResource,
];

#[test]
fn every_quality_profile_classifies_every_feature() {
    let profiles = [
        ViewerQualityProfile::static_page(),
        ViewerQualityProfile::interactive_grid(),
        ViewerQualityProfile::static_slide_with_chart_fallback(),
    ];

    for profile in profiles {
        assert_profile_contract(&profile);
    }
}

#[test]
fn interactive_and_slide_profiles_keep_their_declared_kinds() {
    assert_eq!(
        ViewerFeatureStatus::Supported,
        ViewerQualityProfile::interactive_grid().status(ViewerFeature::ConditionalFormatting)
    );
    assert_eq!(
        ViewerQualityProfileKind::StaticSlideWithChartFallback,
        ViewerQualityProfile::static_slide_with_chart_fallback().kind
    );
}

fn assert_profile_contract(profile: &ViewerQualityProfile) {
    assert_eq!(profile.kind, profile.capabilities.profile());
    for feature in FEATURES {
        let status = profile.status(*feature);
        assert!(matches!(
            status,
            ViewerFeatureStatus::Supported
                | ViewerFeatureStatus::Unsupported
                | ViewerFeatureStatus::Blocked
        ));
    }
    let diagnostics = profile.diagnostics();
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic
            .feature
            .is_some_and(|feature| profile.status(feature) == ViewerFeatureStatus::Unsupported)
    }));
}
