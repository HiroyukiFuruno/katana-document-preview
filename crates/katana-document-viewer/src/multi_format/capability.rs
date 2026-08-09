use super::ViewerDiagnostic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewerFeature {
    PageNavigation,
    SlideNavigation,
    SheetNavigation,
    GridNavigation,
    Zoom,
    Fit,
    CopyText,
    TextSelection,
    OpenLink,
    FormulaValue,
    CellStyle,
    MergedCell,
    ConditionalFormatting,
    Chart,
    PivotTable,
    PrintLayout,
    Animation,
    Macro,
    ExternalResource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewerFeatureStatus {
    Supported,
    Unsupported,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewerQualityProfileKind {
    StaticPage,
    InteractiveGrid,
    StaticSlideWithChartFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewerCapabilities {
    profile: ViewerQualityProfileKind,
}

impl ViewerCapabilities {
    #[must_use]
    pub const fn static_page() -> Self {
        Self {
            profile: ViewerQualityProfileKind::StaticPage,
        }
    }

    #[must_use]
    pub const fn interactive_grid() -> Self {
        Self {
            profile: ViewerQualityProfileKind::InteractiveGrid,
        }
    }

    #[must_use]
    pub const fn static_slide_with_chart_fallback() -> Self {
        Self {
            profile: ViewerQualityProfileKind::StaticSlideWithChartFallback,
        }
    }

    #[must_use]
    pub const fn profile(&self) -> ViewerQualityProfileKind {
        self.profile
    }

    #[must_use]
    pub const fn status(&self, feature: ViewerFeature) -> ViewerFeatureStatus {
        match self.profile {
            ViewerQualityProfileKind::StaticPage => static_page_status(feature),
            ViewerQualityProfileKind::InteractiveGrid => interactive_grid_status(feature),
            ViewerQualityProfileKind::StaticSlideWithChartFallback => static_slide_status(feature),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewerQualityProfile {
    pub kind: ViewerQualityProfileKind,
    pub capabilities: ViewerCapabilities,
}

impl ViewerQualityProfile {
    #[must_use]
    pub const fn static_page() -> Self {
        Self {
            kind: ViewerQualityProfileKind::StaticPage,
            capabilities: ViewerCapabilities::static_page(),
        }
    }

    #[must_use]
    pub const fn interactive_grid() -> Self {
        Self {
            kind: ViewerQualityProfileKind::InteractiveGrid,
            capabilities: ViewerCapabilities::interactive_grid(),
        }
    }

    #[must_use]
    pub const fn static_slide_with_chart_fallback() -> Self {
        Self {
            kind: ViewerQualityProfileKind::StaticSlideWithChartFallback,
            capabilities: ViewerCapabilities::static_slide_with_chart_fallback(),
        }
    }

    #[must_use]
    pub const fn status(&self, feature: ViewerFeature) -> ViewerFeatureStatus {
        self.capabilities.status(feature)
    }

    #[must_use]
    pub fn diagnostics(&self) -> Vec<ViewerDiagnostic> {
        profile_features(self.kind)
            .iter()
            .copied()
            .filter(|feature| self.status(*feature) == ViewerFeatureStatus::Unsupported)
            .map(ViewerDiagnostic::unsupported)
            .collect()
    }
}

const STATIC_PAGE_FEATURES: &[ViewerFeature] = &[
    ViewerFeature::CopyText,
    ViewerFeature::TextSelection,
    ViewerFeature::OpenLink,
];
const INTERACTIVE_GRID_FEATURES: &[ViewerFeature] = &[
    ViewerFeature::Chart,
    ViewerFeature::PivotTable,
    ViewerFeature::PrintLayout,
];
const STATIC_SLIDE_FEATURES: &[ViewerFeature] = &[ViewerFeature::Chart, ViewerFeature::Animation];

const fn profile_features(profile: ViewerQualityProfileKind) -> &'static [ViewerFeature] {
    match profile {
        ViewerQualityProfileKind::StaticPage => STATIC_PAGE_FEATURES,
        ViewerQualityProfileKind::InteractiveGrid => INTERACTIVE_GRID_FEATURES,
        ViewerQualityProfileKind::StaticSlideWithChartFallback => STATIC_SLIDE_FEATURES,
    }
}

const fn static_page_status(feature: ViewerFeature) -> ViewerFeatureStatus {
    match feature {
        ViewerFeature::PageNavigation | ViewerFeature::Zoom | ViewerFeature::Fit => {
            ViewerFeatureStatus::Supported
        }
        ViewerFeature::Macro | ViewerFeature::ExternalResource => ViewerFeatureStatus::Blocked,
        _ => ViewerFeatureStatus::Unsupported,
    }
}

const fn interactive_grid_status(feature: ViewerFeature) -> ViewerFeatureStatus {
    match feature {
        ViewerFeature::SheetNavigation
        | ViewerFeature::GridNavigation
        | ViewerFeature::CopyText
        | ViewerFeature::TextSelection
        | ViewerFeature::FormulaValue
        | ViewerFeature::CellStyle
        | ViewerFeature::MergedCell
        | ViewerFeature::ConditionalFormatting => ViewerFeatureStatus::Supported,
        ViewerFeature::Macro | ViewerFeature::ExternalResource => ViewerFeatureStatus::Blocked,
        _ => ViewerFeatureStatus::Unsupported,
    }
}

const fn static_slide_status(feature: ViewerFeature) -> ViewerFeatureStatus {
    match feature {
        ViewerFeature::SlideNavigation | ViewerFeature::Zoom | ViewerFeature::Fit => {
            ViewerFeatureStatus::Supported
        }
        ViewerFeature::Macro | ViewerFeature::ExternalResource => ViewerFeatureStatus::Blocked,
        _ => ViewerFeatureStatus::Unsupported,
    }
}

#[cfg(test)]
#[path = "capability_tests.rs"]
mod tests;
