use super::{SpreadsheetEngineError, SpreadsheetEngineSession, SpreadsheetEngineSupport};
use crate::multi_format::SpreadsheetViewerLimits;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

#[test]
fn engine_support_rejects_invalid_sizes_and_indices() {
    assert_eq!(0.0, SpreadsheetEngineSupport::track_size(f64::NAN));
    assert_eq!(0.0, SpreadsheetEngineSupport::track_size(-1.0));
    assert!(matches!(
        SpreadsheetEngineSupport::check_limit("cells", 2, 1),
        Err(SpreadsheetEngineError::ResourceLimit { .. })
    ));
    assert!(SpreadsheetEngineSupport::zero_based(0).is_err());
    assert!(SpreadsheetEngineSupport::engine_index(usize::MAX).is_err());
}

#[test]
fn engine_support_preserves_external_error_context() {
    let conversion = usize::try_from(-1_i32);
    assert!(conversion.is_err());
    if let Err(error) = conversion {
        assert!(matches!(
            SpreadsheetEngineSupport::model_error(error),
            SpreadsheetEngineError::Model(_)
        ));
    }
    assert!(matches!(
        SpreadsheetEngineSupport::engine_error("engine failed".to_owned()),
        SpreadsheetEngineError::Model(_)
    ));
}

#[test]
fn engine_import_rejects_a_structurally_incomplete_workbook()
-> Result<(), Box<dyn std::error::Error>> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in [
        (
            "xl/workbook.xml",
            br#"<workbook xmlns:r="urn:r"><sheets><sheet name="Data" r:id="r1"/></sheets></workbook>"#.as_slice(),
        ),
        (
            "xl/_rels/workbook.xml.rels",
            br#"<Relationships><Relationship Id="r1" Target="worksheets/sheet1.xml"/></Relationships>"#.as_slice(),
        ),
        (
            "xl/worksheets/sheet1.xml",
            br#"<worksheet><sheetData/></worksheet>"#.as_slice(),
        ),
    ] {
        writer.start_file(name, SimpleFileOptions::default())?;
        writer.write_all(bytes)?;
    }
    let bytes = writer.finish()?.into_inner();
    assert!(matches!(
        SpreadsheetEngineSession::open(bytes, "incomplete.xlsx", SpreadsheetViewerLimits::strict(),),
        Err(SpreadsheetEngineError::Import(_))
    ));
    Ok(())
}
