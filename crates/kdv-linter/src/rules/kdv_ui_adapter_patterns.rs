#[derive(Clone, Copy)]
pub(super) enum StorybookAdapterPattern {
    KdvKucCrate,
    KucStorybookHost,
    KucRenderedInteractionSurface,
    KucHostActionHitQuery,
    KucCanvasRenderer,
    KucHostActionHitRects,
    StorybookKucRendererModule,
    StorybookKucRendererFunction,
    KucBridgeModule,
}

impl StorybookAdapterPattern {
    pub(super) fn all() -> &'static [Self] {
        &[
            Self::KdvKucCrate,
            Self::KucStorybookHost,
            Self::KucRenderedInteractionSurface,
            Self::KucHostActionHitQuery,
            Self::KucCanvasRenderer,
            Self::KucHostActionHitRects,
            Self::StorybookKucRendererModule,
            Self::StorybookKucRendererFunction,
            Self::KucBridgeModule,
        ]
    }

    pub(super) fn needle(self) -> &'static str {
        match self {
            Self::KdvKucCrate => "katana_document_viewer_kuc",
            Self::KucStorybookHost => "UiTreeStorybookHost",
            Self::KucRenderedInteractionSurface => "UiTreeInteractionSurface",
            Self::KucHostActionHitQuery => "UiTreeHostActionHitQuery",
            Self::KucCanvasRenderer => "UiTreeCanvasRenderer",
            Self::KucHostActionHitRects => "host_action_hit_rects",
            Self::StorybookKucRendererModule => "frame_kuc_renderer",
            Self::StorybookKucRendererFunction => "kuc_tree_host_action_hits_at",
            Self::KucBridgeModule => "kuc_bridge",
        }
    }

    pub(super) fn message(self) -> &'static str {
        match self {
            Self::KdvKucCrate => "Storybook must not depend on a KDV-owned KUC adapter crate.",
            Self::KucStorybookHost
            | Self::KucRenderedInteractionSurface
            | Self::KucHostActionHitQuery
            | Self::KucCanvasRenderer
            | Self::KucHostActionHitRects
            | Self::StorybookKucRendererModule
            | Self::StorybookKucRendererFunction
            | Self::KucBridgeModule => {
                "Storybook must not wrap KUC renderer/hit-test internals; use a KUC-owned host contract."
            }
        }
    }
}
