use super::OfficeStaticViewerSession;

pub(super) fn office_item_sizes(session: &OfficeStaticViewerSession) -> Vec<(f32, f32)> {
    session
        .artifact()
        .items
        .iter()
        .map(|item| (item.width, item.height))
        .collect()
}
