use crate::preview_runtime::source_normalizer::{PreparedPreviewSource, PreviewSourceNormalizer};
use crate::preview_runtime::types::{
    MarkdownSource, PreviewConfig, PreviewDiagnostics, PreviewError, PreviewOutput,
};
use crate::{KdvThemeSnapshot, MarkdownFenceNormalizer};
use std::path::PathBuf;

pub struct PreviewOutputFactory;

impl PreviewOutputFactory {
    pub fn from_source(
        source: &MarkdownSource,
        config: &PreviewConfig,
        content_height: f32,
    ) -> Result<PreviewOutput, PreviewError> {
        let input = Self::viewer_input_from_source(source, config)?;
        Ok(Self::from_input(input, content_height))
    }

    pub fn from_content_height(config: &PreviewConfig, content_height: f32) -> PreviewOutput {
        let input = Self::synthetic_viewer_input(config);
        Self::from_input(input, content_height)
    }

    pub fn from_input(input: crate::ViewerInput, content_height: f32) -> PreviewOutput {
        PreviewOutput {
            scroll_offset: 0.0,
            content_height,
            diagnostics: PreviewDiagnostics::default(),
            surface: None,
            state: crate::ViewerStateEngine::snapshot(&input, content_height, 0.0),
            input,
            commands: Vec::new(),
        }
    }

    pub fn reconfigure(output: &PreviewOutput, config: &PreviewConfig) -> PreviewOutput {
        let mut next = output.clone();
        Self::apply_config(&mut next, config);
        next
    }

    pub fn apply_config(output: &mut PreviewOutput, config: &PreviewConfig) {
        output.input.mode = config.mode.clone();
        output.input.interaction = config.interaction.clone();
        output.input.typography = Self::typography(config);
        output.input.viewport = config.viewport;
        output.input.search = config.search.clone();
        output.input.theme = Self::theme(config);
        output.scroll_offset = config.scroll_offset.max(0.0);
        output.content_height = Self::content_height_for_surface(output, config);
        output.state = crate::ViewerStateEngine::snapshot(
            &output.input,
            output.content_height,
            output.scroll_offset,
        );
    }

    fn content_height_for_surface(output: &PreviewOutput, config: &PreviewConfig) -> f32 {
        let Some(surface) = &output.surface else {
            return output.content_height;
        };
        let scale = if surface.width == 0 || config.viewport.width <= 0.0 {
            1.0
        } else {
            config.viewport.width / surface.width as f32
        };
        Self::scrollable_height(
            surface.content_height as f32 * scale,
            config.viewport.height,
        )
    }

    fn scrollable_height(content_height: f32, viewport_height: f32) -> f32 {
        let viewport_height = viewport_height.max(0.0);
        if content_height <= viewport_height {
            return content_height;
        }
        content_height + viewport_height
    }

    fn viewer_input_from_source(
        source: &MarkdownSource,
        config: &PreviewConfig,
    ) -> Result<crate::ViewerInput, PreviewError> {
        let prepared = PreviewSourceNormalizer::normalize(source)?;
        let content = MarkdownFenceNormalizer::normalize(&prepared.content);
        let document = Self::parse_document(&prepared, &content)?;
        let document_source =
            Self::document_source(&prepared, content, &document.fingerprint.value);
        let snapshot = crate::DocumentSnapshotFactory::from_kmm_with_kind(
            document_source,
            document,
            prepared.document_kind,
        );

        Ok(crate::ViewerInput {
            snapshot,
            artifacts: Vec::new(),
            theme: Self::theme(config),
            mode: config.mode.clone(),
            interaction: config.interaction.clone(),
            typography: Self::typography(config),
            viewport: config.viewport,
            search: config.search.clone(),
        })
    }

    fn parse_document(
        source: &PreparedPreviewSource,
        content: &str,
    ) -> Result<katana_markdown_model::KmmDocument, PreviewError> {
        let input = katana_markdown_model::MarkdownInput::from_content(
            source.source_path.clone(),
            content.to_string(),
        );
        katana_markdown_model::KatanaMarkdownModel::parse(input)
            .map_err(|error| PreviewError::Render(error.to_string()))
    }

    fn document_source(
        source: &PreparedPreviewSource,
        content: String,
        revision: &str,
    ) -> crate::DocumentSource {
        let document_name = source.source_path.to_string_lossy();
        crate::DocumentSource {
            uri: crate::SourceUri(format!("preview://{document_name}")),
            kind: source.source_kind.clone(),
            revision: crate::SourceRevision(revision.to_string()),
            content,
        }
    }

    fn synthetic_viewer_input(config: &PreviewConfig) -> crate::ViewerInput {
        crate::ViewerInput {
            snapshot: Self::synthetic_snapshot(),
            artifacts: Vec::new(),
            theme: Self::theme(config),
            mode: config.mode.clone(),
            interaction: config.interaction.clone(),
            typography: Self::typography(config),
            viewport: config.viewport,
            search: config.search.clone(),
        }
    }

    fn synthetic_snapshot() -> crate::DocumentSnapshot {
        crate::DocumentSnapshot {
            id: crate::DocumentId("preview".to_string()),
            kind: crate::DocumentKind::Markdown,
            source_uri: crate::SourceUri("preview://source".to_string()),
            revision: crate::SourceRevision("preview".to_string()),
            source_path: PathBuf::from("preview.md"),
            document: katana_markdown_model::KmmDocument {
                path: PathBuf::from("preview.md"),
                fingerprint: katana_markdown_model::TextFingerprint {
                    algorithm: "synthetic".to_string(),
                    value: "preview".to_string(),
                },
                nodes: Vec::new(),
            },
            outline: crate::DocumentOutline { items: Vec::new() },
            metadata: crate::DocumentMetadataView {
                unresolved_count: 0,
                diagnostic_keys: Vec::new(),
            },
        }
    }

    fn theme(config: &PreviewConfig) -> KdvThemeSnapshot {
        if config.theme.is_dark() {
            return KdvThemeSnapshot::katana_dark();
        }
        KdvThemeSnapshot::katana_light()
    }

    fn typography(config: &PreviewConfig) -> crate::ViewerTypographyConfig {
        let fallback = crate::ViewerTypographyConfig::default().preview_font_size;
        let preview_font_size = config
            .base_font_size
            .map(|font_size| font_size.round().clamp(12.0, 32.0) as u16)
            .unwrap_or(fallback);
        crate::ViewerTypographyConfig { preview_font_size }
    }
}

#[cfg(test)]
#[path = "output_factory_tests.rs"]
mod tests;
