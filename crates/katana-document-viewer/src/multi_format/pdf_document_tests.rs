use super::{PdfDocumentBuilder, PdfPageRotation};
use hayro::hayro_syntax::page::Rotation;

#[test]
fn rotations_map_to_typed_pdf_semantics() {
    let cases = [
        (Rotation::None, PdfPageRotation::None),
        (Rotation::Horizontal, PdfPageRotation::Clockwise90),
        (Rotation::Flipped, PdfPageRotation::Clockwise180),
        (Rotation::FlippedHorizontal, PdfPageRotation::Clockwise270),
    ];
    for (rotation, expected) in cases {
        assert_eq!(expected, PdfDocumentBuilder::rotation(rotation));
    }
}
