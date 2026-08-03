use crate::DocumentViewerCommand;
use crate::viewer::search::ViewerSearchCommand;
use crate::viewer::types::{ViewerTarget, ViewerVector};
use katana_markdown_model::KmmNodeId;
use serde::{Deserialize, Serialize};

#[path = "commands_factory.rs"]
mod commands_factory;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViewerCommand {
    ScrollToHeading(ViewerScrollCommand),
    Slideshow(SlideshowCommand),
    Search(ViewerSearchCommand),
    Diagram(DiagramControlCommand),
    Image(ImageControlCommand),
    Task(TaskStateCommand),
    Link(LinkCommand),
    Host(HostCommand),
    Document(DocumentViewerCommand),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewerScrollCommand {
    pub target: ViewerTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlideshowCommand {
    NextPage,
    PreviousPage,
    Close,
    UpdateSettings(SlideshowSettingsUpdate),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlideshowSettingsUpdate {
    pub hover_highlight_enabled: bool,
    pub diagram_controls_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiagramControlCommand {
    FullscreenOpen(ViewerTarget),
    FullscreenClose(ViewerTarget),
    Pan(DiagramPanCommand),
    Zoom(DiagramZoomCommand),
    Reset(ViewerTarget),
    TrackpadHelp(ViewerTarget),
}

impl DiagramControlCommand {
    #[must_use]
    pub const fn requires_host_propagation(&self) -> bool {
        matches!(self, Self::FullscreenOpen(_) | Self::FullscreenClose(_))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramPanCommand {
    pub target: ViewerTarget,
    pub delta: ViewerVector,
    pub source: DiagramPanSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagramPanSource {
    ButtonUp,
    ButtonDown,
    ButtonLeft,
    ButtonRight,
    Drag,
    SmoothScroll,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramZoomCommand {
    pub target: ViewerTarget,
    pub multiplier: f32,
    pub source: DiagramZoomSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagramZoomSource {
    ButtonIn,
    ButtonOut,
    Trackpad,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageControlCommand {
    pub target: ViewerTarget,
    pub action: ImageControlAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageControlAction {
    Copy,
    Open,
    Fit,
    RevealInOs,
    ZoomIn,
    ZoomOut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskStateCommand {
    pub target: ViewerTarget,
    #[serde(default)]
    pub task_target: Option<ViewerTaskControlTarget>,
    pub state: ViewerTaskState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewerTaskControlTarget {
    pub node_id: KmmNodeId,
    pub row_index: usize,
    pub state_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkCommand {
    pub target: ViewerTarget,
    pub uri: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewerTaskState {
    Empty,
    Done,
    Progress,
    Blocked,
}

impl ViewerTaskState {
    pub fn from_marker(marker: &str) -> Option<Self> {
        match marker {
            "[ ]" => Some(Self::Empty),
            "[x]" | "[X]" => Some(Self::Done),
            "[/]" => Some(Self::Progress),
            "[-]" => Some(Self::Blocked),
            _ => None,
        }
    }

    pub fn marker(self) -> &'static str {
        match self {
            Self::Empty => "[ ]",
            Self::Done => "[x]",
            Self::Progress => "[/]",
            Self::Blocked => "[-]",
        }
    }

    pub fn toggled_by_click(self) -> Self {
        match self {
            Self::Empty => Self::Done,
            Self::Done | Self::Progress | Self::Blocked => Self::Empty,
        }
    }
}

pub struct ViewerCommandFactory;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HostCommand {
    CopyText(CopyTextCommand),
    OpenUri(String),
    RevealPath(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CopyTextCommand {
    pub source: CopyTextSource,
    pub target: ViewerTarget,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CopyTextSource {
    Code,
    DiagramSource,
}
