use image::RgbaImage;
use katana_document_viewer::{
    DocumentSession, DocumentSessionCommand, DocumentSessionConfig, DocumentViewerCommand,
    DocumentViewport, OfficeDocumentFormat, OfficeDocumentSource, OfficeWorkerConfig, ViewerSource,
    ViewerSourceIdentity,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format")
        .join(name)
}

fn office_source(name: &str, format: OfficeDocumentFormat) -> TestResult<ViewerSource> {
    let mime = match format {
        OfficeDocumentFormat::Docx => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        OfficeDocumentFormat::Xlsx => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        OfficeDocumentFormat::Pptx => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        }
    };
    Ok(ViewerSource::Office(OfficeDocumentSource::new(
        ViewerSourceIdentity::new(format!("file:///fixtures/{name}"), format!("sha256:{name}")),
        format,
        mime,
        std::fs::read(fixture_path(name))?,
    )))
}

fn config(viewport: DocumentViewport) -> DocumentSessionConfig {
    let worker = std::env::var_os("KDV_FIDELITY_WORKER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_kdv-office-worker")));
    DocumentSessionConfig::new(viewport).office_worker(OfficeWorkerConfig::new(worker))
}

fn output_root() -> TestResult<PathBuf> {
    let root = PathBuf::from(std::env::var("KDV_FIDELITY_OUTPUT_DIR")?);
    if root.exists() && std::fs::read_dir(&root)?.next().is_some() {
        return Err(format!(
            "fidelity output directory must be empty: {}",
            root.display()
        )
        .into());
    }
    std::fs::create_dir_all(&root)?;
    Ok(root)
}

fn write_page_png(
    page: &katana_document_viewer::DocumentPageSurfaceFrame,
    path: &Path,
) -> TestResult {
    let image = RgbaImage::from_raw(page.width, page.height, page.rgba.clone())
        .ok_or("KDV page raster does not match its declared dimensions")?;
    image.save(path)?;
    Ok(())
}

fn capture_docx(output: &Path) -> TestResult<Value> {
    let output = output.join("docx");
    std::fs::create_dir_all(&output)?;
    let mut session = DocumentSession::open(
        office_source("representative.docx", OfficeDocumentFormat::Docx)?,
        config(DocumentViewport::new(842, 595)),
    )?;
    let mut frame = session.frame()?;
    let page_count = frame.state.item_count;
    if page_count == 0 {
        return Err("DOCX session returned no page labels".into());
    }
    let mut pages = Vec::with_capacity(page_count);
    for page_index in 0..page_count {
        if page_index > 0 {
            let _ = session.apply(DocumentSessionCommand::Viewer(DocumentViewerCommand::Next))?;
            frame = session.frame()?;
        }
        if frame.state.active_index != page_index {
            return Err(format!(
                "DOCX navigation did not reach page {page_index}: {}",
                frame.state.active_index
            )
            .into());
        }
        let page = frame
            .surface
            .page()
            .ok_or("DOCX frame did not expose a page surface")?;
        let file_name = format!("page-{page_index:04}.png");
        write_page_png(page, &output.join(&file_name))?;
        pages.push(json!({
            "index": page_index,
            "image": file_name,
            "width": page.width,
            "height": page.height,
            "display_width_milli": page.display_width_milli,
            "display_height_milli": page.display_height_milli,
            "content_scale": page.content_scale,
        }));
    }
    session.close();
    Ok(json!({"page_count": page_count, "pages": pages}))
}

fn grid_cell(cell: &katana_document_viewer::DocumentGridCell) -> Value {
    json!({
        "row": cell.coordinate.row,
        "column": cell.coordinate.column,
        "text": &cell.text,
        "bounds": {
            "x": cell.bounds.x,
            "y": cell.bounds.y,
            "width": cell.bounds.width,
            "height": cell.bounds.height,
        },
        "row_span": cell.row_span,
        "column_span": cell.column_span,
        "font": {
            "family": &cell.appearance.font_family,
            "size_px": cell.appearance.font_size_px,
            "text_color": &cell.appearance.text_color,
            "fill_color": &cell.appearance.fill_color,
            "bold": cell.appearance.bold,
            "italic": cell.appearance.italic,
            "underline": cell.appearance.underline,
            "strike": cell.appearance.strike,
            "horizontal_alignment": format!("{:?}", cell.appearance.horizontal_alignment),
            "vertical_alignment": format!("{:?}", cell.appearance.vertical_alignment),
            "wrap_text": cell.appearance.wrap_text,
        },
        "borders": {
            "left": border_side(cell.appearance.borders.left.as_ref()),
            "right": border_side(cell.appearance.borders.right.as_ref()),
            "top": border_side(cell.appearance.borders.top.as_ref()),
            "bottom": border_side(cell.appearance.borders.bottom.as_ref()),
        },
    })
}

fn border_side(side: Option<&katana_document_viewer::DocumentGridBorderSide>) -> Value {
    match side {
        Some(side) => json!({"style": &side.style, "color": &side.color}),
        None => Value::Null,
    }
}

fn capture_xlsx() -> TestResult<Value> {
    let mut session = DocumentSession::open(
        office_source("representative.xlsx", OfficeDocumentFormat::Xlsx)?,
        config(DocumentViewport::new(10_000, 10_000)),
    )?;
    let mut frame = session.frame()?;
    let sheet_count = frame.surface.item_labels().len();
    if sheet_count == 0 {
        return Err("XLSX session returned no sheet labels".into());
    }
    let mut sheets = Vec::with_capacity(sheet_count);
    for sheet_index in 0..sheet_count {
        if sheet_index > 0 {
            let _ = session.apply(DocumentSessionCommand::Viewer(DocumentViewerCommand::Next))?;
            frame = session.frame()?;
        }
        if frame.state.active_index != sheet_index {
            return Err(format!(
                "XLSX navigation did not reach sheet {sheet_index}: {}",
                frame.state.active_index
            )
            .into());
        }
        let grid = frame
            .surface
            .grid()
            .ok_or("XLSX frame did not expose a grid surface")?;
        let cells = grid.cells.iter().map(grid_cell).collect::<Vec<_>>();
        let merged_cell_count = grid
            .cells
            .iter()
            .filter(|cell| cell.row_span > 1 || cell.column_span > 1)
            .count();
        sheets.push(json!({
            "index": sheet_index,
            "label": frame.surface.item_labels()[sheet_index],
            "row_count": grid.row_count,
            "column_count": grid.column_count,
            "total_width": grid.total_width,
            "total_height": grid.total_height,
            "viewport": {
                "width": grid.viewport.width,
                "height": grid.viewport.height,
            },
            "merged_cell_count": merged_cell_count,
            "cells": cells,
        }));
    }
    session.close();
    Ok(json!({"sheet_count": sheet_count, "sheets": sheets}))
}

#[test]
#[ignore = "writes KDV candidate artifacts for the external Office fidelity harness"]
fn capture_representative_office_fidelity_candidate() -> TestResult {
    let root = output_root()?;
    let capture = json!({
        "schema_version": 1,
        "docx": capture_docx(&root)?,
        "xlsx": capture_xlsx()?,
    });
    std::fs::write(
        root.join("candidate.json"),
        serde_json::to_vec_pretty(&capture)?,
    )?;
    Ok(())
}
