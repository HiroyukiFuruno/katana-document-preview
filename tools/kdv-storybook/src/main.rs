// 公開KUCのraster hostを唯一のStorybook描画境界として再公開する。
extern crate self as katana_ui_core_storybook;
pub use katana_ui_core::raster_host::*;

mod args;
mod canvas;
mod catalog;
mod document_viewer;
mod frame;
#[cfg(test)]
mod frame_alert_contract_tests;
mod frame_pixel_guard;
#[cfg(test)]
mod frame_preview_pixels;
#[cfg(test)]
mod frame_role_pixel_case;
mod frame_status;
#[cfg(test)]
mod frame_theme_token_tests;
mod frame_ui_surface;
mod interaction_keys;
mod layout;
mod media_host_action;
#[cfg(test)]
mod media_host_action_tests;
mod mouse;
mod palette;
mod preview;
mod preview_build_request;
mod preview_build_support;
#[cfg(test)]
mod preview_direct_diagram_matrix_tests;
#[cfg(test)]
mod preview_interaction_command_matrix_tests;
#[cfg(test)]
mod preview_interaction_command_metadata_support;
#[cfg(test)]
mod preview_interaction_command_metadata_tests;
#[cfg(test)]
mod preview_interaction_command_support;
#[cfg(test)]
mod preview_interaction_link_metadata_tests;
#[cfg(test)]
mod preview_katana_drawio_matrix_tests;
#[cfg(test)]
mod preview_katana_markdown_matrix_tests;
#[cfg(test)]
mod preview_markdown_raw_marker_tests;
#[cfg(test)]
mod preview_requirement_matrix_assertions;
#[cfg(test)]
mod preview_requirement_matrix_tests;
mod preview_scene;
mod preview_search_targets;
mod preview_theme_bridge;
mod scroll;
mod search_keys;
mod settings_action;
mod sidebar;
mod sidebar_hit;
#[cfg(test)]
#[path = "sidebar/session_state_tests.rs"]
mod sidebar_session_state_tests;
mod sidebar_settings_state;
mod sidebar_settings_stats;
mod sidebar_settings_task_change;
#[cfg(test)]
mod sidebar_test_support;
mod slideshow_keys;
mod smoke_assertions;
#[cfg(test)]
mod storybook_contract_regression_tests;
#[cfg(test)]
mod storybook_score_audit_tests;
mod task_marker_state;
#[cfg(test)]
#[allow(dead_code)]
mod test_assert;
mod window;
mod window_asset_job;
mod window_coordinates;
mod window_host_event;
#[cfg(target_os = "macos")]
mod window_macos_objc;

#[cfg(test)]
pub(crate) use document_viewer::KucViewerAdapter;
pub(crate) use document_viewer::{
    DocumentViewerStorybookHost, KucDiagramControlResolver, KucViewerConfig, KucViewerPlan,
};

mod visual {
    pub(crate) use katana_ui_core_storybook::UiTreeNodeHit;
}

use args::StorybookArgs;
use catalog::FixtureCatalog;
use preview::PreviewBuilder;
use window::StorybookWindow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = StorybookArgs::parse(std::env::args().skip(1))?;
    let catalog = FixtureCatalog::load(&args.fixture_root)?;
    let preview = PreviewBuilder::default();
    let clipboard_smoke = args.clipboard_smoke;
    let clipboard_keyboard_smoke = args.clipboard_keyboard_smoke;
    let clipboard_drag_smoke = args.clipboard_drag_smoke;
    let selection_screenshot_smoke = args.selection_screenshot_smoke;
    let window_selection_screenshot_smoke = args.window_selection_screenshot_smoke;
    let window_hover_screenshot_smoke = args.window_hover_screenshot_smoke;
    let window_footnote_screenshot_smoke = args.window_footnote_screenshot_smoke;
    let window_table_screenshot_smoke = args.window_table_screenshot_smoke;
    let window_code_copy_screenshot_smoke = args.window_code_copy_screenshot_smoke;
    let slideshow_screenshot_smoke = args.slideshow_screenshot_smoke;
    let window_slideshow_screenshot_smoke = args.window_slideshow_screenshot_smoke;
    let window_sidebar_screenshot_smoke = args.window_sidebar_screenshot_smoke;
    let window_diagram_screenshot_smoke = args.window_diagram_screenshot_smoke;
    let print_live_dark_toggle_point = args.print_live_dark_toggle_point;
    let live_acceptance_artifact = args.live_acceptance_artifact;
    let storybook = StorybookWindow::new(args, catalog, preview);
    if clipboard_smoke {
        storybook.run_clipboard_smoke()?;
    } else if clipboard_keyboard_smoke {
        storybook.run_clipboard_keyboard_smoke()?;
    } else if clipboard_drag_smoke {
        storybook.run_clipboard_drag_smoke()?;
    } else if selection_screenshot_smoke {
        storybook.run_selection_screenshot_smoke()?;
    } else if window_selection_screenshot_smoke {
        storybook.run_window_selection_screenshot_smoke()?;
    } else if window_hover_screenshot_smoke {
        storybook.run_window_hover_screenshot_smoke()?;
    } else if window_footnote_screenshot_smoke {
        storybook.run_window_footnote_screenshot_smoke()?;
    } else if window_table_screenshot_smoke {
        storybook.run_window_table_screenshot_smoke()?;
    } else if window_code_copy_screenshot_smoke {
        storybook.run_window_code_copy_screenshot_smoke()?;
    } else if slideshow_screenshot_smoke {
        storybook.run_slideshow_screenshot_smoke()?;
    } else if window_slideshow_screenshot_smoke {
        storybook.run_window_slideshow_screenshot_smoke()?;
    } else if window_sidebar_screenshot_smoke {
        storybook.run_window_sidebar_screenshot_smoke()?;
    } else if window_diagram_screenshot_smoke {
        storybook.run_window_diagram_screenshot_smoke()?;
    } else if print_live_dark_toggle_point {
        storybook.print_live_dark_toggle_point()?;
    } else if live_acceptance_artifact {
        storybook.run_live_acceptance_artifact()?;
    } else {
        storybook.run()?;
    }
    Ok(())
}
