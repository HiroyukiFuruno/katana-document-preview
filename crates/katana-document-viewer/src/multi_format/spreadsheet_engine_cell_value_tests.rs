use super::{CellValue, SpreadsheetCellMaterializer, SpreadsheetCellValue};

#[test]
fn every_cell_value_maps_to_neutral_types() {
    let cases = [
        (
            CellValue::Boolean(true),
            SpreadsheetCellValue::Boolean(true),
        ),
        (
            CellValue::String("text".to_owned()),
            SpreadsheetCellValue::Text("text".to_owned()),
        ),
        (CellValue::Number(42.0), SpreadsheetCellValue::Number(42.0)),
        (CellValue::None, SpreadsheetCellValue::Empty),
    ];
    for (input, expected) in cases {
        assert_eq!(expected, SpreadsheetCellMaterializer::cell_value(input));
    }
}
