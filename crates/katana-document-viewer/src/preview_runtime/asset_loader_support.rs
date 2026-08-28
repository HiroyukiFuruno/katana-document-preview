use crate::{Artifact, ArtifactDiagnostic, ArtifactDiagnostics, DiagnosticSeverity};
use katana_markdown_model::{CodeBlockRole, KmmNode, KmmNodeId, KmmNodeKind};
use std::path::PathBuf;

pub(crate) struct PreviewAssetLoaderSupport;

impl PreviewAssetLoaderSupport {
    pub(crate) fn find_node<'a>(nodes: &'a [KmmNode], node_id: &KmmNodeId) -> Option<&'a KmmNode> {
        for node in nodes {
            if node.id == *node_id {
                return Some(node);
            }
            if let Some(found) = Self::find_node(&node.children, node_id) {
                return Some(found);
            }
            if let Some(found) = Self::find_list_node(node, node_id) {
                return Some(found);
            }
        }
        None
    }

    pub(crate) fn diagram_source(
        node: &KmmNode,
    ) -> Option<(katana_markdown_model::DiagramKind, String)> {
        match &node.kind {
            KmmNodeKind::CodeBlock(CodeBlockRole::Diagram { kind }) => {
                Some((kind.clone(), Self::fenced_body(&node.source.raw.text)))
            }
            _ => None,
        }
    }

    pub(crate) fn math_source(node: &KmmNode) -> Option<String> {
        match &node.kind {
            KmmNodeKind::CodeBlock(CodeBlockRole::Math) => {
                Some(Self::fenced_body(&node.source.raw.text))
            }
            KmmNodeKind::DollarMathBlock(math) => Some(math.expression.clone()),
            _ => None,
        }
    }

    pub(crate) fn file_path_from_uri(uri: &str) -> Option<PathBuf> {
        let raw = uri.strip_prefix("file://")?;
        let path = raw.split(['?', '#']).next().unwrap_or(raw);
        let local_path = if let Some(rest) = path.strip_prefix("localhost/") {
            format!("/{rest}")
        } else {
            let windows_path = path.strip_prefix('/')?;
            if Self::starts_with_windows_drive(windows_path) {
                windows_path.to_string()
            } else {
                path.to_string()
            }
        };
        Some(PathBuf::from(Self::percent_decode(local_path.as_str())?))
    }

    pub(crate) fn error_diagnostics(message: String) -> ArtifactDiagnostics {
        eprintln!("[kdv-preview-asset] asset render failed: {message}");
        ArtifactDiagnostics {
            entries: vec![ArtifactDiagnostic {
                severity: DiagnosticSeverity::Error,
                code: "diagram-render-failed".to_string(),
                message,
            }],
        }
    }

    pub(crate) fn has_error(artifact: &Artifact) -> bool {
        artifact
            .manifest
            .diagnostics
            .entries
            .iter()
            .any(|entry| entry.severity == DiagnosticSeverity::Error)
    }

    fn find_list_node<'a>(node: &'a KmmNode, node_id: &KmmNodeId) -> Option<&'a KmmNode> {
        let KmmNodeKind::List(list) = &node.kind else {
            return None;
        };
        for item in &list.items {
            if let Some(found) = Self::find_node(&item.children, node_id) {
                return Some(found);
            }
        }
        None
    }

    fn fenced_body(text: &str) -> String {
        let lines = text.lines().collect::<Vec<_>>();
        if lines.len() < 2 || !Self::is_fence(lines[0]) {
            return text.to_string();
        }
        let end = lines
            .iter()
            .rposition(|line| Self::is_fence(line))
            .unwrap_or(lines.len());
        if end == 0 {
            return text.to_string();
        }
        lines[1..end].join("\n")
    }

    fn is_fence(line: &str) -> bool {
        let trimmed = line.trim_start();
        trimmed.starts_with("```") || trimmed.starts_with("~~~")
    }

    fn percent_decode(value: &str) -> Option<String> {
        let bytes = value.as_bytes();
        let mut output = Vec::with_capacity(bytes.len());
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'%'
                && index + 2 < bytes.len()
                && let Some(decoded) = Self::hex_byte(bytes[index + 1], bytes[index + 2])
            {
                output.push(decoded);
                index += 3;
                continue;
            }
            output.push(bytes[index]);
            index += 1;
        }
        String::from_utf8(output).ok()
    }

    fn starts_with_windows_drive(value: &str) -> bool {
        let bytes = value.as_bytes();
        bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'/' | b'\\')
    }

    fn hex_byte(high: u8, low: u8) -> Option<u8> {
        Some(Self::hex_digit(high)? * 16 + Self::hex_digit(low)?)
    }

    fn hex_digit(value: u8) -> Option<u8> {
        match value {
            b'0'..=b'9' => Some(value - b'0'),
            b'a'..=b'f' => Some(value - b'a' + 10),
            b'A'..=b'F' => Some(value - b'A' + 10),
            _ => None,
        }
    }
}
