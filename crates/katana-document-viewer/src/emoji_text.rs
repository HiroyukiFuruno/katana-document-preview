pub(crate) use katana_ui_core::render_model::UiEmojiTextSegments as EmojiTextSegments;

#[cfg(test)]
mod tests {
    use super::EmojiTextSegments;

    #[test]
    fn split_marks_raw_emoji_runs_without_marking_surrounding_text() {
        let segments = EmojiTextSegments::split("Emoji: 🦀 text ⚠️");

        assert_eq!(
            segments
                .iter()
                .map(|segment| (segment.text.as_str(), segment.emoji))
                .collect::<Vec<_>>(),
            vec![
                ("Emoji: ", false),
                ("🦀", true),
                (" text ", false),
                ("⚠️", true)
            ]
        );
    }

    #[test]
    fn split_keeps_star_variation_selector_as_one_emoji_run() {
        let segments = EmojiTextSegments::split("Star ⭐️ mark");

        assert_eq!(
            segments
                .iter()
                .map(|segment| (segment.text.as_str(), segment.emoji))
                .collect::<Vec<_>>(),
            vec![("Star ", false), ("⭐️", true), (" mark", false)]
        );
    }
}
