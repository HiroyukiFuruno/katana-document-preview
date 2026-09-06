#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
just_bin="${JUST:-just}"

cd "$repo_root"

expanded_recipe="$("$just_bin" --dry-run storybook 2>&1)"
score_recipe="$("$just_bin" --dry-run storybook-score-check 2>&1)"
storybook_check_recipe="$("$just_bin" --dry-run storybook-check 2>&1)"

require_recipe_contains() {
  local recipe_name="$1"
  local recipe="$2"
  local expected="$3"
  local message="$4"

  case "$recipe" in
    *"$expected"*)
      ;;
    *)
      echo "$message" >&2
      echo "$recipe_name:" >&2
      echo "$recipe" >&2
      exit 1
      ;;
  esac
}

recipe_contains() {
  local recipe="$1"
  local expected="$2"

  case "$recipe" in
    *"$expected"*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

storybook_check_runs_full_kdv_storybook_suite() {
  recipe_contains "$storybook_check_recipe" "cargo test -p kdv-storybook --locked"
}

storybook_check_runs_kuc_visual_suite() {
  recipe_contains "$storybook_check_recipe" "storybook-kuc-smoke.sh" &&
    recipe_contains "$storybook_check_recipe" "test -p kdv-storybook --locked -- --test-threads=1"
}

storybook_check_suite_covers_required() {
  local required="$1"

  case "$required" in
    row_render_and_hit_collector_share_row_layout_contract|tree_canvas_draws_hover_row_background_from_tree_hovered_id|tree_canvas_draws_tree_view_row_hover_border_from_kuc_contract|generic_button_hover_draws_kuc_interactive_preset_border|generic_toggle_hover_draws_kuc_interactive_preset_border|generic_checkbox_hover_draws_kuc_interactive_preset_border|settings_toggle_control_hover_draws_kuc_interactive_preset_border|settings_section_header_hover_draws_kuc_interactive_preset_border|generic_text_hover_draws_kuc_hover_background_before_text|icon_variant_button_keeps_transparent_base_on_tree_canvas|checkbox_mark_uses_katana_task_state_value|checked_task_checkbox_uses_selection_fill_and_light_mark|empty_task_checkbox_keeps_empty_box|explicit_checkbox_height_keeps_katana_icon_box_size|tree_canvas_renders_context_menu_node_and_returns_item_hit|collects_checkbox_action_rect_with_kuc_cursor|collects_task_row_action_rect_without_expanding_checkbox_hit|collects_accordion_header_action_rect_with_kuc_cursor)
      storybook_check_runs_kuc_visual_suite && return 0
      ;;
    sidebar_*|scrolled_file_tree_*|rendered_file_tree_row_center*|storybook_window_file_tree_*)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      recipe_contains "$storybook_check_recipe" "sidebar -- --test-threads=1" && return 0
      recipe_contains "$storybook_check_recipe" "file_tree -- --test-threads=1" && return 0
      ;;
    settings_*|storybook_window_settings_*)
      case "$required" in
        storybook_window_settings_*|sidebar_mode_canvas_click_rebuilds_scene_as_slideshow|sidebar_theme_canvas_click_rebuilds_scene_as_light)
          storybook_check_runs_full_kdv_storybook_suite && return 0
          ;;
      esac
      recipe_contains "$storybook_check_recipe" "settings -- --test-threads=1" && return 0
      recipe_contains "$storybook_check_recipe" "window -- --test-threads=1" && return 0
      ;;
    window_coordinates*|window_input_uses_kuc_core_mouse_normalizer_for_canvas_space|hit_rect_center*|sidebar_hit_accepts_rendered_toggle_track_bounds_at_retina_scale)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      ;;
    hover_*|link_hover_*|accordion_header_hover_*|accordion_hover_*|media_control_hover_uses_pointer_cursor|every_diagram_control_hover_draws_kuc_preset_border|storybook_window_link_hover_*|storybook_window_paragraph_link_*|storybook_window_list_link_*|storybook_window_media_control_hover_*|storybook_window_code_copy_hover_*|storybook_window_accordion_hover_*)
      case "$required" in
        hover_*|link_hover_*|accordion_header_hover_*|accordion_hover_*|media_control_hover_uses_pointer_cursor|every_diagram_control_hover_draws_kuc_preset_border|storybook_window_*)
          storybook_check_runs_full_kdv_storybook_suite && return 0
          ;;
      esac
      recipe_contains "$storybook_check_recipe" "hover -- --test-threads=1" && return 0
      ;;
    scrolled_katana_sample_frame_changes_preview_without_overdrawing_shell|bottom_scroll_*|same_scroll_*|scroll_tests*|preview_scroll_*|window_bottom_scroll_*|window_scroll_to_bottom_*)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      recipe_contains "$storybook_check_recipe" "scroll -- --test-threads=1" && return 0
      ;;
    scene_resize_*|scene_viewport_matches_rendered_preview_content_area|resized_loaded_diagram_scene_reuses_cached_artifacts_without_pending_reload|window_resize_width_preserves_bottom_anchor_when_already_at_tail)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      recipe_contains "$storybook_check_recipe" "resize -- --test-threads=1" && return 0
      recipe_contains "$storybook_check_recipe" "resized_loaded_diagram_scene_reuses_cached_artifacts_without_pending_reload" && return 0
      ;;
    image_*|direct_raster_image_*|mouse_left_click_on_image_control_returns_image_command|direct_image_*|image_control_*)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      recipe_contains "$storybook_check_recipe" "image -- --test-threads=1" && return 0
      ;;
    code_*|storybook_window_code_*|mouse_left_click_on_code_copy_returns_host_copy_command)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      recipe_contains "$storybook_check_recipe" "code -- --test-threads=1" && return 0
      recipe_contains "$storybook_check_recipe" "window -- --test-threads=1" && return 0
      ;;
    checkbox_*|checked_task_*|empty_task_*|explicit_checkbox_*|collects_checkbox_*|collects_task_*)
      recipe_contains "$storybook_check_recipe" "checkbox -- --test-threads=1" && return 0
      ;;
    task_*|katana_task_*|storybook_window_task_*|mouse_left_click_on_task_*|mouse_task_context_menu_selection_sets_task_state|sidebar_state_*task*)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      recipe_contains "$storybook_check_recipe" "task -- --test-threads=1" && return 0
      ;;
    details_html_without_open_attribute_renders_closed_accordion|accordion_*|collects_accordion_*|direct_html_source_keeps_details_as_accordion_node|direct_html_fixture_reaches_alignment_link_table_and_accordion_nodes|storybook_window_accordion_*|storybook_window_closed_accordion_click_opens_body_pixels)
      recipe_contains "$storybook_check_recipe" "accordion -- --test-threads=1" && return 0
      ;;
    loader_*|diagram_asset_cache_*|diagram_disk_cache_*)
      recipe_contains "$storybook_check_recipe" "asset_loader -- --test-threads=1" && return 0
      ;;
    math_scene_*|direct_diagram_*|diagram_controls_*|diagram_control_*|media_control_toggle_changes_scene_and_frame_pixels|media_control_hover_*|mouse_left_click_on_every_diagram_control_returns_command|mouse_left_click_on_diagram_copy_source_uses_top_overlay_control|diagram_command_*|pending_asset_spinner_animates_between_frames|loaded_asset_scene_cache_separates_dark_and_light_diagram_theme|loaded_diagram_scene_*|katana_sample_diagrams_assets_finish_incrementally)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      recipe_contains "$storybook_check_recipe" "diagram -- --test-threads=1" && return 0
      recipe_contains "$storybook_check_recipe" "media_control -- --test-threads=1" && return 0
      ;;
    escape_closes_window_only_outside_slideshow_mode|scene_rebuild_recomputes_slideshow_pages_for_new_viewport|sidebar_mode_setting_rebuilds_scene_as_slideshow|slideshow_page_scroll_updates_scene_state_without_scene_rebuild)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      recipe_contains "$storybook_check_recipe" "slideshow -- --test-threads=1" && return 0
      ;;
    asset_job_*|storybook_loaded_diagram_scene_writes_svg_to_physical_cache_root)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      recipe_contains "$storybook_check_recipe" "asset_job -- --test-threads=1" && return 0
      ;;
    slideshow_*|render_engine_uses_document_surface_for_slideshow_mode|katana_fixture_runs_document_slideshow_toc_search_and_diagram_controls|reconfigure_keeps_surface_when_switching_to_slideshow|preview_build_with_slideshow_mode_reaches_scene_state|preview_config_hides_code_copy_controls_in_slideshow_like_katana|sidebar_state_shows_mode_and_human_slide_index)
      case "$required" in
        slideshow_*|katana_fixture_runs_document_slideshow_toc_search_and_diagram_controls|reconfigure_keeps_surface_when_switching_to_slideshow|preview_build_with_slideshow_mode_reaches_scene_state|preview_config_hides_code_copy_controls_in_slideshow_like_katana|sidebar_state_shows_mode_and_human_slide_index)
          storybook_check_runs_full_kdv_storybook_suite && return 0
          ;;
      esac
      recipe_contains "$storybook_check_recipe" "slideshow -- --test-threads=1" && return 0
      ;;
    renderer_preserves_color_pixels_for_os_emoji|emoji_text_uses_os_emoji_family|text_node_preserves_emoji_span_render_contract)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      ;;
    emoji_span_preserves_color_pixels|from_markdown_splits_raw_emoji_for_os_emoji_rendering)
      recipe_contains "$storybook_check_recipe" "katana-document-viewer --locked emoji -- --test-threads=1" && return 0
      ;;
    storybook_frame_preserves_os_color_emoji_pixels|katana_sample_basic_special_characters_preserve_os_color_emoji_pixels)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      recipe_contains "$storybook_check_recipe" "kdv-storybook --locked emoji -- --test-threads=1" && return 0
      ;;
    preview_interaction_command_matrix|preview_interaction_command_metadata)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      recipe_contains "$storybook_check_recipe" "preview_interaction_command_ -- --test-threads=1" && return 0
      ;;
    mouse*|interaction_tests*|scroll_tests*|sidebar_tests*|task_state_tests*|search*)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      ;;
    storybook_score*|storybook_preview_crop*|storybook_frame_matches_export_surface_for_katana_viewer*)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      ;;
    preview_*|required_kuc_roles_reach_fixture_frame_pixels|katana_alert_scene_keeps_title_body_and_kind_contract|direct_diagram*|loaded_asset_scene_cache_separates_dark_and_light_diagram_theme)
      storybook_check_runs_full_kdv_storybook_suite && return 0
      ;;
  esac

  return 1
}

require_storybook_check_evidence() {
  local required="$1"

  # The locked KDV Storybook library suite is the public registry-consumer
  # authority for every host-frame contract.  It supersedes tests that used
  # to run against KUC's unpublished implementation crate.
  if storybook_check_runs_full_kdv_storybook_suite; then
    return
  fi

  if recipe_contains "$storybook_check_recipe" "$required"; then
    return
  fi

  if storybook_check_suite_covers_required "$required"; then
    return
  fi

  echo "just storybook-check must include the boundary evidence gate: $required" >&2
  echo "storybook-check:" >&2
  echo "$storybook_check_recipe" >&2
  exit 1
}

case "$expanded_recipe" in
  *" run --release --locked -p kdv-storybook -- --interactive "*)
    ;;
  *)
    echo "just storybook must launch the interactive KUC Storybook binary." >&2
    echo "$expanded_recipe" >&2
    exit 1
    ;;
esac

case "$expanded_recipe" in
  *"storybook-kuc-smoke.sh"* | *" cargo test "* | *" test -p "*)
    echo "just storybook must not be a smoke/test-only entrypoint." >&2
    echo "$expanded_recipe" >&2
    exit 1
    ;;
esac

require_recipe_contains \
  "storybook-score-check" \
  "$score_recipe" \
  "fixture_score_matrix" \
  "just storybook-score-check must run the Storybook score gate tests."

require_recipe_contains \
  "storybook-score-check" \
  "$score_recipe" \
  "frame::test_modules::surface_parity_tests::storybook_frame_matches_export_surface_for_katana_viewer -- --ignored --exact --test-threads=1" \
  "just storybook-score-check must run exact real Storybook surface parity tests."

require_recipe_contains \
  "storybook-score-check" \
  "$score_recipe" \
  "frame::test_modules::surface_parity_tests::storybook_frame_matches_export_surface_for_katana_viewer_diagrams -- --ignored --exact --test-threads=1" \
  "just storybook-score-check must run exact diagram-heavy Storybook surface parity tests."

require_recipe_contains \
  "storybook-score-check" \
  "$score_recipe" \
  "storybook_score_visual_uses_katana_preview_crop_reference" \
  "just storybook-score-check must run the KatanA preview crop visual score gate."

require_recipe_contains \
  "storybook-score-check" \
  "$score_recipe" \
  "storybook_score_visual_uses_katana_sample_diagrams_crop_reference" \
  "just storybook-score-check must run the KatanA sample diagrams crop visual score gate."

require_recipe_contains \
  "storybook-score-check" \
  "$score_recipe" \
  "storybook_preview_crop_score_uses_scaled_canvas_pixels" \
  "just storybook-score-check must run the scaled canvas visual score gate."

require_recipe_contains \
  "storybook-score-check" \
  "$score_recipe" \
  "storybook_score_visual_uses_katana_export_png_reference" \
  "just storybook-score-check must run the KatanA export PNG visual score gate."

require_recipe_contains \
  "storybook-score-check" \
  "$score_recipe" \
  "storybook_score_audit" \
  "just storybook-score-check must run broken UI audit tests."

require_recipe_contains \
  "storybook-score-check" \
  "$score_recipe" \
  "storybook-window-diagram-screenshot-smoke" \
  "just storybook-score-check must run real window diagram control screenshot evidence."

require_recipe_contains \
  "storybook-score-check" \
  "$score_recipe" \
  "storybook-window-drawio-diagram-screenshot-smoke" \
  "just storybook-score-check must run real window drawio diagram control screenshot evidence."

for required in \
  "sidebar_file_canvas_click_rebuilds_viewer_scene_for_selected_fixture" \
  "scrolled_file_tree_hover_then_click_matrix_selects_visible_files" \
  "storybook_window_file_tree_hover_draws_kuc_row_background" \
  "storybook_window_settings_toggle_click_uses_kuc_action_target_center" \
  "storybook_window_settings_toggle_click_uses_kuc_action_target_edges" \
  "storybook_window_settings_toggle_hover_draws_kuc_preset_border" \
  "settings_section_header_hover_draws_kuc_interactive_preset_border" \
  "storybook_window_settings_section_header_hover_and_click_use_kuc_action_target" \
  "storybook_window_settings_section_header_e2e_toggle_matrix" \
  "storybook_window_settings_section_header_edge_click_matrix" \
  "mouse_point_and_surface_must_share_the_same_coordinate_space" \
  "row_render_and_hit_collector_share_row_layout_contract" \
  "window_coordinates -- --test-threads=1" \
  "window_input_uses_kuc_core_mouse_normalizer_for_canvas_space" \
  "hit_rect_center -- --test-threads=1" \
  "sidebar_hit_accepts_rendered_toggle_track_bounds_at_retina_scale" \
  "rendered_file_tree_row_center" \
  "storybook_window_paragraph_link_hover_and_click_use_same_kuc_action_rect" \
  "storybook_window_list_link_hover_and_click_use_same_kuc_action_rect" \
  "storybook_window_accordion_hover_click_open_and_close_use_same_kuc_action_rect" \
  "storybook_window_task_click_rebuilds_scene_with_external_state_override" \
  "storybook_window_media_control_hover_draws_interactive_preset_border" \
  "with_hovered_node_id" \
  "tree_canvas_draws_hover_row_background_from_tree_hovered_id" \
  "tree_canvas_draws_tree_view_row_hover_border_from_kuc_contract" \
  "generic_button_hover_draws_kuc_interactive_preset_border" \
  "generic_toggle_hover_draws_kuc_interactive_preset_border" \
  "generic_checkbox_hover_draws_kuc_interactive_preset_border" \
  "settings_toggle_control_hover_draws_kuc_interactive_preset_border" \
  "settings_section_header_hover_draws_kuc_interactive_preset_border" \
  "generic_text_hover_draws_kuc_hover_background_before_text" \
  "hover_resolves_viewer_target -- --test-threads=1" \
  "hover_draws_block_hover_surface -- --test-threads=1" \
  "hover_highlight_covers_common_markdown_block_kinds" \
  "hover_highlight_reaches_storybook_frame_pixels" \
  "hover_highlight_covers_scrolled_target_block_row" \
  "hover_highlight_clips_scrolled_target_top_edge" \
  "sidebar_hover_uses_pointer_cursor_for_kuc_tree_hit" \
  "link_hover_uses_pointer_cursor_from_kuc_node" \
  "accordion_header_hover_uses_pointer_cursor" \
  "media_control_hover_uses_pointer_cursor" \
  "every_diagram_control_hover_draws_kuc_preset_border" \
  "storybook_window_link_hover_draws_block_hover_surface" \
  "storybook_window_accordion_hover_draws_block_hover_surface" \
  "storybook_window_code_copy_hover_draws_interactive_preset_border" \
  "sidebar_mode_canvas_click_rebuilds_scene_as_slideshow" \
  "sidebar_theme_canvas_click_rebuilds_scene_as_light" \
  "icon_variant_button_keeps_transparent_base_on_tree_canvas" \
  "media_control -- --test-threads=1" \
  "preview_interaction_command_matrix" \
  "preview_interaction_command_metadata" \
  "mouse -- --test-threads=1" \
  "interaction_tests -- --test-threads=1" \
  "scroll_tests -- --test-threads=1" \
  "scrolled_katana_sample_frame_changes_preview_without_overdrawing_shell" \
  "bottom_scroll_shows_viewport_tail_space" \
  "bottom_scroll_aligns_last_target_top_before_tail_space" \
  "same_scroll_renders_identically_for_rebuilt_scroll_independent_scene" \
  "preview_scroll_updates_document_scroll_for_tall_content" \
  "scene_resize_clamps_preview_scroll_to_new_viewport_bounds" \
  "scene_viewport_matches_rendered_preview_content_area" \
  "preview_scroll_keeps_current_scene_and_asset_job_scope" \
  "window_bottom_scroll_renders_tail_space_from_storybook_state" \
  "window_scroll_to_bottom_keeps_diagram_asset_job_scope" \
  "window_resize_width_preserves_bottom_anchor_when_already_at_tail" \
  "loaded_diagram_scene_scroll_does_not_restart_or_rerender_assets" \
  "resized_loaded_diagram_scene_reuses_cached_artifacts_without_pending_reload" \
  "sidebar_tests -- --test-threads=1" \
  "task_state_tests -- --test-threads=1" \
  "search -- --test-threads=1" \
  "slideshow -- --test-threads=1" \
  "cargo run --release --locked -p kdv-storybook -- --smoke --frames 2" \
  "render_engine_keeps_diagram_fixture_inside_interactive_budget" \
  "storybook_score_audit" \
  "storybook_frame_matches_export_surface_for_katana_viewer" \
  "storybook_frame_matches_export_surface_for_katana_viewer_diagrams" \
  "storybook_window_visible_text_runs_are_individually_selectable_and_copyable" \
  "selection-screenshot-smoke" \
  "window-selection-screenshot-smoke" \
  "slideshow-screenshot-smoke" \
  "window-slideshow-screenshot-smoke" \
  "window-diagram-screenshot-smoke" \
  "renderer_preserves_color_pixels_for_os_emoji" \
  "emoji_text_uses_os_emoji_family" \
  "text_node_preserves_emoji_span_render_contract" \
  "emoji_span_preserves_color_pixels" \
  "from_markdown_splits_raw_emoji_for_os_emoji_rendering" \
  "storybook_frame_preserves_os_color_emoji_pixels" \
  "katana_sample_basic_special_characters_preserve_os_color_emoji_pixels" \
  "image_controls_have_rendered_frame_hits_for_all_actions" \
  "direct_raster_image_window_loaded_scenes_render_visible_image_surfaces" \
  "image_and_code_controls_hover_draws_kuc_preset_border" \
  "image_control_buttons_keep_transparent_base_in_dark_and_light" \
  "mouse_left_click_on_image_control_returns_image_command" \
  "image_control_command_refreshes_scene" \
  "image_control_non_reloading_commands_cover_open_copy_and_reveal" \
  "image_control_zoom_out_refreshes_scene" \
  "direct_image_asset_job_reaches_image_surface" \
  "direct_image_loaded_scene_reaches_image_surface" \
  "document_code_block -- --test-threads=1" \
  "planner_uses_export_surface_height_for_multiline_fenced_code" \
  "media_control_specs_create_host_action_ids" \
  "preview_requirement_matrix -- --test-threads=1" \
  "required_kuc_roles_reach_fixture_frame_pixels" \
  "katana_alert_scene_keeps_title_body_and_kind_contract" \
  "code_copy_control_does_not_render_blue_filled_button_in_storybook_frame" \
  "storybook_window_code_copy_click_returns_host_copy_from_visible_button" \
  "storybook_window_code_copy_hover_draws_interactive_preset_border" \
  "mouse_left_click_on_code_copy_returns_host_copy_command" \
  "code_copy_host_command_refreshes_scene" \
  "checkbox_mark_uses_katana_task_state_value" \
  "checked_task_checkbox_uses_selection_fill_and_light_mark" \
  "empty_task_checkbox_keeps_empty_box" \
  "explicit_checkbox_height_keeps_katana_icon_box_size" \
  "tree_canvas_renders_context_menu_node_and_returns_item_hit" \
  "collects_checkbox_action_rect_with_kuc_cursor" \
  "collects_task_row_action_rect_without_expanding_checkbox_hit" \
  "katana_task_checkbox_states_reach_scene_and_frame" \
  "task_click_renders_same_checkbox_crop_as_kuc_component" \
  "storybook_window_task_click_rebuilds_scene_with_external_state_override" \
  "storybook_window_task_context_menu_selects_marker_through_external_state" \
  "storybook_window_task_context_menu_selects_every_marker_through_external_state" \
  "task_command_updates_kuc_checkbox_state" \
  "task_session_state_records_external_state_transition" \
  "task_session_state_clears_when_fixture_state_resets" \
  "sidebar_state_shows_task_session_changes" \
  "sidebar_state_keeps_multiple_task_change_locations" \
  "mouse_left_click_on_task_checkbox_toggles_task" \
  "mouse_left_click_on_task_row_body_toggles_task" \
  "mouse_task_context_menu_selection_sets_task_state" \
  "accordion_text_action_exposes_requested_open_without_consumer_inversion" \
  "details_html_without_open_attribute_renders_closed_accordion" \
  "accordion_open_override_toggles_rendered_state" \
  "accordion_exposes_leaf_presets_options_and_toggle_contract" \
  "accordion_story_exposes_presets_settings_and_logs" \
  "collects_accordion_header_action_rect_with_kuc_cursor" \
  "direct_html_source_keeps_details_as_accordion_node" \
  "direct_html_fixture_reaches_alignment_link_table_and_accordion_nodes" \
  "accordion_click_updates_kuc_state_override" \
  "accordion_header_hover_uses_pointer_cursor" \
  "accordion_hover_resolves_viewer_target_for_block_hover" \
  "accordion_hit_rect_center_click_resolves_toggle" \
  "storybook_window_accordion_hover_draws_block_hover_surface" \
  "storybook_window_accordion_hover_click_open_and_close_use_same_kuc_action_rect" \
  "storybook_window_accordion_click_updates_body_visibility_pixels" \
  "storybook_window_closed_accordion_click_opens_body_pixels" \
  "no_reintroduced_manual_storybook_action_contracts" \
  "katana_diagram_fixture_reaches_all_viewer_diagram_kinds" \
  "loader_materializes_visible_diagram_asset" \
  "loader_records_diagram_error_artifact" \
  "diagram_asset_cache_skips_repeated_engine_render_for_same_source_theme" \
  "diagram_asset_cache_renders_again_when_theme_changes" \
  "diagram_disk_cache_survives_memory_cache_clear" \
  "diagram_disk_cache_key_includes_document_path" \
  "visible_asset_load_parallel_starts_visible_diagrams_together" \
  "math_scene_does_not_emit_image_or_diagram_controls" \
  "direct_diagram_fixtures_reach_kuc_image_surface" \
  "diagram_controls_follow_katana_top_and_grid_layout" \
  "every_diagram_control_hover_draws_kuc_preset_border" \
  "diagram_control_buttons_keep_transparent_base_in_dark_and_light" \
  "media_control_toggle_changes_scene_and_frame_pixels" \
  "media_control_hover_reaches_kuc_interactive_preset_border_pixels" \
  "diagram_control_hit_rect_center_click_matches_drawn_button" \
  "mouse_left_click_on_every_diagram_control_returns_command" \
  "mouse_left_click_on_diagram_copy_source_uses_top_overlay_control" \
  "storybook_window_every_diagram_control_click_dispatches_from_kuc_hit" \
  "storybook_window_fullscreen_diagram_overlay_controls_dispatch_from_kuc_hits" \
  "storybook_window_diagram_control_click_repaints_viewer_frame" \
  "storybook_window_diagram_controls_survive_continuous_kuc_hit_sequence_without_asset_reload" \
  "diagram_command_updates_viewport_state" \
  "diagram_command_refreshes_loaded_scene_without_asset_job" \
  "asset_job_streams_partial_scene_before_completion" \
  "asset_job_result_updates_scene_and_clears_pending_count" \
  "preview_scroll_keeps_pending_asset_job_identity" \
  "pending_asset_spinner_animates_between_frames" \
  "loaded_asset_scene_cache_separates_dark_and_light_diagram_theme" \
  "storybook_loaded_diagram_scene_writes_svg_to_physical_cache_root" \
  "katana_sample_diagrams_assets_finish_incrementally" \
  "resized_loaded_diagram_scene_reuses_cached_artifacts_without_pending_reload" \
  "window_scroll_to_bottom_keeps_diagram_asset_job_scope" \
  "fixture_switch_pending_first_frame_completes_loaded_diagram_assets" \
  "slideshow_controls_can_be_hidden_shown_and_zero_height_is_single_page" \
  "slideshow_state_does_not_define_dedicated_theme" \
  "render_engine_uses_document_surface_for_slideshow_mode" \
  "slideshow_navigation_clamps_to_virtual_pages" \
  "slideshow_settings_and_control_visibility_are_stateful" \
  "slideshow_snapshot_uses_scroll_position_as_current_page" \
  "slideshow_page_index_for_scroll_uses_floor_pages" \
  "slideshow_controls_become_slideshow_commands" \
  "katana_fixture_runs_document_slideshow_toc_search_and_diagram_controls" \
  "reconfigure_keeps_surface_when_switching_to_slideshow" \
  "slideshow_controls_reach_kdv_slideshow_commands" \
  "preview_build_with_slideshow_mode_reaches_scene_state" \
  "slideshow_mode_hides_code_copy_controls_like_katana" \
  "preview_config_hides_code_copy_controls_in_slideshow_like_katana" \
  "slideshow_mode_reaches_storybook_frame_pixels" \
  "slideshow_next_button_click_advances_page_without_scene_rebuild" \
  "slideshow_previous_button_click_works_after_page_scroll" \
  "slideshow_close_button_click_works_after_page_scroll" \
  "escape_closes_window_only_outside_slideshow_mode" \
  "scene_rebuild_recomputes_slideshow_pages_for_new_viewport" \
  "sidebar_mode_setting_rebuilds_scene_as_slideshow" \
  "sidebar_mode_canvas_click_rebuilds_scene_as_slideshow" \
  "slideshow_page_scroll_updates_scene_state_without_scene_rebuild" \
  "slideshow_keymap_matches_katana_spec_inputs" \
  "slideshow_next_and_previous_change_page_scroll" \
  "slideshow_spec_keys_change_pages_and_close_mode" \
  "slideshow_page_height_uses_preview_viewport_not_header_only_window_height" \
  "slideshow_next_page_scrolls_by_storybook_preview_viewport_height" \
  "sidebar_state_shows_mode_and_human_slide_index"
do
  require_storybook_check_evidence "$required"
done

echo "storybook-entrypoint-check: ok"
