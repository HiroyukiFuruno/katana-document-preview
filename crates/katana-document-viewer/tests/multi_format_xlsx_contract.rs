use katana_document_viewer::{
    OfficeDocumentFormat, OfficeDocumentSource, OfficeWorkerConfig, OfficeWorkerError,
    SpreadsheetCellValue, SpreadsheetCoordinate, SpreadsheetHorizontalAlignment,
    SpreadsheetViewerSession, ViewerDiagnosticCode, ViewerFeature, ViewerFeatureStatus,
    ViewerSourceIdentity,
};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn fixture(name: &str) -> TestResult<OfficeDocumentSource> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format")
        .join(name);
    Ok(OfficeDocumentSource::new(
        ViewerSourceIdentity::new(format!("file:///fixtures/{name}"), format!("sha256:{name}")),
        OfficeDocumentFormat::Xlsx,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        std::fs::read(path)?,
    ))
}

fn worker_config() -> OfficeWorkerConfig {
    OfficeWorkerConfig::new(PathBuf::from(env!("CARGO_BIN_EXE_kdv-office-worker")))
}

#[cfg(windows)]
fn external_worker_config() -> TestResult<(tempfile::TempDir, OfficeWorkerConfig)> {
    let directory = tempfile::tempdir()?;
    let nested = directory.path().join("external").join("release");
    std::fs::create_dir_all(&nested)?;
    let executable = nested.join("kdv-office-worker.exe");
    std::fs::copy(env!("CARGO_BIN_EXE_kdv-office-worker"), &executable)?;
    Ok((directory, OfficeWorkerConfig::new(executable)))
}

fn preflight_valid_invalid_xlsx() -> TestResult<OfficeDocumentSource> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer.start_file(
        "xl/workbook.xml",
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
    )?;
    writer.write_all(b"<invalid/>")?;
    let bytes = writer.finish()?.into_inner();
    Ok(OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///fixtures/invalid-engine.xlsx", "invalid-engine"),
        OfficeDocumentFormat::Xlsx,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        bytes,
    ))
}

#[test]
fn missing_spreadsheet_worker_is_a_typed_failure_without_in_process_fallback() -> TestResult {
    let result = SpreadsheetViewerSession::open(
        fixture("representative.xlsx")?,
        OfficeWorkerConfig::new(PathBuf::from("/missing/kdv-office-worker")),
    );
    let error = match result {
        Err(error) => error,
        Ok(_) => return Err("missing worker did not fail closed".into()),
    };
    assert!(matches!(error, OfficeWorkerError::WorkerUnavailable { .. }));
    Ok(())
}

#[cfg(windows)]
#[test]
fn windows_appcontainer_stages_an_external_worker_before_launch() -> TestResult {
    let (_directory, config) = external_worker_config()?;
    let mut session = SpreadsheetViewerSession::open(fixture("representative.xlsx")?, config)?;
    let cells = session.materialize_cells(0, vec![SpreadsheetCoordinate::new(0, 0)])?;

    assert_eq!("Quarterly performance", cells[0].display_text);
    Ok(())
}

#[test]
fn xlsx_worker_exposes_neutral_sheet_geometry_and_capabilities() -> TestResult {
    let session = SpreadsheetViewerSession::open(fixture("representative.xlsx")?, worker_config())?;
    assert!(format!("{session:?}").contains("SpreadsheetViewerSession"));
    let artifact = session.artifact();
    let dashboard = &artifact.sheets[0];

    assert_eq!(2, artifact.sheet_count);
    assert_eq!("Dashboard", dashboard.name);
    assert_eq!((7, 6), (dashboard.row_count, dashboard.column_count));
    assert_eq!((3, 0), (dashboard.frozen_rows, dashboard.frozen_columns));
    assert_eq!(7, dashboard.row_tracks.len());
    assert_eq!(6, dashboard.column_tracks.len());
    assert!(!dashboard.show_grid_lines);
    assert_eq!(
        SpreadsheetCoordinate::new(0, 0),
        dashboard.merged_cells[0].anchor
    );
    assert_eq!((1, 6), {
        let merged = dashboard.merged_cells[0];
        (merged.row_span, merged.column_span)
    });
    assert_eq!(
        ViewerFeatureStatus::Supported,
        artifact.capabilities.status(ViewerFeature::FormulaValue)
    );
    for feature in [
        ViewerFeature::Chart,
        ViewerFeature::PivotTable,
        ViewerFeature::PrintLayout,
    ] {
        assert!(artifact.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ViewerDiagnosticCode::UnsupportedFeature
                && diagnostic.feature == Some(feature)
        }));
    }
    Ok(())
}

#[test]
fn xlsx_worker_materializes_formula_style_and_conditional_formatting() -> TestResult {
    let mut session =
        SpreadsheetViewerSession::open(fixture("representative.xlsx")?, worker_config())?;
    let cells = session.materialize_cells(
        0,
        vec![
            SpreadsheetCoordinate::new(0, 0),
            SpreadsheetCoordinate::new(3, 0),
            SpreadsheetCoordinate::new(3, 4),
        ],
    )?;
    let title = &cells[0];
    let formula = &cells[2];

    assert_eq!("Quarterly performance", title.display_text);
    assert!(title.style.bold);
    assert_eq!(Some("#FFFFFF".to_owned()), title.style.font_color);
    assert_eq!(Some("#183B66".to_owned()), title.style.fill_color);
    assert_eq!(
        SpreadsheetHorizontalAlignment::Center,
        title.style.horizontal_alignment
    );
    assert_eq!(
        SpreadsheetCellValue::Text("North".to_owned()),
        cells[1].value
    );
    assert_eq!(Some("=SUM(B4:D4)".to_owned()), formula.formula);
    assert_eq!(SpreadsheetCellValue::Number(400.0), formula.value);
    assert_eq!("400", formula.display_text);
    assert!(formula.conditional_formatting.applied);
    assert!(formula.style.fill_color.is_some());
    assert_eq!(3, cells.len());
    let empty = session.materialize_cells(0, vec![SpreadsheetCoordinate::new(0, 1)])?;
    assert_eq!(SpreadsheetCellValue::Empty, empty[0].value);
    Ok(())
}

#[test]
fn xlsx_worker_bounds_materialization_without_full_grid_expansion() -> TestResult {
    let mut config = worker_config();
    config.spreadsheet_limits.max_materialized_cells = 2;
    let mut session = SpreadsheetViewerSession::open(fixture("stress-100k-cells.xlsx")?, config)?;
    let artifact = session.artifact();
    let sheet = &artifact.sheets[0];

    assert!(sheet.row_count * sheet.column_count >= 100_000);
    let cells = session.materialize_cells(
        0,
        vec![
            SpreadsheetCoordinate::new(0, 0),
            SpreadsheetCoordinate::new(sheet.row_count - 1, sheet.column_count - 1),
        ],
    )?;
    assert_eq!(2, cells.len());
    assert!(matches!(
        session.materialize_cells(
            0,
            vec![
                SpreadsheetCoordinate::new(0, 0),
                SpreadsheetCoordinate::new(0, 1),
                SpreadsheetCoordinate::new(0, 2),
            ],
        ),
        Err(OfficeWorkerError::EngineFailure {
            ref stage,
            ref message,
        }) if stage == "spreadsheet" && message.contains("materialized_cell_count")
    ));
    Ok(())
}

#[test]
fn xlsx_worker_rejects_duplicate_outside_and_wrong_format_requests() -> TestResult {
    let mut session =
        SpreadsheetViewerSession::open(fixture("representative.xlsx")?, worker_config())?;
    let duplicate = SpreadsheetCoordinate::new(0, 0);

    for coordinates in [
        vec![duplicate, duplicate],
        vec![SpreadsheetCoordinate::new(7, 0)],
    ] {
        assert!(matches!(
            session.materialize_cells(0, coordinates),
            Err(OfficeWorkerError::EngineFailure { .. })
        ));
    }
    assert!(matches!(
        session.materialize_cells(2, vec![duplicate]),
        Err(OfficeWorkerError::EngineFailure { .. })
    ));
    let mut docx = fixture("representative.xlsx")?;
    docx.format = OfficeDocumentFormat::Docx;
    assert!(matches!(
        SpreadsheetViewerSession::open(docx, worker_config()),
        Err(OfficeWorkerError::UnsupportedFormat(
            OfficeDocumentFormat::Docx
        ))
    ));
    Ok(())
}

#[test]
fn xlsx_worker_rejects_engine_import_and_workbook_resource_limits() -> TestResult {
    assert!(matches!(
        SpreadsheetViewerSession::open(preflight_valid_invalid_xlsx()?, worker_config()),
        Err(OfficeWorkerError::EngineFailure {
            ref stage,
            ..
        }) if stage == "spreadsheet_open"
    ));

    let mut sheet_limit = worker_config();
    sheet_limit.spreadsheet_limits.max_sheets = 1;
    assert!(matches!(
        SpreadsheetViewerSession::open(fixture("representative.xlsx")?, sheet_limit),
        Err(OfficeWorkerError::EngineFailure { .. })
    ));

    let mut cell_limit = worker_config();
    cell_limit.spreadsheet_limits.max_logical_cells = 1;
    assert!(matches!(
        SpreadsheetViewerSession::open(fixture("representative.xlsx")?, cell_limit),
        Err(OfficeWorkerError::EngineFailure { .. })
    ));
    Ok(())
}
